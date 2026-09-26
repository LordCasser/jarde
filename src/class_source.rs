//! One class file presented as Java source: its declaration, its fields and its member bodies.
//!
//! # What this module owns, and what it does not
//!
//! The reads belong to the layers below, and this module performs none of them itself: the class's
//! declaration, its fields and its method records are the class read [`crate::Engine::class_source`]
//! binds and walks, and every member body is one analysis run with the presentation of that same
//! run's payload — the run [`crate::Engine::recover_method`] performs, on the class that entry
//! prepares once for the whole presentation. What lives here is the **assembly**: the Java spelling
//! of one class from those results, and the report that publishes the facts *and* the spelling
//! beside each other, so a caller can check one against the other instead of trusting the text.
//!
//! # Every member says what it is
//!
//! A class file is not source and this text does not claim to be one; what it must never do is let
//! a member look like something it is not:
//!
//! * a member whose declaration carries no `Code` attribute — an `abstract` or `native` member — is
//!   written as the declaration it is (`public abstract void run();`) with a `// jarde:` marker
//!   above it, and no run is performed for it (there is no body to run on);
//! * a member whose recovery run stopped, produced an artifact holding no statement, or was refused
//!   keeps its declaration and a block whose whole content is the marker that says so: an empty
//!   body is never written for a body that was not recovered;
//! * a member whose raw descriptor is not one this presentation can read as a Java type gets a
//!   marker and no declaration at all, and no run is performed for it either.
//!
//! `// jarde:` is the one marker prefix, so a reader — or a `grep` — finds every place where the
//! text is not a full recovery, and the same lines are published per member in the report
//! ([`ClassSourceField::markers`], [`ClassSourceMethod::markers`]).
//!
//! # What the text is not
//!
//! It is not a compilable project, and it does not claim to be one. The one `package` statement it
//! writes is the package the class's **own** internal name states — `a/b/C` declares `a.b`, and a
//! name with no `/` is the default package and declares none — which is a statement about that name
//! and not about the directory the class was found in. No `import` is resolved or elided, no
//! resource is parsed and no annotation type is resolved outside this class file. An isolated
//! physical member remains named `p.Outer$Inner`; only a separately proved root/child relation,
//! capture, call census and complete bodies can publish a nested root source unit.
//!
//! What the text writes about a declaration is what the class file's own attributes say about it,
//! and never an inference from anything else:
//!
//! * a field's `ConstantValue` (JVMS 4.7.2) is its own initializer; an ordinary interface's
//!   `<clinit>` value reaches a declaration only after the complete field group, evaluation order
//!   and runtime initialization phase have been proved. The original field and method reports stay
//!   available beside that source projection;
//! * a member's `Exceptions` (JVMS 4.7.4) is its `throws` clause, in the attribute's own order, and
//!   no exception is ever inferred from an `athrow`;
//! * a member's `AnnotationDefault` (JVMS 4.7.22) is its `default <literal>`.
//!
//! A value this presentation has no literal for — an unsupported NaN bit pattern, a nested
//! annotation with an invalid source name, or a floating-point field initializer — is written as
//! no initializer and no default rather than guessed. Names are spelled
//! the one way source spells them (`/` becomes `.`, and nothing else), and the raw bytes of every
//! name and descriptor stay published in the report.
//!
//! # Determinism
//!
//! The text is a pure function of the class bytes, the request and the budget's *limits*: no
//! ordering comes from a map, no name is invented twice, no timestamp and no count from a previous
//! run is written. The one field two runs of the same request may differ in is the elapsed clock in
//! `usage`, which is not part of the text.

use crate::facade::{
    ClassDeclarationFacts, ClassDeclarationItem, ClassRef, EnvironmentRequest, FieldItem,
    MethodItem,
};
use crate::{
    AnalysisStage, AttributeShell, Budget, Coverage, CpEntryFacts, Diagnostic, Error,
    ExecutionReport, JvmBytes, JvmString, Limits, MemberHeader, MemberTableStop, NoBodyKind,
    PhysicalDefinitionId, PhysicalMethodId, PhysicalView, Result, TerminationReason, UsageSnapshot,
    attribute_facts, budget_dimension_code,
};
use jarde_java::report::{GenericConstructorCandidate, GenericReturnCandidate, GenericReturnValue};
use jarde_java::{
    LocalVariable, NameTable, RecoveryContent, RecoveryFacts, RecoveryReport, SlotEvidence,
    alias_for, comment_text, escape_string, is_java_identifier, type_of_component,
};
use jarde_jvm::method_ir::parameter_positions;
use jarde_reader::budget::CountedBudgetDimension;
use jarde_reader::classfile::{
    Base, CpEntryKind, DescriptorComponent, DescriptorKind, ElementConstantTag, ElementValueFacts,
    EnclosingMethodFacts, InnerClassFacts, TypeAnnotationFacts, TypePathEntry, cp_entry,
    descriptor_facts,
};
use jarde_reader::signature::{
    ClassSignatureErasureProof, SignatureType, TypeArgument, TypeParameterErasure,
    parse_class_signature, parse_field_signature, parse_method_signature,
    prove_class_signature_erasure, prove_field_signature_erasure_with_class_scope,
    prove_method_signature_erasure_with_class_scope,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------
// The request
// ---------------------------------------------------------------------------------------------

/// One class-source request: the class to present, and the environment it is read in.
///
/// The two fields are the method operations' own shape ([`crate::MethodOperationRequest`]) for the
/// class-level target, so a caller that can name a class for [`crate::Engine::class_view`] can name
/// one here with the same [`ClassRef`] and the same [`EnvironmentRequest`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceRequest {
    /// The class to present: a friendly name searched with the navigation rules over the
    /// environment's scope, or the physical definition a listing handed back. Several definitions
    /// of one name are all returned as candidates and nothing is presented (see
    /// [`crate::OperationOutcome`]).
    pub class: ClassRef,
    /// The environment this request runs in: the physical view its scope is read under, the policy
    /// that states its roots, and the runtime profile every member's recovery is presented under.
    pub environment: EnvironmentRequest,
}

/// Typed class-level nesting facts supplied to class-source assembly from the selected class read.
///
/// These remain an internal handoff: they are not method IR or part of the serialized class-source
/// report. Later assembly proofs may consume them without decoding the class attributes again.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ClassSourceAssemblyContext {
    pub(crate) inner_classes: Vec<InnerClassFacts>,
    pub(crate) enclosing_method: Option<EnclosingMethodFacts>,
}

/// Read the nesting facts the selected class contributes to assembly through the reader's typed
/// attribute parser. The caller supplies only the selected class's own nesting shells.
pub(crate) fn read_class_source_assembly_context(
    bytes: &[u8],
    shells: &[AttributeShell],
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<ClassSourceAssemblyContext> {
    let facts = attribute_facts(bytes, shells, pool, budget)?;
    Ok(ClassSourceAssemblyContext {
        inner_classes: facts.inner_classes,
        enclosing_method: facts.enclosing_method,
    })
}

// ---------------------------------------------------------------------------------------------
// The report
// ---------------------------------------------------------------------------------------------

/// The class-level declaration of one class-source report, and the Java spelling the assembled text
/// opens with.
///
/// The item is the one the class read published — the same read
/// [`crate::ClassViewReport::declaration`] answers with — so the physical definition, the facts the
/// class file's own header states, whether the entry's path agrees with the declared name and where
/// the member walk stopped all keep the meaning they have there. `name` and `declaration` add the
/// spelling, and nothing else: they are derived from `item.declaration` alone.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceDeclaration {
    /// The class-level item of the read this report presents.
    pub item: ClassDeclarationItem,
    /// The class's own name, as its declaration spells it: the simple name `this_class` states —
    /// everything after the last `/` of the internal name, with `$` **kept**, so `p/Outer$Inner` is
    /// `Outer$Inner` and `p/More17$1` is `More17$1`.
    ///
    /// The package the rest of the name states is written as the text's own `package` statement (see
    /// [`ClassSourceReport::text`]) and never as a qualification of this name, and a constructor is
    /// spelled with this name rather than with the internal one. The internal name itself stays in
    /// [`Self::item`], so the bytes the class file states and the one line spelled from them are
    /// published side by side. A class `Signature` can replace the header spelling only after its
    /// variable scope and physical parent identities have been proved; the raw attribute and any
    /// refusal remain alongside the projected text.
    pub name: String,
    /// The declaration line the assembled text opens with, without its opening brace (for example
    /// `public class Base extends java.lang.Object implements p.Marker`).
    ///
    /// Only the flags Java source spells are written (`public`/`protected`/`private`, `abstract`,
    /// `final`, `strictfp`); the raw `access_flags` stay in [`Self::item`], and the bits that are not
    /// source keywords (`ACC_SUPER`, `ACC_SYNTHETIC`, `ACC_MODULE`) are not invented into text.
    /// An ordinary interface's `extends` list is the interfaces the class file declares — its
    /// `super_class` is `java/lang/Object` by the format's own rule, so it is not written as a
    /// superclass. A canonical annotation's implicit `java/lang/annotation/Annotation` remains in
    /// [`Self::item`] and is not written into its `@interface` header. The name of every type the
    /// header does write is the internal name the pool states, `/` written as `.`; only the class's
    /// **own** name is the simple one (see [`Self::name`]).
    pub declaration: String,
    /// Class-level Runtime*Annotations shells and the complete value facts read from each.
    pub annotation_attributes: Vec<ClassAnnotationAttribute>,
    /// Complete class-level annotation uses this presentation can spell, in physical attribute
    /// order and then attribute entry order.
    pub annotation_uses: Vec<String>,
    /// One refusal for every class-level annotation that cannot be faithfully represented in Java.
    pub annotation_refusals: Vec<String>,
    /// The class's own generic Signature, when its attribute could be read.
    pub generic_signature: Option<JvmBytes>,
    /// Why the class Signature could not be published as a complete Java header.
    pub generic_refusal: Option<String>,
}

/// One class-level annotation attribute's original shell and its parsed annotation entries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassAnnotationAttribute {
    pub attribute: AttributeShell,
    pub annotations: Vec<ElementValueFacts>,
}

/// One member's own `Runtime*Annotations` attribute, kept beside the spelling derived from it.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemberAnnotationUses {
    pub attributes: Vec<MemberAnnotationAttribute>,
    pub uses: Vec<String>,
    pub refusals: Vec<String>,
}

/// The raw shell and complete annotation values of one field or method declaration attribute.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemberAnnotationAttribute {
    pub attribute: AttributeShell,
    pub annotations: Vec<ElementValueFacts>,
}

/// One method's own `Runtime*ParameterAnnotations` attribute, including its leading `u1` count.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterAnnotationAttribute {
    pub attribute: AttributeShell,
    /// `None` means the shell was retained after its content could not be read.
    pub parameter_count: Option<u8>,
    pub parameters: Vec<Vec<ElementValueFacts>>,
}

/// Parsed and source-spelled parameter annotations, aligned only by descriptor parameter position.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterAnnotationUses {
    pub attributes: Vec<ParameterAnnotationAttribute>,
    /// One list for each descriptor parameter, including positions with no annotation.
    pub uses_by_position: Vec<Vec<String>>,
    pub refusals: Vec<String>,
}

/// One type annotation shell and every entry it physically contains, with the attempted spelling.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TypeAnnotationUses {
    pub attributes: Vec<TypeAnnotationAttribute>,
    pub field_uses: Vec<String>,
    pub return_uses: Vec<String>,
    /// Descriptor parameter position to type-only source spellings.
    pub parameter_uses: Vec<Vec<String>>,
    /// Refusals retain the entry's owner, target and path in their text.
    pub refusals: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TypeAnnotationAttribute {
    pub attribute: AttributeShell,
    pub annotations: Vec<TypeAnnotationUse>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TypeAnnotationUse {
    pub target_type: u8,
    pub target_info: Vec<u8>,
    pub type_path: Vec<TypePathEntry>,
    pub annotation: ElementValueFacts,
    pub spelling: Option<String>,
    pub refusal: Option<String>,
}

/// The raw member annotation facts handed from the one member-table read to its declaration writer.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct MemberAnnotationFacts {
    pub(crate) declaration: MemberAnnotationUses,
    pub(crate) parameters: ParameterAnnotationUses,
    pub(crate) type_uses: TypeAnnotationUses,
}

/// Parsed annotation facts and local attribute errors from one member's annotation shells.
pub(crate) struct MemberAnnotationRead {
    pub(crate) facts: MemberAnnotationFacts,
    pub(crate) errors: Vec<Error>,
}

/// One field of the presented class, with the spelling the assembled text writes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceField {
    /// The field record of the class read: its table position, its physical identity, its own raw
    /// name and descriptor, and its access flags.
    pub item: FieldItem,
    /// The declaration the text writes for this field, without its terminating `;`. `None` exactly
    /// when the field's raw descriptor is not one this presentation can read as a field descriptor
    /// (JVMS 4.3.2): the text then writes only the markers.
    ///
    /// The field's own `ConstantValue` attribute (JVMS 4.7.2), when it declares one and states a
    /// value this presentation has a literal for, is written as this declaration's initializer
    /// (`static final int N = 3`). For a proved ordinary-interface group, its same-run `<clinit>`
    /// RHS is appended here only after every fragment has been emitted successfully; the field
    /// remains in this vector at its original physical table index. A proved enum group changes
    /// only assembled source text: enum constants have no fabricated field records, while every
    /// physical field (including the backing array) remains here at its original index. A proven
    /// field `Signature` can replace the descriptor type while keeping the physical field identity
    /// in [`Self::item`].
    pub declaration: Option<String>,
    /// The field's own declaration annotation attributes, their complete values, spellings, and
    /// any annotation-level refusal.
    pub annotations: MemberAnnotationUses,
    /// Type annotation attributes physically owned by this field.
    pub type_annotations: TypeAnnotationUses,
    /// Every `// jarde:` marker the text carries for this field, in the order it writes them, each
    /// one line: an unspellable raw name/descriptor or the field `Signature` projection decision.
    /// Empty when no such explanation is needed.
    pub markers: Vec<String>,
}

/// One method of the presented class: the record of the class read, the spelling of its declaration,
/// the text this presentation wrote for it, and the result of its own body's run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceMethod {
    /// The method record of the class read: its table position, its physical identity a single-method
    /// recovery request consumes directly, its own raw name and descriptor, its access flags, and
    /// what its declaration says about its body ([`crate::MemberBodyEvidence`]).
    pub item: MethodItem,
    /// The `abstract`/`native` kind the member's own flags declare, when they declare one. `Some`
    /// exactly when the member declares no `Code` attribute *and* says which of the two it is; a
    /// member that declares neither is `None`, which is what the bytes state.
    pub no_body_kind: Option<NoBodyKind>,
    /// The Java spelling of this physical member, without its opening brace and without a
    /// terminating `;`. `None` exactly when the member's raw descriptor is not one this presentation
    /// can read as a method descriptor (JVMS 4.3.3), in which case no declaration is written and
    /// [`Self::outcome`] is [`ClassSourceOutcome::Unspelled`]. An admitted bridge keeps this physical
    /// spelling here while [`Self::text`] carries its projection explanation.
    ///
    /// A member named `<init>` is spelled as the constructor it is (the class's own name, no return
    /// type), and `<clinit>` as Java's static initializer block (`static`), each with a marker that
    /// names the raw member it came from. A proved enum group may omit its physical constructor,
    /// helpers, and `<clinit>` from assembled source, but their method-table entries and original
    /// outcomes remain here; the projection does not create replacement method records.
    pub declaration: Option<String>,
    /// The method's own declaration annotation attributes, values, spellings, and refusals.
    pub annotations: MemberAnnotationUses,
    /// The method's own parameter annotation attributes, in descriptor-position order when the
    /// attribute's `u1 num_parameters` proves that mapping.
    pub parameter_annotations: ParameterAnnotationUses,
    /// Type annotation attributes physically owned by this method.
    pub type_annotations: TypeAnnotationUses,
    /// The member's own text in the assembled source: its optional marker, its declaration and
    /// either its block or its `;`, or an identity-bearing bridge projection explanation. One level
    /// of indentation is already applied, so the text can be read on its own or found inside
    /// [`ClassSourceReport::text`] unchanged.
    ///
    /// A recovered body is placed by splitting the recovery artifact at its own block — the envelope
    /// comment lines the recovery layer wrote, then the statements it wrote — and indenting both one
    /// level deeper. That split is a reading of the fixpoint shape
    /// [`jarde_java::recover`] emits; a produced artifact that does not have it is quoted line by
    /// line as comments rather than placed as if it were statements.
    pub text: String,
    /// Every `// jarde:` marker the text carries for this member, in the order it writes them, each
    /// one line: a member with no `Code` attribute, a run that stopped or produced no statement, a
    /// member that could not be spelled, the alias of a raw name Java cannot spell, a run whose
    /// analysis did not complete, or a bridge projection explanation.
    pub markers: Vec<String>,
    /// What the member's own body produced: one recovery run's report, a declaration without a body,
    /// an unspellable member, or the member-level refusal of its run.
    pub outcome: ClassSourceOutcome,
    /// Whether the parsed constructor Signature states exactly the source-level int parameter.
    #[serde(skip)]
    pub(crate) enum_constructor_source_signature: bool,
    /// Whether the no-source-argument enum constructor has its exact `()V` source Signature.
    #[serde(skip)]
    pub(crate) enum_constructor_no_arg_source_signature: bool,
    /// Whether the ordinary physical-descriptor projection refused that source tail only because
    /// the VM-injected enum name and ordinal make its descriptor longer.
    #[serde(skip)]
    pub(crate) enum_constructor_signature_erasure_refused: bool,
}

/// The analysis half of one member's run, as this report publishes it.
///
/// The recovery report carries the other half — the artifact and its planes — and this is the part
/// of the run that is the analysis's own: the execution plane (which is where a stage that stopped
/// is stated, including a stop that still left the recovery layer enough tables to produce
/// something) and how many diagnostics the run published. The whole analysis report is not copied:
/// a caller that wants it re-runs the member with [`crate::Engine::recover_method`], which is the
/// same run by construction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceRunFacts {
    /// The execution plane of the member's own analysis run.
    pub execution: ExecutionReport,
    /// How many diagnostics that run published.
    pub diagnostics: u64,
}

/// What one member's body produced.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClassSourceOutcome {
    /// One recovery run over this member's body: the analysis of the member and the presentation of
    /// that same run's payload, which is exactly what one [`crate::Engine::recover_method`] request
    /// answers with. `report` may hold a stop and no text (`jre_ir_table_missing`, a refused output
    /// charge, a cancellation) — the marker and [`ClassSourceReport::execution`] state that this
    /// member body is not part of the presentation.
    Recovered {
        /// The presentation of the run's own payload: the Java text, its segment table, its
        /// content classification, its quality and the planes of the run.
        ///
        /// The text here is the artifact as the recovery layer wrote it — a method **body**, with
        /// its own envelope comments and its own positions — while [`ClassSourceMethod::text`] is
        /// that artifact placed inside this presentation. The segment table's offsets are therefore
        /// offsets of *this* text, not of the assembled source, which carries no map of its own.
        report: Box<RecoveryReport>,
        /// The analysis half of the same run.
        analysis: ClassSourceRunFacts,
    },
    /// The member declares no `Code` attribute: no run was performed, no body attempt was charged,
    /// and no empty body is invented for it. This is a declaration and never a stop — the class is
    /// presented whole beside it — which is why it carries no execution plane of its own.
    NoBody,
    /// The member's raw descriptor is not one this presentation can read as the descriptor of its
    /// kind: no declaration was written, no run was performed, and the member's record keeps the
    /// bytes.
    ///
    /// This is not a stop of the request and not a failure of a read — nothing was read — and it is
    /// deliberately not counted as work that was skipped either (see
    /// [`ClassSourceReport::coverage`]): a member whose bytes are not a descriptor at all is outside
    /// the schema this presentation serves, and the marker it carries is where that is stated.
    Unspelled,
    /// The member's own run was refused before it produced a report: a charge the budget refused, a
    /// cancellation, or a failure of the run's own read. The refusal is this member's own — the
    /// members beside it keep their results (A13) — and its execution plane closes the report's when
    /// the refusal ended the whole request.
    Refused {
        execution: ExecutionReport,
        diagnostics: Vec<Diagnostic>,
    },
}

/// One class presented as Java source, with the facts it was spelled from.
///
/// `text` is the assembled source: the class declaration, then its fields (in proved `<clinit>` write
/// order for a projected ordinary-interface group, otherwise in physical field-table order), then
/// its methods in the order the class file's own method table declares them. A fully projected
/// interface initializer is omitted from assembled source, but remains in `methods` with its
/// original recovery result. A proved enum group likewise changes only this assembled text: its
/// constants and source constructor replace the proved compiler-generated declarations, and a
/// narrowly proved static suffix may replace the matching `<clinit>` statements. Every physical
/// `fields` and `methods` item, table index, identity, and member outcome remains in this report's
/// vectors and JSON; member recovery reports keep their own source maps anchored to the original
/// bytecode. A stopped or refused enum proof leaves the whole enum group in its physical form.
/// `fields` always keeps physical field-table order; `methods` keeps the method-table order and
/// original recovery results. The planes are the request's own: `limits` is
/// the complete effective configuration, `usage` what that configuration was charged, `coverage`
/// the class and member tables this request read beside the members it attempted a body for, and
/// `execution` the merge of every stop of the request.
/// A proved direct member identity is retained in `member_family` with a separate child physical
/// report. A closed source projection replaces only this root report's text; child text, physical
/// methods, outcomes and their method-local source maps remain unchanged.
///
/// One member's stop does not end the class: a member whose recovery stopped keeps its own result
/// and the members beside it are still presented, exactly as a class view keeps the bodies beside a
/// stopped one. What a stopped member *does* do is make [`Self::execution`] non-`Complete`, so a
/// class whose presentation is partial is never published as a complete one.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceReport {
    /// The physical view this presentation read.
    pub view: PhysicalView,
    /// The physical definition this presentation is of: the identity a later read consumes.
    pub class: PhysicalDefinitionId,
    /// The class-level declaration, or `None` when the item charge for it was refused. The same read
    /// established the facts either way; a report that could not pay for its own declaration
    /// publishes nothing else — no field, no member, no text — and states the stop in `execution`
    /// and `diagnostics`.
    pub declaration: Option<ClassSourceDeclaration>,
    /// The analysis stages every member's run was asked for, in phase order: the recovery
    /// operation's own table ([`crate::MethodOperation::Recovery`]), published so a caller can
    /// reproduce one member's run with [`crate::Engine::recover_method`] and read the same schedule.
    pub stages: Vec<AnalysisStage>,
    /// The fields of the class read, in physical field-table order, each with its spelling.
    pub fields: Vec<ClassSourceField>,
    /// The methods of the class read, in the order its method table declares them, each with the
    /// result of its own run.
    pub methods: Vec<ClassSourceMethod>,
    /// Direct member evidence for this request. A prepared child keeps its own physical class,
    /// member table, coverage, execution and source maps. Its `usage` is the cumulative snapshot
    /// of the *same* request budget at the end of that child's preparation.
    pub member_family: ClassSourceMemberFamily,
    /// Same-run class-level bridge admission results. These are adapter evidence for the
    /// subsequent source projection and remain visible beside the physical method records.
    #[doc(hidden)]
    pub bridge_proofs: Vec<ClassSourceBridgeProof>,
    /// Cross-class enum-switch mapping decisions from this class-source run, including refusals.
    #[doc(hidden)]
    pub enum_switch_proofs: Vec<ClassSourceEnumSwitchProof>,
    /// The all-or-nothing proof result for ordinary Java 8 interface field initializers. The
    /// proof's field indices and write positions remain available beside the projected declarations;
    /// the original method records and recovery reports are retained in [`Self::methods`].
    pub initializer_proof: ClassSourceInitializerProof,
    /// Same-run proof input retained only for the next class-source projection stage. This is
    /// private assembly state and is deliberately absent from the serialized report.
    #[serde(skip)]
    pub(crate) enum_constant_proof: crate::enum_constants::ClassSourceEnumConstantProof,
    /// Same-run selected enum subclass relations, awaiting exclusivity, constructor and body proof.
    /// This private handoff has no projection authority and is deliberately absent from JSON.
    #[serde(skip)]
    pub(crate) enum_constant_body_relations: Vec<crate::facade::PendingEnumConstantBodyRelation>,
    /// The assembled Java source. Empty exactly when [`Self::declaration`] is `None`. A proved
    /// member projection contains one root source unit with a nested child declaration; refusal
    /// retains this root's physical text. The narrow derived ranges in `member_family` refer to
    /// this string; method-local recovery maps still refer only to their own recovery text.
    pub text: String,
    /// The complete effective limits this request ran under.
    pub limits: Limits,
    /// What those limits were charged, in the engine's own dimensions.
    ///
    /// The snapshot is cumulative for this request. Binding and preparing each physical class
    /// charge their own reads and bodies; when `member_family` contains a child, this root snapshot
    /// also includes the child's work. The child's `usage` is an earlier cumulative snapshot of the
    /// same budget, not a separately reset counter.
    pub usage: UsageSnapshot,
    /// Which class and member records this request read, and how many members it attempted a body
    /// for (`class_source_bodies`, in the method table's own coordinates).
    ///
    /// The class and member ranges are [`crate::ClassViewReport::coverage`]'s own, under the same
    /// labels and with the same rule for their state: the structure plane is `CompleteWithinSchema`
    /// when the read reached its ends, and a member whose own run stopped below does not rewrite it.
    /// This operation's range counts the members it *could* run a body for — one that declares a
    /// `Code` attribute and whose descriptor it can read — scanned up to the runs it entered and
    /// skipped from there to the end, and the dimension is `Partial` whenever such a member was left
    /// without a run: a stop of the request before it, a body attempt the budget refused, the
    /// preparation of the class failing, or a class that could not be prepared at all.
    ///
    /// A member that declares no body is outside that range, which is the same rule the class view
    /// applies to a body nobody asked for: a declaration is not a body attempt. A member whose bytes
    /// are not a method descriptor is outside it too — such a member is published as
    /// [`ClassSourceOutcome::Unspelled`], which is a statement about the bytes rather than work this
    /// presentation left undone.
    pub coverage: Coverage,
    /// The merge of every plane this request published: the name search, the class read, the member
    /// table's own stop, every member's run and any selected child preparation. Each child report
    /// separately preserves its physical execution state.
    pub execution: ExecutionReport,
    /// The class-level diagnostics this presentation re-publishes: the class read's own (a path that
    /// disagrees with the declared name, a member table that stopped) and one for each stop this
    /// presentation itself took over the member items.
    ///
    /// A member-level failure is deliberately not copied here: a refused member keeps its own
    /// diagnostic inside its record ([`ClassSourceOutcome::Refused`]) and the stop it states, exactly
    /// as a class view keeps a refused body's — one finding has one home, and the class's own list
    /// stays the list of things that happened to the class.
    pub diagnostics: Vec<Diagnostic>,
}

