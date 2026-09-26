//! 语义搜索：embedding 服务调用、向量编解码与混合排序。
//!
//! 回退是本模块的硬性原则——embedding 服务不可用只允许造成"降级"，不允许
//! 造成"失败"：搜索路径超时/报错即回退纯关键词；写入路径只留待补跑，
//! 记忆本体已保存；配置缺失时所有入口都是零开销直通。
//!
//! 网络调用一律在数据库事务之外（调用方保证）：写事务持 IMMEDIATE 锁跨
//! 网络 I/O 是禁止的死锁形态，只读快照拉长事务也无必要。

use crate::search::{self, Hit};
use crate::store::{Store, TxMode};
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

/// embedding 服务配置（OpenAI 兼容 `/embeddings` 端点：云 API 与
/// Ollama/LM Studio 等本地服务共用同一形态）。
#[derive(Clone, Debug)]
pub struct EmbedConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl EmbedConfig {
    fn endpoint(&self) -> String {
        format!("{}/embeddings", self.base_url.trim_end_matches('/'))
    }
}

/// 查询向量化（搜索路径）超时：到点即放弃，回退关键词搜索。
pub const QUERY_TIMEOUT: Duration = Duration::from_secs(3);
/// 批量向量化（写入挂接 / 补跑）超时。
pub const BATCH_TIMEOUT: Duration = Duration::from_secs(30);
/// 单次请求携带的文本条数上限（对供应商批量限额的保守取值）。
pub const MAX_BATCH: usize = 16;
/// 单条文本送入 embedding 服务前的字符截断长度。bge-m3 的 8K token 窗口
/// 约合 8K 中文字符；512 token 的小模型由服务端自行截断，这里只防极端
/// 长文撑爆请求体。
const MAX_INPUT_CHARS: usize = 8_000;
/// RRF（倒数排名融合）常数：名次越靠前贡献越大，K 抹平头部权重差。
const RRF_K: f64 = 60.0;

/// 送入 embedding 服务的记忆文本：摘要 + 空行 + 正文（截断）。
pub fn embed_memory_text(summary: &str, content: &str) -> String {
    let mut text = String::new();
    text.push_str(summary.trim());
    if !content.trim().is_empty() {
        text.push_str("\n\n");
        text.push_str(content.trim());
    }
    if text.chars().count() > MAX_INPUT_CHARS {
        text = text.chars().take(MAX_INPUT_CHARS).collect();
    }
    text
}

/// 批量向量化。任何网络/解析失败都以 Err 返回，由调用方决定降级方式。
pub fn embed_texts(
    cfg: &EmbedConfig,
    texts: &[String],
    timeout: Duration,
) -> Result<Vec<Vec<f32>>, String> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let mut req = ureq::post(&cfg.endpoint()).timeout(timeout);
    if let Some(key) = &cfg.api_key {
        req = req.set("Authorization", &format!("Bearer {key}"));
    }
    let resp = req
        .send_json(json!({ "model": cfg.model, "input": texts }))
        .map_err(|e| format!("embedding request failed: {e}"))?;
    let status = resp.status();
    let body: Value = resp
        .into_json()
        .map_err(|e| format!("embedding response is not valid JSON: {e}"))?;
    if status != 200 {
        let detail = body["error"]["message"]
            .as_str()
            .or_else(|| body["message"].as_str())
            .unwrap_or("unknown error");
        return Err(format!("embedding service returned {status}: {detail}"));
    }
    let mut data = body["data"]
        .as_array()
        .ok_or("embedding response missing 'data' array")?
        .clone();
    data.sort_by_key(|item| item["index"].as_u64().unwrap_or(0));
    if data.len() != texts.len() {
        return Err(format!(
            "embedding service returned {} vectors for {} inputs",
            data.len(),
            texts.len()
        ));
    }
    data.into_iter()
        .map(|item| {
            item["embedding"]
                .as_array()
                .ok_or("embedding entry missing 'embedding' array")?
                .iter()
                .map(|n| {
                    n.as_f64()
                        .map(|f| f as f32)
                        .ok_or_else(|| "embedding contains a non-number".to_string())
                })
                .collect()
        })
        .collect()
}

