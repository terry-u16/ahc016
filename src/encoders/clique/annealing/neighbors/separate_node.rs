use super::Neighbor;
use crate::{encoders::clique::annealing::state::State, graph::Graph};
use rand::Rng;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone, Copy)]
pub struct SeparateNode {
    node: usize,
    group: usize,
    prev_group: usize,
}

impl SeparateNode {
    pub fn gen(graph: &Graph, state: &State, rng: &mut Pcg64Mcg) -> Option<Self> {
        const MAX_TRIAL: usize = 100;
        let group_sizes = state.get_group_size_list();

        for _ in 0..MAX_TRIAL {
            let node = rng.gen_range(0, graph.n);
            let prev_group = state.groups()[node];
            if group_sizes[prev_group] == 1 {
                continue;
            }

            // mexを求める
            for (i, &size) in group_sizes.iter().enumerate() {
                if size == 0 {
                    return Some(Self {
                        node,
                        group: i,
                        prev_group,
                    });
                }
            }

            unreachable!();
        }

        None
    }
}

impl Neighbor for SeparateNode {
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

    use super::SeparateNode;

    #[test]
    fn apply_test() {
        let graph = Graph::new(4);
        let mut state = State::init(&graph);
        state.change_group(&graph, 1, 0);
        let separate_node = SeparateNode {
            node: 1,
            group: 1,
            prev_group: 0,
        };

        separate_node.apply(&graph, &mut state);
        assert_eq!(state.groups()[1], 1);

        separate_node.rollback(&graph, &mut state);
        assert_eq!(state.groups()[1], 0);
    }
}
