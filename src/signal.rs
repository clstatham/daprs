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
    pub fn from_slice(value: &[T]) -> Self
    where
        T: Clone,
    {
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

/// A list of signals that are all of the same type.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct List {
    type_: SignalType,
    items: Vec<AnySignal>,
}

impl List {
    /// Creates an empty list of the given type.
    pub fn new(type_: SignalType) -> Self {
        Self {
            type_,
            items: Vec::new(),
        }
    }

    /// Returns the type of the list.
    pub fn type_(&self) -> SignalType {
        self.type_
    }

    /// Returns a slice of the items in the list.
    pub fn items(&self) -> &[AnySignal] {
        &self.items
    }

    /// Returns a mutable slice of the items in the list.
    pub fn items_mut(&mut self) -> &mut [AnySignal] {
        &mut self.items
    }

    /// Appends an item to the end of the list.
    ///
    /// # Panics
    ///
    /// Panics if the item type does not match the list type.
    pub fn push(&mut self, item: AnySignal) {
        assert_eq!(
            item.type_(),
            self.type_,
            "Item type does not match list type"
        );
        self.items.push(item);
    }

    /// Removes the last item from the list and returns it.
    pub fn pop(&mut self) -> Option<AnySignal> {
        self.items.pop()
    }

    /// Inserts an item at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the item type does not match the list type.
    pub fn insert(&mut self, index: usize, item: AnySignal) {
        assert_eq!(
            item.type_(),
            self.type_,
            "Item type does not match list type"
        );
        self.items.insert(index, item);
    }

    /// Removes the item at the given index and returns it.
    pub fn remove(&mut self, index: usize) -> AnySignal {
        self.items.remove(index)
    }

    /// Returns the number of items in the list.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns a reference to the item at the given index.
    pub fn get(&self, index: usize) -> Option<&AnySignal> {
        self.items.get(index)
    }

    /// Returns a mutable reference to the item at the given index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut AnySignal> {
        self.items.get_mut(index)
    }
}

impl<T: Signal> From<Vec<T>> for List {
    fn from(items: Vec<T>) -> Self {
        let type_ = T::TYPE;
        let items = items.into_iter().map(T::into_signal).collect();
        Self { type_, items }
    }
}

/// A 3-byte MIDI message.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct MidiMessage {
    /// The MIDI message data.
    pub data: [u8; 3],
}

impl MidiMessage {
    /// Creates a new MIDI message from the given data.
    pub fn new(data: [u8; 3]) -> Self {
        Self { data }
    }

    /// Returns the status byte of the MIDI message.
    pub fn status(&self) -> u8 {
        self.data[0] & 0xF0
    }

    /// Returns the channel of the MIDI message.
    pub fn channel(&self) -> u8 {
        self.data[0] & 0x0F
    }

    /// Returns the first data byte of the MIDI message.
    pub fn data1(&self) -> u8 {
        self.data[1]
    }

    /// Returns the second data byte of the MIDI message.
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

/// A trait for types that can be stored in a [`Buffer`] and processed by a [`Processor`](crate::processor::Processor).
pub trait Signal: Debug + Send + Sync + PartialOrd + PartialEq + 'static {
    /// The type of the signal.
    const TYPE: SignalType;

    /// Converts the signal into an [`AnySignal`].
    fn into_signal(self) -> AnySignal;

    /// Attempts to convert an [`AnySignal`] into the signal type.
    fn try_from_signal(signal: AnySignal) -> Option<Self>
    where
        Self: Sized;

    /// Attempts to convert a [`SignalBuffer`] into a buffer of the signal type.
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>>
    where
        Self: Sized;

    /// Attempts to convert a mutable [`SignalBuffer`] into a mutable buffer of the signal type.
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>>
    where
        Self: Sized;

    /// Attempts to cast the signal from another signal type.
    fn cast_from<T>(other: &T) -> Option<Self>
    where
        Self: Sized,
        T: Signal + Clone,
    {
        Self::try_from_signal(other.clone().into_signal())
    }
}