/// Prepared family identity, capture and calls are independent of the final source projection.
/// Only `projection: Projected` states that the root text contains a nested declaration. The
/// separate child report always retains its physical text and method-local source maps.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassSourceMemberFamily {
    Absent,
    Refused {
        reason: String,
        /// A resolved child is retained even when the two physical rows disagree.
        child: Option<Box<ClassSourceReport>>,
    },
    Prepared {
        relation: ClassSourceMemberRelation,
        child: Box<ClassSourceReport>,
        /// Capture evidence only; a refused capture keeps both physical reports intact.
        capture: ClassSourceMemberCapture,
        /// Family-local construction verdicts; original method bodies remain physical.
        calls: ClassSourceMemberCalls,
        /// Source-unit projection is separate from the physical relation and local certificates.
        projection: ClassSourceMemberProjection,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassSourceMemberProjection {
    Refused {
        reason: String,
    },
    Projected {
        derived: Vec<MemberFamilyDerivedProjection>,
    },
}

/// A source-unit byte range created by the proved family writer. It never replaces a physical
/// method's recovery map: the range is in the root report's `text`, while every anchor names its
/// own physical definition and member.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemberFamilyDerivedProjection {
    pub kind: MemberFamilyDerivedKind,
    pub start: usize,
    pub end: usize,
    pub anchors: Vec<MemberFamilyPhysicalAnchor>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberFamilyDerivedKind {
    MemberConstruction,
    CapturedOuterRead,
    HiddenCaptureField,
    HiddenConstructorParameter,
    HiddenCaptureWrite,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemberFamilyPhysicalAnchor {
    Field {
        field: jarde_reader::model::PhysicalMemberId,
        index: u64,
    },
    MethodPoint {
        method: PhysicalMethodId,
        bci: u32,
    },
    ConstructorParameter {
        method: PhysicalMethodId,
        index: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassSourceMemberCalls {
    Proved {
        sites: Vec<MemberCallProof>,
    },
    Refused {
        reason: String,
        sites: Vec<MemberCallProof>,
        refusals: Vec<MemberCallRefusal>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemberCallProof {
    pub caller: PhysicalMethodId,
    pub allocation_bci: u32,
    pub copy_bci: u32,
    pub qualifier_bci: u32,
    pub null_check_bci: u32,
    pub null_pop_bci: u32,
    pub constructor_bci: u32,
    pub constructor: PhysicalMethodId,
    pub ordinary_argument_bcis: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemberCallRefusal {
    pub caller: PhysicalMethodId,
    pub allocation_bci: u32,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassSourceMemberCapture {
    Proved { proof: MemberCaptureProof },
    Refused { reason: String },
}

/// Physical evidence for capture only. A writer must separately prove every changed expression.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemberCaptureProof {
    pub field_index: u64,
    pub field_name: String,
    pub constructor: PhysicalMethodId,
    pub write_bci: u32,
    pub reads: Vec<MemberCaptureRead>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemberCaptureRead {
    pub method: PhysicalMethodId,
    pub bci: u32,
    /// SSA consumers of the value loaded from the capture field, in this physical method.
    pub consumer_bcis: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceMemberRelation {
    pub root: PhysicalDefinitionId,
    pub child: PhysicalDefinitionId,
    pub simple_name: String,
    /// Source modifiers come from the matching InnerClasses rows, not child class access flags.
    pub access_flags: u16,
}

/// The map entries this class-source request proved from a selected helper's physical `<clinit>`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceEnumSwitchEntry {
    pub key: i64,
    pub constant: Vec<u8>,
    pub constant_field_bci: u32,
    pub ordinal_bci: u32,
    pub table_read_bci: u32,
    pub store_bci: u32,
    pub handler_ordinal: u32,
    pub handler_bci: u32,
}

/// One same-run enum-switch candidate's class-level proof and projection outcome.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceEnumSwitchProof {
    pub member: jarde_reader::model::PhysicalMethodId,
    pub switch_bci: u32,
    pub read_bci: u32,
    pub table_owner: String,
    pub table_name: String,
    pub helper: Option<jarde_reader::model::PhysicalDefinitionId>,
    pub enum_definition: Option<jarde_reader::model::PhysicalDefinitionId>,
    pub entries: Vec<ClassSourceEnumSwitchEntry>,
    pub projected: bool,
    pub refusal: Option<String>,
}

/// A class-level bridge admission result and the state of its complete-source projection.
/// Admission preserves the ordinary recovery report and physical method table; `projected` only
/// states whether this class-source body emitted the explanation in place of a Java declaration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceBridgeProof {
    /// The physical bridge method whose same-run plan entered this proof.
    pub member: jarde_reader::model::PhysicalMethodId,
    /// The unique source method when all forward and signature premises were established.
    pub target: Option<jarde_reader::model::PhysicalMethodId>,
    /// The invocation BCI retained by the same-run `bridge@1` candidate.
    pub call_bci: Option<u32>,
    /// Whether the bridge may be elided from Java source under the proved inherited contract.
    pub admitted: bool,
    /// Whether the complete source replaced this member's Java declaration with a proof note.
    pub projected: bool,
    /// The first conservative reason the complete proof refused admission.
    pub refusal: Option<String>,
}

/// Whether the complete ordinary-interface initializer group passed its structural proof.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClassSourceInitializerProof {
    /// The class is outside this proof's Java 8 ordinary-interface scope.
    NotApplicable,
    /// Every runtime field write in the group passed, in original `<clinit>` order.
    Proved {
        /// Physical field index and its same-run write position and BCI, in initializer order.
        fields: Vec<ClassSourceInitializerField>,
    },
    /// The whole runtime initializer group was refused with the first unmet proof condition.
    Refused { reason: String },
}

/// The write whose value would be eligible for a later source projection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceInitializerField {
    /// The unchanged physical `field_info` table index.
    pub field_index: u64,
    /// The statement's order in the same-run `<clinit>` AST.
    pub write_order: usize,
    /// The `putstatic` BCI claimed by `field@1`.
    pub write_bci: u32,
}

// ---------------------------------------------------------------------------------------------
// The class file's flags, as source spells them
// ---------------------------------------------------------------------------------------------
//
// One table per declaration position (JVMS 4.1-B for a class, 4.5-A for a field, 4.6-A for a
// method), read here in one vocabulary: a modifier list has to read every bit of the one table it
// belongs to, so the bits are stated together rather than one part named from another layer and the
// rest from here. What is *not* restated is any rule about them: which of the three visibilities
// wins, that an interface is abstract by its kind, and that a body-less member is `abstract` or
// `native` are all read from the same table, below.

/// `ACC_PUBLIC` (JVMS 4.1-B).
const ACC_PUBLIC: u16 = 0x0001;
/// `ACC_PRIVATE` (JVMS 4.1-B).
const ACC_PRIVATE: u16 = 0x0002;
/// `ACC_PROTECTED` (JVMS 4.1-B).
const ACC_PROTECTED: u16 = 0x0004;
/// `ACC_FINAL` (JVMS 4.1-B).
const ACC_FINAL: u16 = 0x0010;
/// `ACC_STATIC` (JVMS 4.1-B/4.5-A/4.6-A): a class's own `static` describes a nesting this
/// presentation does not claim, so only a member's is written.
const ACC_STATIC: u16 = 0x0008;
/// `ACC_SYNCHRONIZED` on a method, `ACC_SUPER` on a class (JVMS 4.1-B/4.6-A): one bit, two
/// meanings, decided by the table it is read in.
const ACC_SYNCHRONIZED: u16 = 0x0020;
/// `ACC_VOLATILE` on a field (JVMS 4.5-A).
const ACC_VOLATILE: u16 = 0x0040;
/// `ACC_TRANSIENT` on a field, `ACC_VARARGS` on a method (JVMS 4.5-A/4.6-A): one bit, two
/// meanings, decided by the table it is read in.
const ACC_TRANSIENT: u16 = 0x0080;
/// `ACC_VARARGS` (JVMS 4.6-A): [`ACC_TRANSIENT`]'s bit in a *method's* own `access_flags`, stating
/// that the member's last parameter is the variable-arity one. It is read where the parameter list
/// is spelled ([`method_descriptor`]) and nowhere else.
const ACC_VARARGS: u16 = 0x0080;
/// `ACC_NATIVE` on a method (JVMS 4.6-A).
const ACC_NATIVE: u16 = 0x0100;
/// `ACC_INTERFACE` (JVMS 4.1-B).
const ACC_INTERFACE: u16 = 0x0200;
/// `ACC_ABSTRACT` (JVMS 4.1-B/4.6-A).
const ACC_ABSTRACT: u16 = 0x0400;
/// `ACC_STRICT` (JVMS 4.1-B/4.6-A).
const ACC_STRICT: u16 = 0x0800;
/// `ACC_ANNOTATION` (JVMS 4.1-B).
const ACC_ANNOTATION: u16 = 0x2000;
/// `ACC_ENUM` (JVMS 4.1-B): a class declared `enum`, or a field holding an enum constant.
const ACC_ENUM: u16 = 0x4000;

fn is_static(access_flags: u16) -> bool {
    access_flags & ACC_STATIC != 0
}

// ---------------------------------------------------------------------------------------------
// Names and descriptors, as Java source spells them
// ---------------------------------------------------------------------------------------------

/// One internal name as source spells a **reference** to it: `/` becomes `.` and nothing else
/// happens.
///
/// This is the spelling of every type a declaration names and does not declare: a `super_class`, the
/// interfaces of an `implements`/`extends` list, and a member's own `throws` clause. The one change
/// is deliberate. Turning `p.Outer$Inner` into a nesting relation, or splitting a name into a
/// package and a simple name, would be inventing structure the class file does not state; `$` is a
/// legal Java identifier character, and the name stays the one the bytes carry.
fn class_name(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).replace('/', ".")
}

/// The simple name one internal name states: everything after its last `/`, with `$` kept.
///
/// This is the name a class's **own** declaration and its constructors are spelled with. The package
/// the name states before that `/` is not dropped: it is written as the text's `package` statement
/// ([`package_name`]), which is the one place Java source states it. Only the last `/` is a package
/// boundary — `$` is a legal Java identifier character, so `p/More17$1` states the simple name
/// `More17$1`, exactly as `p/Outer$Inner` states `Outer$Inner`.
fn simple_name(raw: &[u8]) -> String {
    let simple = match raw.iter().rposition(|byte| *byte == b'/') {
        Some(position) => &raw[position + 1..],
        None => raw,
    };
    String::from_utf8_lossy(simple).into_owned()
}

/// The package one internal name states, as a `package` statement spells it, or `None` for a name
/// that states none (the default package).
///
/// The package is the internal name in front of its last `/`, with every remaining `/` written as
/// `.` — the same rule [`class_name`] applies to the whole name. `None` means the text writes no
/// `package` line at all: a name with no `/` is in the unnamed package, which source spells with no
/// statement rather than with an empty one.
fn package_name(raw: &[u8]) -> Option<String> {
    let position = raw.iter().rposition(|byte| *byte == b'/')?;
    Some(String::from_utf8_lossy(&raw[..position]).replace('/', "."))
}

/// The name the text writes for one raw member name, and whether it had to be changed.
///
/// The change is [`alias_for`]'s and it is deterministic: a raw name Java cannot spell (a method
/// named `foo-bar`) becomes the alias that function states, and the marker the caller writes says
/// which raw name that alias came from. `<init>` and `<clinit>` never reach this rule — they are
/// spelled as the declarations they are — and a name that is already an identifier is unchanged.
fn written_name(raw: &[u8]) -> (String, bool) {
    let text = String::from_utf8_lossy(raw).into_owned();
    if is_java_identifier(&text) {
        (text, false)
    } else {
        (alias_for(&text), true)
    }
}

/// One descriptor component as Java source spells its type.
///
/// The spelling is the recovery layer's own ([`type_of_component`]), so this presentation and the
/// statements of a recovered body cannot disagree about how a type is written: a primitive is the
/// primitive's own name, and an array is spelled from its element outwards with one `[]` per
/// dimension. Nothing about the production is read here — the reader states it
/// ([`jarde_reader::classfile::descriptor_facts`]) — and an object name Java cannot write is
/// refused rather than spelled lossily into a type position.
fn source_type(component: &DescriptorComponent) -> Option<String> {
    Some(type_of_component(component)?.spell().to_owned())
}

/// One field descriptor as Java source spells it (JVMS 4.3.2), or `None` when the bytes are not
/// that production.
fn field_type(descriptor: &[u8]) -> Option<String> {
    let facts = descriptor_facts(descriptor, DescriptorKind::Field).ok()?;
    source_type(facts.single()?)
}

/// One method descriptor as this presentation writes its declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Signature {
    /// The parameters in declaration order: the type as Java spells it, and the **local slot** the
    /// parameter occupies, with the receiver counted.
    ///
    /// The type is the array spelling the descriptor states — one `[]` per dimension — because that
    /// is the type the descriptor's component names ([`source_type`]); which of the parameters is
    /// written with the dots of [`Self::varargs`] is the **list's** own rule and is applied where
    /// the list is written ([`arguments`]).
    ///
    /// The slot is the JVM layer's derivation from the descriptor's own facts
    /// ([`parameter_positions`]): `this` holds slot 0 of a member that is not `static`, and each
    /// parameter starts where the one before it ended — a `long`/`double` filling two slots and
    /// **an array of either filling one**, like every other array.
    parameters: Vec<(String, u16)>,
    /// Whether the declaration's **last** parameter is written `T...`: true exactly when the
    /// member's own `ACC_VARARGS` is set and the descriptor's last parameter is an array, which is
    /// the one shape the flag can apply to (see [`method_descriptor`]).
    ///
    /// It is stated here and not spelled into [`Self::parameters`] because it is a fact about the
    /// parameter **list** — the last position, and only the last — while each entry above is the
    /// type its own component names.
    varargs: bool,
    /// The return type, or `None` for `V`.
    returns: Option<String>,
    /// How many slots the parameters occupy together, the receiver included: the first slot a body
    /// of this member may declare a local in.
    slots: u16,
}

/// One method descriptor as Java source spells it, or `None` when the bytes are not that production
/// or state a type this presentation cannot write.
///
/// The descriptor is read **once**, by the reader's own facts, and everything the declaration needs
/// comes from that one reading: the types are the recovery layer's spelling of the reader's
/// components, and the slots are the JVM layer's derivation from those same components. So a
/// signature's parameter names and the body's references to them are one reading of one descriptor
/// — never two parses that could place a parameter in two different slots.
///
/// `varargs` is the member's own `ACC_VARARGS` (0x0080), the one fact of the declaration that the
/// descriptor does not state. It reaches the **last** parameter and only when that parameter is an
/// array: `[I` is written `int...` and `[[B` is written `byte[]...` — the element type, then the
/// `[]` it still has, then the dots the last `[]` became. A member whose flag is set and whose last
/// parameter is not an array states no `...` at all: the flag is carried by no position, and the
/// parameters are written as the descriptor states them.
fn method_descriptor(descriptor: &[u8], is_static: bool, varargs: bool) -> Option<Signature> {
    let facts = descriptor_facts(descriptor, DescriptorKind::Method).ok()?;
    let positions = parameter_positions(&facts, is_static)?;
    let parameters = facts
        .parameters()
        .iter()
        .zip(positions)
        .map(|(component, slot)| Some((source_type(component)?, slot)))
        .collect::<Option<Vec<(String, u16)>>>()?;
    // The two facts meet here, and neither alone states a `...`: the member's own flag states that
    // its last parameter is the variable-arity one, and the descriptor's own last component states
    // whether there is an array for that to mean. An array parameter of a member without the flag
    // keeps its brackets, and a flag on a member whose last parameter is not an array is written as
    // the descriptor states it.
    let varargs = varargs
        && facts
            .parameters()
            .last()
            .is_some_and(DescriptorComponent::is_array);
    let returns = match facts.result() {
        Some(component) => Some(source_type(component)?),
        None => None,
    };
    let slots = u16::from(!is_static).checked_add(facts.parameter_slots()?)?;
    Some(Signature {
        parameters,
        varargs,
        returns,
        slots,
    })
}

/// The declaration line one class is written as, without its opening brace.
///
/// The kind comes from the class file's own flags — `@interface`, `interface`, `enum` or `class` —
/// and so does everything else here:
///
/// * only the modifiers source spells are written, and one that the kind already means is not
///   repeated (`abstract` on an interface, `abstract`/`final` on an enum);
/// * a class's superclass is written from `super_class`, which the format requires to be present for
///   every class that is not `module-info`;
/// * an ordinary interface's `extends` list is the interfaces the class file declares, because its
///   own `super_class` is `java/lang/Object` by the format's rule and is not a source superclass;
///   a canonical annotation's implicit `Annotation` interface remains in the facts but is omitted
///   from the `@interface` header.
fn class_declaration(name: &str, facts: &ClassDeclarationFacts) -> String {
    class_declaration_with_types(name, facts, None, None, None)
}

fn class_declaration_with_types(
    name: &str,
    facts: &ClassDeclarationFacts,
    parameters: Option<&str>,
    generic_superclass: Option<&str>,
    generic_interfaces: Option<&[String]>,
) -> String {
    let flags = facts.access_flags;
    let interface = flags & ACC_INTERFACE != 0;
    let canonical_annotation = flags & (ACC_ANNOTATION | ACC_INTERFACE)
        == (ACC_ANNOTATION | ACC_INTERFACE)
        && facts.interfaces.len() == 1
        && facts.interfaces[0].raw().0 == b"java/lang/annotation/Annotation";
    let enumeration = !interface && flags & ACC_ENUM != 0;
    let mut words: Vec<&str> = Vec::new();
    words.extend(visibility(flags));
    if !interface {
        if flags & ACC_ABSTRACT != 0 && !enumeration {
            words.push("abstract");
        }
        if flags & ACC_FINAL != 0 && !enumeration {
            words.push("final");
        }
        if flags & ACC_STRICT != 0 {
            words.push("strictfp");
        }
    }
    let kind = if flags & ACC_ANNOTATION != 0 {
        "@interface"
    } else if interface {
        "interface"
    } else if enumeration {
        "enum"
    } else {
        "class"
    };
    let mut line = String::new();
    for word in words {
        line.push_str(word);
        line.push(' ');
    }
    line.push_str(kind);
    line.push(' ');
    line.push_str(name);
    if let Some(parameters) = parameters {
        line.push('<');
        line.push_str(parameters);
        line.push('>');
    }
    if canonical_annotation {
        return line;
    }
    let supers: Vec<String> = generic_interfaces
        .map(<[String]>::to_vec)
        .unwrap_or_else(|| {
            facts
                .interfaces
                .iter()
                .map(|interface| class_name(&interface.raw().0))
                .collect()
        });
    if interface {
        if !supers.is_empty() {
            line.push_str(" extends ");
            line.push_str(&supers.join(", "));
        }
        return line;
    }
    if !enumeration && let Some(super_class) = &facts.super_class {
        line.push_str(" extends ");
        if let Some(generic_superclass) = generic_superclass {
            line.push_str(generic_superclass);
        } else {
            line.push_str(&class_name(&super_class.raw().0));
        }
    }
    if !supers.is_empty() {
        line.push_str(" implements ");
        line.push_str(&supers.join(", "));
    }
    line
}

/// The names the parameters of one member are written with, in **slot** order: index 0 is slot 0,
/// which is the receiver of a member that is not `static` and the first parameter of one that is.
///
/// The names are the recovery layer's own — [`NameTable`] over the debug names the member's run
/// read — so the signature and the body spell one parameter the same way: a slot the debug table
/// names once takes that name (aliased when Java cannot spell it), a slot it names over several
/// records takes the first variable's name, and a slot it does not name takes the ordinal name the
/// naming table invents (`arg<slot>`), which is exactly what the body's statements say.
///
/// A member no run was performed for — one that declares no body, one that could not be spelled —
/// carries no debug names at all, so its parameters take those ordinal names. That is a rule of this
/// presentation's, stated here, and not a claim about the source: the class file does not state
/// parameter names, and the debug table is the only place one may come from.
fn parameter_names(facts: Option<&RecoveryFacts>, slots: u16) -> Vec<String> {
    let mut records: Vec<Vec<String>> = vec![Vec::new(); usize::from(slots)];
    for record in facts.into_iter().flat_map(|facts| facts.debug_locals()) {
        if let Some(names) = records.get_mut(usize::from(record.slot())) {
            names.push(record.name().to_owned());
        }
    }
    let evidence: Vec<SlotEvidence> = records
        .into_iter()
        .map(|names| match names.len() {
            0 => SlotEvidence::Unnamed,
            1 => SlotEvidence::Whole(names[0].clone()),
            _ => SlotEvidence::Split(names.into_iter().map(Some).collect()),
        })
        .collect();
    let table = NameTable::build(slots, slots, &evidence);
    (0..slots)
        .map(|slot| {
            table
                .name(LocalVariable::new(slot, 0))
                .map(|name| name.text().to_owned())
                .unwrap_or_else(|| format!("arg{slot}"))
        })
        .collect()
}

/// One raw name or descriptor, as a comment line can carry it.
///
/// Two steps, both of them the engine's own: the reader's escaped display of the bytes
/// ([`JvmString::escaped`]), then [`comment_text`], which is what makes the result safe inside a
/// `//` comment. The second step is not cosmetic — Java processes `\uXXXX` before it lexes, so a
/// comment carrying one would end the line it is written on, which is the hazard the recovery
/// layer's own emitter states for its reasons.
fn comment_name(value: &JvmString) -> String {
    comment_text(&value.escaped())
}

/// How one member is named inside a marker: its own raw name and descriptor, as the recovery
/// report's `method` field spells them (`name` then `descriptor`, no separator).
fn label(item: &MethodItem) -> String {
    format!(
        "{}{}",
        comment_name(&item.name),
        comment_name(&item.descriptor)
    )
}

/// The visibility Java source spells, or nothing at all for a package-private declaration.
fn visibility(access_flags: u16) -> Option<&'static str> {
    if access_flags & ACC_PUBLIC != 0 {
        Some("public")
    } else if access_flags & ACC_PROTECTED != 0 {
        Some("protected")
    } else if access_flags & ACC_PRIVATE != 0 {
        Some("private")
    } else {
        None
    }
}

/// One member's declaration as this presentation spells it, with the marker its spelling needs.
///
/// The request's own orchestration reads this value: it spells a member *before* deciding whether to
/// run a recovery for it ([`spellable_descriptor`]), so a member whose declaration cannot be written is never
/// given a run that could not be placed anywhere.
pub(crate) struct Spelled {
    /// The declaration, without a terminating `;` and without an opening brace. `None` exactly when
    /// the member's raw descriptor is not one this presentation can read as the descriptor of its
    /// kind.
    pub(crate) declaration: Option<String>,
    /// The marker the spelling itself needs: an alias for a name Java cannot spell, or the statement
    /// that no declaration could be written at all. `None` when the declaration is faithful.
    pub(crate) marker: Option<String>,
    pub(crate) annotations: MemberAnnotationUses,
    pub(crate) parameter_annotations: ParameterAnnotationUses,
    pub(crate) type_annotations: TypeAnnotationUses,
}

/// Whether this presentation can write a declaration for a member whose raw descriptor these bytes
/// are: a method descriptor (JVMS 4.3.3) whose every type is one Java source spells.
///
/// A member this answers `false` for is published as
/// [`ClassSourceOutcome::Unspelled`](crate::ClassSourceOutcome::Unspelled), and no recovery run is
/// performed for it — a run whose artifact could not be placed under a declaration would be work
/// whose result this presentation has nowhere to write.
pub(crate) fn spellable_descriptor(descriptor: &[u8]) -> bool {
    match descriptor_facts(descriptor, DescriptorKind::Method) {
        Ok(facts) => facts
            .components()
            .all(|component| type_of_component(component).is_some()),
        Err(_) => false,
    }
}

/// Whether one member's own attribute table declares an attribute of this name.
///
/// The name is compared against the shell's own raw bytes, which is what the class file states; the
/// attributes this presentation reads are the declaration attributes whose content it spells
/// (`ConstantValue`, `Exceptions`, `AnnotationDefault`).
fn declares_attribute(member: &MemberHeader, name: &[u8]) -> bool {
    member
        .attributes
        .iter()
        .any(|shell| shell.name.raw().0.as_slice() == name)
}

/// The attribute shells of one member whose own name is this one, in table order.
fn attribute_shells(member: &MemberHeader, name: &[u8]) -> Vec<AttributeShell> {
    member
        .attributes
        .iter()
        .filter(|shell| shell.name.raw().0.as_slice() == name)
        .cloned()
        .collect()
}

/// Whether one member's own declaration annotation table has a content shell to read.
pub(crate) fn declares_member_annotations(member: &MemberHeader) -> bool {
    member.attributes.iter().any(|shell| {
        matches!(
            shell.name.raw().0.as_slice(),
            b"RuntimeVisibleAnnotations"
                | b"RuntimeInvisibleAnnotations"
                | b"RuntimeVisibleTypeAnnotations"
                | b"RuntimeInvisibleTypeAnnotations"
        )
    })
}

/// Whether one method's own parameter annotation table has a content shell to read.
pub(crate) fn declares_parameter_annotations(member: &MemberHeader) -> bool {
    member.attributes.iter().any(|shell| {
        matches!(
            shell.name.raw().0.as_slice(),
            b"RuntimeVisibleParameterAnnotations" | b"RuntimeInvisibleParameterAnnotations"
        )
    })
}

/// Reads only the member's declaration, parameter and type-annotation attributes, preserving every
/// shell even when its contents stop. Invalid content is local to its own shell; a budget refusal or
/// cancellation stops before any later shell is read.
pub(crate) fn declared_member_annotations(
    bytes: &[u8],
    member: &MemberHeader,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> MemberAnnotationRead {
    let mut facts = MemberAnnotationFacts::default();
    let mut errors = Vec::new();
    let shells = member
        .attributes
        .iter()
        .filter(|shell| {
            matches!(
                shell.name.raw().0.as_slice(),
                b"RuntimeVisibleAnnotations"
                    | b"RuntimeInvisibleAnnotations"
                    | b"RuntimeVisibleParameterAnnotations"
                    | b"RuntimeInvisibleParameterAnnotations"
                    | b"RuntimeVisibleTypeAnnotations"
                    | b"RuntimeInvisibleTypeAnnotations"
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    // Attribute names are unique within their owning table. Treat repeated shells as one local
    // structural refusal while leaving other, independent annotation attributes readable.
    let mut counts = std::collections::BTreeMap::<Vec<u8>, usize>::new();
    for shell in &shells {
        *counts.entry(shell.name.raw().0.clone()).or_default() += 1;
    }
    let mut duplicate_errors = std::collections::BTreeSet::<Vec<u8>>::new();
    let mut stopped = false;
    for (index, shell) in shells.iter().enumerate() {
        let name = shell.name.raw().0.as_slice();
        let parameter = matches!(
            name,
            b"RuntimeVisibleParameterAnnotations" | b"RuntimeInvisibleParameterAnnotations"
        );
        let type_annotation = matches!(
            name,
            b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
        );
        let duplicate = counts.get(name).copied().unwrap_or_default() > 1;
        if stopped || duplicate {
            let reason = if stopped {
                "annotation content read stopped before this attribute".to_owned()
            } else {
                let error = Error::invalid_input(
                    "classfile_duplicate_attribute",
                    format!(
                        "member declares more than one {} attribute",
                        String::from_utf8_lossy(name)
                    ),
                );
                if duplicate_errors.insert(name.to_vec()) {
                    errors.push(error.clone());
                }
                error.to_string()
            };
            push_unread_annotation_shell(
                &mut facts,
                shell.clone(),
                parameter,
                type_annotation,
                reason,
            );
            continue;
        }
        let parsed = attribute_facts(bytes, std::slice::from_ref(shell), pool, budget);
        match parsed {
            Ok(parsed) => push_parsed_annotation_shell(&mut facts, shell.clone(), &parsed),
            Err(error) => {
                push_unread_annotation_shell(
                    &mut facts,
                    shell.clone(),
                    parameter,
                    type_annotation,
                    format!("annotation attribute read stopped: {error}"),
                );
                let ends = matches!(
                    error,
                    Error::BudgetExceeded { .. } | Error::Cancelled { .. }
                );
                errors.push(error);
                stopped = ends;
            }
        }
        if stopped {
            for pending in &shells[index + 1..] {
                let pending_name = pending.name.raw().0.as_slice();
                let parameter = matches!(
                    pending_name,
                    b"RuntimeVisibleParameterAnnotations" | b"RuntimeInvisibleParameterAnnotations"
                );
                let type_annotation = matches!(
                    pending_name,
                    b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
                );
                push_unread_annotation_shell(
                    &mut facts,
                    pending.clone(),
                    parameter,
                    type_annotation,
                    "annotation content read stopped before this attribute".to_owned(),
                );
            }
            break;
        }
    }
    MemberAnnotationRead { facts, errors }
}

fn push_parsed_annotation_shell(
    facts: &mut MemberAnnotationFacts,
    shell: AttributeShell,
    parsed: &jarde_reader::classfile::AttributeFacts,
) {
    match shell.name.raw().0.as_slice() {
        b"RuntimeVisibleAnnotations" => {
            facts
                .declaration
                .attributes
                .push(MemberAnnotationAttribute {
                    attribute: shell,
                    annotations: parsed.runtime_visible_annotations.clone(),
                })
        }
        b"RuntimeInvisibleAnnotations" => {
            facts
                .declaration
                .attributes
                .push(MemberAnnotationAttribute {
                    attribute: shell,
                    annotations: parsed.runtime_invisible_annotations.clone(),
                })
        }
        b"RuntimeVisibleParameterAnnotations" => {
            let parsed = parsed.runtime_visible_parameter_annotations.as_ref();
            facts
                .parameters
                .attributes
                .push(ParameterAnnotationAttribute {
                    attribute: shell,
                    parameter_count: parsed.map(|annotations| annotations.parameter_count),
                    parameters: parsed
                        .map_or_else(Vec::new, |annotations| annotations.parameters.clone()),
                });
        }
        b"RuntimeInvisibleParameterAnnotations" => {
            let parsed = parsed.runtime_invisible_parameter_annotations.as_ref();
            facts
                .parameters
                .attributes
                .push(ParameterAnnotationAttribute {
                    attribute: shell,
                    parameter_count: parsed.map(|annotations| annotations.parameter_count),
                    parameters: parsed
                        .map_or_else(Vec::new, |annotations| annotations.parameters.clone()),
                });
        }
        b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations" => {
            let annotations = if shell.name.raw().0 == b"RuntimeVisibleTypeAnnotations" {
                &parsed.runtime_visible_type_annotations
            } else {
                &parsed.runtime_invisible_type_annotations
            };
            facts.type_uses.attributes.push(TypeAnnotationAttribute {
                attribute: shell,
                annotations: annotations.iter().map(unspelled_type_annotation).collect(),
            });
        }
        _ => unreachable!("only member annotation attributes are passed"),
    }
}

fn push_unread_annotation_shell(
    facts: &mut MemberAnnotationFacts,
    shell: AttributeShell,
    parameter: bool,
    type_annotation: bool,
    reason: String,
) {
    if type_annotation {
        facts.type_uses.attributes.push(TypeAnnotationAttribute {
            attribute: shell,
            annotations: Vec::new(),
        });
        facts.type_uses.refusals.push(reason);
    } else if parameter {
        facts
            .parameters
            .attributes
            .push(ParameterAnnotationAttribute {
                attribute: shell,
                parameter_count: None,
                parameters: Vec::new(),
            });
        facts.parameters.refusals.push(reason);
    } else {
        facts
            .declaration
            .attributes
            .push(MemberAnnotationAttribute {
                attribute: shell,
                annotations: Vec::new(),
            });
        facts.declaration.refusals.push(reason);
    }
}

fn unspelled_type_annotation(fact: &TypeAnnotationFacts) -> TypeAnnotationUse {
    TypeAnnotationUse {
        target_type: fact.target_type,
        target_info: fact.target_info.clone(),
        type_path: fact.type_path.clone(),
        annotation: fact.annotation.clone(),
        spelling: None,
        refusal: None,
    }
}

/// Whether one member's own attribute table declares an `AnnotationDefault` attribute (JVMS 4.7.22).
///
/// This is the one predicate behind the two decisions the class-source orchestration makes about that
/// attribute: whether the class's constant pool has to be resolved at all (a class none of whose
/// members declares one spells no `default` and reads no pool for it), and whether this member has an
/// attribute to read.
pub(crate) fn declares_annotation_default(member: &MemberHeader) -> bool {
    declares_attribute(member, b"AnnotationDefault")
}

/// Whether one member's own attribute table declares an `Exceptions` attribute (JVMS 4.7.4).
///
/// It is [`declares_annotation_default`]'s own rule for that attribute: the pool a member's
/// `throws` clause is spelled from is resolved only when some member really declares one, and a
/// member that declares none has nothing to read.
pub(crate) fn declares_exceptions(member: &MemberHeader) -> bool {
    declares_attribute(member, b"Exceptions")
}

pub(crate) fn declares_signature(member: &MemberHeader) -> bool {
    declares_attribute(member, b"Signature")
}

/// Whether one field's own attribute table declares a `ConstantValue` attribute (JVMS 4.7.2).
pub(crate) fn declares_constant_value(field: &MemberHeader) -> bool {
    declares_attribute(field, b"ConstantValue")
}

/// The default one member's own `AnnotationDefault` attribute states (JVMS 4.7.22), resolved against
/// the class's constant pool as far as this presentation spells it, or `None` when no `default` is
/// written.
///
/// The read is [`attribute_facts`] over this member's own `AnnotationDefault` shell(s) — the reader's
/// one parser for that attribute, and only those shells are handed over, so the `attribute_bytes`
/// charge is exactly this attribute's — and nothing here reads a body, a use site or another member:
/// the declaration default is the value the member's own attribute declares.
///
/// `None` is exactly the state "nothing to write", and it is never a stop of the request: the member
/// declares no `AnnotationDefault` at all, or its value is one this presentation has no literal for.
/// Those include an unspellable NaN bit pattern, a nested annotation with an invalid source name,
/// and an array holding one of those (see [`MemberDefault`]). The member keeps
/// the declaration its flags and its descriptor state, which is what the class file's *other* facts
/// say about it; not one literal is invented for it.
///
/// An `Err` is the reader's own failure to read the attribute (its bytes are damaged, its charge was
/// refused, the request was cancelled): the caller decides what that is worth — a budget or a
/// cancellation is the request's own stop, exactly as a refused `Exceptions` read is.
pub(crate) fn declared_annotation_default(
    bytes: &[u8],
    member: &MemberHeader,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<Option<MemberDefault>> {
    if !declares_annotation_default(member) {
        return Ok(None);
    }
    let shells = attribute_shells(member, b"AnnotationDefault");
    let Some(value) = attribute_facts(bytes, &shells, pool, budget)?.annotation_default else {
        return Ok(None);
    };
    Ok(resolve_default(&value, pool))
}

/// The declaration facts one member's own attribute table states, as its declaration writes them.
///
/// `Exceptions` is read once from this member's own shells, in attribute order. Its raw names
/// prove a generic `Signature`'s optional throws suffix; its source spellings keep the same
/// declaration clause before and after the member's recovery run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MemberAttributes {
    /// The member's own `AnnotationDefault` (JVMS 4.7.22), when it declares one this presentation
    /// has a literal for.
    pub(crate) default: Option<MemberDefault>,
    /// The exception types the member's own `Exceptions` attribute (JVMS 4.7.4) states, in the
    /// attribute's own order; empty when the member declares none.
    pub(crate) throws: Vec<String>,
    /// The same physical exception names before source spelling, for signature erasure proof.
    pub(crate) throws_raw: Vec<Vec<u8>>,
}

/// The declaration facts one member's own attribute table states (JVMS 4.7.4, 4.7.22).
///
/// A member that declares neither attribute is read by neither reader (both answer from the shells
/// the member really has), and an `Err` is the first read that failed: a member whose own attributes
/// cannot be read has no declaration this presentation can state faithfully, which is the caller's
/// own decision to make — see [`declared_annotation_default`].
pub(crate) fn declared_member_attributes(
    bytes: &[u8],
    member: &MemberHeader,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<MemberAttributes> {
    let default = declared_annotation_default(bytes, member, pool, budget)?;
    let throws_raw = if declares_exceptions(member) {
        attribute_facts(
            bytes,
            &attribute_shells(member, b"Exceptions"),
            pool,
            budget,
        )?
        .exceptions
        .iter()
        .map(|name| name.0.clone())
        .collect()
    } else {
        Vec::new()
    };
    Ok(MemberAttributes {
        default,
        throws: throws_raw.iter().map(|name| class_name(name)).collect(),
        throws_raw,
    })
}

/// Decide one generic method-header projection from this member's own Signature, an already
/// published class-variable scope, and a typed return candidate from the same recovery run.
/// Syntax/erasure failures are refusals; budget and cancellation remain operation stops.
pub(crate) fn project_method_signature(
    record: &mut ClassSourceMethod,
    member: &MemberHeader,
    attributes: &MemberAttributes,
    candidate: Option<&GenericReturnCandidate>,
    constructor_candidate: Option<&GenericConstructorCandidate>,
    bytes: &[u8],
    pool: &[CpEntryFacts],
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Result<()> {
    let shells = attribute_shells(member, b"Signature");
    if shells.is_empty() {
        return project_member_inner_descriptor_path(record, candidate, budget);
    }
    let result = (|| -> Result<Option<(String, Vec<u8>, &'static str)>> {
        let facts = attribute_facts(bytes, &shells, pool, budget)?;
        let raw = facts
            .signature
            .ok_or_else(|| {
                Error::invalid_input(
                    "jvm_signature_missing",
                    "Signature attribute did not resolve",
                )
            })?
            .0;
        let parsed = parse_method_signature(&raw, budget)?;
        let name = &member.name.raw().0;
        record.enum_constructor_source_signature = shells.len() == 1
            && name.as_slice() == b"<init>"
            && member.descriptor.raw().0.as_slice() == b"(Ljava/lang/String;II)V"
            && parsed.type_parameters.is_empty()
            && parsed.parameters.as_slice() == [SignatureType::Base(b'I')]
            && parsed.result.is_none()
            && parsed.throws.is_empty();
        record.enum_constructor_no_arg_source_signature = shells.len() == 1
            && name.as_slice() == b"<init>"
            && member.descriptor.raw().0.as_slice() == b"(Ljava/lang/String;I)V"
            && parsed.type_parameters.is_empty()
            && parsed.parameters.is_empty()
            && parsed.result.is_none()
            && parsed.throws.is_empty();
        let erasure = prove_method_signature_erasure_with_class_scope(
            &parsed,
            &member.descriptor.raw().0,
            &attributes.throws_raw,
            class_scope,
            budget,
        )?;
        let no_body_generic = matches!(record.outcome, ClassSourceOutcome::NoBody)
            && !parsed.type_parameters.is_empty();
        let generic_constructor = name == b"<init>" && !parsed.type_parameters.is_empty();
        if no_body_generic {
            prove_no_body_generic_hierarchy(
                record,
                class_internal,
                class_flags,
                class_superclass,
                class_interfaces,
                budget,
            )?;
        }
        let body_generic_throws = matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
            && parsed.type_parameters.is_empty()
            && parsed
                .throws
                .iter()
                .any(|throws| matches!(throws, SignatureType::TypeVariable(_)));
        let body_method_local_generic_throws =
            matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
                && !parsed.type_parameters.is_empty()
                && member.descriptor.raw().0.as_slice() == b"()V"
                && parsed.parameters.is_empty()
                && parsed.result.is_none()
                && matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)]);
        let static_method_local_generic_throws =
            matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
                && !parsed.type_parameters.is_empty()
                && matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)]);
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(pool.len()).unwrap_or(u64::MAX),
        )?;
        if pool.iter().any(|entry| matches!(&entry.kind,
            CpEntryKind::MethodRef { owner, name: called, .. } | CpEntryKind::InterfaceMethodRef { owner, name: called, .. }
                if owner.0.as_slice() == class_internal && called.0.as_slice() == name.as_slice()
        )) {
            return Err(Error::unsupported("generic_call_binding_unproved", "a same-class Methodref names this method or an adjacent overload"));
        }
        let (declaration, proof) = if generic_constructor {
            (
                generic_constructor_declaration(
                    record,
                    attributes,
                    &parsed,
                    &erasure.type_parameters,
                    constructor_candidate,
                    class_internal,
                    class_flags,
                    class_superclass,
                    class_interfaces,
                    class_scope,
                    budget,
                )?,
                "same-run AST/SSA empty-constructor proof",
            )
        } else if no_body_generic {
            let declaration = no_body_generic_method_declaration(
                record,
                attributes,
                &parsed,
                &erasure.type_parameters,
                class_scope,
                budget,
            )?;
            (declaration, "physical no-body declaration proof")
        } else if body_method_local_generic_throws {
            let declaration = body_method_local_generic_throws_declaration(
                record,
                attributes,
                &parsed,
                candidate,
                class_internal,
                class_flags,
                class_superclass,
                class_interfaces,
                class_scope,
                class_signature_present,
                budget,
            )?;
            (
                declaration,
                "same-run AST/Code/SSA empty-void and method-local Signature scope/erasure proof",
            )
        } else if static_method_local_generic_throws {
            let declaration = static_method_local_generic_throws_declaration(
                record,
                attributes,
                &parsed,
                candidate,
                class_internal,
                class_flags,
                class_superclass,
                class_interfaces,
                class_scope,
                class_signature_present,
                budget,
            )?;
            (
                declaration,
                "same-run AST/SSA direct parameter return and method-local Signature scope/erasure proof",
            )
        } else if parsed.type_parameters.is_empty() {
            let declaration = ordinary_parameterized_declaration(
                record,
                attributes,
                &parsed,
                candidate,
                class_flags,
                class_internal,
                class_superclass,
                class_interfaces,
                class_scope,
                budget,
            )?;
            (
                declaration,
                if body_generic_throws {
                    "same-run AST/Code/SSA empty-void proof"
                } else {
                    "same-run AST/SSA parameter-return proof"
                },
            )
        } else {
            (
                generic_method_declaration(record, attributes, &parsed, candidate, false)?,
                "same-run AST/SSA parameter-return proof",
            )
        };
        Ok(Some((declaration, raw, proof)))
    })();
    match result {
        Ok(Some((declaration, signature, proof))) => {
            record.project_generic(declaration, &signature, proof, budget)
        }
        Ok(None) => Ok(()),
        Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => Err(error),
        Err(error) => {
            record.enum_constructor_signature_erasure_refused = (record
                .enum_constructor_source_signature
                || record.enum_constructor_no_arg_source_signature)
                && matches!(
                    &error,
                    Error::InvalidInput { code, .. }
                        if code == "jvm_signature_erasure_mismatch"
                );
            record.refuse_generic(&error.to_string(), budget)
        }
    }
}

