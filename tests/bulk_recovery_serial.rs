//! Bulk task 3.1–3.3 through the public entry: one library operation over a whole physical scope,
//! in the physical order the scope's own traversal states.
//!
//! What this file pins:
//!
//! * the stream is `Header → (ClassPrepared → Method* → ClassEnd)* → Final`, the classes arrive in
//!   the `ScopeCursor`'s order — never sorted by name, path or finish time — and every member of a
//!   class arrives in its declaration order;
//! * every class candidate the scope holds gets a disposition (prepared with its declared members, or
//!   refused with an unknown member count), and every declared method of a prepared class gets one
//!   (produced, explanation only, not produced, refused, no body), so the counts in the summary and
//!   the stream's records agree;
//! * the class is prepared **once**: the operation charges no per-method class read, which is the
//!   difference from the single-method entries that this file also measures on the same fixture;
//! * the text and the source evidence one method gets from the bulk operation are the ones the same
//!   method gets from [`jarde::Engine::recover_method`] for the same environment — the same
//!   presentation of the same pipeline, from a prepared class instead of a second read of it.

mod bulk_support;

use bulk_support::{
    FLAT_PREFIXES, NESTED_PREFIXES, Recorded, Recorder, container_roots, environment, flat_fixture,
    nested_fixture, open, request, tree_scope,
};
use jarde::*;
use jarde_reader::scope_cursor::ScopeCursor;

/// The members this fixture declares, class by class, as Java 8 javac wrote them: the denominator
/// every count below is checked against.
const DECLARED: [(&str, u64); 4] = [
    ("Scope.class", 8),
    ("Shape.class", 3),
    ("LambdaSample.class", 3),
    ("d/Holder.class", 4),
];

/// The physical order of the fixture's class candidates, from the traversal itself.
fn cursor_order(snapshot: &ArtifactSnapshot) -> Vec<Vec<u8>> {
    let mut budget = Budget::new(bulk_support::limits());
    let mut cursor: ScopeCursor = snapshot.scope_cursor(&tree_scope()).unwrap();
    let mut order = Vec::new();
    while let Some(class) = cursor.next_class(&mut budget).unwrap() {
        order.push(match &class.entry {
            Some(entry) => entry.raw_name.0.clone(),
            None => Vec::new(),
        });
    }
    order
}

/// One run of the flat fixture under one worker count and one whole-operation wall clock.
///
/// The deadline is the **operation's** (`limits.elapsed_millis`, which the ledger takes from the entry
/// budget), not a method's: it is the clock a wait inside the coordinator is bounded by.
fn run_fixture(workers: usize, elapsed_millis: u64) -> (BulkRecoveryReport, Recorder) {
    let mut limits = bulk_support::limits();
    limits.elapsed_millis = elapsed_millis;
    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(limits);
    let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment, workers);
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");
    (report, sink)
}

