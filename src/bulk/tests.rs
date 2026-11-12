//! Deterministic drain coverage: the real class worker unwinds and joins before any class record
//! can be consumed. The public lifecycle test separately covers Engine orchestration.

use super::*;
use crate::{
    ArtifactInput, CancellationToken, EnvironmentPolicy, EnvironmentRequest, LayoutMode, LoadRoot,
    LoaderId, MultiReleasePolicy, PhysicalScope, RuntimeProfile,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum Received {
    Header,
    Prepared(u64),
    Method(u64, u64),
    End(u64, ClassCompletion),
}

#[derive(Clone, Copy)]
enum Action {
    Continue,
    StopPrepared,
    StopSecondMethod,
    FailSecondMethod,
    CancelAfterFirstMethod,
    ExhaustOutputAfterFirstMethod,
}

struct Recorder {
    calls: Vec<Received>,
    action: Action,
    token: CancellationToken,
    account: Option<DeliveryAccount>,
}

impl Recorder {
    fn new(action: Action) -> Self {
        Self {
            calls: Vec::new(),
            action,
            token: CancellationToken::new(),
            account: None,
        }
    }
}

impl RecoverySink for Recorder {
    fn header(&mut self, _: &BulkHeaderEvent, account: DeliveryAccount) -> Result<SinkControl> {
        self.calls.push(Received::Header);
        self.account = Some(account);
        Ok(SinkControl::Continue)
    }

    fn class_prepared(&mut self, event: &ClassPreparedEvent) -> Result<SinkControl> {
        self.calls.push(Received::Prepared(event.class_ordinal));
        Ok(if matches!(self.action, Action::StopPrepared) {
            SinkControl::Stop
        } else {
            SinkControl::Continue
        })
    }

    fn method(&mut self, event: &MethodResultEvent) -> Result<SinkControl> {
        self.calls
            .push(Received::Method(event.class_ordinal, event.member_ordinal));
        match (self.action, event.member_ordinal) {
            (Action::StopSecondMethod, 1) => return Ok(SinkControl::Stop),
            (Action::FailSecondMethod, 1) => {
                return Err(Error::invalid_input(
                    "drain_sink_failure",
                    "injected failure",
                ));
            }
            (Action::CancelAfterFirstMethod, 0) => self.token.cancel(),
            (Action::ExhaustOutputAfterFirstMethod, 0) => {
                let account = self.account.as_ref().expect("header delivered the account");
                account.charge_output_bytes(account.remaining_output_bytes())?;
            }
            (Action::ExhaustOutputAfterFirstMethod, 1) => {
                self.account
                    .as_ref()
                    .expect("header delivered the account")
                    .charge_output_bytes(1)?;
            }
            _ => {}
        }
        Ok(SinkControl::Continue)
    }

    fn class_end(&mut self, event: &ClassEndEvent) -> Result<SinkControl> {
        self.calls
            .push(Received::End(event.class_ordinal, event.completion.clone()));
        if matches!(event.completion, ClassCompletion::Completed { .. }) {
            assert!(matches!(event.execution, ExecutionReport::Complete { .. }));
        } else {
            assert!(matches!(
                &event.execution,
                ExecutionReport::Failed { reason: TerminationReason::Error { code }, .. }
                    if code == "bulk_worker_panicked"
            ));
        }
        Ok(SinkControl::Continue)
    }

    fn diagnostic(&mut self, _: &BulkDiagnosticEvent) -> Result<SinkControl> {
        panic!("a prepared worker panic does not take the Refused branch");
    }

    fn final_event(&mut self, _: &BulkFinalEvent) -> Result<SinkControl> {
        panic!("the class drain does not publish Final");
    }
}

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 24,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 100_000,
        output_bytes: 1 << 24,
        class_headers: 10_000,
        method_bodies: 10_000,
        ir_items: 1 << 22,
        ir_edges: 1 << 22,
        analysis_steps: 1 << 22,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn with_pending_panic(sink: &mut Recorder, check: impl FnOnce(&Operation<'_>, &mut Delivery<'_>)) {
    with_operation(sink, 0, u64::MAX, |operation, delivery, class| {
        operation.registry.note_seen();
        operation.registry.dispatch_class(class, None);
        operation.registry.note_traversal_done();
        std::thread::scope(|scope| {
            assert!(scope.spawn(|| worker_loop(operation)).join().is_err());
        });
        // No coordinator was running: these records are demonstrably pending at the actual panic,
        // not merely likely to be pending under one scheduler. Both reservation and pool are occupied.
        {
            let state = operation.registry.lock();
            let slot = state.slots.front().expect("pending class");
            assert!(matches!(slot.preparation, PreparationState::Pending { .. }));
            assert_eq!(slot.preparation.methods_produced(), Some(2));
            assert_eq!(slot.methods.len(), 2);
            assert_eq!(slot.methods[0].pool_charge, 0);
            assert!(slot.methods[1].pool_charge > 0);
            assert!(state.retained_weight > 0);
            assert_eq!(state.pool_used, slot.methods[1].pool_charge);
            assert_eq!(state.counts.methods_executed, 2);
            assert_eq!(state.counts.methods_delivered, 0);
            assert_eq!(state.executing, 0);
        }
        assert!(matches!(
            take_front(operation.registry, delivery.budget),
            Front::Closed
        ));
        check(operation, delivery);
    });
}

fn with_operation(
    sink: &mut Recorder,
    panic_class: u64,
    elapsed: u64,
    check: impl FnOnce(&Operation<'_>, &mut Delivery<'_>, ScopeClass),
) {
    let snapshot = ArtifactSnapshot::open(
        ArtifactInput::bytes(
            include_bytes!("../../tests/fixtures/p3-scope/v8/Scope.class").to_vec(),
        ),
        &mut Budget::new(limits()),
    )
    .expect("the existing Scope fixture opens");
    let environment = EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::ExplicitClasspath {
            roots: vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            }],
        },
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".to_owned()),
    };
    let mut total = limits();
    total.elapsed_millis = elapsed;
    let mut request = BulkRecoveryRequest::for_scope(environment, 2, total.clone());
    request.faults.panic_after_method = Some((panic_class, 1));
    let content = [snapshot];
    let snapshot = &content[0];
    let environment = request.environment.build(&content).expect("environment");
    let mut budget = Budget::with_cancellation_token(total, sink.token.clone());
    let store = crate::OperationStore::of(&budget);
    let limits = BulkLimits::effective_from(&request, budget.limits(), store.capacity())
        .expect("valid window");
    let ledger = OperationLedger::new(&budget);
    budget.with_ledger(ledger.clone(), UsageOwner::Discovery);
    let observation = Observation::of(None);
    let registry = Registry::new(observation.clone(), limits.shared_result_pool_weight);
    let operation = Operation {
        source_target: request.source_target,
        content: &content,
        snapshot,
        environment: &environment,
        limits: &limits,
        ledger: &ledger,
        stages: crate::facade::MethodOperation::Recovery.stages(),
        evidence: &request.evidence,
        registry: &registry,
        store,
        discovery_limits: budget.limits().clone(),
        observation: observation.clone(),
        faults: Faults::from_request(&request),
    };
    let account = DeliveryAccount::of(&ledger, budget.limits());
    let mut delivery = Delivery {
        sink,
        budget: &mut budget,
        account,
        closed: false,
        observation,
    };
    let header = BulkHeaderEvent {
        source_target: request.source_target,
        runtime_profile: environment.runtime.profile.clone(),
        view: PhysicalView {
            snapshot: request.snapshot.clone(),
            scope: request.scope.clone(),
        },
        limits: limits.clone(),
    };
    assert!(publish(
        &operation,
        &mut delivery,
        Record::Header(&header),
        None
    ));
    let mut cursor = snapshot.scope_cursor(&request.scope).expect("scope cursor");
    let mut discovery =
        operation.budget_for(operation.discovery_limits.clone(), UsageOwner::Discovery);
    let class = cursor
        .next_class(&mut discovery)
        .expect("read class")
        .expect("one class");
    check(&operation, &mut delivery, class);
    assert!(
        matches!(
            registry.ending(),
            Some(Ending::Infrastructure { code, .. }) if code == "bulk_worker_panicked"
        ),
        "the first infrastructure cause survives delivery stops"
    );
    // Whether delivered or discarded, each charge leaves exactly once. The high-water readings
    // are retained; cleanup must not erase the evidence of the window that really existed.
    let mut state = registry.lock();
    let retained_peak = state.retained_weight_high_water;
    let pool_peak = state.pool_used_high_water;
    while !state.slots.is_empty() {
        retire_front(&mut state);
    }
    assert_eq!((state.retained_weight, state.pool_used), (0, 0));
    assert_eq!(state.retained_weight_high_water, retained_peak);
    assert_eq!(state.pool_used_high_water, pool_peak);
}

