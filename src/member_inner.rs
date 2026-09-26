//! Physical target proof for a Java 8 non-static member constructor.
//!
//! This module knows only the selected target class bytes. The facade selects those bytes from the
//! request's environment; a method recovery consumes the resulting narrow fact, never a class name
//! inferred from `$` or a constructor's first parameter alone.

use crate::class_source::ClassSourceAssemblyContext;
use jarde_reader::budget::Budget;
use jarde_reader::budget::CountedBudgetDimension;
use jarde_reader::classfile::{
    ClassMemberFacts, CpEntryFacts, CpEntryKind, DescriptorKind, attribute_facts,
    class_constant_pool, cp_class_name, cp_entry, descriptor_facts, method_code_facts,
};

/// A class's own typed row is only a candidate until the selected child's row agrees.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FamilyRootCandidate {
    pub(crate) child_name: Vec<u8>,
    pub(crate) simple_name: String,
    pub(crate) access_flags: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FamilyRootScan {
    Absent,
    Refused(String),
    Candidate(FamilyRootCandidate),
}

const FAMILY_FORBIDDEN_FLAGS: u16 = 0x0008 | 0x0200 | 0x2000 | 0x4000;
const FAMILY_VISIBILITY_FLAGS: u16 = 0x0001 | 0x0002 | 0x0004;

