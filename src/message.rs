//! A module for working with messages that can be sent between processors.

use std::fmt::{Debug, Display};

use crate::signal::Sample;

/// A message that can be sent between processors.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Message {
    /// A bang message ("do whatever it is you do").
    Bang,
    /// An integer message.
    Int(i64),
    /// A float message.
    Float(Sample),
    /// A string message.
    String(String),
    /// A list of messages.
    List(Vec<Message>),
    /// A MIDI message.
    Midi(Vec<u8>),
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Bang => write!(f, "bang"),
            Message::Int(i) => write!(f, "{}", i),
            Message::Float(x) => write!(f, "{}", x),
            Message::String(s) => write!(f, "{}", s),
            Message::List(list) => {
                write!(f, "[")?;
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Message::Midi(data) => {
                write!(f, "MIDI(")?;
                for (i, byte) in data.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{:02X}", byte)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl Message {
    /// Returns true if the two messages are of the same message type.
    #[inline]
    pub fn is_same_type(&self, other: &Message) -> bool {
        matches!(
            (self, other),
            (Message::Bang, Message::Bang)
                | (Message::Int(_), Message::Int(_))
                | (Message::Float(_), Message::Float(_))
                | (Message::String(_), Message::String(_))
        )
    }

    /// Attempts to convert the message to an integer.
    ///
    /// This does not attempt to *cast* the message to an integer, but rather checks if the message is already `Message::Int`.
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Message::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Attempts to convert the message to a float.
    ///
    /// This does not attempt to *cast* the message to a float, but rather checks if the message is already `Message::Float`.
    #[inline]
    pub fn as_float(&self) -> Option<Sample> {
        match self {
            Message::Float(x) => Some(*x),
            _ => None,
        }
    }

    /// Attempts to convert the message to a string.
    ///
    /// This does not attempt to *cast* the message to a string, but rather checks if the message is already `Message::String`.
    #[inline]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Message::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns true if the message is a bang.
    #[inline]
    pub fn is_bang(&self) -> bool {
        matches!(self, Message::Bang)
    }

    /// Returns true if the message is an integer.
    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, Message::Int(_))
    }

    /// Returns true if the message is a float.
    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Message::Float(_))
    }

    /// Returns true if the message is a string.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Message::String(_))
    }

    /// Returns true if the message is a list.
    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, Message::List(_))
    }

    /// Returns true if the message is a MIDI message.
    #[inline]
    pub fn is_midi(&self) -> bool {
        matches!(self, Message::Midi(_))
    }

    /// Returns true if the message is truthy (can be reasonably interpreted as `true`).
    #[inline]
    pub fn is_truthy(&self) -> bool {
        match self {
            Message::Bang => true,
            Message::Int(i) => *i != 0,
            Message::Float(x) => *x != 0.0,
            Message::String(s) => !s.is_empty(),
            Message::List(list) => !list.is_empty(),
            Message::Midi(data) => !data.is_empty(),
        }
    }

    /// Returns true if the message is falsy (can be reasonably interpreted as `false`).
    #[inline]
    pub fn is_falsy(&self) -> bool {
        !self.is_truthy()
    }

    /// Attempts to cast the message to an integer using whatever method is most appropriate.
    #[inline]
    pub fn cast_to_int(&self) -> Option<i64> {
        match self {
            Message::Int(i) => Some(*i),
            Message::Float(x) => Some(x.round() as i64),
            Message::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Attempts to cast the message to a float using whatever method is most appropriate.
    #[inline]
    pub fn cast_to_float(&self) -> Option<Sample> {
        match self {
            Message::Int(i) => Some(*i as Sample),
            Message::Float(x) => Some(*x),
            Message::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Attempts to cast the message to a string using whatever method is most appropriate.
    #[inline]
    pub fn cast_to_string(&self) -> Option<String> {
        match self {
            Message::Bang => Some("bang".to_string()),
            Message::Int(i) => Some(i.to_string()),
            Message::Float(x) => Some(x.to_string()),
            Message::String(s) => Some(s.to_string()),
            _ => None,
        }
    }
}

impl From<i64> for Message {
    fn from(i: i64) -> Self {
        Message::Int(i)
    }
}

impl From<Sample> for Message {
    fn from(x: Sample) -> Self {
        Message::Float(x)
    }
}

impl From<&str> for Message {
    fn from(s: &str) -> Self {
        Message::String(String::from(s))
    }
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        Message::String(s)
    }
}

impl From<Vec<Message>> for Message {
    fn from(list: Vec<Message>) -> Self {
        Message::List(list)
    }
}

impl From<Vec<u8>> for Message {
    fn from(data: Vec<u8>) -> Self {
        Message::Midi(data)
    }
}
