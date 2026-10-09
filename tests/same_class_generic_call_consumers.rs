//! End-to-end class-source checks for the same-class generic call consumer milestone. The inputs
//! are the frozen source fixtures used by the four-lane acceptance harness; these checks ensure
//! the reader, same-run AST/SSA facts, class-local staging and emitted declarations agree.

use jarde::*;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct FixtureDirectory(std::path::PathBuf);

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture_bytes(name: &str, source: &str, debug: bool) -> Vec<u8> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "jarde-generic-call-{name}-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("the fixture directory is created");
    let _directory = FixtureDirectory(directory.clone());
    let empty_path = directory.join("empty");
    std::fs::create_dir(&empty_path).expect("the empty Java lookup path is created");
    let source_path = directory.join(format!("{name}.java"));
    std::fs::write(&source_path, source).expect("the fixture source is written");
    let compiled = Command::new("javac")
        .args([
            "--release",
            "8",
            if debug { "-g" } else { "-g:none" },
            "-Xlint:-options",
            "-classpath",
        ])
        .arg(&empty_path)
        .arg("-sourcepath")
        .arg(&empty_path)
        .arg(&source_path)
        .current_dir(&directory)
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    std::fs::read(directory.join(format!("{name}.class"))).expect("the fixture class is written")
}

fn class_source_from_bytes(
    name: &str,
    bytes: &[u8],
    budget: &mut Budget,
) -> Result<OperationOutcome<ClassSourceReport>> {
    let engine = Engine::new();
    let mut open_budget = Budget::new(facade::task_limits(&[]).expect("bounded open limits"));
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut open_budget)
        .expect("the fixture class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
    engine.class_source(&[snapshot], &request, budget)
}

fn class_source(name: &str, source: &str) -> ClassSourceReport {
    let bytes = fixture_bytes(name, source, false);
    let mut budget = Budget::new(facade::task_limits(&[]).expect("bounded task limits"));
    match class_source_from_bytes(name, &bytes, &mut budget) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("the fixture answers one class-source request: {other:?}"),
    }
}

#[test]
fn staged_generic_headers_keep_the_same_debug_names_as_the_emitted_body() {
    for (name, source, header) in [
        (
            "CallRelay",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/CallRelay.java"
            ),
            "public T relay(T ",
        ),
        (
            "NestedCallArgument",
            include_str!(
                "../openspec/changes/recover-same-class-generic-call-consumers/results/extension/nested-call-argument-v1/input/NestedCallArgument.java"
            ),
            "public T relay(T ",
        ),
        (
            "CatchCallMarker",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/CatchCallMarker.java"
            ),
            "private T maybe(T ",
        ),
    ] {
        for debug in [true, false] {
            let bytes = fixture_bytes(name, source, debug);
            let mut budget = Budget::new(facade::task_limits(&[]).expect("bounded task limits"));
            let Ok(OperationOutcome::Performed(report)) =
                class_source_from_bytes(name, &bytes, &mut budget)
            else {
                panic!("the complete fixture produces class source");
            };
            assert!(report.text.contains(header), "{}", report.text);
            // Recompile the complete emitted class in an empty lookup path. A parameter renamed
            // only in its staged header leaves the retained debug AST's names unbound here.
            assert!(!fixture_bytes(name, &report.text, false).is_empty());
        }
    }
}

