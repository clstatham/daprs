//! Utility processors.

use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crossbeam_channel::{Receiver, Sender};

use crate::{
    prelude::{GraphBuilder, Node, Processor, ProcessorInputs, ProcessorOutputs, SignalSpec},
    processor::ProcessorError,
    signal::{Sample, SignalData, SignalKind},
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
#[derive(Clone, Debug, Default)]
pub struct Passthrough<S: SignalData>(PhantomData<S>);

impl<S: SignalData> Passthrough<S> {
    /// Creates a new `Passthrough`.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: SignalData> Processor for Passthrough<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", S::KIND)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::KIND)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let Some(in_signal) = inputs.inputs[0] else {
            return Ok(());
        };

        let out_signal = outputs.output(0);

        out_signal.copy_from(in_signal);

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Cast<S: SignalData, T: SignalData> {
    _phantom: PhantomData<(S, T)>,
}

impl<S: SignalData, T: SignalData> Cast<S, T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<S: SignalData, T: SignalData> Processor for Cast<S, T> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", S::KIND)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", T::KIND)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let Some(in_signal) = inputs.inputs[0] else {
            return Ok(());
        };

        let in_signal = in_signal
            .as_kind::<S>()
            .ok_or(ProcessorError::InputSpecMismatch {
                index: 0,
                expected: S::KIND,
                actual: in_signal.kind(),
            })?;

        let out_signal = outputs.output(0).as_kind_mut::<T>().unwrap();

        for (in_signal, out_signal) in itertools::izip!(in_signal, out_signal) {
            if let Some(in_signal) = in_signal {
                let in_signal = S::into_signal(in_signal.to_owned());
                *out_signal = in_signal.cast();
            }
        }

        Ok(())
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
pub struct MessageSender<S: SignalData> {
    message: S,
}

impl<S: SignalData> MessageSender<S> {
    /// Creates a new `MessageProc` with the given initial message.
    pub fn new(message: S) -> Self {
        Self { message }
    }
}

impl<S: SignalData> Processor for MessageSender<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalKind::Bool),
            SignalSpec::new("message", S::KIND),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::KIND)]
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
                self.message = message.clone();
            }

            if let Some(true) = bang {
                *out = Some(self.message.clone());
            } else {
                *out = None;
            }
        }

        Ok(())
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
            SignalSpec::new("trig", SignalKind::Bool),
            SignalSpec::new("message", SignalKind::String),
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
            inputs.iter_input_as_strings(1)?
        ) {
            if let Some(message) = message {
                self.msg = Some(message.to_string());
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
        vec![SignalSpec::new("sample_rate", SignalKind::Sample)]
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
        sample_rate_out.fill(Some(self.sample_rate));

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
    factor: Sample,
}

impl Processor for Smooth {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("target", SignalKind::Sample),
            SignalSpec::new("factor", SignalKind::Sample),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Sample)]
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
    threshold: Sample,
}

impl Processor for Changed {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalKind::Sample),
            SignalSpec::new("threshold", SignalKind::Sample),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, threshold, out_signal) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
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
        vec![SignalSpec::new("in", SignalKind::Sample)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, out_signal) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
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

/// A signal sender, used for `Param` communication and breaking cycles in the graph.
#[derive(Clone, Debug)]
pub struct SignalTx<S: SignalData> {
    tx: Sender<S>,
}

impl<S: SignalData> SignalTx<S> {
    pub(crate) fn new(tx: Sender<S>) -> Self {
        Self { tx }
    }

    /// Sends a message to the `Param`.
    pub fn send(&self, message: S) {
        self.tx.try_send(message).unwrap();
    }
}

impl<S: SignalData> Processor for SignalTx<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", S::KIND)]
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

/// A message receiver, used for `Param` communication and breaking cycles in the graph.
#[derive(Clone, Debug)]
pub struct SignalRx<S: SignalData> {
    rx: Receiver<S>,
}

impl<S: SignalData> SignalRx<S> {
    pub(crate) fn new(rx: Receiver<S>) -> Self {
        Self { rx }
    }

    /// Receives a message from the `Param`.
    pub fn recv(&mut self) -> Option<S> {
        self.rx.try_recv().ok()
    }
}

impl<S: SignalData> Processor for SignalRx<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::KIND)]
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

/// A receiver for a `Param`.
#[derive(Clone, Debug)]
pub struct ParamRx<S: SignalData> {
    rx: SignalRx<S>,
    last: Arc<Mutex<Option<S>>>,
}

impl<S: SignalData> ParamRx<S> {
    pub(crate) fn new(rx: SignalRx<S>) -> Self {
        Self {
            rx,
            last: Arc::new(Mutex::new(None)),
        }
    }

