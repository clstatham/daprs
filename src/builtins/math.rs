//! Mathematical processors.

use crate::{prelude::*, processor::ProcessorError};
use std::ops::{
    Add as AddOp, Div as DivOp, Mul as MulOp, Neg as NegOp, Rem as RemOp, Sub as SubOp,
};

/// A processor that outputs a constant value every sample.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The constant value. |
#[derive(Clone, Debug)]
pub struct Constant<S: Signal + Clone> {
    value: S,
}

impl<S: Signal + Clone> Constant<S> {
    /// Creates a new `Constant` processor.
    pub fn new(value: S) -> Self {
        Self { value }
    }
}

impl<S: Signal + Clone> Processor for Constant<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::signal_type())]
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for out in outputs.iter_output_as::<S>(0)? {
            *out = Some(self.value.clone());
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// Adds a node that outputs a constant value every sample.
    pub fn constant(&self, value: impl Signal + Clone) -> Node {
        self.add(Constant::new(value))
    }
}

/// A processor that converts MIDI note numbers to frequencies.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `note` | `Float` | The MIDI note number. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `freq` | `Float` | The frequency of the MIDI note. |
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
            outputs.iter_output_mut_as_floats(0)?
        ) {
            let note = note.unwrap_or_default();
            *freq = Some(Float::powf(2.0, (note - 69.0) / 12.0) * 440.0);
        }

        Ok(())
    }
}

/// A processor that converts frequencies to MIDI note numbers.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `freq` | `Float` | The frequency to convert. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `note` | `Float` | The MIDI note number. |
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
            outputs.iter_output_mut_as_floats(0)?
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

/// A processor that executes an arbitrary mathematical expression using the [`evalexpr`] crate.
///
/// This processor is currently limited to only [`Float`] inputs, and the expression must evaluate to a single [`Float`] output.
///
/// # Inputs
///
/// The inputs to this processor are the variables used in the expression.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The result of the expression. |
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
    /// Creates a new `Expr` processor with the given expression. The expression is pre-compiled into an [`evalexpr::Node`] and cannot be changed.
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
                .set_value(name.to_string(), evalexpr::Value::from_float(*value as f64))
                .unwrap();
        }
        self.expr
            .eval_float_with_context_mut(&mut self.context)
            .unwrap() as Float
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
        for (samp_idx, out) in outputs.iter_output_as::<Float>(0)?.enumerate() {
            self.input_values.clear();

            for (inp_idx, name) in self.inputs.iter().enumerate() {
                let buffer = &inputs.input(inp_idx).unwrap();
                let actual = buffer.signal_type();
                let buffer =
                    buffer
                        .as_type::<Float>()
                        .ok_or_else(|| ProcessorError::InputSpecMismatch {
                            index: inp_idx,
                            expected: SignalType::Float,
                            actual,
                        })?;
                let samp = buffer[samp_idx].unwrap();
                self.input_values.push((name.to_string(), samp));
            }

            *out = Some(self.eval());
        }

        Ok(())
    }
}

