use super::*;

const LOOP5_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-named-catch-loop-return-tails/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop5$TestCls.class"
);
const LOOP5_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-named-catch-loop-return-tails/original/no-debug/classes/jadx/tests/integration/loops/TestBreakInLoop5$TestCls.class"
);
const LOOP5_DESC: &str = "(Ljava/lang/String;)J";

fn run_decide_types(
    context: &IteratorTestContext,
    uses: &BTreeMap<LocalVariable, Vec<SlotUse>>,
    catch_parameters: &BTreeMap<RegionPath, (u16, CanonicalBlockId)>,
    budget: &mut Budget,
) -> Result<BTreeMap<LocalVariable, Decided>, StopReason> {
    let ir = context.analysis.ir();
    let declaration = ir.declaration().expect("fixture declaration");
    let ssa = ir.ssa().expect("fixture SSA");
    let code = ir.code().expect("fixture bytecode facts");
    let operations = Operations::of(code, ir.constant_pool());
    let method_facts =
        crate::facts::MethodFacts::new("test", LOOP5_DESC, declaration.parameter_slots())
            .with_access_flags(declaration.access_flags());
    let parameter_types = method_facts.parameter_types();
    let return_ty = return_type(LOOP5_DESC);
    decide_types(
        uses,
        &context.regions,
        &resource_slots(&context.regions),
        !code.exception_handlers.is_empty(),
        ssa,
        &operations,
        &context.reuse,
        method_facts.parameters(),
        &parameter_types,
        return_ty.as_ref(),
        &crate::field::Plan::empty(),
        &BTreeSet::new(),
        catch_parameters,
        budget,
    )
}

fn loop5_inputs(
    bytes: &[u8],
) -> (
    IteratorTestContext,
    Operations,
    RegionPaths,
    BTreeMap<LocalVariable, Vec<SlotUse>>,
    LocalVariable,
    RegionPath,
) {
    let context = iterator_test_context(bytes, "test", LOOP5_DESC, 23);
    let ir = context.analysis.ir();
    let ssa = ir.ssa().expect("fixture SSA");
    let code = ir.code().expect("fixture bytecode facts");
    let operations = Operations::of(code, ir.constant_pool());
    let paths = region_paths(&context.regions);
    let mut cleanup = Vec::new();
    for region in &context.regions {
        region.elided_cleanup_bcis(&mut cleanup);
    }
    let uses = slot_uses(
        ssa,
        &operations,
        &context.reuse,
        &paths,
        &cleanup.into_iter().collect(),
    );
    let variable = context
        .reuse
        .variable_at(2, 1)
        .expect("multiplier identity at its first write");
    let catch_path = paths
        .catch_parameters
        .iter()
        .find_map(|(path, (slot, handler))| {
            (*slot == 2 && handler.bci() == 124).then_some(path.clone())
        })
        .expect("exact catch path owns slot 2 at handler 124");
    (context, operations, paths, uses, variable, catch_path)
}

#[test]
fn loop5_handler_entry_phi_does_not_conflict_with_reused_double_slot() {
    for bytes in [LOOP5_DEBUG, LOOP5_NO_DEBUG] {
        let (context, _, paths, uses, variable, _) = loop5_inputs(bytes);
        let mut budget = child_array_test_budget();
        let decisions = run_decide_types(&context, &uses, &paths.catch_parameters, &mut budget)
            .expect("bounded local type decision");
        assert_eq!(
            decisions.get(&variable),
            Some(&Decided::Type(Type::Double)),
            "the catch header store is lexically separate from the ordinary double local"
        );
    }
}

#[test]
fn loop5_catch_binding_requires_exact_path_handler_slot_and_stack_zero_phi() {
    let (context, _, paths, uses, variable, catch_path) = loop5_inputs(LOOP5_DEBUG);
    let canonical = context
        .analysis
        .ir()
        .canonical()
        .expect("fixture canonical CFG");
    let handler = paths
        .catch_parameters
        .get(&catch_path)
        .expect("exact catch entry")
        .1
        .clone();

    let mut negatives = Vec::new();

    let mut wrong_path = paths.catch_parameters.clone();
    wrong_path.remove(&catch_path);
    negatives.push((uses.clone(), wrong_path));

    let mut wrong_handler = paths.catch_parameters.clone();
    let wrong_handler_id = canonical
        .blocks()
        .iter()
        .find(|block| block.id().bci() == 14)
        .expect("non-handler block exists")
        .id()
        .clone();
    wrong_handler.insert(catch_path.clone(), (2, wrong_handler_id));
    negatives.push((uses.clone(), wrong_handler));

    let mut wrong_slot = paths.catch_parameters.clone();
    wrong_slot.insert(catch_path.clone(), (3, handler.clone()));
    negatives.push((uses.clone(), wrong_slot));

    let ssa = context.analysis.ir().ssa().expect("fixture SSA");
    let ordinary_phi = ssa
        .phis()
        .iter()
        .find(|phi| phi.block() != &handler)
        .expect("fixture has a non-handler phi")
        .value();
    let mut local_phi = uses.clone();
    local_phi
        .get_mut(&variable)
        .expect("multiplier slot uses")
        .iter_mut()
        .find(|use_| use_.bci == handler.bci() && use_.written.is_some())
        .expect("catch entry store use")
        .stored = Some(ordinary_phi);
    let mut budget = child_array_test_budget();
    let with_catch_path =
        run_decide_types(&context, &local_phi, &paths.catch_parameters, &mut budget)
            .expect("bounded local type decision");
    let mut budget = child_array_test_budget();
    let without_catch_path = run_decide_types(&context, &local_phi, &BTreeMap::new(), &mut budget)
        .expect("bounded local type decision");
    assert_eq!(
        with_catch_path, without_catch_path,
        "a non-handler local phi cannot satisfy the handler-entry proof"
    );

    for (mutated_uses, catch_parameters) in negatives {
        let mut budget = child_array_test_budget();
        let decisions = run_decide_types(&context, &mutated_uses, &catch_parameters, &mut budget)
            .expect("bounded local type decision");
        assert!(
            matches!(
                decisions.get(&variable),
                Some(Decided::Unknown(NoType::ConflictingWrites {
                    later: 124,
                    ..
                }))
            ),
            "a mismatched catch fact must preserve the ordinary write conflict"
        );
    }
}

#[test]
fn loop5_catch_binding_propagates_budget_and_cancellation_stops() {
    let (context, _, paths, uses, _, _) = loop5_inputs(LOOP5_DEBUG);

    let mut limits = child_array_test_budget().limits().clone();
    limits.analysis_steps = 0;
    let mut budget = Budget::new(limits);
    assert!(matches!(
        run_decide_types(&context, &uses, &paths.catch_parameters, &mut budget),
        Err(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));

    let token = jarde_reader::budget::CancellationToken::new();
    token.cancel();
    let mut cancelled =
        Budget::with_cancellation_token(jarde_reader::budget::Limits::default(), token);
    assert!(matches!(
        run_decide_types(&context, &uses, &paths.catch_parameters, &mut cancelled),
        Err(StopReason::Cancelled { .. })
    ));
}
