use edge::Edge;
use node::{GraphNode, Process};
use petgraph::{
    prelude::{Direction, EdgeRef, StableDiGraph},
    visit::{Bfs, Visitable},
};

use crate::sample::{Buffer, Sample, SignalKind};

pub mod builder;
pub mod edge;
pub mod node;

pub type GraphIx = u32;
pub type NodeIndex = petgraph::graph::NodeIndex<GraphIx>;
pub type EdgeIndex = petgraph::graph::EdgeIndex<GraphIx>;

pub type DiGraph = StableDiGraph<GraphNode, Edge, GraphIx>;

pub type Visitor = Bfs<NodeIndex, <DiGraph as Visitable>::Map>;

#[derive(Default, Clone)]
pub struct Graph {
    digraph: DiGraph,

    // cached input/output nodes
    input_nodes: Vec<NodeIndex>,
    output_nodes: Vec<NodeIndex>,

    // cached input/output buffers
    input_buffers: Vec<Buffer>,
    output_buffers: Vec<Buffer>,

    // internal flags for various states of the graph
    needs_reset: bool,
    needs_prepare: bool,
    needs_visitor_alloc: bool,

    // cached internal state to avoid allocations in `process()`
    edge_cache: Vec<(NodeIndex, Edge)>,

    visitor: Visitor,
}

