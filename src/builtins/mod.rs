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

#[macro_export]
macro_rules! add_to_builders {
    ($func:ident, $proc:ty, $doc:expr) => {
        impl $crate::builder::graph_builder::GraphBuilder {
            #[doc = $doc]
            pub fn $func(&self) -> $crate::builder::node_builder::Node {
                self.add_processor(<$proc>::default())
            }
        }

        impl $crate::builder::static_graph_builder::StaticGraphBuilder {
            #[doc = $doc]
            pub fn $func(&self) -> $crate::builder::static_node_builder::StaticNode {
                self.add_processor(<$proc>::default())
            }
        }
    };
}
