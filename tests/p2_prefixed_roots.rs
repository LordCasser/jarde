//! `bind-prefixed-load-roots` 1.2/1.3: the prefix contract of the load-root model.
//!
//! A container root declares a physical container (its origin) and a raw entry prefix, and a
//! class name is looked up as `prefix + internal_name + ".class"`, byte for byte. This file holds
//! the evidence that the composition is exact — no trimming, no URL decoding, no case folding, no
//! `.`/`..`/backslash folding, no directory entry required — and that the *binding* half of the
//! lookup walks the very same declared positions: the candidate's own `this_class` has to agree
//! with the requested name, the driver/caller path verifies its claim through the same roots, and
//! changing a prefix is a new environment that reuses no verdict.
//!
//! Every fixture is built in memory by this file itself (the ZIP writer is the repository's own
//! `rawzip` dev-dependency), so no new checked-in bytes and no compiler are involved. The fixtures
//! are deliberately *honest artifacts*: every class entry declares in its header the name its path
//! states, and the tests that want a disagreement build one on purpose.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;

/// Limits that fund the dimensions a resolution request really spends.
fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        output_bytes: 1 << 20,
        result_items: 10_000,
        class_headers: 100,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

// ---------------------------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------------------------

fn u16b(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn u32b(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

/// The smallest class file the reader accepts with a `run()V` method, a `Code` attribute and one
/// `return`, and `this_class` spelled as the caller states it.
///
/// The pool is filled in declaration order, so the indexes below are the ones the bytes really
/// use: `#1`/`#2` the two names, `#3`..`#5` the member text, `#6`/`#7` the two `CONSTANT_Class`
/// entries. The name is written as raw bytes, exactly as the caller passes them: a fixture that
/// means a name the reader cannot decode is the caller's business, not this builder's.
fn class_bytes(this_class: &[u8], major: u16) -> Vec<u8> {
    let names: [&[u8]; 2] = [this_class, b"java/lang/Object"];
    let mut pool = Vec::new();
    for name in names {
        pool.push(1_u8); // CONSTANT_Utf8 #1..#2
        u16b(
            &mut pool,
            u16::try_from(name.len()).expect("fixture name fits u16"),
        );
        pool.extend_from_slice(name);
    }
    for text in [b"run".as_slice(), b"()V".as_slice(), b"Code".as_slice()] {
        pool.push(1_u8); // CONSTANT_Utf8 #3..#5
        u16b(
            &mut pool,
            u16::try_from(text.len()).expect("fixture text fits u16"),
        );
        pool.extend_from_slice(text);
    }
    pool.extend_from_slice(&[7, 0, 1]); // #6 the class entry of this_class
    pool.extend_from_slice(&[7, 0, 2]); // #7 the class entry of java/lang/Object

    // max_stack 0, max_locals 1, one `return`, no handlers, no code attributes.
    let code: &[u8] = &[0, 0, 0, 1, 0, 0, 0, 1, 0xb1, 0, 0, 0, 0];
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, major);
    u16b(&mut bytes, 8); // constant_pool_count: #1..#7
    bytes.extend_from_slice(&pool);
    u16b(&mut bytes, 0x0021); // ACC_PUBLIC | ACC_SUPER
    u16b(&mut bytes, 6); // this_class -> #6
    u16b(&mut bytes, 7); // super_class -> #7
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    u16b(&mut bytes, 1); // methods
    u16b(&mut bytes, 0x0001); // ACC_PUBLIC
    u16b(&mut bytes, 3); // name_index -> "run"
    u16b(&mut bytes, 4); // descriptor_index -> "()V"
    u16b(&mut bytes, 1); // method attributes
    u16b(&mut bytes, 5); // attribute_name_index -> "Code"
    u32b(
        &mut bytes,
        u32::try_from(code.len()).expect("fixture code length fits u32"),
    );
    bytes.extend_from_slice(code);
    u16b(&mut bytes, 0); // class attributes
    bytes
}

/// One stored ZIP with the given entries, in the order given.
///
/// The order is the central-directory order the reader publishes as ordinals, which is why the
/// tests that care about declaration order versus scan order place their entries deliberately.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
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

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut Budget::new(limits()))
        .expect("the fixture snapshot opens")
}

/// The root container of a snapshot: the one origin a fresh ZIP establishes.
fn root_origin(snapshot: &ArtifactSnapshot) -> ContainerOrigin {
    ContainerOrigin {
        snapshot: snapshot.id().clone(),
        root_container: ContainerId("root".into()),
        steps: Vec::new(),
    }
}

/// One declared position: a container with a raw entry prefix.
fn container_root(snapshot: &ArtifactSnapshot, prefix: &[u8]) -> LoadRoot {
    LoadRoot::Container {
        origin: root_origin(snapshot),
        prefix: ArchiveNameBytes(prefix.to_vec()),
    }
}

/// The declared position of one nested container, addressed by the leaf entry name that reaches
/// it. The origin comes from the explicit artifact-tree enumeration, which is the public way a
/// caller learns one.
fn nested_root(snapshot: &ArtifactSnapshot, leaf: &[u8]) -> LoadRoot {
    let mut budget = Budget::new(limits());
    let tree = Engine::new()
        .enumerate_artifact_tree(snapshot, &mut budget)
        .expect("the fixture tree enumerates");
    let origin = tree
        .containers
        .iter()
        .filter(|container| {
            container
                .origin
                .steps
                .last()
                .is_some_and(|step| step.via_raw_name.0 == leaf)
        })
        .map(|container| container.origin.clone())
        .next()
        .unwrap_or_else(|| {
            panic!(
                "the fixture names exactly one container with leaf \"{}\"",
                String::from_utf8_lossy(leaf)
            )
        });
    LoadRoot::Container {
        origin,
        prefix: ArchiveNameBytes(Vec::new()),
    }
}

