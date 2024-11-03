use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferReaderProc {
    buffer: SignalBuffer,
    sample_rate: f64,
    pos: usize,
}

impl BufferReaderProc {
    pub fn new(buffer: Buffer<Sample>) -> Self {
        Self {
            buffer: SignalBuffer::Sample(buffer),
            sample_rate: 0.0,
            pos: 0,
        }
    }
}

#[typetag::serde]
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
