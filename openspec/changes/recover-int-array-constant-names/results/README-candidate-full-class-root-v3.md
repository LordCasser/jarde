# Full-class integer-array candidate collector v3

`prepare-candidate-full-class-root-v3.py` is a prepared replacement for the
unrun v2 collector. It preserves v2 inputs and gates, writes to the separate
`candidate-full-class-root-v3/` directory, and does not overwrite earlier
evidence.

The corrected `PriorAssertIntArray` expectation is a positive case: exactly one
`VALUE` projection must be anchored to the physical `VALUE` field and the
`value` method at a real BCI. The saved `javap` body must still contain numeric
7, and the rendered body must retain `$assertionsDisabled` and the
`AssertionError` throw guard rather than an `assert` statement. The other four
controls keep their prior positive/negative expectations.

Each complete controls class set is now run twice on each JDK, with assertions
enabled and disabled. The original source, Jarde-generated sources, and JADX
generated sources are compared against the corresponding original raw exit,
stdout, and stderr for both modes. This exercises the assertion guard while
keeping the full-class source and class-set checks intact. The original
control run must also print `prior-assert-false=caught` with assertions enabled
and `prior-assert-false=not-thrown` with assertions disabled, proving the two
JVM modes actually exercised distinct guard behavior.

The three accepted array-fill oracles remain imported from the separately
accepted baseline. The collector requires the four frozen CLI/metadata
arguments and has not been run; root must inspect and execute it after freezing
the candidate CLI.
