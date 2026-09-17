//! Member declaration lookup (JVMS 5.4.3) with the access and invocation-kind rules
//! (JVMS 5.4.4).
//!
//! This module is the 2.3 slice of the demand resolver: given a member symbol, the reference
//! use and the caller's identity, it answers which declaration the reference resolves to and
//! whether this reference may use it. It implements no dispatch (2.5) and no use-site scanning
//! (2.4), and it reads class headers only.
//!
//! The search follows the JVMS structure instead of recursing on a name:
//!
//! * a field is looked for in the class itself, then in its direct superinterfaces (each one
//!   recursively, in declaration order), and then in its superclass (JVMS 5.4.3.2);
//! * a method named by a class reference is looked for in the class and its superclass chain,
//!   and only when that chain holds nothing in the superinterfaces (JVMS 5.4.3.3);
//! * a method named by an interface reference is looked for in the interface itself, and then
//!   in its superinterfaces (JVMS 5.4.3.4).
//!
//! The superinterface step reduces its candidates to the maximally-specific set before one is
//! chosen. The set is built the way JVMS 5.4.3.3/5.4.3.4 define it: a candidate is a
//! superinterface declaration that matches the reference and has **neither `ACC_PRIVATE` nor
//! `ACC_STATIC`** set — those are ignored by resolution — and of those, none is overridden by a
//! candidate declared in one of its own subinterfaces. An empty set is a lookup failure
//! (`Missing`), not a conflict. Of what remains, exactly one non-abstract candidate is the
//! default method that resolves, several non-abstract candidates are the Java 8 default
//! conflict, and a set that is abstract only still resolves (JVMS puts that failure at the
//! invocation, not at resolution). JVMS leaves the choice among abstract-only candidates
//! arbitrary; this slice picks the first one in expansion order, which is declaration order
//! level by level, so two runs of one request answer the same way.
//!
//! Every class the search reads is demanded through the 2.2 closure, so a repeated read is
//! answered from the request memo and each step is charged as one `AnalysisSteps` under the
//! dependency-depth bound *before* it is taken. The read reason follows the hierarchy edge the
//! search really took, which is what the report publishes: the class the reference names and
//! the class of the use site are `MemberOwner` reads, a `super_class` step is a `ParentChain`
//! read, and a superinterface step is a `HierarchyClosure` read. A step reached along an
//! `interfaces` edge is therefore never recorded as a parent-chain read and vice versa, in the
//! field walk, the class chain, the interface graph and the protected rule's subtype walk
//! alike.
//!
//! What this slice deliberately does not decide is named where it matters:
//!
//! * a Java 8 default conflict is reported *at resolution* (`IncompatibleClassChange` with
//!   `resolution_default_conflict`) while the JVMS 8 puts that failure in the invocation
//!   selection; the maximally-specific set the JVMS asks for is computed here, the judgement is
//!   one step early on purpose so a caller never sees a silent pick;
//! * the field rules compare the instruction (`GetStatic`/`PutStatic` against
//!   `GetField`/`PutField`), which a `MemberUse` does not carry; the use-site slices provide
//!   that level, and member resolution itself applies no static/instance rule to a field;
//! * the declaring class's own accessibility (JVMS 5.4.3.1) is not part of this rule table —
//!   the access rules here are the member's own `public`/`private`/`protected`/package flags;
//! * an interface owner does not inherit `java/lang/Object`'s methods here, so a member only
//!   `java/lang/Object` declares resolves to `Missing` for an interface owner and never to a
//!   fabricated `Object` declaration;
//! * a supertype that cannot be read (missing, indistinguishable, cyclic) ends its own branch
//!   and is reported as an unread branch: the decision is published with a warning and partial
//!   coverage instead of being presented as a complete search.
//!
//! All of them are the approximations the design fixes as this slice's semantic boundary: they
//! are recorded facts, not claims that JVMS 5.4.3 is fully implemented here.

use crate::budget::{Budget, CountedBudgetDimension};
use crate::classfile::{ClassFacts, MemberHeader};
use crate::environment::CallerContext;
use crate::error::{Error, Result};
use crate::model::{Diagnostic, DiagnosticSeverity, JvmBytes, PhysicalDefinitionId, SymbolRef};
use crate::providers::{
    ClassHandle, HIERARCHY_CYCLE, HeaderClosure, HeaderDemand, HeaderLookupState, escaped,
};
use crate::view::LoaderId;
use std::collections::VecDeque;

/// Diagnostic code of a Java 8 default-method conflict.
const DEFAULT_CONFLICT: &str = "resolution_default_conflict";
/// Diagnostic code of a reference kind that contradicts the declaration it resolves to.
const KIND_MISMATCH: &str = "resolution_kind_mismatch";
/// Diagnostic code of a member the caller's class may not access.
const ACCESS_DENIED: &str = "resolution_access_denied";
/// Diagnostic code of a resolution whose access rules were not applied.
const ACCESS_NOT_CHECKED: &str = "resolution_access_not_checked";
/// Diagnostic code of a signature-polymorphic declaration matched by name.
const SIGNATURE_POLYMORPHIC: &str = "resolution_signature_polymorphic";
/// Diagnostic code of a declaration that is abstract but still resolved.
const METHOD_IS_ABSTRACT: &str = "resolution_method_is_abstract";
/// Diagnostic code of an owner kind this slice does not resolve.
const ARRAY_OWNER: &str = "resolution_array_owner";
/// Diagnostic code of a supertype no readable position holds.
const HIERARCHY_MISSING: &str = "resolution_hierarchy_missing";
/// Diagnostic code of a supertype that cannot be told apart from another definition.
const HIERARCHY_AMBIGUOUS: &str = "resolution_hierarchy_ambiguous";

/// `ACC_PUBLIC`, `ACC_PRIVATE`, `ACC_PROTECTED`, `ACC_STATIC` and the class/interface flags of
/// JVMS 4.6/4.1.
const ACC_PUBLIC: u16 = 0x0001;
const ACC_PRIVATE: u16 = 0x0002;
const ACC_PROTECTED: u16 = 0x0004;
const ACC_STATIC: u16 = 0x0008;
const ACC_INTERFACE: u16 = 0x0200;
const ACC_ABSTRACT: u16 = 0x0400;
const ACC_ANNOTATION: u16 = 0x2000;

/// The class that declares the two signature-polymorphic methods (JVMS 2.9).
const METHOD_HANDLE: &[u8] = b"java/lang/invoke/MethodHandle";
/// The name of the first signature-polymorphic method.
const INVOKE: &[u8] = b"invoke";
/// The name of the second signature-polymorphic method.
const EXACT: &[u8] = b"invokeExact";

