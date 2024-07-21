use std::{cell::RefCell, fmt::Debug};

use crate::builtin::*;

use super::{node::Process, Graph, NodeIndex};

#[derive(Clone, Copy)]
pub struct GraphBuilderRef<'a> {
    graph: &'a RefCell<Option<Graph>>,
}

impl<'a> GraphBuilderRef<'a> {
    pub fn add_input(&self) -> GraphBuilderNode<'a> {
        let mut graph = self.graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_input();
        GraphBuilderNode {
            builder: *self,
            index,
        }
    }

    pub fn add_output(&self) -> GraphBuilderNode<'a> {
        let mut graph = self.graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_output();
        GraphBuilderNode {
            builder: *self,
            index,
        }
    }

    pub fn add_processor(&self, processor: impl Process) -> GraphBuilderNode<'a> {
        let mut graph = self.graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_processor(processor);
        GraphBuilderNode {
            builder: *self,
            index,
        }
    }

    pub fn connect(
        &self,
        source: GraphBuilderNode<'a>,
        source_output: u32,
        target: GraphBuilderNode<'a>,
        target_input: u32,
    ) {
        assert!(
            std::ptr::eq(source.builder.graph, self.graph),
            "Cannot connect nodes from different graphs"
        );
        assert!(
            std::ptr::eq(target.builder.graph, self.graph),
            "Cannot connect nodes from different graphs"
        );

        let mut graph = self.graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        graph.connect(source.index, source_output, target.index, target_input);
    }

    pub fn build(&self) -> Graph {
        self.graph
            .take()
            .expect("GraphBuilder has already been finished")
    }
}

#[derive(Default)]
pub struct GraphBuilder {
    graph: RefCell<Option<Graph>>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: RefCell::new(Some(Graph::default())),
        }
    }

    pub fn add_input(&self) -> GraphBuilderNode {
        GraphBuilderRef { graph: &self.graph }.add_input()
    }

    pub fn add_output(&self) -> GraphBuilderNode {
        GraphBuilderRef { graph: &self.graph }.add_output()
    }

    pub fn add_processor(&self, processor: impl Process) -> GraphBuilderNode {
        GraphBuilderRef { graph: &self.graph }.add_processor(processor)
    }

    pub fn connect(
        &self,
        source: GraphBuilderNode,
        source_output: u32,
        target: GraphBuilderNode,
        target_input: u32,
    ) {
        GraphBuilderRef { graph: &self.graph }.connect(source, source_output, target, target_input)
    }

    pub fn build(&self) -> Graph {
        GraphBuilderRef { graph: &self.graph }.build()
    }
}

#[derive(Clone, Copy)]
pub struct GraphBuilderNode<'a> {
    builder: GraphBuilderRef<'a>,
    index: NodeIndex,
}

impl<'a> GraphBuilderNode<'a> {
    pub fn build(self) -> Graph {
        self.builder.build()
    }

    pub fn connect_inputs(self, inputs: impl IntoIterator<Item = (GraphBuilderNode<'a>, u32)>) {
        for (target_input, (source, source_output)) in inputs.into_iter().enumerate() {
            self.builder
                .connect(source, source_output, self, target_input as u32);
        }
    }

    pub fn sin(self) -> GraphBuilderNode<'a> {
        let processor = self.builder.add_processor(math::Sin);
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn cos(self) -> GraphBuilderNode<'a> {
        let processor = self.builder.add_processor(math::Cos);
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn sqrt(self) -> GraphBuilderNode<'a> {
        let processor = self.builder.add_processor(math::Sqrt);
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn exp(self) -> GraphBuilderNode<'a> {
        let processor = self.builder.add_processor(math::Exp);
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn ln(self) -> GraphBuilderNode<'a> {
        let processor = self.builder.add_processor(math::Ln);
        processor.connect_inputs([(self, 0)]);
        processor
    }
}

impl Debug for GraphBuilderNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.index)
    }
}

impl<'a> std::ops::Add<GraphBuilderNode<'a>> for GraphBuilderNode<'a> {
    type Output = GraphBuilderNode<'a>;

    fn add(self, rhs: GraphBuilderNode<'a>) -> GraphBuilderNode<'a> {
        let processor = self.builder.add_processor(math::Add);
        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Sub<GraphBuilderNode<'a>> for GraphBuilderNode<'a> {
    type Output = GraphBuilderNode<'a>;

    fn sub(self, rhs: GraphBuilderNode<'a>) -> GraphBuilderNode<'a> {
        let processor = self.builder.add_processor(math::Sub);
        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Mul<GraphBuilderNode<'a>> for GraphBuilderNode<'a> {
    type Output = GraphBuilderNode<'a>;

    fn mul(self, rhs: GraphBuilderNode<'a>) -> GraphBuilderNode<'a> {
        let processor = self.builder.add_processor(math::Mul);
        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Div<GraphBuilderNode<'a>> for GraphBuilderNode<'a> {
    type Output = GraphBuilderNode<'a>;

    fn div(self, rhs: GraphBuilderNode<'a>) -> GraphBuilderNode<'a> {
        let processor = self.builder.add_processor(math::Div);
        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Rem<GraphBuilderNode<'a>> for GraphBuilderNode<'a> {
    type Output = GraphBuilderNode<'a>;

    fn rem(self, rhs: GraphBuilderNode<'a>) -> GraphBuilderNode<'a> {
        let processor = self.builder.add_processor(math::Rem);
        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::AddAssign<GraphBuilderNode<'a>> for GraphBuilderNode<'a> {
    fn add_assign(&mut self, rhs: GraphBuilderNode<'a>) {
        *self = *self + rhs;
    }
}

impl<'a> std::ops::SubAssign<GraphBuilderNode<'a>> for GraphBuilderNode<'a> {
    fn sub_assign(&mut self, rhs: GraphBuilderNode<'a>) {
        *self = *self - rhs;
    }
}

impl<'a> std::ops::MulAssign<GraphBuilderNode<'a>> for GraphBuilderNode<'a> {
    fn mul_assign(&mut self, rhs: GraphBuilderNode<'a>) {
        *self = *self * rhs;
    }
}

impl<'a> std::ops::DivAssign<GraphBuilderNode<'a>> for GraphBuilderNode<'a> {
    fn div_assign(&mut self, rhs: GraphBuilderNode<'a>) {
        *self = *self / rhs;
    }
}

impl<'a> std::ops::RemAssign<GraphBuilderNode<'a>> for GraphBuilderNode<'a> {
    fn rem_assign(&mut self, rhs: GraphBuilderNode<'a>) {
        *self = *self % rhs;
    }
}
