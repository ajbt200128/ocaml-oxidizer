(* Networking demo. Run with: cargo run --features networking -- examples/net.ml
   http_get / http_status / load_remote are bound only with that feature. *)
let () = Printf.printf "GET https://example.com -> status %d\n" (http_status "https://example.com")
let () = Printf.printf "body length = %d\n" (String.length (http_get "https://example.com"))
