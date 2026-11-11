//! Real MethodIr controls for declared generic member and SSA-value type binding.
//! This tests type facts only; it does not execute methods or admit a source projection.

use jarde_java::generic_types::{
    declared_call_binding, declared_value_types, proved_runtime_upcast, spell_declared_type,
};
use jarde_java::report::{
    DeclaredGenericMemberTypes, DeclaredGenericType, DeclaredGenericTypes, RecoveryContent,
};
use jarde_java::source_proof::{DependencyResult, RecoveryHeaders, ReferenceClassHeader};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_jvm::method_ir::MethodIr;
use jarde_jvm::resolver::{ReferenceUse, ResolvedMemberRef};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::classfile::{
    class_constant_pool, class_member_facts, cp_class_name, method_code_facts,
};
use jarde_reader::error::Error;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant, SymbolRef,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};
use std::collections::{BTreeMap, HashSet, VecDeque};

const CHILD_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-values-r2/classes/flow/Child.class");
const CAST_CHILD_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-values-r4/classes/flow/Child.class");
const MID_BYTES: &[u8] = include_bytes!("fixtures/s2b-declared-values-r1/classes/flow/Mid.class");
const BASE_BYTES: &[u8] = include_bytes!("fixtures/s2b-declared-values-r1/classes/flow/Base.class");
const CHILD_FOO_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/identity-a/classes/identity/Foo.class");
const PARENT_FOO_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/identity-b/classes/identity/Foo.class");
const LIST_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/platform/java/util/List.class");
const OBJECT_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/platform/java/lang/Object.class");
const STRING_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/platform/java/lang/String.class");
const R5_BASE_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-local-consumer-r5/classes/flow/Base.class");
const R5_CONSUMER_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-local-consumer-r5/classes/flow/Consumer.class");
const BOX_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-static-generic-members-r1/classes/flow/Box.class");
const BOX_CONSUMER_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-static-generic-members-r1/classes/flow/BoxConsumer.class");
const RAW_CHILD_BOX_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-static-generic-members-r1/classes/flow/RawChildBox.class");
const STRING_CHILD_BOX_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-static-generic-members-r1/classes/flow/StringChildBox.class");
const CALL_BOX_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-call-binding-r1/classes/flow/CallBox.class");
const CALL_BOX_CONSUMER_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-call-binding-r1/classes/flow/CallBoxConsumer.class");
const VARARG_BOX_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-call-binding-r1/classes/flow/VarargBox.class");
const BRIDGE_BASE_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-call-binding-r1/classes/flow/BridgeBase.class");
const BRIDGE_CHILD_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-call-binding-r1/classes/flow/BridgeChild.class");
const KEEP_BASE_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-call-binding-r1/classes/flow/KeepBase.class");
const KEEP_CHILD_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-declared-call-binding-r1/classes/flow/KeepChild.class");
const CALL_ADAPTATION_BYTES: &[u8] = include_bytes!(
    "fixtures/s2b-declared-call-adaptation-r1/classes/flow/CallAdaptationControls.class"
);
const RETURN_CONSUMPTION_BYTES: &[u8] = include_bytes!(
    "fixtures/s2b-return-consumption-r1/classes/flow/ReturnConsumptionControls.class"
);
const CONFLICTING_RETURN_BYTES: &[u8] = include_bytes!(
    "fixtures/s2b-conflicting-return-r1/classes/flow/ConflictingReturnControls.class"
);
const OWNED_FIELD_CYCLE_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-owned-field-cycle-r1/classes/flow/OwnedFieldCycle.class",);
const FIELD_BOX_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-field-consumption-r1/classes/flow/FieldBox.class");
const FIELD_DERIVED_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-field-consumption-r1/classes/flow/FieldDerived.class");
const FIELD_CONSUMER_BYTES: &[u8] =
    include_bytes!("fixtures/s2b-field-consumption-r1/classes/flow/FieldConsumer.class");
const R5_PLATFORM_HEADER_BYTES: &[(&[u8], &[u8])] = &[
    (
        b"java/io/Serializable",
        include_bytes!(
            "fixtures/s2b-declared-local-consumer-r5/platform/java/io/Serializable.class"
        ),
    ),
    (
        b"java/lang/CharSequence",
        include_bytes!(
            "fixtures/s2b-declared-local-consumer-r5/platform/java/lang/CharSequence.class"
        ),
    ),
    (
        b"java/lang/Cloneable",
        include_bytes!(
            "fixtures/s2b-declared-local-consumer-r5/platform/java/lang/Cloneable.class"
        ),
    ),
    (
        b"java/lang/Comparable",
        include_bytes!(
            "fixtures/s2b-declared-local-consumer-r5/platform/java/lang/Comparable.class"
        ),
    ),
    (
        b"java/lang/Iterable",
        include_bytes!("fixtures/s2b-declared-local-consumer-r5/platform/java/lang/Iterable.class"),
    ),
    (
        b"java/lang/Object",
        include_bytes!("fixtures/s2b-declared-local-consumer-r5/platform/java/lang/Object.class"),
    ),
    (
        b"java/lang/String",
        include_bytes!("fixtures/s2b-declared-local-consumer-r5/platform/java/lang/String.class"),
    ),
    (
        b"java/util/AbstractCollection",
        include_bytes!(
            "fixtures/s2b-declared-local-consumer-r5/platform/java/util/AbstractCollection.class"
        ),
    ),
    (
        b"java/util/AbstractList",
        include_bytes!(
            "fixtures/s2b-declared-local-consumer-r5/platform/java/util/AbstractList.class"
        ),
    ),
    (
        b"java/util/ArrayList",
        include_bytes!(
            "fixtures/s2b-declared-local-consumer-r5/platform/java/util/ArrayList.class"
        ),
    ),
    (
        b"java/util/Collection",
        include_bytes!(
            "fixtures/s2b-declared-local-consumer-r5/platform/java/util/Collection.class"
        ),
    ),
    (
        b"java/util/List",
        include_bytes!("fixtures/s2b-declared-local-consumer-r5/platform/java/util/List.class"),
    ),
    (
        b"java/util/RandomAccess",
        include_bytes!(
            "fixtures/s2b-declared-local-consumer-r5/platform/java/util/RandomAccess.class"
        ),
    ),
];

const CHILD_LOADER: &str = "s2b-child";
const PARENT_LOADER: &str = "s2b-parent";
const BOOT_LOADER: &str = "s2b-bootstrap";

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 22,
        archive_entries: 100,
        entry_bytes: 1 << 22,
        read_bytes: 1 << 22,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 64,
        method_bodies: 16,
        ir_items: 1 << 18,
        ir_edges: 1 << 18,
        analysis_steps: 1 << 18,
        normalization_clones: 0,
        nested_depth: 32,
        dependency_depth: 32,
        elapsed_millis: u64::MAX,
    }
}

fn snapshot(bytes: &[u8]) -> ArtifactSnapshot {
    ArtifactSnapshot::open(
        ArtifactInput::bytes(bytes.to_vec()),
        &mut Budget::new(limits()),
    )
    .expect("fixture bytes open as a physical snapshot")
}

fn physical_definition(bytes: &[u8], snapshot: &ArtifactSnapshot) -> PhysicalDefinitionId {
    PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: u64::try_from(bytes.len()).expect("fixture length fits"),
        },
        variant: PhysicalVariant::Base,
    }
}

fn header(bytes: &[u8], loader: &str) -> ReferenceClassHeader {
    let mut budget = Budget::new(limits());
    let facts = class_member_facts(bytes, &mut budget).expect("physical class facts decode");
    let pool = class_constant_pool(bytes, &budget).expect("physical class constant pool decodes");
    let snapshot = snapshot(bytes);
    ReferenceClassHeader::from_facts(
        &facts,
        LoaderId(loader.to_owned()),
        physical_definition(bytes, &snapshot),
    )
    .expect("complete physical header")
    .with_generic_signatures(bytes, &facts.attributes, &pool, &mut budget)
    .expect("real Signature attributes agree with their physical erasures")
}

fn domain(loader: &str, parent: Option<&str>, roots: Vec<ArtifactSnapshot>) -> LoadDomain {
    LoadDomain {
        loader: LoaderId(loader.to_owned()),
        parent_loader: parent.map(|parent| LoaderId(parent.to_owned())),
        delegation: DelegationPolicy::ParentFirst,
        roots: roots
            .iter()
            .map(|snapshot| LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            })
            .collect(),
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    }
}

struct Fixture {
    content: Vec<ArtifactSnapshot>,
    child_definition: PhysicalDefinitionId,
    environment: ResolutionEnvironment,
    headers: Headers,
}

impl Fixture {
    fn new() -> Self {
        Self::with_child(CHILD_BYTES)
    }

    fn with_child(child_bytes: &[u8]) -> Self {
        Self::with_child_and_base(child_bytes, BASE_BYTES, false)
    }

