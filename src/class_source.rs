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

#[cfg(test)]
use crate::JvmString;
use crate::facade::{ClassDeclarationItem, ClassRef, EnvironmentRequest, FieldItem, MethodItem};
use crate::{
    AnalysisStage, AttributeShell, Budget, Coverage, CpEntryFacts, Diagnostic, Error,
    ExecutionReport, JvmBytes, Limits, MemberHeader, MemberTableStop, NoBodyKind,
    PhysicalDefinitionId, PhysicalMethodId, PhysicalView, Result, TerminationReason, UsageSnapshot,
    attribute_facts, budget_dimension_code,
};
#[cfg(test)]
use jarde_java::declaration::method_arguments as arguments;
use jarde_java::declaration::method_descriptor;
pub(crate) use jarde_java::declaration::{
    ProvedMethodParameter, decorate_qualified_type,
    spellable_method_descriptor as spellable_descriptor, visibility,
};
#[cfg(test)]
use jarde_java::generic_source_proof::spell_ordinary_signature_type_with_member_path;
pub(crate) use jarde_java::generic_source_proof::{
    ClassHeaderParentPosition, ClassHeaderParentProof, SameClassBodyValueSite,
    SameClassConstructorReceiver, SameClassFieldReceiverSource, SameClassFieldUse,
    SameClassFieldWriteSource, SameClassGenericArgumentOrigin, SameClassGenericArgumentSource,
    SameClassGenericBodyReturnSource, SameClassGenericMethodBodyProof,
    SameClassGenericMethodContract, SameClassInvokeUse, SameClassMemberRef,
    SameClassOverloadCandidate, SameClassPublishedMethodParameters, SameClassUseFacts,
    binary_pool_class_name, class_scope_variable, is_object_instance_method_name,
    signature_class_erasure, spell_ordinary_signature_type,
};
#[cfg(test)]
use jarde_java::generic_source_proof::{
    ordinary_primitive_result_matches, overload_source_type_assignable,
    prove_same_class_generic_substitution, spell_class_header_signature_type,
    wildcard_array_return_matches,
};
use jarde_java::metadata_literal::{
    MemberDefault, TypeUseOwner, resolve_constant_value, spell_member_annotation_uses,
    spell_parameter_annotation_uses, spell_type_annotation_uses,
};
#[cfg(test)]
use jarde_java::names::legal_package_name;
use jarde_java::names::source_identifier;
pub(crate) use jarde_java::names::written_name;
use jarde_java::report::TypedFunctionalKind;
use jarde_java::report::{GenericConstructorCandidate, GenericReturnCandidate, GenericReturnValue};
pub(crate) use jarde_java::source_proof::EnumConstructorSourceTail;
#[cfg(test)]
use jarde_java::{NameTable, SlotEvidence};
use jarde_java::{
    RecoveryContent, RecoveryFacts, RecoveryReport, comment_text, is_java_identifier,
};
use jarde_reader::budget::CountedBudgetDimension;
use jarde_reader::classfile::{
    Base, ClassMemberFacts, CpEntryKind, DescriptorKind, ElementValueFacts, EnclosingMethodFacts,
    InnerClassFacts, TypeAnnotationFacts, descriptor_facts,
};
#[cfg(test)]
use jarde_reader::signature::TypeParameter;
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
    /// Requested output syntax; independent of the runtime environment.
    pub source_target: crate::SourceTarget,
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
    pub(crate) major_version: Option<u16>,
    pub(crate) inner_classes: Vec<InnerClassFacts>,
    pub(crate) enclosing_method: Option<EnclosingMethodFacts>,
    pub(crate) resolved_inner_classes: Vec<ResolvedInnerClass>,
    pub(crate) resolved_enclosing_method: Option<ResolvedEnclosingMethod>,
}

#[cfg(test)]
use jarde_java::names::class_simple_name as simple_name;
use jarde_java::names::{SourceClassNames, class_package_name as package_name};
pub(crate) use jarde_reader::classfile::ResolvedInnerClass;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedEnclosingMethod {
    pub(crate) class: Vec<u8>,
    pub(crate) method: Option<(JvmBytes, JvmBytes)>,
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
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(facts.inner_classes.len())
            .unwrap_or(u64::MAX)
            .saturating_add(u64::from(facts.enclosing_method.is_some())),
    )?;
    let resolved_inner_classes =
        jarde_reader::classfile::resolve_inner_classes(pool, &facts.inner_classes, budget)?;
    let resolved_enclosing_method = facts
        .enclosing_method
        .as_ref()
        .map(|method| {
            let class = jarde_reader::classfile::cp_class_name(pool, method.class_index)?.0;
            let member = if method.method_index == 0 {
                None
            } else {
                match &jarde_reader::classfile::cp_entry(pool, method.method_index)?.kind {
                    jarde_reader::classfile::CpEntryKind::NameAndType {
                        name, descriptor, ..
                    } => Some((name.clone(), descriptor.clone())),
                    _ => {
                        return Err(Error::invalid_input(
                            "classfile_constant_pool_tag_mismatch",
                            "EnclosingMethod method_index does not refer to a NameAndType constant",
                        ));
                    }
                }
            };
            Ok(ResolvedEnclosingMethod {
                class,
                method: member,
            })
        })
        .transpose()?;
    Ok(ClassSourceAssemblyContext {
        major_version: bytes
            .get(6..8)
            .map(|version| u16::from_be_bytes([version[0], version[1]])),
        inner_classes: facts.inner_classes,
        enclosing_method: facts.enclosing_method,
        resolved_inner_classes,
        resolved_enclosing_method,
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
    /// The type-parameter header text a **successful** projection spelled (`U`,
    /// `U, V`, `N extends java.lang.Number`). It is the one Signature projection the member
    /// fold's own nested header can carry verbatim; `None` with a `generic_signature` states a
    /// projection the fold does not have. Deliberately absent from the JSON view.
    #[serde(skip)]
    pub(crate) generic_type_parameters: Option<String>,
    /// The exact class Signature erasure scope that published this header. It is kept for the
    /// separately proved member-family pass and is deliberately absent from the JSON view.
    #[serde(skip)]
    pub(crate) generic_scope: Option<ClassSignatureErasureProof>,
    /// The `implements` entries whose **published header text** carries the interface's own type
    /// arguments, by binary name. This is the fact the bridge admission reads — the shared
    /// contract of `recover-bridge-admission-gates` and this projection: a header that carries
    /// the erased contract's type arguments is what lets a recompiled source regenerate the
    /// bridge, so a bridge whose contract owner is one of these entries may hide, and every
    /// other generic interface contract keeps its bridge visible. Empty for a header no
    /// projection published, and deliberately absent from the JSON view.
    #[serde(skip)]
    pub(crate) header_interface_arguments: Vec<Vec<u8>>,
}

pub use jarde_java::report::{
    ClassAnnotationAttribute, MemberAnnotationAttribute, MemberAnnotationUses,
    ParameterAnnotationAttribute, ParameterAnnotationUses, TypeAnnotationAttribute,
    TypeAnnotationUse, TypeAnnotationUses,
};

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
    /// Whether a field `Signature` has been accepted and published into `declaration`.
    ///
    /// This records the successful proof outcome directly; consumers must not infer it from the
    /// explanatory marker text. A later refusal does not clear the fact while the previously
    /// projected declaration remains published.
    pub generic_signature_projected: bool,
    /// The field's own declaration annotation attributes, their complete values, spellings, and
    /// any annotation-level refusal.
    pub annotations: MemberAnnotationUses,
    /// Type annotation attributes physically owned by this field.
    pub type_annotations: TypeAnnotationUses,
    /// Every `// jarde:` marker the text carries for this field, in the order it writes them, each
    /// one line: an unspellable raw name/descriptor, a refused `ConstantValue`, or the field
    /// `Signature` projection decision.
    /// Empty when no such explanation is needed.
    pub markers: Vec<String>,
}

/// A same-read, same-class field whose exact integer ConstantValue is already in its declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvedIntegerConstant {
    pub(crate) field: FieldItem,
    pub(crate) name: String,
    pub(crate) value: i32,
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
    /// Same-run source candidate retained for the final family writer only.
    #[serde(skip)]
    pub(crate) same_run_generic_return: Option<GenericReturnCandidate>,
    /// A complete same-run AST/SSA proof for the method body under its generic Signature. This is
    /// installed only inside the class transaction after every return, condition, throw and
    /// incoming call source has been joined to the physical method and formal slots.
    #[serde(skip)]
    pub(crate) same_class_generic_body_proof: Option<SameClassGenericMethodBodyProof>,
    /// The physical method's ordinary Signature attempt refused. Family assembly may replace
    /// that one marker on a cloned writer record only after its separate proof succeeds.
    #[serde(skip)]
    pub(crate) generic_signature_refused: bool,
    /// The staged generic declaration and body were charged and published together.
    #[serde(skip)]
    pub(crate) generic_signature_projected: bool,
    /// Closed source-tail shape parsed from this physical constructor's Signature.
    #[serde(skip)]
    pub(crate) enum_constructor_source_tail: EnumConstructorSourceTail,
    /// Whether the ordinary physical-descriptor projection refused that source tail only because
    /// the VM-injected enum name and ordinal make its descriptor longer.
    #[serde(skip)]
    pub(crate) enum_constructor_signature_erasure_refused: bool,
}

/// The fields changed while proving and rendering one generic-call member. Keeping this overlay
/// separate lets the class transaction stage a declaration without cloning its RecoveryReport.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct GenericCallProjectionState {
    pub(crate) declaration: Option<String>,
    pub(crate) text: String,
    pub(crate) markers: Vec<String>,
    pub(crate) same_class_generic_body_proof: Option<SameClassGenericMethodBodyProof>,
    pub(crate) generic_signature_refused: bool,
    pub(crate) generic_signature_projected: bool,
    pub(crate) enum_constructor_source_tail: EnumConstructorSourceTail,
    pub(crate) enum_constructor_signature_erasure_refused: bool,
}

impl ClassSourceMethod {
    /// Save only the mutable projection fields before the ordinary per-member Signature attempt.
    /// The RecoveryReport, annotations, and all other source facts remain in the original record.
    pub(crate) fn generic_call_projection_state(
        &self,
        budget: &mut Budget,
    ) -> Result<GenericCallProjectionState> {
        budget.poll()?;
        if self.same_class_generic_body_proof.is_some() {
            return Err(Error::unsupported(
                "generic_call_projection_state_unexpected_body_proof",
                "the pre-transaction source snapshot already carries a staged body proof",
            ));
        }
        let mut copied_bytes = u64::try_from(self.text.len()).unwrap_or(u64::MAX);
        let mut copied_steps = 8u64;
        if let Some(declaration) = &self.declaration {
            copied_bytes =
                copied_bytes.saturating_add(u64::try_from(declaration.len()).unwrap_or(u64::MAX));
        }
        copied_steps =
            copied_steps.saturating_add(u64::try_from(self.markers.len()).unwrap_or(u64::MAX));
        for marker in &self.markers {
            budget.poll()?;
            copied_bytes =
                copied_bytes.saturating_add(u64::try_from(marker.len()).unwrap_or(u64::MAX));
        }
        budget.charge(CountedBudgetDimension::AnalysisSteps, copied_steps)?;
        budget.charge(CountedBudgetDimension::OutputBytes, copied_bytes)?;
        Ok(GenericCallProjectionState {
            declaration: self.declaration.clone(),
            text: self.text.clone(),
            markers: self.markers.clone(),
            same_class_generic_body_proof: None,
            generic_signature_refused: self.generic_signature_refused,
            generic_signature_projected: self.generic_signature_projected,
            enum_constructor_source_tail: self.enum_constructor_source_tail,
            enum_constructor_signature_erasure_refused: self
                .enum_constructor_signature_erasure_refused,
        })
    }

    /// Move a completed projection overlay onto its physical method record.
    pub(crate) fn install_generic_call_projection_state(
        &mut self,
        state: GenericCallProjectionState,
    ) {
        self.declaration = state.declaration;
        self.text = state.text;
        self.markers = state.markers;
        self.same_class_generic_body_proof = state.same_class_generic_body_proof;
        self.generic_signature_refused = state.generic_signature_refused;
        self.generic_signature_projected = state.generic_signature_projected;
        self.enum_constructor_source_tail = state.enum_constructor_source_tail;
        self.enum_constructor_signature_erasure_refused =
            state.enum_constructor_signature_erasure_refused;
    }

    fn take_generic_call_projection_state(&mut self) -> GenericCallProjectionState {
        GenericCallProjectionState {
            declaration: self.declaration.take(),
            text: std::mem::take(&mut self.text),
            markers: std::mem::take(&mut self.markers),
            same_class_generic_body_proof: self.same_class_generic_body_proof.take(),
            generic_signature_refused: std::mem::take(&mut self.generic_signature_refused),
            generic_signature_projected: std::mem::take(&mut self.generic_signature_projected),
            enum_constructor_source_tail: std::mem::replace(
                &mut self.enum_constructor_source_tail,
                EnumConstructorSourceTail::Unrecognized,
            ),
            enum_constructor_signature_erasure_refused: std::mem::take(
                &mut self.enum_constructor_signature_erasure_refused,
            ),
        }
    }
}

impl GenericCallProjectionState {
    /// Clone a raw pre-transaction overlay for one component's isolated staging area.
    pub(crate) fn clone_raw_for_stage(&self, budget: &mut Budget) -> Result<Self> {
        budget.poll()?;
        if self.same_class_generic_body_proof.is_some() {
            return Err(Error::unsupported(
                "generic_call_projection_state_unexpected_body_proof",
                "only the raw pre-transaction overlay can seed a component stage",
            ));
        }
        let mut copied_bytes = u64::try_from(self.text.len()).unwrap_or(u64::MAX);
        let mut copied_steps = 8u64;
        if let Some(declaration) = &self.declaration {
            copied_bytes =
                copied_bytes.saturating_add(u64::try_from(declaration.len()).unwrap_or(u64::MAX));
        }
        copied_steps =
            copied_steps.saturating_add(u64::try_from(self.markers.len()).unwrap_or(u64::MAX));
        for marker in &self.markers {
            budget.poll()?;
            copied_bytes =
                copied_bytes.saturating_add(u64::try_from(marker.len()).unwrap_or(u64::MAX));
        }
        budget.charge(CountedBudgetDimension::AnalysisSteps, copied_steps)?;
        budget.charge(CountedBudgetDimension::OutputBytes, copied_bytes)?;
        Ok(Self {
            declaration: self.declaration.clone(),
            text: self.text.clone(),
            markers: self.markers.clone(),
            same_class_generic_body_proof: None,
            generic_signature_refused: self.generic_signature_refused,
            generic_signature_projected: self.generic_signature_projected,
            enum_constructor_source_tail: self.enum_constructor_source_tail,
            enum_constructor_signature_erasure_refused: self
                .enum_constructor_signature_erasure_refused,
        })
    }

    /// Stage a declared-source body beside the pre-projection descriptor record. Callers install
    /// this overlay only after all related declarations and physical consumers have passed.
    pub(crate) fn project_declared_body(
        &mut self,
        record: &ClassSourceMethod,
        declaration: String,
        body: &str,
        signature: bool,
        budget: &mut Budget,
    ) -> Result<bool> {
        let Some(placed) = artifact(body) else {
            return Ok(false);
        };
        let marker =
            "// jarde: declared source types and physical consumers committed together".to_owned();
        budget.poll()?;
        budget.charge(CountedBudgetDimension::OutputBytes, marker.len() as u64)?;
        self.markers.push(marker);
        let bytes = block_member_text_bytes(
            &declaration,
            placed.envelope,
            placed.statements,
            &self.markers,
            &record.annotations,
        );
        budget.charge(CountedBudgetDimension::OutputBytes, bytes)?;
        let Some(text) = jarde_java::source_proof::projected_body_method_text(
            Some(&declaration),
            match &record.outcome {
                ClassSourceOutcome::Recovered { report, .. } => Some(report),
                _ => None,
            },
            &self.markers,
            &record.item.name,
            &record.item.descriptor,
            &record.annotations,
            body,
            Some(&declaration),
            &[],
        ) else {
            return Ok(false);
        };
        self.declaration = Some(declaration);
        self.text = text;
        self.generic_signature_refused = false;
        self.generic_signature_projected = signature;
        Ok(true)
    }

