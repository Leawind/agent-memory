//! 持久化：单个 JSON 文件 + 临时文件原子替换 + .bak 备份 + 跨进程文件锁。
//!
//! 标签表（tags）保存标签的元信息（描述、创建时间），记忆条目（memories）通过
//! 标签名字符串引用标签。标签可以零引用存在（由 agent 显式管理生命周期）。
//!
//! 本层只做存取，不做业务校验；数据模型见 `model`。

use crate::model::{Memory, Tag};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub const FORMAT_VERSION: u32 = 1;
const LOCK_STALE: Duration = Duration::from_secs(15);
const LOCK_RETRY: Duration = Duration::from_millis(50);
const LOCK_TOTAL_WAIT: Duration = Duration::from_secs(5);

/// 锁等待上限：AGENT_MEMORY_LOCK_WAIT_MS 环境变量可覆盖（毫秒，上限 60s）。
/// 供测试快速失败，也给想要"锁被占就立刻报错"的使用者一个旋钮。
fn lock_total_wait() -> Duration {
    match std::env::var("AGENT_MEMORY_LOCK_WAIT_MS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
    {
        Some(ms) => Duration::from_millis(ms.min(60_000)),
        None => LOCK_TOTAL_WAIT,
    }
}

#[derive(Serialize, Deserialize)]
struct FileFormat {
    format_version: u32,
    #[serde(default)]
    next_id: u64,
    #[serde(default)]
    tags: Vec<Tag>,
    #[serde(default)]
    memories: Vec<Memory>,
}

pub struct Store {
    pub path: PathBuf,
    pub next_id: u64,
    pub tags: BTreeMap<String, Tag>,
    pub memories: Vec<Memory>,
}

/// 数据文件路径：AGENT_MEMORY_PATH 环境变量优先，否则 ~/.agent-memory/memory.json。
pub fn default_path() -> PathBuf {
    if let Ok(p) = std::env::var("AGENT_MEMORY_PATH") {
        let p = p.trim();
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".agent-memory").join("memory.json")
}

pub fn tmp_path(p: &Path) -> PathBuf {
    PathBuf::from(format!("{}.tmp", p.display()))
}

pub fn backup_path(p: &Path) -> PathBuf {
    PathBuf::from(format!("{}.bak", p.display()))
}

pub fn lock_path(p: &Path) -> PathBuf {
    PathBuf::from(format!("{}.lock", p.display()))
}

impl Store {
    pub fn empty(path: PathBuf) -> Store {
        Store {
            path,
            next_id: 1,
            tags: BTreeMap::new(),
            memories: Vec::new(),
        }
    }

    /// 读取数据文件。文件不存在或为空 → 返回空库；
    /// 主文件损坏但存在可读的 .bak → 用备份恢复；
    /// 都不可用 → 返回错误（调用方应拒绝启动/操作，绝不覆盖数据）。
    pub fn load(path: PathBuf) -> io::Result<Store> {
        match read_store_file(&path) {
            Ok(Some(st)) => Ok(st),
            Ok(None) => Ok(Store::empty(path)),
            Err(primary_err) => {
                let bak = backup_path(&path);
                if bak.exists() {
                    if let Ok(Some(mut st)) = read_store_file(&bak) {
                        eprintln!(
                            "agent-memory: primary store at {} is unreadable ({}); loaded backup {}",
                            path.display(),
                            primary_err,
                            bak.display()
                        );
                        st.path = path;
                        return Ok(st);
                    }
                }
                Err(primary_err)
            }
        }
    }

    pub fn new_id(&mut self) -> String {
        let id = format!("m{}", self.next_id);
        self.next_id += 1;
        id
    }

    /// 查找与给定名字仅大小写不同的既有标签（用于分类体系防碎片化提示）。
    pub fn find_tag_case_insensitive(&self, name: &str) -> Option<String> {
        let fold = name.to_lowercase();
        self.tags
            .keys()
            .find(|k| k.to_lowercase() == fold && k.as_str() != name)
            .cloned()
    }

    /// 体检：报告数据中的隐患（不修改任何内容）。
    /// 覆盖 API 不可能产生、但手工编辑或未来 bug 可能引入的问题。
    pub fn hygiene_issues(&self) -> Vec<String> {
        use std::collections::BTreeSet;

        let mut issues = Vec::new();

        // 记忆引用了标签表里不存在的标签
        let mut orphans: BTreeSet<&str> = BTreeSet::new();
        for m in &self.memories {
            for t in &m.tags {
                if !self.tags.contains_key(t) {
                    orphans.insert(t.as_str());
                }
            }
        }
        if !orphans.is_empty() {
            let list: Vec<&str> = orphans.into_iter().collect();
            issues.push(format!(
                "memories reference tags missing from the tag table: {} (fix with tag_create, or remove the references)",
                list.join(", ")
            ));
        }

        // 仅大小写不同的标签组（分类体系碎片化）
        let mut by_fold: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for name in self.tags.keys() {
            by_fold
                .entry(name.to_lowercase())
                .or_default()
                .push(name.clone());
        }
        for group in by_fold.values() {
            if group.len() > 1 {
                issues.push(format!(
                    "case-conflicting tag group: {} (keep one and merge the rest with tag_rename)",
                    group.join(" / ")
                ));
            }
        }

        // 空摘要 / 空正文（API 层已拦截，这里兜底手工编辑的情况）
        for m in &self.memories {
            if m.summary.trim().is_empty() {
                issues.push(format!("memory {} has an empty summary", m.id));
            }
            if m.content.trim().is_empty() {
                issues.push(format!("memory {} has empty content", m.id));
            }
        }

        issues
    }

    /// 原子保存：先写 .tmp，旧文件拷为 .bak，再用 rename 原子替换主文件。
    /// 任何时刻主文件要么是完整的旧内容，要么是完整的新内容。
    pub fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let ff = FileFormat {
            format_version: FORMAT_VERSION,
            next_id: self.next_id,
            tags: self.tags.values().cloned().collect(),
            memories: self.memories.clone(),
        };
        let body = serde_json::to_string(&ff)?;
        let tmp = tmp_path(&self.path);
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(body.as_bytes())?;
            let _ = f.sync_all();
        }
        if self.path.exists() {
            let _ = fs::copy(&self.path, backup_path(&self.path));
        }
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

fn read_store_file(path: &Path) -> io::Result<Option<Store>> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    if text.trim().is_empty() {
        return Ok(None);
    }
    let ff: FileFormat = serde_json::from_str(&text).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid store file: {}", e),
        )
    })?;
    if ff.format_version > FORMAT_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "store format v{} is newer than supported v{}",
                ff.format_version, FORMAT_VERSION
            ),
        ));
    }
    let mut st = Store {
        path: path.to_path_buf(),
        next_id: ff.next_id.max(1),
        tags: BTreeMap::new(),
        memories: ff.memories,
    };
    for t in ff.tags {
        st.tags.insert(t.name.clone(), t);
    }
    // 防御：即使文件被手工编辑导致 next_id 偏小，也不会分配出重复 id。
    for m in &st.memories {
        if let Some(n) = m.id.strip_prefix('m') {
            if let Ok(k) = n.parse::<u64>() {
                st.next_id = st.next_id.max(k + 1);
            }
        }
    }
    Ok(Some(st))
}

