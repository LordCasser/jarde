//! P5 task 1.1: the corpus fingerprint every later measurement is compared against.
//!
//! P5 chooses what to optimize from measurements (design decision 1: measure first), and a
//! measurement is only comparable while the bytes it read are fixed. This file is that fix.
//! `tests/fixtures/corpus-fingerprint.json` records a blake3 digest and a byte count for every
//! file of the corpus the P1–P4 work reads — the checked-in samples and archives below
//! `tests/fixtures/`, the golden documents that pin the digest of fixtures a test builds in
//! memory, and the committed seeds below `fuzz/corpus/` — and the tests below fail when one of
//! those bytes changes, when a corpus file appears that the manifest does not list, or when the
//! manifest drifts from the classification in this file.
//!
//! The manifest is an index, not a second pin. It duplicates no digest a golden document already
//! holds: `Carrier::Generator` names the test file and the builder that produces the bytes,
//! `Carrier::Golden` names the document that records their digest, and the replayed digest stays
//! asserted where it already was. It also asserts nothing about whether an acceptance row passes
//! — that status belongs to the P5 verification record. What it adds is the *inputs*: which bytes
//! carry each of the eight corpus dimensions, which acceptance rows that corpus feeds, and which
//! corpus is not pinned at all (`known_gaps`), so a later measurement cannot quietly move the
//! corpus it is compared against.
//!
//! ```text
//! verify:      cargo test --test p5_corpus_fingerprint --locked
//! with output: cargo test --test p5_corpus_fingerprint --locked -- --nocapture
//! regenerate:  cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint
//! ```
//!
//! The regenerator re-renders the whole document from this file's tables and the current bytes.
//! Run it only for a corpus change that is intended and review the diff; during a normal run
//! nothing writes the manifest, and a changed, missing or unlisted corpus file is a failure.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

/// The document this file verifies, relative to the repository root.
const MANIFEST: &str = "tests/fixtures/corpus-fingerprint.json";
/// The schema tag a reader keys on. Bump it when a field changes meaning, not when one is added.
const SCHEMA: &str = "jarde-corpus-fingerprint/1";
/// The command that rewrites the document, recorded inside it so a reader finds it there too.
const REGENERATE: &str =
    "cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint";

const PURPOSE: &str = "Fix the bytes every P5 measurement reads. Digests here are a fingerprint of \
the corpus, not evidence that an acceptance row passes and not a performance claim.";

/// The two roots the fingerprint covers: the acceptance corpus and the committed fuzz seeds.
const ROOTS: &[&str] = &["tests/fixtures", "fuzz/corpus"];

/// Directories skipped anywhere below a root. `out` is where a fixture-generation run leaves a
/// compiler's output (`tests/fixtures/p4-modern/out`), `artifacts` is where a fuzz run leaves a
/// crash input, and `target` is build output: none of the three is an input a test reads.
const EXCLUDED_DIRECTORIES: &[&str] = &["target", "out", "artifacts", "__pycache__"];

/// Names skipped anywhere below a root: the manifest itself, so re-rendering it does not change
/// its own input set.
const EXCLUDED_FILE_NAMES: &[&str] = &["corpus-fingerprint.json"];

/// Extensions skipped anywhere below a root: `md` is the provenance prose that sits beside the
/// samples and `py` is the script that regenerates the fuzz seeds. Both are records *about* the
/// corpus rather than inputs to it, and pinning them would fail the fingerprint on a doc edit
/// while catching no change in what a test reads. `Carrier::Record` still checks that a
/// provenance document exists.
const EXCLUDED_EXTENSIONS: &[&str] = &["md", "py"];

const SCOPE_NOTE: &str = "Every file below `roots` whose extension is not excluded, minus the \
excluded directory names, is listed in `files` with its blake3 digest and byte count. The \
exclusions are records about the corpus, not inputs to it: `README.md` files name the compiler, \
the command and the expected digests, and the two `.py` files regenerate the fuzz seeds. This is \
a filesystem walk, not a `git ls-files` listing, so a corpus file that is present but untracked \
is fingerprinted too and fails the check on a clean checkout.";