#[test]
fn the_whole_scope_is_recovered_class_by_class_in_physical_order() {
    let (snapshot, _opened) = open(nested_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, &NESTED_PREFIXES);
    assert_eq!(
        roots.len(),
        2,
        "the fixture really holds a nested container"
    );
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment.clone(), 1);
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");

    // The stream's shape: one header, then the classes, then the final event, and nothing after it.
    assert!(matches!(sink.events.first(), Some(Recorded::Header(_))));
    assert!(matches!(sink.events.last(), Some(Recorded::Final(_))));
    assert!(report.final_delivered);
    assert_eq!(report.summary.status(), "complete", "{:?}", report.summary);
    assert_eq!(
        sink.header().unwrap().limits,
        report.summary.limits,
        "the header publishes the configuration the report states"
    );
    assert_eq!(report.summary.limits.workers_requested, 1);
    assert_eq!(report.summary.limits.workers_effective, 1);

    // The order: exactly the traversal's, and never a sorted one.
    let expected: Vec<Vec<u8>> = cursor_order(&snapshot)
        .into_iter()
        .filter(|name| !name.is_empty())
        .collect();
    let delivered: Vec<Vec<u8>> = sink
        .prepared()
        .iter()
        .map(|prepared| prepared.location.entry().unwrap().raw_name.0.clone())
        .collect();
    assert_eq!(
        delivered, expected,
        "the classes are delivered in the scope's physical order"
    );
    assert_eq!(
        delivered,
        DECLARED
            .iter()
            .map(|(name, _)| name.as_bytes().to_vec())
            .collect::<Vec<_>>(),
        "and that order really is the fixture's own"
    );
    assert_eq!(
        sink.diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.class_ordinal.is_none())
            .count(),
        0,
        "an undamaged scope publishes no traversal diagnostic"
    );

    // Every class is prepared, with the member count its own read states, and ends once.
    assert_eq!(report.summary.classes_seen, 4);
    assert_eq!(report.summary.classes_prepared, 4);
    assert_eq!(report.summary.classes_refused, 0);
    let prepared = sink.prepared();
    assert_eq!(prepared.len(), 4);
    let declared: u64 = DECLARED.iter().map(|(_, count)| count).sum();
    for (index, (event, (_, count))) in prepared.iter().zip(DECLARED.iter()).enumerate() {
        assert_eq!(event.class_ordinal, index as u64);
        assert_eq!(
            event.declared_methods,
            *count,
            "{} declares {count} methods",
            String::from_utf8_lossy(&event.location.entry().unwrap().raw_name.0)
        );
        assert!(event.member_table_stop.is_none());
    }
    assert_eq!(report.summary.methods_declared, declared);
    assert_eq!(report.summary.methods_declared, 18);

    // Every declared method of every prepared class has exactly one record, in ordinal order, and
    // the class ends after them.
    let ends = sink.class_ends();
    assert_eq!(ends.len(), 4);
    let mut expected_delivered = 0_u64;
    for prepared in &prepared {
        let stream = sink.class_stream(prepared.class_ordinal);
        let ordinals: Vec<u64> = stream
            .iter()
            .filter_map(|event| match event {
                Recorded::Method(method) => Some(method.member_ordinal),
                _ => None,
            })
            .collect();
        assert_eq!(
            ordinals,
            (0..prepared.declared_methods).collect::<Vec<_>>(),
            "class {} publishes its members in declaration order",
            prepared.class_ordinal
        );
        assert!(matches!(stream.first(), Some(Recorded::ClassPrepared(_))));
        assert!(matches!(stream.last(), Some(Recorded::ClassEnd(_))));
        let end = ends
            .iter()
            .find(|end| end.class_ordinal == prepared.class_ordinal)
            .unwrap();
        assert_eq!(
            end.completion,
            ClassCompletion::Completed {
                methods: prepared.declared_methods
            },
            "class {} states the members it published",
            prepared.class_ordinal
        );
        assert!(
            matches!(end.execution, ExecutionReport::Complete { .. }),
            "class {} ran to its end: {:?}",
            prepared.class_ordinal,
            end.execution
        );
        expected_delivered += prepared.declared_methods;
    }
    assert_eq!(report.summary.methods_delivered, expected_delivered);
    assert_eq!(report.summary.methods_executed, expected_delivered);
    assert_eq!(report.summary.methods_not_executed, 0);
    assert_eq!(
        report.summary.outcomes.total(),
        expected_delivered,
        "every delivered record states its disposition"
    );
    assert!(
        report.summary.outcomes.produced > 0,
        "the fixture really produces artifacts: {:?}",
        report.summary.outcomes
    );

    // An interface's abstract member is a declaration without a body, stated as such rather than
    // given an empty artifact.
    let shape = prepared
        .iter()
        .find(|prepared| prepared.location.entry().unwrap().raw_name.0 == b"Shape.class".to_vec())
        .unwrap();
    let methods = sink
        .methods()
        .into_iter()
        .filter(|method| method.class_ordinal == shape.class_ordinal)
        .collect::<Vec<_>>();
    assert_eq!(
        methods[0].outcome,
        BulkMethodOutcome::NoBody,
        "`Shape.sides()` declares no Code attribute"
    );
    assert!(methods[0].text.is_none());
    assert!(methods[1].text.is_some(), "`Shape.scaled(int)` has a body");

    // One preparation per class, not one per method: the operation's own account says so, and the
    // single-method path's does not (the contrast is measured in the test below).
    assert!(
        report.usage.class_headers <= report.summary.classes_seen,
        "class headers {} must not grow with the {} methods",
        report.usage.class_headers,
        report.summary.methods_declared
    );
    assert!(
        report.usage.method_bodies <= report.summary.methods_declared,
        "no body is decoded twice: {} attempts for {} declared methods",
        report.usage.method_bodies,
        report.summary.methods_declared
    );
    assert!(
        report.usage.method_bodies
            >= report.summary.methods_declared - report.summary.outcomes.no_body,
        "every declared method with a body costs its own attempt: {:?} for {} methods",
        report.summary.outcomes,
        report.summary.methods_declared
    );
    // The one-read-per-class claim on the dimension that really carries it: the class-file bytes are
    // parsed once per class, whatever the number of methods behind them.
    let class_bytes = bulk_support::SCOPE.len()
        + bulk_support::SHAPE.len()
        + bulk_support::LAMBDA.len()
        + bulk_support::HOLDER.len();
    assert_eq!(
        report.usage.class_bytes, class_bytes as u64,
        "the four classes are parsed once each: {} bytes for {} methods",
        class_bytes, report.summary.methods_declared
    );
    assert_eq!(report.summary.methods_declared, 18);
    assert_eq!(
        report.usage.class_headers, 0,
        "a prepared class costs no `ClassHeaders`: that dimension counts the binding search's reads, \
         and the one read of each class was charged by the preparation"
    );
}