fn domain(
    loader: &str,
    parent: Option<&str>,
    delegation: DelegationPolicy,
    roots: Vec<LoadRoot>,
) -> LoadDomain {
    LoadDomain {
        loader: LoaderId(loader.to_string()),
        parent_loader: parent.map(|parent| LoaderId(parent.to_string())),
        delegation,
        roots,
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    }
}

fn environment(
    runtime_snapshot: &ArtifactSnapshot,
    caller: LoadDomain,
    domains: Vec<LoadDomain>,
) -> ResolutionEnvironment {
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: runtime_snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: caller,
        },
        domains,
        providers: Vec::new(),
    }
}

/// The simplest environment the validator accepts: one `app` loader whose roots are `roots`.
fn single_loader(
    runtime_snapshot: &ArtifactSnapshot,
    roots: Vec<LoadRoot>,
) -> ResolutionEnvironment {
    let caller = domain("app", None, DelegationPolicy::ParentFirst, roots);
    environment(runtime_snapshot, caller.clone(), vec![caller])
}

fn class_request(
    environment: ResolutionEnvironment,
    loader: &str,
    class_name: &[u8],
) -> ResolutionRequest {
    ResolutionRequest {
        environment,
        target: SymbolRef::Class {
            owner: JvmBytes(class_name.to_vec()),
        },
        use_kind: ReferenceUse::ClassReference,
        caller: CallerContext {
            loader: LoaderId(loader.to_string()),
            enclosing: None,
        },
        dispatch: None,
    }
}

fn resolve(content: &[ArtifactSnapshot], request: &ResolutionRequest) -> ResolutionReport {
    resolve_with(content, request, None).0
}

fn resolve_with(
    content: &[ArtifactSnapshot],
    request: &ResolutionRequest,
    cache: Option<&FactsCache>,
) -> (ResolutionReport, UsageSnapshot) {
    let mut budget = match cache {
        Some(cache) => Budget::new(limits()).with_facts_cache(cache.clone()),
        None => Budget::new(limits()),
    };
    let report = Engine::new()
        .resolve_symbol(content, request, &mut budget)
        .expect("a legal request is answered, not raised");
    (report, budget.usage())
}

/// The entry a resolved class definition really lives at, as the report publishes it.
fn resolved_entry(report: &ResolutionReport) -> &PhysicalEntryId {
    report
        .resolved
        .as_ref()
        .expect("the report resolved a class")
        .definition
        .entry()
        .expect("the definition is an archive entry")
}

fn diagnostic_codes(report: &ResolutionReport) -> Vec<String> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.clone())
        .collect()
}

// ---------------------------------------------------------------------------------------------
// 1.1 — the shape of the model itself
// ---------------------------------------------------------------------------------------------

/// The three shapes round-trip through JSON, and the spellings that are gone or unknown are
/// refused.
///
/// The prefix is raw bytes and its JSON form is an array of octets, never lossy text, so a
/// non-UTF-8 prefix survives the wire unchanged. The old model's `snapshot`/`artifact_tree`
/// variants are not aliases of the new ones: they no longer deserialize, which is what "one
/// migration, no compatibility layer" means for a caller holding an old document. And a prefix is
/// part of the declaration's identity: two roots that differ in nothing else but their prefix are
/// two different roots.
#[test]
fn the_root_model_round_trips_and_refuses_the_old_and_unknown_spellings() {
    let snapshot = SnapshotId("snap".to_string());
    let origin = ContainerOrigin {
        snapshot: snapshot.clone(),
        root_container: ContainerId("root".into()),
        steps: Vec::new(),
    };
    let roots = [
        LoadRoot::StandaloneClass {
            snapshot: snapshot.clone(),
        },
        LoadRoot::Container {
            origin: origin.clone(),
            prefix: ArchiveNameBytes(Vec::new()),
        },
        LoadRoot::Container {
            origin,
            prefix: ArchiveNameBytes(vec![0xff, b'a', b'/']),
        },
        LoadRoot::External {
            id: "host-jdk".to_string(),
        },
    ];
    for root in &roots {
        let json = serde_json::to_value(root).expect("a root serializes");
        let round_trip: LoadRoot = serde_json::from_value(json.clone()).expect("and reads back");
        assert_eq!(&round_trip, root, "the document is the value: {json}");
    }
    let raw = serde_json::to_value(&roots[2]).expect("the raw prefix serializes");
    assert_eq!(
        raw["prefix"],
        serde_json::json!([255, 97, 47]),
        "a prefix travels as its octets"
    );

    for old in [
        serde_json::json!({"kind": "snapshot", "snapshot": "snap"}),
        serde_json::json!({"kind": "artifact_tree", "root": {"snapshot": "snap", "root_container": "root", "steps": []}}),
        serde_json::json!({"kind": "war_root", "origin": {"snapshot": "snap", "root_container": "root", "steps": []}}),
    ] {
        assert!(
            serde_json::from_value::<LoadRoot>(old.clone()).is_err(),
            "`{old}` is not a variant of this model"
        );
    }
    assert_ne!(
        roots[1], roots[2],
        "a prefix is part of the declaration, not a spelling of it"
    );
}

