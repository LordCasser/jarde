//! Real classfile Signature facts and generic-supertype paths. This tests the evidence helper only;
//! it does not evaluate the target bytecode or claim Engine/Builder admission.

use jarde_java::source_proof::{
    DependencyResult, RecoveryHeaders, ReferenceClassHeader, declared_generic_field_type,
    declared_generic_method_signature, declared_generic_supertype,
};
use jarde_jvm::resolver::ResolvedMemberRef;
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::classfile::{class_constant_pool, class_member_facts};
use jarde_reader::error::Error;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalVariant,
    SymbolRef,
};
use jarde_reader::signature::{
    ClassType, ClassTypeSegment, FieldSignature, SignatureType, TypeArgument, parse_field_signature,
};
use jarde_reader::view::LoaderId;
use std::collections::BTreeMap;

const CHILD: &[u8] = include_bytes!("fixtures/s2b-signature-paths-r1/custom/CustomChild.class");
const MID: &[u8] = include_bytes!("fixtures/s2b-signature-paths-r1/custom/Mid.class");
const PARENT: &[u8] = include_bytes!("fixtures/s2b-signature-paths-r1/custom/Parent.class");
const ARRAY_LIST: &[u8] =
    include_bytes!("fixtures/s2b-signature-paths-r1/platform/java/util/ArrayList.class");
const ABSTRACT_LIST: &[u8] =
    include_bytes!("fixtures/s2b-signature-paths-r1/platform/java/util/AbstractList.class");
const ABSTRACT_COLLECTION: &[u8] =
    include_bytes!("fixtures/s2b-signature-paths-r1/platform/java/util/AbstractCollection.class");
const LIST: &[u8] = include_bytes!("fixtures/s2b-signature-paths-r1/platform/java/util/List.class");
const COLLECTION: &[u8] =
    include_bytes!("fixtures/s2b-signature-paths-r1/platform/java/util/Collection.class");
const ITERABLE: &[u8] =
    include_bytes!("fixtures/s2b-signature-paths-r1/platform/java/lang/Iterable.class");
const OBJECT: &[u8] =
    include_bytes!("fixtures/s2b-signature-paths-r1/platform/java/lang/Object.class");
const RANDOM_ACCESS: &[u8] =
    include_bytes!("fixtures/s2b-signature-paths-r1/platform/java/util/RandomAccess.class");
const CLONEABLE: &[u8] =
    include_bytes!("fixtures/s2b-signature-paths-r1/platform/java/lang/Cloneable.class");
const SERIALIZABLE: &[u8] =
    include_bytes!("fixtures/s2b-signature-paths-r1/platform/java/io/Serializable.class");

const APP: &str = "app";
const BOOT: &str = "bootstrap";

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
        class_headers: 32,
        method_bodies: 0,
        ir_items: 1 << 16,
        ir_edges: 1 << 16,
        analysis_steps: 1 << 16,
        normalization_clones: 0,
        nested_depth: 8,
        dependency_depth: 16,
        elapsed_millis: u64::MAX,
    }
}

fn header(bytes: &[u8], loader: &str) -> ReferenceClassHeader {
    let mut budget = Budget::new(limits());
    let facts = class_member_facts(bytes, &mut budget).expect("real class member facts decode");
    let pool = class_constant_pool(bytes, &budget).expect("real class constant pool decodes");
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("class bytes open as an artifact snapshot");
    let identity = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: u64::try_from(bytes.len()).expect("fixture length fits"),
        },
        variant: PhysicalVariant::Base,
    };
    ReferenceClassHeader::from_facts(&facts, LoaderId(loader.to_owned()), identity)
        .expect("complete real class header")
        .with_generic_signatures(bytes, &facts.attributes, &pool, &mut budget)
        .expect("Signature parses and erasure matches descriptor/class header")
}

#[derive(Default)]
struct Headers {
    selected: BTreeMap<(String, Vec<u8>), ReferenceClassHeader>,
}

impl Headers {
    fn insert_as(&mut self, request_loader: &str, raw_name: &[u8], header: ReferenceClassHeader) {
        self.selected
            .insert((request_loader.to_owned(), raw_name.to_vec()), header);
    }

