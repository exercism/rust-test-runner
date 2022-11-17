pub fn format_test_name(name: String) -> String {
    let sanitized_name = name.replace("test_", "").replace("_", " ");
    let mut c = sanitized_name.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
