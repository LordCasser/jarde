//! P3 1.1: the read-only IR handoff — the tables one method-analysis run published, and the
//! borrows a recovery consumer reads them through.
//!
//! # What this module is
//!
//! The P2 pipeline publishes three IR artifacts for one method body: the canonical CFG (3.5), the
//! frames (4.1) and the names, phis and effect facts (4.3). Until this slice the driver held them
//! in its own run and dropped them when the run ended, so a consumer outside could read the
//! *report* planes — stages, quality, coverage, execution, diagnostics — and nothing else. The
//! recovery layer needs the artifacts themselves, and re-deriving them from a report is not a
//! thing this engine does: [`crate::ir::MethodAnalysisReport`] holds no block, no BCI, no value
//! and no phi, so a payload rebuilt from its fields could only be an invented graph.
//!
//! So the handoff is the payload itself, in one type:
//!
//! * [`MethodIr`] is owned by this crate and holds the three tables **by value** — the boxes the
//!   passes published, moved into the payload, never a copy and never a second derivation;
//! * the tables are read through the borrows [`MethodIr::canonical`], [`MethodIr::frames`] and
//!   [`MethodIr::ssa`] return, and every type they return is published here with a read-only
//!   method surface: no field of the tables is public, no `&mut` is reachable, and no type of the
//!   middle end can be constructed from outside this crate;
//! * a table the run did not publish is `None`. Presence is the phase validity of the payload:
//!   `ssa` is only ever present with the frames it was named over, and the frames only with the
//!   canonical graph they were derived from, exactly as the pass table requires.
//!
//! # The decode facts travel with the tables (P3 1.3b)
//!
//! The three tables state a method's *structure*, not its **symbolic vocabulary**: which local a
//! `*load`/`*store` names, what a `iconst`/`bipush`/`ldc` pushes, whether a branch transfers when
//! its value is zero, which constant-pool reference an `invoke*` names, which keys a
//! `tableswitch` enumerates. That vocabulary is a *decode* fact — it was read from the class bytes
//! once, by the `raw_facts` pass — and a presentation of the body cannot be written without it.
//!
//! Before this slice the recovery layer asked its caller for it instead, through a table the
//! caller built beside the run: two sources for one body, and a wrong entry in the caller's table
//! (an `ifeq` labelled `ifne`) produced silently inverted Java that nothing in the pipeline could
//! have noticed. So the facts the `raw_facts` pass decoded are moved into the payload here, beside
//! the tables of the same run:
//!
//! * [`MethodIr::code`] — the decoded body (its instructions, their typed operands, its declared
//!   exception table), the very `MethodCodeFacts` the passes above read;
//! * [`MethodIr::constant_pool`] — the class's own constant pool as the same read decoded it,
//!   which is where a reference's owner, name and descriptor and an `ldc`'s value live.
//!
//! Both travel with the payload and neither is re-decoded: the body is moved in, and the pool is
//! read through the facts bundle of the very read that produced it, held by handle — one bundle per
//! read, shared by every payload decoded from it, never a copy per method. A consumer therefore has
//! exactly one source for the polarity of a branch, the slot of a load and the value of a constant,
//! and the compiler enforces it — there is no parameter left through which a caller could hand in a
//! second opinion.
//!
//! # The class's bootstrap table travels with them too (P3 2.1)
//!
//! One more fact of the same header read is neither structure nor an operand: the
//! `BootstrapMethods` attribute, which is what says **which** method handle and which static
//! arguments an `invokedynamic` site's `bootstrap_method_attr_index` names. Without it a consumer
//! that holds only the pool can see that a site exists but cannot tell a `LambdaMetafactory` call
//! site from any other dynamic site — and "is this site a lambda at all" is exactly the question
//! A04 forbids answering by pattern-matching the pool.
//!
//! So [`MethodIr::bootstrap_methods`] carries the table the *same* read decoded, in the same
//! attribute the header enumeration already located: the function that reads it
//! ([`jarde_reader::classfile::bootstrap_methods`]) is the one the xref consumer already uses, it
//! is called on this run's own bytes and pool, and its result is moved into the payload. A class
//! that declares no such attribute hands over an empty table, reads nothing and charges nothing —
//! this is a fact layer, not a second decoder.
//!
//! # The declaring class's two facts travel with the member's declaration
//!
//! One more pair of facts of that same header read is not about the member at all: the class's own
//! internal name (`this_class`) and its own access flags. They are what says **which class** declared
//! this member and **what kind** of class that is, and the member's own flags cannot answer either
//! question — a `public` method that is not `abstract` is a `default` method in an interface and an
//! ordinary method in a class, and only `ACC_INTERFACE` on the declaring class tells the two apart.
//!
//! The read that located the member had both facts in hand, so dropping them made a consumer ask its
//! caller for what this run had already read — or, worse, made it refuse a declaration it could have
//! read. They are therefore carried by [`MethodDeclaration`] ([`MethodDeclaration::class_name`],
//! [`MethodDeclaration::class_access_flags`]), in the reader's own byte representation for the name
//! and as the raw flags, bound to the physical definition [`MethodDeclaration::identity`] names. No
//! second read, no second decoder and no second name system: the bytes handed over are the bytes that
//! read parsed.
//!
//! # Ownership, lifetime, and why nothing here is shared
//!
//! * **Who builds it.** [`crate::engine::analyze_method_ir`] does, at the end of one request: the
//!   payload is assembled from the very locals the scheduled passes filled, so it holds the tables
//!   of *that* run and no other.
//! * **Who owns it.** The caller of that entry point, by value, together with the
//!   [`crate::ir::MethodAnalysisReport`] of the same run ([`MethodIrAnalysis`] returns both).
//! * **How long it lives.** As long as the caller keeps it. The tables own their data — no borrow
//!   of the request, of the snapshot bytes or of any reader buffer — so the payload has no
//!   lifetime parameter and no second lifetime to reason about: handing a `&MethodIr` to the
//!   recovery layer is an ordinary borrow bounded by the caller's own scope.
//! * **Who reads it.** Any consumer that holds `&MethodIr`: 1.3's `jarde-java` reads the graph,
//!   the frames, the names and the effects through those borrows and returns before the payload
//!   is dropped.
//! * **Why the decode facts are shared by handle and the tables are not.** The tables of one run
//!   are that run's own: a canonical graph, frames and names are derived values with no second
//!   consumer, and nothing here shares them. The *facts bundle* the body was decoded from is the
//!   opposite case — it is read once (by this request's own header read, or by the preparation of a
//!   class whose methods are analysed one after another) and immutable, so `M` methods of one
//!   prepared class hold `M` handles to one bundle instead of `M` copies of one constant pool and
//!   one member table ([`crate::engine::analyze_prepared_method_ir`], and
//!   [`jarde_reader::prepared::PreparedClass::facts_handle`] for the handle itself). Each payload
//!   still states its own pool ([`MethodIr::constant_pool`]) and its own decoded body; what it does
//!   not do is re-allocate what it did not derive.
//! * **Why no cache and no second lifetime.** One request produces one payload for one consumer in a
//!   synchronous engine, and the sharing above is a lifetime rather than a store: a payload keeps
//!   the read's bundle alive exactly as long as the payload itself lives, and a class task that
//!   prepared a class hands out one handle rather than deciding when a cache entry becomes stale.
//!   Nothing here keeps the tables of a *finished* request alive, and no identity beyond the read a
//!   payload is a payload of enters this seam.
//!
//! # Billing, stops and cancellation do not change here
//!
//! The payload is built from the artifacts the passes already published and charged; handing it
//! over charges nothing, reads nothing and re-runs no pass. The report of the same run keeps
//! stating the requested and scheduled stages, the quality of the produced artifact, the coverage
//! and the one execution the run ended with — a run that stopped under a budget, a cancellation or
//! an unsupported state hands over exactly the tables it published before that stop, and a run
//! whose environment was rejected hands over an empty payload, because no phase ran at all.
//!
//! # What this module deliberately does not publish
//!
//! The middle end's *machinery* stays crate-private: the raw CFG and its effect facts, the
//! `jsr`/`ret` call contexts, the fact ledger and the pass table, the analysis run, and the pass
//! entry points themselves (`canonical_cfg`, `frames` and `ssa`) with their input views. There is
//! no backend trait, no dynamic pass registration and no cross-layer IR abstraction here: one
//! producer (`jarde-jvm`), one consumer (`jarde-java`, P3 1.3), one concrete payload. Publishing
//! more of the middle end is a later decision that needs a consumer, not a shape invented here.

