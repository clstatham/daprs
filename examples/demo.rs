// use std::time::Duration;

// use daprs::prelude::*;

fn main() {
    // initialize logging
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter("PAPR_LOG")
            .default_filter_or("info"),
    );
}
