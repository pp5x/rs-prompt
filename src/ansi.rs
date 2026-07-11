use crate::vcs::VcsType;

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const UNDERLINE: &str = "\x1b[4m";
pub const UNDERLINE_OFF: &str = "\x1b[24m";

pub const WHITE: &str = "\x1b[37m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";
pub const LIGHT_RED: &str = "\x1b[91m";
pub const BLUE: &str = "\x1b[34m";
pub const GREEN: &str = "\x1b[32m";
pub const LIGHT_GREEN: &str = "\x1b[92m";

pub fn fg_for_vcs(kind: VcsType) -> &'static str {
    match kind {
        VcsType::Jj => YELLOW,
        VcsType::Git => RED,
        VcsType::Repo => WHITE,
    }
}
