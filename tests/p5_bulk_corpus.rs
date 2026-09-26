//! P5 corpus and billing ledger (change `add-parallel-bulk-recovery`, task 1.3): one small,
//! regenerable corpus that holds the shapes the task names, and the counted proxies of what the
//! bulk operation is charged for each of them at that fixed shape.
//!
//! # What this file is for
//!
//! Task 1.3 asks for two things before the performance work of 6.x can mean anything: a corpus small
//! enough to run in CI that really holds the shapes the change's arms are about, and a record of the
//! **counted** work the operation does on it. The second half is the reason the first exists: a count
//! is only comparable to another count when both were produced under the same declared shape, so the
//! shape has to be stated, pinned and re-derivable rather than described.
//!
//! # The corpus
//!
//! Every archive is built **in process** by the builders below out of the repository's own committed
//! `.class` bytes (the [`bulk_support`] fixtures and the four samples this file includes); nothing is
//! checked in as a new binary, nothing is read from `/tmp`, and no compiler, JDK, network or ambient
//! file is needed. The ZIP writer is the repository's `rawzip` dev-dependency, exactly as the existing
//! bulk fixtures use it, so these are real archives read through the production enumeration path.
//!
//! | case | the shape it is the carrier of |
//! | --- | --- |
//! | `flat-mixed` | one flat archive, its four class entries alternating STORED and DEFLATED |
//! | `nested-mixed` | classes at the root, in a STORED nested container and in a DEFLATED one, whose own entries use the opposite method from their container |
//! | `two-origins-one-identity` | the same nested bytes at two entries (one STORED, one DEFLATED), so one class file is reachable under two physical origins |
//! | `many-method-class` | 64 generated members in one class beside the widest committed classes (24 and 19): the large `M` the task asks for, and the sample a slow consumer is run over |
//! | `damaged-tail` | a class entry whose bytes are a truncated class file, a nested container whose tail (its central directory) was cut off, and a nested entry that is not a container at all — beside two classes that still recover |
//! | `deep-expression` | a generated straight-line class with a 16-level `+` tree and a 64-level one — the layer's own rendering ceiling, and one past it — beside the committed nested-evaluation sample whose two members are refusals |
//!
//! The corpus is *regenerable* in the strict sense: every builder is a total function of the
//! committed bytes, every test calls the builders again, and
//! [`the_corpus_is_rebuilt_from_the_committed_bytes_every_time`] states that two calls agree byte for
//! byte. What this corpus deliberately does **not** hold is stated as a gap rather than implied to be
//! covered: a real WAR or Boot tree, a multi-release jar, a resources-heavy archive, and an artifact
//! big enough to be a workload all stay the job of the change's 6.x measurements. This file is the
//! fixed small shape those measurements' *counts* are read against, not a substitute for them.
//!
//! # The counts, and what they are not
//!
//! [`Billing`] is nine counted dimensions of one run, read from the operation's own usage. Each
//! number answers exactly one question — *how many of this unit did this run, under this scope, this
//! declared classpath profile and this engine version, ask the budget to count* — and nothing else:
//!
//! * it is **not a duration**. No wall-clock reading appears in an assertion anywhere in this file:
//!   the elapsed figures are printed beside the counts, under their own label, because a run on a
//!   loaded or unloaded machine produces a different one for the same work, and because a smaller
//!   count is not a faster run. A count falling between two shapes is a statement about how the work
//!   was addressed, never about how long it took;
//! * it is **not comparable across shapes**. `openspec/benchmark-protocol.md` fixes the rule for
//!   every comparison row this change publishes: each row names its request shape, its declared
//!   roots/profile and its engine version, and states what the number counts in *that* scope. The
//!   rows pinned here are per-case rows under [`bulk_support::limits`], one worker and the case's own
//!   prefixes — two rows of this table may not be subtracted from each other, and a case's numbers
//!   may not be compared with the 6.x artifact measurements;
//! * it is **not a retention, memory or RSS reading**. What one shared store kept is the store's own
//!   report, read in the arms test below and named as such.
//!
//! What a count is good for here is a fixed-shape regression: if the operation starts reading a
//! container directory twice per class, or decoding a body twice, the pinned number for the case that
//! holds that shape moves, and the case's name says which shape moved. That is the whole claim, and
//! it is why the table is asserted rather than merely printed.
//!
//! # The two old arms
//!
//! The change's arms A and B (`openspec/benchmark-protocol.md`; the design's table in §10) are the
//! *previous* way of running the very same work: one request per method, one budget per request, the
//! second arm carrying one shared facts store that starts empty. Both are kept here as historical
//! anchors — the old path's real ledger, at the same fixed shape the bulk operation is billed at —
//! and as a semantic check: the three paths must present the same member text and the same physical
//! identity, because the new operation is a new way to run one pipeline, not a second pipeline. Their
//! totals are the same kind of number as [`Billing`] and carry the same warning: recorded so a later
//! run can be compared *at this shape*, never read as a speed.
//!
//! # Running it
//!
//! ```text
//! counts, both arms:  cargo test --test p5_bulk_corpus --locked -- --nocapture
//! regenerate a pin:   cargo test --test p5_bulk_corpus --locked -- --ignored --nocapture record_the_billing_table
//! ```
//!
//! No case needs a JDK, a network, a compiler or a writable temporary directory: every input is a
//! committed `.class` file or a builder over one, every archive is built in memory, and nothing is
//! written anywhere. The `--nocapture` run prints each case's counted dimensions, the two arms'
//! ledgers, and — separately, under their own label — the wall-clock readings those counts are not.

mod bulk_support;

use bulk_support::{
    DEFLATE, Recorder, STORE, container_roots, environment, open, request, tree_scope, zip,
};
use jarde::*;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------------------------
// The fixture bytes: the repository's committed samples, included where they are
// ---------------------------------------------------------------------------------------------

/// `Guarded` (P3 handlers): 24 declared members, the widest single class the committed corpus holds.
const GUARDED: &[u8] = include_bytes!("fixtures/p3-handlers/v8/Guarded.class");

/// `BooleanContexts` (P3 boolean contexts): 19 declared members, the second widest.
const BOOLEAN_CONTEXTS: &[u8] =
    include_bytes!("fixtures/p3-boolean-contexts/v8/BooleanContexts.class");

/// `ModLike` (P3 nested arithmetic): one member whose body is a five-fold nested arithmetic tree.
const MOD_LIKE: &[u8] = include_bytes!("fixtures/p3-nested-arithmetic/v8/ModLike.class");

/// `NestedEval` (P3 nested evaluation): two members evaluated after a write, whose runs state a
/// refusal — quoted inside an artifact that is still delivered as produced.
const NESTED_EVAL: &[u8] = include_bytes!("fixtures/p3-nested-eval/v8/NestedEval.class");

// ---------------------------------------------------------------------------------------------
// The two hand-assembled class builders
// ---------------------------------------------------------------------------------------------

