mod annealing;

use super::Encoder;
use crate::{graph::Graph, utils::ChangeMinMax};

#[derive(Debug, Clone)]
pub struct CliqueEncoder {
    graph_count: usize,
    graph_size: usize,
    k_arries: Vec<KAry>,
}

impl CliqueEncoder {
    pub fn new(graph_count: usize) -> Self {
        // とりあえず暫定値
        // 全bitが1になることがなければ少しケチれる
        let k_arries = vec![
            KAry::new(7, 5),
            KAry::new(12, 3),
            KAry::new(17, 2),
            KAry::new(22, 2),
            KAry::new(27, 2),
        ];

        let mut encoder = Self {
            graph_count,
            graph_size: 0,
            k_arries,
        };

        // 必要なグラフサイズを計算
        for i in 0..graph_count {
            let counts = encoder.to_base_k_num(i);
            let count: usize = counts
                .iter()
                .zip(encoder.k_arries.iter())
                .map(|(&c, k_ary)| c * k_ary.size)
                .sum();

            encoder.graph_size.change_max(count);
        }

        encoder
    }

    fn to_base_k_num(&self, mut index: usize) -> Vec<usize> {
        let mut mul: usize = self.k_arries.iter().map(|a| a.count).product();
        let mut counts = vec![];

        for k_ary in self.k_arries.iter().rev() {
            mul /= k_ary.count;
            let count = index / mul;
            index -= mul * count;
            counts.push(count);
        }

        counts.reverse();

        counts
    }

    fn create_graph(&self, counts: &[usize]) -> Graph {
        let mut v = 0;
        let mut graph = Graph::new(self.graph_size);

        for (&c, k_ary) in counts.iter().zip(self.k_arries.iter()) {
            for _ in 0..c {
                let begin = v;
                let end = v + k_ary.size;

                for i in begin..end {
                    for j in (i + 1)..end {
                        graph.connect(i, j);
                    }
                }

                v += k_ary.size;
            }
        }

        graph
    }

    fn predict(&self, graph: &Graph) -> usize {
        todo!()
    }
}

impl Encoder for CliqueEncoder {
    fn graph_size(&self) -> usize {
        self.graph_size
    }

    fn encode(&self, index: usize) -> Graph {
        let counts = self.to_base_k_num(index);
        self.create_graph(&counts)
    }

    fn decode(&self, graph: &Graph) -> usize {
        self.predict(graph)
    }
}

/// K進数（？）を表す構造体
#[derive(Debug, Clone, Copy)]
struct KAry {
    size: usize,
    count: usize,
}

impl KAry {
    fn new(size: usize, count: usize) -> Self {
        Self { size, count }
    }
}
