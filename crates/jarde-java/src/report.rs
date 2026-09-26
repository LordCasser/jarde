//! The recovery entry point and the report it produces (P3 1.3's public seam).
//!
//! # One run, one request, one artifact
//!
//! [`recover`] takes the P2 payload by reference ([`MethodIr`], the 1.1 handoff) plus the facts the
//! layer below read ([`RecoveryFacts`]), and produces at most one artifact: the Java text, its
//! segment table, a diagnostic list, the six planes that describe what was produced, and
//! [`RecoveryContent`] — what the artifact holds.
//!
//! # The six planes are written, not inferred from each other
//!
//! Each plane has exactly one input, and no plane is derived from another's value (P3 1.2's
//! independent-input table):
//!
//! | plane | this slice writes it from |
//! | --- | --- |
//! | [`Representation`] | whether any region kept bytecode: all Java → `Java`, any fallback → `Mixed` |
//! | [`Quality`] | how strong the recovered structure is: every region structured → `Structured`, any fallback → `Fallback` |
//! | [`SyntaxStatus`] | whether an alias replaced a spelling the source had: any alias → `NotJava`, else `Unchecked` (nothing in this slice runs a syntax check, so `Checked` is never claimed) |
//! | [`CompileStatus`] | `NotAttempted` — only 3.3's controlled recompilation writes anything else |
//! | [`SemanticValidation`] | `Unproven` — this run checks no invariant of its own, and the P2 run's `LocalInvariants` is that run's evidence, not this one's |
//! | [`VerificationStatus`] | `NotPerformed` — nothing verified the artifact |
//!
//! The combinations the P3 spec states are therefore all reachable without any plane being bent to
//! fit another: `Mixed`/`Fallback` for a body with an unprovable region, and `Java`/`NotJava` for a
//! body whose local is named `int` in the source and `int_` in the text.
//!
//! # The content classification is read from the committed artifact
//!
//! [`RecoveryContent`] is not a second quality: it is read from the emission that committed the
//! artifact — how many statements the emitter wrote that were not fallbacks — and no plane is read
//! from it or writes it. It exists because `Produced` says only that an artifact was delivered: a
//! report whose whole artifact is reasons and quoted bytecode is `Produced` too, and before this
//! field a caller had to strip comments from [`RecoveryReport::text`] to guess which of the two it
//! held. A stopped run holds no artifact, so the report of a stop states `NotProduced` without asking
//! whether anything was built or written before the refusal.
//!
//! # A stop is not a produced artifact
//!
//! When the budget refuses, the run is cancelled, or a table the payload needs is missing, the
//! report carries no text, no segments, an [`ExecutionReport`] that says so, and a
//! [`RecoveryOutcome::Stopped`] reason naming what refused and where. Nothing in a stopped report can
//! be read as "an empty body was recovered successfully" — that is the property the P3 tasks call
//! out, and the one a caller has to be able to rely on without reading diagnostics.

use jarde_jvm::ir::{CompileStatus, Quality, Representation, SemanticValidation, SyntaxStatus};
use jarde_jvm::method_ir::{MethodIr, Slot, SsaTable};
use jarde_reader::budget::{Budget, BudgetDimension, UsageSnapshot};
use jarde_reader::classfile::VerificationStatus;
use jarde_reader::model::{
    Diagnostic, DiagnosticSeverity, ExecutionReport, PhysicalDefinitionId, PhysicalMethodId,
    TerminationReason,
};
use serde::Serialize;

use crate::accessor::AccessorRecord;
use crate::artifact::{ArtifactBinding, ArtifactSubject, RecoveryArtifact};
use crate::ast::{AssignOp, ConstructorTarget, Expr, ExprKind, StmtKind, Type};
use crate::bridge::{self, BridgeRecord, ClassSourceBridgeCandidate};
use crate::build;
use crate::concat::{self, ConcatRecord};
use crate::declaration::{self, DeclarationRecord};
use crate::decode::Operations;
use crate::emit::{
    Emitted, emit, emit_class_initializer_value as emit_initializer_value, emit_source_map,
};
use crate::enumswitch::{self, ClassSourceEnumSwitchCandidate, EnumSwitchRecord};
use crate::evidence::{
    EvidencePayload, EvidencePhase, EvidenceRefusal, Materialized, Publication, RecoveryEvidence,
    RecoveryEvidenceKind, RecoveryEvidenceRequest, SegmentPublication,
};
use crate::facts::{
    ACC_ANNOTATION, ACC_INTERFACE, ClassMembers, FieldAccess, Operation, RecoveryFacts,
};

const ACC_ENUM: u16 = 0x4000;
use crate::field::{self, FieldRecord};
use crate::init::{self, InitRecord, NewRecord};
use crate::lambda::LambdaRecord;
use crate::names::NameTable;
use crate::normal_flow::NormalFlowView;
use crate::pass::{
    ACCESSOR, BRIDGE, CONCAT, DECLARATION, ENUMSWITCH, FIELD, INIT, LAMBDA, NEW, RecoveryProfile,
    RuleVersion,
};
use crate::region::{FallbackReason, Recovered, Region};
use crate::reuse;
use crate::source_map::{OriginSet, SourceMap};
use crate::stop::StopReason;

/// One recovery request: the payload of a P2 run, the facts that run did not publish, and the
/// profile the presentation is written under.
#[derive(Clone, Debug)]
pub struct RecoveryRequest<'a> {
    /// The IR payload of the method-analysis run whose body is being presented.
    pub ir: &'a MethodIr,
    /// The two facts the payload does not carry: the method's identity and its debug names. The
    /// decoded operations are not here — they travel inside the payload, so that the branch a
    /// comparison performs, the slot a load names and the value a constant pushes have exactly one
    /// source, the run that decoded them.
    pub facts: &'a RecoveryFacts,
    /// The recovery profile the run is presented under: the rule set whose passes the gate admits
    /// ([`crate::pass`]). It is the request's own runtime profile, stated by the caller — the entry
    /// point hands the environment's profile over — and it is a *policy* input, not evidence: unlike
    /// the three tables above, it says nothing about what the bytes are.
    pub profile: RecoveryProfile,
    /// The class's **other members**, as the caller read them beside this body: the declaration and
    /// the decoded body of a member the payload cannot hold, because the payload is one method's
    /// (P3 2.2). A caller that did not read the class's members states `None`, and the accessor rule
    /// then records the table it is missing rather than guessing from a call's name.
    pub members: Option<&'a ClassMembers>,
    /// Target definitions proved from the class-source request's selected physical environment.
    /// Method-only recovery has none. `new@1` verifies each call site separately; this fact alone
    /// carries no conclusion about any particular allocation.
    pub member_inner_targets: &'a [ProvedMemberInnerTarget],
    /// Exact interface-special targets whose Java source qualifier and default binding were proved
    /// by the facade's selected-definition reads. Direct recovery has no such environment and
    /// therefore leaves interface-qualified `super` calls refused.
    pub interface_super_calls: &'a [ProvedInterfaceSuperCall],
    /// Exact captured-outer reads proved by the selected class-source family assembly.
    /// A direct method request has no lexical family and supplies none.
    pub captured_outer_reads: &'a [ProvedCapturedOuterRead],
    /// Which **optional evidence** this request wants delivered (change
    /// `add-demand-driven-core-results`, D1): the categories of detail records, and the driver BCI
    /// range they are restricted to. [`RecoveryEvidenceRequest::essential`] — the default
    /// [`RecoveryRequest::new`] states — selects none of them, and the run then delivers the
    /// necessary results: the artifact, the planes, the core gaps and the stops. The selection never
    /// decides whether a rule runs, whether a value is proven or whether a region is refused.
    pub evidence: RecoveryEvidenceRequest,
    /// What the artifact this run commits is **of**, as the entry that performed the trusted read
    /// states it (change `add-demand-driven-core-results`, D3'): the physical method identity, the
    /// member record that read established and the environment the run is presented under.
    ///
    /// `None` is a run whose caller stated no subject — the direct entry over a payload it built
    /// itself — and such a run publishes **no** artifact binding
    /// ([`RecoveryReport::artifact`]): identity a caller did not state is never invented from the
    /// request's own spelling.
    pub subject: Option<ArtifactSubject>,
}

/// Narrow, non-serialized physical target fact supplied by class-source assembly. It is a target
/// declaration proof only; the allocation, outer value and effect order remain method-site work.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedMemberInnerTarget {
    pub definition: PhysicalDefinitionId,
    pub owner: String,
    pub outer: String,
    pub simple_name: String,
    pub constructor_descriptor: String,
    pub capture_field: String,
    /// Class and constructor signatures proved the source-level generic member tail.
    pub generic_diamond: bool,
    /// The selected, bidirectionally proved source type path from its top-level enclosing class to
    /// this member. The binary names stay attached so a consumer never splits `$` on its own.
    pub source_type_path: Vec<ProvedMemberInnerSourceSegment>,
}

/// Trusted, non-serialized handoff for one captured field read in one physical child method.
/// The constructor origin is retained for the family projection; it is never inserted into this
/// method's source map, whose BCIs belong to `method` alone.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedCapturedOuterRead {
    pub method: PhysicalMethodId,
    pub read_bci: u32,
    pub field_owner: String,
    pub field_name: String,
    pub field_descriptor: String,
    pub outer_internal_name: String,
    pub outer_source_name: String,
    pub constructor: PhysicalMethodId,
    pub constructor_write_bci: u32,
}

/// One exact `invokespecial InterfaceMethodref` target proved writable as `I.super.m(...)`.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedInterfaceSuperCall {
    pub owner: String,
    pub name: String,
    pub descriptor: String,
}

/// One selected definition in a source type path proved by the class-source adapter.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedMemberInnerSourceSegment {
    pub definition: PhysicalDefinitionId,
    pub binary_name: String,
    /// Fully-qualified Java source name through this segment, for example `matrix.Outer.A`.
    pub source_name: String,
    /// The class Signature's proven type parameter count; a raw use has no arguments, while a
    /// parameterized use must provide exactly this many.
    pub type_parameter_count: usize,
    /// The selected enclosing binary name, when this is a member type.
    pub enclosing_binary_name: Option<String>,
    /// Whether the member relation is static. A non-static generic member requires its enclosing
    /// type segment to remain explicit in a Signature path.
    pub is_static: bool,
}

/// One field write retained beside a `<clinit>` recovery for the class-source assembler.
///
/// This is a private-in-practice handoff: it is returned only by
/// [`recover_for_class_source`], is not part of [`RecoveryReport`] or its serialized schema, and is
/// derived from the same AST and field plan that produce that report. The public visibility is
/// required because `jarde` is a separate adapter crate.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassInitializerFieldWrite {
    /// Position among the recovered top-level `<clinit>` statements.
    pub order: usize,
    /// The exact field instruction this statement was built from.
    pub bci: u32,
    /// The constant-pool owner, in internal form.
    pub owner: String,
    /// The constant-pool field name.
    pub name: String,
    /// The member name the recovery AST wrote for the assignment.
    pub spelled_name: String,
    /// The constant-pool field descriptor.
    pub descriptor: String,
    /// Whether this write names a static field.
    pub is_static: bool,
    /// Whether the emitted assignment carries a receiver expression.
    pub has_receiver: bool,
    /// The assignment operator the AST states.
    pub op: crate::ast::AssignOp,
    /// The statement's original source anchors.
    pub source: OriginSet,
    /// The right-hand side as the recovery AST built it, with its own source anchors.
    pub value: crate::ast::Expr,
    /// Every static field read nested in the RHS, each joined to this run's `field@1` claim.
    /// `None` means a field-shaped AST node had no exact read claim in this method's field plan.
    pub field_reads: Option<Vec<ClassInitializerFieldRead>>,
}

/// One static field read inside a class-initializer write's RHS, retained from `field@1`.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassInitializerFieldRead {
    /// The field instruction's BCI in this `<clinit>` body.
    pub bci: u32,
    /// The constant-pool owner, in internal form.
    pub owner: String,
    /// The constant-pool field name.
    pub name: String,
    /// The constant-pool field descriptor.
    pub descriptor: String,
    /// Whether this read names a static field.
    pub is_static: bool,
}

/// What occupies one top-level position in a `<clinit>` candidate sequence.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassInitializerStep {
    /// A statement whose source is a field write proven by `field@1`.
    FieldWrite(Box<ClassInitializerFieldWrite>),
    /// A statement the later all-or-nothing projection must account for separately.
    Other {
        /// Position among the recovered top-level statements.
        order: usize,
        /// The statement's primary bytecode index.
        bci: u32,
        /// Its AST-level class; fallback text itself is never consulted.
        kind: ClassInitializerStatementKind,
    },
}

/// The AST statement classes relevant to an all-or-nothing class initializer projection.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClassInitializerStatementKind {
    /// A local declaration.
    Declaration,
    /// An assignment to a local.
    LocalAssignment,
    /// An expression statement, commonly a call.
    Expression,
    /// A field-shaped AST statement without a matching claimed write at this BCI.
    UnattributedFieldWrite,
    /// An array element write.
    ArrayWrite,
    /// A constructor invocation.
    ConstructorCall,
    /// A return statement.
    Return,
    /// A throw statement.
    Throw,
    /// A conditional statement.
    Conditional,
    /// A loop statement.
    Loop,
    /// A switch statement.
    Switch,
    /// A try statement.
    Try,
    /// A synchronized statement.
    Synchronized,
    /// A quoted bytecode fallback.
    Fallback,
}

/// The ordered, bounded AST candidates from one `<clinit>()V` recovery.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassInitializerCandidates {
    /// The physical identity of the member this sequence belongs to, when its read stated it.
    pub member: Option<jarde_reader::model::PhysicalMethodId>,
    /// Whether the same Code facts declare at least one exception-table row. Missing Code facts
    /// conservatively make this true so the class-level proof cannot mistake unknown for none.
    pub has_exception_handlers: bool,
    /// Every top-level statement in original recovery order.
    pub steps: Vec<ClassInitializerStep>,
}

/// The minimal ordered AST handoff for one enum constructor recovered in this class-source run.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassEnumConstructorCandidates {
    pub member: Option<jarde_reader::model::PhysicalMethodId>,
    pub complete: bool,
    pub has_exception_handlers: bool,
    pub steps: Vec<ClassEnumConstructorStep>,
}

/// One top-level constructor statement and the same-run AST shape the bytecode built.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassEnumConstructorStep {
    pub order: usize,
    pub bci: u32,
    pub source: OriginSet,
    pub kind: ClassEnumConstructorStepKind,
}

/// Only the constructor statement shapes a later bounded proof can consume are retained as AST.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassEnumConstructorStepKind {
    ConstructorCall {
        target: ConstructorTarget,
        args: Vec<Expr>,
    },
    Expression(Expr),
    FieldWrite {
        field: Option<ClassEnumConstructorField>,
        spelled_name: String,
        receiver: Option<Expr>,
        op: AssignOp,
        value: Expr,
    },
    Return {
        value: Option<Expr>,
    },
    Other(ClassInitializerStatementKind),
}

/// The physical member identity claimed by `field@1` for one constructor store.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassEnumConstructorField {
    pub bci: u32,
    pub owner: String,
    pub name: String,
    pub descriptor: String,
    pub is_static: bool,
}

