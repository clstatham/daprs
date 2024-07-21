use crate::{
    graph::node::Process,
    sample::{Buffer, SignalKind},
};

use super::linear_resample;

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

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![SignalKind::Control]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![SignalKind::Audio]
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn reset(&mut self, audio_rate: f64, control_rate: f64, _block_size: usize) {
        self.audio_rate = audio_rate;
        self.control_rate = control_rate;
    }

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let control = &inputs[0];
        let output = &mut outputs[0];
        linear_resample(control, output, self.audio_rate);
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

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![SignalKind::Audio]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![SignalKind::Control]
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn reset(&mut self, audio_rate: f64, control_rate: f64, _block_size: usize) {
        self.audio_rate = audio_rate;
        self.control_rate = control_rate;
    }

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let audio = &inputs[0];
        let output = &mut outputs[0];
        linear_resample(audio, output, self.control_rate);
    }
}
