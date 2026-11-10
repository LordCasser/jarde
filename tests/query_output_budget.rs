use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const CLASS: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

fn limits(output_bytes: u64) -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes,
        class_headers: 1_000,
        method_bodies: 1_000,
        nested_depth: 8,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

fn zip_class() -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        let (mut entry, config) = archive
            .new_file(EntryPath::verbatim(b"HistoricalControlFlow.class".to_vec()))
            .compression_method(CompressionMethod::new(0))
            .start()
            .unwrap();
        let mut writer = config.wrap(&mut entry);
        writer.write_all(CLASS).unwrap();
        let (_, descriptor) = writer.finish().unwrap();
        entry.finish(descriptor).unwrap();
        archive.finish().unwrap();
    }
    output.into_inner()
}

fn query(snapshot: &ArtifactSnapshot, budget: &mut Budget) -> QueryReport {
    let request = QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol {
            value: SymbolRef::Method {
                owner: JvmBytes(b"java/lang/Object".to_vec()),
                name: JvmBytes(b"<init>".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            },
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items: 0,
        cursor: None,
    };
    Engine::new().query(snapshot, &request, budget).unwrap()
}

#[test]
fn internal_root_and_entry_queries_do_not_charge_output_bytes() {
    let mut open_budget = Budget::new(limits(1 << 20));
    let root = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut open_budget)
        .unwrap();
    let mut query_budget = Budget::new(limits(0));
    let root_report = query(&root, &mut query_budget);
    assert!(!root_report.items.is_empty());
    assert!(matches!(
        root_report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(query_budget.usage().output_bytes, 0);
    assert_eq!(query_budget.usage().read_bytes, CLASS.len() as u64);
    assert_eq!(query_budget.usage().entry_bytes, CLASS.len() as u64);

    let mut open_budget = Budget::new(limits(1 << 20));
    let archive = Engine::new()
        .open(ArtifactInput::bytes(zip_class()), &mut open_budget)
        .unwrap();
    let mut query_budget = Budget::new(limits(0));
    let archive_report = query(&archive, &mut query_budget);
    assert!(!archive_report.items.is_empty());
    assert!(matches!(
        archive_report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(query_budget.usage().output_bytes, 0);
    assert_eq!(query_budget.usage().read_bytes, CLASS.len() as u64);
    assert_eq!(query_budget.usage().entry_bytes, CLASS.len() as u64);
}

#[test]
fn public_root_bytes_still_charges_the_returned_buffer() {
    let mut open_budget = Budget::new(limits(1 << 20));
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut open_budget)
        .unwrap();
    let mut refused_budget = Budget::new(limits(0));
    assert!(matches!(
        snapshot.root_bytes(&mut refused_budget),
        Err(Error::BudgetExceeded {
            dimension: BudgetDimension::OutputBytes,
            ..
        })
    ));
    assert_eq!(refused_budget.usage().read_bytes, 0);
    assert_eq!(refused_budget.usage().output_bytes, 0);

    let mut read_budget = Budget::new(limits(1 << 20));
    let bytes = snapshot.root_bytes(&mut read_budget).unwrap();
    assert_eq!(bytes.as_slice(), CLASS);
    assert_eq!(read_budget.usage().read_bytes, CLASS.len() as u64);
    assert_eq!(read_budget.usage().output_bytes, CLASS.len() as u64);
    assert_eq!(read_budget.usage().entry_bytes, 0);
}
