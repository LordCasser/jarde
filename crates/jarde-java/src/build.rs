//! ③→④ The AST builder: turns the region tree plus the IR's value flow plus the decoded facts into
//! [`Stmt`]s, without writing a single byte of text.
//!
//! # The rule that keeps this layer honest
//!
//! A statement is built only where every piece of its text is *evidence*:
//!
//! * the shape comes from the region ([`crate::region`]) — a straight run, an `if`, or a fallback;
//! * the operands come from the IR and the decoded operations: a value whose definition is an
//!   instruction is rendered from that instruction's [`Operation`], a value described by a frame
//!   entry or a phi of a local slot is rendered as that local's name (both arms assigned it, so the
//!   join reads it by name), and anything else — a caught exception reference, a stack value no
//!   instruction produced, an operation this subset does not model, a receiver that cannot be
//!   rendered — makes the statement it belongs to a [`StmtKind::Fallback`] that quotes its bytecode.
//!
//! Every `None`, `Err` and `fallback` below is therefore a *stated* one: the artifact says "this BCI
//! is bytecode this layer cannot present" instead of printing a guess, and the report counts it (a
//! run with any fallback is `Mixed`/`Fallback`, never `Java`/`Structured`).
//!
//! # Which instruction becomes a statement
//!
//! * [`Operation::Store`], [`Operation::Invoke`] and [`Operation::Return`] are statements: they are
//!   what the bytecode does to the program's state.
//! * [`Operation::Push`], [`Operation::Load`] and [`Operation::Arithmetic`] are not: they produce a
//!   value whose text lands where the value is consumed, and a load whose value nobody consumes has
//!   no Java effect either.
//! * [`Operation::Comparison`] is not: the branch that tests it prints it — as an `if`'s condition
//!   (the *negation* of the jump sense, because a branch transfers only when its sense holds), or as
//!   part of the quoted bytecode when the region could not be structured.
//! * [`Operation::Other`], and an instruction this run decoded no operation for, are *stated* as
//!   unusable: neither is dropped silently, because a silently dropped instruction is exactly the
//!   failure a presentation must not have.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalCfg, Definition, RefType, Slot, SsaInstruction, SsaTable, Value,
    ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{BootstrapMethodFacts, CpEntryFacts};

use crate::accessor::{self, AccessorRecord, AccessorShape};
use crate::ast::{
    BinaryOp, ConcatPart, ConstructorTarget, Expr, ExprKind, LambdaParam, ResourceDecl, Stmt,
    StmtKind, SwitchArm, Type,
};
use crate::bridge;
use crate::concat;
use crate::decode::Operations;
use crate::enumswitch;
use crate::facts::{
    ArithmeticOp, CallTarget, ClassMembers, CompareOp, ConstantValue, DynamicSite, InvokeKind,
    Operation,
};
use crate::field;
use crate::guard;
use crate::init;
use crate::lambda::{self, LambdaCapture, LambdaForm, LambdaRecord, LambdaRefusal, Reach, Refusal};
use crate::names::{LocalVariable, NameTable, RenderedName};
use crate::pass::{LAMBDA, Precondition, RecoveryProfile};
use crate::region::{Continuation, LoopForm, Region};
use crate::reuse;
use crate::source_map::{Origin, OriginSet};
use crate::stop::{StopReason, charge, poll};

/// How deep a value expression may nest before the builder refuses it.
///
/// Bytecode nests as deeply as the source expression did, and a source expression is bounded by the
/// source; the bound exists so that a pathological body cannot make this layer recurse without
/// limit. Twenty-four levels is far above what the corpus holds and far below a stack this process
/// cannot afford.
const MAX_VALUE_DEPTH: usize = 24;

/// The statements of one method, in method order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Program {
    /// The body, in method order.
    pub(crate) stmts: Vec<Stmt>,
    /// How many statements the build produced.
    pub(crate) statements: usize,
    /// Whether any statement is a fallback: the run's representation and quality read this.
    pub(crate) ragged: bool,
    /// Every `invokedynamic` site of this body, in BCI order, with what the class states about it
    /// and what this build did with it (P3 2.1). A site is here whether it was presented or refused,
    /// because "which bootstrap was it, and why was it not a lambda" is the question A04 asks and a
    /// record that only listed the presented ones could not answer it.
    pub(crate) lambdas: Vec<LambdaRecord>,
    /// Every synthetic accessor call site of this body, in BCI order, with what the rule read about
    /// its callee and what this build did with it (P3 2.2, A12). As for a lambda, a refusal is part
    /// of the answer: a call that kept the call it had says which link of the verification failed.
    pub(crate) accessors: Vec<AccessorRecord>,
}

/// The facts of one run the build reads beside the regions and the region tree's own inputs.
///
/// These are the payload's own decode facts (the class's pool and its bootstrap table, the profile
/// whose rule set the run admits) plus the names table the statements are written with. They travel
/// as one value because every one of them is *the same run's*, and a builder that took them one by
/// one could be handed two of something.
pub(crate) struct Inputs<'a> {
    /// The class's constant pool, as the same header read decoded it.
    pub(crate) pool: &'a [CpEntryFacts],
    /// The class's `BootstrapMethods` table, as the same read decoded it.
    pub(crate) bootstrap: &'a [BootstrapMethodFacts],
    /// The profile this run presents under: the gate the `lambda@1` rule is admitted through.
    pub(crate) profile: RecoveryProfile,
    /// How many local slots the method's parameters occupy, `this` included when the caller
    /// counted it: the slots below this one are declared by the signature, not by the body.
    pub(crate) parameters: u16,
    /// The type each parameter slot holds, as the member's own descriptor states it (P3-R5): a
    /// `boolean` parameter and an `int` one share a slot shape, and only this fact tells them apart.
    pub(crate) parameter_types: &'a BTreeMap<u16, Type>,
    /// Whether the member's own descriptor returns `Z` (`(I)Z`, `()Z`, …), as the same reading of
    /// the same descriptor states it. The frames state one slot shape for the four int-sized
    /// primitives, so this signature fact is what says whether a `return` in this body presents a
    /// boolean; the caller derives it where it derives [`Self::parameter_types`], so that a builder
    /// handed this run's facts never reads a second opinion out of a descriptor itself.
    pub(crate) returns_boolean: bool,
    /// The names the presentation decided, in slot order.
    pub(crate) names: &'a NameTable,
    /// The variables each local slot holds (P3 3.4): one per slot unless the debug records name the
    /// slot over two ranges, in which case the uses are grouped into two variables with two names.
    pub(crate) reuse: &'a reuse::Plan,
    /// The concatenation chains of this body, with the shape's own declaration of which
    /// instructions they own (P3 2.2).
    pub(crate) chains: &'a concat::Plan,
    /// The class's other members, as the caller read them: the only evidence a synthetic accessor
    /// call site can be decided from (P3 2.2, A12).
    pub(crate) members: Option<&'a ClassMembers>,
    /// The verdict of the `bridge@1` rule for this very body, when the member is declared a bridge
    /// or its body is the forward a bridge is written as.
    pub(crate) bridge: Option<&'a bridge::Plan>,
    /// The construction sites of this body, with the shape's own declaration of which instructions
    /// they own (P3 2.3, `new@1`).
    pub(crate) sites: &'a init::Sites,
    /// The constructor prologue of this body, when it is an instance initializer (P3 2.3, `init@1`).
    pub(crate) prologues: &'a init::Prologues,
    /// The field accesses this body's instructions were verified to be (P3 2.3, `field@1`).
    pub(crate) fields: &'a field::Plan,
    /// The dispatch-table reads this body performs (P3 2.3, `enumswitch@1`).
    pub(crate) enums: &'a enumswitch::Plan,
}

/// The path of a region in the method's region tree (P3 3.1).
///
/// Each step is the index of a nested region inside the one that holds it, so `[2, 1]` is the
/// `else` arm of the third top-level region. The **empty** path is the method body itself, and a
/// declaration written there is in scope for the whole body.
type RegionPath = Vec<u32>;

/// Where each local variable's declaration is written (P3 3.1).
///
/// The invariant this plan keeps: a variable's declaration is written at the start of a region that
/// contains **every** use of that variable — every read and every write — so the declaration is in
/// scope at each of them. A Java local is in scope from its declaration to the end of the block that
/// declares it, and that is not the same thing as "the whole method knows this slot": a slot filled
/// in a `then` arm and read in the `else` arm or after the join is declared in a place where most of
/// its uses cannot see it, and the text does not compile.
///
/// Two cases, and the evidence for each:
///
/// * **the write that first fills the variable is in the innermost region that contains all uses** —
///   then that write's own statement declares the variable with the value it writes, which is the
///   text a source would have (`int x = 1;`) and the text this layer has always written;
/// * **otherwise** the declaration moves to the start of that innermost region and **every** write
///   becomes a plain assignment: the declaration is no longer any write's, so no write may carry it.
///
/// The "innermost region that contains all uses" is read from the region tree itself: every block
/// belongs to exactly one region (the innermost one that claims it), and the region that contains
/// all uses is the longest common prefix of their region paths.
///
/// Which variable a use belongs to is [`crate::reuse`]'s decision, and it is what makes P3 3.4's
/// slot reuse come out right: the two variables one reused slot holds have **disjoint** uses, so
/// each is planned on its own — the arm's own variable is declared at the write that fills it, in
/// the arm, while a variable written in both arms and read after the join is hoisted above the
/// branch. Both rules are the same rule; only the use sets differ.
///
/// A variable whose uses are all inside one **quoted** region is left exactly as it was: a
/// [`Region::Fallback`] writes no Java statements this layer could declare a local in, so there is
/// nothing to hoist into, and that run is already `Mixed`/`Fallback`. The same holds for a use whose
/// block the region tree does not claim: with no region to name, the variable keeps the declaration
/// it has today instead of a guess.
#[derive(Default)]
struct Declarations {
    /// The variables declared at the start of one region, by that region's path, in slot and
    /// variable order.
    at_region: BTreeMap<RegionPath, Vec<HoistedDeclaration>>,
    /// What the plan decided about each variable's type, by that variable's own identity: the one
    /// decision the hoisted declaration, the in-place declaration, the assignments, the conditions
    /// and the returns all present the variable with.
    decided: BTreeMap<LocalVariable, Decided>,
}

/// What the plan decided about one local variable's type before a statement of this body existed.
///
/// The decision is the same evidence the layer always read — a descriptor, a read of a variable the
/// same plan decided `boolean`, otherwise the frame's own type — read **once**, for every variable,
/// and then only consumed. Deciding here rather than at each use is what keeps the two declaration
/// paths from answering differently, and what lets every write be checked against one answer.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Decided {
    /// The variable holds this type: `boolean` when a descriptor states it or a variable this plan
    /// decided boolean is copied into it, and otherwise the type the frames state for the value its
    /// first write stores.
    Type(Type),
    /// No type can be decided for the variable, with the fact that stopped it: a write of it can
    /// then not be published as a declaration, and the structure that write belongs to is refused.
    Unknown(NoType),
}

impl Decided {
    /// Whether this decision states a `boolean`.
    fn is_boolean(&self) -> bool {
        matches!(self, Self::Type(Type::Boolean))
    }
}

/// Why the plan could not decide a variable's type.
///
/// The two reasons are the ones the in-place declaration path already refused with; they are
/// recorded here because the plan is where they are now read, and the write that carries the
/// refusal states the BCI it is about.
#[derive(Clone, Debug, Eq, PartialEq)]
enum NoType {
    /// The value the variable's first write stores has no frame entry that states a type
    /// (`Top`, a category-2 value's second slot, an uninitialized value, a return address).
    NoFrameEntry,
    /// The value the variable's first write stores has a frame entry that names a descriptor this
    /// layer cannot spell as a Java type.
    Unspellable(String),
}

impl NoType {
    /// The reason a refused declaration states, at the write that would have carried it.
    fn message(&self, slot: u16, at: u32) -> String {
        match self {
            Self::NoFrameEntry => format!("local {slot} has no frame entry stating its type"),
            Self::Unspellable(name) => format!(
                "the local written at BCI {at} holds a value the frames name `{name}`, which is a descriptor this layer cannot spell as a Java type, so the declaration is refused instead of writing it"
            ),
        }
    }
}

/// One declaration written at the start of a region instead of at the write that fills the variable.
#[derive(Clone)]
struct HoistedDeclaration {
    variable: LocalVariable,
    /// The plan's own decision ([`Declarations::decided`]) for this variable: the one type both
    /// declaration paths write, so a variable cannot be typed one way above the branch and another
    /// way inside it.
    ty: Type,
    /// The write whose value states the type and whose frame entry states the variable's type: the
    /// anchor the declaration is written under, exactly like the in-place declaration it replaces.
    at: u32,
}

/// What one write's declaration did.
///
/// The three outcomes are three different answers, and the ambiguity this enum removes is the defect
/// the change fixes: before it, `Ok(None)` meant both "no declaration is due" and "the declaration
/// failed and a fallback was written", so a caller could not tell a variable that needs no
/// declaration from one whose declaration was refused — and went on to write the assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Declaration {
    /// This write carries the variable's declaration, with the plan's type for it.
    Declared(Type),
    /// No declaration is due: a parameter's (the signature declares it), an already declared
    /// variable's, or a variable with no name to declare.
    NotDue,
    /// No declaration could be written: the plan decided no type for the variable, the fallback that
    /// states why is already recorded, and the assignment this write would otherwise become may not
    /// be written.
    Refused,
}

/// Plans where each local variable's declaration is written, and what type every variable's uses
/// are presented with.
///
/// The two are one plan because they are one decision: a variable's type is decided here, by the
/// variable's own identity, before a statement of the body exists, and the declaration's *place*
/// only says where that decision is written. Deciding the type here is what keeps the hoisted path
/// (which runs before any declaration exists) and the in-place path (which used to recognize the
/// declarations it had already written) from answering differently about the same value, and what
/// makes the answer independent of the order the regions are walked in.
#[allow(clippy::too_many_arguments)]
fn declarations(
    regions: &[Region],
    ssa: &SsaTable,
    operations: &Operations,
    names: &NameTable,
    reuse: &reuse::Plan,
    parameters: u16,
    parameter_types: &BTreeMap<u16, Type>,
    fields: &field::Plan,
    budget: &mut Budget,
) -> Result<Declarations, StopReason> {
    let paths = region_paths(regions);
    let uses = slot_uses(ssa, operations, reuse, &paths);
    let mut plan = Declarations {
        decided: decide_types(
            &uses,
            ssa,
            operations,
            reuse,
            parameters,
            parameter_types,
            fields,
            budget,
        )?,
        ..Declarations::default()
    };
    // The slots a guarded statement declares **in its own header** (P3 2.4): a `try (T n = …)`
    // header is the declaration, no statement of the body writes one, and hoisting a second
    // declaration above the statement would declare the same name twice.
    let resources = resource_slots(regions);
    for (variable, variable_uses) in &uses {
        // A parameter's declaration is the signature, not the body, and a variable this layer has no
        // name for is one whose writes are already reported as a fallback of their own.
        if variable.slot() < parameters
            || names.text(*variable).is_none()
            || resources.contains(&variable.slot())
        {
            continue;
        }
        let Some(region) = declaration_region(variable_uses, &paths) else {
            continue;
        };
        // The write that fills the variable first, in method order: the region path a block stands
        // in is the order the regions are written in, and the bytecode index orders the blocks of
        // one region.
        let Some(first) = variable_uses
            .iter()
            .filter(|use_| use_.written.is_some())
            .min_by_key(|use_| (use_.path.clone(), use_.bci))
        else {
            continue;
        };
        if first.path.as_ref() == Some(&region) {
            // Every use is in the first write's own region: the declaration the write carries is in
            // scope for all of them, which is the text this layer has always written.
            continue;
        }
        // The declaration states the type the plan decided for this variable — the same answer the
        // write that fills it would state in place. A variable whose type could not be decided keeps
        // the write's own refusal, which is what the in-place path states for it.
        let Some(Decided::Type(ty)) = plan.decided.get(variable) else {
            continue;
        };
        plan.at_region
            .entry(region)
            .or_default()
            .push(HoistedDeclaration {
                variable: *variable,
                ty: ty.clone(),
                at: first.bci,
            });
    }
    Ok(plan)
}

/// Every local access of this body, grouped by the variable it belongs to.
///
/// A read is recorded where it reads and a write where it writes, with the region path of the block
/// the instruction stands in — the order the regions are written in is the path's own order, which
/// is what makes "the first write" a fact about the method rather than about this walk.
fn slot_uses(
    ssa: &SsaTable,
    operations: &Operations,
    reuse: &reuse::Plan,
    paths: &RegionPaths,
) -> BTreeMap<LocalVariable, Vec<SlotUse>> {
    let mut uses: BTreeMap<LocalVariable, Vec<SlotUse>> = BTreeMap::new();
    for block in ssa.blocks() {
        let path = paths.paths.get(block.block());
        for instruction in block.instructions() {
            for (slot, _) in instruction.reads() {
                if let Slot::Local(slot) = slot
                    && let Some(variable) = reuse.variable_at(*slot, instruction.bci())
                {
                    uses.entry(variable).or_default().push(SlotUse {
                        path: path.cloned(),
                        bci: instruction.bci(),
                        written: None,
                        stored: None,
                    });
                }
            }
            for (slot, value) in instruction.writes() {
                if let Slot::Local(slot) = slot
                    && let Some(variable) = reuse.variable_at(*slot, instruction.bci())
                {
                    uses.entry(variable).or_default().push(SlotUse {
                        path: path.cloned(),
                        bci: instruction.bci(),
                        written: Some(*value),
                        stored: store_operand(operations, instruction),
                    });
                }
            }
        }
    }
    uses
}

/// The write one variable's type is decided from, and the values that state it.
struct FirstWrite {
    /// The write's own bytecode index: the anchor a declaration or a refusal is written under.
    at: u32,
    /// The value the slot takes.
    written: ValueId,
    /// The value the writing instruction **reads** as the one it stores, when the two differ: it is
    /// this value — not the slot's own — whose evidence states what the variable holds.
    stored: ValueId,
}

