use super::*;

#[test]
fn non_candidates_cost_no_ir_and_candidates_require_ir() {
    const NO_ARRAY: &[u8] =
        include_bytes!("../../../../../tests/fixtures/p3-array-types/v8/ArrayTypes.class");
    const MULTI_ARRAY: &[u8] = include_bytes!(
        "../../../../../tests/fixtures/p3-deferred-value-order/v8/DeferredValueOrder.class"
    );
    const INITIALIZERS: &[u8] = include_bytes!(
        "../../../../../tests/fixtures/p3-array-initializers/v8/ArrayInitializerProbe.class"
    );
    for (class, name, descriptor, expected_opcode) in [
        (
            NO_ARRAY,
            "text",
            "(Ljava/lang/String;)Ljava/lang/String;",
            None,
        ),
        (MULTI_ARRAY, "multiArray", "(I)[[I", Some(0xc5)),
        (INITIALIZERS, "literalInts", "()[I", Some(0xbc)),
        (
            INITIALIZERS,
            "literalStrings",
            "()[Ljava/lang/String;",
            Some(0xbd),
        ),
    ] {
        let (analysis, _) = analyze_test_method(class, name, descriptor, 8);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("fixture SSA");
        let effects = ssa.effects().instructions();
        assert!(!effects.is_empty());
        if let Some(opcode) = expected_opcode {
            assert!(
                effects.iter().any(|effect| effect.opcode() == opcode),
                "{name}"
            );
        }
        let candidate = matches!(expected_opcode, Some(0xbc | 0xbd));
        assert_eq!(
            effects
                .iter()
                .any(|effect| matches!(effect.opcode(), 0xbc | 0xbd)),
            candidate
        );
        let operations = Operations::of(ir.code().expect("fixture code"), ir.constant_pool());
        let fields = field::Plan::empty();
        let mut limits = child_array_test_budget().limits().clone();
        limits.ir_items = 0;
        let mut zero = Budget::new(limits);
        let result = ArrayInitializers::prove(ssa, &operations, &fields, &mut zero);
        assert_eq!(zero.usage().ir_items, 0, "{name}");
        if candidate {
            assert!(
                matches!(
                    result,
                    Err(StopReason::Budget {
                        dimension: jarde_reader::budget::BudgetDimension::IrItems,
                        consumed: 0,
                        requested: 1,
                        limit: 0,
                        ..
                    })
                ),
                "{name}"
            );
            let mut complete = child_array_test_budget();
            let arrays =
                ArrayInitializers::prove(ssa, &operations, &fields, &mut complete).unwrap();
            assert_eq!(arrays.allocations.len(), 1, "{name}");
            assert!(complete.usage().ir_items > 0);
        } else {
            let arrays = result.expect("non-candidates require no IR budget");
            assert!(arrays.allocations.is_empty());
            assert!(arrays.aliases.is_empty());
            assert!(arrays.owned.is_empty());
            assert!(arrays.element_sources.is_empty());
            assert!(arrays.pending_sites.is_empty());
        }
    }
}
