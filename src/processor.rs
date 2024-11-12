//! Audio processing utilities and types.

use std::fmt::Debug;

use downcast_rs::{impl_downcast, DowncastSync};
use thiserror::Error;

use crate::signal::{Buffer, Sample, Signal, SignalBuffer, SignalData, SignalKind};

/// An error that can occur when processing signals.
#[derive(Debug, Clone, Error)]
pub enum ProcessorError {
    /// The number of inputs must match the number returned by [`Processor::num_inputs`].
    #[error("The number of inputs must match the number returned by Processor::num_inputs()")]
    NumInputsMismatch,

    /// The number of outputs must match the number returned by [`Processor::num_outputs`].
    #[error("The number of outputs must match the number returned by Processor::num_outputs()")]
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

/// Information about an output of a [`Processor`] implementor.
#[derive(Debug, Clone)]
pub struct OutputSpec {
    /// The name of the signal.
    pub name: String,
    /// The type of the signal.
    pub kind: SignalKind,
}

impl Default for OutputSpec {
    /// Creates a new unnamed [`Sample`] [`OutputSpec`].
    fn default() -> Self {
        Self {
            name: "".into(),
            kind: SignalKind::Sample,
        }
    }
}

impl OutputSpec {
    /// Creates a new bounded [`OutputSpec`] with the given name, minimum, maximum, and default value.
    pub fn new(name: impl Into<String>, kind: SignalKind) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }
}

/// A collection of borrowed input buffers for a [`Processor`] to read from, and related information about them.
#[derive(Debug, Clone, Copy)]
pub struct ProcessorInputs<'a, 'b> {
    /// The input signal names.
    pub input_names: &'a [String],
    /// The input buffers to read from.
    pub inputs: &'a [Option<&'b SignalBuffer>],
}

impl<'a, 'b> ProcessorInputs<'a, 'b> {
    /// Returns the number of input buffers.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_names.len()
    }

    /// Returns the input buffer at the given index, if any.
    #[inline]
    pub fn input(&self, index: usize) -> Option<&'b SignalBuffer> {
        self.inputs.get(index).copied().flatten()
    }

    #[inline]
    pub fn input_as<S: SignalData>(&self, index: usize) -> Result<&'b Buffer<S>, ProcessorError> {
        self.input(index)
            .ok_or(ProcessorError::InputSpecMismatch(index))?
            .as_kind::<S>()
            .ok_or(ProcessorError::InputSpecMismatch(index))
    }

    /// Returns an iterator over the input buffers.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = Option<&'b SignalBuffer>> + '_ {
        self.inputs.iter().copied()
    }

    #[inline]
    pub fn iter_input_as<S: SignalData>(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = &Option<S>> + '_, ProcessorError> {
        let input = self
            .inputs
            .get(index)
            .ok_or(ProcessorError::InputSpecMismatch(index))?;

        if let Some(input) = input {
            let input = input
                .as_kind::<S>()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;

            Ok(itertools::Either::Left(input.iter()))
        } else {
            Ok(itertools::Either::Right(std::iter::repeat(&None)))
        }
    }

    /// Returns an iterator over the input samples at the given index, if the input is a sample buffer.
    #[inline]
    pub fn iter_input_as_samples(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<Sample>> + '_, ProcessorError> {
        let buffer = self.input(index);

        if let Some(buffer) = buffer {
            let buffer = buffer
                .as_sample()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;

            Ok(itertools::Either::Left(buffer.iter().copied()))
        } else {
            Ok(itertools::Either::Right(std::iter::repeat(None)))
        }
    }

    /// Returns an iterator over the input integers at the given index, if the input is an integer buffer.
    #[inline]
    pub fn iter_input_as_ints(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<i64>> + '_, ProcessorError> {
        let buffer = self.input(index);

        if let Some(buffer) = buffer {
            let buffer = buffer
                .as_int()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;

            Ok(itertools::Either::Left(buffer.iter().copied()))
        } else {
            Ok(itertools::Either::Right(std::iter::repeat(None)))
        }
    }

    /// Returns an iterator over the input booleans at the given index, if the input is a boolean buffer.
    #[inline]
    pub fn iter_input_as_bools(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<bool>> + '_, ProcessorError> {
        let buffer = self.input(index);

        if let Some(buffer) = buffer {
            let buffer = buffer
                .as_bool()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;

            Ok(itertools::Either::Left(buffer.iter().copied()))
        } else {
            Ok(itertools::Either::Right(std::iter::repeat(None)))
        }
    }

    /// Returns an iterator over the input strings at the given index, if the input is a string buffer.
    #[inline]
    pub fn iter_input_as_strings(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<&String>> + '_, ProcessorError> {
        let buffer = self.input(index);

        if let Some(buffer) = buffer {
            let buffer = buffer
                .as_string()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;

            Ok(itertools::Either::Left(buffer.iter().map(Option::as_ref)))
        } else {
            Ok(itertools::Either::Right(std::iter::repeat(None)))
        }
    }

    /// Returns an iterator over the input lists at the given index, if the input is a list buffer.
    #[inline]
    pub fn iter_input_as_lists(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<&Vec<Signal>>> + '_, ProcessorError> {
        let buffer = self.input(index);

        if let Some(buffer) = buffer {
            let buffer = buffer
                .as_list()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;

            Ok(itertools::Either::Left(buffer.iter().map(Option::as_ref)))
        } else {
            Ok(itertools::Either::Right(std::iter::repeat(None)))
        }
    }

    /// Returns an iterator over the input MIDI messages at the given index, if the input is a MIDI message buffer.
    #[inline]
    pub fn iter_input_as_midi(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<&Vec<u8>>> + '_, ProcessorError> {
        let buffer = self.input(index);

        if let Some(buffer) = buffer {
            let buffer = buffer
                .as_midi()
                .ok_or(ProcessorError::InputSpecMismatch(index))?;

            Ok(itertools::Either::Left(buffer.iter().map(Option::as_ref)))
        } else {
            Ok(itertools::Either::Right(std::iter::repeat(None)))
        }
    }
}

