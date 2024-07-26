use crate::{
    prelude::*,
    signal::{Signal, SignalKind, SignalSpec},
};

#[derive(Debug, Clone)]
pub struct IfElse {
    rate: SignalRate,
}

impl IfElse {
    pub fn ar() -> Self {
        Self {
            rate: SignalRate::Audio,
        }
    }

    pub fn kr() -> Self {
        Self {
            rate: SignalRate::Control,
        }
    }
}

impl Process for IfElse {
    fn name(&self) -> &str {
        "if_else"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec {
                name: Some("condition"),
                rate: self.rate,
                kind: SignalKind::Buffer,
            },
            SignalSpec {
                name: Some("then"),
                rate: self.rate,
                kind: SignalKind::Buffer,
            },
            SignalSpec {
                name: Some("else"),
                rate: self.rate,
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

    #[inline]
    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
        let cond = inputs[0].unwrap_buffer();
        let if_true = inputs[1].unwrap_buffer();
        let if_false = inputs[2].unwrap_buffer();
        let output = outputs[0].unwrap_buffer_mut();

        for (o, c, tru, fal) in itertools::izip!(output, cond, if_true, if_false) {
            *o = if **c > 0.0 { *tru } else { *fal };
        }
    }
}
