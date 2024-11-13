//! Signal types and operations.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::Path,
};

#[cfg(feature = "f32_samples")]
/// The floating-point sample type.
pub type Float = f32;
#[cfg(not(feature = "f32_samples"))]
/// The floating-point sample type.
pub type Float = f64;

#[cfg(feature = "f32_samples")]
/// The value of PI for the floating-point sample type.
pub const PI: Float = std::f32::consts::PI;
/// The value of PI for the floating-point sample type.
#[cfg(not(feature = "f32_samples"))]
pub const PI: Float = std::f64::consts::PI;

#[cfg(feature = "f32_samples")]
/// The value of TAU (2*PI) for the floating-point sample type.
pub const TAU: Float = std::f32::consts::TAU;
#[cfg(not(feature = "f32_samples"))]
/// The value of TAU (2*PI) for the floating-point sample type.
pub const TAU: Float = std::f64::consts::TAU;

/// A contiguous buffer of signals.
///
/// The signals are stored as a [`Vec`] of [`Option<T>`] to allow for missing values.
///
/// This type implements [`Deref`] and [`DerefMut`] so that it can be used as a slice of [`Option<T>`].
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
    /// Creates a new buffer of the given length filled with `None`.
    #[inline]
    pub fn zeros(length: usize) -> Self {
        let mut buf = Vec::with_capacity(length);
        for _ in 0..length {
            buf.push(None);
        }
        Buffer { buf }
    }

    /// Clones the slice into a new buffer. All elements are wrapped in `Some`.
    #[inline]
    pub fn from_slice(value: &[T]) -> Self {
        Buffer {
            buf: value.iter().map(|v| Some(v.clone())).collect(),
        }
    }
}

impl Buffer<Float> {
    /// Loads a buffer from a WAV file.
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

    /// Saves the buffer to a WAV file. [`None`] entries are written as silence.
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

    /// Returns the maximum value in the buffer out of all entries that are [`Some`].
    ///
    /// If the buffer is empty, this returns [`Float::MIN`].
    #[inline]
    pub fn max(&self) -> Float {
        self.buf
            .iter()
            .flatten()
            .copied()
            .fold(Float::MIN, |a, b| a.max(b))
    }

    /// Returns the minimum value in the buffer out of all entries that are [`Some`].
    ///
    /// If the buffer is empty, this returns [`Float::MAX`].
    #[inline]
    pub fn min(&self) -> Float {
        self.buf
            .iter()
            .flatten()
            .copied()
            .fold(Float::MAX, |a, b| a.min(b))
    }

    /// Returns the sum of all entries that are [`Some`].
    ///
    /// If the buffer is empty, this returns `0.0`.
    #[inline]
    pub fn sum(&self) -> Float {
        self.buf.iter().flatten().copied().fold(0.0, |a, b| a + b)
    }

    /// Returns the mean of all entries that are [`Some`].
    ///
    /// # Panics
    ///
    /// Panics if the buffer is empty.
    #[inline]
    pub fn mean(&self) -> Float {
        self.sum() / self.len() as Float
    }

    /// Returns the root mean square of all entries that are [`Some`].
    ///
    /// If the buffer is empty, this returns `0.0`.
    #[inline]
    pub fn rms(&self) -> Float {
        self.buf
            .iter()
            .flatten()
            .copied()
            .fold(0.0, |a, b| a + b * b)
            .sqrt()
    }

    /// Returns the variance of all entries that are [`Some`].
    ///
    /// If the buffer has less than 2 entries, this returns `0.0`.
    #[inline]
    pub fn variance(&self) -> Float {
        if self.len() < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let sum = self
            .buf
            .iter()
            .flatten()
            .copied()
            .fold(0.0, |a, b| a + (b - mean) * (b - mean));
        sum / (self.len() - 1) as Float
    }

    /// Returns the standard deviation of all entries that are [`Some`].
    ///
    /// This is the square root of the variance.
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

pub trait Signal: Clone + Debug + Send + Sync + PartialOrd + PartialEq + 'static {
    const TYPE: SignalType;

    fn into_signal(this: Self) -> AnySignal;

