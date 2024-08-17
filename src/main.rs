use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use rust_test_runner::cargo_test::TestEvent;
use rust_test_runner::cli::CliArgs;
use rust_test_runner::{convert, parse_test_code};
use serde_json as json;

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();
    let input_dir = cli_args
        .input_dir
        .canonicalize()
        .context("failed to canonicalize input dir")?;
    let output_dir = cli_args
        .output_dir
        .as_ref()
        .unwrap_or(&cli_args.input_dir)
        .canonicalize()
        .context("failed to canonicalize output dir")?;

    let mut irrelevant_path_prefix = input_dir.parent().unwrap().display().to_string();
    irrelevant_path_prefix.push('/');

    std::env::set_current_dir(&input_dir).context("failed to 'cd' into the input directory")?;

    let mut results_out = String::new();

    if !std::fs::metadata("Cargo.toml")
        .context("failed to read Cargo.toml")?
        .is_file()
    {
        results_out
            .push_str("WARNING: student did not upload Cargo.toml. This may cause build errors.\n");
    }

    let profile = determine_profile(&input_dir)
        .context("failed to determine the compilation profile of the solution")?;

    #[rustfmt::skip]
    let cargo_output = Command::new("timeout")
        .args([
            "-v", "15s",
            "cargo", "test",
                "--offline",
                "--profile", profile,
                "--",
                "--include-ignored",
                "-Z", "unstable-options",
                "--format", "json",
        ])
        .output()
        .context("failed to run cargo")?;

    // This regex replacement fixes a flakiness issue for tests that fail to compile.
    // For the test example-syntax-error,
    // the following two output lines may appear in random order:
    //
    // error: could not compile `leap` (lib) due to 2 previous errors
    // error: could not compile `leap` (lib test) due to 2 previous errors
    //
    // Therefore, we remove the stuff in the parentheses.
    let re = Regex::new(r"could not compile `(?<crate>.*)` \(.*\) (?<reason>.*)").unwrap();
    let cargo_stderr = String::from_utf8_lossy(&cargo_output.stderr);
    let deterministic_cargo_stderr =
        re.replace_all(&cargo_stderr, "could not compile `$crate` $reason");

    results_out.push_str(&deterministic_cargo_stderr);

    // if there is no test file at the standard location (tests/<slug>.rs),
    // pretend like the test file is empty
    let test_file =
        std::fs::read_to_string(format!("tests/{}.rs", cli_args.slug)).unwrap_or_default();
    let name_to_code = parse_test_code::parse_file(&test_file);

    let out = convert(
        serde_json::Deserializer::from_slice(&cargo_output.stdout).into_iter::<TestEvent>(),
        name_to_code,
    );
    let mut results_json = serde_json::to_string_pretty(&out)?;

    if results_out.contains("timeout: sending signal TERM") {
        results_json = String::from(
            r#"{"version": 2, "status": "error", "message": "One of the tests timed out"}"#,
        );
    } else if results_json.contains("probable build failure") {
        if results_out.contains("error: no matching package named") {
            results_json = format!(
                r#"{{"version": 2, "status": "error", "message": "{}"}}"#,
                escape(MISSING_CRATE_ERR_MSG)
            );
        } else {
            results_json = format!(
                r#"{{"version": 2, "status": "error", "message": "{}"}}"#,
                escape(&results_out)
            );
        }
    }

    // only display relative path in output to students
    results_json = results_json.replace(&irrelevant_path_prefix, "");

    std::fs::write(output_dir.join("results.json"), results_json)
        .context("failed to write results.json")?;

    Ok(())
}

/// usually `dev`. `release` if `custom.test-in-release-mode` is set.
///
/// documentation:
/// https://doc.rust-lang.org/cargo/reference/profiles.html
///
fn determine_profile(input_dir: &Path) -> Result<&'static str> {
    let config_path = input_dir.join(".meta/config.json");
    let Ok(config) = std::fs::read_to_string(config_path) else {
        return Ok("dev");
    };
    let config: json::Value =
        json::from_str(&config).context("failed to deserialize config file content to json")?;

    if config
        .get("custom")
        .and_then(|c| c.get("test-in-release-mode"))
        .is_some()
    {
        Ok("release")
    } else {
        Ok("dev")
    }
}

fn escape(s: &str) -> String {
    // escape_default turns "'" into "\'" which is incorrect in json
    // and needs to be reverted
    s.escape_default().to_string().replace("\\'", "'")
}

static MISSING_CRATE_ERR_MSG: &str = "\
It looks like you're using a crate which isn't supported by our test runner. \
Please see the file at the below URL to check which ones are supported. \
Please get in touch if you think your crate should be included \
or something else about the user experience could be improved.

List of available crates:
https://github.com/exercism/rust-test-runner/blob/main/local-registry/Cargo.toml

Exercism forum (Rust topic):
https://forum.exercism.org/c/programming/rust
";
