//! Contains the [`Node`] type and related types and traits.

use petgraph::prelude::*;

use crate::{prelude::*, signal::SignalKind};

use super::graph_builder::GraphBuilder;

/// A node in a [`GraphBuilder`].
#[derive(Clone)]
pub struct Node {
    pub(crate) graph: GraphBuilder,
    pub(crate) node_id: NodeIndex,
}

impl Node {
    #[inline]
    pub(crate) fn id(&self) -> NodeIndex {
        self.node_id
    }

    /// Returns the graph that the node belongs to.
    #[inline]
    pub fn graph(&self) -> &GraphBuilder {
        &self.graph
    }

    /// Asserts that the node has a single output.
    #[inline]
    #[track_caller]
    pub fn assert_single_output(&self) {
        assert_eq!(self.num_outputs(), 1, "expected single output");
    }

    /// Returns the number of inputs of the node.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].input_spec().len())
    }

    /// Returns the number of outputs of the node.
    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].output_spec().len())
    }

    /// Returns the input of the node at the given index.
    #[inline]
    pub fn input(&self, index: impl IntoInputIdx) -> Input {
        Input {
            node: self.clone(),
            input_index: index.into_input_idx(self),
        }
    }

    /// Returns the output of the node at the given index.
    #[inline]
    pub fn output(&self, index: impl IntoOutputIdx) -> Output {
        Output {
            node: self.clone(),
            output_index: index.into_output_idx(self),
        }
    }

    /// Returns the signal type of the input at the given index.
    #[inline]
    pub fn input_kind(&self, index: impl IntoInputIdx) -> SignalKind {
        let index = index.into_input_idx(self);
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].input_spec()[index as usize].kind())
    }

    /// Returns the signal type of the output at the given index.
    #[inline]
    pub fn output_kind(&self, index: impl IntoOutputIdx) -> SignalKind {
        let index = index.into_output_idx(self);
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].output_spec()[index as usize].kind())
    }

    /// Connects the given input of this node to the given output of another node.
    #[inline]
    #[track_caller]
    pub fn connect_input(
        &self,
        source: impl IntoNode,
        source_output: impl IntoOutputIdx,
        target_input: impl IntoInputIdx,
    ) {
        let output = source.into_node(&self.graph);
        let source_output = source_output.into_output_idx(&output);
        let target_input = target_input.into_input_idx(self);
        self.graph
            .connect(output.id(), source_output, self.id(), target_input);
    }

    /// Connects the given output of this node to the given input of another node.
    #[inline]
    #[track_caller]
    pub fn connect_output(
        &self,
        output: impl IntoOutputIdx,
        target: impl IntoNode,
        target_input: impl IntoInputIdx,
    ) {
        let target = target.into_node(&self.graph);
        let output_index = output.into_output_idx(self);
        let target_input = target_input.into_input_idx(&target);
        self.graph
            .connect(self.id(), output_index, target.id(), target_input);
    }

    /// Converts the output message to a signal.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    #[track_caller]
    pub fn to_audio(&self) -> Node {
        self.assert_single_output();
        if self.output_kind(0) == SignalKind::Sample {
            return self.clone();
        }
        let proc = self.graph.add(MessageToAudio);
        proc.connect_input(self, 0, 0);
        proc
    }

    /// Converts the output signal to a message.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    #[track_caller]
    pub fn to_message(&self) -> Node {
        self.assert_single_output();
        if self.output_kind(0) == SignalKind::Message {
            return self.clone();
        }
        let proc = self.graph.add(AudioToMessage);
        proc.connect_input(self, 0, 0);
        proc
    }

    /// Smooths the output signal.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    #[track_caller]
    pub fn smooth(&self) -> Node {
        self.assert_single_output();
        let proc = self.graph.add(Smooth::default());
        proc.input("factor").set(0.1);
        proc.input(0).connect(&self.output(0));
        proc
    }

    /// Converts the output signal from a MIDI note to a frequency in Hz.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    #[track_caller]
    pub fn midi2freq(&self) -> Node {
        self.assert_single_output();
        let proc = self.graph.add(MidiToFreq);
        proc.input(0).connect(&self.output(0));
        proc
    }

    /// Converts the output signal from a frequency in Hz to a MIDI note.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    #[track_caller]
    pub fn freq2midi(&self) -> Node {
        self.assert_single_output();
        let proc = self.graph.add(FreqToMidi);
        proc.input(0).connect(&self.output(0));
        proc
    }

    /// Creates a new, single-output node that holds and continuously outputs the last value of this node.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    #[track_caller]
    pub fn make_register(&self) -> Node {
        self.assert_single_output();
        let node = self.graph.add(Register::default());
        node.input(0).connect(&self.output(0));
        node
    }
}

