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
use crate::build::dup_x1_operands;
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
    /// Object-overload append positions on a chain whose verified field read requires the source
    /// conversion to happen before the next operand is evaluated. These positions are rendered as
    /// an explicit `String.valueOf((Object) value)` call at the append's original position.
    pub(crate) object_value_of_appends: BTreeSet<u32>,
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
pub(crate) fn plan(
    ssa: &SsaTable,
    operations: &Operations,
    copies: &crate::build::FieldCopies,
) -> Plan {
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
                head, &ty, head_block, &block_of, &all, block, ssa, operations, copies, None, None,
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

/// Completes only the ordinary concat refusals whose interleaved operation is a directly consumed,
/// precisely claimed instance field read. Every additional value-use and physical-handler scan is
/// charged here; the original plan remains intact until the full certificate and chain verify.
pub(crate) fn certify_ordered_field_operands(
    mut plan: Plan,
    ssa: &SsaTable,
    operations: &Operations,
    copies: &crate::build::FieldCopies,
    fields: &crate::field::Plan,
    code: &jarde_reader::classfile::MethodCodeFacts,
    budget: &mut Budget,
) -> Result<Plan, StopReason> {
    let mut index = 0;
    while index < plan.refused.len() {
        if plan.refused[index].refusal.code() != "jre_concat_interleaved_effect" {
            index += 1;
            continue;
        }
        let head = plan.refused[index].head;
        let class = plan.refused[index].class.clone();
        let Some(field_reads) =
            certify_direct_field_reads(head, &class, ssa, operations, fields, budget)?
        else {
            index += 1;
            continue;
        };
        // The verifier replay is instrumented at each actual instruction, read, copy and Phi it
        // visits. A stop from that replay propagates before either owner set changes.
        let mut blocks = Vec::with_capacity(ssa.blocks().len());
        for block in ssa.blocks() {
            let mut instructions = Vec::with_capacity(block.instructions().len());
            for instruction in block.instructions() {
                charge(
                    budget,
                    CountedBudgetDimension::IrItems,
                    1,
                    Some(instruction.bci()),
                )?;
                poll(budget, Some(instruction.bci()))?;
                instructions.push(instruction);
            }
            blocks.push(instructions);
        }
        let mut all = Vec::new();
        for instruction in blocks.iter().flatten() {
            charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(instruction.bci()),
            )?;
            poll(budget, Some(instruction.bci()))?;
            all.push(*instruction);
        }
        let mut block_of = BTreeMap::new();
        for (block_index, block) in blocks.iter().enumerate() {
            for instruction in block {
                charge(
                    budget,
                    CountedBudgetDimension::IrItems,
                    1,
                    Some(instruction.bci()),
                )?;
                poll(budget, Some(instruction.bci()))?;
                block_of.insert(instruction.bci(), block_index);
            }
        }
        let mut found_head = None;
        for (block_index, block) in blocks.iter().enumerate() {
            for instruction in block {
                charge(
                    budget,
                    CountedBudgetDimension::IrItems,
                    1,
                    Some(instruction.bci()),
                )?;
                poll(budget, Some(instruction.bci()))?;
                if instruction.bci() == head {
                    found_head = Some((block_index, block.as_slice()));
                    break;
                }
            }
            if found_head.is_some() {
                break;
            }
        }
        let Some((head_block, block)) = found_head else {
            index += 1;
            continue;
        };
        let mut replay_meter = VerifyMeter::new(budget);
        let verified = verify(
            head,
            &class,
            head_block,
            &block_of,
            &all,
            block,
            ssa,
            operations,
            copies,
            Some(&field_reads),
            Some(&mut replay_meter),
        );
        if let Some(stop) = replay_meter.stop.take() {
            return Err(stop);
        }
        poll(budget, Some(head))?;
        let Ok(chain) = verified else {
            index += 1;
            continue;
        };
        let mut all_field_reads_owned = true;
        for at in &field_reads {
            charge(budget, CountedBudgetDimension::IrItems, 1, Some(*at))?;
            poll(budget, Some(*at))?;
            all_field_reads_owned &= chain.owned.contains(at);
        }
        if !all_field_reads_owned
            || !validate_field_chain_tail_and_handlers(
                &chain, block, ssa, operations, code, budget,
            )?
        {
            index += 1;
            continue;
        }
        let mut overlaps_existing = false;
        for bci in &chain.owned {
            charge(budget, CountedBudgetDimension::IrItems, 1, Some(*bci))?;
            poll(budget, Some(*bci))?;
            overlaps_existing |= plan.owned.contains(bci);
        }
        if overlaps_existing {
            index += 1;
            continue;
        }
        for bci in &chain.owned {
            charge(budget, CountedBudgetDimension::IrItems, 1, Some(*bci))?;
            poll(budget, Some(*bci))?;
        }
        plan.refused.remove(index);
        plan.owned.extend(&chain.owned);
        plan.chains.insert(chain.tail, chain);
    }
    Ok(plan)
}

/// The scan that upgrades a refusal's specific field reads to candidates. `None` means the shape
/// was not proven; budget and cancellation remain errors and are never collapsed into that answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValueUseSite {
    Instruction(u32),
    Phi,
}