impl Signal for AnySignal {
    const TYPE: SignalType = SignalType::Dynamic;

    #[inline]
    fn into_signal(self) -> AnySignal {
        self
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
    fn into_signal(self) -> AnySignal {
        AnySignal::Float(self)
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
        buffer.as_float()
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        buffer.as_sample_mut()
    }
}

impl Signal for bool {
    const TYPE: SignalType = SignalType::Bool;

    #[inline]
    fn into_signal(self) -> AnySignal {
        AnySignal::Bool(self)
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
    fn into_signal(self) -> AnySignal {
        AnySignal::Int(self)
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
    fn into_signal(self) -> AnySignal {
        AnySignal::String(self)
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
    fn into_signal(self) -> AnySignal {
        AnySignal::List(self)
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
    fn into_signal(self) -> AnySignal {
        AnySignal::Midi(self)
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

/// A type that can hold any signal type.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum AnySignal {
    /// No signal. The inner value is the expected signal type.
    None(SignalType),

    /// A floating-point value.
    Float(Float),

    /// An integer.
    Int(i64),

    /// A boolean.
    Bool(bool),

    /// A string.
    String(String),

    /// A list of signals.
    List(List),

    /// A MIDI message.
    Midi(MidiMessage),
}

impl AnySignal {
    /// Creates a new signal of the given type with no value.
    pub const fn new_none(type_: SignalType) -> Self {
        Self::None(type_)
    }

    /// Creates a new floating-point signal.
    pub const fn new_float(value: Float) -> Self {
        Self::Float(value)
    }

    /// Creates a new integer signal.
    pub const fn new_int(value: i64) -> Self {
        Self::Int(value)
    }

    /// Creates a new boolean signal.
    pub const fn new_bool(value: bool) -> Self {
        Self::Bool(value)
    }

    /// Creates a new string signal.
    pub fn new_string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Creates a new list signal.
    pub fn new_list(value: impl Into<List>) -> Self {
        Self::List(value.into())
    }

    /// Creates a new MIDI signal.
    pub fn new_midi(value: impl Into<MidiMessage>) -> Self {
        Self::Midi(value.into())
    }

    /// Returns `true` if the signal is [`None`](AnySignal::None).
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None(_))
    }

    /// Returns `true` if the signal is a floating-point value.
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

    /// Returns the floating-point value if the signal is a float, without casting.
    #[inline]
    pub fn as_float(&self) -> Option<Float> {
        match self {
            Self::Float(sample) => Some(*sample),
            _ => None,
        }
    }

    /// Returns the integer value if the signal is an integer, without casting.
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(int) => Some(*int),
            _ => None,
        }
    }

    /// Returns the boolean value if the signal is a boolean, without casting.
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(bool) => Some(*bool),
            _ => None,
        }
    }

    /// Returns the string value if the signal is a string, without casting.
    #[inline]
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    /// Returns the list value if the signal is a list, without casting.
    #[inline]
    pub fn as_list(&self) -> Option<&List> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

    /// Returns the MIDI message if the signal is a MIDI message, without casting.
    #[inline]
    pub fn as_midi(&self) -> Option<&MidiMessage> {
        match self {
            Self::Midi(midi) => Some(midi),
            _ => None,
        }
    }

    /// Returns `true` if the signal is of the same type as the other signal.
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

    /// Returns the type of the signal.
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

