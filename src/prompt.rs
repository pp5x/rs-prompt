use std::fmt::Write;

use crate::{ansi, host, path, venv};

const HIDDEN_USERS: &[&str] = &[];

pub struct Options<'a> {
    pub cwd: &'a str,
    pub home: &'a str,
    pub hostname: &'a str,
    pub user: &'a str,
    pub virtual_env: Option<&'a str>,
    pub status: u8,
    pub shell: Shell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Shell {
    Zsh,
    Bash,
}

pub fn render(options: Options<'_>) -> std::io::Result<String> {
    let mut out = String::new();
    let short_hostname = host::short_hostname(options.hostname);

    out.push_str(ansi::WHITE);
    if should_show_user(options.user) {
        write!(out, "{}@", options.user).expect("writing to String failed");
    }
    host::write_highlighted(&mut out, short_hostname).expect("writing to String failed");
    out.push_str(ansi::RESET);
    out.push(' ');

    if let Some(virtual_env) = options.virtual_env {
        let name = venv::basename(virtual_env);
        if !name.is_empty() {
            out.push_str(ansi::YELLOW);
            write!(out, "({name})").expect("writing to String failed");
            out.push_str(ansi::RESET);
            out.push(' ');
        }
    }

    out.push_str(ansi::GREEN);
    out.push_str(&path::display_path(options.cwd, options.home)?);
    out.push_str(ansi::RESET);
    out.push(' ');

    if options.status != 0 {
        out.push('[');
        out.push_str(ansi::RED);
        write!(out, "{}", options.status).expect("writing to String failed");
        out.push_str(ansi::RESET);
        out.push_str("] ");
    }

    write_end_marker(&mut out, options.shell, is_root(options.user));
    Ok(out)
}

fn write_end_marker(out: &mut String, shell: Shell, root: bool) {
    if root {
        out.push_str(ansi::BOLD);
        out.push_str(ansi::RED);
        out.push('#');
        out.push_str(ansi::RESET);
        out.push(' ');
        return;
    }

    match shell {
        Shell::Zsh => {
            out.push_str(ansi::BOLD);
            out.push_str(ansi::BLUE);
            out.push('%');
            out.push_str(ansi::RESET);
        }
        Shell::Bash => {
            out.push_str(ansi::GREEN);
            out.push('$');
            out.push_str(ansi::RESET);
        }
    }
    out.push(' ');
}

fn should_show_user(user: &str) -> bool {
    !user.is_empty() && !HIDDEN_USERS.contains(&user)
}

fn is_root(user: &str) -> bool {
    user == "root"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use std::fs;

    #[test]
    fn composes_prompt_snippets() {
        let tmp = TempDir::new("rs-prompt-render");
        let cwd = tmp.path().join("home").join("dev").join("foo").join("bar");
        fs::create_dir_all(&cwd).unwrap();
        let home = tmp.path().join("home");

        let out = render(Options {
            cwd: cwd.to_str().unwrap(),
            home: home.to_str().unwrap(),
            hostname: "host9.example",
            user: "",
            virtual_env: Some("/venvs/env"),
            status: 2,
            shell: Shell::Zsh,
        })
        .unwrap();

        assert!(out.contains(&format!(
            "{}host{}9{}",
            ansi::WHITE,
            ansi::UNDERLINE,
            ansi::UNDERLINE_OFF
        )));
        assert!(out.contains(&format!("{}(env){}", ansi::YELLOW, ansi::RESET)));
        assert!(out.contains("~/d/f/bar"));
        assert!(out.ends_with(&format!("] {}{}%{} ", ansi::BOLD, ansi::BLUE, ansi::RESET)));
    }

    #[test]
    fn hides_empty_user() {
        let out = render(Options {
            cwd: "/tmp",
            home: "/home/example",
            hostname: "host9.example",
            user: "",
            virtual_env: None,
            status: 0,
            shell: Shell::Zsh,
        })
        .unwrap();

        assert!(out.starts_with(&format!(
            "{}host{}9{}{} ",
            ansi::WHITE,
            ansi::UNDERLINE,
            ansi::UNDERLINE_OFF,
            ansi::RESET
        )));
    }

    #[test]
    fn shows_other_users_and_root() {
        let user = render(Options {
            cwd: "/tmp",
            home: "/home/example",
            hostname: "host9.example",
            user: "alice",
            virtual_env: None,
            status: 0,
            shell: Shell::Zsh,
        })
        .unwrap();
        assert!(user.starts_with(&format!("{}alice@host", ansi::WHITE)));

        let root = render(Options {
            cwd: "/tmp",
            home: "/root",
            hostname: "host9.example",
            user: "root",
            virtual_env: None,
            status: 0,
            shell: Shell::Bash,
        })
        .unwrap();
        assert!(root.starts_with(&format!("{}root@host", ansi::WHITE)));
        assert!(root.ends_with(&format!("{}{}#{} ", ansi::BOLD, ansi::RED, ansi::RESET)));
    }

    #[test]
    fn shell_specific_end_markers() {
        let zsh = render(Options {
            cwd: "/tmp",
            home: "/home/example",
            hostname: "host",
            user: "",
            virtual_env: None,
            status: 0,
            shell: Shell::Zsh,
        })
        .unwrap();
        assert!(zsh.ends_with(&format!("{}{}%{} ", ansi::BOLD, ansi::BLUE, ansi::RESET)));

        let bash = render(Options {
            cwd: "/tmp",
            home: "/home/example",
            hostname: "host",
            user: "",
            virtual_env: None,
            status: 0,
            shell: Shell::Bash,
        })
        .unwrap();
        assert!(bash.ends_with(&format!("{}${} ", ansi::GREEN, ansi::RESET)));
    }
}
