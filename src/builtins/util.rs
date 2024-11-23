//! Utility processors.

use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};
use raug_macros::iter_proc_io_as;

use crate::prelude::*;

use super::lerp;

/// A processor that does nothing.
///
/// This is used for audio inputs to the graph, since a buffer will be allocated for it, which will be filled by the audio backend.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The output signal. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Null;

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Null {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        _: ProcessorInputs,
        _outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        Ok(())
    }
}

/// A processor that passes its input to its output unchanged.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The output signal.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Passthrough {
    signal_type: SignalType,
}

impl Passthrough {
    /// Create a new `Passthrough` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self { signal_type }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Passthrough {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.signal_type.clone())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.signal_type.clone())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, mut out_signal) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(in_signal) = in_signal {
                out_signal.clone_from_ref(in_signal);
            } else {
                out_signal.set_none();
            }
        }
        Ok(())
    }
}

/// A processor that casts its input to a different signal type.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `S` | The input signal.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `T` | The output signal.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cast {
    from: SignalType,
    to: SignalType,
}

impl Cast {
    /// Create a new `Cast` processor.
    pub fn new(from: SignalType, to: SignalType) -> Self {
        Self { from, to }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Cast {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.from.clone())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.to.clone())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, mut out_signal) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            let Some(in_signal) = in_signal else {
                out_signal.set_none();
                continue;
            };
            let in_signal = in_signal.to_owned();
            let Some(cast) = in_signal.cast(self.to.clone()) else {
                return Err(ProcessorError::InvalidCast(
                    in_signal.signal_type(),
                    self.to.clone(),
                ));
            };
            out_signal.clone_from_ref(cast.as_ref());
        }

        Ok(())
    }
}

/// A processor that outputs a signal when triggered.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trig` | `Bool` | The trigger signal. |
/// | `1` | `in` | `Any` | The signal to output. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    message: AnySignal,
}

impl Message {
    /// Create a new `MessageSender` processor with the given message.
    pub fn new(message: impl Signal) -> Self {
        Self::new_any(message.into_any_signal())
    }