/// Spell one generic constructor only after its same-run body candidate, Signature erasure, and
/// the class-level call-binding gates have all passed. The complete declaration is built before
/// `project_generic` publishes any part of it.
#[allow(clippy::too_many_arguments)]
fn generic_constructor_declaration(
    record: &ClassSourceMethod,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    method_scope: &[TypeParameterErasure],
    candidate: Option<&GenericConstructorCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<String> {
    let refused = |why| Error::unsupported("generic_constructor_source_unproved", why);
    let item = &record.item;
    if item.name.raw().0 != b"<init>"
        || record.declaration.is_none()
        || !matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
        || !record.markers.is_empty()
        || item.access_flags & !(ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED) != 0
        || attributes.default.is_some()
        || !attributes.throws_raw.is_empty()
        || !parsed.throws.is_empty()
        || parsed.result.is_some()
        || !record.annotations.attributes.is_empty()
        || !record.annotations.refusals.is_empty()
        || !record.parameter_annotations.attributes.is_empty()
        || !record.parameter_annotations.refusals.is_empty()
        || !record.type_annotations.attributes.is_empty()
        || !record.type_annotations.refusals.is_empty()
        || class_internal.contains(&b'$')
        || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
    {
        return Err(refused(
            "constructor flags, throws, annotations, nesting, or class hierarchy lack a faithful source position",
        ));
    }
    let candidate = candidate.ok_or_else(|| {
        refused("same-run AST/SSA does not prove the empty Object constructor body")
    })?;
    let this_class = std::str::from_utf8(class_internal)
        .map_err(|_| refused("constructor owner has no source UTF-8"))?;
    if !candidate.init.presented
        || candidate.init.target != Some(jarde_java::ast::ConstructorTarget::Super)
        || candidate.init.class.as_deref() != Some("java/lang/Object")
        || candidate.init.declared.as_deref() != Some(this_class)
    {
        return Err(refused(
            "same-run InitRecord does not prove Object() for this class",
        ));
    }
    let mut signature = method_descriptor(&item.descriptor.raw().0, false, false)
        .ok_or_else(|| refused("physical constructor descriptor cannot be spelled"))?;
    if signature.parameters.len() != parsed.parameters.len()
        || signature.parameters.len() != candidate.parameters.len()
        || parsed.type_parameters.is_empty()
    {
        return Err(refused(
            "constructor Signature and physical parameter positions differ",
        ));
    }
    let mut scope = Vec::with_capacity(class_scope.len() + method_scope.len());
    scope.extend_from_slice(method_scope);
    scope.extend_from_slice(class_scope);
    let mut type_parameters = Vec::with_capacity(parsed.type_parameters.len());
    for parameter in &parsed.type_parameters {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let name = std::str::from_utf8(&parameter.name)
            .map_err(|_| refused("constructor type parameter name is not source UTF-8"))?;
        if !is_java_identifier(name) {
            return Err(refused(
                "constructor type parameter has no Java source spelling",
            ));
        }
        let mut bounds = Vec::new();
        for bound in parameter
            .class_bound
            .iter()
            .chain(parameter.interface_bounds.iter())
        {
            bounds.push(spell_ordinary_signature_type(bound, &scope, budget, 0)?);
        }
        if bounds.is_empty() {
            bounds.push("java.lang.Object".to_owned());
        }
        type_parameters.push(format!("{name} extends {}", bounds.join(" & ")));
    }
    let mut parameter_types = Vec::with_capacity(parsed.parameters.len());
    for parameter in &parsed.parameters {
        parameter_types.push(spell_ordinary_signature_type(parameter, &scope, budget, 0)?);
    }
    for ((_, slot), (candidate_slot, name)) in
        signature.parameters.iter().zip(&candidate.parameters)
    {
        if slot != candidate_slot || !is_java_identifier(name) {
            return Err(refused(
                "same-run constructor parameter slot or name is unproved",
            ));
        }
    }
    for ((spelling, _), ty) in signature.parameters.iter_mut().zip(parameter_types) {
        *spelling = ty;
    }
    let class_name = simple_name(class_internal);
    if !is_java_identifier(&class_name) || class_name.contains('$') {
        return Err(refused("class name has no faithful constructor spelling"));
    }
    let args = candidate
        .parameters
        .iter()
        .zip(&signature.parameters)
        .map(|((_, name), (ty, _))| format!("{ty} {name}"))
        .collect::<Vec<_>>();
    let mut words = visibility(item.access_flags)
        .map(str::to_owned)
        .into_iter()
        .collect::<Vec<_>>();
    if !type_parameters.is_empty() {
        words.push(format!("<{}>", type_parameters.join(", ")));
    }
    let prefix = if words.is_empty() {
        String::new()
    } else {
        format!("{} ", words.join(" "))
    };
    Ok(format!("{prefix}{class_name}({})", args.join(", ")))
}

/// The initial no-body projection proves the class has no inherited source contract to satisfy.
/// `$` in a binary name is conservatively treated as nested because this class-source view does not
/// publish an InnerClasses nesting proof.
fn prove_no_body_generic_hierarchy(
    record: &ClassSourceMethod,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    budget: &mut Budget,
) -> Result<()> {
    let refused = |why| Error::unsupported("generic_inherited_contract_unproved", why);
    budget.poll()?;
    if !matches!(record.no_body_kind, Some(NoBodyKind::Abstract))
        || class_flags & (ACC_ANNOTATION | ACC_ENUM) != 0
        || class_internal.contains(&b'$')
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
    {
        return Err(refused(
            "no-body generic projection requires a top-level class or root interface with Object as its physical superclass and no interfaces",
        ));
    }
    let name = &record.item.name.raw().0;
    if is_object_instance_method_name(name) {
        return Err(refused(
            "method name may override an Object instance method",
        ));
    }
    Ok(())
}

/// Re-spell descriptor positions only when the same-run body returned the selected member
/// creation. This is the raw-signature slice: the body's target and qualifier slot constrain every
/// replacement, and the physical descriptor still controls all other positions.
fn project_member_inner_descriptor_path(
    record: &mut ClassSourceMethod,
    candidate: Option<&GenericReturnCandidate>,
    budget: &mut Budget,
) -> Result<()> {
    let Some(GenericReturnValue::MemberCreation {
        target,
        qualifier_slot,
    }) = candidate.map(|candidate| &candidate.value)
    else {
        return Ok(());
    };
    let refused = |why| Error::unsupported("member_inner_source_path_unproved", why);
    if record.declaration.is_none()
        || !record.markers.is_empty()
        || !matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
    {
        return Err(refused(
            "the same-run member creation has no complete recovered declaration to project",
        ));
    }
    let signature = method_descriptor(
        &record.item.descriptor.raw().0,
        is_static(record.item.access_flags),
        record.item.access_flags & ACC_VARARGS != 0,
    )
    .ok_or_else(|| refused("physical descriptor cannot be spelled"))?;
    if candidate.is_some_and(|candidate| candidate.parameters.len() != signature.parameters.len()) {
        return Err(refused(
            "same-run parameter positions differ from the descriptor",
        ));
    }
    let Some((qualifier_type, _)) = signature
        .parameters
        .iter()
        .find(|(_, slot)| slot == qualifier_slot)
    else {
        return Err(refused("member qualifier is not a method parameter slot"));
    };
    let Some(outer) = target
        .source_type_path
        .iter()
        .find(|segment| segment.binary_name == target.outer)
    else {
        return Err(refused("selected source path has no enclosing member type"));
    };
    let Some(member) = target
        .source_type_path
        .iter()
        .find(|segment| segment.binary_name == target.owner)
    else {
        return Err(refused("selected source path has no target member type"));
    };
    if qualifier_type != &target.outer.replace('/', ".")
        || member.enclosing_binary_name.as_deref() != Some(target.outer.as_str())
        || member.is_static
        || !outer.is_static
        || target
            .source_type_path
            .last()
            .is_none_or(|last| last != member)
    {
        return Err(refused(
            "selected qualifier, enclosing type, and member path do not agree",
        ));
    }
    if let Some(result) = &signature.returns
        && result != "java.lang.Object"
        && result != &target.owner.replace('/', ".")
    {
        return Err(refused(
            "physical result is neither Object nor the selected member type",
        ));
    }
    for ((_, descriptor_slot), (candidate_slot, name)) in signature
        .parameters
        .iter()
        .zip(&candidate.expect("candidate matched").parameters)
    {
        if descriptor_slot != candidate_slot || !is_java_identifier(name) {
            return Err(refused("same-run parameter names or slots are incomplete"));
        }
    }

    let mut declaration = record.declaration.clone().expect("checked above");
    let mut mapped_segments = target.source_type_path.iter().collect::<Vec<_>>();
    mapped_segments.sort_by_key(|segment| std::cmp::Reverse(segment.binary_name.len()));
    let mut changed = false;
    for segment in mapped_segments {
        let physical_name = segment.binary_name.replace('/', ".");
        let expected_count = signature
            .parameters
            .iter()
            .filter(|(ty, _)| descriptor_component_base(ty) == physical_name)
            .count()
            + usize::from(
                signature
                    .returns
                    .as_deref()
                    .is_some_and(|ty| descriptor_component_base(ty) == physical_name),
            );
        if expected_count == 0 {
            continue;
        }
        let (rewritten, count) =
            replace_java_type_token(&declaration, &physical_name, &segment.source_name);
        if count != expected_count {
            return Err(refused(
                "declaration type positions do not match the physical member descriptor",
            ));
        }
        declaration = rewritten;
        changed = true;
    }
    if !changed {
        return Err(refused(
            "selected member source path does not occur in the physical method declaration",
        ));
    }
    // Keep the projection explicit beside the recovered method so readers can see why the `$`
    // spelling changed. The block itself is reused verbatim from the same run.
    let marker = format!(
        "// jarde: descriptor type path for `{}` follows the same-run proved member creation",
        target.owner
    );
    let mut markers = record.markers.clone();
    markers.push(marker);
    let report = match &record.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        _ => unreachable!("checked recovered outcome above"),
    };
    let Some(block) = artifact(&report.text) else {
        return Err(refused("same-run body artifact has no complete block"));
    };
    let text = prefix_method_annotations(
        block_member(&declaration, Placed::Block(block), &markers),
        &record.annotations,
    );
    budget.charge(
        CountedBudgetDimension::OutputBytes,
        u64::try_from(text.len()).unwrap_or(u64::MAX),
    )?;
    record.declaration = Some(declaration);
    record.text = text;
    record.markers = markers;
    Ok(())
}

fn descriptor_component_base(ty: &str) -> &str {
    ty.strip_suffix("[]").unwrap_or(ty)
}

fn replace_java_type_token(text: &str, from: &str, to: &str) -> (String, usize) {
    let is_identifier = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$');
    let bytes = text.as_bytes();
    let needle = from.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut count = 0;
    while cursor < bytes.len() {
        let Some(relative) = text[cursor..].find(from) else {
            output.push_str(&text[cursor..]);
            break;
        };
        let start = cursor + relative;
        let end = start + needle.len();
        let left_is_boundary = start == 0 || !is_identifier(bytes[start - 1]);
        let right_is_boundary = end == bytes.len() || !is_identifier(bytes[end]);
        if left_is_boundary && right_is_boundary {
            output.push_str(&text[cursor..start]);
            output.push_str(to);
            cursor = end;
            count += 1;
        } else {
            let next = start + from.len();
            output.push_str(&text[cursor..next]);
            cursor = next;
        }
    }
    (output, count)
}

fn is_object_instance_method_name(name: &[u8]) -> bool {
    const OBJECT_INSTANCE_METHODS: &[&[u8]] = &[
        b"getClass",
        b"hashCode",
        b"equals",
        b"clone",
        b"toString",
        b"notify",
        b"notifyAll",
        b"wait",
        b"finalize",
    ];
    OBJECT_INSTANCE_METHODS.contains(&name)
}

#[allow(clippy::too_many_arguments)]
fn prove_body_generic_throws_shape(
    record: &ClassSourceMethod,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    budget: &mut Budget,
) -> Result<()> {
    let refused = |why| Error::unsupported("body_generic_throws_source_unproved", why);
    let item = &record.item;
    if !matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
        || !matches!(item.descriptor.raw().0.as_slice(), b"()V")
        || item.name.raw().0 == b"<init>"
        || item.name.raw().0 == b"<clinit>"
        || is_object_instance_method_name(&item.name.raw().0)
        || item.access_flags & (ACC_ABSTRACT | ACC_NATIVE | ACC_VARARGS) != 0
        || class_internal.contains(&b'$')
        || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
        || !parsed.type_parameters.is_empty()
        || !parsed.parameters.is_empty()
        || parsed.result.is_some()
        || !matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)])
    {
        return Err(refused(
            "method Signature, member, or class hierarchy exceeds the supported empty-void shape",
        ));
    }
    if !matches!(
        candidate,
        Some(GenericReturnCandidate {
            parameters,
            value: GenericReturnValue::EmptyVoid,
        }) if parameters.is_empty()
    ) {
        return Err(refused(
            "same-run AST/Code/SSA does not prove the exact empty void body",
        ));
    }
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn prove_body_method_local_generic_throws_shape(
    record: &ClassSourceMethod,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Result<()> {
    let refused = |why| Error::unsupported("body_method_local_generic_throws_source_unproved", why);
    let item = &record.item;
    if !matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
        || record.declaration.is_none()
        || !record.markers.is_empty()
        || !record.annotations.refusals.is_empty()
        || !record.parameter_annotations.refusals.is_empty()
        || !record.type_annotations.attributes.is_empty()
        || !record.type_annotations.refusals.is_empty()
        || item.access_flags
            & !(ACC_PUBLIC
                | ACC_PRIVATE
                | ACC_PROTECTED
                | ACC_FINAL
                | ACC_SYNCHRONIZED
                | ACC_STRICT)
            != 0
        || item.descriptor.raw().0.as_slice() != b"()V"
        || matches!(item.name.raw().0.as_slice(), b"<init>" | b"<clinit>")
        || is_object_instance_method_name(&item.name.raw().0)
        || class_internal.contains(&b'$')
        || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
        || !class_scope.is_empty()
        || class_signature_present
        || !parsed.parameters.is_empty()
        || parsed.result.is_some()
        || parsed.type_parameters.len() != 1
        || !matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)])
    {
        return Err(refused(
            "method Signature, member, or class hierarchy exceeds the supported empty-void shape",
        ));
    }
    if !matches!(
        candidate,
        Some(GenericReturnCandidate {
            parameters,
            value: GenericReturnValue::EmptyVoid,
        }) if parameters.is_empty()
    ) {
        return Err(refused(
            "same-run AST/Code/SSA does not prove the exact empty void body",
        ));
    }
    let [parameter] = parsed.type_parameters.as_slice() else {
        unreachable!("shape checked above")
    };
    let bounds = parameter
        .class_bound
        .iter()
        .chain(parameter.interface_bounds.iter())
        .collect::<Vec<_>>();
    let [SignatureType::Class(bound)] = bounds.as_slice() else {
        return Err(refused(
            "method variable must have exactly one proved class bound and no additional bounds",
        ));
    };
    let [segment] = bound.segments.as_slice() else {
        return Err(refused("method variable bound is nested"));
    };
    if !segment.arguments.is_empty()
        || !matches!(
            segment.binary_name.as_slice(),
            b"java/lang/Throwable"
                | b"java/lang/Exception"
                | b"java/lang/RuntimeException"
                | b"java/lang/Error"
        )
    {
        return Err(refused(
            "method variable bound is not a proved JDK throwable root",
        ));
    }
    let name = std::str::from_utf8(&parameter.name)
        .map_err(|_| refused("method variable name is not source UTF-8"))?;
    if !is_java_identifier(name)
        || parsed.throws.first() != Some(&SignatureType::TypeVariable(parameter.name.clone()))
    {
        return Err(refused(
            "throws variable is not the source-spellable method-local variable",
        ));
    }
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn body_method_local_generic_throws_declaration(
    record: &ClassSourceMethod,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Result<String> {
    prove_body_method_local_generic_throws_shape(
        record,
        parsed,
        candidate,
        class_internal,
        class_flags,
        class_superclass,
        class_interfaces,
        class_scope,
        class_signature_present,
        budget,
    )?;
    if attributes.default.is_some() {
        return Err(Error::unsupported(
            "body_method_local_generic_throws_source_unproved",
            "annotation default has no source position in this method shape",
        ));
    }
    let [parameter] = parsed.type_parameters.as_slice() else {
        return Err(Error::unsupported(
            "body_method_local_generic_throws_source_unproved",
            "method must declare exactly one local type variable",
        ));
    };
    let variable_name = std::str::from_utf8(&parameter.name).map_err(|_| {
        Error::unsupported(
            "body_method_local_generic_throws_source_unproved",
            "method variable name is not source UTF-8",
        )
    })?;
    let bound_internal = match parameter.class_bound.as_ref() {
        Some(SignatureType::Class(bound)) => &bound.segments[0].binary_name,
        _ => unreachable!("validated by shape proof"),
    };
    let bound = simple_generic_class_name(bound_internal)?;
    let signature =
        method_descriptor(&record.item.descriptor.raw().0, false, false).ok_or_else(|| {
            Error::unsupported(
                "body_method_local_generic_throws_source_unproved",
                "physical method descriptor cannot be spelled",
            )
        })?;
    let (method_name, aliased) = written_name(&record.item.name.raw().0);
    if aliased || !is_java_identifier(&method_name) {
        return Err(Error::unsupported(
            "body_method_local_generic_throws_source_unproved",
            "method name has no faithful Java spelling",
        ));
    }
    Ok(format_generic_method_header(
        record,
        &signature,
        &method_name,
        &[format!("{variable_name} extends {bound}")],
        &[variable_name.to_owned()],
    ))
}

#[allow(clippy::too_many_arguments)]
fn static_method_local_generic_throws_declaration(
    record: &ClassSourceMethod,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Result<String> {
    let refused = |why| Error::unsupported("generic_source_shape_unproved", why);
    let item = &record.item;
    if !matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
        || item.access_flags & ACC_STATIC == 0
        || class_internal.contains(&b'$')
        || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
        || !class_scope.is_empty()
        || class_signature_present
        || !matches!(
            candidate,
            Some(GenericReturnCandidate {
                value: GenericReturnValue::Parameter(_),
                ..
            })
        )
        || !matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)])
    {
        return Err(refused(
            "method-local generic throws requires a direct parameter return in a top-level, non-generic Object subclass with no interfaces",
        ));
    }
    let [SignatureType::TypeVariable(throws_variable)] = parsed.throws.as_slice() else {
        unreachable!("shape checked above")
    };
    let mut matching_parameters = parsed
        .type_parameters
        .iter()
        .filter(|parameter| parameter.name == *throws_variable);
    let Some(parameter) = matching_parameters.next() else {
        return Err(refused(
            "throws variable is outside the method-local type parameter scope",
        ));
    };
    if matching_parameters.next().is_some() {
        return Err(refused(
            "throws variable does not identify a unique method-local type parameter",
        ));
    }
    if !parameter.interface_bounds.is_empty() {
        return Err(refused(
            "throws variable cannot have additional interface bounds",
        ));
    }
    let Some(SignatureType::Class(bound)) = parameter.class_bound.as_ref() else {
        return Err(refused(
            "throws variable must have one explicit class first bound",
        ));
    };
    let [segment] = bound.segments.as_slice() else {
        return Err(refused("throws variable bound is nested"));
    };
    if !segment.arguments.is_empty()
        || !matches!(
            segment.binary_name.as_slice(),
            b"java/lang/Throwable"
                | b"java/lang/Exception"
                | b"java/lang/RuntimeException"
                | b"java/lang/Error"
        )
    {
        return Err(refused(
            "throws variable bound is not a proved JDK throwable root",
        ));
    }
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    generic_method_declaration(record, attributes, parsed, candidate, true)
}

