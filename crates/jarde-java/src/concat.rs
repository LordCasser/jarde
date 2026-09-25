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

use jarde_jvm::method_ir::{Definition, Slot, SsaInstruction, SsaTable, ValueId};
use serde::Serialize;

use crate::ast::Type;
use crate::build::stack_operands;
use crate::decode::Operations;
use crate::evidence::Publication;
use crate::facts::Operation;
use crate::lambda::parse_method;
use crate::pass::{CONCAT, Precondition, RuleVersion};
use crate::refusal::{Gap, Refusal};

/// The classes whose chain is a string concatenation, in the order the design lists them.
///
/// `StringBuilder` is what javac has built concatenations with from release 5 on; `StringBuffer` is
/// the earlier spelling, and a source that names that type still compiles to a chain on it. Nothing
/// else is accepted: a class that happens to hold `append` and `toString` is not a concatenation.
const CONCAT_CLASSES: [&str; 2] = ["java/lang/StringBuilder", "java/lang/StringBuffer"];

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
    // is the shape a branch inside the concatenation leaves behind, and no `+` expression writes a
    // value that two arms built; the walk below would refuse it anyway, but this states the reason
    // the acceptance asks for.
    if let Some(tail) = all.iter().find(|instruction| {
        matches!(
            operations.get(instruction.bci()),
            Some(Operation::Invoke(target))
                if target.owner() == ty
                    && target.name() == "toString"
                    && target.descriptor() == "()Ljava/lang/String;"
        )
    }) && block_of.get(&tail.bci()).copied() != Some(head_block)
    {
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
