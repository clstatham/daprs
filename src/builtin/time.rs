use crate::{graph::node::Process, sample::Buffer};

#[derive(Default)]
pub struct SampleCount {
    count: u64,
}

impl Process for SampleCount {
    fn name(&self) -> &str {
        "sample_count"
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, _inputs: &[Buffer], outputs: &mut [Buffer]) {
        let output = &mut outputs[0];
        for i in 0..output.len() {
            let sample = self.count as f64;
            output[i] = sample.into();
            self.count += 1;
        }
    }
}

#[derive(Default)]
pub struct Time {
    sample_count: u64,
    sample_rate: f64,
}

impl Process for Time {
    fn name(&self) -> &str {
        "time"
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn reset(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
        self.sample_count = 0;
    }

    #[inline]
    fn process(&mut self, _inputs: &[Buffer], outputs: &mut [Buffer]) {
        let output = &mut outputs[0];
        for i in 0..output.len() {
            let time = (self.sample_count as f64) / self.sample_rate;
            output[i] = time.into();
            self.sample_count += 1;
        }
    }
}
