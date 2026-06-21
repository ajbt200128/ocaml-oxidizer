//! Embeddable OCaml 5.4 bytecode interpreter.
//!
//! The bytecode runtime, stdlib and unix are statically linked into the binary,
//! and the stdlib/unix `.cmi` files are bundled (see `build.rs`). [`Interp::new`]
//! starts the runtime and unpacks the cmis so no OCaml install is needed.

use ocaml::Runtime;

ocaml::import! {
    fn ox_init(cmis: &[u8]);
    fn eval_source(src: String) -> bool;
    fn eval_string(src: String) -> isize;
}

static CMIS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/cmis.bin"));

/// A live embedded OCaml interpreter. The runtime is a process-global singleton,
/// so repeated construction is idempotent.
pub struct Interp {
    rt: Runtime,
}

impl Interp {
    pub fn new() -> Self {
        let rt = ocaml::runtime::init();
        unsafe { ox_init(&rt, CMIS).expect("ox_init") };
        Interp { rt }
    }

    /// Run OCaml source as one implementation. Returns `false` if a phrase
    /// failed; the interpreter reports the error to stderr itself.
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