fn no_body_generic_method_declaration(
    record: &ClassSourceMethod,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    method_scope: &[TypeParameterErasure],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<String> {
    let refused = |why| Error::unsupported("generic_source_shape_unproved", why);
    let item = &record.item;
    if record.declaration.is_none()
        || !matches!(record.no_body_kind, Some(NoBodyKind::Abstract))
        || matches!(item.name.raw().0.as_slice(), b"<init>" | b"<clinit>")
        || item.access_flags
            & !(ACC_PUBLIC
                | ACC_PRIVATE
                | ACC_PROTECTED
                | ACC_STATIC
                | ACC_FINAL
                | ACC_ABSTRACT
                | ACC_NATIVE
                | ACC_SYNCHRONIZED
                | ACC_STRICT
                | ACC_VARARGS)
            != 0
        || attributes.default.is_some()
        || !record.annotations.refusals.is_empty()
        || !record.parameter_annotations.refusals.is_empty()
        || !record.type_annotations.attributes.is_empty()
        || !record.type_annotations.refusals.is_empty()
    {
        return Err(refused(
            "method flags, annotations, or source name cannot be preserved",
        ));
    }
    let mut scope = Vec::with_capacity(class_scope.len() + method_scope.len());
    scope.extend_from_slice(class_scope);
    scope.extend_from_slice(method_scope);
    let mut signature = method_descriptor(
        &item.descriptor.raw().0,
        is_static(item.access_flags),
        item.access_flags & ACC_VARARGS != 0,
    )
    .ok_or_else(|| refused("physical method descriptor cannot be spelled"))?;
    if signature.parameters.len() != parsed.parameters.len() {
        return Err(refused("generic and physical parameter positions differ"));
    }
    let mut parameters = Vec::with_capacity(parsed.parameters.len());
    for parameter in &parsed.parameters {
        parameters.push(spell_ordinary_signature_type(parameter, &scope, budget, 0)?);
    }
    let result = parsed
        .result
        .as_ref()
        .map(|result| spell_ordinary_signature_type(result, &scope, budget, 0))
        .transpose()?;
    let mut type_parameters = Vec::with_capacity(parsed.type_parameters.len());
    for parameter in &parsed.type_parameters {
        let mut bounds = Vec::new();
        for bound in parameter
            .class_bound
            .iter()
            .chain(parameter.interface_bounds.iter())
        {
            bounds.push(spell_ordinary_signature_type(bound, &scope, budget, 0)?);
        }
        let name = std::str::from_utf8(&parameter.name)
            .map_err(|_| refused("type parameter name is not source UTF-8"))?;
        if !is_java_identifier(name) {
            return Err(refused("type parameter has no Java source spelling"));
        }
        if bounds.is_empty() {
            bounds.push("java.lang.Object".to_owned());
        }
        type_parameters.push(format!("{name} extends {}", bounds.join(" & ")));
    }
    let mut generic_throws = Vec::with_capacity(parsed.throws.len());
    for exception in &parsed.throws {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        match exception {
            SignatureType::TypeVariable(variable) => {
                let Some(first_bound) = scope.iter().find(|entry| entry.name == *variable) else {
                    return Err(refused(
                        "throws variable is outside the emitted class and method scopes",
                    ));
                };
                if !matches!(
                    first_bound.descriptor.as_slice(),
                    b"Ljava/lang/Throwable;"
                        | b"Ljava/lang/Exception;"
                        | b"Ljava/lang/RuntimeException;"
                        | b"Ljava/lang/Error;"
                ) {
                    return Err(refused(
                        "throws variable first bound is not a proved JDK throwable root",
                    ));
                }
                generic_throws.push(
                    std::str::from_utf8(variable)
                        .map_err(|_| refused("throws variable name is not source UTF-8"))?
                        .to_owned(),
                );
            }
            SignatureType::Class(class) => {
                let [segment] = class.segments.as_slice() else {
                    return Err(refused(
                        "nested throws class has no preserved source position",
                    ));
                };
                if !segment.arguments.is_empty() {
                    return Err(refused(
                        "parameterized throws class is not a legal Java exception type",
                    ));
                }
                generic_throws.push(simple_generic_class_name(&segment.binary_name)?);
            }
            _ => return Err(refused("throws type has no supported Java source spelling")),
        }
    }
    let throws = if parsed.throws.is_empty() {
        attributes.throws.clone()
    } else {
        generic_throws
    };
    for ((spelling, _), ty) in signature.parameters.iter_mut().zip(parameters) {
        *spelling = ty;
    }
    signature.returns = result;
    let (name, aliased) = written_name(&item.name.raw().0);
    if aliased || !is_java_identifier(&name) {
        return Err(refused("method name has no faithful Java spelling"));
    }
    Ok(format_generic_method_header(
        record,
        &signature,
        &name,
        &type_parameters,
        &throws,
    ))
}

/// Assemble a fully spelled generic method head after its source types and proof gates have
/// succeeded. No-body methods and recovered empty-void methods share this final spelling path.
fn format_generic_method_header(
    record: &ClassSourceMethod,
    signature: &Signature,
    name: &str,
    type_parameters: &[String],
    throws: &[String],
) -> String {
    let flags = record.item.access_flags;
    let mut words = Vec::new();
    words.extend(visibility(flags));
    if flags & ACC_STATIC != 0 {
        words.push("static");
    }
    if flags & ACC_FINAL != 0 {
        words.push("final");
    }
    if flags & ACC_ABSTRACT != 0 {
        words.push("abstract");
    }
    if flags & ACC_NATIVE != 0 {
        words.push("native");
    }
    if flags & ACC_SYNCHRONIZED != 0 {
        words.push("synchronized");
    }
    if flags & ACC_STRICT != 0 {
        words.push("strictfp");
    }
    let mut arguments = Vec::with_capacity(signature.parameters.len());
    for (position, (ty, slot)) in signature.parameters.iter().enumerate() {
        let annotations = record
            .parameter_annotations
            .uses_by_position
            .get(position)
            .into_iter()
            .flatten()
            .map(|annotation| format!("{annotation} "))
            .collect::<String>();
        let varargs = signature.varargs && position + 1 == signature.parameters.len();
        arguments.push(format!(
            "{annotations}{} arg{slot}",
            varargs_type(ty, varargs)
        ));
    }
    let mut declaration = String::new();
    if !words.is_empty() {
        declaration.push_str(&words.join(" "));
        declaration.push(' ');
    }
    if !type_parameters.is_empty() {
        declaration.push('<');
        declaration.push_str(&type_parameters.join(", "));
        declaration.push_str("> ");
    }
    declaration.push_str(signature.returns.as_deref().unwrap_or("void"));
    declaration.push(' ');
    declaration.push_str(name);
    declaration.push('(');
    declaration.push_str(&arguments.join(", "));
    declaration.push(')');
    declaration.push_str(&throws_clause(throws));
    declaration
}

/// Spell ordinary parameterized method types only after the reader has proved their erasures.
/// A recovered body's candidate is the same Program/SSA reading that supplied its parameter
/// names; no-body members have no Program and use descriptor slot names.
fn ordinary_parameterized_declaration(
    record: &ClassSourceMethod,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_flags: u16,
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<String> {
    let refused = |why| Error::unsupported("ordinary_generic_source_unproved", why);
    let item = &record.item;
    if record.declaration.is_none()
        || matches!(item.name.raw().0.as_slice(), b"<init>" | b"<clinit>")
        || item.access_flags
            & !(ACC_PUBLIC
                | ACC_PRIVATE
                | ACC_PROTECTED
                | ACC_STATIC
                | ACC_FINAL
                | ACC_ABSTRACT
                | ACC_NATIVE
                | ACC_SYNCHRONIZED
                | ACC_STRICT
                | ACC_VARARGS)
            != 0
        || attributes.default.is_some()
        || !record.annotations.refusals.is_empty()
        || !record.parameter_annotations.refusals.is_empty()
        || !record.type_annotations.attributes.is_empty()
        || !record.type_annotations.refusals.is_empty()
    {
        return Err(refused(
            "member flags, annotation positions, or source name cannot be preserved",
        ));
    }
    match &record.outcome {
        ClassSourceOutcome::Recovered { .. } if record.markers.is_empty() => {}
        ClassSourceOutcome::NoBody if record.no_body_kind.is_some() => {}
        _ => {
            return Err(refused(
                "method body or no-body declaration has no complete source proof",
            ));
        }
    }
    let mut signature = method_descriptor(
        &item.descriptor.raw().0,
        is_static(item.access_flags),
        item.access_flags & ACC_VARARGS != 0,
    )
    .ok_or_else(|| refused("physical method descriptor cannot be spelled"))?;
    if signature.parameters.len() != parsed.parameters.len() {
        return Err(refused("generic and physical parameter positions differ"));
    }
    let source_type_path = match candidate.map(|candidate| &candidate.value) {
        Some(GenericReturnValue::MemberCreation { target, .. }) => {
            Some(target.source_type_path.as_slice())
        }
        _ => None,
    };
    let mut parameter_types = Vec::with_capacity(parsed.parameters.len());
    for parameter in &parsed.parameters {
        parameter_types.push(spell_ordinary_signature_type_with_member_path(
            parameter,
            class_scope,
            source_type_path.unwrap_or(&[]),
            budget,
            0,
        )?);
    }
    let result = parsed
        .result
        .as_ref()
        .map(|result| {
            spell_ordinary_signature_type_with_member_path(
                result,
                class_scope,
                source_type_path.unwrap_or(&[]),
                budget,
                0,
            )
        })
        .transpose()?;
    let mut generic_throws = Vec::with_capacity(parsed.throws.len());
    for exception in &parsed.throws {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        match exception {
            SignatureType::Class(class) => {
                let [segment] = class.segments.as_slice() else {
                    return Err(refused(
                        "nested throws type has no preserved source position",
                    ));
                };
                if !segment.arguments.is_empty() {
                    return Err(refused(
                        "parameterized throws type has no preserved source position",
                    ));
                }
                generic_throws.push(simple_generic_class_name(&segment.binary_name)?);
            }
            SignatureType::TypeVariable(variable) => {
                match &record.outcome {
                    ClassSourceOutcome::NoBody => {}
                    ClassSourceOutcome::Recovered { .. } => prove_body_generic_throws_shape(
                        record,
                        parsed,
                        candidate,
                        class_internal,
                        class_flags,
                        class_superclass,
                        class_interfaces,
                        budget,
                    )?,
                    _ => {
                        return Err(refused(
                            "generic throws variable has no complete body or no-body proof",
                        ));
                    }
                }
                let Some(class_variable) = class_scope.iter().find(|item| item.name == *variable)
                else {
                    return Err(refused(
                        "generic throws variable is not present in the emitted class header",
                    ));
                };
                if !matches!(
                    class_variable.descriptor.as_slice(),
                    b"Ljava/lang/Throwable;"
                        | b"Ljava/lang/Exception;"
                        | b"Ljava/lang/RuntimeException;"
                        | b"Ljava/lang/Error;"
                ) {
                    return Err(refused(
                        "generic throws variable first bound is not a proved JDK throwable root",
                    ));
                }
                let name = std::str::from_utf8(variable)
                    .map_err(|_| refused("generic throws variable name is not source UTF-8"))?;
                if !is_java_identifier(name) {
                    return Err(refused(
                        "generic throws variable has no Java source spelling",
                    ));
                }
                generic_throws.push(name.to_owned());
            }
            _ => {
                return Err(refused(
                    "generic throws type has no preserved Java source spelling",
                ));
            }
        }
    }
    let throws = if parsed.throws.is_empty() {
        attributes.throws.clone()
    } else {
        generic_throws
    };
    if let ClassSourceOutcome::Recovered { .. } = &record.outcome {
        let candidate = candidate.ok_or_else(|| {
            refused("same-run Program/SSA cannot prove the body under parameterized types")
        })?;
        if candidate.parameters.len() != signature.parameters.len() {
            return Err(refused("same-run parameter count differs from descriptor"));
        }
        for ((_, slot), (candidate_slot, name)) in
            signature.parameters.iter().zip(&candidate.parameters)
        {
            if slot != candidate_slot || !is_java_identifier(name) {
                return Err(refused(
                    "same-run parameter slot or source name is unproved",
                ));
            }
        }
        let returned = match &candidate.value {
            GenericReturnValue::EmptyVoid
                if parsed.result.is_none()
                    && matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)]) =>
            {
                Some(Vec::new())
            }
            GenericReturnValue::EmptyVoid => None,
            GenericReturnValue::Parameter(slot) => signature
                .parameters
                .iter()
                .position(|(_, at)| *at == *slot)
                .map(|position| vec![position]),
            GenericReturnValue::Conditional {
                test,
                when_true,
                when_false,
            } => {
                let test_position = signature.parameters.iter().position(|(_, at)| *at == *test);
                let true_position = signature
                    .parameters
                    .iter()
                    .position(|(_, at)| *at == *when_true);
                let false_position = signature
                    .parameters
                    .iter()
                    .position(|(_, at)| *at == *when_false);
                match (test_position, true_position, false_position) {
                    (Some(test_position), Some(true_position), Some(false_position))
                        if parsed.parameters[test_position] == SignatureType::Base(b'Z') =>
                    {
                        Some(vec![true_position, false_position])
                    }
                    _ => None,
                }
            }
            GenericReturnValue::MemberCreation {
                target,
                qualifier_slot,
            } => {
                let qualifier_position = signature
                    .parameters
                    .iter()
                    .position(|(_, slot)| *slot == *qualifier_slot);
                qualifier_position
                    .is_some_and(|position| {
                        let parameter_matches = parsed.parameters.get(position).is_some_and(|ty| {
                            member_signature_class_matches(
                                ty,
                                &target.outer,
                                &target.source_type_path,
                                false,
                            )
                        });
                        parameter_matches
                            && parsed.result.as_ref().is_some_and(|ty| {
                                signature_result_matches_member_creation(ty, target)
                            })
                    })
                    .then_some(Vec::new())
            }
        };
        let returned = returned.ok_or_else(|| {
            refused("return source is not a proven parameter value or selected member creation")
        })?;
        if !matches!(candidate.value, GenericReturnValue::MemberCreation { .. })
            && returned
                .iter()
                .any(|position| parsed.result.as_ref() != Some(&parsed.parameters[*position]))
        {
            return Err(refused(
                "returned parameter has a different generic source type",
            ));
        }
    }
    for ((spelling, _), ty) in signature.parameters.iter_mut().zip(parameter_types) {
        *spelling = ty;
    }
    signature.returns = result;
    let (name, aliased) = written_name(&item.name.raw().0);
    if aliased || !is_java_identifier(&name) {
        return Err(refused("method name has no faithful Java spelling"));
    }
    let mut words = Vec::new();
    words.extend(visibility(item.access_flags));
    if is_static(item.access_flags) {
        words.push("static");
    }
    if item.access_flags & ACC_FINAL != 0 {
        words.push("final");
    }
    if item.access_flags & ACC_ABSTRACT != 0 {
        words.push("abstract");
    }
    if item.access_flags & ACC_NATIVE != 0 {
        words.push("native");
    }
    if item.access_flags & ACC_SYNCHRONIZED != 0 {
        words.push("synchronized");
    }
    if item.access_flags & ACC_STRICT != 0 {
        words.push("strictfp");
    }
    if is_default_member(class_flags, item) {
        words.push("default");
    }
    let names = candidate.map(|candidate| candidate.parameters.as_slice());
    let mut arguments = Vec::new();
    for (position, (ty, slot)) in signature.parameters.iter().enumerate() {
        let name = names
            .and_then(|names| names.get(position).map(|(_, name)| name.clone()))
            .unwrap_or_else(|| format!("arg{slot}"));
        let varargs = signature.varargs && position + 1 == signature.parameters.len();
        let annotations = record
            .parameter_annotations
            .uses_by_position
            .get(position)
            .into_iter()
            .flatten()
            .map(|annotation| format!("{annotation} "))
            .collect::<String>();
        arguments.push(format!("{annotations}{} {name}", varargs_type(ty, varargs)));
    }
    Ok(format!(
        "{}{} {}({}){}",
        if words.is_empty() {
            String::new()
        } else {
            format!("{} ", words.join(" "))
        },
        signature.returns.as_deref().unwrap_or("void"),
        name,
        arguments.join(", "),
        throws_clause(&throws),
    ))
}

fn spell_ordinary_signature_type(
    ty: &SignatureType,
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
    depth: usize,
) -> Result<String> {
    spell_ordinary_signature_type_with_member_path(ty, class_scope, &[], budget, depth)
}

fn spell_ordinary_signature_type_with_member_path(
    ty: &SignatureType,
    class_scope: &[TypeParameterErasure],
    source_type_path: &[jarde_java::report::ProvedMemberInnerSourceSegment],
    budget: &mut Budget,
    depth: usize,
) -> Result<String> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if depth > 128 {
        return Err(Error::unsupported(
            "ordinary_generic_source_unproved",
            "type nesting exceeds source projection depth",
        ));
    }
    let nested = depth + 1;
    match ty {
        SignatureType::Base(code) => Ok(match code {
            b'B' => "byte",
            b'C' => "char",
            b'D' => "double",
            b'F' => "float",
            b'I' => "int",
            b'J' => "long",
            b'S' => "short",
            b'Z' => "boolean",
            _ => {
                return Err(Error::unsupported(
                    "ordinary_generic_source_unproved",
                    "unknown primitive signature type",
                ));
            }
        }
        .to_owned()),
        SignatureType::Array(element) => Ok(format!(
            "{}[]",
            spell_ordinary_signature_type_with_member_path(
                element,
                class_scope,
                source_type_path,
                budget,
                nested,
            )?
        )),
        SignatureType::TypeVariable(name) => class_scope
            .iter()
            .find(|variable| variable.name == *name)
            .and_then(|variable| std::str::from_utf8(&variable.name).ok())
            .filter(|name| is_java_identifier(name))
            .map(str::to_owned)
            .ok_or_else(|| {
                Error::unsupported(
                    "ordinary_generic_source_unproved",
                    "type variable is not present in the emitted class header",
                )
            }),
        SignatureType::Class(class) => {
            if class.segments.is_empty() {
                return Err(Error::unsupported(
                    "ordinary_generic_source_unproved",
                    "nested binary class path has no unique Java source spelling",
                ));
            }
            let mut result = String::new();
            let mut previous_mapping: Option<&jarde_java::report::ProvedMemberInnerSourceSegment> =
                None;
            for segment in &class.segments {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                let mapping = unique_member_source_segment(source_type_path, &segment.binary_name)?;
                if let Some(mapping) = mapping {
                    if !segment.arguments.is_empty()
                        && segment.arguments.len() != mapping.type_parameter_count
                    {
                        return Err(Error::unsupported(
                            "ordinary_generic_source_unproved",
                            "selected member type has a different number of source arguments",
                        ));
                    }
                    if mapping.is_static || mapping.enclosing_binary_name.is_none() {
                        result = mapping.source_name.clone();
                    } else {
                        let previous = previous_mapping.ok_or_else(|| {
                            Error::unsupported(
                                "ordinary_generic_source_unproved",
                                "non-static member type has no explicit enclosing Signature segment",
                            )
                        })?;
                        if mapping.enclosing_binary_name.as_deref()
                            != Some(previous.binary_name.as_str())
                            || !segment_is_immediate_source_child(
                                &mapping.source_name,
                                &previous.source_name,
                            )
                        {
                            return Err(Error::unsupported(
                                "ordinary_generic_source_unproved",
                                "selected non-static member does not follow the Signature's enclosing segment",
                            ));
                        }
                        let suffix = mapping
                            .source_name
                            .strip_prefix(&previous.source_name)
                            .and_then(|suffix| suffix.strip_prefix('.'))
                            .ok_or_else(|| {
                                Error::unsupported(
                                    "ordinary_generic_source_unproved",
                                    "selected member source path is inconsistent",
                                )
                            })?;
                        if !is_java_identifier(suffix) {
                            return Err(Error::unsupported(
                                "ordinary_generic_source_unproved",
                                "selected member source segment is not a Java identifier",
                            ));
                        }
                        result.push('.');
                        result.push_str(suffix);
                    }
                    previous_mapping = Some(mapping);
                } else {
                    if class.segments.len() != 1 {
                        return Err(Error::unsupported(
                            "ordinary_generic_source_unproved",
                            "nested binary class path has no selected Java source spelling",
                        ));
                    }
                    result = simple_generic_class_name(&segment.binary_name)?;
                    previous_mapping = None;
                }
                if !segment.arguments.is_empty() {
                    let mut arguments = Vec::with_capacity(segment.arguments.len());
                    for argument in &segment.arguments {
                        budget.poll()?;
                        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                        let spelling = match argument {
                            TypeArgument::Any => "?".to_owned(),
                            TypeArgument::Exact(ty) => {
                                spell_ordinary_signature_type_with_member_path(
                                    ty,
                                    class_scope,
                                    source_type_path,
                                    budget,
                                    nested,
                                )?
                            }
                            TypeArgument::Extends(ty) => format!(
                                "? extends {}",
                                spell_ordinary_signature_type_with_member_path(
                                    ty,
                                    class_scope,
                                    source_type_path,
                                    budget,
                                    nested,
                                )?
                            ),
                            TypeArgument::Super(ty) => format!(
                                "? super {}",
                                spell_ordinary_signature_type_with_member_path(
                                    ty,
                                    class_scope,
                                    source_type_path,
                                    budget,
                                    nested,
                                )?
                            ),
                        };
                        arguments.push(spelling);
                    }
                    result.push('<');
                    result.push_str(&arguments.join(", "));
                    result.push('>');
                }
            }
            Ok(result)
        }
    }
}

fn unique_member_source_segment<'a>(
    source_type_path: &'a [jarde_java::report::ProvedMemberInnerSourceSegment],
    binary_name: &[u8],
) -> Result<Option<&'a jarde_java::report::ProvedMemberInnerSourceSegment>> {
    let mut found: Option<&jarde_java::report::ProvedMemberInnerSourceSegment> = None;
    for segment in source_type_path {
        if segment.binary_name.as_bytes() != binary_name {
            continue;
        }
        if found.is_some_and(|previous| previous != segment) {
            return Err(Error::unsupported(
                "ordinary_generic_source_unproved",
                "selected member source path is ambiguous",
            ));
        }
        found = Some(segment);
    }
    Ok(found)
}

fn segment_is_immediate_source_child(child: &str, parent: &str) -> bool {
    child
        .strip_prefix(parent)
        .and_then(|suffix| suffix.strip_prefix('.'))
        .is_some_and(is_java_identifier)
}

fn member_signature_class_matches(
    ty: &SignatureType,
    expected_binary_name: &str,
    source_type_path: &[jarde_java::report::ProvedMemberInnerSourceSegment],
    require_all_source_segments: bool,
) -> bool {
    let SignatureType::Class(class) = ty else {
        return false;
    };
    let Some(expected) = source_type_path
        .iter()
        .find(|segment| segment.binary_name == expected_binary_name)
    else {
        return false;
    };
    if class
        .segments
        .last()
        .is_none_or(|segment| segment.binary_name.as_slice() != expected_binary_name.as_bytes())
        || (require_all_source_segments && class.segments.len() != 2)
    {
        return false;
    }
    if expected.type_parameter_count > 0
        && !class.segments.last().is_some_and(|segment| {
            segment.arguments.is_empty() || segment.arguments.len() == expected.type_parameter_count
        })
    {
        return false;
    }
    true
}

fn signature_result_matches_member_creation(
    ty: &SignatureType,
    target: &jarde_java::report::ProvedMemberInnerTarget,
) -> bool {
    if matches!(
        ty,
        SignatureType::Class(class)
            if class.segments.len() == 1
                && class.segments[0].binary_name.as_slice() == b"java/lang/Object"
                && class.segments[0].arguments.is_empty()
    ) {
        return true;
    }
    let SignatureType::Class(class) = ty else {
        return false;
    };
    let Some(outer) = target
        .source_type_path
        .iter()
        .find(|segment| segment.binary_name == target.outer)
    else {
        return false;
    };
    let Some(member) = target
        .source_type_path
        .iter()
        .find(|segment| segment.binary_name == target.owner)
    else {
        return false;
    };
    class.segments.len() == 2
        && class.segments[0].binary_name.as_slice() == outer.binary_name.as_bytes()
        && class.segments[1].binary_name.as_slice() == member.binary_name.as_bytes()
        && class.segments[0].arguments.len() == outer.type_parameter_count
        && class.segments[1].arguments.len() == member.type_parameter_count
        && !class.segments[0].arguments.is_empty()
        && !class.segments[1].arguments.is_empty()
}

