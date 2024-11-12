use crate::prelude::*;

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
        vec![SignalSpec::new("list", SignalKind::List)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Int)]
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
