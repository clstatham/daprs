//! Contains the [`StaticNode`] type and related types and traits.

use petgraph::prelude::*;

use crate::{prelude::*, signal::SignalKind};

use super::static_graph_builder::StaticGraphBuilder;

/// A node in a [`StaticGraphBuilder`].
///
/// This type has no lifetime parameter, so it can be used in any context.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StaticNode {
    pub(crate) graph: StaticGraphBuilder,
    pub(crate) node_id: NodeIndex,
}

impl StaticNode {
    #[inline]
    pub(crate) fn id(&self) -> NodeIndex {
        self.node_id
    }

    /// Returns the graph that the node belongs to.
    #[inline]
    pub fn graph(&self) -> &StaticGraphBuilder {
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
            .with_graph(|graph| graph.digraph()[self.id()].inputs().len())
    }

    /// Returns the number of outputs of the node.
    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].outputs().len())
    }

    /// Returns the input of the node at the given index.
    #[inline]
    pub fn input(&self, index: impl IntoStaticInputIdx) -> StaticInput {
        StaticInput {
            node: self.clone(),
            input_index: index.into_static_input_idx(self),
        }
    }

    /// Returns the output of the node at the given index.
    #[inline]
    pub fn output(&self, index: impl IntoStaticOutputIdx) -> StaticOutput {
        StaticOutput {
            node: self.clone(),
            output_index: index.into_static_output_idx(self),
        }
    }

    /// Returns the signal type of the input at the given index.
    #[inline]
    pub fn input_kind(&self, index: impl IntoStaticInputIdx) -> SignalKind {
        let index = index.into_static_input_idx(self);
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].input_spec()[index as usize].kind())
    }

    /// Returns the signal type of the output at the given index.
    #[inline]
    pub fn output_kind(&self, index: impl IntoStaticOutputIdx) -> SignalKind {
        let index = index.into_static_output_idx(self);
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].output_spec()[index as usize].kind())
    }

    /// Connects the given input of this node to the given output of another node.
    #[inline]
    #[track_caller]
    pub fn connect_input(
        &self,
        source: impl IntoStaticNode,
        source_output: impl IntoStaticOutputIdx,
        target_input: impl IntoStaticInputIdx,
    ) {
        let output = source.into_static_node(&self.graph);
        let source_output = source_output.into_static_output_idx(&output);
        let target_input = target_input.into_static_input_idx(self);
        self.graph
            .connect(output.id(), source_output, self.id(), target_input);
    }

    /// Connects the given output of this node to the given input of another node.
    #[inline]
    #[track_caller]
    pub fn connect_output(
        &self,
        output: impl IntoStaticOutputIdx,
        target: impl IntoStaticNode,
        target_input: impl IntoStaticInputIdx,
    ) {
        let target = target.into_static_node(&self.graph);
        let output_index = output.into_static_output_idx(self);
        let target_input = target_input.into_static_input_idx(&target);
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
    pub fn m2s(&self) -> StaticNode {
        self.assert_single_output();
        let proc = self.graph.add_processor(MessageToSampleProc);
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
    pub fn s2m(&self) -> StaticNode {
        self.assert_single_output();
        let proc = self.graph.add_processor(SampleToMessageProc);
        proc.connect_input(self, 0, 0);
        proc
    }
}

/// An input of a node in the graph.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StaticInput {
    pub(crate) node: StaticNode,
    pub(crate) input_index: u32,
}

impl StaticInput {
    /// Sets the value of the input.
    #[inline]
    pub fn set(&self, value: impl IntoStaticNode) {
        let value = value.into_static_node(self.node.graph());
        value.assert_single_output();
        self.node.connect_input(value, 0, self.input_index);
    }

    /// Returns the signal type of the input.
    #[inline]
    pub fn kind(&self) -> SignalKind {
        self.node.input_kind(self.input_index)
    }

    /// Connects the input to the given output.
    #[inline]
    #[track_caller]
    pub fn connect(&self, output: &StaticOutput) {
        self.node
            .connect_input(&output.node, output.output_index, self.input_index);
    }