    pub(crate) fn project_generic(
        &mut self,
        record: &ClassSourceMethod,
        declaration: String,
        signature: &[u8],
        proof: &str,
        budget: &mut Budget,
    ) -> Result<()> {
        let placed = match &record.outcome {
            ClassSourceOutcome::Recovered { report, .. } => {
                let Some(body) = artifact(&report.text) else {
                    return Ok(());
                };
                Placed::Block(body)
            }
            ClassSourceOutcome::NoBody if record.no_body_kind.is_some() => Placed::Without,
            _ => return Ok(()),
        };
        let marker = format!(
            "// jarde: generic Signature `{}` projected after descriptor erasure and {proof}",
            comment_text(&String::from_utf8_lossy(signature)),
        );
        let marker_bytes = u64::try_from(marker.len()).unwrap_or(u64::MAX);
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        budget.charge(CountedBudgetDimension::OutputBytes, marker_bytes)?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.markers.len()).unwrap_or(u64::MAX),
        )?;
        for _ in &self.markers {
            budget.poll()?;
        }
        let mut markers = std::mem::take(&mut self.markers);
        let old_marker_count = markers.len();
        markers.push(marker);
        let rendered_bytes = match &placed {
            Placed::Without => {
                declaration_member_text_bytes(&declaration, &markers, &record.annotations)
            }
            Placed::Block(body) => block_member_text_bytes(
                &declaration,
                body.envelope,
                body.statements,
                &markers,
                &record.annotations,
            ),
            Placed::Quoted(_) => unreachable!("generic Signature body is a recovered block"),
        };
        if let Err(error) = budget.charge(CountedBudgetDimension::OutputBytes, rendered_bytes) {
            markers.truncate(old_marker_count);
            self.markers = markers;
            return Err(error.into());
        }
        let text = prefix_method_annotations(
            match placed {
                Placed::Without => declaration_member(&declaration, &markers),
                body => block_member(&declaration, body, &markers),
            },
            &record.annotations,
        );
        self.declaration = Some(declaration);
        self.text = text;
        self.markers = markers;
        self.generic_signature_refused = false;
        self.generic_signature_projected = true;
        Ok(())
    }

    pub(crate) fn refuse_generic(
        &mut self,
        record: &ClassSourceMethod,
        reason: &str,
        budget: &mut Budget,
    ) -> Result<()> {
        let marker = format!(
            "// jarde: generic Signature projection refused for `{}`: {}",
            label(&record.item),
            comment_text(reason)
        );
        let marker_bytes = u64::try_from(marker.len()).unwrap_or(u64::MAX);
        let text_bytes = u64::try_from(self.text.len())
            .unwrap_or(u64::MAX)
            .saturating_add(marker_bytes)
            .saturating_add(5);
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        budget.charge(CountedBudgetDimension::OutputBytes, marker_bytes)?;
        budget.charge(CountedBudgetDimension::OutputBytes, text_bytes)?;
        let text = format!("    {marker}\n{}", self.text);
        self.markers.push(marker);
        self.text = text;
        self.generic_signature_refused = true;
        Ok(())
    }

    pub(crate) fn project_generic_call_arguments(
        &mut self,
        record: &ClassSourceMethod,
        original_emission: &str,
        projected_emission: &str,
        markers: &[String],
        budget: &mut Budget,
    ) -> Result<bool> {
        budget.poll()?;
        let mut copied_bytes = 0u64;
        for marker in &self.markers {
            budget.poll()?;
            copied_bytes =
                copied_bytes.saturating_add(u64::try_from(marker.len()).unwrap_or(u64::MAX));
        }
        for marker in markers {
            budget.poll()?;
            copied_bytes =
                copied_bytes.saturating_add(u64::try_from(marker.len()).unwrap_or(u64::MAX));
        }
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(markers.len())
                .unwrap_or(u64::MAX)
                .saturating_add(u64::try_from(self.markers.len()).unwrap_or(u64::MAX))
                .saturating_add(1),
        )?;
        budget.charge(CountedBudgetDimension::OutputBytes, copied_bytes)?;
        let ClassSourceOutcome::Recovered { report, .. } = &record.outcome else {
            return Ok(false);
        };
        let Some(original_artifact) = artifact(&report.text) else {
            return Ok(false);
        };
        let Some(declaration) = self.declaration.as_deref() else {
            return Ok(false);
        };
        let Some(original_text) = self.lambda_projection_text(record, &report.text, &[], budget)?
        else {
            return Ok(false);
        };
        let original_matches =
            class_source_body_lines_match(original_emission, original_artifact.statements);
        if original_text != self.text || !original_matches {
            return Ok(false);
        }
        let mut all_markers = std::mem::take(&mut self.markers);
        let old_marker_count = all_markers.len();
        all_markers.extend(markers.iter().cloned());
        let rendered_bytes = block_member_text_bytes(
            declaration,
            original_artifact.envelope,
            projected_emission,
            &all_markers,
            &record.annotations,
        );
        if let Err(error) = budget.charge(CountedBudgetDimension::OutputBytes, rendered_bytes) {
            all_markers.truncate(old_marker_count);
            self.markers = all_markers;
            return Err(error.into());
        }
        let text = prefix_method_annotations(
            block_member(
                declaration,
                Placed::Block(Artifact {
                    envelope: original_artifact.envelope,
                    statements: projected_emission,
                }),
                &all_markers,
            ),
            &record.annotations,
        );
        self.text = text;
        self.markers = all_markers;
        Ok(true)
    }

    fn lambda_projection_text(
        &self,
        record: &ClassSourceMethod,
        recovery_text: &str,
        markers: &[String],
        budget: &mut Budget,
    ) -> Result<Option<String>> {
        budget.poll()?;
        let Some(declaration) = self.declaration.as_deref() else {
            return Ok(None);
        };
        let Some(artifact) = artifact(recovery_text) else {
            return Ok(None);
        };
        let mut copied_bytes = 0u64;
        for marker in &self.markers {
            budget.poll()?;
            copied_bytes =
                copied_bytes.saturating_add(u64::try_from(marker.len()).unwrap_or(u64::MAX));
        }
        for marker in markers {
            budget.poll()?;
            copied_bytes =
                copied_bytes.saturating_add(u64::try_from(marker.len()).unwrap_or(u64::MAX));
        }
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.markers.len())
                .unwrap_or(u64::MAX)
                .saturating_add(u64::try_from(markers.len()).unwrap_or(u64::MAX)),
        )?;
        budget.charge(CountedBudgetDimension::OutputBytes, copied_bytes)?;
        budget.charge(
            CountedBudgetDimension::OutputBytes,
            block_member_text_bytes(
                declaration,
                artifact.envelope,
                artifact.statements,
                &self.markers,
                &record.annotations,
            ),
        )?;
        let mut all_markers = self.markers.clone();
        all_markers.extend(markers.iter().cloned());
        Ok(Some(prefix_method_annotations(
            block_member(declaration, Placed::Block(artifact), &all_markers),
            &record.annotations,
        )))
    }
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
/// order for a projected static group, otherwise in physical field-table order), then
/// its methods in the order the class file's own method table declares them. A fully projected
/// proved static initializer is omitted from assembled source, but remains in `methods` with its
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
    pub source_target: crate::SourceTarget,
    pub runtime_profile: jarde_reader::view::RuntimeProfile,
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
    /// Names written only in the assembled source, anchored to their physical method and field.
    pub integer_constant_projections: Vec<MemberFamilyDerivedProjection>,
    #[serde(skip)]
    pub(crate) integer_constant_candidates: Vec<ProvedIntegerConstant>,
    /// Source-only direct override hints, each anchored to the child and selected parent methods.
    /// Neither method's physical annotation attributes are changed.
    pub direct_override_proofs: Vec<ClassSourceDirectOverrideProof>,
    /// Direct member evidence for this request. A prepared child keeps its own physical class,
    /// member table, coverage, execution and source maps. Its `usage` is the cumulative snapshot
    /// of the *same* request budget at the end of that child's preparation.
    pub member_family: ClassSourceMemberFamily,
    /// A separately proved, narrow nested enum source projection. Each enum keeps its own
    /// physical report (including its enum group proof); projecting it only changes the owner's
    /// assembled text.
    pub nested_enum_family: ClassSourceNestedEnumFamily,
    /// A separately proved direct member annotation. Its child remains a physical report even
    /// when the owner's assembled source contains the declaration.
    pub nested_annotation_family: ClassSourceNestedAnnotationFamily,
    /// The bounded anonymous-interface expression projection, with ranges in `text` and anchors
    /// to the physical root/child method sites that supplied each generated expression.
    pub anonymous_interface_projection: ClassSourceAnonymousInterfaceProjection,
    /// Same-run class-level bridge admission results. These are adapter evidence for the
    /// subsequent source projection and remain visible beside the physical method records.
    #[doc(hidden)]
    pub bridge_proofs: Vec<ClassSourceBridgeProof>,
    /// Cross-class enum-switch mapping decisions from this class-source run, including refusals.
    #[doc(hidden)]
    pub enum_switch_proofs: Vec<ClassSourceEnumSwitchProof>,
    /// The all-or-nothing proof result for ordinary Java 8 static field initializers. The
    /// proof's field indices and write positions remain available beside the projected declarations;
    /// the original method records and recovery reports are retained in [`Self::methods`].
    pub initializer_proof: ClassSourceInitializerProof,
    /// The class-text projection inputs this report's assembled [`Self::text`] was staged with,
    /// retained as facts of the first pass so the member-fold channel can re-project the same
    /// text byte-for-byte (see [`ClassSourceProjectionInputs`]). Absent when the first pass
    /// staged no projection beside the physical member records.
    #[serde(skip_serializing_if = "ClassSourceProjectionInputs::is_empty")]
    pub projection_inputs: ClassSourceProjectionInputs,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceDirectOverrideProof {
    pub child: jarde_reader::model::PhysicalMethodId,
    pub parent: jarde_reader::model::PhysicalMethodId,
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
    RefusedPair {
        reason: String,
        /// Every child physically prepared before the joint proof refused.
        children: Vec<Box<ClassSourceReport>>,
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
    PreparedPair {
        /// The root's InnerClasses order, retained with each selected physical child.
        members: [ClassSourceMemberChild; 2],
        projection: ClassSourceMemberProjection,
    },
    /// The direct static member family a fold projection renders: every child stays its own
    /// physical report, and only `projection: Projected` states that the root's text carries
    /// the nested declarations. The members are in the root's InnerClasses row order.
    PreparedStatic {
        members: Vec<ClassSourceMemberChild>,
        projection: ClassSourceMemberProjection,
    },
    /// The member family a fold projection renders with proved non-static rows beside its
    /// static ones: the root's text carries every nested declaration, the synthetic outer
    /// capture (constructor parameter, field and write) is elided inside that scope, and the
    /// javac access bridges the family proved are hidden with their call sites re-spelled. The
    /// members are in the root's InnerClasses row order.
    PreparedFold {
        members: Vec<ClassSourceMemberChild>,
        projection: ClassSourceMemberProjection,
    },
}

/// One selected physical child retained beside its proved relation, in the order the root's
/// own `InnerClasses` table states. The declaration pair and the static fold family share it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ClassSourceMemberChild {
    pub relation: ClassSourceMemberRelation,
    pub child: Box<ClassSourceReport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassSourceNestedEnumFamily {
    Absent,
    Refused {
        reason: String,
        #[serde(skip)]
        child: Option<Box<ClassSourceReport>>,
    },
    Prepared {
        relation: ClassSourceMemberRelation,
        #[serde(skip)]
        child: Box<ClassSourceReport>,
        projection: ClassSourceNestedEnumProjection,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassSourceNestedEnumProjection {
    Refused {
        reason: String,
    },
    Projected {
        derived: Vec<MemberFamilyDerivedProjection>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassSourceNestedAnnotationFamily {
    Absent,
    Refused {
        reason: String,
        #[serde(skip)]
        child: Option<Box<ClassSourceReport>>,
    },
    Prepared {
        relation: ClassSourceMemberRelation,
        #[serde(skip)]
        child: Box<ClassSourceReport>,
        projection: ClassSourceNestedAnnotationProjection,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassSourceNestedAnnotationProjection {
    Refused {
        reason: String,
    },
    Projected {
        derived: Vec<MemberFamilyDerivedProjection>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassSourceAnonymousInterfaceProjection {
    Absent,
    Refused {
        reason: String,
    },
    Projected {
        derived: Vec<MemberFamilyDerivedProjection>,
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
    IntegerConstantName,
    NestedEnumDeclaration,
    NestedAnnotationDeclaration,
    NestedEnumTypeReference,
    MemberConstruction,
    MemberReturnType,
    MemberGenericSignature,
    MemberConstructorName,
    MemberClassDeclaration,
    MemberMethodDeclaration,
    CapturedOuterRead,
    OuterSuperCall,
    HiddenCaptureField,
    HiddenConstructorParameter,
    HiddenCaptureWrite,
    HiddenOuterSuperBridge,
    NestedAnonymousExpression,
    /// One reference a static member fold re-spelled: a pool `$` name of a folded member became
    /// the nested source spelling inside the fold's own text. The anchors name the physical
    /// evidence (field, method point, or member signature) beside both class definitions.
    StaticMemberTypeReference,
    /// One javac access bridge a member fold hid from the root's text: the anchors name the
    /// bridge's own method identity beside the passthrough field it read.
    HiddenAccessBridge,
    /// One access-bridge call site a member fold re-spelled into the qualified field access the
    /// bridge forwarded: the anchors name the invokestatic site and the hidden bridge.
    AccessBridgeCall,
    /// The javac assert switch field the assert projection hid from the class's text: the
    /// anchors name the physical field beside the class definition it was proved against, and
    /// the span is the header the field's declaration would have followed.
    HiddenAssertSwitchField,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemberFamilyPhysicalAnchor {
    ClassDefinition {
        definition: PhysicalDefinitionId,
    },
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
    MethodSignature {
        method: PhysicalMethodId,
    },
    /// The bridge's raw MethodRef and the exact selected target declaration are retained together.
    OuterSuperTarget {
        method: PhysicalMethodId,
        owner: JvmBytes,
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    EnumConstantGroup {
        definition: PhysicalDefinitionId,
        constants: Vec<jarde_reader::model::PhysicalMemberId>,
        initializer: PhysicalMethodId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassSourceMemberCalls {
    Proved {
        sites: Vec<MemberCallProof>,
    },
    StaticProved {
        sites: Vec<StaticMemberCallProof>,
    },
    StaticDeclarationOnly,
    Refused {
        reason: String,
        sites: Vec<MemberCallProof>,
        refusals: Vec<MemberCallRefusal>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StaticMemberCallProof {
    pub caller: PhysicalMethodId,
    pub allocation_bci: u32,
    pub copy_bci: u32,
    pub constructor_bci: u32,
    pub constructor: PhysicalMethodId,
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
    Proved {
        proof: MemberCaptureProof,
    },
    StaticNoCapture {
        #[serde(skip)]
        target: Box<jarde_java::report::StaticMemberTargetInput>,
    },
    /// The physical relation is proved, but no constructor use target is claimed.
    StaticDeclaration,
    Refused {
        reason: String,
    },
}

pub use jarde_java::source_proof::{MemberCaptureProof, MemberCaptureRead};

/// A physical, linear bridge from a selected Outer to its direct superclass method.
/// This is evidence only; the family writer must separately accept every use before hiding it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OuterSuperBridgeProof {
    pub bridge: PhysicalMethodId,
    pub outer_name: JvmBytes,
    pub invoke_bci: u32,
    pub target_method: PhysicalMethodId,
    pub target_owner: JvmBytes,
    pub target_name: JvmBytes,
    pub target_descriptor: JvmBytes,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OuterSuperCallProof {
    pub caller: PhysicalMethodId,
    pub call_bci: u32,
    pub capture_read_bci: u32,
    pub argument_bcis: Vec<u32>,
    pub bridge: OuterSuperBridgeProof,
}

/// Complete physical use census within the selected input and request-visible scope. It does
/// not by itself prove Java source overload binding; the writer must establish that separately.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OuterSuperBridgeClosureProof {
    pub bridge: OuterSuperBridgeProof,
    pub calls: Vec<OuterSuperCallProof>,
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

pub use jarde_java::bridge::ClassSourceBridgeProof;
pub use jarde_java::source_proof::{ClassSourceEnumSwitchEntry, ClassSourceEnumSwitchProof};

pub use jarde_java::source_proof::{ClassSourceInitializerField, ClassSourceInitializerProof};

/// The class-text projection inputs one root's assembled [`ClassSourceReport::text`] was staged
/// with, retained as the facts of that first pass — never recomputed, and never a second decision.
///
/// The first pass assembles its text from the physical member records plus a handful of same-run
/// side channels (omitted helper members, marker lines, substituted member texts, an enum-constant
/// projection and a proved initializer field order). Those channels live in the assembly context
/// alone, so a later projection of the same report — the member fold's root re-projection — could
/// previously rebuild only the physical text and had to refuse whenever the first pass had staged
/// anything else. Retaining them here closes that gap the only sound way it can be closed: as
/// facts. Every entry is what the first pass actually wrote (a text, an index, a marker line), and
/// the consumer's re-projection is byte-equal to the first pass or it is refused.
///
/// The field is absent from the serialized report when every channel is empty — the class without
/// any of these projections serializes exactly as it did before this field existed.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceProjectionInputs {
    /// Physical method indices the assembled text omits entirely: proved array-constructor
    /// helpers and omitted lambda companions, whose every class-wide use a projection rewrote,
    /// and a `<clinit>` the assert projection left empty. The member records themselves stay in
    /// [`ClassSourceReport::methods`].
    pub omitted_methods: Vec<u64>,
    /// Physical field indices the assembled text omits: the javac assert switch field, hidden
    /// with its whole pattern (initialization line and every guard) proved rewritten to `assert`
    /// statements. The field records themselves stay in [`ClassSourceReport::fields`].
    pub hidden_fields: Vec<u64>,
    /// The marker lines the assembled text carries at the top of the class body, in order.
    pub markers: Vec<String>,
    /// The composed member texts the assembled text substitutes for those members' physical
    /// ones: each entry is the exact text the first pass placed (declaration, markers, body).
    pub member_texts: Vec<ClassSourceProjectedMemberText>,
    /// The root's own proved enum-constant projection, as the texts it staged.
    pub enum_projection: Option<ClassSourceEnumProjectionTexts>,
}

impl ClassSourceProjectionInputs {
    /// Whether every channel is empty: the serialized form of a class without projections.
    pub fn is_empty(&self) -> bool {
        self.omitted_methods.is_empty()
            && self.hidden_fields.is_empty()
            && self.markers.is_empty()
            && self.member_texts.is_empty()
            && self.enum_projection.is_none()
    }
}

/// One member's projected text, as the first pass placed it into the assembled class text.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceProjectedMemberText {
    /// The unchanged physical `method_info` table index of the member this text substitutes.
    pub member: u64,
    /// The composed text: the member's own declaration and markers, the projection's added
    /// marker lines, and the projected body placed inside them.
    pub text: String,
    /// The projected body this text was placed from, with the anchors a later re-spelling of
    /// that body needs. `None` for a projection this report states as text only — a renamed
    /// lambda companion, an array-constructor rewrite — whose body a later channel may keep but
    /// not re-spell, because no anchor table was emitted for it.
    pub emission: Option<ClassSourceProjectedMemberEmission>,
}

/// The re-spellable half of one projected member text: the emitted body and its own anchor table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceProjectedMemberEmission {
    /// The emitted body artifact: envelope comments, braces, statements — the text a projection
    /// re-emitted from the member's own same-run program with its edits applied.
    pub body: String,
    /// The marker lines the projection added beside the member's own markers.
    pub extra_markers: Vec<String>,
    /// The emitted body's anchor table: byte ranges in `body` with the physical instruction
    /// positions their nodes came from. Only segments anchored to this member are retained; a
    /// spliced-in donor body (an inlined companion) keeps its own member's anchors and is not
    /// re-spellable through this table.
    pub segments: Vec<ClassSourceBodySegment>,
}

/// One anchor of a projected member emission: a byte range and the physical positions it states.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceBodySegment {
    /// Start of the range, in bytes of [`ClassSourceProjectedMemberEmission::body`].
    pub start: u64,
    /// End of the range, in bytes of the same body.
    pub end: u64,
    /// The bytecode indexes of the physical instructions this range's nodes came from.
    pub bcis: Vec<u32>,
}

/// The root's own enum-constant projection as staged texts: the constants list, the source
/// constructors and the static initializer suffix, with the physical indices they replace or
/// omit. A stopped or refused proof never reaches this retention.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceEnumProjectionTexts {
    /// Physical field indices of the projected enum constants.
    pub constant_field_indices: Vec<u64>,
    /// The backing array field the projection's constants replaced.
    pub backing_field_index: u64,
    /// Physical method indices the projection omits (`values`/`valueOf` and the like).
    pub implicit_method_indices: Vec<u64>,
    /// The constants list text, as placed.
    pub constants_text: String,
    /// The source constructor texts, each with the physical method index it replaces.
    pub constructor_texts: Vec<ClassSourceEnumConstructorText>,
    /// The proved static initializer suffix, when one was staged.
    pub initializer_text: Option<String>,
}

/// One enum constructor the projection re-spelled, keyed by the member it replaces.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceEnumConstructorText {
    /// The unchanged physical `method_info` table index of the constructor.
    pub member: u64,
    /// The source constructor text, as placed.
    pub text: String,
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
/// `ACC_VARARGS` (JVMS 4.6-A) in a *method's* own `access_flags`, stating
/// that the member's last parameter is the variable-arity one. It is read where the parameter list
/// is spelled ([`method_descriptor`]) and nowhere else.
const ACC_VARARGS: u16 = 0x0080;
/// `ACC_INTERFACE` (JVMS 4.1-B).
const ACC_INTERFACE: u16 = 0x0200;
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
fn class_name(raw: &[u8]) -> Option<String> {
    jarde_java::names::source_class_name(&JvmBytes(raw.to_vec()))
}

/// One field descriptor as Java source spells it (JVMS 4.3.2), or `None` when the bytes are not
/// that production. Tests spell field descriptors from literals; the assembly paths read
/// [`jarde_java::declaration::field_type`] where their descriptors already are.
#[cfg(test)]
fn field_type(descriptor: &[u8]) -> Option<String> {
    jarde_java::declaration::field_type(descriptor)
}

/// Adapt the physical member names to Java's shared marker spelling.
fn label(item: &MethodItem) -> String {
    jarde_java::source_proof::member_label(&item.name, &item.descriptor)
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

/// The raw value one member's own `AnnotationDefault` attribute states (JVMS 4.7.22),
/// or `None` when the attribute is absent. Java literal resolution belongs to the caller
/// supplying this source unit's naming facts.
///
/// The read is [`attribute_facts`] over this member's own `AnnotationDefault` shell(s) — the reader's
/// one parser for that attribute, and only those shells are handed over, so the `attribute_bytes`
/// charge is exactly this attribute's — and nothing here reads a body, a use site or another member:
/// the declaration default is the value the member's own attribute declares.
///
/// `None` means that the member has no attribute. A present raw value crosses the Java
/// literal gate with this source unit's naming facts before publication. The caller isolates
/// local refusals and propagates budget/cancellation as shared request stops.
pub(crate) fn declared_annotation_default(
    bytes: &[u8],
    member: &MemberHeader,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<Option<ElementValueFacts>> {
    if !declares_annotation_default(member) {
        return Ok(None);
    }
    let shells = attribute_shells(member, b"AnnotationDefault");
    let value = attribute_facts(bytes, &shells, pool, budget)?
        .annotation_default
        .ok_or_else(|| {
            Error::invalid_input(
                "classfile_invalid_attribute_content",
                "declared AnnotationDefault has no decoded value",
            )
        })?;
    Ok(Some(value))
}

/// The declaration facts one member's own attribute table states, as its declaration writes them.
///
/// `Exceptions` is read once from this member's own shells, in attribute order. Its raw names
/// prove a generic `Signature`'s optional throws suffix; its source spellings keep the same
/// declaration clause before and after the member's recovery run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MemberAttributes {
    /// The member's own `AnnotationDefault` (JVMS 4.7.22), when it declares one this presentation
    /// declares, kept raw until the source unit supplies its naming context.
    pub(crate) default: Option<ElementValueFacts>,
    /// The exception types the member's own `Exceptions` attribute (JVMS 4.7.4) states, in the
    /// attribute's own order; empty when the member declares none.
    pub(crate) throws: Vec<String>,
    /// The same physical exception names before source spelling, for signature erasure proof.
    pub(crate) throws_raw: Vec<Vec<u8>>,
    /// A complete descriptor-to-slot projection of this member's parameter attribute.
    pub(crate) parameters: Option<Vec<ProvedMethodParameter>>,
}

fn prove_method_parameters(
    entries: &[jarde_reader::classfile::MethodParameterFacts],
    descriptor: &[u8],
    flags: u16,
) -> Option<Vec<ProvedMethodParameter>> {
    let signature = method_descriptor(descriptor, is_static(flags), flags & ACC_VARARGS != 0)?;
    if entries.len() != signature.parameters.len() || flags & (0x1000 | 0x0040) != 0 {
        return None;
    }
    // Metadata proves parameter slots and flags. Source spelling belongs to the shared NameTable,
    // including keywords, supplementary names, and collisions with the wide-slot fallback names.
    entries
        .iter()
        .zip(signature.parameters.iter())
        .map(|(entry, (_, slot))| {
            if entry.access_flags & (0x1000 | 0x8000) != 0 {
                return None;
            }
            let raw = entry.name.as_ref()?;
            jarde_reader::model::JvmString::from_mutf8(raw.0.clone())?;
            Some(ProvedMethodParameter {
                slot: *slot,
                name: raw.clone(),
                is_final: entry.access_flags & ACC_FINAL != 0,
            })
        })
        .collect()
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
    let parameters = if member
        .attributes
        .iter()
        .any(|attribute| attribute.name.raw().0 == b"MethodParameters")
    {
        let entries = attribute_facts(
            bytes,
            &attribute_shells(member, b"MethodParameters"),
            pool,
            budget,
        )?
        .method_parameters;
        if let Some(entries) = entries {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(member.descriptor.raw().0.len())
                    .unwrap_or(u64::MAX)
                    .saturating_add(
                        u64::try_from(entries.len())
                            .unwrap_or(u64::MAX)
                            .saturating_mul(4),
                    ),
            )?;
            prove_method_parameters(&entries, &member.descriptor.raw().0, member.access_flags)
        } else {
            None
        }
    } else {
        None
    };
    Ok(MemberAttributes {
        default,
        throws: throws_raw
            .iter()
            .map(|name| class_name(name))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                Error::unsupported(
                    "class_source_name_unspellable",
                    "exception type has no faithful Java source name",
                )
            })?,
        throws_raw,
        parameters,
    })
}

/// Select only the two exact Java 8 functional targets whose source spelling this slice proves.
/// Parsing and erasure use the same path as final declaration projection; this is only a target
/// hypothesis until the direct Code, bootstrap, Program and SSA are checked in the same run.
pub(crate) fn typed_functional_signature_target(
    member: &MemberHeader,
    attributes: &MemberAttributes,
    bytes: &[u8],
    pool: &[CpEntryFacts],
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Result<Option<TypedFunctionalKind>> {
    let shells = attribute_shells(member, b"Signature");
    if shells.len() != 1
        || class_internal.contains(&b'$')
        || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
        || !class_scope.is_empty()
        || class_signature_present
        || !attributes.throws_raw.is_empty()
        || member.descriptor.raw().0.as_slice() != b"()Ljava/util/function/Function;"
            && member.descriptor.raw().0.as_slice() != b"()Ljava/util/function/Supplier;"
    {
        return Ok(None);
    }
    let raw = attribute_facts(bytes, &shells, pool, budget)?
        .signature
        .ok_or_else(|| {
            Error::invalid_input(
                "jvm_signature_missing",
                "Signature attribute did not resolve",
            )
        })?
        .0;
    let parsed = parse_method_signature(&raw, budget)?;
    if !parsed.type_parameters.is_empty()
        || !parsed.parameters.is_empty()
        || !parsed.throws.is_empty()
    {
        return Ok(None);
    }
    prove_method_signature_erasure_with_class_scope(
        &parsed,
        &member.descriptor.raw().0,
        &attributes.throws_raw,
        class_scope,
        budget,
    )?;
    Ok(match raw.as_slice() {
        b"()Ljava/util/function/Function<Ljava/lang/String;Ljava/lang/Integer;>;" => {
            Some(TypedFunctionalKind::FunctionStringInteger)
        }
        b"()Ljava/util/function/Supplier<Ljava/lang/String;>;" => {
            Some(TypedFunctionalKind::SupplierString)
        }
        _ => None,
    })
}

/// The class-level state of one member's same-class binding question, as the assembly drives it.
///
/// The ordinary projection path no longer treats a same-class constant-pool entry as a refusal on
/// its own: the entry is a candidate fact, and the decision belongs to the use-site inventory the
/// class-level assembly collects over this class's own decoded bodies. [`Self::Pending`] is what
/// the member loop passes — a member whose pool holds a naming entry is held back unpublished until
/// the inventory is complete — and the commit phase passes the proof outcome for every held member.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SameClassBinding {
    /// The use-site inventory of this presentation is still being collected: hold the candidate.
    Pending,
    /// Every actual same-class use site of the member is proved to keep binding its physical
    /// target, and no unreadable reference source names it.
    Proven,
    /// Every non-overload binding fact is closed, but an actual same-arity source overload call
    /// still needs the generic-call component's complete staged overload proof before commit.
    OverloadPending,
    /// A use site, a competing same-name declaration, or a reference source is not proved; the
    /// member keeps its erased declaration and the existing refusal.
    Unproved,
}

/// One same-class generic caller dependency. Indices are physical method-table positions, never
/// names or erased descriptors: overloads and method/class binders remain distinct.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GenericCallDependency {
    pub(crate) caller: usize,
    pub(crate) callee: usize,
}

/// A connected set of candidate declarations that must stage or roll back together. `order` is
/// callee-first; a cyclic component has no safe order and is rejected without retrying inference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GenericCallComponent {
    pub(crate) members: Vec<usize>,
    pub(crate) order: Option<Vec<usize>>,
}

// ---------------------------------------------------------------------------------------------
// Closed same-class overload proof for a source cast at a generic call site
// ---------------------------------------------------------------------------------------------

/// Split generic candidates into undirected dependency components and topologically order each
/// acyclic component with callees before callers. This is a bounded DAG pass, not a type fixpoint:
/// candidates outside a dependency component remain independent and can still publish.
pub(crate) fn generic_call_components(
    candidates: &[usize],
    dependencies: &[GenericCallDependency],
    budget: &mut Budget,
) -> Result<Vec<GenericCallComponent>> {
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(candidates.len().saturating_add(dependencies.len())).unwrap_or(u64::MAX),
    )?;
    let mut candidate_set = std::collections::BTreeSet::new();
    for candidate in candidates {
        budget.poll()?;
        if !candidate_set.insert(*candidate) {
            return Err(Error::invalid_input(
                "generic_call_candidate_duplicate",
                "one physical method was staged more than once",
            ));
        }
    }
    let mut adjacency = std::collections::BTreeMap::<usize, Vec<usize>>::new();
    let mut callee_dependencies = std::collections::BTreeMap::<usize, Vec<usize>>::new();
    for candidate in &candidate_set {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        adjacency.entry(*candidate).or_default();
        callee_dependencies.entry(*candidate).or_default();
    }
    for edge in dependencies {
        budget.poll()?;
        if !candidate_set.contains(&edge.caller) || !candidate_set.contains(&edge.callee) {
            continue;
        }
        adjacency.entry(edge.caller).or_default().push(edge.callee);
        adjacency.entry(edge.callee).or_default().push(edge.caller);
        callee_dependencies
            .entry(edge.caller)
            .or_default()
            .push(edge.callee);
    }
    for neighbors in adjacency.values_mut() {
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(neighbors.len()).unwrap_or(u64::MAX),
        )?;
        neighbors.sort_unstable();
        neighbors.dedup();
    }
    for dependencies in callee_dependencies.values_mut() {
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(dependencies.len()).unwrap_or(u64::MAX),
        )?;
        dependencies.sort_unstable();
        dependencies.dedup();
    }

    let mut remaining = candidate_set;
    let mut components = Vec::new();
    while let Some(start) = remaining.first().copied() {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let mut pending = vec![start];
        let mut members = Vec::new();
        remaining.remove(&start);
        while let Some(member) = pending.pop() {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            members.push(member);
            if let Some(neighbors) = adjacency.get(&member) {
                for neighbor in neighbors {
                    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                    if remaining.remove(neighbor) {
                        pending.push(*neighbor);
                    }
                }
            }
        }
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(members.len()).unwrap_or(u64::MAX),
        )?;
        members.sort_unstable();

        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(members.len()).unwrap_or(u64::MAX),
        )?;
        let mut indegree = std::collections::BTreeMap::new();
        let member_set: std::collections::BTreeSet<_> = members.iter().copied().collect();
        for member in &members {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            let mut count = 0usize;
            for callee in callee_dependencies.get(member).into_iter().flatten() {
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                if member_set.contains(callee) {
                    count += 1;
                }
            }
            indegree.insert(*member, count);
        }
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(indegree.len()).unwrap_or(u64::MAX),
        )?;
        let mut ready: std::collections::BTreeSet<usize> = indegree
            .iter()
            .filter_map(|(member, count)| (*count == 0).then_some(*member))
            .collect();
        let mut order = Vec::with_capacity(members.len());
        while let Some(callee) = ready.pop_first() {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            order.push(callee);
            for caller in adjacency.get(&callee).into_iter().flatten() {
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                let depends_on = callee_dependencies
                    .get(caller)
                    .is_some_and(|callees| callees.binary_search(&callee).is_ok());
                if !depends_on {
                    continue;
                }
                let Some(count) = indegree.get_mut(caller) else {
                    continue;
                };
                *count = count.saturating_sub(1);
                if *count == 0 {
                    ready.insert(*caller);
                }
            }
        }
        let order = (order.len() == members.len()).then_some(order);
        components.push(GenericCallComponent { members, order });
    }
    Ok(components)
}

/// Find a same-class constructor call whose receiver is this object's uninitialized `this`, or
/// whose receiver identity the same-run SSA could not establish. The source argument adaptation of
/// that delegation is outside the class-scope constructor proof, so either result blocks that
/// constructor's projected header. A `new` allocation is positively classified and remains safe.
pub(crate) fn unproved_same_class_this_delegate_bci(
    class_internal: &[u8],
    member: &MemberHeader,
    facts: &SameClassUseFacts,
    budget: &mut Budget,
) -> Result<Option<u32>> {
    if member.name.raw().0 != b"<init>" {
        return Ok(None);
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(facts.invokes.len()).unwrap_or(u64::MAX),
    )?;
    for use_site in facts.invokes {
        budget.poll()?;
        if use_site.owner != class_internal
            || use_site.name != b"<init>"
            || use_site.descriptor != member.descriptor.raw().0
        {
            continue;
        }
        match use_site.constructor_receiver {
            Some(SameClassConstructorReceiver::UninitializedThis)
            | Some(SameClassConstructorReceiver::Unknown)
            | None => return Ok(Some(use_site.bci)),
            Some(SameClassConstructorReceiver::NewObject) => {}
        }
    }
    Ok(None)
}

/// The number of parameters one method descriptor declares.
fn descriptor_parameter_count(descriptor: &[u8]) -> Option<usize> {
    method_descriptor(descriptor, false, false).map(|signature| signature.parameters.len())
}

/// Prove one held generic method candidate's same-class binding from the completed inventory.
///
/// The proof is deliberately bounded to what the selected class alone can close: the class's own
/// physical parent is `java/lang/Object` and it implements no interfaces (no external same-name
/// candidate can compete), the name is not one of `Object`'s own instance methods, and every
/// same-name declaration is represented uniquely and can be named in source. With no same-arity
/// sibling, the physical descriptor closes binding directly. A real invoke to any member of the
/// closed same-arity set yields [`SameClassBinding::OverloadPending`], including for a generic
/// sibling that has no direct incoming invoke of its own; only the complete staged overload proof
/// may promote it before the component commits. Variable-arity/bridge members, unexplained sites,
/// non-body references, or incomplete inventories remain refused.
pub(crate) fn prove_same_class_method_binding(
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    method_headers: &[MemberHeader],
    member_index: usize,
    facts: &SameClassUseFacts,
    budget: &mut Budget,
) -> Result<SameClassBinding> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(method_headers.len()).unwrap_or(u64::MAX),
    )?;
    let Some(member) = method_headers.get(member_index) else {
        return Ok(SameClassBinding::Unproved);
    };
    let name = member.name.raw().0.as_slice();
    let descriptor = member.descriptor.raw().0.as_slice();
    // The visible candidate set closes only over the selected class and the platform root.
    if class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
        || is_object_instance_method_name(name)
    {
        return Ok(SameClassBinding::Unproved);
    }
    if name != b"<init>" && source_identifier(name).is_none() {
        return Ok(SameClassBinding::Unproved);
    }
    if !facts.complete {
        return Ok(SameClassBinding::Unproved);
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(method_headers.len())
            .unwrap_or(u64::MAX)
            .saturating_mul(u64::try_from(method_headers.len()).unwrap_or(u64::MAX)),
    )?;

    // A source overload set cannot contain bytecode-only bridge/synthetic declarations or
    // variable-arity members: either can affect applicability without a source declaration that
    // the final staged set can faithfully represent. Also require every descriptor to be valid
    // and unique, so an invoke can be joined to exactly one physical header.
    let mut overload_competition = false;
    let arity = descriptor_parameter_count(descriptor);
    for (index, sibling) in method_headers.iter().enumerate() {
        budget.poll()?;
        if sibling.name.raw().0.as_slice() != name {
            continue;
        }
        let sibling_descriptor = sibling.descriptor.raw().0.as_slice();
        if sibling.access_flags & (ACC_VARARGS | 0x0040 | 0x1000) != 0
            || descriptor_parameter_count(sibling_descriptor).is_none()
        {
            return Ok(SameClassBinding::Unproved);
        }
        if method_headers[..index].iter().any(|prior| {
            prior.name.raw().0.as_slice() == name
                && prior.descriptor.raw().0.as_slice() == sibling_descriptor
        }) {
            return Ok(SameClassBinding::Unproved);
        }
        if index != member_index && descriptor_parameter_count(sibling_descriptor) == arity {
            overload_competition = true;
        }
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(facts.invokes.len() + facts.member_refs.len())
            .unwrap_or(u64::MAX)
            .saturating_add(
                u64::try_from(facts.invokes.len())
                    .unwrap_or(u64::MAX)
                    .saturating_mul(u64::try_from(method_headers.len()).unwrap_or(u64::MAX)),
            )
            .saturating_add(
                u64::try_from(facts.member_refs.len())
                    .unwrap_or(u64::MAX)
                    .saturating_mul(u64::try_from(method_headers.len()).unwrap_or(u64::MAX)),
            ),
    )?;
    budget.poll()?;
    let mut same_arity_invokes = 0usize;
    for use_site in facts.invokes {
        budget.poll()?;
        if use_site.name != name {
            continue;
        }
        if use_site.owner != class_internal {
            if overload_competition {
                return Ok(SameClassBinding::Unproved);
            }
            continue;
        }
        let mut matched = method_headers.iter().filter(|header| {
            header.name.raw().0.as_slice() == name
                && header.descriptor.raw().0.as_slice() == use_site.descriptor
        });
        let Some(target_header) = matched.next() else {
            return Ok(SameClassBinding::Unproved);
        };
        if matched.next().is_some() {
            return Ok(SameClassBinding::Unproved);
        }
        let opcode_matches = if is_static(target_header.access_flags) {
            use_site.opcode == 0xb8
        } else {
            matches!(use_site.opcode, 0xb6 | 0xb7)
        };
        if !opcode_matches {
            return Ok(SameClassBinding::Unproved);
        }
        if descriptor_parameter_count(use_site.descriptor.as_slice()) == arity {
            same_arity_invokes = same_arity_invokes.saturating_add(1);
        }
    }
    for reference in facts.member_refs {
        budget.poll()?;
        if !reference.method || reference.name != name {
            continue;
        }
        if reference.owner != class_internal {
            if overload_competition {
                return Ok(SameClassBinding::Unproved);
            }
            continue;
        }
        let mut matched = method_headers.iter().filter(|header| {
            header.name.raw().0.as_slice() == name
                && header.descriptor.raw().0.as_slice() == reference.descriptor
        });
        if matched.next().is_none() || matched.next().is_some() {
            return Ok(SameClassBinding::Unproved);
        }
        // Method handles/bootstrap references are not an invoke site whose source overload can
        // be checked by the generic-call transaction.
        return Ok(SameClassBinding::Unproved);
    }
    if overload_competition {
        return Ok(if same_arity_invokes > 0 {
            SameClassBinding::OverloadPending
        } else {
            SameClassBinding::Unproved
        });
    }
    Ok(SameClassBinding::Proven)
}

/// Prove one held field `Signature` candidate's same-class binding from the completed inventory.
///
/// Fields do not overload, so the binding itself is the owner/name/descriptor identity every
/// access site states; what needs proof is that the *source* keeps working: a field the projected
/// type would make unspellable at one of its own read sites (the loaded value selecting members,
/// flowing into a local, or entering a functional position) stays on its physical descriptor. A
/// same-name sibling field — legal bytes but no Java source spelling — and any unreadable or
/// non-body source naming the field keep the existing refusal.
pub(crate) fn prove_same_class_field_binding(
    class_internal: &[u8],
    field_headers: &[MemberHeader],
    member_index: usize,
    facts: &SameClassUseFacts,
    budget: &mut Budget,
) -> Result<SameClassBinding> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    let member = &field_headers[member_index];
    let name = member.name.raw().0.as_slice();
    let descriptor = member.descriptor.raw().0.as_slice();
    for (index, sibling) in field_headers.iter().enumerate() {
        if index == member_index || sibling.name.raw().0.as_slice() != name {
            continue;
        }
        // Two same-name fields have one source spelling between them at best; the bytes may be
        // legal but the projection cannot claim a binding it cannot even name.
        return Ok(SameClassBinding::Unproved);
    }
    if !facts.complete {
        return Ok(SameClassBinding::Unproved);
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(facts.field_uses.len() + facts.member_refs.len()).unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    for use_site in facts.field_uses {
        if use_site.owner != class_internal || use_site.name != name {
            continue;
        }
        if use_site.descriptor != descriptor {
            return Ok(SameClassBinding::Unproved);
        }
        if use_site.read_expressible == Some(false) {
            return Ok(SameClassBinding::Unproved);
        }
    }
    for reference in facts.member_refs {
        if !reference.method
            && reference.owner == class_internal
            && reference.name == name
            && reference.descriptor == descriptor
        {
            return Ok(SameClassBinding::Unproved);
        }
    }
    Ok(SameClassBinding::Proven)
}

/// The same-class field sites, with their callers and BCIs, for a proven field's source note.
pub(crate) fn same_class_field_use_note(
    facts: &SameClassUseFacts,
    name: &[u8],
    budget: &mut Budget,
) -> Result<String> {
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(facts.field_uses.len()).unwrap_or(u64::MAX),
    )?;
    let mut sites = Vec::new();
    for use_site in facts.field_uses {
        budget.poll()?;
        if use_site.name == name && sites.len() < 4 {
            sites.push(format!("{}@{}", use_site.caller, use_site.bci));
        }
    }
    if sites.is_empty() {
        return Ok("no consumed same-class entry".to_owned());
    }
    Ok(format!("same-class uses at {}", sites.join(", ")))
}

pub(crate) use jarde_java::generic_source_proof::charge_signature_type_proof;

fn charge_field_write_source_proof(
    source: &SameClassFieldWriteSource,
    target: &SignatureType,
    class_scope: &[TypeParameterErasure],
    methods: &[SameClassPublishedMethodParameters],
    budget: &mut Budget,
) -> Result<()> {
    let target_variables = charge_signature_type_proof(target, budget)?;
    match source {
        SameClassFieldWriteSource::Null => {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        }
        SameClassFieldWriteSource::StringConstant { .. } => {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        }
        SameClassFieldWriteSource::RawAllocation { owner } => {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(owner.len()).unwrap_or(u64::MAX),
            )?;
        }
        SameClassFieldWriteSource::Parameter { method, slot } => {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(methods.len()).unwrap_or(u64::MAX),
            )?;
            budget.poll()?;
            let Some(method) = methods.iter().find(|candidate| candidate.method == *method) else {
                return Ok(());
            };
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(method.parameter_slots.len()).unwrap_or(u64::MAX),
            )?;
            budget.poll()?;
            if let Some(position) = method
                .parameter_slots
                .iter()
                .position(|candidate| candidate == slot)
            {
                if let Some(source) = method
                    .generic_parameters
                    .as_ref()
                    .and_then(|parameters| parameters.get(position))
                {
                    if let Some(descriptor) = method.descriptor_parameters.get(position) {
                        budget.charge(
                            CountedBudgetDimension::AnalysisSteps,
                            u64::try_from(descriptor.len()).unwrap_or(u64::MAX),
                        )?;
                    }
                    let source_variables = charge_signature_type_proof(source, budget)?;
                    budget.charge(
                        CountedBudgetDimension::AnalysisSteps,
                        u64::try_from(
                            class_scope
                                .len()
                                .saturating_add(method.method_type_parameters.len()),
                        )
                        .unwrap_or(u64::MAX)
                        .saturating_mul(target_variables.max(source_variables)),
                    )?;
                    if let SignatureType::TypeVariable(name) = source {
                        for parameter in &method.method_type_parameters {
                            budget.poll()?;
                            if parameter.name != *name {
                                continue;
                            }
                            if let Some(bound) = &parameter.class_bound {
                                charge_signature_type_proof(bound, budget)?;
                            }
                            for bound in &parameter.interface_bounds {
                                charge_signature_type_proof(bound, budget)?;
                            }
                        }
                    }
                } else if let Some(descriptor) = method.descriptor_parameters.get(position) {
                    budget.charge(
                        CountedBudgetDimension::AnalysisSteps,
                        u64::try_from(descriptor.len()).unwrap_or(u64::MAX),
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn class_type_arguments_present(ty: &SignatureType) -> bool {
    match ty {
        SignatureType::Array(element) => class_type_arguments_present(element),
        SignatureType::Class(class) => class
            .segments
            .iter()
            .any(|segment| !segment.arguments.is_empty()),
        SignatureType::Base(_) | SignatureType::TypeVariable(_) => false,
    }
}

fn signature_types_same_class_binding(
    left: &SignatureType,
    right: &SignatureType,
    class_scope: &[TypeParameterErasure],
    method_binders: &[jarde_reader::signature::TypeParameter],
) -> bool {
    match (left, right) {
        (SignatureType::Base(left), SignatureType::Base(right)) => left == right,
        (SignatureType::Array(left), SignatureType::Array(right)) => {
            signature_types_same_class_binding(left, right, class_scope, method_binders)
        }
        (SignatureType::TypeVariable(left), SignatureType::TypeVariable(right)) => {
            left == right
                && class_scope_variable(left, class_scope)
                && !method_binders.iter().any(|binder| binder.name == *left)
        }
        (SignatureType::Class(left), SignatureType::Class(right)) => {
            left.segments.len() == right.segments.len()
                && left
                    .segments
                    .iter()
                    .zip(&right.segments)
                    .all(|(left, right)| {
                        left.binary_name == right.binary_name
                            && left.arguments.len() == right.arguments.len()
                            && left
                                .arguments
                                .iter()
                                .zip(&right.arguments)
                                .all(|(left, right)| match (left, right) {
                                    (TypeArgument::Any, TypeArgument::Any) => true,
                                    (TypeArgument::Exact(left), TypeArgument::Exact(right))
                                    | (TypeArgument::Extends(left), TypeArgument::Extends(right))
                                    | (TypeArgument::Super(left), TypeArgument::Super(right)) => {
                                        signature_types_same_class_binding(
                                            left,
                                            right,
                                            class_scope,
                                            method_binders,
                                        )
                                    }
                                    _ => false,
                                })
                    })
        }
        _ => false,
    }
}

fn descriptor_reference_shape(descriptor: &[u8]) -> Option<(u32, Vec<u8>)> {
    let facts = descriptor_facts(descriptor, DescriptorKind::Field).ok()?;
    let component = facts.single()?;
    let Base::Object(name) = component.base() else {
        return None;
    };
    Some((component.dimensions(), name.0.clone()))
}

fn signature_reference_shape(ty: &SignatureType) -> Option<(u32, &[u8])> {
    let mut dimensions = 0_u32;
    let mut base = ty;
    while let SignatureType::Array(element) = base {
        dimensions = dimensions.checked_add(1)?;
        base = element;
    }
    Some((dimensions, signature_class_erasure(base)?))
}

fn signature_raw_reference_matches_descriptor(ty: &SignatureType, descriptor: &[u8]) -> bool {
    if class_type_arguments_present(ty) {
        return false;
    }
    let Some((signature_dimensions, signature_name)) = signature_reference_shape(ty) else {
        return false;
    };
    let Some((descriptor_dimensions, descriptor_name)) = descriptor_reference_shape(descriptor)
    else {
        return false;
    };
    signature_dimensions == descriptor_dimensions && signature_name == descriptor_name
}

fn raw_parameter_assignable(descriptor: &[u8], target: &SignatureType, java_release: u16) -> bool {
    let Some((source_dimensions, source_name)) = descriptor_reference_shape(descriptor) else {
        return false;
    };
    raw_reference_assignable(source_dimensions, &source_name, target, java_release)
}

fn raw_reference_assignable(
    source_dimensions: u32,
    source_name: &[u8],
    target: &SignatureType,
    java_release: u16,
) -> bool {
    let Some((target_dimensions, target_name)) = signature_reference_shape(target) else {
        return false;
    };
    if source_dimensions != target_dimensions {
        return false;
    }
    if source_name == target_name {
        return true;
    }
    let (Ok(source_name), Ok(target_name)) = (
        std::str::from_utf8(source_name),
        std::str::from_utf8(target_name),
    ) else {
        return false;
    };
    let source_name = source_name.replace('/', ".");
    let target_name = target_name.replace('/', ".");
    jarde_java::release_reference_argument_widens(java_release, &source_name, &target_name)
}

fn descriptor_reference_assignable(source: &[u8], target: &[u8], java_release: u16) -> bool {
    if source == target {
        return source
            .first()
            .is_some_and(|byte| *byte == b'L' || *byte == b'[');
    }
    let source_is_reference = source
        .first()
        .is_some_and(|byte| *byte == b'L' || *byte == b'[');
    if source_is_reference && target == b"Ljava/lang/Object;" {
        return true;
    }
    if source.first() == Some(&b'[')
        && matches!(target, b"Ljava/lang/Cloneable;" | b"Ljava/io/Serializable;")
    {
        return true;
    }
    let mut source_component = source;
    let mut target_component = target;
    while source_component.first() == Some(&b'[') && target_component.first() == Some(&b'[') {
        source_component = &source_component[1..];
        target_component = &target_component[1..];
        let source_primitive = source_component.first().is_some_and(|byte| {
            matches!(*byte, b'Z' | b'B' | b'C' | b'S' | b'I' | b'J' | b'F' | b'D')
        });
        let target_primitive = target_component.first().is_some_and(|byte| {
            matches!(*byte, b'Z' | b'B' | b'C' | b'S' | b'I' | b'J' | b'F' | b'D')
        });
        if source_primitive || target_primitive {
            return source_component == target_component;
        }
    }
    match (source_component.first(), target_component.first()) {
        (Some(b'['), Some(b'L')) => matches!(
            target_component,
            b"Ljava/lang/Object;" | b"Ljava/lang/Cloneable;" | b"Ljava/io/Serializable;"
        ),
        (Some(b'L'), Some(b'L')) => {
            let Some((0, source_name)) = descriptor_reference_shape(source_component) else {
                return false;
            };
            let Some((0, target_name)) = descriptor_reference_shape(target_component) else {
                return false;
            };
            target_name == b"java/lang/Object"
                || raw_reference_names_assignable(&source_name, &target_name, java_release)
        }
        _ => false,
    }
}

fn raw_reference_names_assignable(
    source_name: &[u8],
    target_name: &[u8],
    java_release: u16,
) -> bool {
    if source_name == target_name {
        return true;
    }
    let (Ok(source_name), Ok(target_name)) = (
        std::str::from_utf8(source_name),
        std::str::from_utf8(target_name),
    ) else {
        return false;
    };
    let source_name = source_name.replace('/', ".");
    let target_name = target_name.replace('/', ".");
    jarde_java::release_reference_argument_widens(java_release, &source_name, &target_name)
}

fn raw_field_receiver_source(
    use_site: &SameClassFieldUse,
    class_internal: &[u8],
    class_is_generic: bool,
    methods: &[SameClassPublishedMethodParameters],
    budget: &mut Budget,
) -> Result<bool> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if !class_is_generic || use_site.opcode != 0xb5 || use_site.owner != class_internal {
        return Ok(false);
    }
    match use_site.receiver_source.as_ref() {
        Some(SameClassFieldReceiverSource::RawLocal) => Ok(true),
        Some(SameClassFieldReceiverSource::Parameter { slot }) => {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(methods.len()).unwrap_or(u64::MAX),
            )?;
            let mut matching = methods
                .iter()
                .filter(|method| method.method == use_site.physical_method);
            let Some(method) = matching.next() else {
                return Ok(false);
            };
            if matching.next().is_some() {
                return Ok(false);
            }
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(method.parameter_slots.len()).unwrap_or(u64::MAX),
            )?;
            let Some(position) = method
                .parameter_slots
                .iter()
                .position(|parameter_slot| parameter_slot == slot)
            else {
                return Ok(false);
            };
            let Some(descriptor) = method.descriptor_parameters.get(position) else {
                return Ok(false);
            };
            if !descriptor_reference_shape(descriptor)
                .is_some_and(|(dimensions, owner)| dimensions == 0 && owner == class_internal)
            {
                return Ok(false);
            }
            if let Some(parameters) = &method.generic_parameters {
                let Some(source) = parameters.get(position) else {
                    return Ok(false);
                };
                budget.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(descriptor.len()).unwrap_or(u64::MAX),
                )?;
                Ok(signature_raw_reference_matches_descriptor(
                    source, descriptor,
                ))
            } else {
                Ok(true)
            }
        }
        Some(SameClassFieldReceiverSource::This) | None => Ok(false),
    }
}

fn raw_field_write_source_assignable(
    source: &SameClassFieldWriteSource,
    target_descriptor: &[u8],
    methods: &[SameClassPublishedMethodParameters],
    java_release: u16,
    budget: &mut Budget,
) -> Result<bool> {
    budget.poll()?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(target_descriptor.len()).unwrap_or(u64::MAX),
    )?;
    match source {
        SameClassFieldWriteSource::Null => Ok(true),
        SameClassFieldWriteSource::StringConstant { .. } => Ok(false),
        SameClassFieldWriteSource::RawAllocation { owner } => {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(owner.len()).unwrap_or(u64::MAX),
            )?;
            let mut source = Vec::with_capacity(owner.len().saturating_add(2));
            source.push(b'L');
            source.extend_from_slice(owner);
            source.push(b';');
            Ok(descriptor_reference_assignable(
                &source,
                target_descriptor,
                java_release,
            ))
        }
        SameClassFieldWriteSource::Parameter { method, slot } => {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(methods.len()).unwrap_or(u64::MAX),
            )?;
            let mut matching = methods
                .iter()
                .filter(|candidate| candidate.method == *method);
            let Some(method) = matching.next() else {
                return Ok(false);
            };
            if matching.next().is_some() {
                return Ok(false);
            }
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(method.parameter_slots.len()).unwrap_or(u64::MAX),
            )?;
            let Some(position) = method
                .parameter_slots
                .iter()
                .position(|candidate| candidate == slot)
            else {
                return Ok(false);
            };
            let Some(source_descriptor) = method.descriptor_parameters.get(position) else {
                return Ok(false);
            };
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(source_descriptor.len()).unwrap_or(u64::MAX),
            )?;
            Ok(descriptor_reference_assignable(
                source_descriptor,
                target_descriptor,
                java_release,
            ))
        }
    }
}