/// A class-source recovery result with its non-serialized same-run sidecars.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceRecovery {
    /// The ordinary recovery report, unchanged from [`recover`].
    pub report: RecoveryReport,
    /// The same-run `<clinit>` AST candidates, when the body was produced.
    pub initializer: Option<ClassInitializerCandidates>,
    /// The same-run AST candidates for an enum constructor, when this is a produced body.
    pub enum_constructor: Option<ClassEnumConstructorCandidates>,
    /// The same-run `bridge@1` verdict, independent of the optional `RuleDetails` record.
    pub bridge: Option<ClassSourceBridgeCandidate>,
    /// Same-run enum-table reads consumed by integer switches, for bounded class-level proof.
    pub enum_switches: Vec<ClassSourceEnumSwitchCandidate>,
    /// Same-run proved array-helper call sites, retained privately for atomic class-source
    /// projection after the class-wide helper-use census.
    pub array_constructors: Vec<ClassSourceArrayConstructorCandidate>,
    /// Same-run Fieldref operations of this physical method, used to reject mutable aliases of a
    /// selected synthetic table in visible class-source members.
    pub enum_switch_field_uses: Vec<ClassSourceEnumSwitchFieldUse>,
    /// A bounded same-run AST/SSA proof for a direct parameter return.
    pub generic_return: Option<GenericReturnCandidate>,
    /// A bounded same-run AST/SSA proof for an empty constructor that calls Object().
    pub generic_constructor: Option<GenericConstructorCandidate>,
    /// Every visible allocation instruction in the recovered physical method. `None` means this
    /// run stopped before retaining the scan or had no physical method identity; `complete=false`
    /// means raw `new` opcode facts did not all resolve through the existing decoder.
    pub anonymous_allocations: Option<AnonymousAllocationScan>,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceArrayConstructorCandidate {
    pub member: jarde_reader::model::PhysicalMethodId,
    pub helper: jarde_reader::model::PhysicalMethodId,
    pub helper_owner: jarde_reader::model::JvmBytes,
    pub bootstrap_index: u16,
    pub implementation_index: u16,
    pub sites: Vec<ArrayConstructorProjectionSite>,
    pub(crate) projection: std::sync::Arc<ArrayConstructorProjectionSource>,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrayConstructorProjectionSite {
    pub use_site: u32,
    pub site_cp: u16,
    pub array_type: String,
    pub target_type: String,
    pub helper_name: jarde_reader::model::JvmBytes,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArrayConstructorProjectionSource {
    pub(crate) program: crate::build::Program,
    pub(crate) facts: crate::facts::RecoveryFacts,
    pub(crate) declaration: Option<crate::declaration::Declaration>,
    pub(crate) member: jarde_reader::model::PhysicalMethodId,
}

/// The complete same-run census, with an explicit marker for allocation opcodes not decoded as
/// `Operation::Allocate` by the existing decoder.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousAllocationScan {
    pub complete: bool,
    pub allocations: Vec<AnonymousAllocationCandidate>,
}

/// One allocation candidate carried to bounded class-source proofs.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousAllocationCandidate {
    /// The physical method whose bytecode contains this allocation.
    pub member: PhysicalMethodId,
    /// The allocation instruction's BCI.
    pub head_bci: u32,
    /// The target class in internal form, from the `new` instruction's pool entry.
    pub class: String,
    /// Whether the same-run `new@1` plan verified the allocation as a complete construction site.
    /// False includes rejected shapes and allocations owned by another rule.
    pub verified: bool,
    /// Constructor invocation BCI for verified sites.
    pub constructor_bci: Option<u32>,
    /// Ordered producer BCIs of the constructor's arguments for verified sites.
    pub argument_bcis: Vec<u32>,
}

/// Parameter slots, rather than rendered text, identify the values in this proof.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericReturnCandidate {
    pub parameters: Vec<(u16, String)>,
    pub value: GenericReturnValue,
}

/// Parameter slots, rather than rendered text, identify the values this constructor leaves unused.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericConstructorCandidate {
    pub parameters: Vec<(u16, String)>,
    /// The `init@1` record derived from the same run's prologue decision.
    pub init: InitRecord,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericReturnValue {
    /// The body is exactly one effect-free `return;` instruction.
    EmptyVoid,
    Parameter(u16),
    Conditional {
        test: u16,
        when_true: u16,
        when_false: u16,
    },
    /// The method returns the same-run verified member creation, whose enclosing value is a
    /// parameter slot. The complete selected target is carried to class-source projection.
    MemberCreation {
        target: Box<ProvedMemberInnerTarget>,
        qualifier_slot: u16,
    },
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceEnumSwitchFieldUse {
    pub member: Option<PhysicalMethodId>,
    pub bci: u32,
    pub owner: String,
    pub name: String,
    pub descriptor: String,
    pub is_static: bool,
    pub write: bool,
}

/// Emits one proven class-initializer RHS as a field initializer fragment for the class-source
/// adapter. This is a narrow handoff to the existing expression formatter, not a second printer.
#[doc(hidden)]
pub fn emit_class_initializer_value(
    value: &crate::ast::Expr,
    member: &PhysicalMethodId,
    budget: &mut Budget,
) -> Result<String, StopReason> {
    emit_initializer_value(value, member, budget)
}

/// Re-emits the two user statements of a proved enum terminal constructor after mapping the
/// physical enum parameter slot back to the sole source parameter `arg0`.
///
/// The caller has already proved the corresponding Code locals and Fieldref/Methodref identities;
/// this handoff only binds the captured AST origins to the fixed statement sequence and delegates
/// spelling to the existing statement emitter.
#[doc(hidden)]
pub fn emit_class_enum_constructor_body(
    candidate: &ClassEnumConstructorCandidates,
    member: &PhysicalMethodId,
    budget: &mut Budget,
) -> Result<Option<String>, StopReason> {
    use crate::ast::{AssignOp, ConstructorTarget, ExprKind, Stmt, StmtKind};
    use crate::report::ClassEnumConstructorStepKind as StepKind;

    if candidate.member.as_ref() != Some(member)
        || !candidate.complete
        || candidate.has_exception_handlers
        || candidate.steps.len() != 4
    {
        return Ok(None);
    }
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        4,
        Some(3),
    )?;
    let [super_step, helper_step, field_step, return_step] = candidate.steps.as_slice() else {
        return Ok(None);
    };
    if [super_step, helper_step, field_step, return_step]
        .iter()
        .zip([3, 7, 12, 15])
        .enumerate()
        .any(|(order, (step, bci))| {
            step.order != order || step.bci != bci || step.source.primary().bci() != bci
        })
    {
        return Ok(None);
    }
    if !matches!(
        &super_step.kind,
        StepKind::ConstructorCall {
            target: ConstructorTarget::Super,
            args
        } if args.len() == 2
    ) || !matches!(&return_step.kind, StepKind::Return { value: None })
    {
        return Ok(None);
    }
    let StepKind::Expression(helper_expression) = &helper_step.kind else {
        return Ok(None);
    };
    let mut helper_expression = helper_expression.clone();
    let ExprKind::Call { args, .. } = &mut helper_expression.kind else {
        return Ok(None);
    };
    let [helper_argument] = args.as_mut_slice() else {
        return Ok(None);
    };
    if helper_expression.origin.primary().bci() != 7 || helper_argument.origin.primary().bci() != 6
    {
        return Ok(None);
    }
    let ExprKind::Local(local) = &mut helper_argument.kind else {
        return Ok(None);
    };
    *local = "arg0".to_owned();

    let StepKind::FieldWrite {
        field: Some(field),
        spelled_name,
        receiver: Some(receiver),
        op: AssignOp::Assign,
        value,
    } = &field_step.kind
    else {
        return Ok(None);
    };
    if field.bci != 12
        || field.is_static
        || receiver.origin.primary().bci() != 10
        || value.origin.primary().bci() != 11
    {
        return Ok(None);
    }
    let mut receiver = receiver.clone();
    let ExprKind::Local(local) = &mut receiver.kind else {
        return Ok(None);
    };
    *local = "this".to_owned();

    let mut value = value.clone();
    let ExprKind::Local(local) = &mut value.kind else {
        return Ok(None);
    };
    *local = "arg0".to_owned();

    let statements = [
        Stmt::new(
            StmtKind::Expr(helper_expression),
            helper_step.source.clone(),
        ),
        Stmt::new(
            StmtKind::FieldAssign {
                receiver: Some(receiver),
                name: spelled_name.clone(),
                op: AssignOp::Assign,
                value,
            },
            field_step.source.clone(),
        ),
    ];
    crate::emit::emit_class_enum_constructor_statements(&statements, member, budget).map(Some)
}

