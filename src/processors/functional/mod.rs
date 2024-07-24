pub mod math;
pub mod osc;

pub use math::*;
pub use osc::*;

#[macro_export]
macro_rules! proc_fn {
    ($name:ident ($graph:ident, $($arg:ident),*) $body:expr) => {
        pub fn $name<'g>(
            #[allow(unused)]
            $graph: &'g $crate::graph::builder::GraphBuilder,
            $($arg: $crate::graph::builder::Node<'g>),*
        ) -> $crate::graph::builder::Node<'g> {
            $body
        }
    };
}
