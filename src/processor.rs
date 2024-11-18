//! Audio processing utilities and types.

use std::fmt::Debug;

use downcast_rs::{impl_downcast, DowncastSync};
use itertools::Either;
use thiserror::Error;

use crate::{
    signal::{
        AnySignal, AnySignalMut, AnySignalRef, Float, List, MidiMessage, Signal, SignalBuffer,
        SignalType,
    },
    GraphSerde,
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

    /// Invalid cast.
    #[error("Invalid cast: {0:?} to {1:?}")]
    InvalidCast(SignalType, SignalType),
}

/// Information about an input or output of a [`Processor`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignalSpec {
    /// The name of the input or output.
    pub name: String,
    /// The type of the input or output.
    pub signal_type: SignalType,
}

impl Default for SignalSpec {
    fn default() -> Self {
        Self {
            name: "".into(),
            signal_type: SignalType::Float,
        }
    }
}

impl SignalSpec {
    /// Creates a new [`SignalSpec`] with the given name and type.
    pub fn new(name: impl Into<String>, signal_type: SignalType) -> Self {
        Self {
            name: name.into(),
            signal_type,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Ternary<A, B, C> {
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

/// The mode in which a processor should process signals.
///
/// - `Block` means the processor processes the entire block of samples at once.
/// - `Sample` means the processor processes each sample individually.
#[derive(Debug, Clone, Copy)]
pub enum ProcessMode {
    Block,
    Sample(
        /// The index of the current sample within the block.
        usize,
    ),
}

/// The output of a [`Processor`].
#[derive(Debug)]
pub enum ProcessorOutput<'a> {
    /// A block of signals.
    Block(&'a mut SignalBuffer),
    /// A single sample.
    Sample(&'a mut SignalBuffer, usize),
}

impl<'a> ProcessorOutput<'a> {
    /// Returns the type of the output signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            ProcessorOutput::Block(buffer) => buffer.signal_type(),
            ProcessorOutput::Sample(buffer, _) => buffer.signal_type(),
        }
    }

    /// Returns an iterator over the output signal.
    pub fn iter_mut(&'a mut self) -> impl Iterator<Item = AnySignalMut> {
        match self {
            ProcessorOutput::Block(buffer) => Either::Left(buffer.iter_mut()),
            ProcessorOutput::Sample(buffer, sample_index) => {
                Either::Right(std::iter::once(buffer.get_mut(*sample_index).unwrap()))
            }
        }
    }

    /// Returns an iterator over the output signal, if it is of the given type.
    pub fn iter_mut_as<S: Signal>(&'a mut self) -> impl Iterator<Item = &mut Option<S>> {
        match self {
            ProcessorOutput::Block(buffer) => {
                Either::Left(buffer.as_type_mut().unwrap().iter_mut())
            }
            ProcessorOutput::Sample(buffer, sample_index) => Either::Right(std::iter::once(
                &mut buffer.as_type_mut::<S>().unwrap()[*sample_index],
            )),
        }
    }

    pub fn set(&mut self, index: usize, value: AnySignalRef) {
        match self {
            ProcessorOutput::Block(buffer) => {
                buffer.set(index, value.into());
            }
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.set(*sample_index, value.into())
            }
        }
    }

    pub fn set_as<S: Signal>(&mut self, index: usize, value: impl Into<Option<S>>) {
        match self {
            ProcessorOutput::Block(buffer) => {
                buffer.as_type_mut::<S>().unwrap()[index] = value.into();
            }
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.as_type_mut::<S>().unwrap()[*sample_index] = value.into();
            }
        }
    }

    pub fn set_none(&mut self, index: usize) {
        match self {
            ProcessorOutput::Block(buffer) => buffer.set_none(index),
            ProcessorOutput::Sample(buffer, sample_index) => buffer.set_none(*sample_index),
        }
    }

    pub fn fill_as<S: Signal + Clone>(&mut self, value: impl Into<Option<S>>) {
        match self {
            ProcessorOutput::Block(buffer) => buffer.as_type_mut::<S>().unwrap().fill(value.into()),
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.as_type_mut::<S>().unwrap()[*sample_index] = value.into();
            }
        }
    }

    pub fn fill(&mut self, value: AnySignal) {
        match self {
            ProcessorOutput::Block(buffer) => buffer.fill(value),
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.set(*sample_index, value.as_ref());
            }
        }
    }
}

/// A collection of input signals for a [`Processor`] and their specifications.
#[derive(Clone, Copy)]
pub struct ProcessorInputs<'a, 'b> {
    /// The specifications of the input signals.
    input_specs: &'a [SignalSpec],

    /// The input signals.
    inputs: &'a [Option<&'b SignalBuffer>],

    /// The mode in which the processor should process signals.
    mode: ProcessMode,

    /// The current sample rate.
    sample_rate: Float,

    /// The current block size.
    block_size: usize,
}

