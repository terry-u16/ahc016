use crate::graph::Graph;

#[derive(Debug, Clone)]
pub struct State {
    groups: Vec<usize>,
    score: i32,
}

impl State {
    pub fn init(graph: &Graph) -> Self {
        let mut state = Self {
            groups: (0..graph.n).collect(),
            score: 0,
        };

        state.calc_score_all(graph);
        state
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn groups(&self) -> &[usize] {
        &self.groups
    }

    pub fn get_group_list(&self) -> Vec<usize> {
        let mut found = [false; 100];
        for &g in self.groups.iter() {
            found[g] = true;
        }

        let mut groups = vec![];

        for (i, &found) in found.iter().enumerate() {
            if found {
                groups.push(i);
            }
        }

        groups
    }

    pub fn get_group_size_list(&self) -> Vec<usize> {
        let mut sizes = vec![0; 100];

        for &v in self.groups.iter() {
            sizes[v] += 1;
        }

        sizes
    }

    pub fn change_group(&mut self, graph: &Graph, node: usize, group: usize) {
        self.score -= self.calc_score_of(graph, node);
        self.groups[node] = group;
        self.score += self.calc_score_of(graph, node);
    }

    fn calc_score_of(&self, graph: &Graph, index: usize) -> i32 {
        let mut score = 0;

        let edges = &graph[index];
        let group1 = self.groups[index];

        for (j, (&edge, &group2)) in edges.iter().zip(self.groups.iter()).enumerate() {
            if index == j {
                continue;
            }

            let same_group = group1 == group2;

            if edge == same_group {
                score += 1;
            }
        }

        score
    }

    fn calc_score_all(&mut self, graph: &Graph) {
        self.score = 0;

        for i in 0..graph.n {
            let edges = &graph[i];
            let group1 = self.groups[i];

            for j in (i + 1)..graph.n {
                let group2 = self.groups[j];
                let same_group = group1 == group2;

                if edges[j] == same_group {
                    self.score += 1;
                }
            }
        }
    }
}
