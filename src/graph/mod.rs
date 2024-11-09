//! A directed graph of [`Processor`]s connected by [`Edge`]s.

use std::collections::VecDeque;

use edge::Edge;
use node::BuiltNode;
use petgraph::{
    prelude::{Direction, EdgeRef, StableDiGraph},
    visit::DfsPostOrder,
};
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::{
    prelude::{Param, Passthrough},
    processor::{Processor, ProcessorError},
};

pub mod edge;
pub mod node;

pub(crate) type GraphIx = u32;
pub(crate) type NodeIndex = petgraph::graph::NodeIndex<GraphIx>;

pub(crate) type DiGraph = StableDiGraph<BuiltNode, Edge, GraphIx>;

/// An error that can occur during graph processing.
#[derive(Debug, thiserror::Error)]
#[error("Graph run error at node {} ({}): {kind:?}", node_index.index(), node_processor)]
pub struct GraphRunError {
    /// The index of the node that caused the error.
    pub node_index: NodeIndex,
    /// The name of the processor that caused the error.
    pub node_processor: String,
    /// The kind of error that occurred.
    pub kind: GraphRunErrorKind,
}

/// The kind of error that occurred during graph processing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GraphRunErrorKind {
    /// Miscellaneous error.
    #[error("{0}")]
    Other(&'static str),

    /// An error occurred in a processor.
    #[error("Processor error: {0}")]
    ProcessorError(#[from] ProcessorError),
}

/// An error that can occur during graph construction.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GraphConstructionError {
    /// An error for when a node is attempted to be connected to itself.
    #[error("Cannot connect node to itself directly")]
    FeedbackLoop,

    /// An error for when a graph is attempted to be modified after it has been finalized.
    #[error("Graph has already been constructed and cannot be modified; use `Graph::into_builder()` to get a new builder")]
    GraphAlreadyFinished,

    /// An error for when a node is attempted to be connected to a node from a different graph.
    #[error("Cannot connect nodes from different graphs")]
    MismatchedGraphs,

    /// An error for when an operation that expects a single output is attempted on a node that has multiple outputs.
    #[error("Operation `{op}` invalid: Node type `{kind}` has multiple outputs")]
    NodeHasMultipleOutputs {
        /// The operation that caused the error.
        op: String,
        /// The type of node that caused the error.
        kind: String,
    },

    /// An error occurred while attempting to read or write to the filesystem.
    #[error("Filesystem error: {0}")]
    FilesystemError(#[from] std::io::Error),
}

/// A result type for graph run operations.
pub type GraphRunResult<T> = Result<T, GraphRunError>;

/// A result type for graph construction operations.
pub type GraphConstructionResult<T> = Result<T, GraphConstructionError>;

/// A directed graph of [`Processor`]s connected by [`Edge`]s.
///
/// The graph is responsible for managing the processing of its nodes and edges, and can be used to build complex signal processing networks.
///
/// This struct is meant for the actual management of processing the audio graph, or for building custom graphs using a more direct API.
/// See also the [`builder`](crate::builder) module, which provides a more ergonomic way to construct graphs.
#[derive(Default, Clone)]

pub struct Graph {
    digraph: DiGraph,

    // parameters for the graph
    params: hashbrown::HashMap<String, NodeIndex, FxBuildHasher>,

    // cached input/output nodes
    input_nodes: Vec<NodeIndex>,
    output_nodes: Vec<NodeIndex>,

    // internal flags for various states of the graph
    needs_visitor_alloc: bool,

    // cached visitor state for graph traversal
    visit_path: Vec<NodeIndex>,
}

impl Graph {
    /// Creates a new, empty [`Graph`].
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    /// Returns a reference to the inner [`StableDiGraph`] of the graph.
    pub fn digraph(&self) -> &DiGraph {
        &self.digraph
    }

    #[inline]
    /// Returns a mutable reference to the inner [`StableDiGraph`] of the graph.
    pub fn digraph_mut(&mut self) -> &mut DiGraph {
        &mut self.digraph
    }

    #[inline]
    /// Returns `true` if the graph's visitor needs to be reallocated.
    pub fn needs_visitor_alloc(&self) -> bool {
        self.needs_visitor_alloc
    }

    /// Adds a new input [`Passthrough`] node to the graph.
    pub fn add_input(&mut self) -> NodeIndex {
        let idx = self.digraph.add_node(BuiltNode::new(Passthrough));
        self.input_nodes.push(idx);
        idx
    }

    /// Adds a new output [`Passthrough`] node to the graph.
    pub fn add_output(&mut self) -> NodeIndex {
        let idx = self.digraph.add_node(BuiltNode::new(Passthrough));
        self.output_nodes.push(idx);
        idx
    }

    /// Adds a new [`Processor`] to the graph.
    pub fn add_processor(&mut self, processor: impl Processor) -> NodeIndex {
        self.needs_visitor_alloc = true;
        self.digraph.add_node(BuiltNode::new(processor))
    }

    /// Adds a new [`Processor`] representing a [`Param`] to the graph.
    pub fn add_param(&mut self, param: Param) -> NodeIndex {
        let name = param.name().to_string();
        let index = self.add_processor(param);
        self.params.insert(name, index);
        index
    }

    /// Replaces the [`Processor`] at the given node with a new [`Processor`] and returns the old one.
    pub fn replace_processor(&mut self, node: NodeIndex, processor: impl Processor) -> BuiltNode {
        std::mem::replace(&mut self.digraph[node], BuiltNode::new(processor))
    }

    /// Connects two [`Processor`]s with a new [`Edge`].
    /// The signal will flow from the `source` [`Processor`]'s `source_output`-th output to the `target` [`Processor`]'s `target_input`-th input.
    ///
    /// Duplicate edges will not be recreated, and instead the existing one will be returned.
    ///
    /// If there is already an edge connected to the target input, it will be replaced.
    pub fn connect(
        &mut self,
        source: NodeIndex,
        source_output: u32,
        target: NodeIndex,
        target_input: u32,
    ) -> Result<(), GraphConstructionError> {
        if source == target {
            return Err(GraphConstructionError::FeedbackLoop);
        }

        // check if the edge already exists
        for edge in self.digraph.edges_directed(target, Direction::Incoming) {
            let weight = edge.weight();
            if edge.source() == source
                && weight.source_output == source_output
                && weight.target_input == target_input
            {
                // edge already exists
                return Ok(());
            }
        }

        // check if there's already a connection to the target input
        if let Some(edge) = self
            .digraph
            .edges_directed(target, Direction::Incoming)
            .find(|edge| edge.weight().target_input == target_input)
        {
            // remove the existing edge
            self.digraph.remove_edge(edge.id()).unwrap();
        }

        self.needs_visitor_alloc = true;

        self.digraph
            .add_edge(source, target, Edge::new(source_output, target_input));
        Ok(())
    }

    /// Returns the number of input [`Processor`]s in the graph.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_nodes.len()
    }

    /// Returns the number of output [`Processor`]s in the graph.
    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.output_nodes.len()
    }

    /// Returns the number of [`Param`]s in the graph.
    #[inline]
    pub fn num_params(&self) -> usize {
        self.params.len()
    }

    /// Returns the index of the [`Param`] with the given name.
    #[inline]
    pub fn param_index(&self, name: &str) -> Option<NodeIndex> {
        self.params.get(name).copied()
    }

    /// Returns an iterator over the [`Param`]s in the graph.
    #[inline]
    pub fn param_iter(&self) -> impl Iterator<Item = (&str, &Param)> + '_ {
        self.params
            .keys()
            .map(|name| (name.as_str(), self.param_named(name).unwrap()))
    }

    /// Returns a reference to the [`Param`] with the given name.
    #[inline]
    pub fn param_named(&self, name: impl AsRef<str>) -> Option<&Param> {
        self.params
            .get(name.as_ref())
            .and_then(|&idx| self.digraph.node_weight(idx))
            .and_then(|proc| (*proc.processor).downcast_ref())
    }

    /// Returns the index of the input [`Processor`] at the given index.
    #[inline]
    pub fn node_for_input_index(&self, index: usize) -> Option<NodeIndex> {
        self.input_nodes.get(index).copied()
    }

    /// Returns the index of the output [`Processor`] at the given index.
    #[inline]
    pub fn node_for_output_index(&self, index: usize) -> Option<NodeIndex> {
        self.output_nodes.get(index).copied()
    }

    /// Returns a slice of the input indexes in the graph.
    #[inline]
    pub fn input_indices(&self) -> &[NodeIndex] {
        &self.input_nodes
    }

    /// Returns a slice of the output indexes in the graph.
    #[inline]
    pub fn output_indices(&self) -> &[NodeIndex] {
        &self.output_nodes
    }

    #[inline]
    pub(crate) fn allocate_visitor(&mut self) {
        if self.visit_path.capacity() < self.digraph.node_count() {
            self.visit_path = Vec::with_capacity(self.digraph.node_count());
        }
        self.reset_visitor();

        self.needs_visitor_alloc = false;
    }

    #[inline]
    pub(crate) fn reset_visitor(&mut self) {
        self.visit_path.clear();
        let mut visitor = DfsPostOrder::empty(&self.digraph);
        for node in self.digraph.externals(Direction::Incoming) {
            visitor.stack.push(node);
        }
        while let Some(node) = visitor.next(&self.digraph) {
            self.visit_path.push(node);
        }
        self.visit_path.reverse();
    }

    /// Visits each [`Processor`] in the graph in breadth-first order, calling the given closure with a mutable reference to the graph alongside each index.
    // #[inline]
    pub fn visit<F, E>(&mut self, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut Graph, NodeIndex) -> Result<(), E>,
    {
        assert!(
            !self.needs_visitor_alloc,
            "Graph's cached visitor needs allocation; call `allocate_visitor()` first"
        );

        self.reset_visitor();

        for i in 0..self.visit_path.len() {
            f(self, self.visit_path[i])?;
        }

        Ok(())
    }

    /// Partitions the graph into batches of nodes that can be processed in parallel using Kahn's algorithm.
    ///
    /// Calls the given closure with a mutable reference to the graph alongside each batch of nodes.
    #[inline]
    pub fn visit_batched<F, E>(&mut self, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut Graph, &[NodeIndex]) -> Result<(), E>,
    {
        if let Err(e) = self.make_acyclic() {
            panic!("Failed to make graph acyclic: {:?}", e);
        }

        let mut in_degrees = FxHashMap::default();
        let mut queue = VecDeque::new();

        // initialize in-degrees
        for node in self.digraph.node_indices() {
            let in_degree = self
                .digraph
                .neighbors_directed(node, Direction::Incoming)
                .count();
            in_degrees.insert(node, in_degree);

            // add nodes with no incoming edges to the queue
            if in_degree == 0 {
                queue.push_back(node);
            }
        }

        // process nodes in batches
        while !queue.is_empty() {
            let mut batch = Vec::new();

            // pop nodes with zero in-degree
            for _ in 0..queue.len() {
                let node = queue.pop_front().unwrap();
                batch.push(node);

                // decrement in-degrees of neighbors
                for neighbor in self.digraph.neighbors_directed(node, Direction::Outgoing) {
                    let in_degree = in_degrees.get_mut(&neighbor).unwrap();
                    *in_degree -= 1;

                    // add nodes with zero in-degree to the queue
                    if *in_degree == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }

            f(self, &batch)?;
        }

        Ok(())
    }

    /// Detects cycles in the graph using Tarjan's strongly connected components algorithm.
    ///
    /// Returns `Ok(())` if the graph is acyclic, or `Err(cycles)` if the graph contains cycles.
    #[inline]
    pub fn detect_cycles(&self) -> Result<(), Vec<Vec<NodeIndex>>> {
        if !petgraph::algo::is_cyclic_directed(&self.digraph) {
            return Ok(());
        }

        Err(petgraph::algo::tarjan_scc(&self.digraph))
    }

    /// Fixes cycles in the graph by adding a [`MessageTx`](crate::builtins::util::MessageTx) and [`MessageRx`](crate::builtins::util::MessageRx) to break the cycle.
    #[inline]
    pub fn make_acyclic(&mut self) -> Result<(), GraphConstructionError> {
        let Err(sccs) = self.detect_cycles() else {
            return Ok(());
        };

        struct RemovedEdge {
            source: NodeIndex,
            target: NodeIndex,
            edge: Edge,
        }

        // for each strongly connected component
        for scc in sccs {
            // the sccs are in post-order, so the last one is the "root" of the cycle
            let root = scc.last().unwrap();

            // disconnect the root node from the rest of the scc
            let mut original_edges = Vec::new();
            // `.iter().rev().skip(1)` skips the last (root) node
            for node in scc.iter().rev().skip(1) {
                if let Some(edge) = self.digraph.find_edge(*node, *root) {
                    let edge = self.digraph.remove_edge(edge).unwrap();
                    original_edges.push(RemovedEdge {
                        source: *node,
                        target: *root,
                        edge,
                    });
                }
            }

            // for each edge that was originally connected to the root node, add a new message tx/rx pair
            for edge in original_edges {
                let RemovedEdge {
                    source,
                    target,
                    edge,
                } = edge;

                let (tx, rx) = crate::builtins::util::message_channel();
                let tx_node = self.add_processor(tx);
                let rx_node = self.add_processor(rx);

                self.connect(source, edge.source_output, tx_node, 0)
                    .unwrap();
                self.connect(rx_node, 0, target, edge.target_input).unwrap();
            }
        }

        if petgraph::algo::is_cyclic_directed(&self.digraph) {
            let cycles = petgraph::algo::tarjan_scc(&self.digraph);
            panic!("Failed to make graph acyclic: {:?}", cycles);
        }

        Ok(())
    }

    /// Sets the block size of all [`Processor`]s in the graph. This will implicitly reallocate all internal buffers and resources.
    pub fn resize_buffers(&mut self, sample_rate: f64, block_size: usize) -> GraphRunResult<()> {
        self.visit(|graph, node| {
            graph.digraph[node].resize_buffers(sample_rate, block_size);
            Ok(())
        })
    }

    /// Prepares all [`Processor`]s in the graph for processing.
    ///
    /// This should be run at least once before the audio thread starts running, and again anytime the graph structure is modified.
    pub fn prepare(&mut self) -> GraphRunResult<()> {
        self.allocate_visitor();
        self.visit(|graph, node| {
            graph.digraph[node].prepare();
            Ok(())
        })?;

        Ok(())
    }

    /// Writes a DOT representation of the graph to the given writer, suitable for rendering with Graphviz.
    pub fn write_dot<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{:?}", petgraph::dot::Dot::new(&self.digraph))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::Passthrough;

    use super::*;

    #[test]
    fn test_cyclic_graph() {
        let mut graph = Graph::new();

        let a = graph.add_processor(Passthrough);
        let b = graph.add_processor(Passthrough);
        let c = graph.add_processor(Passthrough);
        let d = graph.add_processor(Passthrough);

        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(b, 0, c, 0).unwrap();
        graph.connect(c, 0, d, 0).unwrap();
        graph.connect(d, 0, a, 0).unwrap();

        assert!(graph.detect_cycles().is_err());
        assert!(graph.make_acyclic().is_ok());
        assert!(graph.detect_cycles().is_ok());
    }

    #[test]
    fn test_cyclic_complex_graph() {
        let mut graph = Graph::new();

        let a = graph.add_processor(Passthrough);
        let b = graph.add_processor(Passthrough);
        let c = graph.add_processor(Passthrough);
        let d = graph.add_processor(Passthrough);
        let e = graph.add_processor(Passthrough);
        let f = graph.add_processor(Passthrough);
        let g = graph.add_processor(Passthrough);

        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(b, 0, c, 0).unwrap();
        graph.connect(c, 0, d, 0).unwrap();
        graph.connect(d, 0, e, 0).unwrap();
        graph.connect(e, 0, f, 0).unwrap();
        graph.connect(f, 0, g, 0).unwrap();
        graph.connect(f, 0, g, 1).unwrap(); // connect to a different input as well
        graph.connect(g, 0, c, 0).unwrap();

        assert!(graph.detect_cycles().is_err());
        assert!(graph.make_acyclic().is_ok());
        assert!(graph.detect_cycles().is_ok());
    }

    #[test]
    fn test_graph_batches() {
        let mut graph = Graph::new();

        let a = graph.add_processor(Passthrough);
        let b = graph.add_processor(Passthrough);
        let c = graph.add_processor(Passthrough);
        let d = graph.add_processor(Passthrough);

        // a -> b -> c
        //  \-> d -/
        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(b, 0, c, 0).unwrap();
        graph.connect(a, 0, d, 0).unwrap();
        graph.connect(d, 0, c, 0).unwrap();

        let mut batches = Vec::new();
        graph
            .visit_batched(|_graph, batch| -> Result<(), ()> {
                batches.push(batch.to_vec());
                Ok(())
            })
            .unwrap();

        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], vec![a]);
        assert_eq!(batches[1], vec![d, b]);
        assert_eq!(batches[2], vec![c]);
    }
}
