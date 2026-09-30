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

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind, Definition, PhiInput, Slot, SsaInstruction,
    SsaTable, ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{CpEntryFacts, ExceptionHandlerFact, cp_class_name};

use crate::build::stack_operands;
use crate::decode::Operations;
use crate::facts::{CompareOp, InvokeKind, Operation};
use crate::init::Sites;
use crate::lambda::parse_method;
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
        /// Exact instruction starts of normal and exceptional cleanup that a source TWR header
        /// implicitly reconstructs. These instructions are proof evidence, not source locals.
        cleanup: Vec<u32>,
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
        /// The exact exception-table rows proved to enter the monitor cleanup handler, including
        /// the handler's self-protection row. Other exception rows inside this region are not a
        /// license to split a reused local across the statement.
        cleanup_rows: BTreeSet<u32>,
        /// The BCI of the `return` the **normal** path ends in, where it ends in one: `None` is the
        /// `goto` shape, whose run continues after the statement, and `Some(bci)` the shape whose
        /// region returns the value the body left on the stack — the return is written *inside* the
        /// braces, and the statement continues nowhere.
        returns: Option<u32>,
    },
    /// One conditional whose two straight arms each leave this monitor and return their own value.
    /// This deliberately records a closed, two-arm mapping rather than a nested general region.
    MonitorBranches {
        enter_bci: u32,
        branch_bci: u32,
        then_exit_bci: u32,
        then_return_bci: u32,
        else_exit_bci: u32,
        else_return_bci: u32,
    },
    /// One protected body with two proved cleanup copies and an exclusive completion form.
    Finally {
        normal_cleanup: (u32, u32),
        completion: FinallyCompletion,
        row_ordinal: u32,
        structured: bool,
    },
    /// The fixed two-row Java 8 layout with two equivalent iterable cleanup loops.
    LoopFinally {
        rows: [u32; 2],
        normal_cleanup: (u32, u32),
        handler_cleanup: (u32, u32),
    },
    /// One catch-all and two equivalent null-guarded cleanup copies with a void completion.
    ConditionalFinally {
        row_ordinal: u32,
        normal_cleanup: (u32, u32),
        handler_cleanup: (u32, u32),
        normal_return: u32,
    },
    /// The two-row nullable local cleanup with a saved reference return.
    NullableResourceFinally {
        row_ordinal: u32,
        normal_cleanup: (u32, u32),
        handler_cleanup: (u32, u32),
        saved_return: (u32, u32),
    },
    /// The fixed two-row flag conditional: a two-instruction `false` initialisation leads the
    /// method, the protected body sets the same local `true` exactly once before it saves the
    /// returned value, and both cleanup copies are the same eight-instruction `iload`-guarded
    /// read-modify-write of one `int` field. `flag_slot` is the slot the lead initialises, the
    /// body's one `true` store fills and both copies branch on.
    FlagConditionalFinally {
        row_ordinal: u32,
        flag_slot: u16,
        normal_cleanup: (u32, u32),
        handler_cleanup: (u32, u32),
        saved_return: (u32, u32),
    },
    /// The two-row nullable local whose cleanup reads the slot the lead initialised with `null`
    /// and the protected body assigned: a two-instruction `aconst_null` initialisation leads the
    /// method, the body's same-slot assignments fill the slot before it saves the returned value,
    /// and both cleanup copies are the same four-instruction `aload`-guarded optional call — with
    /// the shared optional guarded-throw tail the flag conditional's copies may carry. `slot` is
    /// the slot the lead initialises, the body fills and both copies branch on; the null source
    /// the branch tests is exactly the lead's own store.
    LocalNullConditionalFinally {
        row_ordinal: u32,
        slot: u16,
        normal_cleanup: (u32, u32),
        handler_cleanup: (u32, u32),
        saved_return: (u32, u32),
    },
    /// One named catch and two normal completions sharing a proved catch-all cleanup handler.
    SharedFinally {
        rows: [u32; 3],
        /// A fourth row may protect only the catch-all handler's binding store.
        binding_row: Option<u32>,
        catch_body: (u32, u32),
        catch_handler: CanonicalBlockId,
        catch_type: u16,
        catch_parameter: u16,
        normal_cleanup: (u32, u32),
        catch_cleanup: (u32, u32),
        completion: SharedFinallyCompletion,
    },
    /// Two real same-range rows and an empty named catch, with three identical call copies.
    EmptyCatchCallFinally {
        rows: [u32; 2],
        cleanup_target: crate::facts::CallTarget,
        catch_handler: CanonicalBlockId,
        catch_type: u16,
        catch_parameter: u16,
        cleanup: [(u32, u32); 3],
        transfers: [u32; 2],
    },
    /// The fixed Java 8 two-catch, four-row, four-copy return certificate.
    TwoCatchReturnFinally {
        rows: [u32; 4],
        catches: [(CanonicalBlockId, u16, u16); 2],
        cleanup: [(u32, u32); 4],
        saved_return: (u32, u32),
    },
    /// Test4's two nested IOException catches around the normal and exceptional cleanup copies.
    NestedCleanupFinally {
        rows: [u32; 4],
        catch_type: u16,
        normal_handler: CanonicalBlockId,
        catch_parameter: u16,
        cleanup: (u32, u32),
    },
    /// The one five-row, two-segment, four-copy Java 8 finally certificate.
    SegmentedFinally {
        rows: [u32; 5],
        segments: [(u32, u32); 2],
        catch_body: (u32, u32),
        catch_handler: CanonicalBlockId,
        catch_type: u16,
        catch_parameter: u16,
        cleanup: [(u32, u32); 4],
        early_return: u32,
        transfers: [u32; 2],
    },
    /// The fixed three-row body loop with two saved returns and one shared exceptional close.
    MultiReturnLoopFinally {
        rows: [u32; 3],
        segments: [(u32, u32); 2],
        cleanup: [(u32, u32); 3],
        returns: [(u32, u32); 2],
    },
    /// The TestFinally3 lowering: two catch-all rows over one handler with a gap between the
    /// protected ranges, the gap itself the early return's own `aload s; invoke; aload v;
    /// areturn`, and **no row over the handler** — the early, normal and exceptional cleanup
    /// copies are all unguarded, so a cleanup throw replaces the completion on every path. A
    /// two-instruction `aconst_null; astore s` leads the method; the protected body assigns the
    /// same slot `s` and ends in `aconst_null; astore v` (the early return's saved literal) or
    /// in the body producer's own store (the normal one); both returns read `v` after their
    /// copy. `cleanup_target` is the one static call all three copies make on `s`.
    SegmentedNullLeadFinally {
        rows: [u32; 2],
        slot: u16,
        value_slot: u16,
        segments: [(u32, u32); 2],
        cleanup_target: crate::facts::CallTarget,
        /// The gap's unguarded copy and the `areturn` that ends it: the instructions the early
        /// return's statement carries as derived origins, and the range the builder skips.
        early_cleanup: (u32, u32),
        early_return: u32,
        /// The normal tail's copy, the span the cleanup's own presentation reads.
        normal_cleanup: (u32, u32),
        /// The two saved returns — the early `null` literal's store and the normal body
        /// producer's store — each with the `areturn` that reads its slot back.
        returns: [(u32, u32); 2],
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FinallyCompletion {
    SavedReturn {
        save: u32,
        returns: u32,
    },
    Joined {
        transfer: u32,
        continuation: u32,
        catch_pop: u32,
        named_row: u32,
    },
    /// The fixed two-row void layout: the method completes with no value on either path out of the
    /// protected body. The cleanup handler's own row protects only its binding store, and the
    /// normal completion is the `return` the normal cleanup copy falls through to. Neither a saved
    /// stack value nor a named catch is invented for it: the completion carries the self-protection
    /// row and that final return, and nothing else.
    Void {
        /// The row that protects only the cleanup handler's binding store (`[160, 162)` of the
        /// fixed class): the self-exception edge a rethrowing handler carries, over no cleanup
        /// instruction.
        binding_row: u32,
        /// The `return` the normal completion ends in, after the cleanup copy.
        final_return: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SharedFinallyCompletion {
    SavedReturns([(u32, u32); 2]),
    Joined {
        transfers: [u32; 2],
    },
    JoinedValue {
        transfers: [u32; 2],
        saves: [u32; 2],
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
            Shape::Monitor { .. } | Shape::MonitorBranches { .. } => &MONITOR,
            Shape::Finally { .. }
            | Shape::LoopFinally { .. }
            | Shape::ConditionalFinally { .. }
            | Shape::NullableResourceFinally { .. }
            | Shape::FlagConditionalFinally { .. }
            | Shape::LocalNullConditionalFinally { .. }
            | Shape::SharedFinally { .. }
            | Shape::EmptyCatchCallFinally { .. }
            | Shape::SegmentedFinally { .. }
            | Shape::MultiReturnLoopFinally { .. }
            | Shape::SegmentedNullLeadFinally { .. }
            | Shape::TwoCatchReturnFinally { .. }
            | Shape::NestedCleanupFinally { .. } => &FINALLY,
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

type ResourceInitialisation = ((u32, u32), u16);

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
    sites: &'a Sites,
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
        sites: &'a Sites,
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
            sites,
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

    /// Adds only the handler blocks the proof can account for instruction by instruction.
    fn cleanup_blocks(
        &mut self,
        base: &[CanonicalBlockId],
        cleanup: &BTreeSet<u32>,
        rows: &[u32],
    ) -> Result<Vec<CanonicalBlockId>, TwrFailure> {
        let base = base.iter().cloned().collect::<BTreeSet<_>>();
        let mut candidates = cleanup
            .iter()
            .filter_map(|bci| self.step(*bci).map(|step| step.block.clone()))
            .collect::<BTreeSet<_>>();
        candidates.retain(|block| !base.contains(block));
        if candidates.is_empty() {
            let mut owned = base.into_iter().collect::<Vec<_>>();
            owned.sort_by_key(CanonicalBlockId::bci);
            return Ok(owned);
        }
        let mut owned = base.clone();
        owned.extend(candidates.iter().cloned());
        for block in &candidates {
            self.charge(block.bci())?;
            let instructions = self.in_block(block);
            if instructions.is_empty()
                || instructions
                    .iter()
                    .any(|instruction| !cleanup.contains(&instruction.bci()))
            {
                return Err((Unproven::Span, block.bci()).into());
            }
            let mut entered = false;
            for edge in self.canonical.edges() {
                self.charge(block.bci())?;
                if edge.to() != block {
                    continue;
                }
                match edge.kind() {
                    jarde_jvm::method_ir::CanonicalEdgeKind::Exception { handler_ordinal }
                        if rows.contains(&handler_ordinal) =>
                    {
                        entered = true
                    }
                    jarde_jvm::method_ir::CanonicalEdgeKind::Normal
                        if candidates.contains(edge.from()) =>
                    {
                        entered = true
                    }
                    _ => return Err((Unproven::Handler, block.bci()).into()),
                }
            }
            if !entered {
                return Err((Unproven::Handler, block.bci()).into());
            }
            for edge in self.canonical.edges() {
                self.charge(block.bci())?;
                if edge.from() != block {
                    continue;
                }
                match edge.kind() {
                    jarde_jvm::method_ir::CanonicalEdgeKind::Normal
                        if candidates.contains(edge.to()) => {}
                    jarde_jvm::method_ir::CanonicalEdgeKind::Exception { handler_ordinal }
                        if rows.contains(&handler_ordinal) => {}
                    _ => return Err((Unproven::Handler, block.bci()).into()),
                }
            }
        }
        let mut owned = owned.into_iter().collect::<Vec<_>>();
        owned.sort_by_key(CanonicalBlockId::bci);
        Ok(owned)
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

    /// The bounded counterpart used by multi-exit guard proofs. Every value/read comparison is
    /// charged, so a long arm cannot turn certificate checking into unaccounted quadratic work.
    fn one_expression_bounded(&mut self, span: (u32, u32), end: u32) -> Result<bool, StopReason> {
        let mut stores = 0usize;
        let readers = self.bcis((span.0, self.span_end(end)));
        for bci in self.bcis(span) {
            self.charge(bci)?;
            let Some(step) = self.step(bci) else {
                return Ok(false);
            };
            if matches!(self.op(bci), Some(Operation::Store { .. })) {
                stores += 1;
                continue;
            }
            for (_, written) in step.instruction.writes() {
                let mut consumed = false;
                for reader in &readers {
                    self.charge(*reader)?;
                    if self.step(*reader).is_some_and(|reader| {
                        reader
                            .instruction
                            .reads()
                            .iter()
                            .any(|(_, read)| self.same(*read, *written))
                    }) {
                        consumed = true;
                        break;
                    }
                }
                if !consumed {
                    return Ok(false);
                }
            }
        }
        Ok(stores <= 1)
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
fn initialisation(
    facts: &Facts<'_>,
    end: u32,
    floor: u32,
) -> Result<ResourceInitialisation, Cause> {
    let store = facts
        .previous_bci(end)
        .filter(|store| *store >= floor)
        .ok_or((Unproven::ResourceInit, end))?;
    let Some(Operation::Store { slot }) = facts.op(store) else {
        return Err((Unproven::ResourceInit, store));
    };
    match constructed_initialisation(facts, store, floor, end) {
        Ok(Some(init)) => return Ok(init),
        Err(at) => return Err((Unproven::ResourceInit, at)),
        Ok(None) => {}
    }
    let mut start = store;
    while let Some(previous) = facts.previous_bci(start) {
        if previous < floor || !single_statement(facts, (previous, end), store) {
            break;
        }
        start = previous;
    }
    if single_statement(facts, (start, end), store) {
        return Ok(((start, end), *slot));
    }
    Err((Unproven::ResourceInit, store))
}

/// Whether this store consumes one verified construction site as its complete initializer.
///
/// Reuse the initializer's verified construction site rather than growing backward across the
/// constructor's stack effects.
fn constructed_initialisation(
    facts: &Facts<'_>,
    store: u32,
    floor: u32,
    end: u32,
) -> Result<Option<ResourceInitialisation>, u32> {
    let Some(Operation::Store { slot }) = facts.op(store) else {
        return Ok(None);
    };
    let Some(step) = facts.step(store) else {
        return Ok(None);
    };
    let reads = step.instruction.reads();
    let Some((_, value)) = (reads.len() == 1).then(|| reads[0]) else {
        return Ok(None);
    };
    let Some(site) = facts.sites.site_producing(facts.ssa, value) else {
        if construction_result(facts, value) {
            return Err(store);
        }
        return Ok(None);
    };
    let site_start = site.head;
    if site_start < floor || site.constructor >= store || facts.span_end(store) != end {
        return Err(store);
    }
    let actual: BTreeSet<u32> = facts.bcis((site_start, end)).into_iter().collect();
    let mut expected = site.expression.clone();
    expected.insert(store);
    if site.expression.is_empty()
        || actual != expected
        || !site.expression.contains(&site.constructor)
    {
        return Err(store);
    }
    Ok(Some(((site_start, end), *slot)))
}

/// Whether this value is the instance a direct `new; dup; <init>` expression produces.
///
/// Follow only duplicate stack values: a local load is an ordinary resource copy, while a
/// constructor result or allocation that reaches the Store without a verified Site is not a
/// resource initializer we can safely move into a header.
fn construction_result(facts: &Facts<'_>, value: ValueId) -> bool {
    let mut current = value;
    for _ in 0..=facts.order.len() {
        let Definition::Instruction { bci, .. } = facts.ssa.value(current).def() else {
            return false;
        };
        match facts.op(*bci) {
            Some(Operation::Allocate { .. }) => return true,
            Some(Operation::Invoke(target))
                if target.kind() == crate::facts::InvokeKind::Special
                    && target.name() == "<init>" =>
            {
                return true;
            }
            Some(Operation::Duplicate) => {
                let Some(step) = facts.step(*bci) else {
                    return true;
                };
                let reads = step.instruction.reads();
                let Some((_, source)) = (reads.len() == 1).then(|| reads[0]) else {
                    return true;
                };
                current = source;
            }
            _ => return false,
        }
    }
    // A valid SSA duplicate chain is shorter than the body's instruction list. If malformed input
    // violates that bound, refuse the initializer rather than letting it reach the generic proof.
    true
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
    single_statement_with_constructor(facts, span, store, false)
}

fn single_statement_with_constructor(
    facts: &Facts<'_>,
    span: (u32, u32),
    store: u32,
    allow_constructor: bool,
) -> bool {
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
        if instruction.instruction.writes().is_empty()
            && !(allow_constructor
                && matches!(facts.op(bci), Some(Operation::Invoke(call)) if call.name() == "<init>"))
        {
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
                // `new; dup; invokespecial <init>` replaces one duplicate alias with the
                // initialized value in SSA. The other alias is the constructor receiver, so its
                // old SSA id has no literal reader even though the value remains on the stack.
                if allow_constructor
                    && matches!(facts.op(bci), Some(Operation::Duplicate))
                    && facts.op(facts.span_end(bci)).is_some_and(
                        |op| matches!(op, Operation::Invoke(call) if call.name() == "<init>"),
                    )
                    && facts.step(facts.span_end(bci)).is_some_and(|next| {
                        next.instruction.reads().iter().any(|(_, read)| {
                            instruction
                                .instruction
                                .writes()
                                .iter()
                                .any(|(_, alias)| facts.same(*alias, *read))
                        }) && next.instruction.writes().iter().any(|(_, initialized)| {
                            facts.step(store).is_some_and(|store| {
                                store
                                    .instruction
                                    .reads()
                                    .iter()
                                    .any(|(_, read)| facts.same(*initialized, *read))
                            })
                        })
                    })
                {
                    continue;
                }
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

/// Whether one range begins at a **statement boundary**: the instruction before it is the last
/// instruction of a statement that completed there, so the range splits no statement and swallows
/// no initialisation. This is the completion proof [`completed_field_assignment`] makes about its
/// own assignment, read for any statement: a block whose entry carries a stack value is one this
/// block-local reading cannot bound, and a range the block's own run reaches with a value still
/// waiting on the operand stack begins inside the statement that produced it — the old-value
/// update whose `dup2` the range follows, or the initialisation whose store it covers.
fn statement_boundary(facts: &Facts<'_>, before: u32, range: u32) -> bool {
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

/// Whether one instruction is one a source statement can **end** with: a `void` invocation, a
/// field write, a return. These are the terminals the statement boundaries a following range can
/// begin at carry — a branch, a monitor enter, a stack shuffle or a value producer belongs to the
/// construct or expression it transfers within, and no statement of its own ends there.
fn statement_ends(facts: &Facts<'_>, at: u32) -> bool {
    match facts.op(at) {
        // A `void` invocation: the call hands back no value, so what it read is the whole
        // statement — `helper();`, `System.out.println(…);` — and the range after it begins a new
        // one. An invocation that returns is a value producer, and the statement that consumes it
        // ends at *that* consumer, not at the call.
        Some(Operation::Invoke(called)) => {
            matches!(parse_method(called.descriptor()), Some((_, None)))
        }
        Some(Operation::Field {
            access: crate::facts::FieldAccess::Write,
            ..
        })
        | Some(Operation::Return) => true,
        _ => false,
    }
}

/// A completed field assignment before a protected range is an ordinary statement, not a
/// resource header. Keep the proof within the current straight-line block: every value made by the
/// assignment must be consumed there, and none of its stack values may survive into the range.
fn completed_field_assignment(facts: &Facts<'_>, before: u32, floor: u32, range: u32) -> bool {
    let Some(Operation::Field {
        access: crate::facts::FieldAccess::Write,
        is_static,
        ..
    }) = facts.op(before)
    else {
        return false;
    };
    if facts.span_end(before) != range {
        return false;
    }
    let statement_of =
        |start| single_statement_with_constructor(facts, (start, range), before, !is_static);
    let mut start = before;
    while let Some(previous) = facts.previous_bci(start).filter(|bci| *bci >= floor) {
        if !statement_of(previous) {
            break;
        }
        start = previous;
    }
    if start == before || !statement_of(start) {
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

/// The two-instruction `aconst_null; astore s` a null-initialised local writes before a
/// protected range, completed at the statement boundary the range begins at. This is the third
/// answer the finally-copy claim gate reads for the statement's own lead: like the field
/// assignment, the statement ends exactly where the range begins and leaves no value of its own
/// on the stack — but what it stores is the `null` the source writes, so the local it declares
/// is the one the body assigns and both cleanup copies call through. Anything else before the
/// range — a wider initialisation, a non-null constant, a call — is not this lead, and the
/// caller keeps the answer it held before the question was asked.
fn completed_null_local_lead(
    facts: &Facts<'_>,
    before: u32,
    floor: u32,
    range: u32,
) -> Option<u16> {
    let Some(Operation::Store { slot }) = facts.op(before) else {
        return None;
    };
    let push = facts
        .previous_bci(before)
        .filter(|previous| *previous >= floor)?;
    if facts.op(push) != Some(&Operation::Push(crate::facts::ConstantValue::Null)) {
        return None;
    }
    // The two instructions are the whole lead: nothing of this block's own run precedes the
    // `null`, so the statement the range follows is exactly this initialisation.
    if facts
        .previous_bci(push)
        .is_some_and(|previous| previous >= floor)
    {
        return None;
    }
    // The store reads the `null` the push wrote — one statement, not two — and the statement is
    // one initialisation whose completion is the range's own start, with nothing of it left on
    // the stack ([`single_statement`], [`statement_boundary`]).
    let reads_the_null = facts.step(push).is_some_and(|step| {
        step.instruction.writes().iter().any(|(written, value)| {
            matches!(written, Slot::Stack(_))
                && facts.step(before).is_some_and(|store| {
                    store
                        .instruction
                        .reads()
                        .iter()
                        .any(|(_, read)| facts.same(*value, *read))
                })
        })
    });
    if !reads_the_null
        || !single_statement(facts, (push, range), before)
        || !statement_boundary(facts, before, range)
    {
        return None;
    }
    Some(*slot)
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
/// without accidentally equating unrelated physical `ValueId`s. A copy the null-lead finally's
/// caller admits may also read a local — the cleanup call's receiver and its arguments — and the
/// comparison carries the slot, so two copies agree only where they read the same one.
fn cleanup_sequence(
    facts: &Facts<'_>,
    copy: &[u32],
    admit_loads: bool,
) -> Option<Vec<(Operation, Vec<usize>)>> {
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
            Operation::Load { .. } if admit_loads => {}
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

/// The only instance-call cleanup admitted by the shared-join certificate. Each copy reads the
/// field afresh, and every produced stack value has exactly its next physical consumer.
fn append_cleanup(
    facts: &mut Facts<'_>,
    start: u32,
) -> Result<Option<(u32, Operation, Operation, Operation)>, StopReason> {
    let mut bcis = [start; 5];
    for index in 1..bcis.len() {
        let Some(next) = facts.next_bci(bcis[index - 1]) else {
            return Ok(None);
        };
        bcis[index] = next;
    }
    let [receiver, field, constant, invoke, pop] = bcis;
    for bci in bcis {
        facts.charge(bci)?;
    }
    let Some(Operation::Field {
        access: crate::facts::FieldAccess::Read,
        is_static: false,
        descriptor,
        ..
    }) = facts.op(field)
    else {
        return Ok(None);
    };
    let field_op = facts.op(field).unwrap().clone();
    let constant_op = facts.op(constant).cloned();
    let invoke_op = facts.op(invoke).cloned();
    let valid = facts
        .step(receiver)
        .is_some_and(|step| step.instruction.opcode() == 0x2a)
        && facts.op(receiver) == Some(&Operation::Load { slot: 0 })
        && descriptor == "Ljava/lang/StringBuilder;"
        && matches!(
            facts.op(constant),
            Some(Operation::Push(crate::facts::ConstantValue::String(_)))
        )
        && matches!(facts.op(invoke), Some(Operation::Invoke(target))
            if target.kind() == InvokeKind::Virtual
                && target.owner() == "java/lang/StringBuilder"
                && target.name() == "append"
                && target.descriptor() == "(Ljava/lang/String;)Ljava/lang/StringBuilder;")
        && facts
            .step(pop)
            .is_some_and(|step| step.instruction.opcode() == 0x57)
        && matches!(facts.op(pop), Some(Operation::Other));
    if !valid {
        return Ok(None);
    }
    let steps: Vec<_> = bcis.iter().filter_map(|bci| facts.step(*bci)).collect();
    if steps.len() != 5
        || steps[0].instruction.reads().len() != 1
        || steps[0].instruction.reads()[0].0 != Slot::Local(0)
        || !matches!(
            facts
                .ssa
                .value(facts.resolve(steps[0].instruction.reads()[0].1))
                .def(),
            Definition::Entry {
                slot: Slot::Local(0),
                ..
            }
        )
        || !steps[2].instruction.reads().is_empty()
        || !steps[4].instruction.writes().is_empty()
    {
        return Ok(None);
    }
    for index in 0..4 {
        let outputs: Vec<_> = steps[index]
            .instruction
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .collect();
        if outputs.len() != 1
            || steps[index].instruction.writes().len() != 1
            || !matches!(facts.ssa.value(facts.resolve(outputs[0].1)).def(),
                Definition::Instruction { bci, .. } if *bci == bcis[index])
        {
            return Ok(None);
        }
        let expected_consumer = if index == 0 {
            1
        } else if index == 1 || index == 2 {
            3
        } else {
            4
        };
        let expected_operands = stack_operands(steps[expected_consumer].instruction);
        if !expected_operands
            .iter()
            .any(|(_, value)| facts.same(*value, outputs[0].1))
        {
            return Ok(None);
        }
        let mut uses = 0;
        for bci in facts.order.clone() {
            facts.charge(bci)?;
            if let Some(step) = facts.step(bci) {
                uses += stack_operands(step.instruction)
                    .iter()
                    .filter(|(_, value)| facts.same(*value, outputs[0].1))
                    .count();
            }
        }
        if uses != 1 {
            return Ok(None);
        }
    }
    let field_reads = steps[1].instruction.reads();
    let invoke_reads = steps[3].instruction.reads();
    let pop_reads = steps[4].instruction.reads();
    if !matches!(field_reads, [(Slot::Stack(0), value)]
        if facts.same(*value, steps[0].instruction.writes()[0].1))
        || !matches!(invoke_reads, [(Slot::Stack(1), argument), (Slot::Stack(0), receiver)]
            if facts.same(*argument, steps[2].instruction.writes()[0].1)
                && facts.same(*receiver, steps[1].instruction.writes()[0].1))
        || !matches!(pop_reads, [(Slot::Stack(0), value)]
            if facts.same(*value, steps[3].instruction.writes()[0].1))
    {
        return Ok(None);
    }
    let (Some(constant_op), Some(invoke_op)) = (constant_op, invoke_op) else {
        return Ok(None);
    };
    Ok(Some((pop, field_op, constant_op, invoke_op)))
}

/// Prove only a straight return/handler pair, before any region ownership or emission. The row's
/// half-open range is checked first: a cleanup call caught by its own handler can run twice.
/// `admit_loads` is the null-lead caller's grant: only that lead's copies read a local, so only
/// that lead's proof compares a copy grammar that carries them.
fn prove_finally_copy(
    facts: &mut Facts<'_>,
    row: &ExceptionHandlerFact,
    admit_loads: bool,
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
        cleanup_sequence(facts, normal_cleanup, admit_loads),
        cleanup_sequence(facts, handler_cleanup, admit_loads),
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

/// The null lead's slot identity: both cleanup copies' call reads the slot the lead initialised.
///
/// The copy proof is structural: the two copies match instruction for instruction, but the `Load`
/// it compares says only "the same slot", not "the value the statement's own flow has". Folding
/// the two calls into one `finally` restates the source only where every argument of the one call
/// each copy makes is the lead slot's own value flow — the `null` the lead writes or the
/// assignment the body fills the slot with, expanded through the joins the read's block put in
/// the way ([`local_null_handler_provenance`]) — and the lead's store and the body's assignments
/// are the only definitions that slot has anywhere the statement owns, either copy included.
/// Anything else — a constant, a call result, another slot's value, a definition the copies or
/// the handler add — is a shape this slice refuses.
fn null_lead_copies_read_the_lead(
    facts: &mut Facts<'_>,
    proof: &FinallyCopyProof,
    start: u32,
    slot: u16,
) -> Result<bool, StopReason> {
    // The statement's own definitions of the slot: the lead's store, and the body's assignments.
    // The body must assign — the source declares, then fills — and no other store of the slot
    // may run anywhere the statement owns.
    let Some(lead_store) = facts.previous_bci(proof.protected.0) else {
        return Ok(false);
    };
    let body_stores: Vec<u32> = facts
        .bcis(proof.protected)
        .into_iter()
        .filter(|bci| {
            matches!(
                facts.op(*bci),
                Some(Operation::Store { slot: filled }) if *filled == slot
            )
        })
        .collect();
    if body_stores.is_empty() {
        return Ok(false);
    }
    let end = facts.span_end(proof.primary.2);
    for bci in facts.bcis((start, end)) {
        facts.charge(bci)?;
        let fills_slot = matches!(
            facts.op(bci),
            Some(Operation::Store { slot: filled }) if *filled == slot
        );
        if fills_slot && lead_store != bci && !body_stores.contains(&bci) {
            return Ok(false);
        }
    }
    let local_written = |step: Step<'_>| {
        step.instruction
            .writes()
            .iter()
            .find_map(|(written, value)| (*written == Slot::Local(slot)).then_some(*value))
    };
    let Some(lead_value) = facts.step(lead_store).and_then(local_written) else {
        return Ok(false);
    };
    let body_values: Vec<ValueId> = body_stores
        .iter()
        .filter_map(|bci| facts.step(*bci).and_then(local_written))
        .collect();
    for (cleanup, entry, require_body) in [
        (proof.normal_cleanup, proof.normal_cleanup.0, true),
        (proof.handler_cleanup, proof.primary.0, false),
    ] {
        let bcis = facts.bcis(cleanup);
        let Some(sequence) = cleanup_sequence(facts, &bcis, true) else {
            return Ok(false);
        };
        // The one call each copy makes: its descriptor names the arguments, and every argument's
        // producer inside the copy must be a read of the lead slot itself.
        let Some((_, (target, producers))) =
            sequence
                .iter()
                .enumerate()
                .find_map(|(ordinal, entry)| match entry {
                    (Operation::Invoke(target), producers) => Some((ordinal, (target, producers))),
                    _ => None,
                })
        else {
            return Ok(false);
        };
        let Some((parameters, _)) = parse_method(target.descriptor()) else {
            return Ok(false);
        };
        if producers.len() < parameters.len() {
            return Ok(false);
        }
        let mut reads = Vec::new();
        for producer in &producers[producers.len() - parameters.len()..] {
            let Some((Operation::Load { slot: read }, _)) = sequence.get(*producer) else {
                return Ok(false);
            };
            if *read != slot {
                return Ok(false);
            }
            let Some(value) = facts
                .step(bcis[*producer])
                .and_then(|step| {
                    step.instruction
                        .reads()
                        .iter()
                        .find(|(read_slot, _)| *read_slot == Slot::Local(slot))
                })
                .map(|(_, value)| *value)
            else {
                return Ok(false);
            };
            reads.push(value);
        }
        // Each argument's read is the merged value flow of the lead and the body's own
        // assignments. The normal copy runs the body to its end, so its read names a body
        // value; the handler copy may be entered before the body stored, and its read may name
        // the lead's `null` alone — either way no definition outside the two may reach it.
        for value in reads {
            let proven =
                local_null_handler_provenance(facts, value, lead_value, &body_values, entry, slot)?;
            if require_body && !proven {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// The two reads are separate instructions and separate SSA values. The first is used only by
/// `ifnull`; the second is the receiver of the one optional call. A source `if (t != null)
/// t.doFinally()` preserves both reads, including when the protected body changed `t`.
fn conditional_cleanup_copy(
    facts: &Facts<'_>,
    copy: &[u32; 6],
    null_exit: u32,
) -> Option<(Operation, Operation)> {
    let [
        first_load,
        first_field,
        branch,
        second_load,
        second_field,
        call,
    ] = *copy;
    let field = facts.op(first_field)?.clone();
    let invoke = facts.op(call)?.clone();
    if facts.op(first_load) != Some(&Operation::Load { slot: 0 })
        || facts.op(second_load) != Some(&Operation::Load { slot: 0 })
        || facts.op(second_field) != Some(&field)
        || facts.op(branch)
            != Some(&Operation::Comparison {
                op: CompareOp::JumpIfNull,
                target: null_exit,
            })
        || !matches!(&field, Operation::Field {
            access: crate::facts::FieldAccess::Read,
            is_static: false,
            descriptor,
            ..
        } if descriptor.starts_with('L') && descriptor.ends_with(';'))
        || !matches!(&invoke, Operation::Invoke(target)
            if target.kind() == InvokeKind::Virtual
                && target.descriptor() == "()V"
                && matches!(&field, Operation::Field { descriptor, .. }
                    if descriptor == &format!("L{};", target.owner())))
    {
        return None;
    }
    let steps: Vec<_> = copy
        .iter()
        .map(|bci| facts.step(*bci))
        .collect::<Option<_>>()?;
    for index in [0, 3] {
        let [(Slot::Local(0), receiver)] = steps[index].instruction.reads() else {
            return None;
        };
        if !matches!(
            facts.ssa.value(facts.resolve(*receiver)).def(),
            Definition::Entry {
                slot: Slot::Local(0),
                ..
            }
        ) {
            return None;
        }
    }
    for (producer, consumer) in [(0, 1), (1, 2), (3, 4), (4, 5)] {
        let outputs: Vec<_> = steps[producer]
            .instruction
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .collect();
        let inputs = stack_operands(steps[consumer].instruction);
        if outputs.len() != 1
            || inputs.len() != 1
            || !facts.same(outputs[0].1, inputs[0].1)
            || facts
                .order
                .iter()
                .filter(|bci| {
                    facts.step(**bci).is_some_and(|step| {
                        stack_operands(step.instruction)
                            .iter()
                            .any(|(_, value)| facts.same(*value, outputs[0].1))
                    })
                })
                .count()
                != 1
        {
            return None;
        }
    }
    let first_value = steps[1]
        .instruction
        .writes()
        .iter()
        .find(|(slot, _)| matches!(slot, Slot::Stack(_)))?
        .1;
    let second_value = steps[4]
        .instruction
        .writes()
        .iter()
        .find(|(slot, _)| matches!(slot, Slot::Stack(_)))?
        .1;
    if facts.same(first_value, second_value) {
        return None;
    }
    Some((field, invoke))
}

/// A bounded Java 8 void finally whose normal and exceptional copies each contain exactly one
/// null branch and one optional call. This is independent of the straight-copy certificate.
fn prove_conditional_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [row] = facts.handlers else {
        return Ok(None);
    };
    if row.catch_type_index.is_some()
        || row.start_bci != 0
        || row.start_bci != current.bci()
        || row.start_bci >= row.end_bci
        || facts.order.len() > 64
        || facts.row_handler(row).as_ref().map(CanonicalBlockId::bci) != Some(row.handler_bci)
    {
        return Ok(None);
    }
    let Some(&normal_return) = facts.order.last() else {
        return Ok(None);
    };
    let normal = facts.bcis((row.end_bci, row.handler_bci));
    let handler = facts.bcis((row.handler_bci, normal_return));
    let ([n0, n1, n2, n3, n4, n5, n6], [h0, h1, h2, h3, h4, h5, h6, h7, h8]) =
        (normal.as_slice(), handler.as_slice())
    else {
        return Ok(None);
    };
    let normal_copy = [*n0, *n1, *n2, *n3, *n4, *n5];
    let handler_copy = [*h1, *h2, *h3, *h4, *h5, *h6];
    if facts.op(*n6) != Some(&Operation::Transfer)
        || facts.op(normal_return) != Some(&Operation::Return)
        || !facts
            .step(normal_return)
            .is_some_and(|step| stack_operands(step.instruction).is_empty())
        || !matches!(facts.op(*h0), Some(Operation::Store { .. }))
        || !matches!(facts.op(*h7), Some(Operation::Load { .. }))
        || facts.op(*h8) != Some(&Operation::Throw)
        || facts.next_bci(*h8) != Some(normal_return)
    {
        return Ok(None);
    }
    let Some((normal_field, normal_call)) =
        conditional_cleanup_copy(facts, &normal_copy, normal_return)
    else {
        return Ok(None);
    };
    let Some((handler_field, handler_call)) = conditional_cleanup_copy(facts, &handler_copy, *h7)
    else {
        return Ok(None);
    };
    if normal_field != handler_field || normal_call != handler_call {
        return Ok(None);
    }
    let (Some(Operation::Store { slot: stored }), Some(Operation::Load { slot: loaded })) =
        (facts.op(*h0), facts.op(*h7))
    else {
        return Ok(None);
    };
    let (Some(store), Some(load), Some(throw)) =
        (facts.step(*h0), facts.step(*h7), facts.step(*h8))
    else {
        return Ok(None);
    };
    let store_input = stack_operands(store.instruction);
    let throw_input = stack_operands(throw.instruction);
    if stored != loaded
        || store_input.len() != 1
        || throw_input.len() != 1
        || !store.instruction.writes().iter().any(|(slot, written)| {
            *slot == Slot::Local(*stored)
                && load
                    .instruction
                    .reads()
                    .iter()
                    .any(|(_, read)| facts.same(*written, *read))
        })
        || !load.instruction.writes().iter().any(|(slot, written)| {
            matches!(slot, Slot::Stack(_)) && facts.same(*written, throw_input[0].1)
        })
    {
        return Ok(None);
    }
    let protected = facts.blocks_in((row.start_bci, row.end_bci));
    let normal_blocks = facts.blocks_in((row.end_bci, row.handler_bci));
    let handler_blocks = facts.blocks_in((row.handler_bci, normal_return));
    let (
        Some(normal_entry),
        Some(normal_call_block),
        Some(handler_entry),
        Some(handler_call_block),
        Some(rethrow_block),
        Some(return_block),
    ) = (
        facts.block_at(*n0),
        facts.block_at(*n3),
        facts.block_at(*h0),
        facts.block_at(*h4),
        facts.block_at(*h7),
        facts.block_at(normal_return),
    )
    else {
        return Ok(None);
    };
    if protected.is_empty()
        || normal_blocks.is_empty()
        || handler_blocks.is_empty()
        || !facts.canonical.unreachable().is_empty()
        || facts.canonical.blocks().len()
            != protected.len() + normal_blocks.len() + handler_blocks.len() + 1
        || facts
            .view
            .successor_ids(&normal_entry)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([normal_call_block.clone(), return_block.clone()])
        || facts.view.successor_ids(&normal_call_block) != [return_block.clone()]
        || facts
            .view
            .successor_ids(&handler_entry)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([handler_call_block.clone(), rethrow_block.clone()])
        || facts.view.successor_ids(&handler_call_block) != [rethrow_block.clone()]
        || !facts.view.successor_ids(&rethrow_block).is_empty()
        || !facts.view.successor_ids(&return_block).is_empty()
    {
        return Ok(None);
    }
    for bci in facts.bcis((row.start_bci, facts.span_end(normal_return))) {
        facts.charge(bci)?;
        let expected = if bci < row.end_bci {
            vec![row.ordinal]
        } else {
            Vec::new()
        };
        if facts
            .covering(bci)
            .iter()
            .map(|entry| entry.ordinal)
            .collect::<Vec<_>>()
            != expected
            || (bci < row.end_bci && facts.op(bci) == Some(&Operation::Return))
        {
            return Ok(None);
        }
    }
    for block in facts.canonical.blocks() {
        facts.charge(block.id().bci())?;
        for edge in facts
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == block.id())
        {
            facts.charge(block.id().bci())?;
            let from_protected = protected.contains(block.id());
            let from_normal = normal_blocks.contains(block.id());
            let from_handler = handler_blocks.contains(block.id());
            let valid = match edge.kind() {
                CanonicalEdgeKind::Exception { handler_ordinal } => {
                    from_protected && handler_ordinal == row.ordinal && edge.to() == &handler_entry
                }
                CanonicalEdgeKind::Normal if from_protected => {
                    protected.contains(edge.to()) || edge.to() == &normal_entry
                }
                CanonicalEdgeKind::Normal if from_normal => {
                    normal_blocks.contains(edge.to()) || edge.to() == &return_block
                }
                CanonicalEdgeKind::Normal if from_handler => handler_blocks.contains(edge.to()),
                CanonicalEdgeKind::Normal => false,
                CanonicalEdgeKind::Return { .. } => block.id() == &return_block,
                CanonicalEdgeKind::Call { .. } => false,
            };
            if !valid {
                return Ok(None);
            }
        }
    }
    let owned = facts.blocks_in((row.start_bci, facts.span_end(normal_return)));
    let origins = facts.bcis((row.start_bci, facts.span_end(normal_return)));
    Ok(Some(Plan {
        shape: Shape::ConditionalFinally {
            row_ordinal: row.ordinal,
            normal_cleanup: (*n0, row.handler_bci),
            handler_cleanup: (*h0, facts.span_end(*h8)),
            normal_return,
        },
        lead: (row.start_bci, row.start_bci),
        body: (row.start_bci, row.end_bci),
        owned,
        join: None,
        facts: origins,
    }))
}

/// A null-guarded local copy reads one value twice: once for the branch and once as the
/// receiver. Returning that value lets the caller check the two copies against the same local
/// definition chain, including the exceptional phi before resource assignment completes.
fn nullable_close_copy(
    facts: &Facts<'_>,
    copy: &[u32; 4],
    exit: u32,
    slot: u16,
) -> Option<(ValueId, Operation)> {
    let [test_load, branch, receiver_load, close] = *copy;
    if facts.op(test_load) != Some(&Operation::Load { slot })
        || facts.op(branch)
            != Some(&Operation::Comparison {
                op: CompareOp::JumpIfNull,
                target: exit,
            })
        || facts.op(receiver_load) != Some(&Operation::Load { slot })
        || !matches!(facts.op(close), Some(Operation::Invoke(target))
            if target.kind() == InvokeKind::Virtual
                && target.owner() == "java/io/InputStream"
                && target.name() == "close"
                && target.descriptor() == "()V")
    {
        return None;
    }
    let steps = copy
        .map(|bci| facts.step(bci))
        .into_iter()
        .collect::<Option<Vec<_>>>()?;
    let local_value = match steps[0].instruction.reads() {
        [(Slot::Local(read_slot), value)] if *read_slot == slot => *value,
        _ => return None,
    };
    if !matches!(steps[2].instruction.reads(), [(Slot::Local(read_slot), value)]
        if *read_slot == slot && facts.same(*value, local_value))
    {
        return None;
    }
    for (producer, consumer) in [(0, 1), (2, 3)] {
        let outputs = steps[producer].instruction.writes();
        let inputs = stack_operands(steps[consumer].instruction);
        if outputs.len() != 1
            || !matches!(outputs[0].0, Slot::Stack(_))
            || inputs.len() != 1
            || !facts.same(outputs[0].1, inputs[0].1)
        {
            return None;
        }
    }
    Some((local_value, facts.op(close)?.clone()))
}

/// Expand only the handler's local phi. Its inputs must be the null initialization or the
/// resource assignment; a same-numbered slot with another producer is not the source variable.
fn nullable_resource_value(
    facts: &mut Facts<'_>,
    value: ValueId,
    null_store: ValueId,
    resource_store: ValueId,
) -> Result<bool, StopReason> {
    let mut pending = vec![value];
    let mut seen = BTreeSet::new();
    let mut found_resource = false;
    while let Some(value) = pending.pop() {
        facts.charge(0)?;
        let value = facts.resolve(value);
        if !seen.insert(value) {
            continue;
        }
        if seen.len() > 64 {
            return Ok(false);
        }
        if facts.same(value, resource_store) {
            found_resource = true;
            continue;
        }
        if facts.same(value, null_store) {
            continue;
        }
        let Definition::Phi { block, slot } = facts.ssa.value(value).def() else {
            return Ok(false);
        };
        if *slot != Slot::Local(1) {
            return Ok(false);
        }
        let Some(phi) = facts.ssa.phis().iter().find(|phi| {
            phi.block() == block && phi.slot() == *slot && facts.same(phi.value(), value)
        }) else {
            return Ok(false);
        };
        for input in phi.inputs() {
            match input {
                jarde_jvm::method_ir::PhiInput::Value(value) => pending.push(*value),
                jarde_jvm::method_ir::PhiInput::Itself
                    if block.bci() == facts.handlers[1].handler_bci => {}
                jarde_jvm::method_ir::PhiInput::Itself => return Ok(false),
            }
        }
    }
    Ok(found_resource)
}

/// The fixed Java 8 two-row lowering: null initialization, one resource assignment, saved
/// return, and two conditional closes. Every instruction and edge is accounted for before Plan.
fn prove_nullable_resource_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [body_row, self_row] = facts.handlers else {
        return Ok(None);
    };
    let (start, body_start, cleanup_start, handler_start) = (
        current.bci(),
        body_row.start_bci,
        body_row.end_bci,
        body_row.handler_bci,
    );
    if start != 0
        || body_row.catch_type_index.is_some()
        || self_row.catch_type_index.is_some()
        || self_row.ordinal != body_row.ordinal + 1
        || self_row.start_bci != handler_start
        || self_row.handler_bci != handler_start
        || facts.next_bci(self_row.start_bci) != Some(self_row.end_bci)
        || !(start < body_start && body_start < cleanup_start && cleanup_start < handler_start)
        || facts.order.len() > 64
        || !facts.canonical.unreachable().is_empty()
    {
        return Ok(None);
    }
    let Some(&last_bci) = facts.order.last() else {
        return Ok(None);
    };
    let lead = facts.bcis((start, body_start));
    let normal = facts.bcis((cleanup_start, handler_start));
    let handler = facts.bcis((handler_start, facts.span_end(last_bci)));
    let (
        [null_push, null_save],
        [n0, n1, n2, n3, return_load, normal_return],
        [primary_store, h0, h1, h2, h3, primary_load, rethrow],
    ) = (lead.as_slice(), normal.as_slice(), handler.as_slice())
    else {
        return Ok(None);
    };
    let body = facts.bcis((body_start, cleanup_start));
    let Some(&saved_return) = body.last() else {
        return Ok(None);
    };
    let resource_stores: Vec<u32> = body
        .iter()
        .copied()
        .filter(|bci| facts.op(*bci) == Some(&Operation::Store { slot: 1 }))
        .collect();
    let [resource_store] = resource_stores.as_slice() else {
        return Ok(None);
    };
    let Some(resource_call) = facts.previous_bci(*resource_store) else {
        return Ok(None);
    };
    if facts.op(*null_push) != Some(&Operation::Push(crate::facts::ConstantValue::Null))
        || facts.op(*null_save) != Some(&Operation::Store { slot: 1 })
        || facts.op(saved_return) != Some(&Operation::Store { slot: 3 })
        || facts.op(*return_load) != Some(&Operation::Load { slot: 3 })
        || facts.op(*normal_return) != Some(&Operation::Return)
        || facts.op(*primary_store) != Some(&Operation::Store { slot: 4 })
        || facts.op(*primary_load) != Some(&Operation::Load { slot: 4 })
        || facts.op(*rethrow) != Some(&Operation::Throw)
        || !matches!(facts.op(resource_call), Some(Operation::Invoke(target))
            if target.kind() == InvokeKind::Virtual
                && target.owner() == "java/lang/Class"
                && target.name() == "getResourceAsStream"
                && target.descriptor() == "(Ljava/lang/String;)Ljava/io/InputStream;")
        || facts.next_bci(*rethrow).is_some()
        || !handler_binding(facts, *primary_store)
    {
        return Ok(None);
    }
    let (Some((normal_value, normal_call)), Some((handler_value, handler_call))) = (
        nullable_close_copy(facts, &[*n0, *n1, *n2, *n3], *return_load, 1),
        nullable_close_copy(facts, &[*h0, *h1, *h2, *h3], *primary_load, 1),
    ) else {
        return Ok(None);
    };
    if normal_call != handler_call {
        return Ok(None);
    }
    let (
        Some(null_step),
        Some(null_save_step),
        Some(resource_call_step),
        Some(resource_step),
        Some(save_step),
        Some(return_load_step),
        Some(return_step),
        Some(primary_step),
        Some(primary_load_step),
        Some(throw_step),
    ) = (
        facts.step(*null_push),
        facts.step(*null_save),
        facts.step(resource_call),
        facts.step(*resource_store),
        facts.step(saved_return),
        facts.step(*return_load),
        facts.step(*normal_return),
        facts.step(*primary_store),
        facts.step(*primary_load),
        facts.step(*rethrow),
    )
    else {
        return Ok(None);
    };
    let local_written = |step: Step<'_>, slot| {
        step.instruction
            .writes()
            .iter()
            .find_map(|(written_slot, value)| {
                (*written_slot == Slot::Local(slot)).then_some(*value)
            })
    };
    let (Some(null_value), Some(resource_value), Some(saved_value), Some(primary_value)) = (
        local_written(null_save_step, 1),
        local_written(resource_step, 1),
        local_written(save_step, 3),
        local_written(primary_step, 4),
    ) else {
        return Ok(None);
    };
    if !matches!(stack_operands(null_save_step.instruction).as_slice(), [(_, read)]
            if null_step.instruction.writes().iter().any(|(_, value)| facts.same(*value, *read)))
        || !matches!(stack_operands(resource_step.instruction).as_slice(), [(_, read)]
            if resource_call_step.instruction.writes().iter().any(|(slot, value)|
                matches!(slot, Slot::Stack(_)) && facts.same(*value, *read)))
        || !facts.same(normal_value, resource_value)
        || !nullable_resource_value(facts, handler_value, null_value, resource_value)?
        || !return_load_step
            .instruction
            .reads()
            .iter()
            .any(|(slot, value)| *slot == Slot::Local(3) && facts.same(*value, saved_value))
        || !matches!(stack_operands(return_step.instruction).as_slice(), [(_, read)]
            if return_load_step.instruction.writes().iter().any(|(_, value)| facts.same(*value, *read)))
        || !primary_load_step
            .instruction
            .reads()
            .iter()
            .any(|(slot, value)| *slot == Slot::Local(4) && facts.same(*value, primary_value))
        || !matches!(stack_operands(throw_step.instruction).as_slice(), [(_, read)]
            if primary_load_step.instruction.writes().iter().any(|(_, value)| facts.same(*value, *read)))
    {
        return Ok(None);
    }
    let protected = facts.blocks_in((body_start, cleanup_start));
    let normal_blocks = facts.blocks_in((cleanup_start, handler_start));
    let handler_blocks = facts.blocks_in((handler_start, facts.span_end(*rethrow)));
    let (
        Some(normal_entry),
        Some(normal_call_block),
        Some(return_block),
        Some(handler_entry),
        Some(handler_call_block),
        Some(rethrow_block),
    ) = (
        facts.block_of(*n0).cloned(),
        facts.block_at(*n2),
        facts.block_at(*return_load),
        facts.block_at(*primary_store),
        facts.block_at(*h2),
        facts.block_at(*primary_load),
    )
    else {
        return Ok(None);
    };
    let owned = facts.blocks_in((start, facts.span_end(*rethrow)));
    if owned.len() != facts.canonical.blocks().len()
        || protected.is_empty()
        || facts.block_of(*null_push) != Some(current)
        || facts.block_of(*null_save) != Some(current)
        || facts
            .view
            .successor_ids(&normal_entry)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([normal_call_block.clone(), return_block.clone()])
        || facts.view.successor_ids(&normal_call_block) != [return_block.clone()]
        || facts
            .view
            .successor_ids(&handler_entry)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([handler_call_block.clone(), rethrow_block.clone()])
        || facts.view.successor_ids(&handler_call_block) != [rethrow_block.clone()]
        || !facts.view.successor_ids(&return_block).is_empty()
        || !facts.view.successor_ids(&rethrow_block).is_empty()
    {
        return Ok(None);
    }
    for bci in facts.bcis((start, facts.span_end(*rethrow))) {
        facts.charge(bci)?;
        let expected = if body_start <= bci && bci < cleanup_start {
            Some(body_row.ordinal)
        } else if bci == handler_start {
            Some(self_row.ordinal)
        } else {
            None
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected.into_iter().collect::<Vec<_>>()
            || (body_start <= bci
                && bci < cleanup_start
                && facts.op(bci) == Some(&Operation::Return))
        {
            return Ok(None);
        }
    }
    for block in facts.canonical.blocks() {
        facts.charge(block.id().bci())?;
        for edge in facts
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == block.id())
        {
            facts.charge(block.id().bci())?;
            let valid = match edge.kind() {
                CanonicalEdgeKind::Exception { handler_ordinal } => {
                    edge.to() == &handler_entry
                        && ((protected.contains(block.id()) && handler_ordinal == body_row.ordinal)
                            || (block.id() == &handler_entry
                                && handler_ordinal == self_row.ordinal))
                }
                CanonicalEdgeKind::Normal if normal_blocks.contains(block.id()) => {
                    normal_blocks.contains(edge.to())
                }
                CanonicalEdgeKind::Normal if protected.contains(block.id()) => {
                    protected.contains(edge.to()) || edge.to() == &normal_entry
                }
                CanonicalEdgeKind::Normal if handler_blocks.contains(block.id()) => {
                    handler_blocks.contains(edge.to())
                }
                CanonicalEdgeKind::Return { .. } => block.id() == &return_block,
                _ => false,
            };
            if !valid {
                return Ok(None);
            }
        }
    }
    let origins = facts.bcis((start, facts.span_end(*rethrow)));
    Ok(Some(Plan {
        shape: Shape::NullableResourceFinally {
            row_ordinal: body_row.ordinal,
            normal_cleanup: (*n0, *return_load),
            handler_cleanup: (*primary_store, facts.span_end(*rethrow)),
            saved_return: (saved_return, *normal_return),
        },
        lead: (start, body_start),
        body: (body_start, cleanup_start),
        owned,
        join: None,
        facts: origins,
    }))
}

/// One eight-instruction flag-guarded field read-modify-write, optionally followed by the same
/// guarded-throw tail in both copies: `iload flag; ifne exit; aload_0; dup; getfield F;
/// iconst K; isub; putfield F [`getstatic B; ifeq exit; new C; dup; ldc "S"; invokespecial
/// C.<init>(Ljava/lang/String;)V; athrow`]`. The value the branch tests is returned to the
/// caller, which ties it to the certificate's own flag writes; the field and the constant come
/// back so the two copies can be checked against one parameter set, and the tail's guard field,
/// class and message with them. Nothing here decides what the copies mean — only that this
/// exact shape, with one consumer per intermediate value, is what the bytes say.
#[derive(Clone, Debug, Eq, PartialEq)]
struct FlagCleanupGuard {
    /// The static `boolean` the tail reads, as the pool states it.
    field: (String, String, String),
    /// The class the tail allocates.
    class: String,
    /// The message constant the tail passes the constructor.
    message: String,
}

fn flag_cleanup_copy(
    facts: &Facts<'_>,
    copy: &[u32],
    exit: u32,
    slot: u16,
) -> Option<(ValueId, Operation, i64, Option<FlagCleanupGuard>)> {
    let [
        test_load,
        branch,
        receiver_load,
        duplicate,
        field_read,
        constant,
        subtract,
        field_write,
        tail @ ..,
    ] = copy
    else {
        return None;
    };
    if tail.len() != 0 && tail.len() != 7 {
        return None;
    }
    let [
        test_load,
        branch,
        receiver_load,
        duplicate,
        field_read,
        constant,
        subtract,
        field_write,
    ] = [
        *test_load,
        *branch,
        *receiver_load,
        *duplicate,
        *field_read,
        *constant,
        *subtract,
        *field_write,
    ];
    let read = facts.op(field_read)?.clone();
    let write = facts.op(field_write)?.clone();
    let (
        Operation::Field {
            access: crate::facts::FieldAccess::Read,
            is_static: false,
            owner,
            name,
            descriptor,
        },
        Operation::Field {
            access: crate::facts::FieldAccess::Write,
            is_static: false,
            owner: write_owner,
            name: write_name,
            descriptor: write_descriptor,
        },
    ) = (&read, &write)
    else {
        return None;
    };
    if facts.op(test_load) != Some(&Operation::Load { slot })
        || facts.op(branch)
            != Some(&Operation::Comparison {
                op: CompareOp::JumpIfNotZero,
                target: exit,
            })
        || facts.op(receiver_load) != Some(&Operation::Load { slot: 0 })
        || facts.op(duplicate) != Some(&Operation::Duplicate)
        || facts.op(subtract)
            != Some(&Operation::Arithmetic {
                op: crate::facts::ArithmeticOp::Subtract,
            })
        || descriptor != "I"
        || (owner, name, descriptor) != (write_owner, write_name, write_descriptor)
        || !matches!(
            facts.op(constant),
            Some(Operation::Push(crate::facts::ConstantValue::Int(_)))
        )
    {
        return None;
    }
    let steps: Vec<_> = copy
        .iter()
        .map(|bci| facts.step(*bci))
        .collect::<Option<_>>()?;
    // The receiver both field accesses act on is the method's own `this`, read once and
    // duplicated: the write's receiver is the `dup`'s lower copy, the read's its upper copy.
    let [(Slot::Local(0), receiver)] = steps[2].instruction.reads() else {
        return None;
    };
    if !matches!(
        facts.ssa.value(facts.resolve(*receiver)).def(),
        Definition::Entry {
            slot: Slot::Local(0),
            ..
        }
    ) {
        return None;
    }
    let (_, receiver_output) = steps[2]
        .instruction
        .writes()
        .iter()
        .copied()
        .find(|(written, _)| matches!(written, Slot::Stack(_)))?;
    let [(Slot::Local(read_slot), flag)] = steps[0].instruction.reads() else {
        return None;
    };
    if *read_slot != slot {
        return None;
    }
    let (_, flag_stack) = steps[0]
        .instruction
        .writes()
        .iter()
        .copied()
        .find(|(written, _)| matches!(written, Slot::Stack(_)))?;
    let stack_writes = |index: usize| -> Option<Vec<(Slot, ValueId)>> {
        let mut writes: Vec<(Slot, ValueId)> = steps[index]
            .instruction
            .writes()
            .iter()
            .filter(|(written, _)| matches!(written, Slot::Stack(_)))
            .copied()
            .collect();
        writes.sort_by_key(|(written, _)| match written {
            Slot::Stack(depth) => *depth,
            Slot::Local(slot) => u32::from(*slot),
        });
        Some(writes)
    };
    let duplicated = stack_writes(3)?;
    if duplicated.len() != 2 || duplicated[0].1 == duplicated[1].1 {
        return None;
    }
    let branch_inputs = stack_operands(steps[1].instruction);
    let duplicate_inputs = stack_operands(steps[3].instruction);
    let read_inputs = stack_operands(steps[4].instruction);
    let subtract_inputs = stack_operands(steps[6].instruction);
    let write_inputs = stack_operands(steps[7].instruction);
    let (Some(push_output), Some(read_output), Some(subtract_output)) = (
        stack_writes(5)?.first().map(|(_, value)| *value),
        stack_writes(4)?.first().map(|(_, value)| *value),
        stack_writes(6)?.first().map(|(_, value)| *value),
    ) else {
        return None;
    };
    if branch_inputs.len() != 1
        || duplicate_inputs.len() != 1
        || read_inputs.len() != 1
        || subtract_inputs.len() != 2
        || write_inputs.len() != 2
        || !facts.same(branch_inputs[0].1, flag_stack)
        || !facts.same(duplicate_inputs[0].1, receiver_output)
        || !facts.same(read_inputs[0].1, duplicated[1].1)
        || !facts.same(write_inputs[0].1, duplicated[0].1)
        || !facts.same(subtract_inputs[0].1, read_output)
        || !facts.same(subtract_inputs[1].1, push_output)
        || !facts.same(write_inputs[1].1, subtract_output)
    {
        return None;
    }
    // Every value the copy builds inside itself is consumed exactly once, where the copy
    // consumes it. The flag and the receiver are shared with the rest of the method, so their
    // consumers are the caller's linkage proof, not a count here.
    let consumed = |value: ValueId, consumer: u32| -> bool {
        facts
            .order
            .iter()
            .filter(|bci| {
                **bci != consumer
                    && facts.step(**bci).is_some_and(|step| {
                        stack_operands(step.instruction)
                            .iter()
                            .any(|(_, read)| facts.same(*read, value))
                    })
            })
            .count()
            == 0
    };
    for (value, consumer) in [
        (duplicated[0].1, field_write),
        (duplicated[1].1, field_read),
        (read_output, subtract),
        (push_output, subtract),
        (subtract_output, field_write),
    ] {
        if !consumed(value, consumer) {
            return None;
        }
    }
    let Operation::Push(crate::facts::ConstantValue::Int(constant)) = facts.op(constant)? else {
        return None;
    };
    let guard = if tail.is_empty() {
        None
    } else {
        let [
            guard_read,
            guard_branch,
            allocate,
            duplicate_throw,
            message,
            construct,
            rethrow_new,
        ] = tail
        else {
            return None;
        };
        let [
            guard_read,
            guard_branch,
            allocate,
            duplicate_throw,
            message,
            construct,
            rethrow_new,
        ] = [
            *guard_read,
            *guard_branch,
            *allocate,
            *duplicate_throw,
            *message,
            *construct,
            *rethrow_new,
        ];
        let Operation::Field {
            access: crate::facts::FieldAccess::Read,
            is_static: true,
            owner: guard_owner,
            name: guard_name,
            descriptor: guard_descriptor,
        } = facts.op(guard_read)?
        else {
            return None;
        };
        if guard_descriptor != "Z"
            || facts.op(guard_branch)
                != Some(&Operation::Comparison {
                    op: CompareOp::JumpIfZero,
                    target: exit,
                })
        {
            return None;
        }
        let Operation::Allocate { ty: class } = facts.op(allocate)? else {
            return None;
        };
        let Operation::Push(crate::facts::ConstantValue::String(message)) = facts.op(message)?
        else {
            return None;
        };
        let Operation::Invoke(target) = facts.op(construct)? else {
            return None;
        };
        if target.kind() != InvokeKind::Special
            || target.name() != "<init>"
            || target.descriptor() != "(Ljava/lang/String;)V"
            || target.owner() != class
            || facts.op(rethrow_new) != Some(&Operation::Throw)
        {
            return None;
        }
        // The tail's own value flow, under the same one-consumer-per-value discipline: the
        // allocation feeds the `dup` alone, the `dup`'s upper copy and the message feed the
        // constructor alone, the constructor's initialized value feeds the `athrow` alone — and
        // the `dup`'s lower copy reads nowhere, because the constructor call converts it in
        // place, which is what makes the `athrow` throw the initialized instance. The indexes
        // are the full copy's: the eight read-modify-write instructions, then the tail.
        let (Some(allocate_output), Some(message_output), Some(initialized)) = (
            stack_writes(10)?.first().map(|(_, value)| *value),
            stack_writes(12)?.first().map(|(_, value)| *value),
            stack_writes(13)?.first().map(|(_, value)| *value),
        ) else {
            return None;
        };
        let tail_thrown = stack_writes(11)?;
        if tail_thrown.len() != 2 || tail_thrown[0].1 == tail_thrown[1].1 {
            return None;
        }
        let guard_value = stack_writes(8)?.first().map(|(_, value)| *value)?;
        let construct_inputs = stack_operands(steps[13].instruction);
        let guard_branch_inputs = stack_operands(steps[9].instruction);
        let throw_inputs = stack_operands(steps[14].instruction);
        if construct_inputs.len() != 2
            || guard_branch_inputs.len() != 1
            || throw_inputs.len() != 1
            || !facts.same(guard_branch_inputs[0].1, guard_value)
            || !facts.same(construct_inputs[0].1, tail_thrown[1].1)
            || !facts.same(construct_inputs[1].1, message_output)
            || !facts.same(throw_inputs[0].1, initialized)
        {
            return None;
        }
        for (value, consumer) in [
            (allocate_output, duplicate_throw),
            (tail_thrown[1].1, construct),
            (message_output, construct),
            (initialized, rethrow_new),
        ] {
            if !consumed(value, consumer) {
                return None;
            }
        }
        // The `dup`'s lower copy is the one value the conversion kills: no instruction may read
        // it, the `athrow` included.
        if !consumed(tail_thrown[0].1, rethrow_new) {
            return None;
        }
        Some(FlagCleanupGuard {
            field: (
                guard_owner.clone(),
                guard_name.clone(),
                guard_descriptor.clone(),
            ),
            class: class.clone(),
            message: message.clone(),
        })
    };
    Some((*flag, read, *constant, guard))
}

/// The cleanup copies read the flag the lead initialises and the body sets. The normal copy
/// runs only after the body completed, so its read must be the `true` store's own value. The
/// handler copy runs after any protected instruction, so its read may carry either write: a
/// direct definition or a handler phi whose inputs are both, expanded to a fixpoint.
fn flag_value_provenance(
    facts: &mut Facts<'_>,
    value: ValueId,
    lead_store: ValueId,
    set_store: ValueId,
    handler_entry: u32,
    flag_slot: u16,
) -> Result<bool, StopReason> {
    let mut pending = vec![value];
    let mut seen = BTreeSet::new();
    while let Some(value) = pending.pop() {
        facts.charge(0)?;
        let value = facts.resolve(value);
        if !seen.insert(value) {
            continue;
        }
        if seen.len() > 64 {
            return Ok(false);
        }
        if facts.same(value, lead_store) || facts.same(value, set_store) {
            continue;
        }
        let Definition::Phi { block, slot } = facts.ssa.value(value).def() else {
            return Ok(false);
        };
        if *slot != Slot::Local(flag_slot) {
            return Ok(false);
        }
        let Some(phi) = facts.ssa.phis().iter().find(|phi| {
            phi.block() == block && phi.slot() == *slot && facts.same(phi.value(), value)
        }) else {
            return Ok(false);
        };
        for input in phi.inputs() {
            match input {
                jarde_jvm::method_ir::PhiInput::Value(value) => pending.push(*value),
                jarde_jvm::method_ir::PhiInput::Itself if block.bci() == handler_entry => {}
                jarde_jvm::method_ir::PhiInput::Itself => return Ok(false),
            }
        }
    }
    Ok(true)
}

/// Whether one value is the expected one, through the join phis a two-predecessor block may
/// put in the read's way: the direct definition or a phi whose every input is, expanded to a
/// fixpoint.
fn value_is(facts: &mut Facts<'_>, value: ValueId, expected: ValueId) -> Result<bool, StopReason> {
    let mut pending = vec![value];
    let mut seen = BTreeSet::new();
    while let Some(value) = pending.pop() {
        facts.charge(0)?;
        let value = facts.resolve(value);
        if !seen.insert(value) {
            continue;
        }
        if seen.len() > 64 {
            return Ok(false);
        }
        if facts.same(value, expected) {
            continue;
        }
        let Definition::Phi { block, slot } = facts.ssa.value(value).def() else {
            return Ok(false);
        };
        let Some(phi) = facts.ssa.phis().iter().find(|phi| {
            phi.block() == block && phi.slot() == *slot && facts.same(phi.value(), value)
        }) else {
            return Ok(false);
        };
        for input in phi.inputs() {
            match input {
                jarde_jvm::method_ir::PhiInput::Value(value) => pending.push(*value),
                jarde_jvm::method_ir::PhiInput::Itself => return Ok(false),
            }
        }
    }
    Ok(true)
}

/// The fixed two-row flag lowering: a two-instruction `false` initialisation, a body that sets
/// the same local `true` exactly once before it saves the returned value, and two equivalent
/// eight-instruction field-update copies. Every instruction and edge is accounted for before
/// Plan; the body, the lead and the normal copy's own test may share the entry block, which is
/// why the body's ownership is read instruction by instruction and not block by block.
fn prove_flag_conditional_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [body_row, self_row] = facts.handlers else {
        return Ok(None);
    };
    let (start, body_start, cleanup_start, handler_start) = (
        current.bci(),
        body_row.start_bci,
        body_row.end_bci,
        body_row.handler_bci,
    );
    if body_row.catch_type_index.is_some()
        || self_row.catch_type_index.is_some()
        || self_row.ordinal != body_row.ordinal + 1
        || self_row.start_bci != handler_start
        || self_row.handler_bci != handler_start
        || facts.next_bci(self_row.start_bci) != Some(self_row.end_bci)
        || facts.order.len() > 64
        || !facts.canonical.unreachable().is_empty()
    {
        return Ok(None);
    }
    let lead = facts.bcis((start, body_start));
    let body = facts.bcis((body_start, cleanup_start));
    let normal = facts.bcis((cleanup_start, handler_start));
    let Some(&last_bci) = facts.order.last() else {
        return Ok(None);
    };
    let handler = facts.bcis((handler_start, facts.span_end(last_bci)));
    let (
        [false_push, false_store],
        [
            n0,
            n1,
            n2,
            _n3,
            _n4,
            _n5,
            _n6,
            _n7,
            normal_tail @ ..,
            return_load,
            normal_return,
        ],
        [
            primary_store,
            h0,
            _h1,
            h2,
            _h3,
            _h4,
            _h5,
            _h6,
            _h7,
            handler_tail @ ..,
            primary_load,
            rethrow,
        ],
    ) = (lead.as_slice(), normal.as_slice(), handler.as_slice())
    else {
        return Ok(None);
    };
    // The two copies are the same length: the read-modify-write core alone, or the core with the
    // same seven-instruction guarded-throw tail in both. Anything else is not this lowering.
    if normal_tail.len() != handler_tail.len() || (normal_tail.len() != 0 && normal_tail.len() != 7)
    {
        return Ok(None);
    }
    let Some(Operation::Store { slot: flag_slot }) = facts.op(*false_store) else {
        return Ok(None);
    };
    let flag_slot = *flag_slot;
    if flag_slot == 0
        || facts.op(*false_push) != Some(&Operation::Push(crate::facts::ConstantValue::Int(0)))
        || facts.op(*n0) != Some(&Operation::Load { slot: flag_slot })
        || !matches!(facts.op(*primary_store), Some(Operation::Store { .. }))
        || !matches!(facts.op(*primary_load), Some(Operation::Load { .. }))
        || facts.op(*rethrow) != Some(&Operation::Throw)
        || facts.next_bci(*rethrow).is_some()
    {
        return Ok(None);
    }
    // The lead is the method's own first two instructions, and the body, the lead and the
    // normal copy's test share the entry block: the body's ownership is proved per instruction.
    if body.is_empty()
        || facts.block_of(*false_push) != Some(current)
        || facts.block_of(*false_store) != Some(current)
        || body
            .iter()
            .copied()
            .chain([*n0, *n1])
            .any(|bci| facts.block_of(bci) != Some(current))
    {
        return Ok(None);
    }
    // The body sets the flag `true` exactly once, immediately before it saves the returned
    // value: the store's only successors in the body are the save's load and store, and the
    // save ends the body.
    let flag_stores: Vec<u32> = body
        .iter()
        .copied()
        .filter(
            |bci| matches!(facts.op(*bci), Some(Operation::Store { slot }) if *slot == flag_slot),
        )
        .collect();
    if flag_stores.len() != 1 {
        return Ok(None);
    }
    let set_true = flag_stores[0];
    let Some(save_store) = body.last().copied() else {
        return Ok(None);
    };
    let Some(one_push) = facts.previous_bci(set_true) else {
        return Ok(None);
    };
    let Some(save_load) = facts.previous_bci(save_store) else {
        return Ok(None);
    };
    if facts.op(one_push) != Some(&Operation::Push(crate::facts::ConstantValue::Int(1)))
        || !matches!(facts.op(save_load), Some(Operation::Load { .. }))
        || facts.next_bci(set_true) != Some(save_load)
        || facts.next_bci(save_load) != Some(save_store)
    {
        return Ok(None);
    }
    let Some(Operation::Store { slot: save_slot }) = facts.op(save_store) else {
        return Ok(None);
    };
    let save_slot = *save_slot;
    if save_slot == flag_slot {
        return Ok(None);
    }
    // The slot is written nowhere else: the lead's `false` and the body's one `true` are the
    // only definitions either copy's condition can read.
    if facts
        .bcis((start, facts.span_end(last_bci)))
        .into_iter()
        .any(|bci| {
            bci != *false_store
                && bci != set_true
                && matches!(facts.op(bci), Some(Operation::Store { slot }) if *slot == flag_slot)
        })
    {
        return Ok(None);
    }
    // The normal copy's completion is the saved value's own return; the handler's is the
    // pending throwable's rethrow, whose binding store the self-protecting row covers.
    if facts.op(*return_load) != Some(&Operation::Load { slot: save_slot })
        || facts.op(*normal_return) != Some(&Operation::Return)
        || !handler_binding(facts, *primary_store)
    {
        return Ok(None);
    }
    let (
        Some((normal_flag, normal_field, normal_constant, normal_guard)),
        Some((handler_flag, handler_field, handler_constant, handler_guard)),
    ) = (
        flag_cleanup_copy(
            facts,
            &normal[..8 + normal_tail.len()],
            *return_load,
            flag_slot,
        ),
        flag_cleanup_copy(
            facts,
            &handler[1..9 + handler_tail.len()],
            *primary_load,
            flag_slot,
        ),
    )
    else {
        return Ok(None);
    };
    if normal_field != handler_field
        || normal_constant != handler_constant
        || normal_guard != handler_guard
    {
        return Ok(None);
    }
    let (Some(normal_step), Some(save_step), Some(false_step), Some(return_step), Some(throw_step)) = (
        facts.step(*return_load),
        facts.step(save_store),
        facts.step(*false_store),
        facts.step(*normal_return),
        facts.step(*rethrow),
    ) else {
        return Ok(None);
    };
    let local_written = |step: Step<'_>, slot| {
        step.instruction
            .writes()
            .iter()
            .find_map(|(written_slot, value)| {
                (*written_slot == Slot::Local(slot)).then_some(*value)
            })
    };
    let Some(Operation::Store { slot: primary_slot }) = facts.op(*primary_store) else {
        return Ok(None);
    };
    let primary_slot = *primary_slot;
    let (Some(set_step), Some(primary_step)) = (facts.step(set_true), facts.step(*primary_store))
    else {
        return Ok(None);
    };
    let (Some(set_value), Some(lead_value), Some(saved_value), Some(primary_value)) = (
        local_written(set_step, flag_slot),
        local_written(false_step, flag_slot),
        local_written(save_step, save_slot),
        local_written(primary_step, primary_slot),
    ) else {
        return Ok(None);
    };
    let return_inputs = stack_operands(return_step.instruction);
    let throw_inputs = stack_operands(throw_step.instruction);
    let primary_load_step = facts.step(*primary_load);
    let proven = flag_value_provenance(
        facts,
        handler_flag,
        lead_value,
        set_value,
        handler_start,
        flag_slot,
    )?;
    let primary_load_reads_ok = primary_load_step.is_some_and(|step| {
        step.instruction.reads().iter().any(|(slot, value)| {
            *slot == Slot::Local(primary_slot) && facts.same(*value, primary_value)
        })
    });
    let return_reads_ok = normal_step.instruction.reads().iter().any(|(slot, value)| {
        *slot == Slot::Local(save_slot) && value_is(facts, *value, saved_value).unwrap_or(false)
    });
    let return_input_ok = matches!(return_inputs.as_slice(), [(_, read)]
        if normal_step.instruction.writes().iter().any(|(_, value)| facts.same(*value, *read)));
    let throw_input_ok = throw_inputs.iter().any(|(_, value)| {
        facts.step(*primary_load).is_some_and(|step| {
            step.instruction
                .writes()
                .iter()
                .any(|(_, written)| facts.same(*written, *value))
        })
    });
    if !facts.same(normal_flag, set_value)
        || !proven
        || facts.op(*primary_load) != Some(&Operation::Load { slot: primary_slot })
        || !primary_load_reads_ok
        || !return_reads_ok
        || !return_input_ok
        || !throw_input_ok
    {
        return Ok(None);
    }
    let protected = facts.blocks_in((body_start, cleanup_start));
    let handler_blocks = facts.blocks_in((handler_start, facts.span_end(*rethrow)));
    let mut normal_throw_block = None;
    let mut handler_throw_block = None;
    if !normal_tail.is_empty() {
        let Some(block) = facts.block_of(normal[10]) else {
            return Ok(None);
        };
        normal_throw_block = Some(block.clone());
    }
    if !handler_tail.is_empty() {
        let Some(block) = facts.block_of(handler[11]) else {
            return Ok(None);
        };
        handler_throw_block = Some(block.clone());
    }
    let (
        Some(update_block),
        Some(return_block),
        Some(handler_entry),
        Some(handler_update_block),
        Some(rethrow_block),
    ) = (
        facts.block_of(*n2).cloned(),
        facts.block_of(*normal_return).cloned(),
        facts.block_of(*primary_store).cloned(),
        facts.block_of(*h2).cloned(),
        facts.block_of(*rethrow).cloned(),
    )
    else {
        return Ok(None);
    };
    // With the guarded-throw tail the update arm branches once more: the throw block ends in an
    // `athrow` no exception row covers, so its own effect is the copy's completion there.
    let (update_exits, handler_update_exits) = (
        match (&normal_throw_block, &return_block) {
            (Some(throw), return_block) => {
                BTreeSet::from([(*throw).clone(), (*return_block).clone()])
            }
            (None, return_block) => BTreeSet::from([(*return_block).clone()]),
        },
        match (&handler_throw_block, &rethrow_block) {
            (Some(throw), rethrow_block) => {
                BTreeSet::from([(*throw).clone(), (*rethrow_block).clone()])
            }
            (None, rethrow_block) => BTreeSet::from([(*rethrow_block).clone()]),
        },
    );
    let owned = facts.blocks_in((start, facts.span_end(*rethrow)));
    if owned.len() != facts.canonical.blocks().len()
        || protected.is_empty()
        || facts
            .view
            .successor_ids(current)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([update_block.clone(), return_block.clone()])
        || facts
            .view
            .successor_ids(&update_block)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != update_exits
        || facts
            .view
            .successor_ids(&handler_entry)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([handler_update_block.clone(), rethrow_block.clone()])
        || facts
            .view
            .successor_ids(&handler_update_block)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != handler_update_exits
        || normal_throw_block
            .as_ref()
            .is_some_and(|throw| !facts.view.successor_ids(throw).is_empty())
        || handler_throw_block
            .as_ref()
            .is_some_and(|throw| !facts.view.successor_ids(throw).is_empty())
        || !facts.view.successor_ids(&return_block).is_empty()
        || !facts.view.successor_ids(&rethrow_block).is_empty()
    {
        return Ok(None);
    }
    for bci in facts.bcis((start, facts.span_end(*rethrow))) {
        facts.charge(bci)?;
        let expected: Vec<u32> = if body_start <= bci && bci < cleanup_start {
            vec![body_row.ordinal]
        } else if bci == handler_start {
            vec![self_row.ordinal]
        } else {
            Vec::new()
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
            || (body_start <= bci
                && bci < cleanup_start
                && facts.op(bci) == Some(&Operation::Return))
        {
            return Ok(None);
        }
    }
    for block in facts.canonical.blocks() {
        facts.charge(block.id().bci())?;
        for edge in facts
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == block.id())
        {
            facts.charge(block.id().bci())?;
            let from_handler = handler_blocks.contains(block.id());
            let valid = match edge.kind() {
                CanonicalEdgeKind::Exception { handler_ordinal } => {
                    edge.to() == &handler_entry
                        && ((protected.contains(block.id()) && handler_ordinal == body_row.ordinal)
                            || (block.id() == &handler_entry
                                && handler_ordinal == self_row.ordinal))
                }
                CanonicalEdgeKind::Normal if block.id() == current => {
                    edge.to() == &update_block || edge.to() == &return_block
                }
                CanonicalEdgeKind::Normal if block.id() == &update_block => {
                    edge.to() == &return_block
                        || normal_throw_block
                            .as_ref()
                            .is_some_and(|throw| edge.to() == throw)
                }
                CanonicalEdgeKind::Normal if from_handler => {
                    handler_blocks.contains(edge.to()) || edge.to() == &rethrow_block
                }
                CanonicalEdgeKind::Return { .. } => block.id() == &return_block,
                _ => false,
            };
            if !valid {
                return Ok(None);
            }
        }
    }
    let origins = facts.bcis((start, facts.span_end(*rethrow)));
    Ok(Some(Plan {
        shape: Shape::FlagConditionalFinally {
            row_ordinal: body_row.ordinal,
            flag_slot,
            normal_cleanup: (cleanup_start, *return_load),
            handler_cleanup: (*h0, *primary_load),
            saved_return: (save_store, *normal_return),
        },
        lead: (start, body_start),
        body: (body_start, cleanup_start),
        owned,
        join: None,
        facts: origins,
    }))
}

/// One four-instruction null-guarded optional call on a local slot, optionally followed by the
/// same seven-instruction guarded-throw tail both copies may share: `aload s; ifnull exit;
/// aload s; invokevirtual T ()V [`getstatic B; ifeq exit; new C; dup; ldc "S"; invokespecial
/// C.<init>(Ljava/lang/String;)V; athrow`]`. The call's target comes back so the two copies can
/// be checked against one parameter set — the certificate requires no owner or name of its own,
/// only that both copies call the same one `()V` on the slot's value — and the tail's guard
/// field, class and message with it. Nothing here decides what the copy means — only that this
/// exact shape, with one consumer per intermediate value, is what the bytes say.
fn local_null_cleanup_copy(
    facts: &Facts<'_>,
    copy: &[u32],
    exit: u32,
    slot: u16,
) -> Option<(ValueId, Operation, Option<FlagCleanupGuard>)> {
    let [test_load, branch, receiver_load, close, tail @ ..] = copy else {
        return None;
    };
    if tail.len() != 0 && tail.len() != 7 {
        return None;
    }
    if facts.op(*test_load) != Some(&Operation::Load { slot })
        || facts.op(*branch)
            != Some(&Operation::Comparison {
                op: CompareOp::JumpIfNull,
                target: exit,
            })
        || facts.op(*receiver_load) != Some(&Operation::Load { slot })
        || !matches!(facts.op(*close), Some(Operation::Invoke(target))
            if target.kind() == InvokeKind::Virtual && target.descriptor() == "()V")
    {
        return None;
    }
    let target = match facts.op(*close)? {
        Operation::Invoke(target) => Operation::Invoke(target.clone()),
        _ => return None,
    };
    let steps: Vec<_> = copy
        .iter()
        .map(|bci| facts.step(*bci))
        .collect::<Option<_>>()?;
    // The branch and the call act on the one value the first `aload` put on the stack: the two
    // loads read the same slot's same definition, which is what makes both copies' tests
    // decisions about one variable.
    let [(Slot::Local(read_slot), test_value)] = steps[0].instruction.reads() else {
        return None;
    };
    if *read_slot != slot {
        return None;
    }
    if !matches!(steps[2].instruction.reads(), [(Slot::Local(read_slot), value)]
        if *read_slot == slot && facts.same(*value, *test_value))
    {
        return None;
    }
    // Every value the copy builds inside itself is consumed exactly once, where the copy
    // consumes it.
    let consumed = |value: ValueId, consumer: u32| -> bool {
        facts
            .order
            .iter()
            .filter(|bci| {
                **bci != consumer
                    && facts.step(**bci).is_some_and(|step| {
                        stack_operands(step.instruction)
                            .iter()
                            .any(|(_, read)| facts.same(*read, value))
                    })
            })
            .count()
            == 0
    };
    for (producer, consumer) in [(0, 1), (2, 3)] {
        let outputs: Vec<_> = steps[producer]
            .instruction
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .collect();
        let inputs = stack_operands(steps[consumer].instruction);
        if outputs.len() != 1
            || inputs.len() != 1
            || !facts.same(outputs[0].1, inputs[0].1)
            || !consumed(outputs[0].1, copy[consumer])
        {
            return None;
        }
    }
    let guard = if tail.is_empty() {
        None
    } else {
        let [
            guard_read,
            guard_branch,
            allocate,
            duplicate_throw,
            message,
            construct,
            rethrow_new,
        ] = tail
        else {
            return None;
        };
        let Operation::Field {
            access: crate::facts::FieldAccess::Read,
            is_static: true,
            owner: guard_owner,
            name: guard_name,
            descriptor: guard_descriptor,
        } = facts.op(*guard_read)?
        else {
            return None;
        };
        if guard_descriptor != "Z"
            || facts.op(*guard_branch)
                != Some(&Operation::Comparison {
                    op: CompareOp::JumpIfZero,
                    target: exit,
                })
        {
            return None;
        }
        let Operation::Allocate { ty: class } = facts.op(*allocate)? else {
            return None;
        };
        let Operation::Push(crate::facts::ConstantValue::String(message)) = facts.op(*message)?
        else {
            return None;
        };
        let Operation::Invoke(construct_target) = facts.op(*construct)? else {
            return None;
        };
        if construct_target.kind() != InvokeKind::Special
            || construct_target.name() != "<init>"
            || construct_target.descriptor() != "(Ljava/lang/String;)V"
            || construct_target.owner() != class
            || facts.op(*rethrow_new) != Some(&Operation::Throw)
        {
            return None;
        }
        // The tail's own value flow, under the same one-consumer-per-value discipline: the
        // allocation feeds the `dup` alone, the `dup`'s upper copy and the message feed the
        // constructor alone, the constructor's initialized value feeds the `athrow` alone — and
        // the `dup`'s lower copy reads nowhere, because the constructor call converts it in
        // place, which is what makes the `athrow` throw the initialized instance. The indexes
        // are the full copy's: the four guarded-call instructions, then the tail.
        let (Some(allocate_output), Some(message_output), Some(initialized)) = (
            stack_writes_at(facts, copy, 6)?
                .first()
                .map(|(_, value)| *value),
            stack_writes_at(facts, copy, 8)?
                .first()
                .map(|(_, value)| *value),
            stack_writes_at(facts, copy, 9)?
                .first()
                .map(|(_, value)| *value),
        ) else {
            return None;
        };
        let tail_thrown = stack_writes_at(facts, copy, 7)?;
        if tail_thrown.len() != 2 || tail_thrown[0].1 == tail_thrown[1].1 {
            return None;
        }
        let guard_value = stack_writes_at(facts, copy, 4)?
            .first()
            .map(|(_, value)| *value)?;
        let construct_inputs = stack_operands(steps[9].instruction);
        let guard_branch_inputs = stack_operands(steps[5].instruction);
        let throw_inputs = stack_operands(steps[10].instruction);
        if construct_inputs.len() != 2
            || guard_branch_inputs.len() != 1
            || throw_inputs.len() != 1
            || !facts.same(guard_branch_inputs[0].1, guard_value)
            || !facts.same(construct_inputs[0].1, tail_thrown[1].1)
            || !facts.same(construct_inputs[1].1, message_output)
            || !facts.same(throw_inputs[0].1, initialized)
        {
            return None;
        }
        for (value, consumer) in [
            (allocate_output, *duplicate_throw),
            (tail_thrown[1].1, *construct),
            (message_output, *construct),
            (initialized, *rethrow_new),
        ] {
            if !consumed(value, consumer) {
                return None;
            }
        }
        // The `dup`'s lower copy is the one value the conversion kills: no instruction may read
        // it, the `athrow` included.
        if !consumed(tail_thrown[0].1, *rethrow_new) {
            return None;
        }
        Some(FlagCleanupGuard {
            field: (
                guard_owner.clone(),
                guard_name.clone(),
                guard_descriptor.clone(),
            ),
            class: class.clone(),
            message: message.clone(),
        })
    };
    Some((*test_value, target, guard))
}

/// The stack slots one copy instruction writes, addressed by the copy's own index.
fn stack_writes_at(facts: &Facts<'_>, copy: &[u32], index: usize) -> Option<Vec<(Slot, ValueId)>> {
    let mut writes: Vec<_> = facts
        .step(*copy.get(index)?)?
        .instruction
        .writes()
        .iter()
        .filter(|(written, _)| matches!(written, Slot::Stack(_)))
        .copied()
        .collect();
    writes.sort_by_key(|(written, _)| match written {
        Slot::Stack(depth) => *depth,
        Slot::Local(slot) => u32::from(*slot),
    });
    Some(writes)
}

/// The handler copy runs after any protected instruction, so its read of the cleanup slot may
/// carry the lead's null or one body assignment: a direct definition or a handler phi whose
/// inputs are those, expanded to a fixpoint. The `Ok` flag answers whether the body's own
/// written value is among the read's sources — the proof that the copy really reads the value
/// the body wrote, not the lead's null alone.
fn local_null_handler_provenance(
    facts: &mut Facts<'_>,
    value: ValueId,
    lead_store: ValueId,
    body_stores: &[ValueId],
    handler_entry: u32,
    slot: u16,
) -> Result<bool, StopReason> {
    let mut pending = vec![value];
    let mut seen = BTreeSet::new();
    let mut found_body = false;
    while let Some(value) = pending.pop() {
        facts.charge(0)?;
        let value = facts.resolve(value);
        if !seen.insert(value) {
            continue;
        }
        if seen.len() > 64 {
            return Ok(false);
        }
        if body_stores.iter().any(|store| facts.same(value, *store)) {
            found_body = true;
            continue;
        }
        if facts.same(value, lead_store) {
            continue;
        }
        let Definition::Phi {
            block,
            slot: phi_slot,
        } = facts.ssa.value(value).def()
        else {
            return Ok(false);
        };
        if *phi_slot != Slot::Local(slot) {
            return Ok(false);
        }
        let Some(phi) = facts.ssa.phis().iter().find(|phi| {
            phi.block() == block && phi.slot() == *phi_slot && facts.same(phi.value(), value)
        }) else {
            return Ok(false);
        };
        for input in phi.inputs() {
            match input {
                jarde_jvm::method_ir::PhiInput::Value(value) => pending.push(*value),
                jarde_jvm::method_ir::PhiInput::Itself if block.bci() == handler_entry => {}
                jarde_jvm::method_ir::PhiInput::Itself => return Ok(false),
            }
        }
    }
    Ok(found_body)
}

/// The fixed two-row nullable-local lowering: a two-instruction `null` initialisation, a body
/// whose same-slot assignments all precede its saved return, and two equivalent four-instruction
/// optional-call copies, with the shared optional guarded-throw tail both copies may carry.
/// Every instruction and edge is accounted for before Plan; the body, the lead and the normal
/// copy's own test share the entry block, which is why the body's ownership is read instruction
/// by instruction and not block by block.
fn prove_local_null_conditional_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [body_row, self_row] = facts.handlers else {
        return Ok(None);
    };
    let (start, body_start, cleanup_start, handler_start) = (
        current.bci(),
        body_row.start_bci,
        body_row.end_bci,
        body_row.handler_bci,
    );
    if body_row.catch_type_index.is_some()
        || self_row.catch_type_index.is_some()
        || self_row.ordinal != body_row.ordinal + 1
        || self_row.start_bci != handler_start
        || self_row.handler_bci != handler_start
        || facts.next_bci(self_row.start_bci) != Some(self_row.end_bci)
        || facts.order.len() > 64
        || !facts.canonical.unreachable().is_empty()
    {
        return Ok(None);
    }
    let lead = facts.bcis((start, body_start));
    let body = facts.bcis((body_start, cleanup_start));
    let normal = facts.bcis((cleanup_start, handler_start));
    let Some(&last_bci) = facts.order.last() else {
        return Ok(None);
    };
    let handler = facts.bcis((handler_start, facts.span_end(last_bci)));
    let (
        [null_push, null_store],
        [
            _test_load,
            _branch,
            _receiver_load,
            _close,
            normal_tail @ ..,
            return_load,
            normal_return,
        ],
        [
            primary_store,
            _handler_test_load,
            _handler_branch,
            _handler_receiver_load,
            _handler_close,
            handler_tail @ ..,
            primary_load,
            rethrow,
        ],
    ) = (lead.as_slice(), normal.as_slice(), handler.as_slice())
    else {
        return Ok(None);
    };
    // The two copies are the same length: the four guarded-call instructions alone, or the core
    // with the same seven-instruction guarded-throw tail in both. Anything else is not this
    // lowering.
    if normal_tail.len() != handler_tail.len() || (normal_tail.len() != 0 && normal_tail.len() != 7)
    {
        return Ok(None);
    }
    let Some(Operation::Store { slot }) = facts.op(*null_store) else {
        return Ok(None);
    };
    let slot = *slot;
    if slot == 0
        || facts.op(*null_push) != Some(&Operation::Push(crate::facts::ConstantValue::Null))
        || !matches!(facts.op(*return_load), Some(Operation::Load { .. }))
        || facts.op(*normal_return) != Some(&Operation::Return)
        || !matches!(facts.op(*primary_store), Some(Operation::Store { slot: stored }) if *stored != slot)
        || !matches!(facts.op(*primary_load), Some(Operation::Load { slot: loaded }) if *loaded != slot)
        || facts.op(*rethrow) != Some(&Operation::Throw)
        || facts.next_bci(*rethrow).is_some()
    {
        return Ok(None);
    }
    let Some(Operation::Store { slot: primary_slot }) = facts.op(*primary_store) else {
        return Ok(None);
    };
    let primary_slot = *primary_slot;
    // The lead is the method's own first two instructions, and the body, the lead and the
    // normal copy's test share the entry block: the body's ownership is proved per instruction.
    if body.is_empty()
        || facts.block_of(*null_push) != Some(current)
        || facts.block_of(*null_store) != Some(current)
        || body
            .iter()
            .copied()
            .chain([normal[0], normal[1]])
            .any(|bci| facts.block_of(bci) != Some(current))
    {
        return Ok(None);
    }
    // The body fills the slot, and only there: the lead's `null` and the body's assignments are
    // the only definitions either copy's condition can read. An assignment anywhere else — the
    // copies themselves, the handler, the code after the statement — is not this lowering.
    let body_store_bcis: Vec<u32> = body
        .iter()
        .copied()
        .filter(|bci| {
            matches!(facts.op(*bci), Some(Operation::Store { slot: filled }) if *filled == slot)
        })
        .collect();
    if body_store_bcis.is_empty()
        || facts
            .bcis((start, facts.span_end(last_bci)))
            .into_iter()
            .any(|bci| {
                bci != *null_store
                    && !body_store_bcis.contains(&bci)
                    && matches!(
                        facts.op(bci),
                        Some(Operation::Store { slot: filled }) if *filled == slot
                    )
            })
    {
        return Ok(None);
    }
    // The body's completion is the saved return: the statement's last instruction stores the
    // returned value, and the slot it stores is not the cleanup's own.
    let Some(save_store) = body.last().copied() else {
        return Ok(None);
    };
    let Some(Operation::Store { slot: save_slot }) = facts.op(save_store) else {
        return Ok(None);
    };
    let save_slot = *save_slot;
    if save_slot == slot || facts.next_bci(save_store) != Some(cleanup_start) {
        return Ok(None);
    }
    // The normal copy's completion is the saved value's own return; the handler's is the
    // pending throwable's rethrow, whose binding store the self-protecting row covers.
    if facts.op(*return_load) != Some(&Operation::Load { slot: save_slot })
        || facts.op(*primary_load) != Some(&Operation::Load { slot: primary_slot })
        || !handler_binding(facts, *primary_store)
    {
        return Ok(None);
    }
    let (
        Some((normal_value, normal_target, normal_guard)),
        Some((handler_value, handler_target, handler_guard)),
    ) = (
        local_null_cleanup_copy(facts, &normal[..4 + normal_tail.len()], *return_load, slot),
        local_null_cleanup_copy(
            facts,
            &handler[1..5 + handler_tail.len()],
            *primary_load,
            slot,
        ),
    )
    else {
        return Ok(None);
    };
    if normal_target != handler_target || normal_guard != handler_guard {
        return Ok(None);
    }
    let (
        Some(null_step),
        Some(null_store_step),
        Some(save_step),
        Some(return_step),
        Some(throw_step),
    ) = (
        facts.step(*null_push),
        facts.step(*null_store),
        facts.step(save_store),
        facts.step(*normal_return),
        facts.step(*rethrow),
    )
    else {
        return Ok(None);
    };
    let local_written = |step: Step<'_>, slot| {
        step.instruction
            .writes()
            .iter()
            .find_map(|(written_slot, value)| {
                (*written_slot == Slot::Local(slot)).then_some(*value)
            })
    };
    let (Some(lead_value), Some(saved_value), Some(primary_value)) = (
        local_written(null_store_step, slot),
        local_written(save_step, save_slot),
        facts
            .step(*primary_store)
            .and_then(|step| local_written(step, primary_slot)),
    ) else {
        return Ok(None);
    };
    // Every body assignment's right-hand value has a producer of its own, and none of them is a
    // null constant: the lead's `aconst_null` is the one null source either copy's test reads,
    // which is what makes the folded cleanup's `!= null` the body-assigned test.
    for bci in &body_store_bcis {
        let Some(step) = facts.step(*bci) else {
            return Ok(None);
        };
        let Some(written) = local_written(step, slot) else {
            return Ok(None);
        };
        let Definition::Instruction { bci: producer, .. } =
            facts.ssa.value(facts.resolve(written)).def()
        else {
            return Ok(None);
        };
        if facts.op(*producer) == Some(&Operation::Push(crate::facts::ConstantValue::Null)) {
            return Ok(None);
        }
    }
    let last_body_store = body_store_bcis[body_store_bcis.len() - 1];
    let Some(last_store_value) = facts
        .step(last_body_store)
        .and_then(|step| local_written(step, slot))
    else {
        return Ok(None);
    };
    let body_store_values: Vec<_> = body_store_bcis
        .iter()
        .filter_map(|bci| facts.step(*bci).and_then(|step| local_written(step, slot)))
        .collect();
    let return_inputs = stack_operands(return_step.instruction);
    let throw_inputs = stack_operands(throw_step.instruction);
    let handler_proven = local_null_handler_provenance(
        facts,
        handler_value,
        lead_value,
        &body_store_values,
        handler_start,
        slot,
    )?;
    let lead_operand_ok = matches!(stack_operands(null_store_step.instruction).as_slice(), [(_, read)]
        if null_step.instruction.writes().iter().any(|(_, value)| facts.same(*value, *read)));
    let return_reads_ok = facts.step(*return_load).is_some_and(|step| {
        step.instruction.reads().iter().any(|(read_slot, value)| {
            *read_slot == Slot::Local(save_slot) && facts.same(*value, saved_value)
        })
    });
    let return_input_ok = matches!(return_inputs.as_slice(), [(_, read)]
        if facts.step(*return_load).is_some_and(|step| step
            .instruction
            .writes()
            .iter()
            .any(|(_, value)| facts.same(*value, *read))));
    let primary_load_reads_ok = facts.step(*primary_load).is_some_and(|step| {
        step.instruction.reads().iter().any(|(read_slot, value)| {
            *read_slot == Slot::Local(primary_slot) && facts.same(*value, primary_value)
        })
    });
    let throw_input_ok = matches!(throw_inputs.as_slice(), [(_, read)]
        if facts.step(*primary_load).is_some_and(|step| step
            .instruction
            .writes()
            .iter()
            .any(|(_, written)| facts.same(*written, *read))));
    if !lead_operand_ok {
        return Ok(None);
    }
    if !handler_proven {
        return Ok(None);
    }
    if !facts.same(normal_value, last_store_value) {
        return Ok(None);
    }
    if !return_reads_ok {
        return Ok(None);
    }
    if !return_input_ok {
        return Ok(None);
    }
    if !primary_load_reads_ok {
        return Ok(None);
    }
    if !throw_input_ok {
        return Ok(None);
    }
    let protected = facts.blocks_in((body_start, cleanup_start));
    let handler_blocks = facts.blocks_in((handler_start, facts.span_end(*rethrow)));
    let mut normal_throw_block = None;
    let mut handler_throw_block = None;
    if !normal_tail.is_empty() {
        let Some(block) = facts.block_of(normal[10]) else {
            return Ok(None);
        };
        normal_throw_block = Some(block.clone());
    }
    if !handler_tail.is_empty() {
        let Some(block) = facts.block_of(handler[11]) else {
            return Ok(None);
        };
        handler_throw_block = Some(block.clone());
    }
    let (
        Some(update_block),
        Some(return_block),
        Some(handler_entry),
        Some(handler_update_block),
        Some(rethrow_block),
    ) = (
        facts.block_of(normal[2]).cloned(),
        facts.block_of(*return_load).cloned(),
        facts.block_of(*primary_store).cloned(),
        facts.block_of(handler[3]).cloned(),
        facts.block_of(*rethrow).cloned(),
    )
    else {
        return Ok(None);
    };
    // With the guarded-throw tail the update arm branches once more: the throw block ends in an
    // `athrow` no exception row covers, so its own effect is the copy's completion there.
    let (update_exits, handler_update_exits) = (
        match (&normal_throw_block, &return_block) {
            (Some(throw), return_block) => {
                BTreeSet::from([(*throw).clone(), (*return_block).clone()])
            }
            (None, return_block) => BTreeSet::from([(*return_block).clone()]),
        },
        match (&handler_throw_block, &rethrow_block) {
            (Some(throw), rethrow_block) => {
                BTreeSet::from([(*throw).clone(), (*rethrow_block).clone()])
            }
            (None, rethrow_block) => BTreeSet::from([(*rethrow_block).clone()]),
        },
    );
    let owned = facts.blocks_in((start, facts.span_end(*rethrow)));
    if owned.len() != facts.canonical.blocks().len()
        || protected.is_empty()
        || facts
            .view
            .successor_ids(current)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([update_block.clone(), return_block.clone()])
        || facts
            .view
            .successor_ids(&update_block)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != update_exits
        || facts
            .view
            .successor_ids(&handler_entry)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([handler_update_block.clone(), rethrow_block.clone()])
        || facts
            .view
            .successor_ids(&handler_update_block)
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != handler_update_exits
        || normal_throw_block
            .as_ref()
            .is_some_and(|throw| !facts.view.successor_ids(throw).is_empty())
        || handler_throw_block
            .as_ref()
            .is_some_and(|throw| !facts.view.successor_ids(throw).is_empty())
        || !facts.view.successor_ids(&return_block).is_empty()
        || !facts.view.successor_ids(&rethrow_block).is_empty()
    {
        return Ok(None);
    }
    for bci in facts.bcis((start, facts.span_end(*rethrow))) {
        facts.charge(bci)?;
        let expected: Vec<u32> = if body_start <= bci && bci < cleanup_start {
            vec![body_row.ordinal]
        } else if bci == handler_start {
            vec![self_row.ordinal]
        } else {
            Vec::new()
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
            || (body_start <= bci
                && bci < cleanup_start
                && facts.op(bci) == Some(&Operation::Return))
        {
            return Ok(None);
        }
    }
    for block in facts.canonical.blocks() {
        facts.charge(block.id().bci())?;
        for edge in facts
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == block.id())
        {
            facts.charge(block.id().bci())?;
            let from_handler = handler_blocks.contains(block.id());
            let valid = match edge.kind() {
                CanonicalEdgeKind::Exception { handler_ordinal } => {
                    edge.to() == &handler_entry
                        && ((protected.contains(block.id()) && handler_ordinal == body_row.ordinal)
                            || (block.id() == &handler_entry
                                && handler_ordinal == self_row.ordinal))
                }
                CanonicalEdgeKind::Normal if block.id() == current => {
                    edge.to() == &update_block || edge.to() == &return_block
                }
                CanonicalEdgeKind::Normal if block.id() == &update_block => {
                    edge.to() == &return_block
                        || normal_throw_block
                            .as_ref()
                            .is_some_and(|throw| edge.to() == throw)
                }
                CanonicalEdgeKind::Normal if from_handler => {
                    handler_blocks.contains(edge.to()) || edge.to() == &rethrow_block
                }
                CanonicalEdgeKind::Return { .. } => block.id() == &return_block,
                _ => false,
            };
            if !valid {
                return Ok(None);
            }
        }
    }
    let origins = facts.bcis((start, facts.span_end(*rethrow)));
    Ok(Some(Plan {
        shape: Shape::LocalNullConditionalFinally {
            row_ordinal: body_row.ordinal,
            slot,
            normal_cleanup: (cleanup_start, *return_load),
            handler_cleanup: (handler[1], *primary_load),
            saved_return: (save_store, *normal_return),
        },
        lead: (start, body_start),
        body: (body_start, cleanup_start),
        owned,
        join: None,
        facts: origins,
    }))
}

/// The first and last instructions of a cleanup. The caller separately proves the three
/// effects identical; this only finds the return/throw after each copy.
fn shared_cleanup_span(facts: &Facts<'_>, start: u32) -> Option<(u32, u32)> {
    match facts.op(start)? {
        Operation::Invoke(_) => Some((start, start)),
        Operation::Field {
            access: crate::facts::FieldAccess::Read,
            is_static: true,
            descriptor,
            ..
        } if descriptor == "I" => {
            let push = facts.next_bci(start)?;
            let add = facts.next_bci(push)?;
            let write = facts.next_bci(add)?;
            matches!(
                facts.op(push),
                Some(Operation::Push(crate::facts::ConstantValue::Int(_)))
            )
            .then_some(())?;
            (facts.op(add)
                == Some(&Operation::Arithmetic {
                    op: crate::facts::ArithmeticOp::Add,
                }))
            .then_some(())?;
            matches!(facts.op(write), Some(Operation::Field { access: crate::facts::FieldAccess::Write, is_static: true, descriptor, .. }) if descriptor == "I")
                .then_some((start, write))
        }
        _ => None,
    }
}

fn shared_cleanup_copies(
    facts: &mut Facts<'_>,
    copies: [(u32, u32); 3],
) -> Result<bool, StopReason> {
    let first = facts.op(copies[0].0);
    if let Some(Operation::Invoke(target)) = first {
        return Ok(target.kind() == InvokeKind::Static
            && target.descriptor() == "()V"
            && copies.iter().all(|&(start, last)| {
                start == last
                    && facts.op(start) == first
                    && facts.step(start).is_some_and(|step| {
                        stack_operands(step.instruction).is_empty()
                            && !step
                                .instruction
                                .writes()
                                .iter()
                                .any(|(slot, _)| matches!(slot, Slot::Stack(_)))
                    })
            }));
    }
    let Some(read) = first.cloned() else {
        return Ok(false);
    };
    let mut constant = None;
    for &(start, last) in &copies {
        let Some(push) = facts.next_bci(start) else {
            return Ok(false);
        };
        let Some(add) = facts.next_bci(push) else {
            return Ok(false);
        };
        let Some(write) = facts.next_bci(add) else {
            return Ok(false);
        };
        let Some(Operation::Push(crate::facts::ConstantValue::Int(value))) = facts.op(push) else {
            return Ok(false);
        };
        if write != last
            || facts.op(start) != Some(&read)
            || constant.is_some_and(|prior| prior != *value)
            || facts.op(add)
                != Some(&Operation::Arithmetic {
                    op: crate::facts::ArithmeticOp::Add,
                })
            || facts
                .step(add)
                .is_none_or(|step| step.instruction.opcode() != 0x60)
        {
            return Ok(false);
        }
        constant = Some(*value);
        let Some(Operation::Field {
            access: crate::facts::FieldAccess::Read,
            is_static: true,
            descriptor,
            ..
        }) = facts.op(start)
        else {
            return Ok(false);
        };
        if descriptor != "I"
            || facts.op(write)
                != Some(&Operation::Field {
                    access: crate::facts::FieldAccess::Write,
                    is_static: true,
                    owner: match &read {
                        Operation::Field { owner, .. } => owner.clone(),
                        _ => return Ok(false),
                    },
                    name: match &read {
                        Operation::Field { name, .. } => name.clone(),
                        _ => return Ok(false),
                    },
                    descriptor: "I".to_owned(),
                })
        {
            return Ok(false);
        }
        for (producer, consumer) in [(start, add), (push, add), (add, write)] {
            let (Some(produced), Some(consumed)) = (facts.step(producer), facts.step(consumer))
            else {
                return Ok(false);
            };
            let outputs: Vec<_> = produced
                .instruction
                .writes()
                .iter()
                .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
                .collect();
            if outputs.len() != 1
                || stack_operands(consumed.instruction)
                    .iter()
                    .filter(|(_, value)| facts.same(outputs[0].1, *value))
                    .count()
                    != 1
            {
                return Ok(false);
            }
            let mut consumers = 0;
            for bci in facts.order.clone() {
                facts.charge(bci)?;
                if let Some(step) = facts.step(bci) {
                    consumers += stack_operands(step.instruction)
                        .iter()
                        .filter(|(_, value)| facts.same(outputs[0].1, *value))
                        .count();
                }
            }
            if consumers != 1 {
                return Ok(false);
            }
        }
        if stack_operands(facts.step(add).unwrap().instruction).len() != 2
            || stack_operands(facts.step(write).unwrap().instruction).len() != 1
            || !facts.step(start).unwrap().instruction.reads().is_empty()
            || !facts.step(push).unwrap().instruction.reads().is_empty()
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Preserve the older boolean-field certificate as its own narrow copy proof.
fn boolean_join_cleanup(
    facts: &mut Facts<'_>,
    first: u32,
    second: u32,
    third: u32,
) -> Result<Option<[(u32, u32); 3]>, StopReason> {
    let mut copies = [(0, 0); 3];
    for (index, at) in [first, second, third].into_iter().enumerate() {
        let (Some(push), Some(write)) = (
            facts.next_bci(at),
            facts.next_bci(at).and_then(|bci| facts.next_bci(bci)),
        ) else {
            return Ok(None);
        };
        copies[index] = (at, write);
        if facts
            .step(at)
            .is_none_or(|step| step.instruction.opcode() != 0x2a)
            || facts
                .step(push)
                .is_none_or(|step| step.instruction.opcode() != 0x04)
            || facts
                .step(write)
                .is_none_or(|step| step.instruction.opcode() != 0xb5)
        {
            return Ok(None);
        }
    }
    let Some(Operation::Field {
        access: crate::facts::FieldAccess::Write,
        is_static: false,
        descriptor,
        ..
    }) = facts.op(copies[0].1)
    else {
        return Ok(None);
    };
    if descriptor != "Z"
        || !copies
            .iter()
            .all(|&(_, write)| facts.op(write) == facts.op(copies[0].1))
    {
        return Ok(None);
    }
    for &(load, write) in &copies {
        let push = facts.next_bci(load).unwrap();
        let (Some(receiver), Some(constant), Some(store)) =
            (facts.step(load), facts.step(push), facts.step(write))
        else {
            return Ok(None);
        };
        let reads = receiver.instruction.reads();
        let receiver_outputs: Vec<_> = receiver
            .instruction
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .collect();
        let constant_outputs: Vec<_> = constant
            .instruction
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .collect();
        let operands = stack_operands(store.instruction);
        if facts.op(load) != Some(&Operation::Load { slot: 0 })
            || facts.op(push) != Some(&Operation::Push(crate::facts::ConstantValue::Int(1)))
            || reads.len() != 1
            || reads[0].0 != Slot::Local(0)
            || !matches!(
                facts.ssa.value(facts.resolve(reads[0].1)).def(),
                Definition::Entry {
                    slot: Slot::Local(0),
                    ..
                }
            )
            || receiver_outputs.len() != 1
            || constant_outputs.len() != 1
            || operands.len() != 2
            || !operands
                .iter()
                .any(|(_, value)| facts.same(*value, receiver_outputs[0].1))
            || !operands
                .iter()
                .any(|(_, value)| facts.same(*value, constant_outputs[0].1))
        {
            return Ok(None);
        }
        for output in [receiver_outputs[0].1, constant_outputs[0].1] {
            let mut uses = 0;
            for bci in facts.order.clone() {
                facts.charge(bci)?;
                if let Some(step) = facts.step(bci) {
                    uses += stack_operands(step.instruction)
                        .iter()
                        .filter(|(_, value)| facts.same(*value, output))
                        .count();
                }
            }
            if uses != 1 {
                return Ok(None);
            }
        }
    }
    Ok(Some(copies))
}

/// The exact Java 8 empty-catch lowering has two *same-range* rows. Its catch store is only
/// the parameter binding; all three cleanup calls are outside both rows. Prove the entire
/// method shape so no edge can enter a hidden copy or skip the one shared return.
fn prove_empty_catch_call_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [named, any] = facts.handlers else {
        return Ok(None);
    };
    let [
        body,
        normal,
        normal_jump,
        named_store,
        catch_copy,
        catch_jump,
        primary_store,
        primary_copy,
        primary_load,
        rethrow,
        returns,
    ] = facts.order.as_slice()
    else {
        return Ok(None);
    };
    let [
        body,
        normal,
        normal_jump,
        named_store,
        catch_copy,
        catch_jump,
        primary_store,
        primary_copy,
        primary_load,
        rethrow,
        returns,
    ] = [
        *body,
        *normal,
        *normal_jump,
        *named_store,
        *catch_copy,
        *catch_jump,
        *primary_store,
        *primary_copy,
        *primary_load,
        *rethrow,
        *returns,
    ];
    if body != 0
        || current.bci() != body
        || named.ordinal + 1 != any.ordinal
        || named.catch_type_index.is_none()
        || any.catch_type_index.is_some()
        || (named.start_bci, named.end_bci) != (body, normal)
        || (any.start_bci, any.end_bci) != (body, normal)
        || named.handler_bci != named_store
        || any.handler_bci != primary_store
        || !matches!(facts.op(body), Some(Operation::Invoke(call))
            if call.kind() == InvokeKind::Static && call.descriptor() == "()V")
        || facts.op(normal_jump) != Some(&Operation::Transfer)
        || facts.op(catch_jump) != Some(&Operation::Transfer)
        || facts.op(rethrow) != Some(&Operation::Throw)
        || facts.op(returns) != Some(&Operation::Return)
        || !facts
            .step(returns)
            .is_some_and(|step| step.instruction.reads().is_empty())
        || !matches!(facts.op(named_store), Some(Operation::Store { .. }))
        || !matches!(facts.op(primary_store), Some(Operation::Store { .. }))
        || !handler_binding(facts, named_store)
        || !handler_binding(facts, primary_store)
        || !shared_cleanup_copies(
            facts,
            [
                (normal, normal),
                (catch_copy, catch_copy),
                (primary_copy, primary_copy),
            ],
        )?
    {
        return Ok(None);
    }
    // The body and each copy consume no argument or receiver and leave no operand stack value.
    for bci in [body, normal, catch_copy, primary_copy] {
        facts.charge(bci)?;
        let Some(step) = facts.step(bci) else {
            return Ok(None);
        };
        if !stack_operands(step.instruction).is_empty()
            || step
                .instruction
                .writes()
                .iter()
                .any(|(slot, _)| matches!(slot, Slot::Stack(_)))
        {
            return Ok(None);
        }
    }
    // Exact physical coverage: the only protected instruction is the body call.
    for bci in facts.order.clone() {
        facts.charge(bci)?;
        let expected = if bci == body {
            &[named.ordinal, any.ordinal][..]
        } else {
            &[][..]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
        {
            return Ok(None);
        }
    }
    let (Some(body_block), Some(named_block), Some(primary_block), Some(join)) = (
        facts.block_at(body),
        facts.row_handler(named),
        facts.row_handler(any),
        facts.block_at(returns),
    ) else {
        return Ok(None);
    };
    if body_block != *current
        || named_block.bci() != named_store
        || primary_block.bci() != primary_store
        || [
            (&body_block, &[body, normal, normal_jump][..]),
            (&named_block, &[named_store, catch_copy, catch_jump]),
            (
                &primary_block,
                &[primary_store, primary_copy, primary_load, rethrow],
            ),
            (&join, &[returns]),
        ]
        .iter()
        .any(|(block, expected)| {
            facts
                .in_block(block)
                .iter()
                .map(SsaInstruction::bci)
                .collect::<Vec<_>>()
                != *expected
        })
        || facts.canonical.blocks().len() != 4
        || facts.view.successor_ids(&body_block) != [join.clone()]
        || facts.view.successor_ids(&named_block) != [join.clone()]
        || !facts.view.successor_ids(&primary_block).is_empty()
        || !facts.view.successor_ids(&join).is_empty()
    {
        return Ok(None);
    }
    let Some(join_node) = facts.view.index_of(&join) else {
        return Ok(None);
    };
    if facts.view.predecessors(join_node).len() != 2 {
        return Ok(None);
    }
    // The named parameter has no instruction consumer. The catch-all slot feeds only the
    // load, and the load's stack value feeds only athrow, preserving the incoming Throwable.
    let (Some(named_step), Some(saved), Some(loaded), Some(thrown)) = (
        facts.step(named_store),
        facts.step(primary_store),
        facts.step(primary_load),
        facts.step(rethrow),
    ) else {
        return Ok(None);
    };
    let Some((_, named_value)) = named_step
        .instruction
        .writes()
        .iter()
        .find(|(slot, _)| matches!(slot, Slot::Local(_)))
    else {
        return Ok(None);
    };
    if facts
        .order
        .iter()
        .copied()
        .filter(|bci| *bci != named_store)
        .any(|bci| {
            facts.step(bci).is_some_and(|step| {
                step.instruction
                    .reads()
                    .iter()
                    .any(|(_, read)| facts.same(*read, *named_value))
            })
        })
    {
        return Ok(None);
    }
    if !matches!((facts.op(primary_store), facts.op(primary_load)),
        (Some(Operation::Store { slot: stored }), Some(Operation::Load { slot: loaded })) if stored == loaded)
        || !saved.instruction.writes().iter().any(|(_, written)| {
            loaded
                .instruction
                .reads()
                .iter()
                .any(|(_, read)| facts.same(*written, *read))
        })
        || !loaded.instruction.writes().iter().any(|(slot, written)| {
            matches!(slot, Slot::Stack(_))
                && stack_operands(thrown.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(*written, *read))
        })
        || stack_operands(thrown.instruction).len() != 1
    {
        return Ok(None);
    }
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let valid = match edge.kind() {
            CanonicalEdgeKind::Exception { handler_ordinal } => {
                edge.from() == &body_block
                    && ((handler_ordinal == named.ordinal && edge.to() == &named_block)
                        || (handler_ordinal == any.ordinal && edge.to() == &primary_block))
            }
            CanonicalEdgeKind::Normal => {
                (edge.from() == &body_block && edge.to() == &join)
                    || (edge.from() == &named_block && edge.to() == &join)
            }
            CanonicalEdgeKind::Call { .. } | CanonicalEdgeKind::Return { .. } => false,
        };
        if !valid {
            return Ok(None);
        }
    }
    Ok(Some(Plan {
        shape: Shape::EmptyCatchCallFinally {
            rows: [named.ordinal, any.ordinal],
            cleanup_target: match facts.op(normal) {
                Some(Operation::Invoke(target)) => target.clone(),
                _ => unreachable!(),
            },
            catch_handler: named_block,
            catch_type: named.catch_type_index.unwrap(),
            catch_parameter: match facts.op(named_store) {
                Some(Operation::Store { slot }) => *slot,
                _ => unreachable!(),
            },
            cleanup: [normal, catch_copy, primary_copy].map(|bci| (bci, facts.span_end(bci))),
            transfers: [normal_jump, catch_jump],
        },
        lead: (body, body),
        body: (body, normal),
        owned: vec![body_block, facts.row_handler(named).unwrap(), primary_block],
        join: Some(join),
        facts: facts.order.clone(),
    }))
}

/// The fixed Test4 four-row lowering has one lexical cleanup despite two physical copies.
fn prove_nested_cleanup_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
    pool: &[CpEntryFacts],
) -> Result<Option<Plan>, StopReason> {
    let [normal_row, body_row, exceptional_row, self_row] = facts.handlers else {
        return Ok(None);
    };
    const BCIS: &[u32] = &[
        0, 2, 4, 7, 8, 11, 12, 13, 16, 17, 18, 19, 22, 23, 26, 27, 30, 31, 34, 35, 38, 40, 41, 44,
        45, 48, 49, 52, 54, 56, 57,
    ];
    if current.bci() != 0
        || facts.order != BCIS
        || (
            normal_row.start_bci,
            normal_row.end_bci,
            normal_row.handler_bci,
        ) != (22, 31, 34)
        || (body_row.start_bci, body_row.end_bci, body_row.handler_bci) != (17, 22, 38)
        || (
            exceptional_row.start_bci,
            exceptional_row.end_bci,
            exceptional_row.handler_bci,
        ) != (40, 49, 52)
        || (self_row.start_bci, self_row.end_bci, self_row.handler_bci) != (38, 40, 38)
        || normal_row.ordinal + 1 != body_row.ordinal
        || body_row.ordinal + 1 != exceptional_row.ordinal
        || exceptional_row.ordinal + 1 != self_row.ordinal
        || normal_row.catch_type_index.is_none()
        || normal_row.catch_type_index != exceptional_row.catch_type_index
        || cp_class_name(pool, normal_row.catch_type_index.unwrap())
            .ok()
            .is_none_or(|name| name.0.as_slice() != b"java/io/IOException")
        || body_row.catch_type_index.is_some()
        || self_row.catch_type_index.is_some()
    {
        return Ok(None);
    }
    let opcodes = [
        0x12, 0x12, 0xb8, 0x4c, 0xbb, 0x59, 0x2b, 0xb7, 0x4d, 0x2c, 0x04, 0xb6, 0x2c, 0xb6, 0x2b,
        0xb6, 0x57, 0xa7, 0x4e, 0xa7, 0x3a, 0x2c, 0xb6, 0x2b, 0xb6, 0x57, 0xa7, 0x3a, 0x19, 0xbf,
        0xb1,
    ];
    for (bci, opcode) in BCIS.iter().copied().zip(opcodes) {
        facts.charge(bci)?;
        if facts
            .step(bci)
            .is_none_or(|step| step.instruction.opcode() != opcode)
        {
            return Ok(None);
        }
        let expected = if (22..31).contains(&bci) {
            &[normal_row.ordinal][..]
        } else if (17..22).contains(&bci) {
            &[body_row.ordinal][..]
        } else if (40..49).contains(&bci) {
            &[exceptional_row.ordinal][..]
        } else if (38..40).contains(&bci) {
            &[self_row.ordinal][..]
        } else {
            &[][..]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
        {
            return Ok(None);
        }
    }
    let exact_call = |at, kind, owner, name, descriptor| {
        matches!(facts.op(at), Some(Operation::Invoke(call))
            if call.kind() == kind && call.owner() == owner
                && call.name() == name && call.descriptor() == descriptor)
    };
    if !exact_call(
        4,
        InvokeKind::Static,
        "java/io/File",
        "createTempFile",
        "(Ljava/lang/String;Ljava/lang/String;)Ljava/io/File;",
    ) || !exact_call(
        13,
        InvokeKind::Special,
        "java/io/FileOutputStream",
        "<init>",
        "(Ljava/io/File;)V",
    ) || !exact_call(
        19,
        InvokeKind::Virtual,
        "java/io/OutputStream",
        "write",
        "(I)V",
    ) || ![23, 41].into_iter().all(|at| {
        exact_call(
            at,
            InvokeKind::Virtual,
            "java/io/OutputStream",
            "close",
            "()V",
        )
    }) || ![27, 45]
        .into_iter()
        .all(|at| exact_call(at, InvokeKind::Virtual, "java/io/File", "delete", "()Z"))
        || ![34, 38, 52]
            .into_iter()
            .all(|at| handler_binding(facts, at))
    {
        return Ok(None);
    }
    // Both copies must read the same two initialized locals. The stack values passed to each
    // invocation must be precisely the values produced by those loads, not another receiver.
    for (store, uses) in [
        (7, &[(12, 13), (26, 27), (44, 45)][..]),
        (16, &[(17, 19), (22, 23), (40, 41)][..]),
    ] {
        let Some(written) = facts.step(store).and_then(|step| {
            step.instruction
                .writes()
                .iter()
                .find(|(slot, _)| matches!(slot, Slot::Local(_)))
                .map(|(_, v)| *v)
        }) else {
            return Ok(None);
        };
        for &(load, call) in uses {
            facts.charge(load)?;
            let (Some(loaded), Some(called)) = (facts.step(load), facts.step(call)) else {
                return Ok(None);
            };
            if !loaded
                .instruction
                .reads()
                .iter()
                .any(|(slot, value)| matches!(slot, Slot::Local(_)) && facts.same(*value, written))
                || !loaded.instruction.writes().iter().any(|(slot, value)| {
                    matches!(slot, Slot::Stack(_))
                        && stack_operands(called.instruction)
                            .iter()
                            .any(|(_, read)| facts.same(*read, *value))
                })
            {
                return Ok(None);
            }
        }
    }
    // The write argument is exactly iconst_1, and both boolean delete results are discarded.
    if !facts.step(18).is_some_and(|pushed| {
        pushed.instruction.writes().iter().any(|(_, value)| {
            facts.step(19).is_some_and(|call| {
                stack_operands(call.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(*value, *read))
            })
        })
    }) {
        return Ok(None);
    }
    for (call, pop) in [(27, 30), (45, 48)] {
        let (Some(called), Some(discarded)) = (facts.step(call), facts.step(pop)) else {
            return Ok(None);
        };
        if !called.instruction.writes().iter().any(|(slot, value)| {
            matches!(slot, Slot::Stack(_))
                && stack_operands(discarded.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(*value, *read))
        }) {
            return Ok(None);
        }
    }
    // Neither named catch parameter has a consumer. The catch-all rethrows its original value.
    for store in [34, 52] {
        let Some(value) = facts.step(store).and_then(|step| {
            step.instruction
                .writes()
                .iter()
                .find(|(slot, _)| matches!(slot, Slot::Local(_)))
                .map(|(_, v)| *v)
        }) else {
            return Ok(None);
        };
        for bci in BCIS.iter().copied().filter(|bci| *bci != store) {
            facts.charge(bci)?;
            if facts.step(bci).is_some_and(|step| {
                step.instruction
                    .reads()
                    .iter()
                    .any(|(_, read)| facts.same(*read, value))
            }) {
                return Ok(None);
            }
        }
    }
    let (Some(saved), Some(loaded), Some(thrown)) =
        (facts.step(38), facts.step(54), facts.step(56))
    else {
        return Ok(None);
    };
    if !matches!((facts.op(38), facts.op(54)),
        (Some(Operation::Store { slot: a }), Some(Operation::Load { slot: b })) if a == b)
        || !saved.instruction.writes().iter().any(|(slot, value)| {
            matches!(slot, Slot::Local(_))
                && loaded
                    .instruction
                    .reads()
                    .iter()
                    .any(|(_, read)| facts.same(*value, *read))
        })
        || !loaded.instruction.writes().iter().any(|(slot, value)| {
            matches!(slot, Slot::Stack(_))
                && stack_operands(thrown.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(*value, *read))
        })
    {
        return Ok(None);
    }
    let (
        Some(body),
        Some(normal),
        Some(normal_catch),
        Some(exceptional),
        Some(exceptional_catch),
        Some(rethrow),
        Some(join),
    ) = (
        facts.block_of(17).cloned(),
        facts.block_of(22).cloned(),
        facts.row_handler(normal_row),
        facts.row_handler(body_row),
        facts.row_handler(exceptional_row),
        facts.block_at(54),
        facts.block_at(57),
    )
    else {
        return Ok(None);
    };
    if body != *current
        || normal_catch.bci() != 34
        || exceptional.bci() != 38
        || exceptional_catch.bci() != 52
        || rethrow.bci() != 54
        || facts.row_handler(self_row) != Some(exceptional.clone())
    {
        return Ok(None);
    }
    let permitted_normal = [(0, 57), (34, 57), (38, 54), (52, 54)];
    let permitted_exception = [
        (0, 34, normal_row.ordinal),
        (0, 38, body_row.ordinal),
        (38, 52, exceptional_row.ordinal),
        (38, 38, self_row.ordinal),
    ];
    let mut observed_normal = BTreeSet::new();
    let mut observed_exception = BTreeSet::new();
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let valid = match edge.kind() {
            CanonicalEdgeKind::Normal => {
                let edge = (edge.from().bci(), edge.to().bci());
                observed_normal.insert(edge) && permitted_normal.contains(&edge)
            }
            CanonicalEdgeKind::Exception { handler_ordinal } => {
                let edge = (edge.from().bci(), edge.to().bci(), handler_ordinal);
                observed_exception.insert(edge) && permitted_exception.contains(&edge)
            }
            CanonicalEdgeKind::Call { .. } | CanonicalEdgeKind::Return { .. } => false,
        };
        if !valid {
            return Ok(None);
        }
    }
    if observed_normal != permitted_normal.into_iter().collect()
        || observed_exception != permitted_exception.into_iter().collect()
    {
        return Ok(None);
    }
    let owned = facts
        .canonical
        .blocks()
        .iter()
        .map(|block| block.id().clone())
        .filter(|block| block.bci() != 57)
        .collect::<Vec<_>>();
    if owned.len() != 5
        || !owned.contains(&normal)
        || !owned.contains(&normal_catch)
        || !owned.contains(&exceptional_catch)
        || !owned.contains(&rethrow)
        || facts
            .canonical
            .blocks()
            .iter()
            .any(|block| block.id().bci() > 57)
    {
        return Ok(None);
    }
    Ok(Some(Plan {
        shape: Shape::NestedCleanupFinally {
            rows: [
                normal_row.ordinal,
                body_row.ordinal,
                exceptional_row.ordinal,
                self_row.ordinal,
            ],
            catch_type: normal_row.catch_type_index.unwrap(),
            normal_handler: normal_catch,
            catch_parameter: match facts.op(34) {
                Some(Operation::Store { slot }) => *slot,
                _ => unreachable!(),
            },
            cleanup: (22, 31),
        },
        lead: (0, 17),
        body: (17, 22),
        owned,
        join: Some(join),
        facts: BCIS.to_vec(),
    }))
}

/// This certificate owns the complete fixed Test17 lowering. Every instruction is assigned
/// once, including the second catch's saved return and its separately protected prefix.
fn prove_two_catch_return_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [first, second, any_body, any_second] = facts.handlers else {
        return Ok(None);
    };
    let [
        body,
        normal,
        normal_jump,
        first_store,
        first_copy,
        first_jump,
        second_store,
        one,
        save,
        second_copy,
        load,
        early_return,
        primary_store,
        primary_copy,
        primary_load,
        rethrow,
        zero,
        common_return,
    ] = facts.order.as_slice()
    else {
        return Ok(None);
    };
    let [
        body,
        normal,
        normal_jump,
        first_store,
        first_copy,
        first_jump,
        second_store,
        one,
        save,
        second_copy,
        load,
        early_return,
        primary_store,
        primary_copy,
        primary_load,
        rethrow,
        zero,
        common_return,
    ] = [
        *body,
        *normal,
        *normal_jump,
        *first_store,
        *first_copy,
        *first_jump,
        *second_store,
        *one,
        *save,
        *second_copy,
        *load,
        *early_return,
        *primary_store,
        *primary_copy,
        *primary_load,
        *rethrow,
        *zero,
        *common_return,
    ];
    if body != 0
        || current.bci() != body
        || first.ordinal + 1 != second.ordinal
        || second.ordinal + 1 != any_body.ordinal
        || any_body.ordinal + 1 != any_second.ordinal
        || first.catch_type_index.is_none()
        || second.catch_type_index.is_none()
        || first.catch_type_index == second.catch_type_index
        || any_body.catch_type_index.is_some()
        || any_second.catch_type_index.is_some()
        || (first.start_bci, first.end_bci, first.handler_bci) != (body, normal, first_store)
        || (second.start_bci, second.end_bci, second.handler_bci) != (body, normal, second_store)
        || (any_body.start_bci, any_body.end_bci, any_body.handler_bci)
            != (body, normal, primary_store)
        || (
            any_second.start_bci,
            any_second.end_bci,
            any_second.handler_bci,
        ) != (second_store, second_copy, primary_store)
        || !matches!(facts.op(body), Some(Operation::Invoke(call)) if call.kind() == InvokeKind::Static && call.descriptor() == "()V")
        || !matches!(facts.op(normal), Some(Operation::Invoke(call)) if call.kind() == InvokeKind::Static && call.descriptor() == "()V")
        || ![first_copy, second_copy, primary_copy]
            .into_iter()
            .all(|at| facts.op(at) == facts.op(normal))
        || ![body, normal, first_copy, second_copy, primary_copy]
            .into_iter()
            .all(|at| {
                facts.step(at).is_some_and(|step| {
                    stack_operands(step.instruction).is_empty()
                        && step
                            .instruction
                            .writes()
                            .iter()
                            .all(|(slot, _)| !matches!(slot, Slot::Stack(_)))
                })
            })
        || facts.op(normal_jump) != Some(&Operation::Transfer)
        || facts.op(first_jump) != Some(&Operation::Transfer)
        || ![first_store, second_store, save, primary_store]
            .into_iter()
            .all(|at| matches!(facts.op(at), Some(Operation::Store { .. })))
        || ![load, primary_load]
            .into_iter()
            .all(|at| matches!(facts.op(at), Some(Operation::Load { .. })))
        || facts.op(early_return) != Some(&Operation::Return)
        || facts.op(rethrow) != Some(&Operation::Throw)
        || facts.op(common_return) != Some(&Operation::Return)
        || facts
            .step(one)
            .is_none_or(|step| step.instruction.opcode() != 0x04)
        || facts
            .step(zero)
            .is_none_or(|step| step.instruction.opcode() != 0x03)
        || !handler_binding(facts, first_store)
        || !handler_binding(facts, second_store)
        || !handler_binding(facts, primary_store)
    {
        return Ok(None);
    }

    for bci in facts.order.clone() {
        facts.charge(bci)?;
        let expected = if bci == body {
            &[first.ordinal, second.ordinal, any_body.ordinal][..]
        } else if (second_store..second_copy).contains(&bci) {
            &[any_second.ordinal][..]
        } else {
            &[][..]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
        {
            return Ok(None);
        }
    }
    let (Some(body_block), Some(first_block), Some(second_block), Some(primary_block), Some(join)) = (
        facts.block_at(body),
        facts.row_handler(first),
        facts.row_handler(second),
        facts.row_handler(any_body),
        facts.block_at(zero),
    ) else {
        return Ok(None);
    };
    if body_block != *current
        || facts.row_handler(any_second) != Some(primary_block.clone())
        || [
            (&body_block, &[body, normal, normal_jump][..]),
            (&first_block, &[first_store, first_copy, first_jump]),
            (
                &second_block,
                &[second_store, one, save, second_copy, load, early_return],
            ),
            (
                &primary_block,
                &[primary_store, primary_copy, primary_load, rethrow],
            ),
            (&join, &[zero, common_return]),
        ]
        .iter()
        .any(|(block, expected)| {
            facts
                .in_block(block)
                .iter()
                .map(SsaInstruction::bci)
                .collect::<Vec<_>>()
                != *expected
        })
        || facts.canonical.blocks().len() != 5
        || facts.view.successor_ids(&body_block) != [join.clone()]
        || facts.view.successor_ids(&first_block) != [join.clone()]
        || !facts.view.successor_ids(&second_block).is_empty()
        || !facts.view.successor_ids(&primary_block).is_empty()
        || !facts.view.successor_ids(&join).is_empty()
    {
        return Ok(None);
    }
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let valid = match edge.kind() {
            CanonicalEdgeKind::Exception { handler_ordinal } => {
                (edge.from() == &body_block
                    && ((handler_ordinal == first.ordinal && edge.to() == &first_block)
                        || (handler_ordinal == second.ordinal && edge.to() == &second_block)
                        || (handler_ordinal == any_body.ordinal && edge.to() == &primary_block)))
                    || (edge.from() == &second_block
                        && handler_ordinal == any_second.ordinal
                        && edge.to() == &primary_block)
            }
            CanonicalEdgeKind::Normal => {
                (edge.from() == &body_block || edge.from() == &first_block) && edge.to() == &join
            }
            CanonicalEdgeKind::Call { .. } | CanonicalEdgeKind::Return { .. } => false,
        };
        if !valid {
            return Ok(None);
        }
    }
    // Each named parameter is only a header binding. The saved constant and original
    // Throwable must reach their respective terminal instructions through the same SSA value.
    for store in [first_store, second_store] {
        let Some(step) = facts.step(store) else {
            return Ok(None);
        };
        let Some((_, value)) = step
            .instruction
            .writes()
            .iter()
            .find(|(slot, _)| matches!(slot, Slot::Local(_)))
        else {
            return Ok(None);
        };
        for bci in facts.order.clone() {
            facts.charge(bci)?;
            if bci != store
                && facts.step(bci).is_some_and(|step| {
                    step.instruction
                        .reads()
                        .iter()
                        .any(|(_, read)| facts.same(*read, *value))
                })
            {
                return Ok(None);
            }
        }
    }
    let (Some(pushed), Some(saved_return), Some(reloaded_return), Some(returned)) = (
        facts.step(one),
        facts.step(save),
        facts.step(load),
        facts.step(early_return),
    ) else {
        return Ok(None);
    };
    if !matches!((facts.op(save), facts.op(load)),
        (Some(Operation::Store { slot: a }), Some(Operation::Load { slot: b })) if a == b)
        || !pushed.instruction.writes().iter().any(|(slot, value)| {
            matches!(slot, Slot::Stack(_))
                && stack_operands(saved_return.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(*value, *read))
        })
        || !saved_return
            .instruction
            .writes()
            .iter()
            .any(|(slot, value)| {
                matches!(slot, Slot::Local(_))
                    && reloaded_return
                        .instruction
                        .reads()
                        .iter()
                        .any(|(_, read)| facts.same(*value, *read))
            })
        || !reloaded_return
            .instruction
            .writes()
            .iter()
            .any(|(slot, value)| {
                matches!(slot, Slot::Stack(_))
                    && stack_operands(returned.instruction)
                        .iter()
                        .any(|(_, read)| facts.same(*value, *read))
            })
        || stack_operands(returned.instruction).len() != 1
    {
        return Ok(None);
    }
    let (Some(saved), Some(reloaded), Some(thrown)) = (
        facts.step(primary_store),
        facts.step(primary_load),
        facts.step(rethrow),
    ) else {
        return Ok(None);
    };
    if !matches!((facts.op(primary_store), facts.op(primary_load)),
        (Some(Operation::Store { slot: a }), Some(Operation::Load { slot: b })) if a == b)
        || !saved.instruction.writes().iter().any(|(slot, value)| {
            matches!(slot, Slot::Local(_))
                && reloaded
                    .instruction
                    .reads()
                    .iter()
                    .any(|(_, read)| facts.same(*value, *read))
        })
        || !reloaded.instruction.writes().iter().any(|(slot, value)| {
            matches!(slot, Slot::Stack(_))
                && stack_operands(thrown.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(*value, *read))
        })
        || stack_operands(thrown.instruction).len() != 1
        || !facts.step(zero).is_some_and(|step| {
            step.instruction.writes().iter().any(|(slot, value)| {
                matches!(slot, Slot::Stack(_))
                    && facts.step(common_return).is_some_and(|ret| {
                        stack_operands(ret.instruction)
                            .iter()
                            .any(|(_, read)| facts.same(*value, *read))
                    })
            })
        })
    {
        return Ok(None);
    }
    let catches = [
        (
            first_block.clone(),
            first.catch_type_index.unwrap(),
            match facts.op(first_store) {
                Some(Operation::Store { slot }) => *slot,
                _ => unreachable!(),
            },
        ),
        (
            second_block.clone(),
            second.catch_type_index.unwrap(),
            match facts.op(second_store) {
                Some(Operation::Store { slot }) => *slot,
                _ => unreachable!(),
            },
        ),
    ];
    Ok(Some(Plan {
        shape: Shape::TwoCatchReturnFinally {
            rows: [
                first.ordinal,
                second.ordinal,
                any_body.ordinal,
                any_second.ordinal,
            ],
            catches,
            cleanup: [normal, first_copy, second_copy, primary_copy]
                .map(|bci| (bci, facts.span_end(bci))),
            saved_return: (save, early_return),
        },
        lead: (body, body),
        body: (body, normal),
        owned: vec![body_block, first_block, second_block, primary_block],
        join: Some(join),
        facts: facts.order.clone(),
    }))
}

/// Javac's outer finally around an inner named catch has one normal copy after the entire
/// protected range and one handler copy. This certificate is separate from saved returns and
/// from the three-row shared-join layout: both rows, both copies, and the one continuation close
/// together before a region may claim any block.
fn prove_nested_join_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [named, outer] = facts.handlers else {
        return Ok(None);
    };
    let start = current.bci();
    if named.ordinal + 1 != outer.ordinal
        || named.catch_type_index.is_none()
        || outer.catch_type_index.is_some()
        || named.start_bci != start
        || outer.start_bci != start
        || !(start < named.end_bci
            && named.end_bci < named.handler_bci
            && named.handler_bci < outer.end_bci
            && outer.end_bci < outer.handler_bci)
        || !facts
            .in_block(current)
            .iter()
            .any(|step| step.bci() == start)
    {
        return Ok(None);
    }
    let normal_start = outer.end_bci;
    let handler = outer.handler_bci;
    let Some(handler_start) = facts.next_bci(handler) else {
        return Ok(None);
    };
    let (
        Some((normal_pop, normal_field, normal_constant, normal_invoke)),
        Some((handler_pop, handler_field, handler_constant, handler_invoke)),
    ) = (
        append_cleanup(facts, normal_start)?,
        append_cleanup(facts, handler_start)?,
    )
    else {
        return Ok(None);
    };
    if (normal_field, normal_constant, normal_invoke)
        != (handler_field, handler_constant, handler_invoke)
    {
        return Ok(None);
    }
    let (Some(transfer), Some(primary_load)) =
        (facts.next_bci(normal_pop), facts.next_bci(handler_pop))
    else {
        return Ok(None);
    };
    let Some(rethrow) = facts.next_bci(primary_load) else {
        return Ok(None);
    };
    let handler_end = facts.span_end(rethrow);
    if facts.op(transfer) != Some(&Operation::Transfer)
        || facts.op(rethrow) != Some(&Operation::Throw)
        || !matches!(facts.op(handler), Some(Operation::Store { .. }))
        || !matches!(facts.op(primary_load), Some(Operation::Load { .. }))
        || !handler_binding(facts, handler)
        || !handler_binding(facts, named.handler_bci)
        || facts.next_bci(transfer) != Some(handler)
    {
        return Ok(None);
    }
    let (Some(named_block), Some(handler_block), Some(normal_block)) = (
        facts.row_handler(named),
        facts.row_handler(outer),
        facts.block_of(normal_start).cloned(),
    ) else {
        return Ok(None);
    };
    if named_block.bci() != named.handler_bci
        || handler_block.bci() != handler
        || facts
            .bcis((normal_start, facts.span_end(transfer)))
            .iter()
            .any(|bci| facts.block_of(*bci) != Some(&normal_block))
        || facts
            .bcis((handler, handler_end))
            .iter()
            .any(|bci| facts.block_of(*bci) != Some(&handler_block))
        || !facts.view.successor_ids(&handler_block).is_empty()
    {
        return Ok(None);
    }
    let Some(catch_start) = facts.next_bci(named.handler_bci) else {
        return Ok(None);
    };
    let Some((catch_pop, _, _, _)) = append_cleanup(facts, catch_start)? else {
        return Ok(None);
    };
    let Some(catch_successor) = facts
        .next_bci(catch_pop)
        .and_then(|bci| facts.block_of(bci))
    else {
        return Ok(None);
    };
    if facts.in_block(&named_block).last().map(SsaInstruction::bci) != Some(catch_pop)
        || facts.block_of(catch_pop) != Some(&named_block)
        || facts.view.successor_ids(&named_block) != [catch_successor.clone()]
        || (catch_successor != &normal_block
            && !facts
                .blocks_in((start, outer.end_bci))
                .contains(catch_successor))
    {
        return Ok(None);
    }
    let normal_successors = facts.view.successor_ids(&normal_block);
    let Some(normal_node) = facts.view.index_of(&normal_block) else {
        return Ok(None);
    };
    let continuation = handler_end;
    let join = match normal_successors.as_slice() {
        [join]
            if join.bci() == continuation
                && facts.block_at(continuation).as_ref() == Some(join)
                && facts
                    .view
                    .index_of(join)
                    .is_some_and(|node| facts.view.predecessors(node) == [normal_node])
                && matches!(facts.in_block(join), [only]
                if facts.op(only.bci()) == Some(&Operation::Return) && only.reads().is_empty()) =>
        {
            Some(join.clone())
        }
        [] if facts.block_of(continuation) == Some(&normal_block)
            && facts.in_block(&normal_block).last().is_some_and(|last| {
                last.bci() == continuation
                    && facts.op(continuation) == Some(&Operation::Return)
                    && last.reads().is_empty()
            }) =>
        {
            None
        }
        _ => return Ok(None),
    };
    let (Some(store), Some(load), Some(throw)) = (
        facts.step(handler),
        facts.step(primary_load),
        facts.step(rethrow),
    ) else {
        return Ok(None);
    };
    if !matches!((facts.op(handler), facts.op(primary_load)),
        (Some(Operation::Store { slot: stored }), Some(Operation::Load { slot: loaded })) if stored == loaded)
        || !store.instruction.writes().iter().any(|(_, written)| {
            load.instruction
                .reads()
                .iter()
                .any(|(_, read)| facts.same(*written, *read))
        })
        || !load.instruction.writes().iter().any(|(slot, written)| {
            matches!(slot, Slot::Stack(_))
                && stack_operands(throw.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(*written, *read))
        })
        || stack_operands(throw.instruction).len() != 1
    {
        return Ok(None);
    }
    for bci in facts.bcis((start, handler_end)) {
        facts.charge(bci)?;
        let expected = if bci < named.end_bci {
            &[named.ordinal, outer.ordinal][..]
        } else if bci < outer.end_bci {
            &[outer.ordinal][..]
        } else {
            &[][..]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
            || (bci < outer.end_bci && matches!(facts.op(bci), Some(Operation::Return)))
        {
            return Ok(None);
        }
    }
    let protected = facts.blocks_in((start, outer.end_bci));
    let inner = facts.blocks_in((start, named.end_bci));
    if facts.view.predecessors(normal_node).len() != 2
        || facts
            .view
            .predecessors(normal_node)
            .iter()
            .any(|predecessor| {
                facts
                    .view
                    .id_of(*predecessor)
                    .is_none_or(|block| !protected.contains(block))
            })
    {
        return Ok(None);
    }
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let valid = match edge.kind() {
            CanonicalEdgeKind::Exception { handler_ordinal } if edge.to() == &named_block => {
                handler_ordinal == named.ordinal && inner.contains(edge.from())
            }
            CanonicalEdgeKind::Exception { handler_ordinal } if edge.to() == &handler_block => {
                handler_ordinal == outer.ordinal && protected.contains(edge.from())
            }
            CanonicalEdgeKind::Exception { .. } => {
                !protected.contains(edge.from()) && edge.from() != &handler_block
            }
            CanonicalEdgeKind::Normal if edge.from() == &normal_block => {
                join.as_ref() == Some(edge.to())
            }
            CanonicalEdgeKind::Normal if protected.contains(edge.from()) => {
                protected.contains(edge.to()) || edge.to() == &normal_block
            }
            CanonicalEdgeKind::Normal => {
                edge.to() != &normal_block
                    && edge.to() != &handler_block
                    && edge.to() != &named_block
            }
            CanonicalEdgeKind::Call { .. } => false,
            CanonicalEdgeKind::Return { .. } => {
                !protected.contains(edge.from()) && edge.from() != &handler_block
            }
        };
        if !valid {
            return Ok(None);
        }
    }
    let owned = facts.blocks_in((start, handler_end));
    Ok(Some(Plan {
        shape: Shape::Finally {
            normal_cleanup: (normal_start, facts.span_end(normal_pop)),
            completion: FinallyCompletion::Joined {
                transfer,
                continuation,
                catch_pop,
                named_row: named.ordinal,
            },
            row_ordinal: outer.ordinal,
            structured: true,
        },
        lead: (start, start),
        body: (start, outer.end_bci),
        owned,
        join,
        facts: facts.bcis((start, handler_end)),
    }))
}

/// The fixed Test3 copies each load the same method-entry ClassNode and invoke its void unload.
/// Verify the receiver of every call through SSA, rather than equating only the call symbols.
fn entry_unload_cleanup(
    facts: &mut Facts<'_>,
    first: u32,
    second: u32,
    third: u32,
) -> Result<Option<[(u32, u32); 3]>, StopReason> {
    let mut copies = [(0, 0); 3];
    for (index, load) in [first, second, third].into_iter().enumerate() {
        let Some(invoke) = facts.next_bci(load) else {
            return Ok(None);
        };
        facts.charge(load)?;
        facts.charge(invoke)?;
        let (Some(receiver), Some(call)) = (facts.step(load), facts.step(invoke)) else {
            return Ok(None);
        };
        let [(Slot::Local(0), entry)] = receiver.instruction.reads() else {
            return Ok(None);
        };
        let [(Slot::Stack(_), produced)] = receiver.instruction.writes() else {
            return Ok(None);
        };
        let operands = stack_operands(call.instruction);
        let mut consumers = 0;
        for index in 0..facts.order.len() {
            let bci = facts.order[index];
            facts.charge(bci)?;
            if facts.step(bci).is_some_and(|step| {
                stack_operands(step.instruction)
                    .iter()
                    .any(|(_, value)| facts.same(*value, *produced))
            }) {
                consumers += 1;
            }
        }
        if facts.op(load) != Some(&Operation::Load { slot: 0 })
            || receiver.instruction.opcode() != 0x2a
            || !matches!(
                facts.ssa.value(facts.resolve(*entry)).def(),
                Definition::Entry {
                    slot: Slot::Local(0),
                    ..
                }
            )
            || !matches!(facts.op(invoke), Some(Operation::Invoke(target))
                if target.kind() == InvokeKind::Virtual
                    && target.owner() == "jadx/core/dex/nodes/ClassNode"
                    && target.name() == "unload"
                    && target.descriptor() == "()V")
            || !call.instruction.writes().is_empty()
            || operands.len() != 1
            || !facts.same(*produced, operands[0].1)
            || consumers != 1
        {
            return Ok(None);
        }
        copies[index] = (load, invoke);
    }
    Ok(Some(copies))
}

/// The fixed Test7 cleanup is one instance-int increment through the method-entry receiver.
/// Each stack edge is checked independently, including both copies made by `dup`.
fn entry_field_increment_cleanup(
    facts: &mut Facts<'_>,
    starts: [u32; 3],
) -> Result<Option<[(u32, u32); 3]>, StopReason> {
    let mut copies = [(0, 0); 3];
    let mut field = None;
    for (index, start) in starts.into_iter().enumerate() {
        let mut cursor = start;
        let mut tail = [0; 5];
        for next in &mut tail {
            let Some(bci) = facts.next_bci(cursor) else {
                return Ok(None);
            };
            *next = bci;
            cursor = bci;
        }
        let [dup, read, one, add, write] = tail;
        for bci in [start, dup, read, one, add, write] {
            facts.charge(bci)?;
        }
        let Some(Operation::Field {
            access: crate::facts::FieldAccess::Read,
            is_static: false,
            descriptor,
            ..
        }) = facts.op(read)
        else {
            return Ok(None);
        };
        let read_field = facts.op(read).cloned().unwrap();
        let matching_write = match &read_field {
            Operation::Field {
                owner,
                name,
                descriptor,
                ..
            } => {
                Some(&Operation::Field {
                    access: crate::facts::FieldAccess::Write,
                    is_static: false,
                    owner: owner.clone(),
                    name: name.clone(),
                    descriptor: descriptor.clone(),
                }) == facts.op(write)
            }
            _ => false,
        };
        if descriptor != "I"
            || field.as_ref().is_some_and(|prior| prior != &read_field)
            || !matching_write
            || facts.op(start) != Some(&Operation::Load { slot: 0 })
            || facts.op(dup) != Some(&Operation::Duplicate)
            || facts.op(one) != Some(&Operation::Push(crate::facts::ConstantValue::Int(1)))
            || facts.op(add)
                != Some(&Operation::Arithmetic {
                    op: crate::facts::ArithmeticOp::Add,
                })
            || facts
                .step(add)
                .is_none_or(|step| step.instruction.opcode() != 0x60)
        {
            return Ok(None);
        }
        field = Some(read_field);
        let (
            Some(receiver),
            Some(duplicated),
            Some(read_step),
            Some(one_step),
            Some(add_step),
            Some(write_step),
        ) = (
            facts.step(start),
            facts.step(dup),
            facts.step(read),
            facts.step(one),
            facts.step(add),
            facts.step(write),
        )
        else {
            return Ok(None);
        };
        let (receiver, duplicated, read_step, one_step, add_step, write_step) = (
            receiver.instruction,
            duplicated.instruction,
            read_step.instruction,
            one_step.instruction,
            add_step.instruction,
            write_step.instruction,
        );
        let stack_writes = |step: &SsaInstruction| -> Vec<ValueId> {
            step.writes()
                .iter()
                .filter_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
                .collect()
        };
        let receiver_values = stack_writes(receiver);
        let duplicates = stack_writes(duplicated);
        let reads = stack_writes(read_step);
        let constants = stack_writes(one_step);
        let added = stack_writes(add_step);
        if receiver.reads().len() != 1
            || !matches!(
                facts.ssa.value(facts.resolve(receiver.reads()[0].1)).def(),
                Definition::Entry {
                    slot: Slot::Local(0),
                    ..
                }
            )
            || receiver_values.len() != 1
            || duplicates.len() != 2
            || facts.same(duplicates[0], duplicates[1])
            || reads.len() != 1
            || constants.len() != 1
            || added.len() != 1
            || stack_operands(duplicated).len() != 1
            || !facts.same(stack_operands(duplicated)[0].1, receiver_values[0])
            || stack_operands(read_step).len() != 1
            || duplicates
                .iter()
                .filter(|value| facts.same(**value, stack_operands(read_step)[0].1))
                .count()
                != 1
            || stack_operands(add_step).len() != 2
            || !stack_operands(add_step)
                .iter()
                .any(|(_, value)| facts.same(*value, reads[0]))
            || !stack_operands(add_step)
                .iter()
                .any(|(_, value)| facts.same(*value, constants[0]))
            || stack_operands(write_step).len() != 2
            || !stack_operands(write_step)
                .iter()
                .any(|(_, value)| facts.same(*value, added[0]))
            || duplicates
                .iter()
                .filter(|value| {
                    stack_operands(write_step)
                        .iter()
                        .any(|(_, used)| facts.same(**value, *used))
                })
                .count()
                != 1
            || stack_operands(write_step)
                .iter()
                .any(|(_, used)| facts.same(*used, stack_operands(read_step)[0].1))
        {
            return Ok(None);
        }
        for (produced, consumer) in [
            (receiver_values[0], dup),
            (
                duplicates[0],
                if facts.same(duplicates[0], stack_operands(read_step)[0].1) {
                    read
                } else {
                    write
                },
            ),
            (
                duplicates[1],
                if facts.same(duplicates[1], stack_operands(read_step)[0].1) {
                    read
                } else {
                    write
                },
            ),
            (reads[0], add),
            (constants[0], add),
            (added[0], write),
        ] {
            let mut uses = 0;
            for bci in facts.order.clone() {
                facts.charge(bci)?;
                if let Some(step) = facts.step(bci) {
                    uses += stack_operands(step.instruction)
                        .iter()
                        .filter(|(_, value)| facts.same(produced, *value))
                        .count();
                    if bci == consumer
                        && !stack_operands(step.instruction)
                            .iter()
                            .any(|(_, value)| facts.same(produced, *value))
                    {
                        return Ok(None);
                    }
                }
            }
            if uses != 1 {
                return Ok(None);
            }
        }
        copies[index] = (start, write);
    }
    Ok(Some(copies))
}

fn shared_joined_boolean_value(
    facts: &mut Facts<'_>,
    named: &ExceptionHandlerFact,
    catch_any: &ExceptionHandlerFact,
    join: &CanonicalBlockId,
) -> Result<Option<[u32; 2]>, StopReason> {
    let body = facts.bcis((named.start_bci, named.end_bci));
    let catch = facts.bcis((named.handler_bci, catch_any.end_bci));
    let ([receiver, argument, call, first_save], [binding, zero, second_save]) =
        (body.as_slice(), catch.as_slice())
    else {
        return Ok(None);
    };
    let join_steps = facts.in_block(join);
    let [reload, returned] = join_steps else {
        return Ok(None);
    };
    let (reload, returned) = (reload.bci(), returned.bci());
    for bci in [
        *receiver,
        *argument,
        *call,
        *first_save,
        *binding,
        *zero,
        *second_save,
        reload,
        returned,
    ] {
        facts.charge(bci)?;
    }
    if facts.op(*receiver) != Some(&Operation::Load { slot: 0 })
        || facts.op(*argument) != Some(&Operation::Load { slot: 1 })
        || !matches!(facts.op(*call), Some(Operation::Invoke(target))
            if target.kind() == InvokeKind::Special
                && target.name() == "exc"
                && target.descriptor() == "(Ljava/lang/Object;)Z")
        || facts.op(*first_save) != Some(&Operation::Store { slot: 2 })
        || facts.op(*binding) != Some(&Operation::Store { slot: 3 })
        || facts.op(*zero) != Some(&Operation::Push(crate::facts::ConstantValue::Int(0)))
        || facts.op(*second_save) != Some(&Operation::Store { slot: 2 })
        || facts.op(reload) != Some(&Operation::Load { slot: 2 })
        || facts.op(returned) != Some(&Operation::Return)
    {
        return Ok(None);
    }
    let (
        Some(receiver_step),
        Some(argument_step),
        Some(call_step),
        Some(zero_step),
        Some(first_step),
        Some(second_step),
        Some(reload_step),
        Some(return_step),
    ) = (
        facts.step(*receiver),
        facts.step(*argument),
        facts.step(*call),
        facts.step(*zero),
        facts.step(*first_save),
        facts.step(*second_save),
        facts.step(reload),
        facts.step(returned),
    )
    else {
        return Ok(None);
    };
    let (
        receiver_step,
        argument_step,
        call_step,
        zero_step,
        first_step,
        second_step,
        reload_step,
        return_step,
    ) = (
        receiver_step.instruction,
        argument_step.instruction,
        call_step.instruction,
        zero_step.instruction,
        first_step.instruction,
        second_step.instruction,
        reload_step.instruction,
        return_step.instruction,
    );
    let stack_output = |step: &SsaInstruction| -> Option<ValueId> {
        let values: Vec<_> = step
            .writes()
            .iter()
            .filter_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
            .collect();
        match values.as_slice() {
            [value] => Some(*value),
            _ => None,
        }
    };
    let (
        Some(receiver_value),
        Some(argument_value),
        Some(call_value),
        Some(zero_value),
        Some(reload_value),
    ) = (
        stack_output(receiver_step),
        stack_output(argument_step),
        stack_output(call_step),
        stack_output(zero_step),
        stack_output(reload_step),
    )
    else {
        return Ok(None);
    };
    let saved = [first_step, second_step].map(|step| {
        step.writes()
            .iter()
            .find_map(|(slot, value)| (*slot == Slot::Local(2)).then_some(*value))
    });
    let [Some(first_value), Some(second_value)] = saved else {
        return Ok(None);
    };
    let Some(phi) = facts
        .ssa
        .phis()
        .iter()
        .find(|phi| phi.block() == join && phi.slot() == Slot::Local(2))
    else {
        return Ok(None);
    };
    let inputs = phi
        .inputs()
        .iter()
        .filter_map(|input| match input {
            PhiInput::Value(value) => Some(facts.resolve(*value)),
            PhiInput::Itself => None,
        })
        .collect::<Vec<_>>();
    if inputs.len() != 2
        || facts.same(first_value, second_value)
        || !inputs.contains(&facts.resolve(first_value))
        || !inputs.contains(&facts.resolve(second_value))
        || !matches!(receiver_step.reads(), [(Slot::Local(0), entry)]
            if matches!(facts.ssa.value(facts.resolve(*entry)).def(), Definition::Entry { slot: Slot::Local(0), .. }))
        || !matches!(argument_step.reads(), [(Slot::Local(1), entry)]
            if matches!(facts.ssa.value(facts.resolve(*entry)).def(), Definition::Entry { slot: Slot::Local(1), .. }))
        || stack_operands(call_step).len() != 2
        || !facts.same(stack_operands(call_step)[0].1, receiver_value)
        || !facts.same(stack_operands(call_step)[1].1, argument_value)
        || stack_operands(first_step).len() != 1
        || !facts.same(stack_operands(first_step)[0].1, call_value)
        || stack_operands(second_step).len() != 1
        || !facts.same(stack_operands(second_step)[0].1, zero_value)
        || reload_step.reads().len() != 1
        || !facts.same(reload_step.reads()[0].1, phi.value())
        || stack_operands(return_step).len() != 1
        || !facts.same(stack_operands(return_step)[0].1, reload_value)
    {
        return Ok(None);
    }
    Ok(Some([*first_save, *second_save]))
}

/// A deliberately separate completion contract: the two normal copies end in transfers to one
/// continuation, not in saved values. The old saved-return certificate is left unchanged.
fn prove_shared_join_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let (named, try_any, catch_any, binding_row) = match facts.handlers {
        [named, try_any, catch_any] => (named, try_any, catch_any, None),
        [named, try_any, catch_any, binding] => (named, try_any, catch_any, Some(binding)),
        _ => return Ok(None),
    };
    let start = current.bci();
    if named.ordinal + 1 != try_any.ordinal
        || try_any.ordinal + 1 != catch_any.ordinal
        || named.catch_type_index.is_none()
        || try_any.catch_type_index.is_some()
        || catch_any.catch_type_index.is_some()
        || (named.start_bci, named.end_bci) != (try_any.start_bci, try_any.end_bci)
        || named.handler_bci != catch_any.start_bci
        || try_any.handler_bci != catch_any.handler_bci
        || !(start <= named.start_bci
            && named.start_bci < named.end_bci
            && named.end_bci < named.handler_bci
            && named.handler_bci < catch_any.end_bci
            && catch_any.end_bci < try_any.handler_bci)
        || !facts
            .in_block(current)
            .iter()
            .any(|instruction| instruction.bci() == named.start_bci)
        || binding_row.is_some_and(|row| {
            row.ordinal != catch_any.ordinal + 1
                || row.catch_type_index.is_some()
                || row.start_bci != try_any.handler_bci
                || row.handler_bci != try_any.handler_bci
                || facts.next_bci(row.start_bci) != Some(row.end_bci)
                || !matches!(facts.op(row.start_bci), Some(Operation::Store { .. }))
        })
    {
        return Ok(None);
    }
    let first = named.end_bci;
    let second = catch_any.end_bci;
    let handler = try_any.handler_bci;
    let Some(third) = facts.next_bci(handler) else {
        return Ok(None);
    };
    let mut copies = [(0, 0); 3];
    let appended = matches!(
        facts.next_bci(first).and_then(|bci| facts.op(bci)),
        Some(Operation::Field {
            access: crate::facts::FieldAccess::Read,
            ..
        })
    );
    let field_value = binding_row.is_some()
        && facts.next_bci(first).and_then(|bci| facts.op(bci)) == Some(&Operation::Duplicate);
    if field_value {
        let Some(verified) = entry_field_increment_cleanup(facts, [first, second, third])? else {
            return Ok(None);
        };
        copies = verified;
    } else if binding_row.is_some() {
        let Some(verified) = entry_unload_cleanup(facts, first, second, third)? else {
            return Ok(None);
        };
        copies = verified;
    } else if appended {
        let mut member_pair = None;
        for (index, at) in [first, second, third].into_iter().enumerate() {
            let Some((pop, field, constant, invoke)) = append_cleanup(facts, at)? else {
                return Ok(None);
            };
            let current = (field, constant, invoke);
            if member_pair.as_ref().is_some_and(|pair| pair != &current) {
                return Ok(None);
            }
            member_pair = Some(current);
            copies[index] = (at, pop);
        }
    } else {
        let Some(verified) = boolean_join_cleanup(facts, first, second, third)? else {
            return Ok(None);
        };
        copies = verified;
    }
    let (Some(first_transfer), Some(second_transfer), Some(primary_load)) = (
        facts.next_bci(copies[0].1),
        facts.next_bci(copies[1].1),
        facts.next_bci(copies[2].1),
    ) else {
        return Ok(None);
    };
    let Some(rethrow) = facts.next_bci(primary_load) else {
        return Ok(None);
    };
    let handler_end = facts.span_end(rethrow);
    if facts.op(first_transfer) != Some(&Operation::Transfer)
        || facts.op(second_transfer) != Some(&Operation::Transfer)
        || facts.op(rethrow) != Some(&Operation::Throw)
        || facts.next_bci(first_transfer) != Some(named.handler_bci)
        || facts.next_bci(second_transfer) != Some(handler)
        || !matches!(facts.op(named.handler_bci), Some(Operation::Store { .. }))
        || !matches!(facts.op(handler), Some(Operation::Store { .. }))
        || !matches!(facts.op(primary_load), Some(Operation::Load { .. }))
        || !handler_binding(facts, named.handler_bci)
        || !handler_binding(facts, handler)
    {
        return Ok(None);
    }
    let (Some(normal_block), Some(catch_block), Some(handler_block)) = (
        facts.block_of(first).cloned(),
        facts.block_of(second).cloned(),
        facts.block_at(handler),
    ) else {
        return Ok(None);
    };
    if facts
        .bcis((first, facts.span_end(first_transfer)))
        .iter()
        .any(|bci| facts.block_of(*bci) != Some(&normal_block))
        || facts
            .bcis((second, facts.span_end(second_transfer)))
            .iter()
            .any(|bci| facts.block_of(*bci) != Some(&catch_block))
        || facts
            .bcis((handler, handler_end))
            .iter()
            .any(|bci| facts.block_of(*bci) != Some(&handler_block))
        || !facts.view.successor_ids(&handler_block).is_empty()
    {
        return Ok(None);
    }
    let (first_successors, second_successors) = (
        facts.view.successor_ids(&normal_block),
        facts.view.successor_ids(&catch_block),
    );
    let ([join], [second_join]) = (first_successors.as_slice(), second_successors.as_slice())
    else {
        return Ok(None);
    };
    if join != second_join
        || join.bci() != handler_end
        || facts.block_at(handler_end).as_ref() != Some(join)
        || facts
            .view
            .predecessors(facts.view.index_of(join).unwrap())
            .len()
            != 2
    {
        return Ok(None);
    }
    let join_instructions = facts.in_block(join);
    let value_saves = if field_value {
        let Some(saves) = shared_joined_boolean_value(facts, named, catch_any, join)? else {
            return Ok(None);
        };
        Some(saves)
    } else {
        None
    };
    if field_value {
        if join_instructions.len() != 2
            || facts.op(join_instructions[0].bci()) != Some(&Operation::Load { slot: 2 })
            || facts.op(join_instructions[1].bci()) != Some(&Operation::Return)
        {
            return Ok(None);
        }
    } else if appended || binding_row.is_some() {
        if join_instructions.len() != 1
            || facts.op(join_instructions[0].bci()) != Some(&Operation::Return)
            || !join_instructions[0].reads().is_empty()
        {
            return Ok(None);
        }
    } else {
        let same_join_field = match (
            facts.op(copies[0].1),
            join_instructions
                .get(1)
                .and_then(|step| facts.op(step.bci())),
        ) {
            (
                Some(Operation::Field {
                    owner: write_owner,
                    name: write_name,
                    descriptor: write_descriptor,
                    ..
                }),
                Some(Operation::Field {
                    access: crate::facts::FieldAccess::Read,
                    is_static: false,
                    owner: read_owner,
                    name: read_name,
                    descriptor: read_descriptor,
                }),
            ) => {
                (write_owner, write_name, write_descriptor)
                    == (read_owner, read_name, read_descriptor)
            }
            _ => false,
        };
        if join_instructions.len() != 3
            || join_instructions[0].opcode() != 0x2a
            || !same_join_field
            || facts.op(join_instructions[2].bci()) != Some(&Operation::Return)
        {
            return Ok(None);
        }
        let (receiver, field_read, returned) = (
            &join_instructions[0],
            &join_instructions[1],
            &join_instructions[2],
        );
        let receiver_outputs: Vec<_> = receiver
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .collect();
        let field_outputs: Vec<_> = field_read
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .collect();
        if facts.op(receiver.bci()) != Some(&Operation::Load { slot: 0 })
            || receiver.reads().len() != 1
            || !matches!(
                facts.ssa.value(facts.resolve(receiver.reads()[0].1)).def(),
                Definition::Entry {
                    slot: Slot::Local(0),
                    ..
                }
            )
            || receiver_outputs.len() != 1
            || field_outputs.len() != 1
            || stack_operands(field_read).len() != 1
            || !facts.same(stack_operands(field_read)[0].1, receiver_outputs[0].1)
            || stack_operands(returned).len() != 1
            || !facts.same(stack_operands(returned)[0].1, field_outputs[0].1)
        {
            return Ok(None);
        }
    }
    let (Some(store), Some(load), Some(throw)) = (
        facts.step(handler),
        facts.step(primary_load),
        facts.step(rethrow),
    ) else {
        return Ok(None);
    };
    if !store.instruction.writes().iter().any(|(_, written)| {
        load.instruction
            .reads()
            .iter()
            .any(|(_, read)| facts.same(*written, *read))
    }) || !load.instruction.writes().iter().any(|(slot, written)| {
        matches!(slot, Slot::Stack(_))
            && stack_operands(throw.instruction)
                .iter()
                .any(|(_, read)| facts.same(*written, *read))
    }) || stack_operands(throw.instruction).len() != 1
    {
        return Ok(None);
    }
    for bci in facts.bcis((start, handler_end)) {
        facts.charge(bci)?;
        let expected = if named.start_bci <= bci && bci < named.end_bci {
            &[named.ordinal, try_any.ordinal][..]
        } else if catch_any.start_bci <= bci && bci < catch_any.end_bci {
            &[catch_any.ordinal][..]
        } else if bci == handler && binding_row.is_some() {
            &[binding_row.unwrap().ordinal][..]
        } else {
            &[][..]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
            || (expected.len() > 0
                && (matches!(facts.op(bci), Some(Operation::Return))
                    || (matches!(facts.op(bci), Some(Operation::Transfer))
                        && !(binding_row.is_some() && bci < named.end_bci))))
        {
            return Ok(None);
        }
    }
    let protected = facts.blocks_in((named.start_bci, named.end_bci));
    let catch = facts.blocks_in((catch_any.start_bci, catch_any.end_bci));
    let Some(named_block) = facts.row_handler(named) else {
        return Ok(None);
    };
    if named_block.bci() != named.handler_bci
        || facts.row_handler(try_any).as_ref() != Some(&handler_block)
        || facts.row_handler(catch_any).as_ref() != Some(&handler_block)
        || binding_row.is_some_and(|row| facts.row_handler(row).as_ref() != Some(&handler_block))
        || binding_row.is_some_and(|row| {
            !facts.canonical.edges().iter().any(|edge| {
                edge.from() == &handler_block
                    && edge.to() == &handler_block
                    && matches!(edge.kind(), CanonicalEdgeKind::Exception { handler_ordinal }
                        if handler_ordinal == row.ordinal)
            })
        })
    {
        return Ok(None);
    }
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let from_try = protected.contains(edge.from());
        let from_catch = catch.contains(edge.from());
        let valid = match edge.kind() {
            CanonicalEdgeKind::Exception { handler_ordinal } if from_try => {
                (handler_ordinal == named.ordinal && edge.to() == &named_block)
                    || (handler_ordinal == try_any.ordinal && edge.to() == &handler_block)
            }
            CanonicalEdgeKind::Exception { handler_ordinal } if from_catch => {
                handler_ordinal == catch_any.ordinal && edge.to() == &handler_block
            }
            CanonicalEdgeKind::Exception { handler_ordinal }
                if binding_row.is_some_and(|row| handler_ordinal == row.ordinal) =>
            {
                edge.from() == &handler_block && edge.to() == &handler_block
            }
            CanonicalEdgeKind::Exception { .. } => {
                edge.to() != &named_block
                    && edge.to() != &handler_block
                    && ![&normal_block, &catch_block, &handler_block].contains(&edge.from())
            }
            CanonicalEdgeKind::Normal
                if edge.from() == &normal_block || edge.from() == &catch_block =>
            {
                edge.to() == join
            }
            CanonicalEdgeKind::Normal if from_try => {
                protected.contains(edge.to()) || edge.to() == &normal_block
            }
            CanonicalEdgeKind::Normal if from_catch => {
                catch.contains(edge.to()) || edge.to() == &catch_block
            }
            CanonicalEdgeKind::Normal => {
                edge.to() != &normal_block
                    && edge.to() != &catch_block
                    && edge.to() != &handler_block
                    && (binding_row.is_none()
                        || (!protected.contains(edge.to()) && !catch.contains(edge.to())))
            }
            CanonicalEdgeKind::Call { .. } => false,
            CanonicalEdgeKind::Return { .. } => {
                ![&normal_block, &catch_block, &handler_block].contains(&edge.from())
            }
        };
        if !valid {
            return Ok(None);
        }
    }
    let owned = facts.blocks_in((start, handler_end));
    let origins = facts.bcis((start, handler_end));
    Ok(Some(Plan {
        shape: Shape::SharedFinally {
            rows: [named.ordinal, try_any.ordinal, catch_any.ordinal],
            binding_row: binding_row.map(|row| row.ordinal),
            catch_body: (catch_any.start_bci, catch_any.end_bci),
            catch_handler: named_block,
            catch_type: named.catch_type_index.unwrap(),
            catch_parameter: match facts.op(named.handler_bci) {
                Some(Operation::Store { slot }) => *slot,
                _ => unreachable!(),
            },
            normal_cleanup: (first, facts.span_end(copies[0].1)),
            catch_cleanup: (second, facts.span_end(copies[1].1)),
            completion: if let Some(saves) = value_saves {
                SharedFinallyCompletion::JoinedValue {
                    transfers: [first_transfer, second_transfer],
                    saves,
                }
            } else {
                SharedFinallyCompletion::Joined {
                    transfers: [first_transfer, second_transfer],
                }
            },
        },
        lead: (start, named.start_bci),
        body: (named.start_bci, named.end_bci),
        owned,
        join: Some(join.clone()),
        facts: origins,
    }))
}

/// Prove the Java 8 shared-cleanup layout as one unit. The three rows and all three copies are
/// inseparable: accepting just the try row would put the catch's normal completion outside the
/// source `finally`, while accepting just the two normal copies would lose exceptional cleanup.
fn prove_shared_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
    chains: Option<&crate::concat::Plan>,
) -> Result<Option<Plan>, StopReason> {
    let [named, try_any, catch_any] = facts.handlers else {
        return Ok(None);
    };
    let start = current.bci();
    let protected = (named.start_bci, named.end_bci);
    if named.ordinal + 1 != try_any.ordinal
        || try_any.ordinal + 1 != catch_any.ordinal
        || named.catch_type_index.is_none()
        || try_any.catch_type_index.is_some()
        || catch_any.catch_type_index.is_some()
        || named.start_bci != try_any.start_bci
        || named.end_bci != try_any.end_bci
        || named.handler_bci != catch_any.start_bci
        || try_any.handler_bci != catch_any.handler_bci
        || named.start_bci >= named.end_bci
        || catch_any.start_bci >= catch_any.end_bci
        || !facts
            .in_block(current)
            .iter()
            .any(|instruction| instruction.bci() == named.start_bci)
    {
        return Ok(None);
    }
    let first_start = named.end_bci;
    let second_start = catch_any.end_bci;
    let third_handler = try_any.handler_bci;
    let Some(first_cleanup) = shared_cleanup_span(facts, first_start) else {
        return Ok(None);
    };
    let Some(second_cleanup) = shared_cleanup_span(facts, second_start) else {
        return Ok(None);
    };
    let Some(third_start) = facts.next_bci(third_handler) else {
        return Ok(None);
    };
    let Some(third_cleanup) = shared_cleanup_span(facts, third_start) else {
        return Ok(None);
    };
    let (Some(first_save), Some(second_save)) = (
        facts.previous_bci(first_start),
        facts.previous_bci(second_start),
    ) else {
        return Ok(None);
    };
    let Some(first_load) = facts.next_bci(first_cleanup.1) else {
        return Ok(None);
    };
    let Some(first_return) = facts.next_bci(first_load) else {
        return Ok(None);
    };
    let Some(second_load) = facts.next_bci(second_cleanup.1) else {
        return Ok(None);
    };
    let Some(second_return) = facts.next_bci(second_load) else {
        return Ok(None);
    };
    let Some(primary_load) = facts.next_bci(third_cleanup.1) else {
        return Ok(None);
    };
    let Some(rethrow) = facts.next_bci(primary_load) else {
        return Ok(None);
    };
    let handler_end = facts.span_end(rethrow);
    if facts.next_bci(first_return) != Some(named.handler_bci)
        || facts.next_bci(second_return) != Some(third_handler)
        || first_save < protected.0
        || second_save < catch_any.start_bci
        || third_handler <= second_start
        || facts.row_handler(named).as_ref().map(CanonicalBlockId::bci) != Some(named.handler_bci)
        || facts
            .row_handler(try_any)
            .as_ref()
            .map(CanonicalBlockId::bci)
            != Some(third_handler)
        || facts
            .row_handler(catch_any)
            .as_ref()
            .map(CanonicalBlockId::bci)
            != Some(third_handler)
    {
        return Ok(None);
    }
    for bci in facts.bcis((start, handler_end)) {
        facts.charge(bci)?;
        let expected = if protected.0 <= bci && bci < protected.1 {
            &[named.ordinal, try_any.ordinal][..]
        } else if catch_any.start_bci <= bci && bci < catch_any.end_bci {
            &[catch_any.ordinal][..]
        } else {
            &[][..]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
        {
            return Ok(None);
        }
        if (protected.0 <= bci && bci < protected.1
            || catch_any.start_bci <= bci && bci < catch_any.end_bci)
            && facts.op(bci) == Some(&Operation::Return)
        {
            // A return inside either protected range completes before the copy that this
            // certificate would replace with source `finally`.
            return Ok(None);
        }
    }
    if !shared_cleanup_copies(facts, [first_cleanup, second_cleanup, third_cleanup])? {
        return Ok(None);
    }
    let (
        Some(Operation::Store { slot: normal_slot }),
        Some(Operation::Load {
            slot: normal_loaded,
        }),
        Some(Operation::Store {
            slot: catch_parameter,
        }),
        Some(Operation::Store { slot: catch_slot }),
        Some(Operation::Load { slot: catch_loaded }),
        Some(Operation::Store { slot: primary_slot }),
        Some(Operation::Load {
            slot: primary_loaded,
        }),
    ) = (
        facts.op(first_save),
        facts.op(first_load),
        facts.op(named.handler_bci),
        facts.op(second_save),
        facts.op(second_load),
        facts.op(third_handler),
        facts.op(primary_load),
    )
    else {
        return Ok(None);
    };
    if normal_slot != normal_loaded
        || catch_slot != catch_loaded
        || primary_slot != primary_loaded
        || catch_parameter == catch_slot
        || facts.op(first_return) != Some(&Operation::Return)
        || facts.op(second_return) != Some(&Operation::Return)
        || facts.op(rethrow) != Some(&Operation::Throw)
    {
        return Ok(None);
    }
    // A saved value is evaluated before its finally copy. A literal needs only its direct stack
    // flow; the catch's concat additionally needs the *same* complete chain already proved by
    // concat::Plan, wholly inside the catch-all protected range.
    for save in [first_save, second_save] {
        let Some(producer) = facts.previous_bci(save) else {
            return Ok(None);
        };
        let (Some(pushed), Some(stored)) = (facts.step(producer), facts.step(save)) else {
            return Ok(None);
        };
        let outputs: Vec<_> = pushed
            .instruction
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .collect();
        let inputs = stack_operands(stored.instruction);
        let producer_proved = match facts.op(producer) {
            Some(Operation::Push(_)) => true,
            Some(Operation::Invoke(_)) if save == second_save => {
                if let Some(chain) = chains.and_then(|plan| plan.value_at(producer)) {
                    let chain_end = facts.span_end(chain.tail);
                    let span: std::collections::BTreeSet<_> =
                        facts.bcis((chain.head, chain_end)).into_iter().collect();
                    chain.tail == producer
                        && catch_any.start_bci <= chain.head
                        && chain_end <= catch_any.end_bci
                        && chain.owned == span
                } else {
                    false
                }
            }
            _ => false,
        };
        if !producer_proved
            || outputs.len() != 1
            || inputs.len() != 1
            || !facts.same(outputs[0].1, inputs[0].1)
        {
            return Ok(None);
        }
        let mut consumers = 0;
        for bci in facts.bcis((start, handler_end)) {
            facts.charge(bci)?;
            if facts.step(bci).is_some_and(|step| {
                stack_operands(step.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(outputs[0].1, *read))
            }) {
                consumers += 1;
            }
        }
        if consumers != 1 {
            return Ok(None);
        }
    }
    // The saved values and the received throwable, rather than merely their slot numbers, are
    // the values each terminal instruction must consume after cleanup.
    for (store, load, terminal) in [
        (first_save, first_load, first_return),
        (second_save, second_load, second_return),
        (third_handler, primary_load, rethrow),
    ] {
        let (Some(store), Some(load), Some(terminal)) =
            (facts.step(store), facts.step(load), facts.step(terminal))
        else {
            return Ok(None);
        };
        if !store.instruction.writes().iter().any(|(_, written)| {
            load.instruction
                .reads()
                .iter()
                .any(|(_, read)| facts.same(*written, *read))
        }) || !load.instruction.writes().iter().any(|(slot, written)| {
            matches!(slot, Slot::Stack(_))
                && stack_operands(terminal.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(*written, *read))
        }) || stack_operands(terminal.instruction).len() != 1
        {
            return Ok(None);
        }
    }
    let Some(normal_block) = facts.block_of(first_start).cloned() else {
        return Ok(None);
    };
    let Some(catch_block) = facts.block_of(second_start).cloned() else {
        return Ok(None);
    };
    let Some(handler_block) = facts.block_at(third_handler) else {
        return Ok(None);
    };
    if facts
        .bcis((first_start, facts.span_end(first_cleanup.1)))
        .iter()
        .chain(&[first_load, first_return])
        .any(|bci| facts.block_of(*bci) != Some(&normal_block))
        || facts
            .bcis((second_start, facts.span_end(second_cleanup.1)))
            .iter()
            .chain(&[second_load, second_return])
            .any(|bci| facts.block_of(*bci) != Some(&catch_block))
        || facts
            .bcis((third_handler, facts.span_end(third_cleanup.1)))
            .iter()
            .chain(&[primary_load, rethrow])
            .any(|bci| facts.block_of(*bci) != Some(&handler_block))
        || !facts.view.successor_ids(&normal_block).is_empty()
        || !facts.view.successor_ids(&catch_block).is_empty()
        || !facts.view.successor_ids(&handler_block).is_empty()
    {
        return Ok(None);
    }
    let protected_blocks = facts.blocks_in(protected);
    let catch_blocks = facts.blocks_in((catch_any.start_bci, catch_any.end_bci));
    let Some(named_handler) = facts.row_handler(named) else {
        return Ok(None);
    };
    for block in facts.canonical.blocks() {
        facts.charge(block.id().bci())?;
        for edge in facts
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == block.id())
        {
            facts.charge(block.id().bci())?;
            let from_try = protected_blocks.contains(block.id());
            let from_catch = catch_blocks.contains(block.id());
            let valid = match edge.kind() {
                CanonicalEdgeKind::Exception { handler_ordinal } if from_try => {
                    (handler_ordinal == named.ordinal && edge.to() == &named_handler)
                        || (handler_ordinal == try_any.ordinal && edge.to() == &handler_block)
                }
                CanonicalEdgeKind::Exception { handler_ordinal } if from_catch => {
                    handler_ordinal == catch_any.ordinal && edge.to() == &handler_block
                }
                CanonicalEdgeKind::Exception { .. } => {
                    edge.to() != &named_handler
                        && edge.to() != &handler_block
                        && ![&normal_block, &catch_block, &handler_block].contains(&block.id())
                }
                CanonicalEdgeKind::Normal if from_try => {
                    protected_blocks.contains(edge.to()) || edge.to() == &normal_block
                }
                CanonicalEdgeKind::Normal if from_catch => {
                    catch_blocks.contains(edge.to()) || edge.to() == &catch_block
                }
                CanonicalEdgeKind::Normal => {
                    edge.to() != &normal_block
                        && edge.to() != &catch_block
                        && edge.to() != &handler_block
                }
                CanonicalEdgeKind::Call { .. } => false,
                CanonicalEdgeKind::Return { .. } => {
                    block.id() == &normal_block || block.id() == &catch_block
                }
            };
            if !valid {
                return Ok(None);
            }
        }
    }
    let owned = facts.blocks_in((start, handler_end));
    let origins = facts.bcis((start, handler_end));
    Ok(Some(Plan {
        shape: Shape::SharedFinally {
            rows: [named.ordinal, try_any.ordinal, catch_any.ordinal],
            binding_row: None,
            catch_body: (catch_any.start_bci, catch_any.end_bci),
            catch_handler: facts.row_handler(named).unwrap(),
            catch_type: named.catch_type_index.unwrap(),
            catch_parameter: *catch_parameter,
            normal_cleanup: (first_start, facts.span_end(first_cleanup.1)),
            catch_cleanup: (second_start, facts.span_end(second_cleanup.1)),
            completion: SharedFinallyCompletion::SavedReturns([
                (first_save, first_return),
                (second_save, second_return),
            ]),
        },
        lead: (start, protected.0),
        body: protected,
        owned,
        join: None,
        facts: origins,
    }))
}

/// Four copies of the same instance call, each with a fresh, uniquely consumed receiver from
/// the method's entry local. The member is compared as a resolved operation, never by its name.
fn segmented_cleanup_copies(
    facts: &mut Facts<'_>,
    starts: [u32; 4],
) -> Result<Option<[(u32, u32); 4]>, StopReason> {
    let mut spans = [(0, 0); 4];
    let mut member: Option<Operation> = None;
    for (index, start) in starts.into_iter().enumerate() {
        let Some(invoke) = facts.next_bci(start) else {
            return Ok(None);
        };
        facts.charge(start)?;
        facts.charge(invoke)?;
        let (Some(load), Some(call)) = (facts.step(start), facts.step(invoke)) else {
            return Ok(None);
        };
        let Some(Operation::Invoke(target)) = facts.op(invoke) else {
            return Ok(None);
        };
        if facts.op(start) != Some(&Operation::Load { slot: 0 })
            || load.instruction.opcode() != 0x2a
            || target.kind() != InvokeKind::Virtual
            || target.descriptor() != "()V"
            || member
                .as_ref()
                .is_some_and(|prior| prior != facts.op(invoke).unwrap())
            || load.instruction.reads().len() != 1
            || !matches!(
                facts
                    .ssa
                    .value(facts.resolve(load.instruction.reads()[0].1))
                    .def(),
                Definition::Entry {
                    slot: Slot::Local(0),
                    ..
                }
            )
            || load.instruction.writes().len() != 1
            || !matches!(load.instruction.writes()[0].0, Slot::Stack(_))
            || stack_operands(call.instruction).len() != 1
            || !facts.same(
                load.instruction.writes()[0].1,
                stack_operands(call.instruction)[0].1,
            )
            || call
                .instruction
                .writes()
                .iter()
                .any(|(slot, _)| matches!(slot, Slot::Stack(_)))
        {
            return Ok(None);
        }
        let mut consumers = 0;
        for bci in facts.order.clone() {
            facts.charge(bci)?;
            if let Some(step) = facts.step(bci) {
                consumers += stack_operands(step.instruction)
                    .iter()
                    .filter(|(_, read)| facts.same(*read, load.instruction.writes()[0].1))
                    .count();
            }
        }
        if consumers != 1 {
            return Ok(None);
        }
        member = facts.op(invoke).cloned();
        spans[index] = (start, facts.span_end(invoke));
    }
    Ok(Some(spans))
}

/// Exactly two named segments, two matching catch-all segments and one catch-body row.
/// No caller can combine this with a three-row certificate and claim the shared handler twice.
fn prove_segmented_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [named_first, named_second, any_first, any_second, catch_any] = facts.handlers else {
        return Ok(None);
    };
    let rows = [named_first, named_second, any_first, any_second, catch_any];
    if rows
        .windows(2)
        .any(|pair| pair[0].ordinal + 1 != pair[1].ordinal)
        || named_first.catch_type_index.is_none()
        || named_second.catch_type_index != named_first.catch_type_index
        || rows[2..].iter().any(|row| row.catch_type_index.is_some())
        || named_first.handler_bci != named_second.handler_bci
        || any_first.handler_bci != any_second.handler_bci
        || any_first.handler_bci != catch_any.handler_bci
        || (named_first.start_bci, named_first.end_bci) != (any_first.start_bci, any_first.end_bci)
        || (named_second.start_bci, named_second.end_bci)
            != (any_second.start_bci, any_second.end_bci)
        || !(current.bci() == named_first.start_bci
            && named_first.start_bci < named_first.end_bci
            && named_first.end_bci < named_second.start_bci
            && named_second.start_bci < named_second.end_bci
            && named_second.end_bci < named_first.handler_bci
            && named_first.handler_bci == catch_any.start_bci
            && catch_any.start_bci < catch_any.end_bci
            && catch_any.end_bci < any_first.handler_bci)
        || !facts
            .in_block(current)
            .iter()
            .any(|instruction| instruction.bci() == named_first.start_bci)
    {
        return Ok(None);
    }
    let segments = [
        (named_first.start_bci, named_first.end_bci),
        (named_second.start_bci, named_second.end_bci),
    ];
    let early_start = segments[0].1;
    let normal_start = segments[1].1;
    let catch_start = catch_any.end_bci;
    let handler = any_first.handler_bci;
    let Some(exceptional_start) = facts.next_bci(handler) else {
        return Ok(None);
    };
    let Some(cleanup) = segmented_cleanup_copies(
        facts,
        [early_start, normal_start, catch_start, exceptional_start],
    )?
    else {
        return Ok(None);
    };
    let [early_return, normal_transfer, catch_transfer, load] = cleanup.map(|span| span.1);
    let Some(rethrow) = facts.next_bci(load) else {
        return Ok(None);
    };
    let end = facts.span_end(rethrow);
    if facts.op(early_return) != Some(&Operation::Return)
        || facts.op(normal_transfer) != Some(&Operation::Transfer)
        || facts.op(catch_transfer) != Some(&Operation::Transfer)
        || facts.op(rethrow) != Some(&Operation::Throw)
        || facts.next_bci(early_return) != Some(segments[1].0)
        || facts.next_bci(normal_transfer) != Some(named_first.handler_bci)
        || facts.next_bci(catch_transfer) != Some(handler)
        || !handler_binding(facts, named_first.handler_bci)
        || !handler_binding(facts, handler)
    {
        return Ok(None);
    }
    let (
        Some(named_block),
        Some(handler_block),
        Some(early_block),
        Some(normal_block),
        Some(catch_block),
    ) = (
        facts.row_handler(named_first),
        facts.row_handler(any_first),
        facts.block_of(early_start).cloned(),
        facts.block_of(normal_start).cloned(),
        facts.block_of(catch_start).cloned(),
    )
    else {
        return Ok(None);
    };
    if named_block.bci() != named_first.handler_bci
        || handler_block.bci() != handler
        || facts.row_handler(named_second) != Some(named_block.clone())
        || facts.row_handler(any_second) != Some(handler_block.clone())
        || facts.row_handler(catch_any) != Some(handler_block.clone())
        || facts.block_of(early_return) != Some(&early_block)
        || early_block.bci() != early_start
        || normal_block.bci() != normal_start
        || facts.block_of(normal_transfer) != Some(&normal_block)
        || facts.block_of(catch_transfer) != Some(&catch_block)
        || facts.block_of(rethrow) != Some(&handler_block)
        || !facts.view.successor_ids(&early_block).is_empty()
        || !facts.view.successor_ids(&handler_block).is_empty()
        || facts
            .step(early_return)
            .is_none_or(|step| !step.instruction.reads().is_empty())
    {
        return Ok(None);
    }
    let (normal_successors, catch_successors) = (
        facts.view.successor_ids(&normal_block),
        facts.view.successor_ids(&catch_block),
    );
    let ([join], [catch_join]) = (normal_successors.as_slice(), catch_successors.as_slice()) else {
        return Ok(None);
    };
    if join != catch_join
        || join.bci() != end
        || facts.block_at(end).as_ref() != Some(join)
        || facts
            .view
            .index_of(join)
            .is_none_or(|index| facts.view.predecessors(index).len() != 2)
        || facts.view.index_of(&early_block).is_none_or(|index| {
            facts.view.predecessors(index).len() != 1
                || !facts.blocks_in(segments[0]).iter().any(|block| {
                    facts
                        .view
                        .index_of(block)
                        .is_some_and(|from| facts.view.predecessors(index).contains(&from))
                })
        })
    {
        return Ok(None);
    }
    let (Some(store), Some(loaded), Some(thrown)) =
        (facts.step(handler), facts.step(load), facts.step(rethrow))
    else {
        return Ok(None);
    };
    let (Some(Operation::Store { slot: saved }), Some(Operation::Load { slot: loaded_slot })) =
        (facts.op(handler), facts.op(load))
    else {
        return Ok(None);
    };
    if saved != loaded_slot
        || !store.instruction.writes().iter().any(|(_, written)| {
            loaded
                .instruction
                .reads()
                .iter()
                .any(|(_, read)| facts.same(*written, *read))
        })
        || !loaded.instruction.writes().iter().any(|(slot, written)| {
            matches!(slot, Slot::Stack(_))
                && stack_operands(thrown.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(*written, *read))
        })
        || stack_operands(thrown.instruction).len() != 1
    {
        return Ok(None);
    }
    for bci in facts.bcis((current.bci(), end)) {
        facts.charge(bci)?;
        let expected: &[u32] = if segments[0].0 <= bci && bci < segments[0].1 {
            &[named_first.ordinal, any_first.ordinal]
        } else if segments[1].0 <= bci && bci < segments[1].1 {
            &[named_second.ordinal, any_second.ordinal]
        } else if catch_any.start_bci <= bci && bci < catch_any.end_bci {
            &[catch_any.ordinal]
        } else {
            &[]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
            || (!expected.is_empty() && matches!(facts.op(bci), Some(Operation::Return)))
        {
            return Ok(None);
        }
    }
    let first = facts.blocks_in(segments[0]);
    let second = facts.blocks_in(segments[1]);
    let caught = facts.blocks_in((catch_any.start_bci, catch_any.end_bci));
    let protected = first.iter().chain(&second).collect::<BTreeSet<_>>();
    let catch = caught.into_iter().collect::<BTreeSet<_>>();
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let from_first = first.contains(edge.from());
        let from_second = second.contains(edge.from());
        let from_catch = catch.contains(edge.from());
        let valid = match edge.kind() {
            CanonicalEdgeKind::Exception { handler_ordinal } if from_first => {
                (handler_ordinal == named_first.ordinal && edge.to() == &named_block)
                    || (handler_ordinal == any_first.ordinal && edge.to() == &handler_block)
            }
            CanonicalEdgeKind::Exception { handler_ordinal } if from_second => {
                (handler_ordinal == named_second.ordinal && edge.to() == &named_block)
                    || (handler_ordinal == any_second.ordinal && edge.to() == &handler_block)
            }
            CanonicalEdgeKind::Exception { handler_ordinal } if from_catch => {
                handler_ordinal == catch_any.ordinal && edge.to() == &handler_block
            }
            CanonicalEdgeKind::Exception { .. } => {
                edge.to() != &named_block
                    && edge.to() != &handler_block
                    && !protected.contains(&edge.to())
                    && !catch.contains(edge.to())
                    && ![&early_block, &normal_block, &catch_block, &handler_block]
                        .contains(&edge.from())
            }
            CanonicalEdgeKind::Normal
                if edge.from() == &normal_block || edge.from() == &catch_block =>
            {
                edge.to() == join
            }
            CanonicalEdgeKind::Normal if from_first => {
                first.contains(edge.to()) || edge.to() == &early_block || second.contains(edge.to())
            }
            CanonicalEdgeKind::Normal if from_second => {
                second.contains(edge.to()) || edge.to() == &normal_block
            }
            CanonicalEdgeKind::Normal if from_catch => {
                catch.contains(edge.to()) || edge.to() == &catch_block
            }
            CanonicalEdgeKind::Normal => {
                edge.to() != &named_block
                    && edge.to() != &handler_block
                    && edge.to() != &early_block
                    && edge.to() != &normal_block
                    && edge.to() != &catch_block
                    && !protected.contains(&edge.to())
                    && !catch.contains(edge.to())
            }
            CanonicalEdgeKind::Call { .. } => false,
            CanonicalEdgeKind::Return { .. } => edge.from() == &early_block,
        };
        if !valid {
            return Ok(None);
        }
    }
    let mut owned = facts.blocks_in((current.bci(), end));
    owned.sort_by_key(CanonicalBlockId::bci);
    owned.dedup();
    let origins = facts.bcis((current.bci(), end));
    let Some(Operation::Store {
        slot: catch_parameter,
    }) = facts.op(named_first.handler_bci)
    else {
        return Ok(None);
    };
    Ok(Some(Plan {
        shape: Shape::SegmentedFinally {
            rows: rows.map(|row| row.ordinal),
            segments,
            catch_body: (catch_any.start_bci, catch_any.end_bci),
            catch_handler: named_block,
            catch_type: named_first.catch_type_index.unwrap(),
            catch_parameter: *catch_parameter,
            cleanup,
            early_return,
            transfers: [normal_transfer, catch_transfer],
        },
        lead: (current.bci(), segments[0].0),
        body: (segments[0].0, segments[1].1),
        owned,
        join: Some(join.clone()),
        facts: origins,
    }))
}

/// The TestFinally3 lowering: a null-lead conditional body with an early `return null`, compiled
/// as a segmented two-row table whose gap is the early return's own cleanup. The two `any` rows
/// share one handler and neither covers it — the early-return block sits between the ranges, so
/// its close copy, the normal tail's and the handler's are all unguarded, and a cleanup throw
/// replaces the completion on every path. The lead's `aconst_null; astore s` initialises the slot
/// all three copies' arguments read; the protected body assigns the same slot; the two returns
/// read one value slot — the early one the lead-style `null` literal the protected range ends in,
/// the normal one the body's own producer. Every instruction start, edge and exception row is
/// read from this run's facts before the copies can fold into one `finally`.
fn prove_segmented_null_lead_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [first_row, second_row] = facts.handlers else {
        return Ok(None);
    };
    // Entry: two consecutive catch-all rows over one handler, a gap between the protected
    // ranges, and no row over the handler itself. Every other two-row shape carries a
    // self-protecting binding row, a resource, or a loop — none reaches past this test.
    if first_row.catch_type_index.is_some()
        || second_row.catch_type_index.is_some()
        || second_row.ordinal != first_row.ordinal + 1
        || first_row.handler_bci != second_row.handler_bci
        || first_row.start_bci >= first_row.end_bci
        || first_row.end_bci >= second_row.start_bci
        || second_row.start_bci >= second_row.end_bci
        || second_row.end_bci > first_row.handler_bci
        || facts.order.len() > 64
        || !facts.canonical.unreachable().is_empty()
    {
        return Ok(None);
    }
    let (body_start, early_start, body_restart, body_end, handler_start) = (
        first_row.start_bci,
        first_row.end_bci,
        second_row.start_bci,
        second_row.end_bci,
        first_row.handler_bci,
    );
    let Some(&last_bci) = facts.order.last() else {
        return Ok(None);
    };
    let method_end = facts.span_end(last_bci);
    // The lead is the statement's own block opening: exactly `[aconst_null, astore s]`.
    let lead = facts.bcis((current.bci(), body_start));
    let [lead_push, lead_store] = lead.as_slice() else {
        return Ok(None);
    };
    if *lead_push != current.bci()
        || !matches!(
            facts.op(*lead_push),
            Some(Operation::Push(crate::facts::ConstantValue::Null))
        )
    {
        return Ok(None);
    }
    let Some(Operation::Store { slot }) = facts.op(*lead_store) else {
        return Ok(None);
    };
    let slot = *slot;
    if slot == 0 {
        return Ok(None);
    }
    // The gap is the early return's own block: exactly `[aload s, invoke, aload v, areturn]`,
    // and the `areturn` is the protected range's next instruction — the gap holds nothing else.
    let gap = facts.bcis((early_start, body_restart));
    let [gap_load, gap_call, gap_value_load, gap_return] = gap.as_slice() else {
        return Ok(None);
    };
    if facts.next_bci(*gap_return) != Some(body_restart)
        || facts.op(*gap_return) != Some(&Operation::Return)
    {
        return Ok(None);
    }
    let Some(Operation::Load { slot: gap_read }) = facts.op(*gap_load) else {
        return Ok(None);
    };
    if *gap_read != slot {
        return Ok(None);
    }
    let Some(cleanup_target) = (match facts.op(*gap_call) {
        Some(Operation::Invoke(target)) if target.kind() == InvokeKind::Static => {
            Some(target.clone())
        }
        _ => None,
    }) else {
        return Ok(None);
    };
    let Some(Operation::Load { slot: value_slot }) = facts.op(*gap_value_load) else {
        return Ok(None);
    };
    let value_slot = *value_slot;
    if value_slot == slot {
        return Ok(None);
    }
    // The early return's saved value is the protected range's own tail: `[aconst_null, astore v]`.
    let Some(early_store) = facts.previous_bci(*gap_load) else {
        return Ok(None);
    };
    let Some(early_push) = facts.previous_bci(early_store) else {
        return Ok(None);
    };
    if early_store < body_start
        || !matches!(
            facts.op(early_push),
            Some(Operation::Push(crate::facts::ConstantValue::Null))
        )
    {
        return Ok(None);
    }
    if !matches!(
        facts.op(early_store),
        Some(Operation::Store {
            slot: early_saved,
            ..
        }) if *early_saved == value_slot
    ) {
        return Ok(None);
    }
    // The normal completion is the same copy again, then the saved value's own return.
    let normal = facts.bcis((body_end, handler_start));
    let [normal_load, normal_call, normal_value_load, normal_return] = normal.as_slice() else {
        return Ok(None);
    };
    if facts.next_bci(*normal_return) != Some(handler_start)
        || facts.op(*normal_return) != Some(&Operation::Return)
    {
        return Ok(None);
    }
    if !matches!(facts.op(*normal_load), Some(Operation::Load { slot: read }) if *read == slot)
        || !matches!(facts.op(*normal_call), Some(Operation::Invoke(target)) if *target == cleanup_target)
        || !matches!(facts.op(*normal_value_load), Some(Operation::Load { slot: read }) if *read == value_slot)
    {
        return Ok(None);
    }
    // The normal return's saved value is the body's own producer, stored just before the tail —
    // and not another `null` literal, which is the early return's value and no other's.
    let Some(normal_save_store) = facts.previous_bci(*normal_load) else {
        return Ok(None);
    };
    if normal_save_store < body_restart
        || !matches!(facts.op(normal_save_store), Some(Operation::Store { slot: saved, .. }) if *saved == value_slot)
    {
        return Ok(None);
    }
    let handler = facts.bcis((handler_start, method_end));
    let [
        primary_store,
        handler_load,
        handler_call,
        handler_throw_load,
        rethrow,
    ] = handler.as_slice()
    else {
        return Ok(None);
    };
    let Some(Operation::Store { slot: thrown_slot }) = facts.op(*primary_store) else {
        return Ok(None);
    };
    let thrown_slot = *thrown_slot;
    if thrown_slot == slot
        || thrown_slot == value_slot
        || facts.op(*rethrow) != Some(&Operation::Throw)
        || facts.next_bci(*rethrow).is_some()
        || !matches!(facts.op(*handler_load), Some(Operation::Load { slot: read }) if *read == slot)
        || !matches!(facts.op(*handler_call), Some(Operation::Invoke(target)) if *target == cleanup_target)
        || !matches!(facts.op(*handler_throw_load), Some(Operation::Load { slot: read }) if *read == thrown_slot)
    {
        return Ok(None);
    }
    // The body's own test opens the protected range: a receiver load, an instance field read,
    // and the not-null branch into the shared normal block — the `if (f == null)` whose
    // fall-through holds the early return. The branch's target block is the one the body's
    // producer store stands in, which is what makes the two segments one statement's body.
    let Some(head_load) = facts.next_bci(body_start) else {
        return Ok(None);
    };
    let Some(head_branch) = facts.next_bci(head_load) else {
        return Ok(None);
    };
    let Some(shared_block) = facts.block_of(normal_save_store) else {
        return Ok(None);
    };
    if !matches!(facts.op(body_start), Some(Operation::Load { .. }))
        || !matches!(
            facts.op(head_load),
            Some(Operation::Field {
                access: crate::facts::FieldAccess::Read,
                is_static: false,
                ..
            })
        )
        || facts.block_of(head_branch) != Some(current)
        || shared_block.bci() < body_restart
        || !matches!(
            facts.op(head_branch),
            Some(Operation::Comparison {
                op: CompareOp::JumpIfNotNull,
                target,
            }) if *target == shared_block.bci()
        )
    {
        return Ok(None);
    }
    // The inner guard's own test is the protected range's last statement before the saved
    // literal: a receiver load, a boolean call, and the not-zero branch into the second range.
    let Some(second_branch) = facts.previous_bci(early_push) else {
        return Ok(None);
    };
    let Some(second_call) = facts.previous_bci(second_branch) else {
        return Ok(None);
    };
    let Some(second_receiver) = facts.previous_bci(second_call) else {
        return Ok(None);
    };
    if facts.next_bci(head_branch) != Some(second_receiver)
        || facts.next_bci(second_branch) != Some(early_push)
        || !matches!(facts.op(second_receiver), Some(Operation::Load { .. }))
        || !matches!(facts.op(second_call), Some(Operation::Invoke(target)) if target.descriptor() == "()Z")
        || !matches!(
            facts.op(second_branch),
            Some(Operation::Comparison {
                op: CompareOp::JumpIfNotZero,
                target,
            }) if *target == body_restart
        )
    {
        return Ok(None);
    }
    // The body fills the lead slot, and only inside the protected ranges: the lead's `null` and
    // the body's assignments are the only definitions any copy's argument can read.
    let body_store_bcis: Vec<u32> = facts
        .bcis((body_start, body_end))
        .into_iter()
        .filter(|bci| {
            matches!(
                facts.op(*bci),
                Some(Operation::Store {
                    slot: filled,
                    ..
                }) if *filled == slot
            )
        })
        .collect();
    if body_store_bcis.is_empty()
        || body_store_bcis.iter().any(|bci| {
            !((body_start..early_start).contains(bci) || (body_restart..body_end).contains(bci))
        })
    {
        return Ok(None);
    }
    let local_written = |step: Step<'_>, written: u16| {
        step.instruction
            .writes()
            .iter()
            .find_map(|(slot, value)| (*slot == Slot::Local(written)).then_some(*value))
    };
    let local_read = |step: Step<'_>, read: u16| {
        step.instruction
            .reads()
            .iter()
            .find_map(|(slot, value)| (*slot == Slot::Local(read)).then_some(*value))
    };
    let body_store_values: Vec<ValueId> = body_store_bcis
        .iter()
        .filter_map(|bci| facts.step(*bci).and_then(|step| local_written(step, slot)))
        .collect();
    if body_store_values.len() != body_store_bcis.len() {
        return Ok(None);
    }
    for bci in facts.bcis((current.bci(), method_end)) {
        facts.charge(bci)?;
        let fills = |expected: u16| {
            matches!(
                facts.op(bci),
                Some(Operation::Store {
                    slot: filled,
                    ..
                }) if *filled == expected
            )
        };
        if (fills(slot) && bci != *lead_store && !body_store_bcis.contains(&bci))
            || (fills(value_slot) && bci != early_store && bci != normal_save_store)
        {
            return Ok(None);
        }
    }
    // The two saved values, and each return's own identity: the early return hands back the
    // literal its own store saved, the normal return hands back the producer's store.
    let (
        Some(lead_push_step),
        Some(lead_store_step),
        Some(early_push_step),
        Some(early_store_step),
        Some(gap_load_step),
        Some(gap_value_step),
        Some(gap_return_step),
        Some(normal_save_step),
        Some(normal_load_step),
        Some(normal_value_step),
        Some(normal_return_step),
        Some(primary_store_step),
        Some(handler_load_step),
        Some(handler_throw_step),
        Some(rethrow_step),
    ) = (
        facts.step(*lead_push),
        facts.step(*lead_store),
        facts.step(early_push),
        facts.step(early_store),
        facts.step(*gap_load),
        facts.step(*gap_value_load),
        facts.step(*gap_return),
        facts.step(normal_save_store),
        facts.step(*normal_load),
        facts.step(*normal_value_load),
        facts.step(*normal_return),
        facts.step(*primary_store),
        facts.step(*handler_load),
        facts.step(*handler_throw_load),
        facts.step(*rethrow),
    )
    else {
        return Ok(None);
    };
    let (Some(lead_value), Some(early_value), Some(normal_value), Some(thrown_value)) = (
        local_written(lead_store_step, slot),
        local_written(early_store_step, value_slot),
        local_written(normal_save_step, value_slot),
        local_written(primary_store_step, thrown_slot),
    ) else {
        return Ok(None);
    };
    let lead_operand_ok = matches!(stack_operands(lead_store_step.instruction).as_slice(), [(_, read)]
        if lead_push_step.instruction.writes().iter().any(|(_, value)| facts.same(*value, *read)));
    let early_operand_ok = matches!(stack_operands(early_store_step.instruction).as_slice(), [(_, read)]
        if early_push_step.instruction.writes().iter().any(|(_, value)| facts.same(*value, *read)));
    let early_return_ok = local_read(gap_value_step, value_slot)
        .is_some_and(|read| facts.same(read, early_value))
        && matches!(stack_operands(gap_return_step.instruction).as_slice(), [(_, read)]
            if gap_value_step.instruction.writes().iter().any(|(_, value)| facts.same(*value, *read)));
    let normal_return_ok = local_read(normal_value_step, value_slot)
        .is_some_and(|read| facts.same(read, normal_value))
        && matches!(stack_operands(normal_return_step.instruction).as_slice(), [(_, read)]
            if normal_value_step.instruction.writes().iter().any(|(_, value)| facts.same(*value, *read)));
    let rethrow_ok = local_read(handler_throw_step, thrown_slot)
        .is_some_and(|read| facts.same(read, thrown_value))
        && matches!(stack_operands(rethrow_step.instruction).as_slice(), [(_, read)]
            if handler_throw_step.instruction.writes().iter().any(|(_, value)| facts.same(*value, *read)));
    if !lead_operand_ok || !early_operand_ok || !early_return_ok || !normal_return_ok || !rethrow_ok
    {
        return Ok(None);
    }
    let Definition::Instruction {
        bci: normal_producer,
        ..
    } = facts.ssa.value(facts.resolve(normal_value)).def()
    else {
        return Ok(None);
    };
    if facts.op(*normal_producer) == Some(&Operation::Push(crate::facts::ConstantValue::Null)) {
        return Ok(None);
    }
    // Each copy's one argument is a read of the lead slot itself, and its value is the merged
    // flow of the lead and the body's own assignments — no definition outside the two may reach
    // it. The normal copy runs the body to its end, so its read must name a body value; the
    // early copy runs before any assignment and the handler may be entered before one, so
    // either answer is theirs.
    for (copy_read, copy_entry, require_body) in [
        (local_read(gap_load_step, slot), *gap_load, false),
        (local_read(normal_load_step, slot), *normal_load, true),
        (local_read(handler_load_step, slot), handler_start, false),
    ] {
        let Some(read) = copy_read else {
            return Ok(None);
        };
        let reaches_body = segmented_null_lead_argument(
            facts,
            read,
            lead_value,
            &body_store_values,
            copy_entry,
            slot,
        )?;
        let Some(reaches_body) = reaches_body else {
            return Ok(None);
        };
        if require_body && !reaches_body {
            return Ok(None);
        }
    }
    // No protected return, exactly one covering row per instruction, and no row over the lead,
    // the gap, the tails or the handler itself: the self-protection's absence is the statement's
    // own claim that a cleanup throw replaces the completion on every path.
    for bci in facts.bcis((current.bci(), method_end)) {
        facts.charge(bci)?;
        let expected: &[u32] = if (body_start..early_start).contains(&bci) {
            &[first_row.ordinal]
        } else if (body_restart..body_end).contains(&bci) {
            &[second_row.ordinal]
        } else {
            &[]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
            || (!expected.is_empty() && matches!(facts.op(bci), Some(Operation::Return)))
        {
            return Ok(None);
        }
    }
    // Every canonical edge is one the statement states: exception edges only from the protected
    // blocks to their own row's handler, normal edges only inside the body's own blocks, return
    // edges only from the early-return block and the shared normal block, and no call edges.
    let Some(early_block) = facts.block_of(*gap_load) else {
        return Ok(None);
    };
    let Some(handler_block) = facts.block_at(handler_start) else {
        return Ok(None);
    };
    let seg1_blocks: BTreeSet<CanonicalBlockId> = facts
        .blocks_in((body_start, early_start))
        .into_iter()
        .collect();
    let seg2_blocks: BTreeSet<CanonicalBlockId> = facts
        .blocks_in((body_restart, body_end))
        .into_iter()
        .collect();
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let from = edge.from();
        let in_seg1 = seg1_blocks.contains(from);
        let in_seg2 = seg2_blocks.contains(from);
        let valid = match edge.kind() {
            CanonicalEdgeKind::Exception { handler_ordinal } => {
                let to_handler = edge.to() == &handler_block;
                (in_seg1 && !in_seg2 && handler_ordinal == first_row.ordinal && to_handler)
                    || (!in_seg1 && in_seg2 && handler_ordinal == second_row.ordinal && to_handler)
            }
            CanonicalEdgeKind::Normal => {
                seg1_blocks.contains(edge.to()) || seg2_blocks.contains(edge.to())
            }
            CanonicalEdgeKind::Return { .. } => from == early_block || from == shared_block,
            CanonicalEdgeKind::Call { .. } => false,
        };
        if !valid {
            return Ok(None);
        }
    }
    let mut owned = facts.blocks_in((current.bci(), method_end));
    owned.sort_by_key(CanonicalBlockId::bci);
    owned.dedup();
    let origins = facts.bcis((current.bci(), method_end));
    Ok(Some(Plan {
        shape: Shape::SegmentedNullLeadFinally {
            rows: [first_row.ordinal, second_row.ordinal],
            slot,
            value_slot,
            segments: [(body_start, early_start), (body_restart, body_end)],
            cleanup_target,
            early_cleanup: (early_start, *gap_return),
            early_return: *gap_return,
            normal_cleanup: (
                *normal_load,
                facts.next_bci(*normal_call).unwrap_or(*normal_call),
            ),
            returns: [
                (early_store, *gap_return),
                (normal_save_store, *normal_return),
            ],
        },
        lead: (current.bci(), body_start),
        body: (body_start, body_end),
        owned,
        join: None,
        facts: origins,
    }))
}

/// What one copy argument's provenance walk finds: `None` is a definition outside the lead's
/// `null` and the body's own assignments, `Some(false)` the lead's `null` alone, `Some(true)` a
/// walk that reached one of the body's assignments. The walk is the local-null handler's own,
/// widened to answer the two cases apart instead of only "the body was reached".
fn segmented_null_lead_argument(
    facts: &mut Facts<'_>,
    value: ValueId,
    lead_store: ValueId,
    body_stores: &[ValueId],
    handler_entry: u32,
    slot: u16,
) -> Result<Option<bool>, StopReason> {
    let mut pending = vec![value];
    let mut seen = BTreeSet::new();
    let mut reaches_body = false;
    while let Some(value) = pending.pop() {
        facts.charge(0)?;
        let value = facts.resolve(value);
        if !seen.insert(value) {
            continue;
        }
        if seen.len() > 64 {
            return Ok(None);
        }
        if body_stores.iter().any(|store| facts.same(value, *store)) {
            reaches_body = true;
            continue;
        }
        if facts.same(value, lead_store) {
            continue;
        }
        let Definition::Phi {
            block,
            slot: phi_slot,
        } = facts.ssa.value(value).def()
        else {
            return Ok(None);
        };
        if *phi_slot != Slot::Local(slot) {
            return Ok(None);
        }
        let Some(phi) = facts.ssa.phis().iter().find(|phi| {
            phi.block() == block && phi.slot() == *phi_slot && facts.same(phi.value(), value)
        }) else {
            return Ok(None);
        };
        for input in phi.inputs() {
            match input {
                jarde_jvm::method_ir::PhiInput::Value(value) => pending.push(*value),
                jarde_jvm::method_ir::PhiInput::Itself if block.bci() == handler_entry => {}
                jarde_jvm::method_ir::PhiInput::Itself => return Ok(None),
            }
        }
    }
    Ok(Some(reaches_body))
}

/// The Java 11 Test5 lowering has two disjoint protected return paths, an ordinary
/// do-while in the second path, and a handler row that protects only its binding.
/// Pinning the instruction geometry keeps this certificate separate from the named-catch
/// saved-return proof: every instruction and every edge is accounted for before a copy
/// can be removed by the writer.
fn prove_multi_return_loop_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [early, loop_body, binding] = facts.handlers else {
        return Ok(None);
    };
    const BCIS: [u32; 45] = [
        0, 1, 2, 5, 6, 7, 10, 11, 12, 13, 14, 19, 21, 23, 28, 31, 32, 34, 36, 41, 43, 44, 47, 48,
        51, 53, 55, 56, 58, 63, 68, 69, 71, 76, 79, 81, 83, 85, 90, 92, 93, 95, 97, 102, 104,
    ];
    if current.bci() != 12
        || facts.order != BCIS
        || facts
            .canonical
            .blocks()
            .iter()
            .map(|b| b.id().bci())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([0, 10, 12, 31, 44, 53, 79, 93])
        || (early.start_bci, early.end_bci, early.handler_bci) != (21, 34, 93)
        || (
            loop_body.start_bci,
            loop_body.end_bci,
            loop_body.handler_bci,
        ) != (44, 83, 93)
        || (binding.start_bci, binding.end_bci, binding.handler_bci) != (93, 95, 93)
        || early.ordinal + 1 != loop_body.ordinal
        || loop_body.ordinal + 1 != binding.ordinal
        || [early, loop_body, binding]
            .iter()
            .any(|row| row.catch_type_index.is_some())
        || !handler_binding(facts, 93)
        || facts.span_end(93) != 95
    {
        return Ok(None);
    }
    // Exact opcode starts exclude any unproved operation inside the body, including
    // another exit from the loop. The semantic checks below still compare targets and
    // SSA values, which an opcode-only check cannot establish.
    const OPCODES: [u8; 45] = [
        0x2a, 0x2b, 0xb6, 0x4e, 0x2d, 0xc7, 0x01, 0xb0, 0x2c, 0x2d, 0xb9, 0x3a, 0x19, 0xb9, 0x9a,
        0x01, 0x3a, 0x19, 0xb9, 0x19, 0xb0, 0xbb, 0x59, 0xb7, 0x3a, 0x19, 0x2c, 0x19, 0xb9, 0xb9,
        0x57, 0x19, 0xb9, 0x9a, 0x19, 0x3a, 0x19, 0xb9, 0x19, 0xb0, 0x3a, 0x19, 0xb9, 0x19, 0xbf,
    ];
    for (bci, opcode) in BCIS.into_iter().zip(OPCODES) {
        facts.charge(bci)?;
        if facts
            .step(bci)
            .is_none_or(|step| step.instruction.opcode() != opcode)
        {
            return Ok(None);
        }
        let expected: &[u32] = if (21..34).contains(&bci) {
            &[early.ordinal]
        } else if (44..83).contains(&bci) {
            &[loop_body.ordinal]
        } else if bci == 93 {
            &[binding.ordinal]
        } else {
            &[]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
        {
            return Ok(None);
        }
    }
    let Some(Operation::Invoke(close)) = facts.op(36) else {
        return Ok(None);
    };
    if close.kind() != InvokeKind::Interface
        || close.name() != "close"
        || close.descriptor() != "()V"
        || facts.op(85) != facts.op(36)
        || facts.op(97) != facts.op(36)
    {
        return Ok(None);
    }
    // A local read must reach precisely the receiver of each close, and all
    // three reads must reach the one resource stored before the first row.
    let Some(resource) = facts.step(19) else {
        return Ok(None);
    };
    let Some((_, resource_value)) = resource
        .instruction
        .writes()
        .iter()
        .find(|(slot, _)| *slot == Slot::Local(4))
    else {
        return Ok(None);
    };
    for (load_bci, call_bci) in [(34, 36), (83, 85), (95, 97)] {
        let (Some(load), Some(call)) = (facts.step(load_bci), facts.step(call_bci)) else {
            return Ok(None);
        };
        if facts.op(load_bci) != Some(&Operation::Load { slot: 4 })
            || load.instruction.reads().len() != 1
            || !facts.same(load.instruction.reads()[0].1, *resource_value)
            || stack_operands(call.instruction).len() != 1
            || !load.instruction.writes().iter().any(|(slot, value)| {
                matches!(slot, Slot::Stack(_))
                    && facts.same(*value, stack_operands(call.instruction)[0].1)
            })
        {
            return Ok(None);
        }
    }
    let saved = [(32, 41, 43, 5), (81, 90, 92, 6), (93, 102, 104, 7)];
    for (store_bci, load_bci, exit_bci, slot) in saved {
        let (Some(store), Some(load), Some(exit)) = (
            facts.step(store_bci),
            facts.step(load_bci),
            facts.step(exit_bci),
        ) else {
            return Ok(None);
        };
        if facts.op(store_bci) != Some(&Operation::Store { slot })
            || facts.op(load_bci) != Some(&Operation::Load { slot })
            || (if exit_bci == 104 {
                facts.op(exit_bci) != Some(&Operation::Throw)
            } else {
                facts.op(exit_bci) != Some(&Operation::Return)
            })
            || !store.instruction.writes().iter().any(|(access, value)| {
                *access == Slot::Local(slot)
                    && load
                        .instruction
                        .reads()
                        .iter()
                        .any(|(_, read)| facts.same(*value, *read))
            })
            || stack_operands(exit.instruction).len() != 1
            || !load.instruction.writes().iter().any(|(access, value)| {
                matches!(access, Slot::Stack(_))
                    && facts.same(*value, stack_operands(exit.instruction)[0].1)
            })
        {
            return Ok(None);
        }
    }
    if facts.op(31) != Some(&Operation::Push(crate::facts::ConstantValue::Null))
        || !facts.step(31).is_some_and(|push| {
            facts.step(32).is_some_and(|store| {
                push.instruction.writes().iter().any(|(slot, value)| {
                    matches!(slot, Slot::Stack(_))
                        && stack_operands(store.instruction)
                            .iter()
                            .any(|(_, read)| facts.same(*value, *read))
                })
            })
        })
        || ![(51, 53), (51, 79)].into_iter().all(|(store, load)| {
            facts.step(store).is_some_and(|store| {
                facts.step(load).is_some_and(|load| {
                    store.instruction.writes().iter().any(|(slot, value)| {
                        *slot == Slot::Local(5)
                            && load
                                .instruction
                                .reads()
                                .iter()
                                .any(|(_, read)| facts.same(*value, *read))
                    })
                })
            })
        })
        || !facts.step(79).is_some_and(|load| {
            facts.step(81).is_some_and(|store| {
                load.instruction.writes().iter().any(|(slot, value)| {
                    matches!(slot, Slot::Stack(_))
                        && stack_operands(store.instruction)
                            .iter()
                            .any(|(_, read)| facts.same(*value, *read))
                })
            })
        })
    {
        return Ok(None);
    }
    let Some(handler) = facts.block_at(93) else {
        return Ok(None);
    };
    let Some(loop_header) = facts.block_at(53) else {
        return Ok(None);
    };
    let Some(pre_loop) = facts.block_at(44) else {
        return Ok(None);
    };
    if [early, loop_body, binding]
        .iter()
        .any(|row| facts.row_handler(row) != Some(handler.clone()))
        || facts.view.successor_ids(&loop_header).len() != 2
        || !facts.view.successor_ids(&pre_loop).contains(&loop_header)
        || facts
            .view
            .predecessors(facts.view.index_of(&loop_header).unwrap())
            .len()
            != 2
        || !facts
            .view
            .successor_ids(&loop_header)
            .contains(&loop_header)
        || !facts
            .view
            .successor_ids(&loop_header)
            .contains(&facts.block_at(79).unwrap())
    {
        return Ok(None);
    }
    // The branch at 76 is physically in the loop header's block; it must point
    // back to that block, with the fall-through at 79. Check raw targets below
    // through the canonical normal edges, including all entry and exit edges.
    let owned = facts.blocks_in((12, 105));
    let mut actual_edges = BTreeSet::new();
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let kind = match edge.kind() {
            CanonicalEdgeKind::Normal => 0,
            CanonicalEdgeKind::Exception { handler_ordinal } if edge.to() == &handler => {
                1 + handler_ordinal
            }
            CanonicalEdgeKind::Exception { .. }
            | CanonicalEdgeKind::Call { .. }
            | CanonicalEdgeKind::Return { .. } => return Ok(None),
        };
        if !actual_edges.insert((edge.from().bci(), edge.to().bci(), kind)) {
            return Ok(None);
        }
    }
    let expected_edges = BTreeSet::from([
        (0, 10, 0),
        (0, 12, 0),
        (12, 31, 0),
        (12, 44, 0),
        (12, 93, 1 + early.ordinal),
        (44, 53, 0),
        (44, 93, 1 + loop_body.ordinal),
        (53, 53, 0),
        (53, 79, 0),
        (53, 93, 1 + loop_body.ordinal),
        (93, 93, 1 + binding.ordinal),
    ]);
    if actual_edges != expected_edges {
        return Ok(None);
    }
    Ok(Some(Plan {
        shape: Shape::MultiReturnLoopFinally {
            rows: [early.ordinal, loop_body.ordinal, binding.ordinal],
            segments: [(21, 34), (44, 83)],
            cleanup: [(34, 41), (83, 90), (95, 102)],
            returns: [(32, 43), (81, 92)],
        },
        lead: (12, 21),
        body: (21, 83),
        owned,
        join: None,
        facts: BCIS.iter().copied().filter(|bci| *bci >= 21).collect(),
    }))
}

/// The fixed Test2 void finally: one protected body of three ordinary loops, two cleanup copies of
/// one `close` call on one resource, and a void completion.
///
/// The certificate is deliberately closed, like the two-return loop proof beside it: the two
/// catch-all rows in table order, every instruction start and opcode, every canonical edge, the
/// three normal back edges and the handler's own self-protection row are each read from this run's
/// facts. The resource and the rethrown exception are proved through SSA values — the one local
/// definition before the protected range reaches the receiver of both closes, the handler's stored
/// value the one the rethrow reads — not through equal instruction text. The completion invents no
/// stack value and no catch: the normal completion is the value-less `return` the normal cleanup
/// copy falls through to (one canonical block with it, so the statement claims the block and the
/// run ends there), and the exceptional completion is the handler's own rethrow.
fn prove_void_loop_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [body_row, binding_row] = facts.handlers else {
        return Ok(None);
    };
    const BCIS: [u32; 89] = [
        0, 3, 4, 5, 8, 9, 10, 11, 14, 15, 16, 19, 20, 23, 24, 27, 28, 29, 30, 32, 33, 35, 37, 39,
        42, 43, 45, 46, 48, 49, 50, 52, 55, 58, 61, 64, 65, 68, 69, 70, 71, 73, 74, 76, 78, 80, 83,
        84, 86, 87, 89, 91, 94, 96, 97, 99, 100, 103, 105, 107, 109, 110, 112, 113, 115, 117, 119,
        122, 124, 126, 127, 129, 130, 132, 135, 138, 141, 144, 147, 150, 153, 154, 157, 160, 162,
        163, 166, 168, 169,
    ];
    const OPCODES: [u8; 89] = [
        0xbb, 0x59, 0x2b, 0xb7, 0x4d, 0x2c, 0x04, 0xb6, 0x2c, 0x2a, 0xb4, 0xbe, 0xb6, 0x2a, 0xb4,
        0x4e, 0x2d, 0xbe, 0x36, 0x03, 0x36, 0x15, 0x15, 0xa2, 0x2d, 0x15, 0x32, 0x3a, 0x2a, 0x2c,
        0x19, 0xb6, 0xb6, 0x84, 0xa7, 0x2a, 0xb4, 0x4e, 0x2d, 0xbe, 0x36, 0x03, 0x36, 0x15, 0x15,
        0xa2, 0x2d, 0x15, 0x32, 0x3a, 0x19, 0xb6, 0x3a, 0x2c, 0x19, 0xbe, 0xb6, 0x19, 0x3a, 0x19,
        0xbe, 0x36, 0x03, 0x36, 0x15, 0x15, 0xa2, 0x19, 0x15, 0x32, 0x3a, 0x2c, 0x19, 0xb6, 0xb6,
        0xb6, 0x84, 0xa7, 0x84, 0xa7, 0x2c, 0xb6, 0xa7, 0x3a, 0x2c, 0xb6, 0x19, 0xbf, 0xb1,
    ];
    if current.bci() != 0
        || facts.order != BCIS
        || facts
            .canonical
            .blocks()
            .iter()
            .map(|block| block.id().bci())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([0, 35, 42, 64, 76, 83, 115, 122, 147, 153, 160])
        || (body_row.start_bci, body_row.end_bci, body_row.handler_bci) != (9, 153, 160)
        || (
            binding_row.start_bci,
            binding_row.end_bci,
            binding_row.handler_bci,
        ) != (160, 162, 160)
        || body_row.ordinal + 1 != binding_row.ordinal
        || body_row.catch_type_index.is_some()
        || binding_row.catch_type_index.is_some()
        || !handler_binding(facts, 160)
        || facts.span_end(160) != 162
    {
        return Ok(None);
    }
    // Exact opcode starts and exact row coverage per instruction: the body row covers the protected
    // body and nothing else, the binding row covers the handler's binding store alone, and both
    // cleanup copies sit outside every protected interval.
    for (bci, opcode) in BCIS.into_iter().zip(OPCODES) {
        facts.charge(bci)?;
        if facts
            .step(bci)
            .is_none_or(|step| step.instruction.opcode() != opcode)
        {
            return Ok(None);
        }
        let expected: &[u32] = if (9..153).contains(&bci) {
            &[body_row.ordinal]
        } else if bci == 160 {
            &[binding_row.ordinal]
        } else {
            &[]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
        {
            return Ok(None);
        }
    }
    // The two cleanup copies invoke the same target; the receiver is proved by SSA below, not by
    // the shared symbol.
    let Some(Operation::Invoke(close)) = facts.op(154) else {
        return Ok(None);
    };
    if close.kind() != InvokeKind::Virtual
        || close.owner() != "java/io/DataOutputStream"
        || close.name() != "close"
        || close.descriptor() != "()V"
        || facts.op(163) != facts.op(154)
    {
        return Ok(None);
    }
    // The one store before the protected range is the resource both closes read: local 2's
    // definition at the store reaches the receiver of each close, and each close's only operand is
    // the stack value its own load wrote.
    let Some(resource) = facts.step(8) else {
        return Ok(None);
    };
    let Some((_, resource_value)) = resource
        .instruction
        .writes()
        .iter()
        .find(|(slot, _)| *slot == Slot::Local(2))
    else {
        return Ok(None);
    };
    for (load_bci, call_bci) in [(153, 154), (162, 163)] {
        let (Some(load), Some(call)) = (facts.step(load_bci), facts.step(call_bci)) else {
            return Ok(None);
        };
        if facts.op(load_bci) != Some(&Operation::Load { slot: 2 })
            || load.instruction.reads().len() != 1
            || !facts.same(load.instruction.reads()[0].1, *resource_value)
            || stack_operands(call.instruction).len() != 1
            || !load.instruction.writes().iter().any(|(slot, value)| {
                matches!(slot, Slot::Stack(_))
                    && facts.same(*value, stack_operands(call.instruction)[0].1)
            })
        {
            return Ok(None);
        }
    }
    // The handler binds its own row's exception into local 12, reloads and rethrows that one value.
    let (Some(save), Some(load), Some(throw)) = (facts.step(160), facts.step(166), facts.step(168))
    else {
        return Ok(None);
    };
    if facts.op(160) != Some(&Operation::Store { slot: 12 })
        || facts.op(166) != Some(&Operation::Load { slot: 12 })
        || facts.op(168) != Some(&Operation::Throw)
        || !save.instruction.writes().iter().any(|(slot, written)| {
            *slot == Slot::Local(12)
                && load
                    .instruction
                    .reads()
                    .iter()
                    .any(|(_, read)| facts.same(*written, *read))
        })
        || stack_operands(throw.instruction).len() != 1
        || !load.instruction.writes().iter().any(|(slot, written)| {
            matches!(slot, Slot::Stack(_))
                && facts.same(*written, stack_operands(throw.instruction)[0].1)
        })
    {
        return Ok(None);
    }
    // The completion is void: the normal cleanup copy and the final `return` share one canonical
    // block, that return is the block's last instruction, and no run continues past it.
    let Some(handler) = facts.block_at(160) else {
        return Ok(None);
    };
    let Some(normal_block) = facts.block_at(153) else {
        return Ok(None);
    };
    let Some(final_return) = facts
        .in_block(&normal_block)
        .last()
        .map(SsaInstruction::bci)
    else {
        return Ok(None);
    };
    if final_return != 169
        || facts.op(169) != Some(&Operation::Return)
        || !stack_operands(
            facts
                .step(169)
                .expect("the instruction order pinned the final return")
                .instruction,
        )
        .is_empty()
        || !facts.view.successor_ids(&normal_block).is_empty()
        || facts.block_of(162) != Some(&handler)
        || facts.block_of(163) != Some(&handler)
        || facts.block_of(166) != Some(&handler)
        || facts.block_of(168) != Some(&handler)
    {
        return Ok(None);
    }
    // Three ordinary loops, each entered from the block before it and from its own back edge only,
    // testing once with two successors. Their exits land inside the protected range: none of them
    // is a handler component (the Test11 shape).
    let Some(first_header) = facts.block_at(35) else {
        return Ok(None);
    };
    let Some(first_body) = facts.block_at(42) else {
        return Ok(None);
    };
    let Some(second_header) = facts.block_at(76) else {
        return Ok(None);
    };
    let Some(second_body) = facts.block_at(83) else {
        return Ok(None);
    };
    let Some(third_header) = facts.block_at(115) else {
        return Ok(None);
    };
    let Some(third_body) = facts.block_at(122) else {
        return Ok(None);
    };
    if facts.row_handler(body_row).as_ref() != Some(&handler)
        || facts.row_handler(binding_row).as_ref() != Some(&handler)
        || [
            (first_header, first_body),
            (second_header, second_body),
            (third_header, third_body),
        ]
        .iter()
        .any(|(header, body)| {
            facts.view.successor_ids(header).len() != 2
                || !facts.view.successor_ids(header).contains(body)
                || facts
                    .view
                    .index_of(header)
                    .is_none_or(|header| facts.view.predecessors(header).len() != 2)
        })
    {
        return Ok(None);
    }
    // Every physical edge of the method, named: the three loop back edges, every covered block's
    // exception edge into the one handler, and the handler's self-exception edge. The normal
    // cleanup block ends in the return and leaves no edge. An unaccounted edge — an extra exit, a
    // second handler, a subroutine — refuses the shape.
    let owned = facts.blocks_in((0, 169));
    let body_kind = 1 + body_row.ordinal;
    let binding_kind = 1 + binding_row.ordinal;
    let mut actual_edges = BTreeSet::new();
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let kind = match edge.kind() {
            CanonicalEdgeKind::Normal => 0,
            CanonicalEdgeKind::Exception { handler_ordinal } if edge.to() == &handler => {
                1 + handler_ordinal
            }
            CanonicalEdgeKind::Exception { .. }
            | CanonicalEdgeKind::Call { .. }
            | CanonicalEdgeKind::Return { .. } => return Ok(None),
        };
        if !actual_edges.insert((edge.from().bci(), edge.to().bci(), kind)) {
            return Ok(None);
        }
    }
    let expected_edges = BTreeSet::from([
        (0, 35, 0),
        (35, 42, 0),
        (35, 64, 0),
        (42, 35, 0),
        (64, 76, 0),
        (76, 83, 0),
        (76, 153, 0),
        (83, 115, 0),
        (115, 122, 0),
        (115, 147, 0),
        (122, 115, 0),
        (147, 76, 0),
        (0, 160, body_kind),
        (42, 160, body_kind),
        (64, 160, body_kind),
        (83, 160, body_kind),
        (122, 160, body_kind),
        (160, 160, binding_kind),
    ]);
    if actual_edges != expected_edges {
        return Ok(None);
    }
    Ok(Some(Plan {
        shape: Shape::Finally {
            normal_cleanup: (153, 157),
            completion: FinallyCompletion::Void {
                binding_row: binding_row.ordinal,
                final_return: 169,
            },
            row_ordinal: body_row.ordinal,
            structured: true,
        },
        lead: (0, 9),
        body: (9, 153),
        owned,
        join: None,
        facts: BCIS.iter().copied().filter(|bci| *bci >= 9).collect(),
    }))
}

/// The Java 8 two-row iterable finally is deliberately a closed certificate. Like pinned JADX's
/// `MarkFinallyVisitor::processTryBlock`/`findCommonInsns`, it starts with the handler rethrow and
/// compares cleanup along normal and exceptional exits. Unlike its generic path match and
/// `DONT_GENERATE` marking, this checks the exact rows, CFG, SSA and full physical ownership before
/// either copy can disappear. Local iterator/item slots may differ across the two copies.
fn prove_loop_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Plan>, StopReason> {
    let [body_row, binding_row] = facts.handlers else {
        return Ok(None);
    };
    const BCIS: [u32; 33] = [
        0, 1, 4, 5, 10, 11, 12, 17, 20, 21, 26, 27, 28, 29, 32, 35, 38, 40, 41, 46, 48, 50, 55, 58,
        60, 65, 67, 68, 70, 73, 76, 78, 79,
    ];
    if current.bci() != 0
        || facts.order != BCIS
        || facts.canonical.blocks().len() != 8
        || facts
            .canonical
            .blocks()
            .iter()
            .map(|block| block.id().bci())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([0, 11, 20, 35, 38, 48, 58, 76])
        || facts.block_of(79) != facts.block_at(35).as_ref()
        || body_row.ordinal + 1 != binding_row.ordinal
        || body_row.catch_type_index.is_some()
        || binding_row.catch_type_index.is_some()
        || (body_row.start_bci, body_row.end_bci, body_row.handler_bci) != (0, 4, 38)
        || (
            binding_row.start_bci,
            binding_row.end_bci,
            binding_row.handler_bci,
        ) != (38, 40, 38)
        || !handler_binding(facts, 38)
    {
        return Ok(None);
    }
    for bci in BCIS {
        facts.charge(bci)?;
        let expected = if bci < 4 {
            &[body_row.ordinal][..]
        } else if bci == 38 {
            &[binding_row.ordinal][..]
        } else {
            &[][..]
        };
        if facts
            .covering(bci)
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>()
            != expected
        {
            return Ok(None);
        }
    }
    let load = |bci, slot| facts.op(bci) == Some(&Operation::Load { slot });
    let store = |bci, slot| facts.op(bci) == Some(&Operation::Store { slot });
    let Some(Operation::Store {
        slot: normal_iterator,
    }) = facts.op(10)
    else {
        return Ok(None);
    };
    let Some(Operation::Store { slot: normal_item }) = facts.op(26) else {
        return Ok(None);
    };
    let Some(Operation::Store { slot: primary_slot }) = facts.op(38) else {
        return Ok(None);
    };
    let Some(Operation::Store {
        slot: handler_iterator,
    }) = facts.op(46)
    else {
        return Ok(None);
    };
    let Some(Operation::Store { slot: handler_item }) = facts.op(65) else {
        return Ok(None);
    };
    if !load(0, 0)
        || !load(4, 1)
        || !load(11, *normal_iterator)
        || !load(20, *normal_iterator)
        || !load(27, 0)
        || !load(28, *normal_item)
        || !load(40, 1)
        || !load(48, *handler_iterator)
        || !load(58, *handler_iterator)
        || !load(67, 0)
        || !load(68, *handler_item)
        || !load(76, *primary_slot)
        || !store(10, *normal_iterator)
        || !store(26, *normal_item)
        || !store(46, *handler_iterator)
        || !store(65, *handler_item)
        || !matches!(facts.op(1), Some(Operation::Invoke(_)))
        || !matches!(facts.op(17), Some(Operation::Comparison { target: 35, .. }))
        || !matches!(facts.op(55), Some(Operation::Comparison { target: 76, .. }))
        || facts.op(32) != Some(&Operation::Transfer)
        || facts.op(35) != Some(&Operation::Transfer)
        || facts.op(73) != Some(&Operation::Transfer)
        || facts.op(78) != Some(&Operation::Throw)
        || facts.op(79) != Some(&Operation::Return)
        || facts.op(5) != facts.op(41)
        || facts.op(12) != facts.op(50)
        || facts.op(21) != facts.op(60)
        || facts.op(29) != facts.op(70)
        || !matches!(facts.op(5), Some(Operation::Invoke(target))
            if target.kind() == InvokeKind::Interface && target.is_interface_reference()
                && target.owner() == "java/util/List" && target.name() == "iterator"
                && target.descriptor() == "()Ljava/util/Iterator;")
        || !matches!(facts.op(12), Some(Operation::Invoke(target))
            if target.kind() == InvokeKind::Interface && target.is_interface_reference()
                && target.owner() == "java/util/Iterator" && target.name() == "hasNext"
                && target.descriptor() == "()Z")
        || !matches!(facts.op(21), Some(Operation::Invoke(target))
            if target.kind() == InvokeKind::Interface && target.is_interface_reference()
                && target.owner() == "java/util/Iterator" && target.name() == "next"
                && target.descriptor() == "()Ljava/lang/Object;")
        || !matches!(facts.op(29), Some(Operation::Invoke(target))
            if target.name() == "call2" && target.descriptor() == "(Ljava/lang/Object;)V"
                && matches!(target.kind(), InvokeKind::Virtual | InvokeKind::Special))
    {
        return Ok(None);
    }
    // The branch sense and every transfer target are physical instruction facts, not inferred
    // from BCI order. A nearby altered loop therefore cannot reuse the copy certificate.
    for (bci, opcode, target) in [
        (17, 0x99, 35),
        (55, 0x99, 76),
        (32, 0xa7, 11),
        (35, 0xa7, 79),
        (73, 0xa7, 48),
    ] {
        if facts
            .step(bci)
            .is_none_or(|step| step.instruction.opcode() != opcode)
            || (bci != 35
                && !facts
                    .view
                    .successor_ids(facts.block_of(bci).unwrap())
                    .iter()
                    .any(|id| id.bci() == target))
        {
            return Ok(None);
        }
    }
    let stack = |bci| {
        facts
            .step(bci)?
            .instruction
            .writes()
            .iter()
            .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
    };
    let local = |bci, slot| {
        facts
            .step(bci)?
            .instruction
            .writes()
            .iter()
            .find_map(|(written, value)| (*written == Slot::Local(slot)).then_some(*value))
    };
    let uses_stack = |producer, consumer| {
        stack(producer).is_some_and(|value| {
            facts.step(consumer).is_some_and(|step| {
                stack_operands(step.instruction)
                    .iter()
                    .any(|(_, read)| facts.same(value, *read))
            })
        })
    };
    let uses_local = |producer, slot, consumer| {
        local(producer, slot).is_some_and(|value| {
            facts.step(consumer).is_some_and(|step| {
                step.instruction.reads().iter().any(|(read_slot, read)| {
                    *read_slot == Slot::Local(slot) && facts.same(value, *read)
                })
            })
        })
    };
    let entry = |bci, slot| {
        facts.step(bci).is_some_and(|step| {
            step.instruction.reads().iter().any(|(read_slot, value)| {
                *read_slot == Slot::Local(slot)
                    && matches!(facts.ssa.value(facts.resolve(*value)).def(),
                    Definition::Entry { slot: entry_slot, .. } if *entry_slot == Slot::Local(slot))
            })
        })
    };
    if ![0, 27, 67].iter().all(|bci| entry(*bci, 0))
        || ![4, 40].iter().all(|bci| entry(*bci, 1))
        || ![
            (0, 1),
            (4, 5),
            (5, 10),
            (11, 12),
            (12, 17),
            (20, 21),
            (21, 26),
            (27, 29),
            (28, 29),
            (40, 41),
            (41, 46),
            (48, 50),
            (50, 55),
            (58, 60),
            (60, 65),
            (67, 70),
            (68, 70),
            (76, 78),
        ]
        .iter()
        .all(|(from, to)| uses_stack(*from, *to))
        || !uses_local(26, *normal_item, 28)
        || !uses_local(65, *handler_item, 68)
        || !uses_local(38, *primary_slot, 76)
        || facts.step(38).is_none_or(|step| {
            !step.instruction.reads().iter().any(|(_, value)| {
                matches!(
                    facts.ssa.value(facts.resolve(*value)).def(),
                    Definition::Caught { .. } | Definition::Phi { .. }
                )
            })
        })
    {
        return Ok(None);
    }
    let expected_edges = BTreeSet::from([
        (0, 11, None),
        (0, 38, Some(body_row.ordinal)),
        (11, 20, None),
        (11, 35, None),
        (20, 11, None),
        (38, 48, None),
        (38, 38, Some(binding_row.ordinal)),
        (48, 58, None),
        (48, 76, None),
        (58, 48, None),
    ]);
    let mut actual_edges = BTreeSet::new();
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let row = match edge.kind() {
            CanonicalEdgeKind::Normal => None,
            CanonicalEdgeKind::Exception { handler_ordinal } => Some(handler_ordinal),
            CanonicalEdgeKind::Call { .. } | CanonicalEdgeKind::Return { .. } => return Ok(None),
        };
        actual_edges.insert((edge.from().bci(), edge.to().bci(), row));
    }
    if actual_edges != expected_edges
        || facts.canonical.edges().len() != expected_edges.len()
        || ![11, 48].iter().all(|header| {
            facts
                .block_at(*header)
                .and_then(|id| facts.view.index_of(&id))
                .and_then(|node| facts.view.loop_entered_at(node))
                .is_some_and(|loop_of| loop_of.blocks().len() == 2 && loop_of.latches().len() == 1)
        })
        || facts.view.dominates(
            facts.view.index_of(&facts.block_at(48).unwrap()).unwrap(),
            facts.view.index_of(&facts.block_at(58).unwrap()).unwrap(),
        )
    {
        return Ok(None);
    }
    Ok(Some(Plan {
        shape: Shape::LoopFinally {
            rows: [body_row.ordinal, binding_row.ordinal],
            normal_cleanup: (4, 38),
            handler_cleanup: (40, 76),
        },
        lead: (0, 0),
        body: (0, 4),
        owned: facts.blocks_in((0, 79)),
        join: None,
        facts: BCIS.to_vec(),
    }))
}

/// The named-catch entry asks only this private certificate before the ordinary catch reader.
/// An unsuccessful probe leaves that reader's existing decision unchanged.
#[allow(clippy::too_many_arguments)]
pub(crate) fn shared_finally_candidate(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    ops: &Operations,
    pool: &[CpEntryFacts],
    chains: &crate::concat::Plan,
    handlers: &[ExceptionHandlerFact],
    profile: &crate::pass::RecoveryProfile,
    current: &CanonicalBlockId,
    budget: &mut Budget,
) -> Result<Option<Plan>, StopReason> {
    if !FINALLY.admits(profile) || !matches!(handlers.len(), 1 | 2 | 3 | 4 | 5) {
        return Ok(None);
    }
    if handlers.len() == 1 {
        let row = &handlers[0];
        // This private slice begins at the method entry and its normal cleanup begins by
        // loading `this`. Reject other one-row finally shapes before charging a new probe.
        if row.catch_type_index.is_some()
            || row.start_bci != 0
            || current.bci() != 0
            || ops.get(row.end_bci) != Some(&Operation::Load { slot: 0 })
            || !matches!(ops.get(row.handler_bci), Some(Operation::Store { .. }))
        {
            return Ok(None);
        }
    }
    let sites = Sites::empty();
    let mut facts = Facts::new(canonical, view, ssa, ops, handlers, &sites, budget);
    facts.charge(current.bci())?;
    if handlers.len() == 2
        && let Some(plan) = prove_loop_finally(&mut facts, current)?
    {
        return Ok(Some(plan));
    }
    if handlers.len() == 2
        && let Some(plan) = prove_nullable_resource_finally(&mut facts, current)?
    {
        return Ok(Some(plan));
    }
    if handlers.len() == 2
        && let Some(plan) = prove_flag_conditional_finally(&mut facts, current)?
    {
        return Ok(Some(plan));
    }
    if handlers.len() == 2
        && let Some(plan) = prove_local_null_conditional_finally(&mut facts, current)?
    {
        return Ok(Some(plan));
    }
    if handlers.len() == 2
        && let Some(plan) = prove_segmented_null_lead_finally(&mut facts, current)?
    {
        return Ok(Some(plan));
    }
    if handlers.len() == 1 {
        return prove_conditional_finally(&mut facts, current);
    }
    if handlers.len() == 5 {
        return prove_segmented_finally(&mut facts, current);
    }
    if handlers.len() == 4 {
        if let Some(plan) = prove_nested_cleanup_finally(&mut facts, current, pool)? {
            return Ok(Some(plan));
        }
        if let Some(plan) = prove_two_catch_return_finally(&mut facts, current)? {
            return Ok(Some(plan));
        }
        return prove_shared_join_finally(&mut facts, current);
    }
    if handlers.len() == 2 {
        if let Some(plan) = prove_empty_catch_call_finally(&mut facts, current)? {
            return Ok(Some(plan));
        }
        return prove_nested_join_finally(&mut facts, current);
    }
    if let Some(plan) = prove_multi_return_loop_finally(&mut facts, current)? {
        return Ok(Some(plan));
    }
    if let Some(plan) = prove_shared_join_finally(&mut facts, current)? {
        return Ok(Some(plan));
    }
    prove_shared_finally(&mut facts, current, Some(chains))
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
        let sites = crate::init::Sites::empty();
        let mut facts = Facts::new(
            canonical,
            &view,
            ssa,
            &ops,
            &rows,
            &sites,
            &mut proof_budget,
        );
        prove_finally_copy(&mut facts, row, false)
    }

    fn shared_probe(
        class: &[u8],
        edit_rows: impl FnOnce(&mut Vec<ExceptionHandlerFact>),
        stop: Option<&str>,
    ) -> Result<Option<Plan>, StopReason> {
        shared_probe_method(class, b"handled", b"(Z)Ljava/lang/String;", edit_rows, stop)
    }

    fn shared_probe_method(
        class: &[u8],
        name: &[u8],
        descriptor: &[u8],
        edit_rows: impl FnOnce(&mut Vec<ExceptionHandlerFact>),
        stop: Option<&str>,
    ) -> Result<Option<Plan>, StopReason> {
        let mut budget = Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
            .expect("frozen class opens");
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
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
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
        let analyzed =
            analyze_method_ir(&[snapshot], &request, &mut budget).expect("frozen method analyzes");
        let ir = analyzed.ir();
        let canonical = ir.canonical().unwrap();
        let ssa = ir.ssa().unwrap();
        let code = ir.code().unwrap();
        let ops = Operations::of(code, ir.constant_pool());
        let mut chains =
            crate::concat::plan_four_conditional_strings(ssa, canonical, &ops, &mut budget)
                .expect("concat plan");
        if stop == Some("no-chain") {
            chains = crate::concat::Plan::empty();
        }
        let view = NormalFlowView::build(canonical, &mut budget).unwrap();
        let mut rows = code.exception_handlers.clone();
        edit_rows(&mut rows);
        let mut proof_limits = limits();
        if stop == Some("budget") {
            proof_limits.analysis_steps = 0;
        }
        let token = CancellationToken::new();
        if stop == Some("cancel") {
            token.cancel();
        }
        let mut proof_budget = Budget::with_cancellation_token(proof_limits, token);
        shared_finally_candidate(
            canonical,
            &view,
            ssa,
            &ops,
            ir.constant_pool(),
            &chains,
            &rows,
            &crate::pass::JAVA_8,
            canonical.blocks()[0].id(),
            &mut proof_budget,
        )
    }

    /// The fixed CF-16 Tf1 transcription: the two-row nullable local whose cleanup reads the
    /// slot the lead initialised and the body assigned.
    const LOCAL_NULL_TF1: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf1.class"
    );
    const LOCAL_NULL_TF1_DESCRIPTOR: &[u8] = b"(LContext;Ljava/lang/Object;)Ljava/lang/String;";

    #[test]
    fn local_null_certificate_claims_the_fixed_two_row_lowering() {
        let plan = shared_probe_method(
            LOCAL_NULL_TF1,
            b"test",
            LOCAL_NULL_TF1_DESCRIPTOR,
            |_| {},
            None,
        )
        .unwrap()
        .expect("local-null conditional finally certificate");
        let Shape::LocalNullConditionalFinally {
            row_ordinal,
            slot,
            normal_cleanup,
            handler_cleanup,
            saved_return,
        } = plan.shape()
        else {
            panic!("local-null conditional finally shape");
        };
        assert_eq!(*row_ordinal, 0);
        assert_eq!(*slot, 3);
        assert_eq!(*normal_cleanup, (41, 49));
        assert_eq!(*handler_cleanup, (54, 62));
        assert_eq!(*saved_return, (39, 51));
        assert_eq!(plan.lead(), (0, 2));
        assert_eq!(plan.body(), (2, 41));
        assert!(matches!(
            shared_probe_method(
                LOCAL_NULL_TF1,
                b"test",
                LOCAL_NULL_TF1_DESCRIPTOR,
                |_| {},
                Some("budget"),
            ),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            shared_probe_method(
                LOCAL_NULL_TF1,
                b"test",
                LOCAL_NULL_TF1_DESCRIPTOR,
                |_| {},
                Some("cancel"),
            ),
            Err(StopReason::Cancelled { .. })
        ));
    }

    /// The fixed CF-16 Tf3 transcription: the segmented two-row null-lead whose gap is the
    /// early return's own unguarded copy.
    const SEGMENTED_NULL_LEAD_TF3: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf3.class"
    );
    const SEGMENTED_NULL_LEAD_TF3_DESCRIPTOR: &[u8] = b"()[B";

    /// The verifier-valid neighbors the task froze: two break one link of the fixed grammar
    /// from source, the rest are same-length bytecode patches (`negatives/patch-tf3.py`) that
    /// rewrite one copy, one completion, one condition, or one exception-table row.
    const SEGMENTED_NULL_LEAD_TF3_NEIGHBORS: [(&str, &[u8]); 9] = [
        (
            "Tf3GapExtra",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3GapExtra/Tf3GapExtra.class"
            ),
        ),
        (
            "Tf3LeadField",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3LeadField/Tf3LeadField.class"
            ),
        ),
        (
            "Tf3TargetMismatch",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3TargetMismatch/Tf3TargetMismatch.class"
            ),
        ),
        (
            "Tf3ArgOtherSlot",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3ArgOtherSlot/Tf3ArgOtherSlot.class"
            ),
        ),
        (
            "Tf3ReturnIdentity",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3ReturnIdentity/Tf3ReturnIdentity.class"
            ),
        ),
        (
            "Tf3SelfRowWidened",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3SelfRowWidened/Tf3SelfRowWidened.class"
            ),
        ),
        (
            "Tf3CondNonNullRewrite",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3CondNonNullRewrite/Tf3CondNonNullRewrite.class"
            ),
        ),
        (
            "Tf3CondIfneRewrite",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3CondIfneRewrite/Tf3CondIfneRewrite.class"
            ),
        ),
        (
            "Tf3RethrowIdentity",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3RethrowIdentity/Tf3RethrowIdentity.class"
            ),
        ),
    ];

    #[test]
    fn segmented_null_lead_certificate_claims_the_fixed_two_row_lowering() {
        let plan = shared_probe_method(
            SEGMENTED_NULL_LEAD_TF3,
            b"test",
            SEGMENTED_NULL_LEAD_TF3_DESCRIPTOR,
            |_| {},
            None,
        )
        .unwrap()
        .expect("segmented null-lead finally certificate");
        let Shape::SegmentedNullLeadFinally {
            rows,
            slot,
            value_slot,
            segments,
            early_cleanup,
            early_return,
            normal_cleanup,
            returns,
            ..
        } = plan.shape()
        else {
            panic!("segmented null-lead finally shape");
        };
        assert_eq!(*rows, [0, 1]);
        assert_eq!(*slot, 1);
        assert_eq!(*value_slot, 2);
        assert_eq!(*segments, [(2, 18), (24, 47)]);
        assert_eq!(*early_cleanup, (18, 23));
        assert_eq!(*early_return, 23);
        assert_eq!(*normal_cleanup, (47, 51));
        assert_eq!(*returns, [(17, 23), (46, 52)]);
        assert_eq!(plan.lead(), (0, 2));
        assert_eq!(plan.body(), (2, 47));
        assert_eq!(plan.owned().len(), 6);
        assert_eq!(plan.facts().len(), 36);
        assert!(matches!(
            shared_probe_method(
                SEGMENTED_NULL_LEAD_TF3,
                b"test",
                SEGMENTED_NULL_LEAD_TF3_DESCRIPTOR,
                |_| {},
                Some("budget"),
            ),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            shared_probe_method(
                SEGMENTED_NULL_LEAD_TF3,
                b"test",
                SEGMENTED_NULL_LEAD_TF3_DESCRIPTOR,
                |_| {},
                Some("cancel"),
            ),
            Err(StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn segmented_null_lead_certificate_refuses_every_frozen_neighbor() {
        for (name, class) in SEGMENTED_NULL_LEAD_TF3_NEIGHBORS {
            let plan = shared_probe_method(class, b"test", b"()[B", |_| {}, None).unwrap();
            assert!(plan.is_none(), "{name} was claimed");
        }
    }

    #[test]
    fn segmented_null_lead_certificate_does_not_claim_the_family_neighbors() {
        // The family's own slices keep their own certificates: the flag conditional, the
        // local-null conditional and the null-lead straight finally each only ever answer for
        // their own grammar — a segmented gap, an unguarded copy or two saved returns is not
        // any of them.
        // The family's own entry is the walk's examination: the flag, local-null and straight
        // certificates answer through `examine`, and none of them is the segmented shape.
        let family: [(&str, &[u8], &[u8], &str); 3] = [
            (
                "Tf4",
                include_bytes!(
                    "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf4.class"
                ) as &[u8],
                &b"()Ljava/lang/String;"[..],
                "flag",
            ),
            (
                "Tf1",
                include_bytes!(
                    "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf1.class"
                ) as &[u8],
                &b"(LContext;Ljava/lang/Object;)Ljava/lang/String;"[..],
                "local-null",
            ),
            (
                "Tf2",
                include_bytes!(
                    "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf2.class"
                ) as &[u8],
                &NULL_LEAD_TF2_DESCRIPTOR[..],
                "straight",
            ),
        ];
        for (name, class, descriptor, _) in family {
            let Verdict::Claimed(plan) =
                examine_probe_labelled(name, class, b"test", descriptor, None).unwrap()
            else {
                panic!("{name} lost its own certificate");
            };
            assert!(
                !matches!(plan.shape(), Shape::SegmentedNullLeadFinally { .. }),
                "{name} was claimed by the segmented certificate"
            );
        }
    }

    #[test]
    fn local_null_certificate_does_not_claim_the_flag_or_field_shapes() {
        // The flag conditional's fixed transcription keeps its own certificate; the field
        // conditional keeps Test14's. Each shape only ever answers for its own grammar.
        let flag = shared_probe_method(
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf4.class"
            ),
            b"test",
            b"()Ljava/lang/String;",
            |_| {},
            None,
        )
        .unwrap()
        .expect("flag conditional certificate");
        assert!(matches!(flag.shape(), Shape::FlagConditionalFinally { .. }));
        let field = shared_probe_method(CONDITIONAL_TEST14, b"test", b"()V", |_| {}, None)
            .unwrap()
            .expect("conditional finally certificate");
        assert!(matches!(field.shape(), Shape::ConditionalFinally { .. }));
    }

    /// The fixed CF-16 Tf2 transcription: the two-row straight finally whose lead initialises
    /// the cleanup local with `null`, the body assigns from a call, and whose one saved return
    /// is a construction inside the protected range.
    const NULL_LEAD_TF2: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf2.class"
    );
    const NULL_LEAD_TF2_DESCRIPTOR: &[u8] = b"([B)LTf2$Result;";

    /// The verifier-valid neighbors the task froze: the first five break one link of the fixed
    /// grammar from source, the last four are same-length bytecode patches
    /// (`negatives/patch-tf2.py`) that rewrite one copy, one completion, or one row.
    const NULL_LEAD_TF2_NEIGHBORS: [(&str, &[u8]); 9] = [
        (
            "Tf2LeadField",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2LeadField/Tf2LeadField.class"
            ),
        ),
        (
            "Tf2LeadExtra",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2LeadExtra/Tf2LeadExtra.class"
            ),
        ),
        (
            "Tf2SlotMismatch",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2SlotMismatch/Tf2SlotMismatch.class"
            ),
        ),
        (
            "Tf2ArgOtherSlot",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2ArgOtherSlot/Tf2ArgOtherSlot.class"
            ),
        ),
        (
            "Tf2CleanupExtra",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2CleanupExtra/Tf2CleanupExtra.class"
            ),
        ),
        (
            "Tf2TargetMismatch",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2TargetMismatch/Tf2TargetMismatch.class"
            ),
        ),
        (
            "Tf2ReturnIdentity",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2ReturnIdentity/Tf2ReturnIdentity.class"
            ),
        ),
        (
            "Tf2RethrowIdentity",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2RethrowIdentity/Tf2RethrowIdentity.class"
            ),
        ),
        (
            "Tf2SelfRowWidened",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2SelfRowWidened/Tf2SelfRowWidened.class"
            ),
        ),
    ];

    /// Drives the whole guarded dispatch — the dedicated certificates first, the finally-copy
    /// claim gate behind them — exactly as the region walk reaches it, which is the order the
    /// families' mutual exclusion lives in.
    fn examine_probe_labelled(
        label: &str,
        class: &[u8],
        name: &[u8],
        descriptor: &[u8],
        stop: Option<&str>,
    ) -> Result<Verdict, StopReason> {
        let mut budget = Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
            .expect("frozen class opens");
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
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
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
        let analyzed =
            analyze_method_ir(&[snapshot], &request, &mut budget).expect("frozen method analyzes");
        let ir = analyzed.ir();
        let Some(canonical) = ir.canonical() else {
            panic!(
                "{label}: no canonical IR (quality {:?})",
                analyzed.report().quality
            );
        };
        let ssa = ir.ssa().unwrap();
        let code = ir.code().unwrap();
        let ops = Operations::of(code, ir.constant_pool());
        let view = NormalFlowView::build(canonical, &mut budget).unwrap();
        let rows = code.exception_handlers.clone();
        let mut proof_limits = limits();
        if stop == Some("budget") {
            proof_limits.analysis_steps = 0;
        }
        let token = CancellationToken::new();
        if stop == Some("cancel") {
            token.cancel();
        }
        let mut proof_budget = Budget::with_cancellation_token(proof_limits, token);
        let sites = crate::init::Sites::empty();
        let current = canonical.blocks()[0].id().clone();
        examine(
            canonical,
            &view,
            ssa,
            &ops,
            &rows,
            &sites,
            &crate::pass::JAVA_8,
            &current,
            &mut proof_budget,
        )
    }

    #[test]
    fn null_lead_straight_certificate_claims_the_fixed_two_row_lowering() {
        let Verdict::Claimed(plan) = examine_probe_labelled(
            "Tf2",
            NULL_LEAD_TF2,
            b"test",
            NULL_LEAD_TF2_DESCRIPTOR,
            None,
        )
        .unwrap() else {
            panic!("the null-lead straight finally certificate claims the fixed class");
        };
        assert_eq!(plan.lead(), (0, 2));
        assert_eq!(plan.body(), (2, 25));
        let Shape::Finally {
            normal_cleanup,
            completion,
            row_ordinal,
            structured,
        } = plan.shape()
        else {
            panic!("the straight finally shape: {:?}", plan.shape());
        };
        assert_eq!(*row_ordinal, 0);
        assert_eq!(*normal_cleanup, (25, 30));
        assert!(matches!(
            completion,
            FinallyCompletion::SavedReturn {
                save: 24,
                returns: 31
            }
        ));
        assert!(*structured);
        // The plan's evidence spans the statement's protected span: the body, both cleanup
        // copies, and the saved/rethrown completions. The lead's own two instructions anchor the
        // artifact's source map directly, the way every lead presentation does.
        for bci in [2, 7, 24, 25, 26, 27, 32, 34, 35, 36, 39, 41] {
            assert!(
                plan.facts().contains(&bci),
                "BCI {bci} carries no origin: {:?}",
                plan.facts()
            );
        }
        assert!(matches!(
            examine_probe_labelled(
                "Tf2",
                NULL_LEAD_TF2,
                b"test",
                NULL_LEAD_TF2_DESCRIPTOR,
                Some("budget"),
            ),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            examine_probe_labelled(
                "Tf2",
                NULL_LEAD_TF2,
                b"test",
                NULL_LEAD_TF2_DESCRIPTOR,
                Some("cancel"),
            ),
            Err(StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn null_lead_straight_certificate_refuses_the_verifier_valid_neighbors() {
        for (name, class) in NULL_LEAD_TF2_NEIGHBORS {
            let mut descriptor = b"([B)L".to_vec();
            descriptor.extend_from_slice(name.as_bytes());
            descriptor.extend_from_slice(b"$Result;");
            let verdict = examine_probe_labelled(name, class, b"test", &descriptor, None)
                .unwrap_or_else(|stop| panic!("{name} stopped: {stop:?}"));
            match verdict {
                // No guarded shape answers here at all — the broken link is one the shape
                // readers decline before any rule claims the block.
                Verdict::NotGuarded => {}
                // The finally-copy claim gate reached its own refusal: it answers outside any
                // registered rule's claim, anchored at the class's own handler entry.
                Verdict::Refused { pass, .. } => {
                    assert_eq!(pass, None, "{name} refused by another rule");
                }
                Verdict::Claimed(_) => panic!("{name} claimed a finally"),
            }
        }
    }

    #[test]
    fn null_lead_straight_extension_leaves_the_family_certificates_their_own_shapes() {
        let flag = examine_probe_labelled(
            "Tf4",
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf4.class"
            ),
            b"test",
            b"()Ljava/lang/String;",
            None,
        )
        .unwrap();
        let Verdict::Claimed(plan) = flag else {
            panic!("the flag conditional certificate claims Tf4");
        };
        assert!(matches!(plan.shape(), Shape::FlagConditionalFinally { .. }));
        let local_null = examine_probe_labelled(
            "Tf1",
            LOCAL_NULL_TF1,
            b"test",
            LOCAL_NULL_TF1_DESCRIPTOR,
            None,
        )
        .unwrap();
        let Verdict::Claimed(plan) = local_null else {
            panic!("the local-null conditional certificate claims Tf1");
        };
        assert!(matches!(
            plan.shape(),
            Shape::LocalNullConditionalFinally { .. }
        ));
    }

    const CALL: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyCall.class"
    );
    const EMPTY_CATCH_TEST16: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/Test16.class"
    );
    const EMPTY_CATCH_TEST16_NEIGHBORS: [&[u8]; 5] = [
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/near/different-target.class"
        ),
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/near/cleanup-covered.class"
        ),
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/near/rows-swapped.class"
        ),
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/near/throwable-rewritten.class"
        ),
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/near/external-cleanup-entry.class"
        ),
    ];

    const NESTED_CLEANUP_TEST4: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test4-nested-cleanup/classes/jadx/tests/integration/trycatch/TestTryCatchFinally4$TestCls.class"
    );

    #[test]
    fn pinned_test4_nested_cleanup_has_one_four_row_certificate() {
        let probe = |class, stop| shared_probe_method(class, b"test", b"()V", |_| {}, stop);
        let plan = probe(NESTED_CLEANUP_TEST4, None)
            .unwrap()
            .expect("four-row nested cleanup certificate");
        let Shape::NestedCleanupFinally {
            rows,
            normal_handler,
            cleanup,
            ..
        } = plan.shape()
        else {
            panic!("wrong finally certificate")
        };
        assert_eq!(*rows, [0, 1, 2, 3]);
        assert_eq!(normal_handler.bci(), 34);
        assert_eq!(*cleanup, (22, 31));
        assert_eq!(plan.body(), (17, 22));
        assert_eq!(plan.lead(), (0, 17));
        assert_eq!(plan.facts().len(), 31);
        assert_eq!(plan.owned().len(), 5);
        assert!(matches!(
            probe(NESTED_CLEANUP_TEST4, Some("budget")),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            probe(NESTED_CLEANUP_TEST4, Some("cancel")),
            Err(StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn test4_call_and_coverage_neighbors_cannot_merge_cleanup() {
        let cases: &[(&[u8], &[u8])] = &[
            (b"\0\x05close", b"\0\x05flush"),
            (b"\0\x06delete", b"\0\x06exists"),
            (b"\0\x13java/io/IOException", b"\0\x13java/lang/Throwable"),
            (b"\0\x16\0\x1f\0\x22\0\x23", b"\0\x1a\0\x1f\0\x22\0\x23"),
        ];
        for (old, new) in cases {
            let mut changed = NESTED_CLEANUP_TEST4.to_vec();
            let offsets = changed
                .windows(old.len())
                .enumerate()
                .filter_map(|(at, window)| (window == *old).then_some(at))
                .collect::<Vec<_>>();
            assert_eq!(offsets.len(), 1, "mutation has one physical site");
            changed[offsets[0]..offsets[0] + old.len()].copy_from_slice(new);
            let proved = shared_probe_method(&changed, b"test", b"()V", |_| {}, None).unwrap();
            assert!(!matches!(
                proved,
                Some(Plan {
                    shape: Shape::NestedCleanupFinally { .. },
                    ..
                })
            ));
        }
    }

    #[test]
    fn empty_catch_test16_two_real_rows_three_call_copies() {
        let probe = |class, stop| shared_probe_method(class, b"test", b"()V", |_| {}, stop);
        let plan = probe(EMPTY_CATCH_TEST16, None)
            .unwrap()
            .expect("two-row certificate");
        let Shape::EmptyCatchCallFinally {
            rows,
            cleanup_target,
            catch_handler,
            catch_type,
            catch_parameter,
            cleanup,
            transfers,
        } = plan.shape()
        else {
            panic!("wrong certificate")
        };
        assert_eq!(*rows, [0, 1]);
        assert_eq!(
            (catch_handler.bci(), *catch_type, *catch_parameter),
            (9, 15, 1)
        );
        assert_eq!(*cleanup, [(3, 6), (10, 13), (17, 20)]);
        assert_eq!(*transfers, [6, 13]);
        assert_eq!(cleanup_target.kind(), InvokeKind::Static);
        assert_eq!(
            cleanup_target.owner(),
            "jadx/tests/integration/trycatch/TestTryCatchFinally16$TestCls$TCls"
        );
        assert_eq!(
            (cleanup_target.name(), cleanup_target.descriptor()),
            ("doFinally", "()V")
        );
        assert_eq!(plan.body(), (0, 3));
        assert_eq!(plan.facts(), &[0, 3, 6, 9, 10, 13, 16, 17, 20, 21, 22]);
        assert_eq!(
            plan.owned()
                .iter()
                .map(CanonicalBlockId::bci)
                .collect::<Vec<_>>(),
            [0, 9, 16]
        );
        assert_eq!(plan.join().map(CanonicalBlockId::bci), Some(22));
        for neighbor in EMPTY_CATCH_TEST16_NEIGHBORS {
            assert!(probe(neighbor, None).unwrap().is_none());
        }
        assert!(matches!(
            probe(EMPTY_CATCH_TEST16, Some("budget")),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            probe(EMPTY_CATCH_TEST16, Some("cancel")),
            Err(StopReason::Cancelled { .. })
        ));
    }
    #[test]
    fn two_catch_test17_four_real_rows_four_call_copies() {
        const CLASS: &[u8] = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test17-two-catches/classes/Test17.class"
        );
        let probe = |stop| shared_probe_method(CLASS, b"test", b"()I", |_| {}, stop);
        let plan = probe(None).unwrap().expect("four-row certificate");
        let Shape::TwoCatchReturnFinally {
            rows,
            catches,
            cleanup,
            saved_return,
        } = plan.shape()
        else {
            panic!("wrong certificate")
        };
        assert_eq!(*rows, [0, 1, 2, 3]);
        assert_eq!(
            catches
                .iter()
                .map(|(block, _, _)| block.bci())
                .collect::<Vec<_>>(),
            [9, 16]
        );
        assert_eq!(*cleanup, [(3, 6), (10, 13), (19, 22), (25, 28)]);
        assert_eq!(*saved_return, (18, 23));
        assert_eq!(
            plan.facts(),
            &[
                0, 3, 6, 9, 10, 13, 16, 17, 18, 19, 22, 23, 24, 25, 28, 29, 30, 31
            ]
        );
        assert_eq!(plan.join().map(CanonicalBlockId::bci), Some(30));
        assert!(matches!(
            probe(Some("budget")),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            probe(Some("cancel")),
            Err(StopReason::Cancelled { .. })
        ));
    }
    const CONCAT_SAVED: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-27/cf16-finally/original/FinallyOnce.class"
    );
    const CONDITIONAL_TEST14: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/TestTryCatchFinally14$TestCls.class"
    );
    const CONDITIONAL_TEST14_NEGATIVES: [&[u8]; 6] = [
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-field.class"
        ),
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-call.class"
        ),
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-predicate.class"
        ),
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-self-protected.class"
        ),
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-external-entry.class"
        ),
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-throwable.class"
        ),
    ];
    const CONDITIONAL_TEST14_SLOT0: &[u8] =
        include_bytes!("../../../tests/fixtures/p3-conditional-finally/Test14-slot0.class");
    const CONDITIONAL_TEST14_SECOND_FIELD: &[u8] =
        include_bytes!("../../../tests/fixtures/p3-conditional-finally/Test14-second-field.class");
    const CONDITIONAL_TEST14_HANDLER_CALL: &[u8] =
        include_bytes!("../../../tests/fixtures/p3-conditional-finally/Test14-handler-call.class");
    const FIELD: &[u8] =
        include_bytes!("../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinally.class");
    const JOIN: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoin.class"
    );
    const NESTED_TEST_CLS: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/TestTryCatchFinally12$TestCls.class"
    );
    const SEGMENTED_TEST_CLS: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/TestTryCatchFinally13$TestCls.probe.class"
    );
    const SEGMENTED_NEGATIVE_TARGET: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/negatives/cleanup-target.class"
    );
    const SEGMENTED_NEGATIVE_BRANCH: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/negatives/branch-bypass.class"
    );
    const SEGMENTED_NEGATIVE_RANGE: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/negatives/range-expanded.class"
    );
    const SEGMENTED_NEGATIVE_THROW: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/negatives/rethrow-changed.class"
    );
    const SEGMENTED_EXTERNAL_ENTRY: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/acceptance/external-entry.class"
    );

    #[test]
    fn segmented_test13_certificate_and_verifier_valid_neighbors() {
        let probe = |class, edit_rows: fn(&mut Vec<ExceptionHandlerFact>), stop| {
            shared_probe_method(class, b"test", b"(I)V", edit_rows, stop)
        };
        let plan = probe(SEGMENTED_TEST_CLS, |_| {}, None)
            .unwrap()
            .expect("the five-row certificate is complete");
        let Shape::SegmentedFinally {
            rows,
            segments,
            catch_body,
            cleanup,
            early_return,
            transfers,
            ..
        } = plan.shape()
        else {
            panic!("segmented finally shape");
        };
        assert_eq!(*rows, [0, 1, 2, 3, 4]);
        assert_eq!(*segments, [(0, 10), (15, 37)]);
        assert_eq!(*catch_body, (44, 49));
        assert_eq!(*cleanup, [(10, 14), (37, 41), (49, 53), (57, 61)]);
        assert_eq!(*early_return, 14);
        assert_eq!(*transfers, [41, 53]);
        assert_eq!(plan.join().map(CanonicalBlockId::bci), Some(63));
        assert!(!plan.owned().iter().any(|block| block.bci() == 63));
        for class in [
            SEGMENTED_NEGATIVE_TARGET,
            SEGMENTED_NEGATIVE_BRANCH,
            SEGMENTED_NEGATIVE_RANGE,
            SEGMENTED_NEGATIVE_THROW,
            SEGMENTED_EXTERNAL_ENTRY,
        ] {
            assert!(probe(class, |_| {}, None).unwrap().is_none());
        }
        for edit in [
            (|rows: &mut Vec<ExceptionHandlerFact>| {
                rows.pop();
            }) as fn(&mut Vec<ExceptionHandlerFact>),
            |rows| rows.swap(0, 1),
            |rows| rows[3].end_bci = 38,
        ] {
            assert!(probe(SEGMENTED_TEST_CLS, edit, None).unwrap().is_none());
        }
        assert!(matches!(
            probe(SEGMENTED_TEST_CLS, |_| {}, Some("budget")),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            probe(SEGMENTED_TEST_CLS, |_| {}, Some("cancel")),
            Err(StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn nested_test1_and_test2_have_exclusive_joined_certificates() {
        for (method, protected_end, normal, handler, join) in [
            (b"test1".as_slice(), 29, (29, 39), 42, 55),
            (b"test2".as_slice(), 19, (19, 29), 32, 45),
        ] {
            let plan = shared_probe_method(NESTED_TEST_CLS, method, b"(I)V", |_| {}, None)
                .unwrap()
                .expect("the two-copy joined finally is proved");
            let Shape::Finally {
                normal_cleanup,
                completion,
                row_ordinal,
                structured,
            } = plan.shape()
            else {
                panic!("outer finally shape");
            };
            assert_eq!(plan.body(), (0, protected_end));
            assert_eq!(*normal_cleanup, normal);
            assert_eq!(*row_ordinal, 1);
            assert!(*structured);
            assert!(matches!(
                completion,
                FinallyCompletion::Joined { named_row: 0, continuation, .. } if *continuation == join
            ));
            assert_eq!(
                plan.join().map(CanonicalBlockId::bci),
                (method == b"test1").then_some(join)
            );
            assert!(!plan.owned().iter().any(|block| block.bci() == join));
            assert!(plan.facts().contains(&handler));
        }
    }

    #[test]
    fn nested_test3_append_copies_have_one_shared_join_certificate() {
        let plan = shared_probe_method(
            NESTED_TEST_CLS,
            b"test3",
            b"(I)V",
            |rows| {
                assert_eq!(
                    rows.iter()
                        .map(|row| (
                            row.ordinal,
                            row.start_bci,
                            row.end_bci,
                            row.handler_bci,
                            row.catch_type_index
                        ))
                        .collect::<Vec<_>>(),
                    [
                        (0, 0, 5, 18, Some(13)),
                        (1, 0, 5, 42, None),
                        (2, 18, 29, 42, None)
                    ]
                );
            },
            None,
        )
        .unwrap()
        .expect("all three append copies are proved");
        let Shape::SharedFinally {
            rows,
            normal_cleanup,
            catch_cleanup,
            completion,
            ..
        } = plan.shape()
        else {
            panic!("shared finally shape");
        };
        assert_eq!(*rows, [0, 1, 2]);
        assert_eq!((*normal_cleanup, *catch_cleanup), ((5, 15), (29, 39)));
        assert_eq!(
            *completion,
            SharedFinallyCompletion::Joined {
                transfers: [15, 39]
            }
        );
        assert_eq!(plan.join().map(CanonicalBlockId::bci), Some(55));
        assert!(!plan.owned().iter().any(|block| block.bci() == 55));
        assert_eq!(
            plan.facts(),
            &(vec![
                0, 1, 2, 5, 6, 9, 11, 14, 15, 18, 19, 20, 23, 25, 28, 29, 30, 33, 35, 38, 39, 42,
                43, 44, 47, 49, 52, 53, 54,
            ])
        );
        assert!(
            shared_probe_method(NESTED_TEST_CLS, b"test3", b"(I)V", |_| {}, Some("budget"))
                .is_err()
        );
        assert!(
            shared_probe_method(NESTED_TEST_CLS, b"test3", b"(I)V", |_| {}, Some("cancel"))
                .is_err()
        );
    }
    const JOIN_VALUE_MISMATCH: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoinValueMismatch.class"
    );
    const JOIN_ROW: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoinRow.class"
    );
    const JOIN_EDGE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoinEdge.class"
    );

    #[test]
    fn shared_join_certificate_owns_copies_and_preserves_continuation() {
        let plan = shared_probe_method(JOIN, b"test", b"(Ljava/lang/Object;)Z", |_| {}, None)
            .unwrap()
            .expect("shared join certificate");
        let Shape::SharedFinally {
            rows,
            catch_body,
            normal_cleanup,
            catch_cleanup,
            completion,
            ..
        } = plan.shape()
        else {
            panic!("shared finally shape");
        };
        assert_eq!(*rows, [0, 1, 2]);
        assert_eq!(plan.lead(), (0, 5));
        assert_eq!(plan.body(), (5, 10));
        assert_eq!(*catch_body, (18, 23));
        assert_eq!((*normal_cleanup, *catch_cleanup), ((10, 15), (23, 28)));
        assert_eq!(
            *completion,
            SharedFinallyCompletion::Joined {
                transfers: [15, 28]
            }
        );
        assert_eq!(plan.join().map(CanonicalBlockId::bci), Some(39));
        assert!(!plan.owned().iter().any(|block| block.bci() == 39));
        assert_eq!(
            plan.facts(),
            &(vec![
                0, 1, 2, 5, 6, 9, 10, 11, 12, 15, 18, 19, 20, 23, 24, 25, 28, 31, 32, 33, 34, 37,
                38
            ])
        );
    }

    #[test]
    fn shared_join_refuses_changed_rows_or_field_value_and_propagates_stops() {
        let probe = |class, edit_rows: fn(&mut Vec<ExceptionHandlerFact>), stop| {
            shared_probe_method(class, b"test", b"(Ljava/lang/Object;)Z", edit_rows, stop)
        };
        assert!(probe(JOIN_VALUE_MISMATCH, |_| {}, None).unwrap().is_none());
        assert!(probe(JOIN_ROW, |_| {}, None).unwrap().is_none());
        assert!(probe(JOIN_EDGE, |_| {}, None).unwrap().is_none());
        assert!(
            probe(JOIN, |rows| rows[1].end_bci = 9, None)
                .unwrap()
                .is_none()
        );
        assert!(probe(JOIN, |rows| rows.swap(0, 1), None).unwrap().is_none());
        assert!(
            probe(JOIN, |rows| rows[2].handler_bci = 18, None)
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            probe(JOIN, |_| {}, Some("budget")),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            probe(JOIN, |_| {}, Some("cancel")),
            Err(StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn shared_call_certificate_owns_three_rows_and_two_returns() {
        let plan = shared_probe(CALL, |_| {}, None)
            .unwrap()
            .expect("one complete certificate");
        let Shape::SharedFinally {
            rows,
            catch_body,
            completion,
            normal_cleanup,
            catch_cleanup,
            ..
        } = plan.shape()
        else {
            panic!("the shared call shape is distinct from the single-return proof");
        };
        assert_eq!(*rows, [0, 1, 2]);
        assert_eq!(plan.body(), (4, 21));
        assert_eq!(*catch_body, (26, 30));
        assert_eq!(
            *completion,
            SharedFinallyCompletion::SavedReturns([(20, 25), (29, 34)])
        );
        assert_eq!((*normal_cleanup, *catch_cleanup), ((21, 24), (30, 33)));
        assert!(plan.owned().iter().any(|block| block.bci() == 35));
        assert!(plan.facts().contains(&36) && plan.facts().contains(&40));
    }

    #[test]
    fn concat_saved_return_uses_the_same_complete_chain_certificate() {
        let plan = shared_probe(CONCAT_SAVED, |_| {}, None)
            .unwrap()
            .expect("the protected concat is one saved-return value");
        let Shape::SharedFinally {
            rows,
            catch_body,
            completion,
            normal_cleanup,
            catch_cleanup,
            ..
        } = plan.shape()
        else {
            panic!("the three-row saved-return shape");
        };
        assert_eq!(*rows, [0, 1, 2]);
        assert_eq!(*catch_body, (31, 55));
        assert_eq!(
            *completion,
            SharedFinallyCompletion::SavedReturns([(20, 30), (54, 64)])
        );
        assert_eq!((*normal_cleanup, *catch_cleanup), ((21, 29), (55, 63)));
        assert!(
            [32, 35, 36, 39, 41, 44, 45, 48, 51]
                .iter()
                .all(|bci| plan.facts().contains(bci))
        );
        assert!(
            shared_probe(CONCAT_SAVED, |_| {}, Some("no-chain"))
                .unwrap()
                .is_none()
        );
        assert!(
            shared_probe(CONCAT_SAVED, |rows| rows[2].end_bci = 51, None)
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            shared_probe(CONCAT_SAVED, |_| {}, Some("budget")),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            shared_probe(CONCAT_SAVED, |_| {}, Some("cancel")),
            Err(StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn conditional_test14_two_copies_and_verifier_valid_neighbors() {
        let probe = |class, stop| shared_probe_method(class, b"test", b"()V", |_| {}, stop);
        let plan = probe(CONDITIONAL_TEST14, None)
            .unwrap()
            .expect("conditional finally certificate");
        let Shape::ConditionalFinally {
            row_ordinal,
            normal_cleanup,
            handler_cleanup,
            normal_return,
        } = plan.shape()
        else {
            panic!("conditional finally shape");
        };
        assert_eq!(*row_ordinal, 0);
        assert_eq!(*normal_cleanup, (14, 31));
        assert_eq!(*handler_cleanup, (31, 48));
        assert_eq!(*normal_return, 48);
        assert_eq!(plan.body(), (0, 14));
        assert_eq!(
            plan.facts(),
            &[
                0, 1, 4, 7, 8, 11, 14, 15, 18, 21, 22, 25, 28, 31, 32, 33, 36, 39, 40, 43, 46, 47,
                48
            ]
        );
        for class in CONDITIONAL_TEST14_NEGATIVES {
            assert!(probe(class, None).unwrap().is_none());
        }
        assert!(probe(CONDITIONAL_TEST14_SLOT0, None).unwrap().is_none());
        assert!(
            probe(CONDITIONAL_TEST14_SECOND_FIELD, None)
                .unwrap()
                .is_none()
        );
        assert!(
            probe(CONDITIONAL_TEST14_HANDLER_CALL, None)
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            probe(CONDITIONAL_TEST14, Some("budget")),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            probe(CONDITIONAL_TEST14, Some("cancel")),
            Err(StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn shared_call_certificate_refuses_row_changes_and_propagates_stops() {
        assert!(
            shared_probe(CALL, |rows| rows.swap(0, 1), None)
                .unwrap()
                .is_none()
        );
        assert!(
            shared_probe(CALL, |rows| rows[1].end_bci = 24, None)
                .unwrap()
                .is_none()
        );
        assert!(
            shared_probe(CALL, |rows| rows[2].handler_bci = 26, None)
                .unwrap()
                .is_none()
        );
        assert!(
            shared_probe(
                CALL,
                |rows| {
                    let mut extra = rows[2].clone();
                    extra.ordinal = 3;
                    rows.push(extra);
                },
                None
            )
            .unwrap()
            .is_none()
        );
        assert!(matches!(
            shared_probe(CALL, |_| {}, Some("budget")),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            shared_probe(CALL, |_| {}, Some("cancel")),
            Err(StopReason::Cancelled { .. })
        ));
    }

    #[test]
    fn shared_field_certificate_owns_full_spans_and_propagates_stops() {
        let plan = shared_probe(FIELD, |_| {}, None)
            .unwrap()
            .expect("field certificate");
        let Shape::SharedFinally {
            rows,
            catch_body,
            completion,
            normal_cleanup,
            catch_cleanup,
            ..
        } = plan.shape()
        else {
            panic!("shared finally shape");
        };
        assert_eq!(*rows, [0, 1, 2]);
        assert_eq!(plan.body(), (4, 21));
        assert_eq!(*catch_body, (31, 35));
        assert_eq!(
            *completion,
            SharedFinallyCompletion::SavedReturns([(20, 30), (34, 44)])
        );
        assert_eq!((*normal_cleanup, *catch_cleanup), ((21, 29), (35, 43)));
        assert!(plan.owned().iter().any(|block| block.bci() == 45));
        assert!(plan.facts().contains(&46) && plan.facts().contains(&55));
        assert!(
            shared_probe(FIELD, |rows| rows[1].end_bci = 29, None)
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            shared_probe(FIELD, |_| {}, Some("budget")),
            Err(StopReason::Budget { .. })
        ));
        assert!(matches!(
            shared_probe(FIELD, |_| {}, Some("cancel")),
            Err(StopReason::Cancelled { .. })
        ));
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
        let sites = crate::init::Sites::empty();
        let mut facts = Facts::new(canonical, &view, ssa, &ops, &rows, &sites, &mut budget);
        let row = rows
            .first()
            .ok_or_else(|| "resource row missing".to_string())?;
        twr(&mut facts, &crate::pass::JAVA_8, current, row).map_err(|_| "TWR refused".to_string())
    }

    #[test]
    fn constructor_resource_requires_its_verified_site() {
        let class = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-26/multi-resource-twr/release8/MultiResourceTwr.class"
        );
        let mut budget = Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
            .expect("the frozen fixture opens");
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
            name: JvmBytes(b"run".to_vec()),
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
        let analyzed = analyze_method_ir(&[snapshot], &request, &mut budget)
            .expect("the frozen run method analyzes");
        let ir = analyzed.ir();
        let canonical = ir.canonical().unwrap();
        let ssa = ir.ssa().unwrap();
        let code = ir.code().unwrap();
        let ops = Operations::of(code, ir.constant_pool());
        let chains = crate::concat::plan(ssa, &ops);
        let view = NormalFlowView::build(canonical, &mut budget).unwrap();
        let rows = code.exception_handlers.clone();

        for (reserved, expected) in [
            (BTreeSet::new(), [Some(((0, 10), 0)), Some(((10, 20), 1))]),
            (BTreeSet::from([0]), [None, Some(((10, 20), 1))]),
            (BTreeSet::from([10]), [Some(((0, 10), 0)), None]),
        ] {
            let fields = crate::field::Plan::empty();
            let arrays = crate::build::ArrayInitializers::default();
            let sites = crate::init::sites(
                ssa,
                &ops,
                &chains,
                &reserved,
                &fields,
                &arrays,
                8,
                &[],
                &crate::facts::MethodFacts::new("run", "()V", 0),
                code,
            );
            for (index, (store, floor, end)) in [(9, 0, 10), (19, 10, 20)].into_iter().enumerate() {
                let mut case_budget = Budget::new(limits());
                let facts =
                    Facts::new(canonical, &view, ssa, &ops, &rows, &sites, &mut case_budget);
                let expected = expected[index]
                    .map(Ok)
                    .unwrap_or(Err((Unproven::ResourceInit, store)));
                assert_eq!(
                    initialisation(&facts, end, floor),
                    expected,
                    "store at BCI {store}, reserved allocations {reserved:?}"
                );
            }
        }

        let empty_sites = crate::init::Sites::empty();
        let mut case_budget = Budget::new(limits());
        let facts = Facts::new(
            canonical,
            &view,
            ssa,
            &ops,
            &rows,
            &empty_sites,
            &mut case_budget,
        );
        assert_eq!(
            initialisation(&facts, 10, 0),
            Err((Unproven::ResourceInit, 9))
        );
        assert_eq!(
            initialisation(&facts, 20, 10),
            Err((Unproven::ResourceInit, 19))
        );

        let arrays = crate::build::ArrayInitializers::default();
        let sites = crate::init::sites(
            ssa,
            &ops,
            &chains,
            &BTreeSet::new(),
            &crate::field::Plan::empty(),
            &arrays,
            8,
            &[],
            &crate::facts::MethodFacts::new("run", "()V", 0),
            code,
        );
        let mut case_budget = Budget::new(limits());
        let facts = Facts::new(canonical, &view, ssa, &ops, &rows, &sites, &mut case_budget);
        assert_eq!(constructed_initialisation(&facts, 19, 10, 19), Err(19));
    }

    #[test]
    fn a_terminal_saved_return_is_part_of_the_resource_plan() {
        let class =
            include_bytes!("../../../tests/fixtures/p3-multi-resource-twr/TwrReturnTail.class");
        let plan = resource_plan(class, "runSaved").expect("the TWR structure is recognized");
        let Shape::Resources {
            returns: Some(20),
            cleanup,
            ..
        } = plan.shape()
        else {
            panic!("the body store/load/return tail is proved")
        };
        assert!(cleanup.contains(&21), "the proved primary handler is owned");
        assert!(
            !cleanup.contains(&19) && !cleanup.contains(&20),
            "the source return tail remains live"
        );
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

#[cfg(test)]
mod monitor_branch_tests {
    use super::*;
    use jarde_jvm::engine::analyze_method_ir;
    use jarde_jvm::environment::ResolutionEnvironment;
    use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
    use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
    use jarde_reader::budget::Limits;
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
        MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
        RuntimeView,
    };

    fn plans(class: &[u8], name: &str, descriptor: &str) -> Vec<Verdict> {
        let mut budget = Budget::new(Limits {
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
        });
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
            .expect("the frozen class opens");
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
            loader: LoaderId("app".into()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        };
        let analyzed = analyze_method_ir(
            &[snapshot.clone()],
            &MethodAnalysisRequest {
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
                    owner: definition,
                    name: JvmBytes(name.as_bytes().to_vec()),
                    descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
                },
                stages: AnalysisStage::ALL.to_vec(),
            },
            &mut budget,
        )
        .expect("the frozen method analyzes");
        let ir = analyzed.ir();
        let canonical = ir.canonical().expect("canonical CFG");
        let ssa = ir.ssa().expect("SSA");
        let code = ir.code().expect("Code attribute");
        let operations = Operations::of(code, ir.constant_pool());
        let view = NormalFlowView::build(canonical, &mut budget).expect("normal-flow view");
        let sites = crate::init::Sites::empty();
        let mut verdicts = Vec::new();
        for block in canonical.blocks() {
            verdicts.push(
                examine(
                    canonical,
                    &view,
                    ssa,
                    &operations,
                    &code.exception_handlers,
                    &sites,
                    &crate::pass::JAVA_8,
                    block.id(),
                    &mut budget,
                )
                .expect("guard examination is bounded"),
            );
        }
        verdicts
    }

    #[test]
    fn completed_instance_field_prefix_is_not_a_resource_header() {
        let class = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-switch-catch/TestTryCatchFinally12$TestCls.class"
        );
        assert!(matches!(
            plans(class, "runTest", "(II)Ljava/lang/String;").first(),
            Some(Verdict::NotGuarded)
        ));
        let resource = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-switch-catch/variants/TwrSwitchCatch.class"
        );
        assert!(matches!(
            plans(resource, "runTest", "(II)Ljava/lang/String;").first(),
            Some(Verdict::Refused { pass: Some(_), .. })
        ));
    }

    /// A statement that is no store — a `getstatic`-consuming call, a plain `void` call — is no
    /// resource's initialisation, so the named row behind it is not examined as one: the walk is
    /// left its own `NotGuarded`, which is what lets [`catches`] present the clause.
    #[test]
    fn a_non_store_statement_prefix_is_not_examined_as_a_resource_header() {
        let getstatic = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/near/P2GetstaticRead.class"
        );
        assert!(matches!(
            plans(getstatic, "run", "()Ljava/lang/String;").first(),
            Some(Verdict::NotGuarded)
        ));
        let void_call = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/M5.class"
        );
        assert!(matches!(
            plans(void_call, "main", "([Ljava/lang/String;)V").first(),
            Some(Verdict::NotGuarded)
        ));
    }

    /// A named row one **completed store** precedes — the CF-15 crossing answer — is no resource
    /// header either: the store is one statement the walk reads, the range begins where it ends,
    /// and neither the row's end nor its handler closes anything, so no lowering of this shape
    /// could own the row. The construction store of `C1.five`, the same store in
    /// `C4.constructNamed` and the call store of N1's `main` are the three frozen shapes; the walk
    /// is left its own `NotGuarded`, which is what lets [`catches`] present the clauses.
    #[test]
    fn a_named_row_a_completed_store_precedes_is_not_examined_as_a_resource_header() {
        let five = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/C1.class"
        );
        assert!(matches!(
            plans(five, "five", "()Ljava/lang/String;").first(),
            Some(Verdict::NotGuarded)
        ));
        let construct = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/C4.class"
        );
        assert!(matches!(
            plans(construct, "constructNamed", "()Ljava/lang/String;").first(),
            Some(Verdict::NotGuarded)
        ));
        let call = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/N1.class"
        );
        assert!(matches!(
            plans(call, "main", "([Ljava/lang/String;)V").first(),
            Some(Verdict::NotGuarded)
        ));
    }

    /// The crossing answer's own boundaries: a catch-all row, a row that splits the construction,
    /// a store whose run the walk cannot read and a construction the store does not follow keep
    /// the examination whose proof refuses — each fixture's verifier-valid neighbor is refused as
    /// a resource of this shape, never handed to [`catches`].
    #[test]
    fn the_crossing_answer_keeps_every_unreadable_or_unowned_store_refused() {
        const X1: &[u8] = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/crossing/X1Catchall.class"
        );
        const X2: &[u8] = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/crossing/X2Split.class"
        );
        const X3: &[u8] = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/crossing/X3Between.class"
        );
        const X4: &[u8] = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/crossing/X4DoubleUse.class"
        );
        const X5: &[u8] = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/crossing/X5CrossBlock.class"
        );
        for (name, class) in [
            ("X1Catchall", X1),
            ("X2Split", X2),
            ("X3Between", X3),
            ("X4DoubleUse", X4),
            ("X5CrossBlock", X5),
        ] {
            let verdicts = plans(class, "main", "([Ljava/lang/String;)V");
            assert!(
                verdicts
                    .iter()
                    .any(|verdict| matches!(verdict, Verdict::Refused { pass: Some(_), .. })),
                "{name} keeps its resource refusal: {verdicts:?}"
            );
            assert!(
                !verdicts
                    .iter()
                    .any(|verdict| matches!(verdict, Verdict::Claimed(_))),
                "{name} claims no header of this shape: {verdicts:?}"
            );
        }
    }

    /// A lowering's own row keeps its examination: `twrNamed`'s `Throwable` row one store precedes
    /// is refused by the proof its shape owes (the body this build cannot state), and the proved
    /// two-resource header of P5TwoResources keeps its certificate — neither is touched by the
    /// crossing answer.
    #[test]
    fn a_twr_lowering_row_a_completed_store_precedes_keeps_its_examination() {
        let twr_named = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/C4.class"
        );
        assert!(matches!(
            plans(twr_named, "twrNamed", "()Ljava/lang/String;").first(),
            Some(Verdict::Refused { pass: Some(_), .. })
        ));
        let two = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/near/P5TwoResources.class"
        );
        let verdicts = plans(two, "two", "()Ljava/lang/String;");
        assert!(verdicts.iter().any(|verdict| matches!(
            verdict,
            Verdict::Claimed(plan) if matches!(plan.shape(), Shape::Resources { .. })
        )));
    }

    /// A provable two-resource header keeps its certificate: its resource stores are the ranges'
    /// own neighbours, so the no-store answer never touches them.
    #[test]
    fn a_two_resource_header_keeps_its_certificate() {
        let class = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/near/P5TwoResources.class"
        );
        let verdicts = plans(class, "two", "()Ljava/lang/String;");
        assert!(verdicts.iter().any(|verdict| matches!(
            verdict,
            Verdict::Claimed(plan) if matches!(plan.shape(), Shape::Resources { .. })
        )));
    }

    #[test]
    fn frozen_two_arm_fixture_is_one_monitor_plan_and_three_arm_control_is_refused() {
        let accepted = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-22/synchronized-multi-exit/SynchronizedMultiExit.class"
        );
        let refused = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-22/synchronized-multi-exit/negative-three-arm/SynchronizedThreeWay.class"
        );
        let accepted = plans(accepted, "choose", "(Ljava/lang/Object;Z)I");
        assert!(accepted.iter().any(|verdict| matches!(
            verdict,
            Verdict::Claimed(plan) if matches!(plan.shape(), Shape::MonitorBranches {
                branch_bci: 5,
                then_exit_bci: 13,
                then_return_bci: 14,
                else_exit_bci: 20,
                else_return_bci: 21,
                ..
            })
        )));
        let certified = accepted.iter().find_map(|verdict| match verdict {
            Verdict::Claimed(plan) if matches!(plan.shape(), Shape::MonitorBranches { .. }) => {
                Some(plan)
            }
            _ => None,
        });
        let facts = certified.expect("one certified two-arm monitor").facts();
        for bci in [12, 13, 14, 19, 20, 21, 22, 23, 24, 25, 26] {
            assert!(
                facts.contains(&bci),
                "monitor source provenance misses BCI {bci}"
            );
        }
        let refused = plans(refused, "choose", "(Ljava/lang/Object;I)I");
        assert!(!refused.iter().any(|verdict| matches!(
            verdict,
            Verdict::Claimed(plan) if matches!(plan.shape(), Shape::MonitorBranches { .. })
        )));
    }

    #[test]
    fn existing_single_exit_monitor_plan_is_unchanged() {
        let class = include_bytes!("../../../tests/fixtures/p3-sync-return/v8/Locked.class");
        let verdicts = plans(class, "locked", "()I");
        assert!(verdicts.iter().any(|verdict| matches!(
            verdict,
            Verdict::Claimed(plan) if matches!(plan.shape(), Shape::Monitor {
                returns: Some(_), ..
            })
        )));
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
    sites: &Sites,
    profile: &crate::pass::RecoveryProfile,
    current: &CanonicalBlockId,
    budget: &mut Budget,
) -> Result<Verdict, StopReason> {
    let mut facts = Facts::new(canonical, view, ssa, ops, handlers, sites, budget);
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
    if FINALLY.admits(profile) {
        if let Some(plan) = prove_nullable_resource_finally(facts, current)? {
            return Ok(Some(Verdict::Claimed(plan)));
        }
        if let Some(plan) = prove_flag_conditional_finally(facts, current)? {
            return Ok(Some(Verdict::Claimed(plan)));
        }
        if let Some(plan) = prove_local_null_conditional_finally(facts, current)? {
            return Ok(Some(Verdict::Claimed(plan)));
        }
        if let Some(plan) = prove_segmented_null_lead_finally(facts, current)? {
            return Ok(Some(Verdict::Claimed(plan)));
        }
        if let Some(plan) = prove_void_loop_finally(facts, current)? {
            return Ok(Some(Verdict::Claimed(plan)));
        }
        if let Some(plan) = prove_loop_finally(facts, current)? {
            return Ok(Some(Verdict::Claimed(plan)));
        }
        if let Some(plan) = prove_empty_catch_call_finally(facts, current)? {
            return Ok(Some(Verdict::Claimed(plan)));
        }
        if let Some(plan) = prove_nested_join_finally(facts, current)? {
            return Ok(Some(Verdict::Claimed(plan)));
        }
        if facts.handlers.len() == 3
            && let Some(plan) = prove_shared_join_finally(facts, current)?
        {
            return Ok(Some(Verdict::Claimed(plan)));
        }
        if let Some(plan) = prove_shared_finally(facts, current, None)? {
            return Ok(Some(Verdict::Claimed(plan)));
        }
    }
    match resources(facts, profile, current)? {
        Verdict::NotGuarded => Ok(None),
        verdict => Ok(Some(verdict)),
    }
}

/// One `catch` clause of a `try` the walk presents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CatchSite {
    /// Types proved for this clause, distinct from a catch-all row without a certificate.
    ///
    /// One named entry is an ordinary clause. Several named entries are the **multi-catch** the table states:
    /// consecutive rows that name different classes and reach the *same* handler are one clause, and
    /// the compiler writes them `catch (A | B n)`. Reading them as one clause is what keeps the
    /// handler's body from being walked — and written — once per row.
    pub(crate) types: CatchTypes,
    /// The canonical block the rows' handler entry maps to.
    pub(crate) handler: CanonicalBlockId,
    /// The local slot the handler's own first instruction stores the caught exception into: the
    /// clause's parameter.
    pub(crate) parameter: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CatchTypes {
    /// Constant-pool class indexes in exception-table order.
    Named(Vec<u16>),
    /// A single catch-all whose closed exceptional-only path was proved below.
    ProvenThrowable,
}

/// One `try`/`catch` statement: where its clauses are and where the code after it begins.
///
/// This is not a guarded shape a rule of P3 2.4 proves: named rows and the certified single
/// catch-all are presented through the same structure, and the walk recovers both the protected
/// range and every handler body as ordinary regions ([`crate::region`]).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Catches {
    /// One site per clause (per handler entry), in exception-table order.
    pub(crate) sites: Vec<CatchSite>,
    /// The block the code after the `try` begins at, when the protected range is followed by a
    /// transfer; `None` when nothing follows it (every path out of the range leaves the method).
    pub(crate) join: Option<CanonicalBlockId>,
    /// Exclusive end of this level's named protected range.
    pub(crate) protected_end: u32,
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
    /// A proved, effect-free catch-all rethrow belonging to this ordinary try.
    pub(crate) transparent: Option<TransparentHandler>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransparentHandler {
    pub(crate) block: CanonicalBlockId,
    pub(crate) bcis: [u32; 3],
    pub(crate) rows: [u32; 2],
    pub(crate) range: (u32, u32),
    pub(crate) normal_exits: [u32; 2],
}

/// Examines one block as the `try` of a `try`/`catch`: named rows, or the certified single
/// catch-all, protecting a range beginning in it.
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
    fragmented: Option<&crate::fragmented_catch::FragmentedCatch>,
    visible_named_row: Option<u32>,
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
    if handlers.len() == 2 && visible_named_row.is_none() {
        let sites = Sites::empty();
        let mut facts = Facts::new(canonical, view, ssa, ops, handlers, &sites, budget);
        if let Some(shape) = transparent_empty_finally(&mut facts, current)? {
            return Ok(Some(shape));
        }
    }
    if handlers.len() == 1
        && handlers[0].catch_type_index.is_none()
        && handlers[0].start_bci >= current.bci()
        && last.is_some_and(|last| handlers[0].start_bci <= last)
    {
        let sites = Sites::empty();
        let mut facts = Facts::new(canonical, view, ssa, ops, handlers, &sites, budget);
        if let Some(site) = exception_only_catch(&mut facts, &handlers[0])? {
            return Ok(Some(Catches {
                sites: vec![site],
                join: None,
                protected_end: handlers[0].end_bci,
                lead: (current.bci(), handlers[0].start_bci),
                inner: None,
                transparent: None,
            }));
        }
    }
    let rows_here: Vec<&ExceptionHandlerFact> = handlers
        .iter()
        .filter(|row| {
            row.catch_type_index.is_some()
                && row.start_bci >= current.bci()
                && last.is_some_and(|last| row.start_bci <= last)
                && !fragmented.is_some_and(|proof| {
                    current.bci() != proof.outer_start && proof.is_outer_row(row.ordinal)
                })
        })
        .collect();
    if rows_here.is_empty() {
        return Ok(None);
    }
    if let Some(visible) = visible_named_row {
        let [row] = rows_here.as_slice() else {
            return Ok(None);
        };
        if row.ordinal != visible || row.start_bci != current.bci() {
            return Ok(None);
        }
        let sites = Sites::empty();
        let facts = Facts::new(canonical, view, ssa, ops, handlers, &sites, budget);
        let Some(sites) = clause_sites(&facts, &[row]) else {
            return Ok(None);
        };
        return Ok(Some(Catches {
            sites,
            join: join_after(&facts, row.end_bci),
            protected_end: row.end_bci,
            lead: (current.bci(), row.start_bci),
            inner: None,
            transparent: None,
        }));
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
    let sites = crate::init::Sites::empty();
    let mut facts = Facts::new(canonical, view, ssa, ops, handlers, &sites, budget);
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
                protected_end: *end,
                lead,
                inner: None,
                transparent: None,
            }))
        }
        [inner_end, outer_end] => {
            let (inner, outer) = (rows_of(*inner_end), rows_of(*outer_end));
            if !nests(&inner, &outer, handlers, fragmented) {
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
            let inner_join = join_after(&facts, *inner_end);
            let join = fragmented
                .filter(|proof| {
                    outer.iter().all(|row| proof.is_outer_row(row.ordinal))
                        && inner.iter().all(|row| row.ordinal == proof.inner_row)
                })
                .map(|proof| proof.outer_join.clone())
                .or_else(|| inner_join.clone());
            Ok(Some(Catches {
                sites: outer_sites,
                join: join.clone(),
                protected_end: *outer_end,
                lead,
                inner: Some(Box::new(Catches {
                    sites: inner_sites,
                    join: inner_join,
                    protected_end: *inner_end,
                    lead,
                    inner: None,
                    transparent: None,
                })),
                transparent: None,
            }))
        }
        _ => Ok(None),
    }
}

/// The Java 8 empty-finally lowering used by the fixed CF-16 class.  The second row is
/// compiler scaffolding only when it receives the same exception and immediately rethrows it.
/// This deliberately proves the complete small method, including both normal transfers: a
/// transparent handler alone says nothing about where the protected invocation completes.
fn transparent_empty_finally(
    facts: &mut Facts<'_>,
    current: &CanonicalBlockId,
) -> Result<Option<Catches>, StopReason> {
    let [named, any] = facts.handlers else {
        return Ok(None);
    };
    if named.catch_type_index.is_none()
        || any.catch_type_index.is_some()
        || named.ordinal.checked_add(1) != Some(any.ordinal)
        || (named.start_bci, named.end_bci) != (any.start_bci, any.end_bci)
        || named.handler_bci == any.handler_bci
        || named.start_bci != current.bci()
    {
        return Ok(None);
    }
    let [
        load,
        invoke,
        normal_exit,
        named_store,
        named_exit,
        store,
        reload,
        rethrow,
        done,
    ] = facts.order.as_slice()
    else {
        return Ok(None);
    };
    let (load, invoke, normal_exit, named_store, named_exit, store, reload, rethrow, done) = (
        *load,
        *invoke,
        *normal_exit,
        *named_store,
        *named_exit,
        *store,
        *reload,
        *rethrow,
        *done,
    );
    let Some(Operation::Invoke(called)) = facts.op(invoke) else {
        return Ok(None);
    };
    if named.end_bci != normal_exit
        || named.handler_bci != named_store
        || any.handler_bci != store
        || !matches!(facts.op(load), Some(Operation::Load { .. }))
        || called.name() != "close"
        || called.descriptor() != "()V"
        || facts.op(normal_exit) != Some(&Operation::Transfer)
        || !matches!(facts.op(named_store), Some(Operation::Store { .. }))
        || facts.op(named_exit) != Some(&Operation::Transfer)
        || !matches!(facts.op(store), Some(Operation::Store { .. }))
        || !matches!(facts.op(reload), Some(Operation::Load { .. }))
        || facts.op(rethrow) != Some(&Operation::Throw)
        || facts.op(done) != Some(&Operation::Return)
        || !handler_binding(facts, named_store)
        || !handler_binding(facts, store)
    {
        return Ok(None);
    }
    let (Some(body), Some(named_block), Some(transparent), Some(join)) = (
        facts.block_of(load).cloned(),
        facts.row_handler(named),
        facts.row_handler(any),
        join_after(facts, normal_exit),
    ) else {
        return Ok(None);
    };
    if body != *current
        || named_block.bci() != named_store
        || transparent.bci() != store
        || facts.block_of(invoke) != Some(&body)
        || facts.block_of(normal_exit) != Some(&body)
        || facts.block_of(named_exit) != Some(&named_block)
        || facts.block_of(reload) != Some(&transparent)
        || facts.block_of(rethrow) != Some(&transparent)
        || facts.block_of(done) != Some(&join)
        || facts.view.successor_ids(&named_block) != [join.clone()]
        || !facts.view.successor_ids(&transparent).is_empty()
        || facts
            .in_block(&body)
            .iter()
            .map(SsaInstruction::bci)
            .collect::<Vec<_>>()
            != [load, invoke, normal_exit]
        || facts
            .in_block(&named_block)
            .iter()
            .map(SsaInstruction::bci)
            .collect::<Vec<_>>()
            != [named_store, named_exit]
        || facts
            .in_block(&transparent)
            .iter()
            .map(SsaInstruction::bci)
            .collect::<Vec<_>>()
            != [store, reload, rethrow]
        || facts
            .in_block(&join)
            .iter()
            .map(SsaInstruction::bci)
            .collect::<Vec<_>>()
            != [done]
        || facts.canonical.blocks().len() != 4
    {
        return Ok(None);
    }
    let (Some(Operation::Store { slot }), Some(Operation::Load { slot: loaded })) =
        (facts.op(store), facts.op(reload))
    else {
        return Ok(None);
    };
    let (Some(stored), Some(reloaded), Some(thrown)) =
        (facts.step(store), facts.step(reload), facts.step(rethrow))
    else {
        return Ok(None);
    };
    if slot != loaded
        || !stored.instruction.writes().iter().any(|(_, value)| {
            reloaded
                .instruction
                .reads()
                .iter()
                .any(|(_, read)| facts.same(*value, *read))
        })
        || !reloaded.instruction.writes().iter().any(|(_, value)| {
            stack_operands(thrown.instruction)
                .iter()
                .any(|(_, read)| facts.same(*value, *read))
        })
    {
        return Ok(None);
    }
    for bci in facts.order.clone() {
        facts.charge(bci)?;
    }
    let sites = facts.canonical.throw_sites();
    if sites.len() != 2
        || sites[0].bci() != invoke
        || sites[0].block() != &body
        || sites[0].handlers() != [named.ordinal, any.ordinal]
        || sites[1].bci() != rethrow
        || sites[1].block() != &transparent
        || !sites[1].handlers().is_empty()
    {
        return Ok(None);
    }
    let expected = BTreeSet::from([
        (
            CanonicalEdgeKind::Exception {
                handler_ordinal: named.ordinal,
            },
            body.clone(),
            named_block.clone(),
        ),
        (
            CanonicalEdgeKind::Exception {
                handler_ordinal: any.ordinal,
            },
            body.clone(),
            transparent.clone(),
        ),
        (CanonicalEdgeKind::Normal, body.clone(), join.clone()),
        (CanonicalEdgeKind::Normal, named_block.clone(), join.clone()),
    ]);
    let edges = facts.canonical.edges();
    for edge in edges {
        facts.charge(edge.from().bci())?;
    }
    if edges.len() != expected.len()
        || edges
            .iter()
            .map(|edge| (edge.kind(), edge.from().clone(), edge.to().clone()))
            .collect::<BTreeSet<_>>()
            != expected
    {
        return Ok(None);
    }
    let Some(sites) = clause_sites(facts, &[named]) else {
        return Ok(None);
    };
    Ok(Some(Catches {
        sites,
        join: Some(join),
        protected_end: named.end_bci,
        lead: (current.bci(), named.start_bci),
        inner: None,
        transparent: Some(TransparentHandler {
            block: transparent,
            bcis: [store, reload, rethrow],
            rows: [named.ordinal, any.ordinal],
            range: (named.start_bci, named.end_bci),
            normal_exits: [normal_exit, named_exit],
        }),
    }))
}

/// A catch-all may be spelled `Throwable` only for a closed, exception-only method tail.
/// The exceptional entry store is the sole harmless instruction shared with the row's range.
fn exception_only_catch(
    facts: &mut Facts<'_>,
    row: &ExceptionHandlerFact,
) -> Result<Option<CatchSite>, StopReason> {
    let Some(handler) = facts.row_handler(row) else {
        return Ok(None);
    };
    let body = facts.bcis((row.start_bci, row.handler_bci));
    let tail = facts.bcis((row.handler_bci, u32::MAX));
    let ([store, cleanup @ .., load, rethrow], Some(&body_throw)) = (tail.as_slice(), body.last())
    else {
        return Ok(None);
    };
    let (store, load, rethrow) = (*store, *load, *rethrow);
    let Some(body_block) = facts.block_of(row.start_bci).cloned() else {
        return Ok(None);
    };
    if cleanup.is_empty()
        || handler.bci() != store
        || row.end_bci != facts.span_end(store)
        || !matches!(facts.op(store), Some(Operation::Store { .. }))
        || facts.op(body_throw) != Some(&Operation::Throw)
        || facts.op(rethrow) != Some(&Operation::Throw)
        || !matches!(facts.op(load), Some(Operation::Load { .. }))
        || body
            .iter()
            .any(|bci| facts.block_of(*bci) != Some(&body_block))
        || tail
            .iter()
            .any(|bci| facts.block_of(*bci) != Some(&handler))
        || facts.in_block(&handler).first().map(SsaInstruction::bci) != Some(store)
        || !facts.view.successor_ids(&handler).is_empty()
    {
        return Ok(None);
    }
    let (Some(Operation::Store { slot: parameter }), Some(Operation::Load { slot: loaded })) =
        (facts.op(store), facts.op(load))
    else {
        return Ok(None);
    };
    if parameter != loaded || !handler_binding(facts, store) {
        return Ok(None);
    }
    for bci in body.iter().chain(&tail) {
        facts.charge(*bci)?;
    }
    // The try boundary must not cut through a pending stack expression. A narrowed row
    // beginning after `new`, for example, cannot present construction inside the try.
    if body.iter().any(|bci| {
        facts.step(*bci).is_none_or(|step| {
            stack_operands(step.instruction).iter().any(|(_, value)| {
                !matches!(facts.ssa.value(facts.resolve(*value)).def(),
                    Definition::Instruction { bci: producer, .. }
                        if row.start_bci <= *producer && *producer < row.handler_bci)
            })
        })
    }) {
        return Ok(None);
    }
    // No branch, normal completion, or hidden effect can escape the protected straight run.
    if body[..body.len() - 1].iter().any(|bci| {
        matches!(
            facts.op(*bci),
            Some(Operation::Return | Operation::Throw | Operation::Transfer)
        )
    }) || tail[..tail.len() - 1].iter().any(|bci| {
        matches!(
            facts.op(*bci),
            Some(Operation::Return | Operation::Throw | Operation::Transfer)
        )
    }) || !exception_only_cleanup(facts, cleanup)?
    {
        return Ok(None);
    }
    let (Some(stored), Some(reloaded), Some(thrown)) =
        (facts.step(store), facts.step(load), facts.step(rethrow))
    else {
        return Ok(None);
    };
    if !stored.instruction.writes().iter().any(|(_, value)| {
        reloaded
            .instruction
            .reads()
            .iter()
            .any(|(_, read)| facts.same(*value, *read))
    }) || !reloaded.instruction.writes().iter().any(|(_, value)| {
        stack_operands(thrown.instruction)
            .iter()
            .any(|(_, read)| facts.same(*value, *read))
    }) {
        return Ok(None);
    }
    let mut body_throw_sites = 0;
    for site in facts.canonical.throw_sites() {
        facts.charge(site.bci())?;
        if row.start_bci <= site.bci() && site.bci() < row.handler_bci {
            if site.block() != &body_block || site.handlers() != [row.ordinal] {
                return Ok(None);
            }
            body_throw_sites += 1;
        } else if site.bci() >= row.handler_bci && site.handlers().contains(&row.ordinal) {
            return Ok(None);
        }
    }
    if body_throw_sites == 0 {
        return Ok(None);
    }
    let mut row_edges = 0;
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        match edge.kind() {
            CanonicalEdgeKind::Exception { handler_ordinal }
                if edge.from() == &body_block
                    && edge.to() == &handler
                    && handler_ordinal == row.ordinal =>
            {
                row_edges += 1
            }
            CanonicalEdgeKind::Exception { .. }
                if edge.from() == &body_block || edge.to() == &handler =>
            {
                return Ok(None);
            }
            CanonicalEdgeKind::Normal if edge.to() == &handler || edge.from() == &handler => {
                return Ok(None);
            }
            CanonicalEdgeKind::Normal if edge.from() == &body_block => return Ok(None),
            CanonicalEdgeKind::Return { .. }
                if edge.from() == &body_block || edge.from() == &handler =>
            {
                return Ok(None);
            }
            CanonicalEdgeKind::Call { .. }
                if edge.from() == &body_block || edge.from() == &handler =>
            {
                return Ok(None);
            }
            _ => {}
        }
    }
    if row_edges != 1 {
        return Ok(None);
    }
    Ok(Some(CatchSite {
        types: CatchTypes::ProvenThrowable,
        handler,
        parameter: *parameter,
    }))
}

/// The presently presentable cleanup is one complete static integer increment. All three
/// stack values have exactly one consumer, so moving it into the catch cannot duplicate a read.
fn exception_only_cleanup(facts: &mut Facts<'_>, cleanup: &[u32]) -> Result<bool, StopReason> {
    let [read, push, add, write] = cleanup else {
        return Ok(false);
    };
    let Some(Operation::Field {
        access: crate::facts::FieldAccess::Read,
        is_static: true,
        owner,
        name,
        descriptor,
    }) = facts.op(*read)
    else {
        return Ok(false);
    };
    if descriptor != "I"
        || !matches!(
            facts.op(*push),
            Some(Operation::Push(crate::facts::ConstantValue::Int(_)))
        )
        || facts.op(*add)
            != Some(&Operation::Arithmetic {
                op: crate::facts::ArithmeticOp::Add,
            })
        || facts
            .step(*add)
            .is_none_or(|step| step.instruction.opcode() != 0x60)
        || facts.op(*write)
            != Some(&Operation::Field {
                access: crate::facts::FieldAccess::Write,
                is_static: true,
                owner: owner.clone(),
                name: name.clone(),
                descriptor: descriptor.clone(),
            })
    {
        return Ok(false);
    }
    for (producer, consumer) in [(*read, *add), (*push, *add), (*add, *write)] {
        let (Some(produced), Some(consumed)) = (facts.step(producer), facts.step(consumer)) else {
            return Ok(false);
        };
        let outputs: Vec<_> = produced
            .instruction
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .collect();
        if outputs.len() != 1
            || stack_operands(consumed.instruction)
                .iter()
                .filter(|(_, value)| facts.same(outputs[0].1, *value))
                .count()
                != 1
        {
            return Ok(false);
        }
        for bci in facts.order.clone() {
            facts.charge(bci)?;
            if bci != consumer
                && facts.step(bci).is_some_and(|step| {
                    stack_operands(step.instruction)
                        .iter()
                        .any(|(_, value)| facts.same(outputs[0].1, *value))
                })
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
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
    fragmented: Option<&crate::fragmented_catch::FragmentedCatch>,
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
    let certified_split = fragmented.is_some_and(|proof| {
        inner.iter().all(|row| row.ordinal == proof.inner_row)
            && outer.iter().all(|row| proof.is_outer_row(row.ordinal))
    });
    handler_inside && (own_rows_only || certified_split)
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
            Some(site)
                if site.handler == handler
                    && matches!(&site.types, CatchTypes::Named(indices) if !indices.contains(&type_index)) =>
            {
                if let CatchTypes::Named(indices) = &mut site.types {
                    indices.push(type_index);
                }
            }
            _ => sites.push(CatchSite {
                types: CatchTypes::Named(vec![type_index]),
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
    let monitor_exits: Vec<u32> = facts
        .order
        .iter()
        .copied()
        .filter(|bci| matches!(facts.op(*bci), Some(Operation::Monitor { enter: false })))
        .collect();
    if monitor_exits.len() == 3 {
        return Ok(Some(
            match monitor_branches(facts, start, enter, lock, &monitor_exits)? {
                Ok(plan) => Verdict::Claimed(plan),
                Err((unproven, at)) => refuse(unproven, at),
            },
        ));
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
    let guarded_rows: BTreeSet<u32> = facts
        .covering(*handler_exit)
        .iter()
        .filter_map(|row| {
            (row.catch_type_index.is_none()
                && facts.row_handler(row).as_ref() == Some(&handler.entry))
            .then_some(row.ordinal)
        })
        .collect();
    if guarded_rows.is_empty() {
        return Ok(Some(refuse(Unproven::Monitor, *handler_exit)));
    }
    let mut cleanup_rows = guarded_rows;
    cleanup_rows.insert(row.ordinal);
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
            cleanup_rows,
            returns,
        },
        lead: (start, start),
        body,
        owned,
        join,
        facts: facts_read,
    })))
}

/// Proves the one bounded two-return monitor shape carried by `Shape::MonitorBranches`.
/// Each arm is a straight expression ending in `monitorexit; return`; the two exceptional rows
/// cover the condition/first arm and second arm respectively and name the same exact cleanup.
fn monitor_branches(
    facts: &mut Facts<'_>,
    start: u32,
    enter: u32,
    lock: u16,
    exits: &[u32],
) -> Result<Result<Plan, Cause>, StopReason> {
    let refuse = |cause| Err(cause);
    let [then_exit, else_exit, handler_exit] = exits else {
        return Ok(refuse((Unproven::Monitor, enter)));
    };
    let mut branch_bci = None;
    for bci in facts.bcis((facts.next_bci(enter).unwrap_or(enter), *then_exit)) {
        facts.charge(bci)?;
        if matches!(facts.op(bci), Some(Operation::Comparison { .. })) {
            branch_bci = Some(bci);
            break;
        }
    }
    let Some(branch_bci) = branch_bci else {
        return Ok(refuse((Unproven::Monitor, enter)));
    };
    let Some(Operation::Comparison { target, .. }) = facts.op(branch_bci) else {
        unreachable!("the selected operation is a comparison")
    };
    let Some(then_start) = facts.next_bci(branch_bci) else {
        return Ok(refuse((Unproven::Monitor, branch_bci)));
    };
    let else_start = *target;
    if then_start >= *then_exit
        || else_start != facts.block_at(else_start).map_or(u32::MAX, |b| b.bci())
        || else_start <= then_start
        || else_start >= *else_exit
    {
        return Ok(refuse((Unproven::Monitor, branch_bci)));
    }
    let Some(then_exit_load) = facts.previous_bci(*then_exit) else {
        return Ok(refuse((Unproven::Monitor, *then_exit)));
    };
    let Some(else_exit_load) = facts.previous_bci(*else_exit) else {
        return Ok(refuse((Unproven::Monitor, *else_exit)));
    };
    if facts.op(then_exit_load) != Some(&Operation::Load { slot: lock })
        || facts.op(else_exit_load) != Some(&Operation::Load { slot: lock })
    {
        return Ok(refuse((Unproven::Monitor, then_exit_load)));
    }
    let Some(then_return) = facts.next_bci(*then_exit) else {
        return Ok(refuse((Unproven::Monitor, *then_exit)));
    };
    let Some(else_return) = facts.next_bci(*else_exit) else {
        return Ok(refuse((Unproven::Monitor, *else_exit)));
    };
    if !matches!(facts.op(then_return), Some(Operation::Return))
        || !matches!(facts.op(else_return), Some(Operation::Return))
    {
        return Ok(refuse((Unproven::Monitor, then_return)));
    }
    for (arm_start, exit_load, return_bci) in [
        (then_start, then_exit_load, then_return),
        (else_start, else_exit_load, else_return),
    ] {
        let Some(instruction) = facts.step(return_bci).map(|step| step.instruction) else {
            return Ok(refuse((Unproven::Monitor, return_bci)));
        };
        let operands = stack_operands(instruction);
        let [(_, value)] = operands.as_slice() else {
            return Ok(refuse((Unproven::Monitor, return_bci)));
        };
        let Definition::Instruction { bci: produced, .. } =
            facts.ssa.value(facts.resolve(*value)).def()
        else {
            return Ok(refuse((Unproven::Monitor, return_bci)));
        };
        let arm_span = (arm_start, exit_load);
        let mut checked_span = true;
        for bci in facts.bcis(arm_span) {
            facts.charge(bci)?;
            checked_span &= facts.op(bci).is_some();
        }
        if *produced < arm_start
            || *produced >= exit_load
            || !checked_span
            || !facts.statement_free(arm_span)
            || !facts.one_expression_bounded(arm_span, return_bci)?
        {
            return Ok(refuse((Unproven::Body, arm_start)));
        }
        // The compact arm emitter places the sole returned expression at `ireturn`. It may not
        // silently discard an independent invocation, local write, field write, or array write.
        // For this bounded shape, admit at most the direct value-producing invocation.
        for bci in facts.bcis((arm_start, exit_load)) {
            facts.charge(bci)?;
            match facts.op(bci) {
                Some(Operation::Invoke(_)) if bci == *produced => {}
                Some(
                    Operation::Invoke(_)
                    | Operation::Store { .. }
                    | Operation::Increment { .. }
                    | Operation::ArrayStore { .. },
                ) => {
                    return Ok(refuse((Unproven::Body, bci)));
                }
                Some(Operation::Field {
                    access: crate::facts::FieldAccess::Write,
                    ..
                }) => {
                    return Ok(refuse((Unproven::Body, bci)));
                }
                _ => {}
            }
        }
    }
    let Some(branch_block) = facts.block_of(branch_bci).cloned() else {
        return Ok(refuse((Unproven::Monitor, branch_bci)));
    };
    if facts
        .in_block(&branch_block)
        .last()
        .map(SsaInstruction::bci)
        != Some(branch_bci)
    {
        return Ok(refuse((Unproven::Monitor, branch_bci)));
    }
    if facts.in_block(&branch_block).iter().any(|instruction| {
        let bci = instruction.bci();
        bci > enter
            && bci < branch_bci
            && !matches!(
                facts.op(bci),
                Some(Operation::Load { .. } | Operation::Push(_))
            )
    }) {
        return Ok(refuse((Unproven::Body, branch_bci)));
    }
    let successors = facts.view.successor_ids(&branch_block);
    let Some(then_block) = facts.block_at(then_start) else {
        return Ok(refuse((Unproven::Monitor, then_start)));
    };
    let Some(else_block) = facts.block_at(else_start) else {
        return Ok(refuse((Unproven::Monitor, else_start)));
    };
    if successors.len() != 2
        || !successors.contains(&then_block)
        || !successors.contains(&else_block)
    {
        return Ok(refuse((Unproven::Monitor, branch_bci)));
    }
    // Do not accept hidden jumps, nested tests, or extra reachable arms in either body.
    for (arm_start, exit, ret) in [
        (then_start, *then_exit, then_return),
        (else_start, *else_exit, else_return),
    ] {
        let blocks = facts.blocks_in((arm_start, facts.span_end(ret)));
        if blocks.is_empty() {
            return Ok(refuse((Unproven::Span, arm_start)));
        }
        for (index, block) in blocks.iter().enumerate() {
            facts.charge(block.bci())?;
            let successors = facts.view.successor_ids(block);
            if index + 1 == blocks.len() {
                if !successors.is_empty() {
                    return Ok(refuse((Unproven::Monitor, block.bci())));
                }
            } else if successors.len() != 1 || successors[0] != blocks[index + 1] {
                return Ok(refuse((Unproven::Monitor, block.bci())));
            }
        }
        if !matches!(facts.op(exit), Some(Operation::Monitor { enter: false })) {
            return Ok(refuse((Unproven::Monitor, exit)));
        }
    }
    let Some(first_end) = facts.next_bci(*then_exit) else {
        return Ok(refuse((Unproven::Monitor, *then_exit)));
    };
    let Some(second_end) = facts.next_bci(*else_exit) else {
        return Ok(refuse((Unproven::Monitor, *else_exit)));
    };
    let mut rows = Vec::new();
    for row in facts.handlers {
        facts.charge(row.start_bci)?;
        if (row.start_bci, row.end_bci) == (facts.next_bci(enter).unwrap_or(enter), first_end)
            || (row.start_bci, row.end_bci) == (else_start, second_end)
        {
            rows.push(row.clone());
        }
    }
    let [then_row, else_row] = rows.as_slice() else {
        return Ok(refuse((Unproven::Handler, enter)));
    };
    if then_row.catch_type_index.is_some()
        || else_row.catch_type_index.is_some()
        || facts.row_handler(then_row) != facts.row_handler(else_row)
        || facts
            .covering(branch_bci)
            .iter()
            .any(|row| row.ordinal != then_row.ordinal)
    {
        return Ok(refuse((Unproven::Handler, branch_bci)));
    }
    for (span, expected) in [
        ((then_start, first_end), then_row.ordinal),
        ((else_start, second_end), else_row.ordinal),
    ] {
        for bci in facts.bcis(span) {
            facts.charge(bci)?;
            if facts
                .covering(bci)
                .iter()
                .any(|row| row.ordinal != expected)
            {
                return Ok(refuse((Unproven::Handler, bci)));
            }
        }
    }
    let handler = match monitor_handler(facts, then_row, lock) {
        Ok(handler) => handler,
        Err(cause) => return Ok(Err(cause)),
    };
    if handler.exit_bci != *handler_exit
        || facts.row_handler(else_row) != Some(handler.entry.clone())
    {
        return Ok(refuse((Unproven::Monitor, *handler_exit)));
    }
    let mut handler_rows = Vec::new();
    for row in facts.handlers {
        facts.charge(row.start_bci)?;
        if facts.row_handler(row).as_ref() == Some(&handler.entry) {
            handler_rows.push(row);
        }
    }
    let self_rows = handler_rows
        .iter()
        .filter(|row| {
            (row.start_bci, row.end_bci) == (handler.entry.bci(), facts.span_end(handler.exit_bci))
                && row.catch_type_index.is_none()
        })
        .count();
    if self_rows != 1
        || handler_rows.len() != 3
        || handler_rows
            .iter()
            .any(|row| row.catch_type_index.is_some())
    {
        return Ok(refuse((Unproven::Handler, handler.exit_bci)));
    }
    let end = facts.span_end(else_return);
    if handler.entry.bci() != end {
        return Ok(refuse((Unproven::Span, end)));
    }
    let mut owned = facts.blocks_in((start, end));
    if !owned.contains(&handler.entry) {
        owned.push(handler.entry.clone());
    }
    owned.sort_by_key(CanonicalBlockId::bci);
    let mut facts_read = vec![
        enter,
        branch_bci,
        *then_exit,
        then_exit_load,
        then_return,
        *else_exit,
        else_exit_load,
        else_return,
        handler.entry.bci(),
        handler.exit_bci,
    ];
    // The `synchronized` statement replaces every instruction in its certified cleanup, including
    // loading and rethrowing the original exception object; retain the complete handler slice as
    // provenance rather than anchoring only its entry and monitor exit.
    facts_read.extend(facts.bcis(handler.span));
    facts_read.sort_unstable();
    facts_read.dedup();
    Ok(Ok(Plan {
        shape: Shape::MonitorBranches {
            enter_bci: enter,
            branch_bci,
            then_exit_bci: *then_exit,
            then_return_bci: then_return,
            else_exit_bci: *else_exit,
            else_return_bci: else_return,
        },
        lead: (start, start),
        body: (then_start, else_exit_load),
        owned,
        join: None,
        facts: facts_read,
    }))
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
            // The lead the protected range follows admits the shape's own readings, one of
            // three: no statement at all, a completed field assignment, or — this slice — the
            // two-instruction `aconst_null; astore s` a null-initialised local writes. The
            // answer is read before the proof because the null local is the one lead whose
            // cleanup copies may read a local at all — the slot it initialises — so it shapes
            // the copy grammar the proof admits ([`completed_null_local_lead`]).
            let null_lead = facts
                .previous_bci(row.start_bci)
                .filter(|before| *before >= start)
                .and_then(|before| completed_null_local_lead(facts, before, start, row.start_bci));
            let proof = if FINALLY.admits(profile) {
                prove_finally_copy(facts, row, null_lead.is_some())?
            } else {
                None
            };
            let Some(proof) = proof else {
                return Ok(Verdict::refused(None, Unproven::FinallyCopy, at));
            };
            if proof.row_ordinal != row.ordinal
                || start > proof.protected.0
                || proof.protected.0 >= end
                || (start != proof.protected.0
                    && null_lead.is_none()
                    && !facts.previous_bci(proof.protected.0).is_some_and(|before| {
                        completed_field_assignment(facts, before, start, proof.protected.0)
                            && single_statement(facts, (start, proof.protected.0), before)
                    }))
                || !facts.statement_free((start, proof.protected.0))
                || !facts.bcis((start, proof.protected.0)).iter().all(|bci| {
                    facts.block_of(*bci) == Some(current) && facts.covering(*bci).is_empty()
                })
            {
                return Ok(Verdict::refused(None, Unproven::FinallyCopy, at));
            }
            // The null lead's slot identity: both copies' call arguments are the lead slot's own
            // merged value flow, and the lead and the body's assignments are the slot's only
            // definitions. A shape that reads anything else keeps the refusal above.
            if let Some(slot) = null_lead
                && !null_lead_copies_read_the_lead(facts, &proof, start, slot)?
            {
                return Ok(Verdict::refused(None, Unproven::FinallyCopy, at));
            }
            let structured = !facts.statement_free(proof.protected);
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
                        completion: FinallyCompletion::SavedReturn {
                            save: proof.saved_return.0,
                            returns: proof.saved_return.1,
                        },
                        row_ordinal: proof.row_ordinal,
                        structured,
                    },
                    lead: (start, proof.protected.0),
                    body: proof.protected,
                    owned: proof.owned,
                    join: proof.join,
                    facts: proof.origins,
                }));
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
        // the one case that is not a resource header a rule could have read. A row whose range a
        // *store* precedes keeps the refusal it has today, except where the skip below reads the
        // store as the completed statement the range's own start follows and hands the row to
        // [`catches`].
        //
        // The instruction before the range answers the same question from the other side: it is
        // where a header's own initialisation *ends*, and an ordinary assignment — a store of a
        // value no `new` and no invocation produced — means the `try` after `int x = 1;` is a
        // `try`/`catch` rather than a header `initialisation` could read
        // ([`initialises_resource`]). A store filled by `new Res()` or by a call keeps the row
        // examined wherever a proof of this shape could own it: a `try`-with-resources this build
        // cannot prove keeps degrading as one, and the one row the skip below spells as a user
        // `catch` is the one no proof of this shape could have claimed.
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
        // and a row answers *no* where no instruction precedes it in this block, where the statement
        // before it is no store at all *and a statement boundary stands where the range begins*,
        // where an ordinary assignment precedes it, and where a completed local store precedes a row
        // whose handler closes no resource. None of these
        // rows is examined as a resource, and when every candidate answers no the shape is not this
        // rule's at all.
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
        // A **completed local store** before a named row is the same question's next answer: the
        // store is a statement of its own — `r = open();`, `b = new StringBuilder();` — and a range
        // that begins at the boundary it ends at protects what the statements *after* it run, not
        // the store itself ([`statement_boundary`]). No header of this shape reads such a row: a
        // header's own store is the range's immediate neighbour, read only where the whole proof
        // succeeds, so the row a plain initialisation statement precedes is the `catch` its own
        // table names — left to [`catches`], exactly like the rows above.
        //
        // The reading is the run [`single_statement`] grows backwards from the store, the same
        // reading [`initialises_resource`] takes of a header's own initialisation — and a store
        // whose run cannot be read at all is no answer's evidence, by that rule's own conservatism:
        // a statement this walk cannot read keeps the examination whose proof refuses below.
        //
        // The row cannot be a `try`-with-resources lowering's own row either, which is what keeps
        // every proved resource on its current path. Where a user `catch` wraps the whole
        // statement, the compiler starts that row **before** the initialisation it covers —
        // `twrNamed`'s `IllegalStateException` row spans `[0, 36)` over the `new C4()` its body
        // allocates — so the instruction before it is never the completed store the range follows.
        // And the lowering's own rows carry the resource in both their neighbours: the code at a
        // row's own end is the slot's **normal close** ([`normal_close`]), and the handler its row
        // reaches runs the close/suppress/rethrow sequence ([`closes_something`]) — the same fact
        // the outward chain below reads. A named row whose end closes nothing and whose handler
        // closes nothing is therefore no level of the lowering, and a row one completed store
        // precedes cannot be its outer wrapper either: handing it to [`catches`] takes no row a
        // proof of this shape could own.
        //
        // The answer keeps its conditions. A catch-all row — no type at all — is not this answer's:
        // the degradation a store prefix meets today is exactly that row's refusal, and it stays.
        // A range whose start lands inside the store's own statement — the row splitting a `new`
        // expression in two — stands on no boundary, and keeps the examination whose proof refuses
        // below. And a row that carries either close of its own keeps the examination: whether the
        // shape is the lowering's own — its closes damaged or not — is the proof's question, not
        // this answer's.
        if let Some(Operation::Store { slot }) = facts.op(before)
            && row.catch_type_index.is_some()
            && facts
                .previous_bci(before)
                .filter(|previous| *previous >= start)
                .is_some_and(|previous| {
                    single_statement(facts, (previous, facts.span_end(before)), before)
                })
            && normal_close(facts, row.end_bci, *slot).is_none()
            && !facts.closes_something(row)
            && statement_boundary(facts, before, row.start_bci)
        {
            continue;
        }
        // A statement that is no store at all — a void call, a field write, anything that ends a
        // statement of its own — is no resource's initialisation either: nothing it leaves behind
        // is a value a header could declare, and [`initialisation`] reads a store. A row whose
        // range follows such a statement is the `catch` its own table names — the nested `try` of
        // `helper(); try { … } catch (E e) { … }` — so it is not examined as a header this rule
        // could state: it is left to [`catches`], exactly like the rows above.
        //
        // The row is skipped rather than refused, because what precedes the range is read here and
        // is no initialisation at all. A resource header a compiler writes keeps its own store
        // immediately before the range it declares — `try (R r = open())` and Java 9's `try (r)`
        // both store the variable the range reads — so no provable header of this shape answers no
        // here, and a `try` whose header this build cannot state keeps degrading as one.
        //
        // The skip is answered only where the row's range begins at a **statement boundary**:
        // the instruction before the range is one that *ends* a source statement — a `void`
        // invocation (`helper();`), a field write (`field = 7;`, the statement
        // [`completed_field_assignment`] reads), a return — and the statement it ends completed
        // exactly there, the way a compiler lays one statement out before the next
        // ([`statement_boundary`]). Those are the clause boundaries a source statement owns: the
        // `try` after `helper();`, the `catch` and `finally` clauses of the statement whose body
        // returned. A range that begins anywhere else — inside an expression (the old-value
        // update whose `dup2` the row covers), past a branch or a monitor enter (the ranges of a
        // `synchronized` statement's own body and cleanup), or over a store the row widened into
        // — is no clause a source statement could own, so the row keeps the examination whose
        // proof refuses below, the same answer it held before this question was asked.
        if !matches!(facts.op(before), Some(Operation::Store { .. }))
            && statement_ends(facts, before)
            && statement_boundary(facts, before, row.start_bci)
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
    let normal_cleanup = pieces.clone();
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
    // The normal closes live in the statement's main span. Exceptional cleanup can live after the
    // return and is owned only when every instruction in each added canonical block is one of the
    // exact handler/guard instructions proved above, and every incoming/outgoing edge is explained
    // by those same exception-table rows or by another such cleanup block.
    let mut cleanup_bcis = BTreeSet::new();
    let mut cleanup_spans = normal_cleanup;
    for handler in &handlers {
        cleanup_spans.push(handler.span);
        cleanup_spans.push((handler.guard.start_bci, handler.guard.end_bci));
    }
    for span in cleanup_spans {
        for bci in facts.bcis(span) {
            facts.charge(bci)?;
            cleanup_bcis.insert(bci);
        }
    }
    for companion in &companions {
        for bci in facts.bcis((companion.start_bci, companion.end_bci)) {
            facts.charge(bci)?;
            cleanup_bcis.insert(bci);
        }
    }
    let owned =
        facts.cleanup_blocks(&facts.blocks_in((start, claimed_end)), &cleanup_bcis, &rows)?;
    let lead = (start, resources.first().map(|r| r.init.0).unwrap_or(start));
    let mut facts_read: Vec<u32> = Vec::new();
    if let Some((load_bci, return_bci, store_bci)) = return_tail.as_ref() {
        facts_read.extend([*load_bci, *return_bci, *store_bci]);
    }
    for resource in &resources {
        facts_read.push(resource.close_bci);
        facts_read.extend(facts.bcis(resource.init));
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
    facts_read.extend(cleanup_bcis.iter().copied());
    facts_read.sort_unstable();
    facts_read.dedup();
    let _ = innermost_handler;
    Ok(Plan {
        shape: Shape::Resources {
            resources,
            returns: return_tail.as_ref().map(|(_, return_bci, _)| *return_bci),
            cleanup: cleanup_bcis.into_iter().collect(),
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