#[test]
fn exhausted_call_publication_does_not_leave_a_partial_generic_chain() {
    for (name, source, pairs) in [
        (
            "CallRelay",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/CallRelay.java"
            ),
            &[("identity", "relay")][..],
        ),
        (
            "IndependentChains",
            "public class IndependentChains<T> {
             public T first(T x) { return x; }
             public T firstRelay(T x) { return first(x); }
             public T second(T x) { return x; }
             public T secondRelay(T x) { return second(x); }
             }",
            &[("first", "firstRelay"), ("second", "secondRelay")][..],
        ),
    ] {
        let bytes = fixture_bytes(name, source, false);
        let generous = facade::task_limits(&[]).expect("bounded task limits");
        let mut complete_budget = Budget::new(generous.clone());
        let Ok(OperationOutcome::Performed(complete)) =
            class_source_from_bytes(name, &bytes, &mut complete_budget)
        else {
            panic!("the complete control must produce class source");
        };
        assert!(matches!(
            complete.execution,
            ExecutionReport::Complete { .. }
        ));
        for (callee, caller) in pairs {
            assert!(
                complete
                    .text
                    .contains(&format!("public T {callee}(T arg1)"))
            );
            assert!(
                complete
                    .text
                    .contains(&format!("public T {caller}(T arg1)"))
            );
        }
        let usage = complete_budget.usage();
        let mut assembly_stops = 0;
        for output_dimension in [false, true] {
            let total = if output_dimension {
                usage.output_bytes
            } else {
                usage.analysis_steps
            };
            let mut cutoffs: Vec<_> = (1..=64).map(|step| total * step / 64).collect();
            cutoffs.push(total.saturating_sub(1));
            cutoffs.sort_unstable();
            cutoffs.dedup();
            for cutoff in cutoffs {
                let mut limits = generous.clone();
                if output_dimension {
                    limits.output_bytes = cutoff;
                } else {
                    limits.analysis_steps = cutoff;
                }
                let mut budget = Budget::new(limits);
                let answer = class_source_from_bytes(name, &bytes, &mut budget);
                let report = match answer {
                    Ok(OperationOutcome::Performed(report)) => report,
                    Err(Error::BudgetExceeded { .. }) => continue,
                    Ok(OperationOutcome::Incomplete(selection)) => {
                        assert!(!matches!(
                            selection.execution,
                            ExecutionReport::Complete { .. }
                        ));
                        continue;
                    }
                    other => panic!("unexpected bounded answer at {cutoff}: {other:?}"),
                };
                if matches!(report.execution, ExecutionReport::Complete { .. }) {
                    continue;
                }
                for (callee, caller) in pairs {
                    let recovered_chain: Vec<_> = report
                .methods
                .iter()
                .filter(|method| {
                    (method.item.name.raw().0 == callee.as_bytes()
                        || method.item.name.raw().0 == caller.as_bytes())
                        && matches!(
                            &method.outcome,
                            ClassSourceOutcome::Recovered { report, analysis }
                                if matches!(report.execution, ExecutionReport::Complete { .. })
                                    && matches!(analysis.execution, ExecutionReport::Complete { .. })
                                    && report.quality == Quality::Structured
                        )
                })
                .collect();
                    if recovered_chain.len() != 2 {
                        continue;
                    }
                    assembly_stops += 1;
                    let generic_headers = recovered_chain
                        .iter()
                        .filter(|method| {
                            method.declaration.as_deref().is_some_and(|declaration| {
                                declaration.contains("public T ")
                                    && declaration.contains("(T arg1)")
                            })
                        })
                        .count();
                    assert!(
                        generic_headers == 0 || generic_headers == 2,
                        "budget stopped after both bodies recovered but left a partial call chain; \
                 class={name}, pair={callee}/{caller}, output_dimension={output_dimension}, cutoff={cutoff}: {report:#?}",
                    );
                }
            }
        }
        assert!(
            assembly_stops > 0,
            "the sweep must reach call publication for {name}"
        );
    }
}

