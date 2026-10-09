use super::*;
use crate::auth::Permissions;
use serde_json::json;

fn setup(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "agent-memory-pred-{label}-{}-{}.db",
        std::process::id(),
        crate::model::now()
    ));
    execute_with_db(&path,&IdentityCtx::open_mode(),MEMORY_CREATE,&json!({"summary":"private body sentinel", "content":"DO NOT DISCLOSE", "tags":["a"],"create_missing_tags":true})).unwrap();
    for name in ["b", "c"] {
        execute_with_db(
            &path,
            &IdentityCtx::open_mode(),
            TAG_CREATE,
            &json!({"name":name}),
        )
        .unwrap();
    }
    path
}

fn set(
    path: &Path,
    ctx: &IdentityCtx,
    scope: &str,
    name: &str,
    predicate: &str,
) -> Result<Value, ToolError> {
    execute_with_db(
        path,
        ctx,
        PREDICATE_SET,
        &json!({"scope":scope,"name":name,"predicate":predicate,"description":"description"}),
    )
}

#[test]
fn predicates_are_scoped_and_global_management_is_independent_of_admin() {
    let path = setup("scope");
    let (alice, bob) = store::with_db_in(&path, store::TxMode::Write, |st| -> Result<_, String> {
        let (a, _) = st.identity_create(
            "alice",
            &Permissions::from_json(&json!({"read":true,"admin":true}))?,
        )?;
        let (b, _) = st.identity_create(
            "bob",
            &Permissions::from_json(&json!({"read":true,"predicate_manage_global":true}))?,
        )?;
        Ok((
            st.identity_ctx_by_token(&a)?.unwrap(),
            st.identity_ctx_by_token(&b)?.unwrap(),
        ))
    })
    .unwrap();
    assert!(matches!(
        set(&path, &alice, "global", "choice", "a"),
        Err(ToolError::Forbidden(_))
    ));
    assert!(set(&path, &bob, "global", "choice", "a").is_ok());
    assert!(set(&path, &alice, "user", "choice", "b").is_ok());
    for (ctx, expr, total) in [
        (&alice, "@choice", 0),
        (&alice, "@global::choice", 1),
        (&bob, "@choice", 1),
    ] {
        assert_eq!(
            execute_with_db(&path, ctx, MEMORY_LIST, &json!({"tag_expr":expr})).unwrap()["total"],
            total
        );
    }
    let listing = execute_with_db(&path, &bob, PREDICATE_LIST, &json!({})).unwrap();
    assert_eq!(listing["predicates"].as_array().unwrap().len(), 1);
    assert!(!listing.to_string().contains("DO NOT DISCLOSE"));
    assert!(execute_with_db(&path, &bob, PREDICATE_LIST, &json!({"unexpected":true})).is_err());
    assert!(set(&path, &bob, "global", "bad name", "a").is_err());
    assert!(set(&path, &bob, "global", "private", "@user::choice").is_err());
}

#[test]
fn cycles_nonmonotone_dependencies_and_constraint_changes_roll_back_atomically() {
    let path = setup("rollback");
    let ctx = IdentityCtx::open_mode();
    set(&path, &ctx, "global", "root", "a").unwrap();
    set(&path, &ctx, "global", "alias", "@root").unwrap();
    assert!(set(&path, &ctx, "global", "root", "@alias").is_err());
    assert!(set(&path, &ctx, "global", "self", "@self").is_err());
    let rules = json!({"constraints":[],"derivations":[{"name":"derive","predicate":"@alias","derived":["c"]}]});
    store::with_db_in(&path, store::TxMode::Write, |st| {
        st.replace_tag_rules(&rules, false)
    })
    .unwrap();
    assert!(set(&path, &ctx, "global", "root", "!a").is_err());
    assert_eq!(
        execute_with_db(&path, &ctx, MEMORY_LIST, &json!({"tag_expr":"c"})).unwrap()["total"],
        1
    );
    set(&path, &ctx, "global", "root", "b").unwrap();
    assert_eq!(
        execute_with_db(&path, &ctx, MEMORY_LIST, &json!({"tag_expr":"c"})).unwrap()["total"],
        0
    );
    assert!(execute_with_db(
        &path,
        &ctx,
        PREDICATE_DELETE,
        &json!({"scope":"global","name":"root"})
    )
    .is_err());
    assert!(execute_with_db(&path, &ctx, TAG_DELETE, &json!({"name":"b"})).is_err());
    execute_with_db(
        &path,
        &ctx,
        TAG_UPDATE,
        &json!({"name":"b","new_name":"beta"}),
    )
    .unwrap();
    let listing = execute_with_db(&path, &ctx, PREDICATE_LIST, &json!({})).unwrap();
    assert!(listing.to_string().contains("beta"));
    let constraints = json!({"constraints":[{"name":"must","expression":"@alias"}]});
    set(&path, &ctx, "global", "root", "a").unwrap();
    store::with_db_in(&path, store::TxMode::Write, |st| {
        st.replace_tag_rules(&constraints, false)
    })
    .unwrap();
    assert!(set(&path, &ctx, "global", "root", "beta").is_err());
    assert_eq!(
        execute_with_db(&path, &ctx, MEMORY_LIST, &json!({"tag_expr":"@alias"})).unwrap()["total"],
        1
    );
}

