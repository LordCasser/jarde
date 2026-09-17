//! Bootstrap consumer: the deferred bootstrap/condy graph of used dynamic sites.
//!
//! The class-level `BootstrapMethods` attribute is not a reference by itself: it is a
//! table that a dynamic site reaches. This consumer therefore starts from code and from
//! the request, never from the table:
//!
//! 1. it reads a unit only when the request names [`ConsumerKind::Bootstrap`] or
//!    [`ConsumerKind::Type`], and never for the raw constant-pool probe the X0 producer
//!    owns. Both categories have facts here: `Bootstrap` owns the graph nodes, and `Type`
//!    owns the types a reached descriptor names, which no other consumer can report when
//!    the descriptor exists only in this table;
//! 2. it collects use-sites from real instructions: `invokedynamic` (0xba) on a
//!    `CONSTANT_InvokeDynamic` entry, and `ldc`/`ldc_w`/`ldc2_w` (0x12–0x14) on a
//!    `CONSTANT_Dynamic` entry (or, for a malformed class, `CONSTANT_InvokeDynamic`);
//! 3. with no use-site in the unit it returns without reading the `BootstrapMethods`
//!    content at all, so an unused table entry, an unused bootstrap handle and an unused
//!    `CONSTANT_Dynamic` entry produce no item under either category and no second read of
//!    the attribute: the content is never billed again as `AttributeBytes`. The reader's
//!    shell enumeration already bills every attribute entry as `6 + content length` while
//!    `class_facts` runs, and that shell pass is the only cost an unused table has — the
//!    same billing convention the code consumer follows for a `Code` attribute it decodes
//!    (architecture §9.6: only a real consumer explains its bootstrap dependency);
//! 4. with a use-site it reads the attribute once and walks the graph that site opens:
//!    the dynamic node → the bootstrap method handle → each argument, recursing through
//!    `CONSTANT_Dynamic` arguments, which is what nested condy is.
//!
//! # The graph, and what `via` records
//!
//! A graph node is one constant-pool entry. `via` is the ordered node path from the
//! use-site to the node the item describes, so it also records every edge that was
//! walked:
//!
//! * `via[0]` is the use-site itself: `constant_pool_index` is the dynamic entry the
//!   instruction consumed, and both positions are `None` because no edge reaches it;
//! * every later element names the node that was reached together with the position of
//!   the edge that reached it from the previous element: `bootstrap_index` is the
//!   `BootstrapMethods` index of the previous dynamic node, and `argument_index` is the
//!   **static** argument slot that was followed — the position inside that entry's
//!   `arguments` array, so it excludes the three leading arguments the JVM supplies at
//!   invocation — while `None` means that node's bootstrap method handle.
//!
//! `evidence.constant_pool_index` repeats the described node's entry and `evidence.span`
//! is that entry's class-file range, so every hop of every path can be sliced out of the
//! class bytes — including facts this consumer deliberately does not interpret, such as a
//! `MethodHandle`'s `reference_kind` or a `MethodType`'s descriptor. `evidence.bci` and
//! `evidence.opcode` are the use-site instruction, and `evidence.attribute` is the raw
//! `BootstrapMethods` name. The graph node itself has no source position of its own in
//! this class file, so `source` is always the use-site's `Location::Code`: that
//! instruction is why the node is in the result.
//!
//! Item operations:
//!
//! * [`XrefOperation::BootstrapMethod`] — a bootstrap method handle. The described entry
//!   is the `CONSTANT_MethodHandle`, and the target is the member that handle names.
//! * [`XrefOperation::BootstrapArgument`] — an argument node. A `MethodHandle` argument is
//!   reported as the member it names, a `String` as its bytes, a `Class` as a class symbol
//!   and literal, a primitive as its JVM value, a `CONSTANT_Dynamic` as the dynamic site's
//!   name and descriptor (never as a value), and a `MethodType` as its raw descriptor
//!   bytes: the frozen schema has no descriptor target, so the descriptor travels as a
//!   string literal while `evidence.constant_pool_index` keeps pointing at the
//!   `CONSTANT_MethodType` entry itself, never at a `CONSTANT_String`.
//!
//! # The types a reached descriptor names
//!
//! A node whose entry carries a descriptor also names types, and those are a claim of the
//! `Type` category: the entry's own types are published as
//! `ConsumerKind::Type` items with the same operation (`BootstrapMethod` /
//! `BootstrapArgument`), the same `BootstrapEdge` derivation, the same node entry and span
//! in `evidence`, and the same `via` path — the only difference is the target, which is the
//! described type as a `SymbolRef::Class`. The descriptor's types are:
//!
//! * a `MethodType` node — its method descriptor (JVMS 4.4.9);
//! * a `MethodHandle` node — the descriptor of the member its `reference_index` names;
//! * a `Dynamic` node — its **field** descriptor (JVMS 4.4.10);
//! * an `InvokeDynamic` node — its method descriptor.
//!
//! Gating, and why the boundary sits here: the walk is opened by the two categories whose
//! facts it carries — `Bootstrap`, which owns the node items above, and `Type`, because a
//! type that only a reached descriptor names has no other consumer that could report it
//! (decision 26). Each product keeps its own gate, so a `Bootstrap`-only request produces
//! the node items and no type item, and a `Type`-only request produces the type items and
//! no node item.
//!
//! The deferral itself is unchanged (decisions 12/23): nothing is read without a real
//! use-site. A class whose instructions consume no dynamic entry reaches no table entry, so
//! `BootstrapMethods` is never read, never billed and never turned into a fact, and an
//! entry that no dynamic site names produces no item under either category — the
//! descriptor types are produced where an edge is published instead of by scanning the
//! pool, which is what keeps an unused entry invisible.
//!
//! Every node item here is `XrefCertainty::Exact`, `XrefDerivation::BootstrapEdge`,
//! `consumer: Some(Bootstrap)` and `QueryResolution::NotRequested`, and no item is ever
//! filtered out of the scan for being uncertain: a node that names nothing reportable
//! (a handle whose reference is not a member-carrying entry) simply produces no fact. The
//! descriptor types carry `consumer: Some(Type)` and the node's own operation, derivation,
//! evidence and path, and the same rule applies to them.
//!
//! # What this consumer does not claim (architecture §9.6)
//!
//! * Creating a lambda form is not calling its implementation: the implementation handle
//!   is reported as an argument fact, and this module produces no `Invoke*`, `Ldc`, `New`
//!   or `ConstantValue` operation for any bootstrap node.
//! * A custom bootstrap has no statically known call-site target: the bootstrap handle and
//!   its arguments are reported, and nothing claims what the bootstrap would return. The
//!   `QueryResolution` stays `NotRequested`.
//! * A `CONSTANT_Dynamic` argument is a site with dependencies, never a value: the node is
//!   reported as a symbol and its own bootstrap graph is walked instead.
//! * A method reference does not prove that a `lambda$...` method exists: only the member
//!   the handle really names is reported, and no sibling member is invented.
//! * The `LambdaMetafactory` shape is **not** pattern-recognised: the positions below are
//!   the only structure this module states, and they are the positions the class file really
//!   holds. When the JVM invokes a bootstrap method it supplies three leading arguments
//!   itself — a `MethodHandles.Lookup`, the name, and a `Class` or `MethodType` derived from
//!   the dynamic site (JVMS §5.4.3.6) — so the `arguments` array of a `BootstrapMethods`
//!   entry holds the *static* arguments only, and the invoked argument list is those static
//!   arguments appended to the three leading ones. `LambdaMetafactory.metafactory` is a
//!   6-parameter bootstrap method whose three static arguments are, in order, the SAM method
//!   type, the implementation `MethodHandle` and the instantiated method type;
//!   `altMetafactory` declares those same three followed by its flags and the
//!   flag-dependent extras. An implementation handle is therefore the node reached by static
//!   `argument_index` 1, and the SAM name and the invoked type are not in the table at all:
//!   they are the `NameAndType` of the `invokedynamic` site, which the code consumer
//!   ([`ConsumerKind::Invocation`]) reports as that site's symbol. A consumer joins the two
//!   products by the `via` prefix this module records; the three leading arguments the JVM
//!   adds are not in the class file and cannot appear in `via`. The frozen item schema has
//!   no field that could mark an implementation handle further, and this module does not add
//!   one — and no position above is asserted as a resolved lambda, a linkage result or an
//!   invocation.
//! * Nothing here executes a bootstrap, loads a class or resolves a member: the graph is
//!   read out of the class file, not run.
//!
//! # Cost, order and bounds
//!
//! One attribute content is read: `BootstrapMethods`, once per unit, and only when a
//! use-site exists. Use-sites come from `Code` attributes, which this consumer reads
//! through the same reader facts as the code consumer, so a `Code` body is billed by every
//! pass that reads it. `ResultItems` is never charged here — `super` bills every item it
//! publishes.
//!
//! Items come out in a fixed order, and only its first part is a class-file position order.
//! Use-sites are scanned in ascending class-file position: the methods in declaration order,
//! and inside one method the instructions by BCI. Inside one use-site the walk follows the
//! graph as the tables declare it — the bootstrap method handle first, then each static
//! argument in the attribute's argument order, descending into a dynamic argument right after
//! its own edge fact (depth first). That interior order is declaration order, **not** a
//! constant-pool index order or a class-file offset order: a node reached deeper into the
//! graph may well live at a smaller index than the node that reached it. The page cursor in
//! `super` depends on that whole order.
//!
//! The traversal is bounded by the unit's own constant pool: every node and every edge of
//! this graph lands on one entry, so the entry count is the size the traversal may grow to
//! (see [`Ledger`]). A dynamic node is expanded once per unit ([`Graph::expanded`]) and its
//! interior is published once per use-site, while every edge keeps its own path, so a
//! shared subgraph is neither re-expanded nor re-published for a second edge of the same
//! use-site. A cycle is reported as a diagnostic and stops the unit instead of recursing.
//!
//! # Diagnostics
//!
//! `query_bootstrap_attribute_missing`, `query_bootstrap_attribute_duplicate`,
//! `query_bootstrap_malformed`, `query_bootstrap_cycle`, `query_bootstrap_graph_limit` and
//! `query_bootstrap_code_stopped`. Each is a domain diagnostic: `super` bills it once, and a
//! structured error accompanies it so the report is never `Complete` over a scan that
//! stopped.
//!
//! Severity separates "this fact is reliable, the traversal stopped" from "the class file is
//! what could not be read": a cycle is a `Warning`, because every fact published before the
//! repeated node is exact and only the walk ended, while a missing, duplicated or malformed
//! table, an exhausted derived bound and a member body that did not decode are `Error`s.
//! Severity never decides the execution state: all six make the unit end with a structured
//! error, so `execution` is `Failed` in every one of them (a budget stop is `Partial` and a
//! cancellation is `Cancelled`), and `coverage` is `Partial` in all of them.