/// Re-emits one same-run method AST with a proved enum selector and constant labels.
#[doc(hidden)]
pub fn emit_class_source_enum_switch(
    candidate: &ClassSourceEnumSwitchCandidate,
    labels: &std::collections::BTreeMap<i64, String>,
    budget: &mut Budget,
) -> Result<Option<String>, crate::stop::StopReason> {
    let Some(source) = &candidate.projection else {
        return Ok(None);
    };
    let mut program = source.program.clone();
    let mut changed = false;
    fn visit(
        statements: &mut [crate::ast::Stmt],
        candidate: &ClassSourceEnumSwitchCandidate,
        labels: &std::collections::BTreeMap<i64, String>,
        changed: &mut bool,
    ) -> bool {
        use crate::ast::{ExprKind, StmtKind};
        for statement in statements {
            match &mut statement.kind {
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    if visit(then_body, candidate, labels, changed)
                        || visit(else_body, candidate, labels, changed)
                    {
                        return true;
                    }
                }
                StmtKind::While { body, .. }
                | StmtKind::For { body, .. }
                | StmtKind::DoWhile { body, .. }
                | StmtKind::Synchronized { body, .. } => {
                    if visit(body, candidate, labels, changed) {
                        return true;
                    }
                }
                StmtKind::Try {
                    body, finally_body, ..
                } => {
                    if visit(body, candidate, labels, changed)
                        || finally_body
                            .as_mut()
                            .is_some_and(|body| visit(body, candidate, labels, changed))
                    {
                        return true;
                    }
                }
                StmtKind::Switch { value, arms } => {
                    if statement.origin.primary().bci() == candidate.switch_bci {
                        let receiver = match &value.kind {
                            ExprKind::Index { index, .. } => match &index.kind {
                                ExprKind::Call {
                                    receiver: Some(receiver),
                                    name,
                                    args,
                                } if name == "ordinal" && args.is_empty() => {
                                    Some((**receiver).clone())
                                }
                                _ => None,
                            },
                            _ => None,
                        };
                        let Some(receiver) = receiver else {
                            return true;
                        };
                        for arm in arms {
                            let mut projected = Vec::with_capacity(arm.keys.len());
                            for key in &arm.keys {
                                let Some(label) = labels.get(key) else {
                                    return true;
                                };
                                projected.push(label.clone());
                            }
                            arm.labels = Some(crate::ast::SwitchLabels::Enum(projected));
                        }
                        *value = receiver;
                        *changed = true;
                        return true;
                    }
                    for arm in arms {
                        if visit(&mut arm.body, candidate, labels, changed) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }
    visit(&mut program.stmts, candidate, labels, &mut changed);
    if !changed {
        return Ok(None);
    }
    let emitted = emit(
        &program.stmts,
        &source.facts,
        source.declaration.as_ref(),
        source.member.as_ref(),
        budget,
    )?;
    Ok(Some(emitted.text))
}

/// Re-emits the same-run typed body after replacing only the array-constructor sites selected by
/// the class-wide proof. The AST, not previously rendered text, is the input to this projection.
pub fn emit_class_source_array_constructors(
    candidate: &ClassSourceArrayConstructorCandidate,
    budget: &mut Budget,
) -> Result<Option<String>, crate::stop::StopReason> {
    let source = &candidate.projection;
    if candidate.sites.len() != 1 || source.program.stmts.len() != 1 {
        return Ok(None);
    }
    let site = &candidate.sites[0];
    let helper_owner = std::str::from_utf8(&candidate.helper_owner.0)
        .ok()
        .map(|owner| owner.replace('/', "."));
    let helper_name = std::str::from_utf8(&candidate.helper.name.0).ok();
    let (Some(helper_owner), Some(helper_name)) = (helper_owner, helper_name) else {
        return Ok(None);
    };
    let crate::ast::StmtKind::Return {
        value: Some(expression),
    } = &source.program.stmts[0].kind
    else {
        return Ok(None);
    };
    let crate::ast::ExprKind::Lambda { params, body } = &expression.kind else {
        return Ok(None);
    };
    let exact_site = expression.origin.primary().bci() == site.use_site
        && expression.origin.primary().cp() == Some(site.site_cp)
        && params.len() == 1
        && matches!(
            &body.kind,
            crate::ast::ExprKind::Call {
                receiver: Some(receiver),
                name,
                args,
            } if args.len() == 1
            && name == helper_name
                && matches!(&receiver.kind, crate::ast::ExprKind::Path(owner) if owner == &helper_owner)
                && matches!(
                    (params.first(), args.first().map(|argument| &argument.kind)),
                    (Some(parameter), Some(crate::ast::ExprKind::Local(argument)))
                        if &parameter.name == argument
                )
        );
    if !exact_site {
        return Ok(None);
    }
    let work = 7_u64
        .saturating_add(
            u64::try_from(source.program.array_constructor_sites.len()).unwrap_or(u64::MAX),
        )
        .saturating_add(u64::try_from(source.program.lambdas.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(source.program.lambda_refusals.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(source.program.accessors.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(source.program.accessor_refusals.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(source.program.field_increments.len()).unwrap_or(u64::MAX));
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        work,
        Some(site.use_site),
    )?;
    let mut program = source.program.clone();
    let crate::ast::StmtKind::Return {
        value: Some(expression),
    } = &mut program.stmts[0].kind
    else {
        return Ok(None);
    };
    let origin = expression.origin.clone();
    let presented = expression.presented.clone();
    expression.kind = crate::ast::ExprKind::MethodReference {
        qualifier: Box::new(crate::ast::Expr::new(
            crate::ast::ExprKind::Path(site.array_type.clone()),
            origin.clone(),
        )),
        name: "new".to_owned(),
    };
    expression.origin = origin;
    expression.presented = presented;
    let emitted = crate::emit::emit(
        &program.stmts,
        &source.facts,
        source.declaration.as_ref(),
        Some(&source.member),
        budget,
    )?;
    Ok(Some(emitted.text))
}

impl<'a> RecoveryRequest<'a> {
    /// One request over one payload, one fact set and one profile, with no member table.
    ///
    /// The evidence selection is [`RecoveryEvidenceRequest::essential`]: the ordinary recovery, which
    /// delivers the necessary results and materializes no optional detail record. A caller that
    /// wants detail states it with [`RecoveryRequest::with_evidence`].
    pub fn new(ir: &'a MethodIr, facts: &'a RecoveryFacts, profile: RecoveryProfile) -> Self {
        Self {
            ir,
            facts,
            profile,
            members: None,
            member_inner_targets: &[],
            interface_super_calls: &[],
            captured_outer_reads: &[],
            evidence: RecoveryEvidenceRequest::essential(),
            subject: None,
        }
    }

    /// The same request, with the class's other members: the evidence a synthetic accessor call site
    /// is decided from (P3 2.2, A12).
    pub fn with_members(mut self, members: &'a ClassMembers) -> Self {
        self.members = Some(members);
        self
    }

    /// Supply only definitions whose target-side relation and constructor prologue were proved.
    pub fn with_member_inner_targets(mut self, targets: &'a [ProvedMemberInnerTarget]) -> Self {
        self.member_inner_targets = targets;
        self
    }

    /// Supply only interface-special targets proved against this request's selected environment.
    pub fn with_interface_super_calls(mut self, calls: &'a [ProvedInterfaceSuperCall]) -> Self {
        self.interface_super_calls = calls;
        self
    }

    /// Supply only the exact child reads closed by the selected family's capture certificate.
    pub fn with_captured_outer_reads(mut self, reads: &'a [ProvedCapturedOuterRead]) -> Self {
        self.captured_outer_reads = reads;
        self
    }

    /// The same request, selecting the optional evidence this run materializes.
    ///
    /// The selection is the caller's own statement and is echoed in the report beside what each
    /// category really delivered ([`RecoveryReport::evidence`]); it is never a second way to ask for
    /// a different *decision*.
    pub fn with_evidence(mut self, evidence: RecoveryEvidenceRequest) -> Self {
        self.evidence = evidence;
        self
    }

    /// The same request, stating what the artifact it presents is **of** (D3').
    ///
    /// The entry that performed the trusted read is the one that can state this: it holds the
    /// physical identity the run is bound to, the member record the read established and the
    /// environment it was validated under. A run presented with a subject publishes the binding of
    /// the artifact it commits ([`RecoveryReport::artifact`]), which is what makes a later request's
    /// `expected_artifact` checkable at all.
    pub fn with_subject(mut self, subject: ArtifactSubject) -> Self {
        self.subject = Some(subject);
        self
    }
}

/// Whether the run produced an artifact or stopped before it had one.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryOutcome {
    /// The artifact is in [`RecoveryReport::text`], and its positions are in
    /// [`RecoveryReport::source_map`].
    Produced,
    /// Nothing was produced; the reason says what refused and where.
    Stopped(StopReason),
}

impl RecoveryOutcome {
    /// Whether the run produced an artifact.
    pub fn produced(&self) -> bool {
        matches!(self, Self::Produced)
    }

    /// The stop, when the run stopped.
    pub fn stop(&self) -> Option<&StopReason> {
        match self {
            Self::Produced => None,
            Self::Stopped(reason) => Some(reason),
        }
    }
}

/// What a report's artifact holds: whether the run stopped without one, delivered explanation alone,
/// or delivered Java statements.
///
/// This is the answer to "does the artifact hold anything a caller can call code", and it is
/// deliberately the *only* answer to it — not a second quality plane and not a recovery measure. It
/// is read from the committed structure (the statements the emitter wrote, in `crate::emit`) and
/// never from [`RecoveryReport::text`]: no comment is stripped, no token is counted, and no caller
/// has to parse the artifact to learn this. A statement is a declaration, an assignment, a call, a
/// constructor call, a `return` or a control-flow statement; a wrapper line, a brace, a fallback's
/// reason and its bytecode indexes are not.
///
/// Two shapes keep the classification honest in both directions: `return;` alone is
/// `ContainsStatements`, and `if (arg0) {}` is `ContainsStatements` too (its condition is
/// evaluated), while neither of them raises `quality` or becomes a claim about the rest of the body.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryContent {
    /// The run stopped before an artifact was committed, so there is no content to describe: the
    /// text and the segment table are empty, as the stop contract states.
    NotProduced,
    /// An artifact was delivered and it holds no statement: what it says is the envelope, the
    /// reasons and the quoted bytecode of the regions the run refused.
    ExplanationOnly,
    /// The artifact holds at least one statement the emitter wrote as Java.
    ContainsStatements,
}

/// What one recovered region is, stated so that a reader can check the run's own claim about it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RegionRecord {
    /// The BCI the region starts at.
    pub bci: u32,
    /// Whether every block of the region was presented as Java structure.
    pub structured: bool,
    /// The BCIs of the blocks the region claims, in method order.
    pub blocks: Vec<u32>,
    /// The fallback's diagnostic code, when it has one.
    pub code: Option<&'static str>,
    /// The fallback's message, when it has one.
    pub message: Option<String>,
    /// The rule that produced this record: the pass that claimed the region, or the pass whose
    /// declared precondition refused it. `None` when no registered rule is answerable for it (a
    /// whole-body refusal the walk itself states) — never a borrowed rule name.
    pub rule: Option<RuleVersion>,
}

/// The result of one recovery run: one artifact, or the reason there is none.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveryReport {
    /// The method this report is about, as its facts state it.
    pub method: String,
    /// The recovery profile this run was presented under, echoed from the request.
    pub profile: RecoveryProfile,
    /// Every rule that produced a region of this method, each once, in the order it first did —
    /// how "which rule produced this output" is answered by the run's own record.
    pub rules: Vec<RuleVersion>,
    /// What the artifact is made of.
    pub representation: Representation,
    /// How strong the recovered structure is.
    pub quality: Quality,
    /// Whether the text is claimed to be Java syntax.
    pub syntax_status: SyntaxStatus,
    /// Whether the text was ever compiled; never, in this slice.
    pub compile_status: CompileStatus,
    /// What semantic evidence applies to the artifact.
    pub semantic_validation: SemanticValidation,
    /// Whether the artifact was verified; never, in this slice.
    pub verification: VerificationStatus,
    /// The execution of *this* run, in the vocabulary the fact layer already states.
    pub execution: ExecutionReport,
    /// Whether the run produced an artifact or stopped.
    pub outcome: RecoveryOutcome,
    /// What the delivered artifact holds, read from the committed structure. `Produced` says only
    /// that an artifact was delivered; this says whether any statement is in it — and says nothing
    /// about completeness, compilability or semantic equivalence, which the planes above state for
    /// themselves.
    pub content: RecoveryContent,
    /// The Java text, empty when the run stopped.
    pub text: String,
    /// The segment table of [`Self::text`], empty when the run stopped.
    pub source_map: SourceMap,
    /// Every region the run recovered, in method order.
    pub regions: Vec<RegionRecord>,
    /// Every `invokedynamic` site of the body, in BCI order, with the bootstrap, SAM, implementation
    /// and capture evidence behind it and what the run did with it (P3 2.1, A04). A site appears
    /// whether it was presented as a lambda or refused and left as bytecode: the refusals are part
    /// of the answer, not an absence from it.
    pub lambdas: Vec<LambdaRecord>,
    /// Every concatenation chain candidate of the body, in BCI order, with the class it built, every
    /// `append` it called and whether it was presented (P3 2.2). A refused candidate is part of the
    /// answer too: it names the link of the verification that failed.
    pub concats: Vec<ConcatRecord>,
    /// Every synthetic accessor call site of the body, in BCI order, with the member the call named,
    /// the flags the class declared, the field its body accesses and whether the call site was
    /// presented as that field (P3 2.2, A12).
    pub accessors: Vec<AccessorRecord>,
    /// The `bridge@1` rule's verdict for this very method, when the member is declared a bridge or
    /// its body is the forward a bridge is written as (P3 2.2). Empty for every other body: an
    /// ordinary member is not a bridge question.
    pub bridges: Vec<BridgeRecord>,
    /// Every construction site of the body, in BCI order, with the class it allocates, the
    /// constructor it calls and the BCIs of its arguments (P3 2.3, `new@1`). A refused candidate is
    /// part of the answer: it names the link of the verification that failed. This is also the rule
    /// that presents a local, anonymous or inner class's **use** — the class's own name is the one
    /// the pool spells — while the nesting relation itself is a class-level fact this run does not
    /// hold and never claims.
    pub news: Vec<NewRecord>,
    /// Every field instruction of the body, in BCI order, with the member each names and whether it
    /// was presented as a field access (P3 2.3, `field@1`).
    pub fields: Vec<FieldRecord>,
    /// Every dispatch-table read of the body, in BCI order (P3 2.3, `enumswitch@1`). What the record
    /// deliberately does not state is a mapping to enum constants: that is the enum class's own
    /// declaration, which this run never read.
    pub enum_switches: Vec<EnumSwitchRecord>,
    /// The body's constructor prologue, when the body is an instance initializer (P3 2.3, `init@1`).
    /// `None` for every body that is not one.
    pub init: Option<InitRecord>,
    /// What the run read of the member's declaration, and therefore what the artifact's envelope
    /// states (P3 2.3, `declaration@1`). Present for every produced artifact: every body has a
    /// declaration, and a run that cannot read one records the fact it was missing. `None` only when
    /// the run stopped before it read anything.
    pub declaration: Option<DeclarationRecord>,
    /// Every fallback the run had to keep, with its code.
    pub fallbacks: Vec<&'static str>,
    /// The evidence selection this run was presented under, and what each category delivered
    /// (change `add-demand-driven-core-results`, D1).
    ///
    /// The status list is fixed-size — one entry per category, whatever the selection was — and it is
    /// the answer to "was this asked for, and did it arrive", which no empty `Vec` and no `None` can
    /// give on its own. It is checked against this report's own payload before the run returns.
    pub evidence: RecoveryEvidence,
    /// What this run states about the artifact it committed, and about the artifact its request
    /// named as `expected_artifact` (change `add-demand-driven-core-results`, D3').
    ///
    /// The two halves are [`RecoveryArtifact::binding`] — the contract facts of this run's own
    /// artifact, which a caller keeps to explain this very text later — and
    /// [`RecoveryArtifact::agreement`] — what the run decided about the text the request named: no
    /// expectation (`NotStated`), this artifact (`Agreed`, the only verdict that attaches the
    /// selected evidence), a different one (`Mismatched`, stated dimension by dimension) or nothing
    /// to compare (`Unverifiable`). A mismatch never re-points the evidence at the caller's text and
    /// never deletes this run's own artifact.
    pub artifact: RecoveryArtifact,
    /// The names the presentation decided, when the run reached the naming step.
    pub aliased_names: Vec<String>,
    /// What the run states about itself, in the fact layer's diagnostic vocabulary.
    pub diagnostics: Vec<Diagnostic>,
}

impl RecoveryReport {
    /// Whether the artifact is Java text this run wrote.
    pub fn produced(&self) -> bool {
        self.outcome.produced()
    }

    /// The stop reason, when the run stopped.
    pub fn stop(&self) -> Option<&StopReason> {
        self.outcome.stop()
    }

    /// The text one bytecode index reached, in writing order — the question the segment table
    /// answers and the one 3.2 grows on.
    pub fn text_of_bci(&self, bci: u32) -> Vec<&str> {
        self.source_map.text_of_bci(&self.text, bci)
    }

    /// States that the entry which performed this run's read materialized `ReadDetails` beside this
    /// report (change `add-demand-driven-core-results`, D3).
    ///
    /// `ReadDetails` is the one category this layer does not build: the read evidence belongs to the
    /// entry that performed the read and is published beside the report — `RecoveredMethod::callees`
    /// for the facade — so the entry states what it published here, and the list stays the report's
    /// own fixed-size statement about every category.
    ///
    /// The read is one step: an entry that published it published all of it, so the category is
    /// [`EvidenceState::Complete`], and a read that named no member at all is a legal *empty*
    /// complete result for the same reason an empty region table is. An entry that did not publish
    /// it — the run stopped before the read was materialized, or the selection was refused — states
    /// nothing here, and the category stays [`EvidenceState::NotPerformed`].
    pub fn read_details_materialized(&mut self) {
        self.evidence.delivered(RecoveryEvidenceKind::ReadDetails);
    }
}

/// Recovers one method's body.
///
/// The run is charged to the caller's budget: blocks and statements to `IrItems`, the region walk to
/// `AnalysisSteps`, the text to `OutputBytes`. Every charge happens before the work it pays for, so a
/// refusal leaves no work half done — see [`crate::stop`].
pub fn recover(request: &RecoveryRequest<'_>, budget: &mut Budget) -> RecoveryReport {
    recover_inner(
        request, budget, None, None, None, None, None, None, None, None, None, true,
    )
}

/// Capture only the body shape whose generic return type follows directly from unchanged
/// parameter slots. Every local read is checked against its own SSA load; a write to any
/// parameter slot makes the whole candidate unavailable.
fn generic_return_candidate(
    program: &build::Program,
    names: &NameTable,
    ssa: &SsaTable,
    operations: &Operations,
    sites: &init::Sites,
    request: &RecoveryRequest<'_>,
    budget: &mut Budget,
) -> Result<Option<GenericReturnCandidate>, StopReason> {
    let Some(code) = request.ir.code() else {
        return Ok(None);
    };
    let parameter_types = request.facts.method().parameter_types();
    crate::stop::poll(budget, None)?;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        u64::try_from(program.stmts.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1),
        None,
    )?;
    if program.ragged || program.stmts.len() != 1 {
        return Ok(None);
    }
    if matches!(program.stmts[0].kind, StmtKind::Return { value: None }) {
        if program.statements != 1
            || code.stopped_at.is_some()
            || code.exception_handler_count != 0
            || !code.exception_handlers.is_empty()
            || code.instructions.len() != 1
            || code.instructions[0].opcode != 0xb1
            || ssa.blocks().len() != 1
            || !ssa.phis().is_empty()
            || ssa.blocks()[0].instructions().len() != 1
        {
            return Ok(None);
        }
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
            2,
            Some(code.instructions[0].bci),
        )?;
        crate::stop::poll(budget, Some(code.instructions[0].bci))?;
        let instruction = &ssa.blocks()[0].instructions()[0];
        let mut body_operations = operations.iter();
        let Some((operation_bci, Operation::Return)) = body_operations.next() else {
            return Ok(None);
        };
        let effects = ssa.effects().instructions();
        if body_operations.next().is_some()
            || !parameter_types.is_empty()
            || *operation_bci != instruction.bci()
            || instruction.bci() != code.instructions[0].bci
            || instruction.opcode() != 0xb1
            || !instruction.reads().is_empty()
            || !instruction.writes().is_empty()
            || program.stmts[0].origin.primary().bci() != instruction.bci()
            || !matches!(effects, [effect]
                if effect.bci() == instruction.bci()
                    && effect.opcode() == 0xb1
                    && effect.locals_read().is_empty()
                    && effect.locals_written().is_empty()
                    && !effect.may_throw()
                    && effect.handlers().is_empty())
        {
            return Ok(None);
        }
        return Ok(Some(GenericReturnCandidate {
            parameters: Vec::new(),
            value: GenericReturnValue::EmptyVoid,
        }));
    }
    let StmtKind::Return { value: Some(value) } = &program.stmts[0].kind else {
        return Ok(None);
    };
    let mut parameters = Vec::new();
    for slot in parameter_types.keys() {
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            None,
        )?;
        let Some(name) = names.whole(*slot) else {
            return Ok(None);
        };
        parameters.push((*slot, name.text().to_owned()));
    }
    let mut local_reads = std::collections::BTreeMap::new();
    let mut branch_bcis = std::collections::BTreeSet::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
                1,
                Some(instruction.bci()),
            )?;
            crate::stop::poll(budget, Some(instruction.bci()))?;
            if instruction.writes().iter().any(|(slot, _)| matches!(slot, Slot::Local(index) if parameter_types.contains_key(index))) {
                return Ok(None);
            }
            if matches!(instruction.opcode(), 0x99..=0xa6 | 0xc6..=0xc7) {
                branch_bcis.insert(instruction.bci());
            }
            if matches!(instruction.opcode(), 0x15..=0x2d) {
                let mut reads = instruction
                    .reads()
                    .iter()
                    .filter_map(|(slot, _)| match slot {
                        Slot::Local(index) => Some(*index),
                        Slot::Stack(_) => None,
                    });
                if let Some(slot) = reads.next()
                    && reads.next().is_none()
                    && local_reads
                        .insert(instruction.bci(), slot)
                        .is_some_and(|old| old != slot)
                {
                    return Ok(None);
                }
            }
        }
    }
    let local = |expr: &Expr, allow_branch_anchor: bool| -> Option<u16> {
        let ExprKind::Local(name) = &expr.kind else {
            return None;
        };
        if (!allow_branch_anchor && !expr.origin.derived().is_empty())
            || expr.origin.primary().provenance() != crate::source_map::Provenance::Direct
            || (allow_branch_anchor
                && expr
                    .origin
                    .derived()
                    .iter()
                    .any(|anchor| !branch_bcis.contains(&anchor.bci())))
        {
            return None;
        }
        let (slot, _) = parameters
            .iter()
            .find(|(_, parameter_name)| parameter_name == name)?;
        let bci = expr.origin.primary().bci();
        if local_reads.get(&bci) != Some(slot) {
            return None;
        }
        Some(*slot)
    };
    let member_creation = || -> Option<(ProvedMemberInnerTarget, u16)> {
        if !branch_bcis.is_empty()
            || !code.exception_handlers.is_empty()
            || code.exception_handler_count != 0
            || ssa.blocks().len() != 1
            || !ssa.phis().is_empty()
            || code.stopped_at.is_some()
            || program.statements != 1
            || code.instructions.last().is_none_or(|instruction| {
                instruction.opcode != 0xb0
                    || instruction.bci != program.stmts[0].origin.primary().bci()
            })
        {
            return None;
        }
        let ExprKind::New {
            qualifier: Some(qualifier),
            member_name: Some(member_name),
            diamond,
            ..
        } = &value.kind
        else {
            return None;
        };
        let qualifier_slot = local(qualifier, false)?;
        let qualifier_type = parameter_types.get(&qualifier_slot)?;
        let mut anchors = std::collections::BTreeSet::new();
        anchors.extend(program.stmts[0].origin.bcis());
        collect_expression_anchors(value, &mut anchors);
        // A source-level return candidate may only replace a complete same-run body. Every
        // physical instruction must have an AST anchor; a NOP is the only instruction whose
        // absence from Java text has no effect.
        if code
            .instructions
            .iter()
            .any(|instruction| instruction.opcode != 0x00 && !anchors.contains(&instruction.bci))
        {
            return None;
        }
        let site = value
            .origin
            .derived()
            .iter()
            .filter_map(|origin| sites.site_at_head(origin.bci()))
            .find(|site| {
                site.constructor == value.origin.primary().bci()
                    && site.member_inner.as_ref().is_some_and(|member| {
                        member.qualifier == qualifier.origin.primary().bci()
                            && member.simple_name == *member_name
                            && member.generic_diamond == *diamond
                    })
            })?;
        let selected = request.member_inner_targets.iter().find(|target| {
            target.owner == site.class
                && target.simple_name == *member_name
                && target.outer
                    == site
                        .member_inner
                        .as_ref()
                        .map_or("", |member| member.outer.as_str())
                && target.source_type_path.iter().any(|segment| {
                    segment.binary_name == target.outer
                        && qualifier_type == &Type::Reference(segment.binary_name.replace('/', "."))
                })
        })?;
        Some((selected.clone(), qualifier_slot))
    };
    let value = match &value.kind {
        ExprKind::Local(_) => GenericReturnValue::Parameter(match local(value, false) {
            Some(slot) => slot,
            None => return Ok(None),
        }),
        ExprKind::Conditional {
            test,
            when_true,
            when_false,
        } => {
            let (Some(test), Some(when_true), Some(when_false)) = (
                local(test, true),
                local(when_true, false),
                local(when_false, false),
            ) else {
                return Ok(None);
            };
            if parameter_types.get(&test) != Some(&Type::Boolean) {
                return Ok(None);
            }
            GenericReturnValue::Conditional {
                test,
                when_true,
                when_false,
            }
        }
        ExprKind::New { .. } => match member_creation() {
            Some((target, qualifier_slot)) => GenericReturnValue::MemberCreation {
                target: Box::new(target),
                qualifier_slot,
            },
            None => return Ok(None),
        },
        _ => return Ok(None),
    };
    Ok(Some(GenericReturnCandidate { parameters, value }))
}

