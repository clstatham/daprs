//! Utility processors.

use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};

use crate::{
    message::Message,
    prelude::{GraphBuilder, Node, Process, SignalSpec},
    processor::ProcessorError,
    signal::{Sample, Signal, SignalBuffer},
};

/// A processor that sends a message when triggered.
///
/// See also: [message](crate::builder::graph_builder::GraphBuilder::message).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageProc(Message);

impl MessageProc {
    /// Creates a new `MessageProc` with the given message.
    pub fn new(message: Message) -> Self {
        Self(message)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
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

/// A processor that sends a constant message every sample.
///
/// See also: [constant_message](crate::builder::graph_builder::GraphBuilder::constant_message).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstantMessageProc(Message);

impl ConstantMessageProc {
    /// Creates a new `ConstantMessageProc` with the given message.
    pub fn new(message: Message) -> Self {
        Self(message)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
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
    pub fn constant_message(&self, message: impl Into<Message>) -> Node {
        self.add_processor(ConstantMessageProc::new(message.into()))
    }
}

/// A processor that prints a message when triggered.
///
/// See also: [print](crate::builder::graph_builder::GraphBuilder::print).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrintProc {
    name: Option<String>,
    msg: Option<String>,
}

impl PrintProc {
    /// Creates a new `PrintProc`, optionally with a name and message.
    pub fn new(name: Option<&str>, msg: Option<&str>) -> Self {
        Self {
            name: name.map(String::from),
            msg: msg.map(String::from),
        }
    }

    /// Creates a new `PrintProc` with the given name.
    pub fn with_name(name: &str) -> Self {
        Self {
            name: Some(String::from(name)),
            ..Self::default()
        }
    }

    /// Creates a new `PrintProc` with the given message.
    pub fn with_msg(msg: &str) -> Self {
        Self {
            msg: Some(String::from(msg)),
            ..Self::default()
        }
    }

    /// Creates a new `PrintProc` with the given name and message.
    pub fn with_name_and_msg(name: &str, msg: &str) -> Self {
        Self {
            name: Some(String::from(name)),
            msg: Some(String::from(msg)),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
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
    /// | `0` | `trig` | `Message(Bang)` | | Triggers the print. |
    /// | `1` | `message` | `Message` | | The message to print. |
    pub fn print<'a>(
        &self,
        name: impl Into<Option<&'a str>>,
        msg: impl Into<Option<&'a str>>,
    ) -> Node {
        self.add_processor(PrintProc::new(name.into(), msg.into()))
    }
}

/// A processor that converts a message to a sample.
///
/// See also: [to_audio](crate::builder::graph_builder::GraphBuilder::to_audio).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageToSampleProc;

#[cfg_attr(feature = "serde", typetag::serde)]
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
    pub fn to_audio(&self) -> Node {
        self.add_processor(MessageToSampleProc)
    }
}

/// A processor that converts a sample to an f64 message.
///
/// See also: [to_message](crate::builder::graph_builder::GraphBuilder::to_message).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SampleToMessageProc;

#[cfg_attr(feature = "serde", typetag::serde)]
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
    /// A processor that converts a sample to a float message.
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
    /// | `0` | `message` | `Message(Float)` | The message value. |
    pub fn to_message(&self) -> Node {
        self.add_processor(SampleToMessageProc)
    }
}

/// A processor that outputs the sample rate that the graph is running at.
///
/// See also: [sample_rate](crate::builder::graph_builder::GraphBuilder::sample_rate).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SampleRateProc {
    sample_rate: f64,
}

#[cfg_attr(feature = "serde", typetag::serde)]
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
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `sample_rate` | `Sample` | The sample rate. |
    pub fn sample_rate(&self) -> Node {
        self.add_processor(SampleRateProc::default())
    }
}

#[inline(always)]
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// A processor that smoothly ramps between values over time.
///
/// See also: [smooth](crate::builder::graph_builder::GraphBuilder::smooth).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmoothProc {
    current: f64,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Process for SmoothProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("target", 0.0),
            SignalSpec::unbounded("factor", 1.0),
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

        let factor = inputs[1]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (target, factor, out) in itertools::izip!(target, factor, out) {
            let target = **target;
            let factor = **factor;

            let factor = factor.clamp(0.0, 1.0);

            self.current = lerp(self.current, target, factor);

            **out = self.current;
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that smoothly interpolates between values over time.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `target` | `Sample` | 0.0 | The target value. |
    /// | `1` | `factor` | `Sample` | 1.0  | The factor of smoothing (0 <= factor <= 1). |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `out` | `Sample` | The current value of the interpolation. |
    pub fn smooth(&self) -> Node {
        self.add_processor(SmoothProc::default())
    }
}

