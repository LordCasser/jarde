//! Reader-backed checks for the exact saved dup_x1/putfield/direct-compare proof.
use super::*;
use jarde_reader::budget::{CancellationToken, CountedBudgetDimension, Limits};

const SEQ1_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac8-debug/classes/SequentialLoopEffects.class"
);
const SEQ1_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac8-no-debug/classes/SequentialLoopEffects.class"
);
const SEQ1_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac23-debug/classes/SequentialLoopEffects.class"
);
const SEQ1_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac23-no-debug/classes/SequentialLoopEffects.class"
);

const FIELD_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-field-write-test-copies/javac8-debug/classes/FieldWriteCopies.class"
);
const FIELD_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-field-write-test-copies/javac8-no-debug/classes/FieldWriteCopies.class"
);
const FIELD_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-field-write-test-copies/javac23-debug/classes/FieldWriteCopies.class"
);
const FIELD_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-field-write-test-copies/javac23-no-debug/classes/FieldWriteCopies.class"
);

const REFUSALS_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-field-write-test-copies/javac8-debug/classes/FieldWriteCopyRefusals.class"
);
const REFUSALS_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-field-write-test-copies/javac8-no-debug/classes/FieldWriteCopyRefusals.class"
);
const REFUSALS_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-field-write-test-copies/javac23-debug/classes/FieldWriteCopyRefusals.class"
);
const REFUSALS_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-field-write-test-copies/javac23-no-debug/classes/FieldWriteCopyRefusals.class"
);

fn field_copy_limits(analysis_steps: u64, elapsed_millis: u64) -> Limits {
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
        class_headers: 100,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps,
        normalization_clones: 1 << 20,
        nested_depth: 32,
        dependency_depth: 32,
        elapsed_millis,
    }
}

fn field_copy_budget(analysis_steps: u64, elapsed_millis: u64) -> Budget {
    Budget::new(field_copy_limits(analysis_steps, elapsed_millis))
}

fn with_field_method<R>(
    class: &[u8],
    owner: &str,
    method_name: &str,
    descriptor: &str,
    f: impl FnOnce(&jarde_jvm::method_ir::MethodIr, &Operations, &field::Plan) -> R,
) -> R {
    let (analysis, _) = analyze_test_method(class, method_name, descriptor, 8);
    let ir = analysis.ir();
    let code = ir.code().expect("the physical method has complete Code");
    let operations = Operations::of(code, ir.constant_pool());
    let declaring = crate::facts::DeclaringClass::new(owner, 0x0031);
    let fields = field::plan(
        ir.ssa().expect("Reader supplied SSA"),
        &operations,
        &|_| None,
        Some(&declaring),
        method_name,
        descriptor,
        ir.class_fields(),
        &[],
        &mut child_array_test_budget(),
    )
    .expect("Reader field symbols and this-class declaration form a real field plan");
    f(ir, &operations, &fields)
}