/// Decides every variable's type once, from a finite list of evidence, before a statement exists.
///
/// The evidence is a closed list, and nothing is followed anywhere else:
///
/// * a **descriptor** fact about the value a variable's first write stores: a `Z` parameter slot's
///   load, a call whose callee descriptor returns `Z`, a field read a `field@1` claim states is `Z`
///   ([`boolean_proof`]) — for a parameter variable, its own descriptor is that same fact, read
///   directly;
/// * a **read of another variable this plan decided boolean**, one read and one hop: a variable whose
///   first write stores such a read is boolean too, and a chain of copies therefore reaches a
///   fixpoint. Nothing else is followed: a store's own value, a value two definitions away and a
///   value merged out of several pushes state nothing;
/// * the **frame's type** for the value a variable's first write stores — the type of every
///   non-boolean decision, because the frames state one slot shape for the four int-sized
///   primitives and cannot tell a `boolean` from an `int`;
/// * **nothing else**: the `0`/`1` literal is refused as the *initiating* evidence (a fresh local
///   states no type, so `int x = 0;` and `boolean c = true;` are the same bytes), and it adapts to
///   `true`/`false` only where the target is already decided boolean — which is how the consumers
///   spell it, not a decision of this pass.
///
/// A variable whose first write states no usable type is [`Decided::Unknown`]: the write that would
/// have declared it refuses instead, and no second place decides a type for it.
///
/// The propagation runs on a worklist to a fixpoint, so a chain of copies is decided the same way
/// whatever order the variables are met in, and every queue entry is billed and polled against the
/// run's own budget and cancellation before it is processed. The queue holds each variable at most
/// once, so the pass is bounded by the variables' own write count.
#[allow(clippy::too_many_arguments)]
fn decide_types(
    uses: &BTreeMap<LocalVariable, Vec<SlotUse>>,
    ssa: &SsaTable,
    operations: &Operations,
    reuse: &reuse::Plan,
    parameters: u16,
    parameter_types: &BTreeMap<u16, Type>,
    fields: &field::Plan,
    budget: &mut Budget,
) -> Result<BTreeMap<LocalVariable, Decided>, StopReason> {
    // The write each variable's type is decided from: the first one in method order, which is the
    // write both declaration paths read today.
    let mut first: BTreeMap<LocalVariable, FirstWrite> = BTreeMap::new();
    for (variable, variable_uses) in uses {
        let Some(use_) = variable_uses
            .iter()
            .filter(|use_| use_.written.is_some())
            .min_by_key(|use_| (use_.path.clone(), use_.bci))
        else {
            continue;
        };
        let Some(written) = use_.written else {
            continue;
        };
        first.insert(
            *variable,
            FirstWrite {
                at: use_.bci,
                written,
                stored: use_.stored.unwrap_or(written),
            },
        );
    }
    // The variables a descriptor proves boolean on their own: the parameter slots the member's own
    // descriptor declares `Z`, and the variables whose first write stores a value a descriptor
    // proves boolean.
    let mut boolean_variables: BTreeSet<LocalVariable> = BTreeSet::new();
    let mut queue: VecDeque<LocalVariable> = VecDeque::new();
    for (variable, write) in &first {
        let descriptor_proof = (variable.slot() < parameters
            && matches!(parameter_types.get(&variable.slot()), Some(Type::Boolean)))
            || boolean_proof(ssa, operations, parameter_types, fields, write.stored);
        if descriptor_proof && boolean_variables.insert(*variable) {
            queue.push_back(*variable);
        }
    }
    // Which variables read each variable, one read being one edge: a variable whose first write
    // stores a read of a boolean variable is boolean itself.
    let mut readers: BTreeMap<LocalVariable, BTreeSet<LocalVariable>> = BTreeMap::new();
    for (variable, write) in &first {
        if let Some(read) = read_variable(ssa, operations, reuse, write.stored, write.at) {
            readers.entry(read).or_default().insert(*variable);
        }
    }
    while let Some(variable) = queue.pop_front() {
        let Some(write) = first.get(&variable) else {
            continue;
        };
        // The work this entry pays for is the decision it spreads; the entry's own write BCI is
        // where a run that cannot afford it stops, exactly as the statements bill their own.
        charge(budget, CountedBudgetDimension::IrItems, 1, Some(write.at))?;
        poll(budget, Some(write.at))?;
        for reader in readers.get(&variable).into_iter().flatten() {
            if boolean_variables.insert(*reader) {
                queue.push_back(*reader);
            }
        }
    }
    // The decision every consumer reads: the descriptor's boolean for a parameter, the propagated
    // boolean for a variable a read proved one, and otherwise the frame's own type for the value
    // the first write stores.
    let mut decided: BTreeMap<LocalVariable, Decided> = BTreeMap::new();
    for (variable, write) in &first {
        let decision = if variable.slot() < parameters {
            // A parameter's type is the signature's, not the frames': the descriptor states it, and
            // a body's write cannot widen it.
            match parameter_types.get(&variable.slot()) {
                Some(ty) => Decided::Type(ty.clone()),
                None => continue,
            }
        } else if boolean_variables.contains(variable) {
            Decided::Type(Type::Boolean)
        } else {
            match value_type(ssa.value(write.written).ty()) {
                Ok(Some(ty)) => Decided::Type(ty),
                Ok(None) => Decided::Unknown(NoType::NoFrameEntry),
                Err(name) => Decided::Unknown(NoType::Unspellable(name)),
            }
        };
        decided.insert(*variable, decision);
    }
    Ok(decided)
}

/// The variable one value denotes a read of, when it denotes a read at all.
///
/// A `load` names the variable the slot holds at the load's own BCI; the entry state or the phi of
/// a local slot names the variable the slot holds at the use point `at`. Everything else — a value
/// another instruction produced, a stack value, a caught exception — denotes no variable, which is
/// the boundary the plan's propagation keeps: one read, one hop, and no definition chain.
fn read_variable(
    ssa: &SsaTable,
    operations: &Operations,
    reuse: &reuse::Plan,
    value: ValueId,
    at: u32,
) -> Option<LocalVariable> {
    let (slot, read_at) = match ssa.value(value).def() {
        Definition::Instruction { bci, .. } => match operations.get(*bci) {
            Some(Operation::Load { slot }) => (*slot, *bci),
            _ => return None,
        },
        Definition::Entry { slot, .. } | Definition::Phi { slot, .. } => match slot {
            Slot::Local(slot) => (*slot, at),
            Slot::Stack(_) => return None,
        },
        Definition::Caught { .. } => return None,
    };
    reuse.variable_at(slot, read_at)
}

/// Every slot a guarded statement's header declares.
pub(crate) fn resource_slots(regions: &[Region]) -> BTreeSet<u16> {
    let mut slots: BTreeSet<u16> = BTreeSet::new();
    let mut walk = |region: &Region| {
        if let Region::Guard { plan, .. } = region
            && let guard::Shape::Resources(resources) = plan.shape()
        {
            for resource in resources {
                slots.insert(resource.slot());
            }
        }
    };
    for region in regions {
        collect_guards(region, &mut walk);
    }
    slots
}

/// Calls one visitor on every guarded region of a tree.
fn collect_guards(region: &Region, visit: &mut impl FnMut(&Region)) {
    visit(region);
    match region {
        Region::If {
            then_arm, else_arm, ..
        } => {
            collect_guards(then_arm, visit);
            collect_guards(else_arm, visit);
        }
        Region::Switch { groups, .. } => {
            for group in groups {
                collect_guards(&group.arm, visit);
            }
        }
        Region::Loop { body, .. } => collect_guards(body, visit),
        Region::Guard { .. } | Region::Straight { .. } | Region::Fallback { .. } => {}
    }
}

/// One local access of one instruction, with the region the instruction's block stands in.
struct SlotUse {
    /// The innermost region whose statements hold the instruction, or `None` when the region tree
    /// does not claim its block.
    path: Option<RegionPath>,
    bci: u32,
    /// The value a write stores; absent for a read.
    written: Option<ValueId>,
    /// The value a write **reads** as the one it stores, when the two are different values: a
    /// `…; istore` writes the slot and reads the stack, and only the second is the value whose own
    /// evidence says what type the slot holds. Absent where the write reads no stack value (an
    /// `iinc`) and where the instruction produces the value it writes (an invocation the SSA
    /// already places in the slot).
    stored: Option<ValueId>,
}

/// The path of the innermost region every use of one slot sits in.
///
/// `None` means there is no such region this layer could write a declaration in: a use whose block
/// the region tree does not claim, or **any** use inside a region that is quoted bytecode. The second
/// case is why this is not just "the common region is not a fallback": a slot whose uses span a quoted
/// region and a structured one would have to be declared at their common ancestor — but the quoted
/// text does not name the slot at all, so a declaration written above it would declare a variable the
/// produced text never uses. Such a slot keeps the declaration it has today (inside a quote: none),
/// and that run is already `Mixed`/`Fallback`.
fn declaration_region(uses: &[SlotUse], paths: &RegionPaths) -> Option<RegionPath> {
    let mut region: Option<RegionPath> = None;
    for use_ in uses {
        let path = use_.path.as_ref()?;
        if paths.fallbacks.contains(path) {
            return None;
        }
        region = Some(match region {
            None => path.clone(),
            Some(current) => common_prefix(&current, path),
        });
    }
    let region = region?;
    (!paths.fallbacks.contains(&region)).then_some(region)
}

/// The longest prefix two region paths share: the innermost region that contains both.
fn common_prefix(left: &RegionPath, right: &RegionPath) -> RegionPath {
    left.iter()
        .zip(right)
        .take_while(|(left, right)| left == right)
        .map(|(step, _)| *step)
        .collect()
}

/// The path of a region nested inside the region `path` names, at the given child index.
fn child(path: &[u32], index: u32) -> RegionPath {
    let mut nested = path.to_vec();
    nested.push(index);
    nested
}

/// Which region holds each block, as the path from the method body down to it.
struct RegionPaths {
    paths: BTreeMap<CanonicalBlockId, RegionPath>,
    /// The paths whose region is a [`Region::Fallback`]: a quoted run, which holds no statements
    /// this layer could declare a local in.
    fallbacks: BTreeSet<RegionPath>,
}

/// The region tree as paths, for the slots whose declaration has to be moved.
fn region_paths(regions: &[Region]) -> RegionPaths {
    let mut paths = RegionPaths {
        paths: BTreeMap::new(),
        fallbacks: BTreeSet::new(),
    };
    for (index, region) in regions.iter().enumerate() {
        collect_paths(
            region,
            &child(&[], u32::try_from(index).unwrap_or(u32::MAX)),
            &mut paths,
        );
    }
    paths
}

/// Fills the path of every block one region claims, innermost region first.
///
/// A nested region is walked **before** the region that holds it, so a block both could claim
/// belongs to the inner one: a loop's header is the body's first block, and the body's statements are
/// where its statements are written while the loop's own test is written inside the loop statement.
fn collect_paths(region: &Region, path: &RegionPath, out: &mut RegionPaths) {
    match region {
        Region::If {
            then_arm, else_arm, ..
        } => {
            collect_paths(then_arm, &child(path, 0), out);
            collect_paths(else_arm, &child(path, 1), out);
        }
        Region::Switch { groups, .. } => {
            for (index, group) in groups.iter().enumerate() {
                collect_paths(
                    &group.arm,
                    &child(path, u32::try_from(index).unwrap_or(u32::MAX)),
                    out,
                );
            }
        }
        Region::Loop { body, .. } => collect_paths(body, &child(path, 0), out),
        Region::Straight { .. } | Region::Fallback { .. } | Region::Guard { .. } => {}
    }
    if matches!(region, Region::Fallback { .. }) {
        out.fallbacks.insert(path.clone());
    }
    for block in own_blocks(region) {
        out.paths.entry(block).or_insert_with(|| path.clone());
    }
}

/// The blocks a region writes the statements of, as opposed to the ones its nested regions own.
fn own_blocks(region: &Region) -> Vec<CanonicalBlockId> {
    match region {
        Region::Straight { blocks } => blocks.clone(),
        Region::If { prefix, branch, .. } => {
            let mut blocks = prefix.clone();
            blocks.push(branch.clone());
            blocks
        }
        Region::Switch { prefix, branch, .. } => {
            let mut blocks = prefix.clone();
            blocks.push(branch.clone());
            blocks
        }
        // The test block is the condition written *inside* the loop statement; the header belongs
        // to the body when the body claims it (`collect_paths` walks the body first).
        Region::Loop { header, test, .. } => vec![header.clone(), test.clone()],
        Region::Fallback { blocks, .. } => blocks.clone(),
        // A guarded statement writes its own header and the blocks of its body; every other block it
        // claims — the closes, the handlers, the later resources' initialisations — produces no
        // statement of its own, which is exactly what keeps a close from running twice.
        Region::Guard { prefix, plan } => {
            let mut blocks = prefix.clone();
            blocks.extend(plan.owned().iter().cloned());
            blocks
        }
    }
}

/// Builds the statements of one method from its regions.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build(
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    operations: &Operations,
    inputs: Inputs<'_>,
    regions: &[Region],
    budget: &mut Budget,
) -> Result<Program, StopReason> {
    let mut instructions: BTreeMap<u32, &SsaInstruction> = BTreeMap::new();
    let mut block_of: BTreeMap<u32, CanonicalBlockId> = BTreeMap::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            instructions.insert(instruction.bci(), instruction);
            block_of.insert(instruction.bci(), block.block().clone());
        }
    }
    let declarations = declarations(
        regions,
        ssa,
        operations,
        inputs.names,
        inputs.reuse,
        inputs.parameters,
        inputs.parameter_types,
        inputs.fields,
        budget,
    )?;
    let mut builder = Builder {
        canonical,
        ssa,
        operations,
        pool: inputs.pool,
        bootstrap: inputs.bootstrap,
        profile: inputs.profile,
        parameters: inputs.parameters,
        parameter_types: inputs.parameter_types,
        returns_boolean: inputs.returns_boolean,
        names: inputs.names,
        reuse: inputs.reuse,
        chains: inputs.chains,
        members: inputs.members,
        bridge: inputs.bridge,
        sites: inputs.sites,
        prologues: inputs.prologues,
        fields: inputs.fields,
        enums: inputs.enums,
        instructions,
        block_of,
        budget,
        declared: BTreeSet::new(),
        stmts: Vec::new(),
        statements: 0,
        ragged: false,
        lambdas: Vec::new(),
        lambda_params: BTreeSet::new(),
        accessors: Vec::new(),
        deferred: Vec::new(),
        declarations,
    };
    // A slot whose uses span more than one top-level region is declared wherever every one of them
    // can see it: at the start of the body, before the first region's text.
    builder.declare_at(&[])?;
    for (index, region) in regions.iter().enumerate() {
        let path = child(&[], u32::try_from(index).unwrap_or(u32::MAX));
        builder.region(region, &path)?;
    }
    Ok(Program {
        statements: builder.statements,
        ragged: builder.ragged,
        stmts: builder.stmts,
        lambdas: builder.lambdas,
        accessors: builder.accessors,
    })
}

struct Builder<'a> {
    canonical: &'a CanonicalCfg,
    ssa: &'a SsaTable,
    operations: &'a Operations,
    /// The class's pool: where a dynamic site's own entry, the bootstrap handles and the method
    /// types are named (P3 2.1).
    pool: &'a [CpEntryFacts],
    /// The class's bootstrap table: the only fact that says whether a site is a lambda.
    bootstrap: &'a [BootstrapMethodFacts],
    /// The profile whose rule set this build admits.
    profile: RecoveryProfile,
    /// How many local slots the method's parameters occupy, `this` included when the caller
    /// counted it: the slots below this one are declared by the signature, not by the body.
    parameters: u16,
    /// The type each parameter slot holds, as the member's own descriptor states it (P3-R5).
    parameter_types: &'a BTreeMap<u16, Type>,
    /// Whether the member's own descriptor returns `Z`: the fact that decides whether a `return` of
    /// this body presents a boolean (P3-R5's reading, in the return position).
    returns_boolean: bool,
    names: &'a NameTable,
    /// The variables each local slot holds (P3 3.4): which of a slot's two variables a use point
    /// belongs to, and therefore which name that use is written with.
    reuse: &'a reuse::Plan,
    /// The concatenation chains this body's verified shapes own (P3 2.2).
    chains: &'a concat::Plan,
    /// The class's other members, when the caller handed them over (P3 2.2, A12).
    members: Option<&'a ClassMembers>,
    /// The `bridge@1` rule's verdict for this body, when it has one (P3 2.2).
    bridge: Option<&'a bridge::Plan>,
    /// The construction sites this body builds (P3 2.3).
    sites: &'a init::Sites,
    /// The prologue of this body, when the body is an instance initializer (P3 2.3).
    prologues: &'a init::Prologues,
    /// The field accesses this body's instructions were verified to be (P3 2.3).
    fields: &'a field::Plan,
    /// The dispatch-table reads this body performs (P3 2.3).
    enums: &'a enumswitch::Plan,
    instructions: BTreeMap<u32, &'a SsaInstruction>,
    /// The block each instruction belongs to: which block's own entry state and writes state what a
    /// local slot holds where that instruction runs (P3 1.3d).
    block_of: BTreeMap<u32, CanonicalBlockId>,
    budget: &'a mut Budget,
    /// The variables declared so far: a write of a variable whose declaration is already written
    /// becomes an assignment, and every variable's declaration is written once.
    declared: BTreeSet<LocalVariable>,
    stmts: Vec<Stmt>,
    statements: usize,
    ragged: bool,
    /// Every dynamic site this build read, in the order it reached them.
    lambdas: Vec<LambdaRecord>,
    /// The parameter names the lambda shapes of this body have already taken, so that no two of
    /// them spell the same identifier.
    lambda_params: BTreeSet<String>,
    /// Every synthetic accessor call site this build read, in the order it reached them.
    accessors: Vec<AccessorRecord>,
    /// The values one instruction's statement was deferred to a reader for, with the BCI of the
    /// instruction that produced them: what a quote has to name when the reader turns out not to
    /// write them after all (P3 2.3 §0).
    deferred: Vec<(ValueId, u32)>,
    /// Where each local slot's declaration is written (P3 3.1), and what type the plan decided for
    /// every variable: the slot whose declaration is not the first write's is declared at the start
    /// of the region that contains all of its uses, and every consumer of a variable's type reads
    /// this one decision instead of deciding again.
    declarations: Declarations,
}

