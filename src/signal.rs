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

/// A list of signals.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct List(Box<[AnySignal]>);

impl List {
    pub fn new<T: Signal>(signals: impl IntoIterator<Item = T>) -> Self {
        Self(
            signals
                .into_iter()
                .map(Signal::into_any_signal)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }

    /// Creates a new empty list.
    pub fn new_of_type(signal_type: SignalType, length: usize) -> Self {
        Self(vec![AnySignal::default_of_type(&signal_type); length].into_boxed_slice())
    }

    /// Creates a new list from a slice of signals.
    pub fn from_slice(signals: &[AnySignal]) -> Self {
        Self(signals.to_vec().into_boxed_slice())
    }

    pub fn signal_type(&self) -> SignalType {
        self.0
            .first()
            .map(AnySignal::signal_type)
            .expect("empty lists are not supported")
    }

    /// Returns the number of signals in the list.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a reference to the signal at the given index.
    pub fn get(&self, index: usize) -> Option<AnySignalRef> {
        self.0.get(index).map(AnySignal::as_ref)
    }

    /// Returns a mutable reference to the signal at the given index.
    pub fn get_mut(&mut self, index: usize) -> Option<AnySignalMut> {
        self.0.get_mut(index).map(AnySignal::as_mut)
    }

    pub fn set(&mut self, index: usize, signal: AnySignalRef) {
        self.0[index].clone_from_ref(signal);
    }

    /// Returns an iterator over the signals in the list.
    pub fn iter(&self) -> impl Iterator<Item = &AnySignal> {
        self.0.iter()
    }

    /// Returns a mutable iterator over the signals in the list.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut AnySignal> {
        self.0.iter_mut()
    }

    /// Returns a slice of the signals in the list.
    pub fn as_slice(&self) -> &[AnySignal] {
        &self.0
    }

    /// Returns a mutable slice of the signals in the list.
    pub fn as_mut_slice(&mut self) -> &mut [AnySignal] {
        &mut self.0
    }

    /// Converts the list into a [`Vec`] of signals.
    pub fn into_vec(self) -> Vec<AnySignal> {
        self.0.into_vec()
    }
}

impl<T: Signal> FromIterator<T> for List {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        List(
            iter.into_iter()
                .map(Signal::into_any_signal)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
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

/// A type that can be stored in a [`Buffer`] and processed by a [`Processor`](crate::processor::Processor).
pub trait Signal: Sized + Debug + Send + Sync + PartialEq + 'static {
    /// The type of the signal.
    fn signal_type() -> SignalType;

    /// Converts the signal into an [`AnySignal`].
    fn into_any_signal(self) -> AnySignal;

    /// Attempts to convert an [`AnySignal`] into the signal type.
    fn try_from_any_signal(signal: AnySignal) -> Option<Self>;

    /// Attempts to convert an [`AnySignal`] into the signal type.
    fn try_from_any_signal_ref(signal: AnySignalRef) -> Option<&Option<Self>>;

    /// Attempts to convert a mutable [`AnySignal`] into a mutable signal of the signal type.
    fn try_from_any_signal_mut(signal: AnySignalMut) -> Result<&mut Option<Self>, AnySignalMut>;

    /// Attempts to convert a [`SignalBuffer`] into a buffer of the signal type.
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>>;

    /// Attempts to convert a mutable [`SignalBuffer`] into a mutable buffer of the signal type.
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>>;
}

macro_rules! impl_signal {
    ($name:ident, $typ:expr, $variant:ident) => {
        impl Signal for $name {
            fn signal_type() -> SignalType {
                $typ
            }

            #[inline]
            fn into_any_signal(self) -> AnySignal {
                AnySignal::$variant(Some(self))
            }

            #[inline]
            fn try_from_any_signal(signal: AnySignal) -> Option<Self> {
                match signal {
                    AnySignal::$variant(s) => s,
                    _ => None,
                }
            }

            #[inline]
            fn try_from_any_signal_ref(signal: AnySignalRef) -> Option<&Option<Self>>
            where
                Self: Sized,
            {
                match signal {
                    AnySignalRef::$variant(s) => Some(s),
                    _ => None,
                }
            }

            #[inline]
            fn try_from_any_signal_mut(
                signal: AnySignalMut,
            ) -> Result<&mut Option<Self>, AnySignalMut>
            where
                Self: Sized,
            {
                match signal {
                    AnySignalMut::$variant(s) => Ok(s),
                    signal => Err(signal),
                }
            }

            #[inline]
            fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
                match buffer {
                    SignalBuffer::$variant(buf) => Some(buf),
                    _ => None,
                }
            }

            #[inline]
            fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
                match buffer {
                    SignalBuffer::$variant(buf) => Some(buf),
                    _ => None,
                }
            }
        }
    };
}

