use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct SampleCount {
    count: u64,
    rate: SignalRate,
}

impl SampleCount {
    pub fn ar() -> Self {
        Self {
            count: 0,
            rate: SignalRate::Audio,
        }
    }
}

impl SampleCount {
    pub fn kr() -> Self {
        Self {
            count: 0,
            rate: SignalRate::Control,
        }
    }
}

impl Process for SampleCount {
    fn name(&self) -> &str {
        "sample_count"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("output"),
            rate: self.rate,
            kind: SignalKind::Buffer,
        }]
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, _inputs: &[Signal], outputs: &mut [Signal]) {
        let output = outputs[0].unwrap_buffer_mut();
        for i in 0..output.len() {
            let sample = self.count as f64;
            output[i] = sample.into();
            self.count += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Time {
    sample_count: u64,
    sample_rate: f64,
    rate: SignalRate,
}

impl Time {
    pub fn ar() -> Self {
        Self {
            sample_count: 0,
            sample_rate: 0.0,
            rate: SignalRate::Audio,
        }
    }
}

impl Time {
    pub fn kr() -> Self {
        Self {
            sample_count: 0,
            sample_rate: 0.0,
            rate: SignalRate::Control,
        }
    }
}

impl Process for Time {
    fn name(&self) -> &str {
        "time"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("output"),
            rate: self.rate,
            kind: SignalKind::Buffer,
        }]
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn resize_buffers(&mut self, audio_rate: f64, control_rate: f64, _block_size: usize) {
        match self.rate {
            SignalRate::Audio => self.sample_rate = audio_rate,
            SignalRate::Control => self.sample_rate = control_rate,
        }
    }

    #[inline]
    fn process(&mut self, _inputs: &[Signal], outputs: &mut [Signal]) {
        let output = outputs[0].unwrap_buffer_mut();
        for i in 0..output.len() {
            let time = (self.sample_count as f64) / self.sample_rate;
            output[i] = time.into();
            self.sample_count += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstSampleDelay {
    buffer: Box<[Sample]>,
    index: usize,
    rate: SignalRate,
}

impl ConstSampleDelay {
    pub fn ar(delay: usize) -> Self {
        Self {
            buffer: vec![Sample::new(0.0); delay].into_boxed_slice(),
            index: 0,
            rate: SignalRate::Audio,
        }
    }
}

impl ConstSampleDelay {
    pub fn kr(delay: usize) -> Self {
        Self {
            buffer: vec![Sample::new(0.0); delay].into_boxed_slice(),
            index: 0,
            rate: SignalRate::Control,
        }
    }
}

impl Process for ConstSampleDelay {
    fn name(&self) -> &str {
        "delay"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("input"),
            rate: self.rate,
            kind: SignalKind::Buffer,
        }]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("output"),
            rate: self.rate,
            kind: SignalKind::Buffer,
        }]
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
        let input = inputs[0].unwrap_buffer();
        let output = outputs[0].unwrap_buffer_mut();

        for i in 0..output.len() {
            let sample = self.buffer[self.index];
            output[i] = sample;
            self.buffer[self.index] = input[i];
            self.index = (self.index + 1) % self.buffer.len();
        }
    }
}
