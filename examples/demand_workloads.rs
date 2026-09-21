//! One measured workload per invocation, for the protocol fixed in
//! `openspec/changes/add-demand-driven-core-results/verification.md` §7.3.
//!
//! ```text
//! cargo run --release --example demand_workloads -- <artifact> <roots.json> <workload> <arm> [repeats]
//! ```
//!
//! `workload` is one of `nav-class`, `nav-decl`, `recover-one`, `sweep`, `expand`, `page-small`,
//! `page-abandon`; `arm` is `essential` or `all`. Every run prints one JSON line with what the
//! protocol asks for — the time until the workload's first result, the whole sequence, the bytes the
//! run returned and the operation's own counters — and nothing here asserts a duration. The caller
//! interleaves the arms and reads the samples; the example never selects the good ones.
//!
//! `sweep` measures the first *delivery* rather than the first byte of a document: the sink records
//! the instant the first `method` record reaches it, which is the first result a streaming host
//! would see. `expand` runs the essential recovery, keeps the artifact binding it published, and
//! then asks for the same artifact with every category selected under a fresh budget — the two
//! phases are timed separately because the protocol asks for the follow-up's own cost.

use jarde::{
    ArtifactInput, ArtifactSnapshot, Budget, BudgetOverride, BulkDiagnosticEvent, BulkFinalEvent,
    BulkHeaderEvent, BulkRecoveryRequest, ClassEndEvent, ClassNameQuery, ClassPreparedEvent,
    ClassRef, ClassViewRequest, DeliveryAccount, Engine, EnvironmentPolicy, EnvironmentRequest,
    LoadRoot, MemberBodyEvidence, MethodOperationRequest, MethodRef, MethodResultEvent,
    MultiReleasePolicy, OperationOutcome, PhysicalScope, PhysicalView, QueryRelation, QueryRequest,
    QueryTarget, RecoveryEvidenceRequest, RecoveryPresentationPart, RecoverySink, Result,
    RuntimeProfile, SinkControl, SymbolRef, task_limits,
};
use std::path::PathBuf;
use std::slice;
use std::time::Instant;

fn limits() -> Result<jarde::Limits> {
    // The counted dimensions an operation over a whole package needs, plus an hour of wall clock:
    // this is a measurement harness, so the point is to finish rather than to stop.
    let overrides: Vec<BudgetOverride> = [
        "input_bytes",
        "archive_entries",
        "entry_bytes",
        "read_bytes",
        "class_bytes",
        "attribute_bytes",
        "code_bytes",
        "result_items",
        "output_bytes",
        "class_headers",
        "method_bodies",
        "ir_items",
        "ir_edges",
        "analysis_steps",
        "normalization_clones",
    ]
    .into_iter()
    .map(|dimension| BudgetOverride::new(dimension, 1 << 40))
    .chain([BudgetOverride::new("elapsed_millis", 3_600_000)])
    .collect::<Result<Vec<_>>>()?;
    task_limits(&overrides)
}

fn declared_roots(path: &str) -> Vec<LoadRoot> {
    serde_json::from_str(&std::fs::read_to_string(path).expect("the roots document is readable"))
        .expect("the roots document parses")
}

/// The sink `sweep` uses: it counts what it was handed and records when the first method record
/// arrived, which is the first result this workload is about.
#[derive(Default)]
struct FirstDelivery {
    first_method: Option<Instant>,
    methods: u64,
    records: u64,
}

impl RecoverySink for FirstDelivery {
    fn header(
        &mut self,
        _event: &BulkHeaderEvent,
        _delivery: DeliveryAccount,
    ) -> Result<SinkControl> {
        self.records += 1;
        Ok(SinkControl::Continue)
    }
    fn class_prepared(&mut self, _event: &ClassPreparedEvent) -> Result<SinkControl> {
        self.records += 1;
        Ok(SinkControl::Continue)
    }
    fn method(&mut self, _event: &MethodResultEvent) -> Result<SinkControl> {
        self.records += 1;
        self.methods += 1;
        if self.first_method.is_none() {
            self.first_method = Some(Instant::now());
        }
        Ok(SinkControl::Continue)
    }
    fn class_end(&mut self, _event: &ClassEndEvent) -> Result<SinkControl> {
        self.records += 1;
        Ok(SinkControl::Continue)
    }
    fn diagnostic(&mut self, _event: &BulkDiagnosticEvent) -> Result<SinkControl> {
        self.records += 1;
        Ok(SinkControl::Continue)
    }
    fn final_event(&mut self, _event: &BulkFinalEvent) -> Result<SinkControl> {
        self.records += 1;
        Ok(SinkControl::Continue)
    }
}

fn environment_request(snapshot: &ArtifactSnapshot, roots: Vec<LoadRoot>) -> EnvironmentRequest {
    EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::ExplicitClasspath { roots },
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: jarde::LayoutMode::Generic,
        },
        loader: jarde::LoaderId("app".to_owned()),
    }
}

