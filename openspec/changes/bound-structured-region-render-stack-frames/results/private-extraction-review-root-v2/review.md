# Private Builder frame candidate v2

Baseline: formatted shared `crates/jarde-java/src/build.rs`, SHA-256 `582685682efcd19462996ccc826f3a516dc8b148d7b1112d029ec5a1520fd565`. This baseline already contains the independently accepted Try-only extraction; it is not the earlier v1 source.

This candidate extracts the exact `Region::Loop` and `Region::Guard` arm bodies into private `loop_region` and `guard_region` methods. The Region dispatcher still performs the same early special-region routes, incomplete-declaration refusal, Guard checkpoint creation, and exactly one `declare_at(path)` before dispatch. The Loop and Guard match arms now only call their helper. The checkpoint is moved into `guard_region` after the same prelude; the helper retains the same `Region::Guard` destructure and all existing `take`/restore sites. No new type, public API, dependency, stack wrapper, or recursion/depth change is added. Try remains the existing private helper from this baseline.

The extracted Loop and Guard arm bodies compare exactly after indentation normalization. Their internal statement, scope, checkpoint, budget, origin, Stop/Fallback, and recursive body order is unchanged. Helpers are private methods on the existing Builder and are called only for their matching Region variant.

Source size review in this baseline (top-level `Builder::region` match arm line spans, used only as a rough complexity indicator): `If` 92 lines, `CompoundIf` 96, `StringSwitch` 142, `Switch` 158, `Loop` 266, `Guard` 1465. Try is already dispatched to its helper. The Guard branch is the largest remaining arm by a wide margin; the two medium switch arms remain unchanged because this candidate prioritizes the directly requested Loop and Guard and avoids widening the refactor before a new frame measurement. The 190,912-byte region frame on Try-only v1 shows more reduction is needed; only root's measured default-stack run and machine-code frame can establish whether this candidate is sufficient.

No Cargo/Rust compiler/formatter, JDK, JADX, CLI, or Git command was run. This is private static preparation; no pass is claimed.
