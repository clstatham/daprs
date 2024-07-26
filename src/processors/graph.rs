use crate::prelude::*;

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

    fn input_spec(&self) -> Vec<SignalSpec> {
        let mut spec = vec![];
        for input in self.graph.inputs() {
            spec.push(input.spec);
        }
        spec
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        let mut spec = vec![];
        for output in self.graph.outputs() {
            spec.push(output.spec);
        }
        spec
    }

    fn num_inputs(&self) -> usize {
        self.graph.num_inputs()
    }

    fn num_outputs(&self) -> usize {
        self.graph.num_outputs()
    }

    fn resize_buffers(&mut self, audio_rate: f64, control_rate: f64, block_size: usize) {
        self.graph.reset(audio_rate, control_rate, block_size);
    }

    fn prepare(&mut self) {
        self.graph.prepare_nodes();
    }

    fn process(&mut self, inputs: &[Signal], outputs: &mut [Signal]) {
        for (i, input) in inputs.iter().enumerate() {
            self.graph.copy_input(i, input);
        }

        self.graph.process();

        for (i, output) in outputs.iter_mut().enumerate() {
            output.copy_from(self.graph.get_output(i));
        }
    }
}
