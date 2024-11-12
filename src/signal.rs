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

/// An owning, fixed-length array of signal data.
/// This type implements [`Deref`] and [`DerefMut`], so it can be indexed and iterated over just like a normal slice.
/// It can also be [`collected`](std::iter::Iterator::collect) from an iterator of [`Sample`]s.
#[derive(PartialEq, Clone)]
pub struct Buffer<T: SignalData> {
    buf: Vec<Option<T>>,
}

impl<T: SignalData> Debug for Buffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.buf.iter()).finish()
    }
}

impl<T: SignalData> Buffer<T> {
    /// Creates a new buffer filled with zeros.
    #[inline]
    pub fn zeros(length: usize) -> Self {
        let mut buf = Vec::with_capacity(length);
        for _ in 0..length {
            buf.push(None);
        }
        Buffer { buf }
    }

    /// Clones the given slice into a new buffer.
    #[inline]
    pub fn from_slice(value: &[T]) -> Self {
        Buffer {
            buf: value.iter().map(|v| Some(v.clone())).collect(),
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
            writer.write_sample(sample.unwrap_or_default() as f32)?;
        }
        writer.finalize()?;
        Ok(())
    }

    /// Returns the maximum value in the buffer.
    #[inline]
    pub fn max(&self) -> Sample {
        self.buf
            .iter()
            .flatten()
            .copied()
            .fold(Sample::MIN, |a, b| a.max(b))
    }

    /// Returns the minimum value in the buffer.
    #[inline]
    pub fn min(&self) -> Sample {
        self.buf
            .iter()
            .flatten()
            .copied()
            .fold(Sample::MAX, |a, b| a.min(b))
    }

    /// Returns the sum of all values in the buffer.
    #[inline]
    pub fn sum(&self) -> Sample {
        self.buf.iter().flatten().copied().fold(0.0, |a, b| a + b)
    }

    /// Returns the mean of all values in the buffer.
    #[inline]
    pub fn mean(&self) -> Sample {
        self.sum() / self.len() as Sample
    }

    /// Returns the root mean square of all values in the buffer.
    #[inline]
    pub fn rms(&self) -> Sample {
        self.buf
            .iter()
            .flatten()
            .copied()
            .fold(0.0, |a, b| a + b * b)
    }

    /// Returns the variance of all values in the buffer.
    #[inline]
    pub fn variance(&self) -> Sample {
        let mean = self.mean();
        let sum = self
            .buf
            .iter()
            .flatten()
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

impl<T: SignalData> Deref for Buffer<T> {
    type Target = [Option<T>];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.buf.as_ref()
    }
}

impl<T: SignalData> DerefMut for Buffer<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf.as_mut()
    }
}

impl<T: SignalData> AsRef<[Option<T>]> for Buffer<T> {
    #[inline]
    fn as_ref(&self) -> &[Option<T>] {
        self.buf.as_ref()
    }
}

impl<'a, T: SignalData> IntoIterator for &'a Buffer<T> {
    type Item = &'a Option<T>;
    type IntoIter = std::slice::Iter<'a, Option<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter()
    }
}

impl<'a, T: SignalData> IntoIterator for &'a mut Buffer<T> {
    type Item = &'a mut Option<T>;
    type IntoIter = std::slice::IterMut<'a, Option<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut()
    }
}

/// A trait for types that can be used as signal data.
pub trait SignalData: Clone + Debug + Send + Sync + PartialOrd + PartialEq + 'static {
    /// The kind of signal this type represents.
    const KIND: SignalKind;

    fn into_signal(this: Self) -> Signal;
    fn try_from_signal(signal: Signal) -> Option<Self>;

    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>>;
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>>;
}

impl SignalData for Signal {
    const KIND: SignalKind = SignalKind::Dynamic;

    #[inline]
    fn into_signal(this: Self) -> Signal {
        this
    }

    #[inline]
    fn try_from_signal(signal: Signal) -> Option<Self> {
        Some(signal)
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        buffer.as_dynamic()
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        buffer.as_dynamic_mut()
    }
}

impl SignalData for Sample {
    const KIND: SignalKind = SignalKind::Sample;

    #[inline]
    fn into_signal(this: Self) -> Signal {
        Signal::Sample(this)
    }

