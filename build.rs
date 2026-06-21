use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let where_path =
        env::var("OCAML_WHERE_PATH").expect("OCAML_WHERE_PATH not set (run in `devenv shell`)");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let glue_obj = out_dir.join("glue.o");

    // -output-complete-obj bakes the bytecode toplevel + runtime into one object.
    // -linkall keeps glue's Callback.register initializer from being stripped.
    let ok = Command::new("ocamlfind")
        .args(["ocamlc", "-package", "compiler-libs.toplevel", "-linkpkg", "-linkall",
               "-output-complete-obj"])
        .arg("lib/glue.ml")
        .arg("-o")
        .arg(&glue_obj)
        .status()
        .expect("run ocamlfind ocamlc")
        .success();
    assert!(ok, "ocamlc failed to build glue object");

    // Link the object directly: nothing in Rust references it at link time
    // (the glue is reached at runtime via caml_startup + caml_named_value),
    // so an archive member would be dropped.
    println!("cargo:rustc-link-arg={}", glue_obj.display());
    println!("cargo:rustc-link-search=native={where_path}");
    println!("cargo:rustc-link-lib=pthread");

    println!("cargo:rerun-if-changed=lib/glue.ml");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=OCAML_WHERE_PATH");
}
