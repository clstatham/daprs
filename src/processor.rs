//! Audio processing utilities and types.

use std::fmt::Debug;

use downcast_rs::{impl_downcast, DowncastSync};
use itertools::Either;
use thiserror::Error;

use crate::{
    runtime::NodeOutputStorage,
    signal::{AnySignal, Float, MidiMessage, Signal, SignalBuffer, SignalType},
};

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

#[derive(Debug, Clone, Copy)]
pub enum Ternary<A, B, C> {
    A(A),
    B(B),
    C(C),
}

impl<A, B, C> Iterator for Ternary<A, B, C>
where
    A: Iterator,
    B: Iterator<Item = A::Item>,
    C: Iterator<Item = A::Item>,
{
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Ternary::A(a) => a.next(),
            Ternary::B(b) => b.next(),
            Ternary::C(c) => c.next(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessorInput<'a> {
    Block(&'a SignalBuffer),

    Sample(
        /// The signal.
        &'a AnySignal,
        /// The index of the sample within the block.
        usize,
    ),
}

impl ProcessorInput<'_> {
    #[inline]
    pub fn type_(&self) -> Option<SignalType> {
        match self {
            ProcessorInput::Block(buffer) => Some(buffer.type_()),
            ProcessorInput::Sample(signal, _) => Some(signal.type_()),
        }
    }

    pub fn iter<S: Signal>(&self) -> Option<impl Iterator<Item = &Option<S>>> {
        match self {
            ProcessorInput::Block(buffer) => Some(Either::Left(buffer.as_type()?.iter())),
            ProcessorInput::Sample(signal, _) => {
                Some(Either::Right(std::iter::once(signal.as_type::<S>()?)))
            }
        }
    }
}

#[derive(Debug)]
pub enum ProcessorOutput<'a> {
    Block(&'a mut SignalBuffer),
    Sample(&'a mut AnySignal),
}

impl<'a> ProcessorOutput<'a> {
    #[inline]
    pub fn type_(&self) -> SignalType {
        match self {
            ProcessorOutput::Block(buffer) => buffer.type_(),
            ProcessorOutput::Sample(signal) => signal.type_(),
        }
    }

    pub fn iter_mut<S: Signal>(&'a mut self) -> impl Iterator<Item = &mut Option<S>> {
        match self {
            ProcessorOutput::Block(buffer) => {
                Either::Left(buffer.as_type_mut().unwrap().iter_mut())
            }
            ProcessorOutput::Sample(signal) => {
                Either::Right(std::iter::once(signal.as_type_mut::<S>().unwrap()))
            }
        }
    }
}

/// A collection of input signals for a [`Processor`] and their specifications.
#[derive(Debug, Clone, Copy)]
pub struct ProcessorInputs<'a, 'b> {
    /// The specifications of the input signals.
    input_specs: &'a [SignalSpec],

    /// The input signals.
    inputs: &'a [Option<ProcessorInput<'b>>],
}

impl<'a, 'b> ProcessorInputs<'a, 'b> {
    #[inline]
    pub(crate) fn new(
        input_specs: &'a [SignalSpec],
        inputs: &'a [Option<ProcessorInput<'b>>],
    ) -> Self {
        Self {
            input_specs,
            inputs,
        }
    }

    /// Returns the number of input signals.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_specs.len()
    }

    /// Returns the specification of the input signal at the given index.
    #[inline]
    pub fn input_spec(&self, index: usize) -> &SignalSpec {
        &self.input_specs[index]
    }

    /// Returns the input signal at the given index. Unconnected inputs are represented as `None`.
    #[inline]
    pub fn input(&self, index: usize) -> Option<&ProcessorInput<'b>> {
        self.inputs.get(index).and_then(|input| input.as_ref())
    }

    /// Returns an iterator over the input signal at the given index, if it is of the given type.
    #[inline]
    pub fn iter_input_as<S: Signal>(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = &Option<S>> + '_, ProcessorError> {
        let buffer = &self.inputs[index];
        let Some(buffer) = buffer.as_ref() else {
            return Ok(Ternary::C(std::iter::repeat(&None)));
        };

        if let ProcessorInput::Block(input) = buffer {
            let input = input
                .as_type::<S>()
                .ok_or_else(|| ProcessorError::InputSpecMismatch {
                    index,
                    expected: S::TYPE,
                    actual: input.type_(),
                })?;

            Ok(Ternary::A(input.iter()))
        } else if let ProcessorInput::Sample(input, _) = buffer {
            if input.type_() == S::TYPE {
                Ok(Ternary::B(std::iter::once(input.as_type::<S>().unwrap())))
            } else {
                Err(ProcessorError::InputSpecMismatch {
                    index,
                    expected: S::TYPE,
                    actual: input.type_(),
                })
            }
        } else {
            unreachable!()
            // Ok(Ternary::C(std::iter::repeat(&None)))
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
        Self::iter_input_as::<String>(self, index).map(|iter| iter.map(|s| s.as_ref()))
    }

    /// Returns an iterator over the input signal at the given index, if it is a [`Buffer`] signal.
    #[inline]
    pub fn iter_input_as_buffers(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<&SignalBuffer>> + '_, ProcessorError> {
        Self::iter_input_as::<SignalBuffer>(self, index).map(|iter| iter.map(|s| s.as_ref()))
    }

    /// Returns an iterator over the input signal at the given index, if it is a [`MidiMessage`] signal.
    #[inline]
    pub fn iter_input_as_midi(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<&MidiMessage>> + '_, ProcessorError> {
        Self::iter_input_as::<MidiMessage>(self, index).map(|iter| iter.map(|s| s.as_ref()))
    }
}