#[test]
fn infrastructure_drain_delivers_seated_methods_before_the_panicked_class_end() {
    let mut sink = Recorder::new(Action::Continue);
    with_pending_panic(&mut sink, |operation, delivery| {
        assert!(drain_ended_classes(operation, delivery, false));
        let state = operation.registry.lock();
        assert!(state.slots.is_empty());
        assert_eq!((state.retained_weight, state.pool_used), (0, 0));
        assert_eq!(state.counts.methods_delivered, 2);
    });
    assert_eq!(
        sink.calls,
        vec![
            Received::Header,
            Received::Prepared(0),
            Received::Method(0, 0),
            Received::Method(0, 1),
            Received::End(
                0,
                ClassCompletion::Stopped {
                    methods: 2,
                    code: "bulk_worker_panicked".to_owned(),
                }
            ),
        ]
    );
}

#[test]
fn infrastructure_drain_stops_at_a_refused_prepared_record() {
    let mut sink = Recorder::new(Action::StopPrepared);
    with_pending_panic(&mut sink, |operation, delivery| {
        assert!(!drain_ended_classes(operation, delivery, false));
        assert!(!drain_ended_classes(operation, delivery, false));
        let state = operation.registry.lock();
        assert_eq!(
            state.slots.front().expect("unretired slot").methods.len(),
            2
        );
        assert_eq!(state.counts.methods_delivered, 0);
    });
    assert_eq!(sink.calls, vec![Received::Header, Received::Prepared(0)]);
}

