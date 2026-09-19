//! P2 5.3's fixed replay list: the checked-in method-analysis goldens.
//!
//! Each file under `tests/fixtures/p2-golden/` records one group of replays: the input the
//! group names (its builder, its provenance, the blake3 digest and length of the bytes that
//! builder produces), the request, the limits the request ran under, and the complete report
//! the public entry point returned for it. These tests rebuild every input through the same
//! builder, replay the recorded request through `Engine::analyze_method`, and compare the
//! serialized report field by field.
//!
//! The only normalization is deleting `elapsed_millis`: it is the wall-clock number every
//! usage snapshot carries and not part of the analysis semantics. Stage states, quality,
//! representation, verification, `semantic_validation`, coverage ranges, BCI origins in the
//! coverage ranges, diagnostics (code, severity, message), execution status, the reads and
//! every usage dimension are compared exactly as published. The files never regenerate
//! themselves: a missing, changed or unreadable golden is a test failure, not an update.
//!
//! Every replay also carries the **state and stage invariants** the entry exists for, under
//! `expect`: the shape of `stages`, the product planes, the execution status with the reason
//! and code of a stop, the diagnostic code set, the `usage` dimensions the entry pins, and the
//! `published_prefix` of a stopped run. Those are asserted from the typed report as well as
//! through the whole-report comparison, so a reader of the golden can see what the entry is
//! about without reading the fixture builder, and a stop that lost its diagnostic or its
//! published prefix fails here even if every serialized field still round-trips.
//!
//! The list itself is an expectation: [`replays`] names every entry in file order, and the
//! tests compare it with what the JSON really holds, so deleting, renaming or adding a replay
//! fails even though the remaining replays would still replay correctly.
//!
//! # What the public side can and cannot see
//!
//! The canonical graph, the frames and the names are crate-private payloads (design invariant
//! 11): what a replay asserts here is the published report. In particular
//!
//! * a stop is asserted the way a caller sees it — the stage states, the execution reason and
//!   its diagnostic — and the *content* of the last valid phase is asserted by the phase being
//!   `Completed` (`Partial` would mean the phase stopped inside itself);
//! * the origin mapping of a clone is **not** visible here: `MethodAnalysisReport::origin` stays
//!   empty because P2 publishes no IR payload, so the one-to-many origin and the logical phi
//!   inputs are asserted in `crates/jarde-jvm/src/**` instead;
//! * `semantic_validation` is the run's own evidence, and its only public proxy is the
//!   implication "the `ssa` phase completed ⟹ `LocalInvariants`"; the invariant checks
//!   themselves are crate-private.

use jarde::*;
use serde_json::{Value, json};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Fixture builder
// ---------------------------------------------------------------------------

/// A `u2` in the class-file byte order.
fn u16b(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

/// A `u4` in the class-file byte order.
fn u32b(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

/// One constant-pool index inside a body being assembled.
fn cp(bytes: &mut Vec<u8>, index: u16) {
    u16b(bytes, index);
}

/// Constant-pool builder: entries are appended in order and keep their 1-based indexes, so a
/// fixture's body can name the exact entry it points at.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn push(&mut self, entry: Vec<u8>) -> u16 {
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("fixture pool fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("fixture name fits u16"),
        );
        entry.extend_from_slice(text);
        self.push(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.push(entry)
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        let mut entry = vec![12];
        u16b(&mut entry, name);
        u16b(&mut entry, descriptor);
        self.push(entry)
    }

    /// `Fieldref` (9), `Methodref` (10) or `InterfaceMethodref` (11).
    fn member(&mut self, tag: u8, owner: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![tag];
        u16b(&mut entry, owner);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    fn declared(&self) -> u16 {
        u16::try_from(self.entries.len() + 1).expect("fixture pool fits u16")
    }

    fn bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }
}

/// One exception-table record of a fixture body.
struct Handler {
    start: u16,
    end: u16,
    handler: u16,
    /// The `catch_type` pool index: a `Class` entry, or `0` for a catch-all record.
    catch_type: u16,
}

/// One method of a fixture class: its flags, its name and descriptor, and its `Code` body.
struct Method {
    access: u16,
    name: u16,
    descriptor: u16,
    code_name: u16,
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    handlers: Vec<Handler>,
}

/// Assembles a class file around a finished pool.
fn assemble(
    pool: &Pool,
    major: u16,
    minor: u16,
    access: u16,
    this_class: u16,
    super_class: u16,
    methods: &[Method],
) -> Vec<u8> {
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, minor);
    u16b(&mut bytes, major);
    u16b(&mut bytes, pool.declared());
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, access);
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, super_class);
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    u16b(
        &mut bytes,
        u16::try_from(methods.len()).expect("fixture methods fit u16"),
    );
    for method in methods {
        u16b(&mut bytes, method.access);
        u16b(&mut bytes, method.name);
        u16b(&mut bytes, method.descriptor);
        u16b(&mut bytes, 1); // the one attribute: `Code`
        let mut content = Vec::new();
        u16b(&mut content, method.max_stack);
        u16b(&mut content, method.max_locals);
        u32b(
            &mut content,
            u32::try_from(method.code.len()).expect("fixture code fits u32"),
        );
        content.extend_from_slice(&method.code);
        u16b(
            &mut content,
            u16::try_from(method.handlers.len()).expect("fixture handlers fit u16"),
        );
        for handler in &method.handlers {
            u16b(&mut content, handler.start);
            u16b(&mut content, handler.end);
            u16b(&mut content, handler.handler);
            u16b(&mut content, handler.catch_type);
        }
        u16b(&mut content, 0); // the `Code` attribute's own attributes: no debug table at all
        u16b(&mut bytes, method.code_name);
        u32b(
            &mut bytes,
            u32::try_from(content.len()).expect("fixture Code content fits u32"),
        );
        bytes.extend_from_slice(&content);
    }
    u16b(&mut bytes, 0); // class attributes
    bytes
}

/// A tiny assembler for the fixture bodies: labels, the branch forms the bodies use, and the
/// two switch forms with the alignment the JVMS requires.
#[derive(Default)]
struct Asm {
    code: Vec<u8>,
    labels: Vec<(String, u16)>,
    /// `(position of the two-byte operand, BCI of the branch opcode, label)`.
    short: Vec<(usize, u16, String)>,
    /// `(position of the four-byte operand, BCI of the switch opcode, label)`.
    wide: Vec<(usize, u16, String)>,
}