/// One big-endian `u16`, as the class-file format spells every count and index it writes.
fn u16b(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

/// One big-endian `u32`, as the class-file format spells every length it writes.
fn u32b(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

/// One `CONSTANT_Utf8_info` entry: the tag, the length, and the bytes.
fn utf8(pool: &mut Vec<u8>, text: &[u8]) {
    pool.push(1);
    u16b(
        pool,
        u16::try_from(text.len()).expect("the fixture's string fits u16"),
    );
    pool.extend_from_slice(text);
}

/// The depth of the generated member that is meant to be rendered, in `+` levels.
///
/// The presentation layer refuses a value nested deeper than its own ceiling
/// (`crates/jarde-java/src/build.rs`'s `MAX_VALUE_DEPTH`, 24), and the corpus holds a member inside
/// that ceiling as well as one past it: 16 is deep enough that the tree, its text and the analysis
/// over it are not constants, and far enough from the ceiling that this case does not depend on the
/// boundary's exact arithmetic.
const RENDERED_LEVELS: usize = 16;

/// The depth of the generated member that is past that ceiling, in `+` levels.
///
/// 64 is small on purpose: the corpus exists so the *counts* of a deep expression can be pinned, not
/// so the stack gate can be re-run (that gate is 6.4's 2048-append process check, and its own
/// generator lives in `tests/p3_concat_conversion.rs`). A count pin has to stay cheap enough to run
/// on every `cargo test`.
const DEEPER_LEVELS: usize = 64;

/// One hand-assembled class whose members are `rendered`- and `deeper`-deep left-nested `int`
/// additions.
///
/// The trees are built by bytecode rather than by `javac` because the depth is the parameter of the
/// shape, and a straight-line body needs no branch, so version 52 needs no `StackMapTable`. Each
/// member is `arg0 + 1 + 2 + … + levels`, one `bipush` and one `iadd` per level, so the presented
/// text's own length is a function of the depth, and the frame and SSA work over a deep expression
/// tree is not a constant.
fn deep_expression_class(rendered: usize, deeper: usize) -> Vec<u8> {
    /// The straight-line body of one member: `levels` additions over the argument.
    fn body(levels: usize) -> Vec<u8> {
        let mut code: Vec<u8> = Vec::with_capacity(2 + 3 * levels);
        code.push(0x1a); // 0: iload_0 — the argument every level adds to
        for level in 0..levels {
            code.push(0x10); // bipush
            code.push(u8::try_from(1 + level % 126).expect("the level's constant is one byte"));
            code.push(0x60); // iadd
        }
        code.push(0xac); // ireturn
        code
    }
    fn code_attribute(code: &[u8]) -> Vec<u8> {
        let mut attribute: Vec<u8> = Vec::new();
        u16b(&mut attribute, 2); // max_stack
        u16b(&mut attribute, 1); // max_locals: the single `int` parameter
        u32b(
            &mut attribute,
            u32::try_from(code.len()).expect("the fixture body fits u32"),
        );
        attribute.extend_from_slice(code);
        u16b(&mut attribute, 0); // exception table
        u16b(&mut attribute, 0); // code attributes
        attribute
    }

    // The constant pool: the class, its superclass, the two member names, the shared descriptor and
    // the `Code` attribute name.
    let mut pool: Vec<u8> = Vec::new();
    utf8(&mut pool, b"p/DeepExpr"); // 1
    pool.push(7); // 2: Class 1
    u16b(&mut pool, 1);
    utf8(&mut pool, b"java/lang/Object"); // 3
    pool.push(7); // 4: Class 3
    u16b(&mut pool, 3);
    utf8(&mut pool, b"rendered"); // 5
    utf8(&mut pool, b"deeper"); // 6
    utf8(&mut pool, b"(I)I"); // 7
    utf8(&mut pool, b"Code"); // 8

    let mut members: Vec<u8> = Vec::new();
    u16b(&mut members, 2); // methods_count
    for (name, levels) in [(5_u16, rendered), (6, deeper)] {
        let attribute = code_attribute(&body(levels));
        u16b(&mut members, 0x0009); // public static
        u16b(&mut members, name); // the member's own name
        u16b(&mut members, 7); // descriptor → "(I)I"
        u16b(&mut members, 1); // attributes
        u16b(&mut members, 8); // "Code"
        u32b(
            &mut members,
            u32::try_from(attribute.len()).expect("the fixture attribute fits u32"),
        );
        members.extend_from_slice(&attribute);
    }

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor
    u16b(&mut output, 52); // major: Java 8
    u16b(&mut output, 9); // constant_pool_count: the eight entries above
    output.extend_from_slice(&pool);
    u16b(&mut output, 0x0021); // public super
    u16b(&mut output, 2); // this_class
    u16b(&mut output, 4); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    output.extend_from_slice(&members);
    u16b(&mut output, 0); // class attributes
    output
}
/// The member count of the generated wide class: the corpus's large `M`.
///
/// The committed corpus's widest class declares 24 members, which is a committed sample's number
/// rather than a shape's. This one is generated so the member count is the *parameter* of the case
/// and the per-class and per-member costs can be told apart at a fixed, deliberately large M. It is
/// not a workload: 64 members of two instructions each, and the 6.x measurements stay the job of the
/// real artifacts.
const WIDE_MEMBERS: usize = 64;

/// One hand-assembled class declaring `members` `public static int memberN(int)` members, each
/// returning `arg0 + N` so no two members share a text.
///
/// The pool holds the class, its superclass, the shared descriptor `(I)I`, `Code` and one name per
/// member; every member is `iload_0; bipush N; iadd; ireturn`, and a straight-line body needs no
/// `StackMapTable` at version 52.
fn wide_class(members: usize) -> Vec<u8> {
    let mut pool: Vec<u8> = Vec::new();
    utf8(&mut pool, b"p/WideClass"); // 1
    pool.push(7); // 2: Class 1
    u16b(&mut pool, 1);
    utf8(&mut pool, b"java/lang/Object"); // 3
    pool.push(7); // 4: Class 3
    u16b(&mut pool, 3);
    utf8(&mut pool, b"(I)I"); // 5
    utf8(&mut pool, b"Code"); // 6
    for member in 0..members {
        utf8(&mut pool, format!("member{member}").as_bytes()); // 7, 8, …
    }

    let mut methods: Vec<u8> = Vec::new();
    u16b(
        &mut methods,
        u16::try_from(members).expect("the fixture's member count fits u16"),
    );
    for member in 0..members {
        let code: Vec<u8> = vec![
            0x1a, // iload_0
            0x10, // bipush
            u8::try_from(1 + member % 126).expect("the member's constant is one byte"),
            0x60, // iadd
            0xac, // ireturn
        ];

        let mut attribute: Vec<u8> = Vec::new();
        u16b(&mut attribute, 2); // max_stack
        u16b(&mut attribute, 1); // max_locals: the single `int` parameter
        u32b(
            &mut attribute,
            u32::try_from(code.len()).expect("the fixture body fits u32"),
        );
        attribute.extend_from_slice(&code);
        u16b(&mut attribute, 0); // exception table
        u16b(&mut attribute, 0); // code attributes

        u16b(&mut methods, 0x0009); // public static
        u16b(
            &mut methods,
            u16::try_from(7 + member).expect("the member's pool index fits u16"),
        );
        u16b(&mut methods, 5); // descriptor → "(I)I"
        u16b(&mut methods, 1); // attributes
        u16b(&mut methods, 6); // "Code"
        u32b(
            &mut methods,
            u32::try_from(attribute.len()).expect("the fixture attribute fits u32"),
        );
        methods.extend_from_slice(&attribute);
    }

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor
    u16b(&mut output, 52); // major: Java 8
    u16b(
        &mut output,
        u16::try_from(7 + members).expect("the fixture's pool fits u16"),
    );
    output.extend_from_slice(&pool);
    u16b(&mut output, 0x0021); // public super
    u16b(&mut output, 2); // this_class
    u16b(&mut output, 4); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    output.extend_from_slice(&methods);
    u16b(&mut output, 0); // class attributes
    output
}

// ---------------------------------------------------------------------------------------------
// The corpus: one archive per named shape, and what each case is asserted to be
// ---------------------------------------------------------------------------------------------

/// One case: how its archive is built, what its environment declares, and what the run is asserted
/// to bill at that shape.
///
/// The archive is rebuilt by every test through [`Case::build`] rather than carried as bytes, so
/// regenerability is a property of the corpus rather than of one test's order.
struct Case {
    /// The case's name, which is also the shape's name in every failure message.
    name: &'static str,
    /// The shape this case is the carrier of, in one sentence.
    shape: &'static str,
    /// The builder: a total function of committed bytes, called again by every test.
    build: fn() -> Vec<u8>,
    /// The load roots the environment declares, in the caller's own order.
    prefixes: &'static [&'static [u8]],
    /// Every class entry the scope delivers, in the traversal's own order.
    classes: &'static [&'static str],
    /// The declared member count the case's prepared classes add up to.
    declared_methods: u64,
    /// The counted dimensions of one run at this shape (one worker, [`bulk_support::limits`]).
    billing: Billing,
}

/// One flat archive at the root: four committed classes, alternating compression methods.
fn flat_mixed() -> Vec<u8> {
    zip(&[
        (b"Scope.class", bulk_support::SCOPE, STORE),
        (b"Shape.class", bulk_support::SHAPE, DEFLATE),
        (b"LambdaSample.class", bulk_support::LAMBDA, STORE),
        (b"Holder.class", bulk_support::HOLDER, DEFLATE),
    ])
}

/// Classes at the root beside two nested containers, one STORED and one DEFLATED, whose entries use
/// the opposite method from their container.
fn nested_mixed() -> Vec<u8> {
    let stored = zip(&[
        (b"d/Holder.class", bulk_support::HOLDER, STORE),
        (b"d/Shape.class", bulk_support::SHAPE, DEFLATE),
    ]);
    let deflated = zip(&[(b"e/LambdaSample.class", bulk_support::LAMBDA, STORE)]);
    zip(&[
        (b"Scope.class", bulk_support::SCOPE, DEFLATE),
        (b"lib/stored.jar", &stored, STORE),
        (b"lib/deflated.jar", &deflated, DEFLATE),
        (b"ModLike.class", MOD_LIKE, STORE),
    ])
}

/// The same nested archive bytes at two entries — one STORED, one DEFLATED — so `d/Holder.class` is
/// reachable under two physical origins, with one class beside them at the root.
fn two_origins_one_identity() -> Vec<u8> {
    let inner = zip(&[(b"d/Holder.class", bulk_support::HOLDER, DEFLATE)]);
    zip(&[
        (b"Scope.class", bulk_support::SCOPE, STORE),
        (b"lib/one.jar", &inner, STORE),
        (b"lib/two.jar", &inner, DEFLATE),
    ])
}