/// How one reference uses the member it names.
///
/// This mirror of the report's `ReferenceUse` keeps the dependency direction
/// `resolver -> members -> providers` acyclic; the report layer maps the public vocabulary onto
/// it exhaustively, so a new reference kind cannot be forgotten here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MemberUse {
    FieldRead,
    FieldWrite,
    InvokeStatic,
    InvokeSpecial,
    InvokeVirtual,
    InvokeInterface,
    InvokeDynamic,
}

impl MemberUse {
    /// The name this use carries in diagnostics: the request vocabulary's spelling.
    fn name(self) -> &'static str {
        match self {
            Self::FieldRead => "field_read",
            Self::FieldWrite => "field_write",
            Self::InvokeStatic => "invoke_static",
            Self::InvokeSpecial => "invoke_special",
            Self::InvokeVirtual => "invoke_virtual",
            Self::InvokeInterface => "invoke_interface",
            Self::InvokeDynamic => "invoke_dynamic",
        }
    }
}

/// Which member list of a class a search looks in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MemberKind {
    Field,
    Method,
}

/// One class that declares the member a search selected, with the declaration itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MemberLocation {
    pub(crate) loader: LoaderId,
    pub(crate) definition: PhysicalDefinitionId,
    /// Internal name the declaring class declares for itself (`this_class`), never the name it
    /// was demanded under: an archive entry's path is not a class name.
    pub(crate) declaring_class: JvmBytes,
    pub(crate) kind: MemberKind,
    pub(crate) name: JvmBytes,
    /// The declaration's own descriptor, which differs from the request's for a
    /// signature-polymorphic method.
    pub(crate) descriptor: JvmBytes,
    pub(crate) access_flags: u16,
}

/// One class definition that cannot be told apart at the reference's owner position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OwnerCandidate {
    pub(crate) loader: LoaderId,
    pub(crate) definition: PhysicalDefinitionId,
}

/// What one member resolution decided.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MemberDecision {
    /// One declaration was selected.
    ///
    /// Boxed because a location carries a whole physical definition and this enum is handed
    /// around as one value: the box keeps the type small without adding a second allocation for
    /// the lists the other variants own.
    Resolved(Box<MemberLocation>),
    /// No class of the searched hierarchy declares the member.
    Missing,
    /// The reference's owner class is indistinguishable at one selection position.
    OwnerAmbiguous(Vec<OwnerCandidate>),
    /// One class declares the member more than once with the same name and descriptor.
    DeclarationAmbiguous(Vec<MemberLocation>),
    /// The reference kind contradicts the declaration or the owner's kind.
    KindMismatch,
    /// Two or more non-abstract interface defaults are maximally specific.
    DefaultConflict,
    /// The caller's class may not access the member (JVMS 5.4.4).
    AccessDenied,
    /// The owner is an array type, whose members this slice does not resolve.
    ArrayOwner,
}

/// Result of one member resolution that reached a decision.
///
/// A refusal (a budget stop, a cancellation, damaged bytes) is the `Err` of the call: it ended
/// the request before any decision, so it is not a decision that happens to be unhappy.
#[derive(Clone, Debug)]
pub(crate) struct MemberOutcome {
    pub(crate) decision: MemberDecision,
    /// The warnings and notes the decision carries, in the order they were produced: the unread
    /// branches of the hierarchy first, then the rules the selected declaration was held to.
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// False when a branch of the hierarchy could not be read, so the resolution plane is
    /// partial even though a decision was reached.
    pub(crate) hierarchy_complete: bool,
}

impl MemberOutcome {
    fn plain(decision: MemberDecision) -> Self {
        Self {
            decision,
            diagnostics: Vec::new(),
            hierarchy_complete: true,
        }
    }
}

/// Resolves one field or method reference under the caller's own search order.
pub(crate) fn resolve_member(
    closure: &mut HeaderClosure<'_>,
    target: &SymbolRef,
    use_kind: MemberUse,
    caller: &CallerContext,
    budget: &mut Budget,
) -> Result<MemberOutcome> {
    let MemberParts {
        kind,
        owner,
        name,
        descriptor,
    } = member_parts(target)?;
    if owner.first() == Some(&b'[') {
        return Ok(MemberOutcome {
            decision: MemberDecision::ArrayOwner,
            diagnostics: vec![array_owner_diagnostic(owner)],
            hierarchy_complete: true,
        });
    }
    // JVMS 2.9: `MethodHandle.invoke`/`invokeExact` are matched by name, because the call
    // site's descriptor is not the declaration's. The branch is the owner's own, not a
    // property of the name: no other class's `invoke` is matched this way.
    let signature_polymorphic =
        kind == MemberKind::Method && owner == METHOD_HANDLE && (name == INVOKE || name == EXACT);
    let rule = MatchRule {
        kind,
        name,
        descriptor,
        name_only: signature_polymorphic,
    };

    let mut search = Search::default();
    let owner_handle = demand_owner(closure, owner, budget)?;
    match closure.resolution(owner_handle).lookup.state {
        // 1. The owner class is resolved first (JVMS 5.4.3.1), and a reference whose owner does
        //    not resolve inherits that state instead of searching a hierarchy it does not know.
        HeaderLookupState::Missing => return Ok(MemberOutcome::plain(MemberDecision::Missing)),
        HeaderLookupState::Ambiguous => {
            let candidates = closure
                .resolution(owner_handle)
                .lookup
                .candidates
                .iter()
                .map(|location| OwnerCandidate {
                    loader: location.loader.clone(),
                    definition: location.definition.clone(),
                })
                .collect();
            return Ok(MemberOutcome::plain(MemberDecision::OwnerAmbiguous(
                candidates,
            )));
        }
        HeaderLookupState::Found => {}
    }

    let root = class_site(closure, owner_handle, Some(&rule));
    let selection = match kind {
        MemberKind::Field => search_field(closure, &root, &rule, budget, &mut search)?,
        MemberKind::Method => search_method(closure, &root, &rule, use_kind, budget, &mut search)?,
    };
    let mut rules = Vec::new();
    let decision = match selection {
        Selection::None => MemberDecision::Missing,
        Selection::Several(locations) => MemberDecision::DeclarationAmbiguous(locations),
        Selection::OwnerKindMismatch => {
            rules.push(owner_kind_diagnostic(&root, use_kind));
            MemberDecision::KindMismatch
        }
        Selection::Conflict(locations) => {
            rules.push(default_conflict_diagnostic(&locations));
            MemberDecision::DefaultConflict
        }
        Selection::One(location) => {
            // 2. The invocation-kind rules of the resolved declaration (JVMS 5.4.3.3/5.4.3.4).
            if let Some(diagnostic) = kind_rule_violation(kind, use_kind, &location) {
                rules.push(diagnostic);
                MemberDecision::KindMismatch
            } else {
                if signature_polymorphic {
                    rules.push(signature_polymorphic_note(&location, descriptor));
                }
                if location.access_flags & ACC_ABSTRACT != 0 {
                    rules.push(abstract_note(&location));
                }
                // 3. The access rules (JVMS 5.4.4), which are separate from resolution.
                match access_decision(closure, &location, caller, budget, &mut search)? {
                    Access::Allowed => MemberDecision::Resolved(location),
                    Access::Denied { caller_class } => {
                        rules.push(access_denied_diagnostic(&location, &caller_class));
                        MemberDecision::AccessDenied
                    }
                    Access::NotChecked => {
                        rules.push(access_not_checked_diagnostic(caller));
                        MemberDecision::Resolved(location)
                    }
                }
            }
        }
    };
    let hierarchy_complete = !search.incomplete();
    let mut diagnostics = std::mem::take(&mut search.diagnostics);
    diagnostics.extend(rules);
    Ok(MemberOutcome {
        decision,
        diagnostics,
        hierarchy_complete,
    })
}

