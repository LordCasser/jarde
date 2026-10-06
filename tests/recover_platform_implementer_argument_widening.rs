//! `recover-charsequence-argument-widening`, `recover-comparable-argument-widening` and
//! `recover-enum-argument-widening` in one frozen target: the platform implementer tables the
//! `java.util` collection tree and the `java.lang` Throwable channel do not state.
//!
//! The three changes were dispatched as one mechanism, so this target freezes their patrol anchors
//! together, each under the change's own fixture directory, on **both** compiler legs (javac 23.0.1
//! `--release 8` and real javac 8, Corretto 1.8.0_432, the same sources both times):
//!
//! * `recover-charsequence-argument-widening`'s `CS`: `String.join("-", xs)` writes both positions
//!   (`(java.lang.CharSequence) "-"` and `(java.lang.CharSequence[]) arg0`), `Appendable.append`
//!   writes `(java.lang.CharSequence) "x"`, `Collectors.joining(",")` writes
//!   `(java.lang.CharSequence) ","` (the patrol's fifth site, a `java.util.stream` method), and the
//!   multi-bound `both("a", "b")` writes `(java.io.Serializable) "a"`;
//! * `recover-comparable-argument-widening`'s `CO`: `max("a", "b")` and the boxed `max(1, 2)` write
//!   `(java.lang.Comparable)`, and the `RG$IntNode.cmp` shape (`val.compareTo(o.val)`) keeps the
//!   render the patrol recorded as healthy;
//! * `recover-enum-argument-widening`'s `EN`: `EnumSet.of(Flag.A, Flag.C)` writes its constants
//!   under the `java.lang.Enum` parameter the snapshot proof states, `retainAll(fs)` writes
//!   `(java.util.Collection) arg0` from the enum family's `EnumSet` rows, and the `Objects.*`
//!   members are the zero-drift control the change pins byte for byte.
//!
//! Every change's **negative** stands beside its anchor: `CSX` keeps `StringBuilder` at a
//! `Serializable` slot and `javax.swing.text.Segment` at a `CharSequence` slot refused (the closed
//! nine and four rows the specs pin), `COX` keeps `java.math.BigInteger` at a `Comparable` slot
//! refused, and `ENX` keeps a platform enum (`java.lang.Thread$State`, in no snapshot header and in
//! no table) refused.
//!
//! The ignored replay strips the presentations the way the patrols' own stripped sources were made
//! (comment lines dropped), compiles each anchor's family with the installed `javac --release 8`
//! and, when a real javac 8 is present, with that one too, runs both under `-Xverify:all` and
//! compares every answer with the fixture's own class files.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

// -------------------------------------------------------------------------------------------
// The fixtures: the patrols' own shapes, compiled by both javac legs.
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names and the class files it produced.
struct Leg {
    label: &'static str,
    files: &'static [(&'static str, &'static [u8])],
}

