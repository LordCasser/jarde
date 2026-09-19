//! P3 2.4 acceptance: the guarded regions — `try`-with-resources and `synchronized` — presented
//! where a rule proves them, and refused with the link that fell short where it does not.
//!
//! Everything here goes through the entry point the CLI calls ([`Engine::recover_method`]) over a
//! **real compiled sample**: the committed javac 23.0.1 class under `tests/fixtures/p3-handlers/`,
//! whose README states the command, the digests and the bytecode of every member. Nothing here
//! states what a `close`, a handler or a monitor is — the rules read all of it from the same run's
//! payload, and every negative case below is the **same sample** with one stated byte changed, so
//! that a refusal is evidence about the proof rather than about a hand-written fixture.
//!
//! What the positive cases pin:
//!
//! * the resources of a header are the slots the handlers close, **in the order their
//!   initialisations run**, and the closes the normal path performs are matched against them in
//!   reverse — so the text closes what the bytecode closed, in the order it closed it;
//! * the statements that *are* the closers (the `if (r != null) r.close();` groups, the handlers, the
//!   `addSuppressed` calls) are claimed and never written: the text has one close per resource, the
//!   compiler writes it, and the anchors record the BCIs the shape was proved from;
//! * a `synchronized` block is written only where every path out of the region leaves the monitor.
//!
//! And what the negative cases pin: a `finally` copy, a `catch` beside the `try`, a branching body,
//! a close the exception path lacks, a suppression that is the other way round, a handler range that
//! swallows the initialisation, a monitor whose exception path never exits, and a second entry into
//! a monitor region are each **refused** with the rule that examined them and the link that fell
//! short — never presented with the part that could not be proved dropped.

use jarde::*;
use std::collections::BTreeSet;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-handlers/v8/Guarded.class");

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

/// One caller domain rooted at the fixture's own snapshot, and nothing else.
fn environment(snapshot: &ArtifactSnapshot, release: u16) -> ResolutionEnvironment {
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
                java_release: release,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: domain.clone(),
        },
        domains: vec![domain],
        providers: Vec::new(),
    }
}

struct Fixture {
    snapshot: ArtifactSnapshot,
    class_bytes: ClassBytesId,
}

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
    Fixture {
        snapshot,
        class_bytes: inspected.source.class_bytes.clone(),
    }
}

fn recover_with(engine: &Engine, fixture: &Fixture, name: &[u8], release: u16) -> RecoveredMethod {
    let request = MethodAnalysisRequest {
        environment: environment(&fixture.snapshot, release),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: fixture.snapshot.id().clone(),
                },
                class_bytes: fixture.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut budget = Budget::new(limits());
    engine
        .recover_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised")
}

fn report_of(engine: &Engine, fixture: &Fixture, name: &[u8]) -> RecoveryReport {
    recover_with(engine, fixture, name, 8).recovery().clone()
}

/// One member presented as Java structure, with the planes every positive case states.
fn presented(engine: &Engine, fixture: &Fixture, name: &str) -> RecoveryReport {
    let report = report_of(engine, fixture, name.as_bytes());
    assert!(report.produced(), "{name}: {:?}", report.outcome);
    assert_eq!(
        report.representation,
        Representation::Java,
        "{name}: {}\nregions: {:?}\nfallbacks: {:?}",
        report.text,
        report.regions,
        report.fallbacks
    );
    assert_eq!(report.quality, Quality::Structured, "{name}: {report:?}");
    assert!(
        report.fallbacks.is_empty(),
        "{name}: {:?}",
        report.fallbacks
    );
    report
}

/// One member quoted rather than presented, refused under one stated code.
///
/// The scan still has to be **complete** (P3's A13 and the spec's "complete scan with fallback
/// quality"): a region this build cannot present is a weaker *presentation*, never a partial read,
/// and the quality is never rewritten to `Partial` for it.
fn refused(engine: &Engine, fixture: &Fixture, name: &str, code: &str) -> RecoveredMethod {
    let recovered = recover_with(engine, fixture, name.as_bytes(), 8);
    let report = recovered.recovery();
    assert!(report.produced(), "{name}: {:?}", report.outcome);
    assert_eq!(
        report.representation,
        Representation::Mixed,
        "{name}: {}",
        report.text
    );
    assert_eq!(report.quality, Quality::Fallback, "{name}: {report:?}");
    assert!(
        report.fallbacks.contains(&code),
        "{name}: {}\nfallbacks: {:?}",
        report.text,
        report.fallbacks
    );
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == code)
        .unwrap_or_else(|| {
            panic!(
                "{name}: no diagnostic under {code}: {:?}",
                report.diagnostics
            )
        });
    assert!(
        !diagnostic.message.is_empty(),
        "{name}: the diagnostic states nothing"
    );
    assert!(
        diagnostic.message.contains("BCI"),
        "{name}: the diagnostic names no instruction: {}",
        diagnostic.message
    );
    let coverage = &recovered.analysis().coverage;
    assert_eq!(
        coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "{name}: a refused region is not a partial scan"
    );
    assert!(
        coverage.artifact_structural.skipped.is_empty(),
        "{name}: {:?}",
        coverage.artifact_structural.skipped
    );
    recovered
}