/// The parts of one member reference, plus which member list it names.
struct MemberParts<'a> {
    kind: MemberKind,
    owner: &'a [u8],
    name: &'a [u8],
    descriptor: &'a [u8],
}

/// The parts of a member reference.
///
/// A class symbol reaching this slice is a request mismatch: the entry point rejects that
/// pairing, so the error is the same code with the same meaning rather than a panic.
fn member_parts(target: &SymbolRef) -> Result<MemberParts<'_>> {
    match target {
        SymbolRef::Field {
            owner,
            name,
            descriptor,
        } => Ok(MemberParts {
            kind: MemberKind::Field,
            owner: &owner.0,
            name: &name.0,
            descriptor: &descriptor.0,
        }),
        SymbolRef::Method {
            owner,
            name,
            descriptor,
        } => Ok(MemberParts {
            kind: MemberKind::Method,
            owner: &owner.0,
            name: &name.0,
            descriptor: &descriptor.0,
        }),
        SymbolRef::Class { .. } => Err(Error::invalid_input(
            "resolution_target_use_mismatch",
            "a class symbol is not a member reference; member resolution needs a field or a \
             method symbol",
        )),
    }
}

/// What one member reference matches inside one class's header.
struct MatchRule<'a> {
    kind: MemberKind,
    name: &'a [u8],
    descriptor: &'a [u8],
    /// A signature-polymorphic method is matched by name alone (JVMS 5.4.3.3 step 2.1).
    name_only: bool,
}

impl MatchRule<'_> {
    fn matches(&self, member: &MemberHeader) -> bool {
        member.name.raw().0 == self.name
            && (self.name_only || member.descriptor.raw().0 == self.descriptor)
    }
}

/// One member declaration a class declares for the requested member.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Declaration {
    descriptor: JvmBytes,
    access_flags: u16,
}

impl Declaration {
    fn is_abstract(&self) -> bool {
        self.access_flags & ACC_ABSTRACT != 0
    }

    /// Whether resolution counts this declaration in a superinterface step (JVMS 5.4.3.3/5.4.3.4).
    ///
    /// A maximally-specific superinterface method is declared with **neither `ACC_PRIVATE` nor
    /// `ACC_STATIC`** set: "Superinterface methods that are private and static are ignored by
    /// resolution". So a `static` method of one superinterface neither provides the resolution
    /// nor suppresses the `default` of another, and a `private` declaration in a subinterface
    /// does not hide its parent's `default`.
    ///
    /// Only the superinterface step filters this way. A declaration of the owner itself, or of a
    /// class-chain site, is matched unfiltered: `invokestatic` naming a static interface method
    /// resolves, and `invokeinterface` naming one is rejected by the invocation-kind rules.
    fn counts_in_superinterface_step(&self) -> bool {
        self.access_flags & (ACC_STATIC | ACC_PRIVATE) == 0
    }
}

/// One class the search read: its identity, its own supertypes and what it declares for the
/// member.
#[derive(Clone, Debug)]
struct ClassSite {
    loader: LoaderId,
    definition: PhysicalDefinitionId,
    /// The name the class declares for itself, never the name it was demanded under.
    name: JvmBytes,
    super_class: Option<JvmBytes>,
    interfaces: Vec<JvmBytes>,
    is_interface: bool,
    /// The member list this site was matched against, and what it declares for the request.
    /// A site read for its structure only (the access rules' hierarchy walk) declares nothing
    /// and never produces a location.
    kind: Option<MemberKind>,
    declared: Vec<Declaration>,
}

impl ClassSite {
    fn location(&self, name: &[u8], declaration: &Declaration) -> MemberLocation {
        let kind = self
            .kind
            .expect("only a site matched against a member list selects a declaration");
        MemberLocation {
            loader: self.loader.clone(),
            definition: self.definition.clone(),
            declaring_class: self.name.clone(),
            kind,
            name: JvmBytes(name.to_vec()),
            descriptor: declaration.descriptor.clone(),
            access_flags: declaration.access_flags,
        }
    }

    /// The supertypes this class declares, superclass first and interfaces in declaration
    /// order, each with the demand its edge stands for.
    fn supertypes(&self, depth: u64) -> Vec<(JvmBytes, HeaderDemand, u64)> {
        let mut names = Vec::new();
        if let Some(super_class) = &self.super_class {
            names.push((super_class.clone(), HeaderDemand::ParentChain, depth));
        }
        for interface in &self.interfaces {
            names.push((interface.clone(), HeaderDemand::HierarchyClosure, depth));
        }
        names
    }
}

/// The declaration a search selected, or why it selected none.
enum Selection {
    One(Box<MemberLocation>),
    /// Several declarations cannot be told apart at one selection position.
    Several(Vec<MemberLocation>),
    /// Two or more non-abstract interface defaults are maximally specific.
    Conflict(Vec<MemberLocation>),
    /// Nothing declares the member in the branches the search could read.
    None,
    /// The reference kind requires the other owner kind.
    OwnerKindMismatch,
}

/// What the searches of one request found out about the hierarchy itself.
#[derive(Default)]
struct Search {
    /// Branches the searches could not read, with the diagnostics that name them.
    diagnostics: Vec<Diagnostic>,
    /// Branches the searches could not read: a supertype that is missing, indistinguishable or
    /// cyclic. A refusal (budget, cancellation, damage) is not one of them — it ends the request.
    unread: u64,
}

impl Search {
    fn incomplete(&self) -> bool {
        self.unread > 0
    }
}

