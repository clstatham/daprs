//! Mathematical processors.

use crate::{prelude::*, processor::ProcessorError, signal::SignalBuffer};
use std::ops::*;

/// A processor that outputs a constant value.
///
/// See also: [`GraphBuilder::constant`](crate::builder::graph_builder::GraphBuilder::constant).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstantProc {
    value: f64,
}

impl ConstantProc {
    /// Creates a new constant processor with the given value.
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl Default for ConstantProc {
    fn default() -> Self {
        Self { value: 0.0 }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Process for ConstantProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", self.value)]
    }

    fn process(
        &mut self,
        _inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let out = &mut outputs[0];

        out.fill(self.value);

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that outputs a constant value.
    ///
    /// # Outputs
    ///
    /// | Index | Name | Default | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `out` | `0.0` | The constant value. |
    pub fn constant(&self, value: f64) -> Node {
        self.add_processor(ConstantProc::new(value))
    }
}

macro_rules! impl_binary_proc {
    ($name:ident, $method:ident, $doc:expr) => {
        #[derive(Clone, Debug, Default)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[doc = $doc]
        pub struct $name;

        #[cfg_attr(feature = "serde", typetag::serde)]
        impl Process for $name {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![
                    SignalSpec::unbounded("a", 0.0),
                    SignalSpec::unbounded("b", 0.0),
                ]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::unbounded("out", 0.0)]
            }

            fn process(
                &mut self,
                inputs: &[SignalBuffer],
                outputs: &mut [SignalBuffer],
            ) -> Result<(), ProcessorError> {
                let in1 = inputs[0]
                    .as_sample()
                    .ok_or(ProcessorError::InputSpecMismatch(0))?;
                let in2 = inputs[1]
                    .as_sample()
                    .ok_or(ProcessorError::InputSpecMismatch(1))?;
                let out = outputs[0]
                    .as_sample_mut()
                    .ok_or(ProcessorError::OutputSpecMismatch(0))?;

                for (sample, in1, in2) in itertools::izip!(out, in1, in2) {
                    *sample = (**in1).$method(**in2).into();
                }

                Ok(())
            }
        }

        impl GraphBuilder {
            #[doc = $doc]
            pub fn $method(&self) -> Node {
                self.add_processor($name)
            }
        }
    };
}

impl_binary_proc!(
    AddProc,
    add,
    r#"
A processor that adds two signals together.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Sample` | `0.0` | The first signal to add. |
| `1` | `b` | `Sample` | `0.0` | The second signal to add. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The sum of the two input signals. |
    "#
);
impl_binary_proc!(
    SubProc,
    sub,
    r#"
A processor that subtracts one signal from another.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Sample` | `0.0` | The signal to subtract from. |
| `1` | `b` | `Sample` | `0.0` | The signal to subtract. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The difference of the two input signals. |
    "#
);
impl_binary_proc!(
    MulProc,
    mul,
    r#"
A processor that multiplies two signals together.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Sample` | `0.0` | The first signal to multiply. |
| `1` | `b` | `Sample` | `0.0` | The second signal to multiply. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The product of the two input signals. |
    "#
);
impl_binary_proc!(
    DivProc,
    div,
    r#"
A processor that divides one signal by another.

Note that the second input defaults to `0.0`, so be sure to provide a non-zero value.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Sample` | `0.0` | The signal to divide. |
| `1` | `b` | `Sample` | `0.0` | The signal to divide by. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The quotient of the two input signals. |
    "#
);
impl_binary_proc!(
    RemProc,
    rem,
    r#"
A processor that calculates the remainder of one signal divided by another.

Note that the second input defaults to `0.0`, so be sure to provide a non-zero value.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Sample` | `0.0` | The signal to divide. |
| `1` | `b` | `Sample` | `0.0` | The signal to divide by. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The remainder of the two input signals. |
    "#
);
impl_binary_proc!(
    PowfProc,
    powf,
    r#"
A processor that raises one signal to the power of another.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Sample` | `0.0` | The base signal. |
| `1` | `b` | `Sample` | `0.0` | The exponent signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The base signal raised to the power of the exponent signal. |
    "#
);
impl_binary_proc!(
    Atan2Proc,
    atan2,
    r#"
A processor that calculates the arctangent of the ratio of two signals.

Note that the second input defaults to `0.0`, so be sure to provide a non-zero value.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Sample` | `0.0` | The first signal. |
| `1` | `b` | `Sample` | `0.0` | The second signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The arctangent of the ratio of the two input signals. |
    "#
);
impl_binary_proc!(
    HypotProc,
    hypot,
    r#"
A processor that calculates the hypotenuse of two signals.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Sample` | `0.0` | The first signal. |
| `1` | `b` | `Sample` | `0.0` | The second signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The hypotenuse of the two input signals. |
    "#
);
impl_binary_proc!(
    MaxProc,
    max,
    r#"
A processor that calculates the maximum of two signals.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Sample` | `0.0` | The first signal. |
| `1` | `b` | `Sample` | `0.0` | The second signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The maximum of the two input signals. |
    "#
);
impl_binary_proc!(
    MinProc,
    min,
    r#"
A processor that calculates the minimum of two signals.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Sample` | `0.0` | The first signal. |
| `1` | `b` | `Sample` | `0.0` | The second signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The minimum of the two input signals. |
    "#
);

