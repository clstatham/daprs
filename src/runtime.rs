use std::sync::mpsc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{graph::Graph, sample::Sample};

/// The audio backend to use for the runtime.
#[derive(Default, Debug)]
pub enum Backend {
    #[default]
    Default,
    #[cfg(target_os = "linux")]
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
    pub fn reset(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        self.graph.reset(audio_rate, control_rate, block_size);
    }

    /// Runs the preparation phase for every node in the graph.
    pub fn prepare(&mut self) {
        self.graph.prepare_nodes();
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
    pub fn outputs(&mut self) -> impl Iterator<Item = &[Sample]> + '_ {
        let num_outputs = self.graph.num_outputs();
        (0..num_outputs).map(|i| self.graph.get_output(i))
    }

    /// Renders the next block of audio and returns the rendered output channels.
    #[inline]
    pub fn next_buffer(&mut self) -> impl Iterator<Item = &[Sample]> + '_ {
        self.graph.process();

        self.graph.outputs()
    }

    /// Runs the audio graph repeatedly for the given duration's worth of samples, and returns the rendered output channels.
    pub fn run_offline(
        &mut self,
        duration: std::time::Duration,
        audio_rate: f64,
        control_rate: f64,
        block_size: usize,
    ) -> Box<[Box<[Sample]>]> {
        let secs = duration.as_secs_f64();
        let samples = (audio_rate * secs) as usize;

        self.reset(audio_rate, control_rate, block_size);
        self.prepare();

        let num_outputs: usize = self.graph.num_outputs();

        let mut outputs: Box<[Box<[Sample]>]> =
            vec![vec![Sample::new(0.0); samples].into_boxed_slice(); num_outputs]
                .into_boxed_slice();

        let mut sample_count = 0;

        while sample_count < samples {
            let actual_block_size = (samples - sample_count).min(block_size);
            self.graph
                .set_block_size(audio_rate, control_rate, actual_block_size);
            self.graph.process();

            for (i, output) in outputs.iter_mut().enumerate() {
                let buffer = self.graph.get_output(i);
                output[sample_count..sample_count + actual_block_size].copy_from_slice(buffer);
            }

            sample_count += actual_block_size;
        }

        outputs
    }

    pub fn run_offline_to_file(
        &mut self,
        file_path: impl AsRef<std::path::Path>,
        duration: std::time::Duration,
        audio_rate: f64,
        control_rate: f64,
        block_size: usize,
    ) {
        let outputs = self.run_offline(duration, audio_rate, control_rate, block_size);

        let num_channels = outputs.len();

        let num_samples = outputs[0].len();

        let mut samples = vec![0.0; num_samples * num_channels];

        for sample_index in 0..num_samples {
            for channel_index in 0..num_channels {
                let i = sample_index * num_channels + channel_index;
                samples[i] = *outputs[channel_index][sample_index];
            }
        }

        wavers::write(file_path, &samples, audio_rate as i32, num_channels as u16).unwrap();
    }

    pub fn run_for(
        &mut self,
        duration: std::time::Duration,
        backend: Backend,
        device: Device,
        control_rate: f64,
    ) {
        let runtime = std::mem::take(self);
        let handle = runtime.run(backend, device, control_rate);
        std::thread::sleep(duration);
        *self = handle.stop();
    }

    pub fn run(mut self, backend: Backend, device: Device, control_rate: f64) -> RuntimeHandle {
        let (kill_tx, kill_rx) = mpsc::channel();
        let (runtime_tx, runtime_rx) = mpsc::channel();

        let handle = RuntimeHandle {
            kill_tx,
            runtime_rx,
        };

        std::thread::spawn(move || {
            let host_id = match backend {
                Backend::Default => cpal::default_host().id(),
                #[cfg(target_os = "linux")]
                Backend::Alsa => cpal::available_hosts()
                    .into_iter()
                    .find(|h| *h == cpal::HostId::Alsa)
                    .expect("ALSA host was requested but not found"),
                #[cfg(target_os = "linux")]
                Backend::Jack => cpal::available_hosts()
                    .into_iter()
                    .find(|h| *h == cpal::HostId::Jack)
                    .expect("Jack host was requested but not found"),
                #[cfg(target_os = "windows")]
                Backend::Wasapi => cpal::available_hosts()
                    .into_iter()
                    .find(|h| *h == cpal::HostId::Wasapi)
                    .expect("WASAPI host was requested but not found"),
            };
            let host = cpal::host_from_id(host_id).unwrap();

            log::info!("Using host: {:?}", host.id());

            let device = match device {
                Device::Default => host.default_output_device().unwrap(),
                Device::Index(index) => host.output_devices().unwrap().nth(index).unwrap(),
                Device::Name(name) => host
                    .output_devices()
                    .unwrap()
                    .find(|d| d.name().unwrap().contains(&name))
                    .unwrap(),
            };

            // let device = host
            //     .default_output_device()
            //     .expect("No default output device found.");

            log::info!("Using device: {}", device.name().unwrap());

            let config = device.default_output_config().unwrap();

            let channels = config.channels();
            if self.graph.num_outputs() != channels as usize {
                panic!(
                    "Graph has {} outputs but device has {} channels",
                    self.graph.num_outputs(),
                    channels
                );
            }

            log::info!("Configuration: {:#?}", config);

            let audio_rate = config.sample_rate().0 as f64;
            let initial_block_size = audio_rate as usize / 100;

            self.graph
                .reset(audio_rate, control_rate, initial_block_size);

            self.prepare();

            match config.sample_format() {
                cpal::SampleFormat::I8 => self.run_inner::<i8>(
                    &device,
                    &config.config(),
                    control_rate,
                    kill_rx,
                    runtime_tx,
                ),
                cpal::SampleFormat::I16 => self.run_inner::<i16>(
                    &device,
                    &config.config(),
                    control_rate,
                    kill_rx,
                    runtime_tx,
                ),
                cpal::SampleFormat::I32 => self.run_inner::<i32>(
                    &device,
                    &config.config(),
                    control_rate,
                    kill_rx,
                    runtime_tx,
                ),
                cpal::SampleFormat::I64 => self.run_inner::<i64>(
                    &device,
                    &config.config(),
                    control_rate,
                    kill_rx,
                    runtime_tx,
                ),
                cpal::SampleFormat::U8 => self.run_inner::<u8>(
                    &device,
                    &config.config(),
                    control_rate,
                    kill_rx,
                    runtime_tx,
                ),
                cpal::SampleFormat::U16 => self.run_inner::<u16>(
                    &device,
                    &config.config(),
                    control_rate,
                    kill_rx,
                    runtime_tx,
                ),
                cpal::SampleFormat::U32 => self.run_inner::<u32>(
                    &device,
                    &config.config(),
                    control_rate,
                    kill_rx,
                    runtime_tx,
                ),
                cpal::SampleFormat::U64 => self.run_inner::<u64>(
                    &device,
                    &config.config(),
                    control_rate,
                    kill_rx,
                    runtime_tx,
                ),
                cpal::SampleFormat::F32 => self.run_inner::<f32>(
                    &device,
                    &config.config(),
                    control_rate,
                    kill_rx,
                    runtime_tx,
                ),
                cpal::SampleFormat::F64 => self.run_inner::<f64>(
                    &device,
                    &config.config(),
                    control_rate,
                    kill_rx,
                    runtime_tx,
                ),

                sample_format => {
                    panic!("Unsupported sample format {:?}", sample_format);
                }
            }
        });

        handle
    }

    fn run_inner<T>(
        self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        control_rate: f64,
        kill_rx: mpsc::Receiver<()>,
        runtime_tx: mpsc::Sender<Runtime>,
    ) where
        T: cpal::SizedSample + cpal::FromSample<f64>,
    {
        let channels = config.channels as usize;
        let audio_rate = config.sample_rate.0 as f64;

        let mut graph = self.graph.clone();

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _info: &cpal::OutputCallbackInfo| {
                    graph.set_block_size(audio_rate, control_rate, data.len() / channels);
                    graph.process();
                    for (frame_idx, frame) in data.chunks_mut(channels).enumerate() {
                        for (channel_idx, sample) in frame.iter_mut().enumerate() {
                            let buffer = graph.get_output(channel_idx);
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
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        runtime_tx.send(self).unwrap();
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