    fn with_child_and_base(
        child_bytes: &[u8],
        base_bytes: &[u8],
        full_platform_headers: bool,
    ) -> Self {
        let child = snapshot(child_bytes);
        let mid_snapshot = snapshot(MID_BYTES);
        let base_snapshot = snapshot(base_bytes);
        let child_foo_snapshot = snapshot(CHILD_FOO_BYTES);
        let parent_foo_snapshot = snapshot(PARENT_FOO_BYTES);
        let list_snapshot = snapshot(LIST_BYTES);
        let object_snapshot = snapshot(OBJECT_BYTES);
        let string_snapshot = snapshot(STRING_BYTES);
        let content = vec![
            child.clone(),
            mid_snapshot.clone(),
            base_snapshot.clone(),
            child_foo_snapshot.clone(),
            parent_foo_snapshot.clone(),
            list_snapshot.clone(),
            object_snapshot.clone(),
            string_snapshot.clone(),
        ];
        let child_domain = domain(
            CHILD_LOADER,
            Some(PARENT_LOADER),
            vec![child.clone(), child_foo_snapshot.clone()],
        );
        let parent_domain = domain(
            PARENT_LOADER,
            Some(BOOT_LOADER),
            vec![
                mid_snapshot.clone(),
                base_snapshot.clone(),
                parent_foo_snapshot,
            ],
        );
        let boot_domain = domain(
            BOOT_LOADER,
            None,
            vec![list_snapshot, object_snapshot, string_snapshot],
        );
        let environment = ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: child.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: child_domain.clone(),
            },
            domains: vec![child_domain.clone(), parent_domain, boot_domain],
            providers: Vec::new(),
        };
        let headers = Headers::new(child_bytes, base_bytes, full_platform_headers);
        let child_definition = physical_definition(child_bytes, &child);
        Self {
            content,
            child_definition,
            environment,
            headers,
        }
    }

    fn box_members() -> Self {
        let mut fixture = Self::with_child_and_base(BOX_CONSUMER_BYTES, BASE_BYTES, true);
        for bytes in [BOX_BYTES, RAW_CHILD_BOX_BYTES, STRING_CHILD_BOX_BYTES] {
            let artifact = snapshot(bytes);
            let facts = class_member_facts(bytes, &mut Budget::new(limits()))
                .expect("frozen Box-family class header decodes");
            let owner = facts.this_class.raw().0.clone();
            let selected = header(bytes, CHILD_LOADER);
            fixture.headers.insert(CHILD_LOADER, &owner, selected);
            fixture
                .environment
                .runtime
                .load_domain
                .roots
                .push(LoadRoot::StandaloneClass {
                    snapshot: artifact.id().clone(),
                });
            fixture.content.push(artifact.clone());
            if let Some(domain) = fixture
                .environment
                .domains
                .iter_mut()
                .find(|domain| domain.loader.0 == CHILD_LOADER)
            {
                domain.roots.push(LoadRoot::StandaloneClass {
                    snapshot: artifact.id().clone(),
                });
            }
        }
        fixture
    }

    fn call_binding() -> Self {
        let mut fixture = Self::with_child_and_base(CALL_BOX_CONSUMER_BYTES, BASE_BYTES, true);
        for bytes in [
            CALL_BOX_BYTES,
            VARARG_BOX_BYTES,
            BRIDGE_BASE_BYTES,
            BRIDGE_CHILD_BYTES,
            KEEP_BASE_BYTES,
            KEEP_CHILD_BYTES,
        ] {
            let artifact = snapshot(bytes);
            let facts = class_member_facts(bytes, &mut Budget::new(limits()))
                .expect("frozen call-binding class header decodes");
            let owner = facts.this_class.raw().0.clone();
            fixture
                .headers
                .insert(CHILD_LOADER, &owner, header(bytes, CHILD_LOADER));
            fixture
                .environment
                .runtime
                .load_domain
                .roots
                .push(LoadRoot::StandaloneClass {
                    snapshot: artifact.id().clone(),
                });
            fixture.content.push(artifact.clone());
            if let Some(domain) = fixture
                .environment
                .domains
                .iter_mut()
                .find(|domain| domain.loader.0 == CHILD_LOADER)
            {
                domain.roots.push(LoadRoot::StandaloneClass {
                    snapshot: artifact.id().clone(),
                });
            }
        }
        fixture
    }

    fn field_members() -> Self {
        let mut fixture = Self::with_child_and_base(FIELD_CONSUMER_BYTES, BASE_BYTES, true);
        for bytes in [FIELD_BOX_BYTES, FIELD_DERIVED_BYTES] {
            let artifact = snapshot(bytes);
            let facts = class_member_facts(bytes, &mut Budget::new(limits()))
                .expect("frozen field-family class header decodes");
            let owner = facts.this_class.raw().0.clone();
            fixture
                .headers
                .insert(CHILD_LOADER, &owner, header(bytes, CHILD_LOADER));
            fixture
                .environment
                .runtime
                .load_domain
                .roots
                .push(LoadRoot::StandaloneClass {
                    snapshot: artifact.id().clone(),
                });
            fixture.content.push(artifact.clone());
            if let Some(domain) = fixture
                .environment
                .domains
                .iter_mut()
                .find(|domain| domain.loader.0 == CHILD_LOADER)
            {
                domain.roots.push(LoadRoot::StandaloneClass {
                    snapshot: artifact.id().clone(),
                });
            }
        }
        fixture
    }

    fn analyze(&self, name: &str, descriptor: &str) -> jarde_jvm::method_ir::MethodIrAnalysis {
        let method = PhysicalMethodId {
            owner: self.child_definition.clone(),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        };
        let analysis = analyze_method_ir(
            &self.content,
            &MethodAnalysisRequest {
                environment: self.environment.clone(),
                method,
                stages: AnalysisStage::ALL.to_vec(),
            },
            &mut Budget::new(limits()),
        )
        .expect("the actual fixture method passes the selected JVM analysis stages");
        let declaration = analysis
            .ir()
            .declaration()
            .expect("the requested fixture member was physically found");
        assert_eq!(declaration.name().0, name.as_bytes());
        assert_eq!(declaration.descriptor().0, descriptor.as_bytes());
        assert_eq!(
            declaration.identity(),
            &PhysicalMethodId {
                owner: self.child_definition.clone(),
                name: JvmBytes(name.as_bytes().to_vec()),
                descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
            },
            "analysis succeeded for the exact physical member, not a missing-method report"
        );
        analysis
    }

    fn value_types(&mut self, ir: &MethodIr) -> DeclaredGenericTypes {
        match declared_value_types(ir, &mut self.headers, &mut Budget::new(limits()))
            .expect("declared type demand preserves ordinary stops")
        {
            DependencyResult::Found(types) => types,
            DependencyResult::Missing => panic!("fixture dependency unexpectedly missing"),
            DependencyResult::Ambiguous => panic!("fixture dependency unexpectedly ambiguous"),
            DependencyResult::Unknown => panic!("fixture method type facts are complete"),
        }
    }

    fn recover_with_types(
        &self,
        ir: &MethodIr,
        types: &DeclaredGenericTypes,
    ) -> jarde_java::RecoveryReport {
        self.recover_with_optional_types(ir, Some(types))
    }

    fn recover_with_types_and_source_map(
        &self,
        ir: &MethodIr,
        types: &DeclaredGenericTypes,
    ) -> jarde_java::RecoveryReport {
        self.recover_with_evidence(
            ir,
            Some(types),
            jarde_java::RecoveryEvidenceRequest::essential()
                .with_kind(jarde_java::RecoveryEvidenceKind::SourceMap),
        )
    }

    fn recover_with_optional_types(
        &self,
        ir: &MethodIr,
        types: Option<&DeclaredGenericTypes>,
    ) -> jarde_java::RecoveryReport {
        self.recover_with_evidence(ir, types, jarde_java::RecoveryEvidenceRequest::essential())
    }

    fn recover_with_evidence(
        &self,
        ir: &MethodIr,
        types: Option<&DeclaredGenericTypes>,
        evidence: jarde_java::RecoveryEvidenceRequest,
    ) -> jarde_java::RecoveryReport {
        let declaration = ir
            .declaration()
            .expect("fixture MethodIr retains its actual method declaration");
        let class_name = std::str::from_utf8(&declaration.class_name().0)
            .expect("fixture class name uses ordinary UTF-8 bytes")
            .to_owned();
        let facts = jarde_java::RecoveryFacts::new(
            jarde_java::MethodFacts::from_raw(
                declaration.name().clone(),
                declaration.descriptor().clone(),
                declaration.parameter_slots(),
            )
            .with_access_flags(declaration.access_flags())
            .with_declaring_class(jarde_java::DeclaringClass::new(
                class_name,
                declaration.class_access_flags(),
            )),
        );
        let request = jarde_java::RecoveryRequest::new(
            ir,
            &facts,
            jarde_java::pass::JAVA_8,
            self.environment.runtime.profile.clone(),
        )
        .with_evidence(evidence);
        let request = if let Some(types) = types {
            request.with_declared_types(types)
        } else {
            request
        };
        jarde_java::recover(&request, &mut Budget::new(limits()))
    }

    fn recover_class_source(
        &self,
        ir: &MethodIr,
        types: Option<&DeclaredGenericTypes>,
        budget: &mut Budget,
    ) -> jarde_java::report::ClassSourceRecovery {
        self.recover_class_source_with_evidence(
            ir,
            types,
            jarde_java::RecoveryEvidenceRequest::essential(),
            budget,
        )
    }

    fn recover_class_source_with_evidence(
        &self,
        ir: &MethodIr,
        types: Option<&DeclaredGenericTypes>,
        evidence: jarde_java::RecoveryEvidenceRequest,
        budget: &mut Budget,
    ) -> jarde_java::report::ClassSourceRecovery {
        let declaration = ir
            .declaration()
            .expect("fixture MethodIr retains its actual method declaration");
        let class_name = std::str::from_utf8(&declaration.class_name().0)
            .expect("fixture class name uses ordinary UTF-8 bytes")
            .to_owned();
        let facts = jarde_java::RecoveryFacts::new(
            jarde_java::MethodFacts::from_raw(
                declaration.name().clone(),
                declaration.descriptor().clone(),
                declaration.parameter_slots(),
            )
            .with_access_flags(declaration.access_flags())
            .with_declaring_class(jarde_java::DeclaringClass::new(
                class_name,
                declaration.class_access_flags(),
            )),
        );
        let request = jarde_java::RecoveryRequest::new(
            ir,
            &facts,
            jarde_java::pass::JAVA_8,
            self.environment.runtime.profile.clone(),
        );
        let request = request.with_evidence(evidence);
        let request = if let Some(types) = types {
            request.with_declared_types(types)
        } else {
            request
        };
        jarde_java::report::recover_for_class_source(
            &request,
            budget,
            jarde_java::report::ClassSourceOptions::default(),
            None,
        )
    }
}