/// Retain every bytecode origin represented by one complete return expression. The member-return
/// candidate compares this set with the method's instruction table before it can affect a header.
fn collect_expression_anchors(expr: &Expr, anchors: &mut std::collections::BTreeSet<u32>) {
    anchors.extend(expr.origin.bcis());
    match &expr.kind {
        ExprKind::InstanceOf { value, .. }
        | ExprKind::Field {
            receiver: value, ..
        }
        | ExprKind::PostIncrement { target: value }
        | ExprKind::ArrayLength { array: value }
        | ExprKind::Cast { value, .. }
        | ExprKind::Not { value }
        | ExprKind::Neg { value } => collect_expression_anchors(value, anchors),
        ExprKind::Call { receiver, args, .. } => {
            if let Some(receiver) = receiver {
                collect_expression_anchors(receiver, anchors);
            }
            for arg in args {
                collect_expression_anchors(arg, anchors);
            }
        }
        ExprKind::New {
            qualifier, args, ..
        } => {
            if let Some(qualifier) = qualifier {
                collect_expression_anchors(qualifier, anchors);
            }
            for arg in args {
                collect_expression_anchors(arg, anchors);
            }
        }
        ExprKind::Lambda { body, .. } => collect_expression_anchors(body, anchors),
        ExprKind::MethodReference { qualifier, .. } => {
            collect_expression_anchors(qualifier, anchors)
        }
        ExprKind::Index { array, index }
        | ExprKind::Binary {
            left: array,
            right: index,
            ..
        } => {
            collect_expression_anchors(array, anchors);
            collect_expression_anchors(index, anchors);
        }
        ExprKind::NewArray {
            lengths,
            initializers,
            ..
        } => {
            for value in lengths {
                collect_expression_anchors(value, anchors);
            }
            if let Some(values) = initializers {
                for value in values {
                    collect_expression_anchors(value, anchors);
                }
            }
        }
        ExprKind::Conditional {
            test,
            when_true,
            when_false,
        } => {
            collect_expression_anchors(test, anchors);
            collect_expression_anchors(when_true, anchors);
            collect_expression_anchors(when_false, anchors);
        }
        ExprKind::Concat { parts } => {
            for part in parts {
                collect_expression_anchors(&part.value, anchors);
            }
        }
        ExprKind::Local(_)
        | ExprKind::Integer(_)
        | ExprKind::Boolean(_)
        | ExprKind::Long(_)
        | ExprKind::Float(_)
        | ExprKind::Double(_)
        | ExprKind::Str(_)
        | ExprKind::Null
        | ExprKind::ClassLiteral { .. }
        | ExprKind::Path(_)
        | ExprKind::QualifiedThis { .. }
        | ExprKind::Super { .. } => {}
    }
}

/// Capture only the constructor body whose complete AST and SSA state no work beyond `Object()`.
/// The prologue is the decision from which the selected `InitRecord` is materialized; carrying its
/// BCI here makes this sidecar independent of the caller's evidence selection.
fn generic_constructor_candidate(
    program: &build::Program,
    names: &NameTable,
    ssa: &SsaTable,
    operations: &Operations,
    code: &jarde_reader::classfile::MethodCodeFacts,
    prologues: &init::Prologues,
    parameter_types: &std::collections::BTreeMap<u16, Type>,
    budget: &mut Budget,
) -> Result<Option<GenericConstructorCandidate>, StopReason> {
    crate::stop::poll(budget, None)?;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        u64::try_from(program.stmts.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1),
        None,
    )?;
    if program.ragged
        || program.stmts.len() != 2
        || program.statements != 2
        || code.stopped_at.is_some()
        || code.exception_handler_count != 0
        || !code.exception_handlers.is_empty()
        || code.instructions.len() != 3
        || ssa.blocks().len() != 1
        || !ssa.phis().is_empty()
    {
        return Ok(None);
    }
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
        9,
        None,
    )?;
    if !matches!(
        program.stmts[0].kind,
        StmtKind::ConstructorCall {
            target: ConstructorTarget::Super,
            ref args,
        } if args.is_empty()
    ) || !matches!(program.stmts[1].kind, StmtKind::Return { value: None })
    {
        return Ok(None);
    }

    let init = prologues.record();
    let Some(init_bci) = init.bci else {
        return Ok(None);
    };
    if !init.presented
        || init.target != Some(ConstructorTarget::Super)
        || init.class.as_deref() != Some("java/lang/Object")
        || init.declared.is_none()
        || init_bci != program.stmts[0].origin.primary().bci()
    {
        return Ok(None);
    }
    let Some(Operation::Invoke(target)) = operations.get(init_bci) else {
        return Ok(None);
    };
    if target.kind() != crate::facts::InvokeKind::Special
        || target.owner() != "java/lang/Object"
        || target.name() != "<init>"
        || target.descriptor() != "()V"
        || target.is_interface_reference()
    {
        return Ok(None);
    }

    let expected_opcodes = [0x2a, 0xb7, 0xb1];
    let block = &ssa.blocks()[0];
    if block.instructions().len() != expected_opcodes.len()
        || block
            .instructions()
            .iter()
            .zip(expected_opcodes)
            .any(|(instruction, expected)| instruction.opcode() != expected)
        || code
            .instructions
            .iter()
            .zip(expected_opcodes)
            .any(|(instruction, expected)| instruction.opcode != expected)
        || operations.iter().count() != expected_opcodes.len()
    {
        return Ok(None);
    }
    let effects = ssa.effects().instructions();
    if effects.len() != expected_opcodes.len()
        || effects
            .iter()
            .zip(expected_opcodes.into_iter().zip([false, true, false]))
            .any(|(effect, (expected, may_throw))| {
                effect.opcode() != expected
                    || !effect.handlers().is_empty()
                    || effect.may_throw() != may_throw
            })
    {
        return Ok(None);
    }
    let instructions = block.instructions();
    let [(Slot::Stack(0), receiver)] = instructions[0].writes() else {
        return Ok(None);
    };
    if !matches!(instructions[0].reads(), [(Slot::Local(0), _)])
        || !matches!(instructions[1].reads(), [(Slot::Stack(0), value)] if value == receiver)
        || !matches!(instructions[1].writes(), [(Slot::Local(0), _)])
        || !instructions[2].reads().is_empty()
        || !instructions[2].writes().is_empty()
    {
        return Ok(None);
    }

    let mut parameters = Vec::with_capacity(parameter_types.len());
    for slot in parameter_types.keys() {
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            None,
        )?;
        let Some(name) = names.whole(*slot) else {
            return Ok(None);
        };
        parameters.push((*slot, name.text().to_owned()));
    }
    Ok(Some(GenericConstructorCandidate { parameters, init }))
}

/// The same recovery for the class-source assembler, with same-run class-source sidecars.
///
/// The ordinary report follows exactly the same path as [`recover`]. Sidecars are available only
/// to the adapter that assembles one class and are never serialized as report evidence.
#[doc(hidden)]
pub fn recover_for_class_source(
    request: &RecoveryRequest<'_>,
    budget: &mut Budget,
    prove_generic_return: bool,
    collect_enum_constructor_candidates: bool,
) -> ClassSourceRecovery {
    let is_clinit =
        request.facts.method().name() == "<clinit>" && request.facts.method().descriptor() == "()V";
    let is_ordinary_interface = request
        .facts
        .method()
        .declaring_class()
        .is_some_and(|class| {
            let flags = class.access_flags();
            flags & ACC_INTERFACE != 0 && flags & ACC_ANNOTATION == 0
        });
    let is_enum = request
        .facts
        .method()
        .declaring_class()
        .is_some_and(|class| class.access_flags() & ACC_ENUM != 0);
    let collect_initializer = is_clinit && (is_ordinary_interface || is_enum);
    let collect_enum_constructor = collect_enum_constructor_candidates
        && is_enum
        && request.facts.method().name() == "<init>"
        && matches!(
            request.facts.method().descriptor(),
            "(Ljava/lang/String;I)V" | "(Ljava/lang/String;II)V"
        );
    let mut initializer = None;
    let mut enum_constructor = None;
    let mut bridge = None;
    let mut enum_switches = None;
    let mut enum_switch_field_uses = None;
    let mut array_constructors = None;
    let mut generic_return = None;
    let mut generic_constructor = None;
    let mut anonymous_allocations = None;
    let report = recover_inner(
        request,
        budget,
        collect_initializer.then_some(&mut initializer),
        collect_enum_constructor.then_some(&mut enum_constructor),
        Some(&mut bridge),
        Some(&mut enum_switches),
        Some(&mut array_constructors),
        Some(&mut enum_switch_field_uses),
        prove_generic_return.then_some(&mut generic_return),
        prove_generic_return.then_some(&mut generic_constructor),
        Some(&mut anonymous_allocations),
        false,
    );
    if !report.produced() || !matches!(&report.execution, ExecutionReport::Complete { .. }) {
        if !is_enum {
            initializer = None;
        }
        enum_constructor = None;
        bridge = None;
        enum_switches = None;
        array_constructors = None;
        enum_switch_field_uses = None;
        generic_return = None;
        generic_constructor = None;
        anonymous_allocations = None;
    }
    ClassSourceRecovery {
        report,
        initializer,
        enum_constructor,
        bridge,
        enum_switches: enum_switches.unwrap_or_default(),
        array_constructors: array_constructors.unwrap_or_default(),
        enum_switch_field_uses: enum_switch_field_uses.unwrap_or_default(),
        generic_return,
        generic_constructor,
        anonymous_allocations,
    }
}