/// A root that names the other physical shape is refused where it is read.
///
/// A standalone CLASS root is one whole class file and a container root is a ZIP; declaring the
/// wrong one cannot be satisfied, and the refusal says which declaration does not describe the
/// provided content instead of searching nothing and answering `Missing`.
#[test]
fn a_root_that_names_the_other_physical_shape_is_refused_where_it_is_read() {
    let class = class_bytes(b"p/S", 52);
    let standalone = open(class.clone());
    let zip = open(zip_of(&[(b"p/S.class", &class)]));

    let on_class = resolve(
        std::slice::from_ref(&standalone),
        &class_request(
            single_loader(&standalone, vec![container_root(&standalone, b"")]),
            "app",
            b"p/S",
        ),
    );
    assert!(on_class.state.is_none() && on_class.resolved.is_none());
    assert_eq!(diagnostic_codes(&on_class), vec!["not_zip"]);

    let on_zip = resolve(
        std::slice::from_ref(&zip),
        &class_request(
            single_loader(
                &zip,
                vec![LoadRoot::StandaloneClass {
                    snapshot: zip.id().clone(),
                }],
            ),
            "app",
            b"p/S",
        ),
    );
    assert!(on_zip.state.is_none() && on_zip.resolved.is_none());
    assert_eq!(diagnostic_codes(&on_zip), vec!["not_standalone_class"]);
}

// ---------------------------------------------------------------------------------------------
// 1.2 — the exact byte lookup
// ---------------------------------------------------------------------------------------------

/// A WAR whose class layer has no directory entry at all still binds through its declared prefix.
///
/// The ZIP holds one class entry and nothing else: no `WEB-INF/classes/` record exists, because a
/// directory entry is a convention of some writers and never what makes an entry reachable. The
/// declared position is the container plus the prefix, and the result keeps the entry's whole
/// physical identity: the raw name still spells the prefixed path, the ordinal is the directory's,
/// and the definition belongs to the provided snapshot.
#[test]
fn a_prefix_binds_a_war_class_without_a_directory_entry() {
    let class = class_bytes(b"com/demo/A", 52);
    let snapshot = open(zip_of(&[(b"WEB-INF/classes/com/demo/A.class", &class)]));
    let mut budget = Budget::new(limits());
    let listed = Engine::new()
        .enumerate(&snapshot, &mut budget)
        .expect("the fixture lists");
    assert!(
        listed
            .entries
            .iter()
            .all(|entry| entry.id.raw_name.0 != b"WEB-INF/classes/"),
        "the premise: the archive holds no directory entry for the prefix"
    );

    let report = resolve(
        std::slice::from_ref(&snapshot),
        &class_request(
            single_loader(
                &snapshot,
                vec![container_root(&snapshot, b"WEB-INF/classes/")],
            ),
            "app",
            b"com/demo/A",
        ),
    );
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let entry = resolved_entry(&report);
    assert_eq!(
        entry.raw_name.0, b"WEB-INF/classes/com/demo/A.class",
        "the physical raw name is the prefixed path, not the logical name"
    );
    assert_eq!(entry.ordinal, 0, "the ordinal is the directory's own");
    assert_eq!(entry.origin, root_origin(&snapshot));
    assert_eq!(
        report.resolved.as_ref().unwrap().definition.snapshot(),
        snapshot.id(),
        "the definition belongs to the content the request provided"
    );
    assert_eq!(
        report.reads.len(),
        1,
        "one candidate decided the demand: {:?}",
        report.reads
    );
    assert_eq!(
        report.reads[0].definition,
        report.resolved.as_ref().unwrap().definition
    );
    assert_eq!(report.reads[0].reason, ReadReason::RequestedDefinition);
    assert_eq!(report.environment_problems, Vec::new());
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        report.environment_identity.content,
        vec![snapshot.id().clone()]
    );
}

/// An empty prefix, and a layout node on its own, bind nothing.
///
/// The physical evidence the tree enumeration publishes really names `WEB-INF/classes/` — the
/// container and the prefix — and the caller still has to *declare* that position: a root that
/// names the same container with an empty prefix looks for `com/demo/A.class` at the container
/// root and finds nothing. Layout detection is evidence, not a load strategy.
#[test]
fn an_empty_prefix_and_a_layout_node_alone_bind_nothing() {
    let class = class_bytes(b"com/demo/A", 52);
    let snapshot = open(zip_of(&[(b"WEB-INF/classes/com/demo/A.class", &class)]));

    let mut budget = Budget::new(limits());
    let tree = Engine::new()
        .enumerate_artifact_tree(&snapshot, &mut budget)
        .expect("the fixture tree enumerates");
    let layer = tree
        .layout_nodes
        .iter()
        .find(|node| node.kind == LayoutNodeKind::WarClasses)
        .expect("the physical layout evidence names the WAR classes layer");
    let LayoutNodeSource::Prefix {
        container, prefix, ..
    } = &layer.source
    else {
        panic!("a class layer is a prefix node: {:?}", layer.source)
    };
    assert_eq!(prefix.0, b"WEB-INF/classes/");
    assert_eq!(container, &root_origin(&snapshot));

    let unbound = resolve(
        std::slice::from_ref(&snapshot),
        &class_request(
            single_loader(&snapshot, vec![container_root(&snapshot, b"")]),
            "app",
            b"com/demo/A",
        ),
    );
    assert_eq!(
        unbound.state,
        Some(ResolutionState::Missing),
        "the layout node does not add the prefix the caller left out"
    );
    assert!(unbound.resolved.is_none());
    assert!(unbound.reads.is_empty());

    let bound = resolve(
        std::slice::from_ref(&snapshot),
        &class_request(
            single_loader(&snapshot, vec![container_root(&snapshot, &prefix.0)]),
            "app",
            b"com/demo/A",
        ),
    );
    assert_eq!(bound.state, Some(ResolutionState::Resolved));
}

