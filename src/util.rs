/// Returns `None`, or `Some` of a non-empty string.
pub fn non_empty_string(s: &str) -> Option<&str> {
    if s.is_empty() {None} else {Some(s)}
}
