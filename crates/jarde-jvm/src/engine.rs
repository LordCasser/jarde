//! The analysis driver: the one module that calls the resolver, the environment and the IR.
//!
//! Everything a P2 request does beyond the facade's argument handling lives here: the
//! resolver's request checks and report assembly, the environment validation that decides
//! whether a request starts at all, and the method-analysis driver that schedules the pass
//! table, reads one driver method through the reader, and turns every pass outcome into the
//! report's stage, coverage, read and execution planes.
//!
//! The three entry points below are what the facade delegates to, one line each; the
//! composition story — which entry runs what, and what a caller observes — is documented on
//! `jarde::Engine`, which is the surface a consumer names.

use jarde_reader::artifact::ArtifactSnapshot;
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{BytecodeStop, MemberHeader, MethodCodeFacts};
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    Coverage, Diagnostic, DiagnosticSeverity, ExecutionReport, TerminationReason,
};

use crate::ir::{
    MethodAnalysisReport, MethodAnalysisRequest, MethodBodyState, NoBodyKind, StageResult,
    StageState,
};
use crate::passes::{FactLedger, IR_PASS_NOT_IMPLEMENTED, IrPhase, PassDescriptor, implemented};

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

/// Method IR analysis under an explicit environment (P2 entry point).
///
/// The request and the requested stages are validated against the fixed pass table before
/// anything runs, and the declared environment is checked before a read is attempted: a
/// rejected environment never starts the pipeline, and a schedule the table cannot serve is an
/// input error rather than a half-initialized run. What the scheduled passes then do, and what
/// each stop keeps, is [`run_method_analysis`]'s contract; the report is assembled from the
/// validated request and the run.
pub fn analyze_method(
    content: &[ArtifactSnapshot],
    request: &MethodAnalysisRequest,
    budget: &mut Budget,
) -> Result<MethodAnalysisReport> {
    crate::ir::validate_request(content, request)?;
    let scheduled = crate::passes::validate_requested_stages(&request.stages)?;
    let (problems, environment_identity) =
        crate::environment::validate_environment(content, &request.environment);
    let run = if problems.is_empty() {
        run_method_analysis(content, request, scheduled, budget)
    } else {
        // A rejected environment never yields a definition and never starts a read, so the
        // pipeline is not run at all; the report names the problems and the capability that
        // did not run.
        crate::ir::AnalysisRun::not_performed(
            &scheduled
                .iter()
                .map(|pass| pass.phase.stage())
                .collect::<Vec<_>>(),
            crate::ir::METHOD_ANALYSIS_NOT_IMPLEMENTED,
            budget,
        )
    };
    Ok(crate::ir::analysis_report(
        request,
        problems,
        environment_identity,
        run,
    ))
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

/// Runs the scheduled passes of one method-analysis request in table order.
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
/// instead of an invented one.
fn run_method_analysis(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    scheduled: &[PassDescriptor],
    budget: &mut Budget,
) -> crate::ir::AnalysisRun {
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
        execution: ExecutionReport::Complete {
            usage: budget.usage(),
        },
        diagnostics: Vec::new(),
    };
    let mut ledger = FactLedger::new();
    let mut stop: Option<ExecutionReport> = None;
    let mut facts: Option<MethodCodeFacts> = None;
    // The class file's dialect and the raw graph of this run: `legacy_normalization` reads both,
    // and both are facts of the passes that already completed.
    let mut major_version: Option<u16> = None;
    let mut raw: Option<crate::cfg::RawCfgOutcome> = None;
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
                        major_version: version,
                    }) => {
                        major_version = Some(version);
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
                        // The graph and the effect facts are crate-private IR payloads
                        // (invariant 11): 3.4/3.5 consume them, 5.1 decides what becomes
                        // public, and the report keeps publishing their status planes. The
                        // payload stays in this run because the next pass reads it.
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
                    (facts.as_ref(), raw.as_ref(), major_version)
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
                match crate::call_context::call_contexts(decoded, raw, version, budget) {
                    Ok(crate::call_context::CallContextOutcome::Established(_contexts)) => {
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
                        // The contexts are a crate-private payload (invariant 11) with no
                        // consumer in this build yet: the ledger publishes the fact, 3.5 reads
                        // it, 5.1 decides what becomes public, and the report keeps publishing
                        // the status planes.
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
            _ => {
                // A phase this build does not implement: the request is answered with the
                // failure of that pass, and the phases behind it stay `NotPerformed` instead
                // of looking performed.
                stop = stop.or(Some(report_unimplemented(
                    &mut run, index, pass.phase, budget,
                )));
                break;
            }
        }
    }
    run.execution = match stop {
        // The usage of the whole request, under whichever termination stopped it first.
        Some(execution) => jarde_reader::accounting::with_usage(execution, budget.usage()),
        None => ExecutionReport::Complete {
            usage: budget.usage(),
        },
    };
    run
}

/// What the `raw_facts` pass found for the driver method.
///
/// The decoded facts are boxed because they are much larger than the alternative: the enum is
/// built once per request and passed on to the next pass, and the indirection keeps the common
/// path from moving a `MethodCodeFacts` by value twice. The class file's own major version
/// travels with them because the dialect decisions of the later passes are the class file's
/// version alone and nothing else ([`crate::ir::AnalysisStage::LegacyNormalization`]).
enum DriverRead {
    /// The member has a body and the reader decoded (at least a prefix of) it.
    Decoded {
        facts: Box<MethodCodeFacts>,
        major_version: u16,
    },
    /// The member's own declaration says it has no body: there is nothing to analyze, and that
    /// is a fact about the member rather than a failure of the request.
    DeclaredWithoutBody,
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
    let read = read?;
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
        // is read here, once, from the same header the member was located in.
        major_version: read.header.facts.major_version,
    })
}

/// Whether the member declares a `Code` attribute at all, decided from the header's shells so
/// the question costs no read.
fn has_code_attribute(member: &MemberHeader) -> bool {
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
