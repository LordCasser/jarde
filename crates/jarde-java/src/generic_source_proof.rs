//! Java source-site, generic substitution and overload proofs over supplied facts.
//! This module performs no artifact read, definition selection or class transaction.
#![allow(clippy::too_many_arguments)]

use crate as jarde_java;
use crate::declaration::method_descriptor;
use crate::generic_source_proof as class_source;
use crate::report::{GenericReturnCandidate, GenericReturnValue, TypedFunctionalKind};
use crate::source_proof::{
    enum_projection_stop_error, generic_call_reference_type_from_descriptor,
    generic_list_member_contract,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{
    ClassMemberFacts, DescriptorKind, attribute_facts, class_constant_pool, descriptor_facts,
};
use jarde_reader::signature::{
    ClassSignature, ClassSignatureErasureProof, SignatureType, TypeArgument, TypeParameter,
    TypeParameterErasure, parse_method_signature,
};
use jarde_reader::{
    classfile::{InstructionFact, MemberHeader},
    error::{Error, Result},
    model::{JvmBytes, JvmString, PhysicalDefinitionId, PhysicalMethodId},
};

use crate::facts::{
    ACC_ABSTRACT, ACC_ANNOTATION, ACC_INTERFACE, ACC_PRIVATE, ACC_PUBLIC, ACC_STATIC,
};
const ACC_NATIVE: u16 = 0x0100;
const ACC_PROTECTED: u16 = 0x0004;
const ACC_FINAL: u16 = 0x0010;
const ACC_SYNCHRONIZED: u16 = 0x0020;
const ACC_STRICT: u16 = 0x0800;
const ACC_ENUM: u16 = 0x4000;
use crate::{
    is_java_identifier,
    names::{is_source_identifier, source_identifier, source_type_parameter_name},
};
// JVMS 4.6-A: variable arity method parameter.
const ACC_VARARGS: u16 = 0x0080;

pub fn spell_ordinary_signature_type(
    ty: &SignatureType,
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
    depth: usize,
) -> Result<String> {
    spell_ordinary_signature_type_with_member_path(ty, class_scope, &[], budget, depth)
}

/// The class-header spelling of one `Signature` type — the `extends`/`implements` entries
/// `ClassSourceDeclaration::project_generic_signature` publishes. It is the same recursion as
/// the member and annotation positions, with one difference: a single-segment class name is
/// spelled by [`binary_pool_class_name`], the class file's own name with `$` kept, because that
/// is the name the raw header already states for every parent no `Signature` projects. A
/// projected entry therefore adds its type arguments and changes no name.
pub fn spell_class_header_signature_type(
    ty: &SignatureType,
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
    depth: usize,
) -> Result<String> {
    spell_signature_type_with_spelling(
        ty,
        class_scope,
        &[],
        ClassNameSpelling::BinaryPoolName,
        budget,
        depth,
    )
}

/// How one class type's binary name becomes text.
#[derive(Clone, Copy, Eq, PartialEq)]
enum ClassNameSpelling {
    /// A member, annotation, `throws` or type-parameter position: a single-segment name is
    /// spelled only when a member proof selected its source path
    /// ([`simple_generic_class_name`]), so a `$` name without that proof has no spelling here.
    SelectedSourcePath,
    /// The class header's own `extends`/`implements` entries ([`binary_pool_class_name`]).
    BinaryPoolName,
}

pub fn spell_ordinary_signature_type_with_member_path(
    ty: &SignatureType,
    class_scope: &[TypeParameterErasure],
    source_type_path: &[jarde_java::report::MemberInnerSourceSegmentInput],
    budget: &mut Budget,
    depth: usize,
) -> Result<String> {
    spell_signature_type_with_spelling(
        ty,
        class_scope,
        source_type_path,
        ClassNameSpelling::SelectedSourcePath,
        budget,
        depth,
    )
}

fn spell_signature_type_with_spelling(
    ty: &SignatureType,
    class_scope: &[TypeParameterErasure],
    source_type_path: &[jarde_java::report::MemberInnerSourceSegmentInput],
    class_names: ClassNameSpelling,
    budget: &mut Budget,
    depth: usize,
) -> Result<String> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if depth > 128 {
        return Err(Error::unsupported(
            "ordinary_generic_source_unproved",
            "type nesting exceeds source projection depth",
        ));
    }
    let nested = depth + 1;
    match ty {
        SignatureType::Base(code) => Ok(match code {
            b'B' => "byte",
            b'C' => "char",
            b'D' => "double",
            b'F' => "float",
            b'I' => "int",
            b'J' => "long",
            b'S' => "short",
            b'Z' => "boolean",
            _ => {
                return Err(Error::unsupported(
                    "ordinary_generic_source_unproved",
                    "unknown primitive signature type",
                ));
            }
        }
        .to_owned()),
        SignatureType::Array(element) => Ok(format!(
            "{}[]",
            spell_signature_type_with_spelling(
                element,
                class_scope,
                source_type_path,
                class_names,
                budget,
                nested,
            )?
        )),
        SignatureType::TypeVariable(name) => class_scope
            .iter()
            .find(|variable| variable.name == *name)
            .and_then(|variable| source_type_parameter_name(&variable.name))
            .ok_or_else(|| {
                Error::unsupported(
                    "ordinary_generic_source_unproved",
                    "type variable is not present in the emitted class header",
                )
            }),
        SignatureType::Class(class) => {
            if class.segments.is_empty() {
                return Err(Error::unsupported(
                    "ordinary_generic_source_unproved",
                    "nested binary class path has no unique Java source spelling",
                ));
            }
            let mut result = String::new();
            let mut previous_mapping: Option<&jarde_java::report::MemberInnerSourceSegmentInput> =
                None;
            for segment in &class.segments {
                budget.poll()?;
                let mapping = unique_member_source_segment(source_type_path, &segment.binary_name)?;
                if let Some(mapping) = mapping {
                    if !segment.arguments.is_empty()
                        && segment.arguments.len() != mapping.type_parameter_count
                    {
                        return Err(Error::unsupported(
                            "ordinary_generic_source_unproved",
                            "selected member type has a different number of source arguments",
                        ));
                    }
                    if mapping.is_static || mapping.enclosing_binary_name.is_none() {
                        result = mapping.source_name.clone();
                    } else {
                        let previous = previous_mapping.ok_or_else(|| {
                            Error::unsupported(
                                "ordinary_generic_source_unproved",
                                "non-static member type has no explicit enclosing Signature segment",
                            )
                        })?;
                        if mapping.enclosing_binary_name.as_ref() != Some(&previous.binary_name)
                            || !segment_is_immediate_source_child(
                                &mapping.source_name,
                                &previous.source_name,
                            )
                        {
                            return Err(Error::unsupported(
                                "ordinary_generic_source_unproved",
                                "selected non-static member does not follow the Signature's enclosing segment",
                            ));
                        }
                        let suffix = mapping
                            .source_name
                            .strip_prefix(&previous.source_name)
                            .and_then(|suffix| suffix.strip_prefix('.'))
                            .ok_or_else(|| {
                                Error::unsupported(
                                    "ordinary_generic_source_unproved",
                                    "selected member source path is inconsistent",
                                )
                            })?;
                        if !is_java_identifier(suffix) {
                            return Err(Error::unsupported(
                                "ordinary_generic_source_unproved",
                                "selected member source segment is not a Java identifier",
                            ));
                        }
                        result.push('.');
                        result.push_str(suffix);
                    }
                    previous_mapping = Some(mapping);
                } else {
                    if class.segments.len() != 1 {
                        return Err(Error::unsupported(
                            "ordinary_generic_source_unproved",
                            "nested binary class path has no selected Java source spelling",
                        ));
                    }
                    result = match class_names {
                        ClassNameSpelling::SelectedSourcePath => {
                            simple_generic_class_name(&segment.binary_name)?
                        }
                        ClassNameSpelling::BinaryPoolName => {
                            binary_pool_class_name(&segment.binary_name)?
                        }
                    };
                    previous_mapping = None;
                }
                if !segment.arguments.is_empty() {
                    let mut arguments = Vec::with_capacity(segment.arguments.len());
                    for argument in &segment.arguments {
                        budget.poll()?;
                        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                        let spelling = match argument {
                            TypeArgument::Any => "?".to_owned(),
                            TypeArgument::Exact(ty) => spell_signature_type_with_spelling(
                                ty,
                                class_scope,
                                source_type_path,
                                class_names,
                                budget,
                                nested,
                            )?,
                            TypeArgument::Extends(ty) => format!(
                                "? extends {}",
                                spell_signature_type_with_spelling(
                                    ty,
                                    class_scope,
                                    source_type_path,
                                    class_names,
                                    budget,
                                    nested,
                                )?
                            ),
                            TypeArgument::Super(ty) => format!(
                                "? super {}",
                                spell_signature_type_with_spelling(
                                    ty,
                                    class_scope,
                                    source_type_path,
                                    class_names,
                                    budget,
                                    nested,
                                )?
                            ),
                        };
                        arguments.push(spelling);
                    }
                    result.push('<');
                    result.push_str(&arguments.join(", "));
                    result.push('>');
                }
            }
            Ok(result)
        }
    }
}

fn unique_member_source_segment<'a>(
    source_type_path: &'a [jarde_java::report::MemberInnerSourceSegmentInput],
    binary_name: &[u8],
) -> Result<Option<&'a jarde_java::report::MemberInnerSourceSegmentInput>> {
    let mut found: Option<&jarde_java::report::MemberInnerSourceSegmentInput> = None;
    for segment in source_type_path {
        if segment.binary_name.0.as_slice() != binary_name {
            continue;
        }
        if found.is_some_and(|previous| previous != segment) {
            return Err(Error::unsupported(
                "ordinary_generic_source_unproved",
                "selected member source path is ambiguous",
            ));
        }
        found = Some(segment);
    }
    Ok(found)
}

fn segment_is_immediate_source_child(child: &str, parent: &str) -> bool {
    child
        .strip_prefix(parent)
        .and_then(|suffix| suffix.strip_prefix('.'))
        .is_some_and(is_java_identifier)
}

pub fn simple_generic_class_name(raw: &[u8]) -> Result<String> {
    let text = JvmBytes(raw.to_vec()).to_unicode().ok_or_else(|| {
        Error::unsupported(
            "generic_source_shape_unproved",
            "class name has no faithful MUTF-8 Java spelling",
        )
    })?;
    if text.contains('$') || text.split('/').any(|part| !is_source_identifier(part)) {
        return Err(Error::unsupported(
            "generic_source_shape_unproved",
            "class name has no unambiguous Java source spelling",
        ));
    }
    Ok(text.replace('/', "."))
}

/// The class-header spelling of one binary class name: the class file's own name, its package
/// separators written as dots and every `$` kept — exactly the name [`crate::names::source_class_name`] writes for
/// every parent the raw header states, so a projected entry is the same reference the raw header
/// already held and only its type arguments are new.
///
/// This is the one position where a `$` name needs no nesting proof: the presentation writes
/// every class as the top-level unit its binary name states, and the class header is that unit's
/// own declaration. A member, annotation or type-parameter position cannot call this — there a
/// `$` name without the member proof that selected a source path keeps the raw descriptor
/// declaration ([`simple_generic_class_name`]).
///
/// A name with a part that is not a Java identifier has no spelling at all, exactly as in the
/// selected-source-path rule.
pub fn binary_pool_class_name(raw: &[u8]) -> Result<String> {
    jarde_java::names::source_class_name(&JvmBytes(raw.to_vec())).ok_or_else(|| {
        Error::unsupported(
            "generic_source_shape_unproved",
            "class name has no exact MUTF-8 Java source spelling",
        )
    })
}

pub fn spell_generic_body_signature_type(
    ty: &jarde_reader::signature::SignatureType,
    method_variables: &[(Vec<u8>, String, String)],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
    depth: usize,
) -> Result<String> {
    use jarde_reader::signature::SignatureType;

    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if depth > 128 {
        return Err(Error::unsupported(
            "generic_source_shape_unproved",
            "generic method type nesting exceeds source projection depth",
        ));
    }
    match ty {
        SignatureType::TypeVariable(name) => method_variables
            .iter()
            .find(|(candidate, _, _)| candidate == name)
            .map(|(_, spelling, _)| spelling.clone())
            .or_else(|| {
                class_scope
                    .iter()
                    .find(|candidate| candidate.name == *name)
                    .and_then(|candidate| source_type_parameter_name(&candidate.name))
            })
            .ok_or_else(|| {
                Error::unsupported(
                    "generic_source_shape_unproved",
                    "type variable is outside the emitted method and class scopes",
                )
            }),
        SignatureType::Array(element) => Ok(format!(
            "{}[]",
            spell_generic_body_signature_type(
                element,
                method_variables,
                class_scope,
                budget,
                depth + 1,
            )?
        )),
        _ => spell_ordinary_signature_type(ty, class_scope, budget, depth),
    }
}

pub fn generic_call_ast_key(
    invoke: &class_source::SameClassInvokeUse,
) -> Option<jarde_java::report::ClassSourceInvokeKey> {
    Some(jarde_java::report::ClassSourceInvokeKey {
        call_bci: invoke.bci,
        opcode: invoke.opcode,
        target: jarde_java::facts::CallTarget::from_raw(
            match invoke.opcode {
                0xb6 => jarde_java::facts::InvokeKind::Virtual,
                0xb7 => jarde_java::facts::InvokeKind::Special,
                0xb8 => jarde_java::facts::InvokeKind::Static,
                0xb9 => jarde_java::facts::InvokeKind::Interface,
                _ => return None,
            },
            JvmBytes(invoke.owner.clone()),
            JvmBytes(invoke.name.clone()),
            JvmBytes(invoke.descriptor.clone()),
            invoke.opcode == 0xb9,
        ),
    })
}

/// Closes an optional AST binding-fold fact to the already retained same-run local slot facts.
pub fn generic_list_foreach_binding_slot_closed(
    binding: &jarde_java::report::ClassSourceTypedForEachBindingEdit,
    store: &class_source::SameClassBodyValueSite,
    body_values: &[class_source::SameClassBodyValueSite],
    caller: &PhysicalMethodId,
    budget: &mut Budget,
) -> Result<bool> {
    use jarde_jvm::method_ir::Definition;

    let [(slot, written)] = store.local_writes.as_slice() else {
        return Ok(false);
    };
    if !matches!(written.definition, Definition::Instruction { bci, .. } if bci == binding.first_binding_bci)
        || written.replaced_by.is_some()
    {
        return Ok(false);
    }
    let target_count = binding.target_reads.len();
    let bounded = body_values
        .len()
        .saturating_add(target_count)
        .saturating_add(1);
    let use_count = written.uses.len();
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            bounded
                .saturating_mul(bounded)
                .saturating_mul(2)
                .saturating_add(use_count.saturating_mul(2)),
        )
        .unwrap_or(u64::MAX),
    )?;
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(target_count.saturating_add(1)).unwrap_or(u64::MAX),
    )?;
    let mut nested_facts = 0usize;
    for site in body_values {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let site_facts = site
            .local_reads
            .len()
            .saturating_add(site.local_writes.len())
            .saturating_add(site.stack_reads.len())
            .saturating_add(site.stack_writes.len());
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(site_facts).unwrap_or(u64::MAX),
        )?;
        budget.charge(
            CountedBudgetDimension::IrItems,
            u64::try_from(site_facts).unwrap_or(u64::MAX),
        )?;
        nested_facts = nested_facts.saturating_add(site_facts);
        for (_, fact) in &site.local_reads {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(fact.uses.len()).unwrap_or(u64::MAX),
            )?;
            budget.charge(
                CountedBudgetDimension::IrItems,
                u64::try_from(fact.uses.len()).unwrap_or(u64::MAX),
            )?;
        }
    }
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(nested_facts).unwrap_or(u64::MAX),
    )?;
    let mut target_bcis = Vec::with_capacity(target_count);
    for origins in &binding.target_reads {
        budget.poll()?;
        if !jarde_java::report::class_source_origin_set_belongs_to_method(origins, caller, budget)
            .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "foreach target read origin",
                "generic_call_ast_missing",
            )
        })? {
            return Ok(false);
        }
        target_bcis.push(origins.primary().bci());
    }
    target_bcis.sort_unstable();
    if target_bcis.windows(2).any(|pair| pair[0] == pair[1]) {
        return Ok(false);
    }
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(nested_facts.saturating_mul(2)).unwrap_or(u64::MAX),
    )?;
    let mut physical_reads = Vec::with_capacity(nested_facts);
    let mut physical_writes = Vec::with_capacity(nested_facts);
    for site in body_values {
        budget.poll()?;
        let site_facts = site
            .local_reads
            .len()
            .saturating_add(site.local_writes.len())
            .saturating_add(site.stack_reads.len())
            .saturating_add(site.stack_writes.len());
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(site_facts).unwrap_or(u64::MAX),
        )?;
        for (local_slot, fact) in &site.local_writes {
            if *local_slot == *slot {
                physical_writes.push((site.bci, fact));
            }
        }
        for (local_slot, fact) in &site.local_reads {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(fact.uses.len()).unwrap_or(u64::MAX),
            )?;
            if *local_slot == *slot {
                let [loaded] = site.stack_writes.as_slice() else {
                    return Ok(false);
                };
                if !matches!(site.opcode, 0x19 | 0x2a..=0x2d)
                    || site.local_reads.len() != 1
                    || !site.local_writes.is_empty()
                    || !site.stack_reads.is_empty()
                    || !matches!(loaded.definition, Definition::Instruction { bci, .. } if bci == site.bci)
                    || loaded.replaced_by.is_some()
                    || !fact.uses.contains(&Some(site.bci))
                    || fact.value != written.value
                    || fact.definition != written.definition
                    || fact.replaced_by.is_some()
                {
                    return Ok(false);
                }
                physical_reads.push(site.bci);
            }
        }
    }
    physical_writes.sort_by_key(|(bci, _)| *bci);
    if physical_writes.len() != 1
        || physical_writes[0].0 != binding.first_binding_bci
        || physical_writes[0].1 != written
    {
        return Ok(false);
    }
    physical_reads.sort_unstable();
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(use_count).unwrap_or(u64::MAX),
    )?;
    let mut stored_uses = Vec::with_capacity(use_count);
    stored_uses.extend(written.uses.iter().flatten().copied());
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(use_count)
            .unwrap_or(u64::MAX)
            .saturating_mul(u64::from(use_count.checked_ilog2().unwrap_or(0)).saturating_add(2)),
    )?;
    stored_uses.sort_unstable();
    Ok(physical_reads == target_bcis && physical_reads == stored_uses)
}

/// Produces only optional typed foreach edits whose source Signature, retained AST, and same-run
/// next/checkcast/store facts all identify the same direct List formal and element type.
pub fn prove_generic_list_typed_foreach_edits(
    candidate: &jarde_java::report::GenericReturnCandidate,
    ast: &jarde_java::report::ClassSourceMethodAst,
    caller: &PhysicalMethodId,
    invokes: &[class_source::SameClassInvokeUse],
    body_values: &[class_source::SameClassBodyValueSite],
    list_parameter_slots: &[u16],
    element_types: &[(u16, jarde_java::ast::Type)],
    budget: &mut Budget,
) -> Result<Vec<jarde_java::report::ClassSourceTypedForEachEdit>> {
    use jarde_java::ast::Type;
    use jarde_jvm::method_ir::Definition;

    let jarde_java::report::GenericReturnValue::CompleteBody {
        parameter_read_bcis,
        parameter_receiver_calls,
        ..
    } = &candidate.value
    else {
        return Ok(Vec::new());
    };
    let Some(foreach_sites) =
        jarde_java::report::class_source_for_each_local_iterables(ast, budget).map_err(|stop| {
            enum_projection_stop_error(stop, "typed foreach AST sites", "generic_call_ast_missing")
        })?
    else {
        return Ok(Vec::new());
    };
    let mut result = Vec::new();
    let Some(formals) =
        jarde_java::report::class_source_method_formal_names(ast, budget).map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "typed foreach formal inventory",
                "generic_call_ast_missing",
            )
        })?
    else {
        return Ok(result);
    };
    let bounded_work = parameter_read_bcis
        .len()
        .saturating_add(parameter_receiver_calls.len())
        .saturating_add(foreach_sites.len())
        .saturating_add(invokes.len())
        .saturating_add(body_values.len())
        .saturating_add(formals.len())
        .saturating_add(list_parameter_slots.len())
        .saturating_add(element_types.len())
        .saturating_add(1);
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(parameter_read_bcis.len())
            .unwrap_or(u64::MAX)
            .saturating_mul(u64::try_from(bounded_work).unwrap_or(u64::MAX))
            .saturating_mul(8),
    )?;
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(parameter_read_bcis.len()).unwrap_or(u64::MAX),
    )?;
    for (slot, read_bci) in parameter_read_bcis {
        budget.poll()?;
        if !list_parameter_slots.contains(slot) {
            continue;
        }
        let Some((_, element_type)) = element_types
            .iter()
            .find(|(candidate_slot, _)| candidate_slot == slot)
        else {
            continue;
        };
        let mut formal_matches = formals.iter().filter(|formal| formal.slot == *slot);
        let Some(formal) = formal_matches.next() else {
            continue;
        };
        if formal_matches.next().is_some() {
            continue;
        }
        let formal_name = &formal.name;
        let mut receiver_calls = parameter_receiver_calls
            .iter()
            .filter(|site| site.parameter_slot == *slot && site.load_bci == *read_bci);
        let Some(receiver_call) = receiver_calls.next() else {
            continue;
        };
        if receiver_calls.next().is_some() {
            continue;
        }
        let mut iterator_invokes = invokes.iter().filter(|invoke| {
            invoke.physical_caller == *caller
                && invoke.bci == receiver_call.call_bci
                && invoke.owner.as_slice() == b"java/util/List"
                && invoke.name.as_slice() == b"iterator"
                && invoke.descriptor.as_slice() == b"()Ljava/util/Iterator;"
                && invoke.opcode == 0xb9
                && invoke.ssa_operands_complete
        });
        let Some(iterator_invoke) = iterator_invokes.next() else {
            continue;
        };
        if iterator_invokes.next().is_some() {
            continue;
        }
        if iterator_invoke.receiver_value_fact.as_ref().is_none_or(|fact| {
            fact.replaced_by.is_some()
                || fact.uses.as_slice() != [Some(iterator_invoke.bci)]
                || !matches!(fact.definition, Definition::Instruction { bci, .. } if bci == *read_bci)
        }) {
            continue;
        }
        let mut matching_sites = foreach_sites.iter().filter(|site| {
            site.expression_bci == *read_bci
                && site.local_name.as_deref() == Some(formal_name.as_str())
        });
        let Some(site) = matching_sites.next() else {
            continue;
        };
        let binding_origin_owned = match site.first_binding_origin.as_ref() {
            Some(origin) => jarde_java::report::class_source_origin_set_belongs_to_method(
                origin, caller, budget,
            )
            .map_err(|stop| {
                enum_projection_stop_error(
                    stop,
                    "typed foreach binding origin",
                    "generic_call_ast_missing",
                )
            })?,
            None => false,
        };
        let cast_origin_owned = match site.first_binding_cast_origin.as_ref() {
            Some(origin) => jarde_java::report::class_source_origin_set_belongs_to_method(
                origin, caller, budget,
            )
            .map_err(|stop| {
                enum_projection_stop_error(
                    stop,
                    "typed foreach cast origin",
                    "generic_call_ast_missing",
                )
            })?,
            None => false,
        };
        if matching_sites.next().is_some()
            || !site.foreign_origin_bcis.is_empty()
            || !binding_origin_owned
            || !cast_origin_owned
            || !matches!(&site.header_type, Type::Reference(name) if name == "java.lang.Object")
            || site.first_binding_cast_type.as_ref() != Some(element_type)
            || site.first_binding_cast_local.as_deref() != Some(site.header_name.as_str())
        {
            continue;
        }
        let (Some(binding_bci), Some(cast_bci)) =
            (site.first_binding_bci, site.first_binding_cast_bci)
        else {
            continue;
        };
        if jarde_java::report::class_source_instruction_opcode(ast, cast_bci, budget).map_err(
            |stop| {
                enum_projection_stop_error(
                    stop,
                    "typed foreach cast opcode",
                    "generic_call_ast_missing",
                )
            },
        )? != Some(0xc0)
        {
            continue;
        }
        let mut cast_sites = body_values.iter().filter(|value| value.bci == cast_bci);
        let Some(cast) = cast_sites.next() else {
            continue;
        };
        if cast_sites.next().is_some() || cast.opcode != 0xc0 {
            continue;
        }
        let [(_, cast_input)] = cast.stack_reads.as_slice() else {
            continue;
        };
        let [cast_output] = cast.stack_writes.as_slice() else {
            continue;
        };
        if cast_input.replaced_by.is_some()
            || cast_input.uses.as_slice() != [Some(cast_bci)]
            || cast_output.replaced_by.is_some()
            || cast_output.uses.as_slice() != [Some(binding_bci)]
            || !matches!(cast_output.definition, Definition::Instruction { bci, .. } if bci == cast_bci)
        {
            continue;
        }
        let mut matching_next = invokes.iter().filter(|invoke| {
            invoke.physical_caller == *caller
                && invoke.owner.as_slice() == b"java/util/Iterator"
                && invoke.name.as_slice() == b"next"
                && invoke.descriptor.as_slice() == b"()Ljava/lang/Object;"
                && invoke.opcode == 0xb9
                && invoke.ssa_operands_complete
                && invoke.result_value_fact.as_ref().is_some_and(|fact| {
                    fact.value == cast_input.value
                        && fact.replaced_by.is_none()
                        && fact.uses.as_slice() == [Some(cast_bci)]
                        && matches!(fact.definition, Definition::Instruction { bci, .. } if bci == invoke.bci)
                })
        });
        let Some(next_invoke) = matching_next.next() else {
            continue;
        };
        if matching_next.next().is_some() {
            continue;
        }
        budget.poll()?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(
                site.origin_bcis
                    .len()
                    .saturating_add(site.foreign_origin_bcis.len()),
            )
            .unwrap_or(u64::MAX),
        )?;
        if !site.origin_bcis.contains(&next_invoke.bci)
            || site.foreign_origin_bcis.contains(&next_invoke.bci)
        {
            continue;
        }
        let mut stores = body_values.iter().filter(|value| value.bci == binding_bci);
        let Some(store) = stores.next() else {
            continue;
        };
        if stores.next().is_some()
            || store.local_writes.len() != 1
            || store.stack_reads.len() != 1
            || store.stack_reads[0].1.value != cast_output.value
            || store.stack_reads[0].1.replaced_by.is_some()
            || !matches!(store.opcode, 0x3a | 0x4b..=0x4e)
        {
            continue;
        }
        let Some(cast_origin) = site.first_binding_cast_origin.as_ref() else {
            continue;
        };
        let cast_origin_items = cast_origin.derived().len().saturating_add(1);
        budget.charge(
            CountedBudgetDimension::IrItems,
            u64::try_from(cast_origin_items).unwrap_or(u64::MAX),
        )?;
        if let Type::Reference(name) = element_type {
            budget.charge(
                CountedBudgetDimension::OutputBytes,
                u64::try_from(name.len()).unwrap_or(u64::MAX),
            )?;
        }
        let cast_identity_bytes = jarde_java::report::class_source_method_identity_work(
            caller, budget,
        )
        .map_err(|stop| {
            enum_projection_stop_error(stop, "typed foreach cast clone", "generic_call_ast_missing")
        })?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(cast_origin_items.saturating_mul(cast_identity_bytes))
                .unwrap_or(u64::MAX),
        )?;
        let mut binding = None;
        if let Some(candidate_binding) =
            jarde_java::report::class_source_typed_foreach_binding_site(
                ast,
                &jarde_java::report::ClassSourceTypedForEachEdit {
                    statement_bci: site.statement_bci,
                    cast_bci,
                    ty: element_type.clone(),
                    cast_origin: cast_origin.clone(),
                    header_origin_count: site.origin_bcis.len(),
                    binding: None,
                },
                budget,
            )
            .map_err(|stop| {
                enum_projection_stop_error(stop, "foreach binding site", "generic_call_ast_missing")
            })?
        {
            if generic_list_foreach_binding_slot_closed(
                &candidate_binding,
                store,
                body_values,
                caller,
                budget,
            )? {
                binding = Some(candidate_binding);
            }
        }
        result.push(jarde_java::report::ClassSourceTypedForEachEdit {
            statement_bci: site.statement_bci,
            cast_bci,
            ty: element_type.clone(),
            cast_origin: cast_origin.clone(),
            header_origin_count: site.origin_bcis.len(),
            binding,
        });
    }
    result.sort_by_key(|edit| edit.statement_bci);
    if result
        .windows(2)
        .any(|pair| pair[0].statement_bci == pair[1].statement_bci)
    {
        return Ok(Vec::new());
    }
    Ok(result)
}

pub fn push_generic_optional_removal(
    removals: &mut Vec<jarde_java::report::ClassSourceInvokeArgumentPresentationCast>,
    site: &jarde_java::report::ClassSourceInvokeAstSite,
    argument_index: usize,
    budget: &mut Budget,
) -> Result<()> {
    budget.charge(CountedBudgetDimension::IrItems, 1)?;
    let target_bytes = site
        .key
        .target
        .owner()
        .len()
        .saturating_add(site.key.target.name().len())
        .saturating_add(site.key.target.descriptor().len());
    budget.charge(
        CountedBudgetDimension::OutputBytes,
        u64::try_from(target_bytes).unwrap_or(u64::MAX),
    )?;
    removals.push(
        jarde_java::report::ClassSourceInvokeArgumentPresentationCast {
            call_bci: site.key.call_bci,
            opcode: site.key.opcode,
            target: site.key.target.clone(),
            argument_index,
        },
    );
    Ok(())
}

/// Proves that one consumed local is the direct result of an enhanced-for element checkcast.
/// The caller chooses whether the proven reference type is required generic evidence (add) or
/// only supports an optional erased-Object wrapper removal (remove/equals).
pub fn prove_generic_list_element_local_argument(
    ast: &jarde_java::report::ClassSourceMethodAst,
    actual_name: &str,
    load_bci: u32,
    value_fact: &class_source::SameClassInvokeValueFact,
    invoke: &class_source::SameClassInvokeUse,
    caller: &PhysicalMethodId,
    invokes: &[class_source::SameClassInvokeUse],
    body_values: &[class_source::SameClassBodyValueSite],
    formals: &[jarde_java::report::ClassSourceMethodFormalName],
    declarations: &[jarde_java::report::ClassSourceAstLocalDeclaration],
    assignments: &[jarde_java::report::ClassSourceAstLocalAssignment],
    foreach_sites: &[jarde_java::report::ClassSourceAstForEachIterable],
    selected_reads: &[(u16, u32)],
    budget: &mut Budget,
) -> Result<Option<jarde_java::ast::Type>> {
    use jarde_java::ast::Type;
    use jarde_jvm::method_ir::Definition;

    budget.poll()?;
    if invoke.physical_caller != *caller {
        return Ok(None);
    }
    let bound = formals
        .len()
        .saturating_add(declarations.len())
        .saturating_add(assignments.len())
        .saturating_add(foreach_sites.len())
        .saturating_add(selected_reads.len())
        .saturating_add(invokes.len())
        .saturating_add(body_values.len())
        .saturating_add(1);
    let work = u64::try_from(bound).unwrap_or(u64::MAX).saturating_mul(12);
    budget.charge(CountedBudgetDimension::AnalysisSteps, work)?;
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(body_values.len().saturating_add(1)).unwrap_or(u64::MAX),
    )?;

    let name_work = declarations
        .iter()
        .map(|item| item.name.len())
        .chain(assignments.iter().map(|item| item.name.len()))
        .fold(
            actual_name
                .len()
                .saturating_mul(declarations.len().saturating_add(assignments.len())),
            usize::saturating_add,
        );
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(name_work).unwrap_or(u64::MAX),
    )?;
    let mut declaration_matches = declarations.iter().filter(|item| item.name == actual_name);
    let Some(declaration) = declaration_matches.next() else {
        return Ok(None);
    };
    if declaration_matches.next().is_some() {
        return Ok(None);
    }
    let Type::Reference(actual_type) = &declaration.ty else {
        return Ok(None);
    };
    let mut assignment_matches = assignments.iter().filter(|item| item.name == actual_name);
    let Some(assignment) = assignment_matches.next() else {
        return Ok(None);
    };
    if assignment_matches.next().is_some() {
        return Ok(None);
    }
    let Some(cast_ty) = assignment.cast_type.as_ref() else {
        return Ok(None);
    };
    let type_name_work = match (cast_ty, &declaration.ty) {
        (Type::Reference(cast_name), Type::Reference(declared_name)) => {
            cast_name.len().saturating_add(declared_name.len())
        }
        _ => return Ok(None),
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(type_name_work).unwrap_or(u64::MAX),
    )?;
    let identity_bytes = jarde_java::report::class_source_method_identity_work(caller, budget)
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "List local scope identity",
                "generic_call_ast_missing",
            )
        })?;
    let name_bytes = formals
        .iter()
        .map(|formal| formal.name.len())
        .chain(
            foreach_sites
                .iter()
                .filter_map(|site| site.local_name.as_ref().map(String::len)),
        )
        .fold(0usize, usize::saturating_add);
    let foreach_anchor_pairs = foreach_sites
        .len()
        .saturating_mul(assignment.scope_anchors.len());
    let declaration_anchor_work = declaration
        .scope_anchors
        .len()
        .saturating_mul(identity_bytes)
        .saturating_mul(2);
    let scope_work = selected_reads.len().saturating_mul(
        formals
            .len()
            .saturating_add(foreach_sites.len())
            .saturating_add(declaration_anchor_work)
            .saturating_add(foreach_anchor_pairs.saturating_mul(identity_bytes))
            .saturating_add(name_bytes),
    );
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(scope_work).unwrap_or(u64::MAX),
    )?;
    let declaration_scope_is_prefix = assignment
        .scope_anchors
        .get(..declaration.scope_anchors.len())
        == Some(declaration.scope_anchors.as_slice());
    let assignment_loop_matches = selected_reads
        .iter()
        .filter_map(|(loop_slot, loop_read_bci)| {
            let mut formal_matches = formals.iter().filter(|formal| formal.slot == *loop_slot);
            let formal = formal_matches.next()?;
            if formal_matches.next().is_some() {
                return None;
            }
            foreach_sites.iter().find(|site| {
                site.expression_bci == *loop_read_bci
                    && site.local_name.as_deref() == Some(formal.name.as_str())
                    && assignment.scope_anchors.iter().any(|anchor| {
                        anchor.bci == site.statement_bci && anchor.method.as_ref() == Some(caller)
                    })
            })
        })
        .count();
    let cast_bci = assignment.value_bci;
    if cast_ty != &declaration.ty
        || assignment.bci >= invoke.bci
        || !declaration_scope_is_prefix
        || assignment_loop_matches == 0
        || jarde_java::report::class_source_instruction_opcode(ast, cast_bci, budget).map_err(
            |stop| {
                enum_projection_stop_error(
                    stop,
                    "List local cast opcode",
                    "generic_call_ast_missing",
                )
            },
        )? != Some(0xc0)
    {
        return Ok(None);
    }
    let Some(cast_value) = body_values.iter().find(|value| value.bci == cast_bci) else {
        return Ok(None);
    };
    let [(_, cast_input)] = cast_value.stack_reads.as_slice() else {
        return Ok(None);
    };
    let [cast_output] = cast_value.stack_writes.as_slice() else {
        return Ok(None);
    };
    if cast_value.opcode != 0xc0
        || !matches!(cast_output.definition, Definition::Instruction { bci, .. } if bci == cast_bci)
        || cast_input.replaced_by.is_some()
        || cast_output.replaced_by.is_some()
    {
        return Ok(None);
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(cast_output.uses.len().saturating_add(1)).unwrap_or(u64::MAX),
    )?;
    if cast_output.uses.as_slice() != [Some(assignment.bci)] {
        return Ok(None);
    }
    let next_use_work = invokes
        .iter()
        .filter(|next| {
            next.physical_caller == *caller
                && next.owner.as_slice() == b"java/util/Iterator"
                && next.name.as_slice() == b"next"
                && next.descriptor.as_slice() == b"()Ljava/lang/Object;"
                && next.opcode == 0xb9
                && next.ssa_operands_complete
        })
        .map(|next| {
            next.result_value_fact
                .as_ref()
                .map_or(0, |fact| fact.uses.len())
        })
        .fold(0usize, usize::saturating_add);
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(next_use_work).unwrap_or(u64::MAX),
    )?;
    let mut next_calls = invokes.iter().filter(|next| {
        next.physical_caller == *caller
            && next.owner.as_slice() == b"java/util/Iterator"
            && next.name.as_slice() == b"next"
            && next.descriptor.as_slice() == b"()Ljava/lang/Object;"
            && next.opcode == 0xb9
            && next.ssa_operands_complete
            && next.result_value_fact.as_ref().is_some_and(|result| {
                result.value == cast_input.value
                    && matches!(result.definition, Definition::Instruction { bci, .. } if bci == next.bci)
                    && result.replaced_by.is_none()
                    && result.uses.as_slice() == [Some(cast_bci)]
            })
    });
    let Some(_next_call) = next_calls.next() else {
        return Ok(None);
    };
    if next_calls.next().is_some() {
        return Ok(None);
    }
    let Some(declaration_value) = body_values.iter().find(|value| value.bci == assignment.bci)
    else {
        return Ok(None);
    };
    let [(actual_slot, stored_value)] = declaration_value.local_writes.as_slice() else {
        return Ok(None);
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(declaration_value.stack_reads.len().saturating_add(1)).unwrap_or(u64::MAX),
    )?;
    if !matches!(stored_value.definition, Definition::Instruction { bci, .. } if bci == declaration_value.bci)
        || stored_value.replaced_by.is_some()
        || declaration_value.stack_reads.iter().filter(|(_, fact)| {
            fact.value == cast_output.value
                && matches!(fact.definition, Definition::Instruction { bci, .. } if bci == cast_bci)
                && fact.replaced_by.is_none()
        }).count() != 1
    {
        return Ok(None);
    }
    let mut write_count = 0usize;
    for site in body_values {
        budget.poll()?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(site.local_writes.len().saturating_add(1)).unwrap_or(u64::MAX),
        )?;
        write_count = write_count.saturating_add(
            site.local_writes
                .iter()
                .filter(|(local, _)| local == actual_slot)
                .count(),
        );
    }
    if write_count != 1 {
        return Ok(None);
    }
    let mut loads = body_values.iter().filter(|value| value.bci == load_bci);
    let Some(load_value) = loads.next() else {
        return Ok(None);
    };
    if loads.next().is_some() {
        return Ok(None);
    }
    let [(loaded_slot, local_value)] = load_value.local_reads.as_slice() else {
        return Ok(None);
    };
    let [loaded_value] = load_value.stack_writes.as_slice() else {
        return Ok(None);
    };
    if *loaded_slot != *actual_slot
        || load_value.opcode != 0x19 && !matches!(load_value.opcode, 0x2a..=0x2d)
        || load_value.local_writes.len() != 0
        || !load_value.stack_reads.is_empty()
        || local_value.replaced_by.is_some()
        || local_value.value != stored_value.value
        || loaded_value.value != value_fact.value
        || !matches!(value_fact.definition, Definition::Instruction { bci, .. } if bci == load_bci)
        || loaded_value.replaced_by.is_some()
        || value_fact.replaced_by.is_some()
        || !matches!(loaded_value.definition, Definition::Instruction { bci, .. } if bci == load_value.bci)
    {
        return Ok(None);
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            value_fact
                .uses
                .len()
                .saturating_add(loaded_value.uses.len())
                .saturating_add(1),
        )
        .unwrap_or(u64::MAX),
    )?;
    if value_fact.uses.as_slice() != [Some(invoke.bci)]
        || loaded_value.uses.as_slice() != [Some(invoke.bci)]
    {
        return Ok(None);
    }
    budget.charge(
        CountedBudgetDimension::OutputBytes,
        u64::try_from(actual_type.len()).unwrap_or(u64::MAX),
    )?;
    Ok(Some(Type::Reference(actual_type.clone())))
}

/// Proves the closed List receiver uses of a complete-body candidate from the same physical
/// reads and retained AST. The returned slot/type tuples are transaction-local inputs to the
/// parsed Signature gate; the second result contains required add(Object) and independently
/// proved optional argument presentation removals.
pub fn prove_generic_list_parameter_body(
    candidate: &jarde_java::report::GenericReturnCandidate,
    ast: &jarde_java::report::ClassSourceMethodAst,
    caller: &PhysicalMethodId,
    invokes: &[class_source::SameClassInvokeUse],
    body_values: &[class_source::SameClassBodyValueSite],
    list_parameter_slots: &[u16],
    element_types: &[(u16, jarde_java::ast::Type)],
    budget: &mut Budget,
) -> Result<
    Option<(
        Vec<(u16, Vec<String>)>,
        Vec<jarde_java::report::ClassSourceInvokeArgumentPresentationCast>,
    )>,
> {
    use jarde_java::ast::Type;
    use jarde_java::report::{
        ClassSourceAstExpressionShape as Shape, ClassSourceInvokeArgumentPresentationCast,
        ClassSourceInvokeKey, GenericReturnValue,
    };
    use jarde_jvm::method_ir::Definition;

    let GenericReturnValue::CompleteBody {
        result_type,
        parameter_read_bcis,
        parameter_receiver_calls,
        ..
    } = &candidate.value
    else {
        return Ok(None);
    };
    if list_parameter_slots.is_empty() {
        return Ok(Some((Vec::new(), Vec::new())));
    }
    if result_type.is_some()
        && (!matches!(
            result_type,
            Some(
                Type::Boolean
                    | Type::Byte
                    | Type::Char
                    | Type::Short
                    | Type::Int
                    | Type::Long
                    | Type::Float
                    | Type::Double
            )
        ) || element_types.is_empty())
    {
        return Ok(None);
    }
    budget.poll()?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            parameter_read_bcis
                .len()
                .saturating_add(parameter_receiver_calls.len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    let Some(formals) =
        jarde_java::report::class_source_method_formal_names(ast, budget).map_err(|stop| {
            enum_projection_stop_error(stop, "List formal inventory", "generic_call_ast_missing")
        })?
    else {
        return Ok(None);
    };
    let Some(declarations) = jarde_java::report::class_source_local_declarations(ast, budget)
        .map_err(|stop| {
            enum_projection_stop_error(stop, "List local declarations", "generic_call_ast_missing")
        })?
    else {
        return Ok(None);
    };
    let Some(assignments) = jarde_java::report::class_source_local_assignments(ast, budget)
        .map_err(|stop| {
            enum_projection_stop_error(stop, "List local assignments", "generic_call_ast_missing")
        })?
    else {
        return Ok(None);
    };
    let Some(foreach_sites) =
        jarde_java::report::class_source_for_each_local_iterables(ast, budget).map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "List enhanced-for sources",
                "generic_call_ast_missing",
            )
        })?
    else {
        return Ok(None);
    };

    let selected_bound = parameter_read_bcis
        .len()
        .saturating_add(parameter_receiver_calls.len())
        .saturating_add(1);
    let scan_bound = formals
        .len()
        .saturating_add(declarations.len())
        .saturating_add(assignments.len())
        .saturating_add(foreach_sites.len())
        .saturating_add(invokes.len())
        .saturating_add(body_values.len())
        .saturating_add(parameter_receiver_calls.len())
        .saturating_add(parameter_read_bcis.len())
        .saturating_add(1);
    let proof_work = u64::try_from(selected_bound)
        .unwrap_or(u64::MAX)
        .saturating_mul(u64::try_from(scan_bound).unwrap_or(u64::MAX))
        .saturating_mul(8);
    budget.charge(CountedBudgetDimension::AnalysisSteps, proof_work)?;
    budget.charge(CountedBudgetDimension::IrItems, proof_work)?;

    let mut call_keys = Vec::<ClassSourceInvokeKey>::new();
    for physical_use in parameter_receiver_calls
        .iter()
        .filter(|site| list_parameter_slots.contains(&site.parameter_slot))
    {
        budget.poll()?;
        let Some(invoke) = invokes.iter().find(|invoke| {
            invoke.physical_caller == *caller && invoke.bci == physical_use.call_bci
        }) else {
            continue;
        };
        if !invoke.ssa_operands_complete {
            continue;
        }
        let Some(key) = generic_call_ast_key(invoke) else {
            continue;
        };
        if invoke.name.as_slice() != b"iterator" {
            call_keys.push(key);
        }
    }
    call_keys.sort_by_key(|key| key.call_bci);
    if call_keys
        .windows(2)
        .any(|pair| pair[0].call_bci == pair[1].call_bci)
    {
        return Ok(None);
    }
    let Some(call_sites) = jarde_java::report::class_source_invoke_ast_sites(
        ast, &call_keys, budget,
    )
    .map_err(|stop| {
        enum_projection_stop_error(stop, "List call AST sites", "generic_call_ast_missing")
    })?
    else {
        return Ok(None);
    };
    let formal_name = |slot: u16| -> Option<&str> {
        let mut matches = formals.iter().filter(|formal| formal.slot == slot);
        let formal = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        Some(formal.name.as_str())
    };
    let mut type_uses = std::collections::BTreeMap::<u16, Vec<String>>::new();
    let mut removals = Vec::new();
    let mut indexed_get_casts = Vec::<(ClassSourceInvokeKey, u32, Type)>::new();
    let selected_reads = parameter_read_bcis
        .iter()
        .filter(|(slot, _)| list_parameter_slots.contains(slot))
        .copied()
        .collect::<Vec<_>>();
    if selected_reads.is_empty() {
        return Ok(None);
    }
    for slot in list_parameter_slots {
        if !selected_reads.iter().any(|read| read.0 == *slot) {
            return Ok(None);
        }
    }
    for (slot, read_bci) in &selected_reads {
        budget.poll()?;
        let Some(name) = formal_name(*slot) else {
            return Ok(None);
        };
        let mut physical_uses = parameter_receiver_calls
            .iter()
            .filter(|site| site.parameter_slot == *slot && site.load_bci == *read_bci);
        let Some(physical_use) = physical_uses.next() else {
            // A read which was not one exact direct invoke-receiver load prevents List-use
            // closure, while leaving the baseline complete-body candidate intact for old gates.
            return Ok(None);
        };
        if physical_uses.next().is_some() {
            return Ok(None);
        }
        let Some(invoke) = invokes.iter().find(|invoke| {
            invoke.physical_caller == *caller && invoke.bci == physical_use.call_bci
        }) else {
            return Ok(None);
        };
        if !invoke.ssa_operands_complete
            || invoke.receiver_value_fact.as_ref().is_none_or(|receiver| {
                receiver.replaced_by.is_some()
                    || receiver.uses.as_slice() != [Some(invoke.bci)]
                    || !matches!(receiver.definition, Definition::Instruction { bci, .. } if bci == *read_bci)
            })
            || invoke.owner.as_slice() != b"java/util/List"
            || invoke.opcode != 0xb9
        {
            return Ok(None);
        }
        if generic_call_ast_key(invoke).is_none() {
            return Ok(None);
        }
        let Some(call_kind) = generic_list_member_contract(&invoke.name, &invoke.descriptor) else {
            return Ok(None);
        };
        type_uses.entry(*slot).or_default();
        if call_kind == 0 {
            let matches = foreach_sites
                .iter()
                .filter(|site| {
                    site.expression_bci == *read_bci && site.local_name.as_deref() == Some(name)
                })
                .count();
            if matches != 1 {
                return Ok(None);
            }
            continue;
        }
        let Some(site) = call_sites
            .iter()
            .find(|site| site.key.call_bci == invoke.bci)
        else {
            return Ok(None);
        };
        let Some(receiver) = site.receiver.as_ref() else {
            return Ok(None);
        };
        if receiver.shape != Shape::Local
            || receiver.direct_local_name.as_deref() != Some(name)
            || receiver.primary.bci != *read_bci
        {
            return Ok(None);
        }
        if call_kind == 5 {
            if invoke.descriptor.as_slice() != b"(I)Ljava/lang/Object;"
                || invoke.argument_value_facts.len() != 1
            {
                return Ok(None);
            }
            let Some(element_type) = element_types
                .iter()
                .find(|(element_slot, _)| element_slot == slot)
                .map(|(_, ty)| ty)
            else {
                return Ok(None);
            };
            let Some(result_fact) = invoke.result_value_fact.as_ref() else {
                return Ok(None);
            };
            if result_fact.replaced_by.is_some()
                || !matches!(result_fact.definition, Definition::Instruction { bci, .. } if bci == invoke.bci)
                || result_fact.uses.len() != 1
            {
                return Ok(None);
            }
            let [Some(cast_bci)] = result_fact.uses.as_slice() else {
                return Ok(None);
            };
            if *cast_bci <= invoke.bci {
                return Ok(None);
            }
            let mut cast_rows = body_values.iter().filter(|value| value.bci == *cast_bci);
            let Some(cast) = cast_rows.next() else {
                return Ok(None);
            };
            let [cast_output] = cast.stack_writes.as_slice() else {
                return Ok(None);
            };
            let [cast_input] = cast.stack_reads.as_slice() else {
                return Ok(None);
            };
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(
                    result_fact
                        .uses
                        .len()
                        .saturating_add(cast_input.1.uses.len())
                        .saturating_add(1),
                )
                .unwrap_or(u64::MAX),
            )?;
            if cast_rows.next().is_some()
                || cast.opcode != 0xc0
                || &cast_input.1 != result_fact
                || cast_input.1.replaced_by.is_some()
                || !matches!(cast_output.definition, Definition::Instruction { bci, .. } if bci == *cast_bci)
                || cast_output.replaced_by.is_some()
            {
                return Ok(None);
            }
            let clone_work = invoke
                .owner
                .len()
                .saturating_add(invoke.name.len())
                .saturating_add(invoke.descriptor.len())
                .saturating_add(element_type.spell().len());
            budget.poll()?;
            budget.charge(
                CountedBudgetDimension::OutputBytes,
                u64::try_from(clone_work).unwrap_or(u64::MAX),
            )?;
            let Some(key) = generic_call_ast_key(invoke) else {
                return Ok(None);
            };
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            indexed_get_casts.push((key, *cast_bci, element_type.clone()));
            continue;
        }
        if call_kind == 1 {
            let Some(argument) = site.arguments.first() else {
                return Ok(None);
            };
            let Some(wrapper) = argument.presentation_wrapper.as_ref() else {
                return Ok(None);
            };
            if wrapper.ty != Type::Reference("java.lang.Object".to_owned())
                || argument.primary.bci != wrapper.child.primary.bci
            {
                return Ok(None);
            }
            if wrapper.child.shape == Shape::Null {
                let null_bci = wrapper.child.primary.bci;
                budget.poll()?;
                budget.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(body_values.len())
                        .unwrap_or(u64::MAX)
                        .saturating_add(1),
                )?;
                if jarde_java::report::class_source_instruction_opcode(ast, null_bci, budget)
                    .map_err(|stop| {
                        enum_projection_stop_error(
                            stop,
                            "List null argument opcode",
                            "generic_call_ast_missing",
                        )
                    })?
                    != Some(0x01)
                {
                    return Ok(None);
                }
                let mut null_sites = body_values.iter().filter(|value| value.bci == null_bci);
                let Some(null_value) = null_sites.next() else {
                    return Ok(None);
                };
                if null_sites.next().is_some() {
                    return Ok(None);
                }
                let [null_output] = null_value.stack_writes.as_slice() else {
                    return Ok(None);
                };
                let [argument_value] = invoke.argument_value_facts.as_slice() else {
                    return Ok(None);
                };
                if null_value.opcode != 0x01
                    || !null_value.stack_reads.is_empty()
                    || !null_value.local_reads.is_empty()
                    || !null_value.local_writes.is_empty()
                    || !matches!(null_output.definition, Definition::Instruction { bci, .. } if bci == null_bci)
                    || null_output.replaced_by.is_some()
                    || null_output.uses.as_slice() != [Some(invoke.bci)]
                    || argument_value.value != null_output.value
                    || !matches!(argument_value.definition, Definition::Instruction { bci, .. } if bci == null_bci)
                    || argument_value.replaced_by.is_some()
                    || argument_value.uses.as_slice() != [Some(invoke.bci)]
                {
                    return Ok(None);
                }
                removals.push(ClassSourceInvokeArgumentPresentationCast {
                    call_bci: site.key.call_bci,
                    opcode: site.key.opcode,
                    target: site.key.target.clone(),
                    argument_index: 0,
                });
                continue;
            }
            let Some(actual_name) = wrapper.child.direct_local_name.as_deref() else {
                return Ok(None);
            };
            if wrapper.child.shape != Shape::Local {
                return Ok(None);
            }
            let Some(argument_fact) = invoke.argument_value_facts.first() else {
                return Ok(None);
            };
            if invoke.argument_value_facts.len() != 1 {
                return Ok(None);
            }
            let Some(actual_type) = prove_generic_list_element_local_argument(
                ast,
                actual_name,
                wrapper.child.primary.bci,
                argument_fact,
                invoke,
                caller,
                invokes,
                body_values,
                &formals,
                &declarations,
                &assignments,
                &foreach_sites,
                &selected_reads,
                budget,
            )?
            else {
                return Ok(None);
            };
            type_uses.entry(*slot).or_default().push(match actual_type {
                Type::Reference(name) => name,
                _ => unreachable!("the element-local proof returns a reference type"),
            });
            removals.push(ClassSourceInvokeArgumentPresentationCast {
                call_bci: site.key.call_bci,
                opcode: site.key.opcode,
                target: site.key.target.clone(),
                argument_index: 0,
            });
        } else if call_kind == 2 {
            // remove(Object) is a cleanup opportunity only. Its argument never contributes to
            // type_uses, and a missing local proof leaves the original wrapper untouched.
            let Some(argument) = site.arguments.first() else {
                continue;
            };
            let Some(wrapper) = argument.presentation_wrapper.as_ref() else {
                continue;
            };
            if !matches!(&wrapper.ty, Type::Reference(name) if name == "java.lang.Object")
                || site.arguments.len() != 1
                || argument.primary.bci != wrapper.child.primary.bci
                || wrapper.child.primary.method.as_ref() != Some(caller)
                || wrapper.child.shape != Shape::Local
            {
                continue;
            }
            let Some(actual_name) = wrapper.child.direct_local_name.as_deref() else {
                continue;
            };
            let [argument_fact] = invoke.argument_value_facts.as_slice() else {
                continue;
            };
            if prove_generic_list_element_local_argument(
                ast,
                actual_name,
                wrapper.child.primary.bci,
                argument_fact,
                invoke,
                caller,
                invokes,
                body_values,
                &formals,
                &declarations,
                &assignments,
                &foreach_sites,
                &selected_reads,
                budget,
            )?
            .is_some()
            {
                push_generic_optional_removal(&mut removals, site, 0, budget)?;
            }
        }
    }

    if !indexed_get_casts.is_empty() {
        let relation_sort = indexed_get_casts.len().saturating_mul(
            usize::BITS as usize - indexed_get_casts.len().max(1).leading_zeros() as usize + 1,
        );
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(relation_sort).unwrap_or(u64::MAX),
        )?;
        budget.charge(
            CountedBudgetDimension::IrItems,
            u64::try_from(indexed_get_casts.len()).unwrap_or(u64::MAX),
        )?;
        indexed_get_casts.sort_by_key(|(key, _, _)| key.call_bci);
        if indexed_get_casts
            .windows(2)
            .any(|pair| pair[0].0.call_bci == pair[1].0.call_bci)
        {
            return Ok(None);
        }
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(relation_sort).unwrap_or(u64::MAX),
        )?;
        budget.charge(
            CountedBudgetDimension::IrItems,
            u64::try_from(indexed_get_casts.len()).unwrap_or(u64::MAX),
        )?;
        indexed_get_casts.sort_by_key(|(_, cast_bci, _)| *cast_bci);
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(indexed_get_casts.len()).unwrap_or(u64::MAX),
        )?;
        let relation_bytes = indexed_get_casts.iter().fold(0usize, |n, (key, _, ty)| {
            n.saturating_add(key.target.owner().len())
                .saturating_add(key.target.name().len())
                .saturating_add(key.target.descriptor().len())
                .saturating_add(ty.spell().len())
        });
        budget.poll()?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(indexed_get_casts.len().saturating_add(relation_bytes))
                .unwrap_or(u64::MAX),
        )?;
        budget.charge(
            CountedBudgetDimension::IrItems,
            u64::try_from(indexed_get_casts.len()).unwrap_or(u64::MAX),
        )?;
        let relations: Vec<_> = indexed_get_casts
            .iter()
            .map(|(key, cast_bci, ty)| (key, *cast_bci, ty))
            .collect();
        let Some(ast_matches) =
            jarde_java::report::class_source_exact_cast_call_children(ast, &relations, budget)
                .map_err(|stop| {
                    enum_projection_stop_error(
                        stop,
                        "List.get cast AST relations",
                        "generic_call_ast_missing",
                    )
                })?
        else {
            return Ok(None);
        };
        if ast_matches.len() != indexed_get_casts.len()
            || ast_matches.iter().any(|matched| !matched)
        {
            return Ok(None);
        }
    }

    // equals(Object) is not a List receiver call, so it is inventoried separately from the
    // required closed List-use set. Any incomplete optional AST match simply keeps its wrapper.
    let equals_work = u64::try_from(invokes.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(4);
    budget.charge(CountedBudgetDimension::AnalysisSteps, equals_work)?;
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(invokes.len()).unwrap_or(u64::MAX),
    )?;
    let mut equals_key_bytes = 0u64;
    for invoke in invokes {
        budget.poll()?;
        if invoke.physical_caller == *caller
            && invoke.owner.as_slice() == b"java/lang/String"
            && invoke.name.as_slice() == b"equals"
            && invoke.descriptor.as_slice() == b"(Ljava/lang/Object;)Z"
            && invoke.opcode == 0xb6
            && invoke.ssa_operands_complete
        {
            equals_key_bytes = equals_key_bytes
                .saturating_add(u64::try_from(invoke.owner.len()).unwrap_or(u64::MAX))
                .saturating_add(u64::try_from(invoke.name.len()).unwrap_or(u64::MAX))
                .saturating_add(u64::try_from(invoke.descriptor.len()).unwrap_or(u64::MAX));
        }
    }
    budget.charge(CountedBudgetDimension::OutputBytes, equals_key_bytes)?;
    let mut equals_keys = Vec::<(ClassSourceInvokeKey, usize)>::new();
    for (invoke_index, invoke) in invokes.iter().enumerate() {
        budget.poll()?;
        if invoke.physical_caller == *caller
            && invoke.owner.as_slice() == b"java/lang/String"
            && invoke.name.as_slice() == b"equals"
            && invoke.descriptor.as_slice() == b"(Ljava/lang/Object;)Z"
            && invoke.opcode == 0xb6
            && invoke.ssa_operands_complete
        {
            if let Some(key) = generic_call_ast_key(invoke) {
                equals_keys.push((key, invoke_index));
            }
        }
    }
    let sort_work = equals_keys.len().saturating_mul(
        (usize::BITS as usize)
            .saturating_sub(equals_keys.len().max(1).leading_zeros() as usize)
            .saturating_add(1),
    );
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(sort_work.saturating_add(equals_keys.len())).unwrap_or(u64::MAX),
    )?;
    equals_keys.sort_by_key(|(key, _)| key.call_bci);
    for (index, (key, invoke_index)) in equals_keys.iter().enumerate() {
        budget.poll()?;
        let duplicate_call_bci = (index > 0 && equals_keys[index - 1].0.call_bci == key.call_bci)
            || equals_keys
                .get(index + 1)
                .is_some_and(|next| next.0.call_bci == key.call_bci);
        if duplicate_call_bci {
            continue;
        }
        let one_key = std::slice::from_ref(key);
        let Some(sites) = jarde_java::report::class_source_invoke_ast_sites(ast, one_key, budget)
            .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "optional String equals AST site",
                "generic_call_ast_missing",
            )
        })?
        else {
            // Optional sites fail independently: a missing/mismatched AST node leaves this
            // invocation wrapped while other proved equals calls keep their edits.
            continue;
        };
        let identity_bytes = jarde_java::report::class_source_method_identity_work(caller, budget)
            .map_err(|stop| {
                enum_projection_stop_error(
                    stop,
                    "optional String equals identity",
                    "generic_call_ast_missing",
                )
            })?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(
                sites
                    .len()
                    .saturating_mul(identity_bytes.saturating_add(2))
                    .saturating_add(1),
            )
            .unwrap_or(u64::MAX),
        )?;
        let mut matching_sites = sites
            .iter()
            .filter(|site| site.key.call_bci == key.call_bci && site.caller == *caller);
        let Some(site) = matching_sites.next() else {
            continue;
        };
        if matching_sites.next().is_some() {
            continue;
        }
        let invoke = &invokes[*invoke_index];
        let Some(receiver) = site.receiver.as_ref() else {
            continue;
        };
        let Some(receiver_name) = receiver.direct_local_name.as_deref() else {
            continue;
        };
        if receiver.shape != Shape::Local
            || receiver.primary.method.as_ref() != Some(caller)
            || !matches!(
                &receiver.presented_type,
                Some(Type::Reference(name)) if name == "java.lang.String"
            )
        {
            continue;
        }
        let Some(receiver_fact) = invoke.receiver_value_fact.as_ref() else {
            continue;
        };
        let Some(receiver_type) = prove_generic_list_element_local_argument(
            ast,
            receiver_name,
            receiver.primary.bci,
            receiver_fact,
            invoke,
            caller,
            invokes,
            body_values,
            &formals,
            &declarations,
            &assignments,
            &foreach_sites,
            &selected_reads,
            budget,
        )?
        else {
            continue;
        };
        if !matches!(receiver_type, Type::Reference(name) if name == "java.lang.String") {
            continue;
        }
        let Some(argument) = site.arguments.first() else {
            continue;
        };
        let Some(wrapper) = argument.presentation_wrapper.as_ref() else {
            continue;
        };
        if !matches!(&wrapper.ty, Type::Reference(name) if name == "java.lang.Object")
            || site.arguments.len() != 1
            || argument.primary.bci != wrapper.child.primary.bci
            || wrapper.child.primary.method.as_ref() != Some(caller)
            || wrapper.child.shape != Shape::Local
        {
            continue;
        }
        let Some(argument_name) = wrapper.child.direct_local_name.as_deref() else {
            continue;
        };
        let [argument_fact] = invoke.argument_value_facts.as_slice() else {
            continue;
        };
        if prove_generic_list_element_local_argument(
            ast,
            argument_name,
            wrapper.child.primary.bci,
            argument_fact,
            invoke,
            caller,
            invokes,
            body_values,
            &formals,
            &declarations,
            &assignments,
            &foreach_sites,
            &selected_reads,
            budget,
        )?
        .is_some()
        {
            push_generic_optional_removal(&mut removals, site, 0, budget)?;
        }
    }
    let mut proved = Vec::new();
    for (slot, actual_types) in type_uses {
        if actual_types.windows(2).any(|pair| pair[0] != pair[1]) {
            return Ok(None);
        }
        proved.push((slot, actual_types));
    }
    Ok(Some((proved, removals)))
}

#[allow(clippy::too_many_arguments)]
pub fn prove_generic_call_site_from_ast(
    invoke: &class_source::SameClassInvokeUse,
    caller_contract: &class_source::SameClassGenericMethodContract,
    callee_contract: &class_source::SameClassGenericMethodContract,
    caller_ast: &jarde_java::report::ClassSourceMethodAst,
    method_header: &MemberHeader,
    caller_init_record: Option<&jarde_java::init::InitRecord>,
    field_uses: &[class_source::SameClassFieldUse],
    body_values: &[class_source::SameClassBodyValueSite],
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    class_scope: &[jarde_reader::signature::TypeParameterErasure],
    class_parameters: &[jarde_reader::signature::TypeParameter],
    fields: &[MemberHeader],
    invokes: &[class_source::SameClassInvokeUse],
    nested_call_sources: &[class_source::SameClassGenericArgumentSource],
    budget: &mut Budget,
) -> Result<Option<GenericCallSiteProof>> {
    use jarde_java::report::{ClassSourceInvokeFieldTarget, class_source_invoke_ast_sites};
    let Some(key) = generic_call_ast_key(invoke) else {
        return Ok(None);
    };
    let Some(sites) = class_source_invoke_ast_sites(caller_ast, std::slice::from_ref(&key), budget)
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "same-class generic invocation AST projection",
                "generic_call_ast_missing",
            )
        })?
    else {
        return Ok(None);
    };
    let Some(site) = sites.first() else {
        return Ok(None);
    };
    let Some(formals) = jarde_java::report::class_source_method_formal_names(caller_ast, budget)
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "same-class generic formal source proof",
                "generic_call_ast_missing",
            )
        })?
    else {
        return Ok(None);
    };
    let Some(_locals) = jarde_java::report::class_source_local_declarations(caller_ast, budget)
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "same-class generic local source proof",
                "generic_call_ast_missing",
            )
        })?
    else {
        return Ok(None);
    };
    let field_targets = field_uses
        .iter()
        .filter(|site| site.physical_method == invoke.physical_caller && site.opcode == 0xb5)
        .map(|site| ClassSourceInvokeFieldTarget {
            write_bci: site.bci,
            owner: jarde_reader::model::JvmBytes(site.owner.clone()),
            name: jarde_reader::model::JvmBytes(site.name.clone()),
            descriptor: jarde_reader::model::JvmBytes(site.descriptor.clone()),
        })
        .collect::<Vec<_>>();
    let body_consumers = jarde_java::report::class_source_method_body_consumers(caller_ast, budget)
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "same-class generic caller scope proof",
                "generic_call_ast_missing",
            )
        })?;
    let Some(body) = body_consumers else {
        return Ok(None);
    };
    if !generic_call_site_avoids_catch_shadow(&body, site, invoke.bci) {
        return Ok(None);
    }
    prove_generic_call_source_site(
        invoke,
        site,
        caller_contract,
        callee_contract,
        class_scope,
        class_parameters,
        class_internal,
        class_superclass,
        &formals,
        method_header,
        caller_init_record.or(body.init_record.as_ref()),
        body_values,
        fields,
        &field_targets,
        caller_ast,
        invokes,
        nested_call_sources,
        budget,
    )
}

pub fn generic_call_site_avoids_catch_shadow(
    body: &jarde_java::report::ClassSourceMethodBodyConsumers,
    site: &jarde_java::report::ClassSourceInvokeAstSite,
    call_bci: u32,
) -> bool {
    let shadows =
        |scopes: &[jarde_java::report::ClassSourceAstCatchScope],
         expressions: &[&jarde_java::report::ClassSourceInvokeAstExpression]| {
            expressions.iter().any(|expression| {
                expression.primary.bci == call_bci
                    || expression
                        .derived
                        .iter()
                        .any(|anchor| anchor.bci == call_bci)
            }) && scopes.iter().any(|scope| {
                site.arguments.iter().any(|argument| {
                    argument.direct_local_name.as_deref() == Some(scope.local_name.as_str())
                })
            })
        };
    !body
        .returns
        .iter()
        .chain(&body.conditions)
        .chain(&body.throws)
        .chain(&body.invocation_statements)
        .any(|consumer| shadows(&consumer.catch_scopes, &[&consumer.expression]))
        && !body.field_writes.iter().any(|consumer| {
            let mut expressions = vec![&consumer.value];
            if let Some(receiver) = &consumer.receiver {
                expressions.push(receiver);
            }
            shadows(&consumer.catch_scopes, &expressions)
        })
        && !body.constructor_calls.iter().any(|consumer| {
            shadows(
                &consumer.catch_scopes,
                &consumer.arguments.iter().collect::<Vec<_>>(),
            )
        })
}

pub fn generic_call_load_source<'a>(
    body_values: &'a [class_source::SameClassBodyValueSite],
    value: &class_source::SameClassInvokeValueFact,
    consumer_bci: u32,
) -> Option<(u32, &'a (u16, class_source::SameClassInvokeValueFact))> {
    use jarde_jvm::method_ir::Definition;

    let Definition::Instruction { bci: load_bci, .. } = &value.definition else {
        return None;
    };
    let load_bci = *load_bci;
    if value.replaced_by.is_some() || value.uses.as_slice() != [Some(consumer_bci)] {
        return None;
    }
    let mut sites = body_values.iter().filter(|site| site.bci == load_bci);
    let site = sites.next()?;
    if sites.next().is_some()
        || !matches!(site.opcode, 0x15..=0x19 | 0x1a..=0x2d)
        || site.stack_writes.len() != 1
    {
        return None;
    }
    let written = &site.stack_writes[0];
    if written.value != value.value
        || written.definition != value.definition
        || written.uses != value.uses
        || written.replaced_by != value.replaced_by
    {
        return None;
    }
    let mut local_reads = site.local_reads.iter();
    let read = local_reads.next()?;
    if local_reads.next().is_some() || !read.1.uses.contains(&Some(load_bci)) {
        return None;
    }
    Some((load_bci, read))
}

pub fn generic_call_local_definition_matches_declaration(
    body_values: &[class_source::SameClassBodyValueSite],
    local_read: &class_source::SameClassInvokeValueFact,
    local_slot: u16,
    declaration_bci: u32,
    load_bci: u32,
) -> bool {
    use jarde_jvm::method_ir::Definition;

    let Definition::Instruction {
        bci: definition_bci,
        ..
    } = &local_read.definition
    else {
        return false;
    };
    if *definition_bci < declaration_bci || *definition_bci >= load_bci {
        return false;
    }
    let mut sites = body_values
        .iter()
        .filter(|site| site.bci == *definition_bci);
    let Some(site) = sites.next() else {
        return false;
    };
    if sites.next().is_some() {
        return false;
    }
    let mut writes = site.local_writes.iter().filter(|(slot, fact)| {
        *slot == local_slot && fact == local_read && fact.replaced_by.is_none()
    });
    writes.next().is_some() && writes.next().is_none()
}

pub fn generic_call_load_matches_formal(
    body_values: &[class_source::SameClassBodyValueSite],
    value: &class_source::SameClassInvokeValueFact,
    slot: u16,
    consumer_bci: u32,
) -> bool {
    use jarde_jvm::method_ir::{Definition, Slot};

    let Some((_, (read_slot, read))) = generic_call_load_source(body_values, value, consumer_bci)
    else {
        return false;
    };
    *read_slot == slot
        && read.replaced_by.is_none()
        && matches!(read.definition, Definition::Entry { slot: Slot::Local(actual), .. } if actual == slot)
}

pub fn generic_call_null_matches(
    body_values: &[class_source::SameClassBodyValueSite],
    value: &class_source::SameClassInvokeValueFact,
    expression: &jarde_java::report::ClassSourceInvokeAstExpression,
    consumer_bci: u32,
) -> bool {
    use jarde_jvm::method_ir::Definition;

    if !expression.null_literal
        || !matches!(value.definition, Definition::Instruction { bci, .. } if bci == expression.primary.bci)
        || value.replaced_by.is_some()
        || value.uses.as_slice() != [Some(consumer_bci)]
    {
        return false;
    }
    let mut sites = body_values
        .iter()
        .filter(|site| site.bci == expression.primary.bci && site.opcode == 0x01);
    let Some(site) = sites.next() else {
        return false;
    };
    sites.next().is_none()
        && site.stack_writes.len() == 1
        && site.stack_writes[0].value == value.value
        && site.stack_writes[0].definition == value.definition
        && site.stack_writes[0].uses == value.uses
        && site.stack_writes[0].replaced_by.is_none()
}

pub fn generic_call_receiver_is_initialized_this(
    invoke: &class_source::SameClassInvokeUse,
    receiver: &class_source::SameClassInvokeValueFact,
    caller_header: &MemberHeader,
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    init_record: Option<&jarde_java::init::InitRecord>,
    body_values: &[class_source::SameClassBodyValueSite],
    invokes: &[class_source::SameClassInvokeUse],
) -> bool {
    generic_call_receiver_is_initialized_this_at(
        invoke.bci,
        &invoke.physical_caller,
        receiver,
        caller_header,
        class_internal,
        class_superclass,
        init_record,
        body_values,
        invokes,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn generic_call_receiver_is_initialized_this_at(
    consumer_bci: u32,
    caller_method: &PhysicalMethodId,
    receiver: &class_source::SameClassInvokeValueFact,
    caller_header: &MemberHeader,
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    init_record: Option<&jarde_java::init::InitRecord>,
    body_values: &[class_source::SameClassBodyValueSite],
    invokes: &[class_source::SameClassInvokeUse],
) -> bool {
    use jarde_jvm::method_ir::{Definition, Slot};

    let Some((load_bci, (read_slot, read))) =
        generic_call_load_source(body_values, receiver, consumer_bci)
    else {
        return false;
    };
    if caller_header.name.raw().0 != b"<init>" {
        return caller_header.access_flags & 0x0008 == 0
            && *read_slot == 0
            && read.replaced_by.is_none()
            && matches!(
                read.definition,
                Definition::Entry {
                    slot: Slot::Local(0),
                    ..
                }
            );
    }
    let Some(init) = init_record else {
        return false;
    };
    let Some(init_bci) = init.bci else {
        return false;
    };
    if !init.presented
        || init.target != Some(jarde_java::ast::ConstructorTarget::Super)
        || !init.declares_raw_class(class_internal)
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || init.class.as_deref() != Some("java/lang/Object")
        || !matches!(
            read.definition,
            Definition::Instruction { bci, .. } if bci == init_bci
        )
        || *read_slot != 0
        || read.replaced_by.is_some()
        || !read.uses.contains(&Some(load_bci))
    {
        return false;
    }
    let alias_write = body_values
        .iter()
        .filter(|site| site.bci == init_bci && site.opcode == 0xb7)
        .flat_map(|site| site.local_writes.iter())
        .filter(|(slot, fact)| {
            *slot == 0
                && fact.value == read.value
                && fact.replaced_by.is_none()
                && matches!(fact.definition, Definition::Instruction { bci, .. } if bci == init_bci)
        })
        .count()
        == 1;
    alias_write
        && invokes.iter().any(|initialization| {
            initialization.physical_caller == *caller_method
                && initialization.bci == init_bci
                && initialization.opcode == 0xb7
                && initialization.owner == b"java/lang/Object"
                && initialization.name == b"<init>"
                && initialization.descriptor == b"()V"
                && initialization.constructor_receiver
                    == Some(class_source::SameClassConstructorReceiver::UninitializedThis)
        })
}

/// Proves one physical call and every direct AST use of its result from the final caller and
/// callee Signatures. The returned boolean says that this call is the caller's returned source.
#[allow(clippy::too_many_arguments)]
pub fn prove_generic_call_source_site(
    invoke: &class_source::SameClassInvokeUse,
    ast_site: &jarde_java::report::ClassSourceInvokeAstSite,
    caller: &class_source::SameClassGenericMethodContract,
    callee: &class_source::SameClassGenericMethodContract,
    class_scope: &[jarde_reader::signature::TypeParameterErasure],
    class_parameters: &[jarde_reader::signature::TypeParameter],
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    formal_names: &[jarde_java::report::ClassSourceMethodFormalName],
    caller_header: &MemberHeader,
    caller_init_record: Option<&jarde_java::init::InitRecord>,
    body_values: &[class_source::SameClassBodyValueSite],
    fields: &[MemberHeader],
    field_targets: &[jarde_java::report::ClassSourceInvokeFieldTarget],
    caller_ast: &jarde_java::report::ClassSourceMethodAst,
    invokes: &[class_source::SameClassInvokeUse],
    nested_call_sources: &[class_source::SameClassGenericArgumentSource],
    budget: &mut Budget,
) -> Result<Option<GenericCallSiteProof>> {
    macro_rules! reject_generic_call_site {
        () => {{
            return Ok(None);
        }};
    }
    use jarde_java::facts::InvokeKind;
    use jarde_java::report::ClassSourceInvokeResultUseKind as Use;
    use jarde_jvm::method_ir::Definition;
    use jarde_reader::signature::SignatureType;

    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;

    let target_owner = invoke.owner.as_slice();
    if !invoke.ssa_operands_complete
        || ast_site.caller != invoke.physical_caller
        || ast_site.key.call_bci != invoke.bci
        || ast_site.key.opcode != invoke.opcode
        || ast_site.key.target.raw_owner().0.as_slice() != target_owner
        || ast_site.key.target.raw_name().0.as_slice() != invoke.name
        || ast_site.key.target.raw_descriptor().0.as_slice() != invoke.descriptor
        || caller.method != invoke.physical_caller
        || callee.method.name.0 != invoke.name
        || callee.method.descriptor.0 != invoke.descriptor
        || target_owner != class_internal
    {
        reject_generic_call_site!();
    }
    let is_static = invoke.opcode == 0xb8;
    if ast_site.key.target.kind()
        != match invoke.opcode {
            0xb6 => InvokeKind::Virtual,
            0xb7 => InvokeKind::Special,
            0xb8 => InvokeKind::Static,
            0xb9 => InvokeKind::Interface,
            _ => return Ok(None),
        }
    {
        reject_generic_call_site!();
    }
    let descriptor = descriptor_facts(&caller.method.descriptor.0, DescriptorKind::Method)?;
    let callee_descriptor = descriptor_facts(&callee.method.descriptor.0, DescriptorKind::Method)?;
    let caller_is_static = caller_header.access_flags & 0x0008 != 0;
    let formal_slots = jarde_jvm::method_ir::parameter_positions(&descriptor, caller_is_static);
    let Some(formal_slots) = formal_slots else {
        reject_generic_call_site!();
    };
    if caller.parameters.len() != formal_slots.len()
        || ast_site.arguments.len() != callee.parameters.len()
        || invoke.argument_value_facts.len() != callee.parameters.len()
    {
        reject_generic_call_site!();
    }
    let formal_by_name: std::collections::HashMap<&str, u16> = formal_names
        .iter()
        .map(|formal| (formal.name.as_str(), formal.slot))
        .collect();
    if formal_by_name.len() != formal_names.len() {
        reject_generic_call_site!();
    }
    if is_static {
        if ast_site.receiver.is_some() || invoke.receiver_value.is_some() {
            reject_generic_call_site!();
        }
    } else {
        let Some(receiver_fact) = invoke.receiver_value_fact.as_ref() else {
            reject_generic_call_site!();
        };
        if ast_site.receiver.is_none() {
            // An unqualified instance call has no receiver expression in this AST. Its
            // implicit receiver is still required to be the physical initialized `this`.
            if !generic_call_receiver_is_initialized_this(
                invoke,
                receiver_fact,
                caller_header,
                class_internal,
                class_superclass,
                caller_init_record,
                body_values,
                invokes,
            ) {
                reject_generic_call_site!();
            }
        } else {
            let receiver = ast_site.receiver.as_ref().expect("checked above");
            let receiver_slot = receiver
                .direct_local_name
                .as_deref()
                .and_then(|name| formal_by_name.get(name).copied());
            if receiver.direct_local_name.as_deref() == Some("this") {
                if receiver.primary.method.as_ref() != Some(&caller.method)
                    || !generic_call_receiver_is_initialized_this(
                        invoke,
                        receiver_fact,
                        caller_header,
                        class_internal,
                        class_superclass,
                        caller_init_record,
                        body_values,
                        invokes,
                    )
                {
                    reject_generic_call_site!();
                }
            } else {
                let Some(slot) = receiver_slot else {
                    reject_generic_call_site!();
                };
                if !generic_call_load_matches_formal(body_values, receiver_fact, slot, invoke.bci) {
                    reject_generic_call_site!();
                }
                let Some(position) = formal_slots.iter().position(|formal| *formal == slot) else {
                    reject_generic_call_site!();
                };
                if !same_class_parameterized_receiver(
                    &caller.parameters[position],
                    class_internal,
                    class_scope,
                    &caller.method_parameters,
                    budget,
                )? {
                    reject_generic_call_site!();
                }
            }
        }
    }

    let mut actuals = Vec::with_capacity(callee.parameters.len());
    let mut presentation_removals = Vec::new();
    for (position, (outer_expression, value_fact)) in ast_site
        .arguments
        .iter()
        .zip(&invoke.argument_value_facts)
        .enumerate()
    {
        budget.poll()?;
        let expression = if let Some(wrapper) = &outer_expression.presentation_wrapper {
            let Some(required) = callee_descriptor
                .parameters()
                .get(position)
                .and_then(|parameter| parameter.bytes(&callee.method.descriptor.0))
                .and_then(generic_call_reference_type_from_descriptor)
            else {
                reject_generic_call_site!();
            };
            if wrapper.ty != required {
                reject_generic_call_site!();
            }
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            presentation_removals.push(
                jarde_java::report::ClassSourceInvokeArgumentPresentationCast {
                    call_bci: ast_site.key.call_bci,
                    opcode: ast_site.key.opcode,
                    target: ast_site.key.target.clone(),
                    argument_index: position,
                },
            );
            wrapper.child.as_ref()
        } else {
            outer_expression
        };
        let (source_type, origin) = if expression.null_literal {
            if !callee.method_parameters.is_empty()
                || matches!(callee.parameters[position], SignatureType::Base(_))
                || !generic_call_null_matches(body_values, value_fact, expression, invoke.bci)
            {
                reject_generic_call_site!();
            }
            (
                callee.parameters[position].clone(),
                class_source::SameClassGenericArgumentOrigin::NullLiteral,
            )
        } else if matches!(
            expression.shape,
            jarde_java::report::ClassSourceAstExpressionShape::Local
        ) {
            if expression.primary.method.as_ref() != Some(&caller.method) {
                reject_generic_call_site!();
            }
            let Some(local_name) = expression.direct_local_name.as_deref() else {
                reject_generic_call_site!();
            };
            let Some(slot) = formal_by_name.get(local_name).copied() else {
                reject_generic_call_site!();
            };
            if !generic_call_load_matches_formal(body_values, value_fact, slot, invoke.bci) {
                reject_generic_call_site!();
            }
            let Some(caller_position) = formal_slots.iter().position(|formal| *formal == slot)
            else {
                reject_generic_call_site!();
            };
            (
                caller.parameters[caller_position].clone(),
                class_source::SameClassGenericArgumentOrigin::Formal { slot },
            )
        } else {
            let jarde_java::report::ClassSourceAstExpressionShape::Call { argument_count, .. } =
                &expression.shape
            else {
                reject_generic_call_site!();
            };
            let producer_bci = expression.primary.bci;
            let Some(producer) = invokes.iter().find(|producer| {
                producer.physical_caller == caller.method && producer.bci == producer_bci
            }) else {
                reject_generic_call_site!();
            };
            let Some(producer_key) = generic_call_ast_key(producer) else {
                reject_generic_call_site!();
            };
            let Some(source) = nested_call_sources.iter().find(|source| {
                source.caller == caller.method
                    && matches!(
                        &source.origin,
                        class_source::SameClassGenericArgumentOrigin::NestedCall {
                            bci, opcode, owner, name, descriptor,
                        } if *bci == producer.bci
                            && *opcode == producer.opcode
                            && *owner == producer.owner
                            && *name == producer.name
                            && *descriptor == producer.descriptor
                    )
            }) else {
                reject_generic_call_site!();
            };
            if expression.primary.method.as_ref() != Some(&caller.method)
                || usize::try_from(*argument_count).ok()
                    != Some(producer.argument_value_facts.len())
                || value_fact.replaced_by.is_some()
                || value_fact.uses.as_slice() != [Some(invoke.bci)]
                || !matches!(value_fact.definition, Definition::Instruction { bci, .. } if bci == producer.bci)
                || source.origin
                    != (class_source::SameClassGenericArgumentOrigin::NestedCall {
                        bci: producer_key.call_bci,
                        opcode: producer_key.opcode,
                        owner: producer.owner.clone(),
                        name: producer.name.clone(),
                        descriptor: producer.descriptor.clone(),
                    })
            {
                reject_generic_call_site!();
            }
            (source.source_type.clone(), source.origin.clone())
        };
        actuals.push(class_source::SameClassGenericArgumentSource {
            caller: caller.method.clone(),
            origin,
            source_type,
            caller_method_parameters: caller.method_parameters.clone(),
        });
    }
    let Some(substitution) = class_source::prove_same_class_generic_substitution(
        class_scope,
        class_parameters,
        callee,
        &actuals,
        budget,
    )?
    else {
        reject_generic_call_site!();
    };

    let result_uses = if callee.result.is_some() {
        let Some(value_fact) = invoke.result_value_fact.as_ref() else {
            reject_generic_call_site!();
        };
        if value_fact.replaced_by.is_some()
            || !matches!(value_fact.definition, Definition::Instruction { bci, .. } if bci == invoke.bci)
        {
            reject_generic_call_site!();
        }
        let Some(uses) = jarde_java::report::class_source_invoke_result_uses(
            caller_ast,
            &ast_site.key,
            field_targets,
            budget,
        )
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "same-class generic call consumer proof",
                "generic_call_ast_missing",
            )
        })?
        else {
            reject_generic_call_site!();
        };
        if uses.is_empty() || value_fact.uses.iter().any(Option::is_none) {
            reject_generic_call_site!();
        }
        let mut ast_use_bcis = Vec::with_capacity(uses.len());
        let mut direct_return = false;
        for usage in &uses {
            budget.poll()?;
            let bci = match &usage.kind {
                Use::DirectReturn { consumer_bci } => {
                    let Some(result) = caller.result.as_ref() else {
                        reject_generic_call_site!();
                    };
                    let mut consumers = body_values.iter().filter(|site| site.bci == *consumer_bci);
                    let Some(consumer_site) = consumers.next() else {
                        reject_generic_call_site!();
                    };
                    if consumers.next().is_some()
                        || consumer_site.stack_reads.len() != 1
                        || consumer_site.stack_reads[0].1.value != value_fact.value
                        || consumer_site.stack_reads[0].1.replaced_by.is_some()
                        || consumer_site.stack_reads[0].1.uses.as_slice() != [Some(*consumer_bci)]
                    {
                        reject_generic_call_site!();
                    }
                    if !class_source::same_class_generic_result_matches(
                        class_scope,
                        &caller.method_parameters,
                        result,
                        callee,
                        &substitution,
                        budget,
                    )? {
                        reject_generic_call_site!();
                    }
                    direct_return = true;
                    *consumer_bci
                }
                Use::FieldWrite { target } => {
                    if target.owner.0.as_slice() != class_internal {
                        reject_generic_call_site!();
                    }
                    let mut consumers = body_values
                        .iter()
                        .filter(|site| site.bci == target.write_bci && site.opcode == 0xb5);
                    let Some(consumer_site) = consumers.next() else {
                        reject_generic_call_site!();
                    };
                    if consumers.next().is_some()
                        || consumer_site.stack_reads.len() != 2
                        || consumer_site.stack_reads[1].1.value != value_fact.value
                        || consumer_site.stack_reads[1].1.replaced_by.is_some()
                        || consumer_site.stack_reads[1].1.uses.as_slice()
                            != [Some(target.write_bci)]
                    {
                        reject_generic_call_site!();
                    }
                    let matching = fields.iter().enumerate().filter(|(_, field)| {
                        field.name.raw().0 == target.name.0.as_slice()
                            && field.descriptor.raw().0 == target.descriptor.0.as_slice()
                    });
                    let mut matching = matching;
                    let Some((_, field)) = matching.next() else {
                        reject_generic_call_site!();
                    };
                    if matching.next().is_some() || target.owner.0.as_slice() != class_internal {
                        reject_generic_call_site!();
                    }
                    let Some(result_type) = class_source::same_class_generic_substituted_result(
                        callee,
                        &substitution,
                        budget,
                    )?
                    else {
                        reject_generic_call_site!();
                    };
                    if class_source::same_class_generic_source_assignable_to_descriptor(
                        &result_type,
                        &caller.method_parameters,
                        &field.descriptor.raw().0,
                        class_scope,
                        class_parameters,
                        budget,
                    )? != Some(true)
                    {
                        reject_generic_call_site!();
                    }
                    target.write_bci
                }
                Use::DirectCallArgument {
                    consumer: Some(consumer_key),
                    argument_index,
                } => {
                    if consumer_key.target.raw_owner().0.as_slice() != class_internal
                        || consumer_key.call_bci <= invoke.bci
                    {
                        reject_generic_call_site!();
                    }
                    let Some(consumer) = invokes.iter().find(|consumer| {
                        consumer.physical_caller == invoke.physical_caller
                            && generic_call_ast_key(consumer).as_ref() == Some(consumer_key)
                    }) else {
                        reject_generic_call_site!();
                    };
                    let Some(argument_fact) = consumer.argument_value_facts.get(*argument_index)
                    else {
                        reject_generic_call_site!();
                    };
                    if argument_fact.value != value_fact.value {
                        reject_generic_call_site!();
                    }
                    if !matches!(
                        usage.expression.shape,
                        jarde_java::report::ClassSourceAstExpressionShape::Call { .. }
                    ) || usage.expression.primary.method.as_ref()
                        != Some(&invoke.physical_caller)
                        || usage.expression.primary.bci != invoke.bci
                    {
                        reject_generic_call_site!();
                    }
                    let Some(consumer_sites) = jarde_java::report::class_source_invoke_ast_sites(
                        caller_ast,
                        std::slice::from_ref(consumer_key),
                        budget,
                    )
                    .map_err(|stop| {
                        enum_projection_stop_error(
                            stop,
                            "nested generic call AST projection",
                            "generic_call_ast_missing",
                        )
                    })?
                    else {
                        reject_generic_call_site!();
                    };
                    let Some(consumer_site) = consumer_sites.first() else {
                        reject_generic_call_site!();
                    };
                    let Some(argument) = consumer_site.arguments.get(*argument_index) else {
                        reject_generic_call_site!();
                    };
                    let argument = argument
                        .presentation_wrapper
                        .as_ref()
                        .map_or(argument, |wrapper| wrapper.child.as_ref());
                    if !matches!(
                        &argument.shape,
                        jarde_java::report::ClassSourceAstExpressionShape::Call { .. }
                    ) || argument.primary.bci != invoke.bci
                        || argument.primary.method.as_ref() != Some(&invoke.physical_caller)
                    {
                        reject_generic_call_site!();
                    }
                    consumer_key.call_bci
                }
                Use::DirectCallArgument { .. } | Use::LocalStore { .. } | Use::Other { .. } => {
                    reject_generic_call_site!();
                }
            };
            ast_use_bcis.push(Some(bci));
        }
        ast_use_bcis.sort_unstable();
        let mut ssa_use_bcis = value_fact.uses.clone();
        ssa_use_bcis.sort_unstable();
        if ast_use_bcis != ssa_use_bcis {
            reject_generic_call_site!();
        }
        Some(direct_return)
    } else {
        if invoke.result_value.is_some() {
            reject_generic_call_site!();
        }
        None
    };
    let direct_return = result_uses.unwrap_or(false);
    let result_type = if callee.result.is_some() {
        class_source::same_class_generic_substituted_result(callee, &substitution, budget)?
    } else {
        None
    };
    Ok(Some(GenericCallSiteProof {
        direct_return,
        result_type,
        presentation_removals,
    }))
}

pub fn same_class_parameterized_receiver(
    source: &jarde_reader::signature::SignatureType,
    class_internal: &[u8],
    class_scope: &[jarde_reader::signature::TypeParameterErasure],
    method_parameters: &[jarde_reader::signature::TypeParameter],
    budget: &mut Budget,
) -> Result<bool> {
    use jarde_reader::signature::{SignatureType, TypeArgument};

    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    let SignatureType::Class(class) = source else {
        return Ok(false);
    };
    if class.segments.len() != 1 || class.segments[0].binary_name != class_internal {
        return Ok(false);
    }
    let arguments = &class.segments[0].arguments;
    if arguments.len() != class_scope.len() {
        return Ok(false);
    }
    for (argument, variable) in arguments.iter().zip(class_scope) {
        budget.poll()?;
        if method_parameters
            .iter()
            .any(|method| method.name == variable.name)
            || !matches!(argument, TypeArgument::Exact(SignatureType::TypeVariable(name)) if name == &variable.name)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Certify the narrow complete body shapes accepted by generic-call Signature publication. Every
/// local read must be a unique same-method SSA load from a physical entry slot; the proof retains
/// each return source so a caller cannot publish a Signature from a presence bit alone.
pub fn generic_exception_new_receiver_matches(
    body_values: &[class_source::SameClassBodyValueSite],
    allocation_bci: u32,
    constructor_bci: u32,
    receiver: &class_source::SameClassInvokeValueFact,
) -> bool {
    use jarde_jvm::method_ir::Definition;

    let mut allocations = body_values
        .iter()
        .filter(|site| site.bci == allocation_bci && site.opcode == 0xbb);
    let Some(allocation) = allocations.next() else {
        return false;
    };
    if allocations.next().is_some() {
        return false;
    }
    let expected_allocation_consumer = match receiver.definition {
        Definition::Instruction { bci, .. } if bci != allocation_bci => bci,
        _ => constructor_bci,
    };
    let mut allocation_writes = allocation.stack_writes.iter().filter(|write| {
        matches!(write.definition, Definition::Instruction { bci, .. } if bci == allocation_bci)
            && write.replaced_by.is_none()
            && write.uses.as_slice() == [Some(expected_allocation_consumer)]
    });
    let Some(allocation_value) = allocation_writes.next() else {
        return false;
    };
    if allocation_writes.next().is_some() {
        return false;
    }
    if receiver.value == allocation_value.value
        && receiver.definition == allocation_value.definition
        && receiver.replaced_by.is_none()
        && receiver.uses.as_slice() == [Some(constructor_bci)]
    {
        return true;
    }
    let Definition::Instruction { bci: dup_bci, .. } = receiver.definition else {
        return false;
    };
    let mut dup_sites = body_values
        .iter()
        .filter(|site| site.bci == dup_bci && site.opcode == 0x59);
    let Some(dup) = dup_sites.next() else {
        return false;
    };
    if dup_sites.next().is_some() || dup.stack_reads.len() != 1 {
        return false;
    }
    let dup_source = &dup.stack_reads[0].1;
    let mut dup_writes = dup.stack_writes.iter().filter(|write| {
        write.value == receiver.value
            && write.definition == receiver.definition
            && write.replaced_by.is_none()
            && write.uses.as_slice() == [Some(constructor_bci)]
    });
    let Some(_dup_receiver) = dup_writes.next() else {
        return false;
    };
    dup_writes.next().is_none()
        && dup_source.value == allocation_value.value
        && dup_source.definition == allocation_value.definition
        && dup_source.replaced_by.is_none()
        && dup_source.uses.as_slice() == [Some(dup_bci)]
        && allocation_value.uses.as_slice() == [Some(dup_bci)]
}

pub fn prove_generic_method_body(
    method: &PhysicalMethodId,
    contract: &class_source::SameClassGenericMethodContract,
    formal_names: &[jarde_java::report::ClassSourceMethodFormalName],
    body: &jarde_java::report::ClassSourceMethodBodyConsumers,
    body_values: &[class_source::SameClassBodyValueSite],
    allocations: &[(u32, Vec<u8>)],
    validated_return_calls: &[class_source::SameClassGenericBodyReturnSource],
    validated_call_bcis: &[u32],
    validated_overload_call_bcis: &[u32],
    field_uses: &[class_source::SameClassFieldUse],
    invokes: &[class_source::SameClassInvokeUse],
    physical_fields: &[MemberHeader],
    class_internal: &[u8],
    class_scope: &[jarde_reader::signature::TypeParameterErasure],
    class_parameters: &[jarde_reader::signature::TypeParameter],
    constructor_candidate: Option<&jarde_java::report::GenericConstructorCandidate>,
    init_record: Option<&jarde_java::init::InitRecord>,
    class_superclass: Option<&[u8]>,
    header: &MemberHeader,
    budget: &mut Budget,
) -> Result<Option<class_source::SameClassGenericMethodBodyProof>> {
    macro_rules! reject_generic_method_body {
        () => {{
            return Ok(None);
        }};
    }
    use jarde_java::report::ClassSourceAstExpressionShape as Shape;
    use jarde_jvm::method_ir::Definition;
    use jarde_reader::signature::SignatureType;

    budget.poll()?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            body.returns
                .len()
                .saturating_add(body.conditions.len())
                .saturating_add(body.throws.len())
                .saturating_add(body.field_writes.len())
                .saturating_add(body.constructor_calls.len())
                .saturating_add(body.invocation_statements.len())
                .saturating_add(body.return_void_bcis.len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    if contract.method != *method {
        reject_generic_method_body!();
    }
    let descriptor = descriptor_facts(&header.descriptor.raw().0, DescriptorKind::Method)?;
    let Some(slots) =
        jarde_jvm::method_ir::parameter_positions(&descriptor, header.access_flags & 0x0008 != 0)
    else {
        reject_generic_method_body!();
    };
    if formal_names.len() != slots.len() || contract.parameters.len() != slots.len() {
        reject_generic_method_body!();
    }
    let names_by_slot = formal_names
        .iter()
        .map(|formal| (formal.slot, formal.name.as_str()))
        .collect::<std::collections::HashMap<_, _>>();
    if names_by_slot.len() != formal_names.len()
        || slots.iter().any(|slot| !names_by_slot.contains_key(slot))
    {
        reject_generic_method_body!();
    }
    for statement in &body.invocation_statements {
        budget.poll()?;
        let Shape::Call { name, .. } = &statement.expression.shape else {
            reject_generic_method_body!();
        };
        if statement.expression.primary.method.as_ref() != Some(method)
            || statement.expression.primary.bci != statement.bci
            || validated_overload_call_bcis
                .iter()
                .filter(|bci| **bci == statement.bci)
                .count()
                != 1
            || invokes
                .iter()
                .filter(|invoke| {
                    invoke.physical_caller == *method
                        && invoke.bci == statement.bci
                        && invoke.owner == class_internal
                        && source_identifier(&invoke.name).as_deref() == Some(name.as_str())
                })
                .count()
                != 1
        {
            reject_generic_method_body!();
        }
    }
    let unique_formal_source =
        |name: &str, expression_bci: u32, consumer_bci: u32| -> Option<(u16, usize)> {
            let formal = formal_names.iter().find(|formal| formal.name == name)?;
            let position = slots.iter().position(|slot| *slot == formal.slot)?;
            let site = body_values
                .iter()
                .filter(|site| site.bci == expression_bci)
                .next()?;
            if body_values
                .iter()
                .filter(|candidate| candidate.bci == expression_bci)
                .count()
                != 1
            {
                return None;
            }
            let mut facts = site
                .stack_writes
                .iter()
                .filter(|fact| fact.uses.as_slice() == [Some(consumer_bci)]);
            let fact = facts.next()?;
            if facts.next().is_some()
                || !generic_call_load_matches_formal(body_values, fact, formal.slot, consumer_bci)
            {
                return None;
            }
            Some((formal.slot, position))
        };
    let mut formal_pairs = Vec::with_capacity(formal_names.len());
    for formal in formal_names {
        budget.poll()?;
        formal_pairs.push((formal.slot, formal.name.clone()));
    }
    let mut constructor_init = None;
    let constructor_call_bci = if header.name.raw().0 == b"<init>" {
        let init_record =
            init_record.or_else(|| constructor_candidate.map(|candidate| &candidate.init));
        let Some(init_record) = init_record else {
            reject_generic_method_body!();
        };
        let Some(init_bci) = init_record.bci else {
            reject_generic_method_body!();
        };
        if init_record.target != Some(jarde_java::ast::ConstructorTarget::Super)
            || !init_record.presented
            || !init_record.declares_raw_class(class_internal)
            || init_record.class.as_deref() != Some("java/lang/Object")
            || class_superclass != Some(b"java/lang/Object".as_slice())
            || body.constructor_calls.len() != 1
        {
            reject_generic_method_body!();
        }
        let call = &body.constructor_calls[0];
        if call.bci != init_bci
            || call.target != jarde_java::ast::ConstructorTarget::Super
            || !call.arguments.is_empty()
            || !call.catch_scopes.is_empty()
        {
            reject_generic_method_body!();
        }
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            1_u64
                .saturating_add(
                    u64::try_from(init_record.class.as_ref().map_or(0, String::len))
                        .unwrap_or(u64::MAX),
                )
                .saturating_add(
                    u64::try_from(init_record.declared.as_ref().map_or(0, String::len))
                        .unwrap_or(u64::MAX),
                ),
        )?;
        constructor_init = Some(init_record.clone());
        Some(init_bci)
    } else {
        if !body.constructor_calls.is_empty() {
            reject_generic_method_body!();
        }
        None
    };
    let mut field_write_sites = Vec::with_capacity(body.field_writes.len());
    for write in &body.field_writes {
        budget.poll()?;
        if write.op != jarde_java::ast::AssignOp::Assign
            || write.value.primary.method.as_ref() != Some(method)
        {
            reject_generic_method_body!();
        }
        let Some(receiver) = write.receiver.as_ref() else {
            reject_generic_method_body!();
        };
        if receiver.primary.method.as_ref() != Some(method)
            || receiver.direct_local_name.as_deref() != Some("this")
            || !matches!(receiver.shape, Shape::Local)
        {
            reject_generic_method_body!();
        }
        let mut body_sites = body_values
            .iter()
            .filter(|site| site.bci == write.bci && site.opcode == 0xb5);
        let Some(body_site) = body_sites.next() else {
            reject_generic_method_body!();
        };
        if body_sites.next().is_some() || body_site.stack_reads.len() != 2 {
            reject_generic_method_body!();
        }
        let receiver_fact = &body_site.stack_reads[0].1;
        let Some((receiver_load_bci, _)) =
            generic_call_load_source(body_values, receiver_fact, write.bci)
        else {
            reject_generic_method_body!();
        };
        if receiver_load_bci != receiver.primary.bci
            || !generic_call_receiver_is_initialized_this_at(
                write.bci,
                method,
                receiver_fact,
                header,
                class_internal,
                class_superclass,
                init_record,
                body_values,
                invokes,
            )
        {
            reject_generic_method_body!();
        }
        let mut matching = field_uses.iter().filter(|field_use| {
            field_use.physical_method == *method
                && field_use.bci == write.bci
                && field_use.opcode == 0xb5
                && field_use.owner == class_internal
                && source_identifier(&field_use.name).as_deref() == Some(write.name.as_str())
        });
        let Some(field_use) = matching.next() else {
            reject_generic_method_body!();
        };
        if matching.next().is_some() {
            reject_generic_method_body!();
        }
        let mut declarations = physical_fields.iter().filter(|field| {
            field.name.raw().0 == field_use.name && field.descriptor.raw().0 == field_use.descriptor
        });
        let Some(field) = declarations.next() else {
            reject_generic_method_body!();
        };
        if declarations.next().is_some() {
            reject_generic_method_body!();
        }
        match &field_use.write_source {
            Some(class_source::SameClassFieldWriteSource::Null) => {
                if !field_use.source_complete
                    || !write.value.null_literal
                    || !matches!(field_use.descriptor.first().copied(), Some(b'L' | b'['))
                    || !generic_call_null_matches(
                        body_values,
                        &body_site.stack_reads[1].1,
                        &write.value,
                        write.bci,
                    )
                {
                    reject_generic_method_body!();
                }
            }
            Some(class_source::SameClassFieldWriteSource::Parameter {
                method: source_method,
                slot,
            }) => {
                if !field_use.source_complete
                    || source_method != method
                    || !matches!(&write.value.shape, Shape::Local)
                    || write.value.direct_local_name.as_deref() != names_by_slot.get(slot).copied()
                {
                    reject_generic_method_body!();
                }
                let Some(position) = slots.iter().position(|formal| formal == slot) else {
                    reject_generic_method_body!();
                };
                let mut source_type = &contract.parameters[position];
                let parameterized_class_source = loop {
                    budget.poll()?;
                    match source_type {
                        SignatureType::Array(component) => {
                            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                            source_type = component;
                        }
                        SignatureType::Class(class) => {
                            budget.charge(
                                CountedBudgetDimension::AnalysisSteps,
                                u64::try_from(class.segments.len()).unwrap_or(u64::MAX),
                            )?;
                            let mut parameterized = false;
                            for segment in &class.segments {
                                budget.poll()?;
                                parameterized |= !segment.arguments.is_empty();
                            }
                            break parameterized;
                        }
                        SignatureType::Base(_) | SignatureType::TypeVariable(_) => break false,
                    }
                };
                if parameterized_class_source {
                    reject_generic_method_body!();
                }
                if unique_formal_source(
                    names_by_slot.get(slot).copied().unwrap_or_default(),
                    write.value.primary.bci,
                    write.bci,
                ) != Some((*slot, position))
                    || !body_values
                        .iter()
                        .find(|site| site.bci == write.value.primary.bci)
                        .is_some_and(|source_site| {
                            source_site.stack_writes.iter().any(|source| {
                                source.uses.as_slice() == [Some(write.bci)]
                                    && source.value == body_site.stack_reads[1].1.value
                            })
                        })
                    || class_source::same_class_generic_source_assignable_to_descriptor(
                        &contract.parameters[position],
                        &contract.method_parameters,
                        &field_use.descriptor,
                        class_scope,
                        class_parameters,
                        budget,
                    )? != Some(true)
                {
                    reject_generic_method_body!();
                }
            }
            Some(class_source::SameClassFieldWriteSource::StringConstant { value_bci }) => {
                let rhs = &body_site.stack_reads[1].1;
                budget.poll()?;
                budget.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(body_values.len()).unwrap_or(u64::MAX),
                )?;
                let mut constants = body_values.iter().filter(|site| site.bci == *value_bci);
                let Some(constant_site) = constants.next() else {
                    reject_generic_method_body!();
                };
                if !field_use.source_complete
                    || field_use.descriptor != b"Ljava/lang/String;"
                    || !matches!(write.value.shape, Shape::StringLiteral)
                    || write.value.primary.method.as_ref() != Some(method)
                    || write.value.primary.bci != *value_bci
                    || constants.next().is_some()
                    || !matches!(constant_site.opcode, 0x12 | 0x13)
                    || constant_site.stack_writes.len() != 1
                    || constant_site.stack_writes[0].value != rhs.value
                    || constant_site.stack_writes[0].definition != rhs.definition
                    || constant_site.stack_writes[0].replaced_by.is_some()
                    || constant_site.stack_writes[0].uses.as_slice() != [Some(write.bci)]
                    || !matches!(rhs.definition, Definition::Instruction { bci, .. } if bci == *value_bci)
                    || rhs.replaced_by.is_some()
                    || rhs.uses.as_slice() != [Some(write.bci)]
                {
                    reject_generic_method_body!();
                }
            }
            _ if matches!(&write.value.shape, Shape::Call { .. })
                && validated_call_bcis.contains(&write.value.primary.bci)
                && matches!(
                    body_site.stack_reads[1].1.definition,
                    jarde_jvm::method_ir::Definition::Instruction { bci, .. }
                        if bci == write.value.primary.bci
                )
                && body_site.stack_reads[1].1.replaced_by.is_none()
                && body_site.stack_reads[1].1.uses.as_slice() == [Some(write.bci)] => {}
            _ => reject_generic_method_body!(),
        }
        field_write_sites.push(write.bci);
        let _ = field;
    }
    let mut return_sources = Vec::with_capacity(body.returns.len());
    for returned in &body.returns {
        budget.poll()?;
        if returned.expression.primary.method.as_ref() != Some(method) {
            reject_generic_method_body!();
        }
        if returned.catch_scopes.iter().any(|scope| {
            scope.local_name
                == returned
                    .expression
                    .direct_local_name
                    .as_deref()
                    .unwrap_or_default()
        }) {
            reject_generic_method_body!();
        }
        match &returned.expression.shape {
            Shape::Local => {
                let Some(name) = returned.expression.direct_local_name.as_deref() else {
                    reject_generic_method_body!();
                };
                let Some((slot, position)) =
                    unique_formal_source(name, returned.expression.primary.bci, returned.bci)
                else {
                    reject_generic_method_body!();
                };
                if contract.result.as_ref() != contract.parameters.get(position) {
                    reject_generic_method_body!();
                }
                return_sources
                    .push(class_source::SameClassGenericBodyReturnSource::Parameter { slot });
            }
            Shape::Call { .. } => {
                let Some(source) = validated_return_calls.iter().find(|source| {
                    matches!(source,
                        class_source::SameClassGenericBodyReturnSource::Call { bci, .. }
                            if *bci == returned.expression.primary.bci)
                }) else {
                    reject_generic_method_body!();
                };
                return_sources.push(source.clone());
            }
            _ => reject_generic_method_body!(),
        }
    }
    let mut condition_slots = Vec::with_capacity(body.conditions.len());
    for condition in &body.conditions {
        budget.poll()?;
        let (name, expression_bci) = match &condition.expression.shape {
            Shape::Local => {
                let Some(name) = condition.expression.direct_local_name.as_deref() else {
                    reject_generic_method_body!();
                };
                (name, condition.expression.primary.bci)
            }
            Shape::BooleanNotLocal {
                local_name,
                primary,
                ..
            } => (local_name.as_str(), primary.bci),
            _ => reject_generic_method_body!(),
        };
        if condition.expression.primary.method.as_ref() != Some(method) {
            reject_generic_method_body!();
        }
        if condition
            .catch_scopes
            .iter()
            .any(|scope| scope.local_name == name)
        {
            reject_generic_method_body!();
        }
        let Some((slot, position)) = unique_formal_source(name, expression_bci, condition.bci)
        else {
            reject_generic_method_body!();
        };
        if contract.parameters.get(position) != Some(&SignatureType::Base(b'Z')) {
            reject_generic_method_body!();
        }
        condition_slots.push(slot);
    }
    let mut throw_sites = Vec::with_capacity(body.throws.len());
    for thrown in &body.throws {
        budget.poll()?;
        if thrown.expression.primary.method.as_ref() != Some(method) {
            reject_generic_method_body!();
        }
        let exception_shape = matches!(
            &thrown.expression.shape,
            Shape::New { ty, argument_count: 0, qualified: false }
                if ty == "java.lang.RuntimeException" || ty == "RuntimeException"
        );
        let expression_bcis = std::iter::once(thrown.expression.primary.bci)
            .chain(thrown.expression.derived.iter().map(|anchor| anchor.bci))
            .collect::<std::collections::BTreeSet<_>>();
        let allocation_bcis = allocations
            .iter()
            .filter(|(bci, owner)| {
                owner == b"java/lang/RuntimeException" && expression_bcis.contains(bci)
            })
            .map(|(bci, _)| *bci)
            .collect::<Vec<_>>();
        let constructor_sites = invokes
            .iter()
            .filter(|invoke| {
                invoke.physical_caller == *method
                    && invoke.owner == b"java/lang/RuntimeException"
                    && invoke.name == b"<init>"
                    && expression_bcis.contains(&invoke.bci)
            })
            .collect::<Vec<_>>();
        if !exception_shape
            || allocation_bcis.len() != 1
            || !constructor_sites.iter().any(|invoke| {
                invoke.opcode == 0xb7
                    && invoke.descriptor == b"()V"
                    && invoke.argument_values.is_empty()
                    && invoke.ssa_operands_complete
                    && invoke.constructor_receiver
                        == Some(class_source::SameClassConstructorReceiver::NewObject)
                    && invoke.receiver_value_fact.as_ref().is_some_and(|receiver| {
                        generic_exception_new_receiver_matches(
                            body_values,
                            allocation_bcis[0],
                            invoke.bci,
                            receiver,
                        )
                    })
            })
        {
            reject_generic_method_body!();
        }
        throw_sites.push(thrown.bci);
    }
    if contract.result.is_some() && return_sources.is_empty() {
        reject_generic_method_body!();
    }
    Ok(Some(class_source::SameClassGenericMethodBodyProof {
        method: method.clone(),
        formal_names: formal_pairs,
        return_sources,
        condition_slots,
        throw_sites,
        field_write_sites,
        constructor_call_bci,
        constructor_init,
    }))
}

/// One same-read consumer's verdict for a field value under a projected (parameterized) type.
///
/// The safe consumers are the ones that use the value at or above the generality its erasure
/// already had: an unchecked assignment, a comparison, a cast, a monitor, or an argument position
/// whose callee descriptor takes exactly `java.lang.Object` or the field's own erasure at that
/// position. Anything that would *select members on* the value — an invoke receiver, a chained
/// field access, an array store's element position — or that moves it somewhere this scan cannot
/// follow (a local store, a duplication, a phi, a dynamic site) is refused, because a
/// parameterized receiver can change which member the source call binds or stop compiling.
pub fn field_read_consumer_expressible(
    consumer_opcode: u8,
    consumer_reads_stack: &[(jarde_jvm::method_ir::Slot, jarde_jvm::method_ir::ValueId)],
    value_depth: u32,
    field_descriptor: &[u8],
    callee_descriptor: Option<&[u8]>,
) -> bool {
    use jarde_jvm::method_ir::Slot;
    let deepest_stack_depth = |reads: &[(Slot, jarde_jvm::method_ir::ValueId)]| {
        reads
            .iter()
            .filter_map(|(slot, _)| match slot {
                Slot::Stack(depth) => Some(*depth),
                Slot::Local(_) => None,
            })
            .min()
    };
    match consumer_opcode {
        // Reads at or above erasure generality: comparisons, casts, monitors, throws, returns,
        // pops, and a store *into* another array's element position (the value is what is stored,
        // and the verifier proved it fits the array's erasure).
        0x50 | 0x57 | 0x58 | 0xa5 | 0xa6 | 0xb0 | 0xbf | 0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc6
        | 0xc7 => true,
        // A store of the value into a local: the presentation types every local from physical
        // descriptor positions (never from the projected `Signature`), so the emitted local keeps
        // an erasure-level type and the parameterized value assigns to it as the unchecked
        // assignment it already was. Member selection through that local stays governed by the
        // local's own erasure, which the body was already written under.
        0x3a | 0x4a | 0x4b | 0x4c | 0x4d => true,
        // `aastore` (0x53) reads array, index and value; the field value is only safe in the
        // stored-value (topmost) position, not as the array it stores into.
        0x53 => {
            let Some(deepest) = deepest_stack_depth(consumer_reads_stack) else {
                return false;
            };
            value_depth > deepest
        }
        // A stored field value keeps the assignment legal under the projected type (the unchecked
        // assignment it already was); a value used as the *receiver* of the store selects members.
        0xb5 | 0xb3 => {
            let Some(deepest) = deepest_stack_depth(consumer_reads_stack) else {
                return false;
            };
            value_depth > deepest
        }
        // Invocations: the receiver is the deepest operand of a receiver-bearing call; anything
        // else is an argument, safe only at a position the callee descriptor types as exactly
        // `java.lang.Object` or the field's own erasure.
        0xb6 | 0xb7 | 0xb8 | 0xb9 => {
            let Some(deepest) = deepest_stack_depth(consumer_reads_stack) else {
                return false;
            };
            if consumer_opcode != 0xb8 && value_depth == deepest {
                return false;
            }
            let Some(callee) = callee_descriptor else {
                return false;
            };
            let arguments: Vec<u32> = consumer_reads_stack
                .iter()
                .filter_map(|(slot, _)| match slot {
                    Slot::Stack(depth) => Some(*depth),
                    Slot::Local(_) => None,
                })
                .filter(|depth| *depth > deepest || consumer_opcode == 0xb8)
                .collect();
            let position = arguments
                .iter()
                .filter(|depth| **depth < value_depth)
                .count();
            let raw = method_parameter_raw_descriptor(callee, position);
            match raw.as_deref() {
                Some(b"Ljava/lang/Object;") => true,
                Some(segment) => segment == field_descriptor,
                None => false,
            }
        }
        // Everything else — chained member selection, local stores, duplications, dynamic sites —
        // is outside this slice's proof.
        _ => false,
    }
}

/// The raw descriptor segment of one parameter position: `Ljava/util/List;`, `[Ljava/lang/String;`
/// or a primitive letter, exactly as the pool states it.
pub fn method_parameter_raw_descriptor(descriptor: &[u8], position: usize) -> Option<Vec<u8>> {
    let mut rest = descriptor.strip_prefix(b"(")?;
    let mut index = 0usize;
    while let Some(&first) = rest.first() {
        if first == b')' {
            return None;
        }
        let start = rest;
        while rest.first() == Some(&b'[') {
            rest = &rest[1..];
        }
        let segment_end = match rest.first() {
            Some(b'L') => rest.iter().position(|&byte| byte == b';')? + 1,
            Some(_) => 1,
            None => return None,
        };
        rest = &rest[segment_end..];
        let segment = &start[..start.len() - rest.len()];
        if index == position {
            return Some(segment.to_vec());
        }
        index += 1;
    }
    None
}

/// Classify one field-read site's consumers through the same run's SSA: every instruction that
/// reads the value the `getfield`/`getstatic` produced must keep accepting it.
pub fn classify_field_read_consumers(
    ir: &jarde_jvm::method_ir::MethodIr,
    bci: u32,
    pool: &[jarde_reader::classfile::CpEntryFacts],
    field_descriptor: &[u8],
) -> bool {
    use jarde_jvm::method_ir::{PhiInput, Slot};
    let Some(ssa) = ir.ssa() else {
        return false;
    };
    let Some(value) = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find_map(|instruction| {
            (instruction.bci() == bci).then(|| {
                instruction
                    .writes()
                    .iter()
                    .find_map(|(slot, value)| match slot {
                        Slot::Stack(_) => Some(*value),
                        Slot::Local(_) => None,
                    })
            })
        })
        .flatten()
    else {
        return false;
    };
    // A phi that merges this value sends it somewhere this scan does not follow.
    for phi in ssa.phis() {
        if phi
            .inputs()
            .iter()
            .any(|input| matches!(input, PhiInput::Value(other) if *other == value))
        {
            return false;
        }
    }
    let mut consumers: Vec<(
        u8,
        Vec<(Slot, jarde_jvm::method_ir::ValueId)>,
        u32,
        Option<Vec<u8>>,
    )> = Vec::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            if instruction.bci() == bci {
                continue;
            }
            let reads_value = instruction.reads().iter().any(|(_, read)| *read == value);
            if !reads_value {
                continue;
            }
            // Only operand-stack reads are consumers of a freshly loaded value.
            let stack_reads: Vec<(Slot, jarde_jvm::method_ir::ValueId)> = instruction
                .reads()
                .iter()
                .filter_map(|(slot, read)| match slot {
                    Slot::Stack(_) => Some((*slot, *read)),
                    Slot::Local(_) => None,
                })
                .collect();
            let Some(Slot::Stack(depth)) = stack_reads
                .iter()
                .find(|(_, read)| *read == value)
                .map(|(slot, _)| slot)
                .copied()
            else {
                return false;
            };
            let callee_descriptor = if (0xb6..=0xb9).contains(&instruction.opcode()) {
                ir.code().and_then(|code| {
                    code.instructions
                        .iter()
                        .zip(code.operands())
                        .find(|(site, _)| site.bci == instruction.bci())
                        .and_then(|(_, operands)| operands.constant_pool_index)
                        .and_then(|index| {
                            jarde_reader::classfile::cp_entry(pool, index)
                                .ok()
                                .map(|entry| match &entry.kind {
                                    jarde_reader::classfile::CpEntryKind::MethodRef {
                                        descriptor,
                                        ..
                                    }
                                    | jarde_reader::classfile::CpEntryKind::InterfaceMethodRef {
                                        descriptor,
                                        ..
                                    } => descriptor.0.clone(),
                                    _ => Vec::new(),
                                })
                        })
                })
            } else {
                None
            };
            consumers.push((instruction.opcode(), stack_reads, depth, callee_descriptor));
        }
    }
    if consumers.is_empty() {
        return true;
    }
    consumers
        .iter()
        .all(|(opcode, stack_reads, depth, callee)| {
            field_read_consumer_expressible(
                *opcode,
                stack_reads,
                *depth,
                field_descriptor,
                callee.as_deref(),
            )
        })
}

pub fn field_write_source(
    ir: &jarde_jvm::method_ir::MethodIr,
    code: &jarde_reader::classfile::MethodCodeFacts,
    instruction_index: usize,
    target_descriptor: &[u8],
    ssa: &jarde_jvm::method_ir::SsaTable,
    ssa_by_bci: &std::collections::HashMap<u32, &jarde_jvm::method_ir::SsaInstruction>,
    code_by_bci: &std::collections::HashMap<u32, &InstructionFact>,
    ssa_bci_is_unique: bool,
    budget: &mut Budget,
) -> Result<Option<class_source::SameClassFieldWriteSource>> {
    use class_source::SameClassFieldWriteSource as Source;
    use jarde_jvm::method_ir::{Definition, Slot, Value};
    use jarde_reader::classfile::CpEntryKind as K;

    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if !ssa_bci_is_unique {
        return Ok(None);
    }
    let instruction = &code.instructions[instruction_index];
    let Some(ssa_instruction) = ssa_by_bci.get(&instruction.bci) else {
        return Ok(None);
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(ssa_instruction.reads().len()).unwrap_or(u64::MAX),
    )?;
    let Some((_, rhs)) = ssa_instruction
        .reads()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
        .max_by_key(|(slot, _)| *slot)
    else {
        return Ok(None);
    };
    let value = ssa.value(*rhs);
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(value.uses().len()).unwrap_or(u64::MAX),
    )?;
    if matches!(value.ty(), Value::Null) {
        if let Definition::Instruction { bci, .. } = value.def()
            && code_by_bci
                .get(bci)
                .is_some_and(|candidate| candidate.opcode == 0x01)
            && value.uses().len() == 1
            && value.uses()[0].bci() == Some(instruction.bci)
        {
            return Ok(Some(Source::Null));
        }
        return Ok(None);
    }

    // Admit one constant source shape for method-body certificates: the physical RHS is the
    // result of one `ldc`/`ldc_w` String constant and the actual PUTFIELD/PUTSTATIC descriptor is
    // String. Generic field-signature assignment proofs keep this source conservative below.
    if target_descriptor == b"Ljava/lang/String;"
        && let Definition::Instruction { bci, .. } = value.def()
        && let Some(load) = code_by_bci.get(bci)
        && matches!(load.opcode, 0x12 | 0x13)
        && value.uses().len() == 1
        && value.uses()[0].bci() == Some(instruction.bci)
    {
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
        )?;
        let mut matching_loads = code
            .instructions
            .iter()
            .enumerate()
            .filter(|(_, candidate)| candidate.bci == *bci);
        let Some((load_index, _)) = matching_loads.next() else {
            return Ok(None);
        };
        if matching_loads.next().is_some() {
            return Ok(None);
        }
        let Some(load_ssa) = ssa_by_bci.get(bci) else {
            return Ok(None);
        };
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(load_ssa.writes().len()).unwrap_or(u64::MAX),
        )?;
        if load_ssa
            .writes()
            .iter()
            .filter(|(slot, written)| matches!(slot, Slot::Stack(_)) && *written == *rhs)
            .count()
            != 1
        {
            return Ok(None);
        }
        let Some(constant_pool_index) = code
            .operands()
            .get(load_index)
            .and_then(|operand| operand.constant_pool_index)
        else {
            return Ok(None);
        };
        let Ok(constant) =
            jarde_reader::classfile::cp_entry(ir.constant_pool(), constant_pool_index)
        else {
            return Ok(None);
        };
        if matches!(constant.kind, K::String { .. }) {
            return Ok(Some(Source::StringConstant { value_bci: *bci }));
        }
        return Ok(None);
    }

    if let Some(declaration) = ir.declaration() {
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(declaration.descriptor().0.len()).unwrap_or(u64::MAX),
        )?;
    }
    if let Definition::Instruction { bci, .. } = value.def()
        && let Some(load) = code_by_bci.get(bci)
        && matches!(load.opcode, 0x19 | 0x2a..=0x2d)
        && value.uses().len() == 1
        && value.uses()[0].bci() == Some(instruction.bci)
        && let Some(load_ssa) = ssa_by_bci.get(bci)
        && let Some((Slot::Local(local), entry)) = load_ssa
            .reads()
            .iter()
            .find(|(slot, _)| matches!(slot, Slot::Local(_)))
        && matches!(ssa.value(*entry).def(), Definition::Entry { slot: Slot::Local(slot), .. } if slot == local)
        && let Some(declaration) = ir.declaration()
        && let Ok(descriptor) = descriptor_facts(
            declaration.descriptor().0.as_slice(),
            DescriptorKind::Method,
        )
        && let Some(parameters) = jarde_jvm::method_ir::parameter_positions(
            &descriptor,
            declaration.access_flags() & 0x0008 != 0,
        )
        && parameters.contains(local)
    {
        return Ok(Some(Source::Parameter {
            method: declaration.identity().clone(),
            slot: *local,
        }));
    }

    // The only allocation spelling admitted here is the already-supported direct
    // `new; dup; invokespecial <init>()V; putfield/putstatic` source shape. The uninitialized
    // aliases must share one `new_site` token; initialization then defines the RHS ValueId.
    // Locals, casts, merges and nested calls cannot enter this branch.
    if instruction_index < 3 {
        return Ok(None);
    }
    let allocation = &code.instructions[instruction_index - 3];
    let duplicate = &code.instructions[instruction_index - 2];
    let constructor = &code.instructions[instruction_index - 1];
    if allocation.opcode != 0xbb || duplicate.opcode != 0x59 || constructor.opcode != 0xb7 {
        return Ok(None);
    }
    if !matches!(value.def(), Definition::Instruction { bci, .. } if *bci == constructor.bci)
        || value.uses().len() != 1
        || value.uses()[0].bci() != Some(instruction.bci)
    {
        return Ok(None);
    }
    let pool = ir.constant_pool();
    let Some(allocation_index) = code.operands()[instruction_index - 3].constant_pool_index else {
        return Ok(None);
    };
    let Some(constructor_index) = code.operands()[instruction_index - 1].constant_pool_index else {
        return Ok(None);
    };
    let Ok(allocation_entry) = jarde_reader::classfile::cp_entry(pool, allocation_index) else {
        return Ok(None);
    };
    let K::Class { name: owner, .. } = &allocation_entry.kind else {
        return Ok(None);
    };
    let Ok(constructor_entry) = jarde_reader::classfile::cp_entry(pool, constructor_index) else {
        return Ok(None);
    };
    let K::MethodRef {
        owner: constructor_owner,
        name,
        descriptor,
        ..
    } = &constructor_entry.kind
    else {
        return Ok(None);
    };
    if constructor_owner != owner || name.0 != b"<init>" || descriptor.0 != b"()V" {
        return Ok(None);
    }
    let (Some(allocation_ssa), Some(duplicate_ssa), Some(constructor_ssa)) = (
        ssa_by_bci.get(&allocation.bci),
        ssa_by_bci.get(&duplicate.bci),
        ssa_by_bci.get(&constructor.bci),
    ) else {
        return Ok(None);
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            allocation_ssa
                .writes()
                .len()
                .saturating_add(duplicate_ssa.reads().len())
                .saturating_add(duplicate_ssa.writes().len())
                .saturating_add(constructor_ssa.reads().len())
                .saturating_add(constructor_ssa.writes().len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    let uninitialized_at = |value: jarde_jvm::method_ir::ValueId| match ssa.value(value).ty() {
        Value::Uninitialized { new_site } => Some(new_site.bci()),
        _ => None,
    };
    let has_uninitialized_at = |values: &[(Slot, jarde_jvm::method_ir::ValueId)], bci| {
        values
            .iter()
            .any(|(_, value)| uninitialized_at(*value) == Some(bci))
    };
    let allocation_is_same_site = has_uninitialized_at(allocation_ssa.writes(), allocation.bci);
    let duplicate_reads_same_site = has_uninitialized_at(duplicate_ssa.reads(), allocation.bci);
    let duplicate_writes_same_site = has_uninitialized_at(duplicate_ssa.writes(), allocation.bci);
    let constructor_reads_same_site = has_uninitialized_at(constructor_ssa.reads(), allocation.bci);
    let initialized_alias = constructor_ssa
        .writes()
        .iter()
        .any(|(_, value)| *value == *rhs);
    if !allocation_is_same_site
        || !duplicate_reads_same_site
        || !duplicate_writes_same_site
        || !constructor_reads_same_site
        || !initialized_alias
    {
        return Ok(None);
    }
    Ok(Some(Source::RawAllocation {
        owner: owner.0.clone(),
    }))
}

/// The bounded same-class use-site scan over one member body's decoded Code, in the shape of the
/// array-helper census beside it: physical instructions, their pool entries, the readable
/// non-body sources, and a completeness flag that turns any stop or unresolvable source into "not
/// proven" at the class level.
pub fn same_class_invoke_value_fact(
    ssa: &jarde_jvm::method_ir::SsaTable,
    value: jarde_jvm::method_ir::ValueId,
    budget: &mut Budget,
) -> Result<class_source::SameClassInvokeValueFact> {
    budget.poll()?;
    let source = ssa.value(value);
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(source.uses().len()).unwrap_or(u64::MAX),
    )?;
    Ok(class_source::SameClassInvokeValueFact {
        value,
        definition: source.def().clone(),
        uses: source.uses().iter().map(|usage| usage.bci()).collect(),
        replaced_by: source.replaced_by(),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericCallSiteProof {
    pub direct_return: bool,
    pub result_type: Option<jarde_reader::signature::SignatureType>,
    pub presentation_removals: Vec<jarde_java::report::ClassSourceInvokeArgumentPresentationCast>,
}

/// One physically decoded `invoke*` site of the selected class's own body set: the caller's own
/// label, the site's BCI and opcode, and the owner/name/descriptor the pool entry states.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassInvokeUse {
    pub caller: String,
    pub physical_caller: PhysicalMethodId,
    pub bci: u32,
    pub opcode: u8,
    pub owner: Vec<u8>,
    pub name: Vec<u8>,
    pub descriptor: Vec<u8>,
    /// Physical SSA values aligned to descriptor parameter order. Empty is meaningful only when
    /// `ssa_operands_complete` is true for a zero-argument call.
    pub argument_values: Vec<jarde_jvm::method_ir::ValueId>,
    pub receiver_value: Option<jarde_jvm::method_ir::ValueId>,
    pub result_value: Option<jarde_jvm::method_ir::ValueId>,
    pub argument_value_facts: Vec<SameClassInvokeValueFact>,
    pub receiver_value_fact: Option<SameClassInvokeValueFact>,
    pub result_value_fact: Option<SameClassInvokeValueFact>,
    pub ssa_operands_complete: bool,
    /// For a constructor invocation, the receiver identity established by the same run's SSA.
    /// `None` means this is not a constructor invocation; `Unknown` stays local to this site and
    /// must not make unrelated same-class consumers incomplete.
    pub constructor_receiver: Option<SameClassConstructorReceiver>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassInvokeValueFact {
    pub value: jarde_jvm::method_ir::ValueId,
    pub definition: jarde_jvm::method_ir::Definition,
    pub uses: Vec<Option<u32>>,
    pub replaced_by: Option<jarde_jvm::method_ir::ValueId>,
}

/// One same-run SSA output at an AST expression's primary BCI. Generic body proofs retain only
/// the load/result identities needed to close formal reads and return/condition/throw consumers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassBodyValueSite {
    pub bci: u32,
    pub opcode: u8,
    pub stack_reads: Vec<(u32, SameClassInvokeValueFact)>,
    pub stack_writes: Vec<SameClassInvokeValueFact>,
    pub local_reads: Vec<(u16, SameClassInvokeValueFact)>,
    pub local_writes: Vec<(u16, SameClassInvokeValueFact)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SameClassGenericBodyReturnSource {
    Parameter {
        slot: u16,
    },
    Call {
        bci: u32,
        opcode: u8,
        owner: Vec<u8>,
        name: Vec<u8>,
        descriptor: Vec<u8>,
    },
}

/// The evidence retained by the emitter for an AST-shaped generic method body. It records actual
/// physical formal identities and each return source; it is not a boolean saying that a body was
/// merely seen or parsed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassGenericMethodBodyProof {
    pub method: PhysicalMethodId,
    pub formal_names: Vec<(u16, String)>,
    pub return_sources: Vec<SameClassGenericBodyReturnSource>,
    pub condition_slots: Vec<u16>,
    pub throw_sites: Vec<u32>,
    pub field_write_sites: Vec<u32>,
    pub constructor_call_bci: Option<u32>,
    pub constructor_init: Option<jarde_java::init::InitRecord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SameClassConstructorReceiver {
    UninitializedThis,
    NewObject,
    Unknown,
}

/// One physically decoded field access of the selected class's own body set. A write site carries
/// its source-level proof when the RHS is one of the narrowly supported same-scope parameter,
/// literal-null, or completed raw-allocation forms; an absent proof makes the projected field
/// unsafe for that writer. A read site carries the SSA-side classification of what the loaded
/// value feeds: `Some(true)` only when every consumer keeps accepting the parameterized value (an
/// `Object`-parameter argument position, or a consumer that does not select members on it).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassFieldUse {
    pub caller: String,
    pub physical_method: PhysicalMethodId,
    pub bci: u32,
    pub opcode: u8,
    pub owner: Vec<u8>,
    pub name: Vec<u8>,
    pub descriptor: Vec<u8>,
    pub read_expressible: Option<bool>,
    pub write_source: Option<SameClassFieldWriteSource>,
    pub receiver_source: Option<SameClassFieldReceiverSource>,
    pub source_complete: bool,
}

/// A receiver fact from the actual committed source statement at this field-use BCI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SameClassFieldReceiverSource {
    This,
    Parameter { slot: u16 },
    RawLocal,
}

/// A narrowly proved source for one same-class field write. `None` on the use site means the
/// physical write was decoded but its value did not match one of these closed shapes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SameClassFieldWriteSource {
    Null,
    /// A same-run `ldc`/`ldc_w` String constant used once as a String field's physical RHS.
    /// This fact is for the narrow method-body certificate only; generic field assignment proofs
    /// deliberately do not treat it as evidence that a value satisfies a type variable.
    StringConstant {
        value_bci: u32,
    },
    Parameter {
        method: PhysicalMethodId,
        slot: u16,
    },
    RawAllocation {
        owner: Vec<u8>,
    },
}

/// The actual source parameter types and declared method bounds a completed same-class method
/// record publishes. Generic facts are present only when that method's own `Signature` projected
/// successfully; otherwise only the physical descriptor is available to the field-write proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassPublishedMethodParameters {
    pub method: PhysicalMethodId,
    pub parameter_slots: Vec<u16>,
    pub descriptor_parameters: Vec<Vec<u8>>,
    pub generic_parameters: Option<Vec<SignatureType>>,
    pub method_type_parameters: Vec<TypeParameter>,
}

/// A member a readable but non-body reference source names: an `ldc` method handle or a
/// bootstrap-table row/argument. The first slice reads these to keep the inventory honest but does
/// not prove a functional-position target, so a candidate such a source names stays refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassMemberRef {
    pub owner: Vec<u8>,
    pub name: Vec<u8>,
    pub descriptor: Vec<u8>,
    pub method: bool,
}

/// The bounded use-site inventory of one selected class, as the commit phase hands it to the
/// binding proofs: every decoded body's invoke and field sites, every readable non-body source,
/// and whether the scan covered every supported consumer at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SameClassUseFacts<'a> {
    pub complete: bool,
    pub invokes: &'a [SameClassInvokeUse],
    pub field_uses: &'a [SameClassFieldUse],
    pub member_refs: &'a [SameClassMemberRef],
}

/// The published generic contract read from one physical same-class method. The caller may use
/// this only after the matching declaration has been staged from the same Signature and erasure
/// proof; its physical identity keeps overloaded methods and method binders separate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassGenericMethodContract {
    pub method: PhysicalMethodId,
    pub method_parameters: Vec<TypeParameter>,
    pub parameters: Vec<SignatureType>,
    pub result: Option<SignatureType>,
}

/// Source type proved at one invocation argument. Only a direct emitted parameter is admitted in
/// overload selection; generic substitution can additionally receive an exact nested call result
/// whose own callee and uses were proved first. Casts, aliases, unknown locals, and AST `presented`
/// types do not create a source type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassGenericArgumentSource {
    pub caller: PhysicalMethodId,
    pub origin: SameClassGenericArgumentOrigin,
    pub source_type: SignatureType,
    pub caller_method_parameters: Vec<TypeParameter>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SameClassGenericArgumentOrigin {
    Formal {
        slot: u16,
    },
    NullLiteral,
    NestedCall {
        bci: u32,
        opcode: u8,
        owner: Vec<u8>,
        name: Vec<u8>,
        descriptor: Vec<u8>,
    },
}

/// A consistent invocation substitution for the callee's method-level type variables.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassGenericSubstitution {
    pub method_arguments: Vec<SignatureType>,
    pub argument_sources: Vec<SameClassGenericArgumentSource>,
}

/// One source argument admitted by the overload proof. A non-null argument retains the physical
/// caller, its formal or nested-call origin, the Signature type, and the caller binder list that
/// proved its source type. Null has no invented source type and cannot infer a callee method binder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassOverloadActual {
    pub source: Option<SameClassGenericArgumentSource>,
    pub null_literal: bool,
}

/// One complete, source-writable same-class declaration that can participate in overload
/// resolution. `owner` is the class-file internal name paired with the physical method identity;
/// the caller supplies it from the same completed class inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameClassOverloadCandidate {
    pub method: PhysicalMethodId,
    pub owner: Vec<u8>,
    pub descriptor: Vec<u8>,
    pub access_flags: u16,
    pub parameters: Vec<SignatureType>,
    pub method_parameters: Vec<TypeParameter>,
    pub source_writable: bool,
}

/// Prove direct type-variable substitution and all explicit bounds for one invocation. The
/// caller's and callee's method binders are resolved before their shared class scope, so a method
/// `T` shadowing class `T` cannot silently become the class variable. Only direct formal source
/// types are accepted here; general expressions need their own source proof.
pub fn prove_same_class_generic_substitution(
    class_scope: &[TypeParameterErasure],
    class_parameters: &[TypeParameter],
    callee: &SameClassGenericMethodContract,
    actuals: &[SameClassGenericArgumentSource],
    budget: &mut Budget,
) -> Result<Option<SameClassGenericSubstitution>> {
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(callee.parameters.len().saturating_add(actuals.len())).unwrap_or(u64::MAX),
    )?;
    if callee.parameters.len() != actuals.len() {
        return Ok(None);
    }
    let mut method_arguments: Vec<Option<SignatureType>> =
        vec![None; callee.method_parameters.len()];
    for (parameter, actual) in callee.parameters.iter().zip(actuals) {
        budget.poll()?;
        if !same_class_call_type_matches(
            parameter,
            &actual.source_type,
            class_scope,
            class_parameters,
            &callee.method_parameters,
            &actual.caller_method_parameters,
            &mut method_arguments,
            budget,
            0,
        )? {
            return Ok(None);
        }
    }
    if method_arguments.iter().any(Option::is_none) {
        // An unconstrained type variable cannot be inferred from a null, omitted, or otherwise
        // unobserved argument. It must have one direct source at an actual parameter position.
        return Ok(None);
    }
    Ok(Some(SameClassGenericSubstitution {
        method_arguments: method_arguments.into_iter().flatten().collect(),
        argument_sources: actuals.to_vec(),
    }))
}

pub fn same_class_generic_result_matches(
    class_scope: &[TypeParameterErasure],
    caller_method_parameters: &[TypeParameter],
    caller_result: &SignatureType,
    callee: &SameClassGenericMethodContract,
    substitution: &SameClassGenericSubstitution,
    budget: &mut Budget,
) -> Result<bool> {
    let Some(callee_result) = callee.result.as_ref() else {
        return Ok(false);
    };
    let Some(substituted) = substitute_signature_type_parameters(
        callee_result,
        &callee.method_parameters,
        &substitution.method_arguments,
        budget,
        0,
    )?
    else {
        return Ok(false);
    };
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    Ok(signature_types_same_generic_call_binding(
        &substituted,
        caller_result,
        class_scope,
        caller_method_parameters,
    ))
}

pub fn same_class_generic_substituted_result(
    callee: &SameClassGenericMethodContract,
    substitution: &SameClassGenericSubstitution,
    budget: &mut Budget,
) -> Result<Option<SignatureType>> {
    let Some(result) = callee.result.as_ref() else {
        return Ok(None);
    };
    substitute_signature_type_parameters(
        result,
        &callee.method_parameters,
        &substitution.method_arguments,
        budget,
        0,
    )
}

pub fn same_class_generic_source_assignable_to_descriptor(
    source: &SignatureType,
    caller_method_parameters: &[TypeParameter],
    descriptor: &[u8],
    class_scope: &[TypeParameterErasure],
    class_parameters: &[TypeParameter],
    budget: &mut Budget,
) -> Result<Option<bool>> {
    let mut method_descriptor = Vec::with_capacity(descriptor.len().saturating_add(4));
    method_descriptor.push(b'(');
    method_descriptor.extend_from_slice(descriptor);
    method_descriptor.extend_from_slice(b")V");
    let parsed = match parse_method_signature(&method_descriptor, budget) {
        Ok(parsed) => parsed,
        Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => {
            return Err(error);
        }
        Err(_) => return Ok(None),
    };
    let [target] = parsed.parameters.as_slice() else {
        return Ok(None);
    };
    overload_source_type_assignable(
        source,
        target,
        caller_method_parameters,
        class_scope,
        class_parameters,
        budget,
        0,
    )
}

pub(crate) fn substitute_signature_type_parameters(
    source: &SignatureType,
    method_parameters: &[TypeParameter],
    method_arguments: &[SignatureType],
    budget: &mut Budget,
    depth: usize,
) -> Result<Option<SignatureType>> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if depth > 64 {
        return Ok(None);
    }
    match source {
        SignatureType::Base(_) => Ok(Some(source.clone())),
        SignatureType::Array(component) => Ok(substitute_signature_type_parameters(
            component,
            method_parameters,
            method_arguments,
            budget,
            depth + 1,
        )?
        .map(|component| SignatureType::Array(Box::new(component)))),
        SignatureType::TypeVariable(name) => {
            let mut binders = method_parameters
                .iter()
                .enumerate()
                .filter(|(_, binder)| binder.name == *name);
            let Some((index, _)) = binders.next() else {
                return Ok(Some(source.clone()));
            };
            if binders.next().is_some() {
                return Ok(None);
            }
            Ok(method_arguments.get(index).cloned())
        }
        SignatureType::Class(class) => {
            let mut result = class.clone();
            for segment in &mut result.segments {
                budget.poll()?;
                for argument in &mut segment.arguments {
                    let TypeArgument::Exact(source) = argument else {
                        return Ok(None);
                    };
                    let Some(substituted) = substitute_signature_type_parameters(
                        source,
                        method_parameters,
                        method_arguments,
                        budget,
                        depth + 1,
                    )?
                    else {
                        return Ok(None);
                    };
                    *source = substituted;
                }
            }
            Ok(Some(SignatureType::Class(result)))
        }
    }
}

fn same_class_call_type_matches(
    target: &SignatureType,
    actual: &SignatureType,
    class_scope: &[TypeParameterErasure],
    class_parameters: &[TypeParameter],
    callee_method_parameters: &[TypeParameter],
    caller_method_parameters: &[TypeParameter],
    substitutions: &mut [Option<SignatureType>],
    budget: &mut Budget,
    depth: usize,
) -> Result<bool> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if depth > 64 {
        return Ok(false);
    }
    match (target, actual) {
        (SignatureType::Base(left), SignatureType::Base(right)) => Ok(left == right),
        (SignatureType::Array(left), SignatureType::Array(right)) => same_class_call_type_matches(
            left,
            right,
            class_scope,
            class_parameters,
            callee_method_parameters,
            caller_method_parameters,
            substitutions,
            budget,
            depth + 1,
        ),
        (SignatureType::TypeVariable(target_name), _) => {
            if let Some((index, binder)) = callee_method_parameters
                .iter()
                .enumerate()
                .find(|(_, binder)| binder.name == *target_name)
            {
                if !same_class_call_type_satisfies_method_bounds(
                    actual,
                    binder,
                    class_scope,
                    class_parameters,
                    caller_method_parameters,
                    budget,
                )? {
                    return Ok(false);
                }
                let Some(previous) = substitutions.get_mut(index) else {
                    return Ok(false);
                };
                if let Some(previous) = previous {
                    Ok(signature_types_same_generic_call_binding(
                        previous,
                        actual,
                        class_scope,
                        caller_method_parameters,
                    ))
                } else {
                    *previous = Some(actual.clone());
                    Ok(true)
                }
            } else {
                let Some((target_index, _)) = class_scope
                    .iter()
                    .enumerate()
                    .find(|(_, binder)| binder.name == *target_name)
                else {
                    return Ok(false);
                };
                let SignatureType::TypeVariable(actual_name) = actual else {
                    return Ok(false);
                };
                if caller_method_parameters
                    .iter()
                    .any(|binder| binder.name == *actual_name)
                {
                    return Ok(false);
                }
                Ok(class_scope
                    .get(target_index)
                    .is_some_and(|binder| binder.name == *actual_name))
            }
        }
        (SignatureType::Class(target), SignatureType::Class(actual)) => {
            if target.segments.len() != actual.segments.len() {
                return Ok(false);
            }
            for (target, actual) in target.segments.iter().zip(&actual.segments) {
                if target.binary_name != actual.binary_name
                    || target.arguments.len() != actual.arguments.len()
                {
                    return Ok(false);
                }
                for (target_argument, actual_argument) in
                    target.arguments.iter().zip(&actual.arguments)
                {
                    let (TypeArgument::Exact(target), TypeArgument::Exact(actual)) =
                        (target_argument, actual_argument)
                    else {
                        return Ok(false);
                    };
                    if !same_class_call_type_matches(
                        target,
                        actual,
                        class_scope,
                        class_parameters,
                        callee_method_parameters,
                        caller_method_parameters,
                        substitutions,
                        budget,
                        depth + 1,
                    )? {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Generic invocation equality uses binder identity within one physical caller. A method binder
/// shadows a same-named class binder, while repeated references to the same method binder remain
/// equivalent. The older field-writer comparison intentionally keeps its narrower semantics.
fn signature_types_same_generic_call_binding(
    left: &SignatureType,
    right: &SignatureType,
    class_scope: &[TypeParameterErasure],
    method_binders: &[TypeParameter],
) -> bool {
    match (left, right) {
        (SignatureType::Base(left), SignatureType::Base(right)) => left == right,
        (SignatureType::Array(left), SignatureType::Array(right)) => {
            signature_types_same_generic_call_binding(left, right, class_scope, method_binders)
        }
        (SignatureType::TypeVariable(left), SignatureType::TypeVariable(right)) => {
            left == right
                && (method_binders.iter().any(|binder| binder.name == *left)
                    || (class_scope_variable(left, class_scope)
                        && !method_binders.iter().any(|binder| binder.name == *left)))
        }
        (SignatureType::Class(left), SignatureType::Class(right)) => {
            left.segments.len() == right.segments.len()
                && left
                    .segments
                    .iter()
                    .zip(&right.segments)
                    .all(|(left, right)| {
                        left.binary_name == right.binary_name
                            && left.arguments.len() == right.arguments.len()
                            && left
                                .arguments
                                .iter()
                                .zip(&right.arguments)
                                .all(|(left, right)| {
                                    matches!((left, right),
                                    (TypeArgument::Exact(left), TypeArgument::Exact(right))
                                        if signature_types_same_generic_call_binding(
                                            left, right, class_scope, method_binders
                                        ))
                                })
                    })
        }
        _ => false,
    }
}

fn same_class_call_type_satisfies_method_bounds(
    actual: &SignatureType,
    target: &TypeParameter,
    class_scope: &[TypeParameterErasure],
    class_parameters: &[TypeParameter],
    caller_method_parameters: &[TypeParameter],
    budget: &mut Budget,
) -> Result<bool> {
    let mut required = Vec::new();
    if let Some(bound) = &target.class_bound {
        required.push(bound);
    }
    required.extend(target.interface_bounds.iter());
    if required.is_empty() {
        return Ok(matches!(
            actual,
            SignatureType::TypeVariable(_) | SignatureType::Class(_)
        ));
    }
    for bound in required {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let SignatureType::Class(required_class) = bound else {
            return Ok(false);
        };
        let Some(required_segment) = required_class.segments.last() else {
            return Ok(false);
        };
        if required_class
            .segments
            .iter()
            .any(|segment| !segment.arguments.is_empty())
        {
            return Ok(false);
        }
        let required_name = required_segment.binary_name.as_slice();
        if required_name == b"java/lang/Object" {
            continue;
        }
        let actual_bound_matches = match actual {
            SignatureType::Class(actual_class) => actual_class
                .segments
                .last()
                .is_some_and(|segment| segment.binary_name.as_slice() == required_name),
            SignatureType::TypeVariable(actual_name) => caller_method_parameters
                .iter()
                .find(|binder| binder.name == *actual_name)
                .or_else(|| {
                    class_scope
                        .iter()
                        .position(|binder| binder.name == *actual_name)
                        .and_then(|index| class_parameters.get(index))
                })
                .is_some_and(|actual_binder| {
                    std::iter::once(actual_binder.class_bound.as_ref())
                        .flatten()
                        .chain(actual_binder.interface_bounds.iter())
                        .any(|bound| {
                            matches!(bound, SignatureType::Class(class)
                                if class.segments.last().is_some_and(|segment| segment.binary_name.as_slice() == required_name)
                                    && class.segments.iter().all(|segment| segment.arguments.is_empty()))
                        })
                }),
            SignatureType::Base(_) | SignatureType::Array(_) => false,
        };
        if !actual_bound_matches {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Prove that a source invocation remains pinned to its physical target after adding only
/// explicit-bound reference upcasts. This is intentionally a finite proof: it accepts a complete
/// same-class overload inventory on an `Object` subclass with no interfaces, excludes varargs,
/// bridge and synthetic declarations, and resolves only the JDK reference relationships listed in
/// `closed_jdk8_reference_relation`. An unknown relationship makes the whole proof unavailable;
/// the positive-only widening helper is not used as a negative fact.
pub fn prove_same_class_overload_argument_upcasts(
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_parameters: &[TypeParameter],
    caller: &PhysicalMethodId,
    target: &jarde_java::facts::CallTarget,
    call_bci: u32,
    overloads_complete: bool,
    actuals: &[SameClassOverloadActual],
    overloads: &[SameClassOverloadCandidate],
    budget: &mut Budget,
) -> Result<Option<Vec<jarde_java::report::ClassSourceInvokeArgumentCast>>> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if !overloads_complete
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
        || target.raw_owner().0.as_slice() != class_internal
        || target.is_interface_reference()
        || !matches!(
            target.kind(),
            jarde_java::facts::InvokeKind::Static
                | jarde_java::facts::InvokeKind::Virtual
                | jarde_java::facts::InvokeKind::Special
        )
        || target.name().starts_with('<')
        // The closed candidate slice contains declarations from this class only. Object's
        // inherited instance methods are additional source candidates even when this class has no
        // interfaces, so their names are outside this proof.
        || is_object_instance_method_name(target.raw_name().0.as_slice())
        || overloads.is_empty()
    {
        return Ok(None);
    }
    let expected_static = target.kind() == jarde_java::facts::InvokeKind::Static;
    let mut selected = None;
    let mut seen_methods = std::collections::HashSet::new();
    for (index, candidate) in overloads.iter().enumerate() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !candidate.source_writable
            || candidate.owner != class_internal
            || candidate.method.owner != caller.owner
            || candidate.method.name.0.as_slice() != target.raw_name().0.as_slice()
            || candidate.descriptor.as_slice() != candidate.method.descriptor.0.as_slice()
            || candidate.access_flags & (ACC_VARARGS | 0x0040 | 0x1000) != 0
            || (candidate.access_flags & ACC_STATIC != 0) != expected_static
            || !seen_methods.insert(candidate.method.clone())
        {
            return Ok(None);
        }
        let Some(parameter_descriptors) = overload_parameter_descriptors(&candidate.descriptor)
        else {
            return Ok(None);
        };
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(parameter_descriptors.len()).unwrap_or(u64::MAX),
        )?;
        if parameter_descriptors.len() != candidate.parameters.len() {
            return Ok(None);
        }
        if candidate.method.name.0.as_slice() == target.raw_name().0.as_slice()
            && candidate.descriptor.as_slice() == target.raw_descriptor().0.as_slice()
            && target.raw_owner().0.as_slice() == candidate.owner
        {
            if selected.replace(index).is_some() {
                return Ok(None);
            }
        }
    }
    let Some(selected) = selected else {
        return Ok(None);
    };
    let caller_method_parameters = overload_actual_method_parameters(caller, actuals, budget)?;
    if caller_method_parameters.is_none() && actuals.iter().any(|actual| actual.source.is_some()) {
        return Ok(None);
    }
    let caller_method_parameters = caller_method_parameters.unwrap_or_default();
    let mut actual_types = Vec::with_capacity(actuals.len());
    for actual in actuals {
        budget.poll()?;
        match (&actual.source, actual.null_literal) {
            (Some(source), false) if source.caller == *caller => {
                actual_types.push(Some(source.source_type.clone()));
            }
            (None, true) => actual_types.push(None),
            _ => return Ok(None),
        }
    }
    let uncast_resolution = overload_target_is_unique_most_specific(
        selected,
        overloads,
        &actual_types,
        &caller_method_parameters,
        class_scope,
        class_parameters,
        budget,
    )?;
    if uncast_resolution == Some(true) {
        return Ok(Some(Vec::new()));
    }

    // A cast is considered only where the target's physical parameter descriptor is exactly one
    // of this direct formal's explicit reference bounds. Every other argument keeps its proved
    // source type. This prevents an erasure or `Object` type from being used to invent `T`.
    let Some(selected_parameter_descriptors) =
        overload_parameter_descriptors(&overloads[selected].descriptor)
    else {
        return Ok(None);
    };
    let mut cast_choices = Vec::new();
    for (argument_index, actual) in actuals.iter().enumerate() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let Some(source) = &actual.source else {
            continue;
        };
        let SignatureType::TypeVariable(name) = &source.source_type else {
            continue;
        };
        let Some(parameter_descriptor) = selected_parameter_descriptors.get(argument_index) else {
            return Ok(None);
        };
        let Some(target_name) = descriptor_reference_name(parameter_descriptor) else {
            continue;
        };
        if !explicit_reference_bound_matches(
            name,
            &target_name,
            &source.caller_method_parameters,
            class_scope,
            class_parameters,
            budget,
        )? {
            continue;
        }
        let Some(source_name) = crate::names::source_class_name(&JvmBytes(target_name.clone()))
        else {
            continue;
        };
        // Overload resolution still compares physical MUTF-8 names. Only the emitted cast uses
        // source text; re-encoding that text here would change supplementary-name identity.
        let source_type = SignatureType::Class(jarde_reader::signature::ClassType {
            segments: vec![jarde_reader::signature::ClassTypeSegment {
                binary_name: target_name,
                arguments: Vec::new(),
            }],
        });
        cast_choices.push((
            argument_index,
            source_type,
            jarde_java::ast::Type::Reference(source_name),
        ));
    }
    if cast_choices.is_empty() {
        return Ok(None);
    }

    // Apply the closed set together, then remove each cast whose absence still leaves this target
    // uniquely most-specific. This is polynomial in the descriptor arity and keeps only edits
    // that the proof needs; there is no powerset search over JVM arguments.
    let mut keep = vec![true; cast_choices.len()];
    let mut candidate_actuals = actual_types.clone();
    for (argument_index, source_type, _) in &cast_choices {
        let Some(actual) = candidate_actuals.get_mut(*argument_index) else {
            return Ok(None);
        };
        *actual = Some(source_type.clone());
    }
    let cast_resolution = overload_target_is_unique_most_specific(
        selected,
        overloads,
        &candidate_actuals,
        &caller_method_parameters,
        class_scope,
        class_parameters,
        budget,
    )?;
    if cast_resolution != Some(true) {
        return Ok(None);
    }
    loop {
        let mut changed = false;
        for (choice_index, (argument_index, _, _)) in cast_choices.iter().enumerate() {
            if !keep[choice_index] {
                continue;
            }
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            let Some(original) = actual_types.get(*argument_index) else {
                return Ok(None);
            };
            {
                let Some(actual) = candidate_actuals.get_mut(*argument_index) else {
                    return Ok(None);
                };
                *actual = original.clone();
            }
            if overload_target_is_unique_most_specific(
                selected,
                overloads,
                &candidate_actuals,
                &caller_method_parameters,
                class_scope,
                class_parameters,
                budget,
            )? == Some(true)
            {
                keep[choice_index] = false;
                changed = true;
            } else {
                let Some(actual) = candidate_actuals.get_mut(*argument_index) else {
                    return Ok(None);
                };
                *actual = Some(cast_choices[choice_index].1.clone());
            }
        }
        if !changed {
            break;
        }
    }
    let solution: Vec<_> = keep
        .iter()
        .enumerate()
        .filter_map(|(index, keep)| keep.then_some(index))
        .collect();
    if solution.is_empty() {
        return Ok(None);
    }
    let opcode = match target.kind() {
        jarde_java::facts::InvokeKind::Virtual => 0xb6,
        jarde_java::facts::InvokeKind::Special => 0xb7,
        jarde_java::facts::InvokeKind::Static => 0xb8,
        jarde_java::facts::InvokeKind::Interface => return Ok(None),
    };
    Ok(Some(
        solution
            .into_iter()
            .map(|choice| {
                let (argument_index, _, ty) = &cast_choices[choice];
                jarde_java::report::ClassSourceInvokeArgumentCast {
                    call_bci,
                    opcode,
                    target: target.clone(),
                    argument_index: *argument_index,
                    ty: ty.clone(),
                }
            })
            .collect(),
    ))
}

fn overload_actual_method_parameters(
    caller: &PhysicalMethodId,
    actuals: &[SameClassOverloadActual],
    budget: &mut Budget,
) -> Result<Option<Vec<TypeParameter>>> {
    let mut result: Option<Vec<TypeParameter>> = None;
    for actual in actuals {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let Some(source) = &actual.source else {
            continue;
        };
        if source.caller != *caller {
            return Ok(None);
        }
        if result
            .as_ref()
            .is_some_and(|parameters| parameters != &source.caller_method_parameters)
        {
            return Ok(None);
        }
        result.get_or_insert_with(|| source.caller_method_parameters.clone());
    }
    Ok(result)
}

fn overload_parameter_descriptors(descriptor: &[u8]) -> Option<Vec<Vec<u8>>> {
    let facts = descriptor_facts(descriptor, DescriptorKind::Method).ok()?;
    facts
        .parameters()
        .iter()
        .map(|parameter| parameter.bytes(descriptor).map(<[u8]>::to_vec))
        .collect()
}

fn descriptor_reference_name(descriptor: &[u8]) -> Option<Vec<u8>> {
    if descriptor.first() != Some(&b'L') || descriptor.last() != Some(&b';') {
        return None;
    }
    let name = descriptor.get(1..descriptor.len().checked_sub(1)?)?;
    (!name.is_empty() && !name.contains(&b'[')).then(|| name.to_vec())
}

fn explicit_reference_bound_matches(
    variable: &[u8],
    target: &[u8],
    caller_method_parameters: &[TypeParameter],
    class_scope: &[TypeParameterErasure],
    class_parameters: &[TypeParameter],
    budget: &mut Budget,
) -> Result<bool> {
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            caller_method_parameters
                .len()
                .saturating_add(class_scope.len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    let mut method_matches = caller_method_parameters
        .iter()
        .filter(|parameter| parameter.name == variable);
    let binder = if let Some(parameter) = method_matches.next() {
        if method_matches.next().is_some() {
            return Ok(false);
        }
        parameter
    } else {
        let mut matches = class_scope
            .iter()
            .enumerate()
            .filter(|(_, parameter)| parameter.name == variable);
        let Some((index, _)) = matches.next() else {
            return Ok(false);
        };
        if matches.next().is_some() {
            return Ok(false);
        }
        let Some(parameter) = class_parameters.get(index) else {
            return Ok(false);
        };
        parameter
    };
    for bound in std::iter::once(binder.class_bound.as_ref())
        .flatten()
        .chain(binder.interface_bounds.iter())
    {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if matches!(bound, SignatureType::Class(class)
            if signature_class_erasure(bound) == Some(target)
                && !class.segments.is_empty())
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn overload_target_is_unique_most_specific(
    selected: usize,
    overloads: &[SameClassOverloadCandidate],
    actuals: &[Option<SignatureType>],
    caller_method_parameters: &[TypeParameter],
    class_scope: &[TypeParameterErasure],
    class_parameters: &[TypeParameter],
    budget: &mut Budget,
) -> Result<Option<bool>> {
    let mut applicable = Vec::new();
    for (index, candidate) in overloads.iter().enumerate() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let applicability = overload_candidate_applicability(
            candidate,
            actuals,
            caller_method_parameters,
            class_scope,
            class_parameters,
            budget,
        )?;
        match applicability {
            Some(Some(parameters)) => applicable.push((index, parameters)),
            Some(None) => {}
            None => return Ok(None),
        }
    }
    let Some(selected_parameters) = applicable
        .iter()
        .find(|(index, _)| *index == selected)
        .map(|(_, parameters)| parameters.clone())
    else {
        return Ok(Some(false));
    };
    for (index, parameters) in applicable {
        budget.poll()?;
        if index == selected {
            continue;
        }
        let mut strict = false;
        for (selected_type, other_type) in selected_parameters.iter().zip(&parameters) {
            let selected_to_other = overload_source_type_assignable(
                selected_type,
                other_type,
                caller_method_parameters,
                class_scope,
                class_parameters,
                budget,
                0,
            )?;
            let other_to_selected = overload_source_type_assignable(
                other_type,
                selected_type,
                caller_method_parameters,
                class_scope,
                class_parameters,
                budget,
                0,
            )?;
            match (selected_to_other, other_to_selected) {
                (Some(true), Some(false)) => strict = true,
                (Some(true), Some(true)) => {}
                (Some(false), _) | (_, Some(false)) => return Ok(Some(false)),
                _ => return Ok(None),
            }
        }
        if !strict {
            return Ok(Some(false));
        }
    }
    Ok(Some(true))
}

fn overload_candidate_applicability(
    candidate: &SameClassOverloadCandidate,
    actuals: &[Option<SignatureType>],
    caller_method_parameters: &[TypeParameter],
    class_scope: &[TypeParameterErasure],
    class_parameters: &[TypeParameter],
    budget: &mut Budget,
) -> Result<Option<Option<Vec<SignatureType>>>> {
    if candidate.parameters.len() != actuals.len() {
        return Ok(Some(None));
    }
    if !overload_candidate_binders_are_closed(candidate, budget)? {
        return Ok(None);
    }
    let mut method_arguments: Vec<Option<SignatureType>> =
        vec![None; candidate.method_parameters.len()];
    for (parameter, actual) in candidate.parameters.iter().zip(actuals) {
        budget.poll()?;
        match actual {
            Some(actual) => {
                if !overload_infer_method_arguments(
                    parameter,
                    actual,
                    &candidate.method_parameters,
                    &mut method_arguments,
                    class_scope,
                    caller_method_parameters,
                    budget,
                    0,
                )? {
                    return Ok(Some(None));
                }
            }
            None if !candidate.method_parameters.is_empty()
                && overload_type_mentions_method_binder(
                    parameter,
                    &candidate.method_parameters,
                    budget,
                )? =>
            {
                // A generic invocation with a null argument can infer from target typing or other
                // constraints. This proof does not model those constraints, so it cannot use null
                // to declare the overload inapplicable.
                return Ok(None);
            }
            None => {}
        }
    }
    if method_arguments.iter().any(Option::is_none) {
        return Ok(Some(None));
    }
    let method_arguments: Vec<_> = method_arguments.into_iter().flatten().collect();
    for (index, argument) in method_arguments.iter().enumerate() {
        let Some(binder) = candidate.method_parameters.get(index) else {
            return Ok(Some(None));
        };
        match overload_method_argument_satisfies_bounds(
            argument,
            binder,
            &candidate.method_parameters,
            &method_arguments,
            class_scope,
            class_parameters,
            caller_method_parameters,
            budget,
        )? {
            Some(true) => {}
            Some(false) => return Ok(Some(None)),
            None => return Ok(None),
        }
    }
    let mut instantiated = Vec::with_capacity(candidate.parameters.len());
    for (parameter, actual) in candidate.parameters.iter().zip(actuals) {
        let Some(parameter) = substitute_signature_type_parameters(
            parameter,
            &candidate.method_parameters,
            &method_arguments,
            budget,
            0,
        )?
        else {
            return Ok(Some(None));
        };
        if let Some(actual) = actual {
            match overload_source_type_assignable(
                actual,
                &parameter,
                caller_method_parameters,
                class_scope,
                class_parameters,
                budget,
                0,
            )? {
                Some(true) => {}
                Some(false) => return Ok(Some(None)),
                None => return Ok(None),
            }
        } else if !overload_reference_type(&parameter) {
            return Ok(Some(None));
        }
        instantiated.push(parameter);
    }
    Ok(Some(Some(instantiated)))
}

/// The overload proof models method inference only when each method binder occurs once as a
/// complete formal parameter. Repeated/nested inference constraints can choose a common type or
/// depend on target typing; treating those shapes as inapplicable would be an unsafe negative.
fn overload_candidate_binders_are_closed(
    candidate: &SameClassOverloadCandidate,
    budget: &mut Budget,
) -> Result<bool> {
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            candidate
                .parameters
                .len()
                .saturating_add(candidate.method_parameters.len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    let mut names = std::collections::HashSet::new();
    for binder in &candidate.method_parameters {
        budget.poll()?;
        if !names.insert(binder.name.as_slice()) {
            return Ok(false);
        }
        let mut direct_count = 0;
        for parameter in &candidate.parameters {
            budget.poll()?;
            if overload_type_mentions_method_binder(
                parameter,
                &candidate.method_parameters,
                budget,
            )? {
                if matches!(parameter, SignatureType::TypeVariable(name) if name == &binder.name) {
                    direct_count += 1;
                } else {
                    // The broad predicate identifies at least one method binder. A direct binder
                    // can coexist with other independent parameters, but nested forms are opaque.
                    let mentions_this =
                        overload_type_mentions_one_method_binder(parameter, &binder.name, budget)?;
                    if mentions_this {
                        return Ok(false);
                    }
                }
            }
        }
        if direct_count != 1 {
            return Ok(false);
        }
    }
    Ok(true)
}

fn overload_type_mentions_one_method_binder(
    ty: &SignatureType,
    name: &[u8],
    budget: &mut Budget,
) -> Result<bool> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    match ty {
        SignatureType::TypeVariable(candidate) => Ok(candidate == name),
        SignatureType::Array(component) => {
            overload_type_mentions_one_method_binder(component, name, budget)
        }
        SignatureType::Class(class) => {
            for segment in &class.segments {
                budget.poll()?;
                for argument in &segment.arguments {
                    let present = match argument {
                        TypeArgument::Exact(ty)
                        | TypeArgument::Extends(ty)
                        | TypeArgument::Super(ty) => {
                            overload_type_mentions_one_method_binder(ty, name, budget)?
                        }
                        TypeArgument::Any => false,
                    };
                    if present {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
        SignatureType::Base(_) => Ok(false),
    }
}

fn overload_method_argument_satisfies_bounds(
    actual: &SignatureType,
    binder: &TypeParameter,
    method_parameters: &[TypeParameter],
    method_arguments: &[SignatureType],
    class_scope: &[TypeParameterErasure],
    class_parameters: &[TypeParameter],
    caller_method_parameters: &[TypeParameter],
    budget: &mut Budget,
) -> Result<Option<bool>> {
    let mut bounds = Vec::new();
    for bound in std::iter::once(binder.class_bound.as_ref())
        .flatten()
        .chain(binder.interface_bounds.iter())
    {
        let Some(bound) = substitute_signature_type_parameters(
            bound,
            method_parameters,
            method_arguments,
            budget,
            0,
        )?
        else {
            return Ok(None);
        };
        bounds.push(bound);
    }
    if bounds.is_empty() {
        return Ok(overload_reference_type(actual).then_some(true));
    }
    let mut unknown = false;
    for bound in &bounds {
        match overload_source_type_assignable(
            actual,
            bound,
            caller_method_parameters,
            class_scope,
            class_parameters,
            budget,
            0,
        )? {
            Some(true) => {}
            Some(false) => return Ok(Some(false)),
            None => unknown = true,
        }
    }
    Ok((!unknown).then_some(true))
}

fn overload_infer_method_arguments(
    parameter: &SignatureType,
    actual: &SignatureType,
    method_parameters: &[TypeParameter],
    substitutions: &mut [Option<SignatureType>],
    class_scope: &[TypeParameterErasure],
    caller_method_parameters: &[TypeParameter],
    budget: &mut Budget,
    depth: usize,
) -> Result<bool> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if depth > 64 {
        return Ok(false);
    }
    if let SignatureType::TypeVariable(name) = parameter {
        let mut binders = method_parameters
            .iter()
            .enumerate()
            .filter(|(_, binder)| binder.name == *name);
        if let Some((index, _)) = binders.next() {
            if binders.next().is_some() {
                return Ok(false);
            }
            let Some(slot) = substitutions.get_mut(index) else {
                return Ok(false);
            };
            if let Some(previous) = slot {
                return Ok(signature_types_same_generic_call_binding(
                    previous,
                    actual,
                    class_scope,
                    caller_method_parameters,
                ));
            }
            *slot = Some(actual.clone());
            return Ok(true);
        }
    }
    match (parameter, actual) {
        (SignatureType::Array(parameter), SignatureType::Array(actual)) => {
            overload_infer_method_arguments(
                parameter,
                actual,
                method_parameters,
                substitutions,
                class_scope,
                caller_method_parameters,
                budget,
                depth + 1,
            )
        }
        (SignatureType::Class(parameter), SignatureType::Class(actual))
            if parameter.segments.len() == actual.segments.len() =>
        {
            for (parameter, actual) in parameter.segments.iter().zip(&actual.segments) {
                if parameter.binary_name != actual.binary_name
                    || parameter.arguments.len() != actual.arguments.len()
                {
                    return Ok(true);
                }
                for (parameter, actual) in parameter.arguments.iter().zip(&actual.arguments) {
                    let (TypeArgument::Exact(parameter), TypeArgument::Exact(actual)) =
                        (parameter, actual)
                    else {
                        return Ok(false);
                    };
                    if !overload_infer_method_arguments(
                        parameter,
                        actual,
                        method_parameters,
                        substitutions,
                        class_scope,
                        caller_method_parameters,
                        budget,
                        depth + 1,
                    )? {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn overload_type_mentions_method_binder(
    ty: &SignatureType,
    method_parameters: &[TypeParameter],
    budget: &mut Budget,
) -> Result<bool> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    match ty {
        SignatureType::TypeVariable(name) => Ok(method_parameters
            .iter()
            .any(|parameter| parameter.name == *name)),
        SignatureType::Array(component) => {
            overload_type_mentions_method_binder(component, method_parameters, budget)
        }
        SignatureType::Class(class) => {
            for segment in &class.segments {
                budget.poll()?;
                for argument in &segment.arguments {
                    let mentioned = match argument {
                        TypeArgument::Exact(ty) => {
                            overload_type_mentions_method_binder(ty, method_parameters, budget)?
                        }
                        TypeArgument::Any | TypeArgument::Extends(_) | TypeArgument::Super(_) => {
                            true
                        }
                    };
                    if mentioned {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
        SignatureType::Base(_) => Ok(false),
    }
}

fn overload_reference_type(ty: &SignatureType) -> bool {
    matches!(
        ty,
        SignatureType::TypeVariable(_) | SignatureType::Class(_) | SignatureType::Array(_)
    )
}

/// `Some(true/false)` is a closed answer; `None` means the finite hierarchy does not decide it.
pub fn overload_source_type_assignable(
    source: &SignatureType,
    target: &SignatureType,
    caller_method_parameters: &[TypeParameter],
    class_scope: &[TypeParameterErasure],
    class_parameters: &[TypeParameter],
    budget: &mut Budget,
    depth: usize,
) -> Result<Option<bool>> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    if depth > 64 {
        return Ok(None);
    }
    if overload_shared_shadowed_binder_name(
        source,
        target,
        caller_method_parameters,
        class_scope,
        budget,
    )? {
        // The same spelling can denote a caller method binder on one side and the class binder
        // on the other. Without preserving those two binder identities through the candidate
        // projection, equality would turn a shadowed `T` into a false overload match.
        return Ok(None);
    }
    if signature_types_same_generic_call_binding(
        source,
        target,
        class_scope,
        caller_method_parameters,
    ) {
        return Ok(Some(true));
    }
    match (source, target) {
        (SignatureType::Base(source), SignatureType::Base(target)) if source == target => {
            Ok(Some(true))
        }
        // Primitive widening, boxing and unboxing affect JLS applicability. They are outside this
        // reference-upcast proof, so an unequal/base-reference pair is unknown rather than a
        // negative applicability fact.
        (SignatureType::Base(_), _) | (_, SignatureType::Base(_)) => Ok(None),
        (SignatureType::Array(source), SignatureType::Array(target)) => {
            overload_source_type_assignable(
                source,
                target,
                caller_method_parameters,
                class_scope,
                class_parameters,
                budget,
                depth + 1,
            )
        }
        (SignatureType::Array(_), SignatureType::Class(target)) => Ok(signature_class_erasure(
            &SignatureType::Class(target.clone()),
        )
        .map(|name| {
            name == b"java/lang/Object"
                || name == b"java/lang/Cloneable"
                || name == b"java/io/Serializable"
        })),
        (SignatureType::Class(source), SignatureType::Class(target)) => {
            if source == target {
                return Ok(Some(true));
            }
            let source_signature = SignatureType::Class(source.clone());
            let Some(source_name) = signature_class_erasure(&source_signature) else {
                return Ok(None);
            };
            let target_signature = SignatureType::Class(target.clone());
            let Some(target_name) = signature_class_erasure(&target_signature) else {
                return Ok(None);
            };
            if target_name == b"java/lang/Object" {
                return Ok(Some(true));
            }
            // The one parameterized JDK edge in this closed proof is String's actual Java 8
            // `Comparable<String>` header. Wildcard conversions remain unknown below.
            if source_name == b"java/lang/String"
                && target_name == b"java/lang/Comparable"
                && target.segments.last().is_some_and(|segment| {
                    matches!(
                        segment.arguments.as_slice(),
                        [TypeArgument::Exact(SignatureType::Class(string))]
                            if signature_class_erasure(&SignatureType::Class(string.clone()))
                                == Some(b"java/lang/String".as_slice())
                                && string.segments.iter().all(|segment| segment.arguments.is_empty())
                    )
                })
            {
                return Ok(Some(true));
            }
            if source_name == target_name {
                if target
                    .segments
                    .iter()
                    .all(|segment| segment.arguments.is_empty())
                {
                    return Ok(Some(true));
                }
                if target.segments.iter().any(|segment| {
                    segment.arguments.iter().any(|argument| {
                        matches!(
                            argument,
                            TypeArgument::Any | TypeArgument::Extends(_) | TypeArgument::Super(_)
                        )
                    })
                }) {
                    return Ok(None);
                }
                return Ok(Some(false));
            }
            let raw_relation = closed_jdk8_reference_relation(source_name, target_name);
            if source
                .segments
                .iter()
                .any(|segment| !segment.arguments.is_empty())
                || target
                    .segments
                    .iter()
                    .any(|segment| !segment.arguments.is_empty())
            {
                return Ok(match raw_relation {
                    Some(false) => Some(false),
                    Some(true) | None => None,
                });
            }
            Ok(raw_relation)
        }
        (SignatureType::TypeVariable(name), SignatureType::Class(target)) => {
            let target = SignatureType::Class(target.clone());
            if signature_class_erasure(&target).as_deref() == Some(b"java/lang/Object") {
                return Ok(Some(true));
            }
            let Some(binder) = overload_find_binder(
                name,
                caller_method_parameters,
                class_scope,
                class_parameters,
                budget,
            )?
            else {
                return Ok(None);
            };
            let mut bounds = Vec::new();
            bounds.extend(binder.class_bound.iter());
            bounds.extend(binder.interface_bounds.iter());
            if bounds.is_empty() {
                return Ok(Some(false));
            }
            let mut unknown = false;
            for bound in bounds {
                let relation = overload_source_type_assignable(
                    bound,
                    &target,
                    caller_method_parameters,
                    class_scope,
                    class_parameters,
                    budget,
                    depth + 1,
                )?;
                match relation {
                    Some(true) => return Ok(Some(true)),
                    Some(false) => {}
                    None => unknown = true,
                }
            }
            Ok((!unknown).then_some(false))
        }
        (SignatureType::TypeVariable(_), SignatureType::Array(_)) => Ok(Some(false)),
        // Different variables can still be related through their bounds (for example
        // `U extends T`). This helper does not close variable-to-variable bounds, so it cannot
        // use this pair as negative overload-applicability evidence.
        (SignatureType::TypeVariable(_), SignatureType::TypeVariable(_)) => Ok(None),
        (SignatureType::Class(_), SignatureType::Array(_)) => Ok(Some(false)),
        (SignatureType::Array(_), SignatureType::TypeVariable(_)) => Ok(Some(false)),
        (SignatureType::Class(_), SignatureType::TypeVariable(_)) => Ok(Some(false)),
    }
}

fn overload_shared_shadowed_binder_name(
    source: &SignatureType,
    target: &SignatureType,
    caller_method_parameters: &[TypeParameter],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<bool> {
    let mut source_names = std::collections::HashSet::new();
    let mut target_names = std::collections::HashSet::new();
    overload_collect_type_variables(source, &mut source_names, budget)?;
    overload_collect_type_variables(target, &mut target_names, budget)?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(source_names.len()).unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    Ok(source_names.iter().any(|name| {
        target_names.contains(name)
            && caller_method_parameters
                .iter()
                .any(|parameter| parameter.name.as_slice() == *name)
            && class_scope
                .iter()
                .any(|parameter| parameter.name.as_slice() == *name)
    }))
}

fn overload_collect_type_variables<'a>(
    ty: &'a SignatureType,
    names: &mut std::collections::HashSet<&'a [u8]>,
    budget: &mut Budget,
) -> Result<()> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    match ty {
        SignatureType::TypeVariable(name) => {
            names.insert(name);
        }
        SignatureType::Array(component) => {
            overload_collect_type_variables(component, names, budget)?;
        }
        SignatureType::Class(class) => {
            for segment in &class.segments {
                budget.poll()?;
                for argument in &segment.arguments {
                    match argument {
                        TypeArgument::Exact(ty)
                        | TypeArgument::Extends(ty)
                        | TypeArgument::Super(ty) => {
                            overload_collect_type_variables(ty, names, budget)?;
                        }
                        TypeArgument::Any => {}
                    }
                }
            }
        }
        SignatureType::Base(_) => {}
    }
    Ok(())
}

fn overload_find_binder<'a>(
    name: &[u8],
    caller_method_parameters: &'a [TypeParameter],
    class_scope: &[TypeParameterErasure],
    class_parameters: &'a [TypeParameter],
    budget: &mut Budget,
) -> Result<Option<&'a TypeParameter>> {
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            caller_method_parameters
                .len()
                .saturating_add(class_scope.len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    let mut method = caller_method_parameters
        .iter()
        .filter(|parameter| parameter.name == name);
    if let Some(parameter) = method.next() {
        return Ok(method.next().is_none().then_some(parameter));
    }
    let mut class = class_scope
        .iter()
        .enumerate()
        .filter(|(_, parameter)| parameter.name == name);
    let Some((index, _)) = class.next() else {
        return Ok(None);
    };
    if class.next().is_some() {
        return Ok(None);
    }
    Ok(class_parameters.get(index))
}

/// Closed Java 8 header facts used by this proof: `Number` implements `Serializable` but not
/// `Comparable`; `String` implements `Serializable`, `Comparable` and `CharSequence`; all four
/// types inherit from `Object`. The caller's Java source dialect is Java 8, so later JDK interfaces
/// such as `Constable` and `ConstantDesc` are deliberately absent. `None` is unknown, never a
/// negative assignability claim.
fn closed_jdk8_reference_relation(source: &[u8], target: &[u8]) -> Option<bool> {
    const OBJECT: &[u8] = b"java/lang/Object";
    const NUMBER: &[u8] = b"java/lang/Number";
    const STRING: &[u8] = b"java/lang/String";
    const COMPARABLE: &[u8] = b"java/lang/Comparable";
    const SERIALIZABLE: &[u8] = b"java/io/Serializable";
    const CHAR_SEQUENCE: &[u8] = b"java/lang/CharSequence";
    if source == target || target == OBJECT {
        return Some(true);
    }
    let closed = [
        OBJECT,
        NUMBER,
        STRING,
        COMPARABLE,
        SERIALIZABLE,
        CHAR_SEQUENCE,
    ];
    if !closed.contains(&source) || !closed.contains(&target) {
        return None;
    }
    let assignable = match source {
        NUMBER => target == SERIALIZABLE,
        STRING => matches!(target, COMPARABLE | SERIALIZABLE | CHAR_SEQUENCE),
        COMPARABLE | SERIALIZABLE | CHAR_SEQUENCE => false,
        OBJECT => false,
        _ => false,
    };
    Some(assignable)
}

pub fn class_scope_variable(name: &[u8], class_scope: &[TypeParameterErasure]) -> bool {
    class_scope.iter().any(|parameter| parameter.name == name)
}

pub fn signature_class_erasure(ty: &SignatureType) -> Option<&[u8]> {
    let SignatureType::Class(class) = ty else {
        return None;
    };
    class
        .segments
        .last()
        .map(|segment| segment.binary_name.as_slice())
}

pub fn is_object_instance_method_name(name: &[u8]) -> bool {
    const OBJECT_INSTANCE_METHODS: &[&[u8]] = &[
        b"getClass",
        b"hashCode",
        b"equals",
        b"clone",
        b"toString",
        b"notify",
        b"notifyAll",
        b"wait",
        b"finalize",
    ];
    OBJECT_INSTANCE_METHODS.contains(&name)
}

// Append to the existing `jarde_java::generic_source_proof` module.
// It already defines the referenced SameClass* records and imports Budget, Result and reader facts.

#[allow(clippy::too_many_arguments)]
pub fn prove_generic_call_overload_site(
    invoke: &class_source::SameClassInvokeUse,
    caller: &class_source::SameClassGenericMethodContract,
    caller_ast: &jarde_java::report::ClassSourceMethodAst,
    caller_header: &MemberHeader,
    initial_init_record: Option<&jarde_java::init::InitRecord>,
    body_values: &[class_source::SameClassBodyValueSite],
    invokes: &[class_source::SameClassInvokeUse],
    nested_call_sources: &[class_source::SameClassGenericArgumentSource],
    class_internal: &[u8],
    class_scope: &[jarde_reader::signature::TypeParameterErasure],
    class_parameters: &[jarde_reader::signature::TypeParameter],
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    inventory_complete: bool,
    method_inventory_aligned: bool,
    budget: &mut Budget,
    mut provide_overloads: impl FnMut(
        &mut Budget,
    )
        -> Result<Option<Vec<class_source::SameClassOverloadCandidate>>>,
) -> Result<Option<Vec<jarde_java::report::ClassSourceInvokeArgumentCast>>> {
    use jarde_java::facts::InvokeKind;
    use jarde_java::report::{
        ClassSourceAstExpressionShape as Shape, class_source_invoke_ast_sites,
    };
    use jarde_jvm::method_ir::Definition;
    let init_record = initial_init_record;

    macro_rules! reject_overload_none {
        () => {{
            return Ok(None);
        }};
    }

    let Some(key) = generic_call_ast_key(invoke) else {
        reject_overload_none!();
    };

    if invoke.owner.as_slice() != class_internal
        || invoke.name.starts_with(b"<")
        || !inventory_complete
        || !method_inventory_aligned
        || !invoke.ssa_operands_complete
        || caller.method != invoke.physical_caller
        || caller.method.name.0 != caller_header.name.raw().0
        || caller.method.descriptor.0 != caller_header.descriptor.raw().0
    {
        reject_overload_none!();
    }
    let Some(sites) = class_source_invoke_ast_sites(caller_ast, std::slice::from_ref(&key), budget)
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "same-class overload AST projection",
                "generic_call_ast_missing",
            )
        })?
    else {
        reject_overload_none!();
    };
    let Some(site) = sites.first() else {
        reject_overload_none!();
    };
    if site.caller != invoke.physical_caller
        || site.key != key
        || source_identifier(&invoke.name).as_deref() != Some(site.ast_name.as_str())
    {
        reject_overload_none!();
    }
    let Some(body) = jarde_java::report::class_source_method_body_consumers(caller_ast, budget)
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "same-class overload caller scope proof",
                "generic_call_ast_missing",
            )
        })?
    else {
        reject_overload_none!();
    };
    let init_record = init_record.or(body.init_record.as_ref());
    if !generic_call_site_avoids_catch_shadow(&body, site, invoke.bci) {
        reject_overload_none!();
    }
    let Some(formals) = jarde_java::report::class_source_method_formal_names(caller_ast, budget)
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "same-class overload formal proof",
                "generic_call_ast_missing",
            )
        })?
    else {
        reject_overload_none!();
    };
    let formal_by_name = formals
        .iter()
        .map(|formal| (formal.name.as_str(), formal.slot))
        .collect::<std::collections::HashMap<_, _>>();
    if formal_by_name.len() != formals.len() {
        reject_overload_none!();
    }
    let descriptor = descriptor_facts(&caller.method.descriptor.0, DescriptorKind::Method)?;
    let Some(formal_slots) = jarde_jvm::method_ir::parameter_positions(
        &descriptor,
        caller_header.access_flags & 0x0008 != 0,
    ) else {
        reject_overload_none!();
    };
    if caller.parameters.len() != formal_slots.len()
        || site.arguments.len() != invoke.argument_value_facts.len()
    {
        reject_overload_none!();
    }

    // An implicit or explicit instance receiver still has to be the physical initialized `this`,
    // or an exact same-class parameterized formal. The AST's omitted receiver is not evidence.
    match key.target.kind() {
        InvokeKind::Static => {
            if site.receiver.is_some() || invoke.receiver_value_fact.is_some() {
                reject_overload_none!();
            }
        }
        InvokeKind::Virtual | InvokeKind::Special => {
            let Some(receiver_fact) = invoke.receiver_value_fact.as_ref() else {
                reject_overload_none!();
            };
            if let Some(receiver) = &site.receiver {
                if receiver.primary.method.as_ref() != Some(&caller.method)
                    || !matches!(receiver.shape, Shape::Local)
                {
                    reject_overload_none!();
                }
                if receiver.direct_local_name.as_deref() == Some("this") {
                    if !generic_call_receiver_is_initialized_this(
                        invoke,
                        receiver_fact,
                        caller_header,
                        class_internal,
                        class_superclass,
                        init_record,
                        body_values,
                        invokes,
                    ) {
                        reject_overload_none!();
                    }
                } else {
                    if caller_header.access_flags & 0x0008 != 0
                        || receiver_fact.replaced_by.is_some()
                        || receiver_fact.uses.as_slice() != [Some(invoke.bci)]
                    {
                        reject_overload_none!();
                    }
                    let Some(slot) = receiver
                        .direct_local_name
                        .as_deref()
                        .and_then(|name| formal_by_name.get(name).copied())
                    else {
                        reject_overload_none!();
                    };
                    if !generic_call_load_matches_formal(
                        body_values,
                        receiver_fact,
                        slot,
                        invoke.bci,
                    ) {
                        reject_overload_none!();
                    }
                    let Some(position) = formal_slots.iter().position(|formal| *formal == slot)
                    else {
                        reject_overload_none!();
                    };
                    if !same_class_parameterized_receiver(
                        &caller.parameters[position],
                        class_internal,
                        class_scope,
                        &caller.method_parameters,
                        budget,
                    )? {
                        reject_overload_none!();
                    }
                }
            } else if !generic_call_receiver_is_initialized_this(
                invoke,
                receiver_fact,
                caller_header,
                class_internal,
                class_superclass,
                init_record,
                body_values,
                invokes,
            ) {
                reject_overload_none!();
            }
        }
        InvokeKind::Interface => reject_overload_none!(),
    }

    let mut actuals = Vec::with_capacity(site.arguments.len());
    let target_descriptor = descriptor_facts(&invoke.descriptor, DescriptorKind::Method)?;
    for (argument_index, (outer_expression, fact)) in site
        .arguments
        .iter()
        .zip(&invoke.argument_value_facts)
        .enumerate()
    {
        budget.poll()?;
        let expression = if let Some(wrapper) = &outer_expression.presentation_wrapper {
            let Some(required) = target_descriptor
                .parameters()
                .get(argument_index)
                .and_then(|parameter| parameter.bytes(&invoke.descriptor))
                .and_then(generic_call_reference_type_from_descriptor)
            else {
                reject_overload_none!();
            };
            if wrapper.ty != required {
                reject_overload_none!();
            }
            wrapper.child.as_ref()
        } else {
            outer_expression
        };
        if expression.primary.method.as_ref() != Some(&caller.method)
            || fact.replaced_by.is_some()
            || fact.uses.as_slice() != [Some(invoke.bci)]
        {
            reject_overload_none!();
        }
        if expression.null_literal {
            if !generic_call_null_matches(body_values, fact, expression, invoke.bci) {
                reject_overload_none!();
            }
            actuals.push(class_source::SameClassOverloadActual {
                source: None,
                null_literal: true,
            });
            continue;
        }
        if matches!(expression.shape, Shape::Local) {
            let Some(name) = expression.direct_local_name.as_deref() else {
                reject_overload_none!();
            };
            let Some(slot) = formal_by_name.get(name).copied() else {
                reject_overload_none!();
            };
            if !generic_call_load_matches_formal(body_values, fact, slot, invoke.bci) {
                reject_overload_none!();
            }
            let Some(position) = formal_slots.iter().position(|formal| *formal == slot) else {
                reject_overload_none!();
            };
            actuals.push(class_source::SameClassOverloadActual {
                source: Some(class_source::SameClassGenericArgumentSource {
                    caller: caller.method.clone(),
                    origin: class_source::SameClassGenericArgumentOrigin::Formal { slot },
                    source_type: caller.parameters[position].clone(),
                    caller_method_parameters: caller.method_parameters.clone(),
                }),
                null_literal: false,
            });
        } else {
            let Shape::Call {
                name,
                argument_count,
            } = &expression.shape
            else {
                reject_overload_none!();
            };
            let Some(producer) = invokes.iter().find(|producer| {
                producer.physical_caller == caller.method && producer.bci == expression.primary.bci
            }) else {
                reject_overload_none!();
            };
            let Some(source) = nested_call_sources.iter().find(|source| {
                source.caller == caller.method
                    && matches!(
                        &source.origin,
                        class_source::SameClassGenericArgumentOrigin::NestedCall {
                            bci, opcode, owner, name: source_name, descriptor,
                        } if *bci == producer.bci
                            && *opcode == producer.opcode
                            && *owner == producer.owner
                            && *source_name == producer.name
                            && *descriptor == producer.descriptor
                    )
            }) else {
                reject_overload_none!();
            };
            let Some(producer_key) = generic_call_ast_key(producer) else {
                reject_overload_none!();
            };
            let Some(producer_sites) = class_source_invoke_ast_sites(
                caller_ast,
                std::slice::from_ref(&producer_key),
                budget,
            )
            .map_err(|stop| {
                enum_projection_stop_error(
                    stop,
                    "nested overload producer AST projection",
                    "generic_call_ast_missing",
                )
            })?
            else {
                reject_overload_none!();
            };
            let Some(producer_site) = producer_sites.first() else {
                reject_overload_none!();
            };
            let Some(producer_result) = producer.result_value_fact.as_ref() else {
                reject_overload_none!();
            };
            if expression.primary.method.as_ref() != Some(&caller.method)
                || source_identifier(&producer.name).as_deref() != Some(name.as_str())
                || usize::try_from(*argument_count).ok() != Some(producer_site.arguments.len())
                || fact.replaced_by.is_some()
                || fact.uses.as_slice() != [Some(invoke.bci)]
                || fact.value != producer_result.value
                || !matches!(fact.definition, Definition::Instruction { bci, .. } if bci == producer.bci)
            {
                reject_overload_none!();
            }
            actuals.push(class_source::SameClassOverloadActual {
                source: Some(source.clone()),
                null_literal: false,
            });
        }
    }

    let Some(overloads) = provide_overloads(budget)? else {
        reject_overload_none!();
    };

    if class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
        || key.target.raw_owner().0.as_slice() != class_internal
        || key.target.is_interface_reference()
        || key.target.kind() == InvokeKind::Interface
    {
        reject_overload_none!();
    }
    let Some(target_candidate) = overloads.iter().find(|candidate| {
        candidate.owner == class_internal
            && candidate.method.name.0 == key.target.raw_name().0.as_slice()
            && candidate.descriptor == key.target.raw_descriptor().0.as_slice()
    }) else {
        reject_overload_none!();
    };
    if !target_candidate.source_writable
        || target_candidate.method.owner != caller.method.owner
        || (target_candidate.access_flags & 0x0008 != 0) != (invoke.opcode == 0xb8)
        || overloads
            .iter()
            .any(|candidate| candidate.access_flags & (0x0040 | 0x1000 | 0x0080) != 0)
    {
        reject_overload_none!();
    }
    let same_arity = overloads
        .iter()
        .filter(|candidate| candidate.parameters.len() == actuals.len())
        .collect::<Vec<_>>();
    if !same_arity
        .iter()
        .any(|candidate| candidate.descriptor == key.target.raw_descriptor().0.as_slice())
    {
        reject_overload_none!();
    }
    if same_arity.len() == 1 {
        return Ok(Some(Vec::new()));
    }
    let result = class_source::prove_same_class_overload_argument_upcasts(
        class_internal,
        class_superclass,
        class_interfaces,
        class_scope,
        class_parameters,
        &caller.method,
        &key.target,
        invoke.bci,
        true,
        &actuals,
        &overloads,
        budget,
    )?;

    Ok(result)
}

pub fn prove_raw_same_class_void_incoming(
    invoke: &class_source::SameClassInvokeUse,
    caller: &PhysicalMethodId,
    caller_header: &MemberHeader,
    caller_ast: &jarde_java::report::ClassSourceMethodAst,
    body_values: &[class_source::SameClassBodyValueSite],
    method_headers: &[MemberHeader],
    callee: &class_source::SameClassGenericMethodContract,
    class_internal: &[u8],
    budget: &mut Budget,
    mut inventory_matches: impl FnMut(&mut Budget) -> Result<bool>,
) -> Result<bool> {
    use jarde_java::ast::Type;
    use jarde_java::report::ClassSourceAstExpressionShape as Shape;
    use jarde_jvm::method_ir::Definition;

    macro_rules! reject_raw_call {
        () => {{
            return Ok(false);
        }};
    }

    budget.poll()?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(method_headers.len().saturating_add(body_values.len())).unwrap_or(u64::MAX),
    )?;
    if invoke.physical_caller != *caller
        || invoke.owner.as_slice() != class_internal
        || !invoke.ssa_operands_complete
        || callee.method.name.0 != invoke.name
        || callee.method.descriptor.0 != invoke.descriptor
        || callee.result.is_some()
        || !matches!(invoke.opcode, 0xb6 | 0xb7)
        || caller_header.access_flags & 0x0008 != 0 && invoke.receiver_value_fact.is_none()
    {
        reject_raw_call!();
    }
    let target_headers = method_headers.iter().filter(|header| {
        header.name.raw().0 == invoke.name
            && header.descriptor.raw().0 == invoke.descriptor
            && header.access_flags & 0x0008 == 0
    });
    let mut target_headers = target_headers;
    if target_headers.next().is_none() || target_headers.next().is_some() {
        reject_raw_call!();
    }
    let Some(target_descriptor) = descriptor_facts(&invoke.descriptor, DescriptorKind::Method).ok()
    else {
        reject_raw_call!();
    };
    let target_arity = target_descriptor.parameters().len();
    let mut same_arity = 0usize;
    for header in method_headers {
        budget.poll()?;
        if header.name.raw().0 != invoke.name {
            continue;
        }
        if header.access_flags & (0x0080 | 0x0040 | 0x1000) != 0 {
            reject_raw_call!();
        }
        let Ok(descriptor) = descriptor_facts(&header.descriptor.raw().0, DescriptorKind::Method)
        else {
            reject_raw_call!();
        };
        if descriptor.parameters().len() == target_arity {
            same_arity = same_arity.saturating_add(1);
        }
    }
    if same_arity != 1 {
        reject_raw_call!();
    }
    let Some(key) = generic_call_ast_key(invoke) else {
        reject_raw_call!();
    };
    let Some(sites) = jarde_java::report::class_source_invoke_ast_sites(
        caller_ast,
        std::slice::from_ref(&key),
        budget,
    )
    .map_err(|stop| {
        enum_projection_stop_error(
            stop,
            "same-class raw receiver AST proof",
            "generic_call_ast_missing",
        )
    })?
    else {
        reject_raw_call!();
    };
    let Some(site) = sites.first() else {
        reject_raw_call!();
    };
    if sites.len() != 1
        || site.caller != *caller
        || site.key != key
        || source_identifier(&invoke.name).as_deref() != Some(site.ast_name.as_str())
        || !inventory_matches(budget)?
    {
        reject_raw_call!();
    }
    let Some(receiver) = site.receiver.as_ref() else {
        reject_raw_call!();
    };
    if !matches!(receiver.shape, Shape::Local) || receiver.primary.method.as_ref() != Some(caller) {
        reject_raw_call!();
    }
    let Some(receiver_name) = receiver.direct_local_name.as_deref() else {
        reject_raw_call!();
    };
    let Some(receiver_fact) = invoke.receiver_value_fact.as_ref() else {
        reject_raw_call!();
    };
    let Some((load_bci, (slot, local_read))) =
        generic_call_load_source(body_values, receiver_fact, invoke.bci)
    else {
        reject_raw_call!();
    };
    if load_bci != receiver.primary.bci
        || local_read.replaced_by.is_some()
        || receiver_fact.uses.as_slice() != [Some(invoke.bci)]
    {
        reject_raw_call!();
    }
    let Some(declarations) = jarde_java::report::class_source_local_declarations(
        caller_ast, budget,
    )
    .map_err(|stop| {
        enum_projection_stop_error(
            stop,
            "same-class raw receiver local proof",
            "generic_call_ast_missing",
        )
    })?
    else {
        reject_raw_call!();
    };
    let mut matching_declarations = declarations
        .iter()
        .filter(|declaration| declaration.name == receiver_name);
    let Some(declaration) = matching_declarations.next() else {
        reject_raw_call!();
    };
    if matching_declarations.next().is_some()
        || !declaration.scope_anchors.is_empty()
        || declaration.bci >= load_bci
        || !generic_call_local_definition_matches_declaration(
            body_values,
            local_read,
            *slot,
            declaration.bci,
            load_bci,
        )
    {
        reject_raw_call!();
    }
    // This local belongs to the current physical source unit, whose declaration and body keep
    // its binary name. A `$` alone states no member edge; use the same faithful spelling as the
    // body rather than treating classfile MUTF-8 as UTF-8 or inventing a nesting boundary.
    let expected_full = crate::names::source_class_name(&JvmBytes(class_internal.to_vec()));
    let expected_simple = expected_full
        .as_deref()
        .and_then(|name| name.rsplit('.').next());
    let raw_name = declaration
        .source_type_name
        .as_deref()
        .or(match &declaration.ty {
            Type::Reference(name) => Some(name.as_str()),
            _ => None,
        });
    let binary_name_matches = raw_name.is_some_and(|name| {
        Some(name) == expected_full.as_deref() || Some(name) == expected_simple
    });
    if !binary_name_matches {
        // A real non-generic member local may carry a selected source path while its semantic
        // Type still states the binary name. Close that spelling against the same AST's raw
        // InnerClasses rows; a literal `$` must never create this alternative on its own.
        let names = caller_ast.projection.source_names();
        budget.poll()?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(
                names
                    .inner_classes
                    .len()
                    .saturating_mul(class_internal.len()),
            )
            .unwrap_or(u64::MAX),
        )?;
        let member_path = names.stated_path(class_internal).map(|(path, _)| path);
        let package_prefix = expected_full
            .as_deref()
            .and_then(|name| name.rsplit_once('.').map(|(package, _)| package));
        let member_simple = member_path
            .as_deref()
            .and_then(|path| match package_prefix {
                Some(package) => path.strip_prefix(package)?.strip_prefix('.'),
                None => Some(path),
            });
        if !raw_name
            .is_some_and(|name| Some(name) == member_path.as_deref() || Some(name) == member_simple)
        {
            reject_raw_call!();
        }
    }
    let Some(formals) = jarde_java::report::class_source_method_formal_names(caller_ast, budget)
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "same-class raw receiver formal scope",
                "generic_call_ast_missing",
            )
        })?
    else {
        reject_raw_call!();
    };
    if formals.iter().any(|formal| formal.name == receiver_name) {
        reject_raw_call!();
    }
    let caller_descriptor = match descriptor_facts(&caller.descriptor.0, DescriptorKind::Method) {
        Ok(descriptor) => descriptor,
        Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => return Err(error),
        Err(_) => reject_raw_call!(),
    };
    let formal_slots = jarde_jvm::method_ir::parameter_positions(
        &caller_descriptor,
        caller_header.access_flags & 0x0008 != 0,
    );
    let Some(formal_slots) = formal_slots else {
        reject_raw_call!();
    };
    if formal_slots.contains(slot)
        || site.arguments.len() != target_arity
        || invoke.argument_value_facts.len() != target_arity
    {
        reject_raw_call!();
    }
    for (position, (argument, fact)) in site
        .arguments
        .iter()
        .zip(&invoke.argument_value_facts)
        .enumerate()
    {
        budget.poll()?;
        if fact.replaced_by.is_some() || fact.uses.as_slice() != [Some(invoke.bci)] {
            reject_raw_call!();
        }
        let Some(parameter) = target_descriptor.parameters().get(position) else {
            reject_raw_call!();
        };
        let Some(raw_descriptor) = parameter.bytes(&invoke.descriptor) else {
            reject_raw_call!();
        };
        let expected = match raw_descriptor {
            b"Z" => Type::Boolean,
            b"B" => Type::Byte,
            b"C" => Type::Char,
            b"S" => Type::Short,
            b"I" => Type::Int,
            b"J" => Type::Long,
            b"F" => Type::Float,
            b"D" => Type::Double,
            _ => generic_call_reference_type_from_descriptor(raw_descriptor)
                .unwrap_or_else(|| Type::Reference(String::new())),
        };
        if matches!(&expected, Type::Reference(name) if name.is_empty()) {
            reject_raw_call!();
        }
        let source_expression = if let Some(wrapper) = &argument.presentation_wrapper {
            if wrapper.ty != expected {
                reject_raw_call!();
            }
            wrapper.child.as_ref()
        } else {
            if argument.presented_type.as_ref() != Some(&expected) {
                reject_raw_call!();
            }
            argument
        };
        if source_expression.primary.method.as_ref() != Some(caller)
            || source_expression.primary.bci == invoke.bci
        {
            reject_raw_call!();
        }
        match source_expression.shape {
            Shape::Local => {
                let Some(local_name) = source_expression.direct_local_name.as_deref() else {
                    reject_raw_call!();
                };
                let Some((argument_load_bci, (argument_slot, argument_local_read))) =
                    generic_call_load_source(body_values, fact, invoke.bci)
                else {
                    reject_raw_call!();
                };
                let formal_slot = formals
                    .iter()
                    .find(|formal| formal.name == local_name)
                    .map(|formal| formal.slot);
                let local_decl = declarations
                    .iter()
                    .filter(|declaration| declaration.name == local_name)
                    .collect::<Vec<_>>();
                let source_name = local_decl.first().and_then(|declaration| {
                    declaration
                        .source_type_name
                        .as_deref()
                        .or(match &declaration.ty {
                            Type::Reference(name) => Some(name.as_str()),
                            _ => None,
                        })
                });
                let source_type_matches = source_name
                    .zip(source_expression.presented_type.as_ref())
                    .is_some_and(|(name, ty)| ty == &Type::Reference(name.to_owned()));
                if argument_load_bci != source_expression.primary.bci
                    || match formal_slot {
                        Some(slot) => {
                            !generic_call_load_matches_formal(body_values, fact, slot, invoke.bci)
                        }
                        None => {
                            local_decl.len() != 1
                                || local_decl[0].scope_anchors.len() != 0
                                || local_decl[0].bci >= argument_load_bci
                                || !source_type_matches
                                || !generic_call_local_definition_matches_declaration(
                                    body_values,
                                    argument_local_read,
                                    *argument_slot,
                                    local_decl[0].bci,
                                    argument_load_bci,
                                )
                        }
                    }
                {
                    reject_raw_call!();
                }
            }
            _ => {
                let Definition::Instruction { bci, .. } = &fact.definition else {
                    reject_raw_call!();
                };
                if *bci != source_expression.primary.bci
                    || body_values.iter().filter(|site| site.bci == *bci).count() != 1
                    || !body_values
                        .iter()
                        .find(|site| site.bci == *bci)
                        .is_some_and(|site| {
                            site.stack_writes.iter().any(|write| {
                                write.value == fact.value
                                    && write.definition == fact.definition
                                    && write.replaced_by.is_none()
                                    && write.uses.as_slice() == [Some(invoke.bci)]
                            })
                        })
                {
                    reject_raw_call!();
                }
            }
        }
    }
    Ok(true)
}

/// Proves a narrow result-bearing raw caller from a same-run body-return certificate or an
/// already-published abstract no-body declaration: a bounded conditional returned from exact raw
/// arguments, an exact null return, or the no-body method's declared generic contract.
#[allow(clippy::too_many_arguments)]
pub fn prove_raw_same_class_direct_return_incoming(
    invoke: &class_source::SameClassInvokeUse,
    caller: &PhysicalMethodId,
    caller_header: &MemberHeader,
    caller_ast: &jarde_java::report::ClassSourceMethodAst,
    body_values: &[class_source::SameClassBodyValueSite],
    invokes: &[class_source::SameClassInvokeUse],
    method_headers: &[MemberHeader],
    callee: &class_source::SameClassGenericMethodContract,
    callee_no_body_declaration_leaf: bool,
    candidate: Option<&jarde_java::report::GenericReturnCandidate>,
    caller_signature_projected: bool,
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    caller_init_record: Option<&jarde_java::init::InitRecord>,
    budget: &mut Budget,
    mut inventory_matches: impl FnMut(&mut Budget) -> Result<bool>,
    mut caller_contract_from_signature: impl FnMut(
        &mut Budget,
    ) -> Result<
        Option<class_source::SameClassGenericMethodContract>,
    >,
) -> Result<bool> {
    use jarde_java::ast::Type;
    use jarde_java::report::{
        ClassSourceAstExpressionShape as Shape, ClassSourceInvokeResultUseKind as ResultUse,
        GenericReturnValue,
    };
    use jarde_jvm::method_ir::{Definition, parameter_positions};
    use jarde_reader::signature::{SignatureType, TypeParameter};

    macro_rules! reject_raw_result_call {
        () => {{
            return Ok(false);
        }};
    }

    budget.poll()?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            method_headers
                .len()
                .saturating_add(callee.parameters.len())
                .saturating_add(callee.method_parameters.len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    if candidate.is_none() && !callee_no_body_declaration_leaf {
        reject_raw_result_call!();
    }
    if invoke.physical_caller != *caller
        || invoke.owner.as_slice() != class_internal
        || !invoke.ssa_operands_complete
        || callee.method.name.0 != invoke.name
        || callee.method.descriptor.0 != invoke.descriptor
        || invoke.argument_value_facts.len() != callee.parameters.len()
        || invoke.argument_values.len() != callee.parameters.len()
    {
        reject_raw_result_call!();
    }

    // This deliberately accepts only one method-local variable with one ordinary, non-generic
    // class bound. Its physical erasure is the source target type for both the callee result and
    // every use proved below; class variables, interface bounds, and parameterized bounds stay
    // outside this narrow raw-caller proof.
    let Some(TypeParameter {
        name: variable,
        class_bound: Some(bound_signature @ SignatureType::Class(bound)),
        interface_bounds,
    }) = callee.method_parameters.as_slice().first()
    else {
        reject_raw_result_call!();
    };
    let Some(bound_segment) = bound.segments.first() else {
        reject_raw_result_call!();
    };
    budget.poll()?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(bound_segment.binary_name.len()).unwrap_or(u64::MAX),
    )?;
    let bound_is_plain_class = bound.segments.len() == 1 && bound_segment.arguments.is_empty();
    let mut bound_descriptor = Vec::with_capacity(bound_segment.binary_name.len() + 2);
    bound_descriptor.push(b'L');
    bound_descriptor.extend_from_slice(&bound_segment.binary_name);
    bound_descriptor.push(b';');
    if callee.method_parameters.len() != 1
        || !interface_bounds.is_empty()
        || !bound_is_plain_class
        || callee.result.as_ref() != Some(&SignatureType::TypeVariable(variable.clone()))
    {
        reject_raw_result_call!();
    }

    let is_null_return = candidate.is_some_and(|candidate| {
        matches!(&candidate.value, GenericReturnValue::NullLiteral)
            && candidate.parameters.is_empty()
    }) && callee.parameters.is_empty();
    let is_value_return = callee_no_body_declaration_leaf
        || candidate.is_some_and(|candidate| {
            matches!(
                &candidate.value,
                GenericReturnValue::Parameter(_) | GenericReturnValue::Conditional { .. }
            )
        });
    if !is_null_return && !is_value_return {
        reject_raw_result_call!();
    }

    let is_static_target = invoke.opcode == 0xb8;
    if is_static_target {
        if invoke.receiver_value_fact.is_some() || invoke.receiver_value.is_some() {
            reject_raw_result_call!();
        }
    } else {
        if !matches!(invoke.opcode, 0xb6 | 0xb7)
            || invoke.receiver_value_fact.is_none()
            || invoke.receiver_value.is_none()
        {
            reject_raw_result_call!();
        }
    }

    let descriptor = match descriptor_facts(&invoke.descriptor, DescriptorKind::Method) {
        Ok(descriptor) => descriptor,
        Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => return Err(error),
        Err(_) => reject_raw_result_call!(),
    };
    let Some(result_descriptor) = descriptor
        .result()
        .and_then(|result| result.bytes(&invoke.descriptor))
    else {
        reject_raw_result_call!();
    };
    let caller_descriptor =
        match descriptor_facts(&caller_header.descriptor.raw().0, DescriptorKind::Method) {
            Ok(descriptor) => descriptor,
            Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => {
                return Err(error);
            }
            Err(_) => reject_raw_result_call!(),
        };
    let Some(caller_result_descriptor) = caller_descriptor
        .result()
        .and_then(|result| result.bytes(&caller_header.descriptor.raw().0))
    else {
        reject_raw_result_call!();
    };
    if descriptor.parameters().len() != callee.parameters.len()
        || result_descriptor != bound_descriptor.as_slice()
        || class_source::same_class_generic_source_assignable_to_descriptor(
            bound_signature,
            &[],
            caller_result_descriptor,
            &[],
            &[],
            budget,
        )? != Some(true)
    {
        reject_raw_result_call!();
    }
    for (formal, descriptor_component) in callee.parameters.iter().zip(descriptor.parameters()) {
        budget.poll()?;
        let erased = match formal {
            SignatureType::TypeVariable(name) if name == variable => bound_descriptor.as_slice(),
            SignatureType::Base(base) => std::slice::from_ref(base),
            SignatureType::Class(class)
                if class.segments.len() == 1 && class.segments[0].arguments.is_empty() =>
            {
                // The JVM descriptor view is already parsed and validated; this path only admits
                // the non-parameterized class form whose erasure is its binary name.
                let segment = &class.segments[0];
                budget.poll()?;
                budget.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(segment.binary_name.len()).unwrap_or(u64::MAX),
                )?;
                let mut descriptor = Vec::with_capacity(segment.binary_name.len() + 2);
                descriptor.push(b'L');
                descriptor.extend_from_slice(&segment.binary_name);
                descriptor.push(b';');
                let Some(actual) = descriptor_component.bytes(&invoke.descriptor) else {
                    reject_raw_result_call!();
                };
                if actual != descriptor.as_slice() {
                    reject_raw_result_call!();
                }
                continue;
            }
            _ => reject_raw_result_call!(),
        };
        if descriptor_component.bytes(&invoke.descriptor) != Some(erased) {
            reject_raw_result_call!();
        }
    }
    let Some(key) = generic_call_ast_key(invoke) else {
        reject_raw_result_call!();
    };
    let Some(sites) = jarde_java::report::class_source_invoke_ast_sites(
        caller_ast,
        std::slice::from_ref(&key),
        budget,
    )
    .map_err(|stop| {
        enum_projection_stop_error(
            stop,
            "same-class raw result AST proof",
            "generic_call_ast_missing",
        )
    })?
    else {
        reject_raw_result_call!();
    };
    if sites.len() != 1 || !inventory_matches(budget)? {
        reject_raw_result_call!();
    }
    let site = &sites[0];
    if site.caller != *caller
        || site.key != key
        || source_identifier(&invoke.name).as_deref() != Some(site.ast_name.as_str())
        || site.arguments.len() != callee.parameters.len()
    {
        reject_raw_result_call!();
    }

    // The callee census is a closed same-name/arity inventory. This makes the exact raw call a
    // binding proof only when no same-arity sibling can consume it and no synthetic member can
    // obscure the physical target.
    let mut target_headers = 0usize;
    let mut same_arity = 0usize;
    for header in method_headers {
        budget.poll()?;
        if header.name.raw().0 != invoke.name {
            continue;
        }
        if header.access_flags & (0x0080 | 0x0040 | 0x1000) != 0 {
            reject_raw_result_call!();
        }
        let Ok(header_descriptor) =
            descriptor_facts(&header.descriptor.raw().0, DescriptorKind::Method)
        else {
            reject_raw_result_call!();
        };
        if header.name.raw().0 == invoke.name
            && header.descriptor.raw().0 == invoke.descriptor
            && (header.access_flags & 0x0008 != 0) == is_static_target
        {
            target_headers = target_headers.saturating_add(1);
        }
        if header_descriptor.parameters().len() == descriptor.parameters().len() {
            same_arity = same_arity.saturating_add(1);
        }
    }
    if target_headers != 1 || same_arity != 1 {
        reject_raw_result_call!();
    }

    let caller_formals = if is_null_return {
        None
    } else {
        let Some(formals) = jarde_java::report::class_source_method_formal_names(
            caller_ast, budget,
        )
        .map_err(|stop| {
            enum_projection_stop_error(
                stop,
                "same-class raw result caller formals",
                "generic_call_ast_missing",
            )
        })?
        else {
            reject_raw_result_call!();
        };
        Some(formals)
    };
    let caller_slots = if is_null_return {
        None
    } else {
        let Some(slots) =
            parameter_positions(&caller_descriptor, caller_header.access_flags & 0x0008 != 0)
        else {
            reject_raw_result_call!();
        };
        Some(slots)
    };

    if is_static_target {
        if site.receiver.is_some()
            || invoke.receiver_value_fact.is_some()
            || invoke.receiver_value.is_some()
        {
            reject_raw_result_call!();
        }
    } else {
        let Some(receiver_fact) = invoke.receiver_value_fact.as_ref() else {
            reject_raw_result_call!();
        };
        if invoke.receiver_value.is_none() {
            reject_raw_result_call!();
        }
        let mut direct_raw_formal_receiver = false;
        if callee_no_body_declaration_leaf
            && site
                .receiver
                .as_ref()
                .is_some_and(|receiver| receiver.direct_local_name.as_deref() != Some("this"))
        {
            if let Some(receiver) = site.receiver.as_ref() {
                if receiver.primary.method.as_ref() != Some(caller)
                    || !matches!(receiver.shape, Shape::Local)
                {
                    reject_raw_result_call!();
                }
                let Some(name) = receiver.direct_local_name.as_deref() else {
                    reject_raw_result_call!();
                };
                let (Some(formals), Some(slots)) = (caller_formals.as_ref(), caller_slots.as_ref())
                else {
                    reject_raw_result_call!();
                };
                let mut formal = None;
                for candidate in formals {
                    budget.poll()?;
                    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                    if candidate.name == name {
                        if formal.is_some() {
                            reject_raw_result_call!();
                        }
                        formal = Some(candidate);
                    }
                }
                let Some(formal) = formal else {
                    reject_raw_result_call!();
                };
                let mut formal_position = None;
                for (position, slot) in slots.iter().enumerate() {
                    budget.poll()?;
                    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                    if *slot == formal.slot {
                        if formal_position.is_some() {
                            reject_raw_result_call!();
                        }
                        formal_position = Some(position);
                    }
                }
                let Some(formal_position) = formal_position else {
                    reject_raw_result_call!();
                };
                let Some(parameter) = caller_descriptor.parameters().get(formal_position) else {
                    reject_raw_result_call!();
                };
                let Some(parameter_bytes) = parameter.bytes(&caller_header.descriptor.raw().0)
                else {
                    reject_raw_result_call!();
                };
                let mut own_class_descriptor = Vec::with_capacity(class_internal.len() + 2);
                own_class_descriptor.push(b'L');
                own_class_descriptor.extend_from_slice(class_internal);
                own_class_descriptor.push(b';');
                if parameter_bytes != own_class_descriptor.as_slice() {
                    reject_raw_result_call!();
                }
                if caller_signature_projected {
                    let Some(caller_contract) = caller_contract_from_signature(budget)? else {
                        reject_raw_result_call!();
                    };
                    if !matches!(
                        caller_contract.parameters.get(formal_position),
                        Some(SignatureType::Class(class))
                            if class.segments.len() == 1
                                && class.segments[0].binary_name == class_internal
                                && class.segments[0].arguments.is_empty()
                    ) {
                        reject_raw_result_call!();
                    }
                }
                budget.poll()?;
                budget.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(
                        body_values
                            .len()
                            .saturating_mul(2)
                            .saturating_add(invokes.len()),
                    )
                    .unwrap_or(u64::MAX),
                )?;
                let Some((receiver_load_bci, (receiver_slot, receiver_read))) =
                    generic_call_load_source(body_values, receiver_fact, invoke.bci)
                else {
                    reject_raw_result_call!();
                };
                if receiver_load_bci != receiver.primary.bci
                    || *receiver_slot != formal.slot
                    || receiver_read.replaced_by.is_some()
                    || !generic_call_load_matches_formal(
                        body_values,
                        receiver_fact,
                        formal.slot,
                        invoke.bci,
                    )
                {
                    reject_raw_result_call!();
                }
                direct_raw_formal_receiver = true;
            }
        }
        if !direct_raw_formal_receiver {
            if let Some(receiver) = site.receiver.as_ref() {
                if receiver.primary.method.as_ref() != Some(caller)
                    || receiver.direct_local_name.as_deref() != Some("this")
                    || !matches!(receiver.shape, Shape::Local)
                {
                    reject_raw_result_call!();
                }
                budget.poll()?;
                budget.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(
                        body_values
                            .len()
                            .saturating_mul(2)
                            .saturating_add(invokes.len()),
                    )
                    .unwrap_or(u64::MAX),
                )?;
                let Some((receiver_load_bci, _)) =
                    generic_call_load_source(body_values, receiver_fact, invoke.bci)
                else {
                    reject_raw_result_call!();
                };
                if receiver_load_bci != receiver.primary.bci {
                    reject_raw_result_call!();
                }
            } else {
                budget.poll()?;
                budget.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(body_values.len().saturating_add(invokes.len()))
                        .unwrap_or(u64::MAX),
                )?;
            }
            if !generic_call_receiver_is_initialized_this(
                invoke,
                receiver_fact,
                caller_header,
                class_internal,
                class_superclass,
                caller_init_record,
                body_values,
                invokes,
            ) {
                reject_raw_result_call!();
            }
        }
    }
    if !is_null_return {
        let (Some(formal_names), Some(caller_slots)) =
            (caller_formals.as_ref(), caller_slots.as_ref())
        else {
            reject_raw_result_call!();
        };
        budget.poll()?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(
                formal_names
                    .len()
                    .saturating_add(caller_slots.len())
                    .saturating_mul(site.arguments.len()),
            )
            .unwrap_or(u64::MAX),
        )?;
        for (position, (argument, fact)) in site
            .arguments
            .iter()
            .zip(&invoke.argument_value_facts)
            .enumerate()
        {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if fact.replaced_by.is_some() || fact.uses.as_slice() != [Some(invoke.bci)] {
                reject_raw_result_call!();
            }
            let Some(parameter) = descriptor.parameters().get(position) else {
                reject_raw_result_call!();
            };
            let Some(raw_type) = parameter.bytes(&invoke.descriptor) else {
                reject_raw_result_call!();
            };
            let expected = match raw_type {
                b"Z" => Type::Boolean,
                b"B" => Type::Byte,
                b"C" => Type::Char,
                b"S" => Type::Short,
                b"I" => Type::Int,
                b"J" => Type::Long,
                b"F" => Type::Float,
                b"D" => Type::Double,
                _ => generic_call_reference_type_from_descriptor(raw_type)
                    .unwrap_or_else(|| Type::Reference(String::new())),
            };
            if matches!(&expected, Type::Reference(name) if name.is_empty()) {
                reject_raw_result_call!();
            }
            let source = if let Some(wrapper) = &argument.presentation_wrapper {
                if wrapper.ty != expected {
                    reject_raw_result_call!();
                }
                wrapper.child.as_ref()
            } else {
                if argument.presented_type.as_ref() != Some(&expected) {
                    reject_raw_result_call!();
                }
                argument
            };
            if source.primary.method.as_ref() != Some(caller) || source.primary.bci == invoke.bci {
                reject_raw_result_call!();
            }
            let Definition::Instruction { bci, .. } = &fact.definition else {
                reject_raw_result_call!();
            };
            match source.shape {
                Shape::Local => {
                    let Some(name) = source.direct_local_name.as_deref() else {
                        reject_raw_result_call!();
                    };
                    let mut formal = None;
                    for candidate in formal_names {
                        budget.poll()?;
                        if candidate.name == name {
                            if formal.is_some() {
                                reject_raw_result_call!();
                            }
                            formal = Some(candidate);
                        }
                    }
                    let Some(formal) = formal else {
                        reject_raw_result_call!();
                    };
                    let mut caller_position = None;
                    for (position, slot) in caller_slots.iter().enumerate() {
                        budget.poll()?;
                        if *slot == formal.slot {
                            caller_position = Some(position);
                            break;
                        }
                    }
                    let Some(caller_position) = caller_position else {
                        reject_raw_result_call!();
                    };
                    let Some(caller_parameter) =
                        caller_descriptor.parameters().get(caller_position)
                    else {
                        reject_raw_result_call!();
                    };
                    if caller_parameter.bytes(&caller_header.descriptor.raw().0) != Some(raw_type) {
                        reject_raw_result_call!();
                    }
                    budget.poll()?;
                    budget.charge(
                        CountedBudgetDimension::AnalysisSteps,
                        u64::try_from(body_values.len().saturating_mul(2)).unwrap_or(u64::MAX),
                    )?;
                    let Some((load_bci, (slot, read))) =
                        generic_call_load_source(body_values, fact, invoke.bci)
                    else {
                        reject_raw_result_call!();
                    };
                    if load_bci != source.primary.bci
                        || *slot != formal.slot
                        || read.replaced_by.is_some()
                        || !generic_call_load_matches_formal(
                            body_values,
                            fact,
                            formal.slot,
                            invoke.bci,
                        )
                    {
                        reject_raw_result_call!();
                    }
                }
                Shape::Literal if argument.presentation_wrapper.is_none() => {
                    if !matches!(
                        raw_type,
                        b"Z" | b"B" | b"C" | b"S" | b"I" | b"J" | b"F" | b"D"
                    ) {
                        reject_raw_result_call!();
                    }
                }
                Shape::Call { .. } if argument.presentation_wrapper.is_some() => {}
                Shape::Null if argument.presentation_wrapper.is_some() => {
                    budget.poll()?;
                    budget.charge(
                        CountedBudgetDimension::AnalysisSteps,
                        u64::try_from(body_values.len()).unwrap_or(u64::MAX),
                    )?;
                    if !generic_call_null_matches(body_values, fact, source, invoke.bci) {
                        reject_raw_result_call!();
                    }
                }
                _ => reject_raw_result_call!(),
            }
            if *bci != source.primary.bci {
                reject_raw_result_call!();
            }
            let mut matching_sites = 0usize;
            for body_site in body_values {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                if body_site.bci != *bci {
                    continue;
                }
                let mut matched_write = false;
                for write in &body_site.stack_writes {
                    budget.poll()?;
                    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                    if write.value == fact.value
                        && write.definition == fact.definition
                        && write.replaced_by.is_none()
                        && write.uses.as_slice() == [Some(invoke.bci)]
                    {
                        matched_write = true;
                        break;
                    }
                }
                if matched_write {
                    matching_sites = matching_sites.saturating_add(1);
                }
            }
            if matching_sites != 1 {
                reject_raw_result_call!();
            }
        }
    }

    let Some(result_fact) = invoke.result_value_fact.as_ref() else {
        reject_raw_result_call!();
    };
    if result_fact.replaced_by.is_some()
        || !matches!(result_fact.definition, Definition::Instruction { bci, .. } if bci == invoke.bci)
    {
        reject_raw_result_call!();
    }
    let Some(result_uses) =
        jarde_java::report::class_source_invoke_result_uses(caller_ast, &key, &[], budget)
            .map_err(|stop| {
                enum_projection_stop_error(
                    stop,
                    "same-class raw result consumer proof",
                    "generic_call_ast_missing",
                )
            })?
    else {
        reject_raw_result_call!();
    };
    if result_uses.len() != 1 {
        reject_raw_result_call!();
    }
    let ResultUse::DirectReturn { consumer_bci } = &result_uses[0].kind else {
        reject_raw_result_call!();
    };
    if result_fact.uses.as_slice() != [Some(*consumer_bci)] {
        reject_raw_result_call!();
    }
    budget.poll()?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(body_values.len()).unwrap_or(u64::MAX),
    )?;
    let mut return_site = None;
    for body_site in body_values {
        budget.poll()?;
        if body_site.bci == *consumer_bci {
            if return_site.is_some() {
                reject_raw_result_call!();
            }
            return_site = Some(body_site);
        }
    }
    let Some(return_site) = return_site else {
        reject_raw_result_call!();
    };
    if return_site.opcode != 0xb0
        || return_site.stack_reads.len() != 1
        || return_site.stack_reads[0].1.value != result_fact.value
        || return_site.stack_reads[0].1.replaced_by.is_some()
        || return_site.stack_reads[0].1.uses.as_slice() != [Some(*consumer_bci)]
    {
        reject_raw_result_call!();
    }
    Ok(true)
}

pub fn charge_signature_type_proof(ty: &SignatureType, budget: &mut Budget) -> Result<u64> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    match ty {
        SignatureType::Base(_) => Ok(0),
        SignatureType::TypeVariable(_) => Ok(1),
        SignatureType::Array(element) => charge_signature_type_proof(element, budget),
        SignatureType::Class(class) => {
            let mut variable_count = 0_u64;
            for segment in &class.segments {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                for argument in &segment.arguments {
                    budget.poll()?;
                    match argument {
                        TypeArgument::Any => {
                            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                        }
                        TypeArgument::Exact(ty)
                        | TypeArgument::Extends(ty)
                        | TypeArgument::Super(ty) => {
                            variable_count = variable_count
                                .saturating_add(charge_signature_type_proof(ty, budget)?);
                        }
                    }
                }
            }
            Ok(variable_count)
        }
    }
}

pub fn generic_call_signature_needs_transaction(
    member_name: &[u8],
    parsed: &jarde_reader::signature::MethodSignature,
    budget: &mut Budget,
) -> Result<bool> {
    if member_name == b"<init>" && !parsed.type_parameters.is_empty() {
        return Ok(false);
    }
    if !parsed.type_parameters.is_empty() {
        return Ok(true);
    }
    for ty in parsed
        .parameters
        .iter()
        .chain(parsed.result.iter())
        .chain(parsed.throws.iter())
    {
        if charge_signature_type_proof(ty, budget)? != 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn generic_list_parameter_types(
    parsed: &jarde_reader::signature::MethodSignature,
    slots: &[u16],
    budget: &mut Budget,
) -> Result<Option<Vec<(u16, jarde_java::ast::Type)>>> {
    use jarde_reader::signature::{SignatureType, TypeArgument};

    if slots.len() != parsed.parameters.len() {
        return Ok(None);
    }
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(parsed.parameters.len()).unwrap_or(u64::MAX),
    )?;
    let mut selected = Vec::new();
    for (parameter, slot) in parsed.parameters.iter().zip(slots.iter().copied()) {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let SignatureType::Class(class) = parameter else {
            continue;
        };
        let [list] = class.segments.as_slice() else {
            continue;
        };
        if list.binary_name.as_slice() != b"java/util/List" {
            continue;
        }
        let [TypeArgument::Exact(SignatureType::Class(element))] = list.arguments.as_slice() else {
            continue;
        };
        if element.segments.is_empty()
            || element
                .segments
                .iter()
                .any(|segment| !segment.arguments.is_empty())
        {
            continue;
        }
        let Some(binary_name) = element.segments.last().map(|segment| &segment.binary_name) else {
            continue;
        };
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(binary_name.len()).unwrap_or(u64::MAX),
        )?;
        let Some(source_name) = crate::names::source_class_name(&JvmBytes(binary_name.clone()))
        else {
            continue;
        };
        budget.charge(
            CountedBudgetDimension::OutputBytes,
            u64::try_from(source_name.len()).unwrap_or(u64::MAX),
        )?;
        selected.push((slot, jarde_java::ast::Type::Reference(source_name)));
    }
    Ok(Some(selected))
}

pub fn generic_call_ast_inventory_matches(
    ast: &jarde_java::report::ClassSourceMethodAst,
    caller: &PhysicalMethodId,
    invokes: &[class_source::SameClassInvokeUse],
    class_internal: &[u8],
    budget: &mut Budget,
    mut apply_test_fault: impl FnMut(
        &[u8],
        &PhysicalMethodId,
        &mut Vec<jarde_java::report::ClassSourceInvokeKey>,
    ),
) -> Result<bool> {
    budget.poll()?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(invokes.len()).unwrap_or(u64::MAX),
    )?;
    let mut expected_count = 0u64;
    let mut expected_bytes = 0u64;
    for invoke in invokes {
        budget.poll()?;
        if invoke.physical_caller != *caller || invoke.owner.as_slice() != class_internal {
            continue;
        }
        if !matches!(invoke.opcode, 0xb6 | 0xb7 | 0xb8 | 0xb9) {
            return Ok(false);
        }
        expected_count = expected_count.saturating_add(1);
        expected_bytes = expected_bytes
            .saturating_add(u64::try_from(invoke.owner.len()).unwrap_or(u64::MAX))
            .saturating_add(u64::try_from(invoke.name.len()).unwrap_or(u64::MAX))
            .saturating_add(u64::try_from(invoke.descriptor.len()).unwrap_or(u64::MAX));
    }
    budget.charge(CountedBudgetDimension::IrItems, expected_count)?;
    budget.charge(CountedBudgetDimension::OutputBytes, expected_bytes)?;
    let Ok(expected_capacity) = usize::try_from(expected_count) else {
        return Ok(false);
    };
    let mut expected = Vec::with_capacity(expected_capacity);
    for invoke in invokes {
        budget.poll()?;
        if invoke.physical_caller != *caller || invoke.owner.as_slice() != class_internal {
            continue;
        }
        let Some(key) = generic_call_ast_key(invoke) else {
            return Ok(false);
        };
        expected.push(key);
    }
    apply_test_fault(class_internal, caller, &mut expected);
    let inventory = jarde_java::report::class_source_same_class_invoke_inventory_matches(
        ast,
        class_internal,
        &expected,
        budget,
    )
    .map_err(|stop| {
        enum_projection_stop_error(
            stop,
            "same-class generic caller invoke inventory",
            "generic_call_ast_missing",
        )
    })?;
    Ok(inventory.is_some())
}

pub fn same_class_overload_candidates<'facts>(
    method_headers: &[MemberHeader],
    name: &[u8],
    class_internal: &[u8],
    budget: &mut Budget,
    mut declaration_facts: impl FnMut(usize) -> Option<(&'facts PhysicalMethodId, bool, bool)>,
    mut method_contract: impl FnMut(
        usize,
        &mut Budget,
    ) -> Result<Option<class_source::SameClassGenericMethodContract>>,
) -> Result<Option<Vec<class_source::SameClassOverloadCandidate>>> {
    use jarde_reader::signature::parse_method_signature;
    macro_rules! reject_candidate_none {
        () => {{
            return Ok(None);
        }};
    }

    let mut candidates = Vec::new();
    for (index, header) in method_headers.iter().enumerate() {
        budget.poll()?;
        if header.name.raw().0 != name {
            continue;
        }
        let Some((method, generic_signature_projected, declaration_present)) =
            declaration_facts(index)
        else {
            reject_candidate_none!();
        };
        let signature_shells = header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"Signature")
            .cloned()
            .collect::<Vec<_>>();
        if generic_signature_projected && signature_shells.len() != 1 {
            reject_candidate_none!();
        }
        let (parameters, method_parameters, source_contract_valid) = if !generic_signature_projected
        {
            // Overload resolution sees the declaration this report will actually emit. If a
            // generic Signature was refused, that declaration is the erased descriptor form;
            // using the unpublished Signature here would make the candidate set disagree with
            // the source text and could either reject a safe target or prove the wrong overload.
            let parsed = match parse_method_signature(&header.descriptor.raw().0, budget) {
                Ok(parsed) => parsed,
                Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => {
                    return Err(error);
                }
                Err(_) => reject_candidate_none!(),
            };
            (parsed.parameters, Vec::new(), declaration_present)
        } else {
            let contract = match method_contract(index, budget) {
                Ok(Some(contract)) => contract,
                Ok(None) => reject_candidate_none!(),
                Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => {
                    return Err(error);
                }
                Err(_) => reject_candidate_none!(),
            };
            if !declaration_present {
                reject_candidate_none!();
            }
            (contract.parameters, contract.method_parameters, true)
        };
        candidates.push(class_source::SameClassOverloadCandidate {
            method: method.clone(),
            owner: class_internal.to_vec(),
            descriptor: header.descriptor.raw().0.clone(),
            access_flags: header.access_flags,
            parameters,
            method_parameters,
            source_writable: source_contract_valid,
        });
    }
    if candidates.is_empty() {
        Ok(None)
    } else {
        Ok(Some(candidates))
    }
}

#[cfg(test)]
mod signature_source_name_tests {
    use super::*;
    use jarde_reader::budget::{BudgetDimension, Limits};

    fn budget(steps: u64) -> Budget {
        Budget::new(Limits {
            analysis_steps: steps,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        })
    }

    #[test]
    fn signature_spelling_shares_raw_scope_and_stop_boundaries() {
        let raw = JvmBytes::from_unicode("𐐀T").0;
        let scope = [TypeParameterErasure {
            name: raw.clone(),
            descriptor: b"Ljava/lang/Object;".to_vec(),
        }];
        let ty = SignatureType::Array(Box::new(SignatureType::TypeVariable(raw.clone())));
        assert_eq!(
            spell_ordinary_signature_type(&ty, &scope, &mut budget(100), 0).unwrap(),
            "𐐀T[]"
        );
        assert_eq!(scope[0].name, raw);
        let underscore_scope = [TypeParameterErasure {
            name: b"_".to_vec(),
            descriptor: b"Ljava/lang/Object;".to_vec(),
        }];
        let underscore = SignatureType::TypeVariable(b"_".to_vec());
        assert_eq!(
            spell_ordinary_signature_type(&underscore, &underscore_scope, &mut budget(100), 0)
                .unwrap(),
            "_"
        );
        assert!(spell_ordinary_signature_type(&underscore, &scope, &mut budget(100), 0).is_err());
        assert_eq!(simple_generic_class_name(b"_/Bound").unwrap(), "_.Bound");
        assert_eq!(
            simple_generic_class_name(&JvmBytes::from_unicode("p/𐐀Bound").0).unwrap(),
            "p.𐐀Bound"
        );
        assert!(simple_generic_class_name(b"p/Outer$Inner").is_err());
        assert_eq!(
            binary_pool_class_name(b"p/Outer$Inner").unwrap(),
            "p.Outer$Inner"
        );
        let bad = vec![0xed, 0xa0, 0x80];
        assert!(
            spell_ordinary_signature_type(
                &SignatureType::TypeVariable(bad.clone()),
                &[TypeParameterErasure {
                    name: bad,
                    descriptor: b"Ljava/lang/Object;".to_vec()
                }],
                &mut budget(100),
                0,
            )
            .is_err()
        );
        assert!(matches!(
            spell_ordinary_signature_type(&ty, &scope, &mut budget(0), 0),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
                ..
            })
        ));
        let mut cancelled = budget(100);
        cancelled.cancellation_token().cancel();
        assert!(matches!(
            spell_ordinary_signature_type(&ty, &scope, &mut cancelled, 0),
            Err(Error::Cancelled { .. })
        ));
    }
}

// Generic method declaration policy over the caller's already selected facts.

/// Descriptor spelling and JVM slot positions consumed by generic declaration policy.
/// Keep the complete descriptor spellability check, including its physical result.
fn generic_declaration_parameters(
    descriptor: &[u8],
    is_static: bool,
) -> Option<Vec<(String, u16)>> {
    Some(crate::declaration::method_descriptor(descriptor, is_static, false)?.parameters)
}

pub fn generic_void_body_declaration(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    declaration_available: bool,
    has_annotations: bool,
    has_source_refusals: bool,
    throws: &[String],
    has_physical_throws: bool,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    erasures: &[TypeParameterErasure],
    class_internal: &[u8],
    resolved_inner_classes: &[jarde_reader::classfile::ResolvedInnerClass],
    budget: &mut Budget,
) -> Option<Result<String>> {
    // Selection and strict eligibility have one owner. NotApplicable lets the facade
    // continue its existing ordered candidates without creating a source refusal.
    if parsed.type_parameters.is_empty()
        || parsed.result.is_some()
        || !parsed.throws.is_empty()
        || has_physical_throws
        || !matches!(candidate, Some(GenericReturnCandidate {
            parameters,
            value: GenericReturnValue::CompleteBody { result_type: None, .. },
            ..
        }) if !parameters.is_empty())
    {
        return None;
    }
    Some((|| {
        let refused = |why| Error::unsupported("generic_source_shape_unproved", why);
        let [parameter] = parsed.type_parameters.as_slice() else {
            return Err(refused(
                "generic void body requires exactly one method type parameter",
            ));
        };
        let Some(SignatureType::Class(bound)) = parameter.class_bound.as_ref() else {
            return Err(refused("generic void body requires one class bound"));
        };
        let Some(physical) = generic_declaration_parameters(descriptor, false) else {
            return Err(refused(
                "physical generic void descriptor cannot be spelled",
            ));
        };
        if !parameter.interface_bounds.is_empty()
            || bound.segments.len() != 1
            || !bound.segments[0].arguments.is_empty()
            || erasures.len() != 1
            || erasures[0].name != parameter.name
            || !erasures[0].descriptor.starts_with(b"L")
            || parsed.parameters.as_slice()
                != [
                    SignatureType::TypeVariable(parameter.name.clone()),
                    SignatureType::Base(b'Z'),
                ]
            || physical.len() != 2
            || physical[1].0 != "boolean"
            || physical[0].0 == "boolean"
            || has_physical_throws
            || !matches!(candidate, Some(GenericReturnCandidate {
            parameters,
            value: GenericReturnValue::CompleteBody { result_type: None, .. },
                ..
        }) if parameters.len() == 2)
        {
            return Err(refused(
                "generic void body must be `<T extends B> void` with physical `(B, boolean)`, exact erasure, and two same-run parameter slots",
            ));
        }
        let bound_name = if bound.segments[0].binary_name.contains(&b'$') {
            let expected_erasure = [
                b"L".as_slice(),
                bound.segments[0].binary_name.as_slice(),
                b";",
            ]
            .concat();
            if erasures[0].descriptor != expected_erasure {
                return Err(refused(
                    "nested bound does not match the proved physical parameter erasure",
                ));
            }
            selected_nested_sibling_bound_spelling(
                class_internal,
                &bound.segments[0].binary_name,
                &physical[0].0,
                resolved_inner_classes,
            )?
        } else {
            simple_generic_class_name(&bound.segments[0].binary_name)?
        };
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        generic_method_declaration(
            raw_name,
            descriptor,
            access_flags,
            declaration_available,
            has_annotations,
            has_source_refusals,
            throws,
            parsed,
            candidate,
            false,
            Some(&bound_name),
        )
    })())
}

/// A `$` class name is source-spellable here only because the already selected physical method
/// descriptor supplied that exact spelling and the selected class plus the bound prove direct
/// siblings in one named source family. This deliberately does not canonicalize `$` to `.`.
fn selected_nested_sibling_bound_spelling(
    class_internal: &[u8],
    bound_internal: &[u8],
    physical_parameter_spelling: &str,
    resolved_inner_classes: &[jarde_reader::classfile::ResolvedInnerClass],
) -> Result<String> {
    let refused = || {
        Error::unsupported(
            "generic_source_shape_unproved",
            "nested bound lacks an unambiguous spelling in the selected class family",
        )
    };
    let unique = |internal: &[u8]| {
        let mut matches = resolved_inner_classes
            .iter()
            .filter(|entry| entry.class == internal);
        let entry = matches.next()?;
        matches.next().is_none().then_some(entry)
    };
    let class = unique(class_internal).ok_or_else(refused)?;
    let bound = unique(bound_internal).ok_or_else(refused)?;
    let (Some(outer), Some(class_name), Some(bound_name)) = (
        class.outer_class.as_deref(),
        class.inner_name.as_ref().map(|name| name.0.as_slice()),
        bound.inner_name.as_ref().map(|name| name.0.as_slice()),
    ) else {
        return Err(refused());
    };
    let class_binary = [outer, b"$", class_name].concat();
    let bound_binary = [outer, b"$", bound_name].concat();
    let outer_source = crate::names::source_class_name(&JvmBytes(outer.to_vec()))
        .filter(|name| name.split('.').all(is_java_identifier))
        .ok_or_else(refused)?;
    source_identifier(class_name).ok_or_else(refused)?;
    let inner_source = source_identifier(bound_name).ok_or_else(refused)?;
    let expected_source_name = format!("{outer_source}${inner_source}");
    if class_binary != class_internal
        || bound_binary != bound_internal
        || outer != bound.outer_class.as_deref().unwrap_or_default()
        || physical_parameter_spelling != expected_source_name
    {
        return Err(refused());
    }
    Ok(physical_parameter_spelling.to_owned())
}

pub fn generic_method_declaration(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    declaration_available: bool,
    has_annotations: bool,
    has_source_refusals: bool,
    throws: &[String],
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    method_local_throws_proved: bool,
    first_bound_spelling: Option<&str>,
) -> Result<String> {
    let refused = |why| Error::unsupported("generic_source_shape_unproved", why);
    let candidate = candidate
        .ok_or_else(|| refused("the recovered AST/SSA body is not a direct parameter return"))?;
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_PROTECTED: u16 = 0x0004;
    const ACC_FINAL: u16 = 0x0010;
    const ACC_SYNCHRONIZED: u16 = 0x0020;
    const ACC_STRICT: u16 = 0x0800;
    let void_body = matches!(
        &candidate.value,
        GenericReturnValue::CompleteBody {
            result_type: None,
            ..
        }
    );
    let null_instance_return = matches!(candidate.value, GenericReturnValue::NullLiteral);
    let is_static = access_flags & ACC_STATIC != 0;
    if !declaration_available
        || (void_body && is_static)
        || (!void_body && is_static == null_instance_return)
        || access_flags
            & !(ACC_PUBLIC
                | ACC_PRIVATE
                | ACC_PROTECTED
                | ACC_STATIC
                | ACC_FINAL
                | ACC_SYNCHRONIZED
                | ACC_STRICT)
            != 0
        || has_annotations
        || has_source_refusals
    {
        return Err(refused(
            "method flags, annotations, or existing source refusals cannot be preserved in this generic shape",
        ));
    }
    if parsed.type_parameters.is_empty() {
        return Err(refused("method has no local type parameters"));
    }
    let mut variables = Vec::new();
    for parameter in &parsed.type_parameters {
        let mut bounds = Vec::new();
        for bound in parameter
            .class_bound
            .iter()
            .chain(parameter.interface_bounds.iter())
        {
            let SignatureType::Class(bound) = bound else {
                return Err(refused("type-variable bound is unsupported"));
            };
            let [bound] = bound.segments.as_slice() else {
                return Err(refused("nested bounds are unsupported"));
            };
            if !bound.arguments.is_empty() {
                return Err(refused("parameterized bounds are unsupported"));
            }
            bounds.push(if first_bound_spelling.is_some() && variables.is_empty() {
                first_bound_spelling.expect("checked above").to_owned()
            } else {
                simple_generic_class_name(&bound.binary_name)?
            });
        }
        let variable = source_type_parameter_name(&parameter.name)
            .ok_or_else(|| refused("type variable name has no faithful MUTF-8 Java spelling"))?;

        if bounds.is_empty() {
            bounds.push("java.lang.Object".to_owned());
        }
        variables.push((
            parameter.name.clone(),
            variable.to_owned(),
            bounds.join(" & "),
        ));
    }
    let result_name = if void_body {
        if parsed.result.is_some() {
            return Err(refused("proved void body has a non-void Signature result"));
        }
        "void"
    } else {
        let Some(SignatureType::TypeVariable(result_variable)) = &parsed.result else {
            return Err(refused("result is outside the supported source shape"));
        };
        let Some((_, result_name, _)) = variables
            .iter()
            .find(|(name, _, _)| name == result_variable)
        else {
            return Err(refused("result variable is outside the method scope"));
        };
        result_name.as_str()
    };
    if parsed.parameters.len() != candidate.parameters.len() {
        return Err(refused("parameter count differs from the same-run body"));
    }
    for exception in &parsed.throws {
        let physical_class_throws = matches!(
            exception,
            SignatureType::Class(class)
                if class.segments.len() == 1
                    && class.segments[0].arguments.is_empty()
                    && simple_generic_class_name(&class.segments[0].binary_name).is_ok()
        );
        let proved_local_throws = method_local_throws_proved
            && matches!(exception, SignatureType::TypeVariable(variable)
                if parsed.type_parameters.iter().any(|parameter| parameter.name == *variable));
        if !physical_class_throws && !proved_local_throws {
            return Err(refused(
                "generic throws type cannot be preserved by the physical Exceptions clause",
            ));
        }
    }
    let signature = generic_declaration_parameters(descriptor, is_static)
        .ok_or_else(|| refused("physical descriptor cannot be spelled"))?;
    if signature.len() != candidate.parameters.len() {
        return Err(refused("parameter slot mapping differs from descriptor"));
    }
    let mut arguments = Vec::new();
    let mut result_slots = Vec::new();
    let mut bool_slots = Vec::new();
    for (position, ((ty, slot), (candidate_slot, name))) in
        signature.iter().zip(&candidate.parameters).enumerate()
    {
        if slot != candidate_slot || !is_java_identifier(name) {
            return Err(refused("same-run parameter name or slot is unproved"));
        }
        let spelling = match &parsed.parameters[position] {
            SignatureType::TypeVariable(name) => {
                let Some((_, spelling, _)) =
                    variables.iter().find(|(variable, _, _)| variable == name)
                else {
                    return Err(refused("parameter variable is outside the method scope"));
                };
                if parsed
                    .result
                    .as_ref()
                    .is_some_and(|result| matches!(result, SignatureType::TypeVariable(result) if result == name))
                {
                    result_slots.push(*slot);
                }
                spelling.clone()
            }
            SignatureType::Base(b'Z') if ty == "boolean" => {
                bool_slots.push(*slot);
                "boolean".to_owned()
            }
            SignatureType::Base(_) => ty.clone(),
            SignatureType::Class(class) => {
                let [segment] = class.segments.as_slice() else {
                    return Err(refused("nested parameter type is unsupported"));
                };
                if !segment.arguments.is_empty()
                    || simple_generic_class_name(&segment.binary_name).is_err()
                {
                    return Err(refused(
                        "parameterized or unspellable parameter type is unsupported",
                    ));
                }
                ty.clone()
            }
            _ => {
                return Err(refused(
                    "parameter uses an unsupported generic source shape",
                ));
            }
        };
        arguments.push(format!("{spelling} {name}"));
    }
    let return_proved = match candidate.value {
        GenericReturnValue::EmptyVoid => false,
        GenericReturnValue::CompleteBody { .. } => void_body && parsed.result.is_none(),
        GenericReturnValue::NullLiteral => null_instance_return && parsed.parameters.is_empty(),
        GenericReturnValue::Parameter(slot) => result_slots.contains(&slot),
        GenericReturnValue::Conditional {
            test,
            when_true,
            when_false,
        } => {
            bool_slots.contains(&test)
                && result_slots.contains(&when_true)
                && result_slots.contains(&when_false)
        }
        GenericReturnValue::MemberCreation { .. } => false,
        GenericReturnValue::StaticMemberCreation { .. } => false,
        GenericReturnValue::ArrayCreation { .. } => false,
        GenericReturnValue::TypedFunctional { .. } => false,
    };
    if !return_proved {
        return Err(refused(
            "return value does not come solely from parameters of the result type variable",
        ));
    }
    let name = source_identifier(raw_name)
        .ok_or_else(|| refused("method name has no faithful source spelling"))?;
    let throws = if method_local_throws_proved {
        parsed
            .throws
            .iter()
            .map(|throws| match throws {
                SignatureType::TypeVariable(variable) => variables
                    .iter()
                    .find(|(name, _, _)| name == variable)
                    .map(|(_, name, _)| name.clone())
                    .ok_or_else(|| refused("throws variable has no emitted method type parameter")),
                _ => Err(refused(
                    "strict source gate admits one local throws variable",
                )),
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        throws.to_vec()
    };
    let parameters = variables
        .iter()
        .map(|(_, name, bound)| format!("{name} extends {bound}"))
        .collect::<Vec<_>>();
    Ok(format_generic_method_header(
        access_flags,
        &parameters,
        result_name,
        &name,
        &arguments,
        &throws,
    ))
}

#[cfg(test)]
mod generic_declaration_boundary_tests {
    use super::*;
    use jarde_reader::budget::Limits;
    use jarde_reader::classfile::ResolvedInnerClass;

    fn budget() -> Budget {
        Budget::new(Limits {
            elapsed_millis: u64::MAX,
            analysis_steps: u64::MAX,
            ..Limits::default()
        })
    }

    fn sibling_rows() -> Vec<ResolvedInnerClass> {
        [b"Setter".as_slice(), b"Bound".as_slice()]
            .into_iter()
            .map(|name| ResolvedInnerClass {
                class: [b"probe/Family$".as_slice(), name].concat(),
                outer_class: Some(b"probe/Family".to_vec()),
                inner_name: Some(JvmBytes(name.to_vec())),
                access_flags: 0x0009,
            })
            .collect()
    }

    #[test]
    fn nested_bound_requires_unique_raw_siblings_and_physical_spelling() {
        let rows = sibling_rows();
        let spell = |rows: &[ResolvedInnerClass], physical: &str| {
            selected_nested_sibling_bound_spelling(
                b"probe/Family$Setter",
                b"probe/Family$Bound",
                physical,
                rows,
            )
        };
        assert_eq!(
            spell(&rows, "probe.Family$Bound").unwrap(),
            "probe.Family$Bound"
        );
        assert!(
            spell(&rows, "probe.Family.Bound").is_err(),
            "a dot path is not the selected binary spelling"
        );
        assert!(spell(&rows[..1], "probe.Family$Bound").is_err());
        for mutation in 0..7 {
            let mut changed = rows.clone();
            match mutation {
                0 => changed.push(rows[0].clone()),
                1 => changed.push(rows[1].clone()),
                2 => changed[1].outer_class = Some(b"probe/Other".to_vec()),
                3 => changed[0].outer_class = None,
                4 => changed[1].inner_name = None,
                5 => changed[1].inner_name = Some(JvmBytes(b"bad-name".to_vec())),
                6 => changed[1].class = b"probe/Family$Other".to_vec(),
                _ => unreachable!(),
            }
            assert!(
                spell(&changed, "probe.Family$Bound").is_err(),
                "mutation {mutation}"
            );
        }
    }

    #[test]
    fn nested_bound_decodes_mutf8_without_replacing_raw_sibling_identity() {
        for (outer, current, bound) in [
            ("probe/𐐀Family", "Setter", "Bound"),
            ("probe/Family", "𐐀Setter", "Bound"),
            ("probe/Family", "Setter", "𐐀Bound"),
        ] {
            let outer_raw = JvmBytes::from_unicode(outer).0;
            let rows: Vec<_> = [current, bound]
                .into_iter()
                .map(|inner| {
                    let name = JvmBytes::from_unicode(inner);
                    ResolvedInnerClass {
                        class: [outer_raw.as_slice(), b"$", name.0.as_slice()].concat(),
                        outer_class: Some(outer_raw.clone()),
                        inner_name: Some(name),
                        access_flags: 9,
                    }
                })
                .collect();
            let before = rows.clone();
            let source = format!("{}${bound}", outer.replace('/', "."));
            assert_eq!(
                selected_nested_sibling_bound_spelling(
                    &rows[0].class,
                    &rows[1].class,
                    &source,
                    &rows
                )
                .unwrap(),
                source
            );
            assert_eq!(rows, before);
            for bad in [
                vec![0xed, 0xa0, 0x81],
                vec![0xed, 0xa0],
                b"bad-name".to_vec(),
            ] {
                let mut invalid = rows.clone();
                invalid[1].class = [outer_raw.as_slice(), b"$", bad.as_slice()].concat();
                invalid[1].inner_name = Some(JvmBytes(bad));
                assert!(
                    selected_nested_sibling_bound_spelling(
                        &invalid[0].class,
                        &invalid[1].class,
                        &source,
                        &invalid
                    )
                    .is_err()
                );
            }
        }
    }

    #[test]
    fn generic_void_policy_rejects_incompatible_declarations_before_publication() {
        let parsed = parse_method_signature(b"<T:Lprobe/Bound;>(TT;Z)V", &mut budget()).unwrap();
        let erasures = [TypeParameterErasure {
            name: b"T".to_vec(),
            descriptor: b"Lprobe/Bound;".to_vec(),
        }];
        let candidate = GenericReturnCandidate {
            observation: crate::observation::Token::candidate(),
            parameters: vec![(1, "arg1".to_owned()), (2, "arg2".to_owned())],
            value: GenericReturnValue::CompleteBody {
                result_type: None,
                unread_parameter_slots: vec![],
                parameter_read_bcis: vec![],
                parameter_receiver_calls: vec![],
            },
        };
        let run = |flags,
                   declaration,
                   annotations,
                   refusals,
                   throws,
                   signature: &jarde_reader::signature::MethodSignature,
                   body: Option<&GenericReturnCandidate>| {
            generic_void_body_declaration(
                b"set",
                b"(Lprobe/Bound;Z)V",
                flags,
                declaration,
                annotations,
                refusals,
                &[],
                throws,
                signature,
                body,
                &erasures,
                b"probe/Setter",
                &[],
                &mut budget(),
            )
        };
        assert_eq!(
            run(1, true, false, false, false, &parsed, Some(&candidate))
                .unwrap()
                .unwrap(),
            "public <T extends probe.Bound> void set(T arg1, boolean arg2)"
        );
        for (flags, declaration, annotations, refusals, throws) in [
            (0x0009, true, false, false, false),
            (0x0041, true, false, false, false),
            (1, false, false, false, false),
            (1, true, true, false, false),
            (1, true, false, true, false),
            (1, true, false, false, true),
        ] {
            assert!(
                run(
                    flags,
                    declaration,
                    annotations,
                    refusals,
                    throws,
                    &parsed,
                    Some(&candidate)
                )
                .is_none_or(|result| result.is_err())
            );
        }
        assert!(
            run(1, true, false, false, false, &parsed, None).is_none_or(|result| result.is_err())
        );
        let signature_throws = parse_method_signature(
            b"<T:Lprobe/Bound;>(TT;Z)V^Ljava/lang/Exception;",
            &mut budget(),
        )
        .unwrap();
        assert!(
            run(
                1,
                true,
                false,
                false,
                false,
                &signature_throws,
                Some(&candidate)
            )
            .is_none()
        );
        let mut wrong_slot = candidate.clone();
        wrong_slot.parameters[0].0 = 0;
        assert!(
            run(1, true, false, false, false, &parsed, Some(&wrong_slot))
                .is_none_or(|result| result.is_err())
        );
        for bytes in [
            b"<T:Lprobe/Bound;>(TT;I)V".as_slice(),
            b"<T:Lprobe/Bound;>(TT;Z)TT;",
            b"<T:Lprobe/Bound;U:Ljava/lang/Object;>(TT;Z)V",
            b"<T:Lprobe/Bound;>(TT;Z)V^Ljava/lang/Exception;",
        ] {
            let incompatible = parse_method_signature(bytes, &mut budget()).unwrap();
            assert!(
                run(
                    1,
                    true,
                    false,
                    false,
                    false,
                    &incompatible,
                    Some(&candidate)
                )
                .is_none_or(|result| result.is_err())
            );
        }
    }
}

pub fn body_method_local_generic_throws_declaration(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    declaration_available: bool,
    has_source_refusals: bool,
    has_annotation_refusals: bool,
    has_type_annotations: bool,
    has_default: bool,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Option<Result<String>> {
    if parsed.type_parameters.is_empty()
        || descriptor != b"()V"
        || !parsed.parameters.is_empty()
        || parsed.result.is_some()
        || !matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)])
    {
        return None;
    }
    Some((|| {
        prove_body_method_local_generic_throws_shape(
            raw_name,
            descriptor,
            access_flags,
            declaration_available,
            has_source_refusals,
            has_annotation_refusals,
            has_type_annotations,
            parsed,
            candidate,
            class_internal,
            class_flags,
            class_superclass,
            class_interfaces,
            class_scope,
            class_signature_present,
            budget,
        )?;
        if has_default {
            return Err(Error::unsupported(
                "body_method_local_generic_throws_source_unproved",
                "annotation default has no source position in this method shape",
            ));
        }
        let [parameter] = parsed.type_parameters.as_slice() else {
            return Err(Error::unsupported(
                "body_method_local_generic_throws_source_unproved",
                "method must declare exactly one local type variable",
            ));
        };
        let variable_name = source_type_parameter_name(&parameter.name).ok_or_else(|| {
            Error::unsupported(
                "body_method_local_generic_throws_source_unproved",
                "method variable name has no faithful MUTF-8 Java spelling",
            )
        })?;
        let bound_internal = match parameter.class_bound.as_ref() {
            Some(SignatureType::Class(bound)) => &bound.segments[0].binary_name,
            _ => unreachable!("validated by shape proof"),
        };
        let bound = simple_generic_class_name(bound_internal)?;
        generic_declaration_parameters(descriptor, false).ok_or_else(|| {
            Error::unsupported(
                "body_method_local_generic_throws_source_unproved",
                "physical method descriptor cannot be spelled",
            )
        })?;
        let method_name = source_identifier(raw_name).ok_or_else(|| {
            Error::unsupported(
                "body_method_local_generic_throws_source_unproved",
                "method name has no faithful Java spelling",
            )
        })?;
        Ok(format_generic_method_header(
            access_flags,
            &[format!("{variable_name} extends {bound}")],
            "void",
            &method_name,
            &[],
            &[variable_name.to_owned()],
        ))
    })())
}

pub fn generic_null_instance_method_declaration(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    declaration_available: bool,
    has_annotations: bool,
    has_source_refusals: bool,
    throws: &[String],
    has_physical_throws: bool,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Option<Result<String>> {
    if parsed.type_parameters.is_empty()
        || !matches!(candidate, Some(GenericReturnCandidate { parameters, value: GenericReturnValue::NullLiteral, .. }) if parameters.is_empty())
    {
        return None;
    }
    Some((|| {
        let refused = |why| Error::unsupported("generic_source_shape_unproved", why);
        if access_flags & (ACC_PUBLIC | ACC_STATIC) != ACC_PUBLIC
            || descriptor != b"()Ljava/lang/Number;"
            || class_internal.contains(&b'$')
            || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
            || class_superclass != Some(b"java/lang/Object".as_slice())
            || !class_interfaces.is_empty()
            || !class_scope.is_empty()
            || class_signature_present
            || has_physical_throws
            || !parsed.parameters.is_empty()
            || !parsed.throws.is_empty()
            || !matches!(candidate, Some(GenericReturnCandidate {
            parameters,
            value: GenericReturnValue::NullLiteral,
                ..
        }) if parameters.is_empty())
        {
            return Err(refused(
                "null-return method-local type variable requires a public no-argument instance method on a top-level Object subclass with no interfaces",
            ));
        }
        let [parameter] = parsed.type_parameters.as_slice() else {
            return Err(refused(
                "null-return method must declare exactly one type variable",
            ));
        };
        let Some(SignatureType::TypeVariable(result)) = parsed.result.as_ref() else {
            return Err(refused("null-return method result is not a type variable"));
        };
        let Some(SignatureType::Class(bound)) = parameter.class_bound.as_ref() else {
            return Err(refused(
                "null-return type variable must have one class bound",
            ));
        };
        if parameter.name != *result
            || !parameter.interface_bounds.is_empty()
            || bound.segments.len() != 1
            || bound.segments[0].binary_name.as_slice() != b"java/lang/Number"
            || !bound.segments[0].arguments.is_empty()
        {
            return Err(refused(
                "null-return Signature must be one method-local T extends Number result",
            ));
        }
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        generic_method_declaration(
            raw_name,
            descriptor,
            access_flags,
            declaration_available,
            has_annotations,
            has_source_refusals,
            throws,
            parsed,
            candidate,
            false,
            None,
        )
    })())
}

pub fn static_method_local_generic_throws_declaration(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    declaration_available: bool,
    has_annotations: bool,
    has_source_refusals: bool,
    throws: &[String],
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Option<Result<String>> {
    if parsed.type_parameters.is_empty()
        || !matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)])
    {
        return None;
    }
    Some((|| {
        let refused = |why| Error::unsupported("generic_source_shape_unproved", why);
        if access_flags & ACC_STATIC == 0
            || class_internal.contains(&b'$')
            || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
            || class_superclass != Some(b"java/lang/Object".as_slice())
            || !class_interfaces.is_empty()
            || !class_scope.is_empty()
            || class_signature_present
            || !matches!(
                candidate,
                Some(GenericReturnCandidate {
                    value: GenericReturnValue::Parameter(_),
                    ..
                })
            )
            || !matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)])
        {
            return Err(refused(
                "method-local generic throws requires a direct parameter return in a top-level, non-generic Object subclass with no interfaces",
            ));
        }
        let [SignatureType::TypeVariable(throws_variable)] = parsed.throws.as_slice() else {
            unreachable!("shape checked above")
        };
        let mut matching_parameters = parsed
            .type_parameters
            .iter()
            .filter(|parameter| parameter.name == *throws_variable);
        let Some(parameter) = matching_parameters.next() else {
            return Err(refused(
                "throws variable is outside the method-local type parameter scope",
            ));
        };
        if matching_parameters.next().is_some() {
            return Err(refused(
                "throws variable does not identify a unique method-local type parameter",
            ));
        }
        if !parameter.interface_bounds.is_empty() {
            return Err(refused(
                "throws variable cannot have additional interface bounds",
            ));
        }
        let Some(SignatureType::Class(bound)) = parameter.class_bound.as_ref() else {
            return Err(refused(
                "throws variable must have one explicit class first bound",
            ));
        };
        let [segment] = bound.segments.as_slice() else {
            return Err(refused("throws variable bound is nested"));
        };
        if !segment.arguments.is_empty()
            || !matches!(
                segment.binary_name.as_slice(),
                b"java/lang/Throwable"
                    | b"java/lang/Exception"
                    | b"java/lang/RuntimeException"
                    | b"java/lang/Error"
            )
        {
            return Err(refused(
                "throws variable bound is not a proved JDK throwable root",
            ));
        }
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        generic_method_declaration(
            raw_name,
            descriptor,
            access_flags,
            declaration_available,
            has_annotations,
            has_source_refusals,
            throws,
            parsed,
            candidate,
            true,
            None,
        )
    })())
}

fn prove_body_method_local_generic_throws_shape(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    declaration_available: bool,
    has_source_refusals: bool,
    has_annotation_refusals: bool,
    has_type_annotations: bool,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Result<()> {
    let refused = |why| Error::unsupported("body_method_local_generic_throws_source_unproved", why);
    if !declaration_available
        || has_source_refusals
        || has_annotation_refusals
        || has_type_annotations
        || access_flags
            & !(ACC_PUBLIC
                | ACC_PRIVATE
                | ACC_PROTECTED
                | ACC_FINAL
                | ACC_SYNCHRONIZED
                | ACC_STRICT)
            != 0
        || descriptor != b"()V"
        || matches!(raw_name, b"<init>" | b"<clinit>")
        || is_object_instance_method_name(raw_name)
        || class_internal.contains(&b'$')
        || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
        || !class_scope.is_empty()
        || class_signature_present
        || !parsed.parameters.is_empty()
        || parsed.result.is_some()
        || parsed.type_parameters.len() != 1
        || !matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)])
    {
        return Err(refused(
            "method Signature, member, or class hierarchy exceeds the supported empty-void shape",
        ));
    }
    if !matches!(
        candidate,
        Some(GenericReturnCandidate {
            parameters,
            value: GenericReturnValue::EmptyVoid,
                ..
        }) if parameters.is_empty()
    ) {
        return Err(refused(
            "same-run AST/Code/SSA does not prove the exact empty void body",
        ));
    }
    let [parameter] = parsed.type_parameters.as_slice() else {
        unreachable!("shape checked above")
    };
    let Some(SignatureType::Class(bound)) = parameter.class_bound.as_ref() else {
        return Err(refused(
            "method variable must have exactly one proved class bound and no additional bounds",
        ));
    };
    if !parameter.interface_bounds.is_empty() {
        return Err(refused(
            "method variable must have exactly one proved class bound and no additional bounds",
        ));
    }
    let [segment] = bound.segments.as_slice() else {
        return Err(refused("method variable bound is nested"));
    };
    if !segment.arguments.is_empty()
        || !matches!(
            segment.binary_name.as_slice(),
            b"java/lang/Throwable"
                | b"java/lang/Exception"
                | b"java/lang/RuntimeException"
                | b"java/lang/Error"
        )
    {
        return Err(refused(
            "method variable bound is not a proved JDK throwable root",
        ));
    }
    source_type_parameter_name(&parameter.name)
        .ok_or_else(|| refused("method variable name has no faithful MUTF-8 Java spelling"))?;
    if parsed.throws.first() != Some(&SignatureType::TypeVariable(parameter.name.clone())) {
        return Err(refused(
            "throws variable is not the source-spellable method-local variable",
        ));
    }
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    Ok(())
}

pub(crate) fn format_generic_method_header(
    flags: u16,
    parameters: &[String],
    result: &str,
    name: &str,
    arguments: &[String],
    throws: &[String],
) -> String {
    let mut words = Vec::new();
    words.extend(if flags & ACC_PUBLIC != 0 {
        Some("public")
    } else if flags & ACC_PROTECTED != 0 {
        Some("protected")
    } else if flags & ACC_PRIVATE != 0 {
        Some("private")
    } else {
        None
    });
    for (bit, word) in [
        (ACC_STATIC, "static"),
        (ACC_FINAL, "final"),
        (ACC_ABSTRACT, "abstract"),
        (ACC_NATIVE, "native"),
        (ACC_SYNCHRONIZED, "synchronized"),
        (ACC_STRICT, "strictfp"),
    ] {
        if flags & bit != 0 {
            words.push(word);
        }
    }
    format!(
        "{}{}{} {}({}){}",
        if words.is_empty() {
            String::new()
        } else {
            format!("{} ", words.join(" "))
        },
        if parameters.is_empty() {
            String::new()
        } else {
            format!("<{}> ", parameters.join(", "))
        },
        result,
        name,
        arguments.join(", "),
        if throws.is_empty() {
            String::new()
        } else {
            format!(" throws {}", throws.join(", "))
        }
    )
}

/// Source declaration policy over a same-run body certificate supplied by the caller.
/// The caller binds the certificate to the selected physical method before invoking this policy.
pub fn generic_method_body_declaration(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    has_annotations: bool,
    has_source_refusals: bool,
    physical_throws: &[String],
    parsed: &jarde_reader::signature::MethodSignature,
    proof: &SameClassGenericMethodBodyProof,
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<String> {
    let refused = |why| Error::unsupported("generic_source_shape_unproved", why);
    let is_static = access_flags & ACC_STATIC != 0;
    if raw_name == b"<init>"
        || access_flags
            & !(ACC_PUBLIC
                | ACC_PRIVATE
                | ACC_PROTECTED
                | ACC_STATIC
                | ACC_FINAL
                | ACC_SYNCHRONIZED
                | ACC_STRICT)
            != 0
        || has_annotations
        || has_source_refusals
    {
        return Err(refused(
            "body certificate does not match a preservable generic method declaration",
        ));
    }
    if parsed.type_parameters.is_empty() {
        return Err(refused("method has no local type parameters"));
    }
    let formal_count = proof.formal_names.len();
    let signature_type_count = parsed
        .parameters
        .len()
        .saturating_add(usize::from(parsed.result.is_some()))
        .saturating_add(parsed.throws.len());
    let bound_count = parsed
        .type_parameters
        .iter()
        .map(|parameter| {
            parameter
                .class_bound
                .iter()
                .count()
                .saturating_add(parameter.interface_bounds.len())
        })
        .fold(0usize, usize::saturating_add);
    let proof_work = formal_count
        .saturating_mul(formal_count)
        .saturating_add(signature_type_count.saturating_mul(parsed.type_parameters.len()))
        .saturating_add(formal_count)
        .saturating_add(proof.return_sources.len())
        .saturating_add(bound_count)
        .saturating_add(parsed.type_parameters.len())
        .saturating_add(parsed.throws.len());
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(proof_work).unwrap_or(u64::MAX),
    )?;
    let mut variables = Vec::with_capacity(parsed.type_parameters.len());
    for parameter in &parsed.type_parameters {
        budget.poll()?;
        let variable = source_type_parameter_name(&parameter.name)
            .ok_or_else(|| refused("type variable name has no faithful MUTF-8 Java spelling"))?;

        let mut bounds = Vec::new();
        for bound in parameter
            .class_bound
            .iter()
            .chain(parameter.interface_bounds.iter())
        {
            budget.poll()?;
            let SignatureType::Class(class) = bound else {
                return Err(refused("type-variable bound is unsupported"));
            };
            let [segment] = class.segments.as_slice() else {
                return Err(refused("nested bounds are unsupported"));
            };
            if !segment.arguments.is_empty() {
                return Err(refused("parameterized bounds are unsupported"));
            }
            bounds.push(simple_generic_class_name(&segment.binary_name)?);
        }
        if bounds.is_empty() {
            bounds.push("java.lang.Object".to_owned());
        }
        variables.push((
            parameter.name.clone(),
            variable.to_owned(),
            bounds.join(" & "),
        ));
    }

    let signature = generic_declaration_parameters(descriptor, is_static)
        .ok_or_else(|| refused("physical descriptor cannot be spelled"))?;
    let Some(slots) = jarde_jvm::method_ir::parameter_positions(
        &descriptor_facts(descriptor, DescriptorKind::Method)?,
        is_static,
    ) else {
        return Err(refused("physical parameter slots are unavailable"));
    };
    if slots.len() != parsed.parameters.len()
        || proof.formal_names.len() != slots.len()
        || signature.len() != slots.len()
        || proof
            .formal_names
            .iter()
            .map(|(slot, _)| *slot)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != slots.len()
    {
        return Err(refused(
            "body formal identities do not match physical parameters",
        ));
    }
    let mut arguments = Vec::with_capacity(slots.len());
    let mut parameter_types = Vec::with_capacity(slots.len());
    for (position, slot) in slots.iter().enumerate() {
        budget.poll()?;
        let Some((_, name)) = proof
            .formal_names
            .iter()
            .find(|(formal_slot, _)| formal_slot == slot)
        else {
            return Err(refused("body certificate omits a physical formal"));
        };
        if !is_java_identifier(name) {
            return Err(refused("formal name has no Java source spelling"));
        }
        let ty = spell_generic_body_signature_type(
            &parsed.parameters[position],
            &variables,
            class_scope,
            budget,
            0,
        )?;
        parameter_types.push(parsed.parameters[position].clone());
        arguments.push(format!("{ty} {name}"));
    }
    let result_name = match parsed.result.as_ref() {
        None => {
            if !proof.return_sources.is_empty() {
                return Err(refused("void method body has value-return sources"));
            }
            "void".to_owned()
        }
        Some(result) => {
            if proof.return_sources.is_empty() {
                return Err(refused(
                    "value-returning method has no certified return source",
                ));
            }
            for source in &proof.return_sources {
                budget.poll()?;
                match source {
                    SameClassGenericBodyReturnSource::Parameter { slot } => {
                        let Some(position) = slots.iter().position(|formal| formal == slot) else {
                            return Err(refused(
                                "return source is outside the physical formal slots",
                            ));
                        };
                        if parameter_types.get(position) != Some(result) {
                            return Err(refused(
                                "return source formal does not have the Signature result type",
                            ));
                        }
                    }
                    SameClassGenericBodyReturnSource::Call { .. } => {}
                }
            }
            spell_generic_body_signature_type(result, &variables, class_scope, budget, 0)?
        }
    };
    let throws = parsed
        .throws
        .iter()
        .map(|exception| match exception {
            SignatureType::Class(class) => {
                budget.poll()?;
                let [segment] = class.segments.as_slice() else {
                    return Err(refused("nested throws type is unsupported"));
                };
                if !segment.arguments.is_empty() {
                    return Err(refused("parameterized throws type is unsupported"));
                }
                simple_generic_class_name(&segment.binary_name)
            }
            SignatureType::TypeVariable(_) => {
                spell_generic_body_signature_type(exception, &variables, class_scope, budget, 0)
            }
            _ => Err(refused(
                "method-local generic throws type lacks a body proof",
            )),
        })
        .collect::<Result<Vec<_>>>()?;
    if throws != physical_throws {
        return Err(refused(
            "generic throws Signature differs from physical Exceptions",
        ));
    }
    let name = source_identifier(raw_name)
        .ok_or_else(|| refused("method name has no faithful source spelling"))?;
    let type_parameters = variables
        .iter()
        .map(|(_, name, bound)| format!("{name} extends {bound}"))
        .collect::<Vec<_>>();
    Ok(format_generic_method_header(
        access_flags,
        &type_parameters,
        &result_name,
        &name,
        &arguments,
        &throws,
    ))
}

/// The existing exact List<String> null-return route, before strict declaration policy.
pub fn ordinary_parameterized_null_return_is_proved(
    recovered: bool,
    single_signature: bool,
    has_physical_throws: bool,
    descriptor: &[u8],
    access_flags: u16,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
) -> bool {
    recovered
        && parsed.type_parameters.is_empty()
        && single_signature
        && parsed.parameters.is_empty()
        && parsed.throws.is_empty()
        && !has_physical_throws
        && descriptor == b"()Ljava/util/List;"
        && access_flags & (ACC_PUBLIC | ACC_STATIC) == ACC_PUBLIC
        && !class_internal.contains(&b'$')
        && class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) == 0
        && class_superclass == Some(b"java/lang/Object".as_slice())
        && class_interfaces.is_empty()
        && class_scope.is_empty()
        && !class_signature_present
        && matches!(parsed.result.as_ref(), Some(SignatureType::Class(class))
                    if matches!(class.segments.as_slice(), [segment]
                        if segment.binary_name.as_slice() == b"java/util/List"
                            && matches!(segment.arguments.as_slice(), [TypeArgument::Exact(SignatureType::Class(string))]
                                if matches!(string.segments.as_slice(), [string]
                                    if string.binary_name.as_slice() == b"java/lang/String"
                                        && string.arguments.is_empty()))))
        && matches!(candidate, Some(GenericReturnCandidate {
                    parameters,
                    value: GenericReturnValue::NullLiteral,
                ..
                }) if parameters.is_empty())
}

/// Spell ordinary parameterized declarations from reader-validated erasure and same-run facts.
/// Physical proof identity is compared by the caller; its fact is consumed at the original guard.
pub fn ordinary_parameterized_declaration(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    declaration_available: bool,
    recovered: bool,
    no_body: bool,
    has_no_body_kind: bool,
    has_code: bool,
    markers_empty: bool,
    has_annotations: bool,
    has_source_refusals: bool,
    has_type_annotations: bool,
    parameter_annotation_uses: &[Vec<String>],
    has_default: bool,
    physical_throws: &[String],
    has_physical_throws: bool,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    body_proof: Option<&SameClassGenericMethodBodyProof>,
    proof_subject_matches: bool,
    class_flags: u16,
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    allow_null_return: bool,
    proved_list_parameters: &[(u16, Vec<String>)],
    budget: &mut Budget,
) -> Result<(String, &'static str)> {
    let refused = |why| Error::unsupported("ordinary_generic_source_unproved", why);
    let empty_void_wildcard_parameter = ordinary_empty_void_wildcard_parameter_is_proved(
        access_flags,
        has_annotations,
        has_source_refusals,
        has_type_annotations,
        has_physical_throws,
        parsed,
        candidate,
        class_flags,
        class_internal,
        class_superclass,
        class_interfaces,
    );
    if !declaration_available
        || matches!(raw_name, b"<init>" | b"<clinit>")
        || access_flags
            & !(ACC_PUBLIC
                | ACC_PRIVATE
                | ACC_PROTECTED
                | ACC_STATIC
                | ACC_FINAL
                | ACC_ABSTRACT
                | ACC_NATIVE
                | ACC_SYNCHRONIZED
                | ACC_STRICT
                | ACC_VARARGS)
            != 0
        || has_default
        || has_source_refusals
        || has_type_annotations
    {
        return Err(refused(
            "member flags, annotation positions, or source name cannot be preserved",
        ));
    }
    if !(recovered && markers_empty || no_body && has_no_body_kind) {
        return Err(refused(
            "method body or no-body declaration has no complete source proof",
        ));
    }
    let mut signature = method_descriptor(
        descriptor,
        access_flags & ACC_STATIC != 0,
        access_flags & ACC_VARARGS != 0,
    )
    .ok_or_else(|| refused("physical method descriptor cannot be spelled"))?;
    if signature.parameters.len() != parsed.parameters.len() {
        return Err(refused("generic and physical parameter positions differ"));
    }
    let source_type_path = match candidate.map(|candidate| &candidate.value) {
        Some(GenericReturnValue::MemberCreation { target, .. }) => {
            Some(target.source_type_path.as_slice())
        }
        Some(GenericReturnValue::StaticMemberCreation { target, .. }) => {
            Some(target.source_type_path.as_slice())
        }
        _ => None,
    };
    let mut parameter_types = Vec::with_capacity(parsed.parameters.len());
    for parameter in &parsed.parameters {
        parameter_types.push(spell_ordinary_signature_type_with_member_path(
            parameter,
            class_scope,
            source_type_path.unwrap_or(&[]),
            budget,
            0,
        )?);
    }
    let result = parsed
        .result
        .as_ref()
        .map(|result| {
            spell_ordinary_signature_type_with_member_path(
                result,
                class_scope,
                source_type_path.unwrap_or(&[]),
                budget,
                0,
            )
        })
        .transpose()?;
    let mut generic_throws = Vec::with_capacity(parsed.throws.len());
    for exception in &parsed.throws {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        match exception {
            SignatureType::Class(class) => {
                let [segment] = class.segments.as_slice() else {
                    return Err(refused(
                        "nested throws type has no preserved source position",
                    ));
                };
                if !segment.arguments.is_empty() {
                    return Err(refused(
                        "parameterized throws type has no preserved source position",
                    ));
                }
                generic_throws.push(simple_generic_class_name(&segment.binary_name)?);
            }
            SignatureType::TypeVariable(variable) => {
                if !no_body {
                    if recovered {
                        prove_body_generic_throws_shape(
                            raw_name,
                            descriptor,
                            access_flags,
                            recovered,
                            parsed,
                            candidate,
                            class_internal,
                            class_flags,
                            class_superclass,
                            class_interfaces,
                            budget,
                        )?;
                    } else {
                        return Err(refused(
                            "generic throws variable has no complete body or no-body proof",
                        ));
                    }
                }
                let Some(class_variable) = class_scope.iter().find(|item| item.name == *variable)
                else {
                    return Err(refused(
                        "generic throws variable is not present in the emitted class header",
                    ));
                };
                if !matches!(
                    class_variable.descriptor.as_slice(),
                    b"Ljava/lang/Throwable;"
                        | b"Ljava/lang/Exception;"
                        | b"Ljava/lang/RuntimeException;"
                        | b"Ljava/lang/Error;"
                ) {
                    return Err(refused(
                        "generic throws variable first bound is not a proved JDK throwable root",
                    ));
                }
                let name = source_type_parameter_name(variable).ok_or_else(|| {
                    refused("generic throws variable name has no faithful MUTF-8 Java spelling")
                })?;

                generic_throws.push(name.to_owned());
            }
            _ => {
                return Err(refused(
                    "generic throws type has no preserved Java source spelling",
                ));
            }
        }
    }
    let throws = if parsed.throws.is_empty() {
        physical_throws.to_vec()
    } else {
        generic_throws
    };
    if recovered {
        let returned = if let Some(proof) = body_proof {
            if !proof_subject_matches
                || proof.formal_names.len() != signature.parameters.len()
                || (parsed.result.is_some() && proof.return_sources.is_empty())
                || (proof.constructor_call_bci.is_some() != (raw_name == b"<init>"))
                || proof
                    .field_write_sites
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    != proof.field_write_sites.len()
            {
                return Err(refused(
                    "same-run generic body proof has different method formals",
                ));
            }
            for ((_, slot), (proof_slot, proof_name)) in
                signature.parameters.iter().zip(&proof.formal_names)
            {
                if slot != proof_slot || !is_java_identifier(proof_name) {
                    return Err(refused(
                        "same-run generic body proof has different formal identity",
                    ));
                }
            }
            let mut returned_positions = Vec::new();
            for source in &proof.return_sources {
                if let SameClassGenericBodyReturnSource::Parameter { slot } = source {
                    let Some(position) = signature.parameters.iter().position(|(_, at)| at == slot)
                    else {
                        return Err(refused("generic return source is not a physical formal"));
                    };
                    if parsed.result.as_ref() != Some(&parsed.parameters[position]) {
                        return Err(refused(
                            "generic return formal differs from the Signature result",
                        ));
                    }
                    returned_positions.push(position);
                }
            }
            Some(returned_positions)
        } else {
            let candidate = candidate.ok_or_else(|| {
                refused("same-run Program/SSA cannot prove the body under parameterized types")
            })?;
            if candidate.parameters.len() != signature.parameters.len() {
                return Err(refused("same-run parameter count differs from descriptor"));
            }
            for ((_, slot), (candidate_slot, name)) in
                signature.parameters.iter().zip(&candidate.parameters)
            {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                if slot != candidate_slot || !is_java_identifier(name) {
                    return Err(refused(
                        "same-run parameter slot or source name is unproved",
                    ));
                }
            }
            let complete_body_parameters_proved = if let GenericReturnValue::CompleteBody {
                unread_parameter_slots,
                ..
            } = &candidate.value
            {
                budget.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(
                        parsed
                            .parameters
                            .len()
                            .saturating_add(unread_parameter_slots.len()),
                    )
                    .unwrap_or(u64::MAX),
                )?;
                let mut proved = unread_parameter_slots
                    .windows(2)
                    .all(|slots| slots[0] < slots[1]);
                budget.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(proved_list_parameters.len()).unwrap_or(u64::MAX),
                )?;
                proved &= proved_list_parameters
                    .windows(2)
                    .all(|slots| slots[0].0 < slots[1].0);
                for slot in unread_parameter_slots {
                    budget.poll()?;
                    proved &= signature
                        .parameters
                        .binary_search_by_key(slot, |(_, physical_slot)| *physical_slot)
                        .is_ok();
                }
                for (position, parameter) in parsed.parameters.iter().enumerate() {
                    budget.poll()?;
                    let physical = signature.parameters.get(position);
                    let list_signature_element = match parameter {
                        SignatureType::Class(class) => {
                            budget.charge(
                                CountedBudgetDimension::AnalysisSteps,
                                u64::try_from(class.segments.len())
                                    .unwrap_or(u64::MAX)
                                    .saturating_add(1),
                            )?;
                            match class.segments.as_slice() {
                                [list] if list.binary_name.as_slice() == b"java/util/List" => {
                                    match list.arguments.as_slice() {
                                        [TypeArgument::Exact(SignatureType::Class(element))] => {
                                            budget.charge(
                                                CountedBudgetDimension::AnalysisSteps,
                                                u64::try_from(element.segments.len())
                                                    .unwrap_or(u64::MAX)
                                                    .saturating_add(1),
                                            )?;
                                            if element.segments.is_empty()
                                                || element
                                                    .segments
                                                    .iter()
                                                    .any(|segment| !segment.arguments.is_empty())
                                            {
                                                None
                                            } else {
                                                let copied_bytes = element.segments.iter().fold(
                                                    0u64,
                                                    |total, segment| {
                                                        total.saturating_add(
                                                            u64::try_from(
                                                                segment.binary_name.len(),
                                                            )
                                                            .unwrap_or(u64::MAX),
                                                        )
                                                    },
                                                );
                                                budget.charge(
                                                    CountedBudgetDimension::OutputBytes,
                                                    copied_bytes,
                                                )?;
                                                let element_signature =
                                                    SignatureType::Class(element.clone());
                                                Some(
                                                    spell_ordinary_signature_type_with_member_path(
                                                        &element_signature,
                                                        class_scope,
                                                        source_type_path.unwrap_or(&[]),
                                                        budget,
                                                        0,
                                                    )?,
                                                )
                                            }
                                        }
                                        _ => None,
                                    }
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    let list_actual_types = physical.and_then(|(_, slot)| {
                        proved_list_parameters
                            .binary_search_by_key(slot, |(proved_slot, _)| *proved_slot)
                            .ok()
                            .map(|index| &proved_list_parameters[index].1)
                    });
                    let list_element_matches =
                        match (list_signature_element.as_deref(), list_actual_types) {
                            (Some(_), Some(actual_types)) if actual_types.is_empty() => true,
                            (Some(expected), Some(actual_types)) => {
                                actual_types.first().is_some_and(|actual| {
                                    actual.as_str() == expected
                                        && actual_types.iter().all(|candidate| candidate == actual)
                                })
                            }
                            _ => false,
                        };
                    let preserved = physical.is_some_and(|(physical_type, slot)| {
                        matches!(parameter, SignatureType::TypeVariable(_))
                            || parameter_types.get(position) == Some(physical_type)
                            || unread_parameter_slots.binary_search(slot).is_ok()
                            || list_element_matches
                    });
                    proved &= preserved;
                }
                proved
            } else {
                false
            };
            match &candidate.value {
                GenericReturnValue::EmptyVoid
                    if parsed.result.is_none()
                        && matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)]) =>
                {
                    Some(Vec::new())
                }
                GenericReturnValue::EmptyVoid if empty_void_wildcard_parameter => Some(Vec::new()),
                GenericReturnValue::EmptyVoid => None,
                // A complete void body whose parameters are only read keeps compiling
                // when a class-scope variable replaces the erased parameter spelling: the reader
                // proved the variable's erasure is that parameter, so every read stays assignable to
                // the context that accepted the erased reference. The same-run candidate refused any
                // reassignment of the parameter locals, which is the write side this proof refuses.
                GenericReturnValue::CompleteBody {
                    result_type: None, ..
                } if parsed.result.is_none()
                    && parsed.throws.is_empty()
                    && complete_body_parameters_proved =>
                {
                    Some(Vec::new())
                }
                GenericReturnValue::CompleteBody {
                    result_type: Some(result_type),
                    ..
                } if parsed.throws.is_empty()
                    && ordinary_primitive_result_matches(
                        result_type,
                        parsed.result.as_ref(),
                        result.as_deref(),
                        signature.returns.as_deref(),
                    )
                    && complete_body_parameters_proved
                    && !proved_list_parameters.is_empty() =>
                {
                    Some(Vec::new())
                }
                GenericReturnValue::CompleteBody { .. } => None,
                GenericReturnValue::NullLiteral if allow_null_return => Some(Vec::new()),
                GenericReturnValue::NullLiteral => None,
                GenericReturnValue::Parameter(slot) => signature
                    .parameters
                    .iter()
                    .position(|(_, at)| *at == *slot)
                    .map(|position| vec![position]),
                GenericReturnValue::Conditional {
                    test,
                    when_true,
                    when_false,
                } => {
                    let test_position =
                        signature.parameters.iter().position(|(_, at)| *at == *test);
                    let true_position = signature
                        .parameters
                        .iter()
                        .position(|(_, at)| *at == *when_true);
                    let false_position = signature
                        .parameters
                        .iter()
                        .position(|(_, at)| *at == *when_false);
                    match (test_position, true_position, false_position) {
                        (Some(test_position), Some(true_position), Some(false_position))
                            if parsed.parameters[test_position] == SignatureType::Base(b'Z') =>
                        {
                            Some(vec![true_position, false_position])
                        }
                        _ => None,
                    }
                }
                GenericReturnValue::MemberCreation {
                    target,
                    qualifier_slot,
                } => {
                    let qualifier_position = signature
                        .parameters
                        .iter()
                        .position(|(_, slot)| *slot == *qualifier_slot);
                    qualifier_position
                        .is_some_and(|position| {
                            let parameter_matches =
                                parsed.parameters.get(position).is_some_and(|ty| {
                                    member_signature_class_matches(
                                        ty,
                                        &target.outer,
                                        &target.source_type_path,
                                        false,
                                    )
                                });
                            parameter_matches
                                && parsed.result.as_ref().is_some_and(|ty| {
                                    signature_result_matches_member_creation(ty, target)
                                })
                        })
                        .then_some(Vec::new())
                }
                GenericReturnValue::StaticMemberCreation { target, .. } => parsed
                    .result
                    .as_ref()
                    .is_some_and(|ty| {
                        member_signature_class_matches(
                            ty,
                            &target.owner,
                            &target.source_type_path,
                            false,
                        )
                    })
                    .then_some(Vec::new()),
                GenericReturnValue::TypedFunctional { target } => {
                    let expected = match target.kind {
                        TypedFunctionalKind::FunctionStringInteger => {
                            "java.util.function.Function<java.lang.String, java.lang.Integer>"
                        }
                        TypedFunctionalKind::SupplierString => {
                            "java.util.function.Supplier<java.lang.String>"
                        }
                    };
                    (parsed.parameters.is_empty() && result.as_deref() == Some(expected))
                        .then_some(Vec::new())
                }
                GenericReturnValue::ArrayCreation { array_type, .. }
                    if parsed.type_parameters.is_empty()
                        && parsed.parameters.is_empty()
                        && signature.parameters.is_empty()
                        && parsed.throws.is_empty()
                        && !has_physical_throws
                        && throws.is_empty()
                        && candidate.parameters.is_empty()
                        && wildcard_array_return_matches(
                            array_type,
                            parsed.result.as_ref(),
                            signature.returns.as_deref(),
                            budget,
                        )? =>
                {
                    Some(Vec::new())
                }
                GenericReturnValue::ArrayCreation { .. } => None,
            }
        }
        .ok_or_else(|| {
            refused("return source is not a proven parameter value or selected member creation")
        })?;
        if body_proof.is_none() {
            let Some(candidate) = candidate else {
                return Err(refused("generic return candidate is missing"));
            };
            if !matches!(
                candidate.value,
                GenericReturnValue::MemberCreation { .. }
                    | GenericReturnValue::StaticMemberCreation { .. }
                    | GenericReturnValue::TypedFunctional { .. }
            ) && returned
                .iter()
                .any(|position| parsed.result.as_ref() != Some(&parsed.parameters[*position]))
            {
                return Err(refused(
                    "returned parameter has a different generic source type",
                ));
            }
        }
    }
    for ((spelling, _), ty) in signature.parameters.iter_mut().zip(parameter_types) {
        *spelling = ty;
    }
    signature.returns = result;
    let name = source_identifier(raw_name)
        .ok_or_else(|| refused("method name has no faithful Java spelling"))?;
    let mut words = Vec::new();
    words.extend(if access_flags & ACC_PUBLIC != 0 {
        Some("public")
    } else if access_flags & ACC_PROTECTED != 0 {
        Some("protected")
    } else if access_flags & ACC_PRIVATE != 0 {
        Some("private")
    } else {
        None
    });
    if access_flags & ACC_STATIC != 0 {
        words.push("static");
    }
    if access_flags & ACC_FINAL != 0 {
        words.push("final");
    }
    if access_flags & ACC_ABSTRACT != 0 {
        words.push("abstract");
    }
    if access_flags & ACC_NATIVE != 0 {
        words.push("native");
    }
    if access_flags & ACC_SYNCHRONIZED != 0 {
        words.push("synchronized");
    }
    if access_flags & ACC_STRICT != 0 {
        words.push("strictfp");
    }
    if class_flags & ACC_INTERFACE != 0
        && access_flags & (ACC_STATIC | ACC_ABSTRACT) == 0
        && has_code
    {
        words.push("default");
    }
    let names = body_proof
        .map(|proof| proof.formal_names.as_slice())
        .or_else(|| candidate.map(|candidate| candidate.parameters.as_slice()));
    let mut arguments = Vec::new();
    for (position, (ty, slot)) in signature.parameters.iter().enumerate() {
        let name = names
            .and_then(|names| names.get(position).map(|(_, name)| name.clone()))
            .unwrap_or_else(|| format!("arg{slot}"));
        let varargs = signature.varargs && position + 1 == signature.parameters.len();
        let annotations = parameter_annotation_uses
            .get(position)
            .into_iter()
            .flatten()
            .map(|annotation| format!("{annotation} "))
            .collect::<String>();
        arguments.push(format!(
            "{annotations}{} {name}",
            crate::declaration::varargs_type(ty, varargs)
        ));
    }
    let declaration = format!(
        "{}{} {}({}){}",
        if words.is_empty() {
            String::new()
        } else {
            format!("{} ", words.join(" "))
        },
        signature.returns.as_deref().unwrap_or("void"),
        name,
        arguments.join(", "),
        if throws.is_empty() {
            String::new()
        } else {
            format!(" throws {}", throws.join(", "))
        },
    );
    let proof = if no_body {
        "physical NoBody/Signature erasure proof"
    } else if allow_null_return {
        "same-run AST/Code/SSA exact null-return and parameterized method Signature erasure proof"
    } else if recovered
        && parsed.type_parameters.is_empty()
        && parsed
            .throws
            .iter()
            .any(|throws| matches!(throws, SignatureType::TypeVariable(_)))
    {
        "same-run AST/Code/SSA empty-void proof"
    } else if empty_void_wildcard_parameter {
        "same-run AST/Code/SSA unused-parameter proof"
    } else if matches!(
        candidate.map(|candidate| &candidate.value),
        Some(GenericReturnValue::TypedFunctional { .. })
    ) {
        "same-run direct Code/SSA/Program and LambdaMetafactory target proof"
    } else if matches!(
        candidate.map(|candidate| &candidate.value),
        Some(GenericReturnValue::CompleteBody { .. })
    ) {
        "same-run AST/Code/SSA complete body proof"
    } else {
        "same-run AST/SSA parameter-return proof"
    };
    Ok((declaration, proof))
}

/// Accept only `T<?>[][]...` when the same-run creation and reader-spelled physical return name
/// state the same array shape. Erasure validity itself is established by the reader before this
/// declaration projection is called; this gate narrows that proof to one unbounded wildcard leaf.
pub fn wildcard_array_return_matches(
    actual_array_type: &str,
    signature_result: Option<&SignatureType>,
    physical_return_type: Option<&str>,
    budget: &mut Budget,
) -> Result<bool> {
    let Some(physical_return_type) = physical_return_type else {
        return Ok(false);
    };
    if actual_array_type != physical_return_type {
        return Ok(false);
    }

    let mut physical_leaf = actual_array_type;
    let mut physical_rank = 0usize;
    while let Some(leaf) = physical_leaf.strip_suffix("[]") {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        physical_leaf = leaf;
        physical_rank = physical_rank.saturating_add(1);
    }
    if physical_rank == 0 || physical_leaf.is_empty() {
        return Ok(false);
    }

    let Some(mut signature_leaf) = signature_result else {
        return Ok(false);
    };
    let mut signature_rank = 0usize;
    while let SignatureType::Array(component) = signature_leaf {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        signature_leaf = component;
        signature_rank = signature_rank.saturating_add(1);
    }
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    let SignatureType::Class(class) = signature_leaf else {
        return Ok(false);
    };
    let [segment] = class.segments.as_slice() else {
        return Ok(false);
    };
    if segment.arguments.is_empty() {
        return Ok(false);
    }
    let mut all_unbounded = true;
    for argument in &segment.arguments {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        all_unbounded &= matches!(argument, TypeArgument::Any);
    }
    Ok(all_unbounded
        && signature_rank == physical_rank
        && simple_generic_class_name(&segment.binary_name)? == physical_leaf)
}

/// The first narrow empty-body parameter projection: one static `List` wildcard whose bound has
/// an existing simple source spelling. The body candidate is produced from the same complete
/// Code/AST/SSA run and carries every physical parameter slot.
fn ordinary_empty_void_wildcard_parameter_is_proved(
    access_flags: u16,
    has_annotations: bool,
    has_source_refusals: bool,
    has_type_annotations: bool,
    has_physical_throws: bool,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_flags: u16,
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
) -> bool {
    candidate.is_some_and(|candidate| matches!(&candidate.value, GenericReturnValue::EmptyVoid))
        && access_flags & ACC_STATIC != 0
        && access_flags & !(ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED | ACC_STATIC) == 0
        && !has_physical_throws
        && parsed.result.is_none()
        && parsed.throws.is_empty()
        && !has_annotations
        && !has_source_refusals
        && !has_type_annotations
        && !class_internal.contains(&b'$')
        && class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) == 0
        && class_superclass == Some(b"java/lang/Object".as_slice())
        && class_interfaces.is_empty()
        && matches!(parsed.parameters.as_slice(), [SignatureType::Class(class)]
            if matches!(class.segments.as_slice(), [segment]
                if segment.binary_name == b"java/util/List"
                    && matches!(segment.arguments.as_slice(), [argument]
                        if ordinary_simple_wildcard_argument(argument))))
}

fn ordinary_simple_wildcard_argument(argument: &TypeArgument) -> bool {
    let simple_class = |ty: &SignatureType| {
        matches!(ty, SignatureType::Class(class)
            if matches!(class.segments.as_slice(), [segment]
                if segment.arguments.is_empty()
                    && simple_generic_class_name(&segment.binary_name).is_ok()))
    };
    let primitive_array = |ty: &SignatureType| {
        matches!(ty, SignatureType::Array(element)
            if matches!(element.as_ref(), SignatureType::Base(b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z')))
    };
    match argument {
        TypeArgument::Any => true,
        TypeArgument::Extends(ty) | TypeArgument::Super(ty) => {
            simple_class(ty) || primitive_array(ty)
        }
        TypeArgument::Exact(_) => false,
    }
}

fn member_signature_class_matches(
    ty: &SignatureType,
    expected_binary_name: &JvmBytes,
    source_type_path: &[crate::report::MemberInnerSourceSegmentInput],
    require_all_source_segments: bool,
) -> bool {
    let SignatureType::Class(class) = ty else {
        return false;
    };
    let Some(expected) = source_type_path
        .iter()
        .find(|segment| &segment.binary_name == expected_binary_name)
    else {
        return false;
    };
    if class
        .segments
        .last()
        .is_none_or(|segment| segment.binary_name.as_slice() != expected_binary_name.0.as_slice())
        || (require_all_source_segments && class.segments.len() != 2)
    {
        return false;
    }
    if expected.type_parameter_count > 0
        && !class.segments.last().is_some_and(|segment| {
            segment.arguments.is_empty() || segment.arguments.len() == expected.type_parameter_count
        })
    {
        return false;
    }
    true
}

fn signature_result_matches_member_creation(
    ty: &SignatureType,
    target: &crate::report::MemberInnerTargetInput,
) -> bool {
    if matches!(
        ty,
        SignatureType::Class(class)
            if class.segments.len() == 1
                && class.segments[0].binary_name.as_slice() == b"java/lang/Object"
                && class.segments[0].arguments.is_empty()
    ) {
        return true;
    }
    let SignatureType::Class(class) = ty else {
        return false;
    };
    let Some(outer) = target
        .source_type_path
        .iter()
        .find(|segment| segment.binary_name == target.outer)
    else {
        return false;
    };
    let Some(member) = target
        .source_type_path
        .iter()
        .find(|segment| segment.binary_name == target.owner)
    else {
        return false;
    };
    class.segments.len() == 2
        && class.segments[0].binary_name.as_slice() == outer.binary_name.0.as_slice()
        && class.segments[1].binary_name.as_slice() == member.binary_name.0.as_slice()
        && class.segments[0].arguments.len() == outer.type_parameter_count
        && class.segments[1].arguments.len() == member.type_parameter_count
        && !class.segments[0].arguments.is_empty()
}

pub fn ordinary_primitive_result_matches(
    physical: &jarde_java::ast::Type,
    parsed: Option<&SignatureType>,
    signature_spelling: Option<&str>,
    descriptor_spelling: Option<&str>,
) -> bool {
    let exact_base = matches!(
        (physical, parsed),
        (
            jarde_java::ast::Type::Boolean,
            Some(SignatureType::Base(b'Z'))
        ) | (jarde_java::ast::Type::Byte, Some(SignatureType::Base(b'B')))
            | (jarde_java::ast::Type::Char, Some(SignatureType::Base(b'C')))
            | (
                jarde_java::ast::Type::Short,
                Some(SignatureType::Base(b'S'))
            )
            | (jarde_java::ast::Type::Int, Some(SignatureType::Base(b'I')))
            | (jarde_java::ast::Type::Long, Some(SignatureType::Base(b'J')))
            | (
                jarde_java::ast::Type::Float,
                Some(SignatureType::Base(b'F'))
            )
            | (
                jarde_java::ast::Type::Double,
                Some(SignatureType::Base(b'D'))
            )
    );
    exact_base
        && signature_spelling == Some(physical.spell())
        && descriptor_spelling == Some(physical.spell())
}

#[allow(clippy::too_many_arguments)]
fn prove_body_generic_throws_shape(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    recovered: bool,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    budget: &mut Budget,
) -> Result<()> {
    let refused = |why| Error::unsupported("body_generic_throws_source_unproved", why);
    if !recovered
        || !matches!(descriptor, b"()V")
        || raw_name == b"<init>"
        || raw_name == b"<clinit>"
        || is_object_instance_method_name(raw_name)
        || access_flags & (ACC_ABSTRACT | ACC_NATIVE | ACC_VARARGS) != 0
        || class_internal.contains(&b'$')
        || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
        || !parsed.type_parameters.is_empty()
        || !parsed.parameters.is_empty()
        || parsed.result.is_some()
        || !matches!(parsed.throws.as_slice(), [SignatureType::TypeVariable(_)])
    {
        return Err(refused(
            "method Signature, member, or class hierarchy exceeds the supported empty-void shape",
        ));
    }
    if !matches!(
        candidate,
        Some(GenericReturnCandidate {
            parameters,
            value: GenericReturnValue::EmptyVoid,
                ..
        }) if parameters.is_empty()
    ) {
        return Err(refused(
            "same-run AST/Code/SSA does not prove the exact empty void body",
        ));
    }
    budget.poll()?;
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
    Ok(())
}

/// The initial no-body projection proves the class has no inherited source contract to satisfy.
/// `$` in a binary name is conservatively treated as nested because this projection receives no
/// selected InnerClasses nesting proof.
pub fn prove_no_body_generic_hierarchy(
    no_body_kind: Option<jarde_jvm::ir::NoBodyKind>,
    raw_name: &[u8],
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    budget: &mut Budget,
) -> Result<()> {
    let refused = |why| Error::unsupported("generic_inherited_contract_unproved", why);
    budget.poll()?;
    if !matches!(no_body_kind, Some(jarde_jvm::ir::NoBodyKind::Abstract))
        || class_flags & (ACC_ANNOTATION | ACC_ENUM) != 0
        || class_internal.contains(&b'$')
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
    {
        return Err(refused(
            "no-body generic projection requires a top-level class or root interface with Object as its physical superclass and no interfaces",
        ));
    }
    if is_object_instance_method_name(raw_name) {
        return Err(refused(
            "method name may override an Object instance method",
        ));
    }
    Ok(())
}

/// A method-local generic declaration with a physical abstract NoBody classification.
/// The caller has already proved erasure and called the hierarchy preflight at its original site.
pub fn no_body_generic_method_declaration(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    declaration_available: bool,
    no_body_kind: Option<jarde_jvm::ir::NoBodyKind>,
    has_default: bool,
    has_annotation_refusals: bool,
    has_type_annotations: bool,
    parameter_annotation_uses: &[Vec<String>],
    physical_throws: &[String],
    parsed: &jarde_reader::signature::MethodSignature,
    method_scope: &[TypeParameterErasure],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<String> {
    let refused = |why| Error::unsupported("generic_source_shape_unproved", why);
    if !declaration_available
        || !matches!(no_body_kind, Some(jarde_jvm::ir::NoBodyKind::Abstract))
        || matches!(raw_name, b"<init>" | b"<clinit>")
        || access_flags
            & !(ACC_PUBLIC
                | ACC_PRIVATE
                | ACC_PROTECTED
                | ACC_STATIC
                | ACC_FINAL
                | ACC_ABSTRACT
                | ACC_NATIVE
                | ACC_SYNCHRONIZED
                | ACC_STRICT
                | ACC_VARARGS)
            != 0
        || has_default
        || has_annotation_refusals
        || has_type_annotations
    {
        return Err(refused(
            "method flags, annotations, or source name cannot be preserved",
        ));
    }
    let mut scope = Vec::with_capacity(class_scope.len() + method_scope.len());
    scope.extend_from_slice(class_scope);
    scope.extend_from_slice(method_scope);
    let mut signature = method_descriptor(
        descriptor,
        access_flags & ACC_STATIC != 0,
        access_flags & ACC_VARARGS != 0,
    )
    .ok_or_else(|| refused("physical method descriptor cannot be spelled"))?;
    if signature.parameters.len() != parsed.parameters.len() {
        return Err(refused("generic and physical parameter positions differ"));
    }
    let mut parameters = Vec::with_capacity(parsed.parameters.len());
    for parameter in &parsed.parameters {
        parameters.push(spell_ordinary_signature_type(parameter, &scope, budget, 0)?);
    }
    let result = parsed
        .result
        .as_ref()
        .map(|result| spell_ordinary_signature_type(result, &scope, budget, 0))
        .transpose()?;
    let mut type_parameters = Vec::with_capacity(parsed.type_parameters.len());
    for parameter in &parsed.type_parameters {
        let mut bounds = Vec::new();
        for bound in parameter
            .class_bound
            .iter()
            .chain(parameter.interface_bounds.iter())
        {
            bounds.push(spell_ordinary_signature_type(bound, &scope, budget, 0)?);
        }
        let name = source_type_parameter_name(&parameter.name)
            .ok_or_else(|| refused("type parameter name has no faithful MUTF-8 Java spelling"))?;

        if bounds.is_empty() {
            bounds.push("java.lang.Object".to_owned());
        }
        type_parameters.push(format!("{name} extends {}", bounds.join(" & ")));
    }
    let mut generic_throws = Vec::with_capacity(parsed.throws.len());
    for exception in &parsed.throws {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        match exception {
            SignatureType::TypeVariable(variable) => {
                let Some(first_bound) = scope.iter().find(|entry| entry.name == *variable) else {
                    return Err(refused(
                        "throws variable is outside the emitted class and method scopes",
                    ));
                };
                if !matches!(
                    first_bound.descriptor.as_slice(),
                    b"Ljava/lang/Throwable;"
                        | b"Ljava/lang/Exception;"
                        | b"Ljava/lang/RuntimeException;"
                        | b"Ljava/lang/Error;"
                ) {
                    return Err(refused(
                        "throws variable first bound is not a proved JDK throwable root",
                    ));
                }
                generic_throws.push(
                    source_type_parameter_name(variable)
                        .ok_or_else(|| {
                            refused("throws variable name has no faithful MUTF-8 Java spelling")
                        })?
                        .to_owned(),
                );
            }
            SignatureType::Class(class) => {
                let [segment] = class.segments.as_slice() else {
                    return Err(refused(
                        "nested throws class has no preserved source position",
                    ));
                };
                if !segment.arguments.is_empty() {
                    return Err(refused(
                        "parameterized throws class is not a legal Java exception type",
                    ));
                }
                generic_throws.push(simple_generic_class_name(&segment.binary_name)?);
            }
            _ => return Err(refused("throws type has no supported Java source spelling")),
        }
    }
    let throws = if parsed.throws.is_empty() {
        physical_throws.to_vec()
    } else {
        generic_throws
    };
    for ((spelling, _), ty) in signature.parameters.iter_mut().zip(parameters) {
        *spelling = ty;
    }
    signature.returns = result;
    let name = source_identifier(raw_name)
        .ok_or_else(|| refused("method name has no faithful Java spelling"))?;
    Ok(format_slot_named_generic_method_header(
        access_flags,
        &signature,
        parameter_annotation_uses,
        &name,
        &type_parameters,
        &throws,
    ))
}

/// Descriptor slot names, parameter annotations and variable arity for a generic method head.
pub fn format_slot_named_generic_method_header(
    flags: u16,
    signature: &crate::declaration::Signature,
    parameter_annotation_uses: &[Vec<String>],
    name: &str,
    type_parameters: &[String],
    throws: &[String],
) -> String {
    let mut arguments = Vec::with_capacity(signature.parameters.len());
    for (position, (ty, slot)) in signature.parameters.iter().enumerate() {
        let annotations = parameter_annotation_uses
            .get(position)
            .into_iter()
            .flatten()
            .map(|annotation| format!("{annotation} "))
            .collect::<String>();
        let varargs = signature.varargs && position + 1 == signature.parameters.len();
        arguments.push(format!(
            "{annotations}{} arg{slot}",
            crate::declaration::varargs_type(ty, varargs)
        ));
    }
    format_generic_method_header(
        flags,
        type_parameters,
        signature.returns.as_deref().unwrap_or("void"),
        name,
        &arguments,
        throws,
    )
}

/// Source subdomain of a selected physical root/member family; no read or transaction.
pub fn supported_generic_member_family_header(
    root_signature: Option<&[u8]>,
    root_refused: bool,
    root_flags: u16,
    root_superclass: Option<&[u8]>,
    root_interfaces_empty: bool,
    root_internal: &[u8],
    root_scope: &[TypeParameterErasure],
    child_signature_present: bool,
    child_refused: bool,
) -> bool {
    root_signature.is_some_and(|signature| signature == b"<T:Ljava/lang/Object;>Ljava/lang/Object;")
        && !root_refused
        && root_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) == 0
        && root_superclass.is_some_and(|name| name == b"java/lang/Object")
        && root_interfaces_empty
        && root_internal.iter().all(|byte| *byte != b'$')
        && root_scope.len() == 1
        && root_scope[0].name == b"T"
        && root_scope[0].descriptor == b"Ljava/lang/Object;"
        && !child_signature_present
        && !child_refused
}

/// Source eligibility before the caller reads the physical Signature attribute.
pub fn prove_member_family_signature_source_slice(
    signature_count: usize,
    class_scope: &[TypeParameterErasure],
    marker_count: usize,
    generic_signature_refused: bool,
    has_annotation_attributes: bool,
) -> Result<()> {
    if signature_count != 1
        || class_scope.len() != 1
        || class_scope[0].descriptor != b"Ljava/lang/Object;"
        || marker_count != 1
        || !generic_signature_refused
        || has_annotation_attributes
    {
        return Err(Error::unsupported(
            "generic_member_family_signature_unproved",
            "family method is outside the one-variable unannotated Signature slice",
        ));
    }
    Ok(())
}

/// Preserve the family scope boundary after parsing and before physical erasure proof.
pub fn prove_member_family_method_scope(
    parsed: &jarde_reader::signature::MethodSignature,
) -> Result<()> {
    if !parsed.type_parameters.is_empty() {
        return Err(Error::unsupported(
            "generic_member_family_signature_unproved",
            "method-local type variables are outside the family scope",
        ));
    }
    Ok(())
}

/// Select and spell a family method after the caller proves relation, scope and erasure.
/// `scope_parameter` is the single Object-erased variable admitted by the pre-read gate.
pub fn member_family_generic_declaration(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    declaration_available: bool,
    recovered: bool,
    no_body: bool,
    has_no_body_kind: bool,
    has_code: bool,
    has_annotations: bool,
    has_source_refusals: bool,
    has_type_annotations: bool,
    parameter_annotation_uses: &[Vec<String>],
    has_default: bool,
    physical_throws: &[String],
    has_physical_throws: bool,
    parsed: &jarde_reader::signature::MethodSignature,
    candidate: Option<&GenericReturnCandidate>,
    body_proof: Option<&SameClassGenericMethodBodyProof>,
    proof_subject_matches: bool,
    class_flags: u16,
    class_internal: &[u8],
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    scope_parameter: &TypeParameterErasure,
    target: &crate::report::MemberInnerTargetInput,
    root_return: bool,
    budget: &mut Budget,
) -> Result<(String, &'static str)> {
    let refused = |why| Error::unsupported("generic_member_family_signature_unproved", why);
    let class_scope = std::slice::from_ref(scope_parameter);
    let declaration = if root_return {
        let SignatureType::Class(result) = parsed
            .result
            .as_ref()
            .ok_or_else(|| refused("root method has no generic result"))?
        else {
            return Err(refused("root result is not the selected member class"));
        };
        let [outer, inner] = result.segments.as_slice() else {
            return Err(refused(
                "root result does not carry the complete outer/member Signature path",
            ));
        };
        if class_internal != target.outer.0.as_slice()
            || raw_name != b"make"
            || descriptor != [b"()L".as_slice(), target.owner.0.as_slice(), b";"].concat()
            || !parsed.parameters.is_empty()
            || !parsed.throws.is_empty()
            || outer.binary_name != target.outer.0.as_slice()
            || outer.arguments.as_slice()
                != [TypeArgument::Exact(SignatureType::TypeVariable(
                    scope_parameter.name.clone(),
                ))]
            || inner.binary_name != target.owner.0.as_slice()
            || !inner.arguments.is_empty()
        {
            return Err(refused(
                "root method Signature is not the selected Outer<T>.Inner return",
            ));
        }
        let result = spell_ordinary_signature_type_with_member_path(
            parsed.result.as_ref().unwrap(),
            class_scope,
            &target.source_type_path,
            budget,
            0,
        )?;
        let mut signature = method_descriptor(descriptor, false, false)
            .ok_or_else(|| refused("root physical descriptor cannot be spelled"))?;
        signature.returns = Some(result);
        let name = source_identifier(raw_name)
            .ok_or_else(|| refused("root method name has no source spelling"))?;
        format_slot_named_generic_method_header(
            access_flags,
            &signature,
            parameter_annotation_uses,
            &name,
            &[],
            physical_throws,
        )
    } else {
        if raw_name != b"id"
            || descriptor != b"(Ljava/lang/Object;)Ljava/lang/Object;"
            || parsed.parameters.as_slice()
                != [SignatureType::TypeVariable(scope_parameter.name.clone())]
            || parsed.result.as_ref()
                != Some(&SignatureType::TypeVariable(scope_parameter.name.clone()))
            || !parsed.throws.is_empty()
        {
            return Err(refused(
                "child method Signature is not the inherited T id(T) shape",
            ));
        }
        let candidate = candidate.ok_or_else(|| {
            refused("child identity method has no retained same-run body candidate")
        })?;
        ordinary_parameterized_declaration(
            raw_name,
            descriptor,
            access_flags,
            declaration_available,
            recovered,
            no_body,
            has_no_body_kind,
            has_code,
            // The sole marker is the independent request's missing outer scope, already
            // admitted by the pre-read family gate. No temporary method clone is needed.
            true,
            has_annotations,
            has_source_refusals,
            has_type_annotations,
            parameter_annotation_uses,
            has_default,
            physical_throws,
            has_physical_throws,
            parsed,
            Some(candidate),
            body_proof,
            proof_subject_matches,
            class_flags,
            class_internal,
            class_superclass,
            class_interfaces,
            class_scope,
            false,
            &[],
            budget,
        )?
        .0
    };
    Ok((
        declaration,
        if root_return {
            "selected family return Signature and proved member construction"
        } else {
            "proved inherited outer-variable scope and same-run parameter-return proof"
        },
    ))
}

/// Select the class-scope constructor source strategy without retaining or reading new facts.
pub fn class_scope_constructor_is_candidate(
    raw_name: &[u8],
    parsed: &jarde_reader::signature::MethodSignature,
    class_signature_present: bool,
    class_scope: &[TypeParameterErasure],
    candidate: Option<&crate::report::GenericConstructorCandidate>,
    body_proof: Option<&SameClassGenericMethodBodyProof>,
) -> bool {
    raw_name == b"<init>"
        && parsed.type_parameters.is_empty()
        && class_signature_present
        && !class_scope.is_empty()
        && (candidate.is_some()
            || body_proof.is_some_and(|proof| {
                proof.constructor_call_bci.is_some() && proof.constructor_init.is_some()
            }))
}

/// Source admission and complete declaration over the selected constructor's same-run facts.
/// Physical subject equality is supplied by the facade and consumed at the fallback's old guard.
pub fn generic_constructor_declaration(
    raw_name: &[u8],
    descriptor: &[u8],
    access_flags: u16,
    declaration_available: bool,
    recovered: bool,
    markers_empty: bool,
    has_default: bool,
    has_physical_throws: bool,
    has_annotation_attributes: bool,
    has_annotation_refusals: bool,
    parsed: &jarde_reader::signature::MethodSignature,
    method_scope: &[TypeParameterErasure],
    candidate: Option<&crate::report::GenericConstructorCandidate>,
    body_proof: Option<&SameClassGenericMethodBodyProof>,
    proof_subject_matches: bool,
    physical_fields: &[MemberHeader],
    class_internal: &[u8],
    class_flags: u16,
    class_superclass: Option<&[u8]>,
    class_interfaces: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    class_signature_present: bool,
    budget: &mut Budget,
) -> Result<(String, &'static str)> {
    let refused = |why| Error::unsupported("generic_constructor_source_unproved", why);
    if raw_name != b"<init>"
        || !declaration_available
        || !recovered
        || !markers_empty
        || access_flags & !(ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED) != 0
        || has_default
        || has_physical_throws
        || !parsed.throws.is_empty()
        || parsed.result.is_some()
        || has_annotation_attributes
        || has_annotation_refusals
        || class_internal.contains(&b'$')
        || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
        || class_superclass != Some(b"java/lang/Object".as_slice())
        || !class_interfaces.is_empty()
    {
        return Err(refused(
            "constructor flags, throws, annotations, nesting, or class hierarchy lack a faithful source position",
        ));
    }
    let (parameters, init, forwarded_parameter_slots, field_writes) =
        if let Some(candidate) = candidate {
            (
                candidate.parameters.as_slice(),
                &candidate.init,
                candidate.forwarded_parameter_slots.as_slice(),
                candidate.field_writes.as_slice(),
            )
        } else {
            let proof = body_proof.filter(|proof| {
                proof_subject_matches
                    && proof.constructor_call_bci.is_some()
                    && proof
                        .constructor_init
                        .as_ref()
                        .is_some_and(|init| init.bci == proof.constructor_call_bci)
                    && parsed.type_parameters.is_empty()
                    && method_scope.is_empty()
                    && !class_scope.is_empty()
                    && class_signature_present
            });
            let proof = proof
                .ok_or_else(|| refused("same-run AST/SSA does not prove the constructor body"))?;
            (
                proof.formal_names.as_slice(),
                proof.constructor_init.as_ref().unwrap(),
                &[][..],
                &[][..],
            )
        };
    if !init.presented
        || init.target != Some(jarde_java::ast::ConstructorTarget::Super)
        || init.class.as_deref() != Some("java/lang/Object")
        || !init.declares_raw_class(class_internal)
    {
        return Err(refused(
            "same-run InitRecord does not prove Object() for this class",
        ));
    }
    let mut signature = method_descriptor(descriptor, false, false)
        .ok_or_else(|| refused("physical constructor descriptor cannot be spelled"))?;
    let class_scope_constructor = parsed.type_parameters.is_empty();
    if signature.parameters.len() != parsed.parameters.len()
        || signature.parameters.len() != parameters.len()
        || (class_scope_constructor
            && (!forwarded_parameter_slots.is_empty()
                || !method_scope.is_empty()
                || class_scope.is_empty()
                || !class_signature_present))
        || (!class_scope_constructor && !field_writes.is_empty())
    {
        return Err(refused(
            "constructor Signature, physical parameter positions, and same-run body proof differ",
        ));
    }
    let mut scope = Vec::with_capacity(class_scope.len() + method_scope.len());
    scope.extend_from_slice(method_scope);
    scope.extend_from_slice(class_scope);
    let mut type_parameters = Vec::with_capacity(parsed.type_parameters.len());
    for parameter in &parsed.type_parameters {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let name = source_type_parameter_name(&parameter.name).ok_or_else(|| {
            refused("constructor type parameter name has no faithful MUTF-8 Java spelling")
        })?;

        let mut bounds = Vec::new();
        for bound in parameter
            .class_bound
            .iter()
            .chain(parameter.interface_bounds.iter())
        {
            bounds.push(spell_ordinary_signature_type(bound, &scope, budget, 0)?);
        }
        if bounds.is_empty() {
            bounds.push("java.lang.Object".to_owned());
        }
        type_parameters.push(format!("{name} extends {}", bounds.join(" & ")));
    }
    let mut parameter_types = Vec::with_capacity(parsed.parameters.len());
    for parameter in &parsed.parameters {
        parameter_types.push(spell_ordinary_signature_type(parameter, &scope, budget, 0)?);
    }
    for ((_, slot), (candidate_slot, name)) in signature.parameters.iter().zip(parameters) {
        if slot != candidate_slot || !is_java_identifier(name) {
            return Err(refused(
                "same-run constructor parameter slot or name is unproved",
            ));
        }
    }
    for ((spelling, _), ty) in signature.parameters.iter_mut().zip(parameter_types) {
        *spelling = ty;
    }
    if class_scope_constructor {
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(parameters.len())
                .unwrap_or(u64::MAX)
                .saturating_add(
                    u64::try_from(field_writes.len())
                        .unwrap_or(u64::MAX)
                        .saturating_mul(u64::try_from(parameters.len()).unwrap_or(u64::MAX)),
                ),
        )?;
        for write in field_writes {
            budget.poll()?;
            let mut parameter_found = false;
            for (slot, _) in parameters {
                budget.poll()?;
                parameter_found |= *slot == write.parameter_slot;
            }
            if write.owner.0.as_slice() != class_internal || !parameter_found {
                return Err(refused(
                    "same-run field write does not name this class and a physical constructor parameter",
                ));
            }
            if !generic_constructor_field_matches_physical_field(
                write,
                class_internal,
                physical_fields,
                budget,
            )? {
                return Err(refused(
                    "same-run field write does not name one unique instance field declaration",
                ));
            }
        }
    }
    let simple = class_internal
        .rsplit(|byte| *byte == b'/')
        .next()
        .unwrap_or(class_internal);
    let class_name = crate::names::source_class_name(&JvmBytes(simple.to_vec()))
        .ok_or_else(|| refused("class name has no faithful constructor spelling"))?;
    if !is_source_identifier(&class_name) || class_name.contains('$') {
        return Err(refused("class name has no faithful constructor spelling"));
    }
    let args = parameters
        .iter()
        .zip(&signature.parameters)
        .map(|((_, name), (ty, _))| format!("{ty} {name}"))
        .collect::<Vec<_>>();
    let mut words = (if access_flags & ACC_PUBLIC != 0 {
        Some("public")
    } else if access_flags & ACC_PROTECTED != 0 {
        Some("protected")
    } else if access_flags & ACC_PRIVATE != 0 {
        Some("private")
    } else {
        None
    })
    .map(str::to_owned)
    .into_iter()
    .collect::<Vec<_>>();
    if !type_parameters.is_empty() {
        words.push(format!("<{}>", type_parameters.join(", ")));
    }
    let prefix = if words.is_empty() {
        String::new()
    } else {
        format!("{} ", words.join(" "))
    };
    let proof = if class_scope_constructor
        && candidate.is_some_and(|candidate| !candidate.field_writes.is_empty())
        || class_scope_constructor
            && body_proof.is_some_and(|proof| !proof.field_write_sites.is_empty())
    {
        "same-run AST/Code/SSA constructor body proof"
    } else if class_scope_constructor {
        "same-run AST/Code/SSA empty Object-constructor proof"
    } else {
        "same-run AST/SSA empty-constructor proof"
    };
    Ok((format!("{prefix}{class_name}({})", args.join(", ")), proof))
}

pub fn generic_constructor_field_matches_physical_field(
    write: &jarde_java::report::GenericConstructorFieldWrite,
    class_internal: &[u8],
    physical_fields: &[MemberHeader],
    budget: &mut Budget,
) -> Result<bool> {
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(physical_fields.len()).unwrap_or(u64::MAX),
    )?;
    if write.owner.0.as_slice() != class_internal {
        return Ok(false);
    }
    let mut matching = None;
    for field in physical_fields {
        budget.poll()?;
        if field.name.raw().0 == write.name.0.as_slice() {
            if matching.is_some() {
                return Ok(false);
            }
            matching = Some(field);
        }
    }
    Ok(matching.is_some_and(|field| {
        field.descriptor.raw().0 == write.descriptor.0.as_slice()
            && field.access_flags & ACC_STATIC == 0
    }))
}

/// Source eligibility after physical field erasure and same-class writer proof, before CP census.
pub fn prove_field_signature_source(
    declaration_available: bool,
    markers_empty: bool,
    raw_name: &[u8],
    flags: u16,
    class_internal: &[u8],
    class_header_projected: bool,
    class_flags: u16,
    has_type_annotation_attributes: bool,
    has_type_annotation_refusals: bool,
) -> Result<()> {
    if !declaration_available
        || !markers_empty
        || source_identifier(raw_name).is_none()
        || flags
            & !(ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED | ACC_STATIC | ACC_FINAL | 0x0040 | 0x0080)
            != 0
        || (class_internal.contains(&b'$') && !class_header_projected)
        || class_flags & (ACC_INTERFACE | ACC_ENUM | ACC_ANNOTATION) != 0
        || has_type_annotation_attributes
        || has_type_annotation_refusals
    {
        return Err(Error::unsupported(
            "field_generic_source_unproved",
            "field name, flags, descriptor, or type-use annotation path cannot be preserved",
        ));
    }
    Ok(())
}

/// Class Signature source shape, before any raw Signature publication or physical proof.
/// The tuple carries the existing parameterized-superclass, direct-parent and interface-only
/// decisions. Pass it only with the parsed Signature from which it was derived.
pub fn class_signature_source_shape(parsed: &ClassSignature) -> Option<(bool, bool, bool)> {
    let parameterized_superclass = parsed.type_parameters.is_empty()
        && parsed
            .superclass
            .segments
            .iter()
            .any(|segment| !segment.arguments.is_empty());
    let parameterized_interfaces = parsed.interfaces.iter().any(|interface| {
        interface
            .segments
            .iter()
            .any(|segment| !segment.arguments.is_empty())
    });
    let direct_parent_candidate = parsed.type_parameters.is_empty()
        && matches!(parsed.superclass.segments.as_slice(), [segment]
        if matches!(segment.arguments.as_slice(), [TypeArgument::Exact(SignatureType::Class(class))]
            if matches!(class.segments.as_slice(), [string]
                if string.binary_name == b"java/lang/String" && string.arguments.is_empty())));
    // The interface-only shape: no type parameters of its own and no parameterized
    // superclass, so the one generic position the class `Signature` states is an
    // `implements` entry. It is the fourth path below, and it also decides what the
    // shared gates mean for this class: they ask about the class's **own** header
    // position, and a class that entered the projection only through an interface keeps
    // the raw header they would refuse — the `Ok(None)` this shape returned before an
    // interface entry could enter the projection at all.
    let interface_only =
        parsed.type_parameters.is_empty() && !parameterized_superclass && parameterized_interfaces;
    if parsed.type_parameters.is_empty() && !parameterized_superclass && !parameterized_interfaces {
        return None;
    }
    Some((
        parameterized_superclass,
        direct_parent_candidate,
        interface_only,
    ))
}

/// The one `implements` argument shape whose inherited-member substitution is the identity: an
/// interface that declares its own type parameters and passes exactly those variables to its
/// parent interfaces — no concrete argument, no wildcard, no nested binary path, no method
/// variable.
///
/// A parent member reached through such an entry substitutes to the variable itself, and that
/// variable's erasure is the first bound `prove_class_signature_erasure` already proved from this
/// class's own header — the same erasure the class file's descriptors were compiled against. The
/// projection therefore adds a declaration and changes no JVM type on the inheritance path.
///
/// The kind test is a measured boundary, not a proof of safety. A parameterized parent obliges the
/// class to carry regenerated bridge members, and whether a physical `ACC_BRIDGE` member is
/// admitted and hidden is decided by the bridge admission, which runs *after* this projection and
/// is not visible here. Measured on the frozen cohort: publishing
/// `org/bouncycastle/jcajce/PKIXCRLStoreSelector implements Selector<T>` while its
/// `match(Object)` bridge stays un-admitted adds two javac errors (a name clash between
/// `match(Object)` and the inherited `match(T)`); the same bodies under the raw header keep the
/// former count and no clash. So a `class` refuses here. An `interface` was measured clean on the
/// same cohort (`PKIXCRLStore`, `PKIXCertStore`: both compile and reflect identically to the
/// original) — but javac 8 does emit `ACC_BRIDGE` members in interfaces, so an interface carrying
/// an un-admitted bridge can still reach the same clash. That residual risk is recorded in
/// `results/s2b-declared-generics-v1/inherited-proof-r1/README.md` as the next increment's
/// boundary rather than claimed away here.
///
/// Everything else keeps the separate-inherited-member refusal: a concrete argument needs the
/// argument-compatibility proof this shape does not carry, and a wildcard or a variable declared
/// elsewhere has no such erasure identity at all.
fn own_variable_interface_arguments(parsed: &ClassSignature, flags: u16) -> bool {
    if flags & ACC_INTERFACE == 0 {
        return false;
    }
    let mut carried = false;
    for interface in &parsed.interfaces {
        let [segment] = interface.segments.as_slice() else {
            return false;
        };
        for argument in &segment.arguments {
            let TypeArgument::Exact(SignatureType::TypeVariable(name)) = argument else {
                return false;
            };
            if !parsed
                .type_parameters
                .iter()
                .any(|parameter| parameter.name == *name)
            {
                return false;
            }
            carried = true;
        }
    }
    carried
}

/// An own-header failure preserves the physical header for an interface-only projection.
/// Other shapes retain the same explicit source refusal.
pub fn class_signature_own_header_refusal<T>(
    interface_only: bool,
    message: &str,
) -> Result<Option<T>> {
    if interface_only {
        Ok(None)
    } else {
        Err(Error::unsupported("class_generic_source_unproved", message))
    }
}

/// Source admission before nesting/erasure proofs; keep the original short-circuit order.
pub fn class_signature_source_header_is_eligible(
    name: &str,
    flags: u16,
    shells: &[jarde_reader::classfile::AttributeShell],
    interface_only: bool,
) -> Result<bool> {
    if flags & (ACC_ANNOTATION | ACC_ENUM) != 0
        || !is_source_identifier(name)
        || shells.iter().any(|shell| {
            matches!(
                shell.name.raw().0.as_slice(),
                b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
            )
        })
    {
        class_signature_own_header_refusal::<()>(
            interface_only,
            "class kind, nesting, name, or type-use annotations lack a faithful generic header position",
        )?;
        return Ok(false);
    }
    Ok(true)
}

/// Which class-header position one parent definition is proved for: the two positions
/// the class source projection asks the selected environment about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClassHeaderParentPosition {
    /// The direct superclass the relaxed direct-parent path projects (`extends Parent<String>`,
    /// whose parent name is the class file's own binary name).
    DirectSuperclass,
    /// One interface entry the class `Signature` spells with type arguments.
    Interface,
}

/// What one class-header parent's definition in the selected environment states about the
/// class's own `Signature` claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClassHeaderParentProof {
    /// The definition proves the claim this position makes: one single-parameter class parent,
    /// or one interface declaring exactly the number of type parameters the `Signature` spells
    /// as arguments.
    Proved,
    /// The selected environment does not provide the definition. The entry then keeps its
    /// physical spelling: an absent definition is not a claim the class file contradicts.
    Unresolved,
    /// The definition resolves and contradicts the claim — another kind, another arity, a
    /// header this read cannot state. The projection refuses rather than spelling a header from
    /// a contradicted claim.
    Contradicted,
}

/// 对已经由门面选定并读取的直接父类，证明现有单参数泛型父类源码域。
/// 只消费同次既有 facts/bytes；不选择定义、不读取 artifact、不提交类声明。
/// 名称复用严格 MUTF-8 源码出口；保持短路和 CP/Signature/erasure 的预算及错误传播顺序。
pub fn prove_selected_direct_generic_superclass(
    child: &PhysicalDefinitionId,
    definition: &PhysicalDefinitionId,
    parent_name: &[u8],
    facts: &ClassMemberFacts,
    bytes: &[u8],
    budget: &mut Budget,
) -> Result<bool> {
    if definition == child
        || facts.stopped_at.is_some()
        || facts.method_count != facts.methods.len() as u64
        || facts.field_count != facts.fields.len() as u64
        || facts.this_class.raw().0 != parent_name
        || crate::names::source_class_name(facts.this_class.raw()).is_none()
        || facts.access_flags & (ACC_INTERFACE | ACC_ANNOTATION | 0x4000) != 0
        || facts
            .super_class
            .as_ref()
            .is_none_or(|name| name.raw().0 != b"java/lang/Object")
        || !facts.interfaces.is_empty()
        || facts.attributes.iter().any(|attribute| {
            matches!(
                attribute.name.raw().0.as_slice(),
                b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
            )
        })
    {
        return Ok(false);
    }
    let signatures: Vec<_> = facts
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Signature")
        .cloned()
        .collect();
    if signatures.len() != 1 {
        return Ok(false);
    }
    let pool = class_constant_pool(bytes, budget)?;
    let Some(raw) = attribute_facts(bytes, &signatures, &pool, budget)?.signature else {
        return Ok(false);
    };
    let parsed = match jarde_reader::signature::parse_class_signature(&raw.0, budget) {
        Ok(parsed) => parsed,
        Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => return Err(error),
        Err(_) => return Ok(false),
    };
    let parameter_matches = matches!(parsed.type_parameters.as_slice(), [parameter]
        if source_type_parameter_name(&parameter.name).is_some()
        && matches!(parameter.class_bound.as_ref(), Some(jarde_reader::signature::SignatureType::Class(class))
            if matches!(class.segments.as_slice(), [segment]
                if segment.binary_name == b"java/lang/Object" && segment.arguments.is_empty()))
        && parameter.interface_bounds.is_empty());
    let object_super_matches = matches!(parsed.superclass.segments.as_slice(), [segment]
        if segment.binary_name == b"java/lang/Object" && segment.arguments.is_empty());
    if !parameter_matches || !object_super_matches || !parsed.interfaces.is_empty() {
        return Ok(false);
    }
    let physical_interfaces: Vec<Vec<u8>> = facts
        .interfaces
        .iter()
        .map(|name| name.raw().0.clone())
        .collect();
    let Some(parent_super) = facts.super_class.as_ref() else {
        return Ok(false);
    };
    let proof = match jarde_reader::signature::prove_class_signature_erasure(
        &parsed,
        &parent_super.raw().0,
        &physical_interfaces,
        budget,
    ) {
        Ok(proof) => proof,
        Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => return Err(error),
        Err(_) => return Ok(false),
    };
    if proof.type_parameters.len() != 1 {
        return Ok(false);
    }
    Ok(true)
}

/// 对已经由门面选定并读取的接口，核对现有源码种类与泛型参数数量。
/// 只消费同次既有 facts/bytes；不选择定义、不读取 artifact、不提交类声明。
/// 名称复用严格 MUTF-8 源码出口；保持短路和 CP/Signature 的预算及错误传播顺序。
pub fn prove_selected_header_interface(
    child: &PhysicalDefinitionId,
    definition: &PhysicalDefinitionId,
    interface_name: &[u8],
    argument_count: usize,
    facts: &ClassMemberFacts,
    bytes: &[u8],
    budget: &mut Budget,
) -> Result<class_source::ClassHeaderParentProof> {
    if definition == child
        || facts.stopped_at.is_some()
        || facts.method_count != facts.methods.len() as u64
        || facts.field_count != facts.fields.len() as u64
        || facts.this_class.raw().0 != interface_name
        || crate::names::source_class_name(facts.this_class.raw()).is_none()
        || facts.access_flags & ACC_INTERFACE == 0
        || facts.access_flags & (ACC_ANNOTATION | 0x4000) != 0
        || facts.attributes.iter().any(|attribute| {
            matches!(
                attribute.name.raw().0.as_slice(),
                b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
            )
        })
    {
        return Ok(class_source::ClassHeaderParentProof::Contradicted);
    }
    let signatures: Vec<_> = facts
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Signature")
        .cloned()
        .collect();
    if signatures.len() != 1 {
        return Ok(class_source::ClassHeaderParentProof::Contradicted);
    }
    let pool = class_constant_pool(bytes, budget)?;
    let Some(raw) = attribute_facts(bytes, &signatures, &pool, budget)?.signature else {
        return Ok(class_source::ClassHeaderParentProof::Contradicted);
    };
    let parsed = match jarde_reader::signature::parse_class_signature(&raw.0, budget) {
        Ok(parsed) => parsed,
        Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => return Err(error),
        Err(_) => return Ok(class_source::ClassHeaderParentProof::Contradicted),
    };
    if parsed.type_parameters.len() != argument_count {
        return Ok(class_source::ClassHeaderParentProof::Contradicted);
    }
    Ok(class_source::ClassHeaderParentProof::Proved)
}

/// Complete post-erasure class Signature source policy. The caller supplies the shape from
/// `class_signature_source_shape` for this parsed Signature and performs the physical proofs.
/// Parent definition selection remains in the supplied, budgeted callback. No class report is
/// published here; the caller charges the final output and commits the complete tuple atomically.
pub fn project_class_signature_source_header(
    parsed: ClassSignature,
    proof: &ClassSignatureErasureProof,
    (parameterized_superclass, direct_parent_candidate, interface_only): (bool, bool, bool),
    name: &str,
    this_class: &JvmString,
    flags: u16,
    physical_super: &JvmString,
    physical_interface_names: &[JvmString],
    physical_interfaces: &[Vec<u8>],
    budget: &mut Budget,
    mut prove_header_parent: impl FnMut(
        ClassHeaderParentPosition,
        &[u8],
        usize,
        &mut Budget,
    ) -> Result<ClassHeaderParentProof>,
) -> Result<Option<(String, Option<String>, Vec<Vec<u8>>)>> {
    let mut header_interface_arguments: Vec<Vec<u8>> = Vec::new();
    let mut projected_interfaces: Option<Vec<String>> = None;
    if direct_parent_candidate {
        if !parsed.interfaces.is_empty() || !physical_interfaces.is_empty() {
            return Err(Error::unsupported(
                "class_generic_source_unproved",
                "direct parameterized superclass projection does not include interfaces",
            ));
        }
        let [parent] = parsed.superclass.segments.as_slice() else {
            unreachable!("direct parent candidate has exactly one segment")
        };
        // The name proved here is the class file's own binary name (`$` kept): the raw
        // header already spells every parent that way, so the projected entry adds the
        // type arguments and changes no name.
        match prove_header_parent(
            ClassHeaderParentPosition::DirectSuperclass,
            &parent.binary_name,
            parent.arguments.len(),
            budget,
        )? {
            ClassHeaderParentProof::Proved => {}
            ClassHeaderParentProof::Unresolved | ClassHeaderParentProof::Contradicted => {
                return Err(Error::unsupported(
                    "class_generic_source_unproved",
                    "direct superclass does not resolve to one proved single-parameter parent definition",
                ));
            }
        }
    } else if parameterized_superclass {
        return Err(Error::unsupported(
            "class_generic_source_unproved",
            "only a single direct Parent<String> superclass is supported for a class without type parameters",
        ));
    } else if !matches!(parsed.superclass.segments.as_slice(), [segment] if segment.arguments.is_empty()) {
        // A superclass path the header cannot spell from the `Signature` (a nested
        // binary path with no arguments of its own). Every shape whose own type
        // parameters or superclass entered the projection refuses here, exactly as
        // before; the interface-only shape keeps the physical header, which is the
        // whole projection its raw entries would spell anyway.
        return class_signature_own_header_refusal(
            interface_only,
            "parameterized or nested parent needs a separate inherited-member proof",
        );
    } else if interface_only {
        // Each interface entry is spelled with its type arguments only when the selected
        // environment's definition proves the arity the class `Signature` states (and
        // then exactly the entries whose header carries arguments are recorded for the
        // bridge admission). Every other entry keeps the physical spelling the raw
        // header states, so a class this path cannot prove is byte-identical to the
        // header it had before the path existed.
        let mut header_interfaces = Vec::with_capacity(parsed.interfaces.len());
        for (interface, physical) in parsed.interfaces.iter().zip(physical_interfaces) {
            match interface.segments.as_slice() {
                [segment] if !segment.arguments.is_empty() => {
                    match prove_header_parent(
                        ClassHeaderParentPosition::Interface,
                        &segment.binary_name,
                        segment.arguments.len(),
                        budget,
                    )? {
                        ClassHeaderParentProof::Proved => {
                            header_interfaces.push(spell_class_header_signature_type(
                                &SignatureType::Class(interface.clone()),
                                &proof.type_parameters,
                                budget,
                                0,
                            )?);
                            header_interface_arguments.push(physical.clone());
                        }
                        ClassHeaderParentProof::Unresolved => {
                            header_interfaces.push(crate::names::source_class_name(&JvmBytes(physical.clone())).ok_or_else(|| Error::unsupported("class_generic_source_unproved", "interface has no faithful source name"))?);
                        }
                        ClassHeaderParentProof::Contradicted => {
                            return Err(Error::unsupported(
                                "class_generic_source_unproved",
                                "an interface definition contradicts the class Signature's type arguments",
                            ));
                        }
                    }
                }
                _ => header_interfaces.push(crate::names::source_class_name(&JvmBytes(physical.clone())).ok_or_else(|| Error::unsupported("class_generic_source_unproved", "interface has no faithful source name"))?),
            }
        }
        if header_interface_arguments.is_empty() {
            // No entry carried its arguments, so this path publishes nothing: the class
            // keeps the physical declaration it had, with no projection claim and no
            // note. (An interface-only class whose definitions are all absent is the
            // whole pre-change state of every such class.)
            return Ok(None);
        }
        projected_interfaces = Some(header_interfaces);
    } else if own_variable_interface_arguments(&parsed, flags) {
        // This class declares the variables its parent interfaces are handed. The parent's
        // member types then substitute to the variable itself, whose erasure
        // `prove_class_signature_erasure` already proved from this header, so the projected
        // entry states the same JVM inheritance the raw header did — with a variable where the
        // raw header had nothing. The entry is still spelled only for a definition the caller
        // proved at the claimed arity; an absent definition is not a claim the class file
        // contradicts, but it is also not a proof, so the whole header keeps the refusal it
        // had and no variable is published on a hope.
        let mut header_interfaces = Vec::with_capacity(parsed.interfaces.len());
        for (interface, physical) in parsed.interfaces.iter().zip(physical_interfaces) {
            let [segment] = interface.segments.as_slice() else {
                unreachable!("own-variable interface entries are single-segment")
            };
            match segment.arguments.is_empty() {
                false => match prove_header_parent(
                    ClassHeaderParentPosition::Interface,
                    &segment.binary_name,
                    segment.arguments.len(),
                    budget,
                )? {
                    ClassHeaderParentProof::Proved => {
                        header_interfaces.push(spell_class_header_signature_type(
                            &SignatureType::Class(interface.clone()),
                            &proof.type_parameters,
                            budget,
                            0,
                        )?);
                        header_interface_arguments.push(physical.clone());
                    }
                    ClassHeaderParentProof::Unresolved
                    | ClassHeaderParentProof::Contradicted => {
                        return Err(Error::unsupported(
                            "class_generic_source_unproved",
                            "parameterized or nested parent needs a separate inherited-member proof",
                        ));
                    }
                },
                true => header_interfaces.push(
                    crate::names::source_class_name(&JvmBytes(physical.clone())).ok_or_else(
                        || {
                            Error::unsupported(
                                "class_generic_source_unproved",
                                "interface has no faithful source name",
                            )
                        },
                    )?,
                ),
            }
        }
        projected_interfaces = Some(header_interfaces);
    } else if parsed.interfaces.iter().any(|interface| {
        !matches!(interface.segments.as_slice(), [segment] if segment.arguments.is_empty())
    }) {
        return Err(Error::unsupported(
            "class_generic_source_unproved",
            "parameterized or nested parent needs a separate inherited-member proof",
        ));
    }
    let mut parameters = Vec::with_capacity(parsed.type_parameters.len());
    for parameter in &parsed.type_parameters {
        let name = source_type_parameter_name(&parameter.name).ok_or_else(|| {
            Error::unsupported(
                "class_generic_source_unproved",
                "type variable name has no faithful MUTF-8 Java spelling",
            )
        })?;

        let mut bounds = Vec::new();
        for bound in parameter
            .class_bound
            .iter()
            .chain(&parameter.interface_bounds)
        {
            if !matches!(
                bound,
                SignatureType::Class(_) | SignatureType::TypeVariable(_)
            ) {
                return Err(Error::unsupported(
                    "class_generic_source_unproved",
                    "type-variable bound is not a Java class or type variable",
                ));
            }
            bounds.push(spell_ordinary_signature_type(
                bound,
                &proof.type_parameters,
                budget,
                0,
            )?);
        }
        let omit_object = bounds.len() == 1
            && bounds[0] == "java.lang.Object"
            && parameter.interface_bounds.is_empty();
        parameters.push(if bounds.is_empty() || omit_object {
            name.to_owned()
        } else {
            format!("{name} extends {}", bounds.join(" & "))
        });
    }
    let (superclass, interfaces) = match projected_interfaces {
        // The interface-only path spelled every entry itself: a projected entry keeps
        // its type arguments, every other entry keeps the physical name — and the bare
        // superclass keeps the physical name too, exactly the raw header's own spelling.
        Some(interfaces) => (
            crate::names::source_class_name(physical_super.raw()).ok_or_else(|| {
                Error::unsupported(
                    "class_generic_source_unproved",
                    "superclass has no faithful source name",
                )
            })?,
            interfaces,
        ),
        None => {
            // The relaxed direct-parent path publishes a superclass whose binary name
            // may carry `$` (the raw header's own spelling); every other shape keeps
            // the member/annotation spelling, which a `$` name without a selected
            // source path has none of.
            let superclass = if direct_parent_candidate {
                spell_class_header_signature_type(
                    &SignatureType::Class(parsed.superclass),
                    &proof.type_parameters,
                    budget,
                    0,
                )?
            } else {
                spell_ordinary_signature_type(
                    &SignatureType::Class(parsed.superclass),
                    &proof.type_parameters,
                    budget,
                    0,
                )?
            };
            let mut interfaces = Vec::with_capacity(parsed.interfaces.len());
            for interface in parsed.interfaces {
                interfaces.push(spell_ordinary_signature_type(
                    &SignatureType::Class(interface),
                    &proof.type_parameters,
                    budget,
                    0,
                )?);
            }
            (superclass, interfaces)
        }
    };
    let type_parameters = (!parameters.is_empty()).then(|| parameters.join(", "));
    let declaration = crate::declaration::class_declaration_with_types(
        name,
        this_class,
        flags,
        Some(physical_super),
        physical_interface_names,
        type_parameters.as_deref(),
        Some(&superclass),
        Some(&interfaces),
    )
    .ok_or_else(|| {
        Error::unsupported(
            "class_generic_source_unproved",
            "class header has no faithful source name",
        )
    })?;
    Ok(Some((
        declaration,
        type_parameters,
        header_interface_arguments,
    )))
}