#[test]
fn infrastructure_drain_does_not_confirm_a_method_the_sink_stops_or_fails_on() {
    for action in [Action::StopSecondMethod, Action::FailSecondMethod] {
        let mut sink = Recorder::new(action);
        with_pending_panic(&mut sink, |operation, delivery| {
            assert!(!drain_ended_classes(operation, delivery, false));
            assert!(!drain_ended_classes(operation, delivery, false));
            let state = operation.registry.lock();
            assert_eq!(state.counts.methods_delivered, 1);
            assert_eq!((state.retained_weight, state.pool_used), (0, 0));
        });
        assert_eq!(
            sink.calls,
            vec![
                Received::Header,
                Received::Prepared(0),
                Received::Method(0, 0),
                Received::Method(0, 1)
            ]
        );
    }
}

#[test]
fn infrastructure_drain_observes_token_cancellation_between_seated_methods() {
    let mut sink = Recorder::new(Action::CancelAfterFirstMethod);
    with_pending_panic(&mut sink, |operation, delivery| {
        assert!(!drain_ended_classes(operation, delivery, false));
        assert!(!drain_ended_classes(operation, delivery, false));
        assert_eq!(operation.registry.lock().counts.methods_delivered, 1);
        assert_eq!(
            operation.ledger.stop_reason().expect("token stop").kind,
            BulkStopKind::Cancelled
        );
    });
    assert_eq!(
        sink.calls,
        vec![
            Received::Header,
            Received::Prepared(0),
            Received::Method(0, 0)
        ]
    );
}

