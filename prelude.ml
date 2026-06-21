(* ox scripting prelude — a small "batteries" layer for ox scripts.

   Load it at the top of a script and the rest reads like a shell/Python script:

     let () =
       if not (load_remote
         "https://raw.githubusercontent.com/ajbt200128/ocaml-oxidizer/main/prelude.ml")
       then (prerr_endline "failed to load prelude"; exit 1)
     (* everything below now has run/capture/sprintf/get/... in scope *)

   It works because ox evaluates a script one top-level item at a time, so the
   definitions this file injects are in scope for the items that follow the load.
   Requires the `networking` feature (which load_remote itself needs). *)

(* --- formatting (Printf in scope) --- *)
let sprintf = Printf.sprintf
let printf = Printf.printf
let eprintf = Printf.eprintf

(* --- output & files --- *)
let print s = print_string s; print_newline ()      (* like Python print *)
let eprint s = prerr_string s; prerr_newline ()
let read_file path = In_channel.with_open_bin path In_channel.input_all
let write_file path s =
  Out_channel.with_open_bin path (fun oc -> Out_channel.output_string oc s)
let append_file path s =
  Out_channel.with_open_gen [ Open_append; Open_creat ] 0o644 path (fun oc ->
      Out_channel.output_string oc s)
let lines s = String.split_on_char '\n' s

(* --- abort & quoting --- *)
let die msg = prerr_endline msg; exit 1
let quote = Filename.quote

(* --- running commands --- *)
let run cmd = if Sys.command cmd <> 0 then die ("command failed: " ^ cmd)
let try_run cmd = Sys.command cmd = 0                 (* bool; never aborts *)
let capture cmd =                                     (* run, return trimmed stdout *)
  let ic = Unix.open_process_in cmd in
  let out = In_channel.input_all ic in
  (match Unix.close_process_in ic with
   | Unix.WEXITED 0 -> ()
   | _ -> die ("command failed: " ^ cmd));
  String.trim out

(* --- strings (Str in scope), Python-ish --- *)
let trim = String.trim
let lower = String.lowercase_ascii
let upper = String.uppercase_ascii
let starts_with ~prefix s = String.starts_with ~prefix s
let ends_with ~suffix s = String.ends_with ~suffix s
let join sep xs = String.concat sep xs
let regexp = Str.regexp                               (* compile a regex *)
let regexp_string = Str.regexp_string                 (* match a literal string *)
let matches re s = try ignore (Str.search_forward re s 0); true with Not_found -> false
let group n = Str.matched_group n                     (* nth group of the last match *)
let split sep s = if sep = "" then [ s ] else Str.split_delim (regexp_string sep) s
let contains s sub = matches (regexp_string sub) s
(* literal (non-template) replace, so [by] is never reinterpreted *)
let replace ~old ~by s = Str.global_substitute (regexp_string old) (fun _ -> by) s
let replace_first ~old ~by s = Str.substitute_first (regexp_string old) (fun _ -> by) s

(* --- filesystem, shell-like --- *)
let cd path = Sys.chdir path
let pwd () = Sys.getcwd ()
let ls ?(path = ".") () = Array.to_list (Sys.readdir path)
let exists path = Sys.file_exists path
let mkdir ?(parents = true) path =
  run (sprintf "mkdir %s %s" (if parents then "-p" else "") (quote path))
let touch path = run ("touch " ^ quote path)
let rm ?(recursive = false) ?(force = false) path =
  let flags = (if recursive then "r" else "") ^ if force then "f" else "" in
  run (sprintf "rm %s %s" (if flags = "" then "" else "-" ^ flags) (quote path))
let mv src dst = run (sprintf "mv %s %s" (quote src) (quote dst))
let cat path = print_string (read_file path)
let dirname = Filename.dirname
let basename = Filename.basename
let which cmd =
  match capture (sprintf "command -v %s 2>/dev/null || true" (quote cmd)) with
  | "" -> None
  | path -> Some path
let ox a = run ("ox " ^ a)                            (* run ox from inside ox *)

(* --- HTTP (in-process, via the networking primitives) --- *)
let header_lines headers = join "\n" (List.map (fun (k, v) -> k ^ ": " ^ v) headers)
let get ?(headers = []) ?out url =                    (* GET; optionally save to [out] *)
  let body = http_req "GET" url "" (header_lines headers) in
  (match out with Some p -> write_file p body | None -> ());
  body
let post ?(headers = []) ~file url = http_req "POST" url file (header_lines headers)
let put ?(headers = []) ~file url = http_req "PUT" url file (header_lines headers)

(* --- environment, args & input, Python-ish --- *)
let env ?default name =
  match Sys.getenv_opt name, default with
  | Some v, _ -> v
  | None, Some d -> d
  | None, None -> die ("missing env var: " ^ name)
let env_opt name = Sys.getenv_opt name
let args () = Array.to_list Sys.argv                  (* incl. argv0 (the script path) *)
let input ?(prompt = "") () =                         (* read a line from stdin *)
  if prompt <> "" then (print_string prompt; flush stdout);
  input_line stdin
let confirm ?(prompt = "continue? [y/N] ") () =
  match lower (trim (input ~prompt ())) with "y" | "yes" -> true | _ -> false

(* --- ox metadata (host functions) --- *)
let version () = ox_version ()
let features () = match ox_features () with "" -> [] | s -> String.split_on_char ',' s
