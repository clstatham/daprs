use crate::prelude::*;
use std::ops::*;

#[derive(Clone, Debug)]
pub struct ConstantProc {
    value: f64,
    out: Param,
}

impl ConstantProc {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}

impl Default for ConstantProc {
    fn default() -> Self {
        Self {
            value: 0.0,
            out: Param::default_with_name("out"),
        }
    }
}

impl Process for ConstantProc {
    fn input_params(&self) -> Vec<Param> {
        vec![]
    }

    fn output_params(&self) -> Vec<Param> {
        vec![self.out]
    }

    fn process(&mut self, _inputs: &[Buffer], outputs: &mut [Buffer]) {
        let out = &mut outputs[0];

        for sample in out {
            *sample = self.value.into();
        }
    }
}

macro_rules! impl_binary_proc {
    ($name:ident, $method:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, Debug)]
        pub struct $name {
            in1: Param,
            in2: Param,
            out: Param,
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    in1: Param::default_with_name("in1"),
                    in2: Param::default_with_name("in2"),
                    out: Param::default_with_name("out"),
                }
            }
        }

        impl Process for $name {
            fn input_params(&self) -> Vec<Param> {
                vec![self.in1, self.in2]
            }

            fn output_params(&self) -> Vec<Param> {
                vec![self.out]
            }

            fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
                let in1 = &inputs[0];
                let in2 = &inputs[1];
                let out = &mut outputs[0];

                for (sample, in1, in2) in itertools::izip!(out, in1, in2) {
                    *sample = (**in1).$method(**in2).into();
                }
            }
        }
    };
}

impl_binary_proc!(AddProc, add, "A processor that adds two signals together.");
impl_binary_proc!(
    SubProc,
    sub,
    "A processor that subtracts one signal from another."
);
impl_binary_proc!(
    MulProc,
    mul,
    "A processor that multiplies two signals together."
);
impl_binary_proc!(
    DivProc,
    div,
    "A processor that divides one signal by another."
);
impl_binary_proc!(
    RemProc,
    rem,
    "A processor that calculates the remainder of one signal divided by another."
);
impl_binary_proc!(
    PowfProc,
    powf,
    "A processor that raises one signal to the power of another."
);
impl_binary_proc!(
    Atan2Proc,
    atan2,
    "A processor that calculates the arctangent of the ratio of two signals."
);
impl_binary_proc!(
    HypotProc,
    hypot,
    "A processor that calculates the hypotenuse of two signals."
);
impl_binary_proc!(
    MaxProc,
    max,
    "A processor that calculates the maximum of two signals."
);
impl_binary_proc!(
    MinProc,
    min,
    "A processor that calculates the minimum of two signals."
);

macro_rules! impl_unary_proc {
    ($name:ident, $method:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, Debug)]
        pub struct $name {
            in1: Param,
            out: Param,
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    in1: Param::default_with_name("in"),
                    out: Param::default_with_name("out"),
                }
            }
        }

        impl Process for $name {
            fn input_params(&self) -> Vec<Param> {
                vec![self.in1]
            }

            fn output_params(&self) -> Vec<Param> {
                vec![self.out]
            }

            fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
                let in1 = &inputs[0];
                let out = &mut outputs[0];

                for (sample, in1) in itertools::izip!(out, in1) {
                    *sample = (**in1).$method().into();
                }
            }
        }
    };
}

impl_unary_proc!(NegProc, neg, "A processor that negates a signal.");
impl_unary_proc!(
    AbsProc,
    abs,
    "A processor that calculates the absolute value of a signal."
);
impl_unary_proc!(
    SqrtProc,
    sqrt,
    "A processor that calculates the square root of a signal."
);
impl_unary_proc!(
    CbrtProc,
    cbrt,
    "A processor that calculates the cube root of a signal."
);
impl_unary_proc!(
    CeilProc,
    ceil,
    "A processor that rounds a signal up to the nearest integer."
);
impl_unary_proc!(
    FloorProc,
    floor,
    "A processor that rounds a signal down to the nearest integer."
);
impl_unary_proc!(
    RoundProc,
    round,
    "A processor that rounds a signal to the nearest integer."
);
impl_unary_proc!(
    TruncProc,
    trunc,
    "A processor that truncates a signal to an integer."
);
impl_unary_proc!(
    FractProc,
    fract,
    "A processor that returns the fractional part of a signal."
);
impl_unary_proc!(
    RecipProc,
    recip,
    "A processor that calculates the reciprocal of a signal."
);
impl_unary_proc!(
    SignumProc,
    signum,
    "A processor that returns the sign of a signal."
);
impl_unary_proc!(
    SinProc,
    sin,
    "A processor that calculates the sine of a signal."
);
impl_unary_proc!(
    CosProc,
    cos,
    "A processor that calculates the cosine of a signal."
);
impl_unary_proc!(
    TanProc,
    tan,
    "A processor that calculates the tangent of a signal."
);
impl_unary_proc!(
    AsinProc,
    asin,
    "A processor that calculates the arcsine of a signal."
);
impl_unary_proc!(
    AcosProc,
    acos,
    "A processor that calculates the arccosine of a signal."
);
impl_unary_proc!(
    AtanProc,
    atan,
    "A processor that calculates the arctangent of a signal."
);
impl_unary_proc!(
    SinhProc,
    sinh,
    "A processor that calculates the hyperbolic sine of a signal."
);
impl_unary_proc!(
    CoshProc,
    cosh,
    "A processor that calculates the hyperbolic cosine of a signal."
);
impl_unary_proc!(
    TanhProc,
    tanh,
    "A processor that calculates the hyperbolic tangent of a signal."
);
impl_unary_proc!(
    AsinhProc,
    asinh,
    "A processor that calculates the hyperbolic arcsine of a signal."
);
impl_unary_proc!(
    AcoshProc,
    acosh,
    "A processor that calculates the hyperbolic arccosine of a signal."
);
impl_unary_proc!(
    AtanhProc,
    atanh,
    "A processor that calculates the hyperbolic arctangent of a signal."
);
impl_unary_proc!(
    ExpProc,
    exp,
    "A processor that calculates the exponential of a signal."
);
impl_unary_proc!(
    Exp2Proc,
    exp2,
    "A processor that calculates 2 raised to the power of a signal."
);
impl_unary_proc!(
    ExpM1Proc,
    exp_m1,
    "A processor that calculates the exponential of a signal minus 1."
);
impl_unary_proc!(
    LnProc,
    ln,
    "A processor that calculates the natural logarithm of a signal."
);
impl_unary_proc!(
    Log2Proc,
    log2,
    "A processor that calculates the base-2 logarithm of a signal."
);
impl_unary_proc!(
    Log10Proc,
    log10,
    "A processor that calculates the base-10 logarithm of a signal."
);
