//! Request-stop regressions using the original EM11 class bytes.
use jarde::*;
use std::slice;

const EM11: &[u8] = include_bytes!("fixtures/em11-reference-overload.jar");

fn limits(headers: u64) -> Limits {
    let mut limits = task_budget(&[]).unwrap().limits().clone();
    limits.elapsed_millis = u64::MAX;
    limits.class_headers = headers;
    limits
}

// ledger=false: ordinary public local header cap.
// ledger=true: local is wide, a separate caller-owned ledger refuses the actual header charge.
fn present(headers: u64, ledger: bool) -> (ClassSourceReport, Option<OperationLedger>) {
    present_bytes(EM11, headers, ledger)
}

fn present_bytes(
    bytes: &[u8],
    headers: u64,
    ledger: bool,
) -> (ClassSourceReport, Option<OperationLedger>) {
    let mut budget = Budget::new(limits(if ledger { u64::MAX } else { headers }));
    assert!(!budget.cancellation_token().is_cancelled());
    let observed = ledger.then(|| {
        let entry = Budget::with_cancellation_token(limits(headers), CancellationToken::new());
        assert!(!entry.cancellation_token().is_cancelled());
        OperationLedger::new(&entry)
    });
    if let Some(ledger) = &observed {
        budget.with_ledger(ledger.clone(), UsageOwner::Methods);
    }
    let engine = Engine::new();
    // Keep one budget across open and class-source, matching the CLI request lifecycle.
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("em11/InputHierarchyCalls"),
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
    let OperationOutcome::Performed(report) = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .unwrap()
    else {
        panic!("the selected original class retains its public report")
    };
    if ledger {
        assert!(
            !budget.cancellation_token().is_cancelled(),
            "the independent ledger stop need not cancel the local token"
        );
    }
    (report, observed)
}

fn is_header_stop(execution: &ExecutionReport) -> bool {
    matches!(
        execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders
            },
            ..
        }
    )
}

fn run(report: &ClassSourceReport) -> &ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|m| {
            m.item.index == 3
                && m.item.identity.name.0 == b"run"
                && m.item.identity.descriptor.0 == b"()Ljava/lang/String;"
        })
        .unwrap()
}

fn assert_reliable_prefix(wide: &ClassSourceReport, stopped: &ClassSourceReport) {
    for index in 0..3 {
        let expected = &wide.methods[index];
        let actual = &stopped.methods[index];
        assert_eq!(actual.item.identity, expected.item.identity);
        assert_eq!(actual.text, expected.text);
        let ClassSourceOutcome::Recovered { report: a, .. } = &actual.outcome else {
            panic!()
        };
        let ClassSourceOutcome::Recovered { report: b, .. } = &expected.outcome else {
            panic!()
        };
        assert!(matches!(a.execution, ExecutionReport::Complete { .. }));
        assert!(!b.source_map.is_empty());
        assert_eq!(a.source_map.segments(), b.source_map.segments());
    }
}

