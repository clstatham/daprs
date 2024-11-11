use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiNote;

impl Processor for MidiNote {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("midi", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("note", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in itertools::izip!(
            inputs.iter_input_as_messages(0)?,
            outputs.iter_output_mut_as_messages(0)?
        ) {
            *out = None;
            if let Some(Message::Midi(msg)) = midi {
                if msg.len() == 3 {
                    let note = msg[1] as f64;
                    *out = Some(Message::Float(note));
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
        vec![SignalSpec::unbounded("midi", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded(
            "velocity",
            Signal::new_message_none(),
        )]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in itertools::izip!(
            inputs.iter_input_as_messages(0)?,
            outputs.iter_output_mut_as_messages(0)?
        ) {
            *out = None;
            if let Some(Message::Midi(msg)) = midi {
                if msg.len() == 3 {
                    let velocity = msg[2] as f64 / 127.0;
                    *out = Some(Message::Float(velocity));
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
        vec![SignalSpec::unbounded("midi", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("channel", 0.0)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in itertools::izip!(
            inputs.iter_input_as_messages(0)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            *out = 0.0;
            if let Some(Message::Midi(msg)) = midi {
                if msg.len() == 3 {
                    let channel = (msg[0] & 0x0F) as f64;
                    *out = channel;
                }
            }
        }
        Ok(())
    }
}
