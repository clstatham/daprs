//! Processors for working with lists.

use crate::prelude::*;

/// A processor that computes the length of a list.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `list` | `List` | The input list. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Int` | The length of the input list. |
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Len;

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Len {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("list", SignalType::List)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Int)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (list, out) in iter_proc_io_as!(inputs as [List], outputs as [i64]) {
            let Some(list) = list else {
                *out = None;
                continue;
            };

            *out = Some(list.len() as i64);
        }

        Ok(())
    }
}

/// A processor that gets an element from a list.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `list` | `List` | The input list. |
/// | `1` | `index` | `Int` | The index of the element to get. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The element at the specified index. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Get {
    value: AnySignal,
}

impl Get {
    /// Creates a new `Get` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self {
            value: AnySignal::default_of_type(&signal_type),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Get {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("list", SignalType::List),
            SignalSpec::new("index", SignalType::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.value.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (list, index, mut out) in iter_proc_io_as!(
            inputs as [List, i64],
            outputs as [Any]
        ) {
            let Some(list) = list else {
                out.set_none();
                continue;
            };

            if list.signal_type() != self.value.signal_type() {
                return Err(ProcessorError::InputSpecMismatch {
                    index: 0,
                    expected: self.value.signal_type(),
                    actual: list.signal_type(),
                });
            }

            let Some(index) = index else {
                out.set_none();
                continue;
            };

            out.clone_from_ref(list.get(*index as usize).unwrap());
        }

        Ok(())
    }
}

/// A processor that packs multiple signals into a list.
///
/// The input signals must all have the same type.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0..n` | `0..n` | `Any` | The input signals to pack. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `List` | The packed list. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pack {
    inputs: List,
}

impl Pack {
    /// Creates a new `Pack` processor with the specified type and number of inputs.
    pub fn new(signal_type: SignalType, num_inputs: usize) -> Self {
        Self {
            inputs: List::new_of_type(signal_type, num_inputs),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Pack {
    fn input_spec(&self) -> Vec<SignalSpec> {
        (0..self.inputs.len())
            .map(|i| SignalSpec::new(i.to_string(), self.inputs.signal_type()))
            .collect()
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::List)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (sample_index, out) in outputs.iter_output_mut_as::<List>(0).unwrap().enumerate() {
            let num_inputs = self.inputs.len();
            for input_index in 0..num_inputs {
                let input = inputs.input(input_index);
                if let Some(buf) = input.as_ref() {
                    if buf.signal_type() != self.inputs.signal_type() {
                        return Err(ProcessorError::InputSpecMismatch {
                            index: input_index,
                            expected: self.inputs.signal_type(),
                            actual: buf.signal_type(),
                        });
                    }

                    self.inputs.set(input_index, buf.get(sample_index).unwrap());
                }
            }

            if let Some(out) = out {
                // avoid reallocation if the list is already initialized with the correct length
                if out.len() == self.inputs.len() {
                    out.clone_from(&self.inputs);

                    continue;
                }
            }

            // we should only get here if the list is not initialized or has the wrong length
            *out = Some(self.inputs.clone());
        }

        Ok(())
    }
}

/// A processor that unpacks a list into multiple signals.
///
/// The output signals will all have the same type.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `list` | `List` | The input list. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0..n` | `0..n` | `Any` | The unpacked signals. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Unpack {
    num_outputs: usize,
    signal_type: SignalType,
}

impl Unpack {
    /// Creates a new `Unpack` processor with the specified type and number of outputs.
    pub fn new(signal_type: SignalType, num_outputs: usize) -> Self {
        Self {
            num_outputs,
            signal_type,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Unpack {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("list", SignalType::List)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::new(i.to_string(), self.signal_type))
            .collect()
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let list_buf = inputs.input(0).unwrap();
        let list_buf = list_buf.as_type::<List>();

        if let Some(list_buf) = list_buf {
            for (sample_index, list) in list_buf
                .iter()
                .enumerate()
                .filter_map(|(i, s)| s.as_ref().map(|s| (i, s)))
            {
                for output_index in 0..self.num_outputs {
                    let mut output_buf = outputs.output(output_index);

                    output_buf.set(sample_index, list.get(output_index).unwrap());
                }
            }
        }

        Ok(())
    }
}
