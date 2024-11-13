//! Built-in processors for MIDI messages.

use crate::prelude::*;














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
