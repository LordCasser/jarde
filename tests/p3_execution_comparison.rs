//! P3 3.3: the **replayable** Java 8 compile-and-execute comparison of the recovered bodies.

use jarde::*;
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::method_ir::MethodIr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// `ACC_STATIC` (JVMS table 4.6-A): the flag this file reads to decide `static` in a declaration.
const ACC_STATIC: u16 = 0x0008;
/// `ACC_PUBLIC`, for the same reason.
const ACC_PUBLIC: u16 = 0x0001;

// -------------------------------------------------------------------------------------------
// A temporary directory, unique per sample (the ignored JDK oracle's own shape).
// -------------------------------------------------------------------------------------------

static UNIQUE: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-comparison-{label}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create the comparison directory");
        Self(path)
    }

    fn write(&self, name: &str, bytes: &[u8]) {
        fs::write(self.0.join(name), bytes).expect("write into the comparison directory");
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// -------------------------------------------------------------------------------------------
// The budget and environment every request in this file is made under (the P3 test files' shape).
// -------------------------------------------------------------------------------------------

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
        class_headers: 64,
        method_bodies: 64,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
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

// -------------------------------------------------------------------------------------------
// The corpus: what is read, by which compiler, and what each member must turn out to be.
// -------------------------------------------------------------------------------------------

/// One declaration the scratch class states besides the member under test: a name a recovered body
/// may use, spelled the way the layer spells it. The sample stays on the classpath, so the scratch
/// class is a wrapper and never a second environment.
struct Scaffold {
    name: &'static str,
    descriptor: &'static str,
    java: &'static str,
}

/// What the run must have done with one member, and therefore what this file does with it.
enum Expect {
    /// The whole body is Java and the wrapper compiles: compile both sides and compare the traces.
    Executed,
    /// The whole body is written as Java, but its text is not a compilation unit: the boundary
    /// records javac's refusal. The `&str` states what this file thinks the boundary is.
    NotACompilationUnit(&'static str),
    /// Part or all of the body is quoted bytecode: check the quote, do not execute. The `Option` is
    /// the diagnostic code the report must state for the refusal, when it states one.
    Quoted(Option<&'static str>),
}

struct Member {
    name: &'static str,
    expect: Expect,
}

/// One class file of a sample's classpath: the name it is written under, and its bytes.
type ClasspathFile = (&'static str, &'static [u8]);

/// One member's own calls: the member's name and one argument list per call, each argument written
/// as Java source text.
type MemberInputs = (&'static str, &'static [&'static [&'static str]]);

/// The committed driver of one sample: what the **original** class does, run in a controlled way.
///
/// A refusal may rest on an effect the original really performs — a static initializer that runs, the
/// `NullPointerException` a field read throws, the value an increment changed — and asserting that
/// from memory would be asserting the fixture's *intent* rather than its bytes. The driver is a real
/// source beside the fixture (so the corpus fingerprint covers it) that the ignored test compiles
/// with the sample on its classpath and runs as its own program; the lines it prints are asserted
/// exactly, and the fixture's README records the run.
struct Baseline {
    /// The class the committed source declares, and the class `java` runs.
    class: &'static str,
    /// The committed source, read at compile time (`include_str!`, relative to this file): a driver
    /// that is missing, renamed or emptied fails the build rather than quietly observing nothing.
    source: &'static str,
    /// What it prints, exactly, one line per observation.
    lines: &'static [&'static str],
}

struct Sample {
    /// The row label: which fixture, which compiler, which flags.
    label: &'static str,
    /// The class the committed bytes declare.
    class: &'static str,
    bytes: &'static [u8],
    /// Everything that has to be on the classpath besides the sample itself.
    classpath: &'static [ClasspathFile],
    /// The scratch class's `extends` clause: stated for a non-final sample whose bodies name the
    /// sample's own members, so that those names resolve to the committed class.
    extends: Option<&'static str>,
    scaffold: &'static [Scaffold],
    /// The sample's own counter, when it states one: it is printed around every call, which is how
    /// "how many times a call happened" is compared.
    counter: Option<&'static str>,
    /// The members whose **count control** is worth measuring: a refused body whose written text
    /// still performs the effect the count is about (P3-R2's `cast`, whose text calls `make()` once).
    /// A refused body that quotes the effect itself has no count to compare — the fragment it would
    /// be run as does not perform it — so it is not listed here, and what varies with the refusal is
    /// measured by `baseline` on the original instead.
    measured: &'static [&'static str],
    /// The calls one member is made with, when the default input set cannot state the finding's own
    /// inputs: `("a", "bc")` is the pair the receiver-grouping defect was measured on, while a
    /// `String` parameter's default values are `"r"` and `null` — inputs on which the two texts
    /// happen to answer the same. Keyed by member name; a member not listed keeps the default set,
    /// so a sample that states nothing here is called exactly as before.
    inputs: Option<&'static [MemberInputs]>,
    /// The quoted bytecode each refused member must name, for the samples whose refusals exist to pin
    /// *which* instructions the answer accounts for: `(member, the indexes its quotes state)`. The
    /// set is compared exactly, so a quote that names an instruction it did not read fails here as
    /// well as one that dropped the read the refusal was about.
    quotes: &'static [(&'static str, &'static [u32])],
    /// The committed baseline driver of this sample, when its refusals rest on observations of the
    /// original class rather than on the recovered text.
    baseline: Option<Baseline>,
    /// Every member this comparison classifies. A member the sample declares that is **not** named
    /// here fails the run: an unclassified member would be an uncovered one.
    members: &'static [Member],
    /// What this sample proves, in one line, for the record.
    point: &'static str,
}

const FLAGS_MEMBERS: &[Member] = &[
    Member {
        name: "counted",
        expect: Expect::Executed,
    },
    Member {
        name: "copied",
        expect: Expect::Executed,
    },
    Member {
        name: "choose",
        expect: Expect::Executed,
    },
];

const LOCAL_REWRITE: Sample = Sample {
    label: "p3-local-rewrite/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "LocalRewrite",
    bytes: include_bytes!("fixtures/p3-local-rewrite/v8/LocalRewrite.class"),
    classpath: &[],
    extends: None,
    scaffold: &[Scaffold {
        name: "make",
        descriptor: "()Ljava/lang/Object;",
        // The class is `final`, so no wrapper can extend it: the name the body of `cast` uses is
        // stated here as a pass-through, which is what makes the count of `make` calls observable.
        java: "static Object make() { return LocalRewrite.make(); }",
    }],
    counter: Some("LocalRewrite.calls"),
    // R2's substance is the count of `make` calls the refused `cast` text makes; the other three
    // refusals of this sample are counted for the same reason (their control compiles wherever the
    // text still holds the statement the count is about, and the row states where it does not).
    measured: &["post", "saved", "conditional", "cast"],
    inputs: None,
    // The quotes of this sample are pinned by `tests/p3_local_rewrite.rs`, which needs no JDK.
    quotes: &[],
    baseline: None,
    members: &[
        Member {
            name: "post",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "bump",
            expect: Expect::Executed,
        },
        Member {
            name: "doubleIt",
            expect: Expect::Executed,
        },
        Member {
            name: "saved",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "conditional",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "loopAcross",
            expect: Expect::Executed,
        },
        Member {
            name: "cast",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "make",
            expect: Expect::Executed,
        },
    ],
    point: "P3-R1 (`post`, `saved`, `conditional` refuse the read they cannot prove) with its \
            negative controls (`bump`, `doubleIt`, `loopAcross` still write the slot name), and \
            P3-R2 (`cast` keeps the producer call: the count of `make` invocations is measured)",
};

const SCOPE_MEMBERS: &[Member] = &[
    Member {
        name: "scope",
        expect: Expect::Executed,
    },
    Member {
        name: "simple",
        expect: Expect::Executed,
    },
    Member {
        name: "armOnly",
        expect: Expect::Executed,
    },
    Member {
        name: "reuse",
        expect: Expect::Executed,
    },
    Member {
        name: "after",
        expect: Expect::Executed,
    },
    Member {
        name: "reassign",
        expect: Expect::Executed,
    },
    Member {
        name: "receiver",
        expect: Expect::Executed,
    },
];

const SCOPE_NO_DEBUG: Sample = Sample {
    label: "p3-scope/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "Scope",
    bytes: include_bytes!("fixtures/p3-scope/v8/Scope.class"),
    classpath: &[],
    extends: None,
    scaffold: &[],
    counter: None,
    measured: &[],
    inputs: None,
    quotes: &[],
    baseline: None,
    members: SCOPE_MEMBERS,
    point: "P3-R3 (a declaration hoisted above both arms compiles) and P3-R5 (`boolean` from the \
            descriptor; `arg2` is the category-2 parameter's own slot)",
};

const SCOPE_DEBUG: Sample = Sample {
    label: "p3-scope/v8-debug (javac 23.0.1, --release 8 -g) - the same source",
    class: "Scope",
    bytes: include_bytes!("fixtures/p3-scope/v8-debug/Scope.class"),
    classpath: &[],
    extends: None,
    scaffold: &[],
    counter: None,
    measured: &[],
    inputs: None,
    quotes: &[],
    baseline: None,
    members: SCOPE_MEMBERS,
    point: "the same shapes with a `LocalVariableTable`: the wrapper's parameter names come from \
            the class file's own table (`b`, `seed`, `a`) instead of the ordinal the reader invents",
};

const GUARDED: Sample = Sample {
    label: "p3-handlers/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "Guarded",
    bytes: include_bytes!("fixtures/p3-handlers/v8/Guarded.class"),
    classpath: &[(
        "Res.class",
        include_bytes!("fixtures/p3-handlers/v8/Res.class"),
    )],
    extends: Some("Guarded"),
    scaffold: &[],
    counter: None,
    measured: &[],
    inputs: None,
    quotes: &[],
    baseline: None,
    members: &[
        // P3-R5's argument side, closed: the `new@1` rule wrote `new Res(arg0, 0)` — an `int` constant
        // where the constructor's own descriptor declares `boolean` — until the shared argument path
        // started typing every call's arguments by the callee's descriptor. Both members execute now,
        // so the fixture that states what a call's arguments are is compared and not just compiled:
        // the constructor still receives `false`/`true`, and the value the body returns is the same
        // `Res` and the same printed line on both sides.
        Member {
            name: "open",
            expect: Expect::Executed,
        },
        Member {
            name: "openFailing",
            expect: Expect::Executed,
        },
        Member {
            name: "fail",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "body",
            expect: Expect::Executed,
        },
        Member {
            name: "tail",
            expect: Expect::Executed,
        },
        Member {
            name: "boom",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "one",
            expect: Expect::Executed,
        },
        Member {
            name: "two",
            expect: Expect::Executed,
        },
        Member {
            name: "three",
            expect: Expect::Executed,
        },
        Member {
            name: "suppressed",
            expect: Expect::Executed,
        },
        Member {
            name: "secondInitFails",
            expect: Expect::Executed,
        },
        Member {
            name: "sync",
            expect: Expect::Executed,
        },
        Member {
            name: "syncBody",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "syncThrows",
            expect: Expect::Executed,
        },
        Member {
            name: "withCatch",
            expect: Expect::Quoted(Some("jre_guard_unexplained_row")),
        },
        Member {
            name: "branching",
            expect: Expect::Quoted(Some("jre_guard_body")),
        },
        Member {
            name: "fin",
            expect: Expect::Quoted(Some("jre_guard_finally_copy")),
        },
        Member {
            name: "catchFinally",
            expect: Expect::Quoted(Some("jre_guard_finally_copy")),
        },
        Member {
            name: "main",
            expect: Expect::Executed,
        },
        Member {
            name: "syncThrowsCatching",
            expect: Expect::Quoted(Some("jre_guard_resource_init")),
        },
        Member {
            name: "secondInitFailsCatching",
            expect: Expect::Quoted(Some("jre_guard_resource_init")),
        },
        Member {
            name: "suppressedCatching",
            expect: Expect::Quoted(Some("jre_region_irreducible")),
        },
    ],
    point: "P3 2.4's guarded shapes: the trace states the order of every open, body and close \
            (including the reverse close order of `two`/`three`) and the primary/suppressed pair of \
            `suppressed`; the `finally` copies and the guarded bodies that branch stay refusals",
};

const ECJ_V52: Sample = Sample {
    label: "historical/ecj-4.6.1/v52 (ECJ 4.6.1, -source 1.3, target 52.0)",
    class: "HistoricalControlFlow",
    bytes: include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
    classpath: &[],
    extends: None,
    scaffold: &[],
    counter: None,
    measured: &[],
    inputs: None,
    quotes: &[],
    baseline: None,
    members: &[
        Member {
            name: "add",
            expect: Expect::Executed,
        },
        // P3-R7: `finallyPath` declares `Exception table: from 0 to 4 target 9 type any` while the
        // decode reads BCI 9's four instructions and the graph's only node is `[0, 9)` — nothing of
        // `[0, 4)` can throw synchronously, so the handler was never made a node and never listed as
        // dead. The run refuses the body whole under the reason that quotes BCI 9/10/13/14; the `v45`
        // body of the same class file keeps its handler inside a dead node and is refused for its own
        // reasons instead (P3-R7's other half, `p3_java_recovery`).
        Member {
            name: "finallyPath",
            expect: Expect::Quoted(Some("jre_region_unaccounted_instruction")),
        },
    ],
    point: "a second compiler at the same class-file version: `add` is presented whole by the same \
            walk and the values agree, and `finallyPath` — whose graph is not an account of its own \
            handler — is quoted instead of being presented as if it were the body",
};

const MISSING_DEPENDENCY: Sample = Sample {
    label: "p3-corpus/v8-missing-dep (javac 23.0.1, --release 8 -g:none)",
    class: "MissingDependency",
    bytes: include_bytes!("fixtures/p3-corpus/v8-missing-dep/MissingDependency.class"),
    classpath: &[],
    extends: None,
    scaffold: &[],
    counter: None,
    measured: &[],
    inputs: None,
    quotes: &[],
    baseline: None,
    members: &[
        Member {
            name: "viaAbsentLibrary",
            expect: Expect::NotACompilationUnit(
                "the type the body names is not shipped with the fixture: the text needs a class \
                 the layer never had to read",
            ),
        },
        Member {
            name: "plain",
            expect: Expect::Executed,
        },
    ],
    point: "a class whose dependency is missing: the layer reads one class file and states the \
            call, and the wrapper cannot name that type - the boundary is the absent class itself",
};

const fn flags(label: &'static str, bytes: &'static [u8]) -> Sample {
    Sample {
        label,
        class: "Flags",
        bytes,
        classpath: &[],
        extends: Some("Flags"),
        scaffold: &[],
        counter: Some("Flags.probes"),
        measured: &[],
        inputs: None,
        quotes: &[],
        baseline: None,
        members: FLAGS_MEMBERS,
        point: "one source, several legal flag sets: the same shapes, and different debug evidence \
                for them",
    }
}

const FLAGS_G_NONE: Sample = flags(
    "p3-corpus/v8-gnone (javac 23.0.1, --release 8 -g:none)",
    include_bytes!("fixtures/p3-corpus/v8-gnone/Flags.class"),
);
const FLAGS_G: Sample = flags(
    "p3-corpus/v8-g (javac 23.0.1, --release 8 -g)",
    include_bytes!("fixtures/p3-corpus/v8-g/Flags.class"),
);
const FLAGS_G_LINES: Sample = flags(
    "p3-corpus/v8-glines (javac 23.0.1, --release 8 -g:lines,source)",
    include_bytes!("fixtures/p3-corpus/v8-glines/Flags.class"),
);
const FLAGS_PARAMETERS: Sample = flags(
    "p3-corpus/v8-parameters (javac 23.0.1, --release 8 -parameters -g:none)",
    include_bytes!("fixtures/p3-corpus/v8-parameters/Flags.class"),
);
/// The same source and the same debug flag as `v8-gnone`, generated with `-source 8 -target 8`
/// instead of `--release 8`: the corpus states that the two spellings produce the same bytes.
const FLAGS_SOURCE_TARGET: &[u8] =
    include_bytes!("fixtures/p3-corpus/v8-source-target/Flags.class");

