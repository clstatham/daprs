//! Audio processing utilities and types.

use std::fmt::Debug;

use downcast_rs::{impl_downcast, DowncastSync};
use thiserror::Error;

use crate::signal::{Buffer, Float, List, MidiMessage, Signal, SignalBuffer, SignalType};

/// Error type for [`Processor`] operations.
#[derive(Debug, Clone, Error)]
pub enum ProcessorError {
    /// The number of inputs must match the number returned by [`Processor::num_inputs()`].
    #[error("The number of inputs must match the number returned by Processor::num_inputs()")]
    NumInputsMismatch,

    /// The number of outputs must match the number returned by [`Processor::num_outputs()`].
    #[error("The number of outputs must match the number returned by Processor::num_outputs()")]
    NumOutputsMismatch,

    /// Input signal type mismatch.
    #[error("Input {index} signal type mismatch (expected {expected:?}, got {actual:?})")]
    InputSpecMismatch {
        /// The index of the input signal.
        index: usize,
        /// The expected signal type.
        expected: SignalType,
        /// The actual signal type.
        actual: SignalType,
    },

    /// Output signal type mismatch.
    #[error("Output {index} signal type mismatch (expected {expected:?}, got {actual:?})")]
    OutputSpecMismatch {
        /// The index of the output signal.
        index: usize,
        /// The expected signal type.
        expected: SignalType,
        /// The actual signal type.
        actual: SignalType,
    },

    /// Invalid value.
    #[error("Invalid value: {0}")]
    InvalidValue(&'static str),
}

/// Information about an input or output of a [`Processor`].
#[derive(Debug, Clone)]
pub struct SignalSpec {
    /// The name of the input or output.
    pub name: String,
    /// The type of the input or output.
    pub type_: SignalType,
}

impl Default for SignalSpec {
    fn default() -> Self {
        Self {
            name: "".into(),
            type_: SignalType::Float,
        }
    }
}

impl SignalSpec {
    /// Creates a new [`SignalSpec`] with the given name and type.
    pub fn new(name: impl Into<String>, type_: SignalType) -> Self {
        Self {
            name: name.into(),
            type_,
        }
    }
}

/// A collection of input signals for a [`Processor`] and their specifications.
#[derive(Debug, Clone, Copy)]
pub struct ProcessorInputs<'a, 'b> {
    /// The specifications of the input signals.
    pub input_specs: &'a [SignalSpec],

    /// The input signals.
    pub inputs: &'a [Option<&'b SignalBuffer>],
}

impl<'a, 'b> ProcessorInputs<'a, 'b> {
    /// Returns the number of input signals.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_specs.len()
    }

    /// Returns the input signal at the given index. Unconnected inputs are represented as `None`.
    #[inline]
    pub fn input(&self, index: usize) -> Option<&'b SignalBuffer> {
        self.inputs[index]
    }

    /// Returns an iterator over the input signals. Unconnected inputs are represented as `None`.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = Option<&'b SignalBuffer>> + '_ {
        self.inputs.iter().copied()
    }

    /// Returns an iterator over the input signal at the given index, if it is of the given type.
    #[inline]
    pub fn iter_input_as<S: Signal>(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = &Option<S>> + '_, ProcessorError> {
        let buffer = &self.inputs[index];

        if let Some(input) = buffer {
            let input = input
                .as_type::<S>()
                .ok_or(ProcessorError::InputSpecMismatch {
                    index,
                    expected: S::TYPE,
                    actual: input.type_(),
                })?;

            Ok(itertools::Either::Left(input.iter()))
        } else {
            Ok(itertools::Either::Right(std::iter::repeat(&None)))
        }
    }

    /// Returns an iterator over the input signal at the given index, if it is a [`Float`] signal.
    #[inline]
    pub fn iter_input_as_floats(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<Float>> + '_, ProcessorError> {
        Self::iter_input_as::<Float>(self, index).map(|iter| iter.copied())
    }

    /// Returns an iterator over the input signal at the given index, if it is an [`i64`] signal.
    #[inline]
    pub fn iter_input_as_ints(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<i64>> + '_, ProcessorError> {
        Self::iter_input_as::<i64>(self, index).map(|iter| iter.copied())
    }

    /// Returns an iterator over the input signal at the given index, if it is a [`bool`] signal.
    #[inline]
    pub fn iter_input_as_bools(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<bool>> + '_, ProcessorError> {
        Self::iter_input_as::<bool>(self, index).map(|iter| iter.copied())
    }

    /// Returns an iterator over the input signal at the given index, if it is a [`String`] signal.
    #[inline]
    pub fn iter_input_as_strings(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<&String>> + '_, ProcessorError> {
        Self::iter_input_as::<String>(self, index).map(|iter| iter.map(Option::as_ref))
    }

    /// Returns an iterator over the input signal at the given index, if it is a [`List`] signal.
    #[inline]
    pub fn iter_input_as_lists(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<&List>> + '_, ProcessorError> {
        Self::iter_input_as::<List>(self, index).map(|iter| iter.map(Option::as_ref))
    }

    /// Returns an iterator over the input signal at the given index, if it is a [`MidiMessage`] signal.
    #[inline]
    pub fn iter_input_as_midi(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<&MidiMessage>> + '_, ProcessorError> {
        Self::iter_input_as::<MidiMessage>(self, index).map(|iter| iter.map(Option::as_ref))
    }
}