#[test]
fn direct_generic_calls_publish_only_closed_parameterized_headers() {
    let cases = [
        (
            "CallRelay",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/CallRelay.java"
            ),
            &["public T identity(T arg1)", "public T relay(T arg1)"][..],
        ),
        (
            "ArrayRelay",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/ArrayRelay.java"
            ),
            &[
                "public T[] identity(T[] arg1)",
                "public T[] relay(T[] arg1)",
            ][..],
        ),
        (
            "NumberBoundRelay",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/NumberBoundRelay.java"
            ),
            &["public T identity(T arg1)", "public T relay(T arg1)"][..],
        ),
        (
            "WideRelay",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/WideRelay.java"
            ),
            &["public T relay(long arg1, T arg3, double arg4, T arg6)"][..],
        ),
        (
            "DeepRelay",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/DeepRelay.java"
            ),
            &[
                "public T relay0(T arg1)",
                "public T relay1(T arg1)",
                "public T relay2(T arg1)",
                "public T identity(T arg1)",
            ][..],
        ),
        (
            "EmptySink",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/EmptySink.java"
            ),
            &["public void sink(T arg1)"],
        ),
        (
            "ReverseDeclarationRelay",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/ReverseDeclarationRelay.java"
            ),
            &[
                "public T identity(T arg1)",
                "public T relay2(T arg1)",
                "public T relay1(T arg1)",
                "public T relay0(T arg1)",
            ][..],
        ),
        (
            "MethodShadow",
            include_str!(
                "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/MethodShadow.java"
            ),
            &[
                "public <T extends java.lang.Number> T relay(T arg1)",
                "public <U extends java.lang.Number> U identity(U arg1)",
            ][..],
        ),
    ];

    for (name, source, expected_headers) in cases {
        let report = class_source(name, source);
        for expected in expected_headers {
            assert!(
                report.text.contains(expected),
                "{name} did not publish `{expected}`:\n{}",
                report.text
            );
        }
    }
}

#[test]
fn unknown_incoming_call_rolls_back_only_its_connected_component() {
    let source = include_str!(
        "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/UnknownIncoming.java"
    );
    let report = class_source("UnknownIncoming", source);
    assert!(
        report.text.contains("public T untouched(T arg1)"),
        "the independent generic method should remain published:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("public T identity(T arg1)"),
        "the unsafe incoming use must roll back identity and its relay component:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("public T safe(T arg1)"),
        "safe must roll back with its connected callee:\n{}",
        report.text
    );
}

#[test]
fn structured_catch_call_results_keep_the_constructor_contract() {
    let source = include_str!(
        "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/CatchCallMarker.java"
    );
    let report = class_source("CatchCallMarker", source);
    assert!(
        report
            .text
            .contains("public CatchCallMarker(T arg1, boolean arg2)"),
        "constructor Signature was not published:\n{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("private T maybe(T arg1, boolean arg2)"),
        "catch-safe generic callee Signature was not published:\n{}",
        report.text
    );
}

#[test]
fn nested_generic_call_results_feed_the_next_proved_call() {
    let source = include_str!(
        "../openspec/changes/recover-same-class-generic-call-consumers/results/extension/nested-call-argument-v1/input/NestedCallArgument.java"
    );
    let report = class_source("NestedCallArgument", source);
    for expected in [
        "public T first(T arg1)",
        "public T second(T arg1)",
        "public T relay(T arg1)",
    ] {
        assert!(
            report.text.contains(expected),
            "nested generic call proof did not publish `{expected}`:\n{}",
            report.text
        );
    }
}