const NESTED_EVAL: Sample = Sample {
    label: "p3-nested-eval/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "NestedEval",
    bytes: include_bytes!("fixtures/p3-nested-eval/v8/NestedEval.class"),
    classpath: &[],
    extends: Some("NestedEval"),
    scaffold: &[],
    counter: Some("NestedEval.calls"),
    // A refused body of this sample quotes the call it could not write, so the count control — which
    // runs the *written* fragment and compares how often it moves the counter — has no count to
    // compare: nothing of the effect is in the text. What the refusal rests on is the original's own
    // answer, and `baseline` is where that is executed.
    measured: &[],
    inputs: None,
    quotes: &[("nestedLocal", &[8, 0]), ("nestedCall", &[9, 1])],
    baseline: Some(Baseline {
        class: "Baseline",
        source: include_str!("fixtures/p3-nested-eval/Baseline.java"),
        lines: &["nestedLocal(7)=16", "nestedCall(3)=7", "tick calls=1"],
    }),
    members: &[
        Member {
            name: "tick",
            expect: Expect::Executed,
        },
        Member {
            name: "nestedLocal",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "nestedPlain",
            expect: Expect::Executed,
        },
        Member {
            name: "nestedCall",
            expect: Expect::Quoted(None),
        },
    ],
    point: "P3-R8: `(x + 1) + ++x` is refused because the load it needs is judged where the text is \
            evaluated and the slot no longer holds it by then — the quote names the consumer and the \
            read — while `(x + 1) + (x + 2)`, the same shape with no write in between, is written \
            whole and executed (its trace is identical, with the counter moved once by `tick`)",
};

const REFUSED_CAST: Sample = Sample {
    label: "p3-refused-cast/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "RefusedCast",
    bytes: include_bytes!("fixtures/p3-refused-cast/v8/RefusedCast.class"),
    classpath: &[
        (
            "External.class",
            include_bytes!("fixtures/p3-refused-cast/v8/External.class"),
        ),
        (
            "Holder.class",
            include_bytes!("fixtures/p3-refused-cast/v8/Holder.class"),
        ),
    ],
    extends: Some("RefusedCast"),
    scaffold: &[],
    counter: Some("RefusedCast.calls"),
    // The three refused members quote the read and the cast rather than writing them, so the written
    // fragments perform none of the reads: their evidence is the committed original, run by
    // `baseline`, and the quotes' own BCIs (the reads at BCI 0/1/3 down to the class initializer).
    measured: &[],
    inputs: None,
    quotes: &[
        ("fieldCast", &[0, 3, 6]),
        ("instanceCast", &[1, 4, 7]),
        ("chainCast", &[0, 3, 6, 9]),
    ],
    baseline: Some(Baseline {
        class: "Baseline",
        source: include_str!("fixtures/p3-refused-cast/Baseline.java"),
        lines: &[
            "fieldCast()=ok",
            "External initializations=1",
            "instanceCast(new External())=instance-ok",
            "instanceCast(null)=java.lang.NullPointerException",
            "chainCast()=chained",
            "leftRead()=7",
            "rightRead()=7",
            "tick calls=3",
        ],
    }),
    members: &[
        Member {
            name: "fieldCast",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "instanceCast",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "chainCast",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "leftRead",
            expect: Expect::Executed,
        },
        Member {
            name: "rightRead",
            expect: Expect::Executed,
        },
        Member {
            name: "tick",
            expect: Expect::Executed,
        },
    ],
    point: "P3-R9: a refused cast keeps the observable producers its expression depended on — the \
            `getstatic` that can run `External`'s static initializer (measured once by the driver), \
            the `getfield` that can throw for a null receiver, and the read behind another read — \
            while a claimed read composed with a deferred call stays written whole",
};

const NESTED_ARITHMETIC: Sample = Sample {
    label: "p3-nested-arithmetic/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "ModLike",
    bytes: include_bytes!("fixtures/p3-nested-arithmetic/v8/ModLike.class"),
    classpath: &[],
    extends: None,
    scaffold: &[],
    counter: None,
    measured: &[],
    inputs: None,
    quotes: &[],
    // Every member of this sample is written whole, so no refusal rests on the original's own
    // behaviour; the driver is here because the *positive* comparison's inputs are the original's
    // too, and `inverse32(-1)` is the input the defect was measured on.
    baseline: Some(Baseline {
        class: "Baseline",
        source: include_str!("fixtures/p3-nested-arithmetic/Baseline.java"),
        lines: &[
            "inverse32(-1)=-1",
            "inverse32(7)=-1227133513",
            "inverse32(0)=0",
            "scaledDifference(3, 5)=-39",
            "nestedDifference(10, 4, 7)=13",
            "nestedQuotient(20, 3, 4)=1",
            "differenceOfSum(10, 3)=2",
            "productOfSum(2, 3, 4)=20",
            "sumOfProducts(2, 3, 4)=14",
            "leftNestedSum(2, 3, 4)=9",
        ],
    }),
    members: &[
        Member {
            name: "inverse32",
            expect: Expect::Executed,
        },
        Member {
            name: "scaledDifference",
            expect: Expect::Executed,
        },
        Member {
            name: "nestedDifference",
            expect: Expect::Executed,
        },
        Member {
            name: "nestedQuotient",
            expect: Expect::Executed,
        },
        Member {
            name: "differenceOfSum",
            expect: Expect::Executed,
        },
        Member {
            name: "productOfSum",
            expect: Expect::Executed,
        },
        Member {
            name: "sumOfProducts",
            expect: Expect::Executed,
        },
        Member {
            name: "leftNestedSum",
            expect: Expect::Executed,
        },
    ],
    point: "the printer's grouping: every arithmetic operand is printed so that the text parses \
            back into the tree it was printed from, so the original and the generated body return \
            the same value for every input — `inverse32(-1)` is -1 on both sides, where the text \
            that lost its group answered -81 — while the two shapes Java's own precedence and \
            associativity already state (`a + b * c`, `(a + b) + c`) gain no parentheses",
};

const RECEIVER_GROUPING: Sample = Sample {
    label: "p3-receiver-grouping/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "ReceiverGrouping",
    bytes: include_bytes!("fixtures/p3-receiver-grouping/v8/ReceiverGrouping.class"),
    classpath: &[],
    extends: Some("ReceiverGrouping"),
    scaffold: &[],
    counter: None,
    measured: &[],
    // The default `String` values are `"r"` and `null`, and `call("r", "r")` is exactly the input
    // where the two texts agree: the finding is `("a", "bc")`, so the calls the members are made
    // with are stated here, in source text, and both sides are called with the same ones.
    inputs: Some(&[
        (
            "call",
            &[
                &["\"a\"", "\"bc\""],
                &["\"r\"", "\"r\""],
                &["null", "\"bc\""],
            ],
        ),
        ("length", &[&["\"a\"", "\"bc\""], &["\"\"", "\"xy\""]]),
        (
            "nested",
            &[&["\"a\"", "\"bc\"", "\"def\""], &["\"\"", "\"\"", "\"x\""]],
        ),
        ("plain", &[&["\"  a \""], &["\"a\""]]),
        ("chained", &[&["\"  a \""], &["\"a\""]]),
        ("same", &[&["\"a\"", "\"b\"", "\"c\""]]),
        ("argument", &[&["\"a\"", "\"bc\""]]),
    ]),
    quotes: &[],
    baseline: Some(Baseline {
        class: "Baseline",
        source: include_str!("fixtures/p3-receiver-grouping/Baseline.java"),
        lines: &[
            "call(\"a\", \"bc\")=bc",
            "call(\"r\", \"r\")=r",
            "call(null, \"bc\")=ullbc",
            "length(\"a\", \"bc\")=3",
            "nested(\"a\", \"bc\", \"def\")=bcdef",
            "plain(\"  a \")=[a]",
            "chained(\"  a \")=1",
            "same(\"a\", \"b\", \"c\")=abc",
            "argument(\"a\", \"bc\")=abc",
        ],
    }),
    members: &[
        Member {
            name: "call",
            expect: Expect::Executed,
        },
        Member {
            name: "length",
            expect: Expect::Executed,
        },
        Member {
            name: "nested",
            expect: Expect::Executed,
        },
        Member {
            name: "plain",
            expect: Expect::Executed,
        },
        Member {
            name: "chained",
            expect: Expect::Executed,
        },
        Member {
            name: "same",
            expect: Expect::Executed,
        },
        Member {
            name: "argument",
            expect: Expect::Executed,
        },
        Member {
            name: "wrap",
            expect: Expect::Executed,
        },
    ],
    point: "the printer's grouping is a property of the **position**: a call's receiver \
            (`(a + b).substring(1)`), a receiver built from a three-operand chain \
            (`(a + b + c).substring(1)`) and a call on a concatenation's length \
            (`(a + b).length()`) all keep the operand's group, so `call(\"a\", \"bc\")` answers \
            \"bc\" on both sides where the text that dropped the parentheses answered \"ac\" — while \
            a name receiver (`arg0.trim()`), a nested call (`arg0.trim().length()`), an argument \
            (`wrap(a + b)`) and a left-associative chain (`a + b + c`) gain nothing",
};

const BOOLEAN_CONTEXTS: Sample = Sample {
    label: "p3-boolean-contexts/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "BooleanContexts",
    bytes: include_bytes!("fixtures/p3-boolean-contexts/v8/BooleanContexts.class"),
    classpath: &[],
    // `parity` and `callFlag` call the sample's own `flag`, so the scratch class extends the sample
    // (the class is not `final`) and those names resolve to the committed class's members.
    extends: Some("BooleanContexts"),
    scaffold: &[],
    counter: None,
    measured: &[],
    // The default `int` input set is 7, 0, -1 and the review's own inputs are `isZero(0)` and
    // `isZero(1)`: `1` is the one the default set does not carry, so the members the finding was
    // measured on state their calls here and both sides are called with the same ones. `pick` and
    // `intLocal` state theirs so that each arm of the branch is executed explicitly rather than left
    // to the default list's rotation.
    inputs: Some(&[
        ("isZero", &[&["0"], &["1"], &["7"], &["-1"]]),
        ("parity", &[&["0"], &["1"]]),
        ("pick", &[&["7", "true", "true"], &["0", "false", "false"]]),
        ("intLocal", &[&["7"], &["0"]]),
    ]),
    quotes: &[],
    // The positive comparison's inputs are the original's own values too, and the driver prints every
    // member's own answers for them: the two members a `boolean` parameter and a `boolean` field are
    // read from, and the members whose local's own declaration types its later uses.
    baseline: Some(Baseline {
        class: "Baseline",
        source: include_str!("fixtures/p3-boolean-contexts/Baseline.java"),
        lines: &[
            "isZero(0)=true",
            "isZero(1)=false",
            "isZero(7)=false",
            "isZero(-1)=false",
            "flag()=true",
            "parity(0)=1",
            "parity(1)=1",
            "passed(true)=true",
            "passed(false)=false",
            "callFlag()=true",
            "fieldFlag()=true",
            "localFromCall()=true",
            "pick(7, true, true)=true",
            "pick(0, false, false)=false",
            "fromLocal(true)=true",
            "fromLocal(false)=false",
            "assignFromCall(true)=true",
            "assignFromCall(false)=true",
            "throughLocal(true)=true",
            "throughLocal(false)=false",
            "staticFlagCount()=1",
            "intLocal(7)=1",
            "intLocal(0)=0",
            "count(true)=1",
            "count(false)=0",
            "nonzero(0)=0",
            "nonzero(1)=1",
            "answer()=1",
        ],
    }),
    members: &[
        Member {
            name: "isZero",
            expect: Expect::Executed,
        },
        Member {
            name: "flag",
            expect: Expect::Executed,
        },
        Member {
            name: "parity",
            expect: Expect::Executed,
        },
        Member {
            name: "passed",
            expect: Expect::Executed,
        },
        Member {
            name: "callFlag",
            expect: Expect::Executed,
        },
        Member {
            name: "fieldFlag",
            expect: Expect::Executed,
        },
        Member {
            name: "localFromCall",
            expect: Expect::Executed,
        },
        Member {
            name: "pick",
            expect: Expect::Executed,
        },
        Member {
            name: "fromLocal",
            expect: Expect::Executed,
        },
        Member {
            name: "assignFromCall",
            expect: Expect::Executed,
        },
        Member {
            name: "throughLocal",
            expect: Expect::Executed,
        },
        Member {
            name: "staticFlagCount",
            expect: Expect::Executed,
        },
        Member {
            name: "intLocal",
            expect: Expect::Executed,
        },
        Member {
            name: "count",
            expect: Expect::Executed,
        },
        Member {
            name: "nonzero",
            expect: Expect::Executed,
        },
        Member {
            name: "answer",
            expect: Expect::Executed,
        },
        // The one refusal of this sample: `negated`'s value is a stack merge out of two constant
        // pushes that no name denotes, and neither the boolean items nor a local's declaration
        // speaks for it.
        Member {
            name: "negated",
            expect: Expect::Quoted(None),
        },
    ],
    point: "the descriptor types a boolean context, and a local's own declaration states it too: a \
            `Z` method's `return` is written `true`/`false` (`isZero`, `flag`), a condition whose \
            operand is a proven boolean is written as a truth test (`parity`'s `if (flag())`, \
            `staticFlagCount`'s `if (staticFlag)`), and a local filled from a proven boolean value is \
            declared `boolean` with its later uses printing as that boolean (`throughLocal`, \
            `localFromCall`, `pick`, `fromLocal`, `assignFromCall`) — so every one of these bodies \
            compiles under a declaration derived from the run's own facts and answers what the \
            original answers, where the pre-fix text was `return 1;`/`return 0;`, \
            `if (flag() != 0)` and `int local1 = arg0; return local1;`, which javac refuses (`int \
            cannot be converted to boolean`, `incomparable types: boolean and int`, `boolean cannot \
            be converted to int`) — while the int-shaped controls (`nonzero`, `answer`, `count`, \
            `intLocal`, whose `0`/`1` stores stay `int`) keep the text they had",
};

