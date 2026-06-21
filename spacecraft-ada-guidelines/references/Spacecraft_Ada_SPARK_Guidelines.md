# Spacecraft Ada / SPARK Guidelines — Extended Reference

This document supplements `SKILL.md` with concrete, contracted code, `gnatprove`
invocation patterns, and a proof-debugging guide. Load it when the user asks for
"full guidelines", "examples", "a worked proof", or when the concise rules are not
enough.

## Contents

1. Strong typing as proof leverage
2. A fully contracted, Silver-level subprogram (saturating add)
3. A bounded stack proven at Gold (Pre/Post + Type_Invariant + Loop_Invariant)
4. Ownership / access types under SPARK
5. Ghost code for specification
6. Ravenscar protected object (data-race-free shared state)
7. `gnatprove` invocation patterns
8. Common proof failures and how to fix them
9. Assurance-level adoption path

---

## 1. Strong Typing as Proof Leverage

The cheapest proof is the one the type system discharges for you. Prefer this:

```ada
type Tank_Index is range 1 .. 64;
type Pressure_kPa is range 0 .. 10_000;          -- units in the name; ISO/metric (Standard §14)
type Tank_Array is array (Tank_Index) of Pressure_kPa;
```

over `Integer` everywhere. `Tank_Array (I)` cannot be out of range because `I`
*is* a `Tank_Index`. No guard, no proof obligation, no runtime check.

Distinct types stop unit-mixing bugs at compile time:

```ada
type Meters  is new Float;
type Seconds is new Float;
-- D : Meters := M + S;   -- illegal: cannot mix Meters and Seconds
```

---

## 2. Silver-Level Subprogram (Absence of Run-Time Errors)

A saturating add that `gnatprove` proves cannot overflow or trap.

`saturate.ads`:
```ada
-- SPDX-FileCopyrightText: Copyright (C) 2026 Mohamed Hammad & Spacecraft Software
-- SPDX-License-Identifier: GPL-3.0-or-later
package Saturate
  with SPARK_Mode => On
is
   type U16 is mod 2 ** 16;

   function Sat_Add (A, B : U16) return U16
     with
       Global => null,
       Post   =>
         (if U16'Last - A >= B then Sat_Add'Result = A + B
          else Sat_Add'Result = U16'Last);
end Saturate;
```

`saturate.adb`:
```ada
-- SPDX-FileCopyrightText: Copyright (C) 2026 Mohamed Hammad & Spacecraft Software
-- SPDX-License-Identifier: GPL-3.0-or-later
package body Saturate
  with SPARK_Mode => On
is
   function Sat_Add (A, B : U16) return U16 is
   begin
      if U16'Last - A >= B then   -- guard proves A + B will not wrap
         return A + B;
      else
         return U16'Last;
      end if;
   end Sat_Add;
end Saturate;
```

`gnatprove --mode=prove` discharges the overflow check (the guard makes `A + B`
provably safe) and the postcondition. This is Silver; the `Post` also makes it Gold
for the saturation property.

---

## 3. Bounded Stack at Gold

A fixed-capacity stack with a `Type_Invariant`, full functional contracts, and a
proven loop. No heap, no exceptions.

```ada
-- SPDX-FileCopyrightText: Copyright (C) 2026 Mohamed Hammad & Spacecraft Software
-- SPDX-License-Identifier: GPL-3.0-or-later
generic
   type Item is private;
   Capacity : Positive;
package Bounded_Stack
  with SPARK_Mode => On
is
   type Stack is private;

   function Length (S : Stack) return Natural
     with Global => null;

   function Is_Full  (S : Stack) return Boolean is (Length (S) = Capacity)
     with Global => null;
   function Is_Empty (S : Stack) return Boolean is (Length (S) = 0)
     with Global => null;

   procedure Push (S : in out Stack; X : Item)
     with
       Global => null,
       Pre    => not Is_Full (S),
       Post   => Length (S) = Length (S'Old) + 1;

   procedure Pop (S : in out Stack; X : out Item)
     with
       Global => null,
       Pre    => not Is_Empty (S),
       Post   => Length (S) = Length (S'Old) - 1;

private
   type Index is range 0 .. Capacity;
   type Store is array (1 .. Index'Last) of Item;

   type Stack is record
      Top   : Index := 0;
      Items : Store;
   end record
     with Type_Invariant => Top <= Capacity;   -- always true, proven on exit

   function Length (S : Stack) return Natural is (Natural (S.Top));
end Bounded_Stack;
```

Key points: `Pre` makes overflow/underflow unrepresentable, so `gnatprove` proves
the index arithmetic is in range (Silver) *and* the `Post` length contracts (Gold).
The `Type_Invariant` is re-established and proven at every boundary crossing.

A loop inside such a unit needs an invariant:

```ada
procedure Clear (S : in out Stack)
  with Global => null, Post => Is_Empty (S)
is
begin
   while S.Top > 0 loop
      pragma Loop_Invariant (S.Top <= S.Top'Loop_Entry);
      pragma Loop_Variant (Decreases => S.Top);   -- proves termination
      S.Top := S.Top - 1;
   end loop;
end Clear;
```

---

## 4. Ownership / Access Types Under SPARK

Prefer values. When a pointer is unavoidable, SPARK enforces single-writer
ownership (move/borrow/observe) and rejects aliasing of mutable data — the same
discipline as Rust's borrow checker, checked by `gnatprove`.