    fn custom() -> Self {
        let mut headers = Self::default();
        for (name, bytes) in [
            (b"CustomChild".as_slice(), CHILD),
            (b"Mid", MID),
            (b"Parent", PARENT),
        ] {
            headers.insert_as(APP, name, header(bytes, APP));
        }
        headers
    }

    fn platform() -> Self {
        let mut headers = Self::default();
        headers.insert_as(APP, b"java/util/ArrayList", header(ARRAY_LIST, BOOT));
        for (name, bytes) in [
            (b"java/util/AbstractList".as_slice(), ABSTRACT_LIST),
            (b"java/util/AbstractCollection", ABSTRACT_COLLECTION),
            (b"java/util/List", LIST),
            (b"java/util/Collection", COLLECTION),
            (b"java/lang/Iterable", ITERABLE),
            (b"java/lang/Object", OBJECT),
            (b"java/util/RandomAccess", RANDOM_ACCESS),
            (b"java/lang/Cloneable", CLONEABLE),
            (b"java/io/Serializable", SERIALIZABLE),
        ] {
            headers.insert_as(BOOT, name, header(bytes, BOOT));
        }
        headers
    }
}

impl RecoveryHeaders for Headers {
    type Error = Error;

    fn caller_loader(&self) -> &LoaderId {
        static APP_LOADER: std::sync::OnceLock<LoaderId> = std::sync::OnceLock::new();
        APP_LOADER.get_or_init(|| LoaderId(APP.to_owned()))
    }

    fn selected_from(
        &mut self,
        initiating_loader: &LoaderId,
        owner: &[u8],
        budget: &mut Budget,
    ) -> std::result::Result<DependencyResult<ReferenceClassHeader>, Self::Error> {
        budget.poll()?;
        Ok(self
            .selected
            .get(&(initiating_loader.0.clone(), owner.to_vec()))
            .cloned()
            .map_or(DependencyResult::Missing, DependencyResult::Found))
    }

    fn selected_generic_from(
        &mut self,
        initiating_loader: &LoaderId,
        owner: &[u8],
        budget: &mut Budget,
    ) -> std::result::Result<DependencyResult<ReferenceClassHeader>, Self::Error> {
        self.selected_from(initiating_loader, owner, budget)
    }

    fn platform_unprovided(
        &mut self,
        _owner: &[u8],
        budget: &mut Budget,
    ) -> std::result::Result<bool, Self::Error> {
        budget.poll()?;
        Ok(true)
    }
}

fn class_type(signature: &[u8]) -> ClassType {
    let mut budget = Budget::new(limits());
    let FieldSignature {
        ty: SignatureType::Class(class),
    } = parse_field_signature(signature, &mut budget).expect("frozen exact class signature parses")
    else {
        panic!("expected class type signature")
    };
    class
}

/// This constructs a test-supplier declaration only. Production callers must use
/// RecoveryHeaders::resolve_member on the original resolved symbol and use site.
fn resolved_member(
    header: &ReferenceClassHeader,
    owner: &[u8],
    name: &[u8],
    descriptor: &[u8],
    method: bool,
) -> ResolvedMemberRef {
    let found =
        if method {
            header.methods.iter().any(|member| {
                member.name.raw().0 == name && member.descriptor.raw().0 == descriptor
            })
        } else {
            header.fields.iter().any(|member| {
                member.name.raw().0 == name && member.descriptor.raw().0 == descriptor
            })
        };
    assert!(found, "test member is present in the physical header");
    ResolvedMemberRef {
        loader: header.loader.clone(),
        definition: header.definition.clone(),
        member: if method {
            SymbolRef::Method {
                owner: JvmBytes(owner.to_vec()),
                name: JvmBytes(name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            }
        } else {
            SymbolRef::Field {
                owner: JvmBytes(owner.to_vec()),
                name: JvmBytes(name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            }
        },
    }
}

fn class_type_argument(class: ClassType) -> TypeArgument {
    TypeArgument::Exact(SignatureType::Class(class))
}

fn method_signature<'a>(
    header: &'a ReferenceClassHeader,
    name: &[u8],
    descriptor: &[u8],
) -> &'a jarde_reader::signature::MethodSignature {
    let index = header
        .methods
        .iter()
        .position(|member| member.name.raw().0 == name && member.descriptor.raw().0 == descriptor)
        .expect("method descriptor exists in the real header");
    header
        .generic_signatures
        .as_ref()
        .expect("generic bundle exists")
        .methods[index]
        .as_ref()
        .map(|(signature, _proof)| signature)
        .expect("member has its own generic Signature")
}

