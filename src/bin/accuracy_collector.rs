#![allow(non_snake_case)]
use clap::Parser;
use rand::prelude::*;
use std::io::{prelude::*, BufReader};
use std::process::{ChildStdout, Stdio};

const TRIAL_COUNT: usize = 200;

#[derive(Parser, Debug)]
struct AppArg {
    #[clap(short = 'b', long = "bits")]
    bits: usize,
    #[clap(short = 'e', long = "eps")]
    eps: f64,
    #[clap(short = 'm', long = "graph-count")]
    m: usize,
    #[clap(short = 'r', long = "redundancy")]
    redundancy: usize,
    #[clap(short = 's', long = "score_coef")]
    score_coef: f64,
    #[clap(short = 'c', long = "command")]
    command: String,
}

fn read(stdout: &mut BufReader<ChildStdout>) -> Result<String, String> {
    loop {
        let mut out = String::new();
        match stdout.read_line(&mut out) {
            Ok(0) | Err(_) => {
                return Err(format!("Your program has terminated unexpectedly"));
            }
            _ => (),
        }
        // print!("{}", out);
        let v = out.trim();
        if v.len() == 0 || v.starts_with("#") {
            continue;
        }
        return Ok(v.to_owned());
    }
}

fn read_usize(stdout: &mut BufReader<ChildStdout>, lb: usize, ub: usize) -> Result<usize, String> {
    let v = read(stdout)?;
    let v = v
        .parse::<usize>()
        .map_err(|_| format!("Illegal output: {}", v))?;
    if v < lb || ub < v {
        return Err(format!("Illegal output: {}", v));
    }
    Ok(v)
}

fn exec(eps: f64, M: usize, p: &mut std::process::Child) -> Result<(), String> {
    let mut stdin = std::io::BufWriter::new(p.stdin.take().unwrap());
    let mut stdout = std::io::BufReader::new(p.stdout.take().unwrap());
    let _ = writeln!(stdin, "{} {:.2}", M, eps);
    let _ = stdin.flush();

    let N = read_usize(&mut stdout, 4, 100)?;
    let mut gs = vec![];

    for k in 0..M {
        let g = read(&mut stdout)?;
        let cs = g.chars().collect::<Vec<_>>();

        if cs.len() != N * (N - 1) / 2 || cs.iter().any(|&c| c != '0' && c != '1') {
            return Err(format!("Illegal output (g_{}): {}", k, g));
        }

        let mut g = mat![false; N; N];
        let mut p = 0;
        for i in 0..N {
            for j in i + 1..N {
                g[i][j] = cs[p] == '1';
                g[j][i] = g[i][j];
                p += 1;
            }
        }

        gs.push(g);
    }

    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(42);
    let mut accepted = 0;

    for _ in 0..TRIAL_COUNT {
        let mut vs = (0..N).collect::<Vec<_>>();
        vs.shuffle(&mut rng);
        let s = rng.gen_range(0, M);

        let mut h = String::new();
        for i in 0..N {
            for j in i + 1..N {
                if gs[s][vs[i]][vs[j]] ^ rng.gen_bool(eps) {
                    h.push('1');
                } else {
                    h.push('0');
                }
            }
        }

        let _ = writeln!(stdin, "{}", h);
        let _ = stdin.flush();
        let t = read_usize(&mut stdout, 0, M - 1)?;

        if s == t {
            accepted += 1;
        }
    }

    println!("{}", TRIAL_COUNT);
    println!("{}", accepted);

    p.wait().unwrap();
    Ok(())
}

fn main() {
    let args = AppArg::parse();
    let query_count = TRIAL_COUNT;
    let child_args = vec![
        query_count.to_string(),
        args.bits.to_string(),
        args.redundancy.to_string(),
        args.score_coef.to_string(),
    ];

    let mut p = std::process::Command::new(args.command)
        .args(child_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("failed to execute the command");
            eprintln!("{}", e);
            std::process::exit(1)
        });

    match exec(args.eps, args.m, &mut p) {
        Ok(_) => (),
        Err(err) => {
            let _ = p.kill();
            println!("{}", err);
        }
    }
}

#[allow(dead_code)]
mod lib {
    #![allow(non_snake_case, unused_macros)]

    use proconio::input;
    use rand::prelude::*;

