use crate::{
    message::BangMessage,
    prelude::{Process, SignalSpec},
    processor::ProcessorError,
    signal::Signal,
};

#[derive(Debug, Clone)]
pub struct MetroProc {
    period: f64,
    last_time: f64,
    next_time: f64,
    time: f64,
    sample_rate: f64,
}

impl MetroProc {
    pub fn new(period: f64) -> Self {
        Self {
            period,
            last_time: 0.0,
            next_time: 0.0,
            time: 0.0,
            sample_rate: 0.0,
        }
    }

    pub fn next_sample(&mut self) -> bool {
        let out = if self.time >= self.next_time {
            self.last_time = self.time;
            self.next_time = self.time + self.period;
            true
        } else {
            false
        };

        self.time += self.sample_rate.recip();

        out
    }
}

impl Process for MetroProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        _inputs: &[crate::signal::SignalBuffer],
        outputs: &mut [crate::signal::SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let out = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for out in out {
            if self.next_sample() {
                *out = Some(Box::new(BangMessage));
            } else {
                *out = None;
            }
        }

        Ok(())
    }
}
