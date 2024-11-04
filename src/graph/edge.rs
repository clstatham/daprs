//! Contains the definition of the `Edge` struct, which represents an edge in the graph.

use serde::{Deserialize, Serialize};

/// An edge in the graph, connecting an output of one node to an input of another.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {
    /// The output of the source node that this edge connects.
    pub source_output: u32,
    /// The input of the target node that this edge connects.
    pub target_input: u32,
}

impl Edge {
    /// Creates a new edge connecting the given output of the source node to the given input of the target node.
    pub fn new(source_output: u32, target_input: u32) -> Self {
        Edge {
            source_output,
            target_input,
        }
    }
}

impl std::fmt::Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}->{}", self.source_output, self.target_input)
    }
}
