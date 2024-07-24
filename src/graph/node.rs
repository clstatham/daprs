use std::fmt::Debug;

use crate::sample::{Buffer, SignalRate};

/// A trait for processing audio samples.
///
/// This is usually used as part of a [`GraphNode`], operating on its internal input/output buffers.
pub trait Process: 'static + Send + Sync + ProcessClone {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Returns the expected [`SignalRate`]s of the inputs of this [`Process`].
    fn input_rates(&self) -> Vec<SignalRate>;

    /// Returns the [`SignalRate`]s of the outputs this [`Process`] produces.
    fn output_rates(&self) -> Vec<SignalRate>;

    /// Returns the number of input buffers/channels this [`Process`] expects.
    fn num_inputs(&self) -> usize {
        self.input_rates().len()
    }

    /// Returns the number of output buffers/channels this [`Process`] produces.
    fn num_outputs(&self) -> usize {
        self.output_rates().len()
    }

    /// Called before the first [`Process::process`] call, and anytime the graph changes.
    fn prepare(&mut self) {}

    /// Called whenever the runtime's sample rates or block size change.
    #[allow(unused)]
    fn reset(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {}

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
    /// Creates a new [`Processor`] from the given [`Process`] object.
    pub fn new(processor: impl Process) -> Self {
        Self::new_from_boxed(Box::new(processor))
    }

    /// Creates a new [`Processor`] from the given boxed [`Process`] object.
    pub fn new_from_boxed(processor: Box<dyn Process>) -> Self {
        let mut input_buffers = Vec::with_capacity(processor.num_inputs());
        for rate in processor.input_rates() {
            input_buffers.push(Buffer::zeros(0, rate));
        }
        let mut output_buffers = Vec::with_capacity(processor.num_outputs());
        for rate in processor.output_rates() {
            output_buffers.push(Buffer::zeros(0, rate));
        }

        Self {
            input_buffers: input_buffers.into_boxed_slice(),
            output_buffers: output_buffers.into_boxed_slice(),
            processor,
        }
    }

    /// Returns the expected [`SignalRate`]s of the inputs of this [`Processor`].
    pub fn input_rates(&self) -> Vec<SignalRate> {
        self.processor.input_rates()
    }

    /// Returns the [`SignalRate`]s of the outputs this [`Processor`] produces.
    pub fn output_rates(&self) -> Vec<SignalRate> {
        self.processor.output_rates()
    }

    /// Reallocates the input and output buffers to match the given sample rates and block size.
    pub fn reset(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        let control_block_size = (block_size as f64 * control_rate / audio_rate).ceil() as usize;

        for input in self.input_buffers.iter_mut() {
            if input.rate() == SignalRate::Control {
                input.resize(control_block_size);
            } else {
                input.resize(block_size);
            }
        }
        for output in self.output_buffers.iter_mut() {
            if output.rate() == SignalRate::Control {
                output.resize(control_block_size);
            } else {
                output.resize(block_size);
            }
        }
        self.processor.reset(audio_rate, control_rate, block_size);
    }

    /// Returns a slice of the input buffers.
    #[inline]
    pub fn inputs(&self) -> &[Buffer] {
        &self.input_buffers[..]
    }

    /// Returns a mutable slice of the input buffers.
    #[inline]
    pub fn inputs_mut(&mut self) -> &mut [Buffer] {
        &mut self.input_buffers[..]
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
        &self.output_buffers[..]
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
        self.processor
            .process(&self.input_buffers, &mut self.output_buffers);
    }
}

/// A node in the audio graph. This can be an input, processor, or output node.
#[derive(Clone)]
pub enum GraphNode {
    /// A passthrough node that simply forwards its input to its output.
    Passthrough(Buffer),
    /// A processor node that processes its input buffers and writes the results to its output buffers.
    Processor(Processor),
}

impl Debug for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passthrough(_) => f.write_str("Passthrough"),
            Self::Processor(processor) => Debug::fmt(processor, f),
        }
    }
}

impl GraphNode {
    /// Creates a new input node.
    pub fn new_input() -> Self {
        Self::Passthrough(Buffer::zeros(0, SignalRate::Audio))
    }

    /// Creates a new processor node from the given [`Processor`] object.
    pub fn new_processor_object(processor: Processor) -> Self {
        Self::Processor(processor)
    }

    /// Creates a new processor node from the given [`Process`] object.
    pub fn new_processor(processor: impl Process) -> Self {
        Self::Processor(Processor::new(processor))
    }

    /// Creates a new output node.
    pub fn new_output() -> Self {
        Self::Passthrough(Buffer::zeros(0, SignalRate::Audio))
    }

    /// Returns the expected [`SignalRate`]s of the inputs of this [`GraphNode`].
    pub fn input_rates(&self) -> Vec<SignalRate> {
        match self {
            Self::Passthrough(_) => vec![SignalRate::Audio],
            Self::Processor(processor) => processor.input_rates(),
        }
    }

    /// Returns the [`SignalRate`]s of the outputs this [`GraphNode`] produces.
    pub fn output_rates(&self) -> Vec<SignalRate> {
        match self {
            Self::Passthrough(_) => vec![SignalRate::Audio],
            Self::Processor(processor) => processor.output_rates(),
        }
    }

    /// Returns the name of the processor in this [`GraphNode`].
    pub fn name(&self) -> &str {
        match self {
            Self::Passthrough(_) => "Passthrough",
            Self::Processor(processor) => processor.processor.name(),
        }
    }

    /// Returns a slice of the input buffers of this [`GraphNode`].
    pub fn inputs(&self) -> &[Buffer] {
        match self {
            Self::Passthrough(buffer) => std::slice::from_ref(buffer),
            Self::Processor(processor) => processor.inputs(),
        }
    }

    /// Returns a mutable slice of the input buffers of this [`GraphNode`].
    pub fn inputs_mut(&mut self) -> &mut [Buffer] {
        match self {
            Self::Passthrough(buffer) => std::slice::from_mut(buffer),
            Self::Processor(processor) => processor.inputs_mut(),
        }
    }

    /// Returns a slice of the output buffers of this [`GraphNode`].
    pub fn outputs(&self) -> &[Buffer] {
        match self {
            Self::Passthrough(buffer) => std::slice::from_ref(buffer),
            Self::Processor(processor) => processor.outputs(),
        }
    }

    /// Reallocates the input and output buffers to match the given sample rates and block size.
    pub fn reset(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        match self {
            Self::Passthrough(buffer) => buffer.resize(block_size),
            Self::Processor(processor) => processor.reset(audio_rate, control_rate, block_size),
        }
    }

    /// Prepares the processor for processing. This is called before the first [`GraphNode::process`] call, and anytime the graph changes.
    pub fn prepare(&mut self) {
        if let Self::Processor(processor) = self {
            processor.prepare();
        }
    }

    /// Processes the node's input buffers and writes the results to the node's output buffers.
    /// This is a no-op for passthrough nodes.
    pub fn process(&mut self) {
        if let Self::Processor(processor) = self {
            processor.process();
        }
    }
}
