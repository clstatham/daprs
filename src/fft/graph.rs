//! A directed graph of nodes that process FFT signals.

use std::collections::VecDeque;

use num::Complex;
use petgraph::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::prelude::*;

use super::{signal::FloatBuf, FftPlan};

/// A node in an [`FftGraph`] that processes FFT signals.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftProcessorNode {
    processor: Box<dyn FftProcessor>,
    input_spec: Vec<FftSpec>,
    output_spec: Vec<FftSpec>,
}

impl FftProcessorNode {
    /// Creates a new `FftProcessorNode` with the given [`FftProcessor`].
    pub fn new(processor: impl FftProcessor) -> Self {
        Self::new_boxed(Box::new(processor))
    }

    /// Creates a new `FftProcessorNode` with the given boxed [`FftProcessor`].
    pub fn new_boxed(processor: Box<dyn FftProcessor>) -> Self {
        let input_spec = processor.input_spec();
        let output_spec = processor.output_spec();
        Self {
            processor,
            input_spec,
            output_spec,
        }
    }

    /// Returns information about the input signals of the processor.
    pub fn input_spec(&self) -> &[FftSpec] {
        &self.input_spec
    }

    /// Returns information about the output signals of the processor.
    pub fn output_spec(&self) -> &[FftSpec] {
        &self.output_spec
    }

    /// Allocates memory for the processor.
    pub fn allocate(&mut self, fft_length: usize) {
        self.processor.allocate(fft_length);
    }
}

/// A node in an [`FftGraph`].
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FftGraphNode {
    /// An endpoint node that represents an input or output signal.
    Endpoint,
    /// A processor node that processes FFT signals.
    Processor(FftProcessorNode),
}

impl FftGraphNode {
    /// Creates a new endpoint node.
    pub fn new_endpoint() -> Self {
        Self::Endpoint
    }

    /// Creates a new processor node with the given processor.
    pub fn new_processor(processor: impl FftProcessor) -> Self {
        Self::Processor(FftProcessorNode::new(processor))
    }

    /// Allocates memory for the node.
    pub fn allocate(&mut self, fft_length: usize) {
        if let Self::Processor(proc) = self {
            proc.processor.allocate(fft_length);
        }
    }

    /// Processes the input signals and writes the output signals.
    ///
    /// Endpoints will simply copy the input signals to the output signals.
    pub fn process(
        &mut self,
        fft_length: usize,
        inputs: &[&Fft],
        outputs: &mut [Fft],
    ) -> Result<(), ProcessorError> {
        if let Self::Processor(proc) = self {
            proc.processor.process(fft_length, inputs, outputs)
        } else {
            for (input, output) in inputs.iter().zip(outputs.iter_mut()) {
                output.copy_from_slice(input);
            }
            Ok(())
        }
    }
}

/// A connection between two nodes in an [`FftGraph`].
#[derive(Clone, Debug, Default, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftEdge {
    /// The index of the output signal of the source node.
    pub source_output: usize,
    /// The index of the input signal of the target node.
    pub target_input: usize,
}

/// Internal buffer storage for an [`FftNode`] in an [`FftGraph`].
#[derive(Clone)]
pub(crate) enum FftNodeBuffers {
    /// An endpoint node that represents an input or output signal.
    Endpoint {
        /// A ring buffer used as an input/output stream.
        ring_buffer: VecDeque<Float>,
        /// A buffer used to store the overlap between FFT frames.
        overlap_buffer: VecDeque<Float>,
        /// A buffer used to store the time-domain signal.
        time_domain: FloatBuf,
        /// A buffer used to store the frequency-domain signal.
        frequency_domain: Fft,
    },
    /// A processor node that processes FFT signals.
    Processor(
        /// A buffer used to store the frequency-domain output signals of the processor.
        Vec<Fft>,
    ),
}

impl FftNodeBuffers {
    /// Creates a new [`Endpoint`](FftNodeBuffers::Endpoint) node buffer.
    pub fn new_endpoint(fft_length: usize) -> Self {
        Self::Endpoint {
            ring_buffer: VecDeque::new(),
            overlap_buffer: VecDeque::new(),
            time_domain: FloatBuf(vec![0.0; fft_length * 2].into_boxed_slice()),
            frequency_domain: Fft::new_for_real_length(fft_length * 2),
        }
    }

