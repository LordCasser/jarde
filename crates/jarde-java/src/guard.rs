//! The guarded regions of P3 2.4: a `try`-with-resources and a `synchronized` statement, each
//! proved from one run's own facts before either is written.
//!
//! # What a guarded region is, and why proving it is a rule's job
//!
//! Both statements put code the bytecode runs **outside** the statement's own braces into those
//! braces: a `try (T n = expr) { body }` writes the resource's initialisation in its header, and a
//! `synchronized (lock) { body }` writes the monitor's enter there. Neither move may change how often
//! anything runs or in what order — that is the rule's whole obligation (the spec sentence is
//! "恢复 MUST 保留异常优先级、资源关闭/抑制异常、monitor … 和副作用顺序"), and it is what this module
//! checks instruction by instruction rather than assuming from "this looks like a `try`".
//!
//! What javac 23.0.1 emits for `--release 8` is the shape the rules read, and the fixture under
//! `tests/fixtures/p3-handlers/` is the committed sample every claim below was read off:
//!
//! ```text
//! try (Res r = open("r")) { body(); }                 static void one()
//!   0: ldc "r"                 ┐
//!   2: invokestatic open       │ the resource's own initialisation: one statement, whose value
//!   5: astore_0                ┘ lands in slot 0 — **outside** the protected range
//!   6: invokestatic body       ┐ the guarded body, and the row that protects it: [6, 9) → 20
//!   9: aload_0                 │ the normal path: `if (r != null) r.close();`
//!  10: ifnull 40               │
//!  13: aload_0                 │
//!  14: invokevirtual close     ┘
//!  17: goto 40                   … and the run continues at 40
//!  20: astore_1                  the exceptional path: the primary is stored …
//!  21: aload_0 22: ifnull 38 25: aload_0 26: invokevirtual close
//!  32: astore_2                  … the close's own exception is stored (its own row covers 25..29)
//!  33: aload_1 34: aload_2 35: invokevirtual addSuppressed
//!  38: aload_1 39: athrow        … and the **primary**, not the close's exception, is rethrown
//!      Exception table: [6,9) → 20 Throwable;  [25,29) → 32 Throwable
//! ```
//!
//! Three facts are load-bearing and none of them is visible in the *shape* alone:
//!
//! * **close order.** The resources are declared in the order their initialisations run, and Java
//!   closes them in the **reverse** order. The rule does not assume that: it reads the normal-path
//!   close chain, matches it against the resources **in reverse declaration order**, and refuses if
//!   the chain closes them in any other order or misses one. So the header it writes is exactly the
//!   order that makes the presented text close what the bytecode closed, in the order it closed.
//! * **primary and suppressed.** `addSuppressed`'s **receiver** is the exception the handler stored
//!   (the primary), its **argument** is the close's own exception, and the `athrow` that ends the
//!   handler rethrows the **primary**. All three are value-flow facts of the same run: the rule
//!   reads the value the handler's first `astore` wrote and requires the load before
//!   `addSuppressed` and the load before the `athrow` to be **that** value. A handler that closes
//!   and rethrows without suppressing anything, or one whose suppression is the other way round, is
//!   refused with the link named.
//! * **the ranges.** The row that protects the body must start exactly where the body's first
//!   instruction is and end exactly where the normal close chain begins; each outer resource's row
//!   must end exactly at the end of the next level's handler. A row that covers *part* of the shape,
//!   or two rows that both cover it, is refused: partial coverage is exactly the case where writing
//!   the `try` would drop a close.
//!
//! # A narrow `finally` copy can be claimed
//!
//! A `finally` clause is not a region with a handler: javac **copies** its code onto every exit path
//! of the `try` (the fixture's `fin()` shows the two copies of `tail()`, one on the normal path and
//! one in the handler that rethrows), and presenting that as one `try { … } finally { … }` restates
//! the source only if the copies are provably the same code and every exit runs exactly one copy.
//! The narrow `finally@1` rule claims matching straight copies only when its protected body has
//! no branch or transfer, the half-open exception range excludes both copies, and every owned
//! instruction belongs to the proof. A candidate outside that slice remains **refused** and stated
//! ([`Unproven::FinallyCopy`], reported under `jre_guard_finally_copy` with the copy's own BCI).
//!
//! # The boundary this module does not cross
//!
//! * A `try`-with-resources with a `catch` beside it is refused, not partially presented: the
//!   compiler wraps the whole construct in a row of its own, so a row protects part of this shape
//!   and is not part of it, and the rule says so ([`Unproven::Unexplained`]) instead of presenting a
//!   `try` whose outer handler it would have to ignore.
//! * A guarded body that branches is refused ([`Unproven::Body`]): the rule presents a body whose
//!   instructions run one after another, and a branch inside the protected range is a structure the
//!   *nested* recovery would have to prove under the guard's own frames.
//! * Whether the resource's class really implements `AutoCloseable` is a **resolution** fact this
//!   run does not read. What is read is what the value flow says: every `close` call is on the value
//!   the resource's slot holds, and every `monitorexit` reads the slot the `monitorenter`'s value
//!   was stored in.
//!
//! # Why a statement is an instruction range and not a block
//!
//! The canonical graph fuses straight-line code, so for the sample above the resource's
//! initialisation (0…5), the guarded body (6) and the first close of the normal path (9…10) are **one
//! node**. A rule that read whole blocks would therefore be claiming instructions it must not write
//! and writing instructions it must not claim. So what this module reads is `(BCI, operation)` pairs
//! — and what it returns is the range of instructions the statement's own block keeps for itself
//! ([`Plan::lead`]), the range the body is written from ([`Plan::body`]), the facts the proof read
//! ([`Plan::facts`], recorded as the artifact's anchors) and the blocks the statement claims
//! ([`Plan::owned`]). `explained` is the check that makes the claim safe: every instruction between
//! the statement's own start and its join has to belong to one of the shape's pieces, so a block
//! claimed and not written cannot drop a statement.

use std::collections::BTreeMap;

use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalCfg, Definition, Slot, SsaInstruction, SsaTable, ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::ExceptionHandlerFact;

use crate::build::stack_operands;
use crate::decode::Operations;
use crate::facts::{CompareOp, Operation};
use crate::normal_flow::NormalFlowView;
use crate::pass::{FINALLY, MONITOR, Pass, TWR};
use crate::refusal::Refusal;
use crate::stop::{StopReason, charge, poll};

/// One resource of a `try (…)` header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resource {
    /// The local slot the resource's value is stored in.
    slot: u16,
    /// The **instructions** of the resource's own initialisation, as a half-open BCI range. They are
    /// not a block of the canonical graph: that graph fuses straight-line code, so the first
    /// resource's initialisation and the guarded body after it are one node — the header is an
    /// instruction range, and writing it there is what keeps its value from being produced twice.
    init: (u32, u32),
    /// The BCI of the `close` call the **normal** path makes on it: the anchor the header's own text
    /// carries.
    close_bci: u32,
    /// The BCI of the `close` call the **exceptional** path makes on it. This is the same handler
    /// proof that established the close, carried with this resource so its owner can be checked
    /// without guessing among a plan's other facts.
    exceptional_close_bci: u32,
}

impl Resource {
    /// The slot the resource lives in.
    pub fn slot(&self) -> u16 {
        self.slot
    }

    /// The resource's own initialisation, as a BCI range.
    pub fn init(&self) -> (u32, u32) {
        self.init
    }

    /// The BCI of the `close` call the normal path makes on it.
    pub fn close_bci(&self) -> u32 {
        self.close_bci
    }

    /// The BCI of the `close` call the exceptional path makes on it.
    pub fn exceptional_close_bci(&self) -> u32 {
        self.exceptional_close_bci
    }
}

/// Which guarded statement a region is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Shape {
    /// `try (T n = …; …) { body }`, with the resources in **declaration** order.
    Resources {
        resources: Vec<Resource>,
        /// A post-close load/return pair whose saved value was proved to originate in the body.
        returns: Option<u32>,
    },
    /// `synchronized (lock) { body }`, with the BCI of the `monitorenter` the header reads its lock
    /// from.
    Monitor {
        /// The BCI of the `monitorenter` the header reads its lock from.
        enter_bci: u32,
        /// The unique normal-path `monitorexit` proved by this shape. It is distinct from the
        /// handler exit and is carried explicitly so later value placement never infers ownership
        /// from instruction order or from the aggregate proof anchors.
        normal_exit_bci: u32,
        /// The BCI of the `return` the **normal** path ends in, where it ends in one: `None` is the
        /// `goto` shape, whose run continues after the statement, and `Some(bci)` the shape whose
        /// region returns the value the body left on the stack — the return is written *inside* the
        /// braces, and the statement continues nowhere.
        returns: Option<u32>,
    },
    /// A straight protected body whose saved return and two cleanup copies were proved.
    Finally {
        normal_cleanup: (u32, u32),
        returns: u32,
    },
}

/// One proved guarded region: the shape, the body it guards, every block it owns, and where the run
/// continues.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    shape: Shape,
    /// The instruction range of the statement's own block that is written **before** it: what that
    /// block ran before the statement began. Empty where the statement starts where its block does,
    /// which is where javac puts the whole of it.
    lead: (u32, u32),
    /// The guarded body, as a BCI range: the instructions written between the braces.
    body: (u32, u32),
    /// Every block the statement claims. A claimed block is never quoted and written twice, and the
    /// rule checks before it claims one that every instruction of the statement's span belongs to
    /// the shape — a block claimed and not written would drop its statements from the artifact.
    owned: Vec<CanonicalBlockId>,
    /// The block the run continues at after the statement, when it continues at all.
    join: Option<CanonicalBlockId>,
    /// The instructions the proof **read**: the resources' own stores, the closes the handlers
    /// perform, the `addSuppressed` calls and the rethrows. They write no text of their own — the
    /// statement took their place — and they are the anchors the artifact records beside it, so that
    /// which instructions made a `close` or a suppression true is answerable from the text.
    facts: Vec<u32>,
}

impl Plan {
    /// Which guarded statement this is.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// The instruction range written before the statement.
    pub fn lead(&self) -> (u32, u32) {
        self.lead
    }

    /// The guarded body's instructions, as a BCI range.
    pub fn body(&self) -> (u32, u32) {
        self.body
    }

    /// Every block the statement claims.
    pub fn owned(&self) -> &[CanonicalBlockId] {
        &self.owned
    }

    /// The block the run continues at, when it does.
    pub fn join(&self) -> Option<&CanonicalBlockId> {
        self.join.as_ref()
    }

    /// The instructions the proof read, in BCI order.
    pub fn facts(&self) -> &[u32] {
        &self.facts
    }

    /// The registered rule that proved this region.
    pub fn pass(&self) -> &'static Pass {
        match self.shape {
            Shape::Resources { .. } => &TWR,
            Shape::Monitor { .. } => &MONITOR,
            Shape::Finally { .. } => &FINALLY,
        }
    }
}

/// What one examination of a block the walk cannot leave through the normal flow concludes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Verdict {
    /// Nothing here is a guarded region this build examines: the walk states its own reason for the
    /// edge it cannot present.
    NotGuarded,
    /// A guarded shape was examined and **refused**, and this is why. `pass` is the rule that
    /// examined it, or `None` for a candidate outside a registered rule's claim.
    Refused {
        /// The rule that examined the shape and refused it.
        pass: Option<&'static Pass>,
        /// The refusal itself: its diagnostic code and its one-sentence statement.
        refusal: Refusal,
        /// The instruction the refusal is anchored at.
        at: u32,
    },
    /// The shape is proved, and this is the region it is.
    Claimed(Plan),
}

impl Verdict {
    /// One refusal, as the walk reads it: the link that fell short and the anchor it is stated at.
    fn refused(pass: Option<&'static Pass>, unproven: Unproven, at: u32) -> Self {
        Self::Refused {
            pass,
            refusal: unproven.at(at),
            at,
        }
    }
}

/// Why one guarded shape was not presented.
///
/// Each variant is one link of the proof, so that a refusal names what fell short rather than "not a
/// `try`": a reader can tell a body whose close order is not the reverse of its declarations from one
/// whose handler does not suppress into the primary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Unproven {
    /// More than one exception-table row covers the region's first protected instruction.
    RowsOverlap,
    /// The protecting row does not begin where the region's first statement is.
    RangeStart,
    /// The protecting row does not end where this shape requires.
    RangeEnd,
    /// The resource's own initialisation is not one statement of this block whose value lands in a
    /// slot.
    ResourceInit,
    /// The statement's normal path runs on into the rest of the statement's own block, where no
    /// block begins: what the method runs after the statement has nowhere for the walk to continue.
    Continuation,
    /// Two resources of one header would be declared on one slot.
    ResourceSlot,
    /// The handler does not close the resource of its own level.
    CloseTarget,
    /// The normal path does not close every resource in the reverse order of their declarations.
    CloseOrder,
    /// The exceptional path's close is not protected by a row of its own.
    CloseGuard,
    /// The exception a close raises is not suppressed into the primary.
    Suppressed,
    /// The handler does not rethrow the exception it stored.
    Primary,
    /// The guarded body is not a run of statements this rule presents.
    Body,
    /// A handler of the region is not the instruction sequence this rule proves.
    Handler,
    /// The monitor is not entered once and left on every path out of the region.
    Monitor,
    /// A row of this body's table protects part of the shape and is not part of it — the `catch` a
    /// compiler wraps a `try`-with-resources in, for instance.
    Unexplained,
    /// An instruction between the statement's own start and its join is not part of the shape:
    /// claiming its block would drop the statement the rest of that block holds.
    Span,
    /// The run's profile does not admit the rule's output: a `try (…)` header is Java 7 syntax, and
    /// the artifact is presented as an older release ([`crate::pass::Pass::admits`]).
    Profile,
    /// The exceptional path repeats code the normal path also runs: the `finally` copy.
    FinallyCopy,
}

