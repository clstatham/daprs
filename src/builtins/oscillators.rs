use crate::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct SineOscillator {
    t: f64,
    t_step: f64,
}

impl Process for SineOscillator {
    fn input_params(&self) -> Vec<Param> {
        vec![Param::default_with_name("frequency")]
    }

    fn output_params(&self) -> Vec<Param> {
        vec![Param::default_with_name("out")]
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
