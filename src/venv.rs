pub fn basename(path: &str) -> &str {
    if path.is_empty() {
        return "";
    }

    let mut end = path.len();
    let bytes = path.as_bytes();
    while end > 1 && bytes[end - 1] == b'/' {
        end -= 1;
    }

    let trimmed = &path[..end];
    trimmed.rsplit_once('/').map_or(trimmed, |(_, name)| name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_basename() {
        assert_eq!("myenv", basename("/opt/venvs/myenv"));
        assert_eq!("myenv", basename("/opt/venvs/myenv/"));
        assert_eq!("myenv", basename("myenv"));
    }
}
