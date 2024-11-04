//! Contains the [`StaticGraphBuilder`] struct for constructing audio graphs.

use std::sync::{Arc, Mutex};

use crate::{
    graph::{Graph, NodeIndex},
    prelude::Process,
    runtime::Runtime,
};

use super::static_node_builder::StaticNode;

/// A graph builder that produces [`StaticNode`]s.
///
/// These nodes have no lifetime constraints and can be used in any context.
#[derive(Clone, Default)]
pub struct StaticGraphBuilder {
    graph: Arc<Mutex<Graph>>,
}

impl StaticGraphBuilder {
    /// Creates a new graph builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an input node to the graph.
    pub fn input(&self) -> StaticNode {
        self.with_graph_mut(|graph| StaticNode {
            graph: self.clone(),
            node_id: graph.add_input(),
        })
    }

    /// Adds an output node to the graph.
    pub fn output(&self) -> StaticNode {
        self.with_graph_mut(|graph| StaticNode {
            graph: self.clone(),
            node_id: graph.add_output(),
        })
    }

    /// Adds the given processor to the graph.
    pub fn add_processor<T: Process>(&self, processor: T) -> StaticNode {
        self.with_graph_mut(|graph| StaticNode {
            graph: self.clone(),
            node_id: graph.add_processor(processor),
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
    pub fn connect(&self, from: NodeIndex, from_output: u32, to: NodeIndex, to_input: u32) {
        self.with_graph_mut(|graph| graph.connect(from, from_output, to, to_input))
            .unwrap();
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for StaticGraphBuilder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.graph.lock().unwrap().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for StaticGraphBuilder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let graph = Graph::deserialize(deserializer)?;
        Ok(Self::from_graph(graph))
    }
}