    #[inline]
    fn try_from_signal(signal: Signal) -> Option<Self> {
        match signal {
            Signal::Sample(sample) => Some(sample),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        buffer.as_sample()
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        buffer.as_sample_mut()
    }
}

impl SignalData for bool {
    const KIND: SignalKind = SignalKind::Bool;

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
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        buffer.as_bool()
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        buffer.as_bool_mut()
    }
}

impl SignalData for i64 {
    const KIND: SignalKind = SignalKind::Int;

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
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        buffer.as_int()
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        buffer.as_int_mut()
    }
}

impl SignalData for String {
    const KIND: SignalKind = SignalKind::String;

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
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        buffer.as_string()
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        buffer.as_string_mut()
    }
}

impl SignalData for Vec<Signal> {
    const KIND: SignalKind = SignalKind::List;

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
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        buffer.as_list()
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        buffer.as_list_mut()
    }
}

impl SignalData for Vec<u8> {
    const KIND: SignalKind = SignalKind::Midi;

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
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        buffer.as_midi()
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
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

    pub fn cast<T: SignalData>(&self) -> Option<T> {
        if self.kind() == T::KIND {
            T::try_from_signal(self.clone())
        } else {
            match (self, T::KIND) {
                (Self::None(_), _) => None,

                // sample <-> int
                (Self::Sample(sample), SignalKind::Int) => {
                    T::try_from_signal(Signal::Int(*sample as i64))
                }
                (Self::Int(int), SignalKind::Sample) => {
                    T::try_from_signal(Signal::Sample(*int as Sample))
                }

                // sample <-> bool
                (Self::Sample(sample), SignalKind::Bool) => {
                    T::try_from_signal(Signal::Bool(*sample != 0.0))
                }
                (Self::Bool(bool), SignalKind::Sample) => {
                    T::try_from_signal(Signal::Sample(if *bool { 1.0 } else { 0.0 }))
                }

                // int <-> bool
                (Self::Int(int), SignalKind::Bool) => T::try_from_signal(Signal::Bool(*int != 0)),
                (Self::Bool(bool), SignalKind::Int) => {
                    T::try_from_signal(Signal::Int(if *bool { 1 } else { 0 }))
                }

                _ => None,
            }
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Signal> for Sample {
    fn into(self) -> Signal {
        Signal::Sample(self)
    }
}

/// Describes the type of data in a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignalKind {
    /// A signal with any kind of value.
    Dynamic,
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

/// A buffer that can contain signals of any kind.
#[derive(Debug, Clone)]
pub enum SignalBuffer {
    Dynamic(Buffer<Signal>),
    /// A buffer of samples.
    Sample(Buffer<Sample>),
    /// A buffer of integers.
    Int(Buffer<i64>),
    /// A buffer of booleans.
    Bool(Buffer<bool>),
    /// A buffer of strings.
    String(Buffer<String>),
    /// A buffer of lists.
    List(Buffer<Vec<Signal>>),
    /// A buffer of MIDI messages.
    Midi(Buffer<Vec<u8>>),
}

impl SignalBuffer {
    /// Creates a new signal buffer of the given kind and length, filled with zeros.
    pub fn new_of_kind(kind: SignalKind, length: usize) -> Self {
        match kind {
            SignalKind::Dynamic => Self::Dynamic(Buffer::zeros(length)),
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

    /// Creates a new dynamic buffer of size `length`, filled with zeros.
    pub fn new_dynamic(length: usize) -> Self {
        Self::Dynamic(Buffer::zeros(length))
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
            Self::Dynamic(_) => SignalKind::Dynamic,
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

    /// Returns a reference to the dynamic buffer, if this is a dynamic buffer.
    #[inline]
    pub fn as_dynamic(&self) -> Option<&Buffer<Signal>> {
        match self {
            Self::Dynamic(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the sample buffer, if this is a sample buffer.
    #[inline]
    pub fn as_sample(&self) -> Option<&Buffer<Sample>> {
        match self {
            Self::Sample(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the integer buffer, if this is an integer buffer.
    #[inline]
    pub fn as_int(&self) -> Option<&Buffer<i64>> {
        match self {
            Self::Int(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the boolean buffer, if this is a boolean buffer.
    #[inline]
    pub fn as_bool(&self) -> Option<&Buffer<bool>> {
        match self {
            Self::Bool(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the string buffer, if this is a string buffer.
    #[inline]
    pub fn as_string(&self) -> Option<&Buffer<String>> {
        match self {
            Self::String(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the list buffer, if this is a list buffer.
    #[inline]
    pub fn as_list(&self) -> Option<&Buffer<Vec<Signal>>> {
        match self {
            Self::List(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the MIDI buffer, if this is a MIDI buffer.
    #[inline]
    pub fn as_midi(&self) -> Option<&Buffer<Vec<u8>>> {
        match self {
            Self::Midi(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_kind<S: SignalData>(&self) -> Option<&Buffer<S>> {
        S::try_convert_buffer(self)
    }

    /// Returns a mutable reference to the dynamic buffer, if this is a dynamic buffer.
    #[inline]
    pub fn as_dynamic_mut(&mut self) -> Option<&mut Buffer<Signal>> {
        match self {
            Self::Dynamic(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the sample buffer, if this is a sample buffer.
    #[inline]
    pub fn as_sample_mut(&mut self) -> Option<&mut Buffer<Sample>> {
        match self {
            Self::Sample(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the integer buffer, if this is an integer buffer.
    #[inline]
    pub fn as_int_mut(&mut self) -> Option<&mut Buffer<i64>> {
        match self {
            Self::Int(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the boolean buffer, if this is a boolean buffer.
    #[inline]
    pub fn as_bool_mut(&mut self) -> Option<&mut Buffer<bool>> {
        match self {
            Self::Bool(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the string buffer, if this is a string buffer.
    #[inline]
    pub fn as_string_mut(&mut self) -> Option<&mut Buffer<String>> {
        match self {
            Self::String(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the list buffer, if this is a list buffer.
    #[inline]
    pub fn as_list_mut(&mut self) -> Option<&mut Buffer<Vec<Signal>>> {
        match self {
            Self::List(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the MIDI buffer, if this is a MIDI buffer.
    #[inline]
    pub fn as_midi_mut(&mut self) -> Option<&mut Buffer<Vec<u8>>> {
        match self {
            Self::Midi(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_kind_mut<S: SignalData>(&mut self) -> Option<&mut Buffer<S>> {
        S::try_convert_buffer_mut(self)
    }

    /// Returns the length of the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Dynamic(buffer) => buffer.len(),
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
            (Self::Dynamic(buffer), value) => {
                buffer.buf.resize(length, Some(value));
            }
            (Self::Sample(buffer), Signal::Sample(value)) => buffer.buf.resize(length, Some(value)),
            (Self::Int(buffer), Signal::Int(value)) => buffer.buf.resize(length, Some(value)),
            (Self::Bool(buffer), Signal::Bool(value)) => buffer.buf.resize(length, Some(value)),
            (Self::String(buffer), Signal::String(value)) => buffer.buf.resize(length, Some(value)),
            (Self::List(buffer), Signal::List(value)) => buffer.buf.resize(length, Some(value)),
            (Self::Midi(buffer), Signal::Midi(value)) => buffer.buf.resize(length, Some(value)),
            _ => panic!("Cannot resize buffer with value of different type"),
        }
    }

    /// Fills the buffer with the given value.
    pub fn fill(&mut self, value: impl Into<Signal>) {
        let value = value.into();
        match (self, value) {
            (Self::Dynamic(buffer), value) => buffer.fill(Some(value)),
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
            Self::Dynamic(buffer) => buffer.buf.resize(length, None),
            Self::Sample(buffer) => buffer.buf.resize(length, Some(0.0)),
            Self::Int(buffer) => buffer.buf.resize(length, Some(0)),
            Self::Bool(buffer) => buffer.buf.resize(length, Some(false)),
            Self::String(buffer) => buffer.buf.resize(length, Some(String::new())),
            Self::List(buffer) => buffer.buf.resize(length, Some(Vec::new())),
            Self::Midi(buffer) => buffer.buf.resize(length, Some(Vec::new())),
        }
    }

    /// Fills the buffer with an appropriate default value.
    pub fn fill_default(&mut self) {
        match self {
            Self::Dynamic(buffer) => buffer.fill(None),
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
            Self::Dynamic(buffer) => buffer[index].clone().unwrap(),
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
            (Self::Dynamic(this), Self::Dynamic(other)) => {
                this.clone_from_slice(other);
            }
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