fn generic_method_declaration(
    record: &ClassSourceMethod,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    method_local_throws_proved: bool,
) -> Result<String> {
    let refused = |why| Error::unsupported("generic_source_shape_unproved", why);
    let candidate = candidate
        .ok_or_else(|| refused("the recovered AST/SSA body is not a direct parameter return"))?;
    let item = &record.item;
    if record.declaration.is_none()
        || item.access_flags & ACC_STATIC == 0
        || item.access_flags
            & !(ACC_PUBLIC
                | ACC_PRIVATE
                | ACC_PROTECTED
                | ACC_STATIC
                | ACC_FINAL
                | ACC_SYNCHRONIZED
                | ACC_STRICT)
            != 0
        || !record.annotations.attributes.is_empty()
        || !record.parameter_annotations.attributes.is_empty()
        || !record.type_annotations.attributes.is_empty()
        || !record.markers.is_empty()
    {
        return Err(refused(
            "method flags, annotations, or existing source refusals cannot be preserved in this generic shape",
        ));
    }
    if parsed.type_parameters.is_empty() {
        return Err(refused("method has no local type parameters"));
    }
    let mut variables = Vec::new();
    for parameter in &parsed.type_parameters {
        let mut bounds = Vec::new();
        for bound in parameter
            .class_bound
            .iter()
            .chain(parameter.interface_bounds.iter())
        {
            let SignatureType::Class(bound) = bound else {
                return Err(refused("type-variable bound is unsupported"));
            };
            let [bound] = bound.segments.as_slice() else {
                return Err(refused("nested bounds are unsupported"));
            };
            if !bound.arguments.is_empty() {
                return Err(refused("parameterized bounds are unsupported"));
            }
            bounds.push(simple_generic_class_name(&bound.binary_name)?);
        }
        let variable = std::str::from_utf8(&parameter.name)
            .map_err(|_| refused("type variable name is not source UTF-8"))?;
        if !is_java_identifier(variable) {
            return Err(refused("type variable has no Java source spelling"));
        }
        if bounds.is_empty() {
            bounds.push("java.lang.Object".to_owned());
        }
        variables.push((
            parameter.name.clone(),
            variable.to_owned(),
            bounds.join(" & "),
        ));
    }
    let Some(SignatureType::TypeVariable(result_variable)) = &parsed.result else {
        return Err(refused("result is outside the supported source shape"));
    };
    let Some((_, result_name, _)) = variables
        .iter()
        .find(|(name, _, _)| name == result_variable)
    else {
        return Err(refused("result variable is outside the method scope"));
    };
    if parsed.parameters.len() != candidate.parameters.len() {
        return Err(refused("parameter count differs from the same-run body"));
    }
    for exception in &parsed.throws {
        let physical_class_throws = matches!(
            exception,
            SignatureType::Class(class)
                if class.segments.len() == 1
                    && class.segments[0].arguments.is_empty()
                    && simple_generic_class_name(&class.segments[0].binary_name).is_ok()
        );
        let proved_local_throws = method_local_throws_proved
            && matches!(exception, SignatureType::TypeVariable(variable)
                if parsed.type_parameters.iter().any(|parameter| parameter.name == *variable));
        if !physical_class_throws && !proved_local_throws {
            return Err(refused(
                "generic throws type cannot be preserved by the physical Exceptions clause",
            ));
        }
    }
    let signature = method_descriptor(&item.descriptor.raw().0, true, false)
        .ok_or_else(|| refused("physical descriptor cannot be spelled"))?;
    if signature.parameters.len() != candidate.parameters.len() {
        return Err(refused("parameter slot mapping differs from descriptor"));
    }
    let mut arguments = Vec::new();
    let mut result_slots = Vec::new();
    let mut bool_slots = Vec::new();
    for (position, ((ty, slot), (candidate_slot, name))) in signature
        .parameters
        .iter()
        .zip(&candidate.parameters)
        .enumerate()
    {
        if slot != candidate_slot || !is_java_identifier(name) {
            return Err(refused("same-run parameter name or slot is unproved"));
        }
        let spelling = match &parsed.parameters[position] {
            SignatureType::TypeVariable(name) => {
                let Some((_, spelling, _)) =
                    variables.iter().find(|(variable, _, _)| variable == name)
                else {
                    return Err(refused("parameter variable is outside the method scope"));
                };
                if name == result_variable {
                    result_slots.push(*slot);
                }
                spelling.clone()
            }
            SignatureType::Base(b'Z') if ty == "boolean" => {
                bool_slots.push(*slot);
                "boolean".to_owned()
            }
            SignatureType::Base(_) => ty.clone(),
            SignatureType::Class(class) => {
                let [segment] = class.segments.as_slice() else {
                    return Err(refused("nested parameter type is unsupported"));
                };
                if !segment.arguments.is_empty()
                    || simple_generic_class_name(&segment.binary_name).is_err()
                {
                    return Err(refused(
                        "parameterized or unspellable parameter type is unsupported",
                    ));
                }
                ty.clone()
            }
            _ => {
                return Err(refused(
                    "parameter uses an unsupported generic source shape",
                ));
            }
        };
        arguments.push(format!("{spelling} {name}"));
    }
    let return_proved = match candidate.value {
        GenericReturnValue::EmptyVoid => false,
        GenericReturnValue::Parameter(slot) => result_slots.contains(&slot),
        GenericReturnValue::Conditional {
            test,
            when_true,
            when_false,
        } => {
            bool_slots.contains(&test)
                && result_slots.contains(&when_true)
                && result_slots.contains(&when_false)
        }
        GenericReturnValue::MemberCreation { .. } => false,
    };
    if !return_proved {
        return Err(refused(
            "return value does not come solely from parameters of the result type variable",
        ));
    }
    let (name, aliased) = written_name(&item.name.raw().0);
    if aliased || !is_java_identifier(&name) {
        return Err(refused("method name has no faithful source spelling"));
    }
    let mut words = Vec::new();
    words.extend(visibility(item.access_flags));
    words.push("static");
    if item.access_flags & ACC_FINAL != 0 {
        words.push("final")
    }
    if item.access_flags & ACC_SYNCHRONIZED != 0 {
        words.push("synchronized")
    }
    if item.access_flags & ACC_STRICT != 0 {
        words.push("strictfp")
    }
    let throws = if method_local_throws_proved {
        parsed
            .throws
            .iter()
            .map(|throws| match throws {
                SignatureType::TypeVariable(variable) => variables
                    .iter()
                    .find(|(name, _, _)| name == variable)
                    .map(|(_, name, _)| name.clone())
                    .ok_or_else(|| refused("throws variable has no emitted method type parameter")),
                _ => Err(refused(
                    "strict source gate admits one local throws variable",
                )),
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        attributes.throws.clone()
    };
    Ok(format!(
        "{}<{}> {} {}({}){}",
        if words.is_empty() {
            String::new()
        } else {
            format!("{} ", words.join(" "))
        },
        variables
            .iter()
            .map(|(_, name, bound)| format!("{name} extends {bound}"))
            .collect::<Vec<_>>()
            .join(", "),
        result_name,
        name,
        arguments.join(", "),
        throws_clause(&throws),
    ))
}

fn simple_generic_class_name(raw: &[u8]) -> Result<String> {
    let text = std::str::from_utf8(raw).map_err(|_| {
        Error::unsupported(
            "generic_source_shape_unproved",
            "class name is not source UTF-8",
        )
    })?;
    if text.contains('$') || text.split('/').any(|part| !is_java_identifier(part)) {
        return Err(Error::unsupported(
            "generic_source_shape_unproved",
            "class name has no unambiguous Java source spelling",
        ));
    }
    Ok(text.replace('/', "."))
}

/// The initializer one field's own `ConstantValue` attribute states (JVMS 4.7.2), as the field's
/// declaration writes it, or `None` when the field's declaration states none.
///
/// The read is [`attribute_facts`] over this field's own `ConstantValue` shell(s) — the reader's one
/// parser for that attribute, and only those shells are handed over, so the `attribute_bytes` charge
/// is exactly this attribute's — and nothing here reads a `<clinit>`, a use site or another member:
/// the initializer is the constant the field's own attribute declares.
///
/// `None` is "nothing to write": the field declares no `ConstantValue` at all, or its value is one
/// this presentation has no literal for (see [`resolve_constant_value`]). The field then keeps the
/// initializer-less declaration its flags and its descriptor state, and not one literal is invented
/// for it.
///
/// An `Err` is the reader's own failure to read the attribute — its bytes are damaged, its charge was
/// refused, the request was cancelled. A field record states no refusal of its own in this report
/// (unlike a member's own run), so the caller's decision is the read's own: the presentation stops
/// where the read that could not be performed happened, rather than publishing a declaration it
/// could not establish.
pub(crate) fn declared_constant_value(
    bytes: &[u8],
    field: &MemberHeader,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<Option<MemberDefault>> {
    if !declares_constant_value(field) {
        return Ok(None);
    }
    let shells = attribute_shells(field, b"ConstantValue");
    let Some(index) = attribute_facts(bytes, &shells, pool, budget)?.constant_value else {
        return Ok(None);
    };
    Ok(resolve_constant_value(
        &field.descriptor.raw().0,
        index.0,
        pool,
    ))
}

/// The initializer one `ConstantValue` attribute states for one field (JVMS 4.7.2), or `None` when
/// this presentation has no literal for it.
///
/// The attribute states one pool index and **no type**: which constant it is comes from the
/// descriptor of the field it initializes, so the same `CONSTANT_Integer` is `3` in an `int` field,
/// `true` in a `boolean` field and `'A'` in a `char` field, and the pair has to agree for any
/// initializer to be written at all. A `float`/`double` constant — which this field path does not
/// spell — a `Z` that is neither `0` nor `1`, an index that names no entry or an
/// entry of another kind (a `CONSTANT_Utf8`, a `CONSTANT_Long` in an `int` field) each state no
/// initializer rather than one that is invented, and a field that declares the attribute keeps the
/// declaration its flags and its descriptor state.
fn resolve_constant_value(
    descriptor: &[u8],
    index: u16,
    pool: &[CpEntryFacts],
) -> Option<MemberDefault> {
    let entry = &cp_entry(pool, index).ok()?.kind;
    match (descriptor, entry) {
        ([b'I'], CpEntryKind::Integer { value }) => Some(MemberDefault::Integer(*value)),
        ([b'J'], CpEntryKind::Long { value }) => Some(MemberDefault::Long(*value)),
        ([b'B'], CpEntryKind::Integer { value }) => Some(MemberDefault::Byte(*value)),
        ([b'S'], CpEntryKind::Integer { value }) => Some(MemberDefault::Short(*value)),
        ([b'C'], CpEntryKind::Integer { value }) => {
            Some(MemberDefault::Char(u16::try_from(*value).ok()?))
        }
        ([b'Z'], CpEntryKind::Integer { value }) => Some(MemberDefault::Boolean(match value {
            0 => false,
            1 => true,
            _ => return None,
        })),
        (b"Ljava/lang/String;", CpEntryKind::String { value, .. }) => {
            Some(MemberDefault::String(value.clone()))
        }
        _ => None,
    }
}

/// One declaration's literal, as the class file's own attribute states it.
///
/// Two attributes state a literal this presentation writes, and both state it as one of these values:
/// a member's `AnnotationDefault` (JVMS 4.7.22), written after the `default` keyword of its
/// declaration, and a field's `ConstantValue` (JVMS 4.7.2), written as that field's initializer. The
/// value is the attribute's own constant with the pool entries its own indexes name resolved, and it
/// exists only for the kinds this presentation has a literal for: for an `AnnotationDefault` the
/// eight constant tags, a string, an enum constant, a class literal, a nested annotation and arrays
/// of those; for a
/// `ConstantValue` the kinds [`resolve_constant_value`] accepts. A value that is not here is a value
/// the declaration writes no default and no initializer for — [`declared_annotation_default`] and
/// [`declared_constant_value`] resolve nothing for a `float`/`double` field constant or an array
/// that holds one, because a partial literal would be a claim the attribute does not make. F/D
/// variants are constructed only by the AnnotationDefault path.
///
/// The two paths share this one vocabulary so they spell one constant one way: `3L` is the same
/// spelling whichever attribute states it, and a string is escaped by the one rule this engine
/// escapes string literals with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MemberDefault {
    /// `B`: a `byte`, written as the decimal integer it is.
    Byte(i32),
    /// `C`: a `char`, written as the character literal of its UTF-16 code unit.
    Char(u16),
    /// `I`: an `int`, written as the decimal integer it is.
    Integer(i32),
    /// `S`: a `short`, written as the decimal integer it is.
    Short(i32),
    /// `Z`: a `boolean`, written as `true` or `false`.
    Boolean(bool),
    /// `J`: a `long`, written as the decimal integer it is with the `L` its type needs.
    Long(i64),
    /// `F`/`D`: an annotation constant's raw IEEE bits, never used for a field initializer.
    Float(u32),
    Double(u64),
    /// `s`: a string, written as the Java string literal its text is.
    String(JvmBytes),
    /// `e`: an enum constant, written as its type's name, a dot, and the constant's own simple name.
    Enum {
        type_name: String,
        constant_name: String,
    },
    /// `c`: a class literal, written as its type's name and `.class`.
    Class {
        type_name: String,
    },
    /// `@`: a nested annotation with its type and explicitly named values in attribute order.
    Annotation {
        type_name: String,
        elements: Vec<(String, MemberDefault)>,
    },
    /// `[`: the elements the array declares, written `{ elem, elem }`.
    Array(Vec<MemberDefault>),
}

impl MemberDefault {
    /// The literal this default is written as, after the `default` keyword.
    fn spelling(&self) -> String {
        match self {
            Self::Byte(value) | Self::Integer(value) | Self::Short(value) => value.to_string(),
            Self::Boolean(value) => if *value { "true" } else { "false" }.to_owned(),
            Self::Long(value) => format!("{value}L"),
            Self::Float(bits) => float_default_spelling(*bits)
                .expect("only a faithfully spellable annotation float is resolved"),
            Self::Double(bits) => double_default_spelling(*bits)
                .expect("only a faithfully spellable annotation double is resolved"),
            Self::Char(value) => char_literal(*value),
            Self::String(bytes) => {
                format!("\"{}\"", escape_string(&String::from_utf8_lossy(&bytes.0)))
            }
            Self::Enum {
                type_name,
                constant_name,
            } => format!("{type_name}.{constant_name}"),
            Self::Class { type_name } => format!("{type_name}.class"),
            Self::Annotation {
                type_name,
                elements,
            } => {
                let elements: Vec<String> = elements
                    .iter()
                    .map(|(name, value)| format!("{name} = {}", value.spelling()))
                    .collect();
                format!("@{type_name}({})", elements.join(", "))
            }
            Self::Array(elements) if elements.is_empty() => "{}".to_owned(),
            Self::Array(elements) => {
                let written: Vec<String> = elements.iter().map(Self::spelling).collect();
                format!("{{{}}}", written.join(", "))
            }
        }
    }
}

/// One `element_value` (JVMS 4.7.22.1) resolved against the pool this class's read published, or
/// `None` when this presentation has no literal for it ([`declared_annotation_default`]).
///
/// The tag decides the reading, not the member's descriptor: `B`/`C`/`I`/`S`/`Z` all name one
/// `CONSTANT_Integer`, and the value `65` is `65` under `I`, `'A'` under `C` and no boolean at all
/// under `Z`. A value the class file states something else for is no literal either: a `Z` that is
/// neither `0` nor `1`, a `C` outside the `0..=0xFFFF` one UTF-16 code unit is, an index that does not
/// hold the constant its tag names, a `c` whose descriptor is the `void` one (a `V` names no type this
/// presentation spells), and an enum constant whose name is not UTF-8. Each states no default, and no
/// default is invented for it.
fn resolve_default(value: &ElementValueFacts, pool: &[CpEntryFacts]) -> Option<MemberDefault> {
    Some(match value {
        ElementValueFacts::Constant { tag, index } => {
            match (tag, &cp_entry(pool, index.0).ok()?.kind) {
                (ElementConstantTag::Byte, CpEntryKind::Integer { value }) => {
                    MemberDefault::Byte(*value)
                }
                (ElementConstantTag::Char, CpEntryKind::Integer { value }) => {
                    MemberDefault::Char(u16::try_from(*value).ok()?)
                }
                (ElementConstantTag::Integer, CpEntryKind::Integer { value }) => {
                    MemberDefault::Integer(*value)
                }
                (ElementConstantTag::Short, CpEntryKind::Integer { value }) => {
                    MemberDefault::Short(*value)
                }
                (ElementConstantTag::Boolean, CpEntryKind::Integer { value }) => {
                    MemberDefault::Boolean(match value {
                        0 => false,
                        1 => true,
                        _ => return None,
                    })
                }
                (ElementConstantTag::Long, CpEntryKind::Long { value }) => {
                    MemberDefault::Long(*value)
                }
                (ElementConstantTag::Float, CpEntryKind::Float { bits }) => {
                    float_default_spelling(*bits).map(|_| MemberDefault::Float(*bits))?
                }
                (ElementConstantTag::Double, CpEntryKind::Double { bits }) => {
                    double_default_spelling(*bits).map(|_| MemberDefault::Double(*bits))?
                }
                _ => return None,
            }
        }
        ElementValueFacts::Utf8(bytes) => MemberDefault::String(bytes.clone()),
        ElementValueFacts::Enum {
            type_descriptor,
            constant_name,
        } => MemberDefault::Enum {
            type_name: descriptor_type(&type_descriptor.0, DescriptorKind::Field)?,
            constant_name: String::from_utf8(constant_name.0.clone()).ok()?,
        },
        ElementValueFacts::Class(descriptor) => MemberDefault::Class {
            type_name: descriptor_type(&descriptor.0, DescriptorKind::Return)?,
        },
        ElementValueFacts::Annotation {
            type_descriptor,
            elements,
        } => {
            let type_name = annotation_type_name(&type_descriptor.0)?;
            let elements = elements
                .iter()
                .map(|element| {
                    let name = String::from_utf8(element.name.0.clone()).ok()?;
                    is_java_identifier(&name).then_some(())?;
                    Some((name, resolve_default(&element.value, pool)?))
                })
                .collect::<Option<Vec<_>>>()?;
            MemberDefault::Annotation {
                type_name,
                elements,
            }
        }
        ElementValueFacts::Array(elements) => MemberDefault::Array(
            elements
                .iter()
                .map(|element| resolve_default(element, pool))
                .collect::<Option<Vec<MemberDefault>>>()?,
        ),
    })
}

/// Spell finite IEEE values as exact Java hexadecimal floating-point literals without converting
/// the raw facts through host floating-point arithmetic. Unsupported NaN patterns have no spelling.
fn float_default_spelling(bits: u32) -> Option<String> {
    let sign = if bits >> 31 == 0 { "" } else { "-" };
    let exponent = (bits >> 23) & 0xff;
    let fraction = bits & 0x7f_ffff;
    match (exponent, fraction) {
        (0xff, 0) => Some(format!(
            "Float.{}INFINITY",
            if sign.is_empty() {
                "POSITIVE_"
            } else {
                "NEGATIVE_"
            }
        )),
        (0xff, 0x40_0000) if sign.is_empty() => Some("Float.NaN".to_owned()),
        (0xff, _) => None,
        (0, 0) => Some(format!("{sign}0.0f")),
        (0, fraction) => Some(format!("{sign}0x0.{:06x}p-126f", fraction << 1)),
        _ => Some(format!(
            "{sign}0x1.{:06x}p{}f",
            fraction << 1,
            i32::try_from(exponent).unwrap() - 127
        )),
    }
}

fn double_default_spelling(bits: u64) -> Option<String> {
    let sign = if bits >> 63 == 0 { "" } else { "-" };
    let exponent = (bits >> 52) & 0x7ff;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    match (exponent, fraction) {
        (0x7ff, 0) => Some(format!(
            "Double.{}INFINITY",
            if sign.is_empty() {
                "POSITIVE_"
            } else {
                "NEGATIVE_"
            }
        )),
        (0x7ff, 0x0008_0000_0000_0000) if sign.is_empty() => Some("Double.NaN".to_owned()),
        (0x7ff, _) => None,
        (0, 0) => Some(format!("{sign}0.0d")),
        (0, fraction) => Some(format!("{sign}0x0.{fraction:013x}p-1022d")),
        _ => Some(format!(
            "{sign}0x1.{fraction:013x}p{}d",
            i32::try_from(exponent).unwrap() - 1023
        )),
    }
}

/// The class type a nested annotation descriptor names, when each segment is a source identifier.
///
/// Annotation descriptors name a class, not an array or primitive. A slash becomes a package dot;
/// every resulting segment must remain writable without guessing an alias, because aliases would
/// name a different annotation type at the use site.
fn annotation_type_name(descriptor: &[u8]) -> Option<String> {
    if !descriptor.starts_with(b"L") || !descriptor.ends_with(b";") {
        return None;
    }
    let internal_name = &descriptor[1..descriptor.len() - 1];
    if internal_name.is_empty()
        || internal_name.split(|byte| *byte == b'/').any(|segment| {
            segment.is_empty()
                || segment
                    .iter()
                    .any(|byte| matches!(*byte, b'.' | b'[' | b';'))
        })
    {
        return None;
    }
    let type_name = descriptor_type(descriptor, DescriptorKind::Field)?;
    type_name
        .split('.')
        .all(is_java_identifier)
        .then_some(type_name)
}

/// The Java type one field or return descriptor names, as the recovery layer spells types.
///
/// It is the same spelling the parameters and the field declarations are written with
/// ([`source_type`]), so a default's enum type or class literal cannot name a type the declaration
/// beside it would write differently. A descriptor this presentation cannot spell — a `V` class
/// literal names `void`, which is no type here — states no default.
fn descriptor_type(descriptor: &[u8], kind: DescriptorKind) -> Option<String> {
    let facts = descriptor_facts(descriptor, kind).ok()?;
    source_type(facts.single()?)
}

/// One `char` value's own text as a character literal, escaped by the rule every literal in this
/// repository is escaped with: the delimiter, the backslash, the controls and the line terminators
/// are escapes, every other unit inside the BMP is written as itself, and a unit a literal cannot
/// carry is spelled `\uXXXX`.
///
/// This is the character literal the recovery layer's own `switch` keys are written with (`case
/// 'a':`), for the one place a declaration states a character: its escape table is the emitter's,
/// and it is stated here because that table is not published.
fn char_literal(value: u16) -> String {
    let mut literal = String::from("'");
    match value {
        0x27 => literal.push_str("\\'"),
        0x5c => literal.push_str("\\\\"),
        0x08 => literal.push_str("\\b"),
        0x09 => literal.push_str("\\t"),
        0x0a => literal.push_str("\\n"),
        0x0c => literal.push_str("\\f"),
        0x0d => literal.push_str("\\r"),
        0x20..=0x7e => literal.push(char::from_u32(u32::from(value)).unwrap_or('?')),
        other => literal.push_str(&format!("\\u{other:04x}")),
    }
    literal.push('\'');
    literal
}

/// The `throws` clause one member's own `Exceptions` attribute is written as, after its parameter
/// list, or nothing at all when the member declares no exception.
///
/// The clause is one line of the declaration and it states exactly the types the attribute states,
/// in the attribute's own order ([`declared_exceptions`]); an empty list is no clause, because
/// `throws` with nothing after it is not a clause Java writes.
fn throws_clause(throws: &[String]) -> String {
    if throws.is_empty() {
        return String::new();
    }
    format!(" throws {}", throws.join(", "))
}

/// Whether one member is written with `default`: a member of a class whose own flags include
/// `ACC_INTERFACE`, whose own flags are neither `ACC_STATIC` nor `ACC_ABSTRACT`, and whose
/// declaration carries a `Code` attribute.
///
/// Those are exactly the members the format calls default methods: an interface's body-bearing
/// member that is not static. A class never gains one — `default` is no class member modifier — and
/// an interface's `static` member stays `static` with no `default` beside it. The `Code` attribute is
/// read off the member's own record ([`MethodItem::body`]), which is the same fact the member's run
/// is decided by, so a member spelled here and a member run there cannot disagree about whether the
/// class file declares a body.
fn is_default_member(class_flags: u16, item: &MethodItem) -> bool {
    class_flags & ACC_INTERFACE != 0
        && item.access_flags & ACC_STATIC == 0
        && item.access_flags & ACC_ABSTRACT == 0
        && matches!(item.body, crate::MemberBodyEvidence::CodeAttribute { .. })
}

/// Spells one method's declaration, and the marker its own spelling needs.
///
/// `class` is the class's own spelling ([`ClassSourceDeclaration::name`]), which a constructor's
/// declaration is written with: the class file names it `<init>`, and Java spells the same
/// declaration with the class's name, so nothing here claims a name the bytes do not state — the raw
/// name stays in [`ClassSourceMethod::item`]. `<clinit>` is spelled as Java's static initializer
/// block (`static`), which is the declaration the class file means.
///
/// `class_flags` are the flags of the class that **declares** this member, which one rule of the
/// spelling needs and the member's own flags cannot state: a member of an interface
/// (`ACC_INTERFACE`, 0x0200) that is neither `static` (0x0008) nor `abstract` (0x0400) and whose
/// declaration carries a `Code` attribute is a `default` member, and `default` is written between
/// its access modifiers and its return type. A class's own member never gains one, and an
/// interface's `static` member stays `static` whichever class flags it sits in.
///
/// The member's own `ACC_VARARGS` (0x0080) is the one fact of the parameter list beside the
/// descriptor ([`method_descriptor`]): when it is set and the descriptor's last parameter is an
/// array, that parameter is written `T...` in place of `T[]`, and every position before it — and
/// the last one of every member without the flag — is written exactly as the descriptor states it.
/// The declaration is all the flag changes: no argument is expanded or boxed here, and the body
/// still reads the slot as the array it is.
///
/// `throws` is the member's own `Exceptions` attribute (JVMS 4.7.4), as the read that resolved it
/// published it: a member that declares one is written `throws A, B` after its parameter list, in
/// the attribute's own order, and a member that declares none — or one whose attribute declares no
/// exception — writes no clause. No exception is inferred from an `athrow`.
///
/// `default` is the member's own `AnnotationDefault` attribute (JVMS 4.7.22), as the read that
/// resolved it published it: a member that declares one is written `default <literal>` after its
/// parameter list, and a member that declares none — and one whose value this presentation has no
/// literal for — writes no `default` at all. Which class declares such a member is not read here:
/// the attribute's presence is the fact, so an ordinary interface or class member that declares one
/// is written the same way an annotation type's member is.
pub(crate) fn spell_method(
    item: &MethodItem,
    facts: Option<&RecoveryFacts>,
    class: &str,
    class_flags: u16,
    attributes: Option<&MemberAttributes>,
    annotations: &MemberAnnotationFacts,
    pool: &[CpEntryFacts],
) -> Spelled {
    let signature = method_descriptor(
        &item.descriptor.raw().0,
        is_static(item.access_flags),
        item.access_flags & ACC_VARARGS != 0,
    );
    let parameter_count = signature
        .as_ref()
        .map(|signature| signature.parameters.len());
    let mut member_annotations =
        spell_member_annotation_uses(annotations.declaration.clone(), pool);
    let mut parameter_annotations =
        spell_parameter_annotation_uses(annotations.parameters.clone(), pool, parameter_count);
    let type_annotations = spell_type_annotation_uses(
        annotations.type_uses.clone(),
        TypeUseOwner::Method,
        &item.descriptor.raw().0,
        pool,
        &member_annotations,
        Some(&annotations.parameters),
        parameter_count,
    );
    if item.name.raw().0 == b"<clinit>" {
        refuse_uses(
            &mut member_annotations,
            "class initializer blocks have no Java method declaration to annotate",
        );
    }
    let mut spelled = spell_method_declaration(
        item,
        facts,
        class,
        class_flags,
        attributes,
        &parameter_annotations.uses_by_position,
        &type_annotations,
    );
    if spelled.declaration.is_none() {
        refuse_uses(
            &mut member_annotations,
            "method descriptor cannot be spelled as a Java declaration",
        );
        refuse_parameter_uses(
            &mut parameter_annotations,
            "method descriptor cannot establish parameter positions",
        );
    }
    spelled.annotations = member_annotations;
    spelled.parameter_annotations = parameter_annotations;
    spelled.type_annotations = type_annotations;
    spelled
}

fn spell_method_declaration(
    item: &MethodItem,
    facts: Option<&RecoveryFacts>,
    class: &str,
    class_flags: u16,
    attributes: Option<&MemberAttributes>,
    parameter_annotations: &[Vec<String>],
    type_annotations: &TypeAnnotationUses,
) -> Spelled {
    let (default, throws) = attributes.map_or((None, &[][..]), |attributes| {
        (attributes.default.as_ref(), attributes.throws.as_slice())
    });
    let raw = &item.name.raw().0;
    match raw.as_slice() {
        b"<clinit>" => {
            return Spelled {
                declaration: Some("static".to_owned()),
                marker: None,
                annotations: MemberAnnotationUses::default(),
                parameter_annotations: ParameterAnnotationUses::default(),
                type_annotations: TypeAnnotationUses::default(),
            };
        }
        b"<init>" => {
            let Some(signature) = method_descriptor(
                &item.descriptor.raw().0,
                is_static(item.access_flags),
                item.access_flags & ACC_VARARGS != 0,
            ) else {
                return Spelled {
                    declaration: None,
                    marker: Some(not_a_descriptor(item, "method")),
                    annotations: MemberAnnotationUses::default(),
                    parameter_annotations: ParameterAnnotationUses::default(),
                    type_annotations: TypeAnnotationUses::default(),
                };
            };
            let mut declaration = String::new();
            if let Some(word) = visibility(item.access_flags) {
                declaration.push_str(word);
                declaration.push(' ');
            }
            declaration.push_str(class);
            declaration.push_str(&arguments(
                facts,
                &signature,
                parameter_annotations,
                &type_annotations.parameter_uses,
            ));
            declaration.push_str(&throws_clause(throws));
            return Spelled {
                declaration: Some(declaration),
                marker: None,
                annotations: MemberAnnotationUses::default(),
                parameter_annotations: ParameterAnnotationUses::default(),
                type_annotations: TypeAnnotationUses::default(),
            };
        }
        _ => {}
    }
    let Some(signature) = method_descriptor(
        &item.descriptor.raw().0,
        is_static(item.access_flags),
        item.access_flags & ACC_VARARGS != 0,
    ) else {
        return Spelled {
            declaration: None,
            marker: Some(not_a_descriptor(item, "method")),
            annotations: MemberAnnotationUses::default(),
            parameter_annotations: ParameterAnnotationUses::default(),
            type_annotations: TypeAnnotationUses::default(),
        };
    };
    let (name, aliased) = written_name(raw);
    let mut words: Vec<&str> = Vec::new();
    words.extend(visibility(item.access_flags));
    if is_static(item.access_flags) {
        words.push("static");
    }
    if item.access_flags & ACC_FINAL != 0 {
        words.push("final");
    }
    if item.access_flags & ACC_ABSTRACT != 0 {
        words.push("abstract");
    }
    if item.access_flags & ACC_NATIVE != 0 {
        words.push("native");
    }
    if item.access_flags & ACC_SYNCHRONIZED != 0 {
        words.push("synchronized");
    }
    if item.access_flags & ACC_STRICT != 0 {
        words.push("strictfp");
    }
    if is_default_member(class_flags, item) {
        // The last modifier: `default` stands between the access modifiers and the return type,
        // which is where Java source writes it (`public default int zero()`).
        words.push("default");
    }
    let returns = signature.returns.as_deref().unwrap_or("void");
    let returns = if type_annotations.return_uses.is_empty() {
        returns.to_owned()
    } else {
        decorate_qualified_type(returns, &type_annotations.return_uses)
            .unwrap_or_else(|| returns.to_owned())
    };
    let mut declaration = String::new();
    for word in words {
        declaration.push_str(word);
        declaration.push(' ');
    }
    declaration.push_str(&returns);
    declaration.push(' ');
    declaration.push_str(&name);
    declaration.push_str(&arguments(
        facts,
        &signature,
        parameter_annotations,
        &type_annotations.parameter_uses,
    ));
    declaration.push_str(&throws_clause(throws));
    if let Some(default) = default {
        // The default stands between the parameter list and the `;` (or the body), which is the one
        // place Java source states one.
        declaration.push_str(" default ");
        declaration.push_str(&default.spelling());
    }
    Spelled {
        declaration: Some(declaration),
        marker: aliased.then(|| aliased_name(item)),
        annotations: MemberAnnotationUses::default(),
        parameter_annotations: ParameterAnnotationUses::default(),
        type_annotations: TypeAnnotationUses::default(),
    }
}

/// Spells one field's declaration, and the marker its own spelling needs.
///
/// `constant` is the value the field's own `ConstantValue` attribute (JVMS 4.7.2) states, as the
/// read that resolved it published it: a field with one is written `<declaration> = <literal>`, and
/// a field that declares no such attribute — or one whose value this presentation has no literal
/// for — is written without an initializer at all. Nothing is inferred from a `<clinit>` or a use
/// site, so the one `=` this presentation writes is the one the field's own attribute states.
fn spell_field_declaration(item: &FieldItem, constant: Option<&MemberDefault>) -> Spelled {
    let Some(field_type) = field_type(&item.descriptor.raw().0) else {
        return Spelled {
            declaration: None,
            marker: Some(not_a_descriptor_field(item)),
            annotations: MemberAnnotationUses::default(),
            parameter_annotations: ParameterAnnotationUses::default(),
            type_annotations: TypeAnnotationUses::default(),
        };
    };
    let (name, aliased) = written_name(&item.name.raw().0);
    let declaration = field_declaration_with_type(item, &field_type, constant);
    Spelled {
        declaration: Some(declaration),
        marker: aliased.then(|| {
            format!(
                "// jarde: the field's raw name `{}` is not a Java identifier; it is written as `{name}`",
                comment_name(&item.name)
            )
        }),
        annotations: MemberAnnotationUses::default(),
        parameter_annotations: ParameterAnnotationUses::default(),
        type_annotations: TypeAnnotationUses::default(),
    }
}

fn field_declaration_with_type(
    item: &FieldItem,
    ty: &str,
    constant: Option<&MemberDefault>,
) -> String {
    let (name, _) = written_name(&item.name.raw().0);
    let mut words: Vec<&str> = Vec::new();
    words.extend(visibility(item.access_flags));
    if is_static(item.access_flags) {
        words.push("static");
    }
    if item.access_flags & ACC_FINAL != 0 {
        words.push("final");
    }
    if item.access_flags & ACC_VOLATILE != 0 {
        words.push("volatile");
    }
    if item.access_flags & ACC_TRANSIENT != 0 {
        words.push("transient");
    }
    let mut declaration = String::new();
    for word in words {
        declaration.push_str(word);
        declaration.push(' ');
    }
    declaration.push_str(ty);
    declaration.push(' ');
    declaration.push_str(&name);
    if let Some(constant) = constant {
        declaration.push_str(" = ");
        declaration.push_str(&constant.spelling());
    }
    declaration
}

/// Atomically project one field's generic type after reader erasure proof and the first source
/// compatibility gate. Every refusal keeps the physical descriptor declaration intact.
pub(crate) fn project_field_signature(
    record: &mut ClassSourceField,
    member: &MemberHeader,
    bytes: &[u8],
    pool: &[CpEntryFacts],
    class_internal: &[u8],
    class_flags: u16,
    class_scope: &[TypeParameterErasure],
    constant: Option<&MemberDefault>,
    budget: &mut Budget,
) -> Result<()> {
    let shells = attribute_shells(member, b"Signature");
    if shells.is_empty() {
        return Ok(());
    }
    let result = (|| -> Result<(String, Vec<u8>)> {
        let facts = attribute_facts(bytes, &shells, pool, budget)?;
        let raw = facts
            .signature
            .ok_or_else(|| {
                Error::invalid_input("jvm_signature_missing", "Signature did not resolve")
            })?
            .0;
        let parsed = parse_field_signature(&raw, budget)?;
        prove_field_signature_erasure_with_class_scope(
            &parsed,
            &member.descriptor.raw().0,
            class_scope,
            budget,
        )?;

        if record.declaration.is_none()
            || !record.markers.is_empty()
            || !std::str::from_utf8(&record.item.name.raw().0).is_ok_and(is_java_identifier)
            || record.item.access_flags
                & !(ACC_PUBLIC
                    | ACC_PRIVATE
                    | ACC_PROTECTED
                    | ACC_STATIC
                    | ACC_FINAL
                    | ACC_VOLATILE
                    | ACC_TRANSIENT)
                != 0
            || class_internal.contains(&b'$')
            || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
            || !record.type_annotations.attributes.is_empty()
            || !record.type_annotations.refusals.is_empty()
        {
            return Err(Error::unsupported(
                "field_generic_source_unproved",
                "field name, flags, descriptor, or type-use annotation path cannot be preserved",
            ));
        }

        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(pool.len()).unwrap_or(u64::MAX),
        )?;
        if pool.iter().any(|entry| {
            matches!(&entry.kind,
                CpEntryKind::FieldRef { owner, name, descriptor, .. }
                    if owner.0.as_slice() == class_internal
                        && name.0.as_slice() == member.name.raw().0.as_slice()
                        && descriptor.0.as_slice() == member.descriptor.raw().0.as_slice()
            )
        }) {
            return Err(Error::unsupported(
                "field_generic_body_unproved",
                "a same-class Fieldref names this field and descriptor",
            ));
        }

        let ty = spell_ordinary_signature_type(&parsed.ty, class_scope, budget, 0)?;
        Ok((
            field_declaration_with_type(&record.item, &ty, constant),
            raw,
        ))
    })();
    match result {
        Ok((declaration, signature)) => {
            let marker = format!(
                "// jarde: field Signature `{}` projected after descriptor erasure and no same-class Fieldref",
                comment_text(&String::from_utf8_lossy(&signature)),
            );
            let mut markers = record.markers.clone();
            markers.push(marker);
            let text = declaration_member(&declaration, &markers);
            budget.charge(
                CountedBudgetDimension::OutputBytes,
                u64::try_from(text.len()).unwrap_or(u64::MAX),
            )?;
            record.declaration = Some(declaration);
            record.markers = markers;
            Ok(())
        }
        Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => Err(error),
        Err(error) => {
            let marker = format!(
                "// jarde: field Signature projection refused for `{}{}`: {}",
                comment_name(&record.item.name),
                comment_name(&record.item.descriptor),
                comment_text(&error.to_string()),
            );
            let mut markers = record.markers.clone();
            markers.push(marker);
            let text =
                declaration_member(record.declaration.as_deref().unwrap_or_default(), &markers);
            budget.charge(
                CountedBudgetDimension::OutputBytes,
                u64::try_from(text.len()).unwrap_or(u64::MAX),
            )?;
            record.markers = markers;
            Ok(())
        }
    }
}

/// One parameter's type as the list writes it: the type the descriptor states, or — for the last
/// parameter of a member whose own `ACC_VARARGS` applies — that type's last `[]` written as the
/// three dots of a variable-arity parameter (`byte[][]` → `byte[]...`).
///
/// The cut is exact and not a second reading of the bytes: the spelling it trims is the one
/// [`source_type`] wrote from the same component, and [`method_descriptor`] states the flag only
/// for a component that is an array, so a type that does not end in `[]` is written as it stands
/// rather than trimmed to a type the descriptor never named.
fn varargs_type(ty: &str, varargs: bool) -> String {
    match ty.strip_suffix("[]") {
        Some(element) if varargs => format!("{element}..."),
        _ => ty.to_owned(),
    }
}

/// The argument list one declaration is written with: every parameter of the descriptor, in
/// declaration order, each with its type and its name.
///
/// A parameter's slot is the JVM layer's own derivation ([`Signature::parameters`]) — `this` holds
/// slot 0 of a member that is not `static`, a `long`/`double` fills two slots and an **array of
/// either fills one** — and the name is the one the body's own statements use for that slot
/// ([`parameter_names`]), so the declaration and the body cannot name two different slots for one
/// parameter.
///
/// The one parameter whose type is not written as the descriptor states it is the last one of a
/// member whose own `ACC_VARARGS` applies ([`Signature::varargs`]): it is written `T...`, the array
/// it is with the last `[]` spelled as the dots (see [`varargs_type`]). No other position is
/// touched, and no call site is read: the declaration says the member takes a variable number of
/// arguments, and the body still reads the slot as the array the descriptor states.
fn arguments(
    facts: Option<&RecoveryFacts>,
    signature: &Signature,
    parameter_annotations: &[Vec<String>],
    type_annotations: &[Vec<String>],
) -> String {
    let names = parameter_names(facts, signature.slots);
    let written: Vec<String> = signature
        .parameters
        .iter()
        .enumerate()
        .map(|(position, (ty, slot))| {
            let name = names
                .get(usize::from(*slot))
                .cloned()
                .unwrap_or_else(|| format!("arg{slot}"));
            let varargs = signature.varargs && position + 1 == signature.parameters.len();
            let ty = type_annotations
                .get(position)
                .filter(|items| !items.is_empty())
                .and_then(|items| decorate_qualified_type(ty, items))
                .unwrap_or_else(|| ty.clone());
            let ty = varargs_type(&ty, varargs);
            let annotations = parameter_annotations
                .get(position)
                .into_iter()
                .flatten()
                .map(|annotation| format!("{annotation} "))
                .collect::<String>();
            format!("{annotations}{ty} {name}")
        })
        .collect();
    format!("({})", written.join(", "))
}

/// The marker of a member whose raw descriptor is not the descriptor of its kind (JVMS 4.3.2/4.3.3).
fn not_a_descriptor(item: &MethodItem, kind: &str) -> String {
    format!(
        "// jarde: not spelled: the descriptor `{}` of the member `{}` is not a {kind} descriptor, so this presentation writes no declaration for it and performs no run for it",
        comment_name(&item.descriptor),
        comment_name(&item.name)
    )
}

fn not_a_descriptor_field(item: &FieldItem) -> String {
    format!(
        "// jarde: not spelled: the descriptor `{}` of the field `{}` is not a field descriptor, so this presentation writes no declaration for it",
        comment_name(&item.descriptor),
        comment_name(&item.name)
    )
}

/// The marker of a member whose raw name Java cannot spell.
fn aliased_name(item: &MethodItem) -> String {
    format!(
        "// jarde: the member's raw name `{}` is not a Java identifier; it is written as `{}`",
        comment_name(&item.name),
        written_name(&item.name.raw().0).0
    )
}

// ---------------------------------------------------------------------------------------------
// The text: one class, its members, and the blocks the recovery layer wrote
// ---------------------------------------------------------------------------------------------

/// One produced artifact, split at its own block: the envelope comment lines the recovery layer
/// wrote, and the statements it wrote.
struct Artifact<'a> {
    /// The envelope: the `//` lines the artifact opens with (`@method`, `@declaration`, the
    /// statement that the presentation is not claimed to compile).
    envelope: &'a str,
    /// The statements, with neither the artifact's opening brace nor its closing one.
    statements: &'a str,
}

/// Splits one produced artifact into its envelope and its statements, by the shape the emitter of
/// [`jarde_java::recover`] writes: comment lines, a line holding `{` alone, the statements at one
/// level of indentation, and a line holding `}` alone as the last line.
///
/// The split is a reading of that shape and it is deliberately strict: the statements of a body are
/// indented, so `{` alone can only be the block's own opening brace, and `}` alone as the last line
/// can only be its closing one. Anything else is not placed at all — the caller quotes it line by
/// line instead of guessing where a statement starts — because placing a statement outside the
/// member it belongs to would be worse than not placing it.
fn artifact(text: &str) -> Option<Artifact<'_>> {
    let mut open: Option<usize> = None;
    let mut close: Option<usize> = None;
    let mut at = 0usize;
    for line in text.split_inclusive('\n') {
        let bare = line.trim_end_matches('\n');
        if bare == "{" && open.is_none() {
            open = Some(at);
        }
        if bare == "}" {
            close = Some(at);
        }
        at += line.len();
    }
    let open = open?;
    let close = close?;
    if close < open + 2 {
        return None;
    }
    let trailing = &text[close..];
    if trailing != "}\n" && trailing != "}" {
        return None;
    }
    Some(Artifact {
        envelope: &text[..open],
        statements: &text[open + 2..close],
    })
}