fn raw_method_type_variable_bound<'a>(
    source: &SignatureType,
    method: &'a SameClassPublishedMethodParameters,
    descriptor: &[u8],
) -> Option<&'a [u8]> {
    let SignatureType::TypeVariable(name) = source else {
        return None;
    };
    let mut parameters = method
        .method_type_parameters
        .iter()
        .filter(|parameter| parameter.name == *name);
    let parameter = parameters.next()?;
    if parameters.next().is_some() || !parameter.interface_bounds.is_empty() {
        return None;
    }
    let Some(SignatureType::Class(bound)) = parameter.class_bound.as_ref() else {
        return None;
    };
    if bound
        .segments
        .iter()
        .any(|segment| !segment.arguments.is_empty())
    {
        return None;
    }
    let bound_name = bound.segments.last()?.binary_name.as_slice();
    let Some((0, descriptor_name)) = descriptor_reference_shape(descriptor) else {
        return None;
    };
    (bound_name == descriptor_name.as_slice()).then_some(bound_name)
}

fn field_write_source_assignable(
    source: &SameClassFieldWriteSource,
    target: &SignatureType,
    class_scope: &[TypeParameterErasure],
    methods: &[SameClassPublishedMethodParameters],
    java_release: u16,
) -> bool {
    match source {
        SameClassFieldWriteSource::Null => true,
        SameClassFieldWriteSource::StringConstant { .. } => false,
        SameClassFieldWriteSource::RawAllocation { owner } => {
            if !matches!(target, SignatureType::Class(_)) || !class_type_arguments_present(target) {
                return false;
            }
            raw_reference_assignable(0, owner, target, java_release)
        }
        SameClassFieldWriteSource::Parameter { method, slot } => {
            let mut matching = methods
                .iter()
                .filter(|candidate| candidate.method == *method);
            let Some(method) = matching.next() else {
                return false;
            };
            if matching.next().is_some() {
                return false;
            }
            let Some(position) = method
                .parameter_slots
                .iter()
                .position(|candidate| candidate == slot)
            else {
                return false;
            };
            if let Some(parameters) = &method.generic_parameters {
                let Some(source) = parameters.get(position) else {
                    return false;
                };
                let Some(descriptor) = method.descriptor_parameters.get(position) else {
                    return false;
                };
                if signature_types_same_class_binding(
                    source,
                    target,
                    class_scope,
                    &method.method_type_parameters,
                ) {
                    return true;
                }
                if let Some(bound) = raw_method_type_variable_bound(source, method, descriptor)
                    && matches!(target, SignatureType::Class(_))
                    && class_type_arguments_present(target)
                {
                    return raw_reference_assignable(0, bound, target, java_release);
                }
                // A published Signature may still state a raw reference parameter, such as the
                // List parameter of an otherwise-generic method. Preserve that source shape as
                // the unchecked assignment it was; List<String> cannot enter this raw path.
                return !class_type_arguments_present(source)
                    && signature_raw_reference_matches_descriptor(source, descriptor)
                    && class_type_arguments_present(target)
                    && raw_parameter_assignable(descriptor, target, java_release);
            }
            method
                .descriptor_parameters
                .get(position)
                .is_some_and(|descriptor| {
                    // A raw reference (including a raw component in an array) can flow to its
                    // parameterized counterpart through Java's existing unchecked assignment.
                    class_type_arguments_present(target)
                        && raw_parameter_assignable(descriptor, target, java_release)
                })
        }
    }
}

fn prove_field_write_source_assignable(
    source: &SameClassFieldWriteSource,
    target: &SignatureType,
    class_scope: &[TypeParameterErasure],
    methods: &[SameClassPublishedMethodParameters],
    java_release: u16,
    budget: &mut Budget,
) -> Result<bool> {
    charge_field_write_source_proof(source, target, class_scope, methods, budget)?;
    budget.poll()?;
    Ok(field_write_source_assignable(
        source,
        target,
        class_scope,
        methods,
        java_release,
    ))
}

