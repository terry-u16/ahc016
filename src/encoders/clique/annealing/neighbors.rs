mod change_node;
mod merge_groups;
mod separate_node;
mod split_group;

use rand::Rng;
use rand_pcg::Pcg64Mcg;

use super::state::State;
use crate::{
    encoders::clique::annealing::neighbors::{
        change_node::ChangeNode, merge_groups::MergeGroups, separate_node::SeparateNode,
        split_group::SplitGroup,
    },
    graph::Graph,
};

pub trait Neighbor {
    fn apply(&self, graph: &Graph, state: &mut State);
    fn rollback(&self, graph: &Graph, state: &mut State);
}

pub struct NeighborGenerator;

impl NeighborGenerator {
    pub fn gen(&self, graph: &Graph, state: &State, rng: &mut Pcg64Mcg) -> Box<dyn Neighbor> {
        loop {
            let neigh_type = rng.gen_range(0, 100);

            let neighbor: Option<Box<dyn Neighbor>> = if neigh_type < 40 {
                Self::into_box(ChangeNode::gen(graph, state, rng))
            } else if neigh_type < 80 {
                Self::into_box(SeparateNode::gen(graph, state, rng))
            } else if neigh_type < 90 {
                Self::into_box(MergeGroups::gen(graph, state, rng))
            } else {
                Self::into_box(SplitGroup::gen(graph, state, rng))
            };

            if let Some(neighbor) = neighbor {
                return neighbor;
            }
        }
    }

    fn into_box(neighbor: Option<impl Neighbor + 'static>) -> Option<Box<dyn Neighbor>> {
        // なんか.map()だとダメだった
        if let Some(neighbor) = neighbor {
            Some(Box::new(neighbor))
        } else {
            None
        }
    }
}
