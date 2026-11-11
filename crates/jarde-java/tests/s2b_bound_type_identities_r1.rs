//! Bound declared-type identity and state controls over real classfile headers.
//! These assertions exercise the low-level type arena only; they authorize no source projection.

use jarde_java::generic_types::{
    bind_declared_field_type, bind_declared_parameters, bind_declared_type,
    declared_class_declaration, declared_class_types, declared_member_types,
    declared_method_declaration, declared_supertype, prove_declared_relations,
};
use jarde_java::report::{DeclaredGenericMemberTypes, DeclaredGenericType, DeclaredGenericTypes};
use jarde_java::source_proof::{DependencyResult, RecoveryHeaders, ReferenceClassHeader};
use jarde_jvm::resolver::ResolvedMemberRef;
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::classfile::{class_constant_pool, class_member_facts};
use jarde_reader::error::Error;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, JvmString, PhysicalClassLocation, PhysicalDefinitionId,
    PhysicalVariant, SymbolRef,
};
use jarde_reader::signature::{
    SignatureType, parse_class_signature, parse_field_signature, prove_class_signature_erasure,
    prove_method_signature_erasure_with_class_scope,
};
use jarde_reader::view::LoaderId;
use std::collections::BTreeMap;

#[path = "s2b_bound_type_identities_r1/declared_bounds.rs"]
mod declared_bounds_r1;

const GENERIC_CHILD: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/generic/classes/generic/Child.class");
const GENERIC_MID: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/generic/classes/generic/Mid.class");
const GENERIC_BASE: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/generic/classes/generic/Base.class");
const CHILD_FOO: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/identity-a/classes/identity/Foo.class");
const PARENT_FOO: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/identity-b/classes/identity/Foo.class");
const LIST: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/platform/java/util/List.class");
const COLLECTION: &[u8] =
    include_bytes!("fixtures/s2b-declared-local-consumer-r5/platform/java/util/Collection.class");
const ITERABLE: &[u8] =
    include_bytes!("fixtures/s2b-declared-local-consumer-r5/platform/java/lang/Iterable.class");
const OBJECT: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/platform/java/lang/Object.class");
const STRING: &[u8] =
    include_bytes!("fixtures/s2b-bound-type-identities-r1/platform/java/lang/String.class");

const CHILD_LOADER: &str = "child-loader";
const PARENT_LOADER: &str = "parent-loader";
const BOOT_LOADER: &str = "bootstrap-loader";

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 64,
        method_bodies: 0,
        ir_items: 1 << 16,
        ir_edges: 1 << 16,
        analysis_steps: 1 << 16,
        normalization_clones: 0,
        nested_depth: 32,
        dependency_depth: 32,
        elapsed_millis: u64::MAX,
    }
}

fn class_header(bytes: &[u8], loader: &str) -> ReferenceClassHeader {
    let mut budget = Budget::new(limits());
    let facts = class_member_facts(bytes, &mut budget).expect("fixture class member facts decode");
    let pool = class_constant_pool(bytes, &budget).expect("fixture constant pool decodes");
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("class bytes form a standalone physical snapshot");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: u64::try_from(bytes.len()).expect("fixture length fits"),
        },
        variant: PhysicalVariant::Base,
    };
    ReferenceClassHeader::from_facts(&facts, LoaderId(loader.to_owned()), definition)
        .expect("complete physical class header")
        .with_generic_signatures(bytes, &facts.attributes, &pool, &mut budget)
        .expect("fixture generic signatures parse and match physical erasures")
}

#[derive(Default)]
struct Headers {
    selected: BTreeMap<(String, Vec<u8>), ReferenceClassHeader>,
    lookups: Vec<(String, Vec<u8>, Option<(LoaderId, PhysicalDefinitionId)>)>,
}

impl Headers {
    fn insert(&mut self, initiating: &str, name: &[u8], header: ReferenceClassHeader) {
        self.selected
            .insert((initiating.to_owned(), name.to_vec()), header);
    }

    fn full() -> Self {
        let child = class_header(GENERIC_CHILD, CHILD_LOADER);
        let mid = class_header(GENERIC_MID, PARENT_LOADER);
        let base = class_header(GENERIC_BASE, PARENT_LOADER);
        let child_foo = class_header(CHILD_FOO, CHILD_LOADER);
        let parent_foo = class_header(PARENT_FOO, PARENT_LOADER);
        let list = class_header(LIST, BOOT_LOADER);
        let object = class_header(OBJECT, BOOT_LOADER);
        let string = class_header(STRING, BOOT_LOADER);
        let mut headers = Self::default();
        headers.insert(CHILD_LOADER, b"generic/Child", child);
        headers.insert(CHILD_LOADER, b"generic/Mid", mid.clone());
        headers.insert(PARENT_LOADER, b"generic/Mid", mid);
        headers.insert(PARENT_LOADER, b"generic/Base", base);
        headers.insert(CHILD_LOADER, b"identity/Foo", child_foo);
        headers.insert(PARENT_LOADER, b"identity/Foo", parent_foo);
        headers.insert(PARENT_LOADER, b"java/util/List", list.clone());
        headers.insert(BOOT_LOADER, b"java/util/List", list);
        headers.insert(PARENT_LOADER, b"java/lang/Object", object.clone());
        headers.insert(BOOT_LOADER, b"java/lang/Object", object);
        headers.insert(BOOT_LOADER, b"java/lang/String", string);
        let child_object = headers.get(PARENT_LOADER, b"java/lang/Object").clone();
        let child_string = headers.get(BOOT_LOADER, b"java/lang/String").clone();
        headers.insert(CHILD_LOADER, b"java/lang/Object", child_object);
        headers.insert(CHILD_LOADER, b"java/lang/String", child_string);
        headers
    }

