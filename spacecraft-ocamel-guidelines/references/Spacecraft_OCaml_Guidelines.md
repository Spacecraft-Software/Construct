# Spacecraft OCaml Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-12
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the `SKILL.md` for OCaml 5.x systems programming. It provides complete, compile-checked skeletons for effect-based concurrent I/O (`Eio`), worker-pool parallelism (`Domainslib`), GC-safe C FFI wrappers, and testing.

---

## 1. Concurrency vs. Parallelism: Architectural Decisions

OCaml 5 separates cooperative concurrency from true multicore parallelism. 

- **Concurrency (I/O-bound):** Handled via **Eio** fibers. Operating on a single Domain, fibers yield cooperatively at I/O boundaries (sockets, files, timers). Spawning fibers is extremely cheap (low microsecond overhead).
- **Parallelism (Compute-bound):** Handled via **Domains** (which map 1:1 to OS threads). Each Domain runs its own minor garbage collection independently, but shares the major heap. Spawning Domains has significant OS overhead and triggers minor GC synchronization pauses. Keep Domains bounded and run CPU-intensive tasks through a persistent `Domainslib` worker pool.

---

## 2. Eio Fiber Loop Skeleton (Direct-Style Concurrent I/O)

Eio uses effect handlers to implement direct-style concurrency without monadic wrappers. The code reads sequentially.

```ocaml
open Eio.Std

let handle_client flow =
  let buf = Cstruct.create 1024 in
  try
    let rec loop () =
      let got = Eio.Flow.read flow buf in
      if got > 0 then (
        Eio.Flow.write flow [Cstruct.sub buf 0 got];
        loop ()
      )
    in
    loop ()
  with
  | Eio.Io (Eio.Net.Connection_reset _, _) -> ()
  | exn -> Eio.traceln "Client handler failed with: %s" (Printexc.to_string exn)

let server_loop ~net port =
  let addr = `Tcp (Eio.Net.Ipaddr.V4.any, port) in
  Switch.run @@ fun sw ->
  let socket = Eio.Net.listen net ~sw ~backlog:128 ~reuse_addr:true addr in
  Eio.traceln "Server listening on port %d" port;
  while true do
    (* accept yields cooperatively to other fibers on the domain *)
    Eio.Net.accept_fork socket ~sw ~on_error:(fun exn ->
        Eio.traceln "Error accepting connection: %s" (Printexc.to_string exn)
    )
    handle_client
  done

let main () =
  (* Eio_main.run initializes the event loop for the current platform *)
  Eio_main.run @@ fun env ->
  server_loop ~net:(Eio.Stdenv.net env) 8080
```

---

## 3. Parallel Compute Pool Skeleton (Domainslib)

To execute CPU-intensive array transformations in parallel, register a worker pool once at startup and split the compute ranges using task pools.

```ocaml
module T = Domainslib.Task

let parallel_sum pool arr =
  let len = Array.length arr in
  if len < 1000 then
    (* Fall back to serial sum if task size is too small *)
    Array.fold_left ( + ) 0 arr
  else
    T.parallel_for_reduce pool ~start:0 ~finish:(len - 1) ~body:(fun i -> arr.(i)) ( + ) 0

let main () =
  (* Determine physical cores and size pool accordingly *)
  let cores = Domain.recommended_domain_count () - 1 in
  let pool = T.setup_pool ~num_additional_domains:cores () in
  
  let data = Array.init 1_000_000 (fun i -> i) in
  let total = T.run pool (fun () -> parallel_sum pool data) in
  
  Printf.printf "Parallel computation completed: Total = %d\n" total;
  T.teardown_pool pool
```

---

## 4. GC-Safe C FFI Binding

The OCaml GC moves heap objects during compacting phases. When OCaml passes a `value` pointer to C, the pointer must be registered as a GC root if the C function triggers a GC (e.g. by allocating OCaml objects or invoking OCaml callbacks).

### OCaml Module Interface (`telemetry.mli`)
```ocaml
type record = {
  cpu_usage : float;
  ram_usage : int;
}