impl Unproven {
    /// The diagnostic code this refusal is reported under.
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::RowsOverlap => "jre_guard_rows_overlap",
            Self::RangeStart | Self::RangeEnd => "jre_guard_handler_range",
            Self::ResourceInit => "jre_guard_resource_init",
            Self::Continuation => "jre_guard_continuation",
            Self::ResourceSlot => "jre_guard_resource_slot",
            Self::CloseTarget => "jre_guard_close_target",
            Self::CloseOrder => "jre_guard_close_order",
            Self::CloseGuard => "jre_guard_close_guard",
            Self::Suppressed => "jre_guard_suppressed",
            Self::Primary => "jre_guard_primary",
            Self::Body => "jre_guard_body",
            Self::Handler => "jre_guard_handler",
            Self::Monitor => "jre_guard_monitor",
            Self::Unexplained => "jre_guard_unexplained_row",
            Self::Span => "jre_guard_span",
            Self::Profile => "jre_guard_profile",
            Self::FinallyCopy => "jre_guard_finally_copy",
        }
    }

    /// What the refusal states, in one sentence.
    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::RowsOverlap => {
                "more than one exception-table row covers the region's first protected instruction: the priority between them is the table's own order, and the guarded statement would drop the row this shape does not present"
            }
            Self::RangeStart => {
                "the protecting row does not begin where the region's first statement is: the shape's own range is not the one the table declares"
            }
            Self::RangeEnd => {
                "the protecting row does not end where this shape requires: the range a handler covers is not the one the guarded statement would run"
            }
            Self::ResourceInit => {
                "the resource's own initialisation is not one statement of this block whose value lands in a slot: writing it in the header would move or drop an effect"
            }
            Self::Continuation => {
                "the statement's normal path runs on inside the statement's own block, where no block begins: what the method runs after the `try` cannot be written from there, and the guarded statement is refused rather than presented without it"
            }
            Self::ResourceSlot => "two resources of this header would be declared on one slot",
            Self::CloseTarget => {
                "the handler does not close the resource its own level declares: the close it performs is on another value than the one the header would name"
            }
            Self::CloseOrder => {
                "the normal path does not close every resource in the reverse order of their initialisations, so no header order reproduces the closes the bytecode performs"
            }
            Self::CloseGuard => {
                "the exceptional path's close is not protected by a row of its own: an exception it raises would replace the primary instead of being suppressed into it"
            }
            Self::Suppressed => {
                "the exception a close raises is not suppressed into the primary: the receiver of `addSuppressed` is not the exception the handler stored"
            }
            Self::Primary => {
                "the handler does not rethrow the exception it stored: the region would leave with another exception than the one the bytecode raised"
            }
            Self::Body => {
                "the guarded body is not a run of statements this rule presents: it branches, leaves the region, or holds an operation no statement of this subset covers"
            }
            Self::Handler => {
                "the handler's own instruction sequence is not the one this rule proves for a guarded region"
            }
            Self::Monitor => {
                "the monitor is not entered once and left on every path out of the region: a `synchronized` statement would drop the exit the bytecode performs"
            }
            Self::Unexplained => {
                "a row of this body's exception table protects part of this shape and is not part of it: the guarded statement would present the region without the handler that row names"
            }
            Self::Span => {
                "an instruction between the statement's own start and its join is not part of the shape: presenting the statement would drop the statement its block holds"
            }
            Self::Profile => {
                "the run's profile does not admit this rule's output: a `try`-with-resources header is Java 7 syntax, and this run presents the artifact as an older release"
            }
            Self::FinallyCopy => {
                "the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`"
            }
        }
    }

    /// The refusal, anchored at one instruction.
    fn at(self, at: u32) -> Refusal {
        Refusal::shape(self.code(), format!("BCI {at}: {}", self.message()))
    }
}

/// A refusal and the instruction it is about.
type Cause = (Unproven, u32);

/// The resource proof can stop for its shape, or because its bounded read ran out of budget.
enum TwrFailure {
    Proof(Cause),
    Stop(StopReason),
}

impl From<Cause> for TwrFailure {
    fn from(cause: Cause) -> Self {
        Self::Proof(cause)
    }
}

impl From<StopReason> for TwrFailure {
    fn from(reason: StopReason) -> Self {
        Self::Stop(reason)
    }
}

/// One instruction of the body, with the block it belongs to.
#[derive(Clone, Copy)]
struct Step<'a> {
    instruction: &'a SsaInstruction,
    block: &'a CanonicalBlockId,
}

/// The facts one examination reads: the graph, the names, the decode and the exception table.
struct Facts<'a> {
    canonical: &'a CanonicalCfg,
    view: &'a NormalFlowView,
    ssa: &'a SsaTable,
    ops: &'a Operations,
    handlers: &'a [ExceptionHandlerFact],
    budget: &'a mut Budget,
    /// Every instruction of the body by BCI, with the block it belongs to.
    index: BTreeMap<u32, Step<'a>>,
    /// Every BCI of the body, ascending: how one instruction's end is read, without a second decode.
    order: Vec<u32>,
    /// A canonical block, by its own start BCI.
    starts: BTreeMap<u32, &'a CanonicalBlockId>,
    /// A canonical block's end BCI, by its own start BCI.
    ends: BTreeMap<u32, u32>,
}

impl<'a> Facts<'a> {
    fn new(
        canonical: &'a CanonicalCfg,
        view: &'a NormalFlowView,
        ssa: &'a SsaTable,
        ops: &'a Operations,
        handlers: &'a [ExceptionHandlerFact],
        budget: &'a mut Budget,
    ) -> Self {
        let mut index = BTreeMap::new();
        let mut order = Vec::new();
        let mut starts: BTreeMap<u32, &'a CanonicalBlockId> = BTreeMap::new();
        let mut ends: BTreeMap<u32, u32> = BTreeMap::new();
        for block in ssa.blocks() {
            starts
                .entry(block.block().bci())
                .or_insert_with(|| block.block());
            for instruction in block.instructions() {
                index.insert(
                    instruction.bci(),
                    Step {
                        instruction,
                        block: block.block(),
                    },
                );
                order.push(instruction.bci());
            }
        }
        for block in canonical.blocks() {
            ends.insert(block.id().bci(), block.end_bci());
        }
        order.sort_unstable();
        Self {
            canonical,
            view,
            ssa,
            ops,
            handlers,
            budget,
            index,
            order,
            starts,
            ends,
        }
    }

    /// The decoded operation at one bytecode index.
    fn op(&self, bci: u32) -> Option<&'a Operation> {
        self.ops.get(bci)
    }

    /// The instruction at one bytecode index, with the block it belongs to.
    fn step(&self, bci: u32) -> Option<Step<'a>> {
        self.index.get(&bci).copied()
    }

    /// The instructions of one block, in execution order.
    fn in_block(&self, block: &CanonicalBlockId) -> &'a [SsaInstruction] {
        self.ssa
            .block(block)
            .map(|entry| entry.instructions())
            .unwrap_or(&[])
    }

    /// One block's instructions, as `(BCI, operation)` pairs.
    fn sequence(&self, block: &CanonicalBlockId) -> Vec<(u32, Option<&'a Operation>)> {
        self.in_block(block)
            .iter()
            .map(|instruction| (instruction.bci(), self.op(instruction.bci())))
            .collect()
    }

    /// The bytecode index just past one instruction.
    fn next_bci(&self, bci: u32) -> Option<u32> {
        let position = self.order.binary_search(&bci).ok()?;
        self.order.get(position + 1).copied()
    }

    /// The bytecode index just before one instruction.
    fn previous_bci(&self, bci: u32) -> Option<u32> {
        let position = self.order.binary_search(&bci).ok()?;
        position
            .checked_sub(1)
            .and_then(|at| self.order.get(at).copied())
    }

    /// The BCI just past one instruction's range.
    fn span_end(&self, last: u32) -> u32 {
        self.next_bci(last).unwrap_or(last.saturating_add(1))
    }

    /// The BCI just past one block.
    fn end_of(&self, block: &CanonicalBlockId) -> u32 {
        self.ends
            .get(&block.bci())
            .copied()
            .or_else(|| {
                self.in_block(block)
                    .last()
                    .map(|last| self.span_end(last.bci()))
            })
            .unwrap_or_else(|| block.bci().saturating_add(1))
    }

    /// Every BCI of one range, ascending.
    fn bcis(&self, span: (u32, u32)) -> Vec<u32> {
        let start = self.order.partition_point(|bci| *bci < span.0);
        self.order[start..]
            .iter()
            .take_while(|bci| **bci < span.1)
            .copied()
            .collect()
    }

    /// The blocks holding an instruction of one range, in BCI order.
    fn blocks_in(&self, span: (u32, u32)) -> Vec<CanonicalBlockId> {
        let mut blocks: Vec<CanonicalBlockId> = Vec::new();
        for bci in self.bcis(span) {
            if let Some(step) = self.step(bci)
                && !blocks.contains(step.block)
            {
                blocks.push(step.block.clone());
            }
        }
        blocks
    }

    /// The block one bytecode index's instruction belongs to.
    fn block_of(&self, bci: u32) -> Option<&'a CanonicalBlockId> {
        self.index.get(&bci).map(|step| step.block)
    }

    /// The block whose first instruction is at one bytecode index.
    fn block_at(&self, bci: u32) -> Option<CanonicalBlockId> {
        self.starts.get(&bci).map(|block| (*block).clone())
    }

    /// The rows whose declared range covers one bytecode index, in table order.
    fn covering(&self, bci: u32) -> Vec<&'a ExceptionHandlerFact> {
        self.handlers
            .iter()
            .filter(|row| row.start_bci <= bci && bci < row.end_bci)
            .collect()
    }

    /// The **innermost** row whose declared range covers one instruction.
    ///
    /// Nested ranges both cover the code inside them, so "the row that protects this instruction" is
    /// the narrowest one — the innermost record, which is the one the JVM consults first. Two rows of
    /// equal width covering it are a priority the table's order alone decides, and the shapes here
    /// refuse that rather than pick one.
    fn innermost(&self, bci: u32) -> Option<&'a ExceptionHandlerFact> {
        let covering = self.covering(bci);
        let width = |row: &&ExceptionHandlerFact| row.end_bci.saturating_sub(row.start_bci);
        let narrowest = covering.iter().map(width).min()?;
        let mut candidates = covering.iter().filter(|row| width(row) == narrowest);
        let first = *candidates.next()?;
        if candidates.next().is_some() {
            return None;
        }
        Some(first)
    }

    /// Whether one row's handler runs a `close` call anywhere in it.
    ///
    /// This is the cheap half of the level proof — the shape of the handler without the resource it
    /// closes and the suppression it performs — and it is what tells a level of a `try` header from
    /// the `catch` a compiler wraps the whole statement in.
    fn closes_something(&self, row: &ExceptionHandlerFact) -> bool {
        let Some(entry) = self.row_handler(row) else {
            return false;
        };
        let mut pending = vec![entry.clone()];
        let mut seen: Vec<CanonicalBlockId> = Vec::new();
        while let Some(block) = pending.pop() {
            if seen.contains(&block) || seen.len() >= 32 {
                continue;
            }
            if self.in_block(&block).iter().any(|instruction| {
                matches!(
                    self.op(instruction.bci()),
                    Some(Operation::Invoke(called))
                        if called.name() == "close" && called.descriptor() == "()V"
                )
            }) {
                return true;
            }
            pending.extend(self.view.successor_ids(&block));
            seen.push(block);
        }
        false
    }

    /// The canonical block one row maps its handler entry to.
    fn row_handler(&self, row: &ExceptionHandlerFact) -> Option<CanonicalBlockId> {
        self.canonical
            .handler_rows()
            .iter()
            .find(|entry| entry.ordinal() == row.ordinal)
            .and_then(|entry| entry.handler().cloned())
    }

    /// Whether every instruction of a range is one a statement of this subset can carry.
    ///
    /// The body of a guarded region is written between braces, so an instruction the builder would
    /// have to quote — a branch, an exit, an unmodelled operation — is a reason to refuse the whole
    /// region: a quote *inside* the statement would present a `try` whose body is not the one the
    /// bytecode runs.
    fn statement_free(&self, span: (u32, u32)) -> bool {
        self.bcis(span).into_iter().all(|bci| {
            matches!(
                self.op(bci),
                Some(Operation::Push(_))
                    | Some(Operation::Load { .. })
                    | Some(Operation::Store { .. })
                    | Some(Operation::Arithmetic { .. })
                    | Some(Operation::Negate)
                    | Some(Operation::Increment { .. })
                    | Some(Operation::Invoke(_))
                    | Some(Operation::Allocate { .. })
                    | Some(Operation::Duplicate)
                    | Some(Operation::Field { .. })
                    | Some(Operation::CheckCast { .. })
                    | Some(Operation::InvokeDynamic(_))
                    | Some(Operation::ArrayLoad)
            )
        })
    }

    /// Whether every value written in a range that is not a local store is read inside it.
    ///
    /// This is the "the header is one expression" check: what the statement's own header runs has to
    /// be the value it declares (or the lock it enters), so an instruction whose result nothing in
    /// the header reads is a statement of its own — and writing *it* in the header would move it.
    fn one_expression(&self, span: (u32, u32), end: u32) -> bool {
        let mut stores = 0usize;
        for bci in self.bcis(span) {
            let Some(step) = self.step(bci) else {
                return false;
            };
            if matches!(self.op(bci), Some(Operation::Store { .. })) {
                stores += 1;
                continue;
            }
            for (_, written) in step.instruction.writes() {
                let consumed = self
                    .bcis((span.0, self.span_end(end)))
                    .into_iter()
                    .any(|reader| {
                        self.step(reader).is_some_and(|reader| {
                            reader
                                .instruction
                                .reads()
                                .iter()
                                .any(|(_, read)| self.same(*read, *written))
                        })
                    });
                if !consumed {
                    return false;
                }
            }
        }
        stores <= 1
    }

    /// The value one name stands for, following the trivial-phi replacements.
    fn resolve(&self, value: ValueId) -> ValueId {
        let mut value = value;
        for _ in 0..64 {
            match self.ssa.value(value).replaced_by() {
                Some(next) => value = next,
                None => break,
            }
        }
        value
    }

    /// Whether two names stand for the same value of this run.
    fn same(&self, left: ValueId, right: ValueId) -> bool {
        self.resolve(left) == self.resolve(right)
    }

    fn charge(&mut self, at: u32) -> Result<(), StopReason> {
        poll(self.budget, Some(at))?;
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(at),
        )
    }
}

/// Proves the only post-close tail this TWR shape can move into its body: a local read followed by
/// the method's terminal return, where the local's reaching value is a body store of a value the
/// body already evaluated. The exact instruction slice and terminal CFG block keep any intervening
/// statement or continuation outside this narrow rule.
fn twr_return_tail(
    facts: &mut Facts<'_>,
    body: (u32, u32),
    start: u32,
) -> Result<Option<(u32, u32, u32)>, StopReason> {
    let load_bci = start;
    let Some(return_bci) = facts.next_bci(load_bci) else {
        return Ok(None);
    };
    let Some(block) = facts.block_of(start).cloned() else {
        return Ok(None);
    };
    let tail = facts.bcis((start, facts.end_of(&block)));
    if tail != [load_bci, return_bci]
        || facts.block_of(load_bci) != Some(&block)
        || facts.block_of(return_bci) != Some(&block)
    {
        return Ok(None);
    }
    for bci in [load_bci, return_bci] {
        facts.charge(bci)?;
    }
    if !facts.view.successor_ids(&block).is_empty() {
        return Ok(None);
    }
    let Some(load) = facts.step(load_bci).map(|step| step.instruction) else {
        return Ok(None);
    };
    let Some(return_instruction) = facts.step(return_bci).map(|step| step.instruction) else {
        return Ok(None);
    };
    let Some(Operation::Load { slot: loaded_slot }) = facts.op(load_bci) else {
        return Ok(None);
    };
    if facts.op(return_bci) != Some(&Operation::Return) {
        return Ok(None);
    }
    let mut local_reads = load
        .reads()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Local(_)));
    let Some((Slot::Local(slot), local_value)) = local_reads.next() else {
        return Ok(None);
    };
    if slot != loaded_slot || local_reads.next().is_some() {
        return Ok(None);
    }
    let mut load_writes = load
        .writes()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Stack(_)));
    let Some((_, loaded_value)) = load_writes.next() else {
        return Ok(None);
    };
    if load_writes.next().is_some() {
        return Ok(None);
    }
    let return_reads: Vec<ValueId> = return_instruction
        .reads()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
        .map(|(_, value)| *value)
        .collect();
    if return_reads.len() != 1 || !facts.same(return_reads[0], *loaded_value) {
        return Ok(None);
    }

    let mut stores = Vec::new();
    for store_bci in facts.bcis(body) {
        facts.charge(store_bci)?;
        if !matches!(facts.op(store_bci), Some(Operation::Store { slot: stored }) if stored == slot)
        {
            continue;
        }
        let Some(store) = facts.step(store_bci).map(|step| step.instruction) else {
            continue;
        };
        if !store.writes().iter().any(|(target, written)| {
            matches!(target, Slot::Local(target) if target == slot)
                && facts.same(*written, *local_value)
        }) {
            continue;
        }
        let stack_inputs: Vec<ValueId> = store
            .reads()
            .iter()
            .filter(|(source, _)| matches!(source, Slot::Stack(_)))
            .map(|(_, value)| *value)
            .collect();
        if stack_inputs.len() != 1 {
            continue;
        }
        let Definition::Instruction { bci: producer, .. } = facts.ssa.value(stack_inputs[0]).def()
        else {
            continue;
        };
        if body.0 <= *producer && *producer < store_bci {
            stores.push((store_bci, stack_inputs[0]));
        }
    }
    let [(store_bci, _)] = stores.as_slice() else {
        return Ok(None);
    };
    Ok(Some((load_bci, return_bci, *store_bci)))
}

