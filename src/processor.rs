use std::fmt::Debug;

use crate::signal::Buffer;

pub struct Param {
    pub name: &'static str,
    pub min: f64,
    pub max: f64,
}

impl Default for Param {
    fn default() -> Self {
        Self {
            name: "",
            min: f64::MIN,
            max: f64::MAX,
        }
    }
}

impl Param {
    pub fn new(name: &'static str, min: f64, max: f64) -> Self {
        Self { name, min, max }
    }

    pub fn default_with_name(name: &'static str) -> Self {
        Self {
            name,
            min: f64::MIN,
            max: f64::MAX,
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
    fn input_params(&self) -> Vec<Param>;

    /// Returns information about the outputs this [`Process`] produces.
    fn output_params(&self) -> Vec<Param>;

    /// Returns the number of input buffers/channels this [`Process`] expects.
    fn num_inputs(&self) -> usize {
        self.input_params().len()
    }

    /// Returns the number of output buffers/channels this [`Process`] produces.
    fn num_outputs(&self) -> usize {
        self.output_params().len()
    }

    /// Called before the first [`Process::process`] call, and anytime the graph changes.
    fn prepare(&mut self) {}

    /// Called whenever the runtime's sample rates or block size change.
    #[allow(unused)]
    fn resize_buffers(&mut self, sample_rate: f64, block_size: usize) {}

    /// Processes the given input buffers and writes the results to the given output buffers.
    ///
    /// The number of input and output buffers must match the numbers returned by [`Process::num_inputs`] and [`Process::num_outputs`].
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]);

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
    inputs: Box<[Buffer]>,
    outputs: Box<[Buffer]>,
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
        for _param in processor.input_params() {
            input_buffers.push(Buffer::zeros(0));
        }
        let mut output_buffers = Vec::with_capacity(processor.num_outputs());
        for _param in processor.output_params() {
            output_buffers.push(Buffer::zeros(0));
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
    pub fn input_params(&self) -> Vec<Param> {
        self.processor.input_params()
    }

    /// Returns information about the outputs this [`Processor`] produces.
    pub fn output_params(&self) -> Vec<Param> {
        self.processor.output_params()
    }

    /// Resizes the input and output buffers to match the given sample rates and block size.
    pub fn resize_buffers(&mut self, sample_rate: f64, block_size: usize) {
        for input in self.inputs.iter_mut() {
            input.resize(block_size);
        }
        for output in self.outputs.iter_mut() {
            output.resize(block_size);
        }
        self.processor.resize_buffers(sample_rate, block_size);
    }

    /// Returns a slice of the input buffers.
    #[inline]
    pub fn inputs(&self) -> &[Buffer] {
        &self.inputs[..]
    }

    /// Returns a mutable slice of the input buffers.
    #[inline]
    pub fn inputs_mut(&mut self) -> &mut [Buffer] {
        &mut self.inputs[..]
    }

    /// Returns a reference to the input buffer at the given index.
    #[inline]
    pub fn input(&self, index: usize) -> &Buffer {
        &self.inputs()[index]
    }

    /// Returns a mutable reference to the input buffer at the given index.
    #[inline]
    pub fn input_mut(&mut self, index: usize) -> &mut Buffer {
        &mut self.inputs_mut()[index]
    }

    /// Returns a slice of the output buffers.
    #[inline]
    pub fn outputs(&self) -> &[Buffer] {
        &self.outputs[..]
    }

    /// Returns a reference to the output buffer at the given index.
    #[inline]
    pub fn output(&self, index: usize) -> &Buffer {
        &self.outputs()[index]
    }

    /// Prepares the processor for processing. This is called before the first [`Processor::process`] call, and anytime the graph changes.
    #[inline]
    pub fn prepare(&mut self) {
        self.processor.prepare();
    }

    /// Processes the input buffers and writes the results to the output buffers.
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
