//! The verified concatenation chain: which `StringBuilder`/`StringBuffer` shape this layer may
//! present as `a + b + c`, and which it must leave as bytecode (P3 2.2, rule `concat@1`).
//!
//! # The shape, and why the class is part of it
//!
//! A compiler lowers `a + b` to one allocated instance, one constructor call, one `append` per
//! operand and one `toString`:
//!
//! ```text
//! new C ─ dup ─ invokespecial C.<init>()V ─ [producer] append(X)C ─ … ─ toString()Ljava/lang/String;
//! ```
//!
//! `C` is the whole reason this rule exists: a chain on `java/lang/StringBuilder` (javac from
//! release 5 on) or on `java/lang/StringBuffer` (the spelling before that, and one a source may
//! still ask for) is a concatenation, and a chain on any other class is a sequence of calls on
//! that class — whatever its method names are. Both accepted classes are named here, once
//! ([`CONCAT_CLASSES`]); a chain on anything else is not this rule's business and is left to the
//! ordinary presentation, which quotes what it cannot prove.
//!
//! # The conversion is not lost, and that is why `append` is checked per overload
//!
//! `append` is overloaded, and its overloads do **not** all mean what `+` means: `append(char[])`
//! writes the characters where `+` would write the array's `toString`, and `append(CharSequence)`
//! writes the sequence where `+` converts through `String.valueOf`. `append(char)` writes one
//! character, which is what `+` writes for it as well — but only from a **string context**, since
//! `+` on two int-shaped operands adds numbers: an expression whose first part is not a `String`
//! starts from the empty string (`crate::emit`'s `Concat` arm writes `"" +`), so the character is
//! concatenated and never added as the code unit the `append(C)` instruction pushed. So the
//! parameter type the `append` instruction really names is read out of the pool, and only the
//! overloads whose text under `+` is the same text are accepted ([`keeps_its_conversion`]). An
//! overload this rule does not know makes the *whole chain* a refusal, and the bytecode is quoted:
//! a conversion that cannot be proven is never written away.
//!
//! # The order of evaluation is the invariant
//!
//! Writing `a + f() + c` runs `f()` once, between `a` and `c`. The bytecode does the same, and the
//! presentation must not change either count or order:
//!
//! * every operand's own instruction is **owned by the chain** ([`Chain::owned`]), so it is written
//!   inside the expression and never again as a statement of its own — one `append(x)` evaluates
//!   `x` once;
//! * the operand's producer must lie **between the previous chain instruction and its own
//!   `append`**, so the left-to-right order of the written expression is the bytecode's order;
//! * nothing inside the chain's span may produce a statement — an `append` chain interleaved with a
//!   store, a void call or an increment is refused with [`Precondition::StatementFree`] named,
//!   because writing the expression would have to move that effect;
//! * the instance is never written to a local and never read outside the chain
//!   ([`crate::facts::Operation::Duplicate`]'s aliases are all accounted for), so no other
//!   instruction can observe the chain being built.
//!
//! What this rule therefore does *not* do: it never reorders, never duplicates an operand, and
//! never moves a call out of the expression it was evaluated in.

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind, Definition, PhiInput, Slot, SsaInstruction,
    SsaTable, ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use serde::Serialize;

use crate::ast::Type;
use crate::build::stack_operands;
use crate::decode::Operations;
use crate::evidence::Publication;
use crate::facts::{CompareOp, ConstantValue, FieldAccess, InvokeKind, Operation};
use crate::lambda::parse_method;
use crate::pass::{CONCAT, Precondition, RuleVersion};
use crate::refusal::{Gap, Refusal};
use crate::region::Region;
use crate::stop::{StopReason, charge, poll};

/// The classes whose chain is a string concatenation, in the order the design lists them.
///
/// `StringBuilder` is what javac has built concatenations with from release 5 on; `StringBuffer` is
/// the earlier spelling, and a source that names that type still compiles to a chain on it. Nothing
/// else is accepted: a class that happens to hold `append` and `toString` is not a concatenation.
const CONCAT_CLASSES: [&str; 2] = ["java/lang/StringBuilder", "java/lang/StringBuffer"];

/// How far one `toString`'s receiver may be traced back to the allocation that built it
/// ([`consumes_the_instance`]): each `append` return, each local store or load of the builder and
/// each step past the chain's own instructions is one. A receiver the bound runs out on is not
/// this chain's consumer — attribution stays with what the value flow states.
const RECEIVER_TRACE_BOUND: usize = 16;

/// The pass answerable for every verdict of this module.
pub(crate) const RULE: RuleVersion = CONCAT.rule();

/// What one verified chain is, in the terms the builder writes.
pub(crate) struct Chain {
    /// The BCI of the `new` that starts the chain.
    pub(crate) head: u32,
    /// The BCI of the `toString` whose value the written expression is.
    pub(crate) tail: u32,
    /// The class the chain builds, in internal form.
    pub(crate) class: String,
    /// One entry per `append`, in the order the chain calls them: its BCI and the parameter type
    /// the same instruction's own pool reference states.
    ///
    /// The type is carried as the AST states it rather than as its spelling: it is the **target
    /// position** of the part the builder writes for this `append` (`crate::ast::ConcatPart`), and
    /// the conversion that position requires is decided there. The spelled form the report publishes
    /// is [`Type::spell`], so the record and the presentation read the same pool fact once.
    pub(crate) appends: Vec<(u32, Type)>,
    /// Every BCI the chain owns: the allocation, the copy, the constructor, every operand's own
    /// instruction, every `append` and the `toString`. An owned instruction produces no statement of
    /// its own — its text is written inside the concatenation and nowhere else.
    pub(crate) owned: BTreeSet<u32>,
    /// The proved conditional materialization this chain crosses, when one does.
    ///
    /// A chain whose middle blocks are exactly one two-arm constant materialization ends in a
    /// `toString` in another block, and [`verify`]'s ordinary walk refuses that shape with
    /// `jre_concat_split`. [`verify_conditional_cut_chain`]'s certificate owns it instead, and the
    /// cut it proved is kept here because one more question is answered from it: a value produced
    /// **before** the branch may be rendered at a consumer **at or after** the join, since the
    /// blocks between the two hold nothing but the two constants the branch chose between
    /// (`crate::build::Builder::crosses_a_proved_cut`).
    pub(crate) cut: Option<Cut>,
}

/// The proved conditional materialization one chain crosses.
///
/// The branch ends the chain's own block, its two arms push the two constants the join's stack Phi
/// merges, and the chain continues in the join. Nothing else runs in those blocks, which is what
/// makes the cut transparent for the order of everything around it.
#[derive(Clone, Debug)]
pub(crate) struct Cut {
    /// The block the chain's head stands in, whose last instruction is the branch.
    pub(crate) head_block: CanonicalBlockId,
    /// The comparison whose two arms materialize the appended value.
    pub(crate) branch_bci: u32,
    /// The join both arms meet at, where the chain continues. A value crossing the cut is read in
    /// this block, so the declaration its producer writes still stands where the reader is.
    pub(crate) join_block: CanonicalBlockId,
}

/// Every chain of one body, and the refusals of the ones that were not chains.
///
/// # The plan is not the record (change `add-demand-driven-core-results`, D3)
///
/// What this type holds is the *decision*: every chain the rule verified — with the instructions it
/// owns, because the builder writes its text — and every candidate it refused, with the link that
/// failed. [`Self::records`] is where the **owning records** are built, and it is called from the
/// evidence phase *after* the artifact is committed, one record per charge, only for the positions
/// the selection holds: a run that did not select `RuleDetails` builds no `ConcatRecord` at all,
/// while every premise above is still checked for it.
pub(crate) struct Plan {
    chains: BTreeMap<u32, Chain>,
    owned: BTreeSet<u32>,
    /// Why a candidate chain was not presented, in BCI order: the internal decision every selection
    /// reports as a gap beside the records only a selected run materializes.
    refused: Vec<Refused>,
}

/// One candidate chain this rule read and did not present.
///
/// The allocation it was claimed to start at, the class that allocation named and the refusal — the
/// plan's own account of the candidate, from which both the gap every selection states and the
/// owning record a selected run materializes are written.
struct Refused {
    /// The BCI of the allocation the refused candidate starts at.
    head: u32,
    /// The class the allocation named, in internal form.
    class: String,
    /// Which link of the verification failed.
    refusal: Refusal,
}

