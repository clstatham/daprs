//! Contains the [`Node`] type and related types and traits.

use petgraph::prelude::*;

use crate::{
    prelude::*,
    signal::{SignalData, SignalKind},
};

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
            .with_graph(|graph| graph.digraph()[self.id()].num_inputs())
    }

    /// Returns the number of outputs of the node.
    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].num_outputs())
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

    /// Returns the signal type of the output at the given index.
    #[inline]
    pub fn output_kind(&self, index: impl IntoOutputIdx) -> SignalKind {
        let index = index.into_output_idx(self);
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].output_spec()[index as usize].kind)
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

    /// Smooths the output signal.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    #[track_caller]
    pub fn smooth(&self, factor: Sample) -> Node {
        self.assert_single_output();
        self.output(0).smooth(factor)
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
        self.output(0).midi2freq()
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
        self.output(0).freq2midi()
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
        self.output(0).make_register()
    }

    /// Creates a new [`Cond`] node that selects one of its two inputs based on a condition.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    pub fn cond(&self, then: impl IntoNode, else_: impl IntoNode) -> Node {
        self.assert_single_output();
        self.output(0).cond(then, else_)
    }

    /// Creates a new [`Len`] node that outputs the length of the output signal.
    ///
    /// The output signal must be a list.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    pub fn len(&self) -> Node {
        self.assert_single_output();
        self.output(0).len()
    }

    #[inline]
    pub fn cast(&self, kind: SignalKind) -> Node {
        self.assert_single_output();
        self.output(0).cast(kind)
    }
}

/// An input of a node in the graph.
#[derive(Clone)]
pub struct Input {
    pub(crate) node: Node,
    pub(crate) input_index: u32,
}

impl Input {
    /// Returns the signal type of the input.
    #[inline]
    pub fn kind(&self) -> SignalKind {
        self.node.graph.with_graph(|graph| {
            graph.digraph()[self.node.id()].input_spec()[self.input_index as usize].kind
        })
    }

    /// Sets the value of the input.
    #[inline]
    pub fn set(&self, value: impl IntoNode) {
        let value = value.into_node(self.node.graph());
        value.assert_single_output();
        assert_eq!(
            self.kind(),
            value.output_kind(0),
            "output and input signals must have the same type"
        );
        self.node.connect_input(&value, 0, self.input_index);
    }

    /// Returns the node that the input belongs to.
    #[inline]
    pub fn node(&self) -> Node {
        self.node.clone()
    }

    /// Connects the input to the given output.
    #[inline]
    #[track_caller]
    pub fn connect(&self, output: &Output) {
        assert_eq!(
            self.kind(),
            output.kind(),
            "output and input signals must have the same type"
        );
        self.node
            .connect_input(&output.node, output.output_index, self.input_index);
    }