    pub fn new_any(message: AnySignal) -> Self {
        Self { message }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Message {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("message", self.message.signal_type()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.message.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (bang, message, mut out) in iter_proc_io_as!(
            inputs as [bool, Any],
            outputs as [Any]
        ) {
            if let Some(message) = message {
                if message.signal_type() != self.message.signal_type() {
                    return Err(ProcessorError::InputSpecMismatch {
                        index: 1,
                        expected: self.message.signal_type(),
                        actual: message.signal_type(),
                    });
                }
                self.message.clone_from_ref(message);
            }

            if bang.unwrap_or(false) {
                out.clone_from_ref(self.message.as_ref());
            } else {
                out.set_none();
            }
        }

        Ok(())
    }
}

/// A processor that prints a signal to the console when triggered.
///
/// The signal will be cast to a string before printing.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trig` | `Bool` | The trigger signal. |
/// | `1` | `message` | `Any` | The message to print. |
///
/// # Outputs
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Print {
    msg: AnySignal,
}

impl Print {
    /// Create a new `Print` processor with the given message.
    pub fn with_message(message: impl Signal) -> Self {
        Self {
            msg: message.into_any_signal(),
        }
    }

    /// Create a new `Print` processor with an empty message.
    pub fn new(signal_type: SignalType) -> Self {
        Self {
            msg: AnySignal::default_of_type(&signal_type),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Print {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("message", self.msg.signal_type()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (bang, message) in iter_proc_io_as!(inputs as [bool, Any], outputs as []) {
            if let Some(message) = message {
                self.msg = message.to_owned();
            }

            if bang.unwrap_or(false) {
                if let AnySignal::String(msg) = &self.msg {
                    if let Some(msg) = msg {
                        println!("{}", msg);
                    } else {
                        println!();
                    }
                } else {
                    println!("{:?}", self.msg);
                }
            }
        }

        Ok(())
    }
}

/// A processor that continuously outputs the current sample rate.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `sample_rate` | `Float` | The sample rate. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SampleRate;

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for SampleRate {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("sample_rate", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        outputs.output(0).fill_as::<Float>(inputs.sample_rate());

        Ok(())
    }
}

impl GraphBuilder {
    /// Adds a new [`SampleRate`] processor that continuously outputs the current sample rate.
    pub fn sample_rate(&self) -> Node {
        self.add(SampleRate)
    }
}

/// A processor that smooths a signal to a target value using a smoothing factor.
///
/// The output signal will converge to the target value with a speed determined by the smoothing factor.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `target` | `Float` | The target value to smooth to. |
/// | `1` | `factor` | `Float` | The smoothing factor. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The smoothed output signal. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Smooth {
    current: Float,
    factor: Float,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Smooth {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("target", SignalType::Float),
            SignalSpec::new("factor", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (target, factor, out) in iter_proc_io_as!(
            inputs as [Float, Float],
            outputs as [Float]
        ) {
            self.factor = factor.unwrap_or(self.factor).clamp(0.0, 1.0);

            let Some(target) = target else {
                *out = Some(self.current);
                continue;
            };

            self.current = lerp(self.current, *target, self.factor);

            *out = Some(self.current);
        }

        Ok(())
    }
}

/// A processor that outputs a signal when the input signal changes by more than a threshold.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `threshold` | `Float` | The threshold for the change detection. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The change signal. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Changed {
    last: Option<Float>,
    threshold: Float,
    include_none: bool,
}

impl Changed {
    /// Create a new `Changed` processor with the given threshold.
    pub fn new(threshold: Float, include_none: bool) -> Self {
        Self {
            last: None,
            threshold,
            include_none,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Changed {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("threshold", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, threshold, out_signal) in iter_proc_io_as!(
            inputs as [Float, Float],
            outputs as [bool]
        ) {
            self.threshold = threshold.unwrap_or(self.threshold);

            match (self.last, in_signal) {
                (Some(last), Some(in_signal)) => {
                    if (last - in_signal).abs() > self.threshold {
                        *out_signal = Some(true);
                    } else {
                        *out_signal = None;
                    }
                }
                (None, Some(_)) if self.include_none => {
                    *out_signal = Some(true);
                }
                (Some(_), None) if self.include_none => {
                    *out_signal = Some(true);
                }
                _ => {
                    *out_signal = None;
                }
            }

            self.last = *in_signal;
        }

        Ok(())
    }
}

/// A processor that outputs a signal when the input signal crosses zero.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The zero crossing signal. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZeroCrossing {
    last: Float,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for ZeroCrossing {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, out_signal) in iter_proc_io_as!(inputs as [Float], outputs as [bool]) {
            let Some(in_signal) = *in_signal else {
                *out_signal = None;
                continue;
            };

            if (self.last < 0.0 && in_signal >= 0.0) || (self.last > 0.0 && in_signal <= 0.0) {
                *out_signal = Some(true);
            } else {
                *out_signal = None;
            }

            self.last = in_signal;
        }

        Ok(())
    }
}

/// A processor that transmits a signal to a corresponding [`SignalRx`] receiver.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// None.
#[derive(Clone, Debug)]
pub struct SignalTx {
    tx: Sender<AnySignal>,
}

impl SignalTx {
    pub(crate) fn new(tx: Sender<AnySignal>) -> Self {
        Self { tx }
    }

    /// Sends a message to the receiver.
    pub fn send(&self, message: AnySignal) {
        self.tx.try_send(message).ok();
    }
}

/// A processor that receives a signal from a corresponding [`SignalTx`] transmitter.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The output signal. |
#[derive(Clone, Debug)]
pub struct SignalRx {
    rx: Receiver<AnySignal>,
}

impl SignalRx {
    pub(crate) fn new(rx: Receiver<AnySignal>) -> Self {
        Self { rx }
    }

    /// Receives a message from the transmitter.
    pub fn recv(&mut self) -> Option<AnySignal> {
        self.rx.try_recv().ok()
    }
}

/// A wrapper around a [`SignalRx`] receiver that stores the last received message. Used as part of a [`Param`] processor.
#[derive(Clone, Debug)]
pub struct ParamRx {
    rx: SignalRx,
    last: Arc<Mutex<Option<AnySignal>>>,
}

impl ParamRx {
    pub(crate) fn new(rx: SignalRx) -> Self {
        Self {
            rx,
            last: Arc::new(Mutex::new(None)),
        }
    }

    /// Receives a message from the transmitter and stores it as the last message.
    pub fn recv(&mut self) -> Option<AnySignal> {
        let mut last = self.last.try_lock().ok()?;
        if let Some(msg) = self.rx.recv() {
            if let Some(last) = &mut *last {
                last.clone_from(&msg);
            } else {
                *last = Some(msg.clone());
            }
            Some(msg)
        } else {
            None
        }
    }

    /// Returns the last received message.
    pub fn last(&self) -> Option<AnySignal> {
        self.last.try_lock().ok()?.clone()
    }
}

/// Creates a new set of connected [`SignalTx`] and [`SignalRx`] transmitters and receivers.
pub fn signal_channel() -> (SignalTx, SignalRx) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (SignalTx::new(tx), SignalRx::new(rx))
}

pub(crate) fn param_channel() -> (SignalTx, ParamRx) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (SignalTx::new(tx), ParamRx::new(SignalRx::new(rx)))
}

#[derive(Clone, Debug)]
struct ParamChannel(SignalTx, ParamRx);

impl Default for ParamChannel {
    fn default() -> Self {
        let (tx, rx) = param_channel();
        Self(tx, rx)
    }
}

/// A processor that can be used to control a parameter from outside the graph.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `set` | `Any` | The value to set the parameter to. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `get` | `Any` | The current value of the parameter. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Param {
    name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    channel: ParamChannel,
    signal_type: SignalType,
    minimum: Option<Float>,
    maximum: Option<Float>,
}

impl Param {
    /// Creates a new `Param` processor with the given name and optional initial value.
    pub fn new<S: Signal>(name: impl Into<String>, initial_value: impl Into<Option<S>>) -> Self {
        let this = Self {
            name: name.into(),
            channel: ParamChannel::default(),
            signal_type: S::signal_type(),
            minimum: None,
            maximum: None,
        };
        if let Some(initial_value) = initial_value.into() {
            this.send(initial_value);
        }
        this
    }