/// The class a reference names as its owner, demanded as step 0 of the search.
fn demand_owner(
    closure: &mut HeaderClosure<'_>,
    owner: &[u8],
    budget: &mut Budget,
) -> Result<ClassHandle> {
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    closure
        .demand(owner, HeaderDemand::MemberOwner, budget)
        .decision
}

/// Demands one supertype layer, one dependency step above the class that declares it.
///
/// `demand` is the demand of the edge that reached this layer: a `super_class` step is
/// [`HeaderDemand::ParentChain`] and a superinterface step is
/// [`HeaderDemand::HierarchyClosure`], so the report's reasons follow the hierarchy edges
/// instead of labelling every step of the search the same way. A layer that is already read is
/// answered from the request memo and keeps the reason of the read that really happened.
///
/// A layer no position holds, or one that cannot be told apart, ends its own branch: it is
/// recorded as an unread branch and the search continues with the branches it can read. A
/// budget stop, a cancellation and a damaged candidate are refusals, not unread branches.
fn demand_layer(
    closure: &mut HeaderClosure<'_>,
    name: &JvmBytes,
    demand: HeaderDemand,
    depth: u64,
    budget: &mut Budget,
    search: &mut Search,
) -> Result<Option<ClassHandle>> {
    budget.observe_dependency_depth(depth)?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    let handle = closure.demand(&name.0, demand, budget).decision?;
    let (state, loader) = {
        let resolution = closure.resolution(handle);
        (resolution.lookup.state, resolution.loader.clone())
    };
    match state {
        HeaderLookupState::Found => Ok(Some(handle)),
        HeaderLookupState::Missing => {
            search.unread += 1;
            search.diagnostics.push(hierarchy_warning(
                HIERARCHY_MISSING,
                &loader,
                name,
                "no readable position holds it",
            ));
            Ok(None)
        }
        HeaderLookupState::Ambiguous => {
            search.unread += 1;
            search.diagnostics.push(hierarchy_warning(
                HIERARCHY_AMBIGUOUS,
                &loader,
                name,
                "several definitions at one position cannot be told apart",
            ));
            Ok(None)
        }
    }
}

/// The site of one found class, with the declarations that match the rule.
fn class_site(
    closure: &HeaderClosure<'_>,
    handle: ClassHandle,
    rule: Option<&MatchRule<'_>>,
) -> ClassSite {
    let resolution = closure.resolution(handle);
    let location = resolution
        .lookup
        .location
        .as_ref()
        .expect("only a found class is turned into a site");
    let facts = &resolution
        .lookup
        .header
        .as_ref()
        .expect("a found class publishes the header it read")
        .facts;
    ClassSite {
        loader: location.loader.clone(),
        definition: location.definition.clone(),
        name: JvmBytes(facts.this_class.raw().0.clone()),
        super_class: facts
            .super_class
            .as_ref()
            .map(|name| JvmBytes(name.raw().0.clone())),
        interfaces: facts
            .interfaces
            .iter()
            .map(|name| JvmBytes(name.raw().0.clone()))
            .collect(),
        is_interface: is_interface(facts.access_flags),
        kind: rule.map(|rule| rule.kind),
        declared: rule.map_or_else(Vec::new, |rule| matching(facts, rule)),
    }
}

/// The declarations of one class that match the rule, in declaration order.
fn matching(facts: &ClassFacts, rule: &MatchRule<'_>) -> Vec<Declaration> {
    let members = match rule.kind {
        MemberKind::Field => &facts.fields,
        MemberKind::Method => &facts.methods,
    };
    members
        .iter()
        .filter(|member| rule.matches(member))
        .map(|member| Declaration {
            descriptor: JvmBytes(member.descriptor.raw().0.clone()),
            access_flags: member.access_flags,
        })
        .collect()
}

/// A class flag list that declares an interface (`ACC_INTERFACE` or `ACC_ANNOTATION`).
fn is_interface(access_flags: u16) -> bool {
    access_flags & (ACC_INTERFACE | ACC_ANNOTATION) != 0
}

/// JVMS 5.4.3.2 field lookup.
///
/// The order is part of the rule: the class itself, then each direct superinterface (searched
/// completely, in declaration order), and only then the superclass. A field declared in an
/// interface therefore shadows one of the same name in a superclass, which is why this walk is a
/// stack and not a breadth-first queue.
fn search_field(
    closure: &mut HeaderClosure<'_>,
    root: &ClassSite,
    rule: &MatchRule<'_>,
    budget: &mut Budget,
    search: &mut Search,
) -> Result<Selection> {
    if let Some(selection) = hit(root, rule) {
        return Ok(selection);
    }
    // The superclass is queued first so that it is searched last, and each superinterface is
    // followed completely before the next one starts.
    let mut pending: Vec<FieldLayer> = Vec::new();
    push_field_children(&mut pending, root, 1, &[], search);
    let mut visited: Vec<JvmBytes> = vec![root.name.clone()];
    while let Some(FieldLayer {
        name,
        demand,
        depth,
        path,
    }) = pending.pop()
    {
        if visited.contains(&name) {
            continue;
        }
        visited.push(name.clone());
        let Some(handle) = demand_layer(closure, &name, demand, depth, budget, search)? else {
            continue;
        };
        let site = class_site(closure, handle, Some(rule));
        if let Some(selection) = hit(&site, rule) {
            return Ok(selection);
        }
        push_field_children(&mut pending, &site, depth + 1, &path, search);
    }
    Ok(Selection::None)
}

/// One field layer waiting to be searched: its name, the reason of the edge that queued it, its
/// dependency depth and the path that reached it.
struct FieldLayer {
    name: JvmBytes,
    /// A superclass layer is a parent-chain step and a superinterface layer an interface step,
    /// so the read record names the edge the search really took.
    demand: HeaderDemand,
    depth: u64,
    path: Vec<JvmBytes>,
}

/// Queues the supertypes of one field layer, superclass last.
fn push_field_children(
    pending: &mut Vec<FieldLayer>,
    site: &ClassSite,
    depth: u64,
    path: &[JvmBytes],
    search: &mut Search,
) {
    if let Some(super_class) = &site.super_class {
        push_field_layer(
            pending,
            site,
            super_class,
            HeaderDemand::ParentChain,
            depth,
            path,
            search,
        );
    }
    for interface in site.interfaces.iter().rev() {
        push_field_layer(
            pending,
            site,
            interface,
            HeaderDemand::HierarchyClosure,
            depth,
            path,
            search,
        );
    }
}

