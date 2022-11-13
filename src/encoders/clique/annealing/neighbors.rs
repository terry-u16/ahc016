mod change_node;
mod merge_groups;
mod separate_node;
mod split_group;

use super::state::State;
use crate::graph::Graph;

pub trait Neighbor {
    fn apply(&self, graph: &Graph, state: &mut State);
    fn rollback(&self, graph: &Graph, state: &mut State);
}
