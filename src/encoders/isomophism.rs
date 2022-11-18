use self::{
    annealing::{binarygraph::BinaryGraph, state::State},
    checker::{generate_isompic_graphs, IsomophicChecker, Vf2Checker},
};
use super::Encoder;
use crate::{
    encoders::isomophism::annealing::annealer::Annealer, graph::Graph, utils::ChangeMinMax,
};
use rand_pcg::Pcg64Mcg;

mod annealing;
mod checker;

#[derive(Debug, Clone)]
pub struct IsomophismEncoder {
    graphs: Vec<Graph>,
    /// 送信するグラフの種類数
    graph_count: usize,
    /// グラフの大きさ
    graph_size: usize,
    /// 冗長性考慮前のグラフの大きさ
    original_graph_size: usize,
    /// 冗長性
    redundancy: usize,
}

impl IsomophismEncoder {
    pub fn new(graph_count: usize, error_ratio: f64) -> Self {
        let (mut graphs, original_graph_size) = generate_isompic_graphs(graph_count);
        graphs.truncate(graph_count);
        let redundancy = Self::get_redundancy(original_graph_size, error_ratio);
        let graph_size = original_graph_size * redundancy;

        Self {
            graphs,
            graph_count,
            graph_size,
            original_graph_size,
            redundancy,
        }
    }

    fn get_redundancy(original_graph_size: usize, error_ratio: f64) -> usize {
        if error_ratio == 0.0 {
            1
        } else if error_ratio <= 0.03 {
            3
        } else if error_ratio <= 0.05 {
            4
        } else if error_ratio <= 0.13 {
            5
        } else if error_ratio <= 0.15 {
            6
        } else if error_ratio <= 0.19 {
            7
        } else if error_ratio <= 0.22 {
            8
        } else if error_ratio <= 0.25 {
            10
        } else if error_ratio <= 0.28 {
            12
        } else if error_ratio <= 0.32 {
            14
        } else {
            100 / original_graph_size
        }
    }

    fn restore(
        &self,
        graph: &BinaryGraph,
        annealer: &Annealer,
        duration: f64,
        rng: &mut Pcg64Mcg,
    ) -> Option<usize> {
        let state = State::init_rand(&graph, self.original_graph_size, rng);
        let state = annealer.annealing(&graph, state, duration);
        let graph = state.restore_graph();
        let checker = Vf2Checker::new(&graph);

        for (i, g) in self.graphs.iter().enumerate() {
            if checker.is_isomorphic(g) {
                return Some(i);
            }
        }

        eprintln!("failed to decode.");
        None
    }
}

impl Encoder for IsomophismEncoder {
    fn graph_size(&self) -> usize {
        self.graph_size
    }

    fn encode(&self, index: usize) -> Graph {
        let original_graph = &self.graphs[index];

        let mut graph = Graph::new(self.graph_size);
        // クリーク内
        for i in 0..original_graph.n {
            for x in 0..self.redundancy {
                for y in (x + 1)..self.redundancy {
                    let u = i * self.redundancy + x;
                    let v = i * self.redundancy + y;
                    graph.connect(u, v);
                }
            }
        }

        // クリーク間
        for i in 0..original_graph.n {
            for j in (i + 1)..original_graph.n {
                for x in 0..self.redundancy {
                    for y in 0..self.redundancy {
                        let u = i * self.redundancy + x;
                        let v = j * self.redundancy + y;
                        if original_graph[i][j] {
                            graph.connect(u, v);
                        }
                    }
                }
            }
        }

        graph
    }

    fn decode(&self, graph: &Graph, duration: f64) -> usize {
        let mut rng = Pcg64Mcg::new(42);
        let graph = BinaryGraph::new(graph);
        let annealer = Annealer::new(false);
        let mut votes = vec![0; self.graph_count];

        const TRIAL_COUNT: usize = 1;
        let each_duration = duration / TRIAL_COUNT as f64;

        // 多数決を取る
        for _ in 0..TRIAL_COUNT {
            if let Some(i) = self.restore(&graph, &annealer, each_duration, &mut rng) {
                votes[i] += 1;
            }
        }

        let mut max_votes = 0;
        let mut max_index = 0;

        for (i, &c) in votes.iter().enumerate() {
            if max_votes.change_max(c) {
                max_index = i;
            }
        }

        max_index
    }
}
