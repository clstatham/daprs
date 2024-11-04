//! A directed graph of [`GraphNode`]s connected by [`Edge`]s.

use edge::Edge;
use node::GraphNode;
use petgraph::{
    prelude::{Direction, EdgeRef, StableDiGraph},
    visit::DfsPostOrder,
};
use serde::{Deserialize, Serialize};

use crate::{
    processor::{Process, Processor, ProcessorError},
    signal::SignalBuffer,
};

pub mod edge;
pub mod node;

pub(crate) type GraphIx = u32;
pub(crate) type NodeIndex = petgraph::graph::NodeIndex<GraphIx>;
pub(crate) type EdgeIndex = petgraph::graph::EdgeIndex<GraphIx>;

pub(crate) type DiGraph = StableDiGraph<GraphNode, Edge, GraphIx>;

/// An error that can occur during graph processing.
#[derive(Debug, thiserror::Error)]
#[error("Graph run error at node {node_index:?} ({node_processor:?}): {kind:?}")]
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

    /// An error for when a node is attempted to be connected to a node that does not exist.
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

/// A directed graph of [`GraphNode`]s connected by [`Edge`]s.
///
/// The graph is responsible for managing the processing of its nodes and edges, and can be used to build complex signal processing networks.
///
/// This struct is meant for the actual management of processing the audio graph, or for building custom graphs using a more direct API.
/// See also the [`builder`](crate::builder) module, which provides a more ergonomic way to construct graphs.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Graph {
    digraph: DiGraph,

    // cached input/output nodes
    input_nodes: Vec<NodeIndex>,
    output_nodes: Vec<NodeIndex>,

    // internal flags for various states of the graph
    needs_reset: bool,
    needs_prepare: bool,
    needs_visitor_alloc: bool,

    // cached internal state to avoid allocations in `process()`
    edge_cache: Vec<(NodeIndex, Edge)>,

    // cached visitor state for graph traversal
    visit_path: Vec<NodeIndex>,
}

impl Graph {
    /// Creates a new, empty [`Graph`].
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    /// Returns the inner [`StableDiGraph`] of the graph.
    pub fn digraph(&self) -> &DiGraph {
        &self.digraph
    }

    /// Returns `true` if [`reset`](Graph::reset) must be called before the next [`process`](Graph::process) call.
    #[inline]
    pub fn needs_reset(&self) -> bool {
        self.needs_reset
    }

    /// Returns `true` if [`prepare_nodes`](Graph::prepare_nodes) must be called before the next [`process`](Graph::process) call.
    #[inline]
    pub fn needs_prepare(&self) -> bool {
        self.needs_prepare
    }

    /// Adds a new input [`Passthrough`](GraphNode::Passthrough) node to the graph.
    pub fn add_input(&mut self) -> NodeIndex {
        self.needs_reset = true;
        let idx = self.digraph.add_node(GraphNode::new_input());
        self.input_nodes.push(idx);
        idx
    }

    /// Adds a new output [`Passthrough`](GraphNode::Passthrough) node to the graph.
    pub fn add_output(&mut self) -> NodeIndex {
        self.needs_reset = true;
        let idx = self.digraph.add_node(GraphNode::new_output());
        self.output_nodes.push(idx);
        idx
    }

    /// Adds a new [`GraphNode`] with the given [`Processor`] to the graph.
    pub fn add_processor_object(&mut self, processor: Processor) -> NodeIndex {
        self.needs_reset = true;
        self.needs_prepare = true;
        self.needs_visitor_alloc = true;
        self.digraph.add_node(GraphNode::Processor(processor))
    }

    /// Adds a new [`GraphNode`] with the given [`Process`] functionality to the graph.
    pub fn add_processor(&mut self, processor: impl Process) -> NodeIndex {
        self.needs_reset = true;
        self.needs_prepare = true;
        self.needs_visitor_alloc = true;
        self.digraph.add_node(GraphNode::new_processor(processor))
    }

    /// Replaces the [`GraphNode`] at the given index in-place with a new [`Processor`].
    pub fn replace_processor(&mut self, node: NodeIndex, processor: impl Process) -> GraphNode {
        self.needs_reset = true;
        self.needs_prepare = true;
        std::mem::replace(&mut self.digraph[node], GraphNode::new_processor(processor))
    }

