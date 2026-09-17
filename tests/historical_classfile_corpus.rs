use jarde::{
    Budget, ExecutionReport, HeaderStructuralRead, InspectionMode, JvmBytes, Limits,
    MethodSelector, VerificationStatus, inspect_header, inspect_method_bytecode,
};

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

fn selector() -> MethodSelector {
    MethodSelector {
        name: JvmBytes(b"finallyPath".to_vec()),
        descriptor: JvmBytes(b"(I)I".to_vec()),
    }
}

fn inspect(bytes: &[u8]) -> jarde::BytecodeInspection {
    inspect_method_bytecode(bytes, selector(), &mut Budget::new(unlimited())).unwrap()
}

const OLD_FINALLY_PATH: &[(u32, u8, u32)] = &[
    (0, 0x1b, 1),
    (1, 0x04, 1),
    (2, 0x60, 1),
    (3, 0x36, 2),
    (5, 0xa8, 3),
    (8, 0x15, 2),
    (10, 0xac, 1),
    (11, 0x4e, 1),
    (12, 0xa8, 3),
    (15, 0x2d, 1),
    (16, 0xbf, 1),
    (17, 0x4d, 1),
    (18, 0x84, 3),
    (21, 0xa9, 2),
];

const MODERN_FINALLY_PATH: &[(u32, u8, u32)] = &[
    (0, 0x1b, 1),
    (1, 0x04, 1),
    (2, 0x60, 1),
    (3, 0x3e, 1),
    (4, 0x84, 3),
    (7, 0x1d, 1),
    (8, 0xac, 1),
    (9, 0x4d, 1),
    (10, 0x84, 3),
    (13, 0x2c, 1),
    (14, 0xbf, 1),
];

fn fixture(version: u16) -> &'static [u8] {
    match version {
        45 => include_bytes!("fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class"),
        46 => include_bytes!("fixtures/historical/ecj-4.6.1/v46/HistoricalControlFlow.class"),
        47 => include_bytes!("fixtures/historical/ecj-4.6.1/v47/HistoricalControlFlow.class"),
        48 => include_bytes!("fixtures/historical/ecj-4.6.1/v48/HistoricalControlFlow.class"),
        49 => include_bytes!("fixtures/historical/ecj-4.6.1/v49/HistoricalControlFlow.class"),
        50 => include_bytes!("fixtures/historical/ecj-4.6.1/v50/HistoricalControlFlow.class"),
        51 => include_bytes!("fixtures/historical/ecj-4.6.1/v51/HistoricalControlFlow.class"),
        52 => include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
        _ => panic!("unsupported fixture version {version}"),
    }
}

#[test]
fn historical_ecj_headers_and_finally_bytecode_are_stable() {
    for version in 45..=52 {
        let bytes = fixture(version);
        let header =
            inspect_header(bytes, &mut Budget::new(unlimited()), InspectionMode::Strict).unwrap();
        assert_eq!(header.header.major_version, version);
        assert_eq!(
            header.header.minor_version,
            if version == 45 { 3 } else { 0 }
        );
        assert_eq!(header.header.this_class.raw().0, b"HistoricalControlFlow");
        assert_eq!(header.structural_read, HeaderStructuralRead::Complete);
        assert_eq!(header.verification, VerificationStatus::NotPerformed);
        assert!(header.diagnostics.is_empty());
        assert!(
            header.header.methods.iter().any(
                |method| method.name.raw().0 == b"add" && method.descriptor.raw().0 == b"(II)I"
            )
        );
        assert!(header.header.methods.iter().any(|method| {
            method.name.raw().0 == b"finallyPath" && method.descriptor.raw().0 == b"(I)I"
        }));

        let report = inspect(bytes);
        let actual: Vec<_> = report
            .instructions
            .iter()
            .map(|instruction| (instruction.bci, instruction.opcode, instruction.width))
            .collect();
        let expected = if version <= 48 {
            OLD_FINALLY_PATH
        } else {
            MODERN_FINALLY_PATH
        };
        assert_eq!(actual, expected, "classfile major {version}");
        assert_eq!(
            report.code_span.length,
            expected.iter().map(|(_, _, width)| u64::from(*width)).sum()
        );
        assert_eq!(
            report
                .instructions
                .last()
                .map(|instruction| { instruction.bci + instruction.width }),
            Some(report.code_span.length as u32)
        );
        assert!(
            report
                .instructions
                .windows(2)
                .all(|pair| pair[0].bci + pair[0].width == pair[1].bci)
        );
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert_eq!(report.verification, VerificationStatus::NotPerformed);
        assert!(report.diagnostics.is_empty());

        if version <= 48 {
            assert_eq!(
                report
                    .instructions
                    .iter()
                    .filter(|instruction| instruction.opcode == 0xa8)
                    .map(|instruction| instruction.bci)
                    .collect::<Vec<_>>(),
                vec![5, 12]
            );
            assert_eq!(
                report
                    .instructions
                    .iter()
                    .filter(|instruction| instruction.opcode == 0xa9)
                    .map(|instruction| instruction.bci)
                    .collect::<Vec<_>>(),
                vec![21]
            );
        } else {
            assert!(
                !report.instructions.iter().any(|instruction| {
                    instruction.opcode == 0xa8 || instruction.opcode == 0xa9
                })
            );
        }
    }
}