    fn try_from_signal(signal: AnySignal) -> Option<Self>;

    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>>;

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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum AnySignal {
    None(SignalType),

    Float(Float),

    Int(i64),

    Bool(bool),

    String(String),

    List(List),

    Midi(MidiMessage),
}

impl AnySignal {
    pub const fn new_none(type_: SignalType) -> Self {
        Self::None(type_)
    }

    pub const fn new_sample(value: Float) -> Self {
        Self::Float(value)
    }

    pub const fn new_int(value: i64) -> Self {
        Self::Int(value)
    }

    pub const fn new_bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn new_string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn new_list(value: impl Into<List>) -> Self {
        Self::List(value.into())
    }

    pub fn new_midi(value: impl Into<MidiMessage>) -> Self {
        Self::Midi(value.into())
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None(_))
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    #[inline]
    pub fn is_midi(&self) -> bool {
        matches!(self, Self::Midi(_))
    }

    #[inline]
    pub fn as_float(&self) -> Option<Float> {
        match self {
            Self::Float(sample) => Some(*sample),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(int) => Some(*int),
            _ => None,
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(bool) => Some(*bool),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    #[inline]
    pub fn as_list(&self) -> Option<&List> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignalType {
    Dynamic,

    Float,

    Int,

    Bool,

    String,

    List,

    Midi,
}

#[derive(Debug, Clone)]
pub enum SignalBuffer {
    Dynamic(Buffer<AnySignal>),

    Float(Buffer<Float>),

    Int(Buffer<i64>),

    Bool(Buffer<bool>),

    String(Buffer<String>),

    List(Buffer<List>),

    Midi(Buffer<MidiMessage>),
}

impl SignalBuffer {
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

    pub fn new_of_data_kind<T: Signal>(length: usize) -> Self {
        Self::new_of_kind(T::TYPE, length)
    }

    pub fn new_dynamic(length: usize) -> Self {
        Self::Dynamic(Buffer::zeros(length))
    }

    pub fn new_sample(length: usize) -> Self {
        Self::Float(Buffer::zeros(length))
    }

    pub fn new_int(length: usize) -> Self {
        Self::Int(Buffer::zeros(length))
    }

    pub fn new_bool(length: usize) -> Self {
        Self::Bool(Buffer::zeros(length))
    }

    pub fn new_string(length: usize) -> Self {
        Self::String(Buffer::zeros(length))
    }

    pub fn new_list(length: usize) -> Self {
        Self::List(Buffer::zeros(length))
    }

    pub fn new_midi(length: usize) -> Self {
        Self::Midi(Buffer::zeros(length))
    }

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

    pub const fn is_sample(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    pub const fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    pub const fn is_midi(&self) -> bool {
        matches!(self, Self::Midi(_))
    }

    pub fn is_kind(&self, type_: SignalType) -> bool {
        self.type_() == type_
    }

    #[inline]
    pub fn as_dynamic(&self) -> Option<&Buffer<AnySignal>> {
        match self {
            Self::Dynamic(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_sample(&self) -> Option<&Buffer<Float>> {
        match self {
            Self::Float(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int(&self) -> Option<&Buffer<i64>> {
        match self {
            Self::Int(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<&Buffer<bool>> {
        match self {
            Self::Bool(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string(&self) -> Option<&Buffer<String>> {
        match self {
            Self::String(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_list(&self) -> Option<&Buffer<List>> {
        match self {
            Self::List(buffer) => Some(buffer),
            _ => None,
        }
    }

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

    #[inline]
    pub fn as_dynamic_mut(&mut self) -> Option<&mut Buffer<AnySignal>> {
        match self {
            Self::Dynamic(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_sample_mut(&mut self) -> Option<&mut Buffer<Float>> {
        match self {
            Self::Float(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int_mut(&mut self) -> Option<&mut Buffer<i64>> {
        match self {
            Self::Int(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_bool_mut(&mut self) -> Option<&mut Buffer<bool>> {
        match self {
            Self::Bool(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string_mut(&mut self) -> Option<&mut Buffer<String>> {
        match self {
            Self::String(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    pub fn as_list_mut(&mut self) -> Option<&mut Buffer<List>> {
        match self {
            Self::List(buffer) => Some(buffer),
            _ => None,
        }
    }

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

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

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
