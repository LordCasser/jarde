//! Public API regression for loops whose headers contain ordered effects.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryFacts,
    RecoveryReport, RecoveryRequest, StopReason, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Quality};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension, Limits};
use jarde_reader::classfile::class_facts;
use jarde_reader::model::{
    ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
    PhysicalMethodId, PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const ORIGINAL_DEBUG_OUTER: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-statementful-loop-headers/original/debug/classes/jadx/tests/integration/loops/TestSequentialLoops.class"
);
const ORIGINAL_DEBUG_CHILD: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-statementful-loop-headers/original/debug/classes/jadx/tests/integration/loops/TestSequentialLoops$TestCls.class"
);
const ORIGINAL_NO_DEBUG_OUTER: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-statementful-loop-headers/original/no-debug/classes/jadx/tests/integration/loops/TestSequentialLoops.class"
);
const ORIGINAL_NO_DEBUG_CHILD: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-statementful-loop-headers/original/no-debug/classes/jadx/tests/integration/loops/TestSequentialLoops$TestCls.class"
);

const SUPPLEMENT_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac8-debug/classes/SequentialLoopEffects.class"
);
const SUPPLEMENT_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac8-no-debug/classes/SequentialLoopEffects.class"
);
const SUPPLEMENT_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac23-debug/classes/SequentialLoopEffects.class"
);
const SUPPLEMENT_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac23-no-debug/classes/SequentialLoopEffects.class"
);

const ORIGINAL_OWNER: &str = "jadx/tests/integration/loops/TestSequentialLoops$TestCls";
const FIELD_COPY_OWNER: &str = "FieldWriteCopies";

const FIELD_COPY_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-field-write-test-copies/javac8-debug/classes/FieldWriteCopies.class"
);
const FIELD_COPY_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-field-write-test-copies/javac8-no-debug/classes/FieldWriteCopies.class"
);
const FIELD_COPY_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-field-write-test-copies/javac23-debug/classes/FieldWriteCopies.class"
);
const FIELD_COPY_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-field-write-test-copies/javac23-no-debug/classes/FieldWriteCopies.class"
);

const SUPPLEMENT_OWNER: &str = "SequentialLoopEffects";

fn limits(analysis_steps: u64) -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
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
        analysis_steps,
        normalization_clones: 1 << 20,
        nested_depth: 32,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

struct Analyzed {
    analysis: jarde_jvm::method_ir::MethodIrAnalysis,
    facts: RecoveryFacts,
    method: PhysicalMethodId,
    instruction_bcis: Vec<u32>,
}

