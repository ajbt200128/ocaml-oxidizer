# ocaml-oxidizer

A Rust crate + CLI embedding a self-contained OCaml 5.4 bytecode interpreter.
Pass it OCaml source, it evaluates and returns a value. No external OCaml install
required at runtime: the bytecode runtime is statically linked and the stdlib is
bundled into the binary.

## Develop

```sh
devenv shell                       # OCaml 5.4 + Rust toolchain
cargo run -- examples/hello.ml
```

## Status

Spike 0: proving a Rust host can drive the embedded bytecode `Toploop` in-process.
See `plans/` for the roadmap (cmis bundling, CLI, stress tests, `networking` feature).
