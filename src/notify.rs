//! Server → client change notifications: the modern protocol's subscribe-and-notify pattern.
//!
//! `subscriptions/listen` opens a request-scoped SSE stream (`text/event-stream`). The first
//! message is always `notifications/subscriptions/acknowledged` (carrying the subscription id =
//! the listen request's JSON-RPC id); afterwards the stream carries
//! `notifications/resources/list_changed` and `notifications/resources/updated` as writes happen.
//!
//! Mechanics: the transport hands the request to a dedicated thread, which drains a channel fed by
//! the write hooks into the connection via [`serve_stream`]. During silent periods the stream emits
//! an SSE comment every [`KEEP_ALIVE_INTERVAL`] — keeping intermediaries from closing the idle
//! stream and doubling as the disconnect detector (a dead connection is noticed at the latest on
//! the next failed keep-alive write). No resumption (Last-Event-ID), no session ids: closing the
//! stream cancels the subscription.
//!
//! Write hooks run after the write transaction commits (same shape as `embed::after_write`, called
//! from both the MCP and REST faces). Notification vocabulary:
//! - `updated` = the read result of that exact URI will change;
//! - `list_changed` = the membership of `resources/list` changes.

use crate::model::RESERVED_TAG;
use crate::resources;
use crate::tools::{
    MEMORY_CREATE, MEMORY_DELETE, MEMORY_EDIT, MEMORY_MERGE, MEMORY_UPDATE, TAG_CREATE, TAG_DELETE,
    TAG_UPDATE,
};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;

/// Silence period between SSE keep-alive comments.
pub(crate) const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

const SUBSCRIPTION_ID_KEY: &str = "io.modelcontextprotocol/subscriptionId";

/// What a subscriber asked to be told about.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotifyFilter {
    pub tools_list_changed: bool,
    pub resources_list_changed: bool,
    pub resource_uris: Vec<String>,
}

impl NotifyFilter {
    fn matches(&self, event: &Event) -> bool {
        match event {
            Event::ListChanged => self.resources_list_changed,
            Event::Updated(uri) => self.resource_uris.iter().any(|u| u == uri),
        }
    }
}

/// One change worth notifying subscribers about.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    /// `resources/list` membership changed.
    ListChanged,
    /// The read result of this exact URI will change.
    Updated(String),
}

/// A validated `subscriptions/listen` request, handed from the protocol layer to the transport.
pub struct ListenJob {
    /// JSON-RPC id of the listen request = the subscription id echoed on every notification.
    pub client_id: Value,
    /// Whether the caller holds the read capability (mask semantics apply without it).
    pub can_read: bool,
    pub filter: NotifyFilter,
}

struct Subscription {
    /// The client's JSON-RPC id for the listen request — echoed as the subscription id.
    client_id: Value,
    filter: NotifyFilter,
    tx: mpsc::Sender<Vec<u8>>,
}

/// Process-wide subscription registry: token → subscription. Tokens decouple unregistration from
/// the client id, so a client re-using a JSON-RPC id for a second subscription cannot make one
/// stream's cleanup cancel the other.
fn registry() -> &'static Mutex<(u64, HashMap<u64, Subscription>)> {
    static REGISTRY: OnceLock<Mutex<(u64, HashMap<u64, Subscription>)>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new((0, HashMap::new())))
}

/// Open a subscription stream: queue the acknowledgment and register the subscriber. Returns the
/// receiving end the transport must drain into the response (see [`serve_stream`]) plus a handle
/// whose drop unregisters (the transport holds it for the stream's lifetime, covering client
/// disconnects).
pub fn open(job: ListenJob) -> (mpsc::Receiver<Vec<u8>>, SubscriptionHandle) {
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    // Effective filter: notification types we cannot honor are omitted from the ack. toolsListChanged
    // is never honored (the tool surface is static); without the read capability the subscription is
    // empty (mask semantics — same as resources/list returning an empty catalog).
    let effective = NotifyFilter {
        tools_list_changed: false,
        resources_list_changed: job.filter.resources_list_changed && job.can_read,
        resource_uris: if job.can_read {
            job.filter.resource_uris.clone()
        } else {
            Vec::new()
        },
    };
    let client_id = job.client_id;
    let ack = json!({
        "jsonrpc": "2.0",
        "method": "notifications/subscriptions/acknowledged",
        "params": {
            "_meta": { SUBSCRIPTION_ID_KEY: &client_id },
            "notifications": acknowledged_notifications(&effective),
        }
    });
    tx.send(sse_event(&ack)).ok(); // the receiver is held by us; sending cannot fail here

    let mut registry = registry().lock().expect("subscription registry poisoned");
    let token = registry.0;
    registry.0 += 1;
    registry.1.insert(
        token,
        Subscription {
            client_id,
            filter: effective,
            tx,
        },
    );
    (rx, SubscriptionHandle { token })
}

