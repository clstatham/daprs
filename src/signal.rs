//! Signal types and operations.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::Path,
};

#[cfg(feature = "f32_samples")]
/// The type of samples used in the signal processing system.
pub type Float = f32;
#[cfg(not(feature = "f32_samples"))]
/// The type of samples used in the signal processing system.
pub type Float = f64;

#[cfg(feature = "f32_samples")]
/// The value of π.
pub const PI: Float = std::f32::consts::PI;
/// The value of π.
#[cfg(not(feature = "f32_samples"))]
pub const PI: Float = std::f64::consts::PI;

#[cfg(feature = "f32_samples")]
/// The value of τ (2π).
pub const TAU: Float = std::f32::consts::TAU;
#[cfg(not(feature = "f32_samples"))]
/// The value of τ (2π).
pub const TAU: Float = std::f64::consts::TAU;

/// An owning array of signal data.
/// This type implements [`Deref`] and [`DerefMut`], so it can be indexed and iterated over just like a normal slice.
#[derive(PartialEq, Clone)]
pub struct Buffer<T: Signal> {
    buf: Vec<Option<T>>,
}

impl<T: Signal> Debug for Buffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.buf.iter()).finish()
    }
}

impl<T: Signal> Buffer<T> {
    /// Creates a new buffer filled with `None`.
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

impl Buffer<Float> {
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
    pub fn max(&self) -> Float {
        self.buf
            .iter()
            .flatten()
            .copied()
            .fold(Float::MIN, |a, b| a.max(b))
    }

    /// Returns the minimum value in the buffer.
    #[inline]
    pub fn min(&self) -> Float {
        self.buf
            .iter()
            .flatten()
            .copied()
            .fold(Float::MAX, |a, b| a.min(b))
    }

    /// Returns the sum of all values in the buffer.
    #[inline]
    pub fn sum(&self) -> Float {
        self.buf.iter().flatten().copied().fold(0.0, |a, b| a + b)
    }

    /// Returns the mean of all values in the buffer.
    #[inline]
    pub fn mean(&self) -> Float {
        self.sum() / self.len() as Float
    }

    /// Returns the root mean square of all values in the buffer.
    #[inline]
    pub fn rms(&self) -> Float {
        self.buf
            .iter()
            .flatten()
            .copied()
            .fold(0.0, |a, b| a + b * b)
    }

    /// Returns the variance of all values in the buffer.
    #[inline]
    pub fn variance(&self) -> Float {
        let mean = self.mean();
        let sum = self
            .buf
            .iter()
            .flatten()
            .copied()
            .fold(0.0, |a, b| a + (b - mean) * (b - mean));
        sum / (self.len() - 1) as Float
    }

    /// Returns the standard deviation of all values in the buffer.
    #[inline]
    pub fn stddev(&self) -> Float {
        self.variance().sqrt()
    }
}

impl<T: Signal> Deref for Buffer<T> {
    type Target = [Option<T>];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.buf.as_ref()
    }
}

impl<T: Signal> DerefMut for Buffer<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf.as_mut()
    }
}

impl<T: Signal> AsRef<[Option<T>]> for Buffer<T> {
    #[inline]
    fn as_ref(&self) -> &[Option<T>] {
        self.buf.as_ref()
    }
}

impl<'a, T: Signal> IntoIterator for &'a Buffer<T> {
    type Item = &'a Option<T>;
    type IntoIter = std::slice::Iter<'a, Option<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter()
    }
}

impl<'a, T: Signal> IntoIterator for &'a mut Buffer<T> {
    type Item = &'a mut Option<T>;
    type IntoIter = std::slice::IterMut<'a, Option<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct List {
    type_: SignalType,
    items: Vec<AnySignal>,
}

impl List {
    pub fn new(type_: SignalType) -> Self {
        Self {
            type_,
            items: Vec::new(),
        }
    }

    pub fn type_(&self) -> SignalType {
        self.type_
    }

    pub fn items(&self) -> &[AnySignal] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut [AnySignal] {
        &mut self.items
    }

