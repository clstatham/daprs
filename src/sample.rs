use std::{
    fmt::{Debug, Display},
    ops::{
        Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub,
        SubAssign,
    },
};

/// A single 64-bit floating-point sample of signal data.
#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Sample(f64);

impl Debug for Sample {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

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
#[derive(PartialEq, Clone)]
pub struct Buffer {
    buf: Vec<Sample>,
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.buf.iter()).finish()
    }
}

impl Buffer {
    /// Creates a new buffer filled with zeros.
    #[inline]
    pub fn zeros(length: usize) -> Self {
        Buffer {
            buf: vec![Sample::new(0.0); length],
        }
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
    pub fn from_slice(value: &[Sample]) -> Self {
        Buffer {
            buf: value.to_vec(),
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
pub enum SignalRate {
    Control,
    Audio,
}

impl SignalRate {
    pub fn is_audio(self) -> bool {
        matches!(self, Self::Audio)
    }

    pub fn is_control(self) -> bool {
        matches!(self, Self::Control)
    }

    pub fn can_take_as_input(self, other: SignalRate) -> bool {
        match self {
            Self::Control => other.is_control(),
            Self::Audio => other.is_audio(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    Sample,
    Buffer,
    Bundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalSpec {
    pub name: Option<&'static str>,
    pub rate: SignalRate,
    pub kind: SignalKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignalData {
    Sample(Sample),
    Buffer(Buffer),
    Bundle(Vec<SignalData>),
}

impl SignalData {
    pub fn default_for_kind(kind: SignalKind) -> Self {
        match kind {
            SignalKind::Sample => Self::Sample(Sample::new(0.0)),
            SignalKind::Buffer => Self::Buffer(Buffer::zeros(0)),
            SignalKind::Bundle => Self::Bundle(Vec::new()),
        }
    }

    #[inline]
    pub fn kind(&self) -> SignalKind {
        match self {
            Self::Sample(_) => SignalKind::Sample,
            Self::Buffer(_) => SignalKind::Buffer,
            Self::Bundle(_) => SignalKind::Bundle,
        }
    }

    pub fn resize_buffers(&mut self, length: usize) {
        match self {
            Self::Buffer(buffer) => buffer.resize(length),
            Self::Bundle(bundles) => {
                for bundle in bundles {
                    bundle.resize_buffers(length);
                }
            }
            _ => {}
        }
    }

    #[inline]
    pub fn copy_from(&mut self, other: &Self) {
        match (self, other) {
            (Self::Sample(a), Self::Sample(b)) => *a = *b,
            (Self::Buffer(a), Self::Sample(b)) => a.map_mut(|x| *x = *b),
            (Self::Buffer(a), Self::Buffer(b)) => a.copy_map(b, |x| x),
            (Self::Bundle(a), Self::Bundle(b)) => {
                for (a, b) in a.iter_mut().zip(b.iter()) {
                    a.copy_from(b);
                }
            }
            (a, b) => {
                panic!(
                    "SignalData::copy_from: mismatched kinds {:?} and {:?}",
                    a.kind(),
                    b.kind()
                );
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    pub spec: SignalSpec,
    pub data: SignalData,
}

impl Signal {
    pub fn new(name: Option<&'static str>, rate: SignalRate, data: SignalData) -> Self {
        Self {
            spec: SignalSpec {
                name,
                rate,
                kind: data.kind(),
            },
            data,
        }
    }

    pub fn default_for_spec(spec: SignalSpec) -> Self {
        Self {
            spec,
            data: SignalData::default_for_kind(spec.kind),
        }
    }

    pub fn with_spec_and_data(spec: SignalSpec, data: SignalData) -> Self {
        Self { spec, data }
    }

    pub fn name(&self) -> Option<&'static str> {
        self.spec.name
    }

    pub fn rate(&self) -> SignalRate {
        self.spec.rate
    }

    pub fn kind(&self) -> SignalKind {
        self.spec.kind
    }

    pub fn is_audio(&self) -> bool {
        self.rate().is_audio()
    }

    pub fn is_control(&self) -> bool {
        self.rate().is_control()
    }

    pub fn can_take_as_input(&self, other: Signal) -> bool {
        self.rate().can_take_as_input(other.rate())
    }

    pub fn resize_buffers(&mut self, length: usize) {
        self.data.resize_buffers(length);
    }

    #[inline]
    pub fn copy_from(&mut self, other: &Self) {
        self.data.copy_from(&other.data);
    }

    pub fn unwrap_buffer(&self) -> &Buffer {
        match &self.data {
            SignalData::Buffer(buffer) => buffer,
            data => panic!(
                "Signal::unwrap_buffer: expected Buffer, got {:?}",
                data.kind()
            ),
        }
    }

    pub fn unwrap_buffer_mut(&mut self) -> &mut Buffer {
        match &mut self.data {
            SignalData::Buffer(buffer) => buffer,
            data => panic!(
                "Signal::unwrap_buffer_mut: expected Buffer, got {:?}",
                data.kind()
            ),
        }
    }
}
