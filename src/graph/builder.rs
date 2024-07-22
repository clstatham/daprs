use std::{cell::RefCell, fmt::Debug};

use crate::{builtin::*, sample::SignalKind};

use super::{node::Process, Graph, NodeIndex};

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

    pub fn ar_constant(&self, value: f64) -> Node<'a> {
        let processor = math::Constant::ar(value.into());
        self.add_processor(processor)
    }

    pub fn kr_constant(&self, value: f64) -> Node<'a> {
        let processor = math::Constant::kr(value.into());
        self.add_processor(processor)
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

    pub fn input_kinds(self) -> Vec<SignalKind> {
        let graph = self.builder.graph.borrow();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        graph.digraph[self.index].input_kinds()
    }

    pub fn output_kinds(self) -> Vec<SignalKind> {
        let graph = self.builder.graph.borrow();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        graph.digraph[self.index].output_kinds()
    }

    /// Converts this node's single output to an audio rate signal. If the node already outputs an audio rate signal, this is a no-op.
    ///
    /// # Panics
    ///
    /// - If the node has multiple outputs
    /// - If the graph has already been built
    pub fn to_ar(self) -> Node<'a> {
        {
            let graph = self.builder.graph.borrow();
            let graph = graph
                .as_ref()
                .expect("GraphBuilder has already been finished");

            let kinds = graph.digraph[self.index].output_kinds();
            if kinds.len() != 1 {
                panic!("Cannot convert a node with multiple outputs to audio rate");
            };
            if kinds[0] == SignalKind::Audio {
                return self;
            }
        }

        let processor = self.builder.add_processor(io::Smooth::default());
        processor.connect_inputs([(self, 0)]);
        processor
    }

    /// Converts this node's single output to a control rate signal. If the node already outputs a control rate signal, this is a no-op.
    ///
    /// # Panics
    ///
    /// - If the node has multiple outputs
    /// - If the graph has already been built
    pub fn to_kr(self) -> Node<'a> {
        {
            let graph = self.builder.graph.borrow();
            let graph = graph
                .as_ref()
                .expect("GraphBuilder has already been finished");

            let kinds = graph.digraph[self.index].output_kinds();
            if kinds.len() != 1 {
                panic!("Cannot convert a node with multiple outputs to control rate");
            };
            if kinds[0] == SignalKind::Control {
                return self;
            }
        }

        let processor = self.builder.add_processor(io::Quantize::default());
        processor.connect_inputs([(self, 0)]);
        processor
    }

    /// Connects an input of this node to an output of another node.
    pub fn connect_input(self, input_index: u32, source: Node<'a>, source_output: u32) {
        self.builder
            .connect(source, source_output, self, input_index);
    }

    /// Connects multiple inputs of this node to outputs of other nodes.
    ///
    /// The connections are given by an iterator of `(source, source_output)` pairs, and connected to the inputs in the order they are given by the iterator.
    ///
    /// For example, `connect_inputs([(source1, 0), (source2, 0)])` will connect the first input of this node to the first output of `source1`, and the second input of this node to the first output of `source2`.
    ///
    /// # Panics
    ///
    /// - If any of the inputs or outputs have different signal kinds
    pub fn connect_inputs(self, inputs: impl IntoIterator<Item = (Node<'a>, u32)>) {
        for (target_input, (source, source_output)) in inputs.into_iter().enumerate() {
            let target_input_kind = self.input_kinds()[target_input];
            let source_output_kind = source.output_kinds()[source_output as usize];
            assert_eq!(
                target_input_kind, source_output_kind,
                "Cannot connect nodes with different signal kinds"
            );

            self.builder
                .connect(source, source_output, self, target_input as u32);
        }
    }

    pub fn sin(self) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take sin of a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Sin::ar()),
            SignalKind::Control => self.builder.add_processor(math::Sin::kr()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn cos(self) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take cos of a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Cos::ar()),
            SignalKind::Control => self.builder.add_processor(math::Cos::kr()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn sqrt(self) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take sqrt of a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Sqrt::ar()),
            SignalKind::Control => self.builder.add_processor(math::Sqrt::kr()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn exp(self) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take exp of a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Exp::ar()),
            SignalKind::Control => self.builder.add_processor(math::Exp::kr()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn ln(self) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take ln of a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Ln::ar()),
            SignalKind::Control => self.builder.add_processor(math::Ln::kr()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }

    pub fn gt(self, other: Node<'a>) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot compare a node with multiple outputs"
        );
        assert_eq!(
            other.num_outputs(),
            1,
            "Cannot compare a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Gt::ar()),
            SignalKind::Control => self.builder.add_processor(math::Gt::kr()),
        };
        processor.connect_inputs([(self, 0), (other, 0)]);
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
            self.output_kinds() == rhs.output_kinds(),
            "Cannot add nodes of different signal kinds"
        );
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot add a node with multiple outputs"
        );
        assert_eq!(
            rhs.num_outputs(),
            1,
            "Cannot add a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Add::ar()),
            SignalKind::Control => self.builder.add_processor(math::Add::kr()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Sub<Node<'a>> for Node<'a> {
    type Output = Node<'a>;

    fn sub(self, rhs: Node<'a>) -> Node<'a> {
        assert!(
            self.output_kinds() == rhs.output_kinds(),
            "Cannot subtract nodes of different signal kinds"
        );
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot subtract a node with multiple outputs"
        );
        assert_eq!(
            rhs.num_outputs(),
            1,
            "Cannot subtract a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Sub::ar()),
            SignalKind::Control => self.builder.add_processor(math::Sub::kr()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Mul<Node<'a>> for Node<'a> {
    type Output = Node<'a>;

    fn mul(self, rhs: Node<'a>) -> Node<'a> {
        assert!(
            self.output_kinds() == rhs.output_kinds(),
            "Cannot multiply nodes of different signal kinds"
        );
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot multiply a node with multiple outputs"
        );
        assert_eq!(
            rhs.num_outputs(),
            1,
            "Cannot multiply a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Mul::ar()),
            SignalKind::Control => self.builder.add_processor(math::Mul::kr()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Div<Node<'a>> for Node<'a> {
    type Output = Node<'a>;

    fn div(self, rhs: Node<'a>) -> Node<'a> {
        assert!(
            self.output_kinds() == rhs.output_kinds(),
            "Cannot divide nodes of different signal kinds"
        );
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot divide a node with multiple outputs"
        );
        assert_eq!(
            rhs.num_outputs(),
            1,
            "Cannot divide a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Div::ar()),
            SignalKind::Control => self.builder.add_processor(math::Div::kr()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Rem<Node<'a>> for Node<'a> {
    type Output = Node<'a>;

    fn rem(self, rhs: Node<'a>) -> Node<'a> {
        assert!(
            self.output_kinds() == rhs.output_kinds(),
            "Cannot modulo nodes of different signal kinds"
        );
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot modulo a node with multiple outputs"
        );
        assert_eq!(
            rhs.num_outputs(),
            1,
            "Cannot modulo a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Rem::ar()),
            SignalKind::Control => self.builder.add_processor(math::Rem::kr()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl<'a> std::ops::Neg for Node<'a> {
    type Output = Node<'a>;

    fn neg(self) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot negate a node with multiple outputs"
        );
        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Neg::ar()),
            SignalKind::Control => self.builder.add_processor(math::Neg::kr()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }
}

impl<'a> std::ops::Add<f64> for Node<'a> {
    type Output = Node<'a>;

    fn add(self, rhs: f64) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot add a constant to a node with multiple outputs"
        );
        let constant = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.add_processor(math::Constant::kr(rhs.into())),
        };
        self + constant
    }
}

impl<'a> std::ops::Sub<f64> for Node<'a> {
    type Output = Node<'a>;

    fn sub(self, rhs: f64) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot subtract a constant from a node with multiple outputs"
        );
        let constant = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.add_processor(math::Constant::kr(rhs.into())),
        };
        self - constant
    }
}

impl<'a> std::ops::Mul<f64> for Node<'a> {
    type Output = Node<'a>;

    fn mul(self, rhs: f64) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot multiply a node with multiple outputs by a constant"
        );
        let constant = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.add_processor(math::Constant::kr(rhs.into())),
        };
        self * constant
    }
}

impl<'a> std::ops::Div<f64> for Node<'a> {
    type Output = Node<'a>;

    fn div(self, rhs: f64) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot divide a node with multiple outputs by a constant"
        );
        let constant = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.add_processor(math::Constant::kr(rhs.into())),
        };
        self / constant
    }
}

impl<'a> std::ops::Rem<f64> for Node<'a> {
    type Output = Node<'a>;

    fn rem(self, rhs: f64) -> Node<'a> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot modulo a node with multiple outputs by a constant"
        );
        let constant = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.add_processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.add_processor(math::Constant::kr(rhs.into())),
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
