use crate::prelude::*;

#[derive(Default, Debug, Clone)]
pub struct SinOsc<K: SignalKindMarker> {
    sample_rate: f64,
    sample_count: u64,
    _kind: std::marker::PhantomData<K>,
}

impl SinOsc<Audio> {
    pub fn ar() -> Self {
        Self {
            sample_rate: 0.0,
            sample_count: 0,
            _kind: std::marker::PhantomData,
        }
    }
}

impl SinOsc<Control> {
    pub fn kr() -> Self {
        Self {
            sample_rate: 0.0,
            sample_count: 0,
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for SinOsc<K> {
    fn name(&self) -> &str {
        "sin_osc"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND, K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        3
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn reset(&mut self, audio_rate: f64, _control_rate: f64, _block_size: usize) {
        self.sample_rate = audio_rate;
    }

    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let freq = &inputs[0];
        let phase = &inputs[1];
        let amp = &inputs[2];
        let output = &mut outputs[0];

        for (freq, phase, amp, output) in itertools::izip!(freq, phase, amp, output) {
            let phase = **phase % 1.0;
            let phase = phase * 2.0 * std::f64::consts::PI;
            let freq = **freq * 2.0 * std::f64::consts::PI;
            let value = **amp * (phase + freq * self.sample_count as f64 / self.sample_rate).sin();
            *output = value.into();
            self.sample_count += 1;
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct PulseOsc<K: SignalKindMarker> {
    sample_rate: f64,
    sample_count: u64,
    _kind: std::marker::PhantomData<K>,
}

impl PulseOsc<Audio> {
    pub fn ar() -> Self {
        Self {
            sample_rate: 0.0,
            sample_count: 0,
            _kind: std::marker::PhantomData,
        }
    }
}

impl PulseOsc<Control> {
    pub fn kr() -> Self {
        Self {
            sample_rate: 0.0,
            sample_count: 0,
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for PulseOsc<K> {
    fn name(&self) -> &str {
        "pulse_osc"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND, K::KIND, K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        4
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn reset(&mut self, audio_rate: f64, _control_rate: f64, _block_size: usize) {
        self.sample_rate = audio_rate;
    }

    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let freq = &inputs[0];
        let phase = &inputs[1];
        let width = &inputs[2];
        let amp = &inputs[3];
        let output = &mut outputs[0];

        for (freq, phase, width, amp, output) in itertools::izip!(freq, phase, width, amp, output) {
            let phase = **phase % 1.0;
            let phase = phase * 2.0 * std::f64::consts::PI;
            let width = **width;
            let freq = **freq * 2.0 * std::f64::consts::PI;

            let phase = phase + freq * self.sample_count as f64 / self.sample_rate;
            let value = if phase % (2.0 * std::f64::consts::PI) < width * 2.0 * std::f64::consts::PI
            {
                **amp
            } else {
                -**amp
            };

            *output = value.into();
            self.sample_count += 1;
        }
    }
}
