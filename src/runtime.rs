use crate::{
    graph::Graph,
    sample::{Buffer, Sample},
};

#[derive(Default)]
pub struct Runtime {
    graph: Graph,
    sample_rate: f64,
    block_size: usize,
}

impl Runtime {
    pub fn new(sample_rate: f64, block_size: usize) -> Self {
        Runtime {
            graph: Graph::new(),
            sample_rate,
            block_size,
        }
    }

    pub fn from_graph(graph: Graph, sample_rate: f64, block_size: usize) -> Self {
        Runtime {
            graph,
            sample_rate,
            block_size,
        }
    }

    pub fn reset(&mut self) {
        self.graph.reset(self.sample_rate, self.block_size);
    }

    pub fn prepare(&mut self) {
        self.graph.prepare_nodes();
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.reset();
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn set_block_size(&mut self, block_size: usize) {
        self.block_size = block_size;
        self.reset();
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    pub fn outputs(&mut self) -> impl Iterator<Item = &[Sample]> + '_ {
        let num_outputs = self.graph.num_outputs();
        (0..num_outputs).map(|i| self.graph.get_output(i))
    }

    #[inline]
    pub fn next_buffer(&mut self) -> &[Buffer] {
        self.graph.process();

        self.graph.outputs()
    }

    pub fn run_offline(&mut self, duration: std::time::Duration) -> Box<[Box<[Sample]>]> {
        let secs = duration.as_secs_f64();
        let samples = (self.sample_rate * secs) as usize;
        let blocks = samples / self.block_size;

        self.reset();

        let num_outputs = self.graph.num_outputs();

        let mut outputs: Box<[Box<[Sample]>]> =
            vec![vec![Sample::default(); num_outputs].into_boxed_slice(); samples]
                .into_boxed_slice();

        for block_index in 0..blocks {
            self.graph.process();

            for (output_index, output) in outputs[block_index * self.block_size..]
                .iter_mut()
                .enumerate()
            {
                output.copy_from_slice(self.graph.get_output(output_index));
            }
        }

        outputs
    }
}
