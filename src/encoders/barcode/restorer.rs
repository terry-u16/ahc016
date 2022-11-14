use super::annealing::binarygraph::BinaryGraph;
use crate::{
    encoders::barcode::annealing::{annealer::Annealer, state::State},
    graph::Graph,
};
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone)]
pub struct Restorer;

impl Restorer {
    pub fn restore(&self, graph: &Graph, duration: f64, seed: u128) -> Graph {
        let binary_graph = BinaryGraph::new(graph);
        let annealer = Annealer::new(false);
        let mut rng = Pcg64Mcg::new(seed);

        let state = State::init_rand(&binary_graph, &mut rng);
        let state = annealer.run(&binary_graph, state, duration, &mut rng);

        let mut restored_graph = Graph::new(graph.n);

        for row in 0..graph.n {
            let i = state.permutation()[row];
            for col in (row + 1)..graph.n {
                let j = state.permutation()[col];

                if graph[i][j] {
                    restored_graph.connect(row, col);
                }
            }
        }

        restored_graph
    }
}
