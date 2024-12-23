pub fn is_completed(completed: bool) -> String {
    if completed {
        "[x]".to_owned()
    } else {
        "[]".to_owned()
    }
}