impl Leg {
    /// The class files of one fixture family, in container order: the class itself and its own
    /// companions, then any separately declared class the presentation reads.
    fn family(&self, class: &str, extra: &[&str]) -> Vec<(String, &'static [u8])> {
        let own = format!("{class}.class");
        let nested = format!("{class}$");
        let mut files: Vec<(String, &'static [u8])> = Vec::new();
        for (name, bytes) in self.files {
            let name: &str = name;
            if name == own || name.starts_with(&nested) {
                files.push((name.to_owned(), *bytes));
            }
        }
        assert_eq!(
            files.first().map(|(name, _)| name.as_str()),
            Some(own.as_str()),
            "the fixture family of `{class}` is committed"
        );
        for (name, bytes) in self.files {
            let name: &str = name;
            if extra.iter().any(|other| name == format!("{other}.class")) {
                files.push((name.to_owned(), *bytes));
            }
        }
        files
    }

    /// One fixture family's container.
    fn fixture(&self, class: &str, extra: &[&str]) -> Vec<u8> {
        let own = format!("{class}.class");
        let nested = format!("{class}$");
        let mut entries: Vec<(&[u8], &[u8])> = Vec::new();
        for (name, bytes) in self.files {
            let name: &str = name;
            if name == own || name.starts_with(&nested) {
                entries.push((name.as_bytes(), *bytes));
            }
        }
        assert_eq!(
            entries.first().map(|(name, _)| *name),
            Some(own.as_bytes()),
            "the fixture family of `{class}` is committed"
        );
        for (name, bytes) in self.files {
            let name: &str = name;
            if extra.iter().any(|other| name == format!("{other}.class")) {
                entries.push((name.as_bytes(), *bytes));
            }
        }
        zip_of(&entries)
    }
}

/// javac 23.0.1, `javac --release 8 -Xlint:-options -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "CS.class",
        include_bytes!("fixtures/recover-charsequence-argument-widening/v8/CS.class"),
    ),
    (
        "CSX.class",
        include_bytes!("fixtures/recover-charsequence-argument-widening/v8/CSX.class"),
    ),
    (
        "CO.class",
        include_bytes!("fixtures/recover-comparable-argument-widening/v8/CO.class"),
    ),
    (
        "CO$IntNode.class",
        include_bytes!("fixtures/recover-comparable-argument-widening/v8/CO$IntNode.class"),
    ),
    (
        "CO$Node.class",
        include_bytes!("fixtures/recover-comparable-argument-widening/v8/CO$Node.class"),
    ),
    (
        "COX.class",
        include_bytes!("fixtures/recover-comparable-argument-widening/v8/COX.class"),
    ),
    (
        "EN.class",
        include_bytes!("fixtures/recover-enum-argument-widening/v8/EN.class"),
    ),
    (
        "ENN.class",
        include_bytes!("fixtures/recover-enum-argument-widening/v8/ENN.class"),
    ),
    (
        "ENN$Flag.class",
        include_bytes!("fixtures/recover-enum-argument-widening/v8/ENN$Flag.class"),
    ),
    (
        "ENT.class",
        include_bytes!("fixtures/recover-enum-argument-widening/v8/ENT.class"),
    ),
    (
        "ENX.class",
        include_bytes!("fixtures/recover-enum-argument-widening/v8/ENX.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "CS.class",
        include_bytes!("fixtures/recover-charsequence-argument-widening/v8-javac8/CS.class"),
    ),
    (
        "CSX.class",
        include_bytes!("fixtures/recover-charsequence-argument-widening/v8-javac8/CSX.class"),
    ),
    (
        "CO.class",
        include_bytes!("fixtures/recover-comparable-argument-widening/v8-javac8/CO.class"),
    ),
    (
        "CO$IntNode.class",
        include_bytes!("fixtures/recover-comparable-argument-widening/v8-javac8/CO$IntNode.class"),
    ),
    (
        "CO$Node.class",
        include_bytes!("fixtures/recover-comparable-argument-widening/v8-javac8/CO$Node.class"),
    ),
    (
        "COX.class",
        include_bytes!("fixtures/recover-comparable-argument-widening/v8-javac8/COX.class"),
    ),
    (
        "EN.class",
        include_bytes!("fixtures/recover-enum-argument-widening/v8-javac8/EN.class"),
    ),
    (
        "ENN.class",
        include_bytes!("fixtures/recover-enum-argument-widening/v8-javac8/ENN.class"),
    ),
    (
        "ENN$Flag.class",
        include_bytes!("fixtures/recover-enum-argument-widening/v8-javac8/ENN$Flag.class"),
    ),
    (
        "ENT.class",
        include_bytes!("fixtures/recover-enum-argument-widening/v8-javac8/ENT.class"),
    ),
    (
        "ENX.class",
        include_bytes!("fixtures/recover-enum-argument-widening/v8-javac8/ENX.class"),
    ),
];

const LEGS: &[Leg] = &[
    Leg {
        label: "javac 23.0.1 --release 8",
        files: V8_FILES,
    },
    Leg {
        label: "Corretto 1.8.0_432 (real javac 8)",
        files: V8_JAVAC8_FILES,
    },
];

// -------------------------------------------------------------------------------------------
// The class-source surface.
// -------------------------------------------------------------------------------------------

/// The ordinary request plus the one optional category the member fold reads: the presented texts
/// below are the ones this layer writes for a fold that can state its call-site anchors, the same
/// request the platform-interface family's own target pins.
fn evidence() -> RecoveryEvidenceRequest {
    RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap)
}

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a ZIP")
}

