//! How an instance comes to be (P3 2.3, rules `new@1` and `init@1`).
//!
//! Two shapes, one subject — the construction of an object — and they are read from two different
//! places, which is why they are two rules rather than one:
//!
//! * `new@1` reads a **use site**: the allocation, its copy and its constructor call, which a
//!   compiler writes as one run of instructions and which this layer presents as the one
//!   `new Type(args…)` expression the source had. That is what makes a local, anonymous or inner
//!   class's *use* presentable at all: such a class is an ordinary class whose name the pool spells
//!   the way the compiler minted it (`p/Outer$1`, `p/Outer$1$1`, `p/Outer$2`), and the instantiation
//!   of it is this shape. **What that does not claim**: the nesting relation itself. `InnerClasses`,
//!   `EnclosingMethod` and the enclosing-instance argument's *meaning* are class-level facts, and
//!   the payload of one body holds none of them (§3 of the slice notes); the enclosing instance is
//!   written as the value the site really read — a local, or the receiver — and no `Outer.this`
//!   syntax, no `new Inner()` spelling inside an outer class and no synthetic field is invented. A
//!   body whose bytes need that syntax to be readable is quoted, not guessed at.
//! * `init@1` reads the **prologue of a constructor body**: the call an instance initializer makes
//!   on its own uninitialized `this` before anything else runs. It is written `super(…)` or
//!   `this(…)`, and which of the two is not a convention: the receiver is the frames'
//!   `UninitializedThis` — a token only a constructor's own `this` has before its constructor call —
//!   and JVMS 4.9.2 (which the frame pass already enforces when it converts that token) lets such a
//!   call name exactly the class that declares the constructor or that class's direct superclass.
//!   So `this(…)` is the call whose owner **is** the declaring class, `super(…)` is the one whose
//!   owner is not, and the run has to be told which class declares the body: a run that was not is
//!   refused, with that fact named, rather than being written one of the two ways.
//!
//! # What must not move
//!
//! Instance initializers run **after** the constructor call and in source order (JLS 12.5) — which
//! is the order the compiler already wrote into the body, so this layer's whole obligation is not to
//! disturb it: both rules write their text exactly where their instruction runs, and the statements
//! come out in BCI order. The one shape that puts a field write *before* the prologue is the
//! synthetic reference an inner class keeps to its enclosing instance, which JVMS 4.10.1.9 allows
//! and which stays where the bytecode put it ([`crate::field`] presents it under its own proof).

use std::collections::BTreeSet;

use jarde_jvm::method_ir::{Definition, RefType, SsaInstruction, SsaTable, Value, ValueId};
use jarde_reader::classfile::MethodCodeFacts;
use serde::Serialize;

use crate::ast::ConstructorTarget;
use crate::build::stack_operands;
use crate::decode::Operations;
use crate::evidence::Publication;
use crate::facts::{MethodFacts, Operation};
use crate::field;
use crate::pass::{INIT, NEW, Precondition, RuleVersion};
use crate::refusal::{Gap, Refusal};
use crate::report::ProvedMemberInnerTarget;

/// The pass answerable for a construction site.
pub(crate) const NEW_RULE: RuleVersion = NEW.rule();

/// The pass answerable for a constructor's prologue.
pub(crate) const INIT_RULE: RuleVersion = INIT.rule();

/// The requirement the prologue states: which class declares this constructor.
const DECLARING_CLASS: Precondition = Precondition::Metadata {
    attribute: "declaring_class",
};

// ---------------------------------------------------------------------------
// `new@1`: the allocation, its copy and its constructor call
// ---------------------------------------------------------------------------

/// One verified construction site.
pub(crate) struct Site {
    /// The BCI of the `new` that allocates.
    pub(crate) head: u32,
    /// The BCI of the `dup` that copies the instance.
    pub(crate) dup: u32,
    /// The BCI of the constructor call.
    pub(crate) constructor: u32,
    /// The class the allocation builds, in internal form, as the `new`'s own pool entry states it.
    pub(crate) class: String,
    /// The BCIs of the values the constructor call takes, in the order it reads them: the evidence
    /// of the argument order, and the anchors the written expression keeps.
    pub(crate) arguments: Vec<u32>,
    /// Call-site proof for a selected member target. Projection is enabled by the AST slice.
    pub(crate) member_inner: Option<MemberInnerSite>,
    /// Every BCI the site owns: the allocation, the copy and the constructor call. An owned
    /// instruction produces no statement of its own — its text is the `new` expression, written
    /// where the instance is consumed and nowhere else.
    pub(crate) owned: BTreeSet<u32>,
}

/// The local qualifier and exact check already proved for one physical member constructor.
pub(crate) struct MemberInnerSite {
    pub(crate) qualifier: u32,
    pub(crate) check: u32,
    pub(crate) pop: u32,
    pub(crate) outer: String,
    pub(crate) simple_name: String,
    pub(crate) generic_diamond: bool,
}

struct MemberProof {
    site: MemberInnerSite,
    arguments: Vec<u32>,
    owned: BTreeSet<u32>,
}

/// The already-read facts shared by ordinary and member construction verification.
struct ConstructionFacts<'a> {
    ssa: &'a SsaTable,
    operations: &'a Operations,
    fields: &'a field::Plan,
    member_targets: &'a [ProvedMemberInnerTarget],
    code: &'a MethodCodeFacts,
}

/// Every construction site of one body, the candidates that were not sites, and the gaps stated in
/// every selection.
///
/// The plan holds the **decisions**: every verified site (which the builder writes) and every refused
/// candidate with the link that failed. [`Self::materialize`] writes the owning [`NewRecord`]s from
/// them after the artifact is committed, one record per charge and only when the request selected
/// `RuleDetails`.
pub(crate) struct Sites {
    sites: Vec<Site>,
    owned: BTreeSet<u32>,
    /// Every decoded `new` allocation in the body, including candidates reserved by another rule.
    /// A false `verified` value is evidence against a class-level unique allocation claim.
    allocation_candidates: Vec<AllocationCandidate>,
    /// Why a candidate was not a construction site, in BCI order: the internal decision every
    /// selection reports as a gap beside the records only a selected run materializes.
    refusals: Vec<Refused>,
}

/// The bounded census of allocation instructions that `new@1` considered or another rule owned.
pub(crate) struct AllocationCandidate {
    pub(crate) head: u32,
    pub(crate) class: String,
    pub(crate) verified: bool,
}

/// One candidate construction this rule read and did not present.
struct Refused {
    /// The BCI of the allocation the refused candidate starts at.
    head: u32,
    /// The class the allocation named, in internal form.
    class: String,
    /// Which link of the verification failed.
    refusal: Refusal,
}

