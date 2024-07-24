use std::sync::Arc;

use crate::{
    graph::node::Process,
    sample::{Audio, Buffer, Control, Sample, SignalRate, SignalRateMarker},
};

#[derive(Clone)]
pub struct Lambda<R: SignalRateMarker> {
    #[allow(clippy::type_complexity)]
    func: Arc<dyn Fn(&[Sample], &mut [Sample]) + Send + Sync + 'static>,
    _rate: std::marker::PhantomData<R>,
}

impl Lambda<Audio> {
    pub fn ar<F>(func: F) -> Self
    where
        F: Fn(&[Sample], &mut [Sample]) + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(func),
            _rate: std::marker::PhantomData,
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
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Lambda<R> {
    fn name(&self) -> &str {
        "lambda"
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

    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        (self.func)(&inputs[0], &mut outputs[0]);
    }
}

#[derive(Debug, Clone)]
pub struct Constant<R: SignalRateMarker> {
    pub value: Sample,
    _rate: std::marker::PhantomData<R>,
}

impl Constant<Audio> {
    pub fn ar(value: Sample) -> Self {
        Self {
            value,
            _rate: std::marker::PhantomData,
        }
    }
}

impl Constant<Control> {
    pub fn kr(value: Sample) -> Self {
        Self {
            value,
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Constant<R> {
    pub fn new(value: Sample) -> Self {
        Self {
            value,
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Constant<R> {
    fn name(&self) -> &str {
        "constant"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
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
pub struct Add<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Add<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Add<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Add<R> {
    fn name(&self) -> &str {
        "add"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE, R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
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
pub struct Sub<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Sub<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Sub<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Sub<R> {
    fn name(&self) -> &str {
        "sub"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE, R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
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
pub struct Mul<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Mul<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Mul<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Mul<R> {
    fn name(&self) -> &str {
        "mul"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE, R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
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
pub struct Div<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Div<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Div<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Div<R> {
    fn name(&self) -> &str {
        "div"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE, R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
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
pub struct Rem<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Rem<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Rem<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Rem<R> {
    fn name(&self) -> &str {
        "rem"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE, R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
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
pub struct Gt<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Gt<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Gt<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Gt<R> {
    fn name(&self) -> &str {
        "gt"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE, R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
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
pub struct Lt<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Lt<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Lt<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Lt<R> {
    fn name(&self) -> &str {
        "lt"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE, R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
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
pub struct Eq<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Eq<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Eq<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Eq<R> {
    fn name(&self) -> &str {
        "eq"
    }

    fn input_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE, R::RATE]
    }

    fn output_rates(&self) -> Vec<SignalRate> {
        vec![R::RATE]
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
pub struct Clip<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Clip<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Clip<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Clip<R> {
    fn name(&self) -> &str {
        "clip"
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
pub struct Sin<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Sin<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Sin<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Sin<R> {
    fn name(&self) -> &str {
        "sin"
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.sin().into());
    }
}

#[derive(Debug, Clone)]
pub struct Cos<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Cos<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Cos<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Cos<R> {
    fn name(&self) -> &str {
        "cos"
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.cos().into());
    }
}

#[derive(Debug, Clone)]
pub struct Sqrt<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Sqrt<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Sqrt<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Sqrt<R> {
    fn name(&self) -> &str {
        "sqrt"
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.sqrt().into());
    }
}

#[derive(Debug, Clone)]
pub struct Abs<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Abs<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Abs<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Abs<R> {
    fn name(&self) -> &str {
        "abs"
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.abs().into());
    }
}

#[derive(Debug, Clone)]
pub struct Neg<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Neg<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Neg<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Neg<R> {
    fn name(&self) -> &str {
        "neg"
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| -s);
    }
}

#[derive(Debug, Clone)]
pub struct Exp<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Exp<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Exp<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Exp<R> {
    fn name(&self) -> &str {
        "exp"
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.exp().into());
    }
}

#[derive(Debug, Clone)]
pub struct Ln<R: SignalRateMarker> {
    _rate: std::marker::PhantomData<R>,
}

impl Ln<Audio> {
    pub fn ar() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl Ln<Control> {
    pub fn kr() -> Self {
        Self {
            _rate: std::marker::PhantomData,
        }
    }
}

impl<R: SignalRateMarker> Process for Ln<R> {
    fn name(&self) -> &str {
        "ln"
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
        let out = &mut outputs[0];
        out.copy_map(&inputs[0], |s| s.ln().into());
    }
}