```ada
type Int_Ptr is access Integer;

procedure Consume (P : in out Int_Ptr)
  with SPARK_Mode => On, Global => null
is
   Q : Int_Ptr := P;   -- MOVE: ownership transfers to Q; P is now invalid
begin
   Q.all := 0;
   --  Using P here would be rejected: it no longer owns the object.
   P := Q;             -- move back so the caller's `in out` stays valid
end Consume;
```

If the tool complains about aliasing, the fix is almost always to restructure
toward owned values or to borrow (`access` parameter, observed), not to suppress
the check.

---

## 5. Ghost Code for Specification

Ghost entities exist only for proof and are compiled out of production builds.

```ada
function Sorted (A : Arr) return Boolean
  with Ghost,
       Global => null;

procedure Sort (A : in out Arr)
  with Global => null,
       Post   => Sorted (A);          -- spec written in terms of the ghost fn
```

For unbounded reasoning (sums, counts) use big numbers in ghost context:

```ada
with Ada.Numerics.Big_Numbers.Big_Integers;
use  Ada.Numerics.Big_Numbers.Big_Integers;

function Sum (A : Arr) return Big_Integer with Ghost;
```

---

## 6. Ravenscar Protected Object (Data-Race-Free Shared State)

```ada
-- SPDX-FileCopyrightText: Copyright (C) 2026 Mohamed Hammad & Spacecraft Software
-- SPDX-License-Identifier: GPL-3.0-or-later
pragma Profile (Ravenscar);

package Telemetry
  with SPARK_Mode => On
is
   protected Counter
     with Priority => 10            -- ceiling locking: no priority inversion
   is
      procedure Bump;               -- exclusive (read/write)
      function  Value return Natural; -- concurrent reads allowed
   private
      Count : Natural := 0;
   end Counter;
end Telemetry;
```

```ada
package body Telemetry
  with SPARK_Mode => On
is
   protected body Counter is
      procedure Bump is
      begin
         if Count < Natural'Last then   -- guard proves no overflow
            Count := Count + 1;
         end if;
      end Bump;

      function Value return Natural is (Count);
   end Counter;
end Telemetry;
```

`gnatprove` proves data-race freedom: all access to `Count` is mediated by the
protected object, and the ceiling protocol bounds blocking. Tasks call
`Telemetry.Counter.Bump`; periodic tasks use `delay until Next_Release;`.

---

## 7. `gnatprove` Invocation Patterns

```sh
# Stone: is it valid SPARK at all?
gnatprove -P project.gpr --mode=check_all

# Bronze: flow analysis (initialization, globals, no bad aliasing)
gnatprove -P project.gpr --mode=flow

# Silver/Gold: full proof, moderate effort (good CI default)
gnatprove -P project.gpr --mode=all --level=2 --report=fail

# Stubborn checks: more provers, more budget
gnatprove -P project.gpr --mode=all --level=4 \
  --prover=z3,cvc5,altergo --timeout=60 --steps=0

# Investigate a single unit / line
gnatprove -P project.gpr --mode=all -u my_unit.adb
```

`--level` runs 0 (fastest, shallow) to 4 (slowest, deepest). `--report=fail` keeps
output to unproved checks. Treat any unproved check in a Silver+ unit as a build
failure. (Substitute the modern CLI per `spacecraft-cli-preference` when wrapping
these in scripts; in Nushell/Ion, prefer `^gnatprove ...` explicit external calls.)

---

## 8. Common Proof Failures and How to Fix Them

| Message (paraphrased) | Cause | Fix |
|-----------------------|-------|-----|
| "overflow check might fail" | arithmetic can exceed the type | add a guard the prover can see, or use a wider/modular type, or a `Pre` |
| "range check might fail" | value may leave a subtype | constrain the source type or add `Pre`; index by the array's own index type |
| "array index check might fail" | index not provably in bounds | use the index subtype; add `Loop_Invariant` bounding the index |
| "precondition might fail" | caller doesn't establish `Pre` | strengthen caller, or weaken `Pre` if too strong |
| "postcondition might fail" | body doesn't establish `Post`, or proof lacks a lemma | add `Loop_Invariant`/intermediate `pragma Assert` to guide the prover |
| "loop invariant might not be preserved" | invariant too weak or wrong | restate what truly holds each iteration; reference `'Loop_Entry` |
| "subprogram might not terminate" | no variant on a loop | add `pragma Loop_Variant (Decreases => ...)` |
| "possible aliasing / not allowed in SPARK" | two mutable paths to one object | restructure to owned values; move instead of copy |

When the prover is *close*, a well-placed `pragma Assert` that restates an
intermediate fact often unblocks it. When it is not close, the contract or the type
is usually wrong — fix the design, do not paper over it with `pragma Assume`.

---

## 9. Assurance-Level Adoption Path

Adopt incrementally; do not attempt Platinum first.

1. **Stone** — get the unit into the SPARK subset (`--mode=check_all`). Move
   non-SPARK code behind `SPARK_Mode => Off` bodies, keeping specs in SPARK.
2. **Bronze** — pass flow analysis; declare `Global`/`Depends`; fix initialization.
3. **Silver** — discharge all AoRTE checks. This is the floor for safety-critical
   units. Most effort is strengthening types and adding loop invariants.
4. **Gold** — add and prove the key integrity properties (`Pre`/`Post`/invariants).
5. **Platinum** — full functional correctness, only for the small kernel that
   warrants the cost.

Record per unit, in the project ADR: the target level, the actual level
`gnatprove` confirms, and the rationale for the SPARK/Rust boundary.

— Spacecraft Software, 2026
