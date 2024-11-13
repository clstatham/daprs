//! Mathematical processors.

use crate::{prelude::*, processor::ProcessorError};
use std::ops::{
    Add as AddOp, Div as DivOp, Mul as MulOp, Neg as NegOp, Rem as RemOp, Sub as SubOp,
};

#[derive(Clone, Debug)]

pub struct Constant {
    value: AnySignal,
}

impl Constant {
    pub fn new(value: impl Into<AnySignal>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl Processor for Constant {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.value.type_())]
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        match (outputs.output(0), &self.value) {
            (SignalBuffer::Float(out), AnySignal::Float(value)) => {
                out.fill(Some(*value));
            }
            (SignalBuffer::Int(out), AnySignal::Int(value)) => {
                out.fill(Some(*value));
            }
            (SignalBuffer::Bool(out), AnySignal::Bool(value)) => {
                out.fill(Some(*value));
            }
            (SignalBuffer::List(out), AnySignal::List(value)) => {
                out.fill(Some(value.clone()));
            }
            (SignalBuffer::String(out), AnySignal::String(value)) => {
                out.fill(Some(value.clone()));
            }
            (SignalBuffer::Midi(out), AnySignal::Midi(value)) => {
                out.fill(Some(*value));
            }
            (out, _) => {
                return Err(ProcessorError::OutputSpecMismatch {
                    index: 0,
                    expected: self.value.type_(),
                    actual: out.type_(),
                })
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    pub fn constant(&self, value: impl Into<AnySignal>) -> Node {
        self.add(Constant::new(value))
    }
}

#[derive(Clone, Debug, Default)]
pub struct MidiToFreq;

impl Processor for MidiToFreq {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("note", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("freq", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (note, freq) in itertools::izip!(
            inputs.iter_input_as_floats(0)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            let Some(note) = note else {
                *freq = None;
                continue;
            };
            *freq = Some(Float::powf(2.0, (note - 69.0) / 12.0) * 440.0);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct FreqToMidi;

impl Processor for FreqToMidi {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("freq", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("note", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (freq, note) in itertools::izip!(
            inputs.iter_input_as_floats(0)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            let Some(freq) = freq else {
                *note = None;
                continue;
            };
            *note = Some(69.0 + 12.0 * (freq / 440.0).log2());
        }

        Ok(())
    }
}

#[cfg(feature = "expr")]
#[derive(Clone, Debug)]
pub struct Expr {
    context: evalexpr::HashMapContext<evalexpr::DefaultNumericTypes>,
    expr: evalexpr::Node<evalexpr::DefaultNumericTypes>,
    inputs: Vec<String>,
    input_values: Vec<(String, Float)>,
}

#[cfg(feature = "expr")]
impl Expr {
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

    fn eval(&mut self) -> Float {
        use evalexpr::ContextWithMutableVariables;
        self.context.clear_variables();
        for (name, value) in self.input_values.iter() {
            self.context
                .set_value(name.to_string(), evalexpr::Value::from_float(*value))
                .unwrap();
        }
        self.expr
            .eval_float_with_context_mut(&mut self.context)
            .unwrap()
    }
}

#[cfg(feature = "expr")]
impl Processor for Expr {
    fn input_spec(&self) -> Vec<SignalSpec> {
        self.inputs
            .iter()
            .map(|name| SignalSpec::new(name, SignalType::Float))
            .collect()
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let out = outputs.output_as_floats(0)?;

        for (samp_idx, out) in out.iter_mut().enumerate() {
            self.input_values.clear();

            for (inp_idx, name) in self.inputs.iter().enumerate() {
                let buffer = &inputs.inputs[inp_idx].unwrap();
                let buffer = buffer
                    .as_sample()
                    .ok_or(ProcessorError::InputSpecMismatch {
                        index: inp_idx,
                        expected: SignalType::Float,
                        actual: buffer.type_(),
                    })?;

                self.input_values
                    .push((name.to_string(), buffer[samp_idx].unwrap()));
            }

            *out = Some(self.eval());
        }

        Ok(())
    }
}

macro_rules! impl_binary_proc {
    ($name:ident, $method:ident, ($($data:ty),*), $shortdoc:literal, $doc:literal) => {
        #[derive(Clone, Debug)]
        #[doc = $doc]
        pub struct $name<S: Signal>(std::marker::PhantomData<S>);

        impl<S: Signal> $name<S> {
            pub fn new() -> Self {
                Self(std::marker::PhantomData)
            }
        }

        impl<S: Signal> Default for $name<S> {
            fn default() -> Self {
                Self::new()
            }
        }

        $(
        impl Processor for $name<$data> {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![
                    SignalSpec::new("a", <$data as Signal>::TYPE),
                    SignalSpec::new("b", <$data as Signal>::TYPE),
                ]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("out", <$data as Signal>::TYPE)]
            }

            fn process(
                &mut self,
                inputs: ProcessorInputs,
                mut outputs: ProcessorOutputs,
            ) -> Result<(), ProcessorError> {
                for (sample, in1, in2) in itertools::izip!(
                    outputs.iter_output_as::<$data>(0)?,
                    inputs.iter_input_as::<$data>(0)?,
                    inputs.iter_input_as::<$data>(1)?
                ) {
                    let (Some(in1), Some(in2)) = (in1, in2) else {
                        *sample = None;
                        continue;
                    };

                    // debug_assert!(in1.is_finite());
                    // debug_assert!(in2.is_finite());
                    *sample = Some(<$data>::$method(*in1, *in2));
                }

                Ok(())
            }
        }
        )*
    };
}

impl_binary_proc!(
    Add,
    add,
    (Float, i64),
    r#"
A processor that adds two signals together.

See also: [`Add`](crate::builtins::math::Add).
    "#,
    r#"
A processor that adds two signals together.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Float` | `0.0` | The first signal to add. |
| `1` | `b` | `Float` | `0.0` | The second signal to add. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The sum of the two input signals. |
    "#
);
impl_binary_proc!(
    Sub,
    sub,
    (Float, i64),
    r#"
A processor that subtracts one signal from another.

See also: [`Sub`](crate::builtins::math::Sub).
    "#,
    r#"
A processor that subtracts one signal from another.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Float` | `0.0` | The signal to subtract from. |
| `1` | `b` | `Float` | `0.0` | The signal to subtract. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The difference of the two input signals. |
    "#
);
impl_binary_proc!(
    Mul,
    mul,
    (Float, i64),
    r#"
A processor that multiplies two signals together.

See also: [`Mul`](crate::builtins::math::Mul).
    "#,
    r#"
A processor that multiplies two signals together.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Float` | `0.0` | The first signal to multiply. |
| `1` | `b` | `Float` | `0.0` | The second signal to multiply. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The product of the two input signals. |
    "#
);
impl_binary_proc!(
    Div,
    div,
    (Float, i64),
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
| `0` | `a` | `Float` | `0.0` | The signal to divide. |
| `1` | `b` | `Float` | `0.0` | The signal to divide by. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The quotient of the two input signals. |
    "#
);
impl_binary_proc!(
    Rem,
    rem,
    (Float, i64),
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
| `0` | `a` | `Float` | `0.0` | The signal to divide. |
| `1` | `b` | `Float` | `0.0` | The signal to divide by. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The remainder of the two input signals. |
    "#
);
impl_binary_proc!(
    Powf,
    powf,
    (Float),
    r#"
A processor that raises one signal to the power of a constant value.

See also: [`Powf`](crate::builtins::math::Powf).
    "#,
    r#"
A processor that raises one signal to the power of another.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Float` | `0.0` | The base signal. |
| `1` | `b` | `Float` | `0.0` | The exponent signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The base signal raised to the power of the exponent signal. |
    "#
);
impl_binary_proc!(
    Atan2,
    atan2,
    (Float),
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
| `0` | `a` | `Float` | `0.0` | The first signal. |
| `1` | `b` | `Float` | `0.0` | The second signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The arctangent of the ratio of the two input signals. |
    "#
);
impl_binary_proc!(
    Hypot,
    hypot,
    (Float),
    r#"
A processor that calculates the hypotenuse of two signals.

See also: [`Hypot`](crate::builtins::math::Hypot).
    "#,
    r#"
A processor that calculates the hypotenuse of two signals.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Float` | `0.0` | The first signal. |
| `1` | `b` | `Float` | `0.0` | The second signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The hypotenuse of the two input signals. |
    "#
);
impl_binary_proc!(
    Max,
    max,
    (Float, i64),
    r#"
A processor that calculates the maximum of two signals.

See also: [`Max`](crate::builtins::math::Max).
    "#,
    r#"
A processor that calculates the maximum of two signals.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Float` | `0.0` | The first signal. |
| `1` | `b` | `Float` | `0.0` | The second signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The maximum of the two input signals. |
    "#
);
impl_binary_proc!(
    Min,
    min,
    (Float, i64),
    r#"
A processor that calculates the minimum of two signals.

See also: [`Min`](crate::builtins::math::Min).
    "#,
    r#"
A processor that calculates the minimum of two signals.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Float` | `0.0` | The first signal. |
| `1` | `b` | `Float` | `0.0` | The second signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The minimum of the two input signals. |
    "#
);

macro_rules! impl_unary_proc {
    ($name:ident, $method:ident, ($($data:ty),*), $shortdoc:literal, $doc:literal) => {
        #[derive(Clone, Debug)]
        #[doc = $doc]
        pub struct $name<S: Signal>(std::marker::PhantomData<S>);

        impl<S: Signal> $name<S> {
            pub fn new() -> Self {
                Self(std::marker::PhantomData)
            }
        }

        impl<S: Signal> Default for $name<S> {
            fn default() -> Self {
                Self::new()
            }
        }

        $(
        impl Processor for $name<$data> {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("in", <$data as Signal>::TYPE)]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("out", <$data as Signal>::TYPE)]
            }

            fn process(
                &mut self,
                inputs: ProcessorInputs,
                mut outputs: ProcessorOutputs,
            ) -> Result<(), ProcessorError> {
                for (sample, in1) in itertools::izip!(
                    outputs.iter_output_as::<$data>(0)?,
                    inputs.iter_input_as::<$data>(0)?
                ) {
                    let Some(in1) = in1 else {
                        *sample = None;
                        continue;
                    };
                    *sample = Some(in1.$method());
                }

                Ok(())
            }
        }
        )*
    };
}

impl_unary_proc!(
    Neg,
    neg,
    (Float, i64),
    r#"
A processor that negates a signal.

See also: [`Neg`](crate::builtins::math::Neg).
    "#,
    r#"
A processor that negates a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to negate. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The negated input signal. |
    "#
);
impl_unary_proc!(
    Abs,
    abs,
    (Float, i64),
    r#"
A processor that calculates the absolute value of a signal.

See also: [`Abs`](crate::builtins::math::Abs).
    "#,
    r#"
A processor that calculates the absolute value of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to calculate the absolute value of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The absolute value of the input signal. |
    "#
);
impl_unary_proc!(
    Sqrt,
    sqrt,
    (Float),
    r#"
A processor that calculates the square root of a signal.

See also: [`Sqrt`](crate::builtins::math::Sqrt).
    "#,
    r#"
A processor that calculates the square root of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to calculate the square root of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The square root of the input signal. |
    "#
);
impl_unary_proc!(
    Cbrt,
    cbrt,
    (Float),
    r#"
A processor that calculates the cube root of a signal.

See also: [`Cbrt`](crate::builtins::math::Cbrt).
    "#,
    r#"
A processor that calculates the cube root of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to calculate the cube root of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The cube root of the input signal. |
    "#
);
impl_unary_proc!(
    Ceil,
    ceil,
    (Float),
    r#"
A processor that rounds a signal up to the nearest integer.

See also: [`Ceil`](crate::builtins::math::Ceil).
    "#,
    r#"
A processor that rounds a signal up to the nearest integer.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to round up. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The input signal rounded up to the nearest integer. |
    "#
);
impl_unary_proc!(
    Floor,
    floor,
    (Float),
    r#"
A processor that rounds a signal down to the nearest integer.

See also: [`Floor`](crate::builtins::math::Floor).
    "#,
    r#"
A processor that rounds a signal down to the nearest integer.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to round down. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The input signal rounded down to the nearest integer. |
    "#
);
impl_unary_proc!(
    Round,
    round,
    (Float),
    r#"
A processor that rounds a signal to the nearest integer.

See also: [`Round`](crate::builtins::math::Round).
    "#,
    r#"
A processor that rounds a signal to the nearest integer.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to round. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The input signal rounded to the nearest integer. |
    "#
);
impl_unary_proc!(
    Trunc,
    trunc,
    (Float),
    r#"
A processor that truncates a signal to an integer.

See also: [`Trunc`](crate::builtins::math::Trunc).
    "#,
    r#"
A processor that truncates a signal to an integer.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to truncate. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The input signal truncated to an integer. |
    "#
);
impl_unary_proc!(
    Fract,
    fract,
    (Float),
    r#"
A processor that returns the fractional part of a signal.

See also: [`Fract`](crate::builtins::math::Fract).
    "#,
    r#"
A processor that returns the fractional part of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to get the fractional part of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The fractional part of the input signal. |
    "#
);
impl_unary_proc!(
    Recip,
    recip,
    (Float),
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
| `0` | `in` | `Float` | `0.0` | The signal to calculate the reciprocal of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The reciprocal of the input signal. |
    "#
);
impl_unary_proc!(
    Signum,
    signum,
    (Float, i64),
    r#"
A processor that returns the sign of a signal.

See also: [`Signum`](crate::builtins::math::Signum).
    "#,
    r#"
A processor that returns the sign of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to get the sign of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The sign of the input signal. |
    "#
);
impl_unary_proc!(
    Sin,
    sin,
    (Float),
    r#"
A processor that calculates the sine of a signal.

See also: [`Sin`](crate::builtins::math::Sin).
    "#,
    r#"
A processor that calculates the sine of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to calculate the sine of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The sine of the input signal. |
    "#
);
impl_unary_proc!(
    Cos,
    cos,
    (Float),
    r#"
A processor that calculates the cosine of a signal.

See also: [`Cos`](crate::builtins::math::Cos).
    "#,
    r#"
A processor that calculates the cosine of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to calculate the cosine of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The cosine of the input signal. |
    "#
);
impl_unary_proc!(
    Tan,
    tan,
    (Float),
    r#"
A processor that calculates the tangent of a signal.

See also: [`Tan`](crate::builtins::math::Tan).
    "#,
    r#"
A processor that calculates the tangent of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to calculate the tangent of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The tangent of the input signal. |
    "#
);
impl_unary_proc!(
    Tanh,
    tanh,
    (Float),
    r#"
A processor that calculates the hyperbolic tangent of a signal.

See also: [`Tanh`](crate::builtins::math::Tanh).
    "#,
    r#"
A processor that calculates the hyperbolic tangent of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to calculate the hyperbolic tangent of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The hyperbolic tangent of the input signal. |
    "#
);
impl_unary_proc!(
    Exp,
    exp,
    (Float),
    r#"
A processor that calculates the exponential of a signal.

See also: [`Exp`](crate::builtins::math::Exp).
    "#,
    r#"
A processor that calculates the exponential of a signal.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `in` | `Float` | `0.0` | The signal to calculate the exponential of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The exponential of the input signal. |
    "#
);
impl_unary_proc!(
    Ln,
    ln,
    (Float),
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
| `0` | `in` | `Float` | `0.0` | The signal to calculate the natural logarithm of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The natural logarithm of the input signal. |
    "#
);
impl_unary_proc!(
    Log2,
    log2,
    (Float),
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
| `0` | `in` | `Float` | `0.0` | The signal to calculate the base-2 logarithm of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The base-2 logarithm of the input signal. |
    "#
);
impl_unary_proc!(
    Log10,
    log10,
    (Float),
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
| `0` | `in` | `Float` | `0.0` | The signal to calculate the base-10 logarithm of. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Float` | The base-10 logarithm of the input signal. |
    "#
);
