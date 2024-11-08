//! Audio processing utilities and types.

use std::fmt::Debug;

use downcast_rs::{impl_downcast, DowncastSync};
use thiserror::Error;

use crate::{
    message::Message,
    signal::{Sample, Signal, SignalBuffer, SignalKind},
};

/// An error that can occur when processing signals.
#[derive(Debug, Clone, Error)]
pub enum ProcessorError {
    /// The number of inputs must match the number returned by [`Process::num_inputs`].
    #[error("The number of inputs must match the number returned by Process::num_inputs()")]
    NumInputsMismatch,

    /// The number of outputs must match the number returned by [`Process::num_outputs`].
    #[error("The number of outputs must match the number returned by Process::num_outputs()")]
    NumOutputsMismatch,

    /// The input signal type at the given index does not match the expected type.
    #[error("Input {0} signal type mismatch")]
    InputSpecMismatch(usize),

    /// The output signal type at the given index does not match the expected type.
    #[error("Output {0} signal type mismatch")]
    OutputSpecMismatch(usize),

    /// The signal value is invalid for the given reason.
    #[error("Invalid value: {0}")]
    InvalidValue(&'static str),
}

/// Information about an input/output of a [`Process`] implementor.
#[derive(Debug, Clone)]
pub struct SignalSpec {
    /// The name of the signal.
    pub name: String,
    /// The minimum value of the signal, if any.
    pub min: Option<Signal>,
    /// The maximum value of the signal, if any.
    pub max: Option<Signal>,
    /// The default value of the signal.
    pub default_value: Signal,
}

impl Default for SignalSpec {
    /// Creates a new unnamed and unbounded [`SignalSpec`] (min/max are `None`, default value is `0.0`).
    fn default() -> Self {
        Self {
            name: "".into(),
            min: None,
            max: None,
            default_value: Signal::Sample(0.0.into()),
        }
    }
}

impl SignalSpec {
    /// Creates a new bounded [`SignalSpec`] with the given name, minimum, maximum, and default value.
    pub fn new(
        name: impl Into<String>,
        min: Option<impl Into<Signal>>,
        max: Option<impl Into<Signal>>,
        default_value: impl Into<Signal>,
    ) -> Self {
        Self {
            name: name.into(),
            min: min.map(Into::into),
            max: max.map(Into::into),
            default_value: default_value.into(),
        }
    }

    /// Creates a new unbounded [`SignalSpec`] with the given name and default value.
    pub fn unbounded(name: impl Into<String>, default_value: impl Into<Signal>) -> Self {
        Self {
            name: name.into(),
            min: None,
            max: None,
            default_value: default_value.into(),
        }
    }

    /// Returns the type of signal this [`SignalSpec`] represents.
    pub fn kind(&self) -> SignalKind {
        self.default_value.kind()
    }
}

/// A collection of borrowed input buffers for a [`Process`] to read from, and related information about them.
#[derive(Debug, Clone, Copy)]
pub struct ProcessInputs<'a, 'b> {
    /// The input signal specs.
    pub input_spec: &'a [SignalSpec],
    /// The default values for the input signals.
    pub input_spec_defaults: &'a [Signal],
    /// The input buffers to read from.
    pub inputs: &'a [Option<&'b SignalBuffer>],
}

impl<'a, 'b> ProcessInputs<'a, 'b> {
    /// Returns the number of input buffers.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.inputs.len()
    }

    /// Returns the input buffer at the given index, if any.
    #[inline]
    pub fn input(&self, index: usize) -> Option<&'b SignalBuffer> {
        self.inputs.get(index).copied().flatten()
    }

    /// Returns an iterator over the input buffers.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = Option<&'b SignalBuffer>> + '_ {
        self.inputs.iter().copied()
    }

    /// Returns an iterator over the input signals at the given index.
    #[inline]
    pub fn iter_input_as_samples(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = &Sample> + '_, ProcessorError> {
        let buffer = self.input(index);

        if let Some(buffer) = buffer {
            let buffer = buffer
                .as_sample()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;

            Ok(itertools::Either::Left(buffer.iter()))
        } else {
            let default_value = self.input_spec_defaults[index]
                .as_sample()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;
            Ok(itertools::Either::Right(std::iter::repeat(default_value)))
        }
    }

    /// Returns an iterator over the input messages at the given index.
    #[inline]
    pub fn iter_input_as_messages(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = &Option<Message>> + '_, ProcessorError> {
        let buffer = self.input(index);

        if let Some(buffer) = buffer {
            let buffer = buffer
                .as_message()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;

            Ok(itertools::Either::Left(buffer.iter()))
        } else {
            let default_value = self.input_spec_defaults[index]
                .as_message()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;
            Ok(itertools::Either::Right(std::iter::repeat(default_value)))
        }
    }
}

