#[allow(dead_code)]
pub mod barcode;
#[allow(dead_code)]
pub mod binomial;
#[allow(dead_code)]
pub mod clique;
pub mod isomorphism;

use crate::graph::Graph;

pub trait Encoder {
    fn graph_size(&self) -> usize;
    fn encode(&self, index: usize) -> Graph;
    fn decode(&self, graph: &Graph, duration: f64) -> usize;
}
