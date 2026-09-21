//! Bulk task 4.6 through the public entry: the consumer's own bytes are charged to the operation's
//! **one** account.
//!
//! A streamed operation hands its sink a [`DeliveryAccount`] with the header. The tests below use a
//! consumer that really pays for what it "writes" through that account, and they pin the three parts of
//! the design decision 4 claim:
//!
//! * **one total, not two.** The bytes a consumer writes are billed as the operation's `delivery` work
//!   beside the text the library emitted as its `methods` work, and the two **add up**: the account's
//!   total is `entry + discovery + methods + delivery`, and it never crosses the declared
//!   `output_bytes` — whichever part of the operation reaches that number first.
//! * **a refused permit writes nothing.** A record the account refuses is refused whole: it adds no byte
//!   to what the consumer wrote, and the stop the ledger records names the dimension it needed and the
//!   **delivery owner** that needed it, so the run's end is locatable rather than a silently short
//!   stream.
//! * **a worker count does not widen the total.** The same declaration funds the same records at one
//!   worker and at four: the account belongs to the operation, so adding workers cannot buy more output.
//!
//! The consumer states a **fixed** byte cost per record instead of a serialized length on purpose: a
//! fixed cost makes the arithmetic below a property of the account rather than a property of this
//! fixture's digits, which is what makes the refused record a *chosen* one instead of whichever record
//! happened to be the longest.

mod bulk_support;

use bulk_support::{FLAT_PREFIXES, container_roots, environment, flat_fixture, open, tree_scope};
use jarde::*;

/// Where one record sits in the stream, as a consumer can name it without holding the event.
#[derive(Clone, Debug, Eq, PartialEq)]
struct RecordId {
    kind: &'static str,
    class: Option<u64>,
    member: Option<u64>,
}

/// A consumer that pays for its own output out of the operation's one account.
///
/// It models an encoder with a known cost: every record it accepts is charged `cost` bytes through the
/// account the header handed over, and it confirms the record only once the permit is in hand — the
/// bytes it "wrote" are exactly the bytes it paid for. A record whose permit is refused is refused
/// whole: nothing is counted for it, and the refusal is kept so a test can read the numbers the account
/// refused with.
#[derive(Default)]
struct Metered {
    /// The operation's account, handed over with the header.
    account: Option<DeliveryAccount>,
    /// What this consumer pays for one record.
    cost: u64,
    /// The records this consumer confirmed, in the order the operation delivered them.
    accepted: Vec<RecordId>,
    /// The bytes this consumer paid for, which are the bytes it wrote.
    written: u64,
    /// How many records this consumer was handed: the ones it confirmed plus the one it refused.
    handed: u64,
    /// The stop reason of every delivered method record that states one: what the library's own
    /// presentation of that method stopped for.
    stops: Vec<StopReason>,
    /// The summary the streamed terminal record carried, when one was delivered.
    final_summary: Option<BulkSummary>,
    /// The refusal, when the account refused a record.
    refused: Option<Error>,
}

impl Metered {
    fn new(cost: u64) -> Self {
        Self {
            cost,
            ..Self::default()
        }
    }

    /// Charges one record and answers whether the stream goes on.
    fn take(&mut self, id: RecordId) -> Result<SinkControl> {
        self.handed = self.handed.saturating_add(1);
        let account = self
            .account
            .clone()
            .expect("the header hands the operation's account over before any record follows it");
        match account.charge_output_bytes(self.cost) {
            Ok(()) => {
                self.written = self.written.saturating_add(self.cost);
                self.accepted.push(id);
                Ok(SinkControl::Continue)
            }
            Err(error) => {
                self.refused = Some(error);
                Ok(SinkControl::Stop)
            }
        }
    }
}

impl RecoverySink for Metered {
    fn header(
        &mut self,
        _event: &BulkHeaderEvent,
        delivery: DeliveryAccount,
    ) -> Result<SinkControl> {
        self.account = Some(delivery);
        self.take(RecordId {
            kind: "header",
            class: None,
            member: None,
        })
    }

    fn class_prepared(&mut self, event: &ClassPreparedEvent) -> Result<SinkControl> {
        self.take(RecordId {
            kind: "class_prepared",
            class: Some(event.class_ordinal),
            member: None,
        })
    }

    fn method(&mut self, event: &MethodResultEvent) -> Result<SinkControl> {
        if let MethodDelivery::Recovered(recovered) = &event.delivery
            && let Some(stop) = recovered.recovery().outcome.stop()
        {
            self.stops.push(stop.clone());
        }
        self.take(RecordId {
            kind: "method",
            class: Some(event.class_ordinal),
            member: Some(event.member_ordinal),
        })
    }