    pub trait SetMinMax {
        fn setmin(&mut self, v: Self) -> bool;
        fn setmax(&mut self, v: Self) -> bool;
    }
    impl<T> SetMinMax for T
    where
        T: PartialOrd,
    {
        fn setmin(&mut self, v: T) -> bool {
            *self > v && {
                *self = v;
                true
            }
        }
        fn setmax(&mut self, v: T) -> bool {
            *self < v && {
                *self = v;
                true
            }
        }
    }

    #[macro_export]
    macro_rules! mat {
	($($e:expr),*) => { Vec::from(vec![$($e),*]) };
	($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
	($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
	($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
}

    pub const Q: usize = 100;
    pub const MAX_N: usize = 100;

    #[derive(Clone, Debug)]
    pub struct Input {
        pub M: usize,
        pub eps: f64,
        pub ss: Vec<usize>,
        pub seed: u64,
    }

    impl std::fmt::Display for Input {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "{} {:.2}", self.M, self.eps)?;
            for i in 0..Q {
                writeln!(f, "{}", self.ss[i])?;
            }
            writeln!(f, "{}", self.seed)?;
            Ok(())
        }
    }

    pub fn parse_input(f: &str) -> Input {
        let f = proconio::source::once::OnceSource::from(f);
        input! {
            from f,
            M: usize, eps: f64,
            ss: [usize; Q],
            seed: u64,
        }
        Input { M, eps, seed, ss }
    }

    pub struct Output {
        pub N: usize,
        pub gs: Vec<String>,
        pub ts: Vec<usize>,
        pub comments: Vec<String>,
        pub comments_g: Vec<String>,
    }

    pub fn parse_output(input: &Input, f: &str) -> Result<Output, String> {
        let tokens = f.lines();
        let mut comment = String::new();
        let mut N = 0;
        let mut gs = vec![];
        let mut ts = vec![];
        let mut comments_g = vec![];
        let mut comments = vec![];
        for v in tokens {
            let v = v.trim();
            if v.len() == 0 {
                continue;
            } else if v.starts_with("#") {
                comment += v;
                comment.push('\n');
            } else if N == 0 {
                N = v
                    .parse::<usize>()
                    .map_err(|_| format!("Illegal output (N): {}", v))?;
                if N < 4 || MAX_N < N {
                    return Err(format!("Illegal output (N): {}", v));
                }
            } else if gs.len() < input.M {
                let cs = v.chars().collect::<Vec<_>>();
                if cs.len() != N * (N - 1) / 2 || cs.iter().any(|&c| c != '0' && c != '1') {
                    return Err(format!("Illegal output (g_{}): {}", gs.len(), v));
                }
                gs.push(v.to_owned());
                comments_g.push(comment);
                comment = String::new();
            } else {
                let v = v
                    .parse::<usize>()
                    .map_err(|_| format!("Illegal output (t_{}): {}", ts.len(), v))?;
                if v < input.M {
                    ts.push(v);
                    comments.push(comment);
                    comment = String::new();
                } else {
                    return Err(format!("Illegal output (t_{}): {}", ts.len(), v));
                }
            }
        }
        Ok(Output {
            N,
            gs,
            ts,
            comments_g,
            comments,
        })
    }

    pub fn compute_score(input: &Input, out: &Output) -> i64 {
        let mut E = 0;
        for i in 0..Q {
            if input.ss[i] != out.ts[i] {
                E += 1;
            }
        }
        score(E, out.N)
    }

    pub fn score(E: i32, N: usize) -> i64 {
        (1e9 * f64::powi(0.9, E) / N as f64).round() as i64
    }

    pub fn gen(seed: u64, custom_M: Option<usize>, custom_eps: Option<f64>) -> Input {
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed ^ 275473);
        let mut M = rng.gen_range(10, 101i32) as usize;
        if let Some(M2) = custom_M {
            M = M2;
        }
        let mut eps = rng.gen_range(0, 41i32) as f64 * 0.01;
        if let Some(eps2) = custom_eps {
            eps = eps2;
        }
        let mut ss = vec![];
        for _ in 0..Q {
            ss.push(rng.gen_range(0, M as i32) as usize);
        }
        let seed = rng.gen::<u64>();
        Input { M, eps, ss, seed }
    }
}