/// 跨进程互斥锁：用 create_new 语义创建 <data>.lock 文件，进程退出（含 panic 展开）时删除。
/// 锁文件超过 LOCK_STALE 未更新视为残留（如宿主被 kill -9），自动接管。
pub struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    pub fn acquire(path: &Path) -> io::Result<LockGuard> {
        let deadline = SystemTime::now() + lock_total_wait();
        loop {
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(mut f) => {
                    let _ = writeln!(f, "{}", std::process::id());
                    return Ok(LockGuard {
                        path: path.to_path_buf(),
                    });
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    let stale = fs::metadata(path)
                        .and_then(|m| m.modified())
                        .map(|t| t.elapsed().map(|age| age >= LOCK_STALE).unwrap_or(true))
                        .unwrap_or(true);
                    if stale {
                        let _ = fs::remove_file(path);
                        continue;
                    }
                    if SystemTime::now() >= deadline {
                        return Err(io::Error::new(
                            io::ErrorKind::WouldBlock,
                            "another agent-memory process is holding the store lock",
                        ));
                    }
                    std::thread::sleep(LOCK_RETRY);
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "agent-memory-store-{}-{}.json",
            std::process::id(),
            tag
        ))
    }

    fn sample(id: &str) -> Memory {
        Memory {
            id: id.to_string(),
            summary: "s".into(),
            content: "c".into(),
            tags: vec!["t".into()],
            created_at: 1,
            updated_at: 1,
        }
    }

    #[test]
    fn save_load_roundtrip_continues_ids() {
        let path = temp_file("roundtrip");
        let _ = fs::remove_file(&path);
        {
            let mut st = Store::empty(path.clone());
            let id1 = st.new_id();
            st.memories.push(sample(&id1));
            st.tags.insert("t".to_string(), Tag::new("t".into()));
            st.save().unwrap();
            assert_eq!(st.next_id, 2);
        }
        {
            let mut st = Store::load(path.clone()).unwrap();
            assert_eq!(st.memories.len(), 1);
            assert_eq!(st.memories[0].id, "m1");
            assert_eq!(st.tags.len(), 1);
            assert_eq!(st.new_id(), "m2");
        }
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_and_empty_files_are_fresh_stores() {
        let path = temp_file("fresh");
        let _ = fs::remove_file(&path);
        assert!(Store::load(path.clone()).unwrap().memories.is_empty());
        fs::write(&path, "").unwrap();
        assert!(Store::load(path.clone()).unwrap().memories.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn corrupt_primary_falls_back_to_backup() {
        let path = temp_file("bak");
        let _ = fs::remove_file(&path);
        let mut good = Store::empty(path.clone());
        let id = good.new_id();
        good.memories.push(sample(&id));
        good.save().unwrap();
        good.save().unwrap(); // 第二次保存才会生成 .bak
        fs::write(&path, "{corrupt json").unwrap();
        let st = Store::load(path.clone()).unwrap();
        assert_eq!(st.path, path);
        assert_eq!(st.memories.len(), 1);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(backup_path(&path));
    }

    #[test]
    fn future_format_version_is_rejected() {
        let path = temp_file("future");
        let _ = fs::remove_file(&path);
        fs::write(
            &path,
            r#"{"format_version":999,"next_id":1,"tags":[],"memories":[]}"#,
        )
        .unwrap();
        assert!(Store::load(path.clone()).is_err());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn lock_guard_is_reentrant_after_release() {
        let path = temp_file("lock");
        let lock = lock_path(&path);
        let _ = fs::remove_file(&lock);
        {
            let _g = LockGuard::acquire(&lock).unwrap();
            assert!(lock.exists());
        }
        assert!(!lock.exists());
        let _g2 = LockGuard::acquire(&lock).unwrap();
        let _ = fs::remove_file(&lock);
    }

    #[test]
    fn lock_contention_fails_fast_when_wait_exhausted() {
        let path = temp_file("contention");
        let lock = lock_path(&path);
        let _ = fs::remove_file(&lock);
        std::env::set_var("AGENT_MEMORY_LOCK_WAIT_MS", "0");
        let _holder = LockGuard::acquire(&lock).unwrap();
        let err = match LockGuard::acquire(&lock) {
            Err(e) => e,
            Ok(_) => panic!("expected lock acquisition to fail while held"),
        };
        assert_eq!(err.kind(), io::ErrorKind::WouldBlock);
        assert!(err.to_string().contains("holding the store lock"));
        std::env::remove_var("AGENT_MEMORY_LOCK_WAIT_MS");
        drop(_holder);
        assert!(!lock.exists());
        let _ = fs::remove_file(&lock);
    }

    #[test]
    fn case_insensitive_tag_lookup() {
        let mut st = Store::empty(PathBuf::from("unused.json"));
        st.tags.insert("Rust".to_string(), Tag::new("Rust".into()));
        assert_eq!(
            st.find_tag_case_insensitive("rust"),
            Some("Rust".to_string())
        );
        assert_eq!(st.find_tag_case_insensitive("Rust"), None);
        assert_eq!(st.find_tag_case_insensitive("go"), None);
    }

    #[test]
    fn hygiene_issues_reports_real_problems_only() {
        let mut st = Store::empty(PathBuf::from("unused.json"));
        // 干净的库：零问题
        let mut m = sample("m1");
        m.tags = vec!["rust".into()];
        st.memories.push(m);
        st.tags.insert("rust".to_string(), Tag::new("rust".into()));
        assert!(st.hygiene_issues().is_empty());

        // 注入三类问题：孤儿引用、大小写冲突组、空摘要
        st.memories[0].tags.push("ghost".into());
        st.tags.insert("Rust".to_string(), Tag::new("Rust".into()));
        let mut bad = sample("m2");
        bad.summary = "   ".into();
        st.memories.push(bad);

        let issues = st.hygiene_issues().join("\n");
        assert!(
            issues.contains("ghost"),
            "missing orphan report: {}",
            issues
        );
        assert!(
            issues.contains("case-conflicting"),
            "missing case group report: {}",
            issues
        );
        assert!(
            issues.contains("m2 has an empty summary"),
            "missing empty summary report: {}",
            issues
        );
    }
}
