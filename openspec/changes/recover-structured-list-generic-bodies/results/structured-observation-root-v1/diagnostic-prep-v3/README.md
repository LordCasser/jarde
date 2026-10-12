# CF11 structured-void/generic diagnostic prep v3

This directory contains a prepared, observation-only runner for one fixed
`TestNestedLoops` method-signature diagnostic. The runner is intended for root
to consider after the product's own CI is complete; preparing this directory
does not start a build or diagnostic.

The runner checks the two pinned product base files and the fixed fixture,
applies the pinned diagnostic-only patch, runs the existing public API test,
and restores both product files byte-for-byte. It enumerates source pins from
the closure helper at runtime and records the actual count. Its result path is
`/private/tmp/jarde-cf11-structured-void-generic-diagnostic-run-root-v3/`.

See [runplan.md](runplan.md) for the exact scope, invariants, and invocation.
