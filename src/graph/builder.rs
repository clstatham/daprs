use std::{cell::RefCell, fmt::Debug};

use crate::{builtin::*, sample::SignalKind};

use super::{
    node::{Process, Processor},
    Graph, NodeIndex,
};

#[derive(Clone, Copy)]
pub struct GraphBuilderRef<'a> {
    graph: &'a RefCell<Option<Graph>>,
}

impl<'a> GraphBuilderRef<'a> {
    pub fn add_input(&self) -> Node<'a> {
        let mut graph = self.graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_input();
        Node {
            builder: *self,
            index,
        }
    }

    pub fn add_output(&self) -> Node<'a> {
        let mut graph = self.graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_output();
        Node {
            builder: *self,
            index,
        }
    }

    pub fn add_processor(&self, processor: impl Process) -> Node<'a> {
        let mut graph = self.graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_processor(processor);
        Node {
            builder: *self,
            index,
        }
    }

    pub fn connect(
        &self,
        source: Node<'a>,
        source_output: u32,
        target: Node<'a>,
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

    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph: RefCell::new(Some(graph)),
        }
    }

    pub fn input(&self) -> Node {
        GraphBuilderRef { graph: &self.graph }.add_input()
    }

    pub fn output(&self) -> Node {
        GraphBuilderRef { graph: &self.graph }.add_output()
    }

    pub fn ar_constant(&self, value: f64) -> Node {
        let processor = math::Constant::ar(value.into());
        GraphBuilderRef { graph: &self.graph }.add_processor(processor)
    }

    pub fn kr_constant(&self, value: f64) -> Node {
        let processor = math::Constant::kr(value.into());
        GraphBuilderRef { graph: &self.graph }.add_processor(processor)
    }

    pub fn processor(&self, processor: impl Process) -> Node {
        GraphBuilderRef { graph: &self.graph }.add_processor(processor)
    }

    pub fn connect(&self, source: Node, source_output: u32, target: Node, target_input: u32) {
        GraphBuilderRef { graph: &self.graph }.connect(source, source_output, target, target_input)
    }

    pub fn build(&self) -> Graph {
        GraphBuilderRef { graph: &self.graph }.build()
    }
}

#[derive(Clone, Copy)]
pub struct Node<'a> {
    builder: GraphBuilderRef<'a>,
    index: NodeIndex,
}

impl<'a> Node<'a> {
    pub fn num_inputs(self) -> usize {
        let graph = self.builder.graph.borrow();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        let node = &graph.digraph[self.index];
        node.inputs().len()
    }

    pub fn num_outputs(self) -> usize {
        let graph = self.builder.graph.borrow();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        let node = &graph.digraph[self.index];
        node.outputs().len()
    }

    pub fn build(self) -> Graph {
        self.builder.build()
    }

