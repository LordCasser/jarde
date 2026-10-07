//! The frozen `proved-java-structure` fixtures' behavior guards
//! (change `recover-fixture-behavior-guard-coverage`).
//!
//! # Why this file exists
//!
//! `recover-synthetic-ctor-super-order`'s constructor reordering flipped
//! `anonymous-super-dispatch`'s `visibleDuringSuper` from `true` to `false` and CI stayed green
//! (2918 passed): that fixture's behavior baseline lived in its README and its manual `run.sh`,
//! and no test read the class. Eight frozen fixtures were in that state. Two were closed by the
//! guards that followed — `anonymous-super-args` (`tests/class_source.rs`,
//! `tests/ctor_reorder_dispatch_guard.rs`) and `anonymous-super-dispatch`
//! (`tests/ctor_reorder_dispatch_guard.rs`, `tests/anonymous_parameterized_root.rs`) — and this
//! file adds the guards for the remaining six. The discipline it starts: **a new frozen behavior
//! fixture MUST come with a CI test that references it**; `run.sh` is a reproduction tool and
//! never a guard, and `p5_corpus_fingerprint` pins bytes, not behavior.
//!
//! # The two legs
//!
//! * **Presentation leg** — in the default suite, pure text over `Engine::class_source`, no JVM
//!   call, so an order or shape change is visible on every `cargo test`. The anchors are the
//!   *current* presentation at the place the fixture's behavior is decided.
//! * **Behavior leg** — `#[ignore]`, because it needs a JDK on `PATH`: it runs the frozen classes
//!   under `java -Xverify:all`, pins the measured stdout as the golden, and — where the recovered
//!   text compiles — recompiles it and compares the two runs line for line, so a text that
//!   compiles but behaves differently fails here. This is the convention
//!   `tests/p3_execution_comparison.rs` set (CI's JDK step runs such files with `-- --ignored`).
//!
//! # The classification, and the two deliberately uncompilable fixtures
//!
//! | fixture | presentation leg | behavior leg |
//! | --- | --- | --- |
//! | `anonymous-member-base` | member-base ctor order + qualified-receiver refusal | frozen run + uncompilable |
//! | `anonymous-top-level` | child's capture write before the constructor call | frozen run + recompile |
//! | `lambda-body-inline` | forwarding arrow + refused `main` | frozen run + uncompilable |
//! | `enum-arity` | 0/1/4-constant enum headers | frozen run + recompile |
//! | `package-info-basic` | `@java.lang.Deprecated` before `package p;` | frozen run + recompile |
//! | `short-circuit-left-false` | the one short-circuit write | frozen run + recompile |
//!
//! `anonymous-member-base` and `lambda-body-inline` render `jarde_refused_body();` where a body is
//! not recovered, so their text cannot compile: their behavior legs assert the *recorded refusal*
//! (javac exits non-zero with the diagnosis the evidence records) and say so — the fixture is not
//! counted as behaviorally verified. Those assertions fail the day the recovery starts presenting
//! the body, which is the day the classification must be updated, not the day the test is deleted.
//!
//! # Current-presentation pins, and the slices that will move them
//!
//! * `anonymous-top-level`'s root is pinned as the *projection*: ring 2
//!   (`recover-anonymous-supertype-return`, merge `23b69bcf`) unlocked it, so the pre-ring-2
//!   physical text is not the anchor — the projection itself is the frozen fact now. Its child is
//!   pinned with the capture write **before** the constructor call, which is the byte order:
//!   `ctor_order`'s rule moves the group past a constructor call only when the target is exactly
//!   `java/lang/Object.<init>()V`, and `Base`'s constructor runs user code, so this fixture keeps
//!   the byte order (and its child text is deliberately not compilable source — a loud failure
//!   rather than a silently reordered program).
//! * `anonymous-member-base` is `present-proved-java-structure` 5.3's remaining
//!   `this$0` + capture + super-argument coexistence anchor: its anonymous allocation is refused
//!   today, and the slice that unlocks it MUST update this anchor in the same commit.
//! * `lambda-body-inline` is `present-proved-java-structure` 5.2's multi-statement lambda body
//!   anchor: the arrow forwards to the renamed helper today, and the slice that inlines the body
//!   MUST update the anchor with it.
//!
//! ```text
//! default suite:  cargo test --test fixture_behavior_guards --locked
//! behavior legs:  cargo test --test fixture_behavior_guards --locked -- --ignored
//! ```
//!
//! The goldens and the per-fixture SHA-256s are recorded in
//! `openspec/changes/recover-fixture-behavior-guard-coverage/results/README.md`, measured by
//! running the frozen classes (JDK 23.0.1, `java -Xverify:all`) rather than transcribed.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const STORE: u16 = 0;

