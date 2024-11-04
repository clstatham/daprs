# raug

**raug** is a library for writing and running digital audio processors and signal flow graphs in Rust.

## Design Goals

- Fast, lightweight, zero-copy where possible
- Stack memory >>> Heap memory
- No allocations on the realtime audio thread
- Do as much work ahead of time as possible

## Features

- Two main APIs:
  - `processor` API for writing high-performance raw audio processors
  - `builder` API for ergonomically building signal flow graphs
- Runtime capable of running signal flow graphs, either in realtime or offline
- Save rendered audio to WAV files
- Exclusively uses `f64` audio samples internally for highest precision

## Examples

### Processor API

```rust
use raug::prelude::*;

#[derive(Default, Debug, Clone)]
struct MyProcessor {
    gain: f64,
}

impl Process for 

```

## License

MIT / Apache 2.0
