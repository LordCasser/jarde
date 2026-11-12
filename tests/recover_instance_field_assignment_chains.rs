//! Instance copy chains, including independently replayed historical boundaries.
//! Recovery preserves each receiver, producer evaluation and bytecode store order.
//! Strict JVM tests compile complete unedited Engine output on both frozen compiler legs.

use jarde::class_source::ClassSourceReport;
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
// The fixtures and the two compiler legs.
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names and the class files it produced.
struct Leg {
    label: &'static str,
    files: &'static [(&'static str, &'static [u8])],
}

impl Leg {
    /// The class files of one fixture family, in container order.
    fn family(&self, class: &str) -> Vec<(String, &'static [u8])> {
        let own = format!("{class}.class");
        let nested = format!("{class}$");
        let files: Vec<(String, &'static [u8])> = self
            .files
            .iter()
            .filter(|(name, _)| {
                let name: &str = name;
                name == own || name.starts_with(&nested)
            })
            .map(|(name, bytes)| ((*name).to_owned(), *bytes))
            .collect();
        assert_eq!(
            files.first().map(|(name, _)| name.as_str()),
            Some(own.as_str()),
            "the fixture family of `{class}` is committed"
        );
        files
    }

    /// One fixture family's container.
    fn fixture(&self, class: &str) -> Vec<u8> {
        let family = self.family(class);
        let entries: Vec<(&[u8], &[u8])> = family
            .iter()
            .map(|(name, bytes)| (name.as_bytes(), *bytes))
            .collect();
        zip_of(&entries)
    }
}

/// javac 23.0.1, `javac --release 8 -g -nowarn -d v8 CP.java MX.java NEG.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "CP.class",
        include_bytes!("fixtures/recover-instance-field-assignment-chains/v8/CP.class"),
    ),
    (
        "MX.class",
        include_bytes!("fixtures/recover-instance-field-assignment-chains/v8/MX.class"),
    ),
    (
        "NEG.class",
        include_bytes!("fixtures/recover-instance-field-assignment-chains/v8/NEG.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -g -nowarn -d v8-javac8 …`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "CP.class",
        include_bytes!("fixtures/recover-instance-field-assignment-chains/v8-javac8/CP.class"),
    ),
    (
        "MX.class",
        include_bytes!("fixtures/recover-instance-field-assignment-chains/v8-javac8/MX.class"),
    ),
    (
        "NEG.class",
        include_bytes!("fixtures/recover-instance-field-assignment-chains/v8-javac8/NEG.class"),
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

/// Every class this change pins, in the order the tests read them.
const CLASSES: &[&str] = &["CP", "MX", "NEG"];

// -------------------------------------------------------------------------------------------
// The presented texts.
// -------------------------------------------------------------------------------------------

/// `CP.inst` — the three-store instance chain: one assignment per store, in bytecode order.
const CP_INST: &[&str] = &[
    "        this.c = 5;",
    "        this.b = 5;",
    "        this.a = 5;",
];

/// `CP.pair` — the two-store instance chain.
const CP_PAIR: &[&str] = &["        this.b = 7;", "        this.a = 7;"];

/// The recovering methods of `CP` that must present every instruction they have.
const NO_QUOTE_METHODS: &[(&str, &str)] = &[
    ("CP", "<init>()V"),
    ("CP", "inst()V"),
    ("CP", "pair()V"),
    ("CP", "main([Ljava/lang/String;)V"),
];

/// The one diagnostic the static chain's own change left on this shape, and this change removes.
const COPY_REFUSAL: &str = "has no proved local assignment";

// -------------------------------------------------------------------------------------------
// The class-source surface.
// -------------------------------------------------------------------------------------------

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
        source_target: jarde::SourceTarget::JAVA_8,
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
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition: {other:?}"),
    }
}

/// One class's presentation, with jarde's own self-header asserted **before** anything is counted
/// in it: a render of nothing is not a render.
fn presented(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let report = class_source_of(snapshot, name);
    assert!(
        report
            .text
            .starts_with(&format!("// jarde: presentation of `{name}`")),
        "the render of `{name}` carries jarde's own self-header:\n{}",
        report.text
    );
    report
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

/// Every bytecode index one presentation quotes.
fn quoted_bcis(text: &str) -> Vec<u32> {
    let mut quoted = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("// @bytecode ") else {
            continue;
        };
        for token in rest.split_whitespace() {
            if let Ok(bci) = token.parse::<u32>() {
                quoted.push(bci);
            }
        }
    }
    quoted
}

/// The body text of one method, from its own signature line to the closing brace.
///
/// The signature is searched as the presentation writes it — the `// @method` line a member's
/// comment block carries, which precedes the body it belongs to and is part of the text a reader
/// reads. A refused method's marker line names the same signature first, and the slice then carries
/// the marker and the refusal with it, which is what the negative assertions read.
fn method_body<'a>(text: &'a str, signature: &str) -> &'a str {
    let start = text
        .find(signature)
        .unwrap_or_else(|| panic!("the presentation states `{signature}`:\n{text}"));
    let rest = &text[start..];
    let end = rest
        .find("\n    }")
        .unwrap_or_else(|| panic!("the method `{signature}` closes:\n{text}"));
    &rest[..end]
}

