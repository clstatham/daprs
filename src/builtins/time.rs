//! Time-related processors.

use crate::{
    prelude::{Processor, ProcessorInputs, ProcessorOutputs, SignalSpec},
    processor::ProcessorError,
    signal::{Buffer, Float, SignalBuffer, SignalType},
};















#[derive(Debug, Clone)]

pub struct Metro {
    period: Float,
    last_time: Float,
    next_time: Float,
    time: Float,
    sample_rate: Float,
}

impl Metro {
    
    pub fn new(period: Float) -> Self {
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
















#[derive(Debug, Clone, Default)]
pub struct UnitDelay {
    value: Option<Float>,
}

impl UnitDelay {
    
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
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_floats(0)?
        ) {
            *out = self.value;
            self.value = in_signal;
        }

        Ok(())
    }
}

















#[derive(Debug, Clone)]
pub struct SampleDelay {
    play_head: usize,
    buffer: SignalBuffer,
}

impl SampleDelay {
    
    pub fn new(max_delay: usize) -> Self {
        let buffer = SignalBuffer::Float(Buffer::zeros(max_delay));
        Self {
            buffer,
            play_head: 0,
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
        let buffer = self.buffer.as_sample_mut().unwrap();

        for (out, in_signal, delay) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_floats(0)?,
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





















#[derive(Debug, Clone)]
pub struct DecayEnv {
    last_trig: bool,
    tau: Float,
    value: Float,
    time: Float,
    sample_rate: Float,
}

impl DecayEnv {
    
    pub fn new(tau: Float) -> Self {
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
