use super::*;

const CLASS: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/s2a-reference-constraints/corretto8-no-debug.class"
);

#[test]
fn incompatible_numeric_lifetimes_refuse_during_planning() {
    let context = iterator_test_context(
        include_bytes!(
            "../../../../../tests/fixtures/s2a-reference-constraints/primitive-loop.class"
        ),
        "value",
        "(I)J",
        8,
    );
    assert!(
        context
            .declarations
            .decided
            .values()
            .any(|decision| matches!(
                decision,
                Decided::Unknown(NoType::ConflictingWrites {
                    declared: Type::Int,
                    observed: Type::Long,
                    ..
                })
            )),
        "mixed JVM kinds must be rejected before a first-write declaration is built"
    );
}

#[test]
fn reference_constraints_are_independent_of_access_order_and_stop_before_a_plan() {
    for (name, descriptor) in [
        ("nullCopyChain", "(Z)Ljava/lang/String;"),
        ("objectExchange", "(Z)Ljava/lang/Object;"),
        ("explicitCast", "(Z)Ljava/lang/String;"),
    ] {
        let context = iterator_test_context(CLASS, name, descriptor, 8);
        let ir = context.analysis.ir();
        let ssa = ir.ssa().expect("frozen input has SSA");
        let operations = Operations::of(ir.code().unwrap(), ir.constant_pool());
        let paths = region_paths(&context.regions);
        let uses = slot_uses(ssa, &operations, &context.reuse, &paths, &BTreeSet::new());
        let parameter_types = BTreeMap::from([(0, Type::Boolean)]);
        let return_type = return_type(descriptor);
        let solve = |uses: &BTreeMap<LocalVariable, Vec<SlotUse>>, budget: &mut Budget| {
            let mut decisions = context.declarations.decided.clone();
            legalize_reference_locals(
                uses,
                ssa,
                &operations,
                &context.reuse,
                1,
                false,
                &parameter_types,
                return_type.as_ref(),
                &BTreeSet::new(),
                8,
                &[],
                &mut decisions,
                budget,
            )?;
            Ok::<_, StopReason>(decisions)
        };
        let mut budget = child_array_test_budget();
        let expected = solve(&uses, &mut budget).expect("reference plan completes");
        assert!(
            expected
                .values()
                .all(|decision| matches!(decision, Decided::Type(_))),
            "{name}: {expected:?}"
        );
        let expected_reference = if name == "nullCopyChain" {
            "java.lang.String"
        } else {
            "java.lang.Object"
        };
        assert!(expected.iter().filter(|(variable, _)| variable.slot() != 0).all(|(_, decision)| {
            matches!(decision, Decided::Type(Type::Reference(ty)) if ty == expected_reference)
        }), "{name}: {expected:?}");
        let cost = budget.usage().analysis_steps;
        let mut reversed = uses;
        for accesses in reversed.values_mut() {
            accesses.reverse();
        }
        assert_eq!(
            solve(&reversed, &mut child_array_test_budget()).unwrap(),
            expected
        );

        let limits = child_array_test_budget().limits().clone();
        for limit in [0, cost / 2, cost - 1] {
            let mut limits = limits.clone();
            limits.analysis_steps = limit;
            assert!(
                matches!(
                    solve(&reversed, &mut Budget::new(limits)),
                    Err(StopReason::Budget { .. })
                ),
                "{name}: limit {limit}"
            );
        }
        let mut cancelled = child_array_test_budget();
        cancelled.cancellation_token().cancel();
        assert!(matches!(
            solve(&reversed, &mut cancelled),
            Err(StopReason::Cancelled { .. })
        ));
        let mut expired_limits = limits.clone();
        expired_limits.elapsed_millis = 0;
        assert!(matches!(
            solve(&reversed, &mut Budget::new(expired_limits)),
            Err(StopReason::Budget {
                dimension: jarde_reader::budget::BudgetDimension::ElapsedMillis,
                written: 0,
                ..
            })
        ));
    }
}