#[test]
fn real_custom_and_platform_signatures_propagate_over_selected_physical_headers() {
    let source = class_type(b"LCustomChild;");
    let expected = class_type(b"LParent<Ljava/util/List<Ljava/lang/String;>;>;");
    let parent_header = header(PARENT, APP);
    let parent_definition = parent_header.definition.clone();
    let mut custom = Headers::custom();
    let actual = declared_generic_supertype(
        &source,
        &LoaderId(APP.to_owned()),
        b"Parent",
        &mut custom,
        &mut Budget::new(limits()),
    )
    .expect("complete custom path");
    let DependencyResult::Found((found_header, found_type)) = actual else {
        panic!("Child -> Mid -> Parent<List<String>> is established from real Signatures");
    };
    assert_eq!(found_header.definition, parent_definition);
    assert_eq!(found_type, expected);

    // Parent<T>'s field and get/set methods retain T from their own physical Signature attributes.
    let parent = header(PARENT, APP);
    let field_index = parent
        .fields
        .iter()
        .position(|field| field.name.raw().0 == b"field")
        .expect("Parent.field is in the classfile");
    assert_eq!(
        parent.generic_signatures.as_ref().unwrap().fields[field_index]
            .as_ref()
            .unwrap()
            .ty,
        SignatureType::TypeVariable(b"T".to_vec()),
    );
    let get = method_signature(&parent, b"get", b"()Ljava/lang/Object;");
    assert_eq!(get.result, Some(SignatureType::TypeVariable(b"T".to_vec())));
    let set = method_signature(&parent, b"set", b"(Ljava/lang/Object;)V");
    assert_eq!(
        set.parameters,
        vec![SignatureType::TypeVariable(b"T".to_vec())]
    );
    assert_eq!(set.result, None);

    // ArrayList<String> reaches List<String> through the pinned, real JDK 8 classfile closure.
    let source = class_type(b"Ljava/util/ArrayList<Ljava/lang/String;>;");
    let expected = class_type(b"Ljava/util/List<Ljava/lang/String;>;");
    let mut platform = Headers::platform();
    let actual = declared_generic_supertype(
        &source,
        &LoaderId(APP.to_owned()),
        b"java/util/List",
        &mut platform,
        &mut Budget::new(limits()),
    )
    .expect("complete platform path");
    let DependencyResult::Found((list_header, found_type)) = actual else {
        panic!("ArrayList<String> -> List<String> is established from the selected JDK headers");
    };
    assert_eq!(list_header.loader, LoaderId(BOOT.to_owned()));
    assert_eq!(found_type, expected);
    let list = header(LIST, BOOT);
    let get = method_signature(&list, b"get", b"(I)Ljava/lang/Object;");
    assert_eq!(get.parameters, vec![SignatureType::Base(b'I')]);
    assert_eq!(get.result, Some(SignatureType::TypeVariable(b"E".to_vec())));
    let set = method_signature(&list, b"set", b"(ILjava/lang/Object;)Ljava/lang/Object;");
    assert_eq!(
        set.parameters,
        vec![
            SignatureType::Base(b'I'),
            SignatureType::TypeVariable(b"E".to_vec())
        ]
    );
    assert_eq!(set.result, Some(SignatureType::TypeVariable(b"E".to_vec())));
}