/// The acknowledgment's `notifications` field: only the types the server actually agreed to honor.
fn acknowledged_notifications(filter: &NotifyFilter) -> Value {
    let mut out = Map::new();
    if filter.resources_list_changed {
        out.insert("resourcesListChanged".into(), json!(true));
    }
    if !filter.resource_uris.is_empty() {
        out.insert("resourceSubscriptions".into(), json!(filter.resource_uris));
    }
    Value::Object(out)
}

/// Unregisters the subscription when dropped (stream ended: client cancel, disconnect or shutdown).
pub struct SubscriptionHandle {
    token: u64,
}

impl Drop for SubscriptionHandle {
    fn drop(&mut self) {
        registry()
            .lock()
            .expect("subscription registry poisoned")
            .1
            .remove(&self.token);
    }
}

/// Encode one JSON-RPC message as an SSE event (the transport's Streamable-HTTP event shape).
fn sse_event(message: &Value) -> Vec<u8> {
    format!(
        "event: message\ndata: {}\n\n",
        serde_json::to_string(message).unwrap_or_else(|_| "{}".to_string())
    )
    .into_bytes()
}

/// Serve one subscription's response stream on the raw connection writer (obtained via
/// `Request::into_writer`, the library's escape hatch for hand-written responses — its response
/// buffering only flushes when the body finishes, which never happens for an endless stream).
///
/// The head is written once; then the loop drains the subscription's channel, flushing after every
/// message so notifications arrive immediately. Silence periods emit an SSE comment (lines starting
/// with a colon carry no event data) that keeps intermediaries from closing the idle stream and
/// doubles as the disconnect detector — the failing flush of a dead connection ends the loop.
/// The stream ends when the channel's senders are dropped (subscription unregistered) or on the
/// first write error; the connection is close-delimited (`Connection: close`), so ending the loop
/// and dropping the writer is the whole termination protocol.
pub(crate) fn serve_stream<W: std::io::Write>(
    writer: &mut W,
    rx: &mpsc::Receiver<Vec<u8>>,
    keep_alive: Duration,
) {
    const HEAD: &[u8] = b"HTTP/1.1 200 OK\r\n\
        Content-Type: text/event-stream\r\n\
        Cache-Control: no-store\r\n\
        X-Accel-Buffering: no\r\n\
        Connection: close\r\n\
        \r\n";
    const KEEP_ALIVE_LINE: &[u8] = b": keep-alive\n\n";
    if writer.write_all(HEAD).and_then(|_| writer.flush()).is_err() {
        return;
    }
    loop {
        let message = match rx.recv_timeout(keep_alive) {
            Ok(bytes) => bytes,
            // SSE comment: clients must ignore it, intermediaries keep the connection warm
            Err(mpsc::RecvTimeoutError::Timeout) => KEEP_ALIVE_LINE.to_vec(),
            // All senders gone: the subscription was unregistered — graceful end of stream
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        };
        if writer
            .write_all(&message)
            .and_then(|_| writer.flush())
            .is_err()
        {
            return; // client disconnected (noticed at the latest on a keep-alive write)
        }
    }
}

/// Capture the pre-write state the notification hooks need (called OUTSIDE any transaction, before
/// the write begins — once the write commits the old state is unrecoverable). Only the tools whose
/// diff needs the previous tags read anything; everything else costs nothing.
#[derive(Default)]
pub struct PreState {
    /// memory_update: the memory's tags before the change.
    old_tags: Vec<String>,
    /// memory_delete: tags of each memory about to be deleted, keyed by id ("m3").
    tags_by_id: HashMap<String, Vec<String>>,
}