fn certify_direct_field_reads(
    head: u32,
    ty: &str,
    ssa: &SsaTable,
    operations: &Operations,
    fields: &crate::field::Plan,
    budget: &mut Budget,
) -> Result<Option<BTreeSet<u32>>, StopReason> {
    let mut candidate = None;
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(instruction.bci()),
            )?;
            poll(budget, Some(instruction.bci()))?;
            if instruction.bci() == head {
                candidate = Some(block);
                break;
            }
        }
        if candidate.is_some() {
            break;
        }
    }
    let Some(block) = candidate else {
        return Ok(None);
    };
    let instructions = block.instructions();
    let mut start = None;
    for (position, instruction) in instructions.iter().enumerate() {
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        poll(budget, Some(instruction.bci()))?;
        if instruction.bci() == head {
            start = Some(position);
            break;
        }
    }
    let Some(start) = start else {
        return Ok(None);
    };
    let mut uses: BTreeMap<ValueId, Vec<ValueUseSite>> = BTreeMap::new();
    let mut stack_writes = BTreeMap::new();
    let mut loads = BTreeMap::new();
    for instruction in ssa.blocks().iter().flat_map(|block| block.instructions()) {
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        poll(budget, Some(instruction.bci()))?;
        let mut stack_write = None;
        let mut stack_write_count = 0usize;
        for (slot, value) in instruction.writes() {
            charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(instruction.bci()),
            )?;
            poll(budget, Some(instruction.bci()))?;
            if matches!(slot, Slot::Stack(_)) {
                stack_write_count += 1;
                stack_write = Some(*value);
                stack_writes.insert(instruction.bci(), *value);
            }
        }
        let mut local_read = None;
        let mut local_read_count = 0usize;
        for (slot, value) in instruction.reads() {
            charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(instruction.bci()),
            )?;
            poll(budget, Some(instruction.bci()))?;
            uses.entry(*value)
                .or_default()
                .push(ValueUseSite::Instruction(instruction.bci()));
            if matches!(slot, Slot::Local(_)) {
                local_read_count += 1;
                local_read = Some(*value);
            }
        }
        if matches!(
            operations.get(instruction.bci()),
            Some(Operation::Load { .. })
        ) {
            if local_read_count == 1 && stack_write_count == 1 {
                if let (Some(local), Some(output)) = (local_read, stack_write) {
                    loads.insert(instruction.bci(), (local, output));
                }
            }
        }
    }
    for phi in ssa.phis() {
        charge(budget, CountedBudgetDimension::IrItems, 1, Some(head))?;
        poll(budget, Some(head))?;
        for input in phi.inputs() {
            charge(budget, CountedBudgetDimension::IrItems, 1, Some(head))?;
            poll(budget, Some(head))?;
            if let PhiInput::Value(value) = input {
                uses.entry(*value).or_default().push(ValueUseSite::Phi);
            }
        }
    }
    let mut field_reads = BTreeSet::new();
    let mut saw_tail = false;
    for (position, instruction) in instructions.iter().enumerate().skip(start + 3) {
        let at = instruction.bci();
        charge(budget, CountedBudgetDimension::IrItems, 1, Some(at))?;
        poll(budget, Some(at))?;
        match operations.get(at) {
            Some(Operation::Invoke(target))
                if target.owner() == ty && target.name() == "toString" =>
            {
                saw_tail = true;
                break;
            }
            Some(Operation::Field {
                access: FieldAccess::Read,
                is_static,
                owner,
                name,
                descriptor,
            }) => {
                if *is_static {
                    return Ok(None);
                }
                let Some((evidence, shape)) = fields.claim(at) else {
                    return Ok(None);
                };
                let operands = stack_operands_counted(instruction, budget)?;
                if operands.len() != 1 {
                    return Ok(None);
                }
                let Some((_, receiver)) = operands.first().copied() else {
                    return Ok(None);
                };
                let Some(result) = stack_writes.get(&at).copied() else {
                    return Ok(None);
                };
                if evidence.access != FieldAccess::Read
                    || evidence.is_static
                    || evidence.owner != *owner
                    || evidence.name != *name
                    || evidence.descriptor != *descriptor
                    || shape.writes()
                    || shape.receiver != Some(receiver)
                {
                    return Ok(None);
                }
                let Some(previous) = position.checked_sub(1).and_then(|i| instructions.get(i))
                else {
                    return Ok(None);
                };
                if !matches!(operations.get(previous.bci()), Some(Operation::Load { .. }))
                    || !matches!(previous.opcode(), 0x19 | 0x2a..=0x2d)
                    || !loads
                        .get(&previous.bci())
                        .is_some_and(|(_, output)| *output == receiver)
                    || !uses
                        .get(&receiver)
                        .is_some_and(|seen| seen.as_slice() == [ValueUseSite::Instruction(at)])
                {
                    return Ok(None);
                }
                let Some(next) = instructions.get(position + 1) else {
                    return Ok(None);
                };
                let Some(Operation::Invoke(target)) = operations.get(next.bci()) else {
                    return Ok(None);
                };
                if target.owner() != ty || target.name() != "append" {
                    return Ok(None);
                }
                let next_operands = stack_operands_counted(next, budget)?;
                let Some((_, appended)) = next_operands.last().copied() else {
                    return Ok(None);
                };
                if appended != result
                    || !uses.get(&result).is_some_and(|seen| {
                        seen.as_slice() == [ValueUseSite::Instruction(next.bci())]
                    })
                {
                    return Ok(None);
                }
                field_reads.insert(at);
            }
            _ => {}
        }
    }
    if !saw_tail || field_reads.is_empty() {
        return Ok(None);
    }
    Ok(Some(field_reads))
}