pub use crate::canonical::{
    CanonicalBlock, CanonicalBlockId, CanonicalCfg, CanonicalEdge, CanonicalEdgeKind,
    CanonicalHandlerRow, CanonicalThrowSite,
};
pub use crate::cfg::CfgCompleteness;
pub use crate::frame::{BlockFrame, FrameTable, LogicalInput, NewSite, RefType, Value};
pub use crate::ssa::{
    CanonicalEffectFacts, CanonicalInstructionEffect, Definition, PhiInput, Slot, SsaBlock,
    SsaInstruction, SsaPhi, SsaTable, SsaUse, SsaValue, ValueId,
};

use crate::ir::MethodAnalysisReport;
use jarde_reader::classfile::{BootstrapMethodFacts, ClassFacts, CpEntryFacts, MethodCodeFacts};
use jarde_reader::model::{JvmBytes, PhysicalMethodId};
use std::sync::Arc;

/// The access-flag bit a member sets when it is `static` (JVMS 4.6).
const ACC_STATIC: u16 = 0x0008;

/// What the class file declares about the member whose body this run read (P3 3.1), and about the
/// class that declares it (the declaring-class handoff).
///
/// The `raw_facts` pass locates the member by raw name and descriptor in the **one** header read that
/// also yields the body, and this is what that member's own declaration says — kept in the payload
/// instead of being re-read somewhere else or assumed from the request:
///
/// * its **access flags**, exactly as the class declares them: `static` decides whether slot 0 is a
///   receiver, and a member the class declares `bridge`/`synthetic` is one the presentation may act
///   on;
/// * its **name and descriptor**, as the member the read located states them;
/// * the **parameter slots** its descriptor and flags imply — `this` included for an instance
///   member, and a `long`/`double` parameter taking two slots (JVM 2.6.1). The slots below that
///   count are the signature's, not the body's: they are named as parameters and they are never
///   declared by a body statement;
/// * the **physical identity** the read was performed under, which is what a presentation of this
///   payload is a presentation *of* (P3 3.2 binds origins to it);
/// * the **class that declares the member** — its own internal name (`this_class`) and its own
///   access flags — which is the same read's statement about that definition rather than a second
///   look at it. A member's own flags cannot say whether it is an interface's `default` method or a
///   class's ordinary one ([`Self::class_access_flags`] and [`Self::class_name`] are what can), and
///   those two facts were already in hand when the body was read.
///
/// The two class facts are **parse facts of one physical definition**: they are read from the header
/// of the definition the member was located in, they are bound to [`Self::identity`]'s definition
/// rather than to a name, and they claim nothing beyond what that class file declares — not dialect
/// validity, not a successful runtime resolution, not JVM verification, and nothing about the
/// quality of any body presented from this payload.
///
/// A run that read no member header states no declaration: the payload carries the run's own
/// evidence, so an absent one is absent rather than filled in from the request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodDeclaration {
    access_flags: u16,
    name: JvmBytes,
    descriptor: JvmBytes,
    parameter_slots: u16,
    identity: PhysicalMethodId,
    class_name: JvmBytes,
    class_access_flags: u16,
}

