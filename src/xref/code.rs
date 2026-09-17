//! Code consumer: raw constant-pool candidates (X0) and instruction-level references
//! (X1).
//!
//! One reader pass over a unit's class bytes answers both products, and the two are
//! deliberately different claims:
//!
//! * `constant_pool_contains` is the raw constant-pool probe. It answers with
//!   `XrefDerivation::ConstantPoolCandidate`, `consumer: None` and
//!   `XrefOperation::ConstantPoolEntry`: the pool entry exists, and that is all the item
//!   says. A pool entry nobody consumes produces no consumer, no BCI and no call, so it
//!   never appears in a `mentions_symbol` result (acceptance A01).
//! * `mentions_symbol` and `literal_value` are the structural consumer scan. A reference
//!   exists only where an instruction or an exception handler really consumes an entry:
//!   `invoke*`, `get*`/`put*`, `new`/`anewarray`/`multianewarray`/`checkcast`/
//!   `instanceof`, `ldc`/`ldc_w`/`ldc2_w`, `invokedynamic`, and the `Code` exception
//!   table. Those items carry `XrefDerivation::StructuralConsumer`, the consumer
//!   category, the operation, and evidence with the constant-pool index, BCI, opcode and
//!   class-file span. Instructions are scanned linearly: a reference in unreachable code
//!   is a fact about these bytes, so no control-flow graph, dead-code elimination or
//!   execution trace is built here (acceptance A17).
//!
//! # Two products of one instruction
//!
//! One instruction can carry two different claims, and each is gated by its own category:
//!
//! * the **use site** belongs to the opcode family's consumer (`Invocation`, `Field`,
//!   `Type`, `Constant`, `Exception`), which is which opcode the instruction is;
//! * the **descriptor types** belong to `ConsumerKind::Type`, and exist only for an entry
//!   the instruction really consumed *and* that carries a descriptor at all: the `ldc`
//!   family over a `MethodType` (its method descriptor), over a `MethodHandle` (the
//!   descriptor of the member its `reference_index` names) and over a `Dynamic` (its
//!   **field** descriptor, JVMS 4.4.10), plus `invokedynamic` over an `InvokeDynamic`
//!   (its method descriptor). An ordinary `invoke*`/field instruction names its member
//!   directly, so its descriptor stays that member's own declaration and is not a type
//!   reference of this instruction; a `Class` entry names the type itself, which the
//!   `Type` category already answers as the instruction's own symbol.
//!
//! Both products carry this instruction's coordinates — the constant-pool index the
//! instruction read, its BCI, opcode, attribute and span — so a caller can slice the
//! instruction out of the class bytes for either one. A `Type`-only request triggers the
//! reads it needs itself: it never depends on `Invocation`, `Constant` or `Bootstrap`
//! being requested too. A descriptor of a consumed entry that does not parse is a
//! structured stop (`query_descriptor_malformed`, owned by the reader facts) and happens
//! only when the `Type` category was requested at all.
//!
//! # Matching
//!
//! Which candidate answers a scan is the scan context's decision
//! ([`super::ScanContext::candidate_matches`]), not this stream's: the filter the scan was
//! opened with decides it and the published target is the one that decision returns, so a
//! candidate this stream finds can only appear with the symbol the instruction really names.
//! Under a query's exact target the rule is equality on the raw bytes the class file stores:
//! an owner is an internal name, a descriptor its raw bytes, and no dimension is normalised.
//! A `SymbolRef::Method` is answered by both `CONSTANT_Methodref` and
//! `CONSTANT_InterfaceMethodref`, because class-versus-interface method reference is a
//! resolution detail and not one of the three raw dimensions the query compares. An
//! absent dimension is not a wildcard: `NameAndType`, `Dynamic` and `InvokeDynamic`
//! carry no owner, so only an equally empty owner dimension matches them.
//!
//! `invokedynamic` has no owner at all: its symbol is the `NameAndType` name and
//! descriptor of the dynamic site, and the owner dimension takes part as empty. The same
//! rule covers the `Dynamic` (condy) and `NameAndType` entries a raw pool probe sees.
//!
//! Nothing here expands an owner through a hierarchy. A `CONSTANT_Methodref` whose owner
//! is a subclass is not a reference to the inherited method, so `mentions_symbol` reports
//! a miss when the pool owner differs from the queried owner. That candidate expansion is
//! the P2 definition resolver's job behind `references_definition`; keeping the two apart
//! is the boundary acceptance A11 draws. A member-shaped candidate filter works from the
//! other side of that boundary: it matches on the raw name and descriptor and keeps the
//! owner the instruction spells, so comparing that owner with a declaration's is the
//! caller's resolution step and never this stream's.
//!
//! # Cost and order
//!
//! A unit's bytes are read only when the request needs them at all: the raw pool probe
//! needs them for a class candidate entry or the standalone CLASS root, the consumer scan
//! needs them only when a code consumer category is requested, and no other entry is
//! touched. Which archive entries are class candidates, and what a candidate whose bytes
//! are not a class file means, is owned by [`super::class_content`]; this module only
//! decides *when* it needs the bytes. Exactly one attribute content per method is read —
//! `Code` — and this module never charges `ResultItems` (the scan orchestrator owns that
//! budget). A request that names `Type` alone is a consumer scan like any other: it reads
//! the same bodies for the same instructions.
//!
//! Items come out in ascending class-file position: constant-pool entries by index, and
//! per method the instructions by BCI followed by the exception table records by ordinal
//! (the table follows the code array inside the same `Code` attribute). One instruction
//! publishes its use-site item first and then the descriptor types it consumed, in the
//! descriptor's own first-appearance order. The page cursor in `super` depends on that
//! order.