/// Queues one field layer unless it is a supertype of its own path or already waiting.
///
/// The path is what distinguishes the two: a diamond re-reaches a class that another branch
/// already searched (nothing new to look at), while a class that is its own supertype is an
/// illegal hierarchy and ends its own branch here — the same rule the method chain and the 2.2
/// hierarchy walk follow.
fn push_field_layer(
    pending: &mut Vec<FieldLayer>,
    site: &ClassSite,
    child: &JvmBytes,
    demand: HeaderDemand,
    depth: u64,
    path: &[JvmBytes],
    search: &mut Search,
) {
    let mut ancestors = path.to_vec();
    ancestors.push(site.name.clone());
    if ancestors.contains(child) {
        search.unread += 1;
        search
            .diagnostics
            .push(cycle_warning(&site.loader, &ancestors, child));
        return;
    }
    if pending.iter().any(|layer| &layer.name == child) {
        return;
    }
    pending.push(FieldLayer {
        name: child.clone(),
        demand,
        depth,
        path: ancestors,
    });
}

/// The selection one class's declarations make, if they make one.
///
/// Two declarations with the same name and descriptor in one class are illegal (JVMS 4.6), and a
/// forged class file that carries them cannot be told apart: no ordinal or hash may order them,
/// so both are published. The report schema has no within-class coordinate, so both candidates
/// reach the report as the same `ResolvedMemberRef` twice — the count is the evidence, and no
/// field of that type may be invented to tell them apart.
fn hit(site: &ClassSite, rule: &MatchRule<'_>) -> Option<Selection> {
    match site.declared.as_slice() {
        [] => None,
        [declaration] => Some(Selection::One(Box::new(
            site.location(rule.name, declaration),
        ))),
        several => Some(Selection::Several(
            several
                .iter()
                .map(|declaration| site.location(rule.name, declaration))
                .collect(),
        )),
    }
}

/// JVMS 5.4.3.3 (a class owner) and 5.4.3.4 (an interface owner) method lookup.
fn search_method(
    closure: &mut HeaderClosure<'_>,
    root: &ClassSite,
    rule: &MatchRule<'_>,
    use_kind: MemberUse,
    budget: &mut Budget,
    search: &mut Search,
) -> Result<Selection> {
    if owner_kind_mismatch(root, use_kind) {
        return Ok(Selection::OwnerKindMismatch);
    }
    if root.is_interface {
        // An interface inherits from its superinterfaces only, and it is searched first.
        if let Some(selection) = hit(root, rule) {
            return Ok(selection);
        }
        let seeds = root
            .interfaces
            .iter()
            .map(|name| (name.clone(), 1))
            .collect();
        return interface_step(closure, seeds, rule, budget, search);
    }
    // A class is searched together with its superclass chain, and the superinterfaces are
    // reached only when the whole chain holds nothing.
    let chain = class_chain(closure, root, rule, budget, search)?;
    if let Some(selection) = chain.selection {
        return Ok(selection);
    }
    let mut seeds = Vec::new();
    for (site, depth) in &chain.sites {
        for interface in &site.interfaces {
            seeds.push((interface.clone(), depth + 1));
        }
    }
    interface_step(closure, seeds, rule, budget, search)
}

/// The owner-kind rule of the two method-resolution modes.
///
/// An interface method reference (`invoke_interface`) names an interface and a class method
/// reference (`invoke_virtual`) names a class, so each requires the owner kind it names. The
/// other three kinds are used with both: a static interface method is reached by
/// `invoke_static`, a superinterface method by `invoke_special`, and `invoke_dynamic` names no
/// such member reference at all.
fn owner_kind_mismatch(root: &ClassSite, use_kind: MemberUse) -> bool {
    match use_kind {
        MemberUse::InvokeInterface => !root.is_interface,
        MemberUse::InvokeVirtual => root.is_interface,
        _ => false,
    }
}

/// The superclass chain of JVMS 5.4.3.3, with the selection it made if it made one.
struct ChainSearch {
    selection: Option<Selection>,
    /// Every class the chain read, with its own dependency depth: the classes that can still
    /// contribute superinterfaces when the chain holds no declaration.
    sites: Vec<(ClassSite, u64)>,
}

fn class_chain(
    closure: &mut HeaderClosure<'_>,
    root: &ClassSite,
    rule: &MatchRule<'_>,
    budget: &mut Budget,
    search: &mut Search,
) -> Result<ChainSearch> {
    let mut sites = Vec::new();
    let mut seen = vec![root.name.clone()];
    let mut current = root.clone();
    let mut depth = 0;
    loop {
        if let Some(selection) = hit(&current, rule) {
            sites.push((current, depth));
            return Ok(ChainSearch {
                selection: Some(selection),
                sites,
            });
        }
        let Some(super_class) = current.super_class.clone() else {
            sites.push((current, depth));
            return Ok(ChainSearch {
                selection: None,
                sites,
            });
        };
        if seen.contains(&super_class) {
            // A chain that re-enters itself never reaches a class that declares nothing and
            // ends: the class file is illegal, and this branch stops here.
            search.unread += 1;
            search
                .diagnostics
                .push(cycle_warning(&current.loader, &seen, &super_class));
            sites.push((current, depth));
            return Ok(ChainSearch {
                selection: None,
                sites,
            });
        }
        seen.push(super_class.clone());
        sites.push((current, depth));
        depth += 1;
        let Some(handle) = demand_layer(
            closure,
            &super_class,
            HeaderDemand::ParentChain,
            depth,
            budget,
            search,
        )?
        else {
            return Ok(ChainSearch {
                selection: None,
                sites,
            });
        };
        current = class_site(closure, handle, Some(rule));
    }
}

/// The superinterface step: read the whole superinterface closure, then decide on its
/// maximally-specific candidates.
///
/// The step is the JVMS's "attempt to locate the method in the superinterfaces": it considers
/// the declarations that resolution counts ([`Declaration::counts_in_superinterface_step`]) and
/// nothing else. An interface whose only matching declaration is `static` or `private`
/// therefore contributes no candidate, which is what turns a reference that names such a member
/// through a class (or through another interface) into `Missing`.
fn interface_step(
    closure: &mut HeaderClosure<'_>,
    seeds: Vec<(JvmBytes, u64)>,
    rule: &MatchRule<'_>,
    budget: &mut Budget,
    search: &mut Search,
) -> Result<Selection> {
    let graph = InterfaceGraph::build(closure, seeds, rule, budget, search)?;
    Ok(graph.decide(rule))
}

/// The superinterfaces one step reached, with what they declare and how they extend each other.
///
/// The maximally-specific set cannot be computed one class at a time: whether a declaration is
/// overridden depends on the interfaces that extend its declaring interface, wherever they sit
/// in the closure. The graph is read once and then decided, exactly like the 2.2 walk reads its
/// whole closure before a consumer uses it.
struct InterfaceGraph {
    nodes: Vec<InterfaceNode>,
}

