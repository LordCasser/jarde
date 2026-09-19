//! P4 3.1/3.2 acceptance: the versioned plugin plane and the one framework/resource rule.
//!
//! What this file proves through the public API:
//!
//! 1. **a rule is a registered descriptor, and enabling it is naming it by id and version.** A
//!    performed rule answers with its rule version, the entry it read, the range inside that entry
//!    and its own coverage; the declared output schema is the shape the items really have;
//! 2. **an unregistered configuration is `Unsupported`, never an empty answer.** A configuration
//!    this registry does not hold, a version it does not hold, and a configuration registered
//!    without being performed are three distinct refusals, and the control — an entry that really
//!    declares nothing — is `Performed` with a coverage that names the entry it read. That
//!    difference is what keeps "nothing was declared" from being confused with "not read";
//! 3. **the plugin plane leaves the structural plane alone.** The structural answer for the very
//!    same bytes, and the snapshot's own listing, are field for field what they were before the
//!    plugin ran; the structural item has no rule identity, the plugin report has no `relation`
//!    and no `consumers`, and a name the archive does not mention answers nothing there while the
//!    plugin named three declarations from those same bytes;
//! 4. **a budget refusal is a stop with the range it read.** The item budget keeps a published
//!    prefix, the read budget names the entry it did not read, the time budget stops the listing
//!    itself — each with a partial coverage, and each leaving the independent structural query
//!    complete over its own facts;
//! 5. **nothing is executed.** The archive really holds a class file (with a `native` method) and a
//!    manifest whose `Main-Class`, `Launcher-Agent-Class` and `Class-Path` claims name a class it
//!    does not hold and a URL no plane resolves; the plugin reads the configuration and the name it
//!    spells, and its request's own usage shows not one class byte, header or body.
//!
//! Fixtures are real ZIP archives built entry by entry; no framework, and no dependency beyond the
//! ZIP writer and serde the other P4 suites already use.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;
const PUBLIC: u16 = 0x0001;
const SUPER: u16 = 0x0020;
const NATIVE: u16 = 0x0100;
const MAJOR: u16 = 52;

/// The one configuration this plane reads here: a provider-configuration file.
const SERVICES_ENTRY: &[u8] = b"META-INF/services/com.example.Service";
/// The service interface the entry name declares providers for.
const SERVICE_KEY: &[u8] = b"com.example.Service";
/// The declarations, in the syntax of `java.util.ServiceLoader`: a comment, a blank line, a name
/// continued across two lines, and a name this archive does not hold at all.
const SERVICES_CONTENT: &[u8] = b"# the fixture's own provider-configuration file\n\
com.example.ImplA\n\
\n\
com.example.Declared.\n\
   OnlyProvider\n\
com.example.Absent\n";
const PROVIDER_A: &[u8] = b"com.example.ImplA";
const PROVIDER_CONTINUED: &[u8] = b"com.example.Declared.OnlyProvider";
const PROVIDER_ABSENT: &[u8] = b"com.example.Absent";
const MAIN_CLASS_ENTRY: &[u8] = b"com/example/Main.class";
/// A manifest whose launcher and agent claims name classes this archive does not hold, and whose
/// `Class-Path` token is a network URL no plane of this engine resolves.
const MANIFEST: &[u8] = b"Manifest-Version: 1.0\n\
Main-Class: com.example.AbsentMain\n\
Launcher-Agent-Class: com.example.AbsentAgent\n\
Class-Path: https://example.invalid/absent.jar\n";
const CLASS_PATH_TOKEN: &[u8] = b"https://example.invalid/absent.jar";
/// The registered rule this file performs.
const RULE: &str = "service-loader-registrations";
/// A rule this build registers without performing.
const SPRING: &str = "spring-factories";

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 22,
        archive_entries: 10_000,
        entry_bytes: 1 << 22,
        read_bytes: 1 << 22,
        class_bytes: 1 << 22,
        attribute_bytes: 1 << 22,
        output_bytes: 1 << 22,
        code_bytes: 1 << 22,
        result_items: 100_000,
        class_headers: 200,
        method_bodies: 64,
        ir_items: 1_000_000,
        ir_edges: 1_000_000,
        normalization_clones: 10_000,
        nested_depth: 4,
        dependency_depth: 8,
        analysis_steps: 100_000,
        elapsed_millis: u64::MAX,
    }
}

