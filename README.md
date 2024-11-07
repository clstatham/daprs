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

See [examples/processor.rs](https://github.com/clstatham/raug/blob/master/examples/processor.rs) for a simple example of writing a raw audio processor.

See [examples/demo.rs](https://github.com/clstatham/raug/blob/master/examples/demo.rs) for a simple example of building a signal flow graph.

## Related Projects

- Python bindings: [raug-python](https://github.com/clstatham/raug-python)
- GUI using [iced](https://github.com/iced-rs/iced) (WIP): [raug-iced](https://github.com/clstatham/raug-iced)

## Roadmap

- [ ] More built-in processors
- [ ] More examples
- [ ] More tests
- [ ] More bindings (Python? JavaScript?)

## Contributing

This is a personal project, but I'm happy to accept contributions. Please open an issue or PR if you have any ideas or feedback.

## Versioning

This project is in early development and does not yet follow semantic versioning. Breaking changes may occur at any time.

The goal is to reach a somewhat-stable starting point and release version 0.1.0 on crates.io soon(tm).

## License

MIT OR Apache 2.0
