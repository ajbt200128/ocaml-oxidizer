(* Build-time codegen: marshal every .cmi in the given dirs into one blob.
   Usage: ocaml make_cmis.ml <out> <dir>... *)

let read_file path =
  let ic = open_in_bin path in
  let len = in_channel_length ic in
  let data = really_input_string ic len in
  close_in ic;
  data

let () =
  let out = Sys.argv.(1) in
  let dirs = Array.to_list (Array.sub Sys.argv 2 (Array.length Sys.argv - 2)) in
  let cmis =
    List.concat_map
      (fun dir ->
        Sys.readdir dir |> Array.to_list
        |> List.filter (fun f -> Filename.check_suffix f ".cmi")
        |> List.map (fun f -> (f, read_file (Filename.concat dir f))))
      dirs
  in
  let oc = open_out_bin out in
  output_string oc (Marshal.to_string cmis []);
  close_out oc
