/// Returns `None`, or `Some` of a non-empty string.
pub fn non_empty_string(s: &str) -> Option<&str> {
    if s.is_empty() {None} else {Some(s)}
}

/// Like `split_first` but for `&str`.
pub fn split_first_str(s: &str) -> Option<(char, &str)> {
    let mut chars = s.chars();
    chars.next().map(|c| (c, chars.as_str()))
}
