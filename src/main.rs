use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use rust_test_runner::cargo_test::TestEvent;
use rust_test_runner::cli::CliArgs;
use rust_test_runner::convert;
use serde_json as json;

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();
    let output_dir = cli_args
        .output_dir
        .canonicalize()
        .context("failed to canonicalize output dir")?;

    std::env::set_current_dir(&cli_args.input_dir)
        .context("failed to 'cd' into the input directory")?;

    let mut results_out = String::new();

    if !std::fs::metadata("Cargo.toml")
        .context("failed to read Cargo.toml")?
        .is_file()
    {
        results_out
            .push_str("WARNING: student did not upload Cargo.toml. This may cause build errors.\n");
    }

    let profile = determine_profile(&cli_args.input_dir)
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

    let out = convert(
        serde_json::Deserializer::from_slice(&cargo_output.stdout).into_iter::<TestEvent>(),
    );
    let mut results_json = serde_json::to_string_pretty(&out)?;

    if results_out.contains("timeout: sending signal TERM") {
        results_json = String::from(
            r#"{"version": 2, "status": "error", "message": "One of the tests timed out"}"#,
        );
    } else if results_json.contains("probable build failure") {
        results_json = format!(
            r#"{{"version": 2, "status": "error", "message": "{}"}}"#,
            results_out.escape_default()
        );
    }

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
