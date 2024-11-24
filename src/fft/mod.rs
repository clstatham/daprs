//! Fast Fourier Transform (FFT) processing.

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

/// An error that can occur during FFT processing.
#[derive(Debug, Clone, thiserror::Error)]
pub enum FftError {
    /// An error occurred during an FFT operation.
    #[error("realfft error: {0}")]
    RealFft(String),
}

impl From<realfft::FftError> for FftError {
    fn from(err: realfft::FftError) -> Self {
        Self::RealFft(err.to_string())
    }
}

/// Cached state for an RFFT and IRFFT transform for a given FFT window length.
#[derive(Clone)]
pub struct FftPlan {
    fft_length: usize,
    forward: Arc<dyn RealToComplex<Float>>,
    inverse: Arc<dyn ComplexToReal<Float>>,
    forward_scratch: Fft,
    inverse_scratch: Fft,
}

impl FftPlan {
    /// Creates a new `FftPlan` for the given FFT window length.
    pub fn new(fft_length: usize) -> Self {
        let padded_length = fft_length * 2;
        let mut plan = realfft::RealFftPlanner::new();
        let forward = plan.plan_fft_forward(padded_length);
        let inverse = plan.plan_fft_inverse(padded_length);
        let forward_scratch = forward.make_scratch_vec().into_boxed_slice();
        let inverse_scratch = inverse.make_scratch_vec().into_boxed_slice();
        Self {
            fft_length,
            forward,
            inverse,
            forward_scratch: Fft(forward_scratch),
            inverse_scratch: Fft(inverse_scratch),
        }
    }

    /// Performs an RFFT on the given input buffer, storing the result in the output buffer.
    ///
    /// The input buffer will be modified in-place as scratch space, so it should be considered invalid after this call.
    ///
    /// This function does not allocate any memory on the heap.
    pub fn forward(
        &mut self,
        input: &mut [Float],
        output: &mut [Complex<Float>],
    ) -> Result<(), FftError> {
        self.forward
            .process_with_scratch(input, output, &mut self.forward_scratch)?;
        Ok(())
    }

    /// Performs an IRFFT on the given input buffer, storing the result in the output buffer.
    ///
    /// The input buffer will be modified in-place as scratch space, so it should be considered invalid after this call.
    ///
    /// This function does not allocate any memory on the heap.
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

/// A window function to apply to the input signal before FFT processing.
#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WindowFunction {
    /// A rectangular window function (no windowing).
    Rectangular,
    /// A Hann window function.
    #[default]
    Hann,
    /// A Hamming window function.
    Hamming,
    /// A Blackman window function.
    Blackman,
    /// A Nuttall window function.
    Nuttall,
    /// A triangular window function.
    Triangular,
}

impl WindowFunction {
    /// Generates a window of the given length using this window function.
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
            Self::Hamming => {
                buf = apodize::hamming_iter(length).collect();
            }
            Self::Blackman => {
                buf = apodize::blackman_iter(length).collect();
            }
            Self::Nuttall => {
                buf = apodize::nuttall_iter(length).collect();
            }
            Self::Triangular => {
                buf = apodize::triangular_iter(length).collect();
            }
        }
        FloatBuf(buf)
    }
}
