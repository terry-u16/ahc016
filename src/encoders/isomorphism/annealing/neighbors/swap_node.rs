use super::Neighbor;
use crate::encoders::isomorphism::annealing::{binarygraph::BinaryGraph, state::State};
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone)]
pub struct SwapNode {
    group0: usize,
    group1: usize,
    index0: usize,
    index1: usize,
    prev_score: f64,
    self_counts_buffer: [i32; 6],
    cross_counts_buffer: [i32; 15],
}

impl SwapNode {
    pub fn gen(_graph: &BinaryGraph, state: &State, rng: &mut Pcg64Mcg) -> Self {
        let group_count = state.group_count();
        let group_size = state.group_size();

        let group0 = rng.gen_range(0, group_count);
        let group1 = (group0 + rng.gen_range(1, group_count)) % group_count;
        let index0 = rng.gen_range(0, group_size);
        let index1 = rng.gen_range(0, group_size);

        Self {
            group0,
            group1,
            index0,
            index1,
            prev_score: 0.0,
            self_counts_buffer: [0; 6],
            cross_counts_buffer: [0; 15],
        }
    }
}

impl Neighbor for SwapNode {
    fn apply(&mut self, graph: &BinaryGraph, state: &mut State) {
        state.swap_nodes(
            graph,
            self.group0,
            self.group1,
            self.index0,
            self.index1,
            &mut self.prev_score,
            &mut self.self_counts_buffer,
            &mut self.cross_counts_buffer,
        )
    }

    fn rollback(&mut self, graph: &BinaryGraph, state: &mut State) {
        state.revert_swap(
            graph,
            self.group0,
            self.group1,
            self.index0,
            self.index1,
            self.prev_score,
            &self.self_counts_buffer,
            &self.cross_counts_buffer,
        )
    }
}
