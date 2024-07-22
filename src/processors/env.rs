use crate::{prelude::*, sample::SignalKindMarker};

use super::resample;

#[derive(Debug, Clone)]
pub struct DecayEnv<K: SignalKindMarker> {
    audio_rate: f64,
    control_rate: f64,
    level: f64,
    decay_buf: Buffer,
    curve_buf: Buffer,
    _kind: std::marker::PhantomData<K>,
}

impl DecayEnv<Audio> {
    pub fn ar() -> Self {
        Self {
            audio_rate: 0.0,
            control_rate: 0.0,
            level: 0.0,
            decay_buf: Buffer::zeros(0, SignalKind::Control),
            curve_buf: Buffer::zeros(0, SignalKind::Control),
            _kind: std::marker::PhantomData,
        }
    }
}

impl DecayEnv<Control> {
    pub fn kr() -> Self {
        Self {
            audio_rate: 0.0,
            control_rate: 0.0,
            level: 0.0,
            decay_buf: Buffer::zeros(0, SignalKind::Control),
            curve_buf: Buffer::zeros(0, SignalKind::Control),
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for DecayEnv<K> {
    fn name(&self) -> &str {
        "decay_env"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, SignalKind::Control, SignalKind::Control]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        3
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn reset(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        self.audio_rate = audio_rate;
        self.control_rate = control_rate;
        let control_block_size = block_size / self.control_rate as usize;
        match K::KIND {
            SignalKind::Audio => {
                self.decay_buf.resize(block_size);
                self.curve_buf.resize(block_size);
            }
            SignalKind::Control => {
                self.decay_buf.resize(control_block_size);
                self.curve_buf.resize(control_block_size);
            }
        }
    }

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let trigger = &inputs[0];
        let decay = &inputs[1];
        let curve = &inputs[2];

        resample(decay, &mut self.decay_buf);
        resample(curve, &mut self.curve_buf);

        let output = &mut outputs[0];

        let rate = match K::KIND {
            SignalKind::Audio => self.audio_rate,
            SignalKind::Control => self.control_rate,
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
