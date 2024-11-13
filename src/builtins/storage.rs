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
/// | `0` | `out` | `Float` | The sample value read from the buffer. |
/// | `1` | `length` | `Message(i64)` | The length of the buffer in samples. |
#[derive(Clone, Debug)]
pub struct AudioBuffer {
    buffer: Buffer<Float>,
    sample_rate: Float,
    index: Float,
}

impl AudioBuffer {
    /// Creates a new audio buffer processor with the given buffer.
    pub fn new(buffer: Buffer<Float>) -> Self {
        Self {
            buffer,
            sample_rate: 0.0,
            index: 0.0,
        }
    }
}

impl Processor for AudioBuffer {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("index", SignalKind::Float),
            SignalSpec::new("set", SignalKind::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("out", SignalKind::Float),
            SignalSpec::new("length", SignalKind::Int),
        ]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let (mut outputs0, mut outputs1) = outputs.split_at_mut(1);

        for (out, length, index, write) in itertools::izip!(
            outputs0.iter_output_mut_as_samples(0)?,
            outputs1.iter_output_mut_as_ints(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
        ) {
            self.index = index.unwrap_or(self.index);

            if let Some(write) = write {
                self.buffer[self.index as usize] = Some(write);
            }

            if self.index.fract() != 0.0 {
                let pos_floor = self.index.floor() as usize;
                let pos_ceil = self.index.ceil() as usize;

                let value_floor = self.buffer[pos_floor].unwrap_or_default();
                let value_ceil = self.buffer[pos_ceil].unwrap_or_default();

                let t = self.index.fract();

                *out = Some(value_floor + (value_ceil - value_floor) * t);
            } else {
                let index = self.index as i64;

                if index < 0 {
                    self.index = self.buffer.len() as Float + index as Float;
                } else {
                    self.index = index as Float;
                }

                *out = Some(self.buffer[self.index as usize].unwrap_or_default());
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
    value: Option<S>,
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

impl<S: SignalData> Default for Register<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: SignalData> Processor for Register<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("set", S::KIND),
            SignalSpec::new("clear", SignalKind::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::KIND)]
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
            if let Some(set) = set {
                self.value = Some(set.clone());
            }

            if clear.is_some() {
                self.value = None;
            }

            *out = self.value.clone();
        }

        Ok(())
    }
}
