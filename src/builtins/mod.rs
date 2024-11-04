//! Built-in processors and utilities for the audio graph.

pub mod math;
pub mod oscillators;
pub mod storage;
pub mod time;
pub mod util;

pub use math::*;
pub use oscillators::*;
pub use storage::*;
pub use time::*;
pub use util::*;

#[doc(hidden)]
#[macro_export]
macro_rules! add_to_builders {
    ($func:ident, $proc:ty, $doc:literal) => {
        impl $crate::builder::graph_builder::GraphBuilder {
            #[doc = $doc]
            pub fn $func(&self) -> $crate::builder::node_builder::Node {
                self.add_processor(<$proc>::default())
            }
        }
    };
}