// ---------------------------------------------------------------------------------------------
// The frozen class files, one group per fixture. The two nested layouts are spelled as they are
// on disk: `enum-arity` keeps its classes in `v8/probe/` and `package-info-basic` in `v8/p/`,
// while the other four are flat. The `v8/` level is a generation marker, not a package: a jar
// entry's name is `probe/…`/`p/…`, so the fixture's own classes bind as themselves.
// ---------------------------------------------------------------------------------------------

const MEMBER_BASE_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-member-base/AnonymousMemberBase.class"
);
const MEMBER_BASE_ONE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-member-base/AnonymousMemberBase$1.class"
);
const MEMBER_BASE_TWO: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-member-base/AnonymousMemberBase$2.class"
);
const MEMBER_BASE_OUTER: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-member-base/AnonymousMemberBase$Outer.class"
);
const MEMBER_BASE_OUTER_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-member-base/AnonymousMemberBase$Outer$Base.class"
);

const TOP_LEVEL_ROOT: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-top-level/AnonymousTopLevel.class");
const TOP_LEVEL_CHILD: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-top-level/AnonymousTopLevel$1.class");
const TOP_LEVEL_BASE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-top-level/Base.class");
const TOP_LEVEL_RENDERER: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-top-level/Renderer.class");
/// The anchor file declares the interface and the abstract superclass in the same unit, so the
/// recompile set derives that part from it instead of restating it.
const TOP_LEVEL_SOURCE: &str =
    include_str!("fixtures/proved-java-structure/anonymous-top-level/AnonymousTopLevel.java");

const LAMBDA_ROOT: &[u8] =
    include_bytes!("fixtures/proved-java-structure/lambda-body-inline/LambdaBodyInline.class");
const LAMBDA_SAM: &[u8] =
    include_bytes!("fixtures/proved-java-structure/lambda-body-inline/IntAction.class");

const ARITY_EMPTY: &[u8] =
    include_bytes!("fixtures/proved-java-structure/enum-arity/v8/probe/Empty.class");
const ARITY_ONE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/enum-arity/v8/probe/One.class");
const ARITY_FOUR: &[u8] =
    include_bytes!("fixtures/proved-java-structure/enum-arity/v8/probe/Four.class");
const ARITY_RUNNER: &[u8] =
    include_bytes!("fixtures/proved-java-structure/enum-arity/v8/probe/EnumArityRunner.class");

const PACKAGE_INFO: &[u8] =
    include_bytes!("fixtures/proved-java-structure/package-info-basic/v8/p/package-info.class");
const PACKAGE_INFO_CHECK: &[u8] =
    include_bytes!("fixtures/proved-java-structure/package-info-basic/v8/p/Check.class");

const SHORT_CIRCUIT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/short-circuit-left-false/ShortCircuitFalse.class"
);

// ---------------------------------------------------------------------------------------------
// The measured behavior goldens: what each fixture's own frozen classes print under
// `java -Xverify:all` (JDK 23.0.1). They are the baselines the behavior legs pin — measured, not
// transcribed from a README (`results/README.md` records the runs and the SHA-256s).
// ---------------------------------------------------------------------------------------------

const MEMBER_BASE_GOLDEN: &str =
    "normal=8;events=outer,argument,base(7),anonymous\nnull=nullOuter\n";
const TOP_LEVEL_GOLDEN: &str =
    "value=23:captured\nevents=capture,choose,base(23),render\ncounts=1,1,1,1\n";
const LAMBDA_GOLDEN: &str = "created captureCalls=1 bodyCalls=0 events=[capture]\nfirst result=12 bodyCalls=1 events=[capture, start:10:2, end:10:2]\nsecond result=13 bodyCalls=2 events=[capture, start:10:2, end:10:2, start:10:3, end:10:3]\n";
const ARITY_GOLDEN: &str = "empty=0\none=ONLY:0/1\nfour=[NORTH, SOUTH, EAST, WEST]\n";
const PACKAGE_INFO_GOLDEN: &str = "true\n";
const SHORT_CIRCUIT_GOLDEN: &str = "false-result=false,calls=0\ntrue-result=true,calls=1\n";

// ---------------------------------------------------------------------------------------------
// `anonymous-member-base` — the qualified member-base receiver's refusal boundary, and the
// `this$0` + user-field constructor whose presentation the order rule owns.
// ---------------------------------------------------------------------------------------------

