use std::io::{self, BufRead, Write};

fn main() {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for input in io::stdin().lock().lines() {
        let input = input.expect("read oracle request");
        output
            .write_all(&idealis_oracle::dispatch_json(input.as_bytes()))
            .expect("write oracle response");
        output.write_all(b"\n").expect("write newline");
        output.flush().expect("flush oracle response");
    }
}
