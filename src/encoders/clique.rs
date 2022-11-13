mod annealing;

use self::annealing::annealer::Annealer;
use super::Encoder;
use crate::{graph::Graph, utils::ChangeMinMax};
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct CliqueEncoder {
    /// 送信するグラフの種類数
    graph_count: usize,
    /// グラフの大きさ
    graph_size: usize,
    k_arries: Vec<KAry>,
}

impl CliqueEncoder {
    pub fn new(graph_count: usize) -> Self {
        // とりあえず暫定値
        // 全bitが1になることがなければ少しケチれる
        let k_arries = vec![
            KAry::new(7, 6, 5),
            KAry::new(12, 10, 3),
            KAry::new(17, 15, 2),
            KAry::new(22, 20, 2),
            KAry::new(27, 25, 2),
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

    fn expect(&self, graph: &Graph, duration: f64) -> usize {
        const MIN_VIS: usize = 8;
        let annealer = Annealer::new(false);
        let groups = annealer.run(graph, duration);
        let groups = groups.into_iter().filter(|s| *s >= MIN_VIS).collect_vec();
        eprintln!("{:?}", &groups);

        // 復号する
        let mut mul: usize = self.k_arries.iter().map(|a| a.count).product();
        let mut result = 0;
        let mut index = 0;

        for k_ary in self.k_arries.iter().rev() {
            mul /= k_ary.count;
            let mut count = 0;

            // 許容下限以上ならk番目と判断
            // TODO: DPをした方が良さそう
            while index < groups.len()
                && groups[index] >= k_ary.lower_bound
                && count + 1 < k_ary.count
            {
                let next = result + mul;

                // 整数Mを超える場合は無視
                if next >= self.graph_count {
                    break;
                }

                result = next;
                index += 1;
                count += 1;
            }
        }

        result
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

    fn decode(&self, graph: &Graph, duration: f64) -> usize {
        self.expect(graph, duration)
    }
}

/// K進数（？）を表す構造体
#[derive(Debug, Clone, Copy)]
struct KAry {
    /// 連結成分の大きさ
    size: usize,
    /// 連結成分の大きさの許容下限
    lower_bound: usize,
    /// 最大の数
    count: usize,
}

impl KAry {
    fn new(size: usize, lower_bound: usize, count: usize) -> Self {
        Self {
            size,
            lower_bound,
            count,
        }
    }
}