/// Discover at most one direct named non-static child from the root's typed InnerClasses rows.
/// No binary-name search is used: the class index in the row supplies the exact symbolic target.
pub(crate) fn scan_family_root(
    root: &[u8],
    nesting: &ClassSourceAssemblyContext,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<FamilyRootScan> {
    if nesting.enclosing_method.is_some() {
        return Ok(FamilyRootScan::Refused(
            "selected root has EnclosingMethod identity".to_owned(),
        ));
    }
    let mut candidate = None;
    let mut static_names = Vec::new();
    for row in &nesting.inner_classes {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        // This slice writes one top-level source unit. A typed self row with an outer owner
        // proves that the selected root itself belongs to a larger family.
        if row.outer_class_index != 0
            && cp_class_name(pool, row.class_index).is_ok_and(|name| name.0 == root)
        {
            return Ok(FamilyRootScan::Refused(
                "selected root has an InnerClasses self row naming an outer class".to_owned(),
            ));
        }
        if row.outer_class_index == 0 {
            continue;
        }
        let Ok(outer) = cp_class_name(pool, row.outer_class_index) else {
            return Ok(FamilyRootScan::Refused(
                "InnerClasses outer class index is invalid".to_owned(),
            ));
        };
        if outer.0 != root {
            continue;
        }
        // Static nested declarations do not join this captured-member subset. Retain their
        // identities so a contradictory static row for the selected child still rejects it.
        if row.access_flags & 0x0008 != 0 {
            let Ok(name) = cp_class_name(pool, row.class_index) else {
                return Ok(FamilyRootScan::Refused(
                    "static nested class index is invalid".to_owned(),
                ));
            };
            static_names.push(name.0);
            continue;
        }
        let Ok(child) = cp_class_name(pool, row.class_index) else {
            return Ok(FamilyRootScan::Refused(
                "direct member class index is invalid".to_owned(),
            ));
        };
        let Some(simple) = row
            .inner_name
            .as_ref()
            .and_then(|name| std::str::from_utf8(&name.0).ok())
        else {
            return Ok(FamilyRootScan::Refused(
                "direct member has no UTF-8 source name".to_owned(),
            ));
        };
        if !jarde_java::names::is_java_identifier(simple)
            || child.0 != [root, b"$", simple.as_bytes()].concat()
            || row.access_flags & FAMILY_FORBIDDEN_FLAGS != 0
            || (row.access_flags & FAMILY_VISIBILITY_FLAGS).count_ones() > 1
            || row.access_flags & (0x0010 | 0x0400) == (0x0010 | 0x0400)
        {
            return Ok(FamilyRootScan::Refused(
                "direct member is not a source-spellable named non-static class".to_owned(),
            ));
        }
        let next = FamilyRootCandidate {
            child_name: child.0,
            simple_name: simple.to_owned(),
            access_flags: row.access_flags,
        };
        if candidate.replace(next).is_some() {
            return Ok(FamilyRootScan::Refused(
                "multiple direct member rows are outside the one-child family subset".to_owned(),
            ));
        }
    }
    if candidate
        .as_ref()
        .is_some_and(|selected: &FamilyRootCandidate| static_names.contains(&selected.child_name))
    {
        return Ok(FamilyRootScan::Refused(
            "selected member also has a conflicting static InnerClasses row".to_owned(),
        ));
    }
    Ok(candidate.map_or(FamilyRootScan::Absent, FamilyRootScan::Candidate))
}

/// The selected child must independently state the identical unique self relation.
pub(crate) fn child_relation_agrees(
    root: &[u8],
    candidate: &FamilyRootCandidate,
    child: &ClassMemberFacts,
    nesting: &ClassSourceAssemblyContext,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<bool> {
    if child.stopped_at.is_some()
        || child.this_class.raw().0 != candidate.child_name
        || nesting.enclosing_method.is_some()
    {
        return Ok(false);
    }
    let mut self_row = None;
    for row in &nesting.inner_classes {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let Ok(name) = cp_class_name(pool, row.class_index) else {
            return Ok(false);
        };
        if name.0 == candidate.child_name && self_row.replace(row).is_some() {
            return Ok(false);
        }
    }
    let Some(row) = self_row else {
        return Ok(false);
    };
    Ok(row.outer_class_index != 0
        && cp_class_name(pool, row.outer_class_index).is_ok_and(|outer| outer.0 == root)
        && row
            .inner_name
            .as_ref()
            .is_some_and(|name| name.0 == candidate.simple_name.as_bytes())
        && row.access_flags == candidate.access_flags)
}
use jarde_reader::error::{Error, Result};
use jarde_reader::signature::{
    SignatureType, parse_class_signature, parse_method_signature, prove_class_signature_erasure,
    prove_method_signature_erasure_with_class_scope,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MemberInnerTarget {
    pub(crate) owner: String,
    pub(crate) outer: String,
    pub(crate) simple_name: String,
    pub(crate) constructor_descriptor: String,
    pub(crate) capture_field: String,
    pub(crate) generic_diamond: bool,
    pub(crate) source_type_parameter_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GenericOuterRelationProof {
    pub(crate) enclosing: String,
    pub(crate) simple_name: String,
    pub(crate) type_parameter_count: usize,
    /// The selected owner itself is the top-level, non-generic class that declares the target.
    /// This preserves the previously proved `Outer.Inner` source path without treating a `$` in a
    /// binary name as a nested source path.
    pub(crate) top_level_non_generic_owner: bool,
}

fn refusal_or_stop<T>(error: Error) -> Result<Option<T>> {
    match error {
        Error::BudgetExceeded { .. } | Error::Cancelled { .. } => Err(error),
        _ => Ok(None),
    }
}

/// Prove one exact constructor in the target definition selected by the facade. `None` means one
/// structural premise was absent; a request-ending budget/cancellation remains an error.
pub(crate) fn prove_target(
    bytes: &[u8],
    target: &ClassMemberFacts,
    owner: &str,
    descriptor: &str,
    budget: &mut Budget,
) -> Result<Option<MemberInnerTarget>> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_INTERFACE: u16 = 0x0200;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_SYNTHETIC: u16 = 0x1000;
    const ACC_ANNOTATION: u16 = 0x2000;
    const ACC_ENUM: u16 = 0x4000;

    budget.poll()?;
    if target.stopped_at.is_some()
        || target.this_class.raw().0.as_slice() != owner.as_bytes()
        || target.access_flags
            & (ACC_PUBLIC | ACC_INTERFACE | ACC_ABSTRACT | ACC_ANNOTATION | ACC_ENUM)
            != ACC_PUBLIC
    {
        return Ok(None);
    }
    let pool = match class_constant_pool(bytes, budget) {
        Ok(pool) => pool,
        Err(error) => return refusal_or_stop(error),
    };
    let nesting_shells: Vec<_> = target
        .attributes
        .iter()
        .filter(|attribute| {
            matches!(
                attribute.name.raw().0.as_slice(),
                b"InnerClasses" | b"EnclosingMethod"
            )
        })
        .cloned()
        .collect();
    let nesting = match attribute_facts(bytes, &nesting_shells, &pool, budget) {
        Ok(nesting) => nesting,
        Err(error) => return refusal_or_stop(error),
    };
    if nesting.enclosing_method.is_some() {
        return Ok(None);
    }
    let self_entries: Vec<_> = nesting
        .inner_classes
        .iter()
        .filter(|entry| {
            cp_class_name(&pool, entry.class_index).is_ok_and(|name| name.0 == owner.as_bytes())
        })
        .collect();
    let [relation] = self_entries.as_slice() else {
        return Ok(None);
    };
    if relation.outer_class_index == 0
        || relation.access_flags & (ACC_PUBLIC | ACC_STATIC) != ACC_PUBLIC
    {
        return Ok(None);
    }
    let Ok(outer) = cp_class_name(&pool, relation.outer_class_index) else {
        return Ok(None);
    };
    let (Ok(outer), Some(simple_name)) = (
        std::str::from_utf8(&outer.0),
        relation
            .inner_name
            .as_ref()
            .and_then(|name| std::str::from_utf8(&name.0).ok()),
    ) else {
        return Ok(None);
    };
    if !jarde_java::names::is_java_identifier(simple_name)
        || owner != format!("{outer}${simple_name}")
        || descriptor == "()V"
        || !descriptor.starts_with(&format!("(L{outer};"))
        || !descriptor.ends_with(")V")
        || descriptor_facts(descriptor.as_bytes(), DescriptorKind::Method).is_err()
    {
        return Ok(None);
    }

    let class_signature_shells: Vec<_> = target
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Signature")
        .cloned()
        .collect();
    let class_type_annotations_present = target.attributes.iter().any(|attribute| {
        matches!(
            attribute.name.raw().0.as_slice(),
            b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
        )
    });
    let class_attributes = match attribute_facts(bytes, &class_signature_shells, &pool, budget) {
        Ok(attributes) => attributes,
        Err(error) => return refusal_or_stop(error),
    };
    let class_scope = match class_attributes.signature {
        None => None,
        Some(raw_signature) => {
            if class_type_annotations_present {
                return Ok(None);
            }
            let parsed = match parse_class_signature(&raw_signature.0, budget) {
                Ok(signature) => signature,
                Err(error) => return refusal_or_stop(error),
            };
            if parsed.type_parameters.len() != 1
                || parsed.type_parameters[0].name != b"V"
                || parsed.type_parameters[0]
                    .class_bound
                    .as_ref()
                    .is_none_or(|bound| {
                        !matches!(
                            bound,
                            SignatureType::Class(class)
                                if class.segments.len() == 1
                                    && class.segments[0].binary_name == b"java/lang/Object"
                                    && class.segments[0].arguments.is_empty()
                        )
                    })
                || !parsed.type_parameters[0].interface_bounds.is_empty()
                || !parsed.interfaces.is_empty()
                || !target.interfaces.is_empty()
                || parsed.superclass.segments.len() != 1
                || parsed.superclass.segments[0].binary_name != b"java/lang/Object"
                || !parsed.superclass.segments[0].arguments.is_empty()
            {
                return Ok(None);
            }
            let Some(superclass) = target.super_class.as_ref() else {
                return Ok(None);
            };
            let physical_interfaces: Vec<Vec<u8>> = target
                .interfaces
                .iter()
                .map(|interface| interface.raw().0.clone())
                .collect();
            let proof = match prove_class_signature_erasure(
                &parsed,
                &superclass.raw().0,
                &physical_interfaces,
                budget,
            ) {
                Ok(proof) => proof,
                Err(error) => return refusal_or_stop(error),
            };
            if proof.type_parameters.len() != 1
                || proof.type_parameters[0].name != b"V"
                || proof.type_parameters[0].descriptor != b"Ljava/lang/Object;"
            {
                return Ok(None);
            }
            Some(proof.type_parameters)
        }
    };

    let constructors: Vec<_> = target
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0 == b"<init>" && method.descriptor.raw().0 == descriptor.as_bytes()
        })
        .collect();
    let [constructor] = constructors.as_slice() else {
        return Ok(None);
    };
    if constructor.access_flags & ACC_PUBLIC == 0
        || constructor.access_flags & ACC_STATIC != 0
        || constructor
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"Code")
            .count()
            != 1
    {
        return Ok(None);
    }
    let constructor_signatures: Vec<_> = constructor
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Signature")
        .cloned()
        .collect();
    let constructor_type_annotations_present = constructor.attributes.iter().any(|attribute| {
        matches!(
            attribute.name.raw().0.as_slice(),
            b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
        )
    });
    let constructor_attributes =
        match attribute_facts(bytes, &constructor_signatures, &pool, budget) {
            Ok(attributes) => attributes,
            Err(error) => return refusal_or_stop(error),
        };
    let outer_descriptor = format!("L{outer};");
    let generic_diamond = match (class_scope.as_deref(), constructor_attributes.signature) {
        (None, None) => false,
        (Some(class_scope), Some(raw_signature)) => {
            let all_constructors: Vec<_> = target
                .methods
                .iter()
                .filter(|method| method.name.raw().0 == b"<init>")
                .collect();
            if all_constructors.len() != 1 || constructor_type_annotations_present {
                return Ok(None);
            }
            let parsed = match parse_method_signature(&raw_signature.0, budget) {
                Ok(signature) => signature,
                Err(error) => return refusal_or_stop(error),
            };
            if !parsed.type_parameters.is_empty()
                || parsed.parameters.as_slice() != [SignatureType::TypeVariable(b"V".to_vec())]
                || parsed.result.is_some()
                || !parsed.throws.is_empty()
            {
                return Ok(None);
            }
            let physical = match descriptor_facts(descriptor.as_bytes(), DescriptorKind::Method) {
                Ok(physical) => physical,
                Err(_) => return Ok(None),
            };
            let Some(first) = physical.parameters().first() else {
                return Ok(None);
            };
            if first.bytes(descriptor.as_bytes()) != Some(outer_descriptor.as_bytes()) {
                return Ok(None);
            }
            let mut source_descriptor = b"(".to_vec();
            for component in &physical.parameters()[1..] {
                let Some(bytes) = component.bytes(descriptor.as_bytes()) else {
                    return Ok(None);
                };
                source_descriptor.extend_from_slice(bytes);
            }
            source_descriptor.extend_from_slice(b")V");
            let erasure = match prove_method_signature_erasure_with_class_scope(
                &parsed,
                &source_descriptor,
                &[],
                class_scope,
                budget,
            ) {
                Ok(erasure) => erasure,
                Err(error) => return refusal_or_stop(error),
            };
            if erasure.parameters.as_slice() != [b"Ljava/lang/Object;".to_vec()]
                || erasure.result.is_some()
            {
                return Ok(None);
            }
            true
        }
        _ => return Ok(None),
    };
    let source_type_parameter_count = class_scope.as_ref().map_or(0, Vec::len);
    let captures: Vec<_> = target
        .fields
        .iter()
        .filter(|field| {
            field.access_flags & ACC_SYNTHETIC != 0 && field.access_flags & ACC_STATIC == 0
        })
        .collect();
    let [capture] = captures.as_slice() else {
        return Ok(None);
    };
    if capture.descriptor.raw().0 != outer_descriptor.as_bytes() {
        return Ok(None);
    }
    let code = match method_code_facts(bytes, constructor, budget) {
        Ok(code) => code,
        Err(error) => return refusal_or_stop(error),
    };
    if code.stopped_at.is_some() || code.instructions.len() < 3 {
        return Ok(None);
    }
    let prologue = &code.instructions[..3];
    if prologue[0].bci != 0
        || prologue[0].opcode != 0x2a // aload_0: uninitialized this
        || prologue[1].bci != 1
        || prologue[1].opcode != 0x2b // aload_1: physical first parameter
        || prologue[2].bci != 2
        || prologue[2].opcode != 0xb5 // putfield: capture on this
        || prologue[2].constant_pool_index.is_none()
    {
        return Ok(None);
    }
    let field_index = prologue[2].constant_pool_index.expect("checked above");
    let Ok(entry) = cp_entry(&pool, field_index) else {
        return Ok(None);
    };
    if !matches!(
        &entry.kind,
        CpEntryKind::FieldRef { owner: field_owner, name, descriptor: field_descriptor, .. }
            if field_owner.0 == owner.as_bytes()
                && name.0 == capture.name.raw().0
                && field_descriptor.0 == outer_descriptor.as_bytes()
    ) {
        return Ok(None);
    }
    let Ok(capture_field) = std::str::from_utf8(&capture.name.raw().0) else {
        return Ok(None);
    };
    Ok(Some(MemberInnerTarget {
        owner: owner.to_owned(),
        outer: outer.to_owned(),
        simple_name: simple_name.to_owned(),
        constructor_descriptor: descriptor.to_owned(),
        capture_field: capture_field.to_owned(),
        generic_diamond,
        source_type_parameter_count,
    }))
}

