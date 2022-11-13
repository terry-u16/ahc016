use crate::{graph::Graph, utils::ChangeMinMax};

use super::Encoder;

/// 二項分布を考慮し、辺の数によって識別するエンコーダ
#[derive(Debug, Clone)]
pub struct BinomialEncoder {
    graph_size: usize,
    graph_count: usize,
    initial_values: Vec<f64>,
    expected_values: Vec<f64>,
}

impl BinomialEncoder {
    pub fn new(graph_count: usize, error_ratio: f64) -> Self {
        let mut graph_size = !0;
        let mut initial_values = vec![];
        let mut expected_values = vec![];

        for size in (4..=100).rev() {
            let edge_count = size * (size - 1) / 2;

            if edge_count + 1 < graph_count {
                break;
            }

            let (error_count, error_std_div) = Self::calc_error_count(size, error_ratio);

            // 適当に3σ安全側に取っておく
            let error_count = error_count + 3.0 * error_std_div;

            let mut inits = vec![];
            let mut exps = vec![];
            let mut std_devs = vec![];

            for i in 0..graph_count {
                let ratio = i as f64 / (graph_count - 1) as f64;
                let init = edge_count as f64 * ratio;
                let (exp, std_dev) = Self::calc_expected_edge_count(size, error_count, init);
                inits.push(init);
                exps.push(exp);
                std_devs.push(std_dev);
            }

            if graph_size == !0 || Self::check_mergin(&exps, &std_devs) {
                graph_size = size;
                initial_values = inits;
                expected_values = exps;
            }
        }

        Self {
            graph_size,
            graph_count,
            initial_values,
            expected_values,
        }
    }

    /// あるグラフサイズでエラーが発生する辺の数の(期待値, 標準偏差)を求める
    fn calc_error_count(graph_size: usize, error_ratio: f64) -> (f64, f64) {
        let edge_count = graph_size * (graph_size - 1) / 2;
        let expected = Self::expected_value(edge_count, error_ratio);
        let std_dev = Self::standard_deviation(edge_count, error_ratio);

        // 適当に2σくらい取っておく
        (expected, std_dev)
    }

    /// ノイズ発生後にONになる辺の数の(期待値, 標準偏差)を求める
    fn calc_expected_edge_count(
        graph_size: usize,
        error_count: f64,
        initial_count: f64,
    ) -> (f64, f64) {
        let edge_count = graph_size * (graph_size - 1) / 2;
        let filled_ratio = initial_count / edge_count as f64;

        let turn_on_count = error_count * (1.0 - filled_ratio);
        let turn_off_count = error_count * filled_ratio;
        let expected_diff = turn_on_count - turn_off_count;
        let expected = initial_count + expected_diff;
        let std_dev = Self::standard_deviation(error_count.round() as usize, filled_ratio);

        (expected, std_dev)
    }

    fn check_mergin(expected_values: &[f64], std_devs: &[f64]) -> bool {
        for i in 0..(expected_values.len() - 1) {
            let diff = expected_values[i + 1] - expected_values[i];

            // 分散は加法性が成り立つ（らしい）
            let variance1 = std_devs[i] * std_devs[i];
            let variance2 = std_devs[i + 1] * std_devs[i + 1];
            let std_dev = (variance1 + variance2).sqrt();

            // 差の半分を超えるとNG・差の変化は標準偏差の2倍
            // 3σ取っておけばええやろ
            let mergin = diff / 2.0 - std_dev * 2.0 * 3.0;

            if mergin < 0.0 {
                return false;
            }
        }

        true
    }

    fn expected_value(n: usize, p: f64) -> f64 {
        n as f64 * p
    }

    fn variance(n: usize, p: f64) -> f64 {
        n as f64 * p * (1.0 - p)
    }

    fn standard_deviation(n: usize, p: f64) -> f64 {
        Self::variance(n, p).sqrt()
    }
}

impl Encoder for BinomialEncoder {
    fn graph_size(&self) -> usize {
        self.graph_size
    }

    fn encode(&self, index: usize) -> crate::graph::Graph {
        let mut graph = Graph::new(self.graph_size);
        let needed = self.initial_values[index].round() as usize;

        let mut count = 0;

        'main: for i in 0..self.graph_size {
            for j in (i + 1)..self.graph_size {
                if count >= needed {
                    break 'main;
                }

                graph.connect(i, j);
                count += 1;
            }
        }

        graph
    }

    fn decode(&self, graph: &crate::graph::Graph, _duration: f64) -> usize {
        let mut count = 0;

        for i in 0..self.graph_size {
            for j in (i + 1)..self.graph_size {
                if graph[i][j] {
                    count += 1;
                }
            }
        }

        // 一番近いやつを探す
        let mut best_index = !0;
        let mut best_diff = std::f64::MAX;

        for i in 0..self.graph_count {
            let diff = (count as f64 - self.expected_values[i]).abs();

            if best_diff.change_min(diff) {
                best_index = i;
            }
        }

        best_index
    }
}
