//! Contains the `GraphBuilder` type for constructing audio graphs.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::{
    graph::{Graph, NodeIndex},
    prelude::Process,
    runtime::Runtime,
};

use super::node_builder::Node;

/// A builder for constructing audio graphs.
#[derive(Serialize, Deserialize)]
pub struct GraphBuilder {
    graph: Mutex<Graph>,
}

impl Clone for GraphBuilder {
    fn clone(&self) -> Self {
        Self::from_graph(self.with_graph(|graph| graph.clone()))
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self {
            graph: Mutex::new(Graph::new()),
        }
    }
}

impl GraphBuilder {
    /// Creates a new graph builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new graph builder from the given graph.
    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph: Mutex::new(graph),
        }
    }

    /// Builds the graph.
    pub fn build(self) -> Graph {
        Mutex::into_inner(self.graph).unwrap()
    }

    /// Builds a runtime from the graph.
    pub fn build_runtime(self) -> Runtime {
        Runtime::new(self.build())
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

    /// Connects two nodes in the graph.
    pub fn connect(
        &self,
        source: NodeIndex,
        source_output: u32,
        target: NodeIndex,
        target_input: u32,
    ) -> &Self {
        self.with_graph_mut(|graph| graph.connect(source, source_output, target, target_input))
            .expect("failed to connect nodes");
        self
    }

    /// Adds an input to the graph.
    pub fn input(&self) -> Node<'_> {
        let index = self.with_graph_mut(|graph| graph.add_input());
        Node {
            graph_builder: self,
            node_id: index,
        }
    }

    /// Adds an output to the graph.
    pub fn output(&self) -> Node<'_> {
        let index = self.with_graph_mut(|graph| graph.add_output());
        Node {
            graph_builder: self,
            node_id: index,
        }
    }

    /// Adds a processor to the graph.
    pub fn add_processor(&self, processor: impl Process) -> Node<'_> {
        let index = self.with_graph_mut(|graph| graph.add_processor(processor));
        Node {
            graph_builder: self,
            node_id: index,
        }
    }
}
