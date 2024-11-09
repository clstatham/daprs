//! Built-in filters for processing audio signals.

use std::f64::consts::PI;

use crate::prelude::*;

const THERMAL: f64 = 0.000025;

/// A 4-pole low-pass filter based on the Moog ladder filter.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `in` | `Sample` | | The input signal to filter. |
/// | `1` | `cutoff` | `Sample` | `1000.0` | The cutoff frequency of the filter. |
/// | `2` | `resonance` | `Sample` | `0.1` | The resonance of the filter. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The filtered output signal. |
#[derive(Clone, Debug)]
pub struct MoogLadder {
    sample_rate: f64,
    stage: [f64; 4],
    stage_tanh: [f64; 3],
    delay: [f64; 6],
    tune: f64,
    acr: f64,
    res_quad: f64,

    /// The cutoff frequency of the filter.
    pub cutoff: f64,
    /// The resonance of the filter.
    pub resonance: f64,
}

impl Default for MoogLadder {
    fn default() -> Self {
        Self {
            sample_rate: 0.0,
            stage: [0.0; 4],
            stage_tanh: [0.0; 3],
            delay: [0.0; 6],
            tune: 0.0,
            acr: 0.0,
            res_quad: 0.0,
            cutoff: 1000.0,
            resonance: 0.1,
        }
    }
}

impl MoogLadder {
    /// Creates a new Moog ladder filter with the given cutoff frequency and resonance.
    pub fn new(cutoff: f64, resonance: f64) -> Self {
        Self {
            cutoff,
            resonance,
            ..Default::default()
        }
    }
}

impl Processor for MoogLadder {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("in", 0.0),
            SignalSpec::unbounded("cutoff", self.cutoff),
            SignalSpec::unbounded("resonance", self.resonance),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        // based on: https://github.com/ddiakopoulos/MoogLadders/blob/fd147415573e723ba102dfc63dc46af0b7fe55b9/src/HuovilainenModel.h
        for (out, in_signal, cutoff, resonance) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
            inputs.iter_input_as_samples(2)?
        ) {
            let cutoff = cutoff.clamp(0.0, self.sample_rate * 0.5);
            let resonance = resonance.clamp(0.0, 1.0);
            self.cutoff = cutoff;
            self.resonance = resonance;

            let fc = cutoff / self.sample_rate;
            let f = fc * 0.5; // oversampling
            let fc2 = fc * fc;
            let fc3 = fc2 * fc;

            let fcr = 1.8730 * fc3 + 0.4955 * fc2 - 0.6490 * fc + 0.9988;
            self.acr = -3.9364 * fc2 + 1.8409 * fc + 0.9968;
            self.tune = (1.0 - f64::exp(-((2.0 * PI) * f * fcr))) / THERMAL;
            self.res_quad = 4.0 * self.resonance * self.acr;

            // oversample
            for _ in 0..2 {
                let mut inp = in_signal - self.res_quad * self.delay[5];
                self.stage[0] =
                    self.delay[0] + self.tune * (f64::tanh(inp * THERMAL) - self.stage_tanh[0]);
                self.delay[0] = self.stage[0];
                for k in 1..4 {
                    inp = self.stage[k - 1];
                    self.stage_tanh[k - 1] = f64::tanh(inp * THERMAL);
                    if k == 3 {
                        self.stage[k] = self.delay[k]
                            + self.tune
                                * (self.stage_tanh[k - 1] - f64::tanh(self.delay[k] * THERMAL));
                    } else {
                        self.stage[k] = self.delay[k]
                            + self.tune * (self.stage_tanh[k - 1] - self.stage_tanh[k]);
                    }
                    self.delay[k] = self.stage[k];
                }
                self.delay[5] = (self.stage[3] + self.delay[4]) * 0.5;
                self.delay[4] = self.stage[3];
            }

            *out = self.delay[5];
        }

        Ok(())
    }
}