impl Refused {
    /// The refusal as the report's own record states it.
    fn record(&self) -> ConcatRefusal {
        ConcatRefusal::of(&self.refusal, self.head)
    }

    /// The same refusal as the gap every selection carries.
    fn gap(&self) -> Gap {
        let refusal = self.record();
        Gap::at(refusal.code, self.head, refusal.message)
    }
}

/// One verdict of this rule's plan: the chain it verified, or the candidate it refused.
enum Decision<'a> {
    Presented(&'a Chain),
    Refused(&'a Refused),
}

impl Decision<'_> {
    /// The allocation the decision is about: the chains and the refused candidates are both ordered
    /// by it.
    fn head(&self) -> u32 {
        match self {
            Self::Presented(chain) => chain.head,
            Self::Refused(refused) => refused.head,
        }
    }

    /// Every driver BCI this decision states for itself: a chain's head, `toString` and every
    /// `append`, and a refused candidate's own allocation. These are the positions the driver range
    /// selects on, and the record is kept or dropped as a unit with them.
    fn positions(&self) -> Vec<u32> {
        match self {
            Self::Presented(chain) => chain_positions(chain),
            Self::Refused(refused) => vec![refused.head],
        }
    }

    /// The owning record, built here and only here.
    fn record(self) -> ConcatRecord {
        crate::demand_counts::record_built(crate::evidence::RecoveryEvidenceKind::RuleDetails);
        match self {
            Self::Presented(chain) => record_of(chain),
            Self::Refused(refused) => refused_record(refused),
        }
    }
}

impl Plan {
    /// An empty plan: a body with no candidate allocation at all.
    pub(crate) fn empty() -> Self {
        Self {
            chains: BTreeMap::new(),
            owned: BTreeSet::new(),
            refused: Vec::new(),
        }
    }

    /// Whether one instruction belongs to a verified chain and so produces no statement of its own.
    pub(crate) fn owns(&self, bci: u32) -> bool {
        self.owned.contains(&bci)
    }

    /// Every BCI the verified chains own.
    ///
    /// A shape decided after this one reserves these: the allocation a verified chain builds is
    /// written inside the `+` expression, and one instruction is never two shapes (P3 2.3).
    pub(crate) fn owned(&self) -> &BTreeSet<u32> {
        &self.owned
    }

    /// The chain whose value the instruction at one BCI produces, when one does.
    pub(crate) fn value_at(&self, bci: u32) -> Option<&Chain> {
        self.chains.get(&bci)
    }

    /// The proved conditional cut a verified chain crosses in one block, when one does.
    ///
    /// A value produced in that block **before** the branch is read by consumers at or after the
    /// join, and the blocks between the two hold nothing but the two constants the branch chose
    /// between — so the value's own evaluation point and the consumer's expression position differ
    /// by a materialization that performs no effect, and the deferred-binding plan may read its
    /// readers across the cut (`crate::build::Builder::crosses_a_proved_cut`).
    pub(crate) fn cut_of(&self, block: &CanonicalBlockId) -> Option<&Cut> {
        self.chains
            .values()
            .find_map(|chain| chain.cut.as_ref().filter(|cut| &cut.head_block == block))
    }

    /// Every candidate chain the rule refused, in BCI order.
    pub(crate) fn refusals(&self) -> impl Iterator<Item = Gap> + '_ {
        self.refused.iter().map(Refused::gap)
    }

    /// Whether this rule decided anything about this body: it verified a chain or refused a
    /// candidate. The question the rule index is built from, and it does not depend on the evidence
    /// selection or on a driver range.
    pub(crate) fn answered(&self) -> bool {
        !self.chains.is_empty() || !self.refused.is_empty()
    }

    /// How many candidate chains the rule read, and how many of them it presented.
    pub(crate) fn counts(&self) -> (u64, u64) {
        let presented = u64::try_from(self.chains.len()).unwrap_or(u64::MAX);
        let refused = u64::try_from(self.refused.len()).unwrap_or(u64::MAX);
        (presented + refused, presented)
    }

    /// The owning records this plan publishes under `publication`, in BCI order, within the phase's
    /// remaining allowance.
    ///
    /// The decision is taken for every candidate; what the selection decides is which records exist.
    /// A record whose positions are outside the selected range is not built at all, and a record the
    /// phase cannot pay for ends the category with the prefix it already built.
    pub(crate) fn materialize(
        &self,
        publication: Publication,
        phase: &mut crate::evidence::EvidencePhase,
        budget: &mut jarde_reader::budget::Budget,
    ) -> (Vec<ConcatRecord>, crate::evidence::Materialized) {
        let mut decisions: Vec<Decision<'_>> = self
            .chains
            .values()
            .map(Decision::Presented)
            .chain(self.refused.iter().map(Decision::Refused))
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

/// One `append` of a verified chain.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConcatAppend {
    /// The BCI of the `append` instruction.
    pub bci: u32,
    /// The parameter type the instruction's own pool reference states, in source form.
    pub parameter: String,
}

/// What one candidate chain was presented as, or why it was not (P3 2.2).
///
/// This is the record the concatenation acceptance asks for: the allocation that was claimed, the
/// `toString` it ended in (when one was reached), the class the chain really built, and every
/// `append` with its own BCI and the parameter type the class's pool states for it. A refusal
/// states the same head and class and adds the link that failed — so "why is this chain not a
/// `+`" is answered by the run's own record rather than by comparing texts.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConcatRecord {
    /// The BCI of the allocation the chain was claimed to start at.
    pub head: u32,
    /// The BCI of the `toString` it ended in, when the walk reached one.
    pub tail: Option<u32>,
    /// The class the chain builds, in internal form, as the `new`'s own pool entry states it.
    pub class: String,
    /// Every `append` the walk read, in call order.
    pub appends: Vec<ConcatAppend>,
    /// Whether the chain was presented as a concatenation.
    pub presented: bool,
    /// Why it was not, when it was not.
    pub refusal: Option<ConcatRefusal>,
}

impl ConcatRecord {
    /// Whether this candidate was presented.
    pub fn presented(&self) -> bool {
        self.presented
    }

    /// The rule this record is answerable to: the one that presented the chain or refused it.
    pub fn rule(&self) -> RuleVersion {
        RULE
    }
}

/// Why one candidate chain was not presented.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConcatRefusal {
    /// The diagnostic code, in the recovery layer's own vocabulary: one `jre_concat_*` per link of
    /// the chain's verification that can fail.
    pub code: &'static str,
    /// The rule that refused the chain.
    pub rule: RuleVersion,
    /// The declared requirement that fell short, when the refusal is one of the rule's own
    /// preconditions rather than a shape that simply is not this one.
    pub requirement: Option<String>,
    /// One sentence stating which link failed, with the chain's BCI in it.
    pub message: String,
}

impl ConcatRefusal {
    /// The refusal of one chain, as the report records it.
    pub(crate) fn of(refusal: &Refusal, head: u32) -> Self {
        Self {
            code: refusal.code(),
            rule: RULE,
            requirement: refusal.requirement().map(Precondition::describe),
            message: format!(
                "the concatenation at BCI {head} was not presented: {}",
                refusal.message()
            ),
        }
    }
}

/// Reads every concatenation chain of one body.
///
/// The walk runs over the **SSA blocks**, because a chain is a contiguous run of instructions of
/// one block: a chain a branch cuts in two is not this shape and is refused with the split stated
/// rather than presented as an expression the bytecode never evaluated as one.
///
/// Every premise and every refusal is decided here, for every evidence selection; what this function
/// does **not** do is build the owning records. The verified chains and the refused candidates stay
/// in the [`Plan`], and [`Plan::materialize`] writes the records from them after the artifact is
/// committed.
pub(crate) fn plan(ssa: &SsaTable, operations: &Operations) -> Plan {
    let blocks: Vec<Vec<&SsaInstruction>> = ssa
        .blocks()
        .iter()
        .map(|block| block.instructions().iter().collect())
        .collect();
    let all: Vec<&SsaInstruction> = blocks.iter().flatten().copied().collect();
    let mut block_of: BTreeMap<u32, usize> = BTreeMap::new();
    for (index, block) in blocks.iter().enumerate() {
        for instruction in block {
            block_of.insert(instruction.bci(), index);
        }
    }
    let mut plan = Plan::empty();
    for (head_block, block) in blocks.iter().enumerate() {
        for head in block.iter().map(|instruction| instruction.bci()) {
            let Some(Operation::Allocate { ty }) = operations.get(head) else {
                continue;
            };
            if !CONCAT_CLASSES.contains(&ty.as_str()) {
                // Not a candidate: a chain on a class this rule does not know is an ordinary
                // sequence of calls on that class, and nothing here claims anything about it.
                continue;
            }
            let ty = ty.clone();
            match verify(
                head, &ty, head_block, &block_of, &all, block, ssa, operations,
            ) {
                Ok(chain) => {
                    if let Some(shared) = chain.owned.iter().find(|bci| plan.owned.contains(bci)) {
                        plan.refused.push(Refused {
                            head,
                            class: ty,
                            refusal: Refusal::shape(
                                "jre_concat_overlap",
                                format!(
                                    "the instruction at BCI {shared} belongs to another chain this run already claimed, and one instruction is not two concatenations"
                                ),
                            ),
                        });
                        continue;
                    }
                    for bci in &chain.owned {
                        plan.owned.insert(*bci);
                    }
                    plan.chains.insert(chain.tail, chain);
                }
                Err(refusal) => plan.refused.push(Refused {
                    head,
                    class: ty,
                    refusal,
                }),
            }
        }
    }
    plan.refused.sort_by_key(|refused| refused.head);
    plan
}

