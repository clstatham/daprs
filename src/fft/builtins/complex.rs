use num::{Complex, Zero};

use crate::prelude::*;

/// A processor that passes through a complex signal unchanged.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Complex` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Complex` | The output signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComplexBufPassthrough(pub FftBufLength);

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for ComplexBufPassthrough {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new("in", FftSignalType::ComplexBuf(self.0))]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new("out", FftSignalType::ComplexBuf(self.0))]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs[0].as_complex_buf().unwrap();
        let out_signal = outputs[0].as_complex_buf_mut().unwrap();
        out_signal.copy_from_slice(in_signal);
        Ok(())
    }
}

macro_rules! complex_binary_op {
    ($name:ident, $doc:literal, $op:tt) => {
        #[derive(Debug, Clone)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[doc = concat!($doc, r#"\n
# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Complex` | The first signal. |
| `1` | `b` | `Complex` | The second signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Complex` | The result of the operation. |"#)]
        pub struct $name(pub FftBufLength);

        #[cfg_attr(feature = "serde", typetag::serde)]
        impl FftProcessor for $name {
            fn input_spec(&self) -> Vec<FftSpec> {
                vec![
                    FftSpec::new("a", FftSignalType::ComplexBuf(self.0)),
                    FftSpec::new("b", FftSignalType::ComplexBuf(self.0)),
                ]
            }

            fn output_spec(&self) -> Vec<FftSpec> {
                vec![FftSpec::new(
                    "out",
                    FftSignalType::ComplexBuf(self.0),
                )]
            }

            fn process(
                &mut self,
                _fft_length: usize,
                inputs: &[&FftSignal],
                outputs: &mut [FftSignal],
            ) -> Result<(), ProcessorError> {
                let a = inputs[0].as_complex_buf().unwrap();
                let b = inputs[1].as_complex_buf().unwrap();
                let out = outputs[0].as_complex_buf_mut().unwrap();
                for (out, a, b) in itertools::izip!(out.iter_mut(), a, b) {
                    *out = a $op b;
                }

                Ok(())
            }
        }
    };
}

complex_binary_op! {
    ComplexAdd,
    "A processor that adds two complex signals.",
    +
}

complex_binary_op! {
    ComplexSub,
    "A processor that subtracts two complex signals.",
    -
}

complex_binary_op! {
    ComplexMul,
    "A processor that multiplies two complex signals.",
    *
}

complex_binary_op! {
    ComplexDiv,
    "A processor that divides two complex signals.",
    /
}

complex_binary_op! {
    ComplexRem,
    "A processor that calculates the remainder of two complex signals.",
    %
}

/// A processor that calculates the complex conjugate of a complex signal.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Complex` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Complex` | The complex conjugate of the input signal. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComplexConjugate;

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for ComplexConjugate {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "in",
            FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
        )]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "out",
            FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
        )]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let input = inputs[0].as_complex_buf().unwrap();
        let out = outputs[0].as_complex_buf_mut().unwrap();
        for (out, c) in itertools::izip!(out.iter_mut(), input) {
            *out = c.conj();
        }

        Ok(())
    }
}

/// A processor that splits a complex FFT signal into its real and imaginary parts.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Complex` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `real` | `Real` | The real part of the input signal. |
/// | `1` | `imag` | `Real` | The imaginary part of the input signal. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComplexSplit;

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for ComplexSplit {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "in",
            FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
        )]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![
            FftSpec::new(
                "real",
                FftSignalType::RealBuf(FftBufLength::FftLengthPlusOne),
            ),
            FftSpec::new(
                "imag",
                FftSignalType::RealBuf(FftBufLength::FftLengthPlusOne),
            ),
        ]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let input = inputs[0].as_complex_buf().unwrap();
        let [real, imag] = outputs else {
            return Err(ProcessorError::NumOutputsMismatch);
        };
        let real = real.as_real_buf_mut().unwrap();
        let imag = imag.as_real_buf_mut().unwrap();
        for (r, i, c) in itertools::izip!(real.iter_mut(), imag.iter_mut(), input) {
            *r = c.re;
            *i = c.im;
        }

        Ok(())
    }
}

