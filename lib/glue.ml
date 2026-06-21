(* Spike 0: prove the embedded bytecode Toploop runs inside a Rust host. *)

let () = Toploop.set_paths ()
let () = Toploop.initialize_toplevel_env ()

(* Evaluate [src] as an int-typed expression and return its value. *)
let eval_int (src : string) : int =
  let lb = Lexing.from_string ("let __ox = (" ^ src ^ ");;") in
  let phrase = !Toploop.parse_toplevel_phrase lb in
  ignore (Toploop.execute_phrase false Format.std_formatter phrase);
  (* The host owns process exit, so OCaml's at_exit never flushes its channels. *)
  flush stdout;
  flush stderr;
  (Obj.obj (Toploop.getvalue "__ox") : int)

let () = Callback.register "eval_string" eval_int
