use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::var_os("RS_PROMPT_TEST_TMPDIR")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .filter(|path| !has_vcs_marker_ancestor(path))
            .unwrap_or_else(std::env::temp_dir);
        let path = base.join(format!("{prefix}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn has_vcs_marker_ancestor(path: &Path) -> bool {
    path.ancestors().any(|current| {
        current.join(".git").exists()
            || current.join(".jj").exists()
            || current.join(".repo").exists()
    })
}