/// Whether one member's generic projection settled in this call, or is held for the class-level
/// same-class binding proof. A held candidate publishes nothing yet — no header, no refusal — so
/// the commit phase is the only place a half-decided member could appear, and it publishes whole
/// members through the same path a direct projection uses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SignatureProjection {
    /// This call decided the member: a header was projected, a refusal was written, or the member
    /// had no `Signature` to project at all.
    Settled,
    /// The member's pool entry names a same-class member and the use-site inventory is still
    /// being collected: the candidate is held back for the class-level proof.
    DeferredSameClass,
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
    physical_fields: &[MemberHeader],
    bytes: &[u8],
    pool: &[CpEntryFacts],
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    resolved_inner_classes: &[ResolvedInnerClass],
    same_class: SameClassBinding,
    unproved_this_delegate_bci: Option<u32>,
    budget: &mut Budget,
) -> Result<SignatureProjection> {
    let shells = attribute_shells(member, b"Signature");
    if shells.is_empty() {
        if class_flags & 0x4000 != 0
            && member.name.raw().0 == b"<init>"
            && member.descriptor.raw().0 == b"(Ljava/lang/String;ILjava/lang/String;)V"
        {
            record.enum_constructor_source_tail = EnumConstructorSourceTail::SingleString;
        }
        return project_member_inner_descriptor_path(record, candidate, budget)
            .map(|()| SignatureProjection::Settled);
    }
    let mut projection = record.take_generic_call_projection_state();
    let previous_constructor_tail = projection.enum_constructor_source_tail;
    let previous_constructor_erasure_refused =
        projection.enum_constructor_signature_erasure_refused;
    let result = project_method_signature_with_state(
        record,
        &mut projection,
        member,
        attributes,
        candidate,
        constructor_candidate,
        physical_fields,
        bytes,
        pool,
        class_internal,
        class_flags,
        class_superclass,
        class_interfaces,
        class_scope,
        class_signature_present,
        resolved_inner_classes,
        same_class,
        unproved_this_delegate_bci,
        &[],
        budget,
    );
    if result.is_err() {
        projection.enum_constructor_source_tail = previous_constructor_tail;
        projection.enum_constructor_signature_erasure_refused =
            previous_constructor_erasure_refused;
    }
    record.install_generic_call_projection_state(projection);
    result
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn project_method_signature_with_state(
    record: &ClassSourceMethod,
    projection: &mut GenericCallProjectionState,
    member: &MemberHeader,
    attributes: &MemberAttributes,
    candidate: Option<&GenericReturnCandidate>,
    constructor_candidate: Option<&GenericConstructorCandidate>,
    physical_fields: &[MemberHeader],
    bytes: &[u8],
    pool: &[CpEntryFacts],
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    resolved_inner_classes: &[ResolvedInnerClass],
    same_class: SameClassBinding,
    unproved_this_delegate_bci: Option<u32>,
    proved_list_parameters: &[(u16, Vec<String>)],
    budget: &mut Budget,
) -> Result<SignatureProjection> {
    let shells = attribute_shells(member, b"Signature");
    if shells.is_empty() {
        return Err(Error::unsupported(
            "generic_call_projection_state_requires_signature",
            "a generic-call overlay can only project a member with a Signature attribute",
        ));
    }
    /// The closure's own two answers: a projection to publish, or a same-class entry this call
    /// must hold for the class-level proof.
    enum MethodProjection {
        Deferred,
        Declaration(String, Vec<u8>, &'static str),
    }
    let result = (|| -> Result<MethodProjection> {
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
        projection.enum_constructor_source_tail =
            jarde_java::source_proof::enum_constructor_source_tail(
                shells.len(),
                name,
                &member.descriptor.raw().0,
                &parsed,
            );
        let erasure = prove_method_signature_erasure_with_class_scope(
            &parsed,
            &member.descriptor.raw().0,
            &attributes.throws_raw,
            class_scope,
            budget,
        )?;
        let no_body_generic = matches!(record.outcome, ClassSourceOutcome::NoBody)
            && !parsed.type_parameters.is_empty();
        let class_scope_constructor =
            jarde_java::generic_source_proof::class_scope_constructor_is_candidate(
                &name,
                &parsed,
                class_signature_present,
                class_scope,
                constructor_candidate,
                projection.same_class_generic_body_proof.as_ref(),
            );
        let generic_constructor =
            name == b"<init>" && (!parsed.type_parameters.is_empty() || class_scope_constructor);
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
        let body_ordinary_parameterized_null_return =
            jarde_java::generic_source_proof::ordinary_parameterized_null_return_is_proved(
                matches!(record.outcome, ClassSourceOutcome::Recovered { .. }),
                shells.len() == 1,
                !attributes.throws_raw.is_empty(),
                &member.descriptor.raw().0,
                member.access_flags,
                &parsed,
                candidate,
                class_internal,
                class_flags,
                class_superclass,
                class_interfaces,
                class_scope,
                class_signature_present,
            );
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(pool.len()).unwrap_or(u64::MAX),
        )?;
        if pool.iter().any(|entry| matches!(&entry.kind,
            CpEntryKind::MethodRef { owner, name: called, .. } | CpEntryKind::InterfaceMethodRef { owner, name: called, .. }
                if owner.0.as_slice() == class_internal && called.0.as_slice() == name.as_slice()
        )) {
            // The pool entry is a candidate fact, not a refusal: the class-level assembly holds
            // this member until its own decoded bodies state which members the entry's sites
            // really target. A proven binding falls through to the same shape proofs as an
            // unreferenced member; anything unproved keeps the existing refusal text.
            match same_class {
                SameClassBinding::Pending => {
                    return Ok(MethodProjection::Deferred);
                }
                SameClassBinding::Proven => {}
                SameClassBinding::OverloadPending => {
                    return Ok(MethodProjection::Deferred);
                }
                SameClassBinding::Unproved => {
                    return Err(Error::unsupported("generic_call_binding_unproved", "a same-class Methodref names this method or an adjacent overload"));
                }
            }
        }
        if class_scope_constructor && let Some(bci) = unproved_this_delegate_bci {
            return Err(Error::unsupported(
                "unproved_this_delegate_call",
                format!(
                    "same-class this-constructor delegation at BCI {bci} has no source argument adaptation proof"
                ),
            ));
        }
        let (declaration, proof) = if generic_constructor {
            generic_constructor_declaration(
                record,
                projection,
                attributes,
                &parsed,
                &erasure.type_parameters,
                constructor_candidate,
                projection.same_class_generic_body_proof.as_ref(),
                physical_fields,
                class_internal,
                class_flags,
                class_superclass,
                class_interfaces,
                class_scope,
                class_signature_present,
                budget,
            )?
        } else if no_body_generic {
            let declaration = no_body_generic_method_declaration(
                record,
                projection,
                attributes,
                &parsed,
                &erasure.type_parameters,
                class_scope,
                budget,
            )?;
            (declaration, "physical no-body declaration proof")
        } else if let Some(declaration) =
            matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
                .then(|| {
                    body_method_local_generic_throws_declaration(
                        record,
                        projection,
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
                    )
                })
                .flatten()
        {
            (
                declaration?,
                "same-run AST/Code/SSA empty-void and method-local Signature scope/erasure proof",
            )
        } else if let Some(declaration) =
            matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
                .then(|| {
                    generic_null_instance_method_declaration(
                        record,
                        projection,
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
                    )
                })
                .flatten()
        {
            (
                declaration?,
                "same-run AST/Code/SSA exact null-return and method-local Signature scope/erasure proof",
            )
        } else if let Some(declaration) =
            matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
                .then(|| {
                    generic_void_body_declaration(
                        record,
                        projection,
                        attributes,
                        &parsed,
                        candidate,
                        &erasure.type_parameters,
                        class_internal,
                        resolved_inner_classes,
                        budget,
                    )
                })
                .flatten()
        {
            (
                declaration?,
                "same-run complete AST/Code/SSA parameter-use and Signature erasure proof",
            )
        } else if body_ordinary_parameterized_null_return {
            ordinary_parameterized_declaration(
                record,
                Some(projection),
                attributes,
                &parsed,
                candidate,
                class_flags,
                class_internal,
                class_superclass,
                class_interfaces,
                class_scope,
                true,
                proved_list_parameters,
                budget,
            )?
        } else if let Some(declaration) =
            matches!(record.outcome, ClassSourceOutcome::Recovered { .. })
                .then(|| {
                    static_method_local_generic_throws_declaration(
                        record,
                        projection,
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
                    )
                })
                .flatten()
        {
            (
                declaration?,
                "same-run AST/SSA direct parameter return and method-local Signature scope/erasure proof",
            )
        } else if parsed.type_parameters.is_empty() {
            ordinary_parameterized_declaration(
                record,
                Some(projection),
                attributes,
                &parsed,
                candidate,
                class_flags,
                class_internal,
                class_superclass,
                class_interfaces,
                class_scope,
                false,
                proved_list_parameters,
                budget,
            )?
        } else if let Some(body_proof) = projection.same_class_generic_body_proof.as_ref() {
            (
                generic_method_body_declaration(
                    record,
                    projection,
                    attributes,
                    &parsed,
                    body_proof,
                    class_scope,
                    budget,
                )?,
                "same-run complete AST/Code/SSA formal and return-consumer proof",
            )
        } else {
            (
                generic_method_declaration(
                    record, projection, attributes, &parsed, candidate, false, None,
                )?,
                "same-run AST/SSA parameter-return proof",
            )
        };
        Ok(MethodProjection::Declaration(declaration, raw, proof))
    })();
    match result {
        Ok(MethodProjection::Deferred) => Ok(SignatureProjection::DeferredSameClass),
        Ok(MethodProjection::Declaration(declaration, signature, proof)) => {
            let proof = if same_class == SameClassBinding::Proven {
                format!("{proof}; same-class call binding proved")
            } else {
                proof.to_owned()
            };
            projection
                .project_generic(record, declaration, &signature, &proof, budget)
                .map(|()| SignatureProjection::Settled)
        }
        Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => Err(error),
        Err(error) => {
            projection.enum_constructor_signature_erasure_refused =
                projection.enum_constructor_source_tail.is_recognized()
                    && matches!(
                        &error,
                        Error::InvalidInput { code, .. }
                            if code == "jvm_signature_erasure_mismatch"
                    );
            projection
                .refuse_generic(record, &error.to_string(), budget)
                .map(|()| SignatureProjection::Settled)
        }
    }
}

pub(crate) fn staged_same_class_generic_method_contract_with_state(
    record: &ClassSourceMethod,
    state: &GenericCallProjectionState,
    member: &MemberHeader,
    attributes: &MemberAttributes,
    bytes: &[u8],
    pool: &[CpEntryFacts],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<Option<SameClassGenericMethodContract>> {
    staged_same_class_generic_method_contract_facts(
        record,
        state.generic_signature_projected,
        state.declaration.is_some(),
        member,
        attributes,
        bytes,
        pool,
        class_scope,
        budget,
    )
}

fn staged_same_class_generic_method_contract_facts(
    record: &ClassSourceMethod,
    generic_signature_projected: bool,
    declaration_present: bool,
    member: &MemberHeader,
    attributes: &MemberAttributes,
    bytes: &[u8],
    pool: &[CpEntryFacts],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<Option<SameClassGenericMethodContract>> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if !generic_signature_projected
        || record.item.identity.name.0 != member.name.raw().0
        || record.item.identity.descriptor.0 != member.descriptor.raw().0
        || !declaration_present
        || !matches!(
            &record.outcome,
            ClassSourceOutcome::Recovered { .. } | ClassSourceOutcome::NoBody
        )
    {
        return Ok(None);
    }
    same_class_generic_method_contract_from_signature(
        record.item.identity.clone(),
        member,
        attributes,
        bytes,
        pool,
        class_scope,
        budget,
    )
}

pub(crate) fn same_class_generic_method_contract_from_signature(
    method: PhysicalMethodId,
    member: &MemberHeader,
    attributes: &MemberAttributes,
    bytes: &[u8],
    pool: &[CpEntryFacts],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<Option<SameClassGenericMethodContract>> {
    let shells = attribute_shells(member, b"Signature");
    if shells.len() != 1 {
        return Ok(None);
    }
    let facts = attribute_facts(bytes, &shells, pool, budget)?;
    let Some(raw) = facts.signature.map(|signature| signature.0) else {
        return Ok(None);
    };
    let parsed = parse_method_signature(&raw, budget)?;
    prove_method_signature_erasure_with_class_scope(
        &parsed,
        &member.descriptor.raw().0,
        &attributes.throws_raw,
        class_scope,
        budget,
    )?;
    Ok(Some(SameClassGenericMethodContract {
        method,
        method_parameters: parsed.type_parameters,
        parameters: parsed.parameters,
        result: parsed.result,
    }))
}

/// Project the two method Signatures that close the first generic enclosing-member family slice.
/// The caller has already proved the physical root/child relation, capture and complete call
/// closure. This function supplies the root's published Signature scope to the existing method
/// erasure and source-type spelling machinery; it does not infer a type from emitted text.
pub(crate) fn project_member_family_generic_signature(
    record: &mut ClassSourceMethod,
    member: &MemberHeader,
    attributes: &MemberAttributes,
    bytes: &[u8],
    pool: &[CpEntryFacts],
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    target: &jarde_java::report::MemberInnerTargetInput,
    root_return: bool,
    budget: &mut Budget,
) -> Result<()> {
    let shells = attribute_shells(member, b"Signature");
    jarde_java::generic_source_proof::prove_member_family_signature_source_slice(
        shells.len(),
        class_scope,
        record.markers.len(),
        record.generic_signature_refused,
        !record.annotations.attributes.is_empty()
            || !record.parameter_annotations.attributes.is_empty()
            || !record.type_annotations.attributes.is_empty(),
    )?;
    let raw = attribute_facts(bytes, &shells, pool, budget)?
        .signature
        .ok_or_else(|| {
            Error::invalid_input(
                "jvm_signature_missing",
                "Signature attribute did not resolve",
            )
        })?
        .0;
    let parsed = parse_method_signature(&raw, budget)?;
    jarde_java::generic_source_proof::prove_member_family_method_scope(&parsed)?;
    prove_method_signature_erasure_with_class_scope(
        &parsed,
        &member.descriptor.raw().0,
        &attributes.throws_raw,
        class_scope,
        budget,
    )?;

    let proof = record.same_class_generic_body_proof.as_ref();
    let (declaration, classification) =
        jarde_java::generic_source_proof::member_family_generic_declaration(
            &member.name.raw().0,
            &member.descriptor.raw().0,
            record.item.access_flags,
            record.declaration.is_some(),
            matches!(record.outcome, ClassSourceOutcome::Recovered { .. }),
            matches!(record.outcome, ClassSourceOutcome::NoBody),
            record.no_body_kind.is_some(),
            matches!(
                record.item.body,
                crate::MemberBodyEvidence::CodeAttribute { .. }
            ),
            !record.annotations.attributes.is_empty()
                || !record.parameter_annotations.attributes.is_empty(),
            !record.annotations.refusals.is_empty()
                || !record.parameter_annotations.refusals.is_empty()
                || !record.type_annotations.refusals.is_empty(),
            !record.type_annotations.attributes.is_empty(),
            &record.parameter_annotations.uses_by_position,
            attributes.default.is_some(),
            &attributes.throws,
            !attributes.throws_raw.is_empty(),
            &parsed,
            record.same_run_generic_return.as_ref(),
            proof,
            proof.is_none_or(|proof| proof.method == record.item.identity),
            class_flags,
            class_internal,
            class_superclass,
            class_interfaces,
            &class_scope[0],
            target,
            root_return,
            budget,
        )?;
    // `record` is a family-writer clone. Drop only the independently recorded scope refusal;
    // `project_generic` replaces it with the successful family-scope note below.
    record.markers.clear();
    record.project_generic(declaration, &raw, classification, budget)
}

/// Spell one generic constructor only after its same-run body candidate, Signature erasure, and
/// the class-level call-binding gates have all passed. The complete declaration is built before
/// `project_generic` publishes any part of it.
#[allow(clippy::too_many_arguments)]
fn generic_constructor_declaration(
    record: &ClassSourceMethod,
    projection: &GenericCallProjectionState,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    method_scope: &[TypeParameterErasure],
    candidate: Option<&GenericConstructorCandidate>,
    body_proof: Option<&SameClassGenericMethodBodyProof>,
    physical_fields: &[MemberHeader],
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Result<(String, &'static str)> {
    jarde_java::generic_source_proof::generic_constructor_declaration(
        &record.item.name.raw().0,
        &record.item.descriptor.raw().0,
        record.item.access_flags,
        projection.declaration.is_some(),
        matches!(record.outcome, ClassSourceOutcome::Recovered { .. }),
        projection.markers.is_empty(),
        attributes.default.is_some(),
        !attributes.throws_raw.is_empty(),
        !record.annotations.attributes.is_empty()
            || !record.parameter_annotations.attributes.is_empty()
            || !record.type_annotations.attributes.is_empty(),
        !record.annotations.refusals.is_empty()
            || !record.parameter_annotations.refusals.is_empty()
            || !record.type_annotations.refusals.is_empty(),
        parsed,
        method_scope,
        candidate,
        body_proof,
        body_proof.is_some_and(|proof| proof.method == record.item.identity),
        physical_fields,
        class_internal,
        class_flags,
        class_superclass,
        class_interfaces,
        class_scope,
        class_signature_present,
        budget,
    )
}

/// The initial no-body projection proves the class has no inherited source contract to satisfy.
/// `$` in a binary name is conservatively treated as nested because this projection receives no
/// selected InnerClasses nesting proof.
fn prove_no_body_generic_hierarchy(
    record: &ClassSourceMethod,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    budget: &mut Budget,
) -> Result<()> {
    jarde_java::generic_source_proof::prove_no_body_generic_hierarchy(
        record.no_body_kind,
        &record.item.name.raw().0,
        class_internal,
        class_flags,
        class_superclass,
        class_interfaces,
        budget,
    )
}

/// Re-spell descriptor positions only when the same-run body returned the selected member
/// creation. This is the raw-signature slice: the body's target and qualifier slot constrain every
/// replacement, and the physical descriptor still controls all other positions.
fn project_member_inner_descriptor_path(
    record: &mut ClassSourceMethod,
    candidate: Option<&GenericReturnCandidate>,
    budget: &mut Budget,
) -> Result<()> {
    let Some(candidate) = candidate else {
        return Ok(());
    };
    let (source_type_path, target_owner, qualifier_slot) = match &candidate.value {
        GenericReturnValue::MemberCreation {
            target,
            qualifier_slot,
        } => (
            &target.source_type_path,
            &target.owner,
            Some(*qualifier_slot),
        ),
        GenericReturnValue::StaticMemberCreation { target, .. } => {
            (&target.source_type_path, &target.owner, None)
        }
        _ => return Ok(()),
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
    if candidate.parameters.len() != signature.parameters.len() {
        return Err(refused(
            "same-run parameter positions differ from the descriptor",
        ));
    }
    if let Some(qualifier_slot) = qualifier_slot {
        let Some((qualifier_type, _)) = signature
            .parameters
            .iter()
            .find(|(_, slot)| *slot == qualifier_slot)
        else {
            return Err(refused("member qualifier is not a method parameter slot"));
        };
        let Some(outer) = source_type_path.iter().find(|segment| {
            Some(&segment.binary_name)
                == source_type_path
                    .last()
                    .and_then(|segment| segment.enclosing_binary_name.as_ref())
        }) else {
            return Err(refused("selected source path has no enclosing member type"));
        };
        let Some(member) = source_type_path
            .iter()
            .find(|segment| &segment.binary_name == target_owner)
        else {
            return Err(refused("selected source path has no target member type"));
        };
        if qualifier_type != &binary_pool_class_name(&outer.binary_name.0)?
            || member.enclosing_binary_name.as_ref() != Some(&outer.binary_name)
            || member.is_static
            || !outer.is_static
            || source_type_path.last().is_none_or(|last| last != member)
        {
            return Err(refused(
                "selected qualifier, enclosing type, and member path do not agree",
            ));
        }
    }
    if let Some(result) = &signature.returns
        && result != "java.lang.Object"
        && result != &binary_pool_class_name(&target_owner.0)?
    {
        return Err(refused(
            "physical result is neither Object nor the selected member type",
        ));
    }
    for ((_, descriptor_slot), (candidate_slot, name)) in
        signature.parameters.iter().zip(&candidate.parameters)
    {
        if descriptor_slot != candidate_slot || !is_java_identifier(name) {
            return Err(refused("same-run parameter names or slots are incomplete"));
        }
    }

    let mut declaration = record.declaration.clone().expect("checked above");
    let mut mapped_segments = source_type_path.iter().collect::<Vec<_>>();
    mapped_segments.sort_by_key(|segment| std::cmp::Reverse(segment.binary_name.0.len()));
    let mut changed = false;
    for segment in mapped_segments {
        let physical_name = binary_pool_class_name(&segment.binary_name.0)?;
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
        let source_name = match &candidate.value {
            GenericReturnValue::StaticMemberCreation { target, .. }
                if segment.binary_name == target.owner =>
            {
                target.simple_name.as_str()
            }
            _ => segment.source_name.as_str(),
        };
        let (rewritten, count) = replace_java_type_token(&declaration, &physical_name, source_name);
        // The declaration writer may already have used the same proved source path. Count
        // whole type tokens only: a containing type must not match the prefix of a member type.
        let already_projected = if physical_name != source_name {
            replace_java_type_token(&declaration, source_name, source_name).1
        } else {
            0
        };
        if count + already_projected != expected_count {
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
        jarde_java::facts::display_jvm_bytes(target_owner)
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
    ty.trim_end_matches("[]")
}

fn replace_java_type_token(text: &str, from: &str, to: &str) -> (String, usize) {
    let is_identifier = |byte: u8| {
        byte.is_ascii_alphanumeric() || byte >= 0x80 || matches!(byte, b'_' | b'$' | b'.')
    };
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
        let ellipsis_boundary = text[end..].strip_prefix("...").is_some_and(|tail| {
            tail.as_bytes()
                .first()
                .is_none_or(|byte| !is_identifier(*byte))
        });
        let right_is_boundary =
            end == bytes.len() || !is_identifier(bytes[end]) || ellipsis_boundary;
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

#[allow(clippy::too_many_arguments)]
fn body_method_local_generic_throws_declaration(
    record: &ClassSourceMethod,
    projection: &GenericCallProjectionState,
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
) -> Option<Result<String>> {
    jarde_java::generic_source_proof::body_method_local_generic_throws_declaration(
        &record.item.name.raw().0,
        &record.item.descriptor.raw().0,
        record.item.access_flags,
        projection.declaration.is_some(),
        !projection.markers.is_empty(),
        !record.annotations.refusals.is_empty()
            || !record.parameter_annotations.refusals.is_empty()
            || !record.type_annotations.refusals.is_empty(),
        !record.type_annotations.attributes.is_empty(),
        attributes.default.is_some(),
        parsed,
        candidate,
        class_internal,
        class_flags,
        class_superclass,
        class_interfaces,
        class_scope,
        class_signature_present,
        budget,
    )
}

#[allow(clippy::too_many_arguments)]
fn static_method_local_generic_throws_declaration(
    record: &ClassSourceMethod,
    projection: &GenericCallProjectionState,
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
) -> Option<Result<String>> {
    jarde_java::generic_source_proof::static_method_local_generic_throws_declaration(
        &record.item.name.raw().0,
        &record.item.descriptor.raw().0,
        record.item.access_flags,
        projection.declaration.is_some(),
        !record.annotations.attributes.is_empty()
            || !record.parameter_annotations.attributes.is_empty()
            || !record.type_annotations.attributes.is_empty(),
        !projection.markers.is_empty(),
        &attributes.throws,
        parsed,
        candidate,
        class_internal,
        class_flags,
        class_superclass,
        class_interfaces,
        class_scope,
        class_signature_present,
        budget,
    )
}

fn no_body_generic_method_declaration(
    record: &ClassSourceMethod,
    projection: &GenericCallProjectionState,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    method_scope: &[TypeParameterErasure],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<String> {
    jarde_java::generic_source_proof::no_body_generic_method_declaration(
        &record.item.name.raw().0,
        &record.item.descriptor.raw().0,
        record.item.access_flags,
        projection.declaration.is_some(),
        record.no_body_kind,
        attributes.default.is_some(),
        !record.annotations.refusals.is_empty()
            || !record.parameter_annotations.refusals.is_empty()
            || !record.type_annotations.refusals.is_empty(),
        !record.type_annotations.attributes.is_empty(),
        &record.parameter_annotations.uses_by_position,
        &attributes.throws,
        parsed,
        method_scope,
        class_scope,
        budget,
    )
}

/// Spell ordinary parameterized method types only after the reader has proved their erasures.
/// A recovered body's candidate is the same Program/SSA reading that supplied its parameter
/// names; no-body members have no Program and use descriptor slot names.
fn ordinary_parameterized_declaration(
    record: &ClassSourceMethod,
    projection: Option<&GenericCallProjectionState>,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_flags: u16,
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    allow_null_return: bool,
    proved_list_parameters: &[(u16, Vec<String>)],
    budget: &mut Budget,
) -> Result<(String, &'static str)> {
    let proof = projection
        .and_then(|state| state.same_class_generic_body_proof.as_ref())
        .or(record.same_class_generic_body_proof.as_ref());
    jarde_java::generic_source_proof::ordinary_parameterized_declaration(
        &record.item.name.raw().0,
        &record.item.descriptor.raw().0,
        record.item.access_flags,
        projection.map_or(record.declaration.is_some(), |state| {
            state.declaration.is_some()
        }),
        matches!(record.outcome, ClassSourceOutcome::Recovered { .. }),
        matches!(record.outcome, ClassSourceOutcome::NoBody),
        record.no_body_kind.is_some(),
        matches!(
            record.item.body,
            crate::MemberBodyEvidence::CodeAttribute { .. }
        ),
        projection.map_or(record.markers.is_empty(), |state| state.markers.is_empty()),
        !record.annotations.attributes.is_empty()
            || !record.parameter_annotations.attributes.is_empty(),
        !record.annotations.refusals.is_empty()
            || !record.parameter_annotations.refusals.is_empty()
            || !record.type_annotations.refusals.is_empty(),
        !record.type_annotations.attributes.is_empty(),
        &record.parameter_annotations.uses_by_position,
        attributes.default.is_some(),
        &attributes.throws,
        !attributes.throws_raw.is_empty(),
        parsed,
        candidate,
        proof,
        proof.is_none_or(|proof| proof.method == record.item.identity),
        class_flags,
        class_internal,
        class_superclass,
        class_interfaces,
        class_scope,
        allow_null_return,
        proved_list_parameters,
        budget,
    )
}

fn generic_void_body_declaration(
    record: &ClassSourceMethod,
    projection: &GenericCallProjectionState,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    erasures: &[TypeParameterErasure],
    class_internal: &[u8],
    resolved_inner_classes: &[ResolvedInnerClass],
    budget: &mut Budget,
) -> Option<Result<String>> {
    jarde_java::generic_source_proof::generic_void_body_declaration(
        &record.item.name.raw().0,
        &record.item.descriptor.raw().0,
        record.item.access_flags,
        projection.declaration.is_some(),
        !record.annotations.attributes.is_empty()
            || !record.parameter_annotations.attributes.is_empty()
            || !record.type_annotations.attributes.is_empty(),
        !projection.markers.is_empty(),
        &attributes.throws,
        !attributes.throws_raw.is_empty(),
        parsed,
        candidate,
        erasures,
        class_internal,
        resolved_inner_classes,
        budget,
    )
}

fn generic_method_body_declaration(
    record: &ClassSourceMethod,
    projection: &GenericCallProjectionState,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    proof: &SameClassGenericMethodBodyProof,
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<String> {
    if proof.method != record.item.identity {
        return Err(Error::unsupported(
            "generic_source_shape_unproved",
            "body certificate does not match a preservable generic method declaration",
        ));
    }
    jarde_java::generic_source_proof::generic_method_body_declaration(
        &record.item.name.raw().0,
        &record.item.descriptor.raw().0,
        record.item.access_flags,
        !record.annotations.attributes.is_empty()
            || !record.parameter_annotations.attributes.is_empty()
            || !record.type_annotations.attributes.is_empty(),
        !projection.markers.is_empty(),
        &attributes.throws,
        parsed,
        proof,
        class_scope,
        budget,
    )
}

fn generic_method_declaration(
    record: &ClassSourceMethod,
    projection: &GenericCallProjectionState,
    attributes: &MemberAttributes,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    method_local_throws_proved: bool,
    first_bound_spelling: Option<&str>,
) -> Result<String> {
    jarde_java::generic_source_proof::generic_method_declaration(
        &record.item.name.raw().0,
        &record.item.descriptor.raw().0,
        record.item.access_flags,
        projection.declaration.is_some(),
        !record.annotations.attributes.is_empty()
            || !record.parameter_annotations.attributes.is_empty()
            || !record.type_annotations.attributes.is_empty(),
        !projection.markers.is_empty(),
        &attributes.throws,
        parsed,
        candidate,
        method_local_throws_proved,
        first_bound_spelling,
    )
}

#[allow(clippy::too_many_arguments)]
fn generic_null_instance_method_declaration(
    record: &ClassSourceMethod,
    projection: &GenericCallProjectionState,
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
) -> Option<Result<String>> {
    jarde_java::generic_source_proof::generic_null_instance_method_declaration(
        &record.item.name.raw().0,
        &record.item.descriptor.raw().0,
        record.item.access_flags,
        projection.declaration.is_some(),
        !record.annotations.attributes.is_empty()
            || !record.parameter_annotations.attributes.is_empty()
            || !record.type_annotations.attributes.is_empty(),
        !projection.markers.is_empty(),
        &attributes.throws,
        !attributes.throws_raw.is_empty(),
        parsed,
        candidate,
        class_internal,
        class_flags,
        class_superclass,
        class_interfaces,
        class_scope,
        class_signature_present,
        budget,
    )
}

/// The initializer one field's own `ConstantValue` attribute states (JVMS 4.7.2), as the field's
/// declaration writes it, or `None` when the field's declaration states none.
///
/// The read is [`attribute_facts`] over this field's own `ConstantValue` shell(s) — the reader's one
/// parser for that attribute, and only those shells are handed over, so the `attribute_bytes` charge
/// is exactly this attribute's — and nothing here reads a `<clinit>`, a use site or another member:
/// the initializer is the constant the field's own attribute declares.
///
/// `None` means the attribute is absent. An unsupported literal or damaged attribute returns
/// its error; the caller retains this physical field with a local refusal marker. Budget and
/// cancellation still stop the shared request before any further member work.
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
    let index = attribute_facts(bytes, &shells, pool, budget)?
        .constant_value
        .ok_or_else(|| {
            Error::invalid_input(
                "classfile_invalid_attribute_content",
                "declared ConstantValue has no decoded index",
            )
        })?;
    resolve_constant_value(&field.descriptor.raw().0, index.0, pool).map(Some)
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
/// `default` is the already resolved Java value of this member's `AnnotationDefault`
/// (JVMS 4.7.22), written after its parameter list. The caller resolves every present raw
/// value before reaching this spelling path and publishes a refusal if it is unsupported;
/// an absent value alone means no `default` clause. Which class declares the member is not
/// read here: its attribute supplies the value for an annotation, interface or class member.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spell_method(
    item: &MethodItem,
    facts: Option<&RecoveryFacts>,
    recovered_parameter_names: Option<&[Option<String>]>,
    class: &str,
    source_names: SourceClassNames<'_>,
    class_flags: u16,
    attributes: Option<&MemberAttributes>,
    default: Option<&MemberDefault>,
    annotations: &MemberAnnotationFacts,
    pool: &[CpEntryFacts],
) -> Spelled {
    let signature = method_descriptor(
        &item.descriptor.raw().0,
        is_static(item.access_flags),
        item.access_flags & ACC_VARARGS != 0,
    );
    let source_names_valid = signature.as_ref().is_some_and(|signature| {
        jarde_java::declaration::method_signature_source_paths_valid(signature, source_names)
    });
    let parameter_count = signature
        .as_ref()
        .map(|signature| signature.parameters.len());
    let mut member_annotations =
        spell_member_annotation_uses(annotations.declaration.clone(), pool, source_names);
    let mut parameter_annotations = spell_parameter_annotation_uses(
        annotations.parameters.clone(),
        pool,
        parameter_count,
        source_names,
    );
    let type_annotations = spell_type_annotation_uses(
        annotations.type_uses.clone(),
        TypeUseOwner::Method,
        &item.descriptor.raw().0,
        pool,
        &member_annotations,
        Some(&annotations.parameters),
        parameter_count,
        source_names,
    );
    let (declaration, marker) = jarde_java::declaration::spell_method_declaration(
        &item.name,
        &item.descriptor,
        item.access_flags,
        matches!(item.body, crate::MemberBodyEvidence::CodeAttribute { .. }),
        facts,
        recovered_parameter_names,
        class,
        source_names,
        class_flags,
        attributes.map_or(&[][..], |attributes| attributes.throws.as_slice()),
        default,
        &mut member_annotations,
        &mut parameter_annotations,
        &type_annotations.return_uses,
        &type_annotations.parameter_uses,
        attributes.and_then(|attributes| attributes.parameters.as_deref()),
        signature.is_some(),
        source_names_valid,
    );
    let mut spelled = Spelled {
        declaration,
        marker,
        annotations: MemberAnnotationUses::default(),
        parameter_annotations: ParameterAnnotationUses::default(),
        type_annotations: TypeAnnotationUses::default(),
    };
    spelled.annotations = member_annotations;
    spelled.parameter_annotations = parameter_annotations;
    spelled.type_annotations = type_annotations;
    spelled
}

/// Spells one field's declaration, and the marker its own spelling needs.
///
/// `constant` is the value the field's own `ConstantValue` attribute (JVMS 4.7.2) states, as the
/// read that resolved it published it: a field with one is written `<declaration> = <literal>`, and
/// a field that declares no such attribute — or one whose value this presentation has no literal
/// for — is written without an initializer at all. Nothing is inferred from a `<clinit>` or a use
/// site, so the one `=` this presentation writes is the one the field's own attribute states.
fn spell_field_declaration(
    item: &FieldItem,
    source_names: SourceClassNames<'_>,
    constant: Option<&MemberDefault>,
) -> Spelled {
    let (declaration, marker) = jarde_java::declaration::spell_field_declaration(
        &item.name,
        &item.descriptor,
        item.access_flags,
        source_names,
        constant,
    );
    Spelled {
        declaration,
        marker,
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
    jarde_java::declaration::field_declaration_with_type(
        &item.name,
        item.access_flags,
        ty,
        constant,
    )
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
    class_header_projected: bool,
    constant: Option<&MemberDefault>,
    same_class: SameClassBinding,
    same_class_note: &str,
    same_class_field_uses: &[SameClassFieldUse],
    published_method_parameters: &[SameClassPublishedMethodParameters],
    java_release: u16,
    budget: &mut Budget,
) -> Result<SignatureProjection> {
    let shells = attribute_shells(member, b"Signature");
    if shells.is_empty() {
        return Ok(SignatureProjection::Settled);
    }
    enum FieldProjection {
        Deferred,
        Declaration(String, Vec<u8>),
    }
    let result = (|| -> Result<FieldProjection> {
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

        if same_class == SameClassBinding::Proven {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(same_class_field_uses.len()).unwrap_or(u64::MAX),
            )?;
            for use_site in same_class_field_uses {
                budget.poll()?;
                if use_site.owner != class_internal
                    || use_site.name != member.name.raw().0
                    || use_site.descriptor != member.descriptor.raw().0
                    || !matches!(use_site.opcode, 0xb3 | 0xb5)
                {
                    continue;
                }
                let Some(source) = use_site.write_source.as_ref() else {
                    return Err(Error::unsupported(
                        "field_generic_write_source_unproved",
                        format!(
                            "writer {}@{} has no closed SSA source",
                            use_site.caller, use_site.bci
                        ),
                    ));
                };
                let raw_receiver = raw_field_receiver_source(
                    use_site,
                    class_internal,
                    class_header_projected && !class_scope.is_empty(),
                    published_method_parameters,
                    budget,
                )?;
                let source_complete = raw_receiver
                    || matches!(source, SameClassFieldWriteSource::RawAllocation { .. });
                let assignable = if raw_receiver {
                    raw_field_write_source_assignable(
                        source,
                        &use_site.descriptor,
                        published_method_parameters,
                        java_release,
                        budget,
                    )?
                } else {
                    prove_field_write_source_assignable(
                        source,
                        &parsed.ty,
                        class_scope,
                        published_method_parameters,
                        java_release,
                        budget,
                    )?
                };
                if !assignable || (source_complete && !use_site.source_complete) {
                    return Err(Error::unsupported(
                        "field_generic_write_source_unproved",
                        format!(
                            "writer {}@{} is not source-assignable to the projected field type",
                            use_site.caller, use_site.bci
                        ),
                    ));
                }
            }
        }

        jarde_java::generic_source_proof::prove_field_signature_source(
            record.declaration.is_some(),
            record.markers.is_empty(),
            &record.item.name.raw().0,
            record.item.access_flags,
            class_internal,
            class_header_projected,
            class_flags,
            !record.type_annotations.attributes.is_empty(),
            !record.type_annotations.refusals.is_empty(),
        )?;

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
            // As with methods, the entry is a candidate fact: the class-level inventory of this
            // class's own decoded bodies decides whether a real read/write site keeps accepting the
            // parameterized type. Unproved sites keep the existing refusal.
            match same_class {
                SameClassBinding::Pending => {
                    return Ok(FieldProjection::Deferred);
                }
                SameClassBinding::Proven => {}
                SameClassBinding::OverloadPending => {
                    return Err(Error::unsupported(
                        "field_generic_binding_unproved",
                        "overload-pending binding applies only to methods",
                    ));
                }
                SameClassBinding::Unproved => {
                    return Err(Error::unsupported(
                        "field_generic_body_unproved",
                        format!(
                            "a same-class Fieldref names this field and descriptor; {same_class_note}"
                        ),
                    ));
                }
            }
        }

        let ty = spell_ordinary_signature_type(&parsed.ty, class_scope, budget, 0)?;
        Ok(FieldProjection::Declaration(
            field_declaration_with_type(&record.item, &ty, constant),
            raw,
        ))
    })();
    match result {
        Ok(FieldProjection::Deferred) => Ok(SignatureProjection::DeferredSameClass),
        Ok(FieldProjection::Declaration(declaration, signature)) => {
            let marker = format!(
                "// jarde: field Signature `{}` projected after descriptor erasure and {}",
                comment_text(&String::from_utf8_lossy(&signature)),
                if same_class == SameClassBinding::Proven {
                    comment_text(same_class_note)
                } else {
                    "no same-class Fieldref".to_owned()
                },
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
            record.generic_signature_projected = true;
            Ok(SignatureProjection::Settled)
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
            Ok(SignatureProjection::Settled)
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The text: one class, its members, and the blocks the recovery layer wrote
// ---------------------------------------------------------------------------------------------

// Shared Java writer; the physical records and final publication remain here.
use jarde_java::source_proof::{
    Artifact, Placed, artifact, block_member, block_member_text_bytes, comment_name,
    declaration_member, declaration_member_text_bytes, indent, placed_artifact_span,
};

/// One field's written form with a projection's own declaration: the same markers the physical
/// member carries, beside the re-spelled declaration a static member fold states.
pub(crate) fn field_member_text(field: &ClassSourceField, declaration: &str) -> String {
    declaration_member(declaration, &field.markers)
}

/// One member written as its markers alone: nothing else about it can be spelled.
fn comment_member(markers: &[String]) -> String {
    jarde_java::source_proof::comment_member(markers)
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
    markers.extend(run_markers(item, report, analysis));
    markers
}

/// The markers one run's own verdict adds over a spelling: the run's content verdict, and what
/// stopped either of its planes. [`recovered_markers`] writes them after the spelling's own
/// markers; [`ClassSourceMethod::with_rerun_body`] writes them over a body a rerun replaced.
fn run_markers(
    item: &MethodItem,
    report: &RecoveryReport,
    analysis: &ClassSourceRunFacts,
) -> Vec<String> {
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
        return vec![format!(
            "// jarde: not recovered: the recovery run for `{member}` stopped ({stop}){blamed}"
        )];
    }
    let mut markers = Vec::new();
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
    pub(crate) fn of(item: ClassDeclarationItem) -> Result<Self> {
        let (name, declaration) = jarde_java::declaration::class_source_declaration(
            &item.declaration.this_class,
            item.declaration.access_flags,
            item.declaration.super_class.as_ref(),
            &item.declaration.interfaces,
        )?;
        Ok(Self {
            item,
            name,
            declaration,
            annotation_attributes: Vec::new(),
            annotation_uses: Vec::new(),
            annotation_refusals: Vec::new(),
            generic_signature: None,
            generic_refusal: None,
            generic_type_parameters: None,
            generic_scope: None,
            header_interface_arguments: Vec::new(),
        })
    }

    /// Project the class's own Signature only after its parent identities and variable erasures
    /// agree with the physical header — and, for a `$`-nested name, after its own
    /// `InnerClasses` self row proves the member position (the fold's row criteria). The
    /// returned scope is handed to member signatures only when the complete class header was
    /// published.
    pub(crate) fn project_generic_signature(
        &mut self,
        bytes: &[u8],
        shells: &[AttributeShell],
        pool: &[CpEntryFacts],
        nesting: &ClassSourceAssemblyContext,
        budget: &mut Budget,
        mut prove_header_parent: impl FnMut(
            ClassHeaderParentPosition,
            &[u8],
            usize,
            &mut Budget,
        ) -> Result<ClassHeaderParentProof>,
    ) -> Result<Option<ClassSignatureErasureProof>> {
        let signatures: Vec<_> = shells
            .iter()
            .filter(|shell| shell.name.raw().0 == b"Signature")
            .cloned()
            .collect();
        if signatures.is_empty() {
            return Ok(None);
        }
        let result = (|| -> Result<Option<(String, ClassSignatureErasureProof, Vec<Vec<u8>>)>> {
            let raw = attribute_facts(bytes, &signatures, pool, budget)?
                .signature
                .ok_or_else(|| {
                    Error::invalid_input(
                        "jvm_signature_missing",
                        "class Signature attribute did not resolve",
                    )
                })?;
            let parsed = parse_class_signature(&raw.0, budget)?;
            let Some(shape) =
                jarde_java::generic_source_proof::class_signature_source_shape(&parsed)
            else {
                return Ok(None);
            };
            self.generic_signature = Some(raw.clone());
            let facts = &self.item.declaration;
            let (_, _, interface_only) = shape;
            if !jarde_java::generic_source_proof::class_signature_source_header_is_eligible(
                &self.name,
                facts.access_flags,
                shells,
                interface_only,
            )? {
                return Ok(None);
            }
            // A `$` in the class's own name states a nesting claim the header cannot spell on
            // its own: only the class's own InnerClasses self row proves the member position
            // (the same row criteria the member fold requires of a child). The refusal keeps
            // the chain head's exact code and message, so a row that does not join, a local or
            // anonymous shape, or a synthetic `$`-spelled top-level class stays blocked.
            if self.name.contains('$')
                && !crate::member_inner::prove_nested_member_position(
                    &facts.this_class.raw().0,
                    facts.access_flags,
                    nesting,
                    budget,
                )?
            {
                return jarde_java::generic_source_proof::class_signature_own_header_refusal(
                    interface_only,
                    "class kind, nesting, name, or type-use annotations lack a faithful generic header position",
                );
            }
            let physical_super = match facts.super_class.as_ref() {
                Some(physical_super) => physical_super,
                None => {
                    return jarde_java::generic_source_proof::class_signature_own_header_refusal(
                        interface_only,
                        "physical superclass is absent",
                    );
                }
            };
            if facts.access_flags & ACC_INTERFACE != 0
                && physical_super.raw().0 != b"java/lang/Object"
            {
                return jarde_java::generic_source_proof::class_signature_own_header_refusal(
                    interface_only,
                    "interface physical superclass is not java/lang/Object",
                );
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
            let Some((declaration, type_parameters, header_interface_arguments)) =
                jarde_java::generic_source_proof::project_class_signature_source_header(
                    parsed,
                    &proof,
                    shape,
                    &self.name,
                    &facts.this_class,
                    facts.access_flags,
                    physical_super,
                    &facts.interfaces,
                    &physical_interfaces,
                    budget,
                    &mut prove_header_parent,
                )?
            else {
                return Ok(None);
            };
            budget.charge(
                CountedBudgetDimension::OutputBytes,
                u64::try_from(declaration.len()).unwrap_or(u64::MAX),
            )?;
            // Recorded only past the last fallible step of the closure, so a stop never
            // leaves a carried header beside a refusal.
            self.generic_type_parameters = type_parameters;
            Ok(Some((declaration, proof, header_interface_arguments)))
        })();
        match result {
            Ok(Some((declaration, proof, header_interface_arguments))) => {
                self.declaration = declaration;
                self.generic_scope = Some(proof.clone());
                self.header_interface_arguments = header_interface_arguments;
                Ok(Some(proof))
            }
            Ok(None) => {
                // Nothing was published, so no entry may claim a header that carries type
                // arguments: the assignment above only carries the raw attribute for the paths
                // below it, and a gate that answered `Ok(None)` (the total gate, or an
                // interface-only class whose own header gates did not pass) leaves the physical
                // declaration exactly as it was.
                self.generic_signature = None;
                self.header_interface_arguments = Vec::new();
                Ok(None)
            }
            Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => {
                self.generic_refusal = Some(format!("projection stopped: {error}"));
                self.header_interface_arguments = Vec::new();
                Err(error)
            }
            Err(error) => {
                self.generic_refusal = Some(error.to_string());
                self.header_interface_arguments = Vec::new();
                Ok(None)
            }
        }
    }

    pub(crate) fn with_annotations(
        mut self,
        attributes: Vec<ClassAnnotationAttribute>,
        pool: &[CpEntryFacts],
        source_names: SourceClassNames<'_>,
    ) -> Self {
        jarde_java::metadata_literal::spell_class_annotation_uses(
            &attributes,
            pool,
            source_names,
            &mut self.annotation_uses,
            &mut self.annotation_refusals,
        );
        self.annotation_attributes = attributes;
        self
    }
}

/// Supply the selected physical facts to the Java package-info source certificate.
pub(crate) fn proved_package_info_source(
    declaration: &ClassSourceDeclaration,
    facts: &ClassMemberFacts,
    class_version: (u16, u16),
    structure_complete: bool,
    execution: &ExecutionReport,
) -> std::result::Result<String, &'static str> {
    jarde_java::metadata_literal::proved_package_info_source(
        facts,
        class_version,
        structure_complete,
        execution,
        &declaration.annotation_attributes,
        &declaration.annotation_uses,
        &declaration.annotation_refusals,
    )
}

impl ClassSourceField {
    /// Retain the physical field and its explicit initializer gap after a local metadata refusal.
    pub(crate) fn refuse_constant_value(&mut self, error: &Error) {
        self.markers.push(format!(
            "// jarde: ConstantValue refused for field record {} `{}`: {}",
            self.item.index,
            comment_text(&format!(
                "{}:{}",
                String::from_utf8_lossy(&self.item.name.raw().0),
                String::from_utf8_lossy(&self.item.descriptor.raw().0)
            )),
            comment_text(&error.to_string()),
        ));
    }

    /// One field record with the spelling the text writes for it, initializer included when the
    /// field's own `ConstantValue` attribute states one.
    pub(crate) fn of(
        item: FieldItem,
        source_names: SourceClassNames<'_>,
        constant: Option<&MemberDefault>,
        annotations: MemberAnnotationFacts,
        pool: &[CpEntryFacts],
    ) -> Self {
        let mut spelled = spell_field_declaration(&item, source_names, constant);
        let mut member_annotations =
            spell_member_annotation_uses(annotations.declaration.clone(), pool, source_names);
        let type_annotations = spell_type_annotation_uses(
            annotations.type_uses,
            TypeUseOwner::Field,
            &item.descriptor.raw().0,
            pool,
            &member_annotations,
            None,
            None,
            source_names,
        );
        jarde_java::declaration::place_field_annotations(
            &mut spelled.declaration,
            &item.descriptor.raw().0,
            source_names,
            &type_annotations.field_uses,
            &mut member_annotations,
        );
        spelled.annotations = member_annotations;
        spelled.type_annotations = type_annotations;
        let markers = markers_of(&spelled);
        Self {
            item,
            declaration: spelled.declaration,
            generic_signature_projected: false,
            annotations: spelled.annotations,
            type_annotations: spelled.type_annotations,
            markers,
        }
    }
}

impl ClassSourceMethod {
    /// Adapt test facts to Java's integer-name placement policy.
    #[cfg(test)]
    pub(crate) fn integer_constant_projection_text(&self, body: &str) -> Option<String> {
        jarde_java::source_proof::integer_constant_projection_text(
            self.declaration.as_deref(),
            match &self.outcome {
                ClassSourceOutcome::Recovered { report, .. } => Some(report.as_ref()),
                _ => None,
            },
            &self.markers,
            &self.annotations,
            body,
        )
    }

    /// This member's text with its body replaced by the assert projection's staged statements,
    /// keeping the original envelope the recovery wrote and every marker already placed.
    ///
    /// The markers are kept rather than refused (unlike the integer-constant projection): the
    /// assert fold replaces statement text inside the same block, and a marker another
    /// projection placed above the declaration states a fact this replacement does not touch.
    /// A member whose current body is not its own AST's emission is refused by the caller before
    /// this composes anything. Tests call the proof directly; the assembly paths stage the
    /// projection through [`jarde_java::bridge`], which composes this text itself.
    #[cfg(test)]
    pub(crate) fn assert_projection_text(&self, body: &str) -> Option<String> {
        jarde_java::source_proof::assert_projection_text(
            self.declaration.as_deref(),
            match &self.outcome {
                ClassSourceOutcome::Recovered { report, .. } => Some(report.as_ref()),
                _ => None,
            },
            &self.markers,
            &self.annotations,
            body,
        )
    }
    /// The one physical-recovery marker that may be replaced by a later complete family re-run.
    /// It remains on this physical record; only the separately assembled source text omits it.
    pub(crate) fn has_only_explanation_marker(&self) -> bool {
        jarde_java::source_proof::has_only_explanation_marker(
            match &self.outcome {
                ClassSourceOutcome::Recovered { report, .. } => Some(report.as_ref()),
                _ => None,
            },
            &self.markers,
            &self.item.name,
            &self.item.descriptor,
        )
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
        self.generic_signature_refused = false;
        self.generic_signature_projected = true;
        Ok(())
    }

    /// A typed-only method reference cannot be left under a raw descriptor header if its
    /// class-source Signature transaction failed (including an output-budget stop). Retain the
    /// physical report for inspection, but publish only an explicit incomplete-source marker.
    pub(crate) fn withhold_unprojected_functional_body(&mut self) {
        if self.generic_signature_projected {
            return;
        }
        let marker = format!(
            "// jarde: typed functional source for `{}` withheld because its generic declaration was not projected",
            label(&self.item),
        );
        self.markers.push(marker);
        self.text = comment_member(&self.markers);
    }

    /// Replaces this member's complete Java declaration with the supplied proof marker while
    /// retaining its physical declaration and original recovery outcome for the report.
    pub(crate) fn project_bridge(&mut self, marker: String) {
        self.markers.push(marker);
        self.text = comment_member(&self.markers);
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

    /// This member's text re-emitted from a projection's own body, with one marker per projected
    /// site inside the block. Same shape as [`Self::array_projection_text`], for the member-level
    /// lambda-site rewrites that carry several markers at once.
    pub(crate) fn lambda_projection_text(
        &self,
        recovery_text: &str,
        markers: &[String],
    ) -> Option<String> {
        self.projected_body_method_text(recovery_text, None, markers)
    }

    /// Whether a fresh emission of this member's own same-run AST is still exactly the body this
    /// member shows — the safety check a projection that re-emits the whole body makes before it
    /// replaces anything. The comparison is the block's own region, line by line without
    /// indentation, so a marker written beside the declaration does not fail it, while a body
    /// another projection already rewrote does. Such a member keeps its physical presentation.
    pub(crate) fn matches_current_text(&self, recovery_text: &str) -> bool {
        jarde_java::source_proof::matches_current_text(
            self.declaration.as_deref(),
            &self.text,
            recovery_text,
            match &self.outcome {
                ClassSourceOutcome::Recovered { report, .. } => Some(report.as_ref()),
                _ => None,
            },
            &self.markers,
            &self.item.name,
            &self.item.descriptor,
        )
    }

    /// Supply the existing projection facts to the Java-owned declared-cohort replay policy.
    pub(crate) fn matches_declared_projection_input(&self, budget: &mut Budget) -> Result<bool> {
        jarde_java::source_proof::matches_declared_projection_input(
            self.declaration.as_deref(),
            &self.text,
            match &self.outcome {
                ClassSourceOutcome::Recovered { report, .. } => Some(report.as_ref()),
                _ => None,
            },
            &self.markers,
            &self.annotations,
            &self.item.name,
            &self.item.descriptor,
            self.generic_signature_projected,
            self.same_class_generic_body_proof.as_ref(),
            budget,
        )
    }

    /// This member's declaration with the companion's method name replaced by its renamed,
    /// non-conflicting spelling. `None` unless the name occurs exactly once in the declaration —
    /// the rename is surgical or it is not made at all.
    pub(crate) fn renamed_declaration(&self, old: &str, new: &str) -> Option<String> {
        let declaration = self.declaration.as_deref()?;
        let from = format!(" {old}(");
        let to = format!(" {new}(");
        if declaration.matches(&from).count() != 1 {
            return None;
        }
        Some(declaration.replacen(&from, &to, 1))
    }

    /// This member's complete text with a renamed declaration and the rename's own marker inside
    /// the body block, stating the physical name the class file still declares. The rename is
    /// composed on `base` — the member's current text, or a staged body projection of it.
    pub(crate) fn lambda_rename_text(
        &self,
        renamed_declaration: &str,
        marker: &str,
        base: Option<&str>,
    ) -> Option<String> {
        let declaration = self.declaration.as_deref()?;
        let original_line = format!("    {declaration} {{\n");
        let renamed_line = format!("    {renamed_declaration} {{\n");
        let text = base.unwrap_or(&self.text);
        if text.matches(&original_line).count() != 1 {
            return None;
        }
        let mut text = text.replacen(&original_line, &renamed_line, 1);
        let marker_line = indent(&format!("{marker}\n"), 2);
        text.insert_str(renamed_line.len(), &marker_line);
        Some(text)
    }

    /// Places one projected body inside this member's declaration and markers — the exact
    /// composition [`Self::lambda_projection_text`] performs, with the one extension a member
    /// fold needs: the declaration line may be the fold's own re-spelled spelling. The markers
    /// are this member's own followed by the projection's added lines, in the first pass's own
    /// order, so a projected body placed here lands byte-for-byte where the first pass put it.
    pub(crate) fn projected_body_method_text(
        &self,
        body: &str,
        declaration_override: Option<&str>,
        extra_markers: &[String],
    ) -> Option<String> {
        jarde_java::source_proof::projected_body_method_text(
            self.declaration.as_deref(),
            match &self.outcome {
                ClassSourceOutcome::Recovered { report, .. } => Some(report.as_ref()),
                _ => None,
            },
            &self.markers,
            &self.item.name,
            &self.item.descriptor,
            &self.annotations,
            body,
            declaration_override,
            extra_markers,
        )
    }

    /// The span mapping of [`Self::projected_body_method_text`]: one single-line span of the
    /// projected body, translated through the same placement, and verified against the composed
    /// text exactly as the physical member's mapping is.
    pub(crate) fn projected_body_span(
        &self,
        body: &str,
        declaration_override: Option<&str>,
        extra_markers: &[String],
        start: usize,
        end: usize,
    ) -> Option<(usize, usize)> {
        jarde_java::source_proof::projected_body_span(
            self.declaration.as_deref(),
            match &self.outcome {
                ClassSourceOutcome::Recovered { report, .. } => Some(report.as_ref()),
                _ => None,
            },
            &self.markers,
            &self.item.name,
            &self.item.descriptor,
            &self.annotations,
            body,
            declaration_override,
            extra_markers,
            start,
            end,
        )
    }

    /// Spells one fully recovered method inside a class-source anonymous expression. The method
    /// declaration and annotations come from its own selected physical class; `body` is emitted
    /// from that method's same-run AST at the corresponding nested indentation.
    pub(crate) fn anonymous_projection_text(&self, body: &str) -> Option<String> {
        jarde_java::source_proof::anonymous_projection_text(
            self.declaration.as_deref(),
            matches!(self.outcome, ClassSourceOutcome::Recovered { .. }),
            &self.markers,
            (
                &self.annotations,
                &self.parameter_annotations,
                &self.type_annotations,
            ),
            self.item.access_flags,
            body,
        )
    }

    pub(crate) fn anonymous_projection_text_at(
        &self,
        body: &str,
        declaration_indent: usize,
    ) -> Option<String> {
        jarde_java::source_proof::anonymous_projection_text_at(
            self.declaration.as_deref(),
            matches!(self.outcome, ClassSourceOutcome::Recovered { .. }),
            &self.markers,
            (
                &self.annotations,
                &self.parameter_annotations,
                &self.type_annotations,
            ),
            self.item.access_flags,
            body,
            declaration_indent,
        )
    }

    pub(crate) fn anonymous_return_projection_text(&self, body: &str) -> Option<String> {
        jarde_java::source_proof::anonymous_return_projection_text(
            self.declaration.as_deref(),
            matches!(self.outcome, ClassSourceOutcome::Recovered { .. }),
            &self.markers,
            (
                &self.annotations,
                &self.parameter_annotations,
                &self.type_annotations,
            ),
            body,
        )
    }

    pub(crate) fn anonymous_return_projection(
        &self,
        body: &jarde_java::report::ClassSourceAnonymousReturn,
        anchors: Vec<MemberFamilyPhysicalAnchor>,
    ) -> Option<MemberFamilyMethodText> {
        jarde_java::source_proof::anonymous_return_projection(
            self.declaration.as_deref(),
            matches!(self.outcome, ClassSourceOutcome::Recovered { .. }),
            &self.markers,
            (
                &self.annotations,
                &self.parameter_annotations,
                &self.type_annotations,
            ),
            body,
            |text, start, end| MemberFamilyMethodText {
                index: self.item.index,
                text,
                derived: vec![MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::NestedAnonymousExpression,
                    start,
                    end,
                    anchors,
                }],
            },
        )
    }

    /// [`Self::anonymous_return_projection`] for a member whose text must keep what it already
    /// carries: the projected statements are placed under the member's own **physical prefix** —
    /// every marker line exactly as the pass that wrote it wrote it, and the declaration's own
    /// opening line — instead of being composed from the declaration alone.
    ///
    /// [`jarde_java::source_proof::anonymous_return_projection_text`] composes the member text itself and therefore
    /// has to refuse a member that carries a marker (it cannot reproduce one byte for byte: the
    /// generic Signature refusal writes its line one level in, while the recovery-time composer
    /// writes its own two). Keeping the physical prefix instead makes the placement exact for both
    /// writers, and the envelope the recovery layer wrote inside the block is dropped exactly as
    /// the composed placement drops it. `body` is the **statements** form
    /// [`jarde_java::report::emit_class_source_anonymous_return`] writes, and its expression range
    /// is translated through this placement.
    pub(crate) fn projected_statements_method_text(
        &self,
        body: &jarde_java::report::ClassSourceAnonymousReturn,
        anchors: Vec<MemberFamilyPhysicalAnchor>,
    ) -> Option<MemberFamilyMethodText> {
        jarde_java::source_proof::projected_statements_method_text(
            self.declaration.as_deref(),
            matches!(self.outcome, ClassSourceOutcome::Recovered { .. }),
            &self.text,
            body,
            |text, start, end| MemberFamilyMethodText {
                index: self.item.index,
                text,
                derived: vec![MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::NestedAnonymousExpression,
                    start,
                    end,
                    anchors,
                }],
            },
        )
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
            same_run_generic_return: None,
            same_class_generic_body_proof: None,
            generic_signature_refused: false,
            generic_signature_projected: false,
            enum_constructor_source_tail: EnumConstructorSourceTail::Unrecognized,
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
            same_run_generic_return: None,
            same_class_generic_body_proof: None,
            generic_signature_refused: false,
            generic_signature_projected: false,
            enum_constructor_source_tail: EnumConstructorSourceTail::Unrecognized,
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
            same_run_generic_return: None,
            same_class_generic_body_proof: None,
            generic_signature_refused: false,
            generic_signature_projected: false,
            enum_constructor_source_tail: EnumConstructorSourceTail::Unrecognized,
            enum_constructor_signature_erasure_refused: false,
        }
    }

    /// One member one recovery run presented, with the facts of that same run.
    pub(crate) fn recovered(
        item: MethodItem,
        spelled: Spelled,
        report: Box<RecoveryReport>,
        analysis: ClassSourceRunFacts,
        same_run_generic_return: Option<GenericReturnCandidate>,
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
            same_run_generic_return,
            same_class_generic_body_proof: None,
            generic_signature_refused: false,
            generic_signature_projected: false,
            enum_constructor_source_tail: EnumConstructorSourceTail::Unrecognized,
            enum_constructor_signature_erasure_refused: false,
        }
    }

    /// Replaces this record's body with one **rerun's** (change
    /// `recover-nested-class-literal-values`, correction round): the class-source assembly reran
    /// the member under a presentation that keeps member classes pool-spelled, and the run refused
    /// what the first run had recovered. Everything the rerun did not decide stays exactly as the
    /// first presentation wrote it — the declaration, its annotations, and every marker outside
    /// the body (a member-name alias, a refused generic Signature projection) are facts about the
    /// member and the first run's spelling, not about the body — and the run's own verdict writes
    /// the run-level markers after them.
    pub(crate) fn with_rerun_body(
        mut self,
        report: Box<RecoveryReport>,
        analysis: ClassSourceRunFacts,
    ) -> Self {
        let placed = if report.text.is_empty() {
            Placed::Without
        } else {
            match artifact(&report.text) {
                Some(artifact) => Placed::Block(artifact),
                None => {
                    self.markers.push(not_placed_marker(&self.item));
                    Placed::Quoted(&report.text)
                }
            }
        };
        let text = match &self.declaration {
            Some(declaration) => block_member(declaration, placed, &self.markers),
            None => comment_member(&self.markers),
        };
        self.text = prefix_method_annotations(text, &self.annotations);
        self.markers
            .extend(run_markers(&self.item, &report, &analysis));
        self.outcome = ClassSourceOutcome::Recovered { report, analysis };
        self
    }
}

/// The markers one spelling carries, as the vector the member record publishes.
fn class_source_body_lines_match(left: &str, right: &str) -> bool {
    left.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .eq(right.lines().map(str::trim).filter(|line| !line.is_empty()))
}

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

use jarde_java::source_proof::prefix_method_annotations;

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
    pub(crate) source_target: crate::SourceTarget,
    pub(crate) initializer_field_order: Option<&'a [usize]>,
    pub(crate) declared_methods: u64,
    pub(crate) member_table: Option<&'a MemberTableStop>,
    pub(crate) execution: &'a ExecutionReport,
    pub(crate) enum_projection: Option<&'a EnumConstantSourceProjection>,
    pub(crate) array_helper_indices: Option<&'a [u64]>,
    pub(crate) array_method_texts: Option<&'a [(u64, String)]>,
    pub(crate) array_helper_markers: Option<&'a [String]>,
    /// Physical field indices the assembled text omits: the assert projection's hidden switch
    /// field. `None` when no projection staged one, which is the assembly of every class that
    /// keeps its fields physical.
    pub(crate) hidden_fields: Option<&'a [u64]>,
}

impl EnumConstantSourceProjection {
    /// The texts of this projection as the report retains them: a fact copy, index for index.
    pub(crate) fn retained(&self) -> ClassSourceEnumProjectionTexts {
        ClassSourceEnumProjectionTexts {
            constant_field_indices: self.constant_field_indices.clone(),
            backing_field_index: self.backing_field_index,
            implicit_method_indices: self.implicit_method_indices.clone(),
            constants_text: self.constants_text.clone(),
            constructor_texts: self
                .constructor_texts
                .iter()
                .map(|(member, text)| ClassSourceEnumConstructorText {
                    member: *member,
                    text: text.clone(),
                })
                .collect(),
            initializer_text: self.initializer_text.clone(),
        }
    }
}

/// Rebuilds the assembly-side enum projection from its retained texts: the same index sets and
/// the same placed strings, so an assembly over the rebuilt projection writes the same bytes the
/// first pass did. This is placement of retained facts, never a second proof.
pub(crate) fn retained_enum_projection(
    texts: &ClassSourceEnumProjectionTexts,
) -> EnumConstantSourceProjection {
    EnumConstantSourceProjection {
        constant_field_indices: texts.constant_field_indices.clone(),
        backing_field_index: texts.backing_field_index,
        implicit_method_indices: texts.implicit_method_indices.clone(),
        constants_text: texts.constants_text.clone(),
        constructor_texts: texts
            .constructor_texts
            .iter()
            .map(|entry| (entry.member, entry.text.clone()))
            .collect(),
        initializer_text: texts.initializer_text.clone(),
    }
}

/// The initializer field order a proved initializer group staged, derived from the proof the
/// report already carries: proved fields in their `<clinit>` write order, every other field after
/// them in physical order — the same permutation [`crate::facade`]'s first pass staged, read here
/// from the serialized fact instead of being recomputed. `None` exactly when the first pass kept
/// the physical order (no proved group).
pub(crate) fn initializer_field_order_from_proof(
    proof: &ClassSourceInitializerProof,
    fields: &[ClassSourceField],
) -> Option<Vec<usize>> {
    let ClassSourceInitializerProof::Proved { fields: proved } = proof else {
        return None;
    };
    let mut order = Vec::with_capacity(fields.len());
    let mut projected = std::collections::BTreeSet::new();
    for entry in proved {
        let matching: Vec<usize> = fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| (field.item.index == entry.field_index).then_some(index))
            .collect();
        let [physical] = matching.as_slice() else {
            return None;
        };
        let physical = *physical;
        if !projected.insert(physical) {
            return None;
        }
        order.push(physical);
    }
    for (physical, _) in fields.iter().enumerate() {
        if projected.insert(physical) {
            order.push(physical);
        }
    }
    (order.len() == fields.len()).then_some(order)
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
    let projection = jarde_java::source_proof::prepare_enum_constant_source_projection(
        &declaration.name,
        fields,
        methods,
        group,
        terminal_constructor_body,
        initializer,
        |field| {
            (
                field.item.index,
                &field.item.identity.owner,
                field.item.name.raw().0.as_slice(),
                field.item.descriptor.raw().0.as_slice(),
                field.item.access_flags,
                field.declaration.is_some(),
                field.markers.as_slice(),
            )
        },
        |method| {
            (
                method.item.index,
                &method.item.identity.owner,
                method.item.name.raw().0.as_slice(),
                method.item.descriptor.raw().0.as_slice(),
                method.item.access_flags,
            )
        },
        |method| {
            (
                method.declaration.is_some(),
                method.enum_constructor_source_tail,
                method.enum_constructor_signature_erasure_refused,
                method.markers.as_slice(),
                &method.annotations,
                &method.parameter_annotations,
                &method.type_annotations,
            )
        },
        budget,
    )?;
    Ok(projection.map(
        |(
            constant_field_indices,
            backing_field_index,
            implicit_method_indices,
            constants_text,
            constructor_texts,
            initializer_text,
        )| EnumConstantSourceProjection {
            constant_field_indices,
            backing_field_index,
            implicit_method_indices,
            constants_text,
            constructor_texts,
            initializer_text,
        },
    ))
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
    let projection = jarde_java::source_proof::prepare_enum_constant_body_source_projection(
        &declaration.item.definition,
        &declaration.name,
        fields,
        methods,
        &group.constants,
        group.constructor_text.as_ref(),
        group.constructor_user_tail.as_ref(),
        shape,
        |field| {
            (
                field.item.index,
                &field.item.identity.owner,
                field.item.name.raw().0.as_slice(),
                field.declaration.is_some(),
                field.markers.is_empty(),
            )
        },
        |method| {
            (
                method.item.index,
                &method.item.identity,
                method.item.access_flags,
                method.item.name.raw().0.as_slice(),
                method.item.descriptor.raw().0.as_slice(),
            )
        },
        |method| {
            (
                method.declaration.is_some(),
                method.markers.is_empty(),
                matches!(method.outcome, ClassSourceOutcome::Recovered { .. }),
                method.text.as_str(),
                method.enum_constructor_source_tail,
            )
        },
        |constant| {
            (
                constant.field_index,
                constant.constructor_bci,
                constant.source_arguments.as_slice(),
                constant.subclass.as_ref(),
                constant.methods.as_deref(),
            )
        },
        budget,
    )?;
    Ok(projection.map(
        |(
            constant_field_indices,
            backing_field_index,
            implicit_method_indices,
            constants_text,
            constructor_texts,
        )| EnumConstantSourceProjection {
            constant_field_indices,
            backing_field_index,
            implicit_method_indices,
            constants_text,
            constructor_texts,
            initializer_text: None,
        },
    ))
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
    source_text_with_member(
        declaration,
        fields,
        &[],
        methods,
        context,
        None,
        &[],
        &[],
        &mut Vec::new(),
        &[],
    )
    .expect("ordinary class writer has no derived ranges to translate")
}

pub(crate) fn source_text_with_nested_declaration(
    declaration: &ClassSourceDeclaration,
    fields: &[ClassSourceField],
    methods: &[ClassSourceMethod],
    context: &ClassSourceTextContext<'_>,
    method_texts: &[MemberFamilyMethodText],
    nested: &NestedClassSourceText,
) -> (String, Vec<MemberFamilyDerivedProjection>) {
    source_text_with_nested_declarations(
        declaration,
        fields,
        &[],
        methods,
        context,
        method_texts,
        std::slice::from_ref(nested),
        &[],
    )
}

/// The same assembly with any number of sibling nested declarations: the fold's children are
/// appended in the order the caller states (the root's own `InnerClasses` row order), one
/// blank line apart, before the unit's closing brace. The hidden methods are root members a
/// fold proved synthetic (the javac access bridges); each keeps a derived record anchored on
/// the header its omission hides beside.
pub(crate) fn source_text_with_nested_declarations(
    declaration: &ClassSourceDeclaration,
    fields: &[ClassSourceField],
    field_texts: &[MemberFamilyMethodText],
    methods: &[ClassSourceMethod],
    context: &ClassSourceTextContext<'_>,
    method_texts: &[MemberFamilyMethodText],
    nested: &[NestedClassSourceText],
    hidden: &[HiddenFoldMethod],
) -> (String, Vec<MemberFamilyDerivedProjection>) {
    let mut derived = Vec::new();
    let text = source_text_with_member(
        declaration,
        fields,
        field_texts,
        methods,
        context,
        None,
        method_texts,
        nested,
        &mut derived,
        hidden,
    )
    .expect("nested declaration writer has no member-family ranges to translate");
    (text, derived)
}

/// Rebuild one ordinary root source unit with staged method text and translate its physical spans.
/// This intentionally accepts only a root the ordinary writer reproduces exactly; class-family or
/// initializer projections need their own combined certificate and are refused here.
pub(crate) fn source_text_with_method_projections(
    root: &ClassSourceReport,
    method_texts: &[MemberFamilyMethodText],
) -> Option<(String, Vec<MemberFamilyDerivedProjection>)> {
    let declaration = root.declaration.as_ref()?;
    let context = ClassSourceTextContext {
        source_target: root.source_target,
        initializer_field_order: None,
        declared_methods: u64::try_from(root.methods.len()).ok()?,
        member_table: None,
        execution: &root.execution,
        enum_projection: None,
        array_helper_indices: None,
        array_method_texts: None,
        array_helper_markers: None,
        hidden_fields: None,
    };
    jarde_java::source_proof::assemble_method_projection_root(
        source_text(declaration, &root.fields, &root.methods, &context),
        &root.text,
        method_texts,
        |method| method.derived.len(),
        || {
            let mut derived = Vec::new();
            let text = source_text_with_member(
                declaration,
                &root.fields,
                &[],
                &root.methods,
                &context,
                None,
                method_texts,
                &[],
                &mut derived,
                &[],
            )?;
            Some((text, derived))
        },
        |entry| (entry.start, entry.end, !entry.anchors.is_empty()),
    )
}

/// A family source unit is assembled from physical member records, with only certified body
/// replacements and the one certified capture field omitted. No physical report is mutated.
pub(crate) struct MemberFamilyTextProjection<'a> {
    pub(crate) relation: &'a ClassSourceMemberRelation,
    pub(crate) child: &'a ClassSourceReport,
    pub(crate) capture: Option<&'a MemberCaptureProof>,
    pub(crate) static_target: Option<&'a jarde_java::report::StaticMemberTargetInput>,
    pub(crate) generic_static: Option<&'a GenericStaticMemberText>,
    pub(crate) root_methods: &'a [MemberFamilyMethodText],
    pub(crate) child_methods: &'a [MemberFamilyMethodText],
    pub(crate) outer_super_bridges: &'a [OuterSuperBridgeClosureProof],
}

pub(crate) struct GenericStaticMemberText {
    pub(crate) header: String,
    pub(crate) field_index: u64,
    pub(crate) field_declaration: String,
    pub(crate) bridge: PhysicalMethodId,
    pub(crate) bridge_invoke_bci: u32,
}

#[derive(Clone)]
pub(crate) struct MemberFamilyMethodText {
    pub(crate) index: u64,
    pub(crate) text: String,
    /// Ranges in `text`, translated by the family writer when it appends the method.
    pub(crate) derived: Vec<MemberFamilyDerivedProjection>,
}

pub(crate) struct NestedClassSourceText {
    pub(crate) text: String,
    pub(crate) derived: Vec<MemberFamilyDerivedProjection>,
}

/// One root method a member fold omits from the assembled unit — a proved javac access bridge —
/// with the anchors its hidden declaration records.
pub(crate) struct HiddenFoldMethod {
    pub(crate) index: u64,
    pub(crate) anchors: Vec<MemberFamilyPhysicalAnchor>,
}

/// Translate one exact, single-line recovery span through this writer's artifact placement.
/// The equality check catches any drift in the envelope/statements indentation contract.
pub(crate) fn member_family_recovered_span(
    method: &ClassSourceMethod,
    recovery: &RecoveryReport,
    start: usize,
    end: usize,
) -> Option<(usize, usize)> {
    member_family_recovered_span_with_declaration(method, recovery, None, start, end)
}

/// The span mapping of [`member_family_recovered_method_text_with_declaration`]: the same
/// declaration override, so a fold's declaration rewrite moves the spans it publishes by exactly
/// the bytes that rewrite changed.
pub(crate) fn member_family_recovered_span_with_declaration(
    method: &ClassSourceMethod,
    recovery: &RecoveryReport,
    declaration_override: Option<&str>,
    start: usize,
    end: usize,
) -> Option<(usize, usize)> {
    let declaration = declaration_override.unwrap_or(method.declaration.as_ref()?);
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
    let mapped = placed_artifact_span(&body, start, end, block_prefix)?;
    let text = member_family_recovered_method_text_with_declaration(
        method,
        recovery,
        declaration_override,
    )?;
    (text.get(mapped.0..mapped.1)? == source).then_some(mapped)
}

pub(crate) fn member_family_recovered_method_text(
    method: &ClassSourceMethod,
    recovery: &RecoveryReport,
) -> Option<String> {
    member_family_recovered_method_text_with_declaration(method, recovery, None)
}

/// The same reassembly with one projection's own spelling of the declaration: a family fold that
/// renders a nested declaration in this text rewrites the member's declaration line to the source
/// name of the class it folds ([`Self::declaration`]), and both the text and every span the
/// projection publishes have to be computed against that same line or the two drift apart.
pub(crate) fn member_family_recovered_method_text_with_declaration(
    method: &ClassSourceMethod,
    recovery: &RecoveryReport,
    declaration_override: Option<&str>,
) -> Option<String> {
    let declaration = declaration_override.unwrap_or(method.declaration.as_ref()?);
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
    let capture_field = member.capture.and_then(|capture| {
        member
            .child
            .fields
            .iter()
            .find(|field| field.item.index == capture.field_index)
    });
    if root.class != member.relation.root
        || member.child.class != member.relation.child
        || (member.capture.is_some() && capture_field.is_none())
        || member.capture.is_some_and(|capture| capture.constructor.owner != member.child.class)
        // A regenerated capture field cannot preserve extra physical modifiers.
        || capture_field.is_some_and(|field| field.item.access_flags != (ACC_FINAL | 0x1000)
            || !field.annotations.attributes.is_empty()
            || !field.type_annotations.attributes.is_empty())
        || !is_java_identifier(&member.relation.simple_name)
        || (member.generic_static.is_none() && (child_declaration.generic_signature.is_some()
            || child_declaration.generic_refusal.is_some()))
        || !child_declaration.annotation_refusals.is_empty()
        || (member.capture.is_some() && member.relation.access_flags & ACC_STATIC != 0)
        || (member.capture.is_none() && (member.relation.access_flags & ACC_STATIC == 0
            || (member.static_target.is_none_or(|target| target.definition != member.child.class)
                && member.generic_static.is_none())
            || (member.generic_static.is_none() && !member.child.fields.is_empty())))
        || child_declaration.item.declaration.access_flags
            & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION)
            != 0
        || member.child.fields.iter().any(|field| {
            Some(field.item.index) != member.capture.map(|capture| capture.field_index) && field.declaration.is_none()
        })
        || member.outer_super_bridges.iter().any(|closed| {
            closed.calls.is_empty()
                || root
                    .methods
                    .iter()
                    .filter(|method| method.item.identity == closed.bridge.bridge)
                    .count()
                    != 1
                || closed
                    .calls
                    .iter()
                    .any(|call| call.caller.owner != member.child.class || call.bridge != closed.bridge)
        })
        || member.outer_super_bridges.iter().enumerate().any(|(index, closed)| {
            member.outer_super_bridges[index + 1..]
                .iter()
                .any(|other| other.bridge.bridge == closed.bridge.bridge)
        })
    {
        return None;
    }
    // The staged member texts stay out of this rebuild deliberately: a first pass that staged
    // them (a lambda companion rewrite) keeps its pre-existing refusal here, and the assert
    // projection — which refuses beside any staged text — never adds one. The omitted-member and
    // hidden-field channels enter only for a class the assert projection itself staged (one
    // with hidden fields), so every other first pass keeps exactly the equality it had.
    let assert_projected = !root.projection_inputs.hidden_fields.is_empty();
    let context = ClassSourceTextContext {
        source_target: root.source_target,
        initializer_field_order: None,
        declared_methods: root.methods.len() as u64,
        member_table: None,
        execution: &root.execution,
        enum_projection: None,
        array_helper_indices: assert_projected
            .then_some(&root.projection_inputs.omitted_methods[..]),
        array_method_texts: None,
        array_helper_markers: None,
        hidden_fields: assert_projected.then_some(&root.projection_inputs.hidden_fields[..]),
    };
    if source_text(declaration, &root.fields, &root.methods, &context) != root.text {
        return None;
    }
    let mut derived = Vec::new();
    let text = source_text_with_member(
        declaration,
        &root.fields,
        &[],
        &root.methods,
        &context,
        Some(member),
        &[],
        &[],
        &mut derived,
        &[],
    )?;
    let expected = 1
        + usize::from(member.capture.is_none())
        + 3 * usize::from(member.generic_static.is_some())
        + member.outer_super_bridges.len()
        + root.projection_inputs.hidden_fields.len()
        + member.child.projection_inputs.hidden_fields.len()
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

/// Assemble the two separately proved declaration-only children inside the existing root writer.
/// The physical root remains the equality baseline, and neither child's text is published alone.
pub(crate) fn declaration_pair_source_text(
    root: &ClassSourceReport,
    members: &[ClassSourceMemberChild; 2],
) -> Option<(String, Vec<MemberFamilyDerivedProjection>)> {
    let declaration = root.declaration.as_ref()?;
    // The same retained-input equality as the narrow writer: the pair composes the physical root
    // the assert projection staged (omitted members and hidden fields included), while every
    // other first pass — staged member texts among them — keeps the refusal it had.
    let assert_projected = !root.projection_inputs.hidden_fields.is_empty();
    let context = ClassSourceTextContext {
        source_target: root.source_target,
        initializer_field_order: None,
        declared_methods: root.methods.len() as u64,
        member_table: None,
        execution: &root.execution,
        enum_projection: None,
        array_helper_indices: assert_projected
            .then_some(&root.projection_inputs.omitted_methods[..]),
        array_method_texts: None,
        array_helper_markers: None,
        hidden_fields: assert_projected.then_some(&root.projection_inputs.hidden_fields[..]),
    };
    if source_text(declaration, &root.fields, &root.methods, &context) != root.text {
        return None;
    }
    let mut nested_text = String::new();
    let mut nested_derived = Vec::new();
    for (index, member) in members.iter().enumerate() {
        if member.relation.root != root.class || member.relation.child != member.child.class {
            return None;
        }
        let mut methods = Vec::new();
        for method in &member.child.methods {
            let mut derived = Vec::new();
            if method.item.identity.name.0 != b"<init>" {
                let declaration = method.declaration.as_ref()?;
                let mut positions = method.text.match_indices(declaration);
                let (start, _) = positions.next()?;
                if positions.next().is_some() {
                    return None;
                }
                derived.push(MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::MemberMethodDeclaration,
                    start,
                    end: start + declaration.len(),
                    anchors: vec![MemberFamilyPhysicalAnchor::MethodSignature {
                        method: method.item.identity.clone(),
                    }],
                });
            }
            methods.push(MemberFamilyMethodText {
                index: method.item.index,
                text: method.text.clone(),
                derived,
            });
        }
        let projection = MemberFamilyTextProjection {
            relation: &member.relation,
            child: &member.child,
            capture: None,
            static_target: None,
            generic_static: None,
            root_methods: &[],
            child_methods: &methods,
            outer_super_bridges: &[],
        };
        let (text, derived) = render_member_class(&projection)?;
        if index != 0 {
            nested_text.push('\n');
        }
        let offset = nested_text.len();
        nested_text.push_str(&text);
        nested_derived.extend(derived.into_iter().map(|mut entry| {
            entry.start += offset;
            entry.end += offset;
            entry
        }));
    }
    let nested = NestedClassSourceText {
        text: nested_text,
        derived: nested_derived,
    };
    let mut derived = Vec::new();
    let text = source_text_with_member(
        declaration,
        &root.fields,
        &[],
        &root.methods,
        &context,
        None,
        &[],
        std::slice::from_ref(&nested),
        &mut derived,
        &[],
    )?;
    if derived.len() != 6
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
    field_texts: &[MemberFamilyMethodText],
    methods: &[ClassSourceMethod],
    context: &ClassSourceTextContext<'_>,
    member_family: Option<&MemberFamilyTextProjection<'_>>,
    method_texts: &[MemberFamilyMethodText],
    nested_classes: &[NestedClassSourceText],
    derived: &mut Vec<MemberFamilyDerivedProjection>,
    hidden_methods: &[HiddenFoldMethod],
) -> Option<String> {
    if let Some(refusal) = jarde_java::declaration::source_unit_target_refusal(
        context.source_target,
        &declaration.item.declaration.this_class,
        declaration.item.declaration.access_flags,
        fields,
        methods,
        |method| {
            (
                method.item.access_flags,
                matches!(
                    method.item.body,
                    crate::MemberBodyEvidence::CodeAttribute { .. }
                ),
            )
        },
        |field| !field.type_annotations.attributes.is_empty(),
        |method| !method.type_annotations.attributes.is_empty(),
    ) {
        return Some(refusal);
    }
    let initializer_field_order = context.initializer_field_order;
    let declared_methods = context.declared_methods;
    let member_table = context.member_table;
    let execution = context.execution;
    let enum_projection = context.enum_projection;
    let array_helper_indices = context.array_helper_indices;
    let array_method_texts = context.array_method_texts;
    let array_helper_markers = context.array_helper_markers;
    let hidden_fields = context.hidden_fields;
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
            } else if declaration.name.contains('$') {
                "projected after physical parent erasure and nested member proof"
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
    let header_start = out.len();
    out.push_str(&declaration.declaration);
    out.push_str(" {\n");
    if let Some(member) = member_family {
        for closed in member.outer_super_bridges {
            derived.push(MemberFamilyDerivedProjection {
                kind: MemberFamilyDerivedKind::HiddenOuterSuperBridge,
                start: header_start,
                end: out.len() - 1,
                anchors: vec![
                    MemberFamilyPhysicalAnchor::MethodPoint {
                        method: closed.bridge.bridge.clone(),
                        bci: closed.bridge.invoke_bci,
                    },
                    MemberFamilyPhysicalAnchor::OuterSuperTarget {
                        method: closed.bridge.target_method.clone(),
                        owner: closed.bridge.target_owner.clone(),
                        name: closed.bridge.target_name.clone(),
                        descriptor: closed.bridge.target_descriptor.clone(),
                    },
                ],
            });
        }
    }
    for hidden in hidden_methods {
        derived.push(MemberFamilyDerivedProjection {
            kind: MemberFamilyDerivedKind::HiddenAccessBridge,
            start: header_start,
            end: out.len() - 1,
            anchors: hidden.anchors.clone(),
        });
    }
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
        if hidden_fields.is_some_and(|hidden| hidden.contains(&field.item.index)) {
            // The assert projection proved the field's whole pattern rewritten: the declaration
            // goes with it, and the derived record keeps the physical field anchored on the
            // header it was hidden beside.
            derived.push(MemberFamilyDerivedProjection {
                kind: MemberFamilyDerivedKind::HiddenAssertSwitchField,
                start: header_start,
                end: out.len() - 1,
                anchors: vec![
                    MemberFamilyPhysicalAnchor::Field {
                        field: field.item.identity.clone(),
                        index: field.item.index,
                    },
                    MemberFamilyPhysicalAnchor::ClassDefinition {
                        definition: field.item.identity.owner.clone(),
                    },
                ],
            });
            continue;
        }
        if !first {
            out.push('\n');
        }
        first = false;
        for annotation in &field.annotations.uses {
            out.push_str(&indent(&format!("{annotation}\n"), 1));
        }
        if let Some(projected) = field_texts
            .iter()
            .find(|projected| projected.index == field.item.index)
        {
            let offset = out.len();
            out.push_str(&projected.text);
            for entry in &projected.derived {
                let source_span = projected.text.get(entry.start..entry.end)?;
                let mut translated = entry.clone();
                translated.start += offset;
                translated.end += offset;
                if out.get(translated.start..translated.end) != Some(source_span) {
                    return None;
                }
                derived.push(translated);
            }
        } else {
            match (&field.declaration, field.markers.is_empty()) {
                (Some(declaration), _) => {
                    out.push_str(&declaration_member(declaration, &field.markers))
                }
                (None, _) => out.push_str(&comment_member(&field.markers)),
            }
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
        if member_family.is_some_and(|member| {
            member
                .outer_super_bridges
                .iter()
                .any(|closed| closed.bridge.bridge == method.item.identity)
        }) {
            continue;
        }
        if hidden_methods
            .iter()
            .any(|hidden| hidden.index == method.item.index)
        {
            continue;
        }
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
        } else if let Some(projected) = method_texts
            .iter()
            .find(|projected| projected.index == method.item.index)
        {
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
    for nested in nested_classes {
        if !first {
            out.push('\n');
        }
        let offset = out.len();
        out.push_str(&nested.text);
        derived.extend(nested.derived.iter().cloned().map(|mut entry| {
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
    let facts = declaration.item.declaration.clone();
    jarde_java::source_proof::render_member_class(
        &member.relation.simple_name,
        member.relation.access_flags,
        (
            &facts.this_class,
            facts.access_flags,
            facts.super_class.as_ref(),
            &facts.interfaces,
        ),
        &declaration.name,
        &declaration.annotation_uses,
        member.capture.map(|capture| capture.field_index),
        member.generic_static.map(|generic| {
            (
                &generic.header,
                generic.field_index,
                &generic.field_declaration,
                &generic.bridge,
                generic.bridge_invoke_bci,
            )
        }),
        &child.fields,
        &child.methods,
        &child.projection_inputs.hidden_fields,
        &child.projection_inputs.omitted_methods,
        member.child_methods,
        |field| {
            (
                field.item.index,
                field.annotations.uses.as_slice(),
                field.declaration.as_ref(),
                field.markers.as_slice(),
            )
        },
        |method| (method.item.index, &method.item.identity, &method.text),
        |staged| (staged.index, &staged.text, staged.derived.as_slice()),
        |staged| (&mut staged.text, &mut staged.derived),
        |entry| (entry.start, entry.end),
        |entry, start, end| {
            entry.start = start;
            entry.end = end;
        },
        |start, end, capture_field_index| {
            let capture_field = child
                .fields
                .iter()
                .find(|field| field.item.index == capture_field_index)
                .expect("validated capture field");
            MemberFamilyDerivedProjection {
                kind: MemberFamilyDerivedKind::HiddenCaptureField,
                start,
                end,
                anchors: vec![MemberFamilyPhysicalAnchor::Field {
                    field: capture_field.item.identity.clone(),
                    index: capture_field.item.index,
                }],
            }
        },
        |start, end| MemberFamilyDerivedProjection {
            kind: MemberFamilyDerivedKind::MemberClassDeclaration,
            start,
            end,
            anchors: vec![
                MemberFamilyPhysicalAnchor::ClassDefinition {
                    definition: member.relation.child.clone(),
                },
                MemberFamilyPhysicalAnchor::ClassDefinition {
                    definition: member.relation.root.clone(),
                },
            ],
        },
        |start, end, bridge, bci, is_bridge| {
            if is_bridge {
                MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::MemberMethodDeclaration,
                    start,
                    end,
                    anchors: vec![MemberFamilyPhysicalAnchor::MethodPoint {
                        method: bridge.clone(),
                        bci,
                    }],
                }
            } else {
                MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::MemberGenericSignature,
                    start,
                    end,
                    anchors: vec![MemberFamilyPhysicalAnchor::ClassDefinition {
                        definition: member.relation.child.clone(),
                    }],
                }
            }
        },
        |start, end, field, generic| {
            if generic {
                MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::MemberGenericSignature,
                    start,
                    end,
                    anchors: vec![MemberFamilyPhysicalAnchor::Field {
                        field: field.item.identity.clone(),
                        index: field.item.index,
                    }],
                }
            } else {
                MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::HiddenAssertSwitchField,
                    start,
                    end,
                    anchors: vec![MemberFamilyPhysicalAnchor::Field {
                        field: field.item.identity.clone(),
                        index: field.item.index,
                    }],
                }
            }
        },
        |start, end, identity| MemberFamilyDerivedProjection {
            kind: MemberFamilyDerivedKind::MemberConstructorName,
            start,
            end,
            anchors: vec![
                MemberFamilyPhysicalAnchor::MethodPoint {
                    method: identity.clone(),
                    bci: 0,
                },
                MemberFamilyPhysicalAnchor::ClassDefinition {
                    definition: member.relation.child.clone(),
                },
            ],
        },
    )
}

fn append_family_method(
    out: &mut String,
    method: &MemberFamilyMethodText,
    nested: bool,
    derived: &mut Vec<MemberFamilyDerivedProjection>,
) -> Option<()> {
    jarde_java::source_proof::append_family_method(
        out,
        &method.text,
        &method.derived,
        nested,
        derived,
        |entry| (entry.start, entry.end),
        |entry, start, end| {
            entry.start = start;
            entry.end = end;
        },
    )
}

/// Render one direct static member as a nested declaration of the fold's own source unit, from the
/// child's typed declaration and member records. The child's physical report text is never parsed
/// or sliced: the header is rebuilt from the row's own flags with the fold's spelling of the types
/// it names, staged field and method texts carry the re-spelled references, and every physical
/// constructor is renamed to the member's own source name — the pool's `$` name it came in with is
/// not a name the nested unit can resolve. The staged texts' derived ranges are translated through
/// this writer's placement, exactly as the family method writer does.
/// The synthetic members one proved non-static child elides inside a fold's nested declaration:
/// the capture field's row and the constructor's synthetic first parameter. The write statement
/// itself is removed from the staged constructor text before this renderer sees it.
#[derive(Clone, Copy)]
pub(crate) struct InstanceMemberElision {
    pub(crate) capture_field_index: u64,
}

/// The first pass's own class-text projection channels a nested child block carries beside its
/// staged texts: the members its projection omitted, its marker lines, and the member texts its
/// projection substituted. The enum-constant and initializer projections are deliberately absent
/// — a child whose first pass staged either is refused at the fold's child gate rather than
/// degraded here.
pub(crate) struct NestedRetainedTexts<'a> {
    pub(crate) omitted_methods: &'a [u64],
    pub(crate) markers: &'a [String],
    pub(crate) member_texts: &'a [ClassSourceProjectedMemberText],
    pub(crate) hidden_fields: &'a [u64],
}

pub(crate) fn nested_static_member_source_text(
    simple_name: &str,
    access_flags: u16,
    declaration: &ClassSourceDeclaration,
    fields: &[ClassSourceField],
    field_texts: &[MemberFamilyMethodText],
    methods: &[ClassSourceMethod],
    method_texts: &[MemberFamilyMethodText],
    fold_targets: &[(&str, &str)],
    child_definition: PhysicalDefinitionId,
    root_definition: PhysicalDefinitionId,
    elision: Option<InstanceMemberElision>,
    retained: &NestedRetainedTexts<'_>,
) -> Option<NestedClassSourceText> {
    // The original physical declaration clone stays owned here through the writer call.
    let facts = declaration.item.declaration.clone();
    let (text, derived) = jarde_java::source_proof::nested_static_member_source_text(
        simple_name,
        access_flags,
        (
            &facts.this_class,
            facts.access_flags,
            facts.super_class.as_ref(),
            &facts.interfaces,
        ),
        declaration.generic_type_parameters.as_deref(),
        &declaration.name,
        &declaration.annotation_uses,
        &declaration.annotation_refusals,
        fields,
        field_texts,
        methods,
        method_texts,
        fold_targets,
        elision.map(|elision| elision.capture_field_index),
        (
            retained.omitted_methods,
            retained.markers,
            retained.member_texts,
            retained.hidden_fields,
        ),
        |field| {
            (
                field.item.index,
                field.annotations.uses.as_slice(),
                field.declaration.as_ref(),
                field.markers.as_slice(),
            )
        },
        |method| (method.item.index, &method.item.identity, &method.text),
        |staged| (staged.index, &staged.text, staged.derived.as_slice()),
        |staged| (&mut staged.text, &mut staged.derived),
        |retained| (retained.member, &retained.text),
        |index, text| MemberFamilyMethodText {
            index,
            text,
            derived: Vec::new(),
        },
        |entry| {
            (
                entry.start,
                entry.end,
                entry.kind == MemberFamilyDerivedKind::HiddenCaptureWrite,
            )
        },
        |entry, start, end| {
            entry.start = start;
            entry.end = end;
        },
        |start, end| MemberFamilyDerivedProjection {
            kind: MemberFamilyDerivedKind::MemberClassDeclaration,
            start,
            end,
            anchors: vec![
                MemberFamilyPhysicalAnchor::ClassDefinition {
                    definition: child_definition.clone(),
                },
                MemberFamilyPhysicalAnchor::ClassDefinition {
                    definition: root_definition.clone(),
                },
            ],
        },
        |start, end, field, capture| {
            if capture {
                MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::HiddenCaptureField,
                    start,
                    end,
                    anchors: vec![
                        MemberFamilyPhysicalAnchor::Field {
                            field: field.item.identity.clone(),
                            index: field.item.index,
                        },
                        MemberFamilyPhysicalAnchor::ClassDefinition {
                            definition: child_definition.clone(),
                        },
                    ],
                }
            } else {
                MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::HiddenAssertSwitchField,
                    start,
                    end,
                    anchors: vec![
                        MemberFamilyPhysicalAnchor::Field {
                            field: field.item.identity.clone(),
                            index: field.item.index,
                        },
                        MemberFamilyPhysicalAnchor::ClassDefinition {
                            definition: child_definition.clone(),
                        },
                    ],
                }
            }
        },
        |start, end, method, parameter| {
            if parameter {
                MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::HiddenConstructorParameter,
                    start,
                    end,
                    anchors: vec![
                        MemberFamilyPhysicalAnchor::ConstructorParameter {
                            method: method.clone(),
                            index: 0,
                        },
                        MemberFamilyPhysicalAnchor::ClassDefinition {
                            definition: child_definition.clone(),
                        },
                    ],
                }
            } else {
                MemberFamilyDerivedProjection {
                    kind: MemberFamilyDerivedKind::MemberConstructorName,
                    start,
                    end,
                    anchors: vec![
                        MemberFamilyPhysicalAnchor::MethodPoint {
                            method: method.clone(),
                            bci: 0,
                        },
                        MemberFamilyPhysicalAnchor::ClassDefinition {
                            definition: child_definition.clone(),
                        },
                    ],
                }
            }
        },
    )?;
    Some(NestedClassSourceText { text, derived })
}

/// Render a proved enum as a lexical member using the child's typed declaration and member
/// records. The physical report text is never parsed or sliced.
pub(crate) fn nested_enum_source_text(
    simple_name: &str,
    access_flags: u16,
    declaration: &ClassSourceDeclaration,
    fields: &[ClassSourceField],
    methods: &[ClassSourceMethod],
    enum_projection: &EnumConstantSourceProjection,
    method_texts: &[MemberFamilyMethodText],
    descendant: Option<&NestedClassSourceText>,
    anchors: Vec<MemberFamilyPhysicalAnchor>,
) -> Option<NestedClassSourceText> {
    let facts = declaration.item.declaration.clone();
    let (text, derived) = jarde_java::source_proof::nested_enum_source_text(
        simple_name,
        access_flags,
        (
            &facts.this_class,
            facts.access_flags,
            facts.super_class.as_ref(),
            &facts.interfaces,
        ),
        fields,
        methods,
        (
            &enum_projection.constants_text,
            enum_projection.backing_field_index,
            &enum_projection.constant_field_indices,
            &enum_projection.constructor_texts,
            &enum_projection.implicit_method_indices,
            enum_projection.initializer_text.as_deref(),
        ),
        method_texts,
        descendant,
        |field| {
            (
                field.item.index,
                field.annotations.uses.as_slice(),
                field.declaration.as_ref(),
                field.markers.as_slice(),
            )
        },
        |method| (method.item.index, &method.item.identity, &method.text),
        |staged| (staged.index, &staged.text, staged.derived.as_slice()),
        |nested| (&nested.text, nested.derived.as_slice()),
        |entry| (entry.start, entry.end),
        |entry, start, end| {
            entry.start = start;
            entry.end = end;
        },
        |start, end| MemberFamilyDerivedProjection {
            kind: MemberFamilyDerivedKind::NestedEnumDeclaration,
            start,
            end,
            anchors,
        },
    )?;
    Some(NestedClassSourceText { text, derived })
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
    use jarde_java::report::GenericConstructorFieldWrite;

    fn cb_physical_method() -> ClassSourceMethod {
        use crate::*;
        let engine = Engine::new();
        let mut budget = task_budget(&[]).unwrap();
        let snapshot = engine
            .open(
                ArtifactInput::bytes(
                    include_bytes!(
                        "../tests/fixtures/s1-lambda-handle-names/helper-inline-dash.jar"
                    )
                    .to_vec(),
                ),
                &mut budget,
            )
            .unwrap();
        let request = ClassSourceRequest {
            source_target: SourceTarget::JAVA_8,
            class: ClassRef::Name {
                class: ClassNameQuery::internal("Subject"),
            },
            environment: EnvironmentRequest {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
                policy: EnvironmentPolicy::PlainJar,
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                loader: LoaderId("app".into()),
            },
        };
        let OperationOutcome::Performed(class) = engine
            .class_source_with_evidence(
                &[snapshot],
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget,
            )
            .unwrap()
        else {
            panic!("class source did not run")
        };
        class
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"run")
            .unwrap()
            .clone()
    }

    #[test]
    fn cb_anonymous_placement_domains_preserve_distinct_eligibility() {
        let mut base = cb_physical_method();
        base.markers.clear();
        base.item.access_flags &= !(ACC_STATIC | 0x0040 | 0x1000);
        base.declaration = Some("public Object run()".into());
        let body = "        return null;\n";
        assert!(base.anonymous_projection_text(body).is_some());
        for flag in [ACC_STATIC, 0x0040, 0x1000] {
            let mut method = base.clone();
            method.item.access_flags |= flag;
            assert!(method.anonymous_projection_text(body).is_none());
            assert!(method.anonymous_return_projection_text(body).is_some());
        }
        for variant in 0..6 {
            let mut method = base.clone();
            match variant {
                0 => method.markers.push("// refusal".into()),
                1 => method.parameter_annotations.refusals.push("refusal".into()),
                2 => method
                    .parameter_annotations
                    .uses_by_position
                    .push(vec!["@P".into()]),
                3 => method.type_annotations.refusals.push("refusal".into()),
                4 => method.type_annotations.return_uses.push("@T".into()),
                _ => method.outcome = ClassSourceOutcome::NoBody,
            }
            assert!(
                method.anonymous_projection_text(body).is_none(),
                "{variant}"
            );
            assert!(
                method.anonymous_return_projection_text(body).is_none(),
                "{variant}"
            );
            // The ordinary body writer has no recovered/strict-anonymous eligibility gate.
            assert!(
                method
                    .projected_body_method_text("{\n    return null;\n}\n", None, &[])
                    .is_some()
            );
        }
    }

    #[test]
    fn cb_return_and_current_prefix_ranges_are_exact_utf8() {
        let mut method = cb_physical_method();
        method.markers.clear();
        method.declaration = Some("public Object run()".into());
        let expression = "new Object() { String 名 = \"𐐀\"; }";
        let text = format!("        return {expression};\n");
        let from = text.find(expression).unwrap();
        let body = jarde_java::report::ClassSourceAnonymousReturn {
            text,
            expression_range: from..from + expression.len(),
        };
        let anchor = MemberFamilyPhysicalAnchor::ClassDefinition {
            definition: method.item.identity.owner.clone(),
        };
        let placed = method
            .anonymous_return_projection(&body, vec![anchor.clone()])
            .unwrap();
        assert_eq!(placed.index, method.item.index);
        assert_eq!(
            &placed.text[placed.derived[0].start..placed.derived[0].end],
            expression
        );
        assert_eq!(placed.derived[0].anchors, vec![anchor.clone()]);
        let prefix = "    // generic Signature refusal\n    public Object run() {\n";
        method.text = format!("{prefix}        old();\n    }}\n");
        method.markers.push("// generic Signature refusal".into());
        method.parameter_annotations.refusals.push("refusal".into());
        let original = method.clone();
        assert!(method.anonymous_return_projection(&body, vec![]).is_none());
        let placed = method
            .projected_statements_method_text(&body, vec![anchor.clone()])
            .unwrap();
        assert!(placed.text.starts_with(prefix));
        assert_eq!(
            &placed.text[placed.derived[0].start..placed.derived[0].end],
            expression
        );
        assert_eq!(placed.derived[0].anchors, vec![anchor]);
        for range in [
            from..from,
            0..usize::MAX,
            body.text.find('𐐀').unwrap() + 1..body.text.find('𐐀').unwrap() + 4,
        ] {
            let mut bad = body.clone();
            bad.expression_range = range;
            assert!(
                method
                    .projected_statements_method_text(&bad, vec![])
                    .is_none()
            );
            let mut strict = method.clone();
            strict.markers.clear();
            strict.parameter_annotations.refusals.clear();
            assert!(strict.anonymous_return_projection(&bad, vec![]).is_none());
        }
        assert_eq!(method, original);
        method.text.push_str("    public Object run() {\n");
        assert!(
            method
                .projected_statements_method_text(&body, vec![])
                .is_none()
        );
    }

    #[test]
    fn cb_body_writer_and_span_keep_markers_annotations_and_declaration_gate() {
        let mut method = cb_physical_method();
        assert!(method.has_only_explanation_marker());
        method.annotations.uses.push("@名".into());
        let body = "// envelope\n{\n    return \"𐐀\";\n}\n";
        let markers = ["// derived 名".to_string()];
        let text = method
            .projected_body_method_text(body, Some("Object 名()"), &markers)
            .unwrap();
        assert!(text.starts_with("    @名\n    Object 名() {\n"));
        assert!(text.contains("// derived 名"));
        assert!(!text.contains("produced no statement"));
        let at = body.find('𐐀').unwrap();
        let mapped = method
            .projected_body_span(body, Some("Object 名()"), &markers, at, at + 4)
            .unwrap();
        assert_eq!(&text[mapped.0..mapped.1], "𐐀");
        for (start, end) in [(at + 1, at + 4), (at, at), (0, body.len())] {
            assert!(
                method
                    .projected_body_span(body, None, &markers, start, end)
                    .is_none()
            );
        }
        assert!(
            method
                .projected_body_method_text("not an artifact", None, &[])
                .is_none()
        );
        method.declaration = None;
        assert!(
            method
                .projected_body_method_text(body, Some("Object 名()"), &markers)
                .is_none()
        );
        assert!(
            method
                .projected_body_span(body, Some("Object 名()"), &markers, at, at + 4)
                .is_none()
        );
    }

    #[test]
    fn physical_explanation_marker_preserves_exact_body_matching_and_projected_spans() {
        use crate::*;
        let engine = Engine::new();
        let mut budget = task_budget(&[]).unwrap();
        let snapshot = engine
            .open(
                ArtifactInput::bytes(
                    include_bytes!(
                        "../tests/fixtures/s1-lambda-handle-names/helper-inline-dash.jar"
                    )
                    .to_vec(),
                ),
                &mut budget,
            )
            .unwrap();
        let request = ClassSourceRequest {
            source_target: SourceTarget::JAVA_8,
            class: ClassRef::Name {
                class: ClassNameQuery::internal("Subject"),
            },
            environment: EnvironmentRequest {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
                policy: EnvironmentPolicy::PlainJar,
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                loader: LoaderId("app".into()),
            },
        };
        let OperationOutcome::Performed(class) = engine
            .class_source_with_evidence(
                &[snapshot],
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget,
            )
            .unwrap()
        else {
            panic!("class source did not run")
        };
        let method = class
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"run")
            .unwrap();
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("missing method")
        };
        assert!(method.has_only_explanation_marker());
        assert!(method.matches_current_text(&report.text));
        let mut changed = method.clone();
        changed.text = changed.text.replace("@bytecode 0 5", "@bytecode 0");
        assert!(!changed.matches_current_text(&report.text));
        changed = method.clone();
        changed.markers.push("// jarde: another projection".into());
        changed.text = block_member(
            changed.declaration.as_deref().unwrap(),
            Placed::Block(artifact(&report.text).unwrap()),
            &changed.markers,
        );
        assert!(!changed.matches_current_text(&report.text));
        changed.generic_signature_projected = true;
        assert!(
            !changed
                .matches_declared_projection_input(&mut Budget::new(
                    unlimited_annotation_test_limits()
                ))
                .unwrap(),
            "a forged generic flag cannot admit explanation-only text with an extra marker"
        );
        let notes = ["// jarde: derived body".to_string()];
        let placed = method
            .projected_body_method_text(&report.text, None, &notes)
            .unwrap();
        assert!(!placed.contains("produced no statement"));
        let start = report.text.find("@bytecode").unwrap();
        let end = start + "@bytecode 0 5".len();
        let (from, to) = method
            .projected_body_span(&report.text, None, &notes, start, end)
            .unwrap();
        assert_eq!(&placed[from..to], &report.text[start..end]);
    }

    #[test]
    fn member_descriptor_type_tokens_do_not_rewrite_enclosing_prefixes() {
        let physical = "p.𐐀pkg.Outer$Inner";
        let source = "p.𐐀pkg.Outer.Inner";
        let declaration =
            "public static p.𐐀pkg.Outer$Inner make(p.𐐀pkg.Outer outer, p.𐐀pkg.Outer[][] rest)";
        let (projected, count) = replace_java_type_token(declaration, physical, source);
        assert_eq!(count, 1);
        assert_eq!(replace_java_type_token(&projected, source, source).1, 1);
        assert_eq!(
            replace_java_type_token(&projected, "p.𐐀pkg.Outer", "p.𐐀pkg.Outer").1,
            2
        );
        for outside in [
            "𐐀p.𐐀pkg.Outer$Inner",
            "p.𐐀pkg.Outer$InnerExtra",
            "other.p.𐐀pkg.Outer$Inner",
        ] {
            assert_eq!(
                replace_java_type_token(outside, physical, source),
                (outside.to_owned(), 0)
            );
        }
        assert_eq!(
            descriptor_component_base("p.𐐀pkg.Outer[][]"),
            "p.𐐀pkg.Outer"
        );
        assert_eq!(
            replace_java_type_token("p.𐐀pkg.Outer... rest", "p.𐐀pkg.Outer", "Outer"),
            ("Outer... rest".to_owned(), 1)
        );
        assert_eq!(
            replace_java_type_token("p.𐐀pkg.Outer...Extra", "p.𐐀pkg.Outer", "Outer"),
            ("p.𐐀pkg.Outer...Extra".to_owned(), 0)
        );
    }

    #[test]
    fn binary_pool_class_spelling_uses_mutf8_for_package_and_member_segments() {
        for (raw, expected) in [
            ("p/𐐀pkg/Outer$Inner", "p.𐐀pkg.Outer$Inner"),
            ("p/𐐀Outer$𐐀Inner", "p.𐐀Outer$𐐀Inner"),
            ("p/Lfoo", "p.Lfoo"),
            ("p/_", "p._"),
        ] {
            assert_eq!(
                binary_pool_class_name(&JvmBytes::from_unicode(raw).0).unwrap(),
                expected
            );
        }
        for raw in [
            JvmBytes::from_unicode("p/😀"),
            JvmBytes::from_unicode("p/Outer;Inner"),
            JvmBytes(vec![b'p', b'/', 0xed, 0xa0, 0x80]),
            JvmBytes(vec![b'p', b'/', 0xf0, 0x90, 0x90, 0x80]),
        ] {
            assert!(binary_pool_class_name(&raw.0).is_err(), "{raw:?}");
        }
    }

    const ARRAY_RETURN_CLASS: &[u8] = include_bytes!(
        "../tests/fixtures/p3-heterogeneous-array-initializers-v3/direct/javac8/classes/Main.class"
    );

    struct MissingHeaders {
        loader: jarde_reader::view::LoaderId,
    }

    impl jarde_java::source_proof::RecoveryHeaders for MissingHeaders {
        type Error = jarde_reader::error::Error;

        fn caller_loader(&self) -> &jarde_reader::view::LoaderId {
            &self.loader
        }

        fn selected_from(
            &mut self,
            _initiating_loader: &jarde_reader::view::LoaderId,
            _owner: &[u8],
            budget: &mut Budget,
        ) -> std::result::Result<
            jarde_java::source_proof::DependencyResult<
                jarde_java::source_proof::ReferenceClassHeader,
            >,
            Self::Error,
        > {
            budget.poll()?;
            Ok(jarde_java::source_proof::DependencyResult::Missing)
        }

        fn platform_unprovided(
            &mut self,
            _owner: &[u8],
            budget: &mut Budget,
        ) -> std::result::Result<bool, Self::Error> {
            budget.poll()?;
            Ok(true)
        }
    }

    const MEMBER_ANNOTATION_TARGET: &[u8] = include_bytes!(
        "../tests/fixtures/recovered-inputs/evidence/java-syntax-2026-09-22/member-annotation-uses/generated/original/MemberTagged.class"
    );

    #[test]
    fn raw_receiver_assignment_uses_java_reference_and_array_erasure_rules() {
        let java_release = 8;
        for (source, target) in [
            (
                b"Ljava/lang/String;".as_slice(),
                b"Ljava/lang/Object;".as_slice(),
            ),
            (
                b"[Ljava/lang/String;".as_slice(),
                b"[Ljava/lang/Object;".as_slice(),
            ),
            (
                b"[[Ljava/lang/String;".as_slice(),
                b"[[Ljava/lang/Object;".as_slice(),
            ),
            (b"[I".as_slice(), b"Ljava/lang/Object;".as_slice()),
            (
                b"[[Ljava/lang/Object;".as_slice(),
                b"[Ljava/lang/Object;".as_slice(),
            ),
            (b"[[I".as_slice(), b"[Ljava/lang/Object;".as_slice()),
            (b"[I".as_slice(), b"Ljava/lang/Cloneable;".as_slice()),
        ] {
            assert!(descriptor_reference_assignable(
                source,
                target,
                java_release
            ));
        }
        for (source, target) in [
            (b"[I".as_slice(), b"[Ljava/lang/Object;".as_slice()),
            (b"[I".as_slice(), b"[J".as_slice()),
            (
                b"[Ljava/lang/Object;".as_slice(),
                b"[Ljava/lang/Number;".as_slice(),
            ),
        ] {
            assert!(!descriptor_reference_assignable(
                source,
                target,
                java_release
            ));
        }
    }

    #[test]
    fn raw_receiver_type_proof_propagates_budget_and_cancellation() {
        let method = generic_void_probe().0.item.identity;
        let use_site = SameClassFieldUse {
            caller: "set".to_owned(),
            physical_method: method.clone(),
            bci: 12,
            opcode: 0xb5,
            owner: b"probe/Bound".to_vec(),
            name: b"value".to_vec(),
            descriptor: b"Ljava/lang/Object;".to_vec(),
            read_expressible: None,
            write_source: Some(SameClassFieldWriteSource::Null),
            receiver_source: Some(SameClassFieldReceiverSource::Parameter { slot: 0 }),
            source_complete: true,
        };
        let published = [SameClassPublishedMethodParameters {
            method,
            parameter_slots: vec![0],
            descriptor_parameters: vec![b"Lprobe/Bound;".to_vec()],
            generic_parameters: None,
            method_type_parameters: Vec::new(),
        }];

        assert!(
            raw_field_receiver_source(
                &use_site,
                b"probe/Bound",
                true,
                &published,
                &mut Budget::new(unlimited_annotation_test_limits()),
            )
            .expect("a published generic class can have a raw formal")
        );
        assert!(
            !raw_field_receiver_source(
                &use_site,
                b"probe/Bound",
                false,
                &published,
                &mut Budget::new(unlimited_annotation_test_limits()),
            )
            .expect("an ordinary nongeneric class reference is not a raw type")
        );

        let mut limits = unlimited_annotation_test_limits();
        limits.analysis_steps = 0;
        let mut budget = Budget::new(limits);
        assert!(matches!(
            raw_field_receiver_source(&use_site, b"probe/Bound", true, &published, &mut budget),
            Err(Error::BudgetExceeded {
                dimension: crate::BudgetDimension::AnalysisSteps,
                ..
            })
        ));

        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut budget =
            Budget::with_cancellation_token(unlimited_annotation_test_limits(), cancellation);
        assert!(matches!(
            raw_field_receiver_source(&use_site, b"probe/Bound", true, &published, &mut budget),
            Err(Error::Cancelled { .. })
        ));
    }

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

    fn array_return_projection_probe()
    -> (ClassSourceMethod, MemberAttributes, GenericReturnCandidate) {
        let mut setup_budget = Budget::new(unlimited_annotation_test_limits());
        let snapshot = jarde_reader::artifact::ArtifactSnapshot::open(
            jarde_reader::artifact::ArtifactInput::bytes(ARRAY_RETURN_CLASS.to_vec()),
            &mut setup_budget,
        )
        .expect("the direct-array class opens");
        let domain = jarde_reader::view::LoadDomain {
            loader: jarde_reader::view::LoaderId("app".to_owned()),
            parent_loader: None,
            delegation: jarde_reader::view::DelegationPolicy::ParentFirst,
            roots: vec![jarde_reader::view::LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: jarde_reader::view::ModuleMode::ClassPath,
            external_override: jarde_reader::view::RuntimeUncertainty::None,
            runtime_transformation: jarde_reader::view::RuntimeUncertainty::None,
        };
        let method = PhysicalMethodId {
            owner: jarde_reader::model::PhysicalDefinitionId {
                location: jarde_reader::model::PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: jarde_reader::model::ClassBytesId {
                    digest: jarde_reader::model::Digest(
                        blake3::hash(ARRAY_RETURN_CLASS).to_hex().to_string(),
                    ),
                    length: u64::try_from(ARRAY_RETURN_CLASS.len()).expect("fixture length fits"),
                },
                variant: jarde_reader::model::PhysicalVariant::Base,
            },
            name: jarde_reader::model::JvmBytes(b"collectionGridDirect".to_vec()),
            descriptor: jarde_reader::model::JvmBytes(b"()[[Ljava/util/Collection;".to_vec()),
        };
        let analysis_request = jarde_jvm::ir::MethodAnalysisRequest {
            environment: jarde_jvm::environment::ResolutionEnvironment {
                runtime: jarde_reader::view::RuntimeView {
                    physical: jarde_reader::view::PhysicalView {
                        snapshot: snapshot.id().clone(),
                        scope: jarde_reader::view::PhysicalScope::SnapshotAll,
                    },
                    profile: jarde_reader::view::RuntimeProfile {
                        java_release: 8,
                        multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
                        layout: jarde_reader::view::LayoutMode::Generic,
                    },
                    load_domain: domain.clone(),
                },
                domains: vec![domain],
                providers: Vec::new(),
            },
            method: method.clone(),
            stages: jarde_jvm::ir::AnalysisStage::ALL.to_vec(),
        };
        let analysis = jarde_jvm::engine::analyze_method_ir(
            std::slice::from_ref(&snapshot),
            &analysis_request,
            &mut setup_budget,
        )
        .expect("the physical array method analyzes");
        let recovery_facts = jarde_java::RecoveryFacts::new(
            jarde_java::MethodFacts::new("collectionGridDirect", "()[[Ljava/util/Collection;", 0)
                .with_access_flags(0x0009)
                .with_declaring_class(jarde_java::DeclaringClass::new("Main", 0x0021)),
        );
        let request = jarde_java::RecoveryRequest::new(
            analysis.ir(),
            &recovery_facts,
            jarde_java::pass::JAVA_8,
            jarde_reader::view::RuntimeProfile {
                java_release: 8,
                multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
                layout: jarde_reader::view::LayoutMode::Generic,
            },
        );
        let reference_relations = jarde_java::source_proof::prove_reference_relations(
            &analysis_request,
            analysis.ir(),
            &mut MissingHeaders {
                loader: analysis_request
                    .environment
                    .runtime
                    .load_domain
                    .loader
                    .clone(),
            },
            &mut setup_budget,
        )
        .expect("the direct-array reference relation walk completes");
        let request = request.with_reference_relations(&reference_relations);
        let mut recovery_budget = Budget::new(unlimited_annotation_test_limits());
        let recovered = jarde_java::report::recover_for_class_source(
            &request,
            &mut recovery_budget,
            jarde_java::report::ClassSourceOptions {
                prove_generic_return: true,
                ast_retention: jarde_java::report::ClassSourceAstRetention::All,

                ..Default::default()
            },
            None,
        );
        let candidate = recovered
            .candidates
            .generic_return
            .clone()
            .expect("the frozen complete body supplies an array candidate");
        assert!(matches!(
            candidate.value,
            GenericReturnValue::ArrayCreation { .. }
        ));
        let jvm_string = |value: &[u8]| {
            let text = std::str::from_utf8(value).expect("probe strings are ASCII");
            serde_json::from_value(serde_json::json!({
                "raw": value,
                "utf16": value.iter().map(|byte| u16::from(*byte)).collect::<Vec<_>>(),
                "escaped": text,
            }))
            .expect("valid probe JVM string")
        };
        let descriptor_bytes = method.descriptor.0.clone();
        let item = MethodItem {
            index: 0,
            identity: method,
            name: jvm_string(b"collectionGridDirect"),
            descriptor: jvm_string(&descriptor_bytes),
            access_flags: 0x0009,
            body: crate::facade::MemberBodyEvidence::CodeAttribute {
                content_span: jarde_reader::model::ByteSpan::new(0, 0),
            },
        };
        let spelled = Spelled {
            declaration: Some(
                "public static java.util.Collection[][] collectionGridDirect()".to_owned(),
            ),
            marker: None,
            annotations: MemberAnnotationUses::default(),
            parameter_annotations: ParameterAnnotationUses::default(),
            type_annotations: TypeAnnotationUses::default(),
        };
        let run_facts = ClassSourceRunFacts {
            execution: recovered.report.execution.clone(),
            diagnostics: u64::try_from(recovered.report.diagnostics.len())
                .expect("diagnostic count fits"),
        };
        let record = ClassSourceMethod::recovered(
            item,
            spelled,
            Box::new(recovered.report),
            run_facts,
            Some(candidate.clone()),
        );
        let attributes = MemberAttributes {
            default: None,
            throws: Vec::new(),
            throws_raw: Vec::new(),
            parameters: None,
        };
        (record, attributes, candidate)
    }

    fn generic_void_probe() -> (
        ClassSourceMethod,
        MemberAttributes,
        jarde_reader::signature::MethodSignature,
        GenericReturnCandidate,
        Vec<jarde_reader::signature::TypeParameterErasure>,
    ) {
        use jarde_reader::model::{
            ClassBytesId, Digest, PhysicalClassLocation, PhysicalVariant, SnapshotId,
        };
        use jarde_reader::signature::{ClassType, ClassTypeSegment, SignatureType, TypeParameter};

        let owner = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: SnapshotId("void-generic-probe".to_owned()),
            },
            class_bytes: ClassBytesId {
                digest: Digest("void-generic-probe".to_owned()),
                length: 0,
            },
            variant: PhysicalVariant::Base,
        };
        let descriptor_bytes = b"(Lprobe/Bound;Z)V".to_vec();
        let jvm_string = |value: &[u8]| {
            let text = std::str::from_utf8(value).expect("probe strings are ASCII");
            serde_json::from_value(serde_json::json!({
                "raw": value,
                "utf16": value.iter().map(|byte| u16::from(*byte)).collect::<Vec<_>>(),
                "escaped": text,
            }))
            .expect("valid probe JVM string")
        };
        let name: JvmString = jvm_string(b"set");
        let descriptor: JvmString = jvm_string(&descriptor_bytes);
        let method = MethodItem {
            index: 0,
            identity: PhysicalMethodId {
                owner,
                name: JvmBytes(b"set".to_vec()),
                descriptor: JvmBytes(descriptor_bytes.clone()),
            },
            name,
            descriptor,
            access_flags: ACC_PUBLIC,
            body: crate::facade::MemberBodyEvidence::CodeAttribute {
                content_span: jarde_reader::model::ByteSpan::new(0, 0),
            },
        };
        let record = ClassSourceMethod {
            item: method,
            no_body_kind: None,
            declaration: Some("public void set(probe.Bound arg1, boolean arg2)".to_owned()),
            annotations: MemberAnnotationUses::default(),
            parameter_annotations: ParameterAnnotationUses::default(),
            type_annotations: TypeAnnotationUses::default(),
            text: "    public void set(probe.Bound arg1, boolean arg2) { }\n".to_owned(),
            markers: Vec::new(),
            outcome: ClassSourceOutcome::NoBody,
            same_run_generic_return: None,
            same_class_generic_body_proof: None,
            generic_signature_refused: false,
            generic_signature_projected: false,
            enum_constructor_source_tail: EnumConstructorSourceTail::Unrecognized,
            enum_constructor_signature_erasure_refused: false,
        };
        let attributes = MemberAttributes {
            default: None,
            throws: Vec::new(),
            throws_raw: Vec::new(),
            parameters: None,
        };
        let parsed = jarde_reader::signature::MethodSignature {
            type_parameters: vec![TypeParameter {
                name: b"T".to_vec(),
                class_bound: Some(SignatureType::Class(ClassType {
                    segments: vec![ClassTypeSegment {
                        binary_name: b"probe/Bound".to_vec(),
                        arguments: Vec::new(),
                    }],
                })),
                interface_bounds: Vec::new(),
            }],
            parameters: vec![
                SignatureType::TypeVariable(b"T".to_vec()),
                SignatureType::Base(b'Z'),
            ],
            result: None,
            throws: Vec::new(),
        };
        let candidate = GenericReturnCandidate {
            #[cfg(feature = "test-support")]
            observation: jarde_java::observation::Token::candidate(),
            parameters: vec![(1, "arg1".to_owned()), (2, "arg2".to_owned())],
            value: GenericReturnValue::CompleteBody {
                result_type: None,
                unread_parameter_slots: vec![],
                parameter_read_bcis: vec![],
                parameter_receiver_calls: vec![],
            },
        };
        let erasures = vec![jarde_reader::signature::TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Lprobe/Bound;".to_vec(),
        }];
        (record, attributes, parsed, candidate, erasures)
    }

    #[test]
    fn void_generic_signature_stops_before_publishing_on_cancel_or_analysis_budget() {
        let (record, attributes, parsed, candidate, erasures) = generic_void_probe();
        let mut detached_record = record.clone();
        let projection = detached_record.take_generic_call_projection_state();
        assert!(detached_record.declaration.is_none());
        assert!(detached_record.text.is_empty());
        assert!(detached_record.markers.is_empty());
        let before = detached_record.clone();

        let mut success_budget = Budget::new(unlimited_annotation_test_limits());
        let declaration = generic_void_body_declaration(
            &detached_record,
            &projection,
            &attributes,
            &parsed,
            Some(&candidate),
            &erasures,
            b"probe/Setter",
            &[],
            &mut success_budget,
        )
        .expect("Java selects the generic void route")
        .expect("state-aware generic void proof reads the retained overlay");
        assert!(declaration.contains("<T extends probe.Bound> void set(T arg1, boolean arg2)"));

        let cancelled = jarde_reader::budget::CancellationToken::new();
        cancelled.cancel();
        let mut cancelled_budget =
            Budget::with_cancellation_token(unlimited_annotation_test_limits(), cancelled);
        let error = generic_void_body_declaration(
            &detached_record,
            &projection,
            &attributes,
            &parsed,
            Some(&candidate),
            &erasures,
            b"probe/Setter",
            &[],
            &mut cancelled_budget,
        )
        .expect("Java selects the generic void route")
        .expect_err("pre-cancellation stops the void Signature proof");
        assert!(matches!(error, Error::Cancelled { .. }));
        assert_eq!(detached_record, before);

        let mut limits = unlimited_annotation_test_limits();
        limits.analysis_steps = 0;
        let mut exhausted = Budget::new(limits);
        let error = generic_void_body_declaration(
            &detached_record,
            &projection,
            &attributes,
            &parsed,
            Some(&candidate),
            &erasures,
            b"probe/Setter",
            &[],
            &mut exhausted,
        )
        .expect("Java selects the generic void route")
        .expect_err("an exhausted analysis budget stops the void Signature proof");
        assert!(matches!(error, Error::BudgetExceeded { .. }));
        assert_eq!(detached_record, before);
    }

    #[test]
    fn declared_projection_replay_requires_exact_physical_body_and_recorded_metadata() {
        let (mut record, _, _) = array_return_projection_probe();
        record
            .project_generic(
                "public static java.util.Collection<?>[][] collectionGridDirect()".to_owned(),
                b"()[[Ljava/util/Collection<*>;",
                "same-run array-return proof",
                &mut Budget::new(unlimited_annotation_test_limits()),
            )
            .unwrap();
        let ClassSourceOutcome::Recovered { report, .. } = &record.outcome else {
            panic!("the real fixture retains its physical body");
        };
        assert!(!record.matches_current_text(&report.text));
        assert!(
            record
                .matches_declared_projection_input(&mut Budget::new(
                    unlimited_annotation_test_limits()
                ))
                .unwrap()
        );
        let marker_line = format!("        {}\n", record.markers[0]);
        assert!(record.text.contains(&marker_line));
        assert!(record.text.contains("        return "));
        for text in [
            record
                .text
                .replace("        return ", "        return null; // "),
            record.text.replace("// @method ", "// @method stale "),
            record.text.replace(&marker_line, ""),
            record.text.replace(
                &marker_line,
                &format!("{marker_line}        // unrecorded prior projection\n"),
            ),
        ] {
            assert_ne!(text, record.text);
            let mut changed = record.clone();
            changed.text = text;
            assert!(
                !changed
                    .matches_declared_projection_input(&mut Budget::new(
                        unlimited_annotation_test_limits()
                    ))
                    .unwrap(),
                "a stale generic flag does not authorize changed body or metadata: {}",
                changed.text
            );
        }
        let mut changed = record.clone();
        changed.markers[0].push_str(" stale recorded metadata");
        assert!(
            !changed
                .matches_declared_projection_input(&mut Budget::new(
                    unlimited_annotation_test_limits()
                ))
                .unwrap()
        );
        changed = record.clone();
        changed.generic_signature_projected = false;
        assert!(
            !changed
                .matches_declared_projection_input(&mut Budget::new(
                    unlimited_annotation_test_limits()
                ))
                .unwrap(),
            "unowned markers cannot opt another projection into the replay path"
        );
        let mut annotated = record.clone();
        annotated.annotations.uses.push("@Deprecated".into());
        annotated.text = prefix_method_annotations(annotated.text, &annotated.annotations);
        assert!(
            annotated
                .matches_declared_projection_input(&mut Budget::new(
                    unlimited_annotation_test_limits()
                ))
                .unwrap()
        );
        for text in [
            annotated.text.replace("    @Deprecated\n", ""),
            annotated.text.replace("@Deprecated", "@Override"),
            annotated
                .text
                .replace("    @Deprecated\n", "        @Deprecated\n"),
        ] {
            assert_ne!(text, annotated.text);
            let mut changed = annotated.clone();
            changed.text = text;
            assert!(
                !changed
                    .matches_declared_projection_input(&mut Budget::new(
                        unlimited_annotation_test_limits()
                    ))
                    .unwrap(),
                "recorded annotations must retain their exact spelling and placement"
            );
        }
        annotated.annotations.uses[0] = "@Override".into();
        assert!(
            !annotated
                .matches_declared_projection_input(&mut Budget::new(
                    unlimited_annotation_test_limits()
                ))
                .unwrap()
        );
    }

    #[test]
    fn declared_projection_replay_propagates_budget_and_cancellation() {
        let (mut record, _, _) = array_return_projection_probe();
        record
            .project_generic(
                "public static java.util.Collection<?>[][] collectionGridDirect()".to_owned(),
                b"()[[Ljava/util/Collection<*>;",
                "same-run array-return proof",
                &mut Budget::new(unlimited_annotation_test_limits()),
            )
            .unwrap();
        let before = record.clone();
        let mut measured = Budget::new(unlimited_annotation_test_limits());
        assert!(
            record
                .matches_declared_projection_input(&mut measured)
                .unwrap()
        );
        let exact_bytes = measured.usage().output_bytes;
        assert_eq!(exact_bytes, record.text.len() as u64);
        let mut exact = unlimited_annotation_test_limits();
        exact.output_bytes = exact_bytes;
        assert!(
            record
                .matches_declared_projection_input(&mut Budget::new(exact))
                .unwrap()
        );
        let mut short = unlimited_annotation_test_limits();
        short.output_bytes = exact_bytes - 1;
        assert!(matches!(
            record.matches_declared_projection_input(&mut Budget::new(short)),
            Err(Error::BudgetExceeded {
                dimension: crate::BudgetDimension::OutputBytes,
                ..
            })
        ));
        let (raw, _, _) = array_return_projection_probe();
        let mut cancelled_raw = Budget::new(unlimited_annotation_test_limits());
        cancelled_raw.cancellation_token().cancel();
        assert!(matches!(
            raw.matches_declared_projection_input(&mut cancelled_raw),
            Err(Error::Cancelled { .. })
        ));
        let mut limits = unlimited_annotation_test_limits();
        limits.output_bytes = 0;
        assert!(matches!(
            record.matches_declared_projection_input(&mut Budget::new(limits)),
            Err(Error::BudgetExceeded {
                dimension: crate::BudgetDimension::OutputBytes,
                ..
            })
        ));
        let mut limits = unlimited_annotation_test_limits();
        limits.analysis_steps = 0;
        assert!(matches!(
            record.matches_declared_projection_input(&mut Budget::new(limits)),
            Err(Error::BudgetExceeded {
                dimension: crate::BudgetDimension::AnalysisSteps,
                ..
            })
        ));
        let mut cancelled = Budget::new(unlimited_annotation_test_limits());
        cancelled.cancellation_token().cancel();
        assert!(matches!(
            record.matches_declared_projection_input(&mut cancelled),
            Err(Error::Cancelled { .. })
        ));
        assert_eq!(record, before);
    }

    #[test]
    fn generic_call_projection_state_stops_atomically_and_clones_with_metering() {
        let (mut record, _, _, _, _) = generic_void_probe();
        record.no_body_kind = Some(NoBodyKind::Abstract);
        let old_marker = "// jarde: preexisting generic signature refusal".to_owned();
        record.markers.push(old_marker.clone());
        record.text = format!("    {old_marker}\n{}", record.text);

        let mut snapshot_budget = Budget::new(unlimited_annotation_test_limits());
        let mut state = record
            .generic_call_projection_state(&mut snapshot_budget)
            .expect("the pre-attempt projection state is retained");
        let mut raw_clone_budget = Budget::new(unlimited_annotation_test_limits());
        let before = state
            .clone_raw_for_stage(&mut raw_clone_budget)
            .expect("the raw state snapshot is metered");
        let before_record = record.clone();

        let signature = b"<T:Lprobe/Bound;>(TT;Z)V";
        let proof = "same-run AST/Code/SSA void-body proof";
        let marker = format!(
            "// jarde: generic Signature `{}` projected after descriptor erasure and {proof}",
            comment_text(&String::from_utf8_lossy(signature)),
        );
        let mut limits = unlimited_annotation_test_limits();
        limits.output_bytes = u64::try_from(marker.len()).unwrap_or(u64::MAX);
        let mut output_budget = Budget::new(limits);
        let error = state
            .project_generic(
                &record,
                "public <T extends probe.Bound> void set(T arg1, boolean arg2)".to_owned(),
                signature,
                proof,
                &mut output_budget,
            )
            .expect_err("rendered member bytes exceed the remaining output budget");
        assert!(matches!(error, Error::BudgetExceeded { .. }));
        assert_eq!(state, before);
        assert_eq!(record, before_record);

        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(unlimited_annotation_test_limits(), cancellation);
        let error = state
            .project_generic(
                &record,
                "public <T extends probe.Bound> void set(T arg1, boolean arg2)".to_owned(),
                signature,
                proof,
                &mut cancelled,
            )
            .expect_err("pre-cancellation stops before mutating the staged overlay");
        assert!(matches!(error, Error::Cancelled { .. }));
        assert_eq!(state, before);
        assert_eq!(record, before_record);

        let mut no_analysis_limits = unlimited_annotation_test_limits();
        no_analysis_limits.analysis_steps = 0;
        let mut no_analysis = Budget::new(no_analysis_limits);
        assert!(matches!(
            state.clone_raw_for_stage(&mut no_analysis),
            Err(Error::BudgetExceeded {
                dimension: crate::BudgetDimension::AnalysisSteps,
                ..
            })
        ));
        assert_eq!(state, before);
        assert_eq!(record, before_record);

        let mut no_output_limits = unlimited_annotation_test_limits();
        no_output_limits.output_bytes = 0;
        let mut no_output = Budget::new(no_output_limits);
        assert!(matches!(
            state.clone_raw_for_stage(&mut no_output),
            Err(Error::BudgetExceeded {
                dimension: crate::BudgetDimension::OutputBytes,
                ..
            })
        ));
        assert_eq!(state, before);
        assert_eq!(record, before_record);

        let mut metered_copy_budget = Budget::new(unlimited_annotation_test_limits());
        let copy = state
            .clone_raw_for_stage(&mut metered_copy_budget)
            .expect("a funded raw-state clone succeeds");
        assert_eq!(copy, state);
        assert_eq!(state, before);
        assert_eq!(record, before_record);
    }

    #[test]
    fn void_generic_header_output_budget_publishes_no_partial_declaration() {
        let (mut record, _, _, _, _) = generic_void_probe();
        record.no_body_kind = Some(NoBodyKind::Abstract);
        let before = record.clone();
        let mut limits = unlimited_annotation_test_limits();
        limits.output_bytes = 0;
        let error = record
            .project_generic(
                "public <T extends probe.Bound> void set(T arg1, boolean arg2)".to_owned(),
                b"<T:Lprobe/Bound;>(TT;Z)V",
                "same-run AST/SSA void-body proof",
                &mut Budget::new(limits),
            )
            .expect_err("output budget rejects the complete staged projection");
        assert!(matches!(error, Error::BudgetExceeded { .. }));
        assert_eq!(record, before);
        assert!(!record.text.contains("<T extends"));
        assert!(!record.generic_signature_projected);
    }

    #[test]
    fn functional_withholding_uses_projected_state_not_markers_or_source_text() {
        let (mut failed, _, _, _, _) = generic_void_probe();
        failed
            .markers
            .push("// jarde: generic Signature projected".to_owned());
        failed.text = "    public java.util.function.Function raw() { return value; }\n".to_owned();
        failed.withhold_unprojected_functional_body();
        assert!(failed.text.contains("typed functional source"));
        assert!(!failed.text.contains("Function raw"));

        let (mut projected, _, _, _, _) = generic_void_probe();
        projected.no_body_kind = Some(NoBodyKind::Abstract);
        projected
            .project_generic(
                "public <T extends probe.Bound> void set(T arg1, boolean arg2)".to_owned(),
                b"<T:Lprobe/Bound;>(TT;Z)V",
                "same-run AST/SSA void-body proof",
                &mut Budget::new(unlimited_annotation_test_limits()),
            )
            .expect("complete projection fits the budget");
        assert!(projected.generic_signature_projected);
        let text = projected.text.clone();
        projected.withhold_unprojected_functional_body();
        assert_eq!(projected.text, text);
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
    fn class_header_spelling_keeps_pool_names_the_member_spelling_refuses() {
        // The design premise this pin answers: the member/annotation spelling channel refuses
        // every `$`-named single-segment class, so the class header needs its own leaf rule —
        // the class file's own binary name (`class_name`), which is what the raw header already
        // states for every parent no `Signature` projects. Without the header rule the whole
        // parent MVP and the interface projection could not spell a single `$` name.
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let parsed = parse_method_signature(
            b"(LBR$Box<Ljava/lang/String;>;Ljava/lang/Comparable<LBR$Impl;>;Lp/Outer$Mid$Leaf<Ljava/lang/Integer;>;Ljava/lang/String;)V",
            &mut budget,
        )
        .unwrap();
        let written = parsed
            .parameters
            .iter()
            .map(|ty| spell_class_header_signature_type(ty, &[], &mut budget, 0).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            written,
            [
                "BR$Box<java.lang.String>",
                "java.lang.Comparable<BR$Impl>",
                "p.Outer$Mid$Leaf<java.lang.Integer>",
                "java.lang.String",
            ]
        );
        // The same types through the member spelling: the `$` names refuse (the boundary
        // `ordinary-parameterized-signatures` pinned), and the no-`$` name is byte-identical
        // between the two rules — the header rule changes nothing but the `$` refusal.
        assert_eq!(
            spell_ordinary_signature_type(&parsed.parameters[3], &[], &mut budget, 0).unwrap(),
            "java.lang.String"
        );
        for index in [0, 1, 2] {
            assert!(matches!(
                spell_ordinary_signature_type(&parsed.parameters[index], &[], &mut budget, 0),
                Err(Error::Unsupported { .. })
            ));
        }
        // A name that is not a Java identifier has no spelling under either rule.
        let bad = parse_method_signature(b"(Lp/Bad-Name;)V", &mut budget).unwrap();
        assert!(matches!(
            spell_class_header_signature_type(&bad.parameters[0], &[], &mut budget, 0),
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
            jarde_java::report::MemberInnerSourceSegmentInput {
                definition: identity.clone(),
                binary_name: jarde_reader::model::JvmBytes::from_unicode("matrix/Outer"),
                source_name: "matrix.Outer".to_owned(),
                type_parameter_count: 0,
                enclosing_binary_name: None,
                is_static: true,
            },
            jarde_java::report::MemberInnerSourceSegmentInput {
                definition: identity.clone(),
                binary_name: jarde_reader::model::JvmBytes::from_unicode("matrix/Outer$A"),
                source_name: "matrix.Outer.A".to_owned(),
                type_parameter_count: 1,
                enclosing_binary_name: Some(jarde_reader::model::JvmBytes::from_unicode(
                    "matrix/Outer",
                )),
                is_static: true,
            },
            jarde_java::report::MemberInnerSourceSegmentInput {
                definition: identity,
                binary_name: jarde_reader::model::JvmBytes::from_unicode("matrix/Outer$A$Generic"),
                source_name: "matrix.Outer.A.Generic".to_owned(),
                type_parameter_count: 1,
                enclosing_binary_name: Some(jarde_reader::model::JvmBytes::from_unicode(
                    "matrix/Outer$A",
                )),
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

    #[test]
    fn method_parameters_need_a_complete_safe_slot_projection() {
        let entry = |name: Option<&[u8]>, flags| jarde_reader::classfile::MethodParameterFacts {
            name_index: u16::from(name.is_some()),
            name: name.map(|name| jarde_reader::model::JvmBytes(name.to_vec())),
            access_flags: flags,
        };
        let entries = [entry(Some(b"first"), 0), entry(Some(b"second"), ACC_FINAL)];
        let proved = prove_method_parameters(&entries, b"(JDI)V", 0);
        assert!(proved.is_none(), "descriptor count must match");
        let proved = prove_method_parameters(&entries, b"(JD)V", 0).unwrap();
        assert_eq!(
            proved.iter().map(|item| item.slot).collect::<Vec<_>>(),
            [1, 3]
        );
        assert!(proved[1].is_final);
        let static_proved = prove_method_parameters(&entries, b"(JD)V", ACC_STATIC).unwrap();
        assert_eq!(
            static_proved
                .iter()
                .map(|item| item.slot)
                .collect::<Vec<_>>(),
            [0, 2]
        );
        for entries in [
            [entry(None, 0), entry(Some(b"second"), 0)],
            [entry(Some(b"first"), 0x1000), entry(Some(b"second"), 0)],
            [entry(Some(b"first"), 0x8000), entry(Some(b"second"), 0)],
        ] {
            assert!(prove_method_parameters(&entries, b"(JD)V", 0).is_none());
        }
        assert!(prove_method_parameters(&entries, b"(JD)V", 0x0040).is_none());
        assert!(
            prove_method_parameters(
                &[entry(Some(b"wide"), 0), entry(Some(b"arg2"), 0)],
                b"(JLjava/lang/String;)V",
                0,
            )
            .is_some(),
            "source spelling resolves a wide-slot fallback collision after slot proof"
        );
    }

    #[test]
    fn method_parameter_names_preserve_raw_metadata_and_share_body_spelling() {
        let names = [
            JvmBytes::from_unicode("𐐀"),
            JvmBytes(vec![0xed, 0xa0, 0x81]),
            JvmBytes::from_unicode("class"),
            JvmBytes::from_unicode("class"),
        ];
        let entries = names
            .iter()
            .map(|name| jarde_reader::classfile::MethodParameterFacts {
                name_index: 1,
                name: Some(name.clone()),
                access_flags: 0,
            })
            .collect::<Vec<_>>();
        let proved = prove_method_parameters(&entries, b"(IIII)V", ACC_STATIC).unwrap();
        assert_eq!(
            proved
                .iter()
                .map(|parameter| &parameter.name)
                .collect::<Vec<_>>(),
            names.iter().collect::<Vec<_>>()
        );
        let evidence = proved
            .iter()
            .map(|parameter| SlotEvidence::Whole(parameter.name.clone()))
            .collect::<Vec<_>>();
        let table = NameTable::build(4, 4, &evidence);
        assert_eq!(table.whole(0).unwrap().text(), "𐐀");
        assert!(table.whole(1).unwrap().aliased().is_some());
        assert_ne!(
            table.whole(2).unwrap().text(),
            table.whole(3).unwrap().text()
        );
        for raw in [vec![0], vec![0xf0, 0x90, 0x90, 0x80], vec![0xed, 0xa0]] {
            let entries = [jarde_reader::classfile::MethodParameterFacts {
                name_index: 1,
                name: Some(JvmBytes(raw)),
                access_flags: 0,
            }];
            assert!(prove_method_parameters(&entries, b"(I)V", ACC_STATIC).is_none());
        }
    }

    #[test]
    fn wildcard_array_return_projection_requires_the_same_reifiable_array_shape() {
        let physical = "java.util.Collection[][]";
        for (signature, physical, expected) in [
            (
                b"()[[Ljava/util/Collection<*>;".as_slice(),
                "java.util.Collection[][]",
                true,
            ),
            (
                b"()[Ljava/util/Map<**>;".as_slice(),
                "java.util.Map[]",
                true,
            ),
            (
                b"()[[Ljava/util/Collection<Ljava/lang/String;>;".as_slice(),
                "java.util.Collection[][]",
                false,
            ),
            (
                b"()[[Ljava/util/Collection<+Ljava/lang/Number;>;".as_slice(),
                "java.util.Collection[][]",
                false,
            ),
            (
                b"()[[Ljava/util/Collection<-Ljava/lang/String;>;".as_slice(),
                "java.util.Collection[][]",
                false,
            ),
            (b"()TT;".as_slice(), "java.util.Collection[][]", false),
            (
                b"()[Ljava/util/Collection<*>;".as_slice(),
                "java.util.Collection[][]",
                false,
            ),
            (
                b"()[[Ljava/util/List<*>;".as_slice(),
                "java.util.Collection[][]",
                false,
            ),
            (b"()V".as_slice(), "java.util.Collection[][]", false),
        ] {
            let mut budget = Budget::new(unlimited_annotation_test_limits());
            let parsed = parse_method_signature(signature, &mut budget).unwrap();
            assert_eq!(
                wildcard_array_return_matches(
                    physical,
                    parsed.result.as_ref(),
                    Some(physical),
                    &mut budget,
                )
                .unwrap(),
                expected,
                "Signature shape {signature:?}"
            );
        }
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let parsed = parse_method_signature(b"()[[Ljava/util/Collection<*>;", &mut budget).unwrap();
        assert!(
            !wildcard_array_return_matches(
                "java.util.Collection[]",
                parsed.result.as_ref(),
                Some(physical),
                &mut budget,
            )
            .unwrap(),
            "the same-run array spelling must equal the physical descriptor spelling"
        );
    }

    #[test]
    fn ordinary_parameterized_declaration_requires_array_candidate_and_unbounded_wildcards() {
        let (record, attributes, candidate) = array_return_projection_probe();
        let physical_descriptor = record.item.descriptor.raw().0.as_slice();
        for (signature, erasure_matches, accepted) in [
            (b"()[[Ljava/util/Collection<*>;".as_slice(), true, true),
            (
                b"()[[Ljava/util/Collection<Ljava/lang/Object;>;".as_slice(),
                true,
                false,
            ),
            (
                b"()[[Ljava/util/Collection<+Ljava/lang/Object;>;".as_slice(),
                true,
                false,
            ),
            (
                b"()[[Ljava/util/Collection<-Ljava/lang/Object;>;".as_slice(),
                true,
                false,
            ),
            (b"<T:Ljava/util/Collection;>()[[TT;".as_slice(), true, false),
            (b"()[Ljava/util/Collection<*>;".as_slice(), false, false),
            (b"()[[Ljava/util/List<*>;".as_slice(), false, false),
        ] {
            let mut budget = Budget::new(unlimited_annotation_test_limits());
            let parsed = parse_method_signature(signature, &mut budget).unwrap();
            if erasure_matches {
                prove_method_signature_erasure_with_class_scope(
                    &parsed,
                    physical_descriptor,
                    &[],
                    &[],
                    &mut budget,
                )
                .expect("same-erasure Signature control is reader-valid");
            }
            let projected = ordinary_parameterized_declaration(
                &record,
                None,
                &attributes,
                &parsed,
                Some(&candidate),
                0x0021,
                b"Main",
                Some(b"java/lang/Object"),
                &[],
                &[],
                false,
                &[],
                &mut budget,
            );
            if accepted {
                assert_eq!(
                    projected.unwrap().0,
                    "public static java.util.Collection<?>[][] collectionGridDirect()"
                );
            } else {
                assert!(
                    projected.is_err(),
                    "ordinary declaration path accepted Signature {signature:?}"
                );
            }
        }

        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let parsed = parse_method_signature(b"()[[Ljava/util/Collection<*>;", &mut budget).unwrap();
        let missing = ordinary_parameterized_declaration(
            &record,
            None,
            &attributes,
            &parsed,
            None,
            0x0021,
            b"Main",
            Some(b"java/lang/Object"),
            &[],
            &[],
            false,
            &[],
            &mut budget,
        );
        assert!(
            missing.is_err(),
            "the real declaration path requires its same-run candidate"
        );
    }

    #[test]
    fn wildcard_array_declaration_gate_propagates_budget_and_cancellation() {
        let parsed = parse_method_signature(
            b"()[[Ljava/util/Collection<*>;",
            &mut Budget::new(unlimited_annotation_test_limits()),
        )
        .unwrap();
        let cancelled = jarde_reader::budget::CancellationToken::new();
        cancelled.cancel();
        let mut cancelled_budget =
            Budget::with_cancellation_token(unlimited_annotation_test_limits(), cancelled);
        assert!(matches!(
            wildcard_array_return_matches(
                "java.util.Collection[][]",
                parsed.result.as_ref(),
                Some("java.util.Collection[][]"),
                &mut cancelled_budget,
            ),
            Err(Error::Cancelled { .. })
        ));

        let mut limits = unlimited_annotation_test_limits();
        limits.analysis_steps = 0;
        let mut exhausted = Budget::new(limits);
        assert!(matches!(
            wildcard_array_return_matches(
                "java.util.Collection[][]",
                parsed.result.as_ref(),
                Some("java.util.Collection[][]"),
                &mut exhausted,
            ),
            Err(Error::BudgetExceeded { .. })
        ));

        let mut limits = unlimited_annotation_test_limits();
        limits.analysis_steps = 5;
        let mut exhausted_during_arguments = Budget::new(limits);
        assert!(matches!(
            wildcard_array_return_matches(
                "java.util.Collection[][]",
                parsed.result.as_ref(),
                Some("java.util.Collection[][]"),
                &mut exhausted_during_arguments,
            ),
            Err(Error::BudgetExceeded {
                dimension: crate::BudgetDimension::AnalysisSteps,
                ..
            })
        ));
    }

    #[test]
    fn projected_parameter_names_and_final_are_one_declaration() {
        let signature =
            method_descriptor(b"(Ljava/lang/String;I)Ljava/lang/String;", false, false).unwrap();
        let parameters = vec![
            ProvedMethodParameter {
                slot: 1,
                name: JvmBytes::from_unicode("paramStr"),
                is_final: false,
            },
            ProvedMethodParameter {
                slot: 2,
                name: JvmBytes::from_unicode("number"),
                is_final: true,
            },
        ];
        let facts = RecoveryFacts::new(
            jarde_java::MethodFacts::new("named", "(Ljava/lang/String;I)Ljava/lang/String;", 3)
                .with_access_flags(0),
        )
        .with_debug_locals(vec![
            jarde_java::DebugLocal::named(1, "paramStr"),
            jarde_java::DebugLocal::named(2, "number"),
        ]);
        assert_eq!(
            arguments(
                Some(&facts),
                None,
                &signature,
                SourceClassNames::default(),
                &[],
                &[],
                Some(&parameters)
            ),
            "(java.lang.String paramStr, final int number)"
        );
        assert_eq!(
            arguments(
                None,
                None,
                &signature,
                SourceClassNames::default(),
                &[],
                &[],
                Some(&parameters)
            ),
            "(java.lang.String arg1, int arg2)"
        );
        let lvt = facts.clone().with_debug_locals(vec![
            jarde_java::DebugLocal::over(1, "paramStr", 0, 4),
            jarde_java::DebugLocal::over(2, "number", 0, 4),
        ]);
        assert_eq!(
            arguments(
                Some(&lvt),
                None,
                &signature,
                SourceClassNames::default(),
                &[],
                &[],
                Some(&parameters)
            ),
            "(java.lang.String paramStr, int number)"
        );
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
        assert_eq!(
            arguments(
                None,
                None,
                &ints,
                SourceClassNames::default(),
                &[],
                &[],
                None
            ),
            "(int arg0, int... arg1)"
        );

        // `[[B`: the dots take the place of the **last** `[]`, so the element type keeps the `[]`
        // it has and the parameter is written `byte[]...`, never `byte[][]` or `byte...`.
        let grid = method_descriptor(b"([[B)V", true, true).expect("a method descriptor");
        assert_eq!(
            arguments(
                None,
                None,
                &grid,
                SourceClassNames::default(),
                &[],
                &[],
                None
            ),
            "(byte[]... arg0)"
        );

        // The controls. The same descriptor without the flag keeps the array spelling, and the flag
        // on a member whose last parameter is not an array is written as the descriptor states it.
        let without_the_flag =
            method_descriptor(b"(I[I)V", true, false).expect("a method descriptor");
        assert!(!without_the_flag.varargs);
        assert_eq!(
            arguments(
                None,
                None,
                &without_the_flag,
                SourceClassNames::default(),
                &[],
                &[],
                None
            ),
            "(int arg0, int[] arg1)"
        );
        let not_an_array = method_descriptor(b"(II)V", true, true).expect("a method descriptor");
        assert!(!not_an_array.varargs, "no `...` is invented for an `int`");
        assert_eq!(
            arguments(
                None,
                None,
                &not_an_array,
                SourceClassNames::default(),
                &[],
                &[],
                None
            ),
            "(int arg0, int arg1)"
        );
        let no_parameters = method_descriptor(b"()V", true, true).expect("a method descriptor");
        assert!(
            !no_parameters.varargs,
            "there is no last parameter to reach"
        );
        assert_eq!(
            arguments(
                None,
                None,
                &no_parameters,
                SourceClassNames::default(),
                &[],
                &[],
                None
            ),
            "()"
        );

        // The flag reaches the descriptor's last parameter and not the slot the receiver holds: an
        // instance member's `this` is no position of the list, so the dots land on slot 1 here.
        let instance = method_descriptor(b"([I)V", false, true).expect("a method descriptor");
        assert_eq!(
            arguments(
                None,
                None,
                &instance,
                SourceClassNames::default(),
                &[],
                &[],
                None
            ),
            "(int... arg1)"
        );
    }

    #[test]
    fn recovered_names_follow_parameter_slots_across_a_wide_value() {
        let signature = method_descriptor(b"(JI)I", true, false).expect("a method descriptor");
        let recovered = [Some("arg0_2".to_owned()), None, Some("arg2_2".to_owned())];
        assert_eq!(
            arguments(
                None,
                Some(&recovered),
                &signature,
                SourceClassNames::default(),
                &[],
                &[],
                None
            ),
            "(long arg0_2, int arg2_2)"
        );
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
        assert_eq!(
            class_name(b"p/Outer$Inner"),
            Some("p.Outer$Inner".to_owned())
        );
        assert_eq!(
            class_name(b"HistoricalControlFlow"),
            Some("HistoricalControlFlow".to_owned())
        );
        assert_eq!(class_name(b"a/b/C"), Some("a.b.C".to_owned()));
    }

    #[test]
    fn a_binary_name_is_split_where_its_own_last_slash_is() {
        // The class's own name: the package is everything before the **last** `/`, and a `$` inside
        // either part stays where it is — no nesting relation is read out of it.
        assert_eq!(simple_name(b"a/b/C"), Some("C".to_owned()));
        assert_eq!(
            simple_name(b"p/Outer$Inner"),
            Some("Outer$Inner".to_owned())
        );
        assert_eq!(simple_name(b"p/More17$1"), Some("More17$1".to_owned()));
        assert_eq!(simple_name(b"C"), Some("C".to_owned()));
        assert_eq!(simple_name(b"p/_"), Some("_".to_owned()));
        for raw in [
            b"p/a-b".as_slice(),
            b"p/class",
            b"p/9Box",
            b"p/package-info",
        ] {
            assert_eq!(simple_name(raw), None);
        }

        // The package line the same split states; a name with no `/` is the default package, which
        // writes no line at all.
        assert_eq!(package_name(b"a/b/C"), Some("a.b".to_owned()));
        assert_eq!(package_name(b"p/Outer$Inner"), Some("p".to_owned()));
        assert_eq!(package_name(b"C"), None);
    }

    #[test]
    fn raw_class_names_decode_mutf8_without_using_display_identity() {
        let raw = JvmBytes::from_unicode("𐐀pkg/𐐁Outer$𐐂Inner");
        assert_eq!(class_name(&raw.0), Some("𐐀pkg.𐐁Outer$𐐂Inner".to_owned()));
        assert_eq!(simple_name(&raw.0), Some("𐐁Outer$𐐂Inner".to_owned()));
        assert_eq!(package_name(&raw.0), Some("𐐀pkg".to_owned()));
        assert!(legal_package_name(&JvmBytes::from_unicode("𐐀pkg/𐐁part").0));
        assert_eq!(class_name(b"p/\xed\xa0\x81"), None);
        assert_eq!(simple_name(b"p/\xed\xa0\x81"), None);
        assert_eq!(class_name(b"p/\xf0\x90\x90\x80"), None);
        assert_eq!(class_name(&JvmBytes::from_unicode("p/\u{fffd}").0), None);
        assert!(!legal_package_name(b"p/\xed\xa0\x81"));
        assert!(!legal_package_name(b"p/bad-name"));
    }

    #[test]
    fn raw_field_supplementary_declaration_keeps_its_scalar_spelling() {
        let name = JvmBytes::from_unicode("𐐀count");
        assert_eq!(written_name(&name.0), ("𐐀count".to_owned(), false));
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
    fn shared_member_writer_lengths_preserve_unicode_blank_lines_and_annotations() {
        let declaration = "public void 𐐀雪()";
        let marker_sets = [
            vec![],
            vec![String::new()],
            vec!["// 𐐀 marker".to_owned()],
            vec!["// first\n\n// 雪\n".to_owned()],
        ];
        let envelopes = ["", "// @method 𐐀雪()V\n", "\n// 注\n\n"];
        let bodies = ["", "    return;\n", "    String value = \"𐐀雪\";\n\n"];
        let annotation_sets = [vec![], vec!["@p.𐐀A".to_owned(), "@p.注".to_owned()]];
        let mut checked = 0;
        for markers in &marker_sets {
            for envelope in envelopes {
                for body in bodies {
                    for annotations in &annotation_sets {
                        let uses = MemberAnnotationUses {
                            uses: annotations.clone(),
                            ..Default::default()
                        };
                        let block = prefix_method_annotations(
                            block_member(
                                declaration,
                                Placed::Block(Artifact {
                                    envelope,
                                    statements: body,
                                }),
                                markers,
                            ),
                            &uses,
                        );
                        assert_eq!(
                            block_member_text_bytes(declaration, envelope, body, markers, &uses),
                            u64::try_from(block.len()).unwrap(),
                            "UTF-8 bytes and blank-line indentation must match real rendering"
                        );
                        let no_body = prefix_method_annotations(
                            declaration_member(declaration, markers),
                            &uses,
                        );
                        assert_eq!(
                            declaration_member_text_bytes(declaration, markers, &uses),
                            u64::try_from(no_body.len()).unwrap()
                        );
                        if !annotations.is_empty() {
                            assert!(block.starts_with("    @p.𐐀A\n    @p.注\n"));
                            assert!(no_body.starts_with("    @p.𐐀A\n    @p.注\n"));
                        }
                        assert!(block.ends_with("    }\n"));
                        assert!(no_body.ends_with("public void 𐐀雪();\n"));
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(checked, 72);
        println!("BU_WRITER_LENGTH pairs={checked}");
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

    #[test]
    fn field_write_parameter_proof_distinguishes_published_generic_and_raw_parameters() {
        use jarde_reader::signature::{ClassType, ClassTypeSegment, SignatureType, TypeArgument};

        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let field = parse_field_signature(b"Ljava/util/List<TT;>;", &mut budget)
            .expect("the target List<T> signature parses")
            .ty;
        let published =
            parse_method_signature(b"(Ljava/util/List<Ljava/lang/String;>;)V", &mut budget)
                .expect("the published List<String> parameter parses");
        let raw_descriptor = b"Ljava/util/List;".to_vec();
        let method_identity = generic_void_probe().0.item.identity;
        let source = SameClassFieldWriteSource::Parameter {
            method: method_identity.clone(),
            slot: 1,
        };
        let class_scope = vec![jarde_reader::signature::TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Object;".to_vec(),
        }];
        let string_literal = SameClassFieldWriteSource::StringConstant { value_bci: 7 };
        let mut literal_budget = Budget::new(unlimited_annotation_test_limits());
        assert!(!field_write_source_assignable(
            &string_literal,
            &field,
            &class_scope,
            &[],
            8,
        ));
        assert!(
            !raw_field_write_source_assignable(
                &string_literal,
                b"Ljava/lang/String;",
                &[],
                8,
                &mut literal_budget,
            )
            .expect("string constant is deliberately excluded from generic field proofs")
        );
        let published_method = SameClassPublishedMethodParameters {
            method: method_identity.clone(),
            parameter_slots: vec![1],
            descriptor_parameters: vec![raw_descriptor.clone()],
            generic_parameters: Some(published.parameters),
            method_type_parameters: Vec::new(),
        };
        assert!(!field_write_source_assignable(
            &source,
            &field,
            &class_scope,
            &[published_method],
            8,
        ));

        let published_raw_method = SameClassPublishedMethodParameters {
            method: method_identity.clone(),
            parameter_slots: vec![1],
            descriptor_parameters: vec![b"Ljava/util/List;".to_vec()],
            generic_parameters: Some(vec![SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/util/List".to_vec(),
                    arguments: Vec::new(),
                }],
            })]),
            method_type_parameters: Vec::new(),
        };
        assert!(field_write_source_assignable(
            &source,
            &field,
            &class_scope,
            &[published_raw_method],
            8,
        ));
        let published_raw_array_list_method = SameClassPublishedMethodParameters {
            method: method_identity.clone(),
            parameter_slots: vec![1],
            descriptor_parameters: vec![b"Ljava/util/ArrayList;".to_vec()],
            generic_parameters: Some(vec![SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/util/ArrayList".to_vec(),
                    arguments: Vec::new(),
                }],
            })]),
            method_type_parameters: Vec::new(),
        };
        assert!(field_write_source_assignable(
            &source,
            &field,
            &class_scope,
            &[published_raw_array_list_method.clone()],
            8,
        ));
        assert!(!field_write_source_assignable(
            &source,
            &field,
            &class_scope,
            &[published_raw_array_list_method],
            9,
        ));

        let raw_method = SameClassPublishedMethodParameters {
            method: method_identity.clone(),
            parameter_slots: vec![1],
            descriptor_parameters: vec![raw_descriptor],
            generic_parameters: None,
            method_type_parameters: Vec::new(),
        };
        assert!(field_write_source_assignable(
            &source,
            &field,
            &class_scope,
            &[raw_method],
            8,
        ));

        let raw_array_target = SignatureType::Array(Box::new(SignatureType::Class(ClassType {
            segments: vec![ClassTypeSegment {
                binary_name: b"java/util/List".to_vec(),
                arguments: vec![TypeArgument::Exact(SignatureType::TypeVariable(
                    b"T".to_vec(),
                ))],
            }],
        })));
        let array_source = SameClassFieldWriteSource::Parameter {
            method: method_identity.clone(),
            slot: 1,
        };
        let raw_array_method = SameClassPublishedMethodParameters {
            method: method_identity.clone(),
            parameter_slots: vec![1],
            descriptor_parameters: vec![b"[Ljava/util/List;".to_vec()],
            generic_parameters: None,
            method_type_parameters: Vec::new(),
        };
        assert!(field_write_source_assignable(
            &array_source,
            &raw_array_target,
            &class_scope,
            &[raw_array_method],
            8,
        ));
        let published_raw_array_method = SameClassPublishedMethodParameters {
            method: method_identity.clone(),
            parameter_slots: vec![1],
            descriptor_parameters: vec![b"[Ljava/util/List;".to_vec()],
            generic_parameters: Some(vec![SignatureType::Array(Box::new(SignatureType::Class(
                ClassType {
                    segments: vec![ClassTypeSegment {
                        binary_name: b"java/util/List".to_vec(),
                        arguments: Vec::new(),
                    }],
                },
            )))]),
            method_type_parameters: Vec::new(),
        };
        assert!(field_write_source_assignable(
            &array_source,
            &raw_array_target,
            &class_scope,
            &[published_raw_array_method],
            8,
        ));

        let shadowed_method = SameClassPublishedMethodParameters {
            method: method_identity,
            parameter_slots: vec![1],
            descriptor_parameters: vec![b"Ljava/lang/Object;".to_vec()],
            generic_parameters: Some(vec![SignatureType::TypeVariable(b"T".to_vec())]),
            method_type_parameters: vec![TypeParameter {
                name: b"T".to_vec(),
                class_bound: None,
                interface_bounds: Vec::new(),
            }],
        };
        let class_variable_source = SameClassFieldWriteSource::Parameter {
            method: shadowed_method.method.clone(),
            slot: 1,
        };
        assert!(!field_write_source_assignable(
            &class_variable_source,
            &SignatureType::TypeVariable(b"T".to_vec()),
            &class_scope,
            &[shadowed_method],
            8,
        ));
    }

    #[test]
    fn raw_parameter_platform_widening_rejects_invalid_utf8_name_collisions() {
        let target = SignatureType::Class(jarde_reader::signature::ClassType {
            segments: vec![jarde_reader::signature::ClassTypeSegment {
                binary_name: vec![b'f', b'o', b'o', b'/', 0x81],
                arguments: vec![jarde_reader::signature::TypeArgument::Exact(
                    SignatureType::TypeVariable(b"T".to_vec()),
                )],
            }],
        });
        assert!(!raw_parameter_assignable(b"Lfoo/\x80;", &target, 8));
    }

    #[test]
    fn published_method_type_variable_uses_only_its_matching_raw_class_bound() {
        use jarde_reader::signature::{ClassType, ClassTypeSegment, TypeArgument};

        let string = || {
            SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/lang/String".to_vec(),
                    arguments: Vec::new(),
                }],
            })
        };
        let target = SignatureType::Class(ClassType {
            segments: vec![ClassTypeSegment {
                binary_name: b"java/util/Map".to_vec(),
                arguments: vec![TypeArgument::Exact(string()), TypeArgument::Exact(string())],
            }],
        });
        let method_identity = generic_void_probe().0.item.identity;
        let source = SameClassFieldWriteSource::Parameter {
            method: method_identity.clone(),
            slot: 1,
        };
        let raw_bound = TypeParameter {
            name: b"R".to_vec(),
            class_bound: Some(SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/util/HashMap".to_vec(),
                    arguments: Vec::new(),
                }],
            })),
            interface_bounds: Vec::new(),
        };
        let method = SameClassPublishedMethodParameters {
            method: method_identity.clone(),
            parameter_slots: vec![1],
            descriptor_parameters: vec![b"Ljava/util/HashMap;".to_vec()],
            generic_parameters: Some(vec![SignatureType::TypeVariable(b"R".to_vec())]),
            method_type_parameters: vec![raw_bound.clone()],
        };
        assert!(field_write_source_assignable(
            &source,
            &target,
            &[],
            &[method.clone()],
            8,
        ));
        assert!(!field_write_source_assignable(
            &source,
            &target,
            &[],
            &[method.clone()],
            9,
        ));

        let parameterized_bound = TypeParameter {
            class_bound: Some(SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/util/HashMap".to_vec(),
                    arguments: vec![TypeArgument::Exact(string()), TypeArgument::Exact(string())],
                }],
            })),
            ..raw_bound.clone()
        };
        let method_with_parameterized_bound = SameClassPublishedMethodParameters {
            method: method_identity.clone(),
            method_type_parameters: vec![parameterized_bound],
            ..method.clone()
        };
        assert!(!field_write_source_assignable(
            &source,
            &target,
            &[],
            &[method_with_parameterized_bound],
            8,
        ));

        let interface_only_bound = TypeParameter {
            class_bound: None,
            interface_bounds: vec![SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/util/HashMap".to_vec(),
                    arguments: Vec::new(),
                }],
            })],
            ..raw_bound
        };
        let method_with_interface_bound = SameClassPublishedMethodParameters {
            method: method_identity,
            method_type_parameters: vec![interface_only_bound],
            ..method
        };
        assert!(!field_write_source_assignable(
            &source,
            &target,
            &[],
            &[method_with_interface_bound],
            8,
        ));
    }

    #[test]
    fn field_write_source_type_walk_charges_exact_work_and_propagates_stops() {
        let mut parse_budget = Budget::new(unlimited_annotation_test_limits());
        let source_type = parse_field_signature(b"Ljava/util/List<TT;>;", &mut parse_budget)
            .expect("the parameter source type parses")
            .ty;
        let target_type = source_type.clone();
        let method_identity = generic_void_probe().0.item.identity;
        let source = SameClassFieldWriteSource::Parameter {
            method: method_identity.clone(),
            slot: 1,
        };
        let methods = [SameClassPublishedMethodParameters {
            method: method_identity,
            parameter_slots: vec![1],
            descriptor_parameters: vec![b"Ljava/util/List;".to_vec()],
            generic_parameters: Some(vec![source_type]),
            method_type_parameters: Vec::new(),
        }];
        let class_scope = [jarde_reader::signature::TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Object;".to_vec(),
        }];

        let mut exact = unlimited_annotation_test_limits();
        exact.analysis_steps = 25;
        let mut exact_budget = Budget::new(exact);
        let assignable = prove_field_write_source_assignable(
            &source,
            &target_type,
            &class_scope,
            &methods,
            8,
            &mut exact_budget,
        )
        .expect("exactly twenty-five type and lookup steps fit");
        assert!(assignable, "the parameter's published type matches");
        assert_eq!(exact_budget.usage().analysis_steps, 25);

        let mut short = unlimited_annotation_test_limits();
        short.analysis_steps = 24;
        let mut short_budget = Budget::new(short);
        assert!(matches!(
            prove_field_write_source_assignable(
                &source,
                &target_type,
                &class_scope,
                &methods,
                8,
                &mut short_budget,
            ),
            Err(Error::BudgetExceeded {
                dimension: crate::BudgetDimension::AnalysisSteps,
                limit: 24,
                consumed: 24,
                ..
            })
        ));
        assert_eq!(short_budget.usage().analysis_steps, 24);

        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(unlimited_annotation_test_limits(), cancellation);
        assert!(matches!(
            prove_field_write_source_assignable(
                &source,
                &target_type,
                &class_scope,
                &methods,
                8,
                &mut cancelled,
            ),
            Err(Error::Cancelled { .. })
        ));
        assert_eq!(cancelled.usage().analysis_steps, 0);
    }

    #[test]
    fn incomplete_same_class_field_inventory_cannot_prove_a_field_binding() {
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let class = jarde_reader::classfile::class_facts(MEMBER_ANNOTATION_TARGET, &mut budget)
            .expect("the frozen class structure reads");
        assert!(!class.fields.is_empty());
        let facts = SameClassUseFacts {
            complete: false,
            invokes: &[],
            field_uses: &[],
            member_refs: &[],
        };
        assert_eq!(
            prove_same_class_field_binding(
                &class.this_class.raw().0,
                &class.fields,
                0,
                &facts,
                &mut budget,
            )
            .expect("an incomplete inventory is a refusal, not an error"),
            SameClassBinding::Unproved,
        );
    }

    #[test]
    fn generic_constructor_field_write_requires_one_unique_instance_field_by_name() {
        fn member(name: &[u8], descriptor: &[u8], access_flags: u16) -> MemberHeader {
            let jvm_string = |raw: &[u8]| {
                serde_json::from_value::<jarde_reader::model::JvmString>(serde_json::json!({
                    "raw": raw,
                    "utf16": raw.iter().map(|byte| *byte as u16).collect::<Vec<_>>(),
                    "escaped": String::from_utf8_lossy(raw),
                }))
                .unwrap()
            };
            MemberHeader {
                name: jvm_string(name),
                descriptor: jvm_string(descriptor),
                access_flags,
                attributes: Vec::new(),
            }
        }

        let write = GenericConstructorFieldWrite {
            bci: 9,
            owner: JvmBytes::from_unicode("Hold"),
            name: JvmBytes::from_unicode("value"),
            descriptor: JvmBytes::from_unicode("Ljava/lang/Object;"),
            parameter_slot: 1,
        };
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        assert!(
            jarde_java::generic_source_proof::generic_constructor_field_matches_physical_field(
                &write,
                b"Hold",
                &[member(b"value", b"Ljava/lang/Object;", 0)],
                &mut budget,
            )
            .expect("one matching instance field is a proof")
        );

        for fields in [
            vec![member(b"value", b"Ljava/lang/String;", 0)],
            vec![
                member(b"value", b"Ljava/lang/Object;", 0),
                member(b"value", b"Ljava/lang/String;", 0),
            ],
            vec![member(b"value", b"Ljava/lang/Object;", ACC_STATIC)],
        ] {
            let mut budget = Budget::new(unlimited_annotation_test_limits());
            assert!(
                !jarde_java::generic_source_proof::generic_constructor_field_matches_physical_field(
                    &write,
                    b"Hold",
                    &fields,
                    &mut budget,
                )
                .expect("a mismatched physical field is a refusal")
            );
        }
    }

    #[test]
    fn class_scope_constructor_gate_matches_exact_same_class_target_and_receiver() {
        fn member(name: &[u8], descriptor: &[u8]) -> MemberHeader {
            let jvm_string = |raw: &[u8]| {
                serde_json::from_value::<jarde_reader::model::JvmString>(serde_json::json!({
                    "raw": raw,
                    "utf16": raw.iter().map(|byte| *byte as u16).collect::<Vec<_>>(),
                    "escaped": String::from_utf8_lossy(raw),
                }))
                .unwrap()
            };
            MemberHeader {
                name: jvm_string(name),
                descriptor: jvm_string(descriptor),
                access_flags: 0,
                attributes: Vec::new(),
            }
        }
        fn invoke(
            owner: &[u8],
            descriptor: &[u8],
            receiver: Option<SameClassConstructorReceiver>,
            bci: u32,
        ) -> SameClassInvokeUse {
            let physical_owner = crate::PhysicalDefinitionId {
                location: jarde_reader::model::PhysicalClassLocation::StandaloneRoot {
                    snapshot: jarde_reader::model::SnapshotId("generic-call-invoke".to_owned()),
                },
                class_bytes: jarde_reader::model::ClassBytesId {
                    digest: jarde_reader::model::Digest("generic-call-invoke".to_owned()),
                    length: 0,
                },
                variant: jarde_reader::model::PhysicalVariant::Base,
            };
            SameClassInvokeUse {
                caller: "caller()V".to_owned(),
                physical_caller: PhysicalMethodId {
                    owner: physical_owner,
                    name: crate::JvmBytes(b"caller".to_vec()),
                    descriptor: crate::JvmBytes(b"()V".to_vec()),
                },
                bci,
                opcode: 0xb7,
                owner: owner.to_vec(),
                name: b"<init>".to_vec(),
                descriptor: descriptor.to_vec(),
                argument_values: Vec::new(),
                receiver_value: None,
                result_value: None,
                argument_value_facts: Vec::new(),
                receiver_value_fact: None,
                result_value_fact: None,
                ssa_operands_complete: false,
                constructor_receiver: receiver,
            }
        }

        let target = member(b"<init>", b"(Ljava/lang/Object;)V");
        for (receiver, expected) in [
            (
                Some(SameClassConstructorReceiver::UninitializedThis),
                Some(11),
            ),
            (Some(SameClassConstructorReceiver::Unknown), Some(11)),
            (None, Some(11)),
            (Some(SameClassConstructorReceiver::NewObject), None),
        ] {
            let invokes = [invoke(b"pkg/Hold", b"(Ljava/lang/Object;)V", receiver, 11)];
            let facts = SameClassUseFacts {
                complete: true,
                invokes: &invokes,
                field_uses: &[],
                member_refs: &[],
            };
            let mut budget = Budget::new(unlimited_annotation_test_limits());
            assert_eq!(
                unproved_same_class_this_delegate_bci(b"pkg/Hold", &target, &facts, &mut budget,)
                    .expect("bounded invocation scan succeeds"),
                expected,
            );
        }

        let invokes = [
            invoke(
                b"pkg/Other",
                b"(Ljava/lang/Object;)V",
                Some(SameClassConstructorReceiver::UninitializedThis),
                21,
            ),
            invoke(
                b"pkg/Hold",
                b"(Ljava/lang/String;)V",
                Some(SameClassConstructorReceiver::UninitializedThis),
                22,
            ),
        ];
        let facts = SameClassUseFacts {
            complete: true,
            invokes: &invokes,
            field_uses: &[],
            member_refs: &[],
        };
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        assert_eq!(
            unproved_same_class_this_delegate_bci(b"pkg/Hold", &target, &facts, &mut budget,)
                .expect("bounded invocation scan succeeds"),
            None,
        );
    }

    #[test]
    fn generic_call_components_are_callee_first_and_cycles_are_rejected() {
        let candidates = [0, 1, 2, 3];
        let dependencies = [
            GenericCallDependency {
                caller: 0,
                callee: 1,
            },
            GenericCallDependency {
                caller: 1,
                callee: 2,
            },
            GenericCallDependency {
                caller: 2,
                callee: 1,
            },
        ];
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let components = generic_call_components(&candidates, &dependencies, &mut budget)
            .expect("finite graph planning is bounded");
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].members, [0, 1, 2]);
        assert_eq!(components[0].order, None);
        assert_eq!(components[1].members, [3]);
        assert_eq!(components[1].order.as_deref(), Some([3].as_slice()));

        let dependencies = [
            GenericCallDependency {
                caller: 0,
                callee: 1,
            },
            GenericCallDependency {
                caller: 1,
                callee: 2,
            },
        ];
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let components = generic_call_components(&candidates, &dependencies, &mut budget)
            .expect("finite graph planning is bounded");
        assert_eq!(components[0].order.as_deref(), Some([2, 1, 0].as_slice()));
    }

    #[test]
    fn generic_call_components_meter_dense_fan_in_and_honor_cancellation() {
        let callee_count = 8usize;
        let caller_count = 8usize;
        let candidates = (0..callee_count + caller_count).collect::<Vec<_>>();
        let dependencies = (callee_count..callee_count + caller_count)
            .flat_map(|caller| {
                (0..callee_count).map(move |callee| GenericCallDependency { caller, callee })
            })
            .collect::<Vec<_>>();

        let mut constrained_limits = unlimited_annotation_test_limits();
        // The input-manifest precharge alone cannot also fund the graph's processing passes.
        constrained_limits.analysis_steps =
            u64::try_from(candidates.len().saturating_add(dependencies.len())).unwrap_or(u64::MAX);
        let mut constrained_budget = Budget::new(constrained_limits);
        assert!(matches!(
            generic_call_components(&candidates, &dependencies, &mut constrained_budget),
            Err(Error::BudgetExceeded {
                dimension: crate::BudgetDimension::AnalysisSteps,
                ..
            })
        ));

        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let components = generic_call_components(&candidates, &dependencies, &mut budget)
            .expect("a fully budgeted dense fan-in graph is ordered");
        assert_eq!(components.len(), 1);
        let expected_order = (0..callee_count + caller_count).collect::<Vec<_>>();
        assert_eq!(
            components[0].order.as_deref(),
            Some(expected_order.as_slice())
        );

        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut cancelled_budget =
            Budget::with_cancellation_token(unlimited_annotation_test_limits(), cancellation);
        assert!(matches!(
            generic_call_components(&candidates, &dependencies, &mut cancelled_budget),
            Err(Error::Cancelled { .. })
        ));
    }

    #[test]
    fn generic_call_substitution_preserves_binders_bounds_and_arrays() {
        use jarde_reader::signature::{ClassType, ClassTypeSegment};

        let class_parameters = vec![TypeParameter {
            name: b"T".to_vec(),
            class_bound: Some(SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/lang/Number".to_vec(),
                    arguments: Vec::new(),
                }],
            })),
            interface_bounds: vec![SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/lang/Runnable".to_vec(),
                    arguments: Vec::new(),
                }],
            })],
        }];
        let class_scope = vec![TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Number;".to_vec(),
        }];
        let owner = PhysicalDefinitionId {
            location: jarde_reader::model::PhysicalClassLocation::StandaloneRoot {
                snapshot: jarde_reader::model::SnapshotId("generic-call-binder".to_owned()),
            },
            class_bytes: jarde_reader::model::ClassBytesId {
                digest: jarde_reader::model::Digest("generic-call-binder".to_owned()),
                length: 0,
            },
            variant: jarde_reader::model::PhysicalVariant::Base,
        };
        let identity = PhysicalMethodId {
            owner: owner.clone(),
            name: crate::JvmBytes(b"identity".to_vec()),
            descriptor: crate::JvmBytes(b"(Ljava/lang/Number;)Ljava/lang/Number;".to_vec()),
        };
        let caller = PhysicalMethodId {
            owner,
            name: crate::JvmBytes(b"relay".to_vec()),
            descriptor: crate::JvmBytes(b"(Ljava/lang/Number;)Ljava/lang/Number;".to_vec()),
        };
        let bound = |name: &[u8]| TypeParameter {
            name: name.to_vec(),
            class_bound: Some(SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/lang/Number".to_vec(),
                    arguments: Vec::new(),
                }],
            })),
            interface_bounds: vec![SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/lang/Runnable".to_vec(),
                    arguments: Vec::new(),
                }],
            })],
        };
        let callee = SameClassGenericMethodContract {
            method: identity.clone(),
            method_parameters: vec![bound(b"U")],
            parameters: vec![SignatureType::TypeVariable(b"U".to_vec())],
            result: Some(SignatureType::TypeVariable(b"U".to_vec())),
        };
        let actual = SameClassGenericArgumentSource {
            caller: caller.clone(),
            origin: SameClassGenericArgumentOrigin::Formal { slot: 1 },
            source_type: SignatureType::TypeVariable(b"T".to_vec()),
            caller_method_parameters: Vec::new(),
        };
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        let substitution = prove_same_class_generic_substitution(
            &class_scope,
            &class_parameters,
            &callee,
            &[actual],
            &mut budget,
        )
        .expect("the direct binder proof is bounded")
        .expect("the matching class intersection bounds satisfy U");
        assert_eq!(
            substitution.method_arguments,
            [SignatureType::TypeVariable(b"T".to_vec())]
        );

        let shadowed = SameClassGenericArgumentSource {
            caller: caller.clone(),
            origin: SameClassGenericArgumentOrigin::Formal { slot: 1 },
            source_type: SignatureType::TypeVariable(b"T".to_vec()),
            caller_method_parameters: vec![bound(b"T")],
        };
        let class_target = SameClassGenericMethodContract {
            method: identity.clone(),
            method_parameters: Vec::new(),
            parameters: vec![SignatureType::TypeVariable(b"T".to_vec())],
            result: Some(SignatureType::TypeVariable(b"T".to_vec())),
        };
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        assert!(
            prove_same_class_generic_substitution(
                &class_scope,
                &class_parameters,
                &class_target,
                &[shadowed],
                &mut budget,
            )
            .expect("the binder mismatch is an ordinary refusal")
            .is_none()
        );

        let array_contract = SameClassGenericMethodContract {
            method: identity,
            method_parameters: Vec::new(),
            parameters: vec![SignatureType::Array(Box::new(SignatureType::TypeVariable(
                b"T".to_vec(),
            )))],
            result: Some(SignatureType::Array(Box::new(SignatureType::TypeVariable(
                b"T".to_vec(),
            )))),
        };
        let array_source = SameClassGenericArgumentSource {
            caller,
            origin: SameClassGenericArgumentOrigin::Formal { slot: 1 },
            source_type: SignatureType::Array(Box::new(SignatureType::TypeVariable(b"T".to_vec()))),
            caller_method_parameters: Vec::new(),
        };
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        assert!(
            prove_same_class_generic_substitution(
                &class_scope,
                &class_parameters,
                &array_contract,
                &[array_source],
                &mut budget,
            )
            .expect("array binder proof is bounded")
            .is_some()
        );
    }

    #[test]
    fn complete_body_primitive_result_requires_exact_signature_and_descriptor_type() {
        use jarde_java::ast::Type;
        let parsed = SignatureType::Base(b'Z');
        assert!(ordinary_primitive_result_matches(
            &Type::Boolean,
            Some(&parsed),
            Some("boolean"),
            Some("boolean")
        ));
        assert!(!ordinary_primitive_result_matches(
            &Type::Int,
            Some(&parsed),
            Some("int"),
            Some("int")
        ));
        assert!(!ordinary_primitive_result_matches(
            &Type::Boolean,
            Some(&parsed),
            Some("boolean"),
            Some("int")
        ));
        assert!(!ordinary_primitive_result_matches(
            &Type::Boolean,
            Some(&SignatureType::Base(b'I')),
            Some("boolean"),
            Some("boolean")
        ));
        assert!(!ordinary_primitive_result_matches(
            &Type::Boolean,
            None,
            Some("boolean"),
            Some("boolean")
        ));
    }
}