impl Builder<'_> {
    /// What the plan decided about one variable's type, when the plan reached that variable.
    fn decision(&self, variable: LocalVariable) -> Option<&Decided> {
        self.declarations.decided.get(&variable)
    }

    /// Whether the plan decided one variable holds a `boolean`.
    fn decided_boolean(&self, variable: LocalVariable) -> bool {
        self.decision(variable).is_some_and(Decided::is_boolean)
    }

    /// Appends the statements of one region, with the region's own declarations first.
    fn region(&mut self, region: &Region, path: &RegionPath) -> Result<(), StopReason> {
        self.declare_at(path)?;
        match region {
            Region::Straight { blocks } => {
                for block in blocks {
                    self.block(block)?;
                }
                Ok(())
            }
            Region::If {
                prefix,
                branch,
                branch_bci,
                then_arm,
                else_arm,
                ..
            } => {
                for block in prefix {
                    self.block(block)?;
                }
                // Everything the test block does *before* its branch ran before the branch in the
                // bytecode too, so it is written here, in order: a store or a call the test block
                // makes is a statement of its own, and it must not be dropped just because the
                // branch that follows it is what becomes the `if`.
                self.test_effects(branch, *branch_bci)?;
                let cond = match self.test_expr(*branch_bci, false) {
                    Ok(cond) => cond,
                    Err(reason) => {
                        let bcis = self.region_quote(region, *branch_bci);
                        return self.fallback(bcis, &reason, *branch_bci);
                    }
                };
                // The condition's text is tested *by* the branch: the `if` statement is the branch,
                // and the condition node presents it. One BCI reaching two nodes with the two
                // provenances is exactly what the segment table records as it writes.
                let cond = cond.derived_from(*branch_bci);
                let mut then_body = Vec::new();
                self.arm(then_arm, &mut then_body, &child(path, 0))?;
                let mut else_body = Vec::new();
                self.arm(else_arm, &mut else_body, &child(path, 1))?;
                self.push(Stmt::new(
                    StmtKind::If {
                        cond,
                        then_body,
                        else_body,
                    },
                    OriginSet::new(Origin::direct(*branch_bci)),
                ))
            }
            Region::Switch {
                prefix,
                branch,
                branch_bci,
                groups,
                ..
            } => {
                for block in prefix {
                    self.block(block)?;
                }
                // As for an `if`: what the switch block does before its selector is read runs
                // before the switch in the bytecode too.
                self.test_effects(branch, *branch_bci)?;
                let Some(instruction) = self.instructions.get(branch_bci).copied() else {
                    let bcis = self.region_quote(region, *branch_bci);
                    return self.fallback(
                        bcis,
                        &format!("no names record for the switch at BCI {branch_bci}"),
                        *branch_bci,
                    );
                };
                let Some((_, value)) = stack_operands(instruction).last().copied() else {
                    let bcis = self.region_quote(region, *branch_bci);
                    return self.fallback(
                        bcis,
                        &format!("the switch at BCI {branch_bci} reads no value to select on"),
                        *branch_bci,
                    );
                };
                // The selector is a value the switch reads; the text that renders it is anchored
                // where the value was produced, and the switch that reads it is added as a
                // presented anchor, exactly like a branch's condition.
                let value = match self.render_value(value, *branch_bci, 0) {
                    Ok(value) => value.derived_from(*branch_bci),
                    Err(reason) => {
                        let bcis = self.region_quote(region, *branch_bci);
                        return self.fallback(bcis, &reason, *branch_bci);
                    }
                };
                let mut arms = Vec::with_capacity(groups.len());
                for (index, group) in groups.iter().enumerate() {
                    let mut body = Vec::new();
                    self.arm(
                        &group.arm,
                        &mut body,
                        &child(path, u32::try_from(index).unwrap_or(u32::MAX)),
                    )?;
                    arms.push(SwitchArm {
                        keys: group.keys.clone(),
                        default: group.default,
                        body,
                    });
                }
                self.push(Stmt::new(
                    StmtKind::Switch { value, arms },
                    OriginSet::new(Origin::direct(*branch_bci)),
                ))
            }
            Region::Loop {
                test_bci,
                form,
                continuation,
                body,
                ..
            } => {
                // The test is written *inside* the loop statement, so the values it reads and the
                // calls it makes run once per evaluation — the loop's own count, not the count of a
                // hoisted copy. Which of the two senses continues the loop is a decode fact
                // (`Continuation`), and the condition is that sense, not its negation.
                let taken = *continuation == Continuation::Taken;
                let cond = match self.test_expr(*test_bci, taken) {
                    Ok(cond) => cond,
                    Err(reason) => {
                        let bcis = self.region_quote(region, *test_bci);
                        return self.fallback(bcis, &reason, *test_bci);
                    }
                };
                let cond = cond.derived_from(*test_bci);
                let mut loop_body = Vec::new();
                self.arm(body, &mut loop_body, &child(path, 0))?;
                let kind = match form {
                    LoopForm::While => StmtKind::While {
                        cond,
                        body: loop_body,
                    },
                    LoopForm::DoWhile => StmtKind::DoWhile {
                        cond,
                        body: loop_body,
                    },
                };
                self.push(Stmt::new(kind, OriginSet::new(Origin::direct(*test_bci))))
            }
            Region::Guard { prefix, plan } => {
                // The walk wrote nothing of the statement's own header: the first resource's
                // initialisation (or the monitor's entry) is the last block it reached, and the
                // region owns it now.
                for block in prefix {
                    self.block(block)?;
                }
                self.range(plan.lead())?;
                match plan.shape() {
                    guard::Shape::Resources(resources) => {
                        let mut declarations = Vec::with_capacity(resources.len());
                        for resource in resources {
                            match self.resource_declaration(resource) {
                                Ok(declaration) => declarations.push(declaration),
                                Err(reason) => {
                                    let at = resource.close_bci();
                                    let bcis = self.region_quote(region, at);
                                    return self.fallback(bcis, &reason, at);
                                }
                            }
                        }
                        // The body's statements are written between the braces, in the order the
                        // instructions run: the closes the normal path performs are *not* written
                        // here — the compiler writes them for the resource the header declares,
                        // which is what makes each close run exactly once per path.
                        let body = self.body_range(plan.body())?;
                        let mut origin = OriginSet::new(Origin::direct(
                            resources
                                .first()
                                .map(|resource| resource.init().0)
                                .unwrap_or(0),
                        ));
                        for bci in plan.facts() {
                            // Every instruction the proof read — the closes, the suppressions, the
                            // rethrows — is an anchor of the text that took its place, so the
                            // relationship the statement preserves is answerable from the artifact.
                            origin = origin.plus_derived(Origin::derived(*bci));
                        }
                        self.push(Stmt::new(
                            StmtKind::Try {
                                resources: declarations,
                                body,
                            },
                            origin,
                        ))
                    }
                    guard::Shape::Monitor { enter_bci } => {
                        let lock = match self.lock_expr(*enter_bci) {
                            Ok(lock) => lock,
                            Err(reason) => {
                                let bcis = self.region_quote(region, *enter_bci);
                                return self.fallback(bcis, &reason, *enter_bci);
                            }
                        };
                        let body = self.body_range(plan.body())?;
                        let mut origin = OriginSet::new(Origin::direct(*enter_bci));
                        for bci in plan.facts() {
                            origin = origin.plus_derived(Origin::derived(*bci));
                        }
                        self.push(Stmt::new(StmtKind::Synchronized { lock, body }, origin))
                    }
                }
            }
            Region::Fallback { blocks, reason } => {
                let mut bcis: Vec<u32> = blocks
                    .iter()
                    .flat_map(|block| self.covered_bcis(block))
                    .collect();
                // P3-R7: a refusal may state instruction starts that no block covers at all — the
                // ones the graph failed to account for. The quote has to name them beside the
                // region's own blocks, or the artifact would refuse a body while dropping exactly
                // the bytes it refused it for, which is the silence the refusal exists to undo.
                for bci in reason.unaccounted() {
                    if !bcis.contains(bci) {
                        bcis.push(*bci);
                    }
                }
                let at = bcis.first().copied().unwrap_or(0);
                self.fallback(bcis, &reason.message(), at)
            }
        }
    }

    /// The header declaration one proved resource becomes.
    ///
    /// The type is the **value's own**: the value stored into the resource's slot is what the frames
    /// type it as, and a value they name as no reference type (or as a bare `Object`) is refused —
    /// the header would have to declare a resource the text cannot name, and `Object` is not a
    /// resource a `try` header can hold.
    fn resource_declaration(&mut self, resource: &guard::Resource) -> Result<ResourceDecl, String> {
        let (from, to) = resource.init();
        // The initialisation is an instruction range, not a block: the canonical graph fuses
        // straight-line code, and the store that lands the resource is the range's last instruction.
        let Some(store) = self
            .instructions
            .range(from..to)
            .next_back()
            .map(|(_, instruction)| *instruction)
        else {
            return Err(format!(
                "the resource's initialisation at BCI {from} holds no instruction this run decoded"
            ));
        };
        let Some((_, value)) = store
            .writes()
            .iter()
            .find(|(slot, _)| matches!(slot, Slot::Local(_)))
        else {
            return Err(format!(
                "the initialisation at BCI {} stores no local this run names",
                store.bci()
            ));
        };
        let value = *value;
        // The type comes from the value the store **wrote** — the slot's own type — and the text
        // from the value it **read**: the store itself is not an expression, and rendering its own
        // write would ask the store to produce one.
        let ty = match value_type(self.ssa.value(value).ty()) {
            Ok(Some(Type::Reference(name))) if name != "Object" => Type::Reference(name),
            Ok(_) => {
                return Err(format!(
                    "the resource at BCI {} holds a value the frames name as no reference type, so the header cannot declare it",
                    store.bci()
                ));
            }
            // A name the frames state that cannot be spelled as a Java type: the header has no
            // declaration to write, and the fact that stopped it is the reason.
            Err(name) => {
                return Err(format!(
                    "the resource at BCI {} holds a value the frames name `{name}`, which is a descriptor this layer cannot spell as a Java type, so the header cannot declare it",
                    store.bci()
                ));
            }
        };
        // The slot of a guarded statement's header is never split (P3 3.4): the header writes the
        // declaration, so the slot it takes stays one variable, and a run that somehow split it
        // states no name here rather than writing one of its two variables' names for both.
        let Some(name) = self.names.whole(resource.slot()).map(RenderedName::text) else {
            return Err(format!(
                "the resource at BCI {} lives in slot {}, which has no name",
                store.bci(),
                resource.slot()
            ));
        };
        let name = name.to_owned();
        let at = store.bci();
        let Some((_, stored)) = stack_operands(store).first().copied() else {
            return Err(format!(
                "the resource's initialisation at BCI {at} stores no value this run names"
            ));
        };
        let value = self.render_value(stored, at, 0)?;
        // The header declares the slot: a body that wrote it again would otherwise declare it a
        // second time, and the close the compiler writes reads the name the header gives it.
        self.declared.insert(LocalVariable::whole(resource.slot()));
        Ok(ResourceDecl { ty, name, value })
    }

    /// Writes the statements of one instruction range, in order.
    ///
    /// This is how a guarded statement writes the region it guards: the body's instructions and the
    /// block that holds them are not the same thing, because the canonical graph fuses the resource's
    /// initialisation, the body and the first close of the normal path into one node.
    fn range(&mut self, span: (u32, u32)) -> Result<(), StopReason> {
        let instructions: Vec<SsaInstruction> = self
            .instructions
            .range(span.0..span.1)
            .map(|(_, instruction)| (*instruction).clone())
            .collect();
        for instruction in &instructions {
            self.instruction(instruction)?;
        }
        Ok(())
    }

    /// The guarded body's statements, as the list the statement node holds.
    fn body_range(&mut self, span: (u32, u32)) -> Result<Vec<Stmt>, StopReason> {
        let mark = self.stmts.len();
        self.range(span)?;
        Ok(self.stmts.split_off(mark))
    }

    /// The lock expression one verified `synchronized` header reads.
    ///
    /// The value is the `monitorenter`'s own operand — the object the bytecode entered — read
    /// through the duplications javac's idiom puts in front of it (`dup; astore slot; monitorenter`):
    /// a `dup` is not an expression, and the value it duplicated is the one the header writes. It is
    /// rendered where it is produced, so the header runs exactly what the entry block ran.
    fn lock_expr(&mut self, enter_bci: u32) -> Result<Expr, String> {
        let Some(instruction) = self.instructions.get(&enter_bci).copied() else {
            return Err(format!(
                "the monitor at BCI {enter_bci} has no record in this run's names"
            ));
        };
        let Some((_, value)) = stack_operands(instruction).first().copied() else {
            return Err(format!(
                "the monitor at BCI {enter_bci} reads no value to lock"
            ));
        };
        let mut value = value;
        for _ in 0..8 {
            let Definition::Instruction { bci, .. } = self.ssa.value(value).def() else {
                break;
            };
            if !matches!(self.operations.get(*bci), Some(Operation::Duplicate)) {
                break;
            }
            let Some(producer) = self.instructions.get(bci).copied() else {
                break;
            };
            let Some((_, duplicated)) = stack_operands(producer).first().copied() else {
                break;
            };
            value = duplicated;
        }
        self.render_value(value, enter_bci, 0)
    }

    /// Writes what a test block does before the test itself, in the order it does it.
    ///
    /// A block whose terminal instruction is a branch can still hold statements before it — an
    /// assignment made while the condition is evaluated, a call whose result the test reads. For an
    /// `if` or a `switch` those run exactly once, before the test, so writing them there keeps their
    /// count and their order. (A loop's test block is the one place this cannot be done: its
    /// statements would run once per iteration, and a `while (…)` has nowhere to write them that
    /// does — which is why [`crate::region`] refuses such a loop instead of hoisting them.)
    fn test_effects(&mut self, block: &CanonicalBlockId, test_bci: u32) -> Result<(), StopReason> {
        let Some(names) = self.ssa.block(block) else {
            return Ok(());
        };
        let instructions: Vec<SsaInstruction> = names
            .instructions()
            .iter()
            .filter(|instruction| instruction.bci() != test_bci)
            .cloned()
            .collect();
        for instruction in &instructions {
            self.instruction(instruction)?;
        }
        Ok(())
    }

    /// The condition one branch's test states, as the structure that holds it writes it.
    ///
    /// `taken` says which sense of the branch the structure continues on: an `if` always writes the
    /// fall-through condition (the branch *leaves* the `if` when its sense holds), a loop writes the
    /// sense that iterates. Both are the same decode fact read two ways.
    fn test_expr(&mut self, branch_bci: u32, taken: bool) -> Result<Expr, String> {
        let Some((op, _)) = self
            .operations
            .get(branch_bci)
            .and_then(Operation::comparison)
        else {
            return Err(format!(
                "the branch at BCI {branch_bci} has no decoded sense, so its condition cannot be written"
            ));
        };
        let Some(instruction) = self.instructions.get(&branch_bci).copied() else {
            return Err(format!(
                "no names record for the branch at BCI {branch_bci}"
            ));
        };
        condition(op, &stack_operands(instruction), self, branch_bci, taken)
    }

    /// Appends one arm's statements to a vector of its own, so the `if` can hold them.
    fn arm(
        &mut self,
        region: &Region,
        into: &mut Vec<Stmt>,
        path: &RegionPath,
    ) -> Result<(), StopReason> {
        let outer = std::mem::take(&mut self.stmts);
        self.region(region, path)?;
        let arm = std::mem::replace(&mut self.stmts, outer);
        into.extend(arm);
        Ok(())
    }

    /// Writes the declarations a region holds at its own start (P3 3.1).
    ///
    /// A variable is here when the write that first fills it is *not* in the innermost region that
    /// contains all of its uses: declaring it at that write would put it out of scope at the uses
    /// outside that write's region. The declaration written here carries no value — the writes that
    /// fill the variable are the assignments that follow — and every write of the variable then
    /// writes a plain assignment, because the declaration is not any write's any more.
    ///
    /// The statement is anchored at the write whose value states the variable's type, the same anchor
    /// the in-place declaration carried: the declaration's evidence is that instruction.
    fn declare_at(&mut self, path: &[u32]) -> Result<(), StopReason> {
        let Some(declared) = self.declarations.at_region.get(path).cloned() else {
            return Ok(());
        };
        for declaration in declared {
            let Some(name) = self.names.text(declaration.variable).map(str::to_owned) else {
                continue;
            };
            self.declared.insert(declaration.variable);
            self.push(Stmt::new(
                StmtKind::Declare {
                    ty: declaration.ty,
                    name,
                    value: None,
                },
                OriginSet::new(Origin::direct(declaration.at)),
            ))?;
        }
        Ok(())
    }

    /// Appends the statements of one block, in BCI order.
    fn block(&mut self, block: &CanonicalBlockId) -> Result<(), StopReason> {
        let Some(names) = self.ssa.block(block) else {
            let bcis = self.covered_bcis(block);
            let at = bcis.first().copied().unwrap_or(block.bci());
            return self.fallback(
                bcis,
                &format!("no names record for the block at BCI {}", block.bci()),
                at,
            );
        };
        let instructions: Vec<SsaInstruction> = names.instructions().to_vec();
        for instruction in &instructions {
            self.instruction(instruction)?;
        }
        Ok(())
    }

    /// Appends the statement one instruction became, if it became one.
    fn instruction(&mut self, instruction: &SsaInstruction) -> Result<(), StopReason> {
        poll(self.budget, Some(instruction.bci()))?;
        let at = instruction.bci();
        // An instruction a verified concatenation chain or a verified construction site owns
        // produces no statement of its own: the text it would have written is written *inside* the
        // expression that shape became, and skipping it here is exactly what keeps an operand from
        // being evaluated twice (P3 2.2/2.3).
        if self.chains.owns(at) || self.sites.owns(at) {
            return Ok(());
        }
        let write = instruction
            .writes()
            .iter()
            .find_map(|(slot, value)| match slot {
                Slot::Local(slot) => Some((*slot, *value)),
                Slot::Stack(_) => None,
            });
        match self.operations.get(at) {
            Some(Operation::Store { .. }) => {
                let Some((slot, _written)) = write else {
                    return self.fallback(
                        vec![at],
                        &format!("the store at BCI {at} writes no local slot this run names"),
                        at,
                    );
                };
                let (variable, target) = match self.write_target(slot, at, "store") {
                    Ok(target) => target,
                    Err(reason) => {
                        return self.fallback(vec![at], &reason, at);
                    }
                };
                let Some((_, stored)) = stack_operands(instruction).last().copied() else {
                    return self.fallback(
                        vec![at],
                        &format!("the store at BCI {at} reads no value to store"),
                        at,
                    );
                };
                let value = match self.render_value(stored, at, 0) {
                    Ok(value) => value,
                    Err(reason) => {
                        let bcis = self.quoted_bcis(at);
                        return self.fallback(bcis, &reason, at);
                    }
                };
                self.write_statement(variable, target, stored, value, at)
            }
            Some(Operation::Invoke(target)) => {
                // The call an instance initializer makes on its own uninitialized `this` is the
                // first thing a constructor does and is written as `super(…)` or `this(…)` — which
                // of the two is `init@1`'s verdict, read from the frames' token and the class the
                // caller stated (P3 2.3).
                if let Some(target_kind) = self.prologues.at(at).map(|prologue| prologue.target) {
                    return self.constructor_call(at, instruction, target_kind);
                }
                // A prologue the rule could not spell is quoted — the instruction itself, with the
                // requirement it fell short of. Writing it as an ordinary call would put a member
                // named `<init>` into the text and, worse, an assignment to the receiver the SSA
                // says the call converted: neither is a program.
                if let Some(reason) = self
                    .prologues
                    .refused_at(at)
                    .map(|refusal| refusal.message().to_string())
                {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(bcis, &reason, at);
                }
                // A synthetic accessor's call site is a direct field access, or it is not: the
                // verdict is `accessor@1`'s, and it is taken here so that a *presented* accessor
                // becomes the field access the source had — and a refused one keeps the call it had,
                // with the reason recorded against its BCI (P3 2.2, A12).
                let verdict = accessor::verify(target, self.members, self.pool);
                match verdict {
                    accessor::Verdict::Ordinary => {}
                    accessor::Verdict::Refused { evidence, refusal } => {
                        self.accessors.push(AccessorRecord::of(
                            at,
                            &evidence,
                            None,
                            Some(&refusal),
                        ));
                    }
                    accessor::Verdict::Accessor { evidence, shape } => match shape.kind {
                        // A read accessor's value is written where the value is *consumed*: the
                        // store or the call that reads it renders `x.f`. Nothing consumes it here,
                        // so the invocation the bytecode made has no place in the body — unless
                        // something does consume it, and then this instruction writes nothing.
                        AccessorShape::FieldRead => {
                            if self.call_value_reaches_a_reader(instruction) {
                                return Ok(());
                            }
                            let refusal = Refusal::shape(
                                "jre_accessor_unconsumed",
                                "nothing in this method reads the field the accessor returns, so the invocation the call site makes has no place in the body".to_string(),
                            );
                            self.accessors.push(AccessorRecord::of(
                                at,
                                &evidence,
                                None,
                                Some(&refusal),
                            ));
                        }
                        AccessorShape::FieldWrite => {
                            return self.accessor_write(at, instruction, target, evidence, shape);
                        }
                    },
                }
                // A call whose value something *reads* is written where that value is read — by the
                // store, the return, the cast or the condition that renders it. Writing it here as
                // well would evaluate it twice, which is the invariant this layer keeps. What is
                // deliberately not a reader here is a **dynamic site**: a site that is refused still
                // has to leave the invocation that produced a value it captured in the body, and
                // that instruction is where it is written ([`Self::value_is_consumed`] answers that
                // question for the site's own arm).
                if write.is_none() && self.call_value_reaches_a_reader(instruction) {
                    // The statement is deferred to the reader that will write the value. Which
                    // invocation was deferred is remembered with the value it produced, so that a
                    // reader that cannot write after all still has the invocation quoted rather
                    // than lost (P3 2.3 §0).
                    for (slot, value) in instruction.writes() {
                        if matches!(slot, Slot::Stack(_)) {
                            self.deferred.push((*value, at));
                        }
                    }
                    return Ok(());
                }
                let call = match self.call_expr(at, instruction, target, at, 0) {
                    Ok(call) => call,
                    Err(reason) => {
                        let bcis = self.quoted_bcis(at);
                        return self.fallback(bcis, &reason, at);
                    }
                };                let Some((slot, written)) = write else {
                    // No local slot takes the result: the call is a statement of its own.
                    return self.push(Stmt::new(
                        StmtKind::Expr(call),
                        OriginSet::new(Origin::direct(at)),
                    ));
                };
                let (variable, target_name) = match self.write_target(slot, at, "call") {
                    Ok(target) => target,
                    Err(reason) => {
                        return self.fallback(vec![at], &reason, at);
                    }
                };
                // The invocation produces the value its own slot takes, so the value written and the
                // value whose evidence types it are one and the same.
                self.write_statement(variable, target_name, written, call, at)
            }
            Some(Operation::Return) => {
                let value = match stack_operands(instruction).last().copied() {
                    Some((_, value)) => match self.return_expr(value, at) {
                        Ok(value) => Some(value),
                        Err(reason) => {
                            let bcis = self.quoted_bcis(at);
                            return self.fallback(bcis, &reason, at);
                        }
                    },
                    None => None,
                };
                self.push(Stmt::new(
                    StmtKind::Return { value },
                    OriginSet::new(Origin::direct(at)),
                ))
            }
            Some(Operation::Increment { slot, amount }) => {
                // The increment writes an assignment and never a declaration: an `iinc` reads the
                // slot as well as writing it, so its text is the same whether the variable was
                // declared here or earlier.
                let (_, target_name) = match self.write_target(*slot, at, "increment") {
                    Ok(target) => target,
                    Err(reason) => {
                        return self.fallback(vec![at], &reason, at);
                    }
                };
                let (op, magnitude) = if *amount < 0 {
                    (BinaryOp::Subtract, -i64::from(*amount))
                } else {
                    (BinaryOp::Add, i64::from(*amount))
                };
                let value = Expr::direct(
                    ExprKind::Binary {
                        op,
                        left: Box::new(Expr::direct(ExprKind::Local(target_name.clone()), at)),
                        right: Box::new(Expr::direct(ExprKind::Integer(magnitude), at)),
                    },
                    at,
                );
                self.push(Stmt::new(
                    StmtKind::Assign {
                        name: target_name,
                        value,
                    },
                    OriginSet::new(Origin::direct(at)),
                ))
            }
            // Values whose text lands where they are consumed, the branches and switches the region
            // layer turned into `if`/`while`/`switch`, and the transfer the region's edge already
            // stands for: no statement of their own.
            Some(
                Operation::Push(_)
                | Operation::Load { .. }
                | Operation::Arithmetic { .. }
                | Operation::Comparison { .. }
                | Operation::Switch { .. }
                | Operation::Transfer,
            ) => Ok(()),            // A dynamic call site produces the instance it presents, so — like a push or a load —
            // its text lands where the value is consumed. What is *not* the same as a push is what
            // it means to drop it: creating the instance is an invocation the bytecode really makes,
            // so a site whose value nothing reads is quoted instead of being written off as a value
            // with no effect.
            Some(Operation::InvokeDynamic(site)) => {
                let value = instruction
                    .writes()
                    .iter()
                    .find_map(|(slot, value)| match slot {
                        Slot::Stack(_) => Some(*value),
                        Slot::Local(_) => None,
                    });
                // The instance this site produces is a value whose text lands where the value is
                // consumed — the store or the call that reads it renders it, and *that* rendering is
                // where the site is read, presented and recorded. So the only case this instruction
                // has anything of its own to say about is the one where nothing consumes it: the
                // invocation the site makes has no place in the body, and the site is quoted.
                let consumed = value.is_some_and(|value| self.value_is_consumed(value));
                if consumed {
                    return Ok(());
                }
                match self.lambda_expr(at, instruction, site, false) {
                    Ok(_) => Ok(()),
                    Err(reason) => self.fallback(vec![at], &reason, at),
                }
            }
            // A cast is presented only where a rule proved it is the erasure of the value it casts
            // (`bridge@1`, P3 2.2): every other cast is a check that can fail, and it is quoted.
            Some(Operation::CheckCast { .. }) if self.bridge_owns(at) => Ok(()),
            // A field instruction is a field access exactly where `field@1` proved which member it
            // names (P3 2.3). A claimed **write** is a statement of its own — `receiver.f = value`,
            // written where the instruction runs, which is what keeps a constructor's initializer
            // sequence in the order its bytes have it. A claimed **read** is a value: its text lands
            // where the value is consumed, so nothing is written here.
            Some(Operation::Field { .. }) => {
                let fields = self.fields;
                let Some((evidence, shape)) = fields.claim(at) else {
                    return self.fallback(
                        self.quoted_bcis(at),
                        &format!(
                            "the field access at BCI {at} is not one this run proved names the member its own receiver's type declares, and a field instruction is presented only where the member it names is proven"
                        ),
                        at,
                    );
                };
                if shape.writes() {
                    return self.field_write(at, evidence, shape);
                }
                Ok(())
            }
            // The dispatch-table read of an enum `switch` (P3 2.3): `enumswitch@1` claims the read,
            // and its text is written where the switch's selector is written.
            Some(Operation::ArrayLoad) => {
                if self.enums.owns(at) {
                    Ok(())
                } else {
                    self.fallback(
                        self.quoted_bcis(at),
                        &format!(
                            "the array read at BCI {at} is not the dispatch-table shape this run verified: an `int[]` read is presented only where a rule proved which table and which index it reads"
                        ),
                        at,
                    )
                }
            }
            // An allocation, a copy or an unproven cast belongs to no verified shape of this body:
            // each is a *stated* gap, quoted with its own BCI rather than presented from half a
            // proof.
            Some(
                Operation::Allocate { .. }
                | Operation::Duplicate
                | Operation::CheckCast { .. },
            ) => self.fallback(
                self.quoted_bcis(at),
                &format!(
                    "the instruction at BCI {at} belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds"
                ),
                at,
            ),
            // The monitor instructions and the bare `throw` of P3 2.4 are *shape* facts: a
            // `synchronized` block is the monitor's enter and its exits, and a handler's rethrow is
            // an `athrow` of the exception it stored. Where a rule of [`crate::guard`] proved that
            // shape, these instructions are written by the statement and never reach here; an
            // instruction of either kind that no rule claimed has no statement of its own, exactly
            // like every other unclaimed operation.
            Some(Operation::Monitor { .. }) | Some(Operation::Throw) => self.fallback(
                vec![at],
                &format!(
                    "the instruction at BCI {at} belongs to no guarded shape this run proved: a monitor or a bare `throw` is written only where a rule claimed the statement around it"
                ),
                at,
            ),
            Some(Operation::Other) | None => self.fallback(
                vec![at],
                &format!("the instruction at BCI {at} is not part of the provable subset"),
                at,
            ),
        }
    }

    /// The expression one `return` writes, typed by the member's own return descriptor.
    ///
    /// A method whose descriptor returns `Z` returns a **boolean** (P3-R5's reading, in the return
    /// position): the frames state one `int` shape for the four int-sized primitives, so only the
    /// signature says whether the `ireturn` of this body presents a boolean. A value with boolean
    /// evidence ([`Self::boolean_value`]) is written as one — the literal `0`/`1` becomes
    /// `false`/`true`, and every other proven value already prints what it is — while a value
    /// without it is refused: the `int` spelling this layer would otherwise write (`return 1;` in a
    /// `boolean` method) is text the member's own signature rejects, and a body that publishes it
    /// claims a Java method that does not compile. A member that returns anything else keeps the
    /// value exactly as it was rendered.
    fn return_expr(&mut self, value: ValueId, at: u32) -> Result<Expr, String> {
        if !self.returns_boolean {
            return self.render_value(value, at, 0);
        }
        if self.boolean_value(value, at) {
            return self.render_value(value, at, 0).map(boolean_spelling);
        }
        // A value this layer cannot present at all keeps its own, more specific refusal: the
        // boolean context is the *second* reason such a value is not written, and the first one is
        // the evidence the value's own rendering is missing.
        match self.render_value(value, at, 0) {
            Err(reason) => Err(reason),
            Ok(_) => Err(format!(
                "the value at BCI {at} is returned from a method whose own descriptor returns `Z`, and this layer has no evidence that the value is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the member's own signature rejects"
            )),
        }
    }

    /// Whether one value is **proven** boolean in a position that requires a boolean: a `Z` return,
    /// a branch test, or a store into a variable the run already knows holds a `boolean`.
    ///
    /// Three kinds of fact make up the proof, and each is a statement of the class file or of this
    /// body rather than a guess from the value's shape:
    ///
    /// * [`Self::boolean_evidence`] — the class's own descriptors: a `boolean` parameter's load, a
    ///   call whose callee descriptor returns `Z`, a claimed field read whose pool descriptor is `Z`;
    /// * [`Self::boolean_literal`] — the `0`/`1` literal, which is how a `boolean` is pushed, and
    ///   which only a context that *already* requires a boolean can read as one (`int x = 1;` and
    ///   `boolean c = true;` are the same bytes, and a fresh local states no type);
    /// * [`Self::boolean_local`] — a read of a local variable a write of this body declared
    ///   `boolean`, which is the item the plan calls "a local the body has proven boolean".
    ///
    /// Every other value — an arithmetic result, a comparison, a value a branch or a phi merged out
    /// of unproven parts, a `load` of a slot no write proved boolean — is *not* proven boolean, and
    /// a boolean context refuses it rather than guessing. This is not a type system: it reads the
    /// same descriptors ([`typed_arguments`] does) plus the declarations this very build wrote.
    fn boolean_value(&self, value: ValueId, at: u32) -> bool {
        self.boolean_proven(value, at) || self.boolean_literal(value)
    }

    /// Whether the layer presents one value as a boolean **by its own evidence**, outside any
    /// position that requires one: a descriptor fact, or a read of a variable the plan decided
    /// boolean ([`Self::boolean_local`]).
    ///
    /// This is the proof the `0`/`1` literal is deliberately not part of: the literal is how both a
    /// boolean and an `int` are pushed, so it is a boolean only where a position that already
    /// requires one reads it ([`Self::boolean_value`]), and an `int` everywhere else.
    fn boolean_proven(&self, value: ValueId, at: u32) -> bool {
        self.boolean_evidence(value) || self.boolean_local(value, at)
    }

    /// Why no Java comparison spells one branch's pair comparison, when the operands' own evidence
    /// says so.
    ///
    /// A pair comparison (`if_icmp*`, `if_acmp*`) reads two `int`s or two references, and the layer
    /// presents each operand the way that operand's own evidence spells it. One operand it proves
    /// boolean beside one it does not is therefore a comparison with no Java spelling — the text
    /// `flag() == 1` is refused by javac (`incomparable types: boolean and int`) — and ordering two
    /// booleans has none either (`bad operand types for binary operator '<'`), while equality between
    /// two of them does. Both are the same rule as a write that cannot be spelled as its variable's
    /// decided type: the structure terminates instead of publishing text the compiler rejects.
    ///
    /// The shape is the hand-built `Boundary` class in `tests/p3_boolean_contexts.rs` — a compiler
    /// folds `flag() == true` into `flag()`, so source never reaches it — and the change that fixed
    /// the operands' spelling recorded it as a boundary and handed its disposition to this rule.
    fn pair_comparison_refusal(
        &self,
        op: BinaryOp,
        operands: &[(Slot, ValueId)],
        at: u32,
    ) -> Option<String> {
        let left = self.boolean_proven(operands[0].1, at);
        let right = self.boolean_proven(operands[1].1, at);
        if left != right {
            return Some(format!(
                "the branch at BCI {at} compares a value this layer proves boolean with one it does not (`{}`), and no Java comparison spells that pair of operands: the text this layer would write is refused by javac (`incomparable types: boolean and int`)",
                op.spell()
            ));
        }
        (left && !matches!(op, BinaryOp::Equal | BinaryOp::NotEqual)).then(|| {
            format!(
                "the branch at BCI {at} orders two values this layer proves boolean with `{}`, which no Java source spells on `boolean` operands",
                op.spell()
            )
        })
    }

    /// Whether the class's own descriptors state that one value is a boolean, with no context and no
    /// local consulted: a `load` of a parameter slot whose descriptor is `Z` (P3-R5), the result of
    /// a call whose **callee's** descriptor returns `Z` ([`CallTarget`]'s own descriptor, the fact
    /// [`typed_arguments`] reads), or a field access a `field@1` verdict claimed whose pool
    /// descriptor is `Z` — the same kind of fact as the callee descriptor, read from the evidence
    /// the artifact spells the member with.
    ///
    /// Nothing is followed: a value a store wrote, a value merged out of two pushes, and a value
    /// whose descriptor sits one definition away all state nothing here.
    fn boolean_evidence(&self, value: ValueId) -> bool {
        boolean_proof(
            self.ssa,
            self.operations,
            self.parameter_types,
            self.fields,
            value,
        )
    }

    /// Whether one value is a `0`/`1` literal: the constant a `boolean` is pushed as, in the
    /// positions that already require a boolean.
    fn boolean_literal(&self, value: ValueId) -> bool {
        let Definition::Instruction { bci, .. } = self.ssa.value(value).def() else {
            return false;
        };
        matches!(
            self.operations.get(*bci),
            Some(Operation::Push(ConstantValue::Int(0 | 1)))
        )
    }

    /// Whether one value reads a local variable the plan decided boolean: a `load` of that
    /// variable, or the entry/phi its own slot holds at `at`.
    ///
    /// The proof is the plan's own decision for that variable — read from one place, whatever the
    /// order the body is walked in — and it is the item the plan calls "a local the run proved
    /// boolean". Following the value further (through the write that filled the variable, or
    /// through a phi's operands) is exactly what this does **not** do: a variable no evidence
    /// proves is not a boolean local, however boolean the shape of its flow looks.
    fn boolean_local(&self, value: ValueId, at: u32) -> bool {
        read_variable(self.ssa, self.operations, self.reuse, value, at)
            .is_some_and(|variable| self.decided_boolean(variable))
    }

    /// The statement one write of a value into a variable becomes.
    ///
    /// The type is never decided here: [`Self::declare`] answers with the plan's own decision for
    /// the variable, and this method only spells the value that decision requires. Three outcomes:
    ///
    /// * this write **carries** the variable's declaration: the plan's type, with a boolean value
    ///   spelled `true`/`false`;
    /// * an earlier write carried it, or the signature states it: an assignment, whose value is
    ///   checked against the same decision in both directions — a value the layer presents as a
    ///   boolean is not published into a variable the decision says holds something else
    ///   (`int local3; … local3 = <boolean value>;` is text the variable's own type rejects), and a
    ///   value it cannot prove boolean is not published into a variable the decision says holds a
    ///   `boolean` (`boolean local1; … local1 = 2;`, the same rejection read the other way);
    /// * the plan could not decide the variable's type: [`Self::declare`] has already refused the
    ///   structure this write belongs to, and nothing is written for it.
    ///
    /// `stored` is the value the writing instruction reads: a store writes the slot and reads the
    /// stack, and it is that value whose evidence the decision was taken from — so it is the value
    /// the assignment's compatibility check reads too. An instruction that produces the value it
    /// writes (an invocation the SSA already places in the slot) stores nothing it read, and names
    /// its own result there.
    fn write_statement(
        &mut self,
        variable: LocalVariable,
        name: String,
        stored: ValueId,
        value: Expr,
        at: u32,
    ) -> Result<(), StopReason> {
        match self.declare(variable, at)? {
            Declaration::Declared(ty) => {
                let value = if ty == Type::Boolean {
                    boolean_spelling(value)
                } else {
                    value
                };
                self.push(Stmt::new(
                    StmtKind::Declare {
                        ty,
                        name,
                        value: Some(value),
                    },
                    OriginSet::new(Origin::direct(at)),
                ))
            }
            // The declaration this write would have carried was refused, and the fallback that says
            // so is already recorded: the assignment after it is not written, because a name that
            // never got a declaration may not be assigned and a refused write may not be published
            // in a spelling the refusal contradicts.
            Declaration::Refused => Ok(()),
            Declaration::NotDue => self.assignment(variable, name, stored, value, at),
        }
    }

    /// The assignment one write becomes when its declaration is already written elsewhere.
    ///
    /// The value is checked against the variable's decided type before it is published, and the
    /// check reads the same one decision the declaration does — never a second guess. The two
    /// directions are one rule: the value must be spellable as the decided type. A `0`/`1` literal
    /// is spellable as either (it is how both are pushed, and it adapts to `true`/`false` where the
    /// decision is `boolean`), while a value only a descriptor or a decided-boolean local proves is
    /// a boolean and cannot be presented as anything else.
    fn assignment(
        &mut self,
        variable: LocalVariable,
        name: String,
        stored: ValueId,
        value: Expr,
        at: u32,
    ) -> Result<(), StopReason> {
        match self.decision(variable) {
            Some(Decided::Type(Type::Boolean)) => {
                if !self.boolean_value(stored, at) {
                    return self.fallback(
                        vec![at],
                        &format!(
                            "the value at BCI {at} is stored into `{name}`, which this run already stated holds a `boolean`, and this layer has no evidence that the value is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the variable's own type rejects"
                        ),
                        at,
                    );
                }
                let value = boolean_spelling(value);
                self.push(Stmt::new(
                    StmtKind::Assign { name, value },
                    OriginSet::new(Origin::direct(at)),
                ))
            }
            Some(Decided::Type(ty)) => {
                // A value the layer presents as a boolean is not spellable as `{ty}`. The `0`/`1`
                // literal is not one of them: it is the same bytes for both, and it keeps the
                // integer spelling the decision states.
                if self.boolean_proven(stored, at) {
                    return self.fallback(
                        vec![at],
                        &format!(
                            "the value at BCI {at} is stored into `{name}`, which this run decided holds `{}`, and this layer presents the value as a boolean (a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this run decided `boolean`): the `boolean` spelling this layer would write is text the variable's own type rejects",
                            ty.spell()
                        ),
                        at,
                    );
                }
                self.push(Stmt::new(
                    StmtKind::Assign { name, value },
                    OriginSet::new(Origin::direct(at)),
                ))
            }
            // The variable's own type could not be decided: the structure this write belongs to is
            // refused rather than written with a type nobody stated.
            Some(Decided::Unknown(reason)) => {
                let reason = reason.message(variable.slot(), at);
                self.fallback(vec![at], &reason, at)
            }
            // The plan reached no variable here, which its own use map cannot produce for a write:
            // the write keeps the assignment it has always been.
            None => self.push(Stmt::new(
                StmtKind::Assign { name, value },
                OriginSet::new(Origin::direct(at)),
            )),
        }
    }

    /// Whether a local variable has to be declared at this write, and with which type.
    ///
    /// The type is the plan's own decision for this variable ([`decide_types`]), taken before any
    /// statement of the body existed and read here — at the in-place declaration the write carries,
    /// exactly as the hoisted declaration reads it. Deciding it here instead would be the defect
    /// this rule exists to close: the plan runs before the body's statements, so the two declaration
    /// paths read one answer instead of two, and the answer does not depend on the order the regions
    /// are walked in.
    ///
    /// `Declared` means this write carries the declaration; `NotDue` means it does not (the
    /// variable is a parameter's, its declaration is already written, or it has no name to declare);
    /// `Refused` means no declaration can be written and the write's assignment may not follow.
    fn declare(&mut self, variable: LocalVariable, at: u32) -> Result<Declaration, StopReason> {
        if self.declared.contains(&variable) || variable.slot() < self.parameters {
            return Ok(Declaration::NotDue);
        }
        if self.names.text(variable).is_none() {
            // No name to declare: the assignment that follows states the same thing and is already
            // reported as a fallback of its own.
            return Ok(Declaration::NotDue);
        }
        // The decision, and the only one: a variable the plan could not type is refused at the write
        // that would have declared it — the fallback quotes this write's bytecode, and the
        // declaration it would have carried is not written either. A frame entry that states no type
        // and one that states a name this layer cannot spell as a Java type (`spell_reference`) are
        // the two facts that reach this, and both are stated on the variable by the plan.
        let refusal = match self.decision(variable) {
            Some(Decided::Unknown(reason)) => Some(reason.message(variable.slot(), at)),
            _ => None,
        };
        if let Some(reason) = refusal {
            self.fallback(vec![at], &reason, at)?;
            return Ok(Declaration::Refused);
        }
        let Some(Decided::Type(ty)) = self.decision(variable).cloned() else {
            // No decision at all: the plan's own use map holds no write for this variable, which a
            // write cannot produce. Nothing is declared here and the assignment keeps the text it has
            // always had rather than inventing a second answer.
            return Ok(Declaration::NotDue);
        };
        self.declared.insert(variable);
        Ok(Declaration::Declared(ty))
    }

    /// The variable one write of local `slot` at BCI `at` fills, with the text its name is written
    /// as.
    ///
    /// `what` names the shape doing the writing, so that a refusal reads as the shape the caller
    /// refused. Two things stop it, and neither is a name to guess: a slot this run has no name for,
    /// and — P3 3.4 — a slot whose two variables this run cannot place the write in, which happens
    /// only where the evidence itself did not support splitting the slot in the first place.
    fn write_target(
        &self,
        slot: u16,
        at: u32,
        what: &str,
    ) -> Result<(LocalVariable, String), String> {
        let Some(variable) = self.reuse.variable_at(slot, at) else {
            return Err(format!(
                "the {what} at BCI {at} writes local {slot}, and the two variables the debug table states over that slot do not place this write in either"
            ));
        };
        match self.names.text(variable) {
            Some(name) => Ok((variable, name.to_string())),
            None => Err(format!(
                "the {what} at BCI {at} writes local {slot}, which has no name"
            )),
        }
    }

    /// Whether writing the **name** of local `slot` at the use BCI `at` denotes the value `denotes`.
    ///
    /// A local's name is not a name for a value: it is a name for whatever the slot holds where the
    /// name is read. The two agree exactly where the value in use at `at` *is* `denotes` — the last
    /// write to the slot before `at`, or the block's own entry state where nothing in the block wrote
    /// it. The smallest disagreement is a post-increment, `iload_0; iinc 0,1; ireturn`: the load
    /// reads the slot, the increment writes it, and the return reads the *loaded* value — so `local0`
    /// at the return denotes the incremented value, and naming the slot there is the opposite program.
    ///
    /// `at` is the **use** point — the BCI of the instruction that reads the value — and never the
    /// definition the value came from: what a slot holds is a statement about the reader's own point
    /// of the program, which is why a load's value can stop being the slot's value after it was read.
    fn slot_name_denotes_the_same_value(&self, slot: u16, denotes: ValueId, at: u32) -> bool {
        let Some(block) = self
            .block_of
            .get(&at)
            .and_then(|block| self.ssa.block(block))
        else {
            // No block of this run holds the use: nothing states what the slot holds there, and a
            // name written on no evidence states the wrong value half the time.
            return false;
        };
        let mut in_use = block
            .entry()
            .iter()
            .find_map(|(candidate, value)| match candidate {
                Slot::Local(candidate) if *candidate == slot => Some(*value),
                _ => None,
            });
        for instruction in block.instructions() {
            if instruction.bci() >= at {
                break;
            }
            for (written, value) in instruction.writes() {
                if matches!(written, Slot::Local(written) if *written == slot) {
                    in_use = Some(*value);
                }
            }
        }
        in_use == Some(denotes)
    }

    /// Renders one SSA value as an expression, for a use at BCI `at`.
    ///
    /// `at` is the position at which the text produced here is **evaluated**: the instruction whose
    /// statement the expression is written into, or the consumer a nested expression is rendered
    /// for. It is never the definition the value came from, and keeping the two apart is what P3-R8
    /// is about — a value the bytecode computed at BCI 2 can be written into a statement that runs
    /// after a write at BCI 3, so every check that asks what a slot *holds*, and every refusal that
    /// quotes a position, has to be taken at `at` rather than at the producer's own BCI. Which is
    /// why a recursive call passes the context it was given and not the BCI of the instruction it is
    /// descending through.
    ///
    /// The nodes themselves keep the BCIs they were produced at as their own anchors
    /// (`Expr::direct`, `OriginSet` and the derived chain): origin is about provenance, `at` is about
    /// evaluation, and a refusal is the only thing that moves between the two.
    fn render_value(&mut self, value: ValueId, at: u32, depth: usize) -> Result<Expr, String> {
        if depth > MAX_VALUE_DEPTH {
            return Err(format!(
                "the value at BCI {at} nests deeper than this layer renders"
            ));
        }
        match self.ssa.value(value).def() {
            Definition::Entry { block, slot } | Definition::Phi { block, slot } => match slot {
                Slot::Local(slot) => {
                    // The value is a slot's own, and a slot's name is a name for it only where the
                    // slot still holds it at the use (P3 1.3d). Where the body wrote the slot in
                    // between, the name denotes the newer value, so this value is refused rather
                    // than spelled wrongly.
                    if !self.slot_name_denotes_the_same_value(*slot, value, at) {
                        return Err(format!(
                            "the value at BCI {at} is what local {slot} holds at BCI {}, and the body writes the slot again before BCI {at}: the slot's name would denote the value written in between, not this one",
                            block.bci()
                        ));
                    }
                    // Which of the slot's variables that name is (P3 3.4): a reused slot holds one
                    // variable in one arm and another in the other, and the name written here is the
                    // name of the variable whose range covers this use.
                    match self.reuse.variable_at(*slot, at) {
                        Some(variable) => match self.names.text(variable) {
                            Some(name) => Ok(Expr::direct(ExprKind::Local(name.to_string()), at)),
                            None => Err(format!("local {slot} has no name to write")),
                        },
                        None => Err(format!(
                            "the value at BCI {at} is what local {slot} holds, and the two variables the debug table states over that slot do not place this use in either"
                        )),
                    }
                }
                Slot::Stack(stack) => Err(format!(
                    "the value at BCI {at} is the entry state of stack depth {stack}, which no instruction produced"
                )),
            },
            Definition::Instruction { bci, .. } => {
                let bci = *bci;
                // The instance a verified construction site builds: the value a store, a call or a
                // `return` reads *is* the `new` expression, written here and nowhere else (P3 2.3).
                let sites = self.sites;
                if let Some(site) = sites.site_of(bci) {
                    return self.new_expr(site, at, depth);
                }
                let Some(operation) = self.operations.get(bci) else {
                    return Err(format!(
                        "the value at BCI {at} comes from BCI {bci}, whose operation this run did not decode"
                    ));
                };
                match operation {
                    Operation::Push(constant) => Ok(Expr::direct(literal(constant), bci)),
                    // A load yields the value its slot held *where the load ran*, and that value is
                    // what a reader of it means — not the slot. Writing the slot's name at the use
                    // is the same expression only while the slot still holds it (P3 1.3d); where
                    // the body wrote the slot in between, the name would read the newer value and
                    // the text would state the opposite program, so the read is refused instead.
                    Operation::Load { slot } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the load at BCI {bci}"));
                        };
                        let Some(read) = local_read(instruction, *slot) else {
                            return Err(format!(
                                "the value at BCI {at} comes from the load at BCI {bci}, whose read of local {slot} this run does not state"
                            ));
                        };
                        if !self.slot_name_denotes_the_same_value(*slot, read, at) {
                            return Err(format!(
                                "the value at BCI {at} is the value local {slot} held at BCI {bci}, and the slot does not hold it at BCI {at}: the slot's name would read the value the body wrote in between"
                            ));
                        }
                        // The load is a read of the slot, so the variable it names is the one whose
                        // record covers the load's own BCI (P3 3.4).
                        match self.reuse.variable_at(*slot, bci) {
                            Some(variable) => match self.names.text(variable) {
                                Some(name) => {
                                    Ok(Expr::direct(ExprKind::Local(name.to_string()), bci))
                                }
                                None => Err(format!("local {slot} has no name to write")),
                            },
                            None => Err(format!(
                                "the load at BCI {bci} reads local {slot}, and the two variables the debug table states over that slot do not place this read in either"
                            )),
                        }
                    }
                    Operation::Arithmetic { op } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!(
                                "no names record for the instruction at BCI {bci}"
                            ));
                        };
                        let operands = stack_operands(instruction);
                        if operands.len() != 2 {
                            return Err(format!(
                                "the arithmetic at BCI {bci} reads {} values, not two",
                                operands.len()
                            ));
                        }
                        // Both operands are read by the sum, so both are checked where the sum is
                        // evaluated: a nested arithmetic does not move the use point to itself
                        // (P3-R8 — the value the operand names is on the stack when the sum runs,
                        // whatever the slot holds by then).
                        let left = self.render_value(operands[0].1, at, depth + 1)?;
                        let right = self.render_value(operands[1].1, at, depth + 1)?;
                        Ok(Expr::direct(
                            ExprKind::Binary {
                                op: arithmetic_op(*op),
                                left: Box::new(left),
                                right: Box::new(right),
                            },
                            bci,
                        ))
                    }
                    Operation::Invoke(target) => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the call at BCI {bci}"));
                        };
                        // The `toString` a verified concatenation chain ends in *is* the
                        // concatenation: the expression is written here, where its value is read,
                        // and every original BCI of the chain stays in the table as an anchor.
                        if let Some(chain) = self.chains.value_at(bci) {
                            return self.concat_expr(chain, at, depth);
                        }
                        // The call is written where its value is consumed, so its receiver and its
                        // arguments are evaluated *there* and are checked at `at` — not at the
                        // call's own BCI, which nothing in the text rewinds to (P3-R8's call half).
                        self.invoke_expr(bci, instruction, target, at, depth)
                    }
                    Operation::CheckCast { .. } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the cast at BCI {bci}"));
                        };
                        if !self.bridge_owns(bci) {
                            return Err(format!(
                                "the cast at BCI {bci} is not one this run proved to be the erasure of the value it casts"
                            ));
                        }
                        let Some((_, value)) = stack_operands(instruction).first().copied() else {
                            return Err(format!(
                                "the cast at BCI {bci} reads no value this run states"
                            ));
                        };
                        // The cast is dropped *and* kept: the value is written where the forwarded
                        // invocation wrote it, and the cast's own BCI stays in the segment table as
                        // a derived anchor of the node that presents it. The value it drops is read
                        // where the cast's own value is used, so it is checked at `at`.
                        let expr = self.render_value(value, at, depth + 1)?;
                        Ok(expr.derived_from(bci))
                    }
                    // A dynamic call site is a *value* whose shape this layer decides from the
                    // class's own bootstrap table: a verified `LambdaMetafactory` site becomes a
                    // lambda or a method reference, and every other site is refused with the reason
                    // recorded against it (A04). Nothing here falls back to "it looks like a
                    // lambda": the shape comes from the bootstrap, never from the opcode.
                    Operation::InvokeDynamic(site) => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!(
                                "no names record for the dynamic site at BCI {bci}"
                            ));
                        };
                        // A value is being rendered *because* something consumes it.
                        self.lambda_expr(bci, instruction, site, true)
                    }
                    // A field read `field@1` proved is `receiver.f` — or `Type.f` for a static field,
                    // whose receiver is the owner type itself. The member's owner and name are the
                    // instruction's own pool facts; nothing is spelled from a guess (P3 2.3).
                    Operation::Field { .. } => {
                        let fields = self.fields;
                        let Some((evidence, shape)) = fields.claim(bci) else {
                            return Err(format!(
                                "the value at BCI {at} comes from the field access at BCI {bci}, which this run did not prove names the member its receiver's type declares"
                            ));
                        };
                        let receiver = match shape.receiver {
                            // The receiver the read dereferences is evaluated where the read's own
                            // value is used: a chain of reads is one expression, and its inner
                            // reads answer to the position its value is consumed at (P3-R8).
                            Some(value) => self.render_value(value, at, depth + 1)?,
                            None => {
                                // A static read's receiver is the owner type itself, and the owner
                                // is the pool's own name: one this layer cannot spell as a Java type
                                // has no expression to be, and is refused where it would be written.
                                let owner = spell_reference(&evidence.owner).ok_or_else(|| {
                                    format!(
                                        "the field read at BCI {bci} names the owner `{}`, which this layer cannot spell as a Java type",
                                        evidence.owner
                                    )
                                })?;
                                Expr::direct(ExprKind::Path(owner), bci)
                            }
                        };
                        Ok(Expr::new(
                            ExprKind::Field {
                                receiver: Box::new(receiver),
                                name: evidence.name.clone(),
                            },
                            OriginSet::new(Origin::direct(bci)),
                        ))
                    }
                    // The dispatch-table read of an enum `switch`: the table, indexed by the call
                    // the switch reads its case index out of (P3 2.3). Both operands keep their own
                    // anchors, and the read carries the table's and the call's BCIs as derived ones.
                    Operation::ArrayLoad => {
                        let enums = self.enums;
                        let Some((table, index)) = enums.claim(bci) else {
                            return Err(format!(
                                "the value at BCI {at} comes from the array read at BCI {bci}, which this run did not prove is a dispatch-table read"
                            ));
                        };
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the array read at BCI {bci}"));
                        };
                        let operands = stack_operands(instruction);
                        if operands.len() != 2 {
                            return Err(format!(
                                "the array read at BCI {bci} reads {} value(s), not the table and the index",
                                operands.len()
                            ));
                        }
                        // The table and the selector are the values the read indexes with, and the
                        // read's value is used at `at`: both are checked there.
                        let array = self.render_value(operands[0].1, at, depth + 1)?;
                        let selector = self.render_value(operands[1].1, at, depth + 1)?;
                        let origin = OriginSet::new(Origin::direct(bci))
                            .plus_derived(Origin::derived(table.bci))
                            .plus_derived(Origin::derived(index.bci));
                        Ok(Expr::new(
                            ExprKind::Index {
                                array: Box::new(array),
                                index: Box::new(selector),
                            },
                            origin,
                        ))
                    }
                    other => Err(format!(
                        "the value at BCI {at} comes from an {other:?} at BCI {bci}, which produces no expression this subset writes"
                    )),
                }
            }
            Definition::Caught { bci, .. } => Err(format!(
                "the value at BCI {at} is the exception reference of the throw site at BCI {bci}"
            )),
        }
    }

    /// Renders one invocation, with its receiver and its arguments.
    ///
    /// `bci` is the invocation's own instruction, which is the node's anchor and the BCI the pool
    /// is read through; `at` is the position at which the rendered call is **evaluated**. They are
    /// the same for the statement a call writes of its own ([`Self::call_statement`]), and they
    /// differ where the call is written where its *value* is consumed: a nested `tick(x)` inside
    /// `tick(x) + ++x` reads its argument where the sum runs, after the increment (P3-R8).
    ///
    /// `depth` is the value-nesting depth the call is rendered at. It is passed **through** the
    /// call rather than restarted at it: a receiver or an argument is one more level of the same
    /// expression, and a recursion that does not count those levels is a native-stack recursion
    /// this layer has no bound on (the abort this change fixes).
    fn call_expr(
        &mut self,
        bci: u32,
        instruction: &SsaInstruction,
        target: &CallTarget,
        at: u32,
        depth: usize,
    ) -> Result<Expr, String> {
        let operands = stack_operands(instruction);
        let (receiver, args) = match target.kind() {
            InvokeKind::Static => (None, operands.as_slice()),
            _ => {
                let (receiver, args) = operands
                    .split_first()
                    .ok_or_else(|| format!("the call at BCI {bci} reads no receiver"))?;
                // The receiver is a value the call reads; when the subset cannot render it (an
                // uninitialized `new`, an operation it does not model) the call falls back rather
                // than naming the owner type in its place.
                (
                    Some(Box::new(self.render_value(receiver.1, at, depth + 1)?)),
                    args,
                )
            }
        };
        let mut arguments = Vec::with_capacity(args.len());
        for (_, value) in args {
            arguments.push(self.render_value(*value, at, depth + 1)?);
        }
        let arguments = typed_arguments(target.descriptor(), arguments);
        Ok(Expr::direct(
            ExprKind::Call {
                receiver,
                name: target.name().to_string(),
                args: arguments,
            },
            bci,
        ))
    }

    /// Renders one invocation: a verified synthetic accessor's field access, or the call itself.
    ///
    /// A read accessor's call site is a field read of the instance the site passed, and the field is
    /// the one the accessor's own verified body named. The node carries **both** original BCIs: the
    /// call site's as its own anchor and the field access inside the accessor's body as a derived
    /// one — which is exactly what A12 asks the segment table to keep (P3 2.2).
    ///
    /// `at` is the position the rendered expression is evaluated at, passed through to the receiver
    /// for the same reason [`Self::call_expr`] takes it: the instance the accessor reads is read
    /// where the expression is, not where the call site was.
    fn invoke_expr(
        &mut self,
        bci: u32,
        instruction: &SsaInstruction,
        target: &CallTarget,
        at: u32,
        depth: usize,
    ) -> Result<Expr, String> {
        if let accessor::Verdict::Accessor { evidence, shape } =
            accessor::verify(target, self.members, self.pool)
            && shape.kind == AccessorShape::FieldRead
        {
            let receiver = stack_operands(instruction)
                .first()
                .copied()
                .map(|(_, value)| value);
            match receiver.map(|receiver| self.render_value(receiver, at, depth + 1)) {
                Some(Ok(receiver)) => {
                    let origin = OriginSet::new(Origin::direct(bci))
                        .plus_derived(Origin::derived(shape.field_bci).in_method(&shape.method));
                    self.accessors
                        .push(AccessorRecord::of(bci, &evidence, Some(&shape), None));
                    return Ok(Expr::new(
                        ExprKind::Field {
                            receiver: Box::new(receiver),
                            name: shape.name.clone(),
                        },
                        origin,
                    ));
                }
                other => {
                    let reason = match other {
                        Some(Err(reason)) => reason,
                        _ => "the call reads no instance this run states".to_string(),
                    };
                    let refusal = Refusal::shape(
                        "jre_accessor_arguments",
                        format!(
                            "the instance the accessor call at BCI {bci} reads produces no expression this subset writes: {reason}"
                        ),
                    );
                    self.accessors
                        .push(AccessorRecord::of(bci, &evidence, None, Some(&refusal)));
                }
            }
        }
        self.call_expr(bci, instruction, target, at, depth)
    }

    /// Presents one verified write accessor's call site as the assignment it performs.
    ///
    /// The two values the call reads are rendered where the call site is, and a value this subset
    /// cannot write makes the call site a *refused* one — recorded — rather than a guessed
    /// assignment: the call it had is written instead.
    fn accessor_write(
        &mut self,
        at: u32,
        instruction: &SsaInstruction,
        target: &CallTarget,
        evidence: accessor::Evidence,
        shape: accessor::Shape,
    ) -> Result<(), StopReason> {
        let rendered = match stack_operands(instruction).as_slice() {
            [(_, receiver), (_, value)] => Some((*receiver, *value)),
            _ => None,
        }
        .map(|(receiver, value)| {
            (
                self.render_value(receiver, at, 0),
                self.render_value(value, at, 0),
            )
        });
        match rendered {
            Some((Ok(receiver), Ok(value))) => {
                let origin = OriginSet::new(Origin::direct(at))
                    .plus_derived(Origin::derived(shape.field_bci).in_method(&shape.method));
                self.accessors
                    .push(AccessorRecord::of(at, &evidence, Some(&shape), None));
                self.push(Stmt::new(
                    StmtKind::FieldAssign {
                        receiver,
                        name: shape.name.clone(),
                        value,
                    },
                    origin,
                ))
            }
            other => {
                let reason = match other {
                    Some((Err(reason), _)) | Some((_, Err(reason))) => reason,
                    _ => "the call does not read exactly the instance and the value it writes"
                        .to_string(),
                };
                let refusal = Refusal::shape(
                    "jre_accessor_arguments",
                    format!("the write accessor call at BCI {at} was not presented: {reason}"),
                );
                self.accessors
                    .push(AccessorRecord::of(at, &evidence, None, Some(&refusal)));
                self.call_statement(at, instruction, target)
            }
        }
    }

    /// The statement an invocation becomes when no rule presented it as a field access.
    ///
    /// The call writes a statement of its own, so it is evaluated where the instruction is: the
    /// evaluation context passed to [`Self::call_expr`] is the instruction's own BCI.
    fn call_statement(
        &mut self,
        at: u32,
        instruction: &SsaInstruction,
        target: &CallTarget,
    ) -> Result<(), StopReason> {
        let call = match self.call_expr(at, instruction, target, at, 0) {
            Ok(call) => call,
            Err(reason) => return self.fallback(vec![at], &reason, at),
        };
        self.push(Stmt::new(
            StmtKind::Expr(call),
            OriginSet::new(Origin::direct(at)),
        ))
    }

    /// Whether the `bridge@1` rule proved one instruction to be the erasure of the value it casts.
    fn bridge_owns(&self, bci: u32) -> bool {
        self.bridge.is_some_and(|plan| plan.owns(bci))
    }

    /// Renders one verified construction site as the `new` expression it stands for (P3 2.3).
    ///
    /// The arguments are written in the order the constructor call reads them — each an expression of
    /// its own, anchored where *it* was produced — and every BCI the site owns stays in the segment
    /// table as an anchor: one construction reaches several original instructions, and the table says
    /// so rather than keeping one of them.
    ///
    /// The arguments are read by the construction, which is written where the value at `at` is used:
    /// they are checked at `at`, not at the constructor's own BCI (`at` and `site.constructor` differ
    /// only where the `new` expression is evaluated later than the call that initialises it).
    fn new_expr(&mut self, site: &init::Site, at: u32, depth: usize) -> Result<Expr, String> {
        let Some(instruction) = self.instructions.get(&site.constructor).copied() else {
            return Err(format!(
                "no names record for the constructor call at BCI {} of the construction the value at BCI {at} comes from",
                site.constructor
            ));
        };
        let mut args = Vec::new();
        for (_, value) in stack_operands(instruction).iter().skip(1) {
            args.push(self.render_value(*value, at, depth + 1)?);
        }
        // The constructor's **own** descriptor types the arguments, exactly as a call's does: a
        // construction site writes the call the class file holds, and `new Res(arg0, 0)` for a
        // `Res(String, boolean)` constructor is a call the member's own signature refuses to compile.
        let args = match self.invoke_descriptor(site.constructor) {
            Some(descriptor) => typed_arguments(&descriptor, args),
            None => args,
        };
        let origin = site
            .owned
            .iter()
            .filter(|bci| **bci != site.constructor)
            .fold(
                OriginSet::new(Origin::direct(site.constructor)),
                |set, bci| set.plus_derived(Origin::derived(*bci)),
            );
        Ok(Expr::new(
            ExprKind::New {
                ty: spell_reference(&site.class).ok_or_else(|| {
                    format!(
                        "the construction at BCI {} names the class `{}`, which this layer cannot spell as a Java type",
                        site.constructor, site.class
                    )
                })?,
                args,
            },
            origin,
        ))
    }

    /// Writes the call an instance initializer makes on its own uninitialized `this` (P3 2.3).
    ///
    /// The receiver is deliberately not rendered: it *is* what `super` and `this` name, and the value
    /// it holds is the frames' own `UninitializedThis` token rather than any expression. The arguments
    /// are written where the call is, in the order it reads them.
    fn constructor_call(
        &mut self,
        at: u32,
        instruction: &SsaInstruction,
        target: ConstructorTarget,
    ) -> Result<(), StopReason> {
        let mut args = Vec::new();
        for (_, value) in stack_operands(instruction).iter().skip(1) {
            match self.render_value(*value, at, 0) {
                Ok(arg) => args.push(arg),
                Err(reason) => {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(bcis, &reason, at);
                }
            }
        }
        // `super(…)`/`this(…)` is an invocation like any other: the constructor it names states the
        // parameter types its arguments are written under.
        let args = match self.invoke_descriptor(at) {
            Some(descriptor) => typed_arguments(&descriptor, args),
            None => args,
        };
        self.push(Stmt::new(
            StmtKind::ConstructorCall { target, args },
            OriginSet::new(Origin::direct(at)),
        ))
    }

    /// The descriptor of the invocation one BCI holds, as the pool decoded it states it.
    ///
    /// `None` when the instruction is not an invocation of this decode: a site that names no
    /// `invoke*` states no descriptor, and a caller then leaves its arguments as they were rendered
    /// rather than guessing a signature (P3-R5's argument side).
    fn invoke_descriptor(&self, bci: u32) -> Option<String> {
        match self.operations.get(bci) {
            Some(Operation::Invoke(target)) => Some(target.descriptor().to_string()),
            _ => None,
        }
    }

    /// Writes one verified field write as the assignment it performs (P3 2.3).
    ///
    /// The receiver is the instance the instruction read — or, for a static field, the owner type,
    /// which is how a static write is spelled. Nothing is moved: the statement is written where the
    /// instruction runs, which is what keeps a constructor's initializer sequence in the order its
    /// own bytes have it.
    fn field_write(
        &mut self,
        at: u32,
        evidence: &field::Evidence,
        shape: &field::Shape,
    ) -> Result<(), StopReason> {
        let receiver = match shape.receiver {
            Some(value) => match self.render_value(value, at, 0) {
                Ok(receiver) => receiver,
                Err(reason) => {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(bcis, &reason, at);
                }
            },
            None => match spell_reference(&evidence.owner) {
                Some(owner) => Expr::direct(ExprKind::Path(owner), at),
                // A static write's receiver is the owner type itself: a name this layer cannot
                // spell as a Java type has no receiver to be, and the write is quoted.
                None => {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(
                        bcis,
                        &format!(
                            "the field write at BCI {at} names the owner `{}`, which this layer cannot spell as a Java type",
                            evidence.owner
                        ),
                        at,
                    );
                }
            },
        };
        let Some(value) = shape.value else {
            return self.fallback(
                self.quoted_bcis(at),
                &format!("the field write at BCI {at} reads no value to store"),
                at,
            );
        };
        let value = match self.render_value(value, at, 0) {
            Ok(value) => value,
            Err(reason) => {
                let bcis = self.quoted_bcis(at);
                return self.fallback(bcis, &reason, at);
            }
        };
        self.push(Stmt::new(
            StmtKind::FieldAssign {
                receiver,
                name: evidence.name.clone(),
                value,
            },
            OriginSet::new(Origin::direct(at)),
        ))
    }

    /// Whether the value a call produced is read by an instruction this build **writes it into**.
    ///
    /// This is the question the call arm asks before it writes a statement of its own, and it is
    /// deliberately narrower than [`Self::value_is_consumed`]: a **dynamic site** is not a reader
    /// here, because a site that is refused writes the site's own BCIs and not the invocation that
    /// produced the value it captured — so the instruction that produced that value is the only
    /// place left where the invocation can be written.
    ///
    /// The reader must be one this build **writes the value into**, which is what
    /// [`Self::renders_the_value_it_reads`] decides. Treating an instruction that is quoted instead
    /// of presented as a reader loses the invocation altogether — the call writes nothing because
    /// "something reads it", and the reader writes nothing because it is quoted bytecode. That is
    /// the shape P3 2.3 §0 measured before this guard existed: an effect silently dropped, which is
    /// worse than one written twice, because nothing in the artifact says it happened.
    fn call_value_reaches_a_reader(&self, instruction: &SsaInstruction) -> bool {
        instruction
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .any(|(_, value)| {
                let value = *value;
                self.ssa.blocks().iter().any(|block| {
                    block.instructions().iter().any(|reader| {
                        reader.reads().iter().any(|(_, read)| *read == value)
                            && self.renders_the_value_it_reads(reader.bci())
                    })
                })
            })
    }

    /// Whether the instruction at one BCI writes the values it reads into the text this build
    /// produces.
    ///
    /// Every operation that is *presented* renders its operands as part of what it becomes — a
    /// store's initialiser, a call's receiver and arguments, a `return`'s value, a condition, a
    /// switch's selector, an arithmetic — and every operation that is *quoted* writes no value at
    /// all. Which of the two an instruction is, is a fact about what the rules of this build claim:
    /// a `checkcast` is rendered only where `bridge@1` proved it is an erasure, a field access only
    /// where the `field@1` rule claimed it, an array read only where `enumswitch@1` claimed it.
    /// Everything else is a stated gap, and a value whose only reader is a stated gap has no place
    /// in the body: the instruction that produced it must write it itself.
    fn renders_the_value_it_reads(&self, bci: u32) -> bool {
        match self.operations.get(bci) {
            Some(
                Operation::Store { .. }
                | Operation::Invoke(_)
                | Operation::Return
                | Operation::Comparison { .. }
                | Operation::Switch { .. }
                | Operation::Arithmetic { .. },
            ) => true,
            Some(Operation::CheckCast { .. }) => self.bridge_owns(bci),
            // A field access and an array read are readers exactly where their own rules claimed
            // them (P3 2.3): a claimed access renders the value it reads into its text, and one no
            // rule claimed is quoted and writes nothing.
            Some(Operation::Field { .. }) => self.fields.owns(bci),
            Some(Operation::ArrayLoad) => self.enums.owns(bci),
            _ => false,
        }
    }

    /// The BCIs to quote for one instruction this build could not write: the instruction itself, and
    /// every invocation whose own statement was deferred to a reader that did not end up writing the
    /// value it read (P3 2.3 §0).
    fn quoted_bcis(&self, at: u32) -> Vec<u32> {
        let mut bcis = vec![at];
        if let Some(instruction) = self.instructions.get(&at).copied() {
            for (_, value) in stack_operands(instruction) {
                self.deferred_producers(value, at, &mut bcis, 0);
            }
        }
        bcis
    }

    /// The BCIs of the instructions behind one value that no statement wrote, every one of them read
    /// by the instruction at `reader`.
    ///
    /// A call whose value reaches a reader writes no statement of its own
    /// ([`Self::call_value_reaches_a_reader`]), and the reader *usually* writes the value: that is
    /// the whole reason for deferring. When the reader turns out not to be able to — its own
    /// operands may still be bytecode this layer cannot present — the invocation has no place in the
    /// artifact unless a quote names it, which is what this walk collects: through producers this
    /// build did present, the deferred producers of the values they read. A producer that is a
    /// deferred invocation is quoted and the walk stops there: the invocation itself is the effect,
    /// and its operands' statements are their own instructions' business.
    ///
    /// A **load** belongs to the same class for the same reason: it writes no statement either, its
    /// text lands where its value is consumed, and where the slot no longer holds what it read at
    /// `reader` that text is refused (P3 1.3d) — so the read itself is what the quote has to name,
    /// and the read is named rather than dropped from the answer.
    ///
    /// A **claimed field read** belongs here too: it writes no statement of its own either — a
    /// claimed read is a value whose text lands where the value is consumed — so a reader that
    /// refuses must name it, or the answer drops an effect the bytecode really performs (running a
    /// static initializer, or the `NullPointerException` an instance read can throw). The walk names
    /// it and keeps descending, because the receiver's own lost producers are the read's producers
    /// as well — that is how `External.holder.value` is answered for down to the `getstatic` behind
    /// the `getfield` (P3-R9).
    ///
    /// What is deliberately *not* here: an `invokedynamic` site's linkage and an `enumswitch@1`
    /// dispatch-table read stay unnamed when their consumer refuses. Naming them would state a
    /// linkage this walk does not own, and no rule of this change extends the rule to them.
    ///
    /// Every judgement in the walk is taken at the position the walk **started** from: the renderer
    /// that refused judged the loads it could not write at the consumer's use point, so the quote
    /// has to name the same read the refusal was about — a recursion that re-narrowed the use point
    /// to each intermediate instruction would judge a load where the artifact does not evaluate it
    /// and stay silent about the read it actually refused (P3-R8, and P3-R1's `post` for the
    /// direct-operand case).
    fn deferred_producers(&self, value: ValueId, reader: u32, into: &mut Vec<u32>, depth: usize) {
        if depth > MAX_VALUE_DEPTH {
            return;
        }
        let Definition::Instruction { bci, .. } = self.ssa.value(value).def() else {
            return;
        };
        let bci = *bci;
        if let Some((_, deferred)) = self
            .deferred
            .iter()
            .find(|(deferred_value, _)| *deferred_value == value)
        {
            if !into.contains(deferred) {
                into.push(*deferred);
            }
            return;
        }
        if let Some(instruction) = self.instructions.get(&bci).copied() {
            if let Some(Operation::Load { slot }) = self.operations.get(bci)
                && let Some(read) = local_read(instruction, *slot)
                && !self.slot_name_denotes_the_same_value(*slot, read, reader)
            {
                if !into.contains(&bci) {
                    into.push(bci);
                }
                return;
            }
            // A claimed read is named like a deferred call — and, unlike one, the walk continues
            // through it, because what its own operands read is behind the same refusal.
            if self
                .fields
                .claim(bci)
                .is_some_and(|(_, shape)| !shape.writes())
                && !into.contains(&bci)
            {
                into.push(bci);
            }
            for (_, operand) in stack_operands(instruction) {
                // The operand of *this* instruction is read where the reader the walk started from
                // reads it: the walk descends through instructions, but the use point does not move
                // with it — the renderer's checks are taken at the consumer's position.
                self.deferred_producers(operand, reader, into, depth + 1);
            }
        }
    }

    /// Every BCI one region covers, in method order: the bytecode a quote for that region has to
    /// name.
    ///
    /// A region the walk refuses *after* its condition could not be read still covers its arms'
    /// blocks, and those instructions produce no statements of their own — so a quote that named only
    /// the branch would lose them exactly as a refused reader loses a call (P3 2.3 §0). The quote
    /// names every instruction the region would have presented.
    fn region_bcis(&self, region: &Region) -> Vec<u32> {
        let mut bcis: Vec<u32> = Vec::new();
        for block in region.blocks() {
            for bci in self.covered_bcis(block) {
                if !bcis.contains(&bci) {
                    bcis.push(bci);
                }
            }
        }
        bcis
    }

    /// The quote for one region whose test this build could not read: every BCI the region covers,
    /// and the invocations deferred to a reader inside it that did not end up writing them.
    fn region_quote(&self, region: &Region, test_bci: u32) -> Vec<u32> {
        let mut bcis = self.region_bcis(region);
        for extra in self.quoted_bcis(test_bci) {
            if !bcis.contains(&extra) {
                bcis.push(extra);
            }
        }
        bcis
    }

    /// Renders one verified concatenation chain as the parts its `+` expression is made of.
    ///
    /// The parts are written in the order the chain's `append` calls read them, and each one is an
    /// expression of its own — anchored where *it* was produced — so an operand that calls something
    /// calls it once, in the bytecode's order. The `toString` the chain ends in anchors the
    /// expression, and every BCI the chain owns is kept as a derived anchor of it: one concatenation
    /// reaches many original instructions, and the table says so rather than keeping one of them
    /// (P3 2.2, the source-map requirement).
    ///
    /// The parts are a **sequence** ([`ExprKind::Concat`]) and not a nested `Binary` fold, which is
    /// what makes one `append` one entry of a `Vec` rather than one more level of a `Box` chain: the
    /// construction, the printing, the cloning and the release of this node are bounded by the parts'
    /// own nesting (each part's expression is rendered under the value-nesting bound), never by the
    /// chain's length. The fold is what this method replaced — it built the tree the printer then had
    /// to walk level by level, so a chain long enough exhausted the process stack while every
    /// individual value was well inside `MAX_VALUE_DEPTH`.
    ///
    /// Each part keeps its own conversion, and the conversion is applied here, where the `append`'s
    /// parameter type and the value's own evidence meet:
    ///
    /// * every accepted overload's text in a string context is `String.valueOf`'s, which is the
    ///   value's own text — so nothing about the part changes except the empty string the printer
    ///   starts the chain from when the first part is not already a `String`;
    /// * `boolean` is the exception: `append(true)` is `iconst_1`, and only the descriptor's `Z` says
    ///   that this `1` is a boolean, so a part whose parameter is `boolean` is spelled as the boolean
    ///   it is (`boolean_spelling`) — the same reading the call-argument, `return` and store paths
    ///   make of a `Z` position. A `boolean` part whose value has **no** boolean evidence is refused
    ///   rather than spelled as the integer it would otherwise look like.
    ///
    /// `at` is the position the concatenation is evaluated at — the consumer that renders it, since
    /// the chain's own text lands there — and every piece is checked there for the reason
    /// [`Self::render_value`] takes `at` at all (P3-R8).
    ///
    /// `depth` is the value-nesting depth the chain is rendered at, passed through to every piece:
    /// a concatenation inside an argument is one more level of the same native recursion, and the
    /// bound has to see it.
    fn concat_expr(
        &mut self,
        chain: &concat::Chain,
        at: u32,
        depth: usize,
    ) -> Result<Expr, String> {
        let mut parts: Vec<ConcatPart> = Vec::with_capacity(chain.appends.len());
        for (append_bci, parameter) in &chain.appends {
            let Some(instruction) = self.instructions.get(append_bci).copied() else {
                return Err(format!(
                    "no names record for the `append` at BCI {append_bci}"
                ));
            };
            let operands = stack_operands(instruction);
            let Some((_, value)) = operands.last().copied() else {
                return Err(format!(
                    "the `append` at BCI {append_bci} appends no value this run states"
                ));
            };
            // The value's own expression, anchored where it was produced, presenting the `append`
            // that converts it: the part and the instruction are one, and the table says which.
            let rendered = self
                .render_value(value, at, depth + 1)?
                .derived_from(*append_bci);
            let mut part = ConcatPart::new(parameter.clone(), rendered);
            if part.parameter == Type::Boolean {
                if !(self.boolean_literal(value) || self.boolean_proven(value, at)) {
                    return Err(format!(
                        "the `append` at BCI {append_bci} takes `boolean`, and this layer has no evidence that the value it reads at BCI {at} is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the overload's own conversion rejects"
                    ));
                }
                part.value = boolean_spelling(part.value);
            }
            parts.push(part);
        }
        let origin = chain
            .owned
            .iter()
            .filter(|bci| **bci != chain.tail)
            .fold(OriginSet::new(Origin::direct(chain.tail)), |set, bci| {
                set.plus_derived(Origin::derived(*bci))
            });
        Ok(Expr::new(ExprKind::Concat { parts }, origin))
    }

    /// Renders one dynamic call site as a lambda or a method reference — or refuses it and records
    /// which link of the chain failed (P3 2.1, A04).
    ///
    /// Everything read here is the payload's own: the class's bootstrap table and pool decide *what
    /// the site is* ([`crate::lambda`]), the run's names decide the captured values and their BCIs,
    /// and the frames decide a capture's type. The record is pushed here, where the decision is
    /// made, so no site can be presented without a record and no record can describe text that was
    /// never written.
    fn lambda_expr(
        &mut self,
        bci: u32,
        instruction: &SsaInstruction,
        site: &DynamicSite,
        consumed: bool,
    ) -> Result<Expr, String> {
        let operands = stack_operands(instruction);
        let captures: Vec<(Option<u32>, Option<Type>)> = operands
            .iter()
            .map(|(_, value)| {
                (
                    self.value_bci(*value),
                    // A capture whose frame fact is a descriptor with no Java spelling is a value
                    // this layer cannot state a type for, and `lambda::plan` refuses a site with an
                    // unstated capture type under its own preconditions: the site is refused with
                    // its record and the capture's BCI, which is the answer a spelling failure gets
                    // here rather than a second refusal vocabulary.
                    value_type(self.ssa.value(*value).ty()).ok().flatten(),
                )
            })
            .collect();
        let verdict = lambda::plan(site, self.bootstrap, self.pool, &captures, &self.profile);
        let evidence = verdict.evidence;
        charge(self.budget, CountedBudgetDimension::IrItems, 0, Some(bci)).ok();
        // A refused site is recorded with the operands it reads and no text: a refusal states where
        // its evidence came from without pretending to have written it.
        let unrendered = || -> Vec<LambdaCapture> {
            captures
                .iter()
                .map(|(at, _)| LambdaCapture { bci: *at })
                .collect()
        };
        let plan = match verdict.outcome {
            Ok(plan) => plan,
            Err(refusal) => {
                let record =
                    Self::lambda_record(bci, site, &evidence, None, Some(&refusal), &unrendered());
                self.lambdas.push(record);
                return Err(refusal.message().to_string());
            }
        };
        if !consumed {
            // The shape is verified, but the instance it creates reaches no statement: creating it
            // is still an invocation the bytecode makes, so the site is quoted rather than written
            // off as a value with no effect (a `push` whose value nobody reads is dropped; this is
            // not a `push`).
            let refusal = Refusal::shape(
                "jre_lambda_unconsumed",
                "nothing in this method reads the instance the site creates, so the invocation the site makes has no place in the body".to_string(),
            );
            let record =
                Self::lambda_record(bci, site, &evidence, None, Some(&refusal), &unrendered());
            self.lambdas.push(record);
            return Err(refusal.message().to_string());
        }
        // Every captured value has to be replayable before any of its text is written: the value is
        // written where the shape reads it, and the instruction that produced it may also have been
        // written as a statement of its own. The rule states the requirement and this is the check
        // point, so a capture that would run again (or run later) makes the site a fallback.
        for index in 0..plan.captures {
            let (at, _) = captures[index];
            let (_, value) = operands[index];
            if let Some(reason) = self.unreplayable(value) {
                let refusal = Refusal::unmet(
                    &LAMBDA,
                    Precondition::Replayable,
                    format!(
                        "the value captured for the site's argument {index} (BCI {}) cannot be written where the shape reads it: {reason}",
                        at.map_or_else(|| "unknown".to_string(), |at| at.to_string())
                    ),
                );
                let record =
                    Self::lambda_record(bci, site, &evidence, None, Some(&refusal), &unrendered());
                self.lambdas.push(record);
                return Err(refusal.message().to_string());
            }
        }
        // The captured arguments, in the order the site reads them off the stack — which is the order
        // the implementation handle receives them in. Each expression carries its own anchor (the
        // BCI it was produced at), and the nodes whose text reproduces a capture present it as
        // derived evidence.
        let mut captures_rendered: Vec<Expr> = Vec::with_capacity(plan.captures);
        let mut captured: Vec<LambdaCapture> = Vec::with_capacity(plan.captures);
        for index in 0..plan.captures {
            let (at, _) = captures[index];
            let (_, value) = operands[index];
            let Ok(expr) = self.render_value(value, bci, 0) else {
                let refusal = Refusal::shape(
                    "jre_lambda_capture",
                    format!(
                        "the value captured for the site's argument {index} (BCI {}) produces no expression this subset writes",
                        at.map_or_else(|| "unknown".to_string(), |at| at.to_string())
                    ),
                );
                let record =
                    Self::lambda_record(bci, site, &evidence, None, Some(&refusal), &unrendered());
                self.lambdas.push(record);
                return Err(refusal.message().to_string());
            };
            captured.push(LambdaCapture { bci: at });
            captures_rendered.push(expr);
        }
        // The site's own anchor (with the pool entry that states it), and the same anchor carrying
        // the captures as evidence the text reproduces: one lambda expression reaches more than one
        // original BCI, and the table says so. The nodes *inside* the body — the receiver's type
        // name, the parameters — are anchored at the site alone: they do not reproduce a captured
        // value, and a node that claimed to would be evidence that is not true.
        let site_origin = OriginSet::new(Origin::direct(bci).with_cp(site.cp()));
        let origin = captured
            .iter()
            .fold(site_origin.clone(), |set, capture| match capture.bci {
                Some(at) => set.plus_derived(Origin::derived(at)),
                None => set,
            });
        let mut params: Vec<LambdaParam> = Vec::with_capacity(plan.params.len());
        let mut parameters_rendered: Vec<Expr> = Vec::with_capacity(plan.params.len());
        for (index, ty) in plan.params.iter().enumerate() {
            let name = self.param_name(index);
            parameters_rendered.push(Expr::new(
                ExprKind::Local(name.clone()),
                site_origin.clone(),
            ));
            params.push(LambdaParam {
                ty: ty.clone(),
                name,
            });
        }
        let expr = match plan.form {
            LambdaForm::MethodReference => {
                // The captures *are* what the handle's own receiver needs and nothing else, so the
                // site is a reference to the member itself: a type for a static or constructor one,
                // the bound receiver — the one capture — for an instance one. There are no arguments
                // to write: a `::` reference takes none.
                let qualifier = match (plan.reach, captures_rendered.first()) {
                    (Reach::Receiver, Some(receiver)) => receiver.clone(),
                    _ => Expr::new(
                        ExprKind::Path(implementation_owner(&plan, bci)?),
                        site_origin.clone(),
                    ),
                };
                let name = match plan.reach {
                    Reach::Constructor => "new".to_string(),
                    _ => plan.implementation.name().to_string(),
                };
                Expr::new(
                    ExprKind::MethodReference {
                        qualifier: Box::new(qualifier),
                        name,
                    },
                    origin.clone(),
                )
            }
            LambdaForm::Lambda => {
                let mut bound = captures_rendered;
                let parameter_count = parameters_rendered.len();
                bound.extend(parameters_rendered);
                let body = match plan.reach {
                    Reach::Constructor => Expr::new(
                        ExprKind::New {
                            ty: implementation_owner(&plan, bci)?,
                            args: bound,
                        },
                        origin.clone(),
                    ),
                    Reach::Static => Expr::new(
                        ExprKind::Call {
                            receiver: Some(Box::new(Expr::new(
                                ExprKind::Path(implementation_owner(&plan, bci)?),
                                site_origin.clone(),
                            ))),
                            name: plan.implementation.name().to_string(),
                            args: bound,
                        },
                        origin.clone(),
                    ),
                    Reach::Receiver => {
                        let mut operands_bound = bound.into_iter();
                        let receiver = operands_bound.next().ok_or_else(|| {
                            "a receiver-kind implementation with nothing bound to its receiver"
                                .to_string()
                        })?;
                        Expr::new(
                            ExprKind::Call {
                                receiver: Some(Box::new(receiver)),
                                name: plan.implementation.name().to_string(),
                                args: operands_bound.collect(),
                            },
                            origin.clone(),
                        )
                    }
                };
                debug_assert_eq!(params.len(), parameter_count);
                Expr::new(
                    ExprKind::Lambda {
                        params,
                        body: Box::new(body),
                    },
                    origin.clone(),
                )
            }
        };
        let record = Self::lambda_record(bci, site, &evidence, Some(plan.form), None, &captured);
        self.lambdas.push(record);
        Ok(expr)
    }

    /// The record of one dynamic site, as the report reads it back.
    fn lambda_record(
        bci: u32,
        site: &DynamicSite,
        evidence: &crate::lambda::Evidence,
        form: Option<LambdaForm>,
        refusal: Option<&Refusal>,
        captures: &[LambdaCapture],
    ) -> LambdaRecord {
        LambdaRecord {
            use_site: bci,
            site_cp: site.cp(),
            bootstrap_index: site.bootstrap_index(),
            bootstrap: evidence.bootstrap.clone(),
            bootstrap_arguments: evidence.bootstrap_arguments,
            sam_name: site.name().to_string(),
            sam_descriptor: site.descriptor().to_string(),
            sam_method_type: evidence.sam_method_type.clone(),
            instantiated_method_type: evidence.instantiated_method_type.clone(),
            implementation: evidence.implementation.clone(),
            captures: captures.to_vec(),
            form,
            refusal: refusal.map(|refusal| LambdaRefusal::of(refusal, bci)),
        }
    }

    /// Why one captured value cannot be written where the shape reads it, when it cannot.
    ///
    /// A capture's text is written into the artifact, and the instruction that produced it may also
    /// have been written as a statement of its own — a call whose result the site captures is a call
    /// statement *and* an argument. So the value is replayable only when re-reading its text is the
    /// same thing as having read it once: a literal, or a local the body writes exactly once.
    fn unreplayable(&self, value: ValueId) -> Option<String> {
        match self.ssa.value(value).def() {
            Definition::Instruction { bci, .. } => match self.operations.get(*bci) {
                Some(Operation::Push(_)) => None,
                Some(Operation::Load { slot }) => self.written_once(*slot),
                Some(operation) => Some(format!(
                    "it comes from an {operation:?} at BCI {bci}, whose text would run again (or run later) inside the shape"
                )),
                None => Some(format!(
                    "it comes from the instruction at BCI {bci}, which this run did not decode"
                )),
            },
            Definition::Entry {
                slot: Slot::Local(slot),
                ..
            }
            | Definition::Phi {
                slot: Slot::Local(slot),
                ..
            } => self.written_once(*slot),
            Definition::Entry {
                slot: Slot::Stack(depth),
                ..
            }
            | Definition::Phi {
                slot: Slot::Stack(depth),
                ..
            } => Some(format!(
                "it is the entry state of stack depth {depth}, which no instruction produced"
            )),
            Definition::Caught { bci, .. } => Some(format!(
                "it is the exception reference of the throw site at BCI {bci}"
            )),
        }
    }

    /// Why one local's text cannot be read again, when it cannot: the body writes it more than once.
    ///
    /// A local the body writes exactly once is *effectively final* in the source's own sense, and
    /// reading it again — which is what writing it inside a lambda body does — reads the same value.
    /// A local written twice might not: a later write could land between the site and the lambda's
    /// invocation, and the recovered text would then read a value the bytecode never captured.
    /// Refusing is the honest answer, because this layer has no liveness analysis that could prove
    /// the writes apart.
    fn written_once(&self, slot: u16) -> Option<String> {
        let mut writes = 0usize;
        for block in self.ssa.blocks() {
            for instruction in block.instructions() {
                if instruction
                    .writes()
                    .iter()
                    .any(|(written, _)| *written == Slot::Local(slot))
                {
                    writes += 1;
                }
            }
        }
        (writes > 1).then(|| {
            format!(
                "the local {slot} is written {writes} times in this body, so reading it again inside the shape would not read the value the site captured"
            )
        })
    }

    /// Whether one value reaches a statement — a read by an instruction this subset *renders* its
    /// operands for.
    ///
    /// A dynamic site's instance is a value like any other, except that dropping it would drop the
    /// invocation the site makes. The instructions that count are the ones whose rendering writes
    /// the values they read (a store, a call, a return, a branch, a switch, an arithmetic); a `pop`
    /// reads a value and writes nothing with it, which is exactly the case this question is about.
    fn value_is_consumed(&self, value: ValueId) -> bool {
        self.ssa.blocks().iter().any(|block| {
            block.instructions().iter().any(|instruction| {
                instruction.reads().iter().any(|(_, read)| *read == value)
                    && (self.renders_the_value_it_reads(instruction.bci())
                        // A dynamic site reads its captured values, but it writes the site's own
                        // BCIs rather than a rendering of the instruction that produced one of them
                        // — which is why it is not a reader in [`Self::call_value_reaches_a_reader`]
                        // and is one here, where the question is about the *site's* value.
                        || matches!(
                            self.operations.get(instruction.bci()),
                            Some(Operation::InvokeDynamic(_))
                        ))
            })
        })
    }

    /// The BCI the instruction that produced one value sits at, when an instruction produced it.
    fn value_bci(&self, value: ValueId) -> Option<u32> {
        match self.ssa.value(value).def() {
            Definition::Instruction { bci, .. } | Definition::Caught { bci, .. } => Some(*bci),
            Definition::Phi { block, .. } => Some(block.bci()),
            Definition::Entry { .. } => None,
        }
    }

    /// One parameter name for a lambda of this body: the first free `pN`.
    ///
    /// The class file names no lambda parameter, so the name is derived — and it has to differ from
    /// every name the body's own locals carry *and* from every parameter name another lambda of this
    /// body already took (JLS 6.4 forbids a lambda parameter that shadows either). Allocation follows
    /// the order the shapes are written, so the names are a function of the evidence alone.
    fn param_name(&mut self, index: usize) -> String {
        let mut name = self.names.free_name(&format!("p{index}"));
        while self.lambda_params.contains(&name) {
            name.push('_');
        }
        self.lambda_params.insert(name.clone());
        name
    }

    /// Appends the quoted bytecode of one region or instruction.
    fn fallback(&mut self, bcis: Vec<u32>, reason: &str, at: u32) -> Result<(), StopReason> {
        self.ragged = true;
        // Every BCI the quote's text names is an anchor of the quoted node — the region's own
        // instruction first, the rest presented — so a Mixed artifact's fallback is mapped as
        // completely as one of its structured regions: for each bytecode the quote accounts for,
        // the table answers which text covers it. The list is the same one the emitter prints, so
        // the anchors and the quoted line cannot disagree (P3 3.2, A12/A13).
        let origin = bcis
            .iter()
            .filter(|bci| **bci != at)
            .fold(OriginSet::new(Origin::direct(at)), |origin, bci| {
                origin.plus_derived(Origin::derived(*bci))
            });
        self.push(Stmt::new(
            StmtKind::Fallback {
                reason: reason.to_string(),
                bcis,
            },
            origin,
        ))
    }

    /// Bills and appends one statement.
    fn push(&mut self, stmt: Stmt) -> Result<(), StopReason> {
        let at = stmt.origin.primary().bci();
        poll(self.budget, Some(at))?;
        charge(self.budget, CountedBudgetDimension::IrItems, 1, Some(at))?;
        self.statements += 1;
        self.stmts.push(stmt);
        Ok(())
    }

    /// The instruction starts one canonical block covers.
    fn covered_bcis(&self, block: &CanonicalBlockId) -> Vec<u32> {
        self.canonical
            .blocks()
            .iter()
            .find(|candidate| candidate.id() == block)
            .map(|block| block.blocks().to_vec())
            .unwrap_or_default()
    }
}

