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
    inputs: Vec<Option<Signal>>,
}

impl Pack {
    /// Creates a new `Pack` processor with the given number of inputs.
    pub fn new(num_inputs: usize) -> Self {
        Self {
            inputs: vec![None; num_inputs],
        }
    }
}

impl Processor for Pack {
    fn input_names(&self) -> Vec<String> {
        (0..self.inputs.len()).map(|i| i.to_string()).collect()
    }

    fn output_spec(&self) -> Vec<OutputSpec> {
        vec![OutputSpec::new("out", SignalKind::List)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (sample_idx, out) in outputs.iter_output_mut_as_lists(0)?.enumerate() {
            for (i, input) in self.inputs.iter_mut().enumerate() {
                *input = Some(
                    inputs
                        .input(i)
                        .ok_or(ProcessorError::InputSpecMismatch(i))?
                        .clone_signal_at(sample_idx),
                );
            }

            *out = Some(self.inputs.iter().cloned().map(Option::unwrap).collect());
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
    fn input_names(&self) -> Vec<String> {
        vec![String::from("list")]
    }

    fn output_spec(&self) -> Vec<OutputSpec> {
        vec![OutputSpec::new("out", SignalKind::Int)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, list) in itertools::izip!(
            outputs.iter_output_mut_as_ints(0)?,
            inputs.iter_input_as_lists(0)?
        ) {
            let Some(list) = list else {
                *out = None;
                continue;
            };

            *out = Some(list.len() as i64);
        }

        Ok(())
    }
}