    fn get(&self, loader: &str, name: &[u8]) -> &ReferenceClassHeader {
        self.selected
            .get(&(loader.to_owned(), name.to_vec()))
            .expect("fixture selected header exists")
    }

    fn remove(&mut self, loader: &str, name: &[u8]) {
        self.selected.remove(&(loader.to_owned(), name.to_vec()));
    }
}

impl RecoveryHeaders for Headers {
    type Error = Error;

    fn caller_loader(&self) -> &LoaderId {
        static LOADER: std::sync::OnceLock<LoaderId> = std::sync::OnceLock::new();
        LOADER.get_or_init(|| LoaderId(CHILD_LOADER.to_owned()))
    }

    fn selected_from(
        &mut self,
        initiating_loader: &LoaderId,
        owner: &[u8],
        budget: &mut Budget,
    ) -> Result<DependencyResult<ReferenceClassHeader>, Self::Error> {
        budget.poll()?;
        let selected = self
            .selected
            .get(&(initiating_loader.0.clone(), owner.to_vec()))
            .cloned();
        self.lookups.push((
            initiating_loader.0.clone(),
            owner.to_vec(),
            selected
                .as_ref()
                .map(|header| (header.loader.clone(), header.definition.clone())),
        ));
        Ok(selected.map_or(DependencyResult::Missing, DependencyResult::Found))
    }