#[test]
fn arity_wildcard_missing_budget_and_cancellation_never_produce_found() {
    let mut custom = Headers::custom();
    for malformed in [
        class_type(b"LParent<Ljava/lang/String;Ljava/lang/Integer;>;"),
        class_type(b"LParent<*>;"),
    ] {
        let result = declared_generic_supertype(
            &malformed,
            &LoaderId(APP.to_owned()),
            b"Parent",
            &mut custom,
            &mut Budget::new(limits()),
        )
        .expect("unsupported declared source type is a normal dependency result");
        assert!(matches!(result, DependencyResult::Unknown));
    }

    let source = class_type(b"LCustomChild;");
    let mut missing = Headers::custom();
    missing.selected.remove(&(APP.to_owned(), b"Mid".to_vec()));
    assert!(matches!(
        declared_generic_supertype(
            &source,
            &LoaderId(APP.to_owned()),
            b"Parent",
            &mut missing,
            &mut Budget::new(limits())
        )
        .unwrap(),
        DependencyResult::Unknown,
    ));

    let mut no_steps = limits();
    no_steps.analysis_steps = 0;
    assert!(
        declared_generic_supertype(
            &source,
            &LoaderId(APP.to_owned()),
            b"Parent",
            &mut Headers::custom(),
            &mut Budget::new(no_steps),
        )
        .is_err()
    );

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(), token);
    assert!(
        declared_generic_supertype(
            &source,
            &LoaderId(APP.to_owned()),
            b"Parent",
            &mut Headers::custom(),
            &mut cancelled,
        )
        .is_err()
    );
}

#[test]
fn resolved_custom_and_platform_members_instantiate_class_binders_only() {
    let list_of_string = SignatureType::Class(class_type(b"Ljava/util/List<Ljava/lang/String;>;"));
    let child = class_type(b"LCustomChild;");
    let parent = header(PARENT, APP);
    let parent_field = resolved_member(&parent, b"Parent", b"field", b"Ljava/lang/Object;", false);
    let mut custom = Headers::custom();
    let field_ty = declared_generic_field_type(
        &child,
        &LoaderId(APP.to_owned()),
        &parent_field,
        &mut custom,
        &mut Budget::new(limits()),
    )
    .expect("resolved physical field is instantiated");
    assert!(matches!(field_ty, DependencyResult::Found(ty) if ty == list_of_string));
    assert!(
        matches!(&parent_field.member, SymbolRef::Field { descriptor, .. } if descriptor.0 == b"Ljava/lang/Object;")
    );

    let parent_get = resolved_member(&parent, b"Parent", b"get", b"()Ljava/lang/Object;", true);
    let get = declared_generic_method_signature(
        &child,
        &LoaderId(APP.to_owned()),
        &parent_get,
        &mut custom,
        &mut Budget::new(limits()),
    )
    .expect("resolved physical getter is instantiated");
    let DependencyResult::Found(get) = get else {
        panic!("Parent.get returns List<String>")
    };
    assert_eq!(get.parameters, Vec::<SignatureType>::new());
    assert_eq!(get.result, Some(list_of_string.clone()));
    assert!(
        matches!(&parent_get.member, SymbolRef::Method { descriptor, .. } if descriptor.0 == b"()Ljava/lang/Object;")
    );

    let parent_set = resolved_member(&parent, b"Parent", b"set", b"(Ljava/lang/Object;)V", true);
    let set = declared_generic_method_signature(
        &child,
        &LoaderId(APP.to_owned()),
        &parent_set,
        &mut custom,
        &mut Budget::new(limits()),
    )
    .expect("resolved physical setter is instantiated");
    let DependencyResult::Found(set) = set else {
        panic!("Parent.set consumes List<String>")
    };
    assert_eq!(set.parameters, vec![list_of_string.clone()]);
    assert_eq!(set.result, None);
    assert!(
        matches!(&parent_set.member, SymbolRef::Method { descriptor, .. } if descriptor.0 == b"(Ljava/lang/Object;)V")
    );

    let array_list = class_type(b"Ljava/util/ArrayList<Ljava/lang/String;>;");
    let list_header = header(LIST, BOOT);
    let list_get = resolved_member(
        &list_header,
        b"java/util/List",
        b"get",
        b"(I)Ljava/lang/Object;",
        true,
    );
    let mut platform = Headers::platform();
    let get = declared_generic_method_signature(
        &array_list,
        &LoaderId(APP.to_owned()),
        &list_get,
        &mut platform,
        &mut Budget::new(limits()),
    )
    .expect("resolved List.get is instantiated");
    let DependencyResult::Found(get) = get else {
        panic!("List.get returns String in ArrayList<String> context")
    };
    assert_eq!(get.parameters, vec![SignatureType::Base(b'I')]);
    assert_eq!(
        get.result,
        Some(SignatureType::Class(class_type(b"Ljava/lang/String;")))
    );
    assert!(
        matches!(&list_get.member, SymbolRef::Method { descriptor, .. } if descriptor.0 == b"(I)Ljava/lang/Object;")
    );

    let list_set = resolved_member(
        &list_header,
        b"java/util/List",
        b"set",
        b"(ILjava/lang/Object;)Ljava/lang/Object;",
        true,
    );
    let set = declared_generic_method_signature(
        &array_list,
        &LoaderId(APP.to_owned()),
        &list_set,
        &mut platform,
        &mut Budget::new(limits()),
    )
    .expect("resolved List.set is instantiated");
    let DependencyResult::Found(set) = set else {
        panic!("List.set consumes and returns String in this context")
    };
    assert_eq!(
        set.parameters,
        vec![
            SignatureType::Base(b'I'),
            SignatureType::Class(class_type(b"Ljava/lang/String;"))
        ]
    );
    assert_eq!(
        set.result,
        Some(SignatureType::Class(class_type(b"Ljava/lang/String;")))
    );
    assert!(
        matches!(&list_set.member, SymbolRef::Method { descriptor, .. } if descriptor.0 == b"(ILjava/lang/Object;)Ljava/lang/Object;")
    );
}

