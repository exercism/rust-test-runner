pub mod cargo_test;
pub mod cli;
pub mod output;
pub mod parse_test_code;
pub mod test_name_formatter;

use std::collections::HashMap;

use cargo_test as ct;
use output as o;

/// convert a stream of test events into a single test result
pub fn convert<I, E>(events: I, name_to_code: HashMap<String, String>) -> o::Output
where
    I: Iterator<Item = Result<ct::Event, E>>,
    E: serde::de::Error + std::fmt::Display,
{
    let mut out = o::Output {
        version: 2,
        status: o::Status::Error,
        message: Some("no tests detected; probable build failure".into()),
        tests: Vec::new(),
    };
    let mut doctest_cache = HashMap::new();
    for (idx, event) in events.enumerate() {
        let event = match event {
            Err(e) => {
                out.status = o::Status::Error;
                out.message = Some(format!("test event misparse at idx {}: {}", idx, e));
                break;
            }
            Ok(ct::Event::Test(e)) => e,
            _ => continue, // ignore other events
        };
        let name = match event.name {
            Some(n) => n,
            None => {
                out.status = o::Status::Error;
                out.message = Some(format!("a test event had no name at idx {}", idx));
                break;
            }
        };
        let (name, test_code) = if name.contains("src/") {
            // We're dealing with a doctest, those contain "src/" in their name.
            //
            // example name:
            // "macros/src/compile_fail_tests.rs - compile_fail_tests::_ONLY_ARROW (line 48)"
            //
            // The "macros/" prefix is optional, in case the user manually
            // declared a workspace. That's why we ignore that prefix.
            //
            // We parse the files lazily here, because only few exercises
            // contain doctests. Closure for error boundary.
            (|| {
                let mut words = name.split_ascii_whitespace();
                let file_name = words.next()?.split('/').next_back()?;
                let item_name = words.nth(1)?.split("::").last()?.trim_start_matches("_");
                let line = words.nth(1)?.trim_end_matches(")").parse::<usize>().ok()? - 1;
                if !doctest_cache.contains_key(file_name) {
                    let mut line_to_code = HashMap::new();
                    let content = std::fs::read_to_string(format!("src/{file_name}")).ok()?;
                    let mut lines = content.lines().enumerate();
                    'find_doctest: while let Some((i, line)) = lines.next() {
                        if !line.starts_with("/// ```") {
                            continue;
                        }
                        // doctest block start, gather code lines
                        let mut code = if line.contains("compile_fail") {
                            // Doctests marked as "compile_fail" are otherwise
                            // unmarked. Users could get confused by tests that
                            // pass if their code doesn't even compile.
                            String::from("// This code must not compile:\n")
                        } else {
                            String::new()
                        };
                        for (_, line) in lines.by_ref() {
                            let Some(line) = line.strip_prefix("///") else {
                                // doc comment ended before end of code block.
                                // ignore this malformed doctest.
                                continue 'find_doctest;
                            };
                            let Some(line) = line.strip_prefix(' ') else {
                                // empty line inside code block.
                                // skip, to keep test code short.
                                continue;
                            };
                            if line.starts_with("```") {
                                // doctest block end
                                code.pop(); // trim trailing newline
                                line_to_code.insert(i, code);
                                continue 'find_doctest;
                            }
                            code.push_str(line);
                            code.push('\n');
                        }
                        // no end of code block found. very strange. ignore.
                    }
                    doctest_cache.insert(file_name.to_owned(), line_to_code);
                }
                let test_code = doctest_cache.get(file_name)?.get(&line)?;
                Some((item_name.into(), test_code.into()))
            })()
            .unwrap_or_else(|| (name.clone(), TEST_CODE_NOT_FOUND_MSG.into()))
        } else {
            let test_code = name_to_code
                .get(&name)
                .map(String::as_str)
                .unwrap_or(TEST_CODE_NOT_FOUND_MSG)
                .to_string();
            (name, test_code)
        };
        match event.event {
            ct::EventKind::Started => continue,
            ct::EventKind::Ok => {
                // don't override failures with subsequent successes
                if out.status == o::Status::Error {
                    out.status = o::Status::Pass;
                }
                out.message = None;
                out.tests.push(o::TestResult::ok(name, test_code));
            }
            ct::EventKind::Failed => {
                out.status = o::Status::Fail;
                out.message = None;
                out.tests
                    .push(o::TestResult::fail(name, test_code, event.stdout));
            }
            ct::EventKind::Ignored => {
                out.status = o::Status::Error;
                out.message = Some(format!("test {} was ignored", name));
                break;
            }
        }
    }
    out.tests.sort();
    out
}

static TEST_CODE_NOT_FOUND_MSG: &str = "\
It looks like the test runner failed to retrieve the code for this test. \
Please consider reporting this on the forum so we can try to fix it. \
Thanks!

https://forum.exercism.org/c/exercism/bugs-and-features/126";

#[cfg(test)]
mod test {
    use super::*;

    const TEST_DATA: &str = r#"
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
        let out = convert(
            serde_json::Deserializer::from_str(TEST_DATA).into_iter(),
            HashMap::new(),
        );
        assert_eq!(out.status, o::Status::Fail);
        for test in out.tests {
            if test.name == "Test::fail" {
                assert_eq!(test.status, o::Status::Fail);
                assert!(test.message.is_some());
            } else {
                assert_eq!(test.status, o::Status::Pass);
                assert!(test.message.is_none());
            }
        }
    }
}