    /// Creates a new [`Processor`](FftNodeBuffers::Processor) node buffer.
    pub fn new_processor(fft_length: usize, num_outputs: usize) -> Self {
        let mut buffers = Vec::new();
        for _ in 0..num_outputs {
            buffers.push(Fft(
                vec![Complex::default(); fft_length + 1].into_boxed_slice()
            ));
        }
        Self::Processor(buffers)
    }

    /// Allocates memory for the buffers based on the given parameters.
    pub fn allocate(&mut self, fft_length: usize, num_outputs: usize, block_size: usize) {
        match self {
            Self::Endpoint {
                ring_buffer,
                overlap_buffer,
                time_domain,
                frequency_domain,
            } => {
                *ring_buffer = VecDeque::with_capacity(block_size);
                *overlap_buffer = VecDeque::with_capacity(fft_length);
                time_domain.0 = vec![0.0; fft_length * 2].into_boxed_slice();
                frequency_domain.0 = vec![Complex::default(); fft_length + 1].into_boxed_slice();
            }
            Self::Processor(buffers) => {
                buffers.resize_with(num_outputs, || {
                    Fft(vec![Complex::default(); fft_length + 1].into_boxed_slice())
                });
            }
        }
    }
}

type FftGraphVisitor = DfsPostOrder<NodeIndex, FxHashSet<NodeIndex>>;

/// A directed graph of nodes that process FFT signals.
#[derive(Clone)]
pub struct FftGraph {
    digraph: StableDiGraph<FftGraphNode, FftEdge>,

    plan: FftPlan,

    hop_length: usize,
    #[allow(unused)]
    window_function: WindowFunction,
    window: FloatBuf,

    inputs: Vec<NodeIndex>,
    outputs: Vec<NodeIndex>,

    visitor: FftGraphVisitor,
    visit_path: Vec<NodeIndex>,

    buffer_cache: FxHashMap<NodeIndex, FftNodeBuffers>,
}

impl Default for FftGraph {
    fn default() -> Self {
        Self::new(256, 64, WindowFunction::Hann)
    }
}

impl FftGraph {
    /// Creates a new, empty `FftGraph` with the given FFT length, hop length, and window function.
    pub fn new(fft_length: usize, hop_length: usize, window_function: WindowFunction) -> Self {
        let plan = FftPlan::new(fft_length);
        let window = window_function.generate(fft_length);
        let window_sum = window.iter().sum::<Float>();
        let window: Box<[Float]> = window.iter().map(|x| x / window_sum).collect();

        Self {
            plan,
            hop_length,
            window: FloatBuf(window),
            window_function,
            digraph: StableDiGraph::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            visitor: FftGraphVisitor::default(),
            visit_path: Vec::new(),
            buffer_cache: FxHashMap::default(),
        }
    }

    /// Constructs an `FftGraph` with a closure and the [`FftGraphBuilder`] API.
    pub fn build(self, f: impl FnOnce(&mut FftGraphBuilder)) -> Self {
        let mut builder = FftGraphBuilder::from_graph(self);
        f(&mut builder);
        builder.build()
    }

    /// Returns the FFT window length of the graph (how many FFT points are used).
    pub fn fft_length(&self) -> usize {
        self.plan.fft_length
    }

    /// Returns the hop length of the graph (the stride between FFT frames).
    pub fn hop_length(&self) -> usize {
        self.hop_length
    }

    /// Returns the overlap length of the graph (how many samples overlap between FFT frames).
    pub fn overlap_length(&self) -> usize {
        self.plan.fft_length - self.hop_length
    }

    /// Adds an input node to the graph and returns its index.
    pub fn add_input(&mut self) -> NodeIndex {
        let node = self.digraph.add_node(FftGraphNode::new_endpoint());
        self.inputs.push(node);
        node
    }

    /// Adds an output node to the graph and returns its index.
    pub fn add_output(&mut self) -> NodeIndex {
        let node = self.digraph.add_node(FftGraphNode::new_endpoint());
        self.outputs.push(node);
        node
    }

    /// Adds a processor node to the graph and returns its index.
    pub fn add(&mut self, processor: impl FftProcessor) -> NodeIndex {
        self.digraph
            .add_node(FftGraphNode::new_processor(processor))
    }

