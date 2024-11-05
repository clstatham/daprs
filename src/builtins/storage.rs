//! Storage-related processors.

use crate::prelude::*;

/// A processor that reads a sample from a buffer.
///
/// See also: [`GraphBuilder::buffer_reader`](crate::builder::graph_builder::GraphBuilder::buffer_reader).
#[derive(Clone, Debug)]

pub struct BufferReaderProc {
    buffer: SignalBuffer,
    sample_rate: f64,
    pos: usize,
}

impl BufferReaderProc {
    /// Creates a new buffer reader processor.
    pub fn new(buffer: Buffer<Sample>) -> Self {
        Self {
            buffer: SignalBuffer::Sample(buffer),
            sample_rate: 0.0,
            pos: 0,
        }
    }
}

impl Process for BufferReaderProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded(
            "position",
            Signal::new_message_some(Message::Int(0)),
        )]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
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

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        let buffer = self.buffer.as_sample().unwrap();

        for (out, position) in itertools::izip!(out, position) {
            if let Some(pos) = position {
                let Some(pos) = pos.cast_to_int() else {
                    return Err(ProcessorError::InputSpecMismatch(0));
                };

                let pos = if pos < 0 {
                    buffer.len() as i64 + pos
                } else {
                    pos
                } as usize;

                self.pos = pos % buffer.len();
            }

            *out = buffer[self.pos];
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that reads a sample from a buffer.
    ///
    /// If the index is out of bounds, it will wrap around.
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
    pub fn buffer_reader(&self, buffer: impl Into<Buffer<Sample>>) -> Node {
        self.add_processor(BufferReaderProc::new(buffer.into()))
    }
}

/// A processor that stores a message in a register.
///
/// See also: [`GraphBuilder::register`](crate::builder::graph_builder::GraphBuilder::register).
#[derive(Clone, Debug, Default)]
pub struct RegisterProc {
    value: Option<Message>,
}

impl Process for RegisterProc {
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
    pub fn register(&self) -> Node {
        self.add_processor(RegisterProc::default())
    }
}
