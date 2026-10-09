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

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{Definition, RefType, Slot, SsaInstruction, SsaTable, Value, ValueId};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{Base, MethodCodeFacts};
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
    /// The `pop` that discards the finished instance, when this site is a **statement-position**
    /// construction: `new X(args);`. The bytecode's only reader of the instance is that category-1
    /// discard, so the site's text is a statement of its own, written where the bytecode wrote the
    /// constructor call — and the `pop` is the anchor it is written at. `None` for every
    /// construction a store, a call, a `return` or a claimed field access consumes.
    pub(crate) discarded: Option<u32>,
    /// Every BCI the site owns: the allocation, the copy and the constructor call. An owned
    /// instruction produces no statement of its own — its text is the `new` expression, written
    /// where the instance is consumed and nowhere else.
    pub(crate) owned: BTreeSet<u32>,
    /// Every BCI whose value spells the instance this site builds: the allocation, its copy and
    /// its constructor call — plus the discarded null-check tail (`dup; check; pop`), when the
    /// compiler spelled one over the finished instance. The identity set the member proof matches
    /// the physical outer argument against, and the set the consumer of a nested construction is
    /// matched against: the tail's `dup` rewrites which SSA value the consumer reads, and that
    /// copy is still this construction's instance.
    pub(crate) instance: Vec<u32>,
    /// The exact completed instance value paired with an array store, only for composed sites.
    pub(crate) finished_value: Option<ValueId>,
    /// Every instruction from the allocation through the constructor call that the verifier
    /// accepted as this expression, including the value-producing argument instructions. A
    /// resource header can use this closed range to prove that its complete initializer is this
    /// site followed by the store that consumes the constructed instance.
    pub(crate) expression: BTreeSet<u32>,
    /// BCIs whose single-use facts belong to a closed proof atom: this site's and nested sites'
    /// constructor identity scaffolding, plus complete child-array chains already certified by
    /// `ChildArrayFacts`. This is deliberately narrower than `expression`; ordinary argument
    /// producers still pass the array expression's single-use check.
    pub(crate) single_use_atoms: BTreeSet<u32>,
    /// Recursively verified constructor sites inside this site's argument values. They remain
    /// separate allocation records when a composed array commits its construction closure.
    pub(crate) nested_sites: Vec<Site>,
}

impl Site {
    pub(crate) fn into_composition_sites(
        mut self,
        budget: &mut Budget,
    ) -> Result<Vec<Site>, crate::stop::StopReason> {
        crate::stop::poll(budget, Some(self.head))?;
        crate::stop::charge(budget, CountedBudgetDimension::IrItems, 1, Some(self.head))?;
        let mut pending = std::mem::take(&mut self.nested_sites);
        let mut sites = vec![self];
        while let Some(mut site) = pending.pop() {
            crate::stop::poll(budget, Some(site.head))?;
            crate::stop::charge(budget, CountedBudgetDimension::IrItems, 1, Some(site.head))?;
            for nested in std::mem::take(&mut site.nested_sites) {
                crate::stop::poll(budget, Some(nested.head))?;
                crate::stop::charge(
                    budget,
                    CountedBudgetDimension::IrItems,
                    1,
                    Some(nested.head),
                )?;
                pending.push(nested);
            }
            sites.push(site);
        }
        Ok(sites)
    }
}

/// The local qualifier and exact check already proved for one physical member constructor.
pub(crate) struct MemberInnerSite {
    pub(crate) qualifier: u32,
    pub(crate) check: u32,
    pub(crate) pop: u32,
    pub(crate) outer: String,
    pub(crate) simple_name: String,
    pub(crate) generic_diamond: bool,
    pub(crate) implicit_this: bool,
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
    chains: &'a crate::concat::Plan,
    /// The allocations another rule of this run writes the text of, handed in by [`sites`]: a
    /// nested-candidate scan must not claim what another rule already owns.
    reserved: &'a BTreeSet<u32>,
    fields: &'a field::Plan,
    arrays: &'a crate::build::ChildArrayFacts<'a>,
    java_release: u16,
    member_targets: &'a [ProvedMemberInnerTarget],
    method: Option<&'a crate::facts::MethodFacts>,
    code: &'a MethodCodeFacts,
}

/// The existing facts the array candidate needs to ask the construction verifier one narrowly
/// scoped question. The current candidate's child-array facts are supplied separately as a
/// borrowed view, so this context does not clone or publish an ArrayInitializers plan.
pub(crate) struct ArrayCompositionContext<'a> {
    pub(crate) chains: &'a crate::concat::Plan,
    pub(crate) reserved: &'a BTreeSet<u32>,
    pub(crate) java_release: u16,
    pub(crate) member_targets: &'a [ProvedMemberInnerTarget],
    pub(crate) method: &'a crate::facts::MethodFacts,
    pub(crate) code: &'a MethodCodeFacts,
}

#[derive(Debug)]
enum VerifyFailure {
    Refusal(Refusal),
    Stop(crate::stop::StopReason),
}

impl From<Refusal> for VerifyFailure {
    fn from(value: Refusal) -> Self {
        Self::Refusal(value)
    }
}

impl From<crate::stop::StopReason> for VerifyFailure {
    fn from(value: crate::stop::StopReason) -> Self {
        Self::Stop(value)
    }
}

pub(crate) struct VerifyMeter<'a> {
    budget: Option<&'a mut Budget>,
}

impl VerifyMeter<'_> {
    #[cfg(test)]
    pub(crate) fn unmetered() -> Self {
        Self { budget: None }
    }

    pub(crate) fn charge(
        &mut self,
        dimension: CountedBudgetDimension,
        at: Option<u32>,
    ) -> Result<(), crate::stop::StopReason> {
        let Some(budget) = self.budget.as_deref_mut() else {
            return Ok(());
        };
        crate::stop::poll(budget, at)?;
        crate::stop::charge(budget, dimension, 1, at)
    }
}

/// Proves one construction as the value of one already-paired aastore. This keeps ordinary
/// construction consumers closed while allowing the exact store under examination.
pub(crate) fn verify_array_store(
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    arrays: &crate::build::ChildArrayFacts<'_>,
    context: &ArrayCompositionContext<'_>,
    block: &[SsaInstruction],
    head: u32,
    index: usize,
    ty: String,
    store: u32,
    stored_value: ValueId,
    budget: &mut Budget,
) -> Result<Option<Site>, crate::stop::StopReason> {
    if context.reserved.contains(&head) || context.chains.owns(head) {
        return Ok(None);
    }
    let facts = ConstructionFacts {
        ssa,
        operations,
        chains: context.chains,
        reserved: context.reserved,
        fields,
        arrays,
        java_release: context.java_release,
        member_targets: context.member_targets,
        method: Some(context.method),
        code: context.code,
    };
    let mut meter = VerifyMeter {
        budget: Some(budget),
    };
    match verify_metered(
        head,
        index,
        block,
        ty,
        &facts,
        0,
        Some((store, stored_value)),
        &mut meter,
    ) {
        Ok(site) => Ok(Some(site)),
        Err(VerifyFailure::Refusal(_)) => Ok(None),
        Err(VerifyFailure::Stop(stop)) => Err(stop),
    }
}

