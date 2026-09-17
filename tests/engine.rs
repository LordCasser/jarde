use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::sync::Arc;

fn limits(value: u64) -> Limits {
    Limits {
        input_bytes: value,
        archive_entries: value,
        entry_bytes: value,
        read_bytes: value,
        class_bytes: value,
        attribute_bytes: value,
        code_bytes: value,
        result_items: value,
        output_bytes: value,
        nested_depth: value,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

fn counted_usage(budget: &Budget) -> UsageSnapshot {
    let mut usage = budget.usage();
    usage.elapsed_millis = 0;
    usage
}
fn u16b(v: &mut Vec<u8>, n: u16) {
    v.extend_from_slice(&n.to_be_bytes())
}
fn u32b(v: &mut Vec<u8>, n: u32) {
    v.extend_from_slice(&n.to_be_bytes())
}
fn utf8(v: &mut Vec<u8>, s: &[u8]) {
    v.push(1);
    u16b(v, s.len() as u16);
    v.extend_from_slice(s)
}
fn class(code: &[u8]) -> Vec<u8> {
    class_with_handlers(code, &[])
}
fn class_with_handlers(code: &[u8], handlers: &[(u16, u16, u16, u16)]) -> Vec<u8> {
    let mut v = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut v, 0);
    u16b(&mut v, 52);
    u16b(&mut v, 8);
    utf8(&mut v, b"Test");
    v.push(7);
    u16b(&mut v, 1);
    utf8(&mut v, b"java/lang/Object");
    v.push(7);
    u16b(&mut v, 3);
    utf8(&mut v, b"run");
    utf8(&mut v, b"()V");
    utf8(&mut v, b"Code");
    u16b(&mut v, 0x21);
    u16b(&mut v, 2);
    u16b(&mut v, 4);
    u16b(&mut v, 0);
    u16b(&mut v, 0);
    u16b(&mut v, 1);
    u16b(&mut v, 9);
    u16b(&mut v, 5);
    u16b(&mut v, 6);
    u16b(&mut v, 1);
    let mut c = Vec::new();
    u16b(&mut c, 1);
    u16b(&mut c, 0);
    u32b(&mut c, code.len() as u32);
    c.extend_from_slice(code);
    u16b(&mut c, handlers.len() as u16);
    for &(start, end, handler, catch_type) in handlers {
        u16b(&mut c, start);
        u16b(&mut c, end);
        u16b(&mut c, handler);
        u16b(&mut c, catch_type);
    }
    u16b(&mut c, 0);
    u16b(&mut v, 7);
    u32b(&mut v, c.len() as u32);
    v.extend_from_slice(&c);
    u16b(&mut v, 0);
    v
}
fn zip(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for &(name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(data).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        archive.finish().unwrap();
    }
    output.into_inner()
}
fn selector() -> MethodSelector {
    MethodSelector {
        name: JvmBytes(b"run".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    }
}

fn named_selector(name: &[u8]) -> MethodSelector {
    MethodSelector {
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    }
}

fn two_method_class() -> Vec<u8> {
    let mut output = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0);
    u16b(&mut output, 52);
    u16b(&mut output, 9);
    utf8(&mut output, b"Test");
    output.push(7);
    u16b(&mut output, 1);
    utf8(&mut output, b"java/lang/Object");
    output.push(7);
    u16b(&mut output, 3);
    utf8(&mut output, b"wanted");
    utf8(&mut output, b"()V");
    utf8(&mut output, b"Code");
    utf8(&mut output, b"unwanted");
    u16b(&mut output, 0x21);
    u16b(&mut output, 2);
    u16b(&mut output, 4);
    u16b(&mut output, 0);
    u16b(&mut output, 0);
    u16b(&mut output, 2);
    for (name_index, code) in [(5, &[0x00, 0xb1][..]), (8, &[0x00, 0xcb][..])] {
        u16b(&mut output, 0x0009);
        u16b(&mut output, name_index);
        u16b(&mut output, 6);
        u16b(&mut output, 1);
        let mut content = Vec::new();
        u16b(&mut content, 1);
        u16b(&mut content, 0);
        u32b(&mut content, code.len() as u32);
        content.extend_from_slice(code);
        u16b(&mut content, 0);
        u16b(&mut content, 0);
        u16b(&mut output, 7);
        u32b(&mut output, content.len() as u32);
        output.extend_from_slice(&content);
    }
    u16b(&mut output, 0);
    output
}

#[test]
fn standalone_engine_reports_owned_source_and_usage() {
    let bytes = class(&[0xb1]);
    let e = Engine::new();
    let mut b = Budget::new(limits(u64::MAX));
    let s = e
        .open(
            ArtifactInput::bytes(Arc::<[u8]>::from(bytes.clone())),
            &mut b,
        )
        .unwrap();
    let r = e
        .inspect_header(&s, ClassTarget::Root, &mut b, InspectionMode::Strict)
        .unwrap();
    assert!(r.source.entry().is_none());
    assert_eq!(r.source.snapshot(), s.id());
    assert_eq!(r.source.class_bytes.length, bytes.len() as u64);
    assert_eq!(
        r.source.class_bytes.digest.0,
        blake3::hash(&bytes).to_hex().to_string()
    );
    assert!(matches!(r.execution,ExecutionReport::Complete{ref usage}if *usage==b.usage()));
    assert_eq!(
        r.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(
        r.coverage.artifact_structural.scanned,
        vec![CoverageRange {
            label: "class_header_schema".into(),
            start: 0,
            end: bytes.len() as u64
        }]
    );
    assert!(r.coverage.artifact_structural.skipped.is_empty());
    assert_eq!(
        r.coverage.runtime_resolution.state,
        CoverageState::NotRequested
    );
    assert_eq!(
        r.coverage.dynamic_analysis.state,
        CoverageState::NotRequested
    );
    assert_eq!(
        serde_json::from_str::<EngineHeaderReport>(&serde_json::to_string(&r).unwrap()).unwrap(),
        r
    );
    let mut b = Budget::new(limits(u64::MAX));
    let s = e
        .open(ArtifactInput::bytes(Arc::<[u8]>::from(bytes)), &mut b)
        .unwrap();
    let r = e
        .inspect_method_bytecode(&s, ClassTarget::Root, selector(), &mut b)
        .unwrap();
    assert!(
        matches!(r.inspection.execution,ExecutionReport::Complete{ref usage}if *usage==b.usage())
    );
    assert_eq!(
        r.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(
        r.coverage.artifact_structural.scanned,
        vec![
            CoverageRange {
                label: "method_code_bci".into(),
                start: 0,
                end: 1
            },
            CoverageRange {
                label: "exception_handler_ordinal".into(),
                start: 0,
                end: 0
            },
        ]
    );
    assert!(r.coverage.artifact_structural.skipped.is_empty());
    assert_eq!(
        serde_json::from_str::<EngineBytecodeReport>(&serde_json::to_string(&r).unwrap()).unwrap(),
        r
    );
}

#[test]
fn empty_zip_enumeration_reports_the_open_snapshot() {
    let e = Engine::new();
    let mut b = Budget::new(limits(u64::MAX));
    let s = e
        .open(ArtifactInput::bytes(Arc::<[u8]>::from(zip(&[]))), &mut b)
        .unwrap();
    let report = e.enumerate(&s, &mut b).unwrap();

    assert_eq!(&report.snapshot, s.id());
    assert!(report.entries.is_empty());
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        serde_json::from_str::<EnumerationReport>(&serde_json::to_string(&report).unwrap())
            .unwrap(),
        report
    );
}

#[test]
fn zip_duplicate_origins_and_validation_are_preserved() {
    let c = class(&[0xb1]);
    let z = zip(&[(b"A.class", &c), (b"A.class", &c)]);
    let e = Engine::new();
    let mut b = Budget::new(limits(u64::MAX));
    let s = e
        .open(ArtifactInput::bytes(Arc::<[u8]>::from(z)), &mut b)
        .unwrap();
    assert!(
        matches!(e.inspect_header(&s,ClassTarget::Root,&mut b,InspectionMode::Strict),Err(Error::InvalidInput{ref code,..})if code=="class_target_root_on_zip")
    );
    let entries = e.enumerate(&s, &mut b).unwrap().entries;
    let a = e
        .inspect_header(
            &s,
            ClassTarget::Entry(&entries[0]),
            &mut b,
            InspectionMode::Strict,
        )
        .unwrap();
    let d = e
        .inspect_header(
            &s,
            ClassTarget::Entry(&entries[1]),
            &mut b,
            InspectionMode::Strict,
        )
        .unwrap();
    assert_ne!(a.source.entry(), d.source.entry());
    assert_eq!(a.source.class_bytes, d.source.class_bytes);
    let mut forged = entries[0].clone();
    forged.id.raw_name.0 = b"bad".to_vec();
    assert!(
        matches!(e.inspect_header(&s,ClassTarget::Entry(&forged),&mut b,InspectionMode::Strict),Err(Error::InvalidInput{ref code,..})if code=="entry_locator_mismatch")
    );
}

#[test]
fn handler_budget_partial_has_precise_coverage() {
    let c = class_with_handlers(&[0xb1], &[(0, 1, 0, 0), (0, 1, 0, 2)]);
    let e = Engine::new();
    let mut open_budget = Budget::new(limits(u64::MAX));
    let s = e
        .open(ArtifactInput::bytes(Arc::<[u8]>::from(c)), &mut open_budget)
        .unwrap();
    let mut constrained = limits(u64::MAX);
    constrained.result_items = 2; // report root + first handler
    let r = e
        .inspect_method_bytecode(
            &s,
            ClassTarget::Root,
            selector(),
            &mut Budget::new(constrained),
        )
        .unwrap();
    let stop = r.inspection.stopped_at.as_ref().unwrap();
    assert_eq!(stop.phase(), BytecodeStopPhase::ExceptionHandlers);
    let expected_offset = r
        .inspection
        .code_span
        .start
        .checked_add(r.inspection.code_span.length)
        .and_then(|offset| offset.checked_add(2))
        .and_then(|offset| offset.checked_add(8))
        .unwrap();
    assert!(matches!(
        stop,
        BytecodeStop::ExceptionHandlers {
            ordinal: 1,
            class_offset,
            ..
        } if *class_offset == expected_offset
    ));
    assert_eq!(r.inspection.exception_handler_count, 2);
    assert_eq!(r.inspection.exception_handlers.len(), 1);
    assert!(r.inspection.instructions.is_empty());
    assert_eq!(r.coverage.artifact_structural.state, CoverageState::Partial);
    assert_eq!(
        r.coverage.artifact_structural.scanned,
        vec![
            CoverageRange {
                label: "method_code_bci".into(),
                start: 0,
                end: 0
            },
            CoverageRange {
                label: "exception_handler_ordinal".into(),
                start: 0,
                end: 1
            },
        ]
    );
    assert_eq!(
        r.coverage.artifact_structural.skipped,
        vec![
            CoverageRange {
                label: "method_code_bci".into(),
                start: 0,
                end: 1
            },
            CoverageRange {
                label: "exception_handler_ordinal".into(),
                start: 1,
                end: 2
            },
        ]
    );
}

#[test]
fn target_and_header_body_boundaries_are_preserved() {
    let e = Engine::new();
    let c = class(&[0xcb]);
    let mut budget = Budget::new(limits(u64::MAX));
    let standalone = e
        .open(
            ArtifactInput::bytes(Arc::<[u8]>::from(c.clone())),
            &mut budget,
        )
        .unwrap();
    let fake = PhysicalEntry {
        id: PhysicalEntryId {
            origin: ContainerOrigin {
                snapshot: standalone.id().clone(),
                root_container: ContainerId("root".into()),
                steps: Vec::new(),
            },
            ordinal: 0,
            raw_name: ArchiveNameBytes(b"A.class".to_vec()),
        },
        compression: EntryCompression::Stored,
        compression_method: 0,
        flags: EntryFlags {
            raw_bits: 0,
            encrypted: false,
            strong_encryption: false,
            data_descriptor: false,
        },
        crc32: 0,
        compressed_size: 0,
        uncompressed_size: 0,
        layout: EntryLayout {
            local_header_offset: 0,
            central_header_offset: 0,
            compressed_data: ByteSpan::new(0, 0),
        },
        nested_archive: NestedArchiveState::NotCandidate,
        signature_metadata: None,
    };
    assert!(
        matches!(e.inspect_header(&standalone, ClassTarget::Entry(&fake), &mut budget, InspectionMode::Strict), Err(Error::InvalidInput { ref code, .. }) if code == "class_target_entry_on_class")
    );
    assert!(
        e.inspect_header(
            &standalone,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict
        )
        .is_ok()
    );

    let z1 = zip(&[(b"A.class", &c)]);
    let z2 = zip(&[(b"B.class", &c)]);
    let s1 = e
        .open(ArtifactInput::bytes(Arc::<[u8]>::from(z1)), &mut budget)
        .unwrap();
    let s2 = e
        .open(ArtifactInput::bytes(Arc::<[u8]>::from(z2)), &mut budget)
        .unwrap();
    let entry = e.enumerate(&s1, &mut budget).unwrap().entries.remove(0);
    assert!(
        matches!(e.inspect_header(&s2, ClassTarget::Entry(&entry), &mut budget, InspectionMode::Strict), Err(Error::InvalidInput { ref code, .. }) if code == "entry_snapshot_mismatch")
    );
}

#[test]
fn materialization_failure_is_err_and_body_partial_is_preserved() {
    let c = class(&[0x00, 0xcb]);
    let z = zip(&[(b"A.class", &c)]);
    let e = Engine::new();
    let mut b = Budget::new(limits(u64::MAX));
    let s = e
        .open(ArtifactInput::bytes(Arc::<[u8]>::from(z)), &mut b)
        .unwrap();
    let entry = e.enumerate(&s, &mut b).unwrap().entries.remove(0);
    let mut small = limits(u64::MAX);
    small.entry_bytes = 0;
    assert!(matches!(
        e.inspect_method_bytecode(
            &s,
            ClassTarget::Entry(&entry),
            selector(),
            &mut Budget::new(small)
        ),
        Err(Error::BudgetExceeded {
            dimension: BudgetDimension::EntryBytes,
            ..
        })
    ));
    let mut b = Budget::new(limits(u64::MAX));
    let r = e
        .inspect_method_bytecode(&s, ClassTarget::Entry(&entry), selector(), &mut b)
        .unwrap();
    assert!(
        matches!(r.inspection.execution,ExecutionReport::Partial{ref usage,..}if *usage==b.usage())
    );
    assert_eq!(r.inspection.instructions.len(), 1);
    assert!(matches!(
        r.inspection.stopped_at.as_ref().unwrap(),
        BytecodeStop::Instructions { bci: 1, .. }
    ));
    assert_eq!(r.coverage.artifact_structural.state, CoverageState::Partial);
    assert_eq!(
        r.coverage.artifact_structural.scanned,
        vec![
            CoverageRange {
                label: "method_code_bci".into(),
                start: 0,
                end: 1
            },
            CoverageRange {
                label: "exception_handler_ordinal".into(),
                start: 0,
                end: 0
            },
        ]
    );
    assert_eq!(
        r.coverage.artifact_structural.skipped,
        vec![CoverageRange {
            label: "method_code_bci".into(),
            start: 1,
            end: 2
        }]
    );
}

#[test]
fn public_engine_entrypoints_honor_precancelled_requests_at_their_stage_boundary() {
    let engine = Engine::new();
    let class_bytes = class(&[0xb1]);

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(u64::MAX), token);
    assert!(matches!(
        engine.open(
            ArtifactInput::bytes(Arc::<[u8]>::from(class_bytes.clone())),
            &mut cancelled
        ),
        Err(Error::Cancelled { .. })
    ));
    assert_eq!(cancelled.usage().input_bytes, 0);

    let mut setup = Budget::new(limits(u64::MAX));
    let standalone = engine
        .open(
            ArtifactInput::bytes(Arc::<[u8]>::from(class_bytes.clone())),
            &mut setup,
        )
        .unwrap();
    for inspect in [true, false] {
        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(limits(u64::MAX), token);
        let result = if inspect {
            engine
                .inspect_header(
                    &standalone,
                    ClassTarget::Root,
                    &mut cancelled,
                    InspectionMode::Strict,
                )
                .map(|_| ())
        } else {
            engine
                .inspect_method_bytecode(&standalone, ClassTarget::Root, selector(), &mut cancelled)
                .map(|_| ())
        };
        assert!(matches!(result, Err(Error::Cancelled { .. })));
        assert_eq!(counted_usage(&cancelled), UsageSnapshot::default());
    }

    let archive = zip(&[(b"Test.class", &class_bytes)]);
    let mut setup = Budget::new(limits(u64::MAX));
    let zip_snapshot = engine
        .open(ArtifactInput::bytes(Arc::<[u8]>::from(archive)), &mut setup)
        .unwrap();
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(u64::MAX), token);
    let report = engine.enumerate(&zip_snapshot, &mut cancelled).unwrap();
    assert!(report.entries.is_empty());
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        report.coverage.artifact_structural.scanned,
        vec![CoverageRange {
            label: "central_directory_entries".into(),
            start: 0,
            end: 0,
        }]
    );
    assert_eq!(report.coverage.artifact_structural.skipped[0].start, 0);
    assert_eq!(report.coverage.artifact_structural.skipped[0].end, 1);
    assert_eq!(counted_usage(&cancelled), UsageSnapshot::default());
}