fn zip_of(entries: &[(Vec<u8>, Vec<u8>)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.clone()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer
                .write_all(data)
                .expect("the fixture entry is writable");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

fn push_utf8(out: &mut Vec<u8>, value: &[u8]) {
    out.push(1);
    out.extend_from_slice(
        &u16::try_from(value.len())
            .expect("a fixture name fits u16")
            .to_be_bytes(),
    );
    out.extend_from_slice(value);
}

fn push_class(out: &mut Vec<u8>, name_index: u16) {
    out.push(7);
    out.extend_from_slice(&name_index.to_be_bytes());
}

/// A minimal, well-formed class file (major 52) whose one method is `native`: a real JNI
/// declaration, which this plane must never read.
fn class_file() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0xca, 0xfe, 0xba, 0xbe]);
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&MAJOR.to_be_bytes());
    bytes.extend_from_slice(&7_u16.to_be_bytes());
    push_utf8(&mut bytes, b"com/example/Main");
    push_class(&mut bytes, 1);
    push_utf8(&mut bytes, b"java/lang/Object");
    push_class(&mut bytes, 3);
    push_utf8(&mut bytes, b"nativeCall");
    push_utf8(&mut bytes, b"()V");
    bytes.extend_from_slice(&(PUBLIC | SUPER).to_be_bytes());
    bytes.extend_from_slice(&2_u16.to_be_bytes());
    bytes.extend_from_slice(&4_u16.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    bytes.extend_from_slice(&(PUBLIC | NATIVE).to_be_bytes());
    bytes.extend_from_slice(&5_u16.to_be_bytes());
    bytes.extend_from_slice(&6_u16.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes
}

/// The archive every case reads: a manifest with launcher and agent claims, one
/// provider-configuration file, one class file and a plain resource.
fn archive() -> Vec<u8> {
    zip_of(&[
        (b"META-INF/MANIFEST.MF".to_vec(), MANIFEST.to_vec()),
        (SERVICES_ENTRY.to_vec(), SERVICES_CONTENT.to_vec()),
        (MAIN_CLASS_ENTRY.to_vec(), class_file()),
        (b"docs/readme.txt".to_vec(), b"not a resource".to_vec()),
    ])
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the fixture archive opens")
}

fn enumerate(snapshot: &ArtifactSnapshot) -> EnumerationReport {
    let mut budget = Budget::new(limits());
    Engine::new()
        .enumerate(snapshot, &mut budget)
        .expect("the fixture archive lists")
}

fn selection(id: &str, version: &str) -> PluginSelection {
    PluginSelection {
        id: id.to_string(),
        version: version.to_string(),
    }
}

fn request(snapshot: &ArtifactSnapshot, rules: Vec<PluginSelection>) -> PluginRequest {
    PluginRequest {
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        rules,
        max_items: 0,
    }
}

/// Runs one plugin request under the given limits.
fn run(snapshot: &ArtifactSnapshot, rules: Vec<PluginSelection>, limits: Limits) -> PluginReport {
    let request = request(snapshot, rules);
    let mut budget = Budget::new(limits);
    Engine::new()
        .plugins(snapshot, &request, &mut budget)
        .expect("the request is answered")
}

/// The independent structural query: a different request, its own budget, the P1 facts.
fn structural(
    snapshot: &ArtifactSnapshot,
    relation: QueryRelation,
    target: QueryTarget,
) -> QueryReport {
    let request = QueryRequest {
        relation,
        target,
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Resource]),
        max_items: 0,
        cursor: None,
    };
    let mut budget = Budget::new(limits());
    Engine::new()
        .query(snapshot, &request, &mut budget)
        .expect("the structural query runs")
}

fn symbol(name: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Class {
            owner: JvmBytes(name.as_bytes().to_vec()),
        },
    }
}

fn usage(report: &PluginReport) -> UsageSnapshot {
    match &report.execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage.clone(),
    }
}

fn providers(report: &PluginReport) -> Vec<Vec<u8>> {
    report
        .items()
        .map(|item| match &item.value {
            PluginValue::ServiceProvider { provider, .. } => provider.0.clone(),
        })
        .collect()
}

