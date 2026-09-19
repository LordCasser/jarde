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
//! # No `finally` copy is merged
//!
//! A `finally` clause is not a region with a handler: javac **copies** its code onto every exit path
//! of the `try` (the fixture's `fin()` shows the two copies of `tail()`, one on the normal path and
//! one in the handler that rethrows), and presenting that as one `try { … } finally { … }` restates
//! the source only if the copies are provably the same code. Proving *that* means comparing operands
//! this layer does not model and showing that every exit of the `try` runs exactly one copy — the
//! number of times the copy runs is what is at stake. This build does not prove it, so the shape is
//! **refused** and stated ([`Unproven::FinallyCopy`], reported under `jre_guard_finally_copy` with
//! the copy's own BCI): quoting it is the honest answer, and it is the one the spec's fallback
//! clause asks for.
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

use crate::decode::Operations;
use crate::facts::{CompareOp, Operation};
use crate::normal_flow::NormalFlowView;
use crate::pass::{MONITOR, Pass, TWR};
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
}

/// Which guarded statement a region is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Shape {
    /// `try (T n = …; …) { body }`, with the resources in **declaration** order.
    Resources(Vec<Resource>),
    /// `synchronized (lock) { body }`, with the BCI of the `monitorenter` the header reads its lock
    /// from.
    Monitor { enter_bci: u32 },
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
            Shape::Resources(_) => &TWR,
            Shape::Monitor { .. } => &MONITOR,
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
    /// examined it, or `None` for a shape no rule of this build owns (the `finally` copy).
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
                "the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; merging the copies into one `finally` restates the source only if they are provably equal, which this build does not prove"
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
    let head = facts.sequence(&entry);
    let [
        head_store,
        head_exception,
        (
            _,
            Some(Operation::Comparison {
                op: CompareOp::JumpIfNull,
                target,
            }),
        ),
    ] = head.as_slice()
    else {
        return Err((Unproven::Handler, entry.bci()));
    };
    let (Some(Operation::Store { slot: primary }), Some(Operation::Load { slot: loaded })) =
        (head_store.1, head_exception.1)
    else {
        return Err((Unproven::Handler, entry.bci()));
    };
    let (primary, target) = (*primary, *target);
    if *loaded != slot {
        return Err((Unproven::CloseTarget, head_exception.0));
    }
    let successors = facts.view.successor_ids(&entry);
    let exit = facts
        .block_at(target)
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
    // The close's own row: the innermost one that covers it, and its handler suppresses into the
    // primary. A close the table does not protect at all has no place to record its own failure, and
    // the region is refused.
    let Some(guard) = facts.innermost(*call_bci).cloned() else {
        return Err((Unproven::CloseGuard, *call_bci));
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
        primary_bci: head_store.0,
        close_bci: *call_bci,
        suppression_bci: suppression[1].0,
        suppression_call_bci: suppression[3].0,
        rethrow_bci: facts
            .in_block(&exit)
            .last()
            .map(|last| last.bci())
            .unwrap_or(exit.bci()),
    })
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
    if let Some(verdict) = monitor(&mut facts, profile, current)? {
        return Ok(verdict);
    }
    resources(&mut facts, profile, current)
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
    // The range a monitor's row declares ends between the exit and the `goto` that follows it: the
    // exit is protected (it may raise), the transfer is not.
    let Some(goto) = facts.next_bci(*normal_exit) else {
        return Ok(Some(refuse(Unproven::Monitor, *normal_exit)));
    };
    if !matches!(facts.op(goto), Some(Operation::Transfer)) {
        return Ok(Some(refuse(Unproven::Monitor, goto)));
    }
    if row.end_bci != goto {
        return Ok(Some(refuse(Unproven::RangeEnd, row.end_bci)));
    }
    let join_bci = facts.block_of(goto).and_then(|block| {
        facts
            .view
            .successor_ids(block)
            .first()
            .map(|joins| joins.bci())
    });
    let Some(join_bci) = join_bci else {
        return Ok(Some(refuse(Unproven::Monitor, goto)));
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
    // Everything between the statement's own start and the join belongs to it.
    let mut pieces: Vec<(u32, u32)> = vec![(start, facts.span_end(goto)), handler.span];
    let Some(join) = facts.block_at(join_bci) else {
        return Ok(Some(refuse(Unproven::Monitor, goto)));
    };
    if let Err(cause) = explained(facts, start, join_bci, &pieces) {
        return Ok(Some(refuse(cause.0, cause.1)));
    }
    let mut owned: Vec<CanonicalBlockId> = facts.blocks_in((start, join_bci));
    owned.sort_by_key(|block| block.bci());
    pieces.clear();
    let facts_read: Vec<u32> = vec![
        enter,
        exit_load,
        *normal_exit,
        handler.entry.bci(),
        handler.exit_bci,
    ];
    Ok(Some(Verdict::Claimed(Plan {
        shape: Shape::Monitor { enter_bci: enter },
        lead: (start, start),
        body,
        owned,
        join: Some(join),
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
    // The `finally` copy is recognised before anything else, because it is not a guarded region at
    // all: it is a handler that runs code and rethrows, and the only statement that would present it
    // is one this build refuses to write without proving the copies equal.
    for row in &candidates {
        if let Some(at) = finally_copy(facts, row) {
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
    for row in ordered {
        facts.charge(row.start_bci)?;
        match twr(facts, profile, current, row) {
            Ok(plan) => return Ok(Verdict::Claimed(plan)),
            Err(cause) => {
                if failure.is_none() {
                    failure = Some(cause);
                }
            }
        }
    }
    let (unproven, at) = failure.unwrap_or((Unproven::Handler, start));
    Ok(Verdict::refused(Some(&TWR), unproven, at))
}

/// One proved `try`-with-resources, from the shape's innermost row outwards.
fn twr(
    facts: &Facts<'_>,
    profile: &crate::pass::RecoveryProfile,
    current: &CanonicalBlockId,
    innermost: &ExceptionHandlerFact,
) -> Result<Plan, Cause> {
    let start = current.bci();
    if !TWR.admits(profile) {
        return Err((Unproven::Profile, start));
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
            return Err((Unproven::ResourceSlot, init.0));
        }
        let handler = close_handler(facts, row, slot)?;
        resources.push(Resource {
            slot,
            init,
            close_bci: 0,
        });
        handlers.push(handler);
    }
    let innermost_level = *chain.last().expect("the chain holds the innermost row");
    let innermost_handler = handlers.last().expect("one handler per level");
    let body = (innermost_level.start_bci, innermost_level.end_bci);
    if body.0 >= body.1 || !facts.statement_free(body) {
        return Err((Unproven::Body, body.0));
    }
    // The ranges: every level's row must end where this shape ends it — the level inside it ends its
    // handler there, and the innermost one ends where the normal close chain begins.
    for (index, row) in chain.iter().enumerate() {
        let expected = match handlers.get(index + 1) {
            Some(inner) => inner.span.1,
            None => body.1,
        };
        if row.end_bci != expected {
            return Err((Unproven::RangeEnd, row.end_bci));
        }
    }
    // The normal path closes the resources in the reverse order of the header: the chain is matched
    // against the declarations from the last one back, and a chain that closes them in any other
    // order (or misses one) is refused. This is what makes the order the header writes the order the
    // bytecode ran.
    let mut at = body.1;
    let mut closes = vec![0u32; resources.len()];
    let mut pieces: Vec<(u32, u32)> = Vec::new();
    for (position, resource) in resources.iter().enumerate().rev() {
        let Some((close_at, next)) = normal_close(facts, at, resource.slot) else {
            return Err((Unproven::CloseOrder, at));
        };
        closes[position] = close_at;
        pieces.push((at, next));
        at = next;
    }
    for (index, resource) in resources.iter_mut().enumerate() {
        resource.close_bci = closes[index];
    }
    let Some(join) = facts.block_at(at) else {
        return Err((Unproven::CloseOrder, at));
    };
    // Every row that protects part of the statement's span has to be one of its own rows: the
    // `catch` a compiler wraps a `try`-with-resources in is a handler this rule does not present.
    let mut rows: Vec<u32> = chain.iter().map(|row| row.ordinal).collect();
    for handler in &handlers {
        rows.push(handler.guard.ordinal);
    }
    for row in facts.handlers {
        if rows.contains(&row.ordinal) {
            continue;
        }
        let covers = facts
            .bcis((start, at))
            .into_iter()
            .any(|bci| row.start_bci <= bci && bci < row.end_bci);
        if covers {
            return Err((Unproven::Unexplained, row.start_bci));
        }
    }
    // Every instruction between the statement's own start and its join belongs to the shape.
    for resource in &resources {
        pieces.push(resource.init);
    }
    pieces.push(body);
    for handler in &handlers {
        pieces.push(handler.span);
        pieces.push((handler.guard.start_bci, handler.guard.end_bci));
    }
    explained(facts, start, at, &pieces)?;
    // Every block the statement owns holds an instruction of its own span, and `explained` has just
    // checked that no instruction of that span belongs to anything else: a handler the layout put
    // *outside* the span is deliberately not claimed here — the walk quotes it, and the run says so
    // in its fallbacks, rather than the statement claiming a block it does not write.
    let mut owned: Vec<CanonicalBlockId> = facts.blocks_in((start, at));
    owned.sort_by_key(|block| block.bci());
    let lead = (start, resources.first().map(|r| r.init.0).unwrap_or(start));
    let mut facts_read: Vec<u32> = Vec::new();
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
    facts_read.sort_unstable();
    facts_read.dedup();
    let _ = innermost_handler;
    Ok(Plan {
        shape: Shape::Resources(resources),
        lead,
        body,
        owned,
        join: Some(join),
        facts: facts_read,
    })
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

/// `aload r; ifnull L; aload r; invokevirtual close()V; goto L` on the **normal** path.
///
/// The group's `L` is where the run continues — the next resource's own close, or the join.
fn normal_close(facts: &Facts<'_>, at: u32, slot: u16) -> Option<(u32, u32)> {
    let first = facts.step(at)?;
    if facts.op(first.instruction.bci()) != Some(&Operation::Load { slot }) {
        return None;
    }
    let second = facts.step(facts.next_bci(at)?)?;
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
    let fifth = facts.step(facts.next_bci(fourth.instruction.bci())?)?;
    if !matches!(facts.op(fifth.instruction.bci()), Some(Operation::Transfer)) {
        return None;
    }
    // The close's own block runs straight to the continuation: the `goto` that ends it is what the
    // graph states as its only successor.
    let jump = facts.view.successor_ids(fifth.block);
    if jump.len() != 1 || jump[0].bci() != target {
        return None;
    }
    Some((fourth.instruction.bci(), target))
}
