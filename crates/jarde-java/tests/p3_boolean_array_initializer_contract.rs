//! Contract for Java 8's verifier-valid `bastore` boolean-array initializer shape.
//!
//! The frozen patched class is the independent audit artifact. Its stores are at BCI 6, 10, 14,
//! 18 and 22; BCI 23 is only the consumer. The test drives the bytes through the public analysis
//! and recovery surfaces so store provenance cannot be supplied by the test itself.

use jarde_java::{MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest, recover};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Representation};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::classfile::class_facts;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const BOOL_INIT: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-boolean-array-initializers/v8/BoolInit.class");
const EFFECTFUL_BOOL_INIT: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-boolean-array-initializers/v8/BoolInitEffectful.class"
);
const PATCHED_CODE: &[u8] = &[
    0x08, 0xbc, 0x04, 0x59, 0x03, 0x03, 0x54, 0x59, 0x04, 0x04, 0x54, 0x59, 0x05, 0x05, 0x54, 0x59,
    0x06, 0x06, 0x54, 0x59, 0x07, 0x02, 0x54, 0xb0,
];

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

struct Payload {
    analysis: jarde_jvm::method_ir::MethodIrAnalysis,
}

fn analyze(class: &[u8], name: &[u8], descriptor: &[u8]) -> Payload {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("frozen class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("class length fits"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let environment = ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: domain.clone(),
        },
        domains: vec![domain],
        providers: Vec::new(),
    };
    let request = MethodAnalysisRequest {
        environment,
        method,
        stages: AnalysisStage::ALL.to_vec(),
    };
    Payload {
        analysis: analyze_method_ir(&[snapshot], &request, &mut budget)
            .expect("analysis of the frozen method runs"),
    }
}

fn facts(class: &[u8], name: &[u8]) -> RecoveryFacts {
    let mut budget = Budget::new(limits());
    let header = class_facts(class, &mut budget).expect("class facts");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == name)
        .expect("method exists");
    RecoveryFacts::new(MethodFacts::new(
        String::from_utf8_lossy(name),
        String::from_utf8_lossy(&member.descriptor.raw().0),
        u16::from(member.descriptor.raw().0.contains(&b'I')),
    ))
}

#[test]
fn proven_boolean_initializer_uses_each_real_bastore_as_low_bit_origin() {
    let payload = analyze(BOOL_INIT, b"values", b"()[Z");
    let facts = facts(BOOL_INIT, b"values");
    let mut budget = Budget::new(limits());
    let report = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    );

    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(
        report.representation,
        Representation::Java,
        "{}",
        report.text
    );
    for expected in ["false", "true", "2 % 2 != 0", "3 % 2 != 0", "-1 % 2 != 0"] {
        assert!(
            report.text.contains(expected),
            "missing {expected:?}: {}",
            report.text
        );
    }
    assert!(
        !report.text.contains("new boolean[]{0, 1, 2, 3, -1}"),
        "{}",
        report.text
    );

    // Every rewritten integer's own source segment carries its producer and its paired store.
    // The overall consumer is BCI 23 and must not be substituted for any store coordinate.
    for (snippet, store_bci) in [("2 % 2 != 0", 14), ("3 % 2 != 0", 18), ("-1 % 2 != 0", 22)] {
        let matching: Vec<_> = report
            .source_map
            .segments()
            .iter()
            .filter(|segment| segment.text(&report.text) == snippet)
            .collect();
        assert_eq!(matching.len(), 1, "source for {snippet:?}: {matching:?}");
        let bcis: Vec<_> = matching[0].origin().bcis().into_iter().collect();
        assert!(bcis.contains(&store_bci), "{snippet}: {bcis:?}");
        assert!(
            !bcis.contains(&23),
            "consumer leaked into {snippet}: {bcis:?}"
        );
    }
    for store_bci in [6, 10, 14, 18, 22] {
        assert!(
            report
                .source_map
                .of_bci(store_bci)
                .iter()
                .any(|segment| { segment.text(&report.text).contains("new boolean[]") }),
            "store BCI {store_bci} is retained on the initializer"
        );
    }
    assert!(
        report
            .source_map
            .of_bci(23)
            .iter()
            .any(|segment| { segment.text(&report.text).contains("return new boolean[]") })
    );
}

