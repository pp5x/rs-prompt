use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsType {
    Jj,
    Git,
    Repo,
}

pub fn detect(path: &Path) -> io::Result<Option<VcsType>> {
    if marker_exists(path, ".jj", false)? {
        return Ok(Some(VcsType::Jj));
    }
    if marker_exists(path, ".git", true)? {
        return Ok(Some(VcsType::Git));
    }
    if marker_exists(path, ".repo", false)? {
        return Ok(Some(VcsType::Repo));
    }
    Ok(None)
}

fn marker_exists(path: &Path, marker: &str, allow_file: bool) -> io::Result<bool> {
    let marker_path = path.join(marker);
    let metadata = match fs::symlink_metadata(marker_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let file_type = metadata.file_type();
    Ok(file_type.is_dir() || file_type.is_symlink() || (allow_file && file_type.is_file()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use std::fs::{self, File};

    #[cfg(unix)]
    use std::os::unix::fs as unix_fs;

    #[test]
    fn detects_markers_in_priority_order() {
        let tmp = TempDir::new("rs-prompt-vcs");

        let project = tmp.path().join("project");
        fs::create_dir_all(project.join(".git")).unwrap();
        assert_eq!(Some(VcsType::Git), detect(&project).unwrap());

        let jj = tmp.path().join("jjproj");
        fs::create_dir_all(jj.join(".jj")).unwrap();
        assert_eq!(Some(VcsType::Jj), detect(&jj).unwrap());

        let repo = tmp.path().join("repo");
        fs::create_dir_all(repo.join(".repo")).unwrap();
        assert_eq!(Some(VcsType::Repo), detect(&repo).unwrap());

        fs::create_dir_all(project.join(".jj")).unwrap();
        assert_eq!(Some(VcsType::Jj), detect(&project).unwrap());
    }

    #[test]
    fn detects_git_file_marker() {
        let tmp = TempDir::new("rs-prompt-git-file");
        let project = tmp.path().join("worktree");
        fs::create_dir_all(&project).unwrap();
        File::create(project.join(".git")).unwrap();
        assert_eq!(Some(VcsType::Git), detect(&project).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn detects_symlink_marker_without_following() {
        let tmp = TempDir::new("rs-prompt-symlink");
        let target = tmp.path().join("target");
        let linked = tmp.path().join("linked");
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&linked).unwrap();
        unix_fs::symlink(&target, linked.join(".jj")).unwrap();
        assert_eq!(Some(VcsType::Jj), detect(&linked).unwrap());
    }
}
