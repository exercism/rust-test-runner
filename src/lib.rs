use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
pub enum EventType {
    #[strum(serialize = "test", to_string = "test")]
    Test,
    #[strum(serialize = "suite", to_string = "suite")]
    Suite,
}
#[derive(Debug, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
pub enum Event {
    #[strum(serialize = "started", to_string = "started")]
    Started,
    #[strum(serialize = "ok", to_string = "ok")]
    Ok,
}

#[derive(Debug, PartialEq, Eq,  Serialize, Deserialize)]
pub struct TestEvent {
    pub etype: EventType,
    pub event: Event,
    pub name: Option<String>,
    pub passed: Option<u32>,
    pub failed: Option<u32>,
    pub allowed_fail: Option<u32>,
    pub ignored: Option<u32>,
    pub measured: Option<u32>,
    pub filtered_out: Option<u32>,
}

#[cfg(test)]
mod test {
    #[test]
    fn pass() {}

    #[test]
    #[should_panic]
    fn fail() {
        assert!(false);
    }
}