/// An input of a node in the graph.
#[derive(Clone)]
pub struct Input {
    pub(crate) node: Node,
    pub(crate) input_index: u32,
}

impl Input {
    /// Sets the value of the input.
    #[inline]
    pub fn set(&self, value: impl IntoNode) {
        let value = value.into_node(self.node.graph());
        value.assert_single_output();
        match self.kind() {
            SignalKind::Message => self
                .node
                .connect_input(value.to_message(), 0, self.input_index),
            SignalKind::Sample => self
                .node
                .connect_input(value.to_audio(), 0, self.input_index),
        }
    }

    /// Returns the node that the input belongs to.
    #[inline]
    pub fn node(&self) -> Node {
        self.node.clone()
    }

    /// Returns the signal type of the input.
    #[inline]
    pub fn kind(&self) -> SignalKind {
        self.node.input_kind(self.input_index)
    }

    /// Connects the input to the given output.
    #[inline]
    #[track_caller]
    pub fn connect(&self, output: &Output) {
        let output = output.to_kind(self.kind());
        self.node
            .connect_input(&output.node, output.output_index, self.input_index);
    }

    /// Creates a parameter for the input.
    #[inline]
    pub fn param<T: Into<Message>>(
        &self,
        name: impl Into<String>,
        initial_value: Option<T>,
    ) -> Param {
        let name = name.into();
        let param = Param::new(&name, initial_value);
        let proc = self.node.graph().add_param(param.clone());
        match self.kind() {
            SignalKind::Message => proc.output(0).connect(self),
            SignalKind::Sample => proc.to_audio().output(0).connect(self),
        }
        param
    }
}

/// An output of a node in the graph.
#[derive(Clone)]
pub struct Output {
    pub(crate) node: Node,
    pub(crate) output_index: u32,
}

impl Output {
    /// Connects the output to the given input.
    #[inline]
    #[track_caller]
    pub fn connect(&self, input: &Input) {
        let this = self.to_kind(input.kind());
        this.node
            .connect_output(self.output_index, &input.node, input.input_index);
    }

    /// Returns the node that the output belongs to.
    #[inline]
    pub fn node(&self) -> Node {
        self.node.clone()
    }

    /// Returns the signal type of the output.
    #[inline]
    pub fn kind(&self) -> SignalKind {
        self.node.output_kind(self.output_index)
    }

    /// Converts the output to a sample signal.
    #[inline]
    pub fn to_audio(&self) -> Output {
        if self.kind() == SignalKind::Sample {
            return self.clone();
        }
        let proc = self.node.graph().add(MessageToAudio);
        proc.input(0).connect(self);
        proc.output(0)
    }

    /// Converts the output to a message signal.
    #[inline]
    pub fn to_message(&self) -> Output {
        if self.kind() == SignalKind::Message {
            return self.clone();
        }
        let proc = self.node.graph().add(AudioToMessage);
        proc.input(0).connect(self);
        proc.output(0)
    }

    /// Converts the output to the given signal type.
    #[inline]
    pub fn to_kind(&self, kind: SignalKind) -> Output {
        match kind {
            SignalKind::Message => self.to_message(),
            SignalKind::Sample => self.to_audio(),
        }
    }

