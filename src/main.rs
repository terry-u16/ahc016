mod encoders;
mod graph;
mod utils;

use crate::{encoders::isomorphism::IsomorphismEncoder, graph::Graph};
use encoders::Encoder;
use proconio::source::line::LineSource;
use proconio::*;
use std::{
    io::{self, BufReader, BufWriter, Stdin, Write as _},
    time::Instant,
};

const DEFAULT_QUERY_COUNT: usize = 1000;

#[derive(Debug, Clone, Copy)]
struct AppArgs {
    query_count: usize,
    bits: Option<usize>,
    redundancy: Option<usize>,
}

impl AppArgs {
    fn read() -> Self {
        let query_count = if std::env::args().len() < 2 {
            DEFAULT_QUERY_COUNT
        } else {
            std::env::args().nth(1).unwrap().parse().unwrap()
        };

        let bits = if std::env::args().len() < 3 {
            None
        } else {
            Some(std::env::args().nth(2).unwrap().parse().unwrap())
        };

        let redundancy = if std::env::args().len() < 4 {
            None
        } else {
            Some(std::env::args().nth(3).unwrap().parse().unwrap())
        };

        Self {
            query_count,
            bits,
            redundancy,
        }
    }
}

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

fn main() {
    let app_args = AppArgs::read();
    let mut stdin = LineSource::new(BufReader::new(io::stdin()));
    let stdout = io::stdout();
    let stdout = &mut BufWriter::new(stdout.lock());
    let input = Input::read(&mut stdin);

    // グラフ生成
    let encoder = IsomorphismEncoder::new(
        input.graph_count,
        input.error_ratio,
        app_args.bits,
        app_args.redundancy,
    );

    writeln!(stdout, "{}", encoder.graph_size()).unwrap();

    for i in 0..input.graph_count {
        let graph = encoder.encode(i);
        writeln!(stdout, "{}", graph.serialize()).unwrap();
    }

    let elapsed = Instant::now() - input.since;
    let each_duration = (5.0 - (elapsed.as_secs_f64() + 0.5)) / DEFAULT_QUERY_COUNT as f64;
    stdout.flush().unwrap();

    // クエリ回答
    for q in 0..app_args.query_count {
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
