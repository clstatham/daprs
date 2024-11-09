//! Contains the [`BuiltNode`] struct, which represents a node in the audio graph that processes signals.

use std::fmt::Debug;

use crate::prelude::{Processor, ProcessorError, ProcessorInputs, ProcessorOutputs, SignalSpec};

/// A node in the audio graph that processes signals.
#[derive(Clone)]
pub struct BuiltNode {
    pub(crate) processor: Box<dyn Processor>,
    input_spec: Vec<SignalSpec>,
    output_spec: Vec<SignalSpec>,
}

impl Debug for BuiltNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.processor.name())
    }
}

impl BuiltNode {
    /// Creates a new [`BuiltNode`] from the given [`Processor`] object.
    pub fn new(processor: impl Processor) -> Self {
        Self::new_from_boxed(Box::new(processor))
    }

    /// Creates a new [`BuiltNode`] from the given boxed [`Processor`] object.
    pub fn new_from_boxed(processor: Box<dyn Processor>) -> Self {
        let input_spec = processor.input_spec();
        let output_spec = processor.output_spec();
        Self {
            processor,
            input_spec,
            output_spec,
        }
    }

    /// Returns the name of this [`BuiltNode`].
    #[inline]
    pub fn name(&self) -> &str {
        self.processor.name()
    }

    /// Returns information about the inputs this [`BuiltNode`] expects.
    #[inline]
    pub fn input_spec(&self) -> &[SignalSpec] {
        &self.input_spec
    }

    /// Returns information about the outputs this [`BuiltNode`] produces.
    #[inline]
    pub fn output_spec(&self) -> &[SignalSpec] {
        &self.output_spec
    }

    /// Returns the number of input buffers/channels this [`BuiltNode`] expects.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_spec.len()
    }

    /// Returns the number of output buffers/channels this [`BuiltNode`] produces.
    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.output_spec.len()
    }

    /// Resizes the input and output buffers to match the given sample rates and block size.
    pub fn resize_buffers(&mut self, sample_rate: f64, block_size: usize) {
        self.processor.resize_buffers(sample_rate, block_size);
    }

    /// Prepares the processor for processing. This is called before the first [`Processor::process`] call, and anytime the graph changes.
    #[inline]
    pub fn prepare(&mut self) {
        self.processor.prepare();
    }

    /// Processes the input buffers and writes the results to the output buffers.
    #[inline]
    pub fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        self.processor.process(inputs, outputs)?;
        Ok(())
    }
}
