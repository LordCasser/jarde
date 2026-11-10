//! JVM Signature scope boundary: class variables are not visible from static methods.
//!
//! The negative class is a caller-supplied classfile mutation, not legal javac source. Its Code,
//! descriptor and class Signature are unchanged; only `pass` receives Signature `(TT;)TT;`.

use jarde_java::generic_types::{
    declared_method_declaration, declared_value_types, spell_declared_type,
};
use jarde_java::report::DeclaredGenericType;
use jarde_java::source_proof::{DependencyResult, RecoveryHeaders, ReferenceClassHeader};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_jvm::method_ir::Slot;
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::{class_constant_pool, class_member_facts};
use jarde_reader::error::Error;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};
use std::collections::BTreeMap;

const MUTANT: &[u8] = include_bytes!(
    "fixtures/s2b-static-class-scope-boundary-r1/classes/scope/GenericScopeBoundary-invalid-static-signature.class"
);
const SHADOW: &[u8] = include_bytes!(
    "fixtures/s2b-static-class-scope-boundary-r1/classes/scope/GenericScopeShadow.class"
);
const OBJECT: &[u8] =
    include_bytes!("fixtures/s2b-declared-local-consumer-r5/platform/java/lang/Object.class");

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 22,
        archive_entries: 32,
        entry_bytes: 1 << 22,
        read_bytes: 1 << 22,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 18,
        output_bytes: 1 << 20,
        class_headers: 32,
        method_bodies: 8,
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
    .expect("frozen class bytes open")
}

fn definition(bytes: &[u8], snapshot: &ArtifactSnapshot) -> PhysicalDefinitionId {
    PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: bytes.len() as u64,
        },
        variant: PhysicalVariant::Base,
    }
}

fn selected_header(bytes: &[u8], loader: &str) -> ReferenceClassHeader {
    let mut budget = Budget::new(limits());
    let facts = class_member_facts(bytes, &mut budget).expect("real class header parses");
    let pool = class_constant_pool(bytes, &budget).expect("real constant pool parses");
    let snapshot = snapshot(bytes);
    ReferenceClassHeader::from_facts(
        &facts,
        LoaderId(loader.to_owned()),
        definition(bytes, &snapshot),
    )
    .expect("header is complete")
    .with_generic_signatures(bytes, &facts.attributes, &pool, &mut budget)
    .expect("Signatures match physical erasures, including the deliberate static-scope mutant")
}

#[derive(Default)]
struct Headers(BTreeMap<(String, Vec<u8>), ReferenceClassHeader>);

impl RecoveryHeaders for Headers {
    type Error = Error;

    fn caller_loader(&self) -> &LoaderId {
        static APP: std::sync::OnceLock<LoaderId> = std::sync::OnceLock::new();
        APP.get_or_init(|| LoaderId("app".to_owned()))
    }

    fn selected_from(
        &mut self,
        initiating_loader: &LoaderId,
        owner: &[u8],
        budget: &mut Budget,
    ) -> std::result::Result<DependencyResult<ReferenceClassHeader>, Self::Error> {
        budget.poll()?;
        Ok(self
            .0
            .get(&(initiating_loader.0.clone(), owner.to_vec()))
            .cloned()
            .map_or(DependencyResult::Missing, DependencyResult::Found))
    }

    fn platform_unprovided(
        &mut self,
        _owner: &[u8],
        budget: &mut Budget,
    ) -> std::result::Result<bool, Self::Error> {
        budget.poll()?;
        Ok(false)
    }
}

fn header_set(class: &[u8]) -> Headers {
    let mut headers = Headers::default();
    let facts =
        class_member_facts(class, &mut Budget::new(limits())).expect("physical class facts");
    headers.0.insert(
        ("app".to_owned(), facts.this_class.raw().0.clone()),
        selected_header(class, "app"),
    );
    headers.0.insert(
        ("app".to_owned(), b"java/lang/Object".to_vec()),
        selected_header(OBJECT, "boot"),
    );
    headers.0.insert(
        ("boot".to_owned(), b"java/lang/Object".to_vec()),
        selected_header(OBJECT, "boot"),
    );
    headers
}

