# pyPAPR

**pyPAPR** is a WIP realtime digital audio processing runtime for Python.

## Design Goals

- Fast, lightweight, zero-copy where possible
- Stack memory >>> Heap memory
- No allocations on the realtime audio thread
- Do as much work ahead of time as possible

## License

MIT / Apache 2.0