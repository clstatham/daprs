use crate::prelude::*;

use super::lerp;

#[derive(Debug, Clone)]
pub struct DecayEnv {
    audio_rate: f64,
    control_rate: f64,
    level: f64,
    decay_buf: Buffer,
    curve_buf: Buffer,
    rate: SignalRate,
}

impl DecayEnv {
    pub fn ar() -> Self {
        Self {
            audio_rate: 0.0,
            control_rate: 0.0,
            level: 0.0,
            decay_buf: Buffer::zeros(0),
            curve_buf: Buffer::zeros(0),
            rate: SignalRate::Audio,
        }
    }
}

impl DecayEnv {
    pub fn kr() -> Self {
        Self {
            audio_rate: 0.0,
            control_rate: 0.0,
            level: 0.0,
            decay_buf: Buffer::zeros(0),
            curve_buf: Buffer::zeros(0),
            rate: SignalRate::Control,
        }
    }
}

impl Process for DecayEnv {
    fn name(&self) -> &str {
        "decay_env"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec {
                name: Some("trigger"),
                rate: SignalRate::Audio,
                kind: SignalKind::Buffer,
            },
            SignalSpec {
                name: Some("decay"),
                rate: SignalRate::Control,
                kind: SignalKind::Buffer,
            },
            SignalSpec {
                name: Some("curve"),
                rate: SignalRate::Control,
                kind: SignalKind::Buffer,
            },
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("output"),
            rate: self.rate,
            kind: SignalKind::Buffer,
        }]
    }

    fn num_inputs(&self) -> usize {
        3
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn resize_buffers(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        self.audio_rate = audio_rate;
        self.control_rate = control_rate;
        let control_block_size = block_size / self.control_rate as usize;
        match self.rate {
            SignalRate::Audio => {
                self.decay_buf.resize(block_size);
                self.curve_buf.resize(block_size);
            }
            SignalRate::Control => {
                self.decay_buf.resize(control_block_size);
                self.curve_buf.resize(control_block_size);
            }
        }
    }

    #[inline]
    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
        let trigger = inputs[0].unwrap_buffer();
        let decay = inputs[1].unwrap_buffer();
        let curve = inputs[2].unwrap_buffer();

        lerp(decay, &mut self.decay_buf);
        lerp(curve, &mut self.curve_buf);

        let output = outputs[0].unwrap_buffer_mut();

        let rate = match self.rate {
            SignalRate::Audio => self.audio_rate,
            SignalRate::Control => self.control_rate,
        };

        for (output, trigger, decay, curve) in
            itertools::izip!(output, trigger, &self.decay_buf, &self.curve_buf)
        {
            if **trigger <= 0.0 {
                self.level = 1.0;
            } else {
                let decay_samples = (**decay * rate) as u64;
                let decay_samples = decay_samples.max(1);

                let curve = **curve;

                // exponential decay
                let decay = (1.0 - curve).powf(1.0 / decay_samples as f64);
                self.level *= decay;

                self.level = self.level.clamp(0.0, 1.0);
            }

            *output = self.level.into();
        }
    }
}
