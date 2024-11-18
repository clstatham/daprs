//! Processors for working with lists.

use crate::{error_once, prelude::*};

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
        vec![SignalSpec::new(
            "list",
            SignalType::List {
                size: None,
                element_type: None,
            },
        )]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Int)]
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
            SignalSpec::new(
                "list",
                SignalType::List {
                    size: None,
                    element_type: Some(Box::new(self.value.signal_type())),
                },
            ),
            SignalSpec::new("index", SignalType::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.value.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let mut out = outputs.output(0);
        for (i, (list, index)) in itertools::izip!(
            inputs.iter_input_as_lists(0)?,
            inputs.iter_input_as_ints(1)?
        )
        .enumerate()
        {
            let Some(list) = list else {
                out.set_none(i);
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
                out.set_none(i);
                continue;
            };

            out.set(i, list.get(index as usize).unwrap().to_owned());
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
    inputs: SignalBuffer,
}

impl Pack {
    /// Creates a new `Pack` processor with the specified type and number of inputs.
    pub fn new(signal_type: SignalType, num_inputs: usize) -> Self {
        Self {
            inputs: SignalBuffer::new_of_type(&signal_type, num_inputs),
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
        vec![SignalSpec::new(
            "out",
            SignalType::List {
                size: Some(self.inputs.len()),
                element_type: Some(Box::new(self.inputs.signal_type())),
            },
        )]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (sample_index, out) in outputs
            .iter_output_as::<SignalBuffer>(0)
            .unwrap()
            .enumerate()
        {
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

                    match self.inputs.signal_type() {
                        SignalType::Float => {
                            let buf = buf.as_type::<Float>().unwrap();
                            self.inputs.as_type_mut::<Float>().unwrap()[input_index] =
                                buf.get(sample_index).copied().flatten();
                        }
                        SignalType::Int => {
                            let buf = buf.as_type::<i64>().unwrap();
                            self.inputs.as_type_mut::<i64>().unwrap()[input_index] =
                                buf.get(sample_index).copied().flatten();
                        }
                        SignalType::Bool => {
                            let buf = buf.as_type::<bool>().unwrap();
                            self.inputs.as_type_mut::<bool>().unwrap()[input_index] =
                                buf.get(sample_index).copied().flatten();
                        }
                        SignalType::String => {
                            let buf = buf.as_type::<String>().unwrap();
                            self.inputs.as_type_mut::<String>().unwrap()[input_index] =
                                buf.get(sample_index).cloned().flatten();
                        }
                        SignalType::List { .. } => {
                            let buf = buf.as_list().unwrap();
                            self.inputs.as_list_mut().unwrap()[input_index] =
                                buf.get(sample_index).cloned().flatten();
                        }
                        SignalType::Midi => {
                            let buf = buf.as_type::<MidiMessage>().unwrap();
                            self.inputs.as_type_mut::<MidiMessage>().unwrap()[input_index] =
                                buf.get(sample_index).cloned().flatten();
                        }
                    }
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
            error_once!("pack_list" => "list is not initialized or has the wrong length");
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
        vec![SignalSpec::new(
            "list",
            SignalType::List {
                size: Some(self.num_outputs),
                element_type: Some(Box::new(self.signal_type.clone())),
            },
        )]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::new(i.to_string(), self.signal_type.clone()))
            .collect()
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let list_buf = inputs.input(0).unwrap();
        let list_buf = list_buf.as_list();

        if let Some(list_buf) = list_buf {
            for (sample_index, list) in list_buf
                .iter()
                .enumerate()
                .filter_map(|(i, s)| s.as_ref().map(|s| (i, s)))
            {
                for output_index in 0..self.num_outputs {
                    let mut output_buf = outputs.output(output_index);

                    output_buf.set(sample_index, list.get(output_index).unwrap().to_owned());
                }
            }
        }

        Ok(())
    }
}
