(* Cut an immutable release for a commit, reusing the binaries CI already built.
   Run with: ocaml-oxidizer scripts/release.ml <commit>

   Tags v<version-at-commit>, fails if that tag exists, and attaches the CI
   artifacts for that exact commit. There is no separate release build. *)

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

let exists cmd = Sys.command (cmd ^ " >/dev/null 2>&1") = 0

let () =
  if Array.length Sys.argv <> 2 then die ("usage: " ^ Sys.argv.(0) ^ " <commit>");
  Sys.chdir (capture "git rev-parse --show-toplevel");

  let sha = capture ("git rev-parse " ^ Sys.argv.(1)) in
  let version = cargo_version (capture (Printf.sprintf "git show %s:Cargo.toml" sha)) in
  let tag = "v" ^ version in

  (* Immutable: refuse if the tag already exists locally or on the remote. *)
  if exists ("git rev-parse -q --verify refs/tags/" ^ tag)
     || exists ("git ls-remote --exit-code --tags origin " ^ tag)
  then die ("tag already exists: " ^ tag);

  (* Find the successful CI run for this commit and download its artifacts. *)
  let run_id =
    capture (Printf.sprintf
      "gh run list --commit %s --workflow ci.yml --json databaseId,conclusion \
       --jq '[.[] | select(.conclusion==\"success\")][0].databaseId'" sha)
  in
  if run_id = "" || run_id = "null" then die ("no successful ci.yml run for " ^ sha);

  let tmp = capture "mktemp -d" in
  run (Printf.sprintf "gh run download %s --dir %s" run_id tmp);
  let assets = capture (Printf.sprintf "find %s -type f -name 'ocaml-oxidizer-*'" tmp) in
  if assets = "" then die ("no release binaries among CI artifacts for " ^ sha);
  let files = String.split_on_char '\n' assets in
  List.iter (fun f -> run ("chmod +x " ^ f)) files;

  run (Printf.sprintf "git tag %s %s" tag sha);
  run ("git push origin " ^ tag);
  (* gh release create fails if the release already exists -> immutable. *)
  run (Printf.sprintf
         "gh release create %s --target %s --title %s \
          --notes 'Release %s (binaries reused from CI run %s).' %s"
         tag sha tag tag run_id (String.concat " " files));
  Printf.printf "released %s with %d binaries\n" tag (List.length files)
