use std::{sync::mpsc, time::Duration};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    graph::{Graph, GraphRunError},
    processor::ProcessorError,
    signal::{Sample, SignalBuffer},
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[error("Runtime error")]
pub enum RuntimeError {
    StreamError(#[from] cpal::StreamError),
    DevicesError(#[from] cpal::DevicesError),
    Hound(#[from] hound::Error),
    HostUnavailable(#[from] cpal::HostUnavailable),
    #[error("Requested device is unavailable: {0:?}")]
    DeviceUnavailable(Device),
    DeviceNameError(#[from] cpal::DeviceNameError),
    DefaultStreamConfigError(#[from] cpal::DefaultStreamConfigError),
    #[error("Unsupported sample format: {0}")]
    UnsupportedSampleFormat(cpal::SampleFormat),
    GraphRunError(#[from] GraphRunError),
    ProcessorError(#[from] ProcessorError),
    #[error("Channel mismatch: expected {0} channels, got {1}")]
    ChannelMismatch(usize, usize),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// The audio backend to use for the runtime.
#[derive(Default, Debug)]
pub enum Backend {
    #[default]
    Default,
    #[cfg(all(target_os = "linux", feature = "jack"))]
    Jack,
    #[cfg(target_os = "linux")]
    Alsa,
    #[cfg(target_os = "windows")]
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

/// The audio graph processing runtime.
///
/// The runtime is responsible for running the audio graph and rendering audio samples.
/// It can run in real-time ([`run`](Runtime::run)) or offline ([`run_offline`](Runtime::run_offline)) mode.
///
/// In real-time mode, the runtime will render audio samples in real-time using a specified audio backend and device.
///
/// In offline mode, the runtime will render audio samples as fast as possible and return the rendered output channels.
#[derive(Default)]
pub struct Runtime {
    graph: Graph,
}

impl Runtime {
    /// Creates a new runtime with the given audio graph.
    pub fn new(graph: Graph) -> Self {
        Runtime { graph }
    }

    /// Resets the runtime with the given sample rate and block size.
    /// This will reset the state of all nodes in the graph and potentially reallocate internal buffers.
    pub fn reset(&mut self, sample_rate: f64, block_size: usize) -> RuntimeResult<()> {
        self.graph.reset(sample_rate, block_size)?;
        Ok(())
    }

    /// Runs the preparation phase for every node in the graph.
    pub fn prepare(&mut self) -> RuntimeResult<()> {
        self.graph.prepare_nodes()?;
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
        (0..num_outputs).map(|i| self.graph.get_output(i))
    }

    /// Renders the next block of audio and returns the rendered output channels.
    #[inline]
    pub fn next_buffer(&mut self) -> RuntimeResult<impl Iterator<Item = &SignalBuffer> + '_> {
        self.graph.process()?;

        Ok(self.graph.outputs())
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
            self.graph.process()?;

            for (i, output) in outputs.iter_mut().enumerate() {
                let buffer = self.graph.get_output(i);
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

    pub fn run_for(
        &mut self,
        duration: Duration,
        backend: Backend,
        device: Device,
    ) -> RuntimeResult<()> {
        let runtime = std::mem::take(self);
        let handle = runtime.run(backend, device)?;
        std::thread::sleep(duration);
        *self = handle.stop();
        Ok(())
    }

    pub fn run(mut self, backend: Backend, device: Device) -> RuntimeResult<RuntimeHandle> {
        let (kill_tx, kill_rx) = mpsc::channel();
        let (runtime_tx, runtime_rx) = mpsc::channel();

        let handle = RuntimeHandle {
            kill_tx,
            runtime_rx,
        };

        std::thread::spawn(move || -> RuntimeResult<()> {
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

            self.graph.reset(audio_rate, initial_block_size)?;

            self.prepare()?;

            match config.sample_format() {
                cpal::SampleFormat::I8 => {
                    self.run_inner::<i8>(&device, &config.config(), kill_rx, runtime_tx)?
                }
                cpal::SampleFormat::I16 => {
                    self.run_inner::<i16>(&device, &config.config(), kill_rx, runtime_tx)?
                }
                cpal::SampleFormat::I32 => {
                    self.run_inner::<i32>(&device, &config.config(), kill_rx, runtime_tx)?
                }
                cpal::SampleFormat::I64 => {
                    self.run_inner::<i64>(&device, &config.config(), kill_rx, runtime_tx)?
                }
                cpal::SampleFormat::U8 => {
                    self.run_inner::<u8>(&device, &config.config(), kill_rx, runtime_tx)?
                }
                cpal::SampleFormat::U16 => {
                    self.run_inner::<u16>(&device, &config.config(), kill_rx, runtime_tx)?
                }
                cpal::SampleFormat::U32 => {
                    self.run_inner::<u32>(&device, &config.config(), kill_rx, runtime_tx)?
                }
                cpal::SampleFormat::U64 => {
                    self.run_inner::<u64>(&device, &config.config(), kill_rx, runtime_tx)?
                }
                cpal::SampleFormat::F32 => {
                    self.run_inner::<f32>(&device, &config.config(), kill_rx, runtime_tx)?
                }
                cpal::SampleFormat::F64 => {
                    self.run_inner::<f64>(&device, &config.config(), kill_rx, runtime_tx)?
                }

                sample_format => {
                    return Err(RuntimeError::UnsupportedSampleFormat(sample_format));
                }
            }

            Ok(())
        });

        Ok(handle)
    }

    fn run_inner<T>(
        self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        kill_rx: mpsc::Receiver<()>,
        runtime_tx: mpsc::Sender<Runtime>,
    ) -> RuntimeResult<()>
    where
        T: cpal::SizedSample + cpal::FromSample<f64>,
    {
        let channels = config.channels as usize;
        let audio_rate = config.sample_rate.0 as f64;

        let mut graph = self.graph.clone();

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _info: &cpal::OutputCallbackInfo| {
                    graph
                        .resize_buffers(audio_rate, data.len() / channels)
                        .unwrap();
                    graph.process().unwrap();
                    for (frame_idx, frame) in data.chunks_mut(channels).enumerate() {
                        for (channel_idx, sample) in frame.iter_mut().enumerate() {
                            let buffer = graph.get_output(channel_idx);
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

        loop {
            if kill_rx.try_recv().is_ok() {
                drop(stream);
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        runtime_tx.send(self).unwrap();

        Ok(())
    }
}

pub struct RuntimeHandle {
    kill_tx: mpsc::Sender<()>,
    runtime_rx: mpsc::Receiver<Runtime>,
}

impl RuntimeHandle {
    pub fn stop(&self) -> Runtime {
        self.kill_tx.send(()).unwrap();
        self.runtime_rx.recv().unwrap()
    }
}