/// A collection of borrowed output buffers for a [`Processor`] to write to, and related information about them.
pub struct ProcessorOutputs<'a> {
    /// The output signal specs.
    pub output_spec: &'a [OutputSpec],
    /// The output buffers to write to.
    pub outputs: &'a mut [SignalBuffer],
}

impl<'a> ProcessorOutputs<'a> {
    /// Returns the output buffer at the given index.
    #[inline]
    pub fn output(&mut self, index: usize) -> &mut SignalBuffer {
        &mut self.outputs[index]
    }

    #[inline]
    pub fn output_as<S: SignalData>(
        &mut self,
        index: usize,
    ) -> Result<&mut Buffer<S>, ProcessorError> {
        self.output(index)
            .as_kind_mut::<S>()
            .ok_or(ProcessorError::OutputSpecMismatch(index))
    }

    /// Returns the output buffer at the given index, if it is a sample buffer.
    #[inline]
    pub fn output_as_samples(
        &mut self,
        index: usize,
    ) -> Result<&mut Buffer<Sample>, ProcessorError> {
        self.output(index)
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(index))
    }

    /// Returns the output buffer at the given index, if it is an integer buffer.
    #[inline]
    pub fn output_as_ints(&mut self, index: usize) -> Result<&mut Buffer<i64>, ProcessorError> {
        self.output(index)
            .as_int_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(index))
    }

    /// Returns the output buffer at the given index, if it is a boolean buffer.
    #[inline]
    pub fn output_as_bools(&mut self, index: usize) -> Result<&mut Buffer<bool>, ProcessorError> {
        self.output(index)
            .as_bool_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(index))
    }

    /// Returns the output buffer at the given index, if it is a string buffer.
    #[inline]
    pub fn output_as_strings(
        &mut self,
        index: usize,
    ) -> Result<&mut Buffer<String>, ProcessorError> {
        self.output(index)
            .as_string_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(index))
    }

    /// Returns the output buffer at the given index, if it is a list buffer.
    #[inline]
    pub fn output_as_lists(
        &mut self,
        index: usize,
    ) -> Result<&mut Buffer<Vec<Signal>>, ProcessorError> {
        self.output(index)
            .as_list_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(index))
    }

    /// Returns the output buffer at the given index, if it is a MIDI message buffer.
    #[inline]
    pub fn output_as_midi(&mut self, index: usize) -> Result<&mut Buffer<Vec<u8>>, ProcessorError> {
        self.output(index)
            .as_midi_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(index))
    }

    /// Returns an iterator over the output buffers.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SignalBuffer> + '_ {
        self.outputs.iter_mut()
    }

    #[inline]
    pub fn iter_output_as<S: SignalData>(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<S>> + '_, ProcessorError> {
        Ok(self.output_as::<S>(index)?.iter_mut())
    }

    /// Returns an iterator over the output samples at the given index.
    #[inline]
    pub fn iter_output_mut_as_samples(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<Sample>> + '_, ProcessorError> {
        Ok(self.output_as_samples(index)?.iter_mut())
    }

    /// Returns an iterator over the output integers at the given index.
    #[inline]
    pub fn iter_output_mut_as_ints(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<i64>> + '_, ProcessorError> {
        Ok(self.output_as_ints(index)?.iter_mut())
    }

    /// Returns an iterator over the output booleans at the given index.
    #[inline]
    pub fn iter_output_mut_as_bools(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<bool>> + '_, ProcessorError> {
        Ok(self.output_as_bools(index)?.iter_mut())
    }

    /// Returns an iterator over the output strings at the given index.
    #[inline]
    pub fn iter_output_mut_as_strings(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<String>> + '_, ProcessorError> {
        Ok(self.output_as_strings(index)?.iter_mut())
    }

    /// Returns an iterator over the output lists at the given index.
    #[inline]
    pub fn iter_output_mut_as_lists(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<Vec<Signal>>> + '_, ProcessorError> {
        Ok(self.output_as_lists(index)?.iter_mut())
    }

    /// Returns an iterator over the output MIDI messages at the given index.
    #[inline]
    pub fn iter_output_mut_as_midi(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<Vec<u8>>> + '_, ProcessorError> {
        Ok(self.output_as_midi(index)?.iter_mut())
    }

    /// Splits this [`ProcessorOutputs`] into two at the given index.
    ///
    /// Note that the indices in the returned [`ProcessorOutputs`] are relative to the split point, not the original outputs.
    ///
    /// # Example
    ///
    /// ```
    /// use raug::processor::{ProcessorOutputs, SignalSpec};
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
    /// let mut process_outputs = ProcessorOutputs {
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
    pub fn split_at_mut(&mut self, index: usize) -> (ProcessorOutputs, ProcessorOutputs) {
        let (left, right) = self.outputs.split_at_mut(index);
        (
            ProcessorOutputs {
                output_spec: &self.output_spec[..index],
                outputs: left,
            },
            ProcessorOutputs {
                output_spec: &self.output_spec[index..],
                outputs: right,
            },
        )
    }
}

