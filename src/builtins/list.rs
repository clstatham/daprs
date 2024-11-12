use std::marker::PhantomData;

use crate::prelude::*;

/// A processor that outputs the length of a [`List`].
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `list` | `List` | | The list to get the length of. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Int` | The length of the list. |
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

/// A processor that outputs the element at the given index of a [`List`].
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `list` | `List` | | The list to get the element from. |
/// | `1` | `index` | `Int` | | The index of the element to get. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Signal` | The element at the given index. |
#[derive(Default, Debug, Clone)]
pub struct Get<S: SignalData>(PhantomData<S>);

impl<S: SignalData> Get<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: SignalData> Processor for Get<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("list", SignalKind::List),
            SignalSpec::new("index", SignalKind::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::KIND)]
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

            if list.kind() != S::KIND {
                return Err(ProcessorError::InputSpecMismatch {
                    index: 0,
                    expected: S::KIND,
                    actual: list.kind(),
                });
            }

            let Some(index) = index else {
                *out = None;
                continue;
            };

            *out = list.get(index as usize).as_ref().and_then(|s| s.cast());
        }

        Ok(())
    }
}

/// A processor that packs its input signals into a [`List`].
///
/// The signals must all have the same type.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0..N` | `0..N` | `Signal` | | The signals to pack into a list. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `List` | The list containing the input signals. |
#[derive(Debug, Clone)]
pub struct Pack {
    kind: SignalKind,
    inputs: Vec<Signal>,
}

impl Pack {
    pub fn new(kind: SignalKind, num_inputs: usize) -> Self {
        Self {
            kind,
            inputs: vec![Signal::None(kind); num_inputs],
        }
    }
}

impl Processor for Pack {
    fn input_spec(&self) -> Vec<SignalSpec> {
        (0..self.inputs.len())
            .map(|i| SignalSpec::new(i.to_string(), self.kind))
            .collect()
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::List)]
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
                    if buf.kind() != self.kind {
                        return Err(ProcessorError::InputSpecMismatch {
                            index: input_index,
                            expected: self.kind,
                            actual: buf.kind(),
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

/// A processor that unpacks a [`List`] into its elements.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `list` | `List` | | The list to unpack. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0..N` | `0..N` | `Signal` | The unpacked signals. |
#[derive(Debug, Clone)]
pub struct Unpack {
    kind: SignalKind,
    num_outputs: usize,
}

impl Unpack {
    pub fn new(kind: SignalKind, num_outputs: usize) -> Self {
        Self { kind, num_outputs }
    }
}

impl Processor for Unpack {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("list", SignalKind::List)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::new(i.to_string(), self.kind))
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
                    if output_buf.kind() != self.kind {
                        return Err(ProcessorError::OutputSpecMismatch {
                            index: output_index,
                            expected: self.kind,
                            actual: output_buf.kind(),
                        });
                    }

                    match self.kind {
                        SignalKind::Bool => {
                            let output_buf = output_buf.as_bool_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_bool());
                            output_buf[sample_index] = value;
                        }
                        SignalKind::Int => {
                            let output_buf = output_buf.as_int_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_int());
                            output_buf[sample_index] = value;
                        }
                        SignalKind::Sample => {
                            let output_buf = output_buf.as_sample_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_sample());
                            output_buf[sample_index] = value;
                        }
                        SignalKind::String => {
                            let output_buf = output_buf.as_string_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_string());
                            output_buf[sample_index] = value.cloned();
                        }
                        SignalKind::List => {
                            let output_buf = output_buf.as_list_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_list());
                            output_buf[sample_index] = value.cloned();
                        }
                        SignalKind::Midi => {
                            let output_buf = output_buf.as_midi_mut().unwrap();
                            let value = list.get(output_index).and_then(|s| s.as_midi());
                            output_buf[sample_index] = value.copied();
                        }
                        SignalKind::Dynamic => {
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
