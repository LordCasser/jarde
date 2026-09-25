use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryFacts,
    RecoveryRequest,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::class_facts;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::LoaderId;
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const BASE: &[u8] = include_bytes!(
    "../../../tests/fixtures/proved-java-structure/anonymous-capture/AnonymousCaptureCases.class"
);
const OUTER: &[u8] = include_bytes!(
    "../../../tests/fixtures/proved-java-structure/anonymous-capture/AnonymousCaptureCases$Outer.class"
);
const DOUBLE_SITE: &[u8] = include_bytes!(
    "../../../tests/fixtures/proved-java-structure/anonymous-double-site/AnonymousDoubleSite.class"
);
const CONCAT: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-concat-conversion/v8/ConcatConversion.class");

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

struct MethodFixture {
    analysis: jarde_jvm::method_ir::MethodIrAnalysis,
    facts: RecoveryFacts,
}

fn method_fixture(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    parameter_slots: u16,
) -> MethodFixture {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("frozen class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("fixture size fits"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
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
    let analysis = analyze_method_ir(
        &[snapshot],
        &MethodAnalysisRequest {
            environment,
            method,
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("frozen method analyzes");
    let header = class_facts(class, &mut budget).expect("frozen class facts parse");
    let member = header
        .methods
        .iter()
        .find(|method| method.name.raw().0 == name && method.descriptor.raw().0 == descriptor)
        .expect("method exists in frozen class");
    let declaring_name = String::from_utf8_lossy(&header.this_class.raw().0).into_owned();
    let facts = RecoveryFacts::new(
        MethodFacts::new(
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor),
            parameter_slots,
        )
        .with_access_flags(member.access_flags)
        .with_declaring_class(DeclaringClass::new(declaring_name, header.access_flags)),
    );
    MethodFixture { analysis, facts }
}

fn recover(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    slots: u16,
    evidence: RecoveryEvidenceRequest,
) -> jarde_java::report::ClassSourceRecovery {
    let fixture = method_fixture(class, name, descriptor, slots);
    let request = RecoveryRequest::new(
        fixture.analysis.ir(),
        &fixture.facts,
        jarde_java::pass::JAVA_8,
    )
    .with_evidence(evidence);
    jarde_java::report::recover_for_class_source(&request, &mut Budget::new(limits()), false, false)
}

fn allocation_for(
    class: &[u8],
    method: &[u8],
    descriptor: &[u8],
    slots: u16,
    target: &str,
) -> jarde_java::report::AnonymousAllocationCandidate {
    let result = recover(
        class,
        method,
        descriptor,
        slots,
        RecoveryEvidenceRequest::essential(),
    );
    assert!(result.report.produced(), "{:?}", result.report.outcome);
    let scan = result
        .anonymous_allocations
        .expect("physical allocation scan");
    assert!(
        scan.complete,
        "all raw new opcodes must be represented: {scan:?}"
    );
    let found: Vec<_> = scan
        .allocations
        .into_iter()
        .filter(|allocation| allocation.class == target)
        .collect();
    assert_eq!(found.len(), 1, "{target}");
    found.into_iter().next().expect("one target allocation")
}

#[test]
fn same_run_new_sidecar_preserves_order_and_ignores_rule_details_selection() {
    let base = allocation_for(
        BASE,
        b"baseArgumentAndCapture",
        b"()LAnonymousCaptureCases$Renderer;",
        0,
        "AnonymousCaptureCases$1",
    );
    assert!(base.verified);
    assert_eq!(base.head_bci, 4);
    assert_eq!(base.constructor_bci, Some(12));
    assert_eq!(base.argument_bcis, [8, 11], "choose() then captured local");
    assert_eq!(base.member.name.0, b"baseArgumentAndCapture");

    let outer = allocation_for(
        OUTER,
        b"captureOther",
        b"(LAnonymousCaptureCases$Outer;)LAnonymousCaptureCases$Renderer;",
        1,
        "AnonymousCaptureCases$Outer$1",
    );
    assert!(outer.verified);
    assert_eq!(outer.head_bci, 0);
    assert_eq!(outer.constructor_bci, Some(6));
    assert_eq!(outer.argument_bcis, [4, 5], "this then other");
    assert_eq!(outer.member.name.0, b"captureOther");

    for (class, method, descriptor, slots, target, expected_arguments) in [
        (
            BASE,
            b"baseArgumentAndCapture".as_slice(),
            b"()LAnonymousCaptureCases$Renderer;".as_slice(),
            0,
            "AnonymousCaptureCases$1",
            2,
        ),
        (
            OUTER,
            b"captureOther".as_slice(),
            b"(LAnonymousCaptureCases$Outer;)LAnonymousCaptureCases$Renderer;".as_slice(),
            1,
            "AnonymousCaptureCases$Outer$1",
            2,
        ),
    ] {
        let essential = recover(
            class,
            method,
            descriptor,
            slots,
            RecoveryEvidenceRequest::essential(),
        );
        let detailed = recover(
            class,
            method,
            descriptor,
            slots,
            RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::RuleDetails),
        );
        let essential_scan = essential.anonymous_allocations.expect("essential scan");
        let detailed_scan = detailed.anonymous_allocations.expect("detailed scan");
        assert_eq!(essential_scan, detailed_scan);
        assert!(essential.report.news.is_empty());
        assert!(
            detailed
                .report
                .news
                .iter()
                .any(|news| news.class == target && news.presented)
        );
        let candidate = essential_scan
            .allocations
            .iter()
            .find(|entry| entry.class == target)
            .expect("candidate");
        assert!(candidate.verified);
        assert_eq!(candidate.argument_bcis.len(), expected_arguments);
    }
}

#[test]
fn same_method_double_site_scan_preserves_both_bcis_without_rule_details() {
    let essential = recover(
        DOUBLE_SITE,
        b"create",
        b"(Z)LDoubleBase;",
        1,
        RecoveryEvidenceRequest::essential(),
    );
    let detailed = recover(
        DOUBLE_SITE,
        b"create",
        b"(Z)LDoubleBase;",
        1,
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::RuleDetails),
    );
    let scan = essential
        .anonymous_allocations
        .as_ref()
        .expect("same-run allocation scan");
    assert!(scan.complete, "{scan:?}");
    let sites: Vec<_> = scan
        .allocations
        .iter()
        .filter(|candidate| candidate.class == "AnonymousDoubleSite$1")
        .collect();
    assert_eq!(
        sites.len(),
        2,
        "both source sites reuse the one binary class"
    );
    assert_eq!(
        sites
            .iter()
            .map(|candidate| candidate.head_bci)
            .collect::<Vec<_>>(),
        [4, 12],
        "the physical bytecode has two distinct allocation instructions",
    );
    assert!(sites.iter().all(|candidate| candidate.verified));
    assert_eq!(
        sites[0].member, sites[1].member,
        "both sites keep the same physical method identity",
    );
    assert_eq!(
        essential.anonymous_allocations, detailed.anonymous_allocations,
        "RuleDetails selection cannot change the sidecar",
    );
}

