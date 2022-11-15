use rand::prelude::*;

const M: usize = 14;
const BITS: usize = 7;
const N: usize = M * BITS;
const NOISE_PROB: f64 = 0.3;

fn main() {
    let mut graph = Graph::new(N);

    for i in 0..N {
        if (i / M) % 2 != 0 {
            continue;
        }

        for j in (i + 1)..N {
            graph.connect(i, j);
        }
    }

    println!("{}", &graph);

    // ノイズ付与
    let mut rng = Pcg64Mcg::new(42);
    for i in 0..N {
        for j in (i + 1)..N {
            if rng.gen_bool(NOISE_PROB) {
                graph.filp(i, j);
            }
        }
    }

    println!("--------------------------------------------");
    println!("{}", &graph);

    // 焼きなまし
    // 複数回やってアンサンブルする
    const TRIAL_COUNT: usize = 100;
    let mut summary = vec![vec![0; N]; N];

    for _ in 0..TRIAL_COUNT {
        let mut state = (0..N).collect_vec();
        state.shuffle(&mut rng);
        println!("{:?}", &state);

        let state = annealing(&graph, state, 0.1);

        println!("{:?}", &state);

        for i in 0..N {
            for j in (i + 1)..N {
                if graph[state[i]][state[j]] {
                    summary[i][j] += 1;
                } else {
                    summary[i][j] -= 1;
                }
            }
        }
    }

    // 集計 + フィルタ処理
    let mut new_graph = Graph::new(N);

    for row in 0..N {
        for col in 0..N {
            let mut cnt = 0;

            const FILTER_SIZE: i32 = 1;
            for dr in -FILTER_SIZE..=FILTER_SIZE {
                for dc in -FILTER_SIZE..=FILTER_SIZE {
                    let r = row.wrapping_add(dr as usize);
                    let c = col.wrapping_add(dc as usize);

                    if r >= N || c >= N {
                        continue;
                    }

                    cnt += summary[r][c];
                }
            }

            if cnt > 0 {
                new_graph.connect(row, col);
            }
        }
    }

    println!("{}", &new_graph);

    // bitに変換
    let mut bits = vec![];

    for k in 0..BITS {
        let mut s = 0;

        for i in 0..M {
            let i = k * M + i;

            for j in (i + 1)..N {
                if new_graph[i][j] {
                    s += 1;
                } else {
                    s -= 1;
                }
            }
        }

        let bit = if s > 0 { 1 } else { 0 };
        bits.push(bit);
        print!("{} ", s);
    }

    println!();
    println!("{:?}", bits);
}

fn annealing(graph: &Graph, initial_solution: Vec<usize>, duration: f64) -> Vec<usize> {
    let mut solution = initial_solution;
    let mut best_solution = solution.clone();
    let mut current_score = calc_score(graph, &solution);
    let init_score = current_score;
    let mut best_score = current_score;

    let mut all_iter = 0;
    let mut valid_iter = 0;
    let mut accepted_count = 0;
    let mut update_count = 0;
    let mut rng = rand_pcg::Pcg64Mcg::new(42);

    let duration_inv = 1.0 / duration;
    let since = std::time::Instant::now();
    let mut time = 0.0;

    let temp0 = 1e4;
    let temp1 = 1e1;
    let mut inv_temp = 1.0 / temp0;

    while time < 1.0 {
        all_iter += 1;
        if (all_iter & ((1 << 4) - 1)) == 0 {
            time = (std::time::Instant::now() - since).as_secs_f64() * duration_inv;
            let temp = f64::powf(temp0, 1.0 - time) * f64::powf(temp1, time);
            inv_temp = 1.0 / temp;
        }

        // 変形
        let i = rng.gen_range(0, N);
        let j = (i + rng.gen_range(1, N)) % N;
        solution.swap(i, j);

        // スコア計算
        let new_score = calc_score(graph, &solution);
        let score_diff = new_score - current_score;

        if score_diff >= 0 || rng.gen_bool(f64::exp(score_diff as f64 * inv_temp)) {
            // 解の更新
            current_score = new_score;
            accepted_count += 1;

            if best_score < current_score {
                best_score = current_score;
                best_solution = solution.clone();
                update_count += 1;
            }
        } else {
            solution.swap(i, j);
        }

        valid_iter += 1;
    }

    eprintln!("===== annealing =====");
    eprintln!("init score : {}", init_score);
    eprintln!("score      : {}", best_score);
    eprintln!("all iter   : {}", all_iter);
    eprintln!("valid iter : {}", valid_iter);
    eprintln!("accepted   : {}", accepted_count);
    eprintln!("updated    : {}", update_count);
    eprintln!("");

    best_solution
}

fn calc_score(graph: &Graph, state: &[usize]) -> i64 {
    let mut score = 0;

    for row in 0..N {
        let mut s = 0;
        let i = state[row];

        for col in (row + 1)..N {
            let j = state[col];

            if graph[i][j] {
                s += 1;
            } else {
                s -= 1;
            }
        }

        score += s * s;
    }

    score
}

use std::ops::Index;

use itertools::Itertools;
use rand::seq::SliceRandom;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone)]
pub struct Graph {
    pub n: usize,
    edges: Vec<Vec<bool>>,
}

impl Graph {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            edges: vec![vec![false; n]; n],
        }
    }

    pub fn connect(&mut self, u: usize, v: usize) {
        self.edges[u][v] = true;
        self.edges[v][u] = true;
    }

    pub fn filp(&mut self, u: usize, v: usize) {
        self.edges[u][v] ^= true;
        self.edges[v][u] ^= true;
    }

    pub fn deserialize(str: &str, n: usize) -> Self {
        let mut edges = vec![vec![false; n]; n];
        let mut chars = str.chars();

        for row in 0..n {
            for col in (row + 1)..n {
                if chars.next().unwrap() == '1' {
                    edges[row][col] = true;
                    edges[col][row] = true;
                }
            }
        }

        Self { n, edges }
    }

    pub fn serialize(&self) -> String {
        let mut s = vec![];

        for row in 0..self.n {
            for col in (row + 1)..self.n {
                let c = if self.edges[row][col] { '1' } else { '0' };
                s.push(c);
            }
        }

        s.iter().collect()
    }
}

impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.n {
            for col in 0..self.n {
                let c = if self.edges[row][col] { '#' } else { '.' };
                write!(f, "{}", c)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl Index<usize> for Graph {
    type Output = [bool];

    fn index(&self, index: usize) -> &Self::Output {
        &self.edges[index]
    }
}