/// One artifact this presentation could not place, quoted line by line as comments: nothing a run
/// produced is dropped, and none of it is read as a statement.
fn quote(text: &str) -> String {
    let mut quoted = String::new();
    for line in text.lines() {
        quoted.push_str("// ");
        quoted.push_str(line);
        quoted.push('\n');
    }
    quoted
}

/// One block of text, every non-blank line prefixed with `depth` levels of indentation.
fn indent(text: &str, depth: usize) -> String {
    let pad = "    ".repeat(depth);
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        if line.trim_end_matches('\n').is_empty() {
            out.push_str(line);
            continue;
        }
        out.push_str(&pad);
        out.push_str(line);
    }
    out
}

/// What one member's block holds.
enum Placed<'a> {
    /// The artifact's own envelope and statements, as the recovery layer wrote them.
    Block(Artifact<'a>),
    /// A produced artifact whose shape this presentation does not recognize: quoted line by line.
    Quoted(&'a str),
    /// No artifact at all: the marker is the whole body.
    Without,
}

/// One member written as a declaration and a block, at one level of indentation.
fn block_member(declaration: &str, placed: Placed<'_>, markers: &[String]) -> String {
    let mut out = format!("    {declaration} {{\n");
    for marker in markers {
        out.push_str(&indent(&format!("{marker}\n"), 2));
    }
    match placed {
        Placed::Without => {}
        Placed::Block(artifact) => {
            // The envelope is written one level in from the class body (it stood outside the
            // artifact's braces, and this presentation writes it inside them), and the statements
            // keep the one level of indentation their own block gave them: both land at the same
            // depth, one level below the declaration above them.
            out.push_str(&indent(artifact.envelope, 2));
            out.push_str(&indent(artifact.statements, 1));
        }
        Placed::Quoted(text) => out.push_str(&indent(&quote(text), 2)),
    }
    out.push_str("    }\n");
    out
}

/// One member written as a declaration and a `;`, which is how Java spells a member that declares no
/// body at all.
fn declaration_member(declaration: &str, markers: &[String]) -> String {
    let mut out = String::new();
    for marker in markers {
        out.push_str(&indent(&format!("{marker}\n"), 1));
    }
    out.push_str(&format!("    {declaration};\n"));
    out
}

/// One member written as its markers alone: nothing else about it can be spelled.
fn comment_member(markers: &[String]) -> String {
    let mut out = String::new();
    for marker in markers {
        out.push_str(&indent(&format!("{marker}\n"), 1));
    }
    out
}

/// The stable code of one execution plane's stop, or `None` when the plane is `Complete`.
fn stop_code(execution: &ExecutionReport) -> Option<String> {
    match execution {
        ExecutionReport::Complete { .. } => None,
        ExecutionReport::Partial { reason, .. } | ExecutionReport::Failed { reason, .. } => {
            Some(match reason {
                TerminationReason::BudgetExceeded { dimension } => {
                    format!("budget_exceeded_{}", budget_dimension_code(*dimension))
                }
                TerminationReason::Error { code } | TerminationReason::Unsupported { code } => {
                    code.clone()
                }
            })
        }
        ExecutionReport::Cancelled { .. } => Some("cancelled".to_owned()),
    }
}

/// The markers one recovered member carries, in the order the text writes them.
///
/// The order is the member's own spelling first, then its body's: a name Java cannot spell is a fact
/// about the declaration, and a body that was not recovered is a fact about the block below it.
fn recovered_markers(
    item: &MethodItem,
    spelled: &Spelled,
    report: &RecoveryReport,
    analysis: &ClassSourceRunFacts,
) -> Vec<String> {
    let mut markers = markers_of(spelled);
    let member = label(item);
    // A run **stopped** before it had an artifact: the artifact is absent, and the marker says which
    // plane of the run ended it. The recovery's *outcome* is what decides this, not the execution
    // plane alone: a run whose evidence phase stopped after the artifact was committed is a produced
    // member with a non-`Complete` execution, and it is marked as such below
    // (`add-demand-driven-core-results`, D1).
    if report.stop().is_some() {
        let stop = stop_code(&report.execution).unwrap_or_else(|| "unstated".to_owned());
        // Two planes can state a stop of the same run, and they can name different causes: the
        // analysis stops first (a refused charge, a damaged body) and the recovery layer then stops
        // on the table that run never published. Both are written, because "the artifact was not
        // produced" and "what stopped the run that would have produced it" are different answers.
        let blamed = match stop_code(&analysis.execution) {
            Some(analysis) => {
                format!("; the analysis of that member did not complete ({analysis})")
            }
            None => String::new(),
        };
        markers.push(format!(
            "// jarde: not recovered: the recovery run for `{member}` stopped ({stop}){blamed}"
        ));
        return markers;
    }
    match report.content {
        RecoveryContent::ContainsStatements => {}
        RecoveryContent::ExplanationOnly => {
            markers.push(format!(
                "// jarde: not recovered: the recovery run for `{member}` produced no statement (explanation only); the artifact's own comment lines are below"
            ));
            return markers;
        }
        RecoveryContent::NotProduced => {
            markers.push(format!(
                "// jarde: not recovered: the recovery run for `{member}` states that it produced no artifact content; the lines below are what it wrote"
            ));
            return markers;
        }
    }
    if let Some(code) = stop_code(&analysis.execution) {
        markers.push(format!(
            "// jarde: diagnosed stop: the run for `{member}` did not complete ({code}); the text below is what that run produced"
        ));
    }
    if let Some(stop) = stop_code(&report.execution) {
        // The artifact is here and the run is not `Complete`: what stopped is the optional evidence
        // this request selected, and saying so is different from saying the member was not recovered.
        markers.push(format!(
            "// jarde: evidence stopped: the run for `{member}` produced the text below; the evidence it selected stopped ({stop})"
        ));
    }
    markers
}

// ---------------------------------------------------------------------------------------------
// The members, as this presentation builds them
// ---------------------------------------------------------------------------------------------

impl ClassSourceDeclaration {
    /// The declaration of one class-level item, as this presentation spells it: the simple name
    /// `this_class` states, and the declaration line the kind and the flags state.
    pub(crate) fn of(item: ClassDeclarationItem) -> Self {
        let name = simple_name(&item.declaration.this_class.raw().0);
        let declaration = class_declaration(&name, &item.declaration);
        Self {
            item,
            name,
            declaration,
            annotation_attributes: Vec::new(),
            annotation_uses: Vec::new(),
            annotation_refusals: Vec::new(),
            generic_signature: None,
            generic_refusal: None,
        }
    }

    /// Project the class's own Signature only after its parent identities and variable erasures
    /// agree with the physical header. The returned scope is handed to member signatures only
    /// when the complete class header was published.
    pub(crate) fn project_generic_signature(
        &mut self,
        bytes: &[u8],
        shells: &[AttributeShell],
        pool: &[CpEntryFacts],
        budget: &mut Budget,
    ) -> Result<Option<ClassSignatureErasureProof>> {
        let signatures: Vec<_> = shells
            .iter()
            .filter(|shell| shell.name.raw().0 == b"Signature")
            .cloned()
            .collect();
        if signatures.is_empty() {
            return Ok(None);
        }
        let result = (|| -> Result<Option<(String, ClassSignatureErasureProof)>> {
            let raw = attribute_facts(bytes, &signatures, pool, budget)?
                .signature
                .ok_or_else(|| {
                    Error::invalid_input(
                        "jvm_signature_missing",
                        "class Signature attribute did not resolve",
                    )
                })?;
            let parsed = parse_class_signature(&raw.0, budget)?;
            if parsed.type_parameters.is_empty() {
                return Ok(None);
            }
            self.generic_signature = Some(raw.clone());
            let facts = &self.item.declaration;
            if facts.access_flags & (ACC_ANNOTATION | ACC_ENUM) != 0
                || !is_java_identifier(&self.name)
                || self.name.contains('$')
                || shells.iter().any(|shell| {
                    matches!(
                        shell.name.raw().0.as_slice(),
                        b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
                    )
                })
            {
                return Err(Error::unsupported(
                    "class_generic_source_unproved",
                    "class kind, nesting, name, or type-use annotations lack a faithful generic header position",
                ));
            }
            let physical_super = facts.super_class.as_ref().ok_or_else(|| {
                Error::unsupported(
                    "class_generic_source_unproved",
                    "physical superclass is absent",
                )
            })?;
            if facts.access_flags & ACC_INTERFACE != 0
                && physical_super.raw().0 != b"java/lang/Object"
            {
                return Err(Error::unsupported(
                    "class_generic_source_unproved",
                    "interface physical superclass is not java/lang/Object",
                ));
            }
            let physical_interfaces: Vec<Vec<u8>> = facts
                .interfaces
                .iter()
                .map(|name| name.raw().0.clone())
                .collect();
            let proof = prove_class_signature_erasure(
                &parsed,
                &physical_super.raw().0,
                &physical_interfaces,
                budget,
            )?;
            if !matches!(parsed.superclass.segments.as_slice(), [segment] if segment.arguments.is_empty())
                || parsed.interfaces.iter().any(|interface| {
                    !matches!(interface.segments.as_slice(), [segment] if segment.arguments.is_empty())
                })
            {
                return Err(Error::unsupported(
                    "class_generic_source_unproved",
                    "parameterized or nested parent needs a separate inherited-member proof",
                ));
            }
            let mut parameters = Vec::with_capacity(parsed.type_parameters.len());
            for parameter in &parsed.type_parameters {
                let name = std::str::from_utf8(&parameter.name).map_err(|_| {
                    Error::unsupported(
                        "class_generic_source_unproved",
                        "type variable name is not source UTF-8",
                    )
                })?;
                if !is_java_identifier(name) {
                    return Err(Error::unsupported(
                        "class_generic_source_unproved",
                        "type variable name is not a Java identifier",
                    ));
                }
                let mut bounds = Vec::new();
                for bound in parameter
                    .class_bound
                    .iter()
                    .chain(&parameter.interface_bounds)
                {
                    if !matches!(
                        bound,
                        SignatureType::Class(_) | SignatureType::TypeVariable(_)
                    ) {
                        return Err(Error::unsupported(
                            "class_generic_source_unproved",
                            "type-variable bound is not a Java class or type variable",
                        ));
                    }
                    bounds.push(spell_ordinary_signature_type(
                        bound,
                        &proof.type_parameters,
                        budget,
                        0,
                    )?);
                }
                let omit_object = bounds.len() == 1
                    && bounds[0] == "java.lang.Object"
                    && parameter.interface_bounds.is_empty();
                parameters.push(if bounds.is_empty() || omit_object {
                    name.to_owned()
                } else {
                    format!("{name} extends {}", bounds.join(" & "))
                });
            }
            let superclass = spell_ordinary_signature_type(
                &SignatureType::Class(parsed.superclass),
                &proof.type_parameters,
                budget,
                0,
            )?;
            let mut interfaces = Vec::with_capacity(parsed.interfaces.len());
            for interface in parsed.interfaces {
                interfaces.push(spell_ordinary_signature_type(
                    &SignatureType::Class(interface),
                    &proof.type_parameters,
                    budget,
                    0,
                )?);
            }
            let declaration = class_declaration_with_types(
                &self.name,
                facts,
                Some(&parameters.join(", ")),
                Some(&superclass),
                Some(&interfaces),
            );
            budget.charge(
                CountedBudgetDimension::OutputBytes,
                u64::try_from(declaration.len()).unwrap_or(u64::MAX),
            )?;
            Ok(Some((declaration, proof)))
        })();
        match result {
            Ok(Some((declaration, proof))) => {
                self.declaration = declaration;
                Ok(Some(proof))
            }
            Ok(None) => Ok(None),
            Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => Err(error),
            Err(error) => {
                self.generic_refusal = Some(error.to_string());
                Ok(None)
            }
        }
    }

    pub(crate) fn with_annotations(
        mut self,
        attributes: Vec<ClassAnnotationAttribute>,
        pool: &[CpEntryFacts],
    ) -> Self {
        let mut counts = std::collections::BTreeMap::<Vec<u8>, usize>::new();
        for attribute in &attributes {
            for annotation in &attribute.annotations {
                if let ElementValueFacts::Annotation {
                    type_descriptor, ..
                } = annotation
                {
                    *counts.entry(type_descriptor.0.clone()).or_default() += 1;
                }
            }
        }
        for attribute in &attributes {
            for annotation in &attribute.annotations {
                match spell_class_annotation(annotation, pool, &counts) {
                    Ok(spelling) => self.annotation_uses.push(spelling),
                    Err(reason) => self.annotation_refusals.push(reason),
                }
            }
        }
        self.annotation_attributes = attributes;
        self
    }
}

fn spell_class_annotation(
    annotation: &ElementValueFacts,
    pool: &[CpEntryFacts],
    counts: &std::collections::BTreeMap<Vec<u8>, usize>,
) -> std::result::Result<String, String> {
    let ElementValueFacts::Annotation {
        type_descriptor, ..
    } = annotation
    else {
        return Err("class annotation entry is not an annotation fact".to_owned());
    };
    if counts.get(&type_descriptor.0).copied().unwrap_or(0) > 1 {
        return Err(format!(
            "duplicate annotation type `{}` is conservatively refused",
            String::from_utf8_lossy(&type_descriptor.0)
        ));
    }
    spell_annotation_body(annotation, pool)
}

fn spell_annotation_body(
    annotation: &ElementValueFacts,
    pool: &[CpEntryFacts],
) -> std::result::Result<String, String> {
    let ElementValueFacts::Annotation {
        type_descriptor,
        elements,
    } = annotation
    else {
        return Err("annotation entry is not an annotation fact".to_owned());
    };
    let Some(type_name) = annotation_type_name(&type_descriptor.0) else {
        return Err(format!(
            "annotation type `{}` is not a Java source name",
            String::from_utf8_lossy(&type_descriptor.0)
        ));
    };
    let Some(elements) = elements
        .iter()
        .map(|element| {
            let name = String::from_utf8(element.name.0.clone()).ok()?;
            is_java_identifier(&name).then_some(())?;
            Some((name, resolve_default(&element.value, pool)?))
        })
        .collect::<Option<Vec<_>>>()
    else {
        return Err(format!(
            "annotation `@{type_name}` has an element with no faithful Java spelling"
        ));
    };
    if elements.is_empty() {
        Ok(format!("@{type_name}"))
    } else {
        let elements = elements
            .iter()
            .map(|(name, value)| format!("{name} = {}", value.spelling()))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!("@{type_name}({elements})"))
    }
}

#[derive(Clone, Copy)]
enum TypeUseOwner {
    Field,
    Method,
}

fn spell_type_annotation_uses(
    mut uses: TypeAnnotationUses,
    owner: TypeUseOwner,
    descriptor: &[u8],
    pool: &[CpEntryFacts],
    declarations: &MemberAnnotationUses,
    parameter_declarations: Option<&ParameterAnnotationUses>,
    parameter_count: Option<usize>,
) -> TypeAnnotationUses {
    uses.parameter_uses = vec![Vec::new(); parameter_count.unwrap_or_default()];
    let declaration_types = declarations
        .attributes
        .iter()
        .flat_map(|attribute| {
            attribute
                .annotations
                .iter()
                .filter_map(|annotation| match annotation {
                    ElementValueFacts::Annotation {
                        type_descriptor, ..
                    } => Some(type_descriptor.0.clone()),
                    _ => None,
                })
        })
        .collect::<std::collections::BTreeSet<_>>();
    let parameter_declaration_types = parameter_declarations.map(|parameters| {
        let mut by_position =
            vec![std::collections::BTreeSet::<Vec<u8>>::new(); parameter_count.unwrap_or_default()];
        for attribute in &parameters.attributes {
            if attribute.parameter_count.map(usize::from) != parameter_count {
                continue;
            }
            for (position, annotations) in attribute.parameters.iter().enumerate() {
                for annotation in annotations {
                    if let ElementValueFacts::Annotation {
                        type_descriptor, ..
                    } = annotation
                        && let Some(slot) = by_position.get_mut(position)
                    {
                        slot.insert(type_descriptor.0.clone());
                    }
                }
            }
        }
        by_position
    });
    let parameter_declarations_unmapped = parameter_declarations.is_some_and(|parameters| {
        parameters
            .attributes
            .iter()
            .any(|attribute| attribute.parameter_count.map(usize::from) != parameter_count)
    });
    let mut counts = std::collections::BTreeMap::<(u8, Vec<u8>, Vec<u8>), usize>::new();
    for attribute in &uses.attributes {
        for annotation in &attribute.annotations {
            if let ElementValueFacts::Annotation {
                type_descriptor, ..
            } = &annotation.annotation
            {
                *counts
                    .entry((
                        annotation.target_type,
                        annotation.target_info.clone(),
                        type_descriptor.0.clone(),
                    ))
                    .or_default() += 1;
            }
        }
    }
    for attribute in &mut uses.attributes {
        for annotation in &mut attribute.annotations {
            let ElementValueFacts::Annotation {
                type_descriptor, ..
            } = &annotation.annotation
            else {
                annotation.refusal =
                    Some("type annotation entry is not an annotation fact".to_owned());
                continue;
            };
            let position = match (
                owner,
                annotation.target_type,
                annotation.target_info.as_slice(),
            ) {
                (TypeUseOwner::Field, 0x13, []) => None,
                (TypeUseOwner::Method, 0x14, []) => None,
                (TypeUseOwner::Method, 0x16, [position]) => Some(usize::from(*position)),
                (TypeUseOwner::Field, 0x13, _) => {
                    annotation.refusal = Some("FIELD target_info must be empty".to_owned());
                    continue;
                }
                (TypeUseOwner::Method, 0x14, _) => {
                    annotation.refusal = Some("METHOD_RETURN target_info must be empty".to_owned());
                    continue;
                }
                (TypeUseOwner::Method, 0x16, _) => {
                    annotation.refusal = Some(
                        "METHOD_FORMAL_PARAMETER target_info must contain one index byte"
                            .to_owned(),
                    );
                    continue;
                }
                (TypeUseOwner::Field, _, _) => {
                    annotation.refusal =
                        Some("type annotation target does not belong to a field".to_owned());
                    continue;
                }
                (TypeUseOwner::Method, _, _) => {
                    annotation.refusal = Some(
                        "type annotation target is outside the supported method targets".to_owned(),
                    );
                    continue;
                }
            };
            if !annotation.type_path.is_empty() {
                annotation.refusal =
                    Some("non-empty type_path is not source-spellable in this change".to_owned());
                continue;
            }
            if let Some(position) = position
                && parameter_count.is_none_or(|count| position >= count)
            {
                annotation.refusal = Some(format!(
                    "formal parameter position {position} is outside the descriptor parameter count"
                ));
                continue;
            }
            if position.is_some() && parameter_declarations_unmapped {
                annotation.refusal = Some(
                    "parameter declaration annotations do not map to descriptor positions"
                        .to_owned(),
                );
                continue;
            }
            if counts
                .get(&(
                    annotation.target_type,
                    annotation.target_info.clone(),
                    type_descriptor.0.clone(),
                ))
                .copied()
                .unwrap_or(0)
                > 1
            {
                annotation.refusal = Some(
                    "duplicate annotation type at the same target is conservatively refused"
                        .to_owned(),
                );
                continue;
            }
            let declaration_conflict = match position {
                Some(position) => parameter_declaration_types
                    .as_ref()
                    .and_then(|by_position| by_position.get(position))
                    .is_some_and(|types| types.contains(&type_descriptor.0)),
                None => declaration_types.contains(&type_descriptor.0),
            };
            if declaration_conflict {
                annotation.refusal = Some(
                    "same annotation type is present on the declaration and type target".to_owned(),
                );
                continue;
            }
            let Ok(spelling) = spell_annotation_body(&annotation.annotation, pool) else {
                annotation.refusal =
                    Some("annotation value has no faithful Java source spelling".to_owned());
                continue;
            };
            if qualify_type_descriptor(descriptor, owner, position).is_none() {
                annotation.refusal = Some("target type is primitive, array, single-segment, or has ambiguous nested-class spelling".to_owned());
                continue;
            }
            annotation.spelling = Some(spelling.clone());
            match position {
                Some(position) => uses.parameter_uses[position].push(spelling),
                None if matches!(owner, TypeUseOwner::Field) => uses.field_uses.push(spelling),
                None => uses.return_uses.push(spelling),
            }
        }
    }
    for attribute in &uses.attributes {
        for annotation in &attribute.annotations {
            if let Some(reason) = &annotation.refusal {
                uses.refusals.push(format!(
                    "target=0x{:02x} info={:?} path={:?}: {reason}",
                    annotation.target_type, annotation.target_info, annotation.type_path
                ));
            }
        }
    }
    uses
}

fn qualify_type_descriptor(
    descriptor: &[u8],
    owner: TypeUseOwner,
    parameter: Option<usize>,
) -> Option<String> {
    let facts = descriptor_facts(
        descriptor,
        match owner {
            TypeUseOwner::Field => DescriptorKind::Field,
            TypeUseOwner::Method => DescriptorKind::Method,
        },
    )
    .ok()?;
    let component = match owner {
        TypeUseOwner::Field => facts.single()?,
        TypeUseOwner::Method => match parameter {
            Some(position) => facts.parameters().get(position)?,
            None => facts.result()?,
        },
    };
    if component.is_array() || !matches!(component.base(), Base::Object(_)) {
        return None;
    }
    let name = source_type(component)?;
    let (package, simple) = name.rsplit_once('.')?;
    if simple.contains('$') || package.is_empty() {
        return None;
    }
    Some(format!("{package}.@{{annotation}} {simple}"))
}

fn decorate_qualified_type(ty: &str, annotations: &[String]) -> Option<String> {
    let (package, simple) = ty.rsplit_once('.')?;
    if package.is_empty() || simple.contains('$') || ty.contains("[]") {
        return None;
    }
    Some(format!("{package}.{} {simple}", annotations.join(" ")))
}

fn replace_first(text: &str, from: &str, to: &str) -> String {
    text.find(from).map_or_else(
        || text.to_owned(),
        |at| format!("{}{}{}", &text[..at], to, &text[at + from.len()..]),
    )
}

fn spell_member_annotation_uses(
    mut annotations: MemberAnnotationUses,
    pool: &[CpEntryFacts],
) -> MemberAnnotationUses {
    let mut counts = std::collections::BTreeMap::<Vec<u8>, usize>::new();
    for attribute in &annotations.attributes {
        for annotation in &attribute.annotations {
            if let ElementValueFacts::Annotation {
                type_descriptor, ..
            } = annotation
            {
                *counts.entry(type_descriptor.0.clone()).or_default() += 1;
            }
        }
    }
    for attribute in &annotations.attributes {
        for annotation in &attribute.annotations {
            match annotation {
                ElementValueFacts::Annotation {
                    type_descriptor, ..
                } if counts.get(&type_descriptor.0).copied().unwrap_or_default() > 1 => {
                    annotations.refusals.push(format!(
                        "duplicate annotation type `{}` at one member position is conservatively refused",
                        String::from_utf8_lossy(&type_descriptor.0)
                    ));
                }
                _ => match spell_annotation_body(annotation, pool) {
                    Ok(spelling) => annotations.uses.push(spelling),
                    Err(reason) => annotations.refusals.push(reason),
                },
            }
        }
    }
    annotations
}

fn spell_parameter_annotation_uses(
    mut annotations: ParameterAnnotationUses,
    pool: &[CpEntryFacts],
    expected_count: Option<usize>,
) -> ParameterAnnotationUses {
    let Some(expected_count) = expected_count else {
        for attribute in &annotations.attributes {
            if attribute.parameter_count.is_some() {
                annotations.refusals.push(format!(
                    "parameter annotation attribute `{}` has no descriptor positions to attach to",
                    String::from_utf8_lossy(&attribute.attribute.name.raw().0)
                ));
            }
        }
        return annotations;
    };
    annotations.uses_by_position = vec![Vec::new(); expected_count];
    let mut by_position = vec![Vec::<&ElementValueFacts>::new(); expected_count];
    for attribute in &annotations.attributes {
        let Some(parameter_count) = attribute.parameter_count else {
            continue;
        };
        if usize::from(parameter_count) != expected_count
            || attribute.parameters.len() != expected_count
        {
            annotations.refusals.push(format!(
                "parameter annotation attribute `{}` declares {parameter_count} position(s) for a descriptor with {expected_count} parameter(s); its groups are not aligned",
                String::from_utf8_lossy(&attribute.attribute.name.raw().0)
            ));
            continue;
        }
        for (position, group) in attribute.parameters.iter().enumerate() {
            by_position[position].extend(group);
        }
    }
    for (position, entries) in by_position.into_iter().enumerate() {
        let mut counts = std::collections::BTreeMap::<Vec<u8>, usize>::new();
        for annotation in &entries {
            if let ElementValueFacts::Annotation {
                type_descriptor, ..
            } = annotation
            {
                *counts.entry(type_descriptor.0.clone()).or_default() += 1;
            }
        }
        for annotation in entries {
            match annotation {
                ElementValueFacts::Annotation {
                    type_descriptor, ..
                } if counts.get(&type_descriptor.0).copied().unwrap_or_default() > 1 => {
                    annotations.refusals.push(format!(
                        "parameter position {position}: duplicate annotation type `{}` is conservatively refused",
                        String::from_utf8_lossy(&type_descriptor.0)
                    ));
                }
                _ => match spell_annotation_body(annotation, pool) {
                    Ok(spelling) => annotations.uses_by_position[position].push(spelling),
                    Err(reason) => annotations
                        .refusals
                        .push(format!("parameter position {position}: {reason}")),
                },
            }
        }
    }
    annotations
}

fn refuse_uses(annotations: &mut MemberAnnotationUses, reason: &str) {
    for spelling in annotations.uses.drain(..) {
        annotations.refusals.push(format!("{reason}: {spelling}"));
    }
}

fn refuse_parameter_uses(annotations: &mut ParameterAnnotationUses, reason: &str) {
    for (position, uses) in annotations.uses_by_position.iter_mut().enumerate() {
        for spelling in uses.drain(..) {
            annotations.refusals.push(format!(
                "parameter position {position}: {reason}: {spelling}"
            ));
        }
    }
}

impl ClassSourceField {
    /// One field record with the spelling the text writes for it, initializer included when the
    /// field's own `ConstantValue` attribute states one.
    pub(crate) fn of(
        item: FieldItem,
        constant: Option<&MemberDefault>,
        annotations: MemberAnnotationFacts,
        pool: &[CpEntryFacts],
    ) -> Self {
        let mut spelled = spell_field_declaration(&item, constant);
        let mut member_annotations =
            spell_member_annotation_uses(annotations.declaration.clone(), pool);
        let type_annotations = spell_type_annotation_uses(
            annotations.type_uses,
            TypeUseOwner::Field,
            &item.descriptor.raw().0,
            pool,
            &member_annotations,
            None,
            None,
        );
        if let Some(declaration) = &spelled.declaration
            && let (Some(ty), Some(decorated)) = (
                field_type(&item.descriptor.raw().0),
                decorate_qualified_type(
                    &field_type(&item.descriptor.raw().0).unwrap_or_default(),
                    &type_annotations.field_uses,
                ),
            )
            && !type_annotations.field_uses.is_empty()
        {
            spelled.declaration = Some(replace_first(declaration, &ty, &decorated));
        }
        if spelled.declaration.is_none() {
            refuse_uses(
                &mut member_annotations,
                "field descriptor cannot be spelled as a Java declaration",
            );
        }
        spelled.annotations = member_annotations;
        spelled.type_annotations = type_annotations;
        let markers = markers_of(&spelled);
        Self {
            item,
            declaration: spelled.declaration,
            annotations: spelled.annotations,
            type_annotations: spelled.type_annotations,
            markers,
        }
    }
}

impl ClassSourceMethod {
    /// The one physical-recovery marker that may be replaced by a later complete family re-run.
    /// It remains on this physical record; only the separately assembled source text omits it.
    pub(crate) fn has_only_explanation_marker(&self) -> bool {
        let ClassSourceOutcome::Recovered { report, .. } = &self.outcome else {
            return false;
        };
        report.content == RecoveryContent::ExplanationOnly
            && self.markers
                == [format!(
                    "// jarde: not recovered: the recovery run for `{}` produced no statement (explanation only); the artifact's own comment lines are below",
                    label(&self.item)
                )]
    }

    /// Stage the entire declaration, body placement and source note before publishing them.
    pub(crate) fn project_generic(
        &mut self,
        declaration: String,
        signature: &[u8],
        proof: &str,
        budget: &mut Budget,
    ) -> Result<()> {
        let placed = match &self.outcome {
            ClassSourceOutcome::Recovered { report, .. } => {
                let Some(body) = artifact(&report.text) else {
                    return Ok(());
                };
                Placed::Block(body)
            }
            ClassSourceOutcome::NoBody if self.no_body_kind.is_some() => Placed::Without,
            _ => return Ok(()),
        };
        let marker = format!(
            "// jarde: generic Signature `{}` projected after descriptor erasure and {proof}",
            comment_text(&String::from_utf8_lossy(signature)),
        );
        let mut markers = self.markers.clone();
        markers.push(marker);
        let text = prefix_method_annotations(
            match placed {
                Placed::Without => declaration_member(&declaration, &markers),
                body => block_member(&declaration, body, &markers),
            },
            &self.annotations,
        );
        budget.charge(
            CountedBudgetDimension::OutputBytes,
            u64::try_from(text.len()).unwrap_or(u64::MAX),
        )?;
        self.declaration = Some(declaration);
        self.text = text;
        self.markers = markers;
        Ok(())
    }

    pub(crate) fn refuse_generic(&mut self, reason: &str, budget: &mut Budget) -> Result<()> {
        let marker = format!(
            "// jarde: generic Signature projection refused for `{}`: {}",
            label(&self.item),
            comment_text(reason)
        );
        let text = format!("    {marker}\n{}", self.text);
        budget.charge(
            CountedBudgetDimension::OutputBytes,
            u64::try_from(text.len()).unwrap_or(u64::MAX),
        )?;
        self.markers.push(marker);
        self.text = text;
        Ok(())
    }

    /// The complete-source explanation for this bridge's original physical identity, call BCI and
    /// retained source-level target.
    pub(crate) fn bridge_projection_marker(&self, target: &Self, call_bci: u32) -> String {
        format!(
            "// jarde: projected bridge `{}` at physical method record {}: bridge@1 proved a pure call at BCI {} to source declaration `{}` at method record {}; the resolved erased contract lets javac regenerate the bridge",
            label(&self.item),
            self.item.index,
            call_bci,
            label(&target.item),
            target.item.index,
        )
    }

    /// Replaces this member's complete Java declaration with the supplied proof marker while
    /// retaining its physical declaration and original recovery outcome for the report.
    pub(crate) fn project_bridge(&mut self, marker: String) {
        self.markers.push(marker);
        self.text = comment_member(&self.markers);
    }