/// The two anonymous sites are refused — no inlining is claimed, and the children keep the
/// javac 9+ `requireNonNull` constructor spelling this run does not recover yet.
///
/// **Current presentation**: 5.3 of `present-proved-java-structure` is where this shape is
/// unlocked; the slice that inlines `outer.new Base(…) { … }` must rewrite this anchor (and this
/// fixture's `jarde_refused_body();` facts) in the same commit.
#[test]
fn anonymous_member_base_keeps_the_qualified_receiver_refusal() {
    let snapshot = member_base_snapshot();
    let root = class_source_of(&snapshot, "AnonymousMemberBase");
    // The fixture's subject: a receiver qualified by a member base class stays physical. No
    // anonymous class name appears at either allocation site, and the refusals the run recorded
    // stand in the text where a rule did not prove what the allocation builds.
    assert!(
        !root.text.contains("new AnonymousMemberBase$1(")
            && !root.text.contains("new AnonymousMemberBase$2("),
        "the qualified member-base allocation must stay un-inlined:\n{}",
        root.text
    );
    for (declaration, expected) in [
        (
            "private static AnonymousMemberBase$Outer$Base normal()",
            "AnonymousMemberBase$Outer outer = outer();",
        ),
        (
            "private static AnonymousMemberBase$Outer$Base nullPath()",
            "AnonymousMemberBase$Outer outer = nullOuter();",
        ),
    ] {
        assert_eq!(
            body_statements(&root, declaration),
            vec![expected],
            "the allocation contributes no statement of its own:\n{}",
            root.text
        );
        assert!(
            root.text
                .contains("// the instruction at BCI 4 belongs to no shape this run verified"),
            "the refusal keeps its own statement in the text:\n{}",
            root.text
        );
    }
    for name in ["AnonymousMemberBase$1", "AnonymousMemberBase$2"] {
        let child = class_source_of(&snapshot, name);
        assert_eq!(
            body_statements(
                &child,
                &format!("{name}(AnonymousMemberBase$Outer x0, int value)")
            ),
            vec!["jarde_refused_body();"],
            "`{name}` keeps its constructor refusal:\n{}",
            child.text
        );
        assert!(
            child
                .text
                .contains("// the instruction at BCI 2 belongs to no shape this run verified"),
            "`{name}` states the reason it refused:\n{}",
            child.text
        );
    }
}

/// The `this$0` + user-field constructor, in the order the presentation rule certifies: the
/// `Object` call first, then the synthetic group, then the user field, then the side effect the
/// bytes run after all of them.
#[test]
fn anonymous_member_base_presents_the_member_base_constructor_order() {
    let snapshot = member_base_snapshot();
    let base = class_source_of(&snapshot, "AnonymousMemberBase$Outer$Base");
    assert!(
        base.text
            .contains("final AnonymousMemberBase$Outer this$0;")
            && base.text.contains("final int value;"),
        "the synthetic capture and the user field both stay presented:\n{}",
        base.text
    );
    assert_eq!(
        body_statements(
            &base,
            "AnonymousMemberBase$Outer$Base(AnonymousMemberBase$Outer this$0, int value)"
        ),
        vec![
            "super();",
            "this.this$0 = this$0;",
            "this.value = value;",
            "AnonymousMemberBase.access$000(\"base(\" + value + \")\");",
            "return;",
        ],
        "the member-base constructor's presented order:\n{}",
        base.text
    );
}

/// The frozen classes run, and the recovered text still cannot compile — the fact this fixture's
/// classification records, asserted rather than assumed.
#[test]
#[ignore = "needs a JDK on PATH: it runs the frozen classes and asks `javac` whether the recovered \
            text compiles (see openspec/changes/recover-fixture-behavior-guard-coverage/results/)"]