    /// Connects the output of one node to the input of another node.
    ///
    /// If there is already a connection to the target input, it will be replaced.
    pub fn connect(
        &mut self,
        source: NodeIndex,
        source_output: usize,
        target: NodeIndex,
        target_input: usize,
    ) {
        // check if there's already a connection to the target input
        if let Some(edge) = self
            .digraph
            .edges_directed(target, Direction::Incoming)
            .find(|edge| edge.weight().target_input == target_input)
        {
            // remove the existing edge
            self.digraph.remove_edge(edge.id()).unwrap();
        }

        self.digraph.add_edge(
            source,
            target,
            FftEdge {
                source_output,
                target_input,
            },
        );

        self.reset_visitor();
    }

    fn reset_visitor(&mut self) {
        if self.visit_path.capacity() < self.digraph.node_count() {
            self.visit_path = Vec::with_capacity(self.digraph.node_count());
        }
        self.visit_path.clear();
        self.visitor.discovered.clear();
        self.visitor.stack.clear();
        self.visitor.finished.clear();

        for node in self.digraph.externals(Direction::Incoming) {
            self.visitor.stack.push(node);
        }
        while let Some(node) = self.visitor.next(&self.digraph) {
            self.visit_path.push(node);
        }
        self.visit_path.reverse();
    }

