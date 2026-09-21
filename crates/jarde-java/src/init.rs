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

use jarde_jvm::method_ir::{Definition, Slot, SsaInstruction, SsaTable, Value, ValueId};
use serde::Serialize;

use crate::ast::ConstructorTarget;
use crate::build::stack_operands;
use crate::decode::Operations;
use crate::evidence::Publication;
use crate::facts::{MethodFacts, Operation};
use crate::pass::{INIT, NEW, Precondition, RuleVersion};
use crate::refusal::{Gap, Refusal};

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
    /// Every BCI the site owns: the allocation, the copy and the constructor call. An owned
    /// instruction produces no statement of its own — its text is the `new` expression, written
    /// where the instance is consumed and nowhere else.
    pub(crate) owned: BTreeSet<u32>,
}

/// Every construction site of one body, the candidates that were not sites, and the gaps stated in
/// every selection.
pub(crate) struct Sites {
    sites: Vec<Site>,
    owned: BTreeSet<u32>,
    /// Why a candidate was not a construction site, in BCI order.
    refusals: Vec<Gap>,
    /// The records, when this run published rule records.
    records: Vec<NewRecord>,
}

impl Sites {
    /// An empty plan: a body that allocates nothing.
    pub(crate) fn empty() -> Self {
        Self {
            sites: Vec::new(),
            owned: BTreeSet::new(),
            refusals: Vec::new(),
            records: Vec::new(),
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

    /// Every candidate the rule refused, in BCI order.
    pub(crate) fn refusals(&self) -> &[Gap] {
        &self.refusals
    }

    /// Every candidate and every refusal, in BCI order — the evidence a report reads back when this
    /// run selected rule records.
    pub(crate) fn records(&self) -> &[NewRecord] {
        &self.records
    }

    /// How many candidates the rule read, and how many of them it presented as sites.
    pub(crate) fn counts(&self) -> (u64, u64) {
        let presented = u64::try_from(self.sites.len()).unwrap_or(u64::MAX);
        let refused = u64::try_from(self.refusals.len()).unwrap_or(u64::MAX);
        (presented + refused, presented)
    }
}

/// Reads every construction site of one body.
///
/// `reserved` is every BCI another rule of this run already owns — in practice the concatenation
/// chains, whose own allocation and constructor call are written inside a `+` expression and must not
/// be written a second time as a `new`. One instruction is never two shapes.
pub(crate) fn sites(
    ssa: &SsaTable,
    operations: &Operations,
    reserved: &BTreeSet<u32>,
    publication: Publication,
) -> Sites {
    let blocks: Vec<&[SsaInstruction]> = ssa
        .blocks()
        .iter()
        .map(|block| block.instructions())
        .collect();
    let all: Vec<&SsaInstruction> = blocks.iter().flat_map(|block| block.iter()).collect();
    let mut plan = Sites::empty();
    for block in &blocks {
        for (index, instruction) in block.iter().enumerate() {
            let head = instruction.bci();
            let Some(Operation::Allocate { ty }) = operations.get(head) else {
                continue;
            };
            if reserved.contains(&head) {
                // Another rule of this run writes this allocation's text: it is not a second shape.
                continue;
            }
            let ty = ty.clone();
            match verify(head, index, block, ty.clone(), ssa, operations, &all) {
                Ok(site) => {
                    if publication.publishes(&site_positions(&site)) {
                        crate::demand_counts::record_built(
                            crate::evidence::RecoveryEvidenceKind::RuleDetails,
                        );
                        plan.records.push(NewRecord::of_site(&site));
                    }
                    for bci in &site.owned {
                        plan.owned.insert(*bci);
                    }
                    plan.sites.push(site);
                }
                Err(refusal) => {
                    // The refusal *shape* is what every selection reports about this candidate;
                    // the record that owns it is built only when this run publishes rule records
                    // and the candidate's own BCI is inside the selected driver range.
                    let refusal = NewRefusal::of(&refusal, head);
                    plan.refusals
                        .push(Gap::at(refusal.code, head, refusal.message.clone()));
                    if publication.publishes(&[head]) {
                        crate::demand_counts::record_built(
                            crate::evidence::RecoveryEvidenceKind::RuleDetails,
                        );
                        plan.records
                            .push(NewRecord::of_candidate(head, &ty, refusal));
                    }
                }
            }
        }
    }
    plan.records.sort_by_key(|record| record.head);
    plan.refusals.sort_by_key(|gap| gap.position());
    plan
}

/// Every driver BCI one construction site states: the allocation, its copy, the constructor and
/// every argument.
fn site_positions(site: &Site) -> Vec<u32> {
    let mut positions = vec![site.head, site.dup, site.constructor];
    positions.extend(site.arguments.iter().copied());
    positions
}

/// Verifies one candidate construction site, or states the link that failed.
fn verify(
    head: u32,
    index: usize,
    block: &[SsaInstruction],
    ty: String,
    ssa: &SsaTable,
    operations: &Operations,
    all: &[&SsaInstruction],
) -> Result<Site, Refusal> {
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
    // Nothing inside the span may be an effect this rule would have to move: every instruction
    // between the copy and the call is an argument's own instruction (a value expression, or a call
    // whose value the constructor reads), and anything else ends the walk with the requirement
    // stated.
    for instruction in block.iter().skip(index + 2) {
        if instruction.bci() >= at {
            break;
        }
        match operations.get(instruction.bci()) {
            Some(Operation::Push(_) | Operation::Load { .. } | Operation::Arithmetic { .. }) => {}
            Some(Operation::Invoke(_)) if produces_a_read_value(instruction, block) => {}
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
    // The instance has to be read by an instruction this build **writes it into**. A construction
    // reads no value of its own, so a value nothing writes has no place in the body: the instructions
    // that do read it are quoted instead, and the allocation, its copy and its constructor call are
    // quoted with them — never written as a `new` expression that no statement holds.
    let readers = outside_readers(ssa, all, &produced_by);
    let written: Vec<u32> = readers
        .iter()
        .copied()
        .filter(|bci| renders_its_reads(operations, *bci))
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
    let owned: BTreeSet<u32> = produced_by.iter().copied().collect();
    Ok(Site {
        head,
        dup: dup.bci(),
        constructor: at,
        class: ty,
        arguments,
        owned,
    })
}

/// Every instruction outside a site that reads one of the values the site produced.
fn outside_readers(ssa: &SsaTable, all: &[&SsaInstruction], produced_by: &[u32]) -> Vec<u32> {
    let mut readers: Vec<u32> = Vec::new();
    for instruction in all {
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
/// writes nothing, and an instruction a rule of this build does not claim is quoted. Field accesses,
/// array reads and casts are deliberately not here: whether *their* rules claim them is decided after
/// this plan, and a site that leaned on one would be leaning on a verdict not yet taken.
fn renders_its_reads(operations: &Operations, bci: u32) -> bool {
    matches!(
        operations.get(bci),
        Some(
            Operation::Store { .. }
                | Operation::Invoke(_)
                | Operation::InvokeDynamic(_)
                | Operation::Return
                | Operation::Comparison { .. }
                | Operation::Switch { .. }
                | Operation::Arithmetic { .. }
        )
    )
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

/// Whether an invocation inside the span is a value the constructor call really reads.
fn produces_a_read_value(instruction: &SsaInstruction, block: &[SsaInstruction]) -> bool {
    let values: Vec<ValueId> = instruction
        .writes()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
        .map(|(_, value)| *value)
        .collect();
    values.is_empty()
        || block.iter().any(|other| {
            other.bci() > instruction.bci()
                && other.reads().iter().any(|(_, read)| values.contains(read))
        })
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
pub(crate) struct Prologues {
    prologue: Option<Prologue>,
    /// The call the rule refused to spell, with the requirement it fell short of: the builder quotes
    /// exactly that instruction rather than writing it as whatever the generic arms would make of a
    /// constructor call on an uninitialized `this` (P3 2.3).
    refused: Option<(u32, Refusal)>,
    /// The refusal as every selection states it — the call the rule would not spell, or the body
    /// that has no prologue to read at all. `None` for a body this rule presented and for a body
    /// that is not an instance initializer.
    refusal: Option<Gap>,
    /// The record, when this run published rule records.
    record: Option<InitRecord>,
}

impl Prologues {
    /// No prologue and nothing to record: the body is not an instance initializer.
    pub(crate) fn none() -> Self {
        Self {
            prologue: None,
            refused: None,
            refusal: None,
            record: None,
        }
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
    pub(crate) fn refusal(&self) -> Option<&Gap> {
        self.refusal.as_ref()
    }

    /// The record of this body's prologue, when this run published rule records and the body is an
    /// instance initializer.
    pub(crate) fn record(&self) -> Option<&InitRecord> {
        self.record.as_ref()
    }
}

/// Reads the prologue of one body.
pub(crate) fn prologue(
    ssa: &SsaTable,
    operations: &Operations,
    method: &MethodFacts,
    publication: Publication,
) -> Prologues {
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
        let shape = InitRefusal::of(&refusal, None);
        return Prologues {
            prologue: None,
            refused: None,
            refusal: Some(Gap::whole(shape.code, shape.message.clone())),
            record: publication.publishes(&[]).then(|| {
                crate::demand_counts::record_built(
                    crate::evidence::RecoveryEvidenceKind::RuleDetails,
                );
                InitRecord::of_refusal_shape(None, None, shape)
            }),
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
        let shape = InitRefusal::of(&refusal, Some(bci));
        return Prologues {
            prologue: None,
            refused: Some((bci, refusal)),
            refusal: Some(Gap::at(shape.code, bci, shape.message.clone())),
            record: publication.publishes(&[bci]).then(|| {
                crate::demand_counts::record_built(
                    crate::evidence::RecoveryEvidenceKind::RuleDetails,
                );
                InitRecord::of_refusal_shape(Some(bci), Some(class), shape)
            }),
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
        refusal: None,
        record: publication.publishes(&[bci]).then(|| {
            crate::demand_counts::record_built(crate::evidence::RecoveryEvidenceKind::RuleDetails);
            InitRecord {
                bci: Some(bci),
                target: Some(target_kind),
                class: Some(class),
                declared: Some(declared),
                presented: true,
                refusal: None,
            }
        }),
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
