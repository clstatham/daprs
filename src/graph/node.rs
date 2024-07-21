use std::fmt::Debug;

use crate::sample::{Buffer, SignalKind};

/// A trait for processing audio samples.
///
/// This is usually used as part of a [`Node`], operating on its internal input/output buffers.
pub trait Process: 'static + Send + Sync + ProcessClone {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn input_kind(&self) -> SignalKind;
    fn output_kind(&self) -> SignalKind;

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
    pub fn new(processor: impl Process) -> Self {
        Self {
            input_buffers: vec![Buffer::zeros(0, processor.input_kind()); processor.num_inputs()]
                .into_boxed_slice(),
            output_buffers: vec![
                Buffer::zeros(0, processor.output_kind());
                processor.num_outputs()
            ]
            .into_boxed_slice(),
            processor: Box::new(processor),
        }
    }

    pub fn input_kind(&self) -> SignalKind {
        self.processor.input_kind()
    }

    pub fn output_kind(&self) -> SignalKind {
        self.processor.output_kind()
    }

    pub fn set_block_size(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        let input_kind = self.input_kind();
        let output_kind = self.output_kind();
        let control_block_size = (block_size as f64 * control_rate / audio_rate).ceil() as usize;

        for input in self.input_buffers.iter_mut() {
            if input_kind == SignalKind::Control {
                input.resize(control_block_size);
            } else {
                input.resize(block_size);
            }
        }
        for output in self.output_buffers.iter_mut() {
            if output_kind == SignalKind::Control {
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

#[derive(Debug, Clone)]
pub enum GraphNode {
    Input,
    Processor(Processor),
    Output,
}

impl GraphNode {
    pub fn new_input() -> Self {
        Self::Input
    }

    pub fn new_processor(processor: impl Process) -> Self {
        Self::Processor(Processor::new(processor))
    }

    pub fn new_output() -> Self {
        Self::Output
    }

    pub fn input_kind(&self) -> SignalKind {
        match self {
            Self::Input => SignalKind::Audio,
            Self::Processor(processor) => processor.input_kind(),
            Self::Output => SignalKind::Audio,
        }
    }

    pub fn output_kind(&self) -> SignalKind {
        match self {
            Self::Input => SignalKind::Audio,
            Self::Processor(processor) => processor.output_kind(),
            Self::Output => SignalKind::Audio,
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
