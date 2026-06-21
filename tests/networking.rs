//! Networking feature: load_remote fetches an .ml and evaluates it in the same
//! interpreter. Served from a local one-shot HTTP server so the test is offline.
#![cfg(feature = "networking")]

use ocaml_oxidizer::Interp;
use std::io::{Read, Write};
use std::net::TcpListener;

/// Serve `body` once over HTTP on an ephemeral port. Returns the join handle (to
/// be joined after the fetch) and the URL to load.
fn serve_once(body: &'static str) -> (std::thread::JoinHandle<()>, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let mut buf = [0u8; 2048];
        let _ = s.read(&mut buf);
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        s.write_all(resp.as_bytes()).unwrap();
    });
    (handle, format!("http://127.0.0.1:{port}/remote.ml"))
}

// One test: the OCaml runtime is a process-global singleton, so in-process calls
// must stay on a single thread (cargo runs separate #[test]s in parallel).
#[test]
fn load_remote_behaviors() {
    let i = Interp::new();

    // (1) A fetched .ml is evaluated in the same interpreter: it registers a
    // callback that we then invoke from Rust.
    let (srv, url) = serve_once("let () = Callback.register \"remote_val\" (fun (_ : int) -> 4242)");
    let ok = i
        .run("<test>", &format!("let () = assert (load_remote \"{url}\")"))
        .unwrap();
    srv.join().unwrap();
    assert!(ok, "load_remote returned false");
    assert_eq!(i.call_int("remote_val", 0).unwrap(), 4242);

    // (2) Definitions from a prelude loaded by an early top-level item are usable
    // by *later* items in the same script — only possible because items evaluate
    // one at a time, so the remote runs before the next item is type-checked.
    let (srv, url) = serve_once("let shout s = String.uppercase_ascii s");
    let src = format!(
        "let () = assert (load_remote \"{url}\")\n\
         let () = Callback.register \"shouted\" (fun (_ : int) -> String.length (shout \"hi\"))"
    );
    let ok = i.run("<test>", &src).unwrap();
    srv.join().unwrap();
    assert!(ok, "load_remote-then-use failed to evaluate");
    assert_eq!(i.call_int("shouted", 0).unwrap(), 2);
}
