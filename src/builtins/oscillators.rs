//! Oscillator processors.

use rand::prelude::Distribution;

use crate::prelude::*;

/// A phase accumulator.
///
/// The phase accumulator is a simple processor that generates a phase signal that increments linearly over time.
/// It can be used to drive oscillators, or to generate control signals.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `increment` | `Sample` | `0.0` | The phase increment per sample. |
/// | `1` | `reset` | `Message(Bang)` |  | A message to reset the phase accumulator. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The output phase signal. |
#[derive(Clone, Debug, Default)]
pub struct PhaseAccumulator {
    // phase accumulator
    t: f64,
    // phase increment per sample
    t_step: f64,
}

impl Process for PhaseAccumulator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("increment", 0.0),
            SignalSpec::unbounded("reset", Signal::new_message_none()),
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
        let increment = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let reset = inputs[1]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (out, increment, reset) in itertools::izip!(out, increment, reset) {
            if reset.is_some() {
                self.t = 0.0;
            }

            // output the phase accumulator value
            **out = self.t;

            // increment the phase accumulator
            self.t_step = **increment;
            self.t += self.t_step;
        }

        Ok(())
    }
}

/// A free-running sine wave oscillator.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `frequency` | `Sample` | `440.0` | The frequency of the sine wave in Hz. |
/// | `1` | `phase` | `Sample` | `0.0` | The phase of the sine wave in radians. |
/// | `2` | `reset` | `Message(Bang)` |  | A message to reset the oscillator phase. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The output sine wave signal. |
#[derive(Clone, Debug)]
pub struct SineOscillator {
    // phase accumulator
    t: f64,
    // phase increment per sample
    t_step: f64,
    // sample rate
    sample_rate: f64,

    /// The frequency of the sine wave in Hz.
    pub frequency: f64,
    /// The phase of the sine wave in radians.
    pub phase: f64,
}

impl SineOscillator {
    /// Creates a new sine wave oscillator.
    pub fn new(frequency: f64) -> Self {
        Self {
            frequency,
            ..Default::default()
        }
    }
}

impl Default for SineOscillator {
    fn default() -> Self {
        Self {
            t: 0.0,
            t_step: 0.0,
            sample_rate: 0.0,
            frequency: 440.0,
            phase: 0.0,
        }
    }
}

impl Process for SineOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("frequency", self.frequency),
            SignalSpec::unbounded("phase", self.phase),
            SignalSpec::unbounded("reset", Signal::new_message_none()),
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
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let frequency = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let phase = inputs[1]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let reset = inputs[2]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(2))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (out, frequency, phase, reset) in itertools::izip!(out, frequency, phase, reset) {
            if reset.is_some() {
                self.t = 0.0;
            }

            // calculate the sine wave using the phase accumulator
            let sine = (self.t * std::f64::consts::TAU + **phase).sin();
            **out = sine;

            // increment the phase accumulator
            self.t_step = **frequency / self.sample_rate;
            self.t += self.t_step;
        }

        Ok(())
    }
}

/// A free-running sawtooth wave oscillator.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `frequency` | `Sample` | `440.0` | The frequency of the sawtooth wave in Hz. |
/// | `1` | `phase` | `Sample` | `0.0` | The phase of the sawtooth wave in radians. |
/// | `2` | `reset` | `Message(Bang)` |  | A message to reset the oscillator phase. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The output sawtooth wave signal. |
#[derive(Clone, Debug)]
pub struct SawOscillator {
    // phase accumulator
    t: f64,
    // phase increment per sample
    t_step: f64,
    // sample rate
    sample_rate: f64,

    /// The frequency of the sawtooth wave in Hz.
    pub frequency: f64,
    /// The phase of the sawtooth wave in radians.
    pub phase: f64,
}

impl Default for SawOscillator {
    fn default() -> Self {
        Self {
            t: 0.0,
            t_step: 0.0,
            sample_rate: 0.0,
            frequency: 440.0,
            phase: 0.0,
        }
    }
}

impl SawOscillator {
    /// Creates a new sawtooth wave oscillator.
    pub fn new(frequency: f64) -> Self {
        Self {
            frequency,
            ..Default::default()
        }
    }
}

impl Process for SawOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("frequency", self.frequency),
            SignalSpec::unbounded("phase", self.phase),
            SignalSpec::unbounded("reset", Signal::new_message_none()),
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
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let frequency = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let phase = inputs[1]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let reset = inputs[2]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(2))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (out, frequency, phase, reset) in itertools::izip!(out, frequency, phase, reset) {
            if reset.is_some() {
                self.t = 0.0;
            }

            // calculate the sawtooth wave using the phase accumulator
            **out = (self.t + **phase) % 1.0;

            // increment the phase accumulator
            self.t_step = **frequency / self.sample_rate;
            self.t += self.t_step;
        }

        Ok(())
    }
}

