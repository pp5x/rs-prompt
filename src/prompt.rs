use std::fmt::Write;

use crate::{ansi, host, path, venv};

const HIDDEN_USERS: &[&str] = &[];
const HIDDEN_HOSTNAMES: &[&str] = &[];

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
    Fish,
}

pub fn render(options: Options<'_>) -> std::io::Result<String> {
    render_with_hidden(options, HIDDEN_USERS, HIDDEN_HOSTNAMES)
}

fn render_with_hidden(
    options: Options<'_>,
    hidden_users: &[&str],
    hidden_hostnames: &[&str],
) -> std::io::Result<String> {
    let mut out = String::new();
    let show_hostname = should_show_hostname(options.hostname, hidden_hostnames);

    out.push_str(ansi::WHITE);
    if show_hostname {
        let short_hostname = host::short_hostname(options.hostname);
        if should_show_user(options.user, hidden_users) {
            out.push_str(user_color(options.user));
            out.push_str(options.user);
            out.push_str(ansi::RESET);
            out.push_str(ansi::WHITE);
            out.push('@');
        }
        host::write_highlighted(&mut out, short_hostname).expect("writing to String failed");
        out.push_str(ansi::RESET);
        out.push(' ');
    }

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
        Shell::Fish => {
            out.push_str(ansi::BOLD);
            out.push_str(ansi::BLUE);
            out.push('>');
            out.push_str(ansi::RESET);
        }
    }
    out.push(' ');
}

fn should_show_user(user: &str, hidden_users: &[&str]) -> bool {
    !user.is_empty() && !hidden_users.contains(&user)
}

fn should_show_hostname(hostname: &str, hidden_hostnames: &[&str]) -> bool {
    !hostname.is_empty() && !hidden_hostnames.contains(&hostname)
}

fn is_root(user: &str) -> bool {
    user == "root"
}

fn user_color(user: &str) -> &'static str {
    if is_root(user) {
        ansi::LIGHT_RED
    } else {
        ansi::LIGHT_GREEN
    }
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
    fn hides_hostname_and_user_without_leading_whitespace() {
        let out = render_with_hidden(
            Options {
                cwd: "/tmp",
                home: "/home/example",
                hostname: "host9.example",
                user: "alice",
                virtual_env: None,
                status: 0,
                shell: Shell::Zsh,
            },
            &["alice"],
            &["host9.example"],
        )
        .unwrap();

        assert!(out.starts_with(&format!("{}{}", ansi::WHITE, ansi::GREEN)));
        assert!(!out.starts_with(' '));
        assert!(!out.contains("alice@"));
        assert!(!out.contains("host9"));
    }

    #[test]
    fn hides_hostname_identity_even_for_visible_user() {
        let out = render_with_hidden(
            Options {
                cwd: "/tmp",
                home: "/home/example",
                hostname: "host9.example",
                user: "alice",
                virtual_env: None,
                status: 0,
                shell: Shell::Zsh,
            },
            &[],
            &["host9.example"],
        )
        .unwrap();

        assert!(out.starts_with(&format!("{}{}", ansi::WHITE, ansi::GREEN)));
        assert!(!out.contains("alice@"));
        assert!(!out.contains("host9"));
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
        assert!(user.starts_with(&format!(
            "{}{}alice{}{}@host",
            ansi::WHITE,
            ansi::LIGHT_GREEN,
            ansi::RESET,
            ansi::WHITE
        )));

        assert!(!user.contains(&format!("{}alice", ansi::LIGHT_RED)));
    }

    #[test]
    fn colors_root_user_light_red_and_marker_dark_red() {
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
        assert!(root.starts_with(&format!(
            "{}{}root{}{}@host",
            ansi::WHITE,
            ansi::LIGHT_RED,
            ansi::RESET,
            ansi::WHITE
        )));
        assert!(!root.contains(&format!("{}root", ansi::LIGHT_GREEN)));
        assert!(root.ends_with(&format!("{}{}#{} ", ansi::BOLD, ansi::RED, ansi::RESET)));
        assert!(!root.ends_with(&format!(
            "{}{}#{} ",
            ansi::BOLD,
            ansi::LIGHT_RED,
            ansi::RESET
        )));
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

        let fish = render(Options {
            cwd: "/tmp",
            home: "/home/example",
            hostname: "host",
            user: "",
            virtual_env: None,
            status: 0,
            shell: Shell::Fish,
        })
        .unwrap();
        assert!(fish.ends_with(&format!("{}{}>{} ", ansi::BOLD, ansi::BLUE, ansi::RESET)));
    }
}
