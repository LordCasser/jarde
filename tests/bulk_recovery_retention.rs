//! Bulk task 3.2 through the public entry: zero retention, a capacity too small for anything, and what
//! a consumed result's release is observable as.
//!
//! Two claims are separated here on purpose, because only one of them is observable from outside:
//!
//! * **A capacity that keeps nothing changes no conclusion.** The same scope runs under
//!   [`FactsCapacity::new(0, 0)`], under a capacity too small to hold one answer, and under one with
//!   room. The three runs publish the same classes, members, order, texts, dispositions and counts, and
//!   the two refusals are *readable*: a store that retains nothing answers nothing, refuses every answer
//!   it is offered, and makes the operation re-parse what it could not keep — which is exactly the
//!   difference between the runs, and the one thing that does **not** reach the result.
//! * **What a consumed result releases is only partly observable.** The stream's own evidence is that
//!   the records arrive in the order they were produced, that the operation hands over one result at a
//!   time and never accumulates them while the consumer is slow (the window's high-water mark stays
//!   inside the window's ceiling), and that the returned report holds counts, boundaries and this run's
//!   account but **no per-method result** — none of the delivered texts appears in it. The release the
//!   analysis performs when a method ends is *not* observable from here: nothing states when one
//!   method's IR and local facts stopped being needed, and no figure here is RSS, an allocator's view or
//!   a claim about the process's real working set.

mod bulk_support;

use bulk_support::{
    NESTED_PREFIXES, Recorded, Recorder, container_roots, environment, nested_fixture, open,
    request, tree_scope,
};
use jarde::*;

/// One whole run of the nested fixture under one facts retention, and the store it read through.
///
/// The store is attached to the entry budget the operation is opened with — the one budget whose handle
/// every class task and method budget of that operation is built from — and the fixture's roots are
/// declared on a separate budget, so the counters this returns are the operation's own reads.
fn run_with(capacity: FactsCapacity, workers: usize) -> (BulkRecoveryReport, Recorder, FactsCache) {
    let (snapshot, _opened) = open(nested_fixture());
    let content = vec![snapshot.clone()];
    let mut setup = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut setup, &NESTED_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment, workers);
    let store = FactsCache::current(capacity);
    let mut budget = Budget::new(bulk_support::limits()).with_facts_cache(store.clone());
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");
    (report, sink, store)
}

/// One delivered method record as two runs of one scope must agree on it: where it sits, its identity,
/// its disposition, its text and the retention weight it asked the window for.
type Delivered = (
    (u64, u64, PhysicalMethodId),
    BulkMethodOutcome,
    Option<String>,
    u64,
);

/// What one run's stream says, in the order it delivered it.
fn delivered(sink: &Recorder) -> Vec<Delivered> {
    sink.methods()
        .iter()
        .map(|method| {
            (
                method.key(),
                method.outcome,
                method.text.clone(),
                method.weight,
            )
        })
        .collect()
}

/// What one run's summary counted, without the two things a run may differ in: its resource readings and
/// the retention it was opened with.
fn counted(summary: &BulkSummary) -> (u64, u64, u64, u64, u64, u64, BulkOutcomeCounts, bool) {
    (
        summary.classes_seen,
        summary.classes_prepared,
        summary.classes_refused,
        summary.methods_declared,
        summary.methods_executed,
        summary.methods_delivered,
        summary.outcomes,
        summary.traversal_complete,
    )
}