#[test]
fn the_store_the_caller_attached_is_the_store_the_operation_reads_through() {
    // One operation, many budgets: the entry budget the caller opened carries the store, and every
    // budget the operation builds for one of its parts has to read through that same handle. The
    // effective configuration publishes its capacity, and the operation's own reads really land in
    // it — a private store would leave the caller's untouched and its own counters at zero.
    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut setup = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut setup, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let capacity = FactsCapacity::new(8, 1 << 20);
    let store = FactsCache::current(capacity);
    let mut budget = Budget::new(bulk_support::limits()).with_facts_cache(store.clone());
    let request = request(environment, 2);
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");

    assert_eq!(report.summary.status(), "complete", "{:?}", report.summary);
    assert_eq!(report.summary.methods_delivered, 18);
    assert_eq!(
        report.summary.limits.facts_capacity, capacity,
        "the effective configuration publishes the store the caller attached"
    );
    assert_eq!(
        sink.header().unwrap().limits.facts_capacity,
        capacity,
        "and the header states the same configuration"
    );
    let facts = store.report();
    assert!(
        facts.container_consultations > 0 && facts.directory_parses > 0,
        "the operation's own container reads went through the caller's store: {facts:?}"
    );
    assert!(
        facts.container_stored > 0,
        "and a read that ran to its end is resident in it: {facts:?}"
    );
}

