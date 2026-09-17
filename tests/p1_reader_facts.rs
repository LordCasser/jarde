//! Public-entry regression evidence for the crate-private reader fact layer.
//!
//! `classfile::class_facts`, `attribute_content`, `attribute_facts` and
//! `bootstrap_methods` are `pub(crate)`, so an integration test cannot call them;
//! their behaviour is covered by the unit tests next to them in
//! `src/classfile.rs` (module `reader_facts_tests`). What this file pins from the
//! outside is the contract the fact layer must not disturb: the public header and
//! bytecode paths keep the same results and the same byte charges on a real
//! classfile, and the constant-pool index an instruction reports still names the
//! same symbol.

use jarde::{
    Budget, CancellationToken, Error, ExecutionReport, InspectionMode, JvmBytes, Limits,
    MethodSelector, inspect_header, inspect_method_bytecode,
};

const FIXTURE: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

fn unlimited() -> Limits {
    Limits {
        input_bytes: u64::MAX,
        archive_entries: u64::MAX,
        entry_bytes: u64::MAX,
        read_bytes: u64::MAX,
        class_bytes: u64::MAX,
        attribute_bytes: u64::MAX,
        code_bytes: u64::MAX,
        result_items: u64::MAX,
        output_bytes: u64::MAX,
        nested_depth: u64::MAX,
        elapsed_millis: u64::MAX,
    }
}

#[test]
fn public_header_path_keeps_its_result_and_charge_shape() {
    let mut budget = Budget::new(unlimited());
    let inspection = inspect_header(FIXTURE, &mut budget, InspectionMode::Forensic).unwrap();

    assert_eq!(inspection.header.major_version, 52);
    assert_eq!(inspection.header.minor_version, 0);
    assert_eq!(
        inspection.header.this_class.raw().0,
        b"HistoricalControlFlow"
    );
    assert_eq!(
        inspection.header.super_class.as_ref().unwrap().raw().0,
        b"java/lang/Object"
    );
    assert!(inspection.header.interfaces.is_empty());
    assert!(inspection.header.fields.is_empty());
    assert_eq!(inspection.header.methods.len(), 3);
    assert!(inspection.header.attributes.is_empty());
    assert_eq!(inspection.header.methods[2].name.raw().0, b"finallyPath");
    assert_eq!(inspection.header.methods[2].attributes.len(), 1);
    assert_eq!(
        inspection.header.methods[2].attributes[0].name.raw().0,
        b"Code"
    );

    // One class read, three `Code` shells of 23 + 22 + 53 bytes, and one result
    // item per returned structure: three shells plus three methods plus the
    // class-level attribute list.
    let usage = budget.usage();
    assert_eq!(usage.class_bytes, FIXTURE.len() as u64);
    assert_eq!(usage.attribute_bytes, 23 + 22 + 53);
    assert_eq!(usage.result_items, 7);
}

#[test]
fn public_bytecode_path_keeps_its_result_and_charge_shape() {
    let selector = MethodSelector {
        name: JvmBytes(b"<init>".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    };
    let mut budget = Budget::new(unlimited());
    let report = inspect_method_bytecode(FIXTURE, selector, &mut budget).unwrap();

    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert!(report.exception_handlers.is_empty());
    assert_eq!(
        report
            .instructions
            .iter()
            .map(|instruction| (instruction.bci, instruction.opcode, instruction.width))
            .collect::<Vec<_>>(),
        vec![(0, 0x2a, 1), (1, 0xb7, 3), (4, 0xb1, 1)]
    );

    // The instruction reports constant-pool index 8, and the operand bytes its
    // span covers really encode that index: the reader fact layer resolves the
    // same index for the same input.
    let invoke = &report.instructions[1];
    assert_eq!(invoke.constant_pool_index, Some(8));
    let operand = &FIXTURE[invoke.span.start as usize + 1..invoke.span.start as usize + 3];
    assert_eq!(operand, &8u16.to_be_bytes()[..]);

    // One class read, one `Code` shell of 23 bytes, the returned report item plus
    // three instructions.
    let usage = budget.usage();
    assert_eq!(usage.class_bytes, FIXTURE.len() as u64);
    assert_eq!(usage.attribute_bytes, 23);
    assert_eq!(usage.code_bytes, 5);
    assert_eq!(usage.result_items, 4);
}

#[test]
fn public_entry_points_report_budget_hits_and_cancellation_as_errors() {
    let mut configured = unlimited();
    configured.class_bytes = FIXTURE.len() as u64 - 1;
    let mut budget = Budget::new(configured);
    assert!(matches!(
        inspect_header(FIXTURE, &mut budget, InspectionMode::Forensic).unwrap_err(),
        Error::BudgetExceeded { .. }
    ));
    assert_eq!(budget.usage().class_bytes, 0);

    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(unlimited(), token);
    let selector = MethodSelector {
        name: JvmBytes(b"add".to_vec()),
        descriptor: JvmBytes(b"(II)I".to_vec()),
    };
    assert!(matches!(
        inspect_method_bytecode(FIXTURE, selector, &mut budget).unwrap_err(),
        Error::Cancelled { .. }
    ));
    assert_eq!(budget.usage().class_bytes, 0);
}
