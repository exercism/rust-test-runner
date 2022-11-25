/// Removes `test_` from test-name, replaces underscores with a whitespace, turns it into title case, trims extra whitespaces
///
/// ### Example
///
/// ```rust
/// use transform_output::test_name_formatter::format_test_name;
/// 
/// let input = String::from("test___cast_spell_with_insufficient_mana");
/// let output = String::from("Cast spell with insufficient mana");
/// assert_eq!(format_test_name(input), output);
/// ```
///
/// #### Why is this important?
/// See [Long test names and test-summaries have visible overflow #6544](https://github.com/exercism/exercism/issues/6544)
pub fn format_test_name(name: String) -> String {
  let name = name.to_lowercase().replace("_", " ");
  let mut name: Vec<_> = name.split_whitespace().collect();
  if name[0] == "test" {
      name.remove(0);
  }
  let name = name.join(" ");
  let mut c = name.chars();
  match c.next() {
      None => String::new(),
      Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
  }
}