    /// Allocates memory for the graph based on the given parameters.
    pub fn allocate(&mut self, block_size: usize) {
        self.reset_visitor();

        for node_id in &self.visit_path {
            let node = self.digraph.node_weight(*node_id).unwrap();
            match node {
                FftGraphNode::Endpoint => {
                    let buffers = self
                        .buffer_cache
                        .entry(*node_id)
                        .or_insert_with(|| FftNodeBuffers::new_endpoint(self.plan.fft_length));
                    buffers.allocate(self.plan.fft_length, 0, block_size);
                }
                FftGraphNode::Processor(proc) => {
                    let buffers = self.buffer_cache.entry(*node_id).or_insert_with(|| {
                        FftNodeBuffers::new_processor(self.plan.fft_length, proc.output_spec.len())
                    });
                    buffers.allocate(self.plan.fft_length, proc.output_spec.len(), block_size);
                }
            }

            let node = self.digraph.node_weight_mut(*node_id).unwrap();
            node.allocate(self.plan.fft_length);
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn process_inner(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let fft_length = self.fft_length();
        let hop_length = self.hop_length();

        let mut input_buffer_len = 0;
        for input_index in 0..self.inputs.len() {
            let buffers = self
                .buffer_cache
                .get_mut(&self.inputs[input_index])
                .unwrap();
            let FftNodeBuffers::Endpoint { ring_buffer, .. } = buffers else {
                unreachable!()
            };

            let input = inputs.input(input_index).unwrap();
            let input = input.as_type::<Float>().unwrap();

            // fill the input buffer
            for i in 0..input.len() {
                ring_buffer.push_back(input[i].unwrap_or_default());
            }

            input_buffer_len = ring_buffer.len();
        }

        // while we still have enough samples to process
        while input_buffer_len >= fft_length {
            // for each input, window the input, perform the FFT, and advance the ring buffer
            for input_index in 0..self.inputs.len() {
                let buffers = self
                    .buffer_cache
                    .get_mut(&self.inputs[input_index])
                    .unwrap();
                let FftNodeBuffers::Endpoint {
                    ring_buffer,
                    time_domain,
                    frequency_domain,
                    ..
                } = buffers
                else {
                    unreachable!()
                };

                // pad the input buffer with zeros
                for i in 0..fft_length {
                    time_domain[i] = ring_buffer[i] * self.window[i];
                }
                for i in fft_length..fft_length * 2 {
                    time_domain[i] = 0.0;
                }

                // perform the FFT
                self.plan
                    .forward(time_domain.as_mut(), frequency_domain.as_mut())?;

                // advance the input buffer
                ring_buffer.drain(..hop_length);
            }

            // we just consumed `hop_length` samples from each input buffer
            input_buffer_len -= hop_length;

            // run the FFT processor nodes
            for i in 0..self.visit_path.len() {
                let node_id = self.visit_path[i];
                self.process_node(node_id)?;
            }

            // for each output, perform the IFFT, overlap-add, and write to the output's ring buffer
            for output_index in 0..self.outputs.len() {
                let buffers = self
                    .buffer_cache
                    .get_mut(&self.outputs[output_index])
                    .unwrap();
                let FftNodeBuffers::Endpoint {
                    frequency_domain,
                    overlap_buffer,
                    time_domain,
                    ring_buffer,
                } = buffers
                else {
                    unreachable!()
                };

                // perform the IFFT
                self.plan
                    .inverse(frequency_domain.as_mut(), time_domain.as_mut())?;

                // overlap-add
                for i in 0..fft_length * 2 {
                    let denom = 1.0;
                    let sample = time_domain[i] / denom;
                    if i < overlap_buffer.len() {
                        overlap_buffer[i] += sample;
                    } else {
                        overlap_buffer.push_back(sample);
                    }
                }

                time_domain.fill(0.0);

                // write to the output's ring buffer
                ring_buffer.extend(overlap_buffer.drain(..hop_length));
            }
        }

        // for each output, write as much of the output's ring buffer as possible to the block's corresponding output buffer
        for output_index in 0..self.outputs.len() {
            let buffers = self
                .buffer_cache
                .get_mut(&self.outputs[output_index])
                .unwrap();
            let FftNodeBuffers::Endpoint { ring_buffer, .. } = buffers else {
                unreachable!()
            };

            let mut output = outputs.output(output_index);

            for i in 0..inputs.block_size() {
                if let Some(sample) = ring_buffer.pop_front() {
                    output.set_as(i, sample);
                } else {
                    output.set_as(i, 0.0);
                }
            }
        }

        Ok(())
    }

    fn process_node(&mut self, node_id: NodeIndex) -> Result<(), ProcessorError> {
        let mut inputs = smallvec::SmallVec::<[&Fft; 4]>::new();
        let mut outputs = self.buffer_cache.remove(&node_id).unwrap();

        for (source, edge) in self
            .digraph
            .edges_directed(node_id, Direction::Incoming)
            .map(|e| (e.source(), e.weight()))
        {
            let source_buffers = self.buffer_cache.get(&source).unwrap();
            match source_buffers {
                FftNodeBuffers::Endpoint {
                    frequency_domain: current_fft,
                    ..
                } => {
                    inputs.push(current_fft);
                }
                FftNodeBuffers::Processor(buffers) => {
                    inputs.push(&buffers[edge.source_output]);
                }
            }
        }

        {
            let outputs = match &mut outputs {
                FftNodeBuffers::Endpoint {
                    frequency_domain: current_fft,
                    ..
                } => std::slice::from_mut(current_fft),
                FftNodeBuffers::Processor(buffers) => buffers.as_mut(),
            };

            self.digraph[node_id].process(self.plan.fft_length, &inputs, outputs)?;
        }

        drop(inputs);

        self.buffer_cache.insert(node_id, outputs);

        Ok(())
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct FftGraphData {
        graph: StableDiGraph<FftGraphNode, FftEdge>,
        fft_length: usize,
        hop_length: usize,
        window_function: WindowFunction,
        inputs: Vec<NodeIndex>,
        outputs: Vec<NodeIndex>,
    }

    impl Serialize for FftGraph {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let data = FftGraphData {
                graph: self.digraph.clone(),
                fft_length: self.plan.fft_length,
                hop_length: self.hop_length,
                window_function: self.window_function.clone(),
                inputs: self.inputs.clone(),
                outputs: self.outputs.clone(),
            };
            data.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for FftGraph {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let data = FftGraphData::deserialize(deserializer)?;
            Ok(Self {
                digraph: data.graph,
                inputs: data.inputs,
                outputs: data.outputs,
                ..Self::new(data.fft_length, data.hop_length, data.window_function)
            })
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for FftGraph {
    fn input_spec(&self) -> Vec<SignalSpec> {
        let mut specs = Vec::new();
        for i in 0..self.inputs.len() {
            specs.push(SignalSpec::new(i.to_string(), SignalType::Float));
        }
        specs
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        let mut specs = Vec::new();
        for i in 0..self.outputs.len() {
            specs.push(SignalSpec::new(i.to_string(), SignalType::Float));
        }
        specs
    }

    fn allocate(&mut self, _sample_rate: Float, max_block_size: usize) {
        self.allocate(max_block_size);
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        self.process_inner(inputs, outputs)
    }
}
