(* Networking glue, linked only when the `networking` feature is on. The
   compile-time externals force the Rust primitives into the bytecode primitive
   table; [bind] exposes them to scripts under short names. *)

external ox_http_get : string -> string = "ox_http_get"
external ox_http_status : string -> int = "ox_http_status"
external ox_load_remote : string -> bool = "ox_load_remote"

let () = ignore (ox_http_get, ox_http_status, ox_load_remote)

let bind () =
  let phrase =
    "external http_get : string -> string = \"ox_http_get\"\n\
     external http_status : string -> int = \"ox_http_status\"\n\
     external load_remote : string -> bool = \"ox_load_remote\""
  in
  try
    ignore
      (Toploop.execute_phrase false Format.err_formatter
         (Parsetree.Ptop_def (Parse.implementation (Lexing.from_string phrase))));
    true
  with _ -> false

let () = Callback.register "ox_bind_net" bind
