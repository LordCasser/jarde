//! Prints the task-oriented surface's honest answer for one standalone class file.
//!
//! The example shows six things, in the order a host would do them:
//!
//! 1. **Opening stays light**: `Engine::open` reads the artifact's own bytes and nothing else —
//!    no class header, no body, and no analysis plane moves — and the counters that say so are
//!    printed from the same budget the open ran under.
//! 2. **A bounded default budget with a few overrides**: `task_budget(&[…])` is the one place the
//!    default set lives. One `BudgetOverride` replaces one dimension and the rest stay bounded;
//!    a dimension outside `OVERRIDABLE_BUDGET_DIMENSIONS` or a zero limit is an input error.
//! 3. **One target selection**: `ClassRef::Name` (a dotted or internal name) binds one physical
//!    identity through the navigation rules; the class view publishes that identity, its members
//!    from one read, and the bodies it was asked for. The report's `limits`/`usage` are the
//!    effective configuration and what it was charged.
//! 4. **One class read, on-demand bodies**: the view charges one `class_headers` attempt and one
//!    `method_bodies` attempt per requested concrete method — never a header per method — and a
//!    member that declares no `Code` is stated as such, not given an empty body.
//! 5. **An explicit environment policy**: `EnvironmentPolicy::SingleClass` builds the declaration
//!    a caller would write by hand, the engine's own validator accepts it, and
//!    `Engine::recover_target` runs the P2 pipeline under the operation's own stage table and
//!    presents the same run's artifact content-first (`RecoveryPresentation::parts`).
//! 6. **References organised by owning method**: `ReferenceGrouping::from_query` keeps the scan's
//!    own planes and moves only its items, grouping body hits under their method while class-level
//!    and resource positions keep their own places and derivation classes.
//!
//! Usage: `cargo run --example task_operations [standalone.class]`; without an argument the
//! historical v52 fixture is used. The example deliberately does not select a member by name with
//! several declared descriptors: that is the ambiguity case, and `Engine::class_view` answers it
//! with `OperationOutcome::Ambiguous` instead of executing anything.

use jarde::{
    ArtifactInput, BudgetOverride, ClassNameQuery, ClassRef, ClassViewRequest, ConsumerKind,
    ConsumerSchema, Engine, EnvironmentPolicy, EnvironmentRequest, JvmBytes, MethodOperation,
    MethodOperationRequest, MethodRef, OperationOutcome, PhysicalScope, QueryRelation,
    QueryRequest, QueryTarget, RecoveryContent, RecoveryPresentationPart, ReferenceGrouping,
    SymbolRef, task_budget, task_limits, validate_environment,
};
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::slice;