    /// Attempts to cast the signal to the given signal type.
    ///
    /// Currently, the following conversions are supported:
    ///
    /// | From \ To | Float | Int | Bool | String | List | Midi |
    /// |-----------|-------|-----|------|--------|------|------|
    /// | Float     | -     | Yes | Yes  | Yes    | -    | -    |
    /// | Int       | Yes   | -   | Yes  | Yes    | -    | -    |
    /// | Bool      | Yes   | Yes | -    | Yes    | -    | -    |
    /// | String    | Yes   | Yes | Yes  | -      | -    | -    |
    /// | List      | -     | -   | -    | -      | -    | -    |
    /// | Midi      | -     | -   | -    | -      | -    | -    |
    /// | Dynamic   | Yes   | Yes | Yes  | Yes    | Yes  | Yes  |
    pub fn cast<T: Signal>(&self) -> Option<T> {
        if self.type_() == T::TYPE {
            T::try_from_signal(self.clone())
        } else {
            match (self, T::TYPE) {
                (Self::None(_), _) => None,

                // float <-> int
                (Self::Float(float), SignalType::Int) => {
                    T::try_from_signal(AnySignal::Int(*float as i64))
                }
                (Self::Int(int), SignalType::Float) => {
                    T::try_from_signal(AnySignal::Float(*int as Float))
                }

                // float <-> bool
                (Self::Float(float), SignalType::Bool) => {
                    T::try_from_signal(AnySignal::Bool(*float != 0.0))
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

                // string <-> float
                (Self::String(string), SignalType::Float) => {
                    T::try_from_signal(AnySignal::Float(string.parse().ok()?))
                }
                (Self::Float(float), SignalType::String) => {
                    T::try_from_signal(AnySignal::String(float.to_string()))
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

/// A signal type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignalType {
    /// A dynamic signal that can hold any type.
    Dynamic,

    /// A floating-point signal.
    Float,

    /// An integer signal.
    Int,

    /// A boolean signal.
    Bool,

    /// A string signal.
    String,

    /// A list signal.
    List,

    /// A MIDI signal.
    Midi,
}

/// A buffer of signals that can hold any signal type.
#[derive(Debug, Clone)]
pub enum SignalBuffer {
    /// A dynamic buffer that can hold any type of signal.
    Dynamic(Buffer<AnySignal>),

    /// A buffer of floating-point signals.
    Float(Buffer<Float>),

    /// A buffer of integer signals.
    Int(Buffer<i64>),

    /// A buffer of boolean signals.
    Bool(Buffer<bool>),

    /// A buffer of string signals.
    String(Buffer<String>),

    /// A buffer of list signals.
    List(Buffer<List>),

    /// A buffer of MIDI signals.
    Midi(Buffer<MidiMessage>),
}

impl SignalBuffer {
    /// Creates a new buffer of the given type with the given length filled with `None`.
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

    /// Creates a new dynamic buffer with the given length filled with `None`.
    pub fn new_dynamic(length: usize) -> Self {
        Self::Dynamic(Buffer::zeros(length))
    }

    /// Creates a new buffer of floating-point signals with the given length filled with `None`.
    pub fn new_sample(length: usize) -> Self {
        Self::Float(Buffer::zeros(length))
    }

    /// Creates a new buffer of integer signals with the given length filled with `None`.
    pub fn new_int(length: usize) -> Self {
        Self::Int(Buffer::zeros(length))
    }

    /// Creates a new buffer of boolean signals with the given length filled with `None`.
    pub fn new_bool(length: usize) -> Self {
        Self::Bool(Buffer::zeros(length))
    }

    /// Creates a new buffer of string signals with the given length filled with `None`.
    pub fn new_string(length: usize) -> Self {
        Self::String(Buffer::zeros(length))
    }

    /// Creates a new buffer of list signals with the given length filled with `None`.
    pub fn new_list(length: usize) -> Self {
        Self::List(Buffer::zeros(length))
    }

    /// Creates a new buffer of MIDI signals with the given length filled with `None`.
    pub fn new_midi(length: usize) -> Self {
        Self::Midi(Buffer::zeros(length))
    }

    /// Returns the type of the buffer.
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

    /// Returns `true` if the buffer is dynamic and can hold any type of signal.
    pub const fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic(_))
    }

    /// Returns `true` if the buffer is for floating-point signals.
    pub const fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    /// Returns `true` if the buffer is for integer signals.
    pub const fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    /// Returns `true` if the buffer is for boolean signals.
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Returns `true` if the buffer is for string signals.
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns `true` if the buffer is for list signals.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Returns `true` if the buffer is for MIDI signals.
    pub const fn is_midi(&self) -> bool {
        matches!(self, Self::Midi(_))
    }

    /// Returns `true` if the buffer is of the given type.
    pub fn is_type(&self, type_: SignalType) -> bool {
        self.type_() == type_
    }

    /// Returns a reference to the buffer as a dynamic buffer, if it is dynamic.
    #[inline]
    pub fn as_dynamic(&self) -> Option<&Buffer<AnySignal>> {
        match self {
            Self::Dynamic(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the buffer as a buffer of floating-point signals, if it is a float buffer.
    #[inline]
    pub fn as_float(&self) -> Option<&Buffer<Float>> {
        match self {
            Self::Float(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the buffer as a buffer of integer signals, if it is an int buffer.
    #[inline]
    pub fn as_int(&self) -> Option<&Buffer<i64>> {
        match self {
            Self::Int(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the buffer as a buffer of boolean signals, if it is a bool buffer.
    #[inline]
    pub fn as_bool(&self) -> Option<&Buffer<bool>> {
        match self {
            Self::Bool(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the buffer as a buffer of string signals, if it is a string buffer.
    #[inline]
    pub fn as_string(&self) -> Option<&Buffer<String>> {
        match self {
            Self::String(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the buffer as a buffer of list signals, if it is a list buffer.
    #[inline]
    pub fn as_list(&self) -> Option<&Buffer<List>> {
        match self {
            Self::List(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the buffer as a buffer of MIDI signals, if it is a MIDI buffer.
    #[inline]
    pub fn as_midi(&self) -> Option<&Buffer<MidiMessage>> {
        match self {
            Self::Midi(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a reference to the buffer as a buffer of the given signal type, if it is of that type.
    #[inline]
    pub fn as_type<S: Signal>(&self) -> Option<&Buffer<S>> {
        S::try_convert_buffer(self)
    }

    /// Returns a mutable reference to the buffer as a dynamic buffer, if it is dynamic.
    #[inline]
    pub fn as_dynamic_mut(&mut self) -> Option<&mut Buffer<AnySignal>> {
        match self {
            Self::Dynamic(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the buffer as a buffer of floating-point signals, if it is a float buffer.
    #[inline]
    pub fn as_sample_mut(&mut self) -> Option<&mut Buffer<Float>> {
        match self {
            Self::Float(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the buffer as a buffer of integer signals, if it is an int buffer.
    #[inline]
    pub fn as_int_mut(&mut self) -> Option<&mut Buffer<i64>> {
        match self {
            Self::Int(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the buffer as a buffer of boolean signals, if it is a bool buffer.
    #[inline]
    pub fn as_bool_mut(&mut self) -> Option<&mut Buffer<bool>> {
        match self {
            Self::Bool(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the buffer as a buffer of string signals, if it is a string buffer.
    #[inline]
    pub fn as_string_mut(&mut self) -> Option<&mut Buffer<String>> {
        match self {
            Self::String(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the buffer as a buffer of list signals, if it is a list buffer.
    #[inline]
    pub fn as_list_mut(&mut self) -> Option<&mut Buffer<List>> {
        match self {
            Self::List(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the buffer as a buffer of MIDI signals, if it is a MIDI buffer.
    #[inline]
    pub fn as_midi_mut(&mut self) -> Option<&mut Buffer<MidiMessage>> {
        match self {
            Self::Midi(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Returns a mutable reference to the buffer as a buffer of the given signal type, if it is of that type.
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

    /// Resizes the buffer to the given length, filling the new elements with the given value.
    ///
    /// # Panics
    ///
    /// Panics if the value type does not match the buffer type.
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
    ///
    /// # Panics
    ///
    /// Panics if the value type does not match the buffer type.
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

    /// Resizes the buffer to the given length, filling the new elements with `None`.
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

    /// Fills the buffer with `None`.
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

    /// Returns the signal at the given index as an [`AnySignal`].
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

    /// Copies the contents of the other buffer into this buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer types do not match.
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
