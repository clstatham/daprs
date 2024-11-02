use super::graph_builder::GraphBuilder;
use crate::builtins::*;
use crate::graph::NodeIndex;

#[derive(Clone, Copy)]
pub struct Node<'a> {
    pub(crate) graph_builder: &'a GraphBuilder,
    pub(crate) node_id: NodeIndex,
}

impl<'a> std::fmt::Debug for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.node_id, f)
    }
}

impl<'a> Node<'a> {
    #[inline]
    pub const fn id(self) -> NodeIndex {
        self.node_id
    }

    #[inline]
    pub fn graph(self) -> &'a GraphBuilder {
        self.graph_builder
    }

    #[inline]
    pub fn num_inputs(self) -> usize {
        self.graph()
            .with_graph(|graph| graph.digraph()[self.id()].inputs().len())
    }

    #[inline]
    pub fn num_outputs(self) -> usize {
        self.graph()
            .with_graph(|graph| graph.digraph()[self.id()].outputs().len())
    }

    #[inline]
    #[track_caller]
    pub(crate) fn assert_single_output(self) -> Self {
        assert_eq!(self.num_outputs(), 1, "expected a single output");
        self
    }

    #[inline]
    #[track_caller]
    pub fn connect_input(
        self,
        source: impl IntoNode<'a>,
        source_output: impl IntoOutputIdx,
        input: impl IntoInputIdx,
    ) -> Self {
        let source = source.into_node(self.graph_builder);
        let source_output = source_output.into_output_idx(source);
        let target_input = input.into_input_idx(self);
        self.graph_builder
            .connect(source.id(), source_output, self.id(), target_input);
        self
    }

    #[inline]
    #[track_caller]
    pub fn connect_output(
        self,
        output: impl IntoOutputIdx,
        target: impl IntoNode<'a>,
        target_input: impl IntoInputIdx,
    ) -> Self {
        let target = target.into_node(self.graph_builder);
        let output_index = output.into_output_idx(self);
        let target_input = target_input.into_input_idx(target);
        self.graph_builder
            .connect(self.id(), output_index, target.id(), target_input);
        self
    }

    /// Converts the node's output from a float message to a sample.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    pub fn m2s(self) -> Node<'a> {
        self.assert_single_output();
        let m2s = self.graph().m2s();
        m2s.connect_input(self, 0, 0);
        m2s
    }

    /// Converts the node's output from a sample to a float message.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    pub fn s2m(self) -> Node<'a> {
        self.assert_single_output();
        let s2m = self.graph().s2m();
        s2m.connect_input(self, 0, 0);
        s2m
    }

    /// Converts the node's output from a float message to an integer message.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    pub fn f2i(self) -> Node<'a> {
        self.assert_single_output();
        let f2i = self.graph().f2i();
        f2i.connect_input(self, 0, 0);
        f2i
    }

    /// Converts the node's output from an integer message to a float message.
    ///
    /// # Panics
    ///
    /// Panics if the node has more than one output.
    #[inline]
    pub fn i2f(self) -> Node<'a> {
        self.assert_single_output();
        let i2f = self.graph().i2f();
        i2f.connect_input(self, 0, 0);
        i2f
    }
}

#[doc(hidden)]
mod sealed {
    pub trait Sealed {}
    impl Sealed for crate::graph::NodeIndex {}
    impl<'a> Sealed for super::Node<'a> {}
    impl Sealed for f64 {}
    impl Sealed for u32 {}
    impl Sealed for &str {}
}

pub trait IntoNode<'a>: sealed::Sealed {
    fn into_node(self, graph_builder: &'a GraphBuilder) -> Node<'a>;
}

impl<'a> IntoNode<'a> for NodeIndex {
    fn into_node(self, graph_builder: &'a GraphBuilder) -> Node<'a> {
        Node {
            graph_builder,
            node_id: self,
        }
    }
}

impl<'a> IntoNode<'a> for Node<'a> {
    fn into_node(self, _graph_builder: &'a GraphBuilder) -> Node<'a> {
        self
    }
}

impl<'a> IntoNode<'a> for f64 {
    fn into_node(self, graph_builder: &'a GraphBuilder) -> Node<'a> {
        graph_builder.constant(self)
    }
}

pub trait IntoInputIdx: sealed::Sealed {
    fn into_input_idx(self, node: Node) -> u32;
}

impl IntoInputIdx for u32 {
    #[inline]
    fn into_input_idx(self, _node: Node) -> u32 {
        self
    }
}

impl IntoInputIdx for &str {
    #[inline]
    #[track_caller]
    fn into_input_idx(self, node: Node) -> u32 {
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

pub trait IntoOutputIdx: sealed::Sealed {
    fn into_output_idx(self, node: Node) -> u32;
}

impl IntoOutputIdx for u32 {
    #[inline]
    fn into_output_idx(self, _node: Node) -> u32 {
        self
    }
}

impl IntoOutputIdx for &str {
    #[inline]
    #[track_caller]
    fn into_output_idx(self, node: Node) -> u32 {
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
        impl<'a> Node<'a> {
            #[allow(clippy::should_implement_trait)]
            pub fn $name(self, other: impl IntoNode<'a>) -> Node<'a> {
                let other = other.into_node(self.graph());
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
        impl<'a> Node<'a> {
            #[allow(clippy::should_implement_trait)]
            pub fn $name(self, other: impl IntoNode<'a>) -> Node<'a> {
                let other = other.into_node(self.graph());
                self.assert_single_output();
                other.assert_single_output();

                let processor = <$proc>::default();
                let node = self.graph().add_processor(processor);
                node.connect_input(self, 0, 0);
                node.connect_input(other, 0, 1);

                node
            }
        }

        impl<'a, T> std::ops::$std_op<T> for Node<'a>
        where
            T: IntoNode<'a>,
        {
            type Output = Node<'a>;

            fn $name(self, other: T) -> Self::Output {
                Node::$name(self, other)
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
        impl<'a> Node<'a> {
            #[allow(clippy::should_implement_trait)]
            pub fn $name(self) -> Node<'a> {
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

impl<'a> std::ops::Neg for Node<'a> {
    type Output = Node<'a>;

    fn neg(self) -> Self::Output {
        Node::neg(self)
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