#[cfg(test)]
mod same_class_overload_proof_tests {
    use super::*;
    use jarde_java::facts::{CallTarget, InvokeKind};
    use jarde_reader::model::{
        ClassBytesId, Digest, PhysicalClassLocation, PhysicalVariant, SnapshotId,
    };
    use jarde_reader::signature::{ClassType, ClassTypeSegment};

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

    fn owner() -> PhysicalDefinitionId {
        PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: SnapshotId("gc07-overload-proof".to_owned()),
            },
            class_bytes: ClassBytesId {
                digest: Digest("gc07-overload-proof".to_owned()),
                length: 0,
            },
            variant: PhysicalVariant::Base,
        }
    }

    fn method_header(name: &[u8], descriptor: &[u8], access_flags: u16) -> MemberHeader {
        let jvm_string = |raw: &[u8]| {
            serde_json::from_value::<jarde_reader::model::JvmString>(serde_json::json!({
                "raw": raw,
                "utf16": raw.iter().map(|byte| *byte as u16).collect::<Vec<_>>(),
                "escaped": String::from_utf8_lossy(raw),
            }))
            .expect("test JVM string is valid")
        };
        MemberHeader {
            name: jvm_string(name),
            descriptor: jvm_string(descriptor),
            access_flags,
            attributes: Vec::new(),
        }
    }

    fn invoke_use(
        caller: &PhysicalMethodId,
        owner: &[u8],
        name: &[u8],
        descriptor: &[u8],
    ) -> SameClassInvokeUse {
        SameClassInvokeUse {
            caller: "relay(Ljava/lang/Number;)V".to_owned(),
            physical_caller: caller.clone(),
            bci: 7,
            opcode: 0xb6,
            owner: owner.to_vec(),
            name: name.to_vec(),
            descriptor: descriptor.to_vec(),
            argument_values: Vec::new(),
            receiver_value: None,
            result_value: None,
            argument_value_facts: Vec::new(),
            receiver_value_fact: None,
            result_value_fact: None,
            ssa_operands_complete: false,
            constructor_receiver: None,
        }
    }

    #[test]
    fn same_class_binding_defers_only_closed_same_arity_overload_competition() {
        let owner_id = owner();
        let caller = caller(&owner_id);
        let class = b"p/Bound".to_vec();
        let methods = [
            method_header(b"pick", b"(Ljava/lang/Object;)V", ACC_PUBLIC),
            method_header(b"pick", b"(Ljava/lang/String;)V", ACC_PUBLIC),
        ];
        let invoke = invoke_use(&caller, &class, b"pick", b"(Ljava/lang/Object;)V");
        let complete = SameClassUseFacts {
            complete: true,
            invokes: std::slice::from_ref(&invoke),
            field_uses: &[],
            member_refs: &[],
        };
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        assert_eq!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[],
                &methods,
                0,
                &complete,
                &mut budget,
            )
            .expect("closed overload competition is a deferred proof"),
            SameClassBinding::OverloadPending,
        );

        let no_incoming = SameClassUseFacts {
            complete: true,
            invokes: &[],
            field_uses: &[],
            member_refs: &[],
        };
        assert_eq!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[],
                &methods,
                0,
                &no_incoming,
                &mut budget,
            )
            .expect("unused overload has no call-site proof"),
            SameClassBinding::Unproved,
        );

        let incomplete = SameClassUseFacts {
            complete: false,
            invokes: std::slice::from_ref(&invoke),
            field_uses: &[],
            member_refs: &[],
        };
        assert_eq!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[],
                &methods,
                0,
                &incomplete,
                &mut budget,
            )
            .expect("incomplete inventory remains a refusal"),
            SameClassBinding::Unproved,
        );

        let unresolved = invoke_use(&caller, &class, b"pick", b"(Ljava/lang/Number;)V");
        let unknown_use = SameClassUseFacts {
            complete: true,
            invokes: std::slice::from_ref(&unresolved),
            field_uses: &[],
            member_refs: &[],
        };
        assert_eq!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[],
                &methods,
                0,
                &unknown_use,
                &mut budget,
            )
            .expect("unexplained sibling target remains a refusal"),
            SameClassBinding::Unproved,
        );
    }

    #[test]
    fn same_class_binding_defers_unreferenced_generic_sibling_for_real_same_arity_target() {
        let owner_id = owner();
        let caller = caller(&owner_id);
        let class = b"p/BoundOverload".to_vec();
        let methods = [
            method_header(b"pick", b"(Ljava/lang/Number;)V", ACC_PUBLIC),
            method_header(b"pick", b"(Ljava/lang/Comparable;)V", ACC_PUBLIC),
        ];
        // The real bytecode target is pick(Number). The generic sibling erases to Comparable
        // and has no direct invoke, but it still participates in source overload resolution.
        let invoked = invoke_use(&caller, &class, b"pick", b"(Ljava/lang/Number;)V");
        let facts = SameClassUseFacts {
            complete: true,
            invokes: std::slice::from_ref(&invoked),
            field_uses: &[],
            member_refs: &[],
        };
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        assert_eq!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[],
                &methods,
                1,
                &facts,
                &mut budget,
            )
            .expect("closed competing header is deferred for the overload proof"),
            SameClassBinding::OverloadPending,
        );
    }

    #[test]
    fn same_class_binding_keeps_unclosed_overload_inventories_unproved() {
        let owner_id = owner();
        let caller = caller(&owner_id);
        let class = b"p/BoundOverload".to_vec();
        let methods = [
            method_header(b"pick", b"(Ljava/lang/Number;)V", ACC_PUBLIC),
            method_header(b"pick", b"(Ljava/lang/Comparable;)V", ACC_PUBLIC),
        ];
        let mut budget = Budget::new(unlimited_annotation_test_limits());

        let no_same_name_invoke = SameClassUseFacts {
            complete: true,
            invokes: &[],
            field_uses: &[],
            member_refs: &[],
        };
        assert_eq!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[],
                &methods,
                1,
                &no_same_name_invoke,
                &mut budget,
            )
            .expect("an unused overload set is refused"),
            SameClassBinding::Unproved,
        );

        let invoked = invoke_use(&caller, &class, b"pick", b"(Ljava/lang/Number;)V");
        let closed_calls = SameClassUseFacts {
            complete: true,
            invokes: std::slice::from_ref(&invoked),
            field_uses: &[],
            member_refs: &[],
        };
        assert_eq!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Number"),
                &[],
                &methods,
                1,
                &closed_calls,
                &mut budget,
            )
            .expect("a non-Object parent leaves external overloads open"),
            SameClassBinding::Unproved,
        );
        assert_eq!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[b"java/io/Serializable".to_vec()],
                &methods,
                1,
                &closed_calls,
                &mut budget,
            )
            .expect("implemented interfaces leave external overloads open"),
            SameClassBinding::Unproved,
        );
        let member_ref = SameClassMemberRef {
            owner: class.clone(),
            name: b"pick".to_vec(),
            descriptor: b"(Ljava/lang/Number;)V".to_vec(),
            method: true,
        };
        let non_body_reference = SameClassUseFacts {
            complete: true,
            invokes: std::slice::from_ref(&invoked),
            field_uses: &[],
            member_refs: std::slice::from_ref(&member_ref),
        };
        assert_eq!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[],
                &methods,
                1,
                &non_body_reference,
                &mut budget,
            )
            .expect("non-body method references are not source overload proof"),
            SameClassBinding::Unproved,
        );

        for (owner, descriptor, opcode) in [
            (&class[..], &b"(Ljava/io/Serializable;)V"[..], 0xb6),
            (&b"p/Foreign"[..], &b"(Ljava/lang/Number;)V"[..], 0xb6),
            (&class[..], &b"(Ljava/lang/Number;)V"[..], 0xb8),
        ] {
            let mut invoke = invoke_use(&caller, owner, b"pick", descriptor);
            invoke.opcode = opcode;
            let facts = SameClassUseFacts {
                complete: true,
                invokes: std::slice::from_ref(&invoke),
                field_uses: &[],
                member_refs: &[],
            };
            assert_eq!(
                prove_same_class_method_binding(
                    &class,
                    Some(b"java/lang/Object"),
                    &[],
                    &methods,
                    1,
                    &facts,
                    &mut budget,
                )
                .expect("unexplained, foreign, or opcode-inconsistent use is refused"),
                SameClassBinding::Unproved,
            );
        }

        let duplicate = [
            method_header(b"pick", b"(Ljava/lang/Number;)V", ACC_PUBLIC),
            method_header(b"pick", b"(Ljava/lang/Comparable;)V", ACC_PUBLIC),
            method_header(b"pick", b"(Ljava/lang/Comparable;)V", ACC_PUBLIC),
        ];
        let invoked = invoke_use(&caller, &class, b"pick", b"(Ljava/lang/Number;)V");
        let facts = SameClassUseFacts {
            complete: true,
            invokes: std::slice::from_ref(&invoked),
            field_uses: &[],
            member_refs: &[],
        };
        assert_eq!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[],
                &duplicate,
                1,
                &facts,
                &mut budget,
            )
            .expect("ambiguous physical declaration is refused"),
            SameClassBinding::Unproved,
        );

        for bad_flags in [ACC_PUBLIC | ACC_VARARGS, ACC_PUBLIC | 0x0040] {
            let bad = [
                method_header(b"pick", b"(Ljava/lang/Number;)V", ACC_PUBLIC),
                method_header(b"pick", b"(Ljava/lang/Comparable;)V", bad_flags),
            ];
            assert_eq!(
                prove_same_class_method_binding(
                    &class,
                    Some(b"java/lang/Object"),
                    &[],
                    &bad,
                    1,
                    &facts,
                    &mut budget,
                )
                .expect("varargs and bridge competitors are refused"),
                SameClassBinding::Unproved,
            );
        }
    }

    #[test]
    fn same_class_binding_propagates_budget_and_cancellation() {
        let owner_id = owner();
        let caller = caller(&owner_id);
        let class = b"p/BoundOverload".to_vec();
        let methods = [
            method_header(b"pick", b"(Ljava/lang/Number;)V", ACC_PUBLIC),
            method_header(b"pick", b"(Ljava/lang/Comparable;)V", ACC_PUBLIC),
        ];
        let invoked = invoke_use(&caller, &class, b"pick", b"(Ljava/lang/Number;)V");
        let facts = SameClassUseFacts {
            complete: true,
            invokes: std::slice::from_ref(&invoked),
            field_uses: &[],
            member_refs: &[],
        };
        let mut limits = unlimited_annotation_test_limits();
        limits.analysis_steps = 0;
        let mut exhausted = Budget::new(limits);
        assert!(matches!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[],
                &methods,
                1,
                &facts,
                &mut exhausted,
            ),
            Err(Error::BudgetExceeded {
                dimension: crate::BudgetDimension::AnalysisSteps,
                ..
            })
        ));

        let token = crate::CancellationToken::new();
        token.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(unlimited_annotation_test_limits(), token);
        assert!(matches!(
            prove_same_class_method_binding(
                &class,
                Some(b"java/lang/Object"),
                &[],
                &methods,
                1,
                &facts,
                &mut cancelled,
            ),
            Err(Error::Cancelled { .. })
        ));
    }

    fn class_type(name: &[u8]) -> SignatureType {
        SignatureType::Class(ClassType {
            segments: vec![ClassTypeSegment {
                binary_name: name.to_vec(),
                arguments: Vec::new(),
            }],
        })
    }

    fn number_comparable_parameter(name: &[u8]) -> TypeParameter {
        TypeParameter {
            name: name.to_vec(),
            class_bound: Some(class_type(b"java/lang/Number")),
            interface_bounds: vec![SignatureType::Class(ClassType {
                segments: vec![ClassTypeSegment {
                    binary_name: b"java/lang/Comparable".to_vec(),
                    arguments: vec![TypeArgument::Exact(SignatureType::TypeVariable(
                        name.to_vec(),
                    ))],
                }],
            })],
        }
    }

    fn number_parameter(name: &[u8]) -> TypeParameter {
        TypeParameter {
            name: name.to_vec(),
            class_bound: Some(class_type(b"java/lang/Number")),
            interface_bounds: Vec::new(),
        }
    }

    fn caller(owner: &PhysicalDefinitionId) -> PhysicalMethodId {
        PhysicalMethodId {
            owner: owner.clone(),
            name: crate::JvmBytes(b"relay".to_vec()),
            descriptor: crate::JvmBytes(b"(Ljava/lang/Number;)V".to_vec()),
        }
    }

    fn candidate(
        owner: &PhysicalDefinitionId,
        name: &[u8],
        descriptor: &[u8],
        parameters: Vec<SignatureType>,
        method_parameters: Vec<TypeParameter>,
    ) -> SameClassOverloadCandidate {
        SameClassOverloadCandidate {
            method: PhysicalMethodId {
                owner: owner.clone(),
                name: crate::JvmBytes(name.to_vec()),
                descriptor: crate::JvmBytes(descriptor.to_vec()),
            },
            owner: b"p/Bound".to_vec(),
            descriptor: descriptor.to_vec(),
            access_flags: ACC_PUBLIC,
            parameters,
            method_parameters,
            source_writable: true,
        }
    }

    fn source(
        caller: &PhysicalMethodId,
        variable: &[u8],
        binders: Vec<TypeParameter>,
    ) -> SameClassOverloadActual {
        SameClassOverloadActual {
            source: Some(SameClassGenericArgumentSource {
                caller: caller.clone(),
                origin: SameClassGenericArgumentOrigin::Formal { slot: 1 },
                source_type: SignatureType::TypeVariable(variable.to_vec()),
                caller_method_parameters: binders,
            }),
            null_literal: false,
        }
    }

    fn primitive_source(
        caller: &PhysicalMethodId,
        slot: u16,
        descriptor: u8,
    ) -> SameClassOverloadActual {
        SameClassOverloadActual {
            source: Some(SameClassGenericArgumentSource {
                caller: caller.clone(),
                origin: SameClassGenericArgumentOrigin::Formal { slot },
                source_type: SignatureType::Base(descriptor),
                caller_method_parameters: Vec::new(),
            }),
            null_literal: false,
        }
    }

    fn invoke(
        caller: &PhysicalMethodId,
        class_parameters: &[TypeParameter],
        class_scope: &[TypeParameterErasure],
        target_descriptor: &str,
        actuals: &[SameClassOverloadActual],
        overloads: &[SameClassOverloadCandidate],
        complete: bool,
    ) -> Option<Vec<jarde_java::report::ClassSourceInvokeArgumentCast>> {
        let target = CallTarget::new(
            InvokeKind::Virtual,
            "p/Bound",
            "pick",
            target_descriptor,
            false,
        );
        prove_same_class_overload_argument_upcasts(
            b"p/Bound",
            Some(b"java/lang/Object"),
            &[],
            class_scope,
            class_parameters,
            caller,
            &target,
            17,
            complete,
            actuals,
            overloads,
            &mut Budget::new(unlimited_annotation_test_limits()),
        )
        .expect("the overload proof is bounded")
    }

    #[test]
    fn bound_overload_casts_only_to_the_physical_targets_explicit_number_bound() {
        let owner = owner();
        let caller = caller(&owner);
        let parameter = number_comparable_parameter(b"T");
        let scope = vec![TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Number;".to_vec(),
        }];
        let target = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Number;)V",
            vec![class_type(b"java/lang/Number")],
            Vec::new(),
        );
        let comparable = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Comparable;)V",
            vec![class_type(b"java/lang/Comparable")],
            Vec::new(),
        );
        let object = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Object;)V",
            vec![class_type(b"java/lang/Object")],
            Vec::new(),
        );
        let actuals = [source(&caller, b"T", Vec::new())];
        let overloads = [target, comparable, object];
        let edits = invoke(
            &caller,
            &[parameter],
            &scope,
            "(Ljava/lang/Number;)V",
            &actuals,
            &overloads,
            true,
        )
        .expect("the complete closed overload set is provable");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].call_bci, 17);
        assert_eq!(edits[0].argument_index, 0);
        assert_eq!(edits[0].opcode, 0xb6);
        assert_eq!(
            edits[0].ty,
            jarde_java::ast::Type::Reference("java.lang.Number".to_owned())
        );
    }

    #[test]
    fn wide_second_argument_keeps_its_descriptor_position_when_casting_first() {
        let owner = owner();
        let caller = PhysicalMethodId {
            owner: owner.clone(),
            name: crate::JvmBytes(b"relay".to_vec()),
            descriptor: crate::JvmBytes(b"(Ljava/lang/Number;J)V".to_vec()),
        };
        let parameter = number_comparable_parameter(b"T");
        let scope = vec![TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Number;".to_vec(),
        }];
        let number_long = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Number;J)V",
            vec![class_type(b"java/lang/Number"), SignatureType::Base(b'J')],
            Vec::new(),
        );
        let comparable_long = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Comparable;J)V",
            vec![
                class_type(b"java/lang/Comparable"),
                SignatureType::Base(b'J'),
            ],
            Vec::new(),
        );
        let object_long = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Object;J)V",
            vec![class_type(b"java/lang/Object"), SignatureType::Base(b'J')],
            Vec::new(),
        );
        let actuals = [
            source(&caller, b"T", Vec::new()),
            primitive_source(&caller, 2, b'J'),
        ];
        let edits = invoke(
            &caller,
            &[parameter],
            &scope,
            "(Ljava/lang/Number;J)V",
            &actuals,
            &[number_long, comparable_long, object_long],
            true,
        )
        .expect("both descriptor parameters close the same physical call");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].argument_index, 0);
    }

    #[test]
    fn plain_upper_bound_is_already_unique_and_same_name_string_is_closed_negative() {
        let owner = owner();
        let caller = caller(&owner);
        let parameter = number_parameter(b"T");
        let scope = vec![TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Number;".to_vec(),
        }];
        let number = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Number;)V",
            vec![class_type(b"java/lang/Number")],
            Vec::new(),
        );
        let object = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Object;)V",
            vec![class_type(b"java/lang/Object")],
            Vec::new(),
        );
        let actuals = [source(&caller, b"T", Vec::new())];
        assert_eq!(
            invoke(
                &caller,
                &[parameter.clone()],
                &scope,
                "(Ljava/lang/Number;)V",
                &actuals,
                &[number.clone(), object.clone()],
                true,
            ),
            Some(Vec::new())
        );

        let string = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/String;)V",
            vec![class_type(b"java/lang/String")],
            Vec::new(),
        );
        assert_eq!(
            invoke(
                &caller,
                &[parameter],
                &scope,
                "(Ljava/lang/Number;)V",
                &actuals,
                &[number, string, object],
                true,
            ),
            Some(Vec::new())
        );
    }

    #[test]
    fn same_name_overload_uses_unbounded_typevar_and_does_not_narrow_to_string() {
        let owner = owner();
        let caller = caller(&owner);
        let parameter = TypeParameter {
            name: b"T".to_vec(),
            class_bound: Some(class_type(b"java/lang/Object")),
            interface_bounds: Vec::new(),
        };
        let scope = vec![TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Object;".to_vec(),
        }];
        let object = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Object;)V",
            vec![class_type(b"java/lang/Object")],
            Vec::new(),
        );
        let string = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/String;)V",
            vec![class_type(b"java/lang/String")],
            Vec::new(),
        );
        assert_eq!(
            invoke(
                &caller,
                &[parameter],
                &scope,
                "(Ljava/lang/Object;)V",
                &[source(&caller, b"T", Vec::new())],
                &[object, string],
                true,
            ),
            Some(Vec::new()),
            "an unbounded T widens to Object and is not assignable to String"
        );
    }

    #[test]
    fn unknown_typevar_bound_is_not_negative_evidence_for_an_overload() {
        let owner = owner();
        let caller = caller(&owner);
        let parameter = TypeParameter {
            name: b"T".to_vec(),
            class_bound: Some(class_type(b"foreign/Unknown")),
            interface_bounds: Vec::new(),
        };
        let scope = vec![TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Lforeign/Unknown;".to_vec(),
        }];
        let object = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Object;)V",
            vec![class_type(b"java/lang/Object")],
            Vec::new(),
        );
        let other = candidate(
            &owner,
            b"pick",
            b"(Lforeign/MaybeNumber;)V",
            vec![class_type(b"foreign/MaybeNumber")],
            Vec::new(),
        );
        assert_eq!(
            invoke(
                &caller,
                &[parameter],
                &scope,
                "(Ljava/lang/Object;)V",
                &[source(&caller, b"T", Vec::new())],
                &[object, other],
                true,
            ),
            None,
            "an unresolved bound/candidate relation cannot be called inapplicable"
        );
    }

    #[test]
    fn distinct_type_variables_are_not_assumed_incompatible() {
        let method_parameters = [TypeParameter {
            name: b"U".to_vec(),
            class_bound: Some(SignatureType::TypeVariable(b"T".to_vec())),
            interface_bounds: Vec::new(),
        }];
        let class_parameters = [TypeParameter {
            name: b"T".to_vec(),
            class_bound: Some(class_type(b"java/lang/Object")),
            interface_bounds: Vec::new(),
        }];
        let class_scope = [TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Object;".to_vec(),
        }];
        let mut budget = Budget::new(unlimited_annotation_test_limits());
        assert_eq!(
            overload_source_type_assignable(
                &SignatureType::TypeVariable(b"U".to_vec()),
                &SignatureType::TypeVariable(b"T".to_vec()),
                &method_parameters,
                &class_scope,
                &class_parameters,
                &mut budget,
                0,
            )
            .expect("bounded type-variable comparison is charged"),
            None,
            "U extends T is related by its bound and cannot be called inapplicable"
        );
    }

    #[test]
    fn overload_proof_propagates_budget_exhaustion_and_cancellation() {
        let owner = owner();
        let caller = caller(&owner);
        let target = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Number;)V",
            vec![class_type(b"java/lang/Number")],
            Vec::new(),
        );
        let actuals = [source(&caller, b"T", Vec::new())];
        let parameters = [number_parameter(b"T")];
        let scope = [TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Number;".to_vec(),
        }];
        let mut limits = unlimited_annotation_test_limits();
        limits.analysis_steps = 0;
        let mut exhausted = Budget::new(limits);
        assert!(matches!(
            prove_same_class_overload_argument_upcasts(
                b"p/Bound",
                Some(b"java/lang/Object"),
                &[],
                &scope,
                &parameters,
                &caller,
                &CallTarget::new(
                    InvokeKind::Virtual,
                    "p/Bound",
                    "pick",
                    "(Ljava/lang/Number;)V",
                    false,
                ),
                17,
                true,
                &actuals,
                &[target.clone()],
                &mut exhausted,
            ),
            Err(Error::BudgetExceeded { .. })
        ));

        let token = jarde_reader::budget::CancellationToken::new();
        token.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(unlimited_annotation_test_limits(), token);
        assert!(matches!(
            prove_same_class_overload_argument_upcasts(
                b"p/Bound",
                Some(b"java/lang/Object"),
                &[],
                &scope,
                &parameters,
                &caller,
                &CallTarget::new(
                    InvokeKind::Virtual,
                    "p/Bound",
                    "pick",
                    "(Ljava/lang/Number;)V",
                    false,
                ),
                17,
                true,
                &actuals,
                &[target],
                &mut cancelled,
            ),
            Err(Error::Cancelled { .. })
        ));
    }

    #[test]
    fn callee_method_binder_is_substituted_and_unknown_user_hierarchy_refuses() {
        let owner = owner();
        let caller = caller(&owner);
        let class_parameter = number_parameter(b"T");
        let scope = vec![TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Number;".to_vec(),
        }];
        let generic_target = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Number;)V",
            vec![SignatureType::TypeVariable(b"U".to_vec())],
            vec![number_parameter(b"U")],
        );
        let object = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Object;)V",
            vec![class_type(b"java/lang/Object")],
            Vec::new(),
        );
        let actuals = [source(&caller, b"T", Vec::new())];
        assert_eq!(
            invoke(
                &caller,
                &[class_parameter.clone()],
                &scope,
                "(Ljava/lang/Number;)V",
                &actuals,
                &[generic_target.clone(), object.clone()],
                true,
            ),
            Some(Vec::new())
        );

        let unknown = candidate(
            &owner,
            b"pick",
            b"(Lforeign/MaybeNumber;)V",
            vec![class_type(b"foreign/MaybeNumber")],
            Vec::new(),
        );
        assert_eq!(
            invoke(
                &caller,
                &[class_parameter],
                &scope,
                "(Ljava/lang/Number;)V",
                &actuals,
                &[generic_target, object, unknown],
                true,
            ),
            None,
            "unknown hierarchy facts cannot be treated as a negative applicability result"
        );
    }

    #[test]
    fn incomplete_or_foreign_physical_inventories_never_pin_a_target() {
        let owner = owner();
        let caller = caller(&owner);
        let parameter = number_parameter(b"T");
        let scope = vec![TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Ljava/lang/Number;".to_vec(),
        }];
        let target = candidate(
            &owner,
            b"pick",
            b"(Ljava/lang/Number;)V",
            vec![class_type(b"java/lang/Number")],
            Vec::new(),
        );
        let actuals = [source(&caller, b"T", Vec::new())];
        assert_eq!(
            invoke(
                &caller,
                &[parameter.clone()],
                &scope,
                "(Ljava/lang/Number;)V",
                &actuals,
                &[target.clone()],
                false,
            ),
            None
        );

        let mut foreign = target;
        foreign.owner = b"q/Other".to_vec();
        assert_eq!(
            invoke(
                &caller,
                &[parameter],
                &scope,
                "(Ljava/lang/Number;)V",
                &actuals,
                &[foreign],
                true,
            ),
            None
        );

        let object_named_target = CallTarget::new(
            InvokeKind::Virtual,
            "p/Bound",
            "equals",
            "(Ljava/lang/Number;)V",
            false,
        );
        assert!(
            prove_same_class_overload_argument_upcasts(
                b"p/Bound",
                Some(b"java/lang/Object"),
                &[],
                &scope,
                &[number_parameter(b"T")],
                &caller,
                &object_named_target,
                17,
                true,
                &actuals,
                &[candidate(
                    &owner,
                    b"equals",
                    b"(Ljava/lang/Number;)V",
                    vec![class_type(b"java/lang/Number")],
                    Vec::new(),
                )],
                &mut Budget::new(unlimited_annotation_test_limits()),
            )
            .expect("Object method names are an ordinary refusal")
            .is_none(),
            "Object's inherited equals(Object) belongs to the source overload set"
        );
    }
}

#[cfg(test)]
use jarde_java::generic_source_proof::{
    SameClassOverloadActual, prove_same_class_overload_argument_upcasts,
};
