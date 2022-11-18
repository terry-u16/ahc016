use std::{ops::Index, time::Instant};

use itertools::Itertools;

const N: usize = 6;
const EDGE_COUNTS: usize = N * (N - 1) / 2;

fn main() {
    let since = Instant::now();
    let mut graphs = vec![];

    for bits in 0..(1 << EDGE_COUNTS) {
        let graph = gen_graph(bits);
        let checker = Vf2Checker::new(&graph);
        let found = graphs.iter().any(|(_, g)| checker.is_isomorphic(g));

        if !found {
            graphs.push((bits, graph));
        }
    }

    let until = Instant::now();

    println!("counts: {}", graphs.len());

    for (bits, _) in graphs.iter() {
        println!("{:0>1$b}", bits, EDGE_COUNTS);
    }

    println!("{}s", (until - since).as_secs_f64());
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

/// グラフの同型性を判定するトレイト
trait IsomophicChecker {
    /// 2つのグラフが同型かどうか判定する
    fn is_isomorphic(&self, graph: &Graph) -> bool;
}

#[derive(Debug, Clone)]
struct PermutationalChecker {
    n: usize,
    degs: Vec<u32>,
    graph: Graph,
}

#[allow(dead_code)]
impl PermutationalChecker {
    fn new(graph: &Graph) -> Self {
        let n = graph.n;
        let degs = Self::get_degs(&graph);

        Self {
            n,
            degs,
            graph: graph.clone(),
        }
    }

    fn get_degs(graph: &Graph) -> Vec<u32> {
        let mut degs = vec![0; N];

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
}

impl IsomophicChecker for PermutationalChecker {
    fn is_isomorphic(&self, graph: &Graph) -> bool {
        let deg2 = Self::get_degs(graph);

        if self.degs != deg2 {
            return false;
        }

        let n = self.n;

        'main: for p in (0..n).permutations(n) {
            for i in 0..n {
                for j in (i + 1)..n {
                    if self.graph[i][j] != graph[p[i]][p[j]] {
                        continue 'main;
                    }
                }
            }

            return true;
        }

        false
    }
}

/// VF2アルゴリズムによりグラフの同型性判定を行う構造体
/// 参考: http://satemochi.blog.fc2.com/blog-entry-224.html
#[derive(Debug, Clone)]
struct Vf2Checker {
    n: usize,
    degs: Vec<u32>,
    graph: AdjacencyListGraph,
}

impl IsomophicChecker for Vf2Checker {
    fn is_isomorphic(&self, graph: &Graph) -> bool {
        if self.n != graph.n {
            return false;
        }

        let degs = Self::get_degs(&graph);

        if self.degs != degs {
            return false;
        }

        let graph = AdjacencyListGraph::from(graph);
        let mut map12 = vec![None; self.n];
        let mut map21 = vec![None; self.n];
        let mut neighs1 = vec![false; self.n];
        let mut neighs2 = vec![false; self.n];

        Self::isomophism_dfs(
            &self.graph,
            &graph,
            &mut map12,
            &mut map21,
            &mut neighs1,
            &mut neighs2,
            0,
        )
    }
}

impl Vf2Checker {
    fn new(graph: &Graph) -> Self {
        let n = graph.n;
        let degs = Self::get_degs(&graph);
        let graph = AdjacencyListGraph::from(graph);

        Self { n, degs, graph }
    }

    fn get_degs(graph: &Graph) -> Vec<u32> {
        let n = graph.n;
        let mut degs = vec![0; n];

        for i in 0..n {
            for j in (i + 1)..n {
                if graph[i][j] {
                    degs[i] += 1;
                    degs[j] += 1;
                }
            }
        }

        degs.sort_unstable();

        degs
    }

    fn isomophism_dfs(
        graph1: &AdjacencyListGraph,
        graph2: &AdjacencyListGraph,
        map12: &mut [Option<usize>],
        map21: &mut [Option<usize>],
        neighs1: &mut [bool],
        neighs2: &mut [bool],
        depth: usize,
    ) -> bool {
        let (vs1, v2) = Self::generate_candidates(graph1.n, map12, map21, neighs1, neighs2);
        let mut stack1 = vec![];
        let mut stack2 = vec![];

        for &v1 in vs1.iter() {
            let edges1 = &graph1.edges[v1];
            let edges2 = &graph2.edges[v2];

            if Self::is_syntactic_feasible(edges1, edges2, map12, map21, neighs1, neighs2) {
                if depth + 1 == graph1.n {
                    return true;
                }

                map12[v1] = Some(v2);
                map21[v2] = Some(v1);
                Self::update_neighs(graph1, neighs1, &mut stack1, v1);
                Self::update_neighs(graph2, neighs2, &mut stack2, v2);

                let found =
                    Self::isomophism_dfs(graph1, graph2, map12, map21, neighs1, neighs2, depth + 1);
                if found {
                    return true;
                }

                map12[v1] = None;
                map21[v2] = None;
                Self::restore_neighs(neighs1, &mut stack1);
                Self::restore_neighs(neighs2, &mut stack2);
            }
        }

        false
    }

    fn update_neighs(
        graph: &AdjacencyListGraph,
        neighs: &mut [bool],
        stack: &mut Vec<usize>,
        v: usize,
    ) {
        if !neighs[v] {
            neighs[v] = true;
            stack.push(v);
        }

        for &next in graph.edges[v].iter() {
            if !neighs[next] {
                neighs[next] = true;
                stack.push(next);
            }
        }
    }

    fn restore_neighs(neighs: &mut [bool], stack: &mut Vec<usize>) {
        while let Some(v) = stack.pop() {
            neighs[v] = false;
        }
    }

    /// 次に調べる(graph1の頂点列, graph2の頂点)の候補を列挙する
    fn generate_candidates(
        n: usize,
        map12: &[Option<usize>],
        map21: &[Option<usize>],
        neighs1: &[bool],
        neighs2: &[bool],
    ) -> (Vec<usize>, usize) {
        // グラフ1: 今まで見た頂点の隣接頂点であって、まだ確定していないものを列挙する
        // グラフ2: 今まで見た頂点の隣接頂点から適当に1つ選ぶ
        let candidates1 = (0..n)
            .filter(|&v| neighs1[v] && map12[v].is_none())
            .collect_vec();
        let v2 = (0..n).filter(|&v| neighs2[v] && map21[v].is_none()).min();

        // 候補が存在する場合
        if candidates1.len() > 0 {
            if let Some(v2) = v2 {
                return (candidates1, v2);
            }
        }

        // グラフが非連結の場合
        // v1は未確定の頂点全てを候補とし、v2は適当に1つ頂点を持ってきて決める
        let c1 = (0..n).filter(|&v| map12[v].is_none()).collect_vec();
        let v2 = (0..n).filter(|&v| map21[v].is_none()).min().unwrap();
        (c1, v2)
    }

    /// 新しく追加される頂点に隣接する頂点について、頂点相関の実行可能性を判定する
    ///
    /// - 確定済みの点について全単射に矛盾が起きないか
    /// - 隣接点集合に含まれる頂点の数が等しいか
    /// - 隣接点集合に含まれない頂点の数が等しいか
    ///
    /// についてチェック
    fn is_syntactic_feasible(
        edges1: &[usize],
        edges2: &[usize],
        map12: &[Option<usize>],
        map21: &[Option<usize>],
        neighs1: &[bool],
        neighs2: &[bool],
    ) -> bool {
        Self::are_valid_neighbor(edges1, edges2, map12, map21)
            && Self::have_same_inside_neighbors(edges1, edges2, map12, map21, neighs1, neighs2)
            && Self::have_same_outside_neighbors(edges1, edges2, neighs1, neighs2)
    }

    /// ある頂点(u, v)を加えたとき、全単射に矛盾が起きないか
    fn are_valid_neighbor(
        edges1: &[usize],
        edges2: &[usize],
        map12: &[Option<usize>],
        map21: &[Option<usize>],
    ) -> bool {
        if edges1.len() != edges2.len() {
            return false;
        }

        fn ok(my_map: &[Option<usize>], other_edges: &[usize], v: usize) -> bool {
            // v1が確定済みの場合、f(v1)=v2となるv2が相手側の辺にも存在しなければならない
            if let Some(v) = my_map[v] {
                other_edges.contains(&v)
            } else {
                true
            }
        }

        edges1.iter().all(|&v| ok(map12, edges2, v)) && edges2.iter().all(|&v| ok(map21, edges1, v))
    }

    /// neighs \ map について、edgesに含まれる頂点の個数が一致するか
    fn have_same_inside_neighbors(
        edges1: &[usize],
        edges2: &[usize],
        map12: &[Option<usize>],
        map21: &[Option<usize>],
        neighs1: &[bool],
        neighs2: &[bool],
    ) -> bool {
        fn count(edges: &[usize], map: &[Option<usize>], neighs: &[bool]) -> usize {
            edges
                .iter()
                .filter(|&&v| neighs[v] && map[v].is_some())
                .count()
        }

        count(edges1, map12, neighs1) == count(edges2, map21, neighs2)
    }

    /// G \ neighs について、edgesに含まれる頂点の個数が一致するか
    fn have_same_outside_neighbors(
        edges1: &[usize],
        edges2: &[usize],
        neighs1: &[bool],
        neighs2: &[bool],
    ) -> bool {
        fn count(edges: &[usize], neighs: &[bool]) -> usize {
            edges.iter().filter(|&&v| !neighs[v]).count()
        }

        count(edges1, neighs1) == count(edges2, neighs2)
    }
}

#[derive(Debug, Clone)]
struct AdjacencyListGraph {
    n: usize,
    edges: Vec<Vec<usize>>,
}

impl AdjacencyListGraph {
    fn new(n: usize) -> Self {
        Self {
            n,
            edges: vec![vec![]; n],
        }
    }

    fn add_edge(&mut self, u: usize, v: usize) {
        self.edges[u].push(v);
        self.edges[v].push(u);
    }
}

impl From<&Graph> for AdjacencyListGraph {
    fn from(graph: &Graph) -> Self {
        let mut adj_graph = AdjacencyListGraph::new(graph.n);

        for u in 0..graph.n {
            for v in (u + 1)..graph.n {
                if graph[u][v] {
                    adj_graph.add_edge(u, v);
                }
            }
        }

        adj_graph
    }
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
