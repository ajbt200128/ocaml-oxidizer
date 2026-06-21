(* OCaml 5 domains (parallelism). Spawned from OCaml, joined back. *)
let () =
  let d = Domain.spawn (fun () -> List.fold_left ( + ) 0 [ 1; 2; 3; 4; 5 ]) in
  Printf.printf "domain sum = %d\n" (Domain.join d)
