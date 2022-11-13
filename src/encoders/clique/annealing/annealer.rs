use super::state::State;
use crate::{graph::Graph, utils::ChangeMinMax};
use rand::prelude::*;

pub struct Annealer;

impl Annealer {
    pub fn run(&self, graph: &Graph) -> usize {
        let state = State::init(graph);

        todo!();
    }

    pub fn anneal(&self, graph: &Graph, initial_solution: State, duration: f64) -> State {
        let mut solution = initial_solution;
        let mut best_solution = solution.clone();
        let mut current_score = 0;
        let mut best_score = current_score;

        let mut all_iter = 0;
        let mut valid_iter = 0;
        let mut accepted_count = 0;
        let mut update_count = 0;
        let mut rng = rand_pcg::Pcg64Mcg::new(42);

        let duration_inv = 1.0 / duration;
        let since = std::time::Instant::now();
        let mut time = 0.0;

        let temp0 = graph.n as f64;
        let temp1 = 1e0;
        let mut inv_temp = 1.0 / temp0;

        while time < 1.0 {
            all_iter += 1;
            if (all_iter & ((1 << 4) - 1)) == 0 {
                time = (std::time::Instant::now() - since).as_secs_f64() * duration_inv;
                let temp = f64::powf(temp0, 1.0 - time) * f64::powf(temp1, time);
                inv_temp = 1.0 / temp;
            }

            // 変形

            // スコア計算
            let new_score = 0;
            let score_diff = new_score - current_score;

            if score_diff >= 0 || rng.gen_bool(f64::exp(score_diff as f64 * inv_temp)) {
                // 解の更新
                current_score = new_score;
                accepted_count += 1;

                if best_score.change_max(current_score) {
                    best_solution = solution.clone();
                    update_count += 1;
                }
            }

            valid_iter += 1;
        }

        eprintln!("===== annealing =====");
        eprintln!("score      : {}", best_score);
        eprintln!("all iter   : {}", all_iter);
        eprintln!("valid iter : {}", valid_iter);
        eprintln!("accepted   : {}", accepted_count);
        eprintln!("updated    : {}", update_count);
        eprintln!("");

        best_solution
    }
}