/// The arguments of one call, typed by the **callee's own descriptor** (P3-R5's argument side).
///
/// A `boolean` parameter and an `int` one are one slot shape, and the value the bytecode pushes for
/// `true`/`false` is the `int`-shaped `1`/`0` (`iconst_1`/`iconst_0`): a call site cannot state which
/// of the two it is passing, and the descriptor of the member it calls is the only evidence that can.
/// So an argument that is a **literal** `0`/`1` — and only a literal: an expression that computes one
/// is a value whose type this layer would be guessing at — is written `false`/`true` where the
/// parameter is declared `boolean`.
///
/// Every other parameter type keeps the argument exactly as it was rendered. `int`, `long`, `float`,
/// `double`, `String` and reference types already spell what their descriptor states, and a
/// `byte`/`char`/`short` parameter legally takes an `int` **constant** (JLS 5.3 narrows a constant
/// expression of `int` type in a method-invocation context), so re-typing those would be writing a
/// different program than the bytecode holds.
///
/// The mapping is by position, so it is only applied where the descriptor and the arguments really
/// line up: a descriptor that does not parse, or one whose parameter count differs from the number of
/// arguments the call reads, leaves the arguments untouched rather than typing one of them from a
/// guess. This is the shared path every invocation of this layer goes through — `invokevirtual`,
/// `invokespecial`, `invokestatic` and `invokeinterface` alike, the constructor call of a `new` and
/// the `super(…)`/`this(…)` of an instance initializer — because "which argument is a `boolean`" is a
/// fact about the callee, not about the shape that writes the call.
///
/// One place is deliberately **not** this shape: the arguments a lambda's factory site binds are the
/// site's own captures, whose types the `invokedynamic` descriptor states for the SAM rather than for
/// the implementation the reference names, so nothing there is re-typed.
pub(crate) fn typed_arguments(descriptor: &str, arguments: Vec<Expr>) -> Vec<Expr> {
    let Some(parameters) = parameter_descriptors(descriptor) else {
        return arguments;
    };
    if parameters.len() != arguments.len() {
        return arguments;
    }
    arguments
        .into_iter()
        .zip(parameters)
        .map(|(argument, parameter)| {
            if parameter != "Z" {
                return argument;
            }
            let ExprKind::Integer(value) = argument.kind else {
                return argument;
            };
            let spelled = match value {
                0 => false,
                1 => true,
                _ => return argument,
            };
            Expr {
                kind: ExprKind::Boolean(spelled),
                origin: argument.origin,
            }
        })
        .collect()
}

