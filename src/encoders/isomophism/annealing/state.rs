use super::binarygraph::BinaryGraph;
use itertools::Itertools;
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone)]
pub struct State {
    group_count: usize,
    group_size: usize,
    groups: Vec<Vec<usize>>,
    self_counts: Vec<i32>,
    cross_counts: Vec<Vec<i32>>,
    score: i32,
}

impl State {
    pub fn init_rand(graph: &BinaryGraph, group_count: usize, rng: &mut Pcg64Mcg) -> Self {
        let mut permutation = (0..graph.n).collect_vec();
        permutation.shuffle(rng);

        let mut groups = vec![vec![]; group_count];

        for (i, &p) in permutation.iter().enumerate() {
            groups[i % group_count].push(p);
        }

        Self::new(graph, groups)
    }

    fn new(graph: &BinaryGraph, groups: Vec<Vec<usize>>) -> Self {
        let group_count = groups.len();
        let group_size = graph.n / group_count;
        assert!(graph.n == group_count * group_size);

        let mut state = Self {
            group_count,
            groups,
            group_size,
            self_counts: vec![0; group_count],
            cross_counts: vec![vec![0; group_count]; group_count],
            score: 0,
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

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn get_groups(&self) -> Vec<Vec<usize>> {
        self.groups.clone()
    }

    pub fn swap_nodes(
        &mut self,
        graph: &BinaryGraph,
        mut g0: usize,
        mut g1: usize,
        mut i0: usize,
        mut i1: usize,
    ) {
        assert!(g0 != g1);
        if g0 > g1 {
            std::mem::swap(&mut g0, &mut g1);
            std::mem::swap(&mut i0, &mut i1);
        }

        self.sub_relative_counts(graph, g0, g1, i0, i1);

        let temp = self.groups[g0][i0];
        self.groups[g0][i0] = self.groups[g1][i1];
        self.groups[g1][i1] = temp;

        self.add_relative_counts(graph, g0, g1, i0, i1);

        self.update_score_from_counts();
    }

    fn sub_relative_counts(
        &mut self,
        graph: &BinaryGraph,
        g0: usize,
        g1: usize,
        i0: usize,
        i1: usize,
    ) {
        assert!(g0 < g1);

        for &(g0, i0) in [(g0, i0), (g1, i1)].iter() {
            let u = self.groups[g0][i0];
            let edges = &graph[u];

            for (g1, group) in self.groups.iter().enumerate() {
                if g0 == g1 {
                    // self
                    let counts = &mut self.self_counts[g0];

                    for &v in group.iter() {
                        *counts -= edges[v];
                    }
                } else {
                    // cross
                    let (g0, g1) = if g0 < g1 { (g0, g1) } else { (g1, g0) };
                    let counts = &mut self.cross_counts[g0][g1];

                    for &v in group.iter() {
                        *counts -= edges[v];
                    }
                }
            }
        }

        // 引きすぎた分を足す
        let u = self.groups[g0][i0];
        let v = self.groups[g1][i1];
        self.self_counts[g0] += graph[u][u];
        self.self_counts[g1] += graph[v][v];
        self.cross_counts[g0][g1] += graph[u][v];
    }

    fn add_relative_counts(
        &mut self,
        graph: &BinaryGraph,
        g0: usize,
        g1: usize,
        i0: usize,
        i1: usize,
    ) {
        assert!(g0 < g1);

        for &(g0, i0) in [(g0, i0), (g1, i1)].iter() {
            let u = self.groups[g0][i0];
            let edges = &graph[u];

            for (g1, group) in self.groups.iter().enumerate() {
                if g0 == g1 {
                    // self
                    let counts = &mut self.self_counts[g0];

                    for &v in group.iter() {
                        *counts += edges[v];
                    }
                } else {
                    // cross
                    let (g0, g1) = if g0 < g1 { (g0, g1) } else { (g1, g0) };
                    let counts = &mut self.cross_counts[g0][g1];

                    for &v in group.iter() {
                        *counts += edges[v];
                    }
                }
            }
        }

        // 足しすぎた分を引く
        let u = self.groups[g0][i0];
        let v = self.groups[g1][i1];
        self.self_counts[g0] -= graph[u][u];
        self.self_counts[g1] -= graph[v][v];
        self.cross_counts[g0][g1] -= graph[u][v];
    }

    pub fn update_score_all(&mut self, graph: &BinaryGraph) {
        for c in self.self_counts.iter_mut() {
            *c = 0;
        }

        for c in self.cross_counts.iter_mut().flatten() {
            *c = 0;
        }

        // グループ内のcountを計算
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

        // グループ間のcountを計算
        for g0 in 0..self.group_count {
            let group0 = &self.groups[g0];
            for g1 in (g0 + 1)..self.group_count {
                let group1 = &self.groups[g1];
                let count = &mut self.cross_counts[g0][g1];

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
        self.score = 0;

        // グループ内のスコアを計算 (Σi 2max(ci, 0))
        for &c in self.self_counts.iter() {
            const COEFFICIENT: i32 = 2;
            self.score += c.max(0) * COEFFICIENT;
        }

        // グループ間のスコアを計算 (Σij |cij|)
        for c0 in 0..self.group_count {
            let counts = &self.cross_counts[c0];

            for c1 in (c0 + 1)..self.group_count {
                self.score += counts[c1].abs();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;
    use rand_pcg::Pcg64Mcg;

    use crate::{encoders::isomophism::annealing::binarygraph::BinaryGraph, graph::Graph};

    use super::State;

    #[test]
    fn score_test() {
        let graph = gen_graph();
        let groups = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let state = State::new(&graph, groups);

        let expected = state.score;
        let actual = 2 * 2 * 3 + 9;

        assert_eq!(expected, actual);
    }

    #[test]
    fn swap_test() {
        let graph = gen_graph();
        let groups = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let mut state = State::new(&graph, groups);

        // 頂点0と3をswap
        state.swap_nodes(&graph, 0, 1, 0, 0);
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
        let mut state = State::init_rand(&graph, GROUP_COUNT, &mut rng);

        for _ in 0..TRIAL_COUNT {
            let g0 = rng.gen_range(0, state.group_count);
            let g1 = (g0 + rng.gen_range(1, state.group_count)) % state.group_count;
            let i0 = rng.gen_range(0, N / GROUP_COUNT);
            let i1 = rng.gen_range(0, N / GROUP_COUNT);

            state.swap_nodes(&graph, g0, g1, i0, i1);
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