#[test]
fn allocation_candidates_include_new_expressions_reserved_by_concat() {
    let result = recover(
        CONCAT,
        b"twoIntsThenString",
        b"(II)Ljava/lang/String;",
        2,
        RecoveryEvidenceRequest::essential(),
    );
    assert!(result.report.produced(), "{:?}", result.report.outcome);
    let scan = result
        .anonymous_allocations
        .expect("physical allocation scan");
    assert!(scan.complete, "{scan:?}");
    let string_builders: Vec<_> = scan
        .allocations
        .iter()
        .filter(|allocation| allocation.class == "java/lang/StringBuilder")
        .collect();
    assert_eq!(
        string_builders.len(),
        1,
        "the method contains one concat chain"
    );
    assert!(!string_builders[0].verified, "concat owns this allocation");
    assert_eq!(string_builders[0].head_bci, 0);
}

#[test]
fn unresolved_new_constant_pool_entry_cannot_yield_a_complete_scan() {
    let mut malformed = BASE.to_vec();
    // In the frozen baseArgumentAndCapture Code, this unique instruction sequence is
    // `new #38; dup; invokestatic #40; aload_0; invokespecial #43`. Pointing the new operand at
    // existing CONSTANT_Fieldref #1 leaves opcode 0xbb intact but gives it no decodable class.
    let sequence = [
        0xbb, 0x00, 0x26, 0x59, 0xb8, 0x00, 0x28, 0x2a, 0xb7, 0x00, 0x2b,
    ];
    let starts: Vec<_> = malformed
        .windows(sequence.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == sequence).then_some(offset))
        .collect();
    assert_eq!(starts.len(), 1, "the target allocation bytes are unique");
    let operand = starts[0] + 1;
    malformed[operand] = 0;
    malformed[operand + 1] = 1;

    let fixture = method_fixture(
        &malformed,
        b"baseArgumentAndCapture",
        b"()LAnonymousCaptureCases$Renderer;",
        0,
    );
    let request = RecoveryRequest::new(
        fixture.analysis.ir(),
        &fixture.facts,
        jarde_java::pass::JAVA_8,
    )
    .with_evidence(RecoveryEvidenceRequest::essential());
    let result = jarde_java::report::recover_for_class_source(
        &request,
        &mut Budget::new(limits()),
        false,
        false,
    );
    assert!(
        !result.report.produced(),
        "invalid new CP type stops frame analysis"
    );
    assert!(
        result.anonymous_allocations.is_none(),
        "a stopped recovery has no scan; callers must treat absent scan as incomplete"
    );
}
