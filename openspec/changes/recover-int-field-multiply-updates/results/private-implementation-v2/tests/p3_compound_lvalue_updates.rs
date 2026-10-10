//! P3: proved same-block `int` field/array updates and the exact receiver-copy field `*=` shape.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::slice;

const STORE: u16 = 0;

const NESTED_MULTIPLY_ROOT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2/cases/javac23-original/classes/em23/InputFieldIncrement2.class"
);
const NESTED_MULTIPLY_CHILD: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2/cases/javac23-original/classes/em23/InputFieldIncrement2$A.class"
);
const BOUNDARY_PROBE: &[u8] = include_bytes!(
    "fixtures/p3-compound-lvalue-updates/boundaries/v8/CompoundBoundaryProbe.class"
);
const BOUNDARY_BOX: &[u8] = include_bytes!(
    "fixtures/p3-compound-lvalue-updates/boundaries/v8/BoundaryBox.class"
);
const FIELD_DIFFERENT_MEMBER: &[u8] = include_bytes!(
    "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-different-member/CompoundBoundaryProbe.class"
);
const FIELD_MULTI_CONSUMER: &[u8] = include_bytes!(
    "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-multi-consumer/CompoundBoundaryProbe.class"
);

const PROBE: &[u8] = include_bytes!("fixtures/p3-compound-lvalue-updates/v8/CompoundProbe.class");
const BOUNDARY_GAPS: [(&str, &[u8], &[u32]); 6] = [
    (
        "field-gap-before-dup",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-gap-before-dup/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 4, 8, 9, 12, 16],
    ),
    (
        "field-gap-after-dup",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-gap-after-dup/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 4, 5, 9, 12, 16],
    ),
    (
        "field-gap-before-store",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-gap-before-store/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 4, 7, 11, 12, 16],
    ),
    (
        "array-gap-before-index",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-gap-before-index/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 4, 8, 11, 12, 13, 17],
    ),
    (
        "array-gap-after-dup",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-gap-after-dup/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 6, 7, 8, 12, 13, 17],
    ),
    (
        "array-gap-before-store",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-gap-before-store/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 6, 7, 8, 12, 13, 17],
    ),
];
const IDENTITY_BOUNDARIES: [(&str, &[u8], &str, &[u32]); 5] = [
    (
        "field-different-member",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-different-member/CompoundBoundaryProbe.class"
        ),
        "fieldDifferentMember",
        &[0, 3, 4, 8, 12],
    ),
    (
        "array-different-index-copy",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-different-index-copy/CompoundBoundaryProbe.class"
        ),
        "arrayDifferentIndex",
        &[0, 3, 6, 9, 11, 15],
    ),
    (
        "array-different-array-copy",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-different-array-copy/CompoundBoundaryProbe.class"
        ),
        "arrayDifferentArray",
        &[0, 3, 6, 8, 12, 14, 18],
    ),
    (
        "field-multi-consumer",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-multi-consumer/CompoundBoundaryProbe.class"
        ),
        "fieldMultiConsumer",
        &[0, 3, 4, 5, 8, 12, 16],
    ),
    (
        "array-multi-consumer",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-multi-consumer/CompoundBoundaryProbe.class"
        ),
        "arrayMultiConsumer",
        &[0, 3, 6, 7, 8, 11, 13, 17],
    ),
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen class opens")
}

fn request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    request_with_policy(snapshot, class, EnvironmentPolicy::SingleClass)
}

fn request_with_policy(
    snapshot: &ArtifactSnapshot,
    class: &str,
    policy: EnvironmentPolicy,
) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_string()),
        },
    }
}

fn class_source(
    snapshot: &ArtifactSnapshot,
    class: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    class_source_with_policy(snapshot, class, evidence, EnvironmentPolicy::SingleClass)
}

fn class_source_with_policy(
    snapshot: &ArtifactSnapshot,
    class: &str,
    evidence: &RecoveryEvidenceRequest,
    policy: EnvironmentPolicy,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request_with_policy(snapshot, class, policy),
            evidence,
            &mut budget(),
        )
        .expect("the fixture class-source request is valid")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen class resolves uniquely: {other:?}"),
    }
}

fn jar_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
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
            writer.write_all(data).expect("the fixture entry writes");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry.finish(descriptor).expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