impl Asm {
    fn op(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    fn mark(&mut self, label: &str) {
        let at = u16::try_from(self.code.len()).expect("fixture body fits u16");
        self.labels.push((label.to_string(), at));
    }

    /// The byte offset one label names.
    fn label(&self, label: &str) -> u16 {
        self.labels
            .iter()
            .find(|(name, _)| name == label)
            .unwrap_or_else(|| panic!("the fixture names no label {label:?}"))
            .1
    }

    /// One two-byte branch: `goto`, `jsr`, the conditional branches.
    fn branch(&mut self, opcode: u8, label: &str) {
        let at = u16::try_from(self.code.len()).expect("fixture body fits u16");
        self.op(&[opcode, 0, 0]);
        self.short
            .push((self.code.len() - 2, at, label.to_string()));
    }

    fn goto(&mut self, label: &str) {
        self.branch(0xa7, label);
    }

    fn ifeq(&mut self, label: &str) {
        self.branch(0x99, label);
    }

    fn jsr(&mut self, label: &str) {
        self.branch(0xa8, label);
    }

    /// `wide <opcode> <u2 index>`, the form a load or store of a local above 255 takes.
    fn wide_local(&mut self, opcode: u8, index: u16) {
        self.op(&[0xc4, opcode]);
        u16b(&mut self.code, index);
    }

    /// `wide iinc <u2 index> <i2 const>`.
    fn wide_iinc(&mut self, index: u16, increment: i16) {
        self.op(&[0xc4, 0x84]);
        u16b(&mut self.code, index);
        self.op(&increment.to_be_bytes());
    }

    /// The padding a switch needs so that its first operand follows an address that is a
    /// multiple of four **from the start of the code array**.
    fn switch_padding(&mut self) {
        while !self.code.len().is_multiple_of(4) {
            self.code.push(0);
        }
    }

    /// `tableswitch`: one target per key of `low..=low + targets.len() - 1`.
    fn tableswitch(&mut self, low: i32, targets: &[&str], default: &str) {
        let opcode_at = u16::try_from(self.code.len()).expect("fixture body fits u16");
        self.op(&[0xaa]);
        self.switch_padding();
        self.op(&0_i32.to_be_bytes());
        self.wide
            .push((self.code.len() - 4, opcode_at, default.to_string()));
        self.op(&low.to_be_bytes());
        let high = low + i32::try_from(targets.len()).expect("fixture keys fit i32") - 1;
        self.op(&high.to_be_bytes());
        for target in targets {
            self.op(&0_i32.to_be_bytes());
            self.wide
                .push((self.code.len() - 4, opcode_at, target.to_string()));
        }
    }

    /// `lookupswitch`: the caller's `(key, target)` pairs, which must be sorted by key.
    fn lookupswitch(&mut self, pairs: &[(i32, &str)], default: &str) {
        let opcode_at = u16::try_from(self.code.len()).expect("fixture body fits u16");
        self.op(&[0xab]);
        self.switch_padding();
        self.op(&0_i32.to_be_bytes());
        self.wide
            .push((self.code.len() - 4, opcode_at, default.to_string()));
        self.op(&i32::try_from(pairs.len())
            .expect("fixture pairs fit i32")
            .to_be_bytes());
        let mut previous: Option<i32> = None;
        for (key, target) in pairs {
            assert!(
                previous.is_none_or(|held| held < *key),
                "lookupswitch keys must be strictly ascending"
            );
            previous = Some(*key);
            self.op(&key.to_be_bytes());
            self.op(&0_i32.to_be_bytes());
            self.wide
                .push((self.code.len() - 4, opcode_at, target.to_string()));
        }
    }

    /// Writes every branch and switch operand, and answers the body. The labels stay readable
    /// afterwards, so a fixture can name a block it just assembled (a handler entry, a range
    /// end) when it builds its exception table.
    fn code(&mut self) -> Vec<u8> {
        let labels = self.labels.clone();
        let target = |label: &str| -> i32 {
            let at = labels
                .iter()
                .find(|(name, _)| name == label)
                .unwrap_or_else(|| panic!("the fixture names no label {label:?}"))
                .1;
            i32::from(at)
        };
        for (position, from, label) in &self.short {
            let offset = target(label) - i32::from(*from);
            let offset =
                i16::try_from(offset).unwrap_or_else(|_| panic!("fixture branch to {label} fits"));
            self.code[*position..*position + 2].copy_from_slice(&offset.to_be_bytes());
        }
        for (position, from, label) in &self.wide {
            let offset = target(label) - i32::from(*from);
            self.code[*position..*position + 4].copy_from_slice(&offset.to_be_bytes());
        }
        self.code.clone()
    }
}

// ---------------------------------------------------------------------------
// The replay list, the requests and the harness
// ---------------------------------------------------------------------------

/// The golden files, in the order their replay lists are declared below.
const GOLDEN_FILES: [&str; 6] = [
    "historical.json",
    "inputs.json",
    "legacy-clone.json",
    "exception-overlap.json",
    "wide-switch.json",
    "resource-boundary.json",
];

/// The limits every entry gets unless it is a boundary entry: wide enough that only the entry's
/// own dimension can stop the run, and every dimension the pipeline charges is non-zero.
fn ample() -> Limits {
    Limits {
        input_bytes: 1 << 24,
        archive_entries: 10_000,
        entry_bytes: 1 << 24,
        read_bytes: 1 << 24,
        class_bytes: 1 << 24,
        attribute_bytes: 1 << 24,
        code_bytes: 1 << 24,
        result_items: 1 << 20,
        output_bytes: 1 << 24,
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 24,
        ir_edges: 1 << 24,
        analysis_steps: 1 << 22,
        normalization_clones: 16,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

/// The measured prices the boundary entries' limits are derived from, and the points R10's two
/// slots are pinned at. They are fixed numbers, not live measurements: a golden has to record the
/// budget the request really ran under, and a limit recomputed at test time would move with the
/// implementation instead of catching it.
///
/// Every one of them is also recorded, field by field, in the `usage` plane of the golden that
/// measured it, so the comparison fails if any of them changes.
const R10_CANONICAL_ITEMS: u64 = 61;
const R10_FRAME_ITEMS_8_SITES: u64 = 280_088;
const R10_FRAME_ITEMS_64_SITES: u64 = 1_960_592;
const R10_FRAME_ITEMS_8_SITES_NO_HANDLER: u64 = 30_055;
const R10_NAMES_ITEMS_8_SITES: u64 = 300_446;
const DEEP_CHAIN_FRAME_STEPS: u64 = 1_396;
const HIGH_FANOUT_CANONICAL_EDGES: u64 = 195;

/// One entry of the fixed replay list.
struct Replay {
    file: &'static str,
    name: &'static str,
    /// The coverage category the entry carries, worded as the 5.3 list words it.
    category: &'static str,
    /// What the entry exists for, in one sentence: the state or invariant it pins.
    note: &'static str,
    fixture: &'static str,
    method: (&'static str, &'static str),
    stages: &'static [AnalysisStage],
    limits: Limits,
    /// A request cancelled before it ran, which every phase must answer as a cancellation.
    cancel: bool,
}

const SS: &[AnalysisStage] = &[AnalysisStage::Ssa];
const CFG: &[AnalysisStage] = &[AnalysisStage::CanonicalCfg];

/// The eight historical entries, by class-file version: the replay list names each of them, so a
/// version that loses its entry fails in the list comparison.
const HISTORICAL_NAMES: [&str; 8] = [
    "historical-v45-finally",
    "historical-v46-finally",
    "historical-v47-finally",
    "historical-v48-finally",
    "historical-v49-finally",
    "historical-v50-finally",
    "historical-v51-finally",
    "historical-v52-finally",
];

/// The fixed replay list, in file order.
///
/// This list is the expectation: [`expected_replays`] compares it with what the golden files
/// hold, so a replay that was renamed, deleted or added fails even though the remaining replays
/// would still replay correctly. Every category of the 5.3 list is carried by at least one entry,
/// and [`CATEGORIES`] is what says so.
fn replay_list() -> Vec<Replay> {
    let mut replays = Vec::new();

    // ---------------------------------------------------------------- historical
    for version in 45..=52u16 {
        let fixture = match version {
            45 => "historical-45",
            46 => "historical-46",
            47 => "historical-47",
            48 => "historical-48",
            49 => "historical-49",
            50 => "historical-50",
            51 => "historical-51",
            _ => "historical-52",
        };
        replays.push(Replay {
            file: "historical.json",
            name: HISTORICAL_NAMES[usize::from(version - 45)],
            category: "45-52 historical class (ECJ `finally`, `jsr`/`ret` in 45-48)",
            note: "the committed ECJ corpus at this class-file version, analyzed to the names: \
                   the 45-48 bodies normalize their shared `jsr` subroutine into one clone per \
                   call site, the 49-52 bodies inline it, and both complete",
            fixture,
            method: ("finallyPath", "(I)I"),
            stages: SS,
            limits: ample(),
            cancel: false,
        });
    }

    // -------------------------------------------------------------------- inputs
    replays.push(Replay {
        file: "inputs.json",
        name: "missing-debug-derived-frames",
        category: "missing debug",
        note: "a loop whose class file carries no StackMapTable, LineNumberTable or \
               local-variable table: the frames are derived from the descriptor and the body, and \
               every phase completes without a diagnostic",
        fixture: "missing-debug-loop",
        method: ("method", "()I"),
        stages: SS,
        limits: ample(),
        cancel: false,
    });
    replays.push(Replay {
        file: "inputs.json",
        name: "missing-dependency-completes",
        category: "missing dependency",
        note: "a body that names a class no snapshot holds: the bytecode-level analysis does not \
               resolve the owner, so the pipeline completes and the only read is the driver class",
        fixture: "missing-dependency",
        method: ("method", "()V"),
        stages: SS,
        limits: ample(),
        cancel: false,
    });
    replays.push(Replay {
        file: "inputs.json",
        name: "illegal-version-jsr-in-52",
        category: "illegal version",
        note: "the same two-call-site `jsr` body as the legacy entry, at major 52 where the \
               opcode is a dialect violation: the call contexts are refused, the raw facts and \
               the raw graph stay published, and nothing after them runs",
        fixture: "jsr-body-v52",
        method: ("method", "()V"),
        stages: SS,
        limits: ample(),
        cancel: false,
    });

    // -------------------------------------------------------------- legacy clone
    replays.push(Replay {
        file: "legacy-clone.json",
        name: "shared-subroutine-two-call-sites",
        category: "shared subroutine (legacy clone)",
        note: "two `jsr` call sites sharing one subroutine at major 50: the subroutine is cloned \
               once per call site, both charges are visible in the clone dimension, and the whole \
               pipeline over the clones completes",
        fixture: "jsr-body-v50",
        method: ("method", "()V"),
        stages: SS,
        limits: ample(),
        cancel: false,
    });

    // --------------------------------------------------------- exception overlap
    replays.push(Replay {
        file: "exception-overlap.json",
        name: "overlapping-records-and-nested-handler",
        category: "exception overlap (nested handler)",
        note: "two overlapping catch-all records over the same two throw sites, plus a third \
               record over the throw of the first record's handler: every phase completes",
        fixture: "exception-overlap-nested",
        method: ("method", "()V"),
        stages: SS,
        limits: ample(),
        cancel: false,
    });
    replays.push(Replay {
        file: "exception-overlap.json",
        name: "range-starting-inside-a-block",
        category: "exception overlap (mid-block range)",
        note: "a record whose range starts inside a block: the handler row is stated for the \
               throw site the range covers, and the body is not refused as self-contradictory",
        fixture: "exception-mid-block-range",
        method: ("method", "()V"),
        stages: SS,
        limits: ample(),
        cancel: false,
    });
    replays.push(Replay {
        file: "exception-overlap.json",
        name: "exception-back-edge",
        category: "exception overlap (exception back edge)",
        note: "the exception edge runs back into a block that was already processed, so the \
               handler's input has to settle with the loop: R9's back-edge contrast",
        fixture: "exception-back-edge",
        method: ("method", EXCEPTION_METHOD_STR),
        stages: SS,
        limits: ample(),
        cancel: false,
    });
    replays.push(Replay {
        file: "exception-overlap.json",
        name: "two-throw-sites-one-block",
        category: "exception overlap (same block, several throw sites)",
        note: "two throw sites of one block under one record: one raw edge, two logical inputs, \
               and the handler's entry state is their merge",
        fixture: "exception-two-sites",
        method: ("method", EXCEPTION_METHOD_STR),
        stages: SS,
        limits: ample(),
        cancel: false,
    });
    replays.push(Replay {
        file: "exception-overlap.json",
        name: "r9-exception-input-follows-its-throw-site",
        category: "R9 regression",
        note: "the 17 bytes R9 was reported in: the block's exit is stable across the loop while \
               the throw site's locals are not, and the handler's input follows the throw site",
        fixture: "r9-exception-input",
        method: ("method", EXCEPTION_METHOD_STR),
        stages: SS,
        limits: ample(),
        cancel: false,
    });

    // ---------------------------------------------------------------- wide/switch
    replays.push(Replay {
        file: "wide-switch.json",
        name: "wide-forms-and-both-switches",
        category: "wide and switch",
        note: "`wide istore/iload/iinc` on local 300, a `tableswitch` and a `lookupswitch` in one \
               body: the wide forms take part in the block structure and the frames, and the \
               switch targets become blocks",
        fixture: "wide-and-switch",
        method: ("method", "()V"),
        stages: SS,
        limits: ample(),
        cancel: false,
    });

    // ----------------------------------------------------------- resource boundary
    let mut over_budget = ample();
    over_budget.ir_items = R10_CANONICAL_ITEMS + 10_001;
    replays.push(Replay {
        file: "resource-boundary.json",
        name: "over-budget-r10-items",
        category: "resource boundary (over budget)",
        note: "the R10 probe: a budget of the canonical price plus 10001 items, which the frames \
               of this shape exceed because every covered throw site retains its own state, so the \
               run stops there and keeps the published prefix",
        fixture: "throw-sites-8",
        method: ("method", "()V"),
        stages: SS,
        limits: over_budget,
        cancel: false,
    });
    replays.push(Replay {
        file: "resource-boundary.json",
        name: "r10-canonical-price",
        category: "R10 regression (slot product)",
        note: "the R10 shape to the canonical graph: the price the over-budget entry's limit is \
               measured from, and the evidence that building the graph does not depend on the \
               local slots at all",
        fixture: "throw-sites-8",
        method: ("method", "()V"),
        stages: CFG,
        limits: ample(),
        cancel: false,
    });
    replays.push(Replay {
        file: "resource-boundary.json",
        name: "r10-frame-slot-product-8-sites",
        category: "R10 regression (slot product)",
        note: "the same body to the frames: the first of R10's two measured points, where every \
               one of the eight covered throw sites retains the state its handler reads",
        fixture: "throw-sites-8",
        method: ("method", "()V"),
        stages: &[AnalysisStage::Frame],
        limits: ample(),
        cancel: false,
    });
    replays.push(Replay {
        file: "resource-boundary.json",
        name: "r10-frame-slot-product-64-sites",
        category: "R10 regression (slot product)",
        note: "the same shape with 64 throw sites: the second measured point, exactly 56 retained \
               per-site states above the first one",
        fixture: "throw-sites-64",
        method: ("method", "()V"),
        stages: &[AnalysisStage::Frame],
        limits: ample(),
        cancel: false,
    });
    replays.push(Replay {
        file: "resource-boundary.json",
        name: "r10-frames-without-a-handler-retain-far-less",
        category: "R10 regression (slot product)",
        note: "the same eight sites with no exception record at all: without a handler to read \
               them, the per-site states are not retained, which is what makes the two points \
               above a statement about the covered handler rather than about the body",
        fixture: "throw-sites-8-no-handler",
        method: ("method", "()V"),
        stages: &[AnalysisStage::Frame],
        limits: ample(),
        cancel: false,
    });
    replays.push(Replay {
        file: "resource-boundary.json",
        name: "r10-slot-product-8-sites-with-names",
        category: "R10 regression (slot product)",
        note: "the same body to the names: the names are charged on top of the frames and the run \
               still completes",
        fixture: "throw-sites-8",
        method: ("method", "()V"),
        stages: SS,
        limits: ample(),
        cancel: false,
    });
    let mut deep_chain = ample();
    deep_chain.analysis_steps = DEEP_CHAIN_FRAME_STEPS + 1;
    replays.push(Replay {
        file: "resource-boundary.json",
        name: "deep-chain-steps",
        category: "resource boundary (deep chain)",
        note: "64 blocks in a chain, with one step more than the frames of the body cost: the \
               names phase starts, runs out of steps and keeps the frames as the last valid phase",
        fixture: "deep-chain",
        method: ("method", "()V"),
        stages: SS,
        limits: deep_chain,
        cancel: false,
    });
    let mut high_fanout = ample();
    high_fanout.ir_edges = HIGH_FANOUT_CANONICAL_EDGES;
    replays.push(Replay {
        file: "resource-boundary.json",
        name: "high-fanout-edges",
        category: "resource boundary (high fanout)",
        note: "one block with 64 successors, budgeted at exactly the canonical edge price: the \
               names phase cannot charge its first def-use edge and stops with the canonical \
               graph and the frames as the published prefix",
        fixture: "high-fanout",
        method: ("method", "()V"),
        stages: SS,
        limits: high_fanout,
        cancel: false,
    });
    replays.push(Replay {
        file: "resource-boundary.json",
        name: "cancelled-before-the-first-read",
        category: "resource boundary (cancellation)",
        note: "a request whose token is already cancelled: the run is a cancellation with its own \
               diagnostic, no phase publishes anything, and the planes stay unproven",
        fixture: "missing-debug-loop",
        method: ("method", "()I"),
        stages: SS,
        limits: ample(),
        cancel: true,
    });

    replays
}

/// The descriptor of the exception fixtures, as the replay list spells it.
const EXCEPTION_METHOD_STR: &str = "(Ljava/lang/Object;)Ljava/lang/Object;";

/// Every category the 5.3 list names, and the entry that carries it. A category with no entry is
/// a gap in the fixed list, not a missing note.
fn categories() -> &'static [(&'static str, &'static str)] {
    &[
        ("45-52 historical class", "historical-v45-finally"),
        ("missing dependency", "missing-dependency-completes"),
        ("missing debug", "missing-debug-derived-frames"),
        ("illegal version", "illegal-version-jsr-in-52"),
        (
            "shared subroutine (legacy clone)",
            "shared-subroutine-two-call-sites",
        ),
        (
            "exception overlap (nested handler)",
            "overlapping-records-and-nested-handler",
        ),
        ("wide and switch", "wide-forms-and-both-switches"),
        ("resource boundary (over budget)", "over-budget-r10-items"),
        ("resource boundary (deep chain)", "deep-chain-steps"),
        ("resource boundary (high fanout)", "high-fanout-edges"),
        ("R9 regression", "r9-exception-input-follows-its-throw-site"),
        ("R10 regression", "r10-frame-slot-product-8-sites"),
    ]
}

// ---------------------------------------------------------------------------
// Opening a fixture and replaying a request
// ---------------------------------------------------------------------------

/// One opened fixture: the snapshot its bytes opened as, the environment that names it and the
/// physical identity of the method the replay analyzes.
struct Opened {
    snapshot: ArtifactSnapshot,
    environment: ResolutionEnvironment,
    method: PhysicalMethodId,
}

fn environment_of(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
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
    }
}

/// Builds one fixture's bytes and opens them as a standalone CLASS snapshot.
fn open(fixture: &str, method: (&str, &str)) -> Opened {
    let bytes = fixture_bytes(fixture);
    let mut budget = Budget::new(ample());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.clone()), &mut budget)
        .expect("a golden fixture opens as a standalone CLASS");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(&bytes).to_hex().to_string()),
            length: u64::try_from(bytes.len()).expect("a fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(method.0.as_bytes().to_vec()),
        descriptor: JvmBytes(method.1.as_bytes().to_vec()),
    };
    let environment = environment_of(&snapshot);
    Opened {
        snapshot,
        environment,
        method,
    }
}

