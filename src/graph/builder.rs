use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use crate::{
    processors::*,
    sample::{SignalRate, SignalSpec},
};

use super::{
    node::{Process, Processor},
    Graph, NodeIndex,
};

#[macro_export]
macro_rules! add_node {
    ($graph:ident : $proc:expr; {$(($source: expr => $source_out:literal) @ $input_index:literal),*}) => {{
        let node = $graph.processor($proc);
        $(
            node.connect_input($input_index, $source, $source_out);
        )*
        node
    }};
}

/// A builder for constructing a [`Graph`].
#[derive(Clone)]
pub struct GraphBuilder {
    graph: Arc<Mutex<Option<Graph>>>,
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self {
            graph: Arc::new(Mutex::new(Some(Graph::new()))),
        }
    }
}

impl GraphBuilder {
    /// Creates a new [`GraphBuilder`] with the given [`Graph`] as a starting point.
    pub fn new(graph: Graph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(Some(graph))),
        }
    }

    /// Creates a new input node on the graph.
    pub fn input(&self) -> Node {
        let mut graph = self.graph.lock().unwrap();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_input();
        Node {
            builder: self,
            index,
        }
    }

    /// Creates a new output node on the graph.
    pub fn output(&self) -> Node {
        let mut graph = self.graph.lock().unwrap();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_output();
        Node {
            builder: self,
            index,
        }
    }

    /// Creates a new processor node on the graph from a [`Processor`] object.
    pub fn processor_object(&self, processor: Processor) -> Node {
        let mut graph = self.graph.lock().unwrap();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_processor_object(processor);
        Node {
            builder: self,
            index,
        }
    }

    /// Creates a new processor node on the graph from a type that implements [`Process`].
    pub fn processor(&self, processor: impl Process) -> Node {
        let mut graph = self.graph.lock().unwrap();
        let graph = graph
            .as_mut()
            .expect("GraphBuilder has already been finished");

        let index = graph.add_processor(processor);
        Node {
            builder: self,
            index,
        }
    }

    /// Creates a new audio-rate constant node on the graph.
    pub fn ar_constant(&self, value: f64) -> Node {
        let processor = math::Constant::ar(value.into());
        self.processor(processor)
    }

    /// Creates a new control-rate constant node on the graph.
    pub fn kr_constant(&self, value: f64) -> Node {
        let processor = math::Constant::kr(value.into());
        self.processor(processor)
    }

    /// Connects an output of one node to an input of another node.
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

    /// Finishes building the graph and returns the constructed [`Graph`].
    pub fn build(&self) -> Graph {
        self.graph
            .lock()
            .unwrap()
            .take()
            .expect("GraphBuilder has already been finished")
    }
}

pub trait IntoNode<'g> {
    fn into_node(self, graph: &'g GraphBuilder, rate: SignalRate) -> Node<'g>;
}

impl<'g> IntoNode<'g> for Node<'g> {
    fn into_node(self, _: &'g GraphBuilder, rate: SignalRate) -> Node<'g> {
        self.to_rate(rate)
    }
}

impl<'g> IntoNode<'g> for f64 {
    fn into_node(self, graph: &'g GraphBuilder, rate: SignalRate) -> Node<'g> {
        match rate {
            SignalRate::Audio => graph.ar_constant(self),
            SignalRate::Control => graph.kr_constant(self),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Node<'g> {
    builder: &'g GraphBuilder,
    index: NodeIndex,
}

impl<'g> Node<'g> {
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

    pub fn graph(&self) -> &'g GraphBuilder {
        self.builder
    }

    pub fn input_spec(&self) -> Vec<SignalSpec> {
        let graph = self.builder.graph.lock().unwrap();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        graph.digraph[self.index].input_spec()
    }

    pub fn output_spec(&self) -> Vec<SignalSpec> {
        let graph = self.builder.graph.lock().unwrap();
        let graph = graph
            .as_ref()
            .expect("GraphBuilder has already been finished");

        graph.digraph[self.index].output_spec()
    }

    pub fn to_rate(&self, rate: SignalRate) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot convert a node with multiple outputs to a different rate"
        );
        match (self.output_spec()[0].rate, rate) {
            (SignalRate::Audio, SignalRate::Control) => self.to_kr(),
            (SignalRate::Control, SignalRate::Audio) => self.to_ar(),
            _ => *self,
        }
    }

    /// Converts this node's single output to an audio rate signal. If the node already outputs an audio rate signal, this is a no-op.
    ///
    /// # Panics
    ///
    /// - If the node has multiple outputs
    /// - If the graph has already been built
    pub fn to_ar(&self) -> Node<'g> {
        {
            let graph = self.builder.graph.lock().unwrap();
            let graph = graph
                .as_ref()
                .expect("GraphBuilder has already been finished");

            let spec = graph.digraph[self.index].output_spec();
            if spec.len() != 1 {
                panic!("Cannot convert a node with multiple outputs to audio rate");
            };
            if spec[0].rate == SignalRate::Audio {
                return *self;
            }
        }

