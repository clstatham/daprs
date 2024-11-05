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
pub struct MessageProc(Message);

impl MessageProc {
    /// Creates a new `MessageProc` with the given message.
    pub fn new(message: Message) -> Self {
        Self(message)
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
    pub fn message(&self, message: impl Into<Message>) -> Node {
        self.add_processor(MessageProc::new(message.into()))
    }
}

/// A processor that sends a constant message every sample.
///
/// See also: [constant_message](crate::builder::graph_builder::GraphBuilder::constant_message).
#[derive(Clone, Debug)]
pub struct ConstantMessageProc(Message);

impl ConstantMessageProc {
    /// Creates a new `ConstantMessageProc` with the given message.
    pub fn new(message: Message) -> Self {
        Self(message)
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
    pub fn constant_message(&self, message: impl Into<Message>) -> Node {
        self.add_processor(ConstantMessageProc::new(message.into()))
    }
}

/// A processor that prints a message when triggered.
///
/// See also: [print](crate::builder::graph_builder::GraphBuilder::print).
#[derive(Clone, Debug, Default)]

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

pub struct SampleRateProc {
    sample_rate: f64,
}

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

pub struct SmoothProc {
    current: f64,
}

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

pub struct ChangedProc {
    last: f64,
}

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

pub struct ZeroCrossingProc {
    last: f64,
}

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
pub struct Param {
    channels: (ParamTx, ParamRx),
    value: Arc<Mutex<Option<Message>>>,
}

impl Param {
    /// Creates a new `Param`.
    pub fn new() -> Self {
        Self {
            channels: param_channels(),
            value: Arc::new(Mutex::new(None)),
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
        let message = message.into();
        *self.value.try_lock().unwrap() = Some(message.clone());
        self.tx().send(message);
    }

    /// Gets the `Param`'s value.
    pub fn get(&mut self) -> Option<Message> {
        let message = self.rx_mut().recv();
        let mut value = self.value.try_lock().unwrap();
        *value = message.clone();
        value.clone()
    }
}

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

            if let Some(msg) = self.get() {
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
    pub fn param(&self, param: &Param) -> Node {
        self.add_processor(param.clone())
    }
}

/// A processor that routes a message to one of its outputs.
///
/// See also: [select](crate::builder::graph_builder::GraphBuilder::select).
#[derive(Clone, Debug)]

pub struct Select {
    num_outputs: usize,
    last_index: i64,
}

impl Select {
    /// Creates a new `Select` with the given number of outputs.
    pub fn new(num_outputs: usize) -> Self {
        Self {
            last_index: 0,
            num_outputs,
        }
    }
}

impl Default for Select {
    fn default() -> Self {
        Self::new(2)
    }
}

impl Process for Select {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("in", Signal::new_message_none()),
            SignalSpec::unbounded("index", Signal::new_message_some(Message::Int(0))),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::unbounded(format!("{}", i), Signal::new_message_none()))
            .collect()
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let index = inputs[1]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        for (sample_index, (in_signal, index)) in itertools::izip!(in_signal, index).enumerate() {
            let index = index
                .as_ref()
                .and_then(|index| index.cast_to_int())
                .unwrap_or(0);
            if index != self.last_index {
                self.last_index = index;
            }

            if index >= 0 && index < self.num_outputs as i64 {
                let out_signal = outputs[index as usize]
                    .as_message_mut()
                    .ok_or(ProcessorError::OutputSpecMismatch(index as usize))?;

                out_signal[sample_index] = in_signal.clone();
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that routes a message to one of its outputs.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `in` | `Message` | | The message to route. |
    /// | `1` | `index` | `Message(int)` | `0` | The index of the output to route to. |
    ///
    /// # Outputs
    ///
    /// Note that the number of outputs is determined by the number specified at construction.
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `0` | `Message` | The message, if routed to output `0`. |
    /// | `1` | `1` | `Message` | The message, if routed to output `1`. |
    /// | `...` | `...` | `...` | etc... |
    pub fn select(&self, num_outputs: usize) -> Node {
        self.add_processor(Select::new(num_outputs))
    }
}

/// A processor that outputs any messages it receives on any of its inputs.
///
/// See also: [merge](crate::builder::graph_builder::GraphBuilder::merge).
#[derive(Clone, Debug)]

pub struct Merge {
    num_inputs: usize,
}

impl Merge {
    /// Creates a new `Merge` with the given number of inputs.
    pub fn new(num_inputs: usize) -> Self {
        Self { num_inputs }
    }
}

impl Default for Merge {
    fn default() -> Self {
        Self::new(2)
    }
}

impl Process for Merge {
    fn input_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_inputs)
            .map(|i| SignalSpec::unbounded(format!("{}", i), Signal::new_message_none()))
            .collect()
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        for (i, input) in inputs.iter().enumerate() {
            let in_signal = input
                .as_message()
                .ok_or(ProcessorError::InputSpecMismatch(i))?;

            let out_signal = outputs[0]
                .as_message_mut()
                .ok_or(ProcessorError::OutputSpecMismatch(0))?;

            for (in_signal, out_signal) in itertools::izip!(in_signal, out_signal) {
                if let Some(in_signal) = in_signal {
                    *out_signal = Some(in_signal.clone());
                }
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that outputs any messages it receives on any of its inputs.
    ///
    /// If a message is received on multiple inputs, the message from the input with the lowest index is output.
    ///
    /// # Inputs
    ///
    /// Note that the number of inputs is determined by the number specified at construction.
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `0` | `Message` | | The message to merge. |
    /// | `1` | `1` | `Message` | | The message to merge. |
    /// | `...` | `...` | `...` | | etc... |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `out` | `Message` | The merged message. |
    pub fn merge(&self, num_inputs: usize) -> Node {
        self.add_processor(Merge::new(num_inputs))
    }
}

/// A processor that counts the number of times it receives a bang message.
///
/// See also: [counter](crate::builder::graph_builder::GraphBuilder::counter).
#[derive(Clone, Debug, Default)]

pub struct CounterProc {
    count: i64,
}

impl Process for CounterProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("trig", Signal::new_message_none()),
            SignalSpec::unbounded("reset", Signal::new_message_none()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("count", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let trig = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let reset = inputs[1]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let count = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (trig, reset, count) in itertools::izip!(trig, reset, count) {
            if reset.is_some() {
                self.count = 0;
            }

            *count = Some(Message::Int(self.count));

            if trig.is_some() {
                self.count += 1;
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that counts the number of times it receives a bang message.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `trig` | `Message(Bang)` | | Triggers the counter. |
    /// | `1` | `reset` | `Message(Bang)` | | Resets the counter. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `count` | `Message(Int)` | The current count. |
    pub fn counter(&self) -> Node {
        self.add_processor(CounterProc::default())
    }
}
