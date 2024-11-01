use std::fmt::Debug;

use downcast_rs::{impl_downcast, DowncastSync};

pub trait Message: DowncastSync + Debug + MessageClone {}
impl_downcast!(sync Message);

pub type BoxedMessage = Box<dyn Message>;

impl Message for Vec<BoxedMessage> {}

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

#[derive(Debug, Clone)]
pub struct NumberMessage(pub f64);
impl Message for NumberMessage {}

#[derive(Debug, Clone)]
pub struct BoolMessage(pub bool);
impl Message for BoolMessage {}

#[derive(Debug, Clone)]
pub struct NilMessage;
impl Message for NilMessage {}

#[derive(Debug, Clone)]
pub struct BangMessage;
impl Message for BangMessage {}