const INT_COMPARISONS: Sample = Sample {
    label: "p3-int-comparisons/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "IntComparisons",
    bytes: include_bytes!("fixtures/p3-int-comparisons/v8/IntComparisons.class"),
    classpath: &[],
    // No member of this sample calls another: the comparison operands are parameters and literals,
    // and the boolean controls read a parameter, so nothing has to resolve to the committed class.
    extends: None,
    scaffold: &[],
    counter: None,
    measured: &[],
    // The comparisons' input set: `0`, `1`, `2` and `-1` reach both arms of every shape — the
    // constant-on-the-left forms (`1 == n`, `0 < n`, `1 < n`), the constant-on-the-right forms
    // (`n == 1`, `n > 0`) and the variable comparison (`n != 0`) — so no member is proved right by
    // a single comparison direction. `isZero` states the predecessor change's own inputs.
    inputs: Some(&[
        ("oneFirst", &[&["0"], &["1"], &["2"], &["-1"]]),
        ("zeroFirst", &[&["0"], &["1"], &["2"], &["-1"]]),
        ("oneLess", &[&["0"], &["1"], &["2"], &["-1"]]),
        ("oneLast", &[&["0"], &["1"], &["2"], &["-1"]]),
        ("zeroLast", &[&["0"], &["1"], &["2"], &["-1"]]),
        ("nonzero", &[&["0"], &["1"], &["2"], &["-1"]]),
        ("isZero", &[&["0"], &["1"], &["7"], &["-1"]]),
    ]),
    quotes: &[],
    // The original's own answers for those inputs, executed by the committed driver: both sides of
    // the comparison return the values the bytecode returns, which is what makes "the text
    // compiles" also mean "the text answers the same".
    baseline: Some(Baseline {
        class: "Baseline",
        source: include_str!("fixtures/p3-int-comparisons/Baseline.java"),
        lines: &[
            "oneFirst(0)=4",
            "oneFirst(1)=3",
            "oneFirst(2)=4",
            "oneFirst(-1)=4",
            "zeroFirst(0)=4",
            "zeroFirst(1)=3",
            "zeroFirst(2)=3",
            "zeroFirst(-1)=4",
            "oneLess(0)=4",
            "oneLess(1)=4",
            "oneLess(2)=3",
            "oneLess(-1)=4",
            "oneLast(0)=4",
            "oneLast(1)=3",
            "oneLast(2)=4",
            "oneLast(-1)=4",
            "zeroLast(0)=4",
            "zeroLast(1)=3",
            "zeroLast(2)=3",
            "zeroLast(-1)=4",
            "nonzero(0)=4",
            "nonzero(1)=3",
            "nonzero(2)=3",
            "nonzero(-1)=3",
            "isZero(0)=true",
            "isZero(1)=false",
            "isZero(7)=false",
            "isZero(-1)=false",
            "count(true)=1",
            "count(false)=0",
            "throughLocal(true)=true",
            "throughLocal(false)=false",
        ],
    }),
    members: &[
        Member {
            name: "oneFirst",
            expect: Expect::Executed,
        },
        Member {
            name: "zeroFirst",
            expect: Expect::Executed,
        },
        Member {
            name: "oneLess",
            expect: Expect::Executed,
        },
        Member {
            name: "oneLast",
            expect: Expect::Executed,
        },
        Member {
            name: "zeroLast",
            expect: Expect::Executed,
        },
        Member {
            name: "nonzero",
            expect: Expect::Executed,
        },
        Member {
            name: "isZero",
            expect: Expect::Executed,
        },
        Member {
            name: "count",
            expect: Expect::Executed,
        },
        Member {
            name: "throughLocal",
            expect: Expect::Executed,
        },
    ],
    point: "an integer binary comparison keeps the spelling its operands' own evidence states: \
            `1 == n`, `0 < n` and `1 < n` are written `if (1 == arg0)`, `if (0 < arg0)` and \
            `if (1 < arg0)` — where `5a8c36a` spelled the left `0`/`1` as a boolean because the \
            literal is one item of the boolean proof, producing `if (true == arg0)`, \
            `if (false < arg0)` and `if (true < arg0)`, which javac refuses (`incomparable types: \
            boolean and int`, `bad operand types for binary operator '<'`) — the same rule holds \
            with the constant on the right (`if (arg0 == 1)`, `if (arg0 > 0)`) and on the variable \
            control (`if (arg0 != 0)`), so the comparison's boolean *result* is not read as \
            evidence about its operands; the predecessor change's shapes in the same bytes are the \
            control that must not move (`isZero`'s `return true;`/`return false;`, `count`'s \
            `if (arg0)`, `throughLocal`'s `boolean local1 = arg0;`)",
};

const ARRAY_TYPES: Sample = Sample {
    label: "p3-array-types/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "ArrayTypes",
    bytes: include_bytes!("fixtures/p3-array-types/v8/ArrayTypes.class"),
    classpath: &[],
    // `echoed` calls the sample's own `copy`, so the scratch class extends the sample (which is not
    // `final`) and that name resolves to the committed class's member.
    extends: Some("ArrayTypes"),
    scaffold: &[],
    counter: None,
    measured: &[],
    // Every array parameter's default value is `null` (`sample_values`' fallback), which both sides
    // answer as `null`: the declaration is what this sample is about, and a `byte[]` argument is
    // stated here so that the one array whose result a trace can compare (`show` states a `byte[]`
    // by its class, where an `Object[]` would print a hash) is executed with a real value.
    inputs: Some(&[("echoed", &[&["new byte[] {1, 2}"]])]),
    quotes: &[],
    // The original class's own answers for that same input set, executed by the committed driver:
    // the positive comparison's side of the finding, measured rather than assumed.
    baseline: Some(Baseline {
        class: "Baseline",
        source: include_str!("fixtures/p3-array-types/Baseline.java"),
        lines: &[
            "copy(null)=null",
            "echoed({1, 2})=[1, 2]",
            "named(null)=null",
            "grid(null)=null",
            "table(null)=null",
            "text(\"r\")=r",
            "text(null)=null",
        ],
    }),
    members: &[
        Member {
            name: "copy",
            expect: Expect::Executed,
        },
        Member {
            name: "echoed",
            expect: Expect::Executed,
        },
        Member {
            name: "named",
            expect: Expect::Executed,
        },
        Member {
            name: "grid",
            expect: Expect::Executed,
        },
        Member {
            name: "table",
            expect: Expect::Executed,
        },
        Member {
            name: "text",
            expect: Expect::Executed,
        },
    ],
    point: "a type position is spelled as a Java type: a local declared from an array-typed parameter \
            or from a call returning one is `byte[]`, `java.lang.String[]`, `int[][]` and \
            `java.lang.String[][]` under the declaration this file derives from the run's own \
            descriptor — where the pre-fix text published the frames' own `[B local1 = copy(arg0);`, \
            `[Ljava.lang.String; local1 = arg0;`, `[[I local1 = arg0;` and \
            `[[Ljava.lang.String; local1 = arg0;`, which javac refuses (`illegal start of \
            expression`) — while the object-type control (`text`) keeps the declaration it always \
            had",
};

const HOISTED_BOOLEAN: Sample = Sample {
    label: "p3-hoisted-boolean/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "HoistedBoolean",
    bytes: include_bytes!("fixtures/p3-hoisted-boolean/v8/HoistedBoolean.class"),
    classpath: &[],
    // No member of this sample calls another: every value comes from a parameter or from a local of
    // the body itself, so nothing has to resolve to the committed class.
    extends: None,
    scaffold: &[],
    counter: None,
    measured: &[],
    // `copied` and `swapped` are called with both values of `b` and with `n` on either side of their
    // branch, so both writes of the local the review's finding is about are executed on each side;
    // `relayed` runs the copy chain, and the controls keep their own inputs.
    inputs: Some(&[
        (
            "copied",
            &[
                &["true", "0"],
                &["false", "0"],
                &["true", "7"],
                &["false", "-1"],
            ],
        ),
        (
            "swapped",
            &[
                &["true", "0"],
                &["false", "0"],
                &["true", "7"],
                &["false", "-1"],
            ],
        ),
        ("relayed", &[&["true"], &["false"]]),
        ("literalArmed", &[&["true"], &["false"]]),
        (
            "fromParameter",
            &[&["true", "0"], &["false", "7"], &["true", "-1"]],
        ),
        ("intLocal", &[&["7"], &["0"], &["-1"]]),
    ]),
    quotes: &[],
    // The original's own answers for those inputs, executed by the committed driver: the two
    // members the finding is about answer what the bytecode answers, which is what makes "the text
    // compiles under the member's own declaration" also mean "and it returns the same values".
    baseline: Some(Baseline {
        class: "Baseline",
        source: include_str!("fixtures/p3-hoisted-boolean/Baseline.java"),
        lines: &[
            "copied(true, 0)=1",
            "copied(false, 0)=0",
            "copied(true, 7)=1",
            "copied(false, -1)=0",
            "swapped(true, 0)=1",
            "swapped(false, 0)=0",
            "swapped(true, 7)=1",
            "swapped(false, -1)=0",
            "relayed(true)=true",
            "relayed(false)=false",
            "literalArmed(true)=1",
            "literalArmed(false)=0",
            "fromParameter(true, 0)=true",
            "fromParameter(false, 7)=false",
            "fromParameter(true, -1)=true",
            "intLocal(7)=7",
            "intLocal(0)=1",
            "intLocal(-1)=-1",
        ],
    }),
    members: &[
        Member {
            name: "copied",
            expect: Expect::Executed,
        },
        Member {
            name: "swapped",
            expect: Expect::Executed,
        },
        Member {
            name: "relayed",
            expect: Expect::Executed,
        },
        Member {
            name: "literalArmed",
            expect: Expect::Executed,
        },
        Member {
            name: "fromParameter",
            expect: Expect::Executed,
        },
        Member {
            name: "intLocal",
            expect: Expect::Executed,
        },
        // The two refusals of this sample: `unproven`'s only writes are `true`/`false` literals
        // returned from a `Z` method (the position requires a boolean the evidence does not have),
        // and `conflicted` writes a descriptor-proven boolean into the local its own first write
        // decided is an `int`.
        Member {
            name: "unproven",
            expect: Expect::Quoted(None),
        },
        Member {
            name: "conflicted",
            expect: Expect::Quoted(None),
        },
    ],
    point: "a local's type is decided once, before its statements are built: the type the plan \
            decides for a variable is what its hoisted declaration, its in-place declaration and \
            every use are written with, so `copied` — the review's shape, whose local `c` is filled \
            in one arm from a local the body declared `boolean` and in the other from a `boolean` \
            parameter — is `boolean local3; … local3 = local2; … local3 = arg0; if (local3) …` and \
            compiles under the member's own declaration, where the pre-fix text was `int local3; … \
            local3 = local2; … local3 = arg0; … if (local3 != 0)` and javac refused it (`boolean \
            cannot be converted to int`); its twin `swapped`, which differs only in which arm runs \
            first, gets the same type, the same declaration and the same uses; the copy chain \
            `relayed` (a read of a local whose own decision is one hop away) keeps `boolean` on \
            every variable and compiles where the pre-fix text refused it; the literal-armed \
            boundary and the pure-`int` control keep the text they had (`int`); and a write that \
            cannot be spelled as the type its variable's own first write decided (`conflicted`) or \
            a value the evidence does not have in a position that requires one (`unproven`) is \
            refused with the bytecode quoted instead of being published",
};

const CONCAT_CONVERSION: Sample = Sample {
    label: "p3-concat-conversion/v8 (javac 23.0.1, --release 8 -g:none)",
    class: "ConcatConversion",
    bytes: include_bytes!("fixtures/p3-concat-conversion/v8/ConcatConversion.class"),
    classpath: &[],
    // The observable members call the sample's own helpers (`markA`, `markB`, `div`), and the
    // counter the trace prints belongs to the sample's own class: extending the sample resolves
    // those names to the committed bodies, so the generated side calls the *same* helpers the
    // original does and the two counters are one field.
    extends: Some("ConcatConversion"),
    scaffold: &[],
    counter: Some("ConcatConversion.calls"),
    measured: &[],
    // The finding's own inputs come first: `(1, 2)` is the pair the class answers `"12!"` for and the
    // pre-fix text answered `"3!"` for. The controls are called with the values that separate a
    // converted part from an unconverted one (`null`, an empty string, a zero), and the observable
    // members are called with the input that moves the counter and with the one that throws.
    inputs: Some(&[
        (
            "twoIntsThenString",
            &[&["1", "2"], &["7", "0"], &["0", "-1"]],
        ),
        ("onePartIsASum", &[&["1", "2"], &["7", "0"]]),
        (
            "numericLast",
            &[&["\"a\"", "1"], &["\"\"", "0"], &["null", "2"]],
        ),
        (
            "allStrings",
            &[&["\"a\"", "\"b\""], &["\"\"", "\"x\""], &["null", "\"x\""]],
        ),
        ("booleanLiteral", &[&[]]),
        ("booleanParameter", &[&["true"], &["false"]]),
        ("nullPart", &[&[]]),
        ("objectPart", &[&["\"o\""], &["null"]]),
        ("marked", &[&[]]),
        ("failing", &[&["2"], &["0"]]),
    ]),
    quotes: &[],
    baseline: Some(Baseline {
        class: "Baseline",
        source: include_str!("fixtures/p3-concat-conversion/Baseline.java"),
        lines: &[
            "twoIntsThenString(1, 2)=12!",
            "twoIntsThenString(7, 0)=70!",
            "twoIntsThenString(0, -1)=0-1!",
            "onePartIsASum(1, 2)=3!",
            "onePartIsASum(7, 0)=7!",
            "numericLast(\"a\", 1)=a1",
            "numericLast(\"\", 0)=0",
            "numericLast(null, 2)=null2",
            "allStrings(\"a\", \"b\")=ab",
            "allStrings(\"\", \"x\")=x",
            "allStrings(null, \"x\")=nullx",
            "booleanLiteral()=true!",
            "booleanParameter(true)=true!",
            "booleanParameter(false)=false!",
            "nullPart()=null!",
            "objectPart(\"o\")=o!",
            "objectPart(null)=null!",
            "marked()=12! calls=0->2",
            "failing(2)=150! calls=2->3",
            "failing(0)=java.lang.ArithmeticException: / by zero calls=3->4",
            "markA()=1 markB()=2 calls=6",
            "div(7, 7)=1 div(-1, -1)=1",
        ],
    }),
    members: &[
        Member {
            name: "twoIntsThenString",
            expect: Expect::Executed,
        },
        Member {
            name: "onePartIsASum",
            expect: Expect::Executed,
        },
        Member {
            name: "numericLast",
            expect: Expect::Executed,
        },
        Member {
            name: "allStrings",
            expect: Expect::Executed,
        },
        Member {
            name: "booleanLiteral",
            expect: Expect::Executed,
        },
        Member {
            name: "booleanParameter",
            expect: Expect::Executed,
        },
        Member {
            name: "nullPart",
            expect: Expect::Executed,
        },
        Member {
            name: "objectPart",
            expect: Expect::Executed,
        },
        Member {
            name: "marked",
            expect: Expect::Executed,
        },
        Member {
            name: "failing",
            expect: Expect::Executed,
        },
        Member {
            name: "markA",
            expect: Expect::Executed,
        },
        Member {
            name: "markB",
            expect: Expect::Executed,
        },
        Member {
            name: "div",
            expect: Expect::Executed,
        },
    ],
    point: "a presented concatenation keeps each `append`'s own conversion, in the part that \
            performs it (`concat@1`, T4): `twoIntsThenString(1, 2)` is `\"\" + arg0 + arg1 + \"!\"` \
            and answers `\"12!\"` where the pre-fix text `arg0 + arg1 + \"!\"` answered `\"3!\"`; \
            `onePartIsASum(1, 2)` is `\"\" + (arg0 + arg1) + \"!\"` and answers the class's `\"3!\"` \
            — the two members are the same *text* before the fix and two different programs after \
            it; `booleanLiteral()` is `\"\" + true + \"!\"` where the pre-fix `1 + \"!\"` answered \
            `\"1!\"`; `nullPart()` and `objectPart` keep `String.valueOf`'s conversion; `marked()` \
            shows the two parts evaluated once each in the bytecode's order (its counter moves twice \
            and its value is `\"12!\"`); `failing(0)` shows the observable part *before* the throwing \
            one running and no later part running (the counter moves once and the trace carries the \
            `ArithmeticException`); and the two controls — `numericLast` and `allStrings` — keep \
            their text byte for byte",
};

const REQUIRED: &[&Sample] = &[
    &LOCAL_REWRITE,
    &SCOPE_NO_DEBUG,
    &SCOPE_DEBUG,
    &GUARDED,
    &ECJ_V52,
    &NESTED_EVAL,
    &REFUSED_CAST,
    &NESTED_ARITHMETIC,
    &RECEIVER_GROUPING,
    &BOOLEAN_CONTEXTS,
    &INT_COMPARISONS,
    &ARRAY_TYPES,
    &HOISTED_BOOLEAN,
    &CONCAT_CONVERSION,
];

const CORPUS: &[&Sample] = &[
    &FLAGS_G_NONE,
    &FLAGS_G,
    &FLAGS_G_LINES,
    &FLAGS_PARAMETERS,
    &MISSING_DEPENDENCY,
];

// -------------------------------------------------------------------------------------------
// Descriptors: the wrapper's declaration, and the inputs each parameter type is called with.
// -------------------------------------------------------------------------------------------

