mod swap_node;

use self::swap_node::SwapNode;
use super::{binarygraph::BinaryGraph, state::State};
use rand_pcg::Pcg64Mcg;

pub trait Neighbor {
    fn apply(&self, graph: &BinaryGraph, state: &mut State);
    fn rollback(&self, graph: &BinaryGraph, state: &mut State);
}

pub struct NeighborGenerator;

impl NeighborGenerator {
    pub fn gen(&self, graph: &BinaryGraph, state: &State, rng: &mut Pcg64Mcg) -> Box<dyn Neighbor> {
        Box::new(SwapNode::gen(graph, state, rng))
    }
}