/// A processor that combines real and imaginary parts into a complex FFT signal.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `real` | `Real` | The real part of the signal. |
/// | `1` | `imag` | `Real` | The imaginary part of the signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Complex` | The combined signal. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComplexCombine;

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for ComplexCombine {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![
            FftSpec::new(
                "real",
                FftSignalType::RealBuf(FftBufLength::FftLengthPlusOne),
            ),
            FftSpec::new(
                "imag",
                FftSignalType::RealBuf(FftBufLength::FftLengthPlusOne),
            ),
        ]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "out",
            FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
        )]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let [real, imag] = inputs else {
            return Err(ProcessorError::NumInputsMismatch);
        };
        let out = outputs[0].as_complex_buf_mut().unwrap();
        for (out, r, i) in itertools::izip!(
            out.iter_mut(),
            real.as_real_buf().unwrap().iter(),
            imag.as_real_buf().unwrap().iter()
        ) {
            *out = Complex::new(*r, *i);
        }

        Ok(())
    }
}

/// A processor that calculates the magnitude and phase of a complex FFT signal.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Complex` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `mag` | `Real` | The magnitude of the input signal. |
/// | `1` | `phase` | `Real` | The phase of the input signal. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComplexToPolar;

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for ComplexToPolar {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "in",
            FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
        )]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![
            FftSpec::new(
                "mag",
                FftSignalType::RealBuf(FftBufLength::FftLengthPlusOne),
            ),
            FftSpec::new(
                "phase",
                FftSignalType::RealBuf(FftBufLength::FftLengthPlusOne),
            ),
        ]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let input = inputs[0].as_complex_buf().unwrap();
        let [mag, phase] = outputs else {
            return Err(ProcessorError::NumOutputsMismatch);
        };
        let mag = mag.as_real_buf_mut().unwrap();
        let phase = phase.as_real_buf_mut().unwrap();
        for (m, p, c) in itertools::izip!(mag.iter_mut(), phase.iter_mut(), input) {
            *m = c.norm();
            *p = c.arg();
        }

        Ok(())
    }
}

/// A processor that combines magnitude and phase into a complex FFT signal.
///
/// The phase will be wrapped to the range `[-π, π]`.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `mag` | `Real` | The magnitude of the signal. |
/// | `1` | `phase` | `Real` | The phase of the signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Complex` | The combined signal. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComplexFromPolar;

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for ComplexFromPolar {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![
            FftSpec::new(
                "mag",
                FftSignalType::RealBuf(FftBufLength::FftLengthPlusOne),
            ),
            FftSpec::new(
                "phase",
                FftSignalType::RealBuf(FftBufLength::FftLengthPlusOne),
            ),
        ]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "out",
            FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
        )]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let [mag, phase] = inputs else {
            return Err(ProcessorError::NumInputsMismatch);
        };
        let out = outputs[0].as_complex_buf_mut().unwrap();
        for (out, m, p) in itertools::izip!(
            out.iter_mut(),
            mag.as_real_buf().unwrap().iter(),
            phase.as_real_buf().unwrap().iter()
        ) {
            *out = Complex::from_polar(*m, *p);
        }

        Ok(())
    }
}

/// A phase vocoder.
///
/// This can be used as a time-stretching effect by setting "previous_frame" to a different offset of a sample buffer.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `current_frame` | `Complex` | The current FFT frame of the signal. |
/// | `1` | `previous_frame` | `Complex` | The last FFT frame of the signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Complex` | The output signal. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhaseVocoder {
    phase_accum: RealBuf,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for PhaseVocoder {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![
            FftSpec::new(
                "current_frame",
                FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
            ),
            FftSpec::new(
                "previous_frame",
                FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
            ),
        ]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "out",
            FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
        )]
    }

    fn allocate(&mut self, _fft_length: usize, padded_length: usize) {
        self.phase_accum = RealBuf::new(padded_length);
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let [current_frame, previous_frame] = inputs else {
            return Err(ProcessorError::NumInputsMismatch);
        };
        let out = outputs[0].as_complex_buf_mut().unwrap();

        let current_frame = current_frame.as_complex_buf().unwrap();
        let previous_frame = previous_frame.as_complex_buf().unwrap();

        for (n, (out, current, previous)) in
            itertools::izip!(out.iter_mut(), current_frame, previous_frame).enumerate()
        {
            let in_mag = current.norm();
            let in_phase = current.arg();
            let last_phase = previous.arg();

            let delta_phase = in_phase - last_phase;

            self.phase_accum[n] += delta_phase;
            self.phase_accum[n] %= 2.0 * PI;

            *out = Complex::from_polar(in_mag, self.phase_accum[n]);
        }

        out[0] = Complex::zero();
        *out.last_mut().unwrap() = Complex::zero();

        Ok(())
    }
}