/// A collection of output signals for a [`Processor`] and their specifications.
pub struct ProcessorOutputs<'a> {
    /// The specifications of the output signals.
    output_spec: &'a [SignalSpec],

    /// The output signals.
    outputs: &'a mut NodeOutputStorage,
}

impl<'a> ProcessorOutputs<'a> {
    #[inline]
    pub(crate) fn new(output_spec: &'a [SignalSpec], outputs: &'a mut NodeOutputStorage) -> Self {
        Self {
            output_spec,
            outputs,
        }
    }

    /// Returns the output signal at the given index.
    #[inline]
    pub fn output(&mut self, index: usize) -> ProcessorOutput<'_> {
        match self.outputs {
            NodeOutputStorage::Block(outputs) => ProcessorOutput::Block(&mut outputs[index]),
            NodeOutputStorage::Sample(outputs, _) => ProcessorOutput::Sample(&mut outputs[index]),
        }
    }

    /// Returns the specification of the output signal at the given index.
    #[inline]
    pub fn output_spec(&self, index: usize) -> &SignalSpec {
        &self.output_spec[index]
    }

    /// Returns an iterator over the output signal at the given index, if it is of the given type.
    #[inline]
    pub fn iter_output_as<S: Signal>(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<S>> + '_, ProcessorError> {
        match self.outputs {
            NodeOutputStorage::Block(outputs) => {
                let output = &mut outputs[index];
                let actual = output.type_();
                let output = output.as_type_mut::<S>().ok_or_else(|| {
                    ProcessorError::OutputSpecMismatch {
                        index,
                        expected: S::TYPE,
                        actual,
                    }
                })?;

                Ok(Either::Left(output.iter_mut()))
            }
            NodeOutputStorage::Sample(outputs, _) => {
                let output = &mut outputs[index];
                // let mut output = output.as_any_mut();
                if output.type_() == S::TYPE {
                    Ok(Either::Right(std::iter::once(
                        output.as_type_mut::<S>().unwrap(),
                    )))
                } else {
                    Err(ProcessorError::OutputSpecMismatch {
                        index,
                        expected: S::TYPE,
                        actual: output.type_(),
                    })
                }
            }
        }
    }

    /// Returns an iterator over the output signal at the given index, if it is a [`Float`] signal.
    #[inline]
    pub fn iter_output_mut_as_floats(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<Float>> + '_, ProcessorError> {
        self.iter_output_as::<Float>(index)
    }

    /// Returns an iterator over the output signal at the given index, if it is an [`i64`] signal.
    #[inline]
    pub fn iter_output_mut_as_ints(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<i64>> + '_, ProcessorError> {
        self.iter_output_as::<i64>(index)
    }

    /// Returns an iterator over the output signal at the given index, if it is a [`bool`] signal.
    #[inline]
    pub fn iter_output_mut_as_bools(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<bool>> + '_, ProcessorError> {
        self.iter_output_as::<bool>(index)
    }

    /// Returns an iterator over the output signal at the given index, if it is a [`String`] signal.
    #[inline]
    pub fn iter_output_mut_as_strings(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<String>> + '_, ProcessorError> {
        self.iter_output_as::<String>(index)
    }

    /// Returns an iterator over the output signal at the given index, if it is a [`Buffer`] signal.
    #[inline]
    pub fn iter_output_mut_as_lists(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<SignalBuffer>> + '_, ProcessorError> {
        self.iter_output_as::<SignalBuffer>(index)
    }

    /// Returns an iterator over the output signal at the given index, if it is a [`MidiMessage`] signal.
    #[inline]
    pub fn iter_output_mut_as_midi(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<MidiMessage>> + '_, ProcessorError> {
        self.iter_output_as::<MidiMessage>(index)
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
