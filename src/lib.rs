pub mod builtin;
pub mod graph;
pub mod runtime;
pub mod sample;

pub mod prelude {
    pub use crate::builtin::{graph::*, io::*, math::*, time::*};
    pub use crate::graph::{
        builder::{GraphBuilder, GraphBuilderNode},
        edge::Edge,
        node::*,
        Graph,
    };
    pub use crate::runtime::Runtime;
    pub use crate::sample::{Buffer, Sample};
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    pub fn test_graph() {
        let graph = GraphBuilder::new();
        let time = graph.add_processor(Time::default());
        let two_pi = graph.add_processor(Constant::new(std::f64::consts::TAU.into()));
        let freq = graph.add_processor(Constant::new(2.0.into()));

        let sine_wave = (time * freq * two_pi).sin();

        let out = graph.add_output();
        out.connect_inputs([(sine_wave, 0)]);

        let mut runtime = Runtime::from_graph(graph.build(), 32.0, 32);

        runtime.reset();
        runtime.prepare();

        let bufs = runtime.next_buffer();
        assert_eq!(bufs.len(), 1);
        let buf = &bufs[0];
        assert_eq!(buf.len(), 32);

        let mut sum = 0.0f64;
        for i in 0..32 {
            sum += *buf[i];
            // println!("{}", *buf[i]);
        }
        assert!(sum.abs() < 1e-5);
    }
}
