//! Public Engine/class-source regression for declared generic return candidates.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const OWNED_FIELD_CYCLE_CLASS: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-owned-field-cycle-r1/classes/flow/OwnedFieldCycle.class"
);
const RETURN_CLASS: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-return-consumption-r1/classes/flow/ReturnConsumptionControls.class"
);
const BASE_CLASS: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/classes/flow/Base.class"
);
const CONFLICT_CLASS: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-conflicting-return-r1/classes/flow/ConflictingReturnControls.class"
);
const FOO_CLASS: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-bound-type-identities-r1/identity-a/classes/identity/Foo.class"
);
const HEADER_UNCLOSED_CLASS: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-header-unclosed-r1/classes/flow/HeaderUnclosed.class"
);
const HEADER_CLOSED_CLASS: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-header-unclosed-r1/classes/flow/HeaderClosed.class"
);
const HEADER_SHADOW_CLASS: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-header-unclosed-r1/classes/flow/HeaderShadow.class"
);
const GENERIC_PARENT_CLASS: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-header-unclosed-r1/classes/flow/GenericParent.class"
);
const TYPE_TAG_CLASS: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-header-unclosed-r1/classes/flow/TypeTag.class"
);
const PLATFORM_CLASSES: &[(&[u8], &[u8])] = &[
    (
        b"java/io/Serializable.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/io/Serializable.class"
        ),
    ),
    (
        b"java/lang/CharSequence.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/lang/CharSequence.class"
        ),
    ),
    (
        b"java/lang/Cloneable.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/lang/Cloneable.class"
        ),
    ),
    (
        b"java/lang/Comparable.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/lang/Comparable.class"
        ),
    ),
    (
        b"java/lang/Iterable.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/lang/Iterable.class"
        ),
    ),
    (
        b"java/lang/Object.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/lang/Object.class"
        ),
    ),
    (
        b"java/lang/String.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/lang/String.class"
        ),
    ),
    (
        b"java/util/AbstractCollection.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/util/AbstractCollection.class"
        ),
    ),
    (
        b"java/util/AbstractList.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/util/AbstractList.class"
        ),
    ),
    (
        b"java/util/ArrayList.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/util/ArrayList.class"
        ),
    ),
    (
        b"java/util/Collection.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/util/Collection.class"
        ),
    ),
    (
        b"java/util/List.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/util/List.class"
        ),
    ),
    (
        b"java/util/RandomAccess.class",
        include_bytes!(
            "../crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform/java/util/RandomAccess.class"
        ),
    ),
];

fn jar(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .expect("fixture ZIP entry starts");
            let mut writer = config.wrap(&mut entry);
            writer.write_all(bytes).expect("fixture bytes are written");
            let (_, descriptor) = writer.finish().expect("fixture entry closes");
            entry.finish(descriptor).expect("fixture record finishes");
        }
        archive.finish().expect("fixture ZIP closes");
    }
    output.into_inner()
}

fn class_request(
    snapshot: &ArtifactSnapshot,
    platform: &ArtifactSnapshot,
    class_name: &str,
) -> ClassSourceRequest {
    let root = |snapshot: &ArtifactSnapshot| LoadRoot::Container {
        origin: ContainerOrigin {
            snapshot: snapshot.id().clone(),
            root_container: ContainerId("root".to_owned()),
            steps: Vec::new(),
        },
        prefix: ArchiveNameBytes(Vec::new()),
    };
    let app = LoaderId("app".to_owned());
    let boot = LoaderId("boot".to_owned());
    ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::ExplicitDomains {
                domains: vec![
                    LoadDomain {
                        loader: app.clone(),
                        parent_loader: Some(boot.clone()),
                        delegation: DelegationPolicy::ParentFirst,
                        roots: vec![root(snapshot)],
                        module_mode: ModuleMode::ClassPath,
                        external_override: RuntimeUncertainty::None,
                        runtime_transformation: RuntimeUncertainty::None,
                    },
                    LoadDomain {
                        loader: boot,
                        parent_loader: None,
                        delegation: DelegationPolicy::ParentFirst,
                        roots: vec![root(platform)],
                        module_mode: ModuleMode::ClassPath,
                        external_override: RuntimeUncertainty::None,
                        runtime_transformation: RuntimeUncertainty::None,
                    },
                ],
                providers: Vec::new(),
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: app,
        },
    }
}

