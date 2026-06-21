(* ox exposes metadata about itself as host (Rust) functions bound at startup. *)
let () = Printf.printf "ox %s (features: %s)\n" (ox_version ()) (ox_features ())
