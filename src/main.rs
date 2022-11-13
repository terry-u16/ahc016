use encoders::{clique::CliqueEncoder, Encoder};
#[allow(unused_imports)]
use proconio::*;
#[allow(unused_imports)]
use rand::prelude::*;

mod encoders;
mod graph;
mod utils;

#[derive(Debug, Clone)]
struct Input {}

fn main() {
    let encoder = CliqueEncoder::new(100);

    for i in 0..100 {
        let graph = encoder.encode(i);
        eprintln!("{}", &graph);
    }

    eprintln!("{}", encoder.graph_size());
}