fn recover_inner(
    request: &RecoveryRequest<'_>,
    budget: &mut Budget,
    initializer: Option<&mut Option<ClassInitializerCandidates>>,
    mut enum_constructor: Option<&mut Option<ClassEnumConstructorCandidates>>,
    mut bridge_candidate: Option<&mut Option<ClassSourceBridgeCandidate>>,
    mut enum_switch_candidate: Option<&mut Option<Vec<ClassSourceEnumSwitchCandidate>>>,
    array_constructor_candidate: Option<&mut Option<Vec<ClassSourceArrayConstructorCandidate>>>,
    mut enum_switch_field_use: Option<&mut Option<Vec<ClassSourceEnumSwitchFieldUse>>>,
    generic_return: Option<&mut Option<GenericReturnCandidate>>,
    generic_constructor: Option<&mut Option<GenericConstructorCandidate>>,
    mut anonymous_allocations: Option<&mut Option<AnonymousAllocationScan>>,
    allow_array_constructor_method_references: bool,
) -> RecoveryReport {
    let method = format!(
        "{}{}",
        request.facts.method().name(),
        request.facts.method().descriptor()
    );
    // The profile the presentation is written under, taken once: every path of this function —
    // including the ones that stop before any pass runs — echoes the request's own profile, so a
    // stopped report cannot claim a rule set the request did not declare.
    let profile = request.profile.clone();
    // The selection this run is presented under, taken once: every path below — the refusals, the
    // stopped reports and the report itself — echoes the request's own statement, so no report can
    // claim an evidence selection the request did not make.
    let selection = request.evidence.clone();
    let Some(canonical) = request.ir.canonical() else {
        return stopped(
            method,
            profile.clone(),
            &selection,
            StopReason::IrTableMissing { table: "canonical" },
            budget,
        );
    };
    let Some(frames) = request.ir.frames() else {
        return stopped(
            method,
            profile.clone(),
            &selection,
            StopReason::IrTableMissing { table: "frames" },
            budget,
        );
    };
    let Some(ssa) = request.ir.ssa() else {
        return stopped(
            method,
            profile.clone(),
            &selection,
            StopReason::IrTableMissing { table: "ssa" },
            budget,
        );
    };
    // The decode facts of the same run: the operations the presentation is written in, and the
    // exception table it states. A payload that holds a graph but no decode is not one run's
    // artifact (the graph is built from the decode), so this is a malformed payload rather than a
    // body without facts.
    let Some(code) = request.ir.code() else {
        return stopped(
            method,
            profile.clone(),
            &selection,
            StopReason::IrTableMissing { table: "code" },
            budget,
        );
    };
    // The request's own applicability check: a category this entry does not materialize, or a
    // driver range this body cannot support, is refused *before* anything is presented, with the
    // read this run already performed still charged. An unsupported or illegal selection is never
    // widened into a full-evidence delivery.
    if let Err(refusal) = selection.check(code) {
        return refused(method, profile.clone(), &selection, refusal, budget);
    }
    let operations = Operations::of(code, request.ir.constant_pool());
    if let Some(field_uses_slot) = enum_switch_field_use.as_deref_mut() {
        let member = request
            .ir
            .declaration()
            .map(|declaration| declaration.identity().clone());
        let mut field_uses = Vec::new();
        for (bci, operation) in operations.iter() {
            if let Operation::Field {
                access,
                is_static,
                owner,
                name,
                descriptor,
            } = operation
            {
                if let Err(stop) = crate::stop::charge(
                    budget,
                    jarde_reader::budget::CountedBudgetDimension::IrItems,
                    1,
                    Some(*bci),
                ) {
                    return stopped(method, profile.clone(), &selection, stop, budget);
                }
                if let Err(stop) = crate::stop::poll(budget, Some(*bci)) {
                    return stopped(method, profile.clone(), &selection, stop, budget);
                }
                field_uses.push(ClassSourceEnumSwitchFieldUse {
                    member: member.clone(),
                    bci: *bci,
                    owner: owner.clone(),
                    name: name.clone(),
                    descriptor: descriptor.clone(),
                    is_static: *is_static,
                    write: *access == FieldAccess::Write,
                });
            }
        }
        *field_uses_slot = Some(field_uses);
    }
    if canonical.blocks().is_empty() {
        return stopped(
            method,
            profile.clone(),
            &selection,
            StopReason::IrTableMissing {
                table: "canonical blocks",
            },
            budget,
        );
    }
    let view = match NormalFlowView::build(canonical, budget) {
        Ok(view) => view,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    // `MethodCodeFacts` describes only the Code attribute; a missing method flag must not be read
    // as proof that the JVM's implicit synchronized-method monitor is absent.
    let method_synchronized = request
        .facts
        .method()
        .access_flags()
        .map(|flags| flags & 0x0020 != 0);
    // The two-terminal-return claim may commit ownership only when the descriptor's own result
    // is `boolean` — the same reading of the descriptor the builder re-checks (P3 1.4: an
    // `int` method's shared `iconst_0; ireturn` leaf belongs to the one-armed `if` walk).
    let return_is_boolean = build::return_type(request.facts.method().descriptor())
        .is_some_and(|ty| matches!(ty, crate::ast::Type::Boolean));
    // These plans depend only on this run's decoded SSA facts. Build them before Region because
    // Guard needs read-only access to verified construction sites while proving resource headers.
    let fields = match field::plan(
        ssa,
        &operations,
        request.facts.method().declaring_class(),
        request.facts.method().name(),
        request.facts.method().descriptor(),
        request.ir.class_fields(),
        budget,
    ) {
        Ok(fields) => fields,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    let chains = concat::plan(ssa, &operations);
    let sites = init::sites(
        ssa,
        &operations,
        &chains,
        chains.owned(),
        &fields,
        request.member_inner_targets,
        code,
    );
    let mut recovered: Recovered = match crate::region::recover(
        canonical,
        &view,
        ssa,
        &operations,
        &sites,
        code,
        method_synchronized,
        return_is_boolean,
        &request.profile,
        budget,
    ) {
        Ok(recovered) => recovered,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    if let Err(stop) = crate::region::project_string_switches(&mut recovered, request.ir, budget) {
        return stopped(method, profile.clone(), &selection, stop, budget);
    }
    // The slots the names are decided for are the body's own local slots: the frames table states
    // how many there are, and a local the debug metadata never named still needs a name.
    let slots = u16::try_from(frames.locals_slots()).unwrap_or(u16::MAX);
    // Which *variable* each slot holds, before any name is decided (P3 3.4): either disjoint LVT
    // records or a narrow SSA/CFG lifetime proof may split one physical slot. An unnamed segment
    // receives an ordinal name below. Guard-header slots keep their own declaration rule.
    let reuse = match reuse::plan(
        ssa,
        canonical,
        slots,
        request.facts.method().parameters(),
        request.facts.debug_locals(),
        &build::resource_slots(&recovered.regions),
        budget,
    ) {
        Ok(reuse) => reuse,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    // Field declarations are borrowed from the same class facts as the body. The field plan keeps
    // the proof that lets a blank same-class static final write lose its qualifier, and the same
    // proof supplies the names the local naming walk must reserve.
    let reserved_field_names = match fields.simple_static_final_names(budget) {
        Ok(names) => names,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    let names = if request.facts.method().has_receiver() {
        // Slot 0 holds the receiver (JVMS 4.10.1.9), so it is spelled as one: the answer comes from
        // the member's own flags and from nothing else, which is why the naming is told it instead of
        // finding it out from a debug name or a slot ordinal.
        NameTable::build_with_receiver_and_reserved(
            request.facts.method().parameters(),
            slots,
            reuse.evidence(),
            &reserved_field_names,
        )
    } else {
        NameTable::build_with_reserved(
            request.facts.method().parameters(),
            slots,
            reuse.evidence(),
            &reserved_field_names,
        )
    };
    // The two shapes this run decides *before* a single statement is written, each from this run's
    // own tables: the concatenation chains the body builds (P3 2.2) and the bridge verdict for the
    // member itself, when its declaration or its body makes it one. Both are decisions about the
    // bytes, not about the text, which is why they are taken here and read by the builder.
    // Every plan below decides for every evidence selection: the premises are checked, the candidates
    // are verified or refused and the gaps are stated here, and none of these calls builds an owning
    // record. The records the request selected are materialized from these plans *after* the artifact
    // is committed (the evidence phase below), so a run that stops inside the evidence keeps its text.
    let bridge = bridge::plan(request.facts.method(), ssa, &operations);
    if let (Some(plan), Some(candidate_slot)) = (bridge.as_ref(), bridge_candidate.as_deref_mut()) {
        let member = request
            .subject
            .as_ref()
            .map(|subject| subject.method().clone())
            .or_else(|| {
                request
                    .ir
                    .declaration()
                    .map(|declaration| declaration.identity().clone())
            });
        let has_exception_handlers = request
            .ir
            .code()
            .is_none_or(|code| code.exception_handler_count != 0);
        *candidate_slot = Some(
            match plan.class_source_candidate(
                member,
                request.facts.method().access_flags(),
                has_exception_handlers,
                budget,
            ) {
                Ok(candidate) => candidate,
                Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
            },
        );
    }
    // The four shapes P3 2.3 reads — each decided before a statement is written, each from this run's
    // own tables. `field@1` is decided **before** `new@1`, because a construction site's only question
    // about a field access is whether that access is a place the instance is written into, and the
    // answer is that rule's own claim: the site plan reads the verdict instead of guessing it from the
    // opcode (P3 2c.26). The order is a dependency, not a preference — nothing either plan decides is
    // read by the other in the other direction.
    //
    // The construction sites reserve the concatenation chains' instructions, because one instruction
    // is never two shapes: the allocation a verified chain builds is written inside the `+` expression
    // and not a second time as a `new`.
    if let Some(output) = anonymous_allocations.as_deref_mut() {
        let member = request
            .subject
            .as_ref()
            .map(|subject| subject.method().clone())
            .or_else(|| {
                request
                    .ir
                    .declaration()
                    .map(|declaration| declaration.identity().clone())
            });
        if let Some(member) = member {
            if let Err(stop) = crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::IrItems,
                u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
                None,
            ) {
                return stopped(method, profile.clone(), &selection, stop, budget);
            }
            let raw_new_count = code
                .instructions
                .iter()
                .filter(|item| item.opcode == 0xbb)
                .count();
            let mut allocations = Vec::with_capacity(sites.allocation_candidates().len());
            for candidate in sites.allocation_candidates() {
                if let Err(stop) = crate::stop::charge(
                    budget,
                    jarde_reader::budget::CountedBudgetDimension::IrItems,
                    1,
                    Some(candidate.head),
                ) {
                    return stopped(method, profile.clone(), &selection, stop, budget);
                }
                if let Err(stop) = crate::stop::poll(budget, Some(candidate.head)) {
                    return stopped(method, profile.clone(), &selection, stop, budget);
                }
                let site = sites.site_at_head(candidate.head);
                allocations.push(AnonymousAllocationCandidate {
                    member: member.clone(),
                    head_bci: candidate.head,
                    class: candidate.class.clone(),
                    verified: candidate.verified && site.is_some(),
                    constructor_bci: site.map(|site| site.constructor),
                    argument_bcis: site.map_or_else(Vec::new, |site| site.arguments.clone()),
                });
            }
            *output = Some(AnonymousAllocationScan {
                complete: code.stopped_at.is_none() && raw_new_count == allocations.len(),
                allocations,
            });
        }
    }
    let prologues = init::prologue(ssa, &operations, request.facts.method());
    let enums = enumswitch::plan(ssa, &operations);
    // The declaration is read from the two facts the caller stated and decides the artifact's
    // envelope; it never decides a statement, and it is the only shape of this slice that is read
    // without an instruction to read it from.
    let declaration = declaration::plan(request.facts.method());
    // The type each parameter slot holds, as the member's own **descriptor** states it (P3-R5): the
    // frames cannot tell a `boolean` parameter from an `int` one, and the descriptor can.
    let parameter_types = request.facts.method().parameter_types();
    // The same reading for the **return** position: the type this member's own signature returns,
    // which decides both the boolean shape (`(I)Z`, `()Z` and the rest of the descriptor spellings
    // are the same fact) and the conversion every other `return` type requires of its value. The
    // builder reads it as this run's fact — like the parameter types, it is derived here rather than
    // re-read out of a descriptor inside the layer that writes the statements.
    let return_type = build::return_type(request.facts.method().descriptor());
    let program = match build::build(
        canonical,
        ssa,
        &operations,
        build::Inputs {
            code,
            pool: request.ir.constant_pool(),
            bootstrap: request.ir.bootstrap_methods(),
            profile: request.profile.clone(),
            parameters: request.facts.method().parameters(),
            has_receiver: request.facts.method().has_receiver(),
            parameter_types: &parameter_types,
            return_type,
            names: &names,
            reuse: &reuse,
            chains: &chains,
            members: request.members,
            member_inner_targets: request.member_inner_targets,
            interface_super_calls: request.interface_super_calls,
            captured_outer_reads: request.captured_outer_reads,
            physical_method: request.ir.declaration().map(|member| member.identity()),
            // The class this body belongs to, as the run's own member declaration states it: the
            // fact a static call's pool owner is compared against, so that a call to this class is
            // written unqualified and a call to another class names it (P3 4.4). A run that read no
            // member declaration states none, and then no static call gains a qualifier.
            declaring_class: request
                .facts
                .method()
                .declaring_class()
                .map(|declaring| declaring.name()),
            direct_super_class: request.ir.direct_super_class(),
            direct_interfaces: request.ir.direct_interfaces(),
            class_methods: request.ir.class_methods(),
            bridge: bridge.as_ref(),
            sites: &sites,
            prologues: &prologues,
            fields: &fields,
            enums: &enums,
            allow_array_constructor_method_references,
        },
        &recovered.regions,
        budget,
    ) {
        Ok(program) => program,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    if let Some(slot) = generic_return {
        match generic_return_candidate(&program, &names, ssa, &operations, &sites, request, budget)
        {
            Ok(candidate) => *slot = candidate,
            Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
        }
    }
    if let Some(slot) = generic_constructor {
        if request.facts.method().name() == "<init>" {
            match generic_constructor_candidate(
                &program,
                &names,
                ssa,
                &operations,
                code,
                &prologues,
                &parameter_types,
                budget,
            ) {
                Ok(candidate) => *slot = candidate,
                Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
            }
        }
    }
    if let Some(slot) = enum_constructor.as_deref_mut() {
        if request.facts.method().name() == "<init>" {
            match class_enum_constructor_candidates(request, &program, &fields, code, budget) {
                Ok(candidates) => *slot = Some(candidates),
                Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
            }
        }
    }
    if let Some(initializer) = initializer {
        match class_initializer_candidates(request, &program, &fields, budget) {
            Ok(candidates) => *initializer = Some(candidates),
            Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
        }
    }
    if let Some(enum_switches) = enum_switch_candidate.as_deref_mut() {
        let member = request
            .ir
            .declaration()
            .map(|declaration| declaration.identity().clone());
        match enums.class_source_candidates(ssa, &operations, member.clone(), budget) {
            Ok(mut candidates) => {
                if !candidates.is_empty() {
                    if let Err(stop) = crate::stop::charge(
                        budget,
                        jarde_reader::budget::CountedBudgetDimension::IrItems,
                        u64::try_from(program.statements.max(1)).unwrap_or(u64::MAX),
                        candidates.first().map(|candidate| candidate.switch_bci),
                    ) {
                        return stopped(method, profile.clone(), &selection, stop, budget);
                    }
                    if let Err(stop) = crate::stop::poll(
                        budget,
                        candidates.first().map(|candidate| candidate.switch_bci),
                    ) {
                        return stopped(method, profile.clone(), &selection, stop, budget);
                    }
                    let projection = std::sync::Arc::new(enumswitch::EnumSwitchProjectionSource {
                        program: program.clone(),
                        facts: request.facts.clone(),
                        declaration: declaration.declaration().cloned(),
                        member,
                    });
                    for candidate in &mut candidates {
                        candidate.projection = Some(projection.clone());
                    }
                }
                *enum_switches = Some(candidates);
            }
            Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
        }
    }
    if let Some(array_constructors) = array_constructor_candidate
        && !program.array_constructor_sites.is_empty()
    {
        let Some(member) = request
            .ir
            .declaration()
            .map(|declaration| declaration.identity().clone())
        else {
            return stopped(
                method,
                profile.clone(),
                &selection,
                StopReason::IrTableMissing {
                    table: "declaration",
                },
                budget,
            );
        };
        if let Err(stop) = crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            u64::try_from(program.statements.max(1)).unwrap_or(u64::MAX),
            program
                .array_constructor_sites
                .first()
                .map(|site| site.use_site),
        ) {
            return stopped(method, profile.clone(), &selection, stop, budget);
        }
        if let Err(stop) = crate::stop::poll(
            budget,
            program
                .array_constructor_sites
                .first()
                .map(|site| site.use_site),
        ) {
            return stopped(method, profile.clone(), &selection, stop, budget);
        }
        let projection = std::sync::Arc::new(ArrayConstructorProjectionSource {
            program: program.clone(),
            facts: request.facts.clone(),
            declaration: declaration.declaration().cloned(),
            member: member.clone(),
        });
        let exact_helpers = crate::lambda::array_helper_candidates(request.ir);
        let mut candidates = Vec::new();
        for site in &program.array_constructor_sites {
            let Some(helper) = exact_helpers.iter().find(|helper| {
                helper.call_site == site.use_site
                    && helper.site_cp == site.site_cp
                    && request
                        .ir
                        .declaration()
                        .is_some_and(|declaration| helper.owner.0 == declaration.class_name().0)
                    && std::str::from_utf8(&helper.name.0).ok() == Some(site.helper_name.as_str())
                    && std::str::from_utf8(&helper.descriptor.0).ok()
                        == Some(site.helper_descriptor.as_str())
            }) else {
                continue;
            };
            candidates.push(ClassSourceArrayConstructorCandidate {
                member: member.clone(),
                helper: jarde_reader::model::PhysicalMethodId {
                    owner: member.owner.clone(),
                    name: helper.name.clone(),
                    descriptor: helper.descriptor.clone(),
                },
                helper_owner: helper.owner.clone(),
                bootstrap_index: helper.bootstrap_index,
                implementation_index: helper.implementation_index,
                sites: vec![ArrayConstructorProjectionSite {
                    use_site: site.use_site,
                    site_cp: site.site_cp,
                    array_type: site.array_type.clone(),
                    target_type: site.target_type.clone(),
                    helper_name: helper.name.clone(),
                }],
                projection: projection.clone(),
            });
        }
        *array_constructors = Some(candidates);
    }
    // The identity of the body being presented, as the payload's own declaration states it: the
    // member every anchor of this artifact belongs to (P3 3.2). A run that read no member header
    // states none. Both emitter passes state it for the anchors that name no member of their own.
    let member = request
        .ir
        .declaration()
        .map(|declaration| declaration.identity());
    let emitted: Emitted = match emit(
        &program.stmts,
        request.facts,
        declaration.declaration(),
        member,
        budget,
    ) {
        Ok(emitted) => emitted,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    let field_presentations = match field::committed_presentations(&program, &fields, budget) {
        Ok(presentations) => presentations,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    // What the artifact that was just committed holds. The classification is taken here, from the
    // emission itself, and not from the AST the build had produced: a statement that was built and
    // then never committed (the emitter stopped inside it) is not in any artifact, and the run that
    // stopped is classified by [`stopped`], which never asks this question.
    let content = content_of(&emitted);

    // The planes, each from its own input.
    let structured = recovered.is_structured() && !program.ragged;
    let fallbacks: Vec<&'static str> = recovered
        .fallbacks()
        .iter()
        .map(FallbackReason::code)
        .collect();
    let mut diagnostics = Vec::new();
    for region in &recovered.regions {
        for reason in region.fallbacks() {
            diagnostics.push(diagnostic(
                reason.code(),
                DiagnosticSeverity::Warning,
                &reason.message(),
            ));
        }
    }
    // The names the presentation could not write as the source spelled them: a core gap, stated in
    // every selection from the naming table itself. What it states is the count and the affected
    // slots — the alias the run wrote instead, and the raw spelling it could not write, are
    // NameDetails and are materialized only when the request selects them.
    let aliased_slots: Vec<String> = names
        .names()
        .filter(|name| name.aliased().is_some())
        .map(|name| format!("slot {}", name.slot()))
        .collect();
    if !aliased_slots.is_empty() {
        diagnostics.push(diagnostic(
            "jre_name_aliased",
            DiagnosticSeverity::Warning,
            &format!(
                "{} local name(s) could not be written as the source spelled them: {}",
                aliased_slots.len(),
                aliased_slots.join(", ")
            ),
        ));
    }
    diagnostics.push(diagnostic(
        "jre_recovery_produced",
        DiagnosticSeverity::Info,
        &format!(
            "{} region(s), {} statement(s), {} segment(s), {} byte(s) written from {} canonical block(s)",
            recovered.regions.len(),
            program.statements,
            emitted.segments,
            emitted.written,
            recovered.blocks
        ),
    ));
    // The rule index: every rule that decided something about this body, each once, in the order
    // this layer reads them. It is rule evidence — a category the request selects or does not — and
    // it is read from the *plans*, so the driver range never changes it: a range selects records,
    // not the decisions they are written from, and the decisions are the same for every selection.
    let mut rules = if selection.requests(RecoveryEvidenceKind::RuleDetails) {
        recovered.rules()
    } else {
        Vec::new()
    };
    if selection.requests(RecoveryEvidenceKind::RuleDetails) {
        // A dynamic site is a rule's answer too: the decision names `lambda@1` whether it presented
        // the site or refused it, so the report's rule list states both. A body with no site names no
        // lambda rule, which is why the list is built from the plan rather than from a record table.
        let mut answered: Vec<RuleVersion> = Vec::new();
        if !program.lambdas.is_empty() {
            answered.push(crate::lambda::RULE);
        }
        if chains.answered() {
            answered.push(concat::RULE);
        }
        if !program.accessors.is_empty() {
            answered.push(crate::accessor::RULE);
        }
        if bridge.is_some() {
            answered.push(bridge::RULE);
        }
        if sites.answered() {
            answered.push(init::NEW_RULE);
        }
        if fields.answered() {
            answered.push(field::RULE);
        }
        if enums.answered() {
            answered.push(enumswitch::RULE);
        }
        if prologues.answered() {
            answered.push(init::INIT_RULE);
        }
        // The declaration rule is listed when it *read* a declaration: its output is the envelope
        // line, and a run that was not told the declaration facts wrote none. The refusal is not
        // invisible for that — the gap and its diagnostic name the rule — but a rule that concluded
        // nothing about these bytes is not a rule that produced this artifact.
        if declaration.declaration().is_some() {
            answered.push(declaration::RULE);
        }
        for rule in answered {
            if !rules.contains(&rule) {
                rules.push(rule);
            }
        }
    }
    // The refusals are diagnostics of their own: a site that was not presented says which link of
    // the chain failed, whether it was the class's table, the factory, the SAM's shape or a value
    // this layer may not replay (A04). Every one of them is stated from the *gap* the rule recorded
    // where it decided, so closing the rule records does not close the gaps.
    for gap in &program.lambda_refusals {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let lambda_sites = program.lambdas_presented + count_of(program.lambda_refusals.len());
    if lambda_sites > 0 {
        diagnostics.push(diagnostic(
            "jre_lambda_sites",
            DiagnosticSeverity::Info,
            &format!(
                "{lambda_sites} dynamic site(s) read under {}: {} presented, {} refused",
                LAMBDA.rule(),
                program.lambdas_presented,
                program.lambda_refusals.len(),
            ),
        ));
    }
    // The three shapes of 2.2 report the same way: every refusal is a diagnostic of its own, and a
    // summary states how many candidates were read and how many were presented. A run that read no
    // candidate of a shape says nothing about that shape's rule at all.
    for gap in chains.refusals() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let (concat_read, concat_presented) = chains.counts();
    if concat_read > 0 {
        diagnostics.push(diagnostic(
            "jre_concat_chains",
            DiagnosticSeverity::Info,
            &format!(
                "{concat_read} concatenation candidate(s) read under {}: {concat_presented} presented, {} refused",
                CONCAT.rule(),
                concat_read - concat_presented,
            ),
        ));
    }
    for gap in &program.accessor_refusals {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let accessor_sites = program.accessors_presented + count_of(program.accessor_refusals.len());
    if accessor_sites > 0 {
        diagnostics.push(diagnostic(
            "jre_accessor_sites",
            DiagnosticSeverity::Info,
            &format!(
                "{accessor_sites} accessor call site(s) read under {}: {} presented as a field access, {} refused",
                ACCESSOR.rule(),
                program.accessors_presented,
                program.accessor_refusals.len(),
            ),
        ));
    }
    if let Some(gap) = bridge.as_ref().and_then(|plan| plan.refusal()) {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    if let Some(plan) = bridge.as_ref() {
        diagnostics.push(diagnostic(
            "jre_bridge",
            DiagnosticSeverity::Info,
            &format!(
                "the body was read under {}: {}",
                BRIDGE.rule(),
                if plan.presented() {
                    "presented as the forward it is"
                } else {
                    "not presented as a forward"
                }
            ),
        ));
    }
    // P3 2.3's four shapes report the same way the three of 2.2 do: every refusal is a diagnostic of
    // its own, and a summary states how many candidates were read and how many were presented. A body
    // that read none of a shape says nothing about that shape's rule at all — except for the
    // declaration, which every body has and which therefore always states what it read.
    for gap in sites.refusals() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let (new_read, new_presented) = sites.counts();
    if new_read > 0 {
        diagnostics.push(diagnostic(
            "jre_new_sites",
            DiagnosticSeverity::Info,
            &format!(
                "{new_read} construction candidate(s) read under {}: {new_presented} presented as `new`, {} refused",
                NEW.rule(),
                new_read - new_presented,
            ),
        ));
    }
    for gap in fields.refusals(&field_presentations) {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let (field_read, field_presented) = fields.counts(&field_presentations);
    if field_read > 0 {
        diagnostics.push(diagnostic(
            "jre_field_accesses",
            DiagnosticSeverity::Info,
            &format!(
                "{field_read} field instruction(s) read under {}: {field_presented} presented, {} refused",
                FIELD.rule(),
                field_read - field_presented,
            ),
        ));
    }
    for gap in enums.refusals() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let (enum_read, enum_presented) = enums.counts();
    if enum_read > 0 {
        // The boundary is stated where the shape is: the read is the bytecode's own dispatch, and the
        // mapping from its entries to enum constants is a class-level fact of *another* class this
        // run never read (P3 2.3).
        diagnostics.push(diagnostic(
            "jre_enumswitch",
            DiagnosticSeverity::Info,
            &format!(
                "{enum_read} dispatch-table read(s) read under {}: {enum_presented} presented as the table read the bytecode performs, {} refused; the constants those entries stand for are the enum class's own declaration, which this run does not hold, so no `case T.CONST:` label is written",
                ENUMSWITCH.rule(),
                enum_read - enum_presented,
            ),
        ));
    }
    if let Some(gap) = prologues.refusal() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    } else if let Some(prologue) = prologues.prologue() {
        diagnostics.push(diagnostic(
            "jre_constructor_prologue",
            DiagnosticSeverity::Info,
            &format!(
                "the body is an instance initializer and its prologue is written under {} as `{}`: the call at BCI {} names `{}`, and the class that declares this constructor is `{}`",
                INIT.rule(),
                prologue.target.spell(),
                prologue.bci,
                prologue.class,
                prologue.declared,
            ),
        ));
    }
    if let Some(gap) = declaration.refusal() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    } else if let Some(declared) = declaration.declaration() {
        diagnostics.push(diagnostic(
            "jre_declaration",
            DiagnosticSeverity::Info,
            &format!(
                "the artifact's envelope states the member's declaration under {}: {}",
                DECLARATION.rule(),
                declared.form.spell()
            ),
        ));
    }
    // ---------------------------------------------------------------------------------------
    // The artifact binding and the judgement about the artifact the request named (D3', tasks
    // 5.1/5.2). It is taken here — the artifact is committed and every diagnostic of this run is
    // stated, and no evidence record has been materialized yet — because the phase below is exactly
    // what the judgement gates. The caller's `expected_artifact` is a **candidate for verification**:
    // the value it is compared against is this run's own binding, and only an agreement lets this
    // run's evidence describe the text the caller holds. A mismatch refuses the *materialization* and
    // nothing else — the text committed above stays, its planes and its gaps stay, and the categories
    // the request selected stay `NotPerformed`, the state for a selection that was refused.
    //
    // A run whose entry stated no subject publishes no binding at all: identity this layer was not
    // given is never invented from the request's spelling, and a request that names an artifact for
    // such a run is answered `Unverifiable` rather than with a comparison nobody performed.
    // ---------------------------------------------------------------------------------------
    let artifact = match &request.subject {
        Some(subject) => RecoveryArtifact::of(
            ArtifactBinding::of(subject, &request.profile, &emitted.text),
            selection.expected_artifact(),
        ),
        None => RecoveryArtifact::unbound(selection.expected_artifact()),
    };
    if let (Some(refusal), Some(code)) =
        (artifact.agreement().refusal(), artifact.agreement().code())
    {
        // The verdict is a gap of this run, stated in the vocabulary every other gap uses. It is a
        // warning and not a stop: the artifact was committed and is delivered, and what the verdict
        // refuses is the attachment of this run's evidence to another artifact.
        diagnostics.push(diagnostic(code, DiagnosticSeverity::Warning, refusal));
    }
    let attaches = artifact.attaches();
    // ---------------------------------------------------------------------------------------
    // The evidence phase: the artifact is committed and every diagnostic of this run is stated, and
    // what the request selected is materialized now — one owning record at a time, within the same
    // budget, in the fixed order this layer states its categories in.
    //
    // Every category is materialized *after* this line, including the rule records and the segment
    // table: the rules decided before the artifact was written, but their owning records are written
    // from the plans here. A refusal in the phase therefore stops the *materialization* and nothing
    // else — the text, its planes and the gaps above stay exactly what this run produced, the
    // category in flight reports the prefix it delivered, the categories the phase never reached
    // stay `NotPerformed`, and the run's execution states the real stop.
    // ---------------------------------------------------------------------------------------
    let publication = Publication::of(&selection);
    let mut evidence = RecoveryEvidence::pending(&selection);
    let mut phase = EvidencePhase::new();
    let range = selection.driver_bci_range();

    // The region records: the run holds every region either way, and the records are built here —
    // one by one, each after the charge that pays for it — only for the regions the selected driver
    // range intersects. A region is kept or dropped as a unit, with the origins it states for
    // itself.
    let (regions, reached) = if attaches && selection.requests(RecoveryEvidenceKind::RegionDetails)
    {
        phase.materialize(
            budget,
            recovered
                .regions
                .iter()
                .filter(|region| in_driver_range(region, range)),
            region_record,
        )
    } else {
        (Vec::new(), Materialized::None)
    };
    evidence.materialized(RecoveryEvidenceKind::RegionDetails, reached);

    // Every rule record of the run, in the order this layer reads the rules. One category, nine
    // plans: the category is complete only when every one of its records was materialized, and the
    // phase's own stop ends it wherever it lands.
    let mut lambdas: Vec<LambdaRecord> = Vec::new();
    let mut concats: Vec<ConcatRecord> = Vec::new();
    let mut accessors: Vec<AccessorRecord> = Vec::new();
    let mut bridges: Vec<BridgeRecord> = Vec::new();
    let mut news: Vec<NewRecord> = Vec::new();
    let mut field_records: Vec<FieldRecord> = Vec::new();
    let mut enum_switches: Vec<EnumSwitchRecord> = Vec::new();
    let mut init: Option<InitRecord> = None;
    let mut declaration_record: Option<DeclarationRecord> = None;
    if attaches && selection.requests(RecoveryEvidenceKind::RuleDetails) {
        let mut delivery = Delivery::new(&phase);
        lambdas = delivery.take(program.materialize_lambdas(publication, &mut phase, budget));
        concats = delivery.take(chains.materialize(publication, &mut phase, budget));
        accessors = delivery.take(program.materialize_accessors(publication, &mut phase, budget));
        bridges = match bridge.as_ref() {
            Some(plan) => delivery
                .take_one(plan.materialize(publication, &mut phase, budget))
                .into_iter()
                .collect(),
            None => Vec::new(),
        };
        news = delivery.take(sites.materialize(publication, &mut phase, budget));
        field_records = delivery.take(fields.materialize(
            &field_presentations,
            publication,
            &mut phase,
            budget,
        ));
        enum_switches = delivery.take(enums.materialize(publication, &mut phase, budget));
        init = prologues
            .answered()
            .then(|| delivery.take_one(prologues.materialize(publication, &mut phase, budget)))
            .flatten();
        declaration_record =
            delivery.take_one(declaration.materialize(publication, &mut phase, budget));
        evidence.materialized(RecoveryEvidenceKind::RuleDetails, delivery.reached());
    }

    // The names the presentation had to replace: per local slot rather than per bytecode index, so a
    // driver range neither selects nor drops one of them — a request that selects this category over
    // a range gets it whole.
    let (aliased_names, reached) =
        if attaches && selection.requests(RecoveryEvidenceKind::NameDetails) {
            phase.materialize(
                budget,
                names.names().filter(|name| name.aliased().is_some()),
                aliased_name,
            )
        } else {
            (Vec::new(), Materialized::None)
        };
    evidence.materialized(RecoveryEvidenceKind::NameDetails, reached);

    // The source map: the last category, because it is the only one that replays the whole decided
    // AST. The committing pass wrote the text and owns no table; this pass writes no text and
    // verifies every byte it re-writes against the artifact the committing pass produced.
    let mut gate_stop: Option<StopReason> = None;
    let (source_map, reached) = if attaches && selection.requests(RecoveryEvidenceKind::SourceMap) {
        match emit_source_map(
            &program.stmts,
            request.facts,
            declaration.declaration(),
            member,
            SegmentPublication::of(&selection),
            &emitted,
            &mut phase,
            budget,
        ) {
            Ok((map, reached)) => (map, reached),
            Err(stop) => {
                gate_stop = Some(stop);
                (SourceMap::default(), Materialized::None)
            }
        }
    } else {
        (SourceMap::default(), Materialized::None)
    };
    if gate_stop.is_none() {
        evidence.materialized(RecoveryEvidenceKind::SourceMap, reached);
    }

    // The stop of the evidence phase, when it stopped — and the one the source-map gate states when
    // the replayed writes disagreed with the committed artifact: both are stated in the same
    // vocabulary every other stop of this layer uses, and neither is a claim about the artifact.
    let stop = gate_stop.or_else(|| phase.stopped().then(|| phase.reason(budget)));
    let execution = match &stop {
        Some(reason) => {
            diagnostics.push(stop_diagnostic(reason));
            stop_execution(reason, budget.usage())
        }
        None => ExecutionReport::Complete {
            usage: budget.usage(),
        },
    };
    let report = RecoveryReport {
        profile: request.profile.clone(),
        representation: if structured {
            Representation::Java
        } else {
            Representation::Mixed
        },
        quality: if structured {
            Quality::Structured
        } else {
            Quality::Fallback
        },
        syntax_status: if !structured || names.any_aliased() {
            SyntaxStatus::NotJava
        } else {
            SyntaxStatus::Unchecked
        },
        compile_status: CompileStatus::NotAttempted,
        semantic_validation: SemanticValidation::Unproven,
        verification: VerificationStatus::NotPerformed,
        execution,
        outcome: RecoveryOutcome::Produced,
        content,
        text: emitted.text,
        source_map,
        regions,
        lambdas,
        concats,
        accessors,
        bridges,
        news,
        fields: field_records,
        enum_switches,
        init,
        declaration: declaration_record,
        fallbacks,
        aliased_names,
        diagnostics,
        method,
        rules,
        evidence,
        artifact,
    };
    debug_assert!(
        report.evidence.agrees_with(&report),
        "the evidence status list disagrees with the payload it describes"
    );
    report
}

/// Captures the class initializer's already-built top-level statements and the field identities
/// proved by this same recovery's `field@1` plan. The returned candidates are an adapter handoff,
/// never a second artifact or a projection decision.
fn class_initializer_candidates(
    request: &RecoveryRequest<'_>,
    program: &build::Program,
    fields: &field::Plan,
    budget: &mut Budget,
) -> Result<ClassInitializerCandidates, StopReason> {
    crate::stop::poll(budget, None)?;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        1,
        None,
    )?;
    let member = request
        .ir
        .declaration()
        .map(|declaration| declaration.identity().clone());
    let has_exception_handlers = request
        .ir
        .code()
        .is_none_or(|code| code.exception_handler_count != 0 || code.stopped_at.is_some());
    let mut steps = Vec::new();
    for (order, statement) in program.stmts.iter().enumerate() {
        let bci = statement.origin.primary().bci();
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            Some(bci),
        )?;
        crate::stop::poll(budget, Some(bci))?;
        let step = match &statement.kind {
            crate::ast::StmtKind::FieldAssign {
                receiver,
                name: spelled_name,
                op,
                value,
            } => match fields.claim(bci) {
                Some((evidence, shape))
                    if evidence.access == FieldAccess::Write && shape.writes() =>
                {
                    charge_expression_tree(value, budget)?;
                    let field_reads = class_initializer_field_reads(value, fields, budget)?;
                    let source_items = u64::try_from(statement.origin.derived().len())
                        .unwrap_or(u64::MAX)
                        .saturating_add(1);
                    crate::stop::charge(
                        budget,
                        jarde_reader::budget::CountedBudgetDimension::IrItems,
                        source_items.saturating_add(4),
                        Some(bci),
                    )?;
                    crate::stop::poll(budget, Some(bci))?;
                    ClassInitializerStep::FieldWrite(Box::new(ClassInitializerFieldWrite {
                        order,
                        bci: evidence.bci,
                        owner: evidence.owner.clone(),
                        name: evidence.name.clone(),
                        spelled_name: spelled_name.clone(),
                        descriptor: evidence.descriptor.clone(),
                        is_static: evidence.is_static,
                        has_receiver: receiver.is_some(),
                        op: *op,
                        source: statement.origin.clone(),
                        value: value.clone(),
                        field_reads,
                    }))
                }
                _ => ClassInitializerStep::Other {
                    order,
                    bci,
                    kind: ClassInitializerStatementKind::UnattributedFieldWrite,
                },
            },
            kind => ClassInitializerStep::Other {
                order,
                bci,
                kind: class_initializer_statement_kind(kind),
            },
        };
        steps.push(step);
    }
    Ok(ClassInitializerCandidates {
        member,
        has_exception_handlers,
        steps,
    })
}

/// Retains the small top-level AST shapes needed by the bounded enum constructor proof.
fn class_enum_constructor_candidates(
    request: &RecoveryRequest<'_>,
    program: &build::Program,
    fields: &field::Plan,
    code: &jarde_reader::classfile::MethodCodeFacts,
    budget: &mut Budget,
) -> Result<ClassEnumConstructorCandidates, StopReason> {
    crate::stop::poll(budget, None)?;
    let member = request
        .ir
        .declaration()
        .map(|declaration| declaration.identity().clone());
    let has_exception_handlers = code.exception_handler_count != 0
        || code.exception_handler_count as usize != code.exception_handlers.len()
        || code.stopped_at.is_some();
    let mut steps = Vec::with_capacity(program.stmts.len());
    for (order, statement) in program.stmts.iter().enumerate() {
        let bci = statement.origin.primary().bci();
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            u64::try_from(statement.origin.derived().len())
                .unwrap_or(u64::MAX)
                .saturating_add(1),
            Some(bci),
        )?;
        crate::stop::poll(budget, Some(bci))?;
        let kind = match &statement.kind {
            StmtKind::ConstructorCall { target, args } => {
                for argument in args {
                    charge_expression_tree(argument, budget)?;
                }
                ClassEnumConstructorStepKind::ConstructorCall {
                    target: *target,
                    args: args.clone(),
                }
            }
            StmtKind::Expr(expression) => {
                charge_expression_tree(expression, budget)?;
                ClassEnumConstructorStepKind::Expression(expression.clone())
            }
            StmtKind::FieldAssign {
                receiver,
                name,
                op,
                value,
            } => {
                if let Some(receiver) = receiver {
                    charge_expression_tree(receiver, budget)?;
                }
                charge_expression_tree(value, budget)?;
                crate::stop::charge(
                    budget,
                    jarde_reader::budget::CountedBudgetDimension::IrItems,
                    1,
                    Some(bci),
                )?;
                let field = fields.claim(bci).and_then(|(evidence, shape)| {
                    (evidence.access == FieldAccess::Write && shape.writes()).then(|| {
                        ClassEnumConstructorField {
                            bci: evidence.bci,
                            owner: evidence.owner.clone(),
                            name: evidence.name.clone(),
                            descriptor: evidence.descriptor.clone(),
                            is_static: evidence.is_static,
                        }
                    })
                });
                ClassEnumConstructorStepKind::FieldWrite {
                    field,
                    spelled_name: name.clone(),
                    receiver: receiver.clone(),
                    op: *op,
                    value: value.clone(),
                }
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    charge_expression_tree(value, budget)?;
                }
                ClassEnumConstructorStepKind::Return {
                    value: value.clone(),
                }
            }
            other => ClassEnumConstructorStepKind::Other(class_initializer_statement_kind(other)),
        };
        steps.push(ClassEnumConstructorStep {
            order,
            bci,
            source: statement.origin.clone(),
            kind,
        });
    }
    let complete = !program.ragged
        && program.statements == program.stmts.len()
        && code.stopped_at.is_none()
        && matches!(code.execution, ExecutionReport::Complete { .. })
        && code.exception_handler_count as usize == code.exception_handlers.len()
        && code.instructions.len() == code.operands().len();
    Ok(ClassEnumConstructorCandidates {
        member,
        complete,
        has_exception_handlers,
        steps,
    })
}

/// Retains every field-shaped RHS node only when the same method's field plan proves its identity.
fn class_initializer_field_reads(
    expression: &crate::ast::Expr,
    fields: &field::Plan,
    budget: &mut Budget,
) -> Result<Option<Vec<ClassInitializerFieldRead>>, StopReason> {
    let mut reads = Vec::new();
    let mut complete = true;
    visit_class_initializer_field_reads(expression, fields, budget, &mut reads, &mut complete)?;
    Ok(complete.then_some(reads))
}

fn visit_class_initializer_field_reads(
    expression: &crate::ast::Expr,
    fields: &field::Plan,
    budget: &mut Budget,
    reads: &mut Vec<ClassInitializerFieldRead>,
    complete: &mut bool,
) -> Result<(), StopReason> {
    use crate::ast::ExprKind;

    match &expression.kind {
        ExprKind::Field { receiver, .. } => {
            let bci = expression.origin.primary().bci();
            crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::IrItems,
                1,
                Some(bci),
            )?;
            crate::stop::poll(budget, Some(bci))?;
            match fields.claim(bci) {
                Some((evidence, shape))
                    if evidence.access == FieldAccess::Read && !shape.writes() =>
                {
                    reads.push(ClassInitializerFieldRead {
                        bci: evidence.bci,
                        owner: evidence.owner.clone(),
                        name: evidence.name.clone(),
                        descriptor: evidence.descriptor.clone(),
                        is_static: evidence.is_static,
                    });
                }
                _ => *complete = false,
            }
            visit_class_initializer_field_reads(receiver, fields, budget, reads, complete)?;
        }
        ExprKind::Call { receiver, args, .. } => {
            if let Some(receiver) = receiver {
                visit_class_initializer_field_reads(receiver, fields, budget, reads, complete)?;
            }
            for arg in args {
                visit_class_initializer_field_reads(arg, fields, budget, reads, complete)?;
            }
        }
        ExprKind::New {
            qualifier, args, ..
        } => {
            if let Some(qualifier) = qualifier {
                visit_class_initializer_field_reads(qualifier, fields, budget, reads, complete)?;
            }
            for arg in args {
                visit_class_initializer_field_reads(arg, fields, budget, reads, complete)?;
            }
        }
        ExprKind::Lambda { body, .. } => {
            visit_class_initializer_field_reads(body, fields, budget, reads, complete)?;
        }
        ExprKind::MethodReference { qualifier, .. }
        | ExprKind::ArrayLength { array: qualifier }
        | ExprKind::Cast {
            value: qualifier, ..
        }
        | ExprKind::InstanceOf {
            value: qualifier, ..
        }
        | ExprKind::Not { value: qualifier }
        | ExprKind::Neg { value: qualifier }
        | ExprKind::PostIncrement { target: qualifier } => {
            visit_class_initializer_field_reads(qualifier, fields, budget, reads, complete)?;
        }
        ExprKind::Index { array, index } => {
            visit_class_initializer_field_reads(array, fields, budget, reads, complete)?;
            visit_class_initializer_field_reads(index, fields, budget, reads, complete)?;
        }
        ExprKind::NewArray {
            lengths,
            initializers,
            ..
        } => {
            for length in lengths {
                visit_class_initializer_field_reads(length, fields, budget, reads, complete)?;
            }
            if let Some(initializers) = initializers {
                for value in initializers {
                    visit_class_initializer_field_reads(value, fields, budget, reads, complete)?;
                }
            }
        }
        ExprKind::Binary { left, right, .. } => {
            visit_class_initializer_field_reads(left, fields, budget, reads, complete)?;
            visit_class_initializer_field_reads(right, fields, budget, reads, complete)?;
        }
        ExprKind::Conditional {
            test,
            when_true,
            when_false,
        } => {
            visit_class_initializer_field_reads(test, fields, budget, reads, complete)?;
            visit_class_initializer_field_reads(when_true, fields, budget, reads, complete)?;
            visit_class_initializer_field_reads(when_false, fields, budget, reads, complete)?;
        }
        ExprKind::Concat { parts } => {
            for part in parts {
                visit_class_initializer_field_reads(&part.value, fields, budget, reads, complete)?;
            }
        }
        ExprKind::Local(_)
        | ExprKind::Integer(_)
        | ExprKind::Boolean(_)
        | ExprKind::Long(_)
        | ExprKind::Float(_)
        | ExprKind::Double(_)
        | ExprKind::Str(_)
        | ExprKind::Null
        | ExprKind::ClassLiteral { .. }
        | ExprKind::Path(_)
        | ExprKind::QualifiedThis { .. }
        | ExprKind::Super { .. } => {}
    }
    Ok(())
}

/// Counts every AST node and auxiliary list item the candidate clones, before cloning it.
fn charge_expression_tree(
    expression: &crate::ast::Expr,
    budget: &mut Budget,
) -> Result<(), StopReason> {
    charge_expression_tree_at_depth(expression, budget, 0)
}

fn charge_expression_tree_at_depth(
    expression: &crate::ast::Expr,
    budget: &mut Budget,
    depth: usize,
) -> Result<(), StopReason> {
    use crate::ast::ExprKind;

    let at = expression.origin.primary().bci();
    if depth > build::MAX_VALUE_DEPTH {
        return Err(StopReason::Interrupted {
            code: crate::stop::RECURSION_BOUND_CODE,
            at: Some(at),
        });
    }
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        1,
        Some(at),
    )?;
    crate::stop::poll(budget, Some(at))?;
    match &expression.kind {
        ExprKind::Call { receiver, args, .. } => {
            if let Some(receiver) = receiver {
                charge_expression_tree_at_depth(receiver, budget, depth + 1)?;
            }
            for arg in args {
                charge_expression_tree_at_depth(arg, budget, depth + 1)?;
            }
        }
        ExprKind::New {
            qualifier, args, ..
        } => {
            if let Some(qualifier) = qualifier {
                charge_expression_tree_at_depth(qualifier, budget, depth + 1)?;
            }
            for arg in args {
                charge_expression_tree_at_depth(arg, budget, depth + 1)?;
            }
        }
        ExprKind::Lambda { params, body } => {
            crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::IrItems,
                u64::try_from(params.len()).unwrap_or(u64::MAX),
                Some(at),
            )?;
            charge_expression_tree_at_depth(body, budget, depth + 1)?;
        }
        ExprKind::MethodReference { qualifier, .. }
        | ExprKind::ArrayLength { array: qualifier }
        | ExprKind::Cast {
            value: qualifier, ..
        }
        | ExprKind::InstanceOf {
            value: qualifier, ..
        }
        | ExprKind::Not { value: qualifier }
        | ExprKind::Neg { value: qualifier }
        | ExprKind::PostIncrement { target: qualifier } => {
            charge_expression_tree_at_depth(qualifier, budget, depth + 1)?;
        }
        ExprKind::Field { receiver, .. } => {
            charge_expression_tree_at_depth(receiver, budget, depth + 1)?;
        }
        ExprKind::Index { array, index } => {
            charge_expression_tree_at_depth(array, budget, depth + 1)?;
            charge_expression_tree_at_depth(index, budget, depth + 1)?;
        }
        ExprKind::NewArray {
            lengths,
            initializers,
            ..
        } => {
            for length in lengths {
                charge_expression_tree_at_depth(length, budget, depth + 1)?;
            }
            if let Some(initializers) = initializers {
                for value in initializers {
                    charge_expression_tree_at_depth(value, budget, depth + 1)?;
                }
            }
        }
        ExprKind::Binary { left, right, .. } => {
            charge_expression_tree_at_depth(left, budget, depth + 1)?;
            charge_expression_tree_at_depth(right, budget, depth + 1)?;
        }
        ExprKind::Conditional {
            test,
            when_true,
            when_false,
        } => {
            charge_expression_tree_at_depth(test, budget, depth + 1)?;
            charge_expression_tree_at_depth(when_true, budget, depth + 1)?;
            charge_expression_tree_at_depth(when_false, budget, depth + 1)?;
        }
        ExprKind::Concat { parts } => {
            crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::IrItems,
                u64::try_from(parts.len()).unwrap_or(u64::MAX),
                Some(at),
            )?;
            for part in parts {
                charge_expression_tree_at_depth(&part.value, budget, depth + 1)?;
            }
        }
        ExprKind::Integer(_)
        | ExprKind::Local(_)
        | ExprKind::Boolean(_)
        | ExprKind::Long(_)
        | ExprKind::Float(_)
        | ExprKind::Double(_)
        | ExprKind::Str(_)
        | ExprKind::Null
        | ExprKind::ClassLiteral { .. }
        | ExprKind::Path(_)
        | ExprKind::QualifiedThis { .. }
        | ExprKind::Super { .. } => {}
    }
    Ok(())
}