#[test]
fn zero_and_tiny_retention_publish_the_same_scope_as_retention_with_room() {
    let none = FactsCapacity::new(0, 0);
    let tiny = FactsCapacity::new(1, 1);
    let roomy = FactsCapacity::new(1 << 12, 1 << 26);
    let (zero_report, zero_sink, zero_store) = run_with(none, 2);
    let (tiny_report, tiny_sink, tiny_store) = run_with(tiny, 2);
    let (roomy_report, roomy_sink, roomy_store) = run_with(roomy, 2);

    // (1) The same conclusion, whichever capacity the operation read through: the same records, in the
    //     same order, with the same identities, dispositions, texts and weights, and the same counts.
    for (capacity, report) in [
        (none, &zero_report),
        (tiny, &tiny_report),
        (roomy, &roomy_report),
    ] {
        assert_eq!(
            report.summary.status(),
            "complete",
            "{}: {:?}",
            capacity.describe(),
            report.summary
        );
        assert!(report.final_delivered, "{}", capacity.describe());
        assert_eq!(report.summary.limits.facts_capacity, capacity);
        assert_eq!(report.summary.classes_seen, 4, "{}", capacity.describe());
        assert_eq!(report.summary.classes_prepared, 4);
        assert_eq!(report.summary.classes_refused, 0);
        assert_eq!(report.summary.methods_declared, 18);
        assert_eq!(report.summary.methods_executed, 18);
        assert_eq!(report.summary.methods_delivered, 18);
    }
    assert_eq!(
        counted(&zero_report.summary),
        counted(&roomy_report.summary),
        "a store that retains nothing counts exactly what a store with room counts"
    );
    assert_eq!(
        counted(&tiny_report.summary),
        counted(&roomy_report.summary)
    );
    assert_eq!(
        delivered(&zero_sink),
        delivered(&roomy_sink),
        "the delivered methods are the same, in the same order, with the same texts"
    );
    assert_eq!(delivered(&tiny_sink), delivered(&roomy_sink));
    assert_eq!(
        zero_sink.prepared().len(),
        roomy_sink.prepared().len(),
        "and so are the classes the operation prepared"
    );

    // (2) The zero-capacity run really read through a store that kept nothing, and what that store
    //     refused is readable: it answered nothing, it stored nothing, and it refused every answer it
    //     was offered — by the entry bound or by the byte bound, both of which are published.
    let facts = zero_store.report();
    assert_eq!(facts.capacity, none);
    assert_eq!(facts.entries, 0, "{facts:?}");
    assert_eq!(facts.containers, 0, "{facts:?}");
    assert_eq!(facts.retained_bytes, 0, "{facts:?}");
    assert_eq!(facts.stored, 0, "nothing was retained: {facts:?}");
    assert_eq!(facts.container_stored, 0, "{facts:?}");
    assert!(
        facts.answered_nothing(),
        "a store that retains nothing answers nothing: {facts:?}"
    );
    assert!(
        facts.refused_capacity + facts.refused_capacity_bytes > 0,
        "the answers the operation produced were refused by the capacity it ran with: {facts:?}"
    );
    assert!(
        facts.container_consultations > 0 && facts.directory_parses > 0,
        "the operation really read through it: {facts:?}"
    );

    // (3) A capacity too small for anything is still a bound the store keeps, and it refused answers
    //     for the same reason — this is the "it does not fit" case between nothing and room.
    let facts = tiny_store.report();
    assert_eq!(facts.capacity, tiny);
    assert!(
        facts.entries <= tiny.entries && facts.retained_bytes <= tiny.retained_bytes,
        "the store held no more than it was declared to: {facts:?}"
    );
    assert!(
        facts.refused_capacity + facts.refused_capacity_bytes > 0,
        "a capacity of one byte refuses the answers a class read produces: {facts:?}"
    );

    // (4) Retention is what decides the fate of a container **no consumer is holding**, and nothing
    //     else changes. A container a class task is reading is never read twice in either run: the
    //     walk hands its own active handle to the task that reads the class (task 3.4, whose own
    //     target states that count on a scope of nested containers), so what is left for the store to
    //     answer is the lookups that have to consult a container nothing holds — a member's binding
    //     search that reaches a container no class task is reading any more. The run with room
    //     answers those from what it kept; the run without it pays for the directory again. The
    //     conclusion above is identical in both runs, which is the claim: not retaining re-reads, it
    //     does not re-decide.
    let roomy_facts = roomy_store.report();
    assert!(
        roomy_facts.container_hits > 0 && roomy_facts.container_stored > 0,
        "the run with room really kept and answered container facts: {roomy_facts:?}"
    );
    assert!(
        zero_store.report().directory_parses > roomy_facts.directory_parses,
        "a store that keeps nothing re-parses what it could not hold ({} parses) where retention \
         answers it ({}): {:?}",
        zero_store.report().directory_parses,
        roomy_facts.directory_parses,
        roomy_facts
    );
}