/// The Java spelling of one descriptor type, and whether it is a primitive.
fn java_type(descriptor: &str) -> Option<(String, bool)> {
    let mut rest = descriptor;
    let mut dimensions = 0usize;
    while let Some(tail) = rest.strip_prefix('[') {
        dimensions += 1;
        rest = tail;
    }
    let (mut spelling, primitive) = match rest.chars().next()? {
        'Z' => ("boolean".to_string(), true),
        'B' => ("byte".to_string(), true),
        'C' => ("char".to_string(), true),
        'S' => ("short".to_string(), true),
        'I' => ("int".to_string(), true),
        'J' => ("long".to_string(), true),
        'F' => ("float".to_string(), true),
        'D' => ("double".to_string(), true),
        'V' if dimensions == 0 => ("void".to_string(), true),
        'L' => {
            let name = rest.strip_prefix('L')?.strip_suffix(';')?;
            (name.replace('/', "."), false)
        }
        _ => return None,
    };
    for _ in 0..dimensions {
        spelling.push_str("[]");
    }
    Some((spelling, primitive))
}

/// One descriptor type as it is written, and the slot it starts at.
fn take_type(characters: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<(String, bool)> {
    let mut text = String::new();
    while let Some(character) = characters.next() {
        text.push(character);
        match character {
            '[' => continue,
            'L' => {
                for next in characters.by_ref() {
                    text.push(next);
                    if next == ';' {
                        break;
                    }
                }
            }
            _ => {}
        }
        break;
    }
    java_type(&text)
}

/// A descriptor type as this file reads it: the Java spelling, and whether it is a primitive.
type Spelling = (String, bool);
/// A parameter as the descriptor states it: its slot, its spelling, and whether it is a primitive.
type SlotType = (u16, String, bool);

/// A descriptor's parameter list and return type, as `(slot, Java type, primitive)` per parameter.
fn descriptor_parts(descriptor: &str) -> Option<(Vec<SlotType>, Spelling)> {
    let rest = descriptor.strip_prefix('(')?;
    let (parameters, returns) = rest.split_once(')')?;
    let mut characters = parameters.chars().peekable();
    let mut slots = Vec::new();
    let mut slot = 0u16;
    while characters.peek().is_some() {
        let (spelling, primitive) = take_type(&mut characters)?;
        let wide = spelling == "long" || spelling == "double";
        slots.push((slot, spelling, primitive));
        slot = slot.saturating_add(if wide { 2 } else { 1 });
    }
    Some((slots, java_type(returns)?))
}

/// The inputs one parameter is called with, by its Java type. Which values these are is this
/// file's choice; that both sides are called with the same ones is not, and every value is printed
/// into the trace so that a line states which input made which observation.
fn sample_values(spelling: &str) -> Vec<String> {
    match spelling {
        "int" | "short" | "byte" => vec!["7".to_string(), "0".to_string(), "-1".to_string()],
        "char" => vec!["'a'".to_string()],
        "long" => vec!["5L".to_string(), "0L".to_string(), "-1L".to_string()],
        "float" => vec!["1.5f".to_string()],
        "double" => vec!["1.5".to_string()],
        "boolean" => vec!["true".to_string(), "false".to_string()],
        "java.lang.String" => vec!["\"r\"".to_string(), "null".to_string()],
        _ => vec!["null".to_string()],
    }
}

/// The expression a witness body returns, so that a declaration can be compiled **without** the
/// recovered text: a declaration that does not compile on its own would otherwise be recorded as a
/// boundary of the text, which is exactly the confusion this file must not have.
fn default_value(spelling: &str) -> Option<&'static str> {
    match spelling {
        "void" => None,
        "boolean" => Some("false"),
        "long" => Some("0L"),
        "float" => Some("0f"),
        "double" => Some("0d"),
        "int" | "short" | "byte" | "char" => Some("0"),
        _ => Some("null"),
    }
}

fn is_java_identifier(name: &str) -> bool {
    !name.is_empty()
        && !name
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_digit())
        && name
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_' || character == '$')
}

/// The one name the run's debug records state for a slot, when they state exactly one.
///
/// A slot the table names over two ranges with two names carried two source variables (P3 3.4), and
/// neither of them is the name of the whole slot: the wrapper this comparison builds declares such a
/// slot the way it declares a slot no record covers.
fn stated_name(facts: &RecoveryFacts, slot: u16) -> Option<String> {
    let mut names: Vec<&str> = facts
        .debug_locals()
        .iter()
        .filter(|record| record.slot() == slot)
        .map(|record| record.name())
        .collect();
    names.sort_unstable();
    names.dedup();
    match names[..] {
        [only] => Some(only.to_owned()),
        _ => None,
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '_' || character == '$' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

/// The declaration one member's wrapper is written as, from the run's own facts: the flags say
/// whether it is `public` and whether it is `static`, [`MethodFacts::parameter_types`] says what
/// each parameter slot holds (P3-R5's fact), the slot's own debug name says what it is called, and
/// the descriptor says what the method returns.
fn declaration_of(
    facts: &MethodFacts,
    parameters: &[Parameter],
    return_type: &str,
    return_override: Option<&str>,
) -> String {
    let flags = facts
        .access_flags()
        .expect("the run stated the member's flags");
    let mut declaration = String::new();
    if flags & ACC_PUBLIC != 0 {
        declaration.push_str("public ");
    }
    if flags & ACC_STATIC != 0 {
        declaration.push_str("static ");
    }
    declaration.push_str(return_override.unwrap_or(return_type));
    declaration.push(' ');
    declaration.push_str(facts.name());
    declaration.push('(');
    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            declaration.push_str(", ");
        }
        declaration.push_str(&parameter.spelling);
        declaration.push(' ');
        declaration.push_str(&parameter.name);
    }
    declaration.push(')');
    declaration
}

// -------------------------------------------------------------------------------------------
// javac and java: the two commands the comparison is made of.
// -------------------------------------------------------------------------------------------

/// The JDK's `javac` at Java 8, with its messages forced to English so that a recorded refusal is
/// the same string on every machine. A failing compile returns the message rather than raising:
/// whether a boundary is expected is decided by the caller.
fn javac(dir: &Path, files: &[&str]) -> std::result::Result<(), String> {
    let output = Command::new("javac")
        .current_dir(dir)
        .args([
            "--release",
            "8",
            "-g:none",
            "-J-Duser.language=en",
            "-J-Duser.country=US",
            "-cp",
            ".",
            "-d",
            ".",
        ])
        .args(files)
        .output()
        .expect("execute javac from PATH");
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

fn java(dir: &Path, class: &str) -> std::process::Output {
    Command::new("java")
        .current_dir(dir)
        .args(["-cp", ".", class])
        .output()
        .expect("execute java from PATH")
}

/// The scratch class of one member: the sample's extension, the scaffold a body may name, and the
/// member's own wrapper. One member per class, because a text that does not compile must be the
/// only thing that fails when it does.
fn wrapper_source(
    sample: &Sample,
    member_name: &str,
    descriptor: &str,
    declaration: &str,
    body: &str,
    scratch: &str,
) -> String {
    let mut source = String::new();
    source.push_str("import java.util.Arrays;\n");
    source.push_str(&format!("public class {scratch}"));
    if let Some(parent) = sample.extends {
        source.push_str(&format!(" extends {parent}"));
    }
    source.push_str(" {\n");
    for scaffold in sample.scaffold {
        if scaffold.name == member_name && scaffold.descriptor == descriptor {
            continue;
        }
        source.push_str("    ");
        source.push_str(scaffold.java);
        source.push('\n');
    }
    source.push_str("    ");
    source.push_str(declaration);
    source.push(' ');
    source.push_str(body);
    source.push('\n');
    source.push_str("}\n");
    source
}

/// Compiles one wrapper, and answers with javac's refusal when it does not compile.
fn compile_wrapper(
    dir: &Path,
    sample: &Sample,
    member_name: &str,
    descriptor: &str,
    declaration: &str,
    body: &str,
) -> std::result::Result<String, String> {
    let scratch = format!("Gen{}", sanitize(member_name));
    let source = wrapper_source(sample, member_name, descriptor, declaration, body, &scratch);
    let file = format!("{scratch}.java");
    fs::write(dir.join(&file), source).expect("write the wrapper");
    let compiled = javac(dir, &[&file]).map(|()| scratch);
    // The source goes away whatever happened: a text javac refused must not be left where a later
    // `javac` run would find it on the source path and compile it again (the runner's own compile
    // would then fail for a member this comparison already recorded as a boundary).
    let _ = fs::remove_file(dir.join(&file));
    compiled
}

// -------------------------------------------------------------------------------------------
// The runner: one generated Java class per side, printing the same trace for the same inputs.
// -------------------------------------------------------------------------------------------

/// One call of one member, in both sides' source: the arguments' own text is the trace's label and
/// the call's expression at once.
struct Call {
    label: String,
    arguments: String,
}

/// One member's calls: the sample's own list when it states one for this member, and the default
/// input set otherwise.
fn calls_for(sample: &Sample, member: &str, parameters: &[Parameter]) -> Vec<Call> {
    if let Some(inputs) = sample.inputs
        && let Some((_, listed)) = inputs.iter().find(|(name, _)| *name == member)
    {
        return listed
            .iter()
            .map(|arguments| Call {
                label: format!("[{}]", arguments.join(", ")),
                arguments: arguments.join(", "),
            })
            .collect();
    }
    let values: Vec<Vec<String>> = parameters
        .iter()
        .map(|parameter| sample_values(&parameter.spelling))
        .collect();
    let count = values.iter().map(Vec::len).max().unwrap_or(1).max(1);
    (0..count)
        .map(|index| {
            let chosen: Vec<String> = values
                .iter()
                .map(|list| list[index % list.len()].clone())
                .collect();
            Call {
                label: format!("[{}]", chosen.join(", ")),
                arguments: chosen.join(", "),
            }
        })
        .collect()
}

/// The two helpers every generated runner carries: how a value is written into the trace (a value
/// that would print a hash is stated as its class instead, so two JVMs can still be compared), and
/// how a throwable is written (its class, its message, and its suppressed exceptions).
const TRACE_HELPERS: &str = r#"    static String show(Object value) {
        if (value == null) {
            return "null";
        }
        if (value instanceof String) {
            return "\"" + value + "\"";
        }
        if (value instanceof Number || value instanceof Boolean || value instanceof Character
            || value instanceof Object[]) {
            return String.valueOf(value);
        }
        return "class=" + value.getClass().getName();
    }

    static String describe(Throwable thrown) {
        StringBuilder out = new StringBuilder();
        describe(thrown, out, 0);
        return out.toString();
    }

    static void describe(Throwable thrown, StringBuilder out, int depth) {
        if (thrown == null) {
            out.append("null");
            return;
        }
        if (depth > 0) {
            out.append("suppressed ");
        }
        out.append(thrown.getClass().getName()).append(": ").append(thrown.getMessage());
        for (Throwable suppressed : thrown.getSuppressed()) {
            out.append(" | ");
            describe(suppressed, out, depth + 1);
        }
    }
"#;

/// One call's label as it can stand **inside** the trace's string literal.
///
/// A label is the argument list as Java source (`["r"]` for a `String`), and the trace prints it
/// inside a `"…"` literal of its own: the quotes a `String` argument brings have to be escaped, or
/// the runner does not compile for a member whose parameter is a `String`. Both sides print the same
/// escaped label, so the comparison is unchanged — only its ability to run such a member is.
fn trace_label(label: &str) -> String {
    label.replace('\\', "\\\\").replace('"', "\\\"")
}

/// One member's rows in the runner: the value row (this member's own behaviour) and, when the run
/// refused the body's value, the count row (how many times the text it did write calls something).
fn runner_rows(
    sample: &Sample,
    plan: &Planned,
    executed: bool,
    counted: bool,
    target: &str,
) -> String {
    let mut rows = String::new();
    let calls = calls_for(sample, &plan.name, &plan.parameters);
    let counter_before = match sample.counter {
        Some(counter) => format!("long before = {counter};\n            "),
        None => String::new(),
    };
    let counter_after = match sample.counter {
        Some(counter) => format!(" calls \" + before + \"->\" + {counter} + \""),
        None => String::new(),
    };
    for call in &calls {
        if executed {
            // The call happens **before** the line is printed, into a variable of the declared
            // return type: a counter read as an argument of the same `println` as the call would be
            // a count taken *before* the call, which is not a count of anything.
            let invocation = format!("{target}({})", call.arguments);
            let label = trace_label(&call.label);
            if plan.return_type == "void" {
                // A `void` member has no value to show: the trace states `void`, which is what the
                // original's own declaration states too.
                rows.push_str(&format!(
                    "        {{\n            {counter_before}try {{\n                {invocation};\n                \
                     System.out.println(\"value {}{} {}{counter_after} -> void\");\n            \
                     }} catch (Throwable thrown) {{\n                \
                     System.out.println(\"value {}{} {}{counter_after} -> throws \" + describe(thrown));\n            \
                     }}\n        }}\n",
                    plan.name, plan.descriptor, label,
                    plan.name, plan.descriptor, label,
                ));
            } else {
                rows.push_str(&format!(
                    "        {{\n            {counter_before}try {{\n                \
                     {} result = {invocation};\n                \
                     System.out.println(\"value {}{} {}{counter_after} -> \" + show(result));\n            \
                     }} catch (Throwable thrown) {{\n                \
                     System.out.println(\"value {}{} {}{counter_after} -> throws \" + describe(thrown));\n            \
                     }}\n        }}\n",
                    plan.return_type,
                    plan.name, plan.descriptor, label,
                    plan.name, plan.descriptor, label,
                ));
            }
        } else if counted {
            rows.push_str(&format!(
                "        {{\n            {counter_before}String outcome = \"threw=none\";\n            \
                 try {{\n                {target}({});\n            }} catch (Throwable thrown) {{\n                \
                 outcome = \"threw=\" + describe(thrown);\n            }}\n            \
                 System.out.println(\"count {}{} {}{counter_after} \" + outcome);\n        }}\n",
                call.arguments, plan.name, plan.descriptor, trace_label(&call.label),
            ));
        }
    }
    rows
}

/// One side's runner: the same rows, against either the committed sample or the scratch classes.
fn runner_source(rows: &str, class_name: &str) -> String {
    format!(
        "public class {class_name} {{\n    public static void main(String[] args) {{\n{rows}    }}\n\n\
         {TRACE_HELPERS}}}\n"
    )
}

/// The whole comparison of one sample: compile both runners, run both, and compare.
fn compare_traces(
    sample: &Sample,
    dir: &Path,
    plans: &[Planned],
    executed: &[String],
    counted: &[String],
) -> (usize, bool, String) {
    let mut original = String::new();
    let mut generated = String::new();
    for plan in plans {
        let is_executed = executed.iter().any(|name| name == &plan.name);
        let is_counted = counted.iter().any(|name| name == &plan.name);
        if !is_executed && !is_counted {
            continue;
        }
        let instance = !plan
            .declaration
            .split_whitespace()
            .any(|word| word == "static");
        let original_target = if instance {
            format!("new {}().{}", sample.class, plan.name)
        } else {
            format!("{}.{}", sample.class, plan.name)
        };
        let generated_target = if instance {
            format!("new {}().{}", plan.scratch, plan.name)
        } else {
            format!("{}.{}", plan.scratch, plan.name)
        };
        original.push_str(&runner_rows(
            sample,
            plan,
            is_executed,
            is_counted,
            &original_target,
        ));
        generated.push_str(&runner_rows(
            sample,
            plan,
            is_executed,
            is_counted,
            &generated_target,
        ));
    }
    fs::write(
        dir.join("OriginalRunner.java"),
        runner_source(&original, "OriginalRunner"),
    )
    .expect("write the original runner");
    fs::write(
        dir.join("GeneratedRunner.java"),
        runner_source(&generated, "GeneratedRunner"),
    )
    .expect("write the generated runner");
    javac(dir, &["OriginalRunner.java", "GeneratedRunner.java"]).unwrap_or_else(|message| {
        panic!(
            "{}: both runners compile - a declaration this file derived from the run's facts is \
             wrong when one of them does not:\n{message}",
            sample.label
        )
    });
    let original_output = java(dir, "OriginalRunner");
    let generated_output = java(dir, "GeneratedRunner");
    let original_text = String::from_utf8_lossy(&original_output.stdout).into_owned();
    let generated_text = String::from_utf8_lossy(&generated_output.stdout).into_owned();
    assert!(
        original_output.status.success(),
        "{}: the original side ran:\n{}",
        sample.label,
        String::from_utf8_lossy(&original_output.stderr)
    );
    assert!(
        generated_output.status.success(),
        "{}: the generated side ran:\n{}",
        sample.label,
        String::from_utf8_lossy(&generated_output.stderr)
    );
    let identical = original_text == generated_text;
    let observed = format!("--- original ---\n{original_text}--- generated ---\n{generated_text}");
    assert!(
        identical,
        "{}: the two bodies' observable traces differ:\n{}",
        sample.label,
        first_difference(&original_text, &generated_text)
    );
    (original_text.lines().count(), identical, observed)
}

