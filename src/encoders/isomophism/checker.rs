use crate::graph::Graph;

/// グラフの同型性を判定するトレイト
pub trait IsomophicChecker {
    /// 2つのグラフが同型かどうか判定する
    fn are_isomorphic(&self, graph1: &Graph, graph2: &Graph) -> bool;

    /// 互いに同型でないグラフをn個以上生成する
    fn generate_isompic_graphs(&self, n: usize) -> (Vec<Graph>, usize) {
        let mut size = 2;

        loop {
            let mut graphs = vec![];
            let edge_counts = size * (size - 1) / 2;

            for bits in 0..(1 << edge_counts) {
                let graph = gen_graph(bits, size);
                let found = graphs.iter().any(|g| self.are_isomorphic(&graph, g));

                if !found {
                    graphs.push(graph);
                }
            }

            if graphs.len() >= n {
                return (graphs, size);
            }

            size += 1;
        }
    }
}

fn gen_graph(bits: usize, n: usize) -> Graph {
    let mut graph = Graph::new(n);
    let mut index = 0;

    for i in 0..n {
        for j in (i + 1)..n {
            if ((bits >> index) & 1) > 0 {
                graph.connect(i, j);
            }

            index += 1;
        }
    }

    graph
}

/// グラフの次数集合で同型性を判定（大嘘）する構造体
/// 実際は同型性判定はできないのだが、計算量が軽く十分な数のグラフを識別できる
#[derive(Debug, Clone, Copy)]
pub struct DegreeChecker;

impl DegreeChecker {
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
    fn are_isomorphic(&self, graph1: &Graph, graph2: &Graph) -> bool {
        let deg1 = Self::get_degs(graph1);
        let deg2 = Self::get_degs(graph2);

        return deg1 == deg2;
    }
}
