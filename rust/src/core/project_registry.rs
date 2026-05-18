use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub root: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<String>,
    pub last_active: u64,
    pub pid: u32,
}

pub fn upsert_project_meta(project_root: &str, pid: u32) {
    let Ok(dir) = super::data_dir::project_data_dir(project_root) else {
        return;
    };
    let hash = super::project_hash::hash_project_root(project_root);
    let identity = super::project_hash::project_identity(project_root);
    let name = identity
        .as_deref()
        .and_then(|id| id.split_once(':').map(|(_, n)| n))
        .map_or_else(
            || {
                Path::new(project_root)
                    .file_name()
                    .map_or_else(|| hash.clone(), |f| f.to_string_lossy().to_string())
            },
            |n| n.rsplit('/').next().unwrap_or(n).to_string(),
        );

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let meta = ProjectMeta {
        root: project_root.to_string(),
        name,
        identity,
        last_active: ts,
        pid,
    };

    let path = dir.join("project_meta.json");
    if let Ok(json) = serde_json::to_string_pretty(&meta) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, &json).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

pub fn load_project_meta(project_dir: &Path) -> Option<ProjectMeta> {
    let path = project_dir.join("project_meta.json");
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn load_project_by_hash(hash: &str) -> Option<ProjectMeta> {
    let data_dir = super::data_dir::lean_ctx_data_dir().ok()?;
    let dir = data_dir.join("projects").join(hash);
    load_project_meta(&dir)
}

pub fn list_known_projects() -> Vec<(String, ProjectMeta)> {
    let Ok(data_dir) = super::data_dir::lean_ctx_data_dir() else {
        return Vec::new();
    };
    let projects_dir = data_dir.join("projects");
    let Ok(entries) = std::fs::read_dir(&projects_dir) else {
        return Vec::new();
    };

    let mut result = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let hash = entry.file_name().to_string_lossy().to_string();
        if let Some(meta) = load_project_meta(&dir) {
            result.push((hash, meta));
        }
    }
    result.sort_by_key(|entry| std::cmp::Reverse(entry.1.last_active));
    result
}

pub fn list_active_projects() -> Vec<(String, ProjectMeta)> {
    list_known_projects()
        .into_iter()
        .filter(|(_, meta)| super::agents::is_process_alive(meta.pid))
        .collect()
}

pub fn most_recent_project() -> Option<(String, ProjectMeta)> {
    list_known_projects().into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_meta_round_trip() {
        let meta = ProjectMeta {
            root: "/tmp/test-project".to_string(),
            name: "test-project".to_string(),
            identity: Some("git:github.com/user/test".to_string()),
            last_active: 1716000000,
            pid: 12345,
        };
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: ProjectMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.root, meta.root);
        assert_eq!(parsed.name, meta.name);
        assert_eq!(parsed.identity, meta.identity);
        assert_eq!(parsed.last_active, meta.last_active);
        assert_eq!(parsed.pid, meta.pid);
    }

    #[test]
    fn project_meta_without_identity() {
        let meta = ProjectMeta {
            root: "/tmp/bare-project".to_string(),
            name: "bare-project".to_string(),
            identity: None,
            last_active: 1716000000,
            pid: 99999,
        };
        let json = serde_json::to_string(&meta).unwrap();
        assert!(!json.contains("identity"));
        let parsed: ProjectMeta = serde_json::from_str(&json).unwrap();
        assert!(parsed.identity.is_none());
    }
}
