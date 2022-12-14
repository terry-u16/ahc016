use crate::graph::Graph;

use super::binarygraph::BinaryGraph;
use itertools::Itertools;
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone)]
pub struct State {
    group_count: usize,
    group_size: usize,
    groups: Vec<Vec<usize>>,
    groups_u128: Vec<u128>,
    graph_u128: Vec<u128>,
    self_counts: Vec<i32>,
    cross_counts: Vec<i32>,
    score: f64,
    score_coef: f64,
}

impl State {
    pub fn init_rand(
        graph: &BinaryGraph,
        group_count: usize,
        score_coef: f64,
        rng: &mut Pcg64Mcg,
    ) -> Self {
        let mut permutation = (0..graph.n).collect_vec();
        permutation.shuffle(rng);

        let mut groups = vec![vec![]; group_count];

        for (i, &p) in permutation.iter().enumerate() {
            groups[i % group_count].push(p);
        }

        Self::new(graph, groups, score_coef)
    }

    fn new(graph: &BinaryGraph, groups: Vec<Vec<usize>>, score_coef: f64) -> Self {
        let group_count = groups.len();
        let group_size = graph.n / group_count;
        assert!(graph.n == group_count * group_size);

        let mut graph_u128 = vec![0; graph.n];

        for i in 0..graph.n {
            for j in 0..graph.n {
                let edges = &mut graph_u128[i];

                if graph[i][j] == 1 {
                    *edges |= 1 << j;
                }
            }
        }

        let mut groups_u128 = vec![0; group_count];

        for (group, u128) in groups.iter().zip(groups_u128.iter_mut()) {
            for &v in group.iter() {
                *u128 |= 1 << v;
            }
        }

        let mut state = Self {
            group_count,
            groups,
            group_size,
            groups_u128,
            graph_u128,
            self_counts: vec![0; group_count],
            cross_counts: vec![0; group_count * (group_count - 1) / 2],
            score: 0.0,
            score_coef,
        };

        state.update_score_all(graph);
        state
    }

    pub fn group_count(&self) -> usize {
        self.group_count
    }

    pub fn group_size(&self) -> usize {
        self.group_size
    }

    pub fn score(&self) -> f64 {
        self.score
    }

    pub fn swap_nodes(
        &mut self,
        _graph: &BinaryGraph,
        mut g0: usize,
        mut g1: usize,
        mut i0: usize,
        mut i1: usize,
        prev_score: &mut f64,
        self_counts_buf: &mut [i32],
        cross_counts_buf: &mut [i32],
    ) {
        assert!(g0 != g1);
        if g0 > g1 {
            std::mem::swap(&mut g0, &mut g1);
            std::mem::swap(&mut i0, &mut i1);
        }

        *prev_score = self.score;
        let len = self.self_counts.len();
        self_counts_buf[..len].copy_from_slice(&self.self_counts);
        let len = self.cross_counts.len();
        cross_counts_buf[..len].copy_from_slice(&self.cross_counts);

        unsafe {
            self.sub_relative_counts(g0, g1, i0, i1);
            self.swap_inner(g0, i0, g1, i1);
            self.add_relative_counts(g0, g1, i0, i1);
        }

        self.update_score_from_counts();
    }

    pub fn revert_swap(
        &mut self,
        _graph: &BinaryGraph,
        mut g0: usize,
        mut g1: usize,
        mut i0: usize,
        mut i1: usize,
        prev_score: f64,
        self_counts_buf: &[i32],
        cross_counts_buf: &[i32],
    ) {
        assert!(g0 != g1);
        if g0 > g1 {
            std::mem::swap(&mut g0, &mut g1);
            std::mem::swap(&mut i0, &mut i1);
        }

        let len = self.self_counts.len();
        self.self_counts.copy_from_slice(&self_counts_buf[..len]);
        let len = self.cross_counts.len();
        self.cross_counts.copy_from_slice(&cross_counts_buf[..len]);

        self.swap_inner(g0, i0, g1, i1);

        self.score = prev_score;
    }

