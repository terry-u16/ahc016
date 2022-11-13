mod encoders;
mod graph;
mod utils;

use crate::graph::Graph;
use encoders::{clique::CliqueEncoder, Encoder};
use proconio::source::line::LineSource;
use proconio::*;
use std::{
    io::{self, BufReader, BufWriter, Stdin, Write as _},
    time::Instant,
};

#[derive(Debug, Clone)]
struct Input {
    graph_count: usize,
    error_ratio: f64,
    since: Instant,
}

impl Input {
    fn read(source: &mut LineSource<BufReader<Stdin>>) -> Self {
        input! {
            from source,
            graph_count: usize,
            error_ratio: f64
        }

        let since = Instant::now();

        Self {
            graph_count,
            error_ratio,
            since,
        }
    }
}

#[proconio::fastout]
fn main() {
    const QUERY_COUNT: usize = 100;
    let mut stdin = LineSource::new(BufReader::new(io::stdin()));
    let stdout = io::stdout();
    let stdout = &mut BufWriter::new(stdout.lock());
    let input = Input::read(&mut stdin);

    // グラフ生成
    let encoder = CliqueEncoder::new(input.graph_count);
    writeln!(stdout, "{}", encoder.graph_size()).unwrap();

    for i in 0..input.graph_count {
        let graph = encoder.encode(i);
        writeln!(stdout, "{}", graph.serialize()).unwrap();
    }

    let elapsed = Instant::now() - input.since;
    let each_duration = (5.0 - (elapsed.as_secs_f64() + 0.5)) / QUERY_COUNT as f64;
    stdout.flush().unwrap();

    // クエリ回答
    for q in 0..QUERY_COUNT {
        input! {
            from &mut stdin,
            graph: String
        }

        eprintln!("query: {}", q);
        let graph = Graph::deserialize(&graph, encoder.graph_size());
        writeln!(stdout, "{}", encoder.decode(&graph, each_duration)).unwrap();
        stdout.flush().unwrap();
    }

    let elapsed = Instant::now() - input.since;
    eprintln!("elapsed: {:.3}s", elapsed.as_secs_f64());
}
