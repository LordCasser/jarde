# Audit v2

This is a private correction to audit v1. V1 remains unchanged.

Corrections:

- The public stop tests are permanent integration tests in `crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs`; root ran the corrected public suite with command summaries 2/8/2 passing. The late-stop assertion checks the real BCI is a physical instruction and usage is within the limit, while raw observed the specific `@7`, usage 516, written 633. The fixed BCI and usage are observations, not source-level fixed assertions.
- Injected canonical edge rows are accepted as validator-level unit coverage when they use actual Reader-produced identities/base rows and invoke the production validator. OpenSpec does not require every external/duplicate/non-Normal edge case to be synthesized as a separate classfile. This coverage still must not be generalized to traversal, integration, or public fallback coverage.

The audit records source line references and root evidence paths. It is read-only and does not modify the repository or run tools.
