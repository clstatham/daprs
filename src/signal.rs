//! Signal types and operations.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::Path,
};

#[cfg(feature = "f32_samples")]
/// The type of samples used in the signal processing system.
pub type Sample = f32;
#[cfg(not(feature = "f32_samples"))]
/// The type of samples used in the signal processing system.
pub type Sample = f64;

#[cfg(feature = "f32_samples")]
/// The value of π.
pub const PI: Sample = std::f32::consts::PI;
/// The value of π.
#[cfg(not(feature = "f32_samples"))]
pub const PI: Sample = std::f64::consts::PI;

#[cfg(feature = "f32_samples")]
/// The value of τ (2π).
pub const TAU: Sample = std::f32::consts::TAU;
#[cfg(not(feature = "f32_samples"))]
/// The value of τ (2π).
pub const TAU: Sample = std::f64::consts::TAU;

/// An owning, fixed-length array of [`Sample`]s.
/// This type implements [`Deref`] and [`DerefMut`], so it can be indexed and iterated over just like a normal slice.
/// It can also be [`collected`](std::iter::Iterator::collect) from an iterator of [`Sample`]s.
#[derive(PartialEq, Clone)]
pub struct Buffer<T> {
    buf: Vec<T>,
}

impl<T: Debug> Debug for Buffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.buf.iter()).finish()
    }
}

impl<T> Buffer<T> {
    /// Creates a new buffer filled with zeros.
    #[inline]
    pub fn zeros(length: usize) -> Self
    where
        T: Default,
    {
        let mut buf = Vec::with_capacity(length);
        for _ in 0..length {
            buf.push(T::default());
        }
        Buffer { buf }
    }

    /// Resizes the buffer to the given length, filling any new elements with the given value.
    #[inline]
    pub fn resize(&mut self, length: usize, value: T)
    where
        T: Clone,
    {
        if self.len() != length {
            self.buf.resize(length, value);
        }
    }

    /// Maps each sample in `other` with `f`, storing the result in the correspeonding sample in `self`.
    #[inline]
    pub fn copy_map<F>(&mut self, other: &[T], mut f: F)
    where
        F: FnMut(&T) -> T,
    {
        for (a, b) in self.buf.iter_mut().zip(other.iter()) {
            *a = f(b);
        }
    }

    /// Iterates over each sample in the buffer, calling `f` with a mutable reference to each sample.
    #[inline]
    pub fn map_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        for sample in self.buf.iter_mut() {
            f(sample);
        }
    }

    /// Clones the given slice into a new buffer.
    #[inline]
    pub fn from_slice(value: &[T]) -> Self
    where
        T: Clone,
    {
        Buffer {
            buf: value.to_vec(),
        }
    }
}

impl Buffer<Sample> {
    /// Loads a buffer from a WAV file.
    ///
    /// Multi-channel WAV files are supported, but only the first channel will be loaded.
    pub fn load_wav(path: impl AsRef<Path>) -> Result<Self, hound::Error> {
        let reader = hound::WavReader::open(path)?;
        if reader.spec().channels == 1 {
            let samples: Result<Vec<_>, hound::Error> = reader
                .into_samples::<f32>()
                .map(|sample| Ok(sample?.into()))
                .collect();
            let samples = samples?;

            Ok(Buffer::from_slice(&samples))
        } else {
            let channels = reader.spec().channels;

            let samples: Result<Vec<_>, hound::Error> = reader
                .into_samples::<f32>()
                .step_by(channels as usize)
                .map(|sample| Ok(sample?.into()))
                .collect();
            let samples = samples?;

            Ok(Buffer::from_slice(&samples))
        }
    }

