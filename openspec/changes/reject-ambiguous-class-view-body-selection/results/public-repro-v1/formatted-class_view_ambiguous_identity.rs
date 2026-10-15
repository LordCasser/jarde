//! Candidate public-API regression; statically prepared, not compiled or executed.
//! Copy into tests/class_view_ambiguous_identity.rs in a separately authorized change.
//! No test-support feature, JDK, CLI, global counters, or external fixture is needed.

use jarde::*;
use jarde_jvm::ir::NoBodyKind;

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 24,
        archive_entries: 1024,
        entry_bytes: 1 << 24,
        read_bytes: 1 << 24,
        class_bytes: 1 << 24,
        attribute_bytes: 1 << 24,
        code_bytes: 1 << 24,
        result_items: 4096,
        output_bytes: 1 << 24,
        class_headers: 64,
        method_bodies: 64,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn budget() -> Budget {
    Budget::new(limits())
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the physical member-table fixture opens")
}

fn definition_of(engine: &Engine, snapshot: &ArtifactSnapshot) -> PhysicalDefinitionId {
    engine
        .list_class_declarations(snapshot, &PhysicalScope::SnapshotAll, &mut budget())
        .expect("the fixture declaration is listed")
        .items
        .first()
        .expect("the standalone class has one declaration")
        .definition
        .clone()
}

fn request(definition: &PhysicalDefinitionId, bodies: Vec<BodyRef>) -> ClassViewRequest {
    ClassViewRequest {
        class: ClassRef::Definition {
            definition: definition.clone(),
        },
        bodies,
    }
}

fn listed(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
) -> ClassViewReport {
    match engine
        .class_view(
            snapshot,
            &PhysicalScope::SnapshotAll,
            &request(definition, Vec::new()),
            &mut budget(),
        )
        .expect("listing records without requesting a body succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("expected a class view, got {other:?}"),
    }
}

// Copied verbatim from the validated method-v1 candidate's selection_class helper.
// javac cannot emit duplicate signatures; only the declared members/order vary.
fn selection_class(members: &[(u16, u16, u16, bool)]) -> Vec<u8> {
    fn u2(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    let mut bytes = vec![0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 52];
    u2(&mut bytes, 10);
    for (index, entry) in [
        &b"Selection"[..],
        &b""[..],
        &b"java/lang/Object"[..],
        &b""[..],
        &b"m"[..],
        &b"()V"[..],
        &b"Code"[..],
        &b"other"[..],
        &b"(I)V"[..],
    ]
    .iter()
    .enumerate()
    {
        if index == 1 || index == 3 {
            bytes.push(7);
            u2(&mut bytes, index as u16);
        } else {
            bytes.push(1);
            u2(&mut bytes, entry.len() as u16);
            bytes.extend_from_slice(entry);
        }
    }
    for value in [0x0421, 2, 4, 0, 0, members.len() as u16] {
        u2(&mut bytes, value);
    }
    for &(flags, name, descriptor, code) in members {
        for value in [flags, name, descriptor, u16::from(code)] {
            u2(&mut bytes, value);
        }
        if code {
            u2(&mut bytes, 7);
            bytes.extend_from_slice(&13_u32.to_be_bytes());
            // max_stack=0, max_locals=1, code_length=1, return, no exceptions/attributes.
            bytes.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0, 1, 0xb1, 0, 0, 0, 0]);
        }
    }
    u2(&mut bytes, 0);
    bytes
}

fn assert_input_code(error: Error, expected: &str) {
    match error {
        Error::InvalidInput { code, .. } => assert_eq!(code, expected),
        other => panic!("expected {expected}, got {other:?}"),
    }
}

