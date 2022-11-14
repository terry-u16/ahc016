use super::binarygraph::BinaryGraph;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone)]
pub struct State {
    permutation: Vec<usize>,
    score: i32,
}

impl State {
    pub fn init_rand(graph: &BinaryGraph, rng: &mut Pcg64Mcg) -> Self {
        let mut permutation = (0..graph.n).collect_vec();
        permutation.shuffle(rng);
        let mut state = Self {
            permutation,
            score: 0,
        };

        state.update_score_all(graph);
        state
    }

    pub fn permutation(&self) -> &[usize] {
        &self.permutation
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn swap_node(&mut self, graph: &BinaryGraph, u: usize, v: usize) {
        self.permutation.swap(u, v);
        self.update_score_all(graph);
    }

    pub fn swap_node_with(&mut self, _graph: &BinaryGraph, u: usize, v: usize, score: i32) {
        self.permutation.swap(u, v);
        self.score = score;
    }

    fn update_score_all(&mut self, graph: &BinaryGraph) {
        let mut score = 0;

        for row in 0..graph.n {
            let mut sum = 0;
            let i = self.permutation[row];
            let edges = &graph[i];

            for &j in self.permutation[(row + 1)..].iter() {
                sum += edges[j];
            }

            score += sum * sum;
        }

        self.score = score;
    }
}
