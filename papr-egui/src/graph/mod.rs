use node::{Ui, UiNode};
use papr::graph::{Graph, NodeIndex};
use rustc_hash::FxHashMap;

pub mod node;

pub struct UiGraph {
    graph: Graph,
    ui_nodes: FxHashMap<NodeIndex, UiNode>,
}

impl UiGraph {
    // pub fn add_ui(&mut self, ui_node: impl Ui) -> NodeIndex {
    //     let node_index = self.graph.add_processor(ui_node);
    //     self.ui_nodes.insert(node_index, ui_node);
    //     node_index
    // }
}