/// The widest committed classes beside the generated wide one, so the corpus has a large member
/// count whose size is a parameter rather than a committed sample's accident.
fn many_method_class() -> Vec<u8> {
    zip(&[
        (b"Guarded.class", GUARDED, STORE),
        (b"BooleanContexts.class", BOOLEAN_CONTEXTS, DEFLATE),
        (b"p/WideClass.class", &wide_class(WIDE_MEMBERS), STORE),
    ])
}

/// Damaged entries of the kinds the task names, beside classes that still recover: a class entry
/// whose bytes are a truncated class file, a nested container whose tail is cut, and a `.jar` entry
/// that is not a container at all.
fn damaged_tail() -> Vec<u8> {
    let intact = zip(&[(b"d/Holder.class", bulk_support::HOLDER, STORE)]);
    let truncated = &intact[..intact.len() - 64];
    zip(&[
        (b"Scope.class", bulk_support::SCOPE, DEFLATE),
        (b"Broken.class", &bulk_support::HOLDER[..64], STORE),
        (b"lib/truncated.jar", truncated, STORE),
        (
            b"lib/garbage.jar",
            b"PK\x03\x04not a container at all",
            DEFLATE,
        ),
        (b"Holder.class", bulk_support::HOLDER, STORE),
    ])
}

/// The generated deep expression beside the committed nested-evaluation sample.
fn deep_expression() -> Vec<u8> {
    zip(&[
        (
            b"p/DeepExpr.class",
            &deep_expression_class(RENDERED_LEVELS, DEEPER_LEVELS),
            STORE,
        ),
        (b"NestedEval.class", NESTED_EVAL, DEFLATE),
    ])
}

/// The corpus, in the order the tests run it.
fn corpus() -> Vec<Case> {
    vec![
        Case {
            name: "flat-mixed",
            shape: "one flat archive, four class entries, STORED and DEFLATED alternating",
            build: flat_mixed,
            prefixes: &[b""],
            classes: &[
                "Scope.class",
                "Shape.class",
                "LambdaSample.class",
                "Holder.class",
            ],
            declared_methods: 18,
            billing: Billing::FLAT_MIXED,
        },
        Case {
            name: "nested-mixed",
            shape: "root classes beside a STORED and a DEFLATED nested container",
            build: nested_mixed,
            prefixes: &[b"", b"d/", b"e/"],
            classes: &[
                "Scope.class",
                "d/Holder.class",
                "d/Shape.class",
                "e/LambdaSample.class",
                "ModLike.class",
            ],
            declared_methods: 27,
            billing: Billing::NESTED_MIXED,
        },
        Case {
            name: "two-origins-one-identity",
            shape: "the same nested bytes at two origins, one STORED and one DEFLATED",
            build: two_origins_one_identity,
            prefixes: &[b"", b"d/", b"d/"],
            classes: &["Scope.class", "d/Holder.class", "d/Holder.class"],
            declared_methods: 16,
            billing: Billing::TWO_ORIGINS,
        },
        Case {
            name: "many-method-class",
            shape: "64 generated members in one class beside the widest committed ones (24 and 19): the large M, and the slow consumer's sample",
            build: many_method_class,
            prefixes: &[b""],
            classes: &[
                "Guarded.class",
                "BooleanContexts.class",
                "p/WideClass.class",
            ],
            declared_methods: 107,
            billing: Billing::MANY_METHOD_CLASS,
        },
        Case {
            name: "damaged-tail",
            shape: "a truncated class file, a container whose tail is cut and a non-container .jar beside two readable classes",
            build: damaged_tail,
            prefixes: &[b""],
            classes: &["Scope.class", "Broken.class", "Holder.class"],
            declared_methods: 12,
            billing: Billing::DAMAGED_TAIL,
        },
        Case {
            name: "deep-expression",
            shape: "a generated 16-level addition tree and a 64-level one beside the committed nested-evaluation sample",
            build: deep_expression,
            prefixes: &[b""],
            classes: &["p/DeepExpr.class", "NestedEval.class"],
            declared_methods: 7,
            billing: Billing::DEEP_EXPRESSION,
        },
    ]
}

// ---------------------------------------------------------------------------------------------
// The counts
// ---------------------------------------------------------------------------------------------

/// Nine counted dimensions of one run, at a fixed shape.
///
/// Every field is read from [`UsageSnapshot`], the operation's own account of what it asked the
/// budget to count. The type deliberately has no `elapsed_millis` field: a reading of one machine's
/// wall clock is not a count, this file asserts none, and keeping it out of the type is how the table
/// stays a ledger rather than a stopwatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Billing {
    archive_entries: u64,
    entry_bytes: u64,
    class_bytes: u64,
    class_headers: u64,
    method_bodies: u64,
    ir_items: u64,
    analysis_steps: u64,
    result_items: u64,
    output_bytes: u64,
}

impl Billing {
    /// The dimensions of one usage snapshot, in the order the fields are declared.
    fn of(usage: &UsageSnapshot) -> Self {
        Self {
            archive_entries: usage.archive_entries,
            entry_bytes: usage.entry_bytes,
            class_bytes: usage.class_bytes,
            class_headers: usage.class_headers,
            method_bodies: usage.method_bodies,
            ir_items: usage.ir_items,
            analysis_steps: usage.analysis_steps,
            result_items: usage.result_items,
            output_bytes: usage.output_bytes,
        }
    }

    /// The nine dimensions as one line, for a run's own record.
    fn line(&self) -> String {
        format!(
            "archive_entries={} entry_bytes={} class_bytes={} class_headers={} method_bodies={} \
             ir_items={} analysis_steps={} result_items={} output_bytes={}",
            self.archive_entries,
            self.entry_bytes,
            self.class_bytes,
            self.class_headers,
            self.method_bodies,
            self.ir_items,
            self.analysis_steps,
            self.result_items,
            self.output_bytes,
        )
    }

    /// The fields as a `Billing { … }` literal, so an intended shape change is regenerated rather
    /// than guessed.
    fn literal(&self) -> String {
        format!(
            "Billing {{ archive_entries: {}, entry_bytes: {}, class_bytes: {}, class_headers: {}, \
             method_bodies: {}, ir_items: {}, analysis_steps: {}, result_items: {}, output_bytes: {} }}",
            self.archive_entries,
            self.entry_bytes,
            self.class_bytes,
            self.class_headers,
            self.method_bodies,
            self.ir_items,
            self.analysis_steps,
            self.result_items,
            self.output_bytes,
        )
    }
}

