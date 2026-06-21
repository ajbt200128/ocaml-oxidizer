(* Embedded OCaml 5.4 bytecode interpreter glue. *)

let () = Toploop.set_paths ()
let () = Toploop.initialize_toplevel_env ()

let report exn =
  flush stdout;
  (try Location.report_exception Format.err_formatter exn
   with _ -> Printf.eprintf "Uncaught exception: %s\n" (Printexc.to_string exn));
  Format.pp_print_flush Format.err_formatter ()

(* The host owns process exit, so OCaml's at_exit never flushes its channels. *)
let flush_io () =
  flush stdout;
  flush stderr

(* Run a whole .ml source (an implementation structure) as one phrase. *)
let eval_source (src : string) : bool =
  let lb = Lexing.from_string src in
  let ok =
    try
      Toploop.execute_phrase false Format.std_formatter
        (Parsetree.Ptop_def (Parse.implementation lb))
    with exn ->
      report exn;
      false
  in
  flush_io ();
  ok

(* Evaluate an int-typed expression (test helper). *)
let eval_int (src : string) : int =
  let lb = Lexing.from_string ("let __ox = (" ^ src ^ ");;") in
  let phrase = !Toploop.parse_toplevel_phrase lb in
  ignore (Toploop.execute_phrase false Format.std_formatter phrase);
  flush_io ();
  (Obj.obj (Toploop.getvalue "__ox") : int)

let () = Callback.register "eval_source" eval_source
let () = Callback.register "eval_string" eval_int