/// The parameter type descriptors one method descriptor states, in order
/// (`(Ljava/lang/String;Z)V` → `["Ljava/lang/String;", "Z"]`), or `None` when it does not parse.
///
/// Only the parameter list is read: the return type is not part of how an argument is spelled, and a
/// descriptor whose `)` is missing states no parameters a caller could line arguments up with.
fn parameter_descriptors(descriptor: &str) -> Option<Vec<&str>> {
    let arguments = descriptor.strip_prefix('(')?.split_once(')')?.0;
    let bytes = arguments.as_bytes();
    let mut types = Vec::new();
    let mut at = 0usize;
    while at < bytes.len() {
        let start = at;
        while bytes.get(at) == Some(&b'[') {
            at += 1;
        }
        match bytes.get(at) {
            // A reference type runs to its `;`: its name may hold any character, so scanning for the
            // terminator is the only reading that cannot split one in two.
            Some(b'L') => at = arguments[at..].find(';')? + at + 1,
            // Every other field descriptor is one character (a primitive or an element type).
            Some(_) => at += 1,
            None => return None,
        }
        types.push(&arguments[start..at]);
    }
    Some(types)
}

/// Whether the class's own descriptors state that one value is a boolean.
///
/// The free-function form of [`Builder::boolean_evidence`], for the step that decides a **hoisted**
/// variable's type before a single statement is written: that declaration states its type from the
/// first write's stored value, and the two readings have to be one reading.
///
/// What it deliberately does not carry is the `0`/`1` literal (a fresh local has no boolean context,
/// so `int y = 0;` and `boolean c = true;` are the same bytes — see [`Builder::declare`]) and a local
/// this body declared boolean (that declaration is a statement, and this runs before there are any).
fn boolean_proof(
    ssa: &SsaTable,
    operations: &Operations,
    parameter_types: &BTreeMap<u16, Type>,
    fields: &field::Plan,
    value: ValueId,
) -> bool {
    if parameter_boolean(ssa, operations, parameter_types, value) {
        return true;
    }
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return false;
    };
    match operations.get(*bci) {
        Some(Operation::Invoke(target)) => returns_boolean(target.descriptor()),
        Some(Operation::Field { .. }) => fields
            .claim(*bci)
            .is_some_and(|(evidence, _)| evidence.descriptor == "Z"),
        _ => false,
    }
}

