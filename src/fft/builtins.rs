use num::Complex;

use crate::{prelude::*, signal::PI};

#[derive(Debug, Clone, Default)]
pub struct FftPhaseVocoder {
    phases: Box<[Float]>,
}

impl FftProcessor for FftPhaseVocoder {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![
            super::FftSpec::new("carrier"),
            super::FftSpec::new("modulator"),
        ]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![super::FftSpec::new("out")]
    }

    fn allocate(&mut self, fft_length: usize) {
        self.phases = vec![0.0; fft_length].into_boxed_slice();
    }

    fn process(
        &mut self,
        fft_length: usize,
        _hop_length: usize,
        inputs: &[&Fft],
        outputs: &mut [Fft],
    ) -> Result<(), ProcessorError> {
        let carrier = inputs[0];
        let modulator = inputs[1];
        let out = &mut outputs[0];
        for (out, (carrier, modulator)) in out.iter_mut().zip(carrier.iter().zip(modulator.iter()))
        {
            let carrier_phase = carrier.arg();
            let modulator_phase = modulator.arg();
            let conv_phase = (carrier_phase + modulator_phase) % (2.0 * PI);
            let conv_mag = carrier.norm();

            *out = Complex::from_polar(conv_mag, conv_phase);
        }

        out[0] = Complex::new(0.0, 0.0);
        out[fft_length / 2] = Complex::new(0.0, 0.0);

        Ok(())
    }
}