#[test]
fn bounded_same_class_overloads_keep_the_physical_target_with_only_proved_casts() {
    let bound = include_str!(
        "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/frozen-inputs-v1/openjdk23/debug/BoundOverload/BoundOverload.original.java"
    );
    let report = class_source("BoundOverload", bound);
    assert!(
        report.text.contains("public void relay(T arg1)"),
        "the generic caller header should be publishable:\n{}",
        report.text
    );
    assert!(
        report.text.contains("(java.lang.Number) arg1"),
        "the source should retain the proven Number overload selection:\n{}",
        report.text
    );

    assert!(
        report
            .text
            .contains("public void pick(java.lang.Comparable<T> "),
        "the related generic overload declaration should remain independently published:\n{}",
        report.text
    );

    let plain = include_str!(
        "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/PlainUpperBoundOverload.java"
    );
    let report = class_source("PlainUpperBoundOverload", plain);
    assert!(
        report.text.contains("public void relay(T arg1)")
            && report.text.contains("pick(arg1)")
            && !report.text.contains("(java.lang.Number) arg1"),
        "a uniquely applicable bounded overload should not receive an unnecessary cast:\n{}",
        report.text
    );

    let same_name = include_str!(
        "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/SameNameOverload.java"
    );
    let report = class_source("SameNameOverload", same_name);
    assert!(
        report.text.contains("public void pick(T arg1)")
            && report
                .text
                .contains("public void pick(java.lang.String arg1)"),
        "the class and overload source signatures should remain independently published:\n{}",
        report.text
    );
}

#[test]
fn constructor_only_classes_keep_the_existing_generic_projection() {
    let raw = class_source(
        "RawNewHold",
        include_str!(
            "../openspec/changes/recover-class-scope-constructor-parameters/evidence/fixtures/RawNewHold/RawNewHold.java"
        ),
    );
    assert!(
        raw.text.contains("public RawNewHold(T ") && raw.text.contains("public T v;"),
        "a raw construction must preserve the existing constructor and field proof:\n{}",
        raw.text
    );
    let peer = class_source(
        "PeerNewHold",
        include_str!(
            "../openspec/changes/recover-class-scope-constructor-parameters/evidence/fixtures/PeerNewHold/PeerNewHold.java"
        ),
    );
    assert!(
        peer.text.contains("public PeerNewHold(T ")
            && peer.text.contains("boolean ")
            && peer.text.contains("public PeerNewHold(java.lang.Object ")
            && peer.text.contains("public java.lang.Object v;"),
        "the proved two-parameter constructor must survive beside the erased peer caller and field:\n{}",
        peer.text
    );
}

#[test]
fn raw_incoming_call_keeps_the_previously_proved_field_setter() {
    let report = class_source(
        "SCGA",
        include_str!("../openspec/evidence/generic-holder-write-boundaries/SCGA/source/SCGA.java"),
    );
    assert!(
        report.text.contains("public void put(T ") && report.text.contains("public T v;"),
        "the existing void-body proof and actual raw incoming selection must remain valid:\n{}",
        report.text
    );
}

#[test]
fn proved_void_effects_survive_generic_call_staging() {
    let report = class_source(
        "VoidDirect",
        include_str!(
            "../openspec/evidence/same-class-generic-call-consumers-2026-10-09/fixtures/sources/VoidDirect.java"
        ),
    );
    assert!(
        report.text.contains("public void sink(T ")
            && report.text.contains("public void relay(T ")
            && report.text.contains("this.calls++"),
        "the existing void-body proof must preserve both generic headers and the counter effect:\n{}",
        report.text
    );
}

#[test]
fn unread_parameterized_formals_require_complete_slot_evidence() {
    let report = class_source(
        "UnreadContainer",
        "public class UnreadContainer<T> {
         public void unused(Comparable<T> value, long count) { System.out.println(count); }
         public void read(Comparable<T> value, long count) { System.out.println(value); }
         public void overwrite(Comparable<T> value, long count) {
             value = null; System.out.println(count);
         }
         }",
    );
    assert!(
        report
            .text
            .contains("public void unused(java.lang.Comparable<T> "),
        "an actually unread reference formal may retain its declared type beside a read wide formal:\n{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("public void read(java.lang.Comparable ")
            && report
                .text
                .contains("public void overwrite(java.lang.Comparable "),
        "a read or overwritten parameterized formal must not acquire an unread-slot certificate:\n{}",
        report.text
    );
    assert!(
        report.text.contains("java.lang.System.out.println(") && report.text.contains("= null;"),
        "the certificate must preserve the original call and write effects:\n{}",
        report.text
    );
}
