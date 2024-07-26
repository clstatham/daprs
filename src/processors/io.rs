use crate::prelude::*;

use super::lerp;

/// Smooths a control signal at audio rate using a one-pole filter. This processor outputs an audio rate signal.
#[derive(Default, Debug, Clone)]
pub struct Smooth {
    audio_rate: f64,
    control_rate: f64,
}

impl Process for Smooth {
    fn name(&self) -> &str {
        "smooth"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("kr"),
            rate: SignalRate::Control,
            kind: SignalKind::Buffer,
        }]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("ar"),
            rate: SignalRate::Audio,
            kind: SignalKind::Buffer,
        }]
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn resize_buffers(&mut self, audio_rate: f64, control_rate: f64, _block_size: usize) {
        self.audio_rate = audio_rate;
        self.control_rate = control_rate;
    }

    #[inline]
    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
        let control = inputs[0].unwrap_buffer();
        let output = outputs[0].unwrap_buffer_mut();
        lerp(control, output);
    }
}

/// Quantizes an audio rate signal to a control rate signal. This processor outputs a control rate signal.
#[derive(Default, Debug, Clone)]
pub struct Quantize {
    audio_rate: f64,
    control_rate: f64,
}

impl Process for Quantize {
    fn name(&self) -> &str {
        "quantize"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("ar"),
            rate: SignalRate::Audio,
            kind: SignalKind::Buffer,
        }]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("kr"),
            rate: SignalRate::Control,
            kind: SignalKind::Buffer,
        }]
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn resize_buffers(&mut self, audio_rate: f64, control_rate: f64, _block_size: usize) {
        self.audio_rate = audio_rate;
        self.control_rate = control_rate;
    }

    #[inline]
    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
        let audio = inputs[0].unwrap_buffer();
        let output = outputs[0].unwrap_buffer_mut();
        lerp(audio, output);
    }
}

#[derive(Debug, Clone)]
pub struct DebugPrint {
    rate: SignalRate,
}

impl DebugPrint {
    pub fn ar() -> Self {
        Self {
            rate: SignalRate::Audio,
        }
    }
}

impl DebugPrint {
    pub fn kr() -> Self {
        Self {
            rate: SignalRate::Control,
        }
    }
}

impl Process for DebugPrint {
    fn name(&self) -> &str {
        "debug_print"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("input"),
            rate: self.rate,
            kind: SignalKind::Buffer,
        }]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("output"),
            rate: self.rate,
            kind: SignalKind::Buffer,
        }]
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
        let input = inputs[0].unwrap_buffer();
        println!("{:#?}", input);
        outputs[0].unwrap_buffer_mut().copy_from_slice(input);
    }
}