/// A non-empty prefix that does not end with `/` is an invalid environment declaration.
///
/// The refusal is the closed-set environment problem `invalid_root_prefix`, located at the root
/// that declared it, and it is decided from the declaration alone: the report performs nothing
/// and reads nothing. The engine does not add the missing separator (which would bind a class the
/// caller never declared), and the same declaration *with* the separator binds it.
#[test]
fn an_illegal_prefix_is_an_environment_problem_and_never_guessed_around() {
    let class = class_bytes(b"com/demo/A", 52);
    let snapshot = open(zip_of(&[(
        b"WEB-INF/classes/com/demo/A.class",
        class.as_slice(),
    )]));

    let illegal = class_request(
        single_loader(
            &snapshot,
            vec![container_root(&snapshot, b"WEB-INF/classes")],
        ),
        "app",
        b"com/demo/A",
    );
    let (problems, _) = validate_environment(std::slice::from_ref(&snapshot), &illegal.environment);
    assert_eq!(
        problems
            .iter()
            .map(|problem| problem.code.as_str())
            .collect::<Vec<_>>(),
        vec!["invalid_root_prefix"],
        "the prefix shape is the one declaration-only refusal of a container root"
    );
    assert_eq!(
        problems[0].subject,
        EnvironmentSubject::Root {
            loader: LoaderId("app".to_string()),
            index: 0
        }
    );
    assert!(
        problems[0].message.contains("WEB-INF/classes\""),
        "the refusal quotes the declared prefix: {}",
        problems[0].message
    );

    let report = resolve(std::slice::from_ref(&snapshot), &illegal);
    assert_eq!(report.analysis, ResolutionAnalysis::NotPerformed);
    assert!(report.state.is_none());
    assert!(
        report.reads.is_empty(),
        "a rejected environment reads nothing"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["invalid_root_prefix", "resolution_not_implemented"],
        "the environment problem, and the honest unavailable state beside it"
    );
    assert_eq!(
        report.environment_problems[0].code,
        EnvironmentProblemCode::InvalidRootPrefix
    );

    let legal = resolve(
        std::slice::from_ref(&snapshot),
        &class_request(
            single_loader(
                &snapshot,
                vec![container_root(&snapshot, b"WEB-INF/classes/")],
            ),
            "app",
            b"com/demo/A",
        ),
    );
    assert_eq!(
        legal.state,
        Some(ResolutionState::Resolved),
        "the same declaration with the boundary binds: the refusal is the shape, not the name"
    );
}

/// The composed name is raw bytes: case, `.`, `..`, backslashes, non-UTF-8 bytes and multi-byte
/// sequences are all ordinary bytes, and only an exact match is a candidate.
///
/// Every entry below declares in its own header exactly the name its path states, so a resolution
/// that succeeds really matched the bytes and a `Missing` really matched nothing — nothing here is
/// explained by a name comparison inside the class file.
#[test]
fn raw_name_bytes_are_matched_exactly_and_not_normalized() {
    let upper = class_bytes(b"p/A", 52);
    let lower = class_bytes(b"p/a", 52);
    let dotted = class_bytes(b"p/./A", 52);
    let dotted_dot = class_bytes(b"p/../A", 52);
    let backslash = class_bytes(b"p\\A", 52);
    let snapshot = open(zip_of(&[
        (b"p/A.class", &upper),
        (b"p/a.class", &lower),
        (b"p/./A.class", &dotted),
        (b"p/../A.class", &dotted_dot),
        (b"p\\A.class", &backslash),
    ]));
    let resolve_name = |name: &[u8]| {
        resolve(
            std::slice::from_ref(&snapshot),
            &class_request(
                single_loader(&snapshot, vec![container_root(&snapshot, b"")]),
                "app",
                name,
            ),
        )
    };

    assert_eq!(
        resolved_entry(&resolve_name(b"p/A")).raw_name.0,
        b"p/A.class"
    );
    assert_eq!(
        resolved_entry(&resolve_name(b"p/a")).raw_name.0,
        b"p/a.class",
        "case is a byte, not a style"
    );
    assert_eq!(
        resolve_name(b"P/A").state,
        Some(ResolutionState::Missing),
        "no case folding"
    );
    assert_eq!(
        resolved_entry(&resolve_name(b"p/./A")).raw_name.0,
        b"p/./A.class",
        "a dot segment is an ordinary byte: the dotted entry is its own definition"
    );
    assert_eq!(
        resolve_name(b"p/../A").state,
        Some(ResolutionState::Resolved)
    );
    assert_ne!(
        resolved_entry(&resolve_name(b"p/../A")).raw_name.0,
        b"p/./A.class",
        "`.` and `..` are not folded into each other"
    );
    assert_eq!(
        resolved_entry(&resolve_name(b"p\\A")).raw_name.0,
        b"p\\A.class",
        "a backslash is an ordinary byte, not a separator"
    );
    assert_ne!(
        resolved_entry(&resolve_name(b"p\\A")).raw_name.0,
        b"p/A.class"
    );
    assert_eq!(
        resolve_name(b"p/A.class").state,
        Some(ResolutionState::Missing),
        "the suffix is composed, never supplied by the caller"
    );
}