/// One verdict of this rule's plan: the site it verified, or the candidate it refused.
enum Decision<'a> {
    Presented(&'a Site),
    Refused(&'a Refused),
}

impl Decision<'_> {
    fn head(&self) -> u32 {
        match self {
            Self::Presented(site) => site.head,
            Self::Refused(refused) => refused.head,
        }
    }

    /// Every driver BCI this decision states for itself: a site's allocation, its copy, its
    /// constructor and every argument, and a refused candidate's own allocation.
    fn positions(&self) -> Vec<u32> {
        match self {
            Self::Presented(site) => site_positions(site),
            Self::Refused(refused) => vec![refused.head],
        }
    }

    /// The owning record, built here and only here.
    fn record(self) -> NewRecord {
        crate::demand_counts::record_built(crate::evidence::RecoveryEvidenceKind::RuleDetails);
        match self {
            Self::Presented(site) => NewRecord::of_site(site),
            Self::Refused(refused) => NewRecord::of_candidate(
                refused.head,
                &refused.class,
                NewRefusal::of(&refused.refusal, refused.head),
            ),
        }
    }
}

impl Sites {
    /// An empty plan: a body that allocates nothing.
    pub(crate) fn empty() -> Self {
        Self {
            sites: Vec::new(),
            owned: BTreeSet::new(),
            allocation_candidates: Vec::new(),
            refusals: Vec::new(),
        }
    }

    /// Whether one instruction belongs to a verified site and so produces no statement of its own.
    pub(crate) fn owns(&self, bci: u32) -> bool {
        self.owned.contains(&bci)
    }

    /// The site one instruction belongs to, when one of them produces the value it wrote.
    pub(crate) fn site_of(&self, bci: u32) -> Option<&Site> {
        self.sites.iter().find(|site| site.owned.contains(&bci))
    }

    /// Every allocation opcode in this body, in BCI order, whether `new@1` accepted it or another
    /// rule reserved it. Class-level uniqueness proofs must count all of them.
    pub(crate) fn allocation_candidates(&self) -> &[AllocationCandidate] {
        &self.allocation_candidates
    }

    /// The accepted site whose allocation begins at `head`, if `new@1` verified it.
    pub(crate) fn site_at_head(&self, head: u32) -> Option<&Site> {
        self.sites.iter().find(|site| site.head == head)
    }

    /// Every candidate the rule refused, in BCI order.
    pub(crate) fn refusals(&self) -> impl Iterator<Item = Gap> + '_ {
        self.refusals.iter().map(|refused| {
            let refusal = NewRefusal::of(&refused.refusal, refused.head);
            Gap::at(refusal.code, refused.head, refusal.message)
        })
    }

    /// Whether this rule decided anything about this body.
    pub(crate) fn answered(&self) -> bool {
        !self.sites.is_empty() || !self.refusals.is_empty()
    }

    /// How many candidates the rule read, and how many of them it presented as sites.
    pub(crate) fn counts(&self) -> (u64, u64) {
        let presented = u64::try_from(self.sites.len()).unwrap_or(u64::MAX);
        let refused = u64::try_from(self.refusals.len()).unwrap_or(u64::MAX);
        (presented + refused, presented)
    }

    /// The owning records this plan publishes under `publication`, in BCI order, within the phase's
    /// remaining allowance.
    pub(crate) fn materialize(
        &self,
        publication: Publication,
        phase: &mut crate::evidence::EvidencePhase,
        budget: &mut jarde_reader::budget::Budget,
    ) -> (Vec<NewRecord>, crate::evidence::Materialized) {
        let mut decisions: Vec<Decision<'_>> = self
            .sites
            .iter()
            .map(Decision::Presented)
            .chain(self.refusals.iter().map(Decision::Refused))
            .collect();
        decisions.sort_by_key(Decision::head);
        phase.materialize(
            budget,
            decisions
                .into_iter()
                .filter(|decision| publication.publishes(&decision.positions())),
            Decision::record,
        )
    }
}

/// Reads every construction site of one body.
///
/// `reserved` is every BCI another rule of this run already owns — in practice the concatenation
/// chains, whose own allocation and constructor call are written inside a `+` expression and must not
/// be written a second time as a `new`. One instruction is never two shapes.
///
/// `fields` is the `field@1` plan of the same body, which this rule **reads** and never re-derives: a
/// claimed field access is one of the places a construction's instance is written into (P3 2c.26), and
/// whether an instruction really is an access to the member its own receiver's type declares is that
/// rule's verdict. `@field` is therefore decided before `@new` — [`crate::report`] runs the two in
/// that order — and the judgement is handed in rather than taken a second time here.
pub(crate) fn sites(
    ssa: &SsaTable,
    operations: &Operations,
    reserved: &BTreeSet<u32>,
    fields: &field::Plan,
    member_targets: &[ProvedMemberInnerTarget],
    code: &MethodCodeFacts,
) -> Sites {
    let facts = ConstructionFacts {
        ssa,
        operations,
        fields,
        member_targets,
        code,
    };
    let blocks: Vec<&[SsaInstruction]> = ssa
        .blocks()
        .iter()
        .map(|block| block.instructions())
        .collect();
    let mut plan = Sites::empty();
    for block in &blocks {
        for (index, instruction) in block.iter().enumerate() {
            let head = instruction.bci();
            let Some(Operation::Allocate { ty }) = operations.get(head) else {
                continue;
            };
            let candidate_index = plan.allocation_candidates.len();
            plan.allocation_candidates.push(AllocationCandidate {
                head,
                class: ty.clone(),
                verified: false,
            });
            if reserved.contains(&head) {
                // Another rule of this run writes this allocation's text: it is not a second shape.
                continue;
            }
            let ty = ty.clone();
            match verify(head, index, block, ty.clone(), &facts) {
                Ok(site) => {
                    plan.allocation_candidates[candidate_index].verified = true;
                    for bci in &site.owned {
                        plan.owned.insert(*bci);
                    }
                    plan.sites.push(site);
                }
                Err(refusal) => plan.refusals.push(Refused {
                    head,
                    class: ty,
                    refusal,
                }),
            }
        }
    }
    plan.allocation_candidates
        .sort_by_key(|candidate| candidate.head);
    plan.refusals.sort_by_key(|refused| refused.head);
    plan
}

/// Every driver BCI one construction site states: the allocation, its copy, the constructor and
/// every argument.
fn site_positions(site: &Site) -> Vec<u32> {
    let mut positions = vec![site.head, site.dup, site.constructor];
    positions.extend(site.arguments.iter().copied());
    if let Some(member) = &site.member_inner {
        positions.extend([member.qualifier, member.check, member.pop]);
    }
    positions
}

