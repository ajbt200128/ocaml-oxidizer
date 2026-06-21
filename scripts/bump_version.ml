(* Bump the crate version and open a PR. Run with:
     ox scripts/bump_version.ml <major|minor|patch> <current-version>

   The word picks which component to bump (zeroing the lower ones, per semver);
   <current-version> is a safety check — the bump aborts unless Cargo.toml is
   actually at that version. E.g. `bump_version.ml minor 0.1.2` -> 0.2.0. *)

let prelude =
  Option.value (Sys.getenv_opt "OX_PRELUDE_URL")
    ~default:"https://raw.githubusercontent.com/ajbt200128/ocaml-oxidizer/main/prelude.ml"
let () = if not (load_remote prelude) then (prerr_endline "ox: failed to load prelude"; exit 1)

(* The first `version = "..."` line in Cargo.toml is the [package] version. *)
let cargo_version toml =
  match List.find_opt (starts_with ~prefix:"version = \"") (lines toml) with
  | Some line -> List.nth (split "\"" line) 1
  | None -> die "no version in Cargo.toml"

let parse_version v =
  match split "." v |> List.map int_of_string with
  | [ major; minor; patch ] -> (major, minor, patch)
  | _ -> die (sprintf "not a semver version: %s" v)
  | exception _ -> die (sprintf "not a semver version: %s" v)

let bump level (major, minor, patch) =
  match level with
  | "major" -> (major + 1, 0, 0)
  | "minor" -> (major, minor + 1, 0)
  | "patch" -> (major, minor, patch + 1)
  | _ -> die (sprintf "level must be major, minor, or patch (got %S)" level)

let () =
  let level, expected =
    match args () with
    | [ _; level; current ] -> (level, current)
    | _ -> die "usage: bump_version.ml <major|minor|patch> <current-version>"
  in
  cd (capture "git rev-parse --show-toplevel");

  let toml = read_file "Cargo.toml" in
  let current = cargo_version toml in
  if current <> expected then
    die (sprintf "Cargo.toml is at %s, but you said %s" current expected);
  let a, b, c = bump level (parse_version current) in
  let nv = sprintf "%d.%d.%d" a b c in

  let branch = "bump-v" ^ nv in
  run ("git switch -c " ^ branch);
  write_file "Cargo.toml"
    (replace_first
       ~old:(sprintf "version = \"%s\"" current)
       ~by:(sprintf "version = \"%s\"" nv)
       toml);
  write_file "Cargo.lock"
    (replace_first
       ~old:(sprintf "name = \"ocaml-oxidizer\"\nversion = \"%s\"" current)
       ~by:(sprintf "name = \"ocaml-oxidizer\"\nversion = \"%s\"" nv)
       (read_file "Cargo.lock"));

  run "git add Cargo.toml Cargo.lock";
  run (sprintf "git commit -m %s" (quote (sprintf "chore: bump version to %s" nv)));
  run ("git push -u origin " ^ branch);
  let base = capture "gh repo view --json defaultBranchRef --jq .defaultBranchRef.name" in
  run
    (sprintf "gh pr create --base %s --head %s --title %s --body %s" base branch
       (quote (sprintf "Bump version to %s" nv))
       (quote (sprintf "Bumps version from %s to %s." current nv)))