// -------------------------------------------------------------------------------------------
// The anchors: the instance chain recovers, and the presentations are the ones the design states.
// -------------------------------------------------------------------------------------------

/// The instance chain is written on both legs, with the text the design states — and `CP` is a
/// whole recovery: no copy of the class keeps a quote at all.
#[test]
fn the_instance_chain_anchors_write_their_presentations_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CP"));
        let report = presented(&snapshot, "CP");
        for anchor in CP_INST.iter().chain(CP_PAIR) {
            assert!(
                report.text.contains(anchor),
                "`CP` on {} lost the anchor {anchor:?}:\n{}",
                leg.label,
                report.text
            );
        }
        // The chain writes one assignment per store and **no** copy of the source per store: the
        // three stores are the three lines, and the value is written once in each of them.
        assert_eq!(
            report.text.matches("this.a =").count(),
            2,
            "`CP` on {} wrote the chain's stores more than once:\n{}",
            leg.label,
            report.text
        );
        assert!(
            !report.text.contains(COPY_REFUSAL),
            "`CP` on {} still carries the copy family's refusal:\n{}",
            leg.label,
            report.text
        );
        // The class is a whole recovery: an anchor that still quotes one instruction would be a
        // partial recovery counted as one.
        let quoted = quoted_bcis(&report.text);
        assert!(
            quoted.is_empty(),
            "`CP` on {} still quotes {quoted:?}:\n{}",
            leg.label,
            report.text
        );
    }
}

/// The recovering methods present every instruction they have.
#[test]
fn no_recovering_method_keeps_a_quote() {
    for leg in LEGS {
        for &(class, signature) in NO_QUOTE_METHODS {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let body = method_body(&report.text, signature);
            let quoted = quoted_bcis(body);
            assert!(
                quoted.is_empty(),
                "`{class}.{signature}` on {} still quotes {quoted:?}:\n{body}",
                leg.label
            );
            assert!(
                !body.contains("not recovered"),
                "`{class}.{signature}` on {} is not a whole recovery:\n{body}",
                leg.label
            );
            assert!(
                !body.contains(COPY_REFUSAL),
                "`{class}.{signature}` on {} still carries the copy family's refusal:\n{body}",
                leg.label
            );
        }
    }
}

// -------------------------------------------------------------------------------------------
// The boundary and the negatives keep their refusals.
// -------------------------------------------------------------------------------------------

