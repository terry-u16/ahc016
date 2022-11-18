use itertools::Itertools;

use crate::graph::Graph;

/// グラフの同型性を判定するトレイト
pub trait IsomophicChecker {
    /// 与えられたグラフが同型かどうか判定する
    fn is_isomorphic(&self, graph: &Graph) -> bool;
}

/// グラフの次数集合で同型性を判定（大嘘）する構造体
/// 実際は同型性判定はできないのだが、計算量が軽く十分な数のグラフを識別できる
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DegreeChecker {
    n: usize,
    degs: Vec<u32>,
    graph: Graph,
}

#[allow(dead_code)]
impl DegreeChecker {
    pub fn new(graph: &Graph) -> Self {
        let degs = Self::get_degs(graph);
        let n = graph.n;

        Self {
            n,
            degs,
            graph: graph.clone(),
        }
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
}

impl IsomophicChecker for DegreeChecker {
    fn is_isomorphic(&self, graph: &Graph) -> bool {
        let deg = Self::get_degs(graph);

        return self.degs == deg;
    }
}

/// VF2アルゴリズムによりグラフの同型性判定を行う構造体
/// テストを行った結果順列全探索だと時間がかかりすぎると思い込んでしまったために実装されたが、
/// そのテスト自体が間違っていたため本来不要だったという悲しい経歴を持つ
/// 参考: http://satemochi.blog.fc2.com/blog-entry-224.html
#[derive(Debug, Clone)]
pub struct Vf2Checker {
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
    pub fn new(graph: &Graph) -> Self {
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

/// 互いに同型でないグラフをn個以上生成する
pub fn generate_isompic_graphs(n: usize) -> (Vec<Graph>, usize) {
    let graphs = gen_graphs(&GRAPHS_4, 4);

    if graphs.len() >= n {
        return (graphs, 4);
    }

    let graphs = gen_graphs(&GRAPHS_5, 5);

    if graphs.len() >= n {
        return (graphs, 5);
    }

    let graphs = gen_graphs(&GRAPHS_6, 6);

    if graphs.len() >= n {
        return (graphs, 6);
    }

    unreachable!();
}

fn gen_graphs(bits: &[u32], n: usize) -> Vec<Graph> {
    let mut graphs = vec![];

    for &bit in bits.iter() {
        let mut index = 0;
        let mut graph = Graph::new(n);

        for u in 0..n {
            for v in (u + 1)..n {
                if ((bit >> index) & 1) > 0 {
                    graph.connect(u, v);
                }

                index += 1;
            }
        }

        graphs.push(graph);
    }

    graphs
}

const GRAPHS_4: [u32; 11] = [
    0b000000, 0b000001, 0b000011, 0b000111, 0b001011, 0b001100, 0b001101, 0b001111, 0b011110,
    0b011111, 0b111111,
];

const GRAPHS_5: [u32; 34] = [
    0b0000000000,
    0b0000000001,
    0b0000000011,
    0b0000000111,
    0b0000001111,
    0b0000010011,
    0b0000010100,
    0b0000010101,
    0b0000010111,
    0b0000011100,
    0b0000011101,
    0b0000011111,
    0b0000110110,
    0b0000110111,
    0b0000111010,
    0b0000111011,
    0b0000111110,
    0b0000111111,
    0b0001111110,
    0b0001111111,
    0b0010110111,
    0b0010111000,
    0b0010111001,
    0b0010111011,
    0b0010111111,
    0b0011001111,
    0b0011011100,
    0b0011011101,
    0b0011011111,
    0b0011111110,
    0b0011111111,
    0b0111101111,
    0b0111111111,
    0b1111111111,
];

const GRAPHS_6: [u32; 156] = [
    0b000000000000000,
    0b000000000000001,
    0b000000000000011,
    0b000000000000111,
    0b000000000001111,
    0b000000000011111,
    0b000000000100011,
    0b000000000100100,
    0b000000000100101,
    0b000000000100111,
    0b000000000101100,
    0b000000000101101,
    0b000000000101111,
    0b000000000111100,
    0b000000000111101,
    0b000000000111111,
    0b000000001100110,
    0b000000001100111,
    0b000000001101010,
    0b000000001101011,
    0b000000001101110,
    0b000000001101111,
    0b000000001111000,
    0b000000001111001,
    0b000000001111010,
    0b000000001111011,
    0b000000001111110,
    0b000000001111111,
    0b000000011101110,
    0b000000011101111,
    0b000000011110110,
    0b000000011110111,
    0b000000011111110,
    0b000000011111111,
    0b000000111111110,
    0b000000111111111,
    0b000001001100111,
    0b000001001101000,
    0b000001001101001,
    0b000001001101011,
    0b000001001101111,
    0b000001001111000,
    0b000001001111001,
    0b000001001111011,
    0b000001001111111,
    0b000001010001111,
    0b000001010010000,
    0b000001010010001,
    0b000001010010011,
    0b000001010010110,
    0b000001010010111,
    0b000001010011111,
    0b000001010101100,
    0b000001010101101,
    0b000001010101111,
    0b000001010110011,
    0b000001010110100,
    0b000001010110101,
    0b000001010110110,
    0b000001010110111,
    0b000001010111100,
    0b000001010111101,
    0b000001010111111,
    0b000001011101110,
    0b000001011101111,
    0b000001011110110,
    0b000001011110111,
    0b000001011111000,
    0b000001011111001,
    0b000001011111010,
    0b000001011111011,
    0b000001011111110,
    0b000001011111111,
    0b000001110011000,
    0b000001110011001,
    0b000001110011010,
    0b000001110011011,
    0b000001110011110,
    0b000001110011111,
    0b000001110111010,
    0b000001110111011,
    0b000001110111100,
    0b000001110111101,
    0b000001110111110,
    0b000001110111111,
    0b000001111111110,
    0b000001111111111,
    0b000011011001111,
    0b000011011010101,
    0b000011011010111,
    0b000011011011111,
    0b000011011101111,
    0b000011011110100,
    0b000011011110101,
    0b000011011110111,
    0b000011011111100,
    0b000011011111101,
    0b000011011111111,
    0b000011101011000,
    0b000011101011001,
    0b000011101011011,
    0b000011101011100,
    0b000011101011101,
    0b000011101011111,
    0b000011101111011,
    0b000011101111100,
    0b000011101111101,
    0b000011101111111,
    0b000011111011100,
    0b000011111011101,
    0b000011111011110,
    0b000011111011111,
    0b000011111111110,
    0b000011111111111,
    0b000111111011100,
    0b000111111011101,
    0b000111111011111,
    0b000111111111111,
    0b001011011101111,
    0b001011011110000,
    0b001011011110001,
    0b001011011110011,
    0b001011011110111,
    0b001011011111111,
    0b001011100010001,
    0b001011100010011,
    0b001011100010111,
    0b001011100011111,
    0b001011100110101,
    0b001011100110111,
    0b001011100111100,
    0b001011100111101,
    0b001011100111110,
    0b001011100111111,
    0b001011101110111,
    0b001011101111010,
    0b001011101111011,
    0b001011101111110,
    0b001011101111111,
    0b001011111111110,
    0b001011111111111,
    0b001100111111110,
    0b001100111111111,
    0b001101110011111,
    0b001101110111100,
    0b001101110111101,
    0b001101110111111,
    0b001101111111110,
    0b001101111111111,
    0b001111111011101,
    0b001111111011111,
    0b001111111111111,
    0b011110111111110,
    0b011110111111111,
    0b011111111111111,
    0b111111111111111,
];
