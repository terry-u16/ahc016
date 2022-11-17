use super::{binarygraph::BinaryGraph, neighbors::NeighborGenerator, state::State};
use crate::utils::ChangeMinMax;
use rand::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Annealer {
    verbose: bool,
}

impl Annealer {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    fn annealing(&self, graph: &BinaryGraph, initial_solution: State, duration: f64) -> State {
        let mut solution = initial_solution;
        let mut best_solution = solution.clone();
        let mut current_score = solution.score();
        let init_score = current_score;
        let mut best_score = current_score;

        let mut all_iter = 0;
        let mut valid_iter = 0;
        let mut accepted_count = 0;
        let mut update_count = 0;
        let mut rng = rand_pcg::Pcg64Mcg::new(42);

        let duration_inv = 1.0 / duration;
        let since = std::time::Instant::now();

        let temp0 = 5e1;
        let temp1 = 5e-1;
        let mut inv_temp = 1.0 / temp0;
        let generator = NeighborGenerator;

        loop {
            all_iter += 1;
            if (all_iter & ((1 << 4) - 1)) == 0 {
                let time = (std::time::Instant::now() - since).as_secs_f64() * duration_inv;
                if time >= 1.0 {
                    break;
                }

                let temp = f64::powf(temp0, 1.0 - time) * f64::powf(temp1, time);
                inv_temp = 1.0 / temp;
            }

            // 変形
            let neighbor = generator.gen(&graph, &solution, &mut rng);
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
            eprintln!("init score : {}", init_score);
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