impl_signal!(Float, SignalType::Float, Float);
impl_signal!(bool, SignalType::Bool, Bool);
impl_signal!(i64, SignalType::Int, Int);
impl_signal!(String, SignalType::String, String);
impl_signal!(List, SignalType::List, List);
impl_signal!(MidiMessage, SignalType::Midi, Midi);

/// A type that can hold any signal type.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnySignal {
    /// A floating-point value.
    Float(Option<Float>),

    /// An integer.
    Int(Option<i64>),

    /// A boolean.
    Bool(Option<bool>),

    /// A string.
    String(Option<String>),

    /// A list of signals.
    List(Option<List>),

    /// A MIDI message.
    Midi(Option<MidiMessage>),
}

impl AnySignal {
    /// Creates a new signal of the given type with no value.
    pub fn default_of_type(signal_type: &SignalType) -> Self {
        match signal_type {
            SignalType::Float => AnySignal::Float(None),
            SignalType::Int => AnySignal::Int(None),
            SignalType::Bool => AnySignal::Bool(None),
            SignalType::String => AnySignal::String(None),
            SignalType::List { .. } => AnySignal::List(None),
            SignalType::Midi => AnySignal::Midi(None),
        }
    }

    /// Returns `true` if the signal is `Some`.
    pub fn is_some(&self) -> bool {
        match self {
            Self::Float(float) => float.is_some(),
            Self::Int(int) => int.is_some(),
            Self::Bool(bool) => bool.is_some(),
            Self::String(string) => string.is_some(),
            Self::List(list) => list.is_some(),
            Self::Midi(midi) => midi.is_some(),
        }
    }

    /// Returns `true` if the signal is `None`.
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Returns `true` if the signal is of the given type.
    pub fn is_type<T: Signal>(&self) -> bool {
        self.signal_type() == T::signal_type()
    }

