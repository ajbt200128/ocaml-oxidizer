# ocaml-oxidizer

A Rust crate + CLI that embeds a self-contained OCaml 5.4 bytecode interpreter.
The bytecode runtime, stdlib, and Unix are statically linked, and the stdlib/unix
`.cmi` files are bundled into the binary — no OCaml install is needed at runtime.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/ajbt200128/ocaml-oxidizer/main/install.sh | sh
```

Installs the latest release into `~/.local/bin`.

## Usage

```sh
ocaml-oxidizer script.ml          # run a script
ocaml-oxidizer --check script.ml  # typecheck only (pretty errors)
```

Scripts get the full stdlib and Unix: stdout/stderr, stdin, process exec,
effects and domains, and `Callback.register`. Built with the `networking`
feature, scripts also get `http_get`, `http_status`, and `load_remote` (fetch
and evaluate a remote `.ml`).

## Library

```rust
let interp = ocaml_oxidizer::Interp::new();
interp.run("main.ml", "let () = print_endline \"hi\"")?;
let n = interp.eval_int("6 * 7")?;                       // 42
interp.run("_", "let () = Callback.register \"inc\" (fun x -> x + 1)")?;
let m = interp.call_int("inc", 41)?;                     // 42
```

## Build from source

```sh
devenv shell                                  # OCaml 5.4 + Rust toolchain
cargo build --release --features networking
cargo test --all-features
```

A fully static Linux (musl) binary builds via the provided image, which also
runs the stress suite and proves self-containment on bare Alpine:

```sh
docker build -f docker/Dockerfile.alpine -t ocaml-oxidizer .
```

## Use in GitHub Actions

```yaml
- uses: ajbt200128/ocaml-oxidizer@main   # optionally: with: { version: v0.1.0 }
- run: ocaml-oxidizer script.ml
```

## Release tooling

The release tooling is itself OCaml, run by ocaml-oxidizer:

- `ocaml-oxidizer scripts/bump_version.ml <major> <minor> <patch> <expected-current>`
  — bump the version and open a PR.
- `ocaml-oxidizer scripts/release.ml <commit>` — tag `v<version>` (immutable) and
  attach the binaries CI already built for that commit; no separate release build.
