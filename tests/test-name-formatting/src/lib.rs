/* Removes "test_" from test-name, replaces underscores with a whitespace, turns it into title case, trims extra whitespaces
*
* e.g. test_year_divisible_by_400_but_not_by_125_is_still_a_leap_year -> Year divisible by 400 but not by 125 is still a leap year
*
* Why is this important? See https://github.com/exercism/exercism/issues/6544 */
pub fn format_test_name(name: String) -> String {
    let name = name.replace("test_", "").replace("_", " ");
    let name:Vec<_> = name.split_whitespace().collect();
    let name = name.join(" ").to_lowercase();
    let mut c = name.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