#[test]
fn one_budget_accumulates_open_and_header_and_preserves_atomic_stage_failure() {
    let bytes = class(&[0xb1]);
    let length = bytes.len() as u64;
    let engine = Engine::new();
    let mut exact = limits(u64::MAX);
    exact.input_bytes = length;
    exact.read_bytes = length;
    exact.output_bytes = length;
    exact.class_bytes = length;
    exact.attribute_bytes = 19;
    exact.result_items = 3;
    let mut budget = Budget::new(exact);
    let snapshot = engine
        .open(
            ArtifactInput::bytes(Arc::<[u8]>::from(bytes.clone())),
            &mut budget,
        )
        .unwrap();
    let report = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .unwrap();
    assert!(matches!(
        report.execution,
        ExecutionReport::Complete { ref usage } if *usage == budget.usage()
    ));
    assert_eq!(
        counted_usage(&budget),
        UsageSnapshot {
            input_bytes: length,
            read_bytes: length,
            output_bytes: length,
            class_bytes: length,
            attribute_bytes: 19,
            result_items: 3,
            ..UsageSnapshot::default()
        }
    );

    let mut failing = limits(u64::MAX);
    failing.class_bytes = length - 1;
    let mut budget = Budget::new(failing);
    let snapshot = engine
        .open(ArtifactInput::bytes(Arc::<[u8]>::from(bytes)), &mut budget)
        .unwrap();
    let error = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .unwrap_err();
    assert!(matches!(
        error,
        Error::BudgetExceeded {
            dimension: BudgetDimension::ClassBytes,
            consumed: 0,
            requested,
            ..
        } if requested == length
    ));
    let usage = budget.usage();
    assert_eq!(usage.input_bytes, length);
    assert_eq!(usage.read_bytes, length);
    assert_eq!(usage.output_bytes, length);
    assert_eq!(usage.class_bytes, 0);
    assert_eq!(usage.attribute_bytes, 0);
    assert_eq!(usage.code_bytes, 0);
    assert_eq!(usage.result_items, 0);
}

