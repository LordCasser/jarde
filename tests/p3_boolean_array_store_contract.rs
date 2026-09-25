//! A proven `[Z` plus the real `bastore` consumes the presented int's low bit.

use jarde::*;
use std::slice;

const RAW_BOOL: &[u8] = include_bytes!("fixtures/p3-boolean-array-stores/v8/RawBool.class");
const ORDER: &[u8] = include_bytes!("fixtures/p3-boolean-array-stores/v8/Order.class");
const NARROW_STORES: &[u8] =
    include_bytes!("fixtures/p3-narrow-array-stores/v8/NarrowArrayStores.class");

fn budget() -> Budget {
    task_budget(&[]).expect("bounded task defaults")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("audited class opens")
}

fn request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
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
    }
}

fn source(
    snapshot: &ArtifactSnapshot,
    class: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot, class),
            evidence,
            &mut budget(),
        )
        .expect("class source request succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen class resolves: {other:?}"),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing member {name}"))
}

fn body<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("{name} has no recovery report: {other:?}"),
    }
}

fn rewrite_once(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    assert_eq!(from.len(), to.len());
    let matches = bytes
        .windows(from.len())
        .filter(|window| *window == from)
        .count();
    assert_eq!(matches, 1, "descriptor {from:?} occurs once");
    let start = bytes
        .windows(from.len())
        .position(|window| window == from)
        .unwrap();
    bytes[start..start + to.len()].copy_from_slice(to);
}

#[test]
fn audited_bastore_is_low_bit_boolean_and_keeps_store_and_operand_origins() {
    let snapshot = open(RAW_BOOL);
    let report = source(&snapshot, "RawBool", &RecoveryEvidenceRequest::all());
    let put = body(&report, "put");
    assert_eq!(put.representation, Representation::Java, "{}", put.text);
    assert_eq!(put.quality, Quality::Structured, "{}", put.text);
    assert!(
        put.text.contains("arg0[arg1] = arg2 % 2 != 0;"),
        "{}",
        put.text
    );
    for bci in [0, 1, 2, 3] {
        assert!(
            !put.text_of_bci(bci).is_empty(),
            "missing BCI {bci}: {}",
            put.text
        );
    }
    assert!(
        put.source_map.segments().iter().any(|segment| {
            segment.text(&put.text).contains("%") && segment.origin().bcis().contains(&2)
        }),
        "low-bit expression must retain its value origin: {:?}",
        put.source_map.segments()
    );
    assert!(
        !put.source_map.direct_of_bci(3).is_empty(),
        "store BCI has a direct source span"
    );
}

#[test]
fn audited_order_keeps_each_producer_once_and_at_the_original_store() {
    let snapshot = open(ORDER);
    let report = source(&snapshot, "Order", &RecoveryEvidenceRequest::all());
    let put = body(&report, "put");
    assert_eq!(put.representation, Representation::Java, "{}", put.text);
    assert_eq!(put.quality, Quality::Structured, "{}", put.text);
    for producer in ["array(", "index(", "value("] {
        assert_eq!(put.text.matches(producer).count(), 1, "{}", put.text);
    }
    assert!(
        put.text
            .contains("array(arg0)[index()] = value(arg1) % 2 != 0;"),
        "{}",
        put.text
    );
    for bci in [0, 4, 8, 11] {
        assert!(
            !put.text_of_bci(bci).is_empty(),
            "missing BCI {bci}: {}",
            put.text
        );
    }
}

#[test]
fn presented_byte_char_and_short_values_are_admitted_only_at_proven_boolean_stores() {
    for descriptor in ["([ZIB)V", "([ZIC)V", "([ZIS)V"] {
        let mut bytes = NARROW_STORES.to_vec();
        rewrite_once(&mut bytes, b"([BII)V", descriptor.as_bytes());
        let snapshot = open(&bytes);
        let report = source(
            &snapshot,
            "NarrowArrayStores",
            &RecoveryEvidenceRequest::all(),
        );
        let method = report
            .methods
            .iter()
            .find(|method| {
                method.item.name.raw().0 == b"storeByte"
                    && method.item.descriptor.raw().0 == descriptor.as_bytes()
            })
            .unwrap_or_else(|| panic!("missing storeByte{descriptor}"));
        let recovered = match &method.outcome {
            ClassSourceOutcome::Recovered { report, .. } => report,
            other => panic!("storeByte{descriptor} must recover: {other:?}"),
        };
        assert_eq!(
            recovered.representation,
            Representation::Java,
            "{}",
            recovered.text
        );
        assert!(recovered.text.contains("% 2 != 0"), "{}", recovered.text);
        assert!(!recovered.source_map.direct_of_bci(3).is_empty());
    }
}

