(* OCaml 5 effect handlers, within a single eval region. *)
open Effect
open Effect.Deep

type _ Effect.t += Ask : int Effect.t

let handler =
  { effc =
      (fun (type a) (eff : a Effect.t) ->
        match eff with
        | Ask -> Some (fun (k : (a, _) continuation) -> continue k 41)
        | _ -> None) }

let () =
  let r = try_with (fun () -> 1 + perform Ask) () handler in
  Printf.printf "effect result = %d\n" r
