use std::fmt::{Debug, Display};

use downcast_rs::{impl_downcast, DowncastSync};

pub trait Message: DowncastSync + Debug + Display + MessageClone {}
impl_downcast!(sync Message);

pub type BoxedMessage = Box<dyn Message>;

mod sealed {
    pub trait Sealed {}
    impl<T: Clone> Sealed for T {}
}

#[doc(hidden)]
pub trait MessageClone: sealed::Sealed {
    fn clone_boxed(&self) -> Box<dyn Message>;
}

impl<T> MessageClone for T
where
    T: Clone + Message,
{
    fn clone_boxed(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Message> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

#[derive(Debug, Clone)]
pub struct StringMessage(pub String);
impl Message for StringMessage {}

impl StringMessage {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for StringMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for StringMessage {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for StringMessage {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone)]
pub struct NumberMessage(pub f64);
impl Message for NumberMessage {}

impl NumberMessage {
    pub fn new(n: f64) -> Self {
        Self(n)
    }

    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

impl Display for NumberMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f64> for NumberMessage {
    fn from(n: f64) -> Self {
        Self(n)
    }
}

#[derive(Debug, Clone)]
pub struct BoolMessage(pub bool);
impl Message for BoolMessage {}

impl BoolMessage {
    pub fn new(b: bool) -> Self {
        Self(b)
    }

    pub fn as_bool(&self) -> bool {
        self.0
    }
}

impl Display for BoolMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<bool> for BoolMessage {
    fn from(b: bool) -> Self {
        Self(b)
    }
}

pub fn true_() -> BoxedMessage {
    Box::new(BoolMessage::new(true))
}

pub fn false_() -> BoxedMessage {
    Box::new(BoolMessage::new(false))
}

#[derive(Debug, Clone)]
pub struct NilMessage;
impl Message for NilMessage {}

impl Display for NilMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "nil")
    }
}

pub fn nil() -> BoxedMessage {
    Box::new(NilMessage)
}

#[derive(Debug, Clone)]
pub struct BangMessage;
impl Message for BangMessage {}

impl Display for BangMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bang")
    }
}

pub fn bang() -> BoxedMessage {
    Box::new(BangMessage)
}