#[test]
fn a_framework_resource_rule_answers_with_its_rule_version_source_range_and_coverage() {
    let bytes = archive();
    let snapshot = open(&bytes);
    let report = run(&snapshot, vec![selection(RULE, "1")], limits());

    assert_eq!(report.analysis, PluginAnalysis::Performed);
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(report.rules.len(), 1);
    let rule = &report.rules[0];
    assert_eq!(rule.requested, selection(RULE, "1"));
    let descriptor = rule
        .rule
        .expect("this registry holds the rule at the version the request enabled");
    assert_eq!(descriptor.id, RULE);
    assert_eq!(descriptor.rule, PluginRuleVersion::new(RULE, "1"));
    assert_eq!(
        descriptor.rule.to_string(),
        "service-loader-registrations@1"
    );
    assert_eq!(rule.analysis, PluginRuleAnalysis::Performed);
    assert_eq!(rule.returned_items, 3);
    assert!(!rule.has_more);

    // Every item carries the rule, the rule version, the entry it was read from and the range
    // inside that entry that holds the declaration.
    let services = enumerate(&snapshot)
        .entries
        .into_iter()
        .find(|entry| entry.id.raw_name.0 == SERVICES_ENTRY)
        .expect("the archive holds the configuration");
    let mut ranges = Vec::new();
    for item in &rule.items {
        assert_eq!(item.rule, RULE);
        assert_eq!(item.rule_version, PluginRuleVersion::new(RULE, "1"));
        assert_eq!(item.source_entry.ordinal, services.id.ordinal);
        assert_eq!(
            item.source_entry.raw_name,
            ArchiveNameBytes(SERVICES_ENTRY.to_vec())
        );
        match &item.value {
            PluginValue::ServiceProvider { service, provider } => {
                assert_eq!(service.0, SERVICE_KEY);
                ranges.push((
                    provider.0.clone(),
                    usize::try_from(item.matched.start).expect("the fixture range fits usize"),
                    usize::try_from(item.matched.length).expect("the fixture range fits usize"),
                ));
            }
        }
    }
    assert_eq!(
        ranges
            .iter()
            .map(|(name, ..)| name.clone())
            .collect::<Vec<_>>(),
        vec![
            PROVIDER_A.to_vec(),
            PROVIDER_CONTINUED.to_vec(),
            PROVIDER_ABSENT.to_vec()
        ],
        "the declarations are published in file order, the absent one named as declared"
    );
    for (name, start, length) in &ranges {
        let matched = &SERVICES_CONTENT[*start..start + length];
        let joined: Vec<u8> = matched
            .iter()
            .copied()
            .filter(|byte| !byte.is_ascii_whitespace())
            .collect();
        assert_eq!(&joined, name, "the range holds the name it claims to");
    }
    // The first declaration's range is exactly its own bytes: nothing was normalized.
    assert_eq!(
        &SERVICES_CONTENT[ranges[0].1..ranges[0].1 + ranges[0].2],
        PROVIDER_A
    );
    // The continued name's range covers both of its lines, which is what makes it replayable.
    let (_, start, length) = ranges[1];
    let continued = &SERVICES_CONTENT[start..start + length];
    assert!(
        continued.starts_with(b"com.example.Declared."),
        "{continued:?}"
    );
    assert!(continued.contains(&b'\n'), "{continued:?}");

    // The rule's own coverage: the entry it read, and the two dimensions this plane never touches.
    let coverage = &rule.coverage;
    assert_eq!(
        coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(coverage.artifact_structural.scanned.len(), 1);
    assert_eq!(
        coverage.artifact_structural.scanned[0].label,
        format!("plugin:{RULE}:entry:META-INF/services/com.example.Service")
    );
    assert_eq!(coverage.artifact_structural.scanned[0].start, 0);
    assert_eq!(
        coverage.artifact_structural.scanned[0].end,
        SERVICES_CONTENT.len() as u64
    );
    assert!(coverage.artifact_structural.skipped.is_empty());
    assert_eq!(
        coverage.runtime_resolution.state,
        CoverageState::NotRequested
    );
    assert_eq!(coverage.dynamic_analysis.state, CoverageState::NotRequested);

    // The descriptor's declared schema is the shape the items really have: a field added to the
    // item without a schema version is a failing test rather than a drift.
    let item = serde_json::to_value(&rule.items[0]).expect("the item serializes");
    let mut published: Vec<String> = item
        .as_object()
        .expect("an item is an object")
        .keys()
        .cloned()
        .collect();
    published.sort();
    let mut declared: Vec<String> = descriptor
        .schema
        .fields
        .iter()
        .map(|field| (*field).to_string())
        .collect();
    declared.sort();
    assert_eq!(published, declared);
    assert_eq!(descriptor.schema.name, "plugin.service_provider");
    assert_eq!(descriptor.schema.version, 1);
}

#[test]
fn an_unregistered_configuration_is_unsupported_and_never_an_empty_answer() {
    let bytes = archive();
    let snapshot = open(&bytes);

    // A configuration this registry does not hold at all.
    let report = run(&snapshot, vec![selection("yaml-beans", "1")], limits());
    assert_eq!(
        report.analysis,
        PluginAnalysis::NotPerformed {
            code: "plugin_rule_not_registered"
        }
    );
    let rule = &report.rules[0];
    assert!(rule.items.is_empty());
    assert!(rule.rule.is_none(), "no descriptor answers for it");
    assert!(matches!(
        rule.analysis,
        PluginRuleAnalysis::Unsupported {
            code: "plugin_rule_not_registered",
            ..
        }
    ));
    assert_eq!(rule.coverage, Coverage::not_requested());
    assert_eq!(rule.returned_items, 0);
    assert!(!rule.has_more);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "plugin_rule_not_registered"
            && diagnostic.severity == DiagnosticSeverity::Warning
            && diagnostic.provenance.is_none()
    }));

    // A configuration whose id is registered and whose *version* is not: the rule version is part
    // of the identity a result is read under, so this is a refusal too.
    let report = run(&snapshot, vec![selection(RULE, "2")], limits());
    assert_eq!(
        report.analysis,
        PluginAnalysis::NotPerformed {
            code: "plugin_rule_version_not_registered"
        }
    );
    let rule = &report.rules[0];
    assert!(rule.items.is_empty());
    assert!(rule.rule.is_none());
    let message = match &rule.analysis {
        PluginRuleAnalysis::Unsupported { message, .. } => message.clone(),
        other => panic!("expected an unsupported state, read {other:?}"),
    };
    assert!(message.contains("registered at version 1"), "{message}");

    // A configuration this build registers *without* performing: a registry answer with its own
    // reason, and never an omission.
    let report = run(&snapshot, vec![selection(SPRING, "1")], limits());
    assert_eq!(
        report.analysis,
        PluginAnalysis::NotPerformed {
            code: "plugin_rule_unsupported"
        }
    );
    let rule = &report.rules[0];
    assert_eq!(rule.rule.expect("the rule is registered").id, SPRING);
    let reason = match &rule.analysis {
        PluginRuleAnalysis::Unsupported {
            code: "plugin_rule_unsupported",
            message,
        } => message.clone(),
        other => panic!("expected an unsupported state, read {other:?}"),
    };
    assert!(!reason.is_empty());
    assert!(rule.items.is_empty());
    assert_eq!(rule.coverage, Coverage::not_requested());

    // The control: an entry that really declares nothing is `Performed` with no item — and its
    // coverage is what separates it from the three refusals above.
    let empty = open(&zip_of(&[(
        SERVICES_ENTRY.to_vec(),
        b"# only a comment\n\n".to_vec(),
    )]));
    let report = run(&empty, vec![selection(RULE, "1")], limits());
    let rule = &report.rules[0];
    assert_eq!(rule.analysis, PluginRuleAnalysis::Performed);
    assert!(rule.items.is_empty());
    assert_eq!(rule.returned_items, 0);
    assert!(!rule.has_more);
    assert_eq!(
        rule.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(rule.coverage.artifact_structural.scanned.len(), 1);

    // A mixed request: a refusal is one rule's state, and the performed rule still answers.
    let report = run(
        &snapshot,
        vec![selection("yaml-beans", "1"), selection(RULE, "1")],
        limits(),
    );
    assert_eq!(report.analysis, PluginAnalysis::Performed);
    assert_eq!(report.rules.len(), 2);
    assert!(matches!(
        report.rules[0].analysis,
        PluginRuleAnalysis::Unsupported { .. }
    ));
    assert_eq!(report.rules[1].returned_items, 3);
    assert!(report.performed().count() == 1);

    // A scope this rule is not registered over is `NotRequested`, not an empty answer: the
    // snapshot's own root container is the only range this plane reads, and it claims no range of
    // a scope it did not read.
    let mut scoped = request(&snapshot, vec![selection(RULE, "1")]);
    scoped.physical.scope = PhysicalScope::ArtifactTree {
        root_container: ContainerId("root".into()),
    };
    let mut budget = Budget::new(limits());
    let report = Engine::new()
        .plugins(&snapshot, &scoped, &mut budget)
        .expect("the request is answered");
    assert_eq!(
        report.analysis,
        PluginAnalysis::NotPerformed {
            code: "plugin_scope_not_registered"
        }
    );
    let rule = &report.rules[0];
    assert!(matches!(
        rule.analysis,
        PluginRuleAnalysis::NotRequested {
            code: "plugin_scope_not_registered",
            ..
        }
    ));
    assert!(rule.items.is_empty());
    assert_eq!(rule.coverage, Coverage::not_requested());
}