/// The request one replay records, as the replay list describes it.
fn request_of(opened: &Opened, replay: &Replay) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: opened.environment.clone(),
        method: opened.method.clone(),
        stages: replay.stages.to_vec(),
    }
}

/// Runs one replay's request through the public entry point.
fn run_replay(opened: &Opened, replay: &Replay) -> (MethodAnalysisRequest, MethodAnalysisReport) {
    let request = request_of(opened, replay);
    let mut budget = if replay.cancel {
        let token = CancellationToken::new();
        token.cancel();
        Budget::with_cancellation_token(replay.limits.clone(), token)
    } else {
        Budget::new(replay.limits.clone())
    };
    let report = Engine::new()
        .analyze_method(
            std::slice::from_ref(&opened.snapshot),
            &request,
            &mut budget,
        )
        .expect("a recorded request is answered, not raised");
    (request, report)
}

// ---------------------------------------------------------------------------
// The tests
// ---------------------------------------------------------------------------

/// The usage snapshot a report's execution carries.
fn usage_of(report: &MethodAnalysisReport) -> UsageSnapshot {
    match &report.execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage.clone(),
    }
}

/// The diagnostic code a stopped run must carry, derived from its own termination.
fn terminal_code(report: &MethodAnalysisReport) -> Option<String> {
    let reason = match &report.execution {
        ExecutionReport::Complete { .. } => return None,
        ExecutionReport::Cancelled { .. } => return Some("cancelled".to_string()),
        ExecutionReport::Partial { reason, .. } | ExecutionReport::Failed { reason, .. } => reason,
    };
    Some(match reason {
        TerminationReason::BudgetExceeded { dimension } => {
            format!("budget_exceeded_{}", budget_dimension_code(*dimension))
        }
        TerminationReason::Error { code } => code.clone(),
        TerminationReason::Unsupported { code } => code.clone(),
    })
}