fn anonymous_member_base_runs_and_its_text_stays_uncompilable() {
    let entries: [(&str, &[u8]); 5] = [
        ("AnonymousMemberBase.class", MEMBER_BASE_ROOT),
        ("AnonymousMemberBase$1.class", MEMBER_BASE_ONE),
        ("AnonymousMemberBase$2.class", MEMBER_BASE_TWO),
        ("AnonymousMemberBase$Outer.class", MEMBER_BASE_OUTER),
        (
            "AnonymousMemberBase$Outer$Base.class",
            MEMBER_BASE_OUTER_BASE,
        ),
    ];
    assert_eq!(
        run_frozen("member-base", &entries, "AnonymousMemberBase"),
        MEMBER_BASE_GOLDEN
    );
    let snapshot = member_base_snapshot();
    let files = [
        ("AnonymousMemberBase.java", "AnonymousMemberBase"),
        ("AnonymousMemberBase$1.java", "AnonymousMemberBase$1"),
        ("AnonymousMemberBase$2.java", "AnonymousMemberBase$2"),
        (
            "AnonymousMemberBase$Outer.java",
            "AnonymousMemberBase$Outer",
        ),
        (
            "AnonymousMemberBase$Outer$Base.java",
            "AnonymousMemberBase$Outer$Base",
        ),
    ]
    .map(|(file, class)| (file, class_source_of(&snapshot, class).text));
    let sources: Vec<(&str, &str)> = files
        .iter()
        .map(|(file, text)| (*file, text.as_str()))
        .collect();
    recompile_must_fail(
        "member-base",
        &sources,
        &["missing return statement", "cannot find symbol"],
    );
}

// ---------------------------------------------------------------------------------------------
// `anonymous-top-level` — ring 2's supertype-return anchor. Its root projection is pinned by
// `tests/anonymous_supertype_return.rs`; this file adds the child's byte order and the run.
// ---------------------------------------------------------------------------------------------

/// The child keeps the physical class text, and its capture write stands **before** the
/// constructor call — the byte order, kept because `Base`'s constructor runs user code and
/// `ctor_order` moves the group past `java/lang/Object.<init>()V` alone.
#[test]
fn anonymous_top_level_pins_the_childs_capture_write_before_the_super_call() {
    let snapshot = open(&[
        (b"AnonymousTopLevel.class", TOP_LEVEL_ROOT),
        (b"AnonymousTopLevel$1.class", TOP_LEVEL_CHILD),
        (b"Base.class", TOP_LEVEL_BASE),
        (b"Renderer.class", TOP_LEVEL_RENDERER),
    ]);
    let child = class_source_of(&snapshot, "AnonymousTopLevel$1");
    assert!(
        child
            .text
            .contains("class AnonymousTopLevel$1 extends Base {")
            && child.text.contains("final java.lang.String val$captured;"),
        "the child keeps its physical declaration and capture field:\n{}",
        child.text
    );
    assert_eq!(
        body_statements(
            &child,
            "AnonymousTopLevel$1(long seed, java.lang.String arg3)"
        ),
        vec!["this.val$captured = arg3;", "super(seed);", "return;"],
        "the capture write must stay where the bytes have it:\n{}",
        child.text
    );
    // The root's projection is the shape ring 2 unlocked; it is re-asserted here only as the
    // statement this fixture's child order sits beside (its own test is the ring-2 file).
    let root = class_source_of(&snapshot, "AnonymousTopLevel");
    assert!(
        root.text.contains("return new Base(choose()) {")
            && !root.text.contains("AnonymousTopLevel$1"),
        "the root projects the allocation at the superclass:\n{}",
        root.text
    );
}

/// The frozen classes run, and the recovered text compiles and runs as the original ran.
#[test]
#[ignore = "needs a JDK on PATH: it runs the frozen classes and recompiles the recovered text \
            under `javac --release 8` (see openspec/changes/recover-fixture-behavior-guard-coverage/results/)"]
fn anonymous_top_level_runs_and_recompiles_to_the_recorded_output() {
    let entries: [(&str, &[u8]); 4] = [
        ("AnonymousTopLevel.class", TOP_LEVEL_ROOT),
        ("AnonymousTopLevel$1.class", TOP_LEVEL_CHILD),
        ("Base.class", TOP_LEVEL_BASE),
        ("Renderer.class", TOP_LEVEL_RENDERER),
    ];
    assert_eq!(
        run_frozen("top-level", &entries, "AnonymousTopLevel"),
        TOP_LEVEL_GOLDEN
    );
    let snapshot = open(&[
        (b"AnonymousTopLevel.class", TOP_LEVEL_ROOT),
        (b"AnonymousTopLevel$1.class", TOP_LEVEL_CHILD),
        (b"Base.class", TOP_LEVEL_BASE),
        (b"Renderer.class", TOP_LEVEL_RENDERER),
    ]);
    let root = class_source_of(&snapshot, "AnonymousTopLevel");
    recompile_and_run(
        "top-level",
        &[
            ("AnonymousTopLevel.java", &root.text),
            ("Base.java", support_source()),
        ],
        "AnonymousTopLevel",
        TOP_LEVEL_GOLDEN,
    );
}

