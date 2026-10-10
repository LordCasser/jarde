# Conditional switch boundary IR observer draft

This is a temporary integration-test source for root review. Copy it to `crates/jarde-java/tests/conditional_boundary_ir_observer_tmp.rs` only for the guarded observation, then remove that temporary repository file. It changes no production code and adds no test helper to the repository.

The observer uses the existing public `ArtifactSnapshot`, `inspect_method_bytecode`, `analyze_method_ir`, `MethodIr::code/canonical/ssa`, `CanonicalCfg`, and `SsaTable` APIs. It constructs the same standalone physical method identity and Java 8 environment pattern used by `cf12_proved_local_source_types.rs`; it does not inject facts into `build.rs` or `region.rs`. For each exact method name and descriptor, it prints:

- the physical instruction stream and decoded typed operands, verified against the analyzed `MethodCodeFacts`;
- the declared exception table and validated physical control-flow targets;
- canonical block IDs, call paths, covered original starts, protected handler ordinals, every edge kind/from/to, handler rows, throw sites, graph completeness, and unreachable blocks;
- SSA block entry/exit slots, per-instruction reads/writes, and phis.

The two pinned input classes are the original full-class outputs from the root v2 preflight:

- javac8, 1,494 bytes, SHA-256 `27dea02e6dc3003a822ef7db2f447a6d21a508d1fb83c83622f86c64f4ff1b8b`;
- javac23, 1,488 bytes, SHA-256 `e51368a95639b9bba3cd94a0f93ac38c11e465b984644ca28c4dc88ab6d82eb5`.

The observer includes all five exact methods: `partialBreak(II)String`, `innerLoopBreak(II)String`, `innerSwitchBreak(II)String`, `terminalCase(I)String`, and `caughtExceptionThenFallthrough(II)String`. These method identities are selectors only; the graph and SSA facts come from the class bytes and public analysis result. The two SHA-256 values were computed from the exact files by a read-only Python hash pass and are recorded in `preflight.json`; the Rust observer checks the pinned lengths and prints the runtime BLAKE3 identity used by `PhysicalDefinitionId`. Root should keep the v9 invocation pinned to these SHA-256 inputs when it copies the temporary test into the repository.

Suggested root-only execution after review: put this test file temporarily under `crates/jarde-java/tests/`, run the exact guarded `cargo test -p jarde-java --test conditional_boundary_ir_observer_tmp -- --nocapture` command under the established v9 wrapper and 5 GiB / target 1 GiB checks, preserve stdout/stderr raw, then remove the temporary test file and `cargo clean` in root's required order. This draft itself was not compiled or executed; no Cargo, Git, JVM, JADX, or CLI command was run here. No CFG behavior is claimed before root sees the actual observer output.