#[test]
fn duplicate_listed_identity_is_rejected_before_no_code_admission() {
    let abstract_member = (0x0401, 5, 6, false);
    let native_member = (0x0109, 5, 6, false);
    let code_member = (0x0009, 5, 6, true);
    let cases = [
        ("abstract-code", [abstract_member, code_member]),
        ("code-abstract", [code_member, abstract_member]),
        ("native-code", [native_member, code_member]),
        ("code-native", [code_member, native_member]),
        ("abstract-abstract", [abstract_member, abstract_member]),
        ("native-native", [native_member, native_member]),
        ("abstract-native", [abstract_member, native_member]),
        ("native-abstract", [native_member, abstract_member]),
        ("code-code", [code_member, code_member]),
    ];
    let engine = Engine::new();
    for (label, members) in cases {
        let snapshot = open(&selection_class(&members));
        let definition = definition_of(&engine, &snapshot);
        let listing = listed(&engine, &snapshot, &definition);
        let methods: Vec<_> = listing.methods().cloned().collect();
        assert_eq!(methods.len(), 2, "{label}");
        assert_eq!((methods[0].index, methods[1].index), (0, 1), "{label}");
        // The listing retains two ordinals, but BodyRef::Method carries only this identity.
        assert_eq!(methods[0].identity, methods[1].identity, "{label}");

        // Positive control: exact name+descriptor selection already keeps both candidates.
        let mut named_budget = budget();
        let named = engine
            .class_view(
                &snapshot,
                &PhysicalScope::SnapshotAll,
                &request(
                    &definition,
                    vec![BodyRef::Name {
                        name: JvmBytes(b"m".to_vec()),
                        descriptor: Some(JvmBytes(b"()V".to_vec())),
                    }],
                ),
                &mut named_budget,
            )
            .expect("duplicate name lookup answers with candidates");
        let OperationOutcome::Ambiguous(candidates) = named else {
            panic!("{label}: expected named ambiguity, got {named:?}");
        };
        assert_eq!(candidates.candidates.len(), 2, "{label}");
        assert_eq!(named_budget.usage().method_bodies, 0, "{label}");
        assert_eq!(named_budget.usage().code_bytes, 0, "{label}");

        // Proposed minimal contract: identity selection keeps both candidates, like Name,
        // before a first header can elect NoCode or charge a decode. This is expected to FAIL
        // on unpatched 644882d for abstract-code: old code returns Performed/NotDeclared.
        for method in methods {
            let mut identity_budget = budget();
            let result = engine
                .class_view(
                    &snapshot,
                    &PhysicalScope::SnapshotAll,
                    &request(
                        &definition,
                        vec![BodyRef::Method {
                            method: method.identity,
                        }],
                    ),
                    &mut identity_budget,
                )
                .expect("duplicate identity returns the ambiguity candidates");
            let OperationOutcome::Ambiguous(candidates) = result else {
                panic!("{label}: a duplicate identity must not elect a body: {result:?}");
            };
            assert_eq!(candidates.candidates.len(), 2, "{label}");
            assert_eq!(identity_budget.usage().method_bodies, 0, "{label}");
            assert_eq!(identity_budget.usage().code_bytes, 0, "{label}");
        }
    }
}

#[test]
fn unique_abstract_native_and_code_identity_bodies_keep_their_meaning() {
    let engine = Engine::new();
    for (flags, has_code, kind) in [
        (0x0401, false, Some(NoBodyKind::Abstract)),
        (0x0109, false, Some(NoBodyKind::Native)),
        (0x0009, true, None),
    ] {
        let snapshot = open(&selection_class(&[(flags, 5, 6, has_code)]));
        let definition = definition_of(&engine, &snapshot);
        let identity = listed(&engine, &snapshot, &definition)
            .methods()
            .next()
            .expect("one method")
            .identity
            .clone();
        let mut request_budget = budget();
        let result = engine
            .class_view(
                &snapshot,
                &PhysicalScope::SnapshotAll,
                &request(
                    &definition,
                    vec![BodyRef::Method {
                        method: identity.clone(),
                    }],
                ),
                &mut request_budget,
            )
            .expect("one exact declaration is not ambiguous");
        let OperationOutcome::Performed(report) = result else {
            panic!("unique member must be performed: {result:?}");
        };
        assert_eq!(report.bodies.len(), 1);
        match &report.bodies[0] {
            ClassViewBody::Read { method, .. } if has_code => assert_eq!(method, &identity),
            ClassViewBody::NotDeclared {
                method,
                no_body_kind,
            } if !has_code => {
                assert_eq!(method, &identity);
                assert_eq!(no_body_kind, &kind);
            }
            other => panic!("unexpected unique body: {other:?}"),
        }
        assert_eq!(request_budget.usage().method_bodies, u64::from(has_code));
    }
}

#[test]
fn foreign_owner_is_rejected_before_duplicate_selection() {
    let engine = Engine::new();
    let snapshot = open(&selection_class(&[
        (0x0401, 5, 6, false),
        (0x0009, 5, 6, true),
    ]));
    let definition = definition_of(&engine, &snapshot);
    let foreign = open(&selection_class(&[(0x0109, 5, 6, false)]));
    let foreign_definition = definition_of(&engine, &foreign);
    let foreign_identity = listed(&engine, &foreign, &foreign_definition)
        .methods()
        .next()
        .expect("one foreign method")
        .identity
        .clone();
    assert_ne!(foreign_identity.owner, definition);
    let mut request_budget = budget();
    let error = engine
        .class_view(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &request(
                &definition,
                vec![BodyRef::Method {
                    method: foreign_identity,
                }],
            ),
            &mut request_budget,
        )
        .expect_err("foreign owner must fail before duplicate selection");
    assert_input_code(error, "class_view_body_foreign_owner");
    assert_eq!(request_budget.usage().method_bodies, 0);
    assert_eq!(request_budget.usage().code_bytes, 0);
}
