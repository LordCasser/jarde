//! The analysis driver: the one module that calls the resolver, the environment and the IR.
//!
//! Everything a P2 request does beyond the facade's argument handling lives here: the
//! resolver's request checks and report assembly, the environment validation that decides
//! whether a request starts at all, and the method-analysis driver that schedules the pass
//! table, reads one driver method through the reader, and turns every pass outcome into the
//! report's stage, coverage, read and execution planes.
//!
//! The four entry points below are what the facade delegates to, one line each; the
//! composition story — which entry runs what, and what a caller observes — is documented on
//! `jarde::Engine`, which is the surface a consumer names.

use jarde_reader::artifact::ArtifactSnapshot;
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{
    BootstrapMethodFacts, BytecodeStop, CpEntryFacts, MemberHeader, MethodCodeFacts,
    VersionCapability, version_rule_diagnostic,
};
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    Coverage, Diagnostic, DiagnosticSeverity, ExecutionReport, TerminationReason,
};

use crate::environment::{EnvironmentIdentity, EnvironmentProblem};
use crate::frame::{FrameMethod, FrameOutcome, IR_FRAME_DEFERRED, IR_FRAME_INCONSISTENT};
use crate::ir::{
    MethodAnalysisReport, MethodAnalysisRequest, MethodBodyState, NoBodyKind, StageResult,
    StageState,
};
use crate::method_ir::{MethodDeclaration, MethodIr, MethodIrAnalysis};
use crate::passes::{FactLedger, IR_PASS_NOT_IMPLEMENTED, IrPhase, PassDescriptor, implemented};
use crate::ssa::{IR_SSA_INCONSISTENT, SsaOutcome};

/// Access flags that declare a member without a body: `ACC_ABSTRACT` and `ACC_NATIVE`.
const ACC_ABSTRACT: u16 = 0x0400;
const ACC_NATIVE: u16 = 0x0100;

/// Code of a raw CFG that was not built because the body decode stopped before its end.
const IR_RAW_CFG_INCOMPLETE_BODY: &str = "ir_raw_cfg_incomplete_body";

/// Demand-bound symbol resolution under an explicit environment (P2 entry point).
///
/// The request shape is checked first, then the resolution itself runs under the caller's
/// budget; [`crate::resolver::ResolutionReport`] carries the outcome. The facade
/// (`jarde::Engine::resolve_symbol`) documents the full contract a consumer observes.
pub fn resolve_symbol(
    content: &[ArtifactSnapshot],
    request: &crate::resolver::ResolutionRequest,
    budget: &mut Budget,
) -> Result<crate::resolver::ResolutionReport> {
    crate::resolver::validate_request(content, request)?;
    Ok(crate::resolver::resolution_report(content, request, budget))
}

/// Declaration-reference scan under an explicit environment (P2 entry point).
///
/// Same request-level check as [`resolve_symbol`]; the scan itself is the resolver's, and
/// candidates no search could decide are reported as unresolved rather than excluded.
pub fn declaration_references(
    content: &[ArtifactSnapshot],
    query: &crate::resolver::DeclarationRefQuery,
    budget: &mut Budget,
) -> Result<crate::resolver::DeclarationRefReport> {
    crate::resolver::validate_declaration_reference_query(content, query)?;
    crate::resolver::declaration_reference_report(content, query, budget)
}

/// Bounded reflection and `ServiceLoader` pattern scan under an explicit environment (P4 2.3
/// entry point).
///
/// The request shape is checked first, exactly like [`resolve_symbol`]'s. The scan itself is
/// [`crate::reflection`]'s: it enumerates the classes of the explicit scope, analyses the bodies
/// that name a registered overload and answers one site per call site, and the facade
/// (`jarde::Engine::reflection_patterns`) documents the contract a consumer observes.
pub fn reflection_patterns(
    content: &[ArtifactSnapshot],
    request: &crate::reflection::ReflectionPatternRequest,
    budget: &mut Budget,
) -> Result<crate::reflection::ReflectionPatternReport> {
    crate::reflection::validate_request(content, request)?;
    crate::reflection::reflection_patterns(content, request, budget)
}

/// Method IR analysis under an explicit environment (P2 entry point).
///
/// The request and the requested stages are validated against the fixed pass table before
/// anything runs, and the declared environment is checked before a read is attempted: a
/// rejected environment never starts the pipeline, and a schedule the table cannot serve is an
/// input error rather than a half-initialized run. What the scheduled passes then do, and what
/// each stop keeps, is [`run_method_analysis`]'s contract; the report is assembled from the
/// validated request and the run.
///
/// This entry answers with the report alone and drops the IR payload of the run;
/// [`analyze_method_ir`] performs the very same run and hands both over.
pub fn analyze_method(
    content: &[ArtifactSnapshot],
    request: &MethodAnalysisRequest,
    budget: &mut Budget,
) -> Result<MethodAnalysisReport> {
    let analyzed = run_request(content, request, budget)?;
    Ok(crate::ir::analysis_report(
        request,
        analyzed.problems,
        analyzed.environment_identity,
        analyzed.run,
    ))
}

/// The same analysis, handing over the IR payload of the same run (P3 1.1 entry point).
///
/// One request is one run: this entry performs exactly what [`analyze_method`] performs — the same
/// request and schedule validation, the same environment check, the same passes under the same
/// budget, the same stop and the same report — and returns the tables those passes published
/// beside it, so a recovery consumer does not run the pipeline a second time to get them.
///
/// The payload is owned by the caller for as long as it keeps the returned value, and reads
/// through it are ordinary borrows ([`crate::method_ir`]); the report states the phases, the
/// quality, the coverage, the execution and the diagnostics of that one run, and a stopped or
/// refused run hands over exactly what it published, which is nothing at all for a request whose
/// environment was rejected.
pub fn analyze_method_ir(
    content: &[ArtifactSnapshot],
    request: &MethodAnalysisRequest,
    budget: &mut Budget,
) -> Result<MethodIrAnalysis> {
    let analyzed = run_request(content, request, budget)?;
    Ok(MethodIrAnalysis::new(
        crate::ir::analysis_report(
            request,
            analyzed.problems,
            analyzed.environment_identity,
            analyzed.run,
        ),
        analyzed.ir,
    ))
}