use super::{ScanContext, ScanUnit, UnitContent, class_content, to_u64};
use crate::budget::CountedBudgetDimension;
use crate::classfile::{
    BytecodeStop, ClassFacts, CpEntryFacts, CpEntryKind, EntryDescriptor, ExceptionHandlerFact,
    InstructionFact, MethodCodeFacts, class_facts, cp_class_name, cp_entry, descriptor_types,
    entry_descriptor, method_code_facts,
};
use crate::error::{Error, Result};
use crate::model::{
    ArchiveNameBytes, ByteSpan, ClassBytesId, Diagnostic, DiagnosticSeverity, ExecutionReport,
    JvmBytes, Location, PhysicalDefinitionId, PhysicalMethodId, Provenance, SymbolRef,
    TerminationReason,
};
use crate::query::{
    ConsumerKind, LiteralValue, QueryRelation, QueryResolution, QueryTarget, XrefCertainty,
    XrefDerivation, XrefEvidence, XrefItem, XrefOperation, XrefTarget,
};

/// Attribute name of the only attribute this stream reads.
const CODE_ATTRIBUTE: &[u8] = b"Code";

/// Consumer categories the instruction scan produces.
const CODE_CATEGORIES: [ConsumerKind; 5] = [
    ConsumerKind::Invocation,
    ConsumerKind::Field,
    ConsumerKind::Type,
    ConsumerKind::Constant,
    ConsumerKind::Exception,
];

pub(super) fn scan(
    ctx: &mut ScanContext<'_>,
    unit: &ScanUnit,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    let relation = ctx.request().relation;
    // The raw pool probe answers about pool entries, not about consumers, so it is
    // category-independent: `super` already refused a request that names no category at
    // all, and a probe must not silently become "no scan" because the caller named a
    // category the pool does not use.
    let pool_probe = relation == QueryRelation::ConstantPoolContains;
    let consumer_scan = matches!(
        relation,
        QueryRelation::MentionsSymbol | QueryRelation::LiteralValue
    ) && CODE_CATEGORIES.iter().any(|kind| ctx.wants(*kind));
    if !pool_probe && !consumer_scan {
        return Ok(());
    }
    // The shared rule, the read and the damaged-candidate failure are `super`'s: this
    // stream only appears here once it really needs class bytes.
    let Some(content) = class_content(ctx, unit)? else {
        return Ok(());
    };
    let facts = {
        let budget = ctx.budget();
        class_facts(&content.bytes, budget)?
    };
    let definition = unit.definition(ClassBytesId {
        digest: content.digest.clone(),
        length: to_u64(content.bytes.len())?,
    });
    if pool_probe {
        for entry in &facts.constant_pool {
            emit_pool_candidate(ctx, &definition, entry, out);
        }
        return Ok(());
    }
    scan_methods(ctx, &content, &facts, &definition, out)
}

/// Emits one raw constant-pool candidate per entry that carries the requested target.
fn emit_pool_candidate(
    ctx: &ScanContext<'_>,
    definition: &PhysicalDefinitionId,
    entry: &CpEntryFacts,
    out: &mut Vec<XrefItem>,
) {
    if !entry_answers(&ctx.request().target, entry) {
        return;
    }
    out.push(XrefItem {
        relation: ctx.request().relation,
        source: Provenance {
            location: Location::ClassOffset {
                definition: definition.clone(),
                offset: entry.span.start,
            },
        },
        target: item_target(&ctx.request().target),
        consumer: None,
        operation: XrefOperation::ConstantPoolEntry,
        derivation: XrefDerivation::ConstantPoolCandidate,
        certainty: XrefCertainty::Exact,
        resolution: QueryResolution::NotRequested,
        evidence: XrefEvidence {
            constant_pool_index: Some(entry.index),
            bci: None,
            opcode: None,
            attribute: None,
            span: Some(entry.span.clone()),
            via: Vec::new(),
        },
    });
}