/// Whether an instruction's receiver is the value a preceding load produced.
///
/// A call's receiver is its **first** stack operand (`crate::build::stack_operands`' order), and the
/// load that produced it is what ties the call to the slot the rule named: without that tie, "the
/// handler closes the resource" would be a guess from a value that happens to be on the stack.
fn receiver_is(facts: &Facts<'_>, call: &SsaInstruction, load: &SsaInstruction) -> bool {
    let Some(receiver) = call
        .reads()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
        .min_by_key(|(slot, _)| match slot {
            Slot::Stack(depth) => *depth,
            Slot::Local(slot) => u32::from(*slot),
        })
        .map(|(_, value)| *value)
    else {
        return false;
    };
    load.writes()
        .iter()
        .any(|(_, written)| facts.same(*written, receiver))
}

/// The instruction range one resource's initialisation occupies, and the slot it fills.
///
/// The span is grown **backwards** from the store at `end` while it stays one statement: every
/// instruction but the store has to write a value that is read inside the span, and the store has to
/// write a slot with a value the span produced. A statement that ended before this one fails that
/// criterion (its own store's value is read by nothing inside), which is what stops the growth — and
/// it never crosses `floor`, the instruction the walk had already reached.
fn initialisation(facts: &Facts<'_>, end: u32, floor: u32) -> Result<((u32, u32), u16), Cause> {
    let store = facts
        .previous_bci(end)
        .filter(|store| *store >= floor)
        .ok_or((Unproven::ResourceInit, end))?;
    let Some(Operation::Store { slot }) = facts.op(store) else {
        return Err((Unproven::ResourceInit, store));
    };
    let mut start = store;
    while let Some(previous) = facts.previous_bci(start) {
        if previous < floor {
            break;
        }
        if !single_statement(facts, (previous, end), store) {
            break;
        }
        start = previous;
    }
    if !single_statement(facts, (start, end), store) {
        return Err((Unproven::ResourceInit, store));
    }
    Ok(((start, end), *slot))
}

/// Whether the store before a row's protected range is a resource's own initialisation.
///
/// The distinction is the **value**: `Res r = open(…)` and `Res r = new Res()` fill a local from an
/// invocation or a `new`, and a header is what the range that follows can be; `int x = 1`, `x = n`
/// and `x = obj.field` fill it from a value no resource's construction produced, and the row is a
/// `catch` rather than a resource of the shape. What is read is the statement that **ends** at the
/// store — the run [`single_statement`] grows backwards from it, the same reading
/// [`initialisation`] takes of a header's own initialisation — and the `new`/invocation has to be
/// part of that run: a value that was already on the stack when the statement began belongs to
/// another statement, and `int y = 2; int x = 1;` answers about `int x = 1;` alone.
///
/// The growth stops at the instruction that is not part of the statement, which is the end of the
/// statement *before* this one: that boundary is not evidence about this statement's value, so what
/// is inspected below is the run the store ends. A store whose own statement cannot be read at all —
/// its first step grows into an instruction that does not feed the value, or its whole run lies
/// outside the block the row was examined in — keeps today's refusal (`true`): "not a resource" is a
/// conclusion about a value, and a statement the proof could not read is not evidence for it.
fn initialises_resource(facts: &Facts<'_>, store: u32, floor: u32) -> bool {
    let end = facts.span_end(store);
    let mut start = store;
    let mut read = false;
    while let Some(previous) = facts
        .previous_bci(start)
        .filter(|previous| *previous >= floor)
    {
        if !single_statement(facts, (previous, end), store) {
            if !read {
                return true;
            }
            break;
        }
        read = true;
        start = previous;
    }
    if !read {
        return true;
    }
    facts.bcis((start, end)).into_iter().any(|bci| {
        matches!(
            facts.op(bci),
            Some(Operation::Allocate { .. })
                | Some(Operation::Invoke(_))
                | Some(Operation::InvokeDynamic(_))
        )
    })
}

/// Whether one proved initialisation is exactly `aconst_null; astore resource`.
fn exact_null_initializer(facts: &Facts<'_>, init: (u32, u32), slot: u16) -> bool {
    let instructions = facts.bcis(init);
    let [push, store] = instructions.as_slice() else {
        return false;
    };
    facts.op(*push) == Some(&Operation::Push(crate::facts::ConstantValue::Null))
        && facts.op(*store) == Some(&Operation::Store { slot })
}

/// Whether a direct null initializer has both nullable close contours needed to enter the full
/// try-with-resources proof.
fn null_resource_close_outline(facts: &Facts<'_>, row: &ExceptionHandlerFact, slot: u16) -> bool {
    normal_close(facts, row.end_bci, slot).is_some()
        && facts
            .row_handler(row)
            .is_some_and(|entry| close_of_level(facts, &entry, slot).is_ok())
}

/// Whether the store before a row's protected range is a `catch` clause's own **binding**.
///
/// A handler is entered with the exception reference on the operand stack, and the first instruction
/// of the clause's body stores it into the clause's parameter: `astore_1` for `catch (E e)`. The
/// value such a store writes is therefore the reference the handler was entered with, and that is a
/// value **no instruction of this body produced** — the SSA defines it at the edge that carried the
/// exception, once per throw site ([`Definition::Caught`]), or, where several throw sites enter one
/// handler, as the value the handler block's own entry phi names for the operand stack's slot 0.
///
/// Neither is a resource's initialisation, whatever the store's own statement looks like: a header
/// fills its slot from a `new` or an invocation ([`initialises_resource`]), and the run backwards
/// from this store ends at the handler's entry, which is not an instruction. So this is the one
/// place the conservative reading of a statement that cannot be read is *not* what the bytes say,
/// which is what a row whose range begins inside a handler body needs — the nested `try` of
/// `nested(I)I`, whose range `[8, 10)` follows the outer clause's binding store at BCI 7.
///
/// The question is asked of the **store**, not of the slot it fills and not of the local it might
/// have loaded: a `try (AutoCloseable c = e)` inside a `catch` copies the reference through a load,
/// and that store's own value is the load's, which this answers `false` for.
fn handler_binding(facts: &Facts<'_>, store: u32) -> bool {
    let Some(block) = facts.block_of(store) else {
        return false;
    };
    let Some(step) = facts.step(store) else {
        return false;
    };
    step.instruction.reads().iter().any(|(_, read)| {
        let value = facts.resolve(*read);
        match facts.ssa.value(value).def() {
            // The reference one exception edge hands one handler: the edge is its definition, and
            // its own block is the **source** the throw site sits in, not the handler.
            Definition::Caught { .. } => true,
            // Several edges enter this handler, so the reference is the handler block's own entry
            // value for the stack slot the JVM hands it in.
            Definition::Phi { block: phi, slot } => {
                *slot == Slot::Stack(0)
                    && phi == block
                    && facts
                        .handlers
                        .iter()
                        .any(|row| facts.row_handler(row).as_ref() == Some(phi))
            }
            Definition::Instruction { .. } | Definition::Entry { .. } => false,
        }
    })
}

/// The slot the store before a row's protected range copied, when its own statement is one `Load`
/// of another local — the copy `javac` makes of the variable a `try (r)` header names.
///
/// `try (r)` is Java 9 syntax: the header declares no variable of its own, and the compiler keeps
/// the value in a local of its own before the protected range (`aload r; astore copy`). The store is
/// the statement that **ends** there — the same run [`single_statement`] reads for
/// `initialises_resource` — and what it holds is the value of the slot the load read, which is the
/// expression [`initialisation`] grows the header's range to cover: the header writes
/// `try (T copy = r)`, and the close the compiler performs is on the copy of that same value.
///
/// A store of the slot it read (`r = r`) answers `None`: the header would declare the local the
/// source already names, and the copy is not what the compiler wrote.
fn copied_local(facts: &Facts<'_>, store: u32, floor: u32) -> Option<u16> {
    let Some(Operation::Store { slot: stored }) = facts.op(store) else {
        return None;
    };
    let previous = facts
        .previous_bci(store)
        .filter(|previous| *previous >= floor)?;
    let Some(Operation::Load { slot: loaded }) = facts.op(previous) else {
        return None;
    };
    if loaded == stored || !single_statement(facts, (previous, facts.span_end(store)), store) {
        return None;
    }
    Some(*loaded)
}

/// Whether one range is exactly one initialisation: a value expression that ends in the store.
fn single_statement(facts: &Facts<'_>, span: (u32, u32), store: u32) -> bool {
    let Some(step) = facts.step(store) else {
        return false;
    };
    if facts.bcis(span).last() != Some(&store) {
        return false;
    }
    for bci in facts.bcis(span) {
        let Some(instruction) = facts.step(bci) else {
            return false;
        };
        if bci == store {
            continue;
        }
        if instruction.instruction.writes().is_empty() {
            return false;
        }
        for (_, written) in instruction.instruction.writes() {
            let consumed = facts.bcis(span).into_iter().any(|reader| {
                facts.step(reader).is_some_and(|reader| {
                    reader
                        .instruction
                        .reads()
                        .iter()
                        .any(|(_, read)| facts.same(*read, *written))
                })
            });
            if !consumed {
                return false;
            }
        }
    }
    step.instruction.reads().iter().any(|(_, read)| {
        facts.bcis(span).into_iter().any(|producer| {
            facts.step(producer).is_some_and(|producer| {
                producer
                    .instruction
                    .writes()
                    .iter()
                    .any(|(_, written)| facts.same(*written, *read))
            })
        })
    })
}

