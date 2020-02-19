use transform_output::cargo_test::TestEvent;
use transform_output::convert;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let cargo_out = std::fs::read_to_string(&args[1])?;
    let stderr = std::fs::read_to_string(&args[2])?;

    let out = convert(
        serde_json::Deserializer::from_str(cargo_out.as_str()).into_iter::<TestEvent>(),
        stderr,
    );

    let stdout = std::io::stdout();
    serde_json::to_writer_pretty(stdout.lock(), &out)?;

    Ok(())
}
