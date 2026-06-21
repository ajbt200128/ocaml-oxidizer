use std::env;
use std::path::PathBuf;
use std::process::Command;

fn run(cmd: &mut Command, what: &str) {
    let ok = cmd
        .status()
        .unwrap_or_else(|e| panic!("spawn {what}: {e}"))
        .success();
    assert!(ok, "{what} failed");
}

fn ocamlfind_query(pkg: &str) -> String {
    let out = Command::new("ocamlfind")
        .args(["query", pkg])
        .output()
        .expect("run ocamlfind query");
    assert!(out.status.success(), "ocamlfind query {pkg} failed");
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn main() {
    let where_path =
        env::var("OCAML_WHERE_PATH").expect("OCAML_WHERE_PATH not set (run in `devenv shell`)");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Bundle stdlib + unix + str .cmi into cmis.bin for self-contained typechecking.
    let cmis_bin = out_dir.join("cmis.bin");
    let unix_dir = ocamlfind_query("unix");
    let str_dir = ocamlfind_query("str");
    run(
        Command::new("ocaml")
            .arg("lib/make_cmis.ml")
            .arg(&cmis_bin)
            .arg(&where_path)
            .arg(&unix_dir)
            .arg(&str_dir),
        "make_cmis",
    );

    // -output-complete-obj bakes the bytecode toplevel + stdlib + unix + runtime
    // into one object; -linkall keeps glue's Callback.register initializers.
    let glue_obj = out_dir.join("glue.o");
    let mut cmd = Command::new("ocamlfind");
    cmd.args([
        "ocamlc",
        "-package",
        "compiler-libs.toplevel,unix,str",
        "-linkpkg",
        "-linkall",
        "-output-complete-obj",
    ]);
    // net.ml's externals resolve to the Rust networking primitives at final link.
    if env::var("CARGO_FEATURE_NETWORKING").is_ok() {
        cmd.arg("lib/net.ml");
    }
    cmd.arg("lib/glue.ml").arg("-o").arg(&glue_obj);
    run(&mut cmd, "ocamlc glue");

    // whole-archive forces the object in (nothing references it at link time) and
    // places it before libc, so its libc/libm refs resolve (GNU ld is order-sensitive).
    let lib = out_dir.join("libglue.a");
    let _ = std::fs::remove_file(&lib);
    run(Command::new("ar").arg("rcs").arg(&lib).arg(&glue_obj), "ar");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-search=native={where_path}");
    println!("cargo:rustc-link-lib=static:+whole-archive=glue");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=pthread");

    println!("cargo:rerun-if-changed=lib/glue.ml");
    println!("cargo:rerun-if-changed=lib/net.ml");
    println!("cargo:rerun-if-changed=lib/make_cmis.ml");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=OCAML_WHERE_PATH");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_NETWORKING");
}
