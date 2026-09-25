use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::{attribute_facts, class_facts};
use jarde_reader::error::Error;
use jarde_reader::signature::{
    MethodSignature, SignatureType, parse_method_signature, prove_method_signature_erasure,
};

const ORIGINAL: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/GenericMethodProbe.class"
);
const THROWS_WITHOUT_SUFFIX: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/GenericThrowsProbe.class"
);
const OBJECT_BOUND: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/object-bound.class"
);
const UNBOUND_VARIABLE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/unbound-variable.class"
);
const CLASS_VARIABLE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/class-variable.class"
);
const COMPLEX_METHOD: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/complex-method.class"
);
const INCOMPATIBLE_BODY: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/incompatible-body.class"
);

fn budget() -> Budget {
    Budget::new(Limits {
        class_bytes: u64::MAX,
        attribute_bytes: u64::MAX,
        analysis_steps: u64::MAX,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    })
}

fn member_signature(bytes: &[u8], budget: &mut Budget) -> (MethodSignature, Vec<u8>, Vec<Vec<u8>>) {
    let class = class_facts(bytes, budget).unwrap();
    let method = class
        .methods
        .iter()
        .find(|method| method.name.raw().0 == b"choose")
        .unwrap();
    let facts = attribute_facts(bytes, &method.attributes, &class.constant_pool, budget).unwrap();
    let signature = parse_method_signature(&facts.signature.unwrap().0, budget).unwrap();
    (
        signature,
        method.descriptor.raw().0.clone(),
        facts.exceptions.into_iter().map(|name| name.0).collect(),
    )
}

#[test]
fn frozen_method_attributes_prove_the_simple_generic_declaration() {
    let mut budget = budget();
    let (signature, descriptor, exceptions) = member_signature(ORIGINAL, &mut budget);
    assert_eq!(signature.type_parameters[0].name, b"T");
    let proof =
        prove_method_signature_erasure(&signature, &descriptor, &exceptions, &mut budget).unwrap();
    assert_eq!(
        proof.parameters,
        vec![
            b"Ljava/lang/Number;".to_vec(),
            b"Ljava/lang/Number;".to_vec(),
            b"Z".to_vec()
        ]
    );
    assert_eq!(proof.result, Some(b"Ljava/lang/Number;".to_vec()));
}

#[test]
fn ordinary_exceptions_survive_an_absent_generic_throws_suffix() {
    let mut budget = budget();
    let (signature, descriptor, exceptions) = member_signature(THROWS_WITHOUT_SUFFIX, &mut budget);
    assert_eq!(signature.throws, []);
    assert_eq!(exceptions, vec![b"java/io/IOException".to_vec()]);
    let proof =
        prove_method_signature_erasure(&signature, &descriptor, &exceptions, &mut budget).unwrap();
    assert_eq!(proof.parameters, vec![b"Ljava/lang/Number;".to_vec()]);
    assert_eq!(proof.result, Some(b"Ljava/lang/Number;".to_vec()));
    assert!(proof.throws.is_empty());
}

#[test]
fn frozen_negative_classfiles_split_erasure_scope_and_later_projection_proofs() {
    for (label, bytes, expected_code) in [
        (
            "object bound",
            OBJECT_BOUND,
            "jvm_signature_erasure_mismatch",
        ),
        (
            "unbound variable",
            UNBOUND_VARIABLE,
            "jvm_signature_scope_unproved",
        ),
        (
            "class variable",
            CLASS_VARIABLE,
            "jvm_signature_scope_unproved",
        ),
    ] {
        let mut budget = budget();
        let (signature, descriptor, exceptions) = member_signature(bytes, &mut budget);
        let error =
            prove_method_signature_erasure(&signature, &descriptor, &exceptions, &mut budget)
                .unwrap_err();
        let code = match error {
            Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code,
            other => panic!("{label}: unexpected error {other:?}"),
        };
        assert_eq!(code, expected_code, "{label}");
    }

    // Both are valid reader facts. The first needs a source-shape decision; the second needs
    // a method-body proof. Erasure alone cannot reject either one.
    for (label, bytes) in [
        ("complex method", COMPLEX_METHOD),
        ("incompatible body", INCOMPATIBLE_BODY),
    ] {
        let mut budget = budget();
        let (signature, descriptor, exceptions) = member_signature(bytes, &mut budget);
        if label == "complex method" {
            assert!(matches!(signature.parameters[0], SignatureType::Array(_)));
            assert_eq!(exceptions, vec![b"java/io/IOException".to_vec()]);
            assert!(signature.throws.is_empty());
        }
        prove_method_signature_erasure(&signature, &descriptor, &exceptions, &mut budget)
            .unwrap_or_else(|error| panic!("{label}: {error:?}"));
    }
}