fn one_field_instruction_bci(
    engine: &Engine,
    class_bytes: &[u8],
    method_name: &[u8],
    descriptor: &[u8],
    opcode: u8,
) -> u32 {
    let mut budget = task_budget(&[]).expect("header inspection budget");
    let snapshot = engine
        .open(ArtifactInput::bytes(class_bytes.to_vec()), &mut budget)
        .expect("frozen fixture class opens");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("fixture header is readable");
    let method = inspected
        .inspection
        .header
        .methods
        .iter()
        .find(|method| {
            method.name.raw().0.as_slice() == method_name
                && method.descriptor.raw().0.as_slice() == descriptor
        })
        .expect("the named physical method exists");
    let code = jarde_reader::classfile::method_code_facts(class_bytes, method, &mut budget)
        .expect("the method Code is readable");
    let matches = code
        .instructions
        .iter()
        .filter(|instruction| instruction.opcode == opcode)
        .map(|instruction| instruction.bci)
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "the minimal method has one matching field opcode"
    );
    matches[0]
}

#[test]
fn engine_class_source_presents_declared_return_candidates_and_preserves_conflicting_signature() {
    let engine = Engine::new();
    let app_bytes = jar(&[
        (b"flow/ReturnConsumptionControls.class", RETURN_CLASS),
        (b"flow/Base.class", BASE_CLASS),
        (b"flow/ConflictingReturnControls.class", CONFLICT_CLASS),
        (b"identity/Foo.class", FOO_CLASS),
    ]);
    let platform_bytes = jar(PLATFORM_CLASSES);
    let mut open_budget = task_budget(&[]).expect("default open budget");
    let app = engine
        .open(ArtifactInput::bytes(app_bytes), &mut open_budget)
        .expect("frozen application classes open");
    let platform = engine
        .open(ArtifactInput::bytes(platform_bytes), &mut open_budget)
        .expect("frozen platform headers open");
    let request = class_request(&app, &platform, "flow/ReturnConsumptionControls");
    let mut budget = task_budget(&[]).expect("default recovery budget");
    let OperationOutcome::Performed(report) = engine
        .class_source(&[app.clone(), platform.clone()], &request, &mut budget)
        .expect("class-source request is valid")
    else {
        panic!("the frozen class resolves uniquely");
    };

    println!(
        "CLASS_SOURCE_TEXT_BEGIN\n{}\nCLASS_SOURCE_TEXT_END",
        report.text
    );
    assert!(report.text.contains(
        "public static java.util.List<identity.Foo> returnParameter(java.util.List<identity.Foo> input)"
    ));
    assert!(
        report
            .text
            .contains("public static <T extends java.util.List<identity.Foo>> T echo(T input)")
    );
    assert!(
        report.text.contains(
            "public static java.util.List<identity.Foo> unchecked(java.lang.Object input)"
        )
    );
    assert!(report.text.contains(
        "public static java.util.List<identity.Foo> returnField(flow.Base<java.util.List<identity.Foo>> input)"
    ));
    assert_eq!(
        report
            .text
            .matches("declared source types and physical consumers committed together")
            .count(),
        4,
        "the staged class-source pass committed each of the four selected methods"
    );
    assert_eq!(report.projection_inputs.member_texts.len(), 4);
    for (name, required_bci) in [
        (b"returnParameter".as_slice(), 1),
        (b"echo".as_slice(), 1),
        (b"returnField".as_slice(), 4),
        (b"unchecked".as_slice(), 1),
    ] {
        let method = report
            .methods
            .iter()
            .find(|method| method.item.identity.name.0 == name)
            .expect("the selected method remains in the physical member table");
        let projected = report
            .projection_inputs
            .member_texts
            .iter()
            .find(|projected| projected.member == method.item.index)
            .expect("the method has a committed projected member text");
        assert!(report.text.contains(&projected.text));
        let emission = projected
            .emission
            .as_ref()
            .expect("declared method text retains its body emission");
        assert!(!emission.body.is_empty());
        assert!(!emission.segments.is_empty());
        let mut has_required_bci = false;
        for segment in &emission.segments {
            let start = usize::try_from(segment.start).expect("source offset fits usize");
            let end = usize::try_from(segment.end).expect("source offset fits usize");
            assert!(start < end, "emitted source segments are nonempty");
            assert!(
                end <= emission.body.len(),
                "segment stays inside emitted body"
            );
            assert!(emission.body.is_char_boundary(start));
            assert!(emission.body.is_char_boundary(end));
            has_required_bci |= segment.bcis.contains(&required_bci);
        }
        assert!(
            has_required_bci,
            "the projected emission keeps its physical BCI {required_bci}"
        );
    }
    for method in &report.methods {
        if matches!(
            method.item.name.raw().0.as_slice(),
            b"returnParameter" | b"echo" | b"returnField" | b"unchecked"
        ) {
            println!(
                "METHOD={:?} DESC={:?} DECL={:?}",
                method.item.name.raw().0,
                method.item.identity.descriptor.0,
                method.declaration
            );
            match &method.outcome {
                ClassSourceOutcome::Recovered { report: body, .. } => {
                    assert_eq!(
                        body.input_origin,
                        jarde_java::report::RecoveryInputOrigin::SelectedReads
                    );
                    assert_eq!(
                        body.content,
                        jarde_java::report::RecoveryContent::ContainsStatements
                    );
                    let source_bci = match method.item.name.raw().0.as_slice() {
                        b"returnParameter" | b"echo" => 1,
                        b"returnField" => 4,
                        b"unchecked" => 1,
                        _ => unreachable!(),
                    };
                    assert!(
                        !body.source_map.of_bci(source_bci).is_empty(),
                        "selected method BCI {source_bci} keeps source-map provenance"
                    );
                    println!("BODY_BEGIN\n{}\nBODY_END", body.text);
                    println!("SOURCE_MAP={:?}", body.source_map);
                }
                other => println!("OUTCOME={other:?}"),
            }
        }
    }
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));

    let conflict_request = class_request(&app, &platform, "flow/ConflictingReturnControls");
    let mut conflict_budget = task_budget(&[]).expect("default conflict recovery budget");
    let OperationOutcome::Performed(conflict) = engine
        .class_source(
            &[app.clone(), platform.clone()],
            &conflict_request,
            &mut conflict_budget,
        )
        .expect("conflict class-source request is valid")
    else {
        panic!("the frozen conflict class resolves uniquely");
    };
    println!(
        "CONFLICT_CLASS_SOURCE_TEXT_BEGIN\n{}\nCONFLICT_CLASS_SOURCE_TEXT_END",
        conflict.text
    );
    assert!(conflict.text.contains(
        "generic Signature projection refused for `conflict(Ljava/util/List;)Ljava/util/List;`: unsupported (ordinary_generic_source_unproved): returned parameter has a different generic source type"
    ));
    assert!(
        conflict
            .text
            .contains("public static java.util.List conflict(java.util.List source)")
    );
    assert!(!conflict.text.contains("List<identity.Foo> conflict"));
    assert_eq!(
        conflict
            .text
            .matches("declared source types and physical consumers committed together")
            .count(),
        0,
        "the conflicting generic return did not commit a typed candidate"
    );
    for method in &conflict.methods {
        println!(
            "CONFLICT_METHOD={:?} DESC={:?} DECL={:?}",
            method.item.name.raw().0,
            method.item.identity.descriptor.0,
            method.declaration
        );
        match &method.outcome {
            ClassSourceOutcome::Recovered { report: body, .. } => {
                println!("CONFLICT_BODY_BEGIN\n{}\nCONFLICT_BODY_END", body.text);
                println!("CONFLICT_SOURCE_MAP={:?}", body.source_map);
            }
            other => println!("CONFLICT_OUTCOME={other:?}"),
        }
    }
    assert!(matches!(
        conflict.execution,
        ExecutionReport::Complete { .. }
    ));
}

