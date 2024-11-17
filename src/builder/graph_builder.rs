//! Contains the [`GraphBuilder`] struct for constructing audio graphs.

use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use crate::{
    graph::Graph,
    prelude::{Param, Processor},
    runtime::Runtime,
    signal::Signal,
};

use super::node_builder::{IntoInputIdx, IntoNode, IntoOutputIdx, Node};

/// A builder for constructing audio graphs.
#[derive(Clone, Default)]
pub struct GraphBuilder {
    graph: Arc<Mutex<Graph>>,
}

impl GraphBuilder {
    /// Creates a new `GraphBuilder` with an empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an audio input node to the graph.
    pub fn add_audio_input(&self) -> Node {
        self.with_graph_mut(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_audio_input(),
        })
    }

    /// Adds an audio output node to the graph.
    pub fn add_audio_output(&self) -> Node {
        self.with_graph_mut(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_audio_output(),
        })
    }

    /// Adds a MIDI input node to the graph.
    pub fn add_midi_input(&self, name: impl Into<String>) -> Node {
        self.with_graph_mut(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_midi_input(name),
        })
    }

    /// Adds a processor node to the graph.
    pub fn add(&self, processor: impl Processor) -> Node {
        self.with_graph_mut(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_processor(processor),
        })
    }

    /// Adds a parameter node to the graph.
    pub fn add_param<S: Signal + Clone>(&self, value: Param<S>) -> Node {
        self.with_graph_mut(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_param(value),
        })
    }

    /// Creates a new [`GraphBuilder`] with the given graph as a starting point.
    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
        }
    }

    /// Builds the graph, returning a new [`Graph`] instance that can be used in a [`Runtime`].
    pub fn build(&self) -> Graph {
        self.with_graph(|graph| {
            for scc in graph.sccs() {
                if scc.len() > 1 {
                    log::warn!("Strongly connected component with {} nodes", scc.len());
                    for node_id in scc {
                        let node = graph.digraph().node_weight(*node_id).unwrap();
                        log::warn!("{}:  {}", node_id.index(), node.name());
                    }
                }
            }
            graph.clone()
        })
    }

    /// Builds the graph and constructs a new [`Runtime`] instance from the graph.
    pub fn build_runtime(&self) -> Runtime {
        Runtime::new(self.build())
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.with_graph(|graph| graph.digraph().node_count())
    }

    /// Returns the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.with_graph(|graph| graph.digraph().edge_count())
    }

    /// Runs the given closure with a reference to the graph.
    pub fn with_graph<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Graph) -> R,
    {
        f(&self.graph.lock().unwrap())
    }

    /// Runs the given closure with a mutable reference to the graph.
    pub fn with_graph_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Graph) -> R,
    {
        f(&mut self.graph.lock().unwrap())
    }

    /// Connects the given output of one node to the given input of another node.
    #[track_caller]
    #[inline]
    pub fn connect(
        &self,
        from: impl IntoNode,
        from_output: impl IntoOutputIdx,
        to: impl IntoNode,
        to_input: impl IntoInputIdx,
    ) {
        let from = from.into_node(self);
        let to = to.into_node(self);
        let from_output = from_output.into_output_idx(&from);
        let to_input = to_input.into_input_idx(&to);
        self.with_graph_mut(|graph| graph.connect(from.id(), from_output, to.id(), to_input))
            .unwrap();
    }

    /// Writes a DOT representation of the graph to the given writer.
    pub fn write_dot(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.with_graph(|graph| graph.write_dot(writer))
    }
}
