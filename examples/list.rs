use raug::prelude::*;

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let list = graph.constant(List::from(vec![
        "Hello".to_string(),
        "World".to_string(),
        "!".to_string(),
    ]));

    let len = graph.add(Len);
    len.input("list").set(&list);

    let metro = graph.add(Metro::new(1.0));
    let counter = graph.add(Counter::default());
    counter.input("trig").set(&metro);

    let cond = graph.add(Cond::<bool>::new());
    let should_reset = counter.ge(&len);
    cond.input("cond").set(&should_reset);
    cond.input("then").set(graph.constant(true));
    cond.input("else").set(graph.constant(false));

    counter.input("reset").set(cond);

    let get = graph.add(Get::<String>::new());
    get.input("list").set(&list);
    get.input("index").set(&counter);

    let print = graph.add(Print::new(None, None));
    print.input("trig").set(&metro);
    print.input("message").set(get.cast(SignalKind::String));

    let mut runtime = graph.build_runtime();

    runtime
        .simulate(Duration::from_secs(10), 48_000.0, 512)
        .unwrap();
}
