pub mod math;
pub mod osc;

pub use math::*;
pub use osc::*;

#[macro_export]
macro_rules! proc_fn {
    ($name:ident ($graph:ident, $($arg:ident),*) $body:expr) => {
        pub fn $name<'a>(
            #[allow(unused)]
            $graph: &'a $crate::graph::builder::GraphBuilder,
            $($arg: $crate::graph::builder::Node<'a>),*
        ) -> $crate::graph::builder::Node<'a> {
            $body
        }
    };
}