    fn swap_inner(&mut self, g0: usize, i0: usize, g1: usize, i1: usize) {
        let (first, second) = self.groups.split_at_mut(g0 + 1);
        let u = &mut first[g0][i0];
        let v = &mut second[g1 - g0 - 1][i1];
        std::mem::swap(u, v);

        let xor = (1 << *u) | (1 << *v);
        self.groups_u128[g0] ^= xor;
        self.groups_u128[g1] ^= xor;
    }

    // popcnt????????????????????????1.5??????????????????????????????
    #[target_feature(enable = "popcnt")]
    unsafe fn sub_relative_counts(&mut self, g0: usize, g1: usize, i0: usize, i1: usize) {
        assert!(g0 < g1);

        // ???????????????????????????????????????????????????????????????????????????????????????
        // ??????????????????????????????????????????????????????add_relative_counts()???????????????????????????????????????
        for &(g0, i0) in [(g0, i0), (g1, i1)].iter() {
            let u = self.groups[g0][i0];
            let edges = &self.graph_u128[u];

            for (g1, group) in self.groups_u128.iter().enumerate() {
                if g0 == g1 {
                    // self
                    let counts = &mut self.self_counts[g0];

                    // plus - minus = plus - (group_size - plus) = 2 * plus - group_size
                    // group_size ?????????????????????????????????
                    let plus = (edges & group).count_ones() as i32;
                    *counts -= 2 * plus;
                } else {
                    // cross
                    let (g0, g1) = if g0 < g1 { (g0, g1) } else { (g1, g0) };
                    let index = self.cross_index(g0, g1);
                    let counts = &mut self.cross_counts[index];

                    let plus = (edges & group).count_ones() as i32;
                    *counts -= 2 * plus;
                }
            }
        }
    }

    #[target_feature(enable = "popcnt")]
    unsafe fn add_relative_counts(&mut self, g0: usize, g1: usize, i0: usize, i1: usize) {
        assert!(g0 < g1);

        for &(g0, i0) in [(g0, i0), (g1, i1)].iter() {
            let u = self.groups[g0][i0];
            let edges = self.graph_u128[u];

            for (g1, group) in self.groups_u128.iter().enumerate() {
                if g0 == g1 {
                    // self
                    let counts = &mut self.self_counts[g0];
                    let plus = (edges & group).count_ones() as i32;
                    *counts += 2 * plus;
                } else {
                    // cross
                    let (g0, g1) = if g0 < g1 { (g0, g1) } else { (g1, g0) };
                    let index = self.cross_index(g0, g1);
                    let counts = &mut self.cross_counts[index];

                    let plus = (edges & group).count_ones() as i32;
                    *counts += 2 * plus;
                }
            }
        }
    }

    pub fn update_score_all(&mut self, graph: &BinaryGraph) {
        for c in self.self_counts.iter_mut() {
            *c = 0;
        }

        for c in self.cross_counts.iter_mut() {
            *c = 0;
        }

        // ??????????????????count?????????
        for (group, count) in self.groups.iter().zip(self.self_counts.iter_mut()) {
            for i in 0..group.len() {
                let u = group[i];
                let edges = &graph[u];

                for j in (i + 1)..group.len() {
                    let v = group[j];
                    *count += edges[v];
                }
            }
        }

        // ??????????????????count?????????
        for g0 in 0..self.group_count {
            let group0 = &self.groups[g0];
            for g1 in (g0 + 1)..self.group_count {
                let group1 = &self.groups[g1];
                let index = self.cross_index(g0, g1);
                let count = &mut self.cross_counts[index];

                for &u in group0.iter() {
                    let edges = &graph[u];
                    for &v in group1.iter() {
                        *count += edges[v];
                    }
                }
            }
        }

        self.update_score_from_counts();
    }

