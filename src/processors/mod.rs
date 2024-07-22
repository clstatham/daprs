use crate::sample::Sample;

pub mod env;
#[macro_use]
pub mod functional;
pub mod control;
pub mod graph;
pub mod io;
pub mod math;
pub mod osc;
pub mod time;

#[inline]
pub fn resample(input: &[Sample], output: &mut [Sample]) {
    // match input.len().cmp(&output.len()) {
    //     std::cmp::Ordering::Equal => output.copy_from_slice(input),
    //     std::cmp::Ordering::Less => linear_resample(input, output),
    //     std::cmp::Ordering::Greater => lanczos_resample(input, output),
    // }
    linear_resample(input, output);
}

/// Resamples the input signal to the output signal's length using a linear interpolation algorithm.
/// The output signal is completely overwritten.
#[inline]
pub fn linear_resample(input: &[Sample], output: &mut [Sample]) {
    let input_len = input.len();
    let output_len = output.len();
    if input_len == output_len {
        // fast path
        output.copy_from_slice(input);
        return;
    }
    let step = input_len as f64 / output_len as f64;
    let mut i = 0.0;
    for o in output.iter_mut() {
        let i0 = i as usize;
        if i0 >= input_len - 1 {
            *o = input[input_len - 1];
            return;
        }
        let i1 = i0 + 1;
        let a = i - i0 as f64;
        let b = 1.0 - a;
        *o = input[i0] * b.into() + input[i1] * a.into();
        i += step;
    }
}

#[inline]
pub fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else {
        x.sin() / x
    }
}

#[inline]
pub fn lanczos(x: f64, a: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else if x.abs() < a {
        a * sinc(x) * sinc(x / a)
    } else {
        0.0
    }
}

/// Resamples the input signal to the output signal's length using a Lanczos interpolation algorithm.
/// The output signal is completely overwritten.
#[inline]
pub fn lanczos_resample(input: &[Sample], output: &mut [Sample]) {
    let input_len = input.len();
    let output_len = output.len();
    if input_len == output_len {
        // fast path
        output.copy_from_slice(input);
        return;
    }
    let a = 3.0; // Lanczos window size (3 is a good default)
    let step = input_len as f64 / output_len as f64;
    let mut i = 0.0;
    for o in output.iter_mut() {
        let mut sum = 0.0;
        let mut weight_sum = 0.0;
        for j in 0..input_len {
            let x = (i - j as f64) * a;
            let weight = lanczos(x, a);
            sum += **input.get(j).unwrap_or(&Sample::new(0.0)) * weight;
            weight_sum += weight;
        }
        *o = (sum / weight_sum).into();
        i += step;
    }
}