fn run(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new();

    // (1) A light open: the artifact's own bytes are the whole read.
    let mut open_budget = task_budget(&[])?;
    let snapshot = engine.open(ArtifactInput::Path(path), &mut open_budget)?;
    println!(
        "opened {} ({} bytes): class_headers={} method_bodies={} code_bytes={} ir_items={}",
        snapshot.id().0,
        snapshot.len(),
        open_budget.usage().class_headers,
        open_budget.usage().method_bodies,
        open_budget.usage().code_bytes,
        open_budget.usage().ir_items,
    );

    // (2) The bounded default budget, with one explicit override.
    let overrides = [BudgetOverride::new("result_items", 4096)?];
    let effective = task_limits(&overrides)?;
    println!(
        "effective limits: result_items={} class_headers={} elapsed_millis={}",
        effective.result_items, effective.class_headers, effective.elapsed_millis
    );
    let mut budget = task_budget(&overrides)?;

    // (3) One target selection and the class view over it.
    let class_name = class_name_of(&engine, &snapshot)?;
    let mut bodies = Vec::new();
    let view_request = ClassViewRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name.clone()),
        },
        bodies: Vec::new(),
    };
    let view = match engine.class_view(
        &snapshot,
        &PhysicalScope::SnapshotAll,
        &view_request,
        &mut budget,
    )? {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => {
            println!(
                "the name bound {} definitions; the operation executed nothing",
                candidates.candidates.len()
            );
            return Ok(());
        }
        // A search that did not finish — a damaged candidate, an exhausted dimension, a
        // cancellation — is neither an election nor a missing name: the operation publishes the
        // search's own stop beside the candidates it did confirm and executes nothing.
        OperationOutcome::Incomplete(candidates) => {
            println!(
                "the name search did not finish ({} diagnostic(s), {} candidate(s) confirmed); the operation executed nothing",
                candidates.diagnostics.len(),
                candidates.candidates.len()
            );
            return Ok(());
        }
    };
    println!(
        "class view of `{}`: {} member(s), class_headers={} method_bodies={}, effective limits published={}",
        class_name,
        view.items.len(),
        view.usage.class_headers,
        view.usage.method_bodies,
        view.limits == effective
    );

    // (4) The on-demand bodies: one attempt each, and no empty body for a member without `Code`.
    let mut concrete = Vec::new();
    for method in view.methods() {
        if let Some(body) = view.body(&method.identity) {
            bodies.push(body.clone());
        }
        match method.body {
            jarde::MemberBodyEvidence::CodeAttribute { .. } => {
                concrete.push(method.identity.clone());
            }
            jarde::MemberBodyEvidence::NoCodeAttribute => {}
        }
    }
    let requested = concrete.first().cloned();
    if let Some(method) = requested.clone() {
        let mut budget = task_budget(&[])?;
        let report = match engine.class_view(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &ClassViewRequest {
                class: ClassRef::Definition {
                    definition: view.class.clone(),
                },
                bodies: vec![jarde::BodyRef::Method {
                    method: method.clone(),
                }],
            },
            &mut budget,
        )? {
            OperationOutcome::Performed(report) => report,
            OperationOutcome::Ambiguous(_) | OperationOutcome::Incomplete(_) => {
                unreachable!("an identity is not searched: nothing can be ambiguous or unfinished")
            }
        };
        println!(
            "one body of `{}`: class_headers={} method_bodies={}, results={}",
            String::from_utf8_lossy(&method.name.0),
            report.usage.class_headers,
            report.usage.method_bodies,
            report.bodies.len(),
        );
    }

    // (5) The explicit environment policy and the recovery of one named member.
    let environment = EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::SingleClass,
        profile: jarde::RuntimeProfile {
            java_release: 8,
            multi_release: jarde::MultiReleasePolicy::Disabled,
            layout: jarde::LayoutMode::Generic,
        },
        loader: jarde::LoaderId("example".to_string()),
    };
    let declaration = environment.build(slice::from_ref(&snapshot))?;
    let (problems, identity) = validate_environment(slice::from_ref(&snapshot), &declaration);
    println!(
        "single-class policy: loader={:?} roots={} problems={}",
        identity.domain_loaders,
        declaration.domains[0].roots.len(),
        problems.len()
    );

    if let Some(method) = requested {
        let mut budget = task_budget(&overrides)?;
        let recovery = match engine.recover_target(
            slice::from_ref(&snapshot),
            &MethodOperationRequest {
                method: MethodRef::Method {
                    method: method.clone(),
                },
                environment: environment.clone(),
            },
            &mut budget,
        )? {
            OperationOutcome::Performed(report) => report,
            OperationOutcome::Ambiguous(_) | OperationOutcome::Incomplete(_) => {
                unreachable!("an identity is not searched: nothing can be ambiguous or unfinished")
            }
        };
        let parts: Vec<&'static str> = recovery
            .presentation
            .parts()
            .iter()
            .map(|part| match part {
                RecoveryPresentationPart::Content { .. } => "content",
                RecoveryPresentationPart::Quality { .. } => "quality",
                RecoveryPresentationPart::Stop { .. } => "stop",
            })
            .collect();
        println!(
            "recovery of `{}`: operation={:?} stages={} presentation={:?} content={:?} stop={} text_bytes={}",
            String::from_utf8_lossy(&method.name.0),
            MethodOperation::Recovery,
            recovery.stages.len(),
            parts,
            recovery.presentation.content,
            recovery.presentation.stop.is_some(),
            recovery.recovered.recovery().text.len(),
        );
        if recovery.presentation.content == RecoveryContent::NotProduced {
            println!(
                "  the run stopped before delivering an artifact; its text and source map are empty"
            );
        }
    }

    // (6) One query's items, reorganised by owning method. The target is the constructor call
    // every class makes (`java/lang/Object.<init>()V`), so one report holds a body hit in the
    // constructor and the class-level superclass reference beside it.
    let mut query_budget = task_budget(&overrides)?;
    let report = engine.query(
        &snapshot,
        &QueryRequest {
            relation: QueryRelation::MentionsSymbol,
            target: QueryTarget::Symbol {
                value: SymbolRef::Method {
                    owner: JvmBytes(b"java/lang/Object".to_vec()),
                    name: JvmBytes(b"<init>".to_vec()),
                    descriptor: JvmBytes(b"()V".to_vec()),
                },
            },
            physical: jarde::PhysicalView {
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
                    ConsumerKind::Exception,
                    ConsumerKind::Signature,
                    ConsumerKind::Annotation,
                    ConsumerKind::InnerNest,
                    ConsumerKind::Module,
                    ConsumerKind::Bootstrap,
                    ConsumerKind::Resource,
                ],
            ),
            max_items: 0,
            cursor: None,
        },
        &mut query_budget,
    )?;
    let scan = report.clone();
    let grouping = ReferenceGrouping::from_query(report);
    let candidates = scan
        .items
        .iter()
        .filter(|item| item.derivation == jarde::XrefDerivation::ConstantPoolCandidate)
        .count();
    println!(
        "references: {} hit(s) in {} method group(s), {} class-level, {} resource, {} constant-pool candidate(s)",
        grouping.finding_count(),
        grouping.methods.len(),
        grouping.class_level.len(),
        grouping.resources.len(),
        candidates,
    );
    for group in &grouping.methods {
        println!(
            "  owned by `{}`: {} hit(s), first at BCI {:?} as {:?}",
            String::from_utf8_lossy(&group.method.name.0),
            group.findings.len(),
            group.findings[0].bci(),
            group.findings[0].class(),
        );
    }
    Ok(())
}

/// The internal name the class's own header states, from the class view's own read.
fn class_name_of(
    engine: &Engine,
    snapshot: &jarde::ArtifactSnapshot,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut budget = task_budget(&[])?;
    let report =
        engine.list_class_declarations(snapshot, &PhysicalScope::SnapshotAll, &mut budget)?;
    let Some(item) = report.items.first() else {
        return Err("the fixture declares no class".into());
    };
    Ok(String::from_utf8_lossy(&item.declaration.this_class.raw().0).into_owned())
}

fn main() -> ExitCode {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let path = match args.next() {
        Some(path) => PathBuf::from(path),
        None => Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
    };
    if args.next().is_some() {
        eprintln!(
            "usage: {} [standalone.class]",
            PathBuf::from(program).display()
        );
        return ExitCode::from(2);
    }
    match run(path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("task_operations: {error}");
            ExitCode::FAILURE
        }
    }
}
