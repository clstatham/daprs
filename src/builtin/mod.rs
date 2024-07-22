use crate::sample::Sample;

pub mod env;
pub mod graph;
pub mod io;
pub mod math;
pub mod osc;
pub mod time;

/// Resamples the input signal to the output signal's length using a linear interpolation algorithm.
/// The output signal is completely overwritten.
#[inline]
fn linear_resample(input: &[Sample], output: &mut [Sample], input_rate: f64) {
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

    // apply a one-pole filter to smooth the output
    let mut y = **output.first().unwrap();
    let alpha = 1.0 - (2.0 * std::f64::consts::PI * 5.0 / input_rate).exp();
    for s in output.iter_mut() {
        let next_y = alpha * **s + (1.0 - alpha) * y;
        y = next_y;
        *s = next_y.into();
    }
}
