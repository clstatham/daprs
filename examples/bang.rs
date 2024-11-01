use std::time::Duration;

use daprs::prelude::*;

fn main() {
    // initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // create a new graph
    let graph = GraphBuilder::new();

    // add a metronome node with a period of 0.5 seconds
    let bang = graph.metro(0.5);

    // add a print node with a its own initial message
    let print1 = graph.print(Some("print1"), Some("Goodbye, world!"));

    // add a message node with a new message
    let new_message = graph.message("Hello, world!".to_string());

    // connect the metronome to trigger the new message
    bang.connect_output(0, new_message, 0);

    // connect the metronome to trigger the print
    bang.connect_output(0, print1, "print");

    // connect the new message to the print's "message" input (comment this line out to see it print its own message)
    new_message.connect_output(0, print1, "foobar");

    // build the graph
    let graph = graph.build();

    // create a new runtime
    let mut runtime = Runtime::new(graph);

    // run the runtime for 5 seconds
    // this will print a message every 0.5 seconds!
    runtime
        .simulate(Duration::from_secs(5), 44_100.0, 512)
        .unwrap();
}