/// The pinned table, one entry per case, measured on this repository's committed bytes at one worker
/// under [`bulk_support::limits`].
///
/// A number here moves only when the *shape* of the work moves — a fixture's bytes, the containers a
/// case declares, or the operation's own read pattern — and the case's name says which shape. It is
/// not a target, not a budget and not a time: see this file's header for what a count may and may not
/// be read as, and `openspec/benchmark-protocol.md` for the rule every comparison row follows.
///
/// **Re-pinned by D1 and again by D3 of `add-demand-driven-core-results`.** The operation's own
/// constructor states the full evidence selection (`BulkRecoveryRequest::for_scope`), and the
/// evidence phase charges one `IrItems` per optional owning record it materializes. D1 let the phase
/// account for the records the run already built where the rules decided; D3 moved every category
/// into the phase — the rule records are now written from the plans after the artifact is committed,
/// and the segment table is replayed through the same formatter — so `ir_items` carries those
/// charges too, the map included, one per span. That is the *shape* of the work moving: the artifact,
/// the reads, the decodes and the passes are unchanged, and the rows below move on `ir_items` alone
/// (by the number of records and spans each shape really has). The per-case rows and the two arms
/// were regenerated with `record_the_billing_table` and the arms' own reader, not hand-edited.
///
/// P3 2c.26 re-measured five of the six rows: `Holder`'s and `Guarded`'s static initializers build
/// the instance they assign to a static field (`new; dup; invokespecial; putstatic`), and `new@1` now
/// presents that construction where it used to quote it — the two statements that named the
/// allocation and its copy give way to the assignment, so those rows are **2 `IrItems` and 436
/// `output_bytes` lower** and every other dimension stands. `deep-expression` holds no such member
/// and did not move. The deltas were measured by running the reader with and without the
/// field-access reader `@new` counts (the only difference between the two runs), not by reading the
/// old pins back.
///
/// **Re-measured for the 2026-09-26 syntax-recovery expansion.** Every row moved, in the same
/// direction and on the same three dimensions: `ir_items` and `analysis_steps` rise because every
/// body's recovery now runs the expansion's differential-evidence work (the arms' ledger, over the
/// same 181 bodies, rose by exactly +4,905 `IrItems` and +5,073 `AnalysisSteps` — about 27 and 28
/// per body), and `output_bytes` falls where an artifact's quote or explanation gave way to the
/// statement the expansion now writes — `Guarded.boom`'s `throw new` above them all, which moved
/// from `ExplanationOnly` to `Produced` and took `many-method-class`'s explanation-only count from
/// six to five. No read, decode or delivery dimension moved. The rows below were regenerated with
/// `record_the_billing_table`, not hand-edited; the six cases' deltas sum to the arms' deltas.
/// The live-path declaration correction then re-measured the table again. It excludes uses in
/// truly dead uncovered quotes, keeps handlers reachable from actual throw sites, and bills the
/// bounded entry/slot scan. Relative to the preceding expansion pins, `AnalysisSteps` moves by
/// +18/+68/+5/+332/+5/+22 across the six rows (+450 in either arm). The `Guarded` row also drops
/// 68 `IrItems` and 428 `output_bytes` as its declaration/quote text changes; all physical read
/// dimensions remain fixed. These are the recorder's measured counts, not a formula for the pass.
impl Billing {
    /// `flat-mixed`: four classes at one root and nothing nested.
    ///
    /// `Holder`'s `<clinit>` is the 2c.26 shape: its construction is written where the `putstatic`
    /// runs, so this row is 2 `IrItems` and 436 `output_bytes` under the quoted one it had.
    /// The 2026-09-26 expansion moved `ir_items` +382, `analysis_steps` +529 and `output_bytes`
    /// −19: the per-body evidence work, and the quote of `Holder`'s construction the statement
    /// replaced.
    const FLAT_MIXED: Self = Self {
        archive_entries: 8,
        entry_bytes: 1813,
        class_bytes: 1813,
        class_headers: 0,
        method_bodies: 17,
        ir_items: 2371,
        analysis_steps: 1330,
        result_items: 37,
        output_bytes: 3657,
    };
    /// `nested-mixed`: three containers, two of them nested, five classes.
    ///
    /// `entry_bytes` here counts the *logical* bytes of every entry read, so it stands far above
    /// `class_bytes`: the library path of this file attaches no store, and each class's preparation
    /// re-reads (and so re-materializes) the containers that hold the classes it needs. The arms test
    /// below is the same shape with one store attached, and its two ledgers are where that difference
    /// is recorded — as counts of addressed reads, which is all either row states.
    ///
    /// 2c.26, over the `Holder` this case shares with `flat-mixed`: 2 `IrItems` and 436
    /// `output_bytes` under the quoted row. The 2026-09-26 expansion moved `ir_items` +668,
    /// `analysis_steps` +750 and `output_bytes` −19 (the same per-body evidence work, plus the
    /// `Holder` and `ModLike` quotes that became statements).
    const NESTED_MIXED: Self = Self {
        archive_entries: 25,
        entry_bytes: 8061,
        class_bytes: 2381,
        class_headers: 0,
        method_bodies: 26,
        ir_items: 4111,
        analysis_steps: 2060,
        result_items: 64,
        output_bytes: 5628,
    };
    /// `two-origins-one-identity`: one class file behind two physical origins, and the only case with
    /// a nonzero `class_headers`. What that dimension counts here is the binding work a duplicated
    /// name causes — reads taken behind the members' runs rather than by the one preparation every
    /// class gets — and the case's shape test states which origin bound, which one was shadowed and
    /// why the run is `partial` for it. The row pins the cost of that shape; it does not claim a
    /// formula for it.
    ///
    /// 2c.26, over the `Holder` this case names twice: 2 `IrItems` and 436 `output_bytes` under the
    /// quoted row. The 2026-09-26 expansion moved `ir_items` +312, `analysis_steps` +446 and
    /// `output_bytes` −19 (the same `Scope`/`Holder` shape as `flat-mixed`).
    const TWO_ORIGINS: Self = Self {
        archive_entries: 43,
        entry_bytes: 7522,
        class_bytes: 2686,
        class_headers: 4,
        method_bodies: 12,
        ir_items: 1974,
        analysis_steps: 1093,
        result_items: 43,
        output_bytes: 2638,
    };
    /// `many-method-class`: 107 members behind three read classes, one of them generated wide.
    const MANY_METHOD_CLASS: Self = Self {
        archive_entries: 6,
        entry_bytes: 7683,
        class_bytes: 7683,
        class_headers: 0,
        method_bodies: 107,
        // 25688 and not 25710: this case is the one that holds `Guarded` and `BooleanContexts`, and
        // the two `Z` field writes of their static initializers (`Guarded.FLAG`,
        // `BooleanContexts.staticFlag`) are spelled `true` where they were the `int` `1` — three
        // bytes longer each, and `= 1;` is text `javac` refuses (`int cannot be converted to
        // boolean`). No member's classification moved; only those two spellings did.
        //
        // The `try`/`catch` of `Guarded.syncThrowsCatching` and `Guarded.secondInitFailsCatching`
        // moved the other three dimensions: a plain `catch` is presented as the `try` the exception
        // table states where it used to be quoted whole (28 bytes fewer over the two — the clauses
        // and the handler bodies are shorter than the two quotes and the uncovered-block line they
        // replace), and presenting them costs 22 `IrItems` (the statements and anchored spans the
        // text now writes) and 10 `AnalysisSteps` (the two shapes' own examination).
        // Asking whether a store in front of a protected range is a resource then added 4
        // `AnalysisSteps` (7065 to 7069) and changed no text.
        // P3 2.7 wrote those two bodies' own calls: the block that carries `syncThrows()` /
        // `secondInitFails()` is protected by a named row whose handler is the clause the text
        // already writes around it, so the call is a statement of the `try` and no longer a quote.
        // The two calls cost 233 `output_bytes` less (the `// @bytecode` anchor line and its
        // sentence give way to the call) and bill 2 more `IrItems` and 2 more `AnalysisSteps`
        // (7069 to 7071).
        //
        // 2c.26 then moved this row the same way it moved the `Holder` rows: `Guarded`'s
        // `<clinit>` builds the instance it assigns to `LOCK`, and the construction is written
        // instead of quoted — 2 `IrItems` fewer and 436 `output_bytes` fewer.
        //
        // The 2026-09-26 expansion moved `ir_items` +2739, `analysis_steps` +2623 and
        // `output_bytes` −762: the per-body evidence work over this case's 107 bodies, the quotes
        // that became statements, and — the one classification that moved — `boom`'s `throw new`,
        // which the expansion presents as the construction statement it is, so the case's
        // explanation-only count fell from six to five.
        ir_items: 20408,
        analysis_steps: 10026,
        result_items: 122,
        output_bytes: 23829,
    };
    /// `damaged-tail`: the readable classes only; the damaged entries cost their own attempts.
    ///
    /// 2c.26, over its own `Holder`: 2 `IrItems` and 436 `output_bytes` under the quoted row.
    /// The 2026-09-26 expansion moved `ir_items` +312, `analysis_steps` +446 and `output_bytes`
    /// −19, exactly the `two-origins` moves: the readable shape is the same `Scope`/`Holder` pair.
    const DAMAGED_TAIL: Self = Self {
        archive_entries: 17,
        entry_bytes: 1881,
        class_bytes: 1019,
        class_headers: 0,
        method_bodies: 12,
        ir_items: 1974,
        analysis_steps: 1093,
        result_items: 35,
        output_bytes: 2638,
    };
    /// `deep-expression`: the two generated chains and the nested-evaluation sample.
    ///
    /// The 2026-09-26 expansion moved `ir_items` +492, `analysis_steps` +279 and `output_bytes`
    /// −131: the per-body evidence work, and the committed sample's artifacts shortened where its
    /// quoted refusals now sit beside the statements the expansion writes.
    const DEEP_EXPRESSION: Self = Self {
        archive_entries: 4,
        entry_bytes: 753,
        class_bytes: 753,
        class_headers: 0,
        method_bodies: 7,
        ir_items: 3015,
        analysis_steps: 1172,
        result_items: 18,
        output_bytes: 1723,
    };

    /// Arm A — the per-member path without a store — over the whole corpus: 187 requests, one fresh
    /// budget each. Pinned like the case table, and readable under the same rule: a count at this
    /// shape, never a time.
    ///
    /// `ir_items`/`analysis_steps`/`output_bytes` are re-measured by the `try`/`catch` slice: the
    /// corpus holds `Guarded`, whose `syncThrowsCatching` and `secondInitFailsCatching` are now
    /// presented as the `try` their exception tables state instead of being quoted whole — the same
    /// move, and the same 22/10/28, the `many-method-class` row above states. P3 2.7's writing of
    /// those two members' own calls moved the same three by the same 2/2/233 the case row records.
    ///
    /// P3 2c.26 moved this row too, by the same shape it moved the case rows that hold `Holder` and
    /// `Guarded`: the same `new; dup; invokespecial; putstatic` construction is written where its
    /// field write runs instead of being quoted, which is 10 `IrItems` (the quotes of the two
    /// allocations and their copies over the five affected cases) and 2180 `output_bytes` over the
    /// arm. `analysis_steps` stands.
    ///
    /// The 2026-09-26 syntax-recovery expansion moved `ir_items` +4,905, `analysis_steps` +5,073
    /// and `output_bytes` −969 — exactly the sum of the six case rows' moves. The work is the
    /// expansion's per-body differential evidence plus the quotes and explanations that became
    /// statements (`Guarded.boom`'s `throw new` among them); the read, decode and delivery
    /// dimensions stand.
    const DIRECT_ARM: Self = Self {
        archive_entries: 1915,
        entry_bytes: 365912,
        class_bytes: 327895,
        class_headers: 191,
        method_bodies: 181,
        ir_items: 33853,
        analysis_steps: 16774,
        result_items: 1378,
        output_bytes: 40113,
    };

