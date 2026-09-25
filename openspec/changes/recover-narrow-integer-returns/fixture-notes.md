# Fixture notes

`tests/fixtures/p3-narrow-integer-returns/` contains one permanent class,
`NarrowIntegerReturns.class`, and one source-only runner. The Java source intentionally returns
`int` so the exact descriptor patch can change selected return descriptors to `byte`, `char`, or
`short` while leaving every Code attribute byte unchanged. The class contains byte/char/short
local readback plus an int-return control, direct returns, field post/pre increments, and
synchronized returns. It does not contain a condition stack phi, switch boundary, or explicit
conversion opcode.

`openspec/evidence/java-syntax-2026-09-22/numeric-conversions/narrow-return-fixture/patch_descriptors.py`
parses method_info names, adds only the required UTF-8 descriptors, updates the selected
method_info descriptor indexes, and compares every Code attribute before and after. The audit
copies the patch record, compiles the runner against the patched class, executes the original with
`java -Xverify:all`, then records complete Jarde and JADX source compile/run stages. The current
audit records 1,288 bytes, SHA-256
`90219c1ac7c93b53bee50da43ac792cd7e628dba4bff7b0f697a8b64a9b5d14c`, 14 methods and 49 runtime
lines. The deferred worker rebuilt the CLI during the broader fixture work; this fresh audit used
`948a6f9b7db009fbb89a792f83328c9ab63e42ac0cb62e691d97bec28c2c8ba0` before and after, with no
within-audit hash change.
