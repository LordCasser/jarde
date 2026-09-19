//! P3 3.1 acceptance: where a local's declaration is written, what a name is a name for, and which
//! facts the entry reads from the run that decoded the body.
//!
//! Everything here goes through the entry point the CLI calls ([`Engine::recover_method`]) over a
//! **real compiled sample** — the committed javac 23.0.1 classes under `tests/fixtures/p3-scope/`,
//! whose README states the two commands, the digests and the bytecode of every member. The sample is
//! compiled **twice**, so the same shapes are read with and without debug evidence:
//!
//! * `v8/Scope.class` is `--release 8 -g:none`: no `LocalVariableTable`, so every name is an ordinal;
//! * `v8-debug/Scope.class` is the same source with `-g`, so the table states the source names.
//!
//! The closure this file pins is the review's **P3-R3**: `scope(Z)I` is
//! `iload_0; ifeq 9; iconst_1; istore_1; goto 11; iconst_2; istore_1; iload_1; ireturn` — slot 1 is
//! written in the `then` arm and in the `else` arm and read after the join. A declaration written at
//! the first write is out of scope at both of the other uses, and `javac --release 8` refuses the
//! text. The declaration therefore belongs where every use can see it:
//!
//! ```text
//! {
//!     int local1;
//!     if (arg0 != 0) {
//!         local1 = 1;
//!     } else {
//!         local1 = 2;
//!     }
//!     return local1;
//! }
//! ```
//!
//! The controls in the other direction are here too, because "hoist everything" is not a fix:
//! `simple()I` fills and reads one slot in one straight run and keeps `int local0 = 5;`, and
//! `armOnly(I)I` declares its arm-local `z` **inside** the `then` arm where it is used and nowhere
//! else. `reuse(ZI)I` is the slot-reuse shape (two variables in disjoint scopes on one slot), `after`
//! / `reassign` / `receiver` are the category-2 shapes (`long` takes two slots, so the `int`
//! parameter sits at slot 2), and the `-g` sample checks that a name the class file states is used
//! where it exists — and that a slot the table names twice falls back to an ordinal instead of
//! picking one of the two names.
//!
//! What the entry reads to decide all of that is the **same run**'s own facts: one header read, one
//! body attempt (the numbers are asserted below, and for the `-g` sample too — a second read of the
//! class for the debug table would show up in them).

use jarde::*;
use std::slice;

/// The no-debug sample, compiled by javac 23.0.1 `--release 8 -g:none` (see the fixture's README).
const NO_DEBUG: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");
/// The same source with `-g`: its `LocalVariableTable` states the source names.
const DEBUG: &[u8] = include_bytes!("fixtures/p3-scope/v8-debug/Scope.class");

/// The members this sample declares, each with a body: the premise every request below rests on.
const DECLARED: [(&[u8], &[u8]); 8] = [
    (b"<init>", b"()V"),
    (b"scope", b"(Z)I"),
    (b"simple", b"()I"),
    (b"armOnly", b"(I)I"),
    (b"reuse", b"(ZI)I"),
    (b"after", b"(JI)I"),
    (b"reassign", b"(JI)I"),
    (b"receiver", b"(J)J"),
];

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

/// One caller domain rooted at the fixture's own snapshot, and nothing else: the simplest environment
/// the library's validator accepts without a problem.
fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
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

/// The opened sample and the class identity the reader's own header read stated for it.
struct Fixture {
    snapshot: ArtifactSnapshot,
    class_bytes: ClassBytesId,
}

/// Opens one of the two committed samples and reads its header through the reader's own entry point,
/// so that the members presented below are the ones the class declares rather than the ones this file
/// claims.
fn fixture(engine: &Engine, bytes: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let declared: Vec<Vec<u8>> = inspected
        .inspection
        .header
        .methods
        .iter()
        .map(|member| member.name.raw().0.clone())
        .collect();
    for (name, _) in DECLARED {
        assert!(
            declared.iter().any(|member| member.as_slice() == name),
            "the sample declares the member this case presents, `{}`: {declared:?}",
            String::from_utf8_lossy(name)
        );
    }
    Fixture {
        snapshot,
        class_bytes: inspected.source.class_bytes.clone(),
    }
}

