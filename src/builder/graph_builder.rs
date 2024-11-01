use std::sync::Mutex;

use crate::{
    graph::{Graph, NodeIndex},
    prelude::Process,
};

use super::node_builder::Node;

pub struct GraphBuilder {
    graph: Mutex<Option<Graph>>,
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self {
            graph: Mutex::new(Some(Graph::new())),
        }
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph: Mutex::new(Some(graph)),
        }
    }

    pub fn build(&self) -> Graph {
        self.graph.lock().unwrap().take().unwrap()
    }

    pub fn with_graph<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Graph) -> R,
    {
        f(self.graph.lock().unwrap().as_ref().unwrap())
    }

    pub fn with_graph_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Graph) -> R,
    {
        f(self.graph.lock().unwrap().as_mut().unwrap())
    }

    pub fn connect(
        &self,
        source: NodeIndex,
        source_output: u32,
        target: NodeIndex,
        target_input: u32,
    ) -> &Self {
        self.graph
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .connect(source, source_output, target, target_input)
            .unwrap();
        self
    }

    pub fn input(&self) -> Node<'_> {
        let index = self.graph.lock().unwrap().as_mut().unwrap().add_input();
        Node {
            graph_builder: self,
            node_id: index,
        }
    }

    pub fn output(&self) -> Node<'_> {
        let index = self.graph.lock().unwrap().as_mut().unwrap().add_output();
        Node {
            graph_builder: self,
            node_id: index,
        }
    }

    pub fn add_processor(&self, processor: impl Process) -> Node<'_> {
        let index = self
            .graph
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .add_processor(processor);
        Node {
            graph_builder: self,
            node_id: index,
        }
    }
}
