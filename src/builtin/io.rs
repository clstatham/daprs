use crate::{graph::node::Process, sample::Buffer};

pub struct Dac {
    pub num_channels: u8,
}

impl Process for Dac {
    fn name(&self) -> &str {
        "dac"
    }

    fn num_inputs(&self) -> usize {
        self.num_channels as usize
    }

    fn num_outputs(&self) -> usize {
        0
    }

    fn process(&mut self, _inputs: &[Buffer], _outputs: &mut [Buffer]) {}
}
