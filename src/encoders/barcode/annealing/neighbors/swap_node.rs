use super::Neighbor;
use crate::encoders::barcode::annealing::{binarygraph::BinaryGraph, state::State};
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Copy, Clone)]
pub struct SwapNode {
    u: usize,
    v: usize,
    prev_score: i32,
}

impl SwapNode {
    pub fn gen(graph: &BinaryGraph, state: &State, rng: &mut Pcg64Mcg) -> Self {
        let prev_score = state.score();
        let u = rng.gen_range(0, graph.n);
        let v = (u + rng.gen_range(1, graph.n)) % graph.n;

        Self { u, v, prev_score }
    }
}

impl Neighbor for SwapNode {
    fn apply(&self, graph: &BinaryGraph, state: &mut State) {
        assert!(state.score() == self.prev_score);
        state.swap_node(graph, self.u, self.v);
    }

    fn rollback(&self, graph: &BinaryGraph, state: &mut State) {
        state.swap_node_with(graph, self.u, self.v, self.prev_score);
    }
}
