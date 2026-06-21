//! Networking primitives exposed to evaluated OCaml (feature = "networking").
//! reqwest with rustls/ring, statically linked. The glue's lib/net.ml binds
//! these into the toplevel env as `http_get`, `http_status`, `http_req`,
//! `load_remote`.

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;

fn fetch(url: &str) -> Result<String, reqwest::Error> {
    reqwest::blocking::get(url)?.text()
}

/// Parse newline-separated `Key: Value` lines into a header map, skipping blanks
/// and any line that isn't a valid header.
fn parse_headers(raw: &str) -> HeaderMap {
    let mut map = HeaderMap::new();
    for line in raw.lines() {
        let Some((name, value)) = line.split_once(':') else { continue };
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.trim().as_bytes()),
            HeaderValue::from_str(value.trim()),
        ) {
            map.insert(name, value);
        }
    }
    map
}

fn request(
    method: &str,
    url: &str,
    body_file: &str,
    headers: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut req = reqwest::blocking::Client::new()
        .request(Method::from_bytes(method.as_bytes())?, url)
        .headers(parse_headers(headers));
    if !body_file.is_empty() {
        req = req.body(std::fs::read(body_file)?);
    }
    Ok(req.send()?.text()?)
}

#[ocaml::func]
pub fn ox_http_get(url: String) -> String {
    match fetch(&url) {
        Ok(body) => body,
        Err(e) => format!("ERROR: {e}"),
    }
}

#[ocaml::func]
pub fn ox_http_status(url: String) -> isize {
    match reqwest::blocking::get(&url) {
        Ok(r) => r.status().as_u16() as isize,
        Err(_) => -1,
    }
}

/// Fetch an `.ml` from `url` and evaluate it in the same interpreter, so its
/// top-level definitions persist. Re-enters the registered `eval_source`.
#[ocaml::func]
pub fn ox_load_remote(url: String) -> bool {
    let src = match fetch(&url) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("load_remote: {e}");
            return false;
        }
    };
    let eval = match unsafe { ocaml::Value::named("eval_source") } {
        Some(f) => f,
        None => return false,
    };
    unsafe { eval.call(gc, [url, src]) }.unwrap_or(false)
}

/// Generic HTTP request: `method url body_file headers`. An empty `body_file`
/// sends no body; `headers` is newline-separated `Key: Value` lines. Returns the
/// response body, or `"ERROR: <msg>"` on failure (mirrors `ox_http_get`).
#[ocaml::func]
pub fn ox_http_req(method: String, url: String, body_file: String, headers: String) -> String {
    match request(&method, &url, &body_file, &headers) {
        Ok(body) => body,
        Err(e) => format!("ERROR: {e}"),
    }
}
