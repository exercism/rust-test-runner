use transform_output::cargo_test::TestEvent;
use transform_output::convert;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin();

    let out = convert(serde_json::Deserializer::from_reader(stdin.lock()).into_iter::<TestEvent>());

    let stdout = std::io::stdout();
    serde_json::to_writer_pretty(stdout.lock(), &out)?;

    Ok(())
}
