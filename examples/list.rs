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
    len.input("list").connect(&list);

    let metro = graph.add(Metro::new(1.0));
    let counter = graph.add(Counter::default());
    counter.input("trig").connect(&metro);

    let cond = graph.add(Cond::<bool>::new());
    let should_reset = counter.ge(&len);
    cond.input("cond").connect(&should_reset);
    cond.input("then").connect(graph.constant(true));
    cond.input("else").connect(graph.constant(false));

    counter.input("reset").connect(cond);

    let get = graph.add(Get::<String>::new());
    get.input("list").connect(&list);
    get.input("index").connect(&counter);

    let print = graph.add(Print::new(None, None));
    print.input("trig").connect(&metro);
    print.input("message").connect(get.cast(SignalType::String));

    let mut runtime = graph.build_runtime();

    runtime
        .simulate(Duration::from_secs(10), 48_000.0, 512)
        .unwrap();
}
