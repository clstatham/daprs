use std::{
    collections::VecDeque,
    ops::{AddAssign, Deref, DerefMut, Mul, MulAssign},
    sync::{Arc, Mutex},
};

use downcast_rs::{impl_downcast, Downcast};
use num::Complex;
use petgraph::prelude::*;
use realfft::{ComplexToReal, RealToComplex};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::prelude::*;

pub mod builtins;

#[derive(Debug, Clone, thiserror::Error)]
pub enum FftError {
    #[error("realfft error: {0}")]
    RealFft(String),
}

impl From<realfft::FftError> for FftError {
    fn from(err: realfft::FftError) -> Self {
        Self::RealFft(err.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct FloatBuf(pub(crate) Box<[Float]>);

impl Deref for FloatBuf {
    type Target = [Float];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FloatBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<[Float]> for FloatBuf {
    fn as_ref(&self) -> &[Float] {
        &self.0
    }
}

impl AsMut<[Float]> for FloatBuf {
    fn as_mut(&mut self) -> &mut [Float] {
        &mut self.0
    }
}

impl AddAssign<Float> for FloatBuf {
    fn add_assign(&mut self, rhs: Float) {
        for x in self.iter_mut() {
            *x += rhs;
        }
    }
}

impl AddAssign<&Self> for FloatBuf {
    fn add_assign(&mut self, rhs: &Self) {
        for (x, y) in self.iter_mut().zip(rhs.iter()) {
            *x += *y;
        }
    }
}

impl MulAssign<Float> for FloatBuf {
    fn mul_assign(&mut self, rhs: Float) {
        for x in self.iter_mut() {
            *x *= rhs;
        }
    }
}

impl MulAssign<&Self> for FloatBuf {
    fn mul_assign(&mut self, rhs: &Self) {
        for (x, y) in self.iter_mut().zip(rhs.iter()) {
            *x *= *y;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fft(pub(crate) Box<[Complex<Float>]>);

impl Fft {
    pub fn new_for_real_length(fft_length: usize) -> Self {
        let complex_length = fft_length / 2 + 1;
        Self(vec![Complex::default(); complex_length].into_boxed_slice())
    }
}

impl Deref for Fft {
    type Target = [Complex<Float>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Fft {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<[Complex<Float>]> for Fft {
    fn as_ref(&self) -> &[Complex<Float>] {
        &self.0
    }
}

impl AsMut<[Complex<Float>]> for Fft {
    fn as_mut(&mut self) -> &mut [Complex<Float>] {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a Fft {
    type Item = &'a Complex<Float>;
    type IntoIter = std::slice::Iter<'a, Complex<Float>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Fft {
    type Item = &'a mut Complex<Float>;
    type IntoIter = std::slice::IterMut<'a, Complex<Float>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl MulAssign<Float> for Fft {
    fn mul_assign(&mut self, rhs: Float) {
        for x in self.iter_mut() {
            *x *= rhs;
        }
    }
}

impl MulAssign<Complex<Float>> for Fft {
    fn mul_assign(&mut self, rhs: Complex<Float>) {
        for x in self.iter_mut() {
            *x *= rhs;
        }
    }
}

impl MulAssign<Self> for Fft {
    fn mul_assign(&mut self, rhs: Self) {
        for (x, y) in self.iter_mut().zip(rhs.iter()) {
            *x *= *y;
        }
    }
}

#[derive(Clone)]
pub struct FftPlan {
    // frequency-domain `Fft` length will be `fft_length / 2 + 1`, as this is an RFFT
    fft_length: usize,
    padded_length: usize,
    forward: Arc<dyn RealToComplex<Float>>,
    inverse: Arc<dyn ComplexToReal<Float>>,
    forward_scratch: Fft,
    inverse_scratch: Fft,
}

impl FftPlan {
    pub fn new(fft_length: usize) -> Self {
        let padded_length = fft_length * 2;
        let mut plan = realfft::RealFftPlanner::new();
        let forward = plan.plan_fft_forward(padded_length);
        let inverse = plan.plan_fft_inverse(padded_length);
        let forward_scratch = forward.make_scratch_vec().into_boxed_slice();
        let inverse_scratch = inverse.make_scratch_vec().into_boxed_slice();
        Self {
            fft_length,
            padded_length,
            forward,
            inverse,
            forward_scratch: Fft(forward_scratch),
            inverse_scratch: Fft(inverse_scratch),
        }
    }

    pub fn real_length(&self) -> usize {
        self.fft_length
    }

    pub fn complex_length(&self) -> usize {
        self.fft_length / 2 + 1
    }

    pub fn padded_length(&self) -> usize {
        self.padded_length
    }

    pub fn forward(
        &mut self,
        input: &mut [Float],
        output: &mut [Complex<Float>],
    ) -> Result<(), FftError> {
        self.forward
            .process_with_scratch(input, output, &mut self.forward_scratch)?;
        Ok(())
    }

    pub fn inverse(
        &mut self,
        input: &mut [Complex<Float>],
        output: &mut [Float],
    ) -> Result<(), FftError> {
        self.inverse
            .process_with_scratch(input, output, &mut self.inverse_scratch)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftSpec {
    pub name: String,
}

impl FftSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait FftProcessor: Downcast + Send + FftProcessorClone {
    fn input_spec(&self) -> Vec<FftSpec>;
    fn output_spec(&self) -> Vec<FftSpec>;

    #[allow(unused)]
    fn allocate(&mut self, fft_length: usize) {}

    fn process(
        &mut self,
        fft_length: usize,
        inputs: &[&Fft],
        outputs: &mut [Fft],
    ) -> Result<(), ProcessorError>;
}

impl_downcast!(FftProcessor);

#[doc(hidden)]
pub trait FftProcessorClone {
    fn clone_box(&self) -> Box<dyn FftProcessor>;
}

impl<T> FftProcessorClone for T
where
    T: 'static + FftProcessor + Clone,
{
    fn clone_box(&self) -> Box<dyn FftProcessor> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn FftProcessor> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftProcessorNode {
    processor: Box<dyn FftProcessor>,
    input_spec: Vec<FftSpec>,
    output_spec: Vec<FftSpec>,
}

impl FftProcessorNode {
    pub fn new(processor: impl FftProcessor) -> Self {
        Self::new_boxed(Box::new(processor))
    }

    pub fn new_boxed(processor: Box<dyn FftProcessor>) -> Self {
        let input_spec = processor.input_spec();
        let output_spec = processor.output_spec();
        Self {
            processor,
            input_spec,
            output_spec,
        }
    }

    pub fn input_spec(&self) -> &[FftSpec] {
        &self.input_spec
    }

    pub fn output_spec(&self) -> &[FftSpec] {
        &self.output_spec
    }

    pub fn allocate(&mut self, fft_length: usize) {
        self.processor.allocate(fft_length);
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FftGraphNode {
    Endpoint,
    Processor(FftProcessorNode),
}

impl FftGraphNode {
    pub fn new_endpoint() -> Self {
        Self::Endpoint
    }

    pub fn new_processor(processor: impl FftProcessor) -> Self {
        Self::Processor(FftProcessorNode::new(processor))
    }

    pub fn allocate(&mut self, fft_length: usize) {
        if let Self::Processor(proc) = self {
            proc.processor.allocate(fft_length);
        }
    }

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

#[derive(Clone, Debug, Default, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftEdge {
    pub source_output: usize,
    pub target_input: usize,
}

#[derive(Clone)]
pub enum FftNodeBuffers {
    Endpoint {
        ring_buffer: VecDeque<Float>,
        overlap_buffer: VecDeque<Float>,
        time_domain: FloatBuf,
        frequency_domain: Fft,
    },
    Processor(Vec<Fft>),
}

impl FftNodeBuffers {
    pub fn new_endpoint(fft_length: usize) -> Self {
        Self::Endpoint {
            ring_buffer: VecDeque::new(),
            overlap_buffer: VecDeque::new(),
            time_domain: FloatBuf(vec![0.0; fft_length * 2].into_boxed_slice()),
            frequency_domain: Fft::new_for_real_length(fft_length * 2),
        }
    }

    pub fn new_processor(fft_length: usize, num_outputs: usize) -> Self {
        let mut buffers = Vec::new();
        for _ in 0..num_outputs {
            buffers.push(Fft(
                vec![Complex::default(); fft_length + 1].into_boxed_slice()
            ));
        }
        Self::Processor(buffers)
    }

    pub fn resize(&mut self, fft_length: usize, num_outputs: usize, block_size: usize) {
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

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WindowFunction {
    Rectangular,
    #[default]
    Hann,
}

impl WindowFunction {
    pub fn generate(&self, length: usize) -> FloatBuf {
        let mut buf = vec![0.0; length].into_boxed_slice();
        match self {
            Self::Rectangular => {
                for x in buf.iter_mut() {
                    *x = 1.0;
                }
            }
            Self::Hann => {
                buf = apodize::hanning_iter(length).collect();
            }
        }
        FloatBuf(buf)
    }
}

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

    pub fn build(self, f: impl FnOnce(&mut FftGraphBuilder)) -> Self {
        let mut builder = FftGraphBuilder {
            graph: Arc::new(Mutex::new(self)),
        };
        f(&mut builder);
        builder.build()
    }

    pub fn fft_length(&self) -> usize {
        self.plan.fft_length
    }

    pub fn hop_length(&self) -> usize {
        self.hop_length
    }

    pub fn overlap_length(&self) -> usize {
        self.plan.fft_length - self.hop_length
    }

    pub fn add_input(&mut self) -> NodeIndex {
        let node = self.digraph.add_node(FftGraphNode::new_endpoint());
        self.inputs.push(node);
        node
    }

    pub fn add_output(&mut self) -> NodeIndex {
        let node = self.digraph.add_node(FftGraphNode::new_endpoint());
        self.outputs.push(node);
        node
    }

    pub fn add(&mut self, processor: impl FftProcessor) -> NodeIndex {
        self.digraph
            .add_node(FftGraphNode::new_processor(processor))
    }

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

    pub fn reset_visitor(&mut self) {
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
                    buffers.resize(self.plan.fft_length, 0, block_size);
                }
                FftGraphNode::Processor(proc) => {
                    let buffers = self.buffer_cache.entry(*node_id).or_insert_with(|| {
                        FftNodeBuffers::new_processor(self.plan.fft_length, proc.output_spec.len())
                    });
                    buffers.resize(self.plan.fft_length, proc.output_spec.len(), block_size);
                }
            }

            let node = self.digraph.node_weight_mut(*node_id).unwrap();
            node.allocate(self.plan.fft_length);
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn process_inner(
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

#[derive(Clone)]
pub struct FftGraphBuilder {
    graph: Arc<Mutex<FftGraph>>,
}

impl Default for FftGraphBuilder {
    fn default() -> Self {
        Self::new(128, 64, WindowFunction::Hann)
    }
}

impl FftGraphBuilder {
    pub fn new(fft_length: usize, hop_length: usize, window_function: WindowFunction) -> Self {
        Self {
            graph: Arc::new(Mutex::new(FftGraph::new(
                fft_length,
                hop_length,
                window_function,
            ))),
        }
    }

    pub fn with_graph<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut FftGraph) -> R,
    {
        let mut graph = self.graph.lock().unwrap();
        f(&mut graph)
    }

    pub fn build(self) -> FftGraph {
        self.with_graph(|graph| graph.clone())
    }

    pub fn add_input(&self) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add_input());
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    pub fn add_output(&self) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add_output());
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    pub fn add(&self, processor: impl FftProcessor) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add(processor));
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    pub fn connect(
        &self,
        source: &FftNode,
        source_output: usize,
        target: &FftNode,
        target_input: usize,
    ) {
        self.with_graph(|graph| {
            graph.connect(source.id(), source_output, target.id(), target_input)
        });
    }
}

#[derive(Clone)]
pub struct FftNode {
    node_id: NodeIndex,
    graph: FftGraphBuilder,
}

impl FftNode {
    pub fn id(&self) -> NodeIndex {
        self.node_id
    }

    pub fn graph(&self) -> FftGraphBuilder {
        self.graph.clone()
    }

    pub fn input(&self, index: usize) -> FftInput {
        FftInput {
            node: self.clone(),
            index,
        }
    }

    pub fn output(&self, index: usize) -> FftOutput {
        FftOutput {
            node: self.clone(),
            index,
        }
    }
}

pub struct FftInput {
    pub node: FftNode,
    pub index: usize,
}

impl FftInput {
    pub fn node(&self) -> FftNode {
        self.node.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn connect(&self, output: FftOutput) {
        self.node.graph.with_graph(|graph| {
            graph.connect(output.node.id(), output.index, self.node.id(), self.index)
        });
    }
}

pub struct FftOutput {
    pub node: FftNode,
    pub index: usize,
}

impl FftOutput {
    pub fn node(&self) -> FftNode {
        self.node.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn connect(&self, input: FftInput) {
        self.node.graph.with_graph(|graph| {
            graph.connect(self.node.id(), self.index, input.node.id(), input.index)
        });
    }
}

impl Mul for FftNode {
    type Output = FftNode;

    fn mul(self, rhs: FftNode) -> Self::Output {
        let node = self.graph.add(builtins::FftConvolve);
        node.input(0).connect(self.output(0));
        node.input(1).connect(rhs.output(0));
        node
    }
}

impl Mul for &FftNode {
    type Output = FftNode;

    fn mul(self, rhs: &FftNode) -> Self::Output {
        let node = self.graph.add(builtins::FftConvolve);
        node.input(0).connect(self.output(0));
        node.input(1).connect(rhs.output(0));
        node
    }
}