// ---------------------------------------------------------------------------------------------
// `lambda-body-inline` — 5.2's multi-statement lambda body anchor.
// ---------------------------------------------------------------------------------------------

/// The arrow forwards to the renamed helper, the helper carries the three statements in order,
/// and `main` is refused with its own diagnosis.
///
/// **Current presentation**: 5.2 of `present-proved-java-structure` is where the body moves into
/// the arrow; the slice that does it must rewrite this anchor in the same commit.
#[test]
fn lambda_body_inline_pins_the_forwarding_arrow_and_the_refused_main() {
    let snapshot = open(&[
        (b"LambdaBodyInline.class", LAMBDA_ROOT),
        (b"IntAction.class", LAMBDA_SAM),
    ]);
    let root = class_source_of(&snapshot, "LambdaBodyInline");
    assert!(
        root.text
            .contains("return (int p0) -> LambdaBodyInline.lambda$build$0$jarde(captured, p0);")
            && root
                .text
                .contains("// jarde: lambda companion call renamed at invokedynamic@1"),
        "the creation site keeps the forwarding arrow:\n{}",
        root.text
    );
    assert_eq!(
        body_statements(
            &root,
            "private static int lambda$build$0$jarde(int captured, int value)"
        ),
        vec![
            "LambdaBodyInline.events.add((java.lang.Object) (\"start:\" + captured + \":\" + value));",
            "LambdaBodyInline.bodyCalls = LambdaBodyInline.bodyCalls + 1;",
            "LambdaBodyInline.events.add((java.lang.Object) (\"end:\" + captured + \":\" + value));",
            "return captured + value;",
        ],
        "the helper keeps the body's statement order:\n{}",
        root.text
    );
    // `main` is not recovered: its refusal and the reason the run recorded both stay in the text.
    assert!(
        root.text.contains("jarde_refused_body();")
            && root
                .text
                .contains("// the carried argument crosses an independent instruction at BCI 19"),
        "`main` keeps its recorded refusal:\n{}",
        root.text
    );
}

/// The frozen classes run, and the recovered text still cannot compile — the fact this fixture's
/// classification records, asserted rather than assumed.
#[test]
#[ignore = "needs a JDK on PATH: it runs the frozen classes and asks `javac` whether the recovered \
            text compiles (see openspec/changes/recover-fixture-behavior-guard-coverage/results/)"]
fn lambda_body_inline_runs_and_its_text_stays_uncompilable() {
    let entries: [(&str, &[u8]); 2] = [
        ("LambdaBodyInline.class", LAMBDA_ROOT),
        ("IntAction.class", LAMBDA_SAM),
    ];
    assert_eq!(
        run_frozen("lambda-body", &entries, "LambdaBodyInline"),
        LAMBDA_GOLDEN
    );
    let snapshot = open(&[
        (b"LambdaBodyInline.class", LAMBDA_ROOT),
        (b"IntAction.class", LAMBDA_SAM),
    ]);
    let root = class_source_of(&snapshot, "LambdaBodyInline");
    let sam = class_source_of(&snapshot, "IntAction");
    recompile_must_fail(
        "lambda-body",
        &[
            ("IntAction.java", &sam.text),
            ("LambdaBodyInline.java", &root.text),
        ],
        &["cannot find symbol"],
    );
}

// ---------------------------------------------------------------------------------------------
// `enum-arity` — the zero-, one- and four-constant plain enum headers.
// ---------------------------------------------------------------------------------------------

/// The three arities present as plain enum declarations, each with exactly its own constants.
#[test]
fn enum_arity_pins_the_zero_one_and_four_constant_headers() {
    let snapshot = open(&[
        (b"probe/Empty.class", ARITY_EMPTY),
        (b"probe/One.class", ARITY_ONE),
        (b"probe/Four.class", ARITY_FOUR),
        (b"probe/EnumArityRunner.class", ARITY_RUNNER),
    ]);
    for (name, header) in [
        ("probe/Empty", "public enum Empty {\n}"),
        ("probe/One", "public enum One {\n    ONLY;\n}"),
        (
            "probe/Four",
            "public enum Four {\n    NORTH,\n    SOUTH,\n    EAST,\n    WEST;\n}",
        ),
    ] {
        let report = class_source_of(&snapshot, name);
        assert!(
            report.text.contains("package probe;\n"),
            "`{name}` keeps its package line:\n{}",
            report.text
        );
        assert!(
            report.text.contains(header),
            "`{name}` presents its constants as the header:\n{}",
            report.text
        );
    }
    // The runner's three reads: `values()`/`name()`/`ordinal()` on the presented constants.
    let runner = class_source_of(&snapshot, "probe/EnumArityRunner");
    assert!(
        runner.text.contains("probe.Empty.values().length")
            && runner.text.contains("probe.One.ONLY.name()")
            && runner.text.contains("probe.One.ONLY.ordinal()")
            && runner
                .text
                .contains("java.util.Arrays.toString((java.lang.Object[]) probe.Four.values())"),
        "the runner keeps its constant reads:\n{}",
        runner.text
    );
}