#[test]
fn the_plugin_plane_leaves_the_structural_planes_own_answer_untouched() {
    let bytes = archive();
    let snapshot = open(&bytes);
    let target = symbol("com/example/Service");

    // The structural answer and the snapshot's own listing, before any plugin ran.
    let before = structural(&snapshot, QueryRelation::MentionsSymbol, target.clone());
    let before_listing = enumerate(&snapshot);

    // The plugin really ran on this very snapshot.
    let plugin = run(&snapshot, vec![selection(RULE, "1")], limits());
    assert_eq!(plugin.rules[0].returned_items, 3);

    // The same structural answer, field for field, and the same listing.
    let after = structural(&snapshot, QueryRelation::MentionsSymbol, target);
    let after_listing = enumerate(&snapshot);
    assert_eq!(
        serde_json::to_value(&after).expect("the report serializes"),
        serde_json::to_value(&before).expect("the report serializes"),
        "a plugin run leaves the structural plane's own answer identical"
    );
    assert_eq!(
        serde_json::to_value(&after_listing).expect("the listing serializes"),
        serde_json::to_value(&before_listing).expect("the listing serializes"),
        "and the snapshot's own facts identical"
    );

    // The structural item keeps the resource consumer's own shape, and carries no rule identity.
    assert_eq!(before.items.len(), 1);
    let item = serde_json::to_value(&before.items[0]).expect("the item serializes");
    let keys: Vec<&String> = item
        .as_object()
        .expect("an item is an object")
        .keys()
        .collect();
    for expected in [
        "relation",
        "source",
        "target",
        "consumer",
        "operation",
        "derivation",
        "certainty",
        "resolution",
        "evidence",
    ] {
        assert!(
            keys.iter().any(|key| key.as_str() == expected),
            "the structural item keeps `{expected}`: {keys:?}"
        );
    }
    assert!(
        !keys
            .iter()
            .any(|key| key.contains("rule") || key.contains("plugin")),
        "no plugin stamp on the structural plane: {keys:?}"
    );
    // And nothing of this plane's identity is in the structural answer: the item's evidence is the
    // resource consumer's own, byte for byte — the attribute it carries is the registration key the
    // entry name holds, and neither the plugin's rule id nor its version appears in it.
    assert_eq!(
        before.items[0].evidence.attribute,
        Some(ArchiveNameBytes(SERVICE_KEY.to_vec())),
        "the structural item's evidence is the resource consumer's own"
    );
    assert!(before.items[0].evidence.via.is_empty());
    for spelling in [
        RULE,
        "service-loader-registrations@1",
        "plugin.service_provider",
    ] {
        assert!(
            !serde_json::to_string(&before)
                .expect("the report serializes")
                .contains(spelling),
            "the plugin's identity `{spelling}` appears on the structural plane"
        );
    }

    // The two planes are separate in both directions: the structural scan is target-driven and
    // answers nothing for a name these bytes do not mention, while the plugin named three
    // declarations from those same bytes.
    let unrelated = structural(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/NotInFixture"),
    );
    assert!(unrelated.items.is_empty());
    assert_eq!(providers(&plugin).len(), 3);
    let plugin_json = serde_json::to_value(&plugin).expect("the report serializes");
    assert!(
        plugin_json.get("relation").is_none() && plugin_json.get("consumers").is_none(),
        "the plugin report is not a query report: {plugin_json}"
    );
    assert!(plugin_json.get("items").is_none(), "{plugin_json}");

    // The manifest's `Class-Path` token is really in the archive: the structural plane answers it
    // as a relative *literal* it never resolves, and the plugin never reads the manifest at all.
    let manifest = structural(
        &snapshot,
        QueryRelation::LiteralValue,
        QueryTarget::Literal {
            value: LiteralValue::String {
                value: JvmBytes(CLASS_PATH_TOKEN.to_vec()),
            },
        },
    );
    assert_eq!(manifest.items.len(), 1);
    assert!(
        !serde_json::to_string(&plugin)
            .expect("the report serializes")
            .contains("example.invalid"),
        "no manifest token is a declaration of this rule"
    );
}