/// The dimensions P5's corpus matrix has to cover (P5 design, Risks: "覆盖版本、打包、身份、
/// 字节码、恢复、退化和对抗矩阵", with the compiler kept separate because it is what makes the
/// historical samples historical).
///
/// `state` is one of:
/// * `pinned` — every carrier is a checked-in file whose bytes `files` fixes;
/// * `partial` — the dimension is carried, but a named part of it exists only as bytes a test
///   builds in memory, with no golden document recording their digest;
/// * `gap` — no carrier yet. None of the eight is a gap today, and the test allows one so a
///   later change can record the loss instead of deleting the row.
const DIMENSIONS: &[Dimension] = &[
    Dimension {
        key: "versions",
        label: "版本",
        state: "pinned",
        carriers: &[
            file("tests/fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class"),
            file("tests/fixtures/historical/ecj-4.6.1/v46/HistoricalControlFlow.class"),
            file("tests/fixtures/historical/ecj-4.6.1/v47/HistoricalControlFlow.class"),
            file("tests/fixtures/historical/ecj-4.6.1/v48/HistoricalControlFlow.class"),
            file("tests/fixtures/historical/ecj-4.6.1/v49/HistoricalControlFlow.class"),
            file("tests/fixtures/historical/ecj-4.6.1/v50/HistoricalControlFlow.class"),
            file("tests/fixtures/historical/ecj-4.6.1/v51/HistoricalControlFlow.class"),
            file("tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
            file("tests/fixtures/p4-modern/v8/ConcatJava8.class"),
            file("tests/fixtures/p4-modern/v16/RecordSample.class"),
            file("tests/fixtures/p4-modern/v17/SealedSample.class"),
            golden("tests/fixtures/p4-golden/version-boundaries.json"),
            generator("tests/p4_feature_registry.rs", "with_version"),
        ],
        note: "45.3–52.0 are eight ECJ outputs of one source, one per historical target; 52/60/61 \
are javac 23.0.1 outputs. The release registry's 45–71 table is read through `p4_feature_registry`'s \
`with_version`, which patches the two version fields onto the checked-in v52 sample: those bytes \
exist only at run time, and `version-boundaries.json` is where their digests are recorded.",
    },
    Dimension {
        key: "compiler",
        label: "编译器",
        state: "pinned",
        carriers: &[
            file("tests/fixtures/historical/ecj-4.6.1/src/HistoricalControlFlow.java"),
            file("tests/fixtures/p3-corpus/Flags.java"),
            file("tests/fixtures/p4-modern/src/RecordSample.java"),
            file("tests/fixtures/p4-modern/src/SealedSample.java"),
            file("tests/fixtures/p4-modern/src/ConcatSample.java"),
            file("tests/fixtures/r2-annotation-positions/RecordOnly.java"),
            file("tests/fixtures/b2-bootstrap-descriptor/LambdaSample.java"),
            record("tests/fixtures/historical/README.md"),
            record("tests/fixtures/p3-corpus/README.md"),
            record("tests/fixtures/p4-modern/README.md"),
        ],
        note: "Two compilers produce the corpus: ECJ 4.6.1 with `-source 1.3` for the 45–52 \
samples (the `--release 8` story is not a substitute for that codegen) and javac 23.0.1 for P3, \
P4 and the R2/B2 samples. The compiler is a generation-only input and is not checked in, so what \
the fingerprint fixes is the *source* it was given — the `.java` files here — plus the recorded \
command and version in each directory's README.",
    },
    Dimension {
        key: "packaging",
        label: "打包",
        state: "partial",
        carriers: &[
            file("fuzz/corpus/artifact_tree/minimal-jar"),
            file("fuzz/corpus/artifact_tree/nested.jar"),
            file("fuzz/corpus/artifact_tree/multi-release.jar"),
            file("fuzz/corpus/artifact_tree/damaged-entry.jar"),
            file("fuzz/corpus/artifact_tree/truncated-jar"),
            file("fuzz/corpus/query/minimal-jar"),
            file("fuzz/corpus/query/minimal-class"),
            golden("tests/fixtures/p1-golden/nested.json"),
            golden("tests/fixtures/p1-golden/multi-release.json"),
            builder("tests/p1_artifact_tree.rs"),
            builder("tests/p1_multi_release.rs"),
        ],
        note: "Checked-in archives fix the ordinary, nested and multi-release shapes as bytes. \
The WAR, Boot-executable and ZIP64 trees, and the STORED/DEFLATED pair that differs only in \
compression, are built in memory by `p1_artifact_tree`/`p1_xref_code`: the nested pair's bytes \
are pinned by `p1-golden/nested.json`, the WAR/Boot/ZIP64 trees are not pinned anywhere (see \
`known_gaps`).",
    },
    Dimension {
        key: "identity",
        label: "身份",
        state: "partial",
        carriers: &[
            file("fuzz/corpus/artifact_tree/nested.jar"),
            golden("tests/fixtures/p1-golden/nested.json"),
            builder("tests/p1_artifact_tree.rs"),
            builder("tests/p1_multi_release.rs"),
        ],
        note: "Identity is a property of a *physical* origin, so it is exercised by the same \
fixtures that carry duplicates: the golden reuses one jar's bytes at two `WEB-INF/lib` entries \
(one STORED, one DEFLATED) and asserts distinct origins, and the artifact-tree builder adds the \
same bytes under several entries and ordinal positions. Only the golden bytes are pinned; the \
builder's duplicate shapes are not.",
    },
    Dimension {
        key: "bytecode",
        label: "字节码",
        state: "pinned",
        carriers: &[
            file("tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
            file("tests/fixtures/b2-bootstrap-descriptor/v8/LambdaSample.class"),
            file("tests/fixtures/method-signature-grammar/v17/ThrowsMixed.class"),
            file("tests/fixtures/r2-annotation-positions/v8/CodeOnly.class"),
            file("tests/fixtures/p4-modern/v17/ConcatSample.class"),
            file("tests/fixtures/JvmBytecodeOracle.java"),
            golden("tests/fixtures/p1-golden/code.json"),
            generator("tests/p1_xref_code.rs", "fixture"),
            builder("tests/p2_frame.rs"),
            builder("tests/p2_cfg.rs"),
        ],
        note: "Instruction-level coordinates (BCI, opcode, constant-pool index, spans) come from \
the ECJ v52 sample and from the in-memory builder that can pin exact indexes; the historical \
matrix is cross-checked against a JDK 25 Class-File API oracle whose own source is checked in. \
What the oracle *prints* is generated at run time from a JDK the repository does not pin — see \
`known_gaps`.",
    },
    Dimension {
        key: "recovery",
        label: "恢复",
        state: "pinned",
        carriers: &[
            file("tests/fixtures/p3-local-rewrite/v8/LocalRewrite.class"),
            file("tests/fixtures/p3-scope/v8/Scope.class"),
            file("tests/fixtures/p3-scope/v8-debug/Scope.class"),
            file("tests/fixtures/p3-handlers/v8/Guarded.class"),
            file("tests/fixtures/p3-handlers/v8/Res.class"),
            file("tests/fixtures/p3-nested-eval/v8/NestedEval.class"),
            file("tests/fixtures/p3-refused-cast/v8/RefusedCast.class"),
            file("tests/fixtures/p3-refused-cast/v8/External.class"),
            file("tests/fixtures/p3-refused-cast/v8/Holder.class"),
            file("tests/fixtures/p3-corpus/v8-gnone/Flags.class"),
            file("tests/fixtures/p3-corpus/v8-g/Flags.class"),
            file("tests/fixtures/p3-corpus/v8-glines/Flags.class"),
            file("tests/fixtures/p3-corpus/v8-parameters/Flags.class"),
            file("tests/fixtures/p3-corpus/v8-source-target/Flags.class"),
            file("tests/fixtures/p3-corpus/v8-missing-dep/MissingDependency.class"),
            file("tests/fixtures/p3-declaration/v8/Shape.class"),
            file("tests/fixtures/p3-declaration/v8/Holder.class"),
            file("tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
            file("fuzz/corpus/method_analysis/jsr-ret.class"),
            file("fuzz/corpus/method_analysis/legacy-clone.class"),
            golden("tests/fixtures/p2-golden/legacy-clone.json"),
            golden("tests/fixtures/p2-golden/historical.json"),
            builder("tests/p3_execution_comparison.rs"),
        ],
        note: "The recovery dimension is the most byte-dependent one: a slot reuse, a debug \
attribute or an entry count changes which local a value lands in, so every sample P3's \
compile-and-execute comparison reads is checked in, including the flag matrix (which exists to \
show `-g:none` and `-g:lines,source` do not read the same), the sample whose dependency is \
deliberately absent, and the declaration sample, whose two classes are what tell an interface's \
`default`/`static` member from a class's ordinary one.",
    },
    Dimension {
        key: "degradation",
        label: "退化",
        state: "pinned",
        carriers: &[
            file("fuzz/corpus/query/truncated-class"),
            file("fuzz/corpus/query/corrupt-payload.jar"),
            file("fuzz/corpus/query/damaged-candidate.jar"),
            file("fuzz/corpus/artifact_tree/truncated-jar"),
            file("fuzz/corpus/artifact_tree/damaged-entry.jar"),
            file("tests/fixtures/p3-corpus/v8-missing-dep/MissingDependency.class"),
            file("tests/fixtures/p3-corpus/absent-Library-stub/absent/Library.java"),
            golden("tests/fixtures/p2-golden/resource-boundary.json"),
            builder("tests/p1_query_bounds.rs"),
            builder("tests/p2_entry_counts.rs"),
        ],
        note: "Degraded inputs are truncated, corrupt and over-budget ones: the committed fuzz \
seeds carry the first two, `resource-boundary.json` the third, and the missing-dependency sample \
is the fourth — its stub is checked in *as a source* and deliberately not shipped as a class, \
because the absent provider is what the fixture is about.",
    },
    Dimension {
        key: "adversarial",
        label: "对抗",
        state: "partial",
        carriers: &[
            file("fuzz/corpus/artifact_tree/damaged-entry.jar"),
            file("fuzz/corpus/method_analysis/exception-overlap-mixed.class"),
            file("fuzz/corpus/method_analysis/wide-switch.class"),
            file("fuzz/corpus/method_analysis/wide-switch.jar"),
            golden("tests/fixtures/p4-golden/illegal-modern.json"),
            golden("tests/fixtures/p2-golden/wide-switch.json"),
            builder("tests/p1_xref_properties.rs"),
            builder("tests/p2_properties.rs"),
            builder("fuzz/fuzz_targets/method_analysis.rs"),
        ],
        note: "The adversarial matrix is unknown constant-pool entries, illegal indexes, over-long \
attributes, cyclic and over-budget graphs and illegal release shapes: the illegal-modern golden \
and the committed seeds cover the named shapes, the fuzz targets replay them under a budget, and \
the two proptest suites cover the generated ones — those cases exist per run only, pinned by \
CI's fixed `PROPTEST_RNG_SEED` rather than by a digest (see `known_gaps`).",
    },
];

/// Every acceptance row of `openspec/acceptance.md`, with the corpus it reads.
///
/// The theme is the row's subject; `corpus` is the input the row's evidence column names, as
/// carriers. This table says nothing about whether a row passes: a row whose corpus exists and
/// whose behaviour is not implemented yet still appears here, because the corpus is what task
/// 1.1 fixes. The status of each row is recorded in the P5 verification record.
const ACCEPTANCE_ROWS: &[Row] = &[
    Row {
        id: "A01",
        theme: "未使用 Methodref 不是调用",
        corpus: &[
            generator("tests/p1_xref_code.rs", "fixture"),
            golden("tests/fixtures/p1-golden/code.json"),
        ],
        note: "one fixture carries a used and an unused `Methodref`: the constant-pool hit and the \
X1 consumer's absence are asserted over the same bytes.",
    },
    Row {
        id: "A02",
        theme: "invokevirtual 位置精确",
        corpus: &[
            file("tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
            file("tests/fixtures/JvmBytecodeOracle.java"),
            generator("tests/p1_xref_code.rs", "fixture"),
        ],
        note: "the ECJ v52 sample supplies the real invocation, the JDK 25 oracle the independent \
reading of the same instruction, and the builder the CP index and span coordinates.",
    },
    Row {
        id: "A03",
        theme: "annotation/signature/catch 中独有类型",
        corpus: &[
            generator("tests/p1_xref_metadata.rs", "meta_fixture"),
            file("tests/fixtures/r2-annotation-positions/v17/RecordOnly.class"),
            file("tests/fixtures/r2-annotation-positions/v8/CodeOnly.class"),
            file("tests/fixtures/method-signature-grammar/v17/ThrowsMixed.class"),
        ],
        note: "the R2 samples place a type in a nested annotation on a record and in an entry the \
class comment cannot hold; the signature sample puts a type variable in a `throws` clause; the \
builder covers the candidate-filter cases a compiler will not emit.",
    },
    Row {
        id: "A04",
        theme: "LambdaMetafactory 与任意 bootstrap",
        corpus: &[
            file("tests/fixtures/b2-bootstrap-descriptor/v8/LambdaSample.class"),
            golden("tests/fixtures/p1-golden/bootstrap.json"),
            generator("tests/p1_xref_bootstrap.rs", "descriptor_bootstrap_fixture"),
            builder("tests/p4_modern_facts.rs"),
        ],
        note: "the checked-in lambda sample carries a linkable metafactory call; the builder adds \
the unused table entries, the condy forms and the illegal shapes javac cannot be asked to emit.",
    },
    Row {
        id: "A05",
        theme: "nested condy 与共享图",
        corpus: &[
            generator("tests/p1_xref_bootstrap.rs", "real_lambda_fixture"),
            golden("tests/fixtures/p1-golden/bootstrap.json"),
            builder("tests/p4_modern_facts.rs"),
        ],
        note: "condy chains, a shared subgraph and cycle/visited bookkeeping are hand-built: a \
container and its nested arguments need exact pool indexes, and javac 23.0.1 emits no \
`CONSTANT_Dynamic` at all.",
    },
    Row {
        id: "A06",
        theme: "root/11/17 MR 选择",
        corpus: &[
            golden("tests/fixtures/p1-golden/multi-release.json"),
            generator("tests/p1_multi_release.rs", "a06_fixture"),
            file("fuzz/corpus/artifact_tree/multi-release.jar"),
            builder("tests/p4_runtime_matrix.rs"),
        ],
        note: "the golden holds base/v11/v17 entries of one class and replays the Java 8/11/17 \
selections beside one physical query; the committed jar is a second, independent MR shape.",
    },
    Row {
        id: "A07",
        theme: "WAR 同名类",
        corpus: &[
            builder("tests/p1_artifact_tree.rs"),
            golden("tests/fixtures/p1-golden/nested.json"),
            builder("tests/p4_runtime_matrix.rs"),
        ],
        note: "distinct ordinals, origins and explicit loader/order selections are built in \
memory; the golden fixes the same bytes at two origins, which is the identity half.",
    },
    Row {
        id: "A08",
        theme: "DEFLATED nested JAR",
        corpus: &[
            file("fuzz/corpus/artifact_tree/nested.jar"),
            golden("tests/fixtures/p1-golden/nested.json"),
            generator("tests/p1_xref_code.rs", "zip_with_deflate"),
        ],
        note: "one committed nested jar, one golden that stores the same bytes twice — once \
STORED and once DEFLATED — and a builder that can truncate a candidate beside a readable sibling.",
    },
    Row {
        id: "A09",
        theme: "历史 jsr/finally",
        corpus: &[
            file("tests/fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class"),
            file("tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
            golden("tests/fixtures/p2-golden/historical.json"),
            file("fuzz/corpus/method_analysis/jsr-ret.class"),
            file("fuzz/corpus/method_analysis/legacy-clone.class"),
        ],
        note: "the 45–48 ECJ outputs are where `jsr`/`ret` and the shared subroutine survive; the \
committed seeds add the clone and the overlapping-handler shapes, and the P2 golden replays the \
whole historical row.",
    },
    Row {
        id: "A10",
        theme: "缺失 StackMap/debug",
        corpus: &[
            golden("tests/fixtures/p2-golden/inputs.json"),
            file("tests/fixtures/p3-scope/v8-debug/Scope.class"),
            file("tests/fixtures/p3-corpus/v8-gnone/Flags.class"),
            builder("tests/p2_frame.rs"),
            builder("tests/p2_ssa.rs"),
        ],
        note: "the P2 golden's `missing debug` group is the no-attribute case; the P3 flag matrix \
is the same source compiled with and without `-g`, which is the pair that shows naming must not \
read the debug attribute.",
    },
    Row {
        id: "A11",
        theme: "Base.foo 声明、Sub CP owner",
        corpus: &[
            builder("tests/p2_declaration_refs.rs"),
            builder("tests/p2_resolution.rs"),
            builder("tests/p4_runtime_matrix.rs"),
            file("tests/fixtures/p3-corpus/v8-missing-dep/MissingDependency.class"),
        ],
        note: "the symbolic owner, the widened candidate set and the missing-dependency case are \
built by the P2 resolution suites; the shipped P3 sample supplies a real reference whose \
declaration is genuinely absent from the corpus.",
    },
    Row {
        id: "A12",
        theme: "accessor/concat 恢复隐藏调用",
        corpus: &[
            file("tests/fixtures/p3-local-rewrite/v8/LocalRewrite.class"),
            builder("tests/p3_accessor_edges.rs"),
            file("tests/fixtures/p4-modern/v17/ConcatSample.class"),
            file("tests/fixtures/p4-modern/v8/ConcatJava8.class"),
            builder("tests/p4_modern_facts.rs"),
        ],
        note: "the P3 samples are the accessor shapes; the P4 pair compiles one concatenation \
under both releases, which is what the modern facts are read from — the *recovery* increment \
over that pair has not happened (P4 recorded it, with its trigger).",
    },
    Row {
        id: "A13",
        theme: "成员级失败",
        corpus: &[
            generator("tests/p1_xref_code.rs", "broken_member_fixture"),
            generator("tests/p1_xref_code.rs", "dead_code_fixture"),
            builder("tests/p2_members.rs"),
            file("tests/fixtures/p3-corpus/v8-missing-dep/MissingDependency.class"),
            golden("tests/fixtures/p2-golden/historical.json"),
        ],
        note: "one undecodable member beside readable ones is built by the builder, and the \
goldens record the per-plane answers (representation, quality, execution, diagnostics) that a \
failure must not collapse into one.",
    },
    Row {
        id: "A14",
        theme: "全范围中断/缺失依赖",
        corpus: &[
            golden("tests/fixtures/p2-golden/resource-boundary.json"),
            file("fuzz/corpus/query/truncated-class"),
            file("fuzz/corpus/artifact_tree/truncated-jar"),
            builder("tests/p1_query_bounds.rs"),
            builder("tests/p2_entry_counts.rs"),
        ],
        note: "the committed truncated inputs and the resource-boundary golden are the interruption \
cases; the bounds suite drives each of the eighteen limits to its edge.",
    },
    Row {
        id: "A15",
        theme: "冷/热/关缓存完整结果一致",
        corpus: &[
            golden("tests/fixtures/p1-golden/code.json"),
            golden("tests/fixtures/p2-golden/historical.json"),
            builder("tests/p1_xref_golden.rs"),
            file("fuzz/corpus/query/minimal-jar"),
        ],
        note: "the corpus half of A15 already exists: a golden replay fixes a snapshot, a query or \
view and the fully serialized answer, which is the shape a cold/warm comparison compares. No \
cache exists yet, so no cold/warm measurement has been run against these bytes — that is P5 \
1.3/2.x/3.1, and task 1.1 only fixes the inputs it will read.",
    },
    Row {
        id: "A16",
        theme: "单方法按需边界",
        corpus: &[
            builder("tests/p3_recovery_entry.rs"),
            builder("tests/p3_accessor_edges.rs"),
            builder("tests/p3_isolation.rs"),
            golden("tests/fixtures/p2-golden/resource-boundary.json"),
        ],
        note: "what is read is asserted as counts against a fixed fixture (one header, one body, \
one read, three bodies for the on-demand callees) rather than as bytes, because the boundary is \
about what was *not* read.",
    },
    Row {
        id: "A17",
        theme: "X1 零 CFG/SSA/AST",
        corpus: &[
            builder("tests/p3_isolation.rs"),
            builder("tests/p2_contracts.rs"),
            builder("tests/p2_entry_counts.rs"),
        ],
        note: "the construction counts come from the zero-budget/proxy assertions in the entry-count \
and isolation suites, and the source-level token guard reads the guarded files themselves — the \
corpus is this repository's own sources, not a fixture.",
    },
    Row {
        id: "A18",
        theme: "分析期间输入变化",
        corpus: &[
            builder("tests/p1_query_api.rs"),
            builder("tests/p1_query_bounds.rs"),
            file("fuzz/corpus/query/corrupt-payload.jar"),
        ],
        note: "the fixed-byte-source and mid-read cancellation cases are built over a literal \
archive; the committed corrupt payload is the input that must be reported instead of read \
through. The cache/parallel half of the row is P5's and has not run.",
    },
];

/// What the fingerprint does *not* cover. Listed here rather than left implicit, so a later
/// measurement does not read a pinned file list as a pinned corpus.
const KNOWN_GAPS: &[&str] = &[
    "WAR, Boot-executable and ZIP64 trees exist only as bytes a test builds in memory \
(`tests/p1_artifact_tree.rs`): they are replayed and asserted, but no golden document records \
their digest, so a change to that builder moves them without moving this fingerprint.",
    "The STORED/DEFLATED pair and the same-bytes-multiple-origins cases are pinned only where a \
golden already names them (`p1-golden/nested.json`, `p1-golden/multi-release.json`); the other \
shapes those builders produce are not pinned.",
    "The proptest suites (`tests/p1_xref_properties.rs`, `tests/p2_properties.rs`) generate their \
inputs per case. CI fixes `PROPTEST_RNG_SEED`, which makes a run reproducible, but no digest is \
recorded here, so a case-level corpus change is invisible to this fingerprint.",
    "`tests/fixtures/JvmBytecodeOracle.java` is checked in, but the reading it produces comes from \
whatever JDK runs it; P0 records the oracle as a cross-check rather than a pinned input, and this \
fingerprint does not pin a JDK.",
    "`tests/p3_execution_comparison.rs` compiles wrappers generated from the run's own facts with \
the machine's `javac`: the wrappers are a function of pinned bytes, the compiler is not pinned.",
];

struct Dimension {
    key: &'static str,
    label: &'static str,
    state: &'static str,
    carriers: &'static [Carrier],
    note: &'static str,
}

struct Row {
    id: &'static str,
    theme: &'static str,
    corpus: &'static [Carrier],
    note: &'static str,
}

/// One pointer from a dimension or a row to the bytes that carry it.
enum Carrier {
    /// A corpus file: it must appear in `files`, so its bytes are fixed there.
    File(&'static str),
    /// A golden document: a corpus file, and one that records `blake3` digests of the bytes a
    /// test builds in memory. The digests stay asserted where they are; this only points at them.
    Golden(&'static str),
    /// A test file and the named builder in it. The bytes exist per run only, so the check is
    /// that the builder is still there — the name is what a measurement would have to read.
    Generator {
        file: &'static str,
        function: &'static str,
    },
    /// A test file whose bytes are built in more than one place, with no single entry point.
    Builder(&'static str),
    /// A provenance document (a directory README) naming the compiler and command that produced
    /// the samples beside it. Its prose is deliberately outside the walk, so only its existence
    /// is checked.
    Record(&'static str),
}

const fn file(path: &'static str) -> Carrier {
    Carrier::File(path)
}

const fn golden(path: &'static str) -> Carrier {
    Carrier::Golden(path)
}

const fn generator(file: &'static str, function: &'static str) -> Carrier {
    Carrier::Generator { file, function }
}

const fn builder(file: &'static str) -> Carrier {
    Carrier::Builder(file)
}

const fn record(path: &'static str) -> Carrier {
    Carrier::Record(path)
}

/// One fingerprinted file.
#[derive(PartialEq, Eq)]
struct Fingerprint {
    path: String,
    bytes: u64,
    blake3: String,
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn manifest_path() -> PathBuf {
    repository_root().join(MANIFEST)
}

/// Every corpus file, in path order, with the digest of the bytes on disk *now*.
fn corpus_files() -> Vec<Fingerprint> {
    let root = repository_root();
    let mut paths = Vec::new();
    for relative in ROOTS {
        collect(&root, &root.join(relative), &mut paths);
    }
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let data = fs::read(root.join(&path))
                .unwrap_or_else(|error| panic!("read the corpus file {path}: {error}"));
            Fingerprint {
                bytes: data.len() as u64,
                blake3: blake3::hash(&data).to_hex().to_string(),
                path,
            }
        })
        .collect()
}

fn collect(root: &Path, directory: &Path, paths: &mut Vec<String>) {
    let mut entries: Vec<fs::DirEntry> = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read the directory {}: {error}", directory.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("read an entry of {}: {error}", directory.display()))
        })
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let file_type = entry
            .file_type()
            .unwrap_or_else(|error| panic!("stat {}: {error}", path.display()));
        if file_type.is_dir() {
            if !EXCLUDED_DIRECTORIES.contains(&name.as_str()) {
                collect(root, &path, paths);
            }
            continue;
        }
        if !file_type.is_file() || EXCLUDED_FILE_NAMES.contains(&name.as_str()) {
            continue;
        }
        if let Some(extension) = path.extension()
            && EXCLUDED_EXTENSIONS.contains(&extension.to_string_lossy().as_ref())
        {
            continue;
        }
        paths.push(
            path.strip_prefix(root)
                .unwrap_or_else(|_| panic!("{} is below the repository root", path.display()))
                .to_string_lossy()
                .replace('\\', "/"),
        );
    }
}

fn carriers_value(carriers: &[Carrier]) -> Value {
    Value::Array(
        carriers
            .iter()
            .map(|carrier| match carrier {
                Carrier::File(path) => json!({ "kind": "file", "path": path }),
                Carrier::Golden(path) => json!({ "kind": "golden", "path": path }),
                Carrier::Generator { file, function } => json!({
                    "function": function,
                    "kind": "generator",
                    "path": file,
                }),
                Carrier::Builder(path) => json!({ "kind": "builder", "path": path }),
                Carrier::Record(path) => json!({ "kind": "record", "path": path }),
            })
            .collect(),
    )
}

/// The whole manifest, rendered from the tables above and the bytes on disk. The regenerator
/// writes this and the check compares it, so the document cannot drift from either.
fn render_manifest() -> Value {
    let mut files = Vec::new();
    for entry in corpus_files() {
        files.push(json!({
            "blake3": entry.blake3,
            "bytes": entry.bytes,
            "path": entry.path,
        }));
    }
    let mut dimensions = Map::new();
    for dimension in DIMENSIONS {
        dimensions.insert(
            dimension.key.to_string(),
            json!({
                "carriers": carriers_value(dimension.carriers),
                "label": dimension.label,
                "note": dimension.note,
                "state": dimension.state,
            }),
        );
    }
    let mut rows = Map::new();
    for row in ACCEPTANCE_ROWS {
        rows.insert(
            row.id.to_string(),
            json!({
                "corpus": carriers_value(row.corpus),
                "note": row.note,
                "theme": row.theme,
            }),
        );
    }
    json!({
        "acceptance_rows": Value::Object(rows),
        "dimensions": Value::Object(dimensions),
        "files": files,
        "known_gaps": KNOWN_GAPS,
        "purpose": PURPOSE,
        "regenerate": REGENERATE,
        "schema": SCHEMA,
        "scope": {
            "excluded_directories": EXCLUDED_DIRECTORIES,
            "excluded_extensions": EXCLUDED_EXTENSIONS,
            "excluded_file_names": EXCLUDED_FILE_NAMES,
            "note": SCOPE_NOTE,
            "roots": ROOTS,
        },
    })
}

fn recorded_manifest() -> Value {
    let text = fs::read_to_string(manifest_path()).unwrap_or_else(|error| {
        panic!(
            "read {MANIFEST}: {error}. Regenerate it with `{REGENERATE}` if it is missing or \
             was moved."
        )
    });
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{MANIFEST} is not valid JSON: {error}"))
}

fn recorded_files(manifest: &Value) -> BTreeMap<String, Fingerprint> {
    manifest["files"]
        .as_array()
        .unwrap_or_else(|| panic!("{MANIFEST} has no `files` array"))
        .iter()
        .map(|entry| {
            let path = entry["path"]
                .as_str()
                .unwrap_or_else(|| panic!("a `files` entry has no path: {entry}"))
                .to_string();
            let recorded = Fingerprint {
                bytes: entry["bytes"]
                    .as_u64()
                    .unwrap_or_else(|| panic!("{path} has no byte count")),
                blake3: entry["blake3"]
                    .as_str()
                    .unwrap_or_else(|| panic!("{path} has no digest"))
                    .to_string(),
                path: path.clone(),
            };
            (path, recorded)
        })
        .collect()
}

/// The corpus is unchanged: every file the walk finds is listed with the digest of the bytes on
/// disk, and the manifest lists nothing the walk does not find. Either direction is a failure —
/// a rewritten fixture and a fixture the manifest never learned about are both a moved corpus.
#[test]
fn corpus_files_match_the_recorded_fingerprint() {
    let files = corpus_files();
    let recorded = recorded_files(&recorded_manifest());

    println!("corpus fingerprint: {} files", files.len());
    for root in ROOTS {
        let count = files
            .iter()
            .filter(|entry| entry.path.starts_with(&format!("{root}/")))
            .count();
        println!("  {root}: {count} files");
    }
    for dimension in DIMENSIONS {
        println!(
            "  dimension {} ({}): {} carriers, {}",
            dimension.key,
            dimension.label,
            dimension.carriers.len(),
            dimension.state
        );
    }

    let mut problems = Vec::new();
    for entry in &files {
        match recorded.get(&entry.path) {
            None => problems.push(format!(
                "unlisted: {} ({}, blake3 {}) is in the corpus but not in the manifest",
                entry.path, entry.bytes, entry.blake3
            )),
            Some(previous) if previous.blake3 != entry.blake3 || previous.bytes != entry.bytes => {
                problems.push(format!(
                    "changed: {}: recorded blake3 {} / {} bytes, now blake3 {} / {} bytes",
                    entry.path, previous.blake3, previous.bytes, entry.blake3, entry.bytes
                ))
            }
            Some(_) => {}
        }
    }
    for path in recorded.keys() {
        if !files.iter().any(|entry| &entry.path == path) {
            problems.push(format!(
                "vanished: {path} is recorded but the corpus no longer holds it"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "the corpus moved under the fingerprint ({} file(s)):\n{}\n\nIf the change is intended, \
         rerun `{REGENERATE}` and review the diff. If it is not, the corpus is the thing to fix — \
         do not re-record to make this pass.",
        problems.len(),
        problems.join("\n")
    );
}

/// The rest of the document — dimensions, acceptance rows, scope, gaps — is exactly what this
/// file renders. Comparing the parsed halves rather than the text keeps the failure about the
/// field that drifted, and comparing them at all is what stops the two copies from disagreeing.
#[test]
fn the_manifest_matches_its_rendered_classification() {
    let recorded = recorded_manifest();
    let rendered = render_manifest();
    let recorded_keys: Vec<&String> = recorded
        .as_object()
        .unwrap_or_else(|| panic!("{MANIFEST} is not an object"))
        .keys()
        .collect();

    let mut drift = Vec::new();
    for (key, expected) in rendered
        .as_object()
        .expect("the rendered manifest is an object")
    {
        if key == "files" {
            // The file list is compared entry by entry in the test above, where a moved fixture
            // is named with its digests instead of dumping two hundred lines of JSON.
            continue;
        }
        match recorded.get(key) {
            None => drift.push(format!("`{key}` is missing from {MANIFEST}")),
            Some(actual) if actual != expected => drift.push(format!(
                "`{key}` differs from the classification in this file:\n  recorded: {actual}\n  \
                 rendered: {expected}"
            )),
            Some(_) => {}
        }
    }
    for key in &recorded_keys {
        if rendered.get(key).is_none() {
            drift.push(format!("`{key}` is in {MANIFEST} but not rendered here"));
        }
    }
    assert!(
        drift.is_empty(),
        "{MANIFEST} drifted from the tables in tests/p5_corpus_fingerprint.rs:\n{}\n\nRerun \
         `{REGENERATE}` after changing a table.",
        drift.join("\n")
    );
}

/// Every dimension still has something behind it, and every carrier resolves. A dimension whose
/// only carrier was deleted must be recorded as a `gap` — not left as a name with nothing under
/// it, which is what a dimension list that is never read would allow.
#[test]
fn every_dimension_is_carried_by_existing_corpus() {
    let manifest = recorded_manifest();
    let recorded = recorded_files(&manifest);
    let mut problems = Vec::new();

    for dimension in DIMENSIONS {
        if !matches!(dimension.state, "pinned" | "partial" | "gap") {
            problems.push(format!(
                "{}: unknown state `{}`",
                dimension.key, dimension.state
            ));
        }
        if dimension.state != "gap" && dimension.carriers.is_empty() {
            problems.push(format!(
                "{}: state is `{}` with no carrier",
                dimension.key, dimension.state
            ));
        }
        if dimension.note.trim().is_empty() {
            problems.push(format!("{}: no note", dimension.key));
        }
        check_carriers(
            &format!("dimension {}", dimension.key),
            dimension.carriers,
            &recorded,
            &mut problems,
        );
    }

    let keys: Vec<&str> = DIMENSIONS.iter().map(|d| d.key).collect();
    assert_eq!(
        keys,
        vec![
            "versions",
            "compiler",
            "packaging",
            "identity",
            "bytecode",
            "recovery",
            "degradation",
            "adversarial"
        ],
        "the eight dimensions P5's corpus matrix is defined by"
    );
    assert!(
        problems.is_empty(),
        "the corpus index does not resolve:\n{}",
        problems.join("\n")
    );
}

/// All eighteen acceptance rows are indexed, each against corpus that exists. This is the corpus
/// half of the coverage list: it says the inputs a row names are present and pinned (or named as
/// a builder), not that the row passes.
#[test]
fn every_acceptance_row_is_indexed_against_existing_corpus() {
    let manifest = recorded_manifest();
    let recorded = recorded_files(&manifest);
    let mut problems = Vec::new();

    for row in ACCEPTANCE_ROWS {
        if row.corpus.is_empty() {
            problems.push(format!("{}: no corpus", row.id));
        }
        check_carriers(
            &format!("row {}", row.id),
            row.corpus,
            &recorded,
            &mut problems,
        );
    }

    let ids: Vec<&str> = ACCEPTANCE_ROWS.iter().map(|row| row.id).collect();
    let expected: Vec<String> = (1..=18).map(|index| format!("A{index:02}")).collect();
    assert_eq!(
        ids,
        expected.iter().map(String::as_str).collect::<Vec<&str>>(),
        "every row of openspec/acceptance.md is indexed exactly once, in order"
    );
    assert!(
        problems.is_empty(),
        "the acceptance corpus index does not resolve:\n{}",
        problems.join("\n")
    );
}

/// The rows this file indexes are the rows the acceptance table names: a row renamed or removed
/// there must not stay readable here as a silent second list. The status of a row is *not* read
/// from that table — only its identity.
#[test]
fn the_indexed_rows_are_the_rows_the_acceptance_table_names() {
    let path = repository_root().join("openspec/acceptance.md");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut problems = Vec::new();
    for row in ACCEPTANCE_ROWS {
        let plain = format!("| {} |", row.id);
        let bold = format!("| **{}** |", row.id);
        if !text.contains(&plain) && !text.contains(&bold) {
            problems.push(format!(
                "{} is indexed here but openspec/acceptance.md names no such row",
                row.id
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "the acceptance index points at rows the acceptance table does not have:\n{}",
        problems.join("\n")
    );
}

fn check_carriers(
    owner: &str,
    carriers: &[Carrier],
    recorded: &BTreeMap<String, Fingerprint>,
    problems: &mut Vec<String>,
) {
    let root = repository_root();
    for carrier in carriers {
        match carrier {
            Carrier::File(path) | Carrier::Golden(path) => {
                if !recorded.contains_key(*path) {
                    problems.push(format!(
                        "{owner}: {path} is not in the manifest's file list, so its bytes are not \
                         fixed"
                    ));
                    continue;
                }
                if matches!(carrier, Carrier::Golden(_))
                    && !read(root.join(path)).contains("\"blake3\"")
                {
                    problems.push(format!(
                        "{owner}: {path} is named as the document that pins a generated fixture \
                         but records no `blake3` digest"
                    ));
                }
            }
            Carrier::Generator { file, function } => {
                let source = read(root.join(file));
                if !source.contains(&format!("fn {function}(")) {
                    problems.push(format!(
                        "{owner}: {file} has no builder `fn {function}(`, so the pointer is stale"
                    ));
                }
            }
            Carrier::Builder(path) | Carrier::Record(path) => {
                if !root.join(path).is_file() {
                    problems.push(format!("{owner}: {path} does not exist"));
                }
            }
        }
    }
}

fn read(path: PathBuf) -> String {
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// Rewrite the manifest from the tables above and the bytes on disk.
///
/// Ignored so that a normal `cargo test` never writes: this is the explicit step for a corpus
/// change that is intended, and the diff it produces is the review.
#[test]
#[ignore = "writes tests/fixtures/corpus-fingerprint.json; run explicitly for an intended corpus change"]
fn regenerate_corpus_fingerprint() {
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&render_manifest()).expect("render the manifest")
    );
    fs::write(manifest_path(), &rendered)
        .unwrap_or_else(|error| panic!("write {MANIFEST}: {error}"));
    let files = corpus_files().len();
    println!(
        "wrote {MANIFEST}: {files} files, {} bytes of JSON",
        rendered.len()
    );
}