fn analyze(
    class: &[u8],
    name: &[u8],
) -> (ArtifactSnapshot, jarde_jvm::method_ir::MethodIrAnalysis) {
    let app = snapshot(class);
    let boot = snapshot(OBJECT);
    let app_loader = LoaderId("app".to_owned());
    let boot_loader = LoaderId("boot".to_owned());
    let app_domain = LoadDomain {
        loader: app_loader.clone(),
        parent_loader: Some(boot_loader.clone()),
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: app.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let boot_domain = LoadDomain {
        loader: boot_loader,
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: boot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let environment = ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: app.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: app_domain.clone(),
        },
        domains: vec![app_domain, boot_domain],
        providers: Vec::new(),
    };
    let method = PhysicalMethodId {
        owner: definition(class, &app),
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(b"(Ljava/lang/Object;)Ljava/lang/Object;".to_vec()),
    };
    let analysis = analyze_method_ir(
        &[app.clone(), boot],
        &MethodAnalysisRequest {
            environment,
            method,
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut Budget::new(limits()),
    )
    .expect("actual javac Code and descriptor analyze");
    (app, analysis)
}

fn source_declaration(
    class: &[u8],
    method_name: &[u8],
) -> (DeclaredGenericType, Option<String>, Option<String>) {
    let (_snapshot, analysis) = analyze(class, method_name);
    let ir = analysis.ir();
    let mut headers = header_set(class);
    let types = match declared_value_types(ir, &mut headers, &mut Budget::new(limits()))
        .expect("real MethodIr's Signature can be bound")
    {
        DependencyResult::Found(types) => types,
        DependencyResult::Missing => panic!("required class or method Signature header is missing"),
        DependencyResult::Ambiguous => panic!("selected Signature header is ambiguous"),
        DependencyResult::Unknown => panic!("method declaration or SSA type facts are unknown"),
    };
    let ssa = ir.ssa().expect("Code has SSA");
    let local_value = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .flat_map(|instruction| instruction.writes())
        .find_map(|(slot, value)| matches!(slot, Slot::Local(1)).then_some(*value))
        .expect("javac body stores the copied value in physical local slot 1");
    let value_index = ssa
        .values_with_ids()
        .position(|(candidate, _)| candidate == local_value)
        .expect("local write has an SSA value index");
    let local_type_index = *types.values[value_index]
        .as_ref()
        .and_then(|bounds| bounds.first())
        .expect("the parameter copy retains its source type bound");
    let local_type = types.types[local_type_index].clone();
    let local_spelling = spell_declared_type(&types, local_type_index, &mut Budget::new(limits()))
        .expect("spelling is a bounded operation");
    let member = ir.declaration().expect("physical method selected");
    let facts =
        class_member_facts(class, &mut Budget::new(limits())).expect("physical member facts");
    let physical = facts
        .methods
        .iter()
        .find(|method| method.name.raw().0 == method_name)
        .expect("the selected physical method remains in its original table");
    let declaration = declared_method_declaration(
        &types,
        &[(0, "value".to_owned())],
        method_name,
        physical.access_flags,
        &[],
        &mut Budget::new(limits()),
    )
    .expect("declaration admission is bounded");
    assert_eq!(
        types.declaration.as_ref().unwrap().definition,
        member.identity().owner
    );
    (local_type, local_spelling, declaration)
}

#[test]
fn static_class_variable_is_not_published_but_method_shadow_is() {
    let (local_type, local_spelling, declaration) = source_declaration(MUTANT, b"pass");
    assert!(matches!(
        local_type,
        DeclaredGenericType::Variable { method: None, .. }
    ));
    assert_eq!(
        local_spelling, None,
        "a static method cannot spell its enclosing class T as a local type"
    );
    assert_eq!(
        declaration, None,
        "invalid static Signature must not become a Java method declaration"
    );

    let (shadow_type, shadow_local, shadow_declaration) = source_declaration(SHADOW, b"pass");
    assert!(matches!(
        shadow_type,
        DeclaredGenericType::Variable {
            method: Some(_),
            ..
        }
    ));
    assert_eq!(
        shadow_local.as_deref(),
        Some("T"),
        "the method's own T shadows the class T legally"
    );
    let shadow_declaration = shadow_declaration.expect("method-scoped T is source-visible");
    assert!(shadow_declaration.contains("<T"), "{shadow_declaration}");
    assert!(
        shadow_declaration.contains(" pass(T value)"),
        "{shadow_declaration}"
    );
}
