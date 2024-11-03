use serde::{Deserialize, Serialize};

use crate::{
    message::Message,
    prelude::{GraphBuilder, Node, Process, SignalSpec},
    processor::ProcessorError,
    signal::{Sample, Signal, SignalBuffer},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageProc(Message);

impl MessageProc {
    pub fn new(message: Message) -> Self {
        Self(message)
    }
}

#[typetag::serde]
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
    pub fn message(&self, message: Message) -> Node {
        self.add_processor(MessageProc::new(message))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstantMessageProc(Message);

impl ConstantMessageProc {
    pub fn new(message: Message) -> Self {
        Self(message)
    }
}

#[typetag::serde]
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
    pub fn constant_message(&self, message: Message) -> Node {
        self.add_processor(ConstantMessageProc::new(message))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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

#[typetag::serde]
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

        if !print.is_all_bang() {
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
    pub fn print(
        &self,
        name: impl Into<Option<&'static str>>,
        msg: impl Into<Option<&'static str>>,
    ) -> Node {
        self.add_processor(PrintProc::new(name.into(), msg.into()))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MessageToSampleProc;

#[typetag::serde]
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
                if let Some(sample) = message.cast_to_float() {
                    *sample_out = Sample::new(sample);
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SampleToMessageProc;

#[typetag::serde]
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
            *message_out = Some(Message::Float(sample.value()));
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SampleRateProc {
    sample_rate: f64,
}

#[typetag::serde]
impl Process for SampleRateProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("sample_rate", 0.0)]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        _inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let sample_rate_out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        sample_rate_out.fill(Sample::new(self.sample_rate));

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that outputs the sample rate that the graph is running at.
    ///
    /// This processor outputs `Sample`s for convenience in connecting to other audio-rate processors.
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `sample_rate` | `Sample` | The sample rate of the graph. |
    pub fn sample_rate(&self) -> Node {
        self.add_processor(SampleRateProc::default())
    }
}

#[inline(always)]
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SmoothProc {
    current: f64,
}

#[typetag::serde]
impl Process for SmoothProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("target", 0.0),
            SignalSpec::unbounded("rate", 0.0),
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
        let target = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let rate = inputs[1]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (target, rate, out) in itertools::izip!(target, rate, out) {
            let target = **target;
            let rate = **rate;

            let rate = rate.clamp(0.0, 1.0);

            self.current = lerp(self.current, target, rate);

            **out = self.current;
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that smoothly ramps between values over time.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `target` | `Sample` | | The target value. |
    /// | `1` | `rate` | `Sample` | | The rate of smoothing. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `out` | `Sample` | The current value of the ramp. |
    pub fn smooth(&self) -> Node {
        self.add_processor(SmoothProc::default())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChangedProc {
    last: f64,
}

#[typetag::serde]
impl Process for ChangedProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("in", 0.0),
            SignalSpec::unbounded("threshold", 0.0),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let threshold = inputs[1]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let out_signal = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (in_signal, threshold, out_signal) in itertools::izip!(in_signal, threshold, out_signal)
        {
            let in_signal = **in_signal;
            let threshold = **threshold;

            if (self.last - in_signal).abs() > threshold {
                *out_signal = Some(Message::Bang);
            } else {
                *out_signal = None;
            }

            self.last = in_signal;
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that sends a bang message when a value changes beyond a certain threshold from the last value.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `in` | `Sample` | | The input signal to detect changes on. |
    /// | `1` | `threshold` | `Sample` | | The threshold for a change to be detected. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `out` | `Message` | A bang message when a change is detected. |
    pub fn changed(&self) -> Node {
        self.add_processor(ChangedProc::default())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ZeroCrossingProc {
    last: f64,
}

#[typetag::serde]
impl Process for ZeroCrossingProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("in", 0.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let out_signal = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (in_signal, out_signal) in itertools::izip!(in_signal, out_signal) {
            let in_signal = **in_signal;

            if (self.last < 0.0 && in_signal >= 0.0) || (self.last > 0.0 && in_signal <= 0.0) {
                *out_signal = Some(Message::Bang);
            } else {
                *out_signal = None;
            }

            self.last = in_signal;
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that sends a bang message when a zero crossing is detected.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `in` | `Sample` | | The input signal to detect zero crossings on. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `out` | `Message` | A bang message when a zero crossing is detected. |
    pub fn zero_crossing(&self) -> Node {
        self.add_processor(ZeroCrossingProc::default())
    }
}
