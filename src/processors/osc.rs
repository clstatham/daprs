use std::f64::consts::PI;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct BlSawOsc {
    sample_rate: f64,
    phase: f64,
    rate: SignalRate,
}

impl BlSawOsc {
    pub fn ar() -> Self {
        Self {
            sample_rate: 0.0,
            phase: 0.0,
            rate: SignalRate::Audio,
        }
    }
}

impl BlSawOsc {
    pub fn kr() -> Self {
        Self {
            sample_rate: 0.0,
            phase: 0.0,
            rate: SignalRate::Control,
        }
    }
}

impl Process for BlSawOsc {
    fn name(&self) -> &str {
        "bl_saw_osc"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("freq"),
            rate: SignalRate::Control,
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

    fn resize_buffers(&mut self, audio_rate: f64, control_rate: f64, _block_size: usize) {
        match self.rate {
            SignalRate::Audio => self.sample_rate = audio_rate,
            SignalRate::Control => self.sample_rate = control_rate,
        }
    }

    #[inline]
    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
        let freq = inputs[0].unwrap_buffer();
        let output = outputs[0].unwrap_buffer_mut();

        for (o, f) in itertools::izip!(output, freq) {
            if **f <= 0.0 {
                *o = 0.0.into();
                continue;
            }
            let n_harmonics = (self.sample_rate / (2.0 * **f)) as usize;
            let t = self.phase / self.sample_rate;

            // integrate the fourier series
            let mut saw = 0.0;
            for i in 1..=n_harmonics {
                let i = i as f64;
                saw += (2.0 / (i * PI)) * (PI * i * t).sin();
            }

            self.phase += **f;
            self.phase %= self.sample_rate;

            *o = (saw * 2.0 - 1.0).into();
        }
    }
}