/// One validated request and the run it performed: everything the report and the payload need.
///
/// The run is constructed in one place because one request gets one run: the two entry points
/// above differ only in what they hand back from this, never in what was performed, charged or
/// stopped.
struct Analyzed {
    problems: Vec<EnvironmentProblem>,
    environment_identity: EnvironmentIdentity,
    run: crate::ir::AnalysisRun,
    ir: MethodIr,
}

/// Validates one method-analysis request and runs it.
fn run_request(
    content: &[ArtifactSnapshot],
    request: &MethodAnalysisRequest,
    budget: &mut Budget,
) -> Result<Analyzed> {
    crate::ir::validate_request(content, request)?;
    let scheduled = crate::passes::validate_requested_stages(&request.stages)?;
    let (problems, environment_identity) =
        crate::environment::validate_environment(content, &request.environment);
    let (run, ir) = if problems.is_empty() {
        run_method_analysis(content, request, scheduled, budget)
    } else {
        // A rejected environment never yields a definition and never starts a read, so the
        // pipeline is not run at all; the report names the problems and the capability that did
        // not run, and no pass published a table for the payload to hold.
        (
            crate::ir::AnalysisRun::not_performed(
                &scheduled
                    .iter()
                    .map(|pass| pass.phase.stage())
                    .collect::<Vec<_>>(),
                crate::ir::METHOD_ANALYSIS_NOT_IMPLEMENTED,
                budget,
            ),
            MethodIr::new(None, None, None, None, Vec::new(), Vec::new(), None),
        )
    };
    Ok(Analyzed {
        problems,
        environment_identity,
        run,
        ir,
    })
}

/// Reports a scheduled pass this build does not implement: its stage fails under
/// `ir_pass_not_implemented`, the phases behind it stay `NotPerformed`, and the run terminates
/// as the unsupported capability it is.
fn report_unimplemented(
    run: &mut crate::ir::AnalysisRun,
    index: usize,
    phase: IrPhase,
    budget: &Budget,
) -> ExecutionReport {
    run.stages[index].state = StageState::Failed {
        code: IR_PASS_NOT_IMPLEMENTED.to_string(),
    };
    run.diagnostics.push(Diagnostic {
        code: IR_PASS_NOT_IMPLEMENTED.to_string(),
        severity: DiagnosticSeverity::Error,
        message: format!(
            "the phase `{}` is not implemented in this engine build: the phases before it ran, \
             and this request has no result for it or for the phases behind it",
            phase.code()
        ),
        provenance: None,
    });
    ExecutionReport::Failed {
        reason: TerminationReason::Unsupported {
            code: IR_PASS_NOT_IMPLEMENTED.to_string(),
        },
        usage: budget.usage(),
    }
}

