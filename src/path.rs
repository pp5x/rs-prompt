use std::fmt::{self, Write};
use std::path::{Path, PathBuf};

use crate::{ansi, vcs};

#[derive(Clone, Copy)]
struct Component<'a> {
    name: &'a str,
    vcs_type: Option<vcs::VcsType>,
}

pub fn truncate_component(component: &str) -> &str {
    if component.is_empty() {
        return component;
    }
    if component.as_bytes()[0] == b'.' {
        return &component[..component.len().min(2)];
    }
    &component[..component.len().min(1)]
}

pub fn is_under_home(path: &str, home: &str) -> bool {
    if home.is_empty() || !path.starts_with(home) {
        return false;
    }
    path.len() == home.len() || path.as_bytes().get(home.len()) == Some(&b'/')
}

pub fn display_path(cwd: &str, home: &str) -> std::io::Result<String> {
    let absolute = make_absolute(cwd);
    let absolute_str = absolute.to_string_lossy();
    let mut components = split_components(&absolute_str);

    let mut outermost_vcs_index = None;
    for index in 0..components.len() {
        let prefix = join_prefix(&components[..=index]);
        if let Some(kind) = vcs::detect(Path::new(&prefix))? {
            components[index].vcs_type = Some(kind);
            if outermost_vcs_index.is_none() {
                outermost_vcs_index = Some(index);
            }
        }
    }

    let mut out = String::new();
    if let Some(root) = outermost_vcs_index {
        write_components(&mut out, &components[root..], true, root == 0)
            .expect("writing to String failed");
        return Ok(out);
    }

    let home_offset = if is_under_home(&absolute_str, home) {
        count_components(home)
    } else {
        0
    };

    if home_offset > 0 {
        out.push('~');
        if components.len() > home_offset {
            out.push('/');
            write_components(&mut out, &components[home_offset..], false, false)
                .expect("writing to String failed");
        }
        return Ok(out);
    }

    if absolute_str == "/" {
        out.push('/');
        return Ok(out);
    }

    out.push('/');
    write_components(&mut out, &components, false, false).expect("writing to String failed");
    Ok(out)
}

fn make_absolute(path: &str) -> PathBuf {
    PathBuf::from(path)
}

fn split_components(path: &str) -> Vec<Component<'_>> {
    path.split('/')
        .filter(|part| !part.is_empty())
        .map(|name| Component {
            name,
            vcs_type: None,
        })
        .collect()
}

fn count_components(path: &str) -> usize {
    path.split('/').filter(|part| !part.is_empty()).count()
}

fn join_prefix(components: &[Component<'_>]) -> String {
    if components.is_empty() {
        return "/".to_string();
    }

    let mut out = String::from("/");
    for (index, component) in components.iter().enumerate() {
        if index != 0 {
            out.push('/');
        }
        out.push_str(component.name);
    }
    out
}

fn write_components(
    out: &mut String,
    components: &[Component<'_>],
    highlight_vcs: bool,
    preserve_leading_slash: bool,
) -> fmt::Result {
    if preserve_leading_slash {
        out.push('/');
    }
    for (index, component) in components.iter().enumerate() {
        if index != 0 {
            out.push('/');
        }
        if highlight_vcs {
            if let Some(kind) = component.vcs_type {
                out.write_str(ansi::BOLD)?;
                out.write_str(ansi::fg_for_vcs(kind))?;
                out.write_str(component.name)?;
                out.write_str(ansi::RESET)?;
                out.write_str(ansi::GREEN)?;
                continue;
            }
        }
        if index == components.len() - 1 {
            out.write_str(component.name)?;
        } else {
            out.write_str(truncate_component(component.name))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use std::fs;

    #[test]
    fn truncates_components() {
        assert_eq!("d", truncate_component("drivers"));
        assert_eq!(".c", truncate_component(".config"));
    }

    #[test]
    fn detects_home_boundary() {
        assert!(is_under_home("/home/me/dev", "/home/me"));
        assert!(is_under_home("/home/me", "/home/me"));
        assert!(!is_under_home("/home/me2/dev", "/home/me"));
    }

    #[test]
    fn shortens_home_relative_path() {
        assert_eq!(
            "~/d/f/bar",
            display_path("/home/me/dev/foo/bar", "/home/me").unwrap()
        );
    }

    #[test]
    fn starts_at_outermost_vcs_root() {
        let tmp = TempDir::new("rs-prompt-path");
        let path = tmp.path().join("outer").join("inner").join("deep");
        fs::create_dir_all(&path).unwrap();
        fs::create_dir_all(tmp.path().join("outer").join(".git")).unwrap();
        fs::create_dir_all(tmp.path().join("outer").join("inner").join(".jj")).unwrap();

        let rendered = display_path(path.to_str().unwrap(), tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(
            format!(
                "{}{}outer{}{}{}/{}{}inner{}{}{}/deep",
                ansi::BOLD,
                ansi::RED,
                ansi::RESET,
                ansi::GREEN,
                "",
                ansi::BOLD,
                ansi::YELLOW,
                ansi::RESET,
                ansi::GREEN,
                ""
            ),
            rendered
        );
    }

    #[test]
    fn preserves_leading_slash_for_root_level_vcs_paths() {
        let mut out = String::new();
        let components = vec![Component {
            name: "tmp",
            vcs_type: Some(vcs::VcsType::Git),
        }];

        write_components(&mut out, &components, true, true).unwrap();

        assert_eq!(
            format!(
                "/{}{}tmp{}{}",
                ansi::BOLD,
                ansi::RED,
                ansi::RESET,
                ansi::GREEN
            ),
            out
        );
    }
}
