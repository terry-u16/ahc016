pub mod barcode;
pub mod binomial;
#[allow(dead_code)]
pub mod clique;

use crate::graph::Graph;

pub trait Encoder {
    fn graph_size(&self) -> usize;
    fn encode(&self, index: usize) -> Graph;
    fn decode(&self, graph: &Graph, duration: f64) -> usize;
}
