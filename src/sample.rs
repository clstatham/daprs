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
pub struct Buffer(Box<[Sample]>);

impl Buffer {
    pub fn zeros(length: usize) -> Self {
        Buffer(vec![Sample::new(0.0); length].into_boxed_slice())
    }

    pub fn map_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Sample),
    {
        for sample in self.0.iter_mut() {
            f(sample);
        }
    }
}

impl From<Vec<Sample>> for Buffer {
    #[inline]
    fn from(value: Vec<Sample>) -> Self {
        Buffer(value.into_boxed_slice())
    }
}

impl Buffer {
    #[inline]
    pub fn from_slice(value: &[Sample]) -> Self {
        Buffer(value.into())
    }
}

impl Deref for Buffer {
    type Target = [Sample];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl DerefMut for Buffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl AsRef<[Sample]> for Buffer {
    #[inline]
    fn as_ref(&self) -> &[Sample] {
        self.0.as_ref()
    }
}

impl FromIterator<Sample> for Buffer {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Sample>>(iter: T) -> Self {
        Buffer(iter.into_iter().collect())
    }
}

impl AddAssign<&Buffer> for &mut Buffer {
    #[inline]
    fn add_assign(&mut self, rhs: &Buffer) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.iter()) {
            *lhs += *rhs;
        }
    }
}

impl SubAssign<&Buffer> for &mut Buffer {
    #[inline]
    fn sub_assign(&mut self, rhs: &Buffer) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.iter()) {
            *lhs -= *rhs;
        }
    }
}

impl MulAssign<&Buffer> for &mut Buffer {
    #[inline]
    fn mul_assign(&mut self, rhs: &Buffer) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.iter()) {
            *lhs *= *rhs;
        }
    }
}

impl DivAssign<&Buffer> for &mut Buffer {
    #[inline]
    fn div_assign(&mut self, rhs: &Buffer) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.iter()) {
            *lhs /= *rhs;
        }
    }
}

impl RemAssign<&Buffer> for &mut Buffer {
    #[inline]
    fn rem_assign(&mut self, rhs: &Buffer) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.iter()) {
            *lhs %= *rhs;
        }
    }
}