/// A collection of borrowed output buffers for a [`Process`] to write to, and related information about them.
pub struct ProcessOutputs<'a> {
    /// The output signal specs.
    pub output_spec: &'a [SignalSpec],
    /// The output buffers to write to.
    pub outputs: &'a mut [SignalBuffer],
}

impl<'a> ProcessOutputs<'a> {
    /// Returns the output buffer at the given index.
    #[inline]
    pub fn output(&mut self, index: usize) -> &mut SignalBuffer {
        &mut self.outputs[index]
    }

    /// Returns an iterator over the output buffers.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SignalBuffer> + '_ {
        self.outputs.iter_mut()
    }

    /// Returns an iterator over the output samples at the given index.
    #[inline]
    pub fn iter_output_mut_as_samples(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Sample> + '_, ProcessorError> {
        let buffer = self
            .output(index)
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(index))?;

        Ok(buffer.iter_mut())
    }

    /// Returns an iterator over the output messages at the given index.
    #[inline]
    pub fn iter_output_mut_as_messages(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<Message>> + '_, ProcessorError> {
        let buffer = self
            .output(index)
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(index))?;

        Ok(buffer.iter_mut())
    }

    /// Splits this [`ProcessOutputs`] into two at the given index.
    ///
    /// Note that the indices in the returned [`ProcessOutputs`] are relative to the split point, not the original outputs.
    ///
    /// # Example
    ///
    /// ```
    /// use raug::processor::{ProcessOutputs, SignalSpec};
    /// use raug::signal::{Signal, SignalBuffer};
    ///
    /// // Create some output buffers and their specs.
    /// let mut outputs = vec![
    ///     SignalBuffer::new_sample(1),
    ///     SignalBuffer::new_sample(2),
    ///     SignalBuffer::new_sample(3),
    /// ];
    ///
    /// let output_spec = vec![
    ///     SignalSpec::unbounded("a", 0.0),
    ///     SignalSpec::unbounded("b", 0.0),
    ///     SignalSpec::unbounded("c", 0.0),
    /// ];
    ///
    /// let mut process_outputs = ProcessOutputs {
    ///     output_spec: &output_spec,
    ///     outputs: &mut outputs,
    /// };
    ///
    /// // Split the outputs at index 1.
    /// let (mut left, mut right) = process_outputs.split_at_mut(1);
    ///
    /// assert_eq!(left.output_spec.len(), 1);
    /// assert_eq!(right.output_spec.len(), 2);
    ///
    /// // The first output buffer is now in `left` at index 0.
    /// assert_eq!(left.output(0).len(), 1);
    ///
    /// // The second and third output buffers are now in `right` at indices 0 and 1.
    /// assert_eq!(right.output(0).len(), 2);
    /// assert_eq!(right.output(1).len(), 3);
    /// ```
    #[inline]
    pub fn split_at_mut(&mut self, index: usize) -> (ProcessOutputs, ProcessOutputs) {
        let (left, right) = self.outputs.split_at_mut(index);
        (
            ProcessOutputs {
                output_spec: &self.output_spec[..index],
                outputs: left,
            },
            ProcessOutputs {
                output_spec: &self.output_spec[index..],
                outputs: right,
            },
        )
    }
}