val get_telemetry : int -> (record, string) result
```

### OCaml Implementation (`telemetry.ml`)
```ocaml
type record = {
  cpu_usage : float;
  ram_usage : int;
}

(* Declare the external C binding *)
external get_telemetry_raw : int -> record = "stub_get_telemetry"

let get_telemetry client_id =
  try Ok (get_telemetry_raw client_id)
  with Failure msg -> Error msg
```

### C Implementation (`telemetry_stub.c`)
```c
#include <caml/alloc.h>
#include <caml/memory.h>
#include <caml/mlvalues.h>
#include <caml/fail.h>

CAMLprim value stub_get_telemetry(value v_client_id) {
    // 1. Register input parameters
    CAMLparam1(v_client_id);
    
    // 2. Declare and register local OCaml values
    CAMLlocal3(v_record, v_cpu, v_ram);
    
    int client_id = Int_val(v_client_id);
    
    // Simulate reading telemetry data from hardware/system
    double cpu_usage = 0.42; 
    int ram_usage = 1024 * 1024 * 64; 
    
    if (client_id < 0) {
        // Safe exit path that frees roots and raises OCaml Failure exception
        caml_failwith("Invalid client ID");
    }
    
    // 3. Allocate boxed float (triggers GC)
    v_cpu = caml_copy_double(cpu_usage);
    
    // Allocate integer
    v_ram = Val_int(ram_usage);
    
    // 4. Allocate record structure (triggers GC)
    v_record = caml_alloc(2, 0);
    Store_field(v_record, 0, v_cpu);
    Store_field(v_record, 1, v_ram);
    
    // 5. Unregister roots and return value
    CAMLreturn(v_record);
}
```

---

## 5. Testing: Alcotest & QCheck

Test suites must check both happy paths and error boundaries. Use `QCheck` to verify data codec roundtrips and invariants.

```ocaml
(* test/test_suite.ml *)

let test_add () =
  Alcotest.(check int) "adding simple values" 4 (2 + 2)

(* QCheck property-based test *)
let test_reverse_roundtrip =
  QCheck.Test.make
    ~name:"list_reverse_twice_is_identity"
    QCheck.(list int)
    (fun l -> List.rev (List.rev l) = l)

let () =
  let open Alcotest in
  run "Telemetry Service Unit Tests" [
    "arithmetic", [
      test_case "add" `Quick test_add;
    ];
    "properties", [
      QCheck_alcotest.to_alcotest test_reverse_roundtrip;
    ];
  ]
```

---

## 6. Dune Build Configuration (`dune`)

Enforce warnings as errors and optimize compilation settings in your Dune build configs.

```dune
; src/dune
(library
 (name telemetry)
 (public_name telemetry)
 (foreign_stubs
  (language c)
  (names telemetry_stub))
 (flags
  :standard
  -warn-error +a-3)) ; Treat all warnings as errors (excluding warning 3)
```

---

## 7. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Raw `Domain.spawn` in loops** | Severe context-switching lag | Set up a persistent `Domainslib` task pool once on initialization. |
| **Missing FFI GC parameters** | Silent memory corruption | Always use `CAMLparam` / `CAMLlocal` in C stubs returning OCaml values. |
| **monadic `Lwt` / `Async` in Eio** | Compiler types fail to align | Migrate asynchronous I/O to native Eio fiber structures. |
| **Floats boxed inside structures** | High major GC allocations | Put float fields in records where *all* fields are floats, or use `float array`. |
| **Divisions by zero silently passing** | Division evaluations yield 0 or NaN | Validate divisors synchronously before evaluating fraction operations. |
| **Non-tailcall recursive loops** | Stack overflow on large arrays | Rewrite recursive operations to use accumulators and annotate with `[@tailcall]`. |

---

## 8. Code Review Compliance Gate

Before merging OCaml code, verify:
1. `dune build @fmt` output is clean.
2. Warnings are treated as errors (`-warn-error +a-3`).
3. C FFI implementations have been audited for memory parameter roots.
4. Concurrency strategies align: Eio for I/O, Domainslib for compute.
