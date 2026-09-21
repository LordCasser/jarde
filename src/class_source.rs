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
//! It is not a compilable project, and it does not claim to be one. No `package` statement is
//! written (a package is a claim about the directory a name came from, and the class file states
//! none), no `import` is resolved or elided, no resource is parsed, no `throws` clause is written
//! (that would need the content of the `Exceptions` attribute, which the class read does not read),
//! no annotation is presented (attribute tables are read as shells), and no nesting relation is
//! inferred — `p.Outer$Inner` is spelled as the one name the class file states. Names are spelled
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
    AnalysisStage, Coverage, Diagnostic, ExecutionReport, JvmString, Limits, MemberTableStop,
    NoBodyKind, PhysicalDefinitionId, PhysicalView, TerminationReason, UsageSnapshot,
    budget_dimension_code,
};
use jarde_java::{
    LocalVariable, NameTable, RecoveryContent, RecoveryFacts, RecoveryReport, SlotEvidence,
    alias_for, comment_text, is_java_identifier, type_of_component,
};
use jarde_jvm::method_ir::parameter_positions;
use jarde_reader::classfile::{DescriptorComponent, DescriptorKind, descriptor_facts};
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
    /// `this_class` as source spells it: the internal name with `/` replaced by `.`, and no other
    /// change. A name the class file states is never a package plus a simple name here: the text
    /// writes this one spelling everywhere, so `p.Outer$Inner` keeps its `$`.
    pub name: String,
    /// The declaration line the assembled text opens with, without its opening brace (for example
    /// `public class p.Base extends java.lang.Object implements p.Marker`).
    ///
    /// Only the flags Java source spells are written (`public`/`protected`/`private`, `abstract`,
    /// `final`, `strictfp`); the raw `access_flags` stay in [`Self::item`], and the bits that are not
    /// source keywords (`ACC_SUPER`, `ACC_SYNTHETIC`, `ACC_MODULE`) are not invented into text.
    /// An interface's `extends` list is the interfaces the class file declares — its `super_class` is
    /// `java/lang/Object` by the format's own rule, so it is not written as a superclass.
    pub declaration: String,
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
    pub declaration: Option<String>,
    /// Every `// jarde:` marker the text carries for this field, in the order it writes them, each
    /// one line: the field's raw name is not a Java identifier, or its descriptor is not one (see
    /// [`Self::declaration`]). Empty when the spelling is faithful.
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
    /// The declaration the text writes for this member, without its opening brace and without a
    /// terminating `;`. `None` exactly when the member's raw descriptor is not one this presentation
    /// can read as a method descriptor (JVMS 4.3.3), in which case no declaration is written and
    /// [`Self::outcome`] is [`ClassSourceOutcome::Unspelled`].
    ///
    /// A member named `<init>` is spelled as the constructor it is (the class's own name, no return
    /// type), and `<clinit>` as Java's static initializer block (`static`), each with a marker that
    /// names the raw member it came from.
    pub declaration: Option<String>,
    /// The member's own text in the assembled source: its optional marker, its declaration, and
    /// either its block or its `;`. One level of indentation is already applied, so the text can be
    /// read on its own or found inside [`ClassSourceReport::text`] unchanged.
    ///
    /// A recovered body is placed by splitting the recovery artifact at its own block — the envelope
    /// comment lines the recovery layer wrote, then the statements it wrote — and indenting both one
    /// level deeper. That split is a reading of the fixpoint shape
    /// [`jarde_java::recover`] emits; a produced artifact that does not have it is quoted line by
    /// line as comments rather than placed as if it were statements.
    pub text: String,
    /// Every `// jarde:` marker the text carries for this member, in the order it writes them, each
    /// one line: a member with no `Code` attribute, a run that stopped or produced no statement, a
    /// member that could not be spelled, the alias of a raw name Java cannot spell, or a run whose
    /// analysis did not complete. Empty exactly when the member's body was recovered in full.
    pub markers: Vec<String>,
    /// What the member's own body produced: one recovery run's report, a declaration without a body,
    /// an unspellable member, or the member-level refusal of its run.
    pub outcome: ClassSourceOutcome,
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
/// `text` is the assembled source: the class declaration, then its fields in declaration order, then
/// its methods in the order the class file's own method table declares them, each member written
/// once and every member of the table either written or stated as not written. `fields` and
/// `methods` are those same members with the spelling each one contributed, and the planes are the
/// request's own: `limits` is the complete effective configuration, `usage` what that configuration
/// was charged, `coverage` the class and member tables this request read beside the members it
/// attempted a body for, and `execution` the merge of every stop of the request.
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
    /// The fields of the class read, in declaration order, each with its spelling.
    pub fields: Vec<ClassSourceField>,
    /// The methods of the class read, in the order its method table declares them, each with the
    /// result of its own run.
    pub methods: Vec<ClassSourceMethod>,
    /// The assembled Java source. Empty exactly when [`Self::declaration`] is `None`.
    pub text: String,
    /// The complete effective limits this request ran under.
    pub limits: Limits,
    /// What those limits were charged, in the engine's own dimensions.
    ///
    /// The class-read shape does not depend on how many members the class has: one `class_headers`
    /// attempt and one read of the class bytes for the binding, one of each for the one preparation
    /// every member body is decoded against, and then one `method_bodies` attempt per member that
    /// declares a body. A member's own run charges no class header and no class bytes — it consumes
    /// the preparation — so `class_bytes` is twice the class's own length for any class with at least
    /// one body, and a member that declares none, or one whose class could not be prepared, adds no
    /// body attempt at all.
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
    /// table's own stop and every member's run.
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
/// `ACC_TRANSIENT` on a field (JVMS 4.5-A).
const ACC_TRANSIENT: u16 = 0x0080;
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

