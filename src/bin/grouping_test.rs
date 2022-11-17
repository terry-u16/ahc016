use itertools::Itertools;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand_pcg::Pcg64Mcg;
use std::ops::Index;

const WIDTH: usize = 16;
const BITS: usize = 6;
const N: usize = WIDTH * BITS;
const ERROR_RATIO: f64 = 0.35;

type State = Vec<Vec<usize>>;

fn main() {
    let graph = gen_graph();
    let mut state = grouping(&graph);

    state.sort_by_key(|g| g.iter().sum::<usize>());

    let mut new_graph = Graph::new(N);

    for i in 0..N {
        for j in (i + 1)..N {
            let u = state[i / WIDTH][i % WIDTH];
            let v = state[j / WIDTH][j % WIDTH];

            if graph[u][v] {
                new_graph.connect(i, j);
            }
        }
    }
    eprintln!("{}", new_graph);

    for group in state.iter_mut() {
        group.sort_unstable();
        println!("{:?}", &group);
    }
}

fn gen_graph() -> Graph {
    let mut graph = Graph::new(N);

    for i in 0..N {
        for j in (i + 1)..N {
            if i / WIDTH == j / WIDTH {
                graph.connect(i, j);
            }
        }
    }

    // 1と2の間をつなぐ
    for i in 0..N {
        for j in (i + 1)..N {
            if i / WIDTH == 0 && j / WIDTH == 1 {
                graph.connect(i, j);
            }
        }
    }

    println!("{}", graph);

    // ノイズ付与
    let mut rng = Pcg64Mcg::new(42);
    for i in 0..N {
        for j in (i + 1)..N {
            if rng.gen_bool(ERROR_RATIO) {
                graph.flip(i, j);
            }
        }
    }

    println!();
    println!("{}", graph);

    graph
}

fn grouping(graph: &Graph) -> State {
    let state = get_init_state();
    let state = annealing(graph, state, 1.00);
    state
}

fn get_init_state() -> State {
    let mut rng = Pcg64Mcg::new(42);
    let mut perm = (0..N).collect_vec();
    perm.shuffle(&mut rng);

    let mut groups = vec![vec![]; BITS];

    for (i, &p) in perm.iter().enumerate() {
        groups[i % BITS].push(p);
    }

    groups
}

fn calc_score(graph: &Graph, state: &State) -> i32 {
    let mut score = 0;

    for group in state.iter() {
        let mut sum = 0;

        for i in 0..group.len() {
            let u = group[i];

            for j in (i + 1)..group.len() {
                let v = group[j];

                if graph[u][v] {
                    sum += 1;
                }
            }
        }

        // そのままだと↓の半分未満になってしまうので2倍する
        sum *= 2;
        score += sum;
    }

    for g1 in 0..state.len() {
        for g2 in (g1 + 1)..state.len() {
            let mut sum: i32 = 0;

            for &u in state[g1].iter() {
                for &v in state[g2].iter() {
                    if graph[u][v] {
                        sum += 1;
                    } else {
                        sum -= 1;
                    }
                }
            }

            score += sum.abs();
        }
    }

    score
}

fn annealing(graph: &Graph, initial_solution: State, duration: f64) -> State {
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

    let temp0 = 5e1;
    let temp1 = 5e-1;
    let mut inv_temp = 1.0 / temp0;

    while time < 1.0 {
        all_iter += 1;
        if (all_iter & ((1 << 4) - 1)) == 0 {
            time = (std::time::Instant::now() - since).as_secs_f64() * duration_inv;
            let temp = f64::powf(temp0, 1.0 - time) * f64::powf(temp1, time);
            inv_temp = 1.0 / temp;
        }

        // 変形
        let g0 = rng.gen_range(0, solution.len());
        let v0 = rng.gen_range(0, solution[g0].len());
        let g1 = (g0 + rng.gen_range(1, solution.len())) % solution.len();
        let v1 = rng.gen_range(0, solution[g1].len());

        let temp = solution[g0][v0];
        solution[g0][v0] = solution[g1][v1];
        solution[g1][v1] = temp;

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
            let temp = solution[g0][v0];
            solution[g0][v0] = solution[g1][v1];
            solution[g1][v1] = temp;
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

    pub fn flip(&mut self, u: usize, v: usize) {
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