/// A completed static-field assignment before a protected range is an ordinary statement, not a
/// resource header. Keep the proof within the current straight-line block: every value made by the
/// assignment must be consumed there, and none of its stack values may survive into the range.
fn completed_field_assignment(facts: &Facts<'_>, before: u32, floor: u32, range: u32) -> bool {
    if !matches!(
        facts.op(before),
        Some(Operation::Field {
            access: crate::facts::FieldAccess::Write,
            is_static: true,
            ..
        })
    ) || facts.span_end(before) != range
    {
        return false;
    }
    let mut start = before;
    while let Some(previous) = facts.previous_bci(start).filter(|bci| *bci >= floor) {
        if !single_statement(facts, (previous, range), before) {
            break;
        }
        start = previous;
    }
    if start == before || !single_statement(facts, (start, range), before) {
        return false;
    }
    let statement = facts.bcis((start, range));
    if !statement.iter().all(|bci| {
        facts.step(*bci).is_some_and(|reader| {
            reader
                .instruction
                .reads()
                .iter()
                .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
                .all(|(_, read)| {
                    statement
                        .iter()
                        .take_while(|producer| **producer < *bci)
                        .any(|producer| {
                            facts.step(*producer).is_some_and(|writer| {
                                writer
                                    .instruction
                                    .writes()
                                    .iter()
                                    .any(|(_, written)| facts.same(*written, *read))
                            })
                        })
                })
        })
    }) {
        return false;
    }
    let Some(block) = facts.block_of(before) else {
        return false;
    };
    let Some(entry) = facts.ssa.block(block) else {
        return false;
    };
    if entry
        .entry()
        .iter()
        .any(|(slot, _)| matches!(slot, Slot::Stack(_)))
    {
        return false;
    }
    let mut depth = 0i64;
    for effect in facts
        .ssa
        .effects()
        .instructions()
        .iter()
        .filter(|effect| effect.block() == block && effect.bci() < range)
    {
        depth += i64::from(effect.stack_delta());
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}

/// One level's handler, as the close proof reads it.
struct CloseHandler {
    /// The handler's own instruction range.
    span: (u32, u32),
    /// The row that protects the exceptional path's close.
    guard: ExceptionHandlerFact,
    /// The BCI of the store that keeps the primary.
    primary_bci: u32,
    /// The BCI of the `close` call on the exceptional path.
    close_bci: u32,
    /// The BCI of the load that hands `addSuppressed` the primary, and the BCI of the call itself.
    suppression_bci: u32,
    /// The BCI of the `addSuppressed` call that folds the close's own exception into the primary.
    suppression_call_bci: u32,
    /// The BCI of the `athrow` that rethrows the primary.
    rethrow_bci: u32,
}

/// Proves one level: the handler closes `slot`, suppresses the close's own exception **into the
/// primary** and rethrows the primary.
fn close_handler(
    facts: &Facts<'_>,
    row: &ExceptionHandlerFact,
    slot: u16,
) -> Result<CloseHandler, Cause> {
    let entry = facts
        .row_handler(row)
        .ok_or((Unproven::Handler, row.handler_bci))?;
    let (primary, close_bci, exit) = close_of_level(facts, &entry, slot)?;
    // The close's own row: the innermost one that covers it, and its handler suppresses into the
    // primary. A close the table does not protect at all has no place to record its own failure, and
    // the region is refused.
    let Some(guard) = facts.innermost(close_bci).cloned() else {
        return Err((Unproven::CloseGuard, close_bci));
    };
    let guard_entry = facts
        .row_handler(&guard)
        .ok_or((Unproven::CloseGuard, guard.handler_bci))?;
    let suppression = facts.sequence(&guard_entry);
    let [
        (_, Some(Operation::Store { slot: raised })),
        (_, Some(Operation::Load { slot: suppressed })),
        (_, Some(Operation::Load { slot: argument })),
        (_, Some(Operation::Invoke(suppress))),
    ] = suppression.as_slice()
    else {
        return Err((Unproven::Suppressed, guard_entry.bci()));
    };
    if *suppressed != primary || *argument != *raised {
        return Err((Unproven::Suppressed, guard_entry.bci()));
    }
    if suppress.name() != "addSuppressed" || suppress.descriptor() != "(Ljava/lang/Throwable;)V" {
        return Err((Unproven::Suppressed, guard_entry.bci()));
    }
    if facts.view.successor_ids(&guard_entry) != vec![exit.clone()] {
        return Err((Unproven::Suppressed, guard_entry.bci()));
    }
    let tail = facts.sequence(&exit);
    let [
        (_, Some(Operation::Load { slot: rethrown })),
        (_, Some(Operation::Throw)),
    ] = tail.as_slice()
    else {
        return Err((Unproven::Primary, exit.bci()));
    };
    if *rethrown != primary || !facts.view.successor_ids(&exit).is_empty() {
        return Err((Unproven::Primary, exit.bci()));
    }
    let last = facts
        .in_block(&exit)
        .last()
        .ok_or((Unproven::Primary, exit.bci()))?;
    Ok(CloseHandler {
        span: (entry.bci(), facts.span_end(last.bci())),
        guard,
        primary_bci: entry.bci(),
        close_bci,
        suppression_bci: suppression[1].0,
        suppression_call_bci: suppression[3].0,
        rethrow_bci: facts
            .in_block(&exit)
            .last()
            .map(|last| last.bci())
            .unwrap_or(exit.bci()),
    })
}

/// Where one level's handler closes its resource: the primary it kept, the `close` call, and the
/// block the run leaves through.
///
/// `javac` writes two shapes for the close, and both say the same thing. When the resource's own
/// initialisation **could** be null the close is guarded by a test and the handler reads
/// `astore p; aload r; ifnull L` with the close (and its own guard) in the other successor; when the
/// initialisation **proves** the resource non-null (`new Res(…)`) the compiler writes no test at
/// all and the handler reads `astore p; aload r; invokevirtual close; goto L`. The second shape is
/// the same level read one instruction shorter, and the rest of the proof — the row that protects
/// the close, the suppression, the rethrow — is read identically from both.
fn close_of_level(
    facts: &Facts<'_>,
    entry: &CanonicalBlockId,
    slot: u16,
) -> Result<(u16, u32, CanonicalBlockId), Cause> {
    let head = facts.sequence(entry);
    match head.as_slice() {
        [
            (_, Some(Operation::Store { slot: primary })),
            (head_exception, Some(Operation::Load { slot: loaded })),
            (
                _,
                Some(Operation::Comparison {
                    op: CompareOp::JumpIfNull,
                    target,
                }),
            ),
        ] => {
            if *loaded != slot {
                return Err((Unproven::CloseTarget, *head_exception));
            }
            let successors = facts.view.successor_ids(entry);
            let exit = facts
                .block_at(*target)
                .ok_or((Unproven::Handler, entry.bci()))?;
            let Some(close_block) = successors.iter().find(|block| **block != exit).cloned() else {
                return Err((Unproven::Handler, entry.bci()));
            };
            if successors.len() != 2 {
                return Err((Unproven::Handler, entry.bci()));
            }
            let close = facts.sequence(&close_block);
            let [
                (load_bci, Some(Operation::Load { slot: close_slot })),
                (call_bci, Some(Operation::Invoke(called))),
                (_, Some(Operation::Transfer)),
            ] = close.as_slice()
            else {
                return Err((Unproven::Handler, close_block.bci()));
            };
            if *close_slot != slot || called.name() != "close" || called.descriptor() != "()V" {
                return Err((Unproven::CloseTarget, *call_bci));
            }
            let (Some(load), Some(call)) = (facts.step(*load_bci), facts.step(*call_bci)) else {
                return Err((Unproven::Handler, close_block.bci()));
            };
            if !receiver_is(facts, call.instruction, load.instruction) {
                return Err((Unproven::CloseTarget, *call_bci));
            }
            if facts.view.successor_ids(&close_block) != vec![exit.clone()] {
                return Err((Unproven::Handler, close_block.bci()));
            }
            Ok((*primary, *call_bci, exit))
        }
        [
            (_, Some(Operation::Store { slot: primary })),
            (head_exception, Some(Operation::Load { slot: loaded })),
            (call_bci, Some(Operation::Invoke(called))),
            (_, Some(Operation::Transfer)),
        ] => {
            if *loaded != slot {
                return Err((Unproven::CloseTarget, *head_exception));
            }
            if called.name() != "close" || called.descriptor() != "()V" {
                return Err((Unproven::CloseTarget, *call_bci));
            }
            let (Some(load), Some(call)) = (facts.step(*head_exception), facts.step(*call_bci))
            else {
                return Err((Unproven::Handler, entry.bci()));
            };
            if !receiver_is(facts, call.instruction, load.instruction) {
                return Err((Unproven::CloseTarget, *call_bci));
            }
            // The transfer the handler ends in is where the run leaves; the close is the last thing
            // that runs before it.
            let successors = facts.view.successor_ids(entry);
            let [exit] = successors.as_slice() else {
                return Err((Unproven::Handler, entry.bci()));
            };
            Ok((*primary, *call_bci, exit.clone()))
        }
        _ => Err((Unproven::Handler, entry.bci())),
    }
}

/// One `synchronized` handler, as the monitor proof reads it.
struct MonitorHandler {
    /// The handler's own instruction range.
    span: (u32, u32),
    /// The block the handler's instructions are in.
    entry: CanonicalBlockId,
    /// The BCI of the exit the handler performs.
    exit_bci: u32,
}

/// Proves that the row's handler leaves the monitor before it rethrows what it caught.
fn monitor_handler(
    facts: &Facts<'_>,
    row: &ExceptionHandlerFact,
    lock: u16,
) -> Result<MonitorHandler, Cause> {
    let entry = facts
        .row_handler(row)
        .ok_or((Unproven::Handler, row.handler_bci))?;
    let sequence = facts.sequence(&entry);
    let [
        (_, Some(Operation::Store { slot: primary })),
        (_, Some(Operation::Load { slot: loaded })),
        (exit_bci, Some(Operation::Monitor { enter: false })),
        (_, Some(Operation::Load { slot: rethrown })),
        (_, Some(Operation::Throw)),
    ] = sequence.as_slice()
    else {
        return Err((Unproven::Monitor, entry.bci()));
    };
    if *loaded != lock || *rethrown != *primary {
        return Err((Unproven::Monitor, entry.bci()));
    }
    if !facts.view.successor_ids(&entry).is_empty() {
        return Err((Unproven::Monitor, entry.bci()));
    }
    let last = facts
        .in_block(&entry)
        .last()
        .ok_or((Unproven::Monitor, entry.bci()))?;
    Ok(MonitorHandler {
        span: (entry.bci(), facts.span_end(last.bci())),
        entry,
        exit_bci: *exit_bci,
    })
}

/// Whether the row's handler is the `finally` copy: store the exception, run code, rethrow it — and
/// nothing anywhere in the handler that closes, suppresses or leaves a monitor.
fn finally_copy(facts: &Facts<'_>, row: &ExceptionHandlerFact) -> Option<u32> {
    // javac emits the exceptional copy of a `finally` only for an any row. A named handler can
    // have the same store/body/load/throw shape while being an ordinary catch (including precise
    // rethrow), so its type must be checked before classifying the handler's bytecode shape.
    if row.catch_type_index.is_some() {
        return None;
    }
    let entry = facts.row_handler(row)?;
    // The copy runs straight: one block, or a chain of blocks, ending in the `athrow`.
    let mut blocks = vec![entry.clone()];
    let mut current = entry.clone();
    loop {
        let successors = facts.view.successor_ids(&current);
        match successors.as_slice() {
            [] => break,
            [next] => {
                if blocks.contains(next) {
                    return None;
                }
                blocks.push(next.clone());
                current = next.clone();
            }
            _ => return None,
        }
    }
    let mut instructions: Vec<&SsaInstruction> = Vec::new();
    for block in &blocks {
        instructions.extend(facts.in_block(block).iter());
    }
    let (first, rest) = instructions.split_first()?;
    let Some(Operation::Store { slot: primary }) = facts.op(first.bci()) else {
        return None;
    };
    let primary = *primary;
    let (last, middle) = rest.split_last()?;
    if facts.op(last.bci()) != Some(&Operation::Throw) {
        return None;
    }
    let (load, body) = middle.split_last()?;
    if facts.op(load.bci()) != Some(&Operation::Load { slot: primary }) || body.is_empty() {
        return None;
    }
    let runs_code = body.iter().any(|instruction| {
        matches!(
            facts.op(instruction.bci()),
            Some(Operation::Invoke(_))
                | Some(Operation::Field { .. })
                | Some(Operation::CheckCast { .. })
        )
    });
    if !runs_code {
        return None;
    }
    let closes = instructions
        .iter()
        .any(|instruction| match facts.op(instruction.bci()) {
            Some(Operation::Invoke(called)) => {
                called.name() == "close" || called.name() == "addSuppressed"
            }
            Some(Operation::Monitor { .. }) => true,
            _ => false,
        });
    (!closes).then_some(entry.bci())
}

/// The physical pieces read by the narrow copy proof. No region owns them until a later step
/// turns this certificate into a `Plan`.
#[derive(Debug)]
struct FinallyCopyProof {
    row_ordinal: u32,
    protected: (u32, u32),
    normal_cleanup: (u32, u32),
    handler_cleanup: (u32, u32),
    saved_return: (u32, u32),
    primary: (u32, u32, u32),
    owned: Vec<CanonicalBlockId>,
    join: Option<CanonicalBlockId>,
    origins: Vec<u32>,
}

/// Each stack operand must come from an earlier instruction of this copy. The returned producer
/// ordinals make SSA value names local to the copy, so two copies can be compared structurally
/// without accidentally equating unrelated physical `ValueId`s.
fn cleanup_sequence(facts: &Facts<'_>, copy: &[u32]) -> Option<Vec<(Operation, Vec<usize>)>> {
    if copy.is_empty() || copy.len() > 32 {
        return None;
    }
    let mut normalized = Vec::new();
    let mut effects = 0;
    let mut calls = 0;
    for (ordinal, bci) in copy.iter().enumerate() {
        let operation = facts.op(*bci)?;
        let instruction = facts.step(*bci)?.instruction;
        match operation {
            Operation::Push(_) => {}
            Operation::Invoke(_) => {
                effects += 1;
                calls += 1;
            }
            Operation::Field { .. } => effects += 1,
            Operation::Other if instruction.opcode() == 0x57 => {}
            _ => return None,
        }
        let mut producers = Vec::new();
        for (_, read) in stack_operands(instruction) {
            let producer = copy[..ordinal]
                .iter()
                .enumerate()
                .rev()
                .find_map(|(index, bci)| {
                    facts
                        .step(*bci)?
                        .instruction
                        .writes()
                        .iter()
                        .any(|(slot, written)| {
                            matches!(slot, Slot::Stack(_)) && facts.same(*written, read)
                        })
                        .then_some(index)
                })?;
            producers.push(producer);
        }
        normalized.push((operation.clone(), producers));
    }
    if effects == 0 || calls > 1 {
        return None;
    }
    for (ordinal, bci) in copy.iter().enumerate() {
        let stack_outputs = facts
            .step(*bci)?
            .instruction
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .count();
        if stack_outputs > 1
            || (stack_outputs == 1
                && normalized
                    .iter()
                    .map(|(_, inputs)| inputs.iter().filter(|input| **input == ordinal).count())
                    .sum::<usize>()
                    != 1)
        {
            return None;
        }
    }
    Some(normalized)
}

/// Prove only a straight return/handler pair, before any region ownership or emission. The row's
/// half-open range is checked first: a cleanup call caught by its own handler can run twice.
fn prove_finally_copy(
    facts: &mut Facts<'_>,
    row: &ExceptionHandlerFact,
) -> Result<Option<FinallyCopyProof>, StopReason> {
    let Some(handler) = facts.row_handler(row) else {
        return Ok(None);
    };
    if row.catch_type_index.is_some() || row.start_bci >= row.end_bci {
        return Ok(None);
    }
    // This slice does not infer a `try` boundary after a call that may throw. In the narrowed
    // snapshot variant that call is `mark(1)`; moving the row start past it loses cleanup.
    for before in facts.bcis((0, row.start_bci)) {
        facts.charge(before)?;
        if matches!(facts.op(before), Some(Operation::Invoke(_))) {
            return Ok(None);
        }
    }
    let before_handler = facts.bcis((row.start_bci, handler.bci()));
    let exceptional: Vec<u32> = facts
        .in_block(&handler)
        .iter()
        .map(SsaInstruction::bci)
        .collect();
    for bci in before_handler.iter().chain(&exceptional) {
        facts.charge(*bci)?;
    }
    let Some((&normal_return, normal_middle)) = before_handler.split_last() else {
        return Ok(None);
    };
    let Some((&return_load, before_return_load)) = normal_middle.split_last() else {
        return Ok(None);
    };
    let Some((&primary_store, handler_tail)) = exceptional.split_first() else {
        return Ok(None);
    };
    let Some((&rethrow, handler_middle)) = handler_tail.split_last() else {
        return Ok(None);
    };
    let Some((&primary_load, handler_cleanup)) = handler_middle.split_last() else {
        return Ok(None);
    };
    let Some(cleanup_start) = before_return_load.len().checked_sub(handler_cleanup.len()) else {
        return Ok(None);
    };
    let normal_cleanup = &before_return_load[cleanup_start..];
    let Some(save) = cleanup_start
        .checked_sub(1)
        .and_then(|index| before_return_load.get(index))
        .copied()
    else {
        return Ok(None);
    };
    // Find the normal copy from the return suffix, independently of the exception row's end.
    // This lets the widened [0,23) row reveal that BCI 20 lies *inside* its own protection.
    if normal_cleanup.is_empty() || handler_cleanup.is_empty() {
        return Ok(None);
    }
    if normal_cleanup
        .iter()
        .chain(handler_cleanup)
        .any(|bci| row.start_bci <= *bci && *bci < row.end_bci)
    {
        return Ok(None);
    }
    if row.end_bci != normal_cleanup[0] {
        return Ok(None);
    }
    // A branch target inside either copy splits a canonical block. Requiring the entire normal
    // suffix in one block (the handler suffix was read from one block above) therefore prevents
    // an extra predecessor from entering after some cleanup instructions have already run.
    let normal_block = facts.block_of(normal_cleanup[0]);
    if normal_block.is_none()
        || normal_cleanup
            .iter()
            .chain([&return_load, &normal_return])
            .any(|bci| facts.block_of(*bci) != normal_block)
    {
        return Ok(None);
    }
    let (
        Some(Operation::Store { slot: saved }),
        Some(Operation::Load { slot: returned }),
        Some(Operation::Store { slot: primary }),
        Some(Operation::Load { slot: reloaded }),
    ) = (
        facts.op(save),
        facts.op(return_load),
        facts.op(primary_store),
        facts.op(primary_load),
    )
    else {
        return Ok(None);
    };
    if saved != returned
        || primary != reloaded
        || facts.op(normal_return) != Some(&Operation::Return)
        || facts.op(rethrow) != Some(&Operation::Throw)
        || !facts.view.successor_ids(&handler).is_empty()
    {
        return Ok(None);
    }
    let (Some(normal_code), Some(handler_code)) = (
        cleanup_sequence(facts, normal_cleanup),
        cleanup_sequence(facts, handler_cleanup),
    ) else {
        return Ok(None);
    };
    if normal_code != handler_code {
        return Ok(None);
    }
    // Every protected normal edge stays protected or reaches the one normal copy. A throw has no
    // normal successor and is handled by the row. A return inside the range would skip cleanup.
    let mut protected_blocks = facts.blocks_in((row.start_bci, row.end_bci));
    protected_blocks.sort_by_key(CanonicalBlockId::bci);
    for block in &protected_blocks {
        facts.charge(block.bci())?;
        let mut last_protected = None;
        for instruction in facts.in_block(block) {
            if instruction.bci() < row.start_bci || instruction.bci() >= row.end_bci {
                continue;
            }
            last_protected = Some(instruction.bci());
            if facts.op(instruction.bci()) == Some(&Operation::Return) {
                return Ok(None);
            }
        }
        let successors = facts.view.successor_ids(block);
        if successors.is_empty()
            && facts.end_of(block) <= row.end_bci
            && last_protected.is_some_and(|last| facts.op(last) != Some(&Operation::Throw))
        {
            return Ok(None);
        }
        for successor in successors {
            if !protected_blocks.contains(&successor) && successor.bci() != row.end_bci {
                return Ok(None);
            }
        }
    }
    for block in facts.canonical.blocks() {
        facts.charge(block.id().bci())?;
        for successor in facts.view.successor_ids(block.id()) {
            if successor == handler && block.id() != &handler {
                return Ok(None);
            }
            if normal_block == Some(&successor)
                && block.id() != &successor
                && !protected_blocks.contains(block.id())
            {
                return Ok(None);
            }
        }
    }
    // The return and rethrow consume precisely the loads of the saved value and primary.
    let (
        Some(saved_step),
        Some(return_load_step),
        Some(return_step),
        Some(primary_step),
        Some(primary_load_step),
        Some(throw_step),
    ) = (
        facts.step(save),
        facts.step(return_load),
        facts.step(normal_return),
        facts.step(primary_store),
        facts.step(primary_load),
        facts.step(rethrow),
    )
    else {
        return Ok(None);
    };
    let one_link = |producer: &SsaInstruction, consumer: &SsaInstruction| {
        let produced = producer
            .writes()
            .iter()
            .find(|(slot, _)| matches!(slot, Slot::Stack(_)));
        let consumed = stack_operands(consumer);
        produced.is_some_and(|(_, value)| consumed.len() == 1 && facts.same(*value, consumed[0].1))
    };
    if !one_link(return_load_step.instruction, return_step.instruction)
        || !one_link(primary_load_step.instruction, throw_step.instruction)
        || !primary_step
            .instruction
            .writes()
            .iter()
            .any(|(_, written)| {
                primary_load_step
                    .instruction
                    .reads()
                    .iter()
                    .any(|(_, read)| facts.same(*written, *read))
            })
        || !saved_step.instruction.writes().iter().any(|(_, written)| {
            return_load_step
                .instruction
                .reads()
                .iter()
                .any(|(_, read)| facts.same(*written, *read))
        })
    {
        return Ok(None);
    }
    // No other exception-table row may intercept a protected instruction or either cleanup.
    for bci in facts
        .bcis((row.start_bci, row.end_bci))
        .into_iter()
        .chain(normal_cleanup.iter().copied())
        .chain(handler_cleanup.iter().copied())
    {
        facts.charge(bci)?;
        if facts
            .covering(bci)
            .iter()
            .any(|other| other.ordinal != row.ordinal)
        {
            return Ok(None);
        }
    }
    let mut owned = facts.blocks_in((row.start_bci, facts.span_end(normal_return)));
    if !owned.contains(&handler) {
        owned.push(handler);
    }
    owned.sort_by_key(CanonicalBlockId::bci);
    let mut origins = facts.bcis((row.start_bci, facts.span_end(rethrow)));
    origins.sort_unstable();
    origins.dedup();
    Ok(Some(FinallyCopyProof {
        row_ordinal: row.ordinal,
        protected: (row.start_bci, row.end_bci),
        normal_cleanup: (
            normal_cleanup[0],
            facts.span_end(*normal_cleanup.last().unwrap()),
        ),
        handler_cleanup: (
            handler_cleanup[0],
            facts.span_end(*handler_cleanup.last().unwrap()),
        ),
        saved_return: (save, normal_return),
        primary: (primary_store, primary_load, rethrow),
        owned,
        join: None,
        origins,
    }))
}

#[cfg(test)]
mod finally_copy_tests {
    use super::*;
    use jarde_jvm::engine::analyze_method_ir;
    use jarde_jvm::environment::ResolutionEnvironment;
    use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
    use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
    use jarde_reader::budget::{CancellationToken, Limits};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
        MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
        RuntimeView,
    };

    fn limits() -> Limits {
        Limits {
            input_bytes: 1 << 20,
            archive_entries: 1_000,
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

    fn proof(class: &[u8], name: &str) -> Option<FinallyCopyProof> {
        proof_with_row(class, name, false)
    }

    fn proof_with_row(class: &[u8], name: &str, competing: bool) -> Option<FinallyCopyProof> {
        probe(class, name, competing, None).unwrap()
    }

    fn probe(
        class: &[u8],
        name: &str,
        competing: bool,
        stop: Option<&str>,
    ) -> Result<Option<FinallyCopyProof>, StopReason> {
        let mut budget = Budget::new(limits());
        let snapshot =
            ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget).unwrap();
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
        let method = PhysicalMethodId {
            owner: definition,
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(b"()I".to_vec()),
        };
        let domain = LoadDomain {
            loader: LoaderId("app".to_string()),
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
            method,
            stages: AnalysisStage::ALL.to_vec(),
        };
        let analyzed = analyze_method_ir(&[snapshot], &request, &mut budget).unwrap();
        let ir = analyzed.ir();
        let canonical = ir.canonical().unwrap();
        let ssa = ir.ssa().unwrap();
        let code = ir.code().unwrap();
        let ops = Operations::of(code, ir.constant_pool());
        let view = NormalFlowView::build(canonical, &mut budget).unwrap();
        let mut rows = code.exception_handlers.clone();
        if competing {
            let mut other = rows[0].clone();
            other.ordinal = rows.len() as u32;
            other.handler_bci += 1;
            rows.push(other);
        }
        let row = rows.first().unwrap();
        let mut proof_limits = limits();
        if stop == Some("budget") {
            proof_limits.analysis_steps = 0;
        }
        let token = CancellationToken::new();
        if stop == Some("cancel") {
            token.cancel();
        }
        let mut proof_budget = Budget::with_cancellation_token(proof_limits, token);
        let mut facts = Facts::new(canonical, &view, ssa, &ops, &rows, &mut proof_budget);
        prove_finally_copy(&mut facts, row)
    }

    fn resource_plan(class: &[u8], name: &str) -> Result<Plan, String> {
        let mut budget = Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
            .map_err(|error| format!("snapshot: {error:?}"))?;
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
        let method = PhysicalMethodId {
            owner: definition,
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(b"(I)I".to_vec()),
        };
        let domain = LoadDomain {
            loader: LoaderId("app".to_string()),
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
            method,
            stages: AnalysisStage::ALL.to_vec(),
        };
        let analyzed = analyze_method_ir(&[snapshot], &request, &mut budget)
            .map_err(|error| format!("analysis: {error:?}"))?;
        let ir = analyzed.ir();
        let canonical = ir.canonical().unwrap();
        let ssa = ir.ssa().unwrap();
        let code = ir.code().unwrap();
        let ops = Operations::of(code, ir.constant_pool());
        let view = NormalFlowView::build(canonical, &mut budget).unwrap();
        let current = canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() == 0)
            .map(|block| block.id())
            .ok_or_else(|| "entry block missing".to_string())?;
        let rows = code.exception_handlers.clone();
        let mut facts = Facts::new(canonical, &view, ssa, &ops, &rows, &mut budget);
        let row = rows
            .first()
            .ok_or_else(|| "resource row missing".to_string())?;
        twr(&mut facts, &crate::pass::JAVA_8, current, row).map_err(|_| "TWR refused".to_string())
    }

    #[test]
    fn a_terminal_saved_return_is_part_of_the_resource_plan() {
        let class =
            include_bytes!("../../../tests/fixtures/p3-multi-resource-twr/TwrReturnTail.class");
        let plan = resource_plan(class, "runSaved").expect("the TWR structure is recognized");
        assert!(matches!(
            plan.shape(),
            Shape::Resources {
                returns: Some(20),
                ..
            }
        ));
    }

    #[test]
    fn implicit_cleanup_has_a_physical_copy_certificate() {
        let class = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-22/finally-completion/implicit-cleanup/ImplicitCleanup.class"
        );
        let proved = proof(class, "run").expect("the straight call copies match");
        assert_eq!(proved.protected, (0, 20));
        assert_eq!(proved.normal_cleanup, (20, 23));
        assert_eq!(proved.handler_cleanup, (26, 29));
        assert_eq!(proved.saved_return, (19, 24));
        assert_eq!(proved.primary, (25, 29, 30));
        assert_eq!(proved.row_ordinal, 0);
        assert!(proved.join.is_none());
        assert!(proved.owned.iter().any(|block| block.bci() == 25));
        assert!(proved.origins.contains(&20) && proved.origins.contains(&26));
    }

    #[test]
    fn widened_range_cannot_reenter_the_proved_cleanup() {
        let class = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-24/finally-range-widened/ImplicitCleanup.class"
        );
        assert!(proof(class, "run").is_none());
    }

    #[test]
    fn a_competing_row_refuses_the_same_otherwise_proved_copy() {
        let class = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-22/finally-completion/implicit-cleanup/ImplicitCleanup.class"
        );
        assert!(proof(class, "run").is_some());
        assert!(proof_with_row(class, "run", true).is_none());
    }

    #[test]
    fn changed_argument_or_missing_coverage_refuses_the_copy() {
        let baseline = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-22/finally-completion/snapshot-boundaries/baseline/CleanupBoundaries.class"
        );
        let divergent = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-22/finally-completion/snapshot-boundaries/copy-divergence/CleanupBoundaries.class"
        );
        let narrowed = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-22/finally-completion/snapshot-boundaries/range-narrowed/CleanupBoundaries.class"
        );
        let proved = proof(baseline, "snapshotReturn").expect("the matching snapshot copies prove");
        assert_eq!(proved.protected, (5, 14));
        assert_eq!(proved.normal_cleanup, (14, 24));
        assert_eq!(proved.handler_cleanup, (27, 37));
        assert_eq!(proved.saved_return, (13, 25));
        assert!(proof(divergent, "snapshotReturn").is_none());
        assert!(proof(narrowed, "snapshotReturn").is_none());
    }

    #[test]
    fn different_cleanup_member_refuses_the_copy() {
        let baseline =
            include_bytes!("../../../tests/fixtures/p3-finally-proof/TargetCopies.class");
        let changed = include_bytes!(
            "../../../tests/fixtures/p3-finally-proof/TargetCopiesDifferentTarget.class"
        );
        assert!(proof(baseline, "run").is_some());
        assert!(proof(changed, "run").is_none());
    }

    #[test]
    fn repeated_cleanup_calls_stay_outside_the_single_call_slice() {
        let class = include_bytes!("../../../tests/fixtures/p3-finally-proof/RepeatedCopies.class");
        assert!(proof(class, "run").is_none());
    }

    #[test]
    fn an_extra_cleanup_result_consumer_is_refused() {
        let class = include_bytes!("../../../tests/fixtures/p3-finally-proof/ExtraConsumer.class");
        assert!(proof(class, "run").is_none());
    }

    #[test]
    fn proof_propagates_the_existing_budget_and_cancellation_stops() {
        let class = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-22/finally-completion/implicit-cleanup/ImplicitCleanup.class"
        );
        assert!(matches!(
            probe(class, "run", false, Some("budget")),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            probe(class, "run", false, Some("cancel")),
            Err(StopReason::Cancelled { .. })
        ));
    }
}

