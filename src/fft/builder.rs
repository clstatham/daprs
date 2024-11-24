//! Builder API for constructing [`FftGraph`]s.

use std::{
    ops::Mul,
    sync::{Arc, Mutex},
};

use petgraph::prelude::*;

use crate::prelude::*;

/// A builder API version of an [`FftGraph`].
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
    /// Creates a new `FftGraphBuilder` with the given parameters.
    pub fn new(fft_length: usize, hop_length: usize, window_function: WindowFunction) -> Self {
        Self {
            graph: Arc::new(Mutex::new(FftGraph::new(
                fft_length,
                hop_length,
                window_function,
            ))),
        }
    }

    /// Creates a new `FftGraphBuilder` from an existing [`FftGraph`].
    pub fn from_graph(graph: FftGraph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
        }
    }

    /// Executes the given closure with a mutable reference to the underlying [`FftGraph`].
    pub fn with_graph<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut FftGraph) -> R,
    {
        let mut graph = self.graph.lock().unwrap();
        f(&mut graph)
    }

    /// Returns a clone of the underlying [`FftGraph`] as it currently exists.
    pub fn build(&self) -> FftGraph {
        self.with_graph(|graph| graph.clone())
    }

    /// Adds an input node to the graph.
    pub fn add_input(&self) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add_input());
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    /// Adds an output node to the graph.
    pub fn add_output(&self) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add_output());
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    /// Adds a processor node to the graph.
    pub fn add(&self, processor: impl FftProcessor) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add(processor));
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    /// Connects the given output of one node to the given input of another node.
    ///
    /// If a connection already exists at the given input index, it will be replaced.
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

/// A node in an [`FftGraphBuilder`].
#[derive(Clone)]
pub struct FftNode {
    node_id: NodeIndex,
    graph: FftGraphBuilder,
}

impl FftNode {
    /// Returns the ID of the node.
    pub fn id(&self) -> NodeIndex {
        self.node_id
    }

    /// Returns the [`FftGraphBuilder`] that the node belongs to.
    pub fn graph(&self) -> FftGraphBuilder {
        self.graph.clone()
    }

    /// Returns an [`FftInput`] for the given index, allowing further operations on that input.
    pub fn input(&self, index: usize) -> FftInput {
        FftInput {
            node: self.clone(),
            index,
        }
    }

    /// Returns an [`FftOutput`] for the given index, allowing further operations on that output.
    pub fn output(&self, index: usize) -> FftOutput {
        FftOutput {
            node: self.clone(),
            index,
        }
    }
}

/// An input to an [`FftNode`].
pub struct FftInput {
    node: FftNode,
    index: usize,
}

impl FftInput {
    /// Returns the [`FftNode`] that the input belongs to.
    pub fn node(&self) -> FftNode {
        self.node.clone()
    }

    /// Returns the index of the input.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Connects the input to the given output.
    pub fn connect(&self, output: FftOutput) {
        self.node.graph.with_graph(|graph| {
            graph.connect(output.node.id(), output.index, self.node.id(), self.index)
        });
    }
}

/// An output of an [`FftNode`].
pub struct FftOutput {
    node: FftNode,
    index: usize,
}

impl FftOutput {
    /// Returns the [`FftNode`] that the output belongs to.
    pub fn node(&self) -> FftNode {
        self.node.clone()
    }

    /// Returns the index of the output.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Connects the output to the given input.
    pub fn connect(&self, input: FftInput) {
        self.node.graph.with_graph(|graph| {
            graph.connect(self.node.id(), self.index, input.node.id(), input.index)
        });
    }
}

impl Mul for FftNode {
    type Output = FftNode;

    /// Performs a frequency-domain convolution between two nodes.
    fn mul(self, rhs: FftNode) -> Self::Output {
        let node = self.graph.add(FftConvolve);
        node.input(0).connect(self.output(0));
        node.input(1).connect(rhs.output(0));
        node
    }
}

impl Mul for &FftNode {
    type Output = FftNode;

    /// Performs a frequency-domain convolution between two nodes.
    fn mul(self, rhs: &FftNode) -> Self::Output {
        let node = self.graph.add(FftConvolve);
        node.input(0).connect(self.output(0));
        node.input(1).connect(rhs.output(0));
        node
    }
}
