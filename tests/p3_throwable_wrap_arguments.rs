//! `recover-throwable-wrap-arguments` (CF-15/EM-10): the java.lang exception subclasses one call
//! argument widens to its java.lang ancestor, the wrap-and-rethrow family that closes, and the
//! table-out presentations that keep their refusal.
//!
//! The committed inputs are the patrol's own `C2.class` and the wrap family's `C2W.class` /
//! `C2WN.class` (javac 23.0.1 `--release 8 -g:none`, SHA-256 in the evidence README), and the
//! runner sources from the same directory. Three assertions per behavior:
//!
//! * the **text**: the wrap constructor's cause argument spells the required
//!   `java.lang.Throwable`, so the recompiled source cannot retarget the call to another
//!   overload;
//! * the **compile**: the recovered `C2`/`C2W` classes recompile under `javac --release 8`
//!   (the negative class never recompiles — its refusal is the point);
//! * the **runtime**: the original frozen bytes and the recovered class print the same lines
//!   under `java -Xverify:all`, exception paths included; the recorded JADX column beside the
//!   fixtures (`results/jadx-*-run.txt`) printed the same output when this evidence was frozen.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const C2: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/C2.class"
);
const C2W: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/wrap/original/C2W.class"
);
const C2WN: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/wrap/original/C2WN.class"
);
const C2WN_MY_FAILURE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/wrap/original/C2WN$MyFailure.class"
);
const C2WN_MY_ERROR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/wrap/original/C2WN$MyError.class"
);
const C2W_RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/wrap/C2WRunner.java"
);
const C2WN_RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/wrap/C2WNRunner.java"
);

/// The one run of the original `C2.main` under `java -Xverify:all` — also the recorded JADX
/// column's output (`results/jadx-C2-run.txt`).
const C2_EXPECTED_RUN: &str = "w:zero/true\nfine\n";

/// The one run of the original `C2WRunner` under `java -Xverify:all` — also the recorded JADX
/// column's output (`results/jadx-C2W-run.txt`).
const C2W_EXPECTED_RUN: &str = "\
alias:w:zero/true
iaeWrap:w:bad/true
nested:w2:w1/w1/true
log:fwd
forward:logged
note:7:multix
multi:noted
causeOnly:true:java.lang.IllegalStateException: cause
pick-T:ovl
overloadNarrow:picked
objectTarget:id:java.lang.IllegalStateException: obj
=== modes ===
alias:fine
iaeWrap:fine
nested:fine
forward:fine
multi:fine
causeOnly:fine
overloadNarrow:fine
objectTarget:fine
";

/// The one run of the original `C2WNRunner` under `java -Xverify:all`; the negative family's
/// own runtime is valid even where its recovery must stay refused.
const C2WN_EXPECTED_RUN: &str = "\
userWrap:w:user/true
log:err
userForward:logged
ioWrap:w:io/true
causeOnlyUser:true
arrayWrap:held:1:arr
userArrayWrap:held:1:uarr
pick-T:ovl
overloadNarrow:picked
rawMix:raw:1:log0:true
=== modes ===
userWrap:fine
userForward:fine
ioWrap:fine
causeOnlyUser:fine
arrayWrap:fine
userArrayWrap:fine
overloadNarrow:fine
rawMix:fine
";

fn opened(bytes: &[u8], class: &str) -> (artifact::ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    (snapshot, request)
}

fn class_source_of(
    bytes: &[u8],
    class: &str,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    let (snapshot, request) = opened(bytes, class);
    Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential(),
            budget,
        )
        .unwrap()
}

