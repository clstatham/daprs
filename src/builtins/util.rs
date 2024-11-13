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














#[derive(Clone, Debug, Default)]
pub struct Passthrough<S: Signal>(PhantomData<S>);

impl<S: Signal> Passthrough<S> {
    
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: Signal> Processor for Passthrough<S> {
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
        let Some(in_signal) = inputs.inputs[0] else {
            return Ok(());
        };

        let out_signal = outputs.output(0);

        out_signal.copy_from(in_signal);

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Cast<S: Signal, T: Signal> {
    _phantom: PhantomData<(S, T)>,
}

impl<S: Signal, T: Signal> Cast<S, T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<S: Signal, T: Signal> Processor for Cast<S, T> {
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
        let Some(in_signal) = inputs.inputs[0] else {
            return Ok(());
        };

        let in_signal = in_signal
            .as_kind::<S>()
            .ok_or(ProcessorError::InputSpecMismatch {
                index: 0,
                expected: S::TYPE,
                actual: in_signal.type_(),
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















#[derive(Clone, Debug)]
pub struct MessageSender<S: Signal> {
    message: S,
}

impl<S: Signal> MessageSender<S> {
    
    pub fn new(message: S) -> Self {
        Self { message }
    }
}

impl<S: Signal> Processor for MessageSender<S> {
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









#[derive(Clone, Debug, Default)]
pub struct Print {
    name: Option<String>,
    msg: Option<String>,
}

impl Print {
    
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

impl Processor for Print {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("message", SignalType::String),
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
    
    
    
    pub fn print<'a>(
        &self,
        name: impl Into<Option<&'a str>>,
        msg: impl Into<Option<&'a str>>,
    ) -> Node {
        self.add(Print::new(name.into(), msg.into()))
    }
}








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
        let sample_rate_out = outputs.output_as_floats(0)?;
        sample_rate_out.fill(Some(self.sample_rate));

        Ok(())
    }
}

impl GraphBuilder {
    
    
    
    pub fn sample_rate(&self) -> Node {
        self.add(SampleRate::default())
    }
}

#[inline(always)]
fn lerp(a: Float, b: Float, t: Float) -> Float {
    a + (b - a) * t
}















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















#[derive(Clone, Debug, Default)]
pub struct Changed {
    last: Float,
    threshold: Float,
}

impl Changed {
    
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


#[derive(Clone, Debug)]
pub struct SignalTx<S: Signal> {
    tx: Sender<S>,
}

impl<S: Signal> SignalTx<S> {
    pub(crate) fn new(tx: Sender<S>) -> Self {
        Self { tx }
    }

    
    pub fn send(&self, message: S) {
        self.tx.try_send(message).unwrap();
    }
}

impl<S: Signal> Processor for SignalTx<S> {
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


#[derive(Clone, Debug)]
pub struct SignalRx<S: Signal> {
    rx: Receiver<S>,
}

impl<S: Signal> SignalRx<S> {
    pub(crate) fn new(rx: Receiver<S>) -> Self {
        Self { rx }
    }

    
    pub fn recv(&mut self) -> Option<S> {
        self.rx.try_recv().ok()
    }
}

impl<S: Signal> Processor for SignalRx<S> {
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


#[derive(Clone, Debug)]
pub struct ParamRx<S: Signal> {
    rx: SignalRx<S>,
    last: Arc<Mutex<Option<S>>>,
}

impl<S: Signal> ParamRx<S> {
    pub(crate) fn new(rx: SignalRx<S>) -> Self {
        Self {
            rx,
            last: Arc::new(Mutex::new(None)),
        }
    }

    
    pub fn recv(&mut self) -> Option<S> {
        let mut last = self.last.try_lock().ok()?;
        if let Some(msg) = self.rx.recv() {
            *last = Some(msg.clone());
            Some(msg)
        } else {
            None
            // last.clone()
        }
    }

    
    pub fn last(&self) -> Option<S> {
        self.last.try_lock().ok()?.clone()
    }
}

pub(crate) fn param_channel<S: Signal>() -> (SignalTx<S>, ParamRx<S>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (SignalTx::new(tx), ParamRx::new(SignalRx::new(rx)))
}














#[derive(Clone, Debug)]
pub struct Param<S: Signal> {
    name: String,
    channels: (SignalTx<S>, ParamRx<S>),
}

impl<S: Signal> Param<S> {
    
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

    
    pub fn name(&self) -> &str {
        &self.name
    }

    
    pub fn tx(&self) -> &SignalTx<S> {
        &self.channels.0
    }

    pub fn rx(&self) -> &ParamRx<S> {
        &self.channels.1
    }

    
    pub fn rx_mut(&mut self) -> &mut ParamRx<S> {
        &mut self.channels.1
    }

    
    pub fn set(&self, message: impl Into<S>) {
        let message = message.into();
        self.tx().send(message);
    }

    
    pub fn get(&mut self) -> Option<S> {
        self.rx_mut().recv()
    }

    
    pub fn last(&self) -> Option<S> {
        self.rx().last()
    }
}

impl<S: Signal> Processor for Param<S> {
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
                self.set(set.clone());
            }

            *get = self.get();
        }

        Ok(())
    }
}



















#[derive(Clone, Debug)]
pub struct Select<S: Signal> {
    num_outputs: usize,
    last_index: i64,
    _phantom: PhantomData<S>,
}

impl<S: Signal> Select<S> {
    
    pub fn new(num_outputs: usize) -> Self {
        Self {
            last_index: 0,
            num_outputs,
            _phantom: PhantomData,
        }
    }
}

impl<S: Signal> Default for Select<S> {
    fn default() -> Self {
        Self::new(2)
    }
}

impl<S: Signal> Processor for Select<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", S::TYPE),
            SignalSpec::new("index", SignalType::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::new(format!("{}", i), S::TYPE))
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




















#[derive(Clone, Debug)]
pub struct Merge<S: Signal> {
    num_inputs: usize,
    _phantom: PhantomData<S>,
}

impl<S: Signal> Merge<S> {
    
    pub fn new(num_inputs: usize) -> Self {
        Self {
            num_inputs,
            _phantom: PhantomData,
        }
    }
}

impl<S: Signal> Default for Merge<S> {
    fn default() -> Self {
        Self::new(2)
    }
}

impl<S: Signal> Processor for Merge<S> {
    fn input_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_inputs)
            .map(|i| SignalSpec::new(i.to_string(), S::TYPE))
            .collect()
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", S::TYPE)]
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
                    expected: S::TYPE,
                    actual: input.type_(),
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









#[derive(Clone, Debug, Default)]
pub struct CheckFinite {
    context: String,
}

impl CheckFinite {
    
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
        vec![]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        _outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs.iter_input_as_floats(0)?;
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
