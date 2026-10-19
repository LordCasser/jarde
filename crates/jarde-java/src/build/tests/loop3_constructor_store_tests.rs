use super::*;
use jarde_reader::budget::CancellationToken;

const LOOP3_DEBUG: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/recover-proved-prefixed-catch-retry-loops/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop3$TestCls.class"
);

fn loop3_presented_constructor_store() -> (
    IteratorTestContext,
    crate::init::Sites,
    ValueId,
    CanonicalBlockId,
    u32,
) {
    let context = iterator_test_context(LOOP3_DEBUG, "writeMore", "(Ljava/lang/String;)V", 8);
    let ir = context.analysis.ir();
    let ssa = ir.ssa().expect("real Loop3 method SSA");
    let code = ir.code().expect("real Loop3 bytecode facts");
    let operations = Operations::of(code, ir.constant_pool());
    let facts = crate::facts::MethodFacts::new(
        "writeMore",
        "(Ljava/lang/String;)V",
        ir.declaration()
            .expect("real Loop3 declaration")
            .parameter_slots(),
    )
    .with_access_flags(
        ir.declaration()
            .expect("real Loop3 declaration")
            .access_flags(),
    );
    let sites = crate::init::sites(
        ssa,
        &operations,
        &crate::concat::Plan::empty(),
        &BTreeSet::new(),
        &crate::field::Plan::empty(),
        &ArrayInitializers::default(),
        &[],
        &facts,
        code,
    );
    let (value, block, consumer) = ssa
        .values_with_ids()
        .find_map(|(value, facts)| {
            let Definition::Instruction { block, .. } = facts.def() else {
                return None;
            };
            let [usage] = facts.uses() else {
                return None;
            };
            let consumer = usage.bci()?;
            (matches!(operations.get(consumer), Some(Operation::Store { .. }))
                && sites.site_producing(ssa, value).is_some())
            .then(|| (value, block.clone(), consumer))
        })
        .expect("the real entry prefix stores the proved File(String) construction");
    let site = sites
        .site_producing(ssa, value)
        .expect("the stored value has one exact construction site");
    assert_eq!(site.class, "java/io/File");
    let paths = region_paths(&context.regions);
    let path = paths
        .paths
        .get(&block)
        .expect("constructor/store block has a recovered region path");
    assert!(
        !paths
            .tries
            .iter()
            .any(|try_path| { path.starts_with(try_path) && path.get(try_path.len()) == Some(&0) }),
        "the File(String) construction is presented before the protected retry"
    );
    (context, sites, value, block, consumer)
}

#[test]
fn loop3_exact_constructor_site_before_retry_can_feed_presented_store() {
    let (context, sites, value, block, consumer) = loop3_presented_constructor_store();
    let ir = context.analysis.ir();
    let ssa = ir.ssa().expect("real Loop3 method SSA");
    let code = ir.code().expect("real Loop3 bytecode facts");
    let operations = Operations::of(code, ir.constant_pool());
    let paths = region_paths(&context.regions);
    let mut budget = child_array_test_budget();
    assert!(
        presented_reference_store_value(
            ssa,
            &operations,
            &sites,
            &paths,
            value,
            &block,
            consumer,
            &mut budget,
        )
        .expect("the exact constructor proof completes")
    );

    let mut absent_site_budget = child_array_test_budget();
    assert!(
        !presented_reference_store_value(
            ssa,
            &operations,
            &crate::init::Sites::empty(),
            &paths,
            value,
            &block,
            consumer,
            &mut absent_site_budget,
        )
        .expect("an absent exact site refuses without stopping")
    );

    let mut nonadjacent_budget = child_array_test_budget();
    assert!(
        !presented_reference_store_value(
            ssa,
            &operations,
            &sites,
            &paths,
            value,
            &block,
            consumer.saturating_add(1),
            &mut nonadjacent_budget,
        )
        .expect("an enlarged constructor-to-consumer run refuses")
    );
}

#[test]
fn loop3_constructor_store_proof_propagates_budget_and_precancellation() {
    let (context, sites, value, block, consumer) = loop3_presented_constructor_store();
    let ir = context.analysis.ir();
    let ssa = ir.ssa().expect("real Loop3 method SSA");
    let code = ir.code().expect("real Loop3 bytecode facts");
    let operations = Operations::of(code, ir.constant_pool());
    let paths = region_paths(&context.regions);

    let mut limits = child_array_test_budget().limits().clone();
    limits.analysis_steps = 0;
    let mut stopped = Budget::new(limits);
    assert!(matches!(
        presented_reference_store_value(
            ssa,
            &operations,
            &sites,
            &paths,
            value,
            &block,
            consumer,
            &mut stopped,
        ),
        Err(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled =
        Budget::with_cancellation_token(jarde_reader::budget::Limits::default(), token);
    assert!(matches!(
        presented_reference_store_value(
            ssa,
            &operations,
            &sites,
            &paths,
            value,
            &block,
            consumer,
            &mut cancelled,
        ),
        Err(StopReason::Cancelled { .. })
    ));
}
