use crate::proc_fn;

proc_fn!(midi2freq(graph, midi: Control) {
    let two = graph.kr_constant(2.0);
    two.pow((midi - 69.0) / 12.0) * 440.0
});
