use super::Neighbor;
use crate::{encoders::clique::annealing::state::State, graph::Graph};
use rand::{seq::SliceRandom, Rng};
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone, Copy)]
pub struct ChangeNode {
    node: usize,
    group: usize,
    prev_group: usize,
}

impl ChangeNode {
    pub fn gen(graph: &Graph, state: &State, rng: &mut Pcg64Mcg) -> Option<Self> {
        let groups = state.get_group_list();

        if groups.len() <= 1 {
            return None;
        }

        loop {
            let node = rng.gen_range(0, graph.n);
            let group = *groups.choose(rng).unwrap();
            let prev_group = state.groups()[node];

            if prev_group != group {
                return Some(Self {
                    node,
                    group,
                    prev_group,
                });
            }
        }
    }
}

impl Neighbor for ChangeNode {
    fn apply(&self, graph: &Graph, state: &mut State) {
        state.change_group(graph, self.node, self.group);
    }

    fn rollback(&self, graph: &Graph, state: &mut State) {
        state.change_group(graph, self.node, self.prev_group);
    }
}

#[cfg(test)]
mod test {
    use crate::{
        encoders::clique::annealing::{neighbors::Neighbor, state::State},
        graph::Graph,
    };

    use super::ChangeNode;

    #[test]
    fn apply_test() {
        let graph = Graph::new(4);
        let mut state = State::init(&graph);
        let change_node = ChangeNode {
            node: 1,
            group: 0,
            prev_group: 1,
        };

        change_node.apply(&graph, &mut state);
        assert_eq!(state.groups()[1], 0);

        change_node.rollback(&graph, &mut state);
        assert_eq!(state.groups()[1], 1);
    }
}
