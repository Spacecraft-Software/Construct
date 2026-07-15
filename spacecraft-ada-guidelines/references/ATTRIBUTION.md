# Attribution & Further Reading

Per The Steelbore Standard §15.3, this records the named prior art whose insights
shaped this skill. The skill's prose is original; it conforms to the standards below
and adapts adoption-level methodology from AdaCore's freely-available guidance.

| Work | Author | License / Status | Source | Scope |
|------|--------|------------------|--------|-------|
| Guidelines for Safe and Secure Ada/SPARK (Release 2026-05) | Patrick Rogers & Michael Frank (AdaCore) | (c) AdaCore | <https://www.adacore.com/> | 41-rule taxonomy, compliance Levels, ISO 24772-2 / CWE mapping (see root `CREDITS.md`) |
| Ada Reference Manual (Ada 2022) | ISO/IEC JTC 1/SC 22 | ISO/IEC 8652:2023 (standard) | <https://www.ada-auth.org/> | Language semantics, contract aspects, tasking model |
| SPARK Reference Manual (Release 27.0w) | AdaCore & Capgemini Engineering | GPL/CC docs | <https://docs.adacore.com/spark2014-docs/html/lrm/> | Subset legality, `Global`/`Depends`/`Refined_*`/`Part_Of`, ownership model, modular analysis |
| SPARK User's Guide | AdaCore / Capgemini | CC-BY / GPL docs | <https://docs.adacore.com/spark2014-docs/> | `gnatprove` modes/levels, ghost code |
| Implementation Guidance for the Adoption of SPARK | AdaCore | Freely published | <https://www.adacore.com/sparkpro> | Stone/Bronze/Silver/Gold/Platinum assurance ladder |
| Ada Quality and Style Guide (AQ&S) | SEI / Ada community | Public | <https://en.wikibooks.org/wiki/Ada_Style_Guide> | Naming and structural conventions |
| DO-178C / DO-333 (Formal Methods Supplement) | RTCA | Standard | RTCA | Certification framing for "when to use SPARK" |

Well-known standards (ISO, RTCA, IEC) are referenced for conformance and do not by
themselves trigger a root `CREDITS.md` (§15.3 carve-out). AdaCore's adoption-level
methodology is credited here as the conceptual basis for the assurance ladder.

Cross-references within the Spacecraft skill set:
- The NASA/JPL *Power of Ten* rules — bounded-loop / no-recursion / restricted-scope budgets reused here.
- `spacecraft-rust-guidelines` — the Rust side of the same high-assurance posture.
- `spacecraft-standard-constitution` — cross-cutting compliance (licensing, SPDX, ISO 8601, naming).

— Spacecraft Software, 2026
