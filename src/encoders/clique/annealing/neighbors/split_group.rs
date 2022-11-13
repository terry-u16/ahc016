use crate::{encoders::clique::annealing::state::State, graph::Graph};
use itertools::Itertools;
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;

use super::Neighbor;

#[derive(Debug, Clone)]
pub struct SplitGroup {
    parent_group: usize,
    child_group: usize,
    target_nodes: Vec<usize>,
}

impl SplitGroup {
    pub fn gen(graph: &Graph, state: &State, rng: &mut Pcg64Mcg) -> Option<Self> {
        let group_sizes = state.get_group_size_list();
        let candidates = group_sizes
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, size)| *size >= 2)
            .map(|(i, _)| i)
            .collect_vec();

        if candidates.len() == 0 {
            return None;
        }

        let group_id = *candidates.choose(rng).unwrap();
        let nodes = state
            .groups()
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, group)| *group == group_id)
            .map(|(i, _)| i)
            .collect_vec();

        let graph = construct_graph(graph, &nodes);
        let cut = max_cut(&graph);

        let mut target = false;
        let children_count = cut.iter().filter(|b| **b).count();
        let parent_count = nodes.len() - children_count;

        if parent_count < children_count {
            target = true;
        }

        let nodes = nodes
            .iter()
            .copied()
            .zip(cut.iter())
            .filter(|(_, b)| **b == target)
            .map(|(v, _)| v)
            .collect_vec();

        // mexを求める
        for (i, &size) in group_sizes.iter().enumerate() {
            if size == 0 {
                return Some(Self {
                    parent_group: group_id,
                    child_group: i,
                    target_nodes: nodes,
                });
            }
        }

        unreachable!();
    }
}

impl Neighbor for SplitGroup {
    fn apply(&self, graph: &Graph, state: &mut State) {
        for &v in self.target_nodes.iter() {
            state.change_group(graph, v, self.child_group);
        }
    }

    fn rollback(&self, graph: &Graph, state: &mut State) {
        for &v in self.target_nodes.iter() {
            state.change_group(graph, v, self.parent_group);
        }
    }
}

fn construct_graph(graph: &Graph, nodes: &[usize]) -> Vec<Vec<i32>> {
    let mut new_graph = vec![vec![0; nodes.len()]; nodes.len()];

    for (i, &u) in nodes.iter().enumerate() {
        for (j, &v) in nodes.iter().enumerate() {
            if i == j {
                continue;
            }

            // 辺が張られていないところはカットしたいので1/そうでなければ-1
            new_graph[i][j] = if graph[u][v] { -1 } else { 1 };
        }
    }

    new_graph
}

/// 局所探索法により最大カット問題を解く
fn max_cut(graph: &Vec<Vec<i32>>) -> Vec<bool> {
    let mut state = vec![1; graph.len()];

    // TODO: 高速化
    'main: loop {
        for (i, (edges, &group)) in graph.iter().zip(state.iter()).enumerate() {
            let mut score_diff = 0;

            for (j, (&e, &g)) in edges.iter().zip(state.iter()).enumerate() {
                if i == j {
                    continue;
                }

                let same = g * group;
                score_diff += e * same;
            }

            if score_diff > 0 {
                state[i] = -state[i];
                continue 'main;
            }
        }

        break;
    }

    state.iter().map(|v| *v == 1).collect_vec()
}

#[cfg(test)]
mod test {
    use super::SplitGroup;
    use crate::{
        encoders::clique::annealing::{neighbors::Neighbor, state::State},
        graph::Graph,
    };

    use super::{construct_graph, max_cut};

    #[test]
    fn apply_test() {
        let graph = Graph::new(4);
        let mut state = State::init(&graph);
        state.change_group(&graph, 1, 0);
        state.change_group(&graph, 2, 0);
        state.change_group(&graph, 3, 0);

        let split_group = SplitGroup {
            parent_group: 0,
            child_group: 1,
            target_nodes: vec![2, 3],
        };

        split_group.apply(&graph, &mut state);
        assert_eq!(state.groups()[0], 0);
        assert_eq!(state.groups()[1], 0);
        assert_eq!(state.groups()[2], 1);
        assert_eq!(state.groups()[3], 1);

        split_group.rollback(&graph, &mut state);
        assert_eq!(state.groups()[0], 0);
        assert_eq!(state.groups()[1], 0);
        assert_eq!(state.groups()[2], 0);
        assert_eq!(state.groups()[3], 0);
    }

    #[test]
    fn construct_graph_test() {
        // 0--1
        // |  |
        // 2--3
        // ↑の[2, 0, 3]からなる誘導部分グラフ
        let mut original_graph = Graph::new(4);
        original_graph.connect(0, 1);
        original_graph.connect(0, 2);
        original_graph.connect(1, 3);
        original_graph.connect(2, 3);
        let nodes = vec![2, 0, 3];

        let graph = construct_graph(&original_graph, &nodes);
        let expected = vec![vec![0, -1, -1], vec![-1, 0, 1], vec![-1, 1, 0]];

        assert_eq!(expected, graph);
    }

    #[test]
    fn max_cut_test() {
        // 1 -- 2 -- 3 というグラフをカットする
        let graph = vec![vec![0, 1, -1], vec![1, 0, 1], vec![-1, 1, 0]];
        let result = max_cut(&graph);
        assert!(result == vec![true, false, true] || result == vec![false, true, false]);
    }
}