/// A processor that sends a bang message when a value changes beyond a certain threshold from the last value.
///
/// See also: [changed](crate::builder::graph_builder::GraphBuilder::changed).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChangedProc {
    last: f64,
}

#[cfg_attr(feature = "serde", typetag::serde)]
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
    /// | `0` | `out` | `Message(Bang)` | A bang message when a change is detected. |
    pub fn changed(&self) -> Node {
        self.add_processor(ChangedProc::default())
    }
}

/// A processor that sends a bang message when a zero crossing is detected.
///
/// See also: [zero_crossing](crate::builder::graph_builder::GraphBuilder::zero_crossing).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZeroCrossingProc {
    last: f64,
}

#[cfg_attr(feature = "serde", typetag::serde)]
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
    /// | `0` | `out` | `Message(Bang)` | A bang message when a zero crossing is detected. |
    pub fn zero_crossing(&self) -> Node {
        self.add_processor(ZeroCrossingProc::default())
    }
}

/// A sender for a `Param`.
#[derive(Clone, Debug)]
pub struct ParamTx {
    tx: Sender<Message>,
}

impl ParamTx {
    pub(crate) fn new(tx: Sender<Message>) -> Self {
        Self { tx }
    }

    /// Sends a message to the `Param`.
    pub fn send(&self, message: Message) {
        self.tx.try_send(message).unwrap();
    }
}

/// A receiver for a `Param`.
#[derive(Clone, Debug)]
pub struct ParamRx {
    rx: Receiver<Message>,
    last: Arc<Mutex<Option<Message>>>,
}

impl ParamRx {
    pub(crate) fn new(rx: Receiver<Message>) -> Self {
        Self {
            rx,
            last: Arc::new(Mutex::new(None)),
        }
    }

    /// Receives a message, returning the last message if there are no new messages.
    pub fn recv(&mut self) -> Option<Message> {
        let mut last = self.last.try_lock().ok()?;
        if let Ok(msg) = self.rx.try_recv() {
            *last = Some(msg.clone());
            Some(msg)
        } else {
            last.clone()
        }
    }
}

fn param_channels() -> (ParamTx, ParamRx) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (ParamTx::new(tx), ParamRx::new(rx))
}

#[cfg(feature = "serde")]
fn deserialize_param_channels<'de, D>(_deserializer: D) -> Result<(ParamTx, ParamRx), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (tx, rx) = param_channels();
    Ok((tx, rx))
}

/// A processor that can be used to send/receive messages from outside the graph.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `set` | `Message` | | The message to set the parameter to. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `get` | `Message` | The current value of the parameter. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Param {
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing, deserialize_with = "deserialize_param_channels")
    )]
    channels: (ParamTx, ParamRx),
}

impl Param {
    /// Creates a new `Param`.
    pub fn new() -> Self {
        Self {
            channels: param_channels(),
        }
    }

    /// Returns the sender for this `Param`.
    pub fn tx(&self) -> &ParamTx {
        &self.channels.0
    }

    /// Returns the receiver for this `Param`.
    pub fn rx_mut(&mut self) -> &mut ParamRx {
        &mut self.channels.1
    }

    /// Sets the `Param`'s value.
    pub fn set(&self, message: impl Into<Message>) {
        self.tx().send(message.into());
    }

    /// Gets the `Param`'s value.
    pub fn get(&mut self) -> Option<Message> {
        self.rx_mut().recv()
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Process for Param {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("set", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("get", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let set = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let get = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (set, get) in itertools::izip!(set, get) {
            if let Some(set) = set {
                self.tx().send(set.clone());
            }

            if let Some(msg) = self.rx_mut().recv() {
                *get = Some(msg);
            } else {
                *get = None;
            }
        }

        Ok(())
    }
}

impl Default for Param {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilder {
    /// A processor that can be used to send/receive messages from outside the graph.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `set` | `Message` | | The message to set the parameter to. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `get` | `Message` | The current value of the parameter. |
    pub fn param(&self) -> Node {
        self.add_processor(Param::new())
    }
}
