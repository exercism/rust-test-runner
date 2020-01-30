pub mod cargo_test;
pub mod output;

use cargo_test as ct;
use output as o;

/// convert a stream of test events into a single test result
pub fn convert<I, E>(events: I) -> o::Output
where
    I: Iterator<Item = Result<ct::TestEvent, E>>,
    E: serde::de::Error + std::fmt::Display,
{
    let mut out = o::Output {
        status: o::Status::Error,
        message: Some("no tests detected; probable build failure".into()),
        tests: Vec::new(),
    };
    for (idx, event) in events.enumerate() {
        let event = match event {
            Ok(e) => e,
            Err(e) => {
                out.status = o::Status::Error;
                out.message = Some(format!("test event misparse at idx {}: {}", idx, e));
                break;
            }
        };
        if event.etype != ct::EventType::Test {
            continue;
        }
        let name = match event.name {
            Some(n) => n,
            None => {
                out.status = o::Status::Error;
                out.message = Some(format!("a test event had no name at idx {}", idx));
                break;
            }
        };
        match event.event {
            ct::Event::Started => continue,
            ct::Event::Ok => {
                // don't override failures with subsequent successes
                if out.status == o::Status::Error {
                    out.status = o::Status::Pass;
                }
                out.message = None;
                out.tests.push(o::TestResult::ok(name));
            }
            ct::Event::Failed => {
                out.status = o::Status::Fail;
                out.message = None;
                out.tests.push(o::TestResult::fail(name, event.stdout));
            }
            ct::Event::Ignored => {
                out.status = o::Status::Error;
                out.message = Some(format!("test {} was ignored", name));
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pass() {}

    #[test]
    #[ignore]
    fn fail() {
        assert!(false);
    }

    const TEST_DATA: &'static str = r#"
{ "type": "suite", "event": "started", "test_count": 3 }
{ "type": "test", "event": "started", "name": "cargo_test::test::test_parse_events" }
{ "type": "test", "event": "started", "name": "test::fail" }
{ "type": "test", "name": "cargo_test::test::test_parse_events", "event": "ok" }
{ "type": "test", "event": "started", "name": "test::pass" }
{ "type": "test", "name": "test::pass", "event": "ok" }
{ "type": "test", "name": "test::fail", "event": "failed", "stdout": "thread 'test::fail' panicked at 'assertion failed: false', src/lib.rs:52:9\nstack backtrace:\n   0: backtrace::backtrace::libunwind::trace\n             at /cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.35/src/backtrace/libunwind.rs:88\n   1: backtrace::backtrace::trace_unsynchronized\n             at /cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.35/src/backtrace/mod.rs:66\n   2: std::sys_common::backtrace::_print\n             at src/libstd/sys_common/backtrace.rs:47\n   3: std::sys_common::backtrace::print\n             at src/libstd/sys_common/backtrace.rs:36\n   4: std::panicking::default_hook::{{closure}}\n             at src/libstd/panicking.rs:200\n   5: std::panicking::default_hook\n             at src/libstd/panicking.rs:211\n   6: std::panicking::rust_panic_with_hook\n             at src/libstd/panicking.rs:477\n   7: std::panicking::begin_panic\n             at /rustc/760226733e940cb375f791e894fbb554555eeb01/src/libstd/panicking.rs:411\n   8: transform_output::test::fail\n             at src/lib.rs:52\n   9: transform_output::test::fail::{{closure}}\n             at src/lib.rs:51\n  10: core::ops::function::FnOnce::call_once\n             at /rustc/760226733e940cb375f791e894fbb554555eeb01/src/libcore/ops/function.rs:235\n  11: <alloc::boxed::Box<F> as core::ops::function::FnOnce<A>>::call_once\n             at /rustc/760226733e940cb375f791e894fbb554555eeb01/src/liballoc/boxed.rs:922\n  12: __rust_maybe_catch_panic\n             at src/libpanic_unwind/lib.rs:80\n  13: std::panicking::try\n             at /rustc/760226733e940cb375f791e894fbb554555eeb01/src/libstd/panicking.rs:275\n  14: std::panic::catch_unwind\n             at /rustc/760226733e940cb375f791e894fbb554555eeb01/src/libstd/panic.rs:394\n  15: test::run_test::run_test_inner::{{closure}}\n             at src/libtest/lib.rs:1408\nnote: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.\n" }
{ "type": "suite", "event": "failed", "passed": 2, "failed": 1, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 0 }
    "#;

    #[test]
    fn test_convert() {
        let out =
            convert(serde_json::Deserializer::from_str(TEST_DATA).into_iter::<ct::TestEvent>());
        assert_eq!(out.status, o::Status::Fail);
        for test in out.tests {
            if test.name == "test::fail" {
                assert_eq!(test.status, o::Status::Fail);
                assert!(test.message.is_some());
            } else {
                assert_eq!(test.status, o::Status::Pass);
                assert!(test.message.is_none());
            }
        }
    }
}
