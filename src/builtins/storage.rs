//! Storage-related processors.

use std::marker::PhantomData;

use crate::{prelude::*, signal::SignalData};

/// A processor that reads and writes audio samples in a buffer.
///
/// When reading: from the buffer:
/// - If the index is a whole number, the processor reads the sample at the given index.
/// - If the index is a fraction, the processor linearly interpolates between the samples at the floor and ceil positions.
///
/// When writing to the buffer:
/// - The processor sets the value at the given index (rounded down) if a message is received at the `set` input.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `index` | `Message(i64)` | `0` | The sample index to read from the buffer. |
/// | `1` | `set` | `Message(f64)` |  | Set the value at the given index. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The sample value read from the buffer. |
/// | `1` | `length` | `Message(i64)` | The length of the buffer in samples. |
#[derive(Clone, Debug)]
pub struct AudioBuffer {
    buffer: Buffer<Sample>,
    sample_rate: Sample,
    index: Sample,
}

impl AudioBuffer {
    /// Creates a new audio buffer processor with the given buffer.
    pub fn new(buffer: Buffer<Sample>) -> Self {
        Self {
            buffer,
            sample_rate: 0.0,
            index: 0.0,
        }
    }
}

impl Processor for AudioBuffer {
    fn input_names(&self) -> Vec<String> {
        vec![
            String::from("index"),
            String::from("write"),
            String::from("enable_write"),
        ]
    }

    fn output_spec(&self) -> Vec<OutputSpec> {
        vec![
            OutputSpec::new("out", SignalKind::Sample),
            OutputSpec::new("length", SignalKind::Int),
        ]
    }

    fn resize_buffers(&mut self, sample_rate: Sample, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let (mut outputs0, mut outputs1) = outputs.split_at_mut(1);

        for (out, length, index, write, enable_write) in itertools::izip!(
            outputs0.iter_output_mut_as_samples(0)?,
            outputs1.iter_output_mut_as_ints(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
            inputs.iter_input_as_bools(2)?
        ) {
            self.index = index.unwrap_or(self.index);

            if let Some(true) = enable_write {
                if let Some(write) = write {
                    self.buffer[self.index as usize] = write;
                }
            }

            if self.index.fract() != 0.0 {
                let pos_floor = self.index.floor() as usize;
                let pos_ceil = self.index.ceil() as usize;

                let value_floor = self.buffer[pos_floor];
                let value_ceil = self.buffer[pos_ceil];

                let t = self.index.fract();

                *out = Some(value_floor + (value_ceil - value_floor) * t);
            } else {
                let index = self.index as i64;

                if index < 0 {
                    self.index = self.buffer.len() as Sample + index as Sample;
                } else {
                    self.index = index as Sample;
                }

                *out = Some(self.buffer[self.index as usize]);
            }

            *length = Some(self.buffer.len() as i64);
        }

        Ok(())
    }
}

/// A processor that stores a message in a register.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `set` | `Message` |  | Set the register to the value. |
/// | `1` | `clear` | `Message` |  | Clear the register. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Message` | The value stored in the register. |
#[derive(Clone, Debug)]
pub struct Register<S: SignalData> {
    value: Option<S::Value>,
    _phantom: PhantomData<S>,
}

impl<S: SignalData> Register<S> {
    /// Creates a new register processor.
    pub fn new() -> Self {
        Self {
            value: None,
            _phantom: PhantomData,
        }
    }
}

impl<S: SignalData> Processor for Register<S> {
    fn input_names(&self) -> Vec<String> {
        vec![String::from("set"), String::from("clear")]
    }

    fn output_spec(&self) -> Vec<OutputSpec> {
        vec![OutputSpec::new("out", S::KIND)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (set, clear, out) in itertools::izip!(
            inputs.iter_input_as::<S>(0)?,
            inputs.iter_input_as_bools(1)?,
            outputs.iter_output_as::<S>(0)?,
        ) {
            if let Some(set) = S::buffer_element_to_value(set) {
                self.value = Some(set.clone());
            }

            if clear.is_some() {
                self.value = None;
            }

            if let Some(value) = &self.value {
                *out = S::value_to_buffer_element(value);
            } else {
                *out = S::buffer_element_default().clone();
            }
        }

        Ok(())
    }
}
