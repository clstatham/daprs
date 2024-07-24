use std::fmt::Debug;

use crate::sample::{Buffer, SignalKind};

/// A trait for processing audio samples.
///
/// This is usually used as part of a [`Node`], operating on its internal input/output buffers.
pub trait Process: 'static + Send + Sync + ProcessClone {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn input_kinds(&self) -> Vec<SignalKind>;
    fn output_kinds(&self) -> Vec<SignalKind>;

    /// Returns the number of input buffers/channels this [`Processor`] expects.
    fn num_inputs(&self) -> usize;

    /// Returns the number of output buffers/channels this [`Processor`] expects.
    fn num_outputs(&self) -> usize;

    /// Called before the first [`Node::process`] call.
    fn prepare(&mut self) {}

    /// Called whenever the global sample or block size changes.
    #[allow(unused)]
    fn reset(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {}

    /// Processes the given inputs and writes the results to the given outputs.
    ///
    /// The number of inputs and outputs must match the number returned by [`Process::num_inputs`] and [`Process::num_outputs`].
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]);

    fn processor(&self) -> Processor {
        Processor::new_from_boxed(self.clone_boxed())
    }
}

pub trait ProcessClone {
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

/// A node in the audio graph that processes audio samples.
///
/// This is a wrapper around a [`Box<dyn Process>`](Process) that provides input and output buffers for the processor to use.
#[derive(Clone)]
pub struct Processor {
    processor: Box<dyn Process>,
    input_buffers: Box<[Buffer]>,
    output_buffers: Box<[Buffer]>,
}

impl Debug for Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.processor.name())
    }
}

impl Processor {
    pub fn new(processor: impl Process) -> Self {
        let mut input_buffers = Vec::with_capacity(processor.num_inputs());
        for kind in processor.input_kinds() {
            input_buffers.push(Buffer::zeros(0, kind));
        }
        let mut output_buffers = Vec::with_capacity(processor.num_outputs());
        for kind in processor.output_kinds() {
            output_buffers.push(Buffer::zeros(0, kind));
        }

        Self {
            input_buffers: input_buffers.into_boxed_slice(),
            output_buffers: output_buffers.into_boxed_slice(),
            processor: Box::new(processor),
        }
    }

    pub fn new_from_boxed(processor: Box<dyn Process>) -> Self {
        let mut input_buffers = Vec::with_capacity(processor.num_inputs());
        for kind in processor.input_kinds() {
            input_buffers.push(Buffer::zeros(0, kind));
        }
        let mut output_buffers = Vec::with_capacity(processor.num_outputs());
        for kind in processor.output_kinds() {
            output_buffers.push(Buffer::zeros(0, kind));
        }

        Self {
            input_buffers: input_buffers.into_boxed_slice(),
            output_buffers: output_buffers.into_boxed_slice(),
            processor,
        }
    }

    pub fn input_kinds(&self) -> Vec<SignalKind> {
        self.processor.input_kinds()
    }

    pub fn output_kinds(&self) -> Vec<SignalKind> {
        self.processor.output_kinds()
    }

    pub fn set_block_size(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        let control_block_size = (block_size as f64 * control_rate / audio_rate).ceil() as usize;

        for input in self.input_buffers.iter_mut() {
            if input.kind() == SignalKind::Control {
                input.resize(control_block_size);
            } else {
                input.resize(block_size);
            }
        }
        for output in self.output_buffers.iter_mut() {
            if output.kind() == SignalKind::Control {
                output.resize(control_block_size);
            } else {
                output.resize(block_size);
            }
        }
    }

    pub fn reset(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        self.set_block_size(audio_rate, control_rate, block_size);
        self.processor.reset(audio_rate, control_rate, block_size);
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

#[derive(Clone)]
pub enum GraphNode {
    Input,
    Processor(Processor),
    Output,
}

impl Debug for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input => f.write_str("Input"),
            Self::Processor(processor) => Debug::fmt(processor, f),
            Self::Output => f.write_str("Output"),
        }
    }
}

impl GraphNode {
    pub fn new_input() -> Self {
        Self::Input
    }

    pub fn new_processor_pbject(processor: Processor) -> Self {
        Self::Processor(processor)
    }

    pub fn new_processor(processor: impl Process) -> Self {
        Self::Processor(Processor::new(processor))
    }

    pub fn new_output() -> Self {
        Self::Output
    }

    pub fn input_kinds(&self) -> Vec<SignalKind> {
        match self {
            Self::Input => vec![SignalKind::Audio],
            Self::Processor(processor) => processor.input_kinds(),
            Self::Output => vec![SignalKind::Audio],
        }
    }

    pub fn output_kinds(&self) -> Vec<SignalKind> {
        match self {
            Self::Input => vec![SignalKind::Audio],
            Self::Processor(processor) => processor.output_kinds(),
            Self::Output => vec![SignalKind::Audio],
        }
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

    pub fn set_block_size(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        if let Self::Processor(processor) = self {
            processor.set_block_size(audio_rate, control_rate, block_size);
        }
    }

    pub fn reset(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        if let Self::Processor(processor) = self {
            processor.reset(audio_rate, control_rate, block_size);
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