#[test]
fn a_budget_refusal_is_a_stop_with_the_range_it_read() {
    let bytes = archive();
    let snapshot = open(&bytes);
    let entries = enumerate(&snapshot);
    assert_eq!(entries.entries.len(), 4, "the fixture holds four entries");

    // (a) The item budget. Listing the four entries costs one `result_items` each, so a limit of
    //     `entries + 2` publishes exactly two declarations and refuses the third: the answer is the
    //     prefix that was published, and the coverage names the range it read to produce it.
    let mut tight = limits();
    tight.result_items = u64::try_from(entries.entries.len()).expect("four fits u64") + 2;
    let report = run(&snapshot, vec![selection(RULE, "1")], tight.clone());
    assert_eq!(report.analysis, PluginAnalysis::Performed);
    let rule = &report.rules[0];
    assert_eq!(rule.returned_items, 2);
    assert!(rule.has_more);
    assert_eq!(
        rule.analysis,
        PluginRuleAnalysis::Skipped {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            }
        }
    );
    assert_eq!(
        rule.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        rule.coverage.artifact_structural.scanned.len(),
        1,
        "the range it did read is named"
    );
    assert_eq!(
        rule.coverage.artifact_structural.scanned[0].end,
        SERVICES_CONTENT.len() as u64
    );
    assert!(rule.coverage.artifact_structural.skipped.is_empty());
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "budget_exceeded_result_items")
    );

    // (a2) A stop ends the request's scan: a rule that never started says so instead of claiming a
    //      range of its own.
    let report = run(
        &snapshot,
        vec![selection(RULE, "1"), selection(RULE, "1")],
        tight.clone(),
    );
    assert_eq!(report.rules.len(), 2);
    assert_eq!(report.rules[0].returned_items, 2);
    assert!(matches!(
        report.rules[1].analysis,
        PluginRuleAnalysis::NotRequested {
            code: "plugin_request_stopped",
            ..
        }
    ));
    assert!(report.rules[1].items.is_empty());
    assert_eq!(report.rules[1].coverage, Coverage::not_requested());
    assert_eq!(report.analysis, PluginAnalysis::Performed);

    // (b) The read budget. The configuration itself cannot be materialized, so the rule stops
    //     before it reads a byte and names the entry it did not read.
    let mut tiny = limits();
    tiny.entry_bytes = 0;
    let report = run(&snapshot, vec![selection(RULE, "1")], tiny);
    let rule = &report.rules[0];
    assert!(rule.items.is_empty());
    assert!(rule.has_more);
    assert_eq!(
        rule.analysis,
        PluginRuleAnalysis::Skipped {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::EntryBytes
            }
        }
    );
    assert_eq!(
        rule.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(rule.coverage.artifact_structural.scanned.is_empty());
    assert_eq!(rule.coverage.artifact_structural.skipped.len(), 1);
    assert_eq!(
        rule.coverage.artifact_structural.skipped[0].end,
        SERVICES_CONTENT.len() as u64
    );
    assert!(matches!(report.execution, ExecutionReport::Partial { .. }));
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "budget_exceeded_entry_bytes")
        .expect("the stop names the dimension that refused");
    assert!(matches!(
        diagnostic
            .provenance
            .as_ref()
            .map(|provenance| &provenance.location),
        Some(Location::Entry { .. })
    ));

    // (c) The time budget. The listing itself is refused before it names an entry, so every rule
    //     reads a prefix of an unknown range: partial coverage, `has_more`, and the request's own
    //     execution names the wall-clock limit.
    let mut expired = limits();
    expired.elapsed_millis = 0;
    let report = run(&snapshot, vec![selection(RULE, "1")], expired);
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis
            },
            ..
        }
    ));
    let rule = &report.rules[0];
    assert!(rule.items.is_empty());
    assert!(rule.has_more);
    assert_eq!(rule.analysis, PluginRuleAnalysis::Performed);
    assert_eq!(
        rule.coverage.artifact_structural.state,
        CoverageState::Partial
    );

    // (d) The independent structural query is a different request with its own budget: it completes
    //     over its own facts whether or not the plugin request was refused.
    let structural_report = structural(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Service"),
    );
    assert_eq!(structural_report.items.len(), 1);
    assert!(matches!(
        structural_report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        structural_report
            .coverage
            .dimensions
            .artifact_structural
            .state,
        CoverageState::CompleteWithinSchema
    );
}

