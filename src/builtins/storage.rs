//! Storage-related processors.

use crate::prelude::*;

/// A processor that reads a sample from a buffer.
///
/// If the index is out of bounds, it will wrap around.
/// If the index is not an integer, the processor will lineraly interpolate between the two nearest samples.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `position` | `Message(i64)` | `0` | The sample index to read from the buffer. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The sample value read from the buffer. |
/// | `1` | `length` | `Message(i64)` | The length of the buffer in samples. |
#[derive(Clone, Debug)]
pub struct BufferReader {
    buffer: SignalBuffer,
    sample_rate: f64,
    pos: f64,
}

impl BufferReader {
    /// Creates a new buffer reader processor.
    pub fn new(buffer: Buffer<Sample>) -> Self {
        Self {
            buffer: SignalBuffer::Sample(buffer),
            sample_rate: 0.0,
            pos: 0.0,
        }
    }
}

impl Process for BufferReader {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded(
            "position",
            Signal::new_message_some(Message::Int(0)),
        )]
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
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let position = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let (out, length) = outputs.split_at_mut(1);

        let out = out[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        let length = length[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(1))?;

        let buffer = self.buffer.as_sample().unwrap();

        for (out, length, position) in itertools::izip!(out, length, position) {
            if let Some(pos) = position {
                let Some(pos) = pos.cast_to_float() else {
                    return Err(ProcessorError::InputSpecMismatch(0));
                };

                self.pos = pos;

                if pos.fract() != 0.0 {
                    let pos_floor = pos.floor() as usize;
                    let pos_ceil = pos.ceil() as usize;

                    let value_floor = buffer[pos_floor];
                    let value_ceil = buffer[pos_ceil];

                    let t = pos.fract();

                    *out = value_floor + (value_ceil - value_floor) * t.into();
                } else {
                    let pos = pos as i64;

                    if pos < 0 {
                        self.pos = buffer.len() as f64 + pos as f64;
                    } else {
                        self.pos = pos as f64;
                    }

                    *out = buffer[self.pos as usize];
                }
            }

            *length = Some(Message::Int(buffer.len() as i64));
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that reads a sample from a buffer.
    ///
    /// See also: [`BufferReader`].
    pub fn buffer_reader(&self, buffer: impl Into<Buffer<Sample>>) -> Node {
        self.add_processor(BufferReader::new(buffer.into()))
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

impl Process for Register {
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
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let set = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let clear = inputs[1]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let out = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (set, clear, out) in itertools::izip!(set, clear, out) {
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

impl GraphBuilder {
    /// A processor that stores a message in a register.
    ///
    /// See also: [`Register`].
    pub fn register(&self) -> Node {
        self.add_processor(Register::default())
    }
}
