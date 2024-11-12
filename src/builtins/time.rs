//! Time-related processors.

use crate::{
    prelude::{OutputSpec, Processor, ProcessorInputs, ProcessorOutputs},
    processor::ProcessorError,
    signal::{Buffer, Sample, SignalBuffer, SignalKind},
};

/// A metronome that emits a bang at the given period.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `period` | `Message(f64)` | | The period of the metronome in seconds. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bang` | Emits a bang at the given period. |
#[derive(Debug, Clone)]

pub struct Metro {
    period: Sample,
    last_time: Sample,
    next_time: Sample,
    time: Sample,
    sample_rate: Sample,
}

impl Metro {
    /// Creates a new metronome processor with the given period.
    pub fn new(period: Sample) -> Self {
        Self {
            period,
            last_time: 0.0,
            next_time: 0.0,
            time: 0.0,
            sample_rate: 0.0,
        }
    }

    fn next_sample(&mut self) -> bool {
        let out = if self.time >= self.next_time {
            self.last_time = self.time;
            self.next_time = self.time + self.period;
            true
        } else {
            false
        };

        self.time += self.sample_rate.recip();

        out
    }
}

impl Default for Metro {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl Processor for Metro {
    fn input_names(&self) -> Vec<String> {
        vec![String::from("period")]
    }

    fn output_spec(&self) -> Vec<OutputSpec> {
        vec![OutputSpec::new("out", SignalKind::Bool)]
    }

    fn resize_buffers(&mut self, sample_rate: Sample, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (period, out) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            outputs.iter_output_mut_as_bools(0)?
        ) {
            self.period = period.unwrap_or(self.period);

            if self.next_sample() {
                *out = Some(true);
            } else {
                *out = None;
            }
        }

        Ok(())
    }
}

/// A processor that delays a signal by a single sample.
///
/// Note that feedback loops inherently introduce a single sample delay, so this processor is not necessary in those cases.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `in` | `Sample` | | The input signal to delay. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The delayed output signal. |
#[derive(Debug, Clone, Default)]
pub struct UnitDelay {
    value: Option<Sample>,
}

impl UnitDelay {
    /// Creates a new unit delay processor.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Processor for UnitDelay {
    fn input_names(&self) -> Vec<String> {
        vec![String::from("in")]
    }

    fn output_spec(&self) -> Vec<OutputSpec> {
        vec![OutputSpec::new("out", SignalKind::Sample)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, in_signal) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?
        ) {
            *out = self.value;
            self.value = in_signal;
        }

        Ok(())
    }
}

/// A processor that delays a signal by a number of samples.
///
/// If you need to delay a signal by a single sample, consider using [`UnitDelay`] instead.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `in` | `Sample` | | The input signal to delay. |
/// | `1` | `delay` | `Message(int)` | | The number of samples to delay the input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The delayed output signal. |
#[derive(Debug, Clone)]
pub struct SampleDelay {
    play_head: usize,
    buffer: SignalBuffer,
}

impl SampleDelay {
    /// Creates a new sample delay processor.
    pub fn new(max_delay: usize) -> Self {
        let buffer = SignalBuffer::Sample(Buffer::zeros(max_delay));
        Self {
            buffer,
            play_head: 0,
        }
    }
}

impl Processor for SampleDelay {
    fn input_names(&self) -> Vec<String> {
        vec![String::from("in"), String::from("delay")]
    }

    fn output_spec(&self) -> Vec<OutputSpec> {
        vec![OutputSpec::new("out", SignalKind::Sample)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let buffer = self.buffer.as_sample_mut().unwrap();

        for (out, in_signal, delay) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_ints(1)?
        ) {
            let delay = delay.unwrap_or_default() as usize;
            let delay = delay.min(buffer.len() - 1);

            buffer[self.play_head] = in_signal;

            self.play_head = (self.play_head + 1) % buffer.len();

            let delay_head = (self.play_head + buffer.len() - delay) % buffer.len();

            let delayed = buffer[delay_head];

            *out = delayed;
        }

        Ok(())
    }
}