    /// Connects two [`GraphNode`]s with a new [`Edge`].
    /// The signal will flow from the `source` [`GraphNode`]'s `source_output`-th output to the `target` [`GraphNode`]'s `target_input`-th input.
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
    ) -> Result<EdgeIndex, GraphConstructionError> {
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
                return Ok(edge.id());
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

        self.needs_reset = true;
        self.needs_prepare = true;
        self.needs_visitor_alloc = true;

        let edge = self
            .digraph
            .add_edge(source, target, Edge::new(source_output, target_input));
        Ok(edge)
    }

    /// Returns the number of input [`GraphNode`]s in the graph.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_nodes.len()
    }

    /// Returns the number of output [`GraphNode`]s in the graph.
    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.output_nodes.len()
    }

    /// Returns the index of the input [`GraphNode`] at the given index.
    #[inline]
    pub fn node_for_input_index(&self, index: usize) -> Option<NodeIndex> {
        self.input_nodes.get(index).copied()
    }

    /// Returns the index of the output [`GraphNode`] at the given index.
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

    /// Copies the given data into the input [`SignalBuffer`] of the input [`GraphNode`] at the given index.
    #[inline]
    pub fn copy_input(&mut self, input_index: usize, data: &SignalBuffer) {
        let input_index = self
            .input_nodes
            .get(input_index)
            .expect("Input index out of bounds");
        let input = &mut self.digraph[*input_index];
        if let GraphNode::Passthrough(input) = input {
            input.copy_from(data);
        } else {
            panic!("Node at input index is not an input node");
        }
    }

    /// Returns a reference to the output [`SignalBuffer`] of the output [`GraphNode`] at the given index.
    #[inline]
    pub fn get_output(&self, output_index: usize) -> &SignalBuffer {
        let output_index = self
            .output_nodes
            .get(output_index)
            .expect("Output index out of bounds");
        let output = &self.digraph[*output_index];
        if let GraphNode::Passthrough(output) = output {
            output
        } else {
            panic!("Node at output index is not an output node");
        }
    }

    /// Returns an iterator over the input [`SignalBuffer`]s of the input [`GraphNode`]s in the graph.
    #[inline]
    pub fn inputs(&self) -> impl Iterator<Item = &SignalBuffer> {
        self.input_nodes.iter().map(|&idx| {
            if let GraphNode::Passthrough(input) = &self.digraph[idx] {
                input
            } else {
                panic!("Node at input index is not an input node");
            }
        })
    }

    /// Returns an iterator over the output [`SignalBuffer`]s of the output [`GraphNode`]s in the graph.
    #[inline]
    pub fn outputs(&self) -> impl Iterator<Item = &SignalBuffer> {
        self.output_nodes.iter().map(|&idx| {
            if let GraphNode::Passthrough(output) = &self.digraph[idx] {
                output
            } else {
                panic!("Node at output index is not an output node");
            }
        })
    }

    fn allocate_visitor(&mut self) {
        self.visit_path = Vec::with_capacity(self.digraph.node_count());
        self.reset_visitor();

        self.needs_visitor_alloc = false;
    }

    #[inline]
    fn reset_visitor(&mut self) {
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

    /// Visits each [`GraphNode`] in the graph in breadth-first order, calling the given closure with a mutable reference to the graph alongside each index.
    #[inline]
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

    /// Sets the block size of all [`GraphNode`]s in the graph. This will implicitly reallocate all internal buffers and resources.
    pub fn resize_buffers(&mut self, sample_rate: f64, block_size: usize) -> GraphRunResult<()> {
        self.visit(|graph, node| {
            graph.digraph[node].resize_buffers(sample_rate, block_size);
            Ok(())
        })
    }

    /// Allocates all [`GraphNode`]s' internal input and output buffers, along with various internal resources to the graph.
    ///
    /// This should be run at least once before the audio thread starts running, and again anytime the buffer size or sample rate change or the graph structure is modified.
    pub fn reset(&mut self, sample_rate: f64, block_size: usize) -> GraphRunResult<()> {
        let mut max_edges = 0;

        self.allocate_visitor();
        self.visit(|graph, node| -> GraphRunResult<()> {
            // allocate the node's inputs and outputs
            graph.digraph[node].resize_buffers(sample_rate, block_size);

            let num_inputs = graph
                .digraph
                .edges_directed(node, Direction::Incoming)
                .count();
            max_edges = max_edges.max(num_inputs);

            Ok(())
        })?;

        // preallocate the edge cache used in `process()`
        // the number of edges per node is likely relatively small, so we round up the cache size just to be sure that no allocations happen in `process()`
        self.edge_cache = Vec::with_capacity((max_edges * 2).next_power_of_two());

        self.needs_reset = false;

        Ok(())
    }

    /// Prepares all [`GraphNode`]s in the graph for processing.
    ///
    /// This should be run at least once before the audio thread starts running, and again anytime the graph structure is modified.
    pub fn prepare(&mut self) -> GraphRunResult<()> {
        self.allocate_visitor();
        self.visit(|graph, node| {
            graph.digraph[node].prepare();
            Ok(())
        })?;

        self.needs_prepare = false;

        Ok(())
    }

    /// Returns a mutable reference to the input [`SignalBuffer`] of the [`GraphNode`] at the given index and input index.
    #[inline]
    pub fn get_node_input_mut(&mut self, node: NodeIndex, input_index: usize) -> &mut SignalBuffer {
        match &mut self.digraph[node] {
            GraphNode::Passthrough(buffer) => {
                if input_index != 0 {
                    panic!("Input node has only one input buffer");
                }
                buffer
            }
            GraphNode::Processor(processor) => processor.input_mut(input_index),
        }
    }

    /// Returns a reference to the output [`SignalBuffer`] of the [`GraphNode`] at the given index and output index.
    #[inline]
    pub fn get_node_output(&self, node: NodeIndex, output_index: usize) -> &SignalBuffer {
        match &self.digraph[node] {
            GraphNode::Passthrough(buffer) => {
                if output_index != 0 {
                    panic!("Output node has only one output buffer");
                }
                buffer
            }
            GraphNode::Processor(processor) => processor.output(output_index),
        }
    }

    /// Processes all [`GraphNode`]s in the graph.
    /// This should be called once per audio block.
    ///
    /// The results of the processing can be read from the output [`SignalBuffer`]s of the output [`GraphNode`]s via [`Graph::get_output`] or [`Graph::outputs`].
    #[inline]
    pub fn process(&mut self) -> GraphRunResult<()> {
        assert!(
            !self.needs_reset,
            "Graph nodes need reset; call `reset()` first"
        );
        assert!(
            !self.needs_prepare,
            "Graph nodes need preparation; call `prepare_nodes()` first"
        );
        assert!(
            !self.needs_visitor_alloc,
            "Graph's cached visitor needs allocation; call `allocate_visitor()` first"
        );

        self.visit(|graph: &mut Graph, node_id| -> GraphRunResult<()> {
            // copy the inputs from the source nodes to the target node
            graph.edge_cache.extend(
                graph
                    .digraph
                    .edges_directed(node_id, Direction::Incoming)
                    .map(|edge| (edge.source(), *edge.weight())),
            );
            for (source_id, edge) in graph.edge_cache.drain(..) {
                let Edge {
                    source_output,
                    target_input,
                } = edge;

                let (source, target) = graph.digraph.index_twice_mut(source_id, node_id);

                let source_buffer = match source {
                    GraphNode::Processor(processor) => processor.output(source_output as usize),
                    GraphNode::Passthrough(buffer) => buffer,
                };

                let target_buffer = match target {
                    GraphNode::Processor(processor) => processor.input_mut(target_input as usize),
                    GraphNode::Passthrough(buffer) => buffer,
                };
                target_buffer.copy_from(source_buffer);
            }

            // process the node
            graph.digraph[node_id]
                .process()
                .map_err(|err| GraphRunError {
                    node_index: node_id,
                    node_processor: graph.digraph[node_id].name().to_string(),
                    kind: GraphRunErrorKind::ProcessorError(err),
                })?;

            Ok(())
        })?;

        Ok(())
    }

    /// Writes a DOT representation of the graph to the given writer, suitable for rendering with Graphviz.
    pub fn write_dot<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{:?}", petgraph::dot::Dot::new(&self.digraph))
    }
}
