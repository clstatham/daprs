//! Utility processors.

use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};

use crate::{
    message::Message,
    prelude::{GraphBuilder, Node, Process, SignalSpec},
    processor::ProcessorError,
    signal::{Sample, Signal, SignalBuffer},
};

/// A processor that forwards its input to its output.
#[derive(Clone, Debug)]
pub struct Passthrough;

impl Process for Passthrough {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("in", 0.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
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
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        out_signal.copy_from_slice(in_signal);

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that forwards its input to its output.
    ///
    /// See also: [Passthrough].
    pub fn passthrough(&self) -> Node {
        self.add(Passthrough)
    }
}

/// A processor that sends a message when triggered.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `trig` | `Bang` | | Triggers the message. |
/// | `1` | `message` | `Message` | | The message to send. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Message` | The message to send. |
#[derive(Clone, Debug)]
pub struct MessageSender {
    message: Option<Message>,
}

impl MessageSender {
    /// Creates a new `MessageProc` with the given initial message.
    pub fn new(message: Message) -> Self {
        Self {
            message: Some(message),
        }
    }
}

impl Process for MessageSender {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("trig", Signal::new_message_none()),
            SignalSpec::unbounded("message", Signal::new_message_none()),
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
        let bang = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let message = inputs[1]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let out = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (bang, message, out) in itertools::izip!(bang, message, out) {
            if let Some(message) = message {
                self.message = Some(message.clone());
            }

            if bang.is_some() {
                *out = self.message.clone();
            } else {
                *out = None;
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that sends a message when triggered.
    ///
    /// See also: [MessageSender].
    pub fn message(&self, message: impl Into<Message>) -> Node {
        self.add(MessageSender::new(message.into()))
    }
}

/// A processor that sends a constant message every sample.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `message` | `Message` | The constant message. |
#[derive(Clone, Debug)]
pub struct ConstantMessageSender(Message);

impl ConstantMessageSender {
    /// Creates a new `ConstantMessageSender` with the given message.
    pub fn new(message: Message) -> Self {
        Self(message)
    }
}

impl Process for ConstantMessageSender {
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
    /// A processor that sends a constant message every sample.
    ///
    /// See also: [ConstantMessageSender].
    pub fn constant_message(&self, message: impl Into<Message>) -> Node {
        self.add(ConstantMessageSender::new(message.into()))
    }
}

/// A processor that prints a message when triggered.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `trig` | `Message(Bang)` | | Triggers the print. |
/// | `1` | `message` | `Message` | | The message to print. |
#[derive(Clone, Debug, Default)]
pub struct Print {
    name: Option<String>,
    msg: Option<String>,
}

impl Print {
    /// Creates a new `Print`, optionally with a name and message.
    pub fn new(name: Option<&str>, msg: Option<&str>) -> Self {
        Self {
            name: name.map(String::from),
            msg: msg.map(String::from),
        }
    }

    /// Creates a new `Print` with the given name.
    pub fn with_name(name: &str) -> Self {
        Self {
            name: Some(String::from(name)),
            ..Self::default()
        }
    }

    /// Creates a new `Print` with the given message.
    pub fn with_msg(msg: &str) -> Self {
        Self {
            msg: Some(String::from(msg)),
            ..Self::default()
        }
    }

    /// Creates a new `Print` with the given name and message.
    pub fn with_name_and_msg(name: &str, msg: &str) -> Self {
        Self {
            name: Some(String::from(name)),
            msg: Some(String::from(msg)),
        }
    }
}

impl Process for Print {
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
    /// See also: [Print].
    pub fn print<'a>(
        &self,
        name: impl Into<Option<&'a str>>,
        msg: impl Into<Option<&'a str>>,
    ) -> Node {
        self.add(Print::new(name.into(), msg.into()))
    }
}

/// A processor that converts a message to an audio signal.
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
#[derive(Clone, Debug, Default)]

pub struct MessageToAudio;

impl Process for MessageToAudio {
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
    /// A processor that converts a message to an audio signal.
    ///
    /// See also: [MessageToAudio].
    pub fn message_to_audio(&self) -> Node {
        self.add(MessageToAudio)
    }
}

/// A processor that converts an audio sample to a float message.
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
#[derive(Clone, Debug, Default)]

pub struct AudioToMessage;

impl Process for AudioToMessage {
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
    /// A processor that converts an audio sample to a float message.
    ///
    /// See also: [AudioToMessage].
    pub fn audio_to_message(&self) -> Node {
        self.add(AudioToMessage)
    }
}

/// A processor that outputs the sample rate that the graph is running at.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `sample_rate` | `Sample` | The sample rate. |
#[derive(Clone, Debug, Default)]
pub struct SampleRate {
    sample_rate: f64,
}

impl Process for SampleRate {
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
    /// See also: [SampleRate].
    pub fn sample_rate(&self) -> Node {
        self.add(SampleRate::default())
    }
}

#[inline(always)]
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

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
#[derive(Clone, Debug, Default)]
pub struct Smooth {
    current: f64,
}

impl Process for Smooth {
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
    /// See also: [Smooth].
    pub fn smooth(&self) -> Node {
        self.add(Smooth::default())
    }
}

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
#[derive(Clone, Debug, Default)]
pub struct Changed {
    last: f64,
}

impl Process for Changed {
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
    /// See also: [Changed].
    pub fn changed(&self) -> Node {
        self.add(Changed::default())
    }
}

/// A processor that sends a bang message when a zero crossing is detected on the input signal.
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
#[derive(Clone, Debug, Default)]
pub struct ZeroCrossing {
    last: f64,
}

impl Process for ZeroCrossing {
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
    /// A processor that sends a bang message when a zero crossing is detected on the input signal.
    ///
    /// See also: [ZeroCrossing].
    pub fn zero_crossing(&self) -> Node {
        self.add(ZeroCrossing::default())
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

    /// Receives a message from the `Param`.
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
    name: String,
    channels: (ParamTx, ParamRx),
}

impl Param {
    /// Creates a new `Param`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            channels: param_channels(),
        }
    }