macro_rules! impl_unary_proc {
    ($name:ident, $method:ident, $doc:expr) => {
        #[derive(Clone, Debug, Default)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[doc = $doc]
        pub struct $name;

        #[cfg_attr(feature = "serde", typetag::serde)]
        impl Process for $name {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::unbounded("in", 0.0)]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::unbounded("out", 0.0)]
            }

            fn process(
                &mut self,
                inputs: &[SignalBuffer],
                outputs: &mut [SignalBuffer],
            ) -> Result<(), ProcessorError> {
                let in1 = inputs[0]
                    .as_sample()
                    .ok_or(ProcessorError::InputSpecMismatch(0))?;
                let out = outputs[0]
                    .as_sample_mut()
                    .ok_or(ProcessorError::OutputSpecMismatch(0))?;

                for (sample, in1) in itertools::izip!(out, in1) {
                    *sample = (**in1).$method().into();
                }

                Ok(())
            }
        }

        impl GraphBuilder {
            #[doc = $doc]
            pub fn $method(&self) -> Node {
                self.add_processor($name)
            }
        }
    };
}

impl_unary_proc!(
    NegProc,
    neg,
    r#"
A processor that negates a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to negate. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The negated input signal. |
    "#
);
impl_unary_proc!(
    AbsProc,
    abs,
    r#"
A processor that calculates the absolute value of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the absolute value of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The absolute value of the input signal. |
    "#
);
impl_unary_proc!(
    SqrtProc,
    sqrt,
    r#"
A processor that calculates the square root of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the square root of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The square root of the input signal. |
    "#
);
impl_unary_proc!(
    CbrtProc,
    cbrt,
    r#"
A processor that calculates the cube root of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the cube root of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The cube root of the input signal. |
    "#
);
impl_unary_proc!(
    CeilProc,
    ceil,
    r#"
A processor that rounds a signal up to the nearest integer.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to round up. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The input signal rounded up to the nearest integer. |
    "#
);
impl_unary_proc!(
    FloorProc,
    floor,
    r#"
A processor that rounds a signal down to the nearest integer.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to round down. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The input signal rounded down to the nearest integer. |
    "#
);
impl_unary_proc!(
    RoundProc,
    round,
    r#"
A processor that rounds a signal to the nearest integer.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to round. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The input signal rounded to the nearest integer. |
    "#
);
impl_unary_proc!(
    TruncProc,
    trunc,
    r#"
A processor that truncates a signal to an integer.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to truncate. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The input signal truncated to an integer. |
    "#
);
impl_unary_proc!(
    FractProc,
    fract,
    r#"
A processor that returns the fractional part of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to get the fractional part of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The fractional part of the input signal. |
    "#
);
impl_unary_proc!(
    RecipProc,
    recip,
    r#"
A processor that calculates the reciprocal of a signal.

Note that the input signal defaults to `0.0`, so be sure to provide a non-zero value.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the reciprocal of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The reciprocal of the input signal. |
    "#
);
impl_unary_proc!(
    SignumProc,
    signum,
    r#"
A processor that returns the sign of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to get the sign of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The sign of the input signal. |
    "#
);
impl_unary_proc!(
    SinProc,
    sin,
    r#"
A processor that calculates the sine of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the sine of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The sine of the input signal. |
    "#
);
impl_unary_proc!(
    CosProc,
    cos,
    r#"
A processor that calculates the cosine of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the cosine of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The cosine of the input signal. |
    "#
);
impl_unary_proc!(
    TanProc,
    tan,
    r#"
A processor that calculates the tangent of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the tangent of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The tangent of the input signal. |
    "#
);
impl_unary_proc!(
    AsinProc,
    asin,
    r#"
A processor that calculates the arcsine of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the arcsine of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The arcsine of the input signal. |
    "#
);
impl_unary_proc!(
    AcosProc,
    acos,
    r#"
A processor that calculates the arccosine of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the arccosine of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The arccosine of the input signal. |
    "#
);
impl_unary_proc!(
    AtanProc,
    atan,
    r#"
A processor that calculates the arctangent of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the arctangent of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The arctangent of the input signal. |
    "#
);
impl_unary_proc!(
    SinhProc,
    sinh,
    r#"
A processor that calculates the hyperbolic sine of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `0.0` | The signal to calculate the hyperbolic sine of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The hyperbolic sine of the input signal. |
    "#
);
impl_unary_proc!(
    CoshProc,
    cosh,
    r#"
A processor that calculates the hyperbolic cosine of a signal.

# Inputs

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the hyperbolic cosine of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The hyperbolic cosine of the input signal. |
    "#
);
impl_unary_proc!(
    TanhProc,
    tanh,
    r#"
A processor that calculates the hyperbolic tangent of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the hyperbolic tangent of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The hyperbolic tangent of the input signal. |
    "#
);
impl_unary_proc!(
    ExpProc,
    exp,
    r#"
A processor that calculates the exponential of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the exponential of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The exponential of the input signal. |
    "#
);
impl_unary_proc!(
    Exp2Proc,
    exp2,
    r#"
A processor that calculates 2 raised to the power of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate 2 raised to the power of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | 2 raised to the power of the input signal. |
    "#
);
impl_unary_proc!(
    ExpM1Proc,
    exp_m1,
    r#"
A processor that calculates the exponential of a signal minus 1.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the exponential of minus 1. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The exponential of the input signal minus 1. |
    "#
);
impl_unary_proc!(
    LnProc,
    ln,
    r#"
A processor that calculates the natural logarithm of a signal.

Note that the input signal defaults to `0.0`, so be sure to provide a positive value.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the natural logarithm of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The natural logarithm of the input signal. |
    "#
);
impl_unary_proc!(
    Log2Proc,
    log2,
    r#"
A processor that calculates the base-2 logarithm of a signal.

Note that the input signal defaults to `0.0`, so be sure to provide a positive value.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the base-2 logarithm of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The base-2 logarithm of the input signal. |
    "#
);
impl_unary_proc!(
    Log10Proc,
    log10,
    r#"
A processor that calculates the base-10 logarithm of a signal.

Note that the input signal defaults to `0.0`, so be sure to provide a positive value.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Sample` | `0.0` | The signal to calculate the base-10 logarithm of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Sample` | The base-10 logarithm of the input signal. |
    "#
);