    /// Creates a parameter for the input.
    #[inline]
    pub fn param(&self) -> Param {
        let param = Param::new();
        let proc = self.node.graph().add_processor(param.clone());
        match self.kind() {
            SignalKind::Message => proc.output(0).connect(self),
            SignalKind::Sample => proc.m2s().output(0).connect(self),
        }
        param
    }
}

/// An output of a node in the graph.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StaticOutput {
    pub(crate) node: StaticNode,
    pub(crate) output_index: u32,
}

impl StaticOutput {
    /// Connects the output to the given input.
    #[inline]
    #[track_caller]
    pub fn connect(&self, input: &StaticInput) {
        self.node
            .connect_output(self.output_index, &input.node, input.input_index);
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for crate::graph::NodeIndex {}
    impl Sealed for super::StaticNode {}
    impl Sealed for &super::StaticNode {}
    impl Sealed for super::Message {}
    impl Sealed for f64 {}
    impl Sealed for u32 {}
    impl Sealed for &str {}
}

/// Trait for converting a value into a static node.
pub trait IntoStaticNode: sealed::Sealed {
    /// Converts the value into a static node.
    fn into_static_node(self, graph: &StaticGraphBuilder) -> StaticNode;
}

impl IntoStaticNode for StaticNode {
    fn into_static_node(self, graph: &StaticGraphBuilder) -> StaticNode {
        StaticNode {
            graph: graph.clone(),
            node_id: self.node_id,
        }
    }
}

impl IntoStaticNode for &StaticNode {
    fn into_static_node(self, graph: &StaticGraphBuilder) -> StaticNode {
        StaticNode {
            graph: graph.clone(),
            node_id: self.node_id,
        }
    }
}

impl IntoStaticNode for NodeIndex {
    fn into_static_node(self, graph: &StaticGraphBuilder) -> StaticNode {
        StaticNode {
            graph: graph.clone(),
            node_id: self,
        }
    }
}

impl IntoStaticNode for f64 {
    fn into_static_node(self, graph: &StaticGraphBuilder) -> StaticNode {
        graph.constant(self)
    }
}

impl IntoStaticNode for Message {
    fn into_static_node(self, graph: &StaticGraphBuilder) -> StaticNode {
        graph.constant_message(self)
    }
}

/// Trait for converting a value into an input index for a node.
pub trait IntoStaticOutputIdx: sealed::Sealed {
    /// Converts the value into an input index for the given node.
    fn into_static_output_idx(self, node: &StaticNode) -> u32;
}

/// Trait for converting a value into an output index for a node.
pub trait IntoStaticInputIdx: sealed::Sealed {
    /// Converts the value into an output index for the given node.
    fn into_static_input_idx(self, node: &StaticNode) -> u32;
}

impl IntoStaticOutputIdx for u32 {
    #[inline]
    fn into_static_output_idx(self, _: &StaticNode) -> u32 {
        self
    }
}

impl IntoStaticInputIdx for u32 {
    #[inline]
    fn into_static_input_idx(self, _: &StaticNode) -> u32 {
        self
    }
}

impl IntoStaticInputIdx for &str {
    #[track_caller]
    #[inline]
    fn into_static_input_idx(self, node: &StaticNode) -> u32 {
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

impl IntoStaticOutputIdx for &str {
    #[track_caller]
    #[inline]
    fn into_static_output_idx(self, node: &StaticNode) -> u32 {
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
        impl StaticNode {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoStaticNode) -> StaticNode {
                let other = other.into_static_node(self.graph());
                self.assert_single_output();
                other.assert_single_output();

                let processor = <$proc>::default();
                let node = self.graph().add_processor(processor);
                node.connect_input(self, 0, 0);
                node.connect_input(other, 0, 1);

                node
            }
        }
    };
    ($name:ident, $std_op:ident, $proc:ty, $doc:expr) => {
        impl StaticNode {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoStaticNode) -> StaticNode {
                let other = other.into_static_node(self.graph());
                self.assert_single_output();
                other.assert_single_output();

                let processor = <$proc>::default();
                let node = self.graph().add_processor(processor);
                node.connect_input(self, 0, 0);
                node.connect_input(other, 0, 1);

                node
            }
        }

        impl<T> std::ops::$std_op<T> for &StaticNode
        where
            T: IntoStaticNode,
        {
            type Output = StaticNode;

            fn $name(self, other: T) -> StaticNode {
                StaticNode::$name(self, other)
            }
        }
    };
}

impl_binary_node_ops!(add, Add, math::AddProc, "Adds two signals together.");
impl_binary_node_ops!(
    sub,
    Sub,
    math::SubProc,
    "Subtracts one signal from another."
);
impl_binary_node_ops!(mul, Mul, math::MulProc, "Multiplies two signals together.");
impl_binary_node_ops!(div, Div, math::DivProc, "Divides one signal by another.");
impl_binary_node_ops!(
    rem,
    Rem,
    math::RemProc,
    "Calculates the remainder of one signal divided by another."
);
impl_binary_node_ops!(
    powf,
    math::PowfProc,
    "Raises one signal to the power of another."
);
impl_binary_node_ops!(
    atan2,
    math::Atan2Proc,
    "Calculates the arctangent of the ratio of two signals."
);
impl_binary_node_ops!(
    hypot,
    math::HypotProc,
    "Calculates the hypotenuse of two signals."
);
impl_binary_node_ops!(max, math::MaxProc, "Returns the maximum of two signals.");
impl_binary_node_ops!(min, math::MinProc, "Returns the minimum of two signals.");

macro_rules! impl_unary_node_ops {
    ($name:ident, $proc:ty, $doc:expr) => {
        impl StaticNode {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self) -> StaticNode {
                self.assert_single_output();

                let processor = <$proc>::default();
                let node = self.graph().add_processor(processor);
                node.connect_input(self, 0, 0);

                node
            }
        }
    };
}