#[test]
fn a_class_that_cannot_be_prepared_is_refused_and_its_method_count_stays_unknown() {
    // The fixture plus one entry whose name says `.class` and whose bytes are not a class file: the
    // traversal really yields it, the preparation really refuses it, and the classes beside it are
    // still recovered. A refused class declares an **unknown** number of methods, so it contributes
    // nothing to the declared denominator.
    let broken = vec![0_u8; 64];
    let bytes = bulk_support::zip(&[
        (b"Scope.class", bulk_support::SCOPE, bulk_support::DEFLATE),
        (b"Broken.class", &broken, bulk_support::STORE),
        (b"Shape.class", bulk_support::SHAPE, bulk_support::DEFLATE),
        (
            b"LambdaSample.class",
            bulk_support::LAMBDA,
            bulk_support::DEFLATE,
        ),
        (b"Holder.class", bulk_support::HOLDER, bulk_support::DEFLATE),
    ]);
    let (snapshot, _opened) = open(bytes);
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment, 1);
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("a refused class is a disposition, not a failed operation");

    assert_eq!(report.summary.classes_seen, 5);
    assert_eq!(report.summary.classes_prepared, 4);
    assert_eq!(report.summary.classes_refused, 1);
    assert_eq!(
        report.summary.methods_declared, 18,
        "the refused class declares an unknown number of methods and is outside the denominator"
    );
    assert_eq!(
        report.summary.methods_delivered, 18,
        "and the classes beside it are delivered in full"
    );
    assert_eq!(report.summary.methods_not_executed, 0);
    assert_eq!(
        report.summary.status(),
        "partial",
        "a refused class leaves the operation short of complete: {:?}",
        report.summary
    );
    assert_eq!(
        sink.prepared().len(),
        4,
        "the refused class publishes no `ClassPrepared` record, because it has none"
    );

    // The refused class publishes a located diagnostic and an end that names the refusal — never a
    // count it does not have.
    let refused = sink
        .class_ends()
        .into_iter()
        .find(|end| matches!(end.completion, ClassCompletion::Refused { .. }))
        .expect("the refused class publishes its end");
    let ClassCompletion::Refused { code } = &refused.completion else {
        unreachable!()
    };
    assert!(!code.is_empty());
    let diagnostic = sink
        .diagnostics()
        .into_iter()
        .find(|diagnostic| {
            diagnostic.class_ordinal == Some(refused.class_ordinal)
                && diagnostic.diagnostic.code == *code
        })
        .expect("the refusal is located at that class");
    assert!(
        !diagnostic.diagnostic.message.is_empty(),
        "the refusal states why: {:?}",
        diagnostic.diagnostic
    );
    assert_eq!(
        refused
            .location
            .entry()
            .map(|entry| entry.raw_name.0.clone()),
        Some(b"Broken.class".to_vec()),
        "and it is the entry that could not be prepared"
    );
    // The classes after it were prepared and delivered: a refused class stops neither.
    let ordinals: Vec<u64> = sink.prepared().iter().map(|p| p.class_ordinal).collect();
    assert_eq!(
        ordinals,
        vec![0, 2, 3, 4],
        "the classes are delivered in traversal order with the refused one's slot left to its end"
    );
}

#[test]
fn a_class_over_the_preparation_ceiling_is_refused_before_it_is_read_into_memory() {
    // A preparation ceiling below every class of the fixture, and the traversal still walks the whole
    // scope: each class is refused, none is truncated, and the run states the ceiling it refused at.
    //
    // The ceiling is a ceiling on **materialization**, not only on preparation, and the account says
    // so: a class the container's own directory record already states is over the ceiling never has
    // its bytes read, so the entry bytes the operation charges stay at zero. A ceiling checked after
    // the read would still refuse every class — and would show the fixture's 1,813 bytes of class
    // bodies charged, which is the reading this case is about.
    let mut limits = bulk_support::limits();
    limits.entry_bytes = 1 << 20;
    limits.class_bytes = 1 << 20;
    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(limits.clone());
    let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment, 1).with_capacities(
        64,
        jarde::DEFAULT_MAX_RESULT_WEIGHT,
        jarde::DEFAULT_MAX_BUFFERED_RESULT_WEIGHT,
    );
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("an unservable class size is a refusal per class");
    assert_eq!(report.summary.classes_seen, 4);
    assert_eq!(report.summary.classes_prepared, 0);
    assert_eq!(report.summary.classes_refused, 4);
    assert_eq!(report.summary.methods_declared, 0);
    assert_eq!(report.summary.methods_executed, 0);
    assert!(
        report.summary.traversal_complete,
        "the traversal still reached the end of the scope: it walks, the preparation refuses"
    );
    assert_eq!(
        sink.diagnostics().len(),
        4,
        "one located refusal per class: {:?}",
        sink.diagnostics()
    );
    assert!(
        sink.diagnostics()
            .iter()
            .all(|event| event.diagnostic.code == "bulk_class_too_large"),
        "every refusal names the ceiling's own code: {:?}",
        sink.diagnostics()
    );
    assert_eq!(report.summary.status(), "partial", "{:?}", report.summary);
    assert_eq!(
        report
            .usage
            .counted_usage(CountedBudgetDimension::EntryBytes),
        0,
        "no class body was read: the ceiling refused each one at the length its own container \
         record states, before any of its bytes were materialized"
    );
    assert_eq!(
        report
            .usage
            .counted_usage(CountedBudgetDimension::ReadBytes),
        0,
        "and no compressed byte of one was read either"
    );
    assert!(
        report
            .usage
            .counted_usage(CountedBudgetDimension::ArchiveEntries)
            > 0,
        "the walk itself still parsed the directory it read the lengths from: {:?}",
        report.usage
    );
}

