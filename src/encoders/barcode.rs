mod annealing;
mod restorer;
use super::Encoder;
use crate::{encoders::barcode::restorer::Restorer, graph::Graph};
use itertools::Itertools;

/// 2進数のバーコードで識別するエンコーダ
#[derive(Debug, Clone)]
pub struct BarCodeEncoder {
    graph_size: usize,
    graph_count: usize,
    bar_widths: Vec<usize>,
}

impl BarCodeEncoder {
    pub fn new(graph_count: usize, error_ratio: f64) -> Self {
        let bar_widths = Self::get_bar_widths(graph_count, error_ratio);
        let graph_size = bar_widths.iter().sum();

        Self {
            graph_size,
            graph_count,
            bar_widths,
        }
    }

    fn get_bar_widths(graph_count: usize, _error_ratio: f64) -> Vec<usize> {
        let mut bar_widths = vec![14; 7];
        let max_index = (graph_count - 1) as u64;
        let digits = 64 - max_index.leading_zeros();
        bar_widths.truncate(digits as usize);
        bar_widths.reverse();

        bar_widths
    }

    fn restore_bits(&self, duration: f64, graph: &Graph) -> Vec<bool> {
        // K回焼きなましを回して多数決を取る
        const TRIAL_COUNT: usize = 3;
        let mut votes = vec![0; self.bar_widths.len()];
        let duration = duration / TRIAL_COUNT as f64;
        let restorer = Restorer;

        for trial in 0..TRIAL_COUNT {
            let seed = trial as u128 + 42;
            let graph = restorer.restore(graph, duration, seed);
            let mut row = 0;

            for (d, &w) in self.bar_widths.iter().enumerate() {
                let mut count = 0;

                for _ in 0..w {
                    for col in (row + 1)..self.graph_size {
                        if graph[row][col] {
                            count += 1;
                        } else {
                            count -= 1;
                        }
                    }

                    row += 1;
                }

                votes[d] += if count > 0 { 1 } else { -1 };
            }
        }

        let bits = votes.iter().map(|c| *c > 0).collect_vec();
        bits
    }
}

impl Encoder for BarCodeEncoder {
    fn graph_size(&self) -> usize {
        self.graph_size
    }

    fn encode(&self, index: usize) -> Graph {
        let mut graph = Graph::new(self.graph_size);

        let mut row = 0;

        for (d, &w) in self.bar_widths.iter().enumerate() {
            let bit = ((index >> d) & 1) > 0;
            for _ in 0..w {
                if bit {
                    for col in (row + 1)..self.graph_size {
                        graph.connect(row, col);
                    }
                }

                row += 1;
            }
        }

        graph
    }

    fn decode(&self, graph: &Graph, duration: f64) -> usize {
        let bits = self.restore_bits(duration, graph);

        // ビット列をindexに復元
        let mut result = 0;

        for (d, &b) in bits.iter().enumerate() {
            if !b {
                continue;
            }

            let next = result + (1 << d);

            if next < self.graph_count {
                result = next;
            }
        }

        result
    }
}