        let processor = self.builder.processor(io::Smooth::default());
        processor.connect_inputs([(*self, 0)]);
        processor
    }

    /// Converts this node's single output to a control rate signal. If the node already outputs a control rate signal, this is a no-op.
    ///
    /// # Panics
    ///
    /// - If the node has multiple outputs
    /// - If the graph has already been built
    pub fn to_kr(&self) -> Node<'g> {
        {
            let graph = self.builder.graph.lock().unwrap();
            let graph = graph
                .as_ref()
                .expect("GraphBuilder has already been finished");

            let spec = graph.digraph[self.index].output_spec();
            if spec.len() != 1 {
                panic!("Cannot convert a node with multiple outputs to control rate");
            };
            if spec[0].rate == SignalRate::Control {
                return *self;
            }
        }

        let processor = self.builder.processor(io::Quantize::default());
        processor.connect_inputs([(*self, 0)]);
        processor
    }

    /// Connects an input of this node to an output of another node.
    pub fn connect_input(&self, input_index: u32, source: impl IntoNode<'g>, source_output: u32) {
        let source = source.into_node(self.graph(), self.input_spec()[input_index as usize].rate);
        self.builder
            .connect(source, source_output, *self, input_index);
    }

    /// Connects multiple inputs of this node to outputs of other nodes.
    ///
    /// The connections are given by an iterator of `(source, source_output)` pairs, and connected to the inputs in the order they are given by the iterator.
    ///
    /// For example, `connect_inputs([(source1, 0), (source2, 0)])` will connect the first input of this node to the first output of `source1`, and the second input of this node to the first output of `source2`.
    ///
    /// # Panics
    ///
    /// - If any of the inputs or outputs have different signal rates
    pub fn connect_inputs<I: IntoNode<'g>>(&self, inputs: impl IntoIterator<Item = (I, u32)>) {
        for (target_input, (source, source_output)) in inputs.into_iter().enumerate() {
            let target_input_rate = self.input_spec()[target_input].rate;
            let source = source.into_node(self.graph(), target_input_rate);
            let source_output_rate = source.output_spec()[source_output as usize].rate;
            assert_eq!(
                target_input_rate, source_output_rate,
                "Cannot connect nodes with different signal rates"
            );

            self.builder
                .connect(source, source_output, *self, target_input as u32);
        }
    }

    pub fn sin(&self) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take sin of a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Sin::ar()),
            SignalRate::Control => self.builder.processor(math::Sin::kr()),
        };
        processor.connect_inputs([(*self, 0)]);
        processor
    }

    pub fn cos(&self) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take cos of a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Cos::ar()),
            SignalRate::Control => self.builder.processor(math::Cos::kr()),
        };
        processor.connect_inputs([(*self, 0)]);
        processor
    }

    pub fn abs(&self) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take abs of a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Abs::ar()),
            SignalRate::Control => self.builder.processor(math::Abs::kr()),
        };
        processor.connect_inputs([(*self, 0)]);
        processor
    }

    pub fn sqrt(&self) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take sqrt of a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Sqrt::ar()),
            SignalRate::Control => self.builder.processor(math::Sqrt::kr()),
        };
        processor.connect_inputs([(*self, 0)]);
        processor
    }

    pub fn exp(&self) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take exp of a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Exp::ar()),
            SignalRate::Control => self.builder.processor(math::Exp::kr()),
        };
        processor.connect_inputs([(*self, 0)]);
        processor
    }

    pub fn ln(&self) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot take ln of a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Ln::ar()),
            SignalRate::Control => self.builder.processor(math::Ln::kr()),
        };
        processor.connect_inputs([(*self, 0)]);
        processor
    }

    pub fn clip(&self, min: impl IntoNode<'g>, max: impl IntoNode<'g>) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot clip a node with multiple outputs"
        );
        let min = min.into_node(self.graph(), self.output_spec()[0].rate);
        let max = max.into_node(self.graph(), self.output_spec()[0].rate);
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

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Clip::ar()),
            SignalRate::Control => self.builder.processor(math::Clip::kr()),
        };
        processor.connect_inputs([(*self, 0), (min, 0), (max, 0)]);
        processor
    }

    pub fn if_else(&self, if_true: impl IntoNode<'g>, if_false: impl IntoNode<'g>) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot use if_else with a node with multiple outputs"
        );
        let if_true = if_true.into_node(self.graph(), self.output_spec()[0].rate);
        let if_false = if_false.into_node(self.graph(), self.output_spec()[0].rate);
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

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(control::IfElse::ar()),
            SignalRate::Control => self.builder.processor(control::IfElse::kr()),
        };
        processor.connect_inputs([(*self, 0), (if_true, 0), (if_false, 0)]);
        processor
    }

    pub fn debug_print(&self) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot debug_print a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(io::DebugPrint::ar()),
            SignalRate::Control => self.builder.processor(io::DebugPrint::kr()),
        };
        processor.connect_inputs([(*self, 0)]);
        processor
    }

    pub fn gt(&self, rhs: impl IntoNode<'g>) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot compare a node with multiple outputs"
        );

        let rhs = rhs.into_node(self.graph(), self.output_spec()[0].rate);
        assert_eq!(
            rhs.num_outputs(),
            1,
            "Cannot compare a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Gt::ar()),
            SignalRate::Control => self.builder.processor(math::Gt::kr()),
        };
        processor.connect_inputs([(*self, 0), (rhs, 0)]);
        processor
    }

    pub fn lt(&self, rhs: impl IntoNode<'g>) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot compare a node with multiple outputs"
        );

        let rhs = rhs.into_node(self.graph(), self.output_spec()[0].rate);
        assert_eq!(
            rhs.num_outputs(),
            1,
            "Cannot compare a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Lt::ar()),
            SignalRate::Control => self.builder.processor(math::Lt::kr()),
        };
        processor.connect_inputs([(*self, 0), (rhs, 0)]);
        processor
    }

    pub fn eq(&self, rhs: impl IntoNode<'g>) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot compare a node with multiple outputs"
        );

        let rhs = rhs.into_node(self.graph(), self.output_spec()[0].rate);
        assert_eq!(
            rhs.num_outputs(),
            1,
            "Cannot compare a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Eq::ar()),
            SignalRate::Control => self.builder.processor(math::Eq::kr()),
        };
        processor.connect_inputs([(*self, 0), (rhs, 0)]);
        processor
    }

    pub fn pow(&self, rhs: impl IntoNode<'g>) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot powf a node with multiple outputs"
        );

        let rhs = rhs.into_node(self.graph(), self.output_spec()[0].rate);
        assert_eq!(
            rhs.num_outputs(),
            1,
            "Cannot powf a node with multiple outputs"
        );

        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Pow::ar()),
            SignalRate::Control => self.builder.processor(math::Pow::kr()),
        };
        processor.connect_inputs([(*self, 0), (rhs, 0)]);
        processor
    }
}