    /// Receives a message from the `Param`.
    pub fn recv(&mut self) -> Option<S> {
        let mut last = self.last.try_lock().ok()?;
        if let Some(msg) = self.rx.recv() {
            *last = Some(msg.clone());
            Some(msg)
        } else {
            last.clone()
        }
    }
}

pub(crate) fn param_channel<S: SignalData>() -> (SignalTx<S>, ParamRx<S>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (SignalTx::new(tx), ParamRx::new(SignalRx::new(rx)))
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
pub struct Param<S: SignalData> {
    name: String,
    channels: (SignalTx<S>, ParamRx<S>),
}

impl<S: SignalData> Param<S> {
    /// Creates a new `Param`.
    pub fn new(name: impl Into<String>, initial_value: impl Into<Option<S>>) -> Self {
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
    pub fn tx(&self) -> &SignalTx<S> {
        &self.channels.0
    }

    /// Returns the receiver for this `Param`.
    pub fn rx_mut(&mut self) -> &mut ParamRx<S> {
        &mut self.channels.1
    }

    /// Sets the `Param`'s value.
    pub fn set(&self, message: impl Into<S>) {
        let message = message.into();
        self.tx().send(message);
    }

    /// Gets the `Param`'s value.
    pub fn get(&mut self) -> Option<S> {
        self.rx_mut().recv()
    }
}

impl<S: SignalData> Processor for Param<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("set", S::KIND)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("get", S::KIND)]
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
                self.set(set.clone());
            }

            *get = self.get();
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
pub struct Select<S: SignalData> {
    num_outputs: usize,
    last_index: i64,
    _phantom: PhantomData<S>,
}

impl<S: SignalData> Select<S> {
    /// Creates a new `Select` with the given number of outputs.
    pub fn new(num_outputs: usize) -> Self {
        Self {
            last_index: 0,
            num_outputs,
            _phantom: PhantomData,
        }
    }
}

impl<S: SignalData> Default for Select<S> {
    fn default() -> Self {
        Self::new(2)
    }
}

impl<S: SignalData> Processor for Select<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", S::KIND),
            SignalSpec::new("index", SignalKind::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::new(format!("{}", i), S::KIND))
            .collect()
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (sample_index, (in_signal, index)) in
            itertools::izip!(inputs.iter_input_as::<S>(0)?, inputs.iter_input_as_ints(1)?)
                .enumerate()
        {
            let index = index.unwrap_or_default();

            self.last_index = index;

            if index >= 0 && index < self.num_outputs as i64 {
                let out_signal = outputs.output(index as usize).as_kind_mut::<S>().unwrap();

                out_signal[sample_index] = in_signal.clone();

                for (i, out_signal) in outputs.iter_mut().enumerate() {
                    if i != index as usize {
                        let out_signal = out_signal.as_kind_mut::<S>().unwrap();
                        out_signal[sample_index] = None;
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
pub struct Merge<S: SignalData> {
    num_inputs: usize,
    _phantom: PhantomData<S>,
}

impl<S: SignalData> Merge<S> {
    /// Creates a new `Merge` with the given number of inputs.
    pub fn new(num_inputs: usize) -> Self {
        Self {
            num_inputs,
            _phantom: PhantomData,
        }
    }
}

impl<S: SignalData> Default for Merge<S> {
    fn default() -> Self {
        Self::new(2)
    }
}

impl<S: SignalData> Processor for Merge<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_inputs)
            .map(|i| SignalSpec::new(i.to_string(), S::KIND))
            .collect()
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::KIND)]
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
                .as_kind::<S>()
                .ok_or(ProcessorError::InputSpecMismatch {
                    index: i,
                    expected: S::KIND,
                    actual: input.kind(),
                })?;

            let out_signal = outputs.iter_output_as::<S>(0)?;

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
            SignalSpec::new("trig", SignalKind::Bool),
            SignalSpec::new("reset", SignalKind::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("count", SignalKind::Int)]
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
            SignalSpec::new("in", SignalKind::Sample),
            SignalSpec::new("trig", SignalKind::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Sample)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, trig, out_signal) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_bools(1)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            if let Some(true) = trig {
                self.last = in_signal;
            }

            *out_signal = self.last;
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
        vec![SignalSpec::new("in", SignalKind::Sample)]
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
        for in_signal in in_signal.flatten() {
            if in_signal.is_nan() {
                panic!("{}: signal is NaN: {:?}", self.context, in_signal);
            }
            if in_signal.is_infinite() {
                panic!("{}: signal is infinite: {:?}", self.context, in_signal);
            }
        }

        Ok(())
    }
}
