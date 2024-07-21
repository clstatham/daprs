use std::sync::mpsc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    graph::Graph,
    sample::{Buffer, Sample},
};

pub enum Backend {
    #[cfg(target_os = "linux")]
    Jack,
    #[cfg(target_os = "linux")]
    Alsa,
    #[cfg(target_os = "windows")]
    Wasapi,
}

#[derive(Default)]
pub struct Runtime {
    graph: Graph,
}

impl Runtime {
    pub fn new(graph: Graph) -> Self {
        Runtime { graph }
    }

    /// Runs the preparation phase for every node in the graph.
    pub fn prepare(&mut self) {
        self.graph.prepare_nodes();
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

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
    pub fn next_buffer(&mut self) -> &[Buffer] {
        self.graph.process();

        self.graph.outputs()
    }

    /// Runs the audio graph repeatedly for the given duration's worth of samples, and returns the rendered output channels.
    pub fn run_offline(
        &mut self,
        duration: std::time::Duration,
        sample_rate: f64,
        block_size: usize,
    ) -> Box<[Box<[Sample]>]> {
        let secs = duration.as_secs_f64();
        let samples = (sample_rate * secs) as usize;
        let blocks = samples / block_size;

        self.graph.reset(sample_rate, block_size);
        self.prepare();

        let num_outputs = self.graph.num_outputs();

        let mut outputs: Box<[Box<[Sample]>]> =
            vec![vec![Sample::default(); samples].into_boxed_slice(); num_outputs]
                .into_boxed_slice();

        for block_index in 0..blocks {
            self.graph.process();

            for (i, graph_output) in self.graph.outputs().iter().enumerate() {
                let output = &mut outputs[i];
                let block_offset = block_index * block_size;
                let output_slice = &mut output[block_offset..(block_offset + block_size)];
                output_slice.copy_from_slice(graph_output);
            }
        }

        outputs
    }

    pub fn run(mut self, backend: Backend) -> RuntimeHandle {
        let (kill_tx, kill_rx) = mpsc::channel();
        let (runtime_tx, runtime_rx) = mpsc::channel();

        let handle = RuntimeHandle {
            kill_tx,
            runtime_rx,
        };

        std::thread::spawn(move || {
            let host_id = match backend {
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

            let device = host
                .default_output_device()
                .expect("No default output device found.");

            let config = device.default_output_config().unwrap();

            let channels = config.channels();
            if self.graph.num_outputs() != channels as usize {
                panic!(
                    "Graph has {} outputs but device has {} channels",
                    self.graph.num_outputs(),
                    channels
                );
            }

            println!("Configuration: {:#?}", config);

            let sample_rate = config.sample_rate().0 as f64;
            let block_size = *config.buffer_size();
            let block_size = if let cpal::SupportedBufferSize::Range { min, max: _ } = block_size {
                min as usize
            } else {
                panic!("Unsupported buffer size")
            };

            self.graph.reset(sample_rate, block_size);

            self.prepare();

            match config.sample_format() {
                cpal::SampleFormat::I8 => {
                    self.run_inner::<i8>(&device, &config.config(), kill_rx, runtime_tx)
                }
                cpal::SampleFormat::I16 => {
                    self.run_inner::<i16>(&device, &config.config(), kill_rx, runtime_tx)
                }
                cpal::SampleFormat::I32 => {
                    self.run_inner::<i32>(&device, &config.config(), kill_rx, runtime_tx)
                }
                cpal::SampleFormat::I64 => {
                    self.run_inner::<i64>(&device, &config.config(), kill_rx, runtime_tx)
                }
                cpal::SampleFormat::U8 => {
                    self.run_inner::<u8>(&device, &config.config(), kill_rx, runtime_tx)
                }
                cpal::SampleFormat::U16 => {
                    self.run_inner::<u16>(&device, &config.config(), kill_rx, runtime_tx)
                }
                cpal::SampleFormat::U32 => {
                    self.run_inner::<u32>(&device, &config.config(), kill_rx, runtime_tx)
                }
                cpal::SampleFormat::U64 => {
                    self.run_inner::<u64>(&device, &config.config(), kill_rx, runtime_tx)
                }
                cpal::SampleFormat::F32 => {
                    self.run_inner::<f32>(&device, &config.config(), kill_rx, runtime_tx)
                }
                cpal::SampleFormat::F64 => {
                    self.run_inner::<f64>(&device, &config.config(), kill_rx, runtime_tx)
                }

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
        kill_rx: mpsc::Receiver<()>,
        runtime_tx: mpsc::Sender<Runtime>,
    ) where
        T: cpal::SizedSample + cpal::FromSample<f64>,
    {
        let channels = config.channels as usize;

        let mut graph = self.graph.clone();

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _info: &cpal::OutputCallbackInfo| {
                    graph.set_block_size(data.len() / channels);
                    graph.process();
                    for (frame_idx, frame) in data.chunks_mut(channels).enumerate() {
                        for (channel_idx, sample) in frame.iter_mut().enumerate() {
                            let outputs = graph.outputs();
                            let buffer = &outputs[channel_idx];
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
                // stream.pause().unwrap();
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
