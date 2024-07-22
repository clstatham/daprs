use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct IfElse<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl IfElse<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl IfElse<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for IfElse<K> {
    fn name(&self) -> &str {
        "if_else"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND, K::KIND]
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