/// The stage invariants every replay must keep, whatever the report says.
///
/// A stopped run has to explain itself: the stage it stopped in is the only non-`Completed` one
/// before the schedule ends, everything before it is still `Completed` (that is the published
/// prefix, and the planes around it are published with it), everything after it stays
/// `NotPerformed`, and the run carries the diagnostic its own termination names. A run that
/// stopped before its first phase has an empty prefix, which is exactly what it published.
fn assert_stop_is_explainable(replay: &Replay, report: &MethodAnalysisReport) {
    let prefix: Vec<&AnalysisStage> = report
        .stages
        .iter()
        .take_while(|stage| stage.state == StageState::Completed)
        .map(|stage| &stage.stage)
        .collect();
    let stopped: Vec<&StageResult> = report
        .stages
        .iter()
        .filter(|stage| {
            !matches!(
                stage.state,
                StageState::Completed | StageState::NotPerformed
            )
        })
        .collect();
    let code = terminal_code(report);
    match &report.execution {
        ExecutionReport::Complete { .. } => {
            assert!(
                stopped.is_empty(),
                "{}: a complete run has no stopped stage: {stopped:?}",
                replay.name
            );
            assert_eq!(
                prefix.len(),
                report.stages.len(),
                "{}: every scheduled stage completed",
                replay.name
            );
            assert!(
                report.diagnostics.is_empty(),
                "{}: a complete run reports no diagnostic: {:?}",
                replay.name,
                report.diagnostics
            );
        }
        _ => {
            assert_eq!(
                stopped.len(),
                1,
                "{}: exactly the stage the run stopped in is not completed: {stopped:?}",
                replay.name
            );
            let stopped_at = report
                .stages
                .iter()
                .position(|stage| stage.stage == stopped[0].stage)
                .expect("the stopped stage is scheduled");
            assert_eq!(
                stopped_at,
                prefix.len(),
                "{}: the published prefix is the run of completed stages before the stop",
                replay.name
            );
            let after: Vec<&StageResult> = report.stages.iter().skip(stopped_at + 1).collect();
            assert!(
                after
                    .iter()
                    .all(|stage| stage.state == StageState::NotPerformed),
                "{}: nothing after the stop ran: {after:?}",
                replay.name
            );
            let code = code.expect("a stopped run names a termination code");
            assert!(
                report
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == code),
                "{}: the stop is explained by its own diagnostic {code:?}: {:?}",
                replay.name,
                report
                    .diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.code.as_str())
                    .collect::<Vec<_>>()
            );
            if prefix.is_empty() {
                // Nothing was published, so nothing may claim to have been: the coverage plane
                // stays `NotRequested` and no read was recorded.
                assert_eq!(
                    report.coverage,
                    Coverage::not_requested(),
                    "{}: a run that published nothing states no coverage",
                    replay.name
                );
            } else {
                // The published prefix is still there: the phases before the stop are
                // `Completed`, and the planes they publish (coverage, reads) are published with
                // them.
                assert_ne!(
                    report.coverage.artifact_structural.state,
                    CoverageState::NotRequested,
                    "{}: the body the published prefix read keeps its coverage",
                    replay.name
                );
                assert!(
                    !report.reads.is_empty(),
                    "{}: the published prefix read the driver class",
                    replay.name
                );
            }
        }
    }
}

/// Fails unless one golden file records exactly the replays the list names, in order.
fn assert_replay_list(file: &str) {
    let golden = golden(file);
    let recorded: Vec<&str> = golden_replays(&golden)
        .iter()
        .map(|replay| replay["name"].as_str().expect("a replay name"))
        .collect();
    let list = replay_list();
    let expected: Vec<&str> = list
        .iter()
        .filter(|replay| replay.file == file)
        .map(|replay| replay.name)
        .collect();
    assert!(
        !expected.is_empty(),
        "golden {file} carries no entry in the replay list"
    );
    assert_eq!(
        recorded, expected,
        "golden {file} must record exactly its listed replays, in order"
    );
}

/// Replays one recorded entry and asserts the fixture, the request, the whole report and the
/// state invariants against a live run.
fn assert_replay(replay: &Replay, recorded: &Value) {
    let bytes = fixture_bytes(replay.fixture);
    assert_eq!(
        recorded["fixture"]["blake3"],
        blake3::hash(&bytes).to_hex().to_string(),
        "the {} builder produces different bytes than its golden records",
        replay.fixture
    );
    assert_eq!(
        recorded["fixture"]["bytes"],
        bytes.len(),
        "the {} builder produces another length than its golden records",
        replay.fixture
    );
    assert_eq!(recorded["fixture"]["name"], replay.fixture);
    assert_eq!(
        recorded["fixture"]["source"],
        fixture_source(replay.fixture),
        "the recorded provenance of {} moved",
        replay.fixture
    );
    assert_eq!(recorded["category"], replay.category);
    assert_eq!(recorded["note"], replay.note);
    assert_eq!(recorded["kind"], "method_analysis");
    assert_eq!(recorded["cancel_before_run"], replay.cancel);
    assert_eq!(
        recorded["limits"],
        serde_json::to_value(&replay.limits).expect("the limits serialize"),
        "{}: the golden records the limits the entry pins",
        replay.name
    );

    // The recorded report must not carry a run-dependent value: a golden that did would be a
    // moving target the comparison below could never fail on.
    let mut normalized_recorded = recorded["report"].clone();
    normalize(&mut normalized_recorded);
    assert_eq!(
        normalized_recorded, recorded["report"],
        "{}: the golden records the report without run-dependent fields",
        replay.name
    );

    let opened = open(replay.fixture, replay.method);
    let (request, report) = run_replay(&opened, replay);

    // The recorded request is the one this entry sends, field by field.
    assert_eq!(
        serde_json::to_value(&request).expect("the request serializes"),
        recorded["request"],
        "{}: the recorded request is the request the entry sends",
        replay.name
    );
    // And it is replayable as recorded: deserializing the golden's own copy answers the same.
    let replayed: MethodAnalysisRequest =
        serde_json::from_value(recorded["request"].clone()).expect("the golden records a request");
    assert_eq!(
        replayed, request,
        "{}: the recorded request deserializes to the same request",
        replay.name
    );

    let mut actual = serde_json::to_value(&report).expect("the report serializes");
    normalize(&mut actual);
    assert_eq!(
        actual, recorded["report"],
        "golden {} replay {:?} no longer matches the public entry point",
        replay.file, replay.name
    );

    let usage = usage_of(&report);
    assert_eq!(
        published_planes(&report, &usage),
        recorded["expect"],
        "{}: the recorded state and stage invariants are the run's own",
        replay.name
    );

    assert_stop_is_explainable(replay, &report);
}

/// Every recorded entry of one golden file, in file order.
fn assert_golden_file(file: &str) {
    assert_replay_list(file);
    let golden = golden(file);
    let list = replay_list();
    for recorded in golden_replays(&golden) {
        let name = recorded["name"].as_str().expect("a replay name");
        let replay = list
            .iter()
            .find(|replay| replay.name == name)
            .unwrap_or_else(|| panic!("the golden records the unlisted replay {name:?}"));
        assert_replay(replay, recorded);
    }
}

/// Reads one recorded entry's invariants without replaying it, for the cross-entry checks the
/// file-level tests below make.
fn recorded_expect(file: &str, name: &str) -> Value {
    let golden = golden(file);
    golden_replay(&golden, name)["expect"].clone()
}

#[test]
fn the_historical_corpus_replays_from_45_to_52() {
    assert_golden_file("historical.json");

    // The state assertion behind the eight entries: every version completes, and the two call
    // sites of the old dialect's shared subroutine are two clones while the modern dialect has
    // none. That is a statement about the reports, not about the recording.
    for version in 45..=52u16 {
        let name = format!("historical-v{version}-finally");
        let expect = recorded_expect("historical.json", &name);
        assert_eq!(
            expect["quality"], "conservative",
            "{name}: a canonical artifact was produced"
        );
        assert_eq!(
            expect["semantic_validation"], "local_invariants",
            "{name}: the `ssa` phase completed, so its local invariants are this run's evidence"
        );
        assert_eq!(expect["execution"]["status"], "complete", "{name}");
        let clones = expect["usage"]["normalization_clones"]
            .as_u64()
            .expect("a clone count");
        if version <= 48 {
            assert_eq!(
                clones, 2,
                "{name}: the shared subroutine is cloned once per call site"
            );
        } else {
            assert_eq!(clones, 0, "{name}: the modern dialect inlines the finally");
        }
        assert_eq!(
            expect["stages"].as_array().expect("stage results").len(),
            6,
            "{name}: the whole pipeline was scheduled"
        );
    }
}

