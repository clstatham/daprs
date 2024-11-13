//! Processors for working with lists.

use std::marker::PhantomData;

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
pub struct Len;

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
#[derive(Default, Debug, Clone)]
pub struct Get<S: Signal + Clone>(PhantomData<S>);

impl<S: Signal + Clone> Get<S> {
    /// Creates a new `Get` processor.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: Signal + Clone> Processor for Get<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("list", SignalType::List),
            SignalSpec::new("index", SignalType::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::TYPE)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, list, index) in itertools::izip!(
            outputs.iter_output_as::<S>(0)?,
            inputs.iter_input_as_lists(0)?,
            inputs.iter_input_as_ints(1)?
        ) {
            let Some(list) = list else {
                *out = None;
                continue;
            };

            if list.type_() != S::TYPE {
                return Err(ProcessorError::InputSpecMismatch {
                    index: 0,
                    expected: S::TYPE,
                    actual: list.type_(),
                });
            }

            let Some(index) = index else {
                *out = None;
                continue;
            };

            *out = list
                .get(index as usize)
                .and_then(|s| S::try_from_signal(s.clone()));
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
pub struct Pack {
    type_: SignalType,
    inputs: Vec<AnySignal>,
}

impl Pack {
    /// Creates a new `Pack` processor with the specified type and number of inputs.
    pub fn new(type_: SignalType, num_inputs: usize) -> Self {
        Self {
            type_,
            inputs: vec![AnySignal::None(type_); num_inputs],
        }
    }
}

impl Processor for Pack {
    fn input_spec(&self) -> Vec<SignalSpec> {
        (0..self.inputs.len())
            .map(|i| SignalSpec::new(i.to_string(), self.type_))
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
        let out = outputs.output_as_lists(0)?;

        for (sample_index, out) in out.into_iter().enumerate() {
            for (input_index, input) in inputs.inputs.iter().enumerate() {
                if let Some(buf) = input.as_ref() {
                    if buf.type_() != self.type_ {
                        return Err(ProcessorError::InputSpecMismatch {
                            index: input_index,
                            expected: self.type_,
                            actual: buf.type_(),
                        });
                    }

                    self.inputs[input_index] = buf.clone_signal_at(sample_index);
                }
            }

            *out = Some(List::from(self.inputs.clone()));
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
pub struct Unpack {
    type_: SignalType,
    num_outputs: usize,
}

impl Unpack {
    /// Creates a new `Unpack` processor with the specified type and number of outputs.
    pub fn new(type_: SignalType, num_outputs: usize) -> Self {
        Self { type_, num_outputs }
    }
}

impl Processor for Unpack {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("list", SignalType::List)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::new(i.to_string(), self.type_))
            .collect()
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let list_buf = inputs.inputs[0].as_ref().and_then(|s| s.as_list());

        if let Some(list_buf) = list_buf {
            for (sample_index, list) in list_buf
                .iter()
                .enumerate()
                .filter_map(|(i, s)| s.as_ref().map(|s| (i, s)))
            {
                for (output_index, output_buf) in outputs.outputs.iter_mut().enumerate() {
                    if output_buf.type_() != self.type_ {
                        return Err(ProcessorError::OutputSpecMismatch {
                            index: output_index,
                            expected: self.type_,
                            actual: output_buf.type_(),
                        });
                    }

                    match self.type_ {
                        SignalType::Bool => {
                            let output_buf = output_buf.as_bool_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_bool());
                            output_buf[sample_index] = value;
                        }
                        SignalType::Int => {
                            let output_buf = output_buf.as_int_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_int());
                            output_buf[sample_index] = value;
                        }
                        SignalType::Float => {
                            let output_buf = output_buf.as_sample_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_float());
                            output_buf[sample_index] = value;
                        }
                        SignalType::String => {
                            let output_buf = output_buf.as_string_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_string());
                            output_buf[sample_index] = value.cloned();
                        }
                        SignalType::List => {
                            let output_buf = output_buf.as_list_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_list());
                            output_buf[sample_index] = value.cloned();
                        }
                        SignalType::Midi => {
                            let output_buf = output_buf.as_midi_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_midi());
                            output_buf[sample_index] = value.copied();
                        }
                        SignalType::Dynamic => {
                            let output_buf = output_buf.as_dynamic_mut().unwrap();
                            let value = list.get(output_index);
                            output_buf[sample_index] = value.cloned();
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
