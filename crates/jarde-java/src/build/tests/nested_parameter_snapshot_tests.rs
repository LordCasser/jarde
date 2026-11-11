use super::*;
use jarde_reader::budget::{BudgetDimension, CancellationToken};
use jarde_reader::ledger::{
    LedgerEntryTiming, LedgerObserver, LedgerSite, OperationLedger, UsageOwner,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

fn with_builder(test: impl FnOnce(&mut Builder<'_>, ValueId, u32, u32)) {
    let class = include_bytes!("../../../../../tests/fixtures/p3-nested-eval/v8/NestedEval.class");
    with_fixture_builder(class, "nestedLocal", test);
}

fn with_fixture_builder(
    class: &[u8],
    method: &str,
    test: impl FnOnce(&mut Builder<'_>, ValueId, u32, u32),
) {
    let context = iterator_test_context(class, method, "(I)I", 8);
    let ir = context.analysis.ir();
    let ssa = ir.ssa().unwrap();
    let code = ir.code().unwrap();
    let operations = Operations::of(code, ir.constant_pool());
    let chains = crate::concat::Plan::empty();
    let sites = crate::init::Sites::empty();
    let fields = field::Plan::empty();
    let arrays = ArrayInitializers::default();
    let field_copies = FieldCopies::default();
    let prologues = crate::init::Prologues::none();
    let enums = crate::enumswitch::Plan::empty();
    let parameter_types = BTreeMap::from([(0, Type::Int)]);
    let mut setup = child_array_test_budget();
    let mut budget = child_array_test_budget();
    let owner = ir.declaration().unwrap().class_name().to_unicode().unwrap();
    let mut builder = compound_test_builder(
        ir.canonical().unwrap(),
        ssa,
        code,
        &operations,
        ir.constant_pool(),
        ir.bootstrap_methods(),
        ir.direct_super_class(),
        ir.direct_interfaces(),
        ir.class_methods(),
        &chains,
        &context.regions,
        &context.names,
        &context.reuse,
        &parameter_types,
        &sites,
        &fields,
        &arrays,
        &field_copies,
        &prologues,
        &enums,
        1,
        Some(Type::Int),
        Some(&owner),
        &mut setup,
        &mut budget,
    );
    let block = &ssa.blocks()[0];
    let load = block
        .instructions()
        .iter()
        .find(|instruction| {
            matches!(
                operations.get(instruction.bci()),
                Some(Operation::Load { slot: 0 })
            )
        })
        .unwrap();
    let saved = one_stack_output(load).unwrap().1;
    let producer = load.bci();
    let reader = block.instructions().last().unwrap().bci();
    let entry = block
        .entry()
        .iter()
        .find(|(slot, _)| *slot == Slot::Local(0))
        .unwrap()
        .1;
    // A genuine prior commit, independent of the proposed Load value. The proof must preserve
    // the existing declaration and binding even if its last budget charge or poll stops it.
    assert!(
        builder
            .bind_value(BindingPlan {
                value: entry,
                anchor: producer,
                producer,
                name: "earlier".to_owned(),
            })
            .unwrap()
    );
    assert!(builder.bindings.contains_key(&entry));
    assert!(
        matches!(builder.stmts.last().map(|stmt| &stmt.kind), Some(StmtKind::Declare { name, .. }) if name == "earlier")
    );
    *builder.budget = child_array_test_budget();
    test(&mut builder, saved, producer, reader);
}

#[derive(Debug)]
struct CancelAt {
    charges: AtomicUsize,
    at: Option<usize>,
    token: CancellationToken,
}
impl LedgerObserver for CancelAt {
    fn entry(&self, site: LedgerSite, _: Option<LedgerEntryTiming>) {
        if matches!(site, LedgerSite::Charge(_)) {
            let count = self.charges.fetch_add(1, Ordering::SeqCst) + 1;
            if self.at == Some(count) {
                self.token.cancel();
            }
        }
    }
    fn refusal(&self, _: LedgerSite, _: Option<BudgetDimension>) {}
}

#[test]
fn nested_snapshot_exact_analysis_ir_limits_and_every_cancellation_keep_prior_commit() {
    with_builder(|builder, saved, producer, reader| {
        let before = compound_trial_state(builder);
        let observer = Arc::new(CancelAt {
            charges: AtomicUsize::new(0),
            at: None,
            token: CancellationToken::new(),
        });
        builder.budget.with_ledger(
            OperationLedger::new(builder.budget).with_observer(observer.clone()),
            UsageOwner::Methods,
        );
        assert_eq!(
            builder.nested_parameter_snapshot(saved, producer, reader),
            Ok(true)
        );
        assert_eq!(compound_trial_state(builder), before);
        let analysis = builder.budget.usage().analysis_steps;
        let ir = builder.budget.usage().ir_items;
        let charges = observer.charges.load(Ordering::SeqCst);
        assert!(analysis > 0 && ir > 0 && charges > 2);
        let baseline = builder.budget.limits().clone();
        for dimension in [BudgetDimension::AnalysisSteps, BudgetDimension::IrItems] {
            let count = match dimension {
                BudgetDimension::AnalysisSteps => analysis,
                _ => ir,
            };
            for limit in [count - 1, count] {
                let mut limits = baseline.clone();
                match dimension {
                    BudgetDimension::AnalysisSteps => limits.analysis_steps = limit,
                    BudgetDimension::IrItems => limits.ir_items = limit,
                    _ => unreachable!(),
                }
                *builder.budget = Budget::new(limits);
                let result = builder.nested_parameter_snapshot(saved, producer, reader);
                if limit == count {
                    assert_eq!(result, Ok(true), "exact {dimension:?} boundary");
                } else {
                    assert!(
                        matches!(result, Err(StopReason::Budget { dimension: actual, limit: actual_limit, written: 0, .. })
                        if actual == dimension && actual_limit == limit),
                        "{result:?}"
                    );
                }
                let used = match dimension {
                    BudgetDimension::AnalysisSteps => builder.budget.usage().analysis_steps,
                    _ => builder.budget.usage().ir_items,
                };
                assert_eq!(used, limit);
                assert_eq!(compound_trial_state(builder), before);
            }
        }
        for cancel_at in 1..=charges {
            let token = CancellationToken::new();
            let observer = Arc::new(CancelAt {
                charges: AtomicUsize::new(0),
                at: Some(cancel_at),
                token: token.clone(),
            });
            *builder.budget = Budget::with_cancellation_token(baseline.clone(), token);
            builder.budget.with_ledger(
                OperationLedger::new(builder.budget).with_observer(observer.clone()),
                UsageOwner::Methods,
            );
            assert!(
                matches!(
                    builder.nested_parameter_snapshot(saved, producer, reader),
                    Err(StopReason::Cancelled { at: Some(_) })
                ),
                "charge {cancel_at}/{charges}"
            );
            assert_eq!(observer.charges.load(Ordering::SeqCst), cancel_at);
            assert_eq!(compound_trial_state(builder), before);
        }
    });
}

#[test]
fn nested_snapshot_rejects_a_preexisting_bound_terminal_tree_without_mutation() {
    with_builder(|builder, saved, producer, reader| {
        assert!(
            builder
                .bind_value(BindingPlan {
                    value: saved,
                    anchor: producer,
                    producer,
                    name: "alreadySaved".to_owned(),
                })
                .unwrap()
        );
        let before = compound_trial_state(builder);
        assert_eq!(
            builder.nested_parameter_snapshot(saved, producer, reader),
            Ok(false)
        );
        assert_eq!(compound_trial_state(builder), before);
        assert!(builder.bindings.contains_key(&saved));
    });
}

#[test]
fn stable_balanced_parameter_trees_allocate_no_proof_items_and_scan_only_endpoints() {
    fn balanced(code: &mut Vec<u8>, levels: usize) {
        if levels == 0 {
            code.push(0x1a);
        } else {
            balanced(code, levels - 1);
            balanced(code, levels - 1);
            code.push(0x60);
        }
    }
    for levels in [5, 8] {
        let mut code = Vec::new();
        balanced(&mut code, levels);
        code.push(0xac);
        // Reuse the reader's descriptor-capable class builder; its unused pool constant
        // has no instruction or expression in the tested method.
        let class = jarde_reader::classfile::test_class::single_method_floating(
            49,
            16,
            1,
            4,
            &0u32.to_be_bytes(),
            b"(I)I",
            &code,
        );
        with_fixture_builder(&class, "method", |builder, _, _, reader| {
            let before = compound_trial_state(builder);
            let block = &builder.ssa.blocks()[0];
            let loads: Vec<_> = block
                .instructions()
                .iter()
                .filter(|instruction| {
                    matches!(
                        builder.operations.get(instruction.bci()),
                        Some(Operation::Load { .. })
                    )
                })
                .map(|load| {
                    (
                        one_stack_output(load).unwrap().1,
                        load.bci(),
                        2 + builder.canonical.edges().len()
                            + load.reads().len()
                            + load.writes().len()
                            + block.exit().len(),
                    )
                })
                .collect();
            assert_eq!(loads.len(), 1usize << levels);
            let expected_steps: u64 = loads.iter().map(|(_, _, steps)| *steps as u64).sum();
            for (saved, producer, _) in loads {
                assert_eq!(
                    builder.nested_parameter_snapshot(saved, producer, reader),
                    Ok(false)
                );
            }
            assert_eq!(
                builder.budget.usage().ir_items,
                0,
                "stable loads never allocate a proof worklist or memo table"
            );
            assert_eq!(
                builder.budget.usage().analysis_steps,
                expected_steps,
                "only the CFG edges and load/exit endpoints are scanned, not whole trees"
            );
            assert_eq!(compound_trial_state(builder), before);
        });
    }
}
