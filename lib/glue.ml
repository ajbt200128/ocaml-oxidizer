(* Embedded OCaml 5.4 bytecode interpreter glue. *)

let report exn =
  flush stdout;
  (try Location.report_exception Format.err_formatter exn
   with _ -> Printf.eprintf "Uncaught exception: %s\n" (Printexc.to_string exn));
  Format.pp_print_flush Format.err_formatter ()

(* The host owns process exit, so OCaml's at_exit never flushes its channels. *)
let flush_io () =
  flush stdout;
  flush stderr

(* Tag the lexbuf with the real path and register it so errors quote the
   offending source line with a caret. *)
let lexbuf_of fname src =
  let lb = Lexing.from_string src in
  Lexing.set_filename lb fname;
  Location.input_name := fname;
  Location.input_lexbuf := Some lb;
  lb

let initialized = ref false

let ox_init (blob : string) : unit =
  if not !initialized then begin
    let cmis : (string * string) list = Marshal.from_string blob 0 in
    let dir = Filename.temp_dir "ocaml-oxidizer" "cmis" in
    List.iter
      (fun (name, data) ->
        let oc = open_out_bin (Filename.concat dir name) in
        output_string oc data;
        close_out oc)
      cmis;
    (* Pretty errors: quote the source line with a caret. *)
    Clflags.error_style := Some Misc.Error_style.Contextual;
    (* Sole cmi source: no on-disk OCaml install is consulted. *)
    Clflags.include_dirs := [ dir ];
    Clflags.no_std_include := true;
    Toploop.set_paths ();
    Toploop.initialize_toplevel_env ();
    initialized := true
  end

(* Run a whole .ml source (an implementation structure) as one phrase. *)
let eval_source (fname : string) (src : string) : bool =
  let lb = lexbuf_of fname src in
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

(* Typecheck only, without executing. *)
let check_source (fname : string) (src : string) : bool =
  let lb = lexbuf_of fname src in
  let ok =
    try
      ignore (Typemod.type_structure !Toploop.toplevel_env (Parse.implementation lb));
      true
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

let () = Callback.register "ox_init" ox_init
let () = Callback.register "eval_source" eval_source
let () = Callback.register "check_source" check_source
let () = Callback.register "eval_string" eval_int