/// Non-UTF-8 and multi-byte bytes travel through the prefix and the name without transcoding.
///
/// One class sits under a prefix whose bytes are not UTF-8 at all; the declared position spells
/// those very bytes and binds it. A prefix that spells the same *character* the UTF-8 way reaches
/// nothing — there is no decoding, so there is no spelling that "means the same" — and the same
/// holds for an overlong spelling of a multi-byte name.
#[test]
fn raw_non_utf8_and_multibyte_bytes_are_not_transcoded() {
    let raw_prefix: &[u8] = b"WEB-INF/\xffclasses/";
    let utf8_prefix: &[u8] = b"WEB-INF/\xc3\xbfclasses/";
    let class = class_bytes(b"p/S", 52);
    let mut raw_entry = raw_prefix.to_vec();
    raw_entry.extend_from_slice(b"p/S.class");
    let mut utf8_entry = utf8_prefix.to_vec();
    utf8_entry.extend_from_slice(b"p/S.class");

    let multibyte_name = b"\xc3\xbc/A";
    let multibyte_class = class_bytes(multibyte_name, 52);
    let snapshot = open(zip_of(&[
        (raw_entry.as_slice(), class.as_slice()),
        (utf8_entry.as_slice(), class.as_slice()),
        (b"\xc3\xbc/A.class", &multibyte_class),
    ]));

    let raw = resolve(
        std::slice::from_ref(&snapshot),
        &class_request(
            single_loader(&snapshot, vec![container_root(&snapshot, raw_prefix)]),
            "app",
            b"p/S",
        ),
    );
    assert_eq!(raw.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_entry(&raw).raw_name.0,
        raw_entry,
        "the definition keeps the raw prefix bytes"
    );

    let utf8 = resolve(
        std::slice::from_ref(&snapshot),
        &class_request(
            single_loader(&snapshot, vec![container_root(&snapshot, utf8_prefix)]),
            "app",
            b"p/S",
        ),
    );
    assert_eq!(utf8.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_entry(&utf8).raw_name.0,
        utf8_entry,
        "the UTF-8 spelling is another position with its own entry, not the same name decoded"
    );

    let escaped_spelling = resolve(
        std::slice::from_ref(&snapshot),
        &class_request(
            single_loader(
                &snapshot,
                vec![container_root(&snapshot, b"WEB-INF/\\xffclasses/")],
            ),
            "app",
            b"p/S",
        ),
    );
    assert_eq!(
        escaped_spelling.state,
        Some(ResolutionState::Missing),
        "the display escape of a byte is text, never a spelling of the byte"
    );

    let multibyte = resolve(
        std::slice::from_ref(&snapshot),
        &class_request(
            single_loader(&snapshot, vec![container_root(&snapshot, b"")]),
            "app",
            multibyte_name,
        ),
    );
    assert_eq!(multibyte.state, Some(ResolutionState::Resolved));
    assert_eq!(resolved_entry(&multibyte).raw_name.0, b"\xc3\xbc/A.class");
    let overlong = resolve(
        std::slice::from_ref(&snapshot),
        &class_request(
            single_loader(&snapshot, vec![container_root(&snapshot, b"")]),
            "app",
            b"\xc1\xbc/A",
        ),
    );
    assert_eq!(
        overlong.state,
        Some(ResolutionState::Missing),
        "an overlong spelling of the same character is another name"
    );
}

/// A candidate whose header declares another name than the path it was found under is refused at
/// that candidate, and the search does not continue into a later position holding the real class.
///
/// The entry `p/A.class` holds bytes declaring `p/Other`; a second root holds an honest `p/A`.
/// The report names the candidate it read — its origin and both names — and publishes no
/// resolution at all: hiding the disagreement by searching the next position would bind a class
/// this position never proved, and rewriting the demanded name to match would make the artifact
/// look right without making it right.
#[test]
fn a_candidate_that_declares_another_name_is_refused_at_its_own_origin() {
    let liar = class_bytes(b"p/Other", 52);
    let honest = class_bytes(b"p/A", 52);
    let first = open(zip_of(&[(b"p/A.class", &liar)]));
    let second = open(zip_of(&[(b"p/A.class", &honest)]));
    let content = [first.clone(), second.clone()];

    let report = resolve(
        &content,
        &class_request(
            single_loader(
                &first,
                vec![container_root(&first, b""), container_root(&second, b"")],
            ),
            "app",
            b"p/A",
        ),
    );

    assert!(report.state.is_none() && report.resolved.is_none());
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_definition_name_mismatch"],
        "the mismatch is its own locatable diagnostic"
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Error { ref code },
            ..
        } if code == "resolution_definition_name_mismatch"
    ));
    let refusal = &report.diagnostics[0];
    assert_eq!(refusal.severity, DiagnosticSeverity::Error);
    assert!(
        refusal.message.contains("\"p/A.class\"")
            && refusal.message.contains("`p/Other`")
            && refusal.message.contains("`p/A`"),
        "the refusal names the candidate, the name its bytes declare and the name that was \
         requested: {}",
        refusal.message
    );
    assert!(
        refusal.message.contains("root 0 of loader `app`"),
        "the refusal names the declared position it happened at: {}",
        refusal.message
    );
    let read = report
        .reads
        .iter()
        .find(|read| read.definition.snapshot() == first.id())
        .expect("the candidate that disagreed was read and recorded");
    assert_eq!(read.reason, ReadReason::RequestedDefinition);
    assert_eq!(
        report.reads.len(),
        1,
        "the later position was never searched: {:?}",
        report.reads
    );
}