    /// Arm B — the same requests, each carrying the one store that started empty. Pinned for the same
    /// reason; the difference between this row and [`Billing::DIRECT_ARM`] is what retention moved in
    /// the *reads*, and the dimensions the two rows agree on are the ones the work itself charged.
    ///
    /// **Re-pinned by `reuse-selected-class-read`**: this row used to bill `archive_entries: 411`,
    /// `entry_bytes: 330368` (and `read_bytes: 330368`). The store's definition-read layer now answers
    /// a request for a definition an earlier request of the same snapshot already read — the 187
    /// requests ask about 19 definitions, so every request after the first for the same definition
    /// performs no entry read at all (`19 definition reads retained`, `definition_read_hits` 168 of
    /// them). The read dimensions are exactly where this row moves; `class_bytes`, `class_headers`,
    /// `method_bodies`, the IR counts, `result_items` and `output_bytes` are unmoved, which is what
    /// this file's claim means: retention moves reads, never work. The three dimensions the `try`/
    /// `catch` slice moved ([`Billing::DIRECT_ARM`]) moved here by the same amounts — the work is the
    /// same work — and so did the `2/2/233` P3 2.7 moved on that row. The 2026-09-26 expansion
    /// moved this row's three work dimensions by the same +4,905/+5,073/−969 the direct arm
    /// records, for the same reason; retention still touches only the read dimensions.
    const SHARED_ARM: Self = Self {
        archive_entries: 57,
        entry_bytes: 18680,
        class_bytes: 10817,
        class_headers: 191,
        method_bodies: 181,
        ir_items: 33853,
        analysis_steps: 16774,
        result_items: 26,
        output_bytes: 40113,
    };
}
// ---------------------------------------------------------------------------------------------
// Running one case
// ---------------------------------------------------------------------------------------------

/// One run: the report the operation published, everything the consumer recorded, and the wall
/// clock of the call itself — printed beside the counts and asserted nowhere.
struct Run {
    report: BulkRecoveryReport,
    sink: Recorder,
    elapsed_millis: u128,
}

/// One run of one case: the case's archive opened, its roots declared, its scope recovered, with one
/// worker unless the caller asks otherwise.
fn run_case(case: &Case, workers: usize, delay: Duration) -> Run {
    let (snapshot, _opened) = open((case.build)());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, case.prefixes);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment, workers);
    let mut sink = Recorder::new();
    sink.delay = delay;
    let started = Instant::now();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the case's archive is readable and its scope recoverable");
    let elapsed_millis = started.elapsed().as_millis();
    Run {
        report,
        sink,
        elapsed_millis,
    }
}

/// One case's physical read before any operation: its containers, the high-water nesting the tree
/// walk reached, and whatever it located while walking.
fn enumerate(case: &Case) -> (ArtifactTreeReport, UsageSnapshot) {
    let (snapshot, _opened) = open((case.build)());
    let mut budget = Budget::new(bulk_support::limits());
    let report = snapshot
        .enumerate_artifact_tree(&mut budget)
        .expect("the case's archive enumerates");
    let usage = budget.usage();
    (report, usage)
}

/// The entry of one container of a case, by raw name.
fn entry<'report>(
    report: &'report ArtifactTreeReport,
    container: usize,
    raw_name: &[u8],
) -> &'report PhysicalEntry {
    report.containers[container]
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == raw_name)
        .unwrap_or_else(|| {
            panic!(
                "container {container} holds no entry {:?}",
                String::from_utf8_lossy(raw_name)
            )
        })
}