/// The one bounded branched form used by Java 8's four conditional String operands.
/// The ordinary same-block rule and its `jre_concat_split` refusal remain unchanged. A
/// successful certificate replaces that refusal only after the entire physical method, CFG,
/// four stack Phis and one unaliased builder have been checked as a unit.
pub(crate) fn plan_four_conditional_strings(
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    operations: &Operations,
    budget: &mut Budget,
) -> Result<Plan, StopReason> {
    let mut plan = plan(ssa, operations);
    let Some(index) = plan
        .refused
        .iter()
        .position(|candidate| candidate.refusal.code() == "jre_concat_split")
    else {
        return Ok(plan);
    };
    let head = plan.refused[index].head;
    if let Some(chain) = verify_four_conditional_strings(head, ssa, canonical, operations, budget)?
    {
        if chain.owned.is_disjoint(&plan.owned) {
            plan.refused.remove(index);
            plan.owned.extend(&chain.owned);
            plan.chains.insert(chain.tail, chain);
        }
    }
    Ok(plan)
}

fn verify_four_conditional_strings(
    head: u32,
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    operations: &Operations,
    budget: &mut Budget,
) -> Result<Option<Chain>, StopReason> {
    let all: Vec<_> = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions().iter())
        .collect();
    let scan = all
        .len()
        .saturating_add(ssa.phis().len())
        .saturating_add(canonical.edges().len());
    poll(budget, Some(head))?;
    charge(
        budget,
        CountedBudgetDimension::IrItems,
        u64::try_from(scan).unwrap_or(u64::MAX),
        Some(head),
    )?;
    // This certificate deliberately owns one complete method. A statement before, within or
    // after the chain would need a separate evaluation-position proof.
    if all.len() != 33 || canonical.unreachable().len() != 0 {
        return Ok(None);
    }
    // At most 64 value readers chase at most 16 Phi levels; each level scans the Phi
    // table. Reserve that complete work before following any recursive builder identity.
    let phi_walk = 64usize.saturating_mul(17).saturating_mul(ssa.phis().len());
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(scan.saturating_add(phi_walk)).unwrap_or(u64::MAX),
        Some(head),
    )?;
    let mut ordered = all;
    ordered.sort_by_key(|instruction| instruction.bci());
    if ordered[0].bci() != head {
        return Ok(None);
    }
    let op = |index: usize| operations.get(ordered[index].bci());
    let Some(Operation::Allocate { ty }) = op(0) else {
        return Ok(None);
    };
    if ty != "java/lang/StringBuilder" || op(1) != Some(&Operation::Duplicate) {
        return Ok(None);
    }
    let Some(Operation::Invoke(init)) = op(2) else {
        return Ok(None);
    };
    if init.owner() != ty || init.name() != "<init>" || init.descriptor() != "()V" {
        return Ok(None);
    }
    let Some(Operation::Invoke(to_string)) = op(31) else {
        return Ok(None);
    };
    if to_string.owner() != ty
        || to_string.name() != "toString"
        || to_string.descriptor() != "()Ljava/lang/String;"
        || op(32) != Some(&Operation::Return)
    {
        return Ok(None);
    }
    let mut block_of = BTreeMap::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            block_of.insert(instruction.bci(), block.block().clone());
        }
    }
    let mut producers = vec![ordered[0].bci(), ordered[1].bci(), ordered[2].bci()];
    if !stack_operands(ordered[1])
        .first()
        .is_some_and(|(_, value)| is_the_instance(ssa, *value, &producers[..1]))
    {
        return Ok(None);
    }
    if !stack_operands(ordered[2])
        .first()
        .is_some_and(|(_, value)| is_the_instance(ssa, *value, &producers))
    {
        return Ok(None);
    }
    let mut appends = Vec::with_capacity(4);
    let mut previous_append = None;
    let mut field_owner = None;
    let mut field_names = BTreeSet::new();
    for part in 0..4 {
        poll(budget, Some(head))?;
        let base = 3 + 7 * part;
        let [
            load,
            getter,
            branch,
            true_value,
            transfer,
            false_value,
            append,
        ] = &ordered[base..base + 7]
        else {
            unreachable!()
        };
        if !matches!(op(base), Some(Operation::Load { slot: 0 }))
            || !matches!(op(base + 2), Some(Operation::Comparison { .. }))
            || !matches!(
                op(base + 3),
                Some(Operation::Push(ConstantValue::String(_)))
            )
            || op(base + 4) != Some(&Operation::Transfer)
            || !matches!(
                op(base + 5),
                Some(Operation::Push(ConstantValue::String(_)))
            )
        {
            return Ok(None);
        }
        let getter_valid = match op(base + 1) {
            Some(Operation::Field {
                access: FieldAccess::Read,
                is_static: false,
                owner,
                name,
                descriptor,
            }) if part < 3 && descriptor == "Z" => {
                if field_owner.as_ref().is_some_and(|known| known != owner)
                    || !field_names.insert(name.clone())
                {
                    false
                } else {
                    field_owner.get_or_insert_with(|| owner.clone());
                    true
                }
            }
            Some(Operation::Invoke(target))
                if part == 3
                    && target.kind() == InvokeKind::Static
                    && field_owner.as_ref().is_some_and(|owner| {
                        target.owner() == owner && target.descriptor() == format!("(L{owner};)Z")
                    }) =>
            {
                true
            }
            _ => false,
        };
        if !getter_valid
            || stack_operands(getter).last().map(|(_, value)| *value)
                != load
                    .writes()
                    .iter()
                    .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
        {
            return Ok(None);
        }
        let getter_value = getter
            .writes()
            .iter()
            .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value));
        if stack_operands(branch).last().map(|(_, value)| *value) != getter_value {
            return Ok(None);
        }
        let Some(Operation::Invoke(target)) = op(base + 6) else {
            return Ok(None);
        };
        if target.owner() != ty
            || target.name() != "append"
            || target.descriptor() != "(Ljava/lang/String;)Ljava/lang/StringBuilder;"
        {
            return Ok(None);
        }
        let Some(test_block) = block_of.get(&branch.bci()) else {
            return Ok(None);
        };
        let (Some(true_block), Some(false_block), Some(join_block)) = (
            block_of.get(&true_value.bci()),
            block_of.get(&false_value.bci()),
            block_of.get(&append.bci()),
        ) else {
            return Ok(None);
        };
        if block_of.get(&load.bci()) != Some(test_block)
            || block_of.get(&getter.bci()) != Some(test_block)
            || block_of.get(&transfer.bci()) != Some(true_block)
            || test_block == true_block
            || test_block == false_block
            || true_block == false_block
            || join_block == true_block
            || join_block == false_block
            || !matches!(op(base + 2), Some(Operation::Comparison { target, .. }) if *target == false_block.bci())
        {
            return Ok(None);
        }
        let edges = canonical.edges();
        let successors = |from: &_| {
            edges
                .iter()
                .filter(|edge| edge.from() == from)
                .collect::<Vec<_>>()
        };
        let test_edges = successors(test_block);
        let true_edges = successors(true_block);
        let false_edges = successors(false_block);
        if test_edges.len() != 2
            || true_edges.len() != 1
            || false_edges.len() != 1
            || test_edges
                .iter()
                .any(|edge| edge.kind() != CanonicalEdgeKind::Normal)
            || !test_edges.iter().any(|edge| edge.to() == true_block)
            || !test_edges.iter().any(|edge| edge.to() == false_block)
            || true_edges[0].kind() != CanonicalEdgeKind::Normal
            || true_edges[0].to() != join_block
            || false_edges[0].kind() != CanonicalEdgeKind::Normal
            || false_edges[0].to() != join_block
            || edges.iter().filter(|edge| edge.to() == join_block).count() != 2
        {
            return Ok(None);
        }
        let phi = ssa.phis().iter().find(|phi| {
            phi.block() == join_block
                && matches!(phi.slot(), Slot::Stack(_))
                && phi.inputs().len() == 2
                && phi.inputs()[0] != phi.inputs()[1]
                && stack_operands(append)
                    .last()
                    .is_some_and(|(_, value)| *value == phi.value())
        });
        let Some(phi) = phi else { return Ok(None) };
        let true_id = true_value
            .writes()
            .iter()
            .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value));
        let false_id = false_value
            .writes()
            .iter()
            .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value));
        if !matches!((true_id, false_id), (Some(t), Some(f))
            if phi.inputs().contains(&PhiInput::Value(t)) && phi.inputs().contains(&PhiInput::Value(f)))
            || ssa.value(phi.value()).uses().len() != 1
            || ssa.value(phi.value()).uses()[0].bci() != Some(append.bci())
        {
            return Ok(None);
        }
        if !ssa.block(true_block).is_some_and(|block| {
            block
                .exit()
                .iter()
                .any(|(slot, value)| *slot == phi.slot() && Some(*value) == true_id)
        }) || !ssa.block(false_block).is_some_and(|block| {
            block
                .exit()
                .iter()
                .any(|(slot, value)| *slot == phi.slot() && Some(*value) == false_id)
        }) {
            return Ok(None);
        }
        let append_operands = stack_operands(append);
        if append_operands.len() != 2 || !builder_origin(ssa, append_operands[0].1, &producers, 0) {
            return Ok(None);
        }
        // The earlier append must be the only builder producer carried to this test block.
        if let Some(previous) = previous_append
            && !builder_origin(ssa, append_operands[0].1, &[previous], 0)
        {
            return Ok(None);
        }
        producers.push(append.bci());
        previous_append = Some(append.bci());
        appends.push((append.bci(), Type::Reference("java.lang.String".into())));
    }
    let tail_operands = stack_operands(ordered[31]);
    let return_operands = stack_operands(ordered[32]);
    let tail_value = ordered[31]
        .writes()
        .iter()
        .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value));
    if tail_operands.len() != 1
        || !builder_origin(ssa, tail_operands[0].1, &[producers[6]], 0)
        || return_operands.len() != 1
        || tail_value != Some(return_operands[0].1)
    {
        return Ok(None);
    }
    // No protected instruction, exceptional successor, or unaccounted builder observer may be
    // folded into this single expression. Every opcode of this method was matched above.
    if canonical
        .throw_sites()
        .iter()
        .any(|site| !site.handlers().is_empty())
    {
        return Ok(None);
    }
    for instruction in &ordered {
        let at = instruction.bci();
        if [ordered[1].bci(), ordered[2].bci(), ordered[31].bci()].contains(&at)
            || appends.iter().any(|(append, _)| *append == at)
        {
            continue;
        }
        if instruction
            .reads()
            .iter()
            .any(|(_, value)| builder_origin(ssa, *value, &producers, 0))
        {
            return Ok(None);
        }
    }
    Ok(Some(Chain {
        head,
        tail: ordered[31].bci(),
        class: ty.clone(),
        appends,
        owned: ordered[..32]
            .iter()
            .map(|instruction| instruction.bci())
            .collect(),
        cut: None,
    }))
}

