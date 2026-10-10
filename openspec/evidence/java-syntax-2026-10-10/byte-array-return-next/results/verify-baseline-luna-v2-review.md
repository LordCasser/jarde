# Byte-array return baseline verifier v2

Prepared independently from the recorded `baseline-root-v2` evidence. The
script is not run; v1 remains unchanged. V2 fixes two schema/format assumptions
found during review:

Verifier SHA-256: `2d6ee4b2e2bd333ac51568e7e186a99dfb5fad63f7f61b348ccedf56da8f36b0`.

- Original cases have no `runner_class` field. V2 derives `Runner` from the
  original case's recorded runtime argv, requires that value to be exactly
  `Runner`, and uses it when checking original runtime argv. The JADX default
  package-qualified Runner is derived from that same original value plus the
  recorded profile shape.
- JDK 8's javap text omits the `fields: 0, methods: 2` class-summary line. V2
  independently checks the physical declaration block for exactly the
  constructor and `test()[B` method and no fields; it checks the summary when
  the javap version emits one.

All other v1 checks remain: frozen manifest/inventory/tool pins; every raw
command and stream; isolated full-class compile/run cases; exact javac23-only
JADX jar; source/Runner integrity; Jarde report owner and source-map checks
against original javap BCIs; and runtime raw equivalence across both JDKs.
Acceptance output is a new `byte-array-return-independent-acceptance-v2.json`
and refuses to overwrite an existing result.

Only static record inspection and Python bytecode compilation were used for
preparation; no verifier, Java, JADX, Jarde, Rust, or Git command was executed.
