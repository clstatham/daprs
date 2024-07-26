use crate::prelude::*;
#[derive(Debug, Clone)]
pub struct Constant {
    pub value: Sample,
    rate: SignalRate,
}

impl Constant {
    pub fn ar(value: Sample) -> Self {
        Self {
            value,
            rate: SignalRate::Audio,
        }
    }
}

impl Constant {
    pub fn kr(value: Sample) -> Self {
        Self {
            value,
            rate: SignalRate::Control,
        }
    }
}

impl Process for Constant {
    fn name(&self) -> &str {
        "constant"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("value"),
            rate: self.rate,
            kind: SignalKind::Buffer,
        }]
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, _inputs: &[Signal], outputs: &mut [Signal]) {
        let out = outputs[0].unwrap_buffer_mut();
        out.fill(self.value);
    }
}

macro_rules! arith_proc {
    ($type_name:ident $str_name:literal $op:tt) => {
        #[derive(Debug, Clone)]
        pub struct $type_name {
            rate: SignalRate,
        }

        impl $type_name {
            pub fn ar() -> Self {
                Self {
                    rate: SignalRate::Audio,
                }
            }
        }

        impl $type_name {
            pub fn kr() -> Self {
                Self {
                    rate: SignalRate::Control,
                }
            }
        }

        impl Process for $type_name {
            fn name(&self) -> &str {
                $str_name
            }

            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec {
                    name: Some("a"),
                    rate: self.rate,
                    kind: SignalKind::Buffer,
                }, SignalSpec {
                    name: Some("b"),
                    rate: self.rate,
                    kind: SignalKind::Buffer,
                }]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec {
                    name: Some("output"),
                    rate: self.rate,
                    kind: SignalKind::Buffer,
                }]
            }

            fn num_inputs(&self) -> usize {
                2
            }

            fn num_outputs(&self) -> usize {
                1
            }

            fn prepare(&mut self) {}

            #[allow(clippy::assign_op_pattern)]
            #[inline]
            fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
                let out = outputs[0].unwrap_buffer_mut();
                out.copy_from_slice(inputs[0].unwrap_buffer());
                for (o, i) in out.iter_mut().zip(inputs[1].unwrap_buffer().iter()) {
                    *o = *o $op *i;
                }
            }
        }
    };
}

arith_proc!(Add "add" +);
arith_proc!(Sub "sub" -);
arith_proc!(Mul "mul" *);
arith_proc!(Div "div" /);
arith_proc!(Rem "rem" %);

macro_rules! cmp_proc {
    ($type_name:ident $str_name:literal $op:tt) => {
        #[derive(Debug, Clone)]
        pub struct $type_name {
            rate: SignalRate,
        }

        impl $type_name {
            pub fn ar() -> Self {
                Self {
                    rate: SignalRate::Audio,
                }
            }
        }

        impl $type_name {
            pub fn kr() -> Self {
                Self {
                    rate: SignalRate::Control,
                }
            }
        }

        impl Process for $type_name {
            fn name(&self) -> &str {
                $str_name
            }

            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec {
                    name: Some("a"),
                    rate: self.rate,
                    kind: SignalKind::Buffer,
                }, SignalSpec {
                    name: Some("b"),
                    rate: self.rate,
                    kind: SignalKind::Buffer,
                }]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec {
                    name: Some("output"),
                    rate: self.rate,
                    kind: SignalKind::Buffer,
                }]
            }

            fn num_inputs(&self) -> usize {
                2
            }

            fn num_outputs(&self) -> usize {
                1
            }

            fn prepare(&mut self) {}

            #[inline]
            fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
                let out = outputs[0].unwrap_buffer_mut();
                out.copy_from_slice(inputs[0].unwrap_buffer());
                for (o, i) in out.iter_mut().zip(inputs[1].unwrap_buffer().iter()) {
                    *o = if *o $op *i { 1.0 } else { 0.0 }.into();
                }
            }
        }
    };
}

cmp_proc!(Gt "gt" >);
cmp_proc!(Lt "lt" <);
cmp_proc!(Ge "ge" >=);
cmp_proc!(Le "le" <=);
cmp_proc!(Eq "eq" ==);

#[derive(Debug, Clone)]
pub struct Clip {
    rate: SignalRate,
}