    pub fn push(&mut self, item: AnySignal) {
        assert_eq!(
            item.type_(),
            self.type_,
            "Item type does not match list type"
        );
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<AnySignal> {
        self.items.pop()
    }

    pub fn insert(&mut self, index: usize, item: AnySignal) {
        assert_eq!(
            item.type_(),
            self.type_,
            "Item type does not match list type"
        );
        self.items.insert(index, item);
    }

    pub fn remove(&mut self, index: usize) -> AnySignal {
        self.items.remove(index)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&AnySignal> {
        self.items.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut AnySignal> {
        self.items.get_mut(index)
    }
}

impl From<Vec<AnySignal>> for List {
    fn from(items: Vec<AnySignal>) -> Self {
        let type_ = items.first().map_or(SignalType::Dynamic, AnySignal::type_);
        Self { type_, items }
    }
}

impl From<Vec<Float>> for List {
    fn from(items: Vec<Float>) -> Self {
        let items = items.into_iter().map(AnySignal::new_sample).collect();
        Self {
            type_: SignalType::Float,
            items,
        }
    }
}

impl From<Vec<i64>> for List {
    fn from(items: Vec<i64>) -> Self {
        let items = items.into_iter().map(AnySignal::new_int).collect();
        Self {
            type_: SignalType::Int,
            items,
        }
    }
}

impl From<Vec<bool>> for List {
    fn from(items: Vec<bool>) -> Self {
        let items = items.into_iter().map(AnySignal::new_bool).collect();
        Self {
            type_: SignalType::Bool,
            items,
        }
    }
}

impl From<Vec<String>> for List {
    fn from(items: Vec<String>) -> Self {
        let items = items.into_iter().map(AnySignal::new_string).collect();
        Self {
            type_: SignalType::String,
            items,
        }
    }
}

impl From<Vec<List>> for List {
    fn from(items: Vec<List>) -> Self {
        let items = items.into_iter().map(AnySignal::new_list).collect();
        Self {
            type_: SignalType::List,
            items,
        }
    }
}

impl From<Vec<MidiMessage>> for List {
    fn from(items: Vec<MidiMessage>) -> Self {
        let items = items.into_iter().map(AnySignal::new_midi).collect();
        Self {
            type_: SignalType::Midi,
            items,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct MidiMessage {
    pub data: [u8; 3],
}

impl MidiMessage {
    pub fn new(data: [u8; 3]) -> Self {
        Self { data }
    }

    pub fn status(&self) -> u8 {
        self.data[0] & 0xF0
    }

    pub fn channel(&self) -> u8 {
        self.data[0] & 0x0F
    }

    pub fn data1(&self) -> u8 {
        self.data[1]
    }

    pub fn data2(&self) -> u8 {
        self.data[2]
    }
}

impl Deref for MidiMessage {
    type Target = [u8; 3];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for MidiMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// A trait for types that can be used as signal data.
pub trait Signal: Clone + Debug + Send + Sync + PartialOrd + PartialEq + 'static {
    /// The type of signal this type represents.
    const TYPE: SignalType;

    /// Converts this value into a signal.
    fn into_signal(this: Self) -> AnySignal;
    /// Tries to convert a signal into this type.
    /// This is not done by casting (see [`AnySignal::cast`] for that), but by checking if the signal is of the correct type and returning `None` if it is not.
    fn try_from_signal(signal: AnySignal) -> Option<Self>;

    /// Tries to convert a buffer into a buffer of this type.
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>>;
    /// Tries to convert a mutable buffer into a mutable buffer of this type.
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>>;
}

impl Signal for AnySignal {
    const TYPE: SignalType = SignalType::Dynamic;

    #[inline]
    fn into_signal(this: Self) -> AnySignal {
        this
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
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

impl Signal for Float {
    const TYPE: SignalType = SignalType::Float;

    #[inline]
    fn into_signal(this: Self) -> AnySignal {
        AnySignal::Float(this)
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Float(sample) => Some(sample),
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

impl Signal for bool {
    const TYPE: SignalType = SignalType::Bool;

    #[inline]
    fn into_signal(this: Self) -> AnySignal {
        AnySignal::Bool(this)
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Bool(bool) => Some(bool),
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

impl Signal for i64 {
    const TYPE: SignalType = SignalType::Int;

    #[inline]
    fn into_signal(this: Self) -> AnySignal {
        AnySignal::Int(this)
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Int(int) => Some(int),
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

impl Signal for String {
    const TYPE: SignalType = SignalType::String;

    #[inline]
    fn into_signal(this: Self) -> AnySignal {
        AnySignal::String(this)
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::String(string) => Some(string),
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

impl Signal for List {
    const TYPE: SignalType = SignalType::List;

    #[inline]
    fn into_signal(this: Self) -> AnySignal {
        AnySignal::List(this)
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::List(list) => Some(list),
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

impl Signal for MidiMessage {
    const TYPE: SignalType = SignalType::Midi;

    #[inline]
    fn into_signal(this: Self) -> AnySignal {
        AnySignal::Midi(this)
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Midi(midi) => Some(midi),
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
pub enum AnySignal {
    /// A signal with no value. The inner [`SignalKind`] specifies the type of signal this would be if it had a value.
    None(SignalType),
    /// A single sample of audio.
    Float(Float),
    /// An integer.
    Int(i64),
    /// A boolean.
    Bool(bool),
    /// A string.
    String(String),
    /// A list.
    List(List),
    /// A MIDI message.
    Midi(MidiMessage),
}

impl AnySignal {
    /// Creates a new signal with the given type.
    pub const fn new_none(type_: SignalType) -> Self {
        Self::None(type_)
    }

    /// Creates a new sample signal with the given value.
    pub const fn new_sample(value: Float) -> Self {
        Self::Float(value)
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
    pub fn new_list(value: impl Into<List>) -> Self {
        Self::List(value.into())
    }

    /// Creates a new MIDI signal with the given value. The value will be cloned.
    pub fn new_midi(value: impl Into<MidiMessage>) -> Self {
        Self::Midi(value.into())
    }

    /// Returns `true` if the signal is `None`.
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None(_))
    }

    /// Returns `true` if the signal is a float.
    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
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

    /// Returns the inner [`Float`], if this is a float signal.
    #[inline]
    pub fn as_float(&self) -> Option<Float> {
        match self {
            Self::Float(sample) => Some(*sample),
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

    /// Returns the inner [`List`], if this is a list signal.
    #[inline]
    pub fn as_list(&self) -> Option<&List> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

    /// Returns the inner [`MidiMessage`], if this is a MIDI signal.
    #[inline]
    pub fn as_midi(&self) -> Option<&MidiMessage> {
        match self {
            Self::Midi(midi) => Some(midi),
            _ => None,
        }
    }

    pub fn is_same_type(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None(type_), Self::None(other_kind)) => type_ == other_kind,
            (Self::Float(_), Self::Float(_)) => true,
            (Self::Int(_), Self::Int(_)) => true,
            (Self::Bool(_), Self::Bool(_)) => true,
            (Self::String(_), Self::String(_)) => true,
            (Self::List(_), Self::List(_)) => true,
            (Self::Midi(_), Self::Midi(_)) => true,
            _ => false,
        }
    }

    pub fn type_(&self) -> SignalType {
        match self {
            Self::None(type_) => *type_,
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::String(_) => SignalType::String,
            Self::List(_) => SignalType::List,
            Self::Midi(_) => SignalType::Midi,
        }
    }

    pub fn cast<T: Signal>(&self) -> Option<T> {
        if self.type_() == T::TYPE {
            T::try_from_signal(self.clone())
        } else {
            match (self, T::TYPE) {
                (Self::None(_), _) => None,

                // sample <-> int
                (Self::Float(sample), SignalType::Int) => {
                    T::try_from_signal(AnySignal::Int(*sample as i64))
                }
                (Self::Int(int), SignalType::Float) => {
                    T::try_from_signal(AnySignal::Float(*int as Float))
                }

                // sample <-> bool
                (Self::Float(sample), SignalType::Bool) => {
                    T::try_from_signal(AnySignal::Bool(*sample != 0.0))
                }
                (Self::Bool(bool), SignalType::Float) => {
                    T::try_from_signal(AnySignal::Float(if *bool { 1.0 } else { 0.0 }))
                }

                // int <-> bool
                (Self::Int(int), SignalType::Bool) => {
                    T::try_from_signal(AnySignal::Bool(*int != 0))
                }
                (Self::Bool(bool), SignalType::Int) => {
                    T::try_from_signal(AnySignal::Int(if *bool { 1 } else { 0 }))
                }

                // string <-> sample
                (Self::String(string), SignalType::Float) => {
                    T::try_from_signal(AnySignal::Float(string.parse().ok()?))
                }
                (Self::Float(sample), SignalType::String) => {
                    T::try_from_signal(AnySignal::String(sample.to_string()))
                }

                // string <-> int
                (Self::String(string), SignalType::Int) => {
                    T::try_from_signal(AnySignal::Int(string.parse().ok()?))
                }
                (Self::Int(int), SignalType::String) => {
                    T::try_from_signal(AnySignal::String(int.to_string()))
                }

                _ => None,
            }
        }
    }
}

impl From<Float> for AnySignal {
    fn from(sample: Float) -> Self {
        Self::Float(sample)
    }
}

impl From<i64> for AnySignal {
    fn from(int: i64) -> Self {
        Self::Int(int)
    }
}

impl From<bool> for AnySignal {
    fn from(bool: bool) -> Self {
        Self::Bool(bool)
    }
}

impl From<String> for AnySignal {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<List> for AnySignal {
    fn from(list: List) -> Self {
        Self::List(list)
    }
}

impl From<MidiMessage> for AnySignal {
    fn from(midi: MidiMessage) -> Self {
        Self::Midi(midi)
    }
}

/// Describes the type of data in a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignalType {
    /// A signal with any type of value.
    Dynamic,
    /// A floating-point value.
    Float,
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

/// A buffer that can contain signals of any type.
#[derive(Debug, Clone)]
pub enum SignalBuffer {
    Dynamic(Buffer<AnySignal>),
    /// A buffer of samples.
    Float(Buffer<Float>),
    /// A buffer of integers.
    Int(Buffer<i64>),
    /// A buffer of booleans.
    Bool(Buffer<bool>),
    /// A buffer of strings.
    String(Buffer<String>),
    /// A buffer of lists.
    List(Buffer<List>),
    /// A buffer of MIDI messages.
    Midi(Buffer<MidiMessage>),
}

impl SignalBuffer {
    /// Creates a new signal buffer of the given type and length, filled with zeros.
    pub fn new_of_kind(type_: SignalType, length: usize) -> Self {
        match type_ {
            SignalType::Dynamic => Self::Dynamic(Buffer::zeros(length)),
            SignalType::Float => Self::Float(Buffer::zeros(length)),
            SignalType::Int => Self::Int(Buffer::zeros(length)),
            SignalType::Bool => Self::Bool(Buffer::zeros(length)),
            SignalType::String => Self::String(Buffer::zeros(length)),
            SignalType::List => Self::List(Buffer::zeros(length)),
            SignalType::Midi => Self::Midi(Buffer::zeros(length)),
        }
    }

    /// Creates a new signal buffer of the given data type and length, filled with zeros.
    pub fn new_of_data_kind<T: Signal>(length: usize) -> Self {
        Self::new_of_kind(T::TYPE, length)
    }

    /// Creates a new dynamic buffer of size `length`, filled with zeros.
    pub fn new_dynamic(length: usize) -> Self {
        Self::Dynamic(Buffer::zeros(length))
    }

    /// Creates a new sample buffer of size `length`, filled with zeros.
    pub fn new_sample(length: usize) -> Self {
        Self::Float(Buffer::zeros(length))
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

    /// Returns the type of signal in the buffer.
    pub fn type_(&self) -> SignalType {
        match self {
            Self::Dynamic(_) => SignalType::Dynamic,
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::String(_) => SignalType::String,
            Self::List(_) => SignalType::List,
            Self::Midi(_) => SignalType::Midi,
        }
    }

    /// Returns `true` if the buffer contains samples.
    pub const fn is_sample(&self) -> bool {
        matches!(self, Self::Float(_))
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

    /// Returns `true` if the buffer contains the given type of signal.
    pub fn is_kind(&self, type_: SignalType) -> bool {
        self.type_() == type_
    }

    /// Returns a reference to the dynamic buffer, if this is a dynamic buffer.
    #[inline]
    pub fn as_dynamic(&self) -> Option<&Buffer<AnySignal>> {
        match self {
            Self::Dynamic(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the sample buffer, if this is a sample buffer.
    #[inline]
    pub fn as_sample(&self) -> Option<&Buffer<Float>> {
        match self {
            Self::Float(buffer) => Some(buffer),
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
    pub fn as_list(&self) -> Option<&Buffer<List>> {
        match self {
            Self::List(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the MIDI buffer, if this is a MIDI buffer.
    #[inline]
    pub fn as_midi(&self) -> Option<&Buffer<MidiMessage>> {
        match self {
            Self::Midi(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_kind<S: Signal>(&self) -> Option<&Buffer<S>> {
        S::try_convert_buffer(self)
    }

    /// Returns a mutable reference to the dynamic buffer, if this is a dynamic buffer.
    #[inline]
    pub fn as_dynamic_mut(&mut self) -> Option<&mut Buffer<AnySignal>> {
        match self {
            Self::Dynamic(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the sample buffer, if this is a sample buffer.
    #[inline]
    pub fn as_sample_mut(&mut self) -> Option<&mut Buffer<Float>> {
        match self {
            Self::Float(buffer) => Some(buffer),
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
    pub fn as_list_mut(&mut self) -> Option<&mut Buffer<List>> {
        match self {
            Self::List(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the MIDI buffer, if this is a MIDI buffer.
    #[inline]
    pub fn as_midi_mut(&mut self) -> Option<&mut Buffer<MidiMessage>> {
        match self {
            Self::Midi(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_kind_mut<S: Signal>(&mut self) -> Option<&mut Buffer<S>> {
        S::try_convert_buffer_mut(self)
    }

    /// Returns the length of the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Dynamic(buffer) => buffer.len(),
            Self::Float(buffer) => buffer.len(),
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
    pub fn resize(&mut self, length: usize, value: impl Into<AnySignal>) {
        let value = value.into();
        match (self, value) {
            (Self::Dynamic(buffer), value) => {
                buffer.buf.resize(length, Some(value));
            }
            (Self::Float(buffer), AnySignal::Float(value)) => {
                buffer.buf.resize(length, Some(value))
            }
            (Self::Int(buffer), AnySignal::Int(value)) => buffer.buf.resize(length, Some(value)),
            (Self::Bool(buffer), AnySignal::Bool(value)) => buffer.buf.resize(length, Some(value)),
            (Self::String(buffer), AnySignal::String(value)) => {
                buffer.buf.resize(length, Some(value))
            }
            (Self::List(buffer), AnySignal::List(value)) => buffer.buf.resize(length, Some(value)),
            (Self::Midi(buffer), AnySignal::Midi(value)) => buffer.buf.resize(length, Some(value)),
            _ => panic!("Cannot resize buffer with value of different type"),
        }
    }

    /// Fills the buffer with the given value.
    pub fn fill(&mut self, value: impl Into<AnySignal>) {
        let value = value.into();
        match (self, value) {
            (Self::Dynamic(buffer), value) => buffer.fill(Some(value)),
            (Self::Float(buffer), AnySignal::Float(value)) => buffer.fill(Some(value)),
            (Self::Int(buffer), AnySignal::Int(value)) => buffer.fill(Some(value)),
            (Self::Bool(buffer), AnySignal::Bool(value)) => buffer.fill(Some(value)),
            (Self::String(buffer), AnySignal::String(value)) => buffer.fill(Some(value)),
            (Self::List(buffer), AnySignal::List(value)) => buffer.fill(Some(value)),
            (Self::Midi(buffer), AnySignal::Midi(value)) => buffer.fill(Some(value)),
            _ => panic!("Cannot fill buffer with value of different type"),
        }
    }

    /// Resizes the buffer to the given length, filling any new elements with an appropriate default value.
    pub fn resize_default(&mut self, length: usize) {
        match self {
            Self::Dynamic(buffer) => buffer.buf.resize(length, None),
            Self::Float(buffer) => buffer.buf.resize(length, None),
            Self::Int(buffer) => buffer.buf.resize(length, None),
            Self::Bool(buffer) => buffer.buf.resize(length, None),
            Self::String(buffer) => buffer.buf.resize(length, None),
            Self::List(buffer) => buffer.buf.resize(length, None),
            Self::Midi(buffer) => buffer.buf.resize(length, None),
        }
    }

    /// Fills the buffer with an appropriate default value.
    pub fn fill_default(&mut self) {
        match self {
            Self::Dynamic(buffer) => buffer.fill(None),
            Self::Float(buffer) => buffer.fill(None),
            Self::Int(buffer) => buffer.fill(None),
            Self::Bool(buffer) => buffer.fill(None),
            Self::String(buffer) => buffer.fill(None),
            Self::List(buffer) => buffer.fill(None),
            Self::Midi(buffer) => buffer.fill(None),
        }
    }

    /// Clones the signal at the given index.
    #[inline]
    pub fn clone_signal_at(&self, index: usize) -> AnySignal {
        match self {
            Self::Dynamic(buffer) => buffer[index].clone().unwrap(),
            Self::Float(buffer) => AnySignal::Float(buffer[index].unwrap()),
            Self::Int(buffer) => AnySignal::Int(buffer[index].unwrap()),
            Self::Bool(buffer) => AnySignal::Bool(buffer[index].unwrap()),
            Self::String(buffer) => AnySignal::String(buffer[index].clone().unwrap()),
            Self::List(buffer) => AnySignal::List(buffer[index].clone().unwrap()),
            Self::Midi(buffer) => AnySignal::Midi(buffer[index].unwrap()),
        }
    }

    /// Copies the contents of `other` into `self`.
    pub fn copy_from(&mut self, other: &Self) {
        match (self, other) {
            (Self::Dynamic(this), Self::Dynamic(other)) => {
                this.clone_from_slice(other);
            }
            (Self::Float(this), Self::Float(other)) => {
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