#[test]
fn the_stopped_inputs_stay_published_and_explained() {
    assert_golden_file("inputs.json");

    let refused = recorded_expect("inputs.json", "illegal-version-jsr-in-52");
    assert_eq!(
        refused["published_prefix"],
        json!(["raw_facts", "raw_cfg"]),
        "the dialect violation is reported after the facts and the raw graph"
    );
    assert_eq!(refused["execution"]["status"], "failed");
    assert_eq!(
        refused["diagnostic_codes"],
        json!(["ir_legacy_opcode_forbidden"]),
        "the refusal names the opcode the dialect forbids"
    );
    assert!(
        !refused["coverage"]["artifact_structural"]["scanned"]
            .as_array()
            .expect("the scanned ranges")
            .is_empty(),
        "the raw graph the run published keeps its coverage ranges"
    );

    // The dependency is missing and the debug tables are missing, and neither is a failure:
    // both entries complete with an empty diagnostic list.
    for name in [
        "missing-debug-derived-frames",
        "missing-dependency-completes",
    ] {
        let expect = recorded_expect("inputs.json", name);
        assert_eq!(expect["execution"]["status"], "complete", "{name}");
        assert_eq!(expect["diagnostic_codes"], json!([]), "{name}");
        assert_eq!(expect["body"], json!({"kind": "present"}), "{name}");
    }
    assert_eq!(
        recorded_expect("inputs.json", "missing-dependency-completes")["reads"],
        json!({"count": 1, "reasons": ["driver_method_body"]}),
        "the only class read is the driver's own"
    );
}

#[test]
fn the_legacy_clone_entry_bills_two_call_sites() {
    assert_golden_file("legacy-clone.json");
    let expect = recorded_expect("legacy-clone.json", "shared-subroutine-two-call-sites");
    assert_eq!(expect["execution"]["status"], "complete");
    assert_eq!(
        expect["usage"]["normalization_clones"], 2,
        "one clone node per call site of the shared subroutine"
    );
    assert_eq!(expect["quality"], "conservative");
    assert_eq!(expect["semantic_validation"], "local_invariants");
}

#[test]
fn the_exception_entries_complete_with_the_handler_inputs_settled() {
    assert_golden_file("exception-overlap.json");
    for name in [
        "overlapping-records-and-nested-handler",
        "range-starting-inside-a-block",
        "exception-back-edge",
        "two-throw-sites-one-block",
        "r9-exception-input-follows-its-throw-site",
    ] {
        let expect = recorded_expect("exception-overlap.json", name);
        assert_eq!(expect["execution"]["status"], "complete", "{name}");
        assert_eq!(expect["diagnostic_codes"], json!([]), "{name}");
        assert_eq!(
            expect["semantic_validation"], "local_invariants",
            "{name}: the names over these frames completed, so their invariants held"
        );
    }
}

#[test]
fn the_wide_and_switch_entry_completes() {
    assert_golden_file("wide-switch.json");
    let expect = recorded_expect("wide-switch.json", "wide-forms-and-both-switches");
    assert_eq!(expect["execution"]["status"], "complete");
    assert_eq!(expect["diagnostic_codes"], json!([]));
    assert!(
        expect["usage"]["ir_edges"].as_u64().expect("an edge count") > 0,
        "the switch targets are blocks and edges"
    );
}

#[test]
fn the_resource_boundary_entries_stop_where_their_budget_says() {
    assert_golden_file("resource-boundary.json");

    let over = recorded_expect("resource-boundary.json", "over-budget-r10-items");
    assert_eq!(over["execution"]["status"], "partial");
    assert_eq!(
        over["execution"]["reason"],
        json!({"kind": "budget_exceeded", "dimension": "ir_items"})
    );
    assert_eq!(
        over["diagnostic_codes"],
        json!(["budget_exceeded_ir_items"])
    );
    assert_eq!(
        over["published_prefix"],
        json!([
            "raw_facts",
            "raw_cfg",
            "legacy_normalization",
            "canonical_cfg"
        ]),
        "the canonical graph the prefix produced is still the published artifact"
    );
    assert_eq!(
        over["quality"], "conservative",
        "a published canonical graph makes the run conservative even when it stops after it"
    );

    let steps = recorded_expect("resource-boundary.json", "deep-chain-steps");
    assert_eq!(steps["execution"]["status"], "partial");
    assert_eq!(
        steps["execution"]["reason"],
        json!({"kind": "budget_exceeded", "dimension": "analysis_steps"})
    );
    assert_eq!(
        steps["diagnostic_codes"],
        json!(["budget_exceeded_analysis_steps"])
    );
    assert_eq!(
        steps["published_prefix"]
            .as_array()
            .expect("a prefix")
            .len(),
        5
    );
    assert_eq!(steps["semantic_validation"], "unproven");

    let fanout = recorded_expect("resource-boundary.json", "high-fanout-edges");
    assert_eq!(fanout["execution"]["status"], "partial");
    assert_eq!(
        fanout["execution"]["reason"],
        json!({"kind": "budget_exceeded", "dimension": "ir_edges"})
    );
    assert_eq!(
        fanout["diagnostic_codes"],
        json!(["budget_exceeded_ir_edges"])
    );

    let cancelled = recorded_expect("resource-boundary.json", "cancelled-before-the-first-read");
    assert_eq!(cancelled["execution"]["status"], "cancelled");
    assert_eq!(cancelled["diagnostic_codes"], json!(["cancelled"]));
    assert_eq!(cancelled["published_prefix"], json!([]));
    assert_eq!(cancelled["quality"], "fallback");
    assert_eq!(cancelled["semantic_validation"], "unproven");

    // R10's measured points: the retained per-site state grows with the number of covered throw
    // sites, and the numbers themselves are pinned in the entries above and in the constants the
    // boundary limit is derived from.
    let eight = recorded_expect("resource-boundary.json", "r10-frame-slot-product-8-sites");
    let sixty_four = recorded_expect("resource-boundary.json", "r10-frame-slot-product-64-sites");
    let without_handler = recorded_expect(
        "resource-boundary.json",
        "r10-frames-without-a-handler-retain-far-less",
    );
    let canonical = recorded_expect("resource-boundary.json", "r10-canonical-price");
    let named = recorded_expect(
        "resource-boundary.json",
        "r10-slot-product-8-sites-with-names",
    );
    let items = |expect: &Value| expect["usage"]["ir_items"].as_u64().expect("an item count");
    assert_eq!(items(&canonical), R10_CANONICAL_ITEMS);
    assert_eq!(items(&eight), R10_FRAME_ITEMS_8_SITES);
    assert_eq!(items(&sixty_four), R10_FRAME_ITEMS_64_SITES);
    assert_eq!(items(&without_handler), R10_FRAME_ITEMS_8_SITES_NO_HANDLER);
    assert_eq!(items(&named), R10_NAMES_ITEMS_8_SITES);
    assert_eq!(
        items(&eight) - items(&canonical),
        280_027,
        "R10's own measurement: eight covered throw sites over 10000 locals cost 280027 items \
         above the canonical graph of the same body"
    );
    assert_eq!(
        items(&sixty_four) - items(&eight),
        56 * 30_009,
        "56 more covered throw sites retain 56 more per-site states of 30009 items each"
    );
    assert!(
        items(&without_handler) < items(&eight) / 4,
        "without a handler to read them the per-site states are not retained at all: {} vs {}",
        items(&without_handler),
        items(&eight)
    );
    assert!(
        items(&named) > items(&eight),
        "the names are charged on top of the frames: {} vs {}",
        items(&named),
        items(&eight)
    );
    assert_eq!(eight["execution"]["status"], "complete");
    assert_eq!(sixty_four["execution"]["status"], "complete");
}

/// The premises three entries are about, checked against the bytes themselves instead of being
/// taken on trust: the missing-debug body and the old dialect of the historical corpus really
/// carry no debug or stack-map table, the newer dialect does, and the missing-dependency body
/// really names a class the snapshot does not hold.
///
/// The two halves matter together: the frames of these entries are derived from the descriptor
/// and the body, and they are derived the same way whether the class file carries a
/// `StackMapTable` (mandatory from major 50 in the corpus's compiler output) or nothing at all.
#[test]
fn the_input_side_premises_of_the_entries_hold() {
    const DEBUG_TABLES: [&[u8]; 3] = [b"StackMapTable", b"LineNumberTable", b"LocalVariableTable"];

    for fixture in ["missing-debug-loop", "historical-45", "historical-49"] {
        let bytes = fixture_bytes(fixture);
        for table in DEBUG_TABLES {
            assert!(
                !bytes.windows(table.len()).any(|window| window == table),
                "{fixture} must carry no {:?} entry: the frames of this entry are derived",
                String::from_utf8_lossy(table)
            );
        }
    }
    // The premise of the missing-debug entry is not vacuous: the same corpus at major 52 carries
    // the table the JVM requires from 51, and the golden replays the same request over it.
    let modern = fixture_bytes("historical-52");
    assert!(
        modern
            .windows(b"StackMapTable".len())
            .any(|window| window == b"StackMapTable"),
        "the 52 class file carries the StackMapTable it must, and the analysis does not need it"
    );

    // The bodies really decode into instructions, so "no debug table" is a fact about a readable
    // body rather than about bytes nothing can open.
    for (fixture, method) in [
        ("missing-debug-loop", ("method", "()I")),
        ("historical-45", ("finallyPath", "(I)I")),
        ("historical-52", ("finallyPath", "(I)I")),
    ] {
        let opened = open(fixture, method);
        let mut budget = Budget::new(ample());
        let inspection = Engine::new()
            .inspect_method_bytecode(
                &opened.snapshot,
                ClassTarget::Root,
                MethodSelector {
                    name: opened.method.name.clone(),
                    descriptor: opened.method.descriptor.clone(),
                },
                &mut budget,
            )
            .expect("the fixture's method is inspectable");
        assert!(
            !inspection.inspection.instructions.is_empty(),
            "{fixture}: the body decodes into instructions"
        );
    }

    let bytes = fixture_bytes("missing-dependency");
    assert!(
        bytes
            .windows(b"p/Missing".len())
            .any(|window| window == b"p/Missing"),
        "the missing-dependency body names p/Missing"
    );
    // ... and the snapshot it is analyzed in holds that one class and nothing else, so the name
    // really is missing rather than resolvable somewhere in the input.
    let expect = recorded_expect("inputs.json", "missing-dependency-completes");
    assert_eq!(
        expect["usage"]["class_headers"], 1,
        "the request read one class header: the one it was pointed at"
    );
    assert_eq!(
        expect["usage"]["method_bodies"], 1,
        "and one body: the driver's own"
    );
}