/// How many levels of construction one `new` expression of this rule presents: the outer
/// construction plus **two** complete nested constructions in argument positions —
/// `new BufferedReader(new InputStreamReader(new FileInputStream(path), "UTF-8"))`, the
/// wrapped-stream chain the IO patrol's `countLines` writes, whose every inner run is a complete
/// construction and whose every value is the next constructor's own argument. A deeper run
/// (`new A(new B(new C(new D())))`) keeps its refusal and its registration: the sites inside it are
/// still proved on their own by the body walk, and the outermost is refused rather than
/// half-spelled. The limit is the **measured** boundary of that family, not a convenience: the
/// negative test below proves that a four-layer run's outermost still refuses, so raising this
/// number by one is what the chain needs and nothing wider.
const MAX_NESTED_CONSTRUCTION_LAYERS: u32 = 3;

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
    /// Every dynamic site whose bound-receiver creation-time check this run proved dead, with the
    /// three instructions of that check: owned exactly as a construction's own tail is, and read by
    /// the lambda plan's refusal — the check cannot fail, so the site's two presentations of a
    /// non-null receiver no longer disagree.
    receiver_tails: Vec<ReceiverTail>,
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
            receiver_tails: Vec::new(),
            allocation_candidates: Vec::new(),
            refusals: Vec::new(),
        }
    }

    /// Whether one instruction belongs to a verified site and so produces no statement of its own.
    pub(crate) fn owns(&self, bci: u32) -> bool {
        self.owned.contains(&bci)
    }

    /// The proved creation-time receiver check of one dynamic site, when this run proved one there.
    pub(crate) fn receiver_tail_at(&self, site: u32) -> Option<&ReceiverTail> {
        self.receiver_tails.iter().find(|tail| tail.site == site)
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

    /// The uniquely accepted construction site that produced this SSA value.
    pub(crate) fn site_producing(&self, ssa: &SsaTable, value: ValueId) -> Option<&Site> {
        let exact: Vec<&Site> = self
            .sites
            .iter()
            .filter(|site| site.finished_value == Some(value))
            .collect();
        if let [site] = exact.as_slice() {
            return Some(*site);
        }
        let mut matches = self.sites.iter().filter(|site| {
            site.finished_value.is_none()
                && is_the_instance(ssa, value, &[site.head, site.dup, site.constructor])
        });
        let site = matches.next()?;
        matches.next().is_none().then_some(site)
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
/// `chains` is the complete read-only answer of `concat@1`. Its owned instructions are not a
/// second `new`; a chain can be nested only when its tail is the actual value of a constructor
/// argument and this verifier closes the chain's ownership and range.
///
/// `fields` is the `field@1` plan of the same body, which this rule **reads** and never re-derives: a
/// claimed field access is one of the places a construction's instance is written into (P3 2c.26), and
/// whether an instruction really is an access to the member its own receiver's type declares is that
/// rule's verdict. `@field` is therefore decided before `@new` — [`crate::report`] runs the two in
/// that order — and the judgement is handed in rather than taken a second time here.
#[cfg(test)]
pub(crate) fn sites(
    ssa: &SsaTable,
    operations: &Operations,
    chains: &crate::concat::Plan,
    reserved: &BTreeSet<u32>,
    fields: &field::Plan,
    arrays: &crate::build::ArrayInitializers,
    java_release: u16,
    member_targets: &[ProvedMemberInnerTarget],
    method: &crate::facts::MethodFacts,
    code: &MethodCodeFacts,
) -> Sites {
    let mut meter = VerifyMeter::unmetered();
    sites_with_pending_array_composition(
        ssa,
        operations,
        chains,
        reserved,
        fields,
        arrays,
        BTreeMap::new(),
        java_release,
        member_targets,
        method,
        code,
        &mut meter,
    )
    .expect("an unmetered site census cannot stop")
}

/// The report pipeline's one handoff from the array candidate proof. Sites are moved out exactly
/// once, then enter the same allocation census and materializer as ordinary construction sites.
pub(crate) fn sites_after_array_composition(
    ssa: &SsaTable,
    operations: &Operations,
    chains: &crate::concat::Plan,
    reserved: &BTreeSet<u32>,
    fields: &field::Plan,
    arrays: &mut crate::build::ArrayInitializers,
    java_release: u16,
    member_targets: &[ProvedMemberInnerTarget],
    method: &crate::facts::MethodFacts,
    code: &MethodCodeFacts,
    budget: &mut Budget,
) -> Result<Sites, crate::stop::StopReason> {
    let pending_sites = arrays.take_pending_sites();
    let mut meter = VerifyMeter {
        budget: Some(budget),
    };
    sites_with_pending_array_composition(
        ssa,
        operations,
        chains,
        reserved,
        fields,
        arrays,
        pending_sites,
        java_release,
        member_targets,
        method,
        code,
        &mut meter,
    )
}

#[allow(clippy::too_many_arguments)]
fn sites_with_pending_array_composition(
    ssa: &SsaTable,
    operations: &Operations,
    chains: &crate::concat::Plan,
    reserved: &BTreeSet<u32>,
    fields: &field::Plan,
    arrays: &crate::build::ArrayInitializers,
    mut pending_sites: BTreeMap<u32, Site>,
    java_release: u16,
    member_targets: &[ProvedMemberInnerTarget],
    method: &crate::facts::MethodFacts,
    code: &MethodCodeFacts,
    meter: &mut VerifyMeter<'_>,
) -> Result<Sites, crate::stop::StopReason> {
    let array_facts = arrays.child_facts();
    let facts = ConstructionFacts {
        ssa,
        operations,
        chains,
        reserved,
        fields,
        arrays: &array_facts,
        java_release,
        member_targets,
        method: Some(method),
        code,
    };
    let mut plan = Sites::empty();
    for ssa_block in ssa.blocks() {
        let block = ssa_block.instructions();
        for (index, instruction) in block.iter().enumerate() {
            let head = instruction.bci();
            meter.charge(CountedBudgetDimension::AnalysisSteps, Some(head))?;
            let Some(Operation::Allocate { ty }) = operations.get(head) else {
                continue;
            };
            let candidate_index = plan.allocation_candidates.len();
            plan.allocation_candidates.push(AllocationCandidate {
                head,
                class: ty.clone(),
                verified: false,
            });
            if reserved.contains(&head) || chains.owns(head) {
                // Another rule of this run writes this allocation's text: it is not a second shape.
                continue;
            }
            if let Some(site) = pending_sites.remove(&head) {
                plan.allocation_candidates[candidate_index].verified = true;
                plan.owned.extend(site.owned.iter().copied());
                plan.sites.push(site);
                continue;
            }
            let ty = ty.clone();
            match verify_metered(head, index, block, ty.clone(), &facts, 0, None, meter) {
                Ok(site) => {
                    plan.allocation_candidates[candidate_index].verified = true;
                    for bci in &site.owned {
                        plan.owned.insert(*bci);
                    }
                    plan.sites.push(site);
                }
                Err(VerifyFailure::Refusal(refusal)) => plan.refusals.push(Refused {
                    head,
                    class: ty,
                    refusal,
                }),
                Err(VerifyFailure::Stop(stop)) => return Err(stop),
            }
        }
    }
    plan.allocation_candidates
        .sort_by_key(|candidate| candidate.head);
    plan.refusals.sort_by_key(|refused| refused.head);
    // The bound-receiver tails of this body's dynamic sites, after the construction sites have
    // claimed theirs: a window a verified construction already owns is that site's tail, and the
    // first proof to claim three instructions owns them.
    for tail in receiver_tails(ssa, operations) {
        if plan.owned.contains(&tail.copy) {
            continue;
        }
        plan.owned.insert(tail.copy);
        plan.owned.insert(tail.check);
        plan.owned.insert(tail.pop);
        plan.receiver_tails.push(tail);
    }
    plan.receiver_tails.sort_by_key(|tail| tail.site);
    Ok(plan)
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
///
/// `depth` counts construction layers: 0 for the body's own walk, 1 for a construction proved
/// recursively inside another one's argument run. The scan below steps over a nested
/// `new; dup; …; invokespecial` run only while `depth + 1` stays under
/// [`MAX_NESTED_CONSTRUCTION_LAYERS`].
#[cfg(test)]
fn verify(
    head: u32,
    index: usize,
    block: &[SsaInstruction],
    ty: String,
    facts: &ConstructionFacts<'_>,
    depth: u32,
    expected_array_store: Option<(u32, ValueId)>,
) -> Result<Site, Refusal> {
    let mut meter = VerifyMeter { budget: None };
    match verify_metered(
        head,
        index,
        block,
        ty,
        facts,
        depth,
        expected_array_store,
        &mut meter,
    ) {
        Ok(site) => Ok(site),
        Err(VerifyFailure::Refusal(refusal)) => Err(refusal),
        Err(VerifyFailure::Stop(_)) => unreachable!("an unmetered verifier cannot stop"),
    }
}

fn verify_metered(
    head: u32,
    index: usize,
    block: &[SsaInstruction],
    ty: String,
    facts: &ConstructionFacts<'_>,
    depth: u32,
    expected_array_store: Option<(u32, ValueId)>,
    meter: &mut VerifyMeter<'_>,
) -> Result<Site, VerifyFailure> {
    let ConstructionFacts {
        ssa,
        operations,
        chains,
        reserved,
        fields,
        arrays,
        java_release,
        member_targets,
        ..
    } = *facts;
    let shape = |detail: String| VerifyFailure::Refusal(Refusal::shape("jre_new_shape", detail));
    let Some(dup) = block.get(index + 1) else {
        return Err(shape(format!(
            "the allocation at BCI {head} ends its block: a construction continues with the `dup` of the instance"
        )));
    };
    if operations.get(dup.bci()) != Some(&Operation::Duplicate)
        || (expected_array_store.is_some() && dup.opcode() != 0x59)
    {
        return Err(shape(format!(
            "the instruction after the allocation at BCI {head} is at BCI {}, and a construction continues with the `dup` of the instance it allocated",
            dup.bci()
        )));
    }
    // The constructor call this expression writes: the first `<init>` on this allocation's own
    // class after the copy. What stands between the copy and that call are the argument runs, and
    // one argument can be **another complete construction** — `new X(msg, new Y("inner"))`, the
    // wrapped-exception and wrapper-object shape — whose own `new; dup; …; invokespecial` run
    // completes inside this one's arguments. The scan steps over such a run only when this same
    // verification proves it recursively, and what that proof states is exactly what the outer
    // expression needs: the nested constructor matches the nested allocation, the nested value is
    // single-use, and its closed run is this block's own contiguous instruction range. A candidate
    // that does not prove is **not** stepped over — the ordinary walk reads on, and the checks
    // below refuse the outer construction exactly as they did before this rule knew about nesting.
    let mut nested_sites: Vec<Site> = Vec::new();
    let mut scan = index + 2;
    let found = loop {
        let Some(instruction) = block.get(scan) else {
            break None;
        };
        let bci = instruction.bci();
        meter.charge(CountedBudgetDimension::AnalysisSteps, Some(bci))?;
        if depth + 1 < MAX_NESTED_CONSTRUCTION_LAYERS
            && !reserved.contains(&bci)
            && !chains.owns(bci)
            && let Some(Operation::Allocate { ty: nested_ty }) = operations.get(bci)
            && block
                .get(scan + 1)
                .is_some_and(|next| operations.get(next.bci()) == Some(&Operation::Duplicate))
        {
            let site = match verify_metered(
                bci,
                scan,
                block,
                nested_ty.clone(),
                facts,
                depth + 1,
                None,
                meter,
            ) {
                Ok(site) => site,
                Err(VerifyFailure::Refusal(_)) => {
                    scan += 1;
                    continue;
                }
                Err(VerifyFailure::Stop(stop)) => return Err(VerifyFailure::Stop(stop)),
            };
            // Step over the whole closed run: the nested constructor is the last instruction of
            // it, and the outer construction's own call is the first one after it that names this
            // allocation's class.
            let mut after = None;
            for (position, candidate) in block.iter().enumerate() {
                meter.charge(CountedBudgetDimension::AnalysisSteps, Some(candidate.bci()))?;
                if candidate.bci() == site.constructor {
                    after = Some(position + 1);
                    break;
                }
            }
            let after = after.expect("the nested construction's call belongs to this block");
            nested_sites.push(site);
            scan = after;
            continue;
        }
        if matches!(
            operations.get(bci),
            Some(Operation::Invoke(target)) if target.owner() == ty && target.name() == "<init>"
        ) {
            break Some(instruction);
        }
        scan += 1;
    };
    let Some(constructor) = found else {
        return Err(shape(format!(
            "the allocation at BCI {head} is never constructed: no `{ty}.<init>(…)` call reads the instance it builds and the copy of it"
        )));
    };
    let at = constructor.bci();
    // The closed runs the scan stepped over: every instruction of every recursively proved nested
    // construction, which the span check below accepts as part of this one expression.
    let mut nested_expression = BTreeSet::new();
    for site in &nested_sites {
        meter.charge(CountedBudgetDimension::IrItems, Some(site.head))?;
        for bci in &site.expression {
            meter.charge(CountedBudgetDimension::IrItems, Some(*bci))?;
            nested_expression.insert(*bci);
        }
    }
    // The instructions the instance comes from: the allocation, its copy and the constructor that
    // initialized it. A compiler may put the stored value down as any of the three, and what matters
    // is that the value really belongs to *this* allocation.
    let mut produced_by: Vec<u32> = vec![head, dup.bci(), at];
    let operands = stack_operands_metered(constructor, at, meter)?;
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
    // One compiler spells a discarded null check **over the finished instance** before the instance
    // is consumed: `dup; <discarded null check>; pop` immediately after the constructor call —
    // real javac 8 writes it for every source-qualified member construction whose qualifier is a
    // fresh allocation (`new Outer().new Inner(…)`), javac 9+ writes none. The check reads the
    // instance this site builds and its result is dropped, so the three instructions are the
    // qualifier's own spelling, not a reader of the instance: they join the site's own
    // instructions here, and the value the construction's consumer reads is the copy the `dup`
    // writes — the identity set below states that, so the member proof matches its physical outer
    // argument against it (P3 2c.26's "one construction instance, one Java spelling" is untouched:
    // the reader gate still demands exactly one instruction outside the site that writes the
    // instance somewhere, and the check's own result being dropped is proved by the single-use
    // facts of the tail, not by a spelling alone).
    let mut constructor_position = None;
    for (position, instruction) in block.iter().enumerate() {
        meter.charge(
            CountedBudgetDimension::AnalysisSteps,
            Some(instruction.bci()),
        )?;
        if instruction.bci() == at {
            constructor_position = Some(position);
            break;
        }
    }
    let tail = constructor_position
        .map(|position| {
            discarded_null_check_tail_metered(ssa, operations, block, position, &produced_by, meter)
        })
        .transpose()?;
    if let Some(tail) = tail.flatten() {
        produced_by.extend_from_slice(&tail);
    }
    let mut selected_member = None;
    for target in member_targets {
        meter.charge(CountedBudgetDimension::IrItems, Some(at))?;
        if target.owner == ty
            && matches!(
                operations.get(at),
                Some(Operation::Invoke(call)) if call.descriptor() == target.constructor_descriptor
            )
        {
            selected_member = Some(target);
            break;
        }
    }
    let member = selected_member
        .map(|target| {
            verify_member(
                index,
                block,
                constructor,
                &operands,
                facts,
                target,
                &nested_sites,
                meter,
            )
        })
        .transpose()?;
    // Every nested construction the scan stepped over is one **argument** of this call: its
    // completed instance is exactly the value the call reads at that position — the same "the
    // value is the argument" fact the presentation states when it spells the site inside the `new`
    // expression. A run that completes inside this construction for any other reader — a call on
    // the fresh instance, a store of it — has no place in the expression, and this construction
    // keeps its refusal.
    for nested in &nested_sites {
        meter.charge(CountedBudgetDimension::IrItems, Some(nested.head))?;
        let nested_produced_by = nested.instance.as_slice();
        let mut is_argument = false;
        for (_, value) in operands.iter().skip(1) {
            meter.charge(CountedBudgetDimension::IrItems, Some(at))?;
            if is_the_instance(ssa, *value, &nested_produced_by) {
                is_argument = true;
                break;
            }
        }
        if !is_argument {
            return Err(shape(format!(
                "the construction at BCI {} completes inside the construction at BCI {head}, and its value is not one of the arguments of the constructor call at BCI {at}: the `new` expression has no single place to write it",
                nested.head
            )));
        }
    }
    let mut embedded_concat = false;
    let mut embedded_dynamic = false;
    let mut embedded_primitive_conversion = false;
    let mut embedded_array = BTreeSet::new();
    let mut inline_arrays = BTreeSet::new();
    let arguments = if let Some(member) = &member {
        for argument in &member.arguments {
            meter.charge(CountedBudgetDimension::IrItems, Some(*argument))?;
        }
        member.arguments.clone()
    } else {
        // Every argument has to be produced **between the copy and the call**, so that writing it as an
        // argument of the `new` expression evaluates it exactly where the bytecode evaluated it. A value
        // produced elsewhere would move, and this rule never moves a value.
        let mut arguments: Vec<u32> = Vec::new();
        for (_, value) in operands.iter().skip(1) {
            meter.charge(CountedBudgetDimension::IrItems, Some(at))?;
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
        let argument_dependencies = value_dependency_bcis_metered(
            ssa,
            block,
            operands.iter().skip(1).map(|(_, value)| *value),
            meter,
        )?;
        // The restricted Java 8 `String(char[])` shape keeps its own slice: its argument run's
        // inline `char[]` chains — direct or through an intervening call — are embedded only under
        // that slice's exact conditions, and the general inline array acceptance below never
        // applies to this constructor.
        let string_char_array_ctor = java_release == 8
            && ty == "java/lang/String"
            && matches!(operations.get(at), Some(Operation::Invoke(call)) if call.descriptor() == "([C)V")
            && operands.len() == 2;
        if string_char_array_ctor {
            embedded_array = arrays
                .inline_char_argument_bcis_metered(
                    ssa,
                    operations,
                    block,
                    operands[1].1,
                    dup.bci(),
                    at,
                    meter,
                )
                .map_err(VerifyFailure::Stop)?
                .unwrap_or_default();
        }
        // The inline anonymous array chains of this construction's argument run — the varargs
        // lowering `new T; dup; …; anewarray; [dup; index; value; aastore]×n; invoke` that a
        // collection-copy constructor reads through a factory call such as `Arrays.asList(…)`.
        // The chain's own proof (element production, closed interval, single use of every value) is
        // `array@1`'s, read here rather than restated: a construction accepts the chain only when
        // its sole consumer is an invocation this call's own argument dependency walk reaches, so
        // the initializer is written exactly where the bytecode evaluated it and serves exactly
        // this argument. A bare `new T[n]` allocation with no element stores (the empty varargs
        // call) is its own complete chain under the same consumer judgement. A chain the
        // constructor consumes directly is not this shape: `new@1` embeds it only for the
        // `String(char[])` slice, and every other owner keeps the refusal that slice froze.
        let array_serves_argument = |consumer: u32| {
            matches!(operations.get(consumer), Some(Operation::Invoke(_)))
                && argument_dependencies.contains(&consumer)
        };
        if !string_char_array_ctor {
            for instruction in block.iter().skip(index + 2) {
                if instruction.bci() >= at {
                    break;
                }
                meter.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    Some(instruction.bci()),
                )?;
                let bci = instruction.bci();
                if let Some(members) = arrays
                    .inline_argument_chain_bcis_metered(
                        bci,
                        dup.bci(),
                        at,
                        array_serves_argument,
                        meter,
                    )
                    .map_err(VerifyFailure::Stop)?
                {
                    inline_arrays.extend(members);
                    continue;
                }
                if matches!(operations.get(bci), Some(Operation::NewArray { .. }))
                    && let Some((Slot::Stack(_), value)) = instruction.writes().first().copied()
                {
                    let uses = ssa.value(value).uses();
                    for use_site in uses {
                        meter.charge(CountedBudgetDimension::IrItems, use_site.bci())?;
                    }
                    if let [use_site] = uses
                        && let Some(consumer) = use_site.bci()
                        && array_serves_argument(consumer)
                    {
                        inline_arrays.insert(bci);
                    }
                }
            }
        }
        let nested_concat = verify_concat_arguments_metered(
            head,
            dup.bci(),
            at,
            block,
            ssa,
            chains,
            operands.iter().skip(1).map(|(_, value)| *value),
            &argument_dependencies,
            meter,
        )?;
        embedded_concat = !nested_concat.is_empty();
        // Nothing inside the span may be an effect this rule would have to move: every invocation
        // between the copy and the call must be in an argument's own value dependency chain.
        for instruction in block.iter().skip(index + 2) {
            if instruction.bci() >= at {
                break;
            }
            meter.charge(
                CountedBudgetDimension::AnalysisSteps,
                Some(instruction.bci()),
            )?;
            match operations.get(instruction.bci()) {
                Some(_) if nested_expression.contains(&instruction.bci()) => {}
                Some(_) if nested_concat.contains(&instruction.bci()) => {}
                Some(_) if embedded_array.contains(&instruction.bci()) => {}
                Some(_) if inline_arrays.contains(&instruction.bci()) => {}
                Some(
                    Operation::Push(_)
                    | Operation::Load { .. }
                    | Operation::Arithmetic { .. }
                    | Operation::Negate,
                ) => {}
                Some(Operation::Invoke(_))
                    if argument_dependencies.contains(&instruction.bci()) => {}
                Some(Operation::InvokeDynamic(_))
                    if argument_dependencies.contains(&instruction.bci()) =>
                {
                    // Set this only for a dynamic value physically inside this closed run. A
                    // value materialized earlier and later loaded from a local is not re-created
                    // by the `new` expression and does not widen its handler boundary.
                    embedded_dynamic = true;
                }
                Some(Operation::PrimitiveConversion { .. })
                    if argument_dependencies.contains(&instruction.bci()) =>
                {
                    embedded_primitive_conversion = true;
                }
                Some(Operation::Invoke(_)) => {
                    return Err(Refusal::unmet(
                        &NEW,
                        Precondition::StatementFree,
                        format!(
                            "the invocation at BCI {} is not a value dependency of the constructor's physical arguments at BCI {at}, so presenting the construction would move that call effect",
                            instruction.bci()
                        ),
                    )
                    .into());
                }
                Some(operation) => {
                    return Err(Refusal::unmet(
                        &NEW,
                        Precondition::StatementFree,
                        format!(
                            "the instruction at BCI {} is an {operation:?} between the allocation's copy and its constructor call, and presenting the construction would write that effect somewhere else",
                            instruction.bci()
                        ),
                    )
                    .into());
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
    let readers = outside_readers(ssa, block, &produced_by, meter)?;
    let mut written = Vec::new();
    for bci in &readers {
        meter.charge(CountedBudgetDimension::IrItems, Some(*bci))?;
        if renders_its_reads(operations, fields, *bci)
            || expected_array_store.is_some_and(|(store, _)| *bci == store)
        {
            written.push(*bci);
        }
    }
    // The **statement position** (`recover-statement-position-news`): a construction whose finished
    // instance the body discards has no store, call, `return` or claimed field access to write its
    // text at — `new X(args);` is a statement of its own, and the category-1 `pop` that discards
    // the instance is where it is written. Three facts are read here, and all three are the
    // bytecode's own: the reader is the `pop` immediately after the constructor call, that `pop`
    // reads the value the call wrote and nothing else reads it, and every argument is a value the
    // `new` expression writes in place — a constant, a direct local or parameter read, or a
    // construction this same proof completed ([`arguments_without_invocations`]).
    //
    // A construction whose arguments carry an invocation of their own keeps the refusal
    // `refuse-unconsumed-construction-invokes` froze for the CST counterexample, and the invocation
    // that is not an argument's value dependency is refused earlier still — before this check, by
    // the argument-effect scan above — so that counterexample's code and BCI never move.
    let discarded = if member.is_none()
        && written.is_empty()
        && let [pop] = readers.as_slice()
        && discards_the_instance(ssa, block, at, *pop)
        && arguments_without_invocations(ssa, operations, &operands, &nested_sites)
    {
        Some(*pop)
    } else {
        None
    };
    if written.is_empty() && discarded.is_none() {
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
    let invalid_array_store = if let Some((store, stored)) = expected_array_store {
        readers.as_slice() != [store]
            || written.as_slice() != [store]
            || !array_store_consumes_site_value(
                ssa,
                operations,
                block,
                head,
                dup.bci(),
                at,
                store,
                stored,
                &ty,
                meter,
            )?
    } else {
        false
    };
    if readers.len() != 1 || invalid_array_store {
        return Err(shape(format!(
            "the instance the allocation at BCI {head} builds is read by the instructions at BCIs {}, and a construction is written as one `new` expression in one place: a leftover that more than one instruction reads has no single Java spelling",
            readers
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    // The one place the instance is written: the `pop` that discards it, when the body discards it,
    // and the reader's own instruction otherwise.
    let written: Vec<u32> = match discarded {
        Some(pop) => vec![pop],
        None => written,
    };
    let mut owned: BTreeSet<u32> = produced_by.iter().copied().collect();
    if let Some(member) = &member {
        for bci in &member.owned {
            meter.charge(CountedBudgetDimension::IrItems, Some(*bci))?;
            owned.insert(*bci);
        }
    }
    let mut constructor_index = None;
    for (position, instruction) in block.iter().enumerate() {
        meter.charge(
            CountedBudgetDimension::AnalysisSteps,
            Some(instruction.bci()),
        )?;
        if instruction.bci() == at {
            constructor_index = Some(position);
            break;
        }
    }
    let constructor_index =
        constructor_index.expect("the selected constructor belongs to this block");
    if !embedded_array.is_empty()
        && (block.get(constructor_index + 1).map(SsaInstruction::bci) != Some(written[0])
            || !matches!(operations.get(written[0]), Some(Operation::Return { .. })))
    {
        return Err(Refusal::shape(
            "jre_new_inline_char_array_return",
            format!(
                "the String(char[]) construction at BCI {head} does not directly return its sole constructed value"
            ),
        )
        .into());
    }
    if embedded_concat
        || embedded_dynamic
        || embedded_primitive_conversion
        || !embedded_array.is_empty()
        || !inline_arrays.is_empty()
        || !nested_expression.is_empty()
    {
        // A Java expression runs under one exception region. Moving a proved inner run — a
        // concatenation chain, a dynamic functional argument, an inline char[] initializer, an
        // inline array argument chain, a nested construction — into the outer construction is
        // sound only when every instruction in the construction and its sole consumer has the same
        // handler coverage as the outer allocation.
        let mut expected = Vec::new();
        for handler in &facts.code.exception_handlers {
            meter.charge(CountedBudgetDimension::IrItems, Some(head))?;
            if handler.start_bci <= head && head < handler.end_bci {
                expected.push(handler.ordinal);
            }
        }
        let consumer = written[0];
        let mut boundary_crossed = false;
        for instruction in &block[index..=constructor_index] {
            let mut coverage = Vec::new();
            for handler in &facts.code.exception_handlers {
                meter.charge(CountedBudgetDimension::IrItems, Some(instruction.bci()))?;
                if handler.start_bci <= instruction.bci() && instruction.bci() < handler.end_bci {
                    coverage.push(handler.ordinal);
                }
            }
            if coverage != expected {
                boundary_crossed = true;
                break;
            }
        }
        if !boundary_crossed {
            let mut coverage = Vec::new();
            for handler in &facts.code.exception_handlers {
                meter.charge(CountedBudgetDimension::IrItems, Some(consumer))?;
                if handler.start_bci <= consumer && consumer < handler.end_bci {
                    coverage.push(handler.ordinal);
                }
            }
            boundary_crossed = coverage != expected;
        }
        if boundary_crossed {
            let (code, argument) = if embedded_dynamic {
                (
                    "jre_new_dynamic_argument_exception_boundary",
                    "dynamic functional",
                )
            } else if embedded_concat {
                ("jre_new_concat_exception_boundary", "concatenation")
            } else if !embedded_array.is_empty() {
                (
                    "jre_new_inline_char_array_exception_boundary",
                    "inline char[]",
                )
            } else if !inline_arrays.is_empty() {
                ("jre_new_inline_array_exception_boundary", "inline array")
            } else if embedded_primitive_conversion {
                (
                    "jre_new_primitive_conversion_exception_boundary",
                    "primitive conversion",
                )
            } else {
                ("jre_new_nested_exception_boundary", "nested construction")
            };
            return Err(Refusal::shape(
                code,
                format!(
                    "the {argument} argument of the construction at BCI {head} crosses an exception-handler boundary before its constructor or sole consumer at BCI {consumer}"
                ),
            )
            .into());
        }
    }
    let mut expression = BTreeSet::new();
    for instruction in &block[index..=constructor_index] {
        meter.charge(
            CountedBudgetDimension::AnalysisSteps,
            Some(instruction.bci()),
        )?;
        expression.insert(instruction.bci());
    }
    let mut single_use_atoms: BTreeSet<u32> = produced_by.iter().copied().collect();
    for nested in &nested_sites {
        meter.charge(CountedBudgetDimension::IrItems, Some(nested.head))?;
        for bci in &nested.single_use_atoms {
            meter.charge(CountedBudgetDimension::IrItems, Some(*bci))?;
            single_use_atoms.insert(*bci);
        }
    }
    for bci in &embedded_array {
        meter.charge(CountedBudgetDimension::IrItems, Some(*bci))?;
        single_use_atoms.insert(*bci);
    }
    for bci in &inline_arrays {
        meter.charge(CountedBudgetDimension::IrItems, Some(*bci))?;
        single_use_atoms.insert(*bci);
    }
    Ok(Site {
        head,
        dup: dup.bci(),
        constructor: at,
        class: ty,
        arguments,
        member_inner: member.map(|proof| proof.site),
        discarded,
        owned,
        instance: produced_by,
        finished_value: expected_array_store.map(|(_, value)| value),
        expression,
        single_use_atoms,
        nested_sites,
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
    nested_sites: &[Site],
    meter: &mut VerifyMeter<'_>,
) -> Result<MemberProof, VerifyFailure> {
    let ConstructionFacts {
        ssa,
        operations,
        code,
        method,
        ..
    } = *facts;
    let at = constructor.bci();
    let shape =
        |detail: String| VerifyFailure::Refusal(Refusal::shape("jre_new_member_shape", detail));
    let order =
        |detail: String| VerifyFailure::Refusal(Refusal::shape("jre_new_member_order", detail));
    let Some((_, physical_outer)) = operands.get(1).copied() else {
        return Err(shape(format!(
            "the member constructor at BCI {at} has no physical outer argument"
        )));
    };
    let outer_descriptor = format!("L{};", target.outer);
    // The frame facts keep a `new`/`this` class name as an internal name while a descriptor
    // parameter keeps its `L...;` spelling; both spell this exact selected Outer.
    let names_outer = |value: ValueId| {
        matches!(ssa.value(value).ty(),
            Value::Ref(RefType::Named { name, .. })
                if name == target.outer.as_bytes() || name == outer_descriptor.as_bytes())
    };
    if let (Some([receiver, call]), Some(method_facts)) = (block.get(index + 2..index + 4), method)
    {
        let root_descriptor = [b"()L".as_slice(), target.owner.as_bytes(), b";"].concat();
        let constructor_descriptor = format!("(L{};)V", target.outer);
        let exact_root_return = method_facts.name() == "make"
            && method_facts.descriptor().as_bytes() == root_descriptor
            && method_facts
                .access_flags()
                .is_some_and(|flags| flags & 0x0008 == 0)
            && method_facts
                .declaring_class()
                .is_some_and(|class| class.name() == target.outer);
        let mut exact_opcodes = true;
        for (instruction, expected) in code.instructions.iter().zip([0xbb, 0x59, 0x2a, 0xb7, 0xb0])
        {
            meter.charge(CountedBudgetDimension::AnalysisSteps, Some(instruction.bci))?;
            exact_opcodes &= instruction.opcode == expected;
        }
        let exact_code = code.instructions.len() == 5
            && code.stopped_at.is_none()
            && exact_opcodes
            && code.exception_handlers.is_empty()
            && code.exception_handler_count == 0;
        let receiver_value = receiver.writes().first().map(|(_, value)| *value);
        if exact_root_return
            && exact_code
            && receiver.opcode() == 0x2a
            && matches!(
                operations.get(receiver.bci()),
                Some(Operation::Load { slot: 0 })
            )
            && matches!(operations.get(call.bci()), Some(Operation::Invoke(invoke))
                if invoke.kind() == crate::facts::InvokeKind::Special
                    && invoke.owner() == target.owner
                    && invoke.name() == "<init>"
                    && invoke.descriptor() == constructor_descriptor)
            && operands.len() == 2
            && receiver_value == Some(physical_outer)
            && single_use_at_metered(ssa, physical_outer, call.bci(), meter)?
            && receiver_value.is_some_and(names_outer)
        {
            let receiver_bci = receiver.bci();
            return Ok(MemberProof {
                site: MemberInnerSite {
                    qualifier: receiver_bci,
                    check: receiver_bci,
                    pop: receiver_bci,
                    outer: target.outer.clone(),
                    simple_name: target.simple_name.clone(),
                    generic_diamond: false,
                    implicit_this: true,
                },
                arguments: vec![receiver_bci],
                owned: [block[index + 1].bci(), receiver_bci].into_iter().collect(),
            });
        }
    }
    // The enclosing-`this` qualifier: javac needs no null check for the outer instance a
    // non-static member binds to when that instance is the calling method's own `this`, so the
    // physical outer argument is the slot-0 entry value itself. The qualifier's SSA type — never
    // the descriptor's first parameter type — states which class's `this` this is.
    if let Some(qualifier) = block.get(index + 2)
        && matches!(
            operations.get(qualifier.bci()),
            Some(Operation::Load { slot: 0 })
        )
    {
        let qualifier_writes = qualifier.writes();
        if qualifier_writes.len() == 1
            && qualifier_writes[0].1 == physical_outer
            && single_use_at_metered(ssa, physical_outer, at, meter)?
            && names_outer(physical_outer)
        {
            let qualifier_bci = qualifier.bci();
            let ordinary = member_ordinary_arguments(
                index + 3,
                qualifier_bci,
                at,
                block,
                operands,
                ssa,
                operations,
                meter,
            )?;
            let mut arguments = vec![qualifier_bci];
            arguments.extend(ordinary);
            return Ok(MemberProof {
                site: MemberInnerSite {
                    qualifier: qualifier_bci,
                    check: qualifier_bci,
                    pop: qualifier_bci,
                    outer: target.outer.clone(),
                    simple_name: target.simple_name.clone(),
                    generic_diamond: target.generic_diamond,
                    implicit_this: true,
                },
                arguments,
                owned: [block[index + 1].bci(), qualifier_bci]
                    .into_iter()
                    .collect(),
            });
        }
    }
    // The allocation qualifier: `new Outer().new Inner(…)` evaluates a fresh, provably non-null
    // Outer first, and javac again writes no null check for it. The physical outer argument is
    // the completed instance of one nested construction this same scan proved — the scan accepts
    // that run as an argument of this call, and this arm proves it is *the* first argument.
    let mut matching_nested = None;
    for nested in nested_sites {
        meter.charge(CountedBudgetDimension::IrItems, Some(nested.head))?;
        if nested.class == target.outer {
            matching_nested = Some(nested);
            break;
        }
    }
    if let Some(nested) = matching_nested
        && block
            .get(index + 2)
            .is_some_and(|head| head.bci() == nested.head)
        && is_the_instance(ssa, physical_outer, nested.instance.as_slice())
        && single_use_at_metered(ssa, physical_outer, at, meter)?
        && names_outer(physical_outer)
    {
        let qualifier_end = nested
            .instance
            .last()
            .copied()
            .unwrap_or(nested.constructor);
        let mut after_nested = None;
        for (position, instruction) in block.iter().enumerate() {
            meter.charge(
                CountedBudgetDimension::AnalysisSteps,
                Some(instruction.bci()),
            )?;
            if instruction.bci() == qualifier_end {
                after_nested = Some(position + 1);
                break;
            }
        }
        let after_nested =
            after_nested.expect("the nested construction's tail belongs to this block");
        let ordinary = member_ordinary_arguments(
            after_nested,
            nested.constructor,
            at,
            block,
            operands,
            ssa,
            operations,
            meter,
        )?;
        let mut arguments = vec![nested.constructor];
        arguments.extend(ordinary);
        return Ok(MemberProof {
            site: MemberInnerSite {
                qualifier: nested.constructor,
                check: nested.constructor,
                pop: nested.constructor,
                outer: target.outer.clone(),
                simple_name: target.simple_name.clone(),
                generic_diamond: target.generic_diamond,
                implicit_this: false,
            },
            arguments,
            // The qualifier's own instructions belong to the nested construction's site: this
            // site owns only its `dup` beside the constructor it names.
            owned: [block[index + 1].bci()].into_iter().collect(),
        });
    }
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
        if crate::facts::is_discarded_null_check(
            call.kind(),
            call.owner().as_bytes(),
            call.name().as_bytes(),
            call.descriptor().as_bytes(),
            call.is_interface_reference(),
        ))
        || pop.opcode() != 0x57
    {
        return Err(shape(format!(
            "the member constructor at BCI {at} lacks the contiguous local load, dup, discarded null check, pop"
        )));
    }
    let qualifier_writes = qualifier.writes();
    let copy_reads = stack_operands_metered(copy, copy.bci(), meter)?;
    let copy_writes = copy.writes();
    let check_reads = stack_operands_metered(check, check.bci(), meter)?;
    let check_writes = check.writes();
    let pop_reads = stack_operands_metered(pop, pop.bci(), meter)?;
    for _ in qualifier_writes {
        meter.charge(CountedBudgetDimension::IrItems, Some(qualifier.bci()))?;
    }
    for _ in copy_writes {
        meter.charge(CountedBudgetDimension::IrItems, Some(copy.bci()))?;
    }
    for _ in check_writes {
        meter.charge(CountedBudgetDimension::IrItems, Some(check.bci()))?;
    }
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
        || !single_use_at_metered(ssa, qualifier_writes[0].1, copy.bci(), meter)?
        || !single_use_at_metered(ssa, physical_outer, at, meter)?
        || !single_use_at_metered(ssa, check_reads[0].1, check.bci(), meter)?
        || !single_use_at_metered(ssa, check_writes[0].1, pop.bci(), meter)?
    {
        return Err(shape(format!(
            "the member constructor at BCI {at} does not pass the checked qualifier's two SSA copies as its physical outer and null-check operand exactly once"
        )));
    }
    if !names_outer(qualifier_writes[0].1) {
        return Err(shape(format!(
            "the qualifier at BCI {} has no exact static type `{}` for member binding (SSA type {:?})",
            qualifier.bci(),
            target.outer,
            ssa.value(qualifier_writes[0].1).ty()
        )));
    }

    // The Java qualified expression puts its check at this position. Every instruction it owns
    // must have the same handler coverage as the original check; a boundary cannot be crossed.
    let coverage = |bci: u32, meter: &mut VerifyMeter<'_>| -> Result<Vec<u32>, VerifyFailure> {
        let mut covered = Vec::new();
        for handler in &code.exception_handlers {
            meter.charge(CountedBudgetDimension::IrItems, Some(bci))?;
            if handler.start_bci <= bci && bci < handler.end_bci {
                covered.push(handler.ordinal);
            }
        }
        Ok(covered)
    };
    let expected_handlers = coverage(check.bci(), meter)?;
    for instruction in block[index..]
        .iter()
        .take_while(|instruction| instruction.bci() <= at)
    {
        meter.charge(
            CountedBudgetDimension::AnalysisSteps,
            Some(instruction.bci()),
        )?;
        if coverage(instruction.bci(), meter)? != expected_handlers {
            return Err(order(format!(
                "the member constructor at BCI {at} crosses an exception-handler boundary around its qualifier check"
            )));
        }
    }

    let mut arguments = vec![copy.bci()]; // physical first argument, not a source argument
    let mut last = pop.bci();
    for (_, value) in operands.iter().skip(2) {
        meter.charge(CountedBudgetDimension::IrItems, Some(at))?;
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
    let dependencies = value_dependency_bcis_metered(
        ssa,
        block,
        operands.iter().skip(2).map(|(_, value)| *value),
        meter,
    )?;
    let after_check = index + 6;
    for instruction in block
        .iter()
        .skip(after_check)
        .take_while(|instruction| instruction.bci() < at)
    {
        meter.charge(
            CountedBudgetDimension::AnalysisSteps,
            Some(instruction.bci()),
        )?;
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
            implicit_this: false,
        },
        arguments,
        owned,
    })
}

/// The ordinary source arguments of a member construction whose qualifier needs no null check —
/// the enclosing `this` itself, or a fresh allocation. The discipline is the checked qualifier's:
/// every ordinary argument is produced after the qualifier run and before the constructor, and
/// nothing else may stand in that window.
fn member_ordinary_arguments(
    start: usize,
    last: u32,
    at: u32,
    block: &[SsaInstruction],
    operands: &[(jarde_jvm::method_ir::Slot, ValueId)],
    ssa: &SsaTable,
    operations: &crate::decode::Operations,
    meter: &mut VerifyMeter<'_>,
) -> Result<Vec<u32>, VerifyFailure> {
    let order =
        |detail: String| VerifyFailure::Refusal(Refusal::shape("jre_new_member_order", detail));
    let mut arguments = Vec::new();
    let mut last = last;
    for (_, value) in operands.iter().skip(2) {
        meter.charge(CountedBudgetDimension::IrItems, Some(at))?;
        let Some(produced) = produced_at(ssa, *value) else {
            return Err(order(format!(
                "an ordinary argument of the member constructor at BCI {at} has no local producer"
            )));
        };
        if produced <= last || produced >= at {
            return Err(order(format!(
                "the ordinary argument at BCI {produced} is not produced in order after the qualifier at BCI {last} and before the constructor at BCI {at}"
            )));
        }
        arguments.push(produced);
        last = produced;
    }
    let dependencies = value_dependency_bcis_metered(
        ssa,
        block,
        operands.iter().skip(2).map(|(_, value)| *value),
        meter,
    )?;
    for instruction in block
        .iter()
        .skip(start)
        .take_while(|instruction| instruction.bci() < at)
    {
        meter.charge(
            CountedBudgetDimension::AnalysisSteps,
            Some(instruction.bci()),
        )?;
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
    Ok(arguments)
}

fn single_use_at(ssa: &SsaTable, value: ValueId, at: u32) -> bool {
    let uses = ssa.value(value).uses();
    uses.len() == 1 && uses[0].bci() == Some(at)
}

fn single_use_at_metered(
    ssa: &SsaTable,
    value: ValueId,
    at: u32,
    meter: &mut VerifyMeter<'_>,
) -> Result<bool, VerifyFailure> {
    let uses = ssa.value(value).uses();
    for usage in uses {
        meter.charge(CountedBudgetDimension::IrItems, usage.bci())?;
    }
    Ok(uses.len() == 1 && uses[0].bci() == Some(at))
}

fn stack_operands_metered(
    instruction: &SsaInstruction,
    at: u32,
    meter: &mut VerifyMeter<'_>,
) -> Result<Vec<(Slot, ValueId)>, VerifyFailure> {
    let mut operands = Vec::new();
    for (slot, value) in instruction.reads() {
        meter.charge(CountedBudgetDimension::IrItems, Some(at))?;
        if matches!(slot, Slot::Stack(_)) {
            operands.push((*slot, *value));
        }
    }
    operands.sort_by_key(|(slot, _)| match slot {
        Slot::Stack(depth) => *depth,
        Slot::Local(slot) => u32::from(*slot),
    });
    Ok(operands)
}

/// The discarded null-check tail spelled over a construction's finished instance, when there is
/// one: `dup; <discarded null check>; pop` as the three block instructions immediately after the
/// constructor call.
///
/// The `dup` must read the instance this site builds. The rest of the window's discipline is
/// [`discarded_null_check_window`]'s, which the bound-receiver tail of a dynamic site shares: the
/// two spellings of this shape differ only in the value the `dup` copies.
fn discarded_null_check_tail_metered(
    ssa: &SsaTable,
    operations: &Operations,
    block: &[SsaInstruction],
    constructor_index: usize,
    produced_by: &[u32],
    meter: &mut VerifyMeter<'_>,
) -> Result<Option<[u32; 3]>, VerifyFailure> {
    let Some([copy, check, pop]) = block.get(constructor_index + 1..constructor_index + 4) else {
        return Ok(None);
    };
    meter.charge(CountedBudgetDimension::AnalysisSteps, Some(copy.bci()))?;
    meter.charge(CountedBudgetDimension::AnalysisSteps, Some(check.bci()))?;
    meter.charge(CountedBudgetDimension::AnalysisSteps, Some(pop.bci()))?;
    if operations.get(copy.bci()) != Some(&Operation::Duplicate) {
        return Ok(None);
    }
    let mut guarded = false;
    for (_, value) in copy.reads() {
        meter.charge(CountedBudgetDimension::IrItems, Some(copy.bci()))?;
        if is_the_instance(ssa, *value, produced_by) {
            guarded = true;
            break;
        }
    }
    let Some(Operation::Invoke(call)) = operations.get(check.bci()) else {
        return Ok(None);
    };
    if !guarded
        || !crate::facts::is_discarded_null_check(
            call.kind(),
            call.owner().as_bytes(),
            call.name().as_bytes(),
            call.descriptor().as_bytes(),
            call.is_interface_reference(),
        )
    {
        return Ok(None);
    }
    let copy_writes: Vec<ValueId> = copy.writes().iter().map(|(_, value)| *value).collect();
    let check_reads = stack_operands_metered(check, check.bci(), meter)?;
    let check_writes = check.writes();
    let pop_reads = stack_operands_metered(pop, pop.bci(), meter)?;
    for _ in &copy_writes {
        meter.charge(CountedBudgetDimension::IrItems, Some(copy.bci()))?;
    }
    for _ in check_writes {
        meter.charge(CountedBudgetDimension::IrItems, Some(check.bci()))?;
    }
    let check_is_single = if let [(_, value)] = check_reads.as_slice() {
        single_use_at_metered(ssa, *value, check.bci(), meter)?
    } else {
        false
    };
    let pop_is_single = if let [(_, value)] = check_writes {
        single_use_at_metered(ssa, *value, pop.bci(), meter)?
    } else {
        false
    };
    if pop.opcode() != 0x57
        || check_reads.len() != 1
        || check_writes.len() != 1
        || pop_reads.len() != 1
        || !copy_writes.contains(&check_reads[0].1)
        || !check_is_single
        || pop_reads[0].1 != check_writes[0].1
        || !pop_is_single
    {
        return Ok(None);
    }
    Ok(Some([copy.bci(), check.bci(), pop.bci()]))
}

/// The three-instruction window `dup; <discarded null check>; pop`, when `copy` duplicates the value
/// the caller guards and every single-use link holds.
///
/// The check is one of the two spellings [`crate::facts::is_discarded_null_check`] states, and it
/// must consume one of the `dup`'s two writes **once**. The `pop` must be the category-1 discard and
/// must consume the check's result **once** — a kept result (stored, called on) is not this tail and
/// leaves the caller's refusal standing. The tail's contiguity is the same position discipline the
/// checked qualifier's `[qualifier, copy, check, pop]` window applies.
///
/// `reads_guarded` is the whole of what the two callers disagree about, and its answer is handed
/// back: a construction's tail guards the instance that site builds ([`is_the_instance`]) and needs
/// no more than the fact, and a dynamic site's tail guards the bound receiver value its own `dup`
/// copies ([`receiver_tails`]), which has to name the class the receiver's allocation builds.
fn discarded_null_check_window<Guarded>(
    ssa: &SsaTable,
    operations: &Operations,
    copy: &SsaInstruction,
    check: &SsaInstruction,
    pop: &SsaInstruction,
    reads_guarded: impl Fn(ValueId) -> Option<Guarded>,
) -> Option<([u32; 3], Guarded)> {
    if operations.get(copy.bci()) != Some(&Operation::Duplicate) {
        return None;
    }
    let guarded = copy
        .reads()
        .iter()
        .find_map(|(_, read)| reads_guarded(*read))?;
    let Some(Operation::Invoke(call)) = operations.get(check.bci()) else {
        return None;
    };
    if !crate::facts::is_discarded_null_check(
        call.kind(),
        call.owner().as_bytes(),
        call.name().as_bytes(),
        call.descriptor().as_bytes(),
        call.is_interface_reference(),
    ) {
        return None;
    }
    let copy_writes: Vec<ValueId> = copy.writes().iter().map(|(_, value)| *value).collect();
    let check_reads = stack_operands(check);
    let check_writes = check.writes();
    let pop_reads = stack_operands(pop);
    if pop.opcode() != 0x57
        || check_reads.len() != 1
        || check_writes.len() != 1
        || pop_reads.len() != 1
        || !copy_writes.contains(&check_reads[0].1)
        || !single_use_at(ssa, check_reads[0].1, check.bci())
        || pop_reads[0].1 != check_writes[0].1
        || !single_use_at(ssa, check_writes[0].1, pop.bci())
    {
        return None;
    }
    Some(([copy.bci(), check.bci(), pop.bci()], guarded))
}

/// One dynamic site whose bound receiver this run proves non-null, with the creation-time null check
/// javac wrote over that receiver.
///
/// The site's three tail instructions are owned by [`Sites`] exactly as a construction's are: the
/// check cannot fail, so the bytecode that performs it produces no statement of its own, and the
/// value the site captures is the receiver the `dup` copied.
pub(crate) struct ReceiverTail {
    /// The BCI of the `invokedynamic` site the tail belongs to.
    pub(crate) site: u32,
    /// The BCI of the `dup` that copies the receiver value.
    pub(crate) copy: u32,
    /// The BCI of the discarded null check.
    pub(crate) check: u32,
    /// The BCI of the `pop` that discards the check's result.
    pub(crate) pop: u32,
    /// The value the site captures: the receiver the `dup` copies.
    pub(crate) receiver: ValueId,
    /// The value the `dup` writes, which is the operand the site reads off the stack.
    pub(crate) copy_value: ValueId,
}

/// Every dynamic site whose creation-time receiver check this run proves dead.
///
/// javac evaluates a bound method reference (`receiver::name`) by checking the receiver for null
/// **while the functional value is created**, and writes that check as the same
/// `dup; <discarded null check>; pop` window [`discarded_null_check_tail`] reads over a finished
/// construction. The one difference is the value the `dup` copies: a construction's tail guards the
/// instance the site builds, and this one guards the **receiver value**, whose move chain ends at an
/// allocation ([`crate::build::allocation_behind_value`]) with no later store into the local that
/// chain read ([`proved_receiver_class`]). Such a check cannot fail, so the site's creation and its
/// invocation can no longer disagree about a null receiver — which is the whole of what the refusal
/// over a bound receiver states.
///
/// The window is read in **BCI order across canonical blocks**, not inside one block: the check may
/// throw, so the canonical CFG splits the block between the `dup` and the site, and a same-block
/// window was measured to miss every real site of this shape.
///
/// Two more conditions belong to the site itself: it must consume the copy the tail made (or the
/// three instructions are some other discarded check and the site's operand is another value), and
/// its own descriptor must name the class the allocation builds for that value. The second is the
/// frame/site half of the capture check the plan states three ways, and it is required here because
/// a claim made without it would take the site's own check away from a site the plan still refuses
/// (see [`captured_reference`]).
fn receiver_tails(ssa: &SsaTable, operations: &Operations) -> Vec<ReceiverTail> {
    let mut sequence: Vec<&SsaInstruction> = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .collect();
    sequence.sort_by_key(|instruction| instruction.bci());
    let mut tails = Vec::new();
    for (index, site) in sequence.iter().enumerate() {
        let Some(Operation::InvokeDynamic(dynamic)) = operations.get(site.bci()) else {
            continue;
        };
        let Some(window) = index
            .checked_sub(3)
            .and_then(|start| sequence.get(start..index))
        else {
            continue;
        };
        let [copy, check, pop] = window else {
            continue;
        };
        let Some(([copy_bci, check_bci, pop_bci], allocated)) =
            discarded_null_check_window(ssa, operations, copy, check, pop, |value| {
                proved_receiver_class(ssa, operations, value)
            })
        else {
            continue;
        };
        // The site's own descriptor must name the class the allocation builds for the value it
        // captures. A local declared as a supertype of what it holds (`List<String> out = new
        // ArrayList<>()`) is the case the plan's three-way capture check refuses — the frame says
        // `ArrayList`, the site says `List` — and there the site's own creation-time check must stay
        // quoted with the site: the claim would take that quote away from a body that keeps the
        // refusal, and a reader who strips the comments would be left with the refusal's statement
        // gone and the rest of the body presented.
        if captured_reference(dynamic.descriptor()) != Some(allocated) {
            continue;
        }
        let copy_writes: Vec<ValueId> = copy.writes().iter().map(|(_, value)| *value).collect();
        let reads = stack_operands(copy);
        let [receiver] = reads.as_slice() else {
            continue;
        };
        if !stack_operands(site)
            .iter()
            .any(|(_, value)| copy_writes.contains(value))
        {
            continue;
        }
        tails.push(ReceiverTail {
            site: site.bci(),
            copy: copy_bci,
            check: check_bci,
            pop: pop_bci,
            receiver: receiver.1,
            copy_value: copy_writes[0],
        });
    }
    tails
}

/// The class the allocation one bound receiver's value chain ends at builds, when the value is
/// provably non-null where the site captures it: its move chain ends at an allocation, and the local
/// that chain read is not written again after the read.
///
/// The second half is what makes the value the *bytecode* captured and the slot a reader of the
/// written text re-reads the same object: the site's text names the local, and a store after the
/// read would put another object in that slot before the site's own invocation.
pub(crate) fn proved_receiver_class(
    ssa: &SsaTable,
    operations: &Operations,
    value: ValueId,
) -> Option<String> {
    let behind = crate::build::allocation_behind_value(ssa, operations, value)?;
    if let Some((slot, read_at)) = behind.read
        && operations.iter().any(|(bci, operation)| {
            *bci > read_at
                && matches!(
                    operation,
                    Operation::Store { slot: written } if *written == slot
                )
        })
    {
        return None;
    }
    Some(behind.class)
}

/// The internal name of the reference type one dynamic site's descriptor names for the first value
/// it captures, when it names one.
///
/// The descriptor is read by the reader's own reading
/// ([`jarde_reader::classfile::descriptor_facts`]) — the same one
/// [`crate::build::array_descriptor`] uses — so the bytes are not walked a second time by hand, and
/// the position is the descriptor's own: the values a site captures come before the SAM's
/// parameters.
fn captured_reference(descriptor: &str) -> Option<String> {
    let facts = jarde_reader::classfile::descriptor_facts(
        descriptor.as_bytes(),
        jarde_reader::classfile::DescriptorKind::Method,
    )
    .ok()?;
    let first = facts.parameters().first()?;
    match first.base() {
        Base::Object(name) => String::from_utf8(name.0.clone()).ok(),
        Base::Primitive(_) => None,
    }
}

/// The complete concat chains whose values are direct, unique constructor arguments.
///
/// `reserved` membership alone cannot establish any of these facts: this reads the specific
/// `concat@1` chain at each argument's producer and then checks its identity, ownership, position,
/// and physical completeness before the outer verifier may step over an allocation.
#[allow(clippy::too_many_arguments)]
#[cfg(test)]
fn verify_concat_arguments(
    head: u32,
    dup: u32,
    constructor: u32,
    block: &[SsaInstruction],
    ssa: &SsaTable,
    chains: &crate::concat::Plan,
    arguments: impl IntoIterator<Item = ValueId>,
    dependencies: &BTreeSet<u32>,
) -> Result<BTreeSet<u32>, Refusal> {
    let mut meter = VerifyMeter::unmetered();
    match verify_concat_arguments_metered(
        head,
        dup,
        constructor,
        block,
        ssa,
        chains,
        arguments,
        dependencies,
        &mut meter,
    ) {
        Ok(value) => Ok(value),
        Err(VerifyFailure::Refusal(refusal)) => Err(refusal),
        Err(VerifyFailure::Stop(_)) => unreachable!("an unmetered concat check cannot stop"),
    }
}

fn verify_concat_arguments_metered(
    head: u32,
    dup: u32,
    constructor: u32,
    block: &[SsaInstruction],
    ssa: &SsaTable,
    chains: &crate::concat::Plan,
    arguments: impl IntoIterator<Item = ValueId>,
    dependencies: &BTreeSet<u32>,
    meter: &mut VerifyMeter<'_>,
) -> Result<BTreeSet<u32>, VerifyFailure> {
    let reject = |detail: String| {
        VerifyFailure::Refusal(Refusal::shape(
            "jre_new_concat_argument",
            format!("the construction at BCI {head} {detail}"),
        ))
    };
    let mut nested = BTreeSet::new();
    let mut selected = BTreeSet::new();
    for value in arguments {
        meter.charge(CountedBudgetDimension::IrItems, Some(head))?;
        let Some(tail) = produced_at(ssa, value) else {
            continue;
        };
        let Some(chain) = chains.value_at(tail) else {
            continue;
        };
        if !selected.insert(chain.tail) {
            return Err(reject(format!(
                "uses the concatenation ending at BCI {} more than once as constructor arguments",
                chain.tail
            )));
        }
        let Some(result) = stack_value_written_at(ssa, chain.tail) else {
            return Err(reject(format!(
                "cannot identify the string value written by the concatenation ending at BCI {}",
                chain.tail
            )));
        };
        if value != result || !single_use_at_metered(ssa, result, constructor, meter)? {
            return Err(reject(format!(
                "uses the concatenation result at BCI {} outside its one constructor argument at BCI {constructor}",
                chain.tail
            )));
        }
        if chain.head <= dup || chain.tail >= constructor {
            return Err(reject(format!(
                "has a concatenation from BCI {} through {} outside the physical argument interval after dup BCI {dup} and before constructor BCI {constructor}",
                chain.head, chain.tail
            )));
        }
        let mut first = None;
        for (position, instruction) in block.iter().enumerate() {
            meter.charge(
                CountedBudgetDimension::AnalysisSteps,
                Some(instruction.bci()),
            )?;
            if instruction.bci() == chain.head {
                first = Some(position);
                break;
            }
        }
        let Some(first) = first else {
            return Err(reject(format!(
                "has a concatenation beginning at BCI {} outside the construction block",
                chain.head
            )));
        };
        let mut last = None;
        for (position, instruction) in block.iter().enumerate() {
            meter.charge(
                CountedBudgetDimension::AnalysisSteps,
                Some(instruction.bci()),
            )?;
            if instruction.bci() == chain.tail {
                last = Some(position);
                break;
            }
        }
        let Some(last) = last else {
            return Err(reject(format!(
                "has a concatenation ending at BCI {} outside the construction block",
                chain.tail
            )));
        };
        let mut members = BTreeSet::new();
        for instruction in &block[first..=last] {
            meter.charge(
                CountedBudgetDimension::AnalysisSteps,
                Some(instruction.bci()),
            )?;
            members.insert(instruction.bci());
        }
        for bci in &members {
            meter.charge(CountedBudgetDimension::IrItems, Some(*bci))?;
        }
        for bci in chain.owned.iter() {
            meter.charge(CountedBudgetDimension::IrItems, Some(*bci))?;
        }
        if first >= last
            || chain.owned != members
            || !members.is_subset(chains.owned())
            || !members.is_subset(dependencies)
        {
            return Err(reject(format!(
                "does not depend on the complete, uniquely owned concatenation interval at BCIs {} through {}",
                chain.head, chain.tail
            )));
        }
        nested.extend(members);
    }
    Ok(nested)
}

fn stack_value_written_at(ssa: &SsaTable, bci: u32) -> Option<ValueId> {
    ssa.blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == bci)
        .and_then(|instruction| {
            instruction
                .writes()
                .iter()
                .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
        })
}

/// Every instruction outside a site that reads one of the values the site produced.
///
/// The walk is the body's own instructions, in block order: the site's own three instructions are
/// left out by BCI, and everything else is read from the table the caller already holds.
fn outside_readers(
    ssa: &SsaTable,
    block: &[SsaInstruction],
    produced_by: &[u32],
    meter: &mut VerifyMeter<'_>,
) -> Result<Vec<u32>, VerifyFailure> {
    let mut readers: Vec<u32> = Vec::new();
    for producer in block {
        meter.charge(CountedBudgetDimension::AnalysisSteps, Some(producer.bci()))?;
        if !produced_by.contains(&producer.bci()) {
            continue;
        }
        for (_, value) in producer.writes() {
            meter.charge(CountedBudgetDimension::IrItems, Some(producer.bci()))?;
            for usage in ssa.value(*value).uses() {
                meter.charge(CountedBudgetDimension::IrItems, usage.bci())?;
                let Some(bci) = usage.bci() else {
                    continue;
                };
                if !produced_by.contains(&bci) && !readers.contains(&bci) {
                    readers.push(bci);
                }
            }
        }
    }
    readers.sort_unstable();
    Ok(readers)
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

/// The composed path accepts only the instance copy the verified new/dup/init run actually leaves
/// for this one array store. Site.instance is a source-BCI set, so it is not sufficient for this
/// ValueId identity proof.
fn array_store_consumes_site_value(
    ssa: &SsaTable,
    operations: &Operations,
    block: &[SsaInstruction],
    head: u32,
    duplicate: u32,
    constructor: u32,
    store: u32,
    stored: ValueId,
    class: &str,
    meter: &mut VerifyMeter<'_>,
) -> Result<bool, VerifyFailure> {
    for bci in [head, duplicate, constructor, store] {
        meter.charge(CountedBudgetDimension::AnalysisSteps, Some(bci))?;
    }
    let Some(allocation) = instruction_in_block_by_bci_metered(block, head, meter)? else {
        return Ok(false);
    };
    let Some(duplicate_instruction) = instruction_in_block_by_bci_metered(block, duplicate, meter)?
    else {
        return Ok(false);
    };
    let Some(constructor_instruction) =
        instruction_in_block_by_bci_metered(block, constructor, meter)?
    else {
        return Ok(false);
    };
    let Some(store_instruction) = instruction_in_block_by_bci_metered(block, store, meter)? else {
        return Ok(false);
    };
    let store_operands = stack_operands_metered(store_instruction, store, meter)?;
    let [(_, _array), (_, _index), (_, store_value)] = store_operands.as_slice() else {
        return Ok(false);
    };
    if duplicate_instruction.opcode() != 0x59
        || !matches!(operations.get(store), Some(Operation::ArrayStore { .. }))
        || *store_value != stored
    {
        return Ok(false);
    }
    let mut allocation_outputs = Vec::new();
    for (slot, value) in allocation.writes() {
        meter.charge(CountedBudgetDimension::IrItems, Some(head))?;
        if matches!(slot, Slot::Stack(_)) {
            allocation_outputs.push((*slot, *value));
        }
    }
    let mut duplicate_reads = Vec::new();
    for (slot, value) in duplicate_instruction.reads() {
        meter.charge(CountedBudgetDimension::IrItems, Some(duplicate))?;
        if matches!(slot, Slot::Stack(_)) {
            duplicate_reads.push((*slot, *value));
        }
    }
    let mut duplicate_outputs = Vec::new();
    for (slot, value) in duplicate_instruction.writes() {
        meter.charge(CountedBudgetDimension::IrItems, Some(duplicate))?;
        if matches!(slot, Slot::Stack(_)) {
            duplicate_outputs.push((*slot, *value));
        }
    }
    if allocation_outputs.len() != 1
        || duplicate_reads.as_slice() != allocation_outputs.as_slice()
        || duplicate_outputs.len() != 2
        || duplicate_outputs[0].0 == duplicate_outputs[1].0
        || duplicate_outputs[0].1 == duplicate_outputs[1].1
    {
        return Ok(false);
    }
    let [(Slot::Stack(_), allocation_value)] = allocation_outputs.as_slice() else {
        return Ok(false);
    };
    let Value::Uninitialized { new_site } = ssa.value(*allocation_value).ty() else {
        return Ok(false);
    };
    let Definition::Instruction {
        block: allocation_block,
        bci: allocation_bci,
    } = ssa.value(*allocation_value).def()
    else {
        return Ok(false);
    };
    if new_site.bci() != head || new_site.block() != allocation_block || *allocation_bci != head {
        return Ok(false);
    }
    for (_, value) in &duplicate_outputs {
        meter.charge(CountedBudgetDimension::IrItems, Some(duplicate))?;
        if ssa.value(*value).ty() != ssa.value(*allocation_value).ty() {
            return Ok(false);
        }
    }
    let constructor_operands = stack_operands_metered(constructor_instruction, constructor, meter)?;
    let Some((receiver_slot @ Slot::Stack(_), receiver)) = constructor_operands.first().copied()
    else {
        return Ok(false);
    };
    let mut receiver_output = false;
    let mut surviving = None;
    for (slot, value) in &duplicate_outputs {
        meter.charge(CountedBudgetDimension::IrItems, Some(duplicate))?;
        if (*slot, *value) == (receiver_slot, receiver) {
            receiver_output = true;
        }
        if *slot != receiver_slot {
            surviving = Some((*slot, *value));
        }
    }
    if !receiver_output {
        return Ok(false);
    }
    let Some((surviving_slot @ Slot::Stack(_), _)) = surviving else {
        return Ok(false);
    };
    let mut constructor_outputs = Vec::new();
    for (slot, value) in constructor_instruction.writes() {
        meter.charge(CountedBudgetDimension::IrItems, Some(constructor))?;
        if matches!(slot, Slot::Stack(_)) {
            constructor_outputs.push((*slot, *value));
        }
    }
    let completed: Vec<ValueId> = constructor_outputs
        .iter()
        .filter_map(|(slot, value)| (*slot == surviving_slot).then_some(*value))
        .collect();
    let [completed] = completed.as_slice() else {
        return Ok(false);
    };
    let class_matches = matches!(
        ssa.value(*completed).ty(),
        Value::Ref(RefType::Named { name, .. }) if name == class.as_bytes()
    );
    let mut only_store_use = true;
    let stored_uses = ssa.value(stored).uses();
    let Definition::Instruction {
        block: stored_block,
        bci: stored_bci,
    } = ssa.value(stored).def()
    else {
        return Ok(false);
    };
    if *stored_bci != constructor || stored_block != allocation_block {
        return Ok(false);
    }
    for usage in stored_uses {
        meter.charge(CountedBudgetDimension::IrItems, usage.bci())?;
        if usage.block() != stored_block || usage.bci() != Some(store) {
            only_store_use = false;
        }
    }
    Ok(*completed == stored && class_matches && stored_uses.len() == 1 && only_store_use)
}

fn instruction_in_block_by_bci_metered<'a>(
    block: &'a [SsaInstruction],
    bci: u32,
    meter: &mut VerifyMeter<'_>,
) -> Result<Option<&'a SsaInstruction>, VerifyFailure> {
    for instruction in block {
        meter.charge(
            CountedBudgetDimension::AnalysisSteps,
            Some(instruction.bci()),
        )?;
        if instruction.bci() == bci {
            return Ok(Some(instruction));
        }
    }
    Ok(None)
}

/// Whether the instruction at `pop` is the category-1 discard of one construction's finished
/// instance: the statement position's reader.
///
/// Every fact is an identity the bytecode states rather than a guess from the sequence, and they are
/// the ones the builder's own discard plan reads (P3 2c.31): the instruction is the `pop` — the
/// decode states no operation for it, so the opcode is read from the instruction itself — and it is
/// the block instruction **immediately after** the constructor call, so nothing runs between the
/// construction and the discard. It reads exactly one value, that value is the one the constructor
/// call itself wrote — the finished instance, not a copy some slot took — and nothing else reads
/// it: the discard is the whole of what the body did with the instance.
fn discards_the_instance(
    ssa: &SsaTable,
    block: &[SsaInstruction],
    constructor: u32,
    pop: u32,
) -> bool {
    let Some(index) = block
        .iter()
        .position(|instruction| instruction.bci() == constructor)
    else {
        return false;
    };
    let Some(instruction) = block.get(index + 1).filter(|next| next.bci() == pop) else {
        return false;
    };
    if instruction.opcode() != 0x57 {
        return false;
    }
    let reads = stack_operands(instruction);
    let [read] = reads.as_slice() else {
        return false;
    };
    produced_at(ssa, read.1) == Some(constructor) && single_use_at(ssa, read.1, pop)
}

/// Whether every argument of one construction is a value the `new` expression writes **in place**
/// with no invocation of its own.
///
/// Three producers are admitted, and they are the whole of the criterion
/// (`recover-statement-position-news`):
///
/// * a constant the allocation's own argument run pushes ([`Operation::Push`]) — the literal is
///   written where the bytecode pushed it;
/// * a direct local or parameter read ([`Operation::Load`]) whose read value is the slot's own
///   entry state ([`Definition::Entry`]) — the name denotes the value the bytecode read, because no
///   instruction of this body wrote that slot before the read;
/// * the completed instance of a construction this same proof stepped over (`nested_sites`) — the
///   nested `new` expression is written in the argument position, which is where the bytecode
///   evaluated it.
///
/// Every other producer — an invocation, a field read, an arithmetic or conversion chain, a merge —
/// keeps the construction's refusal: the statement position admits no call of its own, so a
/// `new X(args);` whose argument evaluates one is not a statement this rule states. The check reads
/// the physical arguments the constructor call takes, which is the same list the written expression
/// keeps.
fn arguments_without_invocations(
    ssa: &SsaTable,
    operations: &Operations,
    operands: &[(jarde_jvm::method_ir::Slot, ValueId)],
    nested_sites: &[Site],
) -> bool {
    operands.iter().skip(1).all(|(_, value)| {
        nested_sites
            .iter()
            .any(|nested| is_the_instance(ssa, *value, nested.instance.as_slice()))
            || match produced_at(ssa, *value) {
                Some(bci) => match operations.get(bci) {
                    Some(Operation::Push(_)) => true,
                    Some(Operation::Load { .. }) => direct_read(ssa, instruction_at(ssa, bci)),
                    _ => false,
                },
                None => false,
            }
    })
}

/// The instruction of one body at `bci`, when this body states one.
fn instruction_at(ssa: &SsaTable, bci: u32) -> Option<&SsaInstruction> {
    ssa.blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == bci)
}

/// Whether one `Load` reads a slot's own entry state — the value nothing in this body wrote.
///
/// A load's operand is a **local** slot, not a stack value: the read the instruction states is the
/// value that slot holds where the load runs, and [`Definition::Entry`] is the SSA's own statement
/// that no instruction of this body produced it (a parameter, `this`, or a local nothing wrote
/// yet). A load of a slot some store filled reads that store's value instead, and the name would
/// denote a value this body computed rather than the one the constructor argument is.
fn direct_read(ssa: &SsaTable, instruction: Option<&SsaInstruction>) -> bool {
    let Some(instruction) = instruction else {
        return false;
    };
    let reads: Vec<ValueId> = instruction
        .reads()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Local(_)))
        .map(|(_, value)| *value)
        .collect();
    let [read] = reads.as_slice() else {
        return false;
    };
    matches!(ssa.value(*read).def(), Definition::Entry { .. })
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
#[cfg(test)]
fn value_dependency_bcis(
    ssa: &SsaTable,
    block: &[SsaInstruction],
    roots: impl IntoIterator<Item = ValueId>,
) -> BTreeSet<u32> {
    let mut meter = VerifyMeter { budget: None };
    value_dependency_bcis_metered(ssa, block, roots, &mut meter)
        .expect("an unmetered dependency walk cannot stop")
}

fn value_dependency_bcis_metered(
    ssa: &SsaTable,
    block: &[SsaInstruction],
    roots: impl IntoIterator<Item = ValueId>,
    meter: &mut VerifyMeter<'_>,
) -> Result<BTreeSet<u32>, VerifyFailure> {
    let mut instructions = std::collections::BTreeMap::new();
    for instruction in block {
        meter.charge(
            CountedBudgetDimension::AnalysisSteps,
            Some(instruction.bci()),
        )?;
        instructions.insert(instruction.bci(), instruction);
    }
    let mut pending: Vec<ValueId> = roots.into_iter().collect();
    let mut visited = BTreeSet::new();
    let mut dependencies = BTreeSet::new();

    while let Some(value) = pending.pop() {
        meter.charge(CountedBudgetDimension::IrItems, produced_at(ssa, value))?;
        if !visited.insert(value) {
            continue;
        }
        match ssa.value(value).def() {
            Definition::Instruction { bci, .. } => {
                if let Some(instruction) = instructions.get(bci) {
                    dependencies.insert(*bci);
                    for (_, read) in instruction.reads() {
                        meter.charge(CountedBudgetDimension::IrItems, Some(*bci))?;
                        pending.push(*read);
                    }
                }
            }
            Definition::Phi { .. } => {
                let mut found = None;
                for phi in ssa.phis() {
                    meter.charge(CountedBudgetDimension::IrItems, None)?;
                    if phi.value() == value {
                        found = Some(phi);
                        break;
                    }
                }
                if let Some(phi) = found {
                    for input in phi.inputs() {
                        meter.charge(CountedBudgetDimension::IrItems, None)?;
                        if let jarde_jvm::method_ir::PhiInput::Value(input) = input {
                            pending.push(*input);
                        }
                    }
                }
            }
            Definition::Entry { .. } | Definition::Caught { .. } => {}
        }
    }
    Ok(dependencies)
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
    const CONCAT_CONSTRUCTOR: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-26/exception-constructor-concat/original-classes/Probe.class"
    );
    const INLINE_CHAR_ARRAY: &[u8] =
        include_bytes!("../../../tests/fixtures/em27-inline-string/em27/Probe.class");
    const VARARGS_CTOR_W3: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/fixture/W3.class"
    );
    const VARARGS_V1: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/variants-vca/V1.class"
    );
    const VARARGS_V2: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/variants-vca/V2.class"
    );
    const NESTED_PROBE: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/fixture/X2.class"
    );
    const NESTED_VARIANTS: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/variants-nested/X3.class"
    );
    const NESTED_NEGATIVES: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/variants-nested/X4.class"
    );
    /// This change's own depth fixture (`tests/fixtures/recover-io-resource-finally/NestedDepth.java`,
    /// javac 23.0.1 `--release 8 -g:none`): `threeLayer` is the wrapped-stream chain's own depth and
    /// `fourLayer` is the boundary one layer deeper.
    const NESTED_DEPTH: &[u8] =
        include_bytes!("../../../tests/fixtures/recover-io-resource-finally/v8/NestedDepth.class");
    const FUNCTIONAL_CONSTRUCTORS_JAVAC8: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-10-08/lambda-constructor-arguments/v8-javac8/FunctionalConstructors.class"
    );
    const FUNCTIONAL_CONSTRUCTORS_JAVAC23: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-10-08/lambda-constructor-arguments/v8/FunctionalConstructors.class"
    );
    const PRIMITIVE_CONVERSION_CONTROLS_JAVAC8: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-constructor-primitive-conversion-arguments-v1/javac8/classes/ConstructorPrimitiveConversionControls.class"
    );
    const PRIMITIVE_CONVERSION_CONTROLS_JAVAC23: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-constructor-primitive-conversion-arguments-v1/javac23/classes/ConstructorPrimitiveConversionControls.class"
    );
    const PRIMITIVE_HANDLER_CONTROLS_JAVAC8: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-constructor-primitive-conversion-controls-v1/javac8/classes/ConstructorPrimitiveHandlerControls.class"
    );
    const PRIMITIVE_HANDLER_CONTROLS_JAVAC23: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-constructor-primitive-conversion-controls-v1/javac23/classes/ConstructorPrimitiveHandlerControls.class"
    );

    fn inline_char_sites(
        name: &str,
        descriptor: &str,
        java_release: u16,
        handler_range: Option<(u32, u32)>,
    ) -> Sites {
        let (analysis, _) = analyzed_caller(INLINE_CHAR_ARRAY, name, descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
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
        let operations = Operations::of(&code, ir.constant_pool());
        let fields = field::Plan::empty();
        let mut budget = proof_budget();
        let arrays = crate::build::ArrayInitializers::prove(ssa, &operations, &fields, &mut budget)
            .expect("array proof completes");
        let chains = crate::concat::Plan::empty();
        let method_facts = crate::facts::MethodFacts::new(name, descriptor, 0);
        sites(
            ssa,
            &operations,
            &chains,
            chains.owned(),
            &fields,
            &arrays,
            java_release,
            &[],
            &method_facts,
            &code,
        )
    }

    #[test]
    fn only_the_complete_direct_java8_string_char_array_is_embedded() {
        let direct = inline_char_sites("direct", "()Ljava/lang/String;", 8, None);
        let site = direct.site_at_head(0).expect("direct char[] String site");
        assert_eq!((site.head, site.dup, site.constructor), (0, 3, 22));
        assert_eq!(site.arguments, [17]);
        assert_eq!(site.expression.iter().copied().min(), Some(0));
        assert_eq!(site.expression.iter().copied().max(), Some(22));
        assert!(site.expression.contains(&5) && site.expression.contains(&21));
        assert!(site.owned.contains(&0) && site.owned.contains(&3) && site.owned.contains(&22));

        let stored = inline_char_sites("stored", "()Ljava/lang/String;", 8, None);
        assert_eq!(
            stored
                .site_at_head(19)
                .expect("stored array control")
                .constructor,
            24
        );
        let written = inline_char_sites("extraWrite", "()Ljava/lang/String;", 8, None);
        assert_eq!(
            written
                .site_at_head(24)
                .expect("separate mutation control")
                .constructor,
            29
        );
        let read = inline_char_sites("secondConsumer", "()Ljava/lang/String;", 8, None);
        assert_eq!(
            read.site_at_head(23)
                .expect("separate read control")
                .constructor,
            28
        );

        for (name, descriptor) in [
            ("wrongDescriptor", "()Ljava/lang/String;"),
            ("wrongOwner", "()Lem27/Probe$Holder;"),
            ("effectful", "()Ljava/lang/String;"),
            ("extraReader", "()Ljava/lang/String;"),
        ] {
            let refused = inline_char_sites(name, descriptor, 8, None);
            assert!(refused.site_at_head(0).is_none(), "{name} must not embed");
            assert_eq!(
                refused.refusals().next().expect("physical refusal").code(),
                "jre_new_interleaved_effect",
                "{name}"
            );
        }
        let old_profile = inline_char_sites("direct", "()Ljava/lang/String;", 7, None);
        assert!(old_profile.site_at_head(0).is_none());
        let crossed = inline_char_sites("direct", "()Ljava/lang/String;", 8, Some((5, 22)));
        assert!(crossed.site_at_head(0).is_none());
        assert_eq!(
            crossed.refusals().next().expect("handler refusal").code(),
            "jre_new_inline_char_array_exception_boundary"
        );
    }

    /// The change's own frozen fixtures: the Optional patrol's `OP` (whose `sideEffect` captures a
    /// `new StringBuilder()` through the site's own discarded null check) and this change's `BRN`
    /// with its three negatives, compiled on both javac legs.
    const BOUND_RECEIVER_ANCHOR: &[u8] = include_bytes!(
        "../../../tests/fixtures/recover-proved-nonnull-bound-receivers/v8/OP.class"
    );
    const BOUND_RECEIVER_ANCHOR_JAVAC8: &[u8] = include_bytes!(
        "../../../tests/fixtures/recover-proved-nonnull-bound-receivers/v8-javac8/OP.class"
    );
    const BOUND_RECEIVER_NEGATIVES: &[u8] = include_bytes!(
        "../../../tests/fixtures/recover-proved-nonnull-bound-receivers/v8/BRN.class"
    );
    const BOUND_RECEIVER_NEGATIVES_JAVAC8: &[u8] = include_bytes!(
        "../../../tests/fixtures/recover-proved-nonnull-bound-receivers/v8-javac8/BRN.class"
    );

    /// The fourth shape the claim must not take: a proved receiver whose site names **another** type
    /// than the class the allocation builds, because the local is declared as a supertype of what it
    /// holds (`List<String> out = new ArrayList<>(); … forEach(out::add)`). The plan's three-way
    /// capture check refuses that site, so its own check stays quoted with it.
    const BOUND_RECEIVER_WIDENED: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-10-04/realistic-class-combination-patrol/fixture/C1.class"
    );

    /// One method's own `Sites` plan, over the fixture's own class file.
    fn sites_of(class: &[u8], name: &str, descriptor: &str) -> Sites {
        let (analysis, _) = analyzed_caller(class, name, descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        let chains = crate::concat::Plan::empty();
        let fields = field::Plan::empty();
        let mut budget = proof_budget();
        let arrays = crate::build::ArrayInitializers::prove(ssa, &operations, &fields, &mut budget)
            .expect("array proof completes");
        sites(
            ssa,
            &operations,
            &chains,
            chains.owned(),
            &fields,
            &arrays,
            8,
            &[],
            &crate::facts::MethodFacts::new(name, descriptor, 0),
            code,
        )
    }

    fn sites_of_with_handler(
        class: &[u8],
        name: &str,
        descriptor: &str,
        handler_range: (u32, u32),
    ) -> Sites {
        let (analysis, _) = analyzed_caller(class, name, descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let mut code = ir.code().expect("code").clone();
        code.exception_handlers
            .push(jarde_reader::classfile::ExceptionHandlerFact {
                ordinal: u32::try_from(code.exception_handlers.len()).unwrap(),
                start_bci: handler_range.0,
                end_bci: handler_range.1,
                handler_bci: 0,
                catch_type_index: None,
            });
        let operations = Operations::of(&code, ir.constant_pool());
        let fields = field::Plan::empty();
        let chains = crate::concat::Plan::empty();
        let arrays = crate::build::ArrayInitializers::default();
        sites(
            ssa,
            &operations,
            &chains,
            chains.owned(),
            &fields,
            &arrays,
            8,
            &[],
            &crate::facts::MethodFacts::new(name, descriptor, 0),
            &code,
        )
    }

    #[test]
    fn functional_constructor_arguments_are_in_the_verified_dependency_run() {
        for (class, leg) in [
            (FUNCTIONAL_CONSTRUCTORS_JAVAC8, "javac 8"),
            (FUNCTIONAL_CONSTRUCTORS_JAVAC23, "javac 23 --release 8"),
        ] {
            for (name, descriptor, head, dynamic, constructor) in [
                ("runnable", "()Ljava/lang/Thread;", 0, 4, 9),
                ("captured", "(I)Ljava/lang/Thread;", 0, 5, 10),
                ("comparator", "()Ljava/util/PriorityQueue;", 0, 4, 9),
                ("reference", "()Ljava/util/PriorityQueue;", 0, 4, 9),
                ("callable", "()Ljava/util/concurrent/FutureTask;", 0, 4, 9),
                ("primitive", "(I)LIntBox;", 0, 5, 10),
                ("primitiveReference", "()LIntBox;", 0, 4, 9),
                ("ordered", "(I)Ljava/lang/Thread;", 0, 5, 13),
                ("overload", "()Ljava/lang/Thread;", 0, 4, 9),
            ] {
                let plan = sites_of(class, name, descriptor);
                let site = plan
                    .site_at_head(head)
                    .unwrap_or_else(|| panic!("{leg} {name} constructor proves"));
                assert_eq!(site.constructor, constructor, "{leg} {name}");
                assert!(site.expression.contains(&dynamic), "{leg} {name}");
                assert!(plan.refusals().next().is_none(), "{leg} {name}");
            }
        }
    }

    #[test]
    fn ordinary_constructor_conversion_uses_the_shared_census_budget() {
        for (class, leg) in [
            (PRIMITIVE_CONVERSION_CONTROLS_JAVAC8, "javac 8"),
            (
                PRIMITIVE_CONVERSION_CONTROLS_JAVAC23,
                "javac 23 --release 8",
            ),
        ] {
            let name = "wrapperByte";
            let descriptor = "(I)Ljava/lang/Byte;";
            let (analysis, _) = analyzed_caller(class, name, descriptor);
            let ir = analysis.ir();
            let ssa = ir.ssa().expect("ssa");
            let code = ir.code().expect("code");
            let operations = Operations::of(code, ir.constant_pool());
            let fields = field::Plan::empty();
            let chains = crate::concat::Plan::empty();
            let method = crate::facts::MethodFacts::new(name, descriptor, 0);

            let mut arrays = crate::build::ArrayInitializers::default();
            let plan = sites_after_array_composition(
                ssa,
                &operations,
                &chains,
                chains.owned(),
                &fields,
                &mut arrays,
                8,
                &[],
                &method,
                code,
                &mut proof_budget(),
            )
            .unwrap_or_else(|stop| panic!("{leg}: conversion census completes: {stop:?}"));
            let site = plan
                .site_at_head(0)
                .unwrap_or_else(|| panic!("{leg}: Byte construction is a Site"));
            let conversion = site
                .arguments
                .iter()
                .copied()
                .find(|bci| {
                    matches!(
                        operations.get(*bci),
                        Some(Operation::PrimitiveConversion {
                            source: crate::Type::Int,
                            target: crate::Type::Byte
                        })
                    )
                })
                .unwrap_or_else(|| panic!("{leg}: physical Byte argument is its i2b result"));
            assert!(site.expression.contains(&conversion), "{leg}");
            assert!(plan.refusals().next().is_none(), "{leg}");

            // The production census charges the physical ordinary-construction walk itself.
            // Its Result cannot expose a partially filled Sites plan when that walk stops.
            let mut arrays = crate::build::ArrayInitializers::default();
            let mut limits = proof_budget().limits().clone();
            limits.analysis_steps = 1;
            let mut limited = Budget::new(limits);
            assert!(matches!(
                sites_after_array_composition(
                    ssa,
                    &operations,
                    &chains,
                    chains.owned(),
                    &fields,
                    &mut arrays,
                    8,
                    &[],
                    &method,
                    code,
                    &mut limited,
                ),
                Err(crate::stop::StopReason::Budget {
                    dimension: CountedBudgetDimension::AnalysisSteps,
                    at: Some(4),
                    ..
                })
            ));

            let mut arrays = crate::build::ArrayInitializers::default();
            let mut cancelled = proof_budget();
            cancelled.cancellation_token().cancel();
            assert!(matches!(
                sites_after_array_composition(
                    ssa,
                    &operations,
                    &chains,
                    chains.owned(),
                    &fields,
                    &mut arrays,
                    8,
                    &[],
                    &method,
                    code,
                    &mut cancelled,
                ),
                Err(crate::stop::StopReason::Cancelled { at: Some(0) })
            ));
        }
    }

    #[test]
    fn constructor_wrapper_casts_and_stored_local_reuse_keep_their_value_chains() {
        for (class, leg) in [
            (PRIMITIVE_CONVERSION_CONTROLS_JAVAC8, "javac 8"),
            (
                PRIMITIVE_CONVERSION_CONTROLS_JAVAC23,
                "javac 23 --release 8",
            ),
        ] {
            for (name, descriptor, source, target) in [
                (
                    "wrapperByte",
                    "(I)Ljava/lang/Byte;",
                    crate::Type::Int,
                    crate::Type::Byte,
                ),
                (
                    "wrapperShort",
                    "(I)Ljava/lang/Short;",
                    crate::Type::Int,
                    crate::Type::Short,
                ),
                (
                    "wrapperLong",
                    "(I)Ljava/lang/Long;",
                    crate::Type::Int,
                    crate::Type::Long,
                ),
                (
                    "wrapperFloat",
                    "(J)Ljava/lang/Float;",
                    crate::Type::Long,
                    crate::Type::Float,
                ),
                (
                    "wrapperDouble",
                    "(F)Ljava/lang/Double;",
                    crate::Type::Float,
                    crate::Type::Double,
                ),
                (
                    "ordinaryReturnNew",
                    "(I)Ljava/lang/Long;",
                    crate::Type::Int,
                    crate::Type::Long,
                ),
            ] {
                let plan = sites_of(class, name, descriptor);
                let site = plan
                    .site_at_head(0)
                    .unwrap_or_else(|| panic!("{leg}/{name}: exact constructor argument cast"));
                assert_eq!(site.arguments.len(), 1, "{leg}/{name}");
                let (analysis, _) = analyzed_caller(class, name, descriptor);
                let ir = analysis.ir();
                let operations = Operations::of(ir.code().expect("code"), ir.constant_pool());
                assert!(
                    matches!(
                        operations.get(site.arguments[0]),
                        Some(Operation::PrimitiveConversion { source: actual_source, target: actual_target })
                            if actual_source == &source && actual_target == &target
                    ),
                    "{leg}/{name}: argument anchor is the exact decoded conversion"
                );
                assert!(plan.refusals().next().is_none(), "{leg}/{name}");
            }

            let stored = sites_of(class, "storedLocalReuse", "(I)LPrimitiveLongPair;");
            let site = stored
                .site_at_head(7)
                .unwrap_or_else(|| panic!("{leg}: local-backed arguments are still a Site"));
            assert_eq!(site.arguments, [12, 14], "{leg}: two converted arguments");
            let (analysis, _) =
                analyzed_caller(class, "storedLocalReuse", "(I)LPrimitiveLongPair;");
            let ir = analysis.ir();
            let operations = Operations::of(ir.code().expect("code"), ir.constant_pool());
            assert!(
                matches!(operations.get(3), Some(Operation::Invoke(call)) if call.name() == "markInt")
            );
            assert!(matches!(
                operations.get(6),
                Some(Operation::Store { slot: 1 })
            ));
            assert!(matches!(
                operations.get(11),
                Some(Operation::Load { slot: 1 })
            ));
            assert!(matches!(
                operations.get(13),
                Some(Operation::Load { slot: 1 })
            ));
            assert!(matches!(
                operations.get(12),
                Some(Operation::PrimitiveConversion {
                    source: crate::Type::Int,
                    target: crate::Type::Long
                })
            ));
            assert!(matches!(
                operations.get(14),
                Some(Operation::PrimitiveConversion {
                    source: crate::Type::Int,
                    target: crate::Type::Long
                })
            ));

            let contrast = sites_of(class, "integerContrast", "(I)Ljava/lang/Integer;");
            assert!(
                contrast.site_at_head(0).is_some(),
                "{leg}: no-conversion control"
            );
            assert!(contrast.refusals().next().is_none(), "{leg}");
        }
    }

    #[test]
    fn actual_dup2_reader_and_two_ssa_outputs_remain_a_statement_free_refusal() {
        let variant = primitive_pair_extra_dup2_variant(PRIMITIVE_CONVERSION_CONTROLS_JAVAC8);
        let name = "storedLocalReuse";
        let descriptor = "(I)LPrimitiveLongPair;";
        let (analysis, _) = analyzed_caller(&variant, name, descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        let block = ssa
            .blocks()
            .iter()
            .find(|block| {
                block
                    .instructions()
                    .iter()
                    .any(|instruction| instruction.opcode() == 0x5c)
            })
            .expect("the patched straight-line method has dup2")
            .instructions();
        let duplicate = block
            .iter()
            .find(|instruction| instruction.opcode() == 0x5c)
            .expect("dup2 instruction");
        assert_ne!(
            operations.get(duplicate.bci()),
            Some(&Operation::Duplicate),
            "the regular dup fact must not stand in for dup2"
        );
        assert!(
            block.iter().any(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::Invoke(call))
                        if call.owner() == "PrimitiveLongPair"
                            && call.name() == "<init>"
                            && call.descriptor() == "(JJ)V"
                )
            }),
            "the constructor remains the real Pair(JJ)V target"
        );
        assert!(block.iter().any(|instruction| matches!(
            operations.get(instruction.bci()),
            Some(Operation::PrimitiveConversion {
                source: crate::Type::Long,
                target: crate::Type::Float
            })
        )));
        assert!(block.iter().any(|instruction| matches!(
            operations.get(instruction.bci()),
            Some(Operation::PrimitiveConversion {
                source: crate::Type::Float,
                target: crate::Type::Long
            })
        )));
        let reads: Vec<(Slot, ValueId)> = duplicate
            .reads()
            .iter()
            .filter_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some((*slot, *value)))
            .collect();
        let writes: Vec<(Slot, ValueId)> = duplicate
            .writes()
            .iter()
            .filter_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some((*slot, *value)))
            .collect();
        assert_eq!(
            reads.len(),
            1,
            "dup2 reads the actual category-2 value once"
        );
        assert_eq!(writes.len(), 2, "dup2 writes the two actual stack copies");
        assert_ne!(
            writes[0].1, writes[1].1,
            "the SSA outputs are distinct values"
        );
        assert!(writes.iter().all(|(_, value)| *value != reads[0].1));

        let chains = crate::concat::Plan::empty();
        let fields = field::Plan::empty();
        let arrays = crate::build::ArrayInitializers::default();
        let plan = sites(
            ssa,
            &operations,
            &chains,
            chains.owned(),
            &fields,
            &arrays,
            8,
            &[],
            &crate::facts::MethodFacts::new(name, descriptor, 0),
            code,
        );
        assert!(
            plan.site_at_head(7).is_none(),
            "dup2 is not admitted as a value alias"
        );
        let refusal = plan
            .refusals
            .iter()
            .find(|refusal| refusal.head == 7)
            .expect("the allocation keeps its refusal");
        assert_eq!(refusal.refusal.code(), "jre_new_interleaved_effect");
        assert_eq!(
            refusal.refusal.requirement(),
            Some(Precondition::StatementFree),
            "the second physical copy remains an exact statement-free refusal"
        );
        assert!(
            refusal
                .refusal
                .message()
                .contains(&format!("BCI {}", duplicate.bci()))
        );
    }

    #[test]
    fn ordinary_constructor_does_not_admit_an_unrelated_conversion() {
        let variant = primitive_unrelated_conversion_variant(PRIMITIVE_CONVERSION_CONTROLS_JAVAC8);
        let name = "wrapperByte";
        let descriptor = "(I)Ljava/lang/Byte;";
        let (analysis, _) = analyzed_caller(&variant, name, descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        assert!(matches!(
            operations.get(5),
            Some(Operation::PrimitiveConversion {
                source: crate::Type::Int,
                target: crate::Type::Long
            })
        ));
        let chains = crate::concat::Plan::empty();
        let fields = field::Plan::empty();
        let arrays = crate::build::ArrayInitializers::default();
        let plan = sites(
            ssa,
            &operations,
            &chains,
            chains.owned(),
            &fields,
            &arrays,
            8,
            &[],
            &crate::facts::MethodFacts::new(name, descriptor, 0),
            code,
        );
        assert!(plan.site_at_head(0).is_none());
        let refusal = plan
            .refusals
            .iter()
            .find(|refusal| refusal.head == 0)
            .expect("unrelated operation keeps the quote");
        assert_eq!(refusal.refusal.code(), "jre_new_interleaved_effect");
        assert_eq!(
            refusal.refusal.requirement(),
            Some(Precondition::StatementFree)
        );
        assert!(refusal.refusal.message().contains("BCI 5"));
    }

    #[test]
    fn primitive_conversion_reuses_exact_handler_coverage_for_constructor_and_consumer() {
        for (class, leg) in [
            (PRIMITIVE_HANDLER_CONTROLS_JAVAC8, "javac 8"),
            (PRIMITIVE_HANDLER_CONTROLS_JAVAC23, "javac 23 --release 8"),
        ] {
            let name = "sameHandler";
            let descriptor = "(I)Ljava/lang/Long;";
            let plan = sites_of(class, name, descriptor);
            let site = plan
                .site_at_head(0)
                .unwrap_or_else(|| panic!("{leg}: matching allocation-through-astore handlers"));
            assert_eq!(site.constructor, 9, "{leg}");
            assert!(plan.refusals().next().is_none(), "{leg}");

            let mut budget = proof_budget();
            let facts = jarde_reader::classfile::class_facts(class, &mut budget)
                .expect("handler fixture facts");
            let method = facts
                .methods
                .iter()
                .find(|method| {
                    method.name.raw().0.as_slice() == name.as_bytes()
                        && method.descriptor.raw().0.as_slice() == descriptor.as_bytes()
                })
                .expect("exact handler method");
            let code = jarde_reader::classfile::method_code_facts(class, method, &mut budget)
                .expect("handler method code facts");
            assert_eq!(code.exception_handlers.len(), 1, "{leg}");
            assert_eq!(
                (
                    code.exception_handlers[0].start_bci,
                    code.exception_handlers[0].end_bci,
                ),
                (0, 19),
                "the original table covers through the sole astore consumer"
            );
            let start = usize::try_from(code.code_span.start).expect("Code start fits usize");
            let code_end =
                start + usize::try_from(code.code_span.length).expect("Code length fits usize");
            let exception_entry = code_end + 2;
            let mut different_coverage = class.to_vec();
            write_u16_at(&mut different_coverage, exception_entry, 8);

            let (analysis, _) = analyzed_caller(&different_coverage, name, descriptor);
            let ir = analysis.ir();
            let ssa = ir.ssa().expect("patched handler SSA");
            let method_code = ir.code().expect("patched method code");
            assert_eq!(method_code.exception_handlers[0].start_bci, 8);
            let operations = Operations::of(method_code, ir.constant_pool());
            let chains = crate::concat::Plan::empty();
            let fields = field::Plan::empty();
            let arrays = crate::build::ArrayInitializers::default();
            let crossed = sites(
                ssa,
                &operations,
                &chains,
                chains.owned(),
                &fields,
                &arrays,
                8,
                &[],
                &crate::facts::MethodFacts::new(name, descriptor, 0),
                method_code,
            );
            assert!(crossed.site_at_head(0).is_none(), "{leg}");
            let refusal = crossed
                .refusals
                .iter()
                .find(|refusal| refusal.head == 0)
                .expect("handler mismatch stays quoted");
            assert_eq!(
                refusal.refusal.code(),
                "jre_new_primitive_conversion_exception_boundary",
                "{leg}: conversion alone activated the existing boundary proof"
            );
        }
    }

    #[test]
    fn dynamic_constructor_arguments_keep_one_handler_coverage() {
        // `runnable`: allocation 0, dynamic factory 4, constructor 9, sole return consumer 12.
        // Each independently crossed instruction changes exactly that fact's ordinal vector.
        for boundary in [(0, 3), (4, 5), (9, 12), (12, 13)] {
            let plan = sites_of_with_handler(
                FUNCTIONAL_CONSTRUCTORS_JAVAC8,
                "runnable",
                "()Ljava/lang/Thread;",
                boundary,
            );
            assert!(plan.site_at_head(0).is_none(), "boundary {boundary:?}");
            let refusal = plan.refusals().next().expect("handler refusal");
            assert_eq!(
                refusal.code(),
                "jre_new_dynamic_argument_exception_boundary",
                "boundary {boundary:?}: {refusal:?}"
            );
        }
    }

    /// The bound-receiver tail is claimed exactly where the receiver's own value chain ends at an
    /// allocation and the slot it was read from is not written again — and nowhere else. Both legs
    /// answer the same BCIs, so the proof reads the shape and not one compiler's spelling.
    #[test]
    fn the_bound_receiver_tail_is_claimed_exactly_where_the_receiver_is_proved() {
        for (class, leg) in [
            (BOUND_RECEIVER_ANCHOR, "javac 23 --release 8"),
            (BOUND_RECEIVER_ANCHOR_JAVAC8, "real javac 8"),
        ] {
            let plan = sites_of(
                class,
                "sideEffect",
                "(Ljava/util/Optional;)Ljava/lang/String;",
            );
            let tails: Vec<(&u32, &u32, &u32)> = plan
                .receiver_tails
                .iter()
                .map(|tail| (&tail.copy, &tail.check, &tail.pop))
                .collect();
            assert_eq!(tails, [(&10, &11, &14)], "{leg}");
            let tail = &plan.receiver_tails[0];
            assert_eq!(
                tail.site, 15,
                "{leg}: the tail belongs to the site it guards"
            );
            assert!(
                plan.owns(tail.copy) && plan.owns(tail.check) && plan.owns(tail.pop),
                "{leg}: every instruction of the check is owned by the site"
            );
            // The construction the receiver comes from is a site of its own, and the two claims
            // are disjoint: the tail's three instructions are the dynamic site's, not `new@1`'s.
            assert_eq!(
                plan.site_at_head(0)
                    .expect("the receiver's own construction")
                    .owned
                    .iter()
                    .copied()
                    .collect::<Vec<u32>>(),
                [0, 3, 4],
                "{leg}"
            );
        }

        // The three negatives: a parameter read, a field read, and a slot written again after the
        // capture. None of them has a tail, and the three instructions of each one's own check stay
        // unowned — the window is the same shape in all three, and it is the receiver's value chain
        // that decides.
        for (class, leg) in [
            (BOUND_RECEIVER_NEGATIVES, "javac 23 --release 8"),
            (BOUND_RECEIVER_NEGATIVES_JAVAC8, "real javac 8"),
        ] {
            for (name, descriptor, check) in [
                (
                    "nullableParameter",
                    "(Ljava/util/Optional;Ljava/lang/StringBuilder;)Ljava/lang/String;",
                    (2, 3, 6),
                ),
                (
                    "nullableField",
                    "(Ljava/util/Optional;)Ljava/lang/String;",
                    (5, 6, 9),
                ),
                (
                    "rewrittenAfterCapture",
                    "(Ljava/util/Optional;)Ljava/lang/String;",
                    (10, 11, 14),
                ),
            ] {
                let plan = sites_of(class, name, descriptor);
                assert!(
                    plan.receiver_tails.is_empty(),
                    "{leg}/{name}: no receiver is proved here"
                );
                let (copy, check, pop) = check;
                for bci in [copy, check, pop] {
                    assert!(!plan.owns(bci), "{leg}/{name}: BCI {bci} stays unowned");
                }
            }
        }

        // The widened declaration: the same `dup; check; pop` window over a proved receiver, with the
        // site's own descriptor naming the supertype the local was declared as. The claim is refused
        // there, so the window stays unowned and the site's refusal keeps its quote.
        let plan = sites_of(
            BOUND_RECEIVER_WIDENED,
            "chain",
            "(Ljava/util/List;)Ljava/util/List;",
        );
        assert!(
            plan.receiver_tails.is_empty(),
            "a site that names another type than the allocation builds is not claimed"
        );
        for bci in [40, 41, 44] {
            assert!(!plan.owns(bci), "BCI {bci} stays unowned");
        }
    }

    #[test]
    fn inline_char_array_proof_obeys_shared_budget_and_cancellation() {
        let (analysis, _) = analyzed_caller(INLINE_CHAR_ARRAY, "direct", "()Ljava/lang/String;");
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        let fields = field::Plan::empty();

        let mut limits = proof_budget().limits().clone();
        limits.ir_items = 1;
        let mut limited = Budget::new(limits);
        let stop = crate::build::ArrayInitializers::prove(ssa, &operations, &fields, &mut limited)
            .err()
            .expect("array proof must stop before publishing a partial certificate");
        assert!(matches!(
            stop,
            crate::stop::StopReason::Budget { at: Some(_), .. }
        ));

        let mut cancelled = proof_budget();
        cancelled.cancellation_token().cancel();
        let stop =
            crate::build::ArrayInitializers::prove(ssa, &operations, &fields, &mut cancelled)
                .err()
                .expect("cancelled array proof publishes no certificate");
        assert!(matches!(
            stop,
            crate::stop::StopReason::Cancelled { at: Some(_) }
        ));
    }

    #[test]
    fn inline_char_certificate_requires_the_exact_parent_interval_and_consumer() {
        let (analysis, _) = analyzed_caller(INLINE_CHAR_ARRAY, "direct", "()Ljava/lang/String;");
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let operations = Operations::of(ir.code().expect("code"), ir.constant_pool());
        let arrays = crate::build::ArrayInitializers::prove(
            ssa,
            &operations,
            &field::Plan::empty(),
            &mut proof_budget(),
        )
        .expect("complete array certificate");
        let block = ssa.blocks()[0].instructions();
        let constructor = block
            .iter()
            .find(|instruction| instruction.bci() == 22)
            .expect("constructor");
        let argument = stack_operands(constructor)[1].1;
        assert!(
            arrays
                .inline_char_argument_bcis(ssa, &operations, block, argument, 3, 22)
                .is_some()
        );
        assert!(
            arrays
                .inline_char_argument_bcis(ssa, &operations, block, argument, 5, 22)
                .is_none()
        );
        assert!(
            arrays
                .inline_char_argument_bcis(ssa, &operations, block, argument, 3, 25)
                .is_none()
        );
    }

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

    fn sequence_code_facts(bytes: &[u8]) -> (jarde_reader::classfile::MethodCodeFacts, usize) {
        let mut budget = proof_budget();
        let class =
            jarde_reader::classfile::class_facts(bytes, &mut budget).expect("sequence class facts");
        let method = class
            .methods
            .iter()
            .find(|method| {
                method.name.raw().0.as_slice() == b"sequence"
                    && method.descriptor.raw().0.as_slice() == b"()[Ljava/lang/CharSequence;"
            })
            .expect("exact sequence method");
        let code_attribute = method
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0.as_slice() == b"Code")
            .expect("sequence Code attribute");
        let attribute_length_offset = usize::try_from(code_attribute.content_span.start)
            .expect("Code content offset fits usize")
            .checked_sub(4)
            .expect("Code attribute length precedes content");
        let code = jarde_reader::classfile::method_code_facts(bytes, method, &mut budget)
            .expect("sequence method code facts");
        (code, attribute_length_offset)
    }

    fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
        u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
    }

    fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ])
    }

    fn write_u32_at(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
    }

    fn write_u16_at(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }

    fn primitive_unrelated_conversion_variant(class: &[u8]) -> Vec<u8> {
        let mut budget = proof_budget();
        let facts = jarde_reader::classfile::class_facts(class, &mut budget)
            .expect("primitive conversion control class facts");
        let method = facts
            .methods
            .iter()
            .find(|method| {
                method.name.raw().0.as_slice() == b"wrapperByte"
                    && method.descriptor.raw().0.as_slice() == b"(I)Ljava/lang/Byte;"
            })
            .expect("wrapperByte method");
        let code_attribute = method
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0.as_slice() == b"Code")
            .expect("wrapperByte Code attribute");
        let attribute_length_offset = usize::try_from(code_attribute.content_span.start)
            .expect("Code content offset fits usize")
            .checked_sub(4)
            .expect("Code attribute length precedes content");
        let code = jarde_reader::classfile::method_code_facts(class, method, &mut budget)
            .expect("wrapperByte method code facts");
        assert!(code.exception_handlers.is_empty());
        assert!(
            code.control_flow_targets()
                .expect("straight-line control-flow facts")
                .is_empty()
        );
        assert!(
            code.max_stack >= 4,
            "the neutral sequence peaks at four slots"
        );
        assert_eq!(code.instructions[0].bci, 0);
        assert_eq!(code.instructions[0].opcode, 0xbb, "new Byte");
        assert_eq!(code.instructions[1].bci, 3);
        assert_eq!(code.instructions[1].opcode, 0x59, "dup");
        let start = usize::try_from(code.code_span.start).expect("Code start fits usize");
        let attribute_length_offset_expected = start - 12;
        assert_eq!(attribute_length_offset, attribute_length_offset_expected);
        let code_end =
            start + usize::try_from(code.code_span.length).expect("Code length fits usize");
        assert_eq!(read_u16_at(class, code_end), 0, "no exception table");
        assert_eq!(read_u16_at(class, code_end + 2), 0, "no Code subattributes");

        let mut variant = class.to_vec();
        let insert_at = start + 4;
        // A stack-neutral int-to-long conversion between `new; dup` and the actual argument run.
        variant.splice(insert_at..insert_at, [0x03, 0x85, 0x58]);
        write_u32_at(
            &mut variant,
            start - 4,
            u32::try_from(code.code_span.length + 3).expect("patched Code length fits u32"),
        );
        write_u32_at(
            &mut variant,
            attribute_length_offset,
            read_u32_at(class, attribute_length_offset) + 3,
        );
        variant
    }

    fn primitive_pair_extra_dup2_variant(class: &[u8]) -> Vec<u8> {
        let mut budget = proof_budget();
        let facts = jarde_reader::classfile::class_facts(class, &mut budget)
            .expect("primitive conversion control class facts");
        let method = facts
            .methods
            .iter()
            .find(|method| {
                method.name.raw().0.as_slice() == b"storedLocalReuse"
                    && method.descriptor.raw().0.as_slice() == b"(I)LPrimitiveLongPair;"
            })
            .expect("the existing method returns the actual Pair(JJ) owner");
        let code_attribute = method
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0.as_slice() == b"Code")
            .expect("storedLocalReuse Code attribute");
        let attribute_length_offset = usize::try_from(code_attribute.content_span.start)
            .expect("Code content offset fits usize")
            .checked_sub(4)
            .expect("Code attribute length precedes content");
        let code = jarde_reader::classfile::method_code_facts(class, method, &mut budget)
            .expect("storedLocalReuse method code facts");
        assert!(code.exception_handlers.is_empty());
        assert!(
            code.control_flow_targets()
                .expect("straight-line control-flow facts")
                .is_empty()
        );
        assert!(
            code.max_stack >= 6,
            "the unmodified constructor needs six slots"
        );

        let first_i2l = code
            .instructions
            .iter()
            .position(|instruction| instruction.opcode == 0x85)
            .expect("first i2l conversion");
        let second_load = &code.instructions[first_i2l + 1];
        let second_i2l = &code.instructions[first_i2l + 2];
        assert_eq!(second_load.opcode, 0x1b, "the second source is iload_1");
        assert_eq!(
            second_i2l.opcode, 0x85,
            "the second source is converted to long"
        );
        assert_eq!(
            usize::try_from(second_i2l.bci).expect("BCI fits usize"),
            usize::try_from(second_load.bci).expect("BCI fits usize") + 1
        );

        let start = usize::try_from(code.code_span.start).expect("Code start fits usize");
        assert_eq!(attribute_length_offset, start - 12);
        let code_end =
            start + usize::try_from(code.code_span.length).expect("Code length fits usize");
        assert_eq!(read_u16_at(class, code_end), 0, "no exception table");
        assert_eq!(
            read_u16_at(class, code_end + 2),
            0,
            "the Code attribute has no nested offset-bearing attributes"
        );

        let mut variant = class.to_vec();
        let replace_start = start + usize::try_from(second_load.bci).expect("BCI fits usize");
        let replace_end = start + usize::try_from(second_i2l.bci).expect("BCI fits usize") + 1;
        assert_eq!(replace_end - replace_start, 2);
        // Replace the second local-derived argument with a category-2 duplicate whose converted
        // copy is also passed to the existing `(JJ)V` constructor. The source return and descriptor
        // stay unchanged; the inserted dup2 is a genuine one-read/two-value instruction.
        variant.splice(replace_start..replace_end, [0x5c, 0x89, 0x8c]);
        write_u32_at(
            &mut variant,
            start - 4,
            u32::try_from(code.code_span.length + 1).expect("patched Code length fits u32"),
        );
        write_u32_at(
            &mut variant,
            attribute_length_offset,
            read_u32_at(class, attribute_length_offset) + 1,
        );
        assert_eq!(
            read_u32_at(&variant, start - 4),
            u32::try_from(code.code_span.length + 1).expect("patched Code length fits u32")
        );
        assert_eq!(
            read_u32_at(&variant, attribute_length_offset),
            read_u32_at(class, attribute_length_offset) + 1
        );
        assert_eq!(
            read_u16_at(&variant, start - 8),
            code.max_stack,
            "the actual existing max_stack already covers the dup2 peak"
        );
        variant
    }

    fn sequence_index_variant(class: &[u8], descending: bool) -> Vec<u8> {
        let (code, _) = sequence_code_facts(class);
        let start = usize::try_from(code.code_span.start).expect("Code start fits usize");
        assert_eq!(
            code.instructions
                .iter()
                .find(|instruction| instruction.bci == 5)
                .expect("first index producer")
                .opcode,
            0x03
        );
        assert_eq!(
            code.instructions
                .iter()
                .find(|instruction| instruction.bci == 20)
                .expect("second index producer")
                .opcode,
            0x04
        );
        let mut bytes = class.to_vec();
        if descending {
            bytes.swap(start + 5, start + 20);
        } else {
            bytes[start + 20] = 0x03;
        }
        bytes
    }

    fn sequence_extra_reader_variant(class: &[u8]) -> Vec<u8> {
        let (code, attribute_length_offset) = sequence_code_facts(class);
        assert!(code.exception_handlers.is_empty());
        assert!(
            code.control_flow_targets()
                .expect("control-flow facts")
                .is_empty()
        );
        assert!(
            code.max_stack >= 5,
            "dup_x2 peaks at five category-1 values"
        );
        assert_eq!(
            code.instructions
                .iter()
                .find(|instruction| instruction.bci == 18)
                .expect("first aastore")
                .opcode,
            0x53
        );
        let start = usize::try_from(code.code_span.start).expect("Code start fits usize");
        assert_eq!(attribute_length_offset, start - 12);
        let code_end =
            start + usize::try_from(code.code_span.length).expect("Code length fits usize");
        let exception_count = usize::from(read_u16_at(class, code_end));
        assert_eq!(exception_count, 0, "fixture has no exception table");
        let nested_attributes = code_end + 2 + exception_count * 8;
        assert_eq!(
            read_u16_at(class, nested_attributes),
            0,
            "fixture has no Code subattributes"
        );

        let mut bytes = class.to_vec();
        bytes.splice(start + 18..start + 19, [0x5b, 0x53, 0x57]);
        let code_length = u32::try_from(code.code_span.length).expect("Code length fits u32");
        write_u32_at(&mut bytes, start - 4, code_length + 2);
        write_u32_at(
            &mut bytes,
            attribute_length_offset,
            read_u32_at(class, attribute_length_offset) + 2,
        );
        bytes
    }

    fn boundary_unrelated_conversion_variant(class: &[u8], duplicate_bci: u32) -> Vec<u8> {
        let mut budget = proof_budget();
        let facts = jarde_reader::classfile::class_facts(class, &mut budget)
            .expect("boundary fixture class facts");
        let method = facts
            .methods
            .iter()
            .find(|method| {
                method.name.raw().0.as_slice() == b"firstThenUnsupportedStructure"
                    && method.descriptor.raw().0.as_slice() == b"()[Ljava/lang/Object;"
            })
            .expect("exact boundary method");
        let code_attribute = method
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0.as_slice() == b"Code")
            .expect("boundary Code attribute");
        let attribute_length_offset = usize::try_from(code_attribute.content_span.start)
            .expect("Code content offset fits usize")
            .checked_sub(4)
            .expect("Code attribute length precedes content");
        let code = jarde_reader::classfile::method_code_facts(class, method, &mut budget)
            .expect("boundary method code facts");
        assert!(
            code.exception_handlers.is_empty(),
            "no handler offsets need repair"
        );
        assert!(
            code.control_flow_targets()
                .expect("straight-line control-flow facts")
                .is_empty(),
            "no branch offsets need repair"
        );
        assert!(
            code.max_stack >= 7,
            "the inserted sequence peaks at seven slots"
        );
        let duplicate = code
            .instructions
            .iter()
            .find(|instruction| instruction.bci == duplicate_bci)
            .expect("the selected Long dup BCI");
        assert_eq!(duplicate.opcode, 0x59);
        let start = usize::try_from(code.code_span.start).expect("Code start fits usize");
        assert_eq!(attribute_length_offset, start - 12);
        let code_end =
            start + usize::try_from(code.code_span.length).expect("Code length fits usize");
        assert_eq!(read_u16_at(class, code_end), 0, "no exception table");
        assert_eq!(read_u16_at(class, code_end + 2), 0, "no Code subattributes");

        let mut variant = class.to_vec();
        let insert_at = start + usize::try_from(duplicate_bci).expect("BCI fits usize") + 1;
        variant.splice(insert_at..insert_at, [0x03, 0x85, 0x58]);
        let new_code_length = code.code_span.length + 3;
        write_u32_at(
            &mut variant,
            start - 4,
            u32::try_from(new_code_length).expect("patched Code length fits u32"),
        );
        write_u32_at(
            &mut variant,
            attribute_length_offset,
            read_u32_at(class, attribute_length_offset) + 3,
        );
        assert_eq!(
            read_u32_at(&variant, start - 4),
            u32::try_from(new_code_length).expect("patched Code length fits u32")
        );
        assert_eq!(
            read_u32_at(&variant, attribute_length_offset),
            read_u32_at(class, attribute_length_offset) + 3
        );
        assert_eq!(read_u16_at(&variant, start - 8), code.max_stack);
        variant
    }

    fn sequence_first_store_is_proved(class: &[u8]) -> bool {
        let descriptor = "()[Ljava/lang/CharSequence;";
        let (analysis, _) = analyzed_caller(class, "sequence", descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        let block = ssa
            .blocks()
            .iter()
            .find(|block| {
                block.instructions().iter().any(|instruction| {
                    instruction.bci() == 18
                        && matches!(operations.get(18), Some(Operation::ArrayStore { .. }))
                })
            })
            .expect("first sequence store block")
            .instructions();
        let store_instruction = block
            .iter()
            .find(|instruction| instruction.bci() == 18)
            .expect("first sequence store");
        let operands = crate::build::stack_operands(store_instruction);
        let [_, _, (_, stored)] = operands.as_slice() else {
            panic!("aastore has array, index and value operands");
        };
        let head_index = block
            .iter()
            .position(|instruction| instruction.bci() == 6)
            .expect("first new block position");
        let arrays = crate::build::ArrayInitializers::default();
        let child_arrays = arrays.child_facts();
        let chains = crate::concat::Plan::empty();
        let fields = field::Plan::empty();
        let method = crate::facts::MethodFacts::new("sequence", descriptor, 0);
        let context = ArrayCompositionContext {
            chains: &chains,
            reserved: chains.owned(),
            java_release: 8,
            member_targets: &[],
            method: &method,
            code,
        };
        let mut budget = proof_budget();
        verify_array_store(
            ssa,
            &operations,
            &fields,
            &child_arrays,
            &context,
            block,
            6,
            head_index,
            "java/lang/StringBuilder".to_owned(),
            18,
            *stored,
            &mut budget,
        )
        .expect("first store's independent construction proof")
        .is_some()
    }

    fn sequence_first_store_identity_guard_accepts(class: &[u8]) -> bool {
        let (analysis, _) = analyzed_caller(class, "sequence", "()[Ljava/lang/CharSequence;");
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        let block = ssa
            .blocks()
            .iter()
            .find(|block| {
                block
                    .instructions()
                    .iter()
                    .any(|instruction| instruction.bci() == 18)
            })
            .expect("first sequence store block")
            .instructions();
        let store_instruction = block
            .iter()
            .find(|instruction| instruction.bci() == 18)
            .expect("first sequence store");
        let operands = crate::build::stack_operands(store_instruction);
        let [_, _, (_, stored)] = operands.as_slice() else {
            panic!("aastore has array, index and value operands");
        };
        let mut meter = VerifyMeter::unmetered();
        array_store_consumes_site_value(
            ssa,
            &operations,
            block,
            6,
            9,
            15,
            18,
            *stored,
            "java/lang/StringBuilder",
            &mut meter,
        )
        .expect("first store identity check")
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

    #[test]
    fn composed_reference_array_commits_exact_constructed_values_once() {
        const MAIN: &[u8] = include_bytes!(
            "../../../tests/fixtures/p3-constructed-reference-array-elements-v1/javac23/classes/Main.class"
        );
        let descriptor = "()[Ljava/lang/CharSequence;";
        let (analysis, _) = analyzed_caller(MAIN, "sequence", descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        let fields = field::Plan::empty();
        let chains = crate::concat::Plan::empty();
        let method = crate::facts::MethodFacts::new("sequence", descriptor, 0);
        let context = ArrayCompositionContext {
            chains: &chains,
            reserved: chains.owned(),
            java_release: 8,
            member_targets: &[],
            method: &method,
            code,
        };
        let mut limits = proof_budget().limits().clone();
        limits.analysis_steps = 1;
        let mut limited = Budget::new(limits);
        assert!(matches!(
            crate::build::ArrayInitializers::prove_with_composition(
                ssa,
                &operations,
                &fields,
                Some(&context),
                &mut limited,
            ),
            Err(crate::stop::StopReason::Budget { at: Some(_), .. })
        ));
        let mut cancelled = proof_budget();
        cancelled.cancellation_token().cancel();
        assert!(matches!(
            crate::build::ArrayInitializers::prove_with_composition(
                ssa,
                &operations,
                &fields,
                Some(&context),
                &mut cancelled,
            ),
            Err(crate::stop::StopReason::Cancelled { at: Some(_) })
        ));
        let mut budget = proof_budget();
        let mut arrays = crate::build::ArrayInitializers::prove_with_composition(
            ssa,
            &operations,
            &fields,
            Some(&context),
            &mut budget,
        )
        .expect("array and construction proof completes");
        let sites = sites_after_array_composition(
            ssa,
            &operations,
            &chains,
            chains.owned(),
            &fields,
            &mut arrays,
            8,
            &[],
            &method,
            code,
            &mut budget,
        )
        .expect("site census completes");
        let constructed: Vec<&Site> = sites
            .allocation_candidates()
            .iter()
            .filter_map(|candidate| sites.site_at_head(candidate.head))
            .filter(|site| {
                matches!(
                    site.class.as_str(),
                    "java/lang/StringBuilder" | "java/lang/StringBuffer"
                )
            })
            .collect();
        assert_eq!(constructed.len(), 2);
        assert!(constructed.iter().all(|site| site.finished_value.is_some()));
        assert_ne!(
            constructed[0].finished_value, constructed[1].finished_value,
            "each array index retains its own exact constructor output"
        );

        let descriptor = "()[Ljava/util/Collection;";
        let (analysis, _) = analyzed_caller(MAIN, "collections", descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        let method = crate::facts::MethodFacts::new("collections", descriptor, 0);
        let context = ArrayCompositionContext {
            chains: &chains,
            reserved: chains.owned(),
            java_release: 8,
            member_targets: &[],
            method: &method,
            code,
        };
        let mut budget = proof_budget();
        let mut arrays = crate::build::ArrayInitializers::prove_with_composition(
            ssa,
            &operations,
            &fields,
            Some(&context),
            &mut budget,
        )
        .expect("nested child-array and construction proofs complete");
        let sites = sites_after_array_composition(
            ssa,
            &operations,
            &chains,
            chains.owned(),
            &fields,
            &mut arrays,
            8,
            &[],
            &method,
            code,
            &mut budget,
        )
        .expect("site census completes");
        let collection_sites: Vec<&Site> = sites
            .allocation_candidates()
            .iter()
            .filter_map(|candidate| sites.site_at_head(candidate.head))
            .filter(|site| {
                matches!(
                    site.class.as_str(),
                    "java/util/ArrayList" | "java/util/HashSet"
                )
            })
            .collect();
        assert_eq!(collection_sites.len(), 2);
        assert!(
            collection_sites.iter().all(|site| {
                site.single_use_atoms
                    .iter()
                    .any(|bci| matches!(operations.get(*bci), Some(Operation::NewArray { .. })))
            }),
            "the verified Arrays.asList child arrays are retained as identity atoms"
        );
    }

    #[test]
    fn composed_nested_site_ownership_and_store_binding_are_exact() {
        const NESTED: &[u8] = include_bytes!(
            "../../../tests/fixtures/p3-constructed-reference-array-controls-v1/javac23/NestedControls.class"
        );
        let descriptor = "()[Ljava/lang/Object;";
        let (analysis, _) = analyzed_caller(NESTED, "nested", descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        let block = ssa
            .blocks()
            .iter()
            .find(|block| {
                block.instructions().iter().any(|instruction| {
                    matches!(
                        operations.get(instruction.bci()),
                        Some(Operation::ArrayStore { .. })
                    )
                })
            })
            .expect("nested method array-store block")
            .instructions();
        let (store, store_instruction) = block
            .iter()
            .find(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::ArrayStore { .. })
                )
            })
            .map(|instruction| (instruction.bci(), instruction))
            .expect("paired aastore");
        let operands = crate::build::stack_operands(store_instruction);
        let [_, _, (_, stored)] = operands.as_slice() else {
            panic!("aastore has array, index and value operands");
        };
        let stored = *stored;
        let heads: Vec<u32> = block
            .iter()
            .filter(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::Allocate { .. })
                )
            })
            .map(SsaInstruction::bci)
            .collect();
        assert_eq!(heads, [6, 10], "frozen nested controls source anchors");
        assert_eq!(store, 25, "frozen nested controls store anchor");
        let outer_index = block
            .iter()
            .position(|instruction| instruction.bci() == 6)
            .expect("outer allocation block position");
        let inner_index = block
            .iter()
            .position(|instruction| instruction.bci() == 10)
            .expect("inner allocation block position");

        let chains = crate::concat::Plan::empty();
        let fields = field::Plan::empty();
        let method = crate::facts::MethodFacts::new("nested", descriptor, 0);
        let context = ArrayCompositionContext {
            chains: &chains,
            reserved: chains.owned(),
            java_release: 8,
            member_targets: &[],
            method: &method,
            code,
        };
        let mut budget = proof_budget();
        let mut arrays = crate::build::ArrayInitializers::prove_with_composition(
            ssa,
            &operations,
            &fields,
            Some(&context),
            &mut budget,
        )
        .expect("nested array candidate completes");
        assert!(arrays.has_candidate_at(1), "array initializer is committed");
        let pending = arrays.take_pending_sites();
        assert_eq!(
            pending.len(),
            2,
            "outer and inner allocations transfer once"
        );
        let outer = pending.get(&6).expect("outer site");
        let inner = pending.get(&10).expect("inner site");
        assert_eq!(outer.finished_value, Some(stored));
        assert!(outer.owned.is_disjoint(&inner.owned));
        assert_eq!(outer.owned.len(), 3);
        assert_eq!(inner.owned.len(), 3);
        assert!(
            outer
                .instance
                .iter()
                .all(|bci| !inner.instance.contains(bci))
        );
        assert!(
            outer.nested_sites.is_empty(),
            "sites were flattened, not merged"
        );

        // The exact pair succeeds. Counterfactual stored values use the actual array/index
        // operands, followed by wrong allocation and block-position bindings.
        let child_arrays = arrays.child_facts();
        let mut budget = proof_budget();
        assert!(
            verify_array_store(
                ssa,
                &operations,
                &fields,
                &child_arrays,
                &context,
                block,
                6,
                outer_index,
                "java/lang/StringBuilder".to_owned(),
                store,
                stored,
                &mut budget,
            )
            .expect("correct stored-value proof completes")
            .is_some()
        );
        for (head, index, value) in [
            (6, outer_index, operands[0].1),
            (6, outer_index, operands[1].1),
            (10, inner_index, stored),
            (6, inner_index, stored),
        ] {
            let mut budget = proof_budget();
            assert!(
                verify_array_store(
                    ssa,
                    &operations,
                    &fields,
                    &child_arrays,
                    &context,
                    block,
                    head,
                    index,
                    "java/lang/StringBuilder".to_owned(),
                    store,
                    value,
                    &mut budget,
                )
                .expect("counterfactual refusal is not a stop")
                .is_none()
            );
        }
        let mut meter = VerifyMeter::unmetered();
        assert!(
            !array_store_consumes_site_value(
                ssa,
                &operations,
                block,
                6,
                store,
                22,
                store,
                stored,
                "java/lang/StringBuilder",
                &mut meter,
            )
            .expect("wrong-dup check completes")
        );
    }

    #[test]
    fn nested_composition_budget_can_stop_inside_recursive_constructor_proof() {
        const NESTED: &[u8] = include_bytes!(
            "../../../tests/fixtures/p3-constructed-reference-array-controls-v1/javac23/NestedControls.class"
        );
        let descriptor = "()[Ljava/lang/Object;";
        let (analysis, _) = analyzed_caller(NESTED, "nested", descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        let block = ssa
            .blocks()
            .iter()
            .find(|block| {
                block.instructions().iter().any(|instruction| {
                    matches!(
                        operations.get(instruction.bci()),
                        Some(Operation::ArrayStore { .. })
                    )
                })
            })
            .expect("nested method array-store block")
            .instructions();
        let (store, store_instruction) = block
            .iter()
            .find(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::ArrayStore { .. })
                )
            })
            .map(|instruction| (instruction.bci(), instruction))
            .expect("paired aastore");
        let operands = crate::build::stack_operands(store_instruction);
        let [_, _, (_, stored)] = operands.as_slice() else {
            panic!("aastore has array, index and value operands");
        };
        let stored = *stored;
        assert_eq!(store, 25, "frozen nested controls store anchor");
        let outer_index = block
            .iter()
            .position(|instruction| instruction.bci() == 6)
            .expect("outer allocation block position");
        let chains = crate::concat::Plan::empty();
        let fields = field::Plan::empty();
        let method = crate::facts::MethodFacts::new("nested", descriptor, 0);
        let context = ArrayCompositionContext {
            chains: &chains,
            reserved: chains.owned(),
            java_release: 8,
            member_targets: &[],
            method: &method,
            code,
        };
        let arrays = crate::build::ArrayInitializers::default();
        let child_arrays = arrays.child_facts();
        let mut complete_budget = proof_budget();
        assert!(
            verify_array_store(
                ssa,
                &operations,
                &fields,
                &child_arrays,
                &context,
                block,
                6,
                outer_index,
                "java/lang/StringBuilder".to_owned(),
                store,
                stored,
                &mut complete_budget,
            )
            .expect("complete direct nested proof runs")
            .is_some()
        );

        // In this direct seam, verify_metered's outer scan charges BCI 10 once and immediately
        // enters the nested verifier. Its first argument-scan instruction is BCI 14, whose second
        // AnalysisSteps charge must stop there with a limit of one. This cannot be an array
        // candidate discovery or allocation/store pairing scan because those callers are absent.
        let mut limits = proof_budget().limits().clone();
        limits.analysis_steps = 1;
        let mut budget = Budget::new(limits);
        assert!(matches!(
            verify_array_store(
                ssa,
                &operations,
                &fields,
                &child_arrays,
                &context,
                block,
                6,
                outer_index,
                "java/lang/StringBuilder".to_owned(),
                store,
                stored,
                &mut budget,
            ),
            Err(crate::stop::StopReason::Budget { at: Some(14), .. })
        ));
    }

    #[test]
    fn array_argument_conversion_is_accepted_but_unrelated_conversion_aborts_the_candidate() {
        const BOUNDARY: &[u8] = include_bytes!(
            "../../../tests/fixtures/p3-constructed-reference-array-controls-v1/javac23/BoundaryControls.class"
        );
        let descriptor = "()[Ljava/lang/Object;";
        let (analysis, _) = analyzed_caller(BOUNDARY, "firstThenUnsupportedStructure", descriptor);
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let code = ir.code().expect("code");
        let operations = Operations::of(code, ir.constant_pool());
        let block = ssa
            .blocks()
            .iter()
            .find(|block| {
                block.instructions().iter().any(|instruction| {
                    matches!(
                        operations.get(instruction.bci()),
                        Some(Operation::ArrayStore { .. })
                    )
                })
            })
            .expect("boundary method array-store block")
            .instructions();
        let allocation = block
            .iter()
            .find(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::NewArray { .. })
                )
            })
            .expect("fresh Object[] allocation")
            .bci();
        let first_store = block
            .iter()
            .find(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::ArrayStore { .. })
                )
            })
            .expect("first element store")
            .bci();
        let conversion = block
            .iter()
            .find(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::PrimitiveConversion {
                        source: crate::Type::Int,
                        target: crate::Type::Long
                    })
                )
            })
            .expect("second element's supported int-to-long argument conversion")
            .bci();
        assert!(allocation < first_store && first_store < conversion);
        let second_new = block
            .iter()
            .find(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::Allocate { ty }) if ty == "java/lang/Long"
                )
            })
            .expect("second element Long allocation");
        let duplicate = block
            .iter()
            .find(|instruction| instruction.bci() == second_new.bci() + 3)
            .expect("Long allocation's following dup");
        assert_eq!(duplicate.opcode(), 0x59);
        let long_constructor = block
            .iter()
            .find(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::Invoke(call))
                        if call.owner() == "java/lang/Long" && call.name() == "<init>"
                )
            })
            .expect("second element Long constructor");
        let long_operands = crate::build::stack_operands(long_constructor);
        assert!(matches!(
            long_operands.last().map(|(_, value)| ssa.value(*value).def()),
            Some(Definition::Instruction { bci, .. }) if *bci == conversion
        ));

        let chains = crate::concat::Plan::empty();
        let fields = field::Plan::empty();
        let method = crate::facts::MethodFacts::new("firstThenUnsupportedStructure", descriptor, 0);
        let context = ArrayCompositionContext {
            chains: &chains,
            reserved: chains.owned(),
            java_release: 8,
            member_targets: &[],
            method: &method,
            code,
        };
        let mut budget = proof_budget();
        let mut arrays = crate::build::ArrayInitializers::prove_with_composition(
            ssa,
            &operations,
            &fields,
            Some(&context),
            &mut budget,
        )
        .expect("the complete array initializer proves");
        assert!(arrays.has_candidate_at(allocation));
        let pending = arrays.take_pending_sites();
        assert_eq!(pending.len(), 2, "both constructed array elements transfer");
        let long_site = pending
            .values()
            .find(|site| site.class == "java/lang/Long")
            .expect("the second element is an independently committed Long Site");
        assert!(long_site.arguments.contains(&conversion));

        let variant = boundary_unrelated_conversion_variant(BOUNDARY, duplicate.bci());
        let (analysis, _) = analyzed_caller(&variant, "firstThenUnsupportedStructure", descriptor);
        let ir = analysis.ir();
        let patched_ssa = ir.ssa().expect("patched SSA");
        let patched_code = ir.code().expect("patched method code");
        assert!(patched_code.exception_handlers.is_empty());
        let patched_operations = Operations::of(patched_code, ir.constant_pool());
        let unrelated_conversion = duplicate.bci() + 2;
        assert!(matches!(
            patched_operations.get(unrelated_conversion),
            Some(Operation::PrimitiveConversion {
                source: crate::Type::Int,
                target: crate::Type::Long
            })
        ));
        let patched_allocation = patched_ssa
            .blocks()
            .iter()
            .flat_map(|block| block.instructions())
            .find(|instruction| {
                matches!(
                    patched_operations.get(instruction.bci()),
                    Some(Operation::NewArray { .. })
                )
            })
            .expect("same array allocation after the inserted neutral sequence")
            .bci();
        let patched_block = patched_ssa
            .blocks()
            .iter()
            .find(|block| {
                block.instructions().iter().any(|instruction| {
                    matches!(
                        patched_operations.get(instruction.bci()),
                        Some(Operation::ArrayStore { .. })
                    )
                })
            })
            .expect("patched array-store block")
            .instructions();
        assert!(patched_block.iter().any(|instruction| {
            instruction.bci() == duplicate.bci() + 1 && instruction.opcode() == 0x03
        }));
        assert!(patched_block.iter().any(|instruction| {
            instruction.bci() == duplicate.bci() + 3 && instruction.opcode() == 0x58
        }));
        let patched_long_constructor = patched_block
            .iter()
            .find(|instruction| {
                matches!(
                    patched_operations.get(instruction.bci()),
                    Some(Operation::Invoke(call))
                        if call.owner() == "java/lang/Long" && call.name() == "<init>"
                )
            })
            .expect("patched second-element Long constructor");
        let patched_long_operands = crate::build::stack_operands(patched_long_constructor);
        let [_, (_, long_argument)] = patched_long_operands.as_slice() else {
            panic!("Long(J) constructor has receiver and one physical argument");
        };
        let mut meter = VerifyMeter::unmetered();
        let dependencies = value_dependency_bcis_metered(
            patched_ssa,
            patched_block,
            std::iter::once(*long_argument),
            &mut meter,
        )
        .expect("the physical Long argument dependency closes");
        assert!(
            !dependencies.contains(&unrelated_conversion),
            "the inserted conversion's result is discarded and is not the Long argument"
        );
        let patched_context = ArrayCompositionContext {
            chains: &chains,
            reserved: chains.owned(),
            java_release: 8,
            member_targets: &[],
            method: &method,
            code: patched_code,
        };
        let mut patched_budget = proof_budget();
        let mut patched_arrays = crate::build::ArrayInitializers::prove_with_composition(
            patched_ssa,
            &patched_operations,
            &fields,
            Some(&patched_context),
            &mut patched_budget,
        )
        .expect("the unrelated conversion is a structural refusal, not a stop");
        assert!(!patched_arrays.has_candidate_at(patched_allocation));
        assert!(patched_arrays.take_pending_sites().is_empty());
    }

    #[test]
    fn fresh_array_index_byte_variants_do_not_commit_partial_candidates() {
        const JAVAC8: &[u8] = include_bytes!(
            "../../../tests/fixtures/p3-constructed-reference-array-elements-v1/javac8/classes/Main.class"
        );
        const JAVAC23: &[u8] = include_bytes!(
            "../../../tests/fixtures/p3-constructed-reference-array-elements-v1/javac23/classes/Main.class"
        );
        for original in [JAVAC8, JAVAC23] {
            let (original_code, _) = sequence_code_facts(original);
            assert_eq!(
                original_code
                    .instructions
                    .iter()
                    .find(|instruction| instruction.bci == 5)
                    .expect("first index")
                    .opcode,
                0x03
            );
            assert_eq!(
                original_code
                    .instructions
                    .iter()
                    .find(|instruction| instruction.bci == 20)
                    .expect("second index")
                    .opcode,
                0x04
            );
            assert!(sequence_first_store_is_proved(original));

            for descending in [false, true] {
                let variant = sequence_index_variant(original, descending);
                let (patched_code, _) = sequence_code_facts(&variant);
                assert_eq!(
                    patched_code
                        .instructions
                        .iter()
                        .find(|instruction| instruction.bci == 5)
                        .expect("patched first index")
                        .opcode,
                    if descending { 0x04 } else { 0x03 }
                );
                assert_eq!(
                    patched_code
                        .instructions
                        .iter()
                        .find(|instruction| instruction.bci == 20)
                        .expect("patched second index")
                        .opcode,
                    0x03
                );

                let descriptor = "()[Ljava/lang/CharSequence;";
                let (analysis, _) = analyzed_caller(&variant, "sequence", descriptor);
                let ir = analysis.ir();
                let ssa = ir.ssa().expect("patched SSA");
                let code = ir.code().expect("patched method code");
                let operations = Operations::of(code, ir.constant_pool());
                let chains = crate::concat::Plan::empty();
                let fields = field::Plan::empty();
                let method = crate::facts::MethodFacts::new("sequence", descriptor, 0);
                let context = ArrayCompositionContext {
                    chains: &chains,
                    reserved: chains.owned(),
                    java_release: 8,
                    member_targets: &[],
                    method: &method,
                    code,
                };
                let mut budget = proof_budget();
                let mut arrays = crate::build::ArrayInitializers::prove_with_composition(
                    ssa,
                    &operations,
                    &fields,
                    Some(&context),
                    &mut budget,
                )
                .expect("index mismatch is refusal, not a stop");
                assert!(!arrays.has_candidate_at(1));
                assert!(arrays.take_pending_sites().is_empty());

                if !descending {
                    assert!(
                        sequence_first_store_is_proved(&variant),
                        "duplicate-index variant still has an independently proved first new/store"
                    );
                }
            }
        }
    }

    #[test]
    fn extra_stack_reader_byte_variant_reaches_exact_site_value_guard() {
        const JAVAC8: &[u8] = include_bytes!(
            "../../../tests/fixtures/p3-constructed-reference-array-elements-v1/javac8/classes/Main.class"
        );
        const JAVAC23: &[u8] = include_bytes!(
            "../../../tests/fixtures/p3-constructed-reference-array-elements-v1/javac23/classes/Main.class"
        );
        for original in [JAVAC8, JAVAC23] {
            assert!(sequence_first_store_is_proved(original));
            assert!(sequence_first_store_identity_guard_accepts(original));

            let variant = sequence_extra_reader_variant(original);
            let (original_code, attribute_length_offset) = sequence_code_facts(original);
            let (patched_code, patched_attribute_length_offset) = sequence_code_facts(&variant);
            assert_eq!(patched_attribute_length_offset, attribute_length_offset);
            assert_eq!(
                patched_code.code_span.length,
                original_code.code_span.length + 2
            );
            assert_eq!(
                read_u32_at(&variant, attribute_length_offset),
                read_u32_at(original, attribute_length_offset) + 2
            );
            assert!(patched_code.exception_handlers.is_empty());
            assert!(
                patched_code
                    .control_flow_targets()
                    .expect("patched control flow")
                    .is_empty()
            );
            assert!(patched_code.max_stack >= 5);
            assert_eq!(
                patched_code
                    .instructions
                    .iter()
                    .find(|instruction| instruction.bci == 18)
                    .unwrap()
                    .opcode,
                0x5b
            );
            assert_eq!(
                patched_code
                    .instructions
                    .iter()
                    .find(|instruction| instruction.bci == 19)
                    .unwrap()
                    .opcode,
                0x53
            );
            assert_eq!(
                patched_code
                    .instructions
                    .iter()
                    .find(|instruction| instruction.bci == 20)
                    .unwrap()
                    .opcode,
                0x57
            );

            let descriptor = "()[Ljava/lang/CharSequence;";
            let (analysis, _) = analyzed_caller(&variant, "sequence", descriptor);
            let ir = analysis.ir();
            let ssa = ir.ssa().expect("patched SSA");
            let code = ir.code().expect("patched method code");
            let operations = Operations::of(code, ir.constant_pool());
            let block = ssa
                .blocks()
                .iter()
                .find(|block| {
                    block
                        .instructions()
                        .iter()
                        .any(|instruction| instruction.bci() == 19)
                })
                .expect("patched first-store block")
                .instructions();
            let completed = block
                .iter()
                .find(|instruction| instruction.bci() == 15)
                .expect("first constructor")
                .writes()
                .iter()
                .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
                .expect("constructor completed reference");
            let duplicate_x2 = block
                .iter()
                .find(|instruction| instruction.bci() == 18)
                .expect("dup_x2 reader");
            assert!(
                duplicate_x2
                    .reads()
                    .iter()
                    .any(|(_, value)| *value == completed)
            );
            assert!(
                ssa.value(completed)
                    .uses()
                    .iter()
                    .any(|use_| use_.bci() == Some(18))
            );

            let store_instruction = block
                .iter()
                .find(|instruction| instruction.bci() == 19)
                .expect("paired aastore");
            let store_operands = crate::build::stack_operands(store_instruction);
            let [(_, array_value), (_, index_value), (_, stored_alias)] = store_operands.as_slice()
            else {
                panic!("patched aastore keeps array, index, and reference operands");
            };
            assert_ne!(*stored_alias, completed);
            let pop_alias = block
                .iter()
                .find(|instruction| instruction.bci() == 20)
                .expect("post-store pop")
                .reads()
                .iter()
                .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
                .expect("pop reads retained constructed alias");
            let same_class_aliases: Vec<ValueId> = duplicate_x2
                .writes()
                .iter()
                .filter_map(|(_, value)| {
                    (ssa.value(*value).ty() == ssa.value(completed).ty()).then_some(*value)
                })
                .collect();
            assert_eq!(same_class_aliases.len(), 2);
            assert!(same_class_aliases.contains(stored_alias));
            assert!(same_class_aliases.contains(&pop_alias));
            assert_ne!(*stored_alias, pop_alias);
            assert!(matches!(
                ssa.value(*stored_alias).def(),
                Definition::Instruction { bci, .. } if *bci == 18
            ));
            assert!(matches!(
                ssa.value(pop_alias).def(),
                Definition::Instruction { bci, .. } if *bci == 18
            ));
            assert!(matches!(operations.get(21), Some(Operation::Duplicate)));
            let first_array_dup = block
                .iter()
                .find(|instruction| instruction.bci() == 4)
                .expect("initial array dup");
            let array_aliases: Vec<ValueId> = first_array_dup
                .writes()
                .iter()
                .filter_map(|(_, value)| {
                    (ssa.value(*value).ty() == ssa.value(*array_value).ty()).then_some(*value)
                })
                .collect();
            assert_eq!(array_aliases.len(), 2);
            let retained_array = array_aliases
                .iter()
                .find(|value| {
                    ssa.value(**value)
                        .uses()
                        .iter()
                        .any(|use_| use_.bci() == Some(21))
                })
                .copied()
                .expect("retained array alias reaches the next dup");
            let next_array_dup = block
                .iter()
                .find(|instruction| instruction.bci() == 21)
                .expect("next array dup");
            assert!(
                next_array_dup
                    .reads()
                    .iter()
                    .any(|(_, value)| *value == retained_array)
            );
            assert!(array_aliases.iter().any(|value| {
                ssa.value(*value)
                    .uses()
                    .iter()
                    .any(|use_| use_.bci() == Some(18))
            }));
            let _ = index_value;

            let mut meter = VerifyMeter::unmetered();
            assert!(
                !array_store_consumes_site_value(
                    ssa,
                    &operations,
                    block,
                    6,
                    9,
                    15,
                    19,
                    *stored_alias,
                    "java/lang/StringBuilder",
                    &mut meter,
                )
                .expect("exact stored-alias guard returns a decision")
            );
            let mut meter = VerifyMeter::unmetered();
            assert!(
                !array_store_consumes_site_value(
                    ssa,
                    &operations,
                    block,
                    6,
                    9,
                    15,
                    19,
                    completed,
                    "java/lang/StringBuilder",
                    &mut meter,
                )
                .expect("constructor-value counterfactual returns a decision")
            );

            let chains = crate::concat::Plan::empty();
            let fields = field::Plan::empty();
            let method = crate::facts::MethodFacts::new("sequence", descriptor, 0);
            let context = ArrayCompositionContext {
                chains: &chains,
                reserved: chains.owned(),
                java_release: 8,
                member_targets: &[],
                method: &method,
                code,
            };
            let mut budget = proof_budget();
            let mut arrays = crate::build::ArrayInitializers::prove_with_composition(
                ssa,
                &operations,
                &fields,
                Some(&context),
                &mut budget,
            )
            .expect("extra reader is refusal, not a stop");
            assert!(!arrays.has_candidate_at(1));
            assert!(arrays.take_pending_sites().is_empty());
        }
    }

    #[test]
    fn handler_range_byte_variant_reaches_array_effect_closure() {
        const JAVAC8: &[u8] = include_bytes!(
            "../../../tests/fixtures/p3-constructed-reference-array-handler-v1/javac8/HandlerControls.class"
        );
        const JAVAC23: &[u8] = include_bytes!(
            "../../../tests/fixtures/p3-constructed-reference-array-handler-v1/javac23/HandlerControls.class"
        );
        for original in [JAVAC8, JAVAC23] {
            let mut budget = proof_budget();
            let class = jarde_reader::classfile::class_facts(original, &mut budget)
                .expect("handler class facts");
            let method_header = class
                .methods
                .iter()
                .find(|method| {
                    method.name.raw().0.as_slice() == b"handled"
                        && method.descriptor.raw().0.as_slice() == b"()[Ljava/lang/Object;"
                })
                .expect("handled method header");
            let mut budget = proof_budget();
            let original_code =
                jarde_reader::classfile::method_code_facts(original, method_header, &mut budget)
                    .expect("original handled code");
            assert_eq!(original_code.exception_handlers.len(), 1);
            let handler = &original_code.exception_handlers[0];
            assert_eq!(
                (handler.start_bci, handler.end_bci, handler.handler_bci),
                (0, 19, 20)
            );
            assert_eq!(
                original_code
                    .instructions
                    .iter()
                    .find(|instruction| instruction.bci == 1)
                    .expect("array allocation")
                    .opcode,
                0xbd
            );
            assert!(
                original_code
                    .control_flow_targets()
                    .unwrap()
                    .iter()
                    .any(|target| {
                        matches!(
                            target.kind,
                            jarde_reader::classfile::ControlFlowTargetKind::Handler { ordinal: 0 }
                        ) && target.target_bci == 20
                    })
            );

            let code_start =
                usize::try_from(original_code.code_span.start).expect("Code start fits usize");
            let code_end = code_start
                + usize::try_from(original_code.code_span.length).expect("Code length fits usize");
            let exception_count = usize::from(read_u16_at(original, code_end));
            assert_eq!(exception_count, 1);
            let exception_entry = code_end + 2;
            assert_eq!(read_u16_at(original, exception_entry), 0);
            assert_eq!(read_u16_at(original, exception_entry + 2), 19);
            assert_eq!(read_u16_at(original, exception_entry + 4), 20);
            assert_eq!(
                Some(read_u16_at(original, exception_entry + 6)),
                handler.catch_type_index
            );

            let mut patched = original.to_vec();
            write_u16_at(&mut patched, exception_entry, 12);
            assert_eq!(read_u16_at(&patched, exception_entry + 2), 19);
            assert_eq!(read_u16_at(&patched, exception_entry + 4), 20);
            assert_eq!(
                read_u16_at(&patched, exception_entry + 6),
                read_u16_at(original, exception_entry + 6)
            );
            let mut budget = proof_budget();
            let patched_class = jarde_reader::classfile::class_facts(&patched, &mut budget)
                .expect("patched handler class facts");
            let patched_header = patched_class
                .methods
                .iter()
                .find(|method| {
                    method.name.raw().0.as_slice() == b"handled"
                        && method.descriptor.raw().0.as_slice() == b"()[Ljava/lang/Object;"
                })
                .expect("patched handled method header");
            let mut budget = proof_budget();
            let patched_code =
                jarde_reader::classfile::method_code_facts(&patched, patched_header, &mut budget)
                    .expect("patched handled code");
            assert_eq!(
                (
                    patched_code.exception_handlers[0].start_bci,
                    patched_code.exception_handlers[0].end_bci,
                    patched_code.exception_handlers[0].handler_bci
                ),
                (12, 19, 20)
            );

            const COVERED_HANDLER: &[u32] = &[0];
            const NO_HANDLERS: &[u32] = &[];
            for (bytes, expected_array_handlers) in [
                (original, COVERED_HANDLER),
                (patched.as_slice(), NO_HANDLERS),
            ] {
                let (analysis, _) = analyzed_caller(bytes, "handled", "()[Ljava/lang/Object;");
                let ir = analysis.ir();
                let ssa = ir.ssa().expect("handled SSA");
                let code = ir.code().expect("handled method code");
                let operations = Operations::of(code, ir.constant_pool());
                let source_bcis = [1, 6, 12, 15, 18, 19];
                let source_block = ssa
                    .blocks()
                    .iter()
                    .find(|block| {
                        source_bcis.iter().all(|bci| {
                            block
                                .instructions()
                                .iter()
                                .any(|instruction| instruction.bci() == *bci)
                        })
                    })
                    .expect("array, new, mark, constructor and store remain one source block");
                let effects = ssa.effects().instructions();
                let handlers_at = |bci| {
                    effects
                        .iter()
                        .find(|effect| effect.bci() == bci)
                        .expect("instruction effect")
                        .handlers()
                };
                assert_eq!(handlers_at(1), expected_array_handlers);
                for bci in [12, 15, 18] {
                    assert_eq!(handlers_at(bci), &[0]);
                }

                let block = source_block.instructions();
                let head_index = block
                    .iter()
                    .position(|instruction| instruction.bci() == 6)
                    .expect("constructed value allocation position");
                let store_instruction = block
                    .iter()
                    .find(|instruction| instruction.bci() == 18)
                    .expect("paired array store");
                let operands = crate::build::stack_operands(store_instruction);
                let [_, _, (_, stored)] = operands.as_slice() else {
                    panic!("aastore has array, index and value operands");
                };
                let chains = crate::concat::Plan::empty();
                let fields = field::Plan::empty();
                let method = crate::facts::MethodFacts::new("handled", "()[Ljava/lang/Object;", 0);
                let context = ArrayCompositionContext {
                    chains: &chains,
                    reserved: chains.owned(),
                    java_release: 8,
                    member_targets: &[],
                    method: &method,
                    code,
                };
                let empty_arrays = crate::build::ArrayInitializers::default();
                let child_arrays = empty_arrays.child_facts();
                let mut direct_budget = proof_budget();
                assert!(
                    verify_array_store(
                        ssa,
                        &operations,
                        &fields,
                        &child_arrays,
                        &context,
                        block,
                        6,
                        head_index,
                        "java/lang/StringBuilder".to_owned(),
                        18,
                        *stored,
                        &mut direct_budget,
                    )
                    .expect("site verifier passes before array effect closure")
                    .is_some()
                );

                let mut budget = proof_budget();
                let mut arrays = crate::build::ArrayInitializers::prove_with_composition(
                    ssa,
                    &operations,
                    &fields,
                    Some(&context),
                    &mut budget,
                )
                .expect("handler closure is refusal, not a stop");
                if expected_array_handlers.is_empty() {
                    assert!(!arrays.has_candidate_at(1));
                    assert!(arrays.take_pending_sites().is_empty());
                } else {
                    assert!(arrays.has_candidate_at(1));
                    assert_eq!(arrays.take_pending_sites().len(), 1);
                }
            }
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
        let arrays = crate::build::ArrayInitializers::default();
        let array_facts = arrays.child_facts();
        let reserved = BTreeSet::new();
        let facts = ConstructionFacts {
            ssa,
            operations: &operations,
            chains: &crate::concat::Plan::empty(),
            reserved: &reserved,
            fields: &fields,
            arrays: &array_facts,
            java_release: 8,
            member_targets: &targets,
            method: None,
            code: &code,
        };
        verify(
            head,
            index,
            block.instructions(),
            ty.clone(),
            &facts,
            0,
            None,
        )
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
    fn a_verified_concat_is_composed_only_as_the_unique_constructor_argument() {
        for (name, descriptor) in [
            ("thrown", "(Ljava/lang/String;)Ljava/lang/String;"),
            (
                "constructed",
                "(Ljava/lang/String;)Ljava/lang/ArithmeticException;",
            ),
        ] {
            let (analysis, _) = analyzed_caller(CONCAT_CONSTRUCTOR, name, descriptor);
            let ir = analysis.ir();
            let ssa = ir.ssa().expect("ssa");
            let code = ir.code().expect("code");
            let operations = Operations::of(code, ir.constant_pool());
            let chains =
                crate::concat::plan(ssa, &operations, &crate::build::FieldCopies::default());
            let fields = field::Plan::empty();
            let arrays = crate::build::ArrayInitializers::default();
            let array_facts = arrays.child_facts();
            let method_facts = crate::facts::MethodFacts::new(name, descriptor, 0);
            let no_chain_proof = crate::concat::Plan::empty();
            let reserved_only = sites(
                ssa,
                &operations,
                &no_chain_proof,
                chains.owned(),
                &fields,
                &arrays,
                8,
                &[],
                &method_facts,
                code,
            );
            let reserved_refusal = reserved_only
                .refusals()
                .next()
                .expect("reserved ownership cannot prove a nested argument");
            assert_eq!(reserved_refusal.code(), "jre_new_interleaved_effect");
            let sites = sites(
                ssa,
                &operations,
                &chains,
                chains.owned(),
                &fields,
                &arrays,
                8,
                &[],
                &method_facts,
                code,
            );
            let chain = chains
                .value_at(20)
                .expect("the Java 8 concat is proved at its toString tail");
            let site = sites.site_at_head(0).unwrap_or_else(|| {
                panic!(
                    "the outer exception construction is proved: {:?}",
                    sites.refusals().collect::<Vec<_>>()
                )
            });
            assert!(site.arguments.contains(&chain.tail));
            assert!(site.owned.is_disjoint(chains.owned()));
            assert!(site.expression.contains(&chain.head));
            assert!(site.expression.contains(&chain.tail));

            let block = ssa
                .blocks()
                .iter()
                .find(|block| {
                    block
                        .instructions()
                        .iter()
                        .any(|instruction| instruction.bci() == 0)
                })
                .expect("outer allocation block");
            let block = block.instructions();
            let index = block
                .iter()
                .position(|instruction| instruction.bci() == 0)
                .expect("outer allocation index");
            let ctor = block
                .iter()
                .find(|instruction| instruction.bci() == site.constructor)
                .expect("outer constructor instruction");
            let wrong_value = chain
                .owned
                .iter()
                .filter(|bci| **bci != chain.tail)
                .find(|bci| matches!(operations.get(**bci), Some(Operation::Load { .. })))
                .and_then(|bci| stack_value_written_at(ssa, *bci))
                .expect("the concat reads a source String before appending it");
            let wrong_dependencies = value_dependency_bcis(ssa, block, [wrong_value]);
            let wrong = verify_concat_arguments(
                site.head,
                site.dup,
                site.constructor,
                block,
                ssa,
                &chains,
                [wrong_value],
                &wrong_dependencies,
            )
            .expect("an unrelated string value does not claim the proved concat");
            assert!(wrong.is_empty());

            let concat_tail = stack_value_written_at(ssa, chain.tail).expect("tail string value");
            let tail_dependencies = value_dependency_bcis(ssa, block, [concat_tail]);
            let repeated = match verify_concat_arguments(
                site.head,
                site.dup,
                site.constructor,
                block,
                ssa,
                &chains,
                [concat_tail, concat_tail],
                &tail_dependencies,
            ) {
                Ok(_) => panic!("one proved chain cannot fill two constructor arguments"),
                Err(reason) => reason,
            };
            assert_eq!(repeated.code(), "jre_new_concat_argument");
            let crossed = match verify_concat_arguments(
                site.head,
                chain.head,
                site.constructor,
                block,
                ssa,
                &chains,
                [concat_tail],
                &tail_dependencies,
            ) {
                Ok(_) => panic!("the complete chain cannot begin at the outer dup boundary"),
                Err(reason) => reason,
            };
            assert_eq!(crossed.code(), "jre_new_concat_argument");
            assert_eq!(ctor.bci(), site.constructor);

            let mut boundary_code = code.clone();
            boundary_code
                .exception_handlers
                .push(jarde_reader::classfile::ExceptionHandlerFact {
                    ordinal: u32::try_from(boundary_code.exception_handlers.len()).unwrap(),
                    start_bci: chain.head,
                    end_bci: chain.tail + 1,
                    handler_bci: 0,
                    catch_type_index: None,
                });
            let boundary_facts = ConstructionFacts {
                ssa,
                operations: &operations,
                chains: &chains,
                reserved: chains.owned(),
                fields: &fields,
                arrays: &array_facts,
                java_release: 8,
                member_targets: &[],
                method: None,
                code: &boundary_code,
            };
            let boundary = match verify(
                site.head,
                index,
                block,
                site.class.clone(),
                &boundary_facts,
                0,
                None,
            ) {
                Ok(_) => {
                    panic!("a handler boundary cannot be crossed by the nested expression")
                }
                Err(reason) => reason,
            };
            assert_eq!(boundary.code(), "jre_new_concat_exception_boundary");
        }
    }

    fn nested_sites(
        class: &[u8],
        name: &str,
        descriptor: &str,
        handler_range: Option<(u32, u32)>,
    ) -> Sites {
        let (analysis, _) = analyzed_caller(class, name, descriptor);
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
        let fields = field::Plan::empty();
        let chains = crate::concat::Plan::empty();
        let method_facts = crate::facts::MethodFacts::new(name, descriptor, 0);
        sites(
            ssa,
            &operations,
            &chains,
            chains.owned(),
            &fields,
            &crate::build::ArrayInitializers::default(),
            8,
            &[],
            &method_facts,
            &code,
        )
    }

    /// The construction sites of a body whose argument run holds a varargs inline array chain:
    /// the same read as [`nested_sites`], with the body's own `array@1` plan proved first, the
    /// way the run's real order proves it.
    fn varargs_sites(
        class: &[u8],
        name: &str,
        descriptor: &str,
        handler_range: Option<(u32, u32)>,
    ) -> Sites {
        let (analysis, _) = analyzed_caller(class, name, descriptor);
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
        let fields = field::Plan::empty();
        let mut budget = proof_budget();
        let arrays = crate::build::ArrayInitializers::prove(ssa, &operations, &fields, &mut budget)
            .expect("array proof completes");
        let chains = crate::concat::Plan::empty();
        let method_facts = crate::facts::MethodFacts::new(name, descriptor, 0);
        sites(
            ssa,
            &operations,
            &chains,
            chains.owned(),
            &fields,
            &arrays,
            8,
            &[],
            &method_facts,
            &code,
        )
    }

    /// The two sites of one nested construction and the one text they share: the outer call's own
    /// argument run is the inner site's complete closed run, and neither site owns the other's
    /// instructions (P3 2.3 — one expression per allocation, wherever it is consumed).
    #[test]
    fn nested_argument_sites_present_the_complete_inner_construction() {
        let probe = nested_sites(NESTED_PROBE, "nested", "()Ljava/lang/String;", None);
        let outer = probe
            .site_at_head(0)
            .expect("the outer construction proves");
        let inner = probe
            .site_at_head(6)
            .expect("the nested construction proves");
        assert_eq!((outer.head, outer.dup, outer.constructor), (0, 3, 15));
        assert_eq!((inner.head, inner.dup, inner.constructor), (6, 9, 12));
        // The outer's second argument is the nested construction's completed instance: the value
        // the inner constructor call wrote through.
        assert_eq!(outer.arguments, [4, 12]);
        assert_eq!(outer.owned, [0, 3, 15].into_iter().collect::<BTreeSet<_>>());
        assert_eq!(
            outer.expression,
            [0, 3, 4, 6, 9, 10, 12, 15]
                .into_iter()
                .collect::<BTreeSet<_>>()
        );
        assert!(outer.owned.is_disjoint(&inner.owned));
        assert!(outer.expression.is_superset(&inner.expression));
        assert!(probe.refusals().next().is_none());
        assert!(
            probe
                .allocation_candidates()
                .iter()
                .all(|candidate| candidate.verified)
        );

        // `new TwoNested(new B("y"), new C("z"))`, `new Tagged("first", new B("second"))` and
        // `new TwoSame(new B("1"), new B("2"))`: two argument positions, the second position, and
        // one nested class twice — every nested run proves and folds into the outer expression.
        let variants: [(&str, u32, u32, &[u32], &[u32]); 3] = [
            ("doubleNested", 0, 22, &[4, 13], &[10, 19]),
            ("secondPosition", 0, 15, &[6], &[12]),
            ("sameClassTwice", 0, 22, &[4, 13], &[10, 19]),
        ];
        for (name, outer_head, constructor, nested_heads, nested_calls) in variants {
            let plan = nested_sites(NESTED_VARIANTS, name, "()Ljava/lang/String;", None);
            let outer = plan
                .site_at_head(outer_head)
                .unwrap_or_else(|| panic!("{name} outer proves"));
            assert_eq!(outer.constructor, constructor, "{name}");
            for (nested_head, nested_call) in nested_heads.iter().zip(nested_calls) {
                let nested = plan
                    .site_at_head(*nested_head)
                    .unwrap_or_else(|| panic!("{name} nested at BCI {nested_head} proves"));
                assert_eq!(&nested.constructor, nested_call, "{name}");
                assert!(outer.owned.is_disjoint(&nested.owned), "{name}");
                assert!(outer.expression.is_superset(&nested.expression), "{name}");
                assert!(
                    outer.arguments.contains(nested_call),
                    "{name}: the nested completed instance is one outer argument"
                );
            }
            assert!(plan.refusals().next().is_none(), "{name}");
            assert!(
                plan.allocation_candidates()
                    .iter()
                    .all(|candidate| candidate.verified),
                "{name}"
            );
        }

        // Moving the nested run's effect into the outer expression needs one exception region:
        // a handler that splits the nested interval keeps the outer construction refused.
        let crossed = nested_sites(
            NESTED_PROBE,
            "nested",
            "()Ljava/lang/String;",
            Some((6, 12)),
        );
        assert!(crossed.site_at_head(0).is_none());
        assert_eq!(
            crossed.refusals().next().expect("boundary refusal").code(),
            "jre_new_nested_exception_boundary"
        );
    }

    /// The depth boundary, updated by `recover-io-resource-finally`: a **three**-layer run presents
    /// as one expression — the IO patrol's wrapped-stream chain
    /// (`new BufferedReader(new InputStreamReader(new FileInputStream(path), "UTF-8"))`) is
    /// exactly three layers and its outermost has no place to write its `new` unless the whole
    /// chain does — while a **four**-layer run keeps its outermost refusal, so the limit is the
    /// measured boundary and not a convenience. The negatives the patrol froze stay refused: a
    /// nested value with a second purpose keeps both refusals (P3 2c.27 — one `new` expression in
    /// one place), and a nested run split across a handler boundary keeps its own.
    #[test]
    fn three_layers_present_and_a_deeper_run_keeps_its_outermost_refusal() {
        let three = nested_sites(NESTED_NEGATIVES, "threeLayer", "()Ljava/lang/String;", None);
        assert!(
            three.site_at_head(0).is_some(),
            "the outermost of three layers presents"
        );
        assert!(three.site_at_head(4).is_some(), "the middle layer proves");
        assert!(three.site_at_head(8).is_some(), "the inner layer proves");

        // The boundary one layer deeper, measured on this change's own fixture: the fourth layer's
        // own scan cannot step over a fifth, so the outermost keeps the depth refusal this family
        // states for a run too deep to spell.
        let four = nested_sites(NESTED_DEPTH, "fourLayer", "()Ljava/lang/String;", None);
        assert!(
            four.site_at_head(0).is_none(),
            "the outermost of four layers"
        );
        let refusal = four.refusals().next().expect("the outer refusal registers");
        assert!(
            refusal
                .message()
                .contains("completes inside the construction at BCI 0"),
            "the four-layer refusal is the depth boundary's own: {refusal:?}"
        );

        let double = nested_sites(NESTED_NEGATIVES, "doubleUse", "()Ljava/lang/String;", None);
        assert!(double.site_at_head(0).is_none());
        assert!(double.site_at_head(6).is_none());
        let codes: Vec<_> = double
            .refusals()
            .map(|gap| gap.code().to_string())
            .collect();
        assert_eq!(codes, ["jre_new_interleaved_effect", "jre_new_shape"]);

        let cross = nested_sites(
            NESTED_NEGATIVES,
            "crossBlock",
            "(Z)Ljava/lang/String;",
            None,
        );
        assert!(cross.site_at_head(0).is_none());
        assert!(cross.site_at_head(10).is_none());
        assert!(cross.site_at_head(22).is_none());
    }

    /// The varargs inline array chain of a construction's argument run — `new ArrayList<>(Arrays
    /// .asList(1, 2, 3))` and its empty, boxed, `HashSet` and call-element forms — proves as one
    /// site whose argument is the factory call, and the chain's own proof is the `array@1` plan the
    /// walk reads (P3 2.3, `new@1`; the chain and the bare-position spelling are that rule's).
    #[test]
    fn varargs_inline_array_argument_sites_present_the_complete_chain() {
        let w3 = varargs_sites(VARARGS_CTOR_W3, "viaArrays", "()I", None);
        let site = w3
            .site_at_head(0)
            .expect("the collection-copy construction proves");
        assert_eq!((site.head, site.dup, site.constructor), (0, 3, 32));
        assert_eq!(site.arguments, [29]);
        assert!(site.expression.contains(&5) && site.expression.contains(&28));
        assert!(w3.refusals().next().is_none());
        assert!(
            w3.allocation_candidates()
                .iter()
                .all(|candidate| candidate.verified)
        );

        // The empty varargs call is a bare allocation whose single use is the factory call.
        let empty = varargs_sites(VARARGS_CTOR_W3, "viaArraysEmpty", "()I", None);
        let site = empty
            .site_at_head(0)
            .expect("the empty varargs construction proves");
        assert_eq!((site.head, site.dup, site.constructor), (0, 3, 11));
        assert_eq!(site.arguments, [8]);
        assert!(empty.refusals().next().is_none());

        for name in [
            "viaEmptyCall",
            "viaBoxedMix",
            "viaHashSet",
            "viaCallElements",
        ] {
            let plan = varargs_sites(VARARGS_V1, name, "()I", None);
            let site = plan
                .site_at_head(0)
                .unwrap_or_else(|| panic!("{name} proves"));
            assert!(
                site.expression.contains(&5),
                "{name}: the chain allocation belongs to the expression"
            );
            assert!(plan.refusals().next().is_none(), "{name}");
            assert!(
                plan.allocation_candidates()
                    .iter()
                    .all(|candidate| candidate.verified),
                "{name}"
            );
            assert!(
                site.arguments.len() == 1
                    && site.arguments[0] > 5
                    && site.arguments[0] < site.constructor,
                "{name}: the sole argument is the factory call after the chain"
            );
        }
    }

    /// The negatives the patrol froze: a store statement inside the element run and an array that
    /// escapes to a second purpose both keep the construction's refusal — the chain is not a
    /// closed expression, and the ordinary walk quotes the bytecode exactly as before (the refusal
    /// message stays the pre-change interleaved-effect text).
    #[test]
    fn incomplete_or_double_purpose_array_chains_stay_refused() {
        for name in ["midStatement", "doubleUse"] {
            let plan = varargs_sites(VARARGS_V2, name, "()I", None);
            assert!(plan.site_at_head(0).is_none(), "{name}");
            assert_eq!(
                plan.refusals()
                    .next()
                    .unwrap_or_else(|| panic!("{name} refusal registers"))
                    .code(),
                "jre_new_interleaved_effect",
                "{name}"
            );
        }
    }

    /// A handler region that covers the outer allocation but not the array chain keeps the
    /// construction refused: writing the initializer inside the `new` expression would move the
    /// chain into a different exception region.
    #[test]
    fn a_handler_boundary_inside_the_array_chain_keeps_the_refusal() {
        let crossed = varargs_sites(VARARGS_CTOR_W3, "viaArrays", "()I", Some((0, 4)));
        assert!(crossed.site_at_head(0).is_none());
        assert_eq!(
            crossed.refusals().next().expect("boundary refusal").code(),
            "jre_new_inline_array_exception_boundary"
        );
    }

    /// The chain certificate the walk consumes states exactly the proved interval and its own
    /// consumer: another consumer judgement or a shorter argument interval states nothing.
    #[test]
    fn the_array_chain_certificate_requires_the_argument_consumer_interval() {
        let (analysis, _) = analyzed_caller(VARARGS_CTOR_W3, "viaArrays", "()I");
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("ssa");
        let operations = Operations::of(ir.code().expect("code"), ir.constant_pool());
        let arrays = crate::build::ArrayInitializers::prove(
            ssa,
            &operations,
            &field::Plan::empty(),
            &mut proof_budget(),
        )
        .expect("complete array certificate");
        let block = ssa.blocks()[0].instructions();
        let constructor = block
            .iter()
            .find(|instruction| instruction.bci() == 32)
            .expect("constructor");
        let argument = stack_operands(constructor)[1].1;
        let dependencies = value_dependency_bcis(ssa, block, [argument]);
        let serves = |consumer: u32| dependencies.contains(&consumer);
        assert!(
            arrays
                .inline_argument_chain_bcis(5, 3, 32, serves)
                .is_some()
        );
        // The factory call is the chain's consumer: a judgement that does not reach it states no
        // argument chain.
        assert!(
            arrays
                .inline_argument_chain_bcis(5, 3, 32, |consumer| consumer == 32)
                .is_none()
        );
        // The length push at BCI 4 belongs to the chain: an interval that starts after it is not
        // the closed run the proof stated.
        assert!(
            arrays
                .inline_argument_chain_bcis(5, 5, 32, serves)
                .is_none()
        );
        // A constructor bound at the chain's own consumer leaves the invocation outside the run.
        assert!(
            arrays
                .inline_argument_chain_bcis(5, 3, 29, serves)
                .is_none()
        );
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
