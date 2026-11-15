use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        output.write_all(&rtla::dispatch(line?.as_bytes()))?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    Ok(())
}