// ---------------------------------------------------------------------------------------------
// 1.3 — binding through the same positions
// ---------------------------------------------------------------------------------------------

/// The driver path verifies its claim through the declared prefix, not around it.
///
/// The request names a physical definition by identity and the analysis reads it; the binding
/// check then resolves the name the header declares in the declared loader's own order. With the
/// prefix declared, that search walks the position the definition really lives at and the run
/// starts. With the prefix left out — the same definition, the same content, only a different
/// environment — the check cannot reach it and refuses the claim: fixing the candidate path while
/// the binding still compared another position is exactly the failure this pair excludes.
#[test]
fn a_prefixed_position_binds_the_driver_method_through_the_same_root() {
    let class = class_bytes(b"com/demo/A", 52);
    let snapshot = open(zip_of(&[(
        b"WEB-INF/classes/com/demo/A.class",
        class.as_slice(),
    )]));
    let mut budget = Budget::new(limits());
    let listed = Engine::new()
        .enumerate(&snapshot, &mut budget)
        .expect("the fixture lists");
    let entry = listed
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == b"WEB-INF/classes/com/demo/A.class")
        .expect("the fixture holds the class entry");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::ArchiveEntry {
            entry: entry.id.clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(&class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(b"run".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    };

    let run = |roots: Vec<LoadRoot>| {
        let request = MethodAnalysisRequest {
            environment: single_loader(&snapshot, roots),
            method: method.clone(),
            stages: vec![AnalysisStage::RawFacts],
        };
        let mut budget = Budget::new(limits());
        let report = Engine::new()
            .analyze_method(std::slice::from_ref(&snapshot), &request, &mut budget)
            .expect("a legal request is answered, not raised");
        report
            .stages
            .iter()
            .find(|stage| stage.stage == AnalysisStage::RawFacts)
            .expect("raw_facts is scheduled")
            .state
            .clone()
    };

    assert_eq!(
        run(vec![container_root(&snapshot, b"WEB-INF/classes/")]),
        StageState::Completed,
        "the declared prefix binds the definition the request named"
    );
    assert_eq!(
        run(vec![container_root(&snapshot, b"")]),
        StageState::Failed {
            code: "resolution_definition_unbound".to_string()
        },
        "the same definition under an environment that cannot reach it is refused"
    );
}

/// Declaration order decides between two prefixed positions, never the directory's own order.
///
/// The two roots are declared in the order the caller wants searched, while the archive stores
/// the entries the other way round: the first declared position wins although its entry has the
/// higher ordinal, and swapping the declaration swaps the winner. Both definitions keep their own
/// origin.
#[test]
fn declaration_order_decides_between_two_prefixed_positions() {
    let from_a = class_bytes(b"p/S", 52);
    let from_b = class_bytes(b"p/S", 51);
    let snapshot = open(zip_of(&[
        (b"B/p/S.class", from_b.as_slice()),
        (b"A/p/S.class", from_a.as_slice()),
    ]));
    let content = std::slice::from_ref(&snapshot);
    let request_with =
        |roots: Vec<LoadRoot>| class_request(single_loader(&snapshot, roots), "app", b"p/S");

    let a_first = resolve(
        content,
        &request_with(vec![
            container_root(&snapshot, b"A/"),
            container_root(&snapshot, b"B/"),
        ]),
    );
    assert_eq!(a_first.state, Some(ResolutionState::Resolved));
    assert_eq!(resolved_entry(&a_first).raw_name.0, b"A/p/S.class");
    assert_eq!(
        resolved_entry(&a_first).ordinal,
        1,
        "the declared order decided, not the central directory's"
    );

    let b_first = resolve(
        content,
        &request_with(vec![
            container_root(&snapshot, b"B/"),
            container_root(&snapshot, b"A/"),
        ]),
    );
    assert_eq!(b_first.state, Some(ResolutionState::Resolved));
    assert_eq!(resolved_entry(&b_first).raw_name.0, b"B/p/S.class");
    assert_ne!(
        a_first.resolved.as_ref().unwrap().definition,
        b_first.resolved.as_ref().unwrap().definition
    );
}

/// Each loader's own delegation policy orders the positions it declares, over the same content.
///
/// The application directory holds one `p/S` and a parent loader's directory holds another; the
/// two declarations differ only in which loader is `ChildFirst`. The winner changes with the
/// policy and nothing else, which is what "declaration order, not scan order" has to mean across
/// loaders.
#[test]
fn delegation_policy_moves_the_prefixed_positions_it_declares() {
    let in_app = class_bytes(b"p/S", 52);
    let in_parent = class_bytes(b"p/S", 51);
    let app_snapshot = open(zip_of(&[(b"WEB-INF/classes/p/S.class", &in_app)]));
    let parent_snapshot = open(zip_of(&[(b"platform/classes/p/S.class", &in_parent)]));
    let content = [app_snapshot.clone(), parent_snapshot.clone()];

    let ask = |app_delegation: DelegationPolicy, parent_delegation: DelegationPolicy| {
        let app = domain(
            "app",
            Some("platform"),
            app_delegation,
            vec![container_root(&app_snapshot, b"WEB-INF/classes/")],
        );
        let parent = domain(
            "platform",
            None,
            parent_delegation,
            vec![container_root(&parent_snapshot, b"platform/classes/")],
        );
        let environment = environment(&app_snapshot, app.clone(), vec![app, parent]);
        resolve(&content, &class_request(environment, "app", b"p/S"))
    };

    let child_first = ask(DelegationPolicy::ChildFirst, DelegationPolicy::ParentFirst);
    assert_eq!(child_first.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_entry(&child_first).origin.snapshot,
        *app_snapshot.id(),
        "a child-first caller searches its own declared position first"
    );

    let parent_first = ask(DelegationPolicy::ParentFirst, DelegationPolicy::ParentFirst);
    assert_eq!(parent_first.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_entry(&parent_first).origin.snapshot,
        *parent_snapshot.id(),
        "a parent-first caller searches the parent's declared position first"
    );
}

/// A nested library and the application directory can hold the same name; the position wins.
///
/// The nested container is a declared root of its own (its origin comes from the explicit tree
/// enumeration), which is what makes its content a position at all. Declaring the library first
/// resolves there; declaring the application directory first resolves there; the container scan
/// order — the application directory's entry came first in the archive — decides neither.
#[test]
fn a_nested_library_and_the_application_directory_are_two_positions() {
    let nested_class = class_bytes(b"p/S", 52);
    let app_class = class_bytes(b"p/S", 51);
    let library = zip_of(&[(b"p/S.class", &nested_class)]);
    let snapshot = open(zip_of(&[
        (b"WEB-INF/classes/p/S.class", &app_class),
        (b"WEB-INF/lib/L.jar", &library),
    ]));
    let content = std::slice::from_ref(&snapshot);

    let app_first = resolve(
        content,
        &class_request(
            single_loader(
                &snapshot,
                vec![
                    container_root(&snapshot, b"WEB-INF/classes/"),
                    nested_root(&snapshot, b"WEB-INF/lib/L.jar"),
                ],
            ),
            "app",
            b"p/S",
        ),
    );
    assert_eq!(app_first.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_entry(&app_first).raw_name.0,
        b"WEB-INF/classes/p/S.class"
    );

    let library_first = resolve(
        content,
        &class_request(
            single_loader(
                &snapshot,
                vec![
                    nested_root(&snapshot, b"WEB-INF/lib/L.jar"),
                    container_root(&snapshot, b"WEB-INF/classes/"),
                ],
            ),
            "app",
            b"p/S",
        ),
    );
    assert_eq!(library_first.state, Some(ResolutionState::Resolved));
    let entry = resolved_entry(&library_first);
    assert_eq!(entry.raw_name.0, b"p/S.class");
    assert_eq!(
        entry.origin.steps.len(),
        1,
        "the definition keeps the nested container it was read from"
    );
}

/// Two records with one name at one position stay two candidates with their own origins.
///
/// The duplicates are not ordered by ordinal and not merged by content: the position cannot elect
/// one definition, so the answer is `Ambiguous` with both candidates and their own coordinates.
/// Both reads happened and are recorded.
#[test]
fn duplicate_entries_at_one_position_stay_ambiguous_with_their_own_origins() {
    let first = class_bytes(b"p/S", 52);
    let second = class_bytes(b"p/S", 51);
    let snapshot = open(zip_of(&[
        (b"A/p/S.class", &first),
        (b"A/p/S.class", &second),
    ]));

    let report = resolve(
        std::slice::from_ref(&snapshot),
        &class_request(
            single_loader(&snapshot, vec![container_root(&snapshot, b"A/")]),
            "app",
            b"p/S",
        ),
    );
    assert_eq!(report.state, Some(ResolutionState::Ambiguous));
    assert!(report.resolved.is_none());
    let ordinals: Vec<u64> = report
        .candidates
        .iter()
        .map(|candidate| candidate.definition.entry().unwrap().ordinal)
        .collect();
    assert_eq!(ordinals, vec![0, 1], "each record keeps its own ordinal");
    assert_eq!(
        report
            .candidates
            .iter()
            .map(|candidate| candidate.definition.entry().unwrap().raw_name.0.clone())
            .collect::<Vec<_>>(),
        vec![b"A/p/S.class".to_vec(), b"A/p/S.class".to_vec()]
    );
    assert_eq!(
        report.reads.len(),
        2,
        "both indistinguishable candidates were read"
    );
}

/// Byte-equal classes at two origins are two definitions and are never merged.
///
/// The same bytes are stored under two prefixes; each declared position resolves to the entry of
/// its own origin and publishes the same content identity. A result that folded them by content
/// would let one origin's definition answer for the other's position.
#[test]
fn the_same_bytes_at_two_origins_are_two_definitions() {
    let class = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[
        (b"A/p/S.class", &class),
        (b"B/p/S.class", &class),
    ]));
    let content = std::slice::from_ref(&snapshot);

    let from_a = resolve(
        content,
        &class_request(
            single_loader(&snapshot, vec![container_root(&snapshot, b"A/")]),
            "app",
            b"p/S",
        ),
    );
    let from_b = resolve(
        content,
        &class_request(
            single_loader(&snapshot, vec![container_root(&snapshot, b"B/")]),
            "app",
            b"p/S",
        ),
    );
    let left = from_a
        .resolved
        .as_ref()
        .expect("A/ resolves")
        .definition
        .clone();
    let right = from_b
        .resolved
        .as_ref()
        .expect("B/ resolves")
        .definition
        .clone();
    assert_eq!(
        left.class_bytes, right.class_bytes,
        "the two definitions hold the same bytes"
    );
    assert_ne!(left, right, "and they are not one definition");
    assert_eq!(resolved_entry(&from_a).raw_name.0, b"A/p/S.class");
    assert_eq!(resolved_entry(&from_b).raw_name.0, b"B/p/S.class");
}

