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
            inputs.iter_input_as_buffers(0)?,
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
                .as_type::<S>()
                .unwrap()
                .get(index as usize)
                .cloned()
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
pub struct Pack<S: Signal + Clone + Default> {
    inputs: Vec<S>,
}

impl<S: Signal + Clone + Default> Pack<S> {
    /// Creates a new `Pack` processor with the specified type and number of inputs.
    pub fn new(num_inputs: usize) -> Self {
        Self {
            inputs: vec![S::default(); num_inputs],
        }
    }
}

impl<S: Signal + Clone + Default> Processor for Pack<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        (0..self.inputs.len())
            .map(|i| SignalSpec::new(i.to_string(), S::TYPE))
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
        let out = outputs.output_as_buffers(0)?;

        for (sample_index, out) in out.into_iter().enumerate() {
            for (input_index, input) in inputs.inputs.iter().enumerate() {
                if let Some(buf) = input.as_ref() {
                    let buf =
                        buf.as_type::<S>()
                            .ok_or_else(|| ProcessorError::InputSpecMismatch {
                                index: input_index,
                                expected: S::TYPE,
                                actual: buf.type_(),
                            })?;

                    if let Some(item) = &buf[sample_index] {
                        self.inputs[input_index] = item.clone();
                    }
                }
            }

            if let Some(out) = out {
                // avoid reallocation if the list is already initialized with the correct length
                let out = out.as_type_mut::<S>().unwrap();
                if out.len() == self.inputs.len() {
                    for (i, item) in self.inputs.iter().enumerate() {
                        out[i] = Some(item.clone());
                    }
                } else {
                    // reallocate the list *sigh*
                    // FIXME: how do we avoid this?
                    *out = Buffer::from_slice(&self.inputs);
                }
            } else {
                // FIXME: see above
                let mut buf = SignalBuffer::new_of_kind(S::TYPE, self.inputs.len());
                {
                    let buf = buf.as_type_mut::<S>().unwrap();
                    for (i, item) in self.inputs.iter().enumerate() {
                        buf[i] = Some(item.clone());
                    }
                }

                *out = Some(buf);
            }
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
pub struct Unpack<S: Signal + Clone> {
    num_outputs: usize,
    _phantom: PhantomData<S>,
}

impl<S: Signal + Clone> Unpack<S> {
    /// Creates a new `Unpack` processor with the specified type and number of outputs.
    pub fn new(num_outputs: usize) -> Self {
        Self {
            num_outputs,
            _phantom: PhantomData,
        }
    }
}

impl<S: Signal + Clone> Processor for Unpack<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("list", SignalType::List)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::new(i.to_string(), S::TYPE))
            .collect()
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let list_buf = inputs.inputs[0].as_ref().and_then(|s| s.as_buffer());

        if let Some(list_buf) = list_buf {
            for (sample_index, list) in list_buf
                .iter()
                .enumerate()
                .filter_map(|(i, s)| s.as_ref().map(|s| (i, s)))
            {
                for (output_index, output_buf) in outputs.outputs.iter_mut().enumerate() {
                    let output_buf = output_buf.as_type_mut::<S>().unwrap();
                    let list = list.as_type::<S>().unwrap();
                    if output_index < list.len() {
                        output_buf[sample_index] = list[output_index].clone();
                    } else {
                        output_buf[sample_index] = None;
                    }
                }
            }
        }

        Ok(())
    }
}
