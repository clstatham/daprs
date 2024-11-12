//! Contains the [`GraphBuilder`] struct for constructing audio graphs.

use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use crate::{
    graph::Graph,
    prelude::{Param, Processor},
    runtime::Runtime,
    signal::SignalData,
};

use super::node_builder::{IntoInputIdx, IntoNode, IntoOutputIdx, Node};

/// A builder for constructing audio graphs.
#[derive(Clone, Default)]
pub struct GraphBuilder {
    graph: Arc<Mutex<Graph>>,
}

impl GraphBuilder {
    /// Creates a new graph builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an input node to the graph.
    pub fn add_audio_input(&self) -> Node {
        self.with_graph_mut(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_audio_input(),
        })
    }

    /// Adds an output node to the graph.
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

    /// Adds the given processor to the graph.
    pub fn add<T: Processor>(&self, processor: T) -> Node {
        self.with_graph_mut(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_processor(processor),
        })
    }

    /// Adds a [`Param`] node to the graph.
    pub fn add_param<S: SignalData>(&self, value: Param<S>) -> Node {
        self.with_graph_mut(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_param(value),
        })
    }

    /// Creates a new graph builder from the given graph.
    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
        }
    }

    /// Builds a graph from the builder.
    pub fn build(&self) -> Graph {
        self.with_graph(|graph| graph.clone())
    }

    /// Builds a runtime from the graph.
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

    /// Calls the given function with a reference to the graph.
    pub fn with_graph<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Graph) -> R,
    {
        f(&self.graph.lock().unwrap())
    }

    /// Calls the given function with a mutable reference to the graph.
    pub fn with_graph_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Graph) -> R,
    {
        f(&mut self.graph.lock().unwrap())
    }

    /// Connects the given output of the source node to the given input of the target node.
    ///
    /// # Panics
    ///
    /// Panics if the nodes, output, or input are invalid.
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

    /// Writes the graph to the given writer in the DOT format.
    /// This is useful for visualizing the graph.
    pub fn write_dot(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.with_graph(|graph| graph.write_dot(writer))
    }
}