#[test]
fn nothing_is_executed_and_a_claim_that_names_an_absent_class_is_only_a_name() {
    let bytes = archive();
    let snapshot = open(&bytes);
    let entries = enumerate(&snapshot);

    // The archive really holds a class file this engine can read — a `native` method among its
    // declarations — and no class the manifest or the configuration names.
    let main = entries
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == MAIN_CLASS_ENTRY)
        .expect("the fixture holds a class file");
    let mut budget = Budget::new(limits());
    Engine::new()
        .inspect_header(
            &snapshot,
            ClassTarget::Entry(main),
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's class file is really readable");
    assert!(
        !entries
            .entries
            .iter()
            .any(|entry| entry.id.raw_name.0.ends_with(b"Absent.class")),
        "no class the fixture's claims name is part of the archive"
    );

    let report = run(&snapshot, vec![selection(RULE, "1")], limits());

    // The declarations are exactly the three names the configuration spells — the absent one
    // included, because the name is all this plane reads.
    assert_eq!(
        providers(&report),
        vec![
            PROVIDER_A.to_vec(),
            PROVIDER_CONTINUED.to_vec(),
            PROVIDER_ABSENT.to_vec()
        ]
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        report.rules[0].coverage.dynamic_analysis.state,
        CoverageState::NotRequested
    );

    // Nothing of the manifest is an item of this rule, and no claim it makes was followed.
    let json = serde_json::to_string(&report).expect("the report serializes");
    for absent in ["AbsentMain", "AbsentAgent", "example.invalid"] {
        assert!(!json.contains(absent), "the report names {absent}: {json}");
    }

    // This request's own usage shows what it did: it listed the archive, read the one
    // configuration, and touched not one class byte, header, body or output buffer.
    let usage = usage(&report);
    for dimension in [
        CountedBudgetDimension::ClassBytes,
        CountedBudgetDimension::ClassHeaders,
        CountedBudgetDimension::MethodBodies,
        CountedBudgetDimension::AttributeBytes,
        CountedBudgetDimension::CodeBytes,
        CountedBudgetDimension::AnalysisSteps,
        CountedBudgetDimension::IrItems,
        CountedBudgetDimension::IrEdges,
        CountedBudgetDimension::NormalizationClones,
        CountedBudgetDimension::OutputBytes,
    ] {
        assert_eq!(
            usage.counted_usage(dimension),
            0,
            "this request charged {dimension:?}"
        );
    }
    assert_eq!(
        usage.counted_usage(CountedBudgetDimension::EntryBytes),
        SERVICES_CONTENT.len() as u64,
        "the only content this request read is the configuration"
    );
    // The listing charged one `archive_entries` per central-directory record, and locating the one
    // entry this request read re-scanned its way to that ordinal: nothing else was opened.
    let services = entries
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == SERVICES_ENTRY)
        .expect("the archive holds the configuration");
    assert_eq!(
        usage.counted_usage(CountedBudgetDimension::ArchiveEntries),
        u64::try_from(entries.entries.len()).expect("four fits u64") + services.id.ordinal + 1
    );
}