/// One class-source presentation over the snapshot's own root container.
fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let request = ClassSourceRequest {
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
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &evidence(),
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition: {other:?}"),
    }
}

/// One member's own record in the assembled source.
fn method_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's text, for the assertions that compare a whole presentation.
fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &method_of(report, name).text
}

/// The one ZIP the class-source request reads, built from the committed class files.
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

// -------------------------------------------------------------------------------------------
// The presented texts, pinned whole (the p3 surface tests' own convention).
// -------------------------------------------------------------------------------------------

/// `CS.join`: the charsequence change's main anchor — `String.join`'s scalar position and its array
/// position in one call.
const CS_JOIN: &str = "    static java.lang.String join(java.lang.String[] arg0) {\n        // @method join([Ljava/lang/String;)Ljava/lang/String;\n        // @declaration a static method of `CS`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return java.lang.String.join((java.lang.CharSequence) \"-\", (java.lang.CharSequence[]) arg0);\n    }\n";

/// `CS.appender`: the `java.lang.Appendable` slot.
const CS_APPENDER: &str = "    static void appender(java.lang.Appendable arg0) throws java.io.IOException {\n        // @method appender(Ljava/lang/Appendable;)V\n        // @declaration a static method of `CS`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        arg0.append((java.lang.CharSequence) \"x\");\n        return;\n    }\n";

/// `CS.joining`: the patrol's fifth site — `Collectors.joining`'s delimiter, a `java.util.stream`
/// method the java.util table never stated.
const CS_JOINING: &str = "    // jarde: generic Signature projection refused for `joining(Ljava/util/List;)Ljava/lang/String;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types\n    static java.lang.String joining(java.util.List arg0) {\n        // @method joining(Ljava/util/List;)Ljava/lang/String;\n        // @declaration a static method of `CS`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return (java.lang.String) arg0.stream().collect((java.util.stream.Collector) java.util.stream.Collectors.joining((java.lang.CharSequence) \",\"));\n    }\n";

/// `CS.useBoth`: the Serializable change's anchor — the multi-bound call site's erased
/// `java.io.Serializable` parameter.
const CS_USEBOTH: &str = "    static java.lang.String useBoth() {\n        // @method useBoth()Ljava/lang/String;\n        // @declaration a static method of `CS`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return (java.lang.String) both((java.io.Serializable) \"a\", (java.io.Serializable) \"b\");\n    }\n";

/// `CS.same`: the same-name control — a `String` at a `String` slot keeps its unchanged render.
const CS_SAME: &str = "    static java.lang.String same(java.lang.String arg0) {\n        // @method same(Ljava/lang/String;)Ljava/lang/String;\n        // @declaration a static method of `CS`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0;\n    }\n";

/// `CS.main`: every anchor called, so the replay's answer covers all of them.
const CS_MAIN: &str = "    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `CS`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.io.PrintStream saved0 = java.lang.System.out;\n        java.lang.StringBuilder saved1 = new java.lang.StringBuilder();\n        java.lang.StringBuilder saved2 = saved1.append((java.lang.String) join(new java.lang.String[]{\"p\", \"q\"})).append(\"/\");\n        saved0.println((java.lang.String) saved2.append((java.lang.String) joining((java.util.List) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{\"a\", \"b\", \"c\"}))).append(\"/\").append((java.lang.String) useBoth()).append(\"/\").append((java.lang.String) same(\"s\")).toString());\n        return;\n    }\n";

/// `CSX.sealBuilder`: `StringBuilder` implements `Serializable` in fact, but the change pins the
/// java.lang nine-row set the spec records — a row the spec does not state keeps its refusal.
const CSX_SEALBUILDER: &str = "    static java.io.Serializable sealBuilder(java.lang.StringBuilder arg0) {\n        // jarde: not recovered: the recovery run for `sealBuilder(Ljava/lang/StringBuilder;)Ljava/io/Serializable;` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method sealBuilder(Ljava/lang/StringBuilder;)Ljava/io/Serializable;\n        // @declaration a static method of `CSX`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 4 1 0\n        // the parameter 0 of the invocation at BCI 1 is declared `java.io.Serializable` presents `java.lang.StringBuilder` but the invocation requires `java.io.Serializable` and this layer has no safe reference conversion evidence\n    }\n";

