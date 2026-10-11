# For-condition checker v6 private candidate

This is a minimal checker-only extension over checker v5. It independently reads each physical Code attribute exception table (start_pc, end_pc, handler_pc, catch_type resolved through CONSTANT_Class) and checks the complete ordered table against the method-specific `javap -v` exception table. It retains owner, Code-envelope, instruction-BCI, source-map, exact execution, and raw-stream checks. The previous unconditional `exception_count == 0` requirement is replaced by a count-agreement check against the directly parsed Code table.

The candidate targets the already captured baseline at `/private/tmp/jarde-conditional-constructor-values-baseline-root-v1/execution.json`. Its class/javap/parser and v5 checker identities are pinned in `pins.json`; the source evidence includes `handlerAroundConditional` in the refusals class. The root should run the candidate against that immutable execution only after review. This preparation did not execute the checker, collector, or any toolchain.

`check_controls.patch` contains the exact v5-to-v6 diff.