pub fn capture(db_path: &Path, tool: &str, args: &Value) -> PreState {
    let ids: Vec<String> = match tool {
        MEMORY_UPDATE => vec![args["id"].as_str().unwrap_or_default().to_string()],
        // Merge diffs the target's tags (first id) and needs the source's tags for its removal events
        MEMORY_MERGE => vec![
            args["target"].as_str().unwrap_or_default().to_string(),
            args["source"].as_str().unwrap_or_default().to_string(),
        ],
        MEMORY_DELETE => args["ids"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        _ => return PreState::default(),
    };
    let numeric: Vec<i64> = ids
        .iter()
        .filter_map(|i| crate::store::Store::parse_id(i))
        .collect();
    let state = crate::store::with_db_in(db_path, crate::store::TxMode::ReadOnly, |st| {
        let (found, _) = st.get_memories(&numeric)?;
        let mut tags_by_id = HashMap::new();
        for m in &found {
            tags_by_id.insert(m.id.clone(), m.tags.clone());
        }
        Ok::<_, String>(PreState {
            old_tags: tags_by_id
                .get(ids.first().map(String::as_str).unwrap_or_default())
                .cloned()
                .unwrap_or_default(),
            tags_by_id,
        })
    });
    state.unwrap_or_default()
}

/// Fire the notifications a completed write implies (called after the transaction commits; failures
/// are silent — notifications are best-effort companions to the write itself).
pub fn after_write(tool: &str, args: &Value, result: &Value, pre: &PreState) {
    let events = compute_events(tool, args, result, pre);
    if events.is_empty() {
        return;
    }
    let mut registry = registry().lock().expect("subscription registry poisoned");
    let mut dead: Vec<u64> = Vec::new();
    for (token, sub) in registry.1.iter() {
        for event in &events {
            if !sub.filter.matches(event) {
                continue;
            }
            let message = match event {
                Event::ListChanged => json!({
                    "jsonrpc": "2.0",
                    "method": "notifications/resources/list_changed",
                    "params": { "_meta": { SUBSCRIPTION_ID_KEY: &sub.client_id } },
                }),
                Event::Updated(uri) => json!({
                    "jsonrpc": "2.0",
                    "method": "notifications/resources/updated",
                    "params": {
                        "uri": uri,
                        "_meta": { SUBSCRIPTION_ID_KEY: &sub.client_id },
                    },
                }),
            };
            if sub.tx.send(sse_event(&message)).is_err() {
                dead.push(*token);
            }
        }
    }
    for token in dead {
        registry.1.remove(&token);
    }
}

/// Compute the change events a completed write implies. The rules mirror the resource surface:
/// list_changed only when the catalog's membership changes, updated only where the read result of
/// that exact URI changes.
fn compute_events(tool: &str, args: &Value, result: &Value, pre: &PreState) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();
    fn push(events: &mut Vec<Event>, event: Event) {
        if !events.contains(&event) {
            events.push(event);
        }
    }
    match tool {
        TAG_CREATE => push(&mut events, Event::ListChanged),
        TAG_UPDATE => {
            let renamed = result["renamed"] == true;
            let description_updated = result["description_updated"] == true;
            let final_name = result["name"].as_str().unwrap_or_default();
            if renamed {
                push(&mut events, Event::ListChanged);
                push(
                    &mut events,
                    Event::Updated(resources::tag_resource_uri(
                        args["name"].as_str().unwrap_or_default(),
                    )),
                );
                push(
                    &mut events,
                    Event::Updated(resources::tag_resource_uri(final_name)),
                );
            } else if description_updated {
                // A description lands in the tag resource's body (not the catalog membership)
                push(
                    &mut events,
                    Event::Updated(resources::tag_resource_uri(final_name)),
                );
            }
        }
        TAG_DELETE => {
            push(&mut events, Event::ListChanged);
            push(
                &mut events,
                Event::Updated(resources::tag_resource_uri(
                    args["name"].as_str().unwrap_or_default(),
                )),
            );
        }
        MEMORY_CREATE => {
            let autocreated = result["tags_autocreated"]
                .as_array()
                .is_some_and(|a| !a.is_empty());
            if autocreated {
                push(&mut events, Event::ListChanged);
            }
            // Every tag the memory lands on gets a richer catalog, so its resource
            // read result changes (the updated vocabulary: the read result of that exact uri).
            for tag in memory_tags(result) {
                push(
                    &mut events,
                    Event::Updated(resources::tag_resource_uri(&tag)),
                );
            }
            if memory_tags(result).iter().any(|t| t == RESERVED_TAG) {
                push(&mut events, Event::ListChanged);
                push(
                    &mut events,
                    Event::Updated(resources::memory_resource_uri(
                        result["memory"]["id"].as_str().unwrap_or_default(),
                    )),
                );
            }
        }
        MEMORY_UPDATE => {
            let id = result["memory"]["id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            if args.get("summary").is_some() || args.get("content").is_some() {
                push(
                    &mut events,
                    Event::Updated(resources::memory_resource_uri(&id)),
                );
            }
            let final_tags = memory_tags(result);
            for tag in &final_tags {
                if !pre.old_tags.contains(tag) {
                    push(
                        &mut events,
                        Event::Updated(resources::tag_resource_uri(tag)),
                    );
                }
            }
            for tag in &pre.old_tags {
                if !final_tags.contains(tag) {
                    push(
                        &mut events,
                        Event::Updated(resources::tag_resource_uri(tag)),
                    );
                }
            }
            let had_conventions = pre.old_tags.iter().any(|t| t == RESERVED_TAG);
            let has_conventions = final_tags.iter().any(|t| t == RESERVED_TAG);
            if had_conventions != has_conventions {
                push(&mut events, Event::ListChanged);
            }
        }
        MEMORY_EDIT => {
            // Content replaced in place: only that memory resource's read result moves (tags are
            // untouched, so no catalog nor membership event)
            let id = result["memory"]["id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            push(
                &mut events,
                Event::Updated(resources::memory_resource_uri(&id)),
            );
        }
        MEMORY_MERGE => {
            // The target is rewritten (merged content, possibly summary) and gains the source's
            // tags; the source is deleted outright. Same event shapes as memory_update on the
            // target plus memory_delete on the source.
            let id = result["memory"]["id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            push(
                &mut events,
                Event::Updated(resources::memory_resource_uri(&id)),
            );
            let final_tags = memory_tags(result);
            for tag in &final_tags {
                if !pre.old_tags.contains(tag) {
                    push(
                        &mut events,
                        Event::Updated(resources::tag_resource_uri(tag)),
                    );
                }
            }
            if final_tags.iter().any(|t| t == RESERVED_TAG)
                && !pre.old_tags.iter().any(|t| t == RESERVED_TAG)
            {
                // The resident-conventions set gained a member
                push(&mut events, Event::ListChanged);
            }
            if let Some(removed) = result["removed"].as_str() {
                push(
                    &mut events,
                    Event::Updated(resources::memory_resource_uri(removed)),
                );
                if let Some(source_tags) = pre.tags_by_id.get(removed) {
                    for tag in source_tags {
                        // The source's entry leaves every catalog it appeared in (the target may
                        // still carry the tag, but the catalog's content changed either way)
                        push(
                            &mut events,
                            Event::Updated(resources::tag_resource_uri(tag)),
                        );
                        if tag == RESERVED_TAG {
                            // ...and the resident-conventions set lost a member
                            push(&mut events, Event::ListChanged);
                        }
                    }
                }
            }
        }
        MEMORY_DELETE => {
            let mut affected_tags: Vec<String> = Vec::new();
            let mut conventions_gone = false;
            for id in result["deleted"]
                .as_array()
                .map(|a| a.as_slice())
                .unwrap_or(&[])
            {
                push(
                    &mut events,
                    Event::Updated(resources::memory_resource_uri(
                        id.as_str().unwrap_or_default(),
                    )),
                );
                if let Some(tags) = id.as_str().and_then(|i| pre.tags_by_id.get(i)) {
                    for tag in tags {
                        if tag == RESERVED_TAG {
                            conventions_gone = true;
                        }
                        if !affected_tags.contains(tag) {
                            affected_tags.push(tag.clone());
                        }
                    }
                }
            }
            for tag in affected_tags {
                push(
                    &mut events,
                    Event::Updated(resources::tag_resource_uri(&tag)),
                );
            }
            if conventions_gone {
                push(&mut events, Event::ListChanged);
            }
        }
        _ => {}
    }
    events
}

