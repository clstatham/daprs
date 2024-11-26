//! Simple wrappers around the FFT-based processors.

use crate::prelude::*;

/// An FFT-based convolution processor.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `a` | `Float` | The first input signal. |
/// | `1` | `b` | `Float` | The second input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The convolved output signal. |
#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SimpleFftConvolve {
    graph: FftGraph,
}

impl SimpleFftConvolve {
    /// Creates a new FFT-based convolution processor with the given FFT length, hop length, and window function.
    pub fn new(fft_length: usize, hop_length: usize, window_function: WindowFunction) -> Self {
        let graph = FftGraph::new(fft_length, hop_length, window_function).build(|fft| {
            let a = fft.add_audio_input();
            let b = fft.add_audio_input();
            let output = fft.add_audio_output();

            let convolved = a * b;

            output.input(0).connect(convolved.output(0));
        });

        Self { graph }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for SimpleFftConvolve {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("a", SignalType::Float),
            SignalSpec::new("b", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn allocate(&mut self, _sample_rate: Float, max_block_size: usize) {
        self.graph.allocate(max_block_size);
    }

    fn resize_buffers(&mut self, sample_rate: Float, block_size: usize) {
        self.graph.resize_buffers(sample_rate, block_size);
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        self.graph.process(inputs, outputs)
    }
}

/// An FFT-based deconvolution processor.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `a` | `Float` | The input signal. |
/// | `1` | `b` | `Float` | The filter signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The deconvolved output signal. |
#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SimpleFftDeconvolve {
    graph: FftGraph,
}

impl SimpleFftDeconvolve {
    /// Creates a new FFT-based deconvolution processor with the given FFT length, hop length, and window function.
    pub fn new(fft_length: usize, hop_length: usize, window_function: WindowFunction) -> Self {
        let graph = FftGraph::new(fft_length, hop_length, window_function).build(|fft| {
            let a = fft.add_audio_input();
            let b = fft.add_audio_input();
            let output = fft.add_audio_output();

            let deconvolved = a / b;

            output.input(0).connect(deconvolved.output(0));
        });

        Self { graph }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for SimpleFftDeconvolve {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("a", SignalType::Float),
            SignalSpec::new("b", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn allocate(&mut self, _sample_rate: Float, max_block_size: usize) {
        self.graph.allocate(max_block_size);
    }

    fn resize_buffers(&mut self, sample_rate: Float, block_size: usize) {
        self.graph.resize_buffers(sample_rate, block_size);
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        self.graph.process(inputs, outputs)
    }
}
