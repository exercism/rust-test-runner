use std::collections::HashMap;

pub fn parse_file(test_file: &str) -> HashMap<String, String> {
    let mut name_to_code = HashMap::new();
    let mut lines = test_file.lines();
    while let Some(line) = lines.by_ref().next() {
        if line.contains("#[test]") {
            let (name, code) = parse_function(&mut lines);
            name_to_code.insert(name, code);
        }
    }
    name_to_code
}

fn parse_function(lines: &mut std::str::Lines) -> (String, String) {
    let mut should_panic = None;
    while let Some(line) = lines.next() {
        if line.contains("#[ignore]") {
            continue;
        }
        if line.contains("#[should_panic") {
            should_panic = Some(line.trim());
        }
        if let Some((indent, signature)) = line.split_once("fn ") {
            let indent = format!("{indent}    "); // plus 4 spaces
            let name = signature.split(['<', '(']).next().unwrap().to_string();
            let mut body = parse_body(lines, indent);
            if let Some(should_panic) = should_panic {
                body = format!("// {should_panic}\n{body}");
            }
            return (name, body);
        }
    }
    panic!("did not find function definition")
}

fn parse_body(lines: &mut std::str::Lines, indent: String) -> String {
    let mut res = String::new();
    let function_end = format!("{}}}", &indent[4..]);
    for line in lines {
        if line.starts_with(&function_end) {
            res.pop(); // trailing newline
            return res;
        }
        res.push_str(line.strip_prefix(&indent).unwrap_or(line));
        res.push('\n');
    }
    panic!("end of function body not found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let input = "\
#[test]
fn foo() {
    first_line();
    second_line();
}";
        let expected = "\
first_line();
second_line();";
        let name_to_code = parse_file(input);
        assert_eq!(name_to_code["foo"], expected);
    }

    #[test]
    fn ignore() {
        let input = "\
#[test]
#[ignore]
fn foo() {
    first_line();
    second_line();
}";
        let expected = "\
first_line();
second_line();";
        let name_to_code = parse_file(input);
        assert_eq!(name_to_code["foo"], expected);
    }

    #[test]
    fn indented() {
        let input = "\
    #[test]
    fn foo() {
        first_line();
        second_line();
    }";
        let expected = "\
first_line();
second_line();";
        let name_to_code = parse_file(input);
        assert_eq!(name_to_code["foo"], expected);
    }

    #[test]
    fn should_panic() {
        let input = "\
#[test]
#[should_panic]
fn foo() {
    first_line();
    second_line();
}";
        let expected = "\
// #[should_panic]
first_line();
second_line();";
        let name_to_code = parse_file(input);
        assert_eq!(name_to_code["foo"], expected);
    }

    #[test]
    fn should_panic_with_msg() {
        let input = "\
#[test]
#[should_panic(\"reason\")]
fn foo() {
    first_line();
    second_line();
}";
        let expected = "\
// #[should_panic(\"reason\")]
first_line();
second_line();";
        let name_to_code = parse_file(input);
        assert_eq!(name_to_code["foo"], expected);
    }

    #[test]
    fn more_indentation() {
        let input = "\
#[test]
fn foo() {
    first_line();
    if condition {
        indented_line();
    }
}";
        let expected = "\
first_line();
if condition {
    indented_line();
}";
        let name_to_code = parse_file(input);
        assert_eq!(name_to_code["foo"], expected);
    }

    #[test]
    fn empty_line() {
        let input = "\
#[test]
fn foo() {
    first_line();

    second_line();
}";
        let expected = "\
first_line();

second_line();";
        let name_to_code = parse_file(input);
        assert_eq!(name_to_code["foo"], expected);
    }
}
