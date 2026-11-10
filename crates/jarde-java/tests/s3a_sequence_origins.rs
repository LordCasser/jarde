//! 被拒绝的读取者仍须在来源引用中保留产生该值的栈复制指令。

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_jvm::method_ir::{Definition, RefType, Slot, Value};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::{CpEntryKind, cp_entry};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 32,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 4,
        method_bodies: 4,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 16,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn u16_be(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn u32_be(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn utf8(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.push(1);
    u16_be(
        bytes,
        u16::try_from(value.len()).expect("fixture UTF8 length"),
    );
    bytes.extend_from_slice(value);
}

fn method(
    bytes: &mut Vec<u8>,
    name_index: u16,
    descriptor_index: u16,
    code: &[u8],
    max_stack: u16,
) {
    method_with_locals(bytes, name_index, descriptor_index, code, 0, max_stack);
}

fn method_with_locals(
    bytes: &mut Vec<u8>,
    name_index: u16,
    descriptor_index: u16,
    code: &[u8],
    max_locals: u16,
    max_stack: u16,
) {
    u16_be(bytes, 0x0009); // public static
    u16_be(bytes, name_index);
    u16_be(bytes, descriptor_index);
    u16_be(bytes, 1); // Code 属性
    u16_be(bytes, 8); // Code
    let mut content = Vec::new();
    u16_be(&mut content, max_stack); // max_stack
    u16_be(&mut content, max_locals); // max_locals
    u32_be(
        &mut content,
        u32::try_from(code.len()).expect("fixture method length"),
    );
    content.extend_from_slice(code);
    u16_be(&mut content, 0); // 异常表
    u16_be(&mut content, 0); // Code 属性
    u32_be(
        bytes,
        u32::try_from(content.len()).expect("Code attribute length"),
    );
    bytes.extend_from_slice(&content);
}

fn method_with_handler(
    bytes: &mut Vec<u8>,
    name_index: u16,
    descriptor_index: u16,
    code: &[u8],
    max_stack: u16,
    handler: (u16, u16, u16, u16),
) {
    u16_be(bytes, 0x0009);
    u16_be(bytes, name_index);
    u16_be(bytes, descriptor_index);
    u16_be(bytes, 1);
    u16_be(bytes, 8); // Code
    let mut content = Vec::new();
    u16_be(&mut content, max_stack);
    u16_be(&mut content, 0);
    u32_be(&mut content, code.len() as u32);
    content.extend_from_slice(code);
    u16_be(&mut content, 1);
    for value in [handler.0, handler.1, handler.2, handler.3] {
        u16_be(&mut content, value);
    }
    u16_be(&mut content, 0);
    u32_be(bytes, content.len() as u32);
    bytes.extend_from_slice(&content);
}

fn fixture() -> Vec<u8> {
    let mut bytes = Vec::new();
    u32_be(&mut bytes, 0xcafebabe);
    u16_be(&mut bytes, 0);
    u16_be(&mut bytes, 52);
    u16_be(&mut bytes, 13); // 常量池计数
    utf8(&mut bytes, b"Test"); // 1
    bytes.push(7);
    u16_be(&mut bytes, 1); // 2：Test 类
    utf8(&mut bytes, b"java/lang/Object"); // 3
    bytes.push(7);
    u16_be(&mut bytes, 3); // 4：Object 类
    utf8(&mut bytes, b"source"); // 5
    utf8(&mut bytes, b"()Ljava/lang/Object;"); // 6
    utf8(&mut bytes, b"test"); // 7
    utf8(&mut bytes, b"Code"); // 8
    utf8(&mut bytes, b"p/Bad-Name"); // 9：JVM 内部名合法，但不能写成 Java 类型名
    bytes.push(7);
    u16_be(&mut bytes, 9); // 10: Class p/Bad-Name
    bytes.push(12);
    u16_be(&mut bytes, 5);
    u16_be(&mut bytes, 6); // 11：source:()Ljava/lang/Object; 名称类型
    bytes.push(10);
    u16_be(&mut bytes, 2);
    u16_be(&mut bytes, 11); // 12：Test.source 方法引用

    u16_be(&mut bytes, 0x0021); // public、super
    u16_be(&mut bytes, 2);
    u16_be(&mut bytes, 4);
    u16_be(&mut bytes, 0); // 接口
    u16_be(&mut bytes, 0); // 字段
    u16_be(&mut bytes, 2); // 方法
    method(&mut bytes, 5, 6, &[0x01, 0xb0], 1); // source：aconst_null；areturn
    // BCI：invokestatic=0..2，dup=3，checkcast=4..6，areturn=7。
    method(&mut bytes, 7, 6, &[0xb8, 0, 12, 0x59, 0xc0, 0, 10, 0xb0], 2);
    u16_be(&mut bytes, 0); // 类属性
    bytes
}

fn consumer_fixture(with_handler: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    u32_be(&mut bytes, 0xcafebabe);
    u16_be(&mut bytes, 0);
    u16_be(&mut bytes, 52);
    u16_be(&mut bytes, 16); // 常量池计数
    utf8(&mut bytes, b"Test"); // 1
    bytes.push(7);
    u16_be(&mut bytes, 1); // 2
    utf8(&mut bytes, b"java/lang/Object"); // 3
    bytes.push(7);
    u16_be(&mut bytes, 3); // 4
    utf8(&mut bytes, b"source"); // 5
    utf8(&mut bytes, b"()Ljava/lang/Object;"); // 6
    utf8(&mut bytes, b"test"); // 7
    utf8(&mut bytes, b"Code"); // 8
    utf8(&mut bytes, b"()V"); // 9
    utf8(&mut bytes, b"sink"); // 10
    utf8(&mut bytes, b"(Ljava/lang/Object;)V"); // 11
    bytes.push(12);
    u16_be(&mut bytes, 5);
    u16_be(&mut bytes, 6); // 12
    bytes.push(10);
    u16_be(&mut bytes, 2);
    u16_be(&mut bytes, 12); // 13: source()
    bytes.push(12);
    u16_be(&mut bytes, 10);
    u16_be(&mut bytes, 11); // 14
    bytes.push(10);
    u16_be(&mut bytes, 2);
    u16_be(&mut bytes, 14); // 15: sink(Object)

    u16_be(&mut bytes, 0x0021);
    u16_be(&mut bytes, 2);
    u16_be(&mut bytes, 4);
    u16_be(&mut bytes, 0);
    u16_be(&mut bytes, 0);
    u16_be(&mut bytes, 3); // methods
    method(&mut bytes, 5, 6, &[0x01, 0xb0], 1);
    let caller = [
        0xb8, 0, 13, 0x59, 0xb8, 0, 15, 0xb8, 0, 15, 0xb1, 0x57, 0xb1,
    ];
    if with_handler {
        method_with_handler(&mut bytes, 7, 9, &caller, 2, (0, 7, 11, 0));
    } else {
        method(&mut bytes, 7, 9, &caller[..11], 2);
    }
    method(&mut bytes, 10, 11, &[0x57, 0xb1], 1);
    u16_be(&mut bytes, 0);
    bytes
}

fn multi_consumer_fixture() -> Vec<u8> {
    consumer_fixture(false)
}

fn copied_value_crossing_protected_end_fixture() -> Vec<u8> {
    consumer_fixture(true)
}

fn typed_factory_fixture(
    factory_descriptor: &[u8],
    test_code: &[u8],
    factory_locals: u16,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    u32_be(&mut bytes, 0xcafebabe);
    u16_be(&mut bytes, 0);
    u16_be(&mut bytes, 52);
    u16_be(&mut bytes, 14); // 常量池计数
    utf8(&mut bytes, b"Test"); // 1
    bytes.push(7);
    u16_be(&mut bytes, 1); // 2：Test 类
    utf8(&mut bytes, b"java/lang/Object"); // 3
    bytes.push(7);
    u16_be(&mut bytes, 3); // 4：Object 类
    utf8(&mut bytes, b"factory"); // 5
    utf8(&mut bytes, factory_descriptor); // 6
    utf8(&mut bytes, b"test"); // 7
    utf8(&mut bytes, b"Code"); // 8
    utf8(&mut bytes, b"p/Box"); // 9：只由 factory descriptor 提及
    bytes.push(7);
    u16_be(&mut bytes, 9); // 10：p/Box 类常量
    bytes.push(12);
    u16_be(&mut bytes, 5);
    u16_be(&mut bytes, 6); // 11：factory 的名称和 descriptor
    bytes.push(10);
    u16_be(&mut bytes, 2);
    u16_be(&mut bytes, 11); // 12：Test.factory 方法引用
    utf8(&mut bytes, b"()Ljava/lang/Object;"); // 13：test 方法 descriptor

    u16_be(&mut bytes, 0x0021);
    u16_be(&mut bytes, 2);
    u16_be(&mut bytes, 4);
    u16_be(&mut bytes, 0); // 接口
    u16_be(&mut bytes, 0); // 字段
    u16_be(&mut bytes, 2); // 方法
    method_with_locals(
        &mut bytes,
        5,
        6,
        &[0x01, 0xb0], // aconst_null; areturn
        factory_locals,
        1,
    );
    method(&mut bytes, 7, 13, test_code, 1);
    u16_be(&mut bytes, 0); // 类属性
    bytes
}

fn analyze_test(class_bytes: &[u8]) -> jarde_jvm::method_ir::MethodIrAnalysis {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class_bytes.to_vec()), &mut budget)
        .expect("producer fixture opens");
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
    analyze_method_ir(
        std::slice::from_ref(&snapshot),
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
            method: PhysicalMethodId {
                owner: PhysicalDefinitionId {
                    location: PhysicalClassLocation::StandaloneRoot {
                        snapshot: snapshot.id().clone(),
                    },
                    class_bytes: ClassBytesId {
                        digest: Digest(blake3::hash(class_bytes).to_hex().to_string()),
                        length: class_bytes.len() as u64,
                    },
                    variant: PhysicalVariant::Base,
                },
                name: JvmBytes(b"test".to_vec()),
                descriptor: JvmBytes(b"()Ljava/lang/Object;".to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("producer test method analysis succeeds")
}

fn recover_test_with_descriptor(
    class_bytes: &[u8],
    descriptor: &str,
) -> jarde_java::RecoveryReport {
    recover_test_with_recovery_budget(class_bytes, descriptor, None)
}

fn recover_test_with_recovery_budget(
    class_bytes: &[u8],
    descriptor: &str,
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class_bytes.to_vec()), &mut budget)
        .expect("fixture opens");
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
    let analysis = analyze_method_ir(
        std::slice::from_ref(&snapshot),
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
            method: PhysicalMethodId {
                owner: PhysicalDefinitionId {
                    location: PhysicalClassLocation::StandaloneRoot {
                        snapshot: snapshot.id().clone(),
                    },
                    class_bytes: ClassBytesId {
                        digest: Digest(blake3::hash(class_bytes).to_hex().to_string()),
                        length: class_bytes.len() as u64,
                    },
                    variant: PhysicalVariant::Base,
                },
                name: JvmBytes(b"test".to_vec()),
                descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("analysis request is valid");
    let facts = RecoveryFacts::new(
        MethodFacts::new("test", descriptor, 0)
            .with_access_flags(0x0009)
            .with_declaring_class(DeclaringClass::new("Test", 0x0031)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(
            analysis.ir(),
            &facts,
            JAVA_8,
            RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
        )
        .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

fn recover_test(class_bytes: &[u8]) -> jarde_java::RecoveryReport {
    recover_test_with_descriptor(class_bytes, "()Ljava/lang/Object;")
}

#[test]
fn refused_checkcast_quote_retains_the_stack_copy_origin() {
    // 只分析并恢复类文件字节，绝不执行夹具。
    let report = recover_test(&fixture());
    let quoted = report
        .source_map
        .segments()
        .iter()
        .filter(|segment| {
            segment
                .text(&report.text)
                .trim_start()
                .starts_with("// @bytecode")
        })
        .flat_map(|segment| segment.origin().bcis())
        .collect::<Vec<_>>();
    assert!(report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        quoted.contains(&3),
        "the rejected reader quote must retain dup at BCI 3; quoted={quoted:?}\n{}",
        report.text
    );
}

#[test]
fn copied_producer_with_two_renderable_consumers_is_evaluated_once() {
    // 仅解析、恢复 classfile；不执行目标。
    let report = recover_test_with_descriptor(&multi_consumer_fixture(), "()V");
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(
        report.quality,
        jarde_jvm::ir::Quality::Structured,
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        report.text.matches("source()").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(report.text.matches("sink(").count(), 2, "{}", report.text);
    assert!(report.text.contains("saved0"), "{}", report.text);
}

#[test]
fn copied_binding_budget_and_cancellation_publish_no_partial_method() {
    let class = multi_consumer_fixture();
    let full = recover_test_with_recovery_budget(&class, "()V", Some(Budget::new(limits())));
    let jarde_reader::model::ExecutionReport::Complete { usage } = full.execution else {
        panic!("complete copied binding expected, got {:?}", full.execution);
    };
    assert!(usage.analysis_steps > 1);

    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover_test_with_recovery_budget(&class, "()V", Some(Budget::new(late)));
    assert!(!stopped.produced(), "{}", stopped.text);
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(stopped.stop().is_some());

    let token = jarde_reader::budget::CancellationToken::new();
    token.cancel();
    let cancelled = recover_test_with_recovery_budget(
        &class,
        "()V",
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(
        cancelled
            .stop()
            .is_some_and(jarde_java::StopReason::is_cancelled)
    );
}

#[test]
fn copied_value_with_consumers_on_both_sides_of_handler_end_is_not_mis_scoped() {
    let report =
        recover_test_with_descriptor(&copied_value_crossing_protected_end_fixture(), "()V");
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(
        report.quality,
        jarde_jvm::ir::Quality::Fallback,
        "{}",
        report.text
    );
    assert!(report.text.contains("@bytecode"), "{}", report.text);
    assert!(!report.text.contains("saved0"), "{}", report.text);
}

fn assert_producer_descriptor_anchor(
    class_bytes: &[u8],
    producer_bci: u32,
    descriptor: &[u8],
    target_binary: &[u8],
    expected_value_type: &[u8],
    expected_anchor: Option<u32>,
) {
    let analysis = analyze_test(class_bytes);
    let ir = analysis.ir();
    let code = ir.code().unwrap_or_else(|| {
        panic!(
            "analyzed method has no reader Code facts: {:?}",
            analysis.report()
        )
    });
    let ssa = ir
        .ssa()
        .unwrap_or_else(|| panic!("analyzed method has no SSA: {:?}", analysis.report()));
    let instruction = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == producer_bci)
        .expect("producer BCI exists in the real SSA");
    let [(Slot::Stack(_), value)] = instruction.writes() else {
        panic!("producer must define exactly one stack value: {instruction:?}");
    };
    assert!(matches!(
        ssa.value(*value).def(),
        Definition::Instruction { bci, .. } if *bci == producer_bci
    ));
    assert!(
        matches!(
            ssa.value(*value).ty(),
            Value::Ref(RefType::Named { name, .. }) if name == expected_value_type
        ),
        "producer SSA type is {:?}, expected reference {:?}",
        ssa.value(*value).ty(),
        expected_value_type
    );
    let body_instruction = code
        .instructions
        .iter()
        .find(|instruction| instruction.bci == producer_bci)
        .expect("producer BCI exists in reader Code facts");
    assert_eq!(
        body_instruction.opcode, 0xb8,
        "the producer is invokestatic"
    );
    let pool_index = body_instruction
        .constant_pool_index
        .expect("invocation names a real constant-pool entry");
    let entry = cp_entry(ir.constant_pool(), pool_index).expect("real pool entry resolves");
    assert!(matches!(
        &entry.kind,
        CpEntryKind::MethodRef { descriptor: found, .. }
            | CpEntryKind::InterfaceMethodRef { descriptor: found, .. }
            if found.0 == descriptor
    ));
    let anchor = jarde_java::source_proof::static_fold_stored_type_anchor(
        Some(ssa),
        code,
        ir.constant_pool(),
        &std::collections::BTreeSet::from([producer_bci]),
        target_binary,
    );
    assert_eq!(anchor, expected_anchor);
}

#[test]
fn no_store_producer_is_anchored_only_by_its_return_descriptor() {
    // The call itself has one real SSA stack result and its reader Methodref returns p/Box.
    // No astore exists between the call and the method return.
    let returns_box = typed_factory_fixture(b"()Lp/Box;", &[0xb8, 0, 12, 0xb0], 0);
    assert_producer_descriptor_anchor(&returns_box, 0, b"()Lp/Box;", b"p/Box", b"Lp/Box;", Some(0));

    // The parameter mentions p/Box, but the real call result is Object. That input type must not
    // become an anchor for a synthetic saved local of p/Box.
    let box_parameter_only = typed_factory_fixture(
        b"(Lp/Box;)Ljava/lang/Object;",
        &[0x01, 0xb8, 0, 12, 0xb0],
        1,
    );
    assert_producer_descriptor_anchor(
        &box_parameter_only,
        1,
        b"(Lp/Box;)Ljava/lang/Object;",
        b"p/Box",
        b"Ljava/lang/Object;",
        None,
    );
}