impl MethodDeclaration {
    /// One member's declaration, as the header read that located it states it, together with what
    /// that same header states about the class that declares it.
    ///
    /// `None` when the descriptor is not one this layer can walk: the count of parameter slots is
    /// derived from the descriptor, so a descriptor the format does not allow states no declaration
    /// at all instead of a count nobody could justify. (Such a body is refused by the passes that
    /// read the descriptor anyway.)
    pub(crate) fn new(
        access_flags: u16,
        name: JvmBytes,
        descriptor: JvmBytes,
        identity: PhysicalMethodId,
        class_name: JvmBytes,
        class_access_flags: u16,
    ) -> Option<Self> {
        let parameter_slots = parameter_slots(&descriptor.0, access_flags & ACC_STATIC != 0)?;
        Some(Self {
            access_flags,
            name,
            descriptor,
            parameter_slots,
            identity,
            class_name,
            class_access_flags,
        })
    }

    /// The member's access flags, as the class declares them.
    pub fn access_flags(&self) -> u16 {
        self.access_flags
    }

    /// Whether the class declares the member `static`, which is what says that slot 0 is a
    /// parameter rather than a receiver.
    pub fn is_static(&self) -> bool {
        self.access_flags & ACC_STATIC != 0
    }

    /// The member's name, as its own declaration spells it.
    pub fn name(&self) -> &JvmBytes {
        &self.name
    }

