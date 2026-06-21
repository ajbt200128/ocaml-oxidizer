(* Bump the crate version and open a PR. Run with:
   ox scripts/bump_version.ml <major> <minor> <patch> <expected-current>

   The expected-current argument is a safety check: the bump aborts unless the
   crate is currently at that version. *)

let die msg =
  prerr_endline msg;
  exit 1

let run cmd = if Sys.command cmd <> 0 then die ("command failed: " ^ cmd)

let capture cmd =
  let ic = Unix.open_process_in cmd in
  let out = In_channel.input_all ic in
  (match Unix.close_process_in ic with
   | Unix.WEXITED 0 -> ()
   | _ -> die ("command failed: " ^ cmd));
  String.trim out

let read_file path = In_channel.with_open_bin path In_channel.input_all
let write_file path s = Out_channel.with_open_bin path (fun oc -> Out_channel.output_string oc s)

(* Replace the first occurrence of [sub] in [s] with [by]. *)
let replace_first ~sub ~by s =
  let n = String.length sub and len = String.length s in
  let rec find i =
    if i + n > len then die ("could not find: " ^ sub)
    else if String.sub s i n = sub then i
    else find (i + 1)
  in
  let i = find 0 in
  String.sub s 0 i ^ by ^ String.sub s (i + n) (len - i - n)

(* The first `version = "..."` line is the [package] version. *)
let cargo_version toml =
  let prefix = "version = \"" in
  let pn = String.length prefix in
  let rec find = function
    | [] -> die "no version in Cargo.toml"
    | l :: rest ->
        if String.length l > pn && String.sub l 0 pn = prefix then
          let r = String.sub l pn (String.length l - pn) in
          String.sub r 0 (String.index r '"')
        else find rest
  in
  find (String.split_on_char '\n' toml)

let () =
  if Array.length Sys.argv <> 5 then
    die ("usage: " ^ Sys.argv.(0) ^ " <major> <minor> <patch> <expected-current>");
  Sys.chdir (capture "git rev-parse --show-toplevel");

  let nv = Printf.sprintf "%s.%s.%s" Sys.argv.(1) Sys.argv.(2) Sys.argv.(3) in
  let expected = Sys.argv.(4) in
  let toml = read_file "Cargo.toml" in
  let current = cargo_version toml in
  if current <> expected then
    die (Printf.sprintf "current version is %s, expected %s" current expected);
  if nv = current then die ("new version equals current: " ^ nv);

  let branch = "bump-v" ^ nv in
  run ("git switch -c " ^ branch);
  write_file "Cargo.toml"
    (replace_first
       ~sub:(Printf.sprintf "version = \"%s\"" current)
       ~by:(Printf.sprintf "version = \"%s\"" nv)
       toml);
  write_file "Cargo.lock"
    (replace_first
       ~sub:(Printf.sprintf "name = \"ocaml-oxidizer\"\nversion = \"%s\"" current)
       ~by:(Printf.sprintf "name = \"ocaml-oxidizer\"\nversion = \"%s\"" nv)
       (read_file "Cargo.lock"));

  run "git add Cargo.toml Cargo.lock";
  run (Printf.sprintf "git commit -m 'chore: bump version to %s'" nv);
  run ("git push -u origin " ^ branch);
  let base = capture "gh repo view --json defaultBranchRef --jq .defaultBranchRef.name" in
  run (Printf.sprintf
         "gh pr create --base %s --head %s --title 'Bump version to %s' \
          --body 'Bumps version from %s to %s.'"
         base branch nv current nv)
