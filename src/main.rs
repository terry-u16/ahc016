use encoders::{clique::CliqueEncoder, Encoder};
#[allow(unused_imports)]
use proconio::*;
use std::io::{self, BufWriter, Write as _};
mod encoders;
mod graph;
mod utils;

#[derive(Debug, Clone)]
struct Input {}

#[proconio::fastout]
fn main() {
    let encoder = CliqueEncoder::new(100);
    let stdout = io::stdout();
    let stdout = &mut BufWriter::new(stdout.lock());

    writeln!(stdout, "{}", encoder.graph_size()).unwrap();

    for i in 0..100 {
        let graph = encoder.encode(i);
        writeln!(stdout, "{}", &graph).unwrap();
    }

    stdout.flush().unwrap();
}
