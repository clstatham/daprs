use std::sync::Arc;

use num::Complex;
use realfft::{ComplexToReal, RealToComplex};
use signal::FloatBuf;

use crate::prelude::*;

pub mod builder;
pub mod builtins;
pub mod graph;
pub mod processor;
pub mod signal;

#[derive(Debug, Clone, thiserror::Error)]
pub enum FftError {
    #[error("realfft error: {0}")]
    RealFft(String),
}

impl From<realfft::FftError> for FftError {
    fn from(err: realfft::FftError) -> Self {
        Self::RealFft(err.to_string())
    }
}

#[derive(Clone)]
pub struct FftPlan {
    // frequency-domain `Fft` length will be `fft_length / 2 + 1`, as this is an RFFT
    fft_length: usize,
    padded_length: usize,
    forward: Arc<dyn RealToComplex<Float>>,
    inverse: Arc<dyn ComplexToReal<Float>>,
    forward_scratch: Fft,
    inverse_scratch: Fft,
}

impl FftPlan {
    pub fn new(fft_length: usize) -> Self {
        let padded_length = fft_length * 2;
        let mut plan = realfft::RealFftPlanner::new();
        let forward = plan.plan_fft_forward(padded_length);
        let inverse = plan.plan_fft_inverse(padded_length);
        let forward_scratch = forward.make_scratch_vec().into_boxed_slice();
        let inverse_scratch = inverse.make_scratch_vec().into_boxed_slice();
        Self {
            fft_length,
            padded_length,
            forward,
            inverse,
            forward_scratch: Fft(forward_scratch),
            inverse_scratch: Fft(inverse_scratch),
        }
    }

    pub fn real_length(&self) -> usize {
        self.fft_length
    }

    pub fn complex_length(&self) -> usize {
        self.fft_length / 2 + 1
    }

    pub fn padded_length(&self) -> usize {
        self.padded_length
    }

    pub fn forward(
        &mut self,
        input: &mut [Float],
        output: &mut [Complex<Float>],
    ) -> Result<(), FftError> {
        self.forward
            .process_with_scratch(input, output, &mut self.forward_scratch)?;
        Ok(())
    }

    pub fn inverse(
        &mut self,
        input: &mut [Complex<Float>],
        output: &mut [Float],
    ) -> Result<(), FftError> {
        self.inverse
            .process_with_scratch(input, output, &mut self.inverse_scratch)?;
        Ok(())
    }
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WindowFunction {
    Rectangular,
    #[default]
    Hann,
}

impl WindowFunction {
    pub fn generate(&self, length: usize) -> FloatBuf {
        let mut buf = vec![0.0; length].into_boxed_slice();
        match self {
            Self::Rectangular => {
                for x in buf.iter_mut() {
                    *x = 1.0;
                }
            }
            Self::Hann => {
                buf = apodize::hanning_iter(length).collect();
            }
        }
        FloatBuf(buf)
    }
}
