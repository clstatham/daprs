use crate::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct FftConvolve;

impl FftProcessor for FftConvolve {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![
            super::FftSpec::new("carrier"),
            super::FftSpec::new("modulator"),
        ]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![super::FftSpec::new("out")]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&Fft],
        outputs: &mut [Fft],
    ) -> Result<(), ProcessorError> {
        let carrier = inputs[0];
        let modulator = inputs[1];
        let out = &mut outputs[0];
        for (out, carrier, modulator) in itertools::izip!(out.iter_mut(), carrier, modulator) {
            *out = carrier * modulator;
        }

        Ok(())
    }
}
