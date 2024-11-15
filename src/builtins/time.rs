//! Time-related processors.

use std::collections::VecDeque;

use crate::{
    prelude::{Processor, ProcessorInputs, ProcessorOutputs, SignalSpec},
    processor::ProcessorError,
    signal::{Float, SignalType},
};

/// A processor that generates a single-sample pulse at regular intervals.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `period` | `Float` | The period of the pulse in seconds. |
/// | `1` | `reset` | `Bool` | Whether to reset the pulse generator. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The pulse signal. |
#[derive(Debug, Clone)]
pub struct Metro {
    period: Float,
    last_time: u64,
    next_time: u64,
    time: u64,
    sample_rate: Float,
}

impl Metro {
    /// Creates a new `Metro` processor with the given period.
    pub fn new(period: Float) -> Self {
        Self {
            period,
            last_time: 0,
            next_time: 0,
            time: 0,
            sample_rate: 0.0,
        }
    }

    fn next_sample(&mut self) -> bool {
        let out = if self.time >= self.next_time {
            self.last_time = self.time;
            self.next_time = self.time + (self.period * self.sample_rate) as u64;
            true
        } else {
            false
        };

        self.time += 1;

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
            SignalSpec::new("period", SignalType::Float),
            SignalSpec::new("reset", SignalType::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Bool)]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (period, reset, out) in itertools::izip!(
            inputs.iter_input_as_floats(0)?,
            inputs.iter_input_as_bools(1)?,
            outputs.iter_output_mut_as_bools(0)?
        ) {
            if reset.unwrap_or(false) {
                self.time = 0;
                self.last_time = 0;
                self.next_time = 0;
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

/// A processor that delays a signal by one sample.
///
/// Note that feedback loops in a [`Graph`](crate::graph::Graph) implicitly introduce a delay of one sample, so this processor is not usually required to be used manually.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The delayed signal. |
#[derive(Debug, Clone, Default)]
pub struct UnitDelay {
    value: Option<Float>,
}

impl UnitDelay {
    /// Creates a new `UnitDelay` processor.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Processor for UnitDelay {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, in_signal) in itertools::izip!(
            outputs.iter_output_mut_as_floats(0)?,
            inputs.iter_input_as_floats(0)?
        ) {
            *out = self.value;
            self.value = in_signal;
        }

        Ok(())
    }
}

/// A processor that delays a signal by a number of samples.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `delay` | `Int` | The delay in samples. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The delayed signal. |
#[derive(Debug, Clone)]
pub struct SampleDelay {
    buffer: VecDeque<Float>,
}

impl SampleDelay {
    /// Creates a new `SampleDelay` processor with the given maximum delay.
    pub fn new(max_delay: usize) -> Self {
        let buffer = Vec::with_capacity(max_delay + 1);
        Self {
            buffer: buffer.into(),
        }
    }
}

impl Processor for SampleDelay {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("delay", SignalType::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, in_signal, delay) in itertools::izip!(
            outputs.iter_output_mut_as_floats(0)?,
            inputs.iter_input_as_floats(0)?,
            inputs.iter_input_as_ints(1)?
        ) {
            let buffer_len = self.buffer.len();
            let delay = delay.unwrap_or_default() as usize;

            if buffer_len != delay {
                self.buffer.resize(delay, 0.0);
            }

            *out = Some(self.buffer.pop_front().unwrap_or(0.0));

            self.buffer.push_back(in_signal.unwrap_or(0.0));
        }

        Ok(())
    }
}

/// A processor that generates an exponential decay envelope signal.
///
/// The envelope is generated by the following formula:
///
/// ```text
/// y(t) = exp(-t / tau)
/// ```
///
/// where `t` is the time since the last trigger and `tau` is the decay time constant.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trig` | `Bool` | The trigger signal. |
/// | `1` | `tau` | `Float` | The decay time constant. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The envelope signal. |
#[derive(Debug, Clone)]
pub struct DecayEnv {
    last_trig: bool,
    tau: Float,
    value: Float,
    time: Float,
    sample_rate: Float,
}

impl DecayEnv {
    /// Creates a new `DecayEnv` processor with the given decay time constant.
    pub fn new(tau: Float) -> Self {
        Self {
            last_trig: false,
            tau,
            value: 0.0,
            time: 1000.0,
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
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("tau", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (trig, tau, out) in itertools::izip!(
            inputs.iter_input_as_bools(0)?,
            inputs.iter_input_as_floats(1)?,
            outputs.iter_output_mut_as_floats(0)?
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