fn performed<T>(outcome: OperationOutcome<T>, what: &str) -> T {
    match outcome {
        OperationOutcome::Performed(value) => value,
        // An identity is not searched, so the other two outcomes cannot happen here; the harness
        // states that rather than pretending to handle them.
        OperationOutcome::Ambiguous(_) => {
            panic!("{what} was reported ambiguous, and an identity cannot be")
        }
        OperationOutcome::Incomplete(_) => {
            panic!("{what} was reported incomplete, and an identity cannot be")
        }
    }
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("demand_workloads: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut arguments = std::env::args().skip(1);
    let artifact = arguments.next().expect("an artifact path");
    let roots_path = arguments.next().expect("a roots document");
    let workload = arguments.next().expect("a workload name");
    let arm = arguments.next().expect("an arm: essential or all");
    let repeats: usize = arguments
        .next()
        .and_then(|text| text.parse().ok())
        .unwrap_or(1);
    let all = arm == "all";
    let selection = || {
        if all {
            RecoveryEvidenceRequest::all()
        } else {
            RecoveryEvidenceRequest::essential()
        }
    };

    let engine = Engine::new();
    let roots = declared_roots(&roots_path);
    let mut opened = Budget::new(limits()?);
    let snapshot = engine.open(ArtifactInput::Path(PathBuf::from(&artifact)), &mut opened)?;

    // The class and member every workload works on: the first candidate the walk finds, so the
    // measurement is not confounded by the caller's choice of a favourable sample.
    let mut discovery = Budget::new(limits()?);
    let listing =
        engine.list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut discovery)?;
    let class_name = listing
        .items
        .first()
        .map(|item| item.declaration.this_class.raw().clone())
        .expect("the artifact declares at least one class");
    let view = performed(
        engine.class_view(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &ClassViewRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal(
                        String::from_utf8_lossy(&class_name.0).into_owned(),
                    ),
                },
                bodies: Vec::new(),
            },
            &mut discovery,
        )?,
        "the first class candidate",
    );
    let member = view
        .methods()
        .find(|method| matches!(method.body, MemberBodyEvidence::CodeAttribute { .. }))
        .expect("the class declares at least one member with a body")
        .identity
        .clone();

    for repeat in 0..repeats {
        let started = Instant::now();
        let mut first = None;
        let mut returned_bytes = 0_u64;
        let (class_headers, method_bodies, ir_items, code_bytes, detail) = match workload.as_str() {
            "nav-class" => {
                let mut budget = Budget::new(limits()?);
                let report = performed(
                    engine.class_view(
                        &snapshot,
                        &PhysicalScope::SnapshotAll,
                        &ClassViewRequest {
                            class: ClassRef::Definition {
                                definition: view.class.clone(),
                            },
                            bodies: Vec::new(),
                        },
                        &mut budget,
                    )?,
                    "a definition",
                );
                first = Some(started.elapsed());
                returned_bytes = serde_json::to_vec(&report).map_or(0, |bytes| bytes.len() as u64);
                (
                    report.usage.class_headers,
                    report.usage.method_bodies,
                    report.usage.ir_items,
                    report.usage.code_bytes,
                    format!(
                        "members={} bodies={}",
                        report.items.len(),
                        report.bodies.len()
                    ),
                )
            }
            "nav-decl" => {
                let mut budget = Budget::new(limits()?);
                let report = performed(
                    engine.class_view(
                        &snapshot,
                        &PhysicalScope::SnapshotAll,
                        &ClassViewRequest {
                            class: ClassRef::Definition {
                                definition: view.class.clone(),
                            },
                            bodies: vec![jarde::BodyRef::Method {
                                method: member.clone(),
                            }],
                        },
                        &mut budget,
                    )?,
                    "a definition",
                );
                first = Some(started.elapsed());
                returned_bytes = serde_json::to_vec(&report).map_or(0, |bytes| bytes.len() as u64);
                (
                    report.usage.class_headers,
                    report.usage.method_bodies,
                    report.usage.ir_items,
                    report.usage.code_bytes,
                    format!(
                        "members={} bodies={}",
                        report.items.len(),
                        report.bodies.len()
                    ),
                )
            }
            "recover-one" | "expand" => {
                let request = MethodOperationRequest {
                    method: MethodRef::Method {
                        method: member.clone(),
                    },
                    environment: environment_request(&snapshot, roots.clone()),
                };
                let mut budget = Budget::new(limits()?);
                let report = performed(
                    engine.recover_target_with_evidence(
                        slice::from_ref(&snapshot),
                        &request,
                        &selection(),
                        &mut budget,
                    )?,
                    "a member identity",
                );
                first = Some(started.elapsed());
                let recovery = report.recovered.recovery();
                returned_bytes = serde_json::to_vec(&report).map_or(0, |bytes| bytes.len() as u64);
                let parts: Vec<&str> = report
                    .presentation
                    .parts()
                    .iter()
                    .map(|part| match part {
                        RecoveryPresentationPart::Content { .. } => "content",
                        RecoveryPresentationPart::Quality { .. } => "quality",
                        RecoveryPresentationPart::Stop { .. } => "stop",
                    })
                    .collect();
                let mut detail = format!(
                    "text={} regions={} map={} presentation={parts:?}",
                    recovery.text.len(),
                    recovery.regions.len(),
                    recovery.source_map.len(),
                );
                if workload == "expand" {
                    // A later request states the artifact the first run committed, under its own
                    // budget, and the run compares what it commits against it.
                    if let Some(binding) = recovery.artifact.binding() {
                        let mut follow_up = Budget::new(limits()?);
                        let expanded = performed(
                            engine.recover_target_with_evidence(
                                slice::from_ref(&snapshot),
                                &request,
                                &RecoveryEvidenceRequest::all()
                                    .with_expected_artifact(binding.clone()),
                                &mut follow_up,
                            )?,
                            "a member identity",
                        );
                        let expanded_report = expanded.recovered.recovery();
                        returned_bytes =
                            serde_json::to_vec(&expanded).map_or(0, |bytes| bytes.len() as u64);
                        detail = format!(
                            "{detail} agreement={:?} follow_up_bytes={} follow_up_headers={}",
                            expanded_report.artifact.agreement(),
                            expanded_report.text.len(),
                            follow_up.usage().class_headers,
                        );
                    }
                }
                (
                    report.usage.class_headers,
                    report.usage.method_bodies,
                    report.usage.ir_items,
                    report.usage.code_bytes,
                    detail,
                )
            }
            "sweep" => {
                let mut budget = Budget::new(limits()?);
                let mut sink = FirstDelivery::default();
                let workers = std::thread::available_parallelism()
                    .map(|value| value.get())
                    .unwrap_or(1);
                let report = engine.recover_all(
                    slice::from_ref(&snapshot),
                    &BulkRecoveryRequest::for_scope(
                        environment_request(&snapshot, roots.clone()),
                        workers,
                        limits()?,
                    )
                    .with_evidence(selection()),
                    &mut budget,
                    &mut sink,
                )?;
                first = sink
                    .first_method
                    .map(|at| at.saturating_duration_since(started));
                returned_bytes = sink.methods;
                (
                    budget.usage().class_headers,
                    budget.usage().method_bodies,
                    budget.usage().ir_items,
                    budget.usage().code_bytes,
                    format!(
                        "records={} methods={} status={:?} workers={workers}",
                        sink.records,
                        sink.methods,
                        report.summary.status(),
                    ),
                )
            }
            "page-small" | "page-abandon" => {
                let pages = if workload == "page-abandon" { 2 } else { 1 };
                let mut budget = Budget::new(limits()?);
                let mut cursor: Option<jarde::QueryCursor> = None;
                let mut items = 0_u64;
                let mut last = None;
                for page in 0..pages {
                    let report = engine.query(
                        &snapshot,
                        &QueryRequest {
                            relation: QueryRelation::ConstantPoolContains,
                            target: QueryTarget::Symbol {
                                value: SymbolRef::Class {
                                    owner: jarde::JvmBytes(b"java/lang/Object".to_vec()),
                                },
                            },
                            physical: PhysicalView {
                                snapshot: snapshot.id().clone(),
                                scope: PhysicalScope::SnapshotAll,
                            },
                            consumers: jarde::ConsumerSchema::new(
                                1,
                                vec![jarde::ConsumerKind::Type],
                            ),
                            max_items: 4,
                            cursor: cursor.clone(),
                        },
                        &mut budget,
                    )?;
                    if page == 0 {
                        first = Some(started.elapsed());
                    }
                    items += report.items.len() as u64;
                    returned_bytes =
                        serde_json::to_vec(&report).map_or(0, |bytes| bytes.len() as u64);
                    cursor = report.page.cursor.clone();
                    let more = report.page.has_more;
                    let usage = budget.usage();
                    last = Some((
                        usage.class_headers,
                        usage.method_bodies,
                        usage.ir_items,
                        usage.code_bytes,
                        format!(
                            "pages={} items={items} scanned_items={} has_more={more}",
                            page + 1,
                            report.coverage.scanned_items,
                        ),
                    ));
                    if cursor.is_none() {
                        break;
                    }
                }
                let (ch, mb, ir, cb, detail) = last.expect("at least one page");
                (ch, mb, ir, cb, detail)
            }
            other => {
                return Err(jarde::Error::invalid_input(
                    "example_workload",
                    format!("unknown workload `{other}`"),
                ));
            }
        };
        let total = started.elapsed();
        println!(
            "{{\"workload\":\"{workload}\",\"arm\":\"{arm}\",\"run\":{repeat},\
             \"first_micros\":{},\"total_micros\":{},\"returned_bytes\":{returned_bytes},\
             \"class_headers\":{class_headers},\"method_bodies\":{method_bodies},\
             \"ir_items\":{ir_items},\"code_bytes\":{code_bytes},\"detail\":\"{detail}\"}}",
            first.map_or(0, |value| value.as_micros()),
            total.as_micros(),
        );
    }
    Ok(())
}