#[test]
fn infrastructure_drain_cannot_publish_a_method_without_a_result_item_permit() {
    let mut sink = Recorder::new(Action::Continue);
    with_pending_panic(&mut sink, |operation, delivery| {
        // Leave exactly one operation-wide permit: Prepared can publish, the first method cannot.
        let used = operation
            .ledger
            .usage()
            .counted_usage(CountedBudgetDimension::ResultItems);
        operation
            .ledger
            .charge(
                UsageOwner::Delivery,
                CountedBudgetDimension::ResultItems,
                operation.limits.total.result_items - used - 1,
            )
            .expect("remaining quota");
        assert!(!drain_ended_classes(operation, delivery, false));
        assert!(!drain_ended_classes(operation, delivery, false));
        assert_eq!(operation.registry.lock().counts.methods_delivered, 0);
        let stop = operation.ledger.stop_reason().expect("result item stop");
        assert_eq!(stop.kind, BulkStopKind::Budget);
        assert_eq!(stop.dimension, Some(BudgetDimension::ResultItems));
        assert_eq!(
            operation
                .ledger
                .usage()
                .counted_usage(CountedBudgetDimension::ResultItems),
            operation.limits.total.result_items
        );
    });
    assert_eq!(sink.calls, vec![Received::Header, Received::Prepared(0)]);
}

#[test]
fn infrastructure_drain_respects_the_sinks_shared_output_budget() {
    let mut sink = Recorder::new(Action::ExhaustOutputAfterFirstMethod);
    with_pending_panic(&mut sink, |operation, delivery| {
        assert!(!drain_ended_classes(operation, delivery, false));
        assert!(!drain_ended_classes(operation, delivery, false));
        assert_eq!(operation.registry.lock().counts.methods_delivered, 1);
        let stop = operation.ledger.stop_reason().expect("output stop");
        assert_eq!(stop.kind, BulkStopKind::Budget);
        assert_eq!(stop.dimension, Some(BudgetDimension::OutputBytes));
        assert_eq!(
            operation
                .ledger
                .usage()
                .counted_usage(CountedBudgetDimension::OutputBytes),
            operation.limits.total.output_bytes
        );
    });
    assert_eq!(
        sink.calls,
        vec![
            Received::Header,
            Received::Prepared(0),
            Received::Method(0, 0),
            Received::Method(0, 1)
        ]
    );
}

#[test]
fn infrastructure_drain_preserves_each_already_confirmed_prefix_exactly_once() {
    for prefix in 0..=2 {
        let mut sink = Recorder::new(Action::Continue);
        with_pending_panic(&mut sink, |operation, delivery| {
            // Recreate a coordinator that already owns Prepared and a confirmed method prefix.
            // The real worker has still produced and seated every event; no synthetic event counts.
            let control = operation
                .registry
                .lock()
                .slots
                .front_mut()
                .unwrap()
                .preparation
                .hand_off()
                .expect("pending preparation");
            assert!(publish(
                operation,
                delivery,
                Record::Prepared(&control),
                None
            ));
            for _ in 0..prefix {
                let method = take_pending_method(&mut operation.registry.lock()).unwrap();
                assert!(publish(
                    operation,
                    delivery,
                    Record::Method(&method),
                    Some(&method)
                ));
            }
            assert_eq!(operation.registry.lock().counts.methods_delivered, prefix);
            assert!(drain_ended_classes(operation, delivery, false));
            assert!(drain_ended_classes(operation, delivery, false));
            let state = operation.registry.lock();
            assert_eq!(state.counts.methods_delivered, 2);
            assert!(state.slots.is_empty());
            assert_eq!((state.retained_weight, state.pool_used), (0, 0));
        });
        assert_eq!(
            sink.calls,
            vec![
                Received::Header,
                Received::Prepared(0),
                Received::Method(0, 0),
                Received::Method(0, 1),
                Received::End(
                    0,
                    ClassCompletion::Stopped {
                        methods: 2,
                        code: "bulk_worker_panicked".to_owned(),
                    }
                )
            ]
        );
    }
}