struct InterfaceNode {
    site: ClassSite,
    /// Node indices the interface's superinterface names resolved to, in declaration order.
    super_nodes: Vec<usize>,
}

impl InterfaceGraph {
    fn build(
        closure: &mut HeaderClosure<'_>,
        seeds: Vec<(JvmBytes, u64)>,
        rule: &MatchRule<'_>,
        budget: &mut Budget,
        search: &mut Search,
    ) -> Result<Self> {
        let mut graph = Self { nodes: Vec::new() };
        let mut queue: VecDeque<(JvmBytes, u64)> = VecDeque::new();
        let mut seen: Vec<JvmBytes> = Vec::new();
        let mut resolved: Vec<(JvmBytes, usize)> = Vec::new();
        for (name, depth) in seeds {
            if !seen.contains(&name) {
                seen.push(name.clone());
                queue.push_back((name, depth));
            }
        }
        while let Some((name, depth)) = queue.pop_front() {
            // Every layer of this graph was reached along an `interfaces` edge, from the seeds
            // the caller queued or from an interface of the closure, so every one of them is an
            // interface-closure read. The seeds came from `interfaces` edges too.
            let Some(handle) = demand_layer(
                closure,
                &name,
                HeaderDemand::HierarchyClosure,
                depth,
                budget,
                search,
            )?
            else {
                continue;
            };
            let site = class_site(closure, handle, Some(rule));
            // Two demanded names that select the same definition are one interface: the alias
            // must not make it a second candidate.
            let index = match graph.identity_index(&site) {
                Some(index) => index,
                None => {
                    graph.nodes.push(InterfaceNode {
                        site: site.clone(),
                        super_nodes: Vec::new(),
                    });
                    graph.nodes.len() - 1
                }
            };
            resolved.push((name, index));
            for interface in &site.interfaces {
                if !seen.contains(interface) {
                    seen.push(interface.clone());
                    queue.push_back((interface.clone(), depth + 1));
                }
            }
        }
        for node in &mut graph.nodes {
            let super_names = node.site.interfaces.clone();
            node.super_nodes = super_names
                .iter()
                .filter_map(|name| {
                    resolved
                        .iter()
                        .find(|(demanded, _)| demanded == name)
                        .map(|(_, index)| *index)
                })
                .collect();
        }
        Ok(graph)
    }

    /// The node of one `(loader, definition)` binding, if the closure already read it.
    fn identity_index(&self, site: &ClassSite) -> Option<usize> {
        self.nodes.iter().position(|node| {
            node.site.loader == site.loader && node.site.definition == site.definition
        })
    }

    /// The decision the maximally-specific set makes.
    fn decide(&self, rule: &MatchRule<'_>) -> Selection {
        let specific = self.maximally_specific();
        if specific.is_empty() {
            return Selection::None;
        }
        let mut per_node: Vec<(usize, Vec<usize>)> = Vec::new();
        for (position, (index, _)) in specific.iter().enumerate() {
            match per_node.iter_mut().find(|(node, _)| node == index) {
                Some((_, positions)) => positions.push(position),
                None => per_node.push((*index, vec![position])),
            }
        }
        if let Some((_, positions)) = per_node.iter().find(|(_, positions)| positions.len() > 1) {
            return Selection::Several(
                positions
                    .iter()
                    .map(|position| self.location(rule.name, &specific[*position]))
                    .collect(),
            );
        }
        let non_abstract = specific
            .iter()
            .filter(|(_, declaration)| !declaration.is_abstract())
            .collect::<Vec<_>>();
        match non_abstract.as_slice() {
            [declaration] => Selection::One(Box::new(self.location(rule.name, declaration))),
            [] => {
                // Every candidate is abstract: resolution still succeeds and the invocation
                // fails. JVMS leaves which one is chosen arbitrary; the first in expansion
                // order keeps one request's answer stable.
                Selection::One(Box::new(self.location(rule.name, &specific[0])))
            }
            several => Selection::Conflict(
                several
                    .iter()
                    .map(|entry| self.location(rule.name, entry))
                    .collect(),
            ),
        }
    }

    fn location(&self, name: &[u8], entry: &(usize, Declaration)) -> MemberLocation {
        self.nodes[entry.0].site.location(name, &entry.1)
    }

    /// The maximally-specific superinterface methods for the reference's name and descriptor.
    ///
    /// Two rules, applied in this order because the second is defined over the candidates the
    /// first leaves:
    ///
    /// 1. a candidate is a declaration of the interface graph that matches the reference and has
    ///    neither `ACC_PRIVATE` nor `ACC_STATIC` set — the JVMS ignores those by resolution, so a
    ///    `static` interface method is neither a resolution result nor a reason for a conflict,
    ///    and a `private` declaration in a subinterface leaves its parent's `default` in place;
    /// 2. of those, none is declared in an interface that another candidate's interface strictly
    ///    extends: the more derived declaration overrides the less derived one (JVMS 5.4.3.3),
    ///    which is what makes a subinterface's `default` win over its parent's.
    ///
    /// An empty result is a lookup failure for the step — the caller turns it into `Missing` —
    /// never a conflict and never a match on a `static` declaration.
    fn maximally_specific(&self) -> Vec<(usize, Declaration)> {
        let mut candidates: Vec<(usize, Declaration)> = Vec::new();
        for (index, node) in self.nodes.iter().enumerate() {
            for declaration in &node.site.declared {
                if declaration.counts_in_superinterface_step() {
                    candidates.push((index, declaration.clone()));
                }
            }
        }
        candidates
            .iter()
            .enumerate()
            .filter(|(position, (index, _))| {
                !candidates
                    .iter()
                    .enumerate()
                    .any(|(other_position, (other, _))| {
                        other_position != *position
                            && other != index
                            && self.extends(*other, *index)
                    })
            })
            .map(|(_, entry)| entry.clone())
            .collect()
    }

    /// Whether one interface of the closure strictly extends another.
    fn extends(&self, sub: usize, super_interface: usize) -> bool {
        let mut pending = vec![sub];
        let mut seen: Vec<usize> = Vec::new();
        while let Some(index) = pending.pop() {
            if seen.contains(&index) {
                continue;
            }
            seen.push(index);
            for next in &self.nodes[index].super_nodes {
                if *next == super_interface {
                    return true;
                }
                pending.push(*next);
            }
        }
        false
    }
}