    /// Atomically replaces only the rendered body with a re-emission of the same-run AST.
    pub(crate) fn project_enum_switch(&mut self, recovery_text: &str, marker: String) -> bool {
        let Some(declaration) = self.declaration.as_ref() else {
            return false;
        };
        let Some(artifact) = artifact(recovery_text) else {
            return false;
        };
        let mut markers = self.markers.clone();
        markers.push(marker);
        let text = block_member(declaration, Placed::Block(artifact), &markers);
        self.text = prefix_method_annotations(text, &self.annotations);
        self.markers = markers;
        true
    }

    pub(crate) fn array_projection_text(
        &self,
        recovery_text: &str,
        marker: String,
    ) -> Option<String> {
        let declaration = self.declaration.as_ref()?;
        let artifact = artifact(recovery_text)?;
        let mut markers = self.markers.clone();
        markers.push(marker);
        Some(prefix_method_annotations(
            block_member(declaration, Placed::Block(artifact), &markers),
            &self.annotations,
        ))
    }

    /// One member whose declaration carries no `Code` attribute: the declaration Java spells for it
    /// (`public abstract void run();`) and the markers that say why it has no body here. `kind` is
    /// what the member's own flags declare, which decides the form: an `abstract` or `native` member
    /// is written as the body-less declaration Java writes for it, and one that declares neither is
    /// written as a block whose whole content is the marker.
    pub(crate) fn no_body(item: MethodItem, kind: Option<NoBodyKind>, spelled: Spelled) -> Self {
        let mut markers = markers_of(&spelled);
        let member = label(&item);
        markers.push(match kind {
            Some(NoBodyKind::Abstract) => format!(
                "// jarde: no body: the member `{member}` is declared abstract and its declaration \
                 carries no Code attribute"
            ),
            Some(NoBodyKind::Native) => format!(
                "// jarde: no body: the member `{member}` is declared native and its declaration \
                 carries no Code attribute"
            ),
            None => format!(
                "// jarde: no body: the member `{member}` declares no Code attribute and is neither \
                 abstract nor native"
            ),
        });
        let text = match &spelled.declaration {
            Some(declaration) if kind.is_some() => declaration_member(declaration, &markers),
            Some(declaration) => block_member(declaration, Placed::Without, &markers),
            None => comment_member(&markers),
        };
        let text = prefix_method_annotations(text, &spelled.annotations);
        Self {
            item,
            no_body_kind: kind,
            declaration: spelled.declaration,
            annotations: spelled.annotations,
            parameter_annotations: spelled.parameter_annotations,
            type_annotations: spelled.type_annotations,
            text,
            markers,
            outcome: ClassSourceOutcome::NoBody,
            enum_constructor_source_signature: false,
            enum_constructor_no_arg_source_signature: false,
            enum_constructor_signature_erasure_refused: false,
        }
    }

    /// One member whose raw descriptor is not one this presentation can read: no declaration, no
    /// run, and the markers that state both.
    pub(crate) fn unspelled(item: MethodItem, spelled: Spelled) -> Self {
        let markers = markers_of(&spelled);
        Self {
            item,
            no_body_kind: None,
            declaration: None,
            text: comment_member(&markers),
            annotations: spelled.annotations,
            parameter_annotations: spelled.parameter_annotations,
            type_annotations: spelled.type_annotations,
            markers,
            outcome: ClassSourceOutcome::Unspelled,
            enum_constructor_source_signature: false,
            enum_constructor_no_arg_source_signature: false,
            enum_constructor_signature_erasure_refused: false,
        }
    }

    /// One member whose own run was refused before it produced a report.
    pub(crate) fn refused(
        item: MethodItem,
        spelled: Spelled,
        execution: ExecutionReport,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        let mut markers = markers_of(&spelled);
        markers.push(format!(
            "// jarde: not recovered: the run for `{}` was refused ({})",
            label(&item),
            stop_code(&execution).unwrap_or_else(|| "reason not stated".to_owned())
        ));
        let text = match &spelled.declaration {
            Some(declaration) => block_member(declaration, Placed::Without, &markers),
            None => comment_member(&markers),
        };
        let text = prefix_method_annotations(text, &spelled.annotations);
        Self {
            item,
            no_body_kind: None,
            declaration: spelled.declaration,
            annotations: spelled.annotations,
            parameter_annotations: spelled.parameter_annotations,
            type_annotations: spelled.type_annotations,
            text,
            markers,
            outcome: ClassSourceOutcome::Refused {
                execution,
                diagnostics,
            },
            enum_constructor_source_signature: false,
            enum_constructor_no_arg_source_signature: false,
            enum_constructor_signature_erasure_refused: false,
        }
    }

    /// One member one recovery run presented, with the facts of that same run.
    pub(crate) fn recovered(
        item: MethodItem,
        spelled: Spelled,
        report: Box<RecoveryReport>,
        analysis: ClassSourceRunFacts,
    ) -> Self {
        let mut markers = recovered_markers(&item, &spelled, &report, &analysis);
        let placed = if report.text.is_empty() {
            Placed::Without
        } else {
            match artifact(&report.text) {
                Some(artifact) => Placed::Block(artifact),
                None => {
                    markers.push(not_placed_marker(&item));
                    Placed::Quoted(&report.text)
                }
            }
        };
        let text = match &spelled.declaration {
            Some(declaration) => block_member(declaration, placed, &markers),
            None => comment_member(&markers),
        };
        let text = prefix_method_annotations(text, &spelled.annotations);
        Self {
            item,
            no_body_kind: None,
            declaration: spelled.declaration,
            annotations: spelled.annotations,
            parameter_annotations: spelled.parameter_annotations,
            type_annotations: spelled.type_annotations,
            text,
            markers,
            outcome: ClassSourceOutcome::Recovered { report, analysis },
            enum_constructor_source_signature: false,
            enum_constructor_no_arg_source_signature: false,
            enum_constructor_signature_erasure_refused: false,
        }
    }
}

/// The markers one spelling carries, as the vector the member record publishes.
fn markers_of(spelled: &Spelled) -> Vec<String> {
    let mut markers = spelled.marker.clone().into_iter().collect::<Vec<_>>();
    markers.extend(spelled.annotations.refusals.iter().map(|reason| {
        format!(
            "// jarde: member annotation refused: {}",
            comment_text(reason)
        )
    }));
    markers.extend(spelled.parameter_annotations.refusals.iter().map(|reason| {
        format!(
            "// jarde: parameter annotation refused: {}",
            comment_text(reason)
        )
    }));
    markers.extend(spelled.type_annotations.refusals.iter().map(|reason| {
        format!(
            "// jarde: type annotation refused: {}",
            comment_text(reason)
        )
    }));
    markers
}

fn prefix_method_annotations(text: String, annotations: &MemberAnnotationUses) -> String {
    if annotations.uses.is_empty() {
        return text;
    }
    let mut prefixed = String::new();
    for annotation in &annotations.uses {
        prefixed.push_str("    ");
        prefixed.push_str(annotation);
        prefixed.push('\n');
    }
    prefixed.push_str(&text);
    prefixed
}

/// The text-only portion of a proved enum projection. The physical member records remain intact.
pub(crate) struct EnumConstantSourceProjection {
    constant_field_indices: Vec<u64>,
    backing_field_index: u64,
    implicit_method_indices: Vec<u64>,
    constants_text: String,
    constructor_texts: Vec<(u64, String)>,
    initializer_text: Option<String>,
}

/// The already-proved choices and bookkeeping that control one class-text assembly.
pub(crate) struct ClassSourceTextContext<'a> {
    pub(crate) initializer_field_order: Option<&'a [usize]>,
    pub(crate) declared_methods: u64,
    pub(crate) member_table: Option<&'a MemberTableStop>,
    pub(crate) execution: &'a ExecutionReport,
    pub(crate) enum_projection: Option<&'a EnumConstantSourceProjection>,
    pub(crate) array_helper_indices: Option<&'a [u64]>,
    pub(crate) array_method_texts: Option<&'a [(u64, String)]>,
    pub(crate) array_helper_markers: Option<&'a [String]>,
}

pub(crate) fn prepare_enum_constant_source_projection(
    declaration: &ClassSourceDeclaration,
    fields: &[ClassSourceField],
    methods: &[ClassSourceMethod],
    group: &crate::enum_constants::ProvedOrdinaryEnumConstantGroup,
    terminal_constructor_body: Option<String>,
    initializer: Option<(u64, String)>,
    budget: &mut Budget,
) -> Result<Option<EnumConstantSourceProjection>> {
    let Ok(constructor_index) = usize::try_from(group.constructor_method_index) else {
        return Ok(None);
    };
    let Ok(source_field_index) = usize::try_from(group.constructor_field_index) else {
        return Ok(None);
    };
    let (Some(constructor), Some(source_field)) = (
        methods.get(constructor_index),
        fields.get(source_field_index),
    ) else {
        return Ok(None);
    };
    let has_only_enum_signature_marker =
        constructor.enum_constructor_signature_erasure_refused && constructor.markers.len() == 1;
    if constructor.item.index != group.constructor_method_index
        || source_field.item.index != group.constructor_field_index
        || constructor.declaration.is_none()
        || source_field.declaration.is_none()
        || (group.constructor_signature_present && !constructor.enum_constructor_source_signature)
        || (!constructor.markers.is_empty() && !has_only_enum_signature_marker)
        || !constructor.annotations.refusals.is_empty()
        || !constructor.parameter_annotations.refusals.is_empty()
        || !constructor.type_annotations.refusals.is_empty()
        || constructor
            .parameter_annotations
            .uses_by_position
            .iter()
            .take(2)
            .any(|uses| !uses.is_empty())
        || constructor
            .type_annotations
            .parameter_uses
            .iter()
            .take(2)
            .any(|uses| !uses.is_empty())
    {
        return Ok(None);
    }

    let (field_name, aliased) = written_name(&source_field.item.name.raw().0);
    if aliased {
        return Ok(None);
    }
    let mut constructor_texts = Vec::new();
    if let Some(delegating_index) = group.delegating_constructor_method_index {
        let Ok(delegating_index_usize) = usize::try_from(delegating_index) else {
            return Ok(None);
        };
        let Some(delegating) = methods.get(delegating_index_usize) else {
            return Ok(None);
        };
        let valid_hidden_constructor = |method: &ClassSourceMethod| {
            method.declaration.is_some()
                && method.enum_constructor_no_arg_source_signature
                && method.enum_constructor_signature_erasure_refused
                && method.markers.len() == 1
                && method.annotations.refusals.is_empty()
                && method.parameter_annotations.refusals.is_empty()
                && method.type_annotations.refusals.is_empty()
                && method
                    .parameter_annotations
                    .uses_by_position
                    .iter()
                    .all(Vec::is_empty)
                && method
                    .type_annotations
                    .parameter_uses
                    .iter()
                    .all(Vec::is_empty)
        };
        if delegating.item.index != delegating_index
            || delegating.item.name.raw().0 != b"<init>"
            || delegating.item.descriptor.raw().0 != b"(Ljava/lang/String;I)V"
            || !valid_hidden_constructor(delegating)
            || group.constructor_body.is_none()
            || terminal_constructor_body.is_none()
            || group.constructor_signature_present != constructor.enum_constructor_source_signature
        {
            return Ok(None);
        }

        let mut delegating_declaration = String::new();
        if let Some(modifier) = visibility(delegating.item.access_flags) {
            delegating_declaration.push_str(modifier);
            delegating_declaration.push(' ');
        }
        delegating_declaration.push_str(&declaration.name);
        delegating_declaration.push_str("()");
        let delegating_body =
            format!("    {delegating_declaration} {{\n        this(0);\n    }}\n");
        constructor_texts.push((
            delegating_index,
            prefix_method_annotations(delegating_body, &delegating.annotations),
        ));

        let source_parameter_type_uses = constructor
            .type_annotations
            .parameter_uses
            .get(2)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let Some(parameter_type) = (if source_parameter_type_uses.is_empty() {
            Some("int".to_owned())
        } else {
            decorate_qualified_type("int", source_parameter_type_uses)
        }) else {
            return Ok(None);
        };
        let source_parameter_annotations = constructor
            .parameter_annotations
            .uses_by_position
            .get(2)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let parameter_annotations = source_parameter_annotations
            .iter()
            .map(|annotation| format!("{annotation} "))
            .collect::<String>();
        let mut terminal_declaration = String::new();
        if let Some(modifier) = visibility(constructor.item.access_flags) {
            terminal_declaration.push_str(modifier);
            terminal_declaration.push(' ');
        }
        terminal_declaration.push_str(&declaration.name);
        terminal_declaration.push('(');
        terminal_declaration.push_str(&parameter_annotations);
        terminal_declaration.push_str(&parameter_type);
        terminal_declaration.push_str(" arg0)");
        let Some(statements) = terminal_constructor_body else {
            return Ok(None);
        };
        let terminal_body = format!("    {terminal_declaration} {{\n{statements}    }}\n");
        constructor_texts.push((
            group.constructor_method_index,
            prefix_method_annotations(terminal_body, &constructor.annotations),
        ));
    } else {
        if terminal_constructor_body.is_some() {
            return Ok(None);
        }
        let source_parameter_type_uses = constructor
            .type_annotations
            .parameter_uses
            .get(2)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let Some(parameter_type) = (if source_parameter_type_uses.is_empty() {
            Some("int".to_owned())
        } else {
            decorate_qualified_type("int", source_parameter_type_uses)
        }) else {
            return Ok(None);
        };
        let source_parameter_annotations = constructor
            .parameter_annotations
            .uses_by_position
            .get(2)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let parameter_annotations = source_parameter_annotations
            .iter()
            .map(|annotation| format!("{annotation} "))
            .collect::<String>();
        let mut constructor_declaration = String::new();
        if let Some(modifier) = visibility(constructor.item.access_flags) {
            constructor_declaration.push_str(modifier);
            constructor_declaration.push(' ');
        }
        constructor_declaration.push_str(&declaration.name);
        constructor_declaration.push('(');
        constructor_declaration.push_str(&parameter_annotations);
        constructor_declaration.push_str(&parameter_type);
        constructor_declaration.push_str(" arg0)");
        let constructor_body = format!(
            "    {constructor_declaration} {{\n        this.{field_name} = arg0;\n    }}\n"
        );
        // The one known marker comes from the JVM-only name/ordinal parameters making the physical
        // descriptor longer than the already parsed source Signature. The enum proof checks that
        // source tail independently; no other constructor marker is absorbed by this projection.
        constructor_texts.push((
            group.constructor_method_index,
            prefix_method_annotations(constructor_body, &constructor.annotations),
        ));
    }

    if initializer.is_some() {
        budget.charge(
            CountedBudgetDimension::IrItems,
            u64::try_from(fields.len()).unwrap_or(u64::MAX),
        )?;
    }
    let initializer_text = match initializer {
        Some((field_index, text)) => {
            let mut target = None;
            for field in fields {
                budget.poll()?;
                if field.item.index == field_index && target.replace(field).is_some() {
                    return Ok(None);
                }
            }
            let Some(target) = target else {
                return Ok(None);
            };
            if target.item.name.raw().0 != b"totalUnits"
                || target.item.descriptor.raw().0 != b"I"
                || target.item.access_flags != 0x0008
                || target.item.identity.owner != constructor.item.identity.owner
                || target.declaration.is_none()
                || !target.markers.is_empty()
            {
                return Ok(None);
            }
            Some(text)
        }
        None => None,
    };

    let mut constants_text = String::new();
    for (index, constant) in group.constants.iter().enumerate() {
        constants_text.push_str("    ");
        constants_text.push_str(&constant.name);
        if let Some(argument) = constant.source_argument {
            constants_text.push('(');
            constants_text.push_str(&argument.to_string());
            constants_text.push(')');
        }
        if index + 1 == group.constants.len() {
            constants_text.push_str(";\n");
        } else {
            constants_text.push_str(",\n");
        }
    }
    let output_bytes = u64::try_from(constants_text.len())
        .unwrap_or(u64::MAX)
        .saturating_add(constructor_texts.iter().fold(0u64, |total, (_, text)| {
            total.saturating_add(u64::try_from(text.len()).unwrap_or(u64::MAX))
        }))
        .saturating_add(
            initializer_text
                .as_ref()
                .map_or(0, |text| u64::try_from(text.len()).unwrap_or(u64::MAX)),
        );
    budget.charge(CountedBudgetDimension::OutputBytes, output_bytes)?;

    Ok(Some(EnumConstantSourceProjection {
        constant_field_indices: group
            .constants
            .iter()
            .map(|constant| constant.field_index)
            .collect(),
        backing_field_index: group.backing_field_index,
        implicit_method_indices: [
            group.initializer_method_index,
            group.values_method_index,
            group.value_of_method_index,
            group.values_factory_method_index,
        ]
        .into(),
        constants_text,
        constructor_texts,
        initializer_text,
    }))
}

/// Place the members retained by the proved body group directly inside their owning constant.
/// The method text is the selected child's same-run member artifact, not parsed class source.
pub(crate) fn prepare_enum_constant_body_source_projection(
    declaration: &ClassSourceDeclaration,
    fields: &[ClassSourceField],
    methods: &[ClassSourceMethod],
    group: &crate::enum_constants::ProvedEnumConstantBodyGroup,
    shape: &crate::facade::PendingEnumConstantBodyGroupShape,
    budget: &mut Budget,
) -> Result<Option<EnumConstantSourceProjection>> {
    if group.constants.len() != 2 || shape.constants.len() != 2 || shape.implicit_members.len() != 6
    {
        return Ok(None);
    }
    let mut constants_text = String::new();
    let mut constant_field_indices = Vec::with_capacity(2);
    for (position, constant) in group.constants.iter().enumerate() {
        budget.poll()?;
        let Some(field) = usize::try_from(constant.field_index)
            .ok()
            .and_then(|index| fields.get(index))
        else {
            return Ok(None);
        };
        if shape.constants[position].field_index != constant.field_index
            || shape.constants[position].constructor_bci != constant.constructor_bci
            || field.item.index != constant.field_index
            || field.item.identity.owner != declaration.item.definition
            || field.declaration.is_none()
            || !field.markers.is_empty()
        {
            return Ok(None);
        }
        let (name, aliased) = written_name(&field.item.name.raw().0);
        if aliased {
            return Ok(None);
        }
        constants_text.push_str("    ");
        constants_text.push_str(&name);
        match (&constant.subclass, &constant.methods) {
            (None, None) => {}
            (Some(subclass), Some(body_methods)) if !body_methods.is_empty() => {
                constants_text.push_str(" {\n");
                constants_text.push_str("        // jarde: selected enum child definition: ");
                constants_text.push_str(&format!("{subclass:?}"));
                constants_text.push('\n');
                for method in body_methods.iter() {
                    budget.poll()?;
                    if method.item.identity.owner != *subclass
                        || method.declaration.is_none()
                        || !method.markers.is_empty()
                        || !matches!(method.outcome, ClassSourceOutcome::Recovered { .. })
                        || !method.text.ends_with("}\n")
                    {
                        return Ok(None);
                    }
                    constants_text.push_str(&indent(&method.text, 1));
                }
                constants_text.push_str("    }");
            }
            _ => return Ok(None),
        }
        constants_text.push_str(if position + 1 == group.constants.len() {
            ";\n"
        } else {
            ",\n"
        });
        constant_field_indices.push(constant.field_index);
    }
    let backing_field_index = shape.implicit_members[0].table_index;
    let backing_field = usize::try_from(backing_field_index)
        .ok()
        .and_then(|index| fields.get(index));
    if shape.implicit_members[0].name != b"$VALUES"
        || backing_field.is_none_or(|field| {
            field.item.index != backing_field_index
                || field.item.identity.owner != declaration.item.definition
        })
    {
        return Ok(None);
    }
    let mut implicit_method_indices = Vec::new();
    for member in shape
        .implicit_members
        .iter()
        .skip(1)
        .chain(shape.constructors.iter())
    {
        let Some(method) = usize::try_from(member.table_index)
            .ok()
            .and_then(|index| methods.get(index))
        else {
            return Ok(None);
        };
        if method.item.index != member.table_index
            || method.item.identity.owner != declaration.item.definition
            || method.item.name.raw().0 != member.name
            || method.item.descriptor.raw().0 != member.descriptor
        {
            return Ok(None);
        }
        if !implicit_method_indices.contains(&member.table_index) {
            implicit_method_indices.push(member.table_index);
        }
    }
    budget.charge(
        CountedBudgetDimension::OutputBytes,
        u64::try_from(constants_text.len()).unwrap_or(u64::MAX),
    )?;
    Ok(Some(EnumConstantSourceProjection {
        constant_field_indices,
        backing_field_index,
        implicit_method_indices,
        constants_text,
        constructor_texts: Vec::new(),
        initializer_text: None,
    }))
}

/// The marker of a produced artifact whose shape this presentation does not recognize.
fn not_placed_marker(item: &MethodItem) -> String {
    format!(
        "// jarde: not placed: the recovery artifact for `{}` is not a block this presentation can \
         place, so its text is quoted below line by line",
        label(item)
    )
}

/// The class-source text: the `package` line the class's own name states, the declaration, the
/// fields in the requested projection order, the members, and the class file's own shortfall where
/// there is one. A supplied initializer order is a complete permutation prepared only after a full
/// class-level proof and atomic expression emission.
///
/// The `package` statement is the one thing written above the class: it is the package
/// [`package_name`] reads off `this_class`, and a name with no `/` states none — the default
/// package, which source spells with no statement at all. It is a statement about that one name and
/// not about the directory the class was found in, which no read of this presentation established.
pub(crate) fn source_text(
    declaration: &ClassSourceDeclaration,
    fields: &[ClassSourceField],
    methods: &[ClassSourceMethod],
    context: &ClassSourceTextContext<'_>,
) -> String {
    source_text_with_member(declaration, fields, methods, context, None, &mut Vec::new())
        .expect("ordinary class writer has no derived ranges to translate")
}

/// A family source unit is assembled from physical member records, with only certified body
/// replacements and the one certified capture field omitted. No physical report is mutated.
pub(crate) struct MemberFamilyTextProjection<'a> {
    pub(crate) relation: &'a ClassSourceMemberRelation,
    pub(crate) child: &'a ClassSourceReport,
    pub(crate) capture: &'a MemberCaptureProof,
    pub(crate) root_methods: &'a [MemberFamilyMethodText],
    pub(crate) child_methods: &'a [MemberFamilyMethodText],
}

pub(crate) struct MemberFamilyMethodText {
    pub(crate) index: u64,
    pub(crate) text: String,
    /// Ranges in `text`, translated by the family writer when it appends the method.
    pub(crate) derived: Vec<MemberFamilyDerivedProjection>,
}

/// Translate one exact, single-line recovery span through this writer's artifact placement.
/// The equality check catches any drift in the envelope/statements indentation contract.
pub(crate) fn member_family_recovered_span(
    method: &ClassSourceMethod,
    recovery: &RecoveryReport,
    start: usize,
    end: usize,
) -> Option<(usize, usize)> {
    let declaration = method.declaration.as_ref()?;
    let body = artifact(&recovery.text)?;
    let source = recovery.text.get(start..end)?;
    if source.is_empty() || source.contains('\n') {
        return None;
    }
    let annotations = method
        .annotations
        .uses
        .iter()
        .map(|annotation| 4 + annotation.len() + 1)
        .sum::<usize>();
    let marker_len = if method.has_only_explanation_marker() {
        0
    } else {
        method
            .markers
            .iter()
            .map(|marker| indent(&format!("{marker}\n"), 2).len())
            .sum()
    };
    let block_prefix = annotations + format!("    {declaration} {{\n").len() + marker_len;
    let envelope_end = body.envelope.len();
    let (part, local_start, local_end, depth, prefix) = if end <= envelope_end {
        (body.envelope, start, end, 2, block_prefix)
    } else {
        let statement_start = envelope_end + 2;
        if start < statement_start || end > statement_start + body.statements.len() {
            return None;
        }
        (
            body.statements,
            start - statement_start,
            end - statement_start,
            1,
            block_prefix + indent(body.envelope, 2).len(),
        )
    };
    let mapped_start = prefix + indented_offset(part, local_start, depth, false)?;
    let mapped_end = prefix + indented_offset(part, local_end, depth, true)?;
    let text = member_family_recovered_method_text(method, recovery)?;
    (text.get(mapped_start..mapped_end)? == source).then_some((mapped_start, mapped_end))
}

fn indented_offset(text: &str, offset: usize, depth: usize, end_boundary: bool) -> Option<usize> {
    if offset > text.len() || !text.is_char_boundary(offset) {
        return None;
    }
    let mut source = 0;
    let mut written = 0;
    for line in text.split_inclusive('\n') {
        if end_boundary && offset == source {
            return Some(written);
        }
        let padding = if line.trim_end_matches('\n').is_empty() {
            0
        } else {
            4 * depth
        };
        if offset < source + line.len() {
            return Some(written + padding + offset - source);
        }
        written += padding + line.len();
        source += line.len();
    }
    (offset == source).then_some(written)
}

pub(crate) fn member_family_recovered_method_text(
    method: &ClassSourceMethod,
    recovery: &RecoveryReport,
) -> Option<String> {
    let declaration = method.declaration.as_ref()?;
    let body = artifact(&recovery.text)?;
    let markers = if method.has_only_explanation_marker() {
        &[][..]
    } else {
        method.markers.as_slice()
    };
    Some(prefix_method_annotations(
        block_member(declaration, Placed::Block(body), markers),
        &method.annotations,
    ))
}

/// The capture certificate's exact six-instruction template has no source-level constructor
/// work beyond the direct zero-argument super call. Its implicit first parameter and field write
/// are omitted together; ordinary parameters and their annotations retain descriptor positions.
pub(crate) fn member_family_constructor_text(
    method: &ClassSourceMethod,
    simple_name: &str,
) -> Option<String> {
    if method.item.identity.name.0 != b"<init>"
        || method.declaration.is_none()
        || !method.declaration.as_ref()?.ends_with(')')
        || !method.markers.is_empty()
        || !method.annotations.refusals.is_empty()
        || !method.parameter_annotations.refusals.is_empty()
        || !method.type_annotations.refusals.is_empty()
        || !method.type_annotations.field_uses.is_empty()
        || !method.type_annotations.return_uses.is_empty()
        || method.item.access_flags & !(ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED) != 0
    {
        return None;
    }
    let signature = method_descriptor(&method.item.identity.descriptor.0, false, false)?;
    if signature.returns.is_some() || signature.parameters.is_empty() {
        return None;
    }
    if method.parameter_annotations.uses_by_position.len() != signature.parameters.len()
        || method.type_annotations.parameter_uses.len() != signature.parameters.len()
    {
        return None;
    }
    if method
        .parameter_annotations
        .uses_by_position
        .first()
        .is_some_and(|uses| !uses.is_empty())
        || method
            .type_annotations
            .parameter_uses
            .first()
            .is_some_and(|uses| !uses.is_empty())
    {
        return None;
    }
    let mut declaration = String::new();
    if let Some(word) = visibility(method.item.access_flags) {
        declaration.push_str(word);
        declaration.push(' ');
    }
    declaration.push_str(simple_name);
    declaration.push('(');
    for (ordinary_index, (ty, _)) in signature.parameters.iter().skip(1).enumerate() {
        if ordinary_index != 0 {
            declaration.push_str(", ");
        }
        let position = ordinary_index + 1;
        for annotation in method
            .parameter_annotations
            .uses_by_position
            .get(position)
            .into_iter()
            .flatten()
        {
            declaration.push_str(annotation);
            declaration.push(' ');
        }
        let type_uses = method
            .type_annotations
            .parameter_uses
            .get(position)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let ty = if type_uses.is_empty() {
            ty.clone()
        } else {
            decorate_qualified_type(ty, type_uses)?
        };
        declaration.push_str(&ty);
        declaration.push_str(&format!(" arg{ordinary_index}"));
    }
    declaration.push(')');
    let text = format!("    {declaration} {{\n        super();\n        return;\n    }}\n");
    Some(prefix_method_annotations(text, &method.annotations))
}

pub(crate) fn member_family_source_text(
    root: &ClassSourceReport,
    member: &MemberFamilyTextProjection<'_>,
) -> Option<(String, Vec<MemberFamilyDerivedProjection>)> {
    let declaration = root.declaration.as_ref()?;
    let child_declaration = member.child.declaration.as_ref()?;
    let capture_field = member
        .child
        .fields
        .iter()
        .find(|field| field.item.index == member.capture.field_index)?;
    if root.class != member.relation.root
        || member.child.class != member.relation.child
        || member.capture.constructor.owner != member.child.class
        // A regenerated capture field cannot preserve extra physical modifiers.
        || capture_field.item.access_flags != (ACC_FINAL | 0x1000)
        || !capture_field.annotations.attributes.is_empty()
        || !capture_field.type_annotations.attributes.is_empty()
        || !is_java_identifier(&member.relation.simple_name)
        || child_declaration.generic_signature.is_some()
        || child_declaration.generic_refusal.is_some()
        || !child_declaration.annotation_refusals.is_empty()
        || member.relation.access_flags & ACC_STATIC != 0
        || child_declaration.item.declaration.access_flags
            & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION)
            != 0
        || member.child.fields.iter().any(|field| {
            field.item.index != member.capture.field_index && field.declaration.is_none()
        })
    {
        return None;
    }
    let context = ClassSourceTextContext {
        initializer_field_order: None,
        declared_methods: root.methods.len() as u64,
        member_table: None,
        execution: &root.execution,
        enum_projection: None,
        array_helper_indices: None,
        array_method_texts: None,
        array_helper_markers: None,
    };
    // Other class-level projections have their own writer inputs. This equality proves that the
    // narrow family writer can reproduce the physical root before adding the nested declaration.
    if source_text(declaration, &root.fields, &root.methods, &context) != root.text {
        return None;
    }
    let mut derived = Vec::new();
    let text = source_text_with_member(
        declaration,
        &root.fields,
        &root.methods,
        &context,
        Some(member),
        &mut derived,
    )?;
    let expected = 1
        + member
            .root_methods
            .iter()
            .map(|method| method.derived.len())
            .sum::<usize>()
        + member
            .child_methods
            .iter()
            .map(|method| method.derived.len())
            .sum::<usize>();
    if derived.len() != expected
        || derived.iter().any(|entry| {
            entry.start >= entry.end
                || text.get(entry.start..entry.end).is_none()
                || entry.anchors.is_empty()
        })
    {
        return None;
    }
    Some((text, derived))
}