#[test]
fn declared_class_header_stays_raw_when_a_class_variable_field_is_not_staged() {
    let engine = Engine::new();
    let app_bytes = jar(&[
        (b"flow/HeaderUnclosed.class", HEADER_UNCLOSED_CLASS),
        (b"flow/GenericParent.class", GENERIC_PARENT_CLASS),
        (b"flow/TypeTag.class", TYPE_TAG_CLASS),
    ]);
    let platform_bytes = jar(PLATFORM_CLASSES);
    let mut open_budget = task_budget(&[]).expect("default open budget");
    let app = engine
        .open(ArtifactInput::bytes(app_bytes), &mut open_budget)
        .expect("the application fixture opens");
    let platform = engine
        .open(ArtifactInput::bytes(platform_bytes), &mut open_budget)
        .expect("the frozen 13-header platform fixture opens");
    let request = class_request(&app, &platform, "flow/HeaderUnclosed");
    let mut budget = task_budget(&[]).expect("default class-source budget");
    let OperationOutcome::Performed(report) = engine
        .class_source(&[app.clone(), platform.clone()], &request, &mut budget)
        .expect("HeaderUnclosed class-source request is valid")
    else {
        panic!("the frozen class resolves uniquely")
    };

    println!(
        "UNCLOSED_HEADER_REPORT={}",
        serde_json::to_string(&report).unwrap()
    );
    // The generic echo forces the actual complete class member-use census.
    // GenericParent implements Serializable, so the legacy direct-parent projector's
    // single-parameter/no-interface shape rejects this Header. The class-level bound graph can
    // resolve that real edge from the frozen 13-header platform set. Its field has a
    // class-variable Signature and TYPE_USE annotation, so the field stage skips it. Do not
    // publish the dependent Header unless the field joins the same commit.
    assert!(
        report
            .text
            .contains("class HeaderUnclosed extends flow.GenericParent")
    );
    assert!(
        !report
            .text
            .contains("class HeaderUnclosed<T> extends flow.GenericParent<T>")
    );
    assert!(!report.text.contains("T value"));

    let closed_bytes = jar(&[
        (b"flow/HeaderClosed.class", HEADER_CLOSED_CLASS),
        (b"flow/GenericParent.class", GENERIC_PARENT_CLASS),
    ]);
    let closed_app = engine
        .open(ArtifactInput::bytes(closed_bytes), &mut open_budget)
        .unwrap();
    let closed_request = class_request(&closed_app, &platform, "flow/HeaderClosed");
    let OperationOutcome::Performed(closed) = engine
        .class_source(
            &[closed_app.clone(), platform.clone()],
            &closed_request,
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap()
    else {
        panic!("the complete field-scope control resolves");
    };
    println!(
        "CLOSED_HEADER_REPORT={}",
        serde_json::to_string(&closed).unwrap()
    );
    assert!(matches!(closed.execution, ExecutionReport::Complete { .. }));
    assert!(
        closed
            .declaration
            .as_ref()
            .unwrap()
            .declaration
            .contains("extends flow.GenericParent<T>")
    );
    assert!(
        closed
            .declaration
            .as_ref()
            .unwrap()
            .generic_refusal
            .is_none()
    );
    assert!(closed.fields[0].generic_signature_projected);
    assert_eq!(
        closed.fields[0].declaration.as_deref(),
        Some("public T value")
    );
    assert!(closed.fields[1].generic_signature_projected);
    assert_eq!(
        closed.fields[1].declaration.as_deref(),
        Some("public java.util.List<T> nested")
    );
    let mut limits = task_budget(&[]).unwrap().limits().clone();
    limits.output_bytes = closed.usage.output_bytes - 1;
    let OperationOutcome::Performed(stopped) = engine
        .class_source(
            &[closed_app, platform],
            &closed_request,
            &mut Budget::new(limits),
        )
        .unwrap()
    else {
        panic!("the same class remains bound under the output limit");
    };
    assert!(matches!(
        stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::OutputBytes
            },
            ..
        } | ExecutionReport::Failed {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::OutputBytes
            },
            ..
        }
    ));
    assert!(
        !stopped
            .declaration
            .as_ref()
            .unwrap()
            .declaration
            .contains("GenericParent<T>")
    );
    assert!(
        stopped
            .fields
            .iter()
            .all(|field| !field.generic_signature_projected)
    );
    assert!(
        !stopped
            .text
            .contains("declared source types and physical consumers committed together")
    );
}

