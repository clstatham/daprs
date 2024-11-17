//! Built-in processors and utilities for the audio graph.

pub mod control;
pub mod dynamics;
pub mod filters;
pub mod list;
pub mod math;
pub mod midi;
pub mod oscillators;
pub mod storage;
pub mod time;
pub mod util;

pub use control::*;
pub use dynamics::*;
pub use filters::*;
pub use list::*;
pub use math::*;
pub use midi::*;
pub use oscillators::*;
pub use storage::*;
pub use time::*;
pub use util::*;

use crate::signal::Float;

/// Linear interpolation.
#[doc(hidden)]
#[inline]
pub fn lerp(a: Float, b: Float, t: Float) -> Float {
    debug_assert!((0.0..=1.0).contains(&t));
    a + (b - a) * t
}