    fn class_end(&mut self, event: &ClassEndEvent) -> Result<SinkControl> {
        self.take(RecordId {
            kind: "class_end",
            class: Some(event.class_ordinal),
            member: None,
        })
    }

    fn diagnostic(&mut self, event: &BulkDiagnosticEvent) -> Result<SinkControl> {
        self.take(RecordId {
            kind: "diagnostic",
            class: event.class_ordinal,
            member: None,
        })
    }

    fn final_event(&mut self, event: &BulkFinalEvent) -> Result<SinkControl> {
        self.final_summary = Some(event.summary.clone());
        self.take(RecordId {
            kind: "final",
            class: None,
            member: None,
        })
    }
}

/// One whole run of the flat fixture: `total` is the entry budget's own limits — the operation's one
/// total — `method` the local limits one method runs under, and `cost` what the consumer pays per
/// record.
///
/// The two limits are separate parameters on purpose. Tightening `total` bounds the **operation** and
/// leaves every method's own allowance where it was, which is what makes these cases about the shared
/// total: a `method` limit tightened as well would stop methods locally, and a local stop is a method's
/// disposition rather than the operation's stop.
fn run(workers: usize, total: Limits, method: Limits, cost: u64) -> (BulkRecoveryReport, Metered) {
    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut setup = Budget::new(total.clone());
    let roots = container_roots(&snapshot, &mut setup, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = BulkRecoveryRequest::for_scope(environment, workers, method);
    let mut budget = Budget::new(total);
    let mut sink = Metered::new(cost);
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");
    (report, sink)
}

/// What the library itself charged to `output_bytes` in one run: everything the operation produced,
/// with no consumer's bytes in it.
fn emitted(report: &BulkRecoveryReport) -> u64 {
    report
        .discovery_usage
        .counted_usage(CountedBudgetDimension::OutputBytes)
        .saturating_add(
            report
                .method_usage
                .counted_usage(CountedBudgetDimension::OutputBytes),
        )
}

/// The usage one execution plane carries, at the moment it was assembled.
fn execution_usage(execution: &ExecutionReport) -> &UsageSnapshot {
    match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    }
}

/// How many methods state an `output_bytes` budget stop: the library's own presentation meeting the
/// dimension the operation shares with its consumer.
fn budget_stops(sink: &Metered) -> usize {
    sink.stops
        .iter()
        .filter(|stop| {
            matches!(
                stop,
                StopReason::Budget {
                    dimension: CountedBudgetDimension::OutputBytes,
                    ..
                }
            )
        })
        .count()
}