fn class_initializer_statement_kind(kind: &crate::ast::StmtKind) -> ClassInitializerStatementKind {
    use crate::ast::StmtKind;

    match kind {
        StmtKind::Declare { .. } => ClassInitializerStatementKind::Declaration,
        StmtKind::Assign { .. } => ClassInitializerStatementKind::LocalAssignment,
        StmtKind::Expr(_) => ClassInitializerStatementKind::Expression,
        StmtKind::FieldAssign { .. } => ClassInitializerStatementKind::UnattributedFieldWrite,
        StmtKind::IndexAssign { .. } => ClassInitializerStatementKind::ArrayWrite,
        StmtKind::ConstructorCall { .. } => ClassInitializerStatementKind::ConstructorCall,
        StmtKind::Return { .. } => ClassInitializerStatementKind::Return,
        StmtKind::Throw { .. } => ClassInitializerStatementKind::Throw,
        StmtKind::If { .. } => ClassInitializerStatementKind::Conditional,
        StmtKind::While { .. }
        | StmtKind::For { .. }
        | StmtKind::ForEach { .. }
        | StmtKind::DoWhile { .. }
        | StmtKind::Break { .. }
        | StmtKind::Continue { .. } => ClassInitializerStatementKind::Loop,
        StmtKind::Switch { .. } => ClassInitializerStatementKind::Switch,
        StmtKind::Try { .. } => ClassInitializerStatementKind::Try,
        StmtKind::Synchronized { .. } => ClassInitializerStatementKind::Synchronized,
        StmtKind::Fallback { .. } => ClassInitializerStatementKind::Fallback,
    }
}