fn first_difference(original: &str, generated: &str) -> String {
    let left: Vec<&str> = original.lines().collect();
    let right: Vec<&str> = generated.lines().collect();
    for index in 0..left.len().max(right.len()) {
        let ours = left.get(index).copied().unwrap_or("<no line>");
        let theirs = right.get(index).copied().unwrap_or("<no line>");
        if ours != theirs {
            return format!(
                "line {}:\n  original  {ours}\n  generated {theirs}",
                index + 1
            );
        }
    }
    "the traces have the same lines".to_string()
}

// -------------------------------------------------------------------------------------------
// One member's plan: what the run said, the declaration this file derived from it, and what javac
// did with the two.
// -------------------------------------------------------------------------------------------

struct Parameter {
    /// The slot the parameter occupies, as the run's own facts number slots (`this` is 0 for an
    /// instance method).
    slot: u16,
    /// The Java type the wrapper declares it with.
    spelling: String,
    /// The name the wrapper declares it with: the class file's own debug name when it states one,
    /// and the ordinal the reader invents otherwise.
    name: String,
}

struct Planned {
    name: String,
    descriptor: String,
    parameters: Vec<Parameter>,
    return_type: String,
    /// The declaration without a body: `public static int post(int arg0)`.
    declaration: String,
    represent: Representation,
    quality: Quality,
    /// What the delivered artifact holds, as the report's own content plane states it.
    content: RecoveryContent,
    /// The artifact's own text, as the run delivered it: the value this file wraps, compiles and
    /// compares, kept so that two entries' answers to the same member can be compared to each other
    /// and not only to the original class.
    text: String,
    /// Whether the run delivered an artifact at all: a stopped request holds no content, and the
    /// two values are compared rather than assumed to agree.
    produced: bool,
    /// The scratch class this member's wrapper was written into.
    scratch: String,
    /// The diagnostic codes the report's refused regions state, in report order.
    codes: Vec<&'static str>,
    /// Every bytecode index the text quotes, in the order the quotes name them.
    quotes: Vec<u32>,
    /// The quoted bytecode indexes the source map does not answer for: must stay empty.
    unanchored: Vec<u32>,
    /// The bytecode indexes of every region the run refused without structuring it.
    refused_blocks: Vec<u32>,
    /// Exception-table handler entries the *run* leaves unaccounted for: no anchor of the map answers
    /// for the BCI and no quote of the text names it. Printed nowhere — `run_sample` asserts it.
    unanchored_handlers: Vec<u32>,
    /// javac's message, when the wrapper of the recovered text did not compile.
    refusal: Option<String>,
    /// Whether the count control - the same text under a `void` declaration - compiled.
    count_control: Option<std::result::Result<(), String>>,
}

impl Planned {
    /// Whether the run wrote the whole body as Java *and* javac accepted it as a method body: the
    /// only rows that are executed and compared.
    fn executed(&self) -> bool {
        self.refusal.is_none() && matches!(self.represent, Representation::Java)
    }
}

struct SampleOutcome {
    label: String,
    point: String,
    rows: Vec<Planned>,
    /// How many members the class declares, how many of them declare a body, and how many therefore
    /// are not requests at all.
    declared: usize,
    with_code: usize,
    without_code: usize,
    /// How many requests this sample performed, and how the engine's own reports classified them.
    requested: usize,
    produced: usize,
    contains_statements: usize,
    explanation_only: usize,
    stopped: usize,
    skipped: Vec<String>,
    executed: Vec<String>,
    counted: Vec<String>,
    /// What the sample's committed baseline driver printed, when it states one.
    baseline: Option<String>,
    trace_lines: usize,
    trace_identical: bool,
    trace: String,
}

// -------------------------------------------------------------------------------------------
// The two entries the bodies are read through: the single method, and the bulk operation.
//
// Everything below this point is the same for both — the wrapper, the compilation, the execution and
// the comparison — so what a sample proves about one entry is proved about the other by running the
// same sample table through both.
// -------------------------------------------------------------------------------------------

/// Which entry a run reads its bodies through.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Entry {
    /// [`Engine::recover_method`]: one request per member, the entry every earlier reading of this
    /// file was written from.
    SingleMethod,
    /// [`Engine::recover_all`]: one operation over the sample's own physical scope, whose stream
    /// hands the answer of every member over as one record. This is the entry `jarde-cli export`
    /// drives, and the text of a record is what that command writes.
    Bulk,
}

impl Entry {
    fn name(self) -> &'static str {
        match self {
            Self::SingleMethod => "recover_method",
            Self::Bulk => "recover_all",
        }
    }
}

/// The sink one sample's bulk operation is read through: it keeps the operation's own records, so
/// the comparison reads each member's answer from the record about that member.
#[derive(Default)]
struct Collected {
    header: Option<BulkHeaderEvent>,
    /// Every method record the operation delivered, in delivery order.
    records: Vec<MethodResultEvent>,
    /// Every class's own end record.
    ended: Vec<ClassEndEvent>,
    /// The operation's last event, when it reached one.
    final_event: Option<BulkFinalEvent>,
}

impl RecoverySink for Collected {
    /// The header, and with it the operation's own output account — which this sink does not use: it
    /// keeps every record in memory and writes no byte of its own anywhere, so it charges nothing
    /// against the total and only needs to see the event. A sink that owned an output destination
    /// (the `export` adapter's) is the one that stores the handle and bills its records to it.
    fn header(
        &mut self,
        event: &BulkHeaderEvent,
        _delivery: DeliveryAccount,
    ) -> jarde::Result<SinkControl> {
        self.header = Some(event.clone());
        Ok(SinkControl::Continue)
    }

    fn class_prepared(&mut self, _event: &ClassPreparedEvent) -> jarde::Result<SinkControl> {
        Ok(SinkControl::Continue)
    }

    fn method(&mut self, event: &MethodResultEvent) -> jarde::Result<SinkControl> {
        self.records.push(event.clone());
        Ok(SinkControl::Continue)
    }

    fn class_end(&mut self, event: &ClassEndEvent) -> jarde::Result<SinkControl> {
        self.ended.push(event.clone());
        Ok(SinkControl::Continue)
    }

    fn diagnostic(&mut self, _event: &BulkDiagnosticEvent) -> jarde::Result<SinkControl> {
        Ok(SinkControl::Continue)
    }

    fn final_event(&mut self, event: &BulkFinalEvent) -> jarde::Result<SinkControl> {
        self.final_event = Some(event.clone());
        Ok(SinkControl::Continue)
    }
}

/// One sample's bulk operation, as the comparison reads it: the method records it delivered.
struct BulkRead {
    records: Vec<MethodResultEvent>,
}

impl BulkRead {
    /// The presented run of one member: the operation's own record about that member.
    ///
    /// A record the operation did not present — a declaration with no body, a request refused before
    /// any analysis ran, a result the window could not hold — is not an answer this comparison can
    /// wrap, so it is a failure of the run and not a silent skip: the sample table says what each
    /// member's answer is, and only a presented run can be the one the table states.
    fn recovered(&self, label: &str, name: &str, descriptor: &str) -> &RecoveredMethod {
        let mut matching = self.records.iter().filter(|event| {
            event.method.name.0 == name.as_bytes()
                && event.method.descriptor.0 == descriptor.as_bytes()
        });
        let event = matching.next().unwrap_or_else(|| {
            panic!(
                "{label}: the bulk operation delivered no record for `{name}{descriptor}`; it \
                 delivered {:?}",
                self.records
                    .iter()
                    .map(|event| (
                        String::from_utf8_lossy(&event.method.name.0).into_owned(),
                        event.outcome(),
                    ))
                    .collect::<Vec<_>>()
            )
        });
        assert!(
            matching.next().is_none(),
            "{label}: the bulk operation delivered more than one record for `{name}{descriptor}`"
        );
        match &event.delivery {
            MethodDelivery::Recovered(recovered) => recovered,
            delivery => panic!(
                "{label}: the bulk operation's record for `{name}{descriptor}` is {:?} rather than \
                 a presented run, so the member has no text this comparison could wrap",
                delivery.outcome()
            ),
        }
    }
}

/// Runs one sample's own physical scope through [`Engine::recover_all`], once, and answers with what
/// it delivered.
///
/// The request is the shape `jarde-cli export` builds for a standalone class: the sample's own
/// snapshot, the policy one class file is read under, Java 8, the same per-method limits the
/// single-method requests use. Two workers are stated so that the class task runs on a worker of the
/// operation rather than on this test's thread.
fn bulk_read(engine: &Engine, sample: &Sample, snapshot: &ArtifactSnapshot) -> BulkRead {
    let request = BulkRecoveryRequest::for_scope(
        EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
        2,
        limits(),
    );
    let content = [snapshot.clone()];
    let mut budget = Budget::new(limits());
    let mut sink = Collected::default();
    let report = engine
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the sample's own scope is recoverable");
    assert!(
        sink.final_event.is_some() && report.final_delivered,
        "{}: the operation confirmed its `Final` record: {:?}",
        sample.label,
        report.summary
    );
    assert_eq!(
        report.summary.limits.workers_effective, 2,
        "{}: the sample's class task runs on a worker of the operation",
        sample.label
    );
    assert!(
        report.summary.traversal_complete,
        "{}: one standalone class is a scope the traversal reaches the end of: {:?}",
        sample.label, report.summary
    );
    assert_eq!(
        report.summary.methods_delivered, report.summary.methods_executed,
        "{}: every method the operation ran was delivered: {:?}",
        sample.label, report.summary
    );
    assert!(
        report.stop.is_none(),
        "{}: the operation observed no stop: {:?}",
        sample.label,
        report.stop
    );
    // The operation's own account, read back against the records this sink kept: the stream the
    // comparison is about to read is the operation's whole stream and not a prefix of it, and its
    // disposition buckets are the methods it really ran. Both readings are the report's and the
    // sink's, so this is an agreement between two accounts rather than arithmetic on one number.
    assert_eq!(
        report.summary.methods_delivered,
        u64::try_from(sink.records.len()).expect("the record count fits u64"),
        "{}: every record the operation delivered is one this comparison read: {:?}",
        sample.label,
        report.summary
    );
    assert_eq!(
        report.summary.outcomes.total(),
        report.summary.methods_executed,
        "{}: the disposition buckets are the methods the operation ran: {:?}",
        sample.label,
        report.summary
    );
    // The shape of the read the text came from, in the operation's own account: preparing the class
    // once is one parse, and a class that declares N members is not N parses. The single-method entry
    // the other tests of this file read charges its own read per request; a bulk path that went back
    // to one of those per member would hand over the same text and fail here, which is why this
    // comparison states the shape beside the text it compiled.
    assert_eq!(
        report.discovery_usage.class_bytes,
        u64::try_from(sample.bytes.len()).expect("the class length fits u64"),
        "{}: the sample's class file is parsed once for the whole operation: {:?}",
        sample.label,
        report.discovery_usage
    );
    assert_eq!(
        report.discovery_usage.class_headers, 0,
        "{}: preparation charges the one read, so the binding search charges no header: {:?}",
        sample.label, report.discovery_usage
    );
    // The stream and the report are two accounts of one operation, and the comparison reads both:
    // what the header published is the configuration the summary states, and the class the method
    // records belong to ended by publishing every one of them — so the records this comparison wraps
    // are the whole of a completed class, not the visible prefix of a stopped one.
    assert_eq!(
        sink.header.as_ref().map(|header| &header.limits),
        Some(&report.summary.limits),
        "{}: the header publishes the operation's own configuration",
        sample.label
    );
    assert_eq!(
        sink.ended.len(),
        1,
        "{}: one sample declares one class, so the stream states one class end: {:?}",
        sample.label,
        sink.ended
    );
    assert_eq!(
        sink.ended[0].completion,
        ClassCompletion::Completed {
            methods: u64::try_from(sink.records.len()).expect("the record count fits u64"),
        },
        "{}: the class ended having published every method record this comparison read",
        sample.label
    );
    BulkRead {
        records: sink.records,
    }
}

// -------------------------------------------------------------------------------------------
// The run itself: one sample, every member, compiled and executed.
// -------------------------------------------------------------------------------------------