#[test]
fn a_refused_permit_stops_the_stream_at_the_one_total_and_names_the_delivery_owner() {
    // What the operation's own work costs in `output_bytes`, measured under a ceiling nothing reaches:
    // the text the library emits for the whole scope. The consumer below then costs *more per record*
    // than the whole run emitted, which is what makes the account's end the consumer's end: with a cost
    // `C` larger than everything the library charged, a total of `L + m * C` funds exactly `m` records
    // and the `(m + 1)`-th charge can never fit, whatever the worker count or the interleaving is.
    let wide = bulk_support::limits();
    let (complete, whole) = run(1, wide.clone(), wide.clone(), 4096);
    assert_eq!(
        complete.summary.status(),
        "complete",
        "{:?}",
        complete.summary
    );
    assert!(
        complete.final_delivered,
        "the unconstrained run reached its final record"
    );
    let emitted_bytes = emitted(&complete);
    assert!(
        emitted_bytes > 0,
        "the fixture's methods really emitted text, which is the library's own charge to the same \
         dimension: {emitted_bytes} byte(s)"
    );
    let cost = emitted_bytes.saturating_add(4096);
    let funded = 6_u64;
    let allowance = emitted_bytes.saturating_add(funded.saturating_mul(cost));

    for workers in [1_usize, 4] {
        let mut total = wide.clone();
        total.output_bytes = allowance;
        let (report, sink) = run(workers, total, wide.clone(), cost);

        // (1) The refusal is the account's, it names the dimension it needed, and it states the
        //     numbers: the total, what was already admitted, and what the refused record required.
        let refusal = sink
            .refused
            .as_ref()
            .expect("the account refuses the record after the funded ones");
        match refusal {
            Error::BudgetExceeded {
                dimension,
                limit,
                consumed,
                requested,
            } => {
                assert_eq!(*dimension, BudgetDimension::OutputBytes);
                assert_eq!(*limit, allowance, "the total the run was opened with");
                assert_eq!(
                    *consumed,
                    emitted(&report).saturating_add(funded * cost),
                    "what was admitted before the refusal: the library's own work so far *and* every \
                     record this consumer paid for"
                );
                assert_eq!(*requested, cost, "what the refused record needed");
                assert!(
                    *consumed + *requested > *limit,
                    "and the record really did not fit: {consumed} + {requested} > {limit}"
                );
            }
            other => panic!("the refusal is a budget refusal: {other:?}"),
        }

        // (2) Not one byte was counted for the refused record, and every funded record was: a consumer
        //     that writes what it paid for wrote exactly `funded` records' worth of bytes.
        assert_eq!(
            sink.handed,
            funded + 1,
            "the consumer was handed its funded records and the one it refused ({workers} worker(s))"
        );
        assert_eq!(
            sink.accepted.len(),
            usize::try_from(funded).expect("the funded count fits usize")
        );
        assert_eq!(sink.written, funded * cost);
        assert_eq!(
            report.summary.methods_delivered,
            sink.accepted
                .iter()
                .filter(|record| record.kind == "method")
                .count() as u64,
            "the delivered count is exactly the consumer's confirmations, no more"
        );

        // (3) The consumer's bytes are the operation's `delivery` work, and the operation's total is the
        //     sum of its parts — the library's own emission and the consumer's bytes added up in one
        //     account that stayed inside the number it was declared under.
        let entry = report
            .entry_usage
            .counted_usage(CountedBudgetDimension::OutputBytes);
        let discovery = report
            .discovery_usage
            .counted_usage(CountedBudgetDimension::OutputBytes);
        let methods = report
            .method_usage
            .counted_usage(CountedBudgetDimension::OutputBytes);
        let delivery = report
            .delivery_usage
            .counted_usage(CountedBudgetDimension::OutputBytes);
        assert_eq!(
            delivery,
            funded * cost,
            "the consumer's own bytes are billed as the operation's delivery work"
        );
        assert_eq!(
            report
                .usage
                .counted_usage(CountedBudgetDimension::OutputBytes),
            entry + discovery + methods + delivery,
            "one account: the entry, the library's work and the consumer's bytes sum to the total"
        );
        assert!(
            report
                .usage
                .counted_usage(CountedBudgetDimension::OutputBytes)
                <= allowance,
            "and the sum never crosses the declaration: {:?}",
            report.usage
        );
        assert!(sink.written <= allowance);
        assert!(
            emitted(&report) <= emitted_bytes,
            "the stopped run did no more library work than the complete one"
        );

        // (4) The stop is locatable — the dimension it needed *and* the owner that needed it — and the
        //     stream is a prefix: the consumer's confirmed records are the operation's own first records,
        //     in the operation's own order.
        assert_eq!(
            report.stop,
            Some(BulkStop {
                owner: Some(UsageOwner::Delivery),
                kind: BulkStopKind::Budget,
                dimension: Some(BudgetDimension::OutputBytes),
            }),
            "the first stop names the delivery owner and the dimension ({workers} worker(s))"
        );
        assert!(
            matches!(
                report.summary.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: BudgetDimension::OutputBytes,
                    },
                    ..
                }
            ),
            "an account that ran out is a partial run, never a complete one: {:?}",
            report.summary.execution
        );
        assert!(
            !report.final_delivered,
            "no `final` record masquerades as a completion the account did not fund"
        );
        assert_eq!(
            sink.accepted,
            whole.accepted[..sink.accepted.len()].to_vec(),
            "the delivered prefix is the operation's own records in its own order ({workers} worker(s))"
        );
        assert!(
            report.summary.methods_executed > report.summary.methods_delivered,
            "the method the consumer refused ran and is billed even though it was never confirmed: \
             {:?}",
            report.summary
        );
    }
}

