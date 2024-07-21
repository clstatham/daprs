use std::fmt::Debug;

use crate::sample::Buffer;

/// A trait for processing audio samples.
///
/// This is usually used as part of a [`Node`], operating on its internal input/output buffers.
pub trait Process: 'static + Send + Sync + ProcessorClone {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Returns the number of input buffers/channels this [`Processor`] expects.
    fn num_inputs(&self) -> usize;

    /// Returns the number of output buffers/channels this [`Processor`] expects.
    fn num_outputs(&self) -> usize;

    /// Called before the first [`Node::process`] call.
    fn prepare(&mut self) {}

    /// Called whenever the global sample or block size changes.
    #[allow(unused)]
    fn reset(&mut self, sample_rate: f64, block_size: usize) {}

    /// Processes the given inputs and writes the results to the given outputs.
    ///
    /// The number of inputs and outputs must match the number returned by [`Processor::num_inputs`] and [`Processor::num_outputs`].
    ///
    /// The length of each input and output buffer must match the block size returned by [`Node::block_size`].
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]);
}

pub trait ProcessorClone {
    fn clone_boxed(&self) -> Box<dyn Process>;
}

impl<T: ?Sized> ProcessorClone for T
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

/// A node in the audio graph.
///
/// This is a wrapper around a [`Processor`] that provides input and output buffers for the processor to use.
#[derive(Debug, Clone)]
pub struct Processor {
    processor: Box<dyn Process>,
    input_buffers: Box<[Buffer]>,
    output_buffers: Box<[Buffer]>,
}

impl Processor {
    pub fn new(processor: Box<dyn Process>) -> Self {
        Self {
            input_buffers: vec![Buffer::zeros(0); processor.num_inputs()].into_boxed_slice(),
            output_buffers: vec![Buffer::zeros(0); processor.num_outputs()].into_boxed_slice(),
            processor,
        }
    }

    pub fn set_block_size(&mut self, block_size: usize) {
        for input in self.input_buffers.iter_mut() {
            input.resize(block_size);
        }
        for output in self.output_buffers.iter_mut() {
            output.resize(block_size);
        }
    }

    pub fn reset(&mut self, sample_rate: f64, block_size: usize) {
        self.set_block_size(block_size);
        self.processor.reset(sample_rate, block_size);
    }

    #[inline]
    pub fn inputs(&self) -> &[Buffer] {
        &self.input_buffers[..]
    }

    #[inline]
    pub fn inputs_mut(&mut self) -> &mut [Buffer] {
        &mut self.input_buffers[..]
    }

    #[inline]
    pub fn input(&self, index: usize) -> &Buffer {
        &self.inputs()[index]
    }

    #[inline]
    pub fn input_mut(&mut self, index: usize) -> &mut Buffer {
        &mut self.inputs_mut()[index]
    }

    #[inline]
    pub fn outputs(&self) -> &[Buffer] {
        &self.output_buffers[..]
    }

    #[inline]
    pub fn output(&self, index: usize) -> &Buffer {
        &self.outputs()[index]
    }

    #[inline]
    pub fn block_size(&self) -> usize {
        if let Some(first) = self.inputs().first() {
            first.len()
        } else if let Some(first) = self.outputs().first() {
            first.len()
        } else {
            0
        }
    }

    #[inline]
    pub fn prepare(&mut self) {
        self.processor.prepare();
    }

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
        self.processor
            .process(&self.input_buffers, &mut self.output_buffers);
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Input,
    Processor(Processor),
    Output,
}

impl Node {
    pub fn new_input() -> Self {
        Self::Input
    }

    pub fn new_processor<P: Process>(processor: P) -> Self {
        Self::Processor(Processor::new(Box::new(processor)))
    }

    pub fn new_output() -> Self {
        Self::Output
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Input => "Input",
            Self::Processor(processor) => processor.processor.name(),
            Self::Output => "Output",
        }
    }

    pub fn inputs(&self) -> &[Buffer] {
        if let Self::Processor(processor) = self {
            processor.inputs()
        } else {
            &[]
        }
    }

    pub fn inputs_mut(&mut self) -> &mut [Buffer] {
        if let Self::Processor(processor) = self {
            processor.inputs_mut()
        } else {
            &mut []
        }
    }

    pub fn outputs(&self) -> &[Buffer] {
        if let Self::Processor(processor) = self {
            processor.outputs()
        } else {
            &[]
        }
    }

    pub fn set_block_size(&mut self, block_size: usize) {
        if let Self::Processor(processor) = self {
            processor.set_block_size(block_size);
        }
    }

    pub fn reset(&mut self, sample_rate: f64, block_size: usize) {
        if let Self::Processor(processor) = self {
            processor.reset(sample_rate, block_size);
        }
    }

    pub fn prepare(&mut self) {
        if let Self::Processor(processor) = self {
            processor.prepare();
        }
    }

    pub fn process(&mut self) {
        if let Self::Processor(processor) = self {
            processor.process();
        }
    }
}