/// A 2-pole, 2-zero biquad filter.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `in` | `Sample` | | The input signal to filter. |
/// | `1` | `a0` | `Sample` | `1.0` | The a0 coefficient: amount of input signal that contributes to the output. |
/// | `2` | `a1` | `Sample` | `0.0` | The a1 coefficient: amount of input signal delayed by 1 sample that contributes to the output. |
/// | `3` | `a2` | `Sample` | `0.0` | The a2 coefficient: amount of input signal delayed by 2 samples that contributes to the output. |
/// | `4` | `b1` | `Sample` | `0.0` | The b1 coefficient: amount of output signal delayed by 1 sample that contributes to the output. |
/// | `5` | `b2` | `Sample` | `0.0` | The b2 coefficient: amount of output signal delayed by 2 samples that contributes to the output. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The filtered output signal. |
#[derive(Clone, Debug)]
pub struct Biquad {
    sample_rate: f64,
    /// The a0 coefficient: amount of input signal that contributes to the output.
    pub a0: f64,
    /// The a1 coefficient: amount of input signal delayed by 1 sample that contributes to the output.
    pub a1: f64,
    /// The a2 coefficient: amount of input signal delayed by 2 samples that contributes to the output.
    pub a2: f64,
    /// The b1 coefficient: amount of output signal delayed by 1 sample that contributes to the output.
    pub b1: f64,
    /// The b2 coefficient: amount of output signal delayed by 2 samples that contributes to the output.
    pub b2: f64,

    // input state
    x1: f64,
    x2: f64,

    // output state
    y1: f64,
    y2: f64,
}

impl Default for Biquad {
    fn default() -> Self {
        Self {
            sample_rate: 0.0,
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            b1: 0.0,
            b2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
}

impl Biquad {
    /// Creates a new biquad filter with the given coefficients.
    pub fn new(a0: f64, a1: f64, a2: f64, b1: f64, b2: f64) -> Self {
        Self {
            a0,
            a1,
            a2,
            b1,
            b2,
            ..Default::default()
        }
    }
}

impl Processor for Biquad {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("in", 0.0),
            SignalSpec::unbounded("a0", self.a0),
            SignalSpec::unbounded("a1", self.a1),
            SignalSpec::unbounded("a2", self.a2),
            SignalSpec::unbounded("b1", self.b1),
            SignalSpec::unbounded("b2", self.b2),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, in_signal, a0, a1, a2, b1, b2) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
            inputs.iter_input_as_samples(2)?,
            inputs.iter_input_as_samples(3)?,
            inputs.iter_input_as_samples(4)?,
            inputs.iter_input_as_samples(5)?
        ) {
            self.a0 = a0;
            self.a1 = a1;
            self.a2 = a2;
            self.b1 = b1;
            self.b2 = b2;

            let filtered = self.a0 * in_signal + self.a1 * self.x1 + self.a2 * self.x2
                - self.b1 * self.y1
                - self.b2 * self.y2;

            self.x2 = self.x1;
            self.x1 = in_signal;
            self.y2 = self.y1;
            self.y1 = filtered;

            *out = filtered;
        }

        Ok(())
    }
}

/// The type of biquad filter to use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BiquadType {
    /// A low-pass filter.
    LowPass,
    /// A high-pass filter.
    HighPass,
    /// A band-pass filter.
    BandPass,
    /// A notch filter.
    Notch,
    /// A peak filter.
    Peak,
    /// A low-shelf filter.
    LowShelf,
    /// A high-shelf filter.
    HighShelf,
}

impl TryFrom<&str> for BiquadType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "lowpass" => Ok(Self::LowPass),
            "highpass" => Ok(Self::HighPass),
            "bandpass" => Ok(Self::BandPass),
            "notch" => Ok(Self::Notch),
            "peak" => Ok(Self::Peak),
            "lowshelf" => Ok(Self::LowShelf),
            "highshelf" => Ok(Self::HighShelf),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for BiquadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiquadType::LowPass => write!(f, "lowpass"),
            BiquadType::HighPass => write!(f, "highpass"),
            BiquadType::BandPass => write!(f, "bandpass"),
            BiquadType::Notch => write!(f, "notch"),
            BiquadType::Peak => write!(f, "peak"),
            BiquadType::LowShelf => write!(f, "lowshelf"),
            BiquadType::HighShelf => write!(f, "highshelf"),
        }
    }
}

