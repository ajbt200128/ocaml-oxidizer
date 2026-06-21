//! Networking primitives exposed to evaluated OCaml (feature = "networking").
//! reqwest with rustls/ring, statically linked. The glue's lib/net.ml binds
//! these into the toplevel env as `http_get`, `http_status`, `load_remote`.

fn fetch(url: &str) -> Result<String, reqwest::Error> {
    reqwest::blocking::get(url)?.text()
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