/// The bounded branched form of an **inline conditional value as a chain operand**: a `+` chain
/// whose middle blocks are exactly one proved conditional-value materialization.
///
/// `"" + a + (x == y) + b` lowers the comparison to a branch whose two arms push the `0`/`1` the
/// join's stack Phi merges, and the chain continues in the join — so its `toString` stands in
/// another block and [`verify`]'s ordinary same-block walk states `jre_concat_split`. The
/// certificate below replaces that refusal for exactly this shape, and for nothing else: the
/// ordinary rule, its text and the walk that is this rule's admission authority stay untouched.
///
/// The two-arm proof is [`crate::build::prove_conditional_value`]'s — the one `recover-conditional-values`
/// established — read here as a sub-proof: the certificate states the region the branch and its
/// successors form and hands it to that proof, which re-validates every edge, arm entry, Phi input
/// and consumer against the canonical CFG. Nothing about the arms is assumed from the shape that
/// named them. What the certificate adds on top is what the chain needs and that proof does not
/// state: each arm holds **nothing but** the constant it pushes and (for the arm that does not fall
/// through) the transfer to the join, the constants are `0` and `1`, the Phi's one consumer is the
/// chain's own `append(Z)`, and the whole span of the chain — the head block's run to the branch,
/// both arms and the join's run to the `toString` — holds nothing but the chain's own instructions
/// and the operand producers this layer writes inside the expression.
pub(crate) fn plan_conditional_cut_chains(
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    operations: &Operations,
    budget: &mut Budget,
) -> Result<Plan, StopReason> {
    let mut plan = plan_four_conditional_strings(ssa, canonical, operations, budget)?;
    let mut index = 0;
    while index < plan.refused.len() {
        if plan.refused[index].refusal.code() != "jre_concat_split" {
            index += 1;
            continue;
        }
        let head = plan.refused[index].head;
        let class = plan.refused[index].class.clone();
        match verify_conditional_cut_chain(head, &class, ssa, canonical, operations, budget)? {
            Some(chain) if chain.owned.is_disjoint(&plan.owned) => {
                plan.refused.remove(index);
                plan.owned.extend(&chain.owned);
                plan.chains.insert(chain.tail, chain);
            }
            _ => index += 1,
        }
    }
    Ok(plan)
}

