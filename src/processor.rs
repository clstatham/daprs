//! Audio processing utilities and types.

use std::fmt::Debug;

use thiserror::Error;

use crate::signal::{Signal, SignalBuffer, SignalKind};

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

/// A trait for processing audio or control signals.
///
/// This is usually used as part of a [`Processor`], operating on its internal input/output buffers.
pub trait Process: 'static + Send + Sync + ProcessClone {
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
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError>;

    /// Clones this [`Process`] into a [`Processor`] object that can be used in the audio graph.
    fn processor(&self) -> Processor {
        Processor::new_from_boxed(self.clone_boxed())
    }
}

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
///
/// This is a wrapper around a [`Box<dyn Process>`](Process) that provides input and output buffers for the processor to use.
#[derive(Clone)]

pub struct Processor {
    processor: Box<dyn Process>,
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
        Self { processor }
    }

    /// Returns the name of this [`Processor`].
    pub fn name(&self) -> &str {
        self.processor.name()
    }

    /// Returns information about the inputs this [`Processor`] expects.
    pub fn input_spec(&self) -> Vec<SignalSpec> {
        self.processor.input_spec()
    }

    /// Returns information about the outputs this [`Processor`] produces.
    pub fn output_spec(&self) -> Vec<SignalSpec> {
        self.processor.output_spec()
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
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        if inputs.len() != self.processor.num_inputs() {
            return Err(ProcessorError::NumInputsMismatch);
        }
        if outputs.len() != self.processor.num_outputs() {
            return Err(ProcessorError::NumOutputsMismatch);
        }
        self.processor.process(inputs, outputs)?;
        Ok(())
    }
}