/// A trait for processing audio or control signals.
pub trait Process: 'static + Send + Sync + ProcessClone + DowncastSync {
    /// Returns the name of this [`Process`].
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Returns information about the inputs this [`Process`] expects.
    fn input_spec(&self) -> Vec<SignalSpec>;

    /// Returns information about the outputs this [`Process`] produces.
    fn output_spec(&self) -> Vec<SignalSpec>;

    /// Allocates input buffers for this processor with the given length and fills them with their default values.
    fn make_default_input_buffers(&self, length: usize) -> Vec<SignalBuffer> {
        let input_spec = self.input_spec();
        let mut buffers = Vec::with_capacity(input_spec.len());
        for spec in input_spec {
            buffers.push(SignalBuffer::from_spec_default(&spec, length));
        }
        buffers
    }

    /// Allocates output buffers for this processor with the given length and fills them with their default values.
    fn make_default_output_buffers(&self, length: usize) -> Vec<SignalBuffer> {
        let output_spec = self.output_spec();
        let mut buffers = Vec::with_capacity(output_spec.len());
        for spec in output_spec {
            buffers.push(SignalBuffer::from_spec_default(&spec, length));
        }
        buffers
    }

    /// Returns the number of input buffers/channels this [`Process`] expects.
    fn num_inputs(&self) -> usize {
        self.input_spec().len()
    }

    /// Returns the number of output buffers/channels this [`Process`] produces.
    fn num_outputs(&self) -> usize {
        self.output_spec().len()
    }

    /// Called before the first [`Process::process`] call, and anytime the graph changes.
    fn prepare(&mut self) {}

    /// Called whenever the runtime's sample rates or block size change.
    #[allow(unused)]
    fn resize_buffers(&mut self, sample_rate: f64, block_size: usize) {}

    /// Processes the given input buffers and writes the results to the given output buffers.
    ///
    /// The number of input and output buffers must match the numbers returned by [`Process::num_inputs`] and [`Process::num_outputs`].
    fn process(
        &mut self,
        inputs: ProcessInputs,
        outputs: ProcessOutputs,
    ) -> Result<(), ProcessorError>;

    /// Clones this [`Process`] into a [`Processor`] object that can be used in the audio graph.
    fn processor(&self) -> Processor {
        Processor::new_from_boxed(self.clone_boxed())
    }
}
impl_downcast!(sync Process);

mod sealed {
    pub trait Sealed {}
    impl<T: Clone> Sealed for T {}
}

#[doc(hidden)]
pub trait ProcessClone: sealed::Sealed {
    fn clone_boxed(&self) -> Box<dyn Process>;
}

impl<T> ProcessClone for T
where
    T: Clone + Process,
{
    fn clone_boxed(&self) -> Box<dyn Process> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Process> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

impl Debug for dyn Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// A node in the audio graph that processes signals.
#[derive(Clone)]
pub struct Processor {
    pub(crate) processor: Box<dyn Process>,
    input_spec: Vec<SignalSpec>,
    output_spec: Vec<SignalSpec>,
}

impl Debug for Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.processor.name())
    }
}

impl Processor {
    /// Creates a new [`Processor`] from the given [`Process`] object.
    pub fn new(processor: impl Process) -> Self {
        Self::new_from_boxed(Box::new(processor))
    }

    /// Creates a new [`Processor`] from the given boxed [`Process`] object.
    pub fn new_from_boxed(processor: Box<dyn Process>) -> Self {
        let input_spec = processor.input_spec();
        let output_spec = processor.output_spec();
        Self {
            processor,
            input_spec,
            output_spec,
        }
    }

    /// Returns the name of this [`Processor`].
    #[inline]
    pub fn name(&self) -> &str {
        self.processor.name()
    }

    /// Returns information about the inputs this [`Processor`] expects.
    #[inline]
    pub fn input_spec(&self) -> &[SignalSpec] {
        &self.input_spec
    }

    /// Returns information about the outputs this [`Processor`] produces.
    #[inline]
    pub fn output_spec(&self) -> &[SignalSpec] {
        &self.output_spec
    }

    /// Returns the number of input buffers/channels this [`Processor`] expects.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_spec.len()
    }

    /// Returns the number of output buffers/channels this [`Processor`] produces.
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
        inputs: ProcessInputs,
        outputs: ProcessOutputs,
    ) -> Result<(), ProcessorError> {
        self.processor.process(inputs, outputs)?;
        Ok(())
    }
}
