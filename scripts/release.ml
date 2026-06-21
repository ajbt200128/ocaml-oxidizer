(* Cut an immutable release for a commit, reusing the binaries CI already built.
   Run with: ox scripts/release.ml <commit>

   Tags v<version-at-commit>, fails if that tag exists, and attaches the CI
   artifacts for that exact commit. There is no separate release build. *)

let prelude =
  Option.value (Sys.getenv_opt "OX_PRELUDE_URL")
    ~default:"https://raw.githubusercontent.com/ajbt200128/ocaml-oxidizer/main/prelude.ml"
let () = if not (load_remote prelude) then (prerr_endline "ox: failed to load prelude"; exit 1)

(* The first `version = "..."` line in Cargo.toml is the [package] version. *)
let cargo_version toml =
  match List.find_opt (starts_with ~prefix:"version = \"") (lines toml) with
  | Some line -> List.nth (split "\"" line) 1
  | None -> die "no version in Cargo.toml"

let () =
  let commit =
    match args () with [ _; c ] -> c | _ -> die "usage: release.ml <commit>"
  in
  cd (capture "git rev-parse --show-toplevel");

  let sha = capture ("git rev-parse " ^ commit) in
  let version = cargo_version (capture (sprintf "git show %s:Cargo.toml" sha)) in
  let tag = "v" ^ version in

  (* Immutable: refuse if the tag already exists locally or on the remote. *)
  if try_run ("git rev-parse -q --verify refs/tags/" ^ tag ^ " >/dev/null 2>&1")
     || try_run ("git ls-remote --exit-code --tags origin " ^ tag ^ " >/dev/null 2>&1")
  then die ("tag already exists: " ^ tag);

  (* Find the successful CI run for this commit and download its artifacts. *)
  let run_id =
    capture
      (sprintf
         "gh run list --commit %s --workflow ci.yml --json databaseId,conclusion \
          --jq '[.[] | select(.conclusion==\"success\")][0].databaseId'" sha)
  in
  if run_id = "" || run_id = "null" then die ("no successful ci.yml run for " ^ sha);

  let tmp = capture "mktemp -d" in
  run (sprintf "gh run download %s --dir %s" run_id tmp);
  let assets = capture (sprintf "find %s -type f -name 'ox-*'" tmp) in
  if assets = "" then die ("no release binaries among CI artifacts for " ^ sha);
  let files = lines assets in
  List.iter (fun f -> run ("chmod +x " ^ quote f)) files;

  run (sprintf "git tag %s %s" tag sha);
  run ("git push origin " ^ tag);
  (* gh release create fails if the release already exists -> immutable. *)
  run
    (sprintf "gh release create %s --target %s --title %s --notes %s %s" tag sha tag
       (quote (sprintf "Release %s (binaries reused from CI run %s)." tag run_id))
       (join " " files));
  printf "released %s with %d binaries\n" tag (List.length files)
