//! Oscillator processors.

use rand::prelude::Distribution;

use crate::{
    prelude::*,
    processor::ProcessorOutputs,
    signal::{PI, TAU},
};

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
    t: Sample,
    // phase increment per sample
    t_step: Sample,
}

impl Processor for PhaseAccumulator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("increment", SignalKind::Sample),
            SignalSpec::new("reset", SignalKind::Bool),
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
        for (out, increment, reset) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_bools(1)?
        ) {
            if let Some(true) = reset {
                self.t = 0.0;
            }

            // output the phase accumulator value
            *out = Some(self.t);

            // increment the phase accumulator
            if let Some(increment) = increment {
                self.t_step = increment;
            }
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
    t: Sample,
    // phase increment per sample
    t_step: Sample,
    // sample rate
    sample_rate: Sample,

    /// The frequency of the sine wave in Hz.
    pub frequency: Sample,
    /// The phase of the sine wave in radians.
    pub phase: Sample,
}

impl SineOscillator {
    /// Creates a new sine wave oscillator.
    pub fn new(frequency: Sample) -> Self {
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

impl Processor for SineOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("frequency", SignalKind::Sample),
            SignalSpec::new("phase", SignalKind::Sample),
            SignalSpec::new("reset", SignalKind::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Sample)]
    }

    fn resize_buffers(&mut self, sample_rate: Sample, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, frequency, phase, reset) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
            inputs.iter_input_as_bools(2)?
        ) {
            if let Some(true) = reset {
                self.t = 0.0;
            }

            if let Some(frequency) = frequency {
                self.frequency = frequency;
            }

            if let Some(phase) = phase {
                self.phase = phase;
            }

            // calculate the sine wave using the phase accumulator
            let sine = (self.t * TAU + self.phase).sin();
            *out = Some(sine);

            // increment the phase accumulator
            self.t_step = self.frequency / self.sample_rate;
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
    t: Sample,
    // phase increment per sample
    t_step: Sample,
    // sample rate
    sample_rate: Sample,

    /// The frequency of the sawtooth wave in Hz.
    pub frequency: Sample,
    /// The phase of the sawtooth wave in radians.
    pub phase: Sample,
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
    pub fn new(frequency: Sample) -> Self {
        Self {
            frequency,
            ..Default::default()
        }
    }
}

impl Processor for SawOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("frequency", SignalKind::Sample),
            SignalSpec::new("phase", SignalKind::Sample),
            SignalSpec::new("reset", SignalKind::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Sample)]
    }

    fn resize_buffers(&mut self, sample_rate: Sample, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, frequency, phase, reset) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
            inputs.iter_input_as_bools(2)?
        ) {
            if let Some(true) = reset {
                self.t = 0.0;
            }

            if let Some(frequency) = frequency {
                self.frequency = frequency;
            }

            if let Some(phase) = phase {
                self.phase = phase;
            }

            // calculate the sawtooth wave using the phase accumulator
            *out = Some((self.t + self.phase) % 1.0);

            // increment the phase accumulator
            self.t_step = self.frequency / self.sample_rate;
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

impl Processor for NoiseOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Sample)]
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let mut rng = rand::thread_rng();
        for out in outputs.iter_output_mut_as_samples(0)? {
            // generate a random number
            *out = Some(self.distribution.sample(&mut rng) as Sample);
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
    p: Sample,
    dp: Sample,
    saw: Sample,
    sample_rate: Sample,

    /// The frequency of the sawtooth wave in Hz.
    pub frequency: Sample,
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
    pub fn new(frequency: Sample) -> Self {
        Self {
            frequency,
            ..Default::default()
        }
    }
}

impl Processor for BlSawOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("frequency", SignalKind::Sample)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Sample)]
    }

    fn resize_buffers(&mut self, sample_rate: Sample, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        // algorithm courtesy of https://www.musicdsp.org/en/latest/Synthesis/12-bandlimited-waveforms.html
        for (out, frequency) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?
        ) {
            self.frequency = frequency.unwrap_or(self.frequency);
            if self.frequency <= 0.0 {
                *out = None;
                continue;
            }

            let pmax = 0.5 * self.sample_rate / self.frequency;
            let dc = -0.498 / pmax;

            self.p += self.dp;
            if self.p < 0.0 {
                self.p = -self.p;
                self.dp = -self.dp;
            } else if self.p > pmax {
                self.p = 2.0 * pmax - self.p;
                self.dp = -self.dp;
            }

            let mut x = PI * self.p;
            if x < 0.00001 {
                x = 0.00001;
            }

            self.saw = 0.995 * self.saw + dc + x.sin() / x;

            *out = Some(self.saw);
        }

        Ok(())
    }
}

const BL_SQUARE_MAX_HARMONICS: usize = 512;

/// A free-running band-limited square wave oscillator.
#[derive(Clone, Debug)]
pub struct BlSquareOscillator {
    sample_rate: Sample,

    // phase accumulator
    t: Sample,
    // phase increment per sample
    t_step: Sample,

    // band-limited square wave coefficients
    coeff: Box<[Sample; BL_SQUARE_MAX_HARMONICS]>,

    /// The frequency of the square wave in Hz.
    pub frequency: Sample,
    /// The pulse width of the square wave.
    pub pulse_width: Sample,
}

impl Default for BlSquareOscillator {
    fn default() -> Self {
        Self::new(440.0, 0.5)
    }
}

impl BlSquareOscillator {
    /// Creates a new band-limited square wave oscillator.
    pub fn new(frequency: Sample, pulse_width: Sample) -> Self {
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

impl Processor for BlSquareOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("frequency", SignalKind::Sample),
            SignalSpec::new("pulse_width", SignalKind::Sample),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalKind::Sample)]
    }

    fn resize_buffers(&mut self, sample_rate: Sample, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, frequency, pulse_width) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?
        ) {
            self.frequency = frequency.unwrap_or(self.frequency);
            if self.frequency <= 0.0 {
                *out = None;
                continue;
            }

            self.pulse_width = pulse_width.unwrap_or(self.pulse_width);

            self.t_step = self.frequency / self.sample_rate;

            let n_harm = (self.sample_rate / (self.frequency * 4.0)) as usize;
            self.coeff[0] = self.pulse_width - 0.5;
            for i in 1..n_harm + 1 {
                self.coeff[i] =
                    Sample::sin(i as Sample * PI * self.pulse_width) * 2.0 / (i as Sample * PI);
            }

            let theta = self.t * TAU;

            let mut square = 0.0;
            for i in 0..n_harm + 1 {
                square += self.coeff[i] * (theta * i as Sample).cos();
            }

            self.t += self.t_step;

            *out = Some(square);
        }

        Ok(())
    }
}
