use std::io::{self, BufRead, Write};

fn main() {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for input in io::stdin().lock().lines() {
        let input = input.expect("read oracle request");
        serde_json::to_writer(&mut output, &idealis_oracle::dispatch(input.as_bytes()))
            .expect("serialize oracle response");
        output.write_all(b"\n").expect("write newline");
        output.flush().expect("flush oracle response");
    }
}