/// Examines one block the walk cannot leave through the normal flow.
///
/// The canonical graph fuses straight-line code, so the statement is **not** a block: the resource's
/// initialisation, the guarded body and the first close of the normal path are one node. What this
/// function reads is instruction ranges, and what it returns is the range of instructions the
/// statement's own block keeps for itself (`lead`), the range the body is written from, and every
/// block the statement claims.
#[allow(clippy::too_many_arguments)]
pub(crate) fn examine(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    ops: &Operations,
    handlers: &[ExceptionHandlerFact],
    profile: &crate::pass::RecoveryProfile,
    current: &CanonicalBlockId,
    budget: &mut Budget,
) -> Result<Verdict, StopReason> {
    let mut facts = Facts::new(canonical, view, ssa, ops, handlers, budget);
    facts.charge(current.bci())?;
    match guarded(&mut facts, profile, current)? {
        Some(verdict) => Ok(verdict),
        None => Ok(Verdict::NotGuarded),
    }
}

/// What the guarded rules of P3 2.4 say about one block: a verdict, or `None` when no rule owns it.
///
/// The order is the examination's own: the `synchronized` shape is asked first because its header is
/// a `monitorenter` and nothing else, and the `try` header second. "No rule owns it" is the `try`
/// header's own [`Verdict::NotGuarded`] as well: a block neither rule claims or refuses is a block
/// whose guarded shapes are *not here*, which is what the `try`/`catch` shape is read on.
fn guarded(
    facts: &mut Facts<'_>,
    profile: &crate::pass::RecoveryProfile,
    current: &CanonicalBlockId,
) -> Result<Option<Verdict>, StopReason> {
    if let Some(verdict) = monitor(facts, profile, current)? {
        return Ok(Some(verdict));
    }
    match resources(facts, profile, current)? {
        Verdict::NotGuarded => Ok(None),
        verdict => Ok(Some(verdict)),
    }
}

/// One `catch` clause of a `try` the walk presents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CatchSite {
    /// Constant-pool indexes of the `catch` types this clause names, in exception-table order.
    ///
    /// One entry is an ordinary clause. Several entries are the **multi-catch** the table states:
    /// consecutive rows that name different classes and reach the *same* handler are one clause, and
    /// the compiler writes them `catch (A | B n)`. Reading them as one clause is what keeps the
    /// handler's body from being walked — and written — once per row.
    pub(crate) type_indices: Vec<u16>,
    /// The canonical block the rows' handler entry maps to.
    pub(crate) handler: CanonicalBlockId,
    /// The local slot the handler's own first instruction stores the caught exception into: the
    /// clause's parameter.
    pub(crate) parameter: u16,
}

/// One `try`/`catch` statement: where its clauses are and where the code after it begins.
///
/// This is not a guarded shape a rule of P3 2.4 proves: a `try` whose rows name their `catch` types
/// is presented as the structure the table states, and the walk recovers both the protected range and
/// every handler body as ordinary regions ([`crate::region`]).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Catches {
    /// One site per clause (per handler entry), in exception-table order.
    pub(crate) sites: Vec<CatchSite>,
    /// The block the code after the `try` begins at, when the protected range is followed by a
    /// transfer; `None` when nothing follows it (every path out of the range leaves the method).
    pub(crate) join: Option<CanonicalBlockId>,
    /// The instructions of the block the statement begins in that are written **before** the `try`:
    /// the half-open range from that block's own start to the protected range's start. Empty where
    /// the range begins where its block does, which is where `javac` puts it whenever no statement
    /// runs before the `try` in the same straight-line block.
    ///
    /// The canonical graph fuses straight-line code, so the statement's own block may hold the
    /// instructions the block ran *before* the range began — `int x = 1;` in front of `try { … }`.
    /// They are not part of the protected range, so they are written before the `try` and the body
    /// does not write them again ([`crate::region::Region::Try`]).
    pub(crate) lead: (u32, u32),
    /// The `try` this statement's own body **is**, when two protected ranges that begin at one
    /// instruction nest: the narrower range's statement, with its own clauses and join.
    ///
    /// The wider statement's body is then that inner statement and not a second clause of its own —
    /// [`crate::region::Region::Try`]'s body is a region already, so the nesting needs no region kind
    /// — and the levels are built from the inside out ([`crate::region::Walker::try_region`]).
    /// `None` is the ordinary statement of one protected range.
    pub(crate) inner: Option<Box<Catches>>,
}

