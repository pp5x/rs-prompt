mod ansi;
mod host;
mod path;
mod prompt;
mod prompt_escape;
mod vcs;
mod venv;

#[cfg(test)]
mod test_support;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use prompt::Shell;
use prompt_escape::PromptEscape;

const INIT_ZSH: &str = include_str!("../scripts/init.zsh");
const INIT_BASH: &str = include_str!("../scripts/init.bash");
const INIT_FISH: &str = include_str!("../scripts/init.fish");

#[derive(Debug, Eq, PartialEq)]
struct PromptArgs {
    status: u8,
    cwd: Option<String>,
    host: Option<String>,
    user: Option<String>,
    shell: Shell,
    prompt_escape: PromptEscape,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map_or("prompt", String::as_str);

    match command {
        "prompt" => run_prompt(&args[2..]),
        "init" => run_init(&args),
        _ => {
            usage();
            process::exit(64);
        }
    }
}

fn run_prompt(args: &[String]) -> io::Result<()> {
    let parsed = match parse_prompt_args(args) {
        Ok(parsed) => parsed,
        Err(ParseError::Usage) => {
            usage();
            process::exit(64);
        }
        Err(ParseError::MissingValue(name)) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("missing value for {name}"),
            ));
        }
    };

    let cwd = parsed.cwd.unwrap_or_else(|| {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .into_owned()
    });
    let home = env::var("HOME").unwrap_or_default();
    let hostname = parsed.host.unwrap_or_else(system_hostname);
    let user = parsed.user.unwrap_or_else(default_user);
    let virtual_env = env::var("VIRTUAL_ENV").ok();

    let rendered = prompt::render(prompt::Options {
        cwd: &cwd,
        home: &home,
        hostname: &hostname,
        user: &user,
        virtual_env: virtual_env.as_deref(),
        status: parsed.status,
        shell: parsed.shell,
    })?;
    let rendered = prompt_escape::apply(&rendered, parsed.prompt_escape);

    print!("{rendered}");
    io::stdout().flush()
}

fn run_init(args: &[String]) -> io::Result<()> {
    let shell = args.get(2).map_or("zsh", String::as_str);
    let exe_path = args.first().map_or("rs-prompt", String::as_str);
    let mut stdout = io::stdout().lock();
    match shell {
        "zsh" => write_init_script(&mut stdout, exe_path, INIT_ZSH),
        "bash" => write_init_script(&mut stdout, exe_path, INIT_BASH),
        "fish" => write_init_script(&mut stdout, exe_path, INIT_FISH),
        _ => {
            usage();
            process::exit(64);
        }
    }
}

fn write_init_script(mut writer: impl Write, exe_path: &str, script: &str) -> io::Result<()> {
    let mut rest = script;
    while let Some(index) = rest.find("__RS_PROMPT_BIN__") {
        writer.write_all(rest[..index].as_bytes())?;
        write_single_quoted(&mut writer, exe_path)?;
        rest = &rest[index + "__RS_PROMPT_BIN__".len()..];
    }
    writer.write_all(rest.as_bytes())
}

fn write_single_quoted(writer: &mut impl Write, value: &str) -> io::Result<()> {
    writer.write_all(b"'")?;
    for byte in value.bytes() {
        if byte == b'\'' {
            writer.write_all(b"'\\''")?;
        } else {
            writer.write_all(&[byte])?;
        }
    }
    writer.write_all(b"'")
}

fn parse_prompt_args(args: &[String]) -> Result<PromptArgs, ParseError> {
    let mut parsed = PromptArgs {
        status: 0,
        cwd: None,
        host: None,
        user: None,
        shell: Shell::Zsh,
        prompt_escape: PromptEscape::None,
    };

    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix("--status=") {
            parsed.status = parse_status(value);
        } else if arg == "--status" {
            index += 1;
            parsed.status = parse_status(args.get(index).map_or("0", String::as_str));
        } else if let Some(value) = arg.strip_prefix("--cwd=") {
            parsed.cwd = Some(value.to_string());
        } else if arg == "--cwd" {
            index += 1;
            parsed.cwd = Some(
                args.get(index)
                    .ok_or(ParseError::MissingValue("--cwd"))?
                    .to_string(),
            );
        } else if let Some(value) = arg.strip_prefix("--host=") {
            parsed.host = Some(value.to_string());
        } else if arg == "--host" {
            index += 1;
            parsed.host = Some(
                args.get(index)
                    .ok_or(ParseError::MissingValue("--host"))?
                    .to_string(),
            );
        } else if let Some(value) = arg.strip_prefix("--user=") {
            parsed.user = Some(value.to_string());
        } else if arg == "--user" {
            index += 1;
            parsed.user = Some(
                args.get(index)
                    .ok_or(ParseError::MissingValue("--user"))?
                    .to_string(),
            );
        } else if let Some(value) = arg.strip_prefix("--shell=") {
            parsed.shell = parse_shell(value)?;
        } else if arg == "--shell" {
            index += 1;
            parsed.shell = parse_shell(args.get(index).map_or("", String::as_str))?;
        } else if let Some(value) = arg.strip_prefix("--prompt-escape=") {
            parsed.prompt_escape = parse_prompt_escape(value)?;
        } else if arg == "--prompt-escape" {
            index += 1;
            parsed.prompt_escape = parse_prompt_escape(args.get(index).map_or("", String::as_str))?;
        } else {
            return Err(ParseError::Usage);
        }
        index += 1;
    }

    Ok(parsed)
}

