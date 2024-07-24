use crate::{
    graph::node::Process,
    sample::{Audio, Buffer, Control, Sample, SignalRate, SignalRateMarker},
};

#[derive(Default, Debug, Clone)]
pub struct SampleCount<R: SignalRateMarker> {
    count: u64,
    _rate: std::marker::PhantomData<R>,
}

impl SampleCount<Audio> {
    pub fn ar() -> Self {
        Self {
            count: 0,
            _rate: std::marker::PhantomData,
        }
    }
}

impl SampleCount<Control> {
    pub fn kr() -> Self {
        Self {
            count: 0,
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for SampleCount<R> {
    fn name(&self) -> &str {
        "sample_count"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
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

#[derive(Default, Debug, Clone)]
pub struct Time<R: SignalRateMarker> {
    sample_count: u64,
    sample_rate: f64,
    _rate: std::marker::PhantomData<R>,
}

impl Time<Audio> {
    pub fn ar() -> Self {
        Self {
            sample_count: 0,
            sample_rate: 0.0,
            _rate: std::marker::PhantomData,
        }
    }
}

impl Time<Control> {
    pub fn kr() -> Self {
        Self {
            sample_count: 0,
            sample_rate: 0.0,
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Time<R> {
    fn name(&self) -> &str {
        "time"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn reset(&mut self, audio_rate: f64, control_rate: f64, _block_size: usize) {
        match R::RATE {
            SignalRate::Audio => self.sample_rate = audio_rate,
            SignalRate::Control => self.sample_rate = control_rate,
        }
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

#[derive(Debug, Clone)]
pub struct ConstSampleDelay<R: SignalRateMarker> {
    buffer: Box<[Sample]>,
    index: usize,
    _rate: std::marker::PhantomData<R>,
}

impl ConstSampleDelay<Audio> {
    pub fn ar(delay: usize) -> Self {
        Self {
            buffer: vec![Sample::new(0.0); delay].into_boxed_slice(),
            index: 0,
            _rate: std::marker::PhantomData,
        }
    }
}

impl ConstSampleDelay<Control> {
    pub fn kr(delay: usize) -> Self {
        Self {
            buffer: vec![Sample::new(0.0); delay].into_boxed_slice(),
            index: 0,
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for ConstSampleDelay<R> {
    fn name(&self) -> &str {
        "delay"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let input = &inputs[0];
        let output = &mut outputs[0];

        for i in 0..output.len() {
            let sample = self.buffer[self.index];
            output[i] = sample;
            self.buffer[self.index] = input[i];
            self.index = (self.index + 1) % self.buffer.len();
        }
    }
}
