use std::time::Duration;

use daprs::prelude::*;

fn main() {
    // initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // create a new graph
    let graph = GraphBuilder::new();

    // add a metronome processor
    let bang = graph.metro(0.1);

    // add a print processor with a its own message
    let print = graph.print(Some("print1"), Some("Goodbye, world!"));

    // add a message processor with a new message
    let new_message = graph.message(StringMessage::new("Hello, world!"));

    // connect the loadbang to trigger the new message
    bang.connect_output(0, new_message, 0);

    // connect the loadbang to trigger the print
    bang.connect_output(0, print, 0);

    // connect the new message to the print as well (comment this line out to see it print its own message)
    new_message.connect_output(0, print, 1);

    // build the graph
    let graph = graph.build();

    // create a new runtime
    let mut runtime = Runtime::new(graph);

    // run the runtime for 1 second
    runtime
        .run_offline(Duration::from_secs(1), 44_100.0, 512)
        .unwrap();
}
