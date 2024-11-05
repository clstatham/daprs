//! The audio graph processing runtime.

use std::{sync::mpsc, time::Duration};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use petgraph::prelude::*;

use crate::{
    graph::{edge::Edge, node::GraphNode, Graph, GraphRunError, NodeIndex},
    processor::ProcessorError,
    signal::{Sample, SignalBuffer},
};

/// An error that occurred during runtime operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[error("Runtime error")]
pub enum RuntimeError {
    /// An error occurred while accessing the audio stream.
    StreamError(#[from] cpal::StreamError),

    /// An error occurred while accessing audio devices.
    DevicesError(#[from] cpal::DevicesError),

    /// An error occurred while reading or writing a WAV file.
    Hound(#[from] hound::Error),

    /// An error occurred during audio host configuration (host unavailable).
    HostUnavailable(#[from] cpal::HostUnavailable),

    /// An error occurred during audio device configuration (device unavailable).
    #[error("Requested device is unavailable: {0:?}")]
    DeviceUnavailable(Device),

    /// An error occurred during audio device configuration (error getting the device's name).
    DeviceNameError(#[from] cpal::DeviceNameError),

    /// An error occurred during audio device configuration (error getting the default device stream configuration).
    DefaultStreamConfigError(#[from] cpal::DefaultStreamConfigError),

    /// An error occurred during audio device configuration (invalid sample format).
    #[error("Unsupported sample format: {0}")]
    UnsupportedSampleFormat(cpal::SampleFormat),

    /// An error occurred during graph processing.
    GraphRunError(#[from] GraphRunError),

    /// The runtime needs to be reset before processing.
    NeedsReset,

    /// An error occurred during processing.
    ProcessorError(#[from] ProcessorError),

    /// The number of channels in the audio graph does not match the number of channels in the audio device.
    #[error("Channel mismatch: expected {0} channels, got {1}")]
    ChannelMismatch(usize, usize),
}

/// A result type for runtime operations.
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// The audio backend to use for the runtime.
#[derive(Default, Debug, Clone)]
pub enum Backend {
    #[default]
    /// Default audio backend for the current platform.
    Default,
    #[cfg(all(target_os = "linux", feature = "jack"))]
    /// JACK Audio Connection Kit
    Jack,
    #[cfg(target_os = "linux")]
    /// Advanced Linux Sound Architecture
    Alsa,
    #[cfg(target_os = "windows")]
    /// Windows Audio Session API
    Wasapi,
}

/// The audio device to use for the runtime.
#[derive(Default, Debug, Clone)]
pub enum Device {
    /// Use the default audio device as returned by [`cpal::Host::default_output_device`].
    #[default]
    Default,
    /// Use the audio device at the given index.
    Index(usize),
    /// Substring of the device name to match. The first device with a name containing this substring will be used.
    Name(String),
}

#[derive(Clone)]
pub(crate) struct NodeBuffers {
    inputs: Vec<SignalBuffer>,
    outputs: Vec<SignalBuffer>,
}

/// The audio graph processing runtime.
///
/// The runtime is responsible for running the audio graph and rendering audio samples.
/// It can run in real-time ([`run`](Runtime::run)) or offline ([`run_offline`](Runtime::run_offline)) mode.
///
/// In real-time mode, the runtime will render audio samples in real-time using a specified audio backend and device.
///
/// In offline mode, the runtime will render audio samples as fast as possible and return the rendered output channels.
#[derive(Clone, Default)]
pub struct Runtime {
    graph: Graph,
    buffer_cache: hashbrown::HashMap<NodeIndex, NodeBuffers, rustc_hash::FxBuildHasher>,
    inputs: Vec<SignalBuffer>,
    outputs: Vec<SignalBuffer>,
    input_ids: Vec<NodeIndex>,
    output_ids: Vec<NodeIndex>,
    // cached internal state to avoid allocations in `process()`
    edge_cache: Vec<(NodeIndex, Edge)>,
    // cached visitor state for graph traversal
    visit_path: Vec<NodeIndex>,
}

impl Runtime {
    /// Creates a new runtime with the given audio graph.
    pub fn new(graph: Graph) -> Self {
        Runtime {
            inputs: Vec::with_capacity(graph.num_inputs()),
            outputs: Vec::with_capacity(graph.num_outputs()),
            input_ids: graph.input_indices().to_vec(),
            output_ids: graph.output_indices().to_vec(),
            buffer_cache: hashbrown::HashMap::default(),
            graph,
            edge_cache: Vec::new(),
            visit_path: Vec::new(),
        }
    }

    /// Resets the runtime with the given sample rate and block size.
    /// This will reset the state of all nodes in the graph and potentially reallocate internal buffers.
    pub fn reset(&mut self, sample_rate: f64, block_size: usize) -> RuntimeResult<()> {
        self.graph.allocate_visitor();

        let mut max_edges = 0;

        // allocate buffers
        self.inputs = vec![SignalBuffer::new_sample(block_size); self.graph.num_inputs()];
        self.outputs = vec![SignalBuffer::new_sample(block_size); self.graph.num_outputs()];

        self.graph.visit(|graph, node_id| -> RuntimeResult<()> {
            let node = &mut graph.digraph_mut()[node_id];
            let input_spec = node.input_spec();
            let output_spec = node.output_spec();

            node.resize_buffers(sample_rate, block_size);

            let mut inputs = Vec::with_capacity(input_spec.len());
            let mut outputs = Vec::with_capacity(output_spec.len());

            for spec in input_spec {
                let buffer = SignalBuffer::from_spec_default(&spec, block_size);
                inputs.push(buffer);
            }

            for spec in output_spec {
                let buffer = SignalBuffer::from_spec_default(&spec, block_size);
                outputs.push(buffer);
            }

            self.buffer_cache
                .insert(node_id, NodeBuffers { inputs, outputs });

            let num_inputs = graph
                .digraph()
                .edges_directed(node_id, Direction::Incoming)
                .count();
            max_edges = max_edges.max(num_inputs);

            Ok(())
        })?;

        // allocate edge cache
        self.edge_cache = Vec::with_capacity((max_edges * 2).next_power_of_two());

        // populate visitor path
        self.visit_path = Vec::with_capacity(self.graph.digraph().node_count());
        self.visit_path.clear();
        let mut visitor = DfsPostOrder::empty(self.graph.digraph());
        for node in self.graph.digraph().externals(Direction::Incoming) {
            visitor.stack.push(node);
        }
        while let Some(node) = visitor.next(&self.graph.digraph()) {
            self.visit_path.push(node);
        }
        self.visit_path.reverse();

        Ok(())
    }

    /// Runs the preparation phase for every node in the graph.
    pub fn prepare(&mut self) -> RuntimeResult<()> {
        self.graph.prepare()?;
        Ok(())
    }

    /// Returns a reference to the audio graph.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Returns a mutable reference to the audio graph.
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    /// Returns an iterator over the output channels of the runtime.
    pub fn outputs(&mut self) -> impl Iterator<Item = &SignalBuffer> + '_ {
        let num_outputs = self.graph.num_outputs();
        (0..num_outputs).map(|i| &self.outputs[i])
    }

    /// Renders the next block of audio.
    #[inline]
    pub fn process(&mut self) -> RuntimeResult<()> {
        self.graph.visit(|graph, node_id| -> RuntimeResult<()> {
            self.edge_cache.extend(
                graph
                    .digraph()
                    .edges_directed(node_id, Direction::Incoming)
                    .map(|edge| (edge.source(), *edge.weight())),
            );

            for (source_id, edge) in self.edge_cache.drain(..) {
                let Edge {
                    source_output,
                    target_input,
                } = edge;

                let (source, target) = graph.digraph_mut().index_twice_mut(source_id, node_id);

                let (source_buffer, target_buffer) = match (source, target) {
                    (GraphNode::Passthrough, GraphNode::Passthrough) => {
                        let input_idx = self
                            .input_ids
                            .iter()
                            .position(|&id| id == source_id)
                            .unwrap();
                        let output_idx = self
                            .output_ids
                            .iter()
                            .position(|&id| id == node_id)
                            .unwrap();
                        (&self.inputs[input_idx], &mut self.outputs[output_idx])
                    }
                    (GraphNode::Passthrough, GraphNode::Processor(_)) => {
                        let buffers = self.buffer_cache.get_mut(&node_id).unwrap();
                        (
                            &self.inputs[source_output as usize],
                            &mut buffers.inputs[target_input as usize],
                        )
                    }
                    (GraphNode::Processor(_), GraphNode::Passthrough) => {
                        let buffers = &self.buffer_cache[&source_id];
                        let out_idx = self
                            .output_ids
                            .iter()
                            .position(|&id| id == node_id)
                            .unwrap();
                        (
                            &buffers.outputs[source_output as usize],
                            &mut self.outputs[out_idx],
                        )
                    }
                    (GraphNode::Processor(_), GraphNode::Processor(_)) => {
                        let [source_buffer, target_buffer] =
                            self.buffer_cache.get_many_mut([&source_id, &node_id]);

                        let source_buffer = source_buffer.unwrap();
                        let target_buffer = target_buffer.unwrap();

                        (
                            &source_buffer.outputs[source_output as usize],
                            &mut target_buffer.inputs[target_input as usize],
                        )
                    }
                };

                target_buffer.copy_from(source_buffer);
            }

            let buffers = self.buffer_cache.get_mut(&node_id).unwrap();

            // process node
            graph.digraph_mut()[node_id]
                .process(&buffers.inputs, &mut buffers.outputs)
                .map_err(|e| GraphRunError {
                    node_index: node_id,
                    node_processor: graph.digraph()[node_id].name().to_string(),
                    kind: crate::graph::GraphRunErrorKind::ProcessorError(e),
                })?;

            Ok(())
        })?;

        Ok(())
    }

    #[inline]
    pub fn get_node_input_mut(&mut self, node: NodeIndex, input_index: usize) -> &mut SignalBuffer {
        &mut self.buffer_cache.get_mut(&node).unwrap().inputs[input_index]
    }

    #[inline]
    pub fn get_node_output(&self, node: NodeIndex, output_index: usize) -> &SignalBuffer {
        &self.buffer_cache.get(&node).unwrap().outputs[output_index]
    }

    #[inline]
    pub fn get_input_mut(&mut self, input_index: usize) -> &mut SignalBuffer {
        &mut self.inputs[input_index]
    }

    #[inline]
    pub fn get_output(&self, output_index: usize) -> &SignalBuffer {
        &self.outputs[output_index]
    }

    /// Runs the audio graph as fast as possible for the given duration's worth of samples, and returns the rendered output channels.
    pub fn run_offline(
        &mut self,
        duration: Duration,
        sample_rate: f64,
        block_size: usize,
    ) -> RuntimeResult<Box<[Box<[Sample]>]>> {
        self.run_offline_inner(duration, sample_rate, block_size, false)
    }

    /// Simulates the audio graph running for the given duration's worth of samples, and returns the rendered output channels.
    ///
    /// This method will add a delay between each block of samples to simulate real-time processing.
    pub fn simulate(
        &mut self,
        duration: Duration,
        sample_rate: f64,
        block_size: usize,
    ) -> RuntimeResult<Box<[Box<[Sample]>]>> {
        self.run_offline_inner(duration, sample_rate, block_size, true)
    }

    fn run_offline_inner(
        &mut self,
        duration: Duration,
        sample_rate: f64,
        block_size: usize,
        add_delay: bool,
    ) -> RuntimeResult<Box<[Box<[Sample]>]>> {
        let secs = duration.as_secs_f64();
        let samples = (sample_rate * secs) as usize;

        self.reset(sample_rate, block_size)?;
        self.prepare()?;

        let num_outputs: usize = self.graph.num_outputs();

        let mut outputs: Box<[Box<[Sample]>]> =
            vec![vec![Sample::new(0.0); samples].into_boxed_slice(); num_outputs]
                .into_boxed_slice();

        let mut sample_count = 0;
        let mut last_block_size = 0;

        while sample_count < samples {
            let actual_block_size = (samples - sample_count).min(block_size);
            if actual_block_size != last_block_size {
                self.graph.resize_buffers(sample_rate, actual_block_size)?;
                last_block_size = actual_block_size;
            }
            self.process()?;

            for (i, output) in outputs.iter_mut().enumerate() {
                let buffer = &self.outputs[i];
                let SignalBuffer::Sample(buffer) = buffer else {
                    return Err(RuntimeError::ProcessorError(
                        ProcessorError::OutputSpecMismatch(i),
                    ));
                };
                output[sample_count..sample_count + actual_block_size].copy_from_slice(buffer);
            }

            if add_delay {
                std::thread::sleep(Duration::from_secs_f64(
                    actual_block_size as f64 / sample_rate,
                ));
            }

            sample_count += actual_block_size;
        }

        Ok(outputs)
    }

    /// Runs the audio graph as fast as possible for the given duration's worth of samples, and writes the rendered output channels to a WAV file.
    pub fn run_offline_to_file(
        &mut self,
        file_path: impl AsRef<std::path::Path>,
        duration: Duration,
        sample_rate: f64,
        block_size: usize,
    ) -> RuntimeResult<()> {
        let outputs = self.run_offline(duration, sample_rate, block_size)?;

        let num_channels = outputs.len();

        if num_channels == 0 {
            log::warn!("No output channels to write to file");
            return Ok(());
        }

        let num_samples = outputs[0].len();

        let mut samples = vec![0.0; num_samples * num_channels];

        for sample_index in 0..num_samples {
            for channel_index in 0..num_channels {
                let i = sample_index * num_channels + channel_index;
                samples[i] = *outputs[channel_index][sample_index];
            }
        }

        let spec = hound::WavSpec {
            channels: num_channels as u16,
            sample_rate: sample_rate as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = hound::WavWriter::create(file_path, spec)?;

        for sample in samples {
            writer.write_sample(sample as f32)?;
        }

        writer.finalize()?;

        Ok(())
    }

    /// Runs the audio graph in real-time for the given [`Duration`] using the specified audio backend and device.
    pub fn run_for(
        &mut self,
        duration: Duration,
        backend: Backend,
        device: Device,
    ) -> RuntimeResult<()> {
        let handle = self.run(backend, device)?;
        std::thread::sleep(duration);
        handle.stop();
        Ok(())
    }

    /// Runs the audio graph in real-time using the specified audio backend and device.
    pub fn run(&mut self, backend: Backend, device: Device) -> RuntimeResult<RuntimeHandle> {
        let (kill_tx, kill_rx) = mpsc::channel();

        let handle = RuntimeHandle { kill_tx };

        let host_id = match backend {
            Backend::Default => cpal::default_host().id(),
            #[cfg(target_os = "linux")]
            Backend::Alsa => cpal::available_hosts()
                .into_iter()
                .find(|h| *h == cpal::HostId::Alsa)
                .ok_or_else(|| RuntimeError::HostUnavailable(cpal::HostUnavailable))?,
            #[cfg(all(target_os = "linux", feature = "jack"))]
            Backend::Jack => cpal::available_hosts()
                .into_iter()
                .find(|h| *h == cpal::HostId::Jack)
                .ok_or_else(|| RuntimeError::HostUnavailable(cpal::HostUnavailable))?,
            #[cfg(target_os = "windows")]
            Backend::Wasapi => cpal::available_hosts()
                .into_iter()
                .find(|h| *h == cpal::HostId::Wasapi)
                .ok_or_else(|| RuntimeError::HostUnavailable(cpal::HostUnavailable))?,
        };
        let host = cpal::host_from_id(host_id)?;

        log::info!("Using host: {:?}", host.id());

        let cpal_device = match &device {
            Device::Default => host.default_output_device(),
            Device::Index(index) => host.output_devices().unwrap().nth(*index),
            Device::Name(name) => host
                .output_devices()
                .unwrap()
                .find(|d| d.name().unwrap().contains(name)),
        };

        let device = cpal_device.ok_or_else(|| RuntimeError::DeviceUnavailable(device))?;

        log::info!("Using device: {}", device.name()?);

        let config = device.default_output_config()?;

        let channels = config.channels();
        if self.graph.num_outputs() != channels as usize {
            return Err(RuntimeError::ChannelMismatch(
                self.graph.num_outputs(),
                channels as usize,
            ));
        }

        log::info!("Configuration: {:#?}", config);

        let audio_rate = config.sample_rate().0 as f64;
        let initial_block_size = audio_rate as usize / 100;

        self.reset(audio_rate, initial_block_size)?;

        self.prepare()?;

        let runtime = self.clone();

        std::thread::spawn(move || -> RuntimeResult<()> {
            let stream = match config.sample_format() {
                cpal::SampleFormat::I8 => runtime.run_inner::<i8>(&device, &config.config())?,
                cpal::SampleFormat::I16 => runtime.run_inner::<i16>(&device, &config.config())?,
                cpal::SampleFormat::I32 => runtime.run_inner::<i32>(&device, &config.config())?,
                cpal::SampleFormat::I64 => runtime.run_inner::<i64>(&device, &config.config())?,
                cpal::SampleFormat::U8 => runtime.run_inner::<u8>(&device, &config.config())?,
                cpal::SampleFormat::U16 => runtime.run_inner::<u16>(&device, &config.config())?,
                cpal::SampleFormat::U32 => runtime.run_inner::<u32>(&device, &config.config())?,
                cpal::SampleFormat::U64 => runtime.run_inner::<u64>(&device, &config.config())?,
                cpal::SampleFormat::F32 => runtime.run_inner::<f32>(&device, &config.config())?,
                cpal::SampleFormat::F64 => runtime.run_inner::<f64>(&device, &config.config())?,

                sample_format => {
                    return Err(RuntimeError::UnsupportedSampleFormat(sample_format));
                }
            };

            loop {
                if kill_rx.try_recv().is_ok() {
                    drop(stream);
                    break;
                }
                std::thread::yield_now();
            }

            Ok(())
        });

        Ok(handle)
    }

    fn run_inner<T>(
        mut self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> RuntimeResult<cpal::Stream>
    where
        T: cpal::SizedSample + cpal::FromSample<f64>,
    {
        let channels = config.channels as usize;
        let audio_rate = config.sample_rate.0 as f64;

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _info: &cpal::OutputCallbackInfo| {
                    self.reset(audio_rate, data.len() / channels).unwrap();
                    self.process().unwrap();
                    for (frame_idx, frame) in data.chunks_mut(channels).enumerate() {
                        for (channel_idx, sample) in frame.iter_mut().enumerate() {
                            let buffer = &self.outputs[channel_idx];
                            let SignalBuffer::Sample(buffer) = buffer else {
                                panic!("output {channel_idx} signal type mismatch");
                            };
                            let value = buffer[frame_idx];
                            *sample = T::from_sample(*value);
                        }
                    }
                },
                |err| eprintln!("an error occurred on output: {}", err),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        Ok(stream)
    }
}

/// A handle to a running runtime. Can be used to stop the runtime.
#[must_use = "The runtime handle must be kept alive for the runtime to continue running"]
#[derive(Clone)]
pub struct RuntimeHandle {
    kill_tx: mpsc::Sender<()>,
}

impl RuntimeHandle {
    /// Stops the running runtime.
    pub fn stop(&self) {
        self.kill_tx.send(()).ok();
    }
}

impl Drop for RuntimeHandle {
    fn drop(&mut self) {
        self.stop();
    }
}
