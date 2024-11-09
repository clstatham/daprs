//! Storage-related processors.

use crate::prelude::*;

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
    buffer: SignalBuffer,
    sample_rate: f64,
    index: f64,
}

impl AudioBuffer {
    /// Creates a new audio buffer processor with the given buffer.
    pub fn new(buffer: Buffer<Sample>) -> Self {
        Self {
            buffer: SignalBuffer::Sample(buffer),
            sample_rate: 0.0,
            index: 0.0,
        }
    }
}

impl Processor for AudioBuffer {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("index", Signal::new_message_some(Message::Int(0))),
            SignalSpec::unbounded("set", Signal::new_message_none()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("out", 0.0),
            SignalSpec::unbounded("length", Signal::new_message_none()),
        ]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let buffer = self.buffer.as_sample_mut().unwrap();

        let (mut outputs0, mut outputs1) = outputs.split_at_mut(1);

        for (out, length, index, set) in itertools::izip!(
            outputs0.iter_output_mut_as_samples(0)?,
            outputs1.iter_output_mut_as_messages(0)?,
            inputs.iter_input_as_messages(0)?,
            inputs.iter_input_as_messages(1)?
        ) {
            if let Some(index) = index {
                let Some(index) = index.cast_to_float() else {
                    return Err(ProcessorError::InputSpecMismatch(0));
                };

                self.index = index;

                if let Some(set) = set {
                    let set = set
                        .cast_to_float()
                        .ok_or(ProcessorError::InputSpecMismatch(1))?;

                    *buffer[self.index as usize] = set;
                }

                if index.fract() != 0.0 {
                    let pos_floor = index.floor() as usize;
                    let pos_ceil = index.ceil() as usize;

                    let value_floor = buffer[pos_floor];
                    let value_ceil = buffer[pos_ceil];

                    let t = index.fract();

                    *out = value_floor + (value_ceil - value_floor) * t.into();
                } else {
                    let index = index as i64;

                    if index < 0 {
                        self.index = buffer.len() as f64 + index as f64;
                    } else {
                        self.index = index as f64;
                    }

                    *out = buffer[self.index as usize];
                }
            }

            *length = Some(Message::Int(buffer.len() as i64));
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
#[derive(Clone, Debug, Default)]
pub struct Register {
    value: Option<Message>,
}

impl Processor for Register {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("set", Signal::new_message_none()),
            SignalSpec::unbounded("clear", Signal::new_message_none()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (set, clear, out) in itertools::izip!(
            inputs.iter_input_as_messages(0)?,
            inputs.iter_input_as_messages(1)?,
            outputs.iter_output_mut_as_messages(0)?
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
