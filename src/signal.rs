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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Copies the other buffer into this buffer using a memcpy.
    ///
    /// The inner type must be [`Copy`].
    ///
    /// This is faster than using [`Buffer::from_slice`] for large buffers that are already allocated.
    #[inline]
    pub fn copy_from(&mut self, value: impl AsRef<[Option<T>]>)
    where
        T: Copy,
    {
        self.buf.copy_from_slice(value.as_ref());
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

/// A 3-byte MIDI message.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
pub trait Signal: Debug + Send + Sync + PartialEq + 'static {
    /// The type of the signal.
    fn signal_type() -> SignalType;

    /// Converts the signal into an [`AnySignal`].
    fn into_signal(self) -> AnySignal;

    /// Attempts to convert an [`AnySignal`] into the signal type.
    fn try_from_signal(signal: AnySignal) -> Option<Self>
    where
        Self: Sized;

    /// Attempts to convert an [`AnySignal`] into the signal type.
    fn try_from_signal_ref(signal: &AnySignal) -> Option<&Option<Self>>
    where
        Self: Sized;

    /// Attempts to convert a mutable [`AnySignal`] into a mutable signal of the signal type.
    fn try_from_signal_mut(signal: &mut AnySignal) -> Option<&mut Option<Self>>
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

impl Signal for Float {
    fn signal_type() -> SignalType {
        SignalType::Float
    }

    #[inline]
    fn into_signal(self) -> AnySignal {
        AnySignal::Float(Some(self))
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Float(sample) => sample,
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_ref(signal: &AnySignal) -> Option<&Option<Self>>
    where
        Self: Sized,
    {
        match signal {
            AnySignal::Float(sample) => Some(sample),
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_mut(signal: &mut AnySignal) -> Option<&mut Option<Self>>
    where
        Self: Sized,
    {
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
    fn signal_type() -> SignalType {
        SignalType::Bool
    }

    #[inline]
    fn into_signal(self) -> AnySignal {
        AnySignal::Bool(Some(self))
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Bool(bool) => bool,
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_ref(signal: &AnySignal) -> Option<&Option<Self>>
    where
        Self: Sized,
    {
        match signal {
            AnySignal::Bool(bool) => Some(bool),
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_mut(signal: &mut AnySignal) -> Option<&mut Option<Self>>
    where
        Self: Sized,
    {
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
    fn signal_type() -> SignalType {
        SignalType::Int
    }

    #[inline]
    fn into_signal(self) -> AnySignal {
        AnySignal::Int(Some(self))
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Int(int) => int,
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_ref(signal: &AnySignal) -> Option<&Option<Self>>
    where
        Self: Sized,
    {
        match signal {
            AnySignal::Int(int) => Some(int),
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_mut(signal: &mut AnySignal) -> Option<&mut Option<Self>>
    where
        Self: Sized,
    {
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
    fn signal_type() -> SignalType {
        SignalType::String
    }

    #[inline]
    fn into_signal(self) -> AnySignal {
        AnySignal::String(Some(self))
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::String(string) => string,
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_ref(signal: &AnySignal) -> Option<&Option<Self>>
    where
        Self: Sized,
    {
        match signal {
            AnySignal::String(string) => Some(string),
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_mut(signal: &mut AnySignal) -> Option<&mut Option<Self>>
    where
        Self: Sized,
    {
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

impl Signal for SignalBuffer {
    fn signal_type() -> SignalType {
        SignalType::List {
            size: None,
            element_type: None,
        }
    }

    #[inline]
    fn into_signal(self) -> AnySignal {
        AnySignal::List(Some(self))
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::List(list) => list,
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_ref(signal: &AnySignal) -> Option<&Option<Self>>
    where
        Self: Sized,
    {
        match signal {
            AnySignal::List(list) => Some(list),
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_mut(signal: &mut AnySignal) -> Option<&mut Option<Self>>
    where
        Self: Sized,
    {
        match signal {
            AnySignal::List(list) => Some(list),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        buffer.as_buffer()
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        buffer.as_buffer_mut()
    }
}

impl Signal for MidiMessage {
    fn signal_type() -> SignalType {
        SignalType::Midi
    }

    #[inline]
    fn into_signal(self) -> AnySignal {
        AnySignal::Midi(Some(self))
    }

    #[inline]
    fn try_from_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Midi(midi) => midi,
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_ref(signal: &AnySignal) -> Option<&Option<Self>>
    where
        Self: Sized,
    {
        match signal {
            AnySignal::Midi(midi) => Some(midi),
            _ => None,
        }
    }

    #[inline]
    fn try_from_signal_mut(signal: &mut AnySignal) -> Option<&mut Option<Self>>
    where
        Self: Sized,
    {
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
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnySignal {
    /// No signal. The inner value is the expected signal type.
    None(SignalType),

    /// A floating-point value.
    Float(Option<Float>),

    /// An integer.
    Int(Option<i64>),

    /// A boolean.
    Bool(Option<bool>),

    /// A string.
    String(Option<String>),

    /// A list of signals.
    List(Option<SignalBuffer>),

    /// A MIDI message.
    Midi(Option<MidiMessage>),
}

impl AnySignal {
    /// Creates a new signal of the given type with no value.
    pub fn default_of_type(type_: &SignalType) -> Self {
        match type_ {
            SignalType::Float => AnySignal::Float(None),
            SignalType::Int => AnySignal::Int(None),
            SignalType::Bool => AnySignal::Bool(None),
            SignalType::String => AnySignal::String(None),
            SignalType::List { .. } => AnySignal::List(None),
            SignalType::Midi => AnySignal::Midi(None),
        }
    }

    /// Creates a new signal of the given type with no value.
    pub const fn new_none(type_: SignalType) -> Self {
        Self::None(type_)
    }

    /// Creates a new floating-point signal.
    pub const fn new_float(value: Float) -> Self {
        Self::Float(Some(value))
    }

    /// Creates a new integer signal.
    pub const fn new_int(value: i64) -> Self {
        Self::Int(Some(value))
    }

    /// Creates a new boolean signal.
    pub const fn new_bool(value: bool) -> Self {
        Self::Bool(Some(value))
    }

    /// Creates a new string signal.
    pub fn new_string(value: impl Into<String>) -> Self {
        Self::String(Some(value.into()))
    }

    /// Creates a new buffer signal.
    pub fn new_buffer(value: impl Into<SignalBuffer>) -> Self {
        Self::List(Some(value.into()))
    }

    /// Creates a new MIDI signal.
    pub fn new_midi(value: impl Into<MidiMessage>) -> Self {
        Self::Midi(Some(value.into()))
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

    /// Returns `true` if the signal is of the given type.
    pub fn is_type<T: Signal>(&self) -> bool {
        self.signal_type() == T::signal_type()
    }

    /// Returns the floating-point value if the signal is a float, without casting.
    #[inline]
    pub fn as_float(&self) -> Option<Float> {
        match self {
            Self::Float(float) => *float,
            _ => None,
        }
    }

    /// Returns the integer value if the signal is an integer, without casting.
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(int) => *int,
            _ => None,
        }
    }

    /// Returns the boolean value if the signal is a boolean, without casting.
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(bool) => *bool,
            _ => None,
        }
    }

    /// Returns the string value if the signal is a string, without casting.
    #[inline]
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(string) => string.as_ref(),
            _ => None,
        }
    }

    /// Returns the buffer value if the signal is a buffer, without casting.
    #[inline]
    pub fn as_buffer(&self) -> Option<&SignalBuffer> {
        match self {
            Self::List(buf) => buf.as_ref(),
            _ => None,
        }
    }

    /// Returns the MIDI message if the signal is a MIDI message, without casting.
    #[inline]
    pub fn as_midi(&self) -> Option<&MidiMessage> {
        match self {
            Self::Midi(midi) => midi.as_ref(),
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
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::None(type_) => type_.clone(),
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::String(_) => SignalType::String,
            Self::List(l) => SignalType::List {
                size: l.as_ref().and_then(|l| {
                    l.as_buffer()
                        .and_then(|b| b.iter().find_map(|s| s.as_ref()).map(|s| s.len()))
                }),
                element_type: l
                    .as_ref()
                    .and_then(|l| {
                        l.as_buffer()
                            .and_then(|b| b.iter().find_map(|s| s.as_ref()))
                    })
                    .map(|s| s.type_())
                    .map(Box::new),
            },
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
    pub fn cast<T: Signal>(&self) -> Option<T> {
        if self.signal_type() == T::signal_type() {
            T::try_from_signal(self.clone())
        } else {
            match (self, T::signal_type()) {
                (Self::None(_), _) => None,

                // float <-> int
                (Self::Float(float), SignalType::Int) => {
                    T::try_from_signal(AnySignal::Int(float.map(|f| f as i64)))
                }
                (Self::Int(int), SignalType::Float) => {
                    T::try_from_signal(AnySignal::Float(int.map(|i| i as Float)))
                }

                // float <-> bool
                (Self::Float(float), SignalType::Bool) => {
                    T::try_from_signal(AnySignal::Bool(float.map(|f| f != 0.0)))
                }
                (Self::Bool(bool), SignalType::Float) => {
                    T::try_from_signal(AnySignal::Float(bool.map(|b| if b { 1.0 } else { 0.0 })))
                }

                // int <-> bool
                (Self::Int(int), SignalType::Bool) => {
                    T::try_from_signal(AnySignal::Bool(int.map(|i| i != 0)))
                }
                (Self::Bool(bool), SignalType::Int) => {
                    T::try_from_signal(AnySignal::Int(bool.map(|b| if b { 1 } else { 0 })))
                }

                // string <-> float
                (Self::String(string), SignalType::Float) => T::try_from_signal(AnySignal::Float(
                    string.as_ref().and_then(|s| s.parse().ok()),
                )),
                (Self::Float(float), SignalType::String) => {
                    T::try_from_signal(AnySignal::String(float.map(|f| f.to_string())))
                }

                // string <-> int
                (Self::String(string), SignalType::Int) => {
                    T::try_from_signal(AnySignal::Int(string.as_ref().and_then(|s| s.parse().ok())))
                }
                (Self::Int(int), SignalType::String) => {
                    T::try_from_signal(AnySignal::String(int.map(|i| i.to_string())))
                }

                _ => None,
            }
        }
    }

    /// Attempts to extract the signal as the given signal type.
    pub fn as_type<T: Signal>(&self) -> Option<&Option<T>> {
        if self.signal_type() == T::signal_type() {
            T::try_from_signal_ref(self)
        } else {
            None
        }
    }

    /// Attempts to mutably extract the signal as the given signal type.
    pub fn as_type_mut<T: Signal>(&mut self) -> Option<&mut Option<T>> {
        if self.signal_type() == T::signal_type() {
            T::try_from_signal_mut(self)
        } else {
            None
        }
    }
}

/// A signal type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalType {
    /// A floating-point signal.
    Float,

    /// An integer signal.
    Int,

    /// A boolean signal.
    Bool,

    /// A string signal.
    String,

    /// A list signal.
    List {
        /// The size of the list, if known.
        size: Option<usize>,
        /// The type of the elements in the list, if known.
        element_type: Option<Box<SignalType>>,
    },

    /// A MIDI signal.
    Midi,
}

impl SignalType {
    /// Returns `true` if the signal type is compatible with the other signal type.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Float, Self::Float) => true,
            (Self::Int, Self::Int) => true,
            (Self::Bool, Self::Bool) => true,
            (Self::String, Self::String) => true,
            (
                Self::List {
                    size: None,
                    element_type: None,
                },
                Self::List {
                    size: None,
                    element_type: None,
                },
            ) => true,
            (Self::Midi, Self::Midi) => true,
            (
                Self::List {
                    element_type: a_type,
                    ..
                },
                Self::List {
                    element_type: b_type,
                    ..
                },
            ) => match (a_type, b_type) {
                (Some(a), Some(b)) => a.is_compatible_with(b),
                _ => true,
            },
            _ => false,
        }
    }
}

/// A buffer of signals that can hold any signal type.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalBuffer {
    /// A buffer of floating-point signals.
    Float(Buffer<Float>),

    /// A buffer of integer signals.
    Int(Buffer<i64>),

    /// A buffer of boolean signals.
    Bool(Buffer<bool>),

    /// A buffer of string signals.
    String(Buffer<String>),

    /// A buffer of list signals.
    List(Buffer<SignalBuffer>),

    /// A buffer of MIDI signals.
    Midi(Buffer<MidiMessage>),
}

impl SignalBuffer {
    /// Creates a new buffer of the given type with the given length filled with `None`.
    pub fn new_of_type(type_: &SignalType, length: usize) -> Self {
        match type_ {
            SignalType::Float => Self::Float(Buffer::zeros(length)),
            SignalType::Int => Self::Int(Buffer::zeros(length)),
            SignalType::Bool => Self::Bool(Buffer::zeros(length)),
            SignalType::String => Self::String(Buffer::zeros(length)),
            SignalType::List { size, element_type } => match (size, element_type) {
                (Some(size), Some(element_type)) => {
                    let element_type = *element_type.clone();
                    let buf =
                        Buffer::from_slice(&vec![
                            SignalBuffer::new_of_type(&element_type, *size);
                            length
                        ]);
                    Self::List(buf)
                }
                _ => Self::List(Buffer::zeros(length)),
            },
            SignalType::Midi => Self::Midi(Buffer::zeros(length)),
        }
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
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::String(_) => SignalType::String,
            Self::List(l) => {
                let size = l.iter().find_map(|s| s.as_ref()).map(|s| s.len());
                let element_type = l
                    .iter()
                    .find_map(|s| s.as_ref())
                    .map(|s| s.type_())
                    .map(Box::new);
                SignalType::List { size, element_type }
            }
            Self::Midi(_) => SignalType::Midi,
        }
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

    /// Returns a reference to the buffer as a buffer of buffer signals, if it is a buffer buffer.
    ///
    /// This is not a typo.
    #[inline]
    pub fn as_buffer(&self) -> Option<&Buffer<SignalBuffer>> {
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

    /// Returns a mutable reference to the buffer as a buffer of buffer signals, if it is a buffer buffer.
    ///
    /// This is not a typo.
    #[inline]
    pub fn as_buffer_mut(&mut self) -> Option<&mut Buffer<SignalBuffer>> {
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
    pub fn as_type_mut<S: Signal>(&mut self) -> Option<&mut Buffer<S>> {
        S::try_convert_buffer_mut(self)
    }

    /// Returns the length of the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
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
            (Self::Float(buffer), AnySignal::Float(value)) => buffer.buf.resize(length, value),
            (Self::Int(buffer), AnySignal::Int(value)) => buffer.buf.resize(length, value),
            (Self::Bool(buffer), AnySignal::Bool(value)) => buffer.buf.resize(length, value),
            (Self::String(buffer), AnySignal::String(value)) => buffer.buf.resize(length, value),
            (Self::List(buffer), AnySignal::List(value)) => buffer.buf.resize(length, value),
            (Self::Midi(buffer), AnySignal::Midi(value)) => buffer.buf.resize(length, value),
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
            (Self::Float(buffer), AnySignal::Float(value)) => buffer.fill(value),
            (Self::Int(buffer), AnySignal::Int(value)) => buffer.fill(value),
            (Self::Bool(buffer), AnySignal::Bool(value)) => buffer.fill(value),
            (Self::String(buffer), AnySignal::String(value)) => buffer.fill(value),
            (Self::List(buffer), AnySignal::List(value)) => buffer.fill(value),
            (Self::Midi(buffer), AnySignal::Midi(value)) => buffer.fill(value),
            _ => panic!("Cannot fill buffer with value of different type"),
        }
    }

    /// Resizes the buffer to the given length, filling the new elements with `None`.
    pub fn resize_default(&mut self, length: usize) {
        match self {
            Self::Float(buffer) => buffer.buf.resize(length, None),
            Self::Int(buffer) => buffer.buf.resize(length, None),
            Self::Bool(buffer) => buffer.buf.resize(length, None),
            Self::String(buffer) => buffer.buf.resize(length, None),
            Self::List(buffer) => buffer.buf.resize(length, None),
            Self::Midi(buffer) => buffer.buf.resize(length, None),
        }
    }

    /// Resizes the buffer based on the given type hint.
    pub fn resize_with_hint(&mut self, length: usize, type_hint: &SignalType) {
        let type_ = self.type_();
        if type_.is_compatible_with(type_hint) {
            self.resize_default(length);
        } else {
            *self = Self::new_of_type(type_hint, length);
        }
    }

    /// Fills the buffer with `None`.
    pub fn fill_default(&mut self) {
        match self {
            Self::Float(buffer) => buffer.fill(None),
            Self::Int(buffer) => buffer.fill(None),
            Self::Bool(buffer) => buffer.fill(None),
            Self::String(buffer) => buffer.fill(None),
            Self::List(buffer) => buffer.fill(None),
            Self::Midi(buffer) => buffer.fill(None),
        }
    }

    /// Fills the buffer based on the given type hint.
    pub fn fill_with_hint(&mut self, type_hint: &SignalType) {
        let type_ = self.type_();
        if type_.is_compatible_with(type_hint) {
            self.fill_default();
        } else {
            *self = Self::new_of_type(type_hint, self.len());
        }
    }

    /// Returns a mutable reference to the signal at the given index.
    pub fn get_mut<S: Signal>(&mut self, index: usize) -> Option<&mut Option<S>> {
        S::try_convert_buffer_mut(self)?.get_mut(index)
    }

    /// Clones the given signal and stores it at the given index.
    /// Returns `true` if the signal was set successfully.
    #[cfg_attr(feature = "profiling", inline(never))]
    #[cfg_attr(not(feature = "profiling"), inline)]
    pub fn set<S: Signal + Clone>(&mut self, index: usize, value: &Option<S>) -> bool {
        if let Some(buf) = S::try_convert_buffer_mut(self) {
            let slot = buf.get_mut(index).unwrap();
            slot.clone_from(value); // `clone_from` is used to possibly avoid cloning the value twice
            true
        } else {
            false
        }
    }

    /// Clones the given signal and stores it at the given index, but only if the values are not equal.
    /// Returns `true` if the signal was set successfully.
    #[cfg_attr(feature = "profiling", inline(never))]
    #[cfg_attr(not(feature = "profiling"), inline)]
    pub fn set_if_ne<S: Signal + Clone>(&mut self, index: usize, value: &Option<S>) -> bool {
        if let Some(buf) = S::try_convert_buffer_mut(self) {
            let slot = buf.get_mut(index).unwrap();
            if slot != value {
                slot.clone_from(value); // `clone_from` is used to possibly avoid cloning the value twice
                return true;
            }
        }
        false
    }

    /// Clones the given signal and stores it as `Some` at the given index, but only if the values are not equal.
    /// Returns `true` if the signal was set successfully.
    #[cfg_attr(feature = "profiling", inline(never))]
    #[cfg_attr(not(feature = "profiling"), inline)]
    pub fn set_some_if_ne<S: Signal + Clone>(&mut self, index: usize, value: &S) -> bool {
        if let Some(buf) = S::try_convert_buffer_mut(self) {
            let slot = buf.get_mut(index).unwrap();
            if slot.as_ref() != Some(value) {
                *slot = Some(value.clone());
                return true;
            }
        }
        false
    }

    /// Clones the contents of the other buffer into this buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer types do not match.
    pub fn clone_from(&mut self, other: &Self) {
        match (self, other) {
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

    /// Copies the contents of the other buffer into this buffer using a memcpy.
    ///
    /// # Panics
    ///
    /// Panics if the buffer types do not match, or if the types are not `Copy`.
    pub fn copy_from(&mut self, other: &Self) {
        match (self, other) {
            (Self::Float(this), Self::Float(other)) => {
                this.copy_from(other);
            }
            (Self::Int(this), Self::Int(other)) => {
                this.copy_from(other);
            }
            (Self::Bool(this), Self::Bool(other)) => {
                this.copy_from(other);
            }
            (Self::Midi(this), Self::Midi(other)) => {
                this.copy_from(other);
            }
            (Self::String(_), Self::String(_)) => {
                panic!("Cannot copy string buffer; use `clone_from` instead");
            }
            (Self::List(_), Self::List(_)) => {
                panic!("Cannot copy buffer buffer; use `clone_from` instead");
            }
            _ => panic!("Cannot copy buffer of different type"),
        }
    }
}

impl FromIterator<Float> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = Float>>(iter: T) -> Self {
        let iter = iter.into_iter().map(Some);
        Self::Float(Buffer {
            buf: iter.collect(),
        })
    }
}

impl FromIterator<i64> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = i64>>(iter: T) -> Self {
        let iter = iter.into_iter().map(Some);
        Self::Int(Buffer {
            buf: iter.collect(),
        })
    }
}

impl FromIterator<bool> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        let iter = iter.into_iter().map(Some);
        Self::Bool(Buffer {
            buf: iter.collect(),
        })
    }
}

impl FromIterator<String> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let iter = iter.into_iter().map(Some);
        Self::String(Buffer {
            buf: iter.collect(),
        })
    }
}

impl FromIterator<SignalBuffer> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = SignalBuffer>>(iter: T) -> Self {
        let iter = iter.into_iter().map(Some);
        Self::List(Buffer {
            buf: iter.collect(),
        })
    }
}

impl FromIterator<MidiMessage> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = MidiMessage>>(iter: T) -> Self {
        let iter = iter.into_iter().map(Some);
        Self::Midi(Buffer {
            buf: iter.collect(),
        })
    }
}