use super::{ScanContext, ScanUnit, class_content, to_u64};
use crate::budget::{BudgetDimension, Limits, UsageSnapshot};
use crate::classfile::{
    AttributeShell, BytecodeStop, ClassFacts, CpEntryFacts, CpEntryKind, EntryDescriptor,
    MemberHeader, MethodCodeFacts, bootstrap_methods, class_facts, cp_entry, descriptor_types,
    entry_descriptor, method_code_facts,
};
use crate::error::{Error, Result};
use crate::model::{
    ArchiveNameBytes, ByteSpan, ClassBytesId, Diagnostic, DiagnosticSeverity, ExecutionReport,
    JvmBytes, Location, PhysicalDefinitionId, PhysicalMethodId, Provenance, SymbolRef,
    TerminationReason,
};
use crate::query::{
    BootstrapVia, ConsumerKind, LiteralValue, QueryRelation, QueryResolution, QueryTarget,
    XrefCertainty, XrefDerivation, XrefEvidence, XrefItem, XrefOperation, XrefTarget,
};
use std::collections::{BTreeMap, BTreeSet};

/// Raw name of the class-level bootstrap table.
const BOOTSTRAP_METHODS: &[u8] = b"BootstrapMethods";

/// Raw name of a member body attribute.
const CODE_ATTRIBUTE: &[u8] = b"Code";

/// `invokedynamic`: the one opcode that consumes a `CONSTANT_InvokeDynamic` entry.
const INVOKE_DYNAMIC: u8 = 0xba;