/// A damaged candidate stops the search at its own position instead of falling through.
///
/// The first declared position holds an entry whose bytes are not a class file at all; the second
/// holds the real class. The position's own refusal — charged, with the entry's origin in the
/// diagnostic — is the answer, and no later position is searched: a name that *could* be read
/// somewhere else does not make this candidate readable.
#[test]
fn a_damaged_candidate_stops_the_search_at_its_own_position() {
    let honest = class_bytes(b"p/S", 52);
    let broken = open(zip_of(&[(b"A/p/S.class", b"not a class file")]));
    let good = open(zip_of(&[(b"B/p/S.class", &honest)]));
    let content = [broken.clone(), good.clone()];

    let report = resolve(
        &content,
        &class_request(
            single_loader(
                &broken,
                vec![container_root(&broken, b"A/"), container_root(&good, b"B/")],
            ),
            "app",
            b"p/S",
        ),
    );
    assert!(report.state.is_none() && report.resolved.is_none());
    assert_eq!(diagnostic_codes(&report), vec!["classfile_decode"]);
    assert!(
        report.diagnostics[0].message.contains("\"A/p/S.class\""),
        "the refusal names the entry it read: {}",
        report.diagnostics[0].message
    );
    assert!(report.reads.is_empty(), "the bytes produced no definition");
}