/// Verifies one candidate construction site, or states the link that failed.
///
/// `fields` is the `field@1` plan of this body: the one fact consulted here that this rule does not
/// decide for itself, and only for the question of whether an instruction is a place the instance is
/// written (P3 2c.26).
fn verify(
    head: u32,
    index: usize,
    block: &[SsaInstruction],
    ty: String,
    facts: &ConstructionFacts<'_>,
) -> Result<Site, Refusal> {
    let ConstructionFacts {
        ssa,
        operations,
        fields,
        member_targets,
        ..
    } = *facts;
    let shape = |detail: String| Refusal::shape("jre_new_shape", detail);
    let Some(dup) = block.get(index + 1) else {
        return Err(shape(format!(
            "the allocation at BCI {head} ends its block: a construction continues with the `dup` of the instance"
        )));
    };
    if operations.get(dup.bci()) != Some(&Operation::Duplicate) {
        return Err(shape(format!(
            "the instruction after the allocation at BCI {head} is at BCI {}, and a construction continues with the `dup` of the instance it allocated",
            dup.bci()
        )));
    }
    let Some(constructor) = block.iter().skip(index + 2).find(|instruction| {
        matches!(
            operations.get(instruction.bci()),
            Some(Operation::Invoke(target)) if target.owner() == ty && target.name() == "<init>"
        )
    }) else {
        return Err(shape(format!(
            "the allocation at BCI {head} is never constructed: no `{ty}.<init>(…)` call reads the instance it builds and the copy of it"
        )));
    };
    let at = constructor.bci();
    // The instructions the instance comes from: the allocation, its copy and the constructor that
    // initialized it. A compiler may put the stored value down as any of the three, and what matters
    // is that the value really belongs to *this* allocation.
    let produced_by: Vec<u32> = vec![head, dup.bci(), at];
    let operands = stack_operands(constructor);
    let Some((_, receiver)) = operands.first().copied() else {
        return Err(shape(format!(
            "the constructor call at BCI {at} reads no receiver this run states"
        )));
    };
    if !is_the_instance(ssa, receiver, &produced_by) {
        return Err(shape(format!(
            "the constructor at BCI {at} is called on a value this allocation did not produce"
        )));
    }
    let member = member_targets
        .iter()
        .find(|target| {
            target.owner == ty
                && matches!(
                    operations.get(at),
                    Some(Operation::Invoke(call)) if call.descriptor() == target.constructor_descriptor
                )
        })
        .map(|target| verify_member(index, block, constructor, &operands, facts, target))
        .transpose()?;
    let arguments = if let Some(member) = &member {
        member.arguments.clone()
    } else {
        // Every argument has to be produced **between the copy and the call**, so that writing it as an
        // argument of the `new` expression evaluates it exactly where the bytecode evaluated it. A value
        // produced elsewhere would move, and this rule never moves a value.
        let mut arguments: Vec<u32> = Vec::new();
        for (_, value) in operands.iter().skip(1) {
            let Some(produced) = produced_at(ssa, *value) else {
                return Err(shape(format!(
                    "an argument of the constructor call at BCI {at} was not produced by an instruction of this body, so where it is evaluated is not stated"
                )));
            };
            if produced <= dup.bci() || produced >= at {
                return Err(shape(format!(
                    "the argument produced at BCI {produced} is not produced between the `dup` at BCI {} and the constructor call at BCI {at}: writing it as an argument of the `new` expression would evaluate it in a different order",
                    dup.bci()
                )));
            }
            arguments.push(produced);
        }
        // Trace the constructor's physical arguments back through this block's SSA definitions and
        // reads. An invocation is part of the expression only when that actual dependency walk reaches
        // it; a result consumed by unrelated bytecode is not enough, and a void invocation cannot be
        // reached at all.
        let argument_dependencies =
            value_dependency_bcis(ssa, block, operands.iter().skip(1).map(|(_, value)| *value));
        // Nothing inside the span may be an effect this rule would have to move: every invocation
        // between the copy and the call must be in an argument's own value dependency chain.
        for instruction in block.iter().skip(index + 2) {
            if instruction.bci() >= at {
                break;
            }
            match operations.get(instruction.bci()) {
                Some(
                    Operation::Push(_)
                    | Operation::Load { .. }
                    | Operation::Arithmetic { .. }
                    | Operation::Negate,
                ) => {}
                Some(Operation::Invoke(_))
                    if argument_dependencies.contains(&instruction.bci()) => {}
                Some(Operation::Invoke(_)) => {
                    return Err(Refusal::unmet(
                        &NEW,
                        Precondition::StatementFree,
                        format!(
                            "the invocation at BCI {} is not a value dependency of the constructor's physical arguments at BCI {at}, so presenting the construction would move that call effect",
                            instruction.bci()
                        ),
                    ));
                }
                Some(operation) => {
                    return Err(Refusal::unmet(
                        &NEW,
                        Precondition::StatementFree,
                        format!(
                            "the instruction at BCI {} is an {operation:?} between the allocation's copy and its constructor call, and presenting the construction would write that effect somewhere else",
                            instruction.bci()
                        ),
                    ));
                }
                None => {
                    return Err(shape(format!(
                        "the instruction at BCI {} was not decoded by this run, so what it does inside the construction is not stated",
                        instruction.bci()
                    )));
                }
            }
        }
        arguments
    };
    // The instance has to be read by **exactly one** instruction this build **writes it into**. A
    // construction reads no value of its own, so a value nothing writes has no place in the body: the
    // instructions that do read it are quoted instead, and the allocation, its copy and its
    // constructor call are quoted with them — never written as a `new` expression that no statement
    // holds.
    //
    // The instance is written **once**, too, and that is what the count below states (P3 2c.26/2c.27):
    // a site is one `new` expression in one place, so a leftover that more than one instruction reads
    // would have to be spelled twice — two instances where the bytecode allocated one — and such a
    // candidate keeps its refusal. The constructor's own copy is not a reader here: it is one of the
    // three instructions this site owns, and [`outside_readers`] leaves the site's own instructions
    // out. The place the value is written is the reader's own text: a store, a call, a `return`, a
    // test and a *claimed* field access ([`renders_its_reads`]) all write the value they read, and
    // every other instruction is quoted as bytecode and writes nothing.
    let readers = outside_readers(ssa, &produced_by);
    let written: Vec<u32> = readers
        .iter()
        .copied()
        .filter(|bci| renders_its_reads(operations, fields, *bci))
        .collect();
    if written.is_empty() {
        return Err(shape(if readers.is_empty() {
            format!(
                "nothing in this method reads the instance the allocation at BCI {head} builds, so the construction has no place in the body"
            )
        } else {
            format!(
                "the instance the allocation at BCI {head} builds is read only by instructions this build quotes (BCIs {}), so the construction has no place in the body",
                readers
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }));
    }
    if readers.len() != 1 {
        return Err(shape(format!(
            "the instance the allocation at BCI {head} builds is read by the instructions at BCIs {}, and a construction is written as one `new` expression in one place: a leftover that more than one instruction reads has no single Java spelling",
            readers
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    let mut owned: BTreeSet<u32> = produced_by.iter().copied().collect();
    if let Some(member) = &member {
        owned.extend(member.owned.iter().copied());
    }
    Ok(Site {
        head,
        dup: dup.bci(),
        constructor: at,
        class: ty,
        arguments,
        member_inner: member.map(|proof| proof.site),
        owned,
    })
}

/// The restricted javac member shape. The target's declaration is already proved by class-source;
/// this checks only this call's values and effects. In particular, the descriptor's first type is
/// not evidence for the qualifier's static type or identity.
fn verify_member(
    index: usize,
    block: &[SsaInstruction],
    constructor: &SsaInstruction,
    operands: &[(jarde_jvm::method_ir::Slot, ValueId)],
    facts: &ConstructionFacts<'_>,
    target: &ProvedMemberInnerTarget,
) -> Result<MemberProof, Refusal> {
    let ConstructionFacts {
        ssa,
        operations,
        code,
        ..
    } = *facts;
    let at = constructor.bci();
    let shape = |detail: String| Refusal::shape("jre_new_member_shape", detail);
    let order = |detail: String| Refusal::shape("jre_new_member_order", detail);
    let Some((_, physical_outer)) = operands.get(1).copied() else {
        return Err(shape(format!(
            "the member constructor at BCI {at} has no physical outer argument"
        )));
    };
    let Some([qualifier, copy, check, pop]) = block.get(index + 2..index + 6) else {
        return Err(shape(format!(
            "the member constructor at BCI {at} has no complete qualifier check"
        )));
    };
    if pop.bci() >= at
        || !matches!(
            operations.get(qualifier.bci()),
            Some(Operation::Load { .. })
        )
        || operations.get(copy.bci()) != Some(&Operation::Duplicate)
        || !matches!(operations.get(check.bci()), Some(Operation::Invoke(call))
            if call.kind() == crate::facts::InvokeKind::Static
                && !call.is_interface_reference()
                && call.owner() == "java/util/Objects"
                && call.name() == "requireNonNull"
                && call.descriptor() == "(Ljava/lang/Object;)Ljava/lang/Object;")
        || pop.opcode() != 0x57
    {
        return Err(shape(format!(
            "the member constructor at BCI {at} lacks the contiguous local load, dup, exact requireNonNull(Object), pop check"
        )));
    }
    let qualifier_writes = qualifier.writes();
    let copy_reads = stack_operands(copy);
    let copy_writes = copy.writes();
    let check_reads = stack_operands(check);
    let check_writes = check.writes();
    let pop_reads = stack_operands(pop);
    if qualifier_writes.len() != 1
        || copy_reads.len() != 1
        || copy_writes.len() != 2
        || check_reads.len() != 1
        || check_writes.len() != 1
        || pop_reads.len() != 1
        || copy_reads[0].1 != qualifier_writes[0].1
        || !copy_writes
            .iter()
            .any(|(_, value)| *value == physical_outer)
        || !copy_writes
            .iter()
            .any(|(_, value)| *value == check_reads[0].1)
        || physical_outer == check_reads[0].1
        || pop_reads[0].1 != check_writes[0].1
        || !single_use_at(ssa, qualifier_writes[0].1, copy.bci())
        || !single_use_at(ssa, physical_outer, at)
        || !single_use_at(ssa, check_reads[0].1, check.bci())
        || !single_use_at(ssa, check_writes[0].1, pop.bci())
    {
        return Err(shape(format!(
            "the member constructor at BCI {at} does not pass the checked qualifier's two SSA copies as its physical outer and null-check operand exactly once"
        )));
    }
    let outer_descriptor = format!("L{};", target.outer);
    if !matches!(ssa.value(qualifier_writes[0].1).ty(),
        Value::Ref(RefType::Named { name, .. }) if name == outer_descriptor.as_bytes())
    {
        return Err(shape(format!(
            "the qualifier at BCI {} has no exact static type `{}` for member binding (SSA type {:?})",
            qualifier.bci(),
            target.outer,
            ssa.value(qualifier_writes[0].1).ty()
        )));
    }

    // The Java qualified expression puts its check at this position. Every instruction it owns
    // must have the same handler coverage as the original check; a boundary cannot be crossed.
    let coverage = |bci: u32| -> Vec<u32> {
        code.exception_handlers
            .iter()
            .filter(|handler| handler.start_bci <= bci && bci < handler.end_bci)
            .map(|handler| handler.ordinal)
            .collect()
    };
    let expected_handlers = coverage(check.bci());
    if block[index..]
        .iter()
        .take_while(|instruction| instruction.bci() <= at)
        .any(|instruction| coverage(instruction.bci()) != expected_handlers)
    {
        return Err(order(format!(
            "the member constructor at BCI {at} crosses an exception-handler boundary around its qualifier check"
        )));
    }

    let mut arguments = vec![copy.bci()]; // physical first argument, not a source argument
    let mut last = pop.bci();
    for (_, value) in operands.iter().skip(2) {
        let Some(produced) = produced_at(ssa, *value) else {
            return Err(order(format!(
                "an ordinary argument of the member constructor at BCI {at} has no local producer"
            )));
        };
        if produced <= last || produced >= at {
            return Err(order(format!(
                "the ordinary argument at BCI {produced} is not produced in order after the null-check at BCI {} and before the constructor at BCI {at}",
                pop.bci()
            )));
        }
        arguments.push(produced);
        last = produced;
    }
    let dependencies =
        value_dependency_bcis(ssa, block, operands.iter().skip(2).map(|(_, value)| *value));
    let after_check = index + 6;
    for instruction in block
        .iter()
        .skip(after_check)
        .take_while(|instruction| instruction.bci() < at)
    {
        let bci = instruction.bci();
        if !dependencies.contains(&bci) {
            return Err(order(format!(
                "the instruction at BCI {bci} is not an ordinary argument dependency of the member constructor at BCI {at}"
            )));
        }
        if !matches!(
            operations.get(bci),
            Some(
                Operation::Push(_)
                    | Operation::Load { .. }
                    | Operation::Arithmetic { .. }
                    | Operation::Negate
                    | Operation::Invoke(_)
            )
        ) {
            return Err(order(format!(
                "the instruction at BCI {bci} cannot be kept in member argument order"
            )));
        }
    }
    let owned = [copy.bci(), check.bci(), pop.bci()].into_iter().collect();
    Ok(MemberProof {
        site: MemberInnerSite {
            qualifier: qualifier.bci(),
            check: check.bci(),
            pop: pop.bci(),
            outer: target.outer.clone(),
            simple_name: target.simple_name.clone(),
            generic_diamond: target.generic_diamond,
        },
        arguments,
        owned,
    })
}

fn single_use_at(ssa: &SsaTable, value: ValueId, at: u32) -> bool {
    let uses = ssa.value(value).uses();
    uses.len() == 1 && uses[0].bci() == Some(at)
}

/// Every instruction outside a site that reads one of the values the site produced.
///
/// The walk is the body's own instructions, in block order: the site's own three instructions are
/// left out by BCI, and everything else is read from the table the caller already holds.
fn outside_readers(ssa: &SsaTable, produced_by: &[u32]) -> Vec<u32> {
    let mut readers: Vec<u32> = Vec::new();
    for instruction in ssa.blocks().iter().flat_map(|block| block.instructions()) {
        if produced_by.contains(&instruction.bci()) {
            continue;
        }
        if instruction.reads().iter().any(|(_, read)| {
            matches!(ssa.value(*read).def(), Definition::Instruction { bci, .. } if produced_by.contains(bci))
        }) && !readers.contains(&instruction.bci())
        {
            readers.push(instruction.bci());
        }
    }
    readers.sort_unstable();
    readers
}

/// Whether the instruction at one BCI writes the values it reads into the text this build produces.
///
/// The predicate is the one [`crate::build`] applies to a call's reader (P3 2.3 §0), stated here
/// because this rule decides before the builder does: a store, a call, a `return`, a condition, a
/// switch, an arithmetic and a dynamic site write the values they read; a `dup` copies a value and
/// writes nothing, and an instruction a rule of this build does not claim is quoted.
///
/// A **field access** is exactly [`crate::field::Plan::owns`]: a claimed write is the assignment
/// `field@1` writes, and a claimed read is the expression whose receiver or whose value the access
/// is — either way the instance is written into the access's own text (P3 2c.26). An access `@field`
/// refused is quoted as bytecode and writes nothing, so the site must not lean on it: that is why the
/// plan is an input of this rule and the verdict is read rather than guessed from the opcode, and why
/// `@field` is decided before `@new` ([`crate::report`] orders the two plans that way).
///
/// Array reads and casts are deliberately not here: whether *their* rules claim them is decided after
/// this plan, and a site that leaned on one would be leaning on a verdict not yet taken.
fn renders_its_reads(operations: &Operations, fields: &field::Plan, bci: u32) -> bool {
    match operations.get(bci) {
        Some(
            Operation::Store { .. }
            | Operation::Invoke(_)
            | Operation::InvokeDynamic(_)
            | Operation::Return
            | Operation::Throw
            | Operation::Comparison { .. }
            | Operation::Switch { .. }
            | Operation::Arithmetic { .. }
            | Operation::Negate,
        ) => true,
        Some(Operation::Field { .. }) => fields.owns(bci),
        _ => false,
    }
}

/// Whether one value is the instance a candidate site builds: it was produced by the allocation, by
/// the copy of it or by the constructor that initialized it.
fn is_the_instance(ssa: &SsaTable, value: ValueId, produced_by: &[u32]) -> bool {
    match ssa.value(value).def() {
        Definition::Instruction { bci, .. } => produced_by.contains(bci),
        _ => false,
    }
}

/// The BCI the instruction that produced one value sits at, when an instruction produced it.
fn produced_at(ssa: &SsaTable, value: ValueId) -> Option<u32> {
    match ssa.value(value).def() {
        Definition::Instruction { bci, .. } => Some(*bci),
        _ => None,
    }
}

/// The current block's instruction definitions reachable from physical argument values.
///
/// A value dependency follows both edges the SSA table states: an instruction definition leads to
/// that instruction's reads, and a phi definition leads to its incoming values. Entry definitions
/// have no instruction in this block to follow. The visited set makes loop-carried phi inputs and
/// repeated reads finite without relying on instruction order.
fn value_dependency_bcis(
    ssa: &SsaTable,
    block: &[SsaInstruction],
    roots: impl IntoIterator<Item = ValueId>,
) -> BTreeSet<u32> {
    let instructions: std::collections::BTreeMap<u32, &SsaInstruction> = block
        .iter()
        .map(|instruction| (instruction.bci(), instruction))
        .collect();
    let mut pending: Vec<ValueId> = roots.into_iter().collect();
    let mut visited = BTreeSet::new();
    let mut dependencies = BTreeSet::new();

    while let Some(value) = pending.pop() {
        if !visited.insert(value) {
            continue;
        }
        match ssa.value(value).def() {
            Definition::Instruction { bci, .. } => {
                if let Some(instruction) = instructions.get(bci) {
                    dependencies.insert(*bci);
                    pending.extend(instruction.reads().iter().map(|(_, read)| *read));
                }
            }
            Definition::Phi { .. } => {
                if let Some(phi) = ssa.phis().iter().find(|phi| phi.value() == value) {
                    for input in phi.inputs() {
                        if let jarde_jvm::method_ir::PhiInput::Value(input) = input {
                            pending.push(*input);
                        }
                    }
                }
            }
            Definition::Entry { .. } | Definition::Caught { .. } => {}
        }
    }
    dependencies
}

/// What one candidate construction site was presented as, or why it was not (P3 2.3, `new@1`).
///
/// The record states what the pattern's acceptance asks: the allocation, the class the pool named for
/// it, the constructor call and every argument with the BCI that produced it — so the *order* the
/// arguments are written in is checkable against the bytecode without reading the text.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NewRecord {
    /// The BCI of the allocation the site was claimed to start at.
    pub head: u32,
    /// The BCI of the `dup` that copied the instance, when the walk reached one.
    pub dup: Option<u32>,
    /// The BCI of the constructor call, when the walk reached one.
    pub constructor: Option<u32>,
    /// The class the allocation builds, in internal form.
    pub class: String,
    /// The BCI that produced each argument, in the order the constructor reads them.
    pub arguments: Vec<u32>,
    /// Whether the site was presented as a `new` expression.
    pub presented: bool,
    /// Why it was not, when it was not.
    pub refusal: Option<NewRefusal>,
}

impl NewRecord {
    /// Whether this candidate was presented.
    pub fn presented(&self) -> bool {
        self.presented
    }

    /// The rule this record is answerable to.
    pub fn rule(&self) -> RuleVersion {
        NEW_RULE
    }

    fn of_site(site: &Site) -> Self {
        Self {
            head: site.head,
            dup: Some(site.dup),
            constructor: Some(site.constructor),
            class: site.class.clone(),
            arguments: site.arguments.clone(),
            presented: true,
            refusal: None,
        }
    }

    fn of_candidate(head: u32, class: &str, refusal: NewRefusal) -> Self {
        Self {
            head,
            dup: None,
            constructor: None,
            class: class.to_string(),
            arguments: Vec::new(),
            presented: false,
            refusal: Some(refusal),
        }
    }
}

/// Why one candidate construction site was not presented.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NewRefusal {
    /// The diagnostic code, one `jre_new_*` per link of the verification that can fail.
    pub code: &'static str,
    /// The rule that refused the site.
    pub rule: RuleVersion,
    /// The declared requirement that fell short, when the refusal is one of the rule's own
    /// preconditions.
    pub requirement: Option<String>,
    /// One sentence stating which link failed, with the allocation's BCI in it.
    pub message: String,
}

impl NewRefusal {
    fn of(refusal: &Refusal, head: u32) -> Self {
        Self {
            code: refusal.code(),
            rule: NEW_RULE,
            requirement: refusal.requirement().map(Precondition::describe),
            message: format!(
                "the construction at BCI {head} was not presented: {}",
                refusal.message()
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// `init@1`: the prologue of a constructor
// ---------------------------------------------------------------------------

/// One verified constructor prologue.
pub(crate) struct Prologue {
    /// The BCI of the constructor call the prologue makes.
    pub(crate) bci: u32,
    /// Which constructor it calls.
    pub(crate) target: ConstructorTarget,
    /// The class the call names, in internal form.
    pub(crate) class: String,
    /// The class that declares this constructor, as the caller stated it.
    pub(crate) declared: String,
}

/// The prologue of one body, when it has one, and the record either way.
///
/// The plan holds the *decision* — the verified prologue, the call the rule refused to spell, and
/// the verdict of the whole-body case — and [`Self::materialize`] writes the owning [`InitRecord`]
/// from it after the artifact is committed, only when the request selected `RuleDetails`.
pub(crate) struct Prologues {
    prologue: Option<Prologue>,
    /// The call the rule refused to spell, with the requirement it fell short of: the builder quotes
    /// exactly that instruction rather than writing it as whatever the generic arms would make of a
    /// constructor call on an uninitialized `this` (P3 2.3).
    refused: Option<(u32, Refusal)>,
    /// What this rule decided about this body: the plan side of the record, whatever the selection.
    verdict: Verdict,
}

/// What `init@1` decided about one body's prologue.
enum Verdict {
    /// The body is not an instance initializer: this rule claims nothing and refuses nothing, and
    /// materializes no record — it is not a verdict about these bytes.
    NotThisRule,
    /// An instance initializer whose body makes no constructor call on its own `this`. The refusal
    /// is about the whole body, so a driver range neither selects nor drops it.
    NoPrologue(Refusal),
    /// A constructor call the run cannot decide `this`/`super` for, because no declaring class was
    /// stated.
    Undecided {
        bci: u32,
        class: String,
        refusal: Refusal,
    },
    /// A decided prologue.
    Decided { class: String, declared: String },
}

impl Prologues {
    /// No prologue and nothing to record: the body is not an instance initializer.
    pub(crate) fn none() -> Self {
        Self {
            prologue: None,
            refused: None,
            verdict: Verdict::NotThisRule,
        }
    }

    /// Whether this rule decided anything about this body: the rule index is built from this answer,
    /// and it does not depend on the evidence selection or on a driver range.
    pub(crate) fn answered(&self) -> bool {
        !matches!(self.verdict, Verdict::NotThisRule)
    }

    /// The prologue the instruction at one BCI is, when it is the verified one.
    pub(crate) fn at(&self, bci: u32) -> Option<&Prologue> {
        self.prologue
            .as_ref()
            .filter(|prologue| prologue.bci == bci)
    }

    /// The requirement the rule refused the call at one BCI under, when it refused one there.
    pub(crate) fn refused_at(&self, bci: u32) -> Option<&Refusal> {
        self.refused
            .as_ref()
            .filter(|(refused, _)| *refused == bci)
            .map(|(_, refusal)| refusal)
    }

    /// The prologue the artifact writes, when this body has one.
    pub(crate) fn prologue(&self) -> Option<&Prologue> {
        self.prologue.as_ref()
    }

    /// Why the artifact writes no prologue, as the gap every selection carries.
    pub(crate) fn refusal(&self) -> Option<Gap> {
        match &self.verdict {
            Verdict::NotThisRule | Verdict::Decided { .. } => None,
            Verdict::NoPrologue(refusal) => {
                let shape = InitRefusal::of(refusal, None);
                Some(Gap::whole(shape.code, shape.message))
            }
            Verdict::Undecided { bci, refusal, .. } => {
                let shape = InitRefusal::of(refusal, Some(*bci));
                Some(Gap::at(shape.code, *bci, shape.message))
            }
        }
    }

    /// The prologue's own record, when the request selected rule records and the phase can pay for
    /// it: the one owning record of this rule, built after the artifact was committed.
    pub(crate) fn materialize(
        &self,
        publication: Publication,
        phase: &mut crate::evidence::EvidencePhase,
        budget: &mut jarde_reader::budget::Budget,
    ) -> (Option<InitRecord>, crate::evidence::Materialized) {
        let positions: Vec<u32> = match &self.verdict {
            Verdict::NotThisRule => Vec::new(),
            Verdict::NoPrologue(_) => Vec::new(),
            Verdict::Undecided { bci, .. } => vec![*bci],
            Verdict::Decided { .. } => self
                .prologue
                .as_ref()
                .map(|prologue| vec![prologue.bci])
                .unwrap_or_default(),
        };
        let selected = self.answered().then_some(()).filter(|()| {
            // The whole-body verdict states no position of its own and is delivered with its
            // category whatever range the request states; a positioned one follows the range.
            publication.publishes(&positions)
        });
        let (records, reached) = phase.materialize(budget, selected, |()| {
            crate::demand_counts::record_built(crate::evidence::RecoveryEvidenceKind::RuleDetails);
            self.record()
        });
        (records.into_iter().next(), reached)
    }

    /// The record one verdict publishes, built from the decision this plan holds.
    pub(crate) fn record(&self) -> InitRecord {
        match &self.verdict {
            Verdict::NotThisRule => InitRecord {
                bci: None,
                target: None,
                class: None,
                declared: None,
                presented: false,
                refusal: None,
            },
            Verdict::NoPrologue(refusal) => {
                InitRecord::of_refusal_shape(None, None, InitRefusal::of(refusal, None))
            }
            Verdict::Undecided {
                bci,
                class,
                refusal,
            } => InitRecord::of_refusal_shape(
                Some(*bci),
                Some(class.clone()),
                InitRefusal::of(refusal, Some(*bci)),
            ),
            Verdict::Decided { class, declared } => {
                let prologue = self
                    .prologue
                    .as_ref()
                    .expect("a decided prologue is the prologue this plan holds");
                InitRecord {
                    bci: Some(prologue.bci),
                    target: Some(prologue.target),
                    class: Some(class.clone()),
                    declared: Some(declared.clone()),
                    presented: true,
                    refusal: None,
                }
            }
        }
    }
}

/// Reads the prologue of one body.
///
/// The decision — presented, refused, or not this rule's body at all — is taken for every selection;
/// no owning record is built here. The verdict stays in [`Prologues`] and
/// [`Prologues::materialize`] writes the record from it after the artifact is committed.
pub(crate) fn prologue(ssa: &SsaTable, operations: &Operations, method: &MethodFacts) -> Prologues {
    if method.name() != "<init>" {
        // Not an instance initializer: nothing is claimed and nothing is refused.
        return Prologues::none();
    }
    let found = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| {
            let Some(Operation::Invoke(target)) = operations.get(instruction.bci()) else {
                return false;
            };
            target.name() == "<init>"
                && stack_operands(instruction)
                    .first()
                    .is_some_and(|(_, receiver)| {
                        matches!(ssa.value(*receiver).ty(), Value::UninitializedThis)
                    })
        });
    let Some(instruction) = found else {
        // The whole body is the refusal here, so it states no position of its own: the gap is
        // delivered whatever the selection says about positions.
        let refusal = Refusal::shape(
            "jre_init_no_prologue",
            "the body of an instance initializer makes no constructor call on its own uninitialized `this`, so no `super(…)` or `this(…)` can be read from it".to_string(),
        );
        return Prologues {
            prologue: None,
            refused: None,
            verdict: Verdict::NoPrologue(refusal),
        };
    };
    let bci = instruction.bci();
    let Some(Operation::Invoke(target)) = operations.get(bci) else {
        unreachable!("the instruction was found as an invocation");
    };
    let class = target.owner().to_string();
    let Some(declaring) = method.declaring_class() else {
        let refusal = Refusal::unmet(
            &INIT,
            DECLARING_CLASS,
            format!(
                "the constructor call at BCI {bci} names `{}`, and this run was not told which class declares this constructor: JVMS 4.9.2 lets an instance initializer call that class's own constructor or its direct superclass's, and the two are written differently",
                target.owner()
            ),
        );
        return Prologues {
            prologue: None,
            refused: Some((bci, refusal.clone())),
            verdict: Verdict::Undecided {
                bci,
                class,
                refusal,
            },
        };
    };
    // JVMS 4.9.2, which the frame pass already enforces on the very token this receiver is: an
    // `UninitializedThis` is converted by an `<init>` of the class that declares the constructor or
    // of its `super_class` — so a call that does *not* name the declaring class names the direct
    // superclass, and there is no third possibility to guess at.
    let target_kind = if class == declaring.name() {
        ConstructorTarget::This
    } else {
        ConstructorTarget::Super
    };
    let declared = declaring.name().to_string();
    Prologues {
        prologue: Some(Prologue {
            bci,
            target: target_kind,
            class: class.clone(),
            declared: declared.clone(),
        }),
        refused: None,
        verdict: Verdict::Decided { class, declared },
    }
}

/// What one body's constructor prologue was read as (P3 2.3, `init@1`).
///
/// `target` is the whole point of the record: `this` and `super` are different programs, and which
/// one the call is comes from two facts a reader can check here — the class the call names and the
/// class the caller stated declares the body.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InitRecord {
    /// The BCI of the prologue's constructor call, when the body has one.
    pub bci: Option<u32>,
    /// `"this"` or `"super"`, when the run could decide between them.
    pub target: Option<ConstructorTarget>,
    /// The class the call names, in internal form.
    pub class: Option<String>,
    /// The class that declares this constructor, as the caller stated it.
    pub declared: Option<String>,
    /// Whether the prologue was presented.
    pub presented: bool,
    /// Why it was not, when it was not.
    pub refusal: Option<InitRefusal>,
}

impl InitRecord {
    /// Whether the prologue was presented.
    pub fn presented(&self) -> bool {
        self.presented
    }

    /// The rule this record is answerable to.
    pub fn rule(&self) -> RuleVersion {
        INIT_RULE
    }

    fn of_refusal_shape(bci: Option<u32>, class: Option<String>, refusal: InitRefusal) -> Self {
        Self {
            bci,
            target: None,
            class,
            declared: None,
            presented: false,
            refusal: Some(refusal),
        }
    }
}

/// Why one body's constructor prologue was not presented.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InitRefusal {
    /// The diagnostic code, one `jre_init_*` per link of the verification that can fail.
    pub code: &'static str,
    /// The rule that refused the prologue.
    pub rule: RuleVersion,
    /// The declared requirement that fell short, when the refusal is one of the rule's own
    /// preconditions.
    pub requirement: Option<String>,
    /// One sentence stating which link failed.
    pub message: String,
}

impl InitRefusal {
    fn of(refusal: &Refusal, bci: Option<u32>) -> Self {
        Self {
            code: refusal.code(),
            rule: INIT_RULE,
            requirement: refusal.requirement().map(Precondition::describe),
            message: match bci {
                Some(bci) => format!(
                    "the constructor prologue at BCI {bci} was not presented: {}",
                    refusal.message()
                ),
                None => format!(
                    "the constructor prologue was not presented: {}",
                    refusal.message()
                ),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pass::IrTable;
    use jarde_jvm::engine::analyze_method_ir;
    use jarde_jvm::environment::ResolutionEnvironment;
    use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
    use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
    use jarde_reader::budget::{Budget, Limits};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    };
    use jarde_reader::view::LoaderId;
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, ModuleMode, MultiReleasePolicy,
        PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
    };

    const POSITIVE: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/classes/nested/UseInner.class"
    );
    const WRONG_IDENTITY: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/wrong-identity.class"
    );
    const WRONG_IDENTITY_CHECKED: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/wrong-identity-checked.class"
    );
    const LATE_CHECK: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/late-check.class"
    );
    const NESTED_EFFECTS: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/negative-controls/NegativeUse.class"
    );

    fn proof_budget() -> Budget {
        Budget::new(Limits {
            input_bytes: 1 << 20,
            archive_entries: 100,
            entry_bytes: 1 << 20,
            read_bytes: 1 << 20,
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            output_bytes: 1 << 20,
            class_headers: 100,
            method_bodies: 100,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            nested_depth: 8,
            dependency_depth: 8,
            elapsed_millis: u64::MAX,
        })
    }

    fn analyzed_caller(
        class: &[u8],
        name: &str,
        descriptor: &str,
    ) -> (jarde_jvm::method_ir::MethodIrAnalysis, PhysicalDefinitionId) {
        let mut budget = proof_budget();
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
            .expect("caller class opens");
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: class.len() as u64,
            },
            variant: PhysicalVariant::Base,
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
        let request = MethodAnalysisRequest {
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
            method: PhysicalMethodId {
                owner: definition.clone(),
                name: JvmBytes(name.as_bytes().to_vec()),
                descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        };
        (
            analyze_method_ir(&[snapshot], &request, &mut budget).expect("caller IR analyzes"),
            definition,
        )
    }

    fn target(definition: PhysicalDefinitionId, outer: &str) -> ProvedMemberInnerTarget {
        ProvedMemberInnerTarget {
            definition,
            owner: format!("{outer}$Inner"),
            outer: outer.to_owned(),
            simple_name: "Inner".to_owned(),
            constructor_descriptor: format!("(L{outer};I)V"),
            capture_field: "this$0".to_owned(),
            generic_diamond: false,
            source_type_path: Vec::new(),
        }
    }

    fn verdict_with(
        class: &[u8],
        name: &str,
        descriptor: &str,
        has_target: bool,
        handler_range: Option<(u32, u32)>,
        target_outer: Option<&str>,
    ) -> Result<Site, Refusal> {
        let (analysis, definition) = analyzed_caller(class, name, descriptor);
        let ir = analysis.ir();
        let mut code = ir.code().expect("code").clone();
        if let Some((start_bci, end_bci)) = handler_range {
            code.exception_handlers
                .push(jarde_reader::classfile::ExceptionHandlerFact {
                    ordinal: 0,
                    start_bci,
                    end_bci,
                    handler_bci: 0,
                    catch_type_index: None,
                });
        }
        let ssa = ir.ssa().expect("ssa");
        let operations = Operations::of(&code, ir.constant_pool());
        let (block, index) = ssa
            .blocks()
            .iter()
            .find_map(|block| {
                block
                    .instructions()
                    .iter()
                    .position(|instruction| {
                        matches!(
                            operations.get(instruction.bci()),
                            Some(Operation::Allocate { .. })
                        )
                    })
                    .map(|index| (block, index))
            })
            .expect("allocation block");
        let head = block.instructions()[index].bci();
        let Some(Operation::Allocate { ty }) = operations.get(head) else {
            unreachable!()
        };
        let outer = ty.strip_suffix("$Inner").expect("fixture member name");
        let mut targets = Vec::new();
        if has_target {
            let mut fact = target(definition, outer);
            if let Some(outer) = target_outer {
                fact.outer = outer.to_owned();
            }
            targets.push(fact);
        }
        let fields = field::Plan::empty();
        let facts = ConstructionFacts {
            ssa,
            operations: &operations,
            fields: &fields,
            member_targets: &targets,
            code: &code,
        };
        verify(head, index, block.instructions(), ty.clone(), &facts)
    }

    fn verdict(class: &[u8], name: &str, descriptor: &str) -> Result<Site, Refusal> {
        verdict_with(class, name, descriptor, true, None, None)
    }

    #[test]
    fn member_call_proof_requires_same_qualifier_and_early_check() {
        let positive = verdict(
            POSITIVE,
            "make",
            "(Lnested/SimpleOuter;I)Ljava/lang/Object;",
        )
        .expect("javac member call proves");
        let member = positive.member_inner.expect("member proof");
        assert_eq!((member.qualifier, member.check, member.pop), (4, 6, 9));
        assert_eq!(positive.arguments, [5, 13]);
        assert!(positive.owned.contains(&6) && positive.owned.contains(&9));

        let nested = verdict(
            NESTED_EFFECTS,
            "nestedEffects",
            "(Lnegative/NegativeOuter;I)Ljava/lang/Object;",
        )
        .expect("both nested argument effects remain in order");
        assert_eq!(nested.arguments, [5, 18]);
        let pre = verdict(
            NESTED_EFFECTS,
            "preEffect",
            "(Lnegative/NegativeOuter;I)Ljava/lang/Object;",
        )
        .expect("an effect completed before allocation remains outside the site");
        assert_eq!(pre.head, 7);
        assert_eq!(pre.arguments, [12, 20]);

        let wrong = verdict(
            WRONG_IDENTITY,
            "make",
            "(Lnested/SimpleOuter;Lnested/SimpleOuter;I)Ljava/lang/Object;",
        )
        .err()
        .expect("checked and physical outers differ");
        assert_eq!(wrong.code(), "jre_new_member_shape");
        let checked_shape = verdict(
            WRONG_IDENTITY_CHECKED,
            "make",
            "(Lnested/SimpleOuter;Lnested/SimpleOuter;I)Ljava/lang/Object;",
        )
        .err()
        .expect("a syntactically exact check cannot prove a different physical outer");
        assert_eq!(checked_shape.code(), "jre_new_member_shape");
        assert!(checked_shape.message().contains("two SSA copies"));
        let late = verdict(
            LATE_CHECK,
            "make",
            "(Lnested/SimpleOuter;I)Ljava/lang/Object;",
        )
        .err()
        .expect("argument effect precedes check");
        assert_eq!(late.code(), "jre_new_member_shape");

        let no_target = verdict_with(
            POSITIVE,
            "make",
            "(Lnested/SimpleOuter;I)Ljava/lang/Object;",
            false,
            None,
            None,
        )
        .err()
        .expect("without selected target the ordinary rule still refuses the member shape");
        assert_eq!(no_target.code(), "jre_new_interleaved_effect");

        let wrong_static_type = verdict_with(
            POSITIVE,
            "make",
            "(Lnested/SimpleOuter;I)Ljava/lang/Object;",
            true,
            None,
            Some("nested/OtherOuter"),
        )
        .err()
        .expect("qualifier static type must prove exact member binding");
        assert_eq!(wrong_static_type.code(), "jre_new_member_shape");

        for boundary in [(0, 6), (6, 16)] {
            let crossing = verdict_with(
                POSITIVE,
                "make",
                "(Lnested/SimpleOuter;I)Ljava/lang/Object;",
                true,
                Some(boundary),
                None,
            )
            .err()
            .expect("a changed handler region is not folded");
            assert_eq!(crossing.code(), "jre_new_member_order");
        }
    }

    #[test]
    fn the_two_construction_rules_state_what_they_read() {
        assert!(NEW.requires(Precondition::IrTable(IrTable::Ssa)));
        assert!(NEW.requires(Precondition::IrTable(IrTable::Code)));
        assert!(NEW.requires(Precondition::StatementFree));
        assert_eq!(
            NEW.required_release(),
            None,
            "`new T(…)` is Java in every release"
        );
        assert!(INIT.requires(DECLARING_CLASS));
        assert_eq!(INIT_RULE.citation(), "init@1");
        assert_eq!(NEW_RULE.citation(), "new@1");
        assert_eq!(ConstructorTarget::Super.spell(), "super");
        assert_eq!(ConstructorTarget::This.spell(), "this");
    }
}
