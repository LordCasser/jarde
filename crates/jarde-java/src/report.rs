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
use jarde_jvm::method_ir::{Definition, MethodIr, Slot, SsaTable};
use jarde_reader::budget::{Budget, BudgetDimension, UsageSnapshot};
use jarde_reader::classfile::{DescriptorKind, VerificationStatus, descriptor_facts};
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
    ACC_ANNOTATION, ACC_INTERFACE, ClassMembers, ConstantValue, FieldAccess, Operation,
    RecoveryFacts,
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
    /// Exact static member targets selected by class-source relation proof.
    pub static_member_target: Option<&'a ProvedStaticMemberTarget>,
    /// Class-source-only direct functional return target. Method-only recovery has none.
    pub typed_functional_target: Option<TypedFunctionalTarget>,
    /// Exact interface-special targets whose Java source qualifier and default binding were proved
    /// by the facade's selected-definition reads. Direct recovery has no such environment and
    /// therefore leaves interface-qualified `super` calls refused.
    pub interface_super_calls: &'a [ProvedInterfaceSuperCall],
    /// Exact invocation sites whose source reference widening and target declaration were proved
    /// against the selected class-source environment. A method-only request has none.
    pub reference_overload_calls: &'a [ProvedReferenceOverloadCall],
    /// Exact invocation arguments whose snapshot class-file header chain proved the reference
    /// widening. A method-only request has none.
    pub snapshot_hierarchy_widenings: &'a [ProvedSnapshotHierarchyWidening],
    /// Exact superclass field writes proved by the selected class-source environment.
    pub superclass_field_writes: &'a [ProvedSuperclassFieldWrite],
    /// Exact captured-outer reads proved by the selected class-source family assembly.
    /// A direct method request has no lexical family and supplies none.
    pub captured_outer_reads: &'a [ProvedCapturedOuterRead],
    /// Exact calls through a selected outer-super bridge, closed by the family proof.
    pub outer_super_calls: &'a [ProvedOuterSuperCall],
    /// Whether this run's presentation keeps every member class in the pool's own `$` spelling —
    /// no fold projection will rewrite nested names to source nesting in the text this run's
    /// artifacts are placed into. Under that presentation a pool-spelled class literal is a
    /// top-level class in the compiled text, so the structural-reflection reads of
    /// `java/lang/Class` (`getSimpleName`, `getEnclosingClass` and kin) would answer from nesting
    /// metadata the text does not state; the build refuses such a call rather than publishing a
    /// compilable text that behaves differently. A presentation that folds members — which spells
    /// a folded member's own name the source way — states `false`, the default.
    pub pool_spelled_members: bool,
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

/// Source-path proof for a static member class. It intentionally has no capture-field identity.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedStaticMemberTarget {
    pub definition: PhysicalDefinitionId,
    pub constructor: PhysicalMethodId,
    pub owner: String,
    pub outer: String,
    pub simple_name: String,
    pub constructor_descriptor: String,
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

/// A physical capture-field read replaced by a root method's exactly proved parameter value.
/// The superclass projection admits the same handoff for a root **local**: the slot is then the
/// local's own, and the presented type is the one the same-run allocation argument carried.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedCapturedParameterRead {
    pub method: PhysicalMethodId,
    pub read_bci: u32,
    pub field_owner: String,
    pub field_name: String,
    pub field_descriptor: String,
    pub parameter_slot: u16,
    pub parameter_name: String,
    pub parameter_presented: Option<Type>,
    pub constructor: PhysicalMethodId,
    pub constructor_write_bci: u32,
}

/// Trusted, non-serialized handoff for one physical static bridge call. The bridge instruction's
/// BCI belongs to `bridge`, while `call_bci` and `capture_read_bci` belong to `caller`.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedOuterSuperCall {
    pub caller: PhysicalMethodId,
    pub call_bci: u32,
    pub capture_read_bci: u32,
    pub argument_bcis: Vec<u32>,
    pub bridge: PhysicalMethodId,
    pub bridge_invoke_bci: u32,
    pub outer_source_name: String,
    pub target_owner: String,
    pub target_name: String,
    pub target_descriptor: String,
}

/// One exact `invokespecial InterfaceMethodref` target proved writable as `I.super.m(...)`.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedInterfaceSuperCall {
    pub owner: String,
    pub name: String,
    pub descriptor: String,
}

/// One selected invocation's safe upcast and unique source method binding.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedReferenceOverloadCall {
    pub bci: u32,
    pub source: String,
    pub target: String,
}

/// One invocation argument's safe upcast, proved by the snapshot's own class files: the presented
/// type and the required type are both physical classes of the analyzed snapshot, and the
/// presented type's class-file headers state the chain (superclass and interfaces, transitively,
/// bounded) up to the required one.  A chain name absent from the snapshot proved nothing, so a
/// platform target or a platform intermediate keeps its refusal.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedSnapshotHierarchyWidening {
    pub bci: u32,
    pub source: String,
    pub target: String,
}

/// A selected parent declaration and one exact field instruction. The field plan still checks the
/// receiver's SSA type at this BCI before claiming the write.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedSuperclassFieldWrite {
    pub bci: u32,
    pub source: String,
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
    /// Same-run AST nodes retained for an exact, bounded enum suffix projection.
    pub statements: Vec<crate::ast::Stmt>,
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
    /// Same-run unbound synthetic-lambda helper candidates for bounded class-source projection.
    pub lambda_helpers: Vec<ClassSourceLambdaHelperCandidate>,
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
    /// The same-run AST retained only for bounded class-source projection.
    pub ast: Option<ClassSourceMethodAst>,
    /// The actual receiver expressions of committed own-field writes in this body.
    pub field_receivers: Vec<ClassSourceFieldReceiverSite>,
    /// Presented field-write accessors whose caller-side write selects the field type.
    pub field_write_accessors: Vec<ClassSourceFieldWriteAccessor>,
}

/// One actual emitted receiver, bound to a physical own-field write.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceFieldReceiverSite {
    pub method: jarde_reader::model::PhysicalMethodId,
    pub bci: u32,
    pub owner: Vec<u8>,
    pub name: Vec<u8>,
    pub descriptor: Vec<u8>,
    pub source: ClassSourceFieldReceiverSource,
}

/// The source-level form that selected an own instance field at one emitted write.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassSourceFieldReceiverSource {
    This,
    Parameter { slot: u16 },
    RawLocal { name: String },
}

/// A presented synthetic setter and the physical field write its caller emits.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceFieldWriteAccessor {
    pub callee: jarde_reader::model::PhysicalMethodId,
    pub owner: Vec<u8>,
    pub name: Vec<u8>,
    pub descriptor: Vec<u8>,
}

fn same_class_field_receiver_sites(
    program: &build::Program,
    fields: &field::Plan,
    presented: &std::collections::BTreeSet<u32>,
    request: &RecoveryRequest<'_>,
    names: &NameTable,
    reuse: &crate::reuse::Plan,
    init_record: &InitRecord,
    operations: &Operations,
    ssa: &SsaTable,
    budget: &mut Budget,
) -> Result<
    (
        Vec<ClassSourceFieldReceiverSite>,
        Vec<ClassSourceFieldWriteAccessor>,
    ),
    StopReason,
> {
    use crate::ast::{ExprKind, Stmt, StmtKind, Type};
    use jarde_jvm::method_ir::{Definition, Slot};

    enum Node<'a> {
        Stmt(&'a Stmt),
        Expr(&'a crate::ast::Expr),
    }

    fn push_expression_children<'a>(expression: &'a crate::ast::Expr, pending: &mut Vec<Node<'a>>) {
        use crate::ast::ExprKind;
        match &expression.kind {
            ExprKind::LocalAssign { value, .. }
            | ExprKind::InstanceOf { value, .. }
            | ExprKind::Cast { value, .. }
            | ExprKind::Not { value }
            | ExprKind::Neg { value } => pending.push(Node::Expr(value)),
            ExprKind::Call { receiver, args, .. } => {
                pending.extend(args.iter().map(Node::Expr));
                pending.extend(receiver.iter().map(|value| Node::Expr(value)));
            }
            ExprKind::New {
                qualifier, args, ..
            } => {
                pending.extend(args.iter().map(Node::Expr));
                pending.extend(qualifier.iter().map(|value| Node::Expr(value)));
            }
            ExprKind::Lambda { body, .. } => pending.push(Node::Expr(body)),
            ExprKind::MethodReference { qualifier, .. } => {
                pending.push(Node::Expr(qualifier));
            }
            ExprKind::Field { receiver, .. } => pending.push(Node::Expr(receiver)),
            ExprKind::ArrayLength { array } => pending.push(Node::Expr(array)),
            ExprKind::Index { array, index } => {
                pending.push(Node::Expr(index));
                pending.push(Node::Expr(array));
            }
            ExprKind::PostfixUpdate { target, .. } => pending.push(Node::Expr(target)),
            ExprKind::NewArray {
                lengths,
                initializers,
                ..
            } => {
                pending.extend(initializers.iter().flatten().map(Node::Expr));
                pending.extend(lengths.iter().map(Node::Expr));
            }
            ExprKind::Binary { left, right, .. } => {
                pending.push(Node::Expr(right));
                pending.push(Node::Expr(left));
            }
            ExprKind::Conditional {
                test,
                when_true,
                when_false,
            } => {
                pending.push(Node::Expr(when_false));
                pending.push(Node::Expr(when_true));
                pending.push(Node::Expr(test));
            }
            ExprKind::Concat { parts } => {
                pending.extend(parts.iter().rev().map(|part| Node::Expr(&part.value)));
            }
            ExprKind::Local(_)
            | ExprKind::Integer(_)
            | ExprKind::IntegerConstantName { .. }
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

    fn build_ssa_instruction_index<'a>(
        ssa: &'a SsaTable,
        budget: &mut Budget,
    ) -> Result<
        std::collections::HashMap<u32, Option<&'a jarde_jvm::method_ir::SsaInstruction>>,
        StopReason,
    > {
        let mut by_bci = std::collections::HashMap::new();
        for block in ssa.blocks() {
            for instruction in block.instructions() {
                let bci = instruction.bci();
                crate::stop::poll(budget, Some(bci))?;
                crate::stop::charge(
                    budget,
                    jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(bci),
                )?;
                crate::stop::charge(
                    budget,
                    jarde_reader::budget::CountedBudgetDimension::IrItems,
                    1,
                    Some(bci),
                )?;
                if by_bci.insert(bci, Some(instruction)).is_some() {
                    by_bci.insert(bci, None);
                }
            }
        }
        Ok(by_bci)
    }

    fn receiver_load_slot(
        bci: u32,
        receiver: jarde_jvm::method_ir::ValueId,
        ssa: &SsaTable,
        instructions: &std::collections::HashMap<
            u32,
            Option<&jarde_jvm::method_ir::SsaInstruction>,
        >,
        budget: &mut Budget,
    ) -> Result<Option<(u16, u32, jarde_jvm::method_ir::ValueId)>, StopReason> {
        crate::stop::poll(budget, Some(bci))?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
            1,
            Some(bci),
        )?;
        let value = ssa.value(receiver);
        if value.uses().len() != 1 || value.uses()[0].bci() != Some(bci) {
            return Ok(None);
        }
        let Some(Some(use_instruction)) = instructions.get(&bci) else {
            return Ok(None);
        };
        if use_instruction
            .reads()
            .iter()
            .filter(|(slot, value)| matches!(slot, Slot::Stack(_)) && *value == receiver)
            .count()
            != 1
        {
            return Ok(None);
        }
        let Definition::Instruction { bci: load_bci, .. } = value.def() else {
            return Ok(None);
        };
        let Some(Some(load)) = instructions.get(load_bci) else {
            return Ok(None);
        };
        if !matches!(load.opcode(), 0x19 | 0x2a..=0x2d) {
            return Ok(None);
        }
        if load
            .writes()
            .iter()
            .filter(|(slot, value)| matches!(slot, Slot::Stack(_)) && *value == receiver)
            .count()
            != 1
        {
            return Ok(None);
        }
        let mut local_reads = load.reads().iter().filter_map(|(slot, value)| match slot {
            Slot::Local(slot) => Some((*slot, *value)),
            Slot::Stack(_) => None,
        });
        let Some((slot, entry)) = local_reads.next() else {
            return Ok(None);
        };
        if local_reads.next().is_some() {
            return Ok(None);
        }
        Ok(Some((slot, *load_bci, entry)))
    }

    fn initialized_this_value(
        init: &InitRecord,
        operations: &Operations,
        member: &PhysicalMethodId,
        class_internal: &[u8],
        slot: u16,
        local_value: jarde_jvm::method_ir::ValueId,
        ssa: &SsaTable,
        instructions: &std::collections::HashMap<
            u32,
            Option<&jarde_jvm::method_ir::SsaInstruction>,
        >,
        budget: &mut Budget,
    ) -> Result<bool, StopReason> {
        let Some(init_bci) = init.bci else {
            return Ok(false);
        };
        crate::stop::poll(budget, Some(init_bci))?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
            1,
            Some(init_bci),
        )?;
        if member.name.0.as_slice() != b"<init>"
            || member.descriptor.0.is_empty()
            || init.target != Some(crate::ast::ConstructorTarget::Super)
            || init.class.as_deref() != Some("java/lang/Object")
            || init.declared.as_deref() != std::str::from_utf8(class_internal).ok()
            || !init.presented
            || slot != 0
        {
            return Ok(false);
        }
        let Some(Operation::Invoke(target)) = operations.get(init_bci) else {
            return Ok(false);
        };
        if target.kind() != crate::facts::InvokeKind::Special
            || target.owner() != "java/lang/Object"
            || target.name() != "<init>"
            || target.descriptor() != "()V"
            || target.is_interface_reference()
        {
            return Ok(false);
        }
        let Some(Some(initialize)) = instructions.get(&init_bci) else {
            return Ok(false);
        };
        if initialize.opcode() != 0xb7 {
            return Ok(false);
        }
        let [(Slot::Stack(0), receiver)] = initialize.reads() else {
            return Ok(false);
        };
        let [(Slot::Local(0), initialized)] = initialize.writes() else {
            return Ok(false);
        };
        if *initialized != local_value
            || !matches!(
                ssa.value(local_value).def(),
                Definition::Instruction { bci, .. } if *bci == init_bci
            )
        {
            return Ok(false);
        }
        let Definition::Instruction {
            bci: receiver_load_bci,
            ..
        } = ssa.value(*receiver).def()
        else {
            return Ok(false);
        };
        if *receiver_load_bci >= init_bci {
            return Ok(false);
        }
        let Some(Some(receiver_load)) = instructions.get(receiver_load_bci) else {
            return Ok(false);
        };
        if !matches!(receiver_load.opcode(), 0x19 | 0x2a..=0x2d) {
            return Ok(false);
        }
        let [(Slot::Local(0), entry_this)] = receiver_load.reads() else {
            return Ok(false);
        };
        if !matches!(
            ssa.value(*entry_this).def(),
            Definition::Entry {
                slot: Slot::Local(0),
                ..
            }
        ) || !receiver_load
            .writes()
            .iter()
            .any(|(write_slot, write_value)| {
                matches!(write_slot, Slot::Stack(_)) && write_value == receiver
            })
        {
            return Ok(false);
        }
        Ok(true)
    }

    let Some(declaration) = request.ir.declaration() else {
        return Ok((Vec::new(), Vec::new()));
    };
    let Some(class) = request.facts.method().declaring_class() else {
        return Ok((Vec::new(), Vec::new()));
    };
    let class_internal = class.name().replace('.', "/").into_bytes();
    let class_source = String::from_utf8_lossy(&class_internal).replace('/', ".");
    let method = declaration.identity().clone();
    let Ok(descriptor) = descriptor_facts(
        declaration.descriptor().0.as_slice(),
        DescriptorKind::Method,
    ) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let Some(parameter_slots) = jarde_jvm::method_ir::parameter_positions(
        &descriptor,
        declaration.access_flags() & 0x0008 != 0,
    ) else {
        return Ok((Vec::new(), Vec::new()));
    };

    let mut accessors = Vec::new();
    for site in &program.accessors {
        crate::stop::poll(budget, Some(site.call_site))?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
            1,
            Some(site.call_site),
        )?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            Some(site.call_site),
        )?;
        if !site.presented
            || site.evidence.shape != Some(crate::accessor::AccessorShape::FieldWrite)
        {
            continue;
        }
        let (Some(callee), Some(field)) = (&site.evidence.identity, &site.evidence.field) else {
            continue;
        };
        accessors.push(ClassSourceFieldWriteAccessor {
            callee: callee.clone(),
            owner: field.owner.as_bytes().to_vec(),
            name: field.name.as_bytes().to_vec(),
            descriptor: field.descriptor.as_bytes().to_vec(),
        });
    }

    // Static bodies without formals cannot supply this slice's direct-formal receiver or a
    // raw local initialized from one. Accessor guards above still describe their emitted calls.
    if parameter_slots.is_empty() && declaration.access_flags() & 0x0008 != 0 {
        return Ok((Vec::new(), accessors));
    }

    // Do not walk an unrelated method's full AST. The prior committed-presentation pass already
    // charged this complete AST; this narrow pass is needed only for a committed own instance write.
    let mut has_candidate = false;
    for bci in presented {
        crate::stop::poll(budget, Some(*bci))?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
            1,
            Some(*bci),
        )?;
        if fields.claim(*bci).is_some_and(|(evidence, shape)| {
            evidence.access == crate::facts::FieldAccess::Write
                && !evidence.is_static
                && evidence.owner.as_bytes() == class_internal
                && matches!(evidence.descriptor.as_bytes().first(), Some(b'L' | b'['))
                && shape.writes()
        }) {
            has_candidate = true;
            break;
        }
    }
    if !has_candidate {
        return Ok((Vec::new(), accessors));
    }

    let instructions = build_ssa_instruction_index(ssa, budget)?;
    let mut writes = Vec::<&Stmt>::new();
    let mut write_counts = std::collections::HashMap::<u32, usize>::new();
    let mut declarations = std::collections::HashMap::<
        String,
        Vec<(u32, &Type, Option<&String>, Option<&crate::ast::Expr>)>,
    >::new();
    let mut rebound = std::collections::HashSet::<String>::new();
    let mut pending: Vec<_> = program.stmts.iter().map(Node::Stmt).collect();
    while let Some(node) = pending.pop() {
        let at = match node {
            Node::Stmt(statement) => statement.origin.primary().bci(),
            Node::Expr(expression) => expression.origin.primary().bci(),
        };
        crate::stop::poll(budget, Some(at))?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
            1,
            Some(at),
        )?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            Some(at),
        )?;
        match node {
            Node::Expr(expression) => {
                if let ExprKind::LocalAssign { name, .. } = &expression.kind {
                    rebound.insert(name.clone());
                }
                push_expression_children(expression, &mut pending);
            }
            Node::Stmt(statement) => match &statement.kind {
                StmtKind::Declare {
                    ty,
                    source_type_name,
                    name,
                    value,
                } => {
                    declarations.entry(name.clone()).or_default().push((
                        at,
                        ty,
                        source_type_name.as_ref(),
                        value.as_ref(),
                    ));
                    pending.extend(value.iter().map(Node::Expr));
                }
                StmtKind::Assign { name, value } => {
                    rebound.insert(name.clone());
                    pending.push(Node::Expr(value));
                }
                StmtKind::FieldAssign {
                    receiver, value, ..
                } => {
                    writes.push(statement);
                    *write_counts.entry(at).or_default() += 1;
                    pending.push(Node::Expr(value));
                    pending.extend(receiver.iter().map(Node::Expr));
                }
                StmtKind::Expr(value) | StmtKind::Throw { value } => {
                    pending.push(Node::Expr(value));
                }
                StmtKind::Assert { cond, message } => {
                    pending.extend(message.iter().map(Node::Expr));
                    pending.push(Node::Expr(cond));
                }
                StmtKind::IndexAssign {
                    array,
                    index,
                    value,
                    ..
                } => {
                    pending.push(Node::Expr(value));
                    pending.push(Node::Expr(index));
                    pending.push(Node::Expr(array));
                }
                StmtKind::ConstructorCall { args, .. } => {
                    pending.extend(args.iter().map(Node::Expr));
                }
                StmtKind::Return { value } => pending.extend(value.iter().map(Node::Expr)),
                StmtKind::Break { .. } | StmtKind::Continue { .. } | StmtKind::Fallback { .. } => {}
                StmtKind::If {
                    cond,
                    then_body,
                    else_body,
                } => {
                    pending.extend(else_body.iter().map(Node::Stmt));
                    pending.extend(then_body.iter().map(Node::Stmt));
                    pending.push(Node::Expr(cond));
                }
                StmtKind::While { cond, body, .. } | StmtKind::DoWhile { cond, body, .. } => {
                    pending.extend(body.iter().map(Node::Stmt));
                    pending.push(Node::Expr(cond));
                }
                StmtKind::For {
                    init,
                    cond,
                    update,
                    body,
                    ..
                } => {
                    pending.extend(body.iter().map(Node::Stmt));
                    pending.push(Node::Stmt(update));
                    pending.push(Node::Expr(cond));
                    pending.push(Node::Stmt(init));
                }
                StmtKind::ForEach {
                    name,
                    iterable,
                    body,
                    ..
                } => {
                    rebound.insert(name.clone());
                    pending.extend(body.iter().map(Node::Stmt));
                    pending.push(Node::Expr(iterable));
                }
                StmtKind::Switch { value, arms } => {
                    for arm in arms.iter().rev() {
                        pending.extend(arm.body.iter().rev().map(Node::Stmt));
                    }
                    pending.push(Node::Expr(value));
                }
                StmtKind::Try {
                    resources,
                    catches,
                    body,
                    finally_body,
                } => {
                    for resource in resources {
                        rebound.insert(resource.name.clone());
                        pending.push(Node::Expr(&resource.value));
                    }
                    for catch in catches.iter().rev() {
                        rebound.insert(catch.name.clone());
                        pending.extend(catch.body.iter().rev().map(Node::Stmt));
                    }
                    pending.extend(body.iter().rev().map(Node::Stmt));
                    if let Some(body) = finally_body {
                        pending.extend(body.iter().rev().map(Node::Stmt));
                    }
                }
                StmtKind::Synchronized { lock, body } => {
                    pending.extend(body.iter().map(Node::Stmt));
                    pending.push(Node::Expr(lock));
                }
            },
        }
    }

    let mut receivers = Vec::new();
    for statement in writes {
        let bci = statement.origin.primary().bci();
        crate::stop::poll(budget, Some(bci))?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
            1,
            Some(bci),
        )?;
        if statement.origin.primary().method().is_some() || !presented.contains(&bci) {
            continue;
        }
        if write_counts.get(&bci) != Some(&1) {
            continue;
        }
        let StmtKind::FieldAssign {
            receiver: Some(receiver),
            name,
            op: crate::ast::AssignOp::Assign,
            ..
        } = &statement.kind
        else {
            continue;
        };
        let ExprKind::Local(local_name) = &receiver.kind else {
            continue;
        };
        if receiver.origin.primary().method().is_some() {
            continue;
        }
        let Some((evidence, shape)) = fields.claim(bci) else {
            continue;
        };
        if evidence.access != crate::facts::FieldAccess::Write
            || evidence.is_static
            || evidence.owner.as_bytes() != class_internal
            || evidence.name != *name
            || !matches!(evidence.descriptor.as_bytes().first(), Some(b'L' | b'['))
            || !shape.writes()
        {
            continue;
        }
        let Some(receiver_value) = shape.receiver else {
            continue;
        };
        let Some((slot, load_bci, local_value)) =
            receiver_load_slot(bci, receiver_value, ssa, &instructions, budget)?
        else {
            continue;
        };
        if receiver.origin.primary().bci() != load_bci {
            continue;
        }
        let local_definition = ssa.value(local_value).def();

        let is_this_source =
            if local_name == "this" && declaration.access_flags() & 0x0008 == 0 && slot == 0 {
                matches!(
                    local_definition,
                    Definition::Entry {
                        slot: Slot::Local(0),
                        ..
                    }
                ) || initialized_this_value(
                    init_record,
                    operations,
                    &method,
                    &class_internal,
                    slot,
                    local_value,
                    ssa,
                    &instructions,
                    budget,
                )?
            } else {
                false
            };
        let source = if is_this_source {
            Some(ClassSourceFieldReceiverSource::This)
        } else {
            crate::stop::poll(budget, Some(bci))?;
            crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
                u64::try_from(parameter_slots.len()).unwrap_or(u64::MAX),
                Some(bci),
            )?;
            let matching_parameter_slots: Vec<_> = parameter_slots
                .iter()
                .copied()
                .filter(|parameter_slot| {
                    names
                        .whole(*parameter_slot)
                        .is_some_and(|parameter| parameter.text() == local_name)
                })
                .collect();
            if matching_parameter_slots.len() == 1
                && matching_parameter_slots.first() == Some(&slot)
                && !declarations.contains_key(local_name)
                && !rebound.contains(local_name)
                && reuse.variable_at(slot, load_bci)
                    == Some(crate::names::LocalVariable::whole(slot))
                && matches!(local_definition, Definition::Entry { slot: Slot::Local(entry_slot), .. } if entry_slot == &slot)
            {
                Some(ClassSourceFieldReceiverSource::Parameter { slot })
            } else if matching_parameter_slots.is_empty()
                && !rebound.contains(local_name)
                && receiver.presented.as_ref().is_some_and(|ty| {
                    matches!(ty, Type::Reference(source_name) if source_name == &class_source)
                })
                && declarations.get(local_name).is_some_and(|decls| {
                    decls.len() == 1
                        && matches!(local_definition, Definition::Instruction { bci: store_bci, .. } if *store_bci == decls[0].0)
                        && *decls[0].1 == Type::Reference(class_source.clone())
                        && decls[0]
                            .2
                            .map(|name| name.as_str())
                            .is_none_or(|source_name| source_name == class_source)
                })
                && declarations.get(local_name).is_some_and(|decls| {
                    let Some((_, _, _, Some(initializer))) = decls.first() else {
                        return false;
                    };
                    let ExprKind::Local(parameter_name) = &initializer.kind else {
                        return false;
                    };
                    let Some((store_bci, _, _, _)) = decls.first() else {
                        return false;
                    };
                    let Definition::Instruction { bci: actual_store, .. } = local_definition else {
                        return false;
                    };
                    if actual_store != store_bci {
                        return false;
                    }
                    let Some(Some(store)) = instructions.get(actual_store) else {
                        return false;
                    };
                    let local_write_count = store
                        .writes()
                        .iter()
                        .filter(|(store_slot, value)| {
                            *store_slot == Slot::Local(slot) && *value == local_value
                        })
                        .count();
                    let stack_values: Vec<_> = store
                        .reads()
                        .iter()
                        .filter_map(|(store_slot, value)| {
                            matches!(store_slot, Slot::Stack(_)).then_some(*value)
                        })
                        .collect();
                    if local_write_count != 1 || stack_values.len() != 1 {
                        return false;
                    }
                    if initializer.origin.primary().method().is_some() {
                        return false;
                    }
                    let initializer_bci = initializer.origin.primary().bci();
                    let Definition::Instruction {
                        bci: initializer_load_bci,
                        ..
                    } = ssa.value(stack_values[0]).def()
                    else {
                        return false;
                    };
                    if *initializer_load_bci != initializer_bci
                        || ssa.value(stack_values[0]).uses().len() != 1
                        || ssa.value(stack_values[0]).uses()[0].bci() != Some(*actual_store)
                    {
                        return false;
                    }
                    let Some(Some(initializer_load)) = instructions.get(&initializer_bci) else {
                        return false;
                    };
                    if !matches!(initializer_load.opcode(), 0x19 | 0x2a..=0x2d)
                        || initializer_load
                            .writes()
                            .iter()
                            .filter(|(load_slot, value)| {
                                matches!(load_slot, Slot::Stack(_)) && *value == stack_values[0]
                            })
                            .count()
                            != 1
                    {
                        return false;
                    }
                    let mut initializer_local_reads = initializer_load
                        .reads()
                        .iter()
                        .filter_map(|(load_slot, value)| match load_slot {
                            Slot::Local(load_slot) => Some((*load_slot, *value)),
                            Slot::Stack(_) => None,
                        });
                    let Some((parameter_slot, parameter_value)) = initializer_local_reads.next()
                    else {
                        return false;
                    };
                    if initializer_local_reads.next().is_some()
                        || !parameter_slots.contains(&parameter_slot)
                        || reuse.variable_at(parameter_slot, initializer_bci)
                            != Some(crate::names::LocalVariable::whole(parameter_slot))
                    {
                        return false;
                    }
                    let Definition::Entry {
                        slot: Slot::Local(entry_parameter_slot),
                        ..
                    } = ssa.value(parameter_value).def()
                    else {
                        return false;
                    };
                    *entry_parameter_slot == parameter_slot
                        && names
                            .whole(parameter_slot)
                            .is_some_and(|parameter| parameter.text() == parameter_name)
                        && reuse.variable_at(slot, *actual_store)
                            == Some(crate::names::LocalVariable::whole(slot))
                        && reuse.variable_at(slot, load_bci)
                            == Some(crate::names::LocalVariable::whole(slot))
                        && !declarations.contains_key(parameter_name)
                        && !rebound.contains(parameter_name)
                })
            {
                Some(ClassSourceFieldReceiverSource::RawLocal {
                    name: local_name.clone(),
                })
            } else {
                None
            }
        };
        let Some(source) = source else {
            continue;
        };
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            Some(bci),
        )?;
        receivers.push(ClassSourceFieldReceiverSite {
            method: method.clone(),
            bci,
            owner: evidence.owner.as_bytes().to_vec(),
            name: evidence.name.as_bytes().to_vec(),
            descriptor: evidence.descriptor.as_bytes().to_vec(),
            source,
        });
    }

    Ok((receivers, accessors))
}

/// Opaque same-run AST sidecar for class-source adapters; never serialized as report evidence.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceMethodAst {
    pub(crate) projection: std::sync::Arc<ClassSourceMethodAstSource>,
}

/// The exact invocation identity used to join retained AST nodes with this run's physical call
/// census. The caller is the AST's own [`PhysicalMethodId`], so a BCI is never used without its
/// method and class-file definition.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceInvokeKey {
    pub call_bci: u32,
    pub opcode: u8,
    pub target: crate::facts::CallTarget,
}

/// One physical anchor from a retained AST expression.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceAstAnchor {
    pub bci: u32,
    pub method: Option<PhysicalMethodId>,
    pub provenance: crate::source_map::Provenance,
}

/// The small amount of AST information needed to join an expression with its physical value.
/// This is a site descriptor, not a second AST or a source-type proof.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceInvokeAstExpression {
    pub primary: ClassSourceAstAnchor,
    pub derived: Vec<ClassSourceAstAnchor>,
    pub direct_local_name: Option<String>,
    /// True only for the literal `null` AST node; this does not infer a reference type.
    pub null_literal: bool,
    pub shape: ClassSourceAstExpressionShape,
    /// The type the existing AST emitter presents for this expression. This is auxiliary matching
    /// evidence only; generic source types must come from a committed declaration and its source
    /// consumers must be proved separately.
    pub presented_type: Option<crate::ast::Type>,
    /// A narrowly recognized descriptor-reference wrapper added by invocation argument
    /// presentation for a target-typed call or `null` literal. Physical checkcasts never populate
    /// this field.
    pub presentation_wrapper: Option<ClassSourceAstPresentationWrapper>,
}

/// The source-only argument wrapper produced for an invocation's erased reference parameter.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceAstPresentationWrapper {
    pub ty: crate::ast::Type,
    pub child: Box<ClassSourceInvokeAstExpression>,
}

/// One requested invocation that has a unique matching node in a complete retained method AST.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceInvokeAstSite {
    pub caller: PhysicalMethodId,
    pub key: ClassSourceInvokeKey,
    pub ast_name: String,
    pub receiver: Option<ClassSourceInvokeAstExpression>,
    pub arguments: Vec<ClassSourceInvokeAstExpression>,
}

/// An AST field write target supplied by the same-run physical field census.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceInvokeFieldTarget {
    pub write_bci: u32,
    pub owner: String,
    pub name: String,
    pub descriptor: String,
}

/// A finite classification of one direct AST use of an invocation result.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassSourceInvokeResultUseKind {
    DirectReturn {
        consumer_bci: u32,
    },
    DirectCallArgument {
        consumer: Option<ClassSourceInvokeKey>,
        argument_index: usize,
    },
    FieldWrite {
        target: ClassSourceInvokeFieldTarget,
    },
    LocalStore {
        write_bci: u32,
        local_name: String,
    },
    Other {
        consumer_bci: u32,
    },
}

/// One result expression and the direct source position that consumes it.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceInvokeResultUse {
    pub producer: ClassSourceInvokeKey,
    pub kind: ClassSourceInvokeResultUseKind,
    pub expression: ClassSourceInvokeAstExpression,
}

/// One call-argument cast approved by the caller's completed source-type and overload proof.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceInvokeArgumentCast {
    pub call_bci: u32,
    pub opcode: u8,
    pub target: crate::facts::CallTarget,
    pub argument_index: usize,
    pub ty: crate::ast::Type,
}

/// Exact argument position whose proved `(Object)` source-presentation wrapper may be removed.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceInvokeArgumentPresentationCast {
    pub call_bci: u32,
    pub opcode: u8,
    pub target: crate::facts::CallTarget,
    pub argument_index: usize,
}

/// One declaration the retained AST actually emits for a local variable.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceAstLocalDeclaration {
    pub bci: u32,
    pub name: String,
    pub ty: crate::ast::Type,
    pub source_type_name: Option<String>,
    /// Statement anchors of enclosing lexical containers, outermost first.
    pub scope_anchors: Vec<ClassSourceAstAnchor>,
}

/// A retained formal's emitted name tied to its physical JVM local slot.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceMethodFormalName {
    pub slot: u16,
    pub name: String,
}

/// The intentionally small set of expression forms a caller can classify without another AST.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassSourceAstExpressionShape {
    Local,
    Null,
    New {
        ty: String,
        argument_count: usize,
        qualified: bool,
    },
    Call {
        name: String,
        argument_count: usize,
    },
    BooleanNotLocal {
        local_name: String,
        primary: ClassSourceAstAnchor,
        derived: Vec<ClassSourceAstAnchor>,
        presented_type: Option<crate::ast::Type>,
    },
    Cast,
    Literal,
    StringLiteral,
    Other,
}

/// One catch scope that can shadow an emitted formal name.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceAstCatchScope {
    pub try_bci: u32,
    pub exception_type: String,
    pub local_name: String,
}

/// One actual expression consumer in a complete supported method body.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceAstBodyConsumer {
    pub bci: u32,
    pub expression: ClassSourceInvokeAstExpression,
    pub catch_scopes: Vec<ClassSourceAstCatchScope>,
}

/// One actual field assignment statement's expressions, with its exact AST field spelling.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceAstFieldWriteConsumer {
    pub bci: u32,
    pub name: String,
    pub op: crate::ast::AssignOp,
    pub receiver: Option<ClassSourceInvokeAstExpression>,
    pub value: ClassSourceInvokeAstExpression,
    pub catch_scopes: Vec<ClassSourceAstCatchScope>,
}

/// One actual constructor invocation statement retained from the constructor body.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceAstConstructorCallConsumer {
    pub bci: u32,
    pub target: crate::ast::ConstructorTarget,
    pub arguments: Vec<ClassSourceInvokeAstExpression>,
    pub catch_scopes: Vec<ClassSourceAstCatchScope>,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceMethodBodyConsumers {
    /// The constructor prologue decided by this same method run, retained independently of the
    /// selected public rule-detail report. `None` means the run had no verified prologue.
    pub init_record: Option<crate::init::InitRecord>,
    pub returns: Vec<ClassSourceAstBodyConsumer>,
    pub conditions: Vec<ClassSourceAstBodyConsumer>,
    pub throws: Vec<ClassSourceAstBodyConsumer>,
    /// Ordinary invocation expression statements admitted only when their AST call exactly joins
    /// this method's unique same-run invocation target at the expression BCI.
    pub invocation_statements: Vec<ClassSourceAstBodyConsumer>,
    pub field_writes: Vec<ClassSourceAstFieldWriteConsumer>,
    pub constructor_calls: Vec<ClassSourceAstConstructorCallConsumer>,
    pub return_void_bcis: Vec<u32>,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceAnonymousConstructorInitializer {
    pub bci: u32,
    pub super_owner: String,
    pub owner: String,
    pub name: String,
    pub descriptor: String,
}

/// The statement position that produced one proved anonymous allocation site.
///
/// The site scan is shared by the anonymous interface projection and the anonymous superclass
/// projection (design `recover-anonymous-local-decl-site`, criterion 5), so the shape travels as a
/// first-class fact on the site tuple: the interface projection keeps requiring
/// [`AnonymousSiteShape::DirectReturn`] — byte-equivalent with the pre-relaxation gate — while the
/// superclass projection accepts both shapes.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnonymousSiteShape {
    /// The allocation is its method's returned expression (a leading local-declaration prologue
    /// may precede the return).
    DirectReturn,
    /// The allocation is the sole initializer of one local variable declaration, at any statement
    /// position of the method body.
    LocalDeclInitializer,
}

/// The exact anonymous allocation retained from one class-source recovery run, when present, with
/// the statement position that produced it.
#[doc(hidden)]
pub fn class_source_anonymous_return_site(
    ast: &ClassSourceMethodAst,
) -> Option<(Vec<u32>, String, Vec<u32>, AnonymousSiteShape)> {
    let (bcis, ty, args, shape) = class_source_anonymous_site(&ast.projection.program)?;
    Some((
        bcis,
        ty.to_owned(),
        args.iter().map(|arg| arg.origin.primary().bci()).collect(),
        shape,
    ))
}

/// The exact first argument BCI only when the direct-return allocation passes its own receiver as
/// the sole argument. Used to erase an anonymous class's proved synthetic enclosing-instance ctor
/// argument from the source-level interface creation.
#[doc(hidden)]
pub fn class_source_anonymous_outer_argument_bci(ast: &ClassSourceMethodAst) -> Option<u32> {
    let bci = class_source_anonymous_single_argument_bci(ast)?;
    let (_, _, args) = class_source_direct_return_new(&ast.projection.program)?;
    let [argument] = args else { return None };
    (matches!(argument.kind, crate::ast::ExprKind::Local(ref name) if name == "this")
        && argument.origin.primary().bci() == bci)
        .then_some(bci)
}

/// The exact BCI of a direct-return site's sole constructor argument, regardless of its source
/// expression. The facade may erase it only after a physical capture proof closes that value.
#[doc(hidden)]
pub fn class_source_anonymous_single_argument_bci(ast: &ClassSourceMethodAst) -> Option<u32> {
    let (_, _, args) = class_source_direct_return_new(&ast.projection.program)?;
    let [argument] = args else { return None };
    Some(argument.origin.primary().bci())
}

/// The same-run source spelling of one site argument: the local name and the presented type when
/// — and only when — the argument is exactly a local reference. The facade may hide the argument
/// and re-spell the proved capture reads as this local only after the role partition and the
/// allocation-site scan have closed the value; any other expression shape returns `None` and keeps
/// the physical presentation. Both site shapes are taken: the only caller is the superclass
/// projection's capture-site proof, which consumes direct-return and local-declaration sites.
#[doc(hidden)]
pub fn class_source_anonymous_argument_local(
    ast: &ClassSourceMethodAst,
    index: usize,
) -> Option<(String, Option<Type>)> {
    let (_, _, args, _) = class_source_anonymous_site(&ast.projection.program)?;
    let argument = args.get(index)?;
    let crate::ast::ExprKind::Local(name) = &argument.kind else {
        return None;
    };
    Some((name.clone(), argument.presented.clone()))
}

/// The same-run source spelling of one allocation's argument, read from the allocation node the
/// presentation wrote at `allocation_bci` — **wherever its statement sits**.
///
/// [`class_source_anonymous_argument_local`] reads the same spelling out of the site scan's two
/// statement shapes; the double-brace allocation point owns a third one (the allocation is the
/// value of an assignment — the static initializer's field write), so it reads the node by the
/// emission's own match instead. `None` unless the expression is exactly a local reference.
#[doc(hidden)]
pub fn class_source_allocation_argument_local(
    ast: &ClassSourceMethodAst,
    allocation_bci: u32,
    index: usize,
) -> Option<(String, Option<Type>)> {
    let expression = allocation_expression(&ast.projection.program.stmts, allocation_bci)?;
    let ExprKind::New { args, .. } = &expression.kind else {
        return None;
    };
    let argument = args.get(index)?;
    let ExprKind::Local(name) = &argument.kind else {
        return None;
    };
    Some((name.clone(), argument.presented.clone()))
}

/// Whether one allocation's class name is the pool form of an anonymous child of `root`: the
/// root's own internal name, `$`, and an anonymous ordinal.
///
/// The shape is the cheap reading of javac's own minting scheme; the class's `InnerClasses` row
/// (an anonymous row has neither an outer class nor an inner name) is the admission's own proof of
/// the same fact. The double-brace allocation point reads it in both layers: the recovery layer
/// retains this body's AST for it, and the class-source admission selects the site with it.
#[doc(hidden)]
pub fn class_source_anonymous_child_name(root: &str, child: &str) -> bool {
    child
        .strip_prefix(root)
        .and_then(|rest| rest.strip_prefix('$'))
        .is_some_and(|ordinal| {
            !ordinal.is_empty() && ordinal.bytes().all(|byte| byte.is_ascii_digit())
        })
}

/// Whether this body's own allocation scan holds a verified allocation of one of the **declaring
/// class's own** anonymous children (`X$N`) — the double-brace allocation point's cheap
/// pre-filter, read before the AST is retained.
fn allocates_an_anonymous_child(
    request: &RecoveryRequest<'_>,
    anonymous_allocations: Option<&Option<AnonymousAllocationScan>>,
) -> bool {
    let Some(declaring) = request
        .facts
        .method()
        .declaring_class()
        .map(|class| class.name().to_owned())
    else {
        return false;
    };
    anonymous_allocations
        .and_then(|scan| scan.as_ref())
        .is_some_and(|scan| {
            scan.allocations.iter().any(|site| {
                site.verified && class_source_anonymous_child_name(&declaring, &site.class)
            })
        })
}

/// The `new` expression one presentation wrote at `allocation_bci`: the node the anonymous
/// emission itself matches — the allocation's own type with the BCI on the expression's primary
/// anchor or on one of its derived ones.
fn allocation_expression<'a>(
    statements: &'a [crate::ast::Stmt],
    allocation_bci: u32,
) -> Option<&'a Expr> {
    let mut statements: Vec<&crate::ast::Stmt> = statements.iter().collect();
    let mut expressions: Vec<&Expr> = Vec::new();
    while let Some(statement) = statements.pop() {
        use StmtKind as K;
        match &statement.kind {
            K::Declare { value, .. } => expressions.extend(value.iter()),
            K::Assign { value, .. } | K::Expr(value) | K::Throw { value } => {
                expressions.push(value)
            }
            K::FieldAssign {
                receiver, value, ..
            } => {
                expressions.extend(receiver.iter());
                expressions.push(value);
            }
            K::IndexAssign {
                array,
                index,
                value,
                ..
            } => expressions.extend([array, index, value]),
            K::ConstructorCall { args, .. } => expressions.extend(args),
            K::Return { value } => expressions.extend(value.iter()),
            K::Assert { cond, message } => {
                expressions.push(cond);
                expressions.extend(message.iter());
            }
            K::If {
                cond,
                then_body,
                else_body,
            } => {
                expressions.push(cond);
                statements.extend(then_body);
                statements.extend(else_body);
            }
            K::While { cond, body, .. } | K::DoWhile { cond, body, .. } => {
                expressions.push(cond);
                statements.extend(body);
            }
            K::For {
                init,
                cond,
                update,
                body,
                ..
            } => {
                statements.push(init);
                statements.push(update);
                expressions.push(cond);
                statements.extend(body);
            }
            K::ForEach { iterable, body, .. } => {
                expressions.push(iterable);
                statements.extend(body);
            }
            K::Switch { value, arms } => {
                expressions.push(value);
                for arm in arms {
                    statements.extend(&arm.body);
                }
            }
            K::Try {
                resources,
                catches,
                body,
                finally_body,
            } => {
                for resource in resources {
                    expressions.push(&resource.value);
                }
                for catch in catches {
                    statements.extend(&catch.body);
                }
                statements.extend(body);
                statements.extend(finally_body.iter().flatten());
            }
            K::Synchronized { lock, body } => {
                expressions.push(lock);
                statements.extend(body);
            }
            K::Break { .. } | K::Continue { .. } | K::Fallback { .. } => {}
        }
    }
    while let Some(expression) = expressions.pop() {
        if matches!(&expression.kind, ExprKind::New { .. })
            && (expression.origin.primary().bci() == allocation_bci
                || expression
                    .origin
                    .derived()
                    .iter()
                    .any(|origin| origin.bci() == allocation_bci))
        {
            return Some(expression);
        }
        use ExprKind as E;
        match &expression.kind {
            E::LocalAssign { value, .. } => expressions.push(value),
            E::Call { receiver, args, .. } => {
                expressions.extend(receiver.iter().map(|receiver| &**receiver));
                expressions.extend(args);
            }
            E::New {
                qualifier, args, ..
            } => {
                expressions.extend(qualifier.iter().map(|qualifier| &**qualifier));
                expressions.extend(args);
            }
            E::Lambda { body, .. } => expressions.push(body),
            E::MethodReference { qualifier, .. } => expressions.push(qualifier),
            E::Field { receiver, .. } => expressions.push(receiver),
            E::Index { array, index } => expressions.extend([&**array, &**index]),
            E::PostfixUpdate { target, .. } => expressions.push(target),
            E::ArrayLength { array } => expressions.push(array),
            E::NewArray {
                lengths,
                initializers,
                ..
            } => {
                expressions.extend(lengths);
                expressions.extend(initializers.iter().flatten());
            }
            E::Binary { left, right, .. } => expressions.extend([&**left, &**right]),
            E::Conditional {
                test,
                when_true,
                when_false,
            } => expressions.extend([&**test, &**when_true, &**when_false]),
            E::Concat { parts } => expressions.extend(parts.iter().map(|part| &part.value)),
            E::Cast { value, .. } => expressions.push(value),
            E::Not { value } | E::Neg { value } => expressions.push(value),
            E::InstanceOf { value, .. } => expressions.push(value),
            E::Local(_)
            | E::Integer(_)
            | E::IntegerConstantName { .. }
            | E::Boolean(_)
            | E::Long(_)
            | E::Float(_)
            | E::Double(_)
            | E::Str(_)
            | E::Null
            | E::ClassLiteral { .. }
            | E::Path(_)
            | E::QualifiedThis { .. }
            | E::Super { .. } => {}
        }
    }
    None
}

/// The same-run spelling for the sole physical parameter at `slot`. This bounded helper is used
/// only after a descriptor/SSA proof has established that the anonymous constructor receives that
/// exact entry parameter.
#[doc(hidden)]
pub fn class_source_single_parameter_name(ast: &ClassSourceMethodAst, slot: u16) -> Option<String> {
    (slot == 0 && ast.projection.parameter_names.len() == 1)
        .then(|| ast.projection.parameter_names[0].clone())
        .flatten()
}

/// The first retained instruction BCI of the AST's own complete physical method.
#[doc(hidden)]
pub fn class_source_method_first_instruction_bci(ast: &ClassSourceMethodAst) -> Option<u32> {
    if ast.projection.complete_code {
        ast.projection.instruction_bcis.first().copied()
    } else {
        None
    }
}

/// Checks that the caller's complete physical same-owner invoke inventory matches the census
/// supplied from the same class-file run. This validates inventory completeness only; callers
/// still resolve each requested site against its AST with [`class_source_invoke_ast_sites`].
///
/// `owner` is the class-file internal name (for example `sample/Box`). A key's caller is implicit
/// in `ast`, while every key target must name `owner`. Unknown opcodes, duplicate physical BCIs,
/// incomplete Code, and any missing or additional same-owner call are ordinary refusals (`None`).
#[doc(hidden)]
pub fn class_source_same_class_invoke_inventory_matches(
    ast: &ClassSourceMethodAst,
    owner: &str,
    expected_sites: &[ClassSourceInvokeKey],
    budget: &mut Budget,
) -> Result<Option<()>, crate::stop::StopReason> {
    let source = &ast.projection;
    if !source.complete_code || source.instruction_bcis.len() != source.instruction_count {
        return Ok(None);
    }

    let scan_items = source
        .instruction_bcis
        .len()
        .saturating_add(source.call_targets.len())
        .saturating_add(expected_sites.len());
    let sort_cost = source
        .call_targets
        .len()
        .saturating_add(expected_sites.len())
        .saturating_mul(
            log2_upper_bound(
                source
                    .call_targets
                    .len()
                    .saturating_add(expected_sites.len()),
            )
            .saturating_add(1),
        )
        .saturating_add(
            source
                .instruction_bcis
                .len()
                .saturating_mul(log2_upper_bound(source.instruction_bcis.len()).saturating_add(1)),
        );
    charge_class_source_metadata(budget, scan_items.saturating_add(sort_cost), None)?;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        u64::try_from(scan_items).unwrap_or(u64::MAX),
        source.instruction_bcis.first().copied(),
    )?;

    let mut instruction_bcis = source.instruction_bcis.clone();
    instruction_bcis.sort_unstable();
    if instruction_bcis.windows(2).any(|pair| pair[0] == pair[1]) {
        return Ok(None);
    }

    let mut actual_sites = Vec::new();
    let mut seen_call_bcis = std::collections::HashSet::with_capacity(source.call_targets.len());
    for (bci, opcode, target) in &source.call_targets {
        crate::stop::poll(budget, Some(*bci))?;
        if !seen_call_bcis.insert(*bci)
            || instruction_bcis.binary_search(bci).is_err()
            || !class_source_invoke_opcode_matches_target(*opcode, target)
        {
            return Ok(None);
        }
        if target.owner() != owner {
            continue;
        }
        actual_sites.push((*bci, *opcode, target));
    }

    let mut expected = Vec::with_capacity(expected_sites.len());
    for key in expected_sites {
        crate::stop::poll(budget, Some(key.call_bci))?;
        if key.target.owner() != owner
            || instruction_bcis.binary_search(&key.call_bci).is_err()
            || !class_source_invoke_opcode_matches_target(key.opcode, &key.target)
        {
            return Ok(None);
        }
        expected.push((key.call_bci, key.opcode, &key.target));
    }

    actual_sites.sort_by_key(|site| site.0);
    expected.sort_by_key(|site| site.0);
    if actual_sites.len() != expected.len()
        || expected.windows(2).any(|pair| pair[0].0 == pair[1].0)
        || actual_sites
            .iter()
            .zip(&expected)
            .any(|(actual, expected)| actual != expected)
    {
        return Ok(None);
    }
    Ok(Some(()))
}

fn class_source_invoke_opcode_matches_target(
    opcode: u8,
    target: &crate::facts::CallTarget,
) -> bool {
    matches!(
        (opcode, target.kind()),
        (0xb6, crate::facts::InvokeKind::Virtual)
            | (0xb7, crate::facts::InvokeKind::Special)
            | (0xb8, crate::facts::InvokeKind::Static)
            | (0xb9, crate::facts::InvokeKind::Interface)
    )
}

/// Resolves only the requested physical call sites against this retained AST. The full call census
/// remains the caller's responsibility; a missing, duplicate, folded, or textually mismatched site
/// is an ordinary refusal (`None`).
#[doc(hidden)]
pub fn class_source_invoke_ast_sites(
    ast: &ClassSourceMethodAst,
    required_sites: &[ClassSourceInvokeKey],
    budget: &mut Budget,
) -> Result<Option<Vec<ClassSourceInvokeAstSite>>, crate::stop::StopReason> {
    use crate::ast::ExprKind;
    let source = &ast.projection;
    if !source.complete_code || source.instruction_bcis.len() != source.instruction_count {
        return Ok(None);
    }
    if required_sites.is_empty() {
        return Ok(Some(Vec::new()));
    }
    let site_index_cost = required_sites
        .len()
        .saturating_mul(log2_upper_bound(required_sites.len()).saturating_add(1));
    charge_class_source_metadata(budget, site_index_cost, None)?;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        u64::try_from(required_sites.len()).unwrap_or(u64::MAX),
        None,
    )?;
    let mut required_by_bci = std::collections::BTreeMap::<u32, &ClassSourceInvokeKey>::new();
    for key in required_sites {
        if required_by_bci.insert(key.call_bci, key).is_some() {
            return Ok(None);
        }
    }
    let mut result = Vec::with_capacity(required_sites.len());
    for key in required_by_bci.values().copied() {
        crate::stop::poll(budget, Some(key.call_bci))?;
        charge_class_source_ast(source, budget)?;
        charge_class_source_metadata(
            budget,
            source
                .instruction_bcis
                .len()
                .saturating_add(source.call_targets.len()),
            Some(key.call_bci),
        )?;
        if !source.instruction_bcis.contains(&key.call_bci) {
            return Ok(None);
        }
        let target_matches: Vec<_> = source
            .call_targets
            .iter()
            .filter(|(bci, _, _)| *bci == key.call_bci)
            .collect();
        if target_matches.len() != 1
            || target_matches[0].1 != key.opcode
            || target_matches[0].2 != key.target
        {
            return Ok(None);
        }
        let mut matches = Vec::new();
        for_each_statement_expression(&source.program.stmts, &mut |expression| {
            let ExprKind::Call {
                name,
                receiver: _,
                args,
            } = &expression.kind
            else {
                return;
            };
            if expression.origin.primary().bci() != key.call_bci
                || expression
                    .origin
                    .primary()
                    .method()
                    .is_some_and(|method| method != &source.member)
                || name != key.target.name()
                || descriptor_parameter_count(key.target.descriptor()) != Some(args.len())
            {
                return;
            }
            matches.push(expression);
        });
        if matches.len() != 1 {
            return Ok(None);
        }
        let expression = matches.pop().expect("one call-site match");
        let ExprKind::Call {
            name,
            receiver,
            args,
        } = &expression.kind
        else {
            unreachable!("the matched node is a call")
        };
        let anchor_count = receiver
            .iter()
            .map(|receiver| class_source_invoke_ast_expression_anchor_count(receiver))
            .chain(
                args.iter()
                    .map(class_source_invoke_ast_expression_anchor_count),
            )
            .fold(0usize, usize::saturating_add);
        let call_expression = expression;
        let wrapper_child_anchor_count = args
            .iter()
            .enumerate()
            .filter_map(|(argument_index, argument)| {
                class_source_exact_reference_presentation_wrapper(
                    source,
                    call_expression,
                    key,
                    argument_index,
                    argument,
                )
                .map(|(_, child)| class_source_invoke_ast_expression_anchor_count(child))
            })
            .fold(0usize, usize::saturating_add);
        charge_class_source_metadata(
            budget,
            anchor_count
                .saturating_add(wrapper_child_anchor_count)
                .saturating_add(key.target.descriptor().len().saturating_mul(args.len())),
            Some(key.call_bci),
        )?;
        let arguments = args
            .iter()
            .enumerate()
            .map(|(argument_index, argument)| {
                class_source_invoke_ast_argument_expression(
                    source,
                    &source.member,
                    call_expression,
                    key,
                    argument_index,
                    argument,
                )
            })
            .collect();
        result.push(ClassSourceInvokeAstSite {
            caller: source.member.clone(),
            key: key.clone(),
            ast_name: name.clone(),
            receiver: receiver
                .as_deref()
                .map(|receiver| class_source_invoke_ast_expression(receiver, &source.member)),
            arguments,
        });
    }
    Ok(Some(result))
}

/// Lists every direct AST consumer of a selected call result. Callers join the returned positions
/// with their same-run SSA use census; this function never infers a generic source type from that
/// census or from `Expr::presented`.
#[doc(hidden)]
pub fn class_source_invoke_result_uses(
    ast: &ClassSourceMethodAst,
    producer: &ClassSourceInvokeKey,
    field_targets: &[ClassSourceInvokeFieldTarget],
    budget: &mut Budget,
) -> Result<Option<Vec<ClassSourceInvokeResultUse>>, crate::stop::StopReason> {
    use crate::ast::{Expr, ExprKind, Stmt};
    use std::collections::HashMap;

    let source = &ast.projection;
    if !source.complete_code || source.instruction_bcis.len() != source.instruction_count {
        return Ok(None);
    }
    let Some(producer_site) =
        class_source_invoke_ast_sites(ast, std::slice::from_ref(producer), budget)?
    else {
        return Ok(None);
    };
    let _ = producer_site;
    charge_class_source_ast(source, budget)?;
    let mut expression_anchor_count = 0usize;
    let mut descriptor_parse_cost = 0usize;
    for_each_statement_expression(&source.program.stmts, &mut |expression| {
        expression_anchor_count = expression_anchor_count
            .saturating_add(class_source_invoke_ast_expression_anchor_count(expression));
        let ExprKind::Call { name, args, .. } = &expression.kind else {
            return;
        };
        let bci = expression.origin.primary().bci();
        let target_index = source
            .call_targets
            .partition_point(|(call_bci, _, _)| *call_bci < bci);
        if let Some((call_bci, _, target)) = source.call_targets.get(target_index)
            && *call_bci == bci
            && target.name() == name
        {
            descriptor_parse_cost = descriptor_parse_cost
                .saturating_add(target.descriptor().len().saturating_mul(args.len()));
        }
    });
    crate::stop::poll(budget, Some(producer.call_bci))?;
    charge_class_source_metadata(
        budget,
        source
            .call_targets
            .len()
            .saturating_add(field_targets.len())
            .saturating_add(expression_anchor_count)
            .saturating_add(descriptor_parse_cost)
            .saturating_add(
                usize::try_from(program_node_count(&source.program))
                    .unwrap_or(usize::MAX)
                    .saturating_mul(log2_upper_bound(source.call_targets.len()).saturating_add(1)),
            ),
        Some(producer.call_bci),
    )?;
    let mut field_targets_by_site = HashMap::with_capacity(field_targets.len());
    for target in field_targets {
        crate::stop::poll(budget, Some(target.write_bci))?;
        if field_targets_by_site
            .insert((target.write_bci, target.name.clone()), target.clone())
            .is_some()
        {
            return Ok(None);
        }
    }
    let mut result = Vec::new();

    #[derive(Clone)]
    enum Consumer {
        Return(u32),
        CallArgument(Option<ClassSourceInvokeKey>, usize),
        FieldWrite(Option<ClassSourceInvokeFieldTarget>),
        LocalStore(u32, String),
        Other(u32),
    }

    fn matching_call_target(
        source: &ClassSourceMethodAstSource,
        expression: &Expr,
    ) -> Option<ClassSourceInvokeKey> {
        let ExprKind::Call { name, args, .. } = &expression.kind else {
            return None;
        };
        let bci = expression.origin.primary().bci();
        if expression
            .origin
            .primary()
            .method()
            .is_some_and(|method| method != &source.member)
        {
            return None;
        }
        let index = source
            .call_targets
            .partition_point(|(call_bci, _, _)| *call_bci < bci);
        let (call_bci, opcode, target) = source.call_targets.get(index)?;
        if *call_bci != bci
            || source
                .call_targets
                .get(index + 1)
                .is_some_and(|(next_bci, _, _)| *next_bci == bci)
            || target.name() != name
            || descriptor_parameter_count(target.descriptor()) != Some(args.len())
        {
            return None;
        }
        Some(ClassSourceInvokeKey {
            call_bci: bci,
            opcode: *opcode,
            target: target.clone(),
        })
    }

    fn visit_expression(
        source: &ClassSourceMethodAstSource,
        expression: &Expr,
        consumer: Consumer,
        producer: &ClassSourceInvokeKey,
        result: &mut Vec<ClassSourceInvokeResultUse>,
    ) {
        use crate::ast::ExprKind;
        if matches!(&expression.kind, ExprKind::Call { .. })
            && expression.origin.primary().bci() == producer.call_bci
            && matching_call_target(source, expression).as_ref() == Some(producer)
        {
            let kind = match &consumer {
                Consumer::Return(consumer_bci) => ClassSourceInvokeResultUseKind::DirectReturn {
                    consumer_bci: *consumer_bci,
                },
                Consumer::CallArgument(consumer, argument_index) => {
                    ClassSourceInvokeResultUseKind::DirectCallArgument {
                        consumer: consumer.clone(),
                        argument_index: *argument_index,
                    }
                }
                Consumer::FieldWrite(Some(target)) => ClassSourceInvokeResultUseKind::FieldWrite {
                    target: target.clone(),
                },
                Consumer::FieldWrite(None) => ClassSourceInvokeResultUseKind::Other {
                    consumer_bci: expression.origin.primary().bci(),
                },
                Consumer::LocalStore(write_bci, local_name) => {
                    ClassSourceInvokeResultUseKind::LocalStore {
                        write_bci: *write_bci,
                        local_name: local_name.clone(),
                    }
                }
                Consumer::Other(consumer_bci) => ClassSourceInvokeResultUseKind::Other {
                    consumer_bci: *consumer_bci,
                },
            };
            result.push(ClassSourceInvokeResultUse {
                producer: producer.clone(),
                kind,
                expression: class_source_invoke_ast_expression(expression, &source.member),
            });
        }
        match &expression.kind {
            ExprKind::Call { receiver, args, .. } => {
                if let Some(receiver) = receiver {
                    visit_expression(
                        source,
                        receiver,
                        Consumer::Other(expression.origin.primary().bci()),
                        producer,
                        result,
                    );
                }
                let key = matching_call_target(source, expression);
                for (index, argument) in args.iter().enumerate() {
                    visit_expression(
                        source,
                        argument,
                        Consumer::CallArgument(key.clone(), index),
                        producer,
                        result,
                    );
                }
            }
            ExprKind::New {
                qualifier, args, ..
            } => {
                if let Some(qualifier) = qualifier {
                    visit_expression(
                        source,
                        qualifier,
                        Consumer::Other(expression.origin.primary().bci()),
                        producer,
                        result,
                    );
                }
                for argument in args {
                    visit_expression(
                        source,
                        argument,
                        Consumer::Other(expression.origin.primary().bci()),
                        producer,
                        result,
                    );
                }
            }
            ExprKind::LocalAssign { value, .. }
            | ExprKind::InstanceOf { value, .. }
            | ExprKind::Lambda { body: value, .. }
            | ExprKind::MethodReference {
                qualifier: value, ..
            }
            | ExprKind::Field {
                receiver: value, ..
            }
            | ExprKind::ArrayLength { array: value }
            | ExprKind::PostfixUpdate { target: value, .. }
            | ExprKind::Not { value }
            | ExprKind::Neg { value } => visit_expression(
                source,
                value,
                Consumer::Other(expression.origin.primary().bci()),
                producer,
                result,
            ),
            ExprKind::Cast { value, .. } => {
                let transparent_consumer = match &consumer {
                    Consumer::CallArgument(Some(key), argument_index) => {
                        class_source_expression_reference_presentation_wrapper(
                            source,
                            key,
                            *argument_index,
                            expression,
                        )
                        .is_some()
                    }
                    _ => false,
                };
                visit_expression(
                    source,
                    value,
                    if transparent_consumer {
                        consumer.clone()
                    } else {
                        Consumer::Other(expression.origin.primary().bci())
                    },
                    producer,
                    result,
                );
            }
            ExprKind::Index { array, index }
            | ExprKind::Binary {
                left: array,
                right: index,
                ..
            } => {
                for value in [array.as_ref(), index.as_ref()] {
                    visit_expression(
                        source,
                        value,
                        Consumer::Other(expression.origin.primary().bci()),
                        producer,
                        result,
                    );
                }
            }
            ExprKind::NewArray {
                lengths,
                initializers,
                ..
            } => {
                for value in lengths.iter().chain(initializers.iter().flatten()) {
                    visit_expression(
                        source,
                        value,
                        Consumer::Other(expression.origin.primary().bci()),
                        producer,
                        result,
                    );
                }
            }
            ExprKind::Conditional {
                test,
                when_true,
                when_false,
            } => {
                for value in [test.as_ref(), when_true.as_ref(), when_false.as_ref()] {
                    visit_expression(
                        source,
                        value,
                        Consumer::Other(expression.origin.primary().bci()),
                        producer,
                        result,
                    );
                }
            }
            ExprKind::Concat { parts } => {
                for part in parts {
                    visit_expression(
                        source,
                        &part.value,
                        Consumer::Other(expression.origin.primary().bci()),
                        producer,
                        result,
                    );
                }
            }
            ExprKind::Local(_)
            | ExprKind::Integer(_)
            | ExprKind::IntegerConstantName { .. }
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

    fn visit_statements(
        source: &ClassSourceMethodAstSource,
        statements: &[Stmt],
        producer: &ClassSourceInvokeKey,
        field_targets: &HashMap<(u32, String), ClassSourceInvokeFieldTarget>,
        result: &mut Vec<ClassSourceInvokeResultUse>,
    ) {
        use crate::ast::StmtKind;
        for statement in statements {
            let bci = statement.origin.primary().bci();
            match &statement.kind {
                StmtKind::Return { value: Some(value) } => {
                    visit_expression(source, value, Consumer::Return(bci), producer, result)
                }
                StmtKind::FieldAssign { name, value, .. } => {
                    let target = field_targets.get(&(bci, name.clone())).cloned();
                    visit_expression(
                        source,
                        value,
                        Consumer::FieldWrite(target),
                        producer,
                        result,
                    );
                }
                StmtKind::Declare {
                    name,
                    value: Some(value),
                    ..
                }
                | StmtKind::Assign { name, value } => visit_expression(
                    source,
                    value,
                    Consumer::LocalStore(bci, name.clone()),
                    producer,
                    result,
                ),
                StmtKind::Expr(value) | StmtKind::Throw { value } => {
                    visit_expression(source, value, Consumer::Other(bci), producer, result)
                }
                StmtKind::Assert { cond, message } => {
                    visit_expression(source, cond, Consumer::Other(bci), producer, result);
                    if let Some(message) = message {
                        visit_expression(source, message, Consumer::Other(bci), producer, result);
                    }
                }
                StmtKind::IndexAssign {
                    array,
                    index,
                    value,
                    ..
                } => {
                    for value in [array, index, value] {
                        visit_expression(source, value, Consumer::Other(bci), producer, result);
                    }
                }
                StmtKind::ConstructorCall { args, .. } => {
                    for argument in args {
                        visit_expression(source, argument, Consumer::Other(bci), producer, result);
                    }
                }
                StmtKind::Return { value: None }
                | StmtKind::Break { .. }
                | StmtKind::Continue { .. }
                | StmtKind::Fallback { .. } => {}
                StmtKind::If {
                    cond,
                    then_body,
                    else_body,
                } => {
                    visit_expression(source, cond, Consumer::Other(bci), producer, result);
                    visit_statements(source, then_body, producer, field_targets, result);
                    visit_statements(source, else_body, producer, field_targets, result);
                }
                StmtKind::While { cond, body, .. } | StmtKind::DoWhile { cond, body, .. } => {
                    visit_expression(source, cond, Consumer::Other(bci), producer, result);
                    visit_statements(source, body, producer, field_targets, result);
                }
                StmtKind::For {
                    init,
                    cond,
                    update,
                    body,
                    ..
                } => {
                    visit_statements(
                        source,
                        std::slice::from_ref(init),
                        producer,
                        field_targets,
                        result,
                    );
                    visit_expression(source, cond, Consumer::Other(bci), producer, result);
                    visit_statements(
                        source,
                        std::slice::from_ref(update),
                        producer,
                        field_targets,
                        result,
                    );
                    visit_statements(source, body, producer, field_targets, result);
                }
                StmtKind::ForEach { iterable, body, .. } => {
                    visit_expression(source, iterable, Consumer::Other(bci), producer, result);
                    visit_statements(source, body, producer, field_targets, result);
                }
                StmtKind::Switch { value, arms } => {
                    visit_expression(source, value, Consumer::Other(bci), producer, result);
                    for arm in arms {
                        visit_statements(source, &arm.body, producer, field_targets, result);
                    }
                }
                StmtKind::Try {
                    resources,
                    catches,
                    body,
                    finally_body,
                } => {
                    for resource in resources {
                        visit_expression(
                            source,
                            &resource.value,
                            Consumer::Other(bci),
                            producer,
                            result,
                        );
                    }
                    visit_statements(source, body, producer, field_targets, result);
                    for catch in catches {
                        visit_statements(source, &catch.body, producer, field_targets, result);
                    }
                    if let Some(finally_body) = finally_body {
                        visit_statements(source, finally_body, producer, field_targets, result);
                    }
                }
                StmtKind::Synchronized { lock, body } => {
                    visit_expression(source, lock, Consumer::Other(bci), producer, result);
                    visit_statements(source, body, producer, field_targets, result);
                }
                StmtKind::Declare { value: None, .. } => {}
            }
        }
    }

    visit_statements(
        source,
        &source.program.stmts,
        producer,
        &field_targets_by_site,
        &mut result,
    );
    Ok(Some(result))
}

/// Captures only return, condition, throw, direct invocation-statement, and field-write consumer
/// positions the same-class generic proof can use. `None` explicitly refuses an unmodeled
/// statement or try shape; this is not a second control-flow representation and it never replaces
/// the original AST.
#[doc(hidden)]
pub fn class_source_method_body_consumers(
    ast: &ClassSourceMethodAst,
    budget: &mut Budget,
) -> Result<Option<ClassSourceMethodBodyConsumers>, crate::stop::StopReason> {
    use crate::ast::{Stmt, StmtKind};

    if !ast.projection.complete_code
        || ast.projection.instruction_bcis.len() != ast.projection.instruction_count
        || ast.projection.program.ragged
        || !ast.projection.program.field_increments.is_empty()
        || !ast.projection.program.lambdas.is_empty()
        || !ast.projection.program.accessors.is_empty()
        || !ast.projection.program.array_constructor_sites.is_empty()
        || !ast.projection.program.lambda_refusals.is_empty()
        || !ast.projection.program.accessor_refusals.is_empty()
    {
        return Ok(None);
    }
    charge_class_source_ast(&ast.projection, budget)?;

    fn append_consumer(
        statement_bci: u32,
        expression: &crate::ast::Expr,
        member: &PhysicalMethodId,
        catch_scopes: &[ClassSourceAstCatchScope],
        output: &mut Vec<ClassSourceAstBodyConsumer>,
        budget: &mut Budget,
    ) -> Result<(), crate::stop::StopReason> {
        charge_class_source_metadata(
            budget,
            class_source_invoke_ast_expression_anchor_count(expression)
                .saturating_add(catch_scopes.len())
                .saturating_add(1),
            Some(statement_bci),
        )?;
        output.push(ClassSourceAstBodyConsumer {
            bci: statement_bci,
            expression: class_source_invoke_ast_expression(expression, member),
            catch_scopes: catch_scopes.to_vec(),
        });
        Ok(())
    }

    fn is_exact_invocation_statement(
        source: &ClassSourceMethodAstSource,
        statement_bci: u32,
        expression: &crate::ast::Expr,
        budget: &mut Budget,
    ) -> Result<bool, crate::stop::StopReason> {
        let crate::ast::ExprKind::Call { name, args, .. } = &expression.kind else {
            return Ok(false);
        };
        let call_bci = expression.origin.primary().bci();
        if statement_bci != call_bci
            || expression
                .origin
                .primary()
                .method()
                .is_some_and(|method| method != &source.member)
        {
            return Ok(false);
        }
        crate::stop::poll(budget, Some(call_bci))?;
        charge_class_source_metadata(
            budget,
            source
                .instruction_bcis
                .len()
                .saturating_add(source.call_targets.len()),
            Some(call_bci),
        )?;
        if source
            .instruction_bcis
            .iter()
            .filter(|bci| **bci == call_bci)
            .count()
            != 1
        {
            return Ok(false);
        }
        let mut target = None;
        for (bci, opcode, candidate) in &source.call_targets {
            if *bci == call_bci {
                if target.is_some() {
                    return Ok(false);
                }
                target = Some((*opcode, candidate));
            }
        }
        let Some((opcode, target)) = target else {
            return Ok(false);
        };
        let opcode_matches_kind = matches!(
            (opcode, target.kind()),
            (0xb6, crate::facts::InvokeKind::Virtual)
                | (0xb7, crate::facts::InvokeKind::Special)
                | (0xb8, crate::facts::InvokeKind::Static)
                | (0xb9, crate::facts::InvokeKind::Interface)
        );
        Ok(opcode_matches_kind
            && target.name() == name.as_str()
            && descriptor_parameter_count(target.descriptor()) == Some(args.len()))
    }

    fn visit(
        statements: &[Stmt],
        source: &ClassSourceMethodAstSource,
        member: &PhysicalMethodId,
        catch_scopes: &mut Vec<ClassSourceAstCatchScope>,
        result: &mut ClassSourceMethodBodyConsumers,
        budget: &mut Budget,
    ) -> Result<bool, crate::stop::StopReason> {
        for statement in statements {
            let bci = statement.origin.primary().bci();
            charge_class_source_metadata(budget, 1, Some(bci))?;
            match &statement.kind {
                StmtKind::Return { value: Some(value) } => {
                    append_consumer(
                        bci,
                        value,
                        member,
                        catch_scopes,
                        &mut result.returns,
                        budget,
                    )?;
                }
                StmtKind::Return { value: None } => {
                    charge_class_source_metadata(budget, 1, Some(bci))?;
                    result.return_void_bcis.push(bci);
                }
                StmtKind::Throw { value } => {
                    append_consumer(bci, value, member, catch_scopes, &mut result.throws, budget)?;
                }
                StmtKind::Expr(expression)
                    if is_exact_invocation_statement(source, bci, expression, budget)? =>
                {
                    append_consumer(
                        bci,
                        expression,
                        member,
                        catch_scopes,
                        &mut result.invocation_statements,
                        budget,
                    )?;
                }
                StmtKind::ConstructorCall { target, args } => {
                    let arguments = args
                        .iter()
                        .map(class_source_invoke_ast_expression_anchor_count)
                        .fold(0usize, usize::saturating_add);
                    charge_class_source_metadata(
                        budget,
                        arguments
                            .saturating_add(catch_scopes.len())
                            .saturating_add(1),
                        Some(bci),
                    )?;
                    result
                        .constructor_calls
                        .push(ClassSourceAstConstructorCallConsumer {
                            bci,
                            target: *target,
                            arguments: args
                                .iter()
                                .map(|argument| {
                                    class_source_invoke_ast_expression(argument, member)
                                })
                                .collect(),
                            catch_scopes: catch_scopes.to_vec(),
                        });
                }
                StmtKind::FieldAssign {
                    receiver,
                    name,
                    op,
                    value,
                } => {
                    let expression_anchors = receiver
                        .iter()
                        .map(class_source_invoke_ast_expression_anchor_count)
                        .fold(
                            class_source_invoke_ast_expression_anchor_count(value),
                            usize::saturating_add,
                        );
                    charge_class_source_metadata(
                        budget,
                        expression_anchors
                            .saturating_add(catch_scopes.len())
                            .saturating_add(2),
                        Some(bci),
                    )?;
                    result.field_writes.push(ClassSourceAstFieldWriteConsumer {
                        bci,
                        name: name.clone(),
                        op: *op,
                        receiver: receiver
                            .as_ref()
                            .map(|receiver| class_source_invoke_ast_expression(receiver, member)),
                        value: class_source_invoke_ast_expression(value, member),
                        catch_scopes: catch_scopes.to_vec(),
                    });
                }
                StmtKind::If {
                    cond,
                    then_body,
                    else_body,
                } => {
                    append_consumer(
                        bci,
                        cond,
                        member,
                        catch_scopes,
                        &mut result.conditions,
                        budget,
                    )?;
                    if !visit(then_body, source, member, catch_scopes, result, budget)?
                        || !visit(else_body, source, member, catch_scopes, result, budget)?
                    {
                        return Ok(false);
                    }
                }
                StmtKind::Try {
                    resources,
                    catches,
                    body,
                    finally_body,
                } => {
                    if !resources.is_empty() || finally_body.is_some() {
                        return Ok(false);
                    }
                    if !visit(body, source, member, catch_scopes, result, budget)? {
                        return Ok(false);
                    }
                    for catch in catches {
                        charge_class_source_metadata(budget, 3, Some(bci))?;
                        catch_scopes.push(ClassSourceAstCatchScope {
                            try_bci: bci,
                            exception_type: catch.ty.clone(),
                            local_name: catch.name.clone(),
                        });
                        let supported =
                            visit(&catch.body, source, member, catch_scopes, result, budget)?;
                        catch_scopes.pop();
                        if !supported {
                            return Ok(false);
                        }
                    }
                }
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    let mut result = ClassSourceMethodBodyConsumers {
        init_record: {
            if let Some(record) = &ast.projection.generic_call_init {
                charge_class_source_metadata(
                    budget,
                    1usize
                        .saturating_add(record.class.as_ref().map_or(0, String::len))
                        .saturating_add(record.declared.as_ref().map_or(0, String::len)),
                    record.bci,
                )?;
            }
            ast.projection.generic_call_init.clone()
        },
        returns: Vec::new(),
        conditions: Vec::new(),
        throws: Vec::new(),
        invocation_statements: Vec::new(),
        field_writes: Vec::new(),
        constructor_calls: Vec::new(),
        return_void_bcis: Vec::new(),
    };
    if !visit(
        &ast.projection.program.stmts,
        &ast.projection,
        &ast.projection.member,
        &mut Vec::new(),
        &mut result,
        budget,
    )? {
        return Ok(None);
    }
    Ok(Some(result))
}

/// Applies only casts whose exact invocation and argument position the caller has proved. The
/// outer cast presents its source target type; the original child keeps its own `presented` type
/// and origins. This is a source-level upcast for overload selection, not a physical checkcast.
#[doc(hidden)]
pub fn project_class_source_invoke_argument_casts(
    ast: &ClassSourceMethodAst,
    edits: &[ClassSourceInvokeArgumentCast],
    budget: &mut Budget,
) -> Result<Option<ClassSourceMethodAst>, crate::stop::StopReason> {
    project_class_source_invoke_argument_edits(ast, &[], edits, budget)
}

/// Removes only explicitly selected descriptor-erasure reference presentation wrappers, then
/// applies proved source upcasts in one AST clone and traversal.
#[doc(hidden)]
pub fn project_class_source_invoke_argument_edits(
    ast: &ClassSourceMethodAst,
    removals: &[ClassSourceInvokeArgumentPresentationCast],
    edits: &[ClassSourceInvokeArgumentCast],
    budget: &mut Budget,
) -> Result<Option<ClassSourceMethodAst>, crate::stop::StopReason> {
    use crate::ast::{Expr, ExprKind, Type};
    use std::sync::Arc;

    if edits.is_empty() && removals.is_empty() {
        return Ok(Some(ast.clone()));
    }
    let total_edits = edits.len().saturating_add(removals.len());
    let grouping_cost = total_edits.saturating_mul(log2_upper_bound(total_edits).saturating_add(1));
    charge_class_source_metadata(
        budget,
        grouping_cost,
        edits
            .first()
            .map(|edit| edit.call_bci)
            .or_else(|| removals.first().map(|edit| edit.call_bci)),
    )?;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        u64::try_from(total_edits).unwrap_or(u64::MAX),
        edits
            .first()
            .map(|edit| edit.call_bci)
            .or_else(|| removals.first().map(|edit| edit.call_bci)),
    )?;
    if edits
        .iter()
        .any(|edit| !matches!(&edit.ty, Type::Reference(_)))
    {
        return Ok(None);
    }

    let mut sites_by_bci = std::collections::BTreeMap::new();
    let mut edited_arguments = std::collections::HashSet::with_capacity(total_edits);
    let mut removed_arguments = std::collections::HashSet::with_capacity(removals.len());
    for removal in removals {
        let key = ClassSourceInvokeKey {
            call_bci: removal.call_bci,
            opcode: removal.opcode,
            target: removal.target.clone(),
        };
        if let Some(existing) = sites_by_bci.get(&removal.call_bci) {
            if existing != &key {
                return Ok(None);
            }
        } else {
            sites_by_bci.insert(removal.call_bci, key);
        }
        if !removed_arguments.insert((removal.call_bci, removal.argument_index)) {
            return Ok(None);
        }
    }
    for edit in edits {
        let key = ClassSourceInvokeKey {
            call_bci: edit.call_bci,
            opcode: edit.opcode,
            target: edit.target.clone(),
        };
        if let Some(existing) = sites_by_bci.get(&edit.call_bci) {
            if existing != &key {
                return Ok(None);
            }
        } else {
            sites_by_bci.insert(edit.call_bci, key);
        }
        if !edited_arguments.insert((edit.call_bci, edit.argument_index)) {
            return Ok(None);
        }
    }
    let sites: Vec<_> = sites_by_bci.values().cloned().collect();
    if class_source_invoke_ast_sites(ast, &sites, budget)?.is_none() {
        return Ok(None);
    }
    charge_class_source_ast(&ast.projection, budget)?;
    let source_metadata_items = ast
        .projection
        .nested_class_members
        .len()
        .saturating_add(ast.projection.parameter_names.len())
        .saturating_add(ast.projection.parameter_slots.len())
        .saturating_add(ast.projection.instruction_bcis.len())
        .saturating_add(ast.projection.call_targets.len())
        .saturating_add(
            ast.projection
                .generic_call_init
                .as_ref()
                .map_or(0, |record| {
                    1usize
                        .saturating_add(record.class.as_ref().map_or(0, String::len))
                        .saturating_add(record.declared.as_ref().map_or(0, String::len))
                }),
        )
        .saturating_add(1);
    charge_class_source_metadata(
        budget,
        source_metadata_items,
        edits
            .first()
            .map(|edit| edit.call_bci)
            .or_else(|| removals.first().map(|edit| edit.call_bci)),
    )?;
    let mut edits_by_bci = std::collections::HashMap::<u32, Vec<usize>>::new();
    for (index, edit) in edits.iter().enumerate() {
        edits_by_bci.entry(edit.call_bci).or_default().push(index);
    }
    let mut removals_by_bci = std::collections::HashMap::<u32, Vec<usize>>::new();
    for (index, removal) in removals.iter().enumerate() {
        removals_by_bci
            .entry(removal.call_bci)
            .or_default()
            .push(index);
    }
    let mut program = ast.projection.program.clone();
    let mut matched = vec![0usize; edits.len()];
    let mut removed = vec![0usize; removals.len()];
    for_each_statement_expression_mut(&mut program.stmts, &mut |expression| {
        let ExprKind::Call { name, args, .. } = &mut expression.kind else {
            return;
        };
        let bci = expression.origin.primary().bci();
        if expression
            .origin
            .primary()
            .method()
            .is_some_and(|method| method != &ast.projection.member)
        {
            return;
        }
        let Some(site_key) = sites_by_bci.get(&bci) else {
            return;
        };
        if name != site_key.target.name()
            || descriptor_parameter_count(site_key.target.descriptor()) != Some(args.len())
        {
            return;
        }
        if let Some(indices) = removals_by_bci.get(&bci) {
            for index in indices {
                let removal = &removals[*index];
                if site_key.opcode != removal.opcode
                    || site_key.target != removal.target
                    || removal.argument_index >= args.len()
                {
                    continue;
                }
                let argument = &mut args[removal.argument_index];
                if class_source_expression_reference_presentation_wrapper(
                    &ast.projection,
                    site_key,
                    removal.argument_index,
                    argument,
                )
                .is_none()
                {
                    continue;
                }
                let original_origin = argument.origin.clone();
                let old = std::mem::replace(argument, Expr::new(ExprKind::Null, original_origin));
                let ExprKind::Cast { value, .. } = old.kind else {
                    unreachable!("the wrapper predicate requires a cast")
                };
                *argument = *value;
                removed[*index] += 1;
            }
        }
        let Some(indices) = edits_by_bci.get(&bci) else {
            return;
        };
        for index in indices {
            let edit = &edits[*index];
            if site_key.call_bci != bci
                || site_key.opcode != edit.opcode
                || site_key.target != edit.target
                || name != edit.target.name()
                || descriptor_parameter_count(edit.target.descriptor()) != Some(args.len())
                || edit.argument_index >= args.len()
            {
                continue;
            }
            let argument = &mut args[edit.argument_index];
            // Do not stack presentation casts on an already transformed/ambiguous source node.
            if matches!(&argument.kind, ExprKind::Cast { .. }) {
                matched[*index] = usize::MAX;
                continue;
            }
            let original_origin = argument.origin.clone();
            let old =
                std::mem::replace(argument, Expr::new(ExprKind::Null, original_origin.clone()));
            *argument = Expr::new(
                ExprKind::Cast {
                    ty: edit.ty.clone(),
                    value: Box::new(old),
                },
                original_origin,
            );
            matched[*index] += 1;
        }
    });
    if matched.iter().any(|count| *count != 1) || removed.iter().any(|count| *count != 1) {
        return Ok(None);
    }
    let projection = ClassSourceMethodAstSource {
        program,
        member: ast.projection.member.clone(),
        current_class: ast.projection.current_class.clone(),
        nested_class_members: ast.projection.nested_class_members.clone(),
        parameter_names: ast.projection.parameter_names.clone(),
        parameter_slots: ast.projection.parameter_slots.clone(),
        complete_code: ast.projection.complete_code,
        has_exception_handlers: ast.projection.has_exception_handlers,
        instruction_count: ast.projection.instruction_count,
        instruction_bcis: ast.projection.instruction_bcis.clone(),
        call_targets: ast.projection.call_targets.clone(),
        anonymous_constructor_initializer_bci: ast
            .projection
            .anonymous_constructor_initializer_bci
            .clone(),
        generic_call_init: ast.projection.generic_call_init.clone(),
    };
    Ok(Some(ClassSourceMethodAst {
        projection: Arc::new(projection),
    }))
}

/// Lists real declarations from the retained AST, with the container anchors that distinguish
/// same-named locals in different lexical scopes.
#[doc(hidden)]
pub fn class_source_local_declarations(
    ast: &ClassSourceMethodAst,
    budget: &mut Budget,
) -> Result<Option<Vec<ClassSourceAstLocalDeclaration>>, crate::stop::StopReason> {
    use crate::ast::{Stmt, StmtKind};

    if !ast.projection.complete_code {
        return Ok(None);
    }
    charge_class_source_ast(&ast.projection, budget)?;
    fn visit(
        statements: &[Stmt],
        member: &PhysicalMethodId,
        scopes: &mut Vec<ClassSourceAstAnchor>,
        declarations: &mut Vec<ClassSourceAstLocalDeclaration>,
        budget: &mut Budget,
    ) -> Result<(), crate::stop::StopReason> {
        for statement in statements {
            let bci = statement.origin.primary().bci();
            charge_class_source_metadata(budget, 1, Some(bci))?;
            match &statement.kind {
                StmtKind::Declare {
                    ty,
                    source_type_name,
                    name,
                    ..
                } => {
                    charge_class_source_metadata(
                        budget,
                        scopes.len().saturating_add(1),
                        Some(bci),
                    )?;
                    declarations.push(ClassSourceAstLocalDeclaration {
                        bci,
                        name: name.clone(),
                        ty: ty.clone(),
                        source_type_name: source_type_name.clone(),
                        scope_anchors: scopes.clone(),
                    });
                }
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    scopes.push(class_source_ast_anchor(statement.origin.primary(), member));
                    visit(then_body, member, scopes, declarations, budget)?;
                    visit(else_body, member, scopes, declarations, budget)?;
                    scopes.pop();
                }
                StmtKind::While { body, .. }
                | StmtKind::DoWhile { body, .. }
                | StmtKind::Synchronized { body, .. } => {
                    scopes.push(class_source_ast_anchor(statement.origin.primary(), member));
                    visit(body, member, scopes, declarations, budget)?;
                    scopes.pop();
                }
                StmtKind::ForEach { ty, name, body, .. } => {
                    charge_class_source_metadata(
                        budget,
                        scopes.len().saturating_add(1),
                        Some(bci),
                    )?;
                    declarations.push(ClassSourceAstLocalDeclaration {
                        bci,
                        name: name.clone(),
                        ty: ty.clone(),
                        source_type_name: None,
                        scope_anchors: scopes.clone(),
                    });
                    scopes.push(class_source_ast_anchor(statement.origin.primary(), member));
                    visit(body, member, scopes, declarations, budget)?;
                    scopes.pop();
                }
                StmtKind::For {
                    init, update, body, ..
                } => {
                    scopes.push(class_source_ast_anchor(statement.origin.primary(), member));
                    visit(
                        std::slice::from_ref(init),
                        member,
                        scopes,
                        declarations,
                        budget,
                    )?;
                    visit(
                        std::slice::from_ref(update),
                        member,
                        scopes,
                        declarations,
                        budget,
                    )?;
                    visit(body, member, scopes, declarations, budget)?;
                    scopes.pop();
                }
                StmtKind::Switch { arms, .. } => {
                    scopes.push(class_source_ast_anchor(statement.origin.primary(), member));
                    for arm in arms {
                        visit(&arm.body, member, scopes, declarations, budget)?;
                    }
                    scopes.pop();
                }
                StmtKind::Try {
                    resources,
                    catches,
                    body,
                    finally_body,
                } => {
                    for resource in resources {
                        charge_class_source_metadata(
                            budget,
                            scopes.len().saturating_add(1),
                            Some(bci),
                        )?;
                        declarations.push(ClassSourceAstLocalDeclaration {
                            bci,
                            name: resource.name.clone(),
                            ty: resource.ty.clone(),
                            source_type_name: None,
                            scope_anchors: scopes.clone(),
                        });
                    }
                    scopes.push(class_source_ast_anchor(statement.origin.primary(), member));
                    visit(body, member, scopes, declarations, budget)?;
                    for catch in catches {
                        charge_class_source_metadata(
                            budget,
                            scopes.len().saturating_add(1),
                            Some(bci),
                        )?;
                        declarations.push(ClassSourceAstLocalDeclaration {
                            bci,
                            name: catch.name.clone(),
                            ty: crate::ast::Type::Reference(catch.ty.clone()),
                            source_type_name: None,
                            scope_anchors: scopes.clone(),
                        });
                        visit(&catch.body, member, scopes, declarations, budget)?;
                    }
                    if let Some(finally_body) = finally_body {
                        visit(finally_body, member, scopes, declarations, budget)?;
                    }
                    scopes.pop();
                }
                _ => {}
            }
        }
        Ok(())
    }
    let mut declarations = Vec::new();
    visit(
        &ast.projection.program.stmts,
        &ast.projection.member,
        &mut Vec::new(),
        &mut declarations,
        budget,
    )?;
    Ok(Some(declarations))
}

/// Returns only unambiguous emitted formal names with their physical local slots.
#[doc(hidden)]
pub fn class_source_method_formal_names(
    ast: &ClassSourceMethodAst,
    budget: &mut Budget,
) -> Result<Option<Vec<ClassSourceMethodFormalName>>, crate::stop::StopReason> {
    if ast.projection.parameter_slots.len() != ast.projection.parameter_names.len() {
        return Ok(None);
    }
    charge_class_source_metadata(
        budget,
        ast.projection.parameter_slots.len(),
        ast.projection.instruction_bcis.first().copied(),
    )?;
    Ok(Some(
        ast.projection
            .parameter_slots
            .iter()
            .copied()
            .zip(ast.projection.parameter_names.iter())
            .filter_map(|(slot, name)| {
                name.as_ref().map(|name| ClassSourceMethodFormalName {
                    slot,
                    name: name.clone(),
                })
            })
            .collect(),
    ))
}

fn charge_class_source_ast(
    source: &ClassSourceMethodAstSource,
    budget: &mut Budget,
) -> Result<(), crate::stop::StopReason> {
    let first_bci = source
        .program
        .stmts
        .first()
        .map(|stmt| stmt.origin.primary().bci());
    crate::stop::poll(budget, first_bci)?;
    let node_count = program_node_count(&source.program);
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        node_count,
        first_bci,
    )?;
    crate::stop::poll(budget, source.instruction_bcis.first().copied())
}

fn charge_class_source_metadata(
    budget: &mut Budget,
    items: usize,
    bci: Option<u32>,
) -> Result<(), crate::stop::StopReason> {
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
        u64::try_from(items).unwrap_or(u64::MAX),
        bci,
    )?;
    crate::stop::poll(budget, bci)
}

fn log2_upper_bound(items: usize) -> usize {
    if items <= 1 {
        0
    } else {
        usize::BITS as usize - (items - 1).leading_zeros() as usize
    }
}

fn class_source_invoke_ast_expression(
    expression: &crate::ast::Expr,
    member: &PhysicalMethodId,
) -> ClassSourceInvokeAstExpression {
    use crate::ast::ExprKind;

    ClassSourceInvokeAstExpression {
        primary: class_source_ast_anchor(expression.origin.primary(), member),
        derived: expression
            .origin
            .derived()
            .iter()
            .map(|origin| class_source_ast_anchor(origin, member))
            .collect(),
        direct_local_name: match &expression.kind {
            crate::ast::ExprKind::Local(name) => Some(name.clone()),
            _ => None,
        },
        null_literal: matches!(&expression.kind, crate::ast::ExprKind::Null),
        shape: match &expression.kind {
            ExprKind::Local(_) => ClassSourceAstExpressionShape::Local,
            ExprKind::Null => ClassSourceAstExpressionShape::Null,
            ExprKind::New {
                ty,
                qualifier,
                args,
                ..
            } => ClassSourceAstExpressionShape::New {
                ty: ty.clone(),
                argument_count: args.len(),
                qualified: qualifier.is_some(),
            },
            ExprKind::Call { name, args, .. } => ClassSourceAstExpressionShape::Call {
                name: name.clone(),
                argument_count: args.len(),
            },
            ExprKind::Not { value } => match &value.kind {
                ExprKind::Local(local_name) => ClassSourceAstExpressionShape::BooleanNotLocal {
                    local_name: local_name.clone(),
                    primary: class_source_ast_anchor(value.origin.primary(), member),
                    derived: value
                        .origin
                        .derived()
                        .iter()
                        .map(|origin| class_source_ast_anchor(origin, member))
                        .collect(),
                    presented_type: value.presented.clone(),
                },
                _ => ClassSourceAstExpressionShape::Other,
            },
            ExprKind::Cast { .. } => ClassSourceAstExpressionShape::Cast,
            ExprKind::Integer(_)
            | ExprKind::IntegerConstantName { .. }
            | ExprKind::Boolean(_)
            | ExprKind::Long(_)
            | ExprKind::Float(_)
            | ExprKind::Double(_)
            | ExprKind::ClassLiteral { .. } => ClassSourceAstExpressionShape::Literal,
            ExprKind::Str(_) => ClassSourceAstExpressionShape::StringLiteral,
            _ => ClassSourceAstExpressionShape::Other,
        },
        presented_type: expression.presented.clone(),
        presentation_wrapper: None,
    }
}

fn class_source_invoke_ast_argument_expression(
    source: &ClassSourceMethodAstSource,
    member: &PhysicalMethodId,
    call_expression: &crate::ast::Expr,
    key: &ClassSourceInvokeKey,
    argument_index: usize,
    argument: &crate::ast::Expr,
) -> ClassSourceInvokeAstExpression {
    let mut fact = class_source_invoke_ast_expression(argument, member);
    if let Some((ty, child)) = class_source_exact_reference_presentation_wrapper(
        source,
        call_expression,
        key,
        argument_index,
        argument,
    ) {
        fact.presentation_wrapper = Some(ClassSourceAstPresentationWrapper {
            ty: ty.clone(),
            child: Box::new(class_source_invoke_ast_expression(child, member)),
        });
    }
    fact
}

fn class_source_exact_reference_presentation_wrapper<'a>(
    source: &ClassSourceMethodAstSource,
    call_expression: &crate::ast::Expr,
    key: &ClassSourceInvokeKey,
    argument_index: usize,
    argument: &'a crate::ast::Expr,
) -> Option<(&'a crate::ast::Type, &'a crate::ast::Expr)> {
    use crate::ast::{ExprKind, Type};
    let ExprKind::Call { name, args, .. } = &call_expression.kind else {
        return None;
    };
    if call_expression.origin.primary().bci() != key.call_bci
        || call_expression
            .origin
            .primary()
            .method()
            .is_some_and(|method| method != &source.member)
        || name != key.target.name()
        || args
            .get(argument_index)
            .is_none_or(|candidate| !std::ptr::eq(candidate, argument))
    {
        return None;
    }
    let target_index = source
        .call_targets
        .partition_point(|(bci, _, _)| *bci < key.call_bci);
    let (call_bci, opcode, target) = source.call_targets.get(target_index)?;
    if *call_bci != key.call_bci || *opcode != key.opcode || target != &key.target {
        return None;
    }
    let required_type =
        class_source_descriptor_reference_parameter(target.descriptor(), argument_index)?;
    let ExprKind::Cast { ty, value } = &argument.kind else {
        return None;
    };
    if ty != &Type::Reference(required_type)
        || argument.presented.as_ref() != Some(ty)
        || !matches!(&value.kind, ExprKind::Call { .. } | ExprKind::Null)
        || argument.origin.primary() != value.origin.primary()
        || !class_source_origin_belongs_to_method(argument.origin.primary(), &source.member)
        || argument.origin.derived().len() != value.origin.derived().len().saturating_add(1)
        || &argument.origin.derived()[..value.origin.derived().len()] != value.origin.derived()
    {
        return None;
    }
    let wrapper_anchor = argument.origin.derived().last()?;
    if wrapper_anchor.bci() != key.call_bci
        || wrapper_anchor.provenance() != crate::source_map::Provenance::Derived
        || wrapper_anchor.method().is_some()
    {
        return None;
    }
    if value.origin.derived().iter().any(|origin| {
        origin.bci() == key.call_bci
            || !class_source_origin_belongs_to_method(origin, &source.member)
    }) {
        return None;
    }
    Some((ty, value))
}

fn class_source_expression_reference_presentation_wrapper<'a>(
    source: &ClassSourceMethodAstSource,
    key: &ClassSourceInvokeKey,
    argument_index: usize,
    argument: &'a crate::ast::Expr,
) -> Option<&'a crate::ast::Expr> {
    use crate::ast::{ExprKind, Type};
    if !source.instruction_bcis.contains(&key.call_bci) {
        return None;
    }
    let target_index = source
        .call_targets
        .partition_point(|(bci, _, _)| *bci < key.call_bci);
    let (call_bci, opcode, target) = source.call_targets.get(target_index)?;
    if *call_bci != key.call_bci
        || *opcode != key.opcode
        || target != &key.target
        || source
            .call_targets
            .get(target_index + 1)
            .is_some_and(|(next_bci, _, _)| *next_bci == key.call_bci)
    {
        return None;
    }
    let ExprKind::Cast { ty, value } = &argument.kind else {
        return None;
    };
    let required_type =
        class_source_descriptor_reference_parameter(key.target.descriptor(), argument_index)?;
    if ty != &Type::Reference(required_type)
        || argument.presented.as_ref() != Some(ty)
        || !matches!(&value.kind, ExprKind::Call { .. } | ExprKind::Null)
        || argument.origin.primary() != value.origin.primary()
        || !class_source_origin_belongs_to_method(argument.origin.primary(), &source.member)
        || argument.origin.derived().len() != value.origin.derived().len().saturating_add(1)
        || &argument.origin.derived()[..value.origin.derived().len()] != value.origin.derived()
    {
        return None;
    }
    let wrapper_anchor = argument.origin.derived().last()?;
    if wrapper_anchor.bci() != key.call_bci
        || wrapper_anchor.provenance() != crate::source_map::Provenance::Derived
        || wrapper_anchor.method().is_some()
    {
        return None;
    }
    if value.origin.derived().iter().any(|origin| {
        origin.bci() == key.call_bci
            || !class_source_origin_belongs_to_method(origin, &source.member)
    }) {
        return None;
    }
    Some(value)
}

fn class_source_origin_belongs_to_method(
    origin: &crate::source_map::Origin,
    member: &PhysicalMethodId,
) -> bool {
    origin.method().is_none_or(|method| method == member)
}

fn class_source_descriptor_reference_parameter(
    descriptor: &str,
    argument_index: usize,
) -> Option<String> {
    use jarde_reader::classfile::Base;

    let bytes = descriptor.as_bytes();
    let facts = descriptor_facts(bytes, DescriptorKind::Method).ok()?;
    let component = facts.parameters().get(argument_index)?;
    let component_bytes = component.bytes(bytes)?;
    let dimensions = usize::try_from(component.dimensions()).ok()?;
    let base = match component.base() {
        Base::Object(name) => {
            let name = std::str::from_utf8(name.0.as_slice()).ok()?;
            if name.is_empty() {
                return None;
            }
            name.replace('/', ".")
        }
        Base::Primitive(primitive) if dimensions > 0 => match primitive {
            jarde_reader::classfile::BaseType::Boolean => "boolean".to_owned(),
            jarde_reader::classfile::BaseType::Byte => "byte".to_owned(),
            jarde_reader::classfile::BaseType::Char => "char".to_owned(),
            jarde_reader::classfile::BaseType::Short => "short".to_owned(),
            jarde_reader::classfile::BaseType::Int => "int".to_owned(),
            jarde_reader::classfile::BaseType::Long => "long".to_owned(),
            jarde_reader::classfile::BaseType::Float => "float".to_owned(),
            jarde_reader::classfile::BaseType::Double => "double".to_owned(),
        },
        Base::Primitive(_) => return None,
    };
    if component_bytes.is_empty() {
        return None;
    }
    Some(format!("{base}{}", "[]".repeat(dimensions)))
}

fn class_source_invoke_ast_expression_anchor_count(expression: &crate::ast::Expr) -> usize {
    let direct = expression.origin.derived().len().saturating_add(1);
    match &expression.kind {
        crate::ast::ExprKind::Not { value }
            if matches!(&value.kind, crate::ast::ExprKind::Local(_)) =>
        {
            direct.saturating_add(value.origin.derived().len().saturating_add(1))
        }
        _ => direct,
    }
}

fn class_source_ast_anchor(
    origin: &crate::source_map::Origin,
    member: &PhysicalMethodId,
) -> ClassSourceAstAnchor {
    let origin = origin.clone().in_body(Some(member));
    ClassSourceAstAnchor {
        bci: origin.bci(),
        method: origin.method().cloned(),
        provenance: origin.provenance(),
    }
}

fn descriptor_parameter_count(descriptor: &str) -> Option<usize> {
    let facts = descriptor_facts(descriptor.as_bytes(), DescriptorKind::Method).ok()?;
    Some(facts.parameters().len())
}

/// Emits the retained statements of one selected physical class-source method. The supplied
/// indentation is an adapter concern; the AST and physical method identity remain this run's.
#[doc(hidden)]
pub fn emit_class_source_method_ast(
    ast: &ClassSourceMethodAst,
    indentation: usize,
    budget: &mut Budget,
) -> Result<String, crate::stop::StopReason> {
    crate::emit::emit_class_source_statements(
        &ast.projection.program.stmts,
        &ast.projection.member,
        ast.projection.current_class.as_deref(),
        &ast.projection.nested_class_members,
        indentation,
        budget,
    )
}

/// The javac assert-switch field name, for the class-source projection that census-checks it.
#[doc(hidden)]
pub const ASSERT_SWITCH_FIELD_NAME: &str = crate::asserts::SWITCH_FIELD_NAME;

/// One member's guard sites folded to `assert` statements, as the class-source projection
/// consumes them: the folded statements, and the `getstatic` BCIs the fold consumed.
#[doc(hidden)]
pub type ClassSourceAssertMemberFold = crate::asserts::AssertMemberFold;

/// The proved switch-initialization line of one `<clinit>` body: the class literal its
/// `desiredAssertionStatus()` call reads, the write's BCI, and the statement's position.
#[doc(hidden)]
pub type ClassSourceAssertSwitchLine = crate::asserts::AssertSwitchLine;

/// Folds one retained member's javac assert guards into `assert` statements.
///
/// `None` keeps the member verbatim: the body reads the switch field somewhere this fold cannot
/// rewrite. The walk charges the budget per statement and expression node it touches.
#[doc(hidden)]
pub fn class_source_assert_member_fold(
    ast: &ClassSourceMethodAst,
    field: &str,
    budget: &mut Budget,
) -> Result<Option<ClassSourceAssertMemberFold>, crate::stop::StopReason> {
    crate::asserts::fold_member_guards(&ast.projection.program.stmts, field, budget)
}

/// Reads one retained `<clinit>` program's switch-initialization line, when it holds exactly the
/// one javac shape.
#[doc(hidden)]
pub fn class_source_assert_switch_line(
    ast: &ClassSourceMethodAst,
    field: &str,
    budget: &mut Budget,
) -> Result<Option<ClassSourceAssertSwitchLine>, crate::stop::StopReason> {
    crate::asserts::switch_line(&ast.projection.program.stmts, field, budget)
}

/// Emits one projected statement list of a member the assert projection rewrote, under the same
/// member context its own artifact was written in.
#[doc(hidden)]
pub fn emit_class_source_assert_statements(
    ast: &ClassSourceMethodAst,
    statements: &[crate::ast::Stmt],
    budget: &mut Budget,
) -> Result<String, crate::stop::StopReason> {
    crate::emit::emit_class_source_statements(
        statements,
        &ast.projection.member,
        ast.projection.current_class.as_deref(),
        &ast.projection.nested_class_members,
        1,
        budget,
    )
}

/// Emits the member's own retained statements unmodified, as the assert projection's baseline:
/// the caller checks this against the member's placed text before replacing it, so a body
/// another projection already claimed is never silently re-fetched.
#[doc(hidden)]
pub fn class_source_assert_member_current(
    ast: &ClassSourceMethodAst,
    budget: &mut Budget,
) -> Result<String, crate::stop::StopReason> {
    crate::emit::emit_class_source_statements(
        &ast.projection.program.stmts,
        &ast.projection.member,
        ast.projection.current_class.as_deref(),
        &ast.projection.nested_class_members,
        1,
        budget,
    )
}

/// The `<clinit>` program with its one switch-initialization line removed, when the program
/// holds exactly the one line shape. An empty remainder is the caller's signal that the whole
/// member goes.
#[doc(hidden)]
pub fn class_source_assert_clinit_without_line(
    ast: &ClassSourceMethodAst,
    field: &str,
    budget: &mut Budget,
) -> Result<Option<Vec<crate::ast::Stmt>>, crate::stop::StopReason> {
    let Some(line) = crate::asserts::switch_line(&ast.projection.program.stmts, field, budget)?
    else {
        return Ok(None);
    };
    let mut statements = ast.projection.program.stmts.clone();
    statements.remove(line.position);
    Ok(Some(statements))
}

/// A name admitted only for a class-source projection. The physical method report is untouched.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerConstantName {
    pub value: i32,
    pub name: String,
}

/// The original method BCI associated with one name written by the projected AST.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerConstantNameUse {
    pub name: String,
    pub bci: u32,
    pub case_label: bool,
}

/// Project only integer switch labels and the direct integer return in a selected arm.
/// `None` means the same-run AST proves no safe replacement; a stop is propagated before publish.
#[doc(hidden)]
pub fn project_class_source_integer_constants(
    ast: &ClassSourceMethodAst,
    candidates: &[IntegerConstantName],
    returns_int: bool,
    budget: &mut Budget,
) -> Result<Option<(String, Vec<IntegerConstantNameUse>)>, crate::stop::StopReason> {
    use crate::ast::{ExprKind, StmtKind, SwitchLabels, Type};
    if !ast.projection.complete_code
        || candidates.is_empty()
        || ast.projection.parameter_names.iter().any(Option::is_none)
    {
        return Ok(None);
    }
    let mut occupied = std::collections::HashSet::new();
    occupied.extend(ast.projection.parameter_names.iter().flatten().cloned());
    let mut pending: Vec<&crate::ast::Stmt> = ast.projection.program.stmts.iter().collect();
    while let Some(stmt) = pending.pop() {
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            Some(stmt.origin.primary().bci()),
        )?;
        match &stmt.kind {
            StmtKind::Declare { name, .. } | StmtKind::ForEach { name, .. } => {
                occupied.insert(name.clone());
            }
            _ => {}
        }
        match &stmt.kind {
            StmtKind::If {
                then_body,
                else_body,
                ..
            } => {
                pending.extend(then_body);
                pending.extend(else_body);
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::ForEach { body, .. }
            | StmtKind::Synchronized { body, .. } => pending.extend(body),
            StmtKind::For {
                init, update, body, ..
            } => {
                pending.push(init);
                pending.push(update);
                pending.extend(body);
            }
            StmtKind::Switch { arms, .. } => {
                for arm in arms {
                    pending.extend(&arm.body);
                }
            }
            StmtKind::Try {
                resources,
                catches,
                body,
                finally_body,
            } => {
                occupied.extend(resources.iter().map(|resource| resource.name.clone()));
                for catch in catches {
                    occupied.insert(catch.name.clone());
                    pending.extend(&catch.body);
                }
                pending.extend(body);
                pending.extend(finally_body.iter().flatten());
            }
            _ => {}
        }
    }
    let names: std::collections::HashMap<i64, &str> = candidates
        .iter()
        .filter(|candidate| !occupied.contains(&candidate.name))
        .map(|candidate| (i64::from(candidate.value), candidate.name.as_str()))
        .collect();
    if names.is_empty() {
        return Ok(None);
    }
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        program_node_count(&ast.projection.program),
        ast.projection
            .program
            .stmts
            .first()
            .map(|stmt| stmt.origin.primary().bci()),
    )?;
    let mut program = ast.projection.program.clone();
    let mut uses = Vec::new();
    let mut pending: Vec<&mut crate::ast::Stmt> = program.stmts.iter_mut().collect();
    while let Some(stmt) = pending.pop() {
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            Some(stmt.origin.primary().bci()),
        )?;
        match &mut stmt.kind {
            StmtKind::If {
                then_body,
                else_body,
                ..
            } => {
                pending.extend(then_body);
                pending.extend(else_body);
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::ForEach { body, .. }
            | StmtKind::Synchronized { body, .. } => pending.extend(body),
            StmtKind::For {
                init, update, body, ..
            } => {
                pending.push(init);
                pending.push(update);
                pending.extend(body);
            }
            StmtKind::Switch { value, arms } => {
                if value.presented == Some(Type::Int) {
                    for arm in arms.iter_mut() {
                        if arm.labels.is_some() {
                            continue;
                        }
                        let labels: Vec<String> = arm
                            .keys
                            .iter()
                            .map(|key| {
                                names
                                    .get(key)
                                    .map_or_else(|| key.to_string(), |name| (*name).to_owned())
                            })
                            .collect();
                        let replaced = arm
                            .keys
                            .iter()
                            .zip(&labels)
                            .any(|(key, label)| *label != key.to_string());
                        if replaced {
                            for (key, label) in arm.keys.iter().zip(&labels) {
                                if *label != key.to_string() {
                                    uses.push(IntegerConstantNameUse {
                                        name: label.clone(),
                                        bci: stmt.origin.primary().bci(),
                                        case_label: true,
                                    });
                                }
                            }
                            arm.labels = Some(SwitchLabels::Integer(labels));
                        }
                        if returns_int
                            && !arm.keys.is_empty()
                            && let [direct] = arm.body.as_mut_slice()
                            && let StmtKind::Return {
                                value: Some(returned),
                            } = &mut direct.kind
                            && returned.presented == Some(Type::Int)
                            && let ExprKind::Integer(number) = returned.kind
                            && let Some(name) = names.get(&number)
                        {
                            uses.push(IntegerConstantNameUse {
                                name: (*name).to_owned(),
                                bci: returned.origin.primary().bci(),
                                case_label: false,
                            });
                            returned.kind = ExprKind::IntegerConstantName {
                                name: (*name).to_owned(),
                                value: number,
                            };
                        }
                        pending.extend(&mut arm.body);
                    }
                } else {
                    for arm in arms {
                        pending.extend(&mut arm.body);
                    }
                }
            }
            StmtKind::Try {
                catches,
                body,
                finally_body,
                ..
            } => {
                for catch in catches {
                    pending.extend(&mut catch.body);
                }
                pending.extend(body);
                pending.extend(finally_body.iter_mut().flatten());
            }
            _ => {}
        }
    }
    if uses.is_empty() {
        return Ok(None);
    }
    let body = crate::emit::emit_class_source_statements(
        &program.stmts,
        &ast.projection.member,
        ast.projection.current_class.as_deref(),
        &ast.projection.nested_class_members,
        1,
        budget,
    )?;
    Ok(Some((body, uses)))
}

/// The BCI of the one statically proved field write in the narrow anonymous constructor shape.
#[doc(hidden)]
pub fn class_source_anonymous_constructor_initializer_bci(
    ast: &ClassSourceMethodAst,
) -> Option<ClassSourceAnonymousConstructorInitializer> {
    ast.projection.anonymous_constructor_initializer_bci.clone()
}

/// Emits only the initializer statement selected by the same-run constructor proof.
#[doc(hidden)]
pub fn emit_class_source_anonymous_constructor_initializer(
    ast: &ClassSourceMethodAst,
    bci: u32,
    budget: &mut Budget,
) -> Result<Option<String>, crate::stop::StopReason> {
    let Some(statement) = ast.projection.program.stmts.iter().find(|statement| {
        statement.origin.primary().bci() == bci
            && matches!(statement.kind, crate::ast::StmtKind::FieldAssign { .. })
    }) else {
        return Ok(None);
    };
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        1,
        Some(bci),
    )?;
    crate::stop::poll(budget, Some(bci))?;
    crate::emit::emit_class_source_statements(
        std::slice::from_ref(statement),
        &ast.projection.member,
        ast.projection.current_class.as_deref(),
        &ast.projection.nested_class_members,
        4,
        budget,
    )
    .map(Some)
}

/// The proved instance-block body of one anonymous subclass constructor, as the double-brace
/// allocation point presents it (change `recover-double-brace-allocation-site`).
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceAnonymousInstanceBlock {
    /// Every top-level statement the constructor runs after its call, minus the certified capture
    /// store and the closing `return`, emitted at the requested indentation.
    pub text: String,
    /// How many of the constructor's leading parameters the call forwards, in order. The
    /// double-brace form re-states the *allocation's* arguments as the superclass constructor's
    /// own, so this count is what closes the two lists against each other: the site's arguments
    /// are the constructor's parameters, and the call forwards its leading ones.
    pub forwarded_parameters: usize,
}

/// Emits the instance-block statements of one proved anonymous constructor: every top-level
/// statement after the constructor call, minus the certified capture store and the closing
/// `return`, with every proved read of the capture field re-spelled as the allocation site's own
/// local.
///
/// # What the shape is, and why every conjunct is read and not assumed
///
/// javac compiles an anonymous subclass of a superclass as: the synthetic capture stores, the
/// `super(…)` call, the instance initializer's statements — and a closing `return`. The
/// double-brace source form re-creates all of that from one expression, so the reading has to
/// state which statements the block owns and that the form's own re-creation cannot differ:
///
/// * the call is the constructor's own **prologue**, at top level and unique, and everything
///   before it is the certified capture store the caller's proof named (`capture_write_bci`);
/// * the body is everything after the call, minus that same store (the certified group may sit
///   after the call as well as before it) and minus the closing `return`; an **empty** body is not
///   this shape — the double-brace form is the *block*, and a constructor with nothing to run is
///   the empty class body the existing anonymous-superclass presentation already writes;
/// * the call's arguments are the constructor's own **leading parameters, in order**. The form
///   re-states the allocation's arguments in that order, so a call that reads anything else would
///   compile into a program the class file does not have;
/// * the block is placed in the **enclosing method's scope**, so nothing in it may name a thing
///   that scope does not resolve the same way: a read of one of the constructor's parameters
///   (other than through the capture field, which is re-spelled), a `return` — illegal in an
///   initializer — a pool-form allocation (`X$1$1` is not a source name), and a local declaration
///   that would shadow one of the caller's reserved names are each a refusal rather than a text
///   that silently means something else.
#[doc(hidden)]
pub fn emit_class_source_anonymous_instance_block(
    ast: &ClassSourceMethodAst,
    capture_write_bci: Option<u32>,
    capture_reads: &[ProvedCapturedParameterRead],
    reserved_names: &[String],
    indentation: usize,
    budget: &mut Budget,
) -> Result<Option<ClassSourceAnonymousInstanceBlock>, crate::stop::StopReason> {
    let statements = &ast.projection.program.stmts;
    if ast.projection.program.ragged || ast.projection.program.statements != statements.len() {
        return Ok(None);
    }
    let mut prologue = None;
    for (index, statement) in statements.iter().enumerate() {
        if matches!(
            statement.kind,
            StmtKind::ConstructorCall {
                target: ConstructorTarget::Super,
                ..
            }
        ) {
            if prologue.is_some() {
                return Ok(None);
            }
            prologue = Some(index);
        }
    }
    let Some(prologue) = prologue else {
        return Ok(None);
    };
    if statements[..prologue]
        .iter()
        .any(|statement| Some(statement.origin.primary().bci()) != capture_write_bci)
    {
        return Ok(None);
    }
    let Some((closing, body)) = statements[prologue + 1..].split_last() else {
        return Ok(None);
    };
    if !matches!(closing.kind, StmtKind::Return { value: None }) {
        return Ok(None);
    }
    let body: Vec<&crate::ast::Stmt> = body
        .iter()
        .filter(|statement| Some(statement.origin.primary().bci()) != capture_write_bci)
        .collect();
    if body.is_empty() {
        return Ok(None);
    }
    let StmtKind::ConstructorCall { args, .. } = &statements[prologue].kind else {
        unreachable!("the prologue was matched as a constructor call")
    };
    let parameters = &ast.projection.parameter_names;
    if args.len() > parameters.len() {
        return Ok(None);
    }
    let leading: Vec<&str> = parameters[..args.len()]
        .iter()
        .map(|name| name.as_deref())
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default();
    if leading.len() != args.len()
        || args.iter().zip(&leading).any(
            |(argument, name)| !matches!(&argument.kind, ExprKind::Local(local) if local == name),
        )
    {
        return Ok(None);
    }
    let mut findings = InstanceBlockFindings::default();
    for statement in &body {
        instance_block_findings(statement, &mut findings, budget)?;
    }
    let named: std::collections::BTreeSet<&str> = parameters
        .iter()
        .filter_map(|name| name.as_deref())
        .collect();
    if findings
        .reads
        .iter()
        .any(|read| named.contains(read.as_str()))
        || findings.declares.iter().any(|declared| {
            named.contains(declared.as_str()) || reserved_names.iter().any(|name| name == declared)
        })
        || findings.pool_form_allocation
        || findings.returns
    {
        return Ok(None);
    }
    // The capture reads are re-spelled before the emission: the block reads the *enclosing*
    // method's local, which is the value javac's own recapture would hand the recompiled
    // anonymous class. A read the proof lists that does not map to exactly one field expression
    // in this AST keeps the physical presentation.
    let projected =
        match project_class_source_converted_parameter_reads(ast, capture_reads, budget)? {
            Some(projected) => projected,
            None if capture_reads.is_empty() => (*ast).clone(),
            None => return Ok(None),
        };
    let projected_statements = &projected.projection.program.stmts;
    if projected_statements.len() != statements.len() {
        return Ok(None);
    }
    let projected_body: Vec<crate::ast::Stmt> = projected_statements
        [prologue + 1..projected_statements.len() - 1]
        .iter()
        .filter(|statement| Some(statement.origin.primary().bci()) != capture_write_bci)
        .cloned()
        .collect();
    let text = crate::emit::emit_class_source_statements(
        &projected_body,
        &projected.projection.member,
        projected.projection.current_class.as_deref(),
        &projected.projection.nested_class_members,
        indentation,
        budget,
    )?;
    Ok(Some(ClassSourceAnonymousInstanceBlock {
        text,
        forwarded_parameters: args.len(),
    }))
}

/// What an instance block emitted into the enclosing method's scope must not carry, each read
/// from the same-run AST: a read of a name that scope may not resolve to the same thing, a local
/// declaration that would shadow one, a pool-form allocation, and a `return` an initializer may
/// not hold.
#[derive(Default)]
struct InstanceBlockFindings {
    /// Every local name the block reads.
    reads: Vec<String>,
    /// Every local name the block declares or assigns.
    declares: Vec<String>,
    /// Whether the block allocates a type whose name is the pool form (`X$1$1`).
    pool_form_allocation: bool,
    /// Whether the block holds a `return`, at any depth.
    returns: bool,
}

/// Reads one statement subtree into [`InstanceBlockFindings`]. Every charge is per node, so a
/// block that does not close is a stop the caller sees rather than a silent admission.
fn instance_block_findings(
    statement: &crate::ast::Stmt,
    findings: &mut InstanceBlockFindings,
    budget: &mut Budget,
) -> Result<(), crate::stop::StopReason> {
    let mut statements = vec![statement];
    let mut expressions: Vec<&Expr> = Vec::new();
    while let Some(statement) = statements.pop() {
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            Some(statement.origin.primary().bci()),
        )?;
        crate::stop::poll(budget, Some(statement.origin.primary().bci()))?;
        use StmtKind as K;
        match &statement.kind {
            K::Declare { name, value, .. } => {
                findings.declares.push(name.clone());
                expressions.extend(value.iter());
            }
            K::Assign { name, value } => {
                findings.declares.push(name.clone());
                expressions.push(value);
            }
            K::Expr(value) | K::Throw { value } => expressions.push(value),
            K::FieldAssign {
                receiver, value, ..
            } => {
                expressions.extend(receiver.iter());
                expressions.push(value);
            }
            K::IndexAssign {
                array,
                index,
                value,
                ..
            } => expressions.extend([array, index, value]),
            K::ConstructorCall { args, .. } => expressions.extend(args),
            K::Return { value } => {
                findings.returns = true;
                expressions.extend(value.iter());
            }
            K::Assert { cond, message } => {
                expressions.push(cond);
                expressions.extend(message.iter());
            }
            K::If {
                cond,
                then_body,
                else_body,
            } => {
                expressions.push(cond);
                statements.extend(then_body);
                statements.extend(else_body);
            }
            K::While { cond, body, .. } | K::DoWhile { cond, body, .. } => {
                expressions.push(cond);
                statements.extend(body);
            }
            K::For {
                init,
                cond,
                update,
                body,
                ..
            } => {
                statements.push(init);
                statements.push(update);
                expressions.push(cond);
                statements.extend(body);
            }
            K::ForEach {
                name,
                iterable,
                body,
                ..
            } => {
                findings.declares.push(name.clone());
                expressions.push(iterable);
                statements.extend(body);
            }
            K::Switch { value, arms } => {
                expressions.push(value);
                for arm in arms {
                    statements.extend(&arm.body);
                }
            }
            K::Try {
                resources,
                catches,
                body,
                finally_body,
            } => {
                for resource in resources {
                    findings.declares.push(resource.name.clone());
                    expressions.push(&resource.value);
                }
                for catch in catches {
                    findings.declares.push(catch.name.clone());
                    statements.extend(&catch.body);
                }
                statements.extend(body);
                statements.extend(finally_body.iter().flatten());
            }
            K::Synchronized { lock, body } => {
                expressions.push(lock);
                statements.extend(body);
            }
            K::Break { .. } | K::Continue { .. } | K::Fallback { .. } => {}
        }
    }
    while let Some(expression) = expressions.pop() {
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            Some(expression.origin.primary().bci()),
        )?;
        use ExprKind as E;
        match &expression.kind {
            E::Local(name) => findings.reads.push(name.clone()),
            E::LocalAssign { name, value, .. } => {
                findings.declares.push(name.clone());
                expressions.push(value);
            }
            E::Call { receiver, args, .. } => {
                expressions.extend(receiver.iter().map(|receiver| &**receiver));
                expressions.extend(args);
            }
            E::New {
                ty,
                qualifier,
                args,
                ..
            } => {
                findings.pool_form_allocation |= ty.contains('$');
                expressions.extend(qualifier.iter().map(|qualifier| &**qualifier));
                expressions.extend(args);
            }
            E::Lambda { body, .. } => expressions.push(body),
            E::MethodReference { qualifier, .. } => expressions.push(qualifier),
            E::Field { receiver, .. } => expressions.push(receiver),
            E::Index { array, index } => expressions.extend([&**array, &**index]),
            E::PostfixUpdate { target, .. } => expressions.push(target),
            E::ArrayLength { array } => expressions.push(array),
            E::NewArray {
                lengths,
                initializers,
                ..
            } => {
                expressions.extend(lengths);
                expressions.extend(initializers.iter().flatten());
            }
            E::Binary { left, right, .. } => expressions.extend([&**left, &**right]),
            E::Conditional {
                test,
                when_true,
                when_false,
            } => expressions.extend([&**test, &**when_true, &**when_false]),
            E::Concat { parts } => expressions.extend(parts.iter().map(|part| &part.value)),
            E::Cast { value, .. } => expressions.push(value),
            E::Not { value } | E::Neg { value } => expressions.push(value),
            E::InstanceOf { value, .. } => expressions.push(value),
            E::Integer(_)
            | E::IntegerConstantName { .. }
            | E::Boolean(_)
            | E::Long(_)
            | E::Float(_)
            | E::Double(_)
            | E::Str(_)
            | E::Null
            | E::ClassLiteral { .. }
            | E::Path(_)
            | E::QualifiedThis { .. }
            | E::Super { .. } => {}
        }
    }
    Ok(())
}

/// Projects only the exact field-read expressions certified by a class-source capture proof.
/// The AST is the same-run sidecar: this pass does not reanalyze or recover the method again.
#[doc(hidden)]
pub fn project_class_source_captured_outer_reads(
    ast: &ClassSourceMethodAst,
    reads: &[ProvedCapturedOuterRead],
    budget: &mut Budget,
) -> Result<Option<ClassSourceMethodAst>, crate::stop::StopReason> {
    let reads = reads
        .iter()
        .map(CapturedReadReplacement::Outer)
        .collect::<Vec<_>>();
    project_class_source_captured_reads(ast, &reads, CaptureReadAnchor::Field, budget)
}

/// Projects only the exact field-read expressions certified as a root parameter capture.
#[doc(hidden)]
pub fn project_class_source_captured_parameter_reads(
    ast: &ClassSourceMethodAst,
    reads: &[ProvedCapturedParameterRead],
    budget: &mut Budget,
) -> Result<Option<ClassSourceMethodAst>, crate::stop::StopReason> {
    let reads = reads
        .iter()
        .map(CapturedReadReplacement::Parameter)
        .collect::<Vec<_>>();
    project_class_source_captured_reads(ast, &reads, CaptureReadAnchor::Field, budget)
}

/// [`project_class_source_captured_parameter_reads`] with one conversion admitted over the field:
/// the reading the double-brace allocation point states (change
/// `recover-double-brace-allocation-site`), whose reads sit in the companion constructor's block
/// statements — mostly argument positions the build converts explicitly.
#[doc(hidden)]
pub fn project_class_source_converted_parameter_reads(
    ast: &ClassSourceMethodAst,
    reads: &[ProvedCapturedParameterRead],
    budget: &mut Budget,
) -> Result<Option<ClassSourceMethodAst>, crate::stop::StopReason> {
    let reads = reads
        .iter()
        .map(CapturedReadReplacement::Parameter)
        .collect::<Vec<_>>();
    project_class_source_captured_reads(ast, &reads, CaptureReadAnchor::Converted, budget)
}

enum CapturedReadReplacement<'a> {
    Outer(&'a ProvedCapturedOuterRead),
    Parameter(&'a ProvedCapturedParameterRead),
}

/// Which node a proved capture read's anchor may sit on, as the containment pattern every
/// widening in this family keeps: one reading admits one new shape only for the path whose slice
/// proved it, so a widening on one slice never opens another slice's acceptance set as a side
/// effect.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CaptureReadAnchor {
    /// The field expression itself: the reading every companion-body projection states, because
    /// the reads it re-spells sit where the child's own body wrote them.
    Field,
    /// The field expression, or one `Cast` over it. The build wraps a value in an explicit cast to
    /// preserve an argument position's conversion, so a read of `add(Object)`'s argument is
    /// `(java.lang.Object) this.val$x` — the field is one node inside the conversion, and the
    /// conversion stays written around the local the read is re-spelled as. The double-brace
    /// allocation point reads a companion's constructor, whose block statements are mostly
    /// argument positions, so it states this reading.
    Converted,
}

impl CapturedReadReplacement<'_> {
    fn method(&self) -> &PhysicalMethodId {
        match self {
            Self::Outer(read) => &read.method,
            Self::Parameter(read) => &read.method,
        }
    }

    fn read_bci(&self) -> u32 {
        match self {
            Self::Outer(read) => read.read_bci,
            Self::Parameter(read) => read.read_bci,
        }
    }
}

fn project_class_source_captured_reads(
    ast: &ClassSourceMethodAst,
    reads: &[CapturedReadReplacement<'_>],
    anchor: CaptureReadAnchor,
    budget: &mut Budget,
) -> Result<Option<ClassSourceMethodAst>, crate::stop::StopReason> {
    if reads.is_empty() {
        return Ok(None);
    }
    let method = &ast.projection.member;
    let mut expected = std::collections::BTreeMap::new();
    for read in reads {
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            Some(read.read_bci()),
        )?;
        crate::stop::poll(budget, Some(read.read_bci()))?;
        if read.method() != method || expected.insert(read.read_bci(), read).is_some() {
            return Ok(None);
        }
    }
    // Account for the complete AST before allocating its clone. The visitor below also
    // charges each node as it examines it, so both peak memory and traversal work stay bounded.
    let node_count = program_node_count(&ast.projection.program);
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        node_count,
        ast.projection
            .program
            .stmts
            .first()
            .map(|stmt| stmt.origin.primary().bci()),
    )?;
    crate::stop::poll(
        budget,
        ast.projection
            .program
            .stmts
            .first()
            .map(|stmt| stmt.origin.primary().bci()),
    )?;
    let mut projection = (*ast.projection).clone();
    let mut matched = std::collections::BTreeMap::<u32, usize>::new();
    for statement in &mut projection.program.stmts {
        project_captured_stmt(statement, &expected, anchor, &mut matched, budget)?;
    }
    if expected.keys().any(|bci| matched.get(bci) != Some(&1))
        || matched
            .iter()
            .any(|(bci, count)| !expected.contains_key(bci) || *count != 1)
    {
        return Ok(None);
    }
    Ok(Some(ClassSourceMethodAst {
        projection: std::sync::Arc::new(projection),
    }))
}

fn project_captured_stmt(
    stmt: &mut crate::ast::Stmt,
    expected: &std::collections::BTreeMap<u32, &CapturedReadReplacement<'_>>,
    anchor: CaptureReadAnchor,
    matched: &mut std::collections::BTreeMap<u32, usize>,
    budget: &mut Budget,
) -> Result<(), crate::stop::StopReason> {
    use crate::ast::StmtKind;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        1,
        Some(stmt.origin.primary().bci()),
    )?;
    crate::stop::poll(budget, Some(stmt.origin.primary().bci()))?;
    match &mut stmt.kind {
        StmtKind::Declare { value, .. } => value
            .iter_mut()
            .try_for_each(|e| project_captured_expr(e, expected, anchor, matched, budget))?,
        StmtKind::Assign { value, .. } | StmtKind::Expr(value) | StmtKind::Throw { value } => {
            project_captured_expr(value, expected, anchor, matched, budget)?
        }
        StmtKind::Assert { cond, message } => {
            project_captured_expr(cond, expected, anchor, matched, budget)?;
            if let Some(message) = message {
                project_captured_expr(message, expected, anchor, matched, budget)?;
            }
        }
        StmtKind::FieldAssign {
            receiver, value, ..
        } => {
            receiver
                .iter_mut()
                .try_for_each(|e| project_captured_expr(e, expected, anchor, matched, budget))?;
            project_captured_expr(value, expected, anchor, matched, budget)?;
        }
        StmtKind::IndexAssign {
            array,
            index,
            value,
            ..
        } => {
            project_captured_expr(array, expected, anchor, matched, budget)?;
            project_captured_expr(index, expected, anchor, matched, budget)?;
            project_captured_expr(value, expected, anchor, matched, budget)?;
        }
        StmtKind::ConstructorCall { args, .. } => args
            .iter_mut()
            .try_for_each(|e| project_captured_expr(e, expected, anchor, matched, budget))?,
        StmtKind::Return { value } => value
            .iter_mut()
            .try_for_each(|e| project_captured_expr(e, expected, anchor, matched, budget))?,
        StmtKind::If {
            cond,
            then_body,
            else_body,
        } => {
            project_captured_expr(cond, expected, anchor, matched, budget)?;
            project_captured_stmts(then_body, expected, anchor, matched, budget)?;
            project_captured_stmts(else_body, expected, anchor, matched, budget)?;
        }
        StmtKind::While { cond, body, .. } | StmtKind::DoWhile { cond, body, .. } => {
            project_captured_expr(cond, expected, anchor, matched, budget)?;
            project_captured_stmts(body, expected, anchor, matched, budget)?;
        }
        StmtKind::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            project_captured_stmt(init, expected, anchor, matched, budget)?;
            project_captured_expr(cond, expected, anchor, matched, budget)?;
            project_captured_stmt(update, expected, anchor, matched, budget)?;
            project_captured_stmts(body, expected, anchor, matched, budget)?;
        }
        StmtKind::ForEach { iterable, body, .. } => {
            project_captured_expr(iterable, expected, anchor, matched, budget)?;
            project_captured_stmts(body, expected, anchor, matched, budget)?;
        }
        StmtKind::Switch { value, arms } => {
            project_captured_expr(value, expected, anchor, matched, budget)?;
            for arm in arms {
                project_captured_stmts(&mut arm.body, expected, anchor, matched, budget)?;
            }
        }
        StmtKind::Try {
            resources,
            catches,
            body,
            finally_body,
        } => {
            for resource in resources {
                project_captured_expr(&mut resource.value, expected, anchor, matched, budget)?;
            }
            project_captured_stmts(body, expected, anchor, matched, budget)?;
            for catch in catches {
                project_captured_stmts(&mut catch.body, expected, anchor, matched, budget)?;
            }
            if let Some(body) = finally_body {
                project_captured_stmts(body, expected, anchor, matched, budget)?;
            }
        }
        StmtKind::Synchronized { lock, body } => {
            project_captured_expr(lock, expected, anchor, matched, budget)?;
            project_captured_stmts(body, expected, anchor, matched, budget)?;
        }
        StmtKind::Break { .. } | StmtKind::Continue { .. } | StmtKind::Fallback { .. } => {}
    }
    Ok(())
}

fn project_captured_stmts(
    stmts: &mut [crate::ast::Stmt],
    expected: &std::collections::BTreeMap<u32, &CapturedReadReplacement<'_>>,
    anchor: CaptureReadAnchor,
    matched: &mut std::collections::BTreeMap<u32, usize>,
    budget: &mut Budget,
) -> Result<(), crate::stop::StopReason> {
    for stmt in stmts {
        project_captured_stmt(stmt, expected, anchor, matched, budget)?;
    }
    Ok(())
}

fn project_captured_expr(
    expr: &mut Expr,
    expected: &std::collections::BTreeMap<u32, &CapturedReadReplacement<'_>>,
    anchor: CaptureReadAnchor,
    matched: &mut std::collections::BTreeMap<u32, usize>,
    budget: &mut Budget,
) -> Result<(), crate::stop::StopReason> {
    use crate::ast::ExprKind;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        1,
        Some(expr.origin.primary().bci()),
    )?;
    crate::stop::poll(budget, Some(expr.origin.primary().bci()))?;
    if let Some(read) = expected.get(&expr.origin.primary().bci()) {
        let valid_field = match read {
            CapturedReadReplacement::Outer(read) => read.field_name.as_str(),
            CapturedReadReplacement::Parameter(read) => read.field_name.as_str(),
        };
        if let ExprKind::Field { receiver, name } = &expr.kind
            && matches!(receiver.kind, ExprKind::Local(ref local) if local == "this")
            && name == valid_field
        {
            match read {
                CapturedReadReplacement::Outer(read) => {
                    *matched.entry(read.read_bci).or_default() += 1;
                    expr.kind = ExprKind::QualifiedThis {
                        qualifier: read.outer_source_name.clone(),
                    };
                    expr.presented = Some(Type::Reference(read.outer_source_name.clone()));
                }
                CapturedReadReplacement::Parameter(read) => {
                    *matched.entry(read.read_bci).or_default() += 1;
                    expr.kind = ExprKind::Local(read.parameter_name.clone());
                    expr.presented = read.parameter_presented.clone();
                }
            }
            return Ok(());
        }
        // The anchor may sit on one conversion the position wrote rather than on the field
        // itself: the `Cast` the build inserts to preserve an argument position's conversion
        // carries the read's own anchor and holds the field expression one node in.
        if anchor == CaptureReadAnchor::Converted
            && let ExprKind::Cast { value, .. } = &mut expr.kind
        {
            project_captured_expr(value, expected, anchor, matched, budget)?;
            return Ok(());
        }
        *matched.entry(read.read_bci()).or_default() += 2;
        return Ok(());
    }
    match &mut expr.kind {
        ExprKind::LocalAssign { value, .. }
        | ExprKind::InstanceOf { value, .. }
        | ExprKind::PostfixUpdate { target: value, .. }
        | ExprKind::ArrayLength { array: value }
        | ExprKind::Cast { value, .. }
        | ExprKind::Not { value }
        | ExprKind::Neg { value } => {
            project_captured_expr(value, expected, anchor, matched, budget)?
        }
        ExprKind::Call { receiver, args, .. } => {
            receiver
                .iter_mut()
                .try_for_each(|e| project_captured_expr(e, expected, anchor, matched, budget))?;
            args.iter_mut()
                .try_for_each(|e| project_captured_expr(e, expected, anchor, matched, budget))?;
        }
        ExprKind::New {
            qualifier, args, ..
        } => {
            qualifier
                .iter_mut()
                .try_for_each(|e| project_captured_expr(e, expected, anchor, matched, budget))?;
            args.iter_mut()
                .try_for_each(|e| project_captured_expr(e, expected, anchor, matched, budget))?;
        }
        ExprKind::Lambda { body, .. }
        | ExprKind::MethodReference {
            qualifier: body, ..
        } => project_captured_expr(body, expected, anchor, matched, budget)?,
        ExprKind::Field { receiver, .. } => {
            project_captured_expr(receiver, expected, anchor, matched, budget)?
        }
        ExprKind::Index { array, index } => {
            project_captured_expr(array, expected, anchor, matched, budget)?;
            project_captured_expr(index, expected, anchor, matched, budget)?;
        }
        ExprKind::NewArray {
            lengths,
            initializers,
            ..
        } => {
            lengths
                .iter_mut()
                .try_for_each(|e| project_captured_expr(e, expected, anchor, matched, budget))?;
            if let Some(values) = initializers {
                values.iter_mut().try_for_each(|e| {
                    project_captured_expr(e, expected, anchor, matched, budget)
                })?;
            }
        }
        ExprKind::Binary { left, right, .. } => {
            project_captured_expr(left, expected, anchor, matched, budget)?;
            project_captured_expr(right, expected, anchor, matched, budget)?;
        }
        ExprKind::Conditional {
            test,
            when_true,
            when_false,
        } => {
            project_captured_expr(test, expected, anchor, matched, budget)?;
            project_captured_expr(when_true, expected, anchor, matched, budget)?;
            project_captured_expr(when_false, expected, anchor, matched, budget)?;
        }
        ExprKind::Concat { parts } => parts.iter_mut().try_for_each(|part| {
            project_captured_expr(&mut part.value, expected, anchor, matched, budget)
        })?,
        ExprKind::Local(_)
        | ExprKind::Integer(_)
        | ExprKind::IntegerConstantName { .. }
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

/// Re-emits the proved root method while replacing only the exact direct-return allocation node
/// with the staged anonymous class methods. The physical root AST owns the resulting return and
/// allocation statement; the class-source writer never invents either expression.
#[doc(hidden)]
pub fn emit_class_source_anonymous_return(
    ast: &ClassSourceMethodAst,
    allocation_bci: u32,
    allocation_type: &str,
    source_type: &str,
    methods: &str,
    hidden_outer_argument_bci: Option<u32>,
    budget: &mut Budget,
) -> Result<Option<ClassSourceAnonymousReturn>, crate::stop::StopReason> {
    emit_class_source_anonymous_return_at(
        ast,
        allocation_bci,
        allocation_type,
        source_type,
        methods,
        hidden_outer_argument_bci,
        2,
        "        ",
        budget,
    )
}

/// The exact root-method body and expression range emitted for one selected anonymous allocation.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceAnonymousReturn {
    pub text: String,
    pub expression_range: std::ops::Range<usize>,
}

/// Nested form of [`emit_class_source_anonymous_return`] with an explicit AST/body indentation.
#[doc(hidden)]
pub fn emit_class_source_anonymous_return_at(
    ast: &ClassSourceMethodAst,
    allocation_bci: u32,
    allocation_type: &str,
    source_type: &str,
    methods: &str,
    hidden_outer_argument_bci: Option<u32>,
    indentation: usize,
    closing_indent: &str,
    budget: &mut Budget,
) -> Result<Option<ClassSourceAnonymousReturn>, crate::stop::StopReason> {
    let (text, matched, range) = crate::emit::emit_class_source_anonymous_return(
        &ast.projection.program.stmts,
        &ast.projection.member,
        ast.projection.current_class.as_deref(),
        &ast.projection.nested_class_members,
        indentation,
        allocation_bci,
        allocation_type,
        source_type,
        methods,
        hidden_outer_argument_bci,
        closing_indent,
        budget,
    )?;
    Ok(if matched {
        range.map(|(start, end)| ClassSourceAnonymousReturn {
            text,
            expression_range: start..end,
        })
    } else {
        None
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClassSourceMethodAstSource {
    pub(crate) program: crate::build::Program,
    pub(crate) member: jarde_reader::model::PhysicalMethodId,
    /// The class this run's body belongs to, in the source spelling (`p.Outer`), when the run
    /// stated one. Every emitter that re-emits this retained AST spells nested type references
    /// against it ([`crate::names::nested_member_reference_spelling`]), so a fragment written by
    /// the class-source adapter and the body of a plain recovery nest the same class the same way.
    pub(crate) current_class: Option<String>,
    /// The member classes that class's own `InnerClasses` attribute states, in the source
    /// spelling: the row set the nested source spelling is gated on.
    pub(crate) nested_class_members: Vec<String>,
    /// The presentation name assigned to each descriptor parameter slot, in descriptor order.
    /// `None` means a reused slot did not have one unambiguous whole-slot name.
    pub(crate) parameter_names: Vec<Option<String>>,
    /// Physical local slot for each descriptor-order formal name.
    pub(crate) parameter_slots: Vec<u16>,
    pub(crate) complete_code: bool,
    pub(crate) has_exception_handlers: bool,
    pub(crate) instruction_count: usize,
    /// Every physical instruction BCI observed in the complete Code attribute. A certificate
    /// compares this exact set to the admitted AST anchors; a matching node count is insufficient.
    pub(crate) instruction_bcis: Vec<u32>,
    /// Decoded invocation targets, tied to their physical instruction BCIs.
    pub(crate) call_targets: Vec<(u32, u8, crate::facts::CallTarget)>,
    pub(crate) anonymous_constructor_initializer_bci:
        Option<ClassSourceAnonymousConstructorInitializer>,
    /// Same-run verified constructor prologue, retained only for generic-call proofs. The public
    /// `RuleDetails` selection may omit this record even though the body and SSA were recovered.
    pub(crate) generic_call_init: Option<crate::init::InitRecord>,
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
pub struct ClassSourceLambdaHelperCandidate {
    pub member: jarde_reader::model::PhysicalMethodId,
    pub helper: jarde_reader::model::PhysicalMethodId,
    pub helper_owner: jarde_reader::model::JvmBytes,
    pub bootstrap_index: u16,
    pub implementation_index: u16,
    pub use_site: u32,
    pub site_cp: u16,
    pub(crate) implementation_kind: u8,
    pub(crate) projection: std::sync::Arc<LambdaHelperProjectionSource>,
}

impl ClassSourceLambdaHelperCandidate {
    /// The implementation MethodHandle reference kind read from this site's bootstrap row.
    pub fn implementation_kind(&self) -> u8 {
        self.implementation_kind
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LambdaHelperProjectionSource {
    pub(crate) program: crate::build::Program,
    pub(crate) facts: crate::facts::RecoveryFacts,
    pub(crate) declaration: Option<crate::declaration::Declaration>,
    pub(crate) member: jarde_reader::model::PhysicalMethodId,
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
    /// Constructor arguments that are direct, unmodified entry parameters with no other SSA use.
    /// One `None` means that argument could not be closed to one such parameter.
    pub argument_parameter_slots: Vec<Option<u16>>,
}

fn anonymous_direct_double_parameter_slot(
    ssa: &SsaTable,
    producer_bci: u32,
    consumer_bci: u32,
) -> Option<u16> {
    let loads: Vec<_> = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .filter(|instruction| instruction.bci() == producer_bci)
        .collect();
    let [load] = loads.as_slice() else {
        return None;
    };
    if !matches!(load.opcode(), 0x18 | 0x26..=0x29) || load.reads().len() != 1 {
        return None;
    }
    let Slot::Local(slot) = load.reads()[0].0 else {
        return None;
    };
    let [(_, value)] = load.writes() else {
        return None;
    };
    if !matches!(ssa.value(*value).def(), Definition::Instruction { bci, .. } if *bci == producer_bci)
        || !matches!(ssa.value(load.reads()[0].1).def(), Definition::Entry { slot: Slot::Local(entry_slot), .. } if *entry_slot == slot)
        || ssa.value(*value).uses().len() != 1
        || ssa.value(*value).uses()[0].bci() != Some(consumer_bci)
    {
        return None;
    }
    Some(slot)
}

/// Parameter slots, rather than rendered text, identify the values in this proof.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericReturnCandidate {
    pub parameters: Vec<(u16, String)>,
    pub value: GenericReturnValue,
}

/// A narrow class-source target selected from a proved method Signature and a complete direct
/// return Code shape. The site is still checked against the same-run bootstrap and AST.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TypedFunctionalTarget {
    pub kind: TypedFunctionalKind,
    pub use_site: u32,
    pub site_cp: u16,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypedFunctionalKind {
    FunctionStringInteger,
    SupplierString,
}

/// Parameter slots, rather than rendered text, identify the values this constructor leaves unused.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericConstructorCandidate {
    pub parameters: Vec<(u16, String)>,
    /// Parameter slots forwarded unchanged, in the selected superclass constructor's argument
    /// order. This is populated only for the exact single-block `aload_0; load*; invokespecial;
    /// return` prologue proved below.
    pub forwarded_parameter_slots: Vec<u16>,
    /// Direct instance-field writes performed after the proved Object() call. A non-empty list
    /// distinguishes the new initialization proof from the older empty/forwarding proof; each
    /// load is tied to one physical write and may not be reused by another consumer.
    pub field_writes: Vec<GenericConstructorFieldWrite>,
    /// The `init@1` record derived from the same run's prologue decision.
    pub init: InitRecord,
}

/// One proved `this.field = parameter` operation in a generic constructor body.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericConstructorFieldWrite {
    pub bci: u32,
    pub owner: String,
    pub name: String,
    pub descriptor: String,
    pub parameter_slot: u16,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericReturnValue {
    /// The body is exactly one effect-free `return;` instruction.
    EmptyVoid,
    /// A complete, straight-line void body whose parameter locals are never reassigned. This is
    /// used only as same-run evidence for a source-level generic parameter projection: the body
    /// remains typed from its physical descriptor while the declaration may name a subtype
    /// variable with the same proved erasure. Slots in this list are physical parameter starts
    /// that neither the SSA instruction reads nor the effects census says the body reads.
    VoidBody {
        unread_parameter_slots: Vec<u16>,
    },
    /// The body is exactly an effect-free `aconst_null; areturn` sequence.
    NullLiteral,
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
    /// Same-run direct creation of the selected static member type.
    StaticMemberCreation {
        target: Box<ProvedStaticMemberTarget>,
        allocation_bci: u32,
        copy_bci: u32,
        constructor_bci: u32,
    },
    /// Exact same-run direct LambdaMetafactory return, including its physical site.
    TypedFunctional {
        target: TypedFunctionalTarget,
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
    current_class: Option<&str>,
    nested_class_members: &[String],
    budget: &mut Budget,
) -> Result<String, StopReason> {
    emit_initializer_value(value, member, current_class, nested_class_members, budget)
}

/// Emits only statement nodes selected by a completed enum suffix certificate.
#[doc(hidden)]
pub fn emit_class_enum_initializer_statements(
    statements: &[crate::ast::Stmt],
    member: &PhysicalMethodId,
    current_class: Option<&str>,
    nested_class_members: &[String],
    budget: &mut Budget,
) -> Result<String, StopReason> {
    crate::emit::emit_class_source_statements(
        statements,
        member,
        current_class,
        nested_class_members,
        2,
        budget,
    )
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
    current_class: Option<&str>,
    nested_class_members: &[String],
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
    crate::emit::emit_class_enum_constructor_statements(
        &statements,
        member,
        current_class,
        nested_class_members,
        budget,
    )
    .map(Some)
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

/// Re-emits one same-run method AST after replacing every independently proved enum switch site.
#[doc(hidden)]
pub fn emit_class_source_enum_switch_group(
    candidates: &[(
        &ClassSourceEnumSwitchCandidate,
        &std::collections::BTreeMap<i64, String>,
    )],
    budget: &mut Budget,
) -> Result<Option<String>, crate::stop::StopReason> {
    use crate::ast::{ExprKind, StmtKind};
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;

    let Some((first, _)) = candidates.first() else {
        return Ok(None);
    };
    let Some(source) = first.projection.as_ref() else {
        return Ok(None);
    };
    let mut targets = BTreeMap::new();
    for (candidate, labels) in candidates {
        let Some(candidate_source) = candidate.projection.as_ref() else {
            return Ok(None);
        };
        if candidate.member != first.member
            || !Arc::ptr_eq(source, candidate_source)
            || targets.insert(candidate.switch_bci, *labels).is_some()
        {
            return Ok(None);
        }
    }

    fn visit(
        statements: &mut [crate::ast::Stmt],
        targets: &BTreeMap<u32, &std::collections::BTreeMap<i64, String>>,
        matched: &mut BTreeSet<u32>,
        budget: &mut Budget,
    ) -> Result<bool, crate::stop::StopReason> {
        for statement in statements {
            match &mut statement.kind {
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    if !visit(then_body, targets, matched, budget)?
                        || !visit(else_body, targets, matched, budget)?
                    {
                        return Ok(false);
                    }
                }
                StmtKind::While { body, .. }
                | StmtKind::For { body, .. }
                | StmtKind::DoWhile { body, .. }
                | StmtKind::Synchronized { body, .. } => {
                    if !visit(body, targets, matched, budget)? {
                        return Ok(false);
                    }
                }
                StmtKind::Try {
                    body,
                    catches,
                    finally_body,
                    ..
                } => {
                    if !visit(body, targets, matched, budget)? {
                        return Ok(false);
                    }
                    for catch in catches {
                        if !visit(&mut catch.body, targets, matched, budget)? {
                            return Ok(false);
                        }
                    }
                    if let Some(finally_body) = finally_body
                        && !visit(finally_body, targets, matched, budget)?
                    {
                        return Ok(false);
                    }
                }
                StmtKind::Switch { value, arms } => {
                    let bci = statement.origin.primary().bci();
                    if let Some(labels) = targets.get(&bci) {
                        if !matched.insert(bci) {
                            return Ok(false);
                        }
                        crate::stop::poll(budget, Some(bci))?;
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
                            return Ok(false);
                        };
                        for arm in arms.iter_mut() {
                            let mut projected = Vec::with_capacity(arm.keys.len());
                            for key in &arm.keys {
                                let Some(label) = labels.get(key) else {
                                    return Ok(false);
                                };
                                projected.push(label.clone());
                            }
                            arm.labels = Some(crate::ast::SwitchLabels::Enum(projected));
                        }
                        *value = receiver;
                    }
                    for arm in arms.iter_mut() {
                        if !visit(&mut arm.body, targets, matched, budget)? {
                            return Ok(false);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(true)
    }

    let mut program = source.program.clone();
    let mut matched = BTreeSet::new();
    if !visit(&mut program.stmts, &targets, &mut matched, budget)? || matched.len() != targets.len()
    {
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

/// The presentation name of every descriptor parameter slot of one retained class-source AST,
/// in descriptor order. `None` entries are slots without one unambiguous whole-slot name.
#[doc(hidden)]
pub fn class_source_parameter_names(ast: &ClassSourceMethodAst) -> Vec<Option<String>> {
    ast.projection.parameter_names.clone()
}

/// One caller-member site edit the class-source lambda-body channel decided (change
/// `recover-lambda-inline-bodies`).
///
/// Every edit is located by the site it applies to — the `invokedynamic` instruction's own BCI and
/// constant-pool entry — so a member with several lambda sites takes all of its edits in one
/// re-emission and no edit can attach itself to a site it was not proved from.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassSourceLambdaSiteEdit {
    /// The companion's single-return body, substituted onto the site's parameters, replaces the
    /// call inside the lambda expression.
    Inline {
        use_site: u32,
        site_cp: u16,
        params: Vec<crate::ast::LambdaParam>,
        body: crate::ast::Expr,
    },
    /// The call inside the lambda keeps its shape and names the companion's renamed member.
    RenameCall {
        use_site: u32,
        site_cp: u16,
        to: String,
    },
}

/// The proved inline plan for one site: the donor body with every parameter read substituted, and
/// the lambda parameters the inlined expression is bound under.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceLambdaInline {
    pub params: Vec<crate::ast::LambdaParam>,
    pub body: crate::ast::Expr,
}

/// Plans one companion-body inline: the helper's single `return` expression, with every parameter
/// read substituted by the call-site argument the presentation already writes there.
///
/// `None` is a stated refusal, not a stop: the companion takes the rename branch instead. What
/// this proof requires, each item a structural fact of the two retained same-run ASTs:
///
/// * the helper's own run is complete, handler-free, not ragged, and exactly one `return`
///   statement whose anchors cover every physical instruction of its body — the body this proof
///   moves is the whole body the class file states;
/// * the caller's run is not ragged and holds exactly one lambda expression at the candidate's
///   own site whose body is the plain companion call the `lambda@1` rule writes;
/// * the companion's declared parameters line up with the call's arguments by position — captured
///   arguments first, then one argument per lambda parameter, each the (possibly cast) lambda
///   parameter itself, so substitution is a rebinding and never a reordering;
/// * the donor holds no node the inline would change the meaning of: no nested lambda (its own
///   sites belong to the omitted member), no conditional, no short-circuit operator, no local
///   write, and no local that is not one of the companion's own parameters — `this` excepted for
///   an instance companion whose site binds the caller's own `this`;
/// * the chosen lambda parameter names — the companion's own, or the site's when the companion's
///   shadow anything in the caller's scope — bind every read the substituted body makes.
///
/// A captured argument is embedded exactly where the call already wrote it, so an effectful
/// capture producer only matters when the donor reads that parameter more than once; that one
/// case is refused here rather than rewritten.
pub fn plan_class_source_lambda_inline(
    candidate: &ClassSourceLambdaHelperCandidate,
    helper_ast: &ClassSourceMethodAst,
    caller_parameter_names: &[Option<String>],
    budget: &mut Budget,
) -> Result<Option<ClassSourceLambdaInline>, crate::stop::StopReason> {
    let caller = &candidate.projection;
    let helper = &helper_ast.projection;
    if caller.program.ragged || helper.program.ragged {
        return Ok(None);
    }
    let [statement] = helper.program.stmts.as_slice() else {
        return Ok(None);
    };
    let crate::ast::StmtKind::Return { value: Some(donor) } = &statement.kind else {
        return Ok(None);
    };
    if !helper.complete_code || helper.has_exception_handlers {
        return Ok(None);
    }
    let helper_nodes = program_node_count(&helper.program);
    let caller_nodes = program_node_count(&caller.program);
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        helper_nodes.saturating_add(caller_nodes),
        Some(candidate.use_site),
    )?;
    crate::stop::poll(budget, Some(candidate.use_site))?;
    if helper_nodes > MAX_LAMBDA_HELPER_AST_NODES || caller_nodes > MAX_LAMBDA_HELPER_AST_NODES {
        return Ok(None);
    }
    if !lambda_helper_instruction_coverage(helper) {
        return Ok(None);
    }
    if helper.parameter_names.iter().any(Option::is_none) {
        return Ok(None);
    }
    let helper_name = String::from_utf8_lossy(&candidate.helper.name.0).to_string();
    let helper_owner = String::from_utf8_lossy(&candidate.helper_owner.0).replace('/', ".");
    let Some(lambda) =
        locate_lambda_expression(&caller.program.stmts, candidate.use_site, candidate.site_cp)
    else {
        return Ok(None);
    };
    let crate::ast::ExprKind::Lambda {
        params: lambda_params,
        body,
    } = &lambda.kind
    else {
        return Ok(None);
    };
    let crate::ast::ExprKind::Call {
        receiver: Some(receiver),
        name,
        args,
    } = &body.kind
    else {
        return Ok(None);
    };
    let receiver_matches = match candidate.implementation_kind {
        6 => matches!(&receiver.kind, crate::ast::ExprKind::Path(owner) if owner == &helper_owner),
        7 => matches!(&receiver.kind, crate::ast::ExprKind::Local(local) if local == "this"),
        _ => false,
    };
    if !receiver_matches
        || name != &helper_name
        || helper.parameter_names.len() != args.len()
        || helper.parameter_names.len() < lambda_params.len()
    {
        return Ok(None);
    }
    let trailing = lambda_params.len();
    let capture_args = &args[..args.len() - trailing];
    let parameter_args = &args[args.len() - trailing..];
    // Each trailing argument must be the lambda's own parameter, possibly behind the casts the
    // site's adaptation proof wrote: substituting it is a rebinding of that one parameter.
    for (arg, param) in parameter_args.iter().zip(lambda_params) {
        let mut inner = arg;
        while let crate::ast::ExprKind::Cast { value, .. } = &inner.kind {
            inner = value;
        }
        if !matches!(&inner.kind, crate::ast::ExprKind::Local(local) if local == &param.name) {
            return Ok(None);
        }
    }
    let companion_names: Vec<String> = helper.parameter_names
        [helper.parameter_names.len() - trailing..]
        .iter()
        .map(|name| name.clone().expect("checked parameter name"))
        .collect();
    if companion_names
        .iter()
        .any(|name| lambda_params.iter().any(|param| &param.name == name))
    {
        // The companion's own names reappearing as site parameter names would make the
        // substitution map ambiguous; the site spelling stays untouched instead.
        return Ok(None);
    }
    // The lambda parameter names the inlined body binds under: the companion's own when they
    // shadow nothing the caller's scope names — javac rejects a lambda parameter that shadows an
    // enclosing local, and the caller's parameters, declared locals and free reads are that
    // scope — and the site's own otherwise; the site's names were derived to be free of exactly
    // those collisions.
    let mut caller_scope: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    caller_scope.extend(caller_parameter_names.iter().flatten().cloned());
    collect_caller_scope_names(&caller.program.stmts, &mut caller_scope);
    for arg in capture_args {
        collect_free_locals(arg, &mut caller_scope);
    }
    let companion_policy = companion_names
        .iter()
        .all(|name| !caller_scope.contains(name));
    let site_policy = lambda_params
        .iter()
        .all(|param| !caller_scope.contains(&param.name));
    let params: Vec<crate::ast::LambdaParam> = if companion_policy {
        lambda_params
            .iter()
            .zip(&companion_names)
            .map(|(param, name)| crate::ast::LambdaParam {
                ty: param.ty.clone(),
                name: name.clone(),
            })
            .collect()
    } else if site_policy {
        lambda_params.clone()
    } else {
        return Ok(None);
    };
    // Every companion parameter binds to the argument the call already writes: the captured
    // expressions verbatim, the trailing ones as the chosen names behind the site's own
    // conversion chain.
    let mut bindings: Vec<(String, crate::ast::Expr)> = helper
        .parameter_names
        .iter()
        .zip(args)
        .map(|(name, arg)| (name.clone().expect("checked parameter name"), arg.clone()))
        .collect();
    let capture_count = bindings.len() - trailing;
    if companion_policy {
        for (index, binding) in bindings.iter_mut().skip(capture_count).enumerate() {
            binding.1 = rebind_site_parameter(
                &binding.1,
                lambda_params[index].name.as_str(),
                params[index].name.as_str(),
                lambda_params[index].ty.clone(),
            );
        }
    }
    // No two parameters may bind the same expression: a duplicated producer would run twice in
    // the inlined body where the call evaluated it once.
    for (index, (_, left)) in bindings.iter().enumerate() {
        if bindings
            .iter()
            .skip(index + 1)
            .any(|(_, right)| left == right)
        {
            return Ok(None);
        }
    }
    let map: std::collections::BTreeMap<String, crate::ast::Expr> = bindings.into_iter().collect();
    // An effectful captured producer may be embedded once; a second read duplicates its effect.
    let mut read_counts = std::collections::BTreeMap::<String, usize>::new();
    count_parameter_reads(donor, &map, &mut read_counts);
    for (name, count) in read_counts {
        if count > 1 && !map.get(&name).is_some_and(|expr| is_pure_read(&expr.kind)) {
            return Ok(None);
        }
    }
    let this_is_bound = candidate.implementation_kind == 7;
    let Some(body) =
        substitute_lambda_donor(donor, &map, this_is_bound, candidate.use_site, budget)?
    else {
        return Ok(None);
    };
    Ok(Some(ClassSourceLambdaInline { params, body }))
}

/// The same expression node with another shape, keeping the origin and the presented type the
/// proof found the node under.
fn rekind(expression: &crate::ast::Expr, kind: crate::ast::ExprKind) -> crate::ast::Expr {
    let mut rebuilt = expression.clone();
    rebuilt.kind = kind;
    rebuilt
}

/// Rebinds one site parameter name to its chosen inlined name inside the cast chain the site's
/// adaptation wrote, preserving the conversions the lambda's own proof produced.
fn rebind_site_parameter(
    arg: &crate::ast::Expr,
    from: &str,
    to: &str,
    declared: crate::ast::Type,
) -> crate::ast::Expr {
    match &arg.kind {
        crate::ast::ExprKind::Cast { ty, value } => rekind(
            arg,
            crate::ast::ExprKind::Cast {
                ty: ty.clone(),
                value: Box::new(rebind_site_parameter(value, from, to, declared)),
            },
        ),
        crate::ast::ExprKind::Local(local) if local == from => crate::ast::Expr::new(
            crate::ast::ExprKind::Local(to.to_string()),
            arg.origin.clone(),
        )
        .presenting(declared),
        _ => arg.clone(),
    }
}

/// Substitutes every parameter read in the donor with its bound expression, refusing the node
/// kinds whose meaning an inline would change (a nested lambda's own sites, control flow, writes)
/// and every local that is not one of the companion's parameters. `this` passes through for an
/// instance companion whose site binds the caller's own `this` — the one receiver shape the plan
/// admitted.
fn substitute_lambda_donor(
    expression: &crate::ast::Expr,
    bindings: &std::collections::BTreeMap<String, crate::ast::Expr>,
    this_is_bound: bool,
    use_site: u32,
    budget: &mut Budget,
) -> Result<Option<crate::ast::Expr>, crate::stop::StopReason> {
    use crate::ast::{BinaryOp, ExprKind};
    crate::stop::poll(budget, Some(use_site))?;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        1,
        Some(use_site),
    )?;
    let rebuilt: Option<ExprKind> = match &expression.kind {
        ExprKind::Local(name) => {
            if this_is_bound && name == "this" {
                Some(expression.kind.clone())
            } else {
                return Ok(bindings.get(name).cloned());
            }
        }
        ExprKind::Integer(_)
        | ExprKind::IntegerConstantName { .. }
        | ExprKind::Boolean(_)
        | ExprKind::Long(_)
        | ExprKind::Float(_)
        | ExprKind::Double(_)
        | ExprKind::Str(_)
        | ExprKind::Null
        | ExprKind::ClassLiteral { .. }
        | ExprKind::Path(_)
        | ExprKind::QualifiedThis { .. } => Some(expression.kind.clone()),
        // Writes, control flow and a nested lambda's own sites are the shapes the rename branch
        // keeps physical; `super` names a member the lambda's scope cannot reach.
        ExprKind::LocalAssign { .. }
        | ExprKind::PostfixUpdate { .. }
        | ExprKind::Conditional { .. }
        | ExprKind::Lambda { .. }
        | ExprKind::Super { .. } => None,
        ExprKind::InstanceOf { value, ty } => {
            let rebuilt =
                substitute_lambda_donor(value, bindings, this_is_bound, use_site, budget)?;
            rebuilt.map(|value| ExprKind::InstanceOf {
                value: Box::new(value),
                ty: ty.clone(),
            })
        }
        ExprKind::Call {
            receiver,
            name,
            args,
        } => {
            let mut rebuilt_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(arg) =
                    substitute_lambda_donor(arg, bindings, this_is_bound, use_site, budget)?
                else {
                    return Ok(None);
                };
                rebuilt_args.push(arg);
            }
            let rebuilt_receiver = match receiver {
                Some(receiver) => Some(Box::new(
                    match substitute_lambda_donor(
                        receiver,
                        bindings,
                        this_is_bound,
                        use_site,
                        budget,
                    )? {
                        Some(receiver) => receiver,
                        None => return Ok(None),
                    },
                )),
                None => None,
            };
            Some(ExprKind::Call {
                receiver: rebuilt_receiver,
                name: name.clone(),
                args: rebuilt_args,
            })
        }
        ExprKind::New {
            ty,
            qualifier,
            member_name,
            diamond,
            args,
        } => {
            let mut rebuilt_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(arg) =
                    substitute_lambda_donor(arg, bindings, this_is_bound, use_site, budget)?
                else {
                    return Ok(None);
                };
                rebuilt_args.push(arg);
            }
            let rebuilt_qualifier = match qualifier {
                Some(qualifier) => Some(Box::new(
                    match substitute_lambda_donor(
                        qualifier,
                        bindings,
                        this_is_bound,
                        use_site,
                        budget,
                    )? {
                        Some(qualifier) => qualifier,
                        None => return Ok(None),
                    },
                )),
                None => None,
            };
            Some(ExprKind::New {
                ty: ty.clone(),
                qualifier: rebuilt_qualifier,
                member_name: member_name.clone(),
                diamond: *diamond,
                args: rebuilt_args,
            })
        }
        ExprKind::MethodReference { qualifier, name } => {
            let rebuilt =
                substitute_lambda_donor(qualifier, bindings, this_is_bound, use_site, budget)?;
            rebuilt.map(|qualifier| ExprKind::MethodReference {
                qualifier: Box::new(qualifier),
                name: name.clone(),
            })
        }
        ExprKind::Field { receiver, name } => {
            let rebuilt =
                substitute_lambda_donor(receiver, bindings, this_is_bound, use_site, budget)?;
            rebuilt.map(|receiver| ExprKind::Field {
                receiver: Box::new(receiver),
                name: name.clone(),
            })
        }
        ExprKind::Index { array, index } => {
            let Some(array) =
                substitute_lambda_donor(array, bindings, this_is_bound, use_site, budget)?
            else {
                return Ok(None);
            };
            let Some(index) =
                substitute_lambda_donor(index, bindings, this_is_bound, use_site, budget)?
            else {
                return Ok(None);
            };
            Some(ExprKind::Index {
                array: Box::new(array),
                index: Box::new(index),
            })
        }
        ExprKind::ArrayLength { array } => {
            let rebuilt =
                substitute_lambda_donor(array, bindings, this_is_bound, use_site, budget)?;
            rebuilt.map(|array| ExprKind::ArrayLength {
                array: Box::new(array),
            })
        }
        ExprKind::NewArray {
            element,
            lengths,
            initializers,
            total_dimensions,
        } => {
            let mut rebuilt_lengths = Vec::with_capacity(lengths.len());
            for length in lengths {
                let Some(length) =
                    substitute_lambda_donor(length, bindings, this_is_bound, use_site, budget)?
                else {
                    return Ok(None);
                };
                rebuilt_lengths.push(length);
            }
            let rebuilt_initializers = match initializers {
                Some(initializers) => {
                    let mut rebuilt = Vec::with_capacity(initializers.len());
                    for initializer in initializers {
                        let Some(initializer) = substitute_lambda_donor(
                            initializer,
                            bindings,
                            this_is_bound,
                            use_site,
                            budget,
                        )?
                        else {
                            return Ok(None);
                        };
                        rebuilt.push(initializer);
                    }
                    Some(rebuilt)
                }
                None => None,
            };
            Some(ExprKind::NewArray {
                element: element.clone(),
                lengths: rebuilt_lengths,
                initializers: rebuilt_initializers,
                total_dimensions: *total_dimensions,
            })
        }
        ExprKind::Binary { op, left, right } => {
            if matches!(op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
                return Ok(None);
            }
            let Some(left) =
                substitute_lambda_donor(left, bindings, this_is_bound, use_site, budget)?
            else {
                return Ok(None);
            };
            let Some(right) =
                substitute_lambda_donor(right, bindings, this_is_bound, use_site, budget)?
            else {
                return Ok(None);
            };
            Some(ExprKind::Binary {
                op: *op,
                left: Box::new(left),
                right: Box::new(right),
            })
        }
        ExprKind::Concat { parts } => {
            let mut rebuilt = Vec::with_capacity(parts.len());
            for part in parts {
                let Some(value) = substitute_lambda_donor(
                    &part.value,
                    bindings,
                    this_is_bound,
                    use_site,
                    budget,
                )?
                else {
                    return Ok(None);
                };
                rebuilt.push(crate::ast::ConcatPart::new(part.parameter.clone(), value));
            }
            Some(ExprKind::Concat { parts: rebuilt })
        }
        ExprKind::Cast { ty, value } => {
            let rebuilt =
                substitute_lambda_donor(value, bindings, this_is_bound, use_site, budget)?;
            rebuilt.map(|value| ExprKind::Cast {
                ty: ty.clone(),
                value: Box::new(value),
            })
        }
        ExprKind::Not { value } => {
            let rebuilt =
                substitute_lambda_donor(value, bindings, this_is_bound, use_site, budget)?;
            rebuilt.map(|value| ExprKind::Not {
                value: Box::new(value),
            })
        }
        ExprKind::Neg { value } => {
            let rebuilt =
                substitute_lambda_donor(value, bindings, this_is_bound, use_site, budget)?;
            rebuilt.map(|value| ExprKind::Neg {
                value: Box::new(value),
            })
        }
    };
    Ok(rebuilt.map(|kind| rekind(expression, kind)))
}

/// Every name the caller's own scope binds: its declared and assigned locals, its loop, resource
/// and catch names, its parameters (supplied by the caller), and every local its expressions read
/// freely. A lambda parameter that shadows any of these is a compile error, so the companion's
/// parameter names are admitted only outside this set.
fn collect_caller_scope_names(
    statements: &[crate::ast::Stmt],
    names: &mut std::collections::BTreeSet<String>,
) {
    use crate::ast::StmtKind;
    for statement in statements {
        match &statement.kind {
            StmtKind::Declare { name, .. }
            | StmtKind::Assign { name, .. }
            | StmtKind::ForEach { name, .. } => {
                names.insert(name.clone());
            }
            _ => {}
        }
        for_each_statement_value_expression(std::slice::from_ref(statement), &mut |expression| {
            collect_free_locals(expression, names);
        });
        match &statement.kind {
            StmtKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_caller_scope_names(then_body, names);
                collect_caller_scope_names(else_body, names);
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::Synchronized { body, .. } => collect_caller_scope_names(body, names),
            StmtKind::For {
                init, update, body, ..
            } => {
                collect_caller_scope_names(std::slice::from_ref(init), names);
                collect_caller_scope_names(std::slice::from_ref(update), names);
                collect_caller_scope_names(body, names);
            }
            StmtKind::ForEach { body, .. } => collect_caller_scope_names(body, names),
            StmtKind::Switch { arms, .. } => {
                for arm in arms {
                    collect_caller_scope_names(&arm.body, names);
                }
            }
            StmtKind::Try {
                resources,
                catches,
                body,
                finally_body,
            } => {
                for resource in resources {
                    names.insert(resource.name.clone());
                }
                for catch in catches {
                    names.insert(catch.name.clone());
                    collect_caller_scope_names(&catch.body, names);
                }
                collect_caller_scope_names(body, names);
                if let Some(body) = finally_body {
                    collect_caller_scope_names(body, names);
                }
            }
            _ => {}
        }
    }
}

/// The locals one expression reads — the names a lambda parameter must not shadow. A nested
/// lambda's own parameters shadow only inside that lambda, so they are not free names here.
fn collect_free_locals(
    expression: &crate::ast::Expr,
    names: &mut std::collections::BTreeSet<String>,
) {
    use crate::ast::ExprKind;
    match &expression.kind {
        ExprKind::Local(name) => {
            names.insert(name.clone());
        }
        ExprKind::Integer(_)
        | ExprKind::IntegerConstantName { .. }
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
        ExprKind::LocalAssign { name, value, .. } => {
            names.insert(name.clone());
            collect_free_locals(value, names);
        }
        ExprKind::InstanceOf { value, .. } => collect_free_locals(value, names),
        ExprKind::Call { receiver, args, .. } => {
            if let Some(receiver) = receiver {
                collect_free_locals(receiver, names);
            }
            for arg in args {
                collect_free_locals(arg, names);
            }
        }
        ExprKind::New {
            qualifier, args, ..
        } => {
            if let Some(qualifier) = qualifier {
                collect_free_locals(qualifier, names);
            }
            for arg in args {
                collect_free_locals(arg, names);
            }
        }
        ExprKind::Lambda { params, body } => {
            let shadowed: std::collections::BTreeSet<String> =
                params.iter().map(|param| param.name.clone()).collect();
            let mut inner = std::collections::BTreeSet::new();
            collect_free_locals(body, &mut inner);
            names.extend(inner.difference(&shadowed).cloned());
        }
        ExprKind::MethodReference { qualifier, .. } => collect_free_locals(qualifier, names),
        ExprKind::Field { receiver, .. } => collect_free_locals(receiver, names),
        ExprKind::ArrayLength { array } => collect_free_locals(array, names),
        ExprKind::Index { array, index } => {
            collect_free_locals(array, names);
            collect_free_locals(index, names);
        }
        ExprKind::PostfixUpdate { target, .. } => collect_free_locals(target, names),
        ExprKind::NewArray {
            lengths,
            initializers,
            ..
        } => {
            for length in lengths {
                collect_free_locals(length, names);
            }
            for initializer in initializers.iter().flatten() {
                collect_free_locals(initializer, names);
            }
        }
        ExprKind::Binary { left, right, .. } => {
            collect_free_locals(left, names);
            collect_free_locals(right, names);
        }
        ExprKind::Conditional {
            test,
            when_true,
            when_false,
        } => {
            collect_free_locals(test, names);
            collect_free_locals(when_true, names);
            collect_free_locals(when_false, names);
        }
        ExprKind::Concat { parts } => {
            for part in parts {
                collect_free_locals(&part.value, names);
            }
        }
        ExprKind::Cast { value, .. } => collect_free_locals(value, names),
        ExprKind::Not { value } | ExprKind::Neg { value } => collect_free_locals(value, names),
    }
}

/// Whether an expression only reads a value the enclosing scope already holds — embedding it any
/// number of times evaluates nothing new.
fn is_pure_read(kind: &crate::ast::ExprKind) -> bool {
    use crate::ast::ExprKind;
    matches!(
        kind,
        ExprKind::Local(_)
            | ExprKind::Integer(_)
            | ExprKind::Boolean(_)
            | ExprKind::Long(_)
            | ExprKind::Float(_)
            | ExprKind::Double(_)
            | ExprKind::Str(_)
            | ExprKind::Null
            | ExprKind::Path(_)
            | ExprKind::QualifiedThis { .. }
    )
}

/// How many times the donor reads each bound parameter.
fn count_parameter_reads(
    expression: &crate::ast::Expr,
    bindings: &std::collections::BTreeMap<String, crate::ast::Expr>,
    counts: &mut std::collections::BTreeMap<String, usize>,
) {
    use crate::ast::ExprKind;
    if let ExprKind::Local(name) = &expression.kind
        && bindings.contains_key(name)
    {
        *counts.entry(name.clone()).or_default() += 1;
    }
    match &expression.kind {
        ExprKind::Local(_)
        | ExprKind::Integer(_)
        | ExprKind::IntegerConstantName { .. }
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
        ExprKind::LocalAssign { value, .. } => {
            count_parameter_reads(value, bindings, counts);
        }
        ExprKind::InstanceOf { value, .. } => count_parameter_reads(value, bindings, counts),
        ExprKind::Call { receiver, args, .. } => {
            if let Some(receiver) = receiver {
                count_parameter_reads(receiver, bindings, counts);
            }
            for arg in args {
                count_parameter_reads(arg, bindings, counts);
            }
        }
        ExprKind::New {
            qualifier, args, ..
        } => {
            if let Some(qualifier) = qualifier {
                count_parameter_reads(qualifier, bindings, counts);
            }
            for arg in args {
                count_parameter_reads(arg, bindings, counts);
            }
        }
        ExprKind::Lambda { body, .. } => count_parameter_reads(body, bindings, counts),
        ExprKind::MethodReference { qualifier, .. } => {
            count_parameter_reads(qualifier, bindings, counts)
        }
        ExprKind::Field { receiver, .. } => count_parameter_reads(receiver, bindings, counts),
        ExprKind::ArrayLength { array } => count_parameter_reads(array, bindings, counts),
        ExprKind::Index { array, index } => {
            count_parameter_reads(array, bindings, counts);
            count_parameter_reads(index, bindings, counts);
        }
        ExprKind::PostfixUpdate { target, .. } => count_parameter_reads(target, bindings, counts),
        ExprKind::NewArray {
            lengths,
            initializers,
            ..
        } => {
            for length in lengths {
                count_parameter_reads(length, bindings, counts);
            }
            for initializer in initializers.iter().flatten() {
                count_parameter_reads(initializer, bindings, counts);
            }
        }
        ExprKind::Binary { left, right, .. } => {
            count_parameter_reads(left, bindings, counts);
            count_parameter_reads(right, bindings, counts);
        }
        ExprKind::Conditional {
            test,
            when_true,
            when_false,
        } => {
            count_parameter_reads(test, bindings, counts);
            count_parameter_reads(when_true, bindings, counts);
            count_parameter_reads(when_false, bindings, counts);
        }
        ExprKind::Concat { parts } => {
            for part in parts {
                count_parameter_reads(&part.value, bindings, counts);
            }
        }
        ExprKind::Cast { value, .. } => count_parameter_reads(value, bindings, counts),
        ExprKind::Not { value } | ExprKind::Neg { value } => {
            count_parameter_reads(value, bindings, counts)
        }
    }
}

/// Walks only the expressions a statement holds directly — its own values, conditions and
/// arguments — without descending into nested expressions. Scope collection uses this so a
/// nested lambda's own parameters never leak into the caller's scope as free names.
fn for_each_statement_value_expression<'a>(
    statements: &'a [crate::ast::Stmt],
    visit: &mut dyn FnMut(&'a crate::ast::Expr),
) {
    use crate::ast::StmtKind;
    for statement in statements {
        match &statement.kind {
            StmtKind::Declare { value, .. } => {
                if let Some(value) = value {
                    visit(value);
                }
            }
            StmtKind::Assign { value, .. } | StmtKind::Expr(value) | StmtKind::Throw { value } => {
                visit(value)
            }
            StmtKind::Assert { cond, message } => {
                visit(cond);
                if let Some(message) = message {
                    visit(message);
                }
            }
            StmtKind::FieldAssign {
                receiver, value, ..
            } => {
                if let Some(receiver) = receiver {
                    visit(receiver);
                }
                visit(value);
            }
            StmtKind::IndexAssign {
                array,
                index,
                value,
                ..
            } => {
                visit(array);
                visit(index);
                visit(value);
            }
            StmtKind::ConstructorCall { args, .. } => {
                for arg in args {
                    visit(arg);
                }
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    visit(value);
                }
            }
            StmtKind::Break { .. } | StmtKind::Continue { .. } | StmtKind::Fallback { .. } => {}
            StmtKind::If { cond, .. } => visit(cond),
            StmtKind::While { cond, .. } | StmtKind::DoWhile { cond, .. } => visit(cond),
            StmtKind::For {
                init, cond, update, ..
            } => {
                for_each_statement_value_expression(std::slice::from_ref(init), visit);
                visit(cond);
                for_each_statement_value_expression(std::slice::from_ref(update), visit);
            }
            StmtKind::ForEach { iterable, .. } => visit(iterable),
            StmtKind::Switch { value, .. } => visit(value),
            StmtKind::Try { resources, .. } => {
                for resource in resources {
                    visit(&resource.value);
                }
            }
            StmtKind::Synchronized { lock, .. } => visit(lock),
        }
    }
}

/// Locates the lambda expression one site wrote: its primary anchor is the `invokedynamic`
/// instruction's own BCI and constant-pool entry, which no other node claims as its primary.
fn locate_lambda_expression<'a>(
    statements: &'a [crate::ast::Stmt],
    use_site: u32,
    site_cp: u16,
) -> Option<&'a crate::ast::Expr> {
    let mut found = None;
    for_each_statement_expression(statements, &mut |expression| {
        if found.is_none()
            && matches!(&expression.kind, crate::ast::ExprKind::Lambda { .. })
            && expression.origin.primary().bci() == use_site
            && expression.origin.primary().cp() == Some(site_cp)
        {
            found = Some(expression);
        }
    });
    found
}

/// Applies every decided edit to the caller's retained program, returning the re-emitted body text
/// beside an unmodified re-emission of the same program, and the re-emitted body's own source map.
///
/// The unmodified emission is the adapter's own safety check: the member's current text must
/// still be exactly what this same-run AST writes — a member another projection already rewrote
/// is not a member this channel may re-emit from the same AST, and the adapter keeps such a
/// helper's physical presentation instead.
///
/// The map is the edited program's own anchor table, replayed over the edited emission with the
/// same formatter: a span a later consumer re-spells inside this body can still name the physical
/// instruction it came from, and a donor node spliced in by an inline edit keeps the donor's own
/// anchors. The map is enrichment, never a precondition: a budget that stops the replay leaves the
/// committed texts standing and hands out an empty table, and a caller that needs every span
/// anchors nothing through it.
pub fn emit_class_source_lambda_member(
    candidate: &ClassSourceLambdaHelperCandidate,
    edits: &[ClassSourceLambdaSiteEdit],
    budget: &mut Budget,
) -> Result<Option<(String, String, SourceMap)>, crate::stop::StopReason> {
    let caller = &candidate.projection;
    if caller.program.ragged || edits.is_empty() {
        return Ok(None);
    }
    let unmodified_emitted = crate::emit::emit(
        &caller.program.stmts,
        &caller.facts,
        caller.declaration.as_ref(),
        Some(&caller.member),
        budget,
    )?;
    let unmodified = unmodified_emitted.text;
    let mut program = caller.program.clone();
    let mut applied = 0;
    for edit in edits {
        applied += usize::from(apply_lambda_site_edit(&mut program.stmts, edit));
    }
    if applied != edits.len() {
        return Ok(None);
    }
    let edited_emitted = crate::emit::emit(
        &program.stmts,
        &caller.facts,
        caller.declaration.as_ref(),
        Some(&caller.member),
        budget,
    )?;
    let edited_map = emit_source_map(
        &program.stmts,
        &caller.facts,
        caller.declaration.as_ref(),
        Some(&caller.member),
        SegmentPublication::Whole,
        &edited_emitted,
        &mut EvidencePhase::new(),
        budget,
    )
    .map(|(map, _)| map)
    .unwrap_or_default();
    Ok(Some((unmodified, edited_emitted.text, edited_map)))
}

/// Applies one edit to the lambda expression whose site matches, reporting whether it did.
fn apply_lambda_site_edit(
    statements: &mut [crate::ast::Stmt],
    edit: &ClassSourceLambdaSiteEdit,
) -> bool {
    let mut applied = false;
    for_each_statement_expression_mut(statements, &mut |expression| {
        if applied
            || !matches!(&expression.kind, crate::ast::ExprKind::Lambda { .. })
            || expression.origin.primary().bci() != edit_site(edit)
            || expression.origin.primary().cp() != Some(edit_cp(edit))
        {
            return;
        }
        let crate::ast::ExprKind::Lambda { params, body } = &mut expression.kind else {
            return;
        };
        match edit {
            ClassSourceLambdaSiteEdit::Inline {
                params: new_params,
                body: new_body,
                ..
            } => {
                *params = new_params.clone();
                *body = Box::new(new_body.clone());
                applied = true;
            }
            ClassSourceLambdaSiteEdit::RenameCall { to, .. } => {
                if let crate::ast::ExprKind::Call { name, .. } = &mut body.kind {
                    *name = to.clone();
                    applied = true;
                }
            }
        }
    });
    applied
}

fn edit_site(edit: &ClassSourceLambdaSiteEdit) -> u32 {
    match edit {
        ClassSourceLambdaSiteEdit::Inline { use_site, .. }
        | ClassSourceLambdaSiteEdit::RenameCall { use_site, .. } => *use_site,
    }
}

fn edit_cp(edit: &ClassSourceLambdaSiteEdit) -> u16 {
    match edit {
        ClassSourceLambdaSiteEdit::Inline { site_cp, .. }
        | ClassSourceLambdaSiteEdit::RenameCall { site_cp, .. } => *site_cp,
    }
}

/// Walks every expression of every statement, in a stable order, without mutating anything.
fn for_each_statement_expression<'a>(
    statements: &'a [crate::ast::Stmt],
    visit: &mut dyn FnMut(&'a crate::ast::Expr),
) {
    use crate::ast::StmtKind;
    for statement in statements {
        match &statement.kind {
            StmtKind::Declare { value, .. } => {
                if let Some(value) = value {
                    for_each_expression(value, visit);
                }
            }
            StmtKind::Assign { value, .. } | StmtKind::Expr(value) | StmtKind::Throw { value } => {
                for_each_expression(value, visit)
            }
            StmtKind::Assert { cond, message } => {
                for_each_expression(cond, visit);
                if let Some(message) = message {
                    for_each_expression(message, visit);
                }
            }
            StmtKind::FieldAssign {
                receiver, value, ..
            } => {
                if let Some(receiver) = receiver {
                    for_each_expression(receiver, visit);
                }
                for_each_expression(value, visit);
            }
            StmtKind::IndexAssign {
                array,
                index,
                value,
                ..
            } => {
                for_each_expression(array, visit);
                for_each_expression(index, visit);
                for_each_expression(value, visit);
            }
            StmtKind::ConstructorCall { args, .. } => {
                for arg in args {
                    for_each_expression(arg, visit);
                }
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    for_each_expression(value, visit);
                }
            }
            StmtKind::Break { .. } | StmtKind::Continue { .. } | StmtKind::Fallback { .. } => {}
            StmtKind::If {
                cond,
                then_body,
                else_body,
            } => {
                for_each_expression(cond, visit);
                for_each_statement_expression(then_body, visit);
                for_each_statement_expression(else_body, visit);
            }
            StmtKind::While { cond, body, .. } | StmtKind::DoWhile { cond, body, .. } => {
                for_each_expression(cond, visit);
                for_each_statement_expression(body, visit);
            }
            StmtKind::For {
                init,
                cond,
                update,
                body,
                ..
            } => {
                for_each_statement_expression(std::slice::from_ref(init), visit);
                for_each_expression(cond, visit);
                for_each_statement_expression(std::slice::from_ref(update), visit);
                for_each_statement_expression(body, visit);
            }
            StmtKind::ForEach { iterable, body, .. } => {
                for_each_expression(iterable, visit);
                for_each_statement_expression(body, visit);
            }
            StmtKind::Switch { value, arms } => {
                for_each_expression(value, visit);
                for arm in arms {
                    for_each_statement_expression(&arm.body, visit);
                }
            }
            StmtKind::Try {
                resources,
                catches,
                body,
                finally_body,
            } => {
                for resource in resources {
                    for_each_expression(&resource.value, visit);
                }
                for catch in catches {
                    for_each_statement_expression(&catch.body, visit);
                }
                for_each_statement_expression(body, visit);
                if let Some(body) = finally_body {
                    for_each_statement_expression(body, visit);
                }
            }
            StmtKind::Synchronized { lock, body } => {
                for_each_expression(lock, visit);
                for_each_statement_expression(body, visit);
            }
        }
    }
}

/// Walks one expression and every expression inside it, without mutating anything.
fn for_each_expression<'a>(
    expression: &'a crate::ast::Expr,
    visit: &mut dyn FnMut(&'a crate::ast::Expr),
) {
    use crate::ast::ExprKind;
    visit(expression);
    match &expression.kind {
        ExprKind::Local(_)
        | ExprKind::Integer(_)
        | ExprKind::IntegerConstantName { .. }
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
        ExprKind::LocalAssign { value, .. } => for_each_expression(value, visit),
        ExprKind::InstanceOf { value, .. } => for_each_expression(value, visit),
        ExprKind::Call { receiver, args, .. } => {
            if let Some(receiver) = receiver {
                for_each_expression(receiver, visit);
            }
            for arg in args {
                for_each_expression(arg, visit);
            }
        }
        ExprKind::New {
            qualifier, args, ..
        } => {
            if let Some(qualifier) = qualifier {
                for_each_expression(qualifier, visit);
            }
            for arg in args {
                for_each_expression(arg, visit);
            }
        }
        ExprKind::Lambda { body, .. } => for_each_expression(body, visit),
        ExprKind::MethodReference { qualifier, .. } => for_each_expression(qualifier, visit),
        ExprKind::Field { receiver, .. } => for_each_expression(receiver, visit),
        ExprKind::ArrayLength { array } => for_each_expression(array, visit),
        ExprKind::Index { array, index } => {
            for_each_expression(array, visit);
            for_each_expression(index, visit);
        }
        ExprKind::PostfixUpdate { target, .. } => for_each_expression(target, visit),
        ExprKind::NewArray {
            lengths,
            initializers,
            ..
        } => {
            for length in lengths {
                for_each_expression(length, visit);
            }
            for initializer in initializers.iter().flatten() {
                for_each_expression(initializer, visit);
            }
        }
        ExprKind::Binary { left, right, .. } => {
            for_each_expression(left, visit);
            for_each_expression(right, visit);
        }
        ExprKind::Conditional {
            test,
            when_true,
            when_false,
        } => {
            for_each_expression(test, visit);
            for_each_expression(when_true, visit);
            for_each_expression(when_false, visit);
        }
        ExprKind::Concat { parts } => {
            for part in parts {
                for_each_expression(&part.value, visit);
            }
        }
        ExprKind::Cast { value, .. } => for_each_expression(value, visit),
        ExprKind::Not { value } | ExprKind::Neg { value } => for_each_expression(value, visit),
    }
}

/// Walks every expression of every statement, mutably, in the same stable order as the read-only
/// walk so a located site and an edited site are the same node.
fn for_each_statement_expression_mut(
    statements: &mut [crate::ast::Stmt],
    visit: &mut dyn FnMut(&mut crate::ast::Expr),
) {
    use crate::ast::StmtKind;
    for statement in statements {
        match &mut statement.kind {
            StmtKind::Declare { value, .. } => {
                if let Some(value) = value {
                    for_each_expression_mut(value, visit);
                }
            }
            StmtKind::Assign { value, .. } | StmtKind::Expr(value) | StmtKind::Throw { value } => {
                for_each_expression_mut(value, visit)
            }
            StmtKind::Assert { cond, message } => {
                for_each_expression_mut(cond, visit);
                if let Some(message) = message {
                    for_each_expression_mut(message, visit);
                }
            }
            StmtKind::FieldAssign {
                receiver, value, ..
            } => {
                if let Some(receiver) = receiver {
                    for_each_expression_mut(receiver, visit);
                }
                for_each_expression_mut(value, visit);
            }
            StmtKind::IndexAssign {
                array,
                index,
                value,
                ..
            } => {
                for_each_expression_mut(array, visit);
                for_each_expression_mut(index, visit);
                for_each_expression_mut(value, visit);
            }
            StmtKind::ConstructorCall { args, .. } => {
                for arg in args {
                    for_each_expression_mut(arg, visit);
                }
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    for_each_expression_mut(value, visit);
                }
            }
            StmtKind::Break { .. } | StmtKind::Continue { .. } | StmtKind::Fallback { .. } => {}
            StmtKind::If {
                cond,
                then_body,
                else_body,
            } => {
                for_each_expression_mut(cond, visit);
                for_each_statement_expression_mut(then_body, visit);
                for_each_statement_expression_mut(else_body, visit);
            }
            StmtKind::While { cond, body, .. } | StmtKind::DoWhile { cond, body, .. } => {
                for_each_expression_mut(cond, visit);
                for_each_statement_expression_mut(body, visit);
            }
            StmtKind::For {
                init,
                cond,
                update,
                body,
                ..
            } => {
                for_each_statement_expression_mut(std::slice::from_mut(init), visit);
                for_each_expression_mut(cond, visit);
                for_each_statement_expression_mut(std::slice::from_mut(update), visit);
                for_each_statement_expression_mut(body, visit);
            }
            StmtKind::ForEach { iterable, body, .. } => {
                for_each_expression_mut(iterable, visit);
                for_each_statement_expression_mut(body, visit);
            }
            StmtKind::Switch { value, arms } => {
                for_each_expression_mut(value, visit);
                for arm in arms {
                    for_each_statement_expression_mut(&mut arm.body, visit);
                }
            }
            StmtKind::Try {
                resources,
                catches,
                body,
                finally_body,
            } => {
                for resource in resources {
                    for_each_expression_mut(&mut resource.value, visit);
                }
                for catch in catches {
                    for_each_statement_expression_mut(&mut catch.body, visit);
                }
                for_each_statement_expression_mut(body, visit);
                if let Some(body) = finally_body {
                    for_each_statement_expression_mut(body, visit);
                }
            }
            StmtKind::Synchronized { lock, body } => {
                for_each_expression_mut(lock, visit);
                for_each_statement_expression_mut(body, visit);
            }
        }
    }
}

/// Walks one expression and every expression inside it, mutably.
fn for_each_expression_mut(
    expression: &mut crate::ast::Expr,
    visit: &mut dyn FnMut(&mut crate::ast::Expr),
) {
    use crate::ast::ExprKind;
    visit(expression);
    match &mut expression.kind {
        ExprKind::Local(_)
        | ExprKind::Integer(_)
        | ExprKind::IntegerConstantName { .. }
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
        ExprKind::LocalAssign { value, .. } => for_each_expression_mut(value, visit),
        ExprKind::InstanceOf { value, .. } => for_each_expression_mut(value, visit),
        ExprKind::Call { receiver, args, .. } => {
            if let Some(receiver) = receiver {
                for_each_expression_mut(receiver, visit);
            }
            for arg in args {
                for_each_expression_mut(arg, visit);
            }
        }
        ExprKind::New {
            qualifier, args, ..
        } => {
            if let Some(qualifier) = qualifier {
                for_each_expression_mut(qualifier, visit);
            }
            for arg in args {
                for_each_expression_mut(arg, visit);
            }
        }
        ExprKind::Lambda { body, .. } => for_each_expression_mut(body, visit),
        ExprKind::MethodReference { qualifier, .. } => for_each_expression_mut(qualifier, visit),
        ExprKind::Field { receiver, .. } => for_each_expression_mut(receiver, visit),
        ExprKind::ArrayLength { array } => for_each_expression_mut(array, visit),
        ExprKind::Index { array, index } => {
            for_each_expression_mut(array, visit);
            for_each_expression_mut(index, visit);
        }
        ExprKind::PostfixUpdate { target, .. } => for_each_expression_mut(target, visit),
        ExprKind::NewArray {
            lengths,
            initializers,
            ..
        } => {
            for length in lengths {
                for_each_expression_mut(length, visit);
            }
            for initializer in initializers.iter_mut().flatten() {
                for_each_expression_mut(initializer, visit);
            }
        }
        ExprKind::Binary { left, right, .. } => {
            for_each_expression_mut(left, visit);
            for_each_expression_mut(right, visit);
        }
        ExprKind::Conditional {
            test,
            when_true,
            when_false,
        } => {
            for_each_expression_mut(test, visit);
            for_each_expression_mut(when_true, visit);
            for_each_expression_mut(when_false, visit);
        }
        ExprKind::Concat { parts } => {
            for part in parts {
                for_each_expression_mut(&mut part.value, visit);
            }
        }
        ExprKind::Cast { value, .. } => for_each_expression_mut(value, visit),
        ExprKind::Not { value } | ExprKind::Neg { value } => for_each_expression_mut(value, visit),
    }
}

/// Recursive walks over a retained companion stay safely shallow for this proof.
const MAX_LAMBDA_HELPER_AST_NODES: u64 = 256;

/// A count of AST nodes can coincide with Code instructions while dropping an effect or opcode.
/// The lambda proof therefore requires exact set equality between physical BCIs and every anchor
/// claimed by the complete one-return AST.
fn lambda_helper_instruction_coverage(helper: &ClassSourceMethodAstSource) -> bool {
    let [statement] = helper.program.stmts.as_slice() else {
        return false;
    };
    let crate::ast::StmtKind::Return {
        value: Some(expression),
    } = &statement.kind
    else {
        return false;
    };
    let mut ast_anchors = std::collections::BTreeSet::new();
    ast_anchors.extend(statement.origin.bcis());
    collect_expression_anchors(expression, &mut ast_anchors);
    let physical_bcis: std::collections::BTreeSet<_> =
        helper.instruction_bcis.iter().copied().collect();
    !physical_bcis.is_empty()
        && physical_bcis.len() == helper.instruction_bcis.len()
        && physical_bcis == ast_anchors
        && helper.instruction_bcis.len() == helper.instruction_count
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
            static_member_target: None,
            typed_functional_target: None,
            interface_super_calls: &[],
            reference_overload_calls: &[],
            snapshot_hierarchy_widenings: &[],
            superclass_field_writes: &[],
            captured_outer_reads: &[],
            outer_super_calls: &[],
            pool_spelled_members: false,
            evidence: RecoveryEvidenceRequest::essential(),
            subject: None,
        }
    }

    /// The same request, under a presentation that keeps member classes pool-spelled: the flag
    /// that makes the build refuse structural-reflection reads over such literals (see the field's
    /// own contract).
    pub fn with_pool_spelled_members(mut self, pool_spelled: bool) -> Self {
        self.pool_spelled_members = pool_spelled;
        self
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

    #[doc(hidden)]
    pub fn with_static_member_target(mut self, target: &'a ProvedStaticMemberTarget) -> Self {
        self.static_member_target = Some(target);
        self
    }

    #[doc(hidden)]
    pub fn with_typed_functional_target(mut self, target: TypedFunctionalTarget) -> Self {
        self.typed_functional_target = Some(target);
        self
    }

    /// Supply only interface-special targets proved against this request's selected environment.
    pub fn with_interface_super_calls(mut self, calls: &'a [ProvedInterfaceSuperCall]) -> Self {
        self.interface_super_calls = calls;
        self
    }

    /// Supply only invocation sites proved against the selected source hierarchy.
    pub fn with_reference_overload_calls(
        mut self,
        calls: &'a [ProvedReferenceOverloadCall],
    ) -> Self {
        self.reference_overload_calls = calls;
        self
    }

    /// Supply only invocation arguments proved against this snapshot's own class-file headers.
    pub fn with_snapshot_hierarchy_widenings(
        mut self,
        widenings: &'a [ProvedSnapshotHierarchyWidening],
    ) -> Self {
        self.snapshot_hierarchy_widenings = widenings;
        self
    }

    /// Supply only exact owner and declaration proofs from the selected source environment.
    pub fn with_superclass_field_writes(
        mut self,
        writes: &'a [ProvedSuperclassFieldWrite],
    ) -> Self {
        self.superclass_field_writes = writes;
        self
    }

    /// Supply only the exact child reads closed by the selected family's capture certificate.
    pub fn with_captured_outer_reads(mut self, reads: &'a [ProvedCapturedOuterRead]) -> Self {
        self.captured_outer_reads = reads;
        self
    }

    /// Supply only call sites closed by the selected family's outer-super bridge proof.
    pub fn with_outer_super_calls(mut self, calls: &'a [ProvedOuterSuperCall]) -> Self {
        self.outer_super_calls = calls;
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
    /// The same naming table's whole parameter spellings for a class-source declaration.
    /// This handoff is not serialized as recovery evidence.
    #[serde(skip)]
    pub parameter_names: Vec<Option<String>>,
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
        request, budget, None, None, None, None, None, None, None, None, None, None, None, None,
        None, false, false, true,
    )
}

/// Collect unused physical parameter slots from one complete straight-line void body. Code, SSA
/// and effects must agree on every instruction and local access before a slot can be called unread.
fn generic_void_unread_parameter_slots(
    code: &jarde_reader::classfile::MethodCodeFacts,
    ssa: &SsaTable,
    parameter_slots: &std::collections::BTreeSet<u16>,
    budget: &mut Budget,
) -> Result<Option<Vec<u16>>, StopReason> {
    let Some(block) = ssa.blocks().first() else {
        return Ok(None);
    };
    let instructions = block.instructions();
    let effects = ssa.effects().instructions();
    let code_instructions = &code.instructions;
    let code_operands = code.operands();
    if ssa.blocks().len() != 1
        || !ssa.phis().is_empty()
        || code.stopped_at.is_some()
        || code.exception_handler_count != 0
        || !code.exception_handlers.is_empty()
        || instructions.len() != code_instructions.len()
        || code_operands.len() != code_instructions.len()
        || effects.len() != code_instructions.len()
    {
        return Ok(None);
    }

    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
        u64::try_from(code_instructions.len()).unwrap_or(u64::MAX),
        None,
    )?;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        u64::try_from(instructions.len().saturating_add(effects.len())).unwrap_or(u64::MAX),
        None,
    )?;
    let mut local_reads = std::collections::BTreeSet::new();
    let mut parameter_writes = std::collections::BTreeSet::new();
    for (((raw, operands), instruction), effect) in code_instructions
        .iter()
        .zip(code_operands)
        .zip(instructions)
        .zip(effects)
    {
        crate::stop::poll(budget, Some(raw.bci))?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
            u64::try_from(
                instruction
                    .reads()
                    .len()
                    .saturating_add(instruction.writes().len())
                    .saturating_add(effect.locals_read().len())
                    .saturating_add(effect.locals_written().len())
                    .saturating_add(effect.handlers().len()),
            )
            .unwrap_or(u64::MAX),
            Some(raw.bci),
        )?;
        let raw_opcode_matches = if raw.opcode == 0xc4 {
            operands.local.is_some_and(|local| local.wide)
        } else {
            raw.opcode == operands.effective_opcode
                && operands.local.is_none_or(|local| !local.wide)
        };
        if raw.bci != instruction.bci()
            || raw.bci != effect.bci()
            || effect.block() != block.block()
            || !raw_opcode_matches
            || operands.effective_opcode != instruction.opcode()
            || operands.effective_opcode != effect.opcode()
            || !effect.handlers().is_empty()
        {
            return Ok(None);
        }

        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            u64::try_from(instruction.reads().len()).unwrap_or(u64::MAX),
            Some(raw.bci),
        )?;
        let ssa_reads = instruction
            .reads()
            .iter()
            .filter_map(|(slot, _)| match slot {
                Slot::Local(index) => Some(*index),
                Slot::Stack(_) => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            u64::try_from(effect.locals_read().len()).unwrap_or(u64::MAX),
            Some(raw.bci),
        )?;
        let effect_reads = effect
            .locals_read()
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            u64::try_from(instruction.writes().len()).unwrap_or(u64::MAX),
            Some(raw.bci),
        )?;
        let ssa_writes = instruction
            .writes()
            .iter()
            .filter_map(|(slot, _)| match slot {
                Slot::Local(index) => Some(*index),
                Slot::Stack(_) => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            u64::try_from(effect.locals_written().len()).unwrap_or(u64::MAX),
            Some(raw.bci),
        )?;
        let effect_writes = effect
            .locals_written()
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        if ssa_reads != effect_reads || ssa_writes != effect_writes {
            return Ok(None);
        }
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            u64::try_from(ssa_reads.len()).unwrap_or(u64::MAX),
            Some(raw.bci),
        )?;
        local_reads.extend(ssa_reads);
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            u64::try_from(ssa_writes.len()).unwrap_or(u64::MAX),
            Some(raw.bci),
        )?;
        parameter_writes.extend(ssa_writes.intersection(parameter_slots).copied());
    }
    if !parameter_writes.is_empty() {
        return Ok(None);
    }

    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        u64::try_from(parameter_slots.len()).unwrap_or(u64::MAX),
        None,
    )?;
    let mut unread = Vec::with_capacity(parameter_slots.len());
    for slot in parameter_slots {
        crate::stop::poll(budget, None)?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
            1,
            None,
        )?;
        if !local_reads.contains(slot) {
            unread.push(*slot);
        }
    }
    Ok(Some(unread))
}

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
    if program.ragged || program.stmts.is_empty() {
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
        return Ok(Some(GenericReturnCandidate {
            parameters,
            value: GenericReturnValue::EmptyVoid,
        }));
    }
    // This candidate records a complete straight-line void body and unchanged parameter locals.
    // Its unread-slot list is a separate physical fact: declaration projection may use it for a
    // parameter whose Signature is parameterized by a class variable, while read parameters still
    // need the existing narrower source-assignability proof.
    if matches!(
        program.stmts.last().map(|statement| &statement.kind),
        Some(StmtKind::Return { value: None })
    ) && program.statements == program.stmts.len()
        && program.stmts.len() > 1
        && code.stopped_at.is_none()
        && code.exception_handler_count == 0
        && code.exception_handlers.is_empty()
        && ssa.blocks().len() == 1
        && ssa.phis().is_empty()
        && !program.ragged
        && ssa.blocks()[0].instructions().len() == code.instructions.len()
        && ssa.effects().instructions().len() == code.instructions.len()
    {
        crate::stop::poll(budget, None)?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            u64::try_from(parameter_types.len()).unwrap_or(u64::MAX),
            None,
        )?;
        let parameter_slots: std::collections::BTreeSet<_> =
            parameter_types.keys().copied().collect();
        let Some(unread_parameter_slots) =
            generic_void_unread_parameter_slots(code, ssa, &parameter_slots, budget)?
        else {
            return Ok(None);
        };
        let mut parameters = Vec::with_capacity(parameter_types.len());
        for slot in parameter_types.keys() {
            crate::stop::poll(budget, None)?;
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
        return Ok(Some(GenericReturnCandidate {
            parameters,
            value: GenericReturnValue::VoidBody {
                unread_parameter_slots,
            },
        }));
    }
    let StmtKind::Return { value: Some(value) } = &program.stmts[0].kind else {
        return Ok(None);
    };
    if let Some(target) = request.typed_functional_target {
        let instructions = code.instructions.as_slice();
        let bound = instructions.len() == 3;
        let site_index = usize::from(bound);
        let expected = if bound {
            &[0x2a, 0xba, 0xb0][..]
        } else {
            &[0xba, 0xb0][..]
        };
        let exact_code = code.stopped_at.is_none()
            && code.exception_handler_count == 0
            && code.exception_handlers.is_empty()
            && instructions
                .iter()
                .map(|instruction| instruction.opcode)
                .eq(expected.iter().copied())
            && instructions.get(site_index).is_some_and(|site| {
                site.bci == target.use_site && site.constant_pool_index == Some(target.site_cp)
            });
        let exact_program = program.statements == 1
            && program.stmts.len() == 1
            && !program.ragged
            && matches!(value.kind, ExprKind::MethodReference { .. })
            && value.origin.primary().bci() == target.use_site
            && value.origin.primary().cp() == Some(target.site_cp)
            && program.stmts[0].origin.primary().bci()
                == instructions.last().map_or(u32::MAX, |i| i.bci)
            && matches!(program.lambdas.as_slice(), [site]
                if site.use_site == target.use_site
                    && site.site_cp == target.site_cp
                    && site.typed_reference
                    && site.form == Some(crate::lambda::LambdaForm::MethodReference)
                    && site.refusal.is_none()
                    && match target.kind {
                        TypedFunctionalKind::FunctionStringInteger =>
                            site.sam_name == "apply"
                                && site.evidence.sam_method_type.as_deref()
                                    == Some("(Ljava/lang/Object;)Ljava/lang/Object;")
                                && site.evidence.instantiated_method_type.as_deref()
                                    == Some("(Ljava/lang/String;)Ljava/lang/Integer;"),
                        TypedFunctionalKind::SupplierString =>
                            site.sam_name == "get"
                                && site.evidence.sam_method_type.as_deref()
                                    == Some("()Ljava/lang/Object;")
                                && site.evidence.instantiated_method_type.as_deref()
                                    == Some("()Ljava/lang/String;"),
                    }
                    && site.captures.len() == usize::from(bound)
                    && (!bound || site.captures[0].bci == Some(0)));
        let exact_ssa = ssa.blocks().len() == 1
            && ssa.phis().is_empty()
            && ssa.blocks()[0].instructions().len() == instructions.len()
            && ssa.effects().instructions().len() == instructions.len()
            && ssa.blocks()[0]
                .instructions()
                .iter()
                .zip(instructions)
                .all(|(ssa, raw)| ssa.bci() == raw.bci && ssa.opcode() == raw.opcode);
        let exact_flow = if exact_ssa {
            let flow = ssa.blocks()[0].instructions();
            let site = &flow[site_index];
            let returned = &flow[site_index + 1];
            let capture = if bound {
                let load = &flow[0];
                matches!(load.reads(), [(Slot::Local(0), entry)]
                    if matches!(ssa.value(*entry).def(), Definition::Entry { slot: Slot::Local(0), .. }))
                    && matches!(load.writes(), [(Slot::Stack(_), written)]
                        if matches!(site.reads(), [(Slot::Stack(_), captured)] if captured == written))
            } else {
                site.reads().is_empty()
            };
            capture
                && matches!(site.writes(), [(Slot::Stack(_), produced)]
                    if matches!(returned.reads(), [(Slot::Stack(_), read)] if read == produced))
                && returned.writes().is_empty()
                && ssa
                    .effects()
                    .instructions()
                    .iter()
                    .zip(instructions)
                    .all(|(effect, raw)| {
                        effect.bci() == raw.bci
                            && effect.opcode() == raw.opcode
                            && effect.handlers().is_empty()
                    })
        } else {
            false
        };
        let exact_operations = operations
            .iter()
            .map(|(bci, operation)| (bci, operation))
            .collect::<Vec<_>>();
        let exact_operations = exact_operations.len() == instructions.len()
            && exact_operations.iter().zip(instructions).all(|((bci, operation), raw)| {
                **bci == raw.bci && match raw.opcode {
                    0x2a => matches!(operation, Operation::Load { slot: 0 }),
                    0xba => matches!(operation, Operation::InvokeDynamic(site) if site.cp() == target.site_cp),
                    0xb0 => matches!(operation, Operation::Return),
                    _ => false,
                }
            });
        if exact_code
            && exact_program
            && exact_ssa
            && exact_flow
            && exact_operations
            && parameter_types.is_empty()
        {
            crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
                u64::try_from(instructions.len()).unwrap_or(u64::MAX),
                Some(target.use_site),
            )?;
            crate::stop::poll(budget, Some(target.use_site))?;
            return Ok(Some(GenericReturnCandidate {
                parameters: Vec::new(),
                value: GenericReturnValue::TypedFunctional { target },
            }));
        }
        return Ok(None);
    }
    if matches!(value.kind, ExprKind::Null) {
        if program.statements != 1
            || !parameter_types.is_empty()
            || code.stopped_at.is_some()
            || code.exception_handler_count != 0
            || !code.exception_handlers.is_empty()
            || code.instructions.len() != 2
            || ssa.blocks().len() != 1
            || !ssa.phis().is_empty()
            || ssa.blocks()[0].instructions().len() != 2
        {
            return Ok(None);
        }
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
            3,
            Some(code.instructions[0].bci),
        )?;
        let [push, returned] = &code.instructions[..] else {
            unreachable!("instruction count checked above")
        };
        let [ssa_push, ssa_return] = &ssa.blocks()[0].instructions()[..] else {
            unreachable!("SSA instruction count checked above")
        };
        let Some((Slot::Stack(pushed_slot), pushed_value)) = ssa_push.writes().first() else {
            return Ok(None);
        };
        let mut body_operations = operations.iter();
        let Some((push_operation_bci, Operation::Push(ConstantValue::Null))) =
            body_operations.next()
        else {
            return Ok(None);
        };
        let Some((return_operation_bci, Operation::Return)) = body_operations.next() else {
            return Ok(None);
        };
        let effects = ssa.effects().instructions();
        if body_operations.next().is_some()
            || push.opcode != 0x01
            || returned.opcode != 0xb0
            || push.bci.checked_add(1) != Some(returned.bci)
            || *push_operation_bci != push.bci
            || *return_operation_bci != returned.bci
            || ssa_push.bci() != push.bci
            || ssa_push.opcode() != push.opcode
            || !ssa_push.reads().is_empty()
            || !matches!(ssa_push.writes(), [(Slot::Stack(_), _)])
            || ssa_return.bci() != returned.bci
            || ssa_return.opcode() != returned.opcode
            || !matches!(ssa_return.reads(), [(Slot::Stack(slot), value)]
                if slot == pushed_slot && value == pushed_value)
            || !ssa_return.writes().is_empty()
            || value.origin.primary().provenance() != crate::source_map::Provenance::Direct
            || value.origin.primary().bci() != push.bci
            || program.stmts[0].origin.primary().provenance()
                != crate::source_map::Provenance::Direct
            || program.stmts[0].origin.primary().bci() != returned.bci
            || !matches!(effects, [push_effect, return_effect]
                if push_effect.bci() == push.bci
                    && push_effect.opcode() == push.opcode
                    && push_effect.locals_read().is_empty()
                    && push_effect.locals_written().is_empty()
                    && push_effect.stack_delta() == 1
                    && !push_effect.may_throw()
                    && push_effect.handlers().is_empty()
                    && return_effect.bci() == returned.bci
                    && return_effect.opcode() == returned.opcode
                    && return_effect.locals_read().is_empty()
                    && return_effect.locals_written().is_empty()
                    && return_effect.stack_delta() == -1
                    && !return_effect.may_throw()
                    && return_effect.handlers().is_empty())
        {
            return Ok(None);
        }
        crate::stop::poll(budget, Some(returned.bci))?;
        return Ok(Some(GenericReturnCandidate {
            parameters: Vec::new(),
            value: GenericReturnValue::NullLiteral,
        }));
    }
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
    let static_member_creation =
        || -> Option<(ProvedStaticMemberTarget, &crate::init::Site)> {
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
                ty,
                qualifier: None,
                member_name: None,
                args,
                ..
            } = &value.kind
            else {
                return None;
            };
            if !args.is_empty() {
                return None;
            }
            let mut anchors = std::collections::BTreeSet::new();
            anchors.extend(program.stmts[0].origin.bcis());
            collect_expression_anchors(value, &mut anchors);
            if code.instructions.iter().any(|instruction| {
                instruction.opcode != 0x00 && !anchors.contains(&instruction.bci)
            }) {
                return None;
            }
            let site = value
                .origin
                .derived()
                .iter()
                .filter_map(|origin| sites.site_at_head(origin.bci()))
                .find(|site| {
                    site.constructor == value.origin.primary().bci() && site.arguments.is_empty()
                })?;
            let target = request.static_member_target.filter(|target| {
                target.owner == site.class
                    && target.constructor_descriptor == "()V"
                    && target.source_type_path.last().is_some_and(|segment| {
                        segment.binary_name == site.class
                            && (segment.binary_name.replace('/', ".") == *ty
                                || segment.source_name == *ty)
                    })
            })?;
            Some((target.clone(), site))
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
            None => match static_member_creation() {
                Some((target, site)) => GenericReturnValue::StaticMemberCreation {
                    target: Box::new(target),
                    allocation_bci: site.head,
                    copy_bci: site.dup,
                    constructor_bci: site.constructor,
                },
                None => return Ok(None),
            },
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
        ExprKind::LocalAssign { value, .. }
        | ExprKind::InstanceOf { value, .. }
        | ExprKind::Field {
            receiver: value, ..
        }
        | ExprKind::PostfixUpdate { target: value, .. }
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
        | ExprKind::IntegerConstantName { .. }
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

/// Capture a complete constructor shape proved from its AST, Code, SSA, operations, and effects:
/// an empty Object() body, ordered forwarding to the selected superclass constructor, or direct
/// parameter-to-field writes after Object(). The prologue is the decision from which the selected
/// `InitRecord` is materialized; carrying it makes this sidecar independent of evidence selection.
fn generic_constructor_candidate(
    program: &build::Program,
    names: &NameTable,
    ssa: &SsaTable,
    operations: &Operations,
    code: &jarde_reader::classfile::MethodCodeFacts,
    prologues: &init::Prologues,
    parameter_types: &std::collections::BTreeMap<u16, Type>,
    constructor_descriptor: &str,
    declaring_class: Option<&[u8]>,
    budget: &mut Budget,
) -> Result<Option<GenericConstructorCandidate>, StopReason> {
    macro_rules! no_candidate {
        ($stage:expr) => {{ return Ok(None) }};
    }
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
        || program.stmts.len() < 2
        || program.statements != program.stmts.len()
        || code.stopped_at.is_some()
        || code.exception_handler_count != 0
        || !code.exception_handlers.is_empty()
        || ssa.blocks().len() != 1
        || !ssa.phis().is_empty()
    {
        no_candidate!("shape");
    }
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
        9,
        None,
    )?;
    if program.stmts.len() > 2
        || (code.instructions.len() != 3 && code.instructions.len() != parameter_types.len() + 3)
    {
        return generic_constructor_field_writes_candidate(
            program,
            names,
            ssa,
            operations,
            code,
            prologues,
            parameter_types,
            constructor_descriptor,
            declaring_class,
            budget,
        );
    }
    let parameter_slots: Vec<u16> = parameter_types.keys().copied().collect();
    let StmtKind::ConstructorCall {
        target: ConstructorTarget::Super,
        args,
    } = &program.stmts[0].kind
    else {
        no_candidate!("ast");
    };
    let unused_parameters = !parameter_slots.is_empty() && args.is_empty();
    if (args.len() != parameter_slots.len() && !unused_parameters)
        || !matches!(program.stmts[1].kind, StmtKind::Return { value: None })
    {
        no_candidate!("init");
    }

    let init = prologues.record();
    let Some(init_bci) = init.bci else {
        no_candidate!("target");
    };
    if !init.presented
        || init.target != Some(ConstructorTarget::Super)
        || init.class.is_none()
        || init.declared.is_none()
        || init_bci != program.stmts[0].origin.primary().bci()
    {
        no_candidate!("instructions");
    }
    let Some(Operation::Invoke(target)) = operations.get(init_bci) else {
        no_candidate!("effects");
    };
    if target.kind() != crate::facts::InvokeKind::Special
        || Some(target.owner()) != init.class.as_deref()
        || target.name() != "<init>"
        || if unused_parameters {
            target.owner() != "java/lang/Object" || target.descriptor() != "()V"
        } else {
            target.descriptor() != constructor_descriptor
        }
        || !constructor_descriptor.ends_with(")V")
        || target.is_interface_reference()
    {
        no_candidate!("ssa-receiver");
    }

    let mut expected_opcodes = Vec::with_capacity(parameter_slots.len() + 3);
    expected_opcodes.push(0x2a);
    let forwarded_parameter_slots = if unused_parameters {
        &[][..]
    } else {
        parameter_slots.as_slice()
    };
    expected_opcodes.extend(forwarded_parameter_slots.iter().map(|slot| {
        let Some(ty) = parameter_types.get(slot) else {
            return 0;
        };
        let (compact, generic) = match ty {
            Type::Long => (0x1e, 0x16),
            Type::Float => (0x22, 0x17),
            Type::Double => (0x26, 0x18),
            Type::Reference(_) => (0x2a, 0x19),
            Type::Int | Type::Boolean | Type::Byte | Type::Char | Type::Short => (0x1a, 0x15),
        };
        if *slot <= 3 {
            compact + u8::try_from(*slot).unwrap_or(0)
        } else {
            generic
        }
    }));
    expected_opcodes.extend([0xb7, 0xb1]);
    let block = &ssa.blocks()[0];
    if block.instructions().len() != expected_opcodes.len()
        || block
            .instructions()
            .iter()
            .zip(expected_opcodes.iter().copied())
            .any(|(instruction, expected)| instruction.opcode() != expected)
        || code
            .instructions
            .iter()
            .zip(expected_opcodes.iter().copied())
            .any(|(instruction, expected)| instruction.opcode != expected)
        || operations.iter().count() != expected_opcodes.len()
    {
        no_candidate!("opcode-shape");
    }
    let effects = ssa.effects().instructions();
    if effects.len() != expected_opcodes.len()
        || effects
            .iter()
            .zip(
                expected_opcodes
                    .iter()
                    .copied()
                    .map(|opcode| (opcode, opcode == 0xb7)),
            )
            .any(|(effect, (expected, may_throw))| {
                effect.opcode() != expected
                    || !effect.handlers().is_empty()
                    || effect.may_throw() != may_throw
            })
    {
        no_candidate!("effects");
    }
    let instructions = block.instructions();
    let [(Slot::Stack(0), receiver)] = instructions[0].writes() else {
        return Ok(None);
    };
    if !matches!(instructions[0].reads(), [(Slot::Local(0), _)])
        || !instructions.last().is_some_and(|instruction| {
            instruction.reads().is_empty() && instruction.writes().is_empty()
        })
    {
        no_candidate!("receiver");
    }

    let mut forwarded_values = Vec::with_capacity(forwarded_parameter_slots.len());
    for (index, slot) in forwarded_parameter_slots.iter().enumerate() {
        let Some(instruction) = instructions.get(index + 1) else {
            no_candidate!("forwarding");
        };
        if !matches!(operations.get(code.instructions[index + 1].bci), Some(Operation::Load { slot: loaded }) if loaded == slot)
            || !matches!(instruction.reads(), [(Slot::Local(local), _)] if local == slot)
            || !matches!(args.get(index).map(|arg| &arg.kind), Some(crate::ast::ExprKind::Local(name)) if names.whole(*slot).is_some_and(|expected| expected.text() == name))
        {
            no_candidate!("forwarding");
        }
        let [(Slot::Stack(_), value)] = instruction.writes() else {
            no_candidate!("forwarding-output");
        };
        forwarded_values.push(*value);
    }
    let invoke_index = forwarded_parameter_slots.len() + 1;
    let Some(invoke_instruction) = code.instructions.get(invoke_index) else {
        no_candidate!("invoke-ssa");
    };
    let invoke_reads = instructions[invoke_index].reads();
    let expected_invocation_values = forwarded_values
        .iter()
        .rev()
        .copied()
        .chain(std::iter::once(*receiver));
    if !matches!(operations.get(invoke_instruction.bci), Some(Operation::Invoke(actual)) if actual == target)
        || invoke_reads.len() != forwarded_values.len() + 1
        || invoke_reads
            .iter()
            .map(|(_, value)| *value)
            .ne(expected_invocation_values)
        || !matches!(invoke_reads.last(), Some((Slot::Stack(0), value)) if *value == *receiver)
        || instructions[invoke_index].writes().len() != 1
        || !matches!(instructions[invoke_index].writes()[0].0, Slot::Local(0))
    {
        no_candidate!("invoke");
    }

    let mut parameters = Vec::with_capacity(parameter_types.len());
    for slot in &parameter_slots {
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            None,
        )?;
        let Some(name) = names.whole(*slot) else {
            no_candidate!("parameter-name");
        };
        parameters.push((*slot, name.text().to_owned()));
    }
    Ok(Some(GenericConstructorCandidate {
        parameters,
        forwarded_parameter_slots: forwarded_parameter_slots.to_vec(),
        field_writes: Vec::new(),
        init,
    }))
}

/// Prove the deliberately small constructor initializer: Object(), followed by one or more direct
/// writes of unchanged parameter loads to this class's instance fields, then return.
fn generic_constructor_field_writes_candidate(
    program: &build::Program,
    names: &NameTable,
    ssa: &SsaTable,
    operations: &Operations,
    code: &jarde_reader::classfile::MethodCodeFacts,
    prologues: &init::Prologues,
    parameter_types: &std::collections::BTreeMap<u16, Type>,
    constructor_descriptor: &str,
    declaring_class: Option<&[u8]>,
    budget: &mut Budget,
) -> Result<Option<GenericConstructorCandidate>, StopReason> {
    macro_rules! no_candidate {
        () => {{ return Ok(None) }};
    }
    crate::stop::poll(budget, None)?;
    let field_count = program.stmts.len().saturating_sub(2);
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
        u64::try_from(field_count)
            .unwrap_or(u64::MAX)
            .saturating_mul(7)
            .saturating_add(
                u64::try_from(parameter_types.len())
                    .unwrap_or(u64::MAX)
                    .saturating_mul(2),
            ),
        None,
    )?;
    if field_count == 0
        || program.ragged
        || program.statements != program.stmts.len()
        || code.stopped_at.is_some()
        || code.exception_handler_count != 0
        || !code.exception_handlers.is_empty()
        || ssa.blocks().len() != 1
        || !ssa.phis().is_empty()
        || !matches!(program.stmts.first().map(|stmt| &stmt.kind), Some(StmtKind::ConstructorCall { target: ConstructorTarget::Super, args }) if args.is_empty())
        || !matches!(
            program.stmts.last().map(|stmt| &stmt.kind),
            Some(StmtKind::Return { value: None })
        )
    {
        no_candidate!();
    }
    let descriptor =
        match descriptor_facts(constructor_descriptor.as_bytes(), DescriptorKind::Method) {
            Ok(descriptor) if constructor_descriptor.ends_with(")V") => descriptor,
            _ => no_candidate!(),
        };
    if descriptor.parameters().len() != parameter_types.len() {
        no_candidate!();
    }
    let slots = match jarde_jvm::method_ir::parameter_positions(&descriptor, false) {
        Some(slots) if slots.len() == descriptor.parameters().len() => slots,
        _ => no_candidate!(),
    };
    let mut parameter_descriptors = std::collections::BTreeMap::new();
    for (component, slot) in descriptor.parameters().iter().zip(slots) {
        crate::stop::poll(budget, None)?;
        let Some(bytes) = component.bytes(constructor_descriptor.as_bytes()) else {
            no_candidate!();
        };
        let Ok(spelling) = std::str::from_utf8(bytes) else {
            no_candidate!();
        };
        if !parameter_types.contains_key(&slot) {
            no_candidate!();
        }
        parameter_descriptors.insert(slot, spelling.to_owned());
    }
    if parameter_descriptors.len() != parameter_types.len() {
        no_candidate!();
    }

    let init = prologues.record();
    let Some(init_bci) = init.bci else {
        no_candidate!();
    };
    let Some(declaring_class) = declaring_class else {
        no_candidate!();
    };
    let Ok(this_class) = std::str::from_utf8(declaring_class) else {
        no_candidate!();
    };
    if !init.presented
        || init.target != Some(ConstructorTarget::Super)
        || init.class.as_deref() != Some("java/lang/Object")
        || init.declared.as_deref() != Some(this_class)
        || init_bci != program.stmts[0].origin.primary().bci()
    {
        no_candidate!();
    }
    let Some(Operation::Invoke(target)) = operations.get(init_bci) else {
        no_candidate!();
    };
    if target.kind() != crate::facts::InvokeKind::Special
        || target.owner() != "java/lang/Object"
        || target.name() != "<init>"
        || target.descriptor() != "()V"
        || target.is_interface_reference()
    {
        no_candidate!();
    }

    let expected_instruction_count = field_count.saturating_mul(3).saturating_add(3);
    if code.instructions.len() != expected_instruction_count
        || ssa.blocks()[0].instructions().len() != expected_instruction_count
        || ssa.effects().instructions().len() != expected_instruction_count
        || operations.iter().count() != expected_instruction_count
    {
        no_candidate!();
    }
    let mut seen_bcis = std::collections::BTreeSet::new();
    for (index, ((raw, instruction), effect)) in code
        .instructions
        .iter()
        .zip(ssa.blocks()[0].instructions())
        .zip(ssa.effects().instructions())
        .enumerate()
    {
        crate::stop::poll(budget, Some(raw.bci))?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            Some(raw.bci),
        )?;
        if raw.bci != instruction.bci()
            || raw.bci != effect.bci()
            || raw.opcode != instruction.opcode()
            || raw.opcode != effect.opcode()
            || !seen_bcis.insert(raw.bci)
            || !effect.handlers().is_empty()
            || effect.may_throw() != matches!(raw.opcode, 0xb7 | 0xb5)
            || (index == 0 && raw.opcode != 0x2a)
            || (index == 1 && raw.opcode != 0xb7)
            || (index + 1 == expected_instruction_count && raw.opcode != 0xb1)
        {
            no_candidate!();
        }
    }
    let instructions = ssa.blocks()[0].instructions();
    let [(Slot::Local(0), entry_this)] = instructions[0].reads() else {
        no_candidate!();
    };
    let [(Slot::Stack(0), uninitialized_this)] = instructions[0].writes() else {
        no_candidate!();
    };
    let [(Slot::Local(0), initialized_this)] = instructions[1].writes() else {
        no_candidate!();
    };
    if !matches!(
        ssa.value(*entry_this).def(),
        Definition::Entry {
            slot: Slot::Local(0),
            ..
        }
    ) || !matches!(
        operations.get(code.instructions[0].bci),
        Some(Operation::Load { slot: 0 })
    ) || !matches!(operations.get(init_bci), Some(Operation::Invoke(actual)) if actual == target)
        || !matches!(instructions[1].reads(), [(Slot::Stack(0), value)] if value == uninitialized_this)
        || !matches!(ssa.value(*initialized_this).def(), Definition::Instruction { bci, .. } if *bci == init_bci)
    {
        no_candidate!();
    }

    let mut parameters = Vec::with_capacity(parameter_types.len());
    for slot in parameter_types.keys() {
        crate::stop::poll(budget, None)?;
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            1,
            None,
        )?;
        let Some(name) = names.whole(*slot) else {
            no_candidate!();
        };
        parameters.push((*slot, name.text().to_owned()));
    }
    let parameter_names: std::collections::BTreeMap<_, _> = parameters
        .iter()
        .map(|(slot, name)| (*slot, name.as_str()))
        .collect();
    let mut field_writes = Vec::with_capacity(field_count);
    for field_index in 0..field_count {
        let stmt = &program.stmts[field_index + 1];
        let base = field_index * 3 + 2;
        let receiver_load = &instructions[base];
        let parameter_load = &instructions[base + 1];
        let field_instruction = &instructions[base + 2];
        let receiver_bci = code.instructions[base].bci;
        let parameter_bci = code.instructions[base + 1].bci;
        let field_bci = code.instructions[base + 2].bci;
        if receiver_load.opcode() != 0x2a
            || !matches!(
                operations.get(receiver_bci),
                Some(Operation::Load { slot: 0 })
            )
            || !matches!(receiver_load.reads(), [(Slot::Local(0), value)] if *value == *initialized_this)
            || !matches!(receiver_load.writes(), [(Slot::Stack(_), _)])
        {
            no_candidate!();
        }
        let [(Slot::Local(parameter_slot), parameter_entry)] = parameter_load.reads() else {
            no_candidate!();
        };
        let Some(parameter_type) = parameter_types.get(parameter_slot) else {
            no_candidate!();
        };
        let Some(parameter_descriptor) = parameter_descriptors.get(parameter_slot) else {
            no_candidate!();
        };
        let expected_load = if *parameter_slot <= 3 {
            (match parameter_type {
                Type::Long => 0x1e,
                Type::Float => 0x22,
                Type::Double => 0x26,
                Type::Reference(_) => 0x2a,
                Type::Int | Type::Boolean | Type::Byte | Type::Char | Type::Short => 0x1a,
            }) + *parameter_slot as u8
        } else {
            match parameter_type {
                Type::Long => 0x16,
                Type::Float => 0x17,
                Type::Double => 0x18,
                Type::Reference(_) => 0x19,
                Type::Int | Type::Boolean | Type::Byte | Type::Char | Type::Short => 0x15,
            }
        };
        if parameter_load.opcode() != expected_load
            || !matches!(operations.get(parameter_bci), Some(Operation::Load { slot }) if slot == parameter_slot)
            || !matches!(ssa.value(*parameter_entry).def(), Definition::Entry { slot: Slot::Local(entry_slot), .. } if entry_slot == parameter_slot)
            || !matches!(parameter_load.writes(), [(Slot::Stack(_), parameter_value)]
                if ssa.value(*parameter_value).uses().len() == 1
                    && ssa.value(*parameter_value).uses()[0].bci() == Some(field_bci))
        {
            no_candidate!();
        }
        let Some(Operation::Field {
            access: FieldAccess::Write,
            is_static: false,
            owner,
            name,
            descriptor,
        }) = operations.get(field_bci)
        else {
            no_candidate!();
        };
        if owner != this_class
            || descriptor != parameter_descriptor
            || field_instruction.opcode() != 0xb5
            || !field_instruction.writes().is_empty()
            || field_instruction.reads().len() != 2
        {
            no_candidate!();
        }
        let [(Slot::Stack(_), receiver_value)] = receiver_load.writes() else {
            no_candidate!();
        };
        let [(Slot::Stack(_), parameter_value)] = parameter_load.writes() else {
            no_candidate!();
        };
        if !field_instruction
            .reads()
            .iter()
            .any(|(_, value)| value == receiver_value)
            || !field_instruction
                .reads()
                .iter()
                .any(|(_, value)| value == parameter_value)
        {
            no_candidate!();
        }
        let StmtKind::FieldAssign {
            receiver: Some(receiver),
            name: ast_name,
            op: crate::ast::AssignOp::Assign,
            value,
        } = &stmt.kind
        else {
            no_candidate!();
        };
        let Some(parameter_name) = parameter_names.get(parameter_slot).copied() else {
            no_candidate!();
        };
        if ast_name != name
            || stmt.origin.primary().bci() != field_bci
            || !matches!(&receiver.kind, crate::ast::ExprKind::Local(local) if local == "this")
            || receiver.origin.primary().bci() != receiver_bci
            || !matches!(&value.kind, crate::ast::ExprKind::Local(local) if local == parameter_name)
            || value.origin.primary().bci() != parameter_bci
        {
            no_candidate!();
        }
        field_writes.push(GenericConstructorFieldWrite {
            bci: field_bci,
            owner: owner.clone(),
            name: name.clone(),
            descriptor: descriptor.clone(),
            parameter_slot: *parameter_slot,
        });
    }
    if instructions.last().is_none_or(|instruction| {
        instruction.opcode() != 0xb1
            || !instruction.reads().is_empty()
            || !instruction.writes().is_empty()
            || !matches!(
                operations.get(code.instructions.last().map(|raw| raw.bci).unwrap_or(0)),
                Some(Operation::Return)
            )
    }) {
        no_candidate!();
    }
    if code.instructions.last().map(|instruction| instruction.bci)
        != program.stmts.last().map(|stmt| stmt.origin.primary().bci())
    {
        no_candidate!();
    }
    Ok(Some(GenericConstructorCandidate {
        parameters,
        forwarded_parameter_slots: Vec::new(),
        field_writes,
        init,
    }))
}

fn anonymous_constructor_initializer_bci(
    program: &build::Program,
    ssa: &SsaTable,
    operations: &Operations,
    code: &jarde_reader::classfile::MethodCodeFacts,
    budget: &mut Budget,
) -> Result<Option<ClassSourceAnonymousConstructorInitializer>, StopReason> {
    crate::stop::poll(budget, None)?;
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        4,
        None,
    )?;
    if program.ragged
        || program.stmts.len() != 3
        || program.statements != 3
        || code.stopped_at.is_some()
        || code.exception_handler_count != 0
        || !code.exception_handlers.is_empty()
        || code.instructions.len() != 5
        || ssa.blocks().len() != 1
        || !ssa.phis().is_empty()
        || operations.iter().count() != 5
    {
        return Ok(None);
    }
    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
        12,
        None,
    )?;
    let [
        crate::ast::Stmt {
            kind:
                StmtKind::ConstructorCall {
                    target: ConstructorTarget::Super,
                    args,
                },
            origin: super_origin,
        },
        crate::ast::Stmt {
            kind:
                StmtKind::FieldAssign {
                    receiver: Some(receiver),
                    name,
                    op: AssignOp::Assign,
                    value,
                },
            origin: write_origin,
        },
        crate::ast::Stmt {
            kind: StmtKind::Return { value: None },
            ..
        },
    ] = program.stmts.as_slice()
    else {
        return Ok(None);
    };
    if !args.is_empty()
        || super_origin.primary().bci() != code.instructions[1].bci
        || write_origin.primary().bci() != code.instructions[3].bci
        || !matches!(value.kind, ExprKind::Integer(1))
        || value.origin.primary().bci() != code.instructions[2].bci
        || !matches!(receiver.kind, ExprKind::Path(_))
    {
        return Ok(None);
    }
    let expected = [0x2a, 0xb7, 0x04, 0xb3, 0xb1];
    let physical = &code.instructions;
    let ssa_instructions = ssa.blocks()[0].instructions();
    let effects = ssa.effects().instructions();
    if physical.len() != expected.len()
        || ssa_instructions.len() != physical.len()
        || effects.len() != physical.len()
        || physical
            .iter()
            .zip(ssa_instructions)
            .any(|(raw, instruction)| {
                raw.bci != instruction.bci() || raw.opcode != instruction.opcode()
            })
        || physical.iter().zip(effects).any(|(raw, effect)| {
            raw.bci != effect.bci()
                || raw.opcode != effect.opcode()
                || !effect.handlers().is_empty()
        })
        || physical
            .iter()
            .zip(expected)
            .any(|(instruction, opcode)| instruction.opcode != opcode)
    {
        return Ok(None);
    }
    let [
        Operation::Load { slot: 0 },
        Operation::Invoke(target),
        Operation::Push(ConstantValue::Int(1)),
        Operation::Field {
            access: FieldAccess::Write,
            is_static: true,
            owner,
            name: field_name,
            descriptor,
        },
        Operation::Return,
    ] = code
        .instructions
        .iter()
        .map(|instruction| operations.get(instruction.bci))
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default()
        .as_slice()
    else {
        return Ok(None);
    };
    if target.kind() != crate::facts::InvokeKind::Special
        || target.name() != "<init>"
        || target.descriptor() != "()V"
        || target.owner() == "java/lang/Object"
        || target.is_interface_reference()
        || descriptor != "I"
        || name != field_name
        || receiver_text(receiver).as_deref() != Some(owner.replace('/', ".").as_str())
    {
        return Ok(None);
    }
    let instructions = ssa.blocks()[0].instructions();
    if !matches!(instructions[0].reads(), [(Slot::Local(0), _)])
        || !matches!(instructions[0].writes(), [(Slot::Stack(0), _)])
        || !matches!(instructions[1].reads(), [(Slot::Stack(0), _)])
        || !matches!(instructions[2].reads(), [])
        || !matches!(instructions[2].writes(), [(Slot::Stack(0), _)])
        || !matches!(instructions[3].reads(), [(Slot::Stack(0), _)])
        || !instructions[3].writes().is_empty()
        || !instructions[4].reads().is_empty()
        || !instructions[4].writes().is_empty()
    {
        return Ok(None);
    }
    Ok(Some(ClassSourceAnonymousConstructorInitializer {
        bci: code.instructions[3].bci,
        super_owner: target.owner().to_owned(),
        owner: owner.clone(),
        name: field_name.clone(),
        descriptor: descriptor.clone(),
    }))
}

fn receiver_text(expression: &Expr) -> Option<String> {
    match &expression.kind {
        ExprKind::Path(path) => Some(path.clone()),
        _ => None,
    }
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
    recover_for_class_source_with_anonymous_ast(
        request,
        budget,
        prove_generic_return,
        collect_enum_constructor_candidates,
        false,
        false,
        false,
    )
}

/// Same class-source recovery with private AST retention for a class already selected as the
/// anonymous implementation, and an optional exact empty-constructor proof.
#[doc(hidden)]
pub fn recover_for_class_source_with_anonymous_ast(
    request: &RecoveryRequest<'_>,
    budget: &mut Budget,
    prove_generic_return: bool,
    collect_enum_constructor_candidates: bool,
    retain_all_method_asts: bool,
    retain_generic_call_asts: bool,
    prove_empty_constructor: bool,
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
    let is_ordinary_class = request
        .facts
        .method()
        .declaring_class()
        .is_some_and(|class| {
            class.access_flags() & (ACC_INTERFACE | ACC_ANNOTATION | ACC_ENUM | 0x8000) == 0
        });
    let collect_initializer = is_clinit && (is_ordinary_interface || is_ordinary_class || is_enum);
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
    let mut lambda_helpers = None;
    let mut generic_return = None;
    let mut generic_constructor = None;
    let mut anonymous_allocations = None;
    let mut ast = None;
    let mut field_receivers = None;
    let mut field_write_accessors = None;
    let report = recover_inner(
        request,
        budget,
        collect_initializer.then_some(&mut initializer),
        collect_enum_constructor.then_some(&mut enum_constructor),
        Some(&mut bridge),
        Some(&mut enum_switches),
        Some(&mut array_constructors),
        Some(&mut lambda_helpers),
        Some(&mut enum_switch_field_uses),
        prove_generic_return.then_some(&mut generic_return),
        (prove_generic_return || prove_empty_constructor).then_some(&mut generic_constructor),
        Some(&mut anonymous_allocations),
        Some(&mut ast),
        Some(&mut field_receivers),
        Some(&mut field_write_accessors),
        retain_all_method_asts,
        retain_generic_call_asts,
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
        lambda_helpers = None;
        enum_switch_field_uses = None;
        generic_return = None;
        generic_constructor = None;
        anonymous_allocations = None;
        ast = None;
        field_receivers = None;
        field_write_accessors = None;
    }
    ClassSourceRecovery {
        report,
        initializer,
        enum_constructor,
        bridge,
        enum_switches: enum_switches.unwrap_or_default(),
        array_constructors: array_constructors.unwrap_or_default(),
        lambda_helpers: lambda_helpers.unwrap_or_default(),
        enum_switch_field_uses: enum_switch_field_uses.unwrap_or_default(),
        generic_return,
        generic_constructor,
        anonymous_allocations,
        ast,
        field_receivers: field_receivers.unwrap_or_default(),
        field_write_accessors: field_write_accessors.unwrap_or_default(),
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
    mut lambda_helper_candidate: Option<&mut Option<Vec<ClassSourceLambdaHelperCandidate>>>,
    mut enum_switch_field_use: Option<&mut Option<Vec<ClassSourceEnumSwitchFieldUse>>>,
    generic_return: Option<&mut Option<GenericReturnCandidate>>,
    generic_constructor: Option<&mut Option<GenericConstructorCandidate>>,
    mut anonymous_allocations: Option<&mut Option<AnonymousAllocationScan>>,
    mut class_source_ast: Option<&mut Option<ClassSourceMethodAst>>,
    mut field_receiver_sites: Option<&mut Option<Vec<ClassSourceFieldReceiverSite>>>,
    mut field_write_accessors: Option<&mut Option<Vec<ClassSourceFieldWriteAccessor>>>,
    retain_all_method_asts: bool,
    retain_generic_call_asts: bool,
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
    // The field plan is handed the one receiver reading it cannot take itself: an element read's
    // type, which the array operand's own shape states and [`build`] reads (`element_receiver_type`).
    let element_receiver_type = |value| build::element_receiver_type(ssa, &operations, value);
    let fields = match field::plan(
        ssa,
        &operations,
        &element_receiver_type,
        request.facts.method().declaring_class(),
        request.facts.method().name(),
        request.facts.method().descriptor(),
        request.ir.class_fields(),
        request.superclass_field_writes,
        budget,
    ) {
        Ok(fields) => fields,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    // The copies whose consumers are field instructions (`recover-chained-field-assignment`): the
    // chained field assignment's copies and the receiver copies of the compound assignments the
    // update rule does not present. Read before the concatenation rule, which admits the instance a
    // receiver copy carries through a chain — the field's own `+=` shape.
    let field_copies = match build::FieldCopies::prove(
        ssa,
        canonical,
        &operations,
        &fields,
        request.facts.method().has_receiver(),
        budget,
    ) {
        Ok(field_copies) => field_copies,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    let chains = match concat::plan_conditional_cut_chains(
        ssa,
        canonical,
        &operations,
        &field_copies,
        budget,
    ) {
        Ok(chains) => chains,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    let array_initializers =
        match build::ArrayInitializers::prove(ssa, &operations, &fields, budget) {
            Ok(arrays) => arrays,
            Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
        };
    let sites = init::sites(
        ssa,
        &operations,
        &chains,
        chains.owned(),
        &fields,
        &array_initializers,
        request.profile.java_release,
        request.member_inner_targets,
        request.facts.method(),
        code,
    );
    let mut recovered: Recovered = match crate::region::recover(
        canonical,
        &view,
        ssa,
        &operations,
        request.ir.constant_pool(),
        &chains,
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
        &operations,
        &recovered.regions,
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
    let mut reserved_names = match fields.simple_static_final_names(budget) {
        Ok(names) => names,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    // A static member of another type is spelled with its type path. Its first component must
    // remain a type/package name in Java expression context, rather than becoming a generated
    // parameter or local with the same spelling (`arg0.pick()` is otherwise an instance call).
    for (bci, operation) in operations.iter() {
        let owner = match operation {
            Operation::Invoke(target) if target.kind() == crate::facts::InvokeKind::Static => {
                Some(target.owner())
            }
            Operation::Field {
                is_static: true,
                owner,
                ..
            } => Some(owner.as_str()),
            _ => None,
        };
        if let Some(owner) = owner {
            if let Err(stop) = crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::IrItems,
                1,
                Some(*bci),
            ) {
                return stopped(method, profile.clone(), &selection, stop, budget);
            }
            if let Some(root) = owner
                .split('/')
                .next()
                .and_then(|name| name.split('$').next())
                && crate::names::is_java_identifier(root)
            {
                reserved_names.insert(root.to_owned());
            }
        }
    }
    let names = if request.facts.method().has_receiver() {
        // Slot 0 holds the receiver (JVMS 4.10.1.9), so it is spelled as one: the answer comes from
        // the member's own flags and from nothing else, which is why the naming is told it instead of
        // finding it out from a debug name or a slot ordinal.
        NameTable::build_with_receiver_and_reserved(
            request.facts.method().parameters(),
            slots,
            reuse.evidence(),
            &reserved_names,
        )
    } else {
        NameTable::build_with_reserved(
            request.facts.method().parameters(),
            slots,
            reuse.evidence(),
            &reserved_names,
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
                let argument_parameter_slots = site.map_or_else(Vec::new, |site| {
                    site.arguments
                        .iter()
                        .map(|bci| {
                            anonymous_direct_double_parameter_slot(ssa, *bci, site.constructor)
                        })
                        .collect()
                });
                allocations.push(AnonymousAllocationCandidate {
                    member: member.clone(),
                    head_bci: candidate.head,
                    class: candidate.class.clone(),
                    verified: candidate.verified && site.is_some(),
                    constructor_bci: site.map(|site| site.constructor),
                    argument_bcis: site.map_or_else(Vec::new, |site| site.arguments.clone()),
                    argument_parameter_slots,
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
            method_access_flags: request.facts.method().access_flags(),
            debug_locals: request.facts.debug_locals(),
            names: &names,
            reuse: &reuse,
            chains: &chains,
            field_copies: &field_copies,
            members: request.members,
            member_inner_targets: request.member_inner_targets,
            typed_functional_target: request.typed_functional_target,
            interface_super_calls: request.interface_super_calls,
            reference_overload_calls: request.reference_overload_calls,
            snapshot_hierarchy_widenings: request.snapshot_hierarchy_widenings,
            captured_outer_reads: request.captured_outer_reads,
            outer_super_calls: request.outer_super_calls,
            // The declaring class's own member rows, in the spelling the pool names are compared
            // with: the structural-reflection refusal reads them beside the literal's own class.
            nested_class_members: request
                .facts
                .method()
                .declaring_class()
                .map(|declaring| declaring.inner_class_members())
                .unwrap_or(&[]),
            pool_spelled_members: request.pool_spelled_members,
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
            class_fields: request.ir.class_fields(),
            bridge: bridge.as_ref(),
            sites: &sites,
            prologues: &prologues,
            fields: &fields,
            array_initializers,
            enums: &enums,
            allow_array_constructor_method_references,
        },
        &recovered.regions,
        budget,
    ) {
        Ok(program) => program,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    // The AST is retained for three reasons, each a consumer that cannot re-derive it: the
    // class-source assembler's own full retention, the site scan's two statement shapes (the
    // companion-body projections read the node the scan found), and — since change
    // `recover-double-brace-allocation-site` — one verified allocation of a class's **own
    // anonymous child** (`X$N`), whose allocation point the double-brace presentation re-emits
    // wherever the statement sits (the site scan does not own the assignment position the static
    // initializer's field write is).
    if let Some(slot) = class_source_ast.as_deref_mut()
        && (retain_all_method_asts
            || retain_generic_call_asts
            || class_source_anonymous_site(&program).is_some()
            || allocates_an_anonymous_child(request, anonymous_allocations.as_deref()))
        && let Some(member) = request
            .ir
            .declaration()
            .map(|member| member.identity().clone())
    {
        let parameter_slots = request
            .ir
            .declaration()
            .and_then(|declaration| {
                let descriptor = descriptor_facts(
                    declaration.descriptor().0.as_slice(),
                    DescriptorKind::Method,
                )
                .ok()?;
                jarde_jvm::method_ir::parameter_positions(
                    &descriptor,
                    declaration.access_flags() & 0x0008 != 0,
                )
            })
            .unwrap_or_default();
        let parameter_names: Vec<Option<String>> = parameter_slots
            .iter()
            .map(|slot| {
                names
                    .whole(*slot)
                    .map(|rendered| rendered.text().to_owned())
            })
            .collect();
        let retain_complete_invocation_ast = retain_all_method_asts || retain_generic_call_asts;
        if retain_complete_invocation_ast {
            let scan_items = u64::try_from(code.instructions.len())
                .unwrap_or(u64::MAX)
                .saturating_mul(2);
            if let Err(stop) = crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
                scan_items,
                code.instructions.first().map(|instruction| instruction.bci),
            ) {
                return stopped(method, profile.clone(), &selection, stop, budget);
            }
        }
        let opcode_by_bci: std::collections::HashMap<u32, u8> = if retain_complete_invocation_ast {
            code.instructions
                .iter()
                .map(|instruction| (instruction.bci, instruction.opcode))
                .collect()
        } else {
            std::collections::HashMap::new()
        };
        let instruction_bcis: Vec<u32> = if retain_complete_invocation_ast {
            request
                .ir
                .code()
                .map(|code| {
                    code.instructions
                        .iter()
                        .map(|instruction| instruction.bci)
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let call_targets: Vec<(u32, u8, crate::facts::CallTarget)> =
            if retain_complete_invocation_ast {
                operations
                    .iter()
                    .filter_map(|(bci, operation)| match operation {
                        Operation::Invoke(target) => {
                            let opcode = *opcode_by_bci.get(bci)?;
                            Some((*bci, opcode, target.clone()))
                        }
                        _ => None,
                    })
                    .collect()
            } else {
                Vec::new()
            };
        let weight = if retain_complete_invocation_ast {
            program_node_count(&program)
                .saturating_add(u64::try_from(parameter_names.len()).unwrap_or(u64::MAX))
                .saturating_add(u64::try_from(instruction_bcis.len()).unwrap_or(u64::MAX))
                .saturating_add(u64::try_from(call_targets.len()).unwrap_or(u64::MAX))
        } else {
            2
        };
        if let Err(stop) = crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            weight,
            program
                .stmts
                .first()
                .map(|statement| statement.origin.primary().bci()),
        ) {
            return stopped(method, profile.clone(), &selection, stop, budget);
        }
        let anonymous_constructor_initializer_bci = if retain_all_method_asts
            && request.facts.method().name() == "<init>"
        {
            match anonymous_constructor_initializer_bci(&program, ssa, &operations, code, budget) {
                Ok(candidate) => candidate,
                Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
            }
        } else {
            None
        };
        let generic_call_init = if retain_generic_call_asts
            && request.facts.method().name() == "<init>"
            && let Some(prologue) = prologues.prologue()
        {
            let record_cost = prologue
                .class
                .len()
                .saturating_add(prologue.declared.len())
                .saturating_add(1);
            if let Err(stop) = crate::stop::poll(budget, Some(prologue.bci)).and_then(|()| {
                crate::stop::charge(
                    budget,
                    jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(record_cost).unwrap_or(u64::MAX),
                    Some(prologue.bci),
                )
            }) {
                return stopped(method, profile.clone(), &selection, stop, budget);
            }
            Some(prologues.record())
        } else {
            None
        };
        *slot = Some(ClassSourceMethodAst {
            projection: std::sync::Arc::new(ClassSourceMethodAstSource {
                program: program.clone(),
                member,
                current_class: request
                    .facts
                    .method()
                    .declaring_class()
                    .map(|declaring| declaring.name().replace('/', ".")),
                nested_class_members: request
                    .facts
                    .method()
                    .declaring_class()
                    .map(|declaring| declaring.inner_class_members().to_vec())
                    .unwrap_or_default(),
                parameter_names,
                parameter_slots,
                complete_code: request.ir.code().is_some_and(|code| {
                    matches!(code.execution, ExecutionReport::Complete { .. })
                        && code.stopped_at.is_none()
                }),
                has_exception_handlers: request.ir.code().is_some_and(|code| {
                    code.exception_handler_count != 0 || !code.exception_handlers.is_empty()
                }),
                instruction_count: request.ir.code().map_or(0, |code| code.instructions.len()),
                instruction_bcis,
                call_targets,
                anonymous_constructor_initializer_bci,
                generic_call_init,
            }),
        });
    }
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
                request.facts.method().descriptor(),
                request
                    .ir
                    .declaration()
                    .map(|declaration| declaration.class_name().0.as_slice()),
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
    if let Some(lambda_helpers) = lambda_helper_candidate.as_deref_mut()
        && let (Some(member), Some(member_declaration)) = (
            request
                .ir
                .declaration()
                .map(|member| member.identity().clone()),
            request.ir.declaration(),
        )
        && !program.lambdas.is_empty()
    {
        let exact_helpers = crate::lambda::synthetic_lambda_helper_candidates(request.ir);
        let projection = std::sync::Arc::new(LambdaHelperProjectionSource {
            program: program.clone(),
            facts: request.facts.clone(),
            declaration: declaration.declaration().cloned(),
            member: member.clone(),
        });
        let mut candidates = Vec::new();
        for site in &program.lambdas {
            let Some(helper) = exact_helpers.iter().find(|helper| {
                helper.call_site == site.use_site
                    && helper.site_cp == site.site_cp
                    && helper.owner.0 == member_declaration.class_name().0
            }) else {
                continue;
            };
            let helper_identity = jarde_reader::model::PhysicalMethodId {
                owner: member.owner.clone(),
                name: helper.name.clone(),
                descriptor: helper.descriptor.clone(),
            };
            candidates.push(ClassSourceLambdaHelperCandidate {
                member: member.clone(),
                helper: helper_identity,
                helper_owner: helper.owner.clone(),
                bootstrap_index: helper.bootstrap_index,
                implementation_index: helper.implementation_index,
                use_site: helper.call_site,
                site_cp: helper.site_cp,
                implementation_kind: helper.implementation_kind,
                projection: projection.clone(),
            });
        }
        *lambda_helpers = Some(candidates);
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
    if let Some(receiver_slot) = field_receiver_sites.as_deref_mut() {
        // A non-constructor holds only the empty NotThisRule record. It has no owning
        // initialization facts to copy, so it does not pay for this constructor-only handoff.
        if prologues.answered() {
            let init_record_cost = prologues
                .prologue()
                .map(|prologue| prologue.class.len().saturating_add(prologue.declared.len()))
                .unwrap_or(0)
                .saturating_add(1);
            let charged = crate::stop::poll(budget, None).and_then(|()| {
                crate::stop::charge(
                    budget,
                    jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(init_record_cost).unwrap_or(u64::MAX),
                    None,
                )
            });
            if let Err(stop) = charged {
                return stopped(method, profile.clone(), &selection, stop, budget);
            }
        }
        let init_record = prologues.record();
        match same_class_field_receiver_sites(
            &program,
            &fields,
            &field_presentations,
            &request,
            &names,
            &reuse,
            &init_record,
            &operations,
            ssa,
            budget,
        ) {
            Ok((receivers, accessors)) => {
                *receiver_slot = Some(receivers);
                if let Some(accessor_slot) = field_write_accessors.as_deref_mut() {
                    *accessor_slot = Some(accessors);
                }
            }
            Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
        }
    }
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
        parameter_names: (0..request.facts.method().parameters())
            .map(|slot| {
                parameter_types
                    .contains_key(&slot)
                    .then(|| names.whole(slot).map(|name| name.text().to_owned()))
                    .flatten()
            })
            .collect(),
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

/// The only caller AST shape this class-source slice retains: a sole direct `return new T()`.
/// Counts retained statements, expressions, and class-source side tables before cloning a full
/// selected child AST. The iterative walk bounds stack use independently of source nesting depth.
fn program_node_count(program: &build::Program) -> u64 {
    let mut count = u64::try_from(program.statements).unwrap_or(u64::MAX);
    count = count
        .saturating_add(u64::try_from(program.field_increments.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(program.lambdas.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(program.accessors.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(program.array_constructor_sites.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(program.lambda_refusals.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(program.accessor_refusals.len()).unwrap_or(u64::MAX));
    let mut statements: Vec<_> = program.stmts.iter().collect();
    let mut expressions = Vec::<&Expr>::new();
    while let Some(statement) = statements.pop() {
        use StmtKind as K;
        match &statement.kind {
            K::Declare { value, .. } => expressions.extend(value.iter()),
            K::Assign { value, .. } | K::Expr(value) | K::Throw { value } => {
                expressions.push(value)
            }
            K::Assert { cond, message } => {
                expressions.push(cond);
                expressions.extend(message.iter());
            }
            K::FieldAssign {
                receiver, value, ..
            } => {
                expressions.extend(receiver.iter());
                expressions.push(value);
            }
            K::IndexAssign {
                array,
                index,
                value,
                ..
            } => {
                expressions.extend([array, index, value]);
            }
            K::ConstructorCall { args, .. } => expressions.extend(args),
            K::Return { value } => expressions.extend(value.iter()),
            K::If {
                cond,
                then_body,
                else_body,
            } => {
                expressions.push(cond);
                statements.extend(then_body);
                statements.extend(else_body);
            }
            K::While { cond, body, .. } | K::DoWhile { cond, body, .. } => {
                expressions.push(cond);
                statements.extend(body);
            }
            K::For {
                init,
                cond,
                update,
                body,
                ..
            } => {
                statements.extend([init.as_ref(), update.as_ref()]);
                expressions.push(cond);
                statements.extend(body);
            }
            K::ForEach { iterable, body, .. } => {
                expressions.push(iterable);
                statements.extend(body);
            }
            K::Switch { value, arms } => {
                expressions.push(value);
                for arm in arms {
                    statements.extend(&arm.body);
                }
            }
            K::Try {
                resources,
                catches,
                body,
                finally_body,
            } => {
                expressions.extend(resources.iter().map(|resource| &resource.value));
                for catch in catches {
                    statements.extend(&catch.body);
                }
                statements.extend(body);
                statements.extend(finally_body.iter().flatten());
            }
            K::Synchronized { lock, body } => {
                expressions.push(lock);
                statements.extend(body);
            }
            K::Break { .. } | K::Continue { .. } | K::Fallback { .. } => {}
        }
    }
    while let Some(expression) = expressions.pop() {
        count = count.saturating_add(1);
        use ExprKind as K;
        match &expression.kind {
            K::LocalAssign { value, .. }
            | K::InstanceOf { value, .. }
            | K::PostfixUpdate { target: value, .. }
            | K::ArrayLength { array: value }
            | K::Cast { value, .. }
            | K::Not { value }
            | K::Neg { value } => expressions.push(value),
            K::Call { receiver, args, .. } => {
                expressions.extend(receiver.iter().map(Box::as_ref));
                expressions.extend(args);
            }
            K::New {
                qualifier, args, ..
            } => {
                expressions.extend(qualifier.iter().map(Box::as_ref));
                expressions.extend(args);
            }
            K::Lambda { body, .. }
            | K::MethodReference {
                qualifier: body, ..
            } => {
                expressions.push(body);
            }
            K::Field { receiver, .. } => expressions.push(receiver),
            K::Index { array, index } => expressions.extend([array.as_ref(), index.as_ref()]),
            K::NewArray {
                lengths,
                initializers,
                ..
            } => {
                expressions.extend(lengths);
                expressions.extend(initializers.iter().flatten());
            }
            K::Binary { left, right, .. } => {
                expressions.extend([left.as_ref(), right.as_ref()]);
            }
            K::Conditional {
                test,
                when_true,
                when_false,
            } => {
                expressions.extend([test.as_ref(), when_true.as_ref(), when_false.as_ref()]);
            }
            K::Concat { parts } => expressions.extend(parts.iter().map(|part| &part.value)),
            K::Local(_)
            | K::Integer(_)
            | K::IntegerConstantName { .. }
            | K::Boolean(_)
            | K::Long(_)
            | K::Float(_)
            | K::Double(_)
            | K::Str(_)
            | K::Null
            | K::ClassLiteral { .. }
            | K::Path(_)
            | K::QualifiedThis { .. }
            | K::Super { .. } => {}
        }
    }
    count.max(1)
}

fn class_source_direct_return_new(program: &build::Program) -> Option<(Vec<u32>, &str, &[Expr])> {
    // The method returns one anonymous allocation directly. A prologue of local declarations may
    // precede the return — javac still evaluates every statement in order, and the projected
    // `new Super(args) { ... }` replays them unchanged — but no other statement shape: the
    // retained body is exactly local declarations and the one return.
    if program.ragged || program.statements != program.stmts.len() {
        return None;
    }
    let (last, leading) = program.stmts.split_last()?;
    if !leading
        .iter()
        .all(|statement| matches!(statement.kind, StmtKind::Declare { .. }))
    {
        return None;
    }
    let StmtKind::Return {
        value: Some(expression),
    } = &last.kind
    else {
        return None;
    };
    let ExprKind::New {
        ty,
        qualifier: None,
        member_name: None,
        args,
        ..
    } = &expression.kind
    else {
        return None;
    };
    let primary = expression.origin.primary().bci();
    let mut bcis = vec![primary];
    bcis.extend(
        expression
            .origin
            .derived()
            .iter()
            .map(|origin| origin.bci())
            .filter(|bci| *bci != primary),
    );
    Some((bcis, ty.as_str(), args))
}

/// One local-declaration statement whose sole initializer is an anonymous allocation. The target
/// must spell pool-form (`$` in its binary name): this shape serves the anonymous projection, and
/// an ordinary nameable class's allocation (`Inner inner = new Inner();`) is not an anonymous
/// site — counting one would consume the proved-site count and suppress an unrelated anonymous
/// projection in the same class.
fn class_source_local_decl_initializer_new(
    statement: &crate::ast::Stmt,
) -> Option<(Vec<u32>, &str, &[Expr])> {
    let StmtKind::Declare {
        value: Some(expression),
        ..
    } = &statement.kind
    else {
        return None;
    };
    // The declared initializer is exactly the allocation, and the same expression shape the
    // direct-return scan takes: one `new Child(args)` with no qualifier, no explicit constructor
    // body reference and no diamond. The expression-level match is the shared scan's core; only
    // the statement position differs (design criterion 5: the shape is stated by where the
    // statement sits).
    let ExprKind::New {
        ty,
        qualifier: None,
        member_name: None,
        args,
        ..
    } = &expression.kind
    else {
        return None;
    };
    if !ty.contains('$') {
        return None;
    }
    let primary = expression.origin.primary().bci();
    let mut bcis = vec![primary];
    bcis.extend(
        expression
            .origin
            .derived()
            .iter()
            .map(|origin| origin.bci())
            .filter(|bci| *bci != primary),
    );
    Some((bcis, ty.as_str(), args))
}

/// The proved anonymous allocation site of one retained method body, over both shapes the
/// superclass projection consumes. A direct-return match always wins, so every body the
/// pre-relaxation scan accepted produces the same site with the same shape; the local-declaration
/// initializer shape is only derived when no direct return matches, and then only when the body
/// holds exactly one such declaration (two or more initializer sites leave the method without a
/// proved site — design criterion 2 keeps the proved-allocation count at one).
pub(crate) fn class_source_anonymous_site(
    program: &build::Program,
) -> Option<(Vec<u32>, &str, &[Expr], AnonymousSiteShape)> {
    if program.ragged || program.statements != program.stmts.len() {
        return None;
    }
    if let Some((bcis, ty, args)) = class_source_direct_return_new(program) {
        return Some((bcis, ty, args, AnonymousSiteShape::DirectReturn));
    }
    let mut candidates = program
        .stmts
        .iter()
        .filter_map(class_source_local_decl_initializer_new);
    let (bcis, ty, args) = match (candidates.next(), candidates.next()) {
        (Some(site), None) => site,
        _ => return None,
    };
    Some((bcis, ty, args, AnonymousSiteShape::LocalDeclInitializer))
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
    let enum_statements = if request
        .facts
        .method()
        .declaring_class()
        .is_some_and(|class| class.access_flags() & ACC_ENUM != 0)
    {
        crate::stop::charge(
            budget,
            jarde_reader::budget::CountedBudgetDimension::IrItems,
            u64::try_from(program.statements).unwrap_or(u64::MAX),
            None,
        )?;
        crate::stop::poll(budget, None)?;
        program.stmts.clone()
    } else {
        Vec::new()
    };
    Ok(ClassInitializerCandidates {
        member,
        has_exception_handlers,
        steps,
        statements: enum_statements,
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
        ExprKind::LocalAssign {
            value: qualifier, ..
        }
        | ExprKind::MethodReference { qualifier, .. }
        | ExprKind::ArrayLength { array: qualifier }
        | ExprKind::Cast {
            value: qualifier, ..
        }
        | ExprKind::InstanceOf {
            value: qualifier, ..
        }
        | ExprKind::Not { value: qualifier }
        | ExprKind::Neg { value: qualifier }
        | ExprKind::PostfixUpdate {
            target: qualifier, ..
        } => {
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
        | ExprKind::IntegerConstantName { .. }
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
        ExprKind::LocalAssign {
            value: qualifier, ..
        }
        | ExprKind::MethodReference { qualifier, .. }
        | ExprKind::ArrayLength { array: qualifier }
        | ExprKind::Cast {
            value: qualifier, ..
        }
        | ExprKind::InstanceOf {
            value: qualifier, ..
        }
        | ExprKind::Not { value: qualifier }
        | ExprKind::Neg { value: qualifier }
        | ExprKind::PostfixUpdate {
            target: qualifier, ..
        } => {
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
        | ExprKind::IntegerConstantName { .. }
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
        // An `assert` never reaches a `<clinit>` the build produced (the fold that writes it runs
        // on retained copies for class-source projection), and the category that fits its halves
        // if one ever does is the expression one.
        StmtKind::Assert { .. } => ClassInitializerStatementKind::Expression,
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

#[cfg(test)]
mod generic_call_ast_projection_tests {
    use super::*;
    use crate::ast::{Expr, ExprKind, Stmt, StmtKind};
    use jarde_reader::budget::{Budget, CancellationToken, Limits};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalVariant, SnapshotId,
    };
    use std::collections::BTreeMap;

    fn ast() -> (ClassSourceMethodAst, ClassSourceInvokeKey) {
        let member = PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: SnapshotId("generic-call-report-test".to_owned()),
                },
                class_bytes: ClassBytesId {
                    digest: Digest("generic-call-report-test".to_owned()),
                    length: 1,
                },
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"relay".to_vec()),
            descriptor: JvmBytes(b"(Ljava/lang/Number;)Ljava/lang/Object;".to_vec()),
        };
        let target = crate::facts::CallTarget::new(
            crate::facts::InvokeKind::Virtual,
            "sample/Box",
            "pick",
            "(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Object;",
            false,
        );
        let key = ClassSourceInvokeKey {
            call_bci: 3,
            opcode: 0xb6,
            target: target.clone(),
        };
        let argument = Expr::direct(ExprKind::Local("value".to_owned()), 2)
            .presenting(Type::Reference("java.lang.Object".to_owned()));
        let call = Expr::direct(
            ExprKind::Call {
                receiver: Some(Box::new(Expr::direct(
                    ExprKind::Local("this".to_owned()),
                    1,
                ))),
                name: "pick".to_owned(),
                args: vec![argument.clone(), argument],
            },
            3,
        );
        let program = build::Program {
            stmts: vec![Stmt::new(
                StmtKind::Return { value: Some(call) },
                OriginSet::new(crate::source_map::Origin::direct(3)),
            )],
            field_increments: BTreeMap::new(),
            statements: 1,
            ragged: false,
            lambdas: Vec::new(),
            accessors: Vec::new(),
            array_constructor_sites: Vec::new(),
            lambda_refusals: Vec::new(),
            accessor_refusals: Vec::new(),
            lambdas_presented: 0,
            accessors_presented: 0,
        };
        let source = ClassSourceMethodAstSource {
            program,
            member,
            current_class: Some("sample.Box".to_owned()),
            nested_class_members: Vec::new(),
            parameter_names: vec![Some("value".to_owned())],
            parameter_slots: vec![1],
            complete_code: true,
            has_exception_handlers: false,
            instruction_count: 1,
            instruction_bcis: vec![3],
            call_targets: vec![(3, 0xb6, target)],
            anonymous_constructor_initializer_bci: None,
            generic_call_init: None,
        };
        (
            ClassSourceMethodAst {
                projection: std::sync::Arc::new(source),
            },
            key,
        )
    }

    fn two_site_ast() -> (ClassSourceMethodAst, Vec<ClassSourceInvokeKey>) {
        let (ast, first) = ast();
        let mut source = (*ast.projection).clone();
        let second_target = crate::facts::CallTarget::new(
            crate::facts::InvokeKind::Virtual,
            "sample/Box",
            "other",
            "(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Object;",
            false,
        );
        let second = ClassSourceInvokeKey {
            call_bci: 5,
            opcode: 0xb6,
            target: second_target.clone(),
        };
        let argument = Expr::direct(ExprKind::Local("value".to_owned()), 4)
            .presenting(Type::Reference("java.lang.Object".to_owned()));
        let call = Expr::direct(
            ExprKind::Call {
                receiver: Some(Box::new(Expr::direct(
                    ExprKind::Local("this".to_owned()),
                    4,
                ))),
                name: "other".to_owned(),
                args: vec![argument.clone(), argument],
            },
            5,
        );
        source.program.stmts.push(Stmt::new(
            StmtKind::Expr(call),
            OriginSet::new(crate::source_map::Origin::direct(5)),
        ));
        source.instruction_count = 2;
        source.instruction_bcis = vec![3, 5];
        source.call_targets.push((5, 0xb6, second_target));
        (
            ClassSourceMethodAst {
                projection: std::sync::Arc::new(source),
            },
            vec![first, second],
        )
    }

    fn object_presentation_wrapper_ast(
        physical_cast: bool,
    ) -> (
        ClassSourceMethodAst,
        ClassSourceInvokeKey,
        ClassSourceInvokeKey,
    ) {
        let member = PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: SnapshotId("generic-call-wrapper-test".to_owned()),
                },
                class_bytes: ClassBytesId {
                    digest: Digest("generic-call-wrapper-test".to_owned()),
                    length: 1,
                },
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"relay".to_vec()),
            descriptor: JvmBytes(b"()Ljava/lang/Object;".to_vec()),
        };
        let producer_target = crate::facts::CallTarget::new(
            crate::facts::InvokeKind::Static,
            "sample/Box",
            "first",
            "()Ljava/lang/Object;",
            false,
        );
        let consumer_target = crate::facts::CallTarget::new(
            crate::facts::InvokeKind::Static,
            "sample/Box",
            "consume",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            false,
        );
        let producer_key = ClassSourceInvokeKey {
            call_bci: 2,
            opcode: 0xb8,
            target: producer_target.clone(),
        };
        let consumer_bci = if physical_cast { 4 } else { 3 };
        let consumer_key = ClassSourceInvokeKey {
            call_bci: consumer_bci,
            opcode: 0xb8,
            target: consumer_target.clone(),
        };
        let child = Expr::direct(
            ExprKind::Call {
                receiver: None,
                name: "first".to_owned(),
                args: Vec::new(),
            },
            2,
        )
        .presenting(Type::Reference("java.lang.Object".to_owned()));
        let argument = if physical_cast {
            Expr::direct(
                ExprKind::Cast {
                    ty: Type::Reference("java.lang.Object".to_owned()),
                    value: Box::new(child.clone()),
                },
                3,
            )
        } else {
            Expr::new(
                ExprKind::Cast {
                    ty: Type::Reference("java.lang.Object".to_owned()),
                    value: Box::new(child.clone()),
                },
                OriginSet::new(crate::source_map::Origin::direct(2))
                    .plus_derived(crate::source_map::Origin::derived(consumer_bci)),
            )
        };
        let outer = Expr::direct(
            ExprKind::Call {
                receiver: None,
                name: "consume".to_owned(),
                args: vec![argument],
            },
            consumer_bci,
        );
        let program = build::Program {
            stmts: vec![Stmt::new(
                StmtKind::Return { value: Some(outer) },
                OriginSet::new(crate::source_map::Origin::direct(consumer_bci)),
            )],
            field_increments: std::collections::BTreeMap::new(),
            statements: 1,
            ragged: false,
            lambdas: Vec::new(),
            accessors: Vec::new(),
            array_constructor_sites: Vec::new(),
            lambda_refusals: Vec::new(),
            accessor_refusals: Vec::new(),
            lambdas_presented: 0,
            accessors_presented: 0,
        };
        let mut call_targets = vec![
            (2, 0xb8, producer_target),
            (consumer_bci, 0xb8, consumer_target),
        ];
        call_targets.sort_by_key(|(bci, _, _)| *bci);
        let instruction_bcis = if physical_cast {
            vec![2, 3, consumer_bci]
        } else {
            vec![2, consumer_bci]
        };
        let source = ClassSourceMethodAstSource {
            program,
            member,
            current_class: Some("sample.Box".to_owned()),
            nested_class_members: Vec::new(),
            parameter_names: Vec::new(),
            parameter_slots: Vec::new(),
            complete_code: true,
            has_exception_handlers: false,
            instruction_count: instruction_bcis.len(),
            instruction_bcis,
            call_targets,
            anonymous_constructor_initializer_bci: None,
            generic_call_init: None,
        };
        (
            ClassSourceMethodAst {
                projection: std::sync::Arc::new(source),
            },
            producer_key,
            consumer_key,
        )
    }

    fn fresh_budget() -> Budget {
        Budget::new(Limits {
            ir_items: 1000,
            analysis_steps: 1000,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        })
    }

    #[test]
    fn exact_object_presentation_wrapper_is_reported_transparent_and_removed_with_casts() {
        let (ast, producer, consumer) = object_presentation_wrapper_ast(false);
        let mut budget = fresh_budget();
        let sites =
            class_source_invoke_ast_sites(&ast, std::slice::from_ref(&consumer), &mut budget)
                .unwrap()
                .unwrap();
        let wrapper = sites[0].arguments[0]
            .presentation_wrapper
            .as_ref()
            .expect("descriptor-erasure wrapper is explicit in the AST facts");
        assert_eq!(wrapper.ty, Type::Reference("java.lang.Object".to_owned()));
        assert_eq!(wrapper.child.primary.bci, producer.call_bci);
        assert_eq!(
            sites[0].arguments[0].primary.method.as_ref(),
            Some(&ast.projection.member)
        );
        assert_eq!(
            wrapper.child.primary.method.as_ref(),
            Some(&ast.projection.member)
        );
        assert!(matches!(
            &wrapper.child.shape,
            ClassSourceAstExpressionShape::Call { .. }
        ));

        let uses = class_source_invoke_result_uses(&ast, &producer, &[], &mut budget)
            .unwrap()
            .unwrap();
        assert!(uses.iter().any(|use_site| {
            use_site.kind
                == ClassSourceInvokeResultUseKind::DirectCallArgument {
                    consumer: Some(consumer.clone()),
                    argument_index: 0,
                }
                && use_site.expression.primary.bci == producer.call_bci
        }));

        let projected = project_class_source_invoke_argument_edits(
            &ast,
            &[ClassSourceInvokeArgumentPresentationCast {
                call_bci: consumer.call_bci,
                opcode: consumer.opcode,
                target: consumer.target.clone(),
                argument_index: 0,
            }],
            &[ClassSourceInvokeArgumentCast {
                call_bci: consumer.call_bci,
                opcode: consumer.opcode,
                target: consumer.target.clone(),
                argument_index: 0,
                ty: Type::Reference("java.lang.Number".to_owned()),
            }],
            &mut budget,
        )
        .unwrap()
        .unwrap();
        let StmtKind::Return { value: Some(call) } = &projected.projection.program.stmts[0].kind
        else {
            panic!("wrapper is removed before the approved upcast is applied")
        };
        let ExprKind::Call { args, .. } = &call.kind else {
            panic!("the projected consumer remains a call")
        };
        let [argument] = args.as_slice() else {
            panic!("the consumer has one argument")
        };
        let ExprKind::Cast { ty, value } = &argument.kind else {
            panic!("the approved source upcast remains")
        };
        assert_eq!(ty, &Type::Reference("java.lang.Number".to_owned()));
        assert!(matches!(&value.kind, ExprKind::Call { .. }));
        assert_eq!(value.origin.primary().bci(), producer.call_bci);
    }

    #[test]
    fn null_object_wrapper_is_reported_but_physical_checkcast_is_not_removable() {
        let (ast, _, consumer) = object_presentation_wrapper_ast(false);
        let mut source = (*ast.projection).clone();
        let StmtKind::Return {
            value:
                Some(Expr {
                    kind: ExprKind::Call { args, .. },
                    ..
                }),
        } = &mut source.program.stmts[0].kind
        else {
            panic!("fixture returns the consumer call")
        };
        let ExprKind::Cast { value, .. } = &mut args[0].kind else {
            panic!("fixture argument is source-presentation cast")
        };
        *value = Box::new(Expr::direct(ExprKind::Null, 2));
        let null_ast = ClassSourceMethodAst {
            projection: std::sync::Arc::new(source),
        };
        let mut budget = fresh_budget();
        let sites =
            class_source_invoke_ast_sites(&null_ast, std::slice::from_ref(&consumer), &mut budget)
                .unwrap()
                .unwrap();
        let child = &sites[0].arguments[0]
            .presentation_wrapper
            .as_ref()
            .expect("null target type wrapper is recognized")
            .child;
        assert!(child.null_literal);
        assert_eq!(child.presented_type, None);

        let (physical_ast, physical_producer, physical_consumer) =
            object_presentation_wrapper_ast(true);
        let mut physical_budget = fresh_budget();
        let physical_sites = class_source_invoke_ast_sites(
            &physical_ast,
            std::slice::from_ref(&physical_consumer),
            &mut physical_budget,
        )
        .unwrap()
        .unwrap();
        assert!(
            physical_sites[0].arguments[0]
                .presentation_wrapper
                .is_none()
        );
        let physical_uses = class_source_invoke_result_uses(
            &physical_ast,
            &physical_producer,
            &[],
            &mut physical_budget,
        )
        .unwrap()
        .unwrap();
        assert!(physical_uses.iter().any(|use_site| {
            use_site.kind == ClassSourceInvokeResultUseKind::Other { consumer_bci: 3 }
        }));
        assert!(
            project_class_source_invoke_argument_edits(
                &physical_ast,
                &[ClassSourceInvokeArgumentPresentationCast {
                    call_bci: physical_consumer.call_bci,
                    opcode: physical_consumer.opcode,
                    target: physical_consumer.target,
                    argument_index: 0,
                }],
                &[],
                &mut physical_budget,
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn reference_wrapper_tracks_number_and_object_array_descriptors() {
        for (source_type, descriptor_type) in [
            ("java.lang.Number", "Ljava/lang/Number;"),
            ("java.lang.Object[]", "[Ljava/lang/Object;"),
        ] {
            let (ast, _, mut consumer) = object_presentation_wrapper_ast(false);
            let mut source = (*ast.projection).clone();
            let producer_descriptor = format!("(){descriptor_type}");
            let consumer_descriptor = format!("({descriptor_type})Ljava/lang/Object;");
            let producer_target = crate::facts::CallTarget::new(
                crate::facts::InvokeKind::Static,
                "sample/Box",
                "first",
                producer_descriptor,
                false,
            );
            let consumer_target = crate::facts::CallTarget::new(
                crate::facts::InvokeKind::Static,
                "sample/Box",
                "consume",
                consumer_descriptor,
                false,
            );
            source.call_targets[0].2 = producer_target.clone();
            source.call_targets[1].2 = consumer_target.clone();
            consumer.target = consumer_target;
            let StmtKind::Return {
                value:
                    Some(Expr {
                        kind: ExprKind::Call { args, .. },
                        ..
                    }),
            } = &mut source.program.stmts[0].kind
            else {
                panic!("fixture returns the consumer call")
            };
            let argument = &mut args[0];
            let ExprKind::Cast { ty, value } = &mut argument.kind else {
                panic!("fixture argument is source-presentation cast")
            };
            let source_type = Type::Reference(source_type.to_owned());
            *ty = source_type.clone();
            argument.presented = Some(source_type.clone());
            value.presented = Some(source_type);
            let typed_ast = ClassSourceMethodAst {
                projection: std::sync::Arc::new(source),
            };
            let mut budget = fresh_budget();
            let sites = class_source_invoke_ast_sites(
                &typed_ast,
                std::slice::from_ref(&consumer),
                &mut budget,
            )
            .unwrap()
            .unwrap();
            assert!(sites[0].arguments[0].presentation_wrapper.is_some());
            let projected = project_class_source_invoke_argument_edits(
                &typed_ast,
                &[ClassSourceInvokeArgumentPresentationCast {
                    call_bci: consumer.call_bci,
                    opcode: consumer.opcode,
                    target: consumer.target.clone(),
                    argument_index: 0,
                }],
                &[],
                &mut budget,
            )
            .unwrap()
            .unwrap();
            let StmtKind::Return { value: Some(call) } =
                &projected.projection.program.stmts[0].kind
            else {
                panic!("only the presentation wrapper is removed")
            };
            let ExprKind::Call { args, .. } = &call.kind else {
                panic!("the projected consumer remains a call")
            };
            let [argument] = args.as_slice() else {
                panic!("the consumer has one argument")
            };
            assert!(matches!(&argument.kind, ExprKind::Call { .. }));
        }
    }

    #[test]
    fn presentation_wrapper_projection_propagates_budget_and_cancellation() {
        let (ast, _, consumer) = object_presentation_wrapper_ast(false);
        let removal = [ClassSourceInvokeArgumentPresentationCast {
            call_bci: consumer.call_bci,
            opcode: consumer.opcode,
            target: consumer.target,
            argument_index: 0,
        }];
        let limits = Limits {
            ir_items: 0,
            ..Limits::default()
        };
        let mut budget = Budget::new(limits);
        assert!(matches!(
            project_class_source_invoke_argument_edits(&ast, &removal, &[], &mut budget),
            Err(crate::stop::StopReason::Budget { .. })
        ));

        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let mut budget = Budget::with_cancellation_token(
            Limits {
                ir_items: 1000,
                analysis_steps: 1000,
                elapsed_millis: u64::MAX,
                ..Limits::default()
            },
            cancellation,
        );
        assert!(matches!(
            project_class_source_invoke_argument_edits(&ast, &removal, &[], &mut budget),
            Err(crate::stop::StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn exact_site_and_source_cast_preserve_the_child_anchor_and_presentation() {
        let (ast, key) = ast();
        let mut budget = fresh_budget();
        let sites = class_source_invoke_ast_sites(&ast, std::slice::from_ref(&key), &mut budget)
            .unwrap()
            .unwrap();
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].caller, ast.projection.member);
        assert_eq!(
            sites[0].arguments[0].presented_type,
            Some(Type::Reference("java.lang.Object".to_owned()))
        );

        let original = match &ast.projection.program.stmts[0].kind {
            StmtKind::Return {
                value:
                    Some(Expr {
                        kind: ExprKind::Call { args, .. },
                        ..
                    }),
            } => args[0].clone(),
            _ => panic!("fixture call is the return value"),
        };
        let projected = project_class_source_invoke_argument_casts(
            &ast,
            &[
                ClassSourceInvokeArgumentCast {
                    call_bci: key.call_bci,
                    opcode: key.opcode,
                    target: key.target.clone(),
                    argument_index: 0,
                    ty: Type::Reference("java.lang.Number".to_owned()),
                },
                ClassSourceInvokeArgumentCast {
                    call_bci: key.call_bci,
                    opcode: key.opcode,
                    target: key.target.clone(),
                    argument_index: 1,
                    ty: Type::Reference("java.lang.Comparable".to_owned()),
                },
            ],
            &mut budget,
        )
        .unwrap()
        .unwrap();
        let ExprKind::Call { args, .. } = (match &projected.projection.program.stmts[0].kind {
            StmtKind::Return { value: Some(value) } => &value.kind,
            _ => panic!("projected call remains the return value"),
        }) else {
            panic!("call node remains a call")
        };
        let ExprKind::Cast { ty, value } = &args[0].kind else {
            panic!("source upcast is projected around the argument")
        };
        assert_eq!(ty, &Type::Reference("java.lang.Number".to_owned()));
        assert_eq!(
            args[0].presented,
            Some(Type::Reference("java.lang.Number".to_owned()))
        );
        assert_eq!(value.as_ref(), &original);
        assert_eq!(value.origin, original.origin);
        assert_eq!(value.presented, original.presented);
        assert!(matches!(
            &args[1].kind,
            ExprKind::Cast { ty: Type::Reference(name), .. } if name == "java.lang.Comparable"
        ));
    }

    #[test]
    fn cast_mutation_skips_a_foreign_method_node_with_the_same_bci_and_name() {
        let (ast, key) = ast();
        let mut source = (*ast.projection).clone();
        let foreign = PhysicalMethodId {
            name: JvmBytes(b"other".to_vec()),
            ..source.member.clone()
        };
        let call = Expr::new(
            ExprKind::Call {
                receiver: Some(Box::new(Expr::direct(
                    ExprKind::Local("this".to_owned()),
                    1,
                ))),
                name: "pick".to_owned(),
                args: vec![Expr::direct(ExprKind::Local("value".to_owned()), 2)],
            },
            OriginSet::new(crate::source_map::Origin::direct(3).in_method(&foreign)),
        );
        source.program.stmts.push(Stmt::new(
            StmtKind::Expr(call),
            OriginSet::new(crate::source_map::Origin::direct(8)),
        ));
        source.instruction_count = 2;
        source.instruction_bcis.push(8);
        let ast = ClassSourceMethodAst {
            projection: std::sync::Arc::new(source),
        };
        let mut budget = fresh_budget();
        let projected = project_class_source_invoke_argument_casts(
            &ast,
            &[ClassSourceInvokeArgumentCast {
                call_bci: key.call_bci,
                opcode: key.opcode,
                target: key.target,
                argument_index: 0,
                ty: Type::Reference("java.lang.Number".to_owned()),
            }],
            &mut budget,
        )
        .unwrap()
        .unwrap();
        let StmtKind::Expr(Expr {
            kind: ExprKind::Call { args, .. },
            ..
        }) = &projected.projection.program.stmts[1].kind
        else {
            panic!("foreign node remains in the AST")
        };
        assert!(matches!(&args[0].kind, ExprKind::Local(_)));
    }

    #[test]
    fn body_consumers_report_actual_returns_and_catch_scope_or_refuse_unknown_statements() {
        let (ast, _) = ast();
        let mut source = (*ast.projection).clone();
        let mut body = std::mem::take(&mut source.program.stmts);
        body.push(Stmt::new(
            StmtKind::FieldAssign {
                receiver: Some(Expr::direct(ExprKind::Local("this".to_owned()), 6)),
                name: "result".to_owned(),
                op: crate::ast::AssignOp::Assign,
                value: Expr::direct(ExprKind::Null, 6),
            },
            OriginSet::new(crate::source_map::Origin::direct(6)),
        ));
        source.program.stmts = vec![Stmt::new(
            StmtKind::Try {
                resources: Vec::new(),
                catches: vec![crate::ast::CatchClause {
                    ty: "java.lang.Exception".to_owned(),
                    name: "value".to_owned(),
                    body,
                }],
                body: Vec::new(),
                finally_body: None,
            },
            OriginSet::new(crate::source_map::Origin::direct(7)),
        )];
        source.instruction_count = 3;
        source.instruction_bcis.extend([6, 7]);
        let caught = ClassSourceMethodAst {
            projection: std::sync::Arc::new(source),
        };
        let mut budget = fresh_budget();
        let facts = class_source_method_body_consumers(&caught, &mut budget)
            .unwrap()
            .unwrap();
        assert_eq!(facts.returns.len(), 1);
        assert_eq!(facts.returns[0].catch_scopes.len(), 1);
        assert_eq!(facts.returns[0].catch_scopes[0].try_bci, 7);
        assert_eq!(facts.returns[0].catch_scopes[0].local_name, "value");
        assert!(matches!(
            facts.returns[0].expression.shape,
            ClassSourceAstExpressionShape::Call { .. }
        ));
        assert_eq!(facts.field_writes.len(), 1);
        assert_eq!(facts.field_writes[0].name, "result");
        assert_eq!(facts.field_writes[0].op, crate::ast::AssignOp::Assign);
        assert_eq!(facts.field_writes[0].catch_scopes[0].local_name, "value");
        assert!(facts.field_writes[0].value.null_literal);

        let mut unsupported = (*ast.projection).clone();
        unsupported.program.stmts.push(Stmt::new(
            StmtKind::Assign {
                name: "other".to_owned(),
                value: Expr::direct(ExprKind::Integer(1), 9),
            },
            OriginSet::new(crate::source_map::Origin::direct(9)),
        ));
        unsupported.instruction_count = 2;
        unsupported.instruction_bcis.push(9);
        let unsupported = ClassSourceMethodAst {
            projection: std::sync::Arc::new(unsupported),
        };
        let mut unsupported_budget = fresh_budget();
        assert!(
            class_source_method_body_consumers(&unsupported, &mut unsupported_budget)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn body_consumers_capture_only_exact_invocation_statements_and_their_catch_scope() {
        fn call_statement_source(caught: bool) -> ClassSourceMethodAst {
            let (ast, _) = ast();
            let mut source = (*ast.projection).clone();
            let statement = source.program.stmts.pop().expect("call return exists");
            let StmtKind::Return { value: Some(call) } = statement.kind else {
                panic!("fixture starts with a call return")
            };
            let invocation = Stmt::new(
                StmtKind::Expr(call),
                OriginSet::new(crate::source_map::Origin::direct(3)),
            );
            source.program.stmts = if caught {
                source.instruction_count = 2;
                source.instruction_bcis.push(4);
                vec![Stmt::new(
                    StmtKind::Try {
                        resources: Vec::new(),
                        catches: vec![crate::ast::CatchClause {
                            ty: "java.lang.RuntimeException".to_owned(),
                            name: "shadow".to_owned(),
                            body: vec![invocation],
                        }],
                        body: Vec::new(),
                        finally_body: None,
                    },
                    OriginSet::new(crate::source_map::Origin::direct(4)),
                )]
            } else {
                vec![invocation]
            };
            ClassSourceMethodAst {
                projection: std::sync::Arc::new(source),
            }
        }

        let direct = call_statement_source(false);
        let mut budget = fresh_budget();
        let facts = class_source_method_body_consumers(&direct, &mut budget)
            .unwrap()
            .unwrap();
        assert_eq!(facts.invocation_statements.len(), 1);
        assert_eq!(facts.invocation_statements[0].bci, 3);
        assert!(facts.invocation_statements[0].catch_scopes.is_empty());
        assert!(matches!(
            &facts.invocation_statements[0].expression.shape,
            ClassSourceAstExpressionShape::Call {
                name,
                argument_count: 2
            } if name == "pick"
        ));

        let caught = call_statement_source(true);
        let mut caught_budget = fresh_budget();
        let facts = class_source_method_body_consumers(&caught, &mut caught_budget)
            .unwrap()
            .unwrap();
        assert_eq!(facts.invocation_statements.len(), 1);
        assert_eq!(facts.invocation_statements[0].catch_scopes.len(), 1);
        assert_eq!(facts.invocation_statements[0].catch_scopes[0].try_bci, 4);
        assert_eq!(
            facts.invocation_statements[0].catch_scopes[0].local_name,
            "shadow"
        );

        let mut mismatched = (*direct.projection).clone();
        mismatched.call_targets[0].2 = crate::facts::CallTarget::new(
            crate::facts::InvokeKind::Virtual,
            "sample/Box",
            "other",
            "(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Object;",
            false,
        );
        let mismatched = ClassSourceMethodAst {
            projection: std::sync::Arc::new(mismatched),
        };
        let mut mismatch_budget = fresh_budget();
        assert!(
            class_source_method_body_consumers(&mismatched, &mut mismatch_budget)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn body_consumers_capture_if_return_and_throw_expression_shapes() {
        let (ast, _) = ast();
        let mut source = (*ast.projection).clone();
        source.program.stmts = vec![
            Stmt::new(
                StmtKind::ConstructorCall {
                    target: crate::ast::ConstructorTarget::Super,
                    args: Vec::new(),
                },
                OriginSet::new(crate::source_map::Origin::direct(1)),
            ),
            Stmt::new(
                StmtKind::If {
                    cond: Expr::direct(
                        ExprKind::Not {
                            value: Box::new(
                                Expr::direct(ExprKind::Local("fail".to_owned()), 2)
                                    .presenting(Type::Boolean),
                            ),
                        },
                        2,
                    ),
                    then_body: vec![Stmt::new(
                        StmtKind::Return {
                            value: Some(
                                Expr::direct(ExprKind::Local("value".to_owned()), 4)
                                    .presenting(Type::Reference("java.lang.Object".to_owned())),
                            ),
                        },
                        OriginSet::new(crate::source_map::Origin::direct(4)),
                    )],
                    else_body: vec![Stmt::new(
                        StmtKind::Throw {
                            value: Expr::direct(
                                ExprKind::New {
                                    ty: "java.lang.RuntimeException".to_owned(),
                                    qualifier: None,
                                    member_name: None,
                                    diamond: false,
                                    args: Vec::new(),
                                },
                                5,
                            ),
                        },
                        OriginSet::new(crate::source_map::Origin::direct(5)),
                    )],
                },
                OriginSet::new(crate::source_map::Origin::direct(2)),
            ),
        ];
        source.instruction_count = 5;
        source.instruction_bcis = vec![1, 2, 3, 4, 5];
        let ast = ClassSourceMethodAst {
            projection: std::sync::Arc::new(source),
        };
        let mut budget = fresh_budget();
        let facts = class_source_method_body_consumers(&ast, &mut budget)
            .unwrap()
            .unwrap();
        assert_eq!(facts.conditions.len(), 1);
        assert!(matches!(
            &facts.conditions[0].expression.shape,
            ClassSourceAstExpressionShape::BooleanNotLocal { local_name, presented_type: Some(Type::Boolean), .. }
                if local_name == "fail"
        ));
        assert_eq!(facts.returns.len(), 1);
        assert_eq!(
            facts.returns[0].expression.direct_local_name.as_deref(),
            Some("value")
        );
        assert_eq!(facts.throws.len(), 1);
        assert!(matches!(
            &facts.throws[0].expression.shape,
            ClassSourceAstExpressionShape::New { ty, argument_count: 0, qualified: false }
                if ty == "java.lang.RuntimeException"
        ));
        assert_eq!(facts.constructor_calls.len(), 1);
        assert_eq!(facts.constructor_calls[0].bci, 1);
        assert_eq!(
            facts.constructor_calls[0].target,
            crate::ast::ConstructorTarget::Super
        );
        assert!(facts.constructor_calls[0].arguments.is_empty());
    }

    #[test]
    fn body_consumers_refuse_resource_finally_and_propagate_budget_and_cancel() {
        let (ast, _) = ast();
        for (resources, finally_body) in [
            (
                vec![crate::ast::ResourceDecl {
                    ty: Type::Reference("java.lang.AutoCloseable".to_owned()),
                    name: "resource".to_owned(),
                    value: Expr::direct(ExprKind::Null, 8),
                }],
                None,
            ),
            (Vec::new(), Some(Vec::new())),
        ] {
            let mut source = (*ast.projection).clone();
            let body = std::mem::take(&mut source.program.stmts);
            source.program.stmts = vec![Stmt::new(
                StmtKind::Try {
                    resources,
                    catches: Vec::new(),
                    body,
                    finally_body,
                },
                OriginSet::new(crate::source_map::Origin::direct(7)),
            )];
            source.instruction_count = 2;
            source.instruction_bcis.push(7);
            let ast = ClassSourceMethodAst {
                projection: std::sync::Arc::new(source),
            };
            let mut budget = fresh_budget();
            assert!(
                class_source_method_body_consumers(&ast, &mut budget)
                    .unwrap()
                    .is_none()
            );
        }

        let mut exhausted = Budget::new(Limits {
            ir_items: 0,
            analysis_steps: 0,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        assert!(matches!(
            class_source_method_body_consumers(&ast, &mut exhausted),
            Err(crate::stop::StopReason::Budget { .. })
        ));
        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(Limits::default(), token);
        assert!(matches!(
            class_source_method_body_consumers(&ast, &mut cancelled),
            Err(crate::stop::StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn result_use_is_return_and_wrong_or_ambiguous_site_keys_refuse() {
        let (ast, key) = ast();
        let mut budget = fresh_budget();
        let uses = class_source_invoke_result_uses(&ast, &key, &[], &mut budget)
            .unwrap()
            .unwrap();
        assert_eq!(uses.len(), 1);
        assert!(matches!(
            uses[0].kind,
            ClassSourceInvokeResultUseKind::DirectReturn { consumer_bci: 3 }
        ));

        let mut caught = (*ast.projection).clone();
        let nested_return = std::mem::take(&mut caught.program.stmts);
        caught.program.stmts = vec![Stmt::new(
            StmtKind::Try {
                resources: Vec::new(),
                catches: vec![crate::ast::CatchClause {
                    ty: "java.lang.Exception".to_owned(),
                    name: "error".to_owned(),
                    body: nested_return,
                }],
                body: Vec::new(),
                finally_body: None,
            },
            OriginSet::new(crate::source_map::Origin::direct(7)),
        )];
        caught.instruction_count = 2;
        caught.instruction_bcis.push(7);
        let caught = ClassSourceMethodAst {
            projection: std::sync::Arc::new(caught),
        };
        let mut caught_budget = fresh_budget();
        let caught_uses = class_source_invoke_result_uses(&caught, &key, &[], &mut caught_budget)
            .unwrap()
            .unwrap();
        assert!(matches!(
            caught_uses[0].kind,
            ClassSourceInvokeResultUseKind::DirectReturn { consumer_bci: 3 }
        ));

        let mut wrong = key.clone();
        wrong.opcode = 0xb8;
        assert!(
            class_source_invoke_ast_sites(&ast, &[wrong], &mut budget)
                .unwrap()
                .is_none()
        );
        assert!(
            class_source_invoke_ast_sites(&ast, &[key.clone(), key], &mut budget)
                .unwrap()
                .is_none()
        );

        let mut duplicated = (*ast.projection).clone();
        duplicated
            .program
            .stmts
            .push(duplicated.program.stmts[0].clone());
        let duplicated = ClassSourceMethodAst {
            projection: std::sync::Arc::new(duplicated),
        };
        let mut duplicate_budget = fresh_budget();
        assert!(
            class_source_invoke_ast_sites(
                &duplicated,
                &[uses[0].producer.clone()],
                &mut duplicate_budget
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn same_class_invoke_inventory_requires_complete_exact_sites() {
        let (ast, keys) = two_site_ast();
        let mut budget = fresh_budget();
        assert_eq!(
            class_source_same_class_invoke_inventory_matches(
                &ast,
                "sample/Box",
                &keys,
                &mut budget
            )
            .unwrap(),
            Some(())
        );

        let mut omitted_one = fresh_budget();
        assert!(
            class_source_same_class_invoke_inventory_matches(
                &ast,
                "sample/Box",
                &keys[..1],
                &mut omitted_one
            )
            .unwrap()
            .is_none()
        );
        let mut omitted_all = fresh_budget();
        assert!(
            class_source_same_class_invoke_inventory_matches(
                &ast,
                "sample/Box",
                &[],
                &mut omitted_all
            )
            .unwrap()
            .is_none()
        );

        let mut wrong_target = keys.clone();
        wrong_target[0].target = crate::facts::CallTarget::new(
            crate::facts::InvokeKind::Virtual,
            "sample/Box",
            "different",
            keys[0].target.descriptor(),
            false,
        );
        let mut wrong_target_budget = fresh_budget();
        assert!(
            class_source_same_class_invoke_inventory_matches(
                &ast,
                "sample/Box",
                &wrong_target,
                &mut wrong_target_budget
            )
            .unwrap()
            .is_none()
        );

        let mut wrong_opcode = keys.clone();
        wrong_opcode[0].opcode = 0xb8;
        let mut wrong_opcode_budget = fresh_budget();
        assert!(
            class_source_same_class_invoke_inventory_matches(
                &ast,
                "sample/Box",
                &wrong_opcode,
                &mut wrong_opcode_budget
            )
            .unwrap()
            .is_none()
        );

        let mut foreign_owner = keys.clone();
        foreign_owner[0].target = crate::facts::CallTarget::new(
            crate::facts::InvokeKind::Virtual,
            "sample/Other",
            "pick",
            keys[0].target.descriptor(),
            false,
        );
        let mut foreign_owner_budget = fresh_budget();
        assert!(
            class_source_same_class_invoke_inventory_matches(
                &ast,
                "sample/Box",
                &foreign_owner,
                &mut foreign_owner_budget
            )
            .unwrap()
            .is_none()
        );

        let mut duplicate_expected = keys.clone();
        duplicate_expected.push(keys[0].clone());
        let mut duplicate_expected_budget = fresh_budget();
        assert!(
            class_source_same_class_invoke_inventory_matches(
                &ast,
                "sample/Box",
                &duplicate_expected,
                &mut duplicate_expected_budget
            )
            .unwrap()
            .is_none()
        );

        let mut duplicate_actual_source = (*ast.projection).clone();
        let duplicate_entry = duplicate_actual_source.call_targets[0].clone();
        duplicate_actual_source.call_targets.push(duplicate_entry);
        let duplicate_actual = ClassSourceMethodAst {
            projection: std::sync::Arc::new(duplicate_actual_source),
        };
        let mut duplicate_actual_budget = fresh_budget();
        assert!(
            class_source_same_class_invoke_inventory_matches(
                &duplicate_actual,
                "sample/Box",
                &keys,
                &mut duplicate_actual_budget
            )
            .unwrap()
            .is_none()
        );

        let mut unknown_opcode_source = (*ast.projection).clone();
        unknown_opcode_source.call_targets[0].1 = 0xff;
        let unknown_opcode = ClassSourceMethodAst {
            projection: std::sync::Arc::new(unknown_opcode_source),
        };
        let mut unknown_opcode_budget = fresh_budget();
        assert!(
            class_source_same_class_invoke_inventory_matches(
                &unknown_opcode,
                "sample/Box",
                &keys,
                &mut unknown_opcode_budget
            )
            .unwrap()
            .is_none()
        );

        let mut incomplete_source = (*ast.projection).clone();
        incomplete_source.complete_code = false;
        let incomplete = ClassSourceMethodAst {
            projection: std::sync::Arc::new(incomplete_source),
        };
        let mut incomplete_budget = fresh_budget();
        assert!(
            class_source_same_class_invoke_inventory_matches(
                &incomplete,
                "sample/Box",
                &keys,
                &mut incomplete_budget
            )
            .unwrap()
            .is_none()
        );

        let mut duplicate_bci_source = (*ast.projection).clone();
        duplicate_bci_source.instruction_bcis.push(3);
        duplicate_bci_source.instruction_count += 1;
        let duplicate_bci = ClassSourceMethodAst {
            projection: std::sync::Arc::new(duplicate_bci_source),
        };
        let mut duplicate_bci_budget = fresh_budget();
        assert!(
            class_source_same_class_invoke_inventory_matches(
                &duplicate_bci,
                "sample/Box",
                &keys,
                &mut duplicate_bci_budget
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn same_class_invoke_inventory_accepts_complete_empty_inventory() {
        let (ast, _) = ast();
        let mut source = (*ast.projection).clone();
        source.program.stmts.clear();
        source.call_targets.clear();
        source.instruction_bcis = vec![1];
        source.instruction_count = 1;
        let no_calls = ClassSourceMethodAst {
            projection: std::sync::Arc::new(source),
        };
        let mut budget = fresh_budget();
        assert_eq!(
            class_source_same_class_invoke_inventory_matches(
                &no_calls,
                "sample/Box",
                &[],
                &mut budget
            )
            .unwrap(),
            Some(())
        );
    }

    #[test]
    fn same_class_invoke_inventory_propagates_budget_and_cancellation() {
        let (ast, keys) = two_site_ast();
        let mut exhausted = Budget::new(Limits {
            ir_items: 0,
            analysis_steps: 0,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        assert!(matches!(
            class_source_same_class_invoke_inventory_matches(
                &ast,
                "sample/Box",
                &keys,
                &mut exhausted
            ),
            Err(crate::stop::StopReason::Budget { .. })
        ));

        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(Limits::default(), token);
        assert!(matches!(
            class_source_same_class_invoke_inventory_matches(
                &ast,
                "sample/Box",
                &keys,
                &mut cancelled
            ),
            Err(crate::stop::StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn invoke_site_resolver_refuses_zero_ast_match_and_foreign_caller() {
        let (ast, key) = ast();
        let mut no_call_source = (*ast.projection).clone();
        no_call_source.program.stmts.clear();
        let no_call_ast = ClassSourceMethodAst {
            projection: std::sync::Arc::new(no_call_source),
        };
        let mut no_call_budget = fresh_budget();
        assert!(
            class_source_invoke_ast_sites(&no_call_ast, &[key.clone()], &mut no_call_budget)
                .unwrap()
                .is_none()
        );

        let mut foreign_source = (*ast.projection).clone();
        let foreign_method = PhysicalMethodId {
            name: JvmBytes(b"foreign".to_vec()),
            ..foreign_source.member.clone()
        };
        let StmtKind::Return { value: Some(call) } = &mut foreign_source.program.stmts[0].kind
        else {
            panic!("fixture call is the return value")
        };
        call.origin =
            OriginSet::new(crate::source_map::Origin::direct(3).in_method(&foreign_method));
        let foreign_ast = ClassSourceMethodAst {
            projection: std::sync::Arc::new(foreign_source),
        };
        let mut foreign_budget = fresh_budget();
        assert!(
            class_source_invoke_ast_sites(&foreign_ast, &[key], &mut foreign_budget)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn required_site_scan_propagates_budget_stop_and_cancellation() {
        let (ast, key) = ast();
        let mut exhausted = Budget::new(Limits {
            ir_items: 0,
            analysis_steps: 0,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        assert!(matches!(
            class_source_invoke_ast_sites(&ast, &[key.clone()], &mut exhausted),
            Err(crate::stop::StopReason::Budget { .. })
        ));

        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(Limits::default(), token);
        assert!(matches!(
            class_source_invoke_ast_sites(&ast, &[key], &mut cancelled),
            Err(crate::stop::StopReason::Cancelled { .. })
        ));
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
        parameter_names: Vec::new(),
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
        parameter_names: Vec::new(),
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

#[cfg(test)]
mod lambda_helper_instruction_coverage_tests {
    use super::*;
    use crate::ast::{Expr, ExprKind, Stmt, StmtKind};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalVariant, SnapshotId,
    };
    use std::collections::BTreeMap;

    fn helper(instruction_bcis: Vec<u32>) -> ClassSourceMethodAstSource {
        let member = jarde_reader::model::PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: SnapshotId("lambda-coverage-test".to_owned()),
                },
                class_bytes: ClassBytesId {
                    digest: Digest("lambda-coverage-test".to_owned()),
                    length: 1,
                },
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"lambda$test$0".to_vec()),
            descriptor: JvmBytes(b"()I".to_vec()),
        };
        let origin = OriginSet::new(crate::source_map::Origin::direct(0));
        let program = build::Program {
            stmts: vec![Stmt::new(
                StmtKind::Return {
                    value: Some(Expr::new(ExprKind::Integer(1), origin.clone())),
                },
                origin,
            )],
            field_increments: BTreeMap::new(),
            statements: 1,
            ragged: false,
            lambdas: Vec::new(),
            accessors: Vec::new(),
            array_constructor_sites: Vec::new(),
            lambda_refusals: Vec::new(),
            accessor_refusals: Vec::new(),
            lambdas_presented: 0,
            accessors_presented: 0,
        };
        ClassSourceMethodAstSource {
            program,
            member,
            current_class: None,
            nested_class_members: Vec::new(),
            parameter_names: Vec::new(),
            parameter_slots: Vec::new(),
            complete_code: true,
            has_exception_handlers: false,
            instruction_count: instruction_bcis.len(),
            instruction_bcis,
            call_targets: Vec::new(),
            anonymous_constructor_initializer_bci: None,
            generic_call_init: None,
        }
    }

    #[test]
    fn equal_node_and_instruction_counts_do_not_cover_an_extra_physical_instruction() {
        let mismatched = helper(vec![0, 1]);
        assert_eq!(program_node_count(&mismatched.program), 2);
        assert_eq!(mismatched.instruction_count, 2);
        assert!(
            !lambda_helper_instruction_coverage(&mismatched),
            "an extra physical effect/instruction with coincident counts must refuse the helper"
        );

        let exact = helper(vec![0]);
        assert!(lambda_helper_instruction_coverage(&exact));
    }
}

#[cfg(test)]
mod anonymous_capture_projection_tests {
    use super::*;
    use crate::ast::{Expr, ExprKind, Stmt, StmtKind};
    use jarde_reader::{
        budget::{Budget, CancellationToken, Limits},
        model::{
            ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
            PhysicalMethodId, PhysicalVariant, SnapshotId,
        },
    };
    use std::collections::BTreeMap;

    fn ast(expression_bci: u32, field_name: &str, copies: usize) -> ClassSourceMethodAst {
        let owner = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: SnapshotId("capture-test".to_owned()),
            },
            class_bytes: ClassBytesId {
                digest: Digest("capture-test".to_owned()),
                length: 1,
            },
            variant: PhysicalVariant::Base,
        };
        let member = PhysicalMethodId {
            owner,
            name: JvmBytes(b"run".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        };
        let expression = || {
            Expr::direct(
                ExprKind::Field {
                    receiver: Box::new(Expr::direct(ExprKind::Local("this".to_owned()), 2)),
                    name: field_name.to_owned(),
                },
                expression_bci,
            )
        };
        let stmts = (0..copies)
            .map(|_| {
                Stmt::new(
                    StmtKind::Expr(expression()),
                    OriginSet::new(crate::source_map::Origin::direct(expression_bci)),
                )
            })
            .collect();
        let program = build::Program {
            stmts,
            field_increments: BTreeMap::new(),
            statements: copies,
            ragged: false,
            lambdas: Vec::new(),
            accessors: Vec::new(),
            array_constructor_sites: Vec::new(),
            lambda_refusals: Vec::new(),
            accessor_refusals: Vec::new(),
            lambdas_presented: 0,
            accessors_presented: 0,
        };
        ClassSourceMethodAst {
            projection: std::sync::Arc::new(ClassSourceMethodAstSource {
                program,
                member,
                current_class: None,
                nested_class_members: Vec::new(),
                parameter_names: Vec::new(),
                parameter_slots: Vec::new(),
                complete_code: false,
                has_exception_handlers: false,
                instruction_count: 0,
                instruction_bcis: Vec::new(),
                call_targets: Vec::new(),
                anonymous_constructor_initializer_bci: None,
                generic_call_init: None,
            }),
        }
    }

    fn read(ast: &ClassSourceMethodAst, bci: u32, field_name: &str) -> ProvedCapturedOuterRead {
        ProvedCapturedOuterRead {
            method: ast.projection.member.clone(),
            read_bci: bci,
            field_owner: "Inner$1".to_owned(),
            field_name: field_name.to_owned(),
            field_descriptor: "LInner;".to_owned(),
            outer_internal_name: "Inner".to_owned(),
            outer_source_name: "Inner".to_owned(),
            constructor: PhysicalMethodId {
                owner: ast.projection.member.owner.clone(),
                name: JvmBytes(b"<init>".to_vec()),
                descriptor: JvmBytes(b"(LInner;)V".to_vec()),
            },
            constructor_write_bci: 2,
        }
    }

    fn test_budget() -> Budget {
        Budget::new(Limits {
            ir_items: 100,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        })
    }

    #[test]
    fn exact_same_run_capture_read_projects_and_preserves_its_bci() {
        let ast = ast(12, "this$0", 1);
        let proof = read(&ast, 12, "this$0");
        let mut budget = test_budget();
        let projected = project_class_source_captured_outer_reads(&ast, &[proof], &mut budget)
            .unwrap()
            .expect("one exact field expression projects");
        assert!(matches!(
            projected.projection.program.stmts[0].kind,
            StmtKind::Expr(Expr { kind: ExprKind::QualifiedThis { ref qualifier }, .. }) if qualifier == "Inner"
        ));
        assert_eq!(
            projected.projection.program.stmts[0].origin.primary().bci(),
            12
        );
    }

    #[test]
    fn duplicate_or_wrong_capture_ast_matches_refuse_the_whole_projection() {
        let duplicate = ast(12, "this$0", 2);
        let proof = read(&duplicate, 12, "this$0");
        let mut budget = test_budget();
        assert!(
            project_class_source_captured_outer_reads(&duplicate, &[proof], &mut budget)
                .unwrap()
                .is_none()
        );

        let mismatch = ast(12, "other", 1);
        let proof = read(&mismatch, 12, "this$0");
        let mut budget = test_budget();
        assert!(
            project_class_source_captured_outer_reads(&mismatch, &[proof], &mut budget)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn ast_walk_budget_and_cancellation_stop_before_returning_a_projection() {
        let ast = ast(12, "this$0", 1);
        let proof = read(&ast, 12, "this$0");
        let mut limits = test_budget().limits().clone();
        limits.ir_items = 1; // the handoff fits, but charging the whole AST before clone does not
        let mut bounded = Budget::new(limits);
        assert!(matches!(
            project_class_source_captured_outer_reads(&ast, &[proof.clone()], &mut bounded),
            Err(crate::stop::StopReason::Budget {
                dimension: jarde_reader::budget::CountedBudgetDimension::IrItems,
                ..
            })
        ));
        assert!(matches!(
            ast.projection.program.stmts[0].kind,
            StmtKind::Expr(Expr {
                kind: ExprKind::Field { .. },
                ..
            })
        ));

        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(test_budget().limits().clone(), token);
        assert!(matches!(
            project_class_source_captured_outer_reads(&ast, &[proof], &mut cancelled),
            Err(crate::stop::StopReason::Cancelled { .. })
        ));
        assert!(matches!(
            ast.projection.program.stmts[0].kind,
            StmtKind::Expr(Expr {
                kind: ExprKind::Field { .. },
                ..
            })
        ));
    }
}

#[cfg(test)]
mod raw_receiver_site_budget_tests {
    use super::*;
    use crate::ast::{AssignOp, Expr, ExprKind, Stmt, StmtKind};
    use jarde_jvm::engine::analyze_method_ir;
    use jarde_jvm::environment::ResolutionEnvironment;
    use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
    use jarde_jvm::method_ir::Definition;
    use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
    use jarde_reader::budget::{Budget, CancellationToken, Limits};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
        MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
        RuntimeView,
    };
    use std::collections::{BTreeMap, BTreeSet};

    const CLASS: &[u8] = include_bytes!(
        "../../../tests/fixtures/raw-receiver-field-selection/SyntheticAccessorGuard.class"
    );
    const WRITE_BCI: u32 = 2;

    fn limits() -> Limits {
        Limits {
            input_bytes: 1 << 20,
            archive_entries: 100,
            entry_bytes: 1 << 20,
            read_bytes: 1 << 20,
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            output_bytes: 1 << 20,
            class_headers: 10,
            method_bodies: 10,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            nested_depth: 16,
            dependency_depth: 8,
            elapsed_millis: u64::MAX,
        }
    }

    fn unpresented_init() -> InitRecord {
        InitRecord {
            bci: None,
            target: None,
            class: None,
            declared: None,
            presented: false,
            refusal: None,
        }
    }

    #[test]
    fn receiver_collector_stops_at_its_ast_scan_budget_and_cancellation() {
        let mut setup_budget = Budget::new(limits());
        let snapshot =
            ArtifactSnapshot::open(ArtifactInput::bytes(CLASS.to_vec()), &mut setup_budget)
                .expect("the existing field-write fixture opens");
        let method = PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: ClassBytesId {
                    digest: Digest(blake3::hash(CLASS).to_hex().to_string()),
                    length: u64::try_from(CLASS.len()).expect("fixture length fits"),
                },
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"access$set".to_vec()),
            descriptor: JvmBytes(b"(LSyntheticAccessorGuard;Ljava/lang/Object;)V".to_vec()),
        };
        let domain = LoadDomain {
            loader: LoaderId("app".to_owned()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        };
        let analysis = analyze_method_ir(
            &[snapshot.clone()],
            &MethodAnalysisRequest {
                environment: ResolutionEnvironment {
                    runtime: RuntimeView {
                        physical: PhysicalView {
                            snapshot: snapshot.id().clone(),
                            scope: PhysicalScope::SnapshotAll,
                        },
                        profile: RuntimeProfile {
                            java_release: 8,
                            multi_release: MultiReleasePolicy::Disabled,
                            layout: LayoutMode::Generic,
                        },
                        load_domain: domain.clone(),
                    },
                    domains: vec![domain],
                    providers: Vec::new(),
                },
                method,
                stages: AnalysisStage::ALL.to_vec(),
            },
            &mut setup_budget,
        )
        .expect("the existing field-write method analyzes");
        let facts = crate::facts::RecoveryFacts::new(
            crate::facts::MethodFacts::new(
                "access$set",
                "(LSyntheticAccessorGuard;Ljava/lang/Object;)V",
                2,
            )
            .with_access_flags(0x0009)
            .with_declaring_class(crate::facts::DeclaringClass::new(
                "SyntheticAccessorGuard",
                0x1031,
            )),
        );
        let request = RecoveryRequest::new(analysis.ir(), &facts, crate::pass::JAVA_8);
        let ir = request.ir;
        let ssa = ir.ssa().expect("the analyzed method has SSA");
        let canonical = ir.canonical().expect("the analyzed method has a CFG");
        let code = ir.code().expect("the analyzed method has code facts");
        let operations = Operations::of(code, ir.constant_pool());
        let declaring = request
            .facts
            .method()
            .declaring_class()
            .expect("the test supplies declaring-class facts");
        let element_receiver_type = |value| build::element_receiver_type(ssa, &operations, value);
        let fields = field::plan(
            ssa,
            &operations,
            &element_receiver_type,
            Some(declaring),
            "access$set",
            "(LSyntheticAccessorGuard;Ljava/lang/Object;)V",
            ir.class_fields(),
            &[],
            &mut setup_budget,
        )
        .expect("the fixture's own field write is claimed");
        let shape = fields
            .claim(WRITE_BCI)
            .map(|(_, shape)| shape)
            .expect("the fixture's putfield is at BCI 2");
        let receiver_value = shape.receiver.expect("putfield has an instance receiver");
        let Definition::Instruction {
            bci: receiver_bci, ..
        } = ssa.value(receiver_value).def()
        else {
            panic!("the fixture receiver is loaded from a local");
        };
        let origin = |bci| OriginSet::new(crate::source_map::Origin::direct(bci));
        let presented = BTreeSet::from([WRITE_BCI]);
        let frames = ir.frames().expect("the analyzed method has frames");
        let regions: [crate::region::Region; 0] = [];
        let reuse = crate::reuse::plan(
            ssa,
            canonical,
            &operations,
            &regions,
            u16::try_from(frames.locals_slots()).expect("fixture slots fit"),
            facts.method().parameters(),
            facts.debug_locals(),
            &build::resource_slots(&regions),
            &mut setup_budget,
        )
        .expect("the fixture's local reuse plan is complete");
        let names = NameTable::build_with_reserved(
            facts.method().parameters(),
            u16::try_from(frames.locals_slots()).expect("fixture slots fit"),
            reuse.evidence(),
            &BTreeSet::new(),
        );
        let receiver_name = names
            .whole(0)
            .expect("static formal slot 0 has one rendered name")
            .text()
            .to_owned();
        let program = build::Program {
            stmts: vec![Stmt::new(
                StmtKind::FieldAssign {
                    receiver: Some(Expr::new(
                        ExprKind::Local(receiver_name),
                        origin(*receiver_bci),
                    )),
                    name: "value".to_owned(),
                    op: AssignOp::Assign,
                    value: Expr::direct(ExprKind::Boolean(true), WRITE_BCI - 1),
                },
                origin(WRITE_BCI),
            )],
            field_increments: BTreeMap::new(),
            statements: 1,
            ragged: false,
            lambdas: Vec::new(),
            accessors: Vec::new(),
            array_constructor_sites: Vec::new(),
            lambda_refusals: Vec::new(),
            accessor_refusals: Vec::new(),
            lambdas_presented: 0,
            accessors_presented: 0,
        };

        let mut successful = Budget::new(limits());
        let (receivers, accessors) = same_class_field_receiver_sites(
            &program,
            &fields,
            &presented,
            &request,
            &names,
            &reuse,
            &unpresented_init(),
            &operations,
            ssa,
            &mut successful,
        )
        .expect("the actual candidate reaches and completes the collector");
        assert!(accessors.is_empty());
        assert_eq!(receivers.len(), 1);
        assert_eq!(receivers[0].bci, WRITE_BCI);
        assert_eq!(
            receivers[0].source,
            ClassSourceFieldReceiverSource::Parameter { slot: 0 }
        );

        let mut duplicate_site = program.clone();
        duplicate_site.stmts.push(program.stmts[0].clone());
        let (receivers, _) = same_class_field_receiver_sites(
            &duplicate_site,
            &fields,
            &presented,
            &request,
            &names,
            &reuse,
            &unpresented_init(),
            &operations,
            ssa,
            &mut Budget::new(limits()),
        )
        .expect("a duplicated AST receipt is rejected as ambiguous");
        assert!(receivers.is_empty());

        let mut wrong_receiver_origin = program.clone();
        let StmtKind::FieldAssign {
            receiver: Some(receiver),
            ..
        } = &mut wrong_receiver_origin.stmts[0].kind
        else {
            unreachable!("the test builds a field assignment");
        };
        receiver.origin = origin(WRITE_BCI - 1);
        let (receivers, _) = same_class_field_receiver_sites(
            &wrong_receiver_origin,
            &fields,
            &presented,
            &request,
            &names,
            &reuse,
            &unpresented_init(),
            &operations,
            ssa,
            &mut Budget::new(limits()),
        )
        .expect("a receiver from another load site is not a fact");
        assert!(receivers.is_empty());

        let instruction_count: usize = ssa
            .blocks()
            .iter()
            .map(|block| block.instructions().len())
            .sum();
        let mut constrained_limits = limits();
        constrained_limits.ir_items =
            u64::try_from(instruction_count).expect("fixture instruction count fits");
        let mut constrained = Budget::new(constrained_limits);
        let stopped = same_class_field_receiver_sites(
            &program,
            &fields,
            &presented,
            &request,
            &names,
            &reuse,
            &unpresented_init(),
            &operations,
            ssa,
            &mut constrained,
        )
        .expect_err("the first AST node is beyond the exact SSA-index budget");
        assert_eq!(
            stopped,
            StopReason::Budget {
                dimension: jarde_reader::budget::CountedBudgetDimension::IrItems,
                written: 0,
                limit: u64::try_from(instruction_count).expect("fixture instruction count fits"),
                at: Some(WRITE_BCI),
            }
        );

        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(limits(), token);
        let stopped = same_class_field_receiver_sites(
            &program,
            &fields,
            &presented,
            &request,
            &names,
            &reuse,
            &unpresented_init(),
            &operations,
            ssa,
            &mut cancelled,
        )
        .expect_err("the real candidate's collector poll propagates cancellation");
        assert_eq!(
            stopped,
            StopReason::Cancelled {
                at: Some(WRITE_BCI)
            }
        );
    }
}