#[test]
fn member_identity_loader_method_binder_and_injected_cycle_fail_closed() {
    let source = class_type(b"LCustomChild;");
    let parent = header(PARENT, APP);
    let field = resolved_member(&parent, b"Parent", b"field", b"Ljava/lang/Object;", false);
    let mut wrong_definition = field.clone();
    wrong_definition.definition = header(CHILD, APP).definition;
    let result = declared_generic_field_type(
        &source,
        &LoaderId(APP.to_owned()),
        &wrong_definition,
        &mut Headers::custom(),
        &mut Budget::new(limits()),
    )
    .unwrap();
    assert!(matches!(result, DependencyResult::Unknown));

    let mut wrong_loader = field;
    wrong_loader.loader = LoaderId("other-loader".to_owned());
    let result = declared_generic_field_type(
        &source,
        &LoaderId(APP.to_owned()),
        &wrong_loader,
        &mut Headers::custom(),
        &mut Budget::new(limits()),
    )
    .unwrap();
    assert!(matches!(result, DependencyResult::Unknown));

    let list = header(LIST, BOOT);
    let method_generic = resolved_member(
        &list,
        b"java/util/List",
        b"toArray",
        b"([Ljava/lang/Object;)[Ljava/lang/Object;",
        true,
    );
    let result = declared_generic_method_signature(
        &class_type(b"Ljava/util/ArrayList<Ljava/lang/String;>;"),
        &LoaderId(APP.to_owned()),
        &method_generic,
        &mut Headers::platform(),
        &mut Budget::new(limits()),
    )
    .unwrap();
    assert!(
        matches!(result, DependencyResult::Unknown),
        "method-level <T> binder is not captured from class E"
    );

    // Caller-supplied graph variant: actual classfile identity is retained, while Mid's parsed edge
    // is deliberately replaced with a cycle. This checks cycle classification only, not physical evidence.
    let mut cyclic = Headers::custom();
    let mid_key = (APP.to_owned(), b"Mid".to_vec());
    let mid = cyclic.selected.get_mut(&mid_key).unwrap();
    let signatures = std::sync::Arc::make_mut(mid.generic_signatures.as_mut().unwrap());
    let class = signatures.class.as_mut().unwrap();
    class.0.superclass = ClassType {
        segments: vec![ClassTypeSegment {
            binary_name: b"CustomChild".to_vec(),
            arguments: vec![class_type_argument(class_type(b"Ljava/lang/String;"))],
        }],
    };
    let result = declared_generic_supertype(
        &source,
        &LoaderId(APP.to_owned()),
        b"Parent",
        &mut cyclic,
        &mut Budget::new(limits()),
    )
    .unwrap();
    assert!(
        matches!(result, DependencyResult::Unknown),
        "injected ancestor cycle is Unknown, not Missing"
    );
}
