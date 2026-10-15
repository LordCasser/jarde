//! Public DT-29 regressions for request stops in direct-parent dependency proofs.
use jarde::*;
use rawzip::{CompressionMethod, ZipArchive, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Read, Write};
use std::slice;

const DT29: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-27/dt29-same-package-parent-field-writes/outputs/input.jar"
);

fn limits(headers: u64) -> Limits {
    let mut limits = task_budget(&[]).unwrap().limits().clone();
    limits.elapsed_millis = u64::MAX;
    limits.class_headers = headers;
    limits
}

const EXTERNAL_SOURCE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-27/dt29-same-package-parent-field-writes/outputs/combined-input.jar"
);

const DT29_PARENT: &[u8] = b"dt29/SamePackageParentFamily$A.class";
const DT29_TARGET: &[u8] = b"dt29/SamePackageParentFamily$B.class";

fn read_verified<R: Read>(entry: &rawzip::ZipSliceEntry<'_>, reader: R) -> Vec<u8> {
    let mut verified = entry.verifying_reader(reader);
    let mut decoded = Vec::new();
    verified
        .read_to_end(&mut decoded)
        .expect("DT-29 class bytes and CRC verify");
    decoded
}

fn read_dt29_class(entry: &rawzip::ZipSliceEntry<'_>, method: CompressionMethod) -> Vec<u8> {
    match method {
        CompressionMethod::STORE => read_verified(entry, Cursor::new(entry.data())),
        CompressionMethod::DEFLATE => {
            read_verified(entry, flate2::bufread::DeflateDecoder::new(entry.data()))
        }
        other => panic!("unsupported DT-29 class compression method: {other:?}"),
    }
}

fn dt29_class_records(jar: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    let archive = ZipArchive::from_slice(jar).expect("DT-29 fixture is a ZIP");
    let mut headers = archive.entries();
    let mut records = Vec::new();
    while let Some(header) = headers.next_entry().expect("DT-29 ZIP header reads") {
        let name = header.file_path();
        if !name.as_ref().ends_with(b".class") {
            continue;
        }
        let name = name.as_ref().to_vec();
        let method = header.compression_method();
        let entry = archive
            .get_entry(header.wayfinder())
            .expect("DT-29 entry resolves");
        records.push((name, read_dt29_class(&entry, method)));
    }
    records
}

fn write_dt29_class(
    writer: &mut ZipArchiveWriter<&mut Cursor<Vec<u8>>>,
    name: Vec<u8>,
    bytes: &[u8],
) {
    let (mut entry, config) = writer
        .new_file(EntryPath::verbatim(name))
        .compression_method(CompressionMethod::STORE)
        .start()
        .expect("derived DT-29 entry starts");
    let mut sink = config.wrap(&mut entry);
    sink.write_all(bytes).expect("derived DT-29 class writes");
    let (_, descriptor) = sink.finish().expect("derived DT-29 entry closes");
    entry
        .finish(descriptor)
        .expect("derived DT-29 record finishes");
}

/// Derive a class-only ZIP from the committed fixture, preserving and CRC-checking every actual
/// class byte sequence. The requested A shape is the only class-record change; B stays unique.
fn dt29_parent_shape(missing_parent: bool) -> Vec<u8> {
    let records = dt29_class_records(DT29);
    let parent: Vec<_> = records
        .iter()
        .filter(|(name, _)| name.as_slice() == DT29_PARENT)
        .collect();
    let target: Vec<_> = records
        .iter()
        .filter(|(name, _)| name.as_slice() == DT29_TARGET)
        .collect();
    assert_eq!(
        parent.len(),
        1,
        "source has one physical direct-parent class"
    );
    assert_eq!(target.len(), 1, "source has one physical target class");

    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in &records {
            if missing_parent && name.as_slice() == DT29_PARENT {
                continue;
            }
            write_dt29_class(&mut writer, name.clone(), bytes);
            if !missing_parent && name.as_slice() == DT29_PARENT {
                write_dt29_class(&mut writer, name.clone(), bytes);
            }
        }
        writer.finish().expect("derived DT-29 ZIP closes");
    }
    let derived = output.into_inner();
    let derived_records = dt29_class_records(&derived);
    assert_eq!(
        derived_records
            .iter()
            .filter(|(name, _)| name.as_slice() == DT29_TARGET)
            .count(),
        1,
        "target B remains unique"
    );
    assert_eq!(
        derived_records
            .iter()
            .filter(|(name, _)| name.as_slice() == DT29_PARENT)
            .count(),
        if missing_parent { 0 } else { 2 },
        "only the required parent entry shape changes"
    );
    let mut expected_other: Vec<_> = records
        .into_iter()
        .filter(|(name, _)| name.as_slice() != DT29_PARENT)
        .collect();
    let mut actual_other: Vec<_> = derived_records
        .into_iter()
        .filter(|(name, _)| name.as_slice() != DT29_PARENT)
        .collect();
    expected_other.sort();
    actual_other.sort();
    assert_eq!(
        actual_other, expected_other,
        "all other class records retain their original bytes"
    );
    derived
}