    /// Creates a parameter for the input.
    #[inline]
    pub fn param<S: SignalData>(
        &self,
        name: impl Into<String>,
        initial_value: impl Into<Option<S>>,
    ) -> Param<S> {
        let name = name.into();
        let param = Param::<S>::new(&name, initial_value);
        let proc = self.node.graph().add_param(param.clone());
        proc.output(0).connect(self);
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
        assert_eq!(
            self.kind(),
            input.kind(),
            "output and input signals must have the same type"
        );
        self.node
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

    #[inline]
    pub fn cast(&self, kind: SignalKind) -> Node {
        let current_kind = self.kind();
        if current_kind == kind {
            return self.node.clone();
        }
        let cast = match (current_kind, kind) {
            // bool <-> int
            (SignalKind::Bool, SignalKind::Int) => self.node.graph().add(Cast::<bool, i64>::new()),
            (SignalKind::Int, SignalKind::Bool) => self.node.graph().add(Cast::<i64, bool>::new()),

            // bool <-> sample
            (SignalKind::Bool, SignalKind::Sample) => {
                self.node.graph().add(Cast::<bool, Sample>::new())
            }
            (SignalKind::Sample, SignalKind::Bool) => {
                self.node.graph().add(Cast::<Sample, bool>::new())
            }

            // int <-> sample
            (SignalKind::Int, SignalKind::Sample) => {
                self.node.graph().add(Cast::<i64, Sample>::new())
            }
            (SignalKind::Sample, SignalKind::Int) => {
                self.node.graph().add(Cast::<Sample, i64>::new())
            }

            _ => panic!("cannot cast from {:?} to {:?}", current_kind, kind),
        };

        cast.input(0).connect(self);
        cast
    }

    /// Creates a new, single-output node that passes the value of this output through.
    #[inline]
    pub fn make_node(&self) -> Node {
        let kind = self.kind();
        let node = match kind {
            SignalKind::Dynamic => self.node.graph().add(Passthrough::<Signal>::new()),
            SignalKind::Bool => self.node.graph().add(Passthrough::<bool>::new()),
            SignalKind::Int => self.node.graph().add(Passthrough::<i64>::new()),
            SignalKind::Sample => self.node.graph().add(Passthrough::<Sample>::new()),
            SignalKind::String => self.node.graph().add(Passthrough::<String>::new()),
            SignalKind::List => self.node.graph().add(Passthrough::<List>::new()),
            SignalKind::Midi => self.node.graph().add(Passthrough::<MidiMessage>::new()),
        };
        node.input(0).connect(self);
        node
    }

    /// Creates a new, single-output node that holds and continuously outputs the last value of this output.
    #[inline]
    pub fn make_register(&self) -> Node {
        let kind = self.kind();
        let node = match kind {
            SignalKind::Dynamic => self.node.graph().add(Register::<Signal>::new()),
            SignalKind::Bool => self.node.graph().add(Register::<bool>::new()),
            SignalKind::Int => self.node.graph().add(Register::<i64>::new()),
            SignalKind::Sample => self.node.graph().add(Register::<Sample>::new()),
            SignalKind::String => self.node.graph().add(Register::<String>::new()),
            SignalKind::List => self.node.graph().add(Register::<List>::new()),
            SignalKind::Midi => self.node.graph().add(Register::<MidiMessage>::new()),
        };
        node.input(0).connect(self);
        node
    }

    /// Creates a new, single-output node that smooths the output signal.
    #[inline]
    pub fn smooth(&self, factor: Sample) -> Node {
        let proc = self.node.graph().add(Smooth::default());
        proc.input("factor").set(factor);
        proc.input(0).connect(self);
        proc
    }

    /// Creates a new, single-output node that converts the output signal from a MIDI note to a frequency in Hz.
    #[inline]
    pub fn midi2freq(&self) -> Node {
        let proc = self.node.graph().add(MidiToFreq);
        proc.input(0).connect(self);
        proc
    }

    /// Creates a new, single-output node that converts the output signal from a frequency in Hz to a MIDI note.
    #[inline]
    pub fn freq2midi(&self) -> Node {
        let proc = self.node.graph().add(FreqToMidi);
        proc.input(0).connect(self);
        proc
    }

    /// Creates a new [`Cond`] node that selects one of its two inputs based on the condition given by this output signal.
    #[inline]
    pub fn cond(&self, then: impl IntoNode, else_: impl IntoNode) -> Node {
        let then = then.into_node(self.node.graph());
        let else_ = else_.into_node(self.node.graph());
        then.assert_single_output();
        else_.assert_single_output();
        let kind = then.output_kind(0);
        assert_eq!(
            kind,
            else_.output_kind(0),
            "output signals must have the same type"
        );
        let cond = match kind {
            SignalKind::Dynamic => self.node.graph().add(Cond::<Signal>::new()),
            SignalKind::Bool => self.node.graph().add(Cond::<bool>::new()),
            SignalKind::Int => self.node.graph().add(Cond::<i64>::new()),
            SignalKind::Sample => self.node.graph().add(Cond::<Sample>::new()),
            SignalKind::String => self.node.graph().add(Cond::<String>::new()),
            SignalKind::List => self.node.graph().add(Cond::<List>::new()),
            SignalKind::Midi => self.node.graph().add(Cond::<MidiMessage>::new()),
        };
        cond.input("cond").connect(self);
        cond.input("then").connect(&then.output(0));
        cond.input("else").connect(&else_.output(0));
        cond
    }

    /// Creates a new [`Len`] node that outputs the length of the output signal.
    ///
    /// The output signal must be a list.
    #[inline]
    pub fn len(&self) -> Node {
        let proc = self.node.graph().add(Len);
        proc.input(0).connect(self);
        proc
    }
}

mod sealed {
    use crate::signal::SignalData;

