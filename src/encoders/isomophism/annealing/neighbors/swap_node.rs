use super::Neighbor;
use crate::encoders::isomophism::annealing::{binarygraph::BinaryGraph, state::State};
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone, Copy)]
pub struct SwapNode {
    group0: usize,
    group1: usize,
    index0: usize,
    index1: usize,
}

impl SwapNode {
    pub fn gen(graph: &BinaryGraph, state: &State, rng: &mut Pcg64Mcg) -> Self {
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
        }
    }
}

impl Neighbor for SwapNode {
    fn apply(&self, graph: &BinaryGraph, state: &mut State) {
        state.swap_nodes(graph, self.group0, self.group1, self.index0, self.index1)
    }

    fn rollback(&self, graph: &BinaryGraph, state: &mut State) {
        state.swap_nodes(graph, self.group0, self.group1, self.index0, self.index1)
    }
}
