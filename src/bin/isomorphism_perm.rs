use std::ops::Index;

use itertools::Itertools;

const N: usize = 6;
const EDGE_COUNTS: usize = N * (N - 1) / 2;

fn main() {
    let mut graphs = vec![];

    for bits in 0..(1 << EDGE_COUNTS) {
        let graph = gen_graph(bits);
        let mut found = false;

        for (_, g) in graphs.iter() {
            found |= are_isomorphic(&graph, g);
        }

        if !found {
            graphs.push((bits, graph));
        }
    }

    println!("counts: {}", graphs.len());

    for (bits, _) in graphs.iter() {
        println!("{:0>1$b}", bits, EDGE_COUNTS);
    }
}

fn gen_graph(bits: usize) -> Graph {
    let mut graph = Graph::new(N);
    let mut index = 0;

    for i in 0..N {
        for j in (i + 1)..N {
            if ((bits >> index) & 1) > 0 {
                graph.connect(i, j);
            }

            index += 1;
        }
    }

    graph
}

fn are_isomorphic(g1: &Graph, g2: &Graph) -> bool {
    let deg1 = get_degs(g1);
    let deg2 = get_degs(g2);

    if deg1 != deg2 {
        return false;
    }

    'main: for p in (0..N).permutations(N) {
        for i in 0..N {
            for j in (i + 1)..N {
                if g1[i][j] != g2[p[i]][p[j]] {
                    continue 'main;
                }
            }
        }

        return true;
    }

    false
}

fn get_degs(graph: &Graph) -> Vec<u32> {
    let mut degs = vec![0; N * (N - 1) / 2];

    for i in 0..N {
        for j in (i + 1)..N {
            if graph[i][j] {
                degs[i] += 1;
                degs[j] += 1;
            }
        }
    }

    degs.sort_unstable();

    degs
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
