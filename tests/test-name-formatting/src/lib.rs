pub fn format_test_name(name: String) -> String {
    let name = name.replace("_", " ");
    let mut name:Vec<_> = name.split_whitespace().collect();
    if name[0] == "test"{
        name.remove(0);
    }
    let name = name.join(" ").to_lowercase();
    let mut c = name.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