fn source_text_with_member(
    declaration: &ClassSourceDeclaration,
    fields: &[ClassSourceField],
    methods: &[ClassSourceMethod],
    context: &ClassSourceTextContext<'_>,
    member_family: Option<&MemberFamilyTextProjection<'_>>,
    derived: &mut Vec<MemberFamilyDerivedProjection>,
) -> Option<String> {
    let initializer_field_order = context.initializer_field_order;
    let declared_methods = context.declared_methods;
    let member_table = context.member_table;
    let execution = context.execution;
    let enum_projection = context.enum_projection;
    let array_helper_indices = context.array_helper_indices;
    let array_method_texts = context.array_method_texts;
    let array_helper_markers = context.array_helper_markers;
    let mut out = String::new();
    out.push_str(&format!(
        "// jarde: presentation of `{}` from the class file's own declaration and one recovery run per member.\n",
        comment_name(&declaration.item.declaration.this_class)
    ));
    out.push_str(
        "// jarde: not a compilable project: no imports and no resources are claimed (the `package` \
         line is the class file's own name, not a claim about a directory); every place this text is \
         not a full recovery carries a marker of this prefix.\n",
    );
    if let Some(package) = package_name(&declaration.item.declaration.this_class.raw().0) {
        out.push_str(&format!("package {package};\n\n"));
    }
    for use_line in &declaration.annotation_uses {
        out.push_str(use_line);
        out.push('\n');
    }
    for refusal in &declaration.annotation_refusals {
        out.push_str("// jarde: class annotation refused: ");
        out.push_str(refusal);
        out.push('\n');
    }
    if !declaration.annotation_uses.is_empty() || !declaration.annotation_refusals.is_empty() {
        out.push('\n');
    }
    if let Some(signature) = &declaration.generic_signature {
        out.push_str(&format!(
            "// jarde: class Signature `{}` {}\n",
            comment_text(&String::from_utf8_lossy(&signature.0)),
            if declaration.generic_refusal.is_some() {
                "not projected"
            } else {
                "projected after physical parent erasure proof"
            },
        ));
    }
    if let Some(refusal) = &declaration.generic_refusal {
        out.push_str("// jarde: class Signature projection refused: ");
        out.push_str(&comment_text(refusal));
        out.push('\n');
    }
    out.push_str(&declaration.declaration);
    out.push_str(" {\n");
    if let Some(markers) = array_helper_markers {
        for marker in markers {
            out.push_str(&indent(&format!("{marker}\n"), 1));
        }
    }
    let mut first = true;
    if let Some(projection) = enum_projection {
        out.push_str(&projection.constants_text);
        first = false;
    }
    let physical_order;
    let field_order = if let Some(order) = initializer_field_order {
        order
    } else {
        physical_order = (0..fields.len()).collect::<Vec<_>>();
        &physical_order
    };
    for &field_index in field_order {
        let field = &fields[field_index];
        if enum_projection.is_some_and(|projection| {
            field.item.index == projection.backing_field_index
                || projection
                    .constant_field_indices
                    .contains(&field.item.index)
        }) {
            continue;
        }
        if !first {
            out.push('\n');
        }
        first = false;
        for annotation in &field.annotations.uses {
            out.push_str(&indent(&format!("{annotation}\n"), 1));
        }
        match (&field.declaration, field.markers.is_empty()) {
            (Some(declaration), _) => {
                out.push_str(&declaration_member(declaration, &field.markers))
            }
            (None, _) => out.push_str(&comment_member(&field.markers)),
        }
    }
    if let Some(projection) = enum_projection
        && let Some(initializer_text) = &projection.initializer_text
    {
        if !first {
            out.push('\n');
        }
        first = false;
        out.push_str(initializer_text);
    }
    for method in methods {
        if array_helper_indices.is_some_and(|indices| indices.contains(&method.item.index)) {
            continue;
        }
        if let Some(projection) = enum_projection {
            if let Some((_, constructor_text)) = projection
                .constructor_texts
                .iter()
                .find(|(index, _)| *index == method.item.index)
            {
                if !first {
                    out.push('\n');
                }
                first = false;
                out.push_str(constructor_text);
                continue;
            }
            if projection
                .implicit_method_indices
                .contains(&method.item.index)
            {
                continue;
            }
        }
        if (initializer_field_order.is_some() || enum_projection.is_some())
            && method.item.identity.name.0 == b"<clinit>"
            && method.item.identity.descriptor.0 == b"()V"
        {
            continue;
        }
        if !first {
            out.push('\n');
        }
        first = false;
        if let Some(projected) = member_family.and_then(|member| {
            member
                .root_methods
                .iter()
                .find(|projected| projected.index == method.item.index)
        }) {
            append_family_method(&mut out, projected, false, derived)?;
        } else if let Some((_, projected)) = array_method_texts
            .and_then(|texts| texts.iter().find(|(index, _)| *index == method.item.index))
        {
            out.push_str(projected);
        } else {
            out.push_str(&method.text);
        }
    }
    let presented = u64::try_from(methods.len()).unwrap_or(u64::MAX);
    let mut trailing = Vec::new();
    match member_table {
        // The walk stopped inside a member record: what the members behind it are is not read, so no
        // count of them is stated here — the stop's own code and position are what the bytes say.
        Some(stop) => trailing.push(format!(
            "// jarde: the class file's member table stopped at {}[{}] ({}); the members above are \
             the reliable prefix and this presentation lists no others",
            table_name(stop),
            stop.index,
            comment_text(&stop.code)
        )),
        None if presented < declared_methods => trailing.push(format!(
            "// jarde: the class file declares {declared_methods} method record(s) and {presented} \
             were presented: the request stopped before the rest ({})",
            stop_code(execution).unwrap_or_else(|| "reason not stated".to_owned())
        )),
        None => {}
    }
    if !trailing.is_empty() {
        out.push('\n');
        for marker in trailing {
            out.push_str(&indent(&format!("{marker}\n"), 1));
        }
    }
    if let Some(member) = member_family {
        if !first {
            out.push('\n');
        }
        let offset = out.len();
        let (child_text, child_derived) = render_member_class(member)?;
        out.push_str(&child_text);
        derived.extend(child_derived.into_iter().map(|mut entry| {
            entry.start += offset;
            entry.end += offset;
            entry
        }));
    }
    out.push_str("}\n");
    Some(out)
}

fn render_member_class(
    member: &MemberFamilyTextProjection<'_>,
) -> Option<(String, Vec<MemberFamilyDerivedProjection>)> {
    let child = member.child;
    let declaration = child
        .declaration
        .as_ref()
        .expect("validated child declaration");
    let mut facts = declaration.item.declaration.clone();
    const SOURCE_CLASS_FLAGS: u16 = ACC_PUBLIC
        | ACC_PRIVATE
        | ACC_PROTECTED
        | ACC_ABSTRACT
        | ACC_FINAL
        | ACC_STRICT
        | ACC_STATIC;
    facts.access_flags = (facts.access_flags & !SOURCE_CLASS_FLAGS)
        | (member.relation.access_flags & SOURCE_CLASS_FLAGS);
    let mut out = String::new();
    let mut derived = Vec::new();
    for annotation in &declaration.annotation_uses {
        out.push_str(&indent(&format!("{annotation}\n"), 1));
    }
    let class_header = indent(
        &format!(
            "{} {{\n",
            class_declaration(&member.relation.simple_name, &facts)
        ),
        1,
    );
    let header_start = out.len();
    out.push_str(&class_header);
    let capture_field = child
        .fields
        .iter()
        .find(|field| field.item.index == member.capture.field_index)
        .expect("validated capture field");
    derived.push(MemberFamilyDerivedProjection {
        kind: MemberFamilyDerivedKind::HiddenCaptureField,
        start: header_start,
        end: out.len() - 1,
        anchors: vec![MemberFamilyPhysicalAnchor::Field {
            field: capture_field.item.identity.clone(),
            index: capture_field.item.index,
        }],
    });
    let mut first = true;
    for field in &child.fields {
        if field.item.index == member.capture.field_index {
            continue;
        }
        if !first {
            out.push('\n');
        }
        first = false;
        for annotation in &field.annotations.uses {
            out.push_str(&indent(&format!("{annotation}\n"), 2));
        }
        let text = match &field.declaration {
            Some(declaration) => declaration_member(declaration, &field.markers),
            None => comment_member(&field.markers),
        };
        out.push_str(&indent(&text, 1));
    }
    for method in &child.methods {
        if !first {
            out.push('\n');
        }
        first = false;
        if let Some(projected) = member
            .child_methods
            .iter()
            .find(|projected| projected.index == method.item.index)
        {
            append_family_method(&mut out, projected, true, &mut derived)?;
        } else {
            out.push_str(&indent(&method.text, 1));
        }
    }
    out.push_str("    }\n");
    Some((out, derived))
}

fn append_family_method(
    out: &mut String,
    method: &MemberFamilyMethodText,
    nested: bool,
    derived: &mut Vec<MemberFamilyDerivedProjection>,
) -> Option<()> {
    let offset = out.len();
    if nested {
        out.push_str(&indent(&method.text, 1));
    } else {
        out.push_str(&method.text);
    }
    for entry in &method.derived {
        let mut entry = entry.clone();
        if nested {
            let source_span = method.text.get(entry.start..entry.end)?;
            let start = indented_offset(&method.text, entry.start, 1, false)?;
            let end = indented_offset(&method.text, entry.end, 1, true)?;
            entry.start = offset + start;
            entry.end = offset + end;
            if out.get(entry.start..entry.end) != Some(source_span) {
                return None;
            }
        } else {
            entry.start += offset;
            entry.end += offset;
        }
        derived.push(entry);
    }
    Some(())
}

/// The name of the member table one stop happened in.
fn table_name(stop: &MemberTableStop) -> &'static str {
    match stop.phase {
        crate::MemberTablePhase::Fields => "fields",
        crate::MemberTablePhase::Methods => "methods",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEMBER_ANNOTATION_TARGET: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-22/member-annotation-uses/generated/original/MemberTagged.class"
    );

    fn unlimited_annotation_test_limits() -> Limits {
        Limits {
            input_bytes: u64::MAX,
            archive_entries: u64::MAX,
            entry_bytes: u64::MAX,
            read_bytes: u64::MAX,
            class_bytes: u64::MAX,
            attribute_bytes: u64::MAX,
            code_bytes: u64::MAX,
            result_items: u64::MAX,
            output_bytes: u64::MAX,
            class_headers: u64::MAX,
            method_bodies: u64::MAX,
            ir_items: u64::MAX,
            ir_edges: u64::MAX,
            analysis_steps: u64::MAX,
            normalization_clones: u64::MAX,
            nested_depth: u64::MAX,
            dependency_depth: u64::MAX,
            elapsed_millis: u64::MAX,
        }
    }

    #[test]
    fn anonymous_capture_class_facts_are_passed_as_typed_assembly_input() {
        const ANONYMOUS: &[u8] = include_bytes!(
            "../tests/fixtures/proved-java-structure/anonymous-capture/AnonymousCaptureCases$1.class"
        );
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let selected = jarde_reader::classfile::class_facts(ANONYMOUS, &mut budget)
            .expect("the committed Java 8 anonymous class is structurally readable");
        let shells: Vec<_> = selected
            .attributes
            .iter()
            .filter(|attribute| {
                matches!(
                    attribute.name.raw().0.as_slice(),
                    b"InnerClasses" | b"EnclosingMethod"
                )
            })
            .cloned()
            .collect();
        let assembly = read_class_source_assembly_context(
            ANONYMOUS,
            &shells,
            &selected.constant_pool,
            &mut budget,
        )
        .expect("the selected class's nesting attributes decode into typed facts");

        assert!(assembly.inner_classes.iter().any(|inner| {
            jarde_reader::classfile::cp_class_name(&selected.constant_pool, inner.class_index)
                .is_ok_and(|name| name.0 == b"AnonymousCaptureCases$1")
                && inner.inner_name.is_none()
        }));
        let enclosing = assembly
            .enclosing_method
            .expect("anonymous local class has EnclosingMethod");
        assert_eq!(
            jarde_reader::classfile::cp_class_name(&selected.constant_pool, enclosing.class_index)
                .expect("enclosing class index resolves")
                .0,
            b"AnonymousCaptureCases"
        );
        let method = selected
            .constant_pool
            .iter()
            .find(|entry| entry.index == enclosing.method_index)
            .expect("enclosing method index resolves");
        let CpEntryKind::NameAndType {
            name, descriptor, ..
        } = &method.kind
        else {
            panic!("EnclosingMethod points to a NameAndType entry");
        };
        assert_eq!(name.0, b"baseArgumentAndCapture");
        assert_eq!(descriptor.0, b"()LAnonymousCaptureCases$Renderer;");
    }

    #[test]
    fn ordinary_signature_types_spell_each_structured_argument_and_array() {
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let parsed = parse_method_signature(
            b"(Ljava/util/List<*>;Ljava/util/List<+Ljava/lang/Number;>;Ljava/util/List<-Ljava/lang/String;>;Ljava/util/Map<Ljava/lang/String;Ljava/util/List<Ljava/lang/Integer;>;>;[Ljava/util/List<Ljava/lang/String;>;)V",
            &mut budget,
        ).unwrap();
        let written = parsed
            .parameters
            .iter()
            .map(|ty| spell_ordinary_signature_type(ty, &[], &mut budget, 0).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            written,
            [
                "java.util.List<?>",
                "java.util.List<? extends java.lang.Number>",
                "java.util.List<? super java.lang.String>",
                "java.util.Map<java.lang.String, java.util.List<java.lang.Integer>>",
                "java.util.List<java.lang.String>[]",
            ]
        );
        let nested = parse_method_signature(b"(Lp/Outer.Inner;)V", &mut budget).unwrap();
        assert!(matches!(
            spell_ordinary_signature_type(&nested.parameters[0], &[], &mut budget, 0),
            Err(Error::Unsupported { .. })
        ));
    }

    #[test]
    fn selected_member_path_spells_only_exact_nested_signature_segments() {
        let identity = jarde_reader::model::PhysicalDefinitionId {
            location: jarde_reader::model::PhysicalClassLocation::StandaloneRoot {
                snapshot: jarde_reader::model::SnapshotId("member-path-test".to_owned()),
            },
            class_bytes: jarde_reader::model::ClassBytesId {
                digest: jarde_reader::model::Digest("member-path".to_owned()),
                length: 0,
            },
            variant: jarde_reader::model::PhysicalVariant::Base,
        };
        let path = vec![
            jarde_java::report::ProvedMemberInnerSourceSegment {
                definition: identity.clone(),
                binary_name: "matrix/Outer".to_owned(),
                source_name: "matrix.Outer".to_owned(),
                type_parameter_count: 0,
                enclosing_binary_name: None,
                is_static: true,
            },
            jarde_java::report::ProvedMemberInnerSourceSegment {
                definition: identity.clone(),
                binary_name: "matrix/Outer$A".to_owned(),
                source_name: "matrix.Outer.A".to_owned(),
                type_parameter_count: 1,
                enclosing_binary_name: Some("matrix/Outer".to_owned()),
                is_static: true,
            },
            jarde_java::report::ProvedMemberInnerSourceSegment {
                definition: identity,
                binary_name: "matrix/Outer$A$Generic".to_owned(),
                source_name: "matrix.Outer.A.Generic".to_owned(),
                type_parameter_count: 1,
                enclosing_binary_name: Some("matrix/Outer$A".to_owned()),
                is_static: false,
            },
        ];
        let mut budget = Budget::new(unlimited_annotation_test_limits());

        let raw =
            parse_method_signature(b"(Lmatrix/Outer$A;)Lmatrix/Outer$A;", &mut budget).unwrap();
        assert_eq!(
            spell_ordinary_signature_type_with_member_path(
                &raw.parameters[0],
                &[],
                &path,
                &mut budget,
                0,
            )
            .unwrap(),
            "matrix.Outer.A"
        );

        let parameterized = parse_method_signature(
            b"(Lmatrix/Outer$A<Ljava/lang/String;>;)Lmatrix/Outer$A<Ljava/lang/String;>.Generic<Ljava/lang/Integer;>;",
            &mut budget,
        )
        .unwrap();
        assert_eq!(
            spell_ordinary_signature_type_with_member_path(
                &parameterized.parameters[0],
                &[],
                &path,
                &mut budget,
                0,
            )
            .unwrap(),
            "matrix.Outer.A<java.lang.String>"
        );
        assert_eq!(
            spell_ordinary_signature_type_with_member_path(
                parameterized.result.as_ref().unwrap(),
                &[],
                &path,
                &mut budget,
                0,
            )
            .unwrap(),
            "matrix.Outer.A<java.lang.String>.Generic<java.lang.Integer>"
        );

        let wrong_arity = parse_method_signature(
            b"(Lmatrix/Outer$A<Ljava/lang/String;Ljava/lang/Integer;>;)V",
            &mut budget,
        )
        .unwrap();
        assert!(matches!(
            spell_ordinary_signature_type_with_member_path(
                &wrong_arity.parameters[0],
                &[],
                &path,
                &mut budget,
                0,
            ),
            Err(Error::Unsupported { .. })
        ));

        let omitted_enclosing = parse_method_signature(
            b"(Lmatrix/Outer$A$Generic<Ljava/lang/Integer;>;)V",
            &mut budget,
        )
        .unwrap();
        assert!(matches!(
            spell_ordinary_signature_type_with_member_path(
                &omitted_enclosing.parameters[0],
                &[],
                &path,
                &mut budget,
                0,
            ),
            Err(Error::Unsupported { .. })
        ));

        let legal_dollar_name =
            parse_method_signature(b"(Lmatrix/Dollar$Type;)V", &mut budget).unwrap();
        assert!(matches!(
            spell_ordinary_signature_type_with_member_path(
                &legal_dollar_name.parameters[0],
                &[],
                &path,
                &mut budget,
                0,
            ),
            Err(Error::Unsupported { .. })
        ));
    }

    /// The shape [`jarde_java::recover`] emits, so the split is tested against the text it really
    /// produces rather than against a convenient one.
    const ARTIFACT: &str = "// @method run()V\n// @declaration an instance method of `p/T`\n\
                            // recovered from bytecode; presentation is not claimed to compile\n\
                            {\n    if (arg0) {\n        return;\n    }\n}\n";

    #[test]
    fn one_field_descriptor_is_spelled_as_java_spells_it() {
        for (descriptor, expected) in [
            (&b"I"[..], "int"),
            (b"Z", "boolean"),
            (b"J", "long"),
            (b"Ljava/lang/String;", "java.lang.String"),
            (b"[I", "int[]"),
            (b"[[J", "long[][]"),
            (b"[Ljava/util/List;", "java.util.List[]"),
        ] {
            assert_eq!(
                field_type(descriptor).as_deref(),
                Some(expected),
                "descriptor {}",
                String::from_utf8_lossy(descriptor)
            );
        }
    }

    #[test]
    fn bytes_that_are_not_a_field_descriptor_are_refused_rather_than_guessed() {
        for descriptor in [
            &b""[..],
            b"V",
            b"II",
            b"Il",
            b"Ljava/lang/String",
            b"[Z]",
            b"L;",
            b"(",
            b"not-a-descriptor",
        ] {
            assert_eq!(
                field_type(descriptor),
                None,
                "descriptor {}",
                String::from_utf8_lossy(descriptor)
            );
        }
    }

    /// P3 6.7: the third argument is the member's own `ACC_VARARGS`, and none of the descriptors
    /// below declares one — the flag is stated by the test after this one, because it is the only
    /// part of the reading it changes.
    #[test]
    fn one_method_descriptor_is_spelled_with_its_slots_and_its_return_type() {
        let signature = method_descriptor(b"(JLjava/lang/String;[I)V", true, false)
            .expect("a method descriptor");
        assert_eq!(
            signature.parameters,
            vec![
                ("long".to_owned(), 0),
                ("java.lang.String".to_owned(), 2),
                ("int[]".to_owned(), 3),
            ],
            "a wide parameter occupies two slots, and the next one starts after it"
        );
        assert!(!signature.varargs, "no `ACC_VARARGS` is set on this member");
        assert_eq!(signature.returns, None, "`V` states no return type");
        assert_eq!(signature.slots, 4);

        let empty =
            method_descriptor(b"()[Ljava/lang/Object;", true, false).expect("a method descriptor");
        assert!(empty.parameters.is_empty());
        assert!(
            !empty.varargs,
            "there is no parameter for the flag to reach"
        );
        assert_eq!(empty.returns.as_deref(), Some("java.lang.Object[]"));
        assert_eq!(empty.slots, 0);
    }

    /// P3 6.7: `ACC_VARARGS` (0x0080) is the member's own fact that its **last** parameter is the
    /// variable-arity one, and the two facts it takes to write a `...` meet in the signature: the
    /// flag, and a descriptor whose last component is an array. The reading of the descriptor itself
    /// does not move — the types stay the arrays the bytes state — and the dots are what the
    /// parameter list writes for the one position the flag reaches.
    #[test]
    fn a_varargs_flag_writes_the_last_parameters_array_as_the_dots() {
        let ints = method_descriptor(b"(I[I)V", true, true).expect("a method descriptor");
        assert_eq!(
            ints.parameters,
            vec![("int".to_owned(), 0), ("int[]".to_owned(), 1)],
            "the descriptor's own types are read the same way with and without the flag"
        );
        assert!(ints.varargs);
        assert_eq!(arguments(None, &ints, &[], &[]), "(int arg0, int... arg1)");

        // `[[B`: the dots take the place of the **last** `[]`, so the element type keeps the `[]`
        // it has and the parameter is written `byte[]...`, never `byte[][]` or `byte...`.
        let grid = method_descriptor(b"([[B)V", true, true).expect("a method descriptor");
        assert_eq!(arguments(None, &grid, &[], &[]), "(byte[]... arg0)");

        // The controls. The same descriptor without the flag keeps the array spelling, and the flag
        // on a member whose last parameter is not an array is written as the descriptor states it.
        let without_the_flag =
            method_descriptor(b"(I[I)V", true, false).expect("a method descriptor");
        assert!(!without_the_flag.varargs);
        assert_eq!(
            arguments(None, &without_the_flag, &[], &[]),
            "(int arg0, int[] arg1)"
        );
        let not_an_array = method_descriptor(b"(II)V", true, true).expect("a method descriptor");
        assert!(!not_an_array.varargs, "no `...` is invented for an `int`");
        assert_eq!(
            arguments(None, &not_an_array, &[], &[]),
            "(int arg0, int arg1)"
        );
        let no_parameters = method_descriptor(b"()V", true, true).expect("a method descriptor");
        assert!(
            !no_parameters.varargs,
            "there is no last parameter to reach"
        );
        assert_eq!(arguments(None, &no_parameters, &[], &[]), "()");

        // The flag reaches the descriptor's last parameter and not the slot the receiver holds: an
        // instance member's `this` is no position of the list, so the dots land on slot 1 here.
        let instance = method_descriptor(b"([I)V", false, true).expect("a method descriptor");
        assert_eq!(arguments(None, &instance, &[], &[]), "(int... arg1)");
    }

    /// JVMS 2.6.1: an array fills **one** slot whatever its element type, and a member that is not
    /// `static` puts its receiver in slot 0 — the two facts the parameter names of a declaration are
    /// looked up by, and the two the body's own `arg<slot>` names are numbered by.
    #[test]
    fn an_arrays_element_width_does_not_move_the_parameters_after_it() {
        let static_mixed = method_descriptor(b"([JI)V", true, false).expect("a method descriptor");
        assert_eq!(
            static_mixed.parameters,
            vec![("long[]".to_owned(), 0), ("int".to_owned(), 1)],
            "`long[]` is a reference: the `int` after it is at slot 1, not at slot 2"
        );
        assert_eq!(static_mixed.slots, 2);

        let instance_mixed =
            method_descriptor(b"([JI)V", false, false).expect("a method descriptor");
        assert_eq!(
            instance_mixed.parameters,
            vec![("long[]".to_owned(), 1), ("int".to_owned(), 2)],
            "slot 0 holds the receiver of a member that is not static"
        );
        assert_eq!(instance_mixed.slots, 3);

        let double_grid = method_descriptor(b"([[DJ)J", true, false).expect("a method descriptor");
        assert_eq!(
            double_grid.parameters,
            vec![("double[][]".to_owned(), 0), ("long".to_owned(), 1)],
            "a two-dimensional `double[][]` is one slot, and the `long` after it two"
        );
        assert_eq!(double_grid.slots, 3);

        // The controls: a `double` written on its own really is two slots, and an `int[]` (whose
        // element is one slot wide) reads the same way before and after this rule.
        let wide = method_descriptor(b"(DI)J", true, false).expect("a method descriptor");
        assert_eq!(
            wide.parameters,
            vec![("double".to_owned(), 0), ("int".to_owned(), 2)]
        );
        assert_eq!(wide.slots, 3);
        assert_eq!(wide.returns.as_deref(), Some("long"));
        let narrow_array = method_descriptor(b"([II)V", true, false).expect("a method descriptor");
        assert_eq!(
            narrow_array.parameters,
            vec![("int[]".to_owned(), 0), ("int".to_owned(), 1)]
        );
    }

    #[test]
    fn bytes_that_are_not_a_method_descriptor_are_refused_rather_than_guessed() {
        for descriptor in [
            &b""[..],
            b"I",
            b"()",
            b"()VV",
            b"(I",
            b"(I)Z ",
            b"not-a-descriptor",
            // An object name Java cannot write is no type: a type position states a Java type or
            // states that it could not be written, and a lossy spelling is neither.
            b"(L\xff;)V",
        ] {
            assert_eq!(
                // P3 6.7: the varargs flag is stated **beside** the descriptor the parameter list is
                // read from, so a descriptor that is not one is refused before a flag can matter.
                method_descriptor(descriptor, true, false),
                None,
                "descriptor {}",
                String::from_utf8_lossy(descriptor)
            );
            assert!(!spellable_descriptor(descriptor));
        }
        assert!(spellable_descriptor(b"()V"));
        assert!(spellable_descriptor(b"(JLjava/lang/String;[I)V"));
        assert!(spellable_descriptor(b"([JI)V"));
    }

    #[test]
    fn one_internal_name_is_spelled_dotted_and_nothing_else_about_it_changes() {
        assert_eq!(class_name(b"p/Outer$Inner"), "p.Outer$Inner");
        assert_eq!(
            class_name(b"HistoricalControlFlow"),
            "HistoricalControlFlow"
        );
        assert_eq!(class_name(b"a/b/C"), "a.b.C");
    }

    #[test]
    fn a_binary_name_is_split_where_its_own_last_slash_is() {
        // The class's own name: the package is everything before the **last** `/`, and a `$` inside
        // either part stays where it is — no nesting relation is read out of it.
        assert_eq!(simple_name(b"a/b/C"), "C");
        assert_eq!(simple_name(b"p/Outer$Inner"), "Outer$Inner");
        assert_eq!(simple_name(b"p/More17$1"), "More17$1");
        assert_eq!(simple_name(b"C"), "C");

        // The package line the same split states; a name with no `/` is the default package, which
        // writes no line at all.
        assert_eq!(package_name(b"a/b/C"), Some("a.b".to_owned()));
        assert_eq!(package_name(b"p/Outer$Inner"), Some("p".to_owned()));
        assert_eq!(package_name(b"C"), None);
    }

    #[test]
    fn a_name_java_cannot_spell_becomes_the_deterministic_alias() {
        assert_eq!(written_name(b"run"), ("run".to_owned(), false));
        assert_eq!(written_name(b"int"), ("int_".to_owned(), true));
        assert_eq!(written_name(b"weird-name"), ("weird_name".to_owned(), true));
        assert_eq!(
            written_name(b"weird-name"),
            written_name(b"weird-name"),
            "the alias is a function of the raw name alone"
        );
    }

    #[test]
    fn one_artifact_is_split_at_its_own_block() {
        let artifact = artifact(ARTIFACT).expect("the emitter's own shape");
        assert_eq!(
            artifact.envelope,
            "// @method run()V\n// @declaration an instance method of `p/T`\n\
             // recovered from bytecode; presentation is not claimed to compile\n"
        );
        assert_eq!(
            artifact.statements, "    if (arg0) {\n        return;\n    }\n",
            "the statements keep the one level of indentation their own block gave them"
        );
    }

    #[test]
    fn a_text_that_is_not_that_shape_is_not_placed() {
        assert!(artifact("").is_none());
        assert!(artifact("// only comments\n").is_none());
        assert!(artifact("{\n    return;\n}\ntrailing\n").is_none());
        assert!(artifact("}\n{\n").is_none());
        assert!(
            artifact("{\n    return;\n}\n").is_some(),
            "an empty envelope is still the shape"
        );
    }

    #[test]
    fn one_member_block_puts_the_envelope_and_the_statements_at_the_same_depth() {
        let artifact = artifact(ARTIFACT).expect("the emitter's own shape");
        let member = block_member(
            "public void run()",
            Placed::Block(artifact),
            &["// jarde: a marker".to_owned()],
        );
        assert_eq!(
            member,
            "    public void run() {\n\
             \x20       // jarde: a marker\n\
             \x20       // @method run()V\n\
             \x20       // @declaration an instance method of `p/T`\n\
             \x20       // recovered from bytecode; presentation is not claimed to compile\n\
             \x20       if (arg0) {\n\
             \x20           return;\n\
             \x20       }\n\
             \x20   }\n"
        );
    }

    #[test]
    fn one_member_without_an_artifact_keeps_its_marker_inside_the_block() {
        let member = block_member(
            "public void run()",
            Placed::Without,
            &["// jarde: no body".to_owned()],
        );
        assert_eq!(
            member,
            "    public void run() {\n        // jarde: no body\n    }\n"
        );
    }

    #[test]
    fn one_member_whose_artifact_cannot_be_placed_quotes_it_line_by_line() {
        let member = block_member(
            "public void run()",
            Placed::Quoted("not a block\n"),
            &["// jarde: not placed".to_owned()],
        );
        assert_eq!(
            member,
            "    public void run() {\n        // jarde: not placed\n        // not a block\n    }\n"
        );
    }

    #[test]
    fn one_stop_code_is_the_engines_own_spelling_of_its_reason() {
        let cancelled = ExecutionReport::Cancelled {
            usage: UsageSnapshot::default(),
        };
        assert_eq!(stop_code(&cancelled).as_deref(), Some("cancelled"));
        let budget = ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: crate::BudgetDimension::MethodBodies,
            },
            usage: UsageSnapshot::default(),
        };
        assert_eq!(
            stop_code(&budget).as_deref(),
            Some("budget_exceeded_method_bodies")
        );
        let complete = ExecutionReport::Complete {
            usage: UsageSnapshot::default(),
        };
        assert_eq!(stop_code(&complete), None);
    }

    #[test]
    fn member_annotation_budget_stop_keeps_the_raw_shell_unread() {
        let mut discovery = Budget::new(unlimited_annotation_test_limits());
        let class = jarde_reader::classfile::class_facts(MEMBER_ANNOTATION_TARGET, &mut discovery)
            .expect("the frozen class structure");
        let field = class
            .fields
            .iter()
            .find(|field| field.name.raw().0 == b"field")
            .expect("the annotated field");
        let mut limits = unlimited_annotation_test_limits();
        limits.attribute_bytes = 0;
        let mut budget = Budget::new(limits);

        let read = declared_member_annotations(
            MEMBER_ANNOTATION_TARGET,
            field,
            &class.constant_pool,
            &mut budget,
        );

        assert_eq!(read.facts.declaration.attributes.len(), 1);
        assert_eq!(
            read.facts.declaration.attributes[0].attribute.name.raw().0,
            b"RuntimeVisibleAnnotations"
        );
        assert!(read.facts.declaration.attributes[0].annotations.is_empty());
        assert!(read.facts.declaration.refusals[0].contains("budget exceeded for AttributeBytes"));
        assert!(matches!(
            read.errors.as_slice(),
            [Error::BudgetExceeded {
                dimension: crate::BudgetDimension::AttributeBytes,
                ..
            }]
        ));
    }

    #[test]
    fn member_annotation_cancellation_keeps_the_raw_shell_unread() {
        let mut discovery = Budget::new(unlimited_annotation_test_limits());
        let class = jarde_reader::classfile::class_facts(MEMBER_ANNOTATION_TARGET, &mut discovery)
            .expect("the frozen class structure");
        let field = class
            .fields
            .iter()
            .find(|field| field.name.raw().0 == b"field")
            .expect("the annotated field");
        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut budget =
            Budget::with_cancellation_token(unlimited_annotation_test_limits(), cancellation);

        let read = declared_member_annotations(
            MEMBER_ANNOTATION_TARGET,
            field,
            &class.constant_pool,
            &mut budget,
        );

        assert_eq!(read.facts.declaration.attributes.len(), 1);
        assert!(read.facts.declaration.attributes[0].annotations.is_empty());
        assert!(read.facts.declaration.refusals[0].contains("cancelled:"));
        assert!(matches!(read.errors.as_slice(), [Error::Cancelled { .. }]));
    }
}