#[test]
fn infrastructure_drain_obeys_cancellation_before_prepared() {
    let mut sink = Recorder::new(Action::Continue);
    with_pending_panic(&mut sink, |operation, delivery| {
        delivery.budget.cancellation_token().cancel();
        assert!(!drain_ended_classes(operation, delivery, false));
        assert!(!drain_ended_classes(operation, delivery, false));
        let state = operation.registry.lock();
        assert_eq!(state.counts.methods_delivered, 0);
        assert_eq!(state.slots.front().unwrap().methods.len(), 2);
        assert_eq!(
            operation.ledger.stop_reason().unwrap().kind,
            BulkStopKind::Cancelled
        );
    });
    assert_eq!(sink.calls, vec![Received::Header]);
}

#[test]
fn infrastructure_drain_orders_completed_and_panicked_slots_and_releases_both() {
    let mut sink = Recorder::new(Action::Continue);
    let mut expected = vec![Received::Header];
    // Two dispatch positions of the real class exercise registry ownership independently of
    // scope discovery. This is not evidence for distinct-class discovery (public tests cover it).
    with_operation(&mut sink, 1, u64::MAX, |operation, delivery, class| {
        for _ in 0..2 {
            operation.registry.note_seen();
            operation.registry.dispatch_class(class.clone(), None);
        }
        operation.registry.note_traversal_done();
        std::thread::scope(|scope| {
            assert!(scope.spawn(|| worker_loop(operation)).join().is_err());
        });
        let produced = {
            let state = operation.registry.lock();
            assert_eq!(state.slots.len(), 2);
            assert!(matches!(
                state.slots[0].ending,
                Some(ClassEnding::Prepared { .. })
            ));
            assert!(matches!(
                state.slots[1].ending,
                Some(ClassEnding::Closed { .. })
            ));
            assert_eq!(state.slots[1].methods.len(), 2);
            let counts: Vec<_> = state
                .slots
                .iter()
                .map(|slot| slot.methods.len() as u64)
                .collect();
            assert_eq!(state.counts.methods_executed, counts.iter().sum::<u64>());
            assert_eq!(state.counts.methods_delivered, 0);
            assert_eq!(
                state.retained_weight,
                state
                    .slots
                    .iter()
                    .flat_map(|slot| &slot.methods)
                    .map(|p| p.event.weight)
                    .sum::<u64>()
            );
            assert_eq!(
                state.pool_used,
                state
                    .slots
                    .iter()
                    .flat_map(|slot| &slot.methods)
                    .map(|p| p.pool_charge)
                    .sum::<u64>()
            );
            counts
        };
        for (ordinal, &count) in produced.iter().enumerate() {
            expected.push(Received::Prepared(ordinal as u64));
            for member in 0..count {
                expected.push(Received::Method(ordinal as u64, member));
            }
            expected.push(Received::End(
                ordinal as u64,
                if ordinal == 0 {
                    ClassCompletion::Completed { methods: count }
                } else {
                    ClassCompletion::Stopped {
                        methods: count,
                        code: "bulk_worker_panicked".to_owned(),
                    }
                },
            ));
        }
        assert!(drain_ended_classes(operation, delivery, false));
        assert!(drain_ended_classes(operation, delivery, false));
        let state = operation.registry.lock();
        assert!(state.slots.is_empty());
        assert_eq!(state.counts.methods_delivered, produced.iter().sum::<u64>());
        assert_eq!((state.retained_weight, state.pool_used), (0, 0));
    });
    assert_eq!(sink.calls, expected);
}