#[test]
fn method_request_decodes_and_charges_only_the_selected_body() {
    let bytes = two_method_class();
    let length = bytes.len() as u64;
    let engine = Engine::new();
    let mut setup = Budget::new(limits(u64::MAX));
    let snapshot = engine
        .open(ArtifactInput::bytes(Arc::<[u8]>::from(bytes)), &mut setup)
        .unwrap();

    let mut header_budget = Budget::new(limits(u64::MAX));
    let header = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut header_budget,
            InspectionMode::Strict,
        )
        .unwrap();
    assert_eq!(header.inspection.header.methods.len(), 2);
    assert_eq!(header_budget.usage().class_bytes, length);
    assert_eq!(header_budget.usage().attribute_bytes, 40);
    assert_eq!(header_budget.usage().code_bytes, 0);
    assert_eq!(header_budget.usage().result_items, 5);

    let mut wanted_budget = Budget::new(limits(u64::MAX));
    let wanted = engine
        .inspect_method_bytecode(
            &snapshot,
            ClassTarget::Root,
            named_selector(b"wanted"),
            &mut wanted_budget,
        )
        .unwrap();
    assert!(matches!(
        wanted.inspection.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        wanted
            .inspection
            .instructions
            .iter()
            .map(|fact| (fact.bci, fact.opcode, fact.width))
            .collect::<Vec<_>>(),
        vec![(0, 0x00, 1), (1, 0xb1, 1)]
    );
    assert!(wanted.inspection.diagnostics.is_empty());
    assert_eq!(wanted_budget.usage().class_bytes, length);
    assert_eq!(wanted_budget.usage().attribute_bytes, 20);
    assert_eq!(wanted_budget.usage().code_bytes, 2);
    assert_eq!(wanted_budget.usage().result_items, 3);

    let mut unwanted_budget = Budget::new(limits(u64::MAX));
    let unwanted = engine
        .inspect_method_bytecode(
            &snapshot,
            ClassTarget::Root,
            named_selector(b"unwanted"),
            &mut unwanted_budget,
        )
        .unwrap();
    assert!(matches!(
        unwanted.inspection.execution,
        ExecutionReport::Partial { .. }
    ));
    assert_eq!(unwanted.inspection.instructions.len(), 1);
    assert!(matches!(
        unwanted.inspection.stopped_at,
        Some(BytecodeStop::Instructions { bci: 1, .. })
    ));
    assert_eq!(unwanted.inspection.diagnostics.len(), 1);
    assert_eq!(
        unwanted.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(unwanted_budget.usage().class_bytes, length);
    assert_eq!(unwanted_budget.usage().attribute_bytes, 20);
    assert_eq!(unwanted_budget.usage().code_bytes, 1);
    assert_eq!(unwanted_budget.usage().result_items, 2);
}