/// Examines one block as the `try` of a `try`/`catch`: the rows that name `catch` types and protect
/// a range beginning in it.
///
/// `None` is the answer for everything this shape is not, and every one of them is a *reason the
/// shape was refused*, not a silent skip:
///
/// * no row begins in this block, or the rows that do name no `catch` type;
/// * the rows that begin in the block do not all begin at **one instruction**: several clauses of
///   one `try` share their range's start, and two statements merely written in one straight-line
///   block are not one statement's ranges;
/// * the rows that begin at one instruction declare more than two protected ranges: two nest (the
///   narrower one inside the wider one, [`nests`]), and a third is a shape this statement does not
///   state;
/// * a guarded rule of P3 2.4 owns the region — it claimed it, or refused it as its own shape. A
///   `try`-with-resources this build cannot prove is never spelled as a user `catch`;
/// * a row's handler stores the exception into no local: a clause has no parameter to write, and
///   inventing one is exactly what this layer may not do.
///
/// The rows are the ones that begin in this block, which is not the same as the ones that begin
/// *where the block does*: the block may hold the statement before the `try` too, and then the range
/// begins inside it. What that costs is a [`Catches::lead`], written before the statement — never a
/// body that swallows the instructions the range does not protect.
///
/// One clause per **handler**, not per row: consecutive rows that name different classes and reach
/// the same handler entry are the multi-catch the compiler writes `catch (A | B n)`, and the body of
/// that handler is one body. Rows that reach *different* handlers are different clauses, in table
/// order — which is the order the JVM dispatches in, and therefore the order the text must state.
/// The clauses of the nested level are read the same way, so each level may state several of them.
#[allow(clippy::too_many_arguments)]
pub(crate) fn catches(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    ops: &Operations,
    handlers: &[ExceptionHandlerFact],
    profile: &crate::pass::RecoveryProfile,
    current: &CanonicalBlockId,
    budget: &mut Budget,
) -> Result<Option<Catches>, StopReason> {
    // The block's own last instruction: the same reading the region walk's `starts_catch` takes, so
    // the two agree on which blocks hold the head of a `try`. The rows are read before the facts are
    // built: a block whose rows are none of this shape's costs nothing.
    let last = ssa
        .block(current)
        .and_then(|block| block.instructions().last())
        .map(|instruction| instruction.bci());
    let rows_here: Vec<&ExceptionHandlerFact> = handlers
        .iter()
        .filter(|row| {
            row.catch_type_index.is_some()
                && row.start_bci >= current.bci()
                && last.is_some_and(|last| row.start_bci <= last)
        })
        .collect();
    if rows_here.is_empty() {
        return Ok(None);
    }
    // How one set of rows reads as **clauses**: every range begins at one instruction, and there are
    // at most two ends — one `try`, or the nesting of P3 2.5 (the narrower range inside the wider
    // one). Anything else is no statement this function states.
    let clauses_of = |rows: &[&ExceptionHandlerFact]| -> Option<(u32, Vec<u32>)> {
        let start = rows.first()?.start_bci;
        if rows.iter().any(|row| row.start_bci != start) {
            return None;
        }
        let mut ends: Vec<u32> = rows.iter().map(|row| row.end_bci).collect();
        ends.sort_unstable();
        ends.dedup();
        (ends.len() <= 2).then_some((start, ends))
    };
    // What the guarded rules of P3 2.4 say about this block decides which of its rows are **clauses**
    // at all. A rule that **claimed** the block wrote a `try (…)` statement here, and the rows that
    // reach the handlers it proved are the compiler's own — the synthetic `Throwable` rows of that
    // header's cleanup. They are no `catch` a source wrote, and the clauses of the statement are the
    // rows that reach *other* handlers: the user's `catch`, which the walk writes around the
    // statement. A rule that **refused** the block keeps today's answer — a `try`-with-resources
    // this build cannot prove is never spelled as a user `catch` — and where no rule owns the block
    // every row that begins here is a clause, exactly as before.
    //
    // The two orders differ in what they cost, so both are stated. A block whose rows already read
    // as clauses is asked the rules in today's order, and for today's bill; a block whose rows hold
    // the union of the header's own ranges and the clause's is asked **first**, because only the
    // rule can say which of them are its own, and the reading left over is what this statement
    // writes. Both answer `None` for a block the rules own and do not present.
    let mut facts = Facts::new(canonical, view, ssa, ops, handlers, budget);
    let (named, (start, ends)): (Vec<&ExceptionHandlerFact>, (u32, Vec<u32>)) =
        match clauses_of(&rows_here) {
            Some(plain) => {
                facts.charge(current.bci())?;
                match guarded(&mut facts, profile, current)? {
                    Some(Verdict::Claimed(plan)) => {
                        let clauses: Vec<&ExceptionHandlerFact> = rows_here
                            .iter()
                            .copied()
                            .filter(|row| !plan.facts().contains(&row.handler_bci))
                            .collect();
                        let Some(reading) = clauses_of(&clauses) else {
                            return Ok(None);
                        };
                        if reading.1 == plain.1 && clauses.len() == rows_here.len() {
                            // The rule claimed a block whose rows read as clauses already, without
                            // taking one of them for itself: no `try (…)` header is part of this
                            // reading, and the block keeps today's answer.
                            return Ok(None);
                        }
                        (clauses, reading)
                    }
                    Some(_) => return Ok(None),
                    None => (rows_here, plain),
                }
            }
            None => {
                let Some(Verdict::Claimed(plan)) = guarded(&mut facts, profile, current)? else {
                    return Ok(None);
                };
                let clauses: Vec<&ExceptionHandlerFact> = rows_here
                    .iter()
                    .copied()
                    .filter(|row| !plan.facts().contains(&row.handler_bci))
                    .collect();
                let Some(reading) = clauses_of(&clauses) else {
                    return Ok(None);
                };
                (clauses, reading)
            }
        };
    let lead = (current.bci(), start);
    let rows_of = |end: u32| -> Vec<&ExceptionHandlerFact> {
        named
            .iter()
            .copied()
            .filter(|row| row.end_bci == end)
            .collect()
    };
    match ends.as_slice() {
        [end] => {
            let Some(sites) = clause_sites(&facts, &rows_of(*end)) else {
                return Ok(None);
            };
            Ok(Some(Catches {
                sites,
                join: join_after(&facts, *end),
                lead,
                inner: None,
            }))
        }
        [inner_end, outer_end] => {
            let (inner, outer) = (rows_of(*inner_end), rows_of(*outer_end));
            if !nests(&inner, &outer, handlers) {
                return Ok(None);
            }
            let Some(inner_sites) = clause_sites(&facts, &inner) else {
                return Ok(None);
            };
            let Some(outer_sites) = clause_sites(&facts, &outer) else {
                return Ok(None);
            };
            // The outer statement's body **is** the inner statement, so the code after the outer
            // `try` is the code after the inner one: both levels state that join.
            let join = join_after(&facts, *inner_end);
            Ok(Some(Catches {
                sites: outer_sites,
                join: join.clone(),
                lead,
                inner: Some(Box::new(Catches {
                    sites: inner_sites,
                    join,
                    lead,
                    inner: None,
                })),
            }))
        }
        _ => Ok(None),
    }
}

/// Whether two protected ranges that begin at one instruction are one `try`'s nesting.
///
/// The table states the pair, and this is what has to be read off it for the nesting to be the
/// shape the bytes hold:
///
/// * the narrower row's **handler entry lies inside the wider range**: `handler_bci >= inner end`
///   (a handler runs after the code it protects) and `handler_bci < outer end` (the wider range
///   protects the code the handler is entered from — which is also what makes the inner `try`,
///   handler and all, the wider statement's body rather than a second clause beside it);
/// * every clause's handler is reached by **that clause's own rows alone**. A handler a second range
///   also reaches is one body the table protects twice — `javac` splits one `try`'s protection
///   around code that cannot throw, and the wider range then covers instructions this statement's
///   body would have to hold as statements of its own.
fn nests(
    inner: &[&ExceptionHandlerFact],
    outer: &[&ExceptionHandlerFact],
    handlers: &[ExceptionHandlerFact],
) -> bool {
    let (Some(inner_end), Some(outer_end)) = (
        inner.first().map(|row| row.end_bci),
        outer.first().map(|row| row.end_bci),
    ) else {
        return false;
    };
    let handler_inside = inner
        .iter()
        .all(|row| inner_end <= row.handler_bci && row.handler_bci < outer_end);
    let own_rows_only = inner.iter().chain(outer.iter()).all(|row| {
        handlers
            .iter()
            .filter(|other| {
                other.catch_type_index.is_some() && other.handler_bci == row.handler_bci
            })
            .all(|other| (other.start_bci, other.end_bci) == (row.start_bci, row.end_bci))
    });
    handler_inside && own_rows_only
}

/// The clauses of one protected range: one site per handler entry, in exception-table order.
///
/// `rows` are the rows of **one** range, in the table's own order, which is the order the JVM
/// dispatches in and therefore the order the clauses are written in.
fn clause_sites(facts: &Facts<'_>, rows: &[&ExceptionHandlerFact]) -> Option<Vec<CatchSite>> {
    let mut sites: Vec<CatchSite> = Vec::new();
    for row in rows {
        let handler = facts.row_handler(row)?;
        let entry = facts.in_block(&handler).first()?;
        let Some(Operation::Store { slot }) = facts.op(entry.bci()) else {
            return None;
        };
        let type_index = row
            .catch_type_index
            .expect("a named row states its catch type");
        // The multi-catch: the run of rows that end at the same handler this one starts at, with a
        // class each of them names once. A row whose handler differs opens a clause of its own, and
        // so does a handler that came back after another one — the table's order is the priority the
        // clauses state, and merging across it would hand an exception to the wrong body.
        match sites.last_mut() {
            Some(site) if site.handler == handler && !site.type_indices.contains(&type_index) => {
                site.type_indices.push(type_index);
            }
            _ => sites.push(CatchSite {
                type_indices: vec![type_index],
                handler,
                parameter: *slot,
            }),
        }
    }
    Some(sites)
}

/// The block the code after one `try` begins at.
///
/// `javac` protects the instructions that may throw, so the transfer that carries the protected
/// range's **normal** completion to the code after the statement is the instruction the range stops
/// before: reading its own target — the one plain successor of the block that holds it — is that
/// block. A range with no transfer there is one whose normal completion leaves the method (the body
/// returned), and then the statement has no continuation to state.
fn join_after(facts: &Facts<'_>, end_bci: u32) -> Option<CanonicalBlockId> {
    if !matches!(facts.op(end_bci), Some(Operation::Transfer)) {
        return None;
    }
    let block = facts.block_of(end_bci)?;
    let successors = facts.view.successor_ids(block);
    let [only] = successors.as_slice() else {
        return None;
    };
    Some(only.clone())
}

/// The `synchronized` shape: one `monitorenter`, one `monitorexit` on each path out of the region.
fn monitor(
    facts: &mut Facts<'_>,
    profile: &crate::pass::RecoveryProfile,
    current: &CanonicalBlockId,
) -> Result<Option<Verdict>, StopReason> {
    let start = current.bci();
    let Some(enter) = facts
        .in_block(current)
        .iter()
        .map(|instruction| instruction.bci())
        .find(|bci| matches!(facts.op(*bci), Some(Operation::Monitor { enter: true })))
    else {
        return Ok(None);
    };
    facts.charge(enter)?;
    let refuse = |unproven: Unproven, at: u32| Verdict::refused(Some(&MONITOR), unproven, at);
    if !MONITOR.admits(profile) {
        return Ok(Some(refuse(Unproven::Profile, enter)));
    }
    // One enter in the whole body, and the header holds the lock's own value and nothing else.
    let enters: Vec<u32> = facts
        .order
        .iter()
        .copied()
        .filter(|bci| matches!(facts.op(*bci), Some(Operation::Monitor { enter: true })))
        .collect();
    if enters != vec![enter] {
        return Ok(Some(refuse(Unproven::Monitor, enter)));
    }
    let Some(store_bci) = facts.previous_bci(enter) else {
        return Ok(Some(refuse(Unproven::Monitor, enter)));
    };
    let Some(Operation::Store { slot: lock }) = facts.op(store_bci) else {
        return Ok(Some(refuse(Unproven::Monitor, store_bci)));
    };
    let lock = *lock;
    if !facts.one_expression((start, store_bci), enter) {
        return Ok(Some(refuse(Unproven::Monitor, start)));
    }
    // The enter locks the value the header's own store filled: either the very value that store
    // wrote, or — the idiom javac emits, `dup; astore slot; monitorenter` — the other output of the
    // duplication that produced both. Without that tie, "the exits leave the monitor the body
    // entered" would be a guess, and a `synchronized` block that leaves another object is a
    // different program.
    let entered_read = facts.step(enter).map(|step| {
        step.instruction
            .reads()
            .iter()
            .map(|(_, value)| *value)
            .collect::<Vec<ValueId>>()
    });
    let Some(entered_read) = entered_read else {
        return Ok(Some(refuse(Unproven::Monitor, enter)));
    };
    let [entered] = entered_read.as_slice() else {
        return Ok(Some(refuse(Unproven::Monitor, enter)));
    };
    let produced = facts.ssa.value(*entered).def().clone();
    let filled = facts.step(store_bci).is_some_and(|store| {
        store
            .instruction
            .writes()
            .iter()
            .any(|(_, written)| facts.same(*written, *entered))
            || store.instruction.reads().iter().any(|(_, read)| {
                facts.same(*read, *entered) || *facts.ssa.value(*read).def() == produced
            })
    });
    let duplicated = match &produced {
        Definition::Instruction { bci, .. } => matches!(facts.op(*bci), Some(Operation::Duplicate)),
        _ => false,
    };
    if !filled || !duplicated {
        return Ok(Some(refuse(Unproven::Monitor, enter)));
    }
    // The row: its range begins right after the enter.
    let Some(after) = facts.next_bci(enter) else {
        return Ok(Some(refuse(Unproven::Monitor, enter)));
    };
    let covering = facts.covering(after);
    let Some(row) = facts.innermost(after).cloned() else {
        return Ok(Some(refuse(
            if covering.is_empty() {
                Unproven::Handler
            } else {
                Unproven::RowsOverlap
            },
            after,
        )));
    };
    if row.start_bci != after {
        return Ok(Some(refuse(Unproven::RangeStart, after)));
    }
    // The exits of the whole body: exactly two, the normal path's and the handler's.
    let exits: Vec<u32> = facts
        .order
        .iter()
        .copied()
        .filter(|bci| matches!(facts.op(*bci), Some(Operation::Monitor { enter: false })))
        .collect();
    let [normal_exit, handler_exit] = exits.as_slice() else {
        return Ok(Some(refuse(Unproven::Monitor, enter)));
    };
    let Some(exit_load) = facts.previous_bci(*normal_exit) else {
        return Ok(Some(refuse(Unproven::Monitor, *normal_exit)));
    };
    if facts.op(exit_load) != Some(&Operation::Load { slot: lock }) {
        return Ok(Some(refuse(Unproven::Monitor, exit_load)));
    }
    // The instruction after the normal exit: `javac` writes either the `goto` that carries the run
    // on after the statement or — where the statement's body returns — the `return` itself. The
    // range a monitor's row declares ends between the exit and that instruction in both shapes: the
    // exit is protected (it may raise), the instruction after it is not. Any other instruction, and
    // a `return` whose own links the proof below cannot read, is a shape this rule does not present.
    let Some(after) = facts.next_bci(*normal_exit) else {
        return Ok(Some(refuse(Unproven::Monitor, *normal_exit)));
    };
    let returns: Option<u32> = match facts.op(after) {
        // Today's shape: the statement's run continues at the transfer's own target.
        Some(Operation::Transfer) => None,
        // The `return` shape: the method ends with the value the region's own instructions left on
        // the stack. That value is read off the stack — not out of a local, and not a second read of
        // anything — and what proves it is the **definition** of the value the return reads: it has
        // to be an instruction inside the guarded body, which is where the statement's own run is.
        Some(Operation::Return) => {
            let Some(instruction) = facts.step(after).map(|step| step.instruction) else {
                return Ok(Some(refuse(Unproven::Monitor, after)));
            };
            let operands = stack_operands(instruction);
            // A void `return` reads no stack value, and one that reads several is not a return this
            // subset writes: both are refused rather than presented.
            let [(_slot, value)] = operands.as_slice() else {
                return Ok(Some(refuse(Unproven::Monitor, after)));
            };
            let Definition::Instruction { bci: produced, .. } =
                facts.ssa.value(facts.resolve(*value)).def()
            else {
                return Ok(Some(refuse(Unproven::Monitor, after)));
            };
            // A value produced before the `monitorenter` (the header's own load of `this`, say) is
            // outside the region the statement guards, and writing the statement would move where it
            // is read from: it stays refused.
            if *produced < row.start_bci || *produced >= exit_load {
                return Ok(Some(refuse(Unproven::Monitor, after)));
            }
            Some(after)
        }
        _ => return Ok(Some(refuse(Unproven::Monitor, after))),
    };
    if row.end_bci != after {
        return Ok(Some(refuse(Unproven::RangeEnd, row.end_bci)));
    }
    // Where the statement's claim ends: the join the run continues at, or — where the normal path
    // returns — the instruction after the return, which is the end of the method. The handler's own
    // range lies between the transfer and that join in the `goto` shape, and *after* the return in
    // the `return` shape, which is why it is claimed explicitly below.
    let (end, join): (u32, Option<CanonicalBlockId>) = match returns {
        Some(return_bci) => (facts.span_end(return_bci), None),
        None => {
            let join_bci = facts.block_of(after).and_then(|block| {
                facts
                    .view
                    .successor_ids(block)
                    .first()
                    .map(|joins| joins.bci())
            });
            let Some(join_bci) = join_bci else {
                return Ok(Some(refuse(Unproven::Monitor, after)));
            };
            let Some(join) = facts.block_at(join_bci) else {
                return Ok(Some(refuse(Unproven::Monitor, after)));
            };
            (join_bci, Some(join))
        }
    };
    // The handler leaves the same monitor, and its own exit is protected by itself: a `monitorexit`
    // may raise, and an exit outside every range would leave the monitor held.
    let handler = match monitor_handler(facts, &row, lock) {
        Ok(handler) => handler,
        Err((unproven, at)) => return Ok(Some(refuse(unproven, at))),
    };
    if handler.exit_bci != *handler_exit {
        return Ok(Some(refuse(Unproven::Monitor, *handler_exit)));
    }
    let guarded = facts.covering(*handler_exit).iter().any(|row| {
        row.catch_type_index.is_none() && facts.row_handler(row).as_ref() == Some(&handler.entry)
    });
    if !guarded {
        return Ok(Some(refuse(Unproven::Monitor, *handler_exit)));
    }
    // The body: the instructions the region protects, between the enter and the normal exit's load.
    let body = (row.start_bci, exit_load);
    if body.0 >= body.1 || !facts.statement_free(body) {
        return Ok(Some(refuse(Unproven::Body, body.0)));
    }
    // Everything between the statement's own start and where it ends belongs to it: its own run,
    // from the enter through the instruction after the exit's transfer or return, and the handler's
    // range beside it.
    let mut pieces: Vec<(u32, u32)> = vec![(start, facts.span_end(after)), handler.span];
    if let Err(cause) = explained(facts, start, end, &pieces) {
        return Ok(Some(refuse(cause.0, cause.1)));
    }
    let mut owned: Vec<CanonicalBlockId> = facts.blocks_in((start, end));
    // The handler's block is claimed even where it lies *outside* the statement's own range — after
    // a `return` that ends the method — because the statement writes the handler's exit: a block the
    // statement claimed and did not write would drop the statements it holds, and one it does not
    // claim at all is quoted as an uncovered block.
    if !owned.contains(&handler.entry) {
        owned.push(handler.entry.clone());
    }
    owned.sort_by_key(|block| block.bci());
    pieces.clear();
    let mut facts_read: Vec<u32> = vec![
        enter,
        exit_load,
        *normal_exit,
        handler.entry.bci(),
        handler.exit_bci,
    ];
    if let Some(return_bci) = returns {
        // The `return` the normal path ends in is an instruction the proof read, and an anchor of
        // the statement's own text: the value it returns is the one the body's read produced.
        facts_read.push(return_bci);
        facts_read.sort_unstable();
    }
    Ok(Some(Verdict::Claimed(Plan {
        shape: Shape::Monitor {
            enter_bci: enter,
            normal_exit_bci: *normal_exit,
            returns,
        },
        lead: (start, start),
        body,
        owned,
        join,
        facts: facts_read,
    })))
}