/// f32 向量 → BLOB（小端字节序；SQLite 无向量类型，跨平台字节序由绑定保证）。
pub fn vec_to_blob(vec: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(vec.len() * 4);
    for v in vec {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// BLOB → f32 向量；长度非 4 的倍数时丢弃尾部残字节（损坏行按无向量处理）。
pub fn blob_to_vec(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// 余弦相似度；长度不等或零向量返回 0（即不会进入向量召回）。
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (mut dot, mut na, mut nb) = (0f32, 0f32, 0f32);
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

/// 补跑一批的结果。NotConfigured 与 Failed 的区别决定调用方行为：
/// 写入路径对两者都静默（Failed 记 stderr），管理端点把细节带给 UI。
pub enum EmbedOutcome {
    NotConfigured,
    Processed { processed: usize, remaining: usize },
    Failed(String),
}

impl EmbedOutcome {
    pub fn to_json(&self) -> Value {
        match self {
            EmbedOutcome::NotConfigured => json!({ "configured": false }),
            EmbedOutcome::Processed {
                processed,
                remaining,
            } => json!({
                "configured": true,
                "processed": processed,
                "remaining": remaining,
            }),
            EmbedOutcome::Failed(e) => json!({ "configured": true, "error": e }),
        }
    }
}

/// 为缺向量（或向量模型过期）的记忆补跑一批：读配置与待补清单（只读事务）
/// → 调 embedding 服务（事务外）→ 写向量（独立写事务）。
pub fn process_pending(db_path: &Path, batch: usize) -> EmbedOutcome {
    let batch = batch.clamp(1, MAX_BATCH);
    let (cfg, pending) = match crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| {
        let Some(cfg) = st.embedding_config()? else {
            return Ok(None);
        };
        let pending = st.embedding_pending_batch(&cfg.model, batch)?;
        Ok::<_, String>(Some((cfg, pending)))
    }) {
        Ok(Some(pair)) => pair,
        Ok(None) => return EmbedOutcome::NotConfigured,
        Err(e) => return EmbedOutcome::Failed(format!("cannot read database: {e}")),
    };
    if pending.is_empty() {
        return EmbedOutcome::Processed {
            processed: 0,
            remaining: 0,
        };
    }

    let texts: Vec<String> = pending
        .iter()
        .map(|(_, summary, content)| embed_memory_text(summary, content))
        .collect();
    let vectors = match embed_texts(&cfg, &texts, BATCH_TIMEOUT) {
        Ok(v) => v,
        Err(e) => return EmbedOutcome::Failed(e),
    };

    if let Err(e) = crate::store::with_db_in(db_path, TxMode::Write, |st| {
        for ((id, _, _), vec) in pending.iter().zip(&vectors) {
            st.embedding_put(*id, &cfg.model, vec)?;
        }
        Ok::<(), String>(())
    }) {
        return EmbedOutcome::Failed(format!("cannot store embeddings: {e}"));
    }

    match crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| {
        st.embedding_pending_count(&cfg.model)
    }) {
        Ok(remaining) => EmbedOutcome::Processed {
            processed: pending.len(),
            remaining,
        },
        Err(e) => EmbedOutcome::Failed(format!("cannot count pending: {e}")),
    }
}

/// 写入路径的挂接点（memory_create / memory_update 事务提交后调用）。
/// embedding 服务不可用时只记 stderr：记忆已保存，向量留给补跑。
pub fn after_write(db_path: &Path) {
    if let EmbedOutcome::Failed(e) = process_pending(db_path, 4) {
        eprintln!(
            "embedding backfill after write failed (the memory itself is saved; it will be retried by the next backfill): {e}"
        );
    }
}

/// 关键词 + 向量两路召回的 RRF 融合排序。
///
/// 两路各自按名次贡献 `1/(K+rank)`，天然回避"关键词 TF 分"与"余弦值"
/// 的量纲不可比问题。向量独有命中（关键词零命中、但语义相近）由本函数
/// 引入——这正是语义搜索存在的意义；其片段走正文开头的兜底路径。
pub fn hybrid_hits(
    memories: &[crate::model::Memory],
    keyword_hits: Vec<Hit>,
    table: &HashMap<i64, Vec<f32>>,
    query_vec: &[f32],
    tag_filter: &[String],
    tag_regex: Option<&Regex>,
) -> Vec<Hit> {
    // 向量趟：过同一套标签过滤（与关键词趟的召回语义一致），按余弦降序排名
    let mut vector_ranked: Vec<(usize, f32)> = memories
        .iter()
        .enumerate()
        .filter(|(_, m)| search::passes_tag_filters(m, tag_filter, tag_regex))
        .filter_map(|(idx, m)| {
            let id = Store::parse_id(&m.id).unwrap_or(0);
            table.get(&id).map(|v| (idx, cosine(query_vec, v)))
        })
        .filter(|(_, score)| *score > 0.0)
        .collect();
    vector_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut fused: HashMap<usize, f64> = HashMap::new();
    let mut keyword_snippets: HashMap<usize, &str> = HashMap::new();
    for (rank, hit) in keyword_hits.iter().enumerate() {
        *fused.entry(hit.idx).or_default() += 1.0 / (RRF_K + rank as f64);
        keyword_snippets.insert(hit.idx, hit.snippet.as_str());
    }
    for (rank, (idx, _)) in vector_ranked.iter().enumerate() {
        *fused.entry(*idx).or_default() += 1.0 / (RRF_K + rank as f64);
    }

    let mut items: Vec<(usize, f64)> = fused.into_iter().collect();
    // 与关键词趟同一tie-break：分数 → updated_at 降序 → id 升序
    items.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(memories[b.0].updated_at.cmp(&memories[a.0].updated_at))
            .then(memories[a.0].id.cmp(&memories[b.0].id))
    });

    items
        .into_iter()
        .map(|(idx, score)| {
            // RRF 值微小（<0.033），放大 1e6 转整型只保序不保真——score 字段
            // 本来就是排 audio 序的内部值，不承诺可解释
            let score = (score * 1_000_000.0).round() as i64;
            let snippet = match keyword_snippets.get(&idx) {
                Some(s) => (*s).to_string(),
                None => search::fallback_snippet(&memories[idx].content),
            };
            Hit {
                idx,
                score,
                snippet,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Memory;

    fn mem(id: i64, summary: &str, content: &str) -> Memory {
        Memory {
            id: Store::format_id(id),
            summary: summary.to_string(),
            content: content.to_string(),
            tags: Vec::new(),
            created_at: 1,
            updated_at: 1,
        }
    }

    fn unit(dim: usize, hot: usize) -> Vec<f32> {
        (0..dim).map(|i| if i == hot { 1.0 } else { 0.0 }).collect()
    }

    #[test]
    fn blob_roundtrip() {
        let v = vec![0.25f32, -1.5, 3.0e-7, f32::MIN];
        let back = blob_to_vec(&vec_to_blob(&v));
        assert_eq!(v, back);
        // 损坏行：残字节被丢弃而不是 panic
        let mut blob = vec_to_blob(&v);
        blob.push(0xAA);
        assert_eq!(blob_to_vec(&blob).len(), 4);
    }

    #[test]
    fn cosine_basics() {
        let a = unit(3, 0);
        assert!((cosine(&a, &a) - 1.0).abs() < 1e-6);
        assert!(cosine(&a, &unit(3, 1)).abs() < 1e-6);
        assert_eq!(cosine(&a, &unit(4, 0)), 0.0, "维度不等 = 0，不得 panic");
        assert_eq!(cosine(&[0.0, 0.0], &[0.0, 0.0]), 0.0);
    }

    #[test]
    fn embed_memory_text_truncates() {
        let long = "x".repeat(MAX_INPUT_CHARS + 100);
        let text = embed_memory_text("s", &long);
        assert_eq!(text.chars().count(), MAX_INPUT_CHARS);
        assert!(text.starts_with("s\n\nx"));
    }

    /// 换说法召回：关键词趟只命中 m1，向量趟把语义相近的 m2 也带进来，
    /// 且 m1（两路双命中）排在 m2（仅向量）前面。
    #[test]
    fn hybrid_brings_in_vector_only_hits() {
        let memories = vec![
            mem(1, "token hashing", "sha256 of tokens"),
            mem(2, "密码保存", "哈希存储"),
        ];
        let keyword_hits = search::run(&memories, "hashing", &[], None);
        assert_eq!(keyword_hits.len(), 1);

        let mut table = HashMap::new();
        table.insert(1i64, unit(4, 0)); // 与查询同向
        table.insert(2i64, unit(4, 1)); // 与查询正交（相似度 0，不会进入）

        let hits = hybrid_hits(&memories, keyword_hits, &table, &unit(4, 0), &[], None);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);

        // m2 与查询有 0.87 的相似度（非正交）时进入结果
        let tilted: Vec<f32> = vec![0.9, 0.1, 0.0, 0.0];
        let mut table2 = table.clone();
        table2.insert(2i64, tilted);
        let keyword_hits = search::run(&memories, "hashing", &[], None);
        let hits = hybrid_hits(&memories, keyword_hits, &table2, &unit(4, 0), &[], None);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].idx, 0, "双命中必须排在仅向量命中前面");
        assert_eq!(hits[1].idx, 1);
        // 向量独有命中走兜底片段（正文开头），且做 HTML 转义
        assert!(hits[1].snippet.contains("哈希存储"));
    }

    /// 向量趟遵守与关键词趟相同的标签过滤。
    #[test]
    fn hybrid_respects_tag_filters() {
        let mut a = mem(1, "alpha", "a");
        a.tags = vec!["keep".to_string()];
        let mut b = mem(2, "beta", "b");
        b.tags = vec!["other".to_string()];
        let mut table = HashMap::new();
        table.insert(1i64, unit(2, 0));
        table.insert(2i64, unit(2, 0));
        let hits = hybrid_hits(
            &[a, b],
            Vec::new(),
            &table,
            &unit(2, 0),
            &["keep".to_string()],
            None,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);
    }

    /// 关键词与向量排名冲突时的融合：两路都靠前的条目必须胜过单路第一。
    #[test]
    fn hybrid_fuses_both_ranks() {
        let a = mem(1, "aa", "a");
        let b = mem(2, "bb", "b");
        let c = mem(3, "cc", "c");
        // 关键词名次：a 第一、b 第二；向量名次：b 第一、c 第二、a 第三
        // （向量取与查询不同夹角的方向，余弦可分辨）
        let k1 = Hit {
            idx: 0,
            score: 100,
            snippet: "sa".into(),
        };
        let k2 = Hit {
            idx: 1,
            score: 50,
            snippet: "sb".into(),
        };
        let mut table = HashMap::new();
        table.insert(2i64, vec![0.95, 0.31]);
        table.insert(3i64, vec![0.9, 0.44]);
        table.insert(1i64, vec![0.5, 0.87]);
        let hits = hybrid_hits(&[a, b, c], vec![k1, k2], &table, &[1.0, 0.0], &[], None);
        let order: Vec<usize> = hits.iter().map(|h| h.idx).collect();
        assert_eq!(order, vec![1, 0, 2], "b 双路上榜应胜过仅关键词第一的 a");
    }
}