#[test]
fn every_category_of_the_fixed_list_is_carried_by_a_named_entry() {
    let list = replay_list();
    for (category, entry) in categories() {
        let replay = list
            .iter()
            .find(|replay| replay.name == *entry)
            .unwrap_or_else(|| panic!("the list has no entry named {entry:?}"));
        assert!(
            replay.category.contains(category),
            "{entry} carries the category {:?}, not {category:?}",
            replay.category
        );
    }
    // The task's list in full, so a category that loses its entry fails here even when the
    // remaining categories are still covered.
    assert_eq!(
        categories().len(),
        12,
        "the fixed list covers twelve named categories"
    );
}

#[test]
fn every_stopped_entry_across_the_goldens_explains_itself() {
    let mut stopped = 0;
    for file in GOLDEN_FILES {
        let golden = golden(file);
        for recorded in golden_replays(&golden) {
            let name = recorded["name"].as_str().expect("a replay name");
            let status = recorded["expect"]["execution"]["status"]
                .as_str()
                .expect("an execution status");
            if status == "complete" {
                continue;
            }
            stopped += 1;
            assert!(
                !recorded["expect"]["diagnostic_codes"]
                    .as_array()
                    .expect("diagnostic codes")
                    .is_empty(),
                "golden {file} entry {name}: a stop must carry a diagnostic"
            );
            assert!(
                recorded["expect"]["stages"]
                    .as_array()
                    .expect("stage results")
                    .iter()
                    .any(|stage| stage["state"]["kind"] != "completed"),
                "golden {file} entry {name}: a stopped run has a stage that did not complete"
            );
        }
    }
    assert_eq!(
        stopped, 5,
        "the fixed list carries five stopped entries: the refused dialect, over budget, the deep \
         chain, the high fanout, and the cancelled request"
    );
}

fn golden_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/p2-golden")
        .join(file)
}

fn golden(file: &str) -> Value {
    let path = golden_path(file);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "golden {} is missing or unreadable: {error}; goldens are checked-in expectations, \
             they are never generated by a test run",
            path.display()
        )
    });
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("golden {} is not valid JSON: {error}", path.display()))
}

/// The replay entries one golden file records, in file order.
fn golden_replays(golden: &Value) -> &[Value] {
    golden["replays"]
        .as_array()
        .expect("the golden records its replays as an array")
}

fn golden_replay<'a>(golden: &'a Value, name: &str) -> &'a Value {
    golden_replays(golden)
        .iter()
        .find(|replay| replay["name"] == name)
        .unwrap_or_else(|| panic!("the golden has no replay named {name:?}"))
}

/// Deletes `elapsed_millis` at any depth: the one field whose value depends on wall-clock time.
fn normalize(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("elapsed_millis");
            for child in map.values_mut() {
                normalize(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize(item);
            }
        }
        _ => {}
    }
}

/// The state and stage invariants of one replay, read off the typed report: what the entry
/// exists to assert, in the shape the golden records it.
///
/// The planes are the caller-visible ones; `published_prefix` is the leading run of `Completed`
/// stages, which is what a stopped run has already published, and it is empty exactly when the
/// run stopped before its first phase. `origin` is deliberately absent: P2 publishes no IR
/// payload, so the report's `origin` is empty for every entry and the clone origin mapping is
/// asserted inside the crate.
fn published_planes(report: &MethodAnalysisReport, usage: &UsageSnapshot) -> Value {
    let mut execution = serde_json::to_value(&report.execution).expect("execution serializes");
    if let Value::Object(map) = &mut execution {
        map.remove("usage");
    }
    let mut usage = serde_json::to_value(usage).expect("usage serializes");
    if let Value::Object(map) = &mut usage {
        map.remove("elapsed_millis");
    }
    let prefix: Vec<&AnalysisStage> = report
        .stages
        .iter()
        .take_while(|stage| stage.state == StageState::Completed)
        .map(|stage| &stage.stage)
        .collect();
    json!({
        "body": report.body,
        "compile_status": report.compile_status,
        "coverage": report.coverage,
        "diagnostic_codes": report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect::<Vec<_>>(),
        "execution": execution,
        "published_prefix": prefix,
        "quality": report.quality,
        "reads": {
            "count": report.reads.len(),
            "reasons": report
                .reads
                .iter()
                .map(|read| read.reason)
                .collect::<Vec<_>>(),
        },
        "representation": report.representation,
        "requested_stages": report.requested_stages,
        "semantic_validation": report.semantic_validation,
        "stages": report.stages,
        "syntax_status": report.syntax_status,
        "usage": usage,
        "verification": report.verification,
    })
}

/// One fixture body: the bytes, the stack and local budget the class file declares, and the
/// exception table (catch-all records only — the CP-free fixtures name no exception class).
struct Body {
    code: Vec<u8>,
    max_stack: u16,
    max_locals: u16,
    handlers: Vec<(u16, u16, u16)>,
}

/// A real class file of one class `Test extends java/lang/Object` whose single method declares
/// `access`, `name` and `descriptor` and carries `body`, at class-file version `major`.
///
/// The pool holds exactly the five entries such a method needs — `Test`, its superclass, the
/// member's name and descriptor, and `Code` — and no debug attribute of any kind, which is what
/// the missing-debug entry is about.
fn simple_class(major: u16, access: u16, name: &[u8], descriptor: &[u8], body: &Body) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_name = pool.utf8(b"Test");
    let this_class = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let name_index = pool.utf8(name);
    let descriptor_index = pool.utf8(descriptor);
    let code_name = pool.utf8(b"Code");
    assemble(
        &pool,
        major,
        0,
        0x0021,
        this_class,
        object_class,
        &[Method {
            access,
            name: name_index,
            descriptor: descriptor_index,
            code_name,
            max_stack: body.max_stack,
            max_locals: body.max_locals,
            code: body.code.clone(),
            handlers: body
                .handlers
                .iter()
                .map(|(start, end, handler)| Handler {
                    start: *start,
                    end: *end,
                    handler: *handler,
                    catch_type: 0,
                })
                .collect(),
        }],
    )
}

/// The committed historical corpus: ECJ 4.6.1 output of `HistoricalControlFlow.java` for the
/// class-file version `version`, compiled with `-g:none` (no `LineNumberTable`,
/// `LocalVariableTable` or `StackMapTable` anywhere in it).
fn historical(version: u16) -> &'static [u8] {
    match version {
        45 => include_bytes!("fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class"),
        46 => include_bytes!("fixtures/historical/ecj-4.6.1/v46/HistoricalControlFlow.class"),
        47 => include_bytes!("fixtures/historical/ecj-4.6.1/v47/HistoricalControlFlow.class"),
        48 => include_bytes!("fixtures/historical/ecj-4.6.1/v48/HistoricalControlFlow.class"),
        49 => include_bytes!("fixtures/historical/ecj-4.6.1/v49/HistoricalControlFlow.class"),
        50 => include_bytes!("fixtures/historical/ecj-4.6.1/v50/HistoricalControlFlow.class"),
        51 => include_bytes!("fixtures/historical/ecj-4.6.1/v51/HistoricalControlFlow.class"),
        52 => include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
        other => panic!("the historical corpus has no version {other}"),
    }
}

/// The body of the `jsr`-era fixtures: two call sites (BCI 0 and BCI 3) enter **one** subroutine
/// at BCI 7, which stores the return address in local 0 and returns through it. The same bytes
/// are built at major 50 and at major 52, where the opcode is a dialect violation: the two
/// entries differ in the version alone.
fn jsr_body() -> Vec<u8> {
    let mut asm = Asm::default();
    asm.jsr("subroutine"); // 0: the first call site
    asm.jsr("subroutine"); // 3: the second call site
    asm.op(&[0xb1]); // 6: return (the continuation of the second call)
    asm.mark("subroutine");
    asm.op(&[0x4b]); // 7: astore_0
    asm.op(&[0xa9, 0x00]); // 8: ret 0
    asm.code()
}

fn jsr_class(major: u16) -> Vec<u8> {
    let code = jsr_body();
    assert_eq!(code, jsr_body(), "the body assembles deterministically");
    simple_class(
        major,
        0x0009,
        b"method",
        b"()V",
        &Body {
            code,
            max_stack: 1,
            max_locals: 1,
            handlers: Vec::new(),
        },
    )
}

