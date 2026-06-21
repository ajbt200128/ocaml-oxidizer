//! Embeddable OCaml 5.4 bytecode interpreter.
//!
//! The OCaml bytecode runtime and stdlib are statically linked into the binary;
//! [`Interp::new`] starts the runtime and registers the eval entry points.

use ocaml::Runtime;

ocaml::import! {
    fn eval_source(src: String) -> bool;
    fn eval_string(src: String) -> isize;
}

/// A live embedded OCaml interpreter. The runtime is a process-global singleton,
/// so repeated construction is idempotent.
pub struct Interp {
    rt: Runtime,
}

impl Interp {
    pub fn new() -> Self {
        Interp { rt: ocaml::runtime::init() }
    }

    /// Run OCaml source as a sequence of toplevel phrases. Returns `false` if a
    /// phrase failed; the interpreter reports the error to stderr itself.
    pub fn run(&self, src: &str) -> Result<bool, ocaml::Error> {
        unsafe { eval_source(&self.rt, src.to_string()) }
    }

    /// Evaluate an int-typed expression and return its value (test helper).
    pub fn eval_int(&self, src: &str) -> Result<isize, ocaml::Error> {
        unsafe { eval_string(&self.rt, src.to_string()) }
    }
}

impl Default for Interp {
    fn default() -> Self {
        Self::new()
    }
}
