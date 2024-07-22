use std::sync::Arc;

use crate::{
    graph::node::Process,
    sample::{Audio, Buffer, Control, Sample, SignalKind, SignalKindMarker},
};

#[derive(Clone)]
pub struct Lambda<K: SignalKindMarker> {
    #[allow(clippy::type_complexity)]
    func: Arc<dyn Fn(&[Sample], &mut [Sample]) + Send + Sync + 'static>,
    _kind: std::marker::PhantomData<K>,
}

impl Lambda<Audio> {
    pub fn ar<F>(func: F) -> Self
    where
        F: Fn(&[Sample], &mut [Sample]) + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(func),
            _kind: std::marker::PhantomData,
        }
    }
}

impl Lambda<Control> {
    pub fn kr<F>(func: F) -> Self
    where
        F: Fn(&[Sample], &mut [Sample]) + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(func),
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Lambda<K> {
    fn name(&self) -> &str {
        "lambda"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        (self.func)(&inputs[0], &mut outputs[0]);
    }
}

#[derive(Debug, Clone)]
pub struct Constant<K: SignalKindMarker> {
    pub value: Sample,
    _kind: std::marker::PhantomData<K>,
}

impl Constant<Audio> {
    pub fn ar(value: Sample) -> Self {
        Self {
            value,
            _kind: std::marker::PhantomData,
        }
    }
}

impl Constant<Control> {
    pub fn kr(value: Sample) -> Self {
        Self {
            value,
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Constant<K> {
    pub fn new(value: Sample) -> Self {
        Self {
            value,
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Constant<K> {
    fn name(&self) -> &str {
        "constant"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, _inputs: &[Buffer], outputs: &mut [Buffer]) {
        let out = &mut outputs[0];
        out.fill(self.value);
    }
}

impl From<f64> for Constant<Control> {
    fn from(value: f64) -> Self {
        Self::kr(value.into())
    }
}

impl From<f64> for Constant<Audio> {
    fn from(value: f64) -> Self {
        Self::ar(value.into())
    }
}

#[derive(Debug, Clone)]
pub struct Add<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Add<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Add<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Add<K> {
    fn name(&self) -> &str {
        "add"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let out = &mut outputs[0];
        out.copy_from_slice(&inputs[0]);
        for (o, i) in out.iter_mut().zip(inputs[1].iter()) {
            *o += *i;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sub<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Sub<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Sub<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Sub<K> {
    fn name(&self) -> &str {
        "sub"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let out = &mut outputs[0];
        out.copy_from_slice(&inputs[0]);
        for (o, i) in out.iter_mut().zip(inputs[1].iter()) {
            *o -= *i;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mul<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Mul<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Mul<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Mul<K> {
    fn name(&self) -> &str {
        "mul"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let out = &mut outputs[0];
        out.copy_from_slice(&inputs[0]);
        for (o, i) in out.iter_mut().zip(inputs[1].iter()) {
            *o *= *i;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Div<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Div<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Div<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Div<K> {
    fn name(&self) -> &str {
        "div"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let out = &mut outputs[0];
        out.copy_from_slice(&inputs[0]);
        for (o, i) in out.iter_mut().zip(inputs[1].iter()) {
            *o /= *i;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rem<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Rem<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Rem<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Rem<K> {
    fn name(&self) -> &str {
        "rem"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let out = &mut outputs[0];
        out.copy_from_slice(&inputs[0]);
        for (o, i) in out.iter_mut().zip(inputs[1].iter()) {
            *o %= *i;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gt<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Gt<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Gt<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Gt<K> {
    fn name(&self) -> &str {
        "gt"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let out = &mut outputs[0];
        for (o, (a, b)) in out.iter_mut().zip(inputs[0].iter().zip(inputs[1].iter())) {
            *o = if a > b { 1.0 } else { 0.0 }.into();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lt<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Lt<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Lt<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Lt<K> {
    fn name(&self) -> &str {
        "lt"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let out = &mut outputs[0];
        for (o, (a, b)) in out.iter_mut().zip(inputs[0].iter().zip(inputs[1].iter())) {
            *o = if a < b { 1.0 } else { 0.0 }.into();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Eq<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Eq<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Eq<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Eq<K> {
    fn name(&self) -> &str {
        "eq"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND, K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        let out = &mut outputs[0];
        for (o, (a, b)) in out.iter_mut().zip(inputs[0].iter().zip(inputs[1].iter())) {
            *o = if a == b { 1.0 } else { 0.0 }.into();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Clip<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Clip<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Clip<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Clip<K> {
    fn name(&self) -> &str {
        "clip"
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
        let (in_buf, min_buf, max_buf) = (&inputs[0], &inputs[1], &inputs[2]);
        let out = &mut outputs[0];

        for (o, i, min, max) in itertools::izip!(
            out.iter_mut(),
            in_buf.iter(),
            min_buf.iter(),
            max_buf.iter()
        ) {
            *o = i.clamp(**min, **max).into();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sin<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Sin<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Sin<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Sin<K> {
    fn name(&self) -> &str {
        "sin"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.sin().into());
    }
}

#[derive(Debug, Clone)]
pub struct Cos<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Cos<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Cos<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Cos<K> {
    fn name(&self) -> &str {
        "cos"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.cos().into());
    }
}

#[derive(Debug, Clone)]
pub struct Sqrt<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Sqrt<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Sqrt<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Sqrt<K> {
    fn name(&self) -> &str {
        "sqrt"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.sqrt().into());
    }
}

#[derive(Debug, Clone)]
pub struct Abs<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Abs<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Abs<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Abs<K> {
    fn name(&self) -> &str {
        "abs"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.abs().into());
    }
}

#[derive(Debug, Clone)]
pub struct Neg<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Neg<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Neg<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Neg<K> {
    fn name(&self) -> &str {
        "neg"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| -s);
    }
}

#[derive(Debug, Clone)]
pub struct Exp<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Exp<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Exp<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Exp<K> {
    fn name(&self) -> &str {
        "exp"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.exp().into());
    }
}

#[derive(Debug, Clone)]
pub struct Ln<K: SignalKindMarker> {
    _kind: std::marker::PhantomData<K>,
}

impl Ln<Audio> {
    pub fn ar() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl Ln<Control> {
    pub fn kr() -> Self {
        Self {
            _kind: std::marker::PhantomData,
        }
    }
}

impl<K: SignalKindMarker> Process for Ln<K> {
    fn name(&self) -> &str {
        "ln"
    }

    fn input_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
    }

    fn output_kinds(&self) -> Vec<SignalKind> {
        vec![K::KIND]
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.ln().into());
    }
}