/// A method with a loop but **no** debug attribute at all: no `StackMapTable`, no
/// `LineNumberTable`, no local-variable table. The frames have to be derived from the descriptor
/// and the body, and the entry exists to pin that they are.
fn missing_debug_loop_class() -> Vec<u8> {
    let mut asm = Asm::default();
    asm.op(&[0x03, 0x3b]); // 0: iconst_0; 1: istore_0
    asm.mark("head");
    asm.op(&[0x1a]); // 2: iload_0
    asm.op(&[0x06]); // 3: iconst_3
    asm.branch(0xa2, "end"); // 4: if_icmpge end
    asm.op(&[0x84, 0x00, 0x01]); // 7: iinc 0, 1
    asm.goto("head"); // 10: goto head
    asm.mark("end");
    asm.op(&[0x1a, 0xac]); // 13: iload_0; 14: ireturn
    simple_class(
        52,
        0x0009,
        b"method",
        b"()I",
        &Body {
            code: asm.code(),
            max_stack: 2,
            max_locals: 1,
            handlers: Vec::new(),
        },
    )
}

/// A body that names a class the snapshot does not hold: `new p/Missing`, its constructor call,
/// and a static call on it. Nothing in the IR path resolves the owner, and the entry pins that
/// the pipeline completes over a missing dependency instead of inventing a failure for it.
fn missing_dependency_class() -> Vec<u8> {
    let mut pool = Pool::default();
    let this_name = pool.utf8(b"Test");
    let this_class = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let name_index = pool.utf8(b"method");
    let descriptor_index = pool.utf8(b"()V");
    let code_name = pool.utf8(b"Code");
    let missing_name = pool.utf8(b"p/Missing");
    let missing_class = pool.class(missing_name);
    let init = pool.utf8(b"<init>");
    let init_nat = pool.name_and_type(init, descriptor_index);
    let init_ref = pool.member(10, missing_class, init_nat);
    let tail = pool.utf8(b"tail");
    let tail_nat = pool.name_and_type(tail, descriptor_index);
    let tail_ref = pool.member(10, missing_class, tail_nat);

    let mut code = Vec::new();
    code.push(0xbb); // new p/Missing
    cp(&mut code, missing_class);
    code.push(0x59); // dup
    code.push(0xb7); // invokespecial p/Missing.<init>()V
    cp(&mut code, init_ref);
    code.push(0x57); // pop
    code.push(0xb8); // invokestatic p/Missing.tail()V
    cp(&mut code, tail_ref);
    code.push(0xb1); // return

    assemble(
        &pool,
        52,
        0,
        0x0021,
        this_class,
        object_class,
        &[Method {
            access: 0x0009,
            name: name_index,
            descriptor: descriptor_index,
            code_name,
            max_stack: 2,
            max_locals: 0,
            code,
            handlers: Vec::new(),
        }],
    )
}

/// Two overlapping catch-all records over the same two throw sites, and a handler whose own body
/// throws inside a third record: the nested-handler shape.
///
/// ```text
///  0: iconst_1; iconst_0; idiv; pop        site A, covered by records 0 and 1
///  4: iconst_1; iconst_0; idiv; pop        site B, covered by records 0 and 1
///  8: goto 11
/// 11: return
/// 12: pop; iconst_1; iconst_0; idiv; pop   handler 0 — its `idiv` is site C, covered by record 2
/// 18: pop; return                          handler 2
/// exception table: [0,11)->12, [1,8)->12, [12,17)->18   (all catch-all)
/// ```
fn exception_overlap_nested_class() -> Vec<u8> {
    let mut asm = Asm::default();
    asm.op(&[0x04, 0x03, 0x6c, 0x57]); // 0: iconst_1; iconst_0; idiv; pop
    asm.op(&[0x04, 0x03, 0x6c, 0x57]); // 4: iconst_1; iconst_0; idiv; pop
    asm.goto("end"); // 8: goto end
    asm.mark("end");
    asm.op(&[0xb1]); // 11: return
    asm.mark("h0");
    asm.op(&[0x57, 0x04, 0x03, 0x6c, 0x57]); // 12: pop; iconst_1; iconst_0; idiv; pop
    asm.op(&[0xb1]); // 17: return
    asm.mark("h2");
    asm.op(&[0x57, 0xb1]); // 18: pop; return
    let code = asm.code();
    let end = asm.label("end");
    let h0 = asm.label("h0");
    let h2 = asm.label("h2");
    assert_eq!((end, h0, h2), (11, 12, 18));
    simple_class(
        52,
        0x0009,
        b"method",
        b"()V",
        &Body {
            code,
            max_stack: 2,
            max_locals: 0,
            handlers: vec![(0, end, h0), (1, 8, h0), (h0, h0 + 5, h2)],
        },
    )
}

/// One record whose range starts **inside** a block (`[1,4)`) and one that starts on a block
/// (`[7,10)`), each covering one throwing instruction: the shape 4.2b's third fix closed.
fn exception_mid_block_range_class() -> Vec<u8> {
    const CODE: &[u8] = &[
        0x04, 0x03, 0x6c, 0x57, // 0: iconst_1; iconst_0; idiv; pop
        0xa7, 0x00, 0x03, // 4: goto 7
        0x04, 0x03, 0x6c, 0x57, // 7: iconst_1; iconst_0; idiv; pop
        0xb1, // 11: return
        0x57, 0xb1, // 12: pop; return
        0x57, 0xb1, // 14: pop; return
    ];
    simple_class(
        49,
        0x0009,
        b"method",
        b"()V",
        &Body {
            code: CODE.to_vec(),
            max_stack: 2,
            max_locals: 1,
            handlers: vec![(1, 4, 12), (7, 10, 14)],
        },
    )
}

/// The descriptor of the exception fixtures that carry a reference through the handler.
const EXCEPTION_METHOD: &[u8] = b"(Ljava/lang/Object;)Ljava/lang/Object;";

/// R9's counterexample: the exception edge feeds a block that has already been processed, and
/// the throw site's locals differ from the block's exit on the first visit. The bytes are a body
/// an OpenJDK runs and returns `null` for.
fn exception_back_edge_class() -> Vec<u8> {
    const CODE: &[u8] = &[
        0x01, 0x4c, // 0: aconst_null; 1: astore_1
        0x04, 0x03, 0x6c, 0x57, // 2: iconst_1; iconst_0; idiv; pop
        0x2a, 0x4c, 0x2a, 0xc7, 0xff,
        0xf9, // 6: aload_0; 7: astore_1; 8: aload_0; 9: ifnonnull 2
        0x2b, 0xb0, // 12: aload_1; 13: areturn
        0x57, 0xa7, 0xff, 0xf3, // 14: pop; 15: goto 2
    ];
    assert_eq!(i32::from(i16::from_be_bytes([CODE[10], CODE[11]])), -7);
    assert_eq!(i32::from(i16::from_be_bytes([CODE[16], CODE[17]])), -13);
    simple_class(
        49,
        0x0009,
        b"method",
        EXCEPTION_METHOD,
        &Body {
            code: CODE.to_vec(),
            max_stack: 2,
            max_locals: 2,
            handlers: vec![(2, 12, 14)],
        },
    )
}

/// Two throw sites of **one** block, both feeding one handler, with the loop exit stable across
/// visits: one raw edge, two logical inputs.
fn exception_two_sites_class() -> Vec<u8> {
    const CODE: &[u8] = &[
        0x01, 0x4c, // 0: aconst_null; 1: astore_1
        0x04, 0x03, 0x6c, 0x57, // 2: iconst_1; iconst_0; idiv; pop
        0x2a, 0x4c, 0x2a, 0x4d, // 6: aload_0; 7: astore_1; 8: aload_0; 9: astore_2
        0x04, 0x03, 0x6c, 0x57, // 10: iconst_1; iconst_0; idiv; pop
        0x2a, 0xc7, 0xff, 0xf3, // 14: aload_0; 15: ifnonnull 2
        0x2b, 0xb0, // 18: aload_1; 19: areturn
        0x57, 0x2b, 0xb0, // 20: pop; 21: aload_1; 22: areturn
    ];
    assert_eq!(i32::from(i16::from_be_bytes([CODE[16], CODE[17]])), -13);
    simple_class(
        49,
        0x0009,
        b"method",
        EXCEPTION_METHOD,
        &Body {
            code: CODE.to_vec(),
            max_stack: 2,
            max_locals: 3,
            handlers: vec![(2, 18, 20)],
        },
    )
}

/// R9 itself, in the 17 bytes the review reported it in.
fn r9_class() -> Vec<u8> {
    const CODE: &[u8] = &[
        0x01, 0x4c, // 0: aconst_null; 1: astore_1
        0x04, 0x03, 0x6c, 0x57, // 2: iconst_1; iconst_0; idiv; pop
        0x2a, 0x4c, 0x03, 0x99, 0xff, 0xf9, // 6: aload_0; 7: astore_1; 8: iconst_0; 9: ifeq 2
        0x2b, 0xb0, // 12: aload_1; 13: areturn
        0x57, 0x2b, 0xb0, // 14: pop; 15: aload_1; 16: areturn
    ];
    assert_eq!(i32::from(i16::from_be_bytes([CODE[10], CODE[11]])), -7);
    simple_class(
        49,
        0x0009,
        b"method",
        EXCEPTION_METHOD,
        &Body {
            code: CODE.to_vec(),
            max_stack: 2,
            max_locals: 2,
            handlers: vec![(2, 12, 14)],
        },
    )
}