/// `ldc`, `ldc_w`, `ldc2_w`: the family that consumes a loadable constant.
const LDC_FAMILY: std::ops::RangeInclusive<u8> = 0x12..=0x14;

/// A dynamic use-site exists but the class declares no bootstrap table.
const ATTRIBUTE_MISSING_CODE: &str = "query_bootstrap_attribute_missing";

/// A class declares the bootstrap table twice, so no single table is meant.
const ATTRIBUTE_DUPLICATE_CODE: &str = "query_bootstrap_attribute_duplicate";

/// The bootstrap table or one of its references does not match its declared structure.
const MALFORMED_CODE: &str = "query_bootstrap_malformed";

/// The deferred graph reached a node that is already on the current path.
const CYCLE_CODE: &str = "query_bootstrap_cycle";

/// The deferred graph outgrew the bound derived from the unit's constant pool.
const GRAPH_LIMIT_CODE: &str = "query_bootstrap_graph_limit";

/// A member body stopped early, so the use-site list of this unit is incomplete.
const CODE_STOPPED_CODE: &str = "query_bootstrap_code_stopped";

pub(super) fn scan(
    ctx: &mut ScanContext<'_>,
    unit: &ScanUnit,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    // The raw constant-pool probe answers about pool entries, not about consumers, so the
    // X0 producer owns it: a bootstrap edge is a consumer fact.
    if ctx.request().relation == QueryRelation::ConstantPoolContains {
        return Ok(());
    }
    // The graph carries facts of two categories, and either of them opens it: `Bootstrap`
    // owns the nodes (the bootstrap method handle and every argument), and `Type` owns the
    // types a reached descriptor names — a type that only a bootstrap descriptor names is
    // reachable nowhere else, so a `Type`-only request has to trigger these reads itself
    // instead of depending on another category being requested too (decision 26). Which
    // product each category gets is decided where the fact is published ([`push_fact`]).
    let wants_bootstrap = ctx.wants(ConsumerKind::Bootstrap);
    let wants_types = ctx.wants(ConsumerKind::Type);
    if !wants_bootstrap && !wants_types {
        return Ok(());
    }
    // The shared rule, the read and the damaged-candidate failure are `super`'s: an
    // archive entry is a class only by name, a standalone CLASS snapshot root is one by
    // construction, and this stream reads bytes only once its category was requested.
    let Some(content) = class_content(ctx, unit)? else {
        return Ok(());
    };
    let definition = unit.definition(ClassBytesId {
        digest: content.digest.clone(),
        length: to_u64(content.bytes.len())?,
    });
    let facts = {
        let budget = ctx.budget();
        class_facts(&content.bytes, budget)?
    };
    let found = use_sites(ctx, &content.bytes, &facts, &definition)?;
    if found.sites.is_empty() {
        // Deferred: with no dynamic use-site the bootstrap table is not read at all, so a
        // table entry nothing consumes cannot reach the result.
        return match found.stopped {
            Some(error) => Err(error),
            None => Ok(()),
        };
    }
    let first = &found.sites[0];
    let shell = bootstrap_shell(ctx, &facts, first)?;
    let bootstraps = read_bootstraps(ctx, &content.bytes, shell, &facts, first)?;

    let mut graph = Graph::new(&facts.constant_pool, &bootstraps);
    let mut outcome = Ok(());
    for site in &found.sites {
        let open = OpenSite { site, shell };
        if let Err(error) = emit_site(ctx, &mut graph, &open, out) {
            // A stop is deterministic: the items already published stay, the unit ends
            // with the structured error, and the remaining sites are not guessed at.
            outcome = Err(error);
            break;
        }
    }
    match (outcome, found.stopped) {
        (Err(error), _) => Err(error),
        (Ok(()), Some(error)) => Err(error),
        (Ok(()), None) => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// Use-sites: the real consumers that explain a bootstrap dependency
// ---------------------------------------------------------------------------

/// One instruction that consumes a dynamic constant-pool entry.
struct UseSite {
    method: PhysicalMethodId,
    bci: u32,
    opcode: u8,
    /// The dynamic entry the instruction consumed: the first hop of every path.
    entry: u16,
}

/// The dynamic use-sites of one unit, plus the first body that stopped early.
struct SiteScan {
    sites: Vec<UseSite>,
    /// A member body that did not decode completely. The sites found before the stop are
    /// facts, but the unit's use-site list is not complete, so the unit ends with this
    /// error instead of claiming a complete graph.
    stopped: Option<Error>,
}

/// Collects the unit's dynamic use-sites in ascending class-file position.
fn use_sites(
    ctx: &mut ScanContext<'_>,
    bytes: &[u8],
    facts: &ClassFacts,
    definition: &PhysicalDefinitionId,
) -> Result<SiteScan> {
    let mut sites = Vec::new();
    let mut stopped: Option<Error> = None;
    for (index, method) in facts.methods.iter().enumerate() {
        // An abstract or native member declares no body: that absence is normal, and
        // asking the reader to decode one would turn it into a failure.
        if !has_code(method) {
            continue;
        }
        let code = {
            let budget = ctx.budget();
            method_code_facts(bytes, method, budget)?
        };
        let method_id = PhysicalMethodId {
            owner: definition.clone(),
            name: method.name.raw().clone(),
            descriptor: method.descriptor.raw().clone(),
        };
        for instruction in &code.instructions {
            let Some(entry) =
                dynamic_use(facts, instruction.opcode, instruction.constant_pool_index)?
            else {
                continue;
            };
            sites.push(UseSite {
                method: method_id.clone(),
                bci: instruction.bci,
                opcode: instruction.opcode,
                entry,
            });
        }
        match body_stop(ctx, &code) {
            // No further read of this request can succeed, so the unit stops here with the
            // sites it already has.
            Some(BodyStop::Budget(error)) => return Err(error),
            Some(BodyStop::Decode(error)) => {
                ctx.push_diagnostic(stop_diagnostic(index, &method_id, &code));
                if stopped.is_none() {
                    stopped = Some(error);
                }
            }
            None => {}
        }
    }
    Ok(SiteScan { sites, stopped })
}

/// Whether a member declares a `Code` attribute at all.
///
/// Checked from the attribute shells, so the decision costs no read.
fn has_code(method: &MemberHeader) -> bool {
    method
        .attributes
        .iter()
        .any(|shell| shell.name.raw().0.as_slice() == CODE_ATTRIBUTE)
}

/// Constant-pool entry of the dynamic site one instruction consumes.
///
/// The opcode decides which entry kind is a use-site: `invokedynamic` needs a
/// `CONSTANT_InvokeDynamic`, and the `ldc` family consumes a `CONSTANT_Dynamic` (an entry
/// kind it cannot legally load is not turned into a dynamic site here). An opcode outside
/// these families, or an instruction without a constant-pool operand, is no use-site.
fn dynamic_use(facts: &ClassFacts, opcode: u8, index: Option<u16>) -> Result<Option<u16>> {
    if opcode != INVOKE_DYNAMIC && !LDC_FAMILY.contains(&opcode) {
        return Ok(None);
    }
    let Some(index) = index else {
        return Ok(None);
    };
    let kind = &cp_entry(&facts.constant_pool, index)?.kind;
    let dynamic = match opcode {
        INVOKE_DYNAMIC => matches!(kind, CpEntryKind::InvokeDynamic { .. }),
        _ => matches!(
            kind,
            CpEntryKind::Dynamic { .. } | CpEntryKind::InvokeDynamic { .. }
        ),
    };
    Ok(dynamic.then_some(index))
}

/// How a method body stopped before the end of its instruction stream.
enum BodyStop {
    /// Budget or cancellation: no further read in this request can succeed.
    Budget(Error),
    /// The member's code did not decode completely; the remaining members are still read.
    Decode(Error),
}

/// Classifies a method body that stopped, if it stopped at all.
///
/// `Partial { Error { .. } }` is exactly how the reader reports a body whose code did not
/// decode; every other non-complete execution is a budget or cancellation condition, which
/// the live budget re-states from the condition the reader recorded.
fn body_stop(ctx: &mut ScanContext<'_>, facts: &MethodCodeFacts) -> Option<BodyStop> {
    match &facts.execution {
        ExecutionReport::Complete { .. } => None,
        ExecutionReport::Partial {
            reason: TerminationReason::Error { .. },
            ..
        } => Some(BodyStop::Decode(Error::invalid_input(
            stop_code(facts).to_owned(),
            stop_description(facts),
        ))),
        _ => Some(BodyStop::Budget(budget_stop(ctx, facts))),
    }
}

/// Re-states the budget or cancellation condition a stopped body recorded.
///
/// The reader records *which* condition stopped the decode — a cancellation, elapsed time,
/// or one budget dimension — but not the amount it refused, and a re-check of a single unit
/// cannot reproduce a stop whose dimension still has room. The condition is therefore
/// stated against the live budget: a cancellation or an elapsed-time stop comes back with
/// the budget's own counts through `poll`, and a counted dimension keeps the reader's own
/// dimension with its live limit and consumption plus the smallest request that dimension
/// refuses. The condition is never guessed and never narrowed to a different dimension.
fn budget_stop(ctx: &mut ScanContext<'_>, facts: &MethodCodeFacts) -> Error {
    let budget = ctx.budget();
    if let Err(error) = budget.poll() {
        return error;
    }
    let dimension = match &facts.execution {
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded { dimension },
            ..
        } => *dimension,
        _ => {
            return Error::invalid_input("classfile_bytecode_failed", stop_description(facts));
        }
    };
    let (limit, consumed) = dimension_counts(budget.limits(), &budget.usage(), dimension);
    Error::BudgetExceeded {
        dimension,
        limit,
        consumed,
        requested: limit.saturating_sub(consumed).saturating_add(1),
    }
}

/// Live limit and consumption of one budget dimension.
fn dimension_counts(
    limits: &Limits,
    usage: &UsageSnapshot,
    dimension: BudgetDimension,
) -> (u64, u64) {
    match dimension {
        BudgetDimension::InputBytes => (limits.input_bytes, usage.input_bytes),
        BudgetDimension::ArchiveEntries => (limits.archive_entries, usage.archive_entries),
        BudgetDimension::EntryBytes => (limits.entry_bytes, usage.entry_bytes),
        BudgetDimension::ReadBytes => (limits.read_bytes, usage.read_bytes),
        BudgetDimension::ClassBytes => (limits.class_bytes, usage.class_bytes),
        BudgetDimension::AttributeBytes => (limits.attribute_bytes, usage.attribute_bytes),
        BudgetDimension::CodeBytes => (limits.code_bytes, usage.code_bytes),
        BudgetDimension::ResultItems => (limits.result_items, usage.result_items),
        BudgetDimension::OutputBytes => (limits.output_bytes, usage.output_bytes),
        BudgetDimension::NestedDepth => (limits.nested_depth, usage.nested_depth),
        BudgetDimension::ElapsedMillis => (limits.elapsed_millis, usage.elapsed_millis),
    }
}

/// Stable code of the stop a method body reported.
fn stop_code(facts: &MethodCodeFacts) -> &str {
    match facts.stopped_at.as_ref() {
        Some(BytecodeStop::Instructions { code, .. })
        | Some(BytecodeStop::ExceptionHandlers { code, .. }) => code,
        None => "classfile_bytecode_failed",
    }
}

/// Where and why a method body stopped, for its diagnostic and its terminal error.
fn stop_description(facts: &MethodCodeFacts) -> String {
    match facts.stopped_at.as_ref() {
        Some(BytecodeStop::Instructions { bci, code, .. }) => {
            format!("instruction decoding stopped at BCI {bci} with {code}")
        }
        Some(BytecodeStop::ExceptionHandlers { ordinal, code, .. }) => {
            format!("exception-table decoding stopped at handler {ordinal} with {code}")
        }
        None => "code decoding stopped before completion without a recorded position".to_owned(),
    }
}

/// Member-level diagnostic of a body that stopped early.
///
/// The stop makes this unit's use-site list incomplete, which is a different claim from
/// the code consumer's "this member was not scanned completely": a dynamic site that was
/// never reached cannot be reported, and the report must not read as a complete graph.
fn stop_diagnostic(index: usize, method: &PhysicalMethodId, facts: &MethodCodeFacts) -> Diagnostic {
    Diagnostic {
        code: CODE_STOPPED_CODE.to_owned(),
        severity: DiagnosticSeverity::Error,
        message: format!(
            "member {}:{} decoded only part of its body, so this class's dynamic use-sites are not a complete list: {}",
            method.name.0.escape_ascii(),
            method.descriptor.0.escape_ascii(),
            stop_description(facts)
        ),
        provenance: Some(Provenance {
            location: Location::Attribute {
                owner: method.owner.clone(),
                path: format!("methods[{index}].Code"),
                span: facts.code_span.clone(),
            },
        }),
    }
}

// ---------------------------------------------------------------------------
// The class-level bootstrap table
// ---------------------------------------------------------------------------

/// Locates the class-level `BootstrapMethods` attribute one use-site needs.
///
/// A dynamic site requires exactly that attribute (JVMS 4.7.23), and a class declares at
/// most one of them. With two, no single table could be meant, so this consumer refuses to
/// pick one instead of taking the first.
fn bootstrap_shell<'a>(
    ctx: &mut ScanContext<'_>,
    facts: &'a ClassFacts,
    site: &UseSite,
) -> Result<&'a AttributeShell> {
    let mut shells = facts
        .attributes
        .iter()
        .filter(|shell| shell.name.raw().0.as_slice() == BOOTSTRAP_METHODS);
    let first = shells.next();
    let duplicate = shells.next().is_some();
    if duplicate {
        let message = format!(
            "dynamic entry {} is consumed at BCI {} but the class declares more than one BootstrapMethods attribute",
            site.entry, site.bci
        );
        ctx.push_diagnostic(site_diagnostic(
            ATTRIBUTE_DUPLICATE_CODE,
            DiagnosticSeverity::Error,
            message.clone(),
            site,
        ));
        return Err(Error::invalid_input(ATTRIBUTE_DUPLICATE_CODE, message));
    }
    match first {
        Some(shell) => Ok(shell),
        None => {
            let message = format!(
                "dynamic entry {} is consumed at BCI {} but the class declares no BootstrapMethods attribute",
                site.entry, site.bci
            );
            ctx.push_diagnostic(site_diagnostic(
                ATTRIBUTE_MISSING_CODE,
                DiagnosticSeverity::Error,
                message.clone(),
                site,
            ));
            Err(Error::invalid_input(ATTRIBUTE_MISSING_CODE, message))
        }
    }
}

