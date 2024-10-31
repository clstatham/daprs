use edge::Edge;
use node::GraphNode;
use petgraph::{
    prelude::{Direction, EdgeRef, StableDiGraph},
    visit::DfsPostOrder,
};

use crate::{
    processor::{Process, Processor},
    signal::Buffer,
};

pub mod edge;
pub mod node;

pub type GraphIx = u32;
pub type NodeIndex = petgraph::graph::NodeIndex<GraphIx>;
pub type EdgeIndex = petgraph::graph::EdgeIndex<GraphIx>;

pub type DiGraph = StableDiGraph<GraphNode, Edge, GraphIx>;

#[derive(Debug, thiserror::Error)]
#[error("Graph run error at node {node_index:?} ({node_processor:?}): {kind:?}")]
pub struct GraphRunError {
    pub node_index: NodeIndex,
    pub node_processor: String,
    pub kind: GraphRunErrorKind,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GraphRunErrorKind {
    #[error("{0}")]
    Other(&'static str),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GraphConstructionError {
    #[error("Cannot connect node to itself directly")]
    FeedbackLoop,
    #[error("Graph has already been constructed and cannot be modified; use `Graph::into_builder()` to get a new builder")]
    GraphAlreadyFinished,
    #[error("Cannot connect nodes from different graphs")]
    MismatchedGraphs,
    #[error("Operation `{op}` invalid: Node type `{kind}` has multiple outputs")]
    NodeHasMultipleOutputs { op: String, kind: String },
}

pub type GraphRunResult<T> = Result<T, GraphRunError>;
pub type GraphConstructionResult<T> = Result<T, GraphConstructionError>;

/// A directed graph of [`GraphNode`]s connected by [`Edge`]s.
///
/// The graph is responsible for managing the processing of its nodes and edges, and can be used to build complex signal processing networks.
///
/// This struct is meant for the actual management of processing the audio graph, or for building custom graphs using a more direct API.
/// See also the [`builder`] module, which provides a more ergonomic way to construct graphs.
#[derive(Default, Clone)]
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

    /// Adds a new [`GraphNode`] with the given [`Processor`](node::Processor) to the graph.
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

    /// Replaces the [`GraphNode`] at the given [`NodeIndex`] in-place with a new [`Processor`](node::Processor).
    pub fn replace_processor(&mut self, node: NodeIndex, processor: impl Process) -> GraphNode {
        self.needs_reset = true;
        self.needs_prepare = true;
        std::mem::replace(&mut self.digraph[node], GraphNode::new_processor(processor))
    }

    /// Connects two [`GraphNode`]s with a new [`Edge`].
    /// The signal will flow from the `source` [`GraphNode`]'s `source_output`-th output to the `target` [`GraphNode`]'s `target_input`-th input.
    ///
    /// Duplicate edges will not be recreated, and instead the existing one will be returned.
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

    /// Returns the [`NodeIndex`] of the input [`GraphNode`] at the given index.
    #[inline]
    pub fn node_for_input_index(&self, index: usize) -> Option<NodeIndex> {
        self.input_nodes.get(index).copied()
    }

    /// Returns the [`NodeIndex`] of the output [`GraphNode`] at the given index.
    #[inline]
    pub fn node_for_output_index(&self, index: usize) -> Option<NodeIndex> {
        self.output_nodes.get(index).copied()
    }

    /// Returns a slice of the input [`NodeIndex`]es in the graph.
    #[inline]
    pub fn input_indices(&self) -> &[NodeIndex] {
        &self.input_nodes
    }

    /// Returns a slice of the output [`NodeIndex`]es in the graph.
    #[inline]
    pub fn output_indices(&self) -> &[NodeIndex] {
        &self.output_nodes
    }

    /// Copies the given data into the input [`Buffer`] of the input [`GraphNode`] at the given index.
    #[inline]
    pub fn copy_input(&mut self, input_index: usize, data: &Buffer) {
        let input_index = self
            .input_nodes
            .get(input_index)
            .expect("Input index out of bounds");
        let input = &mut self.digraph[*input_index];
        if let GraphNode::Passthrough(input) = input {
            input.copy_from_slice(data);
        } else {
            panic!("Node at input index is not an input node");
        }
    }

    /// Returns a reference to the output [`Buffer`] of the output [`GraphNode`] at the given index.
    #[inline]
    pub fn get_output(&self, output_index: usize) -> &Buffer {
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

    #[inline]
    pub fn inputs(&self) -> impl Iterator<Item = &Buffer> {
        self.input_nodes.iter().map(|&idx| {
            if let GraphNode::Passthrough(input) = &self.digraph[idx] {
                input
            } else {
                panic!("Node at input index is not an input node");
            }
        })
    }

    /// Returns an iterator over the output [`Buffer`]s of the output [`GraphNode`]s in the graph.
    #[inline]
    pub fn outputs(&self) -> impl Iterator<Item = &Buffer> {
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

    /// Visits each [`GraphNode`] in the graph in breadth-first order, calling the given closuure with a mutable reference to the graph alongside each [`NodeIndex`].
    #[inline]
    pub fn visit<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Graph, NodeIndex),
    {
        assert!(
            !self.needs_visitor_alloc,
            "Graph's cached visitor needs allocation; call `allocate_visitor()` first"
        );

        self.reset_visitor();

        for i in 0..self.visit_path.len() {
            f(self, self.visit_path[i]);
        }
    }

    /// Sets the block size of all [`GraphNode`]s in the graph. This will implicitly reallocate all internal buffers and resources.
    pub fn resize_buffers(&mut self, sample_rate: f64, block_size: usize) {
        self.visit(|graph, node| {
            graph.digraph[node].resize_buffers(sample_rate, block_size);
        });
    }

    /// Allocates all [`GraphNode`]s' internal input and output buffers, along with various internal resources to the graph.
    ///
    /// This should be run at least once before the audio thread starts running, and again anytime the buffer size or sample rate change or the graph structure is modified.
    pub fn reset(&mut self, sample_rate: f64, block_size: usize) {
        let mut max_edges = 0;

        self.allocate_visitor();
        self.visit(|graph, node| {
            // allocate the node's inputs and outputs
            graph.digraph[node].resize_buffers(sample_rate, block_size);

            let num_inputs = graph
                .digraph
                .edges_directed(node, Direction::Incoming)
                .count();
            max_edges = max_edges.max(num_inputs);
        });

        // preallocate the edge cache used in `process()`
        // the number of edges per node is likely relatively small, so we round up the cache size just to be sure that no allocations happen in `process()`
        self.edge_cache = Vec::with_capacity((max_edges * 2).next_power_of_two());

        self.needs_reset = false;
    }

    /// Prepares all [`GraphNode`]s in the graph for processing.
    ///
    /// This should be run at least once before the audio thread starts running, and again anytime the graph structure is modified.
    pub fn prepare_nodes(&mut self) {
        self.allocate_visitor();
        self.visit(|graph, node| graph.digraph[node].prepare());

        self.needs_prepare = false;
    }

    /// Returns a mutable reference to the input [`Buffer`] of the [`GraphNode`] at the given [`NodeIndex`] and input index.
    #[inline]
    pub fn get_node_input_mut(&mut self, node: NodeIndex, input_index: usize) -> &mut Buffer {
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

    /// Returns a reference to the output [`Buffer`] of the [`GraphNode`] at the given [`NodeIndex`] and output index.
    #[inline]
    pub fn get_node_output(&self, node: NodeIndex, output_index: usize) -> &Buffer {
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
    /// The results of the processing can be read from the output [`Buffer`]s of the output [`GraphNode`]s via [`Graph::get_output`] or [`Graph::outputs`].
    #[inline]
    pub fn process(&mut self) {
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

        self.visit(|graph: &mut Graph, node_id| {
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
                target_buffer.copy_from_slice(source_buffer);
            }

            // process the node
            graph.digraph[node_id].process();
        });
    }

    /// Writes a DOT representation of the graph to the given writer, suitable for rendering with Graphviz.
    pub fn write_dot<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{:?}", petgraph::dot::Dot::new(&self.digraph))
    }
}