/// The `try`-with-resources shape.
fn resources(
    facts: &mut Facts<'_>,
    profile: &crate::pass::RecoveryProfile,
    current: &CanonicalBlockId,
) -> Result<Verdict, StopReason> {
    let start = current.bci();
    let end = facts.end_of(current);
    let mut candidates: Vec<&ExceptionHandlerFact> = Vec::new();
    for bci in facts.bcis((start, end)) {
        for row in facts.covering(bci) {
            if !candidates.iter().any(|entry| entry.ordinal == row.ordinal) {
                candidates.push(row);
            }
        }
    }
    if candidates.is_empty() {
        return Ok(Verdict::NotGuarded);
    }
    // A catch-all copy is only a candidate. Claim it after the complete certificate and the
    // straight-body presentation gate both pass; otherwise preserve the original refusal.
    for row in &candidates {
        if let Some(at) = finally_copy(facts, row) {
            if FINALLY.admits(profile)
                && let Some(proof) = prove_finally_copy(facts, row)?
                && proof.row_ordinal == row.ordinal
                && start <= proof.protected.0
                && proof.protected.0 < end
                && (start == proof.protected.0
                    || facts.previous_bci(proof.protected.0).is_some_and(|before| {
                        completed_field_assignment(facts, before, start, proof.protected.0)
                            && single_statement(facts, (start, proof.protected.0), before)
                    }))
                && facts.statement_free((start, proof.protected.0))
                && facts.bcis((start, proof.protected.0)).iter().all(|bci| {
                    facts.block_of(*bci) == Some(current) && facts.covering(*bci).is_empty()
                })
                && facts.statement_free(proof.protected)
            {
                let return_end = facts.span_end(proof.saved_return.1);
                let handler_end = facts.span_end(proof.primary.2);
                let pieces = [
                    (start, proof.protected.0),
                    proof.protected,
                    proof.normal_cleanup,
                    (proof.normal_cleanup.1, return_end),
                    (proof.primary.0, proof.handler_cleanup.0),
                    proof.handler_cleanup,
                    (proof.handler_cleanup.1, handler_end),
                ];
                if explained(facts, start, handler_end, &pieces).is_ok()
                    && proof.owned.iter().all(|block| {
                        facts.in_block(block).iter().all(|instruction| {
                            let bci = instruction.bci();
                            start <= bci
                                && bci < handler_end
                                && pieces.iter().any(|piece| piece.0 <= bci && bci < piece.1)
                        })
                    })
                {
                    return Ok(Verdict::Claimed(Plan {
                        shape: Shape::Finally {
                            normal_cleanup: proof.normal_cleanup,
                            returns: proof.saved_return.1,
                        },
                        lead: (start, proof.protected.0),
                        body: proof.protected,
                        owned: proof.owned,
                        join: proof.join,
                        facts: proof.origins,
                    }));
                }
            }
            return Ok(Verdict::refused(None, Unproven::FinallyCopy, at));
        }
    }
    // The candidates, narrowest range first: the narrowest is the shape's own innermost row, and a
    // wider one is what an enclosing construct would be. The first that proves wins; when none does,
    // the narrowest candidate's own failure is the one reported, because it is the one about *this*
    // shape.
    let mut ordered = candidates.clone();
    ordered.sort_by_key(|row| (row.end_bci.saturating_sub(row.start_bci), row.start_bci));
    let mut failure: Option<Cause> = None;
    let mut header = false;
    for row in ordered {
        // A row whose protected range no instruction precedes protects no initialisation: a header
        // declares a resource the statement *before* the range filled, and with no instruction there
        // at all there is nothing the range's own start could be the end of. Such a row is a
        // `catch` — or a `finally` — and not a resource of this shape, so it is not examined as one.
        //
        // [`initialisation`] refuses both this and a resource whose store is not one statement under
        // the same `ResourceInit`; the difference is that here there is no store *at all*, which is
        // the one case that is not a resource header a rule could have read. A row with an
        // instruction before its range keeps the refusal it has today — a `try` whose initialisation
        // this build cannot state is never spelled as a user `catch`.
        //
        // The instruction before the range answers the same question from the other side: it is
        // where a header's own initialisation *ends*, and an ordinary assignment — a store of a
        // value no `new` and no invocation produced — means the `try` after `int x = 1;` is a
        // `try`/`catch` rather than a header `initialisation` could read
        // ([`initialises_resource`]). A store filled by `new Res()` or by a call keeps the row
        // examined: a `try`-with-resources this build cannot prove keeps degrading as one, and is
        // never spelled as a user `catch`.
        //
        // The third value a header's own store can hold is a **local**: `try (r)` is Java 9 syntax,
        // and javac keeps the variable's value in a local of its own before the protected range
        // (`aload r; astore copy`), so the store the range follows reads another slot
        // ([`copied_local`]). That row is this rule's own shape too — the header writes
        // `try (T copy = r)`, the declaration [`initialisation`] already reads — but the reading is
        // claimed only when the **whole** proof succeeds: an ordinary `r = other; try { … }
        // catch (E e) { … }` compiles to the same store, and a copy this rule cannot prove must stay
        // the `catch` its own table names rather than becoming a `try`-with-resources refusal.
        //
        // So the questions are one — "is what precedes this range a resource's own initialisation?" —
        // and a row answers *no* where no instruction precedes it in this block and where an
        // ordinary assignment precedes it. Neither row is examined as a resource, and when every
        // candidate answers no the shape is not this rule's at all.
        let Some(before) = facts
            .previous_bci(row.start_bci)
            .filter(|before| *before >= start)
        else {
            continue;
        };
        // A store that binds a `catch` clause's parameter writes the reference its handler was
        // **entered** with ([`handler_binding`]), and no resource's initialisation does that. A row
        // whose range begins after such a store is the clause's own body — the nested `try` of
        // `nested(I)I`, whose range `[8, 10)` follows the outer clause's `astore_1` at BCI 7 — so it
        // is not examined as a header this rule could state: it is left to [`catches`].
        //
        // The row is skipped rather than refused, because what precedes the range is read here and
        // is no initialisation at all: the conservative refusal below is for a store whose statement
        // could not be read, and this is one whose statement is the handler's own entry.
        if matches!(facts.op(before), Some(Operation::Store { .. }))
            && handler_binding(facts, before)
        {
            continue;
        }
        if row.catch_type_index.is_some()
            && completed_field_assignment(facts, before, start, row.start_bci)
        {
            continue;
        }
        // A direct null literal is not enough to call an ordinary `try` a resource header. Admit it
        // to the existing full proof only when both paths already have the close contour of a
        // nullable resource: the normal close group and the exceptional handler's guarded close.
        // The close handler's suppression and rethrow are deliberately left to `twr`, so a damaged
        // suppression remains a refusal with this rule's source instead of silently becoming a
        // user catch.
        let null_resource = initialisation(facts, row.start_bci, start)
            .ok()
            .filter(|(init, slot)| exact_null_initializer(facts, *init, *slot));
        if let Some((_, slot)) = null_resource {
            if !null_resource_close_outline(facts, row, slot) {
                continue;
            }
            header = true;
        }
        // The copy `javac` makes of the variable a header names, when the store before the range is
        // one: the slot its load read. `None` for every other store — including one this rule claims
        // under [`initialises_resource`], which keeps today's refusal when the rest of the shape
        // fails to prove.
        let mut copy: Option<u16> = None;
        if null_resource.is_none()
            && matches!(facts.op(before), Some(Operation::Store { .. }))
            && !initialises_resource(facts, before, start)
        {
            let Some(loaded) = copied_local(facts, before, start) else {
                continue;
            };
            // The body must not store the original slot again: the header declares the copy, and a
            // body that wrote the name the header reads would present a resource the bytecode's own
            // value flow does not have.
            if facts.bcis((row.start_bci, row.end_bci)).into_iter().any(
                |bci| matches!(facts.op(bci), Some(Operation::Store { slot }) if *slot == loaded),
            ) {
                continue;
            }
            copy = Some(loaded);
        }
        if copy.is_none() {
            header = true;
        }
        facts.charge(row.start_bci)?;
        match twr(facts, profile, current, row) {
            Ok(plan) => return Ok(Verdict::Claimed(plan)),
            // A copy the rest of the shape does not prove is no header of this rule: the row is read
            // as the `catch` its own table names, exactly as one after an ordinary assignment is.
            Err(TwrFailure::Stop(reason)) => return Err(reason),
            Err(TwrFailure::Proof(_)) if copy.is_some() => continue,
            Err(TwrFailure::Proof(cause)) => {
                if failure.is_none() {
                    failure = Some(cause);
                }
            }
        }
    }
    // Every candidate was a row no instruction precedes, or one an ordinary assignment precedes: no
    // initialisation of this block's own run is what a header would be read from, so nothing of the
    // shape's own is here. The walk states its own reason for the edge it cannot leave and no `try`
    // header is claimed — which is what lets the rows that name `catch` types be read as clauses.
    if !header {
        return Ok(Verdict::NotGuarded);
    }
    let (unproven, at) = failure.unwrap_or((Unproven::Handler, start));
    Ok(Verdict::refused(Some(&TWR), unproven, at))
}

