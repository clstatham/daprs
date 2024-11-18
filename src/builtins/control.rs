//! Control flow processors.

use crate::prelude::*;

/// A processor that outputs the value of the second input if the first input is `true`, otherwise the value of the third input.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `cond` | `Bool` | The condition. |
/// | `1` | `then` | `Any` | The value to output if the condition is `true`. |
/// | `2` | `else` | `Any` | The value to output if the condition is `false`. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The output value. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cond {
    cond: bool,
    then: AnySignal,
    else_: AnySignal,
}

impl Cond {
    /// Creates a new `Cond` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self {
            cond: false,
            then: AnySignal::default_of_type(&signal_type),
            else_: AnySignal::default_of_type(&signal_type),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Cond {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("cond", SignalType::Bool),
            SignalSpec::new("then", self.then.signal_type()),
            SignalSpec::new("else", self.else_.signal_type()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.then.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (mut out, cond, then, else_) in itertools::izip!(
            outputs.iter_output_mut(0),
            inputs.iter_input_as_bools(0)?,
            inputs.iter_input(1),
            inputs.iter_input(2)
        ) {
            if let Some(cond) = cond {
                self.cond = cond;
            }

            if let Some(then) = then {
                if then.signal_type() != self.then.signal_type() {
                    return Err(ProcessorError::InputSpecMismatch {
                        index: 1,
                        expected: self.then.signal_type(),
                        actual: then.signal_type(),
                    });
                }
                self.then.clone_from_ref(then);
            }

            if let Some(else_) = else_ {
                if else_.signal_type() != self.else_.signal_type() {
                    return Err(ProcessorError::InputSpecMismatch {
                        index: 2,
                        expected: self.else_.signal_type(),
                        actual: else_.signal_type(),
                    });
                }
                self.else_.clone_from_ref(else_);
            }

            if self.cond {
                out.clone_from_ref(self.then.as_ref());
            } else {
                out.clone_from_ref(self.else_.as_ref());
            }
        }

        Ok(())
    }
}

macro_rules! comparison_op {
    ($doc:literal, $name:ident, $op:tt) => {
        #[derive(Debug, Clone)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[doc = $doc]
        pub struct $name {
            a: AnySignal,
            b: AnySignal,
        }

        impl $name {
            #[doc = concat!("Creates a new `", stringify!($name), "` processor for the given type.")]
            pub fn new(signal_type: SignalType) -> Self {
                assert!(!matches!(signal_type, SignalType::List { .. }), "List comparison not supported");
                Self {
                    a: AnySignal::default_of_type(&signal_type),
                    b: AnySignal::default_of_type(&signal_type),
                }
            }
        }

        #[cfg_attr(feature = "serde", typetag::serde)]
        impl Processor for $name {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![
                    SignalSpec::new("a", self.a.signal_type()),
                    SignalSpec::new("b", self.b.signal_type()),
                ]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("out", SignalType::Bool)]
            }

            fn process(
                &mut self,
                inputs: ProcessorInputs,
                mut outputs: ProcessorOutputs,
            ) -> Result<(), ProcessorError> {
                for (out, a, b) in itertools::izip!(
                    outputs.iter_output_mut_as_bools(0)?,
                    inputs.iter_input(0),
                    inputs.iter_input(1)
                ) {
                    if let Some(a) = a {
                        if a.signal_type() != self.a.signal_type() {
                            return Err(ProcessorError::InputSpecMismatch {
                                index: 0,
                                expected: self.a.signal_type(),
                                actual: a.signal_type(),
                            });
                        }
                        self.a.clone_from_ref(a);
                    }

                    if let Some(b) = b {
                        if b.signal_type() != self.b.signal_type() {
                            return Err(ProcessorError::InputSpecMismatch {
                                index: 1,
                                expected: self.b.signal_type(),
                                actual: b.signal_type(),
                            });
                        }
                        self.b.clone_from_ref(b);
                    }

                    if self.a.is_none() || self.b.is_none() {
                        *out = None;
                        continue;
                    }

                    if self.a.signal_type() != self.b.signal_type() {
                        return Err(ProcessorError::InputSpecMismatch {
                            index: 0,
                            expected: self.a.signal_type(),
                            actual: self.b.signal_type(),
                        });
                    }

                    match (&self.a, &self.b) {
                        (AnySignal::Bool(Some(a)), AnySignal::Bool(Some(b))) => {
                            *out = Some(*a $op *b);
                        }
                        (AnySignal::Int(Some(a)), AnySignal::Int(Some(b))) => {
                            *out = Some(*a $op *b);
                        }
                        (AnySignal::Float(Some(a)), AnySignal::Float(Some(b))) => {
                            *out = Some(*a $op *b);
                        }
                        (AnySignal::String(Some(a)), AnySignal::String(Some(b))) => {
                            *out = Some(*a $op *b);
                        }
                        (AnySignal::Midi(Some(a)), AnySignal::Midi(Some(b))) => {
                            *out = Some(*a $op *b);
                        }
                        (AnySignal::List(Some(_)), AnySignal::List(Some(_))) => {
                            unimplemented!("List comparison not supported");
                        }
                        _ => unreachable!(),
                    }
                }

                Ok(())
            }
        }
    };
}

comparison_op!(
    r#"
A processor that outputs `true` if `a` is less than `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    Less,
    <
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is greater than `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    Greater,
    >
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is equal to `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    Equal,
    ==
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is not equal to `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    NotEqual,
    !=
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is less than or equal to `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    LessOrEqual,
    <=
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is greater than or equal to `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    GreaterOrEqual,
    >=
);