#[test]
fn effectful_boolean_elements_stay_ordered_once_and_keep_their_three_store_origins() {
    let payload = analyze(EFFECTFUL_BOOL_INIT, b"values", b"(I)[Z");
    let facts = facts(EFFECTFUL_BOOL_INIT, b"values");
    let mut budget = Budget::new(limits());
    let report = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    );
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert!(
        report.text.contains("return new boolean[]{"),
        "{}",
        report.text
    );

    let snippets: Vec<_> = report
        .source_map
        .segments()
        .iter()
        .filter(|segment| segment.text(&report.text).contains("% 2 != 0"))
        .collect();
    let relevant: Vec<_> = snippets
        .into_iter()
        .filter(|segment| {
            let text = segment.text(&report.text);
            text.starts_with("element(") && text.ends_with("% 2 != 0")
        })
        .collect();
    assert_eq!(relevant.len(), 3, "{}", report.text);
    let values = [
        ("element(arg0, 0)", 10),
        ("element(arg0, 1)", 18),
        ("element(arg0, 2)", 26),
    ];
    let mut previous_start = 0;
    for (producer, store_bci) in values {
        let snippet = relevant
            .iter()
            .find(|segment| segment.text(&report.text).contains(producer))
            .unwrap_or_else(|| panic!("missing producer {producer}: {}", report.text));
        let text = snippet.text(&report.text);
        assert_eq!(text.matches(producer).count(), 1, "{text}");
        assert!(
            snippet.start() >= previous_start,
            "producer order changed: {text}"
        );
        previous_start = snippet.start();
        assert!(
            snippet.origin().bcis().contains(&store_bci),
            "{text}: {:?}",
            snippet.origin()
        );
        assert!(
            !snippet.origin().bcis().contains(&27),
            "consumer leaked: {text}"
        );
    }
}

#[test]
fn a_store_with_the_wrong_opcode_does_not_enter_the_boolean_initializer() {
    let mut malformed = BOOL_INIT.to_vec();
    let code_start = malformed
        .windows(PATCHED_CODE.len())
        .position(|window| window == PATCHED_CODE)
        .expect("the frozen method's Code bytes occur once");
    // Preserve instruction widths and stack shape while changing element 2's real store from
    // `bastore` to `iastore`. The initializer proof must reject the entire candidate.
    malformed[code_start + 14] = 0x4f;

    let payload = analyze(&malformed, b"values", b"()[Z");
    let facts = facts(&malformed, b"values");
    let mut budget = Budget::new(limits());
    let report = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        !report.text.contains("new boolean[]{"),
        "a mismatched store was projected as an initializer:\n{}",
        report.text
    );
}

#[test]
fn essential_and_all_evidence_select_the_same_initializer_body() {
    let payload = analyze(BOOL_INIT, b"values", b"()[Z");
    let facts = facts(BOOL_INIT, b"values");
    let mut essential_budget = Budget::new(limits());
    let essential = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8),
        &mut essential_budget,
    );
    let mut all_budget = Budget::new(limits());
    let all = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut all_budget,
    );
    let mut replay_budget = Budget::new(limits());
    let replay = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut replay_budget,
    );
    assert!(essential.produced() && all.produced());
    assert_eq!(essential.text, all.text);
    assert_eq!(all.text, replay.text);
    assert_eq!(all.source_map, replay.source_map);
}

#[test]
fn budget_and_cancellation_do_not_publish_a_partial_initializer() {
    let payload = analyze(BOOL_INIT, b"values", b"()[Z");
    let facts = facts(BOOL_INIT, b"values");

    let mut tiny_limits = limits();
    tiny_limits.output_bytes = 8;
    let mut tiny_budget = Budget::new(tiny_limits);
    let stopped = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8),
        &mut tiny_budget,
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty());
    assert!(stopped.source_map.is_empty());

    let token = CancellationToken::default();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(limits(), token);
    let cancelled = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8),
        &mut cancelled_budget,
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty());
    assert!(cancelled.source_map.is_empty());
}