/// Every BCI an artifact's anchors name.
fn anchors(report: &RecoveryReport) -> BTreeSet<u32> {
    report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis())
        .collect()
}

/// Every BCI a quoted region's own comment cites.
fn cited(text: &str) -> BTreeSet<u32> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|bcis| bcis.split_whitespace())
        .map(|bci| bci.parse::<u32>().expect("a cited BCI is a number"))
        .collect()
}

/// One byte sequence of the sample replaced by another, with the count of sites rewritten.
fn patched(bytes: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    assert_eq!(needle.len(), replacement.len(), "a patch keeps every BCI");
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0usize;
    let mut sites = 0usize;
    while at < bytes.len() {
        if bytes[at..].starts_with(needle) {
            out.extend_from_slice(replacement);
            at += needle.len();
            sites += 1;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    assert!(sites > 0, "the sample holds the sequence this case patches");
    out
}

#[test]
fn a_single_resource_is_declared_in_the_header_and_closed_by_the_compiler() {
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let report = presented(&engine, &fixture, "one");
    assert_eq!(
        report.text,
        "// @method one()V\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (Res local0 = open(\"r\")) {\n        body();\n    }\n    return;\n}\n",
        "{}",
        report.text
    );
    // The statement states its rule, and the closes it replaced are anchors of its own text: BCI 14
    // is the normal path's `close`, BCIs 26 and 33 are the exceptional path's.
    assert!(
        report.rules.iter().any(|rule| rule.rule() == "twr"),
        "{:?}",
        report.rules
    );
    let anchors = anchors(&report);
    for bci in [0, 5, 14, 20, 26, 33, 39] {
        assert!(anchors.contains(&bci), "BCI {bci}: {anchors:?}");
    }
    // The statement's own text is anchored where its initialisation runs.
    assert!(
        !report.source_map.derived_of_bci(14).is_empty(),
        "the close at BCI 14 is recorded beside the text that took its place"
    );
    // One region for the statement and one for what follows it, both structured.
    assert_eq!(report.regions.len(), 2, "{:?}", report.regions);
    assert!(report.regions.iter().all(|region| region.structured));
}

#[test]
fn three_resources_are_declared_in_the_order_whose_closes_run_backwards() {
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    for (name, resources) in [
        ("two", "Res local0 = open(\"r\"); Res local1 = open(\"s\")"),
        (
            "three",
            "Res local0 = open(\"r\"); Res local1 = open(\"s\"); Res local2 = open(\"t\")",
        ),
    ] {
        let report = presented(&engine, &fixture, name);
        assert!(
            report.text.contains(&format!("try ({resources}) {{")),
            "{name}: {}",
            report.text
        );
        // One `close` mention in the text: the compiler writes it, and the bytecode's own closes —
        // three of them for three resources — are claimed and never written.
        assert_eq!(
            report.text.matches("close(").count(),
            0,
            "{name}: {}",
            report.text
        );
    }
    // The header order is the *reverse* of the normal path's close chain: `two()` closes `s` (BCI
    // 20) and then `r` (BCI 51), and the header declares `r` first.
    let report = presented(&engine, &fixture, "two");
    let anchors = anchors(&report);
    for bci in [20, 32, 41, 51, 63, 72] {
        assert!(anchors.contains(&bci), "BCI {bci}: {anchors:?}");
    }
}

#[test]
fn a_resource_initialised_after_an_earlier_one_is_inside_the_region_that_closes_it() {
    // `try (Res r = open("r"); Res s = fail("s"))`: the compiler protects the *second*
    // initialisation with the first resource's row, so the header's order is the order that closes
    // `r` when `fail` throws. The rule reads that row rather than assuming it.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let report = presented(&engine, &fixture, "secondInitFails");
    assert!(
        report
            .text
            .contains("try (Res local0 = open(\"r\"); Res local1 = fail(\"s\")) {"),
        "{}",
        report.text
    );
}

#[test]
fn a_synchronized_block_is_written_only_where_every_path_leaves_the_monitor() {
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    for (name, body) in [("sync", "body();"), ("syncThrows", "boom();")] {
        let report = presented(&engine, &fixture, name);
        assert!(
            report.text.contains(&format!(
                "synchronized (Guarded.LOCK) {{\n        {body}\n    }}"
            )),
            "{name}: {}",
            report.text
        );
        assert!(
            report.rules.iter().any(|rule| rule.rule() == "monitor"),
            "{name}: {:?}",
            report.rules
        );
        // The exits — the normal path's and the handler's — are anchors of the statement's text.
        let anchors = anchors(&report);
        // The enter, both exits and the handler's own entry are anchors of the statement.
        for bci in [5, 10, 14, 16] {
            assert!(anchors.contains(&bci), "BCI {bci}: {anchors:?}");
        }
    }
}

#[test]
fn a_try_with_a_catch_beside_it_is_refused_rather_than_partially_presented() {
    // javac wraps the whole construct in a row of its own; the `twr@1` rule refuses the shape
    // instead of presenting a `try` whose outer handler it would have to ignore.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let recovered = refused(&engine, &fixture, "withCatch", "jre_guard_unexplained_row");
    let report = recovered.recovery();
    assert!(!report.text.contains("try ("), "{}", report.text);
    // The citation lists the BCIs of the blocks the refusal could not present, and every one of
    // them is an anchor: what the quote names is what the artifact maps.
    let cited = cited(&report.text);
    assert!(cited.contains(&0), "{cited:?}");
    let anchors = anchors(report);
    for bci in &cited {
        assert!(anchors.contains(bci), "the cited BCI {bci} has no anchor");
    }
}

#[test]
fn a_guarded_body_that_branches_is_refused() {
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let recovered = refused(&engine, &fixture, "branching", "jre_guard_body");
    let report = recovered.recovery();
    assert!(!report.text.contains("try ("), "{}", report.text);
    let region = report
        .regions
        .iter()
        .find(|region| region.code == Some("jre_guard_body"))
        .expect("the refusal is a region of its own");
    assert_eq!(
        region.rule.as_ref().map(|rule| rule.rule()),
        Some("twr"),
        "the rule that examined the shape is stated"
    );
    assert!(!region.structured);
}

#[test]
fn a_finally_copy_is_refused_and_the_refusal_says_which_code_it_copies() {
    // `finally` is not a region with a handler: javac copies its code onto every exit path, and the
    // exceptional copy rethrows the exception it stored. Merging the copies into one `finally`
    // restates the source only if they are provably equal — this build does not prove that, so the
    // shape is refused, under a code of its own, and **no rule** is blamed for it.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    for name in ["fin", "catchFinally"] {
        let recovered = refused(&engine, &fixture, name, "jre_guard_finally_copy");
        let report = recovered.recovery();
        assert!(!report.text.contains("try {"), "{name}: {}", report.text);
        assert!(
            !report.text.contains("} finally"),
            "{name}: {}",
            report.text
        );
        let region = report
            .regions
            .iter()
            .find(|region| region.code == Some("jre_guard_finally_copy"))
            .expect("the refusal is a region of its own");
        assert!(
            region.rule.is_none(),
            "{name}: no rule of this build presents a `finally` copy"
        );
        let message = region.message.clone().unwrap_or_default();
        assert!(message.contains("finally"), "{name}: {message}");
        let cited = cited(&report.text);
        let anchors = anchors(report);
        for bci in &cited {
            assert!(
                anchors.contains(bci),
                "{name}: the cited BCI {bci} has no anchor"
            );
        }
    }
}

#[test]
fn a_close_the_exception_path_lacks_is_refused() {
    // The exceptional path's `close` replaced by a no-op of the same width, so every BCI stands:
    // the handler closes the resource on the normal path only, and the rule refuses rather than
    // write a header whose close would run where the bytecode's did not.
    let engine = Engine::new();
    let mut bytes = SAMPLE.to_vec();
    // `aload_0; invokevirtual Res.close:()V` inside the handler, after `astore_1; aload_0; ifnull`.
    bytes = patched(
        &bytes,
        &[0x4c, 0x2a, 0xc6, 0x00, 0x10, 0x2a, 0xb6, 0x00, 0x3d],
        &[0x4c, 0x2a, 0xc6, 0x00, 0x10, 0x2a, 0x00, 0x57, 0x00],
    );
    let fixture = fixture(&engine, &bytes);
    let recovered = refused(&engine, &fixture, "one", "jre_guard_handler");
    let report = recovered.recovery();
    assert!(!report.text.contains("try ("), "{}", report.text);
}

#[test]
fn a_suppression_that_is_the_other_way_round_is_refused() {
    // The two loads before `addSuppressed` swapped: the close's own exception becomes the receiver
    // and the primary the argument — a different program, which the rule refuses because the
    // suppressed relationship it would present is not the one the bytecode performs.
    let engine = Engine::new();
    let bytes = patched(
        SAMPLE,
        &[0x2b, 0x2c, 0xb6, 0x00, 0x42],
        &[0x2c, 0x2b, 0xb6, 0x00, 0x42],
    );
    let fixture = fixture(&engine, &bytes);
    let recovered = refused(&engine, &fixture, "one", "jre_guard_suppressed");
    let report = recovered.recovery();
    assert!(!report.text.contains("try ("), "{}", report.text);
    // The one-resource sample is the first of the members this mutation rewrites.
    let _ = &recovered;
}

#[test]
fn a_handler_range_that_swallows_the_initialisation_is_refused() {
    // Row `[6, 9) → 20` widened to `[5, 9)`: the protected range now begins inside the resource's
    // own initialisation, and the header would move the store it must not move.
    let engine = Engine::new();
    let bytes = patched(
        SAMPLE,
        &[0x00, 0x06, 0x00, 0x09, 0x00, 0x14],
        &[0x00, 0x05, 0x00, 0x09, 0x00, 0x14],
    );
    let fixture = fixture(&engine, &bytes);
    let recovered = refused(&engine, &fixture, "one", "jre_guard_resource_init");
    let report = recovered.recovery();
    assert!(!report.text.contains("try ("), "{}", report.text);
}

#[test]
fn a_monitor_whose_exception_path_does_not_exit_is_refused() {
    // The handler's `monitorexit` replaced by a no-op: the monitor is left held when the body
    // throws, and the rule refuses — writing `synchronized` would drop exactly that exit.
    let engine = Engine::new();
    let bytes = patched(
        SAMPLE,
        &[0x4c, 0x2a, 0xc3, 0x2b, 0xbf],
        &[0x4c, 0x2a, 0x00, 0x2b, 0xbf],
    );
    let fixture = fixture(&engine, &bytes);
    let recovered = refused(&engine, &fixture, "sync", "jre_guard_monitor");
    let report = recovered.recovery();
    assert!(!report.text.contains("synchronized ("), "{}", report.text);
    assert_eq!(
        report
            .regions
            .iter()
            .find(|region| region.code == Some("jre_guard_monitor"))
            .and_then(|region| region.rule.as_ref())
            .map(|rule| rule.rule()),
        Some("monitor")
    );
}

#[test]
fn a_second_entry_into_a_monitor_region_is_refused() {
    // The `goto` after the normal exit sent back to the region's first instruction: the monitor is
    // entered from two places on one path, and the region is refused (the graph itself states why:
    // a loop this subset does not prove) rather than written as one `synchronized` statement.
    let engine = Engine::new();
    let bytes = patched(SAMPLE, &[0xa7, 0x00, 0x08], &[0xa7, 0xff, 0xf5]);
    let fixture = fixture(&engine, &bytes);
    let recovered = recover_with(&engine, &fixture, b"sync", 8);
    let report = recovered.recovery();
    assert!(report.produced(), "{:?}", report.outcome);
    assert_ne!(
        report.representation,
        Representation::Java,
        "{}",
        report.text
    );
    assert!(!report.text.contains("synchronized ("), "{}", report.text);
    assert!(
        report
            .fallbacks
            .iter()
            .any(|fallback| fallback.starts_with("jre_region_")),
        "{:?}",
        report.fallbacks
    );
}

#[test]
fn a_profile_that_does_not_admit_the_header_refuses_it() {
    // `try (T n = …)` is Java 7 syntax: a run that presents the artifact as release 6 does not
    // admit `twr@1`, and the refusal says so instead of writing Java the profile does not have.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let recovered = recover_with(&engine, &fixture, b"one", 6);
    let report = recovered.recovery();
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(
        report.representation,
        Representation::Mixed,
        "{}",
        report.text
    );
    assert_eq!(report.quality, Quality::Fallback);
    assert!(
        report.fallbacks.contains(&"jre_guard_profile"),
        "{:?}",
        report.fallbacks
    );
    assert!(!report.text.contains("try ("), "{}", report.text);
}