    pub fn bounded(
        name: impl Into<String>,
        initial_value: impl Into<Option<Float>>,
        minimum: impl Into<Option<Float>>,
        maximum: impl Into<Option<Float>>,
    ) -> Self {
        let this = Self {
            name: name.into(),
            channel: ParamChannel::default(),
            signal_type: SignalType::Float,
            minimum: minimum.into(),
            maximum: maximum.into(),
        };
        if let Some(initial_value) = initial_value.into() {
            this.send(initial_value);
        }
        this
    }

    /// Returns the name of the parameter.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the transmitter for the parameter.
    pub fn tx(&self) -> &SignalTx {
        &self.channel.0
    }

    /// Returns the receiver for the parameter.
    pub fn rx(&self) -> &ParamRx {
        &self.channel.1
    }

    /// Returns a mutable reference to the receiver for the parameter.
    pub fn rx_mut(&mut self) -> &mut ParamRx {
        &mut self.channel.1
    }

    /// Sends a value to the parameter.
    pub fn send(&self, message: impl Signal) {
        let message = message.into_any_signal();
        match (message, self.minimum, self.maximum) {
            (AnySignal::Float(Some(value)), Some(min), Some(max)) => {
                self.tx()
                    .send(AnySignal::Float(Some(value.clamp(min, max))));
            }
            (AnySignal::Float(Some(value)), Some(min), None) => {
                self.tx().send(AnySignal::Float(Some(value.max(min))));
            }
            (AnySignal::Float(Some(value)), None, Some(max)) => {
                self.tx().send(AnySignal::Float(Some(value.min(max))));
            }
            (message, _, _) => self.tx().send(message),
        }
    }

    /// Receives the value of the parameter.
    pub fn recv(&mut self) -> Option<AnySignal> {
        let message = self.rx_mut().recv();

        match (message, self.minimum, self.maximum) {
            (Some(AnySignal::Float(Some(value))), Some(min), Some(max)) => {
                Some(AnySignal::Float(Some(value.clamp(min, max))))
            }
            (Some(AnySignal::Float(Some(value))), Some(min), None) => {
                Some(AnySignal::Float(Some(value.max(min))))
            }
            (Some(AnySignal::Float(Some(value))), None, Some(max)) => {
                Some(AnySignal::Float(Some(value.min(max))))
            }
            (message, _, _) => message,
        }
    }

    /// Returns the last received value of the parameter.
    pub fn last(&self) -> Option<AnySignal> {
        let last = self.rx().last();

        match (last, self.minimum, self.maximum) {
            (Some(AnySignal::Float(Some(value))), Some(min), Some(max)) => {
                Some(AnySignal::Float(Some(value.clamp(min, max))))
            }
            (Some(AnySignal::Float(Some(value))), Some(min), None) => {
                Some(AnySignal::Float(Some(value.max(min))))
            }
            (Some(AnySignal::Float(Some(value))), None, Some(max)) => {
                Some(AnySignal::Float(Some(value.min(max))))
            }
            (last, _, _) => last,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Param {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("set", self.signal_type.clone())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("get", self.signal_type.clone())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (set, mut get) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(set) = set {
                self.tx().send(set.to_owned());
            }

            if let Some(msg) = self.rx_mut().recv() {
                get.clone_from_ref(msg.as_ref());
            } else if let Some(last) = self.rx().last() {
                get.clone_from_ref(last.as_ref());
            } else {
                get.set_none();
            }
        }

        Ok(())
    }
}

/// A processor that counts the number of times it has been triggered.
///
/// The counter is reset to zero when the reset signal is `true`.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trig` | `Bool` | The trigger signal. |
/// | `1` | `reset` | `Bool` | The reset signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `count` | `Int` | The current count. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Counter {
    count: i64,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Counter {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("reset", SignalType::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("count", SignalType::Int)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (trig, reset, count) in iter_proc_io_as!(
            inputs as [bool, bool],
            outputs as [i64]
        ) {
            if let Some(true) = reset {
                self.count = 0;
            }

            *count = Some(self.count);

            if let Some(true) = trig {
                self.count += 1;
            }
        }

        Ok(())
    }
}

/// A processor that captures the value of a signal when triggered and contuously outputs it.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `trig` | `Bool` | The trigger signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The output signal. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SampleAndHold {
    last: Option<Float>,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for SampleAndHold {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("trig", SignalType::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, trig, out_signal) in iter_proc_io_as!(
            inputs as [Float, bool],
            outputs as [Float]
        ) {
            if let Some(true) = trig {
                self.last = *in_signal;
            }

            *out_signal = self.last;
        }

        Ok(())
    }
}

/// A processor that panics with a message if the input signal is NaN or infinite.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The input signal passed through. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CheckFinite {
    context: String,
}