fn recover(bytes: &[u8], class: &str) -> ClassSourceReport {
    let mut budget = task_budget(&[]).unwrap();
    match class_source_of(bytes, class, &mut budget) {
        OperationOutcome::Performed(report) => report,
        other => panic!("complete class source expected: {other:?}"),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn assert_presented(report: &ClassSourceReport, name: &str, snippets: &[&str]) {
    let text = &member(report, name).text;
    assert!(
        !text.contains("@bytecode") && !text.contains("not recovered"),
        "`{name}` still quotes bytecode:\n{text}"
    );
    for snippet in snippets {
        assert!(
            text.contains(snippet),
            "`{name}` misses `{snippet}`:\n{text}"
        );
    }
}

fn scratch(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "jarde-p3-throwable-wrap-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create comparison directory");
    root
}

fn compile(dir: &Path, sources: &[&str]) {
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-Xlint:-options")
        .arg("-cp")
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .args(sources.iter().map(|source| dir.join(source)))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run(dir: &Path, main_class: &str) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(main_class)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Installs the frozen class bytes, so the run exercises the exact input the engine consumed.
fn install(dir: &Path, class: &str, bytes: &[u8]) {
    fs::write(dir.join(class), bytes).expect("install the frozen original class");
}

#[test]
fn c2_alias_wrapped_rethrow_presents_the_full_catch_body() {
    let report = recover(C2, "C2");
    let text = &report.text;
    assert!(
        !text.contains("no safe reference conversion evidence"),
        "the wrap argument refusal is closed:\n{text}"
    );
    let alias = member(&report, "alias").text.clone();
    assert!(
        !alias.contains("@bytecode") && !alias.contains("not recovered"),
        "alias still quotes bytecode:\n{alias}"
    );
    for statement in [
        "java.lang.String local2 = local1.getMessage();",
        "java.lang.RuntimeException local3 = new java.lang.RuntimeException(\"w:\" + local2, (java.lang.Throwable) local1);",
        "throw local3;",
    ] {
        assert!(
            alias.contains(statement),
            "alias misses `{statement}`:\n{alias}"
        );
    }
    // The main method the baseline already recovered keeps its exact presentation.
    assert!(
        member(&report, "main").text.contains(
            "java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) local1.getMessage()).append(\"/\").append((java.lang.Object) local1.getCause() instanceof java.lang.IllegalStateException).toString());"
        ),
        "main changed:\n{}",
        member(&report, "main").text
    );
}

#[test]
fn recovered_c2_runtime_matches_the_original_class() {
    let report = recover(C2, "C2");

    let original_dir = scratch("c2-original");
    install(&original_dir, "C2.class", C2);
    let original_run = run(&original_dir, "C2");

    let recovered_dir = scratch("c2-recovered");
    fs::write(recovered_dir.join("C2.java"), &report.text).unwrap();
    compile(&recovered_dir, &["C2.java"]);
    let recovered_run = run(&recovered_dir, "C2");

    assert_eq!(original_run, C2_EXPECTED_RUN, "{original_run}");
    assert_eq!(recovered_run, original_run, "the recovered class diverges");
}

#[test]
fn wrap_family_recovers_every_variant_with_the_required_spelling() {
    let report = recover(C2W, "C2W");
    let text = &report.text;
    assert!(
        !text.contains("no safe reference conversion evidence"),
        "no family member may refuse:\n{text}"
    );
    assert!(
        !text.contains("@bytecode"),
        "the whole class presents:\n{text}"
    );

    assert_presented(
        &report,
        "alias",
        &[
            "java.lang.String local2 = local1.getMessage();",
            "java.lang.RuntimeException local3 = new java.lang.RuntimeException(\"w:\" + local2, (java.lang.Throwable) local1);",
            "throw local3;",
        ],
    );
    assert_presented(
        &report,
        "iaeWrap",
        &[
            "new java.lang.RuntimeException(\"w:\" + local1.getMessage(), (java.lang.Throwable) local1);",
        ],
    );
    // The double wrap keeps both edges: ISE -> Throwable for `local1` and the second wrap's
    // RuntimeException -> Throwable for `local2`.
    assert_presented(
        &report,
        "nested",
        &[
            "java.lang.RuntimeException local2 = new java.lang.RuntimeException(\"w1\", (java.lang.Throwable) local1);",
            "java.lang.RuntimeException local3 = new java.lang.RuntimeException(\"w2:\" + local2.getMessage(), (java.lang.Throwable) local2);",
            "throw local3;",
        ],
    );
    // The user forwarding method and the multi-position call: only the Throwable slots widen,
    // the int and char slots stay untouched (a wrong widening there would not compile).
    assert_presented(&report, "forward", &["log((java.lang.Throwable) local1);"]);
    assert_presented(
        &report,
        "multi",
        &["note(7, (java.lang.Throwable) local1, 'x');"],
    );
    // The one-argument constructor resolves to `(Throwable)` in the original source, so the
    // widening position is 0 here, not 1.
    assert_presented(
        &report,
        "causeOnly",
        &["new java.lang.RuntimeException((java.lang.Throwable) local1);"],
    );
    // The overload protection: the required type's spelling keeps the call on `pick(Throwable)`.
    assert_presented(
        &report,
        "overloadNarrow",
        &["pick((java.lang.Throwable) local1);"],
    );
    assert!(
        !member(&report, "overloadNarrow")
            .text
            .contains("pick(local1)"),
        "the narrowed overload must not retarget:\n{}",
        member(&report, "overloadNarrow").text
    );
    // The existing Object-target answer still answers before the table ever runs.
    assert_presented(
        &report,
        "objectTarget",
        &["identity((java.lang.Object) local1);"],
    );

    let original_dir = scratch("c2w-original");
    install(&original_dir, "C2W.class", C2W);
    fs::write(original_dir.join("C2WRunner.java"), C2W_RUNNER).unwrap();
    compile(&original_dir, &["C2WRunner.java"]);
    let original_run = run(&original_dir, "C2WRunner");

    let recovered_dir = scratch("c2w-recovered");
    fs::write(recovered_dir.join("C2W.java"), &report.text).unwrap();
    fs::write(recovered_dir.join("C2WRunner.java"), C2W_RUNNER).unwrap();
    compile(&recovered_dir, &["C2W.java", "C2WRunner.java"]);
    let recovered_run = run(&recovered_dir, "C2WRunner");

    assert_eq!(original_run, C2W_EXPECTED_RUN, "{original_run}");
    assert_eq!(recovered_run, original_run, "the recovered family diverges");
}

#[test]
fn table_out_presentations_keep_the_refusal_and_the_original_runtime() {
    let report = recover(C2WN, "C2WN");
    let text = &report.text;
    assert_eq!(
        text.matches("no safe reference conversion evidence")
            .count(),
        6,
        "the six table-out calls stay refused:\n{text}"
    );

    // N1: the user exception classes are real Throwables but no java.lang rows.
    let user_wrap = &member(&report, "userWrap").text;
    assert!(
        user_wrap.contains(
            "presents `C2WN$MyFailure` but the invocation requires `java.lang.Throwable`"
        ),
        "{user_wrap}"
    );
    let user_forward = &member(&report, "userForward").text;
    assert!(
        user_forward
            .contains("presents `C2WN$MyError` but the invocation requires `java.lang.Throwable`"),
        "{user_forward}"
    );
    // N2: java.io.IOException stays outside the java.lang closed set in this slice.
    let io_wrap = &member(&report, "ioWrap").text;
    assert!(
        io_wrap.contains(
            "presents `java.io.IOException` but the invocation requires `java.lang.Throwable`"
        ),
        "{io_wrap}"
    );
    // N5: the one-argument constructor's slot 0 refuses the user type just the same.
    let cause_only_user = &member(&report, "causeOnlyUser").text;
    assert!(
        cause_only_user.contains(
            "the parameter 0 of the invocation at BCI 23 is declared `java.lang.Throwable` presents `C2WN$MyFailure`"
        ),
        "{cause_only_user}"
    );
    // N4: arrays stay invariant on their component; the presented array arrives as a parameter.
    let array_wrap = &member(&report, "arrayWrap").text;
    assert!(
        array_wrap.contains("presents `java.lang.IllegalStateException[]` but the invocation requires `java.lang.Throwable[]`"),
        "{array_wrap}"
    );
    let user_array_wrap = &member(&report, "userArrayWrap").text;
    assert!(
        user_array_wrap.contains(
            "presents `C2WN$MyFailure[]` but the invocation requires `java.lang.Throwable[]`"
        ),
        "{user_array_wrap}"
    );

    // The two companion shapes of the same class recover: the narrowed overload keeps its
    // required spelling and the raw boxed argument stays with the Object answer.
    let overload_narrow = &member(&report, "overloadNarrow").text;
    assert!(
        overload_narrow.contains("pick((java.lang.Throwable) local1);")
            && !overload_narrow.contains("@bytecode"),
        "{overload_narrow}"
    );
    let raw_mix = &member(&report, "rawMix").text;
    assert!(
        raw_mix.contains("(java.lang.Object) java.lang.Boolean.valueOf(true)")
            && raw_mix.contains("log0((java.lang.Throwable) local1);")
            && !raw_mix.contains("@bytecode"),
        "{raw_mix}"
    );

    let original_dir = scratch("c2wn-original");
    install(&original_dir, "C2WN.class", C2WN);
    install(&original_dir, "C2WN$MyFailure.class", C2WN_MY_FAILURE);
    install(&original_dir, "C2WN$MyError.class", C2WN_MY_ERROR);
    fs::write(original_dir.join("C2WNRunner.java"), C2WN_RUNNER).unwrap();
    compile(&original_dir, &["C2WNRunner.java"]);
    let original_run = run(&original_dir, "C2WNRunner");
    assert_eq!(original_run, C2WN_EXPECTED_RUN, "{original_run}");
}

#[test]
fn wrap_recovery_stops_and_cancels_before_any_published_output() {
    let mut bounded =
        task_budget(&[BudgetOverride::new("output_bytes", 8).expect("a legal output cap")])
            .unwrap();
    match class_source_of(C2W, "C2W", &mut bounded) {
        OperationOutcome::Incomplete(selection) => {
            assert!(
                !matches!(selection.execution, ExecutionReport::Complete { .. }),
                "an eight-byte output budget is not a complete run"
            );
        }
        OperationOutcome::Performed(report) => {
            panic!(
                "a stopped delivery must not publish a full report: {}",
                report.text
            )
        }
        other => panic!("the bounded class source answered {other:?}"),
    }

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut cancelled =
        Budget::with_cancellation_token(task_budget(&[]).unwrap().limits().clone(), cancellation);
    match class_source_of(C2W, "C2W", &mut cancelled) {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            panic!(
                "a cancelled delivery must not publish a full report: {}",
                report.text
            )
        }
        other => panic!("the cancelled class source answered {other:?}"),
    }
}