macro_rules! impl_binary_proc {
    ($name:ident, $method:ident, ($($data:ty),*), $doc:literal) => {
        #[derive(Clone, Debug)]
        #[doc = $doc]
        pub struct $name<S: Signal>(std::marker::PhantomData<S>);

        impl<S: Signal> $name<S> {
            #[doc = concat!("Creates a new `", stringify!($name), "` processor.")]
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
                    SignalSpec::new("a", <$data as Signal>::signal_type()),
                    SignalSpec::new("b", <$data as Signal>::signal_type()),
                ]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("out", <$data as Signal>::signal_type())]
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
                    match (in1, in2) {
                        (Some(in1), Some(in2)) => {
                            *sample = Some(<$data>::$method(*in1, *in2));
                        }
                        (Some(a), None) => {
                            *sample = Some(<$data>::$method(*a, <$data>::default()));
                        }
                        (None, Some(b)) => {
                            *sample = Some(<$data>::$method(<$data>::default(), *b));
                        }
                        _ => {
                            *sample = None;
                        }
                    }
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
    "A processor that adds two signals together."
);
impl_binary_proc!(
    Sub,
    sub,
    (Float, i64),
    "A processor that subtracts one signal from another."
);
impl_binary_proc!(
    Mul,
    mul,
    (Float, i64),
    "A processor that multiplies two signals together."
);
impl_binary_proc!(
    Div,
    div,
    (Float, i64),
    "A processor that divides one signal by another."
);
impl_binary_proc!(
    Rem,
    rem,
    (Float, i64),
    "A processor that calculates the remainder of dividing one signal by another."
);
impl_binary_proc!(
    Powf,
    powf,
    (Float),
    "A processor that raises one signal to the power of another."
);
impl_binary_proc!(
    Atan2,
    atan2,
    (Float),
    "A processor that calculates the arctangent of the ratio of two signals."
);
impl_binary_proc!(
    Hypot,
    hypot,
    (Float),
    "A processor that calculates the hypotenuse of two signals."
);
impl_binary_proc!(
    Max,
    max,
    (Float, i64),
    "A processor that calculates the maximum of two signals."
);
impl_binary_proc!(
    Min,
    min,
    (Float, i64),
    "A processor that calculates the minimum of two signals."
);

macro_rules! impl_unary_proc {
    ($name:ident, $method:ident, ($($data:ty),*), $doc:literal) => {
        #[derive(Clone, Debug)]
        #[doc = $doc]
        pub struct $name<S: Signal>(std::marker::PhantomData<S>);

        impl<S: Signal> $name<S> {
            #[doc = concat!("Creates a new `", stringify!($name), "` processor.")]
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
                vec![SignalSpec::new("in", <$data as Signal>::signal_type())]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("out", <$data as Signal>::signal_type())]
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

impl_unary_proc!(Neg, neg, (Float, i64), "A processor that negates a signal.");
impl_unary_proc!(
    Abs,
    abs,
    (Float, i64),
    "A processor that calculates the absolute value of a signal."
);
impl_unary_proc!(
    Sqrt,
    sqrt,
    (Float),
    "A processor that calculates the square root of a signal."
);
impl_unary_proc!(
    Cbrt,
    cbrt,
    (Float),
    "A processor that calculates the cube root of a signal."
);
impl_unary_proc!(
    Ceil,
    ceil,
    (Float),
    "A processor that rounds a signal up to the nearest integer."
);
impl_unary_proc!(
    Floor,
    floor,
    (Float),
    "A processor that rounds a signal down to the nearest integer."
);
impl_unary_proc!(
    Round,
    round,
    (Float),
    "A processor that rounds a signal to the nearest integer."
);
impl_unary_proc!(
    Trunc,
    trunc,
    (Float),
    "A processor that truncates a signal to an integer."
);
impl_unary_proc!(
    Fract,
    fract,
    (Float),
    "A processor that outputs the fractional part of a signal."
);
impl_unary_proc!(
    Recip,
    recip,
    (Float),
    "A processor that calculates the reciprocal of a signal."
);
impl_unary_proc!(
    Signum,
    signum,
    (Float, i64),
    "A processor that outputs the sign of a signal."
);
impl_unary_proc!(
    Sin,
    sin,
    (Float),
    "A processor that calculates the sine of a signal."
);
impl_unary_proc!(
    Cos,
    cos,
    (Float),
    "A processor that calculates the cosine of a signal."
);
impl_unary_proc!(
    Tan,
    tan,
    (Float),
    "A processor that calculates the tangent of a signal."
);
impl_unary_proc!(
    Tanh,
    tanh,
    (Float),
    "A processor that calculates the hyperbolic tangent of a signal."
);
impl_unary_proc!(
    Exp,
    exp,
    (Float),
    "A processor that calculates the natural exponential of a signal."
);
impl_unary_proc!(
    Ln,
    ln,
    (Float),
    "A processor that calculates the natural logarithm of a signal."
);
impl_unary_proc!(
    Log2,
    log2,
    (Float),
    "A processor that calculates the base-2 logarithm of a signal."
);
impl_unary_proc!(
    Log10,
    log10,
    (Float),
    "A processor that calculates the base-10 logarithm of a signal."
);