/// A trait for processing audio or control signals.
pub trait Processor: 'static + Send + Sync + ProcessClone + DowncastSync {
    /// Returns the name of this [`Processor`].
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or_default()
    }

    /// Returns the names of the inputs this [`Processor`] expects.
    fn input_names(&self) -> Vec<String>;

    /// Returns information about the outputs this [`Processor`] produces.
    fn output_spec(&self) -> Vec<OutputSpec>;

    /// Returns the number of input buffers/channels this [`Processor`] expects.
    fn num_inputs(&self) -> usize {
        self.input_names().len()
    }

    /// Returns the number of output buffers/channels this [`Processor`] produces.
    fn num_outputs(&self) -> usize {
        self.output_spec().len()
    }

    /// Called before the first [`Processor::process`] call, and anytime the graph changes.
    fn prepare(&mut self) {}

    /// Called whenever the runtime's sample rates or block size change.
    #[allow(unused)]
    fn resize_buffers(&mut self, sample_rate: Sample, block_size: usize) {}

    /// Processes the given input buffers and writes the results to the given output buffers.
    ///
    /// The number of input and output buffers must match the numbers returned by [`Processor::num_inputs`] and [`Processor::num_outputs`].
    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError>;
}
impl_downcast!(sync Processor);

mod sealed {
    pub trait Sealed {}
    impl<T: Clone> Sealed for T {}
}

#[doc(hidden)]
pub trait ProcessClone: sealed::Sealed {
    fn clone_boxed(&self) -> Box<dyn Processor>;
}

impl<T> ProcessClone for T
where
    T: Clone + Processor,
{
    fn clone_boxed(&self) -> Box<dyn Processor> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Processor> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

impl Debug for dyn Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
