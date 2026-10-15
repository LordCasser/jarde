//! Public regression for request stops while proving class-source dependencies.
use jarde::*;
use rawzip::{CompressionMethod, ZipArchive, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Read, Write};
use std::slice;

const H1: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/user-hierarchy-widening-patrol/fixture/hier/hier-fam.jar"
);

fn limits(class_headers: u64) -> Limits {
    let mut limits = task_budget(&[])
        .expect("default budget is valid")
        .limits()
        .clone();
    limits.elapsed_millis = u64::MAX;
    limits.class_headers = class_headers;
    limits
}

fn class_source(class_headers: u64) -> ClassSourceReport {
    class_source_with_evidence(class_headers, &RecoveryEvidenceRequest::essential())
}

fn class_source_with_evidence(
    class_headers: u64,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    class_source_bytes_with_evidence(H1, class_headers, evidence)
}

fn class_source_bytes_with_evidence(
    jar: &[u8],
    class_headers: u64,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let engine = Engine::new();
    let mut open_budget = Budget::new(limits(4096));
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.to_vec()), &mut open_budget)
        .expect("class-source archive opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("H1"),
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
    let mut budget = Budget::new(limits(class_headers));
    let OperationOutcome::Performed(report) = engine
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, evidence, &mut budget)
        .expect("selected H1 produces a public class-source report")
    else {
        panic!("H1 must resolve to one physical class");
    };
    report
}

fn read_verified<R: Read>(entry: &rawzip::ZipSliceEntry<'_>, reader: R) -> Vec<u8> {
    let mut verified = entry.verifying_reader(reader);
    let mut decoded = Vec::new();
    verified
        .read_to_end(&mut decoded)
        .expect("H1 class CRC and bytes verify");
    decoded
}

const H1_CLASS: &[u8] = b"H1.class";
const MID_CLASS: &[u8] = b"H1$Mid.class";

/// Read the pinned fixture's physical class records, checking each actual compression method.
fn h1_class_records() -> Vec<(Vec<u8>, Vec<u8>)> {
    let archive = ZipArchive::from_slice(H1).expect("pinned H1 fixture is a ZIP");
    let mut headers = archive.entries();
    let mut records = Vec::new();
    while let Some(header) = headers.next_entry().expect("H1 ZIP entry header reads") {
        let file_path = header.file_path();
        let name = file_path.as_ref();
        if !name.ends_with(b".class") {
            continue;
        }
        let name = name.to_vec();
        let entry = archive
            .get_entry(header.wayfinder())
            .expect("H1 entry resolves");
        let decoded = match header.compression_method() {
            CompressionMethod::STORE => read_verified(&entry, Cursor::new(entry.data())),
            CompressionMethod::DEFLATE => {
                read_verified(&entry, flate2::bufread::DeflateDecoder::new(entry.data()))
            }
            method => panic!("unsupported pinned H1 compression method: {method:?}"),
        };
        records.push((name, decoded));
    }
    records
}

// TwoLevel needs the intermediate Mid header; reaching the terminal Greet name needs no read.
fn derived_h1(missing_mid: bool) -> Vec<u8> {
    let records = h1_class_records();
    assert_eq!(
        records.iter().filter(|(name, _)| name == H1_CLASS).count(),
        1,
        "the H1 target remains a unique physical entry"
    );
    assert_eq!(
        records.iter().filter(|(name, _)| name == MID_CLASS).count(),
        1,
        "the frozen source has one physical Mid dependency"
    );
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = ZipArchiveWriter::new(&mut output);
        let mut mid_records = 0;
        for (name, bytes) in &records {
            if missing_mid && name.as_slice() == MID_CLASS {
                mid_records += 1;
                continue;
            }
            write_class_record(&mut writer, name.clone(), bytes);
            if name.as_slice() == MID_CLASS {
                mid_records += 1;
                write_class_record(&mut writer, name.clone(), bytes);
            }
        }
        assert_eq!(
            mid_records, 1,
            "the intended dependency record was encountered once"
        );
        writer.finish().expect("derived H1 ZIP closes");
    }
    let bytes = output.into_inner();
    let derived = ZipArchive::from_slice(&bytes).expect("derived H1 archive is valid ZIP");
    let mut entries = derived.entries();
    let mut target_count = 0;
    let mut mid_count = 0;
    while let Some(header) = entries
        .next_entry()
        .expect("derived ZIP entry header reads")
    {
        let name = header.file_path();
        if name.as_ref() == H1_CLASS {
            target_count += 1;
        }
        if name.as_ref() == MID_CLASS {
            mid_count += 1;
        }
    }
    assert_eq!(
        target_count, 1,
        "derivation never removes or duplicates H1.class"
    );
    assert_eq!(
        mid_count,
        if missing_mid { 0 } else { 2 },
        "the derived dependency shape is explicit"
    );
    bytes
}

