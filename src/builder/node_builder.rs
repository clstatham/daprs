//! Contains the [`Node`] type and related types and traits.

use petgraph::prelude::*;

use crate::{
    prelude::*,
    signal::{Signal, SignalType},
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
    pub fn output_kind(&self, index: impl IntoOutputIdx) -> SignalType {
        let index = index.into_output_idx(self);
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].output_spec()[index as usize].type_)
    }

    /// Connects the given input of this node to the given output of another node.
    #[inline]
    #[track_caller]
    pub fn connect_input(
        &self,
        source: impl IntoNode,
        source_output: impl IntoOutputIdx,
        target_input: impl IntoInputIdx,
    ) -> Node {
        let output = source.into_node(&self.graph);
        let source_output = source_output.into_output_idx(&output);
        let target_input = target_input.into_input_idx(self);
        self.graph
            .connect(output.id(), source_output, self.id(), target_input);
        self.clone()
    }

    /// Connects the given output of this node to the given input of another node.
    #[inline]
    #[track_caller]
    pub fn connect_output(
        &self,
        output: impl IntoOutputIdx,
        target: impl IntoNode,
        target_input: impl IntoInputIdx,
    ) -> Node {
        let target = target.into_node(&self.graph);
        let output_index = output.into_output_idx(self);
        let target_input = target_input.into_input_idx(&target);
        self.graph
            .connect(self.id(), output_index, target.id(), target_input);
        self.clone()
    }

    /// Smooths the output signal.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    #[track_caller]
    pub fn smooth(&self, factor: Float) -> Node {
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
    pub fn cast(&self, type_: SignalType) -> Node {
        self.assert_single_output();
        self.output(0).cast(type_)
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
    pub fn type_(&self) -> SignalType {
        self.node.graph.with_graph(|graph| {
            graph.digraph()[self.node.id()].input_spec()[self.input_index as usize].type_
        })
    }

    /// Sets the value of the input.
    #[inline]
    pub fn set(&self, value: impl IntoNode) -> Node {
        let value = value.into_node(self.node.graph());
        value.assert_single_output();
        assert_eq!(
            self.type_(),
            value.output_kind(0),
            "output and input signals must have the same type"
        );
        self.node.connect_input(&value, 0, self.input_index);
        self.node.clone()
    }

    /// Returns the node that the input belongs to.
    #[inline]
    pub fn node(&self) -> Node {
        self.node.clone()
    }

    /// Connects the input to the given output.
    #[inline]
    #[track_caller]
    pub fn connect(&self, output: &Output) -> Node {
        assert_eq!(
            self.type_(),
            output.type_(),
            "output and input signals must have the same type"
        );
        self.node
            .connect_input(&output.node, output.output_index, self.input_index);
        self.node.clone()
    }

    /// Creates a parameter for the input.
    #[inline]
    pub fn param<S: Signal>(
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
    pub fn connect(&self, input: &Input) -> Node {
        assert_eq!(
            self.type_(),
            input.type_(),
            "output and input signals must have the same type"
        );
        self.node
            .connect_output(self.output_index, &input.node, input.input_index);
        self.node.clone()
    }

    /// Returns the node that the output belongs to.
    #[inline]
    pub fn node(&self) -> Node {
        self.node.clone()
    }

    /// Returns the signal type of the output.
    #[inline]
    pub fn type_(&self) -> SignalType {
        self.node.output_kind(self.output_index)
    }

    #[inline]
    pub fn cast(&self, type_: SignalType) -> Node {
        let current_kind = self.type_();
        if current_kind == type_ {
            return self.node.clone();
        }
        let cast = match (current_kind, type_) {
            // bool <-> int
            (SignalType::Bool, SignalType::Int) => self.node.graph().add(Cast::<bool, i64>::new()),
            (SignalType::Int, SignalType::Bool) => self.node.graph().add(Cast::<i64, bool>::new()),

            // bool <-> sample
            (SignalType::Bool, SignalType::Float) => {
                self.node.graph().add(Cast::<bool, Float>::new())
            }
            (SignalType::Float, SignalType::Bool) => {
                self.node.graph().add(Cast::<Float, bool>::new())
            }

            // int <-> sample
            (SignalType::Int, SignalType::Float) => {
                self.node.graph().add(Cast::<i64, Float>::new())
            }
            (SignalType::Float, SignalType::Int) => {
                self.node.graph().add(Cast::<Float, i64>::new())
            }

            // string <-> sample
            (SignalType::String, SignalType::Float) => {
                self.node.graph().add(Cast::<String, Float>::new())
            }
            (SignalType::Float, SignalType::String) => {
                self.node.graph().add(Cast::<Float, String>::new())
            }

            // string <-> int
            (SignalType::String, SignalType::Int) => {
                self.node.graph().add(Cast::<String, i64>::new())
            }
            (SignalType::Int, SignalType::String) => {
                self.node.graph().add(Cast::<i64, String>::new())
            }

            _ => panic!("cannot cast from {:?} to {:?}", current_kind, type_),
        };

        cast.input(0).connect(self);
        cast
    }

    /// Creates a new, single-output node that passes the value of this output through.
    #[inline]
    pub fn make_node(&self) -> Node {
        let type_ = self.type_();
        let node = match type_ {
            SignalType::Dynamic => self.node.graph().add(Passthrough::<AnySignal>::new()),
            SignalType::Bool => self.node.graph().add(Passthrough::<bool>::new()),
            SignalType::Int => self.node.graph().add(Passthrough::<i64>::new()),
            SignalType::Float => self.node.graph().add(Passthrough::<Float>::new()),
            SignalType::String => self.node.graph().add(Passthrough::<String>::new()),
            SignalType::List => self.node.graph().add(Passthrough::<List>::new()),
            SignalType::Midi => self.node.graph().add(Passthrough::<MidiMessage>::new()),
        };
        node.input(0).connect(self);
        node
    }

    /// Creates a new, single-output node that holds and continuously outputs the last value of this output.
    #[inline]
    pub fn make_register(&self) -> Node {
        let type_ = self.type_();
        let node = match type_ {
            SignalType::Dynamic => self.node.graph().add(Register::<AnySignal>::new()),
            SignalType::Bool => self.node.graph().add(Register::<bool>::new()),
            SignalType::Int => self.node.graph().add(Register::<i64>::new()),
            SignalType::Float => self.node.graph().add(Register::<Float>::new()),
            SignalType::String => self.node.graph().add(Register::<String>::new()),
            SignalType::List => self.node.graph().add(Register::<List>::new()),
            SignalType::Midi => self.node.graph().add(Register::<MidiMessage>::new()),
        };
        node.input(0).connect(self);
        node
    }

    /// Creates a new, single-output node that smooths the output signal.
    #[inline]
    pub fn smooth(&self, factor: Float) -> Node {
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
        let type_ = then.output_kind(0);
        assert_eq!(
            type_,
            else_.output_kind(0),
            "output signals must have the same type"
        );
        let cond = match type_ {
            SignalType::Dynamic => self.node.graph().add(Cond::<AnySignal>::new()),
            SignalType::Bool => self.node.graph().add(Cond::<bool>::new()),
            SignalType::Int => self.node.graph().add(Cond::<i64>::new()),
            SignalType::Float => self.node.graph().add(Cond::<Float>::new()),
            SignalType::String => self.node.graph().add(Cond::<String>::new()),
            SignalType::List => self.node.graph().add(Cond::<List>::new()),
            SignalType::Midi => self.node.graph().add(Cond::<MidiMessage>::new()),
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
        assert_eq!(
            self.type_(),
            SignalType::List,
            "output signal must be a list"
        );
        let proc = self.node.graph().add(Len);
        proc.input(0).connect(self);
        proc
    }
}

mod sealed {
    use crate::signal::Signal;

    pub trait Sealed {}
    impl Sealed for crate::graph::NodeIndex {}
    impl Sealed for super::Node {}
    impl Sealed for &super::Node {}
    impl Sealed for super::AnySignal {}
    impl<S: Signal> Sealed for super::Param<S> {}
    impl Sealed for crate::signal::Float {}
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

impl<S: Signal> IntoNode for Param<S> {
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

impl IntoNode for Float {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(self)
    }
}

impl IntoNode for AnySignal {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(self)
    }
}

impl IntoNode for i64 {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(AnySignal::Int(self))
    }
}

impl IntoNode for u32 {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(AnySignal::Int(self as i64))
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
    ($name:ident, $proc:ident, ($($type_:ident => $data:ty),*), $doc:literal) => {
        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoNode) -> Node {
                let other = other.into_node(self.graph());
                self.assert_single_output();
                other.assert_single_output();

                assert_eq!(
                    self.output_kind(0),
                    other.output_kind(0),
                    "output signals must have the same type"
                );

                let type_ = self.output_kind(0);
                let node = match type_ {
                    $(SignalType::$type_ => self.graph().add(<math::$proc<$data>>::default()),)*
                    _ => panic!("unsupported signal type for {:?}: {:?}", stringify!($name), type_),
                };

                node.input(0).connect(&self.output(0));
                node.input(1).connect(&other.output(0));

                node
            }
        }
    };
    ($name:ident, $std_op:ident, $proc:ident, ($($type_:ident => $data:ty),*), $doc:literal) => {
        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoNode) -> Node {
                let other = other.into_node(self.graph());
                self.assert_single_output();
                other.assert_single_output();

                assert_eq!(
                    self.output_kind(0),
                    other.output_kind(0),
                    "output signals must have the same type"
                );

                let type_ = self.output_kind(0);

                let node = match type_ {
                    $(SignalType::$type_ => self.graph().add(<math::$proc<$data>>::default()),)*
                    _ => panic!("unsupported signal type for {:?}: {:?}", stringify!($name), type_),
                };

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

        impl std::ops::$std_op<Node> for Float {
            type Output = Node;

            fn $name(self, other: Node) -> Node {
                Node::$name(&other, self)
            }
        }

        impl std::ops::$std_op<&Node> for Float {
            type Output = Node;

            fn $name(self, other: &Node) -> Node {
                Node::$name(other, self)
            }
        }
    };
}