impl CheckFinite {
    /// Create a new `CheckFinite` processor with the given context for the panic message.
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for CheckFinite {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, out_signal) in iter_proc_io_as!(inputs as [Float], outputs as [Float]) {
            if let Some(in_signal) = in_signal {
                if in_signal.is_nan() {
                    panic!("{}: signal is NaN: {:?}", self.context, in_signal);
                }
                if in_signal.is_infinite() {
                    panic!("{}: signal is infinite: {:?}", self.context, in_signal);
                }
            }

            *out_signal = *in_signal;
        }

        Ok(())
    }
}

/// A processor that outputs 0.0 when the input signal is NaN or infinite.
/// Otherwise, it passes the input signal through unchanged.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The input signal passed through, or 0.0 if the input signal is NaN or infinite. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FiniteOrZero;

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for FiniteOrZero {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, out_signal) in iter_proc_io_as!(inputs as [Float], outputs as [Float]) {
            if let Some(in_signal) = *in_signal {
                if in_signal.is_nan() || in_signal.is_infinite() {
                    *out_signal = Some(0.0);
                } else {
                    *out_signal = Some(in_signal);
                }
            } else {
                *out_signal = None;
            }
        }

        Ok(())
    }
}

/// A processor that deduplicates a signal by only outputting a new value when it changes.
///
/// This can be thought of as the opposite of the [`Register`] processor, and will effectively undo its effect.
///
/// The output signal will likely be much sparser than the input signal, reducing the amount of data that needs to be processed downstream.
///
/// This processor can be useful when placed before an expensive processor (such as those dealing with lists) to reduce the amount of work it needs to do.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The deduplicated output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dedup {
    last: AnySignal,
}

impl Dedup {
    /// Create a new `Dedup` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self {
            last: AnySignal::default_of_type(&signal_type),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Dedup {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.last.signal_type())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.last.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, mut out_signal) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(in_signal) = in_signal {
                if self.last.as_ref() != in_signal {
                    out_signal.clone_from_ref(in_signal);
                } else {
                    out_signal.set_none();
                }
            } else {
                out_signal.set_none();
            }
        }

        Ok(())
    }
}

/// A processor that outputs `true` when the input signal is `Some`, and `false` when it is `None`.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IsSome {
    signal_type: SignalType,
}

impl IsSome {
    /// Create a new `IsSome` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self { signal_type }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for IsSome {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.signal_type.clone())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, out_signal) in iter_proc_io_as!(inputs as [Any], outputs as [bool]) {
            *out_signal = Some(in_signal.is_some_and(|signal| signal.is_some()));
        }

        Ok(())
    }
}

/// A processor that outputs `true` when the input signal is `None`, and `false` when it is `Some`.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IsNone {
    signal_type: SignalType,
}

impl IsNone {
    /// Create a new `IsNone` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self { signal_type }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for IsNone {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.signal_type.clone())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, out_signal) in iter_proc_io_as!(inputs as [Any], outputs as [bool]) {
            *out_signal = Some(!in_signal.is_some_and(|signal| signal.is_some()));
        }

        Ok(())
    }
}

/// A processor that outputs the input signal if it is Some, otherwise it outputs a default value.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The input signal if it is Some, otherwise a default value. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrElse {
    default: AnySignal,
}

impl OrElse {
    /// Create a new `OrElse` processor.
    pub fn new(default: impl Signal) -> Self {
        Self {
            default: default.into_any_signal(),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for OrElse {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.default.signal_type())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.default.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, mut out_signal) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(in_signal) = in_signal {
                if in_signal.is_some() {
                    out_signal.clone_from_ref(in_signal);
                } else {
                    out_signal.clone_from_ref(self.default.as_ref());
                }
            } else {
                out_signal.clone_from_ref(self.default.as_ref());
            }
        }

        Ok(())
    }
}
