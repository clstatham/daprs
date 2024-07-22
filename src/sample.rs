use std::{
    fmt::Display,
    ops::{
        Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub,
        SubAssign,
    },
};

/// A single 64-bit floating-point sample of signal data.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Sample(f64);

impl Sample {
    #[inline]
    pub const fn new(value: f64) -> Self {
        Sample(value)
    }
}

impl From<Sample> for f64 {
    #[inline]
    fn from(sample: Sample) -> Self {
        sample.0
    }
}

impl From<f64> for Sample {
    #[inline]
    fn from(value: f64) -> Self {
        Sample(value)
    }
}

impl AsRef<f64> for Sample {
    #[inline]
    fn as_ref(&self) -> &f64 {
        &self.0
    }
}

impl Deref for Sample {
    type Target = f64;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add for Sample {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Sample(self.0 + rhs.0)
    }
}

impl Sub for Sample {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Sample(self.0 - rhs.0)
    }
}

impl Mul for Sample {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Sample(self.0 * rhs.0)
    }
}

impl Div for Sample {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Sample(self.0 / rhs.0)
    }
}

impl Rem for Sample {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        Sample(self.0 % rhs.0)
    }
}

impl AddAssign for Sample {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Sample {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl MulAssign for Sample {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl DivAssign for Sample {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl RemAssign for Sample {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
    }
}

impl Neg for Sample {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Sample(-self.0)
    }
}

impl Display for Sample {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An owning, fixed-length array of [`Sample`]s.
/// This type implements [`Deref`] and [`DerefMut`], so it can be indexed and iterated over just like a normal slice.
/// It can also be [`collected`](std::iter::Iterator::collect) from an iterator of [`Sample`]s.
#[derive(Debug, PartialEq, Clone)]
pub struct Buffer {
    buf: Vec<Sample>,
    kind: SignalKind,
}

impl Buffer {
    /// Creates a new buffer filled with zeros.
    #[inline]
    pub fn zeros(length: usize, kind: SignalKind) -> Self {
        Buffer {
            buf: vec![Sample::new(0.0); length],
            kind,
        }
    }

    /// Returns the buffer's signal kind.
    #[inline]
    pub fn kind(&self) -> SignalKind {
        self.kind
    }

    /// Resizes the buffer to the given length, filling any new elements with zeros.
    #[inline]
    pub fn resize(&mut self, length: usize) {
        if self.len() != length {
            self.buf.resize(length, Sample::new(0.0));
        }
    }

    /// Maps each sample in `other` with `f`, storing the result in the correspeonding sample in `self`.
    #[inline]
    pub fn copy_map<F>(&mut self, other: &[Sample], mut f: F)
    where
        F: FnMut(Sample) -> Sample,
    {
        for (a, b) in self.buf.iter_mut().zip(other.iter()) {
            *a = f(*b);
        }
    }

    /// Iterates over each sample in the buffer, calling `f` with a mutable reference to each sample.
    #[inline]
    pub fn map_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Sample),
    {
        for sample in self.buf.iter_mut() {
            f(sample);
        }
    }

    #[inline]
    pub fn from_slice(value: &[Sample], kind: SignalKind) -> Self {
        Buffer {
            buf: value.to_vec(),
            kind,
        }
    }
}

impl Deref for Buffer {
    type Target = [Sample];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.buf.as_ref()
    }
}

impl DerefMut for Buffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf.as_mut()
    }
}

impl AsRef<[Sample]> for Buffer {
    #[inline]
    fn as_ref(&self) -> &[Sample] {
        self.buf.as_ref()
    }
}

impl<'a> IntoIterator for &'a Buffer {
    type Item = &'a Sample;
    type IntoIter = std::slice::Iter<'a, Sample>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter()
    }
}

impl<'a> IntoIterator for &'a mut Buffer {
    type Item = &'a mut Sample;
    type IntoIter = std::slice::IterMut<'a, Sample>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    Control,
    Audio,
}

impl SignalKind {
    pub fn is_audio(self) -> bool {
        matches!(self, Self::Audio)
    }

    pub fn is_control(self) -> bool {
        matches!(self, Self::Control)
    }

    pub fn can_take_as_input(self, other: SignalKind) -> bool {
        match self {
            Self::Control => other.is_control(),
            Self::Audio => other.is_audio(),
        }
    }
}

pub trait SignalKindMarker: Copy + Send + Sync + 'static {
    const KIND: SignalKind;
}

#[derive(Debug, Clone, Copy)]
pub struct Audio;
impl SignalKindMarker for Audio {
    const KIND: SignalKind = SignalKind::Audio;
}

#[derive(Debug, Clone, Copy)]
pub struct Control;
impl SignalKindMarker for Control {
    const KIND: SignalKind = SignalKind::Control;
}