    /// Returns the name of this `Param`.
    pub fn name(&self) -> &str {
        &self.name
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
        self.tx().send(message);
    }

    /// Gets the `Param`'s value.
    pub fn get(&mut self) -> Option<Message> {
        self.rx_mut().recv()
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

impl GraphBuilder {
    /// A processor that can be used to send/receive messages from outside the graph.
    ///
    /// See also: [Param].
    pub fn param(&self, param: &Param) -> Node {
        self.add_param(param.clone())
    }
}

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

            self.last_index = index;

            if index >= 0 && index < self.num_outputs as i64 {
                let out_signal = outputs[index as usize]
                    .as_message_mut()
                    .ok_or(ProcessorError::OutputSpecMismatch(index as usize))?;

                out_signal[sample_index] = in_signal.clone();

                for (i, out_signal) in outputs.iter_mut().enumerate() {
                    if i != index as usize {
                        let out_signal = out_signal
                            .as_message_mut()
                            .ok_or(ProcessorError::OutputSpecMismatch(i))?;
                        out_signal[sample_index] = None;
                    }
                }
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A processor that selects an output based on an index.
    ///
    /// See also: [Select].
    pub fn select(&self, num_outputs: usize) -> Node {
        self.add(Select::new(num_outputs))
    }
}

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
    /// See also: [Merge].
    pub fn merge(&self, num_inputs: usize) -> Node {
        self.add(Merge::new(num_inputs))
    }
}

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
#[derive(Clone, Debug, Default)]
pub struct Counter {
    count: i64,
}

impl Process for Counter {
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
    /// See also: [Counter].
    pub fn counter(&self) -> Node {
        self.add(Counter::default())
    }
}

/// A sample-and-hold processor.
///
/// The processor holds the last value it received when triggered.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `in` | `Sample` | | The input signal to sample. |
/// | `1` | `trig` | `Message(Bang)` | | Triggers the sample-and-hold. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The sampled value. |
#[derive(Clone, Debug, Default)]
pub struct SampleAndHold {
    last: Option<Sample>,
}

impl Process for SampleAndHold {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("in", 0.0),
            SignalSpec::unbounded("trig", Signal::new_message_none()),
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
        let in_signal = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let trig = inputs[1]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let out_signal = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (in_signal, trig, out_signal) in itertools::izip!(in_signal, trig, out_signal) {
            let in_signal = **in_signal;

            if trig.is_some() {
                self.last = Some(Sample::new(in_signal));
            }

            if let Some(last) = self.last {
                *out_signal = last;
            } else {
                *out_signal = Sample::new(0.0);
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A sample-and-hold processor.
    ///
    /// See also: [SampleAndHold].
    pub fn sample_and_hold(&self) -> Node {
        self.add(SampleAndHold::default())
    }
}