    /// Saves the buffer to a WAV file.
    ///
    /// The buffer will be saved as a single-channel 32-bit WAV file with the given sample rate.
    pub fn save_wav(&self, path: impl AsRef<Path>, sample_rate: u32) -> Result<(), hound::Error> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(path, spec)?;
        for sample in self.buf.iter() {
            writer.write_sample(*sample as f32)?;
        }
        writer.finalize()?;
        Ok(())
    }

    /// Returns the maximum value in the buffer.
    #[inline]
    pub fn max(&self) -> Sample {
        self.buf.iter().copied().fold(Sample::MIN, |a, b| a.max(b))
    }

    /// Returns the minimum value in the buffer.
    #[inline]
    pub fn min(&self) -> Sample {
        self.buf.iter().copied().fold(Sample::MAX, |a, b| a.min(b))
    }

    /// Returns the sum of all values in the buffer.
    #[inline]
    pub fn sum(&self) -> Sample {
        self.buf.iter().copied().fold(0.0, |a, b| a + b)
    }

    /// Returns the mean of all values in the buffer.
    #[inline]
    pub fn mean(&self) -> Sample {
        self.sum() / self.len() as Sample
    }

    /// Returns the root mean square of all values in the buffer.
    #[inline]
    pub fn rms(&self) -> Sample {
        self.buf.iter().copied().fold(0.0, |a, b| a + b * b)
    }

    /// Returns the variance of all values in the buffer.
    #[inline]
    pub fn variance(&self) -> Sample {
        let mean = self.mean();
        let sum = self
            .buf
            .iter()
            .copied()
            .fold(0.0, |a, b| a + (b - mean) * (b - mean));
        sum / (self.len() - 1) as Sample
    }

    /// Returns the standard deviation of all values in the buffer.
    #[inline]
    pub fn stddev(&self) -> Sample {
        self.variance().sqrt()
    }
}

impl<T> Deref for Buffer<T> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.buf.as_ref()
    }
}

impl<T> DerefMut for Buffer<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf.as_mut()
    }
}

impl<T> AsRef<[T]> for Buffer<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.buf.as_ref()
    }
}

impl<T> From<Vec<T>> for Buffer<T> {
    #[inline]
    fn from(vec: Vec<T>) -> Self {
        Buffer { buf: vec }
    }
}

impl<T> From<Box<[T]>> for Buffer<T> {
    #[inline]
    fn from(buf: Box<[T]>) -> Self {
        Buffer {
            buf: buf.into_vec(),
        }
    }
}

impl<T> From<&[T]> for Buffer<T>
where
    T: Clone,
{
    #[inline]
    fn from(slice: &[T]) -> Self {
        Buffer {
            buf: slice.to_vec(),
        }
    }
}

impl<'a, T> IntoIterator for &'a Buffer<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Buffer<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut()
    }
}

pub trait SignalData: Clone + Debug + Send + Sync + 'static {
    const KIND: SignalKind;
    type BufferElement: Default + Clone + Debug + Send + Sync;
    type Value: Default + Clone + Debug + Send + Sync + PartialOrd + PartialEq;

    #[inline]
    fn new_signal_buffer(length: usize) -> SignalBuffer {
        SignalBuffer::new_of_kind(Self::KIND, length)
    }

    fn buffer_element_default() -> &'static Self::BufferElement;
    fn buffer_element_to_value(element: &Self::BufferElement) -> Option<&Self::Value>;
    fn value_to_buffer_element(value: &Self::Value) -> Self::BufferElement;

    fn into_signal(this: Self::Value) -> Signal;
    fn try_from_signal(signal: Signal) -> Option<Self::Value>;

    fn cast_buffer_element_from_signal(signal: &Signal) -> Self::BufferElement;

    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self::BufferElement>>;
    fn try_convert_buffer_mut(
        buffer: &mut SignalBuffer,
    ) -> Option<&mut Buffer<Self::BufferElement>>;
}

impl SignalData for Sample {
    const KIND: SignalKind = SignalKind::Sample;
    type BufferElement = Option<Sample>;
    type Value = Sample;

