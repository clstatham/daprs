use std::fmt::Debug;

use crate::signal::{Signal, SignalKind, SignalRate, SignalSpec};

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

/// A node in the audio graph.
#[derive(Clone)]
pub enum GraphNode {
    /// A passthrough node that simply forwards its input to its output.
    Passthrough(Signal),
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
        Self::Passthrough(Signal::default_for_spec(SignalSpec {
            name: Some("input"),
            rate: SignalRate::Audio,
            kind: SignalKind::Buffer,
        }))
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
        Self::Passthrough(Signal::default_for_spec(SignalSpec {
            name: Some("output"),
            rate: SignalRate::Audio,
            kind: SignalKind::Buffer,
        }))
    }

    /// Returns information about the inputs this [`GraphNode`] expects.
    pub fn input_spec(&self) -> Vec<SignalSpec> {
        match self {
            Self::Passthrough(sig) => vec![sig.spec],
            Self::Processor(processor) => processor.input_spec(),
        }
    }

    /// Returns information about the outputs this [`GraphNode`] produces.
    pub fn output_spec(&self) -> Vec<SignalSpec> {
        match self {
            Self::Passthrough(sig) => vec![sig.spec],
            Self::Processor(processor) => processor.output_spec(),
        }
    }

    /// Returns the name of the processor in this [`GraphNode`].
    pub fn name(&self) -> &str {
        match self {
            Self::Passthrough(signal) => signal.name().unwrap_or("passthrough"),
            Self::Processor(processor) => processor.processor.name(),
        }
    }

    /// Returns a slice of the input buffers of this [`GraphNode`].
    pub fn inputs(&self) -> &[Signal] {
        match self {
            Self::Passthrough(signal) => std::slice::from_ref(signal),
            Self::Processor(processor) => processor.inputs(),
        }
    }

    /// Returns a mutable slice of the input buffers of this [`GraphNode`].
    pub fn inputs_mut(&mut self) -> &mut [Signal] {
        match self {
            Self::Passthrough(signal) => std::slice::from_mut(signal),
            Self::Processor(processor) => processor.inputs_mut(),
        }
    }

    /// Returns a slice of the output buffers of this [`GraphNode`].
    pub fn outputs(&self) -> &[Signal] {
        match self {
            Self::Passthrough(signal) => std::slice::from_ref(signal),
            Self::Processor(processor) => processor.outputs(),
        }
    }

    /// Resizes the input and output buffers to match the given sample rates and block size.
    pub fn resize_buffers(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        match self {
            Self::Passthrough(signal) => signal.resize_buffers(block_size),
            Self::Processor(processor) => {
                processor.resize_buffers(audio_rate, control_rate, block_size)
            }
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