impl Clip {
    pub fn ar() -> Self {
        Self {
            rate: SignalRate::Audio,
        }
    }
}

impl Clip {
    pub fn kr() -> Self {
        Self {
            rate: SignalRate::Control,
        }
    }
}

impl Process for Clip {
    fn name(&self) -> &str {
        "clip"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec {
                name: Some("input"),
                rate: self.rate,
                kind: SignalKind::Buffer,
            },
            SignalSpec {
                name: Some("min"),
                rate: self.rate,
                kind: SignalKind::Buffer,
            },
            SignalSpec {
                name: Some("max"),
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
        let (in_buf, min_buf, max_buf) = (
            inputs[0].unwrap_buffer(),
            inputs[1].unwrap_buffer(),
            inputs[2].unwrap_buffer(),
        );
        let out = outputs[0].unwrap_buffer_mut();

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

macro_rules! unary_proc {
    ($type_name:ident $str_name:literal $op:tt) => {
        #[derive(Debug, Clone)]
        pub struct $type_name {
            rate: SignalRate,
        }

        impl $type_name {
            pub fn ar() -> Self {
                Self {
                    rate: SignalRate::Audio,
                }
            }
        }

        impl $type_name {
            pub fn kr() -> Self {
                Self {
                    rate: SignalRate::Control,
                }
            }
        }

        impl Process for $type_name {
            fn name(&self) -> &str {
                $str_name
            }

            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec {
                    name: Some("input"),
                    rate: self.rate,
                    kind: SignalKind::Buffer,
                }]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec {
                    name: Some("output"),
                    rate: self.rate,
                    kind: SignalKind::Buffer,
                }]
            }

            fn num_inputs(&self) -> usize {
                1
            }

            fn num_outputs(&self) -> usize {
                1
            }

            fn prepare(&mut self) {}

            #[inline]
            fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
                let out = outputs[0].unwrap_buffer_mut();
                out.copy_from_slice(inputs[0].unwrap_buffer());
                for o in out.iter_mut() {
                    *o = (*o).$op().into();
                }
            }
        }
    };
}

unary_proc!(Sin "sin" sin);
unary_proc!(Cos "cos" cos);
unary_proc!(Sqrt "sqrt" sqrt);
unary_proc!(Abs "abs" abs);
unary_proc!(Exp "exp" exp);
unary_proc!(Ln "ln" ln);
unary_proc!(Recip "recip" recip);

#[derive(Debug, Clone)]
pub struct Neg {
    rate: SignalRate,
}

impl Neg {
    pub fn ar() -> Self {
        Self {
            rate: SignalRate::Audio,
        }
    }
}

impl Neg {
    pub fn kr() -> Self {
        Self {
            rate: SignalRate::Control,
        }
    }
}

impl Process for Neg {
    fn name(&self) -> &str {
        "neg"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("input"),
            rate: self.rate,
            kind: SignalKind::Buffer,
        }]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec {
            name: Some("output"),
            rate: self.rate,
            kind: SignalKind::Buffer,
        }]
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
        let out = outputs[0].unwrap_buffer_mut();
        out.copy_map(inputs[0].unwrap_buffer(), |s| -s);
    }
}

#[derive(Debug, Clone)]
pub struct Pow {
    rate: SignalRate,
}

impl Pow {
    pub fn ar() -> Self {
        Self {
            rate: SignalRate::Audio,
        }
    }
}

impl Pow {
    pub fn kr() -> Self {
        Self {
            rate: SignalRate::Control,
        }
    }
}

impl Process for Pow {
    fn name(&self) -> &str {
        "pow"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec {
                name: Some("base"),
                rate: self.rate,
                kind: SignalKind::Buffer,
            },
            SignalSpec {
                name: Some("exp"),
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
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn prepare(&mut self) {}

    #[inline]
    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
        let out = outputs[0].unwrap_buffer_mut();
        out.copy_from_slice(inputs[0].unwrap_buffer());
        for (o, (b, e)) in out.iter_mut().zip(itertools::izip!(
            inputs[0].unwrap_buffer(),
            inputs[1].unwrap_buffer()
        )) {
            *o = b.powf(**e).into();
        }
    }
}
