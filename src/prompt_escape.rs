#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptEscape {
    None,
    Zsh,
    Bash,
}

pub fn apply(prompt: &str, escape: PromptEscape) -> String {
    match escape {
        PromptEscape::None => prompt.to_string(),
        PromptEscape::Zsh => wrap_and_escape_zsh(prompt),
        PromptEscape::Bash => wrap_bash(prompt),
    }
}

fn wrap_and_escape_zsh(prompt: &str) -> String {
    wrap_ansi(
        prompt,
        |out, ansi| {
            out.push_str("%{");
            out.push_str(ansi);
            out.push_str("%}");
        },
        true,
    )
}

fn wrap_bash(prompt: &str) -> String {
    wrap_ansi(
        prompt,
        |out, ansi| {
            out.push_str("\\[");
            out.push_str(ansi);
            out.push_str("\\]");
        },
        false,
    )
}

fn wrap_ansi(
    prompt: &str,
    mut write_ansi: impl FnMut(&mut String, &str),
    escape_percent: bool,
) -> String {
    let mut out = String::with_capacity(prompt.len());
    let mut rest = prompt;

    while let Some(index) = rest.find("\x1b[") {
        push_visible(&mut out, &rest[..index], escape_percent);
        rest = &rest[index..];

        if let Some(sequence_len) = ansi_sgr_len(rest) {
            write_ansi(&mut out, &rest[..sequence_len]);
            rest = &rest[sequence_len..];
        } else {
            push_visible(&mut out, "\x1b[", escape_percent);
            rest = &rest[2..];
        }
    }

    push_visible(&mut out, rest, escape_percent);
    out
}

fn ansi_sgr_len(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    if !bytes.starts_with(b"\x1b[") {
        return None;
    }

    for (index, byte) in bytes.iter().enumerate().skip(2) {
        if *byte == b'm' {
            return Some(index + 1);
        }
        if !matches!(*byte, b'0'..=b'9' | b';') {
            return None;
        }
    }

    None
}

fn push_visible(out: &mut String, value: &str, escape_percent: bool) {
    if escape_percent {
        out.push_str(&value.replace('%', "%%"));
    } else {
        out.push_str(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi;

    #[test]
    fn leaves_raw_prompt_unchanged() {
        let prompt = format!(
            "{}host{} ~/src {}%{} ",
            ansi::WHITE,
            ansi::RESET,
            ansi::BOLD,
            ansi::RESET
        );

        assert_eq!(prompt, apply(&prompt, PromptEscape::None));
    }

    #[test]
    fn wraps_zsh_ansi_and_escapes_visible_percent() {
        let prompt = format!(
            "{}host{} 100% {}%{} ",
            ansi::WHITE,
            ansi::RESET,
            ansi::BOLD,
            ansi::RESET
        );

        assert_eq!(
            format!(
                "%{{{}%}}host%{{{}%}} 100%% %{{{}%}}%%%{{{}%}} ",
                ansi::WHITE,
                ansi::RESET,
                ansi::BOLD,
                ansi::RESET
            ),
            apply(&prompt, PromptEscape::Zsh)
        );
    }

    #[test]
    fn wraps_bash_ansi() {
        let prompt = format!(
            "{}host{} {}${} ",
            ansi::WHITE,
            ansi::RESET,
            ansi::GREEN,
            ansi::RESET
        );

        assert_eq!(
            format!(
                "\\[{}\\]host\\[{}\\] \\[{}\\]$\\[{}\\] ",
                ansi::WHITE,
                ansi::RESET,
                ansi::GREEN,
                ansi::RESET
            ),
            apply(&prompt, PromptEscape::Bash)
        );
    }

    #[test]
    fn leaves_plain_text_unchanged_except_zsh_percent() {
        assert_eq!("host ~/src $ ", apply("host ~/src $ ", PromptEscape::Bash));
        assert_eq!("host 50%% %% ", apply("host 50% % ", PromptEscape::Zsh));
    }
}