/// One category's materialization across the several plans that publish into it.
///
/// `RuleDetails` is one category and nine rules: the phase delivers their records in one order, and
/// the category is `Complete` only when it reached the end of every one of them. The accumulator is
/// where "reached the end" of several materializations becomes the one state the list states, and it
/// never deletes a record an earlier plan delivered.
struct Delivery {
    /// Whether the phase reached the end of every plan seen so far.
    complete: bool,
    /// How many owning records the category holds so far.
    records: u64,
}

impl Delivery {
    /// A category nothing has been materialized for yet, in a phase that has not stopped.
    fn new(phase: &EvidencePhase) -> Self {
        Self {
            complete: !phase.stopped(),
            records: 0,
        }
    }

    /// Takes one plan's materialization into the category's own account.
    fn take<T>(&mut self, (records, reached): (Vec<T>, Materialized)) -> Vec<T> {
        self.records = self
            .records
            .saturating_add(u64::try_from(records.len()).unwrap_or(u64::MAX));
        if !matches!(reached, Materialized::Complete) {
            self.complete = false;
        }
        records
    }

    /// The same, for a plan that publishes at most one record.
    fn take_one<T>(&mut self, (record, reached): (Option<T>, Materialized)) -> Option<T> {
        self.take((record.into_iter().collect::<Vec<T>>(), reached))
            .into_iter()
            .next()
    }

