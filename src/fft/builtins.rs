//! Built-in FFT processors.

use crate::prelude::*;

/// A convolution processor for [`FftGraph`]s.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `a` | `Fft` | The first input signal. |
/// | `1` | `b` | `Fft` | The second input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Fft` | The convolved output signal. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftConvolve;

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for FftConvolve {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![super::FftSpec::new("a"), super::FftSpec::new("b")]
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
        let a = inputs[0];
        let b = inputs[1];
        let out = &mut outputs[0];
        for (out, a, b) in itertools::izip!(out.iter_mut(), a, b) {
            *out = a * b;
        }

        Ok(())
    }
}