fn analyze_target(
    class: &[u8],
    owner: &str,
    name: &str,
    descriptor: &str,
    parameter_slots: u16,
    analysis_steps: u64,
) -> (Analyzed, Budget) {
    let mut budget = Budget::new(limits(analysis_steps));
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("frozen class opens");
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
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: u64::try_from(class.len()).expect("fixture length fits"),
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
    };
    let analysis = analyze_method_ir(
        &[snapshot.clone()],
        &MethodAnalysisRequest {
            environment: ResolutionEnvironment {
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
            },
            method: method.clone(),
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("the selected physical method analyzes");
    let declaration = analysis
        .ir()
        .declaration()
        .expect("the same read supplies the declaration");
    assert_eq!(String::from_utf8_lossy(&declaration.class_name().0), owner);
    assert_eq!(declaration.name().0.as_slice(), name.as_bytes());
    assert_eq!(declaration.descriptor().0.as_slice(), descriptor.as_bytes());
    assert_eq!(declaration.parameter_slots(), parameter_slots);
    assert!(!declaration.is_static());
    assert_eq!(declaration.identity(), &method);
    let facts = RecoveryFacts::new(
        MethodFacts::new(
            String::from_utf8_lossy(&declaration.name().0).into_owned(),
            String::from_utf8_lossy(&declaration.descriptor().0).into_owned(),
            declaration.parameter_slots(),
        )
        .with_access_flags(declaration.access_flags())
        .with_declaring_class(DeclaringClass::new(
            String::from_utf8_lossy(&declaration.class_name().0).into_owned(),
            declaration.class_access_flags(),
        )),
    );
    assert!(
        analysis
            .ir()
            .canonical()
            .expect("canonical CFG exists")
            .completeness()
            .is_complete()
    );
    let code = analysis.ir().code().expect("physical Code facts exist");
    assert!(code.stopped_at.is_none());
    assert!(matches!(&code.execution, ExecutionReport::Complete { .. }));
    let instruction_bcis = code
        .instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect::<Vec<_>>();
    assert!(!instruction_bcis.is_empty());
    assert!(instruction_bcis.windows(2).all(|pair| pair[0] < pair[1]));
    let mut ssa_bcis = analysis
        .ir()
        .ssa()
        .expect("SSA covers the method")
        .blocks()
        .iter()
        .flat_map(|block| {
            block
                .instructions()
                .iter()
                .map(|instruction| instruction.bci())
        })
        .collect::<Vec<_>>();
    ssa_bcis.sort_unstable();
    assert_eq!(ssa_bcis, instruction_bcis);
    (
        Analyzed {
            analysis,
            facts,
            method,
            instruction_bcis,
        },
        budget,
    )
}

fn recover_with(
    analyzed: &Analyzed,
    evidence: RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> RecoveryReport {
    recover(
        &RecoveryRequest::new(analyzed.analysis.ir(), &analyzed.facts, JAVA_8)
            .with_evidence(evidence),
        budget,
    )
}

fn recover_all(analyzed: &Analyzed, budget: &mut Budget) -> RecoveryReport {
    recover_with(analyzed, RecoveryEvidenceRequest::all(), budget)
}

fn assert_structured(label: &str, report: &RecoveryReport) {
    assert!(
        report.produced(),
        "{label}: {:?}\n{}",
        report.outcome,
        report.text
    );
    assert_eq!(
        report.quality,
        Quality::Structured,
        "{label}: {}",
        report.text
    );
    assert!(report.fallbacks.is_empty(), "{label}: {}", report.text);
    assert!(
        !report.text.contains("@bytecode"),
        "{label}: {}",
        report.text
    );
}

fn assert_source_origins(analyzed: &Analyzed, report: &RecoveryReport) {
    for segment in report.source_map.segments() {
        let start = segment.start();
        let end = segment.end();
        assert!(start <= end && end <= report.text.len());
        assert!(report.text.is_char_boundary(start));
        assert!(report.text.is_char_boundary(end));
        let _ = segment.text(&report.text);
        let origin = segment.origin();
        assert_eq!(origin.primary().method(), Some(&analyzed.method));
        assert!(analyzed.instruction_bcis.contains(&origin.primary().bci()));
        for derived in origin.derived() {
            assert_eq!(derived.method(), Some(&analyzed.method));
            assert!(analyzed.instruction_bcis.contains(&derived.bci()));
        }
    }
    for bci in &analyzed.instruction_bcis {
        assert!(
            !report.source_map.of_bci(*bci).is_empty(),
            "physical instruction BCI {bci} lacks a source origin\n{}",
            report.text
        );
    }
}

fn assert_text_map_parity(label: &str, analyzed: &Analyzed) -> RecoveryReport {
    let mut default_budget = Budget::new(limits(1 << 20));
    let default = recover_with(
        analyzed,
        RecoveryEvidenceRequest::essential(),
        &mut default_budget,
    );
    let mut map_budget = Budget::new(limits(1 << 20));
    let map_only = recover_with(
        analyzed,
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        &mut map_budget,
    );
    let mut all_budget = Budget::new(limits(1 << 20));
    let all = recover_all(analyzed, &mut all_budget);
    assert_structured(&format!("{label}/default"), &default);
    assert_structured(&format!("{label}/map-only"), &map_only);
    assert_structured(&format!("{label}/all"), &all);
    assert_eq!(default.text, all.text, "{label}: default/all text");
    assert_eq!(map_only.text, all.text, "{label}: map-only/all text");
    assert_eq!(
        map_only.source_map, all.source_map,
        "{label}: map-only/all map"
    );
    assert!(!all.source_map.is_empty(), "{label}: complete source map");
    assert_source_origins(analyzed, &all);
    all
}

fn assert_class_identity_and_members(
    class: &[u8],
    expected_name: &str,
    expected_fields: &[&str],
    expected_methods: &[(&str, &str)],
) {
    let mut budget = Budget::new(limits(1 << 20));
    let facts = class_facts(class, &mut budget).expect("fixture class header parses");
    assert_eq!(
        String::from_utf8_lossy(&facts.this_class.raw().0),
        expected_name
    );
    let fields = facts
        .fields
        .iter()
        .map(|field| String::from_utf8_lossy(&field.name.raw().0).into_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        fields,
        expected_fields
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>()
    );
    let methods = facts
        .methods
        .iter()
        .map(|method| {
            (
                String::from_utf8_lossy(&method.name.raw().0).into_owned(),
                String::from_utf8_lossy(&method.descriptor.raw().0).into_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        expected_methods
            .iter()
            .map(|(name, descriptor)| ((*name).to_owned(), (*descriptor).to_owned()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn original_sequential_loops_keep_both_statementful_headers_and_full_classes() {
    const CHILD_METHODS: [(&str, &str); 2] = [("<init>", "()V"), ("test", "(II)I")];
    const OUTER_METHODS: [(&str, &str); 2] = [("<init>", "()V"), ("test", "()V")];
    let mut constructor_text = None;
    for (profile, outer, child) in [
        ("debug", ORIGINAL_DEBUG_OUTER, ORIGINAL_DEBUG_CHILD),
        ("no-debug", ORIGINAL_NO_DEBUG_OUTER, ORIGINAL_NO_DEBUG_CHILD),
    ] {
        assert_class_identity_and_members(
            outer,
            "jadx/tests/integration/loops/TestSequentialLoops",
            &[],
            &OUTER_METHODS,
        );
        assert_class_identity_and_members(child, ORIGINAL_OWNER, &[], &CHILD_METHODS);
        let target = analyze_target(child, ORIGINAL_OWNER, "test", "(II)I", 3, 1 << 20).0;
        let report = assert_text_map_parity(&format!("original/{profile}/test"), &target);
        for bci in [14, 20] {
            assert!(target.instruction_bcis.contains(&bci));
            assert!(!report.source_map.of_bci(bci).is_empty());
        }
        assert_eq!(report.text.matches("while (").count(), 2, "{}", report.text);
        assert!(
            report.text.contains(" + "),
            "header arithmetic remains: {}",
            report.text
        );
        assert!(
            report.text.contains("if ("),
            "header decision remains: {}",
            report.text
        );
        assert!(report.text.contains("break;"), "{}", report.text);
        assert!(report.text.contains("return "), "{}", report.text);
        assert!(!report.text.contains("continue;"), "{}", report.text);

        let constructor = analyze_target(child, ORIGINAL_OWNER, "<init>", "()V", 1, 1 << 20).0;
        let constructor_report =
            assert_text_map_parity(&format!("original/{profile}/constructor"), &constructor);
        assert!(constructor_report.text.contains("super();"));
        assert!(constructor_report.text.contains("return;"));
        if let Some(previous) = &constructor_text {
            assert_eq!(
                previous, &constructor_report.text,
                "constructor stable across debug metadata"
            );
        } else {
            constructor_text = Some(constructor_report.text);
        }
    }
}

#[test]
fn effectful_sequential_loop_headers_preserve_complete_supplemental_methods() {
    const FIELDS: [&str; 14] = [
        "c",
        "z",
        "phase",
        "evaluations",
        "phaseOneCalls",
        "phaseTwoCalls",
        "lastInputC",
        "lastInputA",
        "lastValue",
        "throwAt",
        "overrideAt",
        "overrideValue",
        "phaseTrace",
        "marker",
    ];
    const METHODS: [(&str, &str); 3] = [("<init>", "()V"), ("next", "(II)I"), ("run", "(II)I")];
    for (profile, class) in [
        ("javac8-debug", SUPPLEMENT_JAVAC8_DEBUG),
        ("javac8-no-debug", SUPPLEMENT_JAVAC8_NO_DEBUG),
        ("javac23-debug", SUPPLEMENT_JAVAC23_DEBUG),
        ("javac23-no-debug", SUPPLEMENT_JAVAC23_NO_DEBUG),
    ] {
        assert_class_identity_and_members(class, SUPPLEMENT_OWNER, &FIELDS, &METHODS);
        let target = analyze_target(class, SUPPLEMENT_OWNER, "run", "(II)I", 3, 1 << 20).0;
        let report = assert_text_map_parity(&format!("supplement/{profile}/run"), &target);
        assert_eq!(report.text.matches("while (").count(), 2, "{}", report.text);
        let initial_effect = report
            .text
            .find("this.c = ")
            .expect("initial c store remains");
        let phase_effect = report
            .text
            .find("this.phase = 1;")
            .expect("phase setup remains");
        let first_loop = report.text.find("while (").expect("first loop remains");
        assert!(
            initial_effect < phase_effect && phase_effect < first_loop,
            "{}",
            report.text
        );
        let phase_two = report
            .text
            .find("this.phase = 2;")
            .expect("second-loop phase effect remains");
        let second_loop = report.text.rfind("while (").expect("second loop remains");
        assert!(
            first_loop < phase_two && phase_two < second_loop,
            "{}",
            report.text
        );
        assert!(
            report.text.matches("next(").count() >= 2,
            "both loop evaluations remain: {}",
            report.text
        );
        assert!(
            report.text.contains("return "),
            "method result remains: {}",
            report.text
        );
        for (name, descriptor, slots) in [("<init>", "()V", 1), ("next", "(II)I", 3)] {
            let method =
                analyze_target(class, SUPPLEMENT_OWNER, name, descriptor, slots, 1 << 20).0;
            assert_text_map_parity(&format!("supplement/{profile}/{name}"), &method);
        }
    }
}

fn assert_saved_volatile_write_is_reused(label: &str, report: &RecoveryReport, threshold: i32) {
    let stores = report
        .text
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("z = "))
        .collect::<Vec<_>>();
    assert_eq!(
        stores.len(),
        1,
        "{label}: expected one write to z\n{}",
        report.text
    );
    let (store_index, store_line) = stores[0];
    let saved = store_line
        .split_once("z = ")
        .expect("store line names z")
        .1
        .trim()
        .trim_end_matches(';')
        .trim();
    assert!(
        !saved.is_empty() && !saved.contains('('),
        "{label}: RHS is not a saved value: {store_line}"
    );
    assert!(
        report
            .text
            .lines()
            .any(|line| { line.contains(&format!("int {saved} = ")) && line.contains("next(") }),
        "{label}: saved RHS declaration missing\n{}",
        report.text
    );
    let predicates = [
        format!("{saved} < {threshold}"),
        format!("{saved} >= {threshold}"),
    ];
    let predicate = predicates
        .iter()
        .find(|predicate| report.text.contains(predicate.as_str()))
        .expect("saved value is the operand of the original or inverted comparison");
    assert!(
        report.text.contains(predicate.as_str()),
        "{label}: comparison does not reuse the saved RHS `{predicate}`\n{}",
        report.text
    );
    let lines = report.text.lines().collect::<Vec<_>>();
    assert!(
        lines
            .iter()
            .skip(store_index + 1)
            .any(|line| line.contains(predicate.as_str())),
        "{label}: comparison must follow the field write\n{}",
        report.text
    );
    let z_lines = lines
        .iter()
        .filter(|line| line.contains("z"))
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        z_lines,
        vec![store_line],
        "{label}: volatile field was read again\n{}",
        report.text
    );
}

#[test]
fn saved_field_write_copies_cover_both_compilers_and_debug_profiles() {
    const FIELDS: [&str; 6] = ["z", "calls", "ledger", "throwAt", "marker", "lastValue"];
    const METHODS: [(&str, &str); 4] = [
        ("<init>", "()V"),
        ("next", "(I)I"),
        ("assignAndCompare", "(I)Z"),
        ("statefulWhile", "(I)I"),
    ];
    for (profile, class) in [
        ("javac8-debug", FIELD_COPY_JAVAC8_DEBUG),
        ("javac8-no-debug", FIELD_COPY_JAVAC8_NO_DEBUG),
        ("javac23-debug", FIELD_COPY_JAVAC23_DEBUG),
        ("javac23-no-debug", FIELD_COPY_JAVAC23_NO_DEBUG),
    ] {
        assert_class_identity_and_members(class, FIELD_COPY_OWNER, &FIELDS, &METHODS);
        let mut class_budget = Budget::new(limits(1 << 20));
        let header = class_facts(class, &mut class_budget).expect("class facts parse");
        let volatile_fields = header
            .fields
            .iter()
            .filter(|field| field.access_flags & 0x0040 != 0)
            .map(|field| String::from_utf8_lossy(&field.name.raw().0).into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            volatile_fields,
            vec!["z".to_owned()],
            "{profile}: actual volatile field inventory"
        );
        let z = header
            .fields
            .iter()
            .find(|field| field.name.raw().0.as_slice() == b"z")
            .unwrap();
        assert_eq!(
            z.descriptor.raw().0.as_slice(),
            b"I",
            "{profile}: z descriptor"
        );

        for (name, descriptor, slots) in [
            ("<init>", "()V", 1),
            ("next", "(I)I", 2),
            ("assignAndCompare", "(I)Z", 2),
            ("statefulWhile", "(I)I", 2),
        ] {
            let analyzed =
                analyze_target(class, FIELD_COPY_OWNER, name, descriptor, slots, 1 << 20).0;
            let report = assert_text_map_parity(&format!("field-copy/{profile}/{name}"), &analyzed);
            if name == "assignAndCompare" {
                assert_saved_volatile_write_is_reused("assignAndCompare", &report, 5);
            } else if name == "statefulWhile" {
                assert_saved_volatile_write_is_reused("statefulWhile", &report, 3);
            }
        }
    }
}

#[test]
fn statementful_loop_late_budget_cancellation_and_deadline_publish_no_partial_result() {
    let class = SUPPLEMENT_JAVAC8_DEBUG;
    let (complete, mut full_budget) =
        analyze_target(class, SUPPLEMENT_OWNER, "run", "(II)I", 3, 1 << 20);
    let analysis_steps = full_budget.usage().analysis_steps;
    let full = recover_all(&complete, &mut full_budget);
    assert_structured("complete reference", &full);
    let total_steps = full_budget.usage().analysis_steps;
    assert!(
        total_steps > analysis_steps + 1,
        "reference did not perform recovery work"
    );

    let (limited, mut limited_budget) =
        analyze_target(class, SUPPLEMENT_OWNER, "run", "(II)I", 3, total_steps - 1);
    assert_eq!(limited_budget.usage().analysis_steps, analysis_steps);
    let stopped = recover_all(&limited, &mut limited_budget);
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(limits(1 << 20), token);
    let cancelled = recover_all(&complete, &mut cancelled_budget);
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));

    let mut expired_limits = limits(1 << 20);
    expired_limits.elapsed_millis = 1;
    let mut expired_budget = Budget::new(expired_limits);
    std::thread::sleep(std::time::Duration::from_millis(2));
    let expired = recover_all(&complete, &mut expired_budget);
    assert!(!expired.produced());
    assert!(expired.text.is_empty() && expired.source_map.is_empty());
    assert!(matches!(
        expired.stop(),
        Some(StopReason::Interrupted { code, .. }) if *code == "jre_budget_interrupted"
    ));
}
