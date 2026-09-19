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
//! Both are moved in, never copied, never re-decoded: the payload is what *that* run read. A
//! consumer therefore has exactly one source for the polarity of a branch, the slot of a load and
//! the value of a constant, and the compiler enforces it — there is no parameter left through
//! which a caller could hand in a second opinion.
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
//! * **Why not `Arc`, a cache or a second lifetime.** One request produces one payload for one
//!   consumer in a synchronous engine; there is nothing to share it with, no second reader that
//!   could outlive the first, and no cross-request identity a cache could key on without deciding
//!   a question (staleness against the bytes read) that no slice has asked. A cache would also
//!   have to keep the work of an earlier request alive past the budget that paid for it, which is
//!   exactly the accounting this pipeline is built to keep honest. If a future slice really needs
//!   the tables of a finished request to outlive their scope, that is a new decision with its own
//!   owner and budget story — not something this seam should assume on its behalf.
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
use jarde_reader::classfile::{BootstrapMethodFacts, CpEntryFacts, MethodCodeFacts};

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
    constant_pool: Vec<CpEntryFacts>,
    bootstrap_methods: Vec<BootstrapMethodFacts>,
}

impl MethodIr {
    /// One payload from the artifacts of one run, in the order the passes publish them.
    ///
    /// `code`, `constant_pool` and `bootstrap_methods` are the facts the `raw_facts` pass read:
    /// the decoded body, the class's own constant pool and its `BootstrapMethods` table. All three
    /// are moved in beside the tables, and all three are present exactly when the graph is — the
    /// graph is built from the decode they came out of, and a class that declares no bootstrap
    /// table states that with an empty one.
    pub(crate) fn new(
        canonical: Option<Box<CanonicalCfg>>,
        frames: Option<Box<FrameTable>>,
        ssa: Option<Box<SsaTable>>,
        code: Option<Box<MethodCodeFacts>>,
        constant_pool: Vec<CpEntryFacts>,
        bootstrap_methods: Vec<BootstrapMethodFacts>,
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
        Self {
            canonical,
            frames,
            ssa,
            code,
            constant_pool,
            bootstrap_methods,
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
    pub fn constant_pool(&self) -> &[CpEntryFacts] {
        &self.constant_pool
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
        let pool = header.constant_pool.clone();
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
            pool: &pool,
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
            pool,
            Vec::new(),
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