impl_unary_node_ops!(neg, math::NegProc, "Negates the input signal.");

impl std::ops::Neg for &StaticNode {
    type Output = StaticNode;

    fn neg(self) -> StaticNode {
        StaticNode::neg(self)
    }
}

impl_unary_node_ops!(
    abs,
    math::AbsProc,
    "Calculates the absolute value of the input signal."
);
impl_unary_node_ops!(
    sqrt,
    math::SqrtProc,
    "Calculates the square root of the input signal."
);
impl_unary_node_ops!(
    cbrt,
    math::CbrtProc,
    "Calculates the cube root of the input signal."
);
impl_unary_node_ops!(
    ceil,
    math::CeilProc,
    "Rounds the input signal up to the nearest integer."
);
impl_unary_node_ops!(
    floor,
    math::FloorProc,
    "Rounds the input signal down to the nearest integer."
);
impl_unary_node_ops!(
    round,
    math::RoundProc,
    "Rounds the input signal to the nearest integer."
);
impl_unary_node_ops!(
    sin,
    math::SinProc,
    "Calculates the sine of the input signal."
);
impl_unary_node_ops!(
    cos,
    math::CosProc,
    "Calculates the cosine of the input signal."
);
impl_unary_node_ops!(
    tan,
    math::TanProc,
    "Calculates the tangent of the input signal."
);
impl_unary_node_ops!(
    asin,
    math::AsinProc,
    "Calculates the arcsine of the input signal."
);
impl_unary_node_ops!(
    acos,
    math::AcosProc,
    "Calculates the arccosine of the input signal."
);
impl_unary_node_ops!(
    atan,
    math::AtanProc,
    "Calculates the arctangent of the input signal."
);
impl_unary_node_ops!(
    recip,
    math::RecipProc,
    "Calculates the reciprocal of the input signal."
);
impl_unary_node_ops!(
    signum,
    math::SignumProc,
    "Returns the sign of the input signal."
);
impl_unary_node_ops!(
    fract,
    math::FractProc,
    "Returns the fractional part of the input signal."
);