    #[inline]
    fn buffer_element_default() -> &'static Self::BufferElement {
        &None
    }

    #[inline]
    fn buffer_element_to_value(element: &Self::BufferElement) -> Option<&Self::Value> {
        element.as_ref()
    }

    #[inline]
    fn value_to_buffer_element(value: &Self::Value) -> Self::BufferElement {
        Some(*value)
    }

    #[inline]
    fn into_signal(this: Self) -> Signal {
        Signal::Sample(this)
    }

    #[inline]
    fn try_from_signal(signal: Signal) -> Option<Self::Value> {
        match signal {
            Signal::Sample(sample) => Some(sample),
            _ => None,
        }
    }

    #[inline]
    fn cast_buffer_element_from_signal(signal: &Signal) -> Self::BufferElement {
        match signal {
            Signal::Sample(sample) => Some(*sample),
            Signal::Int(int) => Some(*int as Sample),
            Signal::Bool(bool) => {
                if *bool {
                    Some(1.0)
                } else {
                    Some(0.0)
                }
            }
            Signal::String(string) => string.parse().ok(),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self::BufferElement>> {
        buffer.as_sample()
    }

    #[inline]
    fn try_convert_buffer_mut(
        buffer: &mut SignalBuffer,
    ) -> Option<&mut Buffer<Self::BufferElement>> {
        buffer.as_sample_mut()
    }
}

impl SignalData for bool {
    const KIND: SignalKind = SignalKind::Bool;
    type BufferElement = Option<bool>;
    type Value = bool;

    #[inline]
    fn buffer_element_default() -> &'static Self::BufferElement {
        &None
    }

    #[inline]
    fn buffer_element_to_value(element: &Self::BufferElement) -> Option<&Self::Value> {
        element.as_ref()
    }

    #[inline]
    fn value_to_buffer_element(value: &Self::Value) -> Self::BufferElement {
        Some(*value)
    }

    #[inline]
    fn into_signal(this: Self) -> Signal {
        Signal::Bool(this)
    }

    #[inline]
    fn try_from_signal(signal: Signal) -> Option<Self> {
        match signal {
            Signal::Bool(bool) => Some(bool),
            _ => None,
        }
    }

    #[inline]
    fn cast_buffer_element_from_signal(signal: &Signal) -> Self::BufferElement {
        match signal {
            Signal::Bool(bool) => Some(*bool),
            Signal::Int(int) => Some(*int != 0),
            Signal::Sample(sample) => Some(*sample != 0.0),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self::BufferElement>> {
        buffer.as_bool()
    }

    #[inline]
    fn try_convert_buffer_mut(
        buffer: &mut SignalBuffer,
    ) -> Option<&mut Buffer<Self::BufferElement>> {
        buffer.as_bool_mut()
    }
}

impl SignalData for i64 {
    const KIND: SignalKind = SignalKind::Int;
    type BufferElement = Option<i64>;
    type Value = i64;

    #[inline]
    fn buffer_element_default() -> &'static Self::BufferElement {
        &None
    }

    #[inline]
    fn buffer_element_to_value(element: &Self::BufferElement) -> Option<&Self::Value> {
        element.as_ref()
    }

    #[inline]
    fn value_to_buffer_element(value: &Self::Value) -> Self::BufferElement {
        Some(*value)
    }

    #[inline]
    fn into_signal(this: Self) -> Signal {
        Signal::Int(this)
    }

    #[inline]
    fn try_from_signal(signal: Signal) -> Option<Self> {
        match signal {
            Signal::Int(int) => Some(int),
            _ => None,
        }
    }

    #[inline]
    fn cast_buffer_element_from_signal(signal: &Signal) -> Option<Self> {
        match signal {
            Signal::Int(int) => Some(*int),
            Signal::Bool(bool) => Some(if *bool { 1 } else { 0 }),
            Signal::Sample(sample) => Some(*sample as i64),
            Signal::String(string) => string.parse().ok(),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self::BufferElement>> {
        buffer.as_int()
    }

    #[inline]
    fn try_convert_buffer_mut(
        buffer: &mut SignalBuffer,
    ) -> Option<&mut Buffer<Self::BufferElement>> {
        buffer.as_int_mut()
    }
}

impl SignalData for String {
    const KIND: SignalKind = SignalKind::String;
    type BufferElement = Option<String>;
    type Value = String;

    #[inline]
    fn buffer_element_default() -> &'static Self::BufferElement {
        &None
    }

    #[inline]
    fn buffer_element_to_value(element: &Self::BufferElement) -> Option<&Self::Value> {
        element.as_ref()
    }

    #[inline]
    fn value_to_buffer_element(value: &Self::Value) -> Self::BufferElement {
        Some(value.clone())
    }

    #[inline]
    fn into_signal(this: Self) -> Signal {
        Signal::String(this)
    }

    #[inline]
    fn try_from_signal(signal: Signal) -> Option<Self> {
        match signal {
            Signal::String(string) => Some(string),
            _ => None,
        }
    }

    #[inline]
    fn cast_buffer_element_from_signal(signal: &Signal) -> Option<Self> {
        match signal {
            Signal::String(string) => Some(string.clone()),
            Signal::Int(int) => Some(int.to_string()),
            Signal::Bool(bool) => Some(bool.to_string()),
            Signal::Sample(sample) => Some(sample.to_string()),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self::BufferElement>> {
        buffer.as_string()
    }

    #[inline]
    fn try_convert_buffer_mut(
        buffer: &mut SignalBuffer,
    ) -> Option<&mut Buffer<Self::BufferElement>> {
        buffer.as_string_mut()
    }
}

impl SignalData for Vec<Signal> {
    const KIND: SignalKind = SignalKind::List;
    type BufferElement = Option<Vec<Signal>>;
    type Value = Vec<Signal>;

    #[inline]
    fn buffer_element_default() -> &'static Self::BufferElement {
        &None
    }

    #[inline]
    fn buffer_element_to_value(element: &Self::BufferElement) -> Option<&Self::Value> {
        element.as_ref()
    }

    #[inline]
    fn value_to_buffer_element(value: &Self::Value) -> Self::BufferElement {
        Some(value.clone())
    }

    #[inline]
    fn into_signal(this: Self) -> Signal {
        Signal::List(this)
    }

    #[inline]
    fn try_from_signal(signal: Signal) -> Option<Self> {
        match signal {
            Signal::List(list) => Some(list),
            _ => None,
        }
    }

    #[inline]
    fn cast_buffer_element_from_signal(signal: &Signal) -> Option<Self> {
        match signal {
            Signal::List(list) => Some(list.clone()),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self::BufferElement>> {
        buffer.as_list()
    }

    #[inline]
    fn try_convert_buffer_mut(
        buffer: &mut SignalBuffer,
    ) -> Option<&mut Buffer<Self::BufferElement>> {
        buffer.as_list_mut()
    }
}

impl SignalData for Vec<u8> {
    const KIND: SignalKind = SignalKind::Midi;
    type BufferElement = Option<Vec<u8>>;
    type Value = Vec<u8>;

    #[inline]
    fn buffer_element_default() -> &'static Self::BufferElement {
        &None
    }

    #[inline]
    fn buffer_element_to_value(element: &Self::BufferElement) -> Option<&Self::Value> {
        element.as_ref()
    }

    #[inline]
    fn value_to_buffer_element(value: &Self::Value) -> Self::BufferElement {
        Some(value.clone())
    }

    #[inline]
    fn into_signal(this: Self) -> Signal {
        Signal::Midi(this)
    }

    #[inline]
    fn try_from_signal(signal: Signal) -> Option<Self> {
        match signal {
            Signal::Midi(midi) => Some(midi),
            _ => None,
        }
    }

    #[inline]
    fn cast_buffer_element_from_signal(signal: &Signal) -> Option<Self> {
        match signal {
            Signal::Midi(midi) => Some(midi.clone()),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self::BufferElement>> {
        buffer.as_midi()
    }

    #[inline]
    fn try_convert_buffer_mut(
        buffer: &mut SignalBuffer,
    ) -> Option<&mut Buffer<Self::BufferElement>> {
        buffer.as_midi_mut()
    }
}

/// A signal that can be processed by the audio graph.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Signal {
    /// A signal with no value. The inner [`SignalKind`] specifies the kind of signal this would be if it had a value.
    None(SignalKind),
    /// A single sample of audio.
    Sample(Sample),
    /// An integer.
    Int(i64),
    /// A boolean.
    Bool(bool),
    /// A string.
    String(String),
    /// A list.
    List(Vec<Signal>),
    /// A MIDI message.
    Midi(Vec<u8>),
}

impl Signal {
    /// Creates a new signal with the given kind.
    pub const fn new_none(kind: SignalKind) -> Self {
        Self::None(kind)
    }

    /// Creates a new sample signal with the given value.
    pub const fn new_sample(value: Sample) -> Self {
        Self::Sample(value)
    }

    /// Creates a new integer signal with the given value.
    pub const fn new_int(value: i64) -> Self {
        Self::Int(value)
    }

    /// Creates a new boolean signal with the given value.
    pub const fn new_bool(value: bool) -> Self {
        Self::Bool(value)
    }

    /// Creates a new string signal with the given value. The value will be cloned.
    pub fn new_string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Creates a new list signal with the given value. The value will be cloned.
    pub fn new_list(value: impl Into<Vec<Signal>>) -> Self {
        Self::List(value.into())
    }

    /// Creates a new MIDI signal with the given value. The value will be cloned.
    pub fn new_midi(value: impl Into<Vec<u8>>) -> Self {
        Self::Midi(value.into())
    }

    /// Returns `true` if the signal is `None`.
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None(_))
    }

    /// Returns `true` if the signal is a sample.
    #[inline]
    pub fn is_sample(&self) -> bool {
        matches!(self, Self::Sample(_))
    }

    /// Returns `true` if the signal is an integer.
    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    /// Returns `true` if the signal is a boolean.
    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Returns `true` if the signal is a string.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns `true` if the signal is a list.
    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Returns `true` if the signal is a MIDI message.
    #[inline]
    pub fn is_midi(&self) -> bool {
        matches!(self, Self::Midi(_))
    }

    /// Returns the inner [`Sample`], if this is a sample signal.
    #[inline]
    pub fn as_sample(&self) -> Option<Sample> {
        match self {
            Self::Sample(sample) => Some(*sample),
            _ => None,
        }
    }

    /// Returns the inner [`i64`], if this is an integer signal.
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(int) => Some(*int),
            _ => None,
        }
    }

    /// Returns the inner [`bool`], if this is a boolean signal.
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(bool) => Some(*bool),
            _ => None,
        }
    }

    /// Returns the inner [`String`], if this is a string signal.
    #[inline]
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    /// Returns the inner [`Vec<Signal>`], if this is a list signal.
    #[inline]
    pub fn as_list(&self) -> Option<&Vec<Signal>> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

    /// Returns the inner [`Vec<u8>`], if this is a MIDI signal.
    #[inline]
    pub fn as_midi(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Midi(midi) => Some(midi),
            _ => None,
        }
    }

    pub fn is_same_kind(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None(kind), Self::None(other_kind)) => kind == other_kind,
            (Self::Sample(_), Self::Sample(_)) => true,
            (Self::Int(_), Self::Int(_)) => true,
            (Self::Bool(_), Self::Bool(_)) => true,
            (Self::String(_), Self::String(_)) => true,
            (Self::List(_), Self::List(_)) => true,
            (Self::Midi(_), Self::Midi(_)) => true,
            _ => false,
        }
    }

    pub fn kind(&self) -> SignalKind {
        match self {
            Self::None(kind) => *kind,
            Self::Sample(_) => SignalKind::Sample,
            Self::Int(_) => SignalKind::Int,
            Self::Bool(_) => SignalKind::Bool,
            Self::String(_) => SignalKind::String,
            Self::List(_) => SignalKind::List,
            Self::Midi(_) => SignalKind::Midi,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Signal> for Sample {
    fn into(self) -> Signal {
        Signal::Sample(self)
    }
}

/// A signal kind, which can be either a sample or a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignalKind {
    /// A sample or float value.
    Sample,
    /// An integer message.
    Int,
    /// A boolean message.
    Bool,
    /// A string message.
    String,
    /// A list message.
    List,
    /// A MIDI message.
    Midi,
}

/// A buffer that can contain either samples or messages.
#[derive(Debug, Clone)]
pub enum SignalBuffer {
    /// A buffer of samples. The buffer is guaranteed to be full of contiguous samples.
    Sample(Buffer<Option<Sample>>),
    /// A buffer of integers.
    Int(Buffer<Option<i64>>),
    /// A buffer of booleans.
    Bool(Buffer<Option<bool>>),
    /// A buffer of strings.
    String(Buffer<Option<String>>),
    /// A buffer of lists.
    List(Buffer<Option<Vec<Signal>>>),
    /// A buffer of MIDI messages.
    Midi(Buffer<Option<Vec<u8>>>),
}

impl SignalBuffer {
    /// Creates a new signal buffer of the given kind and length, filled with zeros.
    pub fn new_of_kind(kind: SignalKind, length: usize) -> Self {
        match kind {
            SignalKind::Sample => Self::Sample(Buffer::zeros(length)),
            SignalKind::Int => Self::Int(Buffer::zeros(length)),
            SignalKind::Bool => Self::Bool(Buffer::zeros(length)),
            SignalKind::String => Self::String(Buffer::zeros(length)),
            SignalKind::List => Self::List(Buffer::zeros(length)),
            SignalKind::Midi => Self::Midi(Buffer::zeros(length)),
        }
    }

    /// Creates a new signal buffer of the given data kind and length, filled with zeros.
    pub fn new_of_data_kind<T: SignalData>(length: usize) -> Self {
        Self::new_of_kind(T::KIND, length)
    }

    /// Creates a new sample buffer of size `length`, filled with zeros.
    pub fn new_sample(length: usize) -> Self {
        Self::Sample(Buffer::zeros(length))
    }

    /// Creates a new integer buffer of size `length`, filled with `None`.
    pub fn new_int(length: usize) -> Self {
        Self::Int(Buffer::zeros(length))
    }

    /// Creates a new boolean buffer of size `length`, filled with `None`.
    pub fn new_bool(length: usize) -> Self {
        Self::Bool(Buffer::zeros(length))
    }

    /// Creates a new string buffer of size `length`, filled with `None`.
    pub fn new_string(length: usize) -> Self {
        Self::String(Buffer::zeros(length))
    }

    /// Creates a new list buffer of size `length`, filled with `None`.
    pub fn new_list(length: usize) -> Self {
        Self::List(Buffer::zeros(length))
    }

    /// Creates a new MIDI buffer of size `length`, filled with `None`.
    pub fn new_midi(length: usize) -> Self {
        Self::Midi(Buffer::zeros(length))
    }

    /// Returns the kind of signal in the buffer.
    pub fn kind(&self) -> SignalKind {
        match self {
            Self::Sample(_) => SignalKind::Sample,
            Self::Int(_) => SignalKind::Int,
            Self::Bool(_) => SignalKind::Bool,
            Self::String(_) => SignalKind::String,
            Self::List(_) => SignalKind::List,
            Self::Midi(_) => SignalKind::Midi,
        }
    }

    /// Returns `true` if the buffer contains samples.
    pub const fn is_sample(&self) -> bool {
        matches!(self, Self::Sample(_))
    }

    /// Returns `true` if the buffer contains integers.
    pub const fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    /// Returns `true` if the buffer contains booleans.
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Returns `true` if the buffer contains strings.
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns `true` if the buffer contains lists.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Returns `true` if the buffer contains MIDI messages.
    pub const fn is_midi(&self) -> bool {
        matches!(self, Self::Midi(_))
    }

    /// Returns `true` if the buffer contains the given kind of signal.
    pub fn is_kind(&self, kind: SignalKind) -> bool {
        self.kind() == kind
    }

    /// Returns a reference to the sample buffer, if this is a sample buffer.
    #[inline]
    pub fn as_sample(&self) -> Option<&Buffer<Option<Sample>>> {
        match self {
            Self::Sample(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the integer buffer, if this is an integer buffer.
    #[inline]
    pub fn as_int(&self) -> Option<&Buffer<Option<i64>>> {
        match self {
            Self::Int(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the boolean buffer, if this is a boolean buffer.
    #[inline]
    pub fn as_bool(&self) -> Option<&Buffer<Option<bool>>> {
        match self {
            Self::Bool(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the string buffer, if this is a string buffer.
    #[inline]
    pub fn as_string(&self) -> Option<&Buffer<Option<String>>> {
        match self {
            Self::String(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the list buffer, if this is a list buffer.
    #[inline]
    pub fn as_list(&self) -> Option<&Buffer<Option<Vec<Signal>>>> {
        match self {
            Self::List(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the MIDI buffer, if this is a MIDI buffer.
    #[inline]
    pub fn as_midi(&self) -> Option<&Buffer<Option<Vec<u8>>>> {
        match self {
            Self::Midi(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_kind<S: SignalData>(&self) -> Option<&Buffer<S::BufferElement>> {
        S::try_convert_buffer(self)
    }

    /// Returns a mutable reference to the sample buffer, if this is a sample buffer.
    #[inline]
    pub fn as_sample_mut(&mut self) -> Option<&mut Buffer<Option<Sample>>> {
        match self {
            Self::Sample(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the integer buffer, if this is an integer buffer.
    #[inline]
    pub fn as_int_mut(&mut self) -> Option<&mut Buffer<Option<i64>>> {
        match self {
            Self::Int(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the boolean buffer, if this is a boolean buffer.
    #[inline]
    pub fn as_bool_mut(&mut self) -> Option<&mut Buffer<Option<bool>>> {
        match self {
            Self::Bool(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the string buffer, if this is a string buffer.
    #[inline]
    pub fn as_string_mut(&mut self) -> Option<&mut Buffer<Option<String>>> {
        match self {
            Self::String(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the list buffer, if this is a list buffer.
    #[inline]
    pub fn as_list_mut(&mut self) -> Option<&mut Buffer<Option<Vec<Signal>>>> {
        match self {
            Self::List(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the MIDI buffer, if this is a MIDI buffer.
    #[inline]
    pub fn as_midi_mut(&mut self) -> Option<&mut Buffer<Option<Vec<u8>>>> {
        match self {
            Self::Midi(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_kind_mut<S: SignalData>(&mut self) -> Option<&mut Buffer<S::BufferElement>> {
        S::try_convert_buffer_mut(self)
    }

    /// Returns the length of the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Sample(buffer) => buffer.len(),
            Self::Int(buffer) => buffer.len(),
            Self::Bool(buffer) => buffer.len(),
            Self::String(buffer) => buffer.len(),
            Self::List(buffer) => buffer.len(),
            Self::Midi(buffer) => buffer.len(),
        }
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resizes the buffer to the given length, filling any new elements with the given value.
    pub fn resize(&mut self, length: usize, value: impl Into<Signal>) {
        let value = value.into();
        match (self, value) {
            (Self::Sample(buffer), Signal::Sample(value)) => buffer.resize(length, Some(value)),
            (Self::Int(buffer), Signal::Int(value)) => buffer.resize(length, Some(value)),
            (Self::Bool(buffer), Signal::Bool(value)) => buffer.resize(length, Some(value)),
            (Self::String(buffer), Signal::String(value)) => buffer.resize(length, Some(value)),
            (Self::List(buffer), Signal::List(value)) => buffer.resize(length, Some(value)),
            (Self::Midi(buffer), Signal::Midi(value)) => buffer.resize(length, Some(value)),
            _ => panic!("Cannot resize buffer with value of different type"),
        }
    }

    /// Fills the buffer with the given value.
    pub fn fill(&mut self, value: impl Into<Signal>) {
        let value = value.into();
        match (self, value) {
            (Self::Sample(buffer), Signal::Sample(value)) => buffer.fill(Some(value)),
            (Self::Int(buffer), Signal::Int(value)) => buffer.fill(Some(value)),
            (Self::Bool(buffer), Signal::Bool(value)) => buffer.fill(Some(value)),
            (Self::String(buffer), Signal::String(value)) => buffer.fill(Some(value)),
            (Self::List(buffer), Signal::List(value)) => buffer.fill(Some(value)),
            (Self::Midi(buffer), Signal::Midi(value)) => buffer.fill(Some(value)),
            _ => panic!("Cannot fill buffer with value of different type"),
        }
    }

    /// Resizes the buffer to the given length, filling any new elements with an appropriate default value.
    pub fn resize_default(&mut self, length: usize) {
        match self {
            Self::Sample(buffer) => buffer.resize(length, Some(0.0)),
            Self::Int(buffer) => buffer.resize(length, Some(0)),
            Self::Bool(buffer) => buffer.resize(length, Some(false)),
            Self::String(buffer) => buffer.resize(length, Some(String::new())),
            Self::List(buffer) => buffer.resize(length, Some(Vec::new())),
            Self::Midi(buffer) => buffer.resize(length, Some(Vec::new())),
        }
    }

    /// Fills the buffer with an appropriate default value.
    pub fn fill_default(&mut self) {
        match self {
            Self::Sample(buffer) => buffer.fill(Some(0.0)),
            Self::Int(buffer) => buffer.fill(Some(0)),
            Self::Bool(buffer) => buffer.fill(Some(false)),
            Self::String(buffer) => buffer.fill(Some(String::new())),
            Self::List(buffer) => buffer.fill(Some(Vec::new())),
            Self::Midi(buffer) => buffer.fill(Some(Vec::new())),
        }
    }

    /// Clones the signal at the given index.
    #[inline]
    pub fn clone_signal_at(&self, index: usize) -> Signal {
        match self {
            Self::Sample(buffer) => Signal::Sample(buffer[index].unwrap()),
            Self::Int(buffer) => Signal::Int(buffer[index].unwrap()),
            Self::Bool(buffer) => Signal::Bool(buffer[index].unwrap()),
            Self::String(buffer) => Signal::String(buffer[index].clone().unwrap()),
            Self::List(buffer) => Signal::List(buffer[index].clone().unwrap()),
            Self::Midi(buffer) => Signal::Midi(buffer[index].clone().unwrap()),
        }
    }

    /// Copies the contents of `other` into `self`.
    pub fn copy_from(&mut self, other: &Self) {
        match (self, other) {
            (Self::Sample(this), Self::Sample(other)) => {
                this.copy_from_slice(other);
            }
            (Self::Int(this), Self::Int(other)) => {
                this.copy_from_slice(other);
            }
            (Self::Bool(this), Self::Bool(other)) => {
                this.copy_from_slice(other);
            }
            (Self::String(this), Self::String(other)) => {
                this.clone_from_slice(other);
            }
            (Self::List(this), Self::List(other)) => {
                this.clone_from_slice(other);
            }
            (Self::Midi(this), Self::Midi(other)) => {
                this.clone_from_slice(other);
            }
            _ => panic!("Cannot copy buffer of different type"),
        }
    }
}
