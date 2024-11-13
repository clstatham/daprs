use std::marker::PhantomData;

use crate::{prelude::*, signal::Signal};

#[derive(Debug, Clone, Default)]
pub struct Cond<S: Signal>(PhantomData<S>);

impl<S: Signal> Cond<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: Signal> Processor for Cond<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("cond", SignalType::Bool),
            SignalSpec::new("then", S::TYPE),
            SignalSpec::new("else", S::TYPE),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::TYPE)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, cond, if_true, if_false) in itertools::izip!(
            outputs.iter_output_as::<S>(0)?,
            inputs.iter_input_as_bools(0)?,
            inputs.iter_input_as::<S>(1)?,
            inputs.iter_input_as::<S>(2)?
        ) {
            let Some(cond) = cond else {
                *out = None;
                continue;
            };

            if cond {
                *out = if_true.clone();
            } else {
                *out = if_false.clone();
            }
        }

        Ok(())
    }
}

macro_rules! comparison_op {
    ($doc:literal, $name:ident, $invert:literal, $op:tt) => {
        #[derive(Debug, Clone, Default)]
        #[doc = $doc]
        pub struct $name<S: Signal>(PhantomData<S>);

        impl<S: Signal> $name<S> {
            pub fn new() -> Self {
                Self(PhantomData)
            }
        }

        impl<S: Signal> Processor for $name<S> {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("a", S::TYPE), SignalSpec::new("b", S::TYPE)]
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
                    inputs.iter_input_as::<S>(0)?,
                    inputs.iter_input_as::<S>(1)?
                ) {
                    if let (Some(a), Some(b)) = (a, b) {
                        *out = match a.partial_cmp(&b) {
                            Some(std::cmp::Ordering::$op) => Some(!$invert),
                            _ => Some($invert),
                        };
                    } else {
                        *out = None;
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

The comparison is done by casting the inputs to floats as implemented by the [`Message::cast_to_float`] method.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Message` | | The first value to compare. |
| `1` | `b` | `Message` | | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Message` | The result of the comparison. |
"#,
    Less,
    false,
    Less
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is greater than `b`, otherwise `false`.

The comparison is done by casting the inputs to floats as implemented by the [`Message::cast_to_float`] method.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Message` | | The first value to compare. |
| `1` | `b` | `Message` | | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Message` | The result of the comparison. |
"#,
    Greater,
    false,
    Greater
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is equal to `b`, otherwise `false`.

The comparison is done by casting the inputs to floats as implemented by the [`Message::cast_to_float`] method.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Message` | | The first value to compare. |
| `1` | `b` | `Message` | | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Message` | The result of the comparison. |
"#,
    Equal,
    false,
    Equal
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is not equal to `b`, otherwise `false`.

The comparison is done by casting the inputs to floats as implemented by the [`Message::cast_to_float`] method.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Message` | | The first value to compare. |
| `1` | `b` | `Message` | | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Message` | The result of the comparison. |
"#,
    NotEqual,
    true,
    Equal
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is less than or equal to `b`, otherwise `false`.

The comparison is done by casting the inputs to floats as implemented by the [`Signal::cast_to_float`] method.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Message` | | The first value to compare. |
| `1` | `b` | `Message` | | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Message` | The result of the comparison. |
"#,
    LessOrEqual,
    true,
    Greater
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is greater than or equal to `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `0` | `a` | `Message` | | The first value to compare. |
| `1` | `b` | `Message` | | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Message` | The result of the comparison. |
"#,
    GreaterOrEqual,
    true,
    Less
);