/// A qualified Java creation must also resolve `Inner` through the selected outer class. The
/// target's self row alone does not make that member visible to `javac`: the outer's matching
/// InnerClasses row is a separate physical premise.
pub(crate) fn outer_relation_agrees(
    bytes: &[u8],
    outer_class: &ClassMemberFacts,
    target: &MemberInnerTarget,
    budget: &mut Budget,
) -> Result<Option<GenericOuterRelationProof>> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_PROTECTED: u16 = 0x0004;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_INTERFACE: u16 = 0x0200;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_ANNOTATION: u16 = 0x2000;
    const ACC_ENUM: u16 = 0x4000;

    budget.poll()?;
    if outer_class.stopped_at.is_some()
        || outer_class.this_class.raw().0 != target.outer.as_bytes()
        || outer_class.access_flags
            & (ACC_PUBLIC | ACC_INTERFACE | ACC_ABSTRACT | ACC_ANNOTATION | ACC_ENUM)
            != ACC_PUBLIC
    {
        return Ok(None);
    }
    let pool = match class_constant_pool(bytes, budget) {
        Ok(pool) => pool,
        Err(error) => return refusal_or_stop(error),
    };
    let shells: Vec<_> = outer_class
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"InnerClasses")
        .cloned()
        .collect();
    let mut nesting_shells: Vec<_> = shells;
    nesting_shells.extend(
        outer_class
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"EnclosingMethod")
            .cloned(),
    );
    let nesting = match attribute_facts(bytes, &nesting_shells, &pool, budget) {
        Ok(nesting) => nesting,
        Err(error) => return refusal_or_stop(error),
    };
    if nesting.enclosing_method.is_some() {
        return Ok(None);
    }

    let target_rows: Vec<_> = nesting
        .inner_classes
        .iter()
        .filter(|entry| {
            cp_class_name(&pool, entry.class_index)
                .is_ok_and(|name| name.0 == target.owner.as_bytes())
        })
        .collect();
    let [target_row] = target_rows.as_slice() else {
        return Ok(None);
    };
    if target_row.outer_class_index == 0
        || target_row.access_flags & (ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED | ACC_STATIC)
            != ACC_PUBLIC
        || !cp_class_name(&pool, target_row.outer_class_index)
            .is_ok_and(|name| name.0 == target.outer.as_bytes())
        || !target_row
            .inner_name
            .as_ref()
            .is_some_and(|name| name.0 == target.simple_name.as_bytes())
    {
        return Ok(None);
    }

    // `A<T>` is a public static member of its selected `Outer`. Its own row supplies the candidate
    // name; the reciprocal row on the selected `Outer` is checked by the facade below.
    let self_rows: Vec<_> = nesting
        .inner_classes
        .iter()
        .filter(|entry| {
            cp_class_name(&pool, entry.class_index)
                .is_ok_and(|name| name.0 == target.outer.as_bytes())
        })
        .collect();
    if self_rows.is_empty() {
        // Keep the accepted non-generic `Outer.Inner` slice. A binary `$` by itself cannot prove
        // that the selected owner is a top-level source declaration, so this branch requires a
        // simple top-level binary name, no class Signature, and the exact reciprocal member row
        // already checked above.
        let owner_simple_name = target.outer.rsplit('/').next().unwrap_or_default();
        if owner_simple_name.is_empty()
            || owner_simple_name.contains('$')
            || !jarde_java::names::is_java_identifier(owner_simple_name)
            || outer_class
                .attributes
                .iter()
                .any(|attribute| attribute.name.raw().0 == b"Signature")
        {
            return Ok(None);
        }
        return Ok(Some(GenericOuterRelationProof {
            enclosing: target.outer.clone(),
            simple_name: String::new(),
            type_parameter_count: 0,
            top_level_non_generic_owner: true,
        }));
    }
    let [self_row] = self_rows.as_slice() else {
        return Ok(None);
    };
    if self_row.outer_class_index == 0
        || self_row.access_flags & (ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED | ACC_STATIC)
            != (ACC_PUBLIC | ACC_STATIC)
    {
        return Ok(None);
    }
    let Ok(enclosing_class) = cp_class_name(&pool, self_row.outer_class_index) else {
        return Ok(None);
    };
    let (Ok(enclosing), Some(simple_name)) = (
        String::from_utf8(enclosing_class.0),
        self_row
            .inner_name
            .as_ref()
            .and_then(|name| std::str::from_utf8(&name.0).ok()),
    ) else {
        return Ok(None);
    };
    if !jarde_java::names::is_java_identifier(simple_name)
        || target.outer != format!("{enclosing}${simple_name}")
    {
        return Ok(None);
    }

    let signatures: Vec<_> = outer_class
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Signature")
        .cloned()
        .collect();
    if signatures.len() != 1
        || outer_class.attributes.iter().any(|attribute| {
            matches!(
                attribute.name.raw().0.as_slice(),
                b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
            )
        })
    {
        return Ok(None);
    }
    let signature = match attribute_facts(bytes, &signatures, &pool, budget) {
        Ok(attributes) => attributes.signature,
        Err(error) => return refusal_or_stop(error),
    };
    let Some(signature) = signature else {
        return Ok(None);
    };
    let parsed = match parse_class_signature(&signature.0, budget) {
        Ok(signature) => signature,
        Err(error) => return refusal_or_stop(error),
    };
    if parsed.type_parameters.len() != 1
        || parsed.type_parameters[0].name != b"T"
        || !matches!(
            parsed.type_parameters[0].class_bound.as_ref(),
            Some(SignatureType::Class(class))
                if class.segments.len() == 1
                    && class.segments[0].binary_name == b"java/lang/Object"
                    && class.segments[0].arguments.is_empty()
        )
        || !parsed.type_parameters[0].interface_bounds.is_empty()
        || !parsed.interfaces.is_empty()
        || !outer_class.interfaces.is_empty()
        || parsed.superclass.segments.len() != 1
        || parsed.superclass.segments[0].binary_name != b"java/lang/Object"
        || !parsed.superclass.segments[0].arguments.is_empty()
    {
        return Ok(None);
    }
    let Some(superclass) = outer_class.super_class.as_ref() else {
        return Ok(None);
    };
    let physical_interfaces: Vec<Vec<u8>> = outer_class
        .interfaces
        .iter()
        .map(|interface| interface.raw().0.clone())
        .collect();
    let proof = match prove_class_signature_erasure(
        &parsed,
        &superclass.raw().0,
        &physical_interfaces,
        budget,
    ) {
        Ok(proof) => proof,
        Err(error) => return refusal_or_stop(error),
    };
    if proof.type_parameters.len() != 1
        || proof.type_parameters[0].name != b"T"
        || proof.type_parameters[0].descriptor != b"Ljava/lang/Object;"
    {
        return Ok(None);
    }
    Ok(Some(GenericOuterRelationProof {
        enclosing,
        simple_name: simple_name.to_owned(),
        type_parameter_count: 1,
        top_level_non_generic_owner: false,
    }))
}