fn write_class_record(
    writer: &mut ZipArchiveWriter<&mut Cursor<Vec<u8>>>,
    name: Vec<u8>,
    bytes: &[u8],
) {
    let (mut entry, config) = writer
        .new_file(EntryPath::verbatim(name))
        .compression_method(CompressionMethod::STORE)
        .start()
        .expect("derived class entry starts");
    let mut sink = config.wrap(&mut entry);
    sink.write_all(bytes).expect("derived class bytes write");
    let (_, descriptor) = sink.finish().expect("derived class entry closes");
    entry
        .finish(descriptor)
        .expect("derived class record finishes");
}

fn assert_ordinary_mid_refusal(report: &ClassSourceReport) {
    assert!(
        matches!(&report.execution, ExecutionReport::Complete { .. }),
        "a dependency refusal does not stop the H1 request: {:?}",
        report.execution
    );
    assert!(
        report.text.contains("public static void main("),
        "H1 main stays presented"
    );
    let main = report
        .methods
        .iter()
        .find(|method| {
            method.item.identity.name.0 == b"main"
                && method.item.identity.descriptor.0 == b"([Ljava/lang/String;)V"
        })
        .expect("physical H1.main remains");
    let ClassSourceOutcome::Recovered {
        report: body,
        analysis,
    } = &main.outcome
    else {
        panic!("H1.main keeps its ordinary recovery result");
    };
    assert!(matches!(
        &analysis.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(
        body.produced(),
        "the unrelated H1.main body remains recovered"
    );
    assert!(
        body.text.contains("no safe reference conversion evidence"),
        "the unresolved call stays explicitly refused: {}",
        body.text
    );
}

#[test]
fn missing_physical_mid_refuses_only_the_call_without_stopping_class_source() {
    let jar = derived_h1(true);
    let report =
        class_source_bytes_with_evidence(&jar, 4096, &RecoveryEvidenceRequest::essential());
    assert_ordinary_mid_refusal(&report);
    assert!(
        report.text.contains("H1$Greet"),
        "the refusal identifies the required Greet type"
    );
}

#[test]
fn duplicate_physical_mid_refuses_ambiguous_evidence_without_stopping_class_source() {
    let jar = derived_h1(false);
    let report =
        class_source_bytes_with_evidence(&jar, 4096, &RecoveryEvidenceRequest::essential());
    assert_ordinary_mid_refusal(&report);
    assert!(
        report.text.contains("H1$Greet"),
        "the refusal retains the required Greet target type"
    );
}

fn stopped_for_class_headers(report: &ExecutionReport) -> bool {
    matches!(
        report,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders
            },
            ..
        }
    )
}

#[test]
fn h1_wide_budget_remains_complete_with_main_body() {
    let report = class_source(4096);
    assert!(matches!(
        &report.execution,
        ExecutionReport::Complete { .. }
    ));
    let main = report
        .methods
        .iter()
        .find(|method| method.item.identity.name.0 == b"main")
        .unwrap();
    assert!(
        matches!(&main.outcome, ClassSourceOutcome::Recovered { report, .. } if report.produced() && report.content == RecoveryContent::ContainsStatements)
    );
    assert!(report.text.contains("public static void main("));
}