#[test]
fn the_serial_configuration_finishes_inside_a_deadline_and_holds_no_window_slot() {
    // The serial configuration runs each class task on the calling thread, so it opens no window slot
    // at all — and a deadline a little above the work it really does is what makes that observable. A
    // slot left behind per class would be an end nobody ever publishes, the final drain would wait for
    // it, and the wait would cross the deadline before the `final` record could be delivered: the
    // scope would be fully recovered and delivered, and the run would still end partial without a
    // `final`. The same scope under two workers is the control: there the window holds slots, at most
    // one per worker, and the same deadline is met.
    let measured = run_fixture(1, u64::MAX).0;
    let work = measured
        .usage
        .elapsed_millis
        .max(1)
        .saturating_add(200)
        .max(250);
    assert!(
        work < 500,
        "the deadline below has to fit between the fixture's own work and the drain the defect pays: \
         {work} ms"
    );

    for (workers, slots) in [(1_usize, 0_u64), (2, 2)] {
        let (report, sink) = run_fixture(workers, work);
        assert_eq!(
            report.summary.status(),
            "complete",
            "{workers} worker(s) inside {work} ms: {:?}",
            report.summary
        );
        assert!(
            report.final_delivered,
            "the run published the `final` record its consumer needs ({workers} worker(s))"
        );
        assert_eq!(report.summary.methods_declared, 18);
        assert_eq!(report.summary.methods_delivered, 18);
        assert_eq!(
            sink.methods().len(),
            18,
            "every method record reached the sink before the deadline ({workers} worker(s))"
        );
        assert!(
            report.usage.elapsed_millis < work,
            "the scope really finished inside its own deadline ({workers} worker(s)): {:?}",
            report.usage
        );
        if slots == 0 {
            assert_eq!(
                report.window.window_slots_high_water, slots,
                "the serial configuration opens no window slot, whatever the class count is: {:?}",
                report.window
            );
        } else {
            assert!(
                report.window.window_slots_high_water >= 1
                    && report.window.window_slots_high_water <= slots,
                "a worker configuration holds one slot per class task in flight, never one per \
                 class of the scope: {:?}",
                report.window
            );
        }
    }
}