fn run_sample(sample: &Sample, entry: Entry) -> SampleOutcome {
    let dir = TempDir::new(&sanitize(sample.class));
    dir.write(&format!("{}.class", sample.class), sample.bytes);
    for (name, bytes) in sample.classpath {
        dir.write(name, bytes);
    }

    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(sample.bytes.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let declared: Vec<(String, String, bool)> = inspected
        .inspection
        .header
        .methods
        .iter()
        .map(|member| {
            (
                String::from_utf8_lossy(&member.name.raw().0).into_owned(),
                String::from_utf8_lossy(&member.descriptor.raw().0).into_owned(),
                // Whether the member declares a body at all: the same attribute shell the recovery
                // entry reads, so the applicability line below is a fact about the bytes.
                member
                    .attributes
                    .iter()
                    .any(|shell| shell.name.raw().0.as_slice() == b"Code"),
            )
        })
        .collect();

    // The premise every row rests on: the sample declares the members this comparison classifies,
    // so a renamed fixture fails here instead of quietly covering less.
    for member in sample.members {
        assert!(
            declared.iter().any(|(name, ..)| name == member.name),
            "{}: the sample declares `{}`",
            sample.label,
            member.name
        );
    }

    let mut rows: Vec<Planned> = Vec::new();
    let mut skipped = Vec::new();
    let mut executed = Vec::new();
    let mut counted = Vec::new();

    // The bulk entry's own read of the whole sample, made once before any member is planned: it is
    // one operation, and asking it again per member would be a second operation rather than the
    // entry's shape.
    let bulk = match entry {
        Entry::SingleMethod => None,
        Entry::Bulk => Some(bulk_read(&engine, sample, &snapshot)),
    };

    for (name, descriptor, has_code) in &declared {
        if !has_code {
            // A member with no `Code` attribute (abstract, native) is not a member a run can be
            // asked to present, so it is not a request at all: it is an applicability fact about the
            // sample, listed here and kept out of every denominator below.
            skipped.push(format!(
                "`{name}{descriptor}`: the member declares no `Code` attribute, so there is no body \
                 to present"
            ));
            continue;
        }
        if name.starts_with('<') {
            skipped.push(format!(
                "`{name}{descriptor}`: a constructor or class initializer cannot be re-declared in \
                 the scratch class (its name is the sample's own, and `super()`/`this` are not \
                 modelled)"
            ));
            continue;
        }
        let member = sample
            .members
            .iter()
            .find(|member| member.name == name)
            .unwrap_or_else(|| {
                panic!(
                    "{}: the sample declares `{name}{descriptor}`, which the table does not \
                     classify; every member a run can be asked about is either compared or stated \
                     as a boundary",
                    sample.label
                )
            });

        let request = MethodAnalysisRequest {
            environment: environment(&snapshot),
            method: PhysicalMethodId {
                owner: PhysicalDefinitionId {
                    location: PhysicalClassLocation::StandaloneRoot {
                        snapshot: snapshot.id().clone(),
                    },
                    class_bytes: inspected.source.class_bytes.clone(),
                    variant: PhysicalVariant::Base,
                },
                name: JvmBytes(name.as_bytes().to_vec()),
                descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        };
        // The answer of this member, from the entry this run reads through. The single-method entry
        // asks for this member alone and owns the answer it gets; the bulk entry already read the
        // whole sample in one operation, so the comparison reads the record the operation delivered
        // about this member — the same run, from a class prepared once.
        let asked;
        let recovered: &RecoveredMethod = match &bulk {
            None => {
                let mut budget = Budget::new(limits());
                asked = engine
                    .recover_method(slice::from_ref(&snapshot), &request, &mut budget)
                    .expect("a legal request is answered, not raised");
                &asked
            }
            Some(bulk) => bulk.recovered(sample.label, name, descriptor),
        };
        let report = recovered.recovery();

        // The facts of **this** run: the declaration the presentation read (P3 3.1) and the
        // parameter types P3-R5's fix reads (2.4). Nothing about the signature is spelled by hand.
        let facts = recovered.facts();
        let method = facts.method();
        let (described, (return_type, _)) =
            descriptor_parts(method.descriptor()).unwrap_or_else(|| {
                panic!(
                    "{}: `{name}{descriptor}` is a readable descriptor",
                    sample.label
                )
            });
        let receiver = method.access_flags().is_some_and(|f| f & ACC_STATIC == 0);
        // The run's own parameter-type fact must state what the descriptor states: a `boolean`
        // parameter is spelled `boolean` because that fact says so, and a disagreement here would
        // mean this file is wrapping a different signature than the run wrote for.
        //
        // An **array** parameter is the shape that fact deliberately does not spell: it answers the
        // boolean question, and its `[` branch is the conservative `Object` a value of an array type
        // may not be called a boolean through ([`MethodFacts::parameter_types`]), not a name. The
        // wrapper's own spelling of it is the descriptor's (`byte[]`, `java.lang.String[][]`), it is
        // the declaration witness below that checks it, and `tests/p3_array_types.rs` is what pins
        // the declaration the text publishes against it.
        let stated = method.parameter_types();
        for (slot, spelling, primitive) in &described {
            if !primitive || spelling.ends_with("[]") {
                continue;
            }
            let slot = slot.saturating_add(if receiver { 1 } else { 0 });
            assert_eq!(
                stated
                    .get(&slot)
                    .map(|ty| ty.spell().to_string())
                    .as_deref(),
                Some(spelling.as_str()),
                "{}: `{name}{descriptor}` - the run's parameter-type fact for slot {slot} must spell \
                 what the descriptor states",
                sample.label
            );
        }
        let parameters: Vec<Parameter> = described
            .iter()
            .map(|(slot, spelling, _)| {
                let absolute = slot.saturating_add(if receiver { 1 } else { 0 });
                Parameter {
                    slot: absolute,
                    spelling: spelling.clone(),
                    name: stated_name(facts, absolute)
                        .filter(|name| is_java_identifier(name))
                        .unwrap_or_else(|| format!("arg{absolute}")),
                }
            })
            .collect();
        let declaration = declaration_of(method, &parameters, &return_type, None);
        // The slots a wrapper declares are the run's own numbering, in order: a name written for a
        // parameter of the wrong slot would compile here and compare the wrong body.
        assert!(
            parameters
                .windows(2)
                .all(|pair| pair[0].slot < pair[1].slot),
            "{}: `{name}{descriptor}` - the parameters occupy increasing slots",
            sample.label
        );
        let scratch = format!("Gen{}", sanitize(name));

        let quotes = quoted_bcis(&report.text);
        let unanchored: Vec<u32> = quotes
            .iter()
            .copied()
            .filter(|bci| report.source_map.of_bci(*bci).is_empty())
            .collect();
        let mut codes = Vec::new();
        let mut refused_blocks = Vec::new();
        for region in &report.regions {
            if let Some(code) = region.code {
                codes.push(code);
            }
            if !region.structured {
                refused_blocks.extend(region.blocks.iter().copied());
            }
        }
        // The body's own exception table, read from the fixture's bytes by the reader's entry: this
        // is not a fact about the run, it is what the bytes state. A handler entry the run neither
        // anchors nor quotes is a finding about the run's coverage of that member, and it is asserted
        // by `run_sample` (P3-R7): the comparison refuses to pass over a handler the artifact
        // silently dropped.
        let mut unanchored_handlers = Vec::new();
        let selector = MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        };
        if let Ok(inspection) =
            inspect_method_bytecode(sample.bytes, selector, &mut Budget::new(limits()))
        {
            for handler in &inspection.exception_handlers {
                if report.source_map.of_bci(handler.handler_bci).is_empty() {
                    unanchored_handlers.push(handler.handler_bci);
                }
            }
        }

        // The count control: a body whose value the run refused is placed in the weakest
        // declaration that can hold it, and the calls that text makes are measured there. R2's
        // substance is this count, and it is measured rather than counted in the text. It is
        // measured only for the refusals the sample names in `measured`: where the refusal quoted
        // the effect itself, the fragment has no count to compare (and compiling it only to compare
        // it against an effect it does not perform would be a comparison of two different programs).
        let count_control = if sample.counter.is_some()
            && matches!(member.expect, Expect::Quoted(_))
            && sample.measured.contains(&name.as_str())
        {
            let control_declaration =
                declaration_of(method, &parameters, &return_type, Some("void"));
            let compiled = compile_wrapper(
                dir.path(),
                sample,
                name,
                descriptor,
                &control_declaration,
                &report.text,
            );
            if compiled.is_ok() {
                counted.push(name.clone());
            }
            Some(compiled.map(|_| ()))
        } else {
            None
        };

        let mut planned = Planned {
            name: name.clone(),
            descriptor: descriptor.clone(),
            parameters,
            return_type,
            declaration: declaration.clone(),
            represent: report.representation,
            quality: report.quality,
            content: report.content.clone(),
            text: report.text.clone(),
            produced: report.produced(),
            scratch: scratch.clone(),
            codes,
            quotes,
            unanchored,
            refused_blocks,
            unanchored_handlers,
            refusal: None,
            count_control,
        };
        planned.refusal = compile_wrapper(
            dir.path(),
            sample,
            name,
            descriptor,
            &declaration,
            &report.text,
        )
        .err();

        match member.expect {
            Expect::Executed => {
                assert!(
                    matches!(report.representation, Representation::Java),
                    "{}: `{name}{descriptor}` is a body the run writes whole, and this run states \
                     {:?}",
                    sample.label,
                    report.representation
                );
                assert!(
                    planned.executed(),
                    "{}: the text of `{name}{descriptor}` must compile under the declaration this \
                     file derives from the run's facts; javac refused it:\n{}",
                    sample.label,
                    planned.refusal.as_deref().unwrap_or_default()
                );
                executed.push(name.clone());
            }
            Expect::NotACompilationUnit(why) => {
                assert!(
                    matches!(report.representation, Representation::Java),
                    "{}: `{name}{descriptor}` is written whole by the run, and this run states {:?}",
                    sample.label,
                    report.representation
                );
                assert!(
                    !planned.executed(),
                    "{}: `{name}{descriptor}` is recorded as a boundary ({why}) and its text \
                     compiled, so the boundary no longer is one",
                    sample.label
                );
            }
            Expect::Quoted(code) => {
                assert!(
                    matches!(report.representation, Representation::Mixed),
                    "{}: `{name}{descriptor}` keeps quoted bytecode, and this run states {:?}",
                    sample.label,
                    report.representation
                );
                if let Some(code) = code {
                    assert!(
                        planned.codes.contains(&code),
                        "{}: the refusal of `{name}{descriptor}` must state `{code}`; the regions \
                         state {:?}",
                        sample.label,
                        planned.codes
                    );
                }
            }
        }

        // The quote acceptance of a sample that exists to pin *which* instructions its refusals
        // account for: the indexes the answer's own quotes name, deduplicated, are exactly the ones
        // the table states. No fewer — a named read the quote dropped is an effect the artifact
        // stopped accounting for — and no more, since a quote that names an instruction it did not
        // read would be an anchor pointing at the wrong bytecode.
        if let Some((_, expected)) = sample
            .quotes
            .iter()
            .find(|(member, _)| *member == name.as_str())
        {
            let mut named = planned.quotes.clone();
            named.sort_unstable();
            named.dedup();
            let mut expected = expected.to_vec();
            expected.sort_unstable();
            assert_eq!(
                named, expected,
                "{}: `{name}{descriptor}` must quote exactly the bytecode its refusal accounts \
                 for: {:?}\n{}",
                sample.label, expected, report.text
            );
        }

        // The report's own account of a refusal: every bytecode index of a refused region is quoted
        // in the text, and every quoted index is an anchor of the map. This holds for every member,
        // refused or not: a body written whole quotes nothing.
        for bci in &planned.refused_blocks {
            assert!(
                planned.quotes.contains(bci),
                "{}: `{name}{descriptor}` refuses a region that holds BCI {bci}, and the text must \
                 quote it: {:?}",
                sample.label,
                planned.quotes
            );
        }
        assert!(
            planned.unanchored.is_empty(),
            "{}: `{name}{descriptor}` quotes bytecode no anchor answers for: {:?}",
            sample.label,
            planned.unanchored
        );

        // P3-R7, hardened: the note this file used to print is now a failure. Every instruction the
        // member's bytes decode to has to be **accounted for** by the graph the presentation was
        // built from — covered by a canonical block's half-open span, or held by a node the graph
        // lists as dead — or quoted by the text. The ledger is read from an *independent* analysis of
        // the same bytes through the same request, so this is an assertion about what the class file
        // holds and not about the run's account of itself; and it is stated over the decode's
        // instruction starts rather than over the handler entries alone, because "the graph is not an
        // account of these bytes" is a fact about the body and the handler is only where the committed
        // case shows it (ECJ 4.6.1 v52 `finallyPath`).
        let mut analysis_budget = Budget::new(limits());
        let analyzed =
            analyze_method_ir(slice::from_ref(&snapshot), &request, &mut analysis_budget)
                .expect("the same request whose body was presented reads its own payload");
        let ledger = ledger_of(analyzed.ir());
        assert!(
            ledger.unaccounted.is_empty()
                || ledger
                    .unaccounted
                    .iter()
                    .all(|bci| planned.quotes.contains(bci)),
            "{}: `{name}{descriptor}` decodes instruction(s) {:?} that no canonical block covers and \
             that the graph does not list as unreachable, and the text quotes none of them: a body the \
             graph has no account of may not be presented ({:?}), and the bytes it holds may not be \
             dropped silently",
            sample.label,
            ledger.unaccounted,
            report.representation
        );

        // The handler entries the member's own exception table declares, answered for one by one: the
        // map anchors it, the text quotes it, or a block of the graph covers it — a proved
        // `try`-with-resources states the instructions of the handler it consumed in its region's own
        // record instead of writing a segment per instruction, and that is an answer. An entry with
        // none of the three is one the artifact never mentions, and the run may then not report a
        // complete body.
        let unanswered: Vec<u32> = planned
            .unanchored_handlers
            .iter()
            .copied()
            .filter(|bci| !planned.quotes.contains(bci))
            .filter(|bci| !ledger.accounted.contains(bci))
            .collect();
        assert!(
            unanswered.is_empty() || !matches!(report.representation, Representation::Java),
            "{}: `{name}{descriptor}` declares exception handler entr(ies) {unanswered:?} that no \
             anchor of the map answers for, no quote of the text names and no block of the graph \
             covers, and the run reports {:?}: a body that claims to be complete may not leave a \
             handler it declares unaccounted for",
            sample.label,
            report.representation
        );

        rows.push(planned);
    }

    // The declaration witness: every declaration this file derived is compiled with a body that
    // mentions every parameter it named and returns a value of the declared type. A wrong type or a
    // wrong name is therefore a failure here, never a boundary of the recovered text.
    let witness = witness_source(sample, &rows);
    let witness_file = "Witness.java";
    fs::write(dir.path().join(witness_file), witness).expect("write the witness");
    javac(dir.path(), &[witness_file]).unwrap_or_else(|message| {
        panic!(
            "{}: one of the declarations this file derived from the run's facts does not compile, \
             so the comparison would be wrapping the wrong signature:\n{message}",
            sample.label
        )
    });
    let _ = fs::remove_file(dir.path().join(witness_file));

    // R2's own requirement: the count of `make` calls has to be measurable, so `cast`'s body has to
    // be compilable in the count control.
    if sample.class == "LocalRewrite" {
        assert!(
            counted.iter().any(|name| name == "cast"),
            "{}: the count control of `cast` must compile, or the number of producer calls the \
             refused body makes cannot be measured: {:?}",
            sample.label,
            rows.iter()
                .find(|row| row.name == "cast")
                .map(|row| row.count_control.clone())
        );
    }

    // The original's own answers, executed: the refusals of this sample rest on effects its bytes
    // really perform, and the driver observes them beside the classes the test wrote.
    let baseline = run_baseline(sample, dir.path());

    let (trace_lines, trace_identical, trace) =
        compare_traces(sample, dir.path(), &rows, &executed, &counted);

    // The content reconciliation of this sample, from the reports themselves: every declared member
    // is either requested or stated as not requestable, every request is answered with a produced or
    // a stopped report, the two produced values partition the produced requests, and `not_produced`
    // is exactly the stopped ones. The counts are read from four different places — the header's
    // attribute shells, the loop that performed the requests, the reports' outcome plane and the
    // reports' content field — so the equalities below are statements about the run, not arithmetic
    // on one number.
    let with_code = declared.iter().filter(|(_, _, has_code)| *has_code).count();
    let without_code = declared.len() - with_code;
    let requested = rows.len();
    let produced = rows.iter().filter(|row| row.produced).count();
    let stopped = rows.iter().filter(|row| !row.produced).count();
    let contains_statements = rows
        .iter()
        .filter(|row| row.content == RecoveryContent::ContainsStatements)
        .count();
    let explanation_only = rows
        .iter()
        .filter(|row| row.content == RecoveryContent::ExplanationOnly)
        .count();
    let not_produced = rows
        .iter()
        .filter(|row| row.content == RecoveryContent::NotProduced)
        .count();
    assert_eq!(
        declared.len(),
        requested + skipped.len(),
        "{}: every member the class declares is either requested or stated as not requestable",
        sample.label
    );
    assert_eq!(
        produced + stopped,
        requested,
        "{}: every request is answered with a produced or a stopped report",
        sample.label
    );
    assert_eq!(
        contains_statements + explanation_only,
        produced,
        "{}: the two produced content values partition the produced requests",
        sample.label
    );
    assert_eq!(
        not_produced, stopped,
        "{}: `not_produced` is exactly the stopped requests",
        sample.label
    );

    SampleOutcome {
        label: sample.label.to_string(),
        point: sample.point.to_string(),
        rows,
        declared: declared.len(),
        with_code,
        without_code,
        requested,
        produced,
        contains_statements,
        explanation_only,
        stopped,
        skipped,
        executed,
        counted,
        baseline,
        trace_lines,
        trace_identical,
        trace,
    }
}

/// Runs one sample's committed baseline driver beside the classes this test wrote, and answers with
/// what it printed — or `None` when the sample states none.
///
/// The driver is a real Java source next to the fixture (`include_str!`), it is compiled with the
/// sample on the classpath the way the wrappers are, and it prints one line per observation. The
/// lines are asserted exactly: this is the original class's own behaviour, executed, and it is the
/// only evidence a refusal that quotes an *effect* can rest on.
fn run_baseline(sample: &Sample, dir: &Path) -> Option<String> {
    let baseline = sample.baseline.as_ref()?;
    let file = format!("{}.java", baseline.class);
    fs::write(dir.join(&file), baseline.source)
        .expect("write the baseline driver into the comparison directory");
    javac(dir, &[&file]).unwrap_or_else(|message| {
        panic!(
            "{}: the committed baseline driver compiles beside the sample:\n{message}",
            sample.label
        )
    });
    let output = java(dir, baseline.class);
    assert!(
        output.status.success(),
        "{}: the baseline driver runs:\n{}",
        sample.label,
        String::from_utf8_lossy(&output.stderr)
    );
    let printed = String::from_utf8_lossy(&output.stdout).into_owned();
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(
        lines, baseline.lines,
        "{}: the committed original answers what this comparison and the fixture's README record:\n{printed}",
        sample.label
    );
    Some(printed)
}

/// The sample's declaration witness: every member's declaration, with a body that touches every
/// parameter name and returns a value of the declared type (or nothing, when the type is `void`).
fn witness_source(sample: &Sample, rows: &[Planned]) -> String {
    let mut source = String::new();
    source.push_str("public class Witness");
    if let Some(parent) = sample.extends {
        source.push_str(&format!(" extends {parent}"));
    }
    source.push_str(" {\n");
    for row in rows {
        source.push_str("    ");
        source.push_str(&row.declaration);
        source.push_str(" { ");
        for parameter in &row.parameters {
            source.push_str(&format!("{} = {}; ", parameter.name, parameter.name));
        }
        if let Some(value) = default_value(&row.return_type) {
            source.push_str(&format!("return {value};"));
        }
        source.push_str(" }\n");
    }
    source.push_str("}\n");
    source
}

/// Every bytecode index the text's quotes name, in the order the quotes state them: `// @bytecode 4
/// 0` is one quote naming two indexes.
fn quoted_bcis(text: &str) -> Vec<u32> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|bcis| {
            bcis.split_whitespace().map(|bci| {
                bci.parse::<u32>()
                    .expect("a quoted bytecode index is a number")
            })
        })
        .collect()
}