    /// What the category reached, as the list states it.
    fn reached(&self) -> Materialized {
        if self.complete {
            Materialized::Complete
        } else if self.records == 0 {
            Materialized::None
        } else {
            Materialized::Partial {
                delivered: self.records,
            }
        }
    }
}

/// One replaced name, as a selected `NameDetails` category states it.
fn aliased_name(name: &crate::names::RenderedName) -> String {
    crate::demand_counts::record_built(RecoveryEvidenceKind::NameDetails);
    format!(
        "slot {} written as `{}` (source spelling `{}`)",
        name.slot(),
        name.text(),
        name.raw().unwrap_or("")
    )
}

/// One count as the report states it.
fn count_of(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// Whether one region is inside the selected driver range.
///
/// The range selects the records that *intersect* it and a record is kept or dropped as a unit: the
/// region's own blocks are the positions it states for itself.
fn in_driver_range(region: &Region, range: Option<crate::evidence::BytecodeRange>) -> bool {
    range.is_none_or(|range| range.intersects_any(region.blocks().iter().map(|block| block.bci())))
}

/// One region record, with its fallbacks stated: what a selected `RegionDetails` category holds for
/// one region, built when the evidence phase materializes it.
///
/// The construction is counted **here**, in the function that builds the record, rather than at the
/// call site: a record built anywhere and then dropped is a record that was built, and the port
/// (`crate::demand_counts`) exists to say so.
fn region_record(region: &Region) -> RegionRecord {
    crate::demand_counts::record_built(RecoveryEvidenceKind::RegionDetails);
    let blocks: Vec<u32> = region.blocks().iter().map(|block| block.bci()).collect();
    let reasons = region.fallbacks();
    RegionRecord {
        bci: blocks.first().copied().unwrap_or(0),
        structured: region.is_structured(),
        blocks,
        code: reasons.first().map(FallbackReason::code),
        message: reasons.first().map(FallbackReason::message),
        rule: region.rule(),
    }
}

impl EvidencePayload for RecoveryReport {
    /// How many owning records of one category this report holds right now.
    ///
    /// The count is read off the payload itself — the same fields the status list is checked against
    /// — so the check cannot drift from what a caller receives.
    fn owning_records(&self, kind: RecoveryEvidenceKind) -> u64 {
        match kind {
            RecoveryEvidenceKind::SourceMap => count_of(self.source_map.len()),
            RecoveryEvidenceKind::RegionDetails => count_of(self.regions.len()),
            RecoveryEvidenceKind::RuleDetails => count_of(
                self.lambdas.len()
                    + self.concats.len()
                    + self.accessors.len()
                    + self.bridges.len()
                    + self.news.len()
                    + self.fields.len()
                    + self.enum_switches.len()
                    + usize::from(self.init.is_some())
                    + usize::from(self.declaration.is_some()),
            ),
            RecoveryEvidenceKind::NameDetails => count_of(self.aliased_names.len()),
            // The read evidence a presentation consumed is published beside this report by the entry
            // that performed the read: this layer materializes none of it yet.
            RecoveryEvidenceKind::ReadDetails => 0,
        }
    }
}

/// The content of a committed artifact, from the statements its emission wrote: an artifact that
/// holds only the envelope, the reasons and the quoted bytecode is [`RecoveryContent::ExplanationOnly`],
/// and one statement the emitter spelled as Java makes it [`RecoveryContent::ContainsStatements`].
fn content_of(emitted: &Emitted) -> RecoveryContent {
    if emitted.statements == 0 {
        RecoveryContent::ExplanationOnly
    } else {
        RecoveryContent::ContainsStatements
    }
}

/// The report of a run that stopped: no text, no segments, and an execution plane that says so.
fn stopped(
    method: String,
    profile: RecoveryProfile,
    selection: &RecoveryEvidenceRequest,
    reason: StopReason,
    budget: &Budget,
) -> RecoveryReport {
    let execution = stop_execution(&reason, budget.usage());
    RecoveryReport {
        method,
        profile,
        rules: Vec::new(),
        representation: Representation::Bytecode,
        quality: Quality::Fallback,
        syntax_status: SyntaxStatus::NotJava,
        compile_status: CompileStatus::NotAttempted,
        semantic_validation: SemanticValidation::Unproven,
        verification: VerificationStatus::NotPerformed,
        execution,
        outcome: RecoveryOutcome::Stopped(reason.clone()),
        // Nothing was committed, so there is no content to describe. A stop is never classified from
        // whatever the failed run had built or written before it refused.
        content: RecoveryContent::NotProduced,
        text: String::new(),
        source_map: SourceMap::default(),
        regions: Vec::new(),
        lambdas: Vec::new(),
        concats: Vec::new(),
        accessors: Vec::new(),
        bridges: Vec::new(),
        news: Vec::new(),
        fields: Vec::new(),
        enum_switches: Vec::new(),
        init: None,
        declaration: None,
        fallbacks: Vec::new(),
        // Nothing of a selected category was materialized, and the status list says exactly that
        // rather than leaving an empty `Vec` to be read as "this body has no such evidence".
        evidence: RecoveryEvidence::pending(selection),
        // No artifact was committed, so this run publishes no binding and compares nothing: the
        // verdict states that the artifact the request named could not be checked against one.
        artifact: RecoveryArtifact::not_produced(selection.expected_artifact()),
        aliased_names: Vec::new(),
        diagnostics: vec![stop_diagnostic(&reason)],
    }
}

/// The report of a run whose own evidence selection cannot be answered for this body.
///
/// The refusal is the *request's* fact, not the body's: the decode succeeded, the run could have
/// presented the method, and what it cannot do is apply the selection the caller stated. So nothing
/// is presented — a refused selection is never widened into a full-evidence delivery, and a category
/// this entry does not materialize is never answered with nothing — every category the request
/// selected states `NotPerformed`, and the report carries the refusal's own code and position.
fn refused(
    method: String,
    profile: RecoveryProfile,
    selection: &RecoveryEvidenceRequest,
    refusal: EvidenceRefusal,
    budget: &Budget,
) -> RecoveryReport {
    let reason = StopReason::EvidenceRefused {
        code: refusal.code,
        at: refusal.at,
        message: refusal.message,
    };
    RecoveryReport {
        method,
        profile,
        rules: Vec::new(),
        representation: Representation::Bytecode,
        quality: Quality::Fallback,
        syntax_status: SyntaxStatus::NotJava,
        compile_status: CompileStatus::NotAttempted,
        semantic_validation: SemanticValidation::Unproven,
        verification: VerificationStatus::NotPerformed,
        execution: stop_execution(&reason, budget.usage()),
        outcome: RecoveryOutcome::Stopped(reason.clone()),
        content: RecoveryContent::NotProduced,
        text: String::new(),
        source_map: SourceMap::default(),
        regions: Vec::new(),
        lambdas: Vec::new(),
        concats: Vec::new(),
        accessors: Vec::new(),
        bridges: Vec::new(),
        news: Vec::new(),
        fields: Vec::new(),
        enum_switches: Vec::new(),
        init: None,
        declaration: None,
        fallbacks: Vec::new(),
        evidence: RecoveryEvidence::pending(selection),
        // No artifact was committed — the selection itself was refused before one — so this run
        // publishes no binding and compares nothing.
        artifact: RecoveryArtifact::not_produced(selection.expected_artifact()),
        aliased_names: Vec::new(),
        diagnostics: vec![stop_diagnostic(&reason)],
    }
}

/// The execution plane one stop states, in the fact layer's own vocabulary.
fn stop_execution(reason: &StopReason, usage: UsageSnapshot) -> ExecutionReport {
    match reason {
        StopReason::IrTableMissing { .. } => ExecutionReport::Partial {
            reason: TerminationReason::Unsupported {
                code: "jre_ir_table_missing".to_string(),
            },
            usage,
        },
        StopReason::EvidenceRefused { code, .. } => ExecutionReport::Partial {
            reason: TerminationReason::Unsupported {
                code: (*code).to_string(),
            },
            usage,
        },
        StopReason::Budget { dimension, .. } => ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::from(*dimension),
            },
            usage,
        },
        StopReason::Cancelled { .. } => ExecutionReport::Cancelled { usage },
        StopReason::Interrupted { code, .. } => ExecutionReport::Partial {
            reason: TerminationReason::Error {
                code: (*code).to_string(),
            },
            usage,
        },
    }
}

/// The one diagnostic a stop states: its code, its severity and its sentence, in the vocabulary the
/// fact layer already uses.
fn stop_diagnostic(reason: &StopReason) -> Diagnostic {
    let (code, severity) = match reason {
        StopReason::IrTableMissing { .. } => ("jre_ir_table_missing", DiagnosticSeverity::Error),
        StopReason::EvidenceRefused { code, .. } => (*code, DiagnosticSeverity::Error),
        StopReason::Budget { .. } => ("jre_output_budget", DiagnosticSeverity::Error),
        StopReason::Cancelled { .. } => ("jre_cancelled", DiagnosticSeverity::Warning),
        StopReason::Interrupted { code, .. } => (*code, DiagnosticSeverity::Error),
    };
    let message = match reason {
        StopReason::IrTableMissing { table } => format!(
            "the payload of this run has no {table} table, so no body can be presented from it"
        ),
        StopReason::Budget {
            dimension,
            written,
            limit,
            at,
        } => format!(
            "the run stopped on {dimension:?} after {written} byte(s) of {limit}, at {}",
            at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
        ),
        StopReason::Cancelled { at } => format!(
            "the run was cancelled{}",
            at.map_or(String::new(), |bci| format!(" at BCI {bci}"))
        ),
        StopReason::Interrupted { code, at } => match *code {
            // The three ways this run can be interrupted at a node are different facts about it —
            // "the input went deeper than the bound", "the walk was back inside a structure it is
            // already building" and "a budget poll refused" — and the message names which one
            // stopped it, at which node. The budget's own wording is the one it always had.
            crate::stop::RECURSION_REENTRY_CODE => format!(
                "the recovery recursion re-entered a block this run had already entered and cannot \
                 complete it: the run stopped at the recursion bound ({code}) at {}",
                at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
            ),
            crate::stop::RECURSION_BOUND_CODE => format!(
                "the recovery recursion reached its explicit depth bound ({code}) at {}",
                at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
            ),
            crate::stop::BUDGET_INTERRUPTED_CODE => format!(
                "the budget interrupted the run ({code}) at {}",
                at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
            ),
            crate::stop::SOURCE_MAP_MISMATCH_CODE => format!(
                "the source-map replay of this run did not agree with the artifact the committing                  pass wrote: the map cannot be attached to that text, so it is not delivered and the                  artifact is left exactly as it was ({code})"
            ),
            _ => format!(
                "the run was interrupted ({code}) at {}",
                at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
            ),
        },
        StopReason::EvidenceRefused { message, .. } => message.clone(),
    };
    diagnostic(code, severity, &message)
}

/// One diagnostic in the fact layer's vocabulary.
fn diagnostic(code: &str, severity: DiagnosticSeverity, message: &str) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity,
        message: message.to_string(),
        provenance: None,
    }
}

#[cfg(test)]
mod class_initializer_candidate_tests {
    use super::*;
    use jarde_reader::budget::{Budget, Limits};

    #[test]
    fn candidate_expression_charging_stops_at_the_shared_value_depth_bound() {
        let mut expression = crate::ast::Expr::direct(crate::ast::ExprKind::Integer(1), 7);
        for _ in 0..=build::MAX_VALUE_DEPTH {
            expression = crate::ast::Expr::direct(
                crate::ast::ExprKind::Not {
                    value: Box::new(expression),
                },
                7,
            );
        }
        let mut budget = Budget::new(Limits {
            ir_items: 1_000,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });

        let stopped = charge_expression_tree(&expression, &mut budget)
            .expect_err("future AST shapes must not make this traversal unbounded");
        assert_eq!(
            stopped,
            StopReason::Interrupted {
                code: crate::stop::RECURSION_BOUND_CODE,
                at: Some(7),
            }
        );
    }

    #[test]
    fn candidate_expression_charging_includes_a_member_creation_qualifier() {
        let mut qualifier = crate::ast::Expr::direct(crate::ast::ExprKind::Integer(1), 7);
        for _ in 0..=build::MAX_VALUE_DEPTH {
            qualifier = crate::ast::Expr::direct(
                crate::ast::ExprKind::Not {
                    value: Box::new(qualifier),
                },
                7,
            );
        }
        let expression = crate::ast::Expr::direct(
            crate::ast::ExprKind::New {
                ty: "sample.SimpleOuter$Inner".to_string(),
                qualifier: Some(Box::new(qualifier)),
                member_name: Some("Inner".to_string()),
                diamond: false,
                args: Vec::new(),
            },
            8,
        );
        let mut budget = Budget::new(Limits {
            ir_items: 1_000,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });

        let stopped = charge_expression_tree(&expression, &mut budget)
            .expect_err("a qualified receiver is part of the candidate's bounded AST");
        assert_eq!(
            stopped,
            StopReason::Interrupted {
                code: crate::stop::RECURSION_BOUND_CODE,
                at: Some(7),
            }
        );
    }
}