/// Reads the class-level bootstrap table once for this unit.
///
/// The reader owns the shape contract of the attribute (a `MethodHandle` per entry and
/// loadable constants per argument), so its error is propagated unchanged; the diagnostic
/// names the use-site that made this consumer read the attribute at all.
fn read_bootstraps(
    ctx: &mut ScanContext<'_>,
    bytes: &[u8],
    shell: &AttributeShell,
    facts: &ClassFacts,
    site: &UseSite,
) -> Result<Vec<crate::classfile::BootstrapMethodFacts>> {
    let read = {
        let budget = ctx.budget();
        bootstrap_methods(bytes, shell, &facts.constant_pool, budget)
    };
    match read {
        Ok(bootstraps) => Ok(bootstraps),
        Err(error) => {
            let message = format!(
                "the BootstrapMethods attribute of entry {} could not be read: {error}",
                site.entry
            );
            ctx.push_diagnostic(site_diagnostic(
                MALFORMED_CODE,
                DiagnosticSeverity::Error,
                message,
                site,
            ));
            Err(error)
        }
    }
}

// ---------------------------------------------------------------------------
// The deferred graph
// ---------------------------------------------------------------------------

/// The unit's deferred graph: nodes expanded once, then replayed per use-site.
struct Graph<'a> {
    pool: &'a [CpEntryFacts],
    bootstraps: &'a [crate::classfile::BootstrapMethodFacts],
    /// Expanded dynamic nodes by constant-pool index. This is the visited set that keeps a
    /// shared subgraph from being expanded again, and it is what makes the traversal
    /// terminate without a recursion guard of its own.
    expanded: BTreeMap<u16, DynamicNode>,
    ledger: Ledger,
}