#[test]
fn an_explanation_only_result_is_an_outcome_and_not_a_stop() {
    // The committed ECJ 4.6.1 sample (45.3): its `finallyPath(I)I` really holds a `jsr`/`ret`
    // subroutine, which the recovery layer refuses, so the artifact it presents quotes the bytecode
    // it could not place — an explanation with no statement in it. That is a **result**, not a stop:
    // the operation delivers it, counts it in its own bucket, and does not turn the whole scope into
    // a partial one for it.
    let bytes = bulk_support::zip(&[(
        b"HistoricalControlFlow.class",
        bulk_support::HISTORICAL,
        bulk_support::STORE,
    )]);
    let (snapshot, _opened) = open(bytes);
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment, 1);
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the sample's scope is recoverable");

    assert_eq!(report.summary.classes_seen, 1);
    assert_eq!(report.summary.classes_prepared, 1);
    assert!(
        report.summary.outcomes.explanation_only > 0,
        "the sample really presents explanations: {:?}",
        report.summary.outcomes
    );
    assert_eq!(
        report.summary.outcomes.total(),
        report.summary.methods_delivered,
        "every delivered record has exactly one disposition"
    );
    let explanations: Vec<&bulk_support::MethodRecord> = sink
        .methods()
        .iter()
        .filter(|method| method.outcome == BulkMethodOutcome::ExplanationOnly)
        .cloned()
        .collect::<Vec<_>>()
        .leak()
        .iter()
        .collect();
    for method in explanations {
        assert!(
            method.text.as_ref().is_some_and(|text| !text.is_empty()),
            "an explanation is text: {method:?}"
        );
        assert!(
            method.weight > 0 && method.weight <= report.summary.limits.max_result_weight,
            "and it is a delivered result inside the ceiling: {method:?}"
        );
    }
    // Design decision 6: a result whose content is an explanation alone, and whose run completed, is
    // delivered and does not lower the operation's aggregate. "Range complete" and "Java recovered"
    // are different statements, and this is the assertion that keeps them apart.
    assert_eq!(report.summary.status(), "complete", "{:?}", report.summary);
    assert_eq!(
        report.usage.method_bodies, 3,
        "every member with a body was decoded"
    );
}

#[test]
fn the_same_methods_read_one_at_a_time_cost_one_class_read_each() {
    let (snapshot, _opened) = open(nested_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, &NESTED_PREFIXES);
    let environment_request = environment(&snapshot, tree_scope(), roots);
    let built = environment_request.build(&content).unwrap();
    let request = request(environment_request, 1);
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");
    let bulk_headers = report.usage.class_headers;

    // The same methods, one request each, through the entry that reads its class for every method.
    let mut single = Budget::new(bulk_support::limits());
    let mut compared = 0_u64;
    for method in sink.methods() {
        let direct = Engine::new()
            .recover_method(
                &content,
                &MethodAnalysisRequest {
                    environment: built.clone(),
                    method: method.method.clone(),
                    stages: MethodOperation::Recovery.stages().to_vec(),
                },
                &mut single,
            )
            .expect("the single-method entry recovers the same body");
        if let Some(text) = method.text.as_ref() {
            assert_eq!(
                text,
                &direct.recovery().text,
                "{} `{}` presents the same text on both entries",
                String::from_utf8_lossy(&method.method.name.0),
                String::from_utf8_lossy(&method.method.descriptor.0)
            );
            compared += 1;
        }
        assert_eq!(
            method.method,
            direct.analysis().method,
            "both entries ran over the same physical identity"
        );
    }
    let single_headers = single.usage().class_headers;
    println!(
        "class header attempts: bulk {} for {} classes / {} methods; one-at-a-time {} for the same \
         {} methods ({} texts compared)",
        bulk_headers,
        report.summary.classes_seen,
        report.summary.methods_declared,
        single_headers,
        report.summary.methods_declared,
        compared
    );
    assert_eq!(compared, 17, "every declared method with a body has a text");
    assert!(
        bulk_headers <= report.summary.classes_seen,
        "one read per class: {bulk_headers} header attempts for {} classes",
        report.summary.classes_seen
    );
    assert!(
        single_headers >= report.summary.methods_declared,
        "the one-at-a-time entry really reads its class per method: {single_headers} for {} methods",
        report.summary.methods_declared
    );
    assert!(
        bulk_headers < single_headers,
        "the bulk operation is not the per-method read: {bulk_headers} against {single_headers}"
    );
}
