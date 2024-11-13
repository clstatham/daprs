//! Contains the definition of the `Edge` struct, which represents an edge in the graph.

/// Represents a connection between an output and an input of two nodes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    /// The output index of the source node.
    pub source_output: u32,
    /// The input index of the target node.
    pub target_input: u32,
}

impl Edge {
    /// Creates a new `Edge` with the given source output and target input.
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
