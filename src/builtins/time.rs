//! Time-related processors.

use crate::{
    message::Message,
    prelude::{Processor, ProcessorInputs, ProcessorOutputs, SignalSpec},
    processor::ProcessorError,
    signal::{Buffer, Sample, Signal, SignalBuffer},
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
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded(
            "period",
            Signal::new_message_some(Message::Float(self.period)),
        )]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
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
            inputs.iter_input_as_messages(0)?,
            outputs.iter_output_mut_as_messages(0)?
        ) {
            if let Some(period) = period.cast_to_float() {
                self.period = period;
            }

            if self.next_sample() {
                *out = Message::Bang;
            } else {
                *out = Message::None;
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
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("in", Signal::new_sample(0.0))]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_sample(0.0))]
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
            *out = self.value.unwrap_or_default();
            self.value = Some(in_signal);
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
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("in", Signal::new_sample(0.0)),
            SignalSpec::unbounded("delay", Signal::new_message_none()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_sample(0.0))]
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
            inputs.iter_input_as_messages(1)?
        ) {
            let delay = if delay.is_some() {
                delay.cast_to_int().unwrap_or(0).max(0) as usize
            } else {
                0
            };

            buffer[self.play_head] = in_signal;

            self.play_head = (self.play_head + 1) % buffer.len();

            let delay_head = (self.play_head + buffer.len() - delay) % buffer.len();

            let delayed = buffer[delay_head];

            *out = delayed;
        }

        Ok(())
    }
}
