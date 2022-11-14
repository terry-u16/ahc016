use crate::graph::Graph;
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct BinaryGraph {
    pub n: usize,
    edges: Vec<Vec<i32>>,
}

impl BinaryGraph {
    pub fn new(graph: &Graph) -> Self {
        let mut edges = vec![vec![0; graph.n]; graph.n];

        for i in 0..graph.n {
            for j in 0..graph.n {
                if i == j {
                    continue;
                }

                if graph[i][j] {
                    edges[i][j] = 1;
                } else {
                    edges[i][j] = -1;
                }
            }
        }

        Self { n: graph.n, edges }
    }
}

impl std::fmt::Display for BinaryGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.n {
            for col in 0..self.n {
                let c = if self.edges[row][col] == 1 { '#' } else { '.' };
                write!(f, "{}", c)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl Index<usize> for BinaryGraph {
    type Output = [i32];

    fn index(&self, index: usize) -> &Self::Output {
        &self.edges[index]
    }
}