/// What the scheduled passes of one method-analysis request produced: the run, and the read-only
/// payload of the tables it published ([`MethodIr`]).
///
/// The ledger is the single accounting path (3.2): every pass publishes its facts through
/// [`FactLedger::apply`], and it is applied only after the pass completed, so a stopped run
/// keeps exactly the facts of its last valid phase and `last_completed` stays the highest
/// *completed* phase. The first stop governs the run's termination; the stage results name
/// every pass the run reached, and the passes behind a stop stay `NotPerformed`.
///
/// What "raw" means for the implemented passes is 3.3's contract: `raw_facts` projects the
/// reader's own facts (1.2) and `raw_cfg` builds the blocks, edges, instruction-level throw
/// sites, handler facts and effect facts of the decoded prefix, marking a body whose decode
/// stopped early as `Partial` instead of pretending the graph covers the whole method.
/// `legacy_normalization` is 3.4's contract over that graph: one call context per `jsr` site,
/// the return point of every `ret` from the context that owns it, the exception records that
/// cross a call, and a reported refusal — dialect violation or unestablished call graph —
/// instead of an invented one. `canonical_cfg` is 3.5's contract over exactly those contexts:
/// the bounded clone normalization, which publishes a canonical graph, stops under
/// `ir_legacy_normalization_unbounded` when its own bound is reached, and is the artifact that
/// makes the report's quality plane `Conservative`. `frame` is 4.1–4.2's contract over that graph:
/// the descriptor-driven slot state of every block the entry reaches, with the initialization
/// conversions of a constructor call applied to every alias of the token it was given, which stops
/// under `ir_frame_deferred` where an uninitialized value stands in a place that would need a
/// conversion this build does not define and under `ir_frame_inconsistent` where the bytes
/// contradict themselves.
fn run_method_analysis(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    scheduled: &[PassDescriptor],
    budget: &mut Budget,
) -> (crate::ir::AnalysisRun, MethodIr) {
    let mut run = crate::ir::AnalysisRun {
        body: MethodBodyState::NotInspected,
        stages: scheduled
            .iter()
            .map(|pass| StageResult {
                stage: pass.phase.stage(),
                state: StageState::NotPerformed,
            })
            .collect(),
        reads: Vec::new(),
        coverage: Coverage::not_requested(),
        quality: crate::ir::Quality::Fallback,
        execution: ExecutionReport::Complete {
            usage: budget.usage(),
        },
        diagnostics: Vec::new(),
    };
    let mut ledger = FactLedger::new();
    let mut stop: Option<ExecutionReport> = None;
    let mut facts: Option<MethodCodeFacts> = None;
    // The class file's version, as the reader classifies it, and the raw graph of this run:
    // `legacy_normalization` reads both, and both are facts of the passes that already completed.
    // The classification travels with the read instead of the two version fields coming back
    // together, so the rules that decide a dialect are read from one authority — the reader's own
    // (`jarde_reader::classfile::version_capability`) — rather than restated per pass.
    let mut version: Option<VersionCapability> = None;
    let mut raw: Option<crate::cfg::RawCfgOutcome> = None;
    // The call contexts 3.4b established, kept for `canonical_cfg` (3.5): the payload stays in
    // this run, so the cloning pass consumes the proven contexts instead of re-deriving them.
    let mut contexts: Option<crate::call_context::CallContexts> = None;
    // Whether this run published a canonical graph: the one artifact that makes the report's
    // quality `Conservative` instead of `Fallback`.
    let mut canonical_cfg: Option<Box<crate::canonical::CanonicalCfg>> = None;
    // The declaration facts the `frame` pass reads beside the body and the graph — the member's
    // flags, the class file's own name and its constant pool — which are part of the one header
    // read `raw_facts` performed.
    let mut declaration: Option<FrameDeclaration> = None;
    // What the located member's own declaration says (P3 3.1): the flags, the raw name and
    // descriptor, the parameter slots they imply and the identity the read was bound to. It is read
    // off the same header read `raw_facts` performs — the member it located — and travels in the
    // payload, so a consumer above this crate reads the class file's own statement instead of
    // assuming anything about the member or reading the class a second time.
    let mut member: Option<Box<MethodDeclaration>> = None;
    // The frames 4.1 published. 4.2 is their first consumer, so the payload stays in this run
    // under the same plan the canonical graph is kept under — and it is what the run hands over
    // to the recovery layer (P3 1.1).
    let mut frame_table: Option<Box<crate::frame::FrameTable>> = None;
    // The names 4.3 published, over exactly those frames and the canonical graph: the artifact the
    // next slice consumes, kept in this run for the same reason and handed over with it.
    let mut ssa_table: Option<Box<crate::ssa::SsaTable>> = None;
    for (index, pass) in scheduled.iter().enumerate() {
        if !implemented(pass.phase) {
            stop = stop.or(Some(report_unimplemented(
                &mut run, index, pass.phase, budget,
            )));
            break;
        }
        match pass.phase {
            IrPhase::RawFacts => {
                let decoded = match read_driver_method(content, request, &mut run, budget) {
                    Ok(DriverRead::Decoded {
                        facts,
                        version: read_version,
                        declaration: read_declaration,
                        member: read_member,
                    }) => {
                        version = Some(read_version);
                        declaration = Some(read_declaration);
                        member = read_member;
                        *facts
                    }
                    Ok(DriverRead::DeclaredWithoutBody) => {
                        // The member declares no body: no pass can run, and the request is
                        // complete as far as its input allows. Every scheduled stage stays
                        // `NotPerformed`, which is what "no phase ran" means, and the body fact
                        // says why.
                        break;
                    }
                    Err(error) => {
                        let (execution, diagnostic) = crate::ir::terminal(&error, budget.usage());
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                        break;
                    }
                };
                // The reader's own stop is a fact of this pass: the facts it returned are the
                // reliable prefix, so the stage is `Partial` and the request keeps the
                // reader's reason instead of a second, invented one.
                if let Some((execution, diagnostic)) = reader_stop(&decoded, budget) {
                    run.stages[index].state = StageState::Partial;
                    run.diagnostics.push(diagnostic);
                    stop = stop.or(Some(execution));
                } else {
                    run.stages[index].state = StageState::Completed;
                }
                if let Err(error) = ledger.apply(pass) {
                    // Unreachable while the table is the one the schedule was validated
                    // against; a ledger refusal would still be a stop, not a silent publish.
                    let (execution, diagnostic) = crate::ir::terminal(&error, budget.usage());
                    run.stages[index].state = stage_state(&execution);
                    run.diagnostics.push(diagnostic);
                    stop = stop.or(Some(execution));
                    break;
                }
                let cancelled = matches!(decoded.execution, ExecutionReport::Cancelled { .. });
                facts = Some(decoded);
                if cancelled {
                    // A cancelled request performs no further pass; a budget or decode stop
                    // over a reliable prefix does, because the graph of that prefix is still a
                    // sound part of the answer.
                    break;
                }
            }
            IrPhase::RawCfg => {
                let Some(decoded) = facts.as_ref() else {
                    // This pass requires what `raw_facts` publishes, so a run that reached it
                    // without those facts is the ledger's own `ir_pass_prerequisite_missing`:
                    // the same single accounting path decides it, instead of a second copy of
                    // the check here (the schedule validation makes it unreachable).
                    if let Err(error) = ledger.apply(pass) {
                        let (execution, diagnostic) = crate::ir::terminal(&error, budget.usage());
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                    }
                    break;
                };
                match crate::cfg::raw_cfg(decoded, budget) {
                    Ok(outcome) => {
                        if let Err(error) = ledger.apply(pass) {
                            let (execution, diagnostic) =
                                crate::ir::terminal(&error, budget.usage());
                            run.stages[index].state = stage_state(&execution);
                            run.diagnostics.push(diagnostic);
                            stop = stop.or(Some(execution));
                            break;
                        }
                        run.stages[index].state = if outcome.cfg.completeness.is_complete() {
                            StageState::Completed
                        } else {
                            StageState::Partial
                        };
                        // The graph and the effect facts are the raw pass's own payload: 3.4/3.5
                        // consume them, the report keeps publishing their status planes, and they
                        // stay out of the handoff of P3 1.1, which publishes the canonical tables of
                        // this run instead of this raw one. The payload stays in this run because
                        // the next pass reads it.
                        raw = Some(outcome);
                    }
                    Err(error) => {
                        let (execution, diagnostic) = raw_cfg_failure(&error, decoded, budget);
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                        break;
                    }
                }
            }
            IrPhase::LegacyNormalization => {
                let (Some(decoded), Some(raw), Some(version)) =
                    (facts.as_ref(), raw.as_ref(), version.as_ref())
                else {
                    // The pass requires the facts of the two passes before it, so a run that
                    // reached it without them is the ledger's own `ir_pass_prerequisite_missing`
                    // (the schedule validation makes it unreachable).
                    if let Err(error) = ledger.apply(pass) {
                        let (execution, diagnostic) = crate::ir::terminal(&error, budget.usage());
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                    }
                    break;
                };
                // The format's own version rule comes first, because this is the phase that reads
                // the class file's version at all: a major below the minimum the format defines,
                // or a modern minor that is neither 0 nor 65535, is not a dialect to normalize but
                // a version the format has no class file for. The verdict and its wording are the
                // reader's own — the same code, message and `Error` severity the header plan
                // publishes for these bytes — and the run keeps what the phases before this one
                // produced: the raw facts and the raw graph of the body stay published, nothing
                // canonical is built over them, and the reason travels with them.
                //
                // Whether such a body is *readable* is a different question and stays a different
                // plane: the decode of the body and its coverage are the `raw_facts` facts above,
                // and `verification` stays `NotPerformed` whatever this refusal says about the
                // version.
                if let Some(refusal) = version_rule_diagnostic(version) {
                    let code = refusal.code.clone();
                    run.stages[index].state = StageState::Failed { code: code.clone() };
                    run.diagnostics.push(Diagnostic {
                        code: code.clone(),
                        severity: refusal.severity,
                        message: format!(
                            "{}: the raw facts and the raw graph of this body are kept, and the \
                             method builds no call contexts",
                            refusal.message
                        ),
                        provenance: None,
                    });
                    stop = stop.or(Some(ExecutionReport::Failed {
                        reason: TerminationReason::Error { code },
                        usage: budget.usage(),
                    }));
                    break;
                }
                match crate::call_context::call_contexts(
                    decoded,
                    raw,
                    version.version.major,
                    budget,
                ) {
                    Ok(crate::call_context::CallContextOutcome::Established(established)) => {
                        if let Err(error) = ledger.apply(pass) {
                            let (execution, diagnostic) =
                                crate::ir::terminal(&error, budget.usage());
                            run.stages[index].state = stage_state(&execution);
                            run.diagnostics.push(diagnostic);
                            stop = stop.or(Some(execution));
                            break;
                        }
                        // The fact covers the decoded prefix: on a body whose decode stopped
                        // early the stage is `Partial` for the same reason `raw_cfg` is, and no
                        // second diagnostic repeats what the reader already reported.
                        run.stages[index].state = if raw.cfg.completeness.is_complete() {
                            StageState::Completed
                        } else {
                            StageState::Partial
                        };
                        // The contexts are a payload of this run alone: they stay in it because the
                        // canonical CFG pass consumes exactly this payload instead of re-deriving
                        // any return point (3.5), and the read-only handoff of P3 1.1 publishes the
                        // canonical graph rather than the contexts it was built from.
                        contexts = Some(established);
                    }
                    Ok(crate::call_context::CallContextOutcome::Forbidden { message }) => {
                        // A dialect violation, not a limitation: the raw facts are kept, the
                        // method gets no call contexts and must not enter the canonical CFG.
                        let code = crate::call_context::IR_LEGACY_OPCODE_FORBIDDEN.to_string();
                        run.stages[index].state = StageState::Failed { code: code.clone() };
                        run.diagnostics.push(Diagnostic {
                            code: code.clone(),
                            severity: DiagnosticSeverity::Error,
                            message,
                            provenance: None,
                        });
                        stop = stop.or(Some(ExecutionReport::Failed {
                            reason: TerminationReason::Error { code },
                            usage: budget.usage(),
                        }));
                        break;
                    }
                    Ok(crate::call_context::CallContextOutcome::Unresolved { message }) => {
                        // The raw facts are the part of the answer that survives: the run is
                        // partial under the pass's own code, and no call graph was invented.
                        let code = crate::call_context::IR_CALL_CONTEXT_UNRESOLVED.to_string();
                        run.stages[index].state = StageState::Partial;
                        run.diagnostics.push(Diagnostic {
                            code: code.clone(),
                            severity: DiagnosticSeverity::Warning,
                            message,
                            provenance: None,
                        });
                        stop = stop.or(Some(ExecutionReport::Partial {
                            reason: TerminationReason::Error { code },
                            usage: budget.usage(),
                        }));
                        break;
                    }
                    Err(error) => {
                        let (execution, diagnostic) = crate::ir::terminal(&error, budget.usage());
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                        break;
                    }
                }
            }
            IrPhase::CanonicalCfg => {
                let (Some(decoded), Some(raw), Some(contexts)) =
                    (facts.as_ref(), raw.as_ref(), contexts.as_ref())
                else {
                    // The pass requires the facts of the passes before it, so a run that reached
                    // it without them is the ledger's own `ir_pass_prerequisite_missing` (the
                    // schedule validation makes it unreachable).
                    if let Err(error) = ledger.apply(pass) {
                        let (execution, diagnostic) = crate::ir::terminal(&error, budget.usage());
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                    }
                    break;
                };
                match crate::canonical::canonical_cfg(
                    decoded,
                    raw,
                    contexts,
                    &request.method,
                    budget,
                ) {
                    Ok(crate::canonical::CanonicalOutcome::Canonical(graph)) => {
                        if let Err(error) = ledger.apply(pass) {
                            let (execution, diagnostic) =
                                crate::ir::terminal(&error, budget.usage());
                            run.stages[index].state = stage_state(&execution);
                            run.diagnostics.push(diagnostic);
                            stop = stop.or(Some(execution));
                            break;
                        }
                        // The graph covers the decoded prefix: a body whose decode stopped early
                        // gets the canonical graph of that prefix and says so, exactly like the
                        // two passes before it.
                        run.stages[index].state = if graph.completeness.is_complete() {
                            StageState::Completed
                        } else {
                            StageState::Partial
                        };
                        // The graph is the first table of the read-only handoff (P3 1.1): the run
                        // keeps it for the passes above and hands it over in its payload, and the
                        // report's quality plane is about the fact that it exists and nothing else.
                        canonical_cfg = Some(graph);
                    }
                    Ok(crate::canonical::CanonicalOutcome::Fallback { message }) => {
                        // The normalization stopped on a bound of its own: the raw bytecode facts
                        // and the proven call contexts are what survives, the canonical fact is
                        // not published, and the pass says why instead of completing over a graph
                        // it could not build.
                        let code = crate::canonical::IR_LEGACY_NORMALIZATION_UNBOUNDED.to_string();
                        run.stages[index].state = StageState::Partial;
                        run.diagnostics.push(Diagnostic {
                            code: code.clone(),
                            severity: DiagnosticSeverity::Warning,
                            message,
                            provenance: None,
                        });
                        stop = stop.or(Some(ExecutionReport::Partial {
                            reason: TerminationReason::Error { code },
                            usage: budget.usage(),
                        }));
                        break;
                    }
                    Err(error) => {
                        let (execution, diagnostic) = canonical_failure(&error, budget);
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                        break;
                    }
                }
            }
            IrPhase::Frame => {
                let (Some(decoded), Some(graph), Some(declaration)) =
                    (facts.as_ref(), canonical_cfg.as_ref(), declaration.as_ref())
                else {
                    // The pass requires the facts of the passes before it, so a run that reached
                    // it without them is the ledger's own `ir_pass_prerequisite_missing` (the
                    // schedule validation makes it unreachable).
                    if let Err(error) = ledger.apply(pass) {
                        let (execution, diagnostic) = crate::ir::terminal(&error, budget.usage());
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                    }
                    break;
                };
                let method = frame_method(request, declaration);
                match crate::frame::frames(decoded, graph, &method, budget) {
                    Ok(FrameOutcome::Frames(table)) => {
                        if let Err(error) = ledger.apply(pass) {
                            let (execution, diagnostic) =
                                crate::ir::terminal(&error, budget.usage());
                            run.stages[index].state = stage_state(&execution);
                            run.diagnostics.push(diagnostic);
                            stop = stop.or(Some(execution));
                            break;
                        }
                        // The frames cover the decoded prefix: a body whose decode stopped early
                        // gets the frames of that prefix and says so, like every pass before this
                        // one.
                        run.stages[index].state = if graph.completeness.is_complete() {
                            StageState::Completed
                        } else {
                            StageState::Partial
                        };
                        // The table is a table of the read-only handoff (P3 1.1): 4.2 reads it, and
                        // the run hands it over in its payload beside the graph it was derived
                        // from.
                        frame_table = Some(table);
                    }
                    Ok(FrameOutcome::Unsupported { message }) => {
                        // A state this build does not prove — an uninitialized value used where
                        // only an initialized reference is meaningful, which no conversion here
                        // defines and which this pass does not call illegal either. The phases
                        // before this one keep their facts, no `Frames` fact is published, and the
                        // reason is the frame slice's own boundary code rather than a claim about
                        // the bytes.
                        let code = IR_FRAME_DEFERRED.to_string();
                        run.stages[index].state = StageState::Partial;
                        run.diagnostics.push(Diagnostic {
                            code: code.clone(),
                            severity: DiagnosticSeverity::Warning,
                            message,
                            provenance: None,
                        });
                        stop = stop.or(Some(ExecutionReport::Partial {
                            reason: TerminationReason::Error { code },
                            usage: budget.usage(),
                        }));
                        break;
                    }
                    Ok(FrameOutcome::Inconsistent { message }) => {
                        // The body contradicts itself: the frames before the fault are not a
                        // partial answer about a state that never existed, so nothing is
                        // published and the code says what happened. The severity is the one the
                        // reader's own stops use for a damaged method — an error, unlike the
                        // boundary of this build above.
                        let code = IR_FRAME_INCONSISTENT.to_string();
                        run.stages[index].state = StageState::Partial;
                        run.diagnostics.push(Diagnostic {
                            code: code.clone(),
                            severity: DiagnosticSeverity::Error,
                            message,
                            provenance: None,
                        });
                        stop = stop.or(Some(ExecutionReport::Partial {
                            reason: TerminationReason::Error { code },
                            usage: budget.usage(),
                        }));
                        break;
                    }
                    Err(error) => {
                        let (execution, diagnostic) = crate::ir::terminal(&error, budget.usage());
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                        break;
                    }
                }
            }
            IrPhase::Ssa => {
                let (Some(decoded), Some(graph), Some(published), Some(declaration)) = (
                    facts.as_ref(),
                    canonical_cfg.as_ref(),
                    frame_table.as_ref(),
                    declaration.as_ref(),
                ) else {
                    // The pass requires what the passes before it publish, so a run that reached it
                    // without them is the ledger's own `ir_pass_prerequisite_missing` (the schedule
                    // validation makes it unreachable).
                    if let Err(error) = ledger.apply(pass) {
                        let (execution, diagnostic) = crate::ir::terminal(&error, budget.usage());
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                    }
                    break;
                };
                let method = frame_method(request, declaration);
                match crate::ssa::ssa(decoded, graph, published, &method, budget) {
                    Ok(SsaOutcome::Ssa(names)) => {
                        if let Err(error) = ledger.apply(pass) {
                            let (execution, diagnostic) =
                                crate::ir::terminal(&error, budget.usage());
                            run.stages[index].state = stage_state(&execution);
                            run.diagnostics.push(diagnostic);
                            stop = stop.or(Some(execution));
                            break;
                        }
                        // The names cover the decoded prefix, like every pass before this one: a
                        // body whose decode stopped early gets the names of that prefix and says so.
                        run.stages[index].state = if graph.completeness.is_complete() {
                            StageState::Completed
                        } else {
                            StageState::Partial
                        };
                        // The table is a table of the read-only handoff (P3 1.1): the run hands it
                        // over in its payload, and the effect facts it carries are the canonical
                        // graph's own `Effects`, which this pass re-publishes — the
                        // canonicalization invalidated the raw pass's copy.
                        ssa_table = Some(names);
                    }
                    Ok(SsaOutcome::Inconsistent { message }) => {
                        // The frames, the graph and the instructions of this body contradict each
                        // other about a slot. Like every contradiction in this pipeline, it is an
                        // error rather than a boundary of this build, nothing is published, and the
                        // phases before this one keep their facts.
                        let code = IR_SSA_INCONSISTENT.to_string();
                        run.stages[index].state = StageState::Partial;
                        run.diagnostics.push(Diagnostic {
                            code: code.clone(),
                            severity: DiagnosticSeverity::Error,
                            message,
                            provenance: None,
                        });
                        stop = stop.or(Some(ExecutionReport::Partial {
                            reason: TerminationReason::Error { code },
                            usage: budget.usage(),
                        }));
                        break;
                    }
                    Err(error) => {
                        let (execution, diagnostic) = crate::ir::terminal(&error, budget.usage());
                        run.stages[index].state = stage_state(&execution);
                        run.diagnostics.push(diagnostic);
                        stop = stop.or(Some(execution));
                        break;
                    }
                }
            }
        }
    }
    // Quality of the produced artifact: the canonical CFG is the artifact this slice produces,
    // so a run that published one is `Conservative`, and a run that never got there — because it
    // stopped, or because the caller never scheduled the pass — is `Fallback`. The plane is about
    // the artifact and not about the coverage or the termination: a canonical graph of a
    // truncated prefix is still the faithful low-level structure it is.
    run.quality = if canonical_cfg.is_some() {
        crate::ir::Quality::Conservative
    } else {
        crate::ir::Quality::Fallback
    };
    // A published frame table always holds the entry block of the body it describes: it is the
    // one state the fixpoint starts from. The check is what keeps the payload of this run tied to
    // the fact the ledger recorded instead of becoming an unread local.
    debug_assert!(
        frame_table
            .as_ref()
            .is_none_or(|table| !table.blocks().is_empty()),
        "a published frame table holds at least the entry state"
    );
    // The same property for the names 4.3 published: a table of a body whose entry block the
    // frames hold always names at least that block, so an empty one would mean the payload of this
    // run lost the fact the ledger recorded.
    debug_assert!(
        ssa_table
            .as_ref()
            .is_none_or(|table| !table.blocks().is_empty()),
        "a published SSA table names at least the entry block"
    );
    run.execution = match stop {
        // The usage of the whole request, under whichever termination stopped it first.
        Some(execution) => jarde_reader::accounting::with_usage(execution, budget.usage()),
        None => ExecutionReport::Complete {
            usage: budget.usage(),
        },
    };
    // What this run hands over (P3 1.1): the very tables the passes published, moved into the
    // payload in phase order, each present exactly when its pass published one. Nothing here
    // re-derives an artifact from the report, and nothing was read, charged or run again to build
    // it — a stopped run hands over the tables it published before the stop, and a run that never
    // reached a pass hands over `None` for it.
    //
    // The decode facts the `raw_facts` pass read travel with them (P3 1.3b): the body and the
    // class's constant pool are the one source of the symbolic vocabulary a presentation needs,
    // and moving them in is a move — no pass is re-run, nothing is re-read and nothing is
    // charged twice for it. The block that owns the pool is the same header read that filled every
    // pass above, so the pool handed over is the pool those passes read. The class's
    // `BootstrapMethods` table travels the same way (P3 2.1): it is the one fact an
    // `invokedynamic`'s bootstrap index resolves against, and it was read by that same header read.
    let (constant_pool, bootstrap_methods) = match declaration {
        Some(declaration) => (declaration.pool, declaration.bootstrap_methods),
        None => (Vec::new(), Vec::new()),
    };
    let ir = MethodIr::new(
        canonical_cfg,
        frame_table,
        ssa_table,
        facts.map(Box::new),
        constant_pool,
        bootstrap_methods,
        member,
    );
    (run, ir)
}

