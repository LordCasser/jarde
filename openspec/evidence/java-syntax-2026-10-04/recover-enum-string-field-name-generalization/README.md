# recover-enum-string-field-name-generalization — implementation evidence (2026-10-04)

Implements the OpenSpec change of the same name: the accepted DT-12 String-argument enum slice
hard-coded the String source argument's field name to `op` — exactly its anchor
`TestEnums2a/DoubleOperations`' field name, so its acceptance was self-fulfilling. This slice
makes the constructor bytecode's `putfield` target the only authority for that name, carries the
proved field identity to the emission stage, and freezes a non-`op` control probe into CI.

Worktree baseline: mainline `c226413c`. All renders below produced by `jarde-cli class-source`
(debug profile) with `--policy plain-jar`, over jars/zips packed with the `demo/` prefix intact.

## Data path: field index, not bare name (design decision 2 primary path)

The index path was confirmed available; the fallback (bare name bytes) was not needed:

- The putfield target is bound in `prove_enum_string_constructor_candidate` (the 14197 site,
  `src/facade.rs`), which now also receives the physical field records (`read.facts.fields`) from
  its only caller, the `constructor_chain` closure inside `resolve_enum_constant_body_relations`.
  It resolves the target to **one** field-table index by (raw name, raw descriptor) with the
  static/enum-flagged records excluded — the same resolution shape the already-shipped arbitrary
  tail proof `prove_arbitrary_user_constructor` uses (`enum_constants.rs`), including its
  "absent or ambiguous" refusal.
- The carrier is a new `assigned_field: Option<PendingEnumAssignedField>` on
  `PendingEnumConstructorEdge` (`{ index: u64, name, descriptor }`), set only by the
  String-shape's private constructor edge; the other three edge constructors set `None`. The
  emission stage reads it from `shape.constructor_chain[0]` — the private edge is always first by
  construction.
- **Same index space (design Open Question 1: yes).** The source `fields` vector is built by one
  ordered pass over `read.facts.fields` (`for (index, field) in read.facts.fields.iter().enumerate()`,
  one `ClassSourceField` push per header, same index), and the same vector object is passed both
  to `resolve_enum_constant_body_relations` (as `source_fields`) and to
  `prove_enum_constant_body_group` (as `source_fields`, facade.rs call site ~8630). The shipped
  arbitrary-tail channel already indexes `source_fields` with an index found in the physical
  list, so the equivalence is proven behavior on main. The emission stage therefore locates the
  proved field with `source_fields.get(index)` and never re-matches by name.

## The four criterion replacements + emission text

- 16009 (find by `name == b"op"` + attribute filter) → `source_fields.get(assigned.index)` with
  the attribute filter **preserved verbatim** (descriptor `Ljava/lang/String;`,
  owner == this enum definition, flags `& (0x0008 | 0x0040 | 0x1000) == 0`, `ACC_PRIVATE`, has
  `declaration`, no markers), now written as refusal conditions, plus the identity recheck
  (`field.item.name/descriptor == assigned.name/descriptor`).
- 16023 (count fields named `op`) → count fields matching the proved identity
  (`assigned.name` + `assigned.descriptor`); `!= 1` refuses as "the source String field is
  ambiguous" (text kept). This is the documented relaxation: an enum with two String fields where
  the constructor writes only one now **projects** (`negative-controls-cli.txt`, `twofields`:
  baseline count 0, fixed count 1); refusal fires only when the written target itself is not one
  field (`twostores`: both legs refuse loudly).
- 16026 (`op_field... != b"op"`) → the located field must equal the proved putfield identity.
- 16031 (emitted `this.op = arg0;`) → `this.<proved name> = arg0;`, the name spelled from the
  located field's own raw name (`String::from_utf16(name.utf16())` + `is_java_identifier`, the
  same spelling chain this function already applies to constant names one block above; aliased
  names are already excluded by the verbatim `markers.is_empty()` check).
- 14197 error text: `"the String constructor does not preserve Enum and op semantics"` →
  `"the String constructor does not preserve Enum and the assigned field semantics"`. **No test
  or fixture asserted the old text** (searched `src/`, `crates/*/src/`, `tests/` for
  `does not preserve Enum`, `op semantics`, `this.op`: only the definition sites). The adjacent
  shape-refusal text `"…not a pure `op = arg0` constructor"` was de-`op`-ed the same way
  (`…not a pure single-argument field store`); also unasserted.

## Zero-regression anchor: `TestEnums2a/DoubleOperations` (field name `op`)

