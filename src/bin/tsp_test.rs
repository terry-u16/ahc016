use itertools::Itertools;
use rand::{seq::SliceRandom, Rng};
use rand_pcg::Pcg64Mcg;

const N: usize = 20;
const W: i32 = 5;
const ERROR_RATE: f64 = 0.00;
const SEED: u128 = 42;

fn main() {
    let graph = gen_graph();
    print(&graph);
    let graph = add_noise(graph);
    print(&graph);
    let (graph, perm) = gen_permutation(&graph);
    print(&graph);
    let ans_perm = tsp(&graph);
    println!("pred  : {:?}", ans_perm);
    println!("actual: {:?}", perm);
}

fn gen_graph() -> Vec<Vec<bool>> {
    let mut graph = vec![vec![false; N]; N];

    for i in 0..N {
        for d in -W..=W {
            let j = ((i + N) as i32 + d) as usize % N;
            graph[i][j] = true;
        }
    }

    graph
}

fn gen_permutation(graph: &Vec<Vec<bool>>) -> (Vec<Vec<bool>>, Vec<usize>) {
    let mut rng = Pcg64Mcg::new(SEED);
    let mut permutation = (0..N).collect_vec();

    // 頂点0, 1は固定
    permutation[2..].shuffle(&mut rng);

    let mut new_graph = vec![vec![false; N]; N];

    for i in 0..N {
        for j in 0..N {
            new_graph[permutation[i]][permutation[j]] = graph[i][j];
        }
    }

    (new_graph, permutation)
}

fn add_noise(mut graph: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    let mut rng = Pcg64Mcg::new(SEED);

    for i in 0..N {
        for j in (i + 1)..N {
            let inv = rng.gen_bool(ERROR_RATE);
            graph[i][j] ^= inv;
            graph[j][i] ^= inv;
        }
    }

    graph
}

fn print(graph: &Vec<Vec<bool>>) {
    for v in graph.iter() {
        for v in v {
            let b = if *v { '1' } else { '0' };
            print!("{}", b);
        }

        println!();
    }

    println!();
}

fn tsp(graph: &Vec<Vec<bool>>) -> Vec<usize> {
    let init_solution = (0..N).collect_vec();
    annealing(graph, init_solution, 0.05)
}

fn annealing(graph: &Vec<Vec<bool>>, initial_solution: Vec<usize>, duration: f64) -> Vec<usize> {
    let bits = gen_bits(graph);
    let mut solution = initial_solution;
    let mut best_solution = solution.clone();
    let mut current_score = calc_score(&solution, &bits);
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

    let temp0 = 1e1;
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
        let mut left = rng.gen_range(2, N + 1);
        let mut right = rng.gen_range(2, N + 1);

        if left == right {
            continue;
        } else if left > right {
            std::mem::swap(&mut left, &mut right);
        }

        solution[left..right].reverse();

        // スコア計算
        let new_score = calc_score(&solution, &bits);
        let score_diff = new_score - current_score;

        if score_diff >= 0 || rng.gen_bool(f64::exp(score_diff as f64 * inv_temp)) {
            // 解の更新
            current_score = new_score;
            accepted_count += 1;

            if best_score.set_max(current_score) {
                best_solution = solution.clone();
                update_count += 1;
            } else if best_score == current_score {
                eprintln!("        {:?}", &solution);
            }
        } else {
            solution[left..right].reverse();
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

fn gen_bits(graph: &Vec<Vec<bool>>) -> Vec<u64> {
    let mut bits = vec![];

    for i in 0..N {
        let mut v = 0;

        for j in 0..N {
            if graph[i][j] {
                v |= 1 << j;
            }
        }

        bits.push(v);
    }

    bits
}

fn calc_score(solution: &[usize], bits: &[u64]) -> i32 {
    let mut score = 0;

    let mut bit = 0;

    for d in 1..=W {
        let v = solution[N - d as usize];
        bit |= 1 << v;
    }

    for (i, &v) in solution.iter().enumerate() {
        score += (bit & bits[v]).count_ones();
        bit ^= 1 << solution[(N + i - W as usize) % N];
        bit ^= 1 << solution[i];
    }

    score as i32
}

pub trait SetMinMax {
    fn set_min(&mut self, v: Self) -> bool;
    fn set_max(&mut self, v: Self) -> bool;
}

impl<T> SetMinMax for T
where
    T: PartialOrd,
{
    fn set_min(&mut self, v: T) -> bool {
        *self > v && {
            *self = v;
            true
        }
    }
    fn set_max(&mut self, v: T) -> bool {
        *self < v && {
            *self = v;
            true
        }
    }
}