struct VerifyMeter<'a> {
    budget: &'a mut Budget,
    stop: Option<StopReason>,
}

impl<'a> VerifyMeter<'a> {
    fn new(budget: &'a mut Budget) -> Self {
        Self { budget, stop: None }
    }

    fn step(&mut self, at: Option<u32>) -> bool {
        if self.stop.is_some() {
            return false;
        }
        self.stop = charge(self.budget, CountedBudgetDimension::IrItems, 1, at)
            .and_then(|()| poll(self.budget, at))
            .err();
        self.stop.is_none()
    }
}

fn validate_field_chain_tail_and_handlers(
    chain: &Chain,
    block: &[&SsaInstruction],
    ssa: &SsaTable,
    operations: &Operations,
    code: &jarde_reader::classfile::MethodCodeFacts,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let mut tail_index = None;
    for (index, instruction) in block.iter().enumerate() {
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        poll(budget, Some(instruction.bci()))?;
        if instruction.bci() == chain.tail {
            tail_index = Some(index);
            break;
        }
    }
    let Some(tail_index) = tail_index else {
        return Ok(false);
    };
    let Some(consumer) = block.get(tail_index + 1) else {
        return Ok(false);
    };
    let Some(string_value) = written_value_counted(ssa, chain.tail, budget)? else {
        return Ok(false);
    };
    let mut consumers = Vec::new();
    for instruction in ssa.blocks().iter().flat_map(|block| block.instructions()) {
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        poll(budget, Some(instruction.bci()))?;
        let mut reads_string = false;
        for (_, read) in instruction.reads() {
            charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(instruction.bci()),
            )?;
            poll(budget, Some(instruction.bci()))?;
            reads_string |= *read == string_value;
        }
        if reads_string {
            consumers.push(instruction.bci());
        }
    }
    let mut phi_reads_string = false;
    for phi in ssa.phis() {
        charge(budget, CountedBudgetDimension::IrItems, 1, Some(chain.tail))?;
        poll(budget, Some(chain.tail))?;
        for input in phi.inputs() {
            charge(budget, CountedBudgetDimension::IrItems, 1, Some(chain.tail))?;
            poll(budget, Some(chain.tail))?;
            phi_reads_string |= matches!(input, PhiInput::Value(value) if *value == string_value);
        }
    }
    if phi_reads_string
        || consumers.as_slice() != [consumer.bci()]
        || !renders_its_reads(operations.get(consumer.bci()))
    {
        return Ok(false);
    }
    let mut head_index = None;
    for (index, instruction) in block.iter().enumerate() {
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        poll(budget, Some(instruction.bci()))?;
        if instruction.bci() == chain.head {
            head_index = Some(index);
            break;
        }
    }
    let Some(head_index) = head_index else {
        return Ok(false);
    };
    if head_index > tail_index {
        return Ok(false);
    }
    let mut expected_rows: Option<Vec<jarde_reader::classfile::ExceptionHandlerFact>> = None;
    for instruction in &block[head_index..=tail_index + 1] {
        let at = instruction.bci();
        charge(budget, CountedBudgetDimension::IrItems, 1, Some(at))?;
        poll(budget, Some(at))?;
        let mut rows = Vec::new();
        for row in &code.exception_handlers {
            charge(budget, CountedBudgetDimension::IrItems, 1, Some(at))?;
            poll(budget, Some(at))?;
            if row.start_bci <= at && at < row.end_bci {
                rows.push(row.clone());
            }
        }
        if expected_rows
            .as_ref()
            .is_some_and(|expected| expected != &rows)
        {
            return Ok(false);
        }
        expected_rows.get_or_insert(rows);
    }
    Ok(true)
}

