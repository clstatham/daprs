use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use crate::{processors::*, sample::SignalKind};

use super::{node::Process, Graph, NodeIndex};

#[derive(Clone)]
pub struct GraphBuilder {
    graph: Arc<Mutex<Option<Graph>>>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(Mutex::new(Some(Graph::new()))),
        }
    }

    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(Some(graph))),
        }
    }

    pub fn input(&self) -> Node {
        let mut graph = self.graph.lock().unwrap();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_input();
        Node {
            builder: self.clone(),
            index,
        }
    }

    pub fn output(&self) -> Node {
        let mut graph = self.graph.lock().unwrap();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_output();
        Node {
            builder: self.clone(),
            index,
        }
    }

    pub fn processor(&self, processor: impl Process) -> Node {
        let mut graph = self.graph.lock().unwrap();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_processor(processor);
        Node {
            builder: self.clone(),
            index,
        }
    }

    pub fn ar_constant(&self, value: f64) -> Node {
        let processor = math::Constant::ar(value.into());
        self.processor(processor)
    }

    pub fn kr_constant(&self, value: f64) -> Node {
        let processor = math::Constant::kr(value.into());
        self.processor(processor)
    }

    pub fn connect(&self, source: Node, source_output: u32, target: Node, target_input: u32) {
        assert!(
            Arc::ptr_eq(&source.builder.graph, &self.graph),
            "Cannot connect nodes from different graphs"
        );
        assert!(
            Arc::ptr_eq(&target.builder.graph, &self.graph),
            "Cannot connect nodes from different graphs"
        );

        let mut graph = self.graph.lock().unwrap();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        graph.connect(source.index, source_output, target.index, target_input);
    }

    pub fn build(&self) -> Graph {
        self.graph
            .lock()
            .unwrap()
            .take()
            .expect("GraphBuilder has already been finished")
    }
}

#[derive(Clone)]
pub struct Node {
    builder: GraphBuilder,
    index: NodeIndex,
}

impl Node {
    pub fn num_inputs(&self) -> usize {
        let graph = self.builder.graph.lock().unwrap();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        let node = &graph.digraph[self.index];
        node.inputs().len()
    }

    pub fn num_outputs(&self) -> usize {
        let graph = self.builder.graph.lock().unwrap();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        let node = &graph.digraph[self.index];
        node.outputs().len()
    }

    pub fn build(&self) -> Graph {
        self.builder.build()
    }

    pub fn graph(&self) -> GraphBuilder {
        self.builder.clone()
    }

    pub fn input_kinds(&self) -> Vec<SignalKind> {
        let graph = self.builder.graph.lock().unwrap();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        graph.digraph[self.index].input_kinds()
    }