fn assert_ordinary_parent_refusal(jar: &[u8]) {
    let report = present_class(jar, "dt29/SamePackageParentFamily$B", 4096);
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    let member = physical_set(&report).expect("the unique physical set(ZZ)V remains present");
    let ClassSourceOutcome::Recovered {
        report: body,
        analysis,
    } = &member.outcome
    else {
        panic!("ordinary dependency refusal keeps the actual body outcome");
    };
    assert!(matches!(
        analysis.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(matches!(body.execution, ExecutionReport::Complete { .. }));
    assert!(
        body.produced(),
        "ordinary refusal retains a produced body with source anchors"
    );
    for (bci, field) in [
        (2, "publicField"),
        (7, "protectedField"),
        (12, "packagePrivateField"),
    ] {
        assert!(
            member.text.contains(&format!("@bytecode {bci}")),
            "raw BCI {bci} stays visible"
        );
        assert!(
            !member.text.contains(&format!(
                "((dt29.SamePackageParentFamily$A) this).{field} = arg1;"
            )),
            "unproved {field} write gets no parent cast"
        );
        assert!(
            body.source_map
                .segments()
                .iter()
                .any(|segment| segment.mentions(bci).is_some()),
            "source map retains the physical BCI {bci} anchor"
        );
    }
    for diagnostic in report.diagnostics.iter().chain(body.diagnostics.iter()) {
        let label = format!("{} {}", diagnostic.code, diagnostic.message).to_ascii_lowercase();
        assert!(
            !label.contains("budget") && !label.contains("cancel"),
            "ordinary missing/ambiguous dependency must not become a request stop: {diagnostic:?}"
        );
    }
}

#[test]
fn missing_direct_parent_is_an_ordinary_refusal_with_physical_anchors() {
    assert_ordinary_parent_refusal(&dt29_parent_shape(true));
}

#[test]
fn ambiguous_direct_parent_is_an_ordinary_refusal_with_physical_anchors() {
    assert_ordinary_parent_refusal(&dt29_parent_shape(false));
}

fn present(headers: u64) -> ClassSourceReport {
    present_class(DT29, "dt29/SamePackageParentFamily$B", headers)
}

fn present_class(bytes: &[u8], name: &str, headers: u64) -> ClassSourceReport {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut Budget::new(limits(4096)),
        )
        .expect("original DT-29 opens");
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
    let OperationOutcome::Performed(report) = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits(headers)),
        )
        .expect("selected DT-29 returns its public report")
    else {
        panic!("the selected original class is unique");
    };
    report
}

fn physical_set(report: &ClassSourceReport) -> Option<&ClassSourceMethod> {
    physical_set_with_descriptor(report, b"(ZZ)V")
}

fn physical_set_with_descriptor<'a>(
    report: &'a ClassSourceReport,
    descriptor: &[u8],
) -> Option<&'a ClassSourceMethod> {
    report.methods.iter().find(|method| {
        method.item.identity.name.0 == b"set" && method.item.identity.descriptor.0 == descriptor
    })
}

