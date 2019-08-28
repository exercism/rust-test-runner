use std::io::{BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin();
    let mut stdin_lock = stdin.lock();
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();

    let mut line_buf = String::with_capacity(255);
    while let Ok(n) = stdin_lock.read_line(&mut line_buf) {
        if n == 0 {
            break;
        }
        write!(stdout_lock, "{}", line_buf)?;
        line_buf.clear();
    }
    Ok(())
}
