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

const REQUIRED: &[&Sample] = &[
    &LOCAL_REWRITE,
    &SCOPE_NO_DEBUG,
    &SCOPE_DEBUG,
    &GUARDED,
    &ECJ_V52,
    &NESTED_EVAL,
    &REFUSED_CAST,
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

fn calls_for(parameters: &[Parameter]) -> Vec<Call> {
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
    let calls = calls_for(&plan.parameters);
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
// The run itself: one sample, every member, compiled and executed.
// -------------------------------------------------------------------------------------------

fn run_sample(sample: &Sample) -> SampleOutcome {
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
        let mut budget = Budget::new(limits());
        let recovered = engine
            .recover_method(slice::from_ref(&snapshot), &request, &mut budget)
            .expect("a legal request is answered, not raised");
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
        let stated = method.parameter_types();
        for (slot, spelling, primitive) in &described {
            if !primitive {
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
    let outcomes: Vec<SampleOutcome> = REQUIRED.iter().map(|sample| run_sample(sample)).collect();
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
    let outcomes: Vec<SampleOutcome> = CORPUS.iter().map(|sample| run_sample(sample)).collect();
    print_outcomes(&outcomes);
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
