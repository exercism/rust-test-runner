use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    CorePass,
    Fail,
    Error(String),
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
            name,
            status: Status::Pass,
            message: None,
        }
    }

    pub fn fail(name: String, message: Option<String>) -> TestResult {
        TestResult {
            name,
            message,
            status: Status::Fail,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub status: Status,
    pub tests: Vec<TestResult>,
}
