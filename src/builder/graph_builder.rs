use std::sync::Mutex;

use crate::{
    graph::{Graph, NodeIndex},
    prelude::Process,
    runtime::Runtime,
};

use super::node_builder::Node;

pub struct GraphBuilder {
    graph: Mutex<Graph>,
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self {
            graph: Mutex::new(Graph::new()),
        }
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph: Mutex::new(graph),
        }
    }

    pub fn build(self) -> Graph {
        Mutex::into_inner(self.graph).unwrap()
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

    pub fn input(&self) -> Node<'_> {
        let index = self.with_graph_mut(|graph| graph.add_input());
        Node {
            graph_builder: self,
            node_id: index,
        }
    }

    pub fn output(&self) -> Node<'_> {
        let index = self.with_graph_mut(|graph| graph.add_output());
        Node {
            graph_builder: self,
            node_id: index,
        }
    }

    pub fn add_processor(&self, processor: impl Process) -> Node<'_> {
        let index = self.with_graph_mut(|graph| graph.add_processor(processor));
        Node {
            graph_builder: self,
            node_id: index,
        }
    }
}