/// One dynamic node of the graph, expanded into the edges that leave it.
struct DynamicNode {
    /// `BootstrapMethods` index that governs this node.
    bootstrap_index: u16,
    /// `CONSTANT_MethodHandle` entry of that bootstrap method.
    method_handle: u16,
    /// Argument entries in declaration order.
    arguments: Vec<NodeArgument>,
}

/// One argument edge of a dynamic node.
#[derive(Clone, Copy)]
struct NodeArgument {
    /// Constant-pool entry of the argument.
    entry: u16,
    /// The dynamic node this argument opens, when the argument is a `CONSTANT_Dynamic`.
    nested: Option<u16>,
}

impl<'a> Graph<'a> {
    fn new(
        pool: &'a [CpEntryFacts],
        bootstraps: &'a [crate::classfile::BootstrapMethodFacts],
    ) -> Self {
        Self {
            pool,
            bootstraps,
            expanded: BTreeMap::new(),
            ledger: Ledger::derived_from(pool.len()),
        }
    }

    fn node(&self, index: u16) -> Option<&DynamicNode> {
        self.expanded.get(&index)
    }

    /// Expands one dynamic node into its bootstrap method handle and its arguments.
    ///
    /// Expansion happens once per unit per entry, so a bootstrap subgraph several sites or
    /// several paths share is read once. It is not recursive: a dynamic argument is only
    /// recorded as a nested node, and descending happens while the graph is walked.
    fn expand(&mut self, ctx: &mut ScanContext<'_>, site: &UseSite, index: u16) -> Result<()> {
        if self.expanded.contains_key(&index) {
            return Ok(());
        }
        let entry = cp_entry(self.pool, index)?;
        let bootstrap_index = match &entry.kind {
            CpEntryKind::Dynamic {
                bootstrap_method_attr_index,
                ..
            }
            | CpEntryKind::InvokeDynamic {
                bootstrap_method_attr_index,
                ..
            } => *bootstrap_method_attr_index,
            _ => {
                return Err(malformed(
                    ctx,
                    site,
                    format!("constant-pool entry {index} is not a dynamic site"),
                ));
            }
        };
        let bootstraps = self.bootstraps;
        let Some(bootstrap) = bootstraps.get(usize::from(bootstrap_index)) else {
            return Err(malformed(
                ctx,
                site,
                format!(
                    "dynamic entry {index} names BootstrapMethods entry {bootstrap_index}, which the attribute does not declare"
                ),
            ));
        };
        let method_handle = bootstrap.method_ref;
        self.ledger.charge_node(ctx, site, index)?;
        self.ledger.charge_node(ctx, site, method_handle)?;
        self.ledger.charge_edge(ctx, site)?;
        let mut arguments = Vec::new();
        for &argument in &bootstrap.arguments {
            self.ledger.charge_node(ctx, site, argument)?;
            self.ledger.charge_edge(ctx, site)?;
            let nested = match &cp_entry(self.pool, argument)?.kind {
                CpEntryKind::Dynamic { .. } => Some(argument),
                _ => None,
            };
            arguments.push(NodeArgument {
                entry: argument,
                nested,
            });
        }
        self.expanded.insert(
            index,
            DynamicNode {
                bootstrap_index,
                method_handle,
                arguments,
            },
        );
        Ok(())
    }
}