/// `CSX.viaSegment`: the release-8 CharSequence javadoc also lists `javax.swing.text.Segment`; the
/// closed four rows do not, so it keeps the refusal rather than widening from memory.
const CSX_VIASEGMENT: &str = "    static java.lang.CharSequence viaSegment(javax.swing.text.Segment arg0) {\n        // jarde: not recovered: the recovery run for `viaSegment(Ljavax/swing/text/Segment;)Ljava/lang/CharSequence;` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method viaSegment(Ljavax/swing/text/Segment;)Ljava/lang/CharSequence;\n        // @declaration a static method of `CSX`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 4 1 0\n        // the parameter 0 of the invocation at BCI 1 is declared `java.lang.CharSequence` presents `javax.swing.text.Segment` but the invocation requires `java.lang.CharSequence` and this layer has no safe reference conversion evidence\n    }\n";

/// `CO.callGen`: the recursive-generics patrol's `max("a", "b")`.
const CO_CALLGEN: &str = "    static java.lang.String callGen() {\n        // @method callGen()Ljava/lang/String;\n        // @declaration a static method of `CO`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return (java.lang.String) max((java.lang.Comparable) \"a\", (java.lang.Comparable) \"b\");\n    }\n";

/// `CO.callGen2`: the same call with the boxing step (`Integer.valueOf`).
const CO_CALLGEN2: &str = "    static java.lang.Integer callGen2() {\n        // @method callGen2()Ljava/lang/Integer;\n        // @declaration a static method of `CO`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return (java.lang.Integer) max((java.lang.Comparable) java.lang.Integer.valueOf(1), (java.lang.Comparable) java.lang.Integer.valueOf(2));\n    }\n";

/// `CO.same`: the same call over two `String` parameters — the widening does not depend on a literal.
const CO_SAME: &str = "    static java.lang.String same(java.lang.String arg0, java.lang.String arg1) {\n        // @method same(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;\n        // @declaration a static method of `CO`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return (java.lang.String) max((java.lang.Comparable) arg0, (java.lang.Comparable) arg1);\n    }\n";

/// `CO.main`: the patrol's `RG$IntNode.cmp` shape (the healthy face) beside both call points.
const CO_MAIN: &str = "    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `CO`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(\"\").append(new CO$IntNode((java.lang.Integer) java.lang.Integer.valueOf(2)).cmp((CO$Node) new CO$IntNode((java.lang.Integer) java.lang.Integer.valueOf(1)))).append(\"/\").append((java.lang.String) callGen()).append(\"/\").append((java.lang.Object) callGen2()).append(\"/\").append((java.lang.String) same(\"a\", \"b\")).toString());\n        return;\n    }\n";

/// `COX.big`: `java.math.BigInteger` implements `Comparable` in fact, and the change's table is the
/// `java.lang` set — the spec's own non-goal, kept refused.
const COX_BIG: &str = "    static java.lang.Comparable big(java.math.BigInteger arg0, java.math.BigInteger arg1) {\n        // jarde: not recovered: the recovery run for `big(Ljava/math/BigInteger;Ljava/math/BigInteger;)Ljava/lang/Comparable;` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method big(Ljava/math/BigInteger;Ljava/math/BigInteger;)Ljava/lang/Comparable;\n        // @declaration a static method of `COX`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 5 2 0 1\n        // the parameter 0 of the invocation at BCI 2 is declared `java.lang.Comparable` presents `java.math.BigInteger` but the invocation requires `java.lang.Comparable` and this layer has no safe reference conversion evidence\n    }\n";

/// `EN.flags`: the enum change's main anchor — the patrol's `OB.flags` shape over a top-level enum.
const EN_FLAGS: &str = "    // jarde: generic Signature projection refused for `flags(Ljava/util/EnumSet;)Ljava/lang/String;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types\n    static java.lang.String flags(java.util.EnumSet arg0) {\n        // @method flags(Ljava/util/EnumSet;)Ljava/lang/String;\n        // @declaration a static method of `EN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.util.EnumSet local1 = java.util.EnumSet.of((java.lang.Enum) ENT.A, (java.lang.Enum) ENT.C);\n        local1.retainAll((java.util.Collection) arg0);\n        return local1.contains((java.lang.Object) ENT.A) ? \"hasA\" : \"no\";\n    }\n";

