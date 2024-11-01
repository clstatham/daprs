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

impl Message for String {}
impl Message for f64 {}
impl Message for i64 {}
impl Message for bool {}

#[derive(Debug, Clone)]
pub struct Nil;
impl Message for Nil {}

impl Display for Nil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "nil")
    }
}

pub fn nil() -> BoxedMessage {
    Box::new(Nil)
}

#[derive(Debug, Clone)]
pub struct Bang;
impl Message for Bang {}

impl Display for Bang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bang")
    }
}

pub fn bang() -> BoxedMessage {
    Box::new(Bang)
}