/// Work bound of one unit's graph, derived from that unit's own constant pool.
///
/// Every node and every edge of this graph lands on one constant-pool entry, so the entry
/// count is the size the traversal may grow to. The node bound cannot be exceeded while
/// [`Graph::expanded`] counts each entry once, which is the point: it is the traversal's
/// proven outer bound and it stops a traversal that would count an entry twice. The edge
/// bound is the one a pathological graph really reaches — a long nested-condy chain, or one
/// bootstrap repeating its arguments, needs more edges than the class has entries.
struct Ledger {
    /// The unit's constant-pool entry count.
    limit: usize,
    /// Entries already counted as nodes.
    nodes: BTreeSet<u16>,
    /// Edges already counted.
    edges: usize,
}

impl Ledger {
    fn derived_from(entries: usize) -> Self {
        Self {
            limit: entries,
            nodes: BTreeSet::new(),
            edges: 0,
        }
    }

    fn charge_node(&mut self, ctx: &mut ScanContext<'_>, site: &UseSite, entry: u16) -> Result<()> {
        if self.nodes.insert(entry) && self.nodes.len() > self.limit {
            return Err(self.stop(ctx, site, "nodes", self.nodes.len()));
        }
        Ok(())
    }

    fn charge_edge(&mut self, ctx: &mut ScanContext<'_>, site: &UseSite) -> Result<()> {
        self.edges += 1;
        if self.edges > self.limit {
            return Err(self.stop(ctx, site, "edges", self.edges));
        }
        Ok(())
    }

    fn charge_depth(&self, ctx: &mut ScanContext<'_>, site: &UseSite, depth: usize) -> Result<()> {
        if depth > self.limit {
            return Err(self.stop(ctx, site, "path hops", depth));
        }
        Ok(())
    }

    /// Reports the derived bound instead of expanding an unbounded graph.
    fn stop(&self, ctx: &mut ScanContext<'_>, site: &UseSite, what: &str, reached: usize) -> Error {
        let message = format!(
            "the deferred graph reached {reached} {what}, more than the {} constant-pool entries this class holds, so the traversal stopped",
            self.limit
        );
        ctx.push_diagnostic(site_diagnostic(
            GRAPH_LIMIT_CODE,
            DiagnosticSeverity::Error,
            message.clone(),
            site,
        ));
        Error::invalid_input(GRAPH_LIMIT_CODE, message)
    }
}

// ---------------------------------------------------------------------------
// Walking the graph: one use-site at a time
// ---------------------------------------------------------------------------

/// One open graph node of a use-site walk.
#[derive(Clone, Copy)]
struct Frame {
    node: u16,
    /// Next argument slot of this node to follow.
    cursor: usize,
    /// Path length before this frame's own incoming hop, restored when the frame ends.
    path_len: usize,
}

/// The walk of one use-site: its current path and the nodes it already published.
struct Walk {
    /// `via` hops from the use-site to the node being published.
    path: Vec<BootstrapVia>,
    /// Dynamic nodes on the current path: meeting one again is a cycle.
    on_path: BTreeSet<u16>,
    /// Dynamic nodes whose node facts this use-site already published. A shared subgraph
    /// is published once, while every edge to it keeps its own path in the result.
    published: BTreeSet<u16>,
    frames: Vec<Frame>,
}

/// One use-site being walked: the instruction that opens the graph, and the class-level
/// attribute that recorded it.
struct OpenSite<'a> {
    site: &'a UseSite,
    shell: &'a AttributeShell,
}

/// Publishes the graph facts of one use-site, depth first, in declaration order.
fn emit_site(
    ctx: &mut ScanContext<'_>,
    graph: &mut Graph<'_>,
    open: &OpenSite<'_>,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    let mut walk = Walk {
        // The use-site itself is the first hop of every path this walk records.
        path: vec![BootstrapVia {
            constant_pool_index: open.site.entry,
            bootstrap_index: None,
            argument_index: None,
        }],
        on_path: BTreeSet::new(),
        published: BTreeSet::new(),
        frames: Vec::new(),
    };
    walk.enter(ctx, graph, open, open.site.entry, 0, out)?;
    walk.run(ctx, graph, open, out)
}

impl Walk {
    /// Walks the rest of the graph the root opened.
    fn run(
        &mut self,
        ctx: &mut ScanContext<'_>,
        graph: &mut Graph<'_>,
        open: &OpenSite<'_>,
        out: &mut Vec<XrefItem>,
    ) -> Result<()> {
        while let Some(frame) = self.frames.last().copied() {
            let Some(node) = graph.node(frame.node) else {
                return Err(Error::invalid_input(
                    MALFORMED_CODE,
                    format!("dynamic node {} was never expanded", frame.node),
                ));
            };
            if frame.cursor >= node.arguments.len() {
                self.frames.pop();
                self.on_path.remove(&frame.node);
                self.path.truncate(frame.path_len);
                continue;
            }
            let bootstrap_index = node.bootstrap_index;
            let argument = node.arguments[frame.cursor];
            let position = position(frame.cursor)?;
            if let Some(open) = self.frames.last_mut() {
                open.cursor += 1;
            }
            // Every edge is a fact of its own: a node shared by several edges keeps each of
            // them locatable, with the path that really reached it.
            self.path.push(BootstrapVia {
                constant_pool_index: argument.entry,
                bootstrap_index: Some(bootstrap_index),
                argument_index: Some(position),
            });
            if let Some(fact) =
                node_fact(graph.pool, argument.entry, XrefOperation::BootstrapArgument)?
            {
                push_fact(ctx, open, &self.path, fact, out)?;
            }
            let Some(child) = argument.nested else {
                self.path.pop();
                continue;
            };
            if self.on_path.contains(&child) {
                // The node is already on this path, so descending again would not
                // terminate. The edge above is still a fact; the cycle is reported.
                return Err(cycle_error(ctx, open.site, &self.path, child));
            }
            if self.published.contains(&child) {
                // Shared subgraph: this use-site already published the node's facts, so only
                // this edge's own path is new.
                self.path.pop();
                continue;
            }
            let path_len = self.path.len() - 1;
            self.enter(ctx, graph, open, child, path_len, out)?;
        }
        Ok(())
    }