    pub fn graph(self) -> GraphBuilderRef<'a> {
        self.builder
    }

    pub fn input_kind(self) -> SignalKind {
        let graph = self.builder.graph.borrow();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        graph.digraph[self.index].input_kind()
    }

    pub fn output_kind(self) -> SignalKind {
        let graph = self.builder.graph.borrow();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        graph.digraph[self.index].output_kind()
    }

    pub fn to_ar(self) -> Node<'a> {
        {
            let graph = self.builder.graph.borrow();
            let graph = graph
                .as_ref()
                .expect("GraphBuilder has already been finished");

            let kind = graph.digraph[self.index].output_kind();
            if kind == SignalKind::Audio {
                return self;
            }
        }

        let processor = self.builder.add_processor(io::Smooth::default());
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn to_kr(self) -> Node<'a> {
        {
            let graph = self.builder.graph.borrow();
            let graph = graph
                .as_ref()
                .expect("GraphBuilder has already been finished");

            let kind = graph.digraph[self.index].output_kind();
            if kind == SignalKind::Control {
                return self;
            }
        }

        let processor = self.builder.add_processor(io::Quantize::default());
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn connect_inputs(self, inputs: impl IntoIterator<Item = (Node<'a>, u32)>) {
        for (target_input, (source, source_output)) in inputs.into_iter().enumerate() {
            if self.input_kind().can_take_as_input(source.output_kind()) {
                self.builder
                    .connect(source, source_output, self, target_input as u32);
            } else {
                panic!(
                    "Cannot connect node of kind {:?} to {:?}",
                    source.output_kind(),
                    self.input_kind()
                );
            }
        }
    }

    pub fn sin(self) -> Node<'a> {
        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Sin::ar()),
            SignalKind::Control => self.builder.add_processor(math::Sin::kr()),
            _ => panic!("Cannot take sin of a node of kind {:?}", self.output_kind()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn cos(self) -> Node<'a> {
        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Cos::ar()),
            SignalKind::Control => self.builder.add_processor(math::Cos::kr()),
            _ => panic!("Cannot take cos of a node of kind {:?}", self.output_kind()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn sqrt(self) -> Node<'a> {
        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Sqrt::ar()),
            SignalKind::Control => self.builder.add_processor(math::Sqrt::kr()),
            _ => panic!(
                "Cannot take sqrt of a node of kind {:?}",
                self.output_kind()
            ),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn exp(self) -> Node<'a> {
        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Exp::ar()),
            SignalKind::Control => self.builder.add_processor(math::Exp::kr()),
            _ => panic!("Cannot take exp of a node of kind {:?}", self.output_kind()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn ln(self) -> Node<'a> {
        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Ln::ar()),
            SignalKind::Control => self.builder.add_processor(math::Ln::kr()),
            _ => panic!("Cannot take ln of a node of kind {:?}", self.output_kind()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }
}

impl Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.index)
    }
}