    /// Creates a new, single-output node that passes the value of this output through.
    #[inline]
    pub fn make_node(&self) -> Node {
        let node = self.node.graph().add(Passthrough);
        node.input(0).connect(self);
        node
    }

    /// Creates a new, single-output node that holds and continuously outputs the last value of this output.
    #[inline]
    pub fn make_register(&self) -> Node {
        let node = self.node.graph().add(Register::default());
        node.input(0).connect(self);
        node
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for crate::graph::NodeIndex {}
    impl Sealed for super::Node {}
    impl Sealed for &super::Node {}
    impl Sealed for super::Message {}
    impl Sealed for super::Param {}
    impl Sealed for crate::signal::Sample {}
    impl Sealed for i64 {}
    impl Sealed for u32 {}
    impl Sealed for &str {}
}

/// Trait for converting a value into a node.
pub trait IntoNode: sealed::Sealed {
    /// Converts the value into a node.
    fn into_node(self, graph: &GraphBuilder) -> Node;
}

impl IntoNode for Node {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        Node {
            graph: graph.clone(),
            node_id: self.node_id,
        }
    }
}

impl IntoNode for &Node {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        Node {
            graph: graph.clone(),
            node_id: self.node_id,
        }
    }
}

impl IntoNode for Param {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.add(self)
    }
}

impl IntoNode for NodeIndex {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        Node {
            graph: graph.clone(),
            node_id: self,
        }
    }
}

impl IntoNode for Sample {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(self)
    }
}

impl IntoNode for Message {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant_message(self)
    }
}

/// Trait for converting a value into an input index for a node.
pub trait IntoOutputIdx: sealed::Sealed {
    /// Converts the value into an input index for the given node.
    fn into_output_idx(self, node: &Node) -> u32;
}

/// Trait for converting a value into an output index for a node.
pub trait IntoInputIdx: sealed::Sealed {
    /// Converts the value into an output index for the given node.
    fn into_input_idx(self, node: &Node) -> u32;
}

impl IntoOutputIdx for u32 {
    #[inline]
    fn into_output_idx(self, node: &Node) -> u32 {
        assert!(
            self < node.num_outputs() as u32,
            "output index out of bounds"
        );
        self
    }
}

impl IntoInputIdx for u32 {
    #[inline]
    fn into_input_idx(self, node: &Node) -> u32 {
        assert!(self < node.num_inputs() as u32, "input index out of bounds");
        self
    }
}

impl IntoInputIdx for &str {
    #[track_caller]
    #[inline]
    fn into_input_idx(self, node: &Node) -> u32 {
        let Some(idx) = node.graph().with_graph(|graph| {
            graph.digraph()[node.id()]
                .input_spec()
                .iter()
                .position(|s| s.name == self)
        }) else {
            panic!("no input with name {self}")
        };
        idx as u32
    }
}

impl IntoOutputIdx for &str {
    #[track_caller]
    #[inline]
    fn into_output_idx(self, node: &Node) -> u32 {
        let Some(idx) = node.graph().with_graph(|graph| {
            graph.digraph()[node.id()]
                .output_spec()
                .iter()
                .position(|s| s.name == self)
        }) else {
            panic!("no output with name {self}")
        };
        idx as u32
    }
}