impl_binary_node_ops!(add, Add, Add, (Float => Float, Int => i64), "Adds two signals together.");
impl_binary_node_ops!(sub, Sub, Sub, (Float => Float, Int => i64), "Subtracts one signal from another.");
impl_binary_node_ops!(mul, Mul, Mul, (Float => Float, Int => i64), "Multiplies two signals together.");
impl_binary_node_ops!(div, Div, Div, (Float => Float, Int => i64), "Divides one signal by another.");
impl_binary_node_ops!(
    rem,
    Rem,
    Rem,
    (Float => Float, Int => i64),
    "Calculates the remainder of one signal divided by another."
);
impl_binary_node_ops!(powf, Powf, (Float => Float), "Raises one signal to the power of another.");
impl_binary_node_ops!(
    atan2,
    Atan2,
    (Float => Float),
    "Calculates the arctangent of the ratio of two signals."
);
impl_binary_node_ops!(hypot, Hypot, (Float => Float), "Calculates the hypotenuse of two signals.");
impl_binary_node_ops!(max, Max, (Float => Float, Int => i64), "Outputs the maximum of two signals.");
impl_binary_node_ops!(min, Min, (Float => Float, Int => i64), "Outputs the minimum of two signals.");

macro_rules! impl_comparison_node_ops {
    ($name:ident, $proc:ident, $doc:expr) => {
        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoNode) -> Node {
                let other = other.into_node(self.graph());
                self.assert_single_output();
                other.assert_single_output();

                let type_ = self.output_kind(0);
                assert_eq!(
                    type_,
                    other.output_kind(0),
                    "output signals must have the same type"
                );

                let node = match type_ {
                    SignalType::Dynamic => self.graph().add(control::$proc::<AnySignal>::new()),
                    SignalType::Bool => self.graph().add(control::$proc::<bool>::default()),
                    SignalType::Int => self.graph().add(control::$proc::<i64>::default()),
                    SignalType::Float => self.graph().add(control::$proc::<Float>::default()),
                    SignalType::String => self.graph().add(control::$proc::<String>::default()),
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
    ($name:ident, $proc:ident, ($($type_:ident => $data:ty),*), $doc:literal) => {
        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self) -> Node {
                self.assert_single_output();

                let type_ = self.output_kind(0);

                let node = match type_ {
                    $(SignalType::$type_ => self.graph().add(<math::$proc<$data>>::default()),)*
                    _ => panic!("unsupported signal type for {:?}: {:?}", stringify!($name), type_),
                };

                node.input(0).connect(&self.output(0));

                node
            }
        }
    };
}