impl Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.index)
    }
}

macro_rules! node_ops_binary {
    ($($op:ident $func:ident)*) => {
        $(
        impl<'g> std::ops::$op<Node<'g>> for Node<'g> {
            type Output = Node<'g>;

            fn $func(self, rhs: Node<'g>) -> Node<'g> {
                assert_eq!(
                    self.num_outputs(),
                    1,
                    concat!("Cannot ", stringify!($op), " a node with multiple outputs")
                );
                assert_eq!(
                    rhs.num_outputs(),
                    1,
                    concat!("Cannot ", stringify!($op), " a node with multiple outputs")
                );

                let processor = match self.output_spec()[0].rate {
                    SignalRate::Audio => self.builder.processor(math::$op::ar()),
                    SignalRate::Control => self.builder.processor(math::$op::kr()),
                };

                processor.connect_inputs([(self, 0), (rhs, 0)]);
                processor
            }
        }

        impl<'g> std::ops::$op<f64> for Node<'g> {
            type Output = Node<'g>;

            fn $func(self, rhs: f64) -> Node<'g> {
                assert_eq!(
                    self.num_outputs(),
                    1,
                    concat!("Cannot ", stringify!($op), " a node with multiple outputs")
                );

                let constant = match self.output_spec()[0].rate {
                    SignalRate::Audio => self.builder.processor(math::Constant::ar(rhs.into())),
                    SignalRate::Control => self.builder.processor(math::Constant::kr(rhs.into())),
                };

                self.$func(constant)
            }
        }
        )*
    };
}

node_ops_binary! {
    Add add
    Sub sub
    Mul mul
    Div div
    Rem rem
}

use std::ops::{Add, Div, Mul, Rem, Sub};

macro_rules! node_ops_assign {
    ($($op:ident $assign_func:ident $func:ident)*) => {
        $(
        impl<'g> std::ops::$op<Node<'g>> for Node<'g> {
            fn $assign_func(&mut self, rhs: Node<'g>) {
                *self = (*self).$func(rhs);
            }
        }

        impl<'g> std::ops::$op<f64> for Node<'g> {
            fn $assign_func(&mut self, rhs: f64) {
                *self = (*self).$func(rhs);
            }
        }
        )*
    };
}

node_ops_assign! {
    AddAssign add_assign add
    SubAssign sub_assign sub
    MulAssign mul_assign mul
    DivAssign div_assign div
    RemAssign rem_assign rem
}

impl<'g> std::ops::Neg for Node<'g> {
    type Output = Node<'g>;

    fn neg(self) -> Node<'g> {
        assert_eq!(
            self.num_outputs(),
            1,
            "Cannot negate a node with multiple outputs"
        );
        let processor = match self.output_spec()[0].rate {
            SignalRate::Audio => self.builder.processor(math::Neg::ar()),
            SignalRate::Control => self.builder.processor(math::Neg::kr()),
        };
        processor.connect_inputs([(self, 0)]);
        processor
    }
}
