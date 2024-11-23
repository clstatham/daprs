use downcast_rs::{impl_downcast, Downcast};

use crate::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftSpec {
    pub name: String,
}

impl FftSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait FftProcessor: Downcast + Send + FftProcessorClone {
    fn input_spec(&self) -> Vec<FftSpec>;
    fn output_spec(&self) -> Vec<FftSpec>;

    #[allow(unused)]
    fn allocate(&mut self, fft_length: usize) {}

    fn process(
        &mut self,
        fft_length: usize,
        inputs: &[&Fft],
        outputs: &mut [Fft],
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
