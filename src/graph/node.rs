//! Contains the [`GraphNode`] type, which represents a node in the audio graph.

use std::fmt::Debug;

use crate::processor::{
    Process, ProcessInputs, ProcessOutputs, Processor, ProcessorError, SignalSpec,
};

/// A node in the audio graph.
#[derive(Clone)]

pub enum GraphNode {
    /// A passthrough node that simply forwards its input to its output.
    Passthrough,
    /// A processor node that processes its input buffers and writes the results to its output buffers.
    Processor(Processor),
}

impl Debug for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passthrough => f.write_str("Passthrough"),
            Self::Processor(processor) => Debug::fmt(processor, f),
        }
    }
}

impl GraphNode {
    /// Creates a new input node.
    pub fn new_input() -> Self {
        Self::Passthrough
    }

    /// Creates a new processor node from the given [`Processor`] object.
    pub fn new_processor_object(processor: Processor) -> Self {
        Self::Processor(processor)
    }

    /// Creates a new processor node from the given [`Process`] implementor.
    pub fn new_processor(processor: impl Process) -> Self {
        Self::Processor(Processor::new(processor))
    }

    /// Creates a new output node.
    pub fn new_output() -> Self {
        Self::Passthrough
    }

    /// Returns information about the inputs this [`GraphNode`] expects.
    pub fn input_spec(&self) -> Vec<SignalSpec> {
        match self {
            Self::Passthrough => vec![SignalSpec::unbounded("in", 0.0)],
            Self::Processor(processor) => processor.input_spec(),
        }
    }

    /// Returns information about the outputs this [`GraphNode`] produces.
    pub fn output_spec(&self) -> Vec<SignalSpec> {
        match self {
            Self::Passthrough => vec![SignalSpec::unbounded("out", 0.0)],
            Self::Processor(processor) => processor.output_spec(),
        }
    }

    /// Returns the name of the processor in this [`GraphNode`].
    pub fn name(&self) -> &str {
        match self {
            Self::Passthrough => "Passthrough",
            Self::Processor(processor) => processor.name(),
        }
    }

    /// Resizes the input and output buffers to match the given sample rate and block size.
    pub fn resize_buffers(&mut self, sample_rate: f64, block_size: usize) {
        match self {
            Self::Passthrough => {}
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
    pub fn process(
        &mut self,
        inputs: ProcessInputs,
        mut outputs: ProcessOutputs,
    ) -> Result<(), ProcessorError> {
        if let Self::Processor(processor) = self {
            processor.process(inputs, outputs)?;
        } else {
            // Passthrough
            let input = inputs.input(0).unwrap().as_sample().unwrap();
            let output = outputs.output(0).as_sample_mut().unwrap();
            output.copy_from_slice(input);
        }

        Ok(())
    }
}