/// Prove the reciprocal row on `Outer` needed to spell the selected `Outer.A<T>` source path.
/// This first slice accepts one non-generic top-level enclosing class; another member edge needs a
/// separate path proof.
pub(crate) fn enclosing_relation_agrees(
    bytes: &[u8],
    enclosing_class: &ClassMemberFacts,
    generic_outer: &GenericOuterRelationProof,
    budget: &mut Budget,
) -> Result<bool> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_PROTECTED: u16 = 0x0004;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_INTERFACE: u16 = 0x0200;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_ANNOTATION: u16 = 0x2000;
    const ACC_ENUM: u16 = 0x4000;

    budget.poll()?;
    if enclosing_class.stopped_at.is_some()
        || enclosing_class.this_class.raw().0 != generic_outer.enclosing.as_bytes()
        || enclosing_class.access_flags
            & (ACC_PUBLIC | ACC_INTERFACE | ACC_ABSTRACT | ACC_ANNOTATION | ACC_ENUM)
            != ACC_PUBLIC
        || enclosing_class
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Signature")
    {
        return Ok(false);
    }
    let pool = match class_constant_pool(bytes, budget) {
        Ok(pool) => pool,
        Err(error) => return refusal_or_stop(error).map(|_: Option<()>| false),
    };
    let shells: Vec<_> = enclosing_class
        .attributes
        .iter()
        .filter(|attribute| {
            matches!(
                attribute.name.raw().0.as_slice(),
                b"InnerClasses" | b"EnclosingMethod"
            )
        })
        .cloned()
        .collect();
    let nesting = match attribute_facts(bytes, &shells, &pool, budget) {
        Ok(nesting) => nesting,
        Err(error) => return refusal_or_stop(error).map(|_: Option<()>| false),
    };
    if nesting.enclosing_method.is_some()
        || nesting.inner_classes.iter().any(|entry| {
            cp_class_name(&pool, entry.class_index)
                .is_ok_and(|name| name.0 == generic_outer.enclosing.as_bytes())
        })
    {
        return Ok(false);
    }
    if generic_outer.top_level_non_generic_owner {
        let owner_simple_name = generic_outer
            .enclosing
            .rsplit('/')
            .next()
            .unwrap_or_default();
        return Ok(generic_outer.simple_name.is_empty()
            && generic_outer.type_parameter_count == 0
            && !owner_simple_name.is_empty()
            && !owner_simple_name.contains('$')
            && jarde_java::names::is_java_identifier(owner_simple_name));
    }
    let member = format!("{}${}", generic_outer.enclosing, generic_outer.simple_name);
    let rows: Vec<_> = nesting
        .inner_classes
        .iter()
        .filter(|entry| {
            cp_class_name(&pool, entry.class_index).is_ok_and(|name| name.0 == member.as_bytes())
        })
        .collect();
    let [row] = rows.as_slice() else {
        return Ok(false);
    };
    Ok(row.outer_class_index != 0
        && row.access_flags & (ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED | ACC_STATIC)
            == (ACC_PUBLIC | ACC_STATIC)
        && cp_class_name(&pool, row.outer_class_index)
            .is_ok_and(|name| name.0 == generic_outer.enclosing.as_bytes())
        && row
            .inner_name
            .as_ref()
            .is_some_and(|name| name.0 == generic_outer.simple_name.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarde_reader::budget::{CancellationToken, Limits};
    use jarde_reader::classfile::{CpEntryKind, class_constant_pool, class_member_facts};
    use std::io::Read;

    const INNER: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/classes/nested/SimpleOuter$Inner.class"
    );
    const OWNER: &str = "nested/SimpleOuter$Inner";
    const DESCRIPTOR: &str = "(Lnested/SimpleOuter;I)V";
    const GENERIC_JAR: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/original-outer-g-none.jar"
    );
    const GENERIC_OWNER: &str = "minimal/Outer$Inner";
    const GENERIC_DESCRIPTOR: &str = "(Lminimal/Outer;Ljava/lang/Object;)V";
    const FAMILY_JAR: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-26/named-member-family-stage1/fixture.jar"
    );

    fn family_bytes(path: &[u8]) -> Vec<u8> {
        let archive = rawzip::ZipArchive::from_slice(FAMILY_JAR).unwrap();
        let mut entries = archive.entries();
        while let Some(header) = entries.next_entry().unwrap() {
            if header.file_path().as_ref() == path {
                let entry = archive.get_entry(header.wayfinder()).unwrap();
                let decoder = flate2::bufread::DeflateDecoder::new(entry.data());
                let mut reader = entry.verifying_reader(decoder);
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).unwrap();
                return bytes;
            }
        }
        panic!("frozen family jar contains requested class")
    }

    fn family_facts(
        bytes: &[u8],
        budget: &mut Budget,
    ) -> (
        ClassMemberFacts,
        Vec<CpEntryFacts>,
        ClassSourceAssemblyContext,
    ) {
        let facts = class_member_facts(bytes, budget).unwrap();
        let pool = class_constant_pool(bytes, budget).unwrap();
        let shells: Vec<_> = facts
            .attributes
            .iter()
            .filter(|attribute| {
                matches!(
                    attribute.name.raw().0.as_slice(),
                    b"InnerClasses" | b"EnclosingMethod"
                )
            })
            .cloned()
            .collect();
        let nesting =
            crate::class_source::read_class_source_assembly_context(bytes, &shells, &pool, budget)
                .unwrap();
        (facts, pool, nesting)
    }

    #[test]
    fn family_relation_needs_matching_typed_rows_and_uses_member_flags() {
        let root_bytes = family_bytes(b"NamedMemberFamilyStage1.class");
        let child_bytes = family_bytes(b"NamedMemberFamilyStage1$Member.class");
        let mut budget = budget();
        let (_, root_pool, root_nesting) = family_facts(&root_bytes, &mut budget);
        let (child_facts, child_pool, child_nesting) = family_facts(&child_bytes, &mut budget);
        let FamilyRootScan::Candidate(candidate) = scan_family_root(
            b"NamedMemberFamilyStage1",
            &root_nesting,
            &root_pool,
            &mut budget,
        )
        .unwrap() else {
            panic!("root typed row names one child")
        };
        assert_eq!(candidate.simple_name, "Member");
        assert_eq!(candidate.access_flags & FAMILY_VISIBILITY_FLAGS, 0);
        assert_eq!(child_facts.access_flags & 0x0001, 0);
        assert!(
            child_relation_agrees(
                b"NamedMemberFamilyStage1",
                &candidate,
                &child_facts,
                &child_nesting,
                &child_pool,
                &mut budget,
            )
            .unwrap()
        );

        let mut no_root_row = root_nesting.clone();
        no_root_row.inner_classes.clear();
        assert_eq!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &no_root_row,
                &root_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Absent
        );
        let mut duplicate_root = root_nesting.clone();
        duplicate_root
            .inner_classes
            .push(root_nesting.inner_classes[0].clone());
        assert!(matches!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &duplicate_root,
                &root_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Refused(_)
        ));
        let mut static_only = root_nesting.clone();
        static_only.inner_classes[0].access_flags |= 0x0008;
        assert_eq!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &static_only,
                &root_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Absent
        );
        let mut static_sibling = root_nesting.clone();
        let mut static_row = root_nesting.inner_classes[0].clone();
        static_row.access_flags |= 0x0008;
        static_sibling.inner_classes.push(static_row);
        assert!(matches!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &static_sibling,
                &root_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Refused(_)
        ));
        let mut sibling_pool = root_pool.clone();
        let mut sibling_class = sibling_pool
            .iter()
            .find(|entry| entry.index == root_nesting.inner_classes[0].class_index)
            .unwrap()
            .clone();
        sibling_class.index = sibling_pool.iter().map(|entry| entry.index).max().unwrap() + 1;
        let CpEntryKind::Class { name, .. } = &mut sibling_class.kind else {
            unreachable!("InnerClasses class index names a Class entry")
        };
        name.0 = b"NamedMemberFamilyStage1$Static".to_vec();
        let sibling_index = sibling_class.index;
        sibling_pool.push(sibling_class);
        static_sibling.inner_classes.last_mut().unwrap().class_index = sibling_index;
        assert!(matches!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &static_sibling,
                &sibling_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Candidate(found) if found == candidate
        ));
        let mut dollar_pool = root_pool.clone();
        let child_entry = dollar_pool
            .iter_mut()
            .find(|entry| entry.index == root_nesting.inner_classes[0].class_index)
            .unwrap();
        let CpEntryKind::Class { name, .. } = &mut child_entry.kind else {
            unreachable!("InnerClasses class index names a Class entry")
        };
        name.0 = b"NamedMemberFamilyStage1$$A".to_vec();
        let mut dollar_row = root_nesting.clone();
        dollar_row.inner_classes[0].inner_name =
            Some(jarde_reader::model::JvmBytes(b"$A".to_vec()));
        assert!(matches!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &dollar_row,
                &dollar_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Candidate(found) if found.simple_name == "$A"
        ));
        let root_class_index = root_pool
            .iter()
            .find_map(|entry| match &entry.kind {
                CpEntryKind::Class { name, .. } if name.0 == b"NamedMemberFamilyStage1" => {
                    Some(entry.index)
                }
                _ => None,
            })
            .unwrap();
        let mut nested_root = root_nesting.clone();
        let mut self_row = root_nesting.inner_classes[0].clone();
        self_row.class_index = root_class_index;
        nested_root.inner_classes.push(self_row);
        assert!(matches!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &nested_root,
                &root_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Refused(_)
        ));

        let mut missing_child_row = child_nesting.clone();
        missing_child_row.inner_classes.clear();
        assert!(
            !child_relation_agrees(
                b"NamedMemberFamilyStage1",
                &candidate,
                &child_facts,
                &missing_child_row,
                &child_pool,
                &mut budget,
            )
            .unwrap()
        );
        let mut wrong_flags = child_nesting.clone();
        wrong_flags.inner_classes[0].access_flags |= 0x0001;
        assert!(
            !child_relation_agrees(
                b"NamedMemberFamilyStage1",
                &candidate,
                &child_facts,
                &wrong_flags,
                &child_pool,
                &mut budget,
            )
            .unwrap()
        );
        let mut local = child_nesting.clone();
        local.enclosing_method = Some(jarde_reader::classfile::EnclosingMethodFacts {
            class_index: 1,
            method_index: 1,
        });
        assert!(
            !child_relation_agrees(
                b"NamedMemberFamilyStage1",
                &candidate,
                &child_facts,
                &local,
                &child_pool,
                &mut budget,
            )
            .unwrap()
        );
    }

    #[test]
    fn family_row_scan_observes_real_budget_and_cancellation() {
        let bytes = family_bytes(b"NamedMemberFamilyStage1.class");
        let mut setup = budget();
        let (_, pool, nesting) = family_facts(&bytes, &mut setup);
        let mut limited = Budget::new(Limits {
            analysis_steps: 0,
            ..Limits::default()
        });
        assert!(matches!(
            scan_family_root(b"NamedMemberFamilyStage1", &nesting, &pool, &mut limited),
            Err(Error::BudgetExceeded { .. })
        ));
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let mut cancelled = Budget::with_cancellation_token(Limits::default(), cancellation);
        assert!(matches!(
            scan_family_root(b"NamedMemberFamilyStage1", &nesting, &pool, &mut cancelled),
            Err(Error::Cancelled { .. })
        ));
    }

    fn budget() -> Budget {
        Budget::new(Limits {
            input_bytes: u64::MAX,
            archive_entries: u64::MAX,
            entry_bytes: u64::MAX,
            read_bytes: u64::MAX,
            class_bytes: u64::MAX,
            attribute_bytes: u64::MAX,
            code_bytes: u64::MAX,
            result_items: u64::MAX,
            output_bytes: u64::MAX,
            class_headers: u64::MAX,
            method_bodies: u64::MAX,
            ir_items: u64::MAX,
            ir_edges: u64::MAX,
            analysis_steps: u64::MAX,
            normalization_clones: u64::MAX,
            nested_depth: u64::MAX,
            dependency_depth: u64::MAX,
            elapsed_millis: u64::MAX,
        })
    }

    fn generic_inner_bytes() -> Vec<u8> {
        let archive = rawzip::ZipArchive::from_slice(GENERIC_JAR).expect("frozen jar parses");
        let mut entries = archive.entries();
        while let Some(header) = entries.next_entry().expect("frozen jar directory parses") {
            if header.file_path().as_ref() == b"minimal/Outer$Inner.class" {
                let entry = archive
                    .get_entry(header.wayfinder())
                    .expect("frozen inner entry parses");
                let decoder = flate2::bufread::DeflateDecoder::new(entry.data());
                let mut reader = entry.verifying_reader(decoder);
                let mut bytes = Vec::new();
                reader
                    .read_to_end(&mut bytes)
                    .expect("frozen inner entry inflates and verifies");
                return bytes;
            }
        }
        panic!("frozen jar contains the selected Inner class")
    }

    fn replace_utf8(bytes: &[u8], old: &[u8], new: &[u8]) -> Vec<u8> {
        let mut unlimited = budget();
        let pool = class_constant_pool(bytes, &mut unlimited).expect("constant pool reads");
        let entry = pool
            .iter()
            .find(|entry| matches!(&entry.kind, CpEntryKind::Utf8 { bytes } if bytes.0 == old))
            .expect("fixture contains signature text");
        let start = usize::try_from(entry.span.start).expect("span start fits usize");
        assert_eq!(bytes.get(start), Some(&1), "entry is CONSTANT_Utf8");
        let length_start = start + 1;
        let payload_start = start + 3;
        let payload_end = payload_start + old.len();
        let mut changed = bytes.to_vec();
        changed.splice(payload_start..payload_end, new.iter().copied());
        changed[length_start..payload_start]
            .copy_from_slice(&u16::try_from(new.len()).unwrap().to_be_bytes());
        changed
    }

    #[test]
    fn exact_target_relation_field_and_prologue_are_required() {
        let mut budget = budget();
        let facts = class_member_facts(INNER, &mut budget).unwrap();
        let proof = prove_target(INNER, &facts, OWNER, DESCRIPTOR, &mut budget)
            .unwrap()
            .expect("frozen Java 8 member target proves");
        assert_eq!(proof.outer, "nested/SimpleOuter");
        assert_eq!(proof.simple_name, "Inner");
        assert_eq!(proof.capture_field, "this$0");

        assert!(
            prove_target(INNER, &facts, OWNER, "(Lnested/SimpleOuter;)V", &mut budget)
                .unwrap()
                .is_none()
        );
        let mut no_capture = facts.clone();
        no_capture
            .fields
            .iter_mut()
            .find(|field| field.name.raw().0 == b"this$0")
            .unwrap()
            .access_flags &= !0x1000;
        assert!(
            prove_target(INNER, &no_capture, OWNER, DESCRIPTOR, &mut budget)
                .unwrap()
                .is_none()
        );
        let mut ambiguous_capture = facts.clone();
        let capture = ambiguous_capture
            .fields
            .iter()
            .find(|field| field.name.raw().0 == b"this$0")
            .unwrap()
            .clone();
        ambiguous_capture.fields.push(capture);
        assert!(
            prove_target(INNER, &ambiguous_capture, OWNER, DESCRIPTOR, &mut budget)
                .unwrap()
                .is_none()
        );

        let mut wrong_prologue = INNER.to_vec();
        let constructor = facts
            .methods
            .iter()
            .find(|method| method.name.raw().0 == b"<init>")
            .unwrap();
        let code = method_code_facts(INNER, constructor, &mut budget).unwrap();
        let first_parameter_load = usize::try_from(code.code_span.start).unwrap() + 1;
        wrong_prologue[first_parameter_load] = 0x2c; // aload_2, not the physical outer parameter
        assert!(
            prove_target(&wrong_prologue, &facts, OWNER, DESCRIPTOR, &mut budget)
                .unwrap()
                .is_none()
        );

        let mut wrong_relation = INNER.to_vec();
        let inner_shell = facts
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"InnerClasses")
            .unwrap();
        let outer_index = usize::try_from(inner_shell.content_span.start).unwrap() + 4;
        wrong_relation[outer_index..outer_index + 2].copy_from_slice(&0_u16.to_be_bytes());
        let wrong_facts = class_member_facts(&wrong_relation, &mut budget).unwrap();
        assert!(
            prove_target(
                &wrong_relation,
                &wrong_facts,
                OWNER,
                DESCRIPTOR,
                &mut budget
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn generic_member_signature_and_constructor_tail_are_proved() {
        let bytes = generic_inner_bytes();
        let mut budget = budget();
        let facts = class_member_facts(&bytes, &mut budget).unwrap();
        let proof = prove_target(
            &bytes,
            &facts,
            GENERIC_OWNER,
            GENERIC_DESCRIPTOR,
            &mut budget,
        )
        .unwrap()
        .expect("frozen generic member target proves");
        assert!(proof.generic_diamond);
        assert_eq!(proof.outer, "minimal/Outer");
        assert_eq!(proof.simple_name, "Inner");

        let wrong_class_variable = replace_utf8(
            &bytes,
            b"<V:Ljava/lang/Object;>Ljava/lang/Object;",
            b"<T:Ljava/lang/Object;>Ljava/lang/Object;",
        );
        let wrong_class_facts = class_member_facts(&wrong_class_variable, &mut budget).unwrap();
        assert!(
            prove_target(
                &wrong_class_variable,
                &wrong_class_facts,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        for wrong_class_signature in [
            b"<V:Ljava/lang/Number;>Ljava/lang/Object;".as_slice(),
            b"<V:TT;>Ljava/lang/Object;".as_slice(),
        ] {
            let malformed_scope = replace_utf8(
                &bytes,
                b"<V:Ljava/lang/Object;>Ljava/lang/Object;",
                wrong_class_signature,
            );
            let malformed_facts = class_member_facts(&malformed_scope, &mut budget).unwrap();
            assert!(
                prove_target(
                    &malformed_scope,
                    &malformed_facts,
                    GENERIC_OWNER,
                    GENERIC_DESCRIPTOR,
                    &mut budget,
                )
                .unwrap()
                .is_none()
            );
        }

        let wrong_constructor_erasure = replace_utf8(&bytes, b"(TV;)V", b"(Ljava/lang/String;)V");
        let wrong_constructor_facts =
            class_member_facts(&wrong_constructor_erasure, &mut budget).unwrap();
        assert!(
            prove_target(
                &wrong_constructor_erasure,
                &wrong_constructor_facts,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        let mut missing_class_signature = facts.clone();
        missing_class_signature
            .attributes
            .retain(|attribute| attribute.name.raw().0 != b"Signature");
        assert!(
            prove_target(
                &bytes,
                &missing_class_signature,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        let mut duplicate_class_signature = facts.clone();
        let signature_shell = duplicate_class_signature
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"Signature")
            .unwrap()
            .clone();
        duplicate_class_signature.attributes.push(signature_shell);
        assert!(
            prove_target(
                &bytes,
                &duplicate_class_signature,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        let constructor_index = facts
            .methods
            .iter()
            .position(|method| method.name.raw().0 == b"<init>")
            .unwrap();
        let mut missing_constructor_signature = facts.clone();
        missing_constructor_signature.methods[constructor_index]
            .attributes
            .retain(|attribute| attribute.name.raw().0 != b"Signature");
        assert!(
            prove_target(
                &bytes,
                &missing_constructor_signature,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        let mut duplicate_constructor_signature = facts.clone();
        let constructor_signature_shell = duplicate_constructor_signature.methods
            [constructor_index]
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"Signature")
            .unwrap()
            .clone();
        duplicate_constructor_signature.methods[constructor_index]
            .attributes
            .push(constructor_signature_shell);
        assert!(
            prove_target(
                &bytes,
                &duplicate_constructor_signature,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        let mut overloaded = facts.clone();
        overloaded
            .methods
            .push(overloaded.methods[constructor_index].clone());
        assert!(
            prove_target(
                &bytes,
                &overloaded,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn generic_signature_proof_preserves_budget_and_cancellation_stops() {
        let bytes = generic_inner_bytes();
        let mut setup_budget = budget();
        let facts = class_member_facts(&bytes, &mut setup_budget).unwrap();

        let mut limited = Budget::new(Limits {
            class_bytes: 0,
            ..Limits::default()
        });
        assert!(matches!(
            prove_target(
                &bytes,
                &facts,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut limited,
            ),
            Err(Error::BudgetExceeded { .. })
        ));

        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let mut cancelled = Budget::with_cancellation_token(
            Limits {
                class_bytes: u64::MAX,
                ..Limits::default()
            },
            cancellation,
        );
        assert!(matches!(
            prove_target(
                &bytes,
                &facts,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut cancelled,
            ),
            Err(Error::Cancelled { .. })
        ));

        let mut plain_budget = budget();
        let plain_facts = class_member_facts(INNER, &mut plain_budget).unwrap();
        let plain = prove_target(INNER, &plain_facts, OWNER, DESCRIPTOR, &mut plain_budget)
            .unwrap()
            .expect("existing non-generic member proof remains available");
        assert!(!plain.generic_diamond);
    }
}