/// `EN.three`: the `EnumSet.of(E, E...)` shape with three constants.
const EN_THREE: &str = "    // jarde: generic Signature projection refused for `three(Ljava/util/EnumSet;)Ljava/lang/String;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types\n    static java.lang.String three(java.util.EnumSet arg0) {\n        // @method three(Ljava/util/EnumSet;)Ljava/lang/String;\n        // @declaration a static method of `EN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.util.EnumSet local1 = java.util.EnumSet.of((java.lang.Enum) ENT.A, (java.lang.Enum) ENT.B, (java.lang.Enum) ENT.C);\n        local1.retainAll((java.util.Collection) arg0);\n        return local1.contains((java.lang.Object) ENT.B) ? \"hasB\" : \"no\";\n    }\n";

/// `EN.same`: a value already declared `java.lang.Enum` (the type variable's erasure) — the
/// call site introduces no cast, as the spec's third scenario states.
const EN_SAME: &str = "    // jarde: generic Signature projection refused for `same(Ljava/lang/Enum;Ljava/lang/Enum;)Ljava/util/EnumSet;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return\n    static java.util.EnumSet same(java.lang.Enum arg0, java.lang.Enum arg1) {\n        // @method same(Ljava/lang/Enum;Ljava/lang/Enum;)Ljava/util/EnumSet;\n        // @declaration a static method of `EN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return java.util.EnumSet.of(arg0, arg1);\n    }\n";

/// `EN.eq`: the patrol's zero-drift control, byte for byte the render the objects-enumset patrol
/// recorded.
const EN_EQ: &str = "    static boolean eq(java.lang.String arg0, java.lang.String arg1) {\n        // @method eq(Ljava/lang/String;Ljava/lang/String;)Z\n        // @declaration a static method of `EN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return java.util.Objects.equals((java.lang.Object) arg0, (java.lang.Object) arg1);\n    }\n";

/// `EN.h`: the second zero-drift control.
const EN_H: &str = "    static int h(java.lang.String arg0) {\n        // @method h(Ljava/lang/String;)I\n        // @declaration a static method of `EN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return java.util.Objects.hashCode((java.lang.Object) arg0);\n    }\n";

/// `EN.str`: the third zero-drift control.
const EN_STR: &str = "    static java.lang.String str(java.lang.Object arg0) {\n        // @method str(Ljava/lang/Object;)Ljava/lang/String;\n        // @declaration a static method of `EN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return java.util.Objects.toString(arg0, \"dflt\");\n    }\n";

/// `EN.main`: the anchors called, so the replay's answer is the patrol's `hasA`-family output.
const EN_MAIN: &str = "    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `EN`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) flags((java.util.EnumSet) java.util.EnumSet.of((java.lang.Enum) ENT.A, (java.lang.Enum) ENT.B))).append(\"/\").append((java.lang.String) three((java.util.EnumSet) java.util.EnumSet.noneOf(ENT.class))).append(\"/\").append(same((java.lang.Enum) ENT.A, (java.lang.Enum) ENT.B).contains((java.lang.Object) ENT.A)).append(\"/\").append(eq((java.lang.String) null, (java.lang.String) null)).append(\"/\").append(h((java.lang.String) null)).append(\"/\").append((java.lang.String) str((java.lang.Object) null)).toString());\n        return;\n    }\n";

/// `ENN.flags`: the patrol's own `$`-nested enum shape (`OB$Flag`), rendered pool-spelled.
const ENN_FLAGS: &str = "    // jarde: generic Signature projection refused for `flags(Ljava/util/EnumSet;)Ljava/lang/String;`: unsupported (generic_source_shape_unproved): class name has no unambiguous Java source spelling\n    static java.lang.String flags(java.util.EnumSet arg0) {\n        // @method flags(Ljava/util/EnumSet;)Ljava/lang/String;\n        // @declaration a static method of `ENN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.util.EnumSet local1 = java.util.EnumSet.of((java.lang.Enum) ENN$Flag.A, (java.lang.Enum) ENN$Flag.C);\n        local1.retainAll((java.util.Collection) arg0);\n        return local1.contains((java.lang.Object) ENN$Flag.A) ? \"hasA\" : \"no\";\n    }\n";

