//! Processors for working with lists.

use std::marker::PhantomData;

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
pub struct Len;

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
            inputs.iter_input_as_buffers(0)?
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
pub struct Get<S: Signal + Copy>(PhantomData<S>);

impl<S: Signal + Copy> Get<S> {
    /// Creates a new `Get` processor.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: Signal + Copy> Processor for Get<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new(
                "list",
                SignalType::List {
                    size: None,
                    element_type: Some(Box::new(S::signal_type())),
                },
            ),
            SignalSpec::new("index", SignalType::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, list, index) in itertools::izip!(
            outputs.iter_output_as::<S>(0)?,
            inputs.iter_input_as_buffers(0)?,
            inputs.iter_input_as_ints(1)?
        ) {
            let Some(list) = list else {
                *out = None;
                continue;
            };

            if list.type_() != S::signal_type() {
                return Err(ProcessorError::InputSpecMismatch {
                    index: 0,
                    expected: S::signal_type(),
                    actual: list.type_(),
                });
            }

            let Some(index) = index else {
                *out = None;
                continue;
            };

            *out = list
                .as_type::<S>()
                .unwrap()
                .get(index as usize)
                .copied()
                .flatten();
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
pub struct Pack<S: Signal + Copy> {
    inputs: Vec<Option<S>>,
}

impl<S: Signal + Copy> Pack<S> {
    /// Creates a new `Pack` processor with the specified type and number of inputs.
    pub fn new(num_inputs: usize) -> Self {
        Self {
            inputs: vec![None; num_inputs],
        }
    }
}

impl<S: Signal + Copy> Processor for Pack<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        (0..self.inputs.len())
            .map(|i| SignalSpec::new(i.to_string(), S::signal_type()))
            .collect()
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new(
            "out",
            SignalType::List {
                size: Some(self.inputs.len()),
                element_type: Some(Box::new(S::signal_type())),
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
            let mut any_some = false;
            let num_inputs = self.inputs.len();
            for input_index in 0..num_inputs {
                let input = inputs.input(input_index);
                if let Some(buf) = input.as_ref() {
                    let buf = buf.as_type::<S>().unwrap();

                    self.inputs[input_index] = buf[sample_index];

                    if self.inputs[input_index].is_some() {
                        any_some = true;
                    }
                }
            }

            if !any_some {
                // be lazy if all inputs are None
                // this saves us from allocating a list or cloning the inputs
                continue;
            }

            if let Some(out) = out {
                // avoid reallocation if the list is already initialized with the correct length
                if out.len() == self.inputs.len() {
                    let out = out.as_type_mut::<S>().unwrap();
                    out.copy_from(&self.inputs);

                    continue;
                }
            }

            // we should only get here if the list is not initialized or has the wrong length
            error_once!("pack_list" => "list is not initialized or has the wrong length");

            let mut buf = SignalBuffer::new_of_type(&S::signal_type(), self.inputs.len());
            {
                let buf = buf.as_type_mut::<S>().unwrap();
                buf.copy_from(&self.inputs);
            }

            *out = Some(buf);
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
pub struct Unpack<S: Signal + Copy> {
    num_outputs: usize,
    _phantom: PhantomData<S>,
}

impl<S: Signal + Copy> Unpack<S> {
    /// Creates a new `Unpack` processor with the specified type and number of outputs.
    pub fn new(num_outputs: usize) -> Self {
        Self {
            num_outputs,
            _phantom: PhantomData,
        }
    }
}

impl<S: Signal + Copy> Processor for Unpack<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new(
            "list",
            SignalType::List {
                size: Some(self.num_outputs),
                element_type: Some(Box::new(S::signal_type())),
            },
        )]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::new(i.to_string(), S::signal_type()))
            .collect()
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let list_buf = inputs.input(0).unwrap();
        let list_buf = list_buf.as_buffer();

        if let Some(list_buf) = list_buf {
            for (sample_index, list) in list_buf
                .iter()
                .enumerate()
                .filter_map(|(i, s)| s.as_ref().map(|s| (i, s)))
            {
                for output_index in 0..self.num_outputs {
                    let mut output_buf = outputs.output(output_index);
                    let mut output_buf = output_buf.iter_mut::<S>();
                    let list = list.as_type::<S>().unwrap();
                    let out = output_buf.nth(sample_index).unwrap();
                    out.clone_from(&list.get(output_index).copied().flatten());
                }
            }
        }

        Ok(())
    }
}
