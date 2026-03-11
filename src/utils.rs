/// Detects the indentation of a JSON string.
pub fn detect_indent(json_str: &str) -> String {
    for line in json_str.lines() {
        if line.starts_with("  ") {
            let indent_len = line.find(|c: char| !c.is_whitespace()).unwrap_or(0);
            if indent_len > 0 {
                return line[..indent_len].to_string();
            }
        }
    }
    "  ".to_string() // Default to 2 spaces
}