impl<'a, 'b> ProcessorInputs<'a, 'b> {
    #[inline]
    pub(crate) fn new(
        input_specs: &'a [SignalSpec],
        inputs: &'a [Option<&'b SignalBuffer>],
        mode: ProcessMode,
        sample_rate: Float,
        block_size: usize,
    ) -> Self {
        Self {
            input_specs,
            inputs,
            mode,
            sample_rate,
            block_size,
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

    /// Returns the current sample rate.
    #[inline]
    pub fn sample_rate(&self) -> Float {
        self.sample_rate
    }

    /// Returns the current block size.
    #[inline]
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Returns the input signal at the given index. Unconnected inputs are represented as `None`.
    #[inline]
    pub fn input(&self, index: usize) -> Option<&'b SignalBuffer> {
        self.inputs
            .get(index)
            .and_then(|input| input.as_ref())
            .copied()
    }

    #[inline]
    pub fn iter_input(&self, index: usize) -> impl Iterator<Item = Option<AnySignalRef>> {
        let buffer = &self.inputs[index];
        if let Some(buffer) = buffer.as_ref() {
            if let ProcessMode::Sample(sample_index) = self.mode {
                Ternary::B(std::iter::once(Some(buffer.get(sample_index).unwrap())))
            } else {
                Ternary::A(buffer.iter().map(Some))
            }
        } else {
            Ternary::C(std::iter::repeat(None))
        }
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

        if let ProcessMode::Sample(sample_index) = self.mode {
            if buffer.signal_type().is_compatible_with(&S::signal_type()) {
                Ok(Ternary::B(std::iter::once(
                    &buffer.as_type::<S>().unwrap()[sample_index],
                )))
            } else {
                Err(ProcessorError::InputSpecMismatch {
                    index,
                    expected: S::signal_type(),
                    actual: buffer.signal_type(),
                })
            }
        } else if buffer.signal_type().is_compatible_with(&S::signal_type()) {
            Ok(Ternary::A(buffer.as_type::<S>().unwrap().iter()))
        } else {
            Err(ProcessorError::InputSpecMismatch {
                index,
                expected: S::signal_type(),
                actual: buffer.signal_type(),
            })
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

    /// Returns an iterator over the input signal at the given index, if it is a [`List`] signal.
    #[inline]
    pub fn iter_input_as_lists(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<&List>> + '_, ProcessorError> {
        Self::iter_input_as::<List>(self, index).map(|iter| iter.map(|s| s.as_ref()))
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
    outputs: &'a mut [SignalBuffer],

    /// The mode in which the processor should process signals.
    mode: ProcessMode,
}

impl<'a> ProcessorOutputs<'a> {
    #[inline]
    pub(crate) fn new(
        output_spec: &'a [SignalSpec],
        outputs: &'a mut [SignalBuffer],
        mode: ProcessMode,
    ) -> Self {
        Self {
            output_spec,
            outputs,
            mode,
        }
    }

    /// Returns the output signal at the given index.
    #[inline]
    pub fn output(&mut self, index: usize) -> ProcessorOutput<'_> {
        if let ProcessMode::Sample(sample_index) = self.mode {
            ProcessorOutput::Sample(&mut self.outputs[index], sample_index)
        } else {
            ProcessorOutput::Block(&mut self.outputs[index])
        }
    }

    /// Returns the specification of the output signal at the given index.
    #[inline]
    pub fn output_spec(&self, index: usize) -> &SignalSpec {
        &self.output_spec[index]
    }

    #[inline]
    pub fn iter_output(&mut self, index: usize) -> impl Iterator<Item = AnySignalMut> {
        let output = &mut self.outputs[index];
        if let ProcessMode::Sample(sample_index) = self.mode {
            Either::Left(std::iter::once(output.get_mut(sample_index).unwrap()))
        } else {
            Either::Right(output.iter_mut())
        }
    }

    /// Returns an iterator over the output signal at the given index, if it is of the given type.
    #[inline]
    pub fn iter_output_as<S: Signal>(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<S>> + '_, ProcessorError> {
        if let ProcessMode::Sample(sample_index) = self.mode {
            let output = &mut self.outputs[index];
            if output.signal_type().is_compatible_with(&S::signal_type()) {
                Ok(Either::Left(std::iter::once(
                    &mut output.as_type_mut::<S>().unwrap()[sample_index],
                )))
            } else {
                Err(ProcessorError::OutputSpecMismatch {
                    index,
                    expected: S::signal_type(),
                    actual: output.signal_type(),
                })
            }
        } else {
            let output = &mut self.outputs[index];
            let actual = output.signal_type();
            let output =
                output
                    .as_type_mut::<S>()
                    .ok_or_else(|| ProcessorError::OutputSpecMismatch {
                        index,
                        expected: S::signal_type(),
                        actual,
                    })?;

            Ok(Either::Right(output.iter_mut()))
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

    /// Returns an iterator over the output signal at the given index, if it is a list signal.
    #[inline]
    pub fn iter_output_mut_as_lists(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<List>> + '_, ProcessorError> {
        self.iter_output_as::<List>(index)
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
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Processor
where
    Self: DowncastSync + ProcessorClone + GraphSerde,
{
    /// Returns the name of the processor.
    fn name(&self) -> &str {
        let type_name = std::any::type_name::<Self>();
        let has_generics = type_name.contains('<');
        if has_generics {
            let end = type_name.find('<').unwrap();
            let start = type_name[..end].rfind(':').map_or(0, |i| i + 1);
            &type_name[start..end]
        } else {
            type_name.rsplit(':').next().unwrap()
        }
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
    ///
    /// This method is called once before processing begins.
    #[allow(unused)]
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
pub trait ProcessorClone: sealed::Sealed {
    fn clone_boxed(&self) -> Box<dyn Processor>;
}

impl<T> ProcessorClone for T
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