/// `ENX.platform`: a platform enum (`java.lang.Thread$State`) is in no snapshot header and in no
/// table, so the `java.lang.Enum` position keeps its refusal — the tables state no platform enum
/// from memory, and a class-path proof is another change's ground.
const ENX_PLATFORM: &str = "    // jarde: generic Signature projection refused for `platform()Ljava/util/EnumSet;`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof\n    static java.util.EnumSet platform() {\n        // jarde: not recovered: the recovery run for `platform()Ljava/util/EnumSet;` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method platform()Ljava/util/EnumSet;\n        // @declaration a static method of `ENX`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 9 6 0 3\n        // the parameter 0 of the invocation at BCI 6 is declared `java.lang.Enum` presents `java.lang.Thread$State` but the invocation requires `java.lang.Enum` and this layer has no safe reference conversion evidence\n    }\n";

/// The refusal text every negative below states: the one reference-conversion sentence this layer
/// writes, unchanged by this change.
const REFUSAL: &str = "but the invocation requires";

// -------------------------------------------------------------------------------------------
// The anchors.
// -------------------------------------------------------------------------------------------

/// The charsequence change: `String.join`'s two positions, the `Appendable` slot, the
/// `Collectors.joining` site and the Serializable multi-bound call site, with no refusal left in
/// the class.
#[test]
fn the_charsequence_and_serializable_positions_are_presented() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CS", &[]));
        let report = class_source_of(&snapshot, "CS");
        assert_eq!(text_of(&report, "join"), CS_JOIN, "{}", leg.label);
        assert_eq!(text_of(&report, "appender"), CS_APPENDER, "{}", leg.label);
        assert_eq!(text_of(&report, "joining"), CS_JOINING, "{}", leg.label);
        assert_eq!(text_of(&report, "useBoth"), CS_USEBOTH, "{}", leg.label);
        assert_eq!(text_of(&report, "same"), CS_SAME, "{}", leg.label);
        assert_eq!(text_of(&report, "main"), CS_MAIN, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`{}` keeps no reference-conversion refusal:\n{}",
            leg.label,
            report.text
        );
    }
}

/// The charsequence change's negatives: the closed rows are the spec's own, and a pair no row
/// states keeps the refusal verbatim.
#[test]
fn the_types_outside_the_closed_rows_still_refuse() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CSX", &[]));
        let report = class_source_of(&snapshot, "CSX");
        assert_eq!(
            text_of(&report, "sealBuilder"),
            CSX_SEALBUILDER,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "viaSegment"),
            CSX_VIASEGMENT,
            "{}",
            leg.label
        );
        assert_eq!(
            report.text.matches(REFUSAL).count(),
            2,
            "only the two out-of-table pairs stay refused on `{}`:\n{}",
            leg.label,
            report.text
        );
    }
}

/// The comparable change: both call points and the `cmp` shape, with no refusal left.
#[test]
fn the_comparable_positions_are_presented() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CO", &[]));
        let report = class_source_of(&snapshot, "CO");
        assert_eq!(text_of(&report, "callGen"), CO_CALLGEN, "{}", leg.label);
        assert_eq!(text_of(&report, "callGen2"), CO_CALLGEN2, "{}", leg.label);
        assert_eq!(text_of(&report, "same"), CO_SAME, "{}", leg.label);
        assert_eq!(text_of(&report, "main"), CO_MAIN, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`{}` keeps no reference-conversion refusal:\n{}",
            leg.label,
            report.text
        );
    }
}

/// The comparable change's negative: the `java.math` implementer the spec kept out of the table.
#[test]
fn the_out_of_package_comparable_implementer_still_refuses() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("COX", &[]));
        let report = class_source_of(&snapshot, "COX");
        assert_eq!(text_of(&report, "big"), COX_BIG, "{}", leg.label);
        assert_eq!(
            report.text.matches(REFUSAL).count(),
            1,
            "only the out-of-table pair stays refused on `{}`:\n{}",
            leg.label,
            report.text
        );
    }
}

