use std::ops::Index;

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
                let c = if self.edges[row][col] { '1' } else { '0' };
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

#[cfg(test)]
mod test {
    use super::Graph;

    #[test]
    fn desearialize_test() {
        let n = 4;
        let s = "100101";
        let actual = Graph::deserialize(&s, n);
        let expected = vec![
            vec![false, true, false, false],
            vec![true, false, true, false],
            vec![false, true, false, true],
            vec![false, false, true, false],
        ];

        assert_eq!(expected, actual.edges);
    }

    #[test]
    fn searialize_test() {
        let n = 4;
        let mut graph = Graph::new(n);
        graph.connect(0, 1);
        graph.connect(1, 2);
        graph.connect(2, 3);

        let actual = graph.serialize();
        let expected = "100101";

        assert_eq!(expected, actual);
    }
}