/// One proved `try`-with-resources, from the shape's innermost row outwards.
fn twr(
    facts: &mut Facts<'_>,
    profile: &crate::pass::RecoveryProfile,
    current: &CanonicalBlockId,
    innermost: &ExceptionHandlerFact,
) -> Result<Plan, TwrFailure> {
    let start = current.bci();
    if !TWR.admits(profile) {
        return Err((Unproven::Profile, start).into());
    }
    // The levels, outward from the innermost row: each one is a row whose declared range strictly
    // contains the level inside it and whose handler closes a resource.
    let mut chain: Vec<&ExceptionHandlerFact> = vec![innermost];
    loop {
        let inner = *chain.last().expect("the chain holds the innermost row");
        let outer = facts
            .handlers
            .iter()
            .filter(|row| {
                !chain.iter().any(|entry| entry.ordinal == row.ordinal)
                    && row.start_bci <= inner.start_bci
                    && row.end_bci >= inner.end_bci
                    && (row.start_bci < inner.start_bci || row.end_bci > inner.end_bci)
                    // Only a row whose handler closes something can be a level of this header. A
                    // `catch` a compiler wraps the whole statement in contains the level's range
                    // too, and it is exactly the row this rule must *not* absorb: it is the one the
                    // unexplained-row check refuses the shape over.
                    && facts.closes_something(row)
            })
            .min_by_key(|row| (row.end_bci.saturating_sub(row.start_bci), row.start_bci));
        match outer {
            Some(row) => chain.push(row),
            None => break,
        }
    }
    chain.reverse();
    let mut resources: Vec<Resource> = Vec::with_capacity(chain.len());
    let mut handlers: Vec<CloseHandler> = Vec::with_capacity(chain.len());
    let mut floor = start;
    for (index, row) in chain.iter().enumerate() {
        if index > 0 {
            floor = chain[index - 1].start_bci;
        }
        let (init, slot) = initialisation(facts, row.start_bci, floor)?;
        if resources.iter().any(|resource| resource.slot == slot) {
            return Err((Unproven::ResourceSlot, init.0).into());
        }
        let handler = close_handler(facts, row, slot)?;
        resources.push(Resource {
            slot,
            init,
            close_bci: 0,
            exceptional_close_bci: handler.close_bci,
        });
        handlers.push(handler);
    }
    let innermost_level = *chain.last().expect("the chain holds the innermost row");
    let innermost_handler = handlers.last().expect("one handler per level");
    let body = (innermost_level.start_bci, innermost_level.end_bci);
    if body.0 >= body.1 || !facts.statement_free(body) {
        return Err((Unproven::Body, body.0).into());
    }
    // The normal path closes the resources in the reverse order of the header: the chain is matched
    // against the declarations from the last one back, and a chain that closes them in any other
    // order (or misses one) is refused. This is what makes the order the header writes the order the
    // bytecode ran.
    let mut at = body.1;
    let mut closes = vec![0u32; resources.len()];
    let mut close_starts = vec![0u32; resources.len()];
    let mut pieces: Vec<(u32, u32)> = Vec::new();
    for (position, resource) in resources.iter().enumerate().rev() {
        close_starts[position] = at;
        let Some((close_at, next)) = normal_close(facts, at, resource.slot) else {
            return Err((Unproven::CloseOrder, at).into());
        };
        closes[position] = close_at;
        pieces.push((at, next));
        at = next;
    }
    // A level's main row ends at the first instruction of that resource's normal close group. Prove
    // the close chain first: the endpoint is meaningful only after that group has been tied to this
    // resource.
    for (index, row) in chain.iter().enumerate() {
        if row.end_bci != close_starts[index] {
            return Err((Unproven::RangeEnd, row.end_bci).into());
        }
    }
    for (index, resource) in resources.iter_mut().enumerate() {
        resource.close_bci = closes[index];
    }
    let return_tail = twr_return_tail(facts, body, at)?;
    let claimed_end = return_tail
        .as_ref()
        .map(|(_, return_bci, _)| facts.span_end(*return_bci))
        .unwrap_or(at);
    let mut rows: Vec<u32> = chain.iter().map(|row| row.ordinal).collect();
    for handler in &handlers {
        rows.push(handler.guard.ordinal);
    }
    // Each enclosing resource must protect the complete exceptional cleanup of the level inside
    // it. Some javac layouts do that with the parent's main row. Others end the main row at its
    // own normal close, and add one exact companion row for the child's handler. Read that row
    // only when its protected range, catch type, and target all match the parent row; unexplained
    // overlaps and extra rows remain for the rejection below.
    let mut companions: Vec<ExceptionHandlerFact> = Vec::new();
    for index in 0..chain.len().saturating_sub(1) {
        let parent = chain[index];
        let child_handler = handlers[index + 1].span;
        if parent.start_bci <= child_handler.0 && parent.end_bci >= child_handler.1 {
            continue;
        }
        let parent_catch = parent.catch_type_index;
        let parent_handler = parent.handler_bci;
        let mut exact = Vec::new();
        for candidate_index in 0..facts.handlers.len() {
            let candidate = facts.handlers[candidate_index].clone();
            facts.charge(candidate.start_bci)?;
            if candidate.start_bci == child_handler.0
                && candidate.end_bci == child_handler.1
                && candidate.catch_type_index == parent_catch
                && candidate.handler_bci == parent_handler
                && !rows.contains(&candidate.ordinal)
            {
                exact.push(candidate);
            }
        }
        let [companion] = exact.as_slice() else {
            return Err((Unproven::RangeEnd, child_handler.0).into());
        };
        companions.push(companion.clone());
        rows.push(companion.ordinal);
    }
    let enclosure = enclosing_clauses(facts, current, &rows, handlers[0].span.1);
    for row in facts.handlers {
        if rows.contains(&row.ordinal) {
            continue;
        }
        let covers = facts
            .bcis((start, claimed_end))
            .into_iter()
            .any(|bci| row.start_bci <= bci && bci < row.end_bci);
        if !covers {
            continue;
        }
        let enclosed = enclosure.as_ref().is_some_and(|clauses| {
            clauses
                .iter()
                .any(|clause| clause.handler_bci == row.handler_bci)
        });
        if !enclosed {
            return Err((Unproven::Unexplained, row.start_bci).into());
        }
    }
    // The join is where the run continues after the statement. `javac` writes a `goto` there
    // whenever the statement is followed by code of its own method — the target is then a block —
    // and a statement whose normal path ends the method (`try (…) { return …; }`) has no code to
    // continue at all: the row itself ends where the close chain does.
    let join = return_tail.is_none().then(|| facts.block_at(at)).flatten();
    if return_tail.is_none() && join.is_none() && at < facts.end_of(current) {
        // The close chain runs into the rest of the statement's own block, and no block begins
        // where it continues: the instructions after the statement are those of a block this shape
        // has already claimed, and the walk can present neither them nor a place to continue at.
        // A terminal `try (…) { return …; }` is admitted only after `twr_return_tail` proves the
        // saved body value and its effect-free load/return pair. Reaching this branch means that
        // proof did not apply; treating the tail as an ordinary continuation would drop it.
        return Err((Unproven::Continuation, at).into());
    }
    // Every row that protects part of the statement's span has to be one of its own rows — or one of
    // the clauses of the `try` this statement sits inside, which the walk writes around it. The two
    // cases are told apart by the table's own geometry, and the geometry is javac's: a `try (…) { … }
    // catch (…) { … }` whose body falls through to the code after it emits **two** user rows, because
    // the clause has to cover the cleanup's rethrow as well as the statement's own code, and neither
    // of them spans the statement from its first instruction to the end of its handler. A **single**
    // row that does span it is the `catch` a compiler winds around the whole construct — the shape
    // `tests/p3_guard.rs`'s `withCatch` states — and it keeps today's refusal.
    // Every instruction between the statement's own start and its join belongs to the shape.
    for resource in &resources {
        pieces.push(resource.init);
    }
    pieces.push(body);
    for handler in &handlers {
        pieces.push(handler.span);
        pieces.push((handler.guard.start_bci, handler.guard.end_bci));
    }
    if let Some((load_bci, _, _)) = return_tail.as_ref() {
        pieces.push((*load_bci, claimed_end));
    }
    explained(facts, start, claimed_end, &pieces)?;
    // Every block the statement owns holds an instruction of its own span, and `explained` has just
    // checked that no instruction of that span belongs to anything else: a handler the layout put
    // *outside* the span is deliberately not claimed here — the walk quotes it, and the run says so
    // in its fallbacks, rather than the statement claiming a block it does not write.
    let mut owned: Vec<CanonicalBlockId> = facts.blocks_in((start, claimed_end));
    owned.sort_by_key(|block| block.bci());
    let lead = (start, resources.first().map(|r| r.init.0).unwrap_or(start));
    let mut facts_read: Vec<u32> = Vec::new();
    if let Some((load_bci, return_bci, store_bci)) = return_tail.as_ref() {
        facts_read.extend([*load_bci, *return_bci, *store_bci]);
    }
    for resource in &resources {
        facts_read.push(resource.close_bci);
        facts_read.push(resource.init.1.saturating_sub(1));
    }
    for handler in &handlers {
        facts_read.push(handler.primary_bci);
        facts_read.push(handler.close_bci);
        facts_read.push(handler.suppression_bci);
        facts_read.push(handler.suppression_call_bci);
        facts_read.push(handler.rethrow_bci);
    }
    for companion in &companions {
        facts_read.push(companion.start_bci);
        if let Some(last) = facts.previous_bci(companion.end_bci) {
            facts_read.push(last);
        }
    }
    facts_read.sort_unstable();
    facts_read.dedup();
    let _ = innermost_handler;
    Ok(Plan {
        shape: Shape::Resources {
            resources,
            returns: return_tail.as_ref().map(|(_, return_bci, _)| *return_bci),
        },
        lead,
        body,
        owned,
        join,
        facts: facts_read,
    })
}

/// The clause rows of the `try` this statement sits inside, when the table states one.
///
/// `javac` splits one enclosing clause's protection along the pieces of the statement it wraps: the
/// statement's own code up to the normal close is one range, and the compiler's cleanup — the
/// handlers that close the resource and suppress into the primary — is another, because the code
/// that runs between them cannot raise (it is the `return` the statement's body filled a local for,
/// or the `goto` that carries the run past the statement). The clause's rows therefore begin where
/// the statement's own row does **not** have to: in the statement's own block, where [`catches`]
/// reads them, reaching one handler that is not one of the shape's own.
///
/// What is read here is exactly what [`catches`] will write around the statement: the rows that name
/// a `catch` type, begin inside the statement's own block, reach a handler the shape did not prove —
/// and read as **one** statement's clauses. Every range begins at one instruction and there are at
/// most two ends (the nesting of P3 2.5); a set that does not read that way is no enclosure this
/// rule may lean on, because the walk would not write its clauses and this rule would have claimed a
/// statement with the handler they name dropped from the artifact.
///
/// A row covering the shape from its first instruction to the end of its handler is **not** part of
/// such a set: one range that spans the whole construct is the `catch` a compiler winds around it
/// (the shape `tests/p3_guard.rs` pins as `jre_guard_unexplained_row`), not a clause the source's
/// `try (…)` header sits inside.
fn enclosing_clauses<'a>(
    facts: &Facts<'a>,
    current: &CanonicalBlockId,
    own: &[u32],
    handler_end: u32,
) -> Option<Vec<&'a ExceptionHandlerFact>> {
    let shape_start = current.bci();
    let own_handlers: Vec<u32> = facts
        .handlers
        .iter()
        .filter(|row| own.contains(&row.ordinal))
        .map(|row| row.handler_bci)
        .collect();
    let last = facts
        .in_block(current)
        .last()
        .map(|instruction| instruction.bci());
    let clauses: Vec<&ExceptionHandlerFact> = facts
        .handlers
        .iter()
        .filter(|row| {
            row.catch_type_index.is_some()
                && row.start_bci >= shape_start
                && last.is_some_and(|last| row.start_bci <= last)
                && !own_handlers.contains(&row.handler_bci)
                && !(row.start_bci <= shape_start && row.end_bci >= handler_end)
        })
        .collect();
    let start = clauses.first()?.start_bci;
    if clauses.iter().any(|row| row.start_bci != start) {
        return None;
    }
    let mut ends: Vec<u32> = clauses.iter().map(|row| row.end_bci).collect();
    ends.sort_unstable();
    ends.dedup();
    if ends.len() > 2 {
        return None;
    }
    Some(clauses)
}

/// Whether every instruction between a statement's own start and its join belongs to one of the
/// shape's pieces.
///
/// This is what keeps a claim from dropping a statement: the walk claims whole blocks, and a block
/// it claims is never written by anyone else — so an instruction of such a block that the shape does
/// not account for would leave the artifact.
fn explained(facts: &Facts<'_>, start: u32, join: u32, pieces: &[(u32, u32)]) -> Result<(), Cause> {
    for bci in facts.bcis((start, join)) {
        if !pieces.iter().any(|piece| piece.0 <= bci && bci < piece.1) {
            return Err((Unproven::Span, bci));
        }
    }
    Ok(())
}

/// `aload r; ifnull L; aload r; invokevirtual close()V` and then `goto L` on the **normal** path.
///
/// The group's `L` is where the run continues — the next resource's own close, or the join. A close
/// the compiler leaves as the last instruction of its own block states the same thing by falling
/// through: `L` is then the very next instruction, and the close's block has `L` as its only
/// successor just the same. Either way nothing else runs between the close and the continuation.
fn normal_close(facts: &Facts<'_>, at: u32, slot: u16) -> Option<(u32, u32)> {
    let first = facts.step(at)?;
    if facts.op(first.instruction.bci()) != Some(&Operation::Load { slot }) {
        return None;
    }
    let second = facts.step(facts.next_bci(at)?)?;
    // The unchecked shape: the resource's own initialisation proves it non-null (`new Res(…)`), so
    // `javac` writes the close alone on the normal path too — no test to skip it. The run then
    // continues where the close's own run leaves: at the block the `goto` that follows reaches when
    // the compiler wrote one (`try { … } catch (…) { … }` whose body does not return), and
    // otherwise at the instruction after the close, inside the statement's own block.
    if let Some(Operation::Invoke(called)) = facts.op(second.instruction.bci()) {
        if called.name() != "close" || called.descriptor() != "()V" {
            return None;
        }
        if !receiver_is(facts, second.instruction, first.instruction) {
            return None;
        }
        let after = facts.next_bci(second.instruction.bci())?;
        if matches!(facts.op(after), Some(Operation::Transfer)) {
            let leaving = facts.block_of(second.instruction.bci())?;
            let jump = facts.view.successor_ids(leaving);
            let [continuation] = jump.as_slice() else {
                return None;
            };
            return Some((second.instruction.bci(), continuation.bci()));
        }
        return Some((second.instruction.bci(), after));
    }
    let Some(Operation::Comparison {
        op: CompareOp::JumpIfNull,
        target,
    }) = facts.op(second.instruction.bci())
    else {
        return None;
    };
    let target = *target;
    // The `ifnull` is the block's own branch: its two arms are the close and the continuation the
    // null case jumps to.
    let successors = facts.view.successor_ids(first.block);
    if successors.len() != 2 || !successors.iter().any(|block| block.bci() == target) {
        return None;
    }
    let third = facts.step(facts.next_bci(second.instruction.bci())?)?;
    if facts.op(third.instruction.bci()) != Some(&Operation::Load { slot }) {
        return None;
    }
    let close_block = facts.block_of(third.instruction.bci())?;
    if !successors.iter().any(|block| block == close_block) {
        return None;
    }
    let fourth = facts.step(facts.next_bci(third.instruction.bci())?)?;
    let Some(Operation::Invoke(called)) = facts.op(fourth.instruction.bci()) else {
        return None;
    };
    if called.name() != "close" || called.descriptor() != "()V" {
        return None;
    }
    if !receiver_is(facts, fourth.instruction, third.instruction) {
        return None;
    }
    // The run that leaves the close: the `goto` that ends its block when the compiler wrote one, and
    // otherwise the close's own fall-through, which is the continuation itself as the next
    // instruction. Nothing else may run in between.
    let after = facts.next_bci(fourth.instruction.bci())?;
    let leaving = match after {
        next if next == target => facts.block_of(fourth.instruction.bci())?,
        next => {
            let fifth = facts.step(next)?;
            if !matches!(facts.op(fifth.instruction.bci()), Some(Operation::Transfer)) {
                return None;
            }
            fifth.block
        }
    };
    // The close's own block runs straight to the continuation: what the graph states as its only
    // successor is where the run continues, and it is `L`.
    let jump = facts.view.successor_ids(leaving);
    if jump.len() != 1 || jump[0].bci() != target {
        return None;
    }
    Some((fourth.instruction.bci(), target))
}
