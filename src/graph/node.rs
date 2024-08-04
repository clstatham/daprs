use std::fmt::Debug;

use crate::{
    processor::{Param, Process, Processor},
    signal::Buffer,
};

/// A node in the audio graph.
#[derive(Clone)]
pub enum GraphNode {
    /// A passthrough node that simply forwards its input to its output.
    Passthrough(Buffer),
    /// A processor node that processes its input buffers and writes the results to its output buffers.
    Processor(Processor),
}

impl Debug for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passthrough(_) => f.write_str("Passthrough"),
            Self::Processor(processor) => Debug::fmt(processor, f),
        }
    }
}

impl GraphNode {
    /// Creates a new input node.
    pub fn new_input() -> Self {
        Self::Passthrough(Buffer::zeros(0))
    }

    /// Creates a new processor node from the given [`Processor`] object.
    pub fn new_processor_object(processor: Processor) -> Self {
        Self::Processor(processor)
    }

    /// Creates a new processor node from the given [`Process`] object.
    pub fn new_processor(processor: impl Process) -> Self {
        Self::Processor(Processor::new(processor))
    }

    /// Creates a new output node.
    pub fn new_output() -> Self {
        Self::Passthrough(Buffer::zeros(0))
    }

    /// Returns information about the inputs this [`GraphNode`] expects.
    pub fn input_spec(&self) -> Vec<Param> {
        match self {
            Self::Passthrough(_) => vec![Param::default_with_name("in")],
            Self::Processor(processor) => processor.input_params(),
        }
    }

    /// Returns information about the outputs this [`GraphNode`] produces.
    pub fn output_spec(&self) -> Vec<Param> {
        match self {
            Self::Passthrough(_) => vec![Param::default_with_name("out")],
            Self::Processor(processor) => processor.output_params(),
        }
    }

    /// Returns the name of the processor in this [`GraphNode`].
    pub fn name(&self) -> &str {
        match self {
            Self::Passthrough(_) => "Passthrough",
            Self::Processor(processor) => processor.name(),
        }
    }

    /// Returns a slice of the input buffers of this [`GraphNode`].
    pub fn inputs(&self) -> &[Buffer] {
        match self {
            Self::Passthrough(buffer) => std::slice::from_ref(buffer),
            Self::Processor(processor) => processor.inputs(),
        }
    }

    /// Returns a mutable slice of the input buffers of this [`GraphNode`].
    pub fn inputs_mut(&mut self) -> &mut [Buffer] {
        match self {
            Self::Passthrough(buffer) => std::slice::from_mut(buffer),
            Self::Processor(processor) => processor.inputs_mut(),
        }
    }

    /// Returns a slice of the output buffers of this [`GraphNode`].
    pub fn outputs(&self) -> &[Buffer] {
        match self {
            Self::Passthrough(buffer) => std::slice::from_ref(buffer),
            Self::Processor(processor) => processor.outputs(),
        }
    }

    /// Resizes the input and output buffers to match the given sample rate and block size.
    pub fn resize_buffers(&mut self, sample_rate: f64, block_size: usize) {
        match self {
            Self::Passthrough(buffer) => buffer.resize(block_size),
            Self::Processor(processor) => processor.resize_buffers(sample_rate, block_size),
        }
    }

    /// Prepares the processor for processing. This is called before the first [`GraphNode::process`] call, and anytime the graph changes.
    pub fn prepare(&mut self) {
        if let Self::Processor(processor) = self {
            processor.prepare();
        }
    }

    /// Processes the node's input buffers and writes the results to the node's output buffers.
    /// This is a no-op for passthrough nodes.
    pub fn process(&mut self) {
        if let Self::Processor(processor) = self {
            processor.process();
        }
    }
}
