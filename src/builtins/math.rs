//! Mathematical processors.

use crate::{prelude::*, processor::ProcessorError};
use std::ops::{
    Add as AddOp, Div as DivOp, Mul as MulOp, Neg as NegOp, Rem as RemOp, Sub as SubOp,
};

/// A processor that outputs a constant value.
///
/// # Outputs
///
/// | Index | Name | Default | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `0.0` | The constant value. |
#[derive(Clone, Debug)]

pub struct Constant {
    value: Sample,
}

impl Constant {
    /// Creates a new constant processor with the given value.
    pub fn new(value: Sample) -> Self {
        Self { value }
    }
}

impl Default for Constant {
    fn default() -> Self {
        Self { value: 0.0 }
    }
}

impl Processor for Constant {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", self.value)]
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let out = outputs.output_as_samples(0)?;

        out.fill(self.value);

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that outputs a constant value.
    ///
    /// See also: [`Constant`].
    pub fn constant(&self, value: Sample) -> Node {
        self.add(Constant::new(value))
    }
}

/// A processor that converts a MIDI note number to a frequency in Hz.
///
/// # Inputs
///
/// | Index | Name | Default | Description |
/// | --- | --- | --- | --- |
/// | `0` | `note` | `69.0` | The MIDI note number to convert to a frequency. |
///
/// # Outputs
///
/// | Index | Name | Default | Description |
/// | --- | --- | --- | --- |
/// | `0` | `freq` | `440.0` | The frequency in Hz. |
#[derive(Clone, Debug, Default)]
pub struct MidiToFreq;

impl Processor for MidiToFreq {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("note", 69.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("freq", 440.0)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (note, freq) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            *freq = Sample::powf(2.0, (note - 69.0) / 12.0) * 440.0;
        }

        Ok(())
    }
}

/// A processor that converts a frequency in Hz to a MIDI note number.
///
/// # Inputs
///
/// | Index | Name | Default | Description |
/// | --- | --- | --- | --- |
/// | `0` | `freq` | `440.0` | The frequency in Hz to convert to a MIDI note number. |
///
/// # Outputs
///
/// | Index | Name | Default | Description |
/// | --- | --- | --- | --- |
/// | `0` | `note` | `69.0` | The MIDI note number. |
#[derive(Clone, Debug, Default)]
pub struct FreqToMidi;

impl Processor for FreqToMidi {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("freq", 440.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("note", 69.0)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (freq, note) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            *note = 69.0 + 12.0 * (freq / 440.0).log2();
        }

        Ok(())
    }
}

/// A processor that evaluates an expression.
///
/// The expression uses a simple syntax based on the [`evalexpr`] crate.
///
/// # Inputs
///
/// The inputs are the variables that are referenced in the expression.
///
/// The names of the inputs are extracted from the expression itself.
///
/// The inputs are expected to be of type `Sample`, that is, a floating-point number. They default to `0.0`.
///
/// # Outputs
///
/// | Index | Name | Default | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `0.0` | The result of the expression. |
#[derive(Clone, Debug)]
pub struct Expr {
    context: evalexpr::HashMapContext<evalexpr::DefaultNumericTypes>,
    expr: evalexpr::Node<evalexpr::DefaultNumericTypes>,
    inputs: Vec<String>,
    input_values: Vec<(String, Sample)>,
}

impl Expr {
    /// Creates a new `Eval` processor with the given expression.
    pub fn new(expr: impl AsRef<str>) -> Self {
        let expr: evalexpr::Node<evalexpr::DefaultNumericTypes> =
            evalexpr::build_operator_tree(expr.as_ref()).unwrap();
        let inputs: Vec<String> = expr
            .iter_read_variable_identifiers()
            .map(|s| s.to_string())
            .collect();
        Self {
            context: evalexpr::HashMapContext::new(),
            expr,
            input_values: Vec::with_capacity(inputs.len()),
            inputs,
        }
    }

    fn eval(&mut self) -> Sample {
        use evalexpr::ContextWithMutableVariables;
        self.context.clear_variables();
        for (name, value) in self.input_values.iter() {
            self.context
                .set_value(name.to_string(), evalexpr::Value::from_float(*value))
                .unwrap();
        }
        let result = self.expr.eval_with_context(&self.context).unwrap();
        result.as_float().unwrap()
    }
}

impl Processor for Expr {
    fn input_spec(&self) -> Vec<SignalSpec> {
        self.inputs
            .iter()
            .map(|name| SignalSpec::unbounded(name, 0.0))
            .collect()
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let out = outputs.output_as_samples(0)?;

        for (samp_idx, out) in out.iter_mut().enumerate() {
            self.input_values.clear();

            for (inp_idx, name) in self.inputs.iter().enumerate() {
                let buffer = inputs
                    .input(inp_idx)
                    .ok_or(ProcessorError::InputSpecMismatch(inp_idx))?;
                let buffer = buffer
                    .as_sample()
                    .ok_or(ProcessorError::InputSpecMismatch(inp_idx))?;

                self.input_values.push((name.to_string(), buffer[samp_idx]));
            }

            *out = self.eval();
        }

        Ok(())
    }
}

macro_rules! impl_binary_proc {
    ($name:ident, $method:ident, $shortdoc:literal, $doc:literal) => {
        #[derive(Clone, Debug, Default)]
        #[doc = $doc]
        pub struct $name;

        impl Processor for $name {
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
                inputs: ProcessorInputs,
                mut outputs: ProcessorOutputs,
            ) -> Result<(), ProcessorError> {
                for (sample, in1, in2) in itertools::izip!(
                    outputs.iter_output_mut_as_samples(0)?,
                    inputs.iter_input_as_samples(0)?,
                    inputs.iter_input_as_samples(1)?
                ) {
                    debug_assert!(in1.is_finite());
                    debug_assert!(in2.is_finite());
                    *sample = Sample::$method(in1, in2);
                }

                Ok(())
            }
        }
    };
}

