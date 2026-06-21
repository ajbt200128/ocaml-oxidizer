//! Networking feature: load_remote fetches an .ml and evaluates it in the same
//! interpreter. Served from a local one-shot HTTP server so the test is offline.
#![cfg(feature = "networking")]

use ocaml_oxidizer::Interp;
use std::io::{Read, Write};
use std::net::TcpListener;

#[test]
fn load_remote_evaluates_fetched_ml() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let body = "let () = Callback.register \"remote_val\" (fun (_ : int) -> 4242)";
    let server = std::thread::spawn(move || {
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

    let i = Interp::new();
    let url = format!("http://127.0.0.1:{port}/remote.ml");
    let ok = i
        .run("<test>", &format!("let () = assert (load_remote \"{url}\")"))
        .unwrap();
    server.join().unwrap();

    assert!(ok, "load_remote returned false");
    // The remote script registered a callback; call it back from Rust.
    assert_eq!(i.call_int("remote_val", 0).unwrap(), 4242);
}