/// A free-running unipolar noise oscillator.
///
/// The noise oscillator generates a random signal between 0 and 1 that changes every sample.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The output noise signal. |
#[derive(Clone, Debug)]
pub struct NoiseOscillator {
    distribution: rand::distributions::Uniform<f64>,
}

impl NoiseOscillator {
    /// Creates a new noise oscillator.
    pub fn new() -> Self {
        NoiseOscillator {
            distribution: rand::distributions::Uniform::new(0.0, 1.0),
        }
    }
}

impl Default for NoiseOscillator {
    fn default() -> Self {
        Self::new()
    }
}

impl Process for NoiseOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn process(
        &mut self,
        _inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for out in itertools::izip!(out) {
            // generate a random number
            **out = self.distribution.sample(&mut rand::thread_rng());
        }

        Ok(())
    }
}

/// A free-running band-limited sawtooth wave oscillator.
///
/// The band-limited sawtooth wave oscillator generates a sawtooth wave with reduced aliasing artifacts.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `frequency` | `Sample` | `440.0` | The frequency of the sawtooth wave in Hz. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The output sawtooth wave signal. |
#[derive(Clone, Debug)]
pub struct BlSawOscillator {
    p: f64,
    dp: f64,
    saw: f64,
    sample_rate: f64,

    /// The frequency of the sawtooth wave in Hz.
    pub frequency: f64,
}

impl Default for BlSawOscillator {
    fn default() -> Self {
        Self {
            p: 0.0,
            dp: 1.0,
            saw: 0.0,
            sample_rate: 0.0,
            frequency: 440.0,
        }
    }
}

impl BlSawOscillator {
    /// Creates a new band-limited sawtooth wave oscillator.
    pub fn new(frequency: f64) -> Self {
        Self {
            frequency,
            ..Default::default()
        }
    }
}

impl Process for BlSawOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("frequency", 440.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let frequency = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        // algorithm courtesy of https://www.musicdsp.org/en/latest/Synthesis/12-bandlimited-waveforms.html
        for (out, frequency) in itertools::izip!(out, frequency) {
            if **frequency <= 0.0 {
                **out = 0.0;
                continue;
            }

            let pmax = 0.5 * self.sample_rate / **frequency;
            let dc = -0.498 / pmax;

            self.p += self.dp;
            if self.p < 0.0 {
                self.p = -self.p;
                self.dp = -self.dp;
            } else if self.p > pmax {
                self.p = 2.0 * pmax - self.p;
                self.dp = -self.dp;
            }

            let mut x = std::f64::consts::PI * self.p;
            if x < 0.00001 {
                x = 0.00001;
            }

            self.saw = 0.995 * self.saw + dc + x.sin() / x;

            **out = self.saw;
        }

        Ok(())
    }
}

const BL_SQUARE_MAX_HARMONICS: usize = 512;

/// A free-running band-limited square wave oscillator.
#[derive(Clone, Debug)]
pub struct BlSquareOscillator {
    sample_rate: f64,

    // phase accumulator
    t: f64,
    // phase increment per sample
    t_step: f64,

    // band-limited square wave coefficients
    coeff: Box<[f64; BL_SQUARE_MAX_HARMONICS]>,

    /// The frequency of the square wave in Hz.
    pub frequency: f64,
    /// The pulse width of the square wave.
    pub pulse_width: f64,
}

impl Default for BlSquareOscillator {
    fn default() -> Self {
        Self::new(440.0, 0.5)
    }
}

impl BlSquareOscillator {
    /// Creates a new band-limited square wave oscillator.
    pub fn new(frequency: f64, pulse_width: f64) -> Self {
        Self {
            frequency,
            pulse_width,
            t: 0.0,
            t_step: 0.0,
            coeff: Box::new([0.0; BL_SQUARE_MAX_HARMONICS]),
            sample_rate: 0.0,
        }
    }
}

impl Process for BlSquareOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("frequency", 440.0),
            SignalSpec::unbounded("pulse_width", 0.5),
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
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let frequency = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let pulse_width = inputs[1]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (out, frequency, pulse_width) in itertools::izip!(out, frequency, pulse_width) {
            if **frequency <= 0.0 {
                **out = 0.0;
                continue;
            }

            self.frequency = **frequency;
            self.pulse_width = **pulse_width;

            self.t_step = self.frequency / self.sample_rate;

            let n_harm = (self.sample_rate / (self.frequency * 4.0)) as usize;
            self.coeff[0] = self.pulse_width - 0.5;
            for i in 1..n_harm + 1 {
                self.coeff[i] = f64::sin(i as f64 * std::f64::consts::PI * self.pulse_width) * 2.0
                    / (i as f64 * std::f64::consts::PI);
            }

            let theta = self.t * 2.0 * std::f64::consts::PI;

            let mut square = 0.0;
            for i in 0..n_harm + 1 {
                square += self.coeff[i] * (theta * i as f64).cos();
            }

            self.t += self.t_step;

            **out = square;
        }

        Ok(())
    }
}