#[test]
fn declared_source_type_spelling_respects_lexical_type_variable_scope() {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let app = engine
        .open(
            ArtifactInput::bytes(jar(&[
                (b"flow/HeaderShadow.class", HEADER_SHADOW_CLASS),
                (b"flow/GenericParent.class", GENERIC_PARENT_CLASS),
            ])),
            &mut budget,
        )
        .unwrap();
    let platform = engine
        .open(ArtifactInput::bytes(jar(PLATFORM_CLASSES)), &mut budget)
        .unwrap();
    let request = class_request(&app, &platform, "flow/HeaderShadow");
    let OperationOutcome::Performed(report) = engine
        .class_source(&[app, platform], &request, &mut budget)
        .unwrap()
    else {
        panic!("the original legal source class resolves");
    };
    println!(
        "SHADOW_HEADER_REPORT={}",
        serde_json::to_string(&report).unwrap()
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert!(
        report
            .declaration
            .as_ref()
            .unwrap()
            .generic_refusal
            .is_some(),
        "the type variable flow captures the first identifier of flow.GenericParent"
    );
}

#[test]
fn engine_inherited_class_headers_and_field_consumers_commit_together() {
    // Explicit new Corretto 8 baseline, not a byte reconstruction of the deleted dev-v3 jars.
    let platform_bytes = jar(PLATFORM_CLASSES);
    let platform_child = jar(&[(
        b"PlatformChild.class",
        include_bytes!("fixtures/s2b-inherited-generics/classes/PlatformChild.class"),
    )]);
    let custom_child = jar(&[
        (
            b"CustomChild.class",
            include_bytes!("fixtures/s2b-inherited-generics/classes/CustomChild.class"),
        ),
        (
            b"Mid.class",
            include_bytes!("fixtures/s2b-inherited-generics/classes/Mid.class"),
        ),
        (
            b"Parent.class",
            include_bytes!("fixtures/s2b-inherited-generics/classes/Parent.class"),
        ),
    ]);
    for (class, bytes, parent) in [
        (
            "PlatformChild",
            platform_child,
            "java.util.ArrayList<java.lang.String>",
        ),
        ("CustomChild", custom_child, "Mid<java.lang.String>"),
    ] {
        let engine = Engine::new();
        let mut budget = task_budget(&[]).unwrap();
        let app = engine
            .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
            .unwrap();
        let boot = engine
            .open(ArtifactInput::bytes(platform_bytes.clone()), &mut budget)
            .unwrap();
        let boot_snapshot = boot.id().clone();
        let request = class_request(&app, &boot, class);
        let OperationOutcome::Performed(report) = engine
            .class_source(&[app, boot], &request, &mut budget)
            .unwrap()
        else {
            panic!("the development class resolves");
        };
        println!(
            "INHERITED_CLASS_BEGIN={class}\n{}\nINHERITED_CLASS_END",
            report.text
        );
        println!(
            "INHERITED_REPORT={}",
            serde_json::to_string(&report).unwrap()
        );
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "{class}: {:?}",
            report.execution
        );
        let declaration = report.declaration.as_ref().unwrap();
        assert!(
            declaration.declaration.contains(parent),
            "{class}: {}",
            declaration.declaration
        );
        assert!(declaration.generic_refusal.is_none());
        for name in if class == "PlatformChild" {
            ["inheritedRead", "returnInherited"]
        } else {
            ["getInherited", "readInheritedField"]
        } {
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == name.as_bytes())
                .unwrap();
            let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
                panic!("a complete method report is retained");
            };
            let binding = body
                .artifact
                .binding()
                .expect("the actual selected-read report is bound");
            assert!(
                binding
                    .dependencies()
                    .iter()
                    .any(|dependency| dependency.loader.0 == "boot"
                        && dependency
                            .definition
                            .location
                            .entry()
                            .is_some_and(|entry| entry.origin.snapshot == boot_snapshot
                                && entry.raw_name.0 == b"java/lang/String.class")),
                "{class}.{name}: a Header reused across methods still binds its actual defining loader/definition"
            );
        }
        assert!(report.projection_inputs.member_texts.iter().all(|member| {
            member
                .emission
                .as_ref()
                .is_some_and(|emission| !emission.body.is_empty() && !emission.segments.is_empty())
        }));
        if class == "PlatformChild" {
            let names = report
                .fields
                .iter()
                .find(|field| field.item.name.raw().0 == b"names")
                .unwrap();
            assert!(names.generic_signature_projected);
            assert!(
                names
                    .declaration
                    .as_ref()
                    .unwrap()
                    .contains("java.util.List<java.lang.String>")
            );
            assert!(
                report
                    .methods
                    .iter()
                    .find(|method| method.item.name.raw().0 == b"listRead")
                    .unwrap()
                    .declaration
                    .as_ref()
                    .unwrap()
                    .contains("java.util.List<java.lang.String>")
            );
        } else {
            assert!(
                report
                    .methods
                    .iter()
                    .find(|method| method.item.name.raw().0 == b"getInherited")
                    .unwrap()
                    .declaration
                    .as_ref()
                    .unwrap()
                    .contains("java.util.List<java.lang.String>")
            );
            assert!(
                report
                    .methods
                    .iter()
                    .find(|method| method.item.name.raw().0 == b"echo")
                    .unwrap()
                    .declaration
                    .as_ref()
                    .unwrap()
                    .contains("java.util.List<java.lang.String>")
            );
        }
    }
}

