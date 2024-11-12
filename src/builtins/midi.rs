//! Built-in processors for MIDI messages.

use crate::prelude::*;

/// A processor that extracts the MIDI note number from a MIDI message.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `midi` | `Message(midi)` |  | The MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `note` | `Message(float)` | The MIDI note number. |
#[derive(Debug, Clone)]
pub struct MidiNote;

impl Processor for MidiNote {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("midi", SignalKind::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("note", SignalKind::Sample)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in itertools::izip!(
            inputs.iter_input_as_midi(0)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            *out = None;
            if let Some(msg) = midi {
                if msg.len() == 3 {
                    let note = msg[1] as f64;
                    *out = Some(note);
                }
            }
        }
        Ok(())
    }
}

/// A processor that extracts the MIDI velocity from a MIDI message.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `midi` | `Message(midi)` |  | The MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `velocity` | `Message(float)` | The MIDI velocity. |
#[derive(Debug, Clone)]
pub struct MidiVelocity;

impl Processor for MidiVelocity {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("midi", SignalKind::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("velocity", SignalKind::Sample)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in itertools::izip!(
            inputs.iter_input_as_midi(0)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            *out = None;
            if let Some(msg) = midi {
                if msg.len() == 3 {
                    let velocity = msg[2] as f64;
                    *out = Some(velocity);
                }
            }
        }
        Ok(())
    }
}

/// A processor that extracts the MIDI channel from a MIDI message.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `midi` | `Message(midi)` |  | The MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `channel` | `Message(float)` | The MIDI channel. |
#[derive(Debug, Clone)]
pub struct MidiChannel;

impl Processor for MidiChannel {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("midi", SignalKind::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("channel", SignalKind::Sample)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in itertools::izip!(
            inputs.iter_input_as_midi(0)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            *out = None;
            if let Some(msg) = midi {
                if msg.len() == 3 {
                    let channel = (msg[0] & 0x0F) as f64;
                    *out = Some(channel);
                }
            }
        }
        Ok(())
    }
}