fn dup_x1_locations(
    ir: &jarde_jvm::method_ir::MethodIr,
    operations: &Operations,
) -> Vec<(CanonicalBlockId, usize)> {
    let ssa = ir.ssa().expect("SSA");
    ssa.blocks()
        .iter()
        .flat_map(|block| {
            block
                .instructions()
                .iter()
                .enumerate()
                .filter(|(_, instruction)| {
                    instruction.opcode() == OPCODE_DUP_X1
                        && matches!(operations.get(instruction.bci()), Some(Operation::Other))
                })
                .map(|(index, _)| (block.block().clone(), index))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn assert_single_use_at(ssa: &SsaTable, value: ValueId, bci: u32) {
    let uses = ssa.value(value).uses();
    assert_eq!(
        uses.len(),
        1,
        "value {value:?} has exactly one consumer: {uses:?}"
    );
    assert_eq!(
        uses[0].bci(),
        Some(bci),
        "value {value:?} is consumed at BCI {bci}"
    );
}

fn prove_and_check_saved_test_copy(
    ir: &jarde_jvm::method_ir::MethodIr,
    operations: &Operations,
    fields: &field::Plan,
    expected_copy_bci: Option<u32>,
) -> u32 {
    let ssa = ir.ssa().expect("SSA");
    let canonical = ir.canonical().expect("canonical cfg");
    let locations = dup_x1_locations(ir, operations);
    assert_eq!(
        locations.len(),
        1,
        "one direct dup_x1 candidate in this method: {locations:?}"
    );
    let (block_id, index) = &locations[0];
    let block = ssa
        .block(block_id)
        .expect("candidate block is in the same SSA table");
    let copy_instruction = &block.instructions()[*index];
    let copy_bci = copy_instruction.bci();
    if let Some(expected) = expected_copy_bci {
        assert_eq!(
            copy_bci, expected,
            "the original Seq1 BCI is pinned by its fixture"
        );
    }
    let code = ir.code().expect("physical Code");
    let physical_copy = code
        .instructions
        .iter()
        .find(|instruction| instruction.bci == copy_bci)
        .unwrap();
    assert_eq!(physical_copy.opcode, OPCODE_DUP_X1);
    assert_eq!(operations.get(copy_bci), Some(&Operation::Other));

    let mut budget = child_array_test_budget();
    let plan = FieldCopies::prove(ssa, canonical, operations, fields, true, &mut budget)
        .expect("the complete real method field-copy plan finishes");
    let copy = plan
        .copy_at(copy_bci)
        .expect("the actual test-copy is committed into the plan");
    let FieldCopyShape::Chain {
        stores,
        lead,
        saved,
    } = &copy.shape
    else {
        panic!(
            "test-copy is represented by the existing single-store chain carrier: {:?}",
            copy.shape
        );
    };
    assert_eq!(stores.len(), 1);
    assert!(
        *lead && *saved,
        "RHS is evaluated once into the existing saved lead"
    );
    assert!(copy.pass_through.is_none());
    let (moved_receiver, below) = copy
        .moved
        .as_ref()
        .copied()
        .expect("dup_x1 keeps the moved this receiver and lower test copy");

    let (source, below_source) =
        dup_x1_operands(copy_instruction).expect("physical dup_x1 has two stack inputs");
    assert_eq!(copy.source, source);
    let Definition::Instruction {
        bci: source_bci,
        block: source_block,
    } = ssa.value(source).def()
    else {
        panic!("the once-evaluated RHS is produced in this method body");
    };
    assert_eq!(source_block, block.block());
    let source_index = block
        .instructions()
        .iter()
        .position(|instruction| instruction.bci() == *source_bci)
        .unwrap();
    assert_eq!(
        source_index + 1,
        *index,
        "RHS evaluation is the instruction immediately before dup_x1"
    );
    assert!(matches!(
        operations.get(*source_bci),
        Some(Operation::Invoke(_))
    ));
    let (below_copy, moved_from_stack, copied_store_value) =
        dup_x1_writes(copy_instruction, below_source, source)
            .expect("dup_x1 has its exact three SSA outputs");
    assert_eq!(moved_receiver, moved_from_stack);
    assert_eq!(below, below_source);

    let store_bci = stores[0];
    let store_index = block
        .instructions()
        .iter()
        .position(|instruction| instruction.bci() == store_bci)
        .unwrap();
    let store_instruction = &block.instructions()[store_index];
    assert_eq!(store_instruction.opcode(), 0xb5);
    let store_fact = code
        .instructions
        .iter()
        .find(|instruction| instruction.bci == store_bci)
        .unwrap();
    assert_eq!(store_fact.opcode, 0xb5);
    assert!(
        matches!(operations.get(store_bci), Some(Operation::Field { access: FieldAccess::Write, is_static: false, descriptor, .. }) if descriptor == "I")
    );
    let (evidence, store_shape) = fields
        .claim(store_bci)
        .expect("the store has Reader field evidence");
    assert_eq!(evidence.descriptor, "I");
    assert_eq!(evidence.access, FieldAccess::Write);
    assert!(store_shape.writes());

    let constant = block
        .instructions()
        .get(*index + 2)
        .expect("one constant follows putfield");
    let compare = block
        .instructions()
        .get(*index + 3)
        .expect("direct comparison follows the constant");
    assert_eq!(
        block
            .instructions()
            .get(*index + 1)
            .map(SsaInstruction::bci),
        Some(store_bci)
    );
    assert!(matches!(
        operations.get(constant.bci()),
        Some(Operation::Push(ConstantValue::Int(_)))
    ));
    assert!(
        matches!(operations.get(compare.bci()), Some(Operation::Comparison { op, .. }) if op.reads_two())
    );
    assert!(matches!(compare.opcode(), 0x9f..=0xa4));
    let physical_compare = code
        .instructions
        .iter()
        .find(|instruction| instruction.bci == compare.bci())
        .unwrap();
    assert!(matches!(physical_compare.opcode, 0x9f..=0xa4));
    assert_eq!(
        block.instructions().last().map(SsaInstruction::bci),
        Some(compare.bci())
    );

    let constant_value = one_stack_output(constant)
        .expect("the literal has one stack result")
        .1;
    let operands = stack_operands(compare);
    assert_eq!(operands.len(), 2);
    assert!(operands.iter().any(|(_, value)| *value == below_copy));
    assert!(operands.iter().any(|(_, value)| *value == constant_value));
    assert_eq!(copy.source, source);
    assert_single_use_at(ssa, source, copy_bci);
    assert_single_use_at(ssa, below_copy, compare.bci());
    assert_single_use_at(ssa, moved_receiver, store_bci);
    assert_single_use_at(ssa, copied_store_value, store_bci);
    assert_single_use_at(ssa, constant_value, compare.bci());

    let Definition::Instruction {
        bci: receiver_bci, ..
    } = ssa.value(below).def()
    else {
        panic!("moved receiver is produced by an actual this load");
    };
    assert!(matches!(
        operations.get(*receiver_bci),
        Some(Operation::Load { slot: 0 })
    ));
    let receiver_load = block
        .instructions()
        .iter()
        .find(|instruction| instruction.bci() == *receiver_bci)
        .unwrap();
    let local_zero = local_read(receiver_load, 0).expect("aload_0 reads the actual receiver local");
    assert!(matches!(
        ssa.value(local_zero).def(),
        Definition::Entry {
            slot: Slot::Local(0),
            ..
        }
    ));
    assert_single_use_at(ssa, below, copy_bci);
    copy_bci
}

fn assert_helper_refuses(class: &[u8], owner: &str, method: &str, descriptor: &str) {
    with_field_method(
        class,
        owner,
        method,
        descriptor,
        |ir, operations, fields| {
            let locations = dup_x1_locations(ir, operations);
            assert!(
                !locations.is_empty(),
                "the real method contains a dup_x1 test candidate"
            );
            let ssa = ir.ssa().unwrap();
            for (block_id, index) in locations {
                let block = ssa.block(&block_id).unwrap();
                let mut budget = child_array_test_budget();
                assert!(
                    field_test_copy_at(ssa, operations, fields, true, block, index, &mut budget)
                        .expect("candidate refusal is local and not a Stop")
                        .is_none(),
                    "the new exact helper refuses {owner}.{method}{descriptor} at BCI {}",
                    block.instructions()[index].bci()
                );
            }
        },
    );
}

#[test]
fn real_seq1_and_volatile_methods_have_one_saved_single_store_copy() {
    for (class, copy_bci) in [
        (SEQ1_JAVAC8_DEBUG, Some(61)),
        (SEQ1_JAVAC8_NO_DEBUG, Some(61)),
        (SEQ1_JAVAC23_DEBUG, Some(61)),
        (SEQ1_JAVAC23_NO_DEBUG, Some(61)),
    ] {
        with_field_method(
            class,
            "SequentialLoopEffects",
            "run",
            "(II)I",
            |ir, operations, fields| {
                prove_and_check_saved_test_copy(ir, operations, fields, copy_bci);
            },
        );
    }

    for class in [
        FIELD_JAVAC8_DEBUG,
        FIELD_JAVAC8_NO_DEBUG,
        FIELD_JAVAC23_DEBUG,
        FIELD_JAVAC23_NO_DEBUG,
    ] {
        for (method, descriptor) in [("assignAndCompare", "(I)Z"), ("statefulWhile", "(I)I")] {
            with_field_method(
                class,
                "FieldWriteCopies",
                method,
                descriptor,
                |ir, operations, fields| {
                    let z = ir
                        .class_fields()
                        .unwrap()
                        .iter()
                        .find(|field| {
                            field.name.raw().0.as_slice() == b"z"
                                && field.descriptor.raw().0.as_slice() == b"I"
                        })
                        .unwrap();
                    assert_ne!(
                        z.access_flags & 0x0040,
                        0,
                        "z is the actual volatile int field in the fixture"
                    );
                    prove_and_check_saved_test_copy(ir, operations, fields, None);
                },
            );
        }
    }
}

#[test]
fn real_other_receivers_non_int_fields_and_handler_rows_refuse_the_new_helper() {
    for class in [
        REFUSALS_JAVAC8_DEBUG,
        REFUSALS_JAVAC8_NO_DEBUG,
        REFUSALS_JAVAC23_DEBUG,
        REFUSALS_JAVAC23_NO_DEBUG,
    ] {
        assert_helper_refuses(
            class,
            "FieldWriteCopyRefusals",
            "differentReceiver",
            "(LFieldWriteCopyRefusals;I)Z",
        );
        assert_helper_refuses(class, "FieldWriteCopyRefusals", "nonIntField", "(ZZ)Z");
        assert_helper_refuses(class, "FieldWriteCopyRefusals", "handledAssignment", "(I)Z");
    }
}

#[test]
fn exact_new_proof_cost_stop_and_pre_cancel_publish_no_copy_plan() {
    with_field_method(
        FIELD_JAVAC8_DEBUG,
        "FieldWriteCopies",
        "assignAndCompare",
        "(I)Z",
        |ir, operations, fields| {
            let ssa = ir.ssa().unwrap();
            let (block_id, index) = dup_x1_locations(ir, operations)
                .pop()
                .expect("positive method has one candidate");
            let block = ssa.block(&block_id).unwrap();
            let copy_bci = block.instructions()[index].bci();
            let mut full_budget = field_copy_budget(u64::MAX, u64::MAX);
            let full = field_test_copy_at(
                ssa,
                operations,
                fields,
                true,
                block,
                index,
                &mut full_budget,
            )
            .expect("unlimited candidate proof completes")
            .expect("candidate proof produces its exact FieldCopy");
            assert_eq!(full.duplicate, copy_bci);
            let measured = full_budget.usage().analysis_steps;
            assert!(measured > 0);
            let mut limited_budget = field_copy_budget(measured - 1, u64::MAX);
            assert!(
                matches!(
                    field_test_copy_at(
                        ssa,
                        operations,
                        fields,
                        true,
                        block,
                        index,
                        &mut limited_budget
                    ),
                    Err(StopReason::Budget {
                        dimension: CountedBudgetDimension::AnalysisSteps,
                        ..
                    })
                ),
                "measured proof cost minus one stops inside the actual helper"
            );

            let mut full_plan_budget = field_copy_budget(u64::MAX, u64::MAX);
            let full_plan = FieldCopies::prove(
                ssa,
                ir.canonical().unwrap(),
                operations,
                fields,
                true,
                &mut full_plan_budget,
            )
            .expect("whole plan proof completes");
            assert!(full_plan.copy_at(copy_bci).is_some());
            let total_plan_steps = full_plan_budget.usage().analysis_steps;
            assert!(total_plan_steps > 0);
            let mut stopped_plan_budget = field_copy_budget(total_plan_steps - 1, u64::MAX);
            let stopped_plan = FieldCopies::prove(
                ssa,
                ir.canonical().unwrap(),
                operations,
                fields,
                true,
                &mut stopped_plan_budget,
            );
            assert!(
                matches!(
                    stopped_plan,
                    Err(StopReason::Budget {
                        dimension: CountedBudgetDimension::AnalysisSteps,
                        ..
                    })
                ),
                "a stopped plan returns Err, so its locally accumulated copies cannot be published"
            );

            let token = CancellationToken::new();
            let mut cancelled_budget = Budget::with_cancellation_token(
                field_copy_limits(u64::MAX, u64::MAX),
                token.clone(),
            );
            token.cancel();
            assert!(matches!(
                field_test_copy_at(
                    ssa,
                    operations,
                    fields,
                    true,
                    block,
                    index,
                    &mut cancelled_budget
                ),
                Err(StopReason::Cancelled { .. })
            ));

            let mut deadline_budget = field_copy_budget(u64::MAX, 1);
            std::thread::sleep(std::time::Duration::from_millis(10));
            assert!(
                matches!(
                    field_test_copy_at(
                        ssa,
                        operations,
                        fields,
                        true,
                        block,
                        index,
                        &mut deadline_budget
                    ),
                    Err(StopReason::Interrupted {
                        code: crate::stop::BUDGET_INTERRUPTED_CODE,
                        ..
                    })
                ),
                "an expired elapsed-time limit remains a Stop from the exact proof"
            );
        },
    );
}

// Private-only delta sketch for crates/jarde-java/src/build/tests/field_test_copy_tests.rs.
// Root must copy the generated class into the fixture path before applying this include_bytes.
const EXTRA_STACK_COPY_NEIGHBOR: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-field-write-test-copies/extra-stack-copy/FieldWriteCopies.class"
);

#[test]
fn extra_stack_copy_after_store_is_rejected_by_the_actual_reader_helper_and_plan() {
    with_field_method(
        EXTRA_STACK_COPY_NEIGHBOR,
        "FieldWriteCopies",
        "assignAndCompare",
        "(I)Z",
        |ir, operations, fields| {
            let ssa = ir
                .ssa()
                .expect("Reader SSA for the complete valid-neighbor class");
            let candidates = dup_x1_locations(ir, operations);
            assert_eq!(candidates.len(), 1);
            let (block_id, index) = &candidates[0];
            let block = ssa
                .block(block_id)
                .expect("candidate belongs to this Reader table");
            let copy_bci = block.instructions()[*index].bci();
            assert_eq!(copy_bci, 6);
            let changed = ir
                .code()
                .unwrap()
                .instructions
                .iter()
                .find(|i| i.bci == 10)
                .unwrap();
            assert_eq!(
                changed.opcode, 0x59,
                "the one-byte neighbor is dup at BCI 10"
            );
            assert!(matches!(operations.get(10), Some(Operation::Duplicate)));

            let mut helper_budget = field_copy_budget(u64::MAX, u64::MAX);
            assert!(
                field_test_copy_at(
                    ssa,
                    operations,
                    fields,
                    true,
                    block,
                    *index,
                    &mut helper_budget
                )
                .expect("strict candidate mismatch is a local refusal")
                .is_none(),
                "the post-store dup is not the required integer constant; reject before publishing a FieldCopy"
            );

            let mut plan_budget = field_copy_budget(u64::MAX, u64::MAX);
            let plan = FieldCopies::prove(
                ssa,
                ir.canonical().expect("same Reader canonical graph"),
                operations,
                fields,
                true,
                &mut plan_budget,
            )
            .expect("whole method field-copy analysis completes");
            assert!(plan.copy_at(copy_bci).is_none());
        },
    );
}

#[test]
fn missing_field_evidence_context_refuses_an_otherwise_real_positive_reader_candidate() {
    with_field_method(
        FIELD_JAVAC8_DEBUG,
        "FieldWriteCopies",
        "assignAndCompare",
        "(I)Z",
        |ir, operations, _actual_fields| {
            let ssa = ir.ssa().expect("Reader SSA");
            let (block_id, index) = dup_x1_locations(ir, operations)
                .pop()
                .expect("actual positive method contains the candidate");
            let block = ssa.block(&block_id).expect("same SSA table owns the block");
            let missing_field_evidence = field::Plan::empty();
            let mut budget = field_copy_budget(u64::MAX, u64::MAX);
            assert!(
                field_test_copy_at(
                    ssa,
                    operations,
                    &missing_field_evidence,
                    true,
                    block,
                    index,
                    &mut budget,
                )
                .expect("missing evidence is a local refusal")
                .is_none(),
                "same Reader code and SSA with a deliberately empty field-plan context is not admissible"
            );
        },
    );
}