/// Verifies one candidate whose cross-block `toString` a conditional materialization cuts.
///
/// `None` is the answer for every shape this certificate does not own, and it is the ordinary
/// refusal's answer too: the candidate keeps the `jre_concat_split` its own attribution stated.
fn verify_conditional_cut_chain(
    head: u32,
    ty: &str,
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    operations: &Operations,
    budget: &mut Budget,
) -> Result<Option<Chain>, StopReason> {
    let all: Vec<&SsaInstruction> = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions().iter())
        .collect();
    poll(budget, Some(head))?;
    charge(
        budget,
        CountedBudgetDimension::IrItems,
        u64::try_from(
            all.len()
                .saturating_add(ssa.phis().len())
                .saturating_add(canonical.edges().len()),
        )
        .unwrap_or(u64::MAX),
        Some(head),
    )?;
    let Some(head_block) = ssa.blocks().iter().find(|block| {
        block
            .instructions()
            .iter()
            .any(|instruction| instruction.bci() == head)
    }) else {
        return Ok(None);
    };
    let block = head_block.instructions();
    let block_refs: Vec<&SsaInstruction> = block.iter().collect();
    let Some(index) = block
        .iter()
        .position(|instruction| instruction.bci() == head)
    else {
        return Ok(None);
    };
    // The allocation's own triple, read exactly as the walk reads it.
    let Some(copy) = block.get(index + 1) else {
        return Ok(None);
    };
    if operations.get(copy.bci()) != Some(&Operation::Duplicate) {
        return Ok(None);
    }
    let Some(init) = block.get(index + 2) else {
        return Ok(None);
    };
    if !matches!(
        operations.get(init.bci()),
        Some(Operation::Invoke(target)) if target.owner() == ty && target.name() == "<init>"
    ) {
        return Ok(None);
    }
    let mut produced_by = vec![head, copy.bci(), init.bci()];
    let mut owned: BTreeSet<u32> = BTreeSet::from([head, copy.bci(), init.bci()]);
    let mut appends: Vec<(u32, Type)> = Vec::new();
    let mut previous = init.bci();
    // The head block's run ends in the comparison that cuts the chain. Every instruction of that
    // run is owned, and the branch is the block's last instruction because both arms leave it.
    let mut branch_bci = None;
    for instruction in &block[index + 3..] {
        poll(budget, Some(instruction.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        let at = instruction.bci();
        match operations.get(at) {
            Some(Operation::Invoke(target))
                if target.owner() == ty && target.name() == "append" =>
            {
                let Some((params, returns)) = parse_method(target.descriptor()) else {
                    return Ok(None);
                };
                if params.len() != 1
                    || !keeps_its_conversion(&params[0])
                    || !matches!(returns, Some(Type::Reference(name)) if name == source_name(ty))
                {
                    return Ok(None);
                }
                let Some((_, receiver)) = stack_operands(instruction).first().copied() else {
                    return Ok(None);
                };
                if !is_the_instance(ssa, receiver, &produced_by) {
                    return Ok(None);
                }
                let Some((_, value)) = stack_operands(instruction).last().copied() else {
                    return Ok(None);
                };
                let Some(produced) = produced_at(ssa, value) else {
                    return Ok(None);
                };
                if produced <= previous || produced >= at {
                    return Ok(None);
                }
                appends.push((at, params[0].clone()));
                owned.insert(at);
                produced_by.push(at);
                previous = at;
            }
            // The chain does not end in its own block: this is not the shape this certificate
            // owns, and the candidate keeps the refusal its own attribution stated.
            Some(Operation::Invoke(target))
                if target.owner() == ty && target.name() == "toString" =>
            {
                return Ok(None);
            }
            Some(Operation::Comparison { .. }) => {
                if at != block[block.len() - 1].bci() {
                    return Ok(None);
                }
                branch_bci = Some(at);
                owned.insert(at);
                break;
            }
            Some(
                Operation::Push(_)
                | Operation::Load { .. }
                | Operation::Arithmetic { .. }
                | Operation::Negate,
            ) => {
                owned.insert(at);
            }
            // A field read is a value whose text lands where it is consumed — the same reading the
            // statement walk makes of a claimed read — so it is an operand producer of the chain.
            // The rendering refuses a read the field plan did not claim, so an unproved access
            // leaves the method quoted rather than dropping the instruction.
            Some(Operation::Field {
                access: FieldAccess::Read,
                ..
            }) => {
                owned.insert(at);
            }
            Some(Operation::Invoke(_)) if produces_a_read_value(instruction, &block_refs, at) => {
                owned.insert(at);
            }
            _ => return Ok(None),
        }
    }
    let Some(branch_bci) = branch_bci else {
        return Ok(None);
    };
    // The branch's own two successors, in the region layer's order: the fall-through arm is the
    // `then` arm, the taken target the `else` arm. A branch with any other edge — an exceptional
    // one, or a second target — is not this shape.
    let Some(branch_block) = ssa.blocks().iter().find(|block| {
        block
            .instructions()
            .iter()
            .any(|instruction| instruction.bci() == branch_bci)
    }) else {
        return Ok(None);
    };
    let Some(Operation::Comparison { op, target }) = operations.get(branch_bci) else {
        return Ok(None);
    };
    if !matches!(op, CompareOp::JumpIfSame | CompareOp::JumpIfDifferent) {
        return Ok(None);
    }
    let mut successors = Vec::new();
    for edge in canonical.edges() {
        if edge.from() != branch_block.block() {
            continue;
        }
        if edge.kind() != CanonicalEdgeKind::Normal {
            return Ok(None);
        }
        successors.push(edge.to().clone());
    }
    let taken: Vec<&CanonicalBlockId> = successors
        .iter()
        .filter(|successor| successor.bci() == *target)
        .collect();
    if successors.len() != 2 || taken.len() != 1 {
        return Ok(None);
    }
    let else_entry = taken[0].clone();
    let then_entry = successors
        .iter()
        .find(|successor| **successor != else_entry)
        .cloned()
        .expect("one of the two successors is not the taken target");
    let (Some(then_block), Some(else_block)) = (ssa.block(&then_entry), ssa.block(&else_entry))
    else {
        return Ok(None);
    };
    // Each arm holds the constant it pushes and, where it does not fall through into the join, the
    // transfer that carries it there. Nothing else runs in an arm: that is the "no other side
    // effects" the shape is admitted with, and it is what makes the cut transparent for the order
    // of the values around it.
    for arm in [then_block, else_block] {
        let instructions = arm.instructions();
        let Some((constant, rest)) = instructions.split_first() else {
            return Ok(None);
        };
        if !matches!(
            operations.get(constant.bci()),
            Some(Operation::Push(ConstantValue::Int(0 | 1)))
        ) {
            return Ok(None);
        }
        match rest {
            [] => {}
            [transfer] if operations.get(transfer.bci()) == Some(&Operation::Transfer) => {}
            _ => return Ok(None),
        }
        // The arm's instructions are the chain's own operand: the text writes the constant as the
        // comparison's arm, so neither instruction may write a statement of its own.
        for instruction in instructions {
            owned.insert(instruction.bci());
        }
    }
    // The join: the one block both arms hand their value to.
    let mut arm_successors = Vec::new();
    for arm in [then_block, else_block] {
        let mut exits = Vec::new();
        for edge in canonical.edges() {
            if edge.from() != arm.block() {
                continue;
            }
            if edge.kind() != CanonicalEdgeKind::Normal {
                return Ok(None);
            }
            exits.push(edge.to().clone());
        }
        if exits.len() != 1 {
            return Ok(None);
        }
        arm_successors.push(exits[0].clone());
    }
    if arm_successors[0] != arm_successors[1] {
        return Ok(None);
    }
    let join = arm_successors[0].clone();
    // The two-arm proof itself, read-only: the region this certificate states is re-validated
    // against the canonical CFG by the proof that owns it.
    let region = Region::If {
        prefix: Vec::new(),
        branch: branch_block.block().clone(),
        branch_bci,
        then_arm: Box::new(Region::Straight {
            blocks: vec![then_entry.clone()],
        }),
        else_arm: Box::new(Region::Straight {
            blocks: vec![else_entry.clone()],
        }),
        join: Some(join.clone()),
    };
    let crate::build::ConditionalValueAttempt::Proved(proof) =
        crate::build::prove_conditional_value(&region, canonical, ssa, operations, budget)?
    else {
        return Ok(None);
    };
    if proof.branch_bci != branch_bci || proof.join != join {
        return Ok(None);
    }
    // The Phi's one consumer is this chain's own `append(Z)`: the value enters the chain's argument
    // position as the boolean it is, and the presentation of a `0`/`1` pair beside an equality
    // branch is the comparison itself (`crate::build`'s `boolean_position_values`).
    let Some(consumer) = all
        .iter()
        .find(|instruction| instruction.bci() == proof.consumer_bci)
    else {
        return Ok(None);
    };
    let Some(Operation::Invoke(target)) = operations.get(proof.consumer_bci) else {
        return Ok(None);
    };
    let Some((params, returns)) = parse_method(target.descriptor()) else {
        return Ok(None);
    };
    if target.owner() != ty
        || target.name() != "append"
        || params.as_slice() != [Type::Boolean]
        || !matches!(returns, Some(Type::Reference(name)) if name == source_name(ty))
    {
        return Ok(None);
    }
    let Some((_, receiver)) = stack_operands(consumer).first().copied() else {
        return Ok(None);
    };
    if !is_the_instance(ssa, receiver, &produced_by) {
        return Ok(None);
    }
    let Some((_, value)) = stack_operands(consumer).last().copied() else {
        return Ok(None);
    };
    if value != proof.phi {
        return Ok(None);
    }
    let Some(join_block) = ssa.block(&proof.join) else {
        return Ok(None);
    };
    let join_refs: Vec<&SsaInstruction> = join_block.instructions().iter().collect();
    let Some(join_index) = join_block
        .instructions()
        .iter()
        .position(|instruction| instruction.bci() == proof.consumer_bci)
    else {
        return Ok(None);
    };
    // The Phi is read where the join begins, so nothing runs between the arms and the `append`.
    if join_index != 0 {
        return Ok(None);
    }
    appends.push((proof.consumer_bci, Type::Boolean));
    owned.insert(proof.consumer_bci);
    produced_by.push(proof.consumer_bci);
    previous = proof.consumer_bci;
    // The join's own run: the rest of the chain, ending in the `toString` its `String` is read at.
    let mut tail = None;
    for instruction in &join_block.instructions()[join_index + 1..] {
        poll(budget, Some(instruction.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        let at = instruction.bci();
        match operations.get(at) {
            Some(Operation::Invoke(target))
                if target.owner() == ty && target.name() == "append" =>
            {
                let Some((params, returns)) = parse_method(target.descriptor()) else {
                    return Ok(None);
                };
                if params.len() != 1
                    || !keeps_its_conversion(&params[0])
                    || !matches!(returns, Some(Type::Reference(name)) if name == source_name(ty))
                {
                    return Ok(None);
                }
                let Some((_, receiver)) = stack_operands(instruction).first().copied() else {
                    return Ok(None);
                };
                if !is_the_instance(ssa, receiver, &produced_by) {
                    return Ok(None);
                }
                let Some((_, value)) = stack_operands(instruction).last().copied() else {
                    return Ok(None);
                };
                let Some(produced) = produced_at(ssa, value) else {
                    return Ok(None);
                };
                if produced <= previous || produced >= at {
                    return Ok(None);
                }
                appends.push((at, params[0].clone()));
                owned.insert(at);
                produced_by.push(at);
                previous = at;
            }
            Some(Operation::Invoke(target))
                if target.owner() == ty && target.name() == "toString" =>
            {
                if target.descriptor() != "()Ljava/lang/String;" {
                    return Ok(None);
                }
                let Some((_, receiver)) = stack_operands(instruction).first().copied() else {
                    return Ok(None);
                };
                if !is_the_instance(ssa, receiver, &produced_by) {
                    return Ok(None);
                }
                owned.insert(at);
                tail = Some(at);
                break;
            }
            Some(
                Operation::Push(_)
                | Operation::Load { .. }
                | Operation::Arithmetic { .. }
                | Operation::Negate,
            ) => {
                owned.insert(at);
            }
            Some(Operation::Field {
                access: FieldAccess::Read,
                ..
            }) => {
                owned.insert(at);
            }
            Some(Operation::Invoke(_)) if produces_a_read_value(instruction, &join_refs, at) => {
                owned.insert(at);
            }
            _ => return Ok(None),
        }
    }
    let Some(tail) = tail else {
        return Ok(None);
    };
    // The `String` is read where it is consumed, and no instruction outside the span reads the
    // instance the chain builds.
    let produced = written_value(ssa, tail);
    let consumed = produced.is_some_and(|value| {
        all.iter().any(|instruction| {
            instruction.bci() != tail
                && instruction.reads().iter().any(|(_, read)| *read == value)
                && renders_its_reads(operations.get(instruction.bci()))
        })
    });
    if !consumed {
        return Ok(None);
    }
    for instruction in &all {
        if owned.contains(&instruction.bci()) {
            continue;
        }
        if instruction
            .reads()
            .iter()
            .any(|(_, read)| is_the_instance(ssa, *read, &produced_by))
        {
            return Ok(None);
        }
    }
    if appends.len() < 2 {
        return Ok(None);
    }
    Ok(Some(Chain {
        head,
        tail,
        class: ty.to_string(),
        appends,
        owned,
        cut: Some(Cut {
            head_block: head_block.block().clone(),
            branch_bci,
            join_block: proof.join.clone(),
        }),
    }))
}

fn builder_origin(ssa: &SsaTable, value: ValueId, producers: &[u32], depth: usize) -> bool {
    if depth > 16 {
        return false;
    }
    match ssa.value(value).def() {
        Definition::Instruction { bci, .. } => producers.contains(bci),
        Definition::Phi { .. } => ssa
            .phis()
            .iter()
            .find(|phi| phi.value() == value)
            .is_some_and(|phi| match phi.inputs() {
                [PhiInput::Value(left), PhiInput::Value(right)] if left == right => {
                    builder_origin(ssa, *left, producers, depth + 1)
                }
                _ => false,
            }),
        _ => false,
    }
}

/// Every driver BCI one chain states: its head, its `toString` and each `append`.
fn chain_positions(chain: &Chain) -> Vec<u32> {
    let mut positions = vec![chain.head, chain.tail];
    positions.extend(chain.appends.iter().map(|(bci, _)| *bci));
    positions
}

/// One record of a presented chain.
fn record_of(chain: &Chain) -> ConcatRecord {
    ConcatRecord {
        head: chain.head,
        tail: Some(chain.tail),
        class: chain.class.clone(),
        appends: chain
            .appends
            .iter()
            .map(|(bci, parameter)| ConcatAppend {
                bci: *bci,
                parameter: parameter.spell().to_string(),
            })
            .collect(),
        presented: true,
        refusal: None,
    }
}

/// One record of a refused candidate.
///
/// The construction is counted **here**, in the function that builds the record, rather than at the
/// call site: a record built anywhere and then dropped is a record that was built, and the port
/// (`crate::demand_counts`) exists to say so. The counter of the record itself is charged by
/// [`Decision::record`].
fn refused_record(refused: &Refused) -> ConcatRecord {
    ConcatRecord {
        head: refused.head,
        tail: None,
        class: refused.class.clone(),
        appends: Vec::new(),
        presented: false,
        refusal: Some(refused.record()),
    }
}

/// Verifies one candidate chain, or states the link that failed.
#[allow(clippy::too_many_arguments)]
fn verify(
    head: u32,
    ty: &str,
    head_block: usize,
    block_of: &BTreeMap<u32, usize>,
    all: &[&SsaInstruction],
    block: &[&SsaInstruction],
    ssa: &SsaTable,
    operations: &Operations,
) -> Result<Chain, Refusal> {
    let index = block
        .iter()
        .position(|instruction| instruction.bci() == head)
        .expect("the candidate's block holds the candidate");
    let shape = |detail: String| Refusal::shape("jre_concat_shape", detail);
    // A chain is one contiguous run of one block. A `toString` on the same class in *another* block
    // can be the shape a branch inside the concatenation leaves behind — but only when that
    // `toString` is one **this chain's own builder reaches**. A method builds several chains of one
    // class, and the first `toString` the block order lists belongs to whichever of them its own
    // receiver names (`FinallyOnce.main` reads its first chain's `toString` in the entry block where
    // its third chain is built), so a candidate is attributed through value flow, not through
    // position: its receiver must trace back — through this class's `append` returns, the local
    // slot a source-written builder travels in and the merge a branch leaves on it, within a
    // fixed bound — to the allocation this chain starts at. Only such a consumer in another block
    // refuses this chain, and the refusal names the consumer's own BCI. A `toString` that is not
    // this chain's consumer says nothing about it, and the walk below states its own reason.
    //
    // The allocation's own instructions are read here the way the walk reads them — the
    // allocation, its copy and the constructor that initialized them — without the walk's shape
    // refusals: a head whose next instructions are not that triple is no chain of this rule, and
    // that is the walk's own error to state.
    let built: Option<Vec<u32>> = (|| {
        let copy = block.get(index + 1)?;
        if operations.get(copy.bci()) != Some(&Operation::Duplicate) {
            return None;
        }
        let init = block.get(index + 2)?;
        matches!(
            operations.get(init.bci()),
            Some(Operation::Invoke(target)) if target.owner() == ty && target.name() == "<init>"
        )
        .then_some(vec![head, copy.bci(), init.bci()])
    })();
    if let Some(tail) = built.and_then(|built| {
        all.iter().find(|instruction| {
            instruction.bci() > head
                && matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::Invoke(target))
                        if target.owner() == ty
                            && target.name() == "toString"
                            && target.descriptor() == "()Ljava/lang/String;"
                )
                && block_of.get(&instruction.bci()).copied() != Some(head_block)
                && consumes_the_instance(ssa, operations, all, ty, &built, instruction)
        })
    }) {
        return Err(Refusal::shape(
            "jre_concat_split",
            format!(
                "the concatenation that starts at BCI {head} ends in the `toString` at BCI {}, which is in another block: a branch cuts the chain in two, and the value it builds is not the value of one expression",
                tail.bci()
            ),
        ));
    }
    let Some(dup) = block.get(index + 1).copied() else {
        return Err(shape(format!(
            "the allocation at BCI {head} ends its block: a concatenation continues with the `dup` of the instance"
        )));
    };
    if operations.get(dup.bci()) != Some(&Operation::Duplicate) {
        return Err(shape(format!(
            "the instruction after the allocation at BCI {head} is at BCI {}, and a chain continues with the `dup` of the instance it allocated",
            dup.bci()
        )));
    }
    let Some(init) = block.get(index + 2).copied() else {
        return Err(shape(format!(
            "the `dup` at BCI {} ends its block: a concatenation continues with the constructor of the class it allocated",
            dup.bci()
        )));
    };
    let constructor = match operations.get(init.bci()) {
        Some(Operation::Invoke(target))
            if target.owner() == ty
                && target.name() == "<init>"
                && target.descriptor() == "()V" =>
        {
            target
        }
        _ => {
            return Err(shape(format!(
                "the instruction after the `dup` at BCI {} is not `{ty}.<init>()V`: the instance a chain builds is constructed by its own class",
                dup.bci()
            )));
        }
    };
    // The instructions that produced the instance the chain builds: the allocation, its copy and the
    // constructor that initialized it. Which of the three a later instruction's receiver traces back
    // to is a detail of the way this build names stack values, and it is not what this check is
    // about: what matters is that the receiver **came from this chain's own allocation** and not
    // from anywhere else on the operand stack. Every instruction that reads such a value has to be
    // part of the chain — one outside it is an *alias*, and presenting the chain would hide the use
    // that alias makes of the instance.
    let mut produced_by: Vec<u32> = vec![head, dup.bci(), init.bci()];
    let operands = stack_operands(init);
    let Some((_, receiver)) = operands.first().copied() else {
        return Err(shape(format!(
            "the constructor call at BCI {} reads no receiver this run states",
            init.bci()
        )));
    };
    if !is_the_instance(ssa, receiver, &produced_by) {
        return Err(shape(format!(
            "the constructor at BCI {} is called on a value this chain's allocation did not produce",
            init.bci()
        )));
    }
    let mut owned: BTreeSet<u32> = BTreeSet::from([head, dup.bci(), init.bci()]);
    let mut appends: Vec<(u32, Type)> = Vec::new();
    let mut previous = init.bci();
    let mut tail: Option<u32> = None;
    for instruction in &block[index + 3..] {
        let at = instruction.bci();
        match operations.get(at) {
            Some(Operation::Invoke(target))
                if target.owner() == ty && target.name() == "append" =>
            {
                let Some((params, returns)) = parse_method(target.descriptor()) else {
                    return Err(shape(format!(
                        "the `append` at BCI {at} has the descriptor `{}`, which is not one this layer reads",
                        target.descriptor()
                    )));
                };
                if params.len() != 1 {
                    return Err(shape(format!(
                        "the `append` at BCI {at} takes {} parameter(s); the overload a concatenation calls takes exactly the one value it appends",
                        params.len()
                    )));
                }
                if !keeps_its_conversion(&params[0]) {
                    return Err(shape(format!(
                        "the `append` at BCI {at} takes `{}`, and writing `+` for it would not write what this overload writes: the conversion of an overload this rule does not know is never lost",
                        params[0].spell()
                    )));
                }
                if !matches!(returns, Some(Type::Reference(name)) if name == source_name(ty)) {
                    return Err(shape(format!(
                        "the `append` at BCI {at} does not return `{ty}`, so it is not the chain-building overload"
                    )));
                }
                let operands = stack_operands(instruction);
                let Some((_, receiver)) = operands.first().copied() else {
                    return Err(shape(format!(
                        "the `append` at BCI {at} reads no receiver this run states"
                    )));
                };
                if !is_the_instance(ssa, receiver, &produced_by) {
                    return Err(shape(format!(
                        "the `append` at BCI {at} is called on a value this chain's allocation did not produce: the chain's receiver is not one `new`"
                    )));
                }
                let Some((_, value)) = operands.last().copied() else {
                    return Err(shape(format!(
                        "the `append` at BCI {at} appends no value this run states"
                    )));
                };
                // The order: the value was produced after the previous chain instruction and before
                // this `append`. That is what makes the written expression's left-to-right
                // evaluation the bytecode's own.
                let Some(produced) = produced_at(ssa, value) else {
                    return Err(shape(format!(
                        "the value the `append` at BCI {at} appends was not produced by an instruction of this body, so where it is evaluated is not stated"
                    )));
                };
                if produced <= previous || produced >= at {
                    return Err(shape(format!(
                        "the value the `append` at BCI {at} appends was produced at BCI {produced}, which is not between the chain's previous instruction (BCI {previous}) and the `append`: writing it as an operand of the concatenation would evaluate it in a different order"
                    )));
                }
                appends.push((at, params[0].clone()));
                owned.insert(at);
                // The receiver of every later `append` is what *this* one returned: a chain's
                // receiver is one instance seen through each call it has already made.
                produced_by.push(at);
                previous = at;
            }
            Some(Operation::Invoke(target))
                if target.owner() == ty && target.name() == "toString" =>
            {
                if target.descriptor() != "()Ljava/lang/String;" {
                    return Err(shape(format!(
                        "the `toString` at BCI {at} has the descriptor `{}`, and a concatenation ends in `()Ljava/lang/String;`",
                        target.descriptor()
                    )));
                }
                let operands = stack_operands(instruction);
                let Some((_, receiver)) = operands.first().copied() else {
                    return Err(shape(format!(
                        "the `toString` at BCI {at} reads no receiver this run states"
                    )));
                };
                if !is_the_instance(ssa, receiver, &produced_by) {
                    return Err(shape(format!(
                        "the `toString` at BCI {at} is called on a value this chain's allocation did not produce"
                    )));
                }
                owned.insert(at);
                tail = Some(at);
                break;
            }
            // Everything else inside the span is an *operand's* own instruction, and it may be
            // written as part of the expression only when it is a value expression this layer
            // renders. Anything that produces a statement — a store, a void call, an increment, a
            // field access, an operation this subset does not model — ends the walk: writing the
            // concatenation would have to move that effect.
            Some(
                Operation::Push(_)
                | Operation::Load { .. }
                | Operation::Arithmetic { .. }
                | Operation::Negate,
            ) => {
                owned.insert(at);
            }
            Some(Operation::Invoke(_)) if produces_a_read_value(instruction, block, at) => {
                owned.insert(at);
            }
            Some(operation) => {
                return Err(Refusal::unmet(
                    &CONCAT,
                    Precondition::StatementFree,
                    format!(
                        "the instruction at BCI {at} is an {operation:?} between the chain's own instructions, and presenting the concatenation would write the `append` calls around it: the effect at BCI {at} would run in a different order or a different number of times"
                    ),
                ));
            }
            None => {
                return Err(shape(format!(
                    "the instruction at BCI {at} was not decoded by this run, so what it does between the chain's own instructions is not stated"
                )));
            }
        }
    }
    let Some(tail) = tail else {
        return Err(shape(format!(
            "the chain that starts at BCI {head} reaches the end of its block without a `{ty}.toString()`: what the allocation builds is never read as a `String`"
        )));
    };
    if appends.len() < 2 {
        return Err(shape(format!(
            "the chain that starts at BCI {head} appends {} value(s), and a concatenation this rule writes has at least two operands: one `append` is not a `+`, and the value it built is whatever that one value wrote",
            appends.len()
        )));
    }
    // The result is written where it is consumed: a chain whose `String` nothing reads is an
    // invocation the bytecode made with no place in the body.
    let produced = written_value(ssa, tail);
    let consumed = produced.is_some_and(|value| {
        all.iter().any(|instruction| {
            instruction.bci() != tail
                && instruction.reads().iter().any(|(_, read)| *read == value)
                && renders_its_reads(operations.get(instruction.bci()))
        })
    });
    if !consumed {
        return Err(shape(format!(
            "nothing in this method reads the `String` the `toString` at BCI {tail} returns, so the chain has no place in the body"
        )));
    }
    // No alias: no instruction outside the chain reads the instance it builds.
    for instruction in all {
        if owned.contains(&instruction.bci()) {
            continue;
        }
        if instruction
            .reads()
            .iter()
            .any(|(_, read)| is_the_instance(ssa, *read, &produced_by))
        {
            return Err(shape(format!(
                "the instruction at BCI {} reads the instance the allocation at BCI {head} built, and it is not part of the chain: the instance is aliased, so the chain is not the only thing being done with it",
                instruction.bci()
            )));
        }
    }
    let class = constructor.owner().to_string();
    Ok(Chain {
        head,
        tail,
        class,
        appends,
        owned,
        cut: None,
    })
}

/// Whether one value is the instance a candidate chain builds: it was produced by the allocation,
/// by the copy of it or by the constructor that initialized it.
fn is_the_instance(ssa: &SsaTable, value: ValueId, produced_by: &[u32]) -> bool {
    match ssa.value(value).def() {
        Definition::Instruction { bci, .. } => produced_by.contains(bci),
        _ => false,
    }
}

/// Whether the `toString` at `tail` consumes the instance the candidate chain builds.
///
/// The check reads the chain's own value flow backwards from the `toString`'s receiver: an `append`
/// of the same class hands the instance on through its return, a source-written builder travels in
/// a local slot (a `Store` of the instance, a `Load` of the slot it was stored to), and a branch
/// leaves the instance merged in a phi — one arm of which reaching the allocation makes the value
/// the branch decides, which is the chain cut all the same. Each of those is one step, and the
/// steps share one bound, so the walk ends at the chain's own allocation, at a definition the flow
/// does not state, or at the bound — and only the first of the three is this chain's consumer. A
/// `toString` whose receiver the bound runs out on is **not** attributed: the refusal stays with
/// what the value flow states, and the walk's own reasons cover everything else.
fn consumes_the_instance(
    ssa: &SsaTable,
    operations: &Operations,
    all: &[&SsaInstruction],
    ty: &str,
    built: &[u32],
    tail: &SsaInstruction,
) -> bool {
    let Some((_, receiver)) = stack_operands(tail).first().copied() else {
        return false;
    };
    reaches_the_allocation(
        ssa,
        operations,
        all,
        ty,
        built,
        receiver,
        RECEIVER_TRACE_BOUND,
    )
}

/// One bounded step of [`consumes_the_instance`]: `steps` is what the whole walk has left.
fn reaches_the_allocation(
    ssa: &SsaTable,
    operations: &Operations,
    all: &[&SsaInstruction],
    ty: &str,
    built: &[u32],
    value: ValueId,
    steps: usize,
) -> bool {
    if steps == 0 {
        return false;
    }
    match ssa.value(value).def() {
        Definition::Instruction { bci, .. } => {
            if built.contains(bci) {
                return true;
            }
            let Some(instruction) = all.iter().find(|instruction| instruction.bci() == *bci) else {
                return false;
            };
            let next = match operations.get(*bci) {
                // The instance as this class's `append` returned it: one call further back.
                Some(Operation::Invoke(target))
                    if target.owner() == ty && target.name() == "append" =>
                {
                    stack_operands(instruction).first().map(|(_, value)| *value)
                }
                // The instance as a `Store` wrote it: the value the store read off the stack.
                Some(Operation::Store { .. }) => {
                    stack_operands(instruction).first().map(|(_, value)| *value)
                }
                // The instance as a `Load` read it: the value the slot held at that point.
                Some(Operation::Load { .. }) => instruction
                    .reads()
                    .iter()
                    .find_map(|(slot, value)| matches!(slot, Slot::Local(_)).then_some(*value)),
                _ => None,
            };
            next.is_some_and(|next| {
                reaches_the_allocation(ssa, operations, all, ty, built, next, steps - 1)
            })
        }
        // The instance as the merge of a branch left it: one arm that reaches the allocation is
        // enough — the `toString` then reads a value the branch decides, which is the cut.
        Definition::Phi { .. } => ssa
            .phis()
            .iter()
            .find(|phi| phi.value() == value)
            .is_some_and(|phi| {
                phi.inputs().iter().any(|input| match input {
                    PhiInput::Value(input) => {
                        reaches_the_allocation(ssa, operations, all, ty, built, *input, steps - 1)
                    }
                    PhiInput::Itself => false,
                })
            }),
        _ => false,
    }
}

/// The value one instruction wrote into a stack slot.
fn written_value(ssa: &SsaTable, bci: u32) -> Option<ValueId> {
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

/// The BCI the instruction that produced one value sits at, when an instruction produced it.
fn produced_at(ssa: &SsaTable, value: ValueId) -> Option<u32> {
    match ssa.value(value).def() {
        Definition::Instruction { bci, .. } => Some(*bci),
        _ => None,
    }
}

/// Whether an invocation inside the span is a value an `append` (or another operand) really reads.
///
/// A call whose result nobody in the span reads is a *statement* — a void call, or a call whose
/// value is used after the chain — and writing it inside the expression would move it. So the
/// value it produced must be read by an instruction of the same span, and it must not write a
/// local slot of its own.
fn produces_a_read_value(instruction: &SsaInstruction, block: &[&SsaInstruction], at: u32) -> bool {
    if instruction
        .writes()
        .iter()
        .any(|(slot, _)| matches!(slot, Slot::Local(_)))
    {
        return false;
    }
    let values: Vec<ValueId> = instruction
        .writes()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
        .map(|(_, value)| *value)
        .collect();
    if values.is_empty() {
        return false;
    }
    block.iter().any(|other| {
        other.bci() > at && other.reads().iter().any(|(_, read)| values.contains(read))
    })
}

/// Whether an instruction's operation writes the values it reads into its text, so that dropping it
/// would drop an evaluation.
fn renders_its_reads(operation: Option<&Operation>) -> bool {
    matches!(
        operation,
        Some(
            Operation::Store { .. }
                | Operation::Invoke(_)
                | Operation::InvokeDynamic(_)
                | Operation::Return
                | Operation::Comparison { .. }
                | Operation::Switch { .. }
                | Operation::Arithmetic { .. }
                | Operation::Negate
                | Operation::Field { .. }
        )
    )
}

/// Whether `+` on this parameter type writes the same text as the `append` overload that takes it.
///
/// The int-shaped family, the floating-point family and `boolean` are the same text either way: the
/// overloads are the ones `String.valueOf` converts with. `String` and `Object` are the convention
/// Java's `+` uses for references (`+` on a reference converts it with `String.valueOf`), so
/// `append(Object)` is written as the same text. `char` is the same text **in a string context**:
/// `"" + c` concatenates the one character `append(C)` writes, and the first `+` of every chain this
/// rule presents stands in that context whenever its first part is not a `String` (`crate::emit`'s
/// `Concat` arm), so the code unit is never added as a number. **Everything else is refused** — most
/// importantly `char[]`, which `+` would write as the array's own `toString`, and `CharSequence`,
/// which `append` writes character by character where `+` converts with `toString`.
fn keeps_its_conversion(parameter: &Type) -> bool {
    match parameter {
        Type::Int | Type::Long | Type::Float | Type::Double | Type::Boolean | Type::Char => true,
        Type::Reference(name) => name == "java.lang.String" || name == "java.lang.Object",
        // `byte`, `short` and every other reference: an overload this rule cannot prove writes the
        // text `+` would write.
        _ => false,
    }
}

/// The Java source spelling of an internal name.
fn source_name(internal: &str) -> String {
    internal.replace('/', ".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pass::IrTable;

    #[test]
    fn the_two_concatenation_classes_are_the_only_ones_this_rule_reads() {
        assert_eq!(
            CONCAT_CLASSES,
            ["java/lang/StringBuilder", "java/lang/StringBuffer"]
        );
        assert!(CONCAT.requires(Precondition::StatementFree));
        assert!(CONCAT.requires(Precondition::IrTable(IrTable::Ssa)));
        assert_eq!(
            CONCAT.required_release(),
            None,
            "`+` is Java in every release"
        );
    }

    #[test]
    fn an_append_whose_conversion_plus_would_not_reproduce_is_refused() {
        // The overloads whose text under `+` is the same text.
        for accepted in [
            Type::Int,
            Type::Long,
            Type::Float,
            Type::Double,
            Type::Boolean,
            // `"" + c` concatenates the one character `append(C)` writes: the chain's own text
            // starts in a string context, so the code unit is never added as a number.
            Type::Char,
            Type::Reference("java.lang.String".to_string()),
            Type::Reference("java.lang.Object".to_string()),
        ] {
            assert!(
                keeps_its_conversion(&accepted),
                "{accepted:?} is written the same way by `+`"
            );
        }
        // The ones that are not: `char[]` would be written as an array, a `CharSequence` would be
        // converted through `toString`, `byte` and `short` are the same slot shape but not the same
        // conversion, and a `StringBuffer`/`StringBuilder` (the Java 5 overload) writes characters
        // where `+` converts.
        for refused in [
            Type::Byte,
            Type::Short,
            Type::Reference("char[]".to_string()),
            Type::Reference("java.lang.CharSequence".to_string()),
            Type::Reference("java.lang.StringBuffer".to_string()),
            Type::Reference("java.lang.StringBuilder".to_string()),
        ] {
            assert!(
                !keeps_its_conversion(&refused),
                "{refused:?} is not an overload `+` reproduces"
            );
        }
    }
}