fn memory_tags(result: &Value) -> Vec<String> {
    result["memory"]["tags"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn events_for(tool: &str, args: Value, result: Value, pre: &PreState) -> Vec<Event> {
        compute_events(tool, &args, &result, pre)
    }

    #[test]
    fn tag_events_follow_the_catalog() {
        // tag_create: membership grows
        assert_eq!(
            events_for(
                "tag_create",
                json!({}),
                json!({"created": true}),
                &PreState::default()
            ),
            vec![Event::ListChanged]
        );
        // tag_delete: membership shrinks, the tag resource disappears
        assert_eq!(
            events_for(
                "tag_delete",
                json!({"name": "rust"}),
                json!({}),
                &PreState::default()
            ),
            vec![
                Event::ListChanged,
                Event::Updated("memory://tags/rust".into())
            ]
        );
        // tag_update rename: membership changes (uri changes) + old and new resources move
        let evs = events_for(
            "tag_update",
            json!({"name": "rust"}),
            json!({"renamed": true, "description_updated": false, "name": "lang"}),
            &PreState::default(),
        );
        assert_eq!(
            evs,
            vec![
                Event::ListChanged,
                Event::Updated("memory://tags/rust".into()),
                Event::Updated("memory://tags/lang".into())
            ]
        );
        // tag_update description-only: the resource body changes, membership does not
        assert_eq!(
            events_for(
                "tag_update",
                json!({"name": "rust"}),
                json!({"renamed": false, "description_updated": true, "name": "rust"}),
                &PreState::default()
            ),
            vec![Event::Updated("memory://tags/rust".into())]
        );
        // Percent-encoding of non-ASCII tag names in uris
        let evs = events_for(
            "tag_create",
            json!({}),
            json!({"tag": {"name": "项目"}}),
            &PreState::default(),
        );
        assert_eq!(evs, vec![Event::ListChanged]);
    }

    #[test]
    fn memory_create_events() {
        // New tags change the catalog; plain reuses change nothing
        assert_eq!(
            events_for(
                "memory_create",
                json!({}),
                json!({"memory": {"id": "m1", "tags": ["a"]}, "tags_autocreated": ["a"], "tags_reused": []}),
                &PreState::default(),
            ),
            vec![Event::ListChanged, Event::Updated("memory://tags/a".into())]
        );
        // Even a reused tag's resource read result changes (its catalog grew)
        let evs = events_for(
            "memory_create",
            json!({}),
            json!({"memory": {"id": "m1", "tags": ["a"]}, "tags_autocreated": [], "tags_reused": ["a"]}),
            &PreState::default(),
        );
        assert_eq!(evs, vec![Event::Updated("memory://tags/a".into())]);
        // Conventions member: the conventions tag resource updated, catalog changes
        // AND the memory resource is born
        let evs = events_for(
            "memory_create",
            json!({}),
            json!({"memory": {"id": "m2", "tags": ["conventions"]}, "tags_autocreated": [], "tags_reused": ["conventions"]}),
            &PreState::default(),
        );
        assert_eq!(
            evs,
            vec![
                Event::Updated("memory://tags/conventions".into()),
                Event::ListChanged,
                Event::Updated("memory://memories/m2".into())
            ]
        );
    }

    #[test]
    fn memory_update_events_use_the_tag_diff() {
        let pre = PreState {
            old_tags: vec!["a".into(), "conventions".into()],
            tags_by_id: HashMap::new(),
        };
        // Body change + tags a→b: memory resource updated, both tag resources updated, conventions
        // membership dropped → list_changed
        let evs = events_for(
            "memory_update",
            json!({"id": "m1", "content": "new body"}),
            json!({"memory": {"id": "m1", "tags": ["b"]}}),
            &pre,
        );
        assert_eq!(
            evs,
            vec![
                Event::Updated("memory://memories/m1".into()),
                Event::Updated("memory://tags/b".into()),
                Event::Updated("memory://tags/a".into()),
                Event::Updated("memory://tags/conventions".into()),
                Event::ListChanged
            ]
        );
        // Tag-only change, no conventions involved: no memory-resource update, no list_changed
        let pre = PreState {
            old_tags: vec!["a".into()],
            tags_by_id: HashMap::new(),
        };
        let evs = events_for(
            "memory_update",
            json!({"id": "m1", "add_tags": ["b"]}),
            json!({"memory": {"id": "m1", "tags": ["a", "b"]}}),
            &pre,
        );
        assert_eq!(evs, vec![Event::Updated("memory://tags/b".into())]);
        // No-op updates notify nobody
        let evs = events_for(
            "memory_update",
            json!({"id": "m1"}),
            json!({"updated": false, "memory": {"id": "m1", "tags": ["a"]}}),
            &pre,
        );
        assert!(evs.is_empty());
    }

    #[test]
    fn memory_edit_events_cover_the_memory_resource_only() {
        // Content replaced in place; tags are untouched, so no tag-catalog nor membership event
        let evs = events_for(
            "memory_edit",
            json!({"id": "m1", "old_string": "a", "new_string": "b"}),
            json!({"replaced": 1, "memory": {"id": "m1", "tags": ["a"]}}),
            &PreState::default(),
        );
        assert_eq!(evs, vec![Event::Updated("memory://memories/m1".into())]);
    }

    #[test]
    fn memory_delete_events_cover_memories_and_affected_tags() {
        let pre = PreState {
            old_tags: vec![],
            tags_by_id: HashMap::from([
                ("m1".into(), vec!["a".into(), "shared".into()]),
                ("m2".into(), vec!["shared".into(), "conventions".into()]),
            ]),
        };
        let evs = events_for(
            "memory_delete",
            json!({"ids": ["m1", "m2"]}),
            json!({"deleted": ["m1", "m2"], "missing": []}),
            &pre,
        );
        assert_eq!(
            evs,
            vec![
                Event::Updated("memory://memories/m1".into()),
                Event::Updated("memory://memories/m2".into()),
                Event::Updated("memory://tags/a".into()),
                Event::Updated("memory://tags/shared".into()),
                Event::Updated("memory://tags/conventions".into()),
                Event::ListChanged
            ]
        );
    }

    #[test]
    fn memory_merge_events_cover_target_rewrite_and_source_removal() {
        let pre = PreState {
            old_tags: vec!["a".into()],
            tags_by_id: HashMap::from([
                ("m1".into(), vec!["a".into()]),
                ("m2".into(), vec!["b".into(), "conventions".into()]),
            ]),
        };
        // Target m1 gains tag b (its content is merged); source m2 (tagged b + conventions) dies:
        // its memory resource goes, its tags' catalogs shrink, and the resident-conventions set
        // loses a member (list_changed)
        let evs = events_for(
            "memory_merge",
            json!({"target": "m1", "source": "m2"}),
            json!({"merged": true, "memory": {"id": "m1", "tags": ["a", "b"]}, "removed": "m2"}),
            &pre,
        );
        assert_eq!(
            evs,
            vec![
                Event::Updated("memory://memories/m1".into()),
                Event::Updated("memory://tags/b".into()),
                Event::Updated("memory://memories/m2".into()),
                Event::Updated("memory://tags/conventions".into()),
                Event::ListChanged
            ]
        );
    }

    #[test]
    fn open_acknowledges_the_supported_subset() {
        // Full filter with read capability: both supported types acknowledged, toolsListChanged dropped
        let job = ListenJob {
            client_id: json!(7),
            can_read: true,
            filter: NotifyFilter {
                tools_list_changed: true,
                resources_list_changed: true,
                resource_uris: vec!["memory://tags/rust".into()],
            },
        };
        let (rx, _handle) = open(job);
        let ack = read_message(&rx);
        assert_eq!(ack["method"], "notifications/subscriptions/acknowledged");
        assert_eq!(
            ack["params"]["_meta"]["io.modelcontextprotocol/subscriptionId"],
            7
        );
        assert_eq!(ack["params"]["notifications"]["resourcesListChanged"], true);
        assert_eq!(
            ack["params"]["notifications"]["resourceSubscriptions"],
            json!(["memory://tags/rust"])
        );
        assert!(ack["params"]["notifications"]
            .get("toolsListChanged")
            .is_none());
    }

    #[test]
    fn open_without_read_capability_acknowledges_nothing() {
        let job = ListenJob {
            client_id: json!("sub-1"),
            can_read: false,
            filter: NotifyFilter {
                tools_list_changed: false,
                resources_list_changed: true,
                resource_uris: vec!["memory://memories/m1".into()],
            },
        };
        let (rx, _handle) = open(job);
        let ack = read_message(&rx);
        assert_eq!(
            ack["params"]["_meta"]["io.modelcontextprotocol/subscriptionId"],
            "sub-1"
        );
        assert_eq!(ack["params"]["notifications"], json!({}));
    }

    #[test]
    fn dispatch_reaches_matching_subscribers_only() {
        // A unique tag name keeps parallel tests' writes (which also fire list_changed through
        // the shared registry) out of this test's assertions: only our own writes can match the
        // watched-uri filter, and reads skip any interleaved foreign list_changed noise.
        let watched = format!("watched-{}", std::process::id());
        let watched_uri = resources::tag_resource_uri(&watched);
        let job = ListenJob {
            client_id: json!(1),
            can_read: true,
            filter: NotifyFilter {
                tools_list_changed: false,
                resources_list_changed: true,
                resource_uris: vec![watched_uri.clone()],
            },
        };
        let (rx, _handle) = open(job);
        let _ack = read_message(&rx);

        // A tag_create reaches the list_changed subscriber
        after_write(
            "tag_create",
            &json!({}),
            &json!({"created": true}),
            &PreState::default(),
        );
        let message = read_until(&rx, "notifications/resources/list_changed");
        assert_eq!(
            message["params"]["_meta"]["io.modelcontextprotocol/subscriptionId"],
            1
        );
        // Deleting an unwatched tag produces only a list_changed (skipped by the read-until);
        // deleting the watched tag delivers the updated event with its uri
        after_write(
            "tag_delete",
            &json!({"name": "unwatched"}),
            &json!({}),
            &PreState::default(),
        );
        after_write(
            "tag_delete",
            &json!({"name": watched}),
            &json!({}),
            &PreState::default(),
        );
        let message = read_until(&rx, "notifications/resources/updated");
        assert_eq!(message["params"]["uri"], watched_uri);
        // Cleanup so later tests start with an empty registry (drop _handle)
        drop(_handle);
    }

    #[test]
    fn serve_stream_flushes_events_and_keep_alives_then_ends() {
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        tx.send(sse_event(&json!({"a": 1}))).unwrap();
        // The channel stays open while serve_stream runs; the short keep-alive exercises the
        // silence path. Dropping the sender unregisters the stream end (graceful close).
        let stream_tx = std::thread::spawn(move || {
            let mut sink: Vec<u8> = Vec::new();
            serve_stream(&mut sink, &rx, Duration::from_millis(40));
            sink
        });
        std::thread::sleep(Duration::from_millis(200));
        drop(tx);
        let out = stream_tx.join().unwrap();

        let text = String::from_utf8_lossy(&out).into_owned();
        // The response head comes first, close-delimited
        assert!(
            text.starts_with("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n"),
            "head first: {text}"
        );
        assert!(text.contains("Connection: close\r\n\r\n"), "{text}");
        // The queued event was flushed out
        assert!(
            text.contains("event: message\ndata: {\"a\":1}\n\n"),
            "{text}"
        );
        // Silence periods produced keep-alive comments
        assert!(
            text.matches(": keep-alive\n\n").count() >= 2,
            "keep-alives during silence: {text}"
        );
    }

    /// Pull one complete SSE message out of the channel (the tests' stand-in for an SSE client).
    fn read_message(rx: &mpsc::Receiver<Vec<u8>>) -> Value {
        let bytes = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("a queued SSE message");
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let data = text
            .strip_prefix("event: message\ndata: ")
            .and_then(|d| d.strip_suffix("\n\n"))
            .expect("SSE event shape");
        serde_json::from_str(data).unwrap()
    }

    /// Like `read_message`, but skips messages until one with the expected method arrives
    /// (parallel tests share the process-wide registry, so foreign list_changed noise can interleave).
    fn read_until(rx: &mpsc::Receiver<Vec<u8>>, method: &str) -> Value {
        loop {
            let message = read_message(rx);
            if message["method"] == method {
                return message;
            }
        }
    }
}