/// The enum change: the `EnumSet` positions over a top-level enum, the varargs shape, the
/// `Enum`-typed argument and the patrol's zero-drift controls.
#[test]
fn the_enum_positions_are_presented() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("EN", &["ENT"]));
        let report = class_source_of(&snapshot, "EN");
        assert_eq!(text_of(&report, "flags"), EN_FLAGS, "{}", leg.label);
        assert_eq!(text_of(&report, "three"), EN_THREE, "{}", leg.label);
        assert_eq!(text_of(&report, "same"), EN_SAME, "{}", leg.label);
        assert_eq!(text_of(&report, "eq"), EN_EQ, "{}", leg.label);
        assert_eq!(text_of(&report, "h"), EN_H, "{}", leg.label);
        assert_eq!(text_of(&report, "str"), EN_STR, "{}", leg.label);
        assert_eq!(text_of(&report, "main"), EN_MAIN, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`{}` keeps no reference-conversion refusal:\n{}",
            leg.label,
            report.text
        );
    }
}

/// The patrol's own `$`-nested enum shape: the anchor the objects-enumset patrol recorded, over the
/// same pool-spelled nested type.
#[test]
fn the_patrol_nested_enum_shape_presents_its_call() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("ENN", &[]));
        let report = class_source_of(&snapshot, "ENN");
        assert_eq!(text_of(&report, "flags"), ENN_FLAGS, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`{}` keeps no reference-conversion refusal:\n{}",
            leg.label,
            report.text
        );
    }
}

/// The enum change's negative: a platform enum stays refused.
#[test]
fn the_platform_enum_still_refuses() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("ENX", &[]));
        let report = class_source_of(&snapshot, "ENX");
        assert_eq!(text_of(&report, "platform"), ENX_PLATFORM, "{}", leg.label);
        assert_eq!(
            report.text.matches(REFUSAL).count(),
            1,
            "only the platform enum stays refused on `{}`:\n{}",
            leg.label,
            report.text
        );
    }
}

// -------------------------------------------------------------------------------------------
// The replay: the presented text, stripped, compiled and run (needs a JDK on PATH).
// -------------------------------------------------------------------------------------------

static UNIQUE: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-platform-implementer-widening-{label}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the replay directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// One class text with every `//` comment line dropped — the strip the patrols' own stripped
/// sources were made by.
fn comment_lines_dropped(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::to_owned)
        .collect()
}

/// One anchor's replay: which container to open, which units are compiled **from their own
/// presentation**, which support units come from the fixture's own declaration, and what the
/// fixture's own class files answer.
///
/// `ENN` — the patrol's `$`-nested enum shape — is pinned as a text anchor but not replayed: the
/// nested-enum member projection that publishes the enum's own declaration into the class text is
/// **leg-dependent** on this repository today (the javac 23 leg publishes `enum Flag` and re-spells
/// the references, the real javac 8 leg leaves them pool-spelled with no declaration), so its own
/// unit is not a uniform replay input. That difference is a pre-existing fold behaviour this change
/// does not touch; `EN` carries the enum change's behaviour replay.
struct Replay {
    class: &'static str,
    extra: &'static [&'static str],
    presented: &'static [&'static str],
    support: &'static [(&'static str, &'static str)],
    answer: &'static str,
}

/// An enum class's own presentation is not a legal source unit (it declares the
/// `super(name, ordinal)` call no source may write), so the replay writes the enum support unit
/// from the fixture's own declaration and compiles the presentation for every unit the change is
/// about. The fixture's class file was produced from exactly this declaration.
const REPLAYS: &[Replay] = &[
    Replay {
        class: "CS",
        extra: &[],
        presented: &["CS"],
        support: &[],
        answer: "p-q/a,b,c/b/s\n",
    },
    Replay {
        class: "CO",
        extra: &[],
        presented: &["CO", "CO$Node", "CO$IntNode"],
        support: &[],
        answer: "1/b/2/b\n",
    },
    Replay {
        class: "EN",
        extra: &["ENT"],
        presented: &["EN"],
        support: &[("ENT", "public enum ENT { A, B, C }\n")],
        answer: "hasA/no/true/true/0/dflt\n",
    },
];