    /// Enters one dynamic node: expands it, publishes its bootstrap method edge, and opens
    /// its frame so its arguments follow in order.
    fn enter(
        &mut self,
        ctx: &mut ScanContext<'_>,
        graph: &mut Graph<'_>,
        open: &OpenSite<'_>,
        index: u16,
        path_len: usize,
        out: &mut Vec<XrefItem>,
    ) -> Result<()> {
        graph.ledger.charge_node(ctx, open.site, index)?;
        graph.ledger.charge_depth(ctx, open.site, self.path.len())?;
        graph.expand(ctx, open.site, index)?;
        let Some(node) = graph.node(index) else {
            return Err(Error::invalid_input(
                MALFORMED_CODE,
                format!("dynamic node {index} could not be expanded"),
            ));
        };
        let bootstrap_index = node.bootstrap_index;
        let method_handle = node.method_handle;
        self.published.insert(index);
        self.on_path.insert(index);
        self.path.push(BootstrapVia {
            constant_pool_index: method_handle,
            bootstrap_index: Some(bootstrap_index),
            argument_index: None,
        });
        if let Some(fact) = node_fact(graph.pool, method_handle, XrefOperation::BootstrapMethod)? {
            push_fact(ctx, open, &self.path, fact, out)?;
        }
        self.path.pop();
        self.frames.push(Frame {
            node: index,
            cursor: 0,
            path_len,
        });
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Node facts
// ---------------------------------------------------------------------------

/// One graph node fact before the request-target filter is applied.
struct NodeFact {
    operation: XrefOperation,
    /// The constant-pool entry the item describes.
    entry: u16,
    span: ByteSpan,
    symbol: Option<SymbolRef>,
    literal: Option<LiteralValue>,
    /// Descriptor the described entry carries, when its types are a claim of their own.
    descriptor: Option<EntryDescriptor>,
}

/// The fact one graph node carries, when its entry names something reportable.
///
/// The described entry and its span come from the constant pool, so a consumer can slice
/// the entry out of the class bytes and check its tag: a `MethodHandle`'s reference kind
/// and a `MethodType`'s descriptor are read there rather than claimed here.
fn node_fact(
    pool: &[CpEntryFacts],
    index: u16,
    operation: XrefOperation,
) -> Result<Option<NodeFact>> {
    let entry = cp_entry(pool, index)?;
    let (symbol, literal, descriptor) = match &entry.kind {
        // A method handle is a node of its own; what it names is the member it references,
        // and that member's descriptor is the descriptor this node carries.
        CpEntryKind::MethodHandle {
            reference_index, ..
        } => {
            let referenced = cp_entry(pool, *reference_index)?;
            (
                member_symbol(&referenced.kind),
                None,
                entry_descriptor(&referenced.kind),
            )
        }
        // A dynamic site is a site with dependencies, never a value: its symbol is its own
        // name and descriptor, and its descriptor is the field (condy) or method
        // (`invokedynamic`) descriptor the entry declares.
        kind @ CpEntryKind::Dynamic {
            name, descriptor, ..
        }
        | kind @ CpEntryKind::InvokeDynamic {
            name, descriptor, ..
        } => (
            Some(SymbolRef::Method {
                owner: JvmBytes(Vec::new()),
                name: name.clone(),
                descriptor: descriptor.clone(),
            }),
            None,
            entry_descriptor(kind),
        ),
        kind => (
            entry_symbol(kind),
            entry_literal(kind),
            entry_descriptor(kind),
        ),
    };
    if symbol.is_none() && literal.is_none() && descriptor.is_none() {
        return Ok(None);
    }
    Ok(Some(NodeFact {
        operation,
        entry: index,
        span: entry.span.clone(),
        symbol,
        literal,
        descriptor,
    }))
}

/// Symbol an entry names without resolving anything through it.
fn entry_symbol(kind: &CpEntryKind) -> Option<SymbolRef> {
    match kind {
        CpEntryKind::Class { name, .. } => Some(SymbolRef::Class {
            owner: name.clone(),
        }),
        other => member_symbol(other),
    }
}

/// Symbol a member-carrying entry names.
///
/// A handle that references anything else names no member, and this consumer reports no
/// symbol for it rather than inventing one.
fn member_symbol(kind: &CpEntryKind) -> Option<SymbolRef> {
    match kind {
        CpEntryKind::FieldRef {
            owner,
            name,
            descriptor,
            ..
        } => Some(SymbolRef::Field {
            owner: owner.clone(),
            name: name.clone(),
            descriptor: descriptor.clone(),
        }),
        CpEntryKind::MethodRef {
            owner,
            name,
            descriptor,
            ..
        }
        | CpEntryKind::InterfaceMethodRef {
            owner,
            name,
            descriptor,
            ..
        } => Some(SymbolRef::Method {
            owner: owner.clone(),
            name: name.clone(),
            descriptor: descriptor.clone(),
        }),
        _ => None,
    }
}

/// Literal an entry holds, for the entry kinds a bootstrap argument may be.
fn entry_literal(kind: &CpEntryKind) -> Option<LiteralValue> {
    match kind {
        CpEntryKind::String { value, .. } => Some(LiteralValue::String {
            value: value.clone(),
        }),
        // A `MethodType` argument holds a descriptor, and the frozen schema has no
        // descriptor target, so the raw descriptor bytes travel as a string literal while
        // `evidence.constant_pool_index` keeps addressing the `MethodType` entry itself.
        CpEntryKind::MethodType { descriptor, .. } => Some(LiteralValue::String {
            value: descriptor.clone(),
        }),
        CpEntryKind::Class { name, .. } => Some(LiteralValue::Class {
            value: name.clone(),
        }),
        CpEntryKind::Integer { value } => Some(LiteralValue::Integer { value: *value }),
        CpEntryKind::Long { value } => Some(LiteralValue::Long { value: *value }),
        CpEntryKind::Float { bits } => Some(LiteralValue::Float { value: *bits }),
        CpEntryKind::Double { bits } => Some(LiteralValue::Double { value: *bits }),
        _ => None,
    }
}

/// Publishes one node fact, and the descriptor types the same node carries.
///
/// The two products keep their own category, because they are different claims about the
/// same node:
///
/// * the node's own item — the bootstrap method handle, or an argument and the member,
///   value or dynamic site it names — belongs to `Bootstrap`, the category that owns this
///   graph;
/// * the types a reached descriptor names belong to `Type`, whatever category opened the
///   walk, because a type only a bootstrap descriptor names has no other consumer that
///   could report it.
///
/// Each product is published only for a symbol request whose target it answers, so a
/// `Bootstrap`-only request produces no type item, and a `Type`-only request produces no
/// node item.
fn push_fact(
    ctx: &ScanContext<'_>,
    open: &OpenSite<'_>,
    path: &[BootstrapVia],
    fact: NodeFact,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    if ctx.wants(ConsumerKind::Bootstrap) && answers(&ctx.request().target, &fact) {
        out.push(XrefItem {
            relation: ctx.request().relation,
            source: Provenance {
                location: Location::Code {
                    method: open.site.method.clone(),
                    bci: open.site.bci,
                },
            },
            target: item_target(&ctx.request().target),
            consumer: Some(ConsumerKind::Bootstrap),
            operation: fact.operation,
            derivation: XrefDerivation::BootstrapEdge,
            certainty: XrefCertainty::Exact,
            resolution: QueryResolution::NotRequested,
            evidence: node_evidence(open, &fact, path),
        });
    }
    let Some(descriptor) = &fact.descriptor else {
        return Ok(());
    };
    if !ctx.wants(ConsumerKind::Type) {
        return Ok(());
    }
    for name in descriptor_types(&descriptor.descriptor.0, descriptor.kind)? {
        if !asks_class(&ctx.request().target, &name) {
            continue;
        }
        out.push(XrefItem {
            relation: ctx.request().relation,
            source: Provenance {
                location: Location::Code {
                    method: open.site.method.clone(),
                    bci: open.site.bci,
                },
            },
            target: XrefTarget::Symbol {
                value: SymbolRef::Class { owner: name },
            },
            consumer: Some(ConsumerKind::Type),
            operation: fact.operation,
            derivation: XrefDerivation::BootstrapEdge,
            certainty: XrefCertainty::Exact,
            resolution: QueryResolution::NotRequested,
            evidence: node_evidence(open, &fact, path),
        });
    }
    Ok(())
}

/// Evidence of one fact read at a graph node: the node entry, the use-site that reached it
/// and the path that did.
fn node_evidence(open: &OpenSite<'_>, fact: &NodeFact, path: &[BootstrapVia]) -> XrefEvidence {
    XrefEvidence {
        constant_pool_index: Some(fact.entry),
        bci: Some(open.site.bci),
        opcode: Some(open.site.opcode),
        attribute: Some(ArchiveNameBytes(open.shell.name.raw().0.clone())),
        span: Some(fact.span.clone()),
        via: path.to_vec(),
    }
}

/// Whether a node fact answers the request target.
fn answers(request: &QueryTarget, fact: &NodeFact) -> bool {
    match request {
        QueryTarget::Symbol { value } => fact.symbol.as_ref() == Some(value),
        QueryTarget::Literal { value } => fact.literal.as_ref() == Some(value),
    }
}

/// Whether a class symbol is the requested symbol.
fn asks_class(request: &QueryTarget, owner: &JvmBytes) -> bool {
    matches!(
        request,
        QueryTarget::Symbol {
            value: SymbolRef::Class { owner: asked }
        } if asked == owner
    )
}

/// The requested target in result form.
fn item_target(target: &QueryTarget) -> XrefTarget {
    match target {
        QueryTarget::Symbol { value } => XrefTarget::Symbol {
            value: value.clone(),
        },
        QueryTarget::Literal { value } => XrefTarget::Literal {
            value: value.clone(),
        },
    }
}

// ---------------------------------------------------------------------------
// Diagnostics and small conversions
// ---------------------------------------------------------------------------

/// Domain diagnostic of one use-site: the instruction whose graph this is.
fn site_diagnostic(
    code: &str,
    severity: DiagnosticSeverity,
    message: String,
    site: &UseSite,
) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        severity,
        message,
        provenance: Some(Provenance {
            location: Location::Code {
                method: site.method.clone(),
                bci: site.bci,
            },
        }),
    }
}

