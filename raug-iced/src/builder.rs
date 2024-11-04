use std::ops::Deref;

use raug::{
    graph::Graph,
    prelude::{GraphBuilder, Node},
};

use crate::{runtime::IcedRuntime, widgets::Widget};

#[derive(Default)]
pub struct IcedGraphBuilder {
    graph: GraphBuilder,
}

impl Deref for IcedGraphBuilder {
    type Target = GraphBuilder;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

impl IcedGraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_widget<T: Widget>(&self, widget: T) -> (T, Node) {
        let rx = widget.rx();
        let node = self.add_processor(rx);
        (widget, node)
    }

    pub fn build(self) -> Graph {
        self.graph.build()
    }

    pub fn build_runtime<T: Widget>(self, main_widget: T) -> IcedRuntime<T> {
        IcedRuntime::new(self.build(), main_widget)
    }
}