macro_rules! impl_binary_node_ops {
    ($name:ident, $proc:ty, $doc:expr) => {
        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoNode) -> Node {
                let other = other.into_node(self.graph());
                self.assert_single_output();
                other.assert_single_output();

                let processor = <$proc>::default();
                let node = self.graph().add(processor);
                node.input(0).connect(&self.output(0));
                node.input(1).connect(&other.output(0));

                node
            }
        }
    };
    ($name:ident, $std_op:ident, $proc:ty, $doc:expr) => {
        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoNode) -> Node {
                let other = other.into_node(self.graph());
                self.assert_single_output();
                other.assert_single_output();

                let processor = <$proc>::default();
                let node = self.graph().add(processor);
                node.input(0).connect(&self.output(0));
                node.input(1).connect(&other.output(0));

                node
            }
        }

        impl<T> std::ops::$std_op<T> for Node
        where
            T: IntoNode,
        {
            type Output = Node;

            fn $name(self, other: T) -> Node {
                Node::$name(&self, other)
            }
        }

        impl<T> std::ops::$std_op<T> for &Node
        where
            T: IntoNode,
        {
            type Output = Node;

            fn $name(self, other: T) -> Node {
                Node::$name(self, other)
            }
        }

        impl std::ops::$std_op<Node> for Sample {
            type Output = Node;

            fn $name(self, other: Node) -> Node {
                Node::$name(&other, self)
            }
        }

        impl std::ops::$std_op<&Node> for Sample {
            type Output = Node;

            fn $name(self, other: &Node) -> Node {
                Node::$name(other, self)
            }
        }
    };
}

impl_binary_node_ops!(add, Add, math::Add, "Adds two signals together.");
impl_binary_node_ops!(sub, Sub, math::Sub, "Subtracts one signal from another.");
impl_binary_node_ops!(mul, Mul, math::Mul, "Multiplies two signals together.");
impl_binary_node_ops!(div, Div, math::Div, "Divides one signal by another.");
impl_binary_node_ops!(
    rem,
    Rem,
    math::Rem,
    "Calculates the remainder of one signal divided by another."
);
impl_binary_node_ops!(
    powf,
    math::Powf,
    "Raises one signal to the power of another."
);
impl_binary_node_ops!(
    atan2,
    math::Atan2,
    "Calculates the arctangent of the ratio of two signals."
);
impl_binary_node_ops!(
    hypot,
    math::Hypot,
    "Calculates the hypotenuse of two signals."
);
impl_binary_node_ops!(max, math::Max, "Returns the maximum of two signals.");
impl_binary_node_ops!(min, math::Min, "Returns the minimum of two signals.");

macro_rules! impl_unary_node_ops {
    ($name:ident, $proc:ty, $doc:expr) => {
        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self) -> Node {
                self.assert_single_output();

                let processor = <$proc>::default();
                let node = self.graph().add(processor);
                node.input(0).connect(&self.output(0));

                node
            }
        }
    };
}

impl_unary_node_ops!(neg, math::Neg, "Negates the input signal.");

impl std::ops::Neg for &Node {
    type Output = Node;

    fn neg(self) -> Node {
        Node::neg(self)
    }
}

impl_unary_node_ops!(
    abs,
    math::Abs,
    "Calculates the absolute value of the input signal."
);
impl_unary_node_ops!(
    sqrt,
    math::Sqrt,
    "Calculates the square root of the input signal."
);
impl_unary_node_ops!(
    cbrt,
    math::Cbrt,
    "Calculates the cube root of the input signal."
);
impl_unary_node_ops!(
    ceil,
    math::Ceil,
    "Rounds the input signal up to the nearest integer."
);
impl_unary_node_ops!(
    floor,
    math::Floor,
    "Rounds the input signal down to the nearest integer."
);
impl_unary_node_ops!(
    round,
    math::Round,
    "Rounds the input signal to the nearest integer."
);
impl_unary_node_ops!(sin, math::Sin, "Calculates the sine of the input signal.");
impl_unary_node_ops!(cos, math::Cos, "Calculates the cosine of the input signal.");
impl_unary_node_ops!(
    tan,
    math::Tan,
    "Calculates the tangent of the input signal."
);

impl_unary_node_ops!(
    recip,
    math::Recip,
    "Calculates the reciprocal of the input signal."
);
impl_unary_node_ops!(
    signum,
    math::Signum,
    "Returns the sign of the input signal."
);
impl_unary_node_ops!(
    fract,
    math::Fract,
    "Returns the fractional part of the input signal."
);
