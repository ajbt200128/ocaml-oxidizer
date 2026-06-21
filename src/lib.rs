//! Embeddable OCaml 5.4 bytecode interpreter.
//!
//! The bytecode runtime, stdlib and unix are statically linked into the binary,
//! and the stdlib/unix `.cmi` files are bundled (see `build.rs`). [`Interp::new`]
//! starts the runtime and unpacks the cmis so no OCaml install is needed.

use ocaml::Runtime;

ocaml::import! {
    fn ox_init(cmis: &[u8]);
    fn eval_source(filename: String, src: String) -> bool;
    fn check_source(filename: String, src: String) -> bool;
    fn eval_string(src: String) -> isize;
}

static CMIS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/cmis.bin"));

/// A host (Rust) function exposed to evaluated OCaml as `ox_double : int -> int`.
/// The glue binds it into the toplevel env at startup (see lib/glue.ml).
#[ocaml::func]
pub fn ox_double(x: isize) -> isize {
    x * 2
}

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

    /// Run OCaml source as one implementation. `filename` tags error locations.
    /// Returns `false` if a phrase failed; errors are reported to stderr.
    pub fn run(&self, filename: &str, src: &str) -> Result<bool, ocaml::Error> {
        unsafe { eval_source(&self.rt, filename.to_string(), src.to_string()) }
    }

    /// Typecheck OCaml source without running it. Returns `false` on a type error.
    pub fn check(&self, filename: &str, src: &str) -> Result<bool, ocaml::Error> {
        unsafe { check_source(&self.rt, filename.to_string(), src.to_string()) }
    }

    /// Evaluate an int-typed expression and return its value (test helper).
    pub fn eval_int(&self, src: &str) -> Result<isize, ocaml::Error> {
        unsafe { eval_string(&self.rt, src.to_string()) }
    }

    /// Call an OCaml `int -> int` function registered with `Callback.register`.
    pub fn call_int(&self, name: &str, arg: isize) -> Result<isize, ocaml::Error> {
        let f = unsafe { ocaml::Value::named(name) }
            .ok_or(ocaml::Error::Message("function not registered"))?;
        unsafe { f.call(&self.rt, [arg]) }
    }
}

impl Default for Interp {
    fn default() -> Self {
        Self::new()
    }
}