    pub trait Sealed {}
    impl Sealed for crate::graph::NodeIndex {}
    impl Sealed for super::Node {}
    impl Sealed for &super::Node {}
    impl Sealed for super::Signal {}
    impl<S: SignalData> Sealed for super::Param<S> {}
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

impl<S: SignalData> IntoNode for Param<S> {
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

impl IntoNode for Signal {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(self)
    }
}

impl IntoNode for i64 {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(Signal::Int(self))
    }
}

impl IntoNode for u32 {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(Signal::Int(self as i64))
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
impl_binary_node_ops!(max, math::Max, "Outputs the maximum of two signals.");
impl_binary_node_ops!(min, math::Min, "Outputs the minimum of two signals.");

macro_rules! impl_comparison_node_ops {
    ($name:ident, $proc:ident, $doc:expr) => {
        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoNode) -> Node {
                let other = other.into_node(self.graph());
                self.assert_single_output();
                other.assert_single_output();

                let kind = self.output_kind(0);
                assert_eq!(
                    kind,
                    other.output_kind(0),
                    "output signals must have the same type"
                );

                let node = match kind {
                    SignalKind::Dynamic => self.graph().add(control::$proc::<Signal>::new()),
                    SignalKind::Bool => self.graph().add(control::$proc::<bool>::default()),
                    SignalKind::Int => self.graph().add(control::$proc::<i64>::default()),
                    SignalKind::Sample => self.graph().add(control::$proc::<Sample>::default()),
                    SignalKind::String => self.graph().add(control::$proc::<String>::default()),
                    _ => panic!("unsupported signal type"),
                };

                node.input(0).connect(&self.output(0));
                node.input(1).connect(&other.output(0));

                node
            }
        }
    };
}

impl_comparison_node_ops!(eq, Equal, "Outputs true if the two signals are equal.");
impl_comparison_node_ops!(
    ne,
    NotEqual,
    "Outputs true if the two signals are not equal."
);
impl_comparison_node_ops!(
    lt,
    Less,
    "Outputs true if the first signal is less than the second signal."
);
impl_comparison_node_ops!(
    le,
    LessOrEqual,
    "Outputs true if the first signal is less than or equal to the second signal."
);
impl_comparison_node_ops!(
    gt,
    Greater,
    "Outputs true if the first signal is greater than the second signal."
);
impl_comparison_node_ops!(
    ge,
    GreaterOrEqual,
    "Outputs true if the first signal is greater than or equal to the second signal."
);

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
    "Outputs the absolute value of the input signal."
);
impl_unary_node_ops!(
    sqrt,
    math::Sqrt,
    "Outputs the square root of the input signal."
);
impl_unary_node_ops!(
    cbrt,
    math::Cbrt,
    "Outputs the cube root of the input signal."
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
impl_unary_node_ops!(sin, math::Sin, "Outputs the sine of the input signal.");
impl_unary_node_ops!(cos, math::Cos, "Outputs the cosine of the input signal.");
impl_unary_node_ops!(tan, math::Tan, "Outputs the tangent of the input signal.");

impl_unary_node_ops!(
    recip,
    math::Recip,
    "Outputs the reciprocal of the input signal."
);
impl_unary_node_ops!(
    signum,
    math::Signum,
    "Outputs the sign of the input signal."
);
impl_unary_node_ops!(
    fract,
    math::Fract,
    "Outputs the fractional part of the input signal."
);