/// Whether a constant-pool entry carries the requested target.
fn entry_answers(request: &QueryTarget, entry: &CpEntryFacts) -> bool {
    match request {
        QueryTarget::Symbol { value } => entry_symbol(entry).as_ref() == Some(value),
        QueryTarget::Literal { value } => literal_matches(value, entry),
    }
}

/// Symbol a constant-pool entry carries, using the dimensions the entry has.
///
/// * `Class` names one type: the owner dimension holds the internal name.
/// * `FieldRef`, `MethodRef` and `InterfaceMethodRef` carry the complete
///   owner/name/descriptor triple.
/// * `NameAndType`, `Dynamic` and `InvokeDynamic` carry a name and a descriptor and no
///   owner, so the owner dimension is empty. That is the same rule an `invokedynamic`
///   use site follows: a dynamic site has no owner, and this scan never invents one.
/// * `MethodHandle` carries an index into another entry rather than a symbol, so it does
///   not participate: the entry it points at carries the symbol and answers on its own
///   index. `String` and `Utf8` carry text, not symbols, and answer literal targets only.
fn entry_symbol(entry: &CpEntryFacts) -> Option<SymbolRef> {
    match &entry.kind {
        CpEntryKind::Class { name, .. } => Some(SymbolRef::Class {
            owner: name.clone(),
        }),
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
        CpEntryKind::NameAndType {
            name, descriptor, ..
        }
        | CpEntryKind::Dynamic {
            name, descriptor, ..
        }
        | CpEntryKind::InvokeDynamic {
            name, descriptor, ..
        } => Some(SymbolRef::Method {
            owner: JvmBytes(Vec::new()),
            name: name.clone(),
            descriptor: descriptor.clone(),
        }),
        _ => None,
    }
}

/// Whether a literal target matches a constant-pool entry.
///
/// Byte values are compared against every representation the raw pool probe covers,
/// `Utf8`, `Class` and `String`, because the same bytes are spelled by three entry kinds
/// and the probe is about the bytes rather than the tag. `Integer` and `Long` compare the
/// JVM value and `Float`/`Double` the bit pattern, so `NaN` and `-0.0` keep their exact
/// spelling.
fn literal_matches(target: &LiteralValue, entry: &CpEntryFacts) -> bool {
    match target {
        LiteralValue::String { value } | LiteralValue::Class { value } => {
            let expected = value.0.as_slice();
            match &entry.kind {
                CpEntryKind::Utf8 { bytes }
                | CpEntryKind::Class { name: bytes, .. }
                | CpEntryKind::String { value: bytes, .. } => bytes.0.as_slice() == expected,
                _ => false,
            }
        }
        LiteralValue::Integer { value } => {
            matches!(&entry.kind, CpEntryKind::Integer { value: found } if found == value)
        }
        LiteralValue::Long { value } => {
            matches!(&entry.kind, CpEntryKind::Long { value: found } if found == value)
        }
        LiteralValue::Float { value } => {
            matches!(&entry.kind, CpEntryKind::Float { bits } if bits == value)
        }
        LiteralValue::Double { value } => {
            matches!(&entry.kind, CpEntryKind::Double { bits } if bits == value)
        }
    }
}

