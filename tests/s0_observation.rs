//! Lifetime contract for S0's process counters; this target owns one isolated process.
#![cfg(feature = "test-support")]

use jarde_java::ast::{Expr, ExprKind, Stmt, StmtKind};
use jarde_java::source_map::{Origin, OriginSet};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::class_facts;
use jarde_reader::facts_cache::{FactsCache, FactsCapacity};
use std::sync::{Arc, Barrier};

#[test]
fn shared_facts_and_cloned_ast_have_distinct_balanced_lifetimes() {
    let bytes = include_bytes!("fixtures/p3-scope/v8/Scope.class");
    let limits = Limits {
        input_bytes: 1 << 20,
        archive_entries: 1024,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 4096,
        output_bytes: 1 << 20,
        class_headers: 64,
        method_bodies: 64,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 4,
        dependency_depth: 8,
        elapsed_millis: 60_000,
    };
    let cache = FactsCache::current(FactsCapacity::new(8, 1_000_000));
    let mut budget = Budget::new(limits.clone()).with_facts_cache(cache.clone());
    let before = jarde_reader::observation::snapshot();
    let facts = class_facts(bytes, &mut budget).unwrap();
    let cached = cache.structure_shared(bytes, &mut budget).unwrap().unwrap();
    let built = jarde_reader::observation::snapshot();
    let shared = Arc::clone(&cached);
    assert_eq!(jarde_reader::observation::snapshot(), built);
    let copy = (*shared).clone();
    assert_eq!(
        jarde_reader::observation::snapshot().class_facts_live,
        built.class_facts_live + 1
    );
    assert_eq!(copy, *shared);
    drop(copy);
    assert_eq!(
        jarde_reader::observation::snapshot().class_facts_live,
        built.class_facts_live
    );
    drop(cached);
    drop(shared);
    cache.clear();
    drop(facts);
    assert_eq!(
        jarde_reader::observation::snapshot().class_facts_live,
        before.class_facts_live
    );

    budget = Budget::new(limits.clone());
    // A real archive keeps one container payload while its independently cloned handles live.
    let baseline = jarde_reader::observation::snapshot();
    let path = std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/raw-receiver-field-selection/RawParam.jar"
    ));
    let length = std::fs::metadata(&path).unwrap().len();
    let snapshot = jarde_reader::artifact::ArtifactSnapshot::open(
        jarde_reader::artifact::ArtifactInput::Path(path),
        &mut budget,
    )
    .unwrap();
    let opened = jarde_reader::observation::snapshot();
    assert_eq!(opened.file_read_bytes - baseline.file_read_bytes, length);
    assert!(opened.file_read_attempts > baseline.file_read_attempts);
    let mut cursor = snapshot
        .scope_cursor(&jarde_reader::view::PhysicalScope::SnapshotAll)
        .unwrap();
    let class = cursor.next_class(&mut budget).unwrap().unwrap();
    let read = snapshot
        .prepared_class(class.entry.as_ref().unwrap(), &mut budget)
        .unwrap();
    let prepared = jarde_reader::prepared::PreparedClass::prepare(&read, &mut budget).unwrap();
    let handle = cursor.container_facts().unwrap();
    let held = jarde_reader::observation::snapshot();
    let clone = handle.clone();
    let cloned = jarde_reader::observation::snapshot();
    assert_eq!(cloned.container_facts_live, held.container_facts_live);
    assert_eq!(
        cloned.container_handles_live,
        held.container_handles_live + 1
    );
    assert_eq!(
        held.prepared_handles_live,
        baseline.prepared_handles_live + 1
    );
    assert!(held.read_attempts > baseline.read_attempts);
    drop(clone);
    drop(handle);
    drop(prepared);
    drop(read);
    drop(cursor);
    drop(snapshot);
    let released = jarde_reader::observation::snapshot();
    assert_eq!(released.container_facts_live, baseline.container_facts_live);
    assert_eq!(
        released.container_handles_live,
        baseline.container_handles_live
    );
    assert_eq!(
        released.prepared_handles_live,
        baseline.prepared_handles_live
    );

    let baseline = jarde_java::observation::snapshot();
    jarde_java::observation::reset_peaks();
    let expression = Expr::direct(ExprKind::Integer(7), 0);
    let enter = Arc::new(Barrier::new(5));
    let exit = Arc::new(Barrier::new(5));
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let (enter, exit, expression) = (&enter, &exit, &expression);
            scope.spawn(move || {
                let clone = expression.clone();
                assert_eq!(&clone, expression);
                enter.wait();
                exit.wait();
                drop(clone);
            });
        }
        enter.wait();
        let peak = jarde_java::observation::snapshot();
        assert_eq!(peak.expr_live, baseline.expr_live + 5);
        assert_eq!(peak.expr_peak, baseline.expr_live + 5);
        exit.wait();
    });
    drop(expression);
    let statement = Stmt::new(
        StmtKind::Return { value: None },
        OriginSet::new(Origin::direct(0)),
    );
    let clone = statement.clone();
    assert_eq!(statement, clone);
    assert_eq!(
        jarde_java::observation::snapshot().stmt_live,
        baseline.stmt_live + 2
    );
    drop((statement, clone));
    assert_eq!(
        jarde_java::observation::snapshot().expr_live,
        baseline.expr_live
    );
    assert_eq!(
        jarde_java::observation::snapshot().stmt_live,
        baseline.stmt_live
    );
}