/// The wide forms and both switch forms in one body: a local above 255 is stored, read and
/// incremented through `wide`, then read once more and used by a `tableswitch` whose case for the
/// settled value falls into a `lookupswitch`.
///
/// ```text
///  0: iconst_1;     1: wide istore 300       local 300 = 1
///  4: wide iinc 300, -3                      local 300 = -2
///  8: wide iload 300
/// 11: tableswitch(-2..1) { -2: t0, -1: t1, 0: t2, 1: t3 }, default: d
///    t2: iconst_1; lookupswitch({1: a, 7: b, 300: c}), default: d
///    t0, t1, t3, a, b, c, d: return
/// ```
fn wide_and_switch_class() -> Vec<u8> {
    let mut asm = Asm::default();
    asm.op(&[0x04]); // 0: iconst_1
    asm.wide_local(0x36, 300); // 1: wide istore 300
    asm.wide_iinc(300, -3); // 4: wide iinc 300, -3
    asm.wide_local(0x15, 300); // 8: wide iload 300
    asm.tableswitch(-2, &["t0", "t1", "t2", "t3"], "d"); // 11: tableswitch
    asm.mark("t0");
    asm.op(&[0xb1]);
    asm.mark("t1");
    asm.op(&[0xb1]);
    asm.mark("t2");
    asm.op(&[0x04]); // iconst_1
    asm.lookupswitch(&[(1, "a"), (7, "b"), (300, "c")], "d");
    asm.mark("t3");
    asm.op(&[0xb1]);
    asm.mark("d");
    asm.op(&[0xb1]);
    asm.mark("a");
    asm.op(&[0xb1]);
    asm.mark("b");
    asm.op(&[0xb1]);
    asm.mark("c");
    asm.op(&[0xb1]);
    simple_class(
        52,
        0x0009,
        b"method",
        b"()V",
        &Body {
            code: asm.code(),
            max_stack: 1,
            max_locals: 301,
            handlers: Vec::new(),
        },
    )
}

/// A chain of `blocks` blocks, each entered only through the conditional branch of the one
/// before it and each also holding its own `return`: a body whose control flow is deep rather
/// than wide, and whose blocks the canonical fusion cannot absorb (every block has two
/// successors).
fn deep_chain_class(blocks: usize) -> Vec<u8> {
    let mut asm = Asm::default();
    for index in 0..blocks {
        asm.mark(&format!("l{index}"));
        asm.op(&[0x03]); // iconst_0: the condition the branch takes
        if index + 1 == blocks {
            asm.op(&[0xb1]); // the last block returns
        } else {
            asm.ifeq(&format!("l{}", index + 1));
            asm.op(&[0xb1]); // this block's own exit
        }
    }
    simple_class(
        52,
        0x0009,
        b"method",
        b"()V",
        &Body {
            code: asm.code(),
            max_stack: 1,
            max_locals: 0,
            handlers: Vec::new(),
        },
    )
}

/// One `tableswitch` of `cases` targets: a single block with that many successors, each of them
/// its own block.
fn high_fanout_class(cases: usize) -> Vec<u8> {
    let names: Vec<String> = (0..cases).map(|index| format!("c{index}")).collect();
    let targets: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut asm = Asm::default();
    asm.op(&[0x03]); // 0: iconst_0
    asm.tableswitch(0, &targets, "default");
    for target in &targets {
        asm.mark(target);
        asm.op(&[0xb1]);
    }
    asm.mark("default");
    asm.op(&[0xb1]);
    simple_class(
        52,
        0x0009,
        b"method",
        b"()V",
        &Body {
            code: asm.code(),
            max_stack: 1,
            max_locals: 0,
            handlers: Vec::new(),
        },
    )
}

/// The R10 shape: `sites` throwing instructions in **one** block, `locals` local slots, and — when
/// `handler` — a catch-all record covering every one of those sites.
///
/// `04 04 6c 57` is `iconst_1; iconst_1; idiv; pop`, so each repetition holds one canonical throw
/// site. `max_locals` is what makes the product of sites and locals the thing the budget bounds,
/// and it is the input 4.3b was measured on.
fn throw_sites_class(sites: usize, locals: u16, handler: bool) -> Vec<u8> {
    const REPEAT: &[u8] = &[0x04, 0x04, 0x6c, 0x57];
    let mut code = Vec::new();
    for _ in 0..sites {
        code.extend_from_slice(REPEAT);
    }
    let end = u16::try_from(code.len()).expect("a fixture body fits u16");
    let handlers = if handler {
        code.push(0xb1); // the body's own return
        let handler_pc = u16::try_from(code.len()).expect("a fixture body fits u16");
        code.extend_from_slice(&[0x57, 0xb1]); // pop; return
        vec![(0, end, handler_pc)]
    } else {
        code.push(0xb1);
        Vec::new()
    };
    simple_class(
        49,
        0x0009,
        b"method",
        b"()V",
        &Body {
            code,
            max_stack: 2,
            max_locals: locals,
            handlers,
        },
    )
}

/// The bytes of the one fixture a replay names.
fn fixture_bytes(name: &str) -> Vec<u8> {
    match name {
        "historical-45" => historical(45).to_vec(),
        "historical-46" => historical(46).to_vec(),
        "historical-47" => historical(47).to_vec(),
        "historical-48" => historical(48).to_vec(),
        "historical-49" => historical(49).to_vec(),
        "historical-50" => historical(50).to_vec(),
        "historical-51" => historical(51).to_vec(),
        "historical-52" => historical(52).to_vec(),
        "jsr-body-v50" => jsr_class(50),
        "jsr-body-v52" => jsr_class(52),
        "missing-debug-loop" => missing_debug_loop_class(),
        "missing-dependency" => missing_dependency_class(),
        "exception-overlap-nested" => exception_overlap_nested_class(),
        "exception-mid-block-range" => exception_mid_block_range_class(),
        "exception-back-edge" => exception_back_edge_class(),
        "exception-two-sites" => exception_two_sites_class(),
        "r9-exception-input" => r9_class(),
        "wide-and-switch" => wide_and_switch_class(),
        "deep-chain" => deep_chain_class(64),
        "high-fanout" => high_fanout_class(64),
        "throw-sites-8" => throw_sites_class(8, 10_000, true),
        "throw-sites-64" => throw_sites_class(64, 10_000, true),
        "throw-sites-8-no-handler" => throw_sites_class(8, 10_000, false),
        other => panic!("the golden fixture {other:?} has no builder"),
    }
}

/// Where one fixture's bytes come from, recorded in the golden next to its digest.
fn fixture_source(name: &str) -> &'static str {
    match name {
        "historical-45" | "historical-46" | "historical-47" | "historical-48" | "historical-49"
        | "historical-50" | "historical-51" | "historical-52" => {
            "tests/fixtures/historical/ecj-4.6.1/vNN/HistoricalControlFlow.class (committed, \
             compiled by ECJ 4.6.1 with -g:none; provenance in tests/fixtures/historical/README.md)"
        }
        "jsr-body-v50" | "jsr-body-v52" => {
            "hand-built in tests/p2_golden.rs::jsr_class: two `jsr` call sites sharing one \
             subroutine (`astore_0`; `ret 0`), identical bytes at major 50 and major 52"
        }
        "missing-debug-loop" => {
            "hand-built in tests/p2_golden.rs::missing_debug_loop_class: a loop whose class file \
             carries no StackMapTable, LineNumberTable or local-variable table"
        }
        "missing-dependency" => {
            "hand-built in tests/p2_golden.rs::missing_dependency_class: `new p/Missing`, its \
             constructor call and a static call on it, with p/Missing in no snapshot"
        }
        "exception-overlap-nested" => {
            "hand-built in tests/p2_golden.rs::exception_overlap_nested_class: two overlapping \
             catch-all records over the same sites, and a handler whose own throw is covered by a \
             third record"
        }
        "exception-mid-block-range" => {
            "hand-built in tests/p2_golden.rs::exception_mid_block_range_class: one record whose \
             range starts inside a block ([1,4)), one that starts on a block ([7,10))"
        }
        "exception-back-edge" => {
            "hand-built in tests/p2_golden.rs::exception_back_edge_class: the exception edge runs \
             back into an already-processed loop head (4.2b's back-edge contrast)"
        }
        "exception-two-sites" => {
            "hand-built in tests/p2_golden.rs::exception_two_sites_class: two throw sites of one \
             block under one record (4.2b's multi-site contrast)"
        }
        "r9-exception-input" => {
            "hand-built in tests/p2_golden.rs::r9_class: the 17 bytes of the R9 counterexample"
        }
        "wide-and-switch" => {
            "hand-built in tests/p2_golden.rs::wide_and_switch_class: `wide istore/iload/iinc` on \
             local 300, a `tableswitch` and a `lookupswitch`"
        }
        "deep-chain" => {
            "hand-built in tests/p2_golden.rs::deep_chain_class(64): 64 blocks in a chain, each \
             with its own exit"
        }
        "high-fanout" => {
            "hand-built in tests/p2_golden.rs::high_fanout_class(64): one `tableswitch` block with \
             64 successors"
        }
        "throw-sites-8" | "throw-sites-64" | "throw-sites-8-no-handler" => {
            "hand-built in tests/p2_golden.rs::throw_sites_class: the R10 shape (N throwing \
             instructions in one block, 10000 locals, a catch-all record over them)"
        }
        other => panic!("the golden fixture {other:?} has no recorded source"),
    }
}