/// A cycle in the deferred graph: report it instead of recursing or stopping silently.
fn cycle_error(
    ctx: &mut ScanContext<'_>,
    site: &UseSite,
    path: &[BootstrapVia],
    repeated: u16,
) -> Error {
    let message = format!(
        "the deferred graph of this use-site is cyclic: constant-pool entry {repeated} is reached again after {} path hops, so the traversal stops at the repeated node",
        path.len()
    );
    ctx.push_diagnostic(site_diagnostic(
        CYCLE_CODE,
        DiagnosticSeverity::Warning,
        message.clone(),
        site,
    ));
    Error::invalid_input(CYCLE_CODE, message)
}

/// A reference that does not match the declared bootstrap structure.
fn malformed(ctx: &mut ScanContext<'_>, site: &UseSite, message: String) -> Error {
    ctx.push_diagnostic(site_diagnostic(
        MALFORMED_CODE,
        DiagnosticSeverity::Error,
        message.clone(),
        site,
    ));
    Error::invalid_input(MALFORMED_CODE, message)
}

/// Narrows a declaration position to the `u16` the schema records. The reader already
/// bounded every `BootstrapMethods` count to a `u16`, so this is a total conversion.
fn position(value: usize) -> Result<u16> {
    u16::try_from(value).map_err(|_| {
        Error::invalid_input(
            MALFORMED_CODE,
            "bootstrap argument position does not fit u16",
        )
    })
}
