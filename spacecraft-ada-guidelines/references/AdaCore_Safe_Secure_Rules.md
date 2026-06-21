# AdaCore "Safe and Secure Ada/SPARK" Rule Catalogue (Reference Index)

A compact, citable index of the 41 guideline rules from *Guidelines for Safe and
Secure Ada/SPARK* (Patrick Rogers & Michael Frank, AdaCore, Release 2026-05). Use
this to **cite specific rule IDs** when reviewing code — e.g. "this recurses, which
violates **RPP12 (No Recursion)**" — instead of vague advice. Wording below is an
original restatement of intent; for the full normative text, noncompliant/compliant
examples, and exact ISO/CWE mappings, consult the source PDF (see `CREDITS.md`).

## How each rule is classified (in the source)

- **Level** — compliance weight:
  - **Mandatory**: high bug risk; non-conformance needs a documented technical justification.
  - **Required**: medium-to-high risk; non-conformance needs justification.
  - **Advisory**: low direct risk; record the reason but no formal justification required.
- **Remediation** — cost to retrofit: High / Medium / Low / N/A (N/A = design decision, not a coding flaw).
- **Goal** — which property it serves: Maintainability, Reliability, Portability, Performance, Security.
- **Standards mapping** — each rule cites the applicable **ISO/IEC TR 24772-2** vulnerability and **CWE** (Common Weakness Enumeration) entry, which is what makes this set useful for certification evidence (DO-178C/DO-333, EN 50128, ISO 26262, IEC 61508).

**House policy:** treat **Mandatory** and **Required** rules as gates for any
Spacecraft safety-critical unit; deviations go in the project ADR with the
justification the source requires. Advisory rules are the default unless a
documented reason exists.

## DYN - Dynamic Storage Management

Pick the *strictest allocation policy the component can tolerate*, top of this list first.

| ID | Title | Intent |
|----|-------|--------|
| DYN01 | Common High Integrity Restrictions | Apply the standard high-integrity restriction set (e.g. `No_Allocators`) as the baseline. |
| DYN02 | Traditional Static Allocation Policy | Declare all objects statically; no heap at all. |
| DYN03 | Access Types Without Allocators | Allow access types but forbid `new`; point only at declared objects via `'Access`. |
| DYN04 | Minimal Dynamic Allocation | If heap is unavoidable, allocate once at startup/elaboration and never free. |
| DYN05 | User-Defined Storage Pools | If runtime allocation is truly needed, route it through bounded user-defined pools for analyzability. |
| DYN06 | Statically Determine Max Stack | Bound worst-case stack with static analysis (e.g. GNATstack). |

## RCL - Safe Reclamation (only if you deallocate at all)

| ID | Title | Intent |
|----|-------|--------|
| RCL01 | No Multiple Reclamations | Never free the same object twice (no double-free). |
| RCL02 | Only Reclaim Allocated Storage | Only free what was actually allocated. |
| RCL03 | Only Reclaim to the Same Pool | Return storage to the pool it came from. |

## CON - Concurrency

| ID | Title | Intent |
|----|-------|--------|
| CON01 | Use the Ravenscar Profile | Restrict tasking to the Ravenscar subset (strictest, deterministic). |
| CON02 | Use the Jorvik Profile | Use Jorvik (Ada 2022) where its bounded relaxations of Ravenscar are required. |
| CON03 | Avoid Shared Variables for Inter-task Comms | Communicate via protected objects / rendezvous, not bare shared variables. |

## RPP - Robust Programming Practice

| ID | Title | Intent |
|----|-------|--------|
| RPP01 | No "others" in Case Constructs | Force exhaustive case coverage so new values break the build, not silently fall through. |
| RPP02 | No Enumeration Ranges in Case | List each literal explicitly; adding a literal then forces review. |
| RPP03 | Limited "others" in Aggregates | Avoid `others =>` in aggregates so new components are not silently defaulted. |
| RPP04 | No Unassigned Mode-Out Params | Assign every `out` parameter on every path. |
| RPP05 | No "others" in Exception Handlers | Handle named exceptions, not catch-all. |
| RPP06 | Avoid Function Side-Effects | Functions are pure: no global or out writes. |
| RPP07 | Functions Only Have Mode "in" | Function parameters are all mode `in`. |
| RPP08 | Limit Parameter Aliasing | Avoid aliasing between parameters, or between a parameter and a global. |
| RPP09 | Use Pre/Post Contracts | Contract every subprogram. |
| RPP10 | Don't Re-Verify Preconditions in Bodies | Don't defensively re-check a `Pre` inside the body; the contract guarantees it. |
| RPP11 | Always Use Function Results | Never discard a function's return value. |
| RPP12 | No Recursion | No direct or indirect recursion (bounded stack, analyzable). |
| RPP13 | No Reuse of Standard Typemarks | Don't shadow or redefine standard type names. |
| RPP14 | Use Symbolic Constants for Literals | No magic numbers; name every literal. |

## EXU - Exception Usage

| ID | Title | Intent |
|----|-------|--------|
| EXU01 | Don't Raise Language-Defined Exceptions | Don't explicitly `raise` predefined exceptions (`Constraint_Error`, etc.). |
| EXU02 | No Unhandled App-Defined Exceptions | Every user-defined exception has a handler somewhere reachable. |
| EXU03 | No Propagation Beyond Name Visibility | Don't let an exception escape the scope where its name is visible (else it can't be handled by name). |
| EXU04 | Prove Absence of Run-time Exceptions | Use SPARK to **prove** AoRTE. This is the headline rule and maps to the skill's **Silver** floor. |

## OOP - Object-Oriented Programming

Choose the *most restrictive policy the design allows* (OOP01 is strictest).

| ID | Title | Intent |
|----|-------|--------|
| OOP01 | No Class-wide Constructs | Forbid class-wide types and dispatching entirely. |
| OOP02 | Static Dispatching Only | Allow tagged types but no dynamic dispatch. |
| OOP03 | Limit Inheritance Hierarchy Depth | Cap inheritance depth. |
| OOP04 | Limit Statically-Dispatched Calls to Primitives | Constrain static calls to primitive operations. |
| OOP05 | Use Explicit Overriding Annotations | Mark every override `overriding` / `not overriding`. |
| OOP06 | Use Class-wide Pre/Post Contracts | Use `Pre'Class` / `Post'Class` to keep contracts Liskov-substitutable. |
| OOP07 | Ensure Local Type Consistency | Verify substitutability (LSP) of each derived type. |

## SWE - Software Engineering

| ID | Title | Intent |
|----|-------|--------|
| SWE01 | Use SPARK Extensively | Maximize the SPARK-provable subset across the codebase. |
| SWE02 | Enable Optional Warnings, Treat As Errors | `-gnatwa` plus warnings-as-errors (`-gnatwe`). |
| SWE03 | Use a Static Analysis Tool Extensively | Run CodePeer / GNAT SAS and `gnatcheck` continuously. |
| SWE04 | Hide Implementation Artifacts | Encapsulate with private types; hide representation. |

## Spacecraft cross-references

- **DYN02-04, RPP12, RPP14** reinforce the NASA/JPL *Power of Ten* rules (no heap, no recursion, bounded everything, no magic numbers).
- **EXU04 + SWE01** are the formal-verification spine; they map directly to the assurance ladder in `SKILL.md` (Silver = AoRTE proven).
- **CON01/CON02/CON03** match the skill's Ravenscar/Jorvik + protected-object concurrency rules.

— Indexed for Spacecraft Software, 2026
