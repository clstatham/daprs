use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct SineOscillator {
    t: f64,
    t_step: f64,
    frequency: SignalSpec,
    out: SignalSpec,
}

impl Default for SineOscillator {
    fn default() -> Self {
        Self {
            t: 0.0,
            t_step: 0.0,
            frequency: SignalSpec::unbounded("frequency", 440.0),
            out: SignalSpec::unbounded("out", 0.0),
        }
    }
}

impl Process for SineOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![self.frequency]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![self.out]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.t_step = sample_rate.recip();
    }

    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let frequency = &inputs[0];
        let out = &mut outputs[0];

        for (out, frequency) in itertools::izip!(out, frequency) {
            *out = (self.t * frequency.value() * 2.0 * std::f64::consts::PI)
                .sin()
                .into();
            self.t += self.t_step;
        }
    }
}
