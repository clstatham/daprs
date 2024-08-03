use std::fmt::Debug;

use crate::signal::{Signal, SignalRate, SignalSpec};

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
    fn resize_buffers(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {}

    /// Processes the given input buffers and writes the results to the given output buffers.
    ///
    /// The number of input and output buffers must match the numbers returned by [`Process::num_inputs`] and [`Process::num_outputs`].
    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]);

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

impl<T: ?Sized> ProcessClone for T
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

/// A node in the audio graph that processes audio or control signals.
///
/// This is a wrapper around a [`Box<dyn Process>`](Process) that provides input and output buffers for the processor to use.
#[derive(Clone)]
pub struct Processor {
    processor: Box<dyn Process>,
    inputs: Box<[Signal]>,
    outputs: Box<[Signal]>,
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
        let mut input_buffers = Vec::with_capacity(processor.num_inputs());
        for spec in processor.input_spec() {
            input_buffers.push(Signal::default_for_spec(spec));
        }
        let mut output_buffers = Vec::with_capacity(processor.num_outputs());
        for spec in processor.output_spec() {
            output_buffers.push(Signal::default_for_spec(spec));
        }

        Self {
            inputs: input_buffers.into_boxed_slice(),
            outputs: output_buffers.into_boxed_slice(),
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
    pub fn resize_buffers(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        let control_block_size = (block_size as f64 * control_rate / audio_rate).ceil() as usize;

        for input in self.inputs.iter_mut() {
            if input.rate() == SignalRate::Control {
                input.resize_buffers(control_block_size);
            } else {
                input.resize_buffers(block_size);
            }
        }
        for output in self.outputs.iter_mut() {
            if output.rate() == SignalRate::Control {
                output.resize_buffers(control_block_size);
            } else {
                output.resize_buffers(block_size);
            }
        }
        self.processor
            .resize_buffers(audio_rate, control_rate, block_size);
    }

    /// Returns a slice of the input signals.
    #[inline]
    pub fn inputs(&self) -> &[Signal] {
        &self.inputs[..]
    }

    /// Returns a mutable slice of the input signals.
    #[inline]
    pub fn inputs_mut(&mut self) -> &mut [Signal] {
        &mut self.inputs[..]
    }

    /// Returns a reference to the input signal at the given index.
    #[inline]
    pub fn input(&self, index: usize) -> &Signal {
        &self.inputs()[index]
    }

    /// Returns a mutable reference to the input signal at the given index.
    #[inline]
    pub fn input_mut(&mut self, index: usize) -> &mut Signal {
        &mut self.inputs_mut()[index]
    }

    /// Returns a slice of the output signal.
    #[inline]
    pub fn outputs(&self) -> &[Signal] {
        &self.outputs[..]
    }

    /// Returns a reference to the output signal at the given index.
    #[inline]
    pub fn output(&self, index: usize) -> &Signal {
        &self.outputs()[index]
    }

    /// Prepares the processor for processing. This is called before the first [`Processor::process`] call, and anytime the graph changes.
    #[inline]
    pub fn prepare(&mut self) {
        self.processor.prepare();
    }

    /// Processes the input signals and writes the results to the output signals.
    #[inline]
    pub fn process(&mut self) {
        assert_eq!(
            self.inputs().len(),
            self.processor.num_inputs(),
            "The number of inputs must match the number returned by Processor::num_inputs()"
        );
        assert_eq!(
            self.outputs().len(),
            self.processor.num_outputs(),
            "The number of outputs must match the number returned by Processor::num_outputs()"
        );
        self.processor.process(&self.inputs, &mut self.outputs);
    }
}
