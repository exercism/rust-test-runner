use anyhow::Result;
use clap::Parser;
use rust_test_runner::cargo_test::TestEvent;
use rust_test_runner::cli::CliArgs;
use rust_test_runner::convert;

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    dbg!(cli_args);

    let stdin = std::io::stdin();

    let out = convert(serde_json::Deserializer::from_reader(stdin.lock()).into_iter::<TestEvent>());

    let stdout = std::io::stdout();
    serde_json::to_writer_pretty(stdout.lock(), &out)?;

    Ok(())
}
