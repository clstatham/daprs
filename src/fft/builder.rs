use std::{
    ops::Mul,
    sync::{Arc, Mutex},
};

use petgraph::prelude::*;

use crate::prelude::*;

#[derive(Clone)]
pub struct FftGraphBuilder {
    graph: Arc<Mutex<FftGraph>>,
}

impl Default for FftGraphBuilder {
    fn default() -> Self {
        Self::new(128, 64, WindowFunction::Hann)
    }
}

impl FftGraphBuilder {
    pub fn new(fft_length: usize, hop_length: usize, window_function: WindowFunction) -> Self {
        Self {
            graph: Arc::new(Mutex::new(FftGraph::new(
                fft_length,
                hop_length,
                window_function,
            ))),
        }
    }

    pub fn from_graph(graph: FftGraph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
        }
    }

    pub fn with_graph<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut FftGraph) -> R,
    {
        let mut graph = self.graph.lock().unwrap();
        f(&mut graph)
    }

    pub fn build(self) -> FftGraph {
        self.with_graph(|graph| graph.clone())
    }

    pub fn add_input(&self) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add_input());
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    pub fn add_output(&self) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add_output());
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    pub fn add(&self, processor: impl FftProcessor) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add(processor));
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    pub fn connect(
        &self,
        source: &FftNode,
        source_output: usize,
        target: &FftNode,
        target_input: usize,
    ) {
        self.with_graph(|graph| {
            graph.connect(source.id(), source_output, target.id(), target_input)
        });
    }
}

#[derive(Clone)]
pub struct FftNode {
    node_id: NodeIndex,
    graph: FftGraphBuilder,
}

impl FftNode {
    pub fn id(&self) -> NodeIndex {
        self.node_id
    }

    pub fn graph(&self) -> FftGraphBuilder {
        self.graph.clone()
    }

    pub fn input(&self, index: usize) -> FftInput {
        FftInput {
            node: self.clone(),
            index,
        }
    }

    pub fn output(&self, index: usize) -> FftOutput {
        FftOutput {
            node: self.clone(),
            index,
        }
    }
}

pub struct FftInput {
    pub node: FftNode,
    pub index: usize,
}

impl FftInput {
    pub fn node(&self) -> FftNode {
        self.node.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn connect(&self, output: FftOutput) {
        self.node.graph.with_graph(|graph| {
            graph.connect(output.node.id(), output.index, self.node.id(), self.index)
        });
    }
}

pub struct FftOutput {
    pub node: FftNode,
    pub index: usize,
}

impl FftOutput {
    pub fn node(&self) -> FftNode {
        self.node.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn connect(&self, input: FftInput) {
        self.node.graph.with_graph(|graph| {
            graph.connect(self.node.id(), self.index, input.node.id(), input.index)
        });
    }
}

impl Mul for FftNode {
    type Output = FftNode;

    fn mul(self, rhs: FftNode) -> Self::Output {
        let node = self.graph.add(FftConvolve);
        node.input(0).connect(self.output(0));
        node.input(1).connect(rhs.output(0));
        node
    }
}

impl Mul for &FftNode {
    type Output = FftNode;

    fn mul(self, rhs: &FftNode) -> Self::Output {
        let node = self.graph.add(FftConvolve);
        node.input(0).connect(self.output(0));
        node.input(1).connect(rhs.output(0));
        node
    }
}
