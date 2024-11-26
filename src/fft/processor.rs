//! FFT processors for frequency-domain processing.

use downcast_rs::{impl_downcast, Downcast};

use crate::prelude::*;

use super::signal::FftSignalType;

/// A specification for an input or output of an FFT processor.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftSpec {
    /// The name of the input or output.
    pub name: String,
    pub signal_type: FftSignalType,
}

impl FftSpec {
    /// Creates a new FFT specification with the given name.
    pub fn new(name: impl Into<String>, signal_type: FftSignalType) -> Self {
        Self {
            name: name.into(),
            signal_type,
        }
    }
}

/// A special type of processor that processes frequency-domain data in an [`FftGraph`].
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait FftProcessor: Downcast + Send + FftProcessorClone {
    /// Returns the input specifications for this processor.
    fn input_spec(&self) -> Vec<FftSpec>;
    /// Returns the output specifications for this processor.
    fn output_spec(&self) -> Vec<FftSpec>;

    /// Allocates any necessary resources for the given FFT length.
    #[allow(unused)]
    fn allocate(&mut self, fft_length: usize, padded_length: usize) {}

    /// Processes the given inputs and stores the result in the given outputs.
    fn process(
        &mut self,
        fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError>;
}

impl_downcast!(FftProcessor);

#[doc(hidden)]
pub trait FftProcessorClone {
    fn clone_box(&self) -> Box<dyn FftProcessor>;
}

impl<T> FftProcessorClone for T
where
    T: 'static + FftProcessor + Clone,
{
    fn clone_box(&self) -> Box<dyn FftProcessor> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn FftProcessor> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