#[test]
fn infrastructure_drain_checks_the_original_elapsed_deadline_before_callbacks() {
    let mut sink = Recorder::new(Action::Continue);
    with_operation(&mut sink, 0, 2_000, |operation, delivery, class| {
        operation.registry.note_seen();
        operation.registry.dispatch_class(class, None);
        operation.registry.note_traversal_done();
        std::thread::scope(|scope| {
            assert!(scope.spawn(|| worker_loop(operation)).join().is_err());
        });
        assert_eq!(operation.registry.lock().slots[0].methods.len(), 2);
        // Wait against this operation's actual clock, rather than assuming one sleep expired it.
        while delivery.budget.usage().elapsed_millis <= delivery.budget.limits().elapsed_millis {
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(!drain_ended_classes(operation, delivery, false));
        assert!(!drain_ended_classes(operation, delivery, false));
        assert_eq!(operation.registry.lock().counts.methods_delivered, 0);
        let stop = operation.ledger.stop_reason().expect("deadline stop");
        assert_eq!(stop.dimension, Some(BudgetDimension::ElapsedMillis));
        assert_eq!(operation.registry.lock().slots[0].methods.len(), 2);
    });
    assert_eq!(sink.calls, vec![Received::Header]);
}

struct SeatedRendezvous<'a> {
    inner: SlotConsumer<'a>,
    seated: std::sync::mpsc::SyncSender<()>,
    release: std::sync::mpsc::Receiver<()>,
}
impl ResultConsumer for SeatedRendezvous<'_> {
    fn prepared(&mut self, event: ClassPreparedEvent) -> std::result::Result<bool, Closing> {
        self.inner.prepared(event)
    }
    fn method(&mut self, event: MethodResultEvent) -> std::result::Result<bool, Closing> {
        let rendezvous = event.member_ordinal == 1;
        let result = self.inner.method(event)?;
        if result && rendezvous {
            self.seated.send(()).expect("coordinator present");
            self.release
                .recv_timeout(Duration::from_secs(5))
                .expect("coordinator released worker");
        }
        Ok(result)
    }
}

#[test]
fn coordinator_confirms_prefix_before_worker_panics_without_losing_or_replaying_records() {
    for prefix in 0..=2 {
        let mut sink = Recorder::new(Action::Continue);
        with_operation(&mut sink, 0, u64::MAX, |operation, delivery, class| {
            operation.registry.note_seen();
            operation.registry.dispatch_class(class, None);
            operation.registry.note_traversal_done();
            std::thread::scope(|scope| {
                let (seated_tx, seated_rx) = std::sync::mpsc::sync_channel(0);
                let (release_tx, release_rx) = std::sync::mpsc::sync_channel(0);
                let worker = scope.spawn(move || {
                    let (index, class, _container) = operation.registry.take_task().unwrap();
                    let _guard = TaskGuard { operation, index };
                    let mut budget = operation
                        .budget_for(operation.discovery_limits.clone(), UsageOwner::Discovery);
                    let mut consumer = SeatedRendezvous {
                        inner: SlotConsumer {
                            registry: operation.registry,
                            index,
                        },
                        seated: seated_tx,
                        release: release_rx,
                    };
                    run_class_task(operation, &class, index, &mut budget, &mut consumer)
                });
                seated_rx
                    .recv_timeout(Duration::from_secs(5))
                    .expect("both methods seated");
                assert!(operation.registry.ending().is_none());
                let Front::Control(control) = take_front(operation.registry, delivery.budget)
                else {
                    panic!("Prepared before methods")
                };
                assert!(publish(
                    operation,
                    delivery,
                    Record::Prepared(&control),
                    None
                ));
                for member in 0..prefix {
                    let Front::Method(method) = take_front(operation.registry, delivery.budget)
                    else {
                        panic!("seated method")
                    };
                    assert_eq!(method.member_ordinal, member);
                    assert!(publish(
                        operation,
                        delivery,
                        Record::Method(&method),
                        Some(&method)
                    ));
                }
                assert_eq!(operation.registry.lock().counts.methods_delivered, prefix);
                release_tx.send(()).expect("worker waiting after seating");
                assert!(worker.join().is_err());
            });
            assert!(drain_ended_classes(operation, delivery, false));
            assert!(drain_ended_classes(operation, delivery, false));
            let state = operation.registry.lock();
            assert_eq!(state.counts.methods_delivered, 2);
            assert_eq!(state.counts.methods_executed, 2);
            assert!(state.slots.is_empty());
            assert_eq!((state.retained_weight, state.pool_used), (0, 0));
        });
        assert_eq!(
            sink.calls,
            vec![
                Received::Header,
                Received::Prepared(0),
                Received::Method(0, 0),
                Received::Method(0, 1),
                Received::End(
                    0,
                    ClassCompletion::Stopped {
                        methods: 2,
                        code: "bulk_worker_panicked".to_owned()
                    }
                )
            ]
        );
    }
}
