use crate::{encoders::clique::annealing::state::State, graph::Graph};
use itertools::Itertools;
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;

use super::Neighbor;

#[derive(Debug, Clone)]
pub struct MergeGroups {
    parent_group: usize,
    child_group: usize,
    child_nodes: Vec<usize>,
}

impl MergeGroups {
    pub fn gen(_graph: &Graph, state: &State, rng: &mut Pcg64Mcg) -> Option<Self> {
        let sizes = state.get_group_size_list();
        let candidates = sizes
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, size)| *size >= 2)
            .collect_vec();

        if candidates.len() < 2 {
            return None;
        }

        let i = rng.gen_range(0, candidates.len());
        let j = (i + rng.gen_range(1, candidates.len())) % candidates.len();

        let (mut parent_group, size_p) = candidates[i];
        let (mut child_group, size_c) = candidates[j];

        // マージテク（と言うほどでもないが）
        if size_p < size_c {
            std::mem::swap(&mut parent_group, &mut child_group);
        }

        let child_nodes = state
            .groups()
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, g)| *g == child_group)
            .map(|(i, _)| i)
            .collect_vec();

        Some(Self {
            parent_group,
            child_group,
            child_nodes,
        })
    }
}

impl Neighbor for MergeGroups {
    fn apply(&self, graph: &Graph, state: &mut State) {
        for &v in self.child_nodes.iter() {
            state.change_group(graph, v, self.parent_group);
        }
    }

    fn rollback(&self, graph: &Graph, state: &mut State) {
        for &v in self.child_nodes.iter() {
            state.change_group(graph, v, self.child_group);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        encoders::clique::annealing::{neighbors::Neighbor, state::State},
        graph::Graph,
    };

    use super::MergeGroups;

    #[test]
    fn apply_test() {
        let graph = Graph::new(4);
        let mut state = State::init(&graph);
        state.change_group(&graph, 1, 0);
        state.change_group(&graph, 3, 2);

        let merge_groups = MergeGroups {
            parent_group: 0,
            child_group: 2,
            child_nodes: vec![2, 3],
        };

        merge_groups.apply(&graph, &mut state);
        assert_eq!(state.groups()[0], 0);
        assert_eq!(state.groups()[1], 0);
        assert_eq!(state.groups()[2], 0);
        assert_eq!(state.groups()[3], 0);

        merge_groups.rollback(&graph, &mut state);
        assert_eq!(state.groups()[0], 0);
        assert_eq!(state.groups()[1], 0);
        assert_eq!(state.groups()[2], 2);
        assert_eq!(state.groups()[3], 2);
    }
}
