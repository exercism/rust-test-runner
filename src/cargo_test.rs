use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Test,
    Suite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Started,
    Ok,
    Ignored,
    Failed,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestEvent {
    #[serde(rename = "type")]
    pub etype: EventType,
    pub event: Event,
    pub name: Option<String>,
    pub stdout: Option<String>,
    pub passed: Option<u32>,
    pub failed: Option<u32>,
    pub allowed_fail: Option<u32>,
    pub ignored: Option<u32>,
    pub measured: Option<u32>,
    pub filtered_out: Option<u32>,
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_DATA: &'static str = r#"
{ "type": "suite", "event": "started", "test_count": 2 }
{ "type": "test", "event": "started", "name": "test::fail" }
{ "type": "test", "event": "started", "name": "test::pass" }
{ "type": "test", "name": "test::pass", "event": "ok" }
{ "type": "test", "name": "test::fail", "event": "ok" }
{ "type": "suite", "event": "ok", "passed": 2, "failed": 0, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 0 }
{ "type": "suite", "event": "started", "test_count": 0 }
{ "type": "suite", "event": "ok", "passed": 0, "failed": 0, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 0 }
{ "type": "suite", "event": "started", "test_count": 0 }
{ "type": "suite", "event": "ok", "passed": 0, "failed": 0, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 0 }
    "#;

    #[test]
    fn test_parse_events() {
        // there are 10 events in the data set; all must parse
        assert_eq!(
            serde_json::Deserializer::from_str(TEST_DATA)
                .into_iter::<TestEvent>()
                .count(),
            10
        );
        assert!(serde_json::Deserializer::from_str(TEST_DATA)
            .into_iter::<TestEvent>()
            .all(|e| e.is_ok()),);
    }
}