/// A 2-pole, 2-zero biquad filter with automatic coefficient calculation based on the given filter type.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `in` | `Sample` | | The input signal to filter. |
/// | `1` | `frequency` | `Sample` | `1000.0` | The cutoff frequency of the filter. |
/// | `2` | `q` | `Sample` | `0.707` | The quality factor of the filter. |
/// | `3` | `gain` | `Sample` | `0.0` | The gain of the filter. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The filtered output signal. |
#[derive(Clone, Debug)]
pub struct AutoBiquad {
    sample_rate: f64,

    // biquad state
    a0: f64,
    a1: f64,
    a2: f64,
    b1: f64,
    b2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,

    // the type of biquad filter
    biquad_type: BiquadType,

    /// The cutoff frequency of the filter.
    pub cutoff: f64,
    /// The Q/resonance factor of the filter.
    pub q: f64,
    /// The gain of the filter.
    pub gain: f64,
}

impl Default for AutoBiquad {
    fn default() -> Self {
        Self {
            sample_rate: 0.0,
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            b1: 0.0,
            b2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            biquad_type: BiquadType::LowPass,
            cutoff: 1000.0,
            q: 0.707,
            gain: 0.0,
        }
    }
}

impl AutoBiquad {
    /// Creates a new auto biquad filter with the given type, frequency, Q, and gain.
    pub fn new(biquad_type: BiquadType, cutoff: f64, q: f64, gain: f64) -> Self {
        let mut this = Self {
            biquad_type,
            cutoff,
            q,
            gain,
            ..Default::default()
        };
        this.set_coefficients();
        this
    }

    /// Returns the type of biquad filter used.
    pub fn biquad_type(&self) -> BiquadType {
        self.biquad_type
    }

    /// Creates a new low-pass biquad filter with the given frequency and Q.
    pub fn lowpass(cutoff: f64, q: f64) -> Self {
        let mut this = Self::new(BiquadType::LowPass, cutoff, q, 0.0);
        this.set_coefficients();
        this
    }

    /// Creates a new high-pass biquad filter with the given frequency and Q.
    pub fn highpass(cutoff: f64, q: f64) -> Self {
        let mut this = Self::new(BiquadType::HighPass, cutoff, q, 0.0);
        this.set_coefficients();
        this
    }

    /// Creates a new band-pass biquad filter with the given frequency and Q.
    pub fn bandpass(cutoff: f64, q: f64) -> Self {
        let mut this = Self::new(BiquadType::BandPass, cutoff, q, 0.0);
        this.set_coefficients();
        this
    }

    /// Creates a new notch biquad filter with the given frequency and Q.
    pub fn notch(cutoff: f64, q: f64) -> Self {
        let mut this = Self::new(BiquadType::Notch, cutoff, q, 0.0);
        this.set_coefficients();
        this
    }

    /// Creates a new peak biquad filter with the given frequency, Q, and gain.
    pub fn peak(cutoff: f64, q: f64, gain: f64) -> Self {
        let mut this = Self::new(BiquadType::Peak, cutoff, q, gain);
        this.set_coefficients();
        this
    }

    /// Creates a new low-shelf biquad filter with the given frequency, Q, and gain.
    pub fn lowshelf(cutoff: f64, q: f64, gain: f64) -> Self {
        let mut this = Self::new(BiquadType::LowShelf, cutoff, q, gain);
        this.set_coefficients();
        this
    }

    /// Creates a new high-shelf biquad filter with the given frequency, Q, and gain.
    pub fn highshelf(cutoff: f64, q: f64, gain: f64) -> Self {
        let mut this = Self::new(BiquadType::HighShelf, cutoff, q, gain);
        this.set_coefficients();
        this
    }