    /// Returns `true` if the signal is of the same type as the other signal.
    pub fn is_same_type(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Float(_), Self::Float(_))
                | (Self::Int(_), Self::Int(_))
                | (Self::Bool(_), Self::Bool(_))
                | (Self::String(_), Self::String(_))
                | (Self::List(_), Self::List(_))
                | (Self::Midi(_), Self::Midi(_))
        )
    }

    /// Returns the type of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
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
    #[inline]
    pub fn cast(&self, target: SignalType) -> Option<Self> {
        if self.signal_type() == target {
            return Some(self.clone());
        }
        match (self, target) {
            (Self::Float(float), SignalType::Int) => float.map(|f| Self::Int(Some(f as i64))),
            (Self::Float(float), SignalType::Bool) => float.map(|f| Self::Bool(Some(f != 0.0))),
            (Self::Float(float), SignalType::String) => {
                float.map(|f| Self::String(Some(f.to_string())))
            }
            (Self::Int(int), SignalType::Float) => int.map(|i| Self::Float(Some(i as Float))),
            (Self::Int(int), SignalType::Bool) => int.map(|i| Self::Bool(Some(i != 0))),
            (Self::Int(int), SignalType::String) => int.map(|i| Self::String(Some(i.to_string()))),
            (Self::Bool(bool), SignalType::Float) => {
                bool.map(|b| Self::Float(Some(if b { 1.0 } else { 0.0 })))
            }
            (Self::Bool(bool), SignalType::Int) => {
                bool.map(|b| Self::Int(Some(if b { 1 } else { 0 })))
            }
            (Self::Bool(bool), SignalType::String) => {
                bool.map(|b| Self::String(Some(b.to_string())))
            }
            (Self::String(string), SignalType::Float) => string
                .as_ref()
                .and_then(|s| s.parse().ok().map(|f| Self::Float(Some(f)))),
            (Self::String(string), SignalType::Int) => string
                .as_ref()
                .and_then(|s| s.parse().ok().map(|i| Self::Int(Some(i)))),
            (Self::String(string), SignalType::Bool) => string
                .as_ref()
                .and_then(|s| s.parse().ok().map(|b| Self::Bool(Some(b)))),
            _ => None,
        }
    }

    #[inline]
    pub fn as_ref(&self) -> AnySignalRef {
        match self {
            Self::Float(float) => AnySignalRef::Float(float),
            Self::Int(int) => AnySignalRef::Int(int),
            Self::Bool(bool) => AnySignalRef::Bool(bool),
            Self::String(string) => AnySignalRef::String(string),
            Self::List(list) => AnySignalRef::List(list),
            Self::Midi(midi) => AnySignalRef::Midi(midi),
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> AnySignalMut {
        match self {
            Self::Float(float) => AnySignalMut::Float(float),
            Self::Int(int) => AnySignalMut::Int(int),
            Self::Bool(bool) => AnySignalMut::Bool(bool),
            Self::String(string) => AnySignalMut::String(string),
            Self::List(list) => AnySignalMut::List(list),
            Self::Midi(midi) => AnySignalMut::Midi(midi),
        }
    }

    /// Attempts to extract the signal as the given signal type.
    #[inline]
    pub fn as_type<T: Signal>(&self) -> Option<&Option<T>> {
        if self.signal_type() == T::signal_type() {
            T::try_from_any_signal_ref(self.as_ref())
        } else {
            None
        }
    }

    /// Attempts to mutably extract the signal as the given signal type.
    #[inline]
    pub fn as_type_mut<T: Signal>(&mut self) -> Option<&mut Option<T>> {
        if self.signal_type() == T::signal_type() {
            T::try_from_any_signal_mut(self.as_mut()).ok()
        } else {
            None
        }
    }

    /// Clones the signal into a new signal.
    ///
    /// # Panics
    ///
    /// Panics if the signal type is a list and the list is not empty.
    #[inline]
    pub fn clone_from_ref(&mut self, other: AnySignalRef) {
        match (self, other) {
            (Self::Float(float), AnySignalRef::Float(other)) => *float = *other,
            (Self::Int(int), AnySignalRef::Int(other)) => *int = *other,
            (Self::Bool(bool), AnySignalRef::Bool(other)) => *bool = *other,
            (Self::String(string), AnySignalRef::String(other)) => string.clone_from(other),
            (Self::List(list), AnySignalRef::List(other)) => list.clone_from(other),
            (Self::Midi(midi), AnySignalRef::Midi(other)) => *midi = *other,
            (this, other) => {
                panic!(
                    "Signal types do not match: {:?} and {:?}",
                    this.signal_type(),
                    other.signal_type()
                );
            }
        }
    }
}

/// A reference to a signal that can hold any signal type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnySignalRef<'a> {
    Float(&'a Option<Float>),
    Int(&'a Option<i64>),
    Bool(&'a Option<bool>),
    String(&'a Option<String>),
    List(&'a Option<List>),
    Midi(&'a Option<MidiMessage>),
}

impl<'a> AnySignalRef<'a> {
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::String(_) => SignalType::String,
            Self::List(_) => SignalType::List,
            Self::Midi(_) => SignalType::Midi,
        }
    }

    #[inline]
    pub fn as_type<T: Signal>(self) -> Option<&'a Option<T>> {
        if self.signal_type() == T::signal_type() {
            T::try_from_any_signal_ref(self)
        } else {
            None
        }
    }

    #[inline]
    pub fn to_owned(self) -> AnySignal {
        match self {
            Self::Float(float) => AnySignal::Float(*float),
            Self::Int(int) => AnySignal::Int(*int),
            Self::Bool(bool) => AnySignal::Bool(*bool),
            Self::String(string) => AnySignal::String(string.clone()),
            Self::List(list) => AnySignal::List(list.clone()),
            Self::Midi(midi) => AnySignal::Midi(*midi),
        }
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        match self {
            Self::Float(float) => float.is_some(),
            Self::Int(int) => int.is_some(),
            Self::Bool(bool) => bool.is_some(),
            Self::String(string) => string.is_some(),
            Self::List(list) => list.is_some(),
            Self::Midi(midi) => midi.is_some(),
        }
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
}

/// A mutable reference to a signal that can hold any signal type.
#[derive(Debug, PartialEq)]
pub enum AnySignalMut<'a> {
    Float(&'a mut Option<Float>),
    Int(&'a mut Option<i64>),
    Bool(&'a mut Option<bool>),
    String(&'a mut Option<String>),
    List(&'a mut Option<List>),
    Midi(&'a mut Option<MidiMessage>),
}

impl<'a> AnySignalMut<'a> {
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::String(_) => SignalType::String,
            Self::List(_) => SignalType::List,
            Self::Midi(_) => SignalType::Midi,
        }
    }

    #[inline]
    pub fn as_type<T: Signal>(self) -> Result<&'a mut Option<T>, Self> {
        if self.signal_type() == T::signal_type() {
            T::try_from_any_signal_mut(self)
        } else {
            Err(self)
        }
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        match self {
            Self::Float(float) => float.is_some(),
            Self::Int(int) => int.is_some(),
            Self::Bool(bool) => bool.is_some(),
            Self::String(string) => string.is_some(),
            Self::List(list) => list.is_some(),
            Self::Midi(midi) => midi.is_some(),
        }
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    #[inline]
    pub fn set_none(self) {
        match self {
            Self::Float(float) => *float = None,
            Self::Int(int) => *int = None,
            Self::Bool(bool) => *bool = None,
            Self::String(string) => *string = None,
            Self::List(list) => *list = None,
            Self::Midi(midi) => *midi = None,
        }
    }

    #[inline]
    pub fn set_as<T: Signal>(self, value: T) -> Result<(), Self> {
        match self.as_type() {
            Ok(signal) => {
                *signal = Some(value);
                Ok(())
            }
            Err(this) => Err(this),
        }
    }

    #[inline]
    pub fn to_owned(self) -> AnySignal {
        match self {
            Self::Float(float) => AnySignal::Float(*float),
            Self::Int(int) => AnySignal::Int(*int),
            Self::Bool(bool) => AnySignal::Bool(*bool),
            Self::String(string) => AnySignal::String(string.clone()),
            Self::List(list) => AnySignal::List(list.clone()),
            Self::Midi(midi) => AnySignal::Midi(*midi),
        }
    }

    #[inline]
    pub fn clone_from_ref(&mut self, other: AnySignalRef) {
        match (self, other) {
            (Self::Float(float), AnySignalRef::Float(other)) => **float = *other,
            (Self::Int(int), AnySignalRef::Int(other)) => **int = *other,
            (Self::Bool(bool), AnySignalRef::Bool(other)) => **bool = *other,
            (Self::String(string), AnySignalRef::String(other)) => string.clone_from(other),
            (Self::List(list), AnySignalRef::List(other)) => list.clone_from(other),
            (Self::Midi(midi), AnySignalRef::Midi(other)) => **midi = *other,
            (this, other) => {
                panic!(
                    "Signal types do not match: {:?} and {:?}",
                    this.signal_type(),
                    other.signal_type()
                );
            }
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
    List,

    /// A MIDI signal.
    Midi,
}

impl SignalType {
    /// Returns `true` if the signal type is compatible with the other signal type.
    #[inline]
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Float, Self::Float)
                | (Self::Int, Self::Int)
                | (Self::Bool, Self::Bool)
                | (Self::String, Self::String)
                | (Self::List, Self::List)
                | (Self::Midi, Self::Midi)
        )
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
    List(Buffer<List>),

    /// A buffer of MIDI signals.
    Midi(Buffer<MidiMessage>),
}

impl SignalBuffer {
    /// Creates a new buffer of the given type with the given length filled with `None`.
    pub fn new_of_type(signal_type: &SignalType, length: usize) -> Self {
        match signal_type {
            SignalType::Float => Self::Float(Buffer::zeros(length)),
            SignalType::Int => Self::Int(Buffer::zeros(length)),
            SignalType::Bool => Self::Bool(Buffer::zeros(length)),
            SignalType::String => Self::String(Buffer::zeros(length)),
            SignalType::List => Self::List(Buffer::zeros(length)),
            SignalType::Midi => Self::Midi(Buffer::zeros(length)),
        }
    }

    /// Returns the type of the buffer.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::String(_) => SignalType::String,
            Self::List(_) => SignalType::List,
            Self::Midi(_) => SignalType::Midi,
        }
    }

    /// Returns `true` if the buffer is of the given type.
    #[inline]
    pub fn is_type(&self, signal_type: SignalType) -> bool {
        self.signal_type() == signal_type
    }

    /// Returns a reference to the buffer as a buffer of the given signal type, if it is of that type.
    #[inline]
    pub fn as_type<S: Signal>(&self) -> Option<&Buffer<S>> {
        S::try_convert_buffer(self)
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
        let signal_type = self.signal_type();
        if signal_type.is_compatible_with(type_hint) {
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
        let signal_type = self.signal_type();
        if signal_type.is_compatible_with(type_hint) {
            self.fill_default();
        } else {
            *self = Self::new_of_type(type_hint, self.len());
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<AnySignalRef> {
        match self {
            Self::Float(buffer) => buffer.get(index).map(AnySignalRef::Float),
            Self::Int(buffer) => buffer.get(index).map(AnySignalRef::Int),
            Self::Bool(buffer) => buffer.get(index).map(AnySignalRef::Bool),
            Self::String(buffer) => buffer.get(index).map(AnySignalRef::String),
            Self::List(buffer) => buffer.get(index).map(AnySignalRef::List),
            Self::Midi(buffer) => buffer.get(index).map(AnySignalRef::Midi),
        }
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<AnySignalMut> {
        match self {
            Self::Float(buffer) => buffer.get_mut(index).map(AnySignalMut::Float),
            Self::Int(buffer) => buffer.get_mut(index).map(AnySignalMut::Int),
            Self::Bool(buffer) => buffer.get_mut(index).map(AnySignalMut::Bool),
            Self::String(buffer) => buffer.get_mut(index).map(AnySignalMut::String),
            Self::List(buffer) => buffer.get_mut(index).map(AnySignalMut::List),
            Self::Midi(buffer) => buffer.get_mut(index).map(AnySignalMut::Midi),
        }
    }

    /// Returns the signal at the given index.
    #[inline]
    pub fn get_as<S: Signal>(&self, index: usize) -> Option<&Option<S>> {
        S::try_convert_buffer(self)?.get(index)
    }

    /// Returns a copy of the signal at the given index.
    #[inline]
    pub fn get_copy_as<S: Signal + Copy>(&self, index: usize) -> Option<S> {
        S::try_convert_buffer(self)?.get(index).copied().flatten()
    }

    /// Returns a mutable reference to the signal at the given index.
    #[inline]
    pub fn get_mut_as<S: Signal>(&mut self, index: usize) -> Option<&mut Option<S>> {
        S::try_convert_buffer_mut(self)?.get_mut(index)
    }

    /// Sets the signal at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the signal type does not match the buffer type.
    #[inline]
    pub fn set(&mut self, index: usize, value: AnySignalRef) {
        match (self, value) {
            (Self::Float(buffer), AnySignalRef::Float(value)) => buffer[index] = *value,
            (Self::Int(buffer), AnySignalRef::Int(value)) => buffer[index] = *value,
            (Self::Bool(buffer), AnySignalRef::Bool(value)) => buffer[index] = *value,
            (Self::String(buffer), AnySignalRef::String(value)) => buffer[index].clone_from(value),
            (Self::List(buffer), AnySignalRef::List(value)) => buffer[index].clone_from(value),
            (Self::Midi(buffer), AnySignalRef::Midi(value)) => buffer[index] = *value,
            (this, value) => {
                panic!(
                    "Cannot set signal of different type: {:?} != {:?}",
                    this.signal_type(),
                    value.signal_type()
                );
            }
        }
    }

    /// Clones the given signal and stores it at the given index.
    /// Returns `true` if the signal was set successfully.
    #[cfg_attr(feature = "profiling", inline(never))]
    #[cfg_attr(not(feature = "profiling"), inline)]
    pub fn set_as<S: Signal + Clone>(&mut self, index: usize, value: &Option<S>) -> bool {
        if let Some(buf) = S::try_convert_buffer_mut(self) {
            let slot = buf.get_mut(index).unwrap();
            slot.clone_from(value); // `clone_from` is used to possibly avoid cloning the value twice
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn set_none(&mut self, index: usize) {
        match self {
            Self::Float(buffer) => buffer[index] = None,
            Self::Int(buffer) => buffer[index] = None,
            Self::Bool(buffer) => buffer[index] = None,
            Self::String(buffer) => buffer[index] = None,
            Self::List(buffer) => buffer[index] = None,
            Self::Midi(buffer) => buffer[index] = None,
        }
    }

    /// Clones the contents of the other buffer into this buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer types do not match.
    #[inline]
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
    #[inline]
    pub fn copy_from(&mut self, other: &Self) {
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
            (Self::Midi(this), Self::Midi(other)) => {
                this.copy_from_slice(other);
            }
            (Self::String(_), Self::String(_)) => {
                panic!("Cannot copy string buffer; use `clone_from` instead");
            }
            (Self::List(_), Self::List(_)) => {
                panic!("Cannot copy list buffer; use `clone_from` instead");
            }
            _ => panic!("Cannot copy buffer of different type"),
        }
    }

    /// Returns an iterator over the signals in the buffer.
    #[inline]
    pub fn iter(&self) -> SignalBufferIter {
        SignalBufferIter {
            buffer: self,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns a mutable iterator over the signals in the buffer.
    #[inline]
    pub fn iter_mut(&mut self) -> SignalBufferIterMut {
        SignalBufferIterMut {
            buffer: self,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

/// An iterator over the signals in a buffer.
pub struct SignalBufferIter<'a> {
    buffer: &'a SignalBuffer,
    index: usize,
    _marker: std::marker::PhantomData<AnySignalRef<'a>>,
}

impl<'a> Iterator for SignalBufferIter<'a> {
    type Item = AnySignalRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buffer.len() {
            let signal = match self.buffer {
                SignalBuffer::Float(buffer) => AnySignalRef::Float(&buffer[self.index]),
                SignalBuffer::Int(buffer) => AnySignalRef::Int(&buffer[self.index]),
                SignalBuffer::Bool(buffer) => AnySignalRef::Bool(&buffer[self.index]),
                SignalBuffer::String(buffer) => AnySignalRef::String(&buffer[self.index]),
                SignalBuffer::List(buffer) => AnySignalRef::List(&buffer[self.index]),
                SignalBuffer::Midi(buffer) => AnySignalRef::Midi(&buffer[self.index]),
            };
            self.index += 1;
            Some(signal)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a SignalBuffer {
    type Item = AnySignalRef<'a>;
    type IntoIter = SignalBufferIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SignalBufferIter {
            buffer: self,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

/// An mutable iterator over the signals in a buffer.
pub struct SignalBufferIterMut<'a> {
    buffer: &'a mut SignalBuffer,
    index: usize,
    _marker: std::marker::PhantomData<AnySignalMut<'a>>,
}

impl<'a> Iterator for SignalBufferIterMut<'a> {
    type Item = AnySignalMut<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buffer.len() {
            // SAFETY:
            // We are borrowing the buffer mutably, so we can safely create a mutable reference to the signal.
            // We are also only creating one mutable reference at a time, so there are no issues with aliasing.
            // The lifetime of the mutable reference is limited to the lifetime of the iterator.
            // This is similar to how `std::slice::IterMut` works.
            unsafe {
                let signal = match self.buffer {
                    SignalBuffer::Float(buffer) => {
                        AnySignalMut::Float(&mut *(&mut buffer[self.index] as *mut Option<Float>))
                    }
                    SignalBuffer::Int(buffer) => {
                        AnySignalMut::Int(&mut *(&mut buffer[self.index] as *mut Option<i64>))
                    }
                    SignalBuffer::Bool(buffer) => {
                        AnySignalMut::Bool(&mut *(&mut buffer[self.index] as *mut Option<bool>))
                    }
                    SignalBuffer::String(buffer) => {
                        AnySignalMut::String(&mut *(&mut buffer[self.index] as *mut Option<String>))
                    }
                    SignalBuffer::List(buffer) => {
                        AnySignalMut::List(&mut *(&mut buffer[self.index] as *mut Option<List>))
                    }
                    SignalBuffer::Midi(buffer) => AnySignalMut::Midi(
                        &mut *(&mut buffer[self.index] as *mut Option<MidiMessage>),
                    ),
                };
                self.index += 1;
                Some(signal)
            }
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a mut SignalBuffer {
    type Item = AnySignalMut<'a>;
    type IntoIter = SignalBufferIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SignalBufferIterMut {
            buffer: self,
            index: 0,
            _marker: std::marker::PhantomData,
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

impl FromIterator<List> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = List>>(iter: T) -> Self {
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