fn parse_shell(value: &str) -> Result<Shell, ParseError> {
    match value {
        "zsh" => Ok(Shell::Zsh),
        "bash" => Ok(Shell::Bash),
        "fish" => Ok(Shell::Fish),
        _ => Err(ParseError::Usage),
    }
}

fn parse_prompt_escape(value: &str) -> Result<PromptEscape, ParseError> {
    match value {
        "none" => Ok(PromptEscape::None),
        "zsh" => Ok(PromptEscape::Zsh),
        "bash" => Ok(PromptEscape::Bash),
        "fish" => Ok(PromptEscape::Fish),
        _ => Err(ParseError::Usage),
    }
}

fn parse_status(value: &str) -> u8 {
    value
        .parse::<u16>()
        .map_or(0, |status| status.min(255) as u8)
}

fn default_user() -> String {
    env::var("USER")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| env::var("LOGNAME").ok().filter(|value| !value.is_empty()))
        .unwrap_or_default()
}

fn system_hostname() -> String {
    if let Ok(value) = env::var("HOST") {
        if !value.is_empty() {
            return value;
        }
    }

    fs::read_to_string("/proc/sys/kernel/hostname")
        .or_else(|_| fs::read_to_string("/etc/hostname"))
        .map(|value| value.trim_end_matches(&['\r', '\n'][..]).to_string())
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn usage() {
    eprint!(
        "usage:\n  rs-prompt prompt [--status=N] [--cwd=PATH] [--host=HOST] [--user=USER] [--shell=zsh|bash|fish] [--prompt-escape=none|zsh|bash|fish]\n  rs-prompt init zsh\n  rs-prompt init bash\n  rs-prompt init fish\n"
    );
}

#[derive(Debug, Eq, PartialEq)]
enum ParseError {
    Usage,
    MissingValue(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status() {
        assert_eq!(0, parse_status("0"));
        assert_eq!(2, parse_status("2"));
        assert_eq!(255, parse_status("999"));
        assert_eq!(0, parse_status("nope"));
    }

    #[test]
    fn parses_prompt_options() {
        let args = vec![
            "--status=2".to_string(),
            "--cwd".to_string(),
            "/tmp/project".to_string(),
            "--host=build42.lab".to_string(),
        ];

        assert_eq!(
            PromptArgs {
                status: 2,
                cwd: Some("/tmp/project".to_string()),
                host: Some("build42.lab".to_string()),
                user: None,
                shell: Shell::Zsh,
                prompt_escape: PromptEscape::None,
            },
            parse_prompt_args(&args).unwrap()
        );
    }

    #[test]
    fn parses_prompt_user_and_shell_options() {
        let args = vec![
            "--user".to_string(),
            "root".to_string(),
            "--shell=bash".to_string(),
            "--prompt-escape".to_string(),
            "bash".to_string(),
        ];

        assert_eq!(
            PromptArgs {
                status: 0,
                cwd: None,
                host: None,
                user: Some("root".to_string()),
                shell: Shell::Bash,
                prompt_escape: PromptEscape::Bash,
            },
            parse_prompt_args(&args).unwrap()
        );
    }

    #[test]
    fn parses_fish_shell() {
        let args = vec![
            "--shell=fish".to_string(),
            "--prompt-escape=fish".to_string(),
        ];
        let parsed = parse_prompt_args(&args).unwrap();
        assert_eq!(Shell::Fish, parsed.shell);
        assert_eq!(PromptEscape::Fish, parsed.prompt_escape);
    }

    #[test]
    fn parses_prompt_escape_option() {
        let args = vec!["--prompt-escape=zsh".to_string()];
        assert_eq!(
            PromptEscape::Zsh,
            parse_prompt_args(&args).unwrap().prompt_escape
        );
    }

    #[test]
    fn rejects_unknown_prompt_escape() {
        let args = vec!["--prompt-escape=tcsh".to_string()];
        assert_eq!(Err(ParseError::Usage), parse_prompt_args(&args));
    }

    #[test]
    fn quotes_init_binary_path() {
        let mut out = Vec::new();
        write_single_quoted(&mut out, "/tmp/it's/rs-prompt").unwrap();
        assert_eq!("'/tmp/it'\\''s/rs-prompt'", String::from_utf8(out).unwrap());
    }
}