/// A collection of output signals for a [`Processor`] and their specifications.
pub struct ProcessorOutputs<'a> {
    /// The specifications of the output signals.
    pub output_spec: &'a [SignalSpec],

    /// The output signals.
    pub outputs: &'a mut [SignalBuffer],
}

impl<'a> ProcessorOutputs<'a> {
    /// Returns the output signal at the given index.
    #[inline]
    pub fn output(&mut self, index: usize) -> &mut SignalBuffer {
        &mut self.outputs[index]
    }

    /// Returns the output signal at the given index, if it is of the given type.
    #[inline]
    pub fn output_as<S: Signal>(&mut self, index: usize) -> Result<&mut Buffer<S>, ProcessorError> {
        let actual = self.output(index).type_();
        self.output(index)
            .as_kind_mut::<S>()
            .ok_or(ProcessorError::OutputSpecMismatch {
                index,
                expected: S::TYPE,
                actual,
            })
    }

    /// Returns the output signal at the given index, if it is a [`Float`] signal.
    #[inline]
    pub fn output_as_floats(&mut self, index: usize) -> Result<&mut Buffer<Float>, ProcessorError> {
        self.output_as::<Float>(index)
    }

    /// Returns the output signal at the given index, if it is an [`i64`] signal.
    #[inline]
    pub fn output_as_ints(&mut self, index: usize) -> Result<&mut Buffer<i64>, ProcessorError> {
        self.output_as::<i64>(index)
    }

    /// Returns the output signal at the given index, if it is a [`bool`] signal.
    #[inline]
    pub fn output_as_bools(&mut self, index: usize) -> Result<&mut Buffer<bool>, ProcessorError> {
        self.output_as::<bool>(index)
    }

    /// Returns the output signal at the given index, if it is a [`String`] signal.
    #[inline]
    pub fn output_as_strings(
        &mut self,
        index: usize,
    ) -> Result<&mut Buffer<String>, ProcessorError> {
        self.output_as::<String>(index)
    }

    /// Returns the output signal at the given index, if it is a [`List`] signal.
    #[inline]
    pub fn output_as_lists(&mut self, index: usize) -> Result<&mut Buffer<List>, ProcessorError> {
        self.output_as::<List>(index)
    }

    /// Returns the output signal at the given index, if it is a [`MidiMessage`] signal.
    #[inline]
    pub fn output_as_midi(
        &mut self,
        index: usize,
    ) -> Result<&mut Buffer<MidiMessage>, ProcessorError> {
        self.output_as::<MidiMessage>(index)
    }

    /// Returns an iterator over the output signals.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SignalBuffer> + '_ {
        self.outputs.iter_mut()
    }

    /// Returns an iterator over the output signal at the given index, if it is of the given type.
    #[inline]
    pub fn iter_output_as<S: Signal>(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<S>> + '_, ProcessorError> {
        Ok(self.output_as::<S>(index)?.iter_mut())
    }

    /// Returns an iterator over the output signal at the given index, if it is a [`Float`] signal.
    #[inline]
    pub fn iter_output_mut_as_samples(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<Float>> + '_, ProcessorError> {
        Ok(self.output_as_floats(index)?.iter_mut())
    }

    /// Returns an iterator over the output signal at the given index, if it is an [`i64`] signal.
    #[inline]
    pub fn iter_output_mut_as_ints(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<i64>> + '_, ProcessorError> {
        Ok(self.output_as_ints(index)?.iter_mut())
    }

    /// Returns an iterator over the output signal at the given index, if it is a [`bool`] signal.
    #[inline]
    pub fn iter_output_mut_as_bools(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<bool>> + '_, ProcessorError> {
        Ok(self.output_as_bools(index)?.iter_mut())
    }

    /// Returns an iterator over the output signal at the given index, if it is a [`String`] signal.
    #[inline]
    pub fn iter_output_mut_as_strings(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<String>> + '_, ProcessorError> {
        Ok(self.output_as_strings(index)?.iter_mut())
    }

    /// Returns an iterator over the output signal at the given index, if it is a [`List`] signal.
    #[inline]
    pub fn iter_output_mut_as_lists(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<List>> + '_, ProcessorError> {
        Ok(self.output_as_lists(index)?.iter_mut())
    }

    /// Returns an iterator over the output signal at the given index, if it is a [`MidiMessage`] signal.
    #[inline]
    pub fn iter_output_mut_as_midi(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<MidiMessage>> + '_, ProcessorError> {
        Ok(self.output_as_midi(index)?.iter_mut())
    }

    /// Splits the outputs into two parts at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
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

/// A processor that can process audio signals.
pub trait Processor: 'static + Send + Sync + ProcessClone + DowncastSync {
    /// Returns the name of the processor.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or_default()
    }

    /// Returns the specifications of the input signals of the processor.
    fn input_spec(&self) -> Vec<SignalSpec>;

    /// Returns the specifications of the output signals of the processor.
    fn output_spec(&self) -> Vec<SignalSpec>;

    /// Returns the number of input signals required by the processor.
    fn num_inputs(&self) -> usize {
        self.input_spec().len()
    }

    /// Returns the number of output signals produced by the processor.
    fn num_outputs(&self) -> usize {
        self.output_spec().len()
    }

    /// Prepares the processor for processing.
    fn prepare(&mut self) {}

    /// Called anytime the sample rate or block size changes.
    #[allow(unused)]
    fn resize_buffers(&mut self, sample_rate: Float, block_size: usize) {}

    /// Processes the input signals and writes the output signals.
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
