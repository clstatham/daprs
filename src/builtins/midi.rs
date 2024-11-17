//! Built-in processors for MIDI messages.

use crate::prelude::*;

/// A processor that extracts the note number from a MIDI message.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | The input MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `note` | `Float` | The note number of the input MIDI message. |
#[derive(Debug, Clone, Default)]
pub struct MidiNote {
    note: Float,
}

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
            outputs.iter_output_mut_as_floats(0)?
        ) {
            if let Some(msg) = midi {
                if msg.status() == 0x90 {
                    self.note = msg.data1() as Float;
                }
            }

            *out = Some(self.note);
        }
        Ok(())
    }
}

/// A processor that extracts the velocity from a MIDI message.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | The input MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `velocity` | `Float` | The velocity of the input MIDI message. |
#[derive(Debug, Clone, Default)]
pub struct MidiVelocity {
    velocity: Float,
}

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
            outputs.iter_output_mut_as_floats(0)?
        ) {
            if let Some(msg) = midi {
                self.velocity = msg.data2() as Float;
            }

            *out = Some(self.velocity);
        }
        Ok(())
    }
}

/// A processor that outputs a gate signal from a MIDI note on/off message.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | The input MIDI message. |
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

/// A processor that outputs a trigger signal from a MIDI note on message.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | The input MIDI message. |
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

/// A processor that outputs the channel number from a MIDI message.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | The input MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `channel` | `Float` | The channel number of the input MIDI message. |
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
            outputs.iter_output_mut_as_floats(0)?
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
