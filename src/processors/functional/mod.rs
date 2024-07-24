pub mod math;
pub mod osc;

pub use math::*;
pub use osc::*;

#[macro_export]
macro_rules! proc_fn {
    ($name:ident ($graph:ident, $($arg:ident :  $kind:ident),*) $body:expr) => {
        pub fn $name<'g>(
            #[allow(unused)]
            $graph: &'g $crate::graph::builder::GraphBuilder,
            $($arg: impl $crate::graph::builder::IntoNode<'g>),*
        ) -> $crate::graph::builder::Node<'g> {
            $(
                let $arg = $arg.into_node($graph, $crate::sample::SignalKind::$kind);
            )*
            $body
        }
    };
}
