use raug::prelude::*;

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let list = graph.constant(SignalBuffer::from_iter([
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0,
    ]));

    let len = graph.add(Len);
    len.input("list").connect(&list);

    let metro = graph.add(Metro::new(1.0));
    let counter = graph.add(Counter::default());
    counter.input("trig").connect(&metro);

    let cond = graph.add(Cond::new(SignalType::Bool));
    let should_reset = counter.ge(&len);
    cond.input("cond").connect(&should_reset);
    cond.input("then").connect(graph.constant(true));
    cond.input("else").connect(graph.constant(false));

    counter.input("reset").connect(cond);

    let get = graph.add(Get::new(SignalType::Float));
    get.input("list").connect(&list);
    get.input("index").connect(&counter);

    let print = graph.add(Print::new(SignalType::Float));
    print.input("trig").connect(&metro);
    print.input("message").connect(&get);

    let mut runtime = graph.build_runtime();

    runtime
        .simulate(Duration::from_secs(10), 48_000.0, 512)
        .unwrap();
}
