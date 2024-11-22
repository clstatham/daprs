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
    forward: Arc<dyn RealToComplex<Float>>,
    inverse: Arc<dyn ComplexToReal<Float>>,
    forward_scratch: Fft,
    inverse_scratch: Fft,
}

impl FftPlan {
    pub fn new(fft_length: usize) -> Self {
        let mut plan = realfft::RealFftPlanner::new();
        let forward = plan.plan_fft_forward(fft_length);
        let inverse = plan.plan_fft_inverse(fft_length);
        let forward_scratch = forward.make_scratch_vec().into_boxed_slice();
        let inverse_scratch = inverse.make_scratch_vec().into_boxed_slice();
        Self {
            fft_length,
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
pub struct FftSpec {
    pub name: String,
}

impl FftSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

pub trait FftProcessor: Downcast + Send + FftProcessorClone {
    fn input_spec(&self) -> Vec<FftSpec>;
    fn output_spec(&self) -> Vec<FftSpec>;

    #[allow(unused)]
    fn allocate(&mut self, fft_length: usize) {}

    fn process(
        &mut self,
        fft_length: usize,
        hop_length: usize,
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
        hop_length: usize,
        inputs: &[&Fft],
        outputs: &mut [Fft],
    ) -> Result<(), ProcessorError> {
        if let Self::Processor(proc) = self {
            proc.processor
                .process(fft_length, hop_length, inputs, outputs)
        } else {
            for (input, output) in inputs.iter().zip(outputs.iter_mut()) {
                output.copy_from_slice(input);
            }
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Default, Copy, PartialEq, Eq, Hash)]
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
    pub fn new_endpoint(fft_length: usize, hop_length: usize) -> Self {
        Self::Endpoint {
            ring_buffer: VecDeque::new(),
            overlap_buffer: VecDeque::new(),
            time_domain: FloatBuf(vec![0.0; fft_length].into_boxed_slice()),
            frequency_domain: Fft::new_for_real_length(fft_length),
        }
    }

    pub fn new_processor(fft_length: usize, num_outputs: usize) -> Self {
        let mut buffers = Vec::new();
        for _ in 0..num_outputs {
            buffers.push(Fft(
                vec![Complex::default(); fft_length / 2 + 1].into_boxed_slice()
            ));
        }
        Self::Processor(buffers)
    }

    pub fn resize(
        &mut self,
        fft_length: usize,
        hop_length: usize,
        num_outputs: usize,
        block_size: usize,
    ) {
        match self {
            Self::Endpoint {
                ring_buffer,
                overlap_buffer,
                time_domain,
                frequency_domain,
            } => {
                *ring_buffer = VecDeque::with_capacity(block_size);
                *overlap_buffer = VecDeque::with_capacity(fft_length);
                time_domain.0 = vec![0.0; fft_length].into_boxed_slice();
                frequency_domain.0 =
                    vec![Complex::default(); fft_length / 2 + 1].into_boxed_slice();
            }
            Self::Processor(buffers) => {
                buffers.resize_with(num_outputs, || {
                    Fft(vec![Complex::default(); fft_length / 2 + 1].into_boxed_slice())
                });
            }
        }
    }
}

type FftGraphVisitor = DfsPostOrder<NodeIndex, FxHashSet<NodeIndex>>;

#[derive(Clone, Default)]
pub enum WindowFunction {
    Rectangular,
    #[default]
    Hann,
    Custom(Arc<dyn Fn(usize, usize) -> Float + Send + Sync>),
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
            Self::Custom(f) => {
                for i in 0..length {
                    buf[i] = f(i, length);
                }
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
    window: FloatBuf,

    inputs: Vec<NodeIndex>,
    outputs: Vec<NodeIndex>,

    visitor: FftGraphVisitor,
    visit_path: Vec<NodeIndex>,

    buffer_cache: FxHashMap<NodeIndex, FftNodeBuffers>,
}

impl FftGraph {
    #[track_caller]
    pub fn new(fft_length: usize, hop_length: usize, window_function: WindowFunction) -> Self {
        debug_assert!(
            fft_length.is_power_of_two(),
            "FFT length must be a power of two"
        );
        let plan = FftPlan::new(fft_length);
        let window = window_function.generate(fft_length);
        Self {
            plan,
            hop_length,
            window,
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
                    let buffers = self.buffer_cache.entry(*node_id).or_insert_with(|| {
                        FftNodeBuffers::new_endpoint(self.plan.fft_length, self.hop_length)
                    });
                    buffers.resize(self.plan.fft_length, self.hop_length, 0, block_size);
                }
                FftGraphNode::Processor(proc) => {
                    let buffers = self.buffer_cache.entry(*node_id).or_insert_with(|| {
                        FftNodeBuffers::new_processor(self.plan.fft_length, proc.output_spec.len())
                    });
                    buffers.resize(
                        self.plan.fft_length,
                        self.hop_length,
                        proc.output_spec.len(),
                        block_size,
                    );
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

        while input_buffer_len >= fft_length {
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

                // window the input
                for i in 0..fft_length {
                    time_domain[i] = ring_buffer[i] * self.window[i];
                }

                ring_buffer.drain(0..hop_length);

                self.plan
                    .forward(time_domain.as_mut(), frequency_domain.as_mut())?;
            }

            for i in 0..self.visit_path.len() {
                let node_id = self.visit_path[i];
                self.process_node(node_id)?;
            }

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

                self.plan
                    .inverse(frequency_domain.as_mut(), time_domain.as_mut())?;

                for i in 0..fft_length {
                    let denom =
                        fft_length as Float * 0.5 * (fft_length as Float / hop_length as Float);
                    let sample = time_domain[i] / denom;
                    if i < overlap_buffer.len() {
                        overlap_buffer[i] += sample;
                    } else {
                        overlap_buffer.push_back(sample);
                    }
                }

                ring_buffer.extend(overlap_buffer.drain(..hop_length));
            }

            input_buffer_len -= hop_length;
        }

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

            self.digraph[node_id].process(
                self.plan.fft_length,
                self.hop_length,
                &inputs,
                outputs,
            )?;
        }

        drop(inputs);

        self.buffer_cache.insert(node_id, outputs);

        Ok(())
    }
}

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
        let node = self.graph.add(builtins::FftPhaseVocoder::default());
        node.input(0).connect(self.output(0));
        node.input(1).connect(rhs.output(0));
        node
    }
}