impl Graph {
    /// Creates a new, empty [`Graph`].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_builder() -> builder::GraphBuilder {
        builder::GraphBuilder::new()
    }

    pub fn builder(self) -> builder::GraphBuilder {
        builder::GraphBuilder::from_graph(self)
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

    /// Adds a new [`Input`](Node::Input) node to the graph.
    pub fn add_input(&mut self) -> NodeIndex {
        self.needs_reset = true;
        let idx = self.digraph.add_node(GraphNode::new_input());
        self.input_nodes.push(idx);
        self.input_buffers.push(Buffer::zeros(0, SignalKind::Audio));
        idx
    }

    /// Adds a new [`Output`](Node::Output) node to the graph.
    pub fn add_output(&mut self) -> NodeIndex {
        self.needs_reset = true;
        let idx = self.digraph.add_node(GraphNode::new_output());
        self.output_nodes.push(idx);
        self.output_buffers
            .push(Buffer::zeros(0, SignalKind::Audio));
        idx
    }

    /// Adds a new [`Node`] with the given [`Process`] functionality to the graph.
    pub fn add_processor(&mut self, processor: impl Process) -> NodeIndex {
        self.needs_reset = true;
        self.needs_prepare = true;
        self.needs_visitor_alloc = true;
        self.digraph.add_node(GraphNode::new_processor(processor))
    }

    /// Connects two [`Node`]s with a new [`Edge`].
    /// The signal will flow from the `source` [`Node`]'s `source_output`-th output to the `target` [`Node`]'s `target_input`-th input.
    ///
    /// Duplicate edges will not be recreated, and instead the existing one will be returned.
    pub fn connect(
        &mut self,
        source: NodeIndex,
        source_output: u32,
        target: NodeIndex,
        target_input: u32,
    ) -> EdgeIndex {
        assert_ne!(source, target, "Cannot connect node to itself directly");

        // check if the edge already exists
        for edge in self.digraph.edges_directed(target, Direction::Incoming) {
            let weight = edge.weight();
            if edge.source() == source
                && weight.source_output == source_output
                && weight.target_input == target_input
            {
                // edge already exists
                return edge.id();
            }
        }

        self.needs_reset = true;
        self.needs_prepare = true;
        self.needs_visitor_alloc = true;

        self.digraph
            .add_edge(source, target, Edge::new(source_output, target_input))
    }

    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_nodes.len()
    }

    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.output_nodes.len()
    }

    #[inline]
    pub fn node_for_input_index(&self, index: usize) -> Option<NodeIndex> {
        self.input_nodes.get(index).copied()
    }

    #[inline]
    pub fn node_for_output_index(&self, index: usize) -> Option<NodeIndex> {
        self.output_nodes.get(index).copied()
    }

    #[inline]
    pub fn input_indices(&self) -> &[NodeIndex] {
        &self.input_nodes
    }

    #[inline]
    pub fn output_indices(&self) -> &[NodeIndex] {
        &self.output_nodes
    }

    #[inline]
    pub fn copy_input(&mut self, input_index: usize, data: &[Sample]) {
        self.input_buffers[input_index].copy_from_slice(data);
    }

    #[inline]
    pub fn get_output(&self, output_index: usize) -> &[Sample] {
        &self.output_buffers[output_index]
    }

    #[inline]
    pub fn outputs(&self) -> &[Buffer] {
        &self.output_buffers
    }

    pub fn allocate_visitor(&mut self) {
        self.visitor = Visitor::new(&self.digraph, NodeIndex::default());
        self.reset_visitor();

        self.needs_visitor_alloc = false;
    }

    #[inline]
    pub fn reset_visitor(&mut self) {
        self.visitor.discovered.clear();
        self.visitor.stack.clear();

        let starts = self.digraph.externals(Direction::Incoming);
        self.visitor.stack.extend(starts);
    }

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

        while let Some(node) = self.visitor.next(&self.digraph) {
            f(self, node);
        }
    }

    pub fn set_block_size(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        self.visit(|graph, node| {
            graph.digraph[node].set_block_size(audio_rate, control_rate, block_size);
        });

        for input in &mut self.input_buffers {
            input.resize(block_size);
        }

        for output in &mut self.output_buffers {
            output.resize(block_size);
        }
    }

    /// Allocates all [`Node`]s' internal input and output buffers, along with various internal resources to the graph.
    ///
    /// This should be run at least once before the audio thread starts running, and again anytime the buffer size or sample rate change or the graph structure is modified.
    pub fn reset(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        let mut max_edges = 0;

        self.allocate_visitor();
        self.visit(|graph, node| {
            // allocate the node's inputs and outputs
            graph.digraph[node].reset(audio_rate, control_rate, block_size);

            let num_inputs = graph
                .digraph
                .edges_directed(node, Direction::Incoming)
                .count();
            max_edges = max_edges.max(num_inputs);
        });

        for input in &mut self.input_buffers {
            input.resize(block_size);
        }

        for output in &mut self.output_buffers {
            output.resize(block_size);
        }

        // preallocate the edge cache used in `process()`
        // the number of edges per node is likely relatively small, so we round up the cache size just to be sure that no allocations happen in `process()`
        self.edge_cache = Vec::with_capacity((max_edges * 2).next_power_of_two());

        self.needs_reset = false;
    }

    /// Prepares all [`Node`]s in the graph for processing.
    ///
    /// This should be run at least once before the audio thread starts running, and again anytime the graph structure is modified.
    pub fn prepare_nodes(&mut self) {
        self.allocate_visitor();
        self.visit(|graph, node| graph.digraph[node].prepare());

        self.needs_prepare = false;
    }

    #[inline]
    pub fn get_node_input_mut(&mut self, node: NodeIndex, input_index: usize) -> &mut Buffer {
        match &mut self.digraph[node] {
            GraphNode::Input => &mut self.input_buffers[input_index],
            GraphNode::Processor(processor) => processor.input_mut(input_index),
            GraphNode::Output => panic!("Cannot get input buffer for output node"),
        }
    }

    #[inline]
    pub fn get_node_output(&self, node: NodeIndex, output_index: usize) -> &Buffer {
        match &self.digraph[node] {
            GraphNode::Input => panic!("Cannot get output buffer for input node"),
            GraphNode::Processor(processor) => processor.output(output_index),
            GraphNode::Output => &self.output_buffers[output_index],
        }
    }

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

        self.visit(|graph, node_id| {
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
                    GraphNode::Input => {
                        let index = graph
                            .input_nodes
                            .iter()
                            .position(|&x| x == source_id)
                            .expect("Mismatch in input node indices");
                        &graph.input_buffers[index]
                    }
                    _ => panic!("Cannot get input buffer for output node"),
                };

                let target_buffer = match target {
                    GraphNode::Processor(processor) => processor.input_mut(target_input as usize),
                    GraphNode::Output => {
                        let index = graph
                            .output_nodes
                            .iter()
                            .position(|&x| x == node_id)
                            .expect("Mismatch in output node indices");
                        &mut graph.output_buffers[index]
                    }
                    _ => panic!("Cannot get output buffer for input node"),
                };
                target_buffer.copy_from_slice(source_buffer);
            }

            // process the node
            graph.digraph[node_id].process();
        });
    }
}
