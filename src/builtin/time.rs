use crate::{
    graph::node::Process,
    sample::{Audio, Buffer, Control, SignalKind, SignalKindMarker},
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

    fn input_kind(&self) -> SignalKind {
        SignalKind::None
    }

    fn output_kind(&self) -> SignalKind {
        K::KIND
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

    fn input_kind(&self) -> SignalKind {
        SignalKind::None
    }

    fn output_kind(&self) -> SignalKind {
        K::KIND
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
