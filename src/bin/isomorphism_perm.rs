use std::ops::Index;

use itertools::Itertools;

const N: usize = 6;
const EDGE_COUNTS: usize = N * (N - 1) / 2;

fn main() {
    let mut graphs = vec![];
    let checker = PermutationalChecker;

    for bits in 0..(1 << EDGE_COUNTS) {
        let graph = gen_graph(bits);
        let mut found = false;

        for (_, g) in graphs.iter() {
            found |= checker.are_isomorphic(&graph, g);
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

/// グラフの同型性を判定するトレイト
trait IsomophicChecker {
    /// 2つのグラフが同型かどうか判定する
    fn are_isomorphic(&self, graph1: &Graph, graph2: &Graph) -> bool;
}

#[derive(Debug, Clone, Copy)]
struct PermutationalChecker;

impl PermutationalChecker {
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
}

impl IsomophicChecker for PermutationalChecker {
    fn are_isomorphic(&self, graph1: &Graph, graph2: &Graph) -> bool {
        let deg1 = Self::get_degs(graph1);
        let deg2 = Self::get_degs(graph2);

        return deg1 == deg2;

        if deg1 != deg2 {
            return false;
        }

        'main: for p in (0..N).permutations(N) {
            for i in 0..N {
                for j in (i + 1)..N {
                    if graph1[i][j] != graph2[p[i]][p[j]] {
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
#[derive(Debug, Clone, Copy)]
struct Vf2Checker;

impl IsomophicChecker for Vf2Checker {
    fn are_isomorphic(&self, graph1: &Graph, graph2: &Graph) -> bool {
        todo!()
    }
}

impl Vf2Checker {
    fn isomophism_dfs(
        &self,
        graph1: &Graph,
        graph2: &Graph,
        map12: &mut [Option<usize>],
        map21: &mut [Option<usize>],
        neighs1: &mut [bool],
        neighs2: &mut [bool],
    ) -> bool {
        todo!();
    }

    /// 次に調べる(graph1の頂点列, graph2の頂点)の候補を列挙する
    fn generate_candidates(
        &self,
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
                .filter(|&&v| neighs[v] && map[v].is_none())
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

impl From<Graph> for AdjacencyListGraph {
    fn from(graph: Graph) -> Self {
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
