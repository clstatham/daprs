use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct IfElse<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl IfElse<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl IfElse<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for IfElse<R> {
    fn name(&self) -> &str {
        "if_else"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE, R::RATE, R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
    }

    fn num_inputs(&self) -> usize {
        3
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let cond = &inputs[0];
        let if_true = &inputs[1];
        let if_false = &inputs[2];
        let output = &mut outputs[0];

        for (o, c, tru, fal) in itertools::izip!(output, cond, if_true, if_false) {
            *o = if **c > 0.0 { *tru } else { *fal };
        }
    }
}
