use crate::{
    graph::node::Process,
    sample::{Buffer, Sample},
};

#[derive(Debug, Clone)]
pub struct Constant {
    pub value: Sample,
}

impl Constant {
    pub fn new(value: Sample) -> Self {
        Self { value }
    }
}

impl Process for Constant {
    fn name(&self) -> &str {
        "constant"
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

#[derive(Debug, Clone)]
pub struct Add;

impl Process for Add {
    fn name(&self) -> &str {
        "add"
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
pub struct Sub;

impl Process for Sub {
    fn name(&self) -> &str {
        "sub"
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
pub struct Mul;

impl Process for Mul {
    fn name(&self) -> &str {
        "mul"
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
pub struct Div;

impl Process for Div {
    fn name(&self) -> &str {
        "div"
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
pub struct Rem;

impl Process for Rem {
    fn name(&self) -> &str {
        "rem"
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
pub struct Sin;

impl Process for Sin {
    fn name(&self) -> &str {
        "sin"
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
        out.copy_from_slice(&inputs[0]);
        out.map_mut(|s| {
            *s = s.sin().into();
        });
    }
}

#[derive(Debug, Clone)]
pub struct Cos;

impl Process for Cos {
    fn name(&self) -> &str {
        "cos"
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
        out.copy_from_slice(&inputs[0]);
        out.map_mut(|s| {
            *s = s.cos().into();
        });
    }
}

#[derive(Debug, Clone)]
pub struct Sqrt;

impl Process for Sqrt {
    fn name(&self) -> &str {
        "sqrt"
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
        out.copy_from_slice(&inputs[0]);
        out.map_mut(|s| {
            *s = s.sqrt().into();
        });
    }
}

#[derive(Debug, Clone)]
pub struct Abs;

impl Process for Abs {
    fn name(&self) -> &str {
        "abs"
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
        out.copy_from_slice(&inputs[0]);
        out.map_mut(|s| {
            *s = s.abs().into();
        });
    }
}

#[derive(Debug, Clone)]
pub struct Neg;

impl Process for Neg {
    fn name(&self) -> &str {
        "neg"
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
        out.copy_from_slice(&inputs[0]);
        out.map_mut(|s| {
            *s = -*s;
        });
    }
}

#[derive(Debug, Clone)]
pub struct Exp;

impl Process for Exp {
    fn name(&self) -> &str {
        "exp"
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
        out.copy_from_slice(&inputs[0]);
        out.map_mut(|s| {
            *s = s.exp().into();
        });
    }
}

#[derive(Debug, Clone)]
pub struct Ln;

impl Process for Ln {
    fn name(&self) -> &str {
        "ln"
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
        out.copy_from_slice(&inputs[0]);
        out.map_mut(|s| {
            *s = s.ln().into();
        });
    }
}
