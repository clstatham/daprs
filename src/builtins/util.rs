//! Utility processors.

use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crossbeam_channel::{Receiver, Sender};

use crate::{
    prelude::{GraphBuilder, Node, Processor, ProcessorInputs, ProcessorOutputs, SignalSpec},
    processor::ProcessorError,
    signal::{Float, Signal, SignalType},
};

use super::lerp;

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
#[derive(Clone, Debug, Default)]
pub struct Passthrough<S: Signal + Clone>(PhantomData<S>);

impl<S: Signal + Clone> Passthrough<S> {
    /// Create a new `Passthrough` processor.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: Signal + Clone> Processor for Passthrough<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", S::TYPE)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::TYPE)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let Some(in_signal) = inputs.input(0) else {
            return Ok(());
        };

        let mut out_signal = outputs.output(0);

        let in_signal = in_signal.as_type::<S>().unwrap();
        let out_signal = out_signal.iter_mut::<S>();

        for (in_signal, out_signal) in itertools::izip!(in_signal, out_signal) {
            *out_signal = in_signal.clone();
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
#[derive(Clone, Debug, Default)]
pub struct Cast<S: Signal + Clone, T: Signal + Clone> {
    _phantom: PhantomData<(S, T)>,
}

impl<S: Signal + Clone, T: Signal + Clone> Cast<S, T> {
    /// Create a new `Cast` processor.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<S: Signal + Clone, T: Signal + Clone> Processor for Cast<S, T> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", S::TYPE)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", T::TYPE)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let Some(in_signal) = inputs.input(0) else {
            return Ok(());
        };

        let in_signal = in_signal.as_type::<S>().unwrap();

        let mut out_signal = outputs.output(0);
        let out_signal = out_signal.iter_mut::<T>();

        for (in_signal, out_signal) in itertools::izip!(in_signal, out_signal) {
            if let Some(in_signal) = in_signal {
                let in_signal = S::into_signal(in_signal.to_owned());
                *out_signal = in_signal.cast();
            }
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
pub struct Message<S: Signal + Clone> {
    message: S,
}

impl<S: Signal + Clone> Message<S> {
    /// Create a new `MessageSender` processor with the given message.
    pub fn new(message: S) -> Self {
        Self { message }
    }
}

impl<S: Signal + Clone> Processor for Message<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("message", S::TYPE),
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
        for (bang, message, out) in itertools::izip!(
            inputs.iter_input_as_bools(0)?,
            inputs.iter_input_as::<S>(1)?,
            outputs.iter_output_as::<S>(0)?
        ) {
            if let Some(message) = message {
                self.message.clone_from(message);
            }

            if let Some(true) = bang {
                if let Some(out) = out {
                    out.clone_from(&self.message);
                } else {
                    *out = Some(self.message.clone());
                }
            } else {
                *out = None;
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
pub struct Print<S: Signal + Clone> {
    msg: S,
}

impl<S: Signal + Default + Clone> Default for Print<S> {
    fn default() -> Self {
        Self { msg: S::default() }
    }
}

impl Print<String> {
    /// Create a new `Print` processor that prints a string.
    pub fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

impl Print<Float> {
    /// Create a new `Print` processor that prints a float.
    pub fn new(msg: impl Into<Float>) -> Self {
        Self { msg: msg.into() }
    }
}

impl Print<bool> {
    /// Create a new `Print` processor that prints a boolean.
    pub fn new(msg: impl Into<bool>) -> Self {
        Self { msg: msg.into() }
    }
}

impl Print<i64> {
    /// Create a new `Print` processor that prints an integer.
    pub fn new(msg: impl Into<i64>) -> Self {
        Self { msg: msg.into() }
    }
}

impl<S: Signal + Clone> Processor for Print<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("message", S::TYPE),
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
            inputs.iter_input_as_bools(0)?,
            inputs.iter_input_as::<S>(1)?
        ) {
            if let Some(message) = message {
                self.msg.clone_from(message);
            }

            if bang.unwrap_or(false) {
                println!("{:?}", self.msg);
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
pub struct SampleRate {
    sample_rate: Float,
}

impl Processor for SampleRate {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("sample_rate", SignalType::Float)]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for sample_rate in outputs.iter_output_mut_as_floats(0)? {
            *sample_rate = Some(self.sample_rate);
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// Adds a new [`SampleRate`] processor that continuously outputs the current sample rate.
    pub fn sample_rate(&self) -> Node {
        self.add(SampleRate::default())
    }
}

/// A processor that smooths a signal using linear interpolation.
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
pub struct Smooth {
    current: Float,
    factor: Float,
}

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
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (target, factor, out) in itertools::izip!(
            inputs.iter_input_as_floats(0)?,
            inputs.iter_input_as_floats(1)?,
            outputs.iter_output_mut_as_floats(0)?
        ) {
            self.factor = factor.unwrap_or(self.factor).clamp(0.0, 1.0);

            let Some(target) = target else {
                *out = Some(self.current);
                continue;
            };

            self.current = lerp(self.current, target, self.factor);

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
pub struct Changed {
    last: Float,
    threshold: Float,
}

impl Changed {
    /// Create a new `Changed` processor with the given threshold.
    pub fn new(threshold: Float) -> Self {
        Self {
            last: 0.0,
            threshold,
        }
    }
}

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
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, threshold, out_signal) in itertools::izip!(
            inputs.iter_input_as_floats(0)?,
            inputs.iter_input_as_floats(1)?,
            outputs.iter_output_mut_as_bools(0)?
        ) {
            let Some(in_signal) = in_signal else {
                *out_signal = None;
                continue;
            };

            self.threshold = threshold.unwrap_or(self.threshold);

            if (self.last - in_signal).abs() > self.threshold {
                *out_signal = Some(true);
            } else {
                *out_signal = None;
            }

            self.last = in_signal;
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
pub struct ZeroCrossing {
    last: Float,
}

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
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, out_signal) in itertools::izip!(
            inputs.iter_input_as_floats(0)?,
            outputs.iter_output_mut_as_bools(0)?
        ) {
            let Some(in_signal) = in_signal else {
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
pub struct SignalTx<S: Signal + Clone> {
    tx: Sender<S>,
}

impl<S: Signal + Clone> SignalTx<S> {
    pub(crate) fn new(tx: Sender<S>) -> Self {
        Self { tx }
    }

    /// Sends a message to the receiver.
    pub fn send(&self, message: S) {
        self.tx.try_send(message).ok();
    }
}

impl<S: Signal + Clone> Processor for SignalTx<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", S::TYPE)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        _outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs.iter_input_as::<S>(0)?;

        for message in in_signal.flatten() {
            self.send(message.clone());
        }

        Ok(())
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
pub struct SignalRx<S: Signal + Clone> {
    rx: Receiver<S>,
}

impl<S: Signal + Clone> SignalRx<S> {
    pub(crate) fn new(rx: Receiver<S>) -> Self {
        Self { rx }
    }

    /// Receives a message from the transmitter.
    pub fn recv(&mut self) -> Option<S> {
        self.rx.try_recv().ok()
    }
}

impl<S: Signal + Clone> Processor for SignalRx<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::TYPE)]
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let out = outputs.iter_output_as::<S>(0)?;

        for out in out {
            *out = self.recv();
        }

        Ok(())
    }
}

/// A wrapper around a [`SignalRx`] receiver that stores the last received message. Used as part of a [`Param`] processor.
#[derive(Clone, Debug)]
pub struct ParamRx<S: Signal + Clone> {
    rx: SignalRx<S>,
    last: Arc<Mutex<Option<S>>>,
}

impl<S: Signal + Clone> ParamRx<S> {
    pub(crate) fn new(rx: SignalRx<S>) -> Self {
        Self {
            rx,
            last: Arc::new(Mutex::new(None)),
        }
    }

    /// Receives a message from the transmitter and stores it as the last message.
    pub fn recv(&mut self) -> Option<S> {
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
            // last.clone()
        }
    }

    /// Returns the last received message.
    pub fn last(&self) -> Option<S> {
        self.last.try_lock().ok()?.clone()
    }
}

/// Creates a new set of connected [`SignalTx`] and [`SignalRx`] transmitters and receivers.
pub fn signal_channel<S: Signal + Clone>() -> (SignalTx<S>, SignalRx<S>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (SignalTx::new(tx), SignalRx::new(rx))
}

pub(crate) fn param_channel<S: Signal + Clone>() -> (SignalTx<S>, ParamRx<S>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (SignalTx::new(tx), ParamRx::new(SignalRx::new(rx)))
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
pub struct Param<S: Signal + Clone> {
    name: String,
    channels: (SignalTx<S>, ParamRx<S>),
}

impl<S: Signal + Clone> Param<S> {
    /// Creates a new `Param` processor with the given name and optional initial value.
    pub fn new(name: impl Into<String>, initial_value: impl Into<Option<S>>) -> Self {
        let this = Self {
            name: name.into(),
            channels: param_channel(),
        };
        let initial_value = initial_value.into();
        if let Some(initial_value) = initial_value {
            this.send(initial_value);
        }
        this
    }

    /// Returns the name of the parameter.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the transmitter for the parameter.
    pub fn tx(&self) -> &SignalTx<S> {
        &self.channels.0
    }

    /// Returns the receiver for the parameter.
    pub fn rx(&self) -> &ParamRx<S> {
        &self.channels.1
    }

    /// Returns a mutable reference to the receiver for the parameter.
    pub fn rx_mut(&mut self) -> &mut ParamRx<S> {
        &mut self.channels.1
    }

    /// Sends a value to the parameter.
    pub fn send(&self, message: impl Into<S>) {
        let message = message.into();
        self.tx().send(message);
    }

    /// Receives the value of the parameter.
    pub fn recv(&mut self) -> Option<S> {
        self.rx_mut().recv()
    }

    /// Returns the last received value of the parameter.
    pub fn last(&self) -> Option<S> {
        self.rx().last()
    }
}

impl<S: Signal + Clone> Processor for Param<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("set", S::TYPE)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("get", S::TYPE)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (set, get) in itertools::izip!(
            inputs.iter_input_as::<S>(0)?,
            outputs.iter_output_as::<S>(0)?
        ) {
            if let Some(set) = set {
                self.send(set.clone());
            }

            *get = self.recv();
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
pub struct Counter {
    count: i64,
}

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
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (trig, reset, count) in itertools::izip!(
            inputs.iter_input_as_bools(0)?,
            inputs.iter_input_as_bools(1)?,
            outputs.iter_output_mut_as_ints(0)?
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
pub struct SampleAndHold {
    last: Option<Float>,
}

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
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, trig, out_signal) in itertools::izip!(
            inputs.iter_input_as_floats(0)?,
            inputs.iter_input_as_bools(1)?,
            outputs.iter_output_mut_as_floats(0)?
        ) {
            if let Some(true) = trig {
                self.last = in_signal;
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
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs.iter_input_as_floats(0)?;
        let out_signal = outputs.iter_output_mut_as_floats(0)?;
        for (in_signal, out_signal) in in_signal.zip(out_signal) {
            if let Some(in_signal) = in_signal {
                if in_signal.is_nan() {
                    panic!("{}: signal is NaN: {:?}", self.context, in_signal);
                }
                if in_signal.is_infinite() {
                    panic!("{}: signal is infinite: {:?}", self.context, in_signal);
                }
            }

            *out_signal = in_signal;
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
pub struct FiniteOrZero;

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
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs.iter_input_as_floats(0)?;
        let out_signal = outputs.iter_output_mut_as_floats(0)?;
        for (in_signal, out_signal) in in_signal.zip(out_signal) {
            if let Some(in_signal) = in_signal {
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
/// This can be thought of as the opposite of the [`Register`](crate::builtins::storage::Register) processor, and will effectively undo its effect.
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
pub struct Dedup<S: Signal + Clone> {
    last: Option<S>,
}

impl<S: Signal + Clone> Dedup<S> {
    /// Create a new `Dedup` processor.
    pub fn new() -> Self {
        Self { last: None }
    }
}

impl<S: Signal + Clone> Default for Dedup<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Signal + Clone> Processor for Dedup<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", S::TYPE)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::TYPE)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, out_signal) in itertools::izip!(
            inputs.iter_input_as::<S>(0)?,
            outputs.iter_output_as::<S>(0)?
        ) {
            if let Some(in_signal) = in_signal {
                if self.last.as_ref() != Some(in_signal) {
                    *out_signal = Some(in_signal.clone());
                    self.last = Some(in_signal.clone());
                } else {
                    *out_signal = None;
                }
            } else {
                *out_signal = None;
            }
        }

        Ok(())
    }
}
