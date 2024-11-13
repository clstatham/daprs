//! Time-related processors.

use crate::{
    prelude::{Processor, ProcessorInputs, ProcessorOutputs, SignalSpec},
    processor::ProcessorError,
    signal::{Buffer, Sample, SignalBuffer, SignalKind},
};

/// A metronome that emits a bang at the given period.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `period` | `Sample` | | The period of the metronome in seconds. |
/// | `1` | `reset` | `Bool` | | Resets the metronome. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | Emits a bang at the given period. |
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
        vec![
            SignalSpec::new("period", SignalKind::Sample),
            SignalSpec::new("reset", SignalKind::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Bool)]
    }

    fn resize_buffers(&mut self, sample_rate: Sample, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (period, reset, out) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_bools(1)?,
            outputs.iter_output_mut_as_bools(0)?
        ) {
            if reset.unwrap_or(false) {
                self.time = 0.0;
                self.last_time = 0.0;
                self.next_time = 0.0;
            }

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
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", SignalKind::Sample)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Sample)]
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
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalKind::Sample),
            SignalSpec::new("delay", SignalKind::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Sample)]
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

/// An exponential decay envelope generator.
///
/// The envelope is generated using the formula `y = y * tau`.
/// The envelope is clamped to the range `[0, 1]`.
/// The envelope is triggered by a boolean signal.
/// The envelope is reset to zero when the trigger signal is true.
///
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `trig` | `Bool` | | The trigger signal. |
/// | `1` | `tau` | `Sample` | `1.0` | The time constant of the envelope. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The envelope signal. |
#[derive(Debug, Clone)]
pub struct DecayEnv {
    last_trig: bool,
    tau: Sample,
    value: Sample,
    time: Sample,
    sample_rate: Sample,
}

impl DecayEnv {
    /// Creates a new decay envelope generator processor with the given time constant.
    pub fn new(tau: Sample) -> Self {
        Self {
            last_trig: false,
            tau,
            value: 0.0,
            time: 100.0,
            sample_rate: 0.0,
        }
    }
}

impl Default for DecayEnv {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl Processor for DecayEnv {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalKind::Bool),
            SignalSpec::new("tau", SignalKind::Sample),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Sample)]
    }

    fn resize_buffers(&mut self, sample_rate: Sample, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (trig, tau, out) in itertools::izip!(
            inputs.iter_input_as_bools(0)?,
            inputs.iter_input_as_samples(1)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            self.tau = tau.unwrap_or(self.tau);
            let trig = trig.unwrap_or(false);

            if trig && !self.last_trig {
                self.value = 1.0;
                self.time = 0.0;
            } else {
                self.time += self.sample_rate.recip();
                self.value = (-self.tau.recip() * self.time).exp();
            }

            self.last_trig = trig;

            self.value = self.value.clamp(0.0, 1.0);

            *out = Some(self.value);
        }

        Ok(())
    }
}