/// The invocation-kind rules of one resolved declaration.
///
/// JVMS holds each reference kind to the declaration it may name: `invoke_static` requires a
/// static method, the three instance kinds require an instance method, and a constructor may
/// only be resolved through `invoke_special`. `invoke_special` carries one further rule of its
/// own: it may not resolve an abstract method (JVMS 5.4.3.3).
fn kind_rule_violation(
    kind: MemberKind,
    use_kind: MemberUse,
    location: &MemberLocation,
) -> Option<Diagnostic> {
    if kind == MemberKind::Field {
        // A field read and a field write are the same member in this vocabulary, and the
        // static/instance distinction lives in the instruction (`GetStatic`/`PutStatic` against
        // `GetField`/`PutField`), which the use-site slices carry. Member resolution has no
        // field rule of its own.
        return None;
    }
    if location.name.0 == b"<init>" && use_kind != MemberUse::InvokeSpecial {
        return Some(kind_mismatch(format!(
            "{} is a constructor declaration, which only `invoke_special` may resolve, not `{}`",
            describe(location),
            use_kind.name()
        )));
    }
    let is_static = location.access_flags & ACC_STATIC != 0;
    let is_abstract = location.access_flags & ACC_ABSTRACT != 0;
    match use_kind {
        MemberUse::InvokeStatic if !is_static => Some(kind_mismatch(format!(
            "{} is an instance method, but the reference uses `invoke_static`",
            describe(location)
        ))),
        MemberUse::InvokeVirtual | MemberUse::InvokeSpecial | MemberUse::InvokeInterface
            if is_static =>
        {
            Some(kind_mismatch(format!(
                "{} is static, but the reference uses `{}`",
                describe(location),
                use_kind.name()
            )))
        }
        MemberUse::InvokeSpecial if is_abstract => Some(kind_mismatch(format!(
            "{} is abstract, and `invoke_special` may not resolve an abstract method \
             (JVMS 5.4.3.3)",
            describe(location)
        ))),
        _ => None,
    }
}

/// What the access rules (JVMS 5.4.4) decided.
enum Access {
    Allowed,
    /// The caller class is known and the member is not accessible to it.
    Denied {
        caller_class: JvmBytes,
    },
    /// The caller class is unknown, so no rule was applied.
    NotChecked,
}

/// Applies the access rules to one resolved declaration.
///
/// The caller's class is the class that declares the use site's enclosing method, and the
/// request names it as a physical definition: reading its header is what names it. A request
/// without an enclosing method has no caller class, and this slice then says so instead of
/// claiming a check it did not make.
///
/// Two outcomes are deliberately *not* `NotChecked`: a caller definition that does not describe
/// the provided bytes and a caller definition whose content the request does not provide are
/// stops (`state = None` with the refusal in `execution`), because neither names a class the
/// rules could be applied to. `NotChecked` covers the two cases where the caller class is simply
/// unknown: no enclosing method, and a caller hierarchy the request could not read completely —
/// the latter because "not a subclass" cannot be decided from an incomplete hierarchy, and
/// denying access on a guess would be worse than saying the rules were not applied.
fn access_decision(
    closure: &mut HeaderClosure<'_>,
    location: &MemberLocation,
    caller: &CallerContext,
    budget: &mut Budget,
    search: &mut Search,
) -> Result<Access> {
    // A public member is accessible to every caller, so the rule is decided without reading the
    // caller's class at all.
    if location.access_flags & ACC_PUBLIC != 0 {
        return Ok(Access::Allowed);
    }
    let Some(enclosing) = &caller.enclosing else {
        return Ok(Access::NotChecked);
    };
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    let header = closure.read_definition(
        &caller.loader,
        &enclosing.owner,
        HeaderDemand::MemberOwner,
        budget,
    )?;
    let caller_class = JvmBytes(header.facts.this_class.raw().0.clone());
    if caller_class.0 == location.declaring_class.0 {
        return Ok(Access::Allowed);
    }
    if location.access_flags & ACC_PRIVATE != 0 {
        return Ok(Access::Denied { caller_class });
    }
    let same_package = same_runtime_package(
        &location.loader,
        &location.declaring_class.0,
        &caller.loader,
        &caller_class.0,
    );
    if same_package {
        return Ok(Access::Allowed);
    }
    if location.access_flags & ACC_PROTECTED == 0 {
        // Package private outside its own run-time package and no other rule left.
        return Ok(Access::Denied { caller_class });
    }
    let unread_before = search.unread;
    let declaring_class = location.declaring_class.clone();
    let is_subtype = subtype_of(
        closure,
        &header.facts,
        &caller_class.0,
        &declaring_class.0,
        budget,
        search,
    )?;
    if is_subtype {
        return Ok(Access::Allowed);
    }
    if search.unread > unread_before {
        // The caller's own hierarchy could not be read completely, so "not a subclass" is not
        // decided: the resolution says its access rules were not applied rather than denying
        // access on a guess.
        return Ok(Access::NotChecked);
    }
    Ok(Access::Denied { caller_class })
}

/// Whether the caller's class is a subtype of the declaring class.
///
/// The caller's class itself is compared by name before this walk starts, so the walk starts at
/// its own supertypes. Every layer is reached along a hierarchy edge of the caller's own
/// hierarchy, so the read reason follows that edge exactly like the declaring side's search:
/// `super_class` steps are parent-chain reads and `interfaces` steps are interface reads.
fn subtype_of(
    closure: &mut HeaderClosure<'_>,
    caller_facts: &ClassFacts,
    caller_class: &[u8],
    declaring_class: &[u8],
    budget: &mut Budget,
    search: &mut Search,
) -> Result<bool> {
    let mut pending: Vec<(JvmBytes, HeaderDemand, u64)> = Vec::new();
    let mut seen = vec![JvmBytes(caller_class.to_vec())];
    if let Some(super_class) = &caller_facts.super_class {
        pending.push((
            JvmBytes(super_class.raw().0.clone()),
            HeaderDemand::ParentChain,
            1,
        ));
    }
    for interface in &caller_facts.interfaces {
        pending.push((
            JvmBytes(interface.raw().0.clone()),
            HeaderDemand::HierarchyClosure,
            1,
        ));
    }
    while let Some((name, demand, depth)) = pending.pop() {
        if seen.contains(&name) {
            continue;
        }
        seen.push(name.clone());
        let Some(handle) = demand_layer(closure, &name, demand, depth, budget, search)? else {
            continue;
        };
        let site = class_site(closure, handle, None);
        if site.name.0 == declaring_class {
            return Ok(true);
        }
        for (supertype, demand, depth) in site.supertypes(depth + 1) {
            if !seen.contains(&supertype)
                && !pending.iter().any(|(queued, ..)| *queued == supertype)
            {
                pending.push((supertype, demand, depth));
            }
        }
    }
    Ok(false)
}