    /// The member's descriptor, as its own declaration spells it.
    pub fn descriptor(&self) -> &JvmBytes {
        &self.descriptor
    }

    /// How many local slots the parameters occupy.
    ///
    /// `this` is counted for an instance member, and a category-2 parameter (`long`/`double`) is
    /// counted as the two slots it fills — so this is the first slot a body of this member may
    /// declare a local in.
    pub fn parameter_slots(&self) -> u16 {
        self.parameter_slots
    }

    /// The physical identity this read was performed under.
    pub fn identity(&self) -> &PhysicalMethodId {
        &self.identity
    }

    /// The internal name of the class that declares the member (`this_class`), exactly as the bytes
    /// of that class file spell it.
    ///
    /// Kept in the reader's own JVM byte representation rather than as text: the name is part of what
    /// this declaration is a declaration *of*, and spelling it for a reader is a display decision of
    /// the consumer. The bytes are the ones the header read that located the member and decoded its
    /// body held — that read's `this_class`, never the request's owner spelling, an archive entry
    /// name, a constant-pool reference or anything found through the host classpath.
    pub fn class_name(&self) -> &JvmBytes {
        &self.class_name
    }

    /// The access flags of that class, exactly as the same header states them (JVMS 4.1).
    ///
    /// This is the fact a member's own flags cannot stand in for: whether the member is an
    /// interface's `default` method or an ordinary method of a class is read from `ACC_INTERFACE` on
    /// the declaring class together with the member's flags. It is a parse fact of one read — not a
    /// verdict about the dialect, a runtime resolution, JVM verification, or the quality of the body
    /// presented beside it.
    pub fn class_access_flags(&self) -> u16 {
        self.class_access_flags
    }
}

