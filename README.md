# DAPR

**DAPR** is a WIP crate for writing **D**igital **A**udio **P**rocessors in **R**ust.

## Design Goals

- Fast, lightweight, zero-copy where possible
- Stack memory >>> Heap memory
- No allocations on the realtime audio thread
- Do as much work ahead of time as possible
- Type safety and type-checked graph construction

## License

MIT / Apache 2.0