    // http://www.earlevel.com/scripts/widgets/20131013/biquads2.js
    #[inline]
    fn set_coefficients(&mut self) {
        if self.q < 0.01 {
            self.q = 0.01;
        }

        let v = 10.0_f64.powf(self.gain.abs() / 20.0);
        let k = f64::tan(PI * self.cutoff / self.sample_rate);

        match self.biquad_type {
            BiquadType::LowPass => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = k * k * norm;
                self.a1 = 2.0 * self.a0;
                self.a2 = self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            BiquadType::HighPass => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = 1.0 * norm;
                self.a1 = -2.0 * self.a0;
                self.a2 = self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            BiquadType::BandPass => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = k / self.q * norm;
                self.a1 = 0.0;
                self.a2 = -self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            BiquadType::Notch => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = (1.0 + k * k) * norm;
                self.a1 = 2.0 * (k * k - 1.0) * norm;
                self.a2 = self.a0;
                self.b1 = self.a1;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            BiquadType::Peak => {
                if self.gain >= 0.0 {
                    let norm = 1.0 / (1.0 + 1.0 / self.q * k + k * k);
                    self.a0 = (1.0 + v / self.q * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - v / self.q * k + k * k) * norm;
                    self.b1 = self.a1;
                    self.b2 = (1.0 - 1.0 / self.q * k + k * k) * norm;
                } else {
                    let norm = 1.0 / (1.0 + v / self.q * k + k * k);
                    self.a0 = (1.0 + 1.0 / self.q * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - 1.0 / self.q * k + k * k) * norm;
                    self.b1 = self.a1;
                    self.b2 = (1.0 - v / self.q * k + k * k) * norm;
                }
            }
            BiquadType::LowShelf => {
                if self.gain >= 0.0 {
                    let norm = 1.0 / (1.0 + f64::sqrt(2.0) * k + k * k);
                    self.a0 = (1.0 + f64::sqrt(2.0 * v) * k + v * k * k) * norm;
                    self.a1 = 2.0 * (v * k * k - 1.0) * norm;
                    self.a2 = (1.0 - f64::sqrt(2.0 * v) * k + v * k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - f64::sqrt(2.0) * k + k * k) * norm;
                } else {
                    let norm = 1.0 / (1.0 + f64::sqrt(2.0) * k + k * k);
                    self.a0 = (v + f64::sqrt(2.0 * v) * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - v) * norm;
                    self.a2 = (v - f64::sqrt(2.0 * v) * k + k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - f64::sqrt(2.0) * k + k * k) * norm;
                }
            }
            BiquadType::HighShelf => {
                if self.gain >= 0.0 {
                    let norm = 1.0 / (1.0 + f64::sqrt(2.0) * k + k * k);
                    self.a0 = (v + f64::sqrt(2.0 * v) * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - v) * norm;
                    self.a2 = (v - f64::sqrt(2.0 * v) * k + k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - f64::sqrt(2.0) * k + k * k) * norm;
                } else {
                    let norm = 1.0 / (v + f64::sqrt(2.0 * v) * k + k * k);
                    self.a0 = (1.0 + f64::sqrt(2.0) * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - f64::sqrt(2.0) * k + k * k) * norm;
                    self.b1 = 2.0 * (v * k * k - 1.0) * norm;
                    self.b2 = (v - f64::sqrt(2.0 * v) * k + v * k * k) * norm;
                }
            }
        }

        // #[cfg(debug_assertions)]
        if self.sample_rate > 0.0 {
            // check for NaN
            assert!(self.a0.is_finite());
            assert!(self.a1.is_finite());
            assert!(self.a2.is_finite());
            assert!(self.b1.is_finite());
            assert!(self.b2.is_finite());
        }
    }
}

impl Processor for AutoBiquad {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("in", 0.0),
            SignalSpec::unbounded("frequency", self.cutoff),
            SignalSpec::unbounded("q", self.q),
            SignalSpec::unbounded("gain", self.gain),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
        self.set_coefficients();
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, in_signal, frequency, q, gain) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
            inputs.iter_input_as_samples(2)?,
            inputs.iter_input_as_samples(3)?
        ) {
            let frequency_changed = (frequency - self.cutoff).abs() > f64::EPSILON;
            let q_changed = (q - self.q).abs() > f64::EPSILON;
            let gain_changed = (gain - self.gain).abs() > f64::EPSILON;

            if frequency_changed || q_changed || gain_changed {
                self.cutoff = frequency;
                self.q = q;
                self.gain = gain;

                self.set_coefficients();
            }

            let filtered = self.a0 * in_signal + self.a1 * self.x1 + self.a2 * self.x2
                - self.b1 * self.y1
                - self.b2 * self.y2;

            self.x2 = self.x1;
            self.x1 = in_signal;
            self.y2 = self.y1;
            self.y1 = filtered;

            *out = filtered;
        }

        Ok(())
    }
}
