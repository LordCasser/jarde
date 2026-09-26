use jarde_query::query::{
    ConsumerKind, ConsumerSchema, QueryRelation, QueryRequest, QueryTarget, XrefTarget, execute,
};
use jarde_query::xref::{CandidateFilter, scan_candidates};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::model::{CoverageState, JvmBytes, SymbolRef};
use jarde_reader::view::{PhysicalScope, PhysicalView};

// `javac --release 8 -g -d fixtures fixtures/OwnerRefs.java` reproduces the frozen class bytes.
const OWNER_REFS: &[u8] = include_bytes!("fixtures/OwnerRefs.class");
const OWNER: &[u8] = b"OwnerRefs";

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 10_000,
        output_bytes: 1 << 20,
        nested_depth: 8,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

fn snapshot() -> ArtifactSnapshot {
    ArtifactSnapshot::open(ArtifactInput::bytes(OWNER_REFS), &mut Budget::new(limits()))
        .expect("fixture opens")
}

fn consumers() -> ConsumerSchema {
    ConsumerSchema::new(
        1,
        [
            ConsumerKind::Invocation,
            ConsumerKind::Field,
            ConsumerKind::Type,
            ConsumerKind::Constant,
            ConsumerKind::Bootstrap,
        ],
    )
}

fn owner_symbol() -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Class {
            owner: JvmBytes(OWNER.to_vec()),
        },
    }
}

fn symbols(items: &[jarde_query::query::XrefItem]) -> impl Iterator<Item = &SymbolRef> {
    items.iter().filter_map(|item| match &item.target {
        XrefTarget::Symbol { value } => Some(value),
        XrefTarget::Literal { .. } => None,
    })
}

#[test]
fn owner_filter_finds_class_members_and_bootstrap_handles_with_coordinates() {
    let snapshot = snapshot();
    let mut budget = Budget::new(limits());
    let found = scan_candidates(
        &snapshot,
        &PhysicalScope::SnapshotAll,
        &consumers(),
        CandidateFilter::Owner {
            owner: JvmBytes(OWNER.to_vec()),
        },
        0,
        &mut budget,
    )
    .expect("owner candidate scan completes");

    assert!(!found.has_more);
    assert_eq!(
        found.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(found.items.iter().any(|item| {
        matches!(
            item.target,
            XrefTarget::Symbol {
                value: SymbolRef::Class { .. }
            }
        ) && item.evidence.bci.is_some()
    }));
    assert!(found.items.iter().any(|item| {
        matches!(
            item.target,
            XrefTarget::Symbol {
                value: SymbolRef::Method { .. }
            }
        ) && item.evidence.bci.is_some()
            && item.consumer == Some(ConsumerKind::Invocation)
    }));
    assert!(found.items.iter().any(|item| {
        matches!(
            item.target,
            XrefTarget::Symbol {
                value: SymbolRef::Field { .. }
            }
        ) && item.evidence.bci.is_some()
            && item.consumer == Some(ConsumerKind::Field)
    }));
    assert!(found.items.iter().any(|item| {
        matches!(
            item.target,
            XrefTarget::Symbol {
                value: SymbolRef::Method { .. }
            }
        ) && item.consumer == Some(ConsumerKind::Bootstrap)
            && item.evidence.bci.is_some()
    }));
    assert!(symbols(&found.items).all(|symbol| match symbol {
        SymbolRef::Class { owner }
        | SymbolRef::Field { owner, .. }
        | SymbolRef::Method { owner, .. } => {
            owner.0 == OWNER
        }
    }));

    // P1 exact class matching sees class symbols only; the owner filter additionally finds
    // child-owned method/field uses and the bootstrap method handle.
    let request = QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: owner_symbol(),
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: consumers(),
        max_items: 0,
        cursor: None,
    };
    let exact = execute(&snapshot, &request, &mut Budget::new(limits())).expect("exact query");
    assert!(symbols(&exact.items).all(|symbol| matches!(symbol, SymbolRef::Class { .. })));
    assert!(found.items.len() > exact.items.len());
}

#[test]
fn owner_filter_keeps_item_limit_and_budget_coverage_stops() {
    let snapshot = snapshot();
    let limited = scan_candidates(
        &snapshot,
        &PhysicalScope::SnapshotAll,
        &consumers(),
        CandidateFilter::Owner {
            owner: JvmBytes(OWNER.to_vec()),
        },
        1,
        &mut Budget::new(limits()),
    )
    .expect("limited scan returns its prefix");
    assert_eq!(limited.items.len(), 1);
    assert!(limited.has_more);
    assert_eq!(
        limited.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );

    let mut zero_results = limits();
    zero_results.result_items = 0;
    let stopped = scan_candidates(
        &snapshot,
        &PhysicalScope::SnapshotAll,
        &consumers(),
        CandidateFilter::Owner {
            owner: JvmBytes(OWNER.to_vec()),
        },
        0,
        &mut Budget::new(zero_results),
    )
    .expect("budget stop is reported");
    assert!(stopped.items.is_empty());
    assert!(stopped.has_more);
    assert_eq!(
        stopped.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
}