impl_unary_node_ops!(neg, Neg, (Float => Float, Int => i64), "Negates the input signal.");

impl std::ops::Neg for &Node {
    type Output = Node;

    fn neg(self) -> Node {
        Node::neg(self)
    }
}

impl_unary_node_ops!(
    abs,
    Abs,
    (Float => Float, Int => i64),
    "Outputs the absolute value of the input signal."
);
impl_unary_node_ops!(
    sqrt,
    Sqrt,
    (Float => Float),
    "Outputs the square root of the input signal."
);
impl_unary_node_ops!(
    cbrt,
    Cbrt,
    (Float => Float),
    "Outputs the cube root of the input signal."
);
impl_unary_node_ops!(
    ceil,
    Ceil,
    (Float => Float),
    "Rounds the input signal up to the nearest integer."
);
impl_unary_node_ops!(
    floor,
    Floor,
    (Float => Float),
    "Rounds the input signal down to the nearest integer."
);
impl_unary_node_ops!(
    round,
    Round,
    (Float => Float),
    "Rounds the input signal to the nearest integer."
);
impl_unary_node_ops!(sin, Sin, (Float => Float), "Outputs the sine of the input signal.");
impl_unary_node_ops!(cos, Cos, (Float => Float), "Outputs the cosine of the input signal.");
impl_unary_node_ops!(tan, Tan, (Float => Float), "Outputs the tangent of the input signal.");
impl_unary_node_ops!(
    tanh,
    Tanh,
    (Float => Float),
    "Outputs the hyperbolic tangent of the input signal."
);

impl_unary_node_ops!(
    recip,
    Recip,
    (Float => Float),
    "Outputs the reciprocal of the input signal."
);
impl_unary_node_ops!(
    signum,
    Signum,
    (Float => Float, Int => i64),
    "Outputs the sign of the input signal."
);
impl_unary_node_ops!(
    fract,
    Fract,
    (Float => Float),
    "Outputs the fractional part of the input signal."
);
impl_unary_node_ops!(
    ln,
    Ln,
    (Float => Float),
    "Outputs the natural logarithm of the input signal."
);
impl_unary_node_ops!(
    log2,
    Log2,
    (Float => Float),
    "Outputs the base-2 logarithm of the input signal."
);
impl_unary_node_ops!(
    log10,
    Log10,
    (Float => Float),
    "Outputs the base-10 logarithm of the input signal."
);
impl_unary_node_ops!(
    exp,
    Exp,
    (Float => Float),
    "Outputs the natural exponential of the input signal."
);