/// The package part of an internal class name: everything before the last `/`.
///
/// A name without a slash is in the default package, which is a package of its own.
fn package_of(internal_name: &[u8]) -> &[u8] {
    match internal_name.iter().rposition(|byte| *byte == b'/') {
        Some(index) => &internal_name[..index],
        None => &[],
    }
}

/// Whether two classes are in the same run-time package.
///
/// A run-time package is a package name *and* the loader that defines it (JVMS 5.3): two classes
/// whose names share a package are in different run-time packages when their defining loaders
/// differ, and no package-private or protected rule passes between them.
fn same_runtime_package(
    declaring_loader: &LoaderId,
    declaring_class: &[u8],
    caller_loader: &LoaderId,
    caller_class: &[u8],
) -> bool {
    declaring_loader == caller_loader && package_of(declaring_class) == package_of(caller_class)
}

/// One declaration as a diagnostic name: `` `p/Base`::`m()V` `` for a method and
/// `` `p/Base`::`f:I` `` for a field.
fn describe(location: &MemberLocation) -> String {
    let owner = escaped(&location.declaring_class.0);
    let name = escaped(&location.name.0);
    let descriptor = escaped(&location.descriptor.0);
    match location.kind {
        MemberKind::Field => format!("`{owner}`::`{name}:{descriptor}`"),
        MemberKind::Method => format!("`{owner}`::`{name}{descriptor}`"),
    }
}

fn kind_mismatch(message: String) -> Diagnostic {
    Diagnostic {
        code: KIND_MISMATCH.to_string(),
        severity: DiagnosticSeverity::Error,
        message,
        provenance: None,
    }
}

/// The owner-kind rule of `invoke_interface`/`invoke_virtual` names the section it holds.
fn owner_kind_diagnostic(root: &ClassSite, use_kind: MemberUse) -> Diagnostic {
    let (requirement, section) = match use_kind {
        MemberUse::InvokeInterface => ("an interface owner", "5.4.3.4"),
        _ => ("a class owner", "5.4.3.3"),
    };
    kind_mismatch(format!(
        "the reference uses `{}`, which requires {requirement}, but its owner `{}` is {} \
         (JVMS {section})",
        use_kind.name(),
        escaped(&root.name.0),
        if root.is_interface {
            "an interface"
        } else {
            "a class"
        }
    ))
}

/// The Java 8 default conflict: several maximally-specific defaults are equally specific.
fn default_conflict_diagnostic(locations: &[MemberLocation]) -> Diagnostic {
    let names = locations
        .iter()
        .map(describe)
        .collect::<Vec<_>>()
        .join(", ");
    Diagnostic {
        code: DEFAULT_CONFLICT.to_string(),
        severity: DiagnosticSeverity::Error,
        message: format!(
            "the maximally specific superinterface methods of this reference include more than \
             one non-abstract declaration ({names}); Java 8 does not decide which one a call site \
             invokes, so the reference is incompatible with the hierarchy it names"
        ),
        provenance: None,
    }
}

fn access_denied_diagnostic(location: &MemberLocation, caller_class: &JvmBytes) -> Diagnostic {
    let rule = if location.access_flags & ACC_PRIVATE != 0 {
        "is private and is not declared in the caller's class"
    } else if location.access_flags & ACC_PROTECTED != 0 {
        "is protected and the caller's class is neither the declaring class, nor in its \
         run-time package, nor a subtype of it"
    } else {
        "is package private and the caller's class is in another run-time package"
    };
    Diagnostic {
        code: ACCESS_DENIED.to_string(),
        severity: DiagnosticSeverity::Error,
        message: format!(
            "{} {rule}: the caller's class is `{}` under loader `{}` (JVMS 5.4.4)",
            describe(location),
            escaped(&caller_class.0),
            location.loader.0
        ),
        provenance: None,
    }
}

fn access_not_checked_diagnostic(caller: &CallerContext) -> Diagnostic {
    let reason = if caller.enclosing.is_none() {
        "the request names no enclosing method"
    } else {
        "the class that declares the enclosing method could not be read completely"
    };
    Diagnostic {
        code: ACCESS_NOT_CHECKED.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: format!(
            "the access rules (JVMS 5.4.4) were not applied to this resolution: {reason}, so the \
             caller's class is unknown"
        ),
        provenance: None,
    }
}

/// The note of a declaration that resolved although it is abstract.
///
/// Resolution succeeds (JVMS resolves the declaration and the failure belongs to the
/// invocation), so the state stays `Resolved` and this warning carries the fact.
fn abstract_note(location: &MemberLocation) -> Diagnostic {
    Diagnostic {
        code: METHOD_IS_ABSTRACT.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: format!(
            "{} is abstract: the declaration resolves, and an invocation of it fails at \
             invocation time (JVMS 5.4.3.3)",
            describe(location)
        ),
        provenance: None,
    }
}

/// The note of a signature-polymorphic resolution, which shows both descriptors.
fn signature_polymorphic_note(location: &MemberLocation, call_site: &[u8]) -> Diagnostic {
    Diagnostic {
        code: SIGNATURE_POLYMORPHIC.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: format!(
            "{} is signature-polymorphic (JVMS 2.9): the reference's descriptor `{}` belongs to \
             the call site and the declaration's `{}` to the method, so the two differ by rule",
            describe(location),
            escaped(call_site),
            escaped(&location.descriptor.0)
        ),
        provenance: None,
    }
}

fn array_owner_diagnostic(owner: &[u8]) -> Diagnostic {
    Diagnostic {
        code: ARRAY_OWNER.to_string(),
        severity: DiagnosticSeverity::Error,
        message: format!(
            "the owner `{}` is an array type; this slice resolves no member of an array type and \
             fabricates no `java/lang/Object` declaration for it",
            escaped(owner)
        ),
        provenance: None,
    }
}

fn hierarchy_warning(code: &str, loader: &LoaderId, name: &JvmBytes, reason: &str) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: format!(
            "loader `{}`: `{}` is one step of the member search but {reason}; that branch stays \
             unsearched and this resolution does not cover the whole hierarchy",
            loader.0,
            escaped(&name.0)
        ),
        provenance: None,
    }
}

fn cycle_warning(loader: &LoaderId, path: &[JvmBytes], repeated: &JvmBytes) -> Diagnostic {
    let mut chain = path.iter().map(|name| escaped(&name.0)).collect::<Vec<_>>();
    chain.push(escaped(&repeated.0));
    Diagnostic {
        code: HIERARCHY_CYCLE.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: format!(
            "loader `{}`: `{}` is its own supertype ({}); a cyclic hierarchy is illegal, so this \
             branch stops here",
            loader.0,
            escaped(&repeated.0),
            chain.join(" -> ")
        ),
        provenance: None,
    }
}