/// The value one writing instruction **reads** as the value it stores, when the two are different.
///
/// A `…; istore` writes the slot and reads the stack: the slot takes a value of the store's own, and
/// the value whose evidence states what the slot now holds is the one on the stack. An instruction
/// that writes a slot directly — an invocation whose result the SSA already places in the slot —
/// produces the value it writes, so it stores nothing it read, and `None` says so rather than naming
/// an argument of the call.
fn store_operand(operations: &Operations, instruction: &SsaInstruction) -> Option<ValueId> {
    match operations.get(instruction.bci()) {
        Some(Operation::Store { .. }) => {
            stack_operands(instruction).last().map(|(_, value)| *value)
        }
        _ => None,
    }
}

/// Whether one method descriptor's **return type** is `Z` (`(I)Z`, `()Z`, …).
///
/// The return side of the same reading [`MethodFacts::parameter_types`] makes for the parameters:
/// the frames state one `int` shape for the four int-sized primitives, so only a descriptor says
/// whether the position it types holds a `boolean`. A descriptor this reading cannot parse states no
/// return type at all, and a body presented under it keeps every value as it was rendered.
pub(crate) fn returns_boolean(descriptor: &str) -> bool {
    descriptor
        .strip_prefix('(')
        .and_then(|rest| rest.split_once(')'))
        .is_some_and(|(_, returns)| returns == "Z")
}