    pub fn output_kinds(&self) -> Vec<SignalKind> {
        let graph = self.builder.graph.lock().unwrap();
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
    pub fn to_ar(&self) -> Node {
        {
            let graph = self.builder.graph.lock().unwrap();
            let graph = graph
                .as_ref()
                .expect("GraphBuilder has already been finished");

            let kinds = graph.digraph[self.index].output_kinds();
            if kinds.len() != 1 {
                panic!("Cannot convert a node with multiple outputs to audio rate");
            };
            if kinds[0] == SignalKind::Audio {
                return self.clone();
            }
        }

        let processor = self.builder.processor(io::Smooth::default());
        processor.connect_inputs([(self.clone(), 0)]);
        processor
    }

    /// Converts this node's single output to a control rate signal. If the node already outputs a control rate signal, this is a no-op.
    ///
    /// # Panics
    ///
    /// - If the node has multiple outputs
    /// - If the graph has already been built
    pub fn to_kr(&self) -> Node {
        {
            let graph = self.builder.graph.lock().unwrap();
            let graph = graph
                .as_ref()
                .expect("GraphBuilder has already been finished");

            let kinds = graph.digraph[self.index].output_kinds();
            if kinds.len() != 1 {
                panic!("Cannot convert a node with multiple outputs to control rate");
            };
            if kinds[0] == SignalKind::Control {
                return self.clone();
            }
        }

        let processor = self.builder.processor(io::Quantize::default());
        processor.connect_inputs([(self.clone(), 0)]);
        processor
    }

    /// Connects an input of this node to an output of another node.
    pub fn connect_input(&self, input_index: u32, source: Node, source_output: u32) {
        self.builder
            .connect(source, source_output, self.clone(), input_index);
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
    pub fn connect_inputs(&self, inputs: impl IntoIterator<Item = (Node, u32)>) {
        for (target_input, (source, source_output)) in inputs.into_iter().enumerate() {
            let target_input_kind = self.input_kinds()[target_input];
            let source_output_kind = source.output_kinds()[source_output as usize];
            assert_eq!(
                target_input_kind, source_output_kind,
                "Cannot connect nodes with different signal kinds"
            );

            self.builder
                .connect(source, source_output, self.clone(), target_input as u32);
        }
    }

    pub fn sin(&self) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take sin of a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Sin::ar()),
            SignalKind::Control => self.builder.processor(math::Sin::kr()),
        };
        processor.connect_inputs([(self.clone(), 0)]);
        processor
    }

    pub fn cos(&self) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take cos of a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Cos::ar()),
            SignalKind::Control => self.builder.processor(math::Cos::kr()),
        };
        processor.connect_inputs([(self.clone(), 0)]);
        processor
    }

    pub fn sqrt(&self) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take sqrt of a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Sqrt::ar()),
            SignalKind::Control => self.builder.processor(math::Sqrt::kr()),
        };
        processor.connect_inputs([(self.clone(), 0)]);
        processor
    }

    pub fn exp(&self) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take exp of a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Exp::ar()),
            SignalKind::Control => self.builder.processor(math::Exp::kr()),
        };
        processor.connect_inputs([(self.clone(), 0)]);
        processor
    }

    pub fn ln(&self) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take ln of a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Ln::ar()),
            SignalKind::Control => self.builder.processor(math::Ln::kr()),
        };
        processor.connect_inputs([(self.clone(), 0)]);
        processor
    }

    pub fn gt(&self, other: Node) -> Node {
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
            SignalKind::Audio => self.builder.processor(math::Gt::ar()),
            SignalKind::Control => self.builder.processor(math::Gt::kr()),
        };
        processor.connect_inputs([(self.clone(), 0), (other, 0)]);
        processor
    }

    pub fn lt(&self, other: Node) -> Node {
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
            SignalKind::Audio => self.builder.processor(math::Lt::ar()),
            SignalKind::Control => self.builder.processor(math::Lt::kr()),
        };
        processor.connect_inputs([(self.clone(), 0), (other, 0)]);
        processor
    }

    pub fn eq(&self, other: Node) -> Node {
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
            SignalKind::Audio => self.builder.processor(math::Eq::ar()),
            SignalKind::Control => self.builder.processor(math::Eq::kr()),
        };
        processor.connect_inputs([(self.clone(), 0), (other.clone(), 0)]);
        processor
    }

    pub fn clip(&self, min: Node, max: Node) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot clip a node with multiple outputs"
        );
        assert_eq!(
            min.num_outputs(),
            1,
            "Cannot clip a node with multiple outputs"
        );
        assert_eq!(
            max.num_outputs(),
            1,
            "Cannot clip a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Clip::ar()),
            SignalKind::Control => self.builder.processor(math::Clip::kr()),
        };
        processor.connect_inputs([(self.clone(), 0), (min, 0), (max, 0)]);
        processor
    }

    pub fn if_else(&self, if_true: Node, if_false: Node) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot use if_else with a node with multiple outputs"
        );
        assert_eq!(
            if_true.num_outputs(),
            1,
            "Cannot use if_else with a node with multiple outputs"
        );
        assert_eq!(
            if_false.num_outputs(),
            1,
            "Cannot use if_else with a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(control::IfElse::ar()),
            SignalKind::Control => self.builder.processor(control::IfElse::kr()),
        };
        processor.connect_inputs([(self.clone(), 0), (if_true, 0), (if_false, 0)]);
        processor
    }

    pub fn debug_print(&self) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot debug_print a node with multiple outputs"
        );

        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(io::DebugPrint::ar()),
            SignalKind::Control => self.builder.processor(io::DebugPrint::kr()),
        };
        processor.connect_inputs([(self.clone(), 0)]);
        processor
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.index)
    }
}

impl std::ops::Add<Node> for Node {
    type Output = Node;