/// Change one unique arithmetic opcode in a committed class file. Code lengths, exception tables,
/// stack maps and every other byte remain untouched; the selected opcodes share the same operand
/// stack shape, so this creates proof-boundary variants without a JDK test-time dependency.
fn replace_unique_opcode(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    old_opcode: u8,
    new_opcode: u8,
) -> Vec<u8> {
    let mut budget = budget();
    let header = jarde_reader::classfile::class_facts(class, &mut budget)
        .expect("the committed boundary class parses");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == name && member.descriptor.raw().0 == descriptor)
        .expect("the selected method is physically declared");
    let code = jarde_reader::classfile::method_code_facts(class, member, &mut budget)
        .expect("the selected method's instructions decode");
    let instructions = code
        .instructions
        .iter()
        .filter(|instruction| instruction.opcode == old_opcode)
        .collect::<Vec<_>>();
    assert_eq!(instructions.len(), 1, "the target opcode is unique in this method");
    let instruction = instructions[0];
    let class_offset = usize::try_from(code.code_span.start)
        .expect("class code offset fits usize")
        .checked_add(usize::try_from(instruction.bci).expect("BCI fits usize"))
        .expect("instruction byte offset does not overflow");
    let mut patched = class.to_vec();
    assert_eq!(patched[class_offset], old_opcode, "the decoded opcode is in place");
    patched[class_offset] = new_opcode;
    patched
}

fn method_bcis(class: &[u8], name: &[u8], descriptor: &[u8]) -> Vec<u32> {
    let mut budget = budget();
    let header = jarde_reader::classfile::class_facts(class, &mut budget)
        .expect("the committed class parses");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == name && member.descriptor.raw().0 == descriptor)
        .expect("the selected method is physically declared");
    jarde_reader::classfile::method_code_facts(class, member, &mut budget)
        .expect("the selected method's instructions decode")
        .instructions
        .into_iter()
        .map(|instruction| instruction.bci)
        .collect()
}

fn recovery_request(
    snapshot: &ArtifactSnapshot,
    member: &ClassSourceMethod,
) -> jarde::ir::MethodAnalysisRequest {
    jarde::ir::MethodAnalysisRequest {
        environment: request(snapshot, "CompoundProbe")
            .environment
            .build(slice::from_ref(snapshot))
            .expect("the fixture's single-class environment is valid"),
        method: member.item.identity.clone(),
        stages: jarde::ir::AnalysisStage::ALL.to_vec(),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no method `{name}`"))
}

fn recovered(method: &ClassSourceMethod) -> &RecoveryReport {
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the fixture method was not recovered: {other:?}"),
    }
}

#[test]
fn same_block_field_and_int_array_updates_are_single_evaluation_source_mapped_statements() {
    let snapshot = open(PROBE);
    let essential = class_source(
        &snapshot,
        "CompoundProbe",
        &RecoveryEvidenceRequest::essential(),
    );
    let complete = class_source(&snapshot, "CompoundProbe", &RecoveryEvidenceRequest::all());
    assert_eq!(
        essential.text, complete.text,
        "evidence selection changes no body"
    );

    for (name, statement, once) in [
        ("field", "receiver().value += rhs(2);", "receiver()"),
        ("array", "data[index()] += rhs(4);", "index()"),
        (
            "fieldSnapshot",
            "receiver().value += rhsFieldMutation();",
            "receiver()",
        ),
        (
            "arraySnapshot",
            "data[index()] += rhsArrayMutation();",
            "index()",
        ),
    ] {
        let body = &method(&essential, name).text;
        assert!(body.contains(statement), "`{name}` omitted update:\n{body}");
        assert_eq!(
            body.matches(once).count(),
            1,
            "`{name}` evaluates lhs twice:\n{body}"
        );
        let mapped = recovered(method(&complete, name));
        assert_eq!(mapped.text, recovered(method(&essential, name)).text);
        if name.starts_with("field") {
            let accesses: Vec<_> = mapped
                .fields
                .iter()
                .filter(|field| field.name == "value")
                .map(|field| (field.access, field.presented))
                .collect();
            assert!(
                accesses.contains(&("read", true))
                    && accesses.contains(&("write", true))
                    && accesses.iter().all(|(_, presented)| *presented),
                "{name}: {accesses:?}\n{}",
                mapped.text
            );
        }
        assert!(
            !mapped.source_map.is_empty(),
            "`{name}` has no selected source map"
        );
    }

    for (name, bcis) in [
        ("field", &[0, 3, 4, 7, 8, 11, 12][..]),
        ("fieldSnapshot", &[0, 3, 4, 7, 10, 11][..]),
        ("array", &[0, 3, 6, 7, 8, 9, 12, 13][..]),
        ("arraySnapshot", &[0, 3, 6, 7, 8, 11, 12][..]),
    ] {
        let report = recovered(method(&complete, name));
        for bci in bcis {
            assert!(
                !report.source_map.text_of_bci(&report.text, *bci).is_empty(),
                "`{name}` update lost source BCI {bci}: {:?}",
                report.source_map.segments()
            );
        }
    }
}