/// One internal name as source spells it: `/` becomes `.` and nothing else happens.
///
/// The one change is deliberate. Turning `p.Outer$Inner` into a nesting relation, or splitting a
/// name into a package and a simple name, would be inventing structure the class file does not
/// state; `$` is a legal Java identifier character, and the name stays the one the bytes carry.
fn class_name(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).replace('/', ".")
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
    /// The slot is the JVM layer's derivation from the descriptor's own facts
    /// ([`parameter_positions`]): `this` holds slot 0 of a member that is not `static`, and each
    /// parameter starts where the one before it ended — a `long`/`double` filling two slots and
    /// **an array of either filling one**, like every other array.
    parameters: Vec<(String, u16)>,
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
fn method_descriptor(descriptor: &[u8], is_static: bool) -> Option<Signature> {
    let facts = descriptor_facts(descriptor, DescriptorKind::Method).ok()?;
    let positions = parameter_positions(&facts, is_static)?;
    let parameters = facts
        .parameters()
        .iter()
        .zip(positions)
        .map(|(component, slot)| Some((source_type(component)?, slot)))
        .collect::<Option<Vec<(String, u16)>>>()?;
    let returns = match facts.result() {
        Some(component) => Some(source_type(component)?),
        None => None,
    };
    let slots = u16::from(!is_static).checked_add(facts.parameter_slots()?)?;
    Some(Signature {
        parameters,
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
/// * an interface's `extends` list is the interfaces the class file declares, because its own
///   `super_class` is `java/lang/Object` by the format's rule and is not a superclass in source.
fn class_declaration(name: &str, facts: &ClassDeclarationFacts) -> String {
    let flags = facts.access_flags;
    let interface = flags & ACC_INTERFACE != 0;
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
    let supers: Vec<String> = facts
        .interfaces
        .iter()
        .map(|interface| class_name(&interface.raw().0))
        .collect();
    if interface {
        if !supers.is_empty() {
            line.push_str(" extends ");
            line.push_str(&supers.join(", "));
        }
        return line;
    }
    if !enumeration && let Some(super_class) = &facts.super_class {
        line.push_str(" extends ");
        line.push_str(&class_name(&super_class.raw().0));
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
            _ => SlotEvidence::Split(names),
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

/// Spells one method's declaration, and the marker its own spelling needs.
///
/// `class` is the class's own spelling ([`ClassSourceDeclaration::name`]), which a constructor's
/// declaration is written with: the class file names it `<init>`, and Java spells the same
/// declaration with the class's name, so nothing here claims a name the bytes do not state — the raw
/// name stays in [`ClassSourceMethod::item`]. `<clinit>` is spelled as Java's static initializer
/// block (`static`), which is the declaration the class file means.
pub(crate) fn spell_method(
    item: &MethodItem,
    facts: Option<&RecoveryFacts>,
    class: &str,
) -> Spelled {
    let raw = &item.name.raw().0;
    match raw.as_slice() {
        b"<clinit>" => {
            return Spelled {
                declaration: Some("static".to_owned()),
                marker: None,
            };
        }
        b"<init>" => {
            let Some(signature) =
                method_descriptor(&item.descriptor.raw().0, is_static(item.access_flags))
            else {
                return Spelled {
                    declaration: None,
                    marker: Some(not_a_descriptor(item, "method")),
                };
            };
            let mut declaration = String::new();
            if let Some(word) = visibility(item.access_flags) {
                declaration.push_str(word);
                declaration.push(' ');
            }
            declaration.push_str(class);
            declaration.push_str(&arguments(facts, &signature));
            return Spelled {
                declaration: Some(declaration),
                marker: None,
            };
        }
        _ => {}
    }
    let Some(signature) = method_descriptor(&item.descriptor.raw().0, is_static(item.access_flags))
    else {
        return Spelled {
            declaration: None,
            marker: Some(not_a_descriptor(item, "method")),
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
    let returns = signature.returns.as_deref().unwrap_or("void");
    let mut declaration = String::new();
    for word in words {
        declaration.push_str(word);
        declaration.push(' ');
    }
    declaration.push_str(returns);
    declaration.push(' ');
    declaration.push_str(&name);
    declaration.push_str(&arguments(facts, &signature));
    Spelled {
        declaration: Some(declaration),
        marker: aliased.then(|| aliased_name(item)),
    }
}

/// Spells one field's declaration, and the marker its own spelling needs.
fn spell_field(item: &FieldItem) -> Spelled {
    let Some(field_type) = field_type(&item.descriptor.raw().0) else {
        return Spelled {
            declaration: None,
            marker: Some(not_a_descriptor_field(item)),
        };
    };
    let (name, aliased) = written_name(&item.name.raw().0);
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
    declaration.push_str(&field_type);
    declaration.push(' ');
    declaration.push_str(&name);
    Spelled {
        declaration: Some(declaration),
        marker: aliased.then(|| {
            format!(
                "// jarde: the field's raw name `{}` is not a Java identifier; it is written as `{name}`",
                comment_name(&item.name)
            )
        }),
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
fn arguments(facts: Option<&RecoveryFacts>, signature: &Signature) -> String {
    let names = parameter_names(facts, signature.slots);
    let written: Vec<String> = signature
        .parameters
        .iter()
        .map(|(ty, slot)| {
            let name = names
                .get(usize::from(*slot))
                .cloned()
                .unwrap_or_else(|| format!("arg{slot}"));
            format!("{ty} {name}")
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
    let mut markers = Vec::new();
    if let Some(marker) = &spelled.marker {
        markers.push(marker.clone());
    }
    let member = label(item);
    if let Some(stop) = stop_code(&report.execution) {
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
    markers
}

// ---------------------------------------------------------------------------------------------
// The members, as this presentation builds them
// ---------------------------------------------------------------------------------------------

impl ClassSourceDeclaration {
    /// The declaration of one class-level item, as this presentation spells it.
    pub(crate) fn of(item: ClassDeclarationItem) -> Self {
        let name = class_name(&item.declaration.this_class.raw().0);
        let declaration = class_declaration(&name, &item.declaration);
        Self {
            item,
            name,
            declaration,
        }
    }
}

impl ClassSourceField {
    /// One field record with the spelling the text writes for it.
    pub(crate) fn of(item: FieldItem) -> Self {
        let Spelled {
            declaration,
            marker,
        } = spell_field(&item);
        Self {
            item,
            declaration,
            markers: marker.into_iter().collect(),
        }
    }
}

impl ClassSourceMethod {
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
        Self {
            item,
            no_body_kind: kind,
            declaration: spelled.declaration,
            text,
            markers,
            outcome: ClassSourceOutcome::NoBody,
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
            markers,
            outcome: ClassSourceOutcome::Unspelled,
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
        Self {
            item,
            no_body_kind: None,
            declaration: spelled.declaration,
            text,
            markers,
            outcome: ClassSourceOutcome::Refused {
                execution,
                diagnostics,
            },
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
        Self {
            item,
            no_body_kind: None,
            declaration: spelled.declaration,
            text,
            markers,
            outcome: ClassSourceOutcome::Recovered { report, analysis },
        }
    }
}

/// The markers one spelling carries, as the vector the member record publishes.
fn markers_of(spelled: &Spelled) -> Vec<String> {
    spelled.marker.clone().into_iter().collect()
}

/// The marker of a produced artifact whose shape this presentation does not recognize.
fn not_placed_marker(item: &MethodItem) -> String {
    format!(
        "// jarde: not placed: the recovery artifact for `{}` is not a block this presentation can \
         place, so its text is quoted below line by line",
        label(item)
    )
}

/// The class-source text: the declaration, the fields, the members, and the class file's own
/// shortfall where there is one.
pub(crate) fn source_text(
    declaration: &ClassSourceDeclaration,
    fields: &[ClassSourceField],
    methods: &[ClassSourceMethod],
    declared_methods: u64,
    member_table: Option<&MemberTableStop>,
    execution: &ExecutionReport,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "// jarde: presentation of `{}` from the class file's own declaration and one recovery run per member.\n",
        comment_name(&declaration.item.declaration.this_class)
    ));
    out.push_str(
        "// jarde: not a compilable project: no package, no imports and no resources are claimed; \
         every place this text is not a full recovery carries a marker of this prefix.\n",
    );
    out.push_str(&declaration.declaration);
    out.push_str(" {\n");
    let mut first = true;
    for field in fields {
        if !first {
            out.push('\n');
        }
        first = false;
        match (&field.declaration, field.markers.is_empty()) {
            (Some(declaration), _) => {
                out.push_str(&declaration_member(declaration, &field.markers))
            }
            (None, _) => out.push_str(&comment_member(&field.markers)),
        }
    }
    for method in methods {
        if !first {
            out.push('\n');
        }
        first = false;
        out.push_str(&method.text);
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
    out.push_str("}\n");
    out
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

    #[test]
    fn one_method_descriptor_is_spelled_with_its_slots_and_its_return_type() {
        let signature =
            method_descriptor(b"(JLjava/lang/String;[I)V", true).expect("a method descriptor");
        assert_eq!(
            signature.parameters,
            vec![
                ("long".to_owned(), 0),
                ("java.lang.String".to_owned(), 2),
                ("int[]".to_owned(), 3),
            ],
            "a wide parameter occupies two slots, and the next one starts after it"
        );
        assert_eq!(signature.returns, None, "`V` states no return type");
        assert_eq!(signature.slots, 4);

        let empty = method_descriptor(b"()[Ljava/lang/Object;", true).expect("a method descriptor");
        assert!(empty.parameters.is_empty());
        assert_eq!(empty.returns.as_deref(), Some("java.lang.Object[]"));
        assert_eq!(empty.slots, 0);
    }

    /// JVMS 2.6.1: an array fills **one** slot whatever its element type, and a member that is not
    /// `static` puts its receiver in slot 0 — the two facts the parameter names of a declaration are
    /// looked up by, and the two the body's own `arg<slot>` names are numbered by.
    #[test]
    fn an_arrays_element_width_does_not_move_the_parameters_after_it() {
        let static_mixed = method_descriptor(b"([JI)V", true).expect("a method descriptor");
        assert_eq!(
            static_mixed.parameters,
            vec![("long[]".to_owned(), 0), ("int".to_owned(), 1)],
            "`long[]` is a reference: the `int` after it is at slot 1, not at slot 2"
        );
        assert_eq!(static_mixed.slots, 2);

        let instance_mixed = method_descriptor(b"([JI)V", false).expect("a method descriptor");
        assert_eq!(
            instance_mixed.parameters,
            vec![("long[]".to_owned(), 1), ("int".to_owned(), 2)],
            "slot 0 holds the receiver of a member that is not static"
        );
        assert_eq!(instance_mixed.slots, 3);

        let double_grid = method_descriptor(b"([[DJ)J", true).expect("a method descriptor");
        assert_eq!(
            double_grid.parameters,
            vec![("double[][]".to_owned(), 0), ("long".to_owned(), 1)],
            "a two-dimensional `double[][]` is one slot, and the `long` after it two"
        );
        assert_eq!(double_grid.slots, 3);

        // The controls: a `double` written on its own really is two slots, and an `int[]` (whose
        // element is one slot wide) reads the same way before and after this rule.
        let wide = method_descriptor(b"(DI)J", true).expect("a method descriptor");
        assert_eq!(
            wide.parameters,
            vec![("double".to_owned(), 0), ("int".to_owned(), 2)]
        );
        assert_eq!(wide.slots, 3);
        assert_eq!(wide.returns.as_deref(), Some("long"));
        let narrow_array = method_descriptor(b"([II)V", true).expect("a method descriptor");
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
                method_descriptor(descriptor, true),
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
}