/// What the `raw_facts` pass found for the driver method.
///
/// The decoded facts are boxed because they are much larger than the alternative: the enum is
/// built once per request and passed on to the next pass, and the indirection keeps the common
/// path from moving a `MethodCodeFacts` by value twice. The class file's version travels with them
/// — as the reader classifies it, which is what the format's own version rule needs — because the
/// dialect decisions of the later passes are the class file's version alone and nothing else
/// ([`crate::ir::AnalysisStage::LegacyNormalization`]).
enum DriverRead {
    /// The member has a body and the reader decoded (at least a prefix of) it.
    Decoded {
        facts: Box<MethodCodeFacts>,
        version: VersionCapability,
        /// The declaration facts the later passes need beside the body.
        declaration: FrameDeclaration,
        /// What the located member's own declaration says about the member (P3 3.1), from the same
        /// header read that located it and decoded the body. Boxed for the same reason `facts` is:
        /// the enum is moved once per request and the common path is the one that does not move a
        /// `MethodDeclaration` by value.
        member: Option<Box<MethodDeclaration>>,
    },
    /// The member's own declaration says it has no body: there is nothing to analyze, and that
    /// is a fact about the member rather than a failure of the request.
    DeclaredWithoutBody,
}

/// The declaration facts a later pass reads beside the decoded body.
///
/// They come from the one header read `raw_facts` performs and are kept instead of read again:
/// whether the member is static and what it is called decide the entry frame, the class file's
/// own name is the type of an initialized `this`, and its constant pool is where the descriptor
/// of every `invoke*`, field access, `ldc` and array creation lives. The pool is moved out of the
/// header facts, so the request holds exactly one copy of it.
struct FrameDeclaration {
    /// Raw access flags of the member.
    access_flags: u16,
    /// Internal name of the class the member is declared in (`this_class`).
    this_class: Vec<u8>,
    /// Internal name of that class's superclass (`super_class`), absent for `java/lang/Object`.
    ///
    /// The frame pass reads it for one decision: whether an `invokespecial <init>` of that class
    /// is one of the two constructor calls JVMS 4.9.2 lets an instance initialization method make
    /// on its own uninitialized `this`. It is the class file's own name, taken from the one
    /// header read this pass already performs — no second class is read to learn a superclass
    /// chain this request does not hold.
    super_class: Option<Vec<u8>>,
    /// The class file's constant pool, in index order.
    pool: Vec<CpEntryFacts>,
    /// The class's `BootstrapMethods` table, in attribute order; empty when it declares none.
    ///
    /// The one fact an `invokedynamic`'s `bootstrap_method_attr_index` resolves against, read from
    /// the same attribute enumeration the member was located in (P3 2.1). A class without the
    /// attribute reads nothing for it.
    bootstrap_methods: Vec<BootstrapMethodFacts>,
}