// -------------------------------------------------------------------------------------------
// The two tests: the required members, and the corpus.
// -------------------------------------------------------------------------------------------

#[test]
#[ignore = "needs a JDK on PATH: it compiles the wrappers it generates with `javac --release 8` and \
            runs them. `cargo test` therefore stays green without a compiler; CI's JDK job runs it \
            with `-- --ignored` (see tests/fixtures/p3-corpus/README.md)"]
fn the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies() {
    let outcomes: Vec<SampleOutcome> = REQUIRED
        .iter()
        .map(|sample| run_sample(sample, Entry::SingleMethod))
        .collect();
    print_outcomes(&outcomes);
}

#[test]
#[ignore = "needs a JDK on PATH: it compiles the wrappers it generates with `javac --release 8` and \
            runs them (see tests/fixtures/p3-corpus/README.md)"]
fn the_corpus_is_read_the_same_way_by_every_legal_flag_set() {
    // The two spellings of "Java 8" for one source and one debug flag: the corpus states that the
    // bytes are the same, so the rows below would be the same rows. This is an assertion about the
    // generation, not a comparison of the layer.
    assert_eq!(
        FLAGS_G_NONE.bytes, FLAGS_SOURCE_TARGET,
        "`--release 8` and `-source 8 -target 8` produce the same bytes for Flags.java, which is \
         why the `v8-source-target` build has no rows of its own"
    );
    let outcomes: Vec<SampleOutcome> = CORPUS
        .iter()
        .map(|sample| run_sample(sample, Entry::SingleMethod))
        .collect();
    print_outcomes(&outcomes);
}

// -------------------------------------------------------------------------------------------
// The same comparison through the bulk entry: the text `jarde-cli export` writes.
//
// The one sample this list cannot take from the corpus is assembled here, because the shape it needs
// — an instance member whose body reads its own field — is behind a visibility wall in every committed
// fixture (see `RECEIVER_FIELD_BYTES`).
// -------------------------------------------------------------------------------------------

/// The `Code` attribute's own overhead, in bytes: `max_stack`, `max_locals`, the body's length, the
/// empty exception table and the body's empty attribute table.
const CODE_OVERHEAD: u32 = 2 + 2 + 4 + 2 + 2;

/// One byte into an assembly buffer, at the cursor, which it then advances. Every line of the class
/// file below is a line of this.
macro_rules! emit {
    ($out:ident, $at:ident, $($byte:expr),+ $(,)?) => {
        $( $out[$at] = $byte; $at += 1; )+
    };
}

/// One `CONSTANT_Utf8_info`: the tag, the length the text itself states, and the text.
macro_rules! emit_utf8 {
    ($out:ident, $at:ident, $text:expr) => {{
        let text: &[u8] = $text;
        let length = text.len() as u16;
        emit!($out, $at, 1, (length >> 8) as u8, length as u8);
        let mut index = 0;
        while index < text.len() {
            emit!($out, $at, text[index]);
            index += 1;
        }
    }};
}

/// One two-byte value, most significant byte first.
macro_rules! emit_u2 {
    ($out:ident, $at:ident, $value:expr) => {{
        let value: u16 = $value;
        emit!($out, $at, (value >> 8) as u8, value as u8);
    }};
}

/// One four-byte value, most significant byte first.
macro_rules! emit_u4 {
    ($out:ident, $at:ident, $value:expr) => {{
        let value: u32 = $value;
        emit!(
            $out,
            $at,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8
        );
    }};
}

/// One `method_info` shell: the flags, the name and descriptor indexes, and its one `Code` attribute
/// over `$body`. The attribute's own length is computed from the body rather than stated, so a body
/// this file edits cannot leave a length behind it.
macro_rules! emit_member {
    ($out:ident, $at:ident, $access:expr, $name:expr, $descriptor:expr, $max_stack:expr,
     $max_locals:expr, $body:expr) => {{
        let body: &[u8] = $body;
        emit_u2!($out, $at, $access);
        emit_u2!($out, $at, $name);
        emit_u2!($out, $at, $descriptor);
        emit_u2!($out, $at, 1);
        emit_u2!($out, $at, 13);
        emit_u4!($out, $at, CODE_OVERHEAD + body.len() as u32);
        emit_u2!($out, $at, $max_stack);
        emit_u2!($out, $at, $max_locals);
        emit_u4!($out, $at, body.len() as u32);
        let mut index = 0;
        while index < body.len() {
            emit!($out, $at, body[index]);
            index += 1;
        }
        emit_u2!($out, $at, 0);
        emit_u2!($out, $at, 0);
    }};
}

/// The assembled sample's own length: `10` (the magic, the minor and major versions and the pool's
/// entry count) + `139` (the pool's twenty entries) + `10` (the class flags, `this_class`,
/// `super_class`, the interface count and the field count) + `8` (the one `field_info`) + `2`
/// (`methods_count`) + `139` (the four member shells and their `Code` attributes) + `2` (the class's own
/// attribute table). [`receiver_field_class`] asserts that its writes fill exactly this many bytes, so a
/// wrong number here is a compile error rather than a class file with trailing bytes.
const RECEIVER_FIELD_LEN: usize = 10 + 139 + 10 + 8 + 2 + 139 + 2;

/// `ReceiverField`'s bytes, and the sample the bulk comparison reads them as.
///
/// The sample is the **receiver read**: an instance member whose body reads (and writes) its own
/// instance field through `this`. No committed fixture of this repository can carry it in a probe's
/// reach, and the reason is always the field's visibility or the class's finality:
///
/// * `p3-declaration/v8/Holder.class` is the closest — `value()I` reads its own `int value` — and
///   that field is `private` in a `final` class: a probe can neither extend the class nor inherit the
///   field, and javac refuses the text with `cannot find symbol: variable value`. Its receiver *is*
///   spelled `this.value` now; the body is pinned at the text level by the receiver rule's own tests
///   (`tests/p3_instance_receiver.rs`), and this file adds the behavioural half on a
///   class a probe can read;
/// * `p3-handlers/v8/Res.class` reads its own two fields, but both are `private` and the class declares
///   no no-argument constructor, so a probe can neither inherit the fields nor construct the class;
/// * `p4-modern/v16/RecordSample.class` and `r2-annotation-positions/v17/RecordOnly.class` are records:
///   a `final` class whose fields are `private final`;
/// * `p4-modern/v17/NestSample$Inner.class` reads the *host's* field through the synthetic `this$0`,
///   so its receiver read is of another instance;
/// * `p3-refused-cast/v8/External.class` carries the package-private `Object instance`, and no instance
///   method of that class reads it — `RefusedCast.instanceCast(External)` reads it through a
///   **parameter**, which is not a receiver.
///
/// So the bytes are assembled here, the way `crates/jarde-cli/tests/task_cli.rs` assembles its deep
/// chain: deterministically, from this file's own statement of the shape, by no compiler. The shape is
/// `p3-declaration`'s `Holder` with the visibility a probe needs — a package-private class with a
/// package-private `int value` — beside the two members that make the read observable and the one that
/// says the receiver rule is about instance members only.
const RECEIVER_FIELD_BYTES: &[u8] = &receiver_field_class();

/// `<init>()V`: `aload_0; invokespecial Object.<init>; aload_0; bipush 7; putfield value; return`.
///
/// The constructor is where the compared state comes from: both sides run their subject through it,
/// and `7` is a value the field's own default would not answer, so a trace that says `7` is a trace of
/// the sample's own field.
const RECEIVER_FIELD_INIT: &[u8] = &[0x2a, 0xb7, 0x00, 12, 0x2a, 0x10, 7, 0xb5, 0x00, 8, 0xb1];

/// `value()I`: `aload_0; getfield value; ireturn` — the receiver read itself.
const RECEIVER_FIELD_VALUE: &[u8] = &[0x2a, 0xb4, 0x00, 8, 0xac];

/// `bump()I`: `aload_0; aload_0; getfield value; iconst_1; iadd; putfield value; aload_0; getfield
/// value; ireturn` — the same receiver written and read back, so the receiver is exercised in a
/// `putfield` and not only in a `getfield`.
const RECEIVER_FIELD_BUMP: &[u8] = &[
    0x2a, 0x2a, 0xb4, 0x00, 8, 0x04, 0x60, 0xb5, 0x00, 8, 0x2a, 0xb4, 0x00, 8, 0xac,
];

/// `scaled(I)I`: `iload_0; iconst_2; imul; ireturn` — the same slot 0 in a `static` member, where it is
/// a parameter and not a receiver.
const RECEIVER_FIELD_SCALED: &[u8] = &[0x1a, 0x05, 0x68, 0xac];

/// One class file, written entry by entry: `class ReceiverField` in the unnamed package.
///
/// It declares a package-private `int value` (non-final, so a probe class extending this class reads
/// it), a constructor that sets it to `7`, an instance method that reads it through the receiver, an
/// instance method that writes it through the receiver and reads it back, and a `static` method whose
/// parameter sits at the same slot 0. It carries **no debug attributes** — no `LocalVariableTable` —
/// so every name in a recovered body comes from the layer's own naming rules rather than from a table,
/// which is what makes the receiver visible in the text at all.
///
/// The constant pool's entries are numbered in the comments (`1`..`20`), and the members and bodies
/// above name them by those numbers.
const fn receiver_field_class() -> [u8; RECEIVER_FIELD_LEN] {
    let mut out = [0u8; RECEIVER_FIELD_LEN];
    let mut at = 0;

    // The header: the magic, class-file version 52.0, and the pool's entry count.
    emit_u4!(out, at, 0xcafe_babe);
    emit_u2!(out, at, 0);
    emit_u2!(out, at, 52);
    emit_u2!(out, at, 21);

    // The pool.
    emit_utf8!(out, at, b"ReceiverField"); // 1
    emit!(out, at, 7);
    emit_u2!(out, at, 1); // 2: Class 1
    emit_utf8!(out, at, b"java/lang/Object"); // 3
    emit!(out, at, 7);
    emit_u2!(out, at, 3); // 4: Class 3
    emit_utf8!(out, at, b"value"); // 5: the field, and the instance method that reads it
    emit_utf8!(out, at, b"I"); // 6
    emit!(out, at, 12);
    emit_u2!(out, at, 5);
    emit_u2!(out, at, 6); // 7: NameAndType 5:6
    emit!(out, at, 9);
    emit_u2!(out, at, 2);
    emit_u2!(out, at, 7); // 8: Fieldref 2.7
    emit_utf8!(out, at, b"<init>"); // 9
    emit_utf8!(out, at, b"()V"); // 10
    emit!(out, at, 12);
    emit_u2!(out, at, 9);
    emit_u2!(out, at, 10); // 11: NameAndType 9:10
    emit!(out, at, 10);
    emit_u2!(out, at, 4);
    emit_u2!(out, at, 11); // 12: Methodref Object.<init>
    emit_utf8!(out, at, b"Code"); // 13
    emit_utf8!(out, at, b"()I"); // 14: the descriptor the two `int` methods share
    emit!(out, at, 12);
    emit_u2!(out, at, 5);
    emit_u2!(out, at, 14); // 15: NameAndType value()I
    emit_utf8!(out, at, b"bump"); // 16
    emit!(out, at, 12);
    emit_u2!(out, at, 16);
    emit_u2!(out, at, 14); // 17: NameAndType bump()I
    emit_utf8!(out, at, b"scaled"); // 18
    emit_utf8!(out, at, b"(I)I"); // 19
    emit!(out, at, 12);
    emit_u2!(out, at, 18);
    emit_u2!(out, at, 19); // 20: NameAndType scaled(I)I

    // The class: `ACC_SUPER` alone. Package-private, so a probe class in the same unnamed package may
    // extend it, and deliberately **not** `final` — a final class is the thing a probe cannot extend,
    // and extending it is how the probe's body reads this class's own field.
    emit_u2!(out, at, 0x0020);
    emit_u2!(out, at, 2);
    emit_u2!(out, at, 4);
    emit_u2!(out, at, 0); // no interfaces
    emit_u2!(out, at, 1); // one field
    emit_u2!(out, at, 0x0000); // `int value`: package-private, non-final, no attributes
    emit_u2!(out, at, 5);
    emit_u2!(out, at, 6);
    emit_u2!(out, at, 0);
    emit_u2!(out, at, 4); // four methods
    // The flags, the name and descriptor indexes, and the operand-stack height each body needs:
    // `bump` needs three (`this`, the field it read, and the `1` it adds) and every other body two or
    // one. A height that is too small is a class file the JVM's verifier refuses — which is what the
    // first run of this sample was, before the `3` below was measured rather than guessed.
    emit_member!(out, at, 0x0000, 9, 10, 2, 1, RECEIVER_FIELD_INIT);
    emit_member!(out, at, 0x0001, 5, 14, 1, 1, RECEIVER_FIELD_VALUE);
    emit_member!(out, at, 0x0001, 16, 14, 3, 1, RECEIVER_FIELD_BUMP);
    emit_member!(out, at, 0x0009, 18, 19, 2, 1, RECEIVER_FIELD_SCALED);
    emit_u2!(out, at, 0); // no class attributes

    assert!(
        at == RECEIVER_FIELD_LEN,
        "the assembly writes exactly the buffer it states"
    );
    out
}

/// The receiver sample: the assembled class above, read through both entries like every other sample.
const RECEIVER_FIELD: Sample = Sample {
    label: "assembled receiver-field/v52 (this file's own bytes, no compiler produced them)",
    class: "ReceiverField",
    bytes: RECEIVER_FIELD_BYTES,
    classpath: &[],
    // The probe class extends the sample, which is what puts the recovered body's `this` on an
    // instance that really holds the field.
    extends: Some("ReceiverField"),
    scaffold: &[],
    counter: None,
    measured: &[],
    inputs: None,
    quotes: &[],
    baseline: None,
    members: &[
        Member {
            name: "value",
            expect: Expect::Executed,
        },
        Member {
            name: "bump",
            expect: Expect::Executed,
        },
        Member {
            name: "scaled",
            expect: Expect::Executed,
        },
    ],
    point: "the receiver read as behaviour: `value()I` reads the instance field through `this` and \
            `bump()I` writes and reads it through `this`, both on a probe class that extends the \
            sample and therefore shares its constructor's `7`, so the compared traces state `7` and \
            `8`; the `static` member of the same file states its slot 0 as the parameter it is",
};

