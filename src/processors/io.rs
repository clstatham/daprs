use crate::prelude::*;

use super::resample;

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

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![SignalRate::Control]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![SignalRate::Audio]
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
        resample(control, output);
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

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![SignalRate::Audio]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![SignalRate::Control]
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
        resample(audio, output);
    }
}

#[derive(Default, Debug, Clone)]
pub struct DebugPrint<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl DebugPrint<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl DebugPrint<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for DebugPrint<R> {
    fn name(&self) -> &str {
        "debug_print"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let input = &inputs[0];
        println!("{:#?}", input);
        outputs[0].copy_from_slice(input);
    }
}