#[test]
fn h1_cap_one_keeps_ordinary_class_header_budget_refusal() {
    let report = class_source(1);
    assert!(stopped_for_class_headers(&report.execution));
    assert!(report.methods.iter().any(|method| matches!(&method.outcome,
        ClassSourceOutcome::Refused { execution, .. } if stopped_for_class_headers(execution))));
}

#[test]
fn h1_cap_two_keeps_ordinary_class_header_budget_refusal() {
    let report = class_source(2);
    assert!(stopped_for_class_headers(&report.execution));
    assert!(report.methods.iter().any(|method| matches!(&method.outcome,
        ClassSourceOutcome::Refused { execution, .. } if stopped_for_class_headers(execution))));
}

fn assert_prebody_stop(cap: u64) {
    let report = class_source(cap);
    assert!(
        stopped_for_class_headers(&report.execution),
        "{:?}",
        report.execution
    );
    let main = report
        .methods
        .iter()
        .find(|method| method.item.identity.name.0 == b"main")
        .unwrap();
    let ClassSourceOutcome::Recovered {
        report: body,
        analysis,
    } = &main.outcome
    else {
        panic!("main retains the analysis and recovery result");
    };
    assert!(matches!(
        &analysis.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(
        stopped_for_class_headers(&body.execution),
        "{:?}",
        body.execution
    );
    assert!(matches!(&body.outcome, RecoveryOutcome::Stopped(_)));
    assert_eq!(body.content, RecoveryContent::NotProduced);
    assert!(body.text.is_empty());
    assert!(body.source_map.is_empty());
    assert!(body.declaration.is_none());
    assert!(body.artifact.binding().is_none());
    assert!(
        main.markers
            .iter()
            .any(|marker| marker.contains("not recovered")
                && marker.contains("budget_exceeded_class_headers")),
        "{:?}",
        main.markers
    );
}

#[test]
fn h1_cap_eight_stops_snapshot_proof_before_main_body() {
    assert_prebody_stop(8);
}

#[test]
fn h1_cap_fourteen_stops_snapshot_proof_before_main_body() {
    assert_prebody_stop(14);
}

#[test]
fn h1_cap_fifteen_keeps_main_body_but_stops_class_projection() {
    let report = class_source_with_evidence(15, &RecoveryEvidenceRequest::all());
    assert!(
        stopped_for_class_headers(&report.execution),
        "{:?}",
        report.execution
    );
    let main = report
        .methods
        .iter()
        .find(|method| method.item.identity.name.0 == b"main")
        .unwrap();
    let ClassSourceOutcome::Recovered { report: body, .. } = &main.outcome else {
        panic!("main body remains committed");
    };
    assert!(body.produced());
    assert_eq!(body.content, RecoveryContent::ContainsStatements);
    assert!(report.text.contains("public static void main("));
    assert!(report.text.contains("System.out"));
    let wide = class_source_with_evidence(4096, &RecoveryEvidenceRequest::all());
    let wide_main = wide
        .methods
        .iter()
        .find(|method| method.item.identity.name.0 == b"main")
        .unwrap();
    let ClassSourceOutcome::Recovered {
        report: wide_body, ..
    } = &wide_main.outcome
    else {
        panic!("wide H1.main has recovery");
    };
    assert!(
        !wide_body.source_map.is_empty(),
        "map preservation requires an actually requested nonempty map"
    );
    assert!(!body.source_map.is_empty());
    assert_eq!(report.methods.len(), wide.methods.len());
    for (actual, expected) in report.methods.iter().zip(&wide.methods) {
        assert_eq!(actual.item.identity, expected.item.identity);
        assert_eq!(actual.item.index, expected.item.index);
        assert_eq!(actual.text, expected.text);
        assert_eq!(
            without_elapsed(serde_json::to_value(&actual.outcome).unwrap()),
            without_elapsed(serde_json::to_value(&expected.outcome).unwrap())
        );
    }
}

fn without_elapsed(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(fields) => serde_json::Value::Object(
            fields
                .into_iter()
                .filter(|(key, _)| key != "elapsed_millis")
                .map(|(key, value)| (key, without_elapsed(value)))
                .collect(),
        ),
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(without_elapsed).collect())
        }
        other => other,
    }
}