/// The samples the bulk comparison runs, and what each one is in the list for.
///
/// The list is short on purpose: every sample is run through **both** entries, and each run compiles
/// and executes one wrapper per member. What it has to cover is the coverage the change's task 6.4
/// names — an instance method that reads its receiver, a body with control flow, and bodies the run
/// does not write as compilable Java — and the five samples below cover all of it between them:
///
/// * `SCOPE_NO_DEBUG` — control flow (`scope`, `armOnly`, `reuse`, whose locals are written across
///   arms and read after a join) and the instance shape whose receiver sits below a category-2
///   parameter (`receiver(long)`, wrapped as an instance method);
/// * `RECEIVER_FIELD` — a body that really **reads** its receiver (`value()I` reads the instance
///   field, `bump()I` writes and reads it), on a class whose field a probe can inherit, beside the
///   `static` member whose slot 0 is a parameter and must not be spelled as a receiver;
/// * `LOCAL_REWRITE` — the refusals whose text quotes the bytecode it could not write (`post`,
///   `saved`, `conditional`, `cast`), with the count control that measures what the quoted text still
///   performs;
/// * `GUARDED` — the guarded shapes (try-with-resources, handler bodies, synchronized bodies), several
///   of which are refused with a stated diagnostic code;
/// * `MISSING_DEPENDENCY` — a body the run writes whole that is **not** a compilation unit, because
///   the type it names is not shipped with the fixture: the boundary is the expected result there, not
///   a failure of the comparison.
const BULK_COMPARISON: &[&Sample] = &[
    &SCOPE_NO_DEBUG,
    &RECEIVER_FIELD,
    &LOCAL_REWRITE,
    &GUARDED,
    &MISSING_DEPENDENCY,
];

/// The same wrappers, the same `javac --release 8`, the same `java`, the same traces — with every
/// body read from [`Engine::recover_all`] instead of [`Engine::recover_method`].
///
/// What this case adds to the two above is the entry, not the comparison: the bulk operation is the
/// one `jarde-cli export` drives, it prepares each class once and hands one record per member over,
/// and until this case no gate fed its text to a compiler and a JVM. The two entries are also compared
/// to each other: the same member's text byte for byte, the same content, the same representation and
/// the same set of executed members, so "the bulk entry is the same pipeline" is checked where the
/// text differs from the original's behaviour (the refusals) rather than only where it does not.
#[test]
#[ignore = "needs a JDK on PATH: it compiles the wrappers it generates with `javac --release 8` and \
            runs them (see tests/fixtures/p3-corpus/README.md)"]
fn the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour() {
    // The same switch the two comparisons above read: with `P3_COMPARISON_TRACES` set, the traces both
    // sides agreed on are printed as well, so the lines this case asserts about the receiver sample are
    // also a run's own output rather than only a message inside an assertion.
    let traces = std::env::var_os("P3_COMPARISON_TRACES").is_some();
    let mut members = 0usize;
    let mut boundaries = 0usize;
    let mut instance = Vec::new();
    for sample in BULK_COMPARISON {
        let single = run_sample(sample, Entry::SingleMethod);
        let bulk = run_sample(sample, Entry::Bulk);

        // Each entry's own trace was compared with the committed original's inside `run_sample`, so
        // the two cannot answer differently for a member they both executed. The rest of this block
        // is about the artifact itself: the text of the member, which no execution covers when the
        // run refused to write it.
        assert_eq!(
            bulk.executed, single.executed,
            "{}: the two entries execute the same members",
            sample.label
        );
        assert_eq!(
            bulk.trace_lines, single.trace_lines,
            "{}: both entries' traces carry the same number of compared lines",
            sample.label
        );
        assert_eq!(
            bulk.rows.len(),
            single.rows.len(),
            "{}: both entries are asked about the same members",
            sample.label
        );

        let mut sample_boundaries = 0usize;
        for (single_row, bulk_row) in single.rows.iter().zip(bulk.rows.iter()) {
            assert_eq!(bulk_row.name, single_row.name, "{}", sample.label);
            assert_eq!(
                bulk_row.descriptor, single_row.descriptor,
                "{}",
                sample.label
            );
            assert_eq!(
                bulk_row.text, single_row.text,
                "{}: `{}{}` - the two entries deliver the same text, byte for byte",
                sample.label, bulk_row.name, bulk_row.descriptor
            );
            assert_eq!(bulk_row.content, single_row.content, "{}", sample.label);
            assert_eq!(bulk_row.represent, single_row.represent, "{}", sample.label);
            assert_eq!(
                bulk_row.executed(),
                single_row.executed(),
                "{}: `{}{}` - both entries reach the same verdict about the wrapper",
                sample.label,
                bulk_row.name,
                bulk_row.descriptor
            );
            // A member this comparison did **not** execute is a boundary the run itself states: the
            // text quotes bytecode the run refused to write as Java, or javac refused the wrapper the
            // text was placed in. An unexecuted row that states neither is a row this file silently
            // dropped, which is the one thing this comparison may not do — so the boundary is not
            // only tolerated here, it is required to have a reason.
            if !bulk_row.executed() {
                assert!(
                    matches!(bulk_row.represent, Representation::Mixed)
                        || bulk_row.refusal.is_some(),
                    "{}: `{}{}` is not executed and states no reason: representation {:?}, quotes \
                     {:?}",
                    sample.label,
                    bulk_row.name,
                    bulk_row.descriptor,
                    bulk_row.represent,
                    bulk_row.quotes
                );
                sample_boundaries += 1;
            }
            if !bulk_row
                .declaration
                .split_whitespace()
                .any(|word| word == "static")
            {
                instance.push(format!("{}{}", bulk_row.name, bulk_row.descriptor));
            }
            members += 1;
        }
        boundaries += sample_boundaries;
        // The receiver read, asserted from the artifact **and** from the trace rather than from the
        // run's own classification. `ReceiverField.value()I` and `bump()I` read the instance field
        // through the receiver, so their text spells it `this.value` — the slot ordinal this layer
        // wrote before the receiver rule landed would be `arg0.value`, which the wrapper declares no
        // name for and which javac therefore refuses — and the traces both sides agreed on are the
        // sample's own state: `7` is the constructor's value rather than the field's default, and `8`
        // is the same field written through the same receiver and read back. The `static` member of
        // the same file is the control: the same slot 0 is a parameter there, and a `this` written
        // into it would be a body no compiler accepts.
        if sample.class == "ReceiverField" {
            let row = |name: &str| {
                bulk.rows
                    .iter()
                    .find(|row| row.name == name)
                    .unwrap_or_else(|| panic!("{}: `{name}` is planned", sample.label))
            };
            for name in ["value", "bump"] {
                let member = row(name);
                assert!(
                    member.text.contains("this.value"),
                    "{}: `{name}()I` reads the instance field through its receiver, so its text reads \
                     `this.value`: {:?}\n{}",
                    sample.label,
                    member.refusal,
                    member.text
                );
                assert!(
                    !member.text.contains("arg0."),
                    "{}: `{name}()I`'s receiver is not the slot ordinal: {}",
                    sample.label,
                    member.text
                );
                assert!(
                    member.executed(),
                    "{}: `{name}()I` compiles under a probe class that extends the sample and shares \
                     its state: {:?}\n{}",
                    sample.label,
                    member.refusal,
                    member.text
                );
            }
            let scaled = row("scaled");
            assert!(
                !scaled.text.contains("this") && scaled.text.contains("arg0"),
                "{}: `scaled(I)I` is `static`, so its slot 0 is a parameter and is spelled as one: \
                 {:?}\n{}",
                sample.label,
                scaled.refusal,
                scaled.text
            );
            assert!(
                scaled.executed(),
                "{}: the `static` member runs as every other member of this list does: {:?}\n{}",
                sample.label,
                scaled.refusal,
                scaled.text
            );
            for line in [
                "value value()I [] -> 7",
                "value bump()I [] -> 8",
                "value scaled(I)I [7] -> 14",
                "value scaled(I)I [0] -> 0",
                "value scaled(I)I [-1] -> -2",
            ] {
                assert!(
                    bulk.trace.contains(line),
                    "{}: the compared traces state `{line}`:\n{}",
                    sample.label,
                    bulk.trace
                );
            }
        }
        println!(
            "\n## {} through {} — {} member(s), {} executed, {} boundary/boundaries, {} trace line(s) \
             identical to the original",
            sample.label,
            Entry::Bulk.name(),
            bulk.rows.len(),
            bulk.executed.len(),
            sample_boundaries,
            bulk.trace_lines
        );
        if traces {
            println!("{}", bulk.trace);
        }
    }

    assert!(
        instance
            .iter()
            .any(|member| member.starts_with("receiver(")),
        "the list covers the instance-member shape (`receiver(long)` reads `this` at slot 0 below a \
         category-2 parameter); the members with a receiver were {instance:?}"
    );
    assert!(
        boundaries > 0,
        "the list covers the refusal side of the comparison: a run whose every row executed could not \
         tell \"the bulk entry refuses the same bodies\" from \"this list has no refusals\" ({members} \
         member(s) compared)"
    );
}

fn print_outcomes(outcomes: &[SampleOutcome]) {
    let traces = std::env::var_os("P3_COMPARISON_TRACES").is_some();
    for outcome in outcomes {
        println!("\n## {}", outcome.label);
        println!("{}", outcome.point.replace('\n', " "));
        println!();
        println!("| member | run | content | wrapper | result |");
        println!("| --- | --- | --- | --- | --- |");
        for row in &outcome.rows {
            let run = format!(
                "{}/{}",
                representation_name(row.represent),
                quality_name(row.quality)
            );
            let wrapper = match &row.refusal {
                None => "compiles".to_string(),
                Some(message) => format!("javac refuses: {}", first_line(message)),
            };
            let result = if outcome.executed.iter().any(|name| name == &row.name) {
                "executed: traces identical".to_string()
            } else if outcome.counted.iter().any(|name| name == &row.name) {
                format!(
                    "not executed; count control {}",
                    match row.count_control {
                        Some(Ok(())) => "compiles",
                        _ => "does not compile",
                    }
                )
            } else {
                format!(
                    "boundary: {} quoted BCI(s), refused regions {:?}",
                    row.quotes.len(),
                    row.refused_blocks
                )
            };
            println!(
                "| `{}{}` | {run} | {} | {wrapper} | {result} |",
                row.name,
                row.descriptor,
                content_name(&row.content)
            );
            if let Some(Err(message)) = &row.count_control {
                println!(
                    "| | | | | the count control does not compile either: {} |",
                    first_line(message)
                );
            }
        }
        for skip in &outcome.skipped {
            println!("- not wrapped: {skip}");
        }
        // The content reconciliation of this sample, printed from the report fields the rows carry:
        // what the requests were, what the engine delivered, and how the two content values and the
        // stops add up. The sample's applicability is stated beside the coverage numbers so that a
        // member with no body, or an initializer the comparison cannot re-declare, is not read as a
        // failed request. `contains_statements` is not a recovery rate: `explanation_only` is the
        // share of *produced* artifacts that hold no statement, and neither value nor their sum says
        // anything about how much of the body was recovered.
        println!();
        println!("| content | count |");
        println!("| --- | --- |");
        println!("| members declared | {} |", outcome.declared);
        println!("| members with `Code` | {} |", outcome.with_code);
        println!(
            "| members without `Code` (applicability only) | {} |",
            outcome.without_code
        );
        println!("| requests performed | {} |", outcome.requested);
        println!("| produced | {} |", outcome.produced);
        println!("| contains_statements | {} |", outcome.contains_statements);
        println!("| explanation_only | {} |", outcome.explanation_only);
        println!("| stopped | {} |", outcome.stopped);
        println!(
            "| not requested (initializer/constructor) | {} |",
            outcome.skipped.len() - outcome.without_code
        );
        println!(
            "reconciliation: {} declared = {} with `Code` + {} without; {} requested = {} produced \
             + {} stopped; {} produced = {} contains_statements + {} explanation_only",
            outcome.declared,
            outcome.with_code,
            outcome.without_code,
            outcome.requested,
            outcome.produced,
            outcome.stopped,
            outcome.produced,
            outcome.contains_statements,
            outcome.explanation_only
        );
        // The refusals that rest on the original's own behaviour: the committed driver's output, as
        // the fixture's README records it.
        if let Some(baseline) = &outcome.baseline {
            println!("\nthe committed baseline driver printed:");
            println!("```text");
            print!("{baseline}");
            println!("```");
        }
        println!(
            "The declaration of each member, as this comparison derived it from that run's own facts:"
        );
        for row in &outcome.rows {
            println!(
                "- `{}{}` is wrapped as `{}`",
                row.name, row.descriptor, row.declaration
            );
        }
        println!(
            "\ntrace: {} line(s), {}",
            outcome.trace_lines,
            if outcome.trace_identical {
                "identical on both sides"
            } else {
                "DIFFERENT"
            }
        );
        if traces {
            println!("{}", outcome.trace);
        }
    }
}

/// What one member's decode and its graph say about each other (P3-R7).
///
/// The two lists partition the instruction starts the bytes decode to: `accounted` holds the ones
/// the canonical graph answers for — a block's half-open span, from the first original block start it
/// stands for to its `end_bci`, or a node the graph lists as dead, which is a statement about an
/// instruction and not a silence about it — and `unaccounted` holds the rest: the instruction starts
/// in no range and in no dead node, which are the ones no presentation of this graph may drop.
struct Ledger {
    accounted: Vec<u32>,
    unaccounted: Vec<u32>,
}

/// Reads one analysis's ledger: where the decode and the canonical graph disagree (P3-R7).
fn ledger_of(ir: &MethodIr) -> Ledger {
    let (Some(code), Some(canonical)) = (ir.code(), ir.canonical()) else {
        return Ledger {
            accounted: Vec::new(),
            unaccounted: Vec::new(),
        };
    };
    let mut covered = std::collections::BTreeSet::new();
    for block in canonical.blocks() {
        let start = block
            .blocks()
            .first()
            .copied()
            .unwrap_or_else(|| block.id().bci());
        covered.extend(start..block.end_bci());
    }
    for id in canonical.unreachable() {
        covered.insert(id.bci());
        if let Some(block) = canonical.blocks().iter().find(|block| block.id() == id) {
            let start = block.blocks().first().copied().unwrap_or_else(|| id.bci());
            covered.extend(start..block.end_bci());
        }
    }
    let mut accounted = Vec::new();
    let mut unaccounted = Vec::new();
    for instruction in &code.instructions {
        if covered.contains(&instruction.bci) {
            accounted.push(instruction.bci);
        } else {
            unaccounted.push(instruction.bci);
        }
    }
    Ledger {
        accounted,
        unaccounted,
    }
}

/// The line of a javac message that states the refusal: the first line that says `error`, because
/// the lines before it are the obsolete-option warnings every `--release 8` compile prints.
fn first_line(message: &str) -> String {
    message
        .lines()
        .find(|line| line.contains("error"))
        .or_else(|| message.lines().find(|line| !line.trim().is_empty()))
        .unwrap_or("")
        .trim()
        .to_string()
}

fn representation_name(representation: Representation) -> &'static str {
    match representation {
        Representation::Java => "Java",
        Representation::Mixed => "Mixed",
        _ => "other",
    }
}

fn quality_name(quality: Quality) -> &'static str {
    match quality {
        Quality::Structured => "Structured",
        Quality::Fallback => "Fallback",
        _ => "other",
    }
}

/// The wire name of one content value, so the table prints the same spelling the report publishes.
fn content_name(content: &RecoveryContent) -> &'static str {
    match content {
        RecoveryContent::NotProduced => "not_produced",
        RecoveryContent::ExplanationOnly => "explanation_only",
        RecoveryContent::ContainsStatements => "contains_statements",
    }
}