/// The one bounded branched form used by Java 8's four conditional String operands.
/// The ordinary same-block rule and its `jre_concat_split` refusal remain unchanged. A
/// successful certificate replaces that refusal only after the entire physical method, CFG,
/// four stack Phis and one unaliased builder have been checked as a unit.
pub(crate) fn plan_four_conditional_strings(
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    operations: &Operations,
    copies: &crate::build::FieldCopies,
    budget: &mut Budget,
) -> Result<Plan, StopReason> {
    let mut plan = plan(ssa, operations, copies);
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
        object_value_of_appends: BTreeSet::new(),
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
    copies: &crate::build::FieldCopies,
    budget: &mut Budget,
) -> Result<Plan, StopReason> {
    let mut plan = plan_four_conditional_strings(ssa, canonical, operations, copies, budget)?;
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

/// Plans every supported concat shape through one entry point, including the narrowly proved
/// instance-field operands. The field certificate consumes the same facts and budget as the
/// ordinary planner; it is a completion of this plan, not a second concat pass in the caller.
pub(crate) fn plan_with_ordered_field_operands(
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    operations: &Operations,
    copies: &crate::build::FieldCopies,
    fields: &crate::field::Plan,
    code: &jarde_reader::classfile::MethodCodeFacts,
    budget: &mut Budget,
) -> Result<Plan, StopReason> {
    let plan = plan_conditional_cut_chains(ssa, canonical, operations, copies, budget)?;
    certify_ordered_field_operands(plan, ssa, operations, copies, fields, code, budget)
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
            Some(Operation::Invoke(_))
                if produces_a_read_value(instruction, &block_refs, at, None) =>
            {
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
            Some(Operation::Invoke(_))
                if produces_a_read_value(instruction, &join_refs, at, None) =>
            {
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
        object_value_of_appends: BTreeSet::new(),
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
    copies: &crate::build::FieldCopies,
    ordered_field_reads: Option<&BTreeSet<u32>>,
    mut meter: Option<&mut VerifyMeter<'_>>,
) -> Result<Chain, Refusal> {
    let mut index = None;
    for (position, instruction) in block.iter().enumerate() {
        if meter
            .as_deref_mut()
            .is_some_and(|meter| !meter.step(Some(instruction.bci())))
        {
            return Err(Refusal::shape(
                "jre_concat_shape",
                "concat verification stopped".to_owned(),
            ));
        }
        if instruction.bci() == head {
            index = Some(position);
            break;
        }
    }
    let index = index.expect("the candidate's block holds the candidate");
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
    let mut split_tail = None;
    if let Some(built) = &built {
        for instruction in all {
            if meter
                .as_deref_mut()
                .is_some_and(|meter| !meter.step(Some(instruction.bci())))
            {
                return Err(shape("concat verification stopped".to_owned()));
            }
            if instruction.bci() > head
                && matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::Invoke(target))
                        if target.owner() == ty
                            && target.name() == "toString"
                            && target.descriptor() == "()Ljava/lang/String;"
                )
                && block_of.get(&instruction.bci()).copied() != Some(head_block)
                && consumes_the_instance(
                    ssa,
                    operations,
                    all,
                    ty,
                    built,
                    instruction,
                    meter.as_deref_mut(),
                )
            {
                split_tail = Some(*instruction);
                break;
            }
            if meter
                .as_deref_mut()
                .is_some_and(|meter| meter.stop.is_some())
            {
                return Err(shape("concat verification stopped".to_owned()));
            }
        }
    }
    if let Some(tail) = split_tail {
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
    let Some(operands) = stack_operands_metered(init, &mut meter) else {
        return Err(shape("concat verification stopped".to_owned()));
    };
    let Some((_, receiver)) = operands.first().copied() else {
        return Err(shape(format!(
            "the constructor call at BCI {} reads no receiver this run states",
            init.bci()
        )));
    };
    if !is_the_instance_metered(ssa, receiver, &produced_by, &mut meter, head) {
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
        if meter
            .as_deref_mut()
            .is_some_and(|meter| !meter.step(Some(instruction.bci())))
        {
            return Err(shape("concat verification stopped".to_owned()));
        }
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
                let Some(operands) = stack_operands_metered(instruction, &mut meter) else {
                    return Err(shape("concat verification stopped".to_owned()));
                };
                let Some((_, receiver)) = operands.first().copied() else {
                    return Err(shape(format!(
                        "the `append` at BCI {at} reads no receiver this run states"
                    )));
                };
                if !is_the_instance_metered(ssa, receiver, &produced_by, &mut meter, at) {
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
                let Some(operands) = stack_operands_metered(instruction, &mut meter) else {
                    return Err(shape("concat verification stopped".to_owned()));
                };
                let Some((_, receiver)) = operands.first().copied() else {
                    return Err(shape(format!(
                        "the `toString` at BCI {at} reads no receiver this run states"
                    )));
                };
                if !is_the_instance_metered(ssa, receiver, &produced_by, &mut meter, at) {
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
            Some(Operation::Invoke(_))
                if produces_a_read_value(instruction, block, at, meter.as_deref_mut()) =>
            {
                owned.insert(at);
            }
            // The receiver copy of a proved compound assignment whose pass-through is **this
            // chain's instance**: `this.field += "[" + x + "]"` lowers the receiver under the
            // builder the concatenation is built in (`aload_0; dup_x1`), so the copy carries the
            // instance through its own chain and the first `append` is called on the value it
            // passed through. The copy's consumers are pinned by the field-copy proof — one copy is
            // the read's receiver and the other the write's — and what this walk adds is the
            // instance's own identity: the value below the duplicate is the one this chain
            // allocated, so the `append` that reads the pass-through is a reader of the instance and
            // not of some other value. Its BCI joins the instance's identity set below, which is how
            // the later `append`'s receiver is recognised as this chain's own instance.
            Some(Operation::Other)
                if copies
                    .receiver_copy_at(at)
                    .is_some_and(|copy| copy.pass_through.is_some())
                    && dup_x1_operands(instruction).is_some_and(|(_, below)| {
                        is_the_instance_metered(ssa, below, &produced_by, &mut meter, at)
                    }) =>
            {
                owned.insert(at);
                produced_by.push(at);
            }
            // The field **read** a proved receiver copy is the receiver of: the copy's other value
            // is the read's receiver, and the read's own value is an operand of one of this chain's
            // `append`s — which the checks above state, exactly as they do for a call's. Writing it
            // as part of the concatenation evaluates it once, at its own position in the chain.
            Some(Operation::Field {
                access: FieldAccess::Read,
                ..
            }) if ordered_field_reads.is_some_and(|reads| reads.contains(&at)) => {
                owned.insert(at);
            }
            Some(Operation::Field {
                access: FieldAccess::Read,
                ..
            }) if copies.copy_at(at).is_none()
                && field_read_is_receiver_copy(at, copies, &owned, &mut meter) =>
            {
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
    let produced = written_value_metered(ssa, tail, &mut meter);
    let mut consumed = false;
    if let Some(value) = produced {
        for instruction in all {
            if meter
                .as_deref_mut()
                .is_some_and(|meter| !meter.step(Some(instruction.bci())))
            {
                return Err(shape("concat verification stopped".to_owned()));
            }
            if instruction.bci() == tail {
                continue;
            }
            for (_, read) in instruction.reads() {
                if meter
                    .as_deref_mut()
                    .is_some_and(|meter| !meter.step(Some(instruction.bci())))
                {
                    return Err(shape("concat verification stopped".to_owned()));
                }
                if *read == value && renders_its_reads(operations.get(instruction.bci())) {
                    consumed = true;
                    break;
                }
            }
            if consumed {
                break;
            }
        }
    }
    if !consumed {
        return Err(shape(format!(
            "nothing in this method reads the `String` the `toString` at BCI {tail} returns, so the chain has no place in the body"
        )));
    }
    // No alias: no instruction outside the chain reads the instance it builds — but the **write**
    // of the receiver copy this chain carries is not an alias. The instance's own identity flows
    // through that copy, so the compound assignment's `putfield` reads the instance by the copy's
    // other value; the copy is the chain's own instruction, and the store is the one statement the
    // `+=` this chain is the value of is written as.
    for instruction in all {
        if meter
            .as_deref_mut()
            .is_some_and(|meter| !meter.step(Some(instruction.bci())))
        {
            return Err(shape("concat verification stopped".to_owned()));
        }
        if owned.contains(&instruction.bci()) {
            continue;
        }
        let mut copied_receiver = false;
        for copy in copies.values() {
            if meter
                .as_deref_mut()
                .is_some_and(|meter| !meter.step(Some(instruction.bci())))
            {
                return Err(shape("concat verification stopped".to_owned()));
            }
            if matches!(
                copy.shape,
                crate::build::FieldCopyShape::Receiver { store, .. } if store == instruction.bci()
            ) && owned.contains(&copy.duplicate)
            {
                copied_receiver = true;
                break;
            }
        }
        if copied_receiver {
            continue;
        }
        let mut reads_instance = false;
        for (_, read) in instruction.reads() {
            if meter
                .as_deref_mut()
                .is_some_and(|meter| !meter.step(Some(instruction.bci())))
            {
                return Err(shape("concat verification stopped".to_owned()));
            }
            if is_the_instance_metered(ssa, *read, &produced_by, &mut meter, head) {
                reads_instance = true;
                break;
            }
        }
        if reads_instance {
            return Err(shape(format!(
                "the instruction at BCI {} reads the instance the allocation at BCI {head} built, and it is not part of the chain: the instance is aliased, so the chain is not the only thing being done with it",
                instruction.bci()
            )));
        }
    }
    let class = constructor.owner().to_string();
    let mut object_value_of_appends = BTreeSet::new();
    if ordered_field_reads.is_some_and(|reads| !reads.is_empty()) {
        for (append_bci, parameter) in &appends {
            if meter
                .as_deref_mut()
                .is_some_and(|meter| !meter.step(Some(*append_bci)))
            {
                return Err(shape("concat verification stopped".to_owned()));
            }
            if matches!(parameter, Type::Reference(name) if name == "java.lang.Object") {
                object_value_of_appends.insert(*append_bci);
            }
        }
    }
    Ok(Chain {
        head,
        tail,
        class,
        appends,
        owned,
        object_value_of_appends,
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
    mut meter: Option<&mut VerifyMeter<'_>>,
) -> bool {
    if meter
        .as_deref_mut()
        .is_some_and(|meter| !meter.step(Some(tail.bci())))
    {
        return false;
    }
    let Some(operands) = stack_operands_metered(tail, &mut meter) else {
        return false;
    };
    let Some((_, receiver)) = operands.first().copied() else {
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
        meter.as_deref_mut(),
    )
}

fn is_the_instance_metered(
    ssa: &SsaTable,
    value: ValueId,
    produced_by: &[u32],
    meter: &mut Option<&mut VerifyMeter<'_>>,
    at: u32,
) -> bool {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return false;
    };
    for produced in produced_by {
        if meter
            .as_deref_mut()
            .is_some_and(|meter| !meter.step(Some(at)))
        {
            return false;
        }
        if produced == bci {
            return true;
        }
    }
    false
}

fn stack_operands_metered(
    instruction: &SsaInstruction,
    meter: &mut Option<&mut VerifyMeter<'_>>,
) -> Option<Vec<(Slot, ValueId)>> {
    let mut operands = Vec::new();
    for (slot, value) in instruction.reads() {
        if meter
            .as_deref_mut()
            .is_some_and(|meter| !meter.step(Some(instruction.bci())))
        {
            return None;
        }
        if matches!(slot, Slot::Stack(_)) {
            operands.push((*slot, *value));
        }
    }
    operands.sort_by_key(|(slot, _)| match slot {
        Slot::Stack(depth) => *depth,
        Slot::Local(slot) => u32::from(*slot),
    });
    Some(operands)
}

fn stack_operands_counted(
    instruction: &SsaInstruction,
    budget: &mut Budget,
) -> Result<Vec<(Slot, ValueId)>, StopReason> {
    let mut operands = Vec::new();
    for (slot, value) in instruction.reads() {
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        poll(budget, Some(instruction.bci()))?;
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

fn field_read_is_receiver_copy(
    at: u32,
    copies: &crate::build::FieldCopies,
    owned: &BTreeSet<u32>,
    meter: &mut Option<&mut VerifyMeter<'_>>,
) -> bool {
    for copy in copies.values() {
        if meter
            .as_deref_mut()
            .is_some_and(|meter| !meter.step(Some(at)))
        {
            return false;
        }
        if matches!(
            copy.shape,
            crate::build::FieldCopyShape::Receiver { read, .. } if read == at
        ) {
            return owned.contains(&copy.duplicate);
        }
    }
    false
}

#[cfg(test)]
mod ordered_field_budget_tests {
    use super::*;
    use jarde_jvm::engine::analyze_method_ir;
    use jarde_jvm::environment::ResolutionEnvironment;
    use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
    use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
    use jarde_reader::budget::{Budget, CancellationToken, Limits};
    use jarde_reader::classfile::{ExceptionHandlerFact, MethodCodeFacts};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
        MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
        RuntimeView,
    };

    const FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/proved-ordered-field-concat/javac8-debug/OrderedFieldConcat.class"
    );

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
            nested_depth: 8,
            dependency_depth: 4,
            elapsed_millis: u64::MAX,
        }
    }

    fn direct_candidate() -> (
        jarde_jvm::method_ir::MethodIrAnalysis,
        crate::decode::Operations,
        crate::field::Plan,
        crate::build::FieldCopies,
    ) {
        let mut setup_budget = Budget::new(limits());
        let snapshot =
            ArtifactSnapshot::open(ArtifactInput::bytes(FIXTURE.to_vec()), &mut setup_budget)
                .expect("fixture class opens");
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(FIXTURE).to_hex().to_string()),
                length: u64::try_from(FIXTURE.len()).expect("fixture length fits"),
            },
            variant: PhysicalVariant::Base,
        };
        let method = PhysicalMethodId {
            owner: definition,
            name: JvmBytes(b"instanceFields".to_vec()),
            descriptor: JvmBytes(b"(LOrderedFieldConcat;)Ljava/lang/String;".to_vec()),
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
            &[snapshot],
            &MethodAnalysisRequest {
                environment: ResolutionEnvironment {
                    runtime: RuntimeView {
                        physical: PhysicalView {
                            snapshot: method.owner.location.snapshot().clone(),
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
        .expect("fixture method analyzes");
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("SSA table is present");
        let canonical = ir.canonical().expect("canonical CFG is present");
        let code = ir.code().expect("decoded Code facts are present");
        let operations = crate::decode::Operations::of(code, ir.constant_pool());
        let declaring = crate::facts::DeclaringClass::new("OrderedFieldConcat", 0x0031);
        let element_receiver_type =
            |value| crate::build::element_receiver_type(ssa, &operations, value);
        let fields = crate::field::plan(
            ssa,
            &operations,
            &element_receiver_type,
            Some(&declaring),
            "instanceFields",
            "(LOrderedFieldConcat;)Ljava/lang/String;",
            ir.class_fields(),
            &[],
            &mut setup_budget,
        )
        .expect("field plan completes");
        let copies = crate::build::FieldCopies::prove(
            ssa,
            canonical,
            &operations,
            &fields,
            false,
            &mut setup_budget,
        )
        .expect("field-copy plan completes");
        (analysis, operations, fields, copies)
    }

    #[test]
    fn short_budget_and_precancel_stop_inside_field_certificate_before_owner_commit() {
        let (analysis, operations, fields, copies) = direct_candidate();
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("SSA table is present");
        let canonical = ir.canonical().expect("canonical CFG is present");
        let code = ir.code().expect("decoded Code facts are present");

        let mut full_budget = Budget::new(limits());
        let full = plan_with_ordered_field_operands(
            ssa,
            canonical,
            &operations,
            &copies,
            &fields,
            code,
            &mut full_budget,
        );
        let full = full.expect("the direct candidate reaches a complete certificate");
        assert!(
            !full.chains.is_empty(),
            "the accepted chain owns its append plan"
        );
        let used = full_budget.usage().ir_items;
        let mut short_limits = limits();
        short_limits.ir_items = used.checked_sub(1).expect("certificate charged IR work");
        let mut short_budget = Budget::new(short_limits);
        let short = plan_with_ordered_field_operands(
            ssa,
            canonical,
            &operations,
            &copies,
            &fields,
            code,
            &mut short_budget,
        );
        assert!(matches!(
            short,
            Err(StopReason::Budget {
                dimension: CountedBudgetDimension::IrItems,
                at: Some(_),
                ..
            })
        ));
        assert!(short_budget.usage().ir_items < used);

        let ordinary = plan(ssa, &operations, &copies);
        assert!(
            ordinary
                .refused
                .iter()
                .any(|refused| { refused.refusal.code() == "jre_concat_interleaved_effect" })
        );
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let mut cancelled_budget = Budget::with_cancellation_token(limits(), cancellation);
        let cancelled = certify_ordered_field_operands(
            ordinary,
            ssa,
            &operations,
            &copies,
            &fields,
            code,
            &mut cancelled_budget,
        );
        assert!(matches!(
            cancelled,
            Err(StopReason::Cancelled { at: Some(_) })
        ));
    }

    #[test]
    fn a_physical_handler_row_split_across_the_field_chain_keeps_the_refusal() {
        let (analysis, operations, fields, copies) = direct_candidate();
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("SSA table is present");
        let canonical = ir.canonical().expect("canonical CFG is present");
        let code = ir.code().expect("decoded Code facts are present");
        let field_bci = code
            .instructions
            .iter()
            .find_map(|instruction| match operations.get(instruction.bci) {
                Some(Operation::Field {
                    access: FieldAccess::Read,
                    is_static: false,
                    ..
                }) => Some(instruction.bci),
                _ => None,
            })
            .expect("fixture method has an instance field read");
        let tail = code
            .instructions
            .iter()
            .find(|instruction| {
                matches!(
                    operations.get(instruction.bci),
                    Some(Operation::Invoke(target)) if target.name() == "toString"
                )
            })
            .expect("fixture method has a String tail");
        let tail_index = code
            .instructions
            .iter()
            .position(|instruction| instruction.bci == tail.bci)
            .expect("tail is in the decoded instructions");
        let consumer = &code.instructions[tail_index + 1];
        let end = consumer.bci.saturating_add(consumer.width);
        // These rows are added to the real Code rows used by the planner. Their boundary is at
        // the actual getfield BCI; no claim is made that the modified class bytes would verify.
        let split_code = MethodCodeFacts::from_parts(
            code.max_stack,
            code.max_locals,
            code.code_span.clone(),
            code.instructions
                .iter()
                .cloned()
                .zip(code.operands().iter().cloned())
                .collect(),
            vec![
                ExceptionHandlerFact {
                    ordinal: 0,
                    start_bci: code.instructions[0].bci,
                    end_bci: field_bci,
                    handler_bci: consumer.bci,
                    catch_type_index: None,
                },
                ExceptionHandlerFact {
                    ordinal: 1,
                    start_bci: field_bci,
                    end_bci: end,
                    handler_bci: consumer.bci,
                    catch_type_index: None,
                },
            ],
            2,
            code.execution.clone(),
            code.stopped_at.clone(),
            code.debug().clone(),
        );
        let mut budget = Budget::new(limits());
        let plan = plan_with_ordered_field_operands(
            ssa,
            canonical,
            &operations,
            &copies,
            &fields,
            &split_code,
            &mut budget,
        )
        .expect("coverage rejection is an ordinary shape refusal");
        assert!(plan.chains.is_empty());
        assert!(
            plan.refused
                .iter()
                .any(|refused| { refused.refusal.code() == "jre_concat_interleaved_effect" })
        );
    }

    #[test]
    fn an_unclaimed_field_read_cannot_complete_the_concat_refusal() {
        let (analysis, operations, _claimed_fields, copies) = direct_candidate();
        let ir = analysis.ir();
        let ssa = ir.ssa().expect("SSA table is present");
        let canonical = ir.canonical().expect("canonical CFG is present");
        let code = ir.code().expect("decoded Code facts are present");
        let mut budget = Budget::new(limits());
        let plan = plan_with_ordered_field_operands(
            ssa,
            canonical,
            &operations,
            &copies,
            &crate::field::Plan::empty(),
            code,
            &mut budget,
        )
        .expect("an unclaimed read remains an ordinary refusal");
        assert!(plan.chains.is_empty());
        assert!(
            plan.refused
                .iter()
                .any(|refused| { refused.refusal.code() == "jre_concat_interleaved_effect" })
        );
    }
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
    mut meter: Option<&mut VerifyMeter<'_>>,
) -> bool {
    if steps == 0 {
        return false;
    }
    match ssa.value(value).def() {
        Definition::Instruction { bci, .. } => {
            for produced in built {
                if meter
                    .as_deref_mut()
                    .is_some_and(|meter| !meter.step(Some(*bci)))
                {
                    return false;
                }
                if produced == bci {
                    return true;
                }
            }
            let mut instruction = None;
            for candidate in all {
                if meter
                    .as_deref_mut()
                    .is_some_and(|meter| !meter.step(Some(candidate.bci())))
                {
                    return false;
                }
                if candidate.bci() == *bci {
                    instruction = Some(*candidate);
                    break;
                }
            }
            let Some(instruction) = instruction else {
                return false;
            };
            let next = match operations.get(*bci) {
                // The instance as this class's `append` returned it: one call further back.
                Some(Operation::Invoke(target))
                    if target.owner() == ty && target.name() == "append" =>
                {
                    stack_operands_metered(instruction, &mut meter)
                        .and_then(|operands| operands.first().map(|(_, value)| *value))
                }
                // The instance as a `Store` wrote it: the value the store read off the stack.
                Some(Operation::Store { .. }) => stack_operands_metered(instruction, &mut meter)
                    .and_then(|operands| operands.first().map(|(_, value)| *value)),
                // The instance as a `Load` read it: the value the slot held at that point.
                Some(Operation::Load { .. }) => {
                    let mut local = None;
                    for (slot, value) in instruction.reads() {
                        if meter
                            .as_deref_mut()
                            .is_some_and(|meter| !meter.step(Some(instruction.bci())))
                        {
                            return false;
                        }
                        if matches!(slot, Slot::Local(_)) {
                            local = Some(*value);
                            break;
                        }
                    }
                    local
                }
                _ => None,
            };
            next.is_some_and(|next| {
                reaches_the_allocation(
                    ssa,
                    operations,
                    all,
                    ty,
                    built,
                    next,
                    steps - 1,
                    meter.as_deref_mut(),
                )
            })
        }
        // The instance as the merge of a branch left it: one arm that reaches the allocation is
        // enough — the `toString` then reads a value the branch decides, which is the cut.
        Definition::Phi { .. } => {
            let mut phi = None;
            for candidate in ssa.phis() {
                if meter.as_deref_mut().is_some_and(|meter| !meter.step(None)) {
                    return false;
                }
                if candidate.value() == value {
                    phi = Some(candidate);
                    break;
                }
            }
            let Some(phi) = phi else {
                return false;
            };
            for input in phi.inputs() {
                if meter.as_deref_mut().is_some_and(|meter| !meter.step(None)) {
                    return false;
                }
                if let PhiInput::Value(input) = input {
                    if reaches_the_allocation(
                        ssa,
                        operations,
                        all,
                        ty,
                        built,
                        *input,
                        steps - 1,
                        meter.as_deref_mut(),
                    ) {
                        return true;
                    }
                }
            }
            false
        }
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

fn written_value_counted(
    ssa: &SsaTable,
    bci: u32,
    budget: &mut Budget,
) -> Result<Option<ValueId>, StopReason> {
    for instruction in ssa.blocks().iter().flat_map(|block| block.instructions()) {
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        poll(budget, Some(instruction.bci()))?;
        if instruction.bci() != bci {
            continue;
        }
        for (slot, value) in instruction.writes() {
            charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(instruction.bci()),
            )?;
            poll(budget, Some(instruction.bci()))?;
            if matches!(slot, Slot::Stack(_)) {
                return Ok(Some(*value));
            }
        }
        return Ok(None);
    }
    Ok(None)
}

fn written_value_metered(
    ssa: &SsaTable,
    bci: u32,
    meter: &mut Option<&mut VerifyMeter<'_>>,
) -> Option<ValueId> {
    for instruction in ssa.blocks().iter().flat_map(|block| block.instructions()) {
        if meter
            .as_deref_mut()
            .is_some_and(|meter| !meter.step(Some(instruction.bci())))
        {
            return None;
        }
        if instruction.bci() != bci {
            continue;
        }
        for (slot, value) in instruction.writes() {
            if meter
                .as_deref_mut()
                .is_some_and(|meter| !meter.step(Some(instruction.bci())))
            {
                return None;
            }
            if matches!(slot, Slot::Stack(_)) {
                return Some(*value);
            }
        }
        return None;
    }
    None
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
fn produces_a_read_value(
    instruction: &SsaInstruction,
    block: &[&SsaInstruction],
    at: u32,
    mut meter: Option<&mut VerifyMeter<'_>>,
) -> bool {
    let mut writes_local = false;
    for (slot, _) in instruction.writes() {
        if meter
            .as_deref_mut()
            .is_some_and(|meter| !meter.step(Some(at)))
        {
            return false;
        }
        writes_local |= matches!(slot, Slot::Local(_));
    }
    if writes_local {
        return false;
    }
    let mut values = Vec::new();
    for (slot, value) in instruction.writes() {
        if meter
            .as_deref_mut()
            .is_some_and(|meter| !meter.step(Some(at)))
        {
            return false;
        }
        if matches!(slot, Slot::Stack(_)) {
            values.push(*value);
        }
    }
    if values.is_empty() {
        return false;
    }
    for other in block {
        if meter
            .as_deref_mut()
            .is_some_and(|meter| !meter.step(Some(other.bci())))
        {
            return false;
        }
        if other.bci() <= at {
            continue;
        }
        for (_, read) in other.reads() {
            if meter
                .as_deref_mut()
                .is_some_and(|meter| !meter.step(Some(other.bci())))
            {
                return false;
            }
            for value in &values {
                if meter
                    .as_deref_mut()
                    .is_some_and(|meter| !meter.step(Some(other.bci())))
                {
                    return false;
                }
                if value == read {
                    return true;
                }
            }
        }
    }
    false
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
