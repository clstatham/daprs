//! Utility processors.

use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};

use crate::{
    message::Message,
    prelude::{GraphBuilder, Node, Processor, ProcessorInputs, ProcessorOutputs, SignalSpec},
    processor::ProcessorError,
    signal::{Sample, Signal},
};

/// A processor that forwards its input to its output.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `in` | `Sample` | | The input signal to forward. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The output signal. |
#[derive(Clone, Debug)]
pub struct Passthrough;

impl Processor for Passthrough {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("in", 0.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let Some(in_signal) = inputs.input(0) else {
            return Ok(());
        };

        let in_signal = in_signal
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let out_signal = outputs.output(0).as_sample_mut().unwrap();

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
    message: Message,
}

impl MessageSender {
    /// Creates a new `MessageProc` with the given initial message.
    pub fn new(message: Message) -> Self {
        Self { message }
    }
}

impl Processor for MessageSender {
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
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (bang, message, out) in itertools::izip!(
            inputs.iter_input_as_messages(0)?,
            inputs.iter_input_as_messages(1)?,
            outputs.iter_output_mut_as_messages(0)?
        ) {
            if message.is_some() {
                self.message = message.clone();
            }

            if bang.is_some() {
                *out = self.message.clone();
            } else {
                *out = Message::None;
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

impl Processor for ConstantMessageSender {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("message", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        outputs
            .output(0)
            .as_message_mut()
            .unwrap()
            .fill(self.0.clone());

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

impl Processor for Print {
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
        inputs: ProcessorInputs,
        _outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (bang, message) in itertools::izip!(
            inputs.iter_input_as_messages(0)?,
            inputs.iter_input_as_messages(1)?
        ) {
            if message.is_some() {
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
/// The message is converted to a float via [`Message::cast_to_float`]. If the conversion fails, the output is 0.0.
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
/// | `0` | `audio` | `Sample` | The audio value. |
#[derive(Clone, Debug, Default)]

pub struct MessageToAudio;

impl Processor for MessageToAudio {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("message", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("audio", 0.0)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (message, sample_out) in itertools::izip!(
            inputs.iter_input_as_messages(0)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            *sample_out = 0.0;
            if let Some(sample) = message.cast_to_float() {
                *sample_out = sample;
            }
        }

        Ok(())
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

impl Processor for AudioToMessage {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("sample", 0.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("message", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (sample, message_out) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            outputs.iter_output_mut_as_messages(0)?
        ) {
            *message_out = Message::Float(sample);
        }

        Ok(())
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
    sample_rate: Sample,
}

impl Processor for SampleRate {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("sample_rate", 0.0)]
    }

    fn resize_buffers(&mut self, sample_rate: Sample, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let sample_rate_out = outputs.output_as_samples(0)?;
        sample_rate_out.fill(self.sample_rate);

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
fn lerp(a: Sample, b: Sample, t: Sample) -> Sample {
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
    current: Sample,
}

impl Processor for Smooth {
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
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (target, factor, out) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            let factor = factor.clamp(0.0, 1.0);

            self.current = lerp(self.current, target, factor);

            *out = self.current;
        }

        Ok(())
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
    last: Sample,
}

impl Processor for Changed {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("in", 0.0),
            SignalSpec::unbounded("threshold", Sample::EPSILON),
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
        for (in_signal, threshold, out_signal) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
            outputs.iter_output_mut_as_messages(0)?
        ) {
            if (self.last - in_signal).abs() > threshold {
                *out_signal = Message::Bang;
            } else {
                *out_signal = Message::None;
            }

            self.last = in_signal;
        }

        Ok(())
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
    last: Sample,
}

impl Processor for ZeroCrossing {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("in", 0.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, out_signal) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            outputs.iter_output_mut_as_messages(0)?
        ) {
            if (self.last < 0.0 && in_signal >= 0.0) || (self.last > 0.0 && in_signal <= 0.0) {
                *out_signal = Message::Bang;
            } else {
                *out_signal = Message::None;
            }

            self.last = in_signal;
        }

        Ok(())
    }
}

/// A message sender, used for `Param` communication and breaking cycles in the graph.
#[derive(Clone, Debug)]
pub struct MessageTx {
    tx: Sender<Message>,
}

impl MessageTx {
    pub(crate) fn new(tx: Sender<Message>) -> Self {
        Self { tx }
    }

    /// Sends a message to the `Param`.
    pub fn send(&self, message: Message) {
        self.tx.try_send(message).unwrap();
    }
}

impl Processor for MessageTx {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("in", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        _outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs.iter_input_as_messages(0)?;

        for message in in_signal {
            self.send(message.clone());
        }

        Ok(())
    }
}

/// A message receiver, used for `Param` communication and breaking cycles in the graph.
#[derive(Clone, Debug)]
pub struct MessageRx {
    rx: Receiver<Message>,
}

impl MessageRx {
    pub(crate) fn new(rx: Receiver<Message>) -> Self {
        Self { rx }
    }

    /// Receives a message from the `Param`.
    pub fn recv(&mut self) -> Option<Message> {
        self.rx.try_recv().ok()
    }
}

impl Processor for MessageRx {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let out = outputs.iter_output_mut_as_messages(0)?;

        for out in out {
            if let Some(msg) = self.recv() {
                *out = msg;
            } else {
                *out = Message::None;
            }
        }

        Ok(())
    }
}

/// A receiver for a `Param`.
#[derive(Clone, Debug)]
pub struct ParamRx {
    rx: MessageRx,
    last: Arc<Mutex<Option<Message>>>,
}

impl ParamRx {
    pub(crate) fn new(rx: MessageRx) -> Self {
        Self {
            rx,
            last: Arc::new(Mutex::new(None)),
        }
    }

    /// Receives a message from the `Param`.
    pub fn recv(&mut self) -> Option<Message> {
        let mut last = self.last.try_lock().ok()?;
        if let Some(msg) = self.rx.recv() {
            *last = Some(msg.clone());
            Some(msg)
        } else {
            last.clone()
        }
    }
}

pub(crate) fn message_channel() -> (MessageTx, MessageRx) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (MessageTx::new(tx), MessageRx::new(rx))
}

pub(crate) fn param_channel() -> (MessageTx, ParamRx) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (MessageTx::new(tx), ParamRx::new(MessageRx::new(rx)))
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
    channels: (MessageTx, ParamRx),
}

impl Param {
    /// Creates a new `Param`.
    pub fn new(name: impl Into<String>, initial_value: impl Into<Option<Message>>) -> Self {
        let this = Self {
            name: name.into(),
            channels: param_channel(),
        };
        let initial_value = initial_value.into();
        if let Some(initial_value) = initial_value {
            this.set(initial_value);
        }
        this
    }

    /// Returns the name of this `Param`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the sender for this `Param`.
    pub fn tx(&self) -> &MessageTx {
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

impl Processor for Param {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("set", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("get", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (set, get) in itertools::izip!(
            inputs.iter_input_as_messages(0)?,
            outputs.iter_output_mut_as_messages(0)?
        ) {
            if set.is_some() {
                self.tx().send(set.clone());
            }

            if let Some(msg) = self.get() {
                *get = msg;
            } else {
                *get = Message::None;
            }
        }

        Ok(())
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

impl Processor for Select {
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
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (sample_index, (in_signal, index)) in itertools::izip!(
            inputs.iter_input_as_messages(0)?,
            inputs.iter_input_as_messages(1)?
        )
        .enumerate()
        {
            let index = index.cast_to_int().unwrap_or(0);

            self.last_index = index;

            if index >= 0 && index < self.num_outputs as i64 {
                let out_signal = outputs.output(index as usize).as_message_mut().unwrap();

                out_signal[sample_index] = in_signal.clone();

                for (i, out_signal) in outputs.iter_mut().enumerate() {
                    if i != index as usize {
                        let out_signal = out_signal
                            .as_message_mut()
                            .ok_or(ProcessorError::OutputSpecMismatch(i))?;
                        out_signal[sample_index] = Message::None;
                    }
                }
            }
        }

        Ok(())
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

impl Processor for Merge {
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
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (i, input) in inputs.iter().enumerate() {
            let Some(input) = input else {
                continue;
            };
            let in_signal = input
                .as_message()
                .ok_or(ProcessorError::InputSpecMismatch(i))?;

            let out_signal = outputs.iter_output_mut_as_messages(0)?;

            for (in_signal, out_signal) in itertools::izip!(in_signal, out_signal) {
                if in_signal.is_some() {
                    *out_signal = in_signal.clone();
                }
            }
        }

        Ok(())
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

impl Processor for Counter {
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
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (trig, reset, count) in itertools::izip!(
            inputs.iter_input_as_messages(0)?,
            inputs.iter_input_as_messages(1)?,
            outputs.iter_output_mut_as_messages(0)?
        ) {
            if reset.is_some() {
                self.count = 0;
            }

            *count = Message::Int(self.count);

            if trig.is_some() {
                self.count += 1;
            }
        }

        Ok(())
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

impl Processor for SampleAndHold {
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
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, trig, out_signal) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_messages(1)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            if trig.is_some() {
                self.last = Some(in_signal);
            }

            if let Some(last) = self.last {
                *out_signal = last;
            } else {
                *out_signal = 0.0;
            }
        }

        Ok(())
    }
}

/// A processor that panics if the input signal is infinite or NaN.
/// This is useful for debugging.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `in` | `Sample` | | The input signal to check. |
#[derive(Clone, Debug, Default)]
pub struct CheckFinite {
    context: String,
}

impl CheckFinite {
    /// Creates a new `CheckFinite` with the given context.
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
        }
    }
}

impl Processor for CheckFinite {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("in", 0.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        _outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs.iter_input_as_samples(0)?;
        for in_signal in in_signal {
            if !in_signal.is_finite() {
                panic!("{}: signal is not finite: {:?}", self.context, in_signal);
            }
        }

        Ok(())
    }
}
