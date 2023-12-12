use crate::test_name_formatter::format_test_name;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

impl TestResult {
    pub fn ok(name: String) -> TestResult {
        TestResult {
            name: format_test_name(name),
            status: Status::Pass,
            message: None,
            output: None,
        }
    }

    pub fn fail(name: String, message: Option<String>) -> TestResult {
        let name = format_test_name(name);

        let (output, message) = match message.as_ref().and_then(|m| m.split_once("thread '")) {
            Some((output, message)) if !output.is_empty() => {
                (Some(output.to_owned()), Some(format!("thread '{message}")))
            }
            _ => (None, message),
        };

        // This note is attached to the error message of only one test case that fails,
        // but not always the same one. To avoid CI failing unnecessarily, this note
        // is stripped from all messages.
        // It's also not useful to students reading the output of the test runner,
        // as they can't set this environment variable in the test runner themselves.
        let message = message.map(|m| {
            m.trim_end_matches(
                "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n",
            )
            .to_owned()
        });

        TestResult {
            name,
            message,
            status: Status::Fail,
            output,
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
