//! End-to-end stress tests driving the built CLI as a subprocess, one fresh
//! process per case (the OCaml runtime is a process-global singleton).

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_ox");

struct Output {
    stdout: String,
    stderr: String,
    code: i32,
}

fn run(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(BIN)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ox");
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    Output {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

fn script(name: &str, src: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ox_test_{name}.ml"));
    std::fs::write(&p, src).unwrap();
    p
}

fn run_file(name: &str, src: &str, stdin: &str) -> Output {
    let p = script(name, src);
    run(&[p.to_str().unwrap()], stdin)
}

#[test]
fn stdout_works() {
    let o = run_file("stdout", "let () = print_string \"hello-stdout\"", "");
    assert_eq!(o.code, 0);
    assert!(o.stdout.contains("hello-stdout"), "{:?}", o.stdout);
}

#[test]
fn stderr_works() {
    let o = run_file("stderr", "let () = prerr_string \"hello-stderr\"", "");
    assert_eq!(o.code, 0);
    assert!(o.stderr.contains("hello-stderr"), "{:?}", o.stderr);
}

#[test]
fn stdin_works() {
    let o = run_file("stdin", "let () = Printf.printf \"got:%s\" (read_line ())", "piped-line");
    assert_eq!(o.code, 0);
    assert!(o.stdout.contains("got:piped-line"), "{:?}", o.stdout);
}

#[test]
fn process_exec_works() {
    let o = run_file("proc", "let () = ignore (Sys.command \"echo from-subprocess\")", "");
    assert_eq!(o.code, 0);
    assert!(o.stdout.contains("from-subprocess"), "{:?}", o.stdout);
}

#[test]
fn unix_is_available() {
    let o = run_file("unix", "let () = Printf.printf \"ppid-ok:%b\" (Unix.getppid () > 0)", "");
    assert_eq!(o.code, 0);
    assert!(o.stdout.contains("ppid-ok:true"), "{:?}", o.stdout);
}

#[test]
fn effects_work() {
    let o = run(&["examples/effects.ml"], "");
    assert_eq!(o.code, 0);
    assert!(o.stdout.contains("effect result = 42"), "{:?}", o.stdout);
}

#[test]
fn domains_work() {
    let o = run(&["examples/domains.ml"], "");
    assert_eq!(o.code, 0);
    assert!(o.stdout.contains("domain sum = 15"), "{:?}", o.stdout);
}

#[test]
fn host_metadata_callable() {
    let o = run_file(
        "hostfn",
        "let () = Printf.printf \"v=%s feats=%s\" (ox_version ()) (ox_features ())",
        "",
    );
    assert_eq!(o.code, 0);
    assert!(o.stdout.contains(&format!("v={}", env!("CARGO_PKG_VERSION"))), "{:?}", o.stdout);
    if cfg!(feature = "networking") {
        assert!(o.stdout.contains("networking"), "{:?}", o.stdout);
    }
}

#[test]
fn check_accepts_valid() {
    let o = run(&["--check", "examples/hello.ml"], "");
    assert_eq!(o.code, 0);
    assert!(o.stdout.is_empty(), "check should not run: {:?}", o.stdout);
}

#[test]
fn check_rejects_type_error_prettily() {
    let p = script("checkbad", "let () = print_string 1");
    let o = run(&["--check", p.to_str().unwrap()], "");
    assert_ne!(o.code, 0);
    assert!(o.stderr.contains("Error"), "{:?}", o.stderr);
    assert!(o.stderr.contains("print_string 1"), "snippet missing: {:?}", o.stderr);
}

#[test]
fn check_does_not_execute() {
    let p = script(
        "checknorun",
        "let () = print_string 1\nlet () = print_string \"SHOULD-NOT-PRINT\"",
    );
    let o = run(&["--check", p.to_str().unwrap()], "");
    assert_ne!(o.code, 0);
    assert!(!o.stdout.contains("SHOULD-NOT-PRINT"), "{:?}", o.stdout);
}

#[test]
fn script_args_become_argv() {
    let p = script("argv", "let () = print_string (String.concat \",\" (Array.to_list Sys.argv))");
    let o = run(&[p.to_str().unwrap(), "a", "b"], "");
    assert_eq!(o.code, 0);
    assert!(o.stdout.ends_with(",a,b"), "{:?}", o.stdout);
}

#[test]
fn runtime_exception_exits_nonzero() {
    let o = run_file("exn", "let () = failwith \"boom\"", "");
    assert_ne!(o.code, 0);
    assert!(o.stderr.contains("boom") || o.stderr.contains("Failure"), "{:?}", o.stderr);
}