/// The declaration facts one IR pass reads beside the body and the graph, as the frame-family
/// passes take them.
///
/// `frame` and `ssa` read the same view of the member and its class: the flags and name decide the
/// entry frame, the descriptor decides the parameter slots and the `invoke*` shapes, the class's
/// own name is the type of an initialized `this` and its superclass is the one other constructor
/// call JVMS 4.9.2 permits, the pool is where every descriptor lives, and the loader is the anchor
/// of this request's named references. One constructor keeps the two passes reading one view
/// instead of two copies of it.
fn frame_method<'a>(
    request: &'a MethodAnalysisRequest,
    declaration: &'a FrameDeclaration,
) -> FrameMethod<'a> {
    FrameMethod {
        access_flags: declaration.access_flags,
        name: &request.method.name.0,
        descriptor: &request.method.descriptor.0,
        owner: &declaration.this_class,
        super_class: declaration.super_class.as_deref(),
        pool: &declaration.pool,
        loader: &request.environment.runtime.load_domain.loader,
    }
}

/// Reads the driver method's class header and body, and fills the planes that follow from it.
///
/// This is the work of the `raw_facts` pass: one header read by identity (which also charges
/// `ClassHeaders` and checks that the bytes are the ones the definition names), the member
/// lookup by raw name and descriptor, and one `MethodBodies` attempt before the decode. The
/// member's declaration decides the body fact: a member with no `Code` attribute that declares
/// itself abstract or native is `DeclaredWithoutBody`, and a member whose declaration
/// contradicts the class-file format is the structured failure the reader would raise for it.
///
/// The class the request names is claimed under the declared load domain's own loader, and that
/// claim is what the header read **binds**: the loader's own search order has to select exactly
/// that `(loader, definition)` pair before any runtime semantics are built on it (the 0.1
/// contract, D25). A definition the declared loader would not select — one no position of its
/// order holds, one it cannot tell apart, one a nearer position shadows with another definition —
/// stops the pass under [`crate::providers::UNBOUND_DEFINITION`]; the read that happened stays in
/// `reads`, so the physical facts survive the refusal. The two snapshots involved do not have to
/// be equal: a definition provided by a depending root is a legitimate dependency, and the check
/// is the loader's decision, never snapshot equality.
fn read_driver_method(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    run: &mut crate::ir::AnalysisRun,
    budget: &mut Budget,
) -> Result<DriverRead> {
    let mut closure = crate::providers::HeaderClosure::new(content, &request.environment);
    let read = closure.read_own_definition(
        &request.environment.runtime.load_domain.loader,
        &request.method.owner,
        crate::providers::HeaderDemand::DriverMethodBody,
        budget,
    );
    // The one read a method-analysis request performs, and the demand that caused it: the
    // driver method's body. It is published after the read was attempted, exactly like every
    // other header read of this engine (a refused charge records nothing), and a binding the
    // loader refuses keeps the record of the read it was decided on.
    run.reads = crate::resolver::published_reads(&closure);
    let mut read = read?;
    let Some(member) = read.header.facts.methods.iter().find(|member| {
        member.name.raw().0 == request.method.name.0
            && member.descriptor.raw().0 == request.method.descriptor.0
    }) else {
        return Err(Error::invalid_input(
            "classfile_method_not_found",
            format!(
                "the driver method `{}` `{}` is not declared by its own class definition",
                String::from_utf8_lossy(&request.method.name.0),
                String::from_utf8_lossy(&request.method.descriptor.0),
            ),
        ));
    };
    if !has_code_attribute(member) {
        return match no_body_kind(member.access_flags) {
            Some(no_body_kind) => {
                // The declaration says there is no body: the body plane states that fact, no
                // pass can run on it, and the request is complete as far as its input allows.
                run.body = MethodBodyState::DeclaredWithoutBody { no_body_kind };
                run.diagnostics.push(Diagnostic {
                    code: "ir_method_declared_without_body".to_string(),
                    severity: DiagnosticSeverity::Info,
                    message: format!(
                        "the driver method declares no `Code` attribute and its access flags \
                         say {no_body_kind:?}: the request has no body to analyze, so no phase ran"
                    ),
                    provenance: None,
                });
                Ok(DriverRead::DeclaredWithoutBody)
            }
            None => Err(Error::invalid_input(
                "classfile_method_has_no_code",
                "the driver method declares no `Code` attribute and is neither abstract nor \
                 native, so its declaration contradicts the class-file format",
            )),
        };
    }
    // One body read attempt, charged before the read; the member has a body, so the attempt is
    // a body the request really demands. From here on the body is a located one: a decode that
    // fails is a failure of this pass, not a missing body.
    budget.charge(CountedBudgetDimension::MethodBodies, 1)?;
    run.body = MethodBodyState::Present;
    let decoded = jarde_reader::classfile::method_code_facts(&read.bytes, member, budget)?;
    run.coverage = jarde_reader::classfile::method_code_coverage(
        decoded.code_span.length,
        &decoded.instructions,
        decoded.exception_handlers.len(),
        decoded.exception_handler_count,
        &decoded.execution,
        decoded.stopped_at.as_ref(),
    )?;
    Ok(DriverRead::Decoded {
        facts: Box::new(decoded),
        // The dialect of the later passes is the class file's version and nothing else, so it
        // is classified here, once, from the same header the member was located in: the reader's
        // own rule over the two version fields, not a second reading of them.
        version: jarde_reader::classfile::version_capability(
            read.header.facts.major_version,
            read.header.facts.minor_version,
        ),
        // The same read carries the declaration facts the `frame` pass needs. The constant pool
        // is *moved* out of the header facts: this request keeps one copy of it, and the pool of
        // no other class is read for it.
        //
        // The class's `BootstrapMethods` table is read here too (P3 2.1): the attribute's shell was
        // already enumerated by this header read, the bytes are the ones this read holds, and the
        // pool it resolves against is the very pool this request keeps. A class that declares no
        // such attribute reads nothing and charges nothing — a class *with* one pays its own
        // attribute bytes, which is what every other attribute read in this pipeline does.
        //
        // A read that fails is propagated rather than swallowed: the reader validates that the
        // bootstrap handle and every argument are loadable constants (JVMS 4.7.23), so a class that
        // fails it is one whose structure contradicts the format, and this pass already treats a
        // body read that fails the same way. An empty table therefore means "this class declares
        // no bootstrap table", never "the table could not be read".
        declaration: {
            let pool = std::mem::take(&mut read.header.facts.constant_pool);
            let bootstrap_methods = match read
                .header
                .facts
                .attributes
                .iter()
                .find(|shell| shell.name.raw().0.as_slice() == b"BootstrapMethods")
            {
                Some(shell) => {
                    jarde_reader::classfile::bootstrap_methods(&read.bytes, shell, &pool, budget)?
                }
                None => Vec::new(),
            };
            FrameDeclaration {
                access_flags: member.access_flags,
                this_class: read.header.facts.this_class.raw().0.clone(),
                super_class: read
                    .header
                    .facts
                    .super_class
                    .as_ref()
                    .map(|name| name.raw().0.clone()),
                pool,
                bootstrap_methods,
            }
        },
        // The member's own declaration (P3 3.1), taken from the member this read located rather than
        // from the request: the flags and the descriptor are what the class file states, and the
        // parameter slots those two imply are derived once, here, from that same statement.
        member: MethodDeclaration::new(
            member.access_flags,
            member.name.raw().clone(),
            member.descriptor.raw().clone(),
            request.method.clone(),
        )
        .map(Box::new),
    })
}