/// The value one instruction reads out of one local slot, when it reads that slot at all.
///
/// A load of a slot *produces* a value rather than consuming one, so this is deliberately not
/// [`stack_operands`]: it is what the slot held where the instruction ran, which is what the value
/// the instruction yields stands for — and therefore what a reader of that value means.
fn local_read(instruction: &SsaInstruction, slot: u16) -> Option<ValueId> {
    instruction
        .reads()
        .iter()
        .find_map(|(read, value)| match read {
            Slot::Local(read) if *read == slot => Some(*value),
            _ => None,
        })
}

/// The values one instruction reads off the operand stack, ordered by depth.
///
/// A local read is not an operand: an instruction that loads a local *produces* a value, and reading
/// the local here would print `x = x` for a store that loads the slot it writes. A statement's
/// operands are exactly the stack values the instruction consumes, from the bottom of the consumed
/// region upwards, which is the order a call's receiver and arguments are written in.
///
/// The pattern rules of P3 2.2 read the same operands (the `append` an operand belongs to, the
/// instance a bridge must have been given, the receiver and value of an accessor call), so this is
/// the one reader of them.
pub(crate) fn stack_operands(instruction: &SsaInstruction) -> Vec<(Slot, ValueId)> {
    let mut reads: Vec<(Slot, ValueId)> = instruction
        .reads()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
        .copied()
        .collect();
    reads.sort_by_key(|(slot, _)| match slot {
        Slot::Stack(depth) => *depth,
        Slot::Local(slot) => u32::from(*slot),
    });
    reads
}