#[test]
fn lvalue_prefix_gaps_refuse_compound_claims_and_keep_the_complete_chain() {
    for (name, bytes, bcis) in BOUNDARY_GAPS {
        let snapshot = open(bytes);
        let report = class_source(
            &snapshot,
            "CompoundBoundaryProbe",
            &RecoveryEvidenceRequest::all(),
        );
        let is_field = name.starts_with("field-");
        let method_name = if is_field {
            "fieldSnapshot"
        } else {
            "arraySnapshot"
        };
        let method = method(&report, method_name);
        let recovery = recovered(method);
        assert!(
            !method.text.contains("+= "),
            "uninterrupted update proof crossed {name}:\n{}",
            method.text
        );
        for bci in bcis {
            assert!(
                !recovery
                    .source_map
                    .text_of_bci(&recovery.text, *bci)
                    .is_empty(),
                "{name} lost original BCI {bci}: {:?}\n{}",
                recovery.source_map.segments(),
                method.text
            );
        }
    }
}

#[test]
fn mismatched_and_shared_lvalue_copies_are_refused_with_their_source_anchors() {
    for (name, bytes, method_name, bcis) in IDENTITY_BOUNDARIES {
        let snapshot = open(bytes);
        let report = class_source(
            &snapshot,
            "CompoundBoundaryProbe",
            &RecoveryEvidenceRequest::all(),
        );
        let method = method(&report, method_name);
        let recovery = recovered(method);
        assert!(
            !method.text.contains("+= "),
            "identity/consumer boundary {name} was presented as compound update:\n{}",
            method.text
        );
        for bci in bcis {
            assert!(
                !recovery
                    .source_map
                    .text_of_bci(&recovery.text, *bci)
                    .is_empty(),
                "{name} lost original BCI {bci}: {:?}\n{}",
                recovery.source_map.segments(),
                method.text
            );
        }
    }
}

#[test]
fn nested_int_field_multiply_keeps_every_instruction_and_field_read_presented() {
    let snapshot = open(&jar_of(&[
        (b"em23/InputFieldIncrement2.class", NESTED_MULTIPLY_ROOT),
        (b"em23/InputFieldIncrement2$A.class", NESTED_MULTIPLY_CHILD),
    ]));
    let essential = class_source_with_policy(
        &snapshot,
        "em23/InputFieldIncrement2",
        &RecoveryEvidenceRequest::essential(),
        EnvironmentPolicy::PlainJar,
    );
    let complete = class_source_with_policy(
        &snapshot,
        "em23/InputFieldIncrement2",
        &RecoveryEvidenceRequest::all(),
        EnvironmentPolicy::PlainJar,
    );
    assert_eq!(essential.text, complete.text, "evidence selection changes no text");

    let method = method(&complete, "test2");
    let report = recovered(method);
    assert!(method.text.contains("this.a.f *= "), "{}", method.text);
    assert!(!method.text.contains("jarde_refused_body"), "{}", method.text);
    for bci in [0, 1, 4, 5, 8, 9, 10, 13] {
        assert!(
            !report.source_map.text_of_bci(&report.text, bci).is_empty(),
            "multiply update lost BCI {bci}: {:?}\n{}",
            report.source_map.segments(),
            method.text
        );
    }
    for (bci, access, name) in [
        (1, "read", "a"),
        (5, "read", "f"),
        (10, "write", "f"),
    ] {
        assert!(
            report.fields.iter().any(|field| {
                field.bci == bci && field.access == access && field.name == name && field.presented
            }),
            "field access {access} {name}@{bci} was not presented: {:?}\n{}",
            report.fields,
            method.text
        );
    }
    assert!(!essential.text.is_empty());
}

