use serde::{Deserialize, Serialize};
use crate::test_name_formatter::format_test_name;

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


#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub version: u8,
    pub status: Status,
    pub message: Option<String>,
    pub tests: Vec<TestResult>,
}
