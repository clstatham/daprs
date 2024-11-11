use crate::prelude::*;

/// A processor that packs multiple messages into a single [`Message::List`].
///
/// If any of the inputs are not connected, the output will be `None`.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `a` | `Message` | | The first value to pack. |
/// | `1` | `b` | `Message` | | The second value to pack. |
/// | `...` | `...` | `...` | | Additional values to pack. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Message(List)` | The packed list. |
#[derive(Debug, Clone)]
pub struct Pack {
    inputs: Vec<Message>,
}

impl Pack {
    /// Creates a new `Pack` processor with the given number of inputs.
    pub fn new(num_inputs: usize) -> Self {
        Self {
            inputs: vec![Message::None; num_inputs],
        }
    }
}

impl Processor for Pack {
    fn input_spec(&self) -> Vec<SignalSpec> {
        self.inputs
            .iter()
            .enumerate()
            .map(|(i, _)| SignalSpec::unbounded(i.to_string(), Signal::new_message_none()))
            .collect()
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (sample_idx, out) in outputs.iter_output_mut_as_messages(0)?.enumerate() {
            for (i, input) in self.inputs.iter_mut().enumerate() {
                *input = inputs
                    .input(i)
                    .ok_or(ProcessorError::InputSpecMismatch(i))?
                    .as_message()
                    .ok_or(ProcessorError::InputSpecMismatch(i))?[sample_idx]
                    .clone();
            }

            *out = Message::List(self.inputs.to_vec());
        }

        Ok(())
    }
}

/// A processor that unpacks a [`Message::List`] into multiple messages.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `list` | `Message(List)` | | The list to unpack. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `0` | `Message` | The first value in the list. |
/// | `1` | `1` | `Message` | The second value in the list. |
/// | `...` | `...` | `...` | Additional values in the list. |
#[derive(Debug, Clone)]
pub struct Unpack {
    num_outputs: usize,
}

impl Unpack {
    /// Creates a new `Unpack` processor with the given number of outputs.
    pub fn new(num_outputs: usize) -> Self {
        Self { num_outputs }
    }
}

impl Processor for Unpack {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("list", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::unbounded(i.to_string(), Signal::new_message_none()))
            .collect()
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (sample_idx, list) in inputs.iter_input_as_messages(0)?.enumerate() {
            let Message::List(list) = list else {
                for output_idx in 0..self.num_outputs {
                    outputs.output(output_idx).as_message_mut().unwrap()[sample_idx] =
                        Message::None;
                }
                continue;
            };

            for output_idx in 0..self.num_outputs {
                outputs.output(output_idx).as_message_mut().unwrap()[sample_idx] = list
                    .get(output_idx)
                    .ok_or(ProcessorError::OutputSpecMismatch(output_idx))?
                    .clone();
            }
        }

        Ok(())
    }
}

/// A processor that indexes into a [`Message::List`] and outputs the value at the given index.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `list` | `Message(List)` | | The list to index into. |
/// | `1` | `index` | `Message(Int)` | | The index to retrieve. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Message` | The value at the given index. |
#[derive(Default, Debug, Clone)]
pub struct Index;

impl Processor for Index {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("list", Signal::new_message_none()),
            SignalSpec::unbounded("index", Signal::new_message_none()),
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
        for (out, list, index) in itertools::izip!(
            outputs.iter_output_mut_as_messages(0)?,
            inputs.iter_input_as_messages(0)?,
            inputs.iter_input_as_messages(1)?
        ) {
            let Message::List(list) = list else {
                *out = Message::None;
                continue;
            };

            let Some(index) = index.cast_to_int() else {
                *out = Message::None;
                continue;
            };

            *out = list.get(index as usize).cloned().unwrap_or(Message::None);
        }

        Ok(())
    }
}

/// A processor that outputs the length of a [`Message::List`].
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `list` | `Message(List)` | | The list to get the length of. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Message(Int)` | The length of the list. |
#[derive(Default, Debug, Clone)]
pub struct Len;

impl Processor for Len {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("list", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, list) in itertools::izip!(
            outputs.iter_output_mut_as_messages(0)?,
            inputs.iter_input_as_messages(0)?
        ) {
            let Message::List(list) = list else {
                *out = Message::None;
                continue;
            };

            *out = Message::Int(list.len() as i64);
        }

        Ok(())
    }
}
