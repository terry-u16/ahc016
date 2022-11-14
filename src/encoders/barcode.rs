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

    fn get_bar_widths(graph_count: usize, error_ratio: f64) -> Vec<usize> {
        let mut bar_widths = if error_ratio <= 0.05 {
            vec![8, 7, 6, 6, 6, 5, 5]
        } else if error_ratio <= 0.1 {
            vec![11, 10, 9, 9, 8, 8, 7]
        } else if error_ratio <= 0.15 {
            vec![15, 13, 13, 13, 12, 10, 8]
        } else {
            vec![17, 16, 16, 15, 15, 13, 8]
        };

        let max_index = (graph_count - 1) as u64;
        let digits = 64 - max_index.leading_zeros();
        bar_widths.truncate(digits as usize + 1);
        bar_widths.reverse();

        // 拡大可能なら拡大する
        if error_ratio > 0.2 {
            let size: usize = bar_widths.iter().sum();
            let zoom_ratio = 100.0 / size as f64;
            bar_widths = bar_widths
                .iter()
                .map(|&w| (w as f64 * zoom_ratio) as usize)
                .collect_vec();
        }

        bar_widths
    }

    fn restore_bits(&self, duration: f64, graph: &Graph) -> Vec<bool> {
        let restorer = Restorer;
        let mut trial = 0;

        loop {
            let seed = trial as u128 + 42;
            let graph = restorer.restore(graph, duration * 0.5, seed);
            let mut row = 0;
            let mut bits = vec![false; self.bar_widths.len()];

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

                bits[d] = if count > 0 { true } else { false };
            }

            if bits.iter().filter(|&&b| b).count() % 2 == 0 {
                return bits;
            }

            trial += 1;
        }
    }
}

impl Encoder for BarCodeEncoder {
    fn graph_size(&self) -> usize {
        self.graph_size
    }

    fn encode(&self, index: usize) -> Graph {
        let mut graph = Graph::new(self.graph_size);

        let mut row = 0;
        let pairty = index.count_ones() as usize % 2;
        let index = index | (pairty << (self.bar_widths.len() - 1));

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
        let mut bits = self.restore_bits(duration, graph);
        bits[self.bar_widths.len() - 1] &= false;

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