/// The local slots one method descriptor's parameters occupy (JVM 2.6.1).
///
/// The walk is the descriptor's own: `this` takes slot 0 when the member is not `static`, and a
/// `long` or `double` parameter takes two slots where every other type takes one. The return
/// descriptor is not read — it occupies no local slot — and `None` means the descriptor is not one
/// this walk can read.
fn parameter_slots(descriptor: &[u8], is_static: bool) -> Option<u16> {
    let mut rest = descriptor.strip_prefix(b"(")?;
    let mut slots = u32::from(!is_static);
    let mut closed = false;
    while let Some((&first, tail)) = rest.split_first() {
        if first == b')' {
            closed = true;
            break;
        }
        rest = match first {
            b'J' | b'D' => {
                slots += 2;
                tail
            }
            b'B' | b'C' | b'F' | b'I' | b'S' | b'Z' => {
                slots += 1;
                tail
            }
            b'L' => {
                let end = tail.iter().position(|byte| *byte == b';')?;
                slots += 1;
                &tail[end + 1..]
            }
            b'[' => {
                // An array parameter is one slot whatever its element type, so the `[`-chain is
                // read only to find where the parameter ends.
                let mut element = tail;
                while let Some((&b'[', rest)) = element.split_first() {
                    element = rest;
                }
                match element.split_first() {
                    Some((&b'L', after)) => {
                        let end = after.iter().position(|byte| *byte == b';')?;
                        element = &after[end + 1..];
                    }
                    Some((&(b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z'), after)) => {
                        element = after;
                    }
                    _ => return None,
                }
                slots += 1;
                element
            }
            _ => return None,
        };
    }
    if !closed {
        return None;
    }
    u16::try_from(slots).ok()
}

/// The IR payload of one method-analysis request: the tables that run published, each present
/// exactly when the pass that produces it published one, together with the decode facts the
/// `raw_facts` pass read them from.
///
/// The payload owns the tables; see the module documentation for who builds it, how long it lives
/// and why it is not shared.
#[derive(Debug)]
pub struct MethodIr {
    canonical: Option<Box<CanonicalCfg>>,
    frames: Option<Box<FrameTable>>,
    ssa: Option<Box<SsaTable>>,
    code: Option<Box<MethodCodeFacts>>,
    /// The facts bundle of the read that decoded [`Self::code`], held as the **shared handle** of
    /// that read ([`jarde_reader::prepared::PreparedClass::facts_handle`] for a prepared class, the
    /// request's own header read otherwise).
    ///
    /// The class's constant pool is read through this handle ([`Self::constant_pool`]) instead of
    /// being copied per payload: `M` methods of one prepared class share **one** pool allocation, its
    /// member table and its attribute shells, and each payload keeps its own tables above it. The
    /// direct path shares the bundle its own single read produced, so one request still produces one
    /// pool.
    facts: Option<Arc<ClassFacts>>,
    bootstrap_methods: Vec<BootstrapMethodFacts>,
    declaration: Option<Box<MethodDeclaration>>,
}

impl MethodIr {
    /// One payload from the artifacts of one run, in the order the passes publish them.
    ///
    /// `code`, `facts` and `bootstrap_methods` are the facts the `raw_facts` pass read: the decoded
    /// body, the read's own facts bundle (whose constant pool [`Self::constant_pool`] reads) and the
    /// class's `BootstrapMethods` table. All three travel with the tables, and all three are present
    /// exactly when the graph is — the graph is built from the decode they came out of, and a class
    /// that declares no bootstrap table states that with an empty one.
    ///
    /// `facts` is the **handle** of the read, not a copy of it: the direct entry hands over the bundle
    /// its one header read produced, and a prepared class hands over the bundle its preparation
    /// produced, which every method of that class shares.
    pub(crate) fn new(
        canonical: Option<Box<CanonicalCfg>>,
        frames: Option<Box<FrameTable>>,
        ssa: Option<Box<SsaTable>>,
        code: Option<Box<MethodCodeFacts>>,
        facts: Option<Arc<ClassFacts>>,
        bootstrap_methods: Vec<BootstrapMethodFacts>,
        declaration: Option<Box<MethodDeclaration>>,
    ) -> Self {
        debug_assert!(
            frames.is_none() || canonical.is_some(),
            "the frames are derived from the canonical graph: a payload holding frames without a graph is not one run's artifact"
        );
        debug_assert!(
            ssa.is_none() || frames.is_some(),
            "the names are built over the frames: a payload holding them without the frames is not one run's artifact"
        );
        debug_assert!(
            canonical.is_none() || code.is_some(),
            "the canonical graph is built from the decoded body: a payload holding a graph without its decode facts is not one run's artifact"
        );
        debug_assert!(
            code.is_some() == facts.is_some(),
            "the pool travels with the read that decoded the body: a payload states a body and the facts it was read from, or neither"
        );
        Self {
            canonical,
            frames,
            ssa,
            code,
            facts,
            bootstrap_methods,
            declaration,
        }
    }

    /// The canonical CFG of this run, or `None` when the normalization published no graph.
    pub fn canonical(&self) -> Option<&CanonicalCfg> {
        self.canonical.as_deref()
    }

    /// The frames of this run, or `None` when the frame pass published no table.
    ///
    /// Present only when [`Self::canonical`] is: the frames are the entry states of that graph's
    /// blocks.
    pub fn frames(&self) -> Option<&FrameTable> {
        self.frames.as_deref()
    }

    /// The names, phis and effect facts of this run, or `None` when the `ssa` pass published none.
    ///
    /// Present only when [`Self::frames`] is: the names are the values of those frames.
    pub fn ssa(&self) -> Option<&SsaTable> {
        self.ssa.as_deref()
    }

    /// The decoded body of this run, or `None` when the `raw_facts` pass read no body.
    ///
    /// This is the class file's own decode — the instructions, their typed operands and the
    /// declared exception table, exactly as one read of the bytes produced them — and it is the
    /// **only** source of the body's symbolic vocabulary for a consumer above this crate: which
    /// local an instruction reads or writes, the value a constant pushes, the sense and the target
    /// of a conditional branch, the keys of a `switch`, the exception ranges the class declares.
    /// Nothing re-decodes and nothing re-states them.
    pub fn code(&self) -> Option<&MethodCodeFacts> {
        self.code.as_deref()
    }

    /// The class's constant pool as the same read decoded it; empty when no body was read.
    ///
    /// The pool is where a symbolic *reference* lives: an `invoke*`'s owner, member name and
    /// descriptor, an `ldc`'s constant. Together with [`Self::code`] it is one decode's answer to
    /// "what does this instruction name", which is why it travels with the payload instead of
    /// being resolved a second time by a consumer.
    ///
    /// It is read out of the run's own facts bundle, which the payload holds as a handle: the slice
    /// this returns is the *same allocation* for every payload decoded from one read — the pointer
    /// is the evidence a caller can check — and no method of a prepared class ever gets a copy of
    /// it.
    pub fn constant_pool(&self) -> &[CpEntryFacts] {
        match &self.facts {
            Some(facts) => &facts.constant_pool,
            None => &[],
        }
    }

    /// The class's `BootstrapMethods` table as the same header read decoded it; empty when the
    /// class declares no such attribute.
    ///
    /// This is what turns an `invokedynamic`'s `bootstrap_method_attr_index` into the method handle
    /// and the static arguments the site really names — the only fact from which "this dynamic site
    /// is a `LambdaMetafactory` call" can be read. Like [`Self::constant_pool`] it is *this* read's
    /// answer: one run, one table, no second opinion about which bootstrap a site uses.
    ///
    /// An entry holds the method-handle index and the argument indexes **as the class states them**
    /// — resolving them is the consumer's reading of [`Self::constant_pool`], exactly as it is for
    /// an `invoke*`'s target.
    pub fn bootstrap_methods(&self) -> &[BootstrapMethodFacts] {
        &self.bootstrap_methods
    }

    /// What the member's own declaration says, as the header read that located it stated it (P3 3.1).
    ///
    /// Present exactly when the `raw_facts` pass located the member: it is read from the same header
    /// read that produced [`Self::code`] and [`Self::constant_pool`], so a presentation that needs
    /// the member's flags, its descriptor or how many slots its parameters occupy reads the class
    /// file's own statement instead of a second read of the class (which would be billed again) or an
    /// assumption about the member. The same statement carries the declaring class's own internal
    /// name and access flags, which is what tells an interface's member from a class's.
    pub fn declaration(&self) -> Option<&MethodDeclaration> {
        self.declaration.as_deref()
    }
}

/// One method-analysis request's report and the IR payload the same run produced.
///
/// The two describe one run and are handed over together so that a consumer does not have to run
/// the pipeline twice to get both: the report is where the phase states, the quality, the coverage,
/// the execution and the diagnostics live, and the payload is what those phases produced.
#[derive(Debug)]
pub struct MethodIrAnalysis {
    report: MethodAnalysisReport,
    ir: MethodIr,
}

impl MethodIrAnalysis {
    pub(crate) fn new(report: MethodAnalysisReport, ir: MethodIr) -> Self {
        Self { report, ir }
    }

    /// The report of the run that produced [`Self::ir`].
    pub fn report(&self) -> &MethodAnalysisReport {
        &self.report
    }

    /// The payload of the run [`Self::report`] describes.
    pub fn ir(&self) -> &MethodIr {
        &self.ir
    }
}

#[cfg(test)]
mod tests {
    //! The one case the committed corpus does not hold: a body whose merge point needs a phi.
    //!
    //! Every class under `tests/fixtures` is straight-line or `jsr`-based, so the payload of a
    //! fixture run carries no phi at all, and the read surface of [`crate::method_ir`] would be
    //! checked without ever reaching a merge. The body below is assembled with the reader's own
    //! class builder (the same route `cfg`'s, `frame`'s and `ssa`'s tests take) and driven through
    //! the very passes the engine schedules, so the tables are the real ones; the test then reads
    //! them through the published surface only, exactly like the integration test.

    use super::*;

    use jarde_reader::budget::{Budget, Limits};
    use jarde_reader::classfile::{class_facts, method_code_facts, test_class};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant, SnapshotId,
    };
    use jarde_reader::view::LoaderId;

    use crate::call_context::{CallContextOutcome, call_contexts};
    use crate::canonical::{CanonicalOutcome, canonical_cfg};
    use crate::cfg::raw_cfg;
    use crate::frame::{FrameMethod, FrameOutcome, frames};
    use crate::ssa::{SsaOutcome, ssa};

    fn limits() -> Limits {
        Limits {
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        }
    }

    /// The three tables of one assembled body, over the passes the driver itself schedules — plus
    /// the decode facts the payload now carries beside them (the body and its class's pool).
    fn tables_of(code: &[u8], max_locals: u16) -> MethodIr {
        let bytes = test_class::single_method(52, 8, max_locals, code);
        let mut budget = Budget::new(limits());
        let header = class_facts(&bytes, &mut budget).expect("the assembled class is a class file");
        let member = header
            .methods
            .iter()
            .find(|member| member.name.raw().0 == b"method")
            .expect("the assembled class declares `method`");
        let facts =
            method_code_facts(&bytes, member, &mut budget).expect("the assembled body decodes");
        let pool = &header.constant_pool;
        let raw = raw_cfg(&facts, &mut budget).expect("the assembled body has a raw graph");
        let contexts = match call_contexts(&facts, &raw, header.major_version, &mut budget)
            .expect("the call-context walk runs")
        {
            CallContextOutcome::Established(contexts) => contexts,
            other => {
                panic!("an assembled body without `jsr`/`ret` establishes contexts, got {other:?}")
            }
        };
        let method_id = PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: SnapshotId("test".to_string()),
                },
                class_bytes: ClassBytesId {
                    digest: Digest("test".to_string()),
                    length: 0,
                },
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"method".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        };
        let graph = match canonical_cfg(&facts, &raw, &contexts, &method_id, &mut budget)
            .expect("the budget is ample")
        {
            CanonicalOutcome::Canonical(graph) => graph,
            CanonicalOutcome::Fallback { message } => {
                panic!("an assembled body normalizes, but stopped: {message}")
            }
        };
        let loader = LoaderId("app".to_string());
        let method = FrameMethod {
            access_flags: member.access_flags,
            name: b"method",
            descriptor: b"()V",
            owner: b"Test",
            super_class: Some(b"java/lang/Object"),
            pool,
            loader: &loader,
        };
        let table = match frames(&facts, &graph, &method, &mut budget).expect("the budget is ample")
        {
            FrameOutcome::Frames(table) => table,
            other => panic!("an assembled body has frames, got {other:?}"),
        };
        let names =
            match ssa(&facts, &graph, &table, &method, &mut budget).expect("the budget is ample") {
                SsaOutcome::Ssa(names) => names,
                SsaOutcome::Inconsistent { message } => {
                    panic!("the assembled body is consistent, but was refused: {message}")
                }
            };
        MethodIr::new(
            Some(graph),
            Some(table),
            Some(names),
            Some(Box::new(facts)),
            Some(Arc::new(header)),
            Vec::new(),
            // These parts are assembled instead of read, so the payload states no member declaration:
            // the declaration facts (P3 3.1) come from the header read that locates the member, and
            // no assembled body has one.
            None,
        )
    }

    #[test]
    fn a_merge_point_of_a_branching_body_hands_over_its_phi() {
        // 0 iconst_0, 1 istore_0, 2 iconst_0, 3 ifeq -> 11, 6 iconst_1, 7 istore_0,
        // 8 goto -> 13, 11 iconst_2, 12 istore_0, 13 return.
        //
        // Local 0 is written on both arms and the two writes disagree, so the merge block at BCI
        // 13 has exactly one entry phi with two operands — the one shape the committed fixtures
        // never produce.
        let ir = tables_of(
            &[
                0x03, // 0: iconst_0
                0x3b, // 1: istore_0
                0x03, // 2: iconst_0
                0x99, 0x00, 0x08, // 3: ifeq 11
                0x04, // 6: iconst_1
                0x3b, // 7: istore_0
                0xa7, 0x00, 0x05, // 8: goto 13
                0x05, // 11: iconst_2
                0x3b, // 12: istore_0
                0xb1, // 13: return
            ],
            1,
        );
        let canonical = ir.canonical().expect("the graph was published");
        let frames = ir.frames().expect("the frames were published");
        let ssa = ir.ssa().expect("the names were published");

        // P3 1.3b: the decode facts of the same read travel with the tables, so a consumer above
        // this crate has one source for the body's vocabulary instead of a table of its own.
        let code = ir.code().expect("the decoded body travels with the tables");
        assert_eq!(code.instructions.len(), 10, "one fact per instruction");
        assert_eq!(
            code.operands().len(),
            code.instructions.len(),
            "the operands are in lockstep with the instructions"
        );
        assert_eq!(
            code.instructions[3].opcode, 0x99,
            "the branch at BCI 3 is the `ifeq` the caller's own table would have restated"
        );
        assert!(
            !ir.constant_pool().is_empty(),
            "the class's own pool travels with the body it was decoded from"
        );
        assert!(
            ir.bootstrap_methods().is_empty(),
            "this class declares no `BootstrapMethods` attribute, and the payload states that with an empty table — the same read found none, so nothing was invented for it"
        );

        assert_eq!(canonical.blocks().len(), 4, "0, 6, 11 and 13");
        assert_eq!(canonical.edges().len(), 4, "0→6, 0→11, 6→13 and 11→13");
        assert!(canonical.unreachable().is_empty(), "every node is reached");
        assert_eq!(frames.blocks().len(), 4, "one entry state per block");
        assert_eq!(frames.locals_slots(), 1);

        assert_eq!(ssa.blocks().len(), 4, "every reached block is named");
        assert_eq!(ssa.phis().len(), 1, "one merge, one phi: {:#?}", ssa.phis());
        let phi = ssa.phis()[0].clone();
        assert_eq!(phi.block().bci(), 13, "the phi belongs to the merge block");
        assert_eq!(phi.slot(), Slot::Local(0));
        assert_eq!(phi.inputs().len(), 2, "one operand per arm");
        assert!(
            phi.inputs()
                .iter()
                .all(|input| matches!(input, PhiInput::Value(_))),
            "both arms hand their own value, so neither operand is the phi itself: {:#?}",
            phi.inputs()
        );
        assert!(
            matches!(ssa.value(phi.value()).def(), Definition::Phi { block, slot } if block.bci() == 13 && *slot == Slot::Local(0)),
            "the phi's own definition names the merge point and the slot"
        );
        assert_eq!(
            ssa.values().len(),
            8,
            "the values of the two arms and the merge"
        );
        assert!(
            frames
                .entry(phi.block())
                .expect("a named block is reached")
                .locals()
                .len()
                == 1,
            "the merge block is entered with its one local slot"
        );
    }
}
