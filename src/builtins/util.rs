use crate::{
    message::{Bang, BoxedMessage, Message},
    prelude::{GraphBuilder, Node, Process, SignalSpec},
    processor::ProcessorError,
    signal::{Sample, Signal, SignalBuffer},
};

#[derive(Clone, Debug)]
pub struct MessageProc(BoxedMessage);

impl MessageProc {
    pub fn new(message: impl Message) -> Self {
        Self(Box::new(message))
    }
}

impl Process for MessageProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("trig", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("message", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let bang = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let message = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (bang, message) in itertools::izip!(bang, message) {
            if bang.is_some() {
                *message = Some(self.0.clone());
            } else {
                *message = None;
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that sends a message when triggered.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `trig` | `Bang` | | Triggers the message. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `message` | `Message` | The message to send. |
    pub fn message(&self, message: impl Message) -> Node {
        self.add_processor(MessageProc::new(message))
    }
}

#[derive(Clone, Debug)]
pub struct ConstantMessageProc(BoxedMessage);

impl ConstantMessageProc {
    pub fn new(message: impl Message) -> Self {
        Self(Box::new(message))
    }
}

impl Process for ConstantMessageProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("message", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        _inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let message = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for message in message {
            *message = Some(self.0.clone());
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that sends a constant message.
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `message` | `Message` | The constant message. |
    pub fn constant_message(&self, message: impl Message) -> Node {
        self.add_processor(ConstantMessageProc::new(message))
    }
}

#[derive(Clone, Debug, Default)]
pub struct PrintProc {
    pub name: Option<String>,
    pub msg: Option<String>,
}

impl PrintProc {
    pub fn new(name: Option<&str>, msg: Option<&str>) -> Self {
        Self {
            name: name.map(String::from),
            msg: msg.map(String::from),
        }
    }

    pub fn with_name(name: &str) -> Self {
        Self {
            name: Some(String::from(name)),
            ..Self::default()
        }
    }

    pub fn with_msg(msg: &str) -> Self {
        Self {
            msg: Some(String::from(msg)),
            ..Self::default()
        }
    }

    pub fn with_name_and_msg(name: &str, msg: &str) -> Self {
        Self {
            name: Some(String::from(name)),
            msg: Some(String::from(msg)),
        }
    }
}

impl Process for PrintProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("trig", Signal::new_message_none()),
            SignalSpec::unbounded("message", Signal::new_message_none()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        _outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let print = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;
        let message = inputs[1]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        if !print.is_all::<Bang>() {
            return Err(ProcessorError::InputSpecMismatch(0));
        }

        for (bang, message) in itertools::izip!(print, message) {
            if let Some(message) = message {
                self.msg = Some(format!("{}", message));
            }

            if bang.is_some() {
                match (self.name.as_ref(), self.msg.as_ref()) {
                    (Some(name), Some(msg)) => {
                        println!("{}: {}", name, msg);
                    }
                    (Some(name), None) => {
                        println!("{}", name);
                    }
                    (None, Some(msg)) => {
                        println!("{}", msg);
                    }
                    (None, None) => {
                        println!();
                    }
                }
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that prints a message when triggered.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `trig` | `Bang` | | Triggers the print. |
    /// | `1` | `message` | `Message` | | The message to print. |
    pub fn print(&self, name: Option<&str>, msg: Option<&str>) -> Node {
        self.add_processor(PrintProc::new(name, msg))
    }
}

#[derive(Clone, Debug, Default)]
pub struct MessageToSampleProc;

impl Process for MessageToSampleProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("message", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("sample", 0.0)]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let message = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;
        let sample_out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (message, sample_out) in itertools::izip!(message, sample_out) {
            if let Some(message) = message {
                if let Some(sample) = message.downcast_ref::<f64>() {
                    *sample_out = Sample::new(*sample);
                }
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that converts a message to a sample.
    ///
    /// Non-f64 messages are ignored.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `message` | `Message` | | The message to convert. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `sample` | `Sample` | The sample value. |
    pub fn m2s(&self) -> Node {
        self.add_processor(MessageToSampleProc)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SampleToMessageProc;

impl Process for SampleToMessageProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("sample", 0.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("message", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let sample = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;
        let message_out = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (sample, message_out) in itertools::izip!(sample, message_out) {
            *message_out = Some(Box::new(sample.value()));
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that converts a sample to an f64 message.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `sample` | `Sample` | | The sample to convert. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `message` | `Message` | The message value. |
    pub fn s2m(&self) -> Node {
        self.add_processor(SampleToMessageProc)
    }
}

#[derive(Clone, Debug, Default)]
pub struct F2IProc;

impl Process for F2IProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("f", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("i", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let f = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;
        let i = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (f, i) in itertools::izip!(f, i) {
            if let Some(f) = f {
                if let Some(f) = (**f).downcast_ref::<f64>() {
                    *i = Some(Box::new(f.floor() as i64));
                } else {
                    return Err(ProcessorError::InputSpecMismatch(0));
                }
            } else {
                *i = None;
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that converts a float message to an integer message.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `f` | `Message(f64)` | | The float message to convert. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `i` | `Message(i64)` | The integer message. |
    pub fn f2i(&self) -> Node {
        self.add_processor(F2IProc)
    }
}

#[derive(Clone, Debug, Default)]
pub struct I2FProc;

impl Process for I2FProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("i", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("f", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let i = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;
        let f = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (i, f) in itertools::izip!(i, f) {
            if let Some(i) = i {
                if let Some(i) = (**i).downcast_ref::<i64>() {
                    *f = Some(Box::new(*i as f64));
                } else {
                    return Err(ProcessorError::InputSpecMismatch(0));
                }
            } else {
                *f = None;
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that converts an integer message to a float message.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `i` | `Message(i64)` | | The integer message to convert. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `f` | `Message(f64)` | The float message. |
    pub fn i2f(&self) -> Node {
        self.add_processor(I2FProc)
    }
}