/// The units of one replay: every presented unit stripped, then the fixture's own support units.
fn replay_sources(leg: &Leg, replay: &Replay) -> Vec<(String, String)> {
    let snapshot = open(&leg.fixture(replay.class, replay.extra));
    let mut sources: Vec<(String, String)> = Vec::new();
    for name in replay.presented {
        let text = class_source_of(&snapshot, name).text;
        sources.push((
            format!("{name}.java"),
            comment_lines_dropped(&text).join("\n") + "\n",
        ));
    }
    for (name, text) in replay.support {
        sources.push((format!("{name}.java"), (*text).to_owned()));
    }
    sources
}

/// The fixture's own class files, written where a JVM can load them.
fn original_classes(leg: &Leg, replay: &Replay, directory: &Path) {
    for (name, bytes) in leg.family(replay.class, replay.extra) {
        fs::write(directory.join(name), bytes).expect("the fixture class is written");
    }
}

/// The real javac 8 the frozen `v8-javac8` leg was compiled by, when this machine holds it.
fn javac8() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("JARDE_JAVAC8") {
        return Some(PathBuf::from(path));
    }
    let default = PathBuf::from(
        "/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac",
    );
    default.is_file().then_some(default)
}

/// Compile one replay's stripped units: the installed javac under `--release 8`, or a real javac 8
/// (whose default target is Java 8, and which has no `--release` flag).
fn compile_with(
    javac: &Path,
    release_8: bool,
    directory: &Path,
    sources: &[(String, String)],
) -> Vec<PathBuf> {
    let paths: Vec<PathBuf> = sources
        .iter()
        .map(|(file, text)| {
            let path = directory.join(file);
            fs::write(&path, text).expect("the stripped unit is written");
            path
        })
        .collect();
    let mut command = Command::new(javac);
    if release_8 {
        command.args(["--release", "8"]);
    }
    let output = command
        .arg("-Xlint:-options")
        .arg("-d")
        .arg(directory)
        .args(&paths)
        .output()
        .expect("the named javac runs");
    assert!(
        output.status.success(),
        "javac rejected the presented text:\n{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        paths
            .iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .collect::<Vec<_>>()
            .join("\n")
    );
    paths
}

fn run(classpath: &Path, main_class: &str) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(classpath)
        .arg(main_class)
        .output()
        .expect("the installed JVM runs");
    assert!(
        output.status.success(),
        "the stripped text did not run:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the fixture prints text")
}

/// One replay's stripped text compiled and run by both compiler legs this machine holds, and the
/// fixture's own classes run — one answer per compilation.
fn replay(leg: &Leg, replay: &Replay) -> Vec<(String, String, String)> {
    let sources = replay_sources(leg, replay);
    let original_dir = TempDir::new(&format!("{}-original", leg.label.replace(' ', "-")));
    original_classes(leg, replay, original_dir.path());
    let original_run = run(original_dir.path(), replay.class);

    let mut answers = Vec::new();
    let installed = PathBuf::from("javac");
    answers.push((
        format!(
            "{} / compiled by the installed javac --release 8",
            leg.label
        ),
        {
            let directory = TempDir::new(&format!("{}-current", leg.label.replace(' ', "-")));
            compile_with(&installed, true, directory.path(), &sources);
            run(directory.path(), replay.class)
        },
        original_run.clone(),
    ));
    if let Some(javac) = javac8() {
        answers.push((
            format!("{} / compiled by {}", leg.label, javac.display()),
            {
                let directory = TempDir::new(&format!("{}-javac8", leg.label.replace(' ', "-")));
                compile_with(&javac, false, directory.path(), &sources);
                run(directory.path(), replay.class)
            },
            original_run.clone(),
        ));
    }
    answers
}

#[test]
#[ignore = "needs a JDK on PATH: it strips the class-source comment lines, compiles the text with \
            the installed javac --release 8 (and with a real javac 8 when one is present) and runs \
            both, comparing every answer with the fixture's own class files"]
fn every_stripped_anchor_answers_what_its_class_answers() {
    eprintln!(
        "the real javac 8 leg is {}",
        javac8().map_or_else(
            || "absent on this machine".to_owned(),
            |path| path.display().to_string()
        )
    );
    for leg in LEGS {
        for anchor in REPLAYS {
            for (label, answer, original) in replay(leg, anchor) {
                assert_eq!(
                    original, anchor.answer,
                    "{label}: the fixture's own run moved"
                );
                assert_eq!(answer, original, "{label}: the stripped text diverges");
            }
        }
    }
}