#[test]
fn em11_wide_keeps_the_actual_mid_overload_and_physical_call_bci() {
    let (report, _) = present(4096, false);
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    let member = run(&report);
    let ClassSourceOutcome::Recovered {
        report: body,
        analysis,
    } = &member.outcome
    else {
        panic!()
    };
    assert!(matches!(
        analysis.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(matches!(body.execution, ExecutionReport::Complete { .. }));
    assert!(member.text.contains("take((em11.HMid) new em11.HLeaf())"));
    assert!(
        body.source_map
            .segments()
            .iter()
            .any(|segment| segment.mentions(7).is_some())
    );
    // The allocation instruction is BCI0; the constructor source anchor is BCI4.
    assert!(
        body.source_map
            .segments()
            .iter()
            .any(|segment| segment.mentions(0).is_some())
    );
}

#[test]
fn em11_first_reference_header_ledger_stop_keeps_original_budget_cause() {
    let (wide, _) = present(4096, false);
    let (report, ledger) = present(1, true);
    let first = ledger.as_ref().unwrap().stop_reason().unwrap();
    assert_eq!(first.kind, BulkStopKind::Budget);
    assert_eq!(first.dimension, Some(BudgetDimension::ClassHeaders));
    // The first real refusal must survive any later cancelled-token notification.
    assert!(is_header_stop(&report.execution), "{:?}", report.execution);
    assert_reliable_prefix(&wide, &report);
    assert_eq!(
        report.methods.len(),
        3,
        "the sticky ledger cannot admit an unpaid stopped run record"
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.code == "budget_exceeded_class_headers")
    );
}

#[test]
fn em11_plain_header_stop_pays_no_later_snapshot_analysis_work() {
    let (wide, _) = present(4096, false);
    let (_ledger_report, ledger) = present(1, true);
    let ledger = ledger.unwrap();
    assert_eq!(
        ledger.stop_reason().unwrap().dimension,
        Some(BudgetDimension::ClassHeaders)
    );
    let actual_first_refusal_usage = ledger.usage();
    let (report, _) = present(1, false);
    assert!(is_header_stop(&report.execution)); // Already passes old public code; insufficient alone.
    assert_reliable_prefix(&wide, &report);
    let ClassSourceOutcome::Recovered {
        report: body,
        analysis,
    } = &run(&report).outcome
    else {
        panic!()
    };
    assert!(matches!(
        analysis.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(is_header_stop(&body.execution));
    assert_eq!(body.content, RecoveryContent::NotProduced);
    assert!(body.text.is_empty() && body.source_map.is_empty());
    assert!(body.artifact.binding().is_none());
    // A live ledger prevents any charge after its first real refusal regardless of the old
    // carrier bug. It supplies the measured fee; no invented 223/253 total is used as oracle.
    assert_eq!(
        report.usage.analysis_steps, actual_first_refusal_usage.analysis_steps,
        "plain requests must not execute later proof loops after the first real header refusal"
    );
    // ResultItems may legitimately differ: the plain request can publish its stopped run.
}

#[test]
fn em11_cap_two_keeps_the_existing_snapshot_stop_and_prefix() {
    let (wide, _) = present(4096, false);
    let (report, _) = present(2, false);
    assert!(is_header_stop(&report.execution));
    assert_reliable_prefix(&wide, &report);
    let ClassSourceOutcome::Recovered { report: body, .. } = &run(&report).outcome else {
        panic!()
    };
    assert!(is_header_stop(&body.execution));
    assert_eq!(body.content, RecoveryContent::NotProduced);
}

// Change only the availability of the required physical HLeaf definition. Every class byte
// sequence is CRC-verified; all other records (including the unique caller) are preserved.
fn leaf_shape(copies: usize) -> Vec<u8> {
    use rawzip::{CompressionMethod, ZipArchive, ZipArchiveWriter, path::EntryPath};
    use std::io::{Cursor, Read, Write};
    fn records(jar: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let zip = ZipArchive::from_slice(jar).unwrap();
        let mut headers = zip.entries();
        let mut records = Vec::new();
        while let Some(header) = headers.next_entry().unwrap() {
            assert_eq!(header.compression_method(), CompressionMethod::STORE);
            let entry = zip.get_entry(header.wayfinder()).unwrap();
            let mut bytes = Vec::new();
            entry
                .verifying_reader(Cursor::new(entry.data()))
                .read_to_end(&mut bytes)
                .unwrap();
            records.push((header.file_path().as_ref().to_vec(), bytes));
        }
        records
    }
    let original = records(EM11);
    assert_eq!(original.len(), 7);
    const LEAF: &[u8] = b"em11/HLeaf.class";
    assert_eq!(
        original
            .iter()
            .filter(|(n, _)| n.as_slice() == LEAF)
            .count(),
        1
    );
    let mut out = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut out);
        for (name, bytes) in &original {
            for _ in 0..if name.as_slice() == LEAF { copies } else { 1 } {
                let (mut entry, config) = zip
                    .new_file(EntryPath::verbatim(name.clone()))
                    .compression_method(CompressionMethod::STORE)
                    .start()
                    .unwrap();
                let mut sink = config.wrap(&mut entry);
                sink.write_all(bytes).unwrap();
                let (_, descriptor) = sink.finish().unwrap();
                entry.finish(descriptor).unwrap();
            }
        }
        zip.finish().unwrap();
    }
    let derived = out.into_inner();
    let actual = records(&derived);
    assert_eq!(
        actual.iter().filter(|(n, _)| n.as_slice() == LEAF).count(),
        copies
    );
    let others = |records: Vec<(Vec<u8>, Vec<u8>)>| {
        records
            .into_iter()
            .filter(|(n, _)| n.as_slice() != LEAF)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        others(original),
        others(actual),
        "all other physical class bytes remain unchanged"
    );
    derived
}

fn assert_ordinary_leaf_refusal(jar: &[u8]) {
    let (report, _) = present_bytes(jar, 4096, false);
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    let member = run(&report);
    let ClassSourceOutcome::Recovered {
        analysis,
        report: body,
    } = &member.outcome
    else {
        panic!("ordinary missing/ambiguous evidence keeps the conservative body result");
    };
    assert!(matches!(
        analysis.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(matches!(body.execution, ExecutionReport::Complete { .. }));
    assert!(body.produced());
    assert!(!member.text.contains("take((em11.HMid) new em11.HLeaf())"));
    assert!(
        body.source_map
            .segments()
            .iter()
            .any(|s| s.mentions(7).is_some()),
        "the physical invocation origin survives conservative refusal"
    );
    for d in report.diagnostics.iter().chain(body.diagnostics.iter()) {
        assert!(
            !d.code.contains("budget") && !d.code.contains("cancel"),
            "{d:?}"
        );
    }
}

#[test]
fn em11_missing_leaf_is_an_ordinary_refusal() {
    assert_ordinary_leaf_refusal(&leaf_shape(0));
}

#[test]
fn em11_ambiguous_leaf_is_an_ordinary_refusal() {
    assert_ordinary_leaf_refusal(&leaf_shape(2));
}