#[test]
fn multiply_does_not_merge_two_reads_or_cross_field_width_or_consumer_boundaries() {
    // Keep the actual explicit two-read shape, but change only iadd to imul. The source reads
    // `this.a` independently for the store and the old value, so it must remain ordinary `=`.
    let explicit_two_reads = replace_unique_opcode(
        NESTED_MULTIPLY_ROOT,
        b"test1",
        b"(I)V",
        0x60,
        0x68,
    );
    let snapshot = open(&jar_of(&[
        (b"em23/InputFieldIncrement2.class", &explicit_two_reads),
        (b"em23/InputFieldIncrement2$A.class", NESTED_MULTIPLY_CHILD),
    ]));
    let report = class_source_with_policy(
        &snapshot,
        "em23/InputFieldIncrement2",
        &RecoveryEvidenceRequest::all(),
        EnvironmentPolicy::PlainJar,
    );
    let member = method(&report, "test1");
    let recovery = recovered(member);
    assert!(member.text.contains("this.a.f = this.a.f * "), "{}", member.text);
    assert!(!member.text.contains("*="), "{}", member.text);
    for bci in method_bcis(&explicit_two_reads, b"test1", b"(I)V") {
        assert!(
            !recovery.source_map.text_of_bci(&recovery.text, bci).is_empty(),
            "two-read control lost BCI {bci}: {:?}\n{}",
            recovery.source_map.segments(),
            member.text
        );
    }
    for bci in [1, 5] {
        assert!(
            recovery.fields.iter().any(|field| {
                field.bci == bci && field.access == "read" && field.name == "a" && field.presented
            }),
            "independent receiver read a@{bci} was lost: {:?}",
            recovery.fields
        );
    }

    for (label, class, method_name, descriptor, arithmetic, replacement) in [
        (
            "field-member-mismatch",
            FIELD_DIFFERENT_MEMBER,
            b"fieldDifferentMember".as_slice(),
            b"()V".as_slice(),
            0x60,
            0x68,
        ),
        (
            "field-extra-consumer",
            FIELD_MULTI_CONSUMER,
            b"fieldMultiConsumer".as_slice(),
            b"()V".as_slice(),
            0x60,
            0x68,
        ),
        (
            "wide-field",
            BOUNDARY_PROBE,
            b"wideField".as_slice(),
            b"()V".as_slice(),
            0x61,
            0x69,
        ),
    ] {
        let patched = replace_unique_opcode(class, method_name, descriptor, arithmetic, replacement);
        let archive = jar_of(&[
            (b"CompoundBoundaryProbe.class", &patched),
            (b"BoundaryBox.class", BOUNDARY_BOX),
        ]);
        let snapshot = open(&archive);
        let report = class_source_with_policy(
            &snapshot,
            "CompoundBoundaryProbe",
            &RecoveryEvidenceRequest::all(),
            EnvironmentPolicy::PlainJar,
        );
        let method = method(&report, std::str::from_utf8(method_name).unwrap());
        let recovery = recovered(method);
        assert!(
            !method.text.contains("*="),
            "{label} was presented as multiplication assignment:\n{}",
            method.text
        );
        for bci in method_bcis(&patched, method_name, descriptor) {
            assert!(
                !recovery.source_map.text_of_bci(&recovery.text, bci).is_empty(),
                "{label} lost BCI {bci}: {:?}\n{}",
                recovery.source_map.segments(),
                method.text
            );
        }
    }
}