impl Mul for &FftNode {
    type Output = FftNode;

    fn mul(self, rhs: &FftNode) -> Self::Output {
        let node = self.graph.add(builtins::FftPhaseVocoder::default());
        node.input(0).connect(self.output(0));
        node.input(1).connect(rhs.output(0));
        node
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, path::Path};

    use num::complex::Complex64;

    use super::{WindowFunction, PI};

    const SAMPLE_RATE: usize = 48000;
    const BLOCK_SIZE: usize = 512;
    const FFT_SIZE: usize = 512;
    const HOP_SIZE: usize = 128;
    const TEST_LENGTH_SECONDS: usize = 5;
    const TEST_LENGTH_SAMPLES: usize = TEST_LENGTH_SECONDS * SAMPLE_RATE;

    fn generate_audio(len: usize) -> Vec<f64> {
        let mut out = vec![];
        for t in 0..len {
            let t = t as f64 / SAMPLE_RATE as f64;
            out.push((t * 440.0 * 2.0 * PI).sin());
        }
        out
    }

    fn output_audio(audio: &[f64], path: impl AsRef<Path>) {
        let mut writer = hound::WavWriter::new(
            std::fs::File::create(path).unwrap(),
            hound::WavSpec {
                channels: 1,
                sample_rate: SAMPLE_RATE as u32,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
        )
        .unwrap();
        for sample in audio {
            writer.write_sample(*sample as f32).unwrap();
        }
        writer.finalize().unwrap();
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_realtime_stft() {
        let mut planner = realfft::RealFftPlanner::<f64>::new();
        let rfft = planner.plan_fft_forward(FFT_SIZE);
        let irfft = planner.plan_fft_inverse(FFT_SIZE);
        let window = WindowFunction::Hann.generate(FFT_SIZE);

        // Circular streaming buffers for input and output
        let mut input_buffer: VecDeque<f64> = VecDeque::new();
        let mut scratch_buffer: VecDeque<f64> = VecDeque::new();
        let mut output_buffer: VecDeque<f64> = VecDeque::new();

        let audio_input = generate_audio(TEST_LENGTH_SAMPLES);
        let mut audio_output = vec![];

        for block_in in audio_input.chunks(BLOCK_SIZE) {
            let mut block_in = block_in.to_vec();
            if block_in.len() < BLOCK_SIZE {
                block_in.resize(BLOCK_SIZE, 0.0); // Zero pad if necessary
            }

            input_buffer.extend(block_in.iter().cloned());

            let mut block_out = vec![0.0; BLOCK_SIZE];

            // If we have enough samples of input, process as much as we can
            while input_buffer.len() >= FFT_SIZE {
                let mut time_domain = vec![0.0; FFT_SIZE];
                for i in 0..FFT_SIZE {
                    time_domain[i] = input_buffer[i] * window[i];
                }
                input_buffer.drain(0..HOP_SIZE);

                let mut spectrum = vec![Complex64::new(0.0, 0.0); FFT_SIZE / 2 + 1];
                rfft.process(&mut time_domain, &mut spectrum).unwrap();

                irfft.process(&mut spectrum, &mut time_domain).unwrap();

                // Overlap-add
                for i in 0..FFT_SIZE {
                    let denom = FFT_SIZE as f64 * 0.5 * (FFT_SIZE as f64 / HOP_SIZE as f64);
                    let sample = time_domain[i] / denom;
                    if i < scratch_buffer.len() {
                        scratch_buffer[i] += sample;
                    } else {
                        scratch_buffer.push_back(sample);
                    }
                }

                output_buffer.extend(scratch_buffer.drain(0..HOP_SIZE));
            }

            // Output as much as we can
            for i in 0..BLOCK_SIZE {
                if let Some(sample) = output_buffer.pop_front() {
                    block_out[i] = sample;
                }
            }

            // Add the processed block to the output
            audio_output.extend_from_slice(&block_out);
        }

        // Save input and output audio for comparison
        output_audio(&audio_input, "target/test_in.wav");
        output_audio(&audio_output, "target/test_out.wav");
    }
}