impl_binary_proc!(
    Add,
    add,
    r#"
A processor that adds two signals together.

See also: [`Add`](crate::builtins::math::Add).
    "#,
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
    Sub,
    sub,
    r#"
A processor that subtracts one signal from another.

See also: [`Sub`](crate::builtins::math::Sub).
    "#,
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
    Mul,
    mul,
    r#"
A processor that multiplies two signals together.

See also: [`Mul`](crate::builtins::math::Mul).
    "#,
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
    Div,
    div,
    r#"
A processor that divides one signal by another.

See also: [`Div`](crate::builtins::math::Div).
    "#,
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
    Rem,
    rem,
    r#"
A processor that calculates the remainder of one signal divided by another.

See also: [`Rem`](crate::builtins::math::Rem).
    "#,
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
    Powf,
    powf,
    r#"
A processor that raises one signal to the power of a constant value.

See also: [`Powf`](crate::builtins::math::Powf).
    "#,
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
    Atan2,
    atan2,
    r#"
A processor that calculates the arctangent of the ratio of two signals.

See also: [`Atan2`](crate::builtins::math::Atan2).
    "#,
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
    Hypot,
    hypot,
    r#"
A processor that calculates the hypotenuse of two signals.

See also: [`Hypot`](crate::builtins::math::Hypot).
    "#,
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
    Max,
    max,
    r#"
A processor that calculates the maximum of two signals.

See also: [`Max`](crate::builtins::math::Max).
    "#,
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
    Min,
    min,
    r#"
A processor that calculates the minimum of two signals.

See also: [`Min`](crate::builtins::math::Min).
    "#,
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
    ($name:ident, $method:ident, $shortdoc:literal, $doc:literal) => {
        #[derive(Clone, Debug, Default)]
        #[doc = $doc]
        pub struct $name;

        impl Processor for $name {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::unbounded("in", 0.0)]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::unbounded("out", 0.0)]
            }

            fn process(
                &mut self,
                inputs: ProcessorInputs,
                mut outputs: ProcessorOutputs,
            ) -> Result<(), ProcessorError> {
                for (sample, in1) in itertools::izip!(
                    outputs.iter_output_mut_as_samples(0)?,
                    inputs.iter_input_as_samples(0)?
                ) {
                    debug_assert!(in1.is_finite());
                    *sample = in1.$method().into();
                }

                Ok(())
            }
        }
    };
}

impl_unary_proc!(
    Neg,
    neg,
    r#"
A processor that negates a signal.

See also: [`Neg`](crate::builtins::math::Neg).
    "#,
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
    Abs,
    abs,
    r#"
A processor that calculates the absolute value of a signal.

See also: [`Abs`](crate::builtins::math::Abs).
    "#,
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
    Sqrt,
    sqrt,
    r#"
A processor that calculates the square root of a signal.

See also: [`Sqrt`](crate::builtins::math::Sqrt).
    "#,
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
    Cbrt,
    cbrt,
    r#"
A processor that calculates the cube root of a signal.

See also: [`Cbrt`](crate::builtins::math::Cbrt).
    "#,
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
    Ceil,
    ceil,
    r#"
A processor that rounds a signal up to the nearest integer.

See also: [`Ceil`](crate::builtins::math::Ceil).
    "#,
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
    Floor,
    floor,
    r#"
A processor that rounds a signal down to the nearest integer.

See also: [`Floor`](crate::builtins::math::Floor).
    "#,
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
    Round,
    round,
    r#"
A processor that rounds a signal to the nearest integer.

See also: [`Round`](crate::builtins::math::Round).
    "#,
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
    Trunc,
    trunc,
    r#"
A processor that truncates a signal to an integer.

See also: [`Trunc`](crate::builtins::math::Trunc).
    "#,
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
    Fract,
    fract,
    r#"
A processor that returns the fractional part of a signal.

See also: [`Fract`](crate::builtins::math::Fract).
    "#,
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
    Recip,
    recip,
    r#"
A processor that calculates the reciprocal of a signal.

See also: [`Recip`](crate::builtins::math::Recip).
    "#,
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
    Signum,
    signum,
    r#"
A processor that returns the sign of a signal.

See also: [`Signum`](crate::builtins::math::Signum).
    "#,
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
    Sin,
    sin,
    r#"
A processor that calculates the sine of a signal.

See also: [`Sin`](crate::builtins::math::Sin).
    "#,
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
    Cos,
    cos,
    r#"
A processor that calculates the cosine of a signal.

See also: [`Cos`](crate::builtins::math::Cos).
    "#,
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
    Tan,
    tan,
    r#"
A processor that calculates the tangent of a signal.

See also: [`Tan`](crate::builtins::math::Tan).
    "#,
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
    Tanh,
    tanh,
    r#"
A processor that calculates the hyperbolic tangent of a signal.

See also: [`Tanh`](crate::builtins::math::Tanh).
    "#,
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
    Exp,
    exp,
    r#"
A processor that calculates the exponential of a signal.

See also: [`Exp`](crate::builtins::math::Exp).
    "#,
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
    Ln,
    ln,
    r#"
A processor that calculates the natural logarithm of a signal.

See also: [`Ln`](crate::builtins::math::Ln).
    "#,
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
    Log2,
    log2,
    r#"
A processor that calculates the base-2 logarithm of a signal.

See also: [`Log2`](crate::builtins::math::Log2).
    "#,
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
    Log10,
    log10,
    r#"
A processor that calculates the base-10 logarithm of a signal.

See also: [`Log10`](crate::builtins::math::Log10).
    "#,
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
