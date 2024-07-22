use crate::prelude::*;
use crate::proc_fn;

proc_fn!(midi2freq(graph, midi) {
    let freq = graph.processor(Lambda::kr(|midi, freq| {
        for (f, m) in freq.iter_mut().zip(midi.iter()) {
            *f = (2.0_f64.powf((**m - 69.0) / 12.0) * 440.0).into();
        }
    }));

    freq.connect_inputs([(midi, 0)]);
    freq
});