    fn update_score_from_counts(&mut self) {
        let mut inside_score = 0;

        // ???????????????????????????????????? (??i 2max(ci, 0))
        for &c in self.self_counts.iter() {
            inside_score += c.max(0);
        }

        let mut outside_score = 0;

        // ???????????????????????????????????? (??ij |cij|)
        for c in self.cross_counts.iter() {
            outside_score += c.abs();
        }

        self.score = inside_score as f64 * self.score_coef + outside_score as f64;
    }

    pub fn restore_graph(&self) -> Graph {
        let mut restored_graph = Graph::new(self.group_count);

        for i in 0..self.group_count {
            for j in (i + 1)..self.group_count {
                if self.cross_counts[self.cross_index(i, j)] > 0 {
                    restored_graph.connect(i, j);
                }
            }
        }

        restored_graph
    }

    fn cross_index(&self, i: usize, j: usize) -> usize {
        let index = i * self.group_count + j;

        // ??????????????????????????????????????????????????????
        let exceeded = (i + 1) * (i + 2) / 2;
        index - exceeded
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;
    use rand_pcg::Pcg64Mcg;

    use crate::{encoders::isomorphism::annealing::binarygraph::BinaryGraph, graph::Graph};

    use super::State;

    #[test]
    fn score_test() {
        let graph = gen_graph();
        let groups = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let state = State::new(&graph, groups, 2.0);

        let expected = state.score;
        let actual = (2 * 2 * 3 + 9) as f64;

        assert_eq!(expected, actual);
    }

    #[test]
    fn swap_test() {
        let graph = gen_graph();
        let groups = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let mut state = State::new(&graph, groups, 2.0);

        // ??????0???3???swap
        let mut prev_score = 0.0;
        let mut buffer1 = [0; 6];
        let mut buffer2 = [0; 15];
        state.swap_nodes(
            &graph,
            0,
            1,
            0,
            0,
            &mut prev_score,
            &mut buffer1,
            &mut buffer2,
        );
        let actual = state.score;

        state.update_score_all(&graph);
        let expected = state.score;

        assert_eq!(expected, actual);
        assert_eq!(state.groups[0][0], 3);
        assert_eq!(state.groups[1][0], 0);
    }

    #[test]
    fn rand_swap_test() {
        const N: usize = 50;
        const GROUP_COUNT: usize = 5;
        const TRIAL_COUNT: usize = 1000;

        let mut graph = Graph::new(N);
        let mut rng = Pcg64Mcg::new(42);
        for i in 0..N {
            for j in (i + 1)..N {
                if rng.gen_bool(0.5) {
                    graph.connect(i, j);
                }
            }
        }

        let graph = BinaryGraph::new(&graph);
        let mut state = State::init_rand(&graph, GROUP_COUNT, 2.0, &mut rng);

        for _ in 0..TRIAL_COUNT {
            let g0 = rng.gen_range(0, state.group_count);
            let g1 = (g0 + rng.gen_range(1, state.group_count)) % state.group_count;
            let i0 = rng.gen_range(0, N / GROUP_COUNT);
            let i1 = rng.gen_range(0, N / GROUP_COUNT);

            let mut prev_score = 0.0;
            let mut buffer1 = [0; 6];
            let mut buffer2 = [0; 15];
            state.swap_nodes(
                &graph,
                g0,
                g1,
                i0,
                i1,
                &mut prev_score,
                &mut buffer1,
                &mut buffer2,
            );

            if rng.gen_bool(0.5) {
                state.revert_swap(&graph, g0, g1, i0, i1, prev_score, &buffer1, &buffer2);
            }

            let actual = state.score;

            state.update_score_all(&graph);
            let expected = state.score;
            assert_eq!(expected, actual);
        }
    }

    fn gen_graph() -> BinaryGraph {
        // \##...
        // #\#...
        // ##\...
        // ...\##
        // ...#\#
        // ...##\
        let mut graph = Graph::new(6);
        graph.connect(0, 1);
        graph.connect(0, 2);
        graph.connect(1, 2);
        graph.connect(3, 4);
        graph.connect(3, 5);
        graph.connect(4, 5);

        BinaryGraph::new(&graph)
    }
}