/// Generic copy proofs retain each producer and store in bytecode evaluation order.
/// The complete class and side-effect controls are replayed by the strict JVM oracles.
#[test]
fn the_historical_boundaries_preserve_proved_evaluation_and_store_order_on_both_legs() {
    for leg in LEGS {
        for (class, cases) in [
            (
                "MX",
                vec![
                    (
                        "mixStaticRight()V",
                        vec!["MX.s = 5;", "this.b = 5;", "this.a = 5;"],
                    ),
                    (
                        "mixStaticLeft()V",
                        vec!["this.b = 6;", "this.a = 6;", "MX.s = 6;"],
                    ),
                    (
                        "mixStaticMid()V",
                        vec![
                            "this.b = 7;",
                            "int saved0 = 7;",
                            "MX.s = saved0;",
                            "this.a = saved0;",
                        ],
                    ),
                ],
            ),
            (
                "NEG",
                vec![
                    (
                        "cross()V",
                        vec![
                            "NEG saved0 = NEG.o1;",
                            "NEG saved1 = NEG.o2;",
                            "saved1.b = 5;",
                            "saved0.a = 5;",
                        ],
                    ),
                    (
                        "expr()I",
                        vec!["this.b = 5;", "this.a = 5 + 1;", "return this.b;"],
                    ),
                    (
                        "saved()V",
                        vec![
                            "int saved0 = f();",
                            "this.c = saved0;",
                            "this.b = saved0;",
                            "this.a = saved0;",
                        ],
                    ),
                    (
                        "holderChain()V",
                        vec![
                            "NEG saved0 = this.h();",
                            "NEG saved1 = this.h();",
                            "saved1.b = 5;",
                            "saved0.a = 5;",
                        ],
                    ),
                ],
            ),
        ] {
            let report = presented(&open(&leg.fixture(class)), class);
            assert!(
                quoted_bcis(&report.text).is_empty(),
                "{}: {}",
                leg.label,
                report.text
            );
            assert!(!report.text.contains("not recovered"), "{}", report.text);
            for (signature, statements) in cases {
                let body = method_body(&report.text, signature);
                let mut end = 0;
                for statement in statements {
                    assert_eq!(
                        body.matches(statement).count(),
                        1,
                        "{class}.{signature}: {body}"
                    );
                    let start = body.find(statement).unwrap();
                    assert!(start >= end, "{class}.{signature}: out of order: {body}");
                    end = start + statement.len();
                }
                if signature == "saved()V" {
                    assert_eq!(body.matches("f()").count(), 1, "{body}");
                }
                if signature == "holderChain()V" {
                    assert_eq!(body.matches("this.h()").count(), 2, "{body}");
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------
// The replay: both compiler legs, real execution, the fixture's own answers.
// -------------------------------------------------------------------------------------------

/// A scratch directory under the target tree.
fn scratch(label: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let ordinal = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("jarde-instance-chain-{label}-{stamp}-{ordinal}"));
    fs::create_dir_all(&path).expect("the scratch directory is created");
    path
}

/// Run one committed fixture under `-Xverify:all`, answering its standard output and exit status.
fn run_class(runner: &str, dir: &Path, class: &str) -> String {
    let run = Command::new(runner)
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(class)
        .output()
        .expect("the JVM runs");
    format!(
        "{}\nstatus={}",
        String::from_utf8_lossy(&run.stdout).trim_end(),
        run.status.code().unwrap_or(-1)
    )
}

/// Compile one complete unedited presentation and run it under `-Xverify:all`, answering its standard
/// output and exit status.
fn compile_and_run(compiler: &str, runner: &str, release: bool, dir: &Path, class: &str) -> String {
    let source = dir.join(format!("{class}.java"));
    let mut command = Command::new(compiler);
    if release {
        command.arg("--release").arg("8");
    }
    let output = command
        .arg("-nowarn")
        .arg("-d")
        .arg(dir)
        .arg(&source)
        .output()
        .expect("the compiler runs");
    assert!(
        output.status.success(),
        "{class}: javac failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    run_class(runner, dir, class)
}

/// The fixture's own class files, extracted once.
fn original_dir(label: &str, leg: &Leg, class: &str) -> PathBuf {
    let dir = scratch(&format!("original-{label}-{class}"));
    for (name, bytes) in leg.family(class) {
        fs::write(dir.join(&name), bytes).expect("the fixture class is written");
    }
    dir
}

#[test]
#[ignore = "compiles and runs the recovered text; needs the installed JDK (and javac 8 for the second leg)"]
fn the_recovered_text_compiles_and_runs_identically_on_both_legs() {
    let javac8 =
        Path::new("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac");
    let java8 =
        Path::new("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java");
    for leg in LEGS {
        for &class in CLASSES {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let text = report.text.clone();
            assert!(
                !text.contains("jarde_refused_body"),
                "`{class}` on {} still holds a refused body marker:\n{text}",
                leg.label
            );
            let want_dir = original_dir(leg.label, leg, class);
            let want = run_class("java", &want_dir, class);
            let work = scratch(&format!("recovered-{class}"));
            fs::write(work.join(format!("{class}.java")), &text)
                .expect("the presentation is written");
            let got = compile_and_run("javac", "java", true, &work, class);
            assert_eq!(
                got, want,
                "`{class}` on {} answers differently after the round trip:\n{text}",
                leg.label
            );
            if javac8.is_file() {
                let work8 = scratch(&format!("recovered-8-{class}"));
                fs::write(work8.join(format!("{class}.java")), &text)
                    .expect("the presentation is written");
                let got8 = compile_and_run(
                    javac8.to_str().expect("the path is UTF-8"),
                    java8.to_str().expect("the path is UTF-8"),
                    false,
                    &work8,
                    class,
                );
                assert_eq!(
                    got8, want,
                    "`{class}` on {} answers differently under javac 8:\n{text}",
                    leg.label
                );
            }
        }
    }
}

#[path = "support/copy_boundary_oracle.rs"]
mod boundary_oracle;

const CHAIN_RUNNER: &str = r#"public class Replay { public static void main(String[] args) { NEG.main(args); for (int mode=0;mode<4;mode++) { NEG.o1=mode==1?null:new NEG(); NEG.o2=mode==2?null:mode==3?NEG.o1:new NEG(); try { new NEG().cross(); System.out.println("ok"); } catch(Throwable t) { System.out.println(t.getClass().getName()); } System.out.println((NEG.o1==null?-99:NEG.o1.a)+"/"+(NEG.o2==null?-99:NEG.o2.b)); } for(int mode=0;mode<3;mode++) { Probe p=new Probe(); p.mode=mode; try {p.holderChain();System.out.println("ok");} catch(Throwable t){System.out.println(t.getClass().getName());} System.out.println(p.calls+"/"+p.left.a+"/"+p.right.b); } } } class Probe extends NEG { int calls,mode; NEG left=new NEG(),right=new NEG(); NEG h() { calls++; if(mode==1 && calls==2) throw new IllegalStateException(); if(mode==2 && calls==1) return null; return calls==1?left:right; } }"#;

#[test]
#[ignore = "strict full unedited class JVM equality, both frozen compiler legs"]
fn proved_chain_classes_require_full_class_equality() {
    for (i, leg) in LEGS.iter().enumerate() {
        for class in ["MX", "NEG"] {
            let report = presented(&open(&leg.fixture(class)), class);
            let replay = if class == "MX" {
                "public class Replay { public static void main(String[] args) { MX.main(args); } }"
            } else {
                CHAIN_RUNNER
            };
            boundary_oracle::require_equal(
                &format!("proved-chain-{i}-{class}"),
                class,
                leg.family(class)[0].1,
                &report.text,
                replay,
            );
        }
    }
}
