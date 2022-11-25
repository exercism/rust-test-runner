use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    Error,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub status: Status,
    pub message: Option<String>,
}

impl TestResult {
    pub fn ok(name: String) -> TestResult {
        TestResult {
            name: format_test_name(name),
            status: Status::Pass,
            message: None,
        }
    }

    pub fn fail(name: String, message: Option<String>) -> TestResult {
        TestResult {
            name: format_test_name(name),
            message,
            status: Status::Fail,
        }
    }
}

/* Removes "test" if it is at the beginning of the word, replaces an underscore with a whitespace, turns the name of test into title case, trims extra whitespaces
*
* e.g. test_year_divisible_by_400_but_not_by_125_is_still_a_leap_year -> Year divisible by 400 but not by 125 is still a leap year
*
* Why is this important? See https://github.com/exercism/exercism/issues/6544 */
fn format_test_name(name: String) -> String {
    let name = name.to_lowercase().replace("_", " ");
    let mut name:Vec<_> = name.split_whitespace().collect();
    if name[0] == "test"{
        name.remove(0);
    }
    let name = name.join(" ");
    let mut c = name.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub version: u8,
    pub status: Status,
    pub message: Option<String>,
    pub tests: Vec<TestResult>,
}