#[test]
fn backup_restores_forward_references_rules_and_stable_tag_binding() {
    let path = setup("backup");
    let ctx = IdentityCtx::open_mode();
    set(&path, &ctx, "global", "z-root", "a").unwrap();
    set(&path, &ctx, "global", "a-alias", "@z-root").unwrap();
    store::with_db_in(&path,store::TxMode::Write,|st| st.replace_tag_rules(&json!({"constraints":[],"derivations":[{"name":"derive","expression":"@a-alias => c"}]}),false)).unwrap();
    let dump = store::with_db_in(&path, store::TxMode::ReadOnly, |st| st.export_dump()).unwrap();
    let restored = path.with_extension("restored.db");
    store::with_db_in(&restored, store::TxMode::Write, |st| st.import_dump(&dump)).unwrap();
    assert_eq!(
        execute_with_db(
            &restored,
            &ctx,
            MEMORY_LIST,
            &json!({"tag_expr":"@a-alias&c"})
        )
        .unwrap()["total"],
        1
    );
}

#[test]
fn decay_selectors_use_named_and_derived_tags_and_protect_references() {
    let path = setup("decay");
    let ctx = IdentityCtx::open_mode();
    set(&path, &ctx, "global", "category", "c").unwrap();
    store::with_db_in(&path,store::TxMode::Write,|st| -> Result<(),String> {
        st.replace_tag_rules(&json!({"constraints":[],"derivations":[{"name":"classify","expression":"a => c"}]}),false)?;
        st.settings_put(Store::SETTING_LIFECYCLE_POLICY,&json!({"rules":[{"predicate":"@category&!b", "half_life_days":7}],"default_half_life_days":30}).to_string())?;
        let policy = st.lifecycle_policy_compiled()?;
        let memory = &st.get_memories(&[1])?.0[0];
        assert!((policy.freshness(&st.lifecycle_get(1)?,&memory.tags,100,100+7*86400)-0.5).abs()<1e-12);
        Ok(())
    }).unwrap();
    assert!(execute_with_db(
        &path,
        &ctx,
        PREDICATE_DELETE,
        &json!({"scope":"global","name":"category"})
    )
    .is_err());
    assert!(execute_with_db(&path, &ctx, TAG_DELETE, &json!({"name":"b"})).is_err());
    assert!(execute_with_db(&path, &ctx, MEMORY_LIST, &json!({"tag_expr":"!@missing"})).is_err());
    execute_with_db(
        &path,
        &ctx,
        TAG_UPDATE,
        &json!({"name":"b","new_name":"beta"}),
    )
    .unwrap();
    let dump = store::with_db_in(&path, store::TxMode::ReadOnly, |st| st.export_dump()).unwrap();
    assert!(dump["lifecycle_policy"]["rules"][0]["predicate"]
        .as_str()
        .unwrap()
        .contains("beta"));
    let restored = path.with_extension("restored.db");
    store::with_db_in(
        &restored,
        store::TxMode::Write,
        |st| -> Result<(), String> {
            st.import_dump(&dump)?;
            assert!(
                (st.lifecycle_policy_compiled()?.freshness(
                    &crate::lifecycle::Metadata::default(),
                    &["a".into(), "c".into()],
                    0,
                    7 * 86400
                ) - 0.5)
                    .abs()
                    < 1e-12
            );
            Ok(())
        },
    )
    .unwrap();
}
