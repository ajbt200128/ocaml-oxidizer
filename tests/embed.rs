//! In-process tests of the embeddable `Interp` API. A single test: the OCaml
//! runtime is a process-global singleton, so all in-process calls stay on one
//! thread.

use ocaml_oxidizer::Interp;

#[test]
fn embed_api() {
    let i = Interp::new();

    // Evaluate an expression and read the int back into Rust.
    assert_eq!(i.eval_int("6 * 7").unwrap(), 42);

    // OCaml registers a closure via Callback.register; Rust fetches and calls it.
    assert!(i
        .run("<test>", "let () = Callback.register \"inc\" (fun (x : int) -> x + 1)")
        .unwrap());
    assert_eq!(i.call_int("inc", 41).unwrap(), 42);

    // check accepts well-typed and rejects ill-typed source.
    assert!(i.check("<test>", "let x : int = 1").unwrap());
    assert!(!i.check("<test>", "let x : int = \"no\"").unwrap());
}