fn field_member_bci(types: &DeclaredGenericTypes, field: &str) -> u32 {
    let matches = types
        .members
        .iter()
        .filter_map(|(_, member)| {
            let DeclaredGenericMemberTypes::Field {
                bci, declaration, ..
            } = member
            else {
                return None;
            };
            let SymbolRef::Field { name, .. } = &declaration.member else {
                return None;
            };
            (name.0.as_slice() == field.as_bytes()).then_some(*bci)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "the fixture method has one physical use of {field}"
    );
    matches[0]
}

#[derive(Default)]
struct Headers {
    selected: BTreeMap<(String, Vec<u8>), ReferenceClassHeader>,
    unknown: HashSet<(String, Vec<u8>)>,
}

impl Headers {
    fn new(child_bytes: &[u8], base_bytes: &[u8], full_platform_headers: bool) -> Self {
        let child = header(child_bytes, CHILD_LOADER);
        let mid = header(MID_BYTES, PARENT_LOADER);
        let base = header(base_bytes, PARENT_LOADER);
        let child_foo = header(CHILD_FOO_BYTES, CHILD_LOADER);
        let parent_foo = header(PARENT_FOO_BYTES, PARENT_LOADER);
        let list = header(LIST_BYTES, BOOT_LOADER);
        let object = header(OBJECT_BYTES, BOOT_LOADER);
        let string = header(STRING_BYTES, BOOT_LOADER);
        let mut result = Self::default();
        for loader in [CHILD_LOADER, PARENT_LOADER] {
            result.insert(loader, b"flow/Mid", mid.clone());
            result.insert(loader, b"flow/Base", base.clone());
            result.insert(loader, b"java/util/List", list.clone());
            result.insert(loader, b"java/lang/Object", object.clone());
            result.insert(loader, b"java/lang/String", string.clone());
            result.insert(
                loader,
                b"identity/Foo",
                if loader == CHILD_LOADER {
                    child_foo.clone()
                } else {
                    parent_foo.clone()
                },
            );
        }
        let child_facts = class_member_facts(child_bytes, &mut Budget::new(limits()))
            .expect("child header facts decode");
        result.insert(CHILD_LOADER, &child_facts.this_class.raw().0, child);
        result.insert(CHILD_LOADER, b"identity/Foo", child_foo);
        result.insert(PARENT_LOADER, b"identity/Foo", parent_foo);
        result.insert(BOOT_LOADER, b"java/util/List", list);
        result.insert(BOOT_LOADER, b"java/lang/Object", object);
        result.insert(BOOT_LOADER, b"java/lang/String", string);
        if full_platform_headers {
            for (owner, bytes) in R5_PLATFORM_HEADER_BYTES {
                let selected = header(bytes, BOOT_LOADER);
                for loader in [CHILD_LOADER, PARENT_LOADER, BOOT_LOADER] {
                    result.insert(loader, owner, selected.clone());
                }
            }
        }
        result
    }

    fn insert(&mut self, loader: &str, owner: &[u8], header: ReferenceClassHeader) {
        self.selected
            .insert((loader.to_owned(), owner.to_vec()), header);
    }

    fn lookup(&self, loader: &LoaderId, owner: &[u8]) -> Option<&ReferenceClassHeader> {
        self.selected.get(&(loader.0.clone(), owner.to_vec()))
    }

    fn resolve_member_inner(
        &mut self,
        owner: &[u8],
        name: &[u8],
        descriptor: &[u8],
        field: bool,
        budget: &mut Budget,
    ) -> Result<DependencyResult<ResolvedMemberRef>, Error> {
        let mut pending = VecDeque::from([(self.caller_loader().clone(), owner.to_vec())]);
        let mut visited = HashSet::new();
        let mut incomplete = false;
        while let Some((loader, class_name)) = pending.pop_front() {
            budget.poll()?;
            if !visited.insert((loader.clone(), class_name.clone())) {
                continue;
            }
            let header = match self.selected_from(&loader, &class_name, budget)? {
                DependencyResult::Found(header) => header,
                DependencyResult::Missing => continue,
                DependencyResult::Ambiguous | DependencyResult::Unknown => {
                    incomplete = true;
                    continue;
                }
            };
            let table = if field {
                &header.fields
            } else {
                &header.methods
            };
            if table.iter().any(|member| {
                member.name.raw().0 == name && member.descriptor.raw().0 == descriptor
            }) {
                let member = if field {
                    SymbolRef::Field {
                        owner: JvmBytes(class_name),
                        name: JvmBytes(name.to_vec()),
                        descriptor: JvmBytes(descriptor.to_vec()),
                    }
                } else {
                    SymbolRef::Method {
                        owner: JvmBytes(class_name),
                        name: JvmBytes(name.to_vec()),
                        descriptor: JvmBytes(descriptor.to_vec()),
                    }
                };
                return Ok(DependencyResult::Found(ResolvedMemberRef {
                    loader: header.loader,
                    definition: header.definition,
                    member,
                }));
            }
            let next_loader = header.loader.clone();
            if let Some(parent) = &header.super_name {
                pending.push_back((next_loader.clone(), parent.clone()));
            }
            for interface in &header.interfaces {
                pending.push_back((next_loader.clone(), interface.clone()));
            }
        }
        Ok(if incomplete {
            DependencyResult::Unknown
        } else {
            DependencyResult::Missing
        })
    }
}

impl RecoveryHeaders for Headers {
    type Error = Error;

    fn caller_loader(&self) -> &LoaderId {
        static CHILD: std::sync::OnceLock<LoaderId> = std::sync::OnceLock::new();
        CHILD.get_or_init(|| LoaderId(CHILD_LOADER.to_owned()))
    }

    fn selected_from(
        &mut self,
        initiating_loader: &LoaderId,
        owner: &[u8],
        budget: &mut Budget,
    ) -> Result<DependencyResult<ReferenceClassHeader>, Self::Error> {
        budget.poll()?;
        if self
            .unknown
            .contains(&(initiating_loader.0.clone(), owner.to_vec()))
        {
            return Ok(DependencyResult::Unknown);
        }
        Ok(self
            .lookup(initiating_loader, owner)
            .cloned()
            .map_or(DependencyResult::Missing, DependencyResult::Found))
    }

    fn selected_generic_from(
        &mut self,
        initiating_loader: &LoaderId,
        owner: &[u8],
        budget: &mut Budget,
    ) -> Result<DependencyResult<ReferenceClassHeader>, Self::Error> {
        self.selected_from(initiating_loader, owner, budget)
    }

    fn resolve_member(
        &mut self,
        symbol: &SymbolRef,
        _use_kind: ReferenceUse,
        budget: &mut Budget,
    ) -> Result<DependencyResult<ResolvedMemberRef>, Self::Error> {
        match symbol {
            SymbolRef::Field {
                owner,
                name,
                descriptor,
            } => self.resolve_member_inner(&owner.0, &name.0, &descriptor.0, true, budget),
            SymbolRef::Method {
                owner,
                name,
                descriptor,
            } => self.resolve_member_inner(&owner.0, &name.0, &descriptor.0, false, budget),
            SymbolRef::Class { .. } => {
                budget.poll()?;
                Ok(DependencyResult::Unknown)
            }
        }
    }

    fn platform_unprovided(
        &mut self,
        _owner: &[u8],
        budget: &mut Budget,
    ) -> Result<bool, Self::Error> {
        budget.poll()?;
        Ok(true)
    }
}

fn foo_result(types: &DeclaredGenericTypes, node: usize) -> bool {
    matches!(
        types.types.get(node),
        Some(DeclaredGenericType::Reference { declaration, arguments })
            if arguments.is_empty()
                && declaration.loader == LoaderId(CHILD_LOADER.to_owned())
                && declaration.definition == header(CHILD_FOO_BYTES, CHILD_LOADER).definition
    )
}

fn invokeinterface_output(ir: &MethodIr) -> (u32, jarde_jvm::method_ir::ValueId, usize) {
    let ssa = ir.ssa().expect("SSA stage publishes a table");
    let mut calls = Vec::new();
    for block in ssa.blocks() {
        for instruction in block
            .instructions()
            .iter()
            .filter(|instruction| instruction.opcode() == 0xb9)
        {
            for (slot, value) in instruction.writes() {
                if matches!(slot, jarde_jvm::method_ir::Slot::Stack(_)) {
                    let index = ssa
                        .values_with_ids()
                        .position(|(candidate, _)| candidate == *value)
                        .expect("SSA output id belongs to the same table");
                    calls.push((instruction.bci(), *value, index));
                }
            }
        }
    }
    assert_eq!(
        calls.len(),
        1,
        "fixture method has one invokeinterface result"
    );
    calls[0]
}

fn stack_output_index_at_bci(ir: &MethodIr, bci: u32) -> usize {
    let ssa = ir.ssa().expect("SSA stage publishes a table");
    let mut outputs = Vec::new();
    for block in ssa.blocks() {
        for instruction in block.instructions().iter().filter(|item| item.bci() == bci) {
            for (slot, value) in instruction.writes() {
                if matches!(slot, jarde_jvm::method_ir::Slot::Stack(_)) {
                    outputs.push(
                        ssa.values_with_ids()
                            .position(|(candidate, _)| candidate == *value)
                            .expect("SSA output id belongs to the same table"),
                    );
                }
            }
        }
    }
    assert_eq!(outputs.len(), 1, "BCI {bci} has one stack result");
    outputs[0]
}

fn reference_node_for_header(
    types: &DeclaredGenericTypes,
    expected: &ReferenceClassHeader,
    arguments: &[usize],
) -> usize {
    types
        .types
        .iter()
        .position(|ty| {
            matches!(ty,
            DeclaredGenericType::Reference { declaration, arguments: actual }
                if actual == arguments
                    && declaration.loader == expected.loader
                    && declaration.definition == expected.definition)
        })
        .expect("the requested physical header identity has a bound type node")
}

fn field_type_for_member(types: &DeclaredGenericTypes, member_name: &[u8]) -> Option<usize> {
    types.members.iter().find_map(|(_, member)| match member {
        DeclaredGenericMemberTypes::Field {
            declaration:
                ResolvedMemberRef {
                    member: SymbolRef::Field { name, .. },
                    ..
                },
            ty,
            ..
        } if name.0 == member_name => Some(*ty),
        _ => None,
    })
}

fn method_types_for_member(
    types: &DeclaredGenericTypes,
    member_name: &[u8],
) -> Option<(Vec<usize>, Option<usize>)> {
    types.members.iter().find_map(|(_, member)| match member {
        DeclaredGenericMemberTypes::Method {
            declaration:
                ResolvedMemberRef {
                    member: SymbolRef::Method { name, .. },
                    ..
                },
            parameters,
            result,
            ..
        } if name.0 == member_name => Some((parameters.clone(), *result)),
        _ => None,
    })
}

fn method_binding_for_member(
    types: &DeclaredGenericTypes,
    member_name: &[u8],
) -> Option<DeclaredGenericMemberTypes> {
    types.members.iter().find_map(|(_, member)| match member {
        DeclaredGenericMemberTypes::Method {
            declaration:
                ResolvedMemberRef {
                    member: SymbolRef::Method { name, .. },
                    ..
                },
            ..
        } if name.0 == member_name => Some(member.clone()),
        _ => None,
    })
}

fn binding_bci(member: &DeclaredGenericMemberTypes) -> u32 {
    match member {
        DeclaredGenericMemberTypes::Method { bci, .. } => *bci,
        DeclaredGenericMemberTypes::Field { .. } => panic!("call binding requires a method"),
    }
}

fn has_unique_call(types: &DeclaredGenericTypes, member: &DeclaredGenericMemberTypes) -> bool {
    let bci = binding_bci(member);
    types
        .unique_calls
        .iter()
        .any(|(_, candidate)| *candidate == bci)
}

fn is_reference_node(
    types: &DeclaredGenericTypes,
    node: usize,
    expected: &ReferenceClassHeader,
    arguments: &[usize],
) -> bool {
    matches!(types.types.get(node),
        Some(DeclaredGenericType::Reference { declaration, arguments: actual })
            if actual == arguments
                && declaration.loader == expected.loader
                && declaration.definition == expected.definition)
}

fn invoke_output_for_opcode(ir: &MethodIr, opcode: u8) -> usize {
    let code = ir.code().expect("MethodIr retains decoded invocation Code");
    let instructions: Vec<_> = code
        .instructions
        .iter()
        .filter(|instruction| instruction.opcode == opcode)
        .collect();
    assert_eq!(
        instructions.len(),
        1,
        "fixture method has one opcode {opcode:#x}"
    );
    stack_output_index_at_bci(ir, instructions[0].bci)
}

fn has_reference_array_load(ir: &MethodIr) -> bool {
    ir.ssa().is_some_and(|ssa| {
        ssa.blocks().iter().any(|block| {
            block
                .instructions()
                .iter()
                .any(|instruction| instruction.opcode() == 0x32)
        })
    })
}

fn has_branch_phi(ir: &MethodIr) -> bool {
    ir.ssa()
        .is_some_and(|ssa| ssa.phis().iter().any(|phi| phi.inputs().len() >= 2))
}

fn generic_get_result(types: &DeclaredGenericTypes, bci: u32) -> usize {
    let matches: Vec<_> = types
        .members
        .iter()
        .filter_map(|(_, member)| match member {
            DeclaredGenericMemberTypes::Method {
                bci: member_bci,
                declaration,
                result: Some(result),
                ..
            } if *member_bci == bci
                && matches!(&declaration.member, SymbolRef::Method { name, descriptor, .. }
                    if name.0 == b"get" && descriptor.0 == b"(I)Ljava/lang/Object;") =>
            {
                Some(*result)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "the physical List.get use is recorded once"
    );
    matches[0]
}

fn decoded_method_code(
    bytes: &[u8],
    name: &str,
    descriptor: &str,
) -> jarde_reader::classfile::MethodCodeFacts {
    let mut budget = Budget::new(limits());
    let facts = class_member_facts(bytes, &mut budget).expect("fixture class member table decodes");
    let method = facts
        .methods
        .iter()
        .find(|member| {
            member.name.raw().0 == name.as_bytes()
                && member.descriptor.raw().0 == descriptor.as_bytes()
        })
        .expect("fixture method is physically declared");
    method_code_facts(bytes, method, &mut budget).expect("fixture method Code decodes")
}

fn checkcast_targets(ir: &MethodIr) -> Vec<(u32, Vec<u8>)> {
    ir.code()
        .expect("MethodIr retains the original decoded Code")
        .instructions
        .iter()
        .filter(|instruction| instruction.opcode == 0xc0)
        .map(|instruction| {
            let index = instruction
                .constant_pool_index
                .expect("checkcast names a CONSTANT_Class");
            (
                instruction.bci,
                cp_class_name(ir.constant_pool(), index)
                    .expect("checkcast constant-pool entry is a real class")
                    .0,
            )
        })
        .collect()
}

fn method_local_stores(ir: &MethodIr) -> Vec<(u32, u16)> {
    let code = ir.code().expect("fixture retains decoded method Code");
    code.instructions
        .iter()
        .zip(code.operands())
        .filter_map(|(instruction, operands)| {
            matches!(instruction.opcode, 0x36..=0x3a | 0x3b..=0x4e)
                .then(|| operands.local.map(|local| (instruction.bci, local.index)))
                .flatten()
        })
        .collect()
}

fn assert_methodir_code_matches_classfile(
    bytes: &[u8],
    ir: &MethodIr,
    name: &str,
    descriptor: &str,
) {
    let raw = decoded_method_code(bytes, name, descriptor);
    let analyzed = ir.code().expect("MethodIr retains decoded Code");
    let raw_instructions: Vec<_> = raw
        .instructions
        .iter()
        .map(|instruction| {
            (
                instruction.bci,
                instruction.opcode,
                instruction.width,
                instruction.constant_pool_index,
            )
        })
        .collect();
    let analyzed_instructions: Vec<_> = analyzed
        .instructions
        .iter()
        .map(|instruction| {
            (
                instruction.bci,
                instruction.opcode,
                instruction.width,
                instruction.constant_pool_index,
            )
        })
        .collect();
    assert_eq!(analyzed_instructions, raw_instructions);
}

fn assert_list_foo(types: &DeclaredGenericTypes, node: usize) {
    let DeclaredGenericType::Reference {
        declaration: list,
        arguments,
    } = &types.types[node]
    else {
        panic!("node is the physical List<Foo> type")
    };
    assert_eq!(list.loader, LoaderId(BOOT_LOADER.to_owned()));
    assert_eq!(list.definition, header(LIST_BYTES, BOOT_LOADER).definition);
    assert_eq!(arguments.len(), 1);
    let DeclaredGenericType::Reference {
        declaration: foo,
        arguments,
    } = &types.types[arguments[0]]
    else {
        panic!("List argument is the selected Foo type")
    };
    assert!(arguments.is_empty());
    assert_eq!(foo.loader, LoaderId(CHILD_LOADER.to_owned()));
    assert_eq!(
        foo.definition,
        header(CHILD_FOO_BYTES, CHILD_LOADER).definition
    );
}

fn assert_list_cast_is_bound(types: &DeclaredGenericTypes, ir: &MethodIr, bci: u32) {
    let cast_block = ir
        .ssa()
        .unwrap()
        .blocks()
        .iter()
        .find(|block| {
            block
                .instructions()
                .iter()
                .any(|instruction| instruction.bci() == bci)
        })
        .expect("checkcast BCI belongs to an analyzed canonical block")
        .block()
        .clone();
    let matches: Vec<_> = types
        .casts
        .iter()
        .filter_map(|(block, cast_bci, ty)| {
            (block == &cast_block && *cast_bci == bci).then_some(*ty)
        })
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "one cast fact retains the original c0 BCI"
    );
    assert_list_foo(types, matches[0]);
}

#[test]
fn parameter_copy_branch_phi_array_load_and_list_get_bind_child_foo() {
    let mut fixture = Fixture::new();
    let analysis = fixture.analyze(
        "knownFlow",
        "(Ljava/util/List;[Ljava/util/List;ZI)Lidentity/Foo;",
    );
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    assert!(
        has_branch_phi(ir),
        "both selected list sources reach a real SSA phi"
    );
    assert!(
        has_reference_array_load(ir),
        "the chosen list arrives through aaload"
    );
    let (get_bci, output, value_index) = invokeinterface_output(ir);
    let get_result = generic_get_result(&types, get_bci);
    assert!(foo_result(&types, get_result));
    assert!(
        types.values[value_index]
            .as_ref()
            .is_some_and(|value_types| value_types.contains(&get_result))
    );
    assert_eq!(
        output,
        ir.ssa()
            .unwrap()
            .values_with_ids()
            .nth(value_index)
            .unwrap()
            .0
    );
    assert!(types.members.iter().any(
        |(_, member)| matches!(member, DeclaredGenericMemberTypes::Method { declaration, .. }
            if matches!(&declaration.member, SymbolRef::Method { name, .. } if name.0 == b"get"))
    ));
}

#[test]
fn unknown_raw_array_phi_edge_blocks_generic_list_get_inference() {
    let mut fixture = Fixture::new();
    let analysis = fixture.analyze(
        "unknownFlow",
        "(Ljava/util/List;[Ljava/util/List;ZI)Ljava/lang/Object;",
    );
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    assert!(
        has_branch_phi(ir),
        "the known and raw list inputs reach a real SSA phi"
    );
    assert!(
        has_reference_array_load(ir),
        "the raw ingress is a real aaload value"
    );
    let (get_bci, _output, value_index) = invokeinterface_output(ir);
    assert!(!types.members.iter().any(|(_, member)| matches!(member,
        DeclaredGenericMemberTypes::Method { bci, declaration, .. }
            if *bci == get_bci && matches!(&declaration.member,
                SymbolRef::Method { name, descriptor, .. }
                    if name.0 == b"get" && descriptor.0 == b"(I)Ljava/lang/Object;"))));
    assert!(
        types.values[value_index].is_none(),
        "the unresolved receiver edge leaves get's result unbound"
    );
}

#[test]
fn null_receiver_does_not_create_a_generic_member_binding() {
    let mut fixture = Fixture::new();
    let analysis = fixture.analyze("nullOnly", "(I)Ljava/lang/Object;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let (get_bci, _output, value_index) = invokeinterface_output(ir);
    assert!(!types.members.iter().any(|(_, member)| matches!(member,
        DeclaredGenericMemberTypes::Method { bci, declaration, .. }
            if *bci == get_bci && matches!(&declaration.member,
                SymbolRef::Method { name, .. } if name.0 == b"get"))));
    assert!(
        types.values[value_index].is_none(),
        "null supplies no receiver type node"
    );
}

#[test]
fn inherited_generic_field_and_method_results_retain_base_declaration() {
    for (name, descriptor, field) in [
        ("inheritedField", "()Ljava/lang/Object;", true),
        ("inheritedMethod", "()Ljava/lang/Object;", false),
    ] {
        let mut fixture = Fixture::new();
        let analysis = fixture.analyze(name, descriptor);
        let ir = analysis.ir();
        let types = fixture.value_types(ir);
        let base = header(BASE_BYTES, PARENT_LOADER);
        let member = types.members.iter().find(|(_, member)| match member {
            DeclaredGenericMemberTypes::Field { declaration, .. } if field => {
                declaration.definition == base.definition
                    && matches!(&declaration.member, SymbolRef::Field { owner, name, descriptor }
                        if owner.0 == b"flow/Base" && name.0 == b"field" && descriptor.0 == b"Ljava/lang/Object;")
            }
            DeclaredGenericMemberTypes::Method { declaration, .. } if !field => {
                declaration.definition == base.definition
                    && matches!(&declaration.member, SymbolRef::Method { owner, name, descriptor }
                        if owner.0 == b"flow/Base" && name.0 == b"getBase" && descriptor.0 == b"()Ljava/lang/Object;")
            }
            _ => false,
        });
        let result = match &member
            .expect("member binds to the physical Base declaration")
            .1
        {
            DeclaredGenericMemberTypes::Field { ty, .. } => *ty,
            DeclaredGenericMemberTypes::Method {
                result: Some(ty), ..
            } => *ty,
            _ => panic!("inherited generic access has a result type"),
        };
        let DeclaredGenericType::Reference {
            declaration,
            arguments,
        } = &types.types[result]
        else {
            panic!("Base<T> result substitutes List<Foo>")
        };
        assert_eq!(declaration.loader, LoaderId(BOOT_LOADER.to_owned()));
        assert_eq!(
            declaration.definition,
            header(LIST_BYTES, BOOT_LOADER).definition
        );
        let DeclaredGenericType::Reference {
            declaration: foo,
            arguments,
        } = &types.types[arguments[0]]
        else {
            panic!("List argument remains the child Foo")
        };
        assert!(arguments.is_empty());
        assert_eq!(foo.loader, LoaderId(CHILD_LOADER.to_owned()));
        assert_eq!(
            foo.definition,
            header(CHILD_FOO_BYTES, CHILD_LOADER).definition
        );
    }
}

#[test]
fn inherited_getter_and_field_keep_real_list_checkcast_and_bind_get_result() {
    for name in ["fromGetter", "fromField"] {
        let mut fixture = Fixture::with_child(CAST_CHILD_BYTES);
        let analysis = fixture.analyze(name, "(I)Lidentity/Foo;");
        let ir = analysis.ir();
        assert_methodir_code_matches_classfile(CAST_CHILD_BYTES, ir, name, "(I)Lidentity/Foo;");
        let types = fixture.value_types(ir);
        let (get_bci, _output, value_index) = invokeinterface_output(ir);
        let get_result = generic_get_result(&types, get_bci);
        assert!(foo_result(&types, get_result));
        assert!(
            types.values[value_index]
                .as_ref()
                .is_some_and(|value_types| value_types.contains(&get_result))
        );

        let list_casts: Vec<_> = checkcast_targets(ir)
            .into_iter()
            .filter(|(_, target)| target == b"java/util/List")
            .collect();
        assert_eq!(
            list_casts.len(),
            1,
            "javac emits the inherited erasure checkcast"
        );
        let (cast_bci, exact_target) = &list_casts[0];
        assert_eq!(exact_target, b"java/util/List");
        assert_list_cast_is_bound(&types, ir, *cast_bci);
    }
}

#[test]
fn runtime_bounds_follow_physical_descriptors_until_original_checkcasts() {
    let mut fixture = Fixture::new();
    let child_foo_header = header(CHILD_FOO_BYTES, CHILD_LOADER);
    let parent_foo_header = header(PARENT_FOO_BYTES, PARENT_LOADER);
    let child_foo_ref = ResolvedMemberRef {
        loader: child_foo_header.loader.clone(),
        definition: child_foo_header.definition.clone(),
        member: SymbolRef::Class {
            owner: JvmBytes(b"identity/Foo".to_vec()),
        },
    };
    let parent_foo_ref = ResolvedMemberRef {
        loader: parent_foo_header.loader.clone(),
        definition: parent_foo_header.definition.clone(),
        member: SymbolRef::Class {
            owner: JvmBytes(b"identity/Foo".to_vec()),
        },
    };
    assert!(
        !jarde_java::source_proof::selected_header_widens(
            &child_foo_ref,
            &parent_foo_ref,
            &mut fixture.headers,
            &mut Budget::new(limits()),
        )
        .expect("same-spelled classes from distinct loaders are compared by selected headers")
    );

    let analysis = fixture.analyze(
        "knownFlow",
        "(Ljava/util/List;[Ljava/util/List;ZI)Lidentity/Foo;",
    );
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let foo = reference_node_for_header(&types, &child_foo_header, &[]);
    let object_header = header(OBJECT_BYTES, BOOT_LOADER);
    let object = reference_node_for_header(&types, &object_header, &[]);
    let (get_bci, _get_output, get_index) = invokeinterface_output(ir);
    let get_result = generic_get_result(&types, get_bci);
    assert!(foo_result(&types, get_result));
    assert!(
        types.values[get_index]
            .as_ref()
            .is_some_and(|bounds| bounds.contains(&get_result))
    );
    assert_eq!(types.runtime_values[get_index], Some(vec![object]));
    assert!(
        !proved_runtime_upcast(&types, get_index, get_result, &mut Budget::new(limits()))
            .expect("Signature substitution does not prove the runtime Object is Foo")
    );

    let foo_cast_bci = checkcast_targets(ir)
        .into_iter()
        .find_map(|(bci, target)| (target == b"identity/Foo").then_some(bci))
        .expect("knownFlow retains its physical Foo checkcast");
    let foo_cast_index = stack_output_index_at_bci(ir, foo_cast_bci);
    assert_eq!(types.runtime_values[foo_cast_index], Some(vec![foo]));
    assert!(
        proved_runtime_upcast(
            &types,
            foo_cast_index,
            get_result,
            &mut Budget::new(limits()),
        )
        .expect("the physical Foo checkcast supplies the runtime Foo bound")
    );

    let list_foo = types
        .types
        .iter()
        .position(|ty| matches!(ty,
            DeclaredGenericType::Reference { declaration, arguments }
                if arguments.as_slice() == [foo]
                    && matches!(&declaration.member, SymbolRef::Class { owner } if owner.0 == b"java/util/List")))
        .expect("knownFlow binds a real List<Foo> declaration node");
    let list_header = header(LIST_BYTES, BOOT_LOADER);
    let raw_list = reference_node_for_header(&types, &list_header, &[]);
    assert!(
        (0..types.values.len()).any(|index| {
            types.values[index]
                .as_ref()
                .is_some_and(|bounds| bounds.contains(&list_foo))
                && types.runtime_values[index]
                    .as_ref()
                    .is_some_and(|bounds| bounds.contains(&raw_list))
        }),
        "SSA phi/array list values keep the physical List descriptor beside List<Foo> source facts"
    );
    for method in ["fromGetter", "fromField"] {
        let mut fixture = Fixture::with_child(CAST_CHILD_BYTES);
        let analysis = fixture.analyze(method, "(I)Lidentity/Foo;");
        let ir = analysis.ir();
        let types = fixture.value_types(ir);
        let object = reference_node_for_header(&types, &object_header, &[]);
        let (list_cast_bci, _) = checkcast_targets(&ir)
            .into_iter()
            .find(|(_, target)| target == b"java/util/List")
            .expect("the inherited generic access keeps its original List checkcast");
        let before_cast_index = stack_output_index_at_bci(&ir, 1);
        assert_eq!(types.runtime_values[before_cast_index], Some(vec![object]));
        let list_cast_index = stack_output_index_at_bci(&ir, list_cast_bci);
        let list_target = types
            .casts
            .iter()
            .find_map(|(_, bci, target)| (*bci == list_cast_bci).then_some(*target))
            .expect("the cast fact keeps its canonical block and original BCI");
        assert!(
            proved_runtime_upcast(
                &types,
                before_cast_index,
                list_target,
                &mut Budget::new(limits()),
            )
            .map(|proved| !proved)
            .expect("an inherited generic Signature cannot strengthen the Object producer")
        );
        assert!(
            proved_runtime_upcast(
                &types,
                list_cast_index,
                list_target,
                &mut Budget::new(limits()),
            )
            .expect("the original c0List establishes the erased runtime List type")
        );

        let get_bci = invokeinterface_output(&ir).0;
        let get_result = generic_get_result(&types, get_bci);
        let foo_cast_bci = checkcast_targets(&ir)
            .into_iter()
            .find_map(|(bci, target)| (target == b"identity/Foo").then_some(bci))
            .expect("the returned element keeps its original Foo checkcast");
        let foo_cast_index = stack_output_index_at_bci(&ir, foo_cast_bci);
        assert!(
            proved_runtime_upcast(
                &types,
                foo_cast_index,
                get_result,
                &mut Budget::new(limits()),
            )
            .expect("the original c0Foo supplies the runtime result bound")
        );
    }

    let mut unknown_fixture = Fixture::new();
    let unknown_analysis = unknown_fixture.analyze(
        "unknownFlow",
        "(Ljava/util/List;[Ljava/util/List;ZI)Ljava/lang/Object;",
    );
    let unknown_ir = unknown_analysis.ir();
    let unknown_types = unknown_fixture.value_types(unknown_ir);
    let (_, _, unknown_index) = invokeinterface_output(unknown_ir);
    assert!(unknown_types.values[unknown_index].is_none());
    let declared_return =
        reference_node_for_header(&unknown_types, &header(OBJECT_BYTES, BOOT_LOADER), &[]);
    assert_eq!(
        unknown_types.runtime_values[unknown_index],
        Some(vec![declared_return])
    );

    let mut no_steps = limits();
    no_steps.analysis_steps = 0;
    assert!(matches!(
        proved_runtime_upcast(
            &types,
            foo_cast_index,
            get_result,
            &mut Budget::new(no_steps),
        ),
        Err(Error::BudgetExceeded { .. })
    ));
    let token = CancellationToken::new();
    token.cancel();
    assert!(matches!(
        proved_runtime_upcast(
            &types,
            foo_cast_index,
            get_result,
            &mut Budget::with_cancellation_token(limits(), token),
        ),
        Err(Error::Cancelled { .. })
    ));
}

#[test]
fn an_unknown_object_cast_to_raw_list_gets_only_the_erased_member_type() {
    let mut fixture = Fixture::with_child(CAST_CHILD_BYTES);
    let name = "unknownObject";
    let descriptor = "(Ljava/lang/Object;I)Ljava/lang/Object;";
    let analysis = fixture.analyze(name, descriptor);
    let ir = analysis.ir();
    assert_methodir_code_matches_classfile(CAST_CHILD_BYTES, ir, name, descriptor);
    let types = fixture.value_types(ir);
    let list_casts: Vec<_> = checkcast_targets(ir)
        .into_iter()
        .filter(|(_, target)| target == b"java/util/List")
        .collect();
    assert_eq!(
        list_casts.len(),
        1,
        "the original raw List c0 remains in Code"
    );
    let (cast_bci, _) = list_casts[0];
    assert!(
        !types
            .casts
            .iter()
            .any(|(_, candidate_bci, _)| *candidate_bci == cast_bci)
    );
    let (get_bci, _output, value_index) = invokeinterface_output(ir);
    let result = generic_get_result(&types, get_bci);
    let object = reference_node_for_header(&types, &header(OBJECT_BYTES, BOOT_LOADER), &[]);
    assert_eq!(
        result, object,
        "a raw member has its actual erased Object result, not inferred Foo"
    );
    assert_eq!(types.values[value_index], Some(vec![object]));
}

#[test]
fn class_and_method_variable_list_bounds_bind_get_without_method_inference() {
    for (name, expected_method_parameter) in [("fromClassBound", false), ("fromMethodBound", true)]
    {
        let mut fixture = Fixture::with_child(CAST_CHILD_BYTES);
        let analysis = fixture.analyze(name, "(Ljava/util/List;I)Lidentity/Foo;");
        let ir = analysis.ir();
        let types = fixture.value_types(ir);
        let (get_bci, _output, _value_index) = invokeinterface_output(ir);
        let get_result = generic_get_result(&types, get_bci);
        assert!(foo_result(&types, get_result));

        let variable = if expected_method_parameter {
            assert_eq!(types.method_parameters.len(), 1);
            types.method_parameters[0]
        } else {
            assert!(types.method_parameters.is_empty());
            assert_eq!(types.class_parameters.len(), 1);
            types.class_parameters[0]
        };
        let bounds = types
            .bounds
            .iter()
            .find(|(candidate, _)| *candidate == variable)
            .map(|(_, bounds)| bounds)
            .expect("declared variable carries its actual List<Foo> bound");
        assert_eq!(bounds.len(), 1);
        assert_list_foo(&types, bounds[0]);
    }
}

#[test]
fn generic_source_spelling_uses_the_method_namespace_definition() {
    let mut fixture = Fixture::new();
    let analysis = fixture.analyze(
        "knownFlow",
        "(Ljava/util/List;[Ljava/util/List;ZI)Lidentity/Foo;",
    );
    let mut types = fixture.value_types(analysis.ir());
    let mut budget = Budget::new(limits());

    let child_foo = types
        .types
        .iter()
        .position(|ty| matches!(ty,
            DeclaredGenericType::Reference { declaration, arguments }
                if arguments.is_empty()
                    && declaration.loader == LoaderId(CHILD_LOADER.to_owned())
                    && declaration.definition == header(CHILD_FOO_BYTES, CHILD_LOADER).definition
                    && matches!(&declaration.member, SymbolRef::Class { owner } if owner.0 == b"identity/Foo")))
        .expect("the actual method's declared List<Foo> input binds Child's Foo");
    assert_eq!(
        spell_declared_type(&types, child_foo, &mut budget).unwrap(),
        Some("identity.Foo".to_owned())
    );

    let parent_header = header(PARENT_FOO_BYTES, PARENT_LOADER);
    let parent_foo = types.types.len();
    types.types.push(DeclaredGenericType::Reference {
        declaration: ResolvedMemberRef {
            loader: parent_header.loader,
            definition: parent_header.definition,
            member: SymbolRef::Class {
                owner: JvmBytes(b"identity/Foo".to_vec()),
            },
        },
        arguments: Vec::new(),
    });
    assert_eq!(
        spell_declared_type(&types, parent_foo, &mut budget).unwrap(),
        None,
        "the same raw Foo name cannot spell the other loader's definition"
    );

    let list = types
        .types
        .iter()
        .position(|ty| matches!(ty,
            DeclaredGenericType::Reference { declaration, arguments }
                if arguments.as_slice() == [child_foo]
                    && matches!(&declaration.member, SymbolRef::Class { owner } if owner.0 == b"java/util/List")))
        .expect("the method has an actual parameterized List<Foo> type");
    assert_eq!(
        spell_declared_type(&types, list, &mut budget).unwrap(),
        Some("java.util.List<identity.Foo>".to_owned()),
        "a real Child Foo remains spellable as the argument of its List"
    );
}

#[test]
fn declared_types_supply_parameterized_local_types_to_the_public_recovery_request() {
    let mut fixture = Fixture::new();
    let analysis = fixture.analyze(
        "knownFlow",
        "(Ljava/util/List;[Ljava/util/List;ZI)Lidentity/Foo;",
    );
    let types = fixture.value_types(analysis.ir());
    let incomplete = fixture.recover_with_types(analysis.ir(), &types);
    assert!(
        !incomplete.text.contains("java.util.List<identity.Foo>"),
        "missing Collection/Iterable headers cannot close the source overload set: {}",
        incomplete.text
    );

    let mut fixture = Fixture::with_child_and_base(CHILD_BYTES, BASE_BYTES, true);
    let analysis = fixture.analyze(
        "knownFlow",
        "(Ljava/util/List;[Ljava/util/List;ZI)Lidentity/Foo;",
    );
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    assert!(has_branch_phi(ir));
    assert!(has_reference_array_load(ir));

    let report = fixture.recover_with_types(ir, &types);
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("java.util.List<identity.Foo> local6 = local5;"),
        "the known phi/array-derived local is parameterized from declared SSA facts:\n{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("return (identity.Foo) local6.get(arg4);"),
        "{}",
        report.text
    );

    let mut fixture = Fixture::with_child(CAST_CHILD_BYTES);
    let analysis = fixture.analyze("fromGetter", "(I)Lidentity/Foo;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let report = fixture.recover_with_types(ir, &types);
    let baseline = fixture.recover_with_optional_types(ir, None);
    assert_eq!(
        baseline.content,
        RecoveryContent::ContainsStatements,
        "{}",
        baseline.text
    );
    assert_eq!(
        report.content,
        RecoveryContent::ExplanationOnly,
        "an incomplete generic bound refuses the typed candidate, leaving its descriptor baseline available: {}",
        report.text
    );

    let mut fixture = Fixture::with_child_and_base(CAST_CHILD_BYTES, BASE_BYTES, true);
    let analysis = fixture.analyze("fromGetter", "(I)Lidentity/Foo;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let report = fixture.recover_with_types_and_source_map(ir, &types);
    let baseline = fixture.recover_with_evidence(
        ir,
        None,
        jarde_java::RecoveryEvidenceRequest::essential()
            .with_kind(jarde_java::RecoveryEvidenceKind::SourceMap),
    );
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("return (identity.Foo) ((java.util.List<identity.Foo>)")
            && report.text.contains(".getBase()).get(arg1);"),
        "complete headers admit the inherited declaring application while retaining both physical casts:\n{}",
        report.text
    );
    for (bci, _) in checkcast_targets(ir) {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "original checkcast BCI {bci} has a published origin"
        );
        assert!(
            report
                .source_map
                .of_bci(bci)
                .iter()
                .map(|segment| segment.mentions(bci))
                .collect::<Vec<_>>()
                == baseline
                    .source_map
                    .of_bci(bci)
                    .iter()
                    .map(|segment| segment.mentions(bci))
                    .collect::<Vec<_>>(),
            "declared types preserve the baseline source-origin entries for CHECKCAST BCI {bci}"
        );
    }
}

#[test]
fn raw_unknown_phi_does_not_gain_a_parameterized_local_from_known_inputs() {
    let mut fixture = Fixture::new();
    let analysis = fixture.analyze(
        "unknownFlow",
        "(Ljava/util/List;[Ljava/util/List;ZI)Ljava/lang/Object;",
    );
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    assert!(has_branch_phi(ir));
    assert!(has_reference_array_load(ir));

    let report = fixture.recover_with_types(ir, &types);
    assert_ne!(
        report.content,
        RecoveryContent::NotProduced,
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("java.util.List<identity.Foo> local"),
        "the unknown raw phi edge must not be filled from the other input:\n{}",
        report.text
    );
}

#[test]
fn declared_raw_field_local_preserves_the_object_overload_target() {
    let mut fixture = Fixture::with_child_and_base(R5_CONSUMER_BYTES, R5_BASE_BYTES, true);
    let analysis = fixture.analyze("rawFieldOverload", "(Lflow/Base;)I");
    let ir = analysis.ir();
    assert_eq!(method_local_stores(ir), vec![(1, 1)]);
    let types = fixture.value_types(ir);
    let report = fixture.recover_with_types(ir, &types);
    if report.content == RecoveryContent::ContainsStatements {
        assert!(
            report.text.contains("flow.Base local1 = arg0;"),
            "a typed Base<List<Foo>> copy changes baseValue from Object and selects a different overload:\n{}",
            report.text
        );
        assert!(
            report.text.contains("select(local1.baseValue)"),
            "{}",
            report.text
        );
    }
    assert!(
        !report
            .text
            .contains("Base<java.util.List<identity.Foo>> local1"),
        "the raw local must not be projected as Base<List<Foo>>:\n{}",
        report.text
    );
}

#[test]
fn r5_standard_library_headers_are_bootstrap_facts_through_both_parent_aliases() {
    let fixture = Fixture::with_child_and_base(R5_CONSUMER_BYTES, R5_BASE_BYTES, true);
    for (owner, bytes) in R5_PLATFORM_HEADER_BYTES {
        let expected = header(bytes, BOOT_LOADER);
        for initiator in [CHILD_LOADER, PARENT_LOADER, BOOT_LOADER] {
            let selected = fixture
                .headers
                .lookup(&LoaderId(initiator.to_owned()), owner)
                .expect("the frozen v3 standard-library closure is aliased for this initiator");
            assert_eq!(selected.loader, LoaderId(BOOT_LOADER.to_owned()));
            assert_eq!(selected.definition, expected.definition);
            assert_eq!(selected.super_name, expected.super_name);
            assert_eq!(selected.interfaces, expected.interfaces);
        }
    }
}

#[test]
fn declared_raw_list_local_keeps_object_argument_writable() {
    let mut fixture = Fixture::with_child_and_base(R5_CONSUMER_BYTES, R5_BASE_BYTES, true);
    let analysis = fixture.analyze("rawAdd", "(Ljava/util/List;Ljava/lang/Object;)V");
    let ir = analysis.ir();
    assert_eq!(method_local_stores(ir), vec![(1, 2)]);
    let types = fixture.value_types(ir);
    let report = fixture.recover_with_types(ir, &types);
    assert!(
        !report.text.contains("List<identity.Foo> local2"),
        "the actual raw List.add(Object) body cannot use a List<Foo> local for the Object argument:\n{}",
        report.text
    );
}

#[test]
fn declared_raw_list_locals_keep_generic_arguments_comparable() {
    let mut fixture = Fixture::with_child_and_base(R5_CONSUMER_BYTES, R5_BASE_BYTES, true);
    let analysis = fixture.analyze("rawListEquality", "(Ljava/util/List;Ljava/util/List;)Z");
    let ir = analysis.ir();
    assert_eq!(method_local_stores(ir), vec![(1, 2), (3, 3)]);
    let types = fixture.value_types(ir);
    let report = fixture.recover_with_types(ir, &types);
    let left_is_parameterized = report
        .text
        .contains("java.util.List<identity.Foo> local2 = arg0;");
    let right_is_parameterized = report
        .text
        .contains("java.util.List<java.lang.String> local3 = arg1;");
    assert!(
        !(left_is_parameterized && right_is_parameterized),
        "typed List<Foo> and List<String> locals are incomparable in Java; the recovered body must remain compilable:\n{}",
        report.text
    );
}

#[test]
fn raw_and_static_generic_members_follow_java_erasure_and_binders() {
    let mut fixture = Fixture::box_members();
    let object = header(OBJECT_BYTES, BOOT_LOADER);
    let list = header(LIST_BYTES, BOOT_LOADER);
    let string = header(STRING_BYTES, BOOT_LOADER);

    for (method, descriptor, field, expected) in [
        (
            "rawValue",
            "(Lflow/Box;)Ljava/lang/Object;",
            b"value".as_slice(),
            &object,
        ),
        (
            "rawFixed",
            "(Lflow/Box;)Ljava/lang/Object;",
            b"fixed".as_slice(),
            &list,
        ),
    ] {
        let analysis = fixture.analyze(method, descriptor);
        let types = fixture.value_types(analysis.ir());
        let ty = field_type_for_member(&types, field)
            .unwrap_or_else(|| panic!("{method} has a selected physical field binding"));
        assert!(
            is_reference_node(&types, ty, expected, &[]),
            "{method}: {:#?}",
            types.types[ty]
        );
    }

    let analysis = fixture.analyze(
        "rawEcho",
        "(Lflow/Box;Ljava/lang/Object;)Ljava/lang/Object;",
    );
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let (parameters, result) = method_types_for_member(&types, b"echo")
        .expect("raw Box echo is resolved under its erased descriptor");
    assert_eq!(parameters.len(), 1);
    assert!(is_reference_node(&types, parameters[0], &object, &[]));
    assert!(is_reference_node(
        &types,
        result.expect("erased echo has its physical Object result"),
        &object,
        &[]
    ));

    let analysis = fixture.analyze(
        "inheritedRawValue",
        "(Lflow/RawChildBox;)Ljava/lang/Object;",
    );
    let types = fixture.value_types(analysis.ir());
    let inherited_raw = field_type_for_member(&types, b"value")
        .expect("the raw superclass member resolves to Box.value");
    assert!(is_reference_node(&types, inherited_raw, &object, &[]));

    let analysis = fixture.analyze(
        "inheritedStringValue",
        "(Lflow/StringChildBox;)Ljava/lang/String;",
    );
    let types = fixture.value_types(analysis.ir());
    let inherited_string = field_type_for_member(&types, b"value")
        .expect("the parameterized superclass member resolves to Box.value");
    assert!(is_reference_node(&types, inherited_string, &string, &[]));

    let analysis = fixture.analyze("parameterizedFixed", "(Lflow/Box;)Ljava/util/List;");
    let types = fixture.value_types(analysis.ir());
    let list_string_element = reference_node_for_header(&types, &string, &[]);
    let list_string = types
        .types
        .iter()
        .enumerate()
        .find_map(|(index, _)| {
            is_reference_node(&types, index, &list, &[list_string_element]).then_some(index)
        })
        .expect("the fixed field Signature binds List<String>");
    let fixed_type = field_type_for_member(&types, b"fixed")
        .expect("parameterized Box.fixed retains its declared Signature");
    assert_eq!(fixed_type, list_string);

    let analysis = fixture.analyze(
        "parameterizedEcho",
        "(Lflow/Box;Ljava/lang/Object;)Ljava/lang/Object;",
    );
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    assert!(method_types_for_member(&types, b"echo").is_none());
    assert!(types.values[invoke_output_for_opcode(ir, 0xb6)].is_none());

    for (method, descriptor) in [
        ("staticField", "()Ljava/util/List;"),
        ("rawQualifierStaticField", "(Lflow/Box;)Ljava/util/List;"),
        ("typedQualifierStaticField", "(Lflow/Box;)Ljava/util/List;"),
    ] {
        let analysis = fixture.analyze(method, descriptor);
        let types = fixture.value_types(analysis.ir());
        let list_string_element = reference_node_for_header(&types, &string, &[]);
        let list_string = types
            .types
            .iter()
            .enumerate()
            .find_map(|(index, _)| {
                is_reference_node(&types, index, &list, &[list_string_element]).then_some(index)
            })
            .expect("static field Signature remains List<String>");
        assert_eq!(
            field_type_for_member(&types, b"fixedStatic"),
            Some(list_string),
            "{method} preserves the static field's own Signature"
        );
    }

    let analysis = fixture.analyze("staticPass", "(Ljava/util/List;)Ljava/util/List;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let list_string_element = reference_node_for_header(&types, &string, &[]);
    let list_string = types
        .types
        .iter()
        .enumerate()
        .find_map(|(index, _)| {
            is_reference_node(&types, index, &list, &[list_string_element]).then_some(index)
        })
        .expect("static method Signature binds List<String>");
    let (parameters, result) = method_types_for_member(&types, b"passthrough")
        .expect("non-generic static method Signature is retained");
    assert_eq!(parameters, vec![list_string]);
    assert_eq!(result, Some(list_string));

    let analysis = fixture.analyze("staticEcho", "(Ljava/lang/Object;)Ljava/lang/Object;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    assert!(method_types_for_member(&types, b"staticEcho").is_none());
    assert!(types.values[invoke_output_for_opcode(ir, 0xb8)].is_none());
}

#[test]
fn class_source_declared_call_requires_emitted_static_call_and_rejects_raw_write() {
    let mut fixture = Fixture::box_members();
    let analysis = fixture.analyze("staticPass", "(Ljava/util/List;)Ljava/util/List;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    assert!(
        types.unique_calls.iter().any(|(_, bci)| *bci == 1),
        "Box.passthrough at the real invokestatic site has a closed source overload set"
    );
    let class_source = fixture.recover_class_source(ir, Some(&types), &mut Budget::new(limits()));
    let ast = class_source
        .candidates
        .declared_ast
        .as_ref()
        .expect("staticPass retains its complete declared-source AST");
    assert!(
        jarde_java::report::class_source_declared_call_consumed(
            ast,
            1,
            &mut Budget::new(limits()),
        )
        .expect("the exact call-consumption check completes"),
        "the emitted static helper call consumes the proved declaration"
    );
    let (body, source_map) = jarde_java::report::emit_class_source_method_artifact(
        ast,
        ir.declaration()
            .expect("staticPass declaration exists")
            .access_flags(),
        jarde_java::SourceTarget::JAVA_8,
        &mut Budget::new(limits()),
    )
    .expect("the exact class-source AST emits through the existing artifact writer");
    assert!(body.contains("Box.passthrough(arg0)"), "{body}");
    assert_eq!(body.matches("Box.passthrough(arg0)").count(), 1, "{body}");
    assert!(
        !source_map.of_bci(1).is_empty(),
        "the call retains its physical BCI"
    );

    let mut raw_fixture = Fixture::with_child_and_base(CALL_ADAPTATION_BYTES, BASE_BYTES, true);
    let raw_analysis = raw_fixture.analyze("rawWrite", "(Ljava/util/List;Ljava/lang/Object;)V");
    let raw_ir = raw_analysis.ir();
    let raw_types = raw_fixture.value_types(raw_ir);
    let raw_class_source =
        raw_fixture.recover_class_source(raw_ir, Some(&raw_types), &mut Budget::new(limits()));
    let raw_call_consumed = raw_class_source
        .candidates
        .declared_ast
        .as_ref()
        .map(|raw_ast| {
            jarde_java::report::class_source_declared_call_consumed(
                raw_ast,
                4,
                &mut Budget::new(limits()),
            )
            .expect("the raw invocation site check completes")
        })
        .unwrap_or(false);
    assert!(
        !raw_call_consumed,
        "the List.add(Object) invocation through a raw local is not a proved source call"
    );
    let raw_report = raw_fixture.recover_with_types(raw_ir, &raw_types);
    assert_eq!(raw_report.content, RecoveryContent::ContainsStatements);
    assert!(raw_report.text.contains("java.util.List local2 = arg0;"));
    assert!(raw_report.text.contains("local2.add(arg1)"));
    assert!(!raw_report.text.contains("List<identity.Foo> local2"));
}

#[test]
fn declared_call_binding_checks_real_overload_sets_and_propagates_stops() {
    let mut fixture = Fixture::call_binding();
    let analysis = fixture.analyze("rawGet", "(Lflow/CallBox;)Ljava/lang/Object;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let get = method_binding_for_member(&types, b"get")
        .expect("CallBox.get has a real resolved member binding");
    assert!(
        has_unique_call(&types, &get),
        "the unique get call is published"
    );
    assert!(matches!(
        declared_call_binding(
            &mut types.clone(),
            &get,
            &mut fixture.headers,
            &mut Budget::new(limits())
        ),
        Ok(DependencyResult::Found(true))
    ));

    let analysis = fixture.analyze("rawFixed", "(Lflow/CallBox;)Ljava/util/List;");
    let types = fixture.value_types(analysis.ir());
    let fixed = method_binding_for_member(&types, b"fixed")
        .expect("CallBox.fixed() has a real resolved member binding");
    assert!(
        has_unique_call(&types, &fixed),
        "the different-arity fixed(String,String) overload does not block fixed()"
    );

    let analysis = fixture.analyze(
        "collisionSources",
        "(Lflow/CallBox;Lflow/CallBox;Ljava/lang/String;)V",
    );
    let mut types = fixture.value_types(analysis.ir());
    let set = method_binding_for_member(&types, b"set")
        .expect("the physical raw call selected CallBox.set(String)");
    let DeclaredGenericMemberTypes::Method {
        bci,
        declaration,
        application: _,
        parameters,
        result,
        throws,
        erased_parameters,
        erased_result,
    } = set.clone()
    else {
        unreachable!()
    };
    assert!(matches!(&declaration.member,
        SymbolRef::Method { owner, name, descriptor }
            if owner.0 == b"flow/CallBox"
                && name.0 == b"set"
                && descriptor.0 == b"(Ljava/lang/String;)V"));
    let string = header(STRING_BYTES, BOOT_LOADER);
    let call_box = header(CALL_BOX_BYTES, CHILD_LOADER);
    let string_type = reference_node_for_header(&types, &string, &[]);
    let typed_application = reference_node_for_header(&types, &call_box, &[string_type]);
    let typed_collision_member = DeclaredGenericMemberTypes::Method {
        bci,
        declaration,
        application: typed_application,
        parameters,
        result,
        throws,
        erased_parameters,
        erased_result,
    };
    assert!(
        matches!(
            declared_call_binding(
                &mut types,
                &typed_collision_member,
                &mut fixture.headers,
                &mut Budget::new(limits()),
            )
            .expect("the complete CallBox overload set is available"),
            DependencyResult::Found(false)
        ),
        "C<String>.set(T) and set(String) have colliding substituted source formals"
    );
    assert!(!has_unique_call(&types, &set));

    let analysis = fixture.analyze(
        "typedInfer",
        "(Lflow/CallBox;Ljava/lang/String;)Ljava/lang/Object;",
    );
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    assert!(method_binding_for_member(&types, b"infer").is_none());
    assert!(types.unique_calls.is_empty());
    assert!(types.values[invoke_output_for_opcode(ir, 0xb6)].is_none());

    let analysis = fixture.analyze("varargRun", "(Lflow/VarargBox;Ljava/lang/String;)V");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let run = method_binding_for_member(&types, b"run")
        .expect("the fixed-arity String run overload is the actual invocation");
    assert!(!has_unique_call(&types, &run));
    assert!(matches!(
        declared_call_binding(
            &mut types.clone(),
            &run,
            &mut fixture.headers,
            &mut Budget::new(limits())
        ),
        Ok(DependencyResult::Found(false))
    ));

    let analysis = fixture.analyze("bridgeEcho", "(Lflow/BridgeChild;Ljava/lang/String;)V");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let echo = method_binding_for_member(&types, b"echo")
        .expect("BridgeChild.echo(String) is the selected physical invocation");
    let bridge_header = header(BRIDGE_CHILD_BYTES, CHILD_LOADER);
    assert!(bridge_header.methods.iter().any(|method| {
        method.name.raw().0 == b"echo"
            && method.descriptor.raw().0 == b"(Ljava/lang/Object;)Ljava/lang/Object;"
            && method.access_flags & 0x0040 != 0
            && method.access_flags & 0x1000 != 0
    }));
    assert!(!has_unique_call(&types, &echo));
    assert!(matches!(
        declared_call_binding(
            &mut types.clone(),
            &echo,
            &mut fixture.headers,
            &mut Budget::new(limits())
        ),
        Ok(DependencyResult::Found(false))
    ));

    let analysis = fixture.analyze("inheritedKeep", "(Lflow/KeepChild;Ljava/lang/String;)V");
    let mut types = fixture.value_types(analysis.ir());
    let keep = method_binding_for_member(&types, b"keep")
        .expect("KeepChild.keep(String) overrides the same inherited descriptor");
    assert!(has_unique_call(&types, &keep));
    assert!(matches!(
        declared_call_binding(
            &mut types,
            &keep,
            &mut fixture.headers,
            &mut Budget::new(limits())
        ),
        Ok(DependencyResult::Found(true))
    ));

    let analysis = fixture.analyze("rawGet", "(Lflow/CallBox;)Ljava/lang/Object;");
    let mut missing_types = fixture.value_types(analysis.ir());
    let missing_get = method_binding_for_member(&missing_types, b"get")
        .expect("the member is captured before making its header unknown");
    fixture
        .headers
        .unknown
        .insert((CHILD_LOADER.to_owned(), b"flow/CallBox".to_vec()));
    assert!(matches!(
        declared_call_binding(
            &mut missing_types,
            &missing_get,
            &mut fixture.headers,
            &mut Budget::new(limits()),
        )
        .expect("header uncertainty is a value result, not a fabricated match"),
        DependencyResult::Unknown
    ));

    let mut fixture = Fixture::call_binding();
    let analysis = fixture.analyze("rawGet", "(Lflow/CallBox;)Ljava/lang/Object;");
    let mut types = fixture.value_types(analysis.ir());
    let get = method_binding_for_member(&types, b"get").expect("unique get member exists");
    let mut bounded = limits();
    bounded.ir_items = 0;
    assert!(matches!(
        declared_call_binding(
            &mut types,
            &get,
            &mut fixture.headers,
            &mut Budget::new(bounded)
        ),
        Err(Error::BudgetExceeded { .. })
    ));
    let token = CancellationToken::new();
    token.cancel();
    assert!(matches!(
        declared_call_binding(
            &mut types,
            &get,
            &mut fixture.headers,
            &mut Budget::with_cancellation_token(limits(), token),
        ),
        Err(Error::Cancelled { .. })
    ));
}

#[test]
fn declared_call_adaptation_controls_preserve_physical_casts_and_raw_writes() {
    let mut fixture = Fixture::with_child_and_base(CALL_ADAPTATION_BYTES, BASE_BYTES, true);

    let analysis = fixture.analyze(
        "typedAdd",
        "(Ljava/util/List;Lidentity/Foo;I)Lidentity/Foo;",
    );
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let report = fixture.recover_with_types_and_source_map(ir, &types);
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        report.text
    );
    assert!(
        report.text.contains("java.util.List<identity.Foo>"),
        "typedAdd must retain the parameterized local and receiver:\n{}",
        report.text
    );
    assert!(report.text.contains(".add(arg1)"), "{}", report.text);
    assert!(
        report.text.contains("identity.Foo") && report.text.contains(".get(arg2)"),
        "typedAdd retains its result type and List.get call:\n{}",
        report.text
    );
    let foo_cast_bci = checkcast_targets(ir)
        .into_iter()
        .find_map(|(bci, target)| (target == b"identity/Foo").then_some(bci))
        .expect("the source class contains its original physical Foo checkcast");
    assert!(
        !report.source_map.of_bci(foo_cast_bci).is_empty(),
        "public Recovery keeps a source origin for the original c0 Foo"
    );
    let analysis = fixture.analyze("genericForward", "(Ljava/util/List;Ljava/util/List;I)V");
    let ir = analysis.ir();
    assert!(
        checkcast_targets(ir).is_empty(),
        "the physical get-to-add bytecode has no Foo checkcast"
    );
    let types = fixture.value_types(ir);
    let report = fixture.recover_with_types(ir, &types);
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        report.text
    );
    assert!(report.text.contains(".get(arg2)"), "{}", report.text);
    assert!(report.text.contains(".add("), "{}", report.text);
    assert!(
        !report.text.contains("(identity.Foo)"),
        "genericForward must not invent a Foo checkcast absent from the original Code:\n{}",
        report.text
    );

    let analysis = fixture.analyze("rawWrite", "(Ljava/util/List;Ljava/lang/Object;)V");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let report = fixture.recover_with_types(ir, &types);
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("List<identity.Foo> local"),
        "rawWrite's physical local remains raw:\n{}",
        report.text
    );
    assert!(report.text.contains(".add(arg1)"), "{}", report.text);
}

#[test]
fn declared_return_consumption_keeps_bound_returns_and_physical_casts() {
    let mut fixture = Fixture::with_child_and_base(RETURN_CONSUMPTION_BYTES, R5_BASE_BYTES, true);

    let analysis = fixture.analyze("returnParameter", "(Ljava/util/List;)Ljava/util/List;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let report = fixture.recover_with_types(ir, &types);
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        report.text
    );
    assert!(
        report.text.contains("java.util.List<identity.Foo>"),
        "{}",
        report.text
    );
    assert!(report.text.contains("return local1;"), "{}", report.text);
    println!(
        "__RETURN_CONSUMPTION_PARAMETER_BEGIN__\n{}\n__RETURN_CONSUMPTION_PARAMETER_END__",
        report.text
    );

    let analysis = fixture.analyze("echo", "(Ljava/util/List;)Ljava/util/List;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    assert_eq!(
        types.method_parameters.len(),
        1,
        "the method T binder remains present"
    );
    assert_eq!(types.parameters.len(), 1);
    assert_eq!(types.result, types.parameters.first().map(|(_, ty)| *ty));
    let report = fixture.recover_with_types(ir, &types);
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        report.text
    );
    assert!(report.text.contains("return arg0;"), "{}", report.text);
    println!(
        "__RETURN_CONSUMPTION_ECHO_BEGIN__\n{}\n__RETURN_CONSUMPTION_ECHO_END__",
        report.text
    );

    let analysis = fixture.analyze("returnField", "(Lflow/Base;)Ljava/util/List;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let report = fixture.recover_with_types_and_source_map(ir, &types);
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        report.text
    );
    assert_eq!(
        report
            .text
            .matches("(java.util.List<identity.Foo>)")
            .count(),
        1,
        "the existing c0List is presented once, without a repeated return conversion:\n{}",
        report.text
    );
    let list_cast_bci = checkcast_targets(ir)
        .into_iter()
        .find_map(|(bci, target)| (target == b"java/util/List").then_some(bci))
        .expect("the original inherited-field Code contains c0List");
    assert_eq!(list_cast_bci, 4);
    assert!(!report.source_map.of_bci(list_cast_bci).is_empty());
    println!(
        "__RETURN_CONSUMPTION_FIELD_BEGIN__\n{}\n__RETURN_CONSUMPTION_FIELD_END__",
        report.text
    );

    let analysis = fixture.analyze("unchecked", "(Ljava/lang/Object;)Ljava/util/List;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let list_cast_bci = checkcast_targets(ir)
        .into_iter()
        .find_map(|(bci, target)| (target == b"java/util/List").then_some(bci))
        .expect("the descriptor baseline retains its physical raw List cast");
    assert_eq!(list_cast_bci, 1);
    assert!(
        !types.casts.iter().any(|(_, bci, _)| *bci == list_cast_bci),
        "the source-level List<Foo> enrichment is not a fabricated runtime cast fact"
    );
    let baseline = fixture.recover_with_optional_types(ir, None);
    let report = fixture.recover_with_types_and_source_map(ir, &types);
    assert_eq!(
        baseline.content,
        RecoveryContent::ContainsStatements,
        "{}",
        baseline.text
    );
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        report.text
    );
    assert_eq!(
        report
            .text
            .matches("(java.util.List<identity.Foo>)")
            .count(),
        1,
        "the original raw List c0 receives source generic arguments exactly once:\n{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("return (java.util.List<identity.Foo>) arg0;"),
        "{}",
        report.text
    );
    assert!(!report.source_map.of_bci(list_cast_bci).is_empty());
    println!(
        "__RETURN_CONSUMPTION_UNCHECKED_BASELINE_BEGIN__\n{}\n__RETURN_CONSUMPTION_UNCHECKED_BASELINE_END__",
        baseline.text
    );
    println!(
        "__RETURN_CONSUMPTION_UNCHECKED_TYPED_BEGIN__\n{}\n__RETURN_CONSUMPTION_UNCHECKED_TYPED_END__",
        report.text
    );
}

#[test]
fn conflicting_nonempty_generic_arguments_do_not_collapse_to_raw_conversion() {
    let mut fixture = Fixture::with_child_and_base(CONFLICTING_RETURN_BYTES, R5_BASE_BYTES, true);
    let analysis = fixture.analyze("conflict", "(Ljava/util/List;)Ljava/util/List;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let input = types
        .parameters
        .first()
        .expect("one real List<String> parameter")
        .1;
    let result = types.result.expect("the declared result is List<Foo>");
    let DeclaredGenericType::Reference {
        declaration: input_declaration,
        arguments: input_arguments,
    } = &types.types[input]
    else {
        panic!("input fact should be a bound reference type")
    };
    let DeclaredGenericType::Reference {
        declaration: result_declaration,
        arguments: result_arguments,
    } = &types.types[result]
    else {
        panic!("result fact should be a bound reference type")
    };
    assert_eq!(
        input_declaration, result_declaration,
        "both types use the selected List definition"
    );
    assert_eq!(input_arguments.len(), 1);
    assert_eq!(result_arguments.len(), 1);
    assert_ne!(
        input_arguments, result_arguments,
        "List<String> must not be erased into List<Foo>"
    );

    let baseline = fixture.recover_with_optional_types(ir, None);
    let typed = fixture.recover_with_types(ir, &types);
    assert_eq!(
        baseline.content,
        RecoveryContent::ContainsStatements,
        "{}",
        baseline.text
    );
    assert_eq!(
        typed.content,
        RecoveryContent::ExplanationOnly,
        "different nonempty source arguments do not authorize a raw unchecked conversion:\n{}",
        typed.text
    );
    println!(
        "__CONFLICTING_RETURN_BASELINE_BEGIN__\n{}\n__CONFLICTING_RETURN_BASELINE_END__",
        baseline.text
    );
    println!(
        "__CONFLICTING_RETURN_TYPED_REFUSAL_BEGIN__\n{}\n__CONFLICTING_RETURN_TYPED_REFUSAL_END__",
        typed.text
    );
}

#[test]
fn class_source_keeps_descriptor_report_and_retains_only_proved_declared_ast() {
    let mut positive_fixture =
        Fixture::with_child_and_base(RETURN_CONSUMPTION_BYTES, R5_BASE_BYTES, true);
    let positive_analysis =
        positive_fixture.analyze("returnParameter", "(Ljava/util/List;)Ljava/util/List;");
    let positive_ir = positive_analysis.ir();
    let positive_types = positive_fixture.value_types(positive_ir);

    let baseline =
        positive_fixture.recover_class_source(positive_ir, None, &mut Budget::new(limits()));
    let typed = positive_fixture.recover_class_source(
        positive_ir,
        Some(&positive_types),
        &mut Budget::new(limits()),
    );
    assert_eq!(
        typed.report.text, baseline.report.text,
        "declared types are held in the class-source sidecar and do not replace the public report"
    );
    assert!(
        baseline.candidates.declared_ast.is_none(),
        "missing declared facts produce no alternative AST"
    );
    let declared_ast = typed
        .candidates
        .declared_ast
        .as_ref()
        .expect("the proven typed return method retains its private declared AST");
    let declared_body = jarde_java::report::emit_class_source_method_ast(
        declared_ast,
        0,
        &mut Budget::new(limits()),
    )
    .expect("the retained declared AST emits through the existing class-source emitter");
    assert!(
        declared_body.contains("java.util.List<identity.Foo> local1 = arg0;"),
        "the private candidate carries typed declarations:\n{declared_body}"
    );
    assert!(declared_body.contains("return local1;"), "{declared_body}");

    let mut conflict_fixture =
        Fixture::with_child_and_base(CONFLICTING_RETURN_BYTES, R5_BASE_BYTES, true);
    let conflict_analysis =
        conflict_fixture.analyze("conflict", "(Ljava/util/List;)Ljava/util/List;");
    let conflict_ir = conflict_analysis.ir();
    let conflict_types = conflict_fixture.value_types(conflict_ir);
    let conflict_baseline =
        conflict_fixture.recover_class_source(conflict_ir, None, &mut Budget::new(limits()));
    let conflict_typed = conflict_fixture.recover_class_source(
        conflict_ir,
        Some(&conflict_types),
        &mut Budget::new(limits()),
    );
    assert_eq!(conflict_typed.report.text, conflict_baseline.report.text);
    assert!(
        conflict_typed.candidates.declared_ast.is_none(),
        "conflicting nonempty List<String> and List<Foo> arguments produce no declared candidate"
    );

    let mut limited = limits();
    limited.analysis_steps = 0;
    let budget_stopped = positive_fixture.recover_class_source(
        positive_ir,
        Some(&positive_types),
        &mut Budget::new(limited),
    );
    assert!(
        matches!(
            budget_stopped.report.outcome,
            jarde_java::RecoveryOutcome::Stopped(_)
        ),
        "budget stop is preserved"
    );
    assert_eq!(budget_stopped.report.content, RecoveryContent::NotProduced);
    assert!(budget_stopped.report.text.is_empty());
    assert_eq!(
        budget_stopped.candidates,
        jarde_java::report::ClassSourceCandidates::default(),
        "a stopped run publishes no partial candidate"
    );

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = positive_fixture.recover_class_source(
        positive_ir,
        Some(&positive_types),
        &mut Budget::with_cancellation_token(limits(), token),
    );
    assert!(
        matches!(
            cancelled.report.outcome,
            jarde_java::RecoveryOutcome::Stopped(_)
        ),
        "precancellation is preserved"
    );
    assert_eq!(cancelled.report.content, RecoveryContent::NotProduced);
    assert!(cancelled.report.text.is_empty());
    assert_eq!(
        cancelled.candidates,
        jarde_java::report::ClassSourceCandidates::default(),
        "cancellation publishes no partial candidate"
    );
}

#[test]
fn declared_field_consumption_respects_owner_erasure_and_static_members() {
    let mut fixture = Fixture::field_members();

    let analysis = fixture.analyze("typedWrite", "(Lflow/FieldBox;Ljava/util/List;)V");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let typed_write = fixture.recover_with_types(ir, &types);
    assert_eq!(
        typed_write.content,
        RecoveryContent::ContainsStatements,
        "{}",
        typed_write.text
    );
    assert!(
        typed_write.text.contains("arg0.value"),
        "{}",
        typed_write.text
    );
    assert!(typed_write.text.contains("arg1"), "{}", typed_write.text);
    println!(
        "__FIELD_TYPED_WRITE_BEGIN__\n{}\n__FIELD_TYPED_WRITE_END__",
        typed_write.text
    );

    let analysis = fixture.analyze("rawWrite", "(Lflow/FieldBox;Ljava/lang/Object;)V");
    let ir = analysis.ir();
    assert_eq!(method_local_stores(ir), vec![(1, 2)]);
    let types = fixture.value_types(ir);
    let raw_write = fixture.recover_with_types(ir, &types);
    assert_eq!(
        raw_write.content,
        RecoveryContent::ContainsStatements,
        "{}",
        raw_write.text
    );
    assert!(
        raw_write.text.contains("FieldBox local2 = arg0;"),
        "{}",
        raw_write.text
    );
    assert!(
        raw_write.text.contains("local2.value"),
        "{}",
        raw_write.text
    );
    assert!(raw_write.text.contains("= arg1;"), "{}", raw_write.text);
    assert!(
        !raw_write
            .text
            .contains("FieldBox<java.util.List<identity.Foo>> local2"),
        "raw FieldBox.value has erased Object assignment semantics:\n{}",
        raw_write.text
    );
    println!(
        "__FIELD_RAW_WRITE_BEGIN__\n{}\n__FIELD_RAW_WRITE_END__",
        raw_write.text
    );

    let analysis = fixture.analyze("staticRead", "()Ljava/util/List;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let static_read = fixture.recover_with_types(ir, &types);
    assert_eq!(
        static_read.content,
        RecoveryContent::ContainsStatements,
        "{}",
        static_read.text
    );
    assert!(
        static_read.text.contains("FieldBox.shared"),
        "{}",
        static_read.text
    );
    println!(
        "__FIELD_STATIC_READ_BEGIN__\n{}\n__FIELD_STATIC_READ_END__",
        static_read.text
    );

    let analysis = fixture.analyze("staticWrite", "(Ljava/util/List;)V");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let static_write = fixture.recover_with_types(ir, &types);
    assert_eq!(
        static_write.content,
        RecoveryContent::ContainsStatements,
        "{}",
        static_write.text
    );
    assert!(
        static_write.text.contains("FieldBox.shared"),
        "{}",
        static_write.text
    );
    assert!(static_write.text.contains("arg0"), "{}", static_write.text);
    println!(
        "__FIELD_STATIC_WRITE_BEGIN__\n{}\n__FIELD_STATIC_WRITE_END__",
        static_write.text
    );

    let analysis = fixture.analyze("primitiveAndLiteral", "(Lflow/FieldBox;)V");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let primitive = fixture.recover_with_types(ir, &types);
    assert_eq!(
        primitive.content,
        RecoveryContent::ContainsStatements,
        "{}",
        primitive.text
    );
    assert!(primitive.text.contains(".count"), "{}", primitive.text);
    assert!(primitive.text.contains("= 1;"), "{}", primitive.text);
    assert!(primitive.text.contains(".title"), "{}", primitive.text);
    assert!(primitive.text.contains("= \"x\";"), "{}", primitive.text);
    println!(
        "__FIELD_PRIMITIVE_LITERAL_BEGIN__\n{}\n__FIELD_PRIMITIVE_LITERAL_END__",
        primitive.text
    );

    let analysis = fixture.analyze("getBaseField", "(Lflow/FieldDerived;)Ljava/util/List;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let source_map_report = fixture.recover_with_types_and_source_map(ir, &types);
    assert_eq!(
        source_map_report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        source_map_report.text
    );
    let casts = checkcast_targets(ir);
    let list_casts = casts
        .iter()
        .filter(|(_, target)| *target == b"java/util/List")
        .collect::<Vec<_>>();
    assert_eq!(
        list_casts.len(),
        1,
        "the inherited generic field has one original c0List: {casts:?}"
    );
    assert!(
        source_map_report
            .text
            .matches("(java.util.List<identity.Foo>)")
            .count()
            == 1,
        "the inherited field conversion appears once, without reevaluating the qualifier:\n{}",
        source_map_report.text
    );
    assert!(
        source_map_report
            .text
            .contains("((flow.FieldBox<java.util.List<identity.Foo>>) arg0).value"),
        "the emitted owner qualifier selects the resolved parent field, not FieldDerived.value:Object:\n{}",
        source_map_report.text
    );
    assert_eq!(
        source_map_report.text.matches("arg0").count(),
        1,
        "{}",
        source_map_report.text
    );
    assert!(
        !source_map_report
            .source_map
            .of_bci(list_casts[0].0)
            .is_empty(),
        "the source map binds the original c0List BCI {}",
        list_casts[0].0
    );
    println!(
        "__FIELD_INHERITED_READ_BEGIN__\n{}\n__FIELD_INHERITED_READ_END__",
        source_map_report.text
    );
}

#[test]
fn class_source_field_consumption_requires_the_exact_typed_ast_operation() {
    let mut fixture = Fixture::field_members();

    let analysis = fixture.analyze("typedWrite", "(Lflow/FieldBox;Ljava/util/List;)V");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let bci = field_member_bci(&types, "value");
    let recovered = fixture.recover_class_source(ir, Some(&types), &mut Budget::new(limits()));
    let ast = recovered
        .candidates
        .declared_ast
        .expect("typed class-source AST is retained");
    assert!(
        jarde_java::report::class_source_declared_field_consumed(
            &ast,
            bci,
            true,
            &mut Budget::new(limits())
        )
        .expect("field-consumption check completes")
    );
    assert!(
        !jarde_java::report::class_source_declared_field_consumed(
            &ast,
            bci,
            false,
            &mut Budget::new(limits())
        )
        .expect("wrong-direction check completes")
    );
    assert!(
        !jarde_java::report::class_source_declared_field_consumed(
            &ast,
            bci + 1,
            true,
            &mut Budget::new(limits())
        )
        .expect("wrong-BCI check completes")
    );

    let analysis = fixture.analyze("staticWrite", "(Ljava/util/List;)V");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let bci = field_member_bci(&types, "shared");
    let recovered = fixture.recover_class_source(ir, Some(&types), &mut Budget::new(limits()));
    let ast = recovered
        .candidates
        .declared_ast
        .expect("typed static-write AST is retained");
    assert!(
        jarde_java::report::class_source_declared_field_consumed(
            &ast,
            bci,
            true,
            &mut Budget::new(limits())
        )
        .expect("static write check completes")
    );

    let analysis = fixture.analyze("staticRead", "()Ljava/util/List;");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let bci = field_member_bci(&types, "shared");
    let recovered = fixture.recover_class_source(ir, Some(&types), &mut Budget::new(limits()));
    let ast = recovered
        .candidates
        .declared_ast
        .expect("typed static-read AST is retained");
    assert!(
        jarde_java::report::class_source_declared_field_consumed(
            &ast,
            bci,
            false,
            &mut Budget::new(limits())
        )
        .expect("static read check completes")
    );
    assert!(
        !jarde_java::report::class_source_declared_field_consumed(
            &ast,
            bci,
            true,
            &mut Budget::new(limits())
        )
        .expect("wrong-direction check completes")
    );

    let analysis = fixture.analyze("rawWrite", "(Lflow/FieldBox;Ljava/lang/Object;)V");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let bci = field_member_bci(&types, "value");
    let recovered = fixture.recover_class_source(ir, Some(&types), &mut Budget::new(limits()));
    let ast = recovered
        .candidates
        .declared_ast
        .expect("raw class-source AST is retained");
    assert!(
        !jarde_java::report::class_source_declared_field_consumed(
            &ast,
            bci,
            true,
            &mut Budget::new(limits())
        )
        .expect("raw write check completes without a typed field marker")
    );

    let mut no_analysis = limits();
    no_analysis.analysis_steps = 0;
    assert!(
        jarde_java::report::class_source_declared_field_consumed(
            &ast,
            bci,
            true,
            &mut Budget::new(no_analysis)
        )
        .is_err(),
        "budget exhaustion propagates as a stop"
    );
    let token = CancellationToken::new();
    token.cancel();
    assert!(
        jarde_java::report::class_source_declared_field_consumed(
            &ast,
            bci,
            true,
            &mut Budget::with_cancellation_token(limits(), token)
        )
        .is_err(),
        "precancellation propagates as a stop"
    );
}

#[test]
fn owned_field_cycle_field_reads_and_writes_are_bound_to_physical_bcis() {
    let mut fixture = Fixture::with_child_and_base(OWNED_FIELD_CYCLE_BYTES, R5_BASE_BYTES, true);
    for (name, descriptor, write) in [
        ("put", "(Lflow/Base;)V", true),
        ("get", "()Ljava/util/List;", false),
    ] {
        let analysis = fixture.analyze(name, descriptor);
        let ir = analysis.ir();
        let types = fixture.value_types(ir);
        let bci = field_member_bci(&types, "values");
        let recovered = fixture.recover_class_source(ir, Some(&types), &mut Budget::new(limits()));
        assert_eq!(
            recovered.report.content,
            RecoveryContent::ContainsStatements,
            "{name}: {}",
            recovered.report.text
        );
        let ast = recovered
            .candidates
            .declared_ast
            .expect("the typed candidate is retained with its complete AST");
        assert!(
            jarde_java::report::class_source_declared_field_consumed(
                &ast,
                bci,
                write,
                &mut Budget::new(limits())
            )
            .expect("the exact physical field-consumption check completes"),
            "{name} BCI {bci} is consumed in the correct direction"
        );
        assert!(
            !jarde_java::report::class_source_declared_field_consumed(
                &ast,
                bci,
                !write,
                &mut Budget::new(limits())
            )
            .expect("wrong-direction check completes")
        );
        assert!(
            !jarde_java::report::class_source_declared_field_consumed(
                &ast,
                bci + 1,
                write,
                &mut Budget::new(limits())
            )
            .expect("wrong-BCI check completes")
        );
    }
}

#[test]
fn constructor_witness_retention_obeys_whole_recovery_ir_budget() {
    let mut fixture = Fixture::new();
    let analysis = fixture.analyze("<init>", "()V");
    let ir = analysis.ir();
    let types = fixture.value_types(ir);
    let declaration = ir.declaration().unwrap();
    let owner = declaration.class_name().0.as_slice();
    let invokes: Vec<_> = ir
        .code()
        .unwrap()
        .instructions
        .iter()
        .filter(|instruction| instruction.opcode == 0xb7)
        .collect();
    assert_eq!(
        invokes.len(),
        1,
        "the actual constructor has one invokespecial"
    );
    let super_bci = invokes[0].bci;

    let check_witnesses = |recovered: &jarde_java::report::ClassSourceRecovery| {
        assert!(recovered.report.produced(), "{}", recovered.report.text);
        for ast in [
            recovered
                .candidates
                .ast
                .as_ref()
                .expect("physical AST retained"),
            recovered
                .candidates
                .declared_ast
                .as_ref()
                .expect("declared AST retained"),
        ] {
            let consumers = jarde_java::report::class_source_method_body_consumers(
                ast,
                &mut Budget::new(limits()),
            )
            .expect("consumer inspection completes")
            .expect("simple constructor has complete body consumers");
            let init = consumers.init_record.expect("constructor witness retained");
            assert!(init.presented);
            assert!(init.declares_raw_class(owner));
            assert_eq!(init.bci, Some(super_bci));
            assert_eq!(init.target, Some(jarde_java::ast::ConstructorTarget::Super));
            assert_eq!(consumers.constructor_calls.len(), 1);
            assert_eq!(consumers.constructor_calls[0].bci, super_bci);
        }
    };
    for (late_evidence, evidence) in [
        (false, jarde_java::RecoveryEvidenceRequest::essential()),
        (true, jarde_java::RecoveryEvidenceRequest::all()),
    ] {
        let mut completed_budget = Budget::new(limits());
        let completed = fixture.recover_class_source_with_evidence(
            ir,
            Some(&types),
            evidence.clone(),
            &mut completed_budget,
        );
        check_witnesses(&completed);
        let used = completed_budget.usage().ir_items;
        assert!(used > 0);

        // This is the whole recovery boundary, not the individual witness allocation frontier.
        let mut exact_limits = limits();
        exact_limits.ir_items = used;
        let exact = fixture.recover_class_source_with_evidence(
            ir,
            Some(&types),
            evidence.clone(),
            &mut Budget::new(exact_limits),
        );
        check_witnesses(&exact);
        assert_eq!(exact.report.text, completed.report.text);

        let mut short_limits = limits();
        short_limits.ir_items = used - 1;
        let stopped = fixture.recover_class_source_with_evidence(
            ir,
            Some(&types),
            evidence.clone(),
            &mut Budget::new(short_limits),
        );
        if late_evidence {
            // Optional evidence is materialized after text commit. Its resource stop keeps
            // that text but cannot publish reusable physical/declared AST candidates.
            assert!(matches!(
                stopped.report.outcome,
                jarde_java::RecoveryOutcome::Produced
            ));
            assert_eq!(stopped.report.text, completed.report.text);
            assert!(matches!(
                stopped.report.execution,
                jarde_reader::model::ExecutionReport::Partial {
                    reason: jarde_reader::model::TerminationReason::BudgetExceeded {
                        dimension: jarde_reader::budget::BudgetDimension::IrItems,
                    },
                    ..
                }
            ));
        } else {
            assert!(matches!(
                stopped.report.outcome,
                jarde_java::RecoveryOutcome::Stopped(jarde_java::StopReason::Budget {
                    dimension: jarde_reader::budget::BudgetDimension::IrItems,
                    ..
                })
            ));
            assert!(stopped.report.text.is_empty());
        }
        assert!(stopped.candidates.ast.is_none());
        assert!(stopped.candidates.declared_ast.is_none());

        let token = CancellationToken::new();
        token.cancel();
        let cancelled = fixture.recover_class_source_with_evidence(
            ir,
            Some(&types),
            evidence,
            &mut Budget::with_cancellation_token(limits(), token),
        );
        assert!(matches!(
            cancelled.report.outcome,
            jarde_java::RecoveryOutcome::Stopped(jarde_java::StopReason::Cancelled { .. })
        ));
        assert!(cancelled.candidates.ast.is_none());
        assert!(cancelled.candidates.declared_ast.is_none());
    }
}