/// Whether the member declares a `Code` attribute at all, decided from the header's shells so
/// the question costs no read.
pub(crate) fn has_code_attribute(member: &MemberHeader) -> bool {
    member
        .attributes
        .iter()
        .any(|shell| shell.name.raw().0.as_slice() == b"Code")
}

/// The body kind a member's own access flags declare, when it declares one.
fn no_body_kind(access_flags: u16) -> Option<NoBodyKind> {
    if access_flags & ACC_ABSTRACT != 0 {
        Some(NoBodyKind::Abstract)
    } else if access_flags & ACC_NATIVE != 0 {
        Some(NoBodyKind::Native)
    } else {
        None
    }
}

/// The termination and diagnostic of a reader that stopped before the end of the body.
///
/// The reason is the reader's own, including its dimension for a budget stop, so the analysis
/// report never invents a second vocabulary for the same stop; the diagnostic carries the
/// reader's stable code and the position it stopped at, and its severity follows the reason
/// (a decode failure is an error, an exhausted budget or a cancellation is a warning).
fn reader_stop(facts: &MethodCodeFacts, budget: &Budget) -> Option<(ExecutionReport, Diagnostic)> {
    let stopped_at = facts.stopped_at.as_ref()?;
    let code = stopped_at_code(stopped_at);
    let position = match stopped_at {
        BytecodeStop::Instructions {
            bci, class_offset, ..
        } => format!("BCI {bci} (class offset {class_offset})"),
        BytecodeStop::ExceptionHandlers {
            ordinal,
            class_offset,
            ..
        } => format!("exception-table record {ordinal} (class offset {class_offset})"),
    };
    let reason = match &facts.execution {
        ExecutionReport::Partial { reason, .. } => reason.clone(),
        ExecutionReport::Cancelled { .. } => TerminationReason::Error {
            code: code.to_string(),
        },
        // A stop the report cannot name is not a stop this function invents a reason for.
        ExecutionReport::Complete { .. } | ExecutionReport::Failed { .. } => return None,
    };
    let severity = match &facts.execution {
        ExecutionReport::Partial { reason, .. } => stop_severity(reason),
        _ => DiagnosticSeverity::Warning,
    };
    Some((
        ExecutionReport::Partial {
            reason,
            usage: budget.usage(),
        },
        Diagnostic {
            code: code.to_string(),
            severity,
            message: format!(
                "the method body stopped before its end at {position}: the analysis covers the \
                 reliable decoded prefix and the coverage plane names the rest"
            ),
            provenance: None,
        },
    ))
}

