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
    let code = [
        0x08, 0x3c, 0x1b, 0x1b, 0x84, 1, 1, 0x04, 0x60, 0x60, 0x3c, 0x1b, 0xac,
    ];
    let class = source_class(&code);
    with_fixture_builder(&class, "method", test);
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
                Some(Operation::Load { slot: 1 })
            )
        })
        .unwrap();
    let saved = one_stack_output(load).unwrap().1;
    let producer = load.bci();
    let reader = block
        .instructions()
        .iter()
        .rev()
        .find(|instruction| {
            matches!(
                operations.get(instruction.bci()),
                Some(Operation::Store { slot: 1 })
            )
        })
        .unwrap()
        .bci();
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

fn source_class(code: &[u8]) -> Vec<u8> {
    jarde_reader::classfile::test_class::single_method_floating(
        49,
        16,
        2,
        4,
        &0u32.to_be_bytes(),
        b"(I)I",
        code,
    )
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
fn source_local_snapshot_exact_analysis_ir_limits_and_every_cancellation_keep_prior_commit() {
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
            builder.source_local_snapshot(saved, producer, reader),
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
                let result = builder.source_local_snapshot(saved, producer, reader);
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
                    builder.source_local_snapshot(saved, producer, reader),
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
fn source_local_snapshot_rejects_a_preexisting_bound_terminal_tree_without_mutation() {
    with_builder(|builder, saved, producer, reader| {
        let prefix: Vec<_> = builder
            .instructions
            .range(..producer)
            .map(|(_, instruction)| (**instruction).clone())
            .collect();
        for instruction in &prefix {
            builder.instruction(instruction).unwrap();
        }
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
            builder.source_local_snapshot(saved, producer, reader),
            Ok(false)
        );
        assert_eq!(compound_trial_state(builder), before);
        assert!(builder.bindings.contains_key(&saved));
    });
}

#[test]
fn stable_balanced_source_trees_have_exact_linear_endpoint_cost_and_no_allocations() {
    fn balanced(code: &mut Vec<u8>, levels: usize) {
        if levels == 0 {
            code.push(0x1b);
        } else {
            balanced(code, levels - 1);
            balanced(code, levels - 1);
            code.push(0x60);
        }
    }
    for levels in [5, 8] {
        let mut code = vec![0x08, 0x3c];
        balanced(&mut code, levels);
        code.extend([0x3c, 0x1b, 0xac]);
        let class = source_class(&code);
        with_fixture_builder(&class, "method", |builder, _, _, reader| {
            let before = compound_trial_state(builder);
            let block = &builder.ssa.blocks()[0];
            assert_eq!(builder.canonical.blocks().len(), 1);
            assert!(builder.canonical.edges().is_empty());
            let loads: Vec<_> = block
                .instructions()
                .iter()
                .filter(|instruction| {
                    instruction.bci() < reader
                        && matches!(
                            builder.operations.get(instruction.bci()),
                            Some(Operation::Load { slot: 1 })
                        )
                })
                .map(|load| {
                    (
                        one_stack_output(load).unwrap().1,
                        load.bci(),
                        2 + load.reads().len() + load.writes().len(),
                    )
                })
                .collect();
            assert_eq!(loads.len(), 1usize << levels);
            let expected_steps: u64 = loads.iter().map(|(_, _, steps)| *steps as u64).sum();
            assert_eq!(expected_steps, 4 * (1u64 << levels));
            for (saved, producer, _) in loads {
                assert_eq!(
                    builder.source_local_snapshot(saved, producer, reader),
                    Ok(false)
                );
            }
            assert_eq!(
                builder.budget.usage().ir_items,
                0,
                "stable leaves cannot allocate a prefix map or load list"
            );
            assert_eq!(
                builder.budget.usage().analysis_steps,
                expected_steps,
                "32/256 stable leaves pay exactly four endpoint checks each"
            );
            assert_eq!(compound_trial_state(builder), before);
        });
    }
}

// Same constant-pool, Code and LVT layout as the public Engine generator, with actual encoded
// rows. iterator_test_context reads those bytes and invokes the production reuse::plan path.
fn debug_source_class(read_updated: bool) -> Vec<u8> {
    fn u16b(out: &mut Vec<u8>, n: u16) {
        out.extend(n.to_be_bytes());
    }
    fn u32b(out: &mut Vec<u8>, n: u32) {
        out.extend(n.to_be_bytes());
    }
    fn utf8(out: &mut Vec<u8>, text: &str) {
        out.push(1);
        u16b(out, text.len().try_into().unwrap());
        out.extend(text.as_bytes());
    }
    let original = [
        0x08, 0x3b, 0x1a, 0x1a, 0x84, 0, 1, 0x04, 0x60, 0x60, 0x3b, 0x1a, 0xac,
    ];
    let consumed = [
        0x08, 0x3b, 0x1a, 0x1a, 0x84, 0, 1, 0x1a, 0x04, 0x60, 0x60, 0x60, 0x3b, 0x1a, 0xac,
    ];
    let (class, code, rows): (&str, &[u8], [(u16, u16); 2]) = if read_updated {
        ("OwnedSplitSourceIdentity", &consumed, [(2, 10), (13, 2)])
    } else {
        ("SplitSourceIdentity", &original, [(2, 8), (11, 2)])
    };
    let mut out = 0xcafebabe_u32.to_be_bytes().to_vec();
    for n in [0, 49, 12] {
        u16b(&mut out, n);
    }
    utf8(&mut out, class);
    out.push(7);
    u16b(&mut out, 1);
    utf8(&mut out, "java/lang/Object");
    out.push(7);
    u16b(&mut out, 3);
    for text in [
        "compute",
        "()I",
        "Code",
        "LocalVariableTable",
        "I",
        "before",
        "after",
    ] {
        utf8(&mut out, text);
    }
    for n in [0x21, 2, 4, 0, 0, 1, 0x09, 5, 6, 1, 7] {
        u16b(&mut out, n);
    }
    u32b(&mut out, 40 + code.len() as u32);
    u16b(&mut out, 16);
    u16b(&mut out, 4);
    u32b(&mut out, code.len() as u32);
    out.extend(code);
    for n in [0, 1, 8] {
        u16b(&mut out, n);
    }
    u32b(&mut out, 22);
    u16b(&mut out, 2);
    for (index, (start, length)) in rows.into_iter().enumerate() {
        for n in [start, length, 10 + index as u16, 9, 0] {
            u16b(&mut out, n);
        }
    }
    u16b(&mut out, 0);
    out
}

#[test]
fn source_local_debug_controls_have_actual_unnamed_and_split_reuse_evidence() {
    for read_updated in [false, true] {
        let class = debug_source_class(read_updated);
        let context = iterator_test_context(&class, "compute", "()I", 8);
        let ir = context.analysis.ir();
        let ssa = ir.ssa().unwrap();
        let operations = Operations::of(ir.code().unwrap(), ir.constant_pool());
        let debug = iterator_test_debug_locals(ir.code().unwrap());
        assert_eq!(
            debug.iter().map(DebugLocal::range).collect::<Vec<_>>(),
            if read_updated {
                vec![Some((2, 12)), Some((13, 15))]
            } else {
                vec![Some((2, 10)), Some((11, 13))]
            }
        );
        assert_eq!(ir.canonical().unwrap().blocks().len(), 1);
        let block = &ssa.blocks()[0];
        let update = block
            .instructions()
            .iter()
            .find(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::Increment { slot: 0, .. })
                )
            })
            .unwrap();
        let [(Slot::Local(0), updated)] = update.writes() else {
            panic!("one real iinc local write")
        };
        let final_load = block
            .instructions()
            .iter()
            .rev()
            .find(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::Load { slot: 0 })
                )
            })
            .unwrap();
        if read_updated {
            let [use_] = ssa.value(*updated).uses() else {
                panic!("the added load is iinc's only reader")
            };
            let load = block
                .instructions()
                .iter()
                .find(|instruction| Some(instruction.bci()) == use_.bci())
                .unwrap();
            assert_eq!(load.reads(), &[(Slot::Local(0), *updated)]);
            assert!(matches!(
                operations.get(load.bci()),
                Some(Operation::Load { slot: 0 })
            ));
            assert_eq!(
                context.reuse.evidence().first(),
                Some(&SlotEvidence::Split(vec![
                    Some(JvmBytes::from_unicode("before")),
                    Some(JvmBytes::from_unicode("after")),
                ]))
            );
            assert_eq!(
                context.reuse.variable_at(0, update.bci()),
                Some(LocalVariable::new(0, 0))
            );
            assert_eq!(
                context.reuse.variable_at(0, final_load.bci()),
                Some(LocalVariable::new(0, 1))
            );
            assert_eq!(context.names.text(LocalVariable::new(0, 0)), Some("before"));
            assert_eq!(context.names.text(LocalVariable::new(0, 1)), Some("after"));
        } else {
            assert!(
                ssa.value(*updated).uses().is_empty(),
                "the original iinc result has no local read owner"
            );
            assert_eq!(
                context.reuse.evidence().first(),
                Some(&SlotEvidence::Unnamed)
            );
            assert_eq!(
                context.reuse.variable_at(0, update.bci()),
                Some(LocalVariable::whole(0))
            );
            assert_eq!(
                context.reuse.variable_at(0, final_load.bci()),
                Some(LocalVariable::whole(0))
            );
            assert_eq!(context.names.text(LocalVariable::whole(0)), Some("local0"));
        }
    }
}