/// The frozen classes run, and the recovered texts compile and run as the originals ran.
#[test]
#[ignore = "needs a JDK on PATH: it runs the frozen classes and recompiles the recovered text \
            under `javac --release 8` (see openspec/changes/recover-fixture-behavior-guard-coverage/results/)"]
fn enum_arity_runs_and_recompiles_to_the_recorded_output() {
    let entries: [(&str, &[u8]); 4] = [
        ("probe/Empty.class", ARITY_EMPTY),
        ("probe/One.class", ARITY_ONE),
        ("probe/Four.class", ARITY_FOUR),
        ("probe/EnumArityRunner.class", ARITY_RUNNER),
    ];
    assert_eq!(
        run_frozen("enum-arity", &entries, "probe.EnumArityRunner"),
        ARITY_GOLDEN
    );
    let snapshot = open(&[
        (b"probe/Empty.class", ARITY_EMPTY),
        (b"probe/One.class", ARITY_ONE),
        (b"probe/Four.class", ARITY_FOUR),
        (b"probe/EnumArityRunner.class", ARITY_RUNNER),
    ]);
    let empty = class_source_of(&snapshot, "probe/Empty");
    let one = class_source_of(&snapshot, "probe/One");
    let four = class_source_of(&snapshot, "probe/Four");
    let runner = class_source_of(&snapshot, "probe/EnumArityRunner");
    recompile_and_run(
        "enum-arity",
        &[
            ("probe/Empty.java", &empty.text),
            ("probe/One.java", &one.text),
            ("probe/Four.java", &four.text),
            ("probe/EnumArityRunner.java", &runner.text),
        ],
        "probe.EnumArityRunner",
        ARITY_GOLDEN,
    );
}

// ---------------------------------------------------------------------------------------------
// `package-info-basic` — the package declaration and its package annotation.
// ---------------------------------------------------------------------------------------------

/// `package-info` presents the annotation before the package line — the whole text is the anchor,
/// because the fixture exists to prove the package annotation is not decoration.
#[test]
fn package_info_basic_pins_the_package_annotation_before_the_package_line() {
    let snapshot = open(&[
        (b"p/package-info.class", PACKAGE_INFO),
        (b"p/Check.class", PACKAGE_INFO_CHECK),
    ]);
    let package_info = class_source_of(&snapshot, "p/package-info");
    assert_eq!(package_info.text, "@java.lang.Deprecated\npackage p;\n");
    let check = class_source_of(&snapshot, "p/Check");
    assert!(
        check
            .text
            .contains("java.lang.Class.forName(\"p.package-info\");")
            && check.text.contains(
                "p.Check.class.getPackage().isAnnotationPresent(java.lang.Deprecated.class)"
            ),
        "the check reads the package's annotation at run time:\n{}",
        check.text
    );
}

/// The frozen classes run (the package annotation is visible at run time), and the recovered
/// texts compile and print the same `true`.
#[test]
#[ignore = "needs a JDK on PATH: it runs the frozen classes and recompiles the recovered text \
            under `javac --release 8` (see openspec/changes/recover-fixture-behavior-guard-coverage/results/)"]
fn package_info_basic_runs_and_recompiles_to_the_recorded_output() {
    let entries: [(&str, &[u8]); 2] = [
        ("p/package-info.class", PACKAGE_INFO),
        ("p/Check.class", PACKAGE_INFO_CHECK),
    ];
    assert_eq!(
        run_frozen("package-info", &entries, "p.Check"),
        PACKAGE_INFO_GOLDEN
    );
    let snapshot = open(&[
        (b"p/package-info.class", PACKAGE_INFO),
        (b"p/Check.class", PACKAGE_INFO_CHECK),
    ]);
    let package_info = class_source_of(&snapshot, "p/package-info");
    let check = class_source_of(&snapshot, "p/Check");
    recompile_and_run(
        "package-info",
        &[
            ("p/package-info.java", &package_info.text),
            ("p/Check.java", &check.text),
        ],
        "p.Check",
        PACKAGE_INFO_GOLDEN,
    );
}