    fn add(self, rhs: Node) -> Node {
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
            SignalKind::Audio => self.builder.processor(math::Add::ar()),
            SignalKind::Control => self.builder.processor(math::Add::kr()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl std::ops::Sub<Node> for Node {
    type Output = Node;

    fn sub(self, rhs: Node) -> Node {
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
            SignalKind::Audio => self.builder.processor(math::Sub::ar()),
            SignalKind::Control => self.builder.processor(math::Sub::kr()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl std::ops::Mul<Node> for Node {
    type Output = Node;

    fn mul(self, rhs: Node) -> Node {
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
            SignalKind::Audio => self.builder.processor(math::Mul::ar()),
            SignalKind::Control => self.builder.processor(math::Mul::kr()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl std::ops::Div<Node> for Node {
    type Output = Node;

    fn div(self, rhs: Node) -> Node {
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
            SignalKind::Audio => self.builder.processor(math::Div::ar()),
            SignalKind::Control => self.builder.processor(math::Div::kr()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl std::ops::Rem<Node> for Node {
    type Output = Node;

    fn rem(self, rhs: Node) -> Node {
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
            SignalKind::Audio => self.builder.processor(math::Rem::ar()),
            SignalKind::Control => self.builder.processor(math::Rem::kr()),
        };

        processor.connect_inputs([(self, 0), (rhs, 0)]);
        processor
    }
}

impl std::ops::Neg for Node {
    type Output = Node;

    fn neg(self) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot negate a node with multiple outputs"
        );
        let processor = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Neg::ar()),
            SignalKind::Control => self.builder.processor(math::Neg::kr()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }
}

impl std::ops::Add<f64> for Node {
    type Output = Node;

    fn add(self, rhs: f64) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot add a constant to a node with multiple outputs"
        );
        let constant = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.processor(math::Constant::kr(rhs.into())),
        };
        self + constant
    }
}

impl std::ops::Sub<f64> for Node {
    type Output = Node;

    fn sub(self, rhs: f64) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot subtract a constant from a node with multiple outputs"
        );
        let constant = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.processor(math::Constant::kr(rhs.into())),
        };
        self - constant
    }
}

impl std::ops::Mul<f64> for Node {
    type Output = Node;

    fn mul(self, rhs: f64) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot multiply a node with multiple outputs by a constant"
        );
        let constant = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.processor(math::Constant::kr(rhs.into())),
        };
        self * constant
    }
}

impl std::ops::Div<f64> for Node {
    type Output = Node;

    fn div(self, rhs: f64) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot divide a node with multiple outputs by a constant"
        );
        let constant = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.processor(math::Constant::kr(rhs.into())),
        };
        self / constant
    }
}

impl std::ops::Rem<f64> for Node {
    type Output = Node;

    fn rem(self, rhs: f64) -> Node {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot modulo a node with multiple outputs by a constant"
        );
        let constant = match self.output_kinds()[0] {
            SignalKind::Audio => self.builder.processor(math::Constant::ar(rhs.into())),
            SignalKind::Control => self.builder.processor(math::Constant::kr(rhs.into())),
        };
        self % constant
    }
}

impl std::ops::AddAssign<Node> for Node {
    fn add_assign(&mut self, rhs: Node) {
        *self = self.clone() + rhs;
    }
}

impl std::ops::SubAssign<Node> for Node {
    fn sub_assign(&mut self, rhs: Node) {
        *self = self.clone() - rhs;
    }
}

impl std::ops::MulAssign<Node> for Node {
    fn mul_assign(&mut self, rhs: Node) {
        *self = self.clone() * rhs;
    }
}

impl std::ops::DivAssign<Node> for Node {
    fn div_assign(&mut self, rhs: Node) {
        *self = self.clone() / rhs;
    }
}

impl std::ops::RemAssign<Node> for Node {
    fn rem_assign(&mut self, rhs: Node) {
        *self = self.clone() % rhs;
    }
}

impl std::ops::AddAssign<f64> for Node {
    fn add_assign(&mut self, rhs: f64) {
        *self = self.clone() + rhs;
    }
}

impl std::ops::SubAssign<f64> for Node {
    fn sub_assign(&mut self, rhs: f64) {
        *self = self.clone() - rhs;
    }
}

impl std::ops::MulAssign<f64> for Node {
    fn mul_assign(&mut self, rhs: f64) {
        *self = self.clone() * rhs;
    }
}

impl std::ops::DivAssign<f64> for Node {
    fn div_assign(&mut self, rhs: f64) {
        *self = self.clone() / rhs;
    }
}

impl std::ops::RemAssign<f64> for Node {
    fn rem_assign(&mut self, rhs: f64) {
        *self = self.clone() % rhs;
    }
}