impl<'a> std::ops::Add<Node<'a>> for Node<'a> {
    type Output = Node<'a>;

    fn add(self, rhs: Node<'a>) -> Node<'a> {
        assert!(
            self.output_kind() == rhs.output_kind(),
            "Cannot add nodes of different signal kinds"
        );

        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Add::ar()),
            SignalKind::Control => self.builder.add_processor(math::Add::kr()),
            _ => panic!("Cannot add nodes of kind {:?}", self.output_kind()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Sub<Node<'a>> for Node<'a> {
    type Output = Node<'a>;

    fn sub(self, rhs: Node<'a>) -> Node<'a> {
        assert!(
            self.output_kind() == rhs.output_kind(),
            "Cannot subtract nodes of different signal kinds"
        );

        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Sub::ar()),
            SignalKind::Control => self.builder.add_processor(math::Sub::kr()),
            _ => panic!("Cannot subtract nodes of kind {:?}", self.output_kind()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Mul<Node<'a>> for Node<'a> {
    type Output = Node<'a>;

    fn mul(self, rhs: Node<'a>) -> Node<'a> {
        assert!(
            self.output_kind() == rhs.output_kind(),
            "Cannot multiply nodes of different signal kinds"
        );

        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Mul::ar()),
            SignalKind::Control => self.builder.add_processor(math::Mul::kr()),
            _ => panic!("Cannot multiply nodes of kind {:?}", self.output_kind()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Div<Node<'a>> for Node<'a> {
    type Output = Node<'a>;

    fn div(self, rhs: Node<'a>) -> Node<'a> {
        assert!(
            self.output_kind() == rhs.output_kind(),
            "Cannot divide nodes of different signal kinds"
        );

        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Div::ar()),
            SignalKind::Control => self.builder.add_processor(math::Div::kr()),
            _ => panic!("Cannot divide nodes of kind {:?}", self.output_kind()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Rem<Node<'a>> for Node<'a> {
    type Output = Node<'a>;

    fn rem(self, rhs: Node<'a>) -> Node<'a> {
        assert!(
            self.output_kind() == rhs.output_kind(),
            "Cannot modulo nodes of different signal kinds"
        );

        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Rem::ar()),
            SignalKind::Control => self.builder.add_processor(math::Rem::kr()),
            _ => panic!("Cannot modulo nodes of kind {:?}", self.output_kind()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Neg for Node<'a> {
    type Output = Node<'a>;

    fn neg(self) -> Node<'a> {
        let processor = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Neg::ar()),
            SignalKind::Control => self.builder.add_processor(math::Neg::kr()),
            _ => panic!("Cannot negate a node of kind {:?}", self.output_kind()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }
}

impl<'a> std::ops::Add<f64> for Node<'a> {
    type Output = Node<'a>;

    fn add(self, rhs: f64) -> Node<'a> {
        let constant = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.add_processor(math::Constant::kr(rhs.into())),
            _ => panic!(
                "Cannot add a constant to a node of kind {:?}",
                self.output_kind()
            ),
        };
        self + constant
    }
}

impl<'a> std::ops::Sub<f64> for Node<'a> {
    type Output = Node<'a>;

    fn sub(self, rhs: f64) -> Node<'a> {
        let constant = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.add_processor(math::Constant::kr(rhs.into())),
            _ => panic!(
                "Cannot subtract a constant from a node of kind {:?}",
                self.output_kind()
            ),
        };
        self - constant
    }
}

impl<'a> std::ops::Mul<f64> for Node<'a> {
    type Output = Node<'a>;

    fn mul(self, rhs: f64) -> Node<'a> {
        let constant = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.add_processor(math::Constant::kr(rhs.into())),
            _ => panic!(
                "Cannot multiply a node of kind {:?} by a constant",
                self.output_kind()
            ),
        };
        self * constant
    }
}

impl<'a> std::ops::Div<f64> for Node<'a> {
    type Output = Node<'a>;

    fn div(self, rhs: f64) -> Node<'a> {
        let constant = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.add_processor(math::Constant::kr(rhs.into())),
            _ => panic!(
                "Cannot divide a node of kind {:?} by a constant",
                self.output_kind()
            ),
        };
        self / constant
    }
}

impl<'a> std::ops::Rem<f64> for Node<'a> {
    type Output = Node<'a>;

    fn rem(self, rhs: f64) -> Node<'a> {
        let constant = match self.output_kind() {
            SignalKind::Audio => self.builder.add_processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.add_processor(math::Constant::kr(rhs.into())),
            _ => panic!(
                "Cannot modulo a node of kind {:?} by a constant",
                self.output_kind()
            ),
        };
        self % constant
    }
}

impl<'a> std::ops::AddAssign<Node<'a>> for Node<'a> {
    fn add_assign(&mut self, rhs: Node<'a>) {
        *self = *self + rhs;
    }
}

impl<'a> std::ops::SubAssign<Node<'a>> for Node<'a> {
    fn sub_assign(&mut self, rhs: Node<'a>) {
        *self = *self - rhs;
    }
}

impl<'a> std::ops::MulAssign<Node<'a>> for Node<'a> {
    fn mul_assign(&mut self, rhs: Node<'a>) {
        *self = *self * rhs;
    }
}

impl<'a> std::ops::DivAssign<Node<'a>> for Node<'a> {
    fn div_assign(&mut self, rhs: Node<'a>) {
        *self = *self / rhs;
    }
}

impl<'a> std::ops::RemAssign<Node<'a>> for Node<'a> {
    fn rem_assign(&mut self, rhs: Node<'a>) {
        *self = *self % rhs;
    }
}

impl<'a> std::ops::AddAssign<f64> for Node<'a> {
    fn add_assign(&mut self, rhs: f64) {
        *self = *self + rhs;
    }
}

impl<'a> std::ops::SubAssign<f64> for Node<'a> {
    fn sub_assign(&mut self, rhs: f64) {
        *self = *self - rhs;
    }
}

impl<'a> std::ops::MulAssign<f64> for Node<'a> {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl<'a> std::ops::DivAssign<f64> for Node<'a> {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}

impl<'a> std::ops::RemAssign<f64> for Node<'a> {
    fn rem_assign(&mut self, rhs: f64) {
        *self = *self % rhs;
    }
}