// ---------------------------------------------------------------------------------------------
// `short-circuit-left-false` — the left-false short-circuit field write.
// ---------------------------------------------------------------------------------------------

/// `assign` presents one write whose right-hand side is the `&&` — the short-circuit is the
/// operand, not an `if`/`else` — and `rhs` keeps its counter increment.
#[test]
fn short_circuit_left_false_pins_the_single_short_circuit_write() {
    let snapshot = open(&[(b"ShortCircuitFalse.class", SHORT_CIRCUIT)]);
    let report = class_source_of(&snapshot, "ShortCircuitFalse");
    assert_eq!(
        body_statements(&report, "static void assign(boolean left)"),
        vec!["ShortCircuitFalse.result = left && rhs();", "return;"],
        "the left-false write is the short-circuit operand itself:\n{}",
        report.text
    );
    assert_eq!(
        body_statements(&report, "static boolean rhs()"),
        vec![
            "ShortCircuitFalse.calls = ShortCircuitFalse.calls + 1;",
            "return true;",
        ],
        "the right-hand side keeps its side effect:\n{}",
        report.text
    );
}

/// The frozen class runs, and the recovered text compiles and runs as the original ran — the
/// left-false input skips `rhs` (one call after both inputs), the value is the `false` the
/// short-circuit produced.
#[test]
#[ignore = "needs a JDK on PATH: it runs the frozen classes and recompiles the recovered text \
            under `javac --release 8` (see openspec/changes/recover-fixture-behavior-guard-coverage/results/)"]
fn short_circuit_left_false_runs_and_recompiles_to_the_recorded_output() {
    let entries: [(&str, &[u8]); 1] = [("ShortCircuitFalse.class", SHORT_CIRCUIT)];
    assert_eq!(
        run_frozen("short-circuit", &entries, "ShortCircuitFalse"),
        SHORT_CIRCUIT_GOLDEN
    );
    let snapshot = open(&[(b"ShortCircuitFalse.class", SHORT_CIRCUIT)]);
    let report = class_source_of(&snapshot, "ShortCircuitFalse");
    recompile_and_run(
        "short-circuit",
        &[("ShortCircuitFalse.java", &report.text)],
        "ShortCircuitFalse",
        SHORT_CIRCUIT_GOLDEN,
    );
}

// ---------------------------------------------------------------------------------------------
// The library side of every case
// ---------------------------------------------------------------------------------------------

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(entries: &[(&[u8], &[u8])]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(zip_of(entries)), &mut budget())
        .expect("the fixture snapshot opens")
}

/// The five frozen `anonymous-member-base` classes as one snapshot, in the fixture's own flat
/// layout.
fn member_base_snapshot() -> ArtifactSnapshot {
    open(&[
        (b"AnonymousMemberBase.class", MEMBER_BASE_ROOT),
        (b"AnonymousMemberBase$1.class", MEMBER_BASE_ONE),
        (b"AnonymousMemberBase$2.class", MEMBER_BASE_TWO),
        (b"AnonymousMemberBase$Outer.class", MEMBER_BASE_OUTER),
        (
            b"AnonymousMemberBase$Outer$Base.class",
            MEMBER_BASE_OUTER_BASE,
        ),
    ])
}

fn performed<T>(outcome: OperationOutcome<T>) -> T {
    match outcome {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "expected one bound class, got {} candidate(s) and no execution",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "expected one bound class, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    performed(
        Engine::new()
            .class_source(
                slice::from_ref(snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal(name),
                    },
                    environment: EnvironmentRequest {
                        snapshot: snapshot.id().clone(),
                        scope: PhysicalScope::SnapshotAll,
                        policy: EnvironmentPolicy::PlainJar,
                        profile: RuntimeProfile {
                            java_release: 8,
                            multi_release: MultiReleasePolicy::Disabled,
                            layout: LayoutMode::Generic,
                        },
                        loader: LoaderId("app".to_owned()),
                    },
                },
                &mut budget(),
            )
            .expect("a legal class-source request is answered"),
    )
}

/// The frozen source's support part: everything before the root class declaration (the interface
/// and the abstract superclass), which the recompile set compiles beside the recovered root.
fn support_source() -> &'static str {
    let index = TOP_LEVEL_SOURCE
        .find("public final class")
        .expect("the frozen source declares the root class");
    &TOP_LEVEL_SOURCE[..index]
}