#[test]
fn dt29_wide_preserves_parent_owner_casts_and_physical_accessor_call() {
    let report = present(4096);
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(report.methods.len(), 2);
    let member = physical_set(&report).expect("set(ZZ)V is the actual physical method");
    assert_eq!(member.item.index, 1);
    let ClassSourceOutcome::Recovered {
        report: body,
        analysis,
    } = &member.outcome
    else {
        panic!("the wide set body is recovered");
    };
    assert!(matches!(
        analysis.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(matches!(body.execution, ExecutionReport::Complete { .. }));
    assert_eq!(body.content, RecoveryContent::ContainsStatements);
    assert!(body.fallbacks.is_empty());
    for field in ["publicField", "protectedField", "packagePrivateField"] {
        assert!(member.text.contains(&format!(
            "((dt29.SamePackageParentFamily$A) this).{field} = arg1;"
        )));
    }
    assert!(member.text.contains(
        "dt29.SamePackageParentFamily$A.access$002((dt29.SamePackageParentFamily$A) this, arg2);"
    ));
}

fn assert_header_stop(cap: u64) {
    let wide = present(4096);
    let report = present(cap);
    assert_header_stop_reports(&wide, &report, b"(ZZ)V");
}

fn assert_header_stop_reports(
    wide: &ClassSourceReport,
    report: &ClassSourceReport,
    descriptor: &[u8],
) {
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: jarde_reader::budget::BudgetDimension::ClassHeaders,
                },
                ..
            }
        ),
        "actual class execution: {:?}",
        report.execution
    );
    let constructor = report
        .methods
        .first()
        .expect("the admitted constructor prefix remains");
    assert_eq!(constructor.item.index, 0);
    assert_eq!(constructor.item.identity.name.0, b"<init>");
    assert_eq!(constructor.item.identity.descriptor.0, b"()V");
    assert_eq!(constructor.text, wide.methods[0].text);
    let ClassSourceOutcome::Recovered {
        report: ctor,
        analysis,
    } = &constructor.outcome
    else {
        panic!("the prefix keeps its actual completed run");
    };
    let ClassSourceOutcome::Recovered {
        report: wide_ctor, ..
    } = &wide.methods[0].outcome
    else {
        panic!("wide constructor is recovered");
    };
    assert!(matches!(
        analysis.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(matches!(ctor.execution, ExecutionReport::Complete { .. }));
    assert_eq!(ctor.source_map.segments(), wide_ctor.source_map.segments());
    // A plain local header refusal leaves the wide ResultItems budget available.
    // Ledger in-flight cases cover the separate unpaid-publication boundary.
    let member = physical_set_with_descriptor(report, descriptor)
        .expect("the stopped setter must be admitted by this plain local budget");
    assert_eq!(member.item.index, 1);
    let ClassSourceOutcome::Recovered {
        report: body,
        analysis,
    } = &member.outcome
    else {
        panic!("a published stop member retains its typed recovery outcome");
    };
    assert!(matches!(
        analysis.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(matches!(
        body.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: jarde_reader::budget::BudgetDimension::ClassHeaders,
            },
            ..
        }
    ));
    assert!(matches!(
        body.outcome,
        jarde::RecoveryOutcome::Stopped(jarde::StopReason::Interrupted {
            code: "jre_budget_interrupted",
            at: None
        })
    ));
    assert_eq!(body.content, RecoveryContent::NotProduced);
    assert!(body.text.is_empty());
    assert!(body.source_map.segments().is_empty());
    assert!(body.artifact.binding().is_none());
    let body_diagnostics = body.diagnostics.as_slice();

    assert!(
        report
            .diagnostics
            .iter()
            .chain(body_diagnostics)
            .any(|diagnostic| { diagnostic.message.contains("ClassHeaders") }),
        "the original stop diagnostic survives in aggregate or admitted member"
    );
    assert!(
        report
            .coverage
            .artifact_structural
            .scanned
            .iter()
            .any(|range| { range.label == "class_source_bodies" && range.end > 1 })
    );
    assert!(
        !report
            .coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| { range.label == "class_source_bodies" && range.start > 1 }),
        "set is last; do not invent a physical skipped suffix"
    );
}

#[test]
fn dt29_header_cap1_stops_at_the_first_direct_parent_dependency() {
    assert_header_stop(1);
}

#[test]
fn dt29_header_cap2_preserves_the_direct_parent_field_header_stop() {
    assert_header_stop(2);
}

#[test]
fn external_receiver_source_header_stop_is_not_treated_as_an_absent_relation() {
    // The committed family has C.set(B,Z)V: current IR declares C, the receiver is B,
    // and the physical accessor owner is A. B's header must be read from the snapshot.
    let class = "dt29/FieldCast$C";
    let descriptor = b"(Ldt29/FieldCast$B;Z)V";
    let wide = present_class(EXTERNAL_SOURCE, class, 4096);
    assert!(matches!(wide.execution, ExecutionReport::Complete { .. }));
    let member = physical_set_with_descriptor(&wide, descriptor)
        .expect("the committed external-source setter is selected");
    let ClassSourceOutcome::Recovered { analysis, .. } = &member.outcome else {
        panic!("wide source control retains the actual completed analysis");
    };
    let ExecutionReport::Complete { usage } = &analysis.execution else {
        panic!("source control reaches a completed method analysis");
    };
    // Use the real public analysis prefix price, not an invented fixture header total.
    // This method contains no `new`: the legacy reference-overload proof does no reads.
    // The scoped source-read observation described with this candidate must additionally
    // verify that the first refused read is B, inside accessor proof, before field proof.
    let cap = usage.class_headers;
    assert!(cap > 0);
    let report = present_class(EXTERNAL_SOURCE, class, cap);
    assert_header_stop_reports(&wide, &report, descriptor);
}