/// The reader's stable code of one stop.
fn stopped_at_code(stop: &BytecodeStop) -> &str {
    match stop {
        BytecodeStop::ExceptionHandlers { code, .. } | BytecodeStop::Instructions { code, .. } => {
            code
        }
    }
}

/// Severity of a reader stop: a damaged body is an error, an exhausted budget or a cancelled
/// request is a warning that the prefix is still usable.
fn stop_severity(reason: &TerminationReason) -> DiagnosticSeverity {
    match reason {
        TerminationReason::Error { .. } => DiagnosticSeverity::Error,
        TerminationReason::BudgetExceeded { .. } | TerminationReason::Unsupported { .. } => {
            DiagnosticSeverity::Warning
        }
    }
}

/// The failure of the `raw_cfg` pass, mapped to the report's planes.
///
/// A target the reader refuses on a body whose decode stopped early is 1.2's
/// sound-but-incomplete view, not corruption: the unread suffix may contain the instruction
/// start the target needs, so the pass reports a `Partial` stage under its own code instead of
/// calling the method damaged. On a fully decoded body the same error is the structured failure
/// the reader raised.
fn raw_cfg_failure(
    error: &Error,
    decoded: &MethodCodeFacts,
    budget: &Budget,
) -> (ExecutionReport, Diagnostic) {
    let incomplete = !crate::cfg::completeness_of(decoded).is_complete();
    if incomplete && matches!(error, Error::InvalidInput { .. }) {
        return (
            ExecutionReport::Partial {
                reason: TerminationReason::Error {
                    code: IR_RAW_CFG_INCOMPLETE_BODY.to_string(),
                },
                usage: budget.usage(),
            },
            Diagnostic {
                code: IR_RAW_CFG_INCOMPLETE_BODY.to_string(),
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "the raw CFG was not built: the body decode stopped before its end, so the \
                     branch and handler targets of the unread suffix cannot be validated \
                     ({error})"
                ),
                provenance: None,
            },
        );
    }
    crate::ir::terminal(error, budget.usage())
}

