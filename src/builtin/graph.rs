use crate::{
    graph::{node::Process, Graph},
    sample::Buffer,
};

#[derive(Default, Clone)]
pub struct SubGraph {
    pub graph: Graph,
}

impl SubGraph {
    pub fn new(graph: Graph) -> Self {
        Self { graph }
    }
}

impl Process for SubGraph {
    fn name(&self) -> &str {
        "graph"
    }

    fn num_inputs(&self) -> usize {
        self.graph.num_inputs()
    }

    fn num_outputs(&self) -> usize {
        self.graph.num_outputs()
    }

    fn reset(&mut self, sample_rate: f64, block_size: usize) {
        self.graph.reset(sample_rate, block_size);
    }

    fn prepare(&mut self) {
        self.graph.prepare_nodes();
    }

    fn process(&mut self, inputs: &[Buffer], outputs: &mut [Buffer]) {
        for (i, input) in inputs.iter().enumerate() {
            self.graph.copy_input(i, input);
        }

        self.graph.process();

        for (i, output) in outputs.iter_mut().enumerate() {
            output.copy_from_slice(self.graph.get_output(i));
        }
    }
}