/// Scans every method of one class file for consumed references.
///
/// A member whose code cannot be decoded completely is reported as a member-level
/// diagnostic and does not hide the remaining members: their references are still facts,
/// and the scan ends the unit with the terminal error so the report stays `Partial`
/// instead of claiming a complete scan it did not perform. A budget or cancellation stop
/// returns immediately, because no further read in this request can succeed.
fn scan_methods(
    ctx: &mut ScanContext<'_>,
    content: &UnitContent,
    facts: &ClassFacts,
    definition: &PhysicalDefinitionId,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    let mut pending: Option<Error> = None;
    for (index, method) in facts.methods.iter().enumerate() {
        // An abstract or native member declares no body. That absence is normal, so it is
        // skipped rather than reported as a member that failed to decode.
        if !has_code(method) {
            continue;
        }
        let code = {
            let budget = ctx.budget();
            method_code_facts(&content.bytes, method, budget)?
        };
        let method_id = PhysicalMethodId {
            owner: definition.clone(),
            name: method.name.raw().clone(),
            descriptor: method.descriptor.raw().clone(),
        };
        emit_instructions(ctx, &facts.constant_pool, &method_id, &code, out)?;
        emit_handlers(ctx, index, definition, &facts.constant_pool, &code, out)?;
        match code_stop(ctx, &code) {
            Some(CodeStop::Budget(error)) => return Err(error),
            Some(CodeStop::Decode(error)) => {
                ctx.push_diagnostic(stop_diagnostic(index, definition, &method_id, &code)?);
                if pending.is_none() {
                    pending = Some(error);
                }
            }
            None => {}
        }
    }
    match pending {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/// Whether a member declares a `Code` attribute at all.
///
/// Checked from the attribute shells, so the decision costs no read: an abstract or
/// native member has no instruction stream, and asking the reader to decode one would
/// turn a normal declaration into a decode failure.
fn has_code(method: &crate::classfile::MemberHeader) -> bool {
    method
        .attributes
        .iter()
        .any(|shell| shell.name.raw().0.as_slice() == CODE_ATTRIBUTE)
}

/// Emits one item per instruction that consumes the requested target.
///
/// One instruction publishes its use-site item first, and then one item per distinct object
/// type of the descriptor it consumed. The two are different claims with different targets
/// (a member or value, and a class), so a request answers whichever of them it names. Which
/// candidate answers the scan is the context's decision (its filter), not this stream's: the
/// published target is the one that decision returns, which under an exact target is the
/// target the caller asked for and under a member shape is the symbol this instruction
/// really names.
fn emit_instructions(
    ctx: &ScanContext<'_>,
    pool: &[CpEntryFacts],
    method: &PhysicalMethodId,
    code: &MethodCodeFacts,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    for instruction in &code.instructions {
        let Some(site) = instruction_use(ctx, pool, instruction)? else {
            continue;
        };
        if let Some(target) = ctx.published_target(site.symbol.as_ref(), site.literal.as_ref()) {
            out.push(XrefItem {
                relation: ctx.request().relation,
                source: Provenance {
                    location: Location::Code {
                        method: method.clone(),
                        bci: instruction.bci,
                    },
                },
                target,
                consumer: Some(site.consumer),
                operation: site.operation,
                derivation: XrefDerivation::StructuralConsumer,
                certainty: XrefCertainty::Exact,
                resolution: QueryResolution::NotRequested,
                evidence: instruction_evidence(instruction),
            });
        }
        let Some(descriptor) = &site.descriptor else {
            continue;
        };
        for name in descriptor_types(&descriptor.descriptor.0, descriptor.kind)? {
            let symbol = SymbolRef::Class { owner: name };
            if !ctx.candidate_matches(Some(&symbol), None) {
                continue;
            }
            out.push(XrefItem {
                relation: ctx.request().relation,
                source: Provenance {
                    location: Location::Code {
                        method: method.clone(),
                        bci: instruction.bci,
                    },
                },
                target: XrefTarget::Symbol { value: symbol },
                consumer: Some(ConsumerKind::Type),
                operation: site.operation,
                derivation: XrefDerivation::StructuralConsumer,
                certainty: XrefCertainty::Exact,
                resolution: QueryResolution::NotRequested,
                evidence: instruction_evidence(instruction),
            });
        }
    }
    Ok(())
}

/// Evidence of one fact read at an instruction: the instruction's own coordinates.
fn instruction_evidence(instruction: &InstructionFact) -> XrefEvidence {
    XrefEvidence {
        constant_pool_index: instruction.constant_pool_index,
        bci: Some(instruction.bci),
        opcode: Some(instruction.opcode),
        attribute: Some(ArchiveNameBytes(CODE_ATTRIBUTE.to_vec())),
        span: Some(instruction.span.clone()),
        via: Vec::new(),
    }
}

/// Emits the catch types of one method's exception table.
///
/// A `catch_type_index` of 0 is the catch-all handler: it names no type, so it produces
/// no type reference. The evidence keeps the protected range in class-file coordinates
/// and the handler BCI, the location's span addresses that exception table record, and
/// its attribute path names the handler ordinal, so both the ordinal and the protected
/// range can be checked against the class bytes.
fn emit_handlers(
    ctx: &ScanContext<'_>,
    index: usize,
    definition: &PhysicalDefinitionId,
    pool: &[CpEntryFacts],
    code: &MethodCodeFacts,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    if !ctx.wants(ConsumerKind::Exception) {
        return Ok(());
    }
    for handler in &code.exception_handlers {
        let Some(catch_type_index) = handler.catch_type_index else {
            continue;
        };
        let caught = SymbolRef::Class {
            owner: cp_class_name(pool, catch_type_index)?,
        };
        let Some(target) = ctx.published_target(Some(&caught), None) else {
            continue;
        };
        out.push(XrefItem {
            relation: ctx.request().relation,
            source: Provenance {
                location: Location::Attribute {
                    owner: definition.clone(),
                    path: handler_path(index, handler.ordinal),
                    span: handler_record_span(&code.code_span, handler.ordinal)?,
                },
            },
            target,
            consumer: Some(ConsumerKind::Exception),
            operation: XrefOperation::ExceptionHandler,
            derivation: XrefDerivation::StructuralConsumer,
            certainty: XrefCertainty::Exact,
            resolution: QueryResolution::NotRequested,
            evidence: XrefEvidence {
                constant_pool_index: Some(catch_type_index),
                bci: Some(handler.handler_bci),
                opcode: None,
                attribute: Some(ArchiveNameBytes(CODE_ATTRIBUTE.to_vec())),
                span: Some(protected_span(&code.code_span, handler)?),
                via: Vec::new(),
            },
        });
    }
    Ok(())
}

/// One reference an instruction really consumes.
struct UseSite {
    operation: XrefOperation,
    consumer: ConsumerKind,
    /// Symbol the use site names; answers `mentions_symbol`.
    symbol: Option<SymbolRef>,
    /// Value the use site consumes; answers `literal_value`.
    literal: Option<LiteralValue>,
    /// Descriptor the same instruction consumed, when its entry carries one.
    descriptor: Option<EntryDescriptor>,
}

/// Member shape an opcode family names.
#[derive(Clone, Copy, Eq, PartialEq)]
enum MemberKind {
    Field,
    Method,
}

/// Reference shape an opcode family consumes.
#[derive(Clone, Copy)]
enum Shape {
    Member(MemberKind),
    /// A `CONSTANT_Class` entry naming a type.
    Type,
    /// A `CONSTANT_InvokeDynamic` entry naming a dynamic call site.
    DynamicSite,
    /// Any loadable constant an `ldc` family instruction can consume.
    Loadable,
}

/// Operation, consumer category and expected entry shape of one opcode.
#[derive(Clone, Copy)]
struct OpcodeFamily {
    operation: XrefOperation,
    consumer: ConsumerKind,
    shape: Shape,
}

/// Classifies one instruction and resolves the constant-pool entry it consumes.
///
/// One instruction produces up to two products, each gated by its own category: the use
/// site of the opcode family's consumer, and the descriptor types of the entry that family
/// consumes (`Type`). The opcode, not the entry, decides the consumer category, so an
/// instruction of a category this request does not want is skipped before its entry is
/// resolved: a malformed reference of an unrequested category cannot fail a query that
/// never asked about it. An opcode whose entry kind cannot carry the reference the opcode
/// family names (a verifier error) yields no use site; the raw entry stays visible to the
/// `constant_pool_contains` probe, and no symbol is invented for it.
fn instruction_use(
    ctx: &ScanContext<'_>,
    pool: &[CpEntryFacts],
    instruction: &InstructionFact,
) -> Result<Option<UseSite>> {
    let Some(family) = opcode_family(instruction.opcode) else {
        return Ok(None);
    };
    let wants_use = ctx.wants(family.consumer);
    // A descriptor type is the `Type` category's claim, whatever consumer the opcode
    // family belongs to, and a request that names neither reads nothing here.
    let wants_types = ctx.wants(ConsumerKind::Type);
    if !wants_use && !wants_types {
        return Ok(None);
    }
    let Some(index) = instruction.constant_pool_index else {
        return Ok(None);
    };
    let entry = cp_entry(pool, index)?;
    let descriptor = if wants_types {
        descriptor_use(pool, entry, family.shape)?
    } else {
        None
    };
    let resolved = if wants_use {
        match family.shape {
            Shape::Member(kind) => member_use(entry, family, kind),
            Shape::Type => type_use(entry, family),
            Shape::DynamicSite => dynamic_use(entry, family),
            Shape::Loadable => load_use(pool, entry, family)?,
        }
    } else {
        None
    };
    let mut site = resolved.unwrap_or(UseSite {
        operation: family.operation,
        consumer: family.consumer,
        symbol: None,
        literal: None,
        descriptor: None,
    });
    site.descriptor = descriptor;
    if site.symbol.is_none() && site.literal.is_none() && site.descriptor.is_none() {
        return Ok(None);
    }
    Ok(Some(site))
}

/// The descriptor the entry one instruction consumed carries, when the instruction's
/// family can consume a descriptor at all.
///
/// The `ldc` family loads a loadable constant and `invokedynamic` names a dynamic site:
/// those are the entries whose descriptor the instruction really uses (`MethodType`,
/// `MethodHandle`, `Dynamic`, `InvokeDynamic`), and a `MethodHandle` carries its
/// descriptor on the member it references. An ordinary `invoke*`/field instruction and a
/// `Class` entry produce none — see the module doc for why.
fn descriptor_use(
    pool: &[CpEntryFacts],
    entry: &CpEntryFacts,
    shape: Shape,
) -> Result<Option<EntryDescriptor>> {
    match shape {
        Shape::Loadable | Shape::DynamicSite => Ok(match &entry.kind {
            CpEntryKind::MethodHandle {
                reference_index, ..
            } => entry_descriptor(&cp_entry(pool, *reference_index)?.kind),
            kind => entry_descriptor(kind),
        }),
        Shape::Member(_) | Shape::Type => Ok(None),
    }
}

/// Operation, consumer category and reference shape of the scanned opcodes.
fn opcode_family(opcode: u8) -> Option<OpcodeFamily> {
    let (operation, consumer, shape) = match opcode {
        // JVMS invoke kinds: the kind is the operation, not a resolution result.
        0xb6 => (
            XrefOperation::InvokeVirtual,
            ConsumerKind::Invocation,
            Shape::Member(MemberKind::Method),
        ),
        0xb7 => (
            XrefOperation::InvokeSpecial,
            ConsumerKind::Invocation,
            Shape::Member(MemberKind::Method),
        ),
        0xb8 => (
            XrefOperation::InvokeStatic,
            ConsumerKind::Invocation,
            Shape::Member(MemberKind::Method),
        ),
        0xb9 => (
            XrefOperation::InvokeInterface,
            ConsumerKind::Invocation,
            Shape::Member(MemberKind::Method),
        ),
        // Field access keeps read/write and static/instance distinct.
        0xb2 => (
            XrefOperation::GetStatic,
            ConsumerKind::Field,
            Shape::Member(MemberKind::Field),
        ),
        0xb3 => (
            XrefOperation::PutStatic,
            ConsumerKind::Field,
            Shape::Member(MemberKind::Field),
        ),
        0xb4 => (
            XrefOperation::GetField,
            ConsumerKind::Field,
            Shape::Member(MemberKind::Field),
        ),
        0xb5 => (
            XrefOperation::PutField,
            ConsumerKind::Field,
            Shape::Member(MemberKind::Field),
        ),
        // Type operations: the entry is the type itself, including array component types.
        0xbb => (XrefOperation::New, ConsumerKind::Type, Shape::Type),
        0xbd => (XrefOperation::NewArray, ConsumerKind::Type, Shape::Type),
        0xc5 => (
            XrefOperation::MultiNewArray,
            ConsumerKind::Type,
            Shape::Type,
        ),
        0xc0 => (XrefOperation::CheckCast, ConsumerKind::Type, Shape::Type),
        0xc1 => (XrefOperation::InstanceOf, ConsumerKind::Type, Shape::Type),
        // Loads consume a value, so they stay `Constant` whatever entry kind they read.
        0x12..=0x14 => (XrefOperation::Ldc, ConsumerKind::Constant, Shape::Loadable),
        0xba => (
            XrefOperation::InvokeDynamic,
            ConsumerKind::Invocation,
            Shape::DynamicSite,
        ),
        _ => return None,
    };
    Some(OpcodeFamily {
        operation,
        consumer,
        shape,
    })
}

/// A member reference consumed by one instruction.
fn member_use(entry: &CpEntryFacts, family: OpcodeFamily, kind: MemberKind) -> Option<UseSite> {
    let symbol = match (kind, &entry.kind) {
        (
            MemberKind::Method,
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
            },
        ) => SymbolRef::Method {
            owner: owner.clone(),
            name: name.clone(),
            descriptor: descriptor.clone(),
        },
        (
            MemberKind::Field,
            CpEntryKind::FieldRef {
                owner,
                name,
                descriptor,
                ..
            },
        ) => SymbolRef::Field {
            owner: owner.clone(),
            name: name.clone(),
            descriptor: descriptor.clone(),
        },
        _ => return None,
    };
    Some(UseSite {
        operation: family.operation,
        consumer: family.consumer,
        symbol: Some(symbol),
        literal: None,
        descriptor: None,
    })
}

/// A type reference consumed by one instruction.
fn type_use(entry: &CpEntryFacts, family: OpcodeFamily) -> Option<UseSite> {
    let CpEntryKind::Class { name, .. } = &entry.kind else {
        return None;
    };
    Some(UseSite {
        operation: family.operation,
        consumer: family.consumer,
        symbol: Some(SymbolRef::Class {
            owner: name.clone(),
        }),
        literal: None,
        descriptor: None,
    })
}

/// A dynamic call site consumed by `invokedynamic`.
///
/// The site names no owner, so the owner dimension of the symbol is empty and only the
/// `NameAndType` name and descriptor take part in matching.
fn dynamic_use(entry: &CpEntryFacts, family: OpcodeFamily) -> Option<UseSite> {
    let CpEntryKind::InvokeDynamic {
        name, descriptor, ..
    } = &entry.kind
    else {
        return None;
    };
    Some(UseSite {
        operation: family.operation,
        consumer: family.consumer,
        symbol: Some(SymbolRef::Method {
            owner: JvmBytes(Vec::new()),
            name: name.clone(),
            descriptor: descriptor.clone(),
        }),
        literal: None,
        descriptor: None,
    })
}

/// A loadable constant consumed by `ldc`/`ldc_w`/`ldc2_w`.
///
/// The entry kind decides what the instruction read: `String`, `Class`, `Integer`,
/// `Long`, `Float` and `Double` are values this scan can compare as literals, a `Class`
/// entry is also a type symbol, and a `MethodHandle` or `Dynamic` entry is a symbolic
/// reference that is resolved here — to the member the handle points at, or to the name
/// and descriptor of the dynamic constant. `MethodType` carries a descriptor and no
/// member, so it yields no symbol: descriptor types belong to the descriptor and
/// signature consumer.
fn load_use(
    pool: &[CpEntryFacts],
    entry: &CpEntryFacts,
    family: OpcodeFamily,
) -> Result<Option<UseSite>> {
    let site = match &entry.kind {
        CpEntryKind::MethodHandle {
            reference_index, ..
        } => {
            let referenced = cp_entry(pool, *reference_index)?;
            entry_symbol(referenced).map(|symbol| UseSite {
                operation: family.operation,
                consumer: family.consumer,
                symbol: Some(symbol),
                literal: None,
                descriptor: None,
            })
        }
        CpEntryKind::Dynamic {
            name, descriptor, ..
        } => Some(UseSite {
            operation: family.operation,
            consumer: family.consumer,
            symbol: Some(SymbolRef::Method {
                owner: JvmBytes(Vec::new()),
                name: name.clone(),
                descriptor: descriptor.clone(),
            }),
            literal: None,
            descriptor: None,
        }),
        _ => {
            let symbol = entry_symbol(entry);
            let literal = load_literal(entry);
            if symbol.is_none() && literal.is_none() {
                None
            } else {
                Some(UseSite {
                    operation: family.operation,
                    consumer: family.consumer,
                    symbol,
                    literal,
                    descriptor: None,
                })
            }
        }
    };
    Ok(site)
}

/// Literal value an `ldc` family instruction reads from one entry kind.
fn load_literal(entry: &CpEntryFacts) -> Option<LiteralValue> {
    match &entry.kind {
        CpEntryKind::String { value, .. } => Some(LiteralValue::String {
            value: value.clone(),
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

/// The requested target in result form.
///
/// The item answers the target the request asked about. The entry that carried the bytes
/// is recovered from the class file through `evidence.constant_pool_index`, which is also
/// what distinguishes a `Utf8` from a `String` or `Class` entry for a byte literal.
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

/// How a method body stopped before the end of its instruction stream.
enum CodeStop {
    /// Budget or cancellation condition: no further read in this request can succeed.
    Budget(Error),
    /// The member's code did not decode completely; the member is reported and the scan
    /// continues with the remaining members.
    Decode(Error),
}

/// Classifies a method body that stopped, if it stopped at all.
///
/// `Partial { Error { .. } }` is exactly how the reader reports a body whose code did not
/// decode; every other non-complete execution is a budget or cancellation condition,
/// which the live budget re-states.
fn code_stop(ctx: &mut ScanContext<'_>, facts: &MethodCodeFacts) -> Option<CodeStop> {
    match &facts.execution {
        ExecutionReport::Complete { .. } => None,
        ExecutionReport::Partial {
            reason: TerminationReason::Error { .. },
            ..
        } => Some(CodeStop::Decode(Error::invalid_input(
            stop_code(facts).to_owned(),
            stop_description(facts),
        ))),
        _ => Some(CodeStop::Budget(live_stop(ctx))),
    }
}

/// The structured error of a budget or cancellation stop.
///
/// `method_code_facts` returns its reliable prefix together with the stop instead of the
/// failing error, so the stop is re-issued as a one-unit `CodeBytes` check against the
/// live budget. `check` polls cancellation and the elapsed limit before the dimension
/// limit, so the reported condition, dimension and counts are the ones that stopped the
/// sub-scan, and `requested` states the re-check's own one-unit request.
fn live_stop(ctx: &mut ScanContext<'_>) -> Error {
    ctx.budget()
        .check(CountedBudgetDimension::CodeBytes, 1)
        .err()
        .unwrap_or_else(|| {
            Error::invalid_input(
                "classfile_bytecode_failed",
                "code decoding stopped without a live budget condition",
            )
        })
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
        Some(BytecodeStop::Instructions {
            bci,
            class_offset,
            code,
        }) => format!(
            "instruction decoding stopped at BCI {bci} (class offset {class_offset}) with {code}"
        ),
        Some(BytecodeStop::ExceptionHandlers {
            ordinal,
            class_offset,
            code,
        }) => format!(
            "exception table decoding stopped at handler {ordinal} (class offset {class_offset}) with {code}"
        ),
        None => "code decoding stopped before completion without a recorded position".to_owned(),
    }
}

/// Member-level diagnostic of a body that could not be decoded completely.
///
/// The provenance names the stopped member — its instruction BCI, or the exception table
/// record — so the member stays locatable even though the terminal diagnostic only names
/// the entry.
fn stop_diagnostic(
    index: usize,
    definition: &PhysicalDefinitionId,
    method: &PhysicalMethodId,
    facts: &MethodCodeFacts,
) -> Result<Diagnostic> {
    let location = match facts.stopped_at.as_ref() {
        Some(BytecodeStop::Instructions { bci, .. }) => Location::Code {
            method: method.clone(),
            bci: *bci,
        },
        Some(BytecodeStop::ExceptionHandlers { ordinal, .. }) => Location::Attribute {
            owner: definition.clone(),
            path: handler_path(index, *ordinal),
            span: handler_record_span(&facts.code_span, *ordinal)?,
        },
        None => Location::Attribute {
            owner: definition.clone(),
            path: format!("methods[{index}].Code"),
            span: facts.code_span.clone(),
        },
    };
    Ok(Diagnostic {
        code: "query_code_stopped".into(),
        severity: DiagnosticSeverity::Error,
        message: format!(
            "member {}:{} was not scanned completely: {}",
            method.name.0.escape_ascii(),
            method.descriptor.0.escape_ascii(),
            stop_description(facts)
        ),
        provenance: Some(Provenance { location }),
    })
}

/// Attribute path of one exception table record inside a method's `Code` attribute.
fn handler_path(method_index: usize, ordinal: u32) -> String {
    format!("methods[{method_index}].Code.exception_handlers[{ordinal}]")
}

/// Class-file span of one exception table record.
///
/// The table follows the instruction array: `code_span` covers the code array, the next
/// two bytes are the table length, and each record is eight bytes — the same layout the
/// reader uses to report a handler-phase stop.
fn handler_record_span(code_span: &ByteSpan, ordinal: u32) -> Result<ByteSpan> {
    let table_start = code_span
        .start
        .checked_add(code_span.length)
        .and_then(|end| end.checked_add(2))
        .ok_or_else(span_overflow)?;
    let offset = u64::from(ordinal)
        .checked_mul(8)
        .ok_or_else(span_overflow)?;
    let start = table_start.checked_add(offset).ok_or_else(span_overflow)?;
    Ok(ByteSpan::new(start, 8))
}

/// Class-file span of the bytecode range one handler protects.
///
/// The range is `[start_bci, end_bci)` in BCI coordinates, translated into class-file
/// coordinates so it can be sliced out of the class bytes directly.
fn protected_span(code_span: &ByteSpan, handler: &ExceptionHandlerFact) -> Result<ByteSpan> {
    let length = handler
        .end_bci
        .checked_sub(handler.start_bci)
        .ok_or_else(|| {
            Error::invalid_input(
                "query_handler_range_invalid",
                "exception handler protected range ends before it starts",
            )
        })?;
    let start = code_span
        .start
        .checked_add(u64::from(handler.start_bci))
        .ok_or_else(span_overflow)?;
    Ok(ByteSpan::new(start, u64::from(length)))
}

fn span_overflow() -> Error {
    Error::invalid_input("query_span_overflow", "classfile span exceeds u64")
}
