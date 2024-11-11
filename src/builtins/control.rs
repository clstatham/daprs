use crate::prelude::*;

/// A processor that selects one of its two inputs based on a condition.
///
/// The condition is considered truthy based on [`Message::is_truthy`].
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `cond` | `Message` | | The condition to evaluate for truthiness. |
/// | `1` | `then` | `Message` | | The value to output if the condition is truthy. |
/// | `2` | `else` | `Message` | | The value to output if the condition is falsy. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Message` | The selected value. |
#[derive(Debug, Clone)]
pub struct Cond;

impl Processor for Cond {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("cond", Signal::new_message_none()),
            SignalSpec::unbounded("then", Signal::new_message_none()),
            SignalSpec::unbounded("else", Signal::new_message_none()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, cond, if_true, if_false) in itertools::izip!(
            outputs.iter_output_mut_as_messages(0)?,
            inputs.iter_input_as_messages(0)?,
            inputs.iter_input_as_messages(1)?,
            inputs.iter_input_as_messages(2)?
        ) {
            let Some(cond) = cond else {
                *out = None;
                continue;
            };

            let Some(cond) = cond.cast_to_bool() else {
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
    ($doc:literal, $name:ident, $op:tt) => {
        #[derive(Debug, Clone, Default)]
        #[doc = $doc]
        pub struct $name;

        impl Processor for $name {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![
                    SignalSpec::unbounded("a", Signal::new_message_none()),
                    SignalSpec::unbounded("b", Signal::new_message_none()),
                ]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::unbounded("out", Signal::new_message_none())]
            }

            fn process(
                &mut self,
                inputs: ProcessorInputs,
                mut outputs: ProcessorOutputs,
            ) -> Result<(), ProcessorError> {
                for (out, a, b) in itertools::izip!(
                    outputs.iter_output_mut_as_messages(0)?,
                    inputs.iter_input_as_messages(0)?,
                    inputs.iter_input_as_messages(1)?
                ) {
                    let Some(a) = a else {
                        *out = None;
                        continue;
                    };
                    let Some(b) = b else {
                        *out = None;
                        continue;
                    };

                    if let (Some(a), Some(b)) = (a.cast_to_float(), b.cast_to_float()) {
                        *out = Some(Message::Bool(a $op b));
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
    Less, <);

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
    Greater, >);

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
    Equal, ==);

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
    NotEqual, !=);

comparison_op!(
        r#"
A processor that outputs `true` if `a` is less than or equal to `b`, otherwise `false`.

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
    LessOrEqual, <=);

comparison_op!(
        r#"
A processor that outputs `true` if `a` is greater than or equal to `b`, otherwise `false`.

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
    GreaterOrEqual, >=);