- `renders/op-anchor-baseline.txt` vs `renders/op-anchor-fixed.txt`: **byte-identical**
  (`op-anchor-byte-identical-proof.txt`, sha256 of the render recorded there). Same compile
  (`javac --release 8 -g:none`, javac 23.0.1 — the frozen fixture's own toolchain), same jar.
- Against the frozen DT-12 transcription
  (`java-syntax-2026-09-27/dt12-anonymous-enum-audit/fixed/outputs/.../DoubleOperations.java`):
  `renders/op-anchor-vs-frozen-transcription.diff` — the only differences are (a) today's main
  class-Signature marker lines (present identically in both my baseline and fixed renders — they
  cancel in the before/after comparison) and (b) the snapshot id (different jar composition: my
  op.jar has no Runner.class). The frozen child class digests
  (`2f837d0c…`, `b1866ff7…`) match my recompiled bytes exactly, proving the toolchain
  reproduction is faithful. The anchor still projects the constant-list form on today's main
  before and after this slice.

## Non-`op` positive (tasks 1.3): `demo.LabeledOps` (field `label`, getter `getLabel`)

Frozen into `tests/fixtures/enum-string-field-name/` (sources, `javac --release 8 -g:none`
classes, Probe.java, `original-behavior.txt`; toolchain javac 23.0.1). CI-referenced by
`tests/enum_string_field_name.rs` (include_bytes! + Engine + ClassSourceRequest; one text-guard
test, one recompile-and-run test) — verified **red on the unfixed library, green after**
(`git stash` run during development: both tests fail at `c226413c` lib, pass with the fix).

- `renders/labeled-baseline-degraded.txt`: constants degrade to
  `public static final demo.LabeledOps TIMES;` — illegal Java.
- `renders/labeled-fixed-projected.txt`: `TIMES("*") { … }, DIVIDE("/") { … };` +
  `private LabeledOps(java.lang.String arg0) { this.label = arg0; }`.
- `javac/labeled-baseline-javac.log`: full source set `javac --release 8` **exit 1**
  (`此处需要枚举常量`) → `javac/labeled-fixed-javac.log`: **exit 0**.
- `runs/labeled-original-run.txt` vs `runs/labeled-projected-run.txt`: `java -Xverify:all`
  outputs **byte-identical** (`TIMES=*:0:6.0:demo.LabeledOps$1` / `DIVIDE=/:1:2.0:demo.LabeledOps$2`).

## Negative controls (tasks 1.4)

In-crate tests (`src/facade.rs`, module `enum_constant_body_relation_tests`):

- `string_constructor_negatives_refuse_loudly` — a constructor that stores the argument into a
  second String field refuses the whole group; single-byte constructor mutations (iload→iconst at
  the ordinal slot, putfield→getfield at the store) either refuse the chain or stop the member
  run — never a projection, physical text kept in every case.
- `string_constructor_identity_binds_the_written_field_and_refuses_everything_else` — direct unit
  coverage of the 14197 gate: canonical store binds `index=2, name="label"`; non-String store
  descriptor, foreign owner, unresolved reference →
  `"the String constructor does not preserve Enum and the assigned field semantics"`;
  absent/duplicate/static-or-enum-flagged target →
  `"the String constructor's assigned field target is absent or ambiguous"`.
- `a_second_unwritten_string_field_leaves_the_proved_target_unique` — the design Risks section's
  relaxation boundary: two String fields, one written → projects (`this.op = arg0;` + the spare
  field), recompiles.
- CLI-level transcript of the same two boundary cases: `renders/negative-controls-cli.txt`.

Note: a type-consistent non-String putfield of a String argument is not constructible at the
bytecode level (the value's type forces the descriptor), which is why the descriptor gate's
end-to-end negative lands as a member-run stop (loud, physical text) while the gate itself is
unit-covered above.

## Gates (all run from the worktree; numbers as measured)

- `cargo test --workspace --tests --locked --no-fail-fast` — see final report (baseline family
  flakes re-judged by double re-runs; known flake names from handoff.md).
- `cargo fmt --all -- --check` — clean.
- clippy, generated verbatim from `.github/workflows/ci.yml` lines 46–76 (29 `-A` items,
  `--all-features`, `-D warnings`) — exit 0.
- `openspec validate --all --strict` — 272 items.
- corpus two-leg scan — `corpus/corpus-two-leg-scan.txt`: single-class leg 532/532 identical;
  family leg 532 renders, exactly 1 diff (the new `demo/LabeledOps` probe itself), 62 identical
  both-legs non-zero exits, 0 a≠b.
- `git diff --check` — clean.
- corpus fingerprint: `tests/fixtures/corpus-fingerprint.json` regenerated after the fixture
  addition; the diff is a pure 45-line addition listing exactly the 9 new fixture files
  (README.md excluded by the manifest's own `.md` rule).

## Decisions the implementation made within its mandate

- `written_name` (class_source's aliasing helper) was **not** imported into the emission path:
  `markers.is_empty()` already excludes alias-marked fields, and the spelling chain mirrors the
  constant-name precedent in the same function. No class_source API surface was widened.
- The uniqueness count keys on (proved name, proved descriptor) rather than the old name-only
  count. Consequence: a hostile class with `String op` + `int op` now projects (the written
  String field is unique) where the old code refused; javac would then reject the duplicate
  source name loudly. This is inside decision 4's stated boundary ("被写入的 String 字段恰一个").
- Open Question 2 (`class_source.rs:8309` `totalUnits`): **no coupling** — that check lives in
  `prepare_enum_constant_source_projection`, which takes `&ProvedOrdinaryEnumConstantGroup`
  (the Ordinary variant) only; the DT-12 Body group assembles through a separate path
  (facade.rs ~27195). The Ordinary scalar-String proof (`prove_string_constructor`,
  enum_constants.rs) is already index-based and hardcode-free; untouched.

## No `ask_parent` attributions

No scope decision in this report comes from an `ask_parent` answer: no `ask_parent` call was made
in this task.
