use jarde_query::query::{
    ConsumerKind, ConsumerSchema, QueryRelation, QueryRequest, QueryTarget, execute,
};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::facts_cache::{FactsCache, FactsCapacity};
use jarde_reader::model::{JvmBytes, SymbolRef};
use jarde_reader::view::{PhysicalScope, PhysicalView};

const OWNER_REFS: &[u8] = include_bytes!("fixtures/OwnerRefs.class");

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

#[test]
fn query_results_are_identical_when_all_three_producers_reuse_reader_facts() {
    let snapshot =
        ArtifactSnapshot::open(ArtifactInput::bytes(OWNER_REFS), &mut Budget::new(limits()))
            .expect("fixture opens");
    let request = QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol {
            value: SymbolRef::Class {
                owner: JvmBytes(b"OwnerRefs".to_vec()),
            },
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(
            1,
            [
                ConsumerKind::Invocation,
                ConsumerKind::Field,
                ConsumerKind::Type,
                ConsumerKind::Constant,
                ConsumerKind::Bootstrap,
            ],
        ),
        max_items: 0,
        cursor: None,
    };
    let uncached =
        execute(&snapshot, &request, &mut Budget::new(limits())).expect("uncached query completes");
    let cache = FactsCache::current(FactsCapacity::new(32, 1 << 20));
    let mut first_budget = Budget::new(limits()).with_facts_cache(cache.clone());
    let first = execute(&snapshot, &request, &mut first_budget).expect("initial cache query");
    let mut second_budget = Budget::new(limits()).with_facts_cache(cache.clone());
    let second = execute(&snapshot, &request, &mut second_budget).expect("warm cache query");

    for cached in [&first, &second] {
        assert_eq!(cached.physical, uncached.physical);
        assert_eq!(cached.relation, uncached.relation);
        assert_eq!(cached.consumers, uncached.consumers);
        assert_eq!(cached.analysis, uncached.analysis);
        assert_eq!(cached.items, uncached.items);
        assert_eq!(cached.page, uncached.page);
        assert_eq!(cached.coverage, uncached.coverage);
        assert_eq!(cached.diagnostics, uncached.diagnostics);
        assert!(matches!(
            cached.execution,
            jarde_reader::model::ExecutionReport::Complete { .. }
        ));
    }
    let facts = cache.report();
    assert_eq!(facts.consultations, 6);
    assert_eq!(facts.misses, 1);
    assert_eq!(facts.hits, 5);
    assert_eq!(facts.consultations, facts.hits + facts.misses);
}