/// The body's own statement lines, in the order the report presents them, each trimmed. The
/// envelope's `// jarde:`/`// @method`/`// @bytecode` lines are records about the body rather
/// than statements of it, and the body ends at the first line that is a lone closing brace.
fn body_statements(report: &ClassSourceReport, declaration: &str) -> Vec<String> {
    let mut lines = report
        .text
        .lines()
        .skip_while(|line| !line.trim_start().starts_with(declaration));
    lines.next();
    lines
        .map(str::trim)
        .take_while(|line| *line != "}")
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .map(str::to_owned)
        .collect()
}

/// A stored-only archive, built with the repository's own `rawzip` dev-dependency.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer
                .write_all(data)
                .expect("the fixture entry is written");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

// ---------------------------------------------------------------------------------------------
// The JDK side of the behavior legs
// ---------------------------------------------------------------------------------------------

/// Runs one class of the frozen fixture classes under `java -Xverify:all` — the run the
/// fixture's own `run.sh` performs — and answers its standard output.
fn run_frozen(label: &str, entries: &[(&str, &[u8])], class: &str) -> String {
    let scratch = Scratch::new(label);
    for (name, bytes) in entries {
        scratch.write(name, bytes);
    }
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(scratch.path())
        .arg(class)
        .output()
        .expect("JDK java is available for the frozen fixture run");
    assert!(
        run.status.success(),
        "the frozen `{class}` did not verify:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the fixture prints text")
}

/// Compiles the recovered texts together under `javac --release 8`, runs the family's main class
/// under `-Xverify:all`, and asserts the run prints the original's own output.
fn recompile_and_run(label: &str, files: &[(&str, &str)], main: &str, baseline: &str) {
    let scratch = Scratch::new(label);
    for (name, text) in files {
        scratch.write(name, text.as_bytes());
    }
    let sources: Vec<PathBuf> = files
        .iter()
        .map(|(name, _)| scratch.path().join(name))
        .collect();
    let out = scratch.path().join("out");
    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options"])
        .arg("-d")
        .arg(&out)
        .args(&sources)
        .output()
        .expect("JDK javac is available for the recovered family");
    assert!(
        compile.status.success(),
        "javac rejected the recovered {label} family:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&out)
        .arg(main)
        .output()
        .expect("JDK java is available for the recompiled run");
    assert!(
        run.status.success(),
        "the recompiled {label} family did not verify:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        baseline,
        "a text that compiles must run as the original ran — never a silent change"
    );
}

/// Asserts the recovered texts do **not** compile, with the diagnosis the evidence records: the
/// current-state fact these fixtures' classification states, never a claim that their behavior
/// is verified. `-J-Duser.language` fixes the diagnostic language so the assertion reads the
/// same on every machine.
fn recompile_must_fail(label: &str, files: &[(&str, &str)], diagnoses: &[&str]) -> String {
    let scratch = Scratch::new(label);
    for (name, text) in files {
        scratch.write(name, text.as_bytes());
    }
    let sources: Vec<PathBuf> = files
        .iter()
        .map(|(name, _)| scratch.path().join(name))
        .collect();
    let out = scratch.path().join("out");
    let compile = Command::new("javac")
        .args([
            "-J-Duser.language=en",
            "-J-Duser.country=US",
            "--release",
            "8",
            "-Xlint:-options",
        ])
        .arg("-d")
        .arg(&out)
        .args(&sources)
        .output()
        .expect("JDK javac is available for the recovered family");
    let stderr = String::from_utf8_lossy(&compile.stderr).into_owned();
    assert!(
        !compile.status.success(),
        "the recovered {label} text compiled: the fixture's classification (and this test's \
         uncompilable fact) must be updated, not the run silenced\n{stderr}"
    );
    for diagnosis in diagnoses {
        assert!(
            stderr.contains(diagnosis),
            "the recovered {label} text failed for a different reason than the recorded \
             `{diagnosis}`:\n{stderr}"
        );
    }
    stderr
}

/// One throwaway directory per compile-and-run case, removed with the test.
struct Scratch(PathBuf);

static NEXT_SCRATCH: AtomicU64 = AtomicU64::new(0);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-fixture-guards-{label}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("a private fixture-guard directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    /// Writes one file below the scratch directory, creating its parents: a fixture's nested
    /// layout (`v8/probe/`'s classes carry the `probe` package) is reproduced by the relative
    /// name, so the classpath root is the directory that holds the package.
    fn write(&self, name: &str, bytes: &[u8]) {
        let path = self.0.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the fixture file's directory is created");
        }
        fs::write(path, bytes).expect("the fixture file is written");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