#[test]
fn multiply_public_recovery_keeps_output_budget_and_cancellation_atomic() {
    let snapshot = open(&jar_of(&[
        (b"em23/InputFieldIncrement2.class", NESTED_MULTIPLY_ROOT),
        (b"em23/InputFieldIncrement2$A.class", NESTED_MULTIPLY_CHILD),
    ]));
    let complete = class_source_with_policy(
        &snapshot,
        "em23/InputFieldIncrement2",
        &RecoveryEvidenceRequest::all(),
        EnvironmentPolicy::PlainJar,
    );
    let member = method(&complete, "test2");
    let environment = request_with_policy(
        &snapshot,
        "em23/InputFieldIncrement2",
        EnvironmentPolicy::PlainJar,
    )
    .environment
    .build(slice::from_ref(&snapshot))
    .expect("the same two-class family builds its request environment");
    let request = jarde::ir::MethodAnalysisRequest {
        environment,
        method: member.item.identity.clone(),
        stages: jarde::ir::AnalysisStage::ALL.to_vec(),
    };
    let engine = Engine::new();
    let evidence = RecoveryEvidenceRequest::all();
    let mut full_budget = budget();
    let full = engine
        .recover_method_with_evidence(slice::from_ref(&snapshot), &request, &evidence, &mut full_budget)
        .expect("the proved multiply body recovers");
    let full_report = full.recovery();
    assert!(full_report.produced());
    assert!(full_report.text.contains("this.a.f *= "));

    let emitted = u64::try_from(full_report.text.len()).expect("the body length fits u64");
    let complete_output = full_budget.usage().output_bytes;
    assert!(complete_output > emitted);
    let mut output_budget = Budget::new(Limits {
        output_bytes: complete_output - emitted,
        ..task_limits(&[]).expect("the task limits are bounded")
    });
    let stopped = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut output_budget,
        )
        .expect("output exhaustion is reported");
    let stopped_report = stopped.recovery();
    assert!(!stopped_report.produced());
    assert!(stopped_report.text.is_empty());
    assert!(stopped_report.source_map.is_empty());
    assert!(matches!(
        stopped_report.stop(),
        Some(StopReason::Budget {
            dimension: jarde::budget::CountedBudgetDimension::OutputBytes,
            ..
        })
    ));

    let token = jarde::budget::CancellationToken::new();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(
        task_limits(&[]).expect("the task limits are bounded"),
        token,
    );
    let cancelled = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut cancelled_budget,
        )
        .expect("cancellation is reported");
    assert!(matches!(
        cancelled.analysis().execution,
        ExecutionReport::Cancelled { .. }
    ));
    let cancelled_report = cancelled.recovery();
    assert!(!cancelled_report.produced());
    assert!(cancelled_report.text.is_empty());
    assert!(cancelled_report.source_map.is_empty());
}

#[test]
fn compound_recovery_keeps_output_evidence_and_cancellation_stops() {
    let snapshot = open(PROBE);
    let complete = class_source(&snapshot, "CompoundProbe", &RecoveryEvidenceRequest::all());
    let method = method(&complete, "array");
    let request = recovery_request(&snapshot, method);
    let engine = Engine::new();
    let evidence = RecoveryEvidenceRequest::all();
    let mut full_budget = task_budget(&[]).expect("the task defaults are bounded");
    let full = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut full_budget,
        )
        .expect("the compound method is recovered");
    let full_report = full.recovery();
    assert!(full_report.produced());
    assert!(full_report.text.contains("data[index()] += rhs(4);"));
    assert!(matches!(
        full_report.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    ));
    let used_items = full_budget
        .usage()
        .counted_usage(jarde::budget::CountedBudgetDimension::IrItems);
    assert!(used_items > 1, "the proved chain charges bounded work");

    let emitted = u64::try_from(full_report.text.len()).expect("the artifact length fits u64");
    let complete_output = full_budget.usage().output_bytes;
    assert!(
        complete_output > emitted,
        "analysis output precedes the artifact"
    );
    let mut output_budget = Budget::new(Limits {
        output_bytes: complete_output - emitted,
        ..task_limits(&[]).expect("the task defaults are bounded")
    });
    let stopped_output = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut output_budget,
        )
        .expect("an output stop is reported");
    let stopped_report = stopped_output.recovery();
    assert!(!stopped_report.produced());
    assert!(stopped_report.text.is_empty());
    assert!(stopped_report.source_map.is_empty());
    assert!(
        matches!(
            stopped_report.stop(),
            Some(StopReason::Budget {
                dimension: jarde::budget::CountedBudgetDimension::OutputBytes,
                ..
            })
        ),
        "the body stop keeps its budget dimension: {:?}",
        stopped_report.stop()
    );

    let mut map_budget = Budget::new(Limits {
        ir_items: used_items - 1,
        ..task_limits(&[]).expect("the task defaults are bounded")
    });
    let partial_map = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut map_budget,
        )
        .expect("an evidence stop is reported");
    let partial_report = partial_map.recovery();
    assert!(partial_report.produced(), "text survives an evidence stop");
    assert_eq!(partial_report.text, full_report.text);
    assert!(partial_report.source_map.len() < full_report.source_map.len());
    assert!(matches!(
        partial_report
            .evidence
            .state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Partial { .. }
    ));

    let token = jarde::budget::CancellationToken::new();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(
        task_limits(&[]).expect("the task defaults are bounded"),
        token,
    );
    let cancelled = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut cancelled_budget,
        )
        .expect("cancellation is reported");
    assert!(matches!(
        cancelled.analysis().execution,
        ExecutionReport::Cancelled { .. }
    ));
    let cancelled_report = cancelled.recovery();
    assert!(!cancelled_report.produced());
    assert!(cancelled_report.text.is_empty());
    assert!(cancelled_report.source_map.is_empty());
}
