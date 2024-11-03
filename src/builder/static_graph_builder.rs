use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::{
    graph::{Graph, NodeIndex},
    prelude::Process,
    runtime::Runtime,
};

use super::static_node_builder::StaticNode;

#[derive(Clone, Default)]
pub struct StaticGraphBuilder {
    graph: Arc<Mutex<Graph>>,
}

impl StaticGraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn input(&self) -> StaticNode {
        self.with_graph_mut(|graph| StaticNode {
            graph: self.clone(),
            node_id: graph.add_input(),
        })
    }

    pub fn output(&self) -> StaticNode {
        self.with_graph_mut(|graph| StaticNode {
            graph: self.clone(),
            node_id: graph.add_output(),
        })
    }

    pub fn add_processor<T: Process>(&self, processor: T) -> StaticNode {
        self.with_graph_mut(|graph| StaticNode {
            graph: self.clone(),
            node_id: graph.add_processor(processor),
        })
    }

    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
        }
    }

    pub fn build(self) -> Graph {
        self.graph.lock().unwrap().clone()
    }

    pub fn build_runtime(self) -> Runtime {
        Runtime::new(self.build())
    }

    pub fn with_graph<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Graph) -> R,
    {
        f(&self.graph.lock().unwrap())
    }

    pub fn with_graph_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Graph) -> R,
    {
        f(&mut self.graph.lock().unwrap())
    }

    #[track_caller]
    #[inline]
    pub fn connect(&self, from: NodeIndex, from_output: u32, to: NodeIndex, to_input: u32) {
        self.with_graph_mut(|graph| graph.connect(from, from_output, to, to_input))
            .unwrap();
    }
}

impl Serialize for StaticGraphBuilder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.graph.lock().unwrap().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StaticGraphBuilder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let graph = Graph::deserialize(deserializer)?;
        Ok(Self::from_graph(graph))
    }
}
