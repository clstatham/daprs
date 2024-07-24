use std::f64::consts::PI;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct BlSawOsc<R: SignalRateMarker> {
    sample_rate: f64,
    phase: f64,
    _rate: std::marker::PhantomData<R>,
}

impl BlSawOsc<Audio> {
    pub fn ar() -> Self {
        Self {
            sample_rate: 0.0,
            phase: 0.0,
            _rate: std::marker::PhantomData,
        }
    }
}

impl BlSawOsc<Control> {
    pub fn kr() -> Self {
        Self {
            sample_rate: 0.0,
            phase: 0.0,
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for BlSawOsc<R> {
    fn name(&self) -> &str {
        "bl_saw_osc"
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

    fn reset(&mut self, audio_rate: f64, control_rate: f64, _block_size: usize) {
        match R::RATE {
            SignalRate::Audio => self.sample_rate = audio_rate,
            SignalRate::Control => self.sample_rate = control_rate,
        }
    }

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let freq = &inputs[0];
        let output = &mut outputs[0];

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