/// The failure of the `canonical_cfg` pass, mapped to the report's planes.
///
/// A bound this pass runs into — the clone ceiling, or an exhausted `Blocks`/`Steps`/`Clones`
/// dimension it declares — is exactly what 3.5 answers with a fallback: the raw bytecode facts
/// and the proven call contexts stay, no canonical fact is published, and the reason is the
/// normalization code of the design rather than a dimension name a reader would have to translate
/// back into "the cloning stopped". The underlying measure is not hidden: the message names the
/// `Error` the budget layer raised, and `usage` carries the counts.
///
/// Cancellation and a structural failure keep the ordinary mapping: neither is a bound of this
/// pass, and `ExecutionReport::Cancelled` must stay a cancellation.
fn canonical_failure(error: &Error, budget: &Budget) -> (ExecutionReport, Diagnostic) {
    if matches!(error, Error::BudgetExceeded { .. }) {
        let code = crate::canonical::IR_LEGACY_NORMALIZATION_UNBOUNDED.to_string();
        return (
            ExecutionReport::Partial {
                reason: TerminationReason::Error { code: code.clone() },
                usage: budget.usage(),
            },
            Diagnostic {
                code,
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "the legacy normalization stopped at its own bound ({error}): the raw \
                     bytecode facts and the proven call contexts of the decoded prefix are kept, \
                     and no canonical CFG was published"
                ),
                provenance: None,
            },
        );
    }
    crate::ir::terminal(error, budget.usage())
}

/// The stage state one pass ended in, from the terminal outcome of its work, so a stage result
/// and the execution plane can never disagree about the same pass.
fn stage_state(execution: &ExecutionReport) -> StageState {
    match execution {
        ExecutionReport::Complete { .. } => StageState::Completed,
        ExecutionReport::Partial { .. } | ExecutionReport::Cancelled { .. } => StageState::Partial,
        ExecutionReport::Failed { reason, .. } => StageState::Failed {
            code: termination_code(reason),
        },
    }
}

/// Stable code of one termination reason, the code a `Failed` stage states.
fn termination_code(reason: &TerminationReason) -> String {
    match reason {
        TerminationReason::Error { code } | TerminationReason::Unsupported { code } => code.clone(),
        TerminationReason::BudgetExceeded { dimension } => {
            format!(
                "budget_exceeded_{}",
                jarde_reader::artifact::budget_dimension_code(*dimension)
            )
        }
    }
}
