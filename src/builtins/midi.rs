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
        vec![SignalSpec::new("midi", SignalType::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("note", SignalType::Float)]
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
                if msg.status() == 0x90 {
                    let note = msg.data1() as Float;
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
        vec![SignalSpec::new("midi", SignalType::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("velocity", SignalType::Float)]
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
                let velocity = msg.data2() as Float;
                *out = Some(velocity);
            }
        }
        Ok(())
    }
}

/// A processor that outputs a gate signal based on a MIDI note on/off message.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | | The MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `gate` | `Bool` | The gate signal. |
#[derive(Debug, Clone, Default)]
pub struct MidiGate {
    gate: bool,
}

impl Processor for MidiGate {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("midi", SignalType::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("gate", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in itertools::izip!(
            inputs.iter_input_as_midi(0)?,
            outputs.iter_output_mut_as_bools(0)?
        ) {
            if let Some(msg) = midi {
                let gate = match msg.status() {
                    0x90 => msg.data2() > 0,
                    0x80 => false,
                    _ => false,
                };
                self.gate = gate;
            }

            *out = Some(self.gate);
        }
        Ok(())
    }
}

/// A processor that triggers a signal based on a MIDI note on message.
///
/// The trigger signal is `true` for a single sample when a note on message is received.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | | The MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trigger` | `Bool` | The trigger signal. |
#[derive(Debug, Clone, Default)]
pub struct MidiTrigger;

impl Processor for MidiTrigger {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("midi", SignalType::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("trigger", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in itertools::izip!(
            inputs.iter_input_as_midi(0)?,
            outputs.iter_output_mut_as_bools(0)?
        ) {
            *out = None;
            if let Some(msg) = midi {
                if msg.status() == 0x90 && msg.data2() > 0 {
                    *out = Some(true);
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
        vec![SignalSpec::new("midi", SignalType::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("channel", SignalType::Float)]
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
                let channel = msg.channel() as Float;
                *out = Some(channel);
            }
        }
        Ok(())
    }
}