#[test]
fn a_boolean_provenance_value_keeps_the_existing_direct_spelling() {
    let mut bytes = NARROW_STORES.to_vec();
    rewrite_once(&mut bytes, b"([BII)V", b"([ZIZ)V");
    let snapshot = open(&bytes);
    let report = source(
        &snapshot,
        "NarrowArrayStores",
        &RecoveryEvidenceRequest::all(),
    );
    let method = report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == b"storeByte" && method.item.descriptor.raw().0 == b"([ZIZ)V"
        })
        .expect("rewritten boolean store is present");
    let recovered = match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("proven boolean value remains recoverable: {other:?}"),
    };
    assert!(
        recovered.text.contains("arg0[arg1] = arg2;"),
        "{}",
        recovered.text
    );
    assert!(
        !recovered.text.contains("% 2"),
        "boolean spelling stays direct: {}",
        recovered.text
    );
}

#[test]
fn evidence_modes_and_method_replay_keep_the_same_body_and_origins() {
    let snapshot = open(RAW_BOOL);
    let default = source(&snapshot, "RawBool", &RecoveryEvidenceRequest::essential());
    let all = source(&snapshot, "RawBool", &RecoveryEvidenceRequest::all());
    assert_eq!(default.text, all.text);
    let recovery_request = jarde::ir::MethodAnalysisRequest {
        environment: request(&snapshot, "RawBool")
            .environment
            .build(slice::from_ref(&snapshot))
            .expect("environment builds"),
        method: member(&all, "put").item.identity.clone(),
        stages: jarde::ir::AnalysisStage::ALL.to_vec(),
    };
    let recover = || {
        Engine::new()
            .recover_method_with_evidence(
                slice::from_ref(&snapshot),
                &recovery_request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("method recovery succeeds")
    };
    let first = recover();
    let replay = recover();
    assert_eq!(first.recovery().text, replay.recovery().text);
    assert_eq!(first.recovery().source_map, replay.recovery().source_map);
}

#[test]
fn output_budget_and_precancellation_publish_no_partial_low_bit_result() {
    let snapshot = open(RAW_BOOL);
    let report = source(&snapshot, "RawBool", &RecoveryEvidenceRequest::all());
    let method_request = jarde::ir::MethodAnalysisRequest {
        environment: request(&snapshot, "RawBool")
            .environment
            .build(slice::from_ref(&snapshot))
            .expect("environment builds"),
        method: member(&report, "put").item.identity.clone(),
        stages: jarde::ir::AnalysisStage::ALL.to_vec(),
    };
    let mut full_budget = budget();
    let full = Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &method_request,
            &RecoveryEvidenceRequest::all(),
            &mut full_budget,
        )
        .expect("full recovery succeeds");
    assert!(full.recovery().text.contains("% 2 != 0"));
    let emitted = u64::try_from(full.recovery().text.len()).expect("text length fits");
    let mut limited = Budget::new(Limits {
        output_bytes: full_budget.usage().output_bytes - emitted,
        ..task_limits(&[]).expect("bounded task defaults")
    });
    let stopped = Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &method_request,
            &RecoveryEvidenceRequest::all(),
            &mut limited,
        )
        .expect("budget stop is returned")
        .recovery()
        .clone();
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty());
    assert!(stopped.source_map.is_empty());
    assert!(stopped.stop().is_some());

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &method_request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::with_cancellation_token(task_limits(&[]).unwrap(), token),
        )
        .expect("cancellation is returned")
        .recovery()
        .clone();
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty());
    assert!(cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some());
}
