#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub source_output: u32,
    pub target_input: u32,
}

impl Edge {
    pub fn new(source_output: u32, target_input: u32) -> Self {
        Edge {
            source_output,
            target_input,
        }
    }
}

impl std::fmt::Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}->{}", self.source_output, self.target_input)
    }
}
