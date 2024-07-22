use crate::{
    graph::node::Process,
    sample::{Audio, Buffer, Control, Sample, SignalKind, SignalKindMarker},
};

#[derive(Default, Debug, Clone)]
pub struct SampleCount<K: SignalKindMarker> {
    count: u64,
    _kind: std::marker::PhantomData<K>,
}

impl SampleCount<Audio> {
    pub fn ar() -> Self {
        Self {
            count: 0,
            _kind: std::marker::PhantomData,
        }
    }
}

impl SampleCount<Control> {
    pub fn kr() -> Self {
        Self {
            count: 0,
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for SampleCount<K> {
    fn name(&self) -> &str {
        "sample_count"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
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
pub struct Time<K: SignalKindMarker> {
    sample_count: u64,
    sample_rate: f64,
    _kind: std::marker::PhantomData<K>,
}

impl Time<Audio> {
    pub fn ar() -> Self {
        Self {
            sample_count: 0,
            sample_rate: 0.0,
            _kind: std::marker::PhantomData,
        }
    }
}

impl Time<Control> {
    pub fn kr() -> Self {
        Self {
            sample_count: 0,
            sample_rate: 0.0,
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Time<K> {
    fn name(&self) -> &str {
        "time"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn reset(&mut self, audio_rate: f64, _control_rate: f64, _block_size: usize) {
        self.sample_rate = audio_rate;
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
pub struct ConstSampleDelay<K: SignalKindMarker> {
    buffer: Box<[Sample]>,
    index: usize,
    _kind: std::marker::PhantomData<K>,
}

impl ConstSampleDelay<Audio> {
    pub fn ar(delay: usize) -> Self {
        Self {
            buffer: vec![Sample::new(0.0); delay].into_boxed_slice(),
            index: 0,
            _kind: std::marker::PhantomData,
        }
    }
}

impl ConstSampleDelay<Control> {
    pub fn kr(delay: usize) -> Self {
        Self {
            buffer: vec![Sample::new(0.0); delay].into_boxed_slice(),
            index: 0,
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for ConstSampleDelay<K> {
    fn name(&self) -> &str {
        "delay"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
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