/// A directory that could not be read completely is a refusal, never a `Missing` verdict.
///
/// The container holds three entries and the request funds one: the directory parse stops before
/// the name's own record is proven absent, so the position refuses with the budget stop and the
/// report keeps no semantic decision. An empty answer would claim that the container holds no such
/// name, which a stopped listing cannot state.
#[test]
fn an_incomplete_directory_is_a_refusal_not_a_missing_verdict() {
    let class = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[
        (b"A/p/S.class", &class),
        (b"A/other.txt", b"one"),
        (b"A/third.txt", b"two"),
    ]));
    let request = class_request(
        single_loader(&snapshot, vec![container_root(&snapshot, b"A/")]),
        "app",
        b"p/S",
    );

    let mut budget = Budget::new(Limits {
        archive_entries: 1,
        ..limits()
    });
    let report = Engine::new()
        .resolve_symbol(std::slice::from_ref(&snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised");
    assert_eq!(
        report.state,
        Some(ResolutionState::BudgetExceeded),
        "the stop is the decision this request reached, never a `Missing`"
    );
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ArchiveEntries
                },
                ..
            }
        ),
        "the stopped directory parse is the stop: {:?}",
        report.execution
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["budget_exceeded_archive_entries"]
    );
    assert_ne!(report.state, Some(ResolutionState::Missing));
}

/// Changing the prefix is a new environment: the old verdict is neither reused nor cached.
///
/// The same content is asked for the same name twice: once under an empty prefix (which cannot
/// reach the WAR classes directory) and once under the declared prefix. The second request
/// resolves, with and without a facts cache attached, and the cache keeps answering the *physical*
/// container facts across both: a hit on the directory is not a verdict about a name, so the
/// second environment is decided from scratch.
#[test]
fn a_changed_prefix_is_a_new_environment_that_reuses_no_verdict() {
    let class = class_bytes(b"com/demo/A", 52);
    let snapshot = open(zip_of(&[(
        b"WEB-INF/classes/com/demo/A.class",
        class.as_slice(),
    )]));
    let content = std::slice::from_ref(&snapshot);

    let run = |prefix: &[u8], cache: Option<&FactsCache>| {
        resolve_with(
            content,
            &class_request(
                single_loader(&snapshot, vec![container_root(&snapshot, prefix)]),
                "app",
                b"com/demo/A",
            ),
            cache,
        )
    };

    // Cache off: the two environments decide independently.
    let (unbound, _) = run(b"", None);
    assert_eq!(unbound.state, Some(ResolutionState::Missing));
    let (bound, _) = run(b"WEB-INF/classes/", None);
    assert_eq!(bound.state, Some(ResolutionState::Resolved));
    assert_ne!(
        unbound.environment_identity.runtime, bound.environment_identity.runtime,
        "the prefix is part of the runtime environment's identity, not a lookup detail"
    );

    // Cache on: the same two verdicts, and the physical container facts are what is reused.
    let store = FactsCache::current(FactsCapacity::new(4, u64::MAX));
    let (unbound_warm, _) = run(b"", Some(&store));
    assert_eq!(
        unbound_warm.state,
        Some(ResolutionState::Missing),
        "a warm store does not turn the empty prefix into a hit"
    );
    let after_unbound = store.report();
    assert_eq!(
        (after_unbound.consultations, after_unbound.entries),
        (0, 0),
        "a name no position holds is not a fact this layer stores: {after_unbound:?}"
    );

    let (bound_warm, _) = run(b"WEB-INF/classes/", Some(&store));
    assert_eq!(bound_warm.state, Some(ResolutionState::Resolved));
    let after_bound = store.report();
    assert!(
        after_bound.container_hits > after_unbound.container_hits,
        "the second environment reused the verified directory, not a verdict: {after_bound:?}"
    );
    assert_eq!(
        after_bound.entries, 1,
        "the one cached CP/Header product is the class the resolved request read, keyed by its \
         content rather than by the name or prefix of the request: {after_bound:?}"
    );
    assert_eq!(
        store.report().consultations,
        1,
        "the unbound request consulted no class facts: a `Missing` verdict is not a fact"
    );
}
