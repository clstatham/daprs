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
    pub use crate::runtime::{Backend, Runtime};
    pub use crate::sample::{Buffer, Sample};
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    pub fn test_runtime_offline() {
        let graph = GraphBuilder::new();
        let time = graph.add_processor(Time::default());
        let two_pi = graph.add_processor(Constant::new(std::f64::consts::TAU.into()));
        let freq = graph.add_processor(Constant::new(2.0.into()));

        let sine_wave = (time * freq * two_pi).sin();

        let out = graph.add_output();
        out.connect_inputs([(sine_wave, 0)]);

        let mut runtime = Runtime::new(graph.build());

        let bufs = runtime.run_offline(std::time::Duration::from_secs(2), 32.0, 64);
        assert_eq!(bufs.len(), 1);
        let buf = &bufs[0];
        assert_eq!(buf.len(), 64);

        let mut sum = 0.0f64;
        for i in 0..64 {
            sum += *buf[i];
            // println!("{}", *buf[i]);
        }
        assert!(sum.abs() < 1e-5);
    }
}
