use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct BufferReaderProc {
    buffer: SignalBuffer,
    sample_rate: f64,
    t: usize,
}

impl BufferReaderProc {
    pub fn new(buffer: Buffer<Sample>) -> Self {
        Self {
            buffer: SignalBuffer::Sample(buffer),
            sample_rate: 0.0,
            t: 0,
        }
    }
}

impl Process for BufferReaderProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("t", Signal::new_message_some(0))]
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
        let t = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        let buffer = self.buffer.as_sample().unwrap();

        for (out, t) in itertools::izip!(out, t) {
            if let Some(t) = t {
                let Some(&t) = (**t).downcast_ref::<i64>() else {
                    return Err(ProcessorError::InputSpecMismatch(0));
                };

                let t = if t < 0 { buffer.len() as i64 + t } else { t } as usize;

                self.t = t % buffer.len();
            }

            *out = buffer[self.t];
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
    /// | `0` | `t` | `Message(i64)` | `0` | The sample index to read from the buffer. |
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
