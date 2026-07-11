use std::fmt::{self, Write};

use crate::ansi;

pub fn short_hostname(hostname: &str) -> &str {
    hostname
        .split_once('.')
        .map_or(hostname, |(short, _)| short)
}

pub fn write_highlighted(out: &mut String, hostname: &str) -> fmt::Result {
    let mut in_digits = false;

    for ch in hostname.chars() {
        let is_digit = ch.is_ascii_digit();
        if is_digit && !in_digits {
            out.write_str(ansi::UNDERLINE)?;
            in_digits = true;
        } else if !is_digit && in_digits {
            out.write_str(ansi::UNDERLINE_OFF)?;
            in_digits = false;
        }
        out.write_char(ch)?;
    }

    if in_digits {
        out.write_str(ansi::UNDERLINE_OFF)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortens_at_first_dot() {
        assert_eq!("build42", short_hostname("build42.lab"));
        assert_eq!("workstation", short_hostname("workstation"));
    }

    #[test]
    fn highlights_digit_runs() {
        let mut out = String::new();
        write_highlighted(&mut out, "db12zone34").unwrap();
        assert_eq!(
            format!(
                "db{}12{}zone{}34{}",
                ansi::UNDERLINE,
                ansi::UNDERLINE_OFF,
                ansi::UNDERLINE,
                ansi::UNDERLINE_OFF
            ),
            out
        );
    }
}
