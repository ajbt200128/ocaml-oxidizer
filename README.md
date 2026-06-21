# ocaml-oxidizer

A Rust crate + CLI that embeds a self-contained OCaml 5.4 bytecode interpreter.
The bytecode runtime, stdlib, and Unix are statically linked, and the stdlib/unix
`.cmi` files are bundled into the binary — no OCaml install is needed at runtime.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/ajbt200128/ocaml-oxidizer/main/install.sh | sh
```

Installs the latest release into `~/.local/bin`. Already installed? `ox update`
replaces the running binary in place with the latest release.

## Usage

```sh
ox script.ml          # run a script
ox --check script.ml  # typecheck only (pretty errors)
```

Scripts get the full stdlib, Unix, and Str: stdout/stderr, stdin, process exec,
effects and domains, and `Callback.register`. Two host functions report ox's own
metadata: `ox_version ()` and `ox_features ()`. Built with the `networking`
feature, scripts also get `http_get`, `http_status`, `http_req`, and
`load_remote` (fetch and evaluate a remote `.ml`).

A script is evaluated one top-level item at a time, so definitions pulled in by an
early `load_remote` are in scope for the items that follow — which is what makes
the scripting prelude below work.

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
- run: ox script.ml
```

## Scripting prelude

`prelude.ml` is a small "batteries" layer for ox scripts. Load it at the top of a
script and the rest reads like a shell/Python script:

```ocaml
let () =
  if not (load_remote
    "https://raw.githubusercontent.com/ajbt200128/ocaml-oxidizer/main/prelude.ml")
  then (prerr_endline "failed to load prelude"; exit 1)

let () = print (sprintf "ox %s on %s" (version ()) (capture "uname -s"))
let names = ls ~path:"src" ()
let body = get ~headers:[ ("Accept", "text/plain") ] "https://example.com"
```

It pulls `Printf`/`Str` helpers (`sprintf`, `split`, `replace`, `matches`, …) and
shell-like commands (`run`, `capture`, `cd`, `ls`, `rm`, `mv`, `mkdir`, `which`,
`get`/`post`/`put`, …) plus `input`, `env`, and `args` into scope. Requires the
`networking` feature (which `load_remote` itself needs).

## Release tooling

The release tooling is itself OCaml, run by ox — and dogfooded by two workflows
that install `ox` via this repo's own action (`uses: ./`) and run the scripts on
the prelude:

- `ox scripts/bump_version.ml <major|minor|patch> <current-version>` — bump the
  chosen component (the version arg is a safety check) and open a PR. The **bump**
  workflow (`workflow_dispatch`) runs this: pick a level, it opens the PR.
- `ox scripts/release.ml <commit>` — tag `v<version>` (immutable) and attach the
  binaries CI already built for that commit; no separate release build. The
  **release** workflow runs this automatically after CI succeeds on `main`,
  whenever the version changed.

Bootstrap: both workflows install `ox` from the latest release, so cut the first
release by hand (build `ox` locally, then `ox scripts/release.ml <commit>`); after
that the pipeline self-hosts.
