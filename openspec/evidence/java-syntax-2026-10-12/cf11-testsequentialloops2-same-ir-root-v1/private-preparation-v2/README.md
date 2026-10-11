# SequentialLoops2 same-IR diagnostic candidate v2

This private v2 corrects the target method to `asciiToLowerCase(Ljava/lang/String;)Ljava/lang/String;`, verified against the frozen javac 23 original `javap` output and the Jarde all-profile report. The class is major version 52, so the site census uses Java release 8, matching the existing `analyzed_caller` test setup.

The candidate is intentionally limited to the suspected rejection seam: exact SSA instructions at BCIs 83–99, canonical CFG and NormalFlow edges, referenced ValueIds with definitions and uses (including `bci=None` Phi operands), the direct `outside_readers` result for producer BCIs 91/94/96, and the existing `new@1` site census/refusal. It does not attempt to infer a root cause.

The full candidate and minimal insertion patch are pinned in `pins.json`. SHA-256 pins file bytes; the candidate logs Jarde's BLAKE3 class digest without conflating the algorithms. `Gap` has no position accessor; the `new@1` refusal message itself carries the site BCI, so the logger prints only `code` and `message`.

This is static preparation only. It has not been compiled or run, no shared file was changed, and no Cargo, Git, Java, Jarde CLI, or toolchain command was executed. The future test command is recorded for root after product delivery and lock release.

Interpret the trace narrowly: a unique Phi operand/use chain can identify a candidate reader position for `Sites`; it must not by itself accept arbitrary Phi-mediated allocations. The builder's full conditional proof remains a separate requirement, and multi-arm, alias, and multiple-consumer cases need independent proof before any implementation choice.
