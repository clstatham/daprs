//! Contains the [`ProcessorNode`] struct, which represents a node in the audio graph that processes signals.

use std::fmt::Debug;

use crate::{
    prelude::{Processor, ProcessorError, ProcessorInputs, ProcessorOutputs, SignalSpec},
    signal::Float,
};

/// A node in the audio graph that processes signals.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessorNode {
    pub(crate) processor: Box<dyn Processor>,
    input_spec: Vec<SignalSpec>,
    output_spec: Vec<SignalSpec>,
}

impl Debug for ProcessorNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.processor.name())
    }
}

impl ProcessorNode {
    /// Creates a new `ProcessorNode` with the given processor.
    pub fn new(processor: impl Processor) -> Self {
        Self::new_from_boxed(Box::new(processor))
    }

    /// Creates a new `ProcessorNode` with the given boxed processor.
    pub fn new_from_boxed(processor: Box<dyn Processor>) -> Self {
        let input_spec = processor.input_spec();
        let output_spec = processor.output_spec();
        Self {
            processor,
            input_spec,
            output_spec,
        }
    }

    /// Returns the name of the processor.
    #[inline]
    pub fn name(&self) -> &str {
        self.processor.name()
    }

    /// Returns information about the input signals of the processor.
    #[inline]
    pub fn input_spec(&self) -> &[SignalSpec] {
        &self.input_spec
    }

    /// Returns information about the output signals of the processor.
    #[inline]
    pub fn output_spec(&self) -> &[SignalSpec] {
        &self.output_spec
    }

    /// Returns the number of input signals of the processor.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_spec.len()
    }

    /// Returns the number of output signals of the processor.
    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.output_spec.len()
    }

    /// Resizes the internal buffers of the processor and updates the sample rate and block size.
    #[inline]
    pub fn resize_buffers(&mut self, sample_rate: Float, block_size: usize) {
        self.processor.resize_buffers(sample_rate, block_size);
    }

    /// Prepares the processor for processing.
    #[inline]
    pub fn prepare(&mut self) {
        self.processor.prepare();
    }

    /// Processes the input signals and writes the output signals to the given buffers.
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
