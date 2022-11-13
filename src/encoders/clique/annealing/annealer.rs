use super::state::State;
use crate::{
    encoders::clique::annealing::neighbors::NeighborGenerator, graph::Graph, utils::ChangeMinMax,
};
use itertools::Itertools;
use rand::prelude::*;
use std::cmp::Reverse;

#[derive(Debug, Clone, Copy)]
pub struct Annealer {
    verbose: bool,
}

impl Annealer {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn run(&self, graph: &Graph, duration: f64) -> Vec<usize> {
        // TODO: 初期解を貪欲で作る
        let state = State::init(graph);
        let state = Self::anneal(&self, graph, state, duration);

        // グループの大きさを集計
        let mut sizes = state
            .get_group_size_list()
            .iter()
            .copied()
            .filter(|s| *s > 0)
            .collect_vec();

        sizes.sort_by_key(|s| Reverse(*s));
        sizes
    }

    fn anneal(&self, graph: &Graph, initial_solution: State, duration: f64) -> State {
        let mut solution = initial_solution;
        let mut best_solution = solution.clone();
        let mut current_score = solution.score();
        let mut best_score = current_score;

        let mut all_iter = 0;
        let mut valid_iter = 0;
        let mut accepted_count = 0;
        let mut update_count = 0;
        let mut rng = rand_pcg::Pcg64Mcg::new(42);

        let duration_inv = 1.0 / duration;
        let since = std::time::Instant::now();

        let temp0 = graph.n as f64;
        let temp1 = 1e-1;
        let mut inv_temp = 1.0 / temp0;
        let neighbor_generator = NeighborGenerator;

        loop {
            all_iter += 1;
            if (all_iter & ((1 << 4) - 1)) == 0 {
                let time = (std::time::Instant::now() - since).as_secs_f64() * duration_inv;
                let temp = f64::powf(temp0, 1.0 - time) * f64::powf(temp1, time);
                inv_temp = 1.0 / temp;

                if time >= 1.0 {
                    break;
                }
            }

            // 変形
            let neighbor = neighbor_generator.gen(graph, &solution, &mut rng);
            neighbor.apply(graph, &mut solution);

            // スコア計算
            let new_score = solution.score();
            let score_diff = new_score - current_score;

            if score_diff >= 0 || rng.gen_bool(f64::exp(score_diff as f64 * inv_temp)) {
                // 解の更新
                current_score = new_score;
                accepted_count += 1;

                if best_score.change_max(current_score) {
                    best_solution = solution.clone();
                    update_count += 1;
                }
            } else {
                neighbor.rollback(graph, &mut solution);
            }

            valid_iter += 1;
        }

        if self.verbose {
            eprintln!("===== annealing =====");
            eprintln!("score      : {}", best_score);
            eprintln!("all iter   : {}", all_iter);
            eprintln!("valid iter : {}", valid_iter);
            eprintln!("accepted   : {}", accepted_count);
            eprintln!("updated    : {}", update_count);
            eprintln!("");
        }

        best_solution
    }
}