/// The class entries one run prepared, in the order the stream delivered them.
fn delivered_classes(run: &Run) -> Vec<String> {
    run.sink
        .prepared()
        .iter()
        .map(|prepared| {
            String::from_utf8_lossy(&prepared.location.entry().unwrap().raw_name.0).into_owned()
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// The tests
// ---------------------------------------------------------------------------------------------

/// The corpus is a regenerable thing: the same builders, called again in the same process, produce
/// the same bytes, every case's bytes are a real archive to the reader, and the one hand-assembled
/// input of the corpus is a class file the reader accepts rather than bytes that merely look like one.
#[test]
fn the_corpus_is_rebuilt_from_the_committed_bytes_every_time() {
    for case in corpus() {
        let first = (case.build)();
        let second = (case.build)();
        assert!(
            !first.is_empty(),
            "{}: the builder produced no bytes",
            case.name
        );
        assert_eq!(
            first, second,
            "{}: the builder is not a function of the committed bytes",
            case.name
        );
        let (snapshot, _opened) = open(first);
        assert_eq!(
            snapshot.kind(),
            ArtifactKind::Zip,
            "{}: the case's bytes are not an archive",
            case.name
        );
    }
    // The generated class is read as one class file by the reader's own header entry: a version, a
    // name and its two members, not bytes that merely begin with `cafebabe`.
    let generated = deep_expression_class(RENDERED_LEVELS, DEEPER_LEVELS);
    let mut budget = Budget::new(bulk_support::limits());
    let header = inspect_header(&generated, &mut budget, InspectionMode::Forensic)
        .expect("the generated class file parses");
    assert_eq!(header.header.major_version, 52);
    assert_eq!(header.header.this_class.raw().0, b"p/DeepExpr");
    let names: Vec<Vec<u8>> = header
        .header
        .methods
        .iter()
        .map(|member| member.name.raw().0.clone())
        .collect();
    assert_eq!(names, vec![b"rendered".to_vec(), b"deeper".to_vec()]);
    for member in &header.header.methods {
        assert_eq!(member.descriptor.raw().0, b"(I)I");
    }
}

/// Every case really holds the shape it is named for, read back from the artifact rather than taken
/// from the builder: compression methods, containers and nesting, duplicate origins, the damaged
/// entries' own dispositions, and the deep member's presented text.
#[test]
fn every_case_holds_the_shape_it_is_named_for() {
    let cases = corpus();

    // `flat-mixed`: one container, no nesting, and both compression methods at its root.
    let case = &cases[0];
    let (tree, usage) = enumerate(case);
    assert_eq!(tree.containers.len(), 1);
    assert_eq!(usage.nested_depth, 0, "a flat archive nests nothing");
    for (index, method) in [(0, STORE), (1, DEFLATE), (2, STORE), (3, DEFLATE)] {
        assert_eq!(
            tree.containers[0].entries[index].compression_method, method,
            "{} entry {index} is not the method the case declares",
            case.name
        );
    }
    let run = run_case(case, 1, Duration::ZERO);
    assert_eq!(delivered_classes(&run), case.classes);
    assert_eq!(run.report.summary.status(), "complete");

    // `nested-mixed`: three containers, the nested two carrying the classes the root does not, with
    // their entries compressed the other way round from their own container.
    let case = &cases[1];
    let (tree, usage) = enumerate(case);
    assert_eq!(tree.containers.len(), 3);
    assert_eq!(usage.nested_depth, 1);
    assert_eq!(tree.containers[1].depth, 1);
    assert_eq!(tree.containers[2].depth, 1);
    assert_eq!(tree.containers[1].entries.len(), 2);
    assert_eq!(tree.containers[2].entries.len(), 1);
    assert_eq!(entry(&tree, 0, b"lib/stored.jar").compression_method, STORE);
    assert_eq!(entry(&tree, 1, b"d/Holder.class").compression_method, STORE);
    assert_eq!(
        entry(&tree, 1, b"d/Shape.class").compression_method,
        DEFLATE
    );
    assert_eq!(
        entry(&tree, 0, b"lib/deflated.jar").compression_method,
        DEFLATE
    );
    assert_eq!(
        entry(&tree, 2, b"e/LambdaSample.class").compression_method,
        STORE
    );
    let run = run_case(case, 1, Duration::ZERO);
    assert_eq!(delivered_classes(&run), case.classes);
    assert_eq!(
        run.report.summary.status(),
        "complete",
        "a nested scope whose containers are all readable completes: {:#?}",
        run.report.summary
    );

    // `two-origins-one-identity`: one class file behind two origins. The two entries really hold the
    // same bytes — their CRC and uncompressed size agree, though their compression methods do not —
    // and the run really prepares the class twice, from the two containers.
    //
    // The two origins are *not* interchangeable to the environment, and the corpus pins that rather
    // than hiding it: the declared order's first position for the name `Holder` is the one the
    // loader binds, so the class from `lib/one.jar` is recovered and every member of the class from
    // `lib/two.jar` is `NotProduced` with `resolution_definition_unbound` — a physical scope reads
    // its candidates by identity, and a name binds to one position. The class is still walked and
    // prepared; nothing is silently attributed to the other origin, and the operation states itself
    // `partial` for it. This is the duplicate-candidate shape B01 is about, and it is the reason
    // this case exists beside the four single-origin ones.
    let case = &cases[2];
    let (tree, _usage) = enumerate(case);
    let one = entry(&tree, 1, b"d/Holder.class");
    let two = entry(&tree, 2, b"d/Holder.class");
    assert_eq!(one.crc32, two.crc32, "the two origins hold one class file");
    assert_eq!(one.uncompressed_size, two.uncompressed_size);
    assert_ne!(
        one.id.origin, two.id.origin,
        "and they are two physical origins, not one entry seen twice"
    );
    assert_eq!(entry(&tree, 0, b"lib/one.jar").compression_method, STORE);
    assert_eq!(entry(&tree, 0, b"lib/two.jar").compression_method, DEFLATE);
    let run = run_case(case, 1, Duration::ZERO);
    assert_eq!(delivered_classes(&run), case.classes);
    assert_eq!(run.report.summary.classes_prepared, 3);
    let by_class = |ordinal: u64| -> Vec<bulk_support::MethodRecord> {
        run.sink
            .methods()
            .into_iter()
            .filter(|method| method.class_ordinal == ordinal)
            .collect()
    };
    let bound = by_class(1);
    let shadowed = by_class(2);
    assert_eq!(
        bound.len(),
        4,
        "the bound origin's four members are records"
    );
    assert_eq!(shadowed.len(), 4, "and so are the shadowed origin's four");
    assert!(
        bound.iter().all(|method| method.text.is_some()),
        "the origin the loader binds presents its text"
    );
    assert!(
        shadowed
            .iter()
            .all(|method| method.outcome == BulkMethodOutcome::NotProduced && method.text.is_none()),
        "the shadowed origin states its members as not produced, rather than borrowing the other \
         origin's text: {shadowed:?}"
    );
    assert_eq!(run.report.summary.outcomes.produced, 12);
    assert_eq!(run.report.summary.outcomes.not_produced, 4);
    assert_eq!(run.report.summary.status(), "partial");
    let ExecutionReport::Partial { reason, .. } = &run.report.summary.execution else {
        panic!(
            "a shadowed candidate is a partial run, not a complete one: {:?}",
            run.report.summary.execution
        );
    };
    assert_eq!(
        reason,
        &TerminationReason::Error {
            code: "resolution_definition_unbound".to_owned()
        },
        "and it names the fact that decided it"
    );
    let shadowed_end = run
        .sink
        .class_ends()
        .into_iter()
        .find(|end| end.class_ordinal == 2)
        .expect("the shadowed class still ends");
    assert_eq!(
        shadowed_end.completion,
        ClassCompletion::Completed { methods: 4 },
        "its members were all delivered as records"
    );
    assert!(
        matches!(shadowed_end.execution, ExecutionReport::Failed { .. }),
        "and its own run states that it failed: {:?}",
        shadowed_end.execution
    );

    // `many-method-class`: the widest committed classes beside the generated wide one, read whole.
    let case = &cases[3];
    let run = run_case(case, 1, Duration::ZERO);
    assert_eq!(delivered_classes(&run), case.classes);
    let declared: Vec<u64> = run
        .sink
        .prepared()
        .iter()
        .map(|prepared| prepared.declared_methods)
        .collect();
    assert_eq!(
        declared,
        vec![24, 19, WIDE_MEMBERS as u64],
        "the large M is read, not assumed"
    );
    assert_eq!(run.report.summary.methods_declared, 107);
    assert_eq!(run.report.summary.status(), "complete");
    assert_eq!(
        run.report.summary.outcomes.explanation_only, 5,
        "the committed `Guarded` sample is where the case's explanation-shaped members are — its \
         `fin`/`catchFinally` copies, its guarded bodies that branch (`branching`, `withCatch`) and \
         the irreducible `suppressedCatching` — and the generated members are all produced. The \
         sixth member the previous ledger counted here, `boom`'s `throw new`, is presented as the \
         construction statement it is since the explicit-cast/construction presentation landed, so \
         it is `Produced` now: {:?}",
        run.report.summary.outcomes
    );

    // `damaged-tail`: the two readable classes are delivered, the truncated class is refused with a
    // located diagnostic, and the two damaged nested entries are located by the walk too. The walk
    // still reaches the end of the scope, and the classes after the damaged entries are delivered.
    let case = &cases[4];
    let (tree, _usage) = enumerate(case);
    let located: Vec<String> = tree
        .diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.provenance.as_ref())
        .map(|provenance| match &provenance.location {
            Location::Entry { id, .. } => String::from_utf8_lossy(&id.raw_name.0).into_owned(),
            other => format!("{other:?}"),
        })
        .collect();
    for damaged in ["lib/truncated.jar", "lib/garbage.jar"] {
        assert!(
            located.iter().any(|name| name == damaged),
            "{damaged} is located by the tree walk: {located:?}"
        );
    }
    let run = run_case(case, 1, Duration::ZERO);
    assert_eq!(run.report.summary.classes_seen, 3);
    assert_eq!(run.report.summary.classes_prepared, 2);
    assert_eq!(run.report.summary.classes_refused, 1);
    assert_eq!(run.report.summary.methods_declared, 12);
    assert_eq!(
        run.report.summary.methods_delivered, 12,
        "a refused class takes no members of the classes beside it with it"
    );
    assert!(
        !run.report.summary.traversal_complete,
        "a damaged subtree is never a scope-wide denominator: the classes the cut container would \
         have held are unknown, and the summary says so instead of guessing"
    );
    assert_eq!(run.report.summary.status(), "partial");
    let refused = run
        .sink
        .class_ends()
        .into_iter()
        .find(|end| matches!(end.completion, ClassCompletion::Refused { .. }))
        .expect("the truncated class is refused");
    assert_eq!(
        refused
            .location
            .entry()
            .map(|entry| entry.raw_name.0.clone()),
        Some(b"Broken.class".to_vec())
    );

    // `deep-expression`: the generated class really holds one chain the layer renders and one past
    // its rendering ceiling; the committed nested-evaluation sample really carries its refusals.
    // All of them are read through the operation.
    let case = &cases[5];
    let run = run_case(case, 1, Duration::ZERO);
    assert_eq!(delivered_classes(&run), case.classes);
    assert_eq!(run.report.summary.methods_declared, 7);
    let methods = run.sink.methods();
    let member = |name: &[u8]| -> bulk_support::MethodRecord {
        methods
            .iter()
            .find(|method| method.method.name.0 == name.to_vec())
            .unwrap_or_else(|| {
                panic!(
                    "the generated class delivers `{}`",
                    String::from_utf8_lossy(name)
                )
            })
            .clone()
    };
    let rendered = member(b"rendered");
    let text = rendered
        .text
        .as_deref()
        .expect("the chain inside the ceiling is presented as text");
    assert_eq!(
        text.matches('+').count(),
        RENDERED_LEVELS,
        "the presented text holds one `+` per level: {text}"
    );
    let deeper = member(b"deeper");
    assert_eq!(
        deeper.outcome,
        BulkMethodOutcome::ExplanationOnly,
        "the chain past the ceiling is delivered as an explanation, not as text"
    );
    let explanation = deeper.text.as_deref().expect("an explanation is text");
    assert!(
        explanation.contains("nests deeper than this layer renders"),
        "and the explanation names the ceiling that decided it: {explanation}"
    );
    let nested_plain = member(b"nestedPlain");
    assert!(
        nested_plain
            .text
            .as_deref()
            .is_some_and(|text| text.contains("return arg0 + 1 + (arg0 + 2);")),
        "the committed sample's nested expression keeps its grouping: {:?}",
        nested_plain.text
    );
    let nested_local = member(b"nestedLocal");
    let refusal = "the slot's name would read the value the body wrote in between";
    assert!(
        nested_local
            .text
            .as_deref()
            .is_some_and(|text| text.contains(refusal)),
        "and its refusal is delivered *inside* an artifact that states it: {:?}",
        nested_local.text
    );
    assert_eq!(
        nested_local.outcome,
        BulkMethodOutcome::Produced,
        "`Produced` says the artifact holds at least one statement, not that every byte of the \
         member was placed: this member's text is one statement and one quoted refusal"
    );
    assert_eq!(
        (
            run.report.summary.outcomes.produced,
            run.report.summary.outcomes.explanation_only,
            run.report.summary.outcomes.not_produced,
        ),
        (6, 1, 0),
        "the case's dispositions: the past-ceiling member is the explanation, and the committed \
         sample's two refusals are quoted inside artifacts that are still produced"
    );
    assert_eq!(
        run.report.summary.status(),
        "complete",
        "a value past the rendering ceiling is an outcome, not a stop: {:?}",
        run.report.summary
    );
}
// ---------------------------------------------------------------------------------------------
// The ledger the shape pins
// ---------------------------------------------------------------------------------------------

/// The fixed-shape counts, one case at a time.
///
/// This is the ledger task 1.3 asks for. Each assertion compares the operation's own counted usage
/// with the table's entry for that case; the elapsed readings are printed afterwards, under their own
/// label, and no assertion reads them.
#[test]
fn the_fixed_shape_bills_the_counts_the_corpus_pins() {
    println!("--- counted dimensions at the pinned shape (counts, not timings) ---");
    let mut readings = Vec::new();
    for case in corpus() {
        let run = run_case(&case, 1, Duration::ZERO);
        let measured = Billing::of(&run.report.usage);
        println!(
            "{:<26} {} classes(seen={} prepared={} refused={}) methods(declared={} delivered={})",
            case.name,
            measured.line(),
            run.report.summary.classes_seen,
            run.report.summary.classes_prepared,
            run.report.summary.classes_refused,
            run.report.summary.methods_declared,
            run.report.summary.methods_delivered,
        );
        assert_eq!(
            measured,
            case.billing,
            "{} ({}) is billed differently than its pinned shape:\n  pinned:   {}\n  measured: {}",
            case.name,
            case.shape,
            case.billing.literal(),
            measured.literal()
        );
        assert_eq!(
            run.report.summary.methods_delivered, case.declared_methods,
            "{} delivers the members its own archive declares",
            case.name
        );
        assert_eq!(
            run.report.summary.methods_declared, case.declared_methods,
            "{} declares them in its prepared classes",
            case.name
        );
        assert!(
            run.report.final_delivered,
            "{} reached its final event",
            case.name
        );
        readings.push((
            case.name,
            run.report.usage.elapsed_millis,
            run.elapsed_millis,
        ));
    }
    println!("--- wall-clock readings of those runs (readings, not counts) ---");
    for (name, operation, call) in readings {
        println!(
            "{name:<26} operation elapsed_millis={operation} process call elapsed_millis={call}"
        );
    }
}

/// A consumer slower than its producers changes a run's readings and nothing about its shape: the
/// `many-method-class` case — the corpus's slow-sink sample, with 107 deliveries to answer — is run
/// twice, once serially and once with two workers and a consumer that sleeps before answering every
/// callback, and the two runs publish the same summary, the same per-method records in the same order
/// and the same counted dimensions.
///
/// The two wall clocks differ, and that difference is exactly what keeps them out of the ledger: it
/// says something about this machine at this moment, not about what either run was charged.
#[test]
fn a_slow_consumer_changes_the_readings_and_not_the_shape() {
    let case = corpus()
        .into_iter()
        .find(|case| case.name == "many-method-class")
        .expect("the corpus holds the slow consumer's sample");
    let quick = run_case(&case, 1, Duration::ZERO);
    let slow = run_case(&case, 2, Duration::from_millis(1));

    assert_eq!(quick.report.summary.status(), "complete");
    assert_eq!(slow.report.summary.status(), "complete");
    let mut serial = quick.report.summary.clone();
    let mut parallel = slow.report.summary.clone();
    serial.limits.workers_requested = 0;
    serial.limits.workers_effective = 0;
    parallel.limits.workers_requested = 0;
    parallel.limits.workers_effective = 0;
    assert_eq!(
        bulk_support::fingerprint(&serial),
        bulk_support::fingerprint(&parallel),
        "a slow consumer and a second worker do not move the summary"
    );
    assert_eq!(
        Billing::of(&quick.report.usage),
        Billing::of(&slow.report.usage),
        "and they do not move a counted dimension either"
    );
    let keys = |run: &Run| -> Vec<(u64, u64, PhysicalMethodId)> {
        run.sink
            .methods()
            .iter()
            .map(|method| method.key())
            .collect()
    };
    assert_eq!(
        keys(&quick),
        keys(&slow),
        "the delivered records keep their identities and their order"
    );
    let quiet = quick.sink.methods();
    let loud = slow.sink.methods();
    assert_eq!(quiet.len(), 107);
    for (one, many) in quiet.iter().zip(&loud) {
        assert_eq!(one.text, many.text);
        assert_eq!(one.outcome, many.outcome);
    }
    println!(
        "slow consumer (readings only): serial elapsed_millis={} process call elapsed_millis={}; \
         two workers with a 1 ms consumer elapsed_millis={} process call elapsed_millis={}",
        quick.report.usage.elapsed_millis,
        quick.elapsed_millis,
        slow.report.usage.elapsed_millis,
        slow.elapsed_millis
    );
}

// ---------------------------------------------------------------------------------------------
// The two old arms: one request per method, without and with the one shared store
// ---------------------------------------------------------------------------------------------

/// One arm's real ledger: what every request of it was charged, added up, and how many requests it
/// was.
///
/// The dimensions are kept by name rather than by field so an arm's whole account is printed without
/// this file having to restate every dimension's spelling: it reads
/// [`CountedBudgetDimension::ALL`]'s order and the usage snapshot's own accessor.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ArmLedger {
    requests: u64,
    counted: Vec<(&'static str, u64)>,
}

impl ArmLedger {
    fn new() -> Self {
        Self {
            requests: 0,
            counted: CountedBudgetDimension::ALL
                .iter()
                .map(|dimension| (dimension_name(*dimension), 0))
                .collect(),
        }
    }

    /// Adds one request's usage to this arm.
    fn add(&mut self, usage: &UsageSnapshot) {
        self.requests += 1;
        for (slot, dimension) in self.counted.iter_mut().zip(CountedBudgetDimension::ALL) {
            slot.1 += usage.counted_usage(dimension);
        }
    }

    fn get(&self, name: &str) -> u64 {
        self.counted
            .iter()
            .find(|(dimension, _)| *dimension == name)
            .map(|(_, value)| *value)
            .unwrap_or_else(|| panic!("no counted dimension named {name}"))
    }

    /// The nine dimensions the case table pins, read off this arm's ledger, so an arm's account is
    /// compared with the table's own spelling of a shape's cost.
    fn billing(&self) -> Billing {
        Billing {
            archive_entries: self.get("archive_entries"),
            entry_bytes: self.get("entry_bytes"),
            class_bytes: self.get("class_bytes"),
            class_headers: self.get("class_headers"),
            method_bodies: self.get("method_bodies"),
            ir_items: self.get("ir_items"),
            analysis_steps: self.get("analysis_steps"),
            result_items: self.get("result_items"),
            output_bytes: self.get("output_bytes"),
        }
    }

    fn line(&self) -> String {
        self.counted
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// What an execution report *states*, without what it spent: the state, and for a stop its reason.
///
/// The usage half of an execution report is a reading of the run that produced it, so two arms'
/// reports are compared through their result half — the same rule the bulk fingerprints apply when
/// they leave a run's resource fields out of a semantic comparison.
fn termination(execution: &ExecutionReport) -> String {
    match execution {
        ExecutionReport::Complete { .. } => "complete".to_owned(),
        ExecutionReport::Partial { reason, .. } => format!("partial {reason:?}"),
        ExecutionReport::Cancelled { .. } => "cancelled".to_owned(),
        ExecutionReport::Failed { reason, .. } => format!("failed {reason:?}"),
    }
}

/// A counted dimension's own name, so a printed ledger does not depend on this file's spelling.
fn dimension_name(dimension: CountedBudgetDimension) -> &'static str {
    match dimension {
        CountedBudgetDimension::InputBytes => "input_bytes",
        CountedBudgetDimension::ArchiveEntries => "archive_entries",
        CountedBudgetDimension::EntryBytes => "entry_bytes",
        CountedBudgetDimension::ReadBytes => "read_bytes",
        CountedBudgetDimension::ClassBytes => "class_bytes",
        CountedBudgetDimension::AttributeBytes => "attribute_bytes",
        CountedBudgetDimension::CodeBytes => "code_bytes",
        CountedBudgetDimension::ResultItems => "result_items",
        CountedBudgetDimension::OutputBytes => "output_bytes",
        CountedBudgetDimension::ClassHeaders => "class_headers",
        CountedBudgetDimension::MethodBodies => "method_bodies",
        CountedBudgetDimension::IrItems => "ir_items",
        CountedBudgetDimension::IrEdges => "ir_edges",
        CountedBudgetDimension::AnalysisSteps => "analysis_steps",
        CountedBudgetDimension::NormalizationClones => "normalization_clones",
    }
}

/// The old per-method way of running the same corpus work, as the change's arms A and B: one request
/// per method, one fresh budget per request, the second arm carrying one shared facts store that
/// starts empty.
///
/// What is asserted about them is semantics and shape, never speed:
///
/// * the three paths — this operation's own records, the direct arm and the shared-store arm — name
///   the same physical member, and the two arms present the same text for it, member for member;
/// * the store really answered (the shared arm is not an arm that silently ran without one) and
///   refused nothing it was given, so that arm's ledger is retention's own;
/// * retention moves the *read* dimensions and leaves the *work* dimensions alone: decoding,
///   analysis and delivery are charged the same in both arms.
///
/// The two totals are printed beside the bulk operation's own ledger for the same corpus. They are
/// the per-request shape's real account, and reading them as a comparison of speed would be exactly
/// the mistake this corpus exists to make visible: the per-request path reads its class again for
/// every member, and the operation reads it once per class — a difference in how the work is
/// *addressed*, which is what the counts state and all they state.
#[test]
fn the_old_per_method_arms_keep_their_ledger_and_the_same_text() {
    let store = FactsCache::current(FactsCapacity::new(1 << 10, 1 << 24));
    let mut direct = ArmLedger::new();
    let mut shared = ArmLedger::new();
    let mut compared_texts = 0_u64;
    let mut compared_planes = 0_u64;
    let workload = Instant::now();
    for case in corpus() {
        let run = run_case(&case, 1, Duration::ZERO);
        let (snapshot, _opened) = open((case.build)());
        let content = vec![snapshot.clone()];
        let mut setup = Budget::new(bulk_support::limits());
        let roots = container_roots(&snapshot, &mut setup, case.prefixes);
        let built = environment(&snapshot, tree_scope(), roots)
            .build(&content)
            .expect("the case's environment builds");
        for record in run.sink.methods() {
            let request = MethodAnalysisRequest {
                environment: built.clone(),
                method: record.method.clone(),
                stages: MethodOperation::Recovery.stages().to_vec(),
            };
            let mut budget = Budget::new(bulk_support::limits());
            let one_at_a_time = Engine::new()
                .recover_method_with_evidence(
                    &content,
                    &request,
                    &RecoveryEvidenceRequest::all(),
                    &mut budget,
                )
                .expect("the direct arm recovers the member");
            direct.add(&budget.usage());
            assert_eq!(
                one_at_a_time.analysis().method,
                record.method,
                "{} `{}` is the member the operation delivered",
                String::from_utf8_lossy(&record.method.name.0),
                String::from_utf8_lossy(&record.method.descriptor.0)
            );
            if let Some(text) = record.text.as_ref() {
                assert_eq!(
                    text,
                    &one_at_a_time.recovery().text,
                    "the bulk operation and the direct arm present the same text"
                );
                compared_texts += 1;
            }
            assert_eq!(
                record.text.is_some(),
                !one_at_a_time.recovery().text.is_empty(),
                "the bulk operation and the direct arm agree on whether this member has text at all"
            );

            let mut budget = Budget::new(bulk_support::limits()).with_facts_cache(store.clone());
            let through_the_store = Engine::new()
                .recover_method_with_evidence(
                    &content,
                    &request,
                    &RecoveryEvidenceRequest::all(),
                    &mut budget,
                )
                .expect("the shared-store arm recovers the member");
            shared.add(&budget.usage());
            assert_eq!(
                through_the_store.recovery().text,
                one_at_a_time.recovery().text,
                "the shared store changes no text"
            );
            assert_eq!(
                through_the_store.recovery().outcome,
                one_at_a_time.recovery().outcome,
                "and no outcome"
            );
            assert_eq!(
                through_the_store.recovery().content,
                one_at_a_time.recovery().content,
                "and no content classification"
            );
            assert_eq!(
                termination(&through_the_store.recovery().execution),
                termination(&one_at_a_time.recovery().execution),
                "and no execution state; only its resource readings may differ"
            );
            compared_planes += 1;
        }
    }
    let workload_millis = workload.elapsed().as_millis();

    // Semantics: both arms ran over every member the operation delivered, text for text and plane
    // for plane.
    let members: u64 = corpus().iter().map(|case| case.declared_methods).sum();
    assert_eq!(members, 187, "the corpus's declared denominator");
    assert_eq!(direct.requests, members, "one request per declared member");
    assert_eq!(shared.requests, members);
    assert_eq!(
        compared_planes, members,
        "both arms ran over every member the operation delivered"
    );
    assert_eq!(
        compared_texts, 181,
        "every member that carries text is compared on all three paths; the six that carry none \
         are the corpus's two declarations without a body and the four members of the shadowed \
         origin, and each of those is also checked for that absence itself"
    );

    // The store really was a store: it answered, and refused none of the answers it was given, so the
    // shared arm's ledger is retention's own and not a ledger of capacity refusals.
    let facts = store.report();
    assert!(
        facts.hits > 0,
        "the shared arm asked the store and it answered: {facts:?}"
    );
    assert_eq!(
        (facts.refused_capacity, facts.refused_capacity_bytes),
        (0, 0),
        "the store's capacity is wide enough that the arm is retention, not refusal: {facts:?}"
    );
    assert!(
        facts.directory_parses > 0 && facts.container_stored > 0,
        "the container reads of this arm went through it: {facts:?}"
    );

    // Shape, and only shape: retention moves reads, not work. The dimensions below are the ones the
    // two arms charge identically — decoding a body, building the IR, analysing and delivering it are
    // the same work whether or not a store answered the reads behind them. A count falling in the
    // other list is a statement about how much less was *addressed*, never about how long anything
    // took: the two arms' wall clocks are printed below under their own label, and this test asserts
    // nothing about them.
    println!(
        "--- the two old arms' ledgers over the whole corpus (counts, not timings) ---\n\
         direct  requests={} {}\n\
         shared  requests={} {}\n\
         store   {}\n\
         reuse   {}",
        direct.requests,
        direct.line(),
        shared.requests,
        shared.line(),
        facts.residency(),
        facts.reuse(),
    );
    // The arms' own pinned account, over the whole corpus at the same shape the case table pins.
    assert_eq!(
        direct.billing(),
        Billing::DIRECT_ARM,
        "the direct arm is billed differently than the shape it is pinned at"
    );
    assert_eq!(
        shared.billing(),
        Billing::SHARED_ARM,
        "the shared-store arm is billed differently than the shape it is pinned at"
    );
    for dimension in [
        "class_headers",
        "method_bodies",
        "ir_items",
        "ir_edges",
        "analysis_steps",
        "normalization_clones",
        "code_bytes",
        "output_bytes",
    ] {
        assert_eq!(
            shared.get(dimension),
            direct.get(dimension),
            "`{dimension}` is charged by the work the two arms share, not by retention"
        );
    }
    for dimension in [
        "archive_entries",
        "entry_bytes",
        "read_bytes",
        "class_bytes",
        "attribute_bytes",
        "result_items",
    ] {
        assert!(
            shared.get(dimension) < direct.get(dimension),
            "the one store stops this arm's per-request `{dimension}` reads: {} against {}",
            shared.get(dimension),
            direct.get(dimension)
        );
    }

    println!(
        "--- wall-clock readings of the two arms (readings, not counts) ---\n\
         the two arms' {} requests — one per arm per member — took {workload_millis} ms together, on \
         this machine at this moment; no assertion reads this number, and the counts above cannot be \
         read as it",
        members * 2
    );
}

/// Regenerates the pinned table after an intended shape change.
///
/// Run `cargo test --test p5_bulk_corpus --locked -- --ignored --nocapture record_the_billing_table`
/// and paste the printed literals into the `Billing` constants. A run of this test is a *shape* change
/// by definition: no machine, no load and no moment may cause one, and a diff that only replaces these
/// numbers without a reason in the case's own shape is exactly what the pin is for.
#[test]
#[ignore = "regenerates the pinned table; run with --ignored --nocapture after an intended shape change"]
fn record_the_billing_table() {
    for case in corpus() {
        let run = run_case(&case, 1, Duration::ZERO);
        let measured = Billing::of(&run.report.usage);
        println!("// {:<26} {}", case.name, case.shape);
        println!("{}", measured.literal());
        println!(
            "// classes(seen={} prepared={} refused={}) methods(declared={} delivered={}) \
             outcomes(produced={} explanation_only={} not_produced={} refused={} no_body={})",
            run.report.summary.classes_seen,
            run.report.summary.classes_prepared,
            run.report.summary.classes_refused,
            run.report.summary.methods_declared,
            run.report.summary.methods_delivered,
            run.report.summary.outcomes.produced,
            run.report.summary.outcomes.explanation_only,
            run.report.summary.outcomes.not_produced,
            run.report.summary.outcomes.refused,
            run.report.summary.outcomes.no_body,
        );
    }
}