/// One recovery run over one member of a sample, through the entry point the CLI calls: both halves of
/// the answer, so the run's own usage is readable beside the presentation.
fn recover(engine: &Engine, fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> RecoveredMethod {
    let request = MethodAnalysisRequest {
        environment: environment(&fixture.snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: fixture.snapshot.id().clone(),
                },
                class_bytes: fixture.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut budget = Budget::new(limits());
    engine
        .recover_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised")
}

/// The usage snapshot of a finished run.
fn usage(recovered: &RecoveredMethod) -> UsageSnapshot {
    match &recovered.analysis().execution {
        ExecutionReport::Complete { usage } => usage.clone(),
        other => panic!("a legal request completes: {other:?}"),
    }
}

/// One member's body as the presentation wrote it, with the planes this file checks everywhere.
fn body(engine: &Engine, fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> String {
    let recovered = recover(engine, fixture, name, descriptor);
    let report = recovered.recovery();
    assert!(report.produced(), "{name:?}: {:?}", report.outcome);
    assert_eq!(
        report.representation,
        Representation::Java,
        "{name:?}: {}\nregions: {:?}\nfallbacks: {:?}",
        report.text,
        report.regions,
        report.fallbacks
    );
    assert_eq!(report.quality, Quality::Structured, "{name:?}: {report:?}");
    report.text.clone()
}

#[test]
fn a_slot_written_in_both_arms_is_declared_where_both_can_see_it() {
    // P3-R3's own member: the review's counterexample, compiled by the same compiler generation.
    let engine = Engine::new();
    let fixture = fixture(&engine, NO_DEBUG);
    let text = body(&engine, &fixture, b"scope", b"(Z)I");

    // The declaration is written once, with no value, before the `if` — the innermost region that
    // contains every use of the slot (`then` arm, `else` arm and the join all see it).
    let declaration = text
        .find("int local1;")
        .unwrap_or_else(|| panic!("the declaration is hoisted above both arms:\n{text}"));
    let if_at = text
        .find("if (")
        .unwrap_or_else(|| panic!("the branch is presented:\n{text}"));
    assert!(
        declaration < if_at,
        "the declaration precedes the branch it is used on both sides of:\n{text}"
    );
    // Both writes are plain assignments: the declaration is not either write's.
    assert!(
        text.contains("local1 = 1;"),
        "the `then` arm's write is an assignment:\n{text}"
    );
    assert!(
        text.contains("local1 = 2;"),
        "the `else` arm's write is an assignment:\n{text}"
    );
    assert!(
        text.contains("return local1;"),
        "the join reads the slot by its name:\n{text}"
    );
    // Not the shape `javac --release 8` refuses: no arm declares the slot.
    assert!(
        !text.contains("int local1 = "),
        "a declaration inside an arm is out of scope in the other one:\n{text}"
    );
    assert_eq!(
        text.matches("int local1;").count(),
        1,
        "one storage location, one declaration:\n{text}"
    );
}

#[test]
fn a_slot_filled_and_read_in_one_region_keeps_its_initial_value() {
    // The control in the other direction: every use of slot 0 is in the region of its first write, so
    // the write still declares it with its value. A fix that hoisted every declaration would answer
    // `int local0; local0 = 5;` here, which is the same program with one more statement — and would
    // mean the rule is "declare nothing where you can" rather than "declare where all uses see it".
    let engine = Engine::new();
    let fixture = fixture(&engine, NO_DEBUG);
    let text = body(&engine, &fixture, b"simple", b"()I");
    assert!(
        text.contains("int local0 = 5;"),
        "the first write still declares the slot with the value it writes:\n{text}"
    );
    assert!(
        text.contains("return local0;"),
        "and the read after it names the slot:\n{text}"
    );
    assert!(
        !text.contains("int local0;"),
        "the declaration was not moved away from the write that fills it:\n{text}"
    );
}

#[test]
fn an_arms_own_local_is_still_declared_inside_that_arm() {
    // `armOnly(I)I` holds both shapes at once: slot 1 (`y`) is written in the prefix and in the arm
    // and read after the join, while slot 2 (`z`) is written and read in the `then` arm alone. The
    // first must be hoisted; the second must stay where it is, or the fix would be an over-hoist.
    let engine = Engine::new();
    let fixture = fixture(&engine, NO_DEBUG);
    let text = body(&engine, &fixture, b"armOnly", b"(I)I");

    let if_at = text
        .find("if (")
        .unwrap_or_else(|| panic!("the branch is presented:\n{text}"));
    let else_at = text
        .find("} else {")
        .unwrap_or_else(|| panic!("both arms are written:\n{text}"));
    let inner = text
        .find("int local2 = arg0 + 1;")
        .unwrap_or_else(|| panic!("the arm's own local is declared with its value:\n{text}"));
    assert!(
        if_at < inner && inner < else_at,
        "the arm's local stays inside the arm that uses it:\n{text}"
    );
    // And the slot both arms and the join touch is hoisted instead.
    let shared = text.find("int local1;").unwrap_or_else(|| {
        panic!("the shared slot is declared where all its uses see it:\n{text}")
    });
    assert!(
        shared < if_at,
        "the shared declaration precedes the branch:\n{text}"
    );
    assert!(
        text.contains("return local1;"),
        "the join reads the shared slot:\n{text}"
    );
}

#[test]
fn two_variables_sharing_one_slot_are_declared_once_where_both_are_visible() {
    // `reuse(ZI)I` is the slot-reuse shape: the compiler gives the `then` arm's `c` and the `else`
    // arm's `d` **one** slot (3), because their scopes are disjoint. This layer names *slots*, not
    // source variables, so the honest presentation is one variable for that one storage location: its
    // declaration is hoisted above the branch — every use of the slot is in one of the two arms — and
    // both writes assign it. The alternative (refusing the body) would lose a shape Java represents
    // faithfully; the one thing that must not happen is two declarations of one slot in two arms.
    let engine = Engine::new();
    let fixture = fixture(&engine, NO_DEBUG);
    let text = body(&engine, &fixture, b"reuse", b"(ZI)I");

    let if_at = text
        .find("if (")
        .unwrap_or_else(|| panic!("the branch is presented:\n{text}"));
    for declaration in ["int local2;", "int local3;"] {
        let at = text
            .find(declaration)
            .unwrap_or_else(|| panic!("`{declaration}` is declared once:\n{text}"));
        assert!(
            at < if_at,
            "the shared slot's declaration precedes both arms:\n{text}"
        );
        assert_eq!(
            text.matches(declaration).count(),
            1,
            "one storage location, one declaration:\n{text}"
        );
    }
    assert!(
        text.contains("local3 = arg1 + 1;") && text.contains("local3 = arg1 + 2;"),
        "each arm writes the reused slot:\n{text}"
    );
    assert!(
        text.contains("local2 = local3;"),
        "and each arm stores it where the join reads it:\n{text}"
    );
    assert!(
        text.contains("return local2;"),
        "the join reads the slot the body really declares:\n{text}"
    );
}

#[test]
fn a_category_two_parameter_leaves_its_second_slot_to_the_signature() {
    // `after(JI)I`, `reassign(JI)I` and `receiver(J)J` are the category-2 shapes: a `long` parameter
    // fills two slots (JVM 2.6.1), so the `int` parameter of the first two sits at slot **2** and the
    // receiver of the third is slot 0 with the `long` at 1 and 2. The count the run states decides
    // both the names and the declarations — a count that stopped at the descriptor's parameter
    // *count* would name the `int` `arg1` and would treat slot 2 as a local the body must declare.
    let engine = Engine::new();
    let fixture = fixture(&engine, NO_DEBUG);

    let after = body(&engine, &fixture, b"after", b"(JI)I");
    assert!(
        after.trim_end().ends_with("{\n    return arg2;\n}"),
        "the `int` parameter is slot 2 and is named as a parameter:\n{after}"
    );
    assert!(
        !after.contains("local"),
        "and nothing of this body is named as a local:\n{after}"
    );

    let reassign = body(&engine, &fixture, b"reassign", b"(JI)I");
    assert!(
        reassign
            .trim_end()
            .ends_with("{\n    arg2 = arg2 + 1;\n    return arg2;\n}"),
        "a parameter is never declared by the body, and its slot is the one the descriptor says:\n{reassign}"
    );
    assert!(
        !reassign.contains("int "),
        "the body declares no local at all: slot 2 is a parameter and the `long` has no local:\n{reassign}"
    );

    let receiver = body(&engine, &fixture, b"receiver", b"(J)J");
    assert!(
        receiver.trim_end().ends_with("{\n    return arg1;\n}"),
        "slot 0 is the receiver, the `long` is slots 1 and 2, and the read names slot 1:\n{receiver}"
    );
    assert!(
        !receiver.contains("local"),
        "the instance member's only local slots are its receiver and its parameter:\n{receiver}"
    );
}

#[test]
fn a_table_that_names_a_slot_twice_states_no_name_for_it() {
    // The `-g` sample: the same shapes, with a `LocalVariableTable`. Three things are checked at once:
    // a name the class file states is used; a slot the table names **twice** (the reused slot 3 of
    // `reuse`, named `c` over one arm and `d` over the other) takes **no** name, because one storage
    // location has one name and neither record's name is the truth about the whole of it; and the
    // ordinal it falls back to is the one the parameter slots imply.
    let engine = Engine::new();
    let fixture = fixture(&engine, DEBUG);

    let scope = body(&engine, &fixture, b"scope", b"(Z)I");
    assert!(
        scope.contains("int x;"),
        "the table's name for the slot is used:\n{scope}"
    );
    assert!(
        scope.contains("if (b != 0) {") && scope.contains("x = 1;") && scope.contains("x = 2;"),
        "the parameter and the local are named, and both writes assign:\n{scope}"
    );
    assert!(
        scope.contains("return x;"),
        "the join reads the name the table states:\n{scope}"
    );

    let reuse = body(&engine, &fixture, b"reuse", b"(ZI)I");
    assert!(
        reuse.contains("int local3;"),
        "a slot the table names twice keeps the ordinal name instead of one of the two:\n{reuse}"
    );
    assert!(
        !reuse.contains("local3 = c;") && !reuse.contains("local3 = d;"),
        "neither of the two names is written for the shared slot:\n{reuse}"
    );
    assert!(
        reuse.contains("int a;") && reuse.contains("a = local3;"),
        "the slot the table names once (`a`, over two ranges) keeps its name:\n{reuse}"
    );
}

#[test]
fn the_debug_table_is_read_by_the_same_run_that_decoded_the_body() {
    // A16 for this slice: the names and the parameter slots come from the run's own reads. The `-g`
    // sample's table is inside the `Code` entry the one body decode already billed, so the numbers do
    // not move: one header, one body attempt, and the bytes a body of that size costs. A second read
    // of the class for the table would show up in them.
    let engine = Engine::new();
    let fixture = fixture(&engine, DEBUG);
    let recovered = recover(&engine, &fixture, b"scope", b"(Z)I");
    let run = usage(&recovered);
    assert_eq!(run.class_headers, 1, "one header read per request");
    assert_eq!(run.method_bodies, 1, "and one body attempt");
    assert_eq!(
        recovered
            .analysis()
            .reads
            .iter()
            .map(|read| read.reason)
            .collect::<Vec<_>>(),
        vec![ReadReason::DriverMethodBody],
        "the one read this request charged is the driver method's body"
    );
    assert_eq!(
        (
            run.attribute_bytes,
            run.code_bytes,
            run.class_bytes,
            run.read_bytes
        ),
        PINNED,
        "the debug table is inside the body's own `Code` entry, so it costs no dimension of its own"
    );
}

/// The dimensions one `scope(Z)I` request over the `-g` sample charges, measured before the debug
/// table was read at all: the table is inside the `Code` entry the body decode already billed, so
/// these numbers are the same with and without it.
const PINNED: (u64, u64, u64, u64) = (797, 13, 1101, 1101);
