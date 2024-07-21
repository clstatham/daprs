use crate::{
    graph::node::Process,
    sample::{Buffer, Sample, SignalKind},
};

/// Resamples the input signal to the output signal's length using a linear interpolation algorithm.
#[inline]
fn linear_resample(input: &[Sample], output: &mut [Sample]) {
    let input_len = input.len();
    let output_len = output.len();
    if input_len == output_len {
        // fast path
        output.copy_from_slice(input);
        return;
    }
    let step = input_len as f64 / output_len as f64;
    let mut i = 0.0;
    for o in output.iter_mut() {
        let i0 = i as usize;
        if i0 >= input_len - 1 {
            *o = input[input_len - 1];
            return;
        }
        let i1 = i0 + 1;
        let a = i - i0 as f64;
        let b = 1.0 - a;
        *o = input[i0] * b.into() + input[i1] * a.into();
        i += step;
    }
}

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

    fn input_kind(&self) -> SignalKind {
        SignalKind::Control
    }

    fn output_kind(&self) -> SignalKind {
        SignalKind::Audio
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
        linear_resample(control, output);
        let alpha = 1.0 - (2.0 * std::f64::consts::PI * 5.0 / self.audio_rate).exp();
        let mut y = *output[0];
        output.map_mut(|s| {
            let next_y = alpha * **s + (1.0 - alpha) * y;
            y = next_y;
            *s = next_y.into();
        });
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

    fn input_kind(&self) -> SignalKind {
        SignalKind::Audio
    }

    fn output_kind(&self) -> SignalKind {
        SignalKind::Control
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
        linear_resample(audio, output);
    }
}