#[test]
fn a_consumed_result_is_handed_over_once_and_the_report_keeps_no_result() {
    // The fixture's own result weights first, so the window below bounds *these* results rather than a
    // number taken from thin air: with the largest real weight as the fixed per-result ceiling, every
    // method of the scope fits, and a window of two of them is far smaller than all eighteen summed —
    // which is what makes "the retained weight never crosses the window" a statement about releasing
    // results as they are consumed instead of about a ceiling nothing ever reached.
    let (_, learned, _learned_store) = run_with(FactsCapacity::new(0, 0), 1);
    let mut weights: Vec<u64> = learned
        .methods()
        .iter()
        .map(|method| method.weight)
        .collect();
    weights.sort_unstable();
    let ceiling = *weights
        .last()
        .expect("the fixture declares method records to learn a ceiling from");
    assert!(ceiling > 0, "the fixture's results really retain something");
    let window = ceiling.saturating_mul(2);

    // One run with a slow consumer and a window that holds two results, so every producer has to wait
    // for its own slot: the operation cannot accumulate results, and the weight it retains while a
    // consumer answers is bounded by the window's own ceiling.
    let (snapshot, _opened) = open(nested_fixture());
    let content = vec![snapshot.clone()];
    let mut setup = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut setup, &NESTED_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment, 2).with_capacities(DEFAULT_MAX_CLASS_BYTES, ceiling, window);
    assert_eq!(
        request.workers, 2,
        "the window holds exactly two results of this ceiling, so two class tasks run"
    );
    let mut budget = Budget::new(bulk_support::limits());
    let mut sink = Recorder::new();
    sink.delay = std::time::Duration::from_millis(5);
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");
    assert_eq!(report.summary.status(), "complete", "{:?}", report.summary);
    assert_eq!(report.summary.methods_delivered, 18);
    let retained: u64 = sink.methods().iter().map(|method| method.weight).sum();
    assert!(
        retained > report.window.buffered_weight_limit,
        "the scope's own results weigh more than the window holds, so they cannot all be held at once: \
         {retained} byte(s) of results against a {}-byte window",
        report.window.buffered_weight_limit
    );
    assert!(
        report.window.buffered_weight_high_water <= report.window.buffered_weight_limit,
        "a consumed result is released as soon as its consumer confirmed it, so the retained weight \
         never crosses the window's ceiling: {:?}",
        report.window
    );
    assert!(
        report.window.buffered_weight_high_water >= report.window.largest_result_weight
            && report.window.largest_result_weight > 0,
        "the run really retained the result it was waiting to deliver: {:?}",
        report.window
    );

    // (1) What the stream itself states: one header, each class's records between its own `ClassPrepared`
    //     and its `ClassEnd`, methods in the declaration's order, classes in the physical order, one
    //     `Final` last — the order the operation produced them in, one record per member, once.
    assert!(
        matches!(sink.events.first(), Some(Recorded::Header(_))),
        "the stream starts with its header"
    );
    assert!(
        matches!(sink.events.last(), Some(Recorded::Final(_))),
        "and ends with its final record"
    );
    let mut next_class = 0_u64;
    let mut open: Option<(u64, u64)> = None;
    for event in &sink.events {
        match event {
            Recorded::Header(_) | Recorded::Final(_) => {}
            Recorded::ClassPrepared(prepared) => {
                assert!(
                    open.is_none(),
                    "a class is prepared inside another class's frame"
                );
                assert_eq!(
                    prepared.class_ordinal, next_class,
                    "classes are delivered in the traversal's own physical order"
                );
                open = Some((prepared.class_ordinal, 0));
            }
            Recorded::Method(method) => {
                let (class, member) = open.expect("a method record sits inside its class's frame");
                assert_eq!(method.class_ordinal, class);
                assert_eq!(
                    method.member_ordinal, member,
                    "methods are delivered once each, in their declaration order"
                );
                open = Some((class, member + 1));
            }
            Recorded::ClassEnd(end) => {
                let (class, members) = open.take().expect("a class ends inside its own frame");
                assert_eq!(end.class_ordinal, class);
                let stated = match &end.completion {
                    ClassCompletion::Completed { methods } => *methods,
                    ClassCompletion::Stopped { methods, .. } => *methods,
                    ClassCompletion::Refused { code } => panic!(
                        "every class of this fixture is prepared, and this one was refused with \
                         `{code}`"
                    ),
                };
                assert_eq!(
                    members, stated,
                    "the class's own end states the records it handed over"
                );
                next_class = next_class.saturating_add(1);
            }
            Recorded::Diagnostic(_) => {}
        }
    }
    assert_eq!(
        next_class, 4,
        "every class of the scope was delivered in its frame"
    );

    // (2) What the returned report is: counts, boundaries and this run's own account — never the
    //     results. None of the delivered texts appears in it, which is the structural half a consumer
    //     can check for itself; the report's own fields are the counts, the coverage ranges and the
    //     account of the run that produced them.
    //
    //     A *size* bound is deliberately not claimed. The report's document is a function of the counts,
    //     the boundaries and the two limit documents it publishes (the operation's totals and the
    //     per-method limits), so a byte budget over it would be a statement about the configuration
    //     rather than about what the operation kept. What the absence above states is the retention
    //     claim itself, and the release the analysis performs is not observable from here at all.
    let document =
        serde_json::to_string(&report).expect("the report serializes as its own document");
    let texts: Vec<String> = sink
        .methods()
        .iter()
        .filter_map(|method| method.text.clone())
        .collect();
    assert!(
        !texts.is_empty(),
        "this scope delivered real text, which is what the report must not keep"
    );
    for text in &texts {
        assert!(
            !document.contains(text.as_str()),
            "the report holds no per-method result, so no delivered text appears in it"
        );
    }
    for field in ["\"summary\"", "\"coverage\"", "\"usage\"", "\"window\""] {
        assert!(
            document.contains(field),
            "and what it does hold is this operation's own account ({field}): {document}"
        );
    }
}