#[test]
fn one_declaration_bounds_the_librarys_own_presentation_too() {
    // The other side of "one total": a total that funds less than what the scope's own presentation
    // costs does not stop the *operation* — the library's presentation of a method that runs out of the
    // total stops **that method**, discarding the artifact it cannot afford and stating the dimension it
    // needed — and the operation goes on to deliver every record of the scope. What must never happen,
    // from either side of the stream, is the account crossing its declaration: the library's work and
    // the consumer's bytes are two parts of one number, and this case states the library's part.
    let wide = bulk_support::limits();
    let (complete, complete_sink) = run(1, wide.clone(), wide.clone(), 0);
    assert_eq!(
        complete.summary.status(),
        "complete",
        "{:?}",
        complete.summary
    );
    let emitted_bytes = emitted(&complete);
    assert!(
        emitted_bytes > 1,
        "the scope really emits text: {emitted_bytes}"
    );
    assert_eq!(
        budget_stops(&complete_sink),
        0,
        "under a ceiling nothing reaches, no method stops for the account: {:?}",
        complete_sink.stops
    );
    let allowance = emitted_bytes / 2;

    for workers in [1_usize, 4] {
        let mut total = wide.clone();
        total.output_bytes = allowance;
        let (report, sink) = run(workers, total, wide.clone(), 0);

        assert!(
            budget_stops(&sink) > 0,
            "a total that funds half of the scope's own presentation really stops methods inside it \
             ({workers} worker(s)): {:?}",
            sink.stops
        );
        for stop in &sink.stops {
            assert!(
                matches!(
                    stop,
                    StopReason::Budget {
                        dimension: CountedBudgetDimension::OutputBytes,
                        ..
                    }
                ),
                "and the stop every one of them states names the dimension the operation shares with \
                 its consumer ({workers} worker(s)): {stop:?}"
            );
        }
        let spent = report
            .usage
            .counted_usage(CountedBudgetDimension::OutputBytes);
        assert!(
            spent <= allowance,
            "the account stayed inside the declaration ({workers} worker(s)): {spent} of {allowance}"
        );
        assert!(
            emitted(&report) <= allowance,
            "and the library's own part of it did too ({workers} worker(s))"
        );
        assert_eq!(
            report.summary.methods_delivered, report.summary.methods_declared,
            "the operation itself was not stopped: every declared method's record was delivered \
             ({workers} worker(s))"
        );
        assert!(
            !matches!(report.summary.execution, ExecutionReport::Complete { .. }),
            "a scope whose methods stopped for the account is not complete ({workers} worker(s)): {:?}",
            report.summary.execution
        );
    }
}

#[test]
fn the_terminal_record_pays_for_itself_before_the_call_returns() {
    // One run, two readings of its account: what the **streamed** `final` record states, and what the
    // call hands back. The terminal record is delivered like any other, so its own cost belongs to the
    // operation's total *before* the call returns — one delivered record for the library and this
    // consumer's bytes for the line it wrote. A total that left them out would claim a run spent less
    // than it did, and a quota it then reports as met would not be.
    let wide = bulk_support::limits();
    let cost = 64_u64;
    let (complete, sink) = run(1, wide.clone(), wide.clone(), cost);
    assert_eq!(
        complete.summary.status(),
        "complete",
        "{:?}",
        complete.summary
    );
    assert!(complete.final_delivered);
    let streamed = sink
        .final_summary
        .as_ref()
        .expect("the complete run delivered its terminal record");
    let streamed_usage = execution_usage(&streamed.execution);
    let returned = execution_usage(&complete.summary.execution);
    assert_eq!(
        returned.result_items,
        streamed_usage.result_items + 1,
        "the `final` record's own delivery is part of the total the call hands back"
    );
    assert_eq!(
        returned.output_bytes,
        streamed_usage.output_bytes + cost,
        "and so is what writing that line cost the consumer, charged to the same account"
    );
    assert_eq!(
        complete.usage, *returned,
        "the report's own reading is that total, not a second one"
    );

    // One byte short of what the whole run charged. Nothing is charged after the terminal record, so
    // the record the account refuses is that one: every method is executed and delivered, no `final`
    // is written, and the run is not complete. The declaration cannot be met by leaving the terminal
    // record's own cost out of it.
    let mut total = wide.clone();
    total.output_bytes = complete
        .usage
        .counted_usage(CountedBudgetDimension::OutputBytes)
        .saturating_sub(1);
    let (short, short_sink) = run(1, total, wide.clone(), cost);
    assert!(
        short_sink.refused.is_some(),
        "the account refuses the record after the funded ones"
    );
    assert_eq!(
        short_sink.written,
        sink.written.saturating_sub(cost),
        "the one record this run could not afford is the terminal one, and it was not written"
    );
    assert_eq!(
        short.summary.methods_delivered, complete.summary.methods_delivered,
        "every method record was still delivered: the refused record is the operation's own end"
    );
    assert!(
        !short.final_delivered,
        "a stream whose `final` was refused is a prefix"
    );
    assert!(
        matches!(
            short.summary.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::OutputBytes,
                },
                ..
            }
        ),
        "and it is not complete: {:?}",
        short.summary.execution
    );
    assert_eq!(
        short.stop,
        Some(BulkStop {
            owner: Some(UsageOwner::Delivery),
            kind: BulkStopKind::Budget,
            dimension: Some(BudgetDimension::OutputBytes),
        }),
        "the stop the short run states is the delivery account's, at the same dimension"
    );
}
