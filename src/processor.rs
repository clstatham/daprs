use std::fmt::Debug;

use thiserror::Error;

use crate::signal::{Signal, SignalBuffer};

#[derive(Debug, Clone, Error)]
pub enum ProcessorError {
    #[error("The number of inputs must match the number returned by Process::num_inputs()")]
    NumInputsMismatch,
    #[error("The number of outputs must match the number returned by Process::num_outputs()")]
    NumOutputsMismatch,
    #[error("Input {0} signal type mismatch")]
    InputSpecMismatch(usize),
    #[error("Output {0} signal type mismatch")]
    OutputSpecMismatch(usize),
}

/// Information about an input/output of a [`Process`] implementor.
#[derive(Debug)]
pub struct SignalSpec {
    pub name: &'static str,
    pub min: Option<Signal>,
    pub max: Option<Signal>,
    pub default_value: Signal,
}

impl Default for SignalSpec {
    /// Creates a new unnamed and unbounded [`SignalSpec`] (min = [`f64::MIN`], max = [`f64::MAX`]).
    fn default() -> Self {
        Self {
            name: "",
            min: None,
            max: None,
            default_value: Signal::Sample(0.0.into()),
        }
    }
}

impl SignalSpec {
    /// Creates a new bounded [`SignalSpec`] with the given name, minimum and maximum values.
    pub fn new(
        name: &'static str,
        min: Option<Signal>,
        max: Option<Signal>,
        default_value: Signal,
    ) -> Self {
        Self {
            name,
            min,
            max,
            default_value,
        }
    }

    /// Creates a new unbounded [`SignalSpec`] with the given name.
    pub fn unbounded(name: &'static str, default_value: impl Into<Signal>) -> Self {
        Self {
            name,
            min: None,
            max: None,
            default_value: default_value.into(),
        }
    }
}

/// A trait for processing audio or control signals.
///
/// This is usually used as part of a [`Processor`], operating on its internal input/output buffers.
pub trait Process: 'static + Send + Sync + ProcessClone {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Returns information about the inputs this [`Process`] expects.
    fn input_spec(&self) -> Vec<SignalSpec>;

    /// Returns information about the outputs this [`Process`] produces.
    fn output_spec(&self) -> Vec<SignalSpec>;

    fn make_default_input_buffers(&self, length: usize) -> Vec<SignalBuffer> {
        let input_spec = self.input_spec();
        let mut buffers = Vec::with_capacity(input_spec.len());
        for spec in input_spec {
            buffers.push(SignalBuffer::from_spec_default(&spec, length));
        }
        buffers
    }

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
    inputs: Box<[SignalBuffer]>,
    outputs: Box<[SignalBuffer]>,
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
        Self {
            inputs: processor.make_default_input_buffers(0).into_boxed_slice(),
            outputs: processor.make_default_output_buffers(0).into_boxed_slice(),
            processor,
        }
    }

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
        let input_spec = self.input_spec();
        for (input, spec) in self.inputs.iter_mut().zip(input_spec) {
            input.resize(block_size, spec.default_value);
        }
        let output_spec = self.output_spec();
        for (output, spec) in self.outputs.iter_mut().zip(output_spec) {
            output.resize(block_size, spec.default_value);
        }
        self.processor.resize_buffers(sample_rate, block_size);
    }

    /// Returns a slice of the input buffers.
    #[inline]
    pub fn inputs(&self) -> &[SignalBuffer] {
        &self.inputs[..]
    }

    /// Returns a mutable slice of the input buffers.
    #[inline]
    pub fn inputs_mut(&mut self) -> &mut [SignalBuffer] {
        &mut self.inputs[..]
    }

    /// Returns a reference to the input buffer at the given index.
    #[inline]
    pub fn input(&self, index: usize) -> &SignalBuffer {
        &self.inputs()[index]
    }

    /// Returns a mutable reference to the input buffer at the given index.
    #[inline]
    pub fn input_mut(&mut self, index: usize) -> &mut SignalBuffer {
        &mut self.inputs_mut()[index]
    }

    /// Returns a slice of the output buffers.
    #[inline]
    pub fn outputs(&self) -> &[SignalBuffer] {
        &self.outputs[..]
    }

    /// Returns a reference to the output buffer at the given index.
    #[inline]
    pub fn output(&self, index: usize) -> &SignalBuffer {
        &self.outputs()[index]
    }

    /// Prepares the processor for processing. This is called before the first [`Processor::process`] call, and anytime the graph changes.
    #[inline]
    pub fn prepare(&mut self) {
        self.processor.prepare();
    }

    /// Processes the input buffers and writes the results to the output buffers.
    #[inline]
    pub fn process(&mut self) -> Result<(), ProcessorError> {
        if self.inputs.len() != self.processor.num_inputs() {
            return Err(ProcessorError::NumInputsMismatch);
        }
        if self.outputs.len() != self.processor.num_outputs() {
            return Err(ProcessorError::NumOutputsMismatch);
        }
        self.processor.process(&self.inputs, &mut self.outputs)?;
        Ok(())
    }
}
