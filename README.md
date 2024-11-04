# raug

**raug** is a library for writing and running digital audio processors and signal flow graphs in Rust.

## Design Goals

- Fast, lightweight, zero-copy where possible
- Stack memory >>> Heap memory
- No allocations on the realtime audio thread
- Do as much work ahead of time as possible

## License

MIT / Apache 2.0