/// The expression a constant becomes.
fn literal(constant: &ConstantValue) -> ExprKind {
    match constant {
        ConstantValue::Int(value) => ExprKind::Integer(*value),
        ConstantValue::Long(value) => ExprKind::Long(*value),
        ConstantValue::String(value) => ExprKind::Str(value.clone()),
        ConstantValue::Null => ExprKind::Null,
    }
}

/// One expression whose value a boolean context has proven boolean, spelled as that boolean.
///
/// The only value whose *text* changes is the literal: the frames state `0`/`1` for a boolean
/// literal exactly as they do for an `int` one, and the boolean context is the fact that says which
/// of the two the bytecode pushed — the same reading [`typed_arguments`] makes for a `Z` argument.
/// Every other proven value (a `boolean` parameter's load, a call whose callee descriptor returns
/// `Z`, a claimed field read of descriptor `Z`, a read of a local this body declared `boolean`)
/// already prints what it is, and an expression of any other type is left untouched: the caller is
/// the only place that knows its context, and this helper never invents one.
fn boolean_spelling(expr: Expr) -> Expr {
    let Expr { kind, origin } = expr;
    let spelled = match kind {
        ExprKind::Integer(0) => false,
        ExprKind::Integer(1) => true,
        _ => return Expr { kind, origin },
    };
    Expr {
        kind: ExprKind::Boolean(spelled),
        origin,
    }
}

/// The operator an arithmetic operation becomes.
fn arithmetic_op(op: ArithmeticOp) -> BinaryOp {
    match op {
        ArithmeticOp::Add => BinaryOp::Add,
        ArithmeticOp::Subtract => BinaryOp::Subtract,
        ArithmeticOp::Multiply => BinaryOp::Multiply,
        ArithmeticOp::Divide => BinaryOp::Divide,
        ArithmeticOp::Remainder => BinaryOp::Remainder,
    }
}

/// The condition one sense of a branch states, as the structure that holds it writes it.
///
/// `taken = false` writes the **fall-through** condition — the sense negated, which is what an `if`
/// statement tests: a branch transfers only when its sense holds, so control stays in the `if` when
/// it does not. `taken = true` writes the sense itself, which is what a loop tests when the branch's
/// target is the block that iterates. Writing either one here — once, next to the sense's meaning —
/// is what makes an emitted condition a consequence of the decoded fact rather than of the order two
/// successors happened to be published in.
///
/// The condition node is anchored where the value it tests was produced, and the branch that tests it
/// is added as a *presented* anchor by the caller: the text is the value's, the test is the branch's,
/// and keeping those two apart is what makes a shared BCI visible in the segment table with both
/// provenances instead of one node claiming both.
fn condition(
    op: CompareOp,
    operands: &[(Slot, ValueId)],
    builder: &mut Builder<'_>,
    branch_bci: u32,
    taken: bool,
) -> Result<Expr, String> {
    let expected = if op.reads_two() { 2 } else { 1 };
    if operands.len() != expected {
        return Err(format!(
            "the branch at BCI {branch_bci} reads {} values, but its decoded sense needs {expected}",
            operands.len()
        ));
    }
    // Whether this **position** requires a boolean is decided before any operand is spelled, and
    // from the shape of the test rather than from the shape of a value. A zero test whose operand is
    // **proven boolean** is not a comparison (P3-R5): the frames state one slot shape for the four
    // int-sized primitives, so the fact that decides the text there is a descriptor — `ifeq` on a
    // `boolean` parameter is `!b`, `ifne` is `b`, and any other zero test on it (`iflt`, `ifgt`, …)
    // has no Java spelling at all — the signature would refuse it — so the region is refused rather
    // than written as an int comparison. An integer binary comparison (`if_icmp*`) has no such
    // requirement: it reads two `int`s, and the fact that its *result* is a boolean says nothing
    // about its operands, so both keep the spelling their own evidence states. An operand the layer
    // cannot prove boolean keeps the integer comparison it has always had: that is not a defect, and
    // refusing it would trade the two shapes this rule is about for a layer that refuses every `int`
    // branch.
    //
    // The order is the regression this rule corrects. `boolean_value` counts the `0`/`1` literal as
    // one item of the proof, and a literal is a boolean only where the position already requires
    // one: reading it before the test's shape was known spelled the left operand of
    // `iconst_1; iload_0; if_icmpne` as `true` (`5a8c36a`), text javac refuses
    // (`incomparable types: boolean and int`), where the same bytes had been `1 == arg0`.
    //
    // The operator the *sense* states, and the operator its negation states.
    let (positive, negative) = match op {
        CompareOp::JumpIfZero => (Test::Zero(BinaryOp::Equal), Test::Zero(BinaryOp::NotEqual)),
        CompareOp::JumpIfNotZero => (Test::Zero(BinaryOp::NotEqual), Test::Zero(BinaryOp::Equal)),
        CompareOp::JumpIfNegative => (
            Test::Zero(BinaryOp::Less),
            Test::Zero(BinaryOp::GreaterOrEqual),
        ),
        CompareOp::JumpIfNotNegative => (
            Test::Zero(BinaryOp::GreaterOrEqual),
            Test::Zero(BinaryOp::Less),
        ),
        CompareOp::JumpIfPositive => (
            Test::Zero(BinaryOp::Greater),
            Test::Zero(BinaryOp::LessOrEqual),
        ),
        CompareOp::JumpIfNotPositive => (
            Test::Zero(BinaryOp::LessOrEqual),
            Test::Zero(BinaryOp::Greater),
        ),
        CompareOp::JumpIfNull => (Test::Null(BinaryOp::Equal), Test::Null(BinaryOp::NotEqual)),
        CompareOp::JumpIfNotNull => (Test::Null(BinaryOp::NotEqual), Test::Null(BinaryOp::Equal)),
        CompareOp::JumpIfSame => (Test::Pair(BinaryOp::Equal), Test::Pair(BinaryOp::NotEqual)),
        CompareOp::JumpIfDifferent => (Test::Pair(BinaryOp::NotEqual), Test::Pair(BinaryOp::Equal)),
        CompareOp::JumpIfLess => (
            Test::Pair(BinaryOp::Less),
            Test::Pair(BinaryOp::GreaterOrEqual),
        ),
        CompareOp::JumpIfLessOrEqual => (
            Test::Pair(BinaryOp::LessOrEqual),
            Test::Pair(BinaryOp::Greater),
        ),
        CompareOp::JumpIfGreater => (
            Test::Pair(BinaryOp::Greater),
            Test::Pair(BinaryOp::LessOrEqual),
        ),
        CompareOp::JumpIfGreaterOrEqual => (
            Test::Pair(BinaryOp::GreaterOrEqual),
            Test::Pair(BinaryOp::Less),
        ),
    };
    let test = if taken { positive } else { negative };
    // A pair comparison is the one branch shape that carries a *requirement of its own* about its
    // operands: it reads two `int`s (or, for a null test, two references), so an operand this layer
    // proves boolean beside one it does not has no Java spelling at all — javac refuses
    // `flag() == 1` with `incomparable types: boolean and int` — and ordering two booleans has none
    // either. The two are refused before any operand is rendered, as one rule: a comparison the
    // layer's own evidence says cannot be spelled is not published with a type nobody could accept.
    // The predecessor decision that fixed the *spelling* of a pair's operands (`1 == arg0` keeps its
    // integer literal, each operand keeps its own evidence) is untouched by this: what is refused is
    // a pair whose two spellings cannot stand in one comparison.
    if let Test::Pair(op) = test
        && let Some(reason) = builder.pair_comparison_refusal(op, operands, branch_bci)
    {
        return Err(reason);
    }
    // The position's requirement, and only it, is what may read the literal proof: a boolean is
    // required here exactly where the test is a zero test on a value the evidence owns. The operands
    // are rendered **after** this decision, and each keeps its own evidence: `render_value` spells a
    // `Test::Pair` operand as the `int` it is, and `boolean_spelling` — the `0`/`1` to `false`/`true`
    // adaptation — happens inside the truth-test branch below and nowhere else.
    let boolean = matches!(test, Test::Zero(_)) && builder.boolean_value(operands[0].1, branch_bci);
    let left = builder.render_value(operands[0].1, branch_bci, 0)?;
    let anchor = left.origin.primary().bci();
    if let Test::Zero(op) = test
        && boolean
    {
        let left = boolean_spelling(left);
        return match op {
            BinaryOp::NotEqual => Ok(left),
            BinaryOp::Equal => Ok(Expr::new(
                ExprKind::Not {
                    value: Box::new(left),
                },
                OriginSet::new(crate::source_map::Origin::direct(anchor)),
            )),
            other => Err(format!(
                "the branch at BCI {branch_bci} tests a value proven boolean with `{}`, which no Java source spells",
                other.spell()
            )),
        };
    }
    match test {
        // The second operand is rendered only where it is written, so a zero or null test does not
        // ask the value flow for a value the instruction never read.
        Test::Zero(op) => Ok(binary(op, left, zero(branch_bci), anchor)),
        Test::Null(op) => Ok(binary(
            op,
            left,
            Expr::direct(ExprKind::Null, branch_bci),
            anchor,
        )),
        Test::Pair(op) => {
            let right = builder.render_value(operands[1].1, branch_bci, 0)?;
            Ok(binary(op, left, right, anchor))
        }
    }
}

/// The comparison one sense of a branch states, before its operands are rendered.
enum Test {
    /// One value against the constant zero, with the operator the sense states.
    Zero(BinaryOp),
    /// One reference against `null`.
    Null(BinaryOp),
    /// The two values the branch reads.
    Pair(BinaryOp),
}

/// A binary expression anchored at one bytecode index.
fn binary(op: BinaryOp, left: Expr, right: Expr, bci: u32) -> Expr {
    Expr::direct(
        ExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
        bci,
    )
}

/// The `0` a zero test compares against, anchored where the test is.
fn zero(bci: u32) -> Expr {
    Expr::direct(ExprKind::Integer(0), bci)
}

/// Whether one value is read from a parameter slot the member's descriptor declares `boolean`.
///
/// The free functions of this module read the same fact the builder's own method reads: the value has
/// to be the result of a single `load` of that slot (P3-R5).
fn parameter_boolean(
    ssa: &SsaTable,
    operations: &Operations,
    parameter_types: &BTreeMap<u16, Type>,
    value: ValueId,
) -> bool {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return false;
    };
    let Some(Operation::Load { slot }) = operations.get(*bci) else {
        return false;
    };
    matches!(parameter_types.get(slot), Some(Type::Boolean))
}

/// The class name one dynamic site's implementation handle states, spelled as Java source.
///
/// The handle's owner is the class the implementation member lives in: the qualifier of a `::`
/// reference, the type a constructor's `new` states, the receiver of a static call. One shape names
/// it in three places, so it is spelled once here — and a class name this layer cannot spell as a
/// Java type refuses the site instead of being written as the pool spells it ([`spell_reference`]).
fn implementation_owner(plan: &lambda::Plan, bci: u32) -> Result<String, String> {
    spell_reference(plan.implementation.owner()).ok_or_else(|| {
        format!(
            "the dynamic site at BCI {bci} implements `{}` in the class `{}`, which this layer cannot spell as a Java type",
            plan.implementation.name(),
            plan.implementation.owner()
        )
    })
}

/// The declared type of a local, when the frames state one.
///
/// A reference the frames *name* carries the descriptor form the class file states
/// (`Ljava/lang/Runnable;`, and every array as its own descriptor: `[B`, `[[Ljava/lang/String;`) —
/// that is the fact the frame pass read and kept — so this is where it is spelled as Java source
/// (`java.lang.Runnable`, `byte[]`, `java.lang.String[][]`).
///
/// `Err` carries the fact that has no spelling: the reference name the frames state, which
/// [`spell_reference`] refused as a descriptor with no Java type to be (`[`, `[Lfoo`, `L;`). The
/// caller states the position that would have carried it and refuses that position — never the
/// name, and never a placeholder type in its place.
fn value_type(value: &Value) -> Result<Option<Type>, String> {
    Ok(match value {
        Value::Int => Some(Type::Int),
        Value::Long => Some(Type::Long),
        Value::Float => Some(Type::Float),
        Value::Double => Some(Type::Double),
        Value::Ref(RefType::Named { name, .. }) => {
            let name = String::from_utf8_lossy(name).into_owned();
            Some(Type::Reference(spell_reference(&name).ok_or(name)?))
        }
        Value::Ref(_) | Value::Null => Some(Type::Reference("Object".to_string())),
        _ => None,
    })
}

/// One reference type as the frames and the class file state it, spelled as Java source.
///
/// Published to the crate because four rules write a type name from a class-file fact — a
/// declaration's class, a construction's class, a static field's owner, a dynamic site's
/// implementation class — and a second spelling of "internal form to source form" is exactly the
/// kind of duplicate that drifts.
///
/// Three forms reach this entry point, and `None` is the answer for a form that states no Java type:
///
/// * a **field descriptor** — the frames state a named reference this way (`Ljava/lang/Runnable;`),
///   and so does every array (`[B`, `[[I`, `[[Ljava/lang/String;`). What a descriptor describes is a
///   type, so it must be spelled as one: the `L…;` wrapping comes off, `/` becomes `.`, and an array
///   is spelled from the element outwards with one `[]` per dimension. The array spelling itself is
///   [`crate::lambda::parse_type`]'s, reused rather than copied, so a declaration here and a lambda
///   parameter there cannot drift. A descriptor that cannot be read as a type — a truncated array
///   (`[`, `[V`, `[Lfoo`), a class type with no name (`L;`) — has no Java spelling to publish and is
///   `None`.
/// * a **method descriptor** (`(I)V`) is not a type at all: `None`.
/// * an **internal name** (`java/lang/Math`, the pool's own spelling of a class) is not a descriptor
///   and keeps the historical reading, `/` → `.`. A bare name is never interpreted as one: a class
///   may be called `Lfoo`, so a name that merely looks like a truncated descriptor stays a name.
pub(crate) fn spell_reference(descriptor: &str) -> Option<String> {
    match descriptor.as_bytes().first()? {
        // An array descriptor is read by the one descriptor parser of this crate: the elements it
        // spells are the object-name rule below, applied to the element the descriptor states.
        b'[' => match crate::lambda::parse_type(descriptor.as_bytes(), 0) {
            Some((Type::Reference(spelled), end)) if end == descriptor.len() => Some(spelled),
            _ => None,
        },
        b'(' => None,
        b'L' => match descriptor
            .strip_prefix('L')
            .and_then(|rest| rest.strip_suffix(';'))
        {
            // The object-descriptor form: its interior is the class name, which is never empty and
            // never carries the `;` that would end it.
            Some(internal) if !internal.is_empty() && !internal.contains(';') => {
                Some(internal.replace('/', "."))
            }
            Some(_) => None,
            // No `L…;` wrapping to take off: the pool's own spelling of a class, kept as it is.
            None => Some(descriptor.replace('/', ".")),
        },
        _ => Some(descriptor.replace('/', ".")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_descriptor_that_states_a_type_is_spelled_as_java_source() {
        // The object form, spelled exactly as it always was: the `L…;` wrapping comes off and the
        // separators become dots.
        assert_eq!(
            spell_reference("Ljava/lang/String;").as_deref(),
            Some("java.lang.String")
        );
        // A pool's own internal name (no wrapping to take off) keeps the same reading, which is what
        // keeps the class-name positions and the declarations spelling the same class the same way.
        assert_eq!(
            spell_reference("java/lang/String").as_deref(),
            Some("java.lang.String")
        );
        // An array descriptor is a type: the element is spelled first, with the object-name rule
        // above, and one `[]` follows per dimension.
        assert_eq!(spell_reference("[B").as_deref(), Some("byte[]"));
        assert_eq!(spell_reference("[Z").as_deref(), Some("boolean[]"));
        assert_eq!(
            spell_reference("[Ljava/lang/String;").as_deref(),
            Some("java.lang.String[]")
        );
        assert_eq!(spell_reference("[[I").as_deref(), Some("int[][]"));
        assert_eq!(
            spell_reference("[[Ljava/lang/String;").as_deref(),
            Some("java.lang.String[][]")
        );
    }

    #[test]
    fn a_name_that_states_no_java_type_is_refused_rather_than_written() {
        // The empty string and a method descriptor are not types.
        assert_eq!(spell_reference(""), None);
        assert_eq!(spell_reference("(I)V"), None);
        // A class type whose name is missing or carries the `;` that would end it earlier.
        assert_eq!(spell_reference("L;"), None);
        assert_eq!(spell_reference("Lfoo;bar;"), None);
        // A bare name is a name and not a descriptor: a class may be called `Lfoo`, so the forms
        // only a truncated descriptor could have are still read as the pool's own spelling.
        assert_eq!(spell_reference("Lfoo").as_deref(), Some("Lfoo"));
    }
}