    fn selected_generic_from(
        &mut self,
        initiating_loader: &LoaderId,
        owner: &[u8],
        budget: &mut Budget,
    ) -> Result<DependencyResult<ReferenceClassHeader>, Self::Error> {
        self.selected_from(initiating_loader, owner, budget)
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

fn field_type(signature: &[u8]) -> SignatureType {
    let mut budget = Budget::new(limits());
    parse_field_signature(signature, &mut budget)
        .expect("test Signature parses")
        .ty
}

fn reference_target(header: &ReferenceClassHeader, owner: &[u8]) -> ResolvedMemberRef {
    ResolvedMemberRef {
        loader: header.loader.clone(),
        definition: header.definition.clone(),
        member: SymbolRef::Class {
            owner: JvmBytes(owner.to_vec()),
        },
    }
}

#[test]
fn declared_class_header_binds_selected_supertype_without_method_ir() {
    let mut headers = Headers::full();
    let child = headers.get(CHILD_LOADER, b"generic/Child").clone();
    let actual = reference_target(&child, b"generic/Child");
    let mut budget = Budget::new(limits());
    let (types, superclass, interfaces) =
        declared_class_types(&actual, &child, &mut headers, &mut budget)
            .unwrap()
            .found()
            .expect("selected class Signature binds from physical headers only");

    assert_eq!(types.declaration.as_ref(), Some(&actual));
    assert!(types.class_parameters.is_empty());
    assert!(interfaces.is_empty());
    let DeclaredGenericType::Reference {
        declaration: mid,
        arguments,
    } = &types.types[superclass]
    else {
        panic!("Child's real Signature names its parameterized Mid superclass")
    };
    assert_eq!(mid.loader.0, PARENT_LOADER);
    assert!(matches!(&mid.member, SymbolRef::Class { owner } if owner.0 == b"generic/Mid"));
    assert_eq!(arguments.len(), 1);
    assert!(matches!(
        &types.types[arguments[0]],
        DeclaredGenericType::Reference { declaration: foo, arguments }
            if foo.loader.0 == CHILD_LOADER
                && arguments.is_empty()
                && matches!(&foo.member, SymbolRef::Class { owner } if owner.0 == b"identity/Foo")
    ));
    // Fixture limits set method_bodies=0: this result is backed by class/member facts only.
}

#[test]
fn declared_class_header_propagates_missing_parent_and_rejects_wrong_identity() {
    let mut headers = Headers::full();
    let child = headers.get(CHILD_LOADER, b"generic/Child").clone();
    let actual = reference_target(&child, b"generic/Child");
    let mut budget = Budget::new(limits());
    let mut wrong = actual.clone();
    wrong.definition = headers
        .get(PARENT_LOADER, b"generic/Base")
        .definition
        .clone();
    assert!(matches!(
        declared_class_types(&wrong, &child, &mut headers, &mut budget).unwrap(),
        DependencyResult::Unknown
    ));

    headers.remove(CHILD_LOADER, b"generic/Mid");
    assert!(matches!(
        declared_class_types(&actual, &child, &mut headers, &mut budget).unwrap(),
        DependencyResult::Missing
    ));
}

#[test]
fn declared_class_header_spells_only_the_bound_physical_parent_nodes() {
    let mut headers = Headers::full();
    let child = headers.get(CHILD_LOADER, b"generic/Child").clone();
    let actual = reference_target(&child, b"generic/Child");
    let mut budget = Budget::new(limits());
    let (types, superclass, interfaces) =
        declared_class_types(&actual, &child, &mut headers, &mut budget)
            .unwrap()
            .found()
            .expect("Child class Signature binds from metadata only");
    let physical_super = JvmString::from_mutf8(child.super_name.clone().unwrap()).unwrap();
    let physical_interfaces: Vec<_> = child
        .interfaces
        .iter()
        .cloned()
        .map(|name| JvmString::from_mutf8(name).unwrap())
        .collect();
    let (declaration, parameters) = declared_class_declaration(
        &types,
        superclass,
        &interfaces,
        "Child",
        &JvmString::from_mutf8(b"generic/Child".to_vec()).unwrap(),
        child.flags,
        &physical_super,
        &physical_interfaces,
        &mut budget,
    )
    .unwrap()
    .expect("Child header candidate uses the original physical superclass");
    assert_eq!(parameters, None);
    assert!(declaration.contains("extends generic.Mid<identity.Foo>"));

    let mid = headers.get(PARENT_LOADER, b"generic/Mid").clone();
    // The inherited `List<X>` argument is checked against Base<T>'s Object bound. Its
    // real platform path goes through Collection and Iterable, so provide those physical
    // Headers locally rather than broadening every existing control's dependency closure.
    headers.insert(
        BOOT_LOADER,
        b"java/util/Collection",
        class_header(COLLECTION, BOOT_LOADER),
    );
    headers.insert(
        BOOT_LOADER,
        b"java/lang/Iterable",
        class_header(ITERABLE, BOOT_LOADER),
    );
    let mid_actual = reference_target(&mid, b"generic/Mid");
    let (mid_types, mid_superclass, mid_interfaces) =
        declared_class_types(&mid_actual, &mid, &mut headers, &mut budget)
            .unwrap()
            .found()
            .expect("Mid class Signature binds from metadata only");
    let mid_physical_super = JvmString::from_mutf8(mid.super_name.clone().unwrap()).unwrap();
    let mid_physical_interfaces: Vec<_> = mid
        .interfaces
        .iter()
        .cloned()
        .map(|name| JvmString::from_mutf8(name).unwrap())
        .collect();
    let (declaration, parameters) = declared_class_declaration(
        &mid_types,
        mid_superclass,
        &mid_interfaces,
        "Mid",
        &JvmString::from_mutf8(b"generic/Mid".to_vec()).unwrap(),
        mid.flags,
        &mid_physical_super,
        &mid_physical_interfaces,
        &mut budget,
    )
    .unwrap()
    .expect("Mid header candidate uses the original physical superclass");
    assert_eq!(parameters.as_deref(), Some("X extends java.lang.Object"));
    assert!(declaration.contains("extends generic.Base<java.util.List<X>>"));

    let wrong_super = JvmString::from_mutf8(b"generic/Base".to_vec()).unwrap();
    assert!(
        declared_class_declaration(
            &types,
            superclass,
            &interfaces,
            "Child",
            &JvmString::from_mutf8(b"generic/Child".to_vec()).unwrap(),
            child.flags,
            &wrong_super,
            &physical_interfaces,
            &mut budget,
        )
        .unwrap()
        .is_none()
    );

    let mut wrong_owner = types.clone();
    let Some(ResolvedMemberRef {
        member: SymbolRef::Class { owner },
        ..
    }) = wrong_owner.declaration.as_mut()
    else {
        panic!("class binding retains its physical class declaration")
    };
    *owner = JvmBytes(b"generic/Other".to_vec());
    assert!(
        declared_class_declaration(
            &wrong_owner,
            superclass,
            &interfaces,
            "Child",
            &JvmString::from_mutf8(b"generic/Child".to_vec()).unwrap(),
            child.flags,
            &physical_super,
            &physical_interfaces,
            &mut budget,
        )
        .unwrap()
        .is_none()
    );

    let mut inadmissible_parent = types;
    inadmissible_parent
        .admissible
        .retain(|node| *node != superclass);
    assert!(
        declared_class_declaration(
            &inadmissible_parent,
            superclass,
            &interfaces,
            "Child",
            &JvmString::from_mutf8(b"generic/Child".to_vec()).unwrap(),
            child.flags,
            &physical_super,
            &physical_interfaces,
            &mut budget,
        )
        .unwrap()
        .is_none()
    );
    // Fixture limits set method_bodies=0; all candidate text came from class metadata.
}

fn field_target(
    header: &ReferenceClassHeader,
    owner: &[u8],
    name: &[u8],
    descriptor: &[u8],
) -> ResolvedMemberRef {
    assert!(
        header.fields.iter().any(|member| {
            member.name.raw().0 == name && member.descriptor.raw().0 == descriptor
        })
    );
    ResolvedMemberRef {
        loader: header.loader.clone(),
        definition: header.definition.clone(),
        member: SymbolRef::Field {
            owner: JvmBytes(owner.to_vec()),
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
    }
}

fn method_target(
    header: &ReferenceClassHeader,
    owner: &[u8],
    name: &[u8],
    descriptor: &[u8],
) -> ResolvedMemberRef {
    assert!(
        header.methods.iter().any(|member| {
            member.name.raw().0 == name && member.descriptor.raw().0 == descriptor
        })
    );
    ResolvedMemberRef {
        loader: header.loader.clone(),
        definition: header.definition.clone(),
        member: SymbolRef::Method {
            owner: JvmBytes(owner.to_vec()),
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
    }
}

fn list_to_array_declaration_candidate(
    headers: &mut Headers,
    budget: &mut Budget,
) -> (DeclaredGenericTypes, u16) {
    let list = headers.get(BOOT_LOADER, b"java/util/List").clone();
    let signatures = list.generic_signatures.as_ref().unwrap();
    let (class_signature, class_proof) = signatures.class.as_ref().unwrap();
    let mut types = DeclaredGenericTypes::default();
    let class_scope = bind_declared_parameters(
        &mut types,
        &list,
        None,
        &class_signature.type_parameters,
        &class_proof.type_parameters,
        &[],
        headers,
        budget,
    )
    .unwrap()
    .found()
    .unwrap();
    types.class_parameters = class_scope.iter().map(|(_, node)| *node).collect();

    let (index, (signature, proof)) = signatures
        .methods
        .iter()
        .enumerate()
        .find_map(|(index, method)| {
            method
                .as_ref()
                .filter(|(signature, _)| {
                    list.methods[index].name.raw().0 == b"toArray"
                        && list.methods[index].descriptor.raw().0
                            == b"([Ljava/lang/Object;)[Ljava/lang/Object;"
                        && !signature.type_parameters.is_empty()
                })
                .map(|pair| (index, pair))
        })
        .expect("List.toArray(T[]) has the pinned method Signature");
    let identity = method_target(
        &list,
        b"java/util/List",
        b"toArray",
        b"([Ljava/lang/Object;)[Ljava/lang/Object;",
    );
    types.declaration = Some(identity);
    types.method_signature = Some(signature.clone());
    let variables = bind_declared_parameters(
        &mut types,
        &list,
        Some((b"toArray", b"([Ljava/lang/Object;)[Ljava/lang/Object;")),
        &signature.type_parameters,
        &proof.type_parameters,
        &class_scope,
        headers,
        budget,
    )
    .unwrap()
    .found()
    .unwrap();
    types.method_parameters = variables[class_scope.len()..]
        .iter()
        .map(|(_, node)| *node)
        .collect();
    let parameter = bind_declared_type(
        &mut types,
        &signature.parameters[0],
        &list.loader,
        &variables,
        headers,
        budget,
    )
    .unwrap()
    .found()
    .unwrap();
    let result = bind_declared_type(
        &mut types,
        signature.result.as_ref().unwrap(),
        &list.loader,
        &variables,
        headers,
        budget,
    )
    .unwrap()
    .found()
    .unwrap();
    types.parameters.push((1, parameter));
    types.result = Some(result);
    prove_declared_relations(&mut types, headers, budget).unwrap();
    (types, list.methods[index].access_flags)
}

#[test]
fn declared_method_header_spells_actual_method_binders_and_slots() {
    // 窄修正前helper把raw name当UTF-8字符串；现在直接传classfile的JvmBytes并走项目MUTF-8拼写。
    // 这个固定正例保留同一实际List方法的参数slot、声明Signature和返回类型断言。
    let mut headers = Headers::full();
    let mut budget = Budget::new(limits());
    let (types, flags) = list_to_array_declaration_candidate(&mut headers, &mut budget);
    let header = declared_method_declaration(
        &types,
        &[(1, "values".to_owned())],
        b"toArray",
        flags,
        &[],
        &mut budget,
    )
    .unwrap();
    assert_eq!(
        header.as_deref(),
        Some("public abstract <T extends java.lang.Object> T[] toArray(T[] values)")
    );
}

#[test]
fn declared_method_header_refuses_unproved_slots_types_and_throws() {
    let mut headers = Headers::full();
    let mut budget = Budget::new(limits());
    let (types, flags) = list_to_array_declaration_candidate(&mut headers, &mut budget);
    assert!(
        declared_method_declaration(
            &types,
            &[(0, "values".to_owned())],
            b"toArray",
            flags,
            &[],
            &mut budget,
        )
        .unwrap()
        .is_none()
    );
    assert!(
        declared_method_declaration(
            &types,
            &[(1, "values".to_owned())],
            b"toArray",
            flags | 0x0080,
            &[],
            &mut budget,
        )
        .unwrap()
        .is_none()
    );
    assert!(
        declared_method_declaration(
            &types,
            &[(1, "values".to_owned())],
            b"toArray",
            flags,
            &["java.io.IOException".to_owned()],
            &mut budget,
        )
        .unwrap()
        .is_none()
    );

    let mut unproved = types;
    unproved.admissible.clear();
    assert!(
        declared_method_declaration(
            &unproved,
            &[(1, "values".to_owned())],
            b"toArray",
            flags,
            &[],
            &mut budget,
        )
        .unwrap()
        .is_none()
    );
}

fn child_mid_node(
    headers: &mut Headers,
    types: &mut DeclaredGenericTypes,
    budget: &mut Budget,
) -> usize {
    let child = headers.get(CHILD_LOADER, b"generic/Child").clone();
    let signature = child
        .generic_signatures
        .as_ref()
        .and_then(|signatures| signatures.class.as_ref())
        .expect("Child has its real class Signature")
        .0
        .clone();
    bind_declared_type(
        types,
        &SignatureType::Class(signature.superclass),
        &child.loader,
        &[],
        headers,
        budget,
    )
    .expect("selected headers resolve the Child superclass")
    .found()
    .expect("Child superclass is known")
}

#[test]
fn child_hierarchy_keeps_argument_loader_and_selected_physical_definitions() {
    let mut headers = Headers::full();
    let mut types = DeclaredGenericTypes::default();
    let mut budget = Budget::new(limits());
    let source = child_mid_node(&mut headers, &mut types, &mut budget);
    let base = headers.get(PARENT_LOADER, b"generic/Base").clone();
    let result = declared_supertype(
        &mut types,
        source,
        &reference_target(&base, b"generic/Base"),
        &mut headers,
        &mut budget,
    )
    .expect("generic parent walk completes");
    let DependencyResult::Found(base_node) = result else {
        panic!("Child reaches the selected parent Base definition")
    };
    let DeclaredGenericType::Reference {
        declaration,
        arguments,
    } = &types.types[base_node]
    else {
        panic!("target node is a reference")
    };
    assert_eq!(declaration.loader, LoaderId(PARENT_LOADER.to_owned()));
    assert_eq!(declaration.definition, base.definition);
    assert_eq!(arguments.len(), 1);
    let DeclaredGenericType::Reference {
        declaration: list,
        arguments,
    } = &types.types[arguments[0]]
    else {
        panic!("Base argument is List<Foo>")
    };
    assert_eq!(list.loader, LoaderId(BOOT_LOADER.to_owned()));
    assert_eq!(
        list.definition,
        headers.get(BOOT_LOADER, b"java/util/List").definition
    );
    assert_eq!(arguments.len(), 1);
    let DeclaredGenericType::Reference {
        declaration: foo,
        arguments,
    } = &types.types[arguments[0]]
    else {
        panic!("List argument is Foo")
    };
    assert!(arguments.is_empty());
    assert_eq!(foo.loader, LoaderId(CHILD_LOADER.to_owned()));
    assert_eq!(
        foo.definition,
        headers.get(CHILD_LOADER, b"identity/Foo").definition
    );
    assert_ne!(
        foo.definition,
        headers.get(PARENT_LOADER, b"identity/Foo").definition
    );

    let child_foo = bind_declared_type(
        &mut types,
        &field_type(b"Lidentity/Foo;"),
        &LoaderId(CHILD_LOADER.to_owned()),
        &[],
        &mut headers,
        &mut budget,
    )
    .unwrap()
    .found()
    .unwrap();
    let parent_foo = bind_declared_type(
        &mut types,
        &field_type(b"Lidentity/Foo;"),
        &LoaderId(PARENT_LOADER.to_owned()),
        &[],
        &mut headers,
        &mut budget,
    )
    .unwrap()
    .found()
    .unwrap();
    assert_ne!(child_foo, parent_foo);
    assert_ne!(types.types[child_foo], types.types[parent_foo]);
}

#[test]
fn declared_field_type_substitutes_class_variable_and_keeps_erased_declaration() {
    let mut headers = Headers::full();
    let mut types = DeclaredGenericTypes::default();
    let mut budget = Budget::new(limits());
    let source = child_mid_node(&mut headers, &mut types, &mut budget);
    let base = headers.get(PARENT_LOADER, b"generic/Base").clone();
    let field = field_target(&base, b"generic/Base", b"baseValue", b"Ljava/lang/Object;");
    let result = declared_member_types(&mut types, source, &field, 17, &mut headers, &mut budget)
        .expect("actual Base field declaration instantiates");
    let DependencyResult::Found(DeclaredGenericMemberTypes::Field {
        bci,
        declaration,
        application,
        ty,
        ..
    }) = result
    else {
        panic!("Base<T>.baseValue is a declared field")
    };
    assert_eq!(bci, 17);
    assert_eq!(declaration, field);
    let DeclaredGenericType::Reference {
        declaration: applied_base,
        arguments: applied_arguments,
    } = &types.types[application]
    else {
        panic!("the member application is the resolved Base class type")
    };
    assert_eq!(applied_base.definition, base.definition);
    assert_eq!(applied_arguments.as_slice(), &[ty]);
    assert!(matches!(
        &field.member,
        SymbolRef::Field { descriptor, .. } if descriptor.0 == b"Ljava/lang/Object;"
    ));
    let DeclaredGenericType::Reference {
        declaration: list,
        arguments,
    } = &types.types[ty]
    else {
        panic!("Base<T> is instantiated as List<Foo>")
    };
    assert_eq!(list.loader, LoaderId(BOOT_LOADER.to_owned()));
    assert_eq!(
        list.definition,
        headers.get(BOOT_LOADER, b"java/util/List").definition
    );
    let DeclaredGenericType::Reference {
        declaration: foo,
        arguments,
    } = &types.types[arguments[0]]
    else {
        panic!("List argument is the Child-selected Foo")
    };
    assert!(arguments.is_empty());
    assert_eq!(foo.loader, LoaderId(CHILD_LOADER.to_owned()));
    assert_eq!(
        foo.definition,
        headers.get(CHILD_LOADER, b"identity/Foo").definition
    );

    let mut wrong_definition = field.clone();
    wrong_definition.definition = headers
        .get(PARENT_LOADER, b"identity/Foo")
        .definition
        .clone();
    assert!(matches!(
        declared_member_types(
            &mut types,
            source,
            &wrong_definition,
            17,
            &mut headers,
            &mut budget,
        )
        .unwrap(),
        DependencyResult::Missing
    ));
}

#[test]
fn declared_list_get_uses_child_foo_and_method_type_binders_remain_unknown() {
    let mut headers = Headers::full();
    let mut types = DeclaredGenericTypes::default();
    let mut budget = Budget::new(limits());
    let foo_child = bind_declared_type(
        &mut types,
        &field_type(b"Lidentity/Foo;"),
        &LoaderId(CHILD_LOADER.to_owned()),
        &[],
        &mut headers,
        &mut budget,
    )
    .unwrap()
    .found()
    .unwrap();
    let variables = vec![(b"F".to_vec(), foo_child)];
    let list_child = bind_declared_type(
        &mut types,
        &field_type(b"Ljava/util/List<TF;>;"),
        &LoaderId(PARENT_LOADER.to_owned()),
        &variables,
        &mut headers,
        &mut budget,
    )
    .unwrap()
    .found()
    .unwrap();

    let list = headers.get(BOOT_LOADER, b"java/util/List").clone();
    let get = method_target(&list, b"java/util/List", b"get", b"(I)Ljava/lang/Object;");
    let result = declared_member_types(&mut types, list_child, &get, 29, &mut headers, &mut budget)
        .expect("physical List.get declaration is found");
    let DependencyResult::Found(DeclaredGenericMemberTypes::Method {
        bci,
        declaration,
        application,
        parameters,
        result: Some(result),
        throws,
        ..
    }) = result
    else {
        panic!("List<E>.get(int) has a bound result")
    };
    assert_eq!(bci, 29);
    assert_eq!(declaration, get);
    assert_eq!(application, list_child);
    assert!(matches!(
        &get.member,
        SymbolRef::Method { descriptor, .. } if descriptor.0 == b"(I)Ljava/lang/Object;"
    ));
    assert!(matches!(
        types.types[parameters[0]],
        DeclaredGenericType::Base(b'I')
    ));
    assert_eq!(result, foo_child);
    assert!(throws.is_empty());
    assert!(matches!(
        &types.types[result],
        DeclaredGenericType::Reference { declaration, .. }
            if declaration.loader == LoaderId(CHILD_LOADER.to_owned())
                && declaration.definition == headers.get(CHILD_LOADER, b"identity/Foo").definition
    ));

    let signatures = list.generic_signatures.as_ref().unwrap();
    let (index, to_array_signature) = signatures
        .methods
        .iter()
        .enumerate()
        .find_map(|(index, method)| {
            method
                .as_ref()
                .filter(|(signature, _)| !signature.type_parameters.is_empty())
                .map(|(signature, _)| (index, signature))
        })
        .expect("the real List header contains a method-local generic declaration");
    assert_eq!(list.methods[index].name.raw().0, b"toArray");
    assert_eq!(
        list.methods[index].descriptor.raw().0,
        b"([Ljava/lang/Object;)[Ljava/lang/Object;"
    );
    assert!(!to_array_signature.type_parameters.is_empty());
    let to_array = method_target(
        &list,
        b"java/util/List",
        b"toArray",
        b"([Ljava/lang/Object;)[Ljava/lang/Object;",
    );
    assert!(matches!(
        declared_member_types(
            &mut types,
            list_child,
            &to_array,
            41,
            &mut headers,
            &mut budget,
        )
        .unwrap(),
        DependencyResult::Unknown
    ));
}

#[test]
fn same_named_class_and_method_variables_have_distinct_scopes_and_shadowing() {
    let mut headers = Headers::full();
    let header = headers.get(CHILD_LOADER, b"generic/Child").clone();
    let mut types = DeclaredGenericTypes::default();
    let mut budget = Budget::new(limits());
    let class_signature =
        parse_class_signature(b"<T:Ljava/lang/Object;>Ljava/lang/Object;", &mut budget).unwrap();
    let class_proof =
        prove_class_signature_erasure(&class_signature, b"java/lang/Object", &[], &mut budget)
            .unwrap();
    let class_scope = bind_declared_parameters(
        &mut types,
        &header,
        None,
        &class_signature.type_parameters,
        &class_proof.type_parameters,
        &[],
        &mut headers,
        &mut budget,
    )
    .unwrap()
    .found()
    .unwrap();
    let class_t = class_scope[0].1;

    let method_signature =
        jarde_reader::signature::parse_method_signature(b"<T:Ljava/lang/String;>()V", &mut budget)
            .unwrap();
    let method_proof = prove_method_signature_erasure_with_class_scope(
        &method_signature,
        b"()V",
        &[],
        &class_proof.type_parameters,
        &mut budget,
    )
    .unwrap();
    let method_scope = bind_declared_parameters(
        &mut types,
        &header,
        Some((b"shadow", b"()V")),
        &method_signature.type_parameters,
        &method_proof.type_parameters,
        &class_scope,
        &mut headers,
        &mut budget,
    )
    .unwrap()
    .found()
    .unwrap();
    let method_t = method_scope[1].1;
    assert_ne!(class_t, method_t);
    assert!(matches!(
        &types.types[class_t],
        DeclaredGenericType::Variable { method: None, ordinal: 0, name, .. } if name.0 == b"T"
    ));
    assert!(matches!(
        &types.types[method_t],
        DeclaredGenericType::Variable { method: Some((name, descriptor)), ordinal: 0, .. }
            if name.0 == b"shadow" && descriptor.0 == b"()V"
    ));

    let bound = bind_declared_type(
        &mut types,
        &SignatureType::TypeVariable(b"T".to_vec()),
        &header.loader,
        &method_scope,
        &mut headers,
        &mut budget,
    )
    .unwrap()
    .found()
    .unwrap();
    assert_eq!(bound, method_t, "the later method scope shadows class T");
    let class_bound = bind_declared_type(
        &mut types,
        &SignatureType::TypeVariable(b"T".to_vec()),
        &header.loader,
        &class_scope,
        &mut headers,
        &mut budget,
    )
    .unwrap()
    .found()
    .unwrap();
    assert_eq!(class_bound, class_t);
}

#[test]
fn incomplete_leaf_wrong_physical_target_arity_and_wildcards_do_not_authorize_binding() {
    let mut headers = Headers::full();
    headers.remove(CHILD_LOADER, b"identity/Foo");
    let mut types = DeclaredGenericTypes::default();
    let mut budget = Budget::new(limits());
    let child = headers.get(CHILD_LOADER, b"generic/Child").clone();
    let signature = child
        .generic_signatures
        .as_ref()
        .unwrap()
        .class
        .as_ref()
        .unwrap()
        .0
        .clone();
    assert!(matches!(
        bind_declared_type(
            &mut types,
            &SignatureType::Class(signature.superclass),
            &child.loader,
            &[],
            &mut headers,
            &mut budget,
        )
        .unwrap(),
        DependencyResult::Missing
    ));

    let mut headers = Headers::full();
    let mut types = DeclaredGenericTypes::default();
    let mut budget = Budget::new(limits());
    let source = child_mid_node(&mut headers, &mut types, &mut budget);
    let wrong_target = headers.get(PARENT_LOADER, b"identity/Foo").clone();
    assert!(matches!(
        declared_supertype(
            &mut types,
            source,
            &reference_target(&wrong_target, b"identity/Foo"),
            &mut headers,
            &mut budget,
        )
        .unwrap(),
        DependencyResult::Missing
    ));

    let over_arity = field_type(b"Lgeneric/Mid<Lidentity/Foo;Lidentity/Foo;>;");
    assert!(matches!(
        bind_declared_type(
            &mut types,
            &over_arity,
            &LoaderId(CHILD_LOADER.to_owned()),
            &[],
            &mut headers,
            &mut budget,
        )
        .unwrap(),
        DependencyResult::Unknown
    ));
    let wildcard = field_type(b"Lgeneric/Mid<+Lidentity/Foo;>;");
    assert!(matches!(
        bind_declared_type(
            &mut types,
            &wildcard,
            &LoaderId(CHILD_LOADER.to_owned()),
            &[],
            &mut headers,
            &mut budget,
        )
        .unwrap(),
        DependencyResult::Unknown
    ));
}

#[test]
fn caller_injected_physical_header_cycle_returns_unknown_without_fabricated_bytes() {
    let mut headers = Headers::full();
    // The bytes remain the actual Mid classfile. This provider-only alias deliberately feeds
    // that same physical header for the `generic/Base` lookup to exercise ancestor-cycle handling.
    let mid = headers.get(PARENT_LOADER, b"generic/Mid").clone();
    headers.insert(PARENT_LOADER, b"generic/Base", mid.clone());
    let mut types = DeclaredGenericTypes::default();
    let mut budget = Budget::new(limits());
    let source = child_mid_node(&mut headers, &mut types, &mut budget);
    let wrong_target = headers.get(PARENT_LOADER, b"identity/Foo").clone();
    assert!(matches!(
        declared_supertype(
            &mut types,
            source,
            &reference_target(&wrong_target, b"identity/Foo"),
            &mut headers,
            &mut budget,
        )
        .unwrap(),
        DependencyResult::Unknown
    ));
    let repeated_mid_definition = headers
        .lookups
        .iter()
        .filter(|(loader, name, selected)| {
            loader.as_str() == PARENT_LOADER
                && name.as_slice() == b"generic/Base"
                && selected
                    .as_ref()
                    .is_some_and(|(selected_loader, definition)| {
                        selected_loader == &mid.loader && definition == &mid.definition
                    })
        })
        .count();
    assert!(
        repeated_mid_definition >= 2,
        "the injected Base lookup selected the same physical Mid definition twice"
    );
}

#[test]
fn type_binding_preserves_budget_and_precancellation_errors() {
    let mut headers = Headers::full();
    let mut types = DeclaredGenericTypes::default();
    let mut constrained = limits();
    constrained.analysis_steps = 0;
    let mut budget = Budget::new(constrained);
    let error = match bind_declared_type(
        &mut types,
        &field_type(b"Ljava/lang/String;"),
        &LoaderId(BOOT_LOADER.to_owned()),
        &[],
        &mut headers,
        &mut budget,
    ) {
        Err(error) => error,
        Ok(_) => panic!("first analysis step exceeds the explicit limit"),
    };
    assert!(matches!(error, Error::BudgetExceeded { .. }));

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), cancellation);
    let error = match bind_declared_type(
        &mut types,
        &field_type(b"Ljava/lang/String;"),
        &LoaderId(BOOT_LOADER.to_owned()),
        &[],
        &mut headers,
        &mut budget,
    ) {
        Err(error) => error,
        Ok(_) => panic!("pre-cancellation propagates from the first poll"),
    };
    assert!(matches!(error, Error::Cancelled { .. }));
}

fn own_field_index(header: &ReferenceClassHeader, name: &[u8]) -> usize {
    header
        .fields
        .iter()
        .position(|field| field.name.raw().0 == name)
        .expect("fixture has the requested physical field")
}

#[test]
fn declared_field_binding_uses_the_selected_class_variable_node() {
    let mut headers = Headers::full();
    let base = headers.get(PARENT_LOADER, b"generic/Base").clone();
    let actual = reference_target(&base, b"generic/Base");
    let mut budget = Budget::new(limits());
    let (mut types, _, _) = declared_class_types(&actual, &base, &mut headers, &mut budget)
        .unwrap()
        .found()
        .expect("Base's physical class Signature binds");
    let index = own_field_index(&base, b"baseValue");
    let node = bind_declared_field_type(&mut types, &base, index, &mut headers, &mut budget)
        .unwrap()
        .found()
        .expect("Base.baseValue Signature binds");
    assert_eq!(types.class_parameters, vec![node]);
    assert!(matches!(
        &types.types[node],
        DeclaredGenericType::Variable { owner, method: None, ordinal: 0, name, .. }
            if owner == &base.definition && name.0 == b"T"
    ));
}

#[test]
fn declared_field_binding_refuses_wrong_header_identity_and_absent_signature() {
    let mut headers = Headers::full();
    let base = headers.get(PARENT_LOADER, b"generic/Base").clone();
    let actual = reference_target(&base, b"generic/Base");
    let mut budget = Budget::new(limits());
    let (mut types, _, _) = declared_class_types(&actual, &base, &mut headers, &mut budget)
        .unwrap()
        .found()
        .expect("Base's physical class Signature binds");
    let base_index = own_field_index(&base, b"baseValue");

    let wrong_loader = headers.get(CHILD_LOADER, b"identity/Foo").clone();
    assert!(matches!(
        bind_declared_field_type(
            &mut types,
            &wrong_loader,
            base_index,
            &mut headers,
            &mut budget
        )
        .unwrap(),
        DependencyResult::Unknown
    ));
    let wrong_definition = headers.get(PARENT_LOADER, b"generic/Mid").clone();
    assert!(matches!(
        bind_declared_field_type(
            &mut types,
            &wrong_definition,
            base_index,
            &mut headers,
            &mut budget
        )
        .unwrap(),
        DependencyResult::Unknown
    ));

    let child = headers.get(CHILD_LOADER, b"generic/Child").clone();
    let child_actual = reference_target(&child, b"generic/Child");
    let (mut child_types, _, _) =
        declared_class_types(&child_actual, &child, &mut headers, &mut budget)
            .unwrap()
            .found()
            .expect("Child has a physical class header");
    let child_index = own_field_index(&child, b"childValue");
    assert!(child.generic_signatures.as_ref().unwrap().fields[child_index].is_none());
    assert!(matches!(
        bind_declared_field_type(
            &mut child_types,
            &child,
            child_index,
            &mut headers,
            &mut budget
        )
        .unwrap(),
        DependencyResult::Unknown
    ));
}

#[test]
fn declared_field_binding_propagates_precancellation() {
    let mut headers = Headers::full();
    let base = headers.get(PARENT_LOADER, b"generic/Base").clone();
    let actual = reference_target(&base, b"generic/Base");
    let mut setup_budget = Budget::new(limits());
    let (mut types, _, _) = declared_class_types(&actual, &base, &mut headers, &mut setup_budget)
        .unwrap()
        .found()
        .expect("Base's physical class Signature binds");
    let index = own_field_index(&base, b"baseValue");

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), cancellation);
    assert!(matches!(
        bind_declared_field_type(&mut types, &base, index, &mut headers, &mut budget),
        Err(Error::Cancelled { .. })
    ));
}