#[test]
fn engine_commits_an_owned_generic_field_with_its_call_producer_and_read_consumer() {
    const MARKER: &str = "declared source types and physical consumers committed together";
    let engine = Engine::new();
    let app_bytes = jar(&[
        (b"flow/OwnedFieldCycle.class", OWNED_FIELD_CYCLE_CLASS),
        (b"flow/Base.class", BASE_CLASS),
        (b"identity/Foo.class", FOO_CLASS),
    ]);
    let platform_bytes = jar(PLATFORM_CLASSES);
    let mut open_budget = task_budget(&[]).expect("open budget");
    let app = engine
        .open(ArtifactInput::bytes(app_bytes), &mut open_budget)
        .expect("app classes open");
    let platform = engine
        .open(ArtifactInput::bytes(platform_bytes), &mut open_budget)
        .expect("platform headers open");
    let request = class_request(&app, &platform, "flow/OwnedFieldCycle");
    let mut budget = task_budget(&[]).expect("recovery budget");
    let OperationOutcome::Performed(report) = engine
        .class_source(&[app.clone(), platform.clone()], &request, &mut budget)
        .expect("class-source request is valid")
    else {
        panic!("the fixture class resolves uniquely");
    };

    println!(
        "OWNED_CLASS_REPORT={}",
        serde_json::to_string(&report).unwrap()
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    let values = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"values")
        .expect("the owned values field remains in its physical field row");
    assert!(values.generic_signature_projected);
    assert!(
        values
            .declaration
            .as_deref()
            .is_some_and(|text| text.contains("java.util.List<identity.Foo>"))
    );
    for (name, descriptor, opcode) in [
        (b"put".as_slice(), b"(Lflow/Base;)V".as_slice(), 0xb5),
        (b"get".as_slice(), b"()Ljava/util/List;".as_slice(), 0xb4),
    ] {
        let method = report
            .methods
            .iter()
            .find(|method| method.item.identity.name.0 == name)
            .expect("the physical writer/reader method is retained");
        assert!(
            method
                .declaration
                .as_deref()
                .is_some_and(|text| text.contains("identity.Foo")),
            "the member declaration carries the selected source type: {:?}",
            method.declaration
        );
        let body = match &method.outcome {
            ClassSourceOutcome::Recovered { report, .. } => report,
            other => panic!("the physical method body is recovered: {other:?}"),
        };
        assert_eq!(
            body.content,
            jarde_java::report::RecoveryContent::ContainsStatements
        );
        let bci =
            one_field_instruction_bci(&engine, OWNED_FIELD_CYCLE_CLASS, name, descriptor, opcode);
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "original field-op BCI {bci} remains mapped"
        );
        let projected = report
            .projection_inputs
            .member_texts
            .iter()
            .find(|candidate| candidate.member == method.item.index)
            .expect("the committed member is an assembled projection input");
        assert!(report.text.contains(&projected.text));
        let emission = projected
            .emission
            .as_ref()
            .expect("the method body remains re-emittable");
        assert!(
            emission
                .segments
                .iter()
                .any(|segment| segment.bcis.contains(&bci)),
            "typed emission keeps the exact original field-op BCI {bci}"
        );
        assert!(projected.text.contains(MARKER));
    }
    assert_eq!(
        report.text.matches(MARKER).count(),
        2,
        "put/get commit with their physical consumers"
    );

    let mut stop_limits = budget.limits().clone();
    stop_limits.output_bytes = report.usage.output_bytes - 1;
    let mut stop_budget = Budget::new(stop_limits);
    let OperationOutcome::Performed(stopped) = engine
        .class_source(&[app.clone(), platform.clone()], &request, &mut stop_budget)
        .expect("the same request can stop during its final staged output")
    else {
        panic!("the root definition still resolves");
    };
    assert!(matches!(
        stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::OutputBytes
            },
            ..
        } | ExecutionReport::Failed {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::OutputBytes
            },
            ..
        }
    ));
    assert_eq!(stopped.text.matches(MARKER).count(), 0);
    assert!(
        !stopped
            .text
            .contains("declared field type and physical consumers committed together")
    );
    assert!(
        !stopped
            .fields
            .iter()
            .find(|field| field.item.name.raw().0 == b"values")
            .expect("the physical field survives the stop")
            .generic_signature_projected
    );

    // Keep the same compiled class bytes but omit the selected Foo definition. This is a real
    // dependency-resolution failure control, not a modified Signature or hand-built classfile.
    let incomplete_app_bytes = jar(&[
        (b"flow/OwnedFieldCycle.class", OWNED_FIELD_CYCLE_CLASS),
        (b"flow/Base.class", BASE_CLASS),
    ]);
    let mut incomplete_open_budget = task_budget(&[]).expect("incomplete app open budget");
    let incomplete_app = engine
        .open(
            ArtifactInput::bytes(incomplete_app_bytes),
            &mut incomplete_open_budget,
        )
        .expect("incomplete app snapshot opens");
    let incomplete_request = class_request(&incomplete_app, &platform, "flow/OwnedFieldCycle");
    let mut incomplete_budget = task_budget(&[]).expect("incomplete recovery budget");
    let OperationOutcome::Performed(incomplete) = engine
        .class_source(
            &[incomplete_app, platform],
            &incomplete_request,
            &mut incomplete_budget,
        )
        .expect("missing generic dependency is represented in the report")
    else {
        panic!("the root class still resolves uniquely");
    };
    assert!(matches!(
        incomplete.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(
        !incomplete
            .fields
            .iter()
            .find(|field| field.item.name.raw().0 == b"values")
            .expect("the physical field remains visible without its type dependency")
            .generic_signature_projected
    );
    assert_eq!(
        incomplete.text.matches(MARKER).count(),
        0,
        "the absent Foo definition prevents staged writer/read consumer admission"
    );
}

#[test]
#[ignore = "requires python3 and S2B_CORRETTO8_HOME pointing to the pinned Corretto 8.432.06.1 JDK"]
fn engine_inherited_generics_complete_classes_match_original_runtime() {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    let jdk = std::env::var_os("S2B_CORRETTO8_HOME").expect(
        "set S2B_CORRETTO8_HOME to the Corretto 8 JDK pinned in the fixture toolchain.json",
    );
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = repository.join("tests/fixtures/s2b-inherited-generics");
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the epoch")
        .as_nanos();
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jarde-s2b-inherited-runtime-{}-{nonce}",
        std::process::id()
    )));
    fs::create_dir(&scratch.0).expect("create isolated runtime workspace");
    let recovered = scratch.0.join("recovered-sources");
    fs::create_dir(&recovered).expect("create recovered source directory");

    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).unwrap();
    let app = engine
        .open(
            ArtifactInput::bytes(jar(&[
                (
                    b"PlatformChild.class",
                    include_bytes!("fixtures/s2b-inherited-generics/classes/PlatformChild.class"),
                ),
                (
                    b"CustomChild.class",
                    include_bytes!("fixtures/s2b-inherited-generics/classes/CustomChild.class"),
                ),
                (
                    b"Mid.class",
                    include_bytes!("fixtures/s2b-inherited-generics/classes/Mid.class"),
                ),
                (
                    b"Parent.class",
                    include_bytes!("fixtures/s2b-inherited-generics/classes/Parent.class"),
                ),
            ])),
            &mut open_budget,
        )
        .expect("frozen targets and original support classes open");
    let platform = engine
        .open(
            ArtifactInput::bytes(jar(PLATFORM_CLASSES)),
            &mut open_budget,
        )
        .expect("the same 13 frozen platform headers open");
    for class in ["CustomChild", "PlatformChild"] {
        let request = class_request(&app, &platform, class);
        let OperationOutcome::Performed(report) = engine
            .class_source(
                &[app.clone(), platform.clone()],
                &request,
                &mut task_budget(&[]).unwrap(),
            )
            .expect("public Engine class-source request succeeds")
        else {
            panic!("{class} resolves uniquely");
        };
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "{class}: {:?}",
            report.execution
        );
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        for method in &report.methods {
            let ClassSourceOutcome::Recovered {
                report: body,
                analysis,
            } = &method.outcome
            else {
                panic!(
                    "{class} contains an unrecovered method: {:?}",
                    method.outcome
                );
            };
            assert!(matches!(
                analysis.execution,
                ExecutionReport::Complete { .. }
            ));
            assert!(matches!(body.execution, ExecutionReport::Complete { .. }));
            assert_eq!(body.quality, Quality::Structured, "{}", body.text);
            assert!(
                body.fallbacks.is_empty(),
                "{}: {:?}",
                body.text,
                body.fallbacks
            );
        }
        fs::write(recovered.join(format!("{class}.java")), &report.text)
            .expect("write the complete Engine report.text unchanged");
    }

    // The generator checks the compiler pins, builds/runs the original baseline, then compiles
    // both unchanged recovered targets with only Parent/Mid support. It separately compiles
    // Driver against that recovered class set and executes without original target fallback.
    let result = Command::new("python3")
        .arg(fixture.join("generate.py"))
        .arg("--jdk")
        .arg(jdk)
        .arg("--repo")
        .arg(repository)
        .arg("--output")
        .arg(scratch.0.join("validation"))
        .arg("--recovered-sources")
        .arg(&recovered)
        .output()
        .expect("python3 runs the pinned fixture validation generator");
    assert!(
        result.status.success(),
        "full-class runtime validation failed: {}\nstdout:\n{}\nstderr:\n{}",
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    println!("{}", String::from_utf8_lossy(&result.stdout));
    println!(
        "S2B_RECOVERED_RUNTIME={}",
        fs::read_to_string(scratch.0.join("validation/recovered-validation.json"))
            .expect("generator records recovered full-class runtime evidence")
    );
}
