//! The private class-level proof for the narrow Java 8 enum source projection.
//!
//! A candidate here is copied from the `MethodIr` of the class-source member run that already
//! decoded it. It is not decoded again, printed, serialized, or retained for non-candidate method
//! names. The table index binds it to the physical method record; the identity is checked again
//! before any conclusion is made from it.

use crate::class_source::{
    ClassSourceDeclaration, ClassSourceField, ClassSourceMethod, ClassSourceOutcome,
};
use crate::{Budget, CountedBudgetDimension, Result};
use jarde_reader::classfile::MemberHeader;
#[cfg(test)]
use jarde_reader::model::ExecutionReport;

// Java owns the existing same-run enum Code facts and their pure capture/API proof stage.
#[cfg(test)]
use jarde_java::source_proof::enum_instruction;
pub(crate) use jarde_java::source_proof::{
    EnumCodeInstruction, EnumCodeReference, EnumMethodCodeCandidate, ProvedEnumUserConstructorTail,
    capture_method_code, may_capture_group_code,
};
#[cfg(test)]
use jarde_java::source_proof::{
    ProvedEnumIntArgument, ProvedEnumSourceArgument, parse_arbitrary_ctor_descriptor,
};
#[cfg(test)]
use jarde_reader::classfile::CpEntryKind;

use jarde_java::source_proof::unique_method;
#[cfg(test)]
use jarde_java::source_proof::{
    EnumCodeUse, EnumIntUserParameter, EnumSourceArgument, InitializerConstructorCall,
    InitializerPrefixInput, ProvedEnumStringArgument, STRING_CTOR_DESCRIPTOR,
    STRING_VARARGS_CTOR_DESCRIPTOR, array_descriptor, narrowed_int_user_text, object_descriptor,
    prove_initializer_prefix, prove_string_source_argument, prove_values_factory,
    value_of_descriptor, values_descriptor,
};
pub(crate) use jarde_java::source_proof::{EnumUserParameter, ProvedEnumUserArgument};

pub(crate) use jarde_java::source_proof::{
    CTOR_DESCRIPTOR, DELEGATING_CTOR_DESCRIPTOR, ProvedEnumConstructorBody,
    ProvedEnumConstructorDelegationEdge,
};

pub(crate) use jarde_java::source_proof::{
    ProvedEnumConstant, ProvedEnumMapSuffix, ProvedEnumStaticAssignment,
    ProvedOrdinaryEnumConstantGroup,
};

#[cfg(test)]
const ACC_ENUM: u16 = 0x4000;
#[cfg(test)]
const ACC_ABSTRACT: u16 = 0x0400;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProvedEnumConstantGroup {
    Ordinary(ProvedOrdinaryEnumConstantGroup),
    Body(ProvedEnumConstantBodyGroup),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvedEnumConstantBodyGroup {
    pub(crate) constants: Vec<ProvedEnumBodyConstant>,
    /// Source constructor recovered from the exact single-String assignment proof.
    pub(crate) constructor_text: Option<String>,
    /// The proved user tail of the shared source constructor, present exactly when the constant
    /// arguments came through the arbitrary grammar and the constructor stores each parameter
    /// into its own instance field.
    pub(crate) constructor_user_tail: Option<ProvedEnumUserConstructorTail>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvedEnumBodyConstant {
    pub(crate) field_index: u64,
    pub(crate) allocation_bci: u32,
    pub(crate) constructor_bci: u32,
    pub(crate) field_write_bci: u32,
    /// The proved user argument tail of this constant's constructor step, empty exactly when the
    /// constructor takes no user arguments. Spellings follow the arbitrary-argument channel.
    pub(crate) source_arguments: Vec<ProvedEnumUserArgument>,
    pub(crate) subclass: Option<jarde_reader::model::PhysicalDefinitionId>,
    /// The selected child's already-recovered source-visible members. A direct constant has no
    /// child and no member sidecar; sharing this slice does not run recovery or copy reports.
    pub(crate) methods: Option<std::sync::Arc<[ClassSourceMethod]>>,
}

/// No public JSON conclusion is made from this proof. The state stays in the class-source
/// assembler for the next projection task and for focused in-crate tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ClassSourceEnumConstantProof {
    NotApplicable,
    Proved(ProvedEnumConstantGroup),
    Refused { reason: String },
    Stopped { reason: String },
}

pub(crate) fn has_only_terminal_initializer_return(
    group: &ProvedOrdinaryEnumConstantGroup,
    code_candidates: &[EnumMethodCodeCandidate],
    initializer_candidates: &[jarde_java::report::ClassInitializerCandidates],
    methods: &[ClassSourceMethod],
) -> bool {
    jarde_java::source_proof::has_only_terminal_initializer_return(
        group,
        code_candidates,
        initializer_candidates,
        |index| {
            methods
                .get(index)
                .map(|method| method.enum_constructor_source_tail)
        },
        |index| methods.get(index).map(|method| &method.item.identity),
    )
}

pub(crate) struct EnumStaticAssignmentInput<'a> {
    pub(crate) group: &'a ProvedOrdinaryEnumConstantGroup,
    pub(crate) owner: &'a [u8],
    pub(crate) field_headers: &'a [MemberHeader],
    pub(crate) source_fields: &'a [ClassSourceField],
    pub(crate) method_headers: &'a [MemberHeader],
    pub(crate) source_methods: &'a [ClassSourceMethod],
    pub(crate) code_candidates: &'a [EnumMethodCodeCandidate],
    pub(crate) initializer_candidates: &'a [jarde_java::report::ClassInitializerCandidates],
    pub(crate) budget: &'a mut Budget,
}

/// Supply the same-run facts for Java's own-static-int-call assignment proof.
/// Other suffix operations keep the original `<clinit>` and physical enum members.
pub(crate) fn prove_static_assignment_suffix(
    input: EnumStaticAssignmentInput<'_>,
) -> Result<Option<ProvedEnumStaticAssignment>> {
    let EnumStaticAssignmentInput {
        group,
        owner,
        field_headers,
        source_fields,
        method_headers,
        source_methods,
        code_candidates,
        initializer_candidates,
        budget,
    } = input;
    jarde_java::source_proof::prove_static_assignment_suffix(
        group,
        owner,
        field_headers,
        method_headers,
        source_fields.len(),
        source_methods.len(),
        code_candidates,
        initializer_candidates,
        |index, header| {
            source_fields.get(index).map(|field| {
                (
                    &field.item.identity.owner,
                    field.item.index == u64::try_from(index).unwrap_or(u64::MAX)
                        && field.item.name.raw() == header.name.raw()
                        && field.item.descriptor.raw() == header.descriptor.raw()
                        && field.item.access_flags == header.access_flags,
                    field.declaration.is_some(),
                    field.markers.is_empty(),
                )
            })
        },
        |index| {
            source_methods.get(index).map(|method| {
                (
                    method.item.index,
                    &method.item.identity,
                    method.declaration.is_some(),
                    match &method.outcome {
                        ClassSourceOutcome::Recovered { report, analysis } => {
                            Some((report.as_ref(), &analysis.execution))
                        }
                        _ => None,
                    },
                )
            })
        },
        budget,
    )
}

pub(crate) fn prove_map_initializer_suffix(
    input: EnumStaticAssignmentInput<'_>,
) -> Result<Option<ProvedEnumMapSuffix>> {
    let EnumStaticAssignmentInput {
        group,
        owner,
        field_headers,
        source_fields,
        method_headers,
        source_methods,
        code_candidates,
        initializer_candidates,
        budget,
    } = input;
    jarde_java::source_proof::prove_map_initializer_suffix(
        group,
        owner,
        field_headers,
        method_headers,
        source_fields.len(),
        source_methods.len(),
        code_candidates,
        initializer_candidates,
        |index| {
            source_fields.get(index).map(|field| {
                (
                    field.item.index,
                    &field.item.identity.owner,
                    field.declaration.as_deref(),
                    field.markers.as_slice(),
                    field.annotations.refusals.is_empty(),
                    field.type_annotations.refusals.is_empty(),
                )
            })
        },
        |index| {
            source_methods.get(index).map(|method| {
                (
                    method.item.index,
                    &method.item.identity,
                    method.declaration.is_some(),
                    match &method.outcome {
                        ClassSourceOutcome::Recovered { report, analysis } => {
                            Some((report.as_ref(), &analysis.execution))
                        }
                        _ => None,
                    },
                )
            })
        },
        budget,
    )
}

/// Prove the complete two-constant Java 8 enum pattern from one class read and its same-run
/// candidates. A refusal is a normal result; budget/cancellation errors remain errors so the
/// caller can publish them in the request's existing execution plane.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prove_group(
    declaration: &ClassSourceDeclaration,
    class_version: (u16, u16),
    field_headers: &[MemberHeader],
    field_count: u64,
    fields_complete: bool,
    source_fields: &[ClassSourceField],
    method_headers: &[MemberHeader],
    method_count: u64,
    methods_complete: bool,
    source_methods: &[ClassSourceMethod],
    code_candidates: &[EnumMethodCodeCandidate],
    initializer_candidates: &[jarde_java::report::ClassInitializerCandidates],
    constructor_candidates: &[jarde_java::report::ClassEnumConstructorCandidates],
    budget: &mut Budget,
) -> Result<ClassSourceEnumConstantProof> {
    let class = &declaration.item.declaration;
    let result = jarde_java::source_proof::prove_ordinary_enum_group(
        class_version,
        class.access_flags,
        class.this_class.raw().0.as_slice(),
        class
            .super_class
            .as_ref()
            .map(|name| name.raw().0.as_slice()),
        field_headers,
        method_headers,
        initializer_candidates,
        constructor_candidates,
        |budget| {
            let refuse = |reason: &str| Err(reason.to_owned());
            if declaration.item.definition
                != source_fields
                    .first()
                    .map(|field| field.item.identity.owner.clone())
                    .unwrap_or_else(|| declaration.item.definition.clone())
            {
                return Ok(refuse(
                    "the class-source declaration and physical member owner disagree",
                ));
            }

            if !fields_complete
                || !methods_complete
                || u64::try_from(field_headers.len()).unwrap_or(u64::MAX) != field_count
                || source_fields.len() != field_headers.len()
                || u64::try_from(method_headers.len()).unwrap_or(u64::MAX) != method_count
                || source_methods.len() != method_headers.len()
            {
                return Ok(refuse("the physical field or method table is incomplete"));
            }
            let work = u64::try_from(field_headers.len())
                .unwrap_or(u64::MAX)
                .saturating_add(u64::try_from(method_headers.len()).unwrap_or(u64::MAX));
            budget.charge(CountedBudgetDimension::IrItems, work)?;

            for (index, (header, source)) in field_headers.iter().zip(source_fields).enumerate() {
                if source.item.index != u64::try_from(index).unwrap_or(u64::MAX)
                    || source.item.name.raw() != header.name.raw()
                    || source.item.descriptor.raw() != header.descriptor.raw()
                    || source.item.access_flags != header.access_flags
                    || source.item.identity.owner != declaration.item.definition
                {
                    return Ok(refuse(
                        "a published field does not match its physical table record",
                    ));
                }
            }
            for (index, (header, source)) in method_headers.iter().zip(source_methods).enumerate() {
                if source.item.index != u64::try_from(index).unwrap_or(u64::MAX)
                    || source.item.name.raw() != header.name.raw()
                    || source.item.descriptor.raw() != header.descriptor.raw()
                    || source.item.access_flags != header.access_flags
                    || source.item.identity.owner != declaration.item.definition
                {
                    return Ok(refuse(
                        "a published method does not match its physical table record",
                    ));
                }
            }

            Ok(Ok(()))
        },
        |index| method_code(index, method_headers, source_methods, code_candidates),
        |index| {
            source_fields
                .get(index)
                .is_some_and(|source| source.declaration.is_some())
        },
        |index| {
            let source = &source_methods[index];
            (
                source.enum_constructor_source_tail,
                source.enum_constructor_signature_erasure_refused,
                source.markers.len(),
            )
        },
        |index| &source_methods[index].item.identity,
        |constants, backing_name, budget| {
            prove_constructor_delegation_edge(
                DelegationEdgeInput {
                    owner: class.this_class.raw().0.as_slice(),
                    constants,
                    backing_name,
                    field_headers,
                    method_headers,
                    source_methods,
                    code_candidates,
                },
                budget,
            )
        },
        |method_index, budget| {
            prove_terminal_constructor_body(
                TerminalConstructorBodyInput {
                    owner: class.this_class.raw().0.as_slice(),
                    method_index,
                    field_headers,
                    source_fields,
                    method_headers,
                    source_methods,
                    code_candidates,
                    constructor_candidates,
                },
                budget,
            )
        },
        budget,
    )?;
    Ok(match result {
        Ok(None) => ClassSourceEnumConstantProof::NotApplicable,
        Ok(Some(group)) => {
            ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(group))
        }
        Err(reason) => ClassSourceEnumConstantProof::Refused { reason },
    })
}

fn method_code<'a>(
    index: usize,
    headers: &[MemberHeader],
    source_methods: &[ClassSourceMethod],
    candidates: &'a [EnumMethodCodeCandidate],
) -> std::result::Result<&'a EnumMethodCodeCandidate, String> {
    let matching: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.table_index == u64::try_from(index).unwrap_or(u64::MAX))
        .collect();
    let [candidate] = matching.as_slice() else {
        return Err(format!(
            "the same-run Code sidecar for physical method table index {index} is absent or ambiguous"
        ));
    };
    let source = source_methods.get(index).ok_or_else(|| {
        format!("the class-source method record at physical table index {index} is absent")
    })?;
    let header = headers
        .get(index)
        .ok_or_else(|| format!("the method header at physical table index {index} is absent"))?;
    if candidate.member.as_ref() != Some(&source.item.identity)
        || source.item.name.raw() != header.name.raw()
        || source.item.descriptor.raw() != header.descriptor.raw()
        || candidate.member.as_ref().is_none_or(|member| {
            member.name.0 != header.name.raw().0 || member.descriptor.0 != header.descriptor.raw().0
        })
    {
        return Err(format!(
            "the Code sidecar at physical method table index {index} names a different member"
        ));
    }
    Ok(candidate)
}

/// The physical and same-run facts needed to prove one bounded constructor edge.
struct DelegationEdgeInput<'a> {
    owner: &'a [u8],
    constants: &'a [(usize, String, i32)],
    backing_name: &'a [u8],
    field_headers: &'a [MemberHeader],
    method_headers: &'a [MemberHeader],
    source_methods: &'a [ClassSourceMethod],
    code_candidates: &'a [EnumMethodCodeCandidate],
}

struct TerminalConstructorBodyInput<'a> {
    owner: &'a [u8],
    method_index: usize,
    field_headers: &'a [MemberHeader],
    source_fields: &'a [ClassSourceField],
    method_headers: &'a [MemberHeader],
    source_methods: &'a [ClassSourceMethod],
    code_candidates: &'a [EnumMethodCodeCandidate],
    constructor_candidates: &'a [jarde_java::report::ClassEnumConstructorCandidates],
}

/// Prove the bounded javac enum constructor edge from candidates captured during the current
/// class-source member run. No method is decoded or analyzed here: the only inputs are the physical
/// tables and same-run Code candidates already collected by `capture_method_code`.
fn prove_constructor_delegation_edge(
    input: DelegationEdgeInput<'_>,
    budget: &mut Budget,
) -> Result<std::result::Result<ProvedEnumConstructorDelegationEdge, String>> {
    let DelegationEdgeInput {
        owner,
        constants,
        backing_name,
        field_headers,
        method_headers,
        source_methods,
        code_candidates,
    } = input;
    jarde_java::source_proof::prove_constructor_delegation_edge(
        owner,
        constants,
        backing_name,
        field_headers,
        method_headers,
        code_candidates.len(),
        |index| method_code(index, method_headers, source_methods, code_candidates),
        budget,
    )
}

/// Prove the terminal constructor's complete, ordered user body from same-run AST and Code facts.
/// The physical method remains refused by generic Signature erasure and this private result does
/// not authorize source projection on its own.
fn prove_terminal_constructor_body(
    input: TerminalConstructorBodyInput<'_>,
    budget: &mut Budget,
) -> Result<std::result::Result<ProvedEnumConstructorBody, String>> {
    let TerminalConstructorBodyInput {
        owner,
        method_index,
        field_headers,
        source_fields,
        method_headers,
        source_methods,
        code_candidates,
        constructor_candidates,
    } = input;
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(field_headers.len())
            .unwrap_or(u64::MAX)
            .saturating_add(u64::try_from(method_headers.len()).unwrap_or(u64::MAX))
            .saturating_add(u64::try_from(code_candidates.len()).unwrap_or(u64::MAX))
            .saturating_add(u64::try_from(constructor_candidates.len()).unwrap_or(u64::MAX)),
    )?;
    budget.poll()?;

    let Some(header) = method_headers.get(method_index) else {
        return refuse("the edge's terminal method index is outside the physical method table");
    };
    let Some(source) = source_methods.get(method_index) else {
        return refuse("the terminal constructor has no class-source member record");
    };
    let Ok(method_index_u64) = u64::try_from(method_index) else {
        return refuse("the terminal constructor index does not fit its physical identity");
    };
    if source.item.index != method_index_u64
        || source.item.name.raw().0 != b"<init>"
        || source.item.descriptor.raw().0 != CTOR_DESCRIPTOR
        || header.name.raw().0 != b"<init>"
        || header.descriptor.raw().0 != CTOR_DESCRIPTOR
        || !jarde_java::source_proof::terminal_enum_constructor_source_applicable(
            header,
            source.enum_constructor_source_tail,
            source.enum_constructor_signature_erasure_refused,
            source.markers.len(),
            match &source.outcome {
                ClassSourceOutcome::Recovered { report, analysis } => {
                    Some((report.as_ref(), &analysis.execution))
                }
                _ => None,
            },
        )
    {
        return refuse(
            "the terminal constructor identity, source Signature, refusal marker, or body outcome is not exact",
        );
    }
    let delegating_index =
        match unique_method(method_headers, b"<init>", DELEGATING_CTOR_DESCRIPTOR) {
            Ok(index) => index,
            Err(reason) => return refuse(&reason),
        };
    let Some(delegating_header) = method_headers.get(delegating_index) else {
        return refuse("the no-source-argument constructor is absent from the physical table");
    };
    let Some(delegating_source) = source_methods.get(delegating_index) else {
        return refuse("the no-source-argument constructor has no class-source record");
    };
    if !enum_constructor_signature_matches(
        delegating_header,
        delegating_source,
        DELEGATING_CTOR_DESCRIPTOR,
    ) {
        return refuse("the no-source-argument constructor does not have the exact ()V Signature");
    }

    let matching_code: Vec<_> = code_candidates
        .iter()
        .filter(|candidate| candidate.table_index == method_index_u64)
        .collect();
    let [code] = matching_code.as_slice() else {
        return refuse("the terminal constructor Code candidate is absent or ambiguous");
    };
    if code.member.as_ref() != Some(&source.item.identity) {
        return refuse("the terminal constructor Code stream is partial, handled, or not exact");
    }
    jarde_java::source_proof::prove_terminal_constructor_body(
        owner,
        method_index,
        field_headers,
        code,
        constructor_candidates,
        &source.item.identity,
        |index, name| {
            source_fields.get(index).map(|source| {
                (
                    source.item.index == u64::try_from(index).unwrap_or(u64::MAX)
                        && source.item.name.raw().0 == name
                        && source.item.descriptor.raw().0 == b"I",
                    source.declaration.is_some(),
                    source.markers.is_empty(),
                    source.annotations.refusals.is_empty(),
                    source.type_annotations.refusals.is_empty(),
                )
            })
        },
        budget,
    )
}

fn enum_constructor_signature_matches(
    header: &MemberHeader,
    source: &ClassSourceMethod,
    physical_descriptor: &[u8],
) -> bool {
    jarde_java::source_proof::enum_constructor_signature_matches(
        header,
        source.enum_constructor_source_tail,
        source.enum_constructor_signature_erasure_refused,
        source.markers.len(),
        physical_descriptor,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    const STAGE: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/classes-original/Stage.class"
    );
    const FROZEN_EMPTY_ENUM: &[u8] =
        include_bytes!("../tests/fixtures/proved-java-structure/enum-arity/v8/probe/Empty.class");
    const FROZEN_ONE_ENUM: &[u8] =
        include_bytes!("../tests/fixtures/proved-java-structure/enum-arity/v8/probe/One.class");
    const FROZEN_FOUR_ENUM: &[u8] =
        include_bytes!("../tests/fixtures/proved-java-structure/enum-arity/v8/probe/Four.class");
    const MEASURE: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/helper-constructor-prefix-boundaries/generated/input/source-baseline/Measure.class"
    );
    const MEASURE_SOURCE: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/user-static-boundary/Measure.java"
    );
    const MEASURE_RUNNER: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/user-static-boundary/MeasureRunner.java"
    );
    const MEASURE_JADX_SOURCE: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/user-static-boundary/jadx/sources/defpackage/Measure.java"
    );
    const MEASURE_JADX_RUNNER: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/user-static-boundary/jadx/sources/defpackage/MeasureRunner.java"
    );
    const CUSTOM_INIT_SOURCE: &str =
        include_str!("../openspec/evidence/java-syntax-2026-09-27/dt14-enum-init/CustomInit.java");
    const CUSTOM_INIT_RUNNER: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-27/dt14-enum-init/CustomInitRunner.java"
    );
    const OP_SOURCE: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-25/enum-constant-specific-body/Op.java"
    );
    const MIXED_SOURCE: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-25/enum-constant-specific-body/Mixed.java"
    );
    const PLAIN_SOURCE: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-25/enum-constant-specific-body/Plain.java"
    );
    const DELEGATING_ENUM_SOURCE: &str = r#"
public enum DelegatingEnum {
    ZERO,
    ONE(1);

    private final int value;

    DelegatingEnum() {
        this(0);
    }

    DelegatingEnum(int value) {
        ConstructorEffects.record(value);
        this.value = value;
    }

    int value() {
        return value;
    }
}

final class ConstructorEffects {
    static void record(int value) { }
}
"#;

    #[test]
    fn object_and_array_descriptors_follow_the_classfile_owner() {
        assert_eq!(object_descriptor(b"p/Stage"), b"Lp/Stage;");
        assert_eq!(array_descriptor(b"p/Stage"), b"[Lp/Stage;");
        assert_eq!(values_descriptor(b"p/Stage"), b"()[Lp/Stage;");
        assert_eq!(
            value_of_descriptor(b"p/Stage"),
            b"(Ljava/lang/String;)Lp/Stage;"
        );
    }

    #[test]
    fn java8_two_constant_enum_capture_admits_op_without_proving_or_fabricating_code() {
        for debug in [true, false] {
            for (name, source, abstract_enum, expected_allocations) in [
                (
                    "Op",
                    OP_SOURCE,
                    true,
                    vec![
                        (0, 7, b"demo/Op$1".as_slice()),
                        (13, 20, b"demo/Op$2".as_slice()),
                    ],
                ),
                (
                    "Mixed",
                    MIXED_SOURCE,
                    false,
                    vec![
                        (0, 7, b"demo/Mixed$1".as_slice()),
                        (13, 20, b"demo/Mixed".as_slice()),
                    ],
                ),
                (
                    "Plain",
                    PLAIN_SOURCE,
                    false,
                    vec![
                        (0, 7, b"demo/Plain".as_slice()),
                        (13, 20, b"demo/Plain".as_slice()),
                    ],
                ),
            ] {
                let bytes = compile_frozen_enum(name, source, debug);
                let facts = jarde_reader::classfile::class_facts(&bytes, &mut test_budget())
                    .expect("the enum class facts decode");
                assert_eq!(facts.access_flags & ACC_ABSTRACT != 0, abstract_enum);
                assert!(
                    may_capture_group_code(&facts, true),
                    "{name} should enter the same-run candidate collector"
                );
                let snapshot = crate::Engine::new()
                    .open(
                        crate::ArtifactInput::bytes(bytes.clone()),
                        &mut test_budget(),
                    )
                    .expect("the enum fixture opens");
                let request = enum_request(&snapshot, &format!("demo/{name}"));
                let report = performed(
                    crate::Engine::new()
                        .class_source(
                            std::slice::from_ref(&snapshot),
                            &request,
                            &mut test_budget(),
                        )
                        .expect("the candidate class-source pass completes"),
                );
                if name == "Plain" {
                    assert!(matches!(
                        report.enum_constant_proof,
                        ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(_))
                    ));
                    assert!(report.text.contains("READY,\n    WAITING;"));
                } else {
                    assert!(matches!(
                        report.enum_constant_proof,
                        ClassSourceEnumConstantProof::Refused { .. }
                    ));
                }
                if name == "Op" {
                    let abstract_apply = report
                        .methods
                        .iter()
                        .find(|method| method.item.name.raw().0 == b"apply")
                        .expect("the physical abstract method remains in the method table");
                    assert_eq!(
                        abstract_apply.no_body_kind,
                        Some(crate::NoBodyKind::Abstract)
                    );
                    assert!(matches!(abstract_apply.outcome, ClassSourceOutcome::NoBody));
                    assert!(!report.text.contains("ADD {"));
                    assert!(!report.text.contains("MULTIPLY {"));
                }
                let environment = request
                    .environment
                    .build(std::slice::from_ref(&snapshot))
                    .expect("the fixture environment builds");
                for method in report
                    .methods
                    .iter()
                    .filter(|method| matches!(method.outcome, ClassSourceOutcome::Recovered { .. }))
                {
                    // This direct candidate extraction test keeps the `MethodIr` and candidate
                    // from one analysis run. The production class-source handoff is exercised by
                    // the report above; no run's candidate is reconstructed from report text.
                    let analysis = jarde_jvm::analyze_method_ir(
                        std::slice::from_ref(&snapshot),
                        &crate::ir::MethodAnalysisRequest {
                            environment: environment.clone(),
                            method: method.item.identity.clone(),
                            stages: crate::AnalysisStage::ALL.to_vec(),
                        },
                        &mut test_budget(),
                    )
                    .expect("the method analysis completes");
                    let candidate = capture_method_code(
                        method.item.index,
                        &method.item.identity,
                        analysis.ir(),
                        &mut test_budget(),
                    )
                    .expect("same-run enum facts fit the budget")
                    .expect("every recovered method is retained as a candidate");
                    assert_eq!(candidate.member.as_ref(), Some(&method.item.identity));
                    assert!(candidate.complete, "{name} method Code is complete");
                    if method.item.name.raw().0 == b"<clinit>" {
                        for (new_bci, constructor_bci, owner) in &expected_allocations {
                            let new = candidate
                                .instructions
                                .iter()
                                .find(|instruction| instruction.bci == *new_bci)
                                .expect("the expected allocation BCI is retained");
                            assert_eq!(new.opcode, 0xbb);
                            assert_eq!(
                                new.reference,
                                Some(EnumCodeReference::Class(owner.to_vec()))
                            );
                            let constructor = candidate
                                .instructions
                                .iter()
                                .find(|instruction| instruction.bci == *constructor_bci)
                                .expect("the expected constructor BCI is retained");
                            assert_eq!(constructor.opcode, 0xb7);
                            assert_eq!(
                                constructor.reference,
                                Some(EnumCodeReference::Method {
                                    owner: owner.to_vec(),
                                    name: b"<init>".to_vec(),
                                    descriptor: b"(Ljava/lang/String;I)V".to_vec(),
                                    interface: false,
                                })
                            );
                        }
                    }
                    if name == "Op" && method.item.name.raw().0 == b"tag" {
                        for name in [b"name".as_slice(), b"ordinal".as_slice()] {
                            assert!(candidate.member_uses.iter().any(|use_site| {
                                matches!(
                                    &use_site.reference,
                                    EnumCodeReference::Method { name: actual, .. }
                                        if actual.as_slice() == name
                                )
                            }));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn frozen_plain_enum_arities_project_as_one_ordered_physical_group() {
        for (bytes, name, expected) in [
            (FROZEN_EMPTY_ENUM, "Empty", Vec::<&str>::new()),
            (FROZEN_ONE_ENUM, "One", vec!["ONLY"]),
            (
                FROZEN_FOUR_ENUM,
                "Four",
                vec!["NORTH", "SOUTH", "EAST", "WEST"],
            ),
        ] {
            let report = enum_report(bytes, &format!("probe/{name}"), &mut test_budget());
            let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(group)) =
                &report.enum_constant_proof
            else {
                panic!(
                    "{name} should have a complete ordinary enum proof: {:?}",
                    report.enum_constant_proof
                );
            };
            assert_eq!(
                group
                    .constants
                    .iter()
                    .map(|constant| constant.name.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                group
                    .constants
                    .iter()
                    .map(|constant| constant.ordinal)
                    .collect::<Vec<_>>(),
                (0..expected.len())
                    .map(|ordinal| i32::try_from(ordinal).unwrap())
                    .collect::<Vec<_>>()
            );
            assert!(report.text.contains(&format!("enum {name} {{")));
            for constant in &expected {
                assert!(
                    report.text.contains(constant),
                    "{name} source omits {constant}"
                );
            }
            assert!(
                report
                    .fields
                    .iter()
                    .any(|field| field.item.name.raw().0 == b"$VALUES")
            );
            assert!(
                report
                    .methods
                    .iter()
                    .any(|method| method.item.name.raw().0 == b"<init>")
            );
            assert!(
                !report
                    .text
                    .contains(&format!("private {name}(java.lang.String")),
                "{name} source retained its implicit constructor: {}",
                report.text
            );
        }
    }

    #[test]
    fn string_varargs_arrays_are_proved_and_projected_atomically() {
        const SOURCE: &str = include_str!(
            "../openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/StringVarargs.java"
        );
        let bytes = compile_java_class("StringVarargs", SOURCE, false);
        let report = enum_report(&bytes, "StringVarargs", &mut test_budget());
        let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(group)) =
            &report.enum_constant_proof
        else {
            panic!(
                "String... group should prove from complete class facts: {:?}",
                report.enum_constant_proof
            );
        };
        assert_eq!(group.constants.len(), 3);
        let expected = [
            ("PAIR", vec![".dex", ".class"]),
            ("SINGLE", vec![".xml"]),
            ("EMPTY", vec![]),
        ];
        for (constant, (name, values)) in group.constants.iter().zip(expected) {
            assert_eq!(constant.name, name);
            let Some(ProvedEnumSourceArgument::StringVarargs(actual)) = &constant.source_argument
            else {
                panic!("{} must retain a proved String... argument", constant.name);
            };
            assert_eq!(
                actual
                    .iter()
                    .map(|value| value.value.as_str())
                    .collect::<Vec<_>>(),
                values
            );
            assert_eq!(actual.len(), values.len());
            assert!(actual.windows(2).all(|pair| pair[0].bci < pair[1].bci));
            assert!(
                actual
                    .iter()
                    .all(|value| value.bci < constant.constructor_bci)
            );
        }
        assert!(report.text.contains("PAIR(\".dex\", \".class\"),"));
        assert!(report.text.contains("SINGLE(\".xml\"),"));
        assert!(report.text.contains("EMPTY();"));
        assert!(
            report
                .text
                .contains("private StringVarargs(java.lang.String... arg0)")
        );
        assert!(
            !report
                .text
                .contains("public static final StringVarargs PAIR;")
        );
    }

    #[test]
    fn unsupported_string_varargs_classfiles_refuse_the_complete_group() {
        let cases = [
            (
                "missing-varargs-flag",
                "public enum Bad { ONE(new String[]{\"x\"}); private final String[] values; Bad(String[] values) { this.values = values; } }",
            ),
            (
                "ambiguous-string-fields",
                "public enum Bad { ONE(\"x\"); private final String[] values; private String[] other; Bad(String... values) { this.values = values; } }",
            ),
            (
                "nonliteral-element",
                "public enum Bad { ONE(value()); private final String[] values; Bad(String... values) { this.values = values; } static String value() { return \"x\"; } }",
            ),
            (
                "extra-array-consumer",
                "public enum Bad { ONE(consume(new String[]{\"x\"})); private final String[] values; Bad(String... values) { this.values = values; } static String[] consume(String[] values) { return values; } }",
            ),
            (
                "different-array-type",
                "public enum Bad { ONE((String[])(Object)new StringBuilder[]{new StringBuilder(\"x\")}); private final String[] values; Bad(String... values) { this.values = values; } }",
            ),
            (
                "non-ascii-mutf8-literal",
                "public enum Bad { ONE(\"é\"); private final String[] values; Bad(String... values) { this.values = values; } }",
            ),
            (
                "invalid-initializer-suffix",
                "public enum Bad { ONE(\"x\"); private final String[] values; Bad(String... values) { this.values = values; } static { touch(); } static void touch() {} }",
            ),
        ];
        for (case, source) in cases {
            let bytes = compile_java_class("Bad", source, false);
            if case == "different-array-type" {
                verify_classfile_without_initialization("Bad", &bytes);
            }
            let report = enum_report(&bytes, "Bad", &mut test_budget());
            if case == "invalid-initializer-suffix" {
                assert!(matches!(
                    report.enum_constant_proof,
                    ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(_))
                ));
            } else {
                assert!(
                    matches!(
                        report.enum_constant_proof,
                        ClassSourceEnumConstantProof::Refused { .. }
                    ),
                    "{case}: {:?}",
                    report.enum_constant_proof
                );
            }
            assert!(
                report.text.contains("public static final Bad ONE;"),
                "{case}: {}",
                report.text
            );
            assert!(
                !report.text.contains("enum Bad {\n    ONE("),
                "{case}: {}",
                report.text
            );
        }
    }

    #[test]
    fn string_varargs_source_uses_the_existing_java_literal_escaper() {
        let source = r#"public enum Escaped { VALUE("\"\\\n"); private final String[] values; Escaped(String... values) { this.values = values; } }"#;
        let bytes = compile_java_class("Escaped", source, false);
        let report = enum_report(&bytes, "Escaped", &mut test_budget());
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(_))
        ));
        assert!(
            report.text.contains(r#"VALUE("\"\\\n");"#),
            "{}",
            report.text
        );
        let _recompiled = compile_java_class("Escaped", &report.text, false);
    }

    #[test]
    fn one_unproved_string_varargs_constant_refuses_all_constants() {
        let source = "public enum Bad { GOOD(\"ok\"), BAD(value()); private final String[] values; Bad(String... values) { this.values = values; } static String value() { return \"bad\"; } }";
        let bytes = compile_java_class("Bad", source, false);
        let report = enum_report(&bytes, "Bad", &mut test_budget());
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Refused { .. }
        ));
        assert!(report.text.contains("public static final Bad GOOD;"));
        assert!(report.text.contains("public static final Bad BAD;"));
        assert!(!report.text.contains("enum Bad {\n    GOOD("));
    }

    #[test]
    fn string_varargs_signature_mismatch_refuses_without_partial_projection() {
        const SOURCE: &str = include_str!(
            "../openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/StringVarargs.java"
        );
        let mut bytes = compile_java_class("StringVarargs", SOURCE, false);
        replace_constant_pool_utf8(
            &mut bytes,
            b"([Ljava/lang/String;)V",
            b"([Ljava/lang/Strung;)V",
        );
        verify_classfile_without_initialization("StringVarargs", &bytes);
        let report = enum_report(&bytes, "StringVarargs", &mut test_budget());
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Refused { .. }
        ));
        assert!(
            report
                .text
                .contains("public static final StringVarargs PAIR;")
        );
        assert!(!report.text.contains("PAIR(\".dex\""));
    }

    #[test]
    fn missing_string_varargs_constructor_code_refuses_the_group() {
        const SOURCE: &str = include_str!(
            "../openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/StringVarargs.java"
        );
        let mut bytes = compile_java_class("StringVarargs", SOURCE, false);
        remove_method_attribute(
            &mut bytes,
            b"<init>",
            STRING_VARARGS_CTOR_DESCRIPTOR,
            b"Code",
        );
        let report = enum_report(&bytes, "StringVarargs", &mut test_budget());
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Refused { .. }
        ));
        assert!(
            report
                .text
                .contains("public static final StringVarargs PAIR;")
        );
        assert!(!report.text.contains("PAIR(\".dex\""));
    }

    #[test]
    fn repeated_string_array_index_write_is_jvm_verifiable_and_refused() {
        const SOURCE: &str = include_str!(
            "../openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/StringVarargs.java"
        );
        let mut bytes = compile_java_class("StringVarargs", SOURCE, false);
        let candidates: Vec<_> = bytes
            .windows(4)
            .enumerate()
            .filter_map(|(at, bytes)| {
                (bytes[0] == 0x59 && bytes[1] == 0x04 && matches!(bytes[2], 0x12 | 0x13))
                    .then_some(at + 1)
            })
            .collect();
        assert_eq!(candidates.len(), 1, "the PAIR second index store is unique");
        bytes[candidates[0]] = 0x03; // PAIR writes index zero twice; the JVM still verifies it.
        verify_classfile_without_initialization("StringVarargs", &bytes);
        let report = enum_report(&bytes, "StringVarargs", &mut test_budget());
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Refused { .. }
        ));
        assert!(
            report
                .text
                .contains("public static final StringVarargs PAIR;")
        );
        assert!(!report.text.contains("PAIR(\".dex\""));
    }

    #[test]
    fn string_varargs_low_budget_and_cancellation_keep_the_group_unpublished() {
        const SOURCE: &str = include_str!(
            "../openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/StringVarargs.java"
        );
        let bytes = compile_java_class("StringVarargs", SOURCE, false);

        let mut limits = test_budget().limits().clone();
        limits.ir_items = 150;
        let budgeted = enum_report(&bytes, "StringVarargs", &mut Budget::new(limits));
        assert!(matches!(
            budgeted.enum_constant_proof,
            ClassSourceEnumConstantProof::Stopped { .. }
        ));
        assert!(!budgeted.text.contains("PAIR(\".dex\""));
        assert!(
            budgeted
                .text
                .contains("public static final StringVarargs PAIR;")
        );

        let token = crate::CancellationToken::new();
        token.cancel();
        let engine = crate::Engine::new();
        let snapshot = engine
            .open(crate::ArtifactInput::bytes(bytes), &mut test_budget())
            .expect("the fixture opens before cancellation");
        let request = enum_request(&snapshot, "StringVarargs");
        let mut cancelled = Budget::with_cancellation_token(test_budget().limits().clone(), token);
        match engine
            .class_source(std::slice::from_ref(&snapshot), &request, &mut cancelled)
            .expect("cancellation remains an ordinary partial operation result")
        {
            crate::OperationOutcome::Incomplete(candidates) => {
                assert!(matches!(
                    candidates.execution,
                    ExecutionReport::Cancelled { .. }
                ));
                assert!(candidates.candidates.is_empty());
            }
            crate::OperationOutcome::Performed(report) => {
                assert!(matches!(
                    report.execution,
                    ExecutionReport::Cancelled { .. }
                ));
                assert!(!report.text.contains("PAIR(\".dex\""));
            }
            crate::OperationOutcome::Ambiguous(candidates) => {
                panic!("the standalone cancellation fixture became ambiguous: {candidates:?}");
            }
        }
    }

    #[test]
    fn int_static_field_arguments_preserve_the_same_run_expression_and_runtime_order() {
        const INT_ENUM: &str = r#"public enum IntArgs {
    LITERAL(1), FIELD(Ints.THREE), EXPR(Ints.THREE + 1);
    private final int value;
    IntArgs(int value) { this.value = value; }
    int value() { return value; }
}
"#;
        const INTS: &str = r#"final class Ints {
    static int initializations;
    static int THREE;
    static { initializations++; THREE = 3; }
}
"#;
        const RUNNER: &str = r#"public final class IntArgsRunner {
    public static void main(String[] args) {
        if (IntArgs.LITERAL.value() != 1 || IntArgs.FIELD.value() != 3 || IntArgs.EXPR.value() != 4) {
            throw new AssertionError("enum constructor values or order changed");
        }
        if (Ints.initializations != 1) {
            throw new AssertionError("static owner initialization count changed: " + Ints.initializations);
        }
        System.out.println("OK IntArgs");
    }
}
"#;

        let compiled = compile_java_sources(
            "int-arguments-input",
            &[("IntArgs", INT_ENUM), ("Ints", INTS)],
        );
        let bytes = compiled[0].clone();
        let ints_bytes = compiled[1].clone();
        let report =
            enum_report_with_dependencies(&bytes, "IntArgs", &[ints_bytes], &mut test_budget());
        let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(group)) =
            &report.enum_constant_proof
        else {
            panic!(
                "bounded int argument group should prove: {:?}; clinit={:?}",
                report.enum_constant_proof,
                report
                    .methods
                    .iter()
                    .find(|method| method.item.name.raw().0 == b"<clinit>")
                    .map(|method| (&method.outcome, &method.markers, &method.text))
            );
        };
        assert_eq!(group.constants.len(), 3);
        assert!(matches!(
            group.constants[0].source_argument,
            Some(ProvedEnumSourceArgument::Int(
                ProvedEnumIntArgument::Literal { value: 1, .. }
            ))
        ));
        let Some(ProvedEnumSourceArgument::Int(ProvedEnumIntArgument::StaticField {
            owner,
            name,
            descriptor,
            source_owner,
            source_name,
            bci: field_bci,
        })) = &group.constants[1].source_argument
        else {
            panic!("FIELD must retain one field-reference leaf");
        };
        assert_eq!(owner, b"Ints");
        assert_eq!(name, b"THREE");
        assert_eq!(descriptor, b"I");
        assert_eq!(source_owner, "Ints");
        assert_eq!(source_name, "THREE");
        assert_eq!(*field_bci + 3, group.constants[1].constructor_bci);
        let Some(ProvedEnumSourceArgument::Int(ProvedEnumIntArgument::Add {
            left,
            right,
            bci: add_bci,
        })) = &group.constants[2].source_argument
        else {
            panic!("EXPR must retain the ordered field-plus-literal expression");
        };
        assert!(matches!(
            left.as_ref(),
            ProvedEnumIntArgument::StaticField { owner, name, descriptor, .. }
                if owner == b"Ints" && name == b"THREE" && descriptor == b"I"
        ));
        assert!(matches!(
            right.as_ref(),
            ProvedEnumIntArgument::Literal { value: 1, .. }
        ));
        assert_eq!(*add_bci + 1, group.constants[2].constructor_bci);
        assert_eq!(
            group.constants[1]
                .source_argument
                .as_ref()
                .unwrap()
                .source_text(),
            "Ints.THREE"
        );
        assert_eq!(
            group.constants[2]
                .source_argument
                .as_ref()
                .unwrap()
                .source_text(),
            "Ints.THREE + 1"
        );
        assert!(
            report
                .text
                .contains("LITERAL(1),\n    FIELD(Ints.THREE),\n    EXPR(Ints.THREE + 1);"),
            "projected enum source: {}",
            report.text
        );
        let clinit = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"<clinit>")
            .expect("the physical initializer report remains present");
        let ClassSourceOutcome::Recovered { report: clinit, .. } = &clinit.outcome else {
            panic!("the physical initializer fallback remains independently reportable");
        };
        assert!(clinit.text.contains("@bytecode 14"));
        assert!(
            clinit
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == "jre_new_interleaved_effect" })
        );

        let original = compile_and_run_sources(
            "enum-int-arguments-original",
            &[
                ("IntArgs.java", &format!("{INT_ENUM}\n{INTS}")),
                ("IntArgsRunner.java", RUNNER),
            ],
            false,
            "IntArgsRunner",
        );
        let jarde = compile_and_run_sources(
            "enum-int-arguments-jarde",
            &[
                ("IntArgs.java", &report.text),
                ("Ints.java", INTS),
                ("IntArgsRunner.java", RUNNER),
            ],
            false,
            "IntArgsRunner",
        );
        assert_eq!(original, "OK IntArgs\n");
        assert_eq!(jarde, original);
    }

    #[test]
    fn int_ternary_arguments_bind_both_arms_and_preserve_condition_evaluation() {
        const ENUM: &str = r#"public enum TernaryArgs {
    FIRST(choose() ? 1 : 10), SECOND(choose() ? 2 : 20), ANY(choose() ? 1 : 2),
    INVERTED(!choose() ? 2 : 20);
    static int calls;
    final int value;
    TernaryArgs(int value) { this.value = value; }
    static boolean choose() { calls++; return calls % 2 == 1; }
}"#;
        const RUNNER: &str = r#"public final class TernaryArgsRunner {
    public static void main(String[] args) {
        System.out.println(TernaryArgs.FIRST.value + ":" + TernaryArgs.SECOND.value + ":"
            + TernaryArgs.ANY.value + ":" + TernaryArgs.INVERTED.value + ":" + TernaryArgs.calls);
    }
}"#;
        let bytes = compile_java_class("TernaryArgs", ENUM, false);
        verify_classfile_without_initialization("TernaryArgs", &bytes);
        let report = enum_report(&bytes, "TernaryArgs", &mut test_budget());
        let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(group)) =
            &report.enum_constant_proof
        else {
            panic!(
                "int ternary group should prove: {:?}",
                report.enum_constant_proof
            );
        };
        let Some(ProvedEnumSourceArgument::Int(ProvedEnumIntArgument::Ternary {
            condition,
            when_true,
            when_false,
            branch_bci,
            join_bci,
        })) = &group.constants[0].source_argument
        else {
            panic!("the first argument must retain its conditional proof");
        };
        assert_eq!(condition.owner, b"TernaryArgs");
        assert_eq!(condition.name, b"choose");
        assert_eq!(condition.descriptor, b"()Z");
        assert_eq!(condition.source_name, "choose");
        assert!(condition.bci < *branch_bci && *branch_bci < *join_bci);
        assert_eq!(when_true.source_text(), "1");
        assert_eq!(when_false.source_text(), "10");
        assert_eq!(group.constants[0].constructor_bci, *join_bci);
        assert_eq!(
            group
                .constants
                .iter()
                .map(|constant| constant.source_argument.as_ref().unwrap().source_text())
                .collect::<Vec<_>>(),
            [
                "choose() ? 1 : 10",
                "choose() ? 2 : 20",
                "choose() ? 1 : 2",
                "choose() ? 20 : 2"
            ]
        );
        assert!(report.text.contains(
            "FIRST(choose() ? 1 : 10),\n    SECOND(choose() ? 2 : 20),\n    ANY(choose() ? 1 : 2),\n    INVERTED(choose() ? 20 : 2);"
        ));
        assert!(
            !report
                .text
                .contains("public static final TernaryArgs FIRST;")
        );
        let initializer = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"<clinit>")
            .unwrap();
        assert!(initializer.text.contains("@bytecode"));

        let original = compile_and_run_sources(
            "int-ternary-original",
            &[
                ("TernaryArgs.java", ENUM),
                ("TernaryArgsRunner.java", RUNNER),
            ],
            false,
            "TernaryArgsRunner",
        );
        let recovered = compile_and_run_sources(
            "int-ternary-recovered",
            &[
                ("TernaryArgs.java", &report.text),
                ("TernaryArgsRunner.java", RUNNER),
            ],
            false,
            "TernaryArgsRunner",
        );
        assert_eq!(original, "1:20:1:2:4\n");
        assert_eq!(recovered, original);

        let mut limits = test_budget().limits().clone();
        limits.ir_items = 150;
        let limited = enum_report(&bytes, "TernaryArgs", &mut Budget::new(limits));
        assert!(matches!(
            limited.enum_constant_proof,
            ClassSourceEnumConstantProof::Stopped { .. }
        ));
        assert!(!limited.text.contains("FIRST(choose()"));
        assert!(
            limited
                .text
                .contains("public static final TernaryArgs FIRST;")
        );

        let engine = crate::Engine::new();
        let snapshot = engine
            .open(crate::ArtifactInput::bytes(bytes), &mut test_budget())
            .expect("the class opens before cancellation");
        let request = enum_request(&snapshot, "TernaryArgs");
        let token = crate::CancellationToken::new();
        token.cancel();
        let cancelled = engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &request,
                &mut Budget::with_cancellation_token(test_budget().limits().clone(), token),
            )
            .expect("cancellation is an operation stop");
        match cancelled {
            crate::OperationOutcome::Incomplete(candidates) => {
                assert!(candidates.candidates.is_empty())
            }
            crate::OperationOutcome::Performed(report) => {
                assert!(!report.text.contains("FIRST(choose()"));
            }
            crate::OperationOutcome::Ambiguous(candidates) => {
                panic!("the ternary fixture became ambiguous: {candidates:?}")
            }
        }
    }

    #[test]
    fn unsupported_verifier_valid_int_ternary_diamonds_keep_physical_origins() {
        const FIXTURES: [(&str, &str); 3] = [
            (
                "TernaryEffectArm",
                "public enum TernaryEffectArm { VALUE(choose() ? effect(1) : 2); final int value; TernaryEffectArm(int value) { this.value=value; } static boolean choose() { return true; } static int effect(int value) { return value; } }",
            ),
            (
                "TernaryNestedJoin",
                "public enum TernaryNestedJoin { VALUE(choose() ? (choose() ? 1 : 2) : 3); final int value; TernaryNestedJoin(int value) { this.value=value; } static boolean choose() { return true; } }",
            ),
            (
                "TernaryWriteArm",
                "public enum TernaryWriteArm { VALUE(choose() ? (marker = 4) : 2); static int marker; final int value; TernaryWriteArm(int value) { this.value=value; } static boolean choose() { return true; } }",
            ),
        ];
        for (name, source) in FIXTURES {
            let bytes = compile_java_class(name, source, false);
            verify_classfile_without_initialization(name, &bytes);
            let report = enum_report(&bytes, name, &mut test_budget());
            assert!(
                matches!(
                    report.enum_constant_proof,
                    ClassSourceEnumConstantProof::Refused { .. }
                ),
                "{name} must be refused atomically: {:?}",
                report.enum_constant_proof
            );
            assert!(
                report
                    .text
                    .contains(&format!("public static final {name} VALUE;"))
            );
            assert!(!report.text.contains("VALUE(choose()"));
            let initializer = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"<clinit>")
                .unwrap();
            assert!(
                initializer.text.contains("@bytecode"),
                "{name} retains the initializer origin"
            );
        }
    }

    #[test]
    fn scalar_string_arguments_preserve_literal_and_alternating_ternary_runtime() {
        const ENUM: &str = r#"public enum ScalarStringArgs {
    FIRST(choose() ? "1" : "A"), SECOND(choose() ? "2" : "B"),
    LAST(choose() ? "X" : "Z"), LITERAL("plain"),
    INVERTED(!choose() ? "Y" : "N");
    static int calls;
    final String value;
    ScalarStringArgs(String value) { this.value = value; }
    static boolean choose() { calls++; return calls % 2 == 1; }
}"#;
        const RUNNER: &str = r#"public final class ScalarStringArgsRunner {
    public static void main(String[] args) {
        System.out.println(ScalarStringArgs.FIRST.value + ":" + ScalarStringArgs.SECOND.value
            + ":" + ScalarStringArgs.LAST.value + ":" + ScalarStringArgs.LITERAL.value
            + ":" + ScalarStringArgs.INVERTED.value + ":" + ScalarStringArgs.calls);
    }

}"#;
        let bytes = compile_java_class("ScalarStringArgs", ENUM, false);
        verify_classfile_without_initialization("ScalarStringArgs", &bytes);
        let report = enum_report(&bytes, "ScalarStringArgs", &mut test_budget());
        let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(group)) =
            &report.enum_constant_proof
        else {
            panic!("scalar group refused: {:?}", report.enum_constant_proof);
        };
        assert_eq!(group.constants.len(), 5);
        let Some(ProvedEnumSourceArgument::String(ProvedEnumStringArgument::Ternary {
            condition,
            when_true,
            when_false,
            branch_bci,
            join_bci,
        })) = &group.constants[0].source_argument
        else {
            panic!("missing String ternary certificate");
        };
        assert_eq!(condition.name, b"choose");
        assert_eq!(when_true.value, "1");
        assert_eq!(when_false.value, "A");
        assert!(condition.bci < *branch_bci && *branch_bci < *join_bci);
        assert_eq!(*join_bci, group.constants[0].constructor_bci);
        assert!(report.text.contains("FIRST(choose() ? \"1\" : \"A\")"));
        assert!(report.text.contains("SECOND(choose() ? \"2\" : \"B\")"));
        assert!(report.text.contains("LITERAL(\"plain\")"));
        assert!(report.text.contains("INVERTED(choose() ? \"N\" : \"Y\")"));
        assert!(
            report
                .text
                .contains("ScalarStringArgs(java.lang.String arg0)")
        );
        assert_eq!(
            report
                .fields
                .iter()
                .filter(|field| field.item.access_flags & ACC_ENUM != 0)
                .count(),
            5
        );
        for name in [b"<init>".as_slice(), b"<clinit>".as_slice()] {
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == name)
                .expect("the projected source retains its physical method");
            let ClassSourceOutcome::Recovered {
                report: recovery, ..
            } = &method.outcome
            else {
                panic!("the physical method has no recovery report");
            };
            assert!(!recovery.source_map.segments().is_empty());
        }
        let original = compile_and_run_sources(
            "string-original",
            &[
                ("ScalarStringArgs.java", ENUM),
                ("ScalarStringArgsRunner.java", RUNNER),
            ],
            false,
            "ScalarStringArgsRunner",
        );
        let recovered = compile_and_run_sources(
            "string-recovered",
            &[
                ("ScalarStringArgs.java", &report.text),
                ("ScalarStringArgsRunner.java", RUNNER),
            ],
            false,
            "ScalarStringArgsRunner",
        );
        assert_eq!(original, "1:B:X:plain:Y:4\n");
        assert_eq!(recovered, original);

        let mut limits = test_budget().limits().clone();
        limits.ir_items = 150;
        let limited = enum_report(&bytes, "ScalarStringArgs", &mut Budget::new(limits));
        assert!(matches!(
            limited.enum_constant_proof,
            ClassSourceEnumConstantProof::Stopped { .. }
        ));
        assert!(!limited.text.contains("FIRST(choose()"));
        assert!(
            limited
                .text
                .contains("public static final ScalarStringArgs FIRST;")
        );

        let engine = crate::Engine::new();
        let snapshot = engine
            .open(crate::ArtifactInput::bytes(bytes), &mut test_budget())
            .expect("the class opens before cancellation");
        let request = enum_request(&snapshot, "ScalarStringArgs");
        let token = crate::CancellationToken::new();
        token.cancel();
        let cancelled = engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &request,
                &mut Budget::with_cancellation_token(test_budget().limits().clone(), token),
            )
            .expect("cancellation is an operation stop");
        match cancelled {
            crate::OperationOutcome::Incomplete(candidates) => {
                assert!(candidates.candidates.is_empty())
            }
            crate::OperationOutcome::Performed(report) => {
                assert!(!report.text.contains("FIRST(choose()"))
            }
            crate::OperationOutcome::Ambiguous(candidates) => panic!("ambiguous: {candidates:?}"),
        }
    }

    #[test]
    fn unsupported_scalar_string_arguments_refuse_the_whole_group() {
        const CASES: [(&str, &str); 8] = [
            (
                "WrongStringArity",
                "public enum WrongStringArity { A(\"a\", 1); final String value; WrongStringArity(String value, int number) { this.value = value; } }",
            ),
            (
                "WrongStringField",
                "public enum WrongStringField { A(\"a\"); final String value; final String other; WrongStringField(String value) { this.value = value; this.other = value; } }",
            ),
            (
                "StringCtorEffect",
                "public enum StringCtorEffect { A(\"a\"); final String value; StringCtorEffect(String value) { value.trim(); this.value = value; } }",
            ),
            (
                "StringArmEffect",
                "public enum StringArmEffect { A(choose() ? effect(\"a\") : \"b\"); final String value; StringArmEffect(String value) { this.value=value; } static boolean choose() { return true; } static String effect(String value) { return value; } }",
            ),
            (
                "StringNestedArm",
                "public enum StringNestedArm { A(choose() ? (choose() ? \"a\" : \"b\") : \"c\"); final String value; StringNestedArm(String value) { this.value=value; } static boolean choose() { return true; } }",
            ),
            (
                "StringNonAscii",
                "public enum StringNonAscii { A(\"é\"); final String value; StringNonAscii(String value) { this.value=value; } }",
            ),
            (
                "StringSecondRefused",
                "public enum StringSecondRefused { A(\"a\"), B(effect()); final String value; StringSecondRefused(String value) { this.value=value; } static String effect() { return \"b\"; } }",
            ),
            (
                "StringHandler",
                "public enum StringHandler { A(choose() ? \"a\" : \"b\"); final String value; StringHandler(String value) { this.value=value; } static boolean choose() { return true; } static { try { if (System.nanoTime() < 0) throw new RuntimeException(); } catch (RuntimeException ignored) { } } }",
            ),
        ];
        for (name, source) in CASES {
            let bytes = compile_java_class(name, source, false);
            verify_classfile_without_initialization(name, &bytes);
            let report = enum_report(&bytes, name, &mut test_budget());
            assert!(
                matches!(
                    report.enum_constant_proof,
                    ClassSourceEnumConstantProof::Refused { .. }
                ),
                "{name} should refuse: {:?}",
                report.enum_constant_proof
            );
            assert!(
                report
                    .text
                    .contains(&format!("public static final {name} A;")),
                "{name} must retain its physical enum field"
            );
            assert!(
                report
                    .methods
                    .iter()
                    .any(|method| method.item.name.raw().0 == b"<clinit>"),
                "{name} must retain the physical initializer"
            );
        }
    }

    #[test]
    fn scalar_string_diamond_requires_exact_branch_targets_and_literal_arms() {
        const SOURCE: &str = "public enum StringBranchShape { A(choose() ? \"x\" : \"y\"); final String value; StringBranchShape(String value) { this.value=value; } static boolean choose() { return true; } }";
        let bytes = compile_java_class("StringBranchShape", SOURCE, false);
        let snapshot = crate::Engine::new()
            .open(crate::ArtifactInput::bytes(bytes), &mut test_budget())
            .expect("the class opens");
        let mut budget = test_budget();
        let read = snapshot
            .prepared_root_class(&mut budget)
            .expect("the class prepares");
        let prepared = jarde_reader::prepared::PreparedClass::prepare(&read, &mut budget)
            .expect("member tables prepare");
        let facts = prepared.class_facts();
        let index = unique_method(&facts.methods, b"<clinit>", b"()V").unwrap();
        let code = prepared
            .method_code(
                jarde_reader::prepared::MethodOrdinal(index as u32),
                &mut budget,
            )
            .expect("initializer Code decodes");
        let mut instructions: Vec<_> = code
            .instructions
            .iter()
            .enumerate()
            .map(|(index, instruction)| {
                enum_instruction(
                    instruction,
                    code.operands().get(index),
                    &facts.constant_pool,
                    None,
                )
            })
            .collect();
        instructions[5].branch_target_bci = Some(instructions[8].bci);
        instructions[7].branch_target_bci = Some(instructions[9].bci);
        let prove = |instructions: &[EnumCodeInstruction]| {
            prove_string_source_argument(instructions, 4, &facts.this_class.raw().0, &facts.methods)
        };
        assert!(
            matches!(
                prove(&instructions),
                Some((5, ProvedEnumStringArgument::Ternary { .. }))
            ),
            "{instructions:?}"
        );
        let mut wrong_branch = instructions.clone();
        wrong_branch[5].branch_target_bci = Some(wrong_branch[9].bci);
        assert!(prove(&wrong_branch).is_none());
        let mut wrong_join = instructions.clone();
        wrong_join[7].branch_target_bci = Some(wrong_join[8].bci);
        assert!(prove(&wrong_join).is_none());
        let mut wrong_arm = instructions.clone();
        wrong_arm[6].reference = Some(EnumCodeReference::Integer(1));
        assert!(prove(&wrong_arm).is_none());
        let mut non_ascii_arm = instructions.clone();
        non_ascii_arm[6].reference = Some(EnumCodeReference::String(vec![0xc3, 0xa9]));
        assert!(prove(&non_ascii_arm).is_none());

        let constants = [(b"A".to_vec(), 0)];
        let calls = [InitializerConstructorCall {
            descriptor: STRING_CTOR_DESCRIPTOR.to_vec(),
            source_argument: EnumSourceArgument::SingleString,
        }];
        let enum_descriptor = object_descriptor(&facts.this_class.raw().0);
        let prefix = |instructions| {
            let candidate = EnumMethodCodeCandidate {
                table_index: index as u64,
                member: None,
                complete: true,
                exception_handler_count: 0,
                instructions,
                member_uses: Vec::new(),
            };
            prove_initializer_prefix(InitializerPrefixInput {
                code: &candidate,
                owner: &facts.this_class.raw().0,
                constants: &constants,
                constructor_calls: &calls,
                enum_descriptor: &enum_descriptor,
                fields: &facts.fields,
                methods: &facts.methods,
                backing_name: b"$VALUES",
            })
        };
        assert!(prefix(instructions.clone()).is_ok());
        let mut wrong_consumer = instructions;
        wrong_consumer[9].opcode = 0xb8;
        assert!(prefix(wrong_consumer).is_err());
    }

    #[test]
    fn same_package_field_uses_its_short_source_name_and_preserves_initialization() {
        const ENUM: &str = r#"package p;
public enum PackageArgs {
    VALUE(Ints.THREE);
    final int value;
    PackageArgs(int value) { this.value = value; }
}
"#;
        const INTS: &str = r#"package p;
final class Ints {
    static int initializations;
    static int THREE;
    static { initializations++; THREE = 7; }
}
"#;
        const RUNNER: &str = r#"package p;
public final class PackageArgsRunner {
    public static void main(String[] args) {
        if (PackageArgs.VALUE.value != 7 || Ints.initializations != 1) {
            throw new AssertionError("value or initializer count changed");
        }
        System.out.println("OK PackageArgs");
    }
}
"#;
        let compiled = compile_java_sources(
            "same-package-short-owner",
            &[("p/PackageArgs", ENUM), ("p/Ints", INTS)],
        );
        let report = enum_report_with_dependencies(
            &compiled[0],
            "p/PackageArgs",
            &[compiled[1].clone()],
            &mut test_budget(),
        );
        assert!(
            report.text.contains("VALUE(Ints.THREE)"),
            "same-package owner should use its short source name: {}",
            report.text
        );
        assert!(!report.text.contains("VALUE(p.Ints.THREE)"));
        let original = compile_and_run_sources(
            "same-package-short-owner-original",
            &[
                ("p/PackageArgs.java", ENUM),
                ("p/Ints.java", INTS),
                ("p/PackageArgsRunner.java", RUNNER),
            ],
            false,
            "p.PackageArgsRunner",
        );
        let jarde = compile_and_run_sources(
            "same-package-short-owner-jarde",
            &[
                ("p/PackageArgs.java", &report.text),
                ("p/Ints.java", INTS),
                ("p/PackageArgsRunner.java", RUNNER),
            ],
            false,
            "p.PackageArgsRunner",
        );
        assert_eq!(original, "OK PackageArgs\n");
        assert_eq!(jarde, original);
    }

    #[test]
    fn unsupported_int_argument_shapes_refuse_the_whole_constant_group() {
        for (label, argument, extra) in [
            ("method-call", "Ints.value()", ""),
            ("second-getstatic", "Ints.THREE + Ints.ONE", ""),
            ("unsupported-subtraction", "Ints.THREE - 1", ""),
            ("literal-left", "1 + Ints.THREE", ""),
            (
                "constant-used-outside-prefix",
                "Ints.THREE",
                "static IntArgumentRefusal read() { return FIELD; }",
            ),
            (
                "suffix-uses-constant",
                "Ints.THREE",
                "static { marker = IntArgumentRefusal.FIELD.value(); }",
            ),
        ] {
            let source = format!(
                "public enum IntArgumentRefusal {{ LITERAL(1), FIELD({argument}); final int n; static int marker; IntArgumentRefusal(int n) {{ this.n=n; }} int value() {{ return n; }} {extra} }}\nfinal class Ints {{ static int THREE=3; static int ONE=1; static int value() {{ return 3; }} }}\n"
            );
            let bytes = compile_java_class("IntArgumentRefusal", &source, false);
            let report = enum_report(&bytes, "IntArgumentRefusal", &mut test_budget());
            assert!(
                matches!(
                    report.enum_constant_proof,
                    ClassSourceEnumConstantProof::Refused { .. }
                ),
                "{label} should refuse atomically: {:?}",
                report.enum_constant_proof
            );
            assert!(
                report
                    .text
                    .contains("public static final IntArgumentRefusal LITERAL;"),
                "{label} must retain the first constant as a physical field:\n{}",
                report.text
            );
            assert!(
                !report.text.contains("LITERAL(1),"),
                "{label} must not publish a partial enum constant group:\n{}",
                report.text
            );
        }
    }

    #[test]
    fn unsafe_static_field_owners_and_metadata_refuse_the_whole_group() {
        let fixtures: [(&str, &[(&str, &str)], &[&str]); 4] = [
            (
                "final-field",
                &[
                    (
                        "FieldGate",
                        "public enum FieldGate { VALUE(Ints.THREE); final int n; FieldGate(int n) { this.n=n; } }",
                    ),
                    (
                        "Ints",
                        "final class Ints { static final int THREE = Integer.parseInt(\"3\"); }",
                    ),
                ],
                &["Ints"],
            ),
            (
                "cross-package",
                &[
                    (
                        "demo/FieldGate",
                        "package demo; public enum FieldGate { VALUE(elsewhere.Ints.THREE); final int n; FieldGate(int n) { this.n=n; } }",
                    ),
                    (
                        "elsewhere/Ints",
                        "package elsewhere; public final class Ints { public static int THREE = 3; }",
                    ),
                ],
                &["demo/FieldGate", "elsewhere/Ints"],
            ),
            (
                "nested-owner",
                &[
                    (
                        "FieldGate",
                        "public enum FieldGate { VALUE(Outer.Ints.THREE); final int n; FieldGate(int n) { this.n=n; } }",
                    ),
                    (
                        "Outer",
                        "public final class Outer { public static final class Ints { public static int THREE = 3; } }",
                    ),
                ],
                &["Outer"],
            ),
            (
                "same-package-shadow",
                &[
                    (
                        "p/ShadowArgs",
                        "package p; public enum ShadowArgs { VALUE(p.Ints.THREE); final int n; ShadowArgs(int n) { this.n=n; } static final class Ints { } }",
                    ),
                    (
                        "p/Ints",
                        "package p; public final class Ints { public static int THREE = 3; }",
                    ),
                ],
                &["p/Ints"],
            ),
        ];
        for (label, sources, dependency_names) in fixtures {
            let compiled = compile_java_sources(&format!("field-gate-{label}"), sources);
            let enum_index = sources
                .iter()
                .position(|(name, _)| name.ends_with("FieldGate") || name == &"p/ShadowArgs")
                .expect("fixture enum is listed");
            let enum_name = if label == "cross-package" {
                "demo/FieldGate"
            } else if label == "same-package-shadow" {
                "p/ShadowArgs"
            } else {
                "FieldGate"
            };
            let dependencies = dependency_names
                .iter()
                .map(|dependency| {
                    let index = sources
                        .iter()
                        .position(|(name, _)| name == dependency)
                        .expect("dependency source is listed");
                    compiled[index].clone()
                })
                .collect::<Vec<_>>();
            let report = enum_report_with_dependencies(
                &compiled[enum_index],
                enum_name,
                &dependencies,
                &mut test_budget(),
            );
            assert!(
                matches!(
                    report.enum_constant_proof,
                    ClassSourceEnumConstantProof::Refused { .. }
                ),
                "{label} must refuse: {:?}",
                report.enum_constant_proof
            );
            assert!(
                !report.text.contains("VALUE(Ints.THREE)")
                    && !report.text.contains("VALUE(Outer.Ints.THREE)"),
                "{label} must not publish a partially projected enum: {}",
                report.text
            );
        }
    }

    #[test]
    fn selected_environment_field_ambiguity_and_budget_stop_never_publish_int_arguments() {
        const ENUM: &str = "public enum ResolvedInts { VALUE(Ints.THREE); final int n; ResolvedInts(int n) { this.n=n; } }";
        const INTS_WITH_DUPLICATE_CANDIDATE: &str =
            "final class Ints { static int THREE = 3; static int OTHER = 4; }";
        let compiled = compile_java_sources(
            "int-field-ambiguity",
            &[
                ("ResolvedInts", ENUM),
                ("Ints", INTS_WITH_DUPLICATE_CANDIDATE),
            ],
        );
        let mut ambiguous_field = compiled[1].clone();
        let duplicate_name = ambiguous_field
            .windows(b"OTHER".len())
            .position(|window| window == b"OTHER")
            .expect("the same-length field name is in the constant pool");
        ambiguous_field[duplicate_name..duplicate_name + b"OTHER".len()].copy_from_slice(b"THREE");
        let ambiguous = enum_report_with_dependencies(
            &compiled[0],
            "ResolvedInts",
            &[ambiguous_field],
            &mut test_budget(),
        );
        assert!(matches!(
            ambiguous.enum_constant_proof,
            ClassSourceEnumConstantProof::Refused { .. }
        ));
        assert!(!ambiguous.text.contains("VALUE(Ints.THREE)"));

        let mut limits = test_budget().limits().clone();
        limits.class_headers = 2;
        let stopped = enum_report_with_dependencies(
            &compiled[0],
            "ResolvedInts",
            &[compiled[1].clone()],
            &mut Budget::new(limits),
        );
        assert!(
            matches!(
                stopped.enum_constant_proof,
                ClassSourceEnumConstantProof::Stopped { .. }
            ),
            "a stopped owner read must remain stopped: {:?}",
            stopped.enum_constant_proof
        );
        assert!(!stopped.text.contains("VALUE(Ints.THREE)"));
    }

    #[test]
    fn frozen_four_constant_factory_rejects_wrong_length_and_duplicate_entries() {
        let snapshot = crate::Engine::new()
            .open(
                crate::ArtifactInput::bytes(FROZEN_FOUR_ENUM.to_vec()),
                &mut test_budget(),
            )
            .expect("the four-constant enum opens");
        let mut budget = test_budget();
        let read = snapshot
            .prepared_root_class(&mut budget)
            .expect("the same class read prepares");
        let prepared = jarde_reader::prepared::PreparedClass::prepare(&read, &mut budget)
            .expect("the complete physical table prepares");
        let facts = prepared.class_facts();
        let factory_index = facts
            .methods
            .iter()
            .position(|method| method.name.raw().0 == b"$values")
            .expect("the enum factory is present");
        let factory_code = prepared
            .method_code(
                jarde_reader::prepared::MethodOrdinal(
                    u32::try_from(factory_index).expect("factory index fits"),
                ),
                &mut budget,
            )
            .expect("the factory Code decodes");
        let mut candidate = EnumMethodCodeCandidate {
            table_index: u64::try_from(factory_index).expect("factory index fits u64"),
            member: None,
            complete: factory_code.stopped_at.is_none()
                && matches!(factory_code.execution, ExecutionReport::Complete { .. })
                && factory_code.exception_handler_count as usize
                    == factory_code.exception_handlers.len(),
            exception_handler_count: factory_code.exception_handler_count,
            instructions: factory_code
                .instructions
                .iter()
                .enumerate()
                .map(|(index, instruction)| {
                    enum_instruction(
                        instruction,
                        factory_code.operands().get(index),
                        &facts.constant_pool,
                        None,
                    )
                })
                .collect(),
            member_uses: Vec::new(),
        };
        let owner = facts.this_class.raw().0.as_slice();
        let constants: Vec<_> = facts
            .fields
            .iter()
            .enumerate()
            .filter(|(_, field)| field.access_flags & ACC_ENUM != 0)
            .enumerate()
            .map(|(ordinal, (index, field))| {
                (
                    index,
                    String::from_utf16(field.name.utf16()).expect("enum name is Java text"),
                    i32::try_from(ordinal).expect("fixture ordinal fits i32"),
                )
            })
            .collect();
        assert!(prove_values_factory(
            &candidate,
            owner,
            &constants,
            &facts.fields
        ));

        candidate.instructions[0].opcode = 0x06;
        assert!(!prove_values_factory(
            &candidate,
            owner,
            &constants,
            &facts.fields
        ));
        candidate.instructions[0].opcode = 0x07;
        candidate.instructions[8].reference = candidate.instructions[4].reference.clone();
        assert!(!prove_values_factory(
            &candidate,
            owner,
            &constants,
            &facts.fields
        ));
    }

    #[test]
    fn implicit_enum_constructor_with_an_extra_effect_refuses_the_whole_group() {
        let bytes = compile_java_class(
            "EffectEnum",
            "public enum EffectEnum { ONLY; private EffectEnum() { Effect.touch(); } } final class Effect { static void touch() {} }",
            false,
        );
        let report = enum_report(&bytes, "EffectEnum", &mut test_budget());
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Refused { .. }
        ));
        let constructor = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"<init>")
            .expect("the refused constructor remains physical");
        assert!(constructor.text.contains("touch"));
    }

    #[test]
    fn implicit_enum_constructor_with_wrong_ordinal_forwarding_refuses_the_group() {
        let mut bytes = FROZEN_ONE_ENUM.to_vec();
        let pattern = [0x2a, 0x2b, 0x1c, 0xb7];
        let matches: Vec<_> = bytes
            .windows(pattern.len())
            .enumerate()
            .filter_map(|(index, window)| (window == pattern).then_some(index))
            .collect();
        let [constructor_code] = matches.as_slice() else {
            panic!("the frozen constructor forwarding sequence is unique: {matches:?}");
        };
        bytes[constructor_code + 2] = 0x03; // iconst_0 keeps verifier shape, but stops forwarding ordinal.
        let report = enum_report(&bytes, "probe/One", &mut test_budget());
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Refused { .. }
        ));
        assert!(report.methods.iter().any(|method| {
            method.item.name.raw().0 == b"<init>"
                && method.item.descriptor.raw().0 == DELEGATING_CTOR_DESCRIPTOR
        }));
    }

    #[test]
    fn empty_enum_with_a_user_method_emits_its_required_separator() {
        let bytes = compile_java_class(
            "EmptyMethod",
            "public enum EmptyMethod { ; public int marker() { return 9; } }",
            false,
        );
        let report = enum_report(&bytes, "EmptyMethod", &mut test_budget());
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(_))
        ));
        assert!(report.text.contains("enum EmptyMethod {\n    ;\n"));
        assert!(report.text.contains("int marker()"));
        assert!(!report.text.contains("private EmptyMethod(java.lang.String"));
    }

    #[test]
    fn incomplete_enum_code_budget_does_not_project_any_member() {
        let mut limits = test_budget().limits().clone();
        limits.code_bytes = 1;
        let report = enum_report(FROZEN_EMPTY_ENUM, "probe/Empty", &mut Budget::new(limits));
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Stopped { .. }
                | ClassSourceEnumConstantProof::Refused { .. }
        ));
        assert!(report.text.contains("$VALUES"));
        assert!(!report.text.contains("ONLY"));
    }

    #[test]
    fn abstract_enum_candidate_capture_observes_budget_and_cancellation() {
        let bytes = compile_frozen_enum("Op", OP_SOURCE, true);
        let snapshot = crate::Engine::new()
            .open(crate::ArtifactInput::bytes(bytes), &mut test_budget())
            .expect("the Op enum opens");
        let request = enum_request(&snapshot, "demo/Op");

        let mut limits = test_budget().limits().clone();
        limits.ir_items = 0;
        let mut exhausted = Budget::new(limits);
        let report = performed(
            crate::Engine::new()
                .class_source(std::slice::from_ref(&snapshot), &request, &mut exhausted)
                .expect("the report carries the bounded stop"),
        );
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Stopped { .. }
        ));
        assert!(!matches!(
            report.execution,
            ExecutionReport::Complete { .. }
        ));

        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(test_budget().limits().clone(), cancellation);
        match crate::Engine::new().class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut cancelled,
        ) {
            Ok(crate::OperationOutcome::Incomplete(candidates)) => assert!(matches!(
                candidates.execution,
                ExecutionReport::Cancelled { .. }
            )),
            Err(crate::Error::Cancelled { .. }) => {}
            other => panic!("cancellation did not stop Op candidate collection: {other:?}"),
        }
    }

    #[test]
    fn enum_code_capture_is_structured_and_charges_each_instruction() {
        let snapshot = crate::Engine::new()
            .open(
                crate::ArtifactInput::bytes(STAGE.to_vec()),
                &mut test_budget(),
            )
            .expect("the enum class opens");
        let request = enum_request(&snapshot, "Stage");
        let mut budget = test_budget();
        let report = performed(
            crate::Engine::new()
                .class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
                .expect("the class-source request succeeds"),
        );
        let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(proof)) =
            &report.enum_constant_proof
        else {
            panic!(
                "the exact javac enum pattern proves: {:?}",
                report.enum_constant_proof
            );
        };
        assert_eq!(
            proof
                .constants
                .iter()
                .map(|constant| constant.name.as_str())
                .collect::<Vec<_>>(),
            ["START", "FINISH"]
        );
        assert_eq!(
            proof
                .constants
                .iter()
                .map(|constant| {
                    constant
                        .source_argument
                        .as_ref()
                        .map(ProvedEnumSourceArgument::source_text)
                })
                .collect::<Vec<_>>(),
            [Some("4".to_owned()), Some("9".to_owned())]
        );
        assert_eq!(proof.initializer_prefix_statement_count, 3);
        assert_eq!(proof.initializer_prefix_end_bci, 35);
        assert!(report.text.contains("START(4),\n    FINISH(9);"));
        assert!(report.text.contains("private Stage(int arg0)"));
        assert!(report.text.contains("this.ordinalValue = arg0;"));
        assert!(!report.text.contains("public static final Stage START;"));
        assert!(!report.text.contains("$VALUES"));
        assert!(!report.text.contains("$values"));
        assert!(!report.text.contains("valueOf("));
        assert!(!report.text.contains("static {"));
        assert!(!report.text.contains("generic Signature projection refused"));
        let json = serde_json::to_value(&report).expect("the report serializes");
        let fields = json["fields"].as_array().expect("physical fields remain");
        let methods = json["methods"].as_array().expect("physical methods remain");
        assert_eq!(fields.len(), 4);
        assert_eq!(methods.len(), 6);
        assert!(
            fields
                .iter()
                .any(|field| field["item"]["name"]["escaped"] == "$VALUES")
        );
        assert!(
            methods
                .iter()
                .any(|method| method["item"]["name"]["escaped"] == "<clinit>")
        );
        assert!(
            methods
                .iter()
                .any(|method| method["item"]["name"]["escaped"] == "$values")
        );
        assert!(json.get("enum_constructor_source_signature").is_none());
        assert!(
            json.get("enum_constructor_signature_erasure_refused")
                .is_none()
        );
        assert!(methods.iter().any(|method| {
            method["item"]["name"]["escaped"] == "<init>"
                && method["markers"].as_array().is_some_and(|markers| {
                    markers.iter().any(|marker| {
                        marker
                            .as_str()
                            .is_some_and(|marker| marker.contains("jvm_signature_erasure_mismatch"))
                    })
                })
        }));
        assert!(budget.usage().ir_items > 0);
    }

    #[test]
    fn measure_constants_used_by_user_methods_remain_provable() {
        let report = enum_report(MEASURE, "Measure", &mut test_budget());
        let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(proof)) =
            &report.enum_constant_proof
        else {
            panic!(
                "Measure's source-visible LOW/HIGH uses are allowed: {:?}",
                report.enum_constant_proof
            );
        };
        assert_eq!(
            proof
                .constants
                .iter()
                .map(|constant| constant.name.as_str())
                .collect::<Vec<_>>(),
            ["LOW", "HIGH"]
        );
        assert_eq!(
            proof
                .constants
                .iter()
                .map(|constant| {
                    constant
                        .source_argument
                        .as_ref()
                        .map(ProvedEnumSourceArgument::source_text)
                })
                .collect::<Vec<_>>(),
            [Some("2".to_owned()), Some("5".to_owned())]
        );
        assert!(report.text.contains("LOW(2),\n    HIGH(5);"));
        assert!(report.text.contains("static int totalUnits;"));
        assert!(
            report
                .text
                .contains("static {\n        totalUnits = sumUnits();\n    }")
        );
        assert!(report.text.contains("private Measure(int arg0)"));
        assert!(!report.text.contains("public static final Measure LOW;"));
        assert!(!report.text.contains("Measure.totalUnits = sumUnits();"));
        assert!(!report.text.contains("Measure.$VALUES = $values();"));
        assert!(!report.text.contains("Measure.$VALUES"));

        let field_declaration = report
            .text
            .find("static int totalUnits;")
            .expect("the physical user field stays in the text");
        let initializer_block = report
            .text
            .find("static {\n        totalUnits = sumUnits();")
            .expect("the user suffix is emitted as a static block");
        assert!(field_declaration < initializer_block);
        let json = serde_json::to_value(&report).expect("the report serializes");
        assert_eq!(json["fields"].as_array().unwrap().len(), 5);
        assert_eq!(json["methods"].as_array().unwrap().len(), 6);
    }

    #[test]
    fn measure_static_suffix_recompiles_and_matches_original_and_jadx_with_both_debug_modes() {
        for debug in [true, false] {
            let input = compile_java_class("Measure", MEASURE_SOURCE, debug);
            let report = enum_report(&input, "Measure", &mut test_budget());
            assert!(
                report
                    .text
                    .contains("static {\n        totalUnits = sumUnits();")
            );
            let debug_label = if debug { "debug" } else { "nodebug" };
            let original = compile_and_run_sources(
                &format!("measure-original-{debug_label}"),
                &[
                    ("Measure.java", MEASURE_SOURCE),
                    ("MeasureRunner.java", MEASURE_RUNNER),
                ],
                debug,
                "MeasureRunner",
            );
            let jadx = compile_and_run_sources(
                &format!("measure-jadx-{debug_label}"),
                &[
                    ("defpackage/Measure.java", MEASURE_JADX_SOURCE),
                    ("defpackage/MeasureRunner.java", MEASURE_JADX_RUNNER),
                ],
                debug,
                "defpackage.MeasureRunner",
            );
            let jarde = compile_and_run_sources(
                &format!("measure-jarde-{debug_label}"),
                &[
                    ("Measure.java", &report.text),
                    ("MeasureRunner.java", MEASURE_RUNNER),
                ],
                debug,
                "MeasureRunner",
            );
            assert_eq!(original, "user static boundary: PASS\n");
            assert_eq!(jadx, original);
            assert_eq!(jarde, original);
        }
    }

    #[test]
    fn custom_map_suffix_recompiles_and_preserves_singleton_identity() {
        let bytes = compile_java_sources(
            "custom-map-input",
            &[("dt14/CustomInit", CUSTOM_INIT_SOURCE)],
        )
        .remove(0);
        let report = enum_report(&bytes, "dt14/CustomInit", &mut test_budget());
        assert!(report.text.contains("RED,\n    BLUE;"), "{}", report.text);
        assert!(report.text.contains("static {\n"), "{}", report.text);
        assert!(
            !report
                .text
                .contains("public static final dt14.CustomInit RED;")
        );
        let output = compile_and_run_sources(
            "custom-map-projected",
            &[
                ("dt14/CustomInit.java", &report.text),
                ("dt14/CustomInitRunner.java", CUSTOM_INIT_RUNNER),
            ],
            false,
            "dt14.CustomInitRunner",
        );
        assert_eq!(output, "map=2:true:true\n");
        assert_eq!(report.fields.len(), 4);
        assert!(
            report
                .methods
                .iter()
                .any(|method| method.item.name.raw().0 == b"<clinit>")
        );
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Proved(_)
        ));
        let initializer = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"<clinit>")
            .expect("the physical initializer remains indexed");
        let ClassSourceOutcome::Recovered {
            report: recovered, ..
        } = &initializer.outcome
        else {
            panic!("the original initializer recovery remains available");
        };
        assert!(
            recovered
                .text_of_bci(39)
                .iter()
                .any(|text| text.contains("BY_NAME"))
        );
        assert!(
            recovered
                .text_of_bci(53)
                .iter()
                .any(|text| text.contains("for ("))
        );
        assert!(
            recovered
                .text_of_bci(68)
                .iter()
                .any(|text| text.contains("put("))
        );
    }

    #[test]
    fn custom_map_suffix_rejects_changed_loop_increment() {
        let mut bytes = compile_java_sources(
            "custom-map-increment",
            &[("dt14/CustomInit", CUSTOM_INIT_SOURCE)],
        )
        .remove(0);
        let pattern = [0x84, 0x02, 0x01, 0xa7, 0xff, 0xe6];
        let locations: Vec<_> = bytes
            .windows(pattern.len())
            .enumerate()
            .filter_map(|(index, window)| (window == pattern).then_some(index))
            .collect();
        let [location] = locations.as_slice() else {
            panic!("the javac loop increment has one exact byte sequence")
        };
        bytes[*location + 2] = 2;
        let report = enum_report(&bytes, "dt14/CustomInit", &mut test_budget());
        assert!(
            report
                .text
                .contains("public static final dt14.CustomInit RED;")
        );
        assert!(!report.text.contains("    RED,\n"));
        assert!(
            report
                .methods
                .iter()
                .any(|method| method.item.name.raw().0 == b"<clinit>")
        );
    }

    #[test]
    fn custom_map_suffix_refuses_unproved_effects_and_loop_shapes_atomically() {
        let variants = [
            ("extra-map-write", CUSTOM_INIT_SOURCE.replace(
                "BY_NAME.put(value.name(), value);",
                "BY_NAME.put(value.name(), value); BY_NAME.put(\"other\", RED);",
            )),
            ("extra-call", CUSTOM_INIT_SOURCE.replace(
                "BY_NAME.put(value.name(), value);",
                "BY_NAME.put(value.name(), value); System.nanoTime();",
            )),
            ("exception-edge", CUSTOM_INIT_SOURCE.replace(
                "for (CustomInit value : values()) {\n            BY_NAME.put(value.name(), value);\n        }",
                "try { for (CustomInit value : values()) { BY_NAME.put(value.name(), value); } } catch (RuntimeException ignored) { }",
            )),
            ("unknown-array", CUSTOM_INIT_SOURCE.replace(
                "for (CustomInit value : values())",
                "for (CustomInit value : new CustomInit[]{RED, BLUE})",
            )),
        ];
        for (label, source) in variants {
            assert_ne!(
                source, CUSTOM_INIT_SOURCE,
                "fixture replacement must apply: {label}"
            );
            let bytes = compile_java_sources(label, &[("dt14/CustomInit", &source)]).remove(0);
            let report = enum_report(&bytes, "dt14/CustomInit", &mut test_budget());
            assert!(
                report
                    .text
                    .contains("public static final dt14.CustomInit RED;"),
                "{label}: {}",
                report.text
            );
            assert!(
                report
                    .text
                    .contains("private static final dt14.CustomInit[] $VALUES;"),
                "{label}: {}",
                report.text
            );
            assert!(
                report
                    .text
                    .contains("public static final java.util.Map BY_NAME;"),
                "{label}: {}",
                report.text
            );
            assert!(
                !report.text.contains("    RED,\n"),
                "{label}: {}",
                report.text
            );
            assert!(
                report
                    .methods
                    .iter()
                    .any(|method| method.item.name.raw().0 == b"<clinit>"),
                "{label}"
            );
            assert!(
                report
                    .text
                    .contains("dt14.CustomInit.BY_NAME = new java.util.HashMap();")
                    || report.text.contains("@bytecode"),
                "{label}: {}",
                report.text
            );
        }
    }

    #[test]
    fn custom_map_suffix_output_stop_and_cancellation_leave_no_partial_projection() {
        let bytes = compile_java_sources(
            "custom-map-budget-input",
            &[("dt14/CustomInit", CUSTOM_INIT_SOURCE)],
        )
        .remove(0);
        let complete = enum_report(&bytes, "dt14/CustomInit", &mut test_budget());
        assert!(complete.text.contains("    RED,\n    BLUE;"));
        let mut limits = test_budget().limits().clone();
        limits.output_bytes = complete.usage.output_bytes - 1;
        let stopped = enum_report(&bytes, "dt14/CustomInit", &mut Budget::new(limits));
        assert!(matches!(
            stopped.enum_constant_proof,
            ClassSourceEnumConstantProof::Stopped { .. }
        ));
        assert!(
            stopped
                .text
                .contains("public static final dt14.CustomInit RED;")
        );
        assert!(!stopped.text.contains("    RED,\n"));
        assert!(
            stopped
                .methods
                .iter()
                .any(|method| method.item.name.raw().0 == b"<clinit>")
        );

        let engine = crate::Engine::new();
        let snapshot = engine
            .open(crate::ArtifactInput::bytes(bytes), &mut test_budget())
            .expect("the custom enum opens");
        let request = enum_request(&snapshot, "dt14/CustomInit");
        let token = crate::CancellationToken::new();
        token.cancel();
        let mut budget = Budget::with_cancellation_token(test_budget().limits().clone(), token);
        let outcome = engine
            .class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
            .expect("cancellation remains in the operation plane");
        assert!(matches!(outcome, crate::OperationOutcome::Incomplete(_)));
    }

    #[test]
    fn measure_projection_text_is_equal_for_default_and_all_evidence_with_physical_members_intact()
    {
        let snapshot = crate::Engine::new()
            .open(
                crate::ArtifactInput::bytes(MEASURE.to_vec()),
                &mut test_budget(),
            )
            .expect("the Measure class opens");
        let request = enum_request(&snapshot, "Measure");
        let default = performed(
            crate::Engine::new()
                .class_source(
                    std::slice::from_ref(&snapshot),
                    &request,
                    &mut test_budget(),
                )
                .expect("the default evidence request succeeds"),
        );
        let all = performed(
            crate::Engine::new()
                .class_source_with_evidence(
                    std::slice::from_ref(&snapshot),
                    &request,
                    &crate::RecoveryEvidenceRequest::all(),
                    &mut test_budget(),
                )
                .expect("the complete evidence request succeeds"),
        );
        assert_eq!(default.text, all.text);
        for report in [&default, &all] {
            let json = serde_json::to_value(report).expect("the report serializes");
            let fields = json["fields"].as_array().expect("physical fields remain");
            let methods = json["methods"].as_array().expect("physical methods remain");
            assert_eq!(fields.len(), 5);
            assert_eq!(methods.len(), 6);
            assert_eq!(
                fields
                    .iter()
                    .map(|field| field["item"]["name"]["escaped"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                ["LOW", "HIGH", "units", "totalUnits", "$VALUES"]
            );
            assert_eq!(
                fields
                    .iter()
                    .map(|field| field["item"]["index"].as_u64().unwrap())
                    .collect::<Vec<_>>(),
                [0, 1, 2, 3, 4]
            );
            assert!(fields.iter().all(|field| {
                field["item"]["identity"].is_object()
                    && field["item"]["descriptor"].is_object()
                    && field["item"]["identity"]["owner"] == json["class"]
                    && field["item"]["identity"]["member"]["name"] == field["item"]["name"]["raw"]
                    && field["item"]["identity"]["member"]["descriptor"]
                        == field["item"]["descriptor"]["raw"]
            }));
            assert_eq!(
                methods
                    .iter()
                    .map(|method| method["item"]["name"]["escaped"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                [
                    "values", "valueOf", "<init>", "sumUnits", "$values", "<clinit>"
                ]
            );
            assert_eq!(
                methods
                    .iter()
                    .map(|method| method["item"]["index"].as_u64().unwrap())
                    .collect::<Vec<_>>(),
                [0, 1, 2, 3, 4, 5]
            );
            assert!(methods.iter().all(|method| {
                method["item"]["identity"].is_object()
                    && method["outcome"]["kind"].as_str().is_some()
                    && method["item"]["identity"]["owner"] == json["class"]
                    && method["item"]["identity"]["name"] == method["item"]["name"]["raw"]
                    && method["item"]["identity"]["descriptor"]
                        == method["item"]["descriptor"]["raw"]
            }));
            assert!(report.fields.iter().all(|field| {
                &field.item.identity.owner == &report.class
                    && matches!(
                        &field.item.identity.member,
                        jarde_reader::model::MemberKey::Field { name, descriptor }
                            if name.0 == field.item.name.raw().0
                                && descriptor.0 == field.item.descriptor.raw().0
                    )
            }));
            assert!(report.methods.iter().all(|method| {
                &method.item.identity.owner == &report.class
                    && method.item.identity.name.0 == method.item.name.raw().0
                    && method.item.identity.descriptor.0 == method.item.descriptor.raw().0
            }));
            assert!(!report.text.contains("$values()"));
            assert!(!report.text.contains("Measure[] values()"));
            assert!(!report.text.contains("valueOf(java.lang.String"));
        }

        let initializer = all
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"<clinit>")
            .expect("the physical initializer remains in the method table");
        let crate::ClassSourceOutcome::Recovered { report, .. } = &initializer.outcome else {
            panic!("the physical initializer keeps its original recovery outcome")
        };
        assert!(report.text.contains("Measure.$VALUES = $values();"));
        assert!(report.text.contains("Measure.totalUnits = sumUnits();"));
        assert!(
            report
                .source_map
                .text_of_bci(&report.text, 34)
                .iter()
                .any(|text| text.contains("sumUnits"))
        );
        assert!(
            report
                .source_map
                .text_of_bci(&report.text, 37)
                .iter()
                .any(|text| text.contains("totalUnits"))
        );
        let all_json = serde_json::to_value(&all).expect("the full report serializes");
        let initializer_json = all_json["methods"]
            .as_array()
            .unwrap()
            .iter()
            .find(|method| method["item"]["name"]["escaped"] == "<clinit>")
            .expect("the JSON keeps the physical initializer");
        assert!(
            initializer_json["outcome"]["report"]["source_map"]["segments"]
                .as_array()
                .is_some_and(|segments| !segments.is_empty())
        );
        let serialized_segments = initializer_json["outcome"]["report"]["source_map"]["segments"]
            .as_array()
            .expect("the serialized source map keeps its segments");
        for bci in [34, 37] {
            assert!(serialized_segments.iter().any(|segment| {
                let origin = &segment["origin"];
                origin["primary"]["bci"].as_u64() == Some(bci)
                    || origin["derived"].as_array().is_some_and(|derived| {
                        derived
                            .iter()
                            .any(|anchor| anchor["bci"].as_u64() == Some(bci))
                    })
            }));
        }
        assert_eq!(initializer_json["outcome"]["report"]["text"], report.text);
    }

    #[test]
    fn ordinary_class_fields_and_static_initializer_keep_the_existing_projection() {
        const SOURCE: &str = r#"class OrdinaryInit {
  static int first;
  static int second;

  static { first = 4; second = twice(); }

  static int twice() { return first * 2; }
}
"#;
        let bytes = compile_java_class("OrdinaryInit", SOURCE, true);
        let snapshot = crate::Engine::new()
            .open(crate::ArtifactInput::bytes(bytes), &mut test_budget())
            .expect("the ordinary class opens");
        let request = enum_request(&snapshot, "OrdinaryInit");
        let default = performed(
            crate::Engine::new()
                .class_source(
                    std::slice::from_ref(&snapshot),
                    &request,
                    &mut test_budget(),
                )
                .expect("the default ordinary class report succeeds"),
        );
        let all = performed(
            crate::Engine::new()
                .class_source_with_evidence(
                    std::slice::from_ref(&snapshot),
                    &request,
                    &crate::RecoveryEvidenceRequest::all(),
                    &mut test_budget(),
                )
                .expect("the full-evidence ordinary class report succeeds"),
        );

        assert_eq!(
            default.enum_constant_proof,
            ClassSourceEnumConstantProof::NotApplicable
        );
        assert_eq!(default.text, all.text);
        assert_eq!(default.fields.len(), 2);

        let crate::ClassSourceInitializerProof::Proved { fields } = &default.initializer_proof
        else {
            panic!(
                "ordinary runtime static writes are projected: {:?}",
                default.initializer_proof
            );
        };
        assert_eq!(fields.len(), 2);
        assert_eq!(
            fields
                .iter()
                .map(|field| field.write_order)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        let first = default
            .fields
            .iter()
            .find(|field| field.item.name.raw().0 == b"first")
            .unwrap();
        let second = default
            .fields
            .iter()
            .find(|field| field.item.name.raw().0 == b"second")
            .unwrap();
        assert_eq!(
            fields
                .iter()
                .map(|field| field.field_index)
                .collect::<Vec<_>>(),
            vec![first.item.index, second.item.index]
        );
        for field in [first, second] {
            assert_ne!(
                field.item.access_flags & 0x0008,
                0,
                "runtime fields remain static"
            );
            assert_eq!(
                field.item.access_flags & 0x0010,
                0,
                "runtime fields remain nonfinal"
            );
        }
        assert!(first.declaration.as_deref().unwrap().contains("first = 4"));
        assert!(
            second
                .declaration
                .as_deref()
                .unwrap()
                .contains("second = twice()")
        );
        let first_decl = default.text.find("static int first = 4;").unwrap();
        let second_decl = default.text.find("static int second = twice();").unwrap();
        assert!(
            first_decl < second_decl,
            "proved field declarations keep physical order"
        );
        assert!(
            !default.text.contains("static {"),
            "root projection has no duplicate initializer block"
        );

        let clinit = default
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"<clinit>")
            .expect("physical <clinit> remains in the class report");
        let crate::ClassSourceOutcome::Recovered { report: body, .. } = &clinit.outcome else {
            panic!("physical <clinit> keeps its recovery outcome")
        };
        assert!(clinit.text.contains("static {"));
        assert!(clinit.text.contains("OrdinaryInit.first = 4;"));
        assert!(clinit.text.contains("OrdinaryInit.second = twice();"));
        assert!(
            !body.source_map.segments().is_empty(),
            "physical initializer keeps its source map"
        );
        for field in fields {
            assert!(
                !body.source_map.of_bci(field.write_bci).is_empty(),
                "physical write BCI {} remains anchored",
                field.write_bci
            );
        }
    }

    #[test]
    fn counted_side_effectful_suffix_executes_once_in_the_projected_source() {
        const COUNTED: &str = r#"enum Counted {
  LOW(2), HIGH(5);

  final int units;
  static int calls;
  static int totalUnits;

  Counted(int units) { this.units = units; }

  static { totalUnits = sumUnits(); }

  static int sumUnits() {
    calls++;
    return LOW.units + HIGH.units;
  }
}
"#;
        const RUNNER: &str = r#"class CountedRunner {
  public static void main(String[] args) {
    if (Counted.calls != 1 || Counted.totalUnits != 7) {
      throw new AssertionError("calls=" + Counted.calls + ",total=" + Counted.totalUnits);
    }
    System.out.println("calls=" + Counted.calls + ",total=" + Counted.totalUnits);
  }
}
"#;

        for debug in [true, false] {
            let input = compile_java_class("Counted", COUNTED, debug);
            let report = enum_report(&input, "Counted", &mut test_budget());
            assert!(
                report
                    .text
                    .contains("static {\n        totalUnits = sumUnits();")
            );
            assert!(report.text.contains("Counted.calls = Counted.calls + 1;"));
            let original = compile_and_run_sources(
                &format!("counted-original-{}", if debug { "debug" } else { "none" }),
                &[("Counted.java", COUNTED), ("CountedRunner.java", RUNNER)],
                debug,
                "CountedRunner",
            );
            let jarde = compile_and_run_sources(
                &format!("counted-jarde-{}", if debug { "debug" } else { "none" }),
                &[
                    ("Counted.java", &report.text),
                    ("CountedRunner.java", RUNNER),
                ],
                debug,
                "CountedRunner",
            );
            assert_eq!(original, "calls=1,total=7\n");
            assert_eq!(jarde, original);
        }
    }

    #[test]
    fn static_suffix_uses_structure_across_names_values_and_prefix_lengths() {
        const SOURCE: &str = r#"enum Measure {
  LOW(2), HIGH(5);
  final int units;
  static int calls;
  static int totalUnits;
  Measure(int units) { this.units = units; }
  static { totalUnits = sumUnits(); }
  static int sumUnits() { calls++; return LOW.units + HIGH.units; }
}
"#;
        const RUNNER: &str = r#"class MeasureRunner {
  public static void main(String[] args) {
    System.out.println(Measure.calls + ":" + Measure.totalUnits + ":" + Measure.values().length);
  }
}
"#;
        for (case, source, runner, expected) in [
            (
                "renamed",
                SOURCE
                    .replace("LOW", "SMALL")
                    .replace("HIGH", "LARGE")
                    .replace("totalUnits", "aggregate")
                    .replace("sumUnits", "compute"),
                RUNNER.replace("totalUnits", "aggregate"),
                "1:7:2\n",
            ),
            (
                "wide-values",
                SOURCE.replace("LOW(2), HIGH(5)", "LOW(127), HIGH(300)"),
                RUNNER.to_owned(),
                "1:427:2\n",
            ),
            (
                "three-constants",
                SOURCE.replace("LOW(2), HIGH(5)", "LOW(2), MID(3), HIGH(5)"),
                RUNNER.to_owned(),
                "1:7:3\n",
            ),
            (
                "combined",
                SOURCE
                    .replace("LOW(2), HIGH(5)", "LOW(127), MID(3), HIGH(300)")
                    .replace("LOW", "SMALL")
                    .replace("HIGH", "LARGE")
                    .replace("totalUnits", "aggregate")
                    .replace("sumUnits", "compute"),
                RUNNER.replace("totalUnits", "aggregate"),
                "1:427:3\n",
            ),
        ] {
            let field = if matches!(case, "renamed" | "combined") {
                "aggregate"
            } else {
                "totalUnits"
            };
            let method = if matches!(case, "renamed" | "combined") {
                "compute"
            } else {
                "sumUnits"
            };
            for debug in [true, false] {
                let input = compile_java_class("Measure", &source, debug);
                let report = enum_report(&input, "Measure", &mut test_budget());
                assert!(
                    report
                        .text
                        .contains(&format!("static {{\n        {field} = {method}();")),
                    "{case}, debug={debug}, {:?}: {}",
                    report.enum_constant_proof,
                    report.text
                );
                if case != "renamed" {
                    let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(
                        group,
                    )) = &report.enum_constant_proof
                    else {
                        panic!("{case}: ordinary enum proof missing")
                    };
                    assert_ne!(
                        group.initializer_prefix_end_bci, 34,
                        "{case}: prefix must move"
                    );
                }
                let original = compile_and_run_sources(
                    &format!("suffix-{case}-original-{debug}"),
                    &[("Measure.java", &source), ("MeasureRunner.java", &runner)],
                    debug,
                    "MeasureRunner",
                );
                let recovered = compile_and_run_sources(
                    &format!("suffix-{case}-recovered-{debug}"),
                    &[
                        ("Measure.java", &report.text),
                        ("MeasureRunner.java", &runner),
                    ],
                    debug,
                    "MeasureRunner",
                );
                assert_eq!(original, expected, "{case}, debug={debug}");
                assert_eq!(recovered, original, "{case}, debug={debug}");
            }
        }
    }

    #[test]
    fn extra_calls_field_writes_and_exception_edges_keep_the_unprojected_initializer() {
        let cases = [
            (
                "extra-call",
                r#"enum Measure {
  LOW(2), HIGH(5);
  final int units;
  static int totalUnits;
  Measure(int units) { this.units = units; }
  static { totalUnits = sumUnits(); audit(); }
  static void audit() { }
  static int sumUnits() { return LOW.units + HIGH.units; }
}
"#,
            ),
            (
                "extra-field-write",
                r#"enum Measure {
  LOW(2), HIGH(5);
  final int units;
  static int totalUnits;
  static int marker;
  Measure(int units) { this.units = units; }
  static { totalUnits = sumUnits(); marker = 3; }
  static int sumUnits() { return LOW.units + HIGH.units; }
}
"#,
            ),
            (
                "exception-edge",
                r#"enum Measure {
  LOW(2), HIGH(5);
  final int units;
  static int totalUnits;
  static int marker;
  Measure(int units) { this.units = units; }
  static {
    totalUnits = sumUnits();
    try { audit(); } catch (RuntimeException error) { marker = 3; }
  }
  static void audit() { }
  static int sumUnits() { return LOW.units + HIGH.units; }
}
"#,
            ),
        ];
        for (case, source) in cases {
            let bytes = compile_java_class("Measure", source, false);
            let report = enum_report(&bytes, "Measure", &mut test_budget());
            assert!(!report.text.contains("LOW(2),"), "{case}: {}", report.text);
            assert!(
                report.text.contains("public static final Measure LOW;"),
                "{case}: physical enum fields stay visible\n{}",
                report.text
            );
            assert!(report.text.contains("sumUnits"), "{case}: {}", report.text);
            assert!(
                report.text.contains("totalUnits"),
                "{case}: {}",
                report.text
            );
            match case {
                "extra-call" => assert!(report.text.contains("audit()"), "{}", report.text),
                "extra-field-write" => {
                    assert!(report.text.contains("marker"), "{}", report.text)
                }
                "exception-edge" => {
                    assert!(report.text.contains("marker"), "{}", report.text)
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn constructor_index_uses_the_method_table_not_the_shorter_field_table() {
        let snapshot = crate::Engine::new()
            .open(
                crate::ArtifactInput::bytes(STAGE.to_vec()),
                &mut test_budget(),
            )
            .expect("the enum class opens");
        let mut budget = test_budget();
        let read = snapshot
            .prepared_root_class(&mut budget)
            .expect("the same standalone class prepares");
        let prepared = jarde_reader::prepared::PreparedClass::prepare(&read, &mut budget)
            .expect("the physical member tables prepare");
        let facts = prepared.class_facts();
        let constructor_index = facts
            .methods
            .iter()
            .position(|method| method.name.raw().0 == b"<init>")
            .expect("the constructor has one method-table position");
        let initializer_index = facts
            .methods
            .iter()
            .position(|method| method.name.raw().0 == b"<clinit>")
            .expect("the initializer has one method-table position");
        let constructor = &facts.methods[constructor_index];
        assert_eq!(constructor.descriptor.raw().0, CTOR_DESCRIPTOR);
        let code = prepared
            .method_code(
                jarde_reader::prepared::MethodOrdinal(
                    u32::try_from(initializer_index).expect("method index fits the ordinal"),
                ),
                &mut budget,
            )
            .expect("the same-run initializer code decodes");
        let candidate = EnumMethodCodeCandidate {
            table_index: u64::try_from(initializer_index).expect("method index fits u64"),
            member: None,
            complete: code.stopped_at.is_none()
                && matches!(code.execution, ExecutionReport::Complete { .. })
                && code.exception_handler_count as usize == code.exception_handlers.len(),
            exception_handler_count: code.exception_handler_count,
            instructions: code
                .instructions
                .iter()
                .enumerate()
                .map(|(index, instruction)| {
                    enum_instruction(
                        instruction,
                        code.operands().get(index),
                        &facts.constant_pool,
                        None,
                    )
                })
                .collect(),
            member_uses: Vec::new(),
        };
        let constant_fields = &facts.fields[..2];
        assert_eq!(constructor_index, constant_fields.len());
        let result = prove_initializer_prefix(InitializerPrefixInput {
            code: &candidate,
            owner: b"Stage",
            constants: &[(b"START".to_vec(), 0), (b"FINISH".to_vec(), 1)],
            constructor_calls: &[
                InitializerConstructorCall {
                    descriptor: CTOR_DESCRIPTOR.to_vec(),
                    source_argument: EnumSourceArgument::AnyIntExpression,
                },
                InitializerConstructorCall {
                    descriptor: CTOR_DESCRIPTOR.to_vec(),
                    source_argument: EnumSourceArgument::AnyIntExpression,
                },
            ],
            enum_descriptor: &object_descriptor(b"Stage"),
            fields: constant_fields,
            methods: &facts.methods,
            backing_name: b"$VALUES",
        });
        assert!(
            result.is_ok(),
            "method index must remain in its own table: {result:?}"
        );
        assert_eq!(result.unwrap().prefix_end_bci, 35);
    }

    #[test]
    fn two_constructor_edge_and_terminal_body_project_as_one_source_group() {
        let bytes = compile_java_class("DelegatingEnum", DELEGATING_ENUM_SOURCE, true);
        let report = enum_report(&bytes, "DelegatingEnum", &mut test_budget());
        let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(group)) =
            &report.enum_constant_proof
        else {
            panic!(
                "both constructors, terminal body, and common enum group gates prove: {:?}",
                report.enum_constant_proof
            );
        };
        let body = group
            .constructor_body
            .as_deref()
            .expect("the terminal body joins the complete group proof");
        assert_eq!(body.enum_super_call_bci, 3);
        assert_eq!(body.helper_call_bci, 7);
        assert_eq!(body.field_write_bci, 12);
        assert_eq!(body.return_bci, 15);
        assert!(report.text.contains("ZERO,\n    ONE(1);"));
        assert!(
            report
                .text
                .contains("private DelegatingEnum() {\n        this(0);")
        );
        assert!(report.text.contains("private DelegatingEnum(int arg0) {"));
        assert!(
            report
                .text
                .contains("ConstructorEffects.record(arg0);\n        this.value = arg0;")
        );
        assert!(!report.text.contains("java.lang.String arg1"));
        assert!(!report.text.contains("super(arg1, arg2)"));
        assert!(
            !report
                .text
                .contains("public static final DelegatingEnum ZERO;")
        );
        assert!(!report.text.contains("$VALUES"));
        assert!(!report.text.contains("$values"));
        assert!(!report.text.contains("valueOf("));
        let physical_constructor_descriptors = report
            .methods
            .iter()
            .filter(|method| method.item.name.raw().0 == b"<init>")
            .map(|method| method.item.descriptor.raw().0.as_slice())
            .collect::<Vec<_>>();
        assert_eq!(
            physical_constructor_descriptors,
            [DELEGATING_CTOR_DESCRIPTOR, CTOR_DESCRIPTOR]
        );

        let fixture = delegation_fixture(&bytes);
        let mut budget = test_budget();
        let proof = prove_constructor_delegation_edge(fixture.edge_input(), &mut budget)
            .expect("the exact-edge proof remains within the request budget")
            .expect("the constructor edges prove");
        let delegating_index = fixture.method_headers[proof.delegating_method_index]
            .descriptor
            .raw()
            .0
            .as_slice();
        let terminal_index = fixture.method_headers[proof.terminal_method_index]
            .descriptor
            .raw()
            .0
            .as_slice();
        assert_eq!(delegating_index, DELEGATING_CTOR_DESCRIPTOR);
        assert_eq!(terminal_index, CTOR_DESCRIPTOR);
        assert_eq!(proof.constant_constructor_bcis, [7, 21]);
        assert_eq!(
            proof.constant_constructor_descriptors,
            [
                DELEGATING_CTOR_DESCRIPTOR.to_vec(),
                CTOR_DESCRIPTOR.to_vec()
            ]
        );
        assert_eq!(proof.constant_source_arguments, [None, Some(1)]);
        assert!(budget.usage().ir_items > 0);
    }

    #[test]
    fn delegating_projection_keeps_default_all_json_sources_and_method_only_recovery_aligned() {
        for debug in [true, false] {
            let bytes = compile_java_class("DelegatingEnum", DELEGATING_ENUM_SOURCE, debug);
            let snapshot = crate::Engine::new()
                .open(crate::ArtifactInput::bytes(bytes), &mut test_budget())
                .expect("the enum class opens");
            let request = enum_request(&snapshot, "DelegatingEnum");
            let default = performed(
                crate::Engine::new()
                    .class_source(
                        std::slice::from_ref(&snapshot),
                        &request,
                        &mut test_budget(),
                    )
                    .expect("the default class-source request succeeds"),
            );
            let all = performed(
                crate::Engine::new()
                    .class_source_with_evidence(
                        std::slice::from_ref(&snapshot),
                        &request,
                        &crate::RecoveryEvidenceRequest::all(),
                        &mut test_budget(),
                    )
                    .expect("the complete-evidence class-source request succeeds"),
            );

            assert_eq!(default.text, all.text, "debug={debug}");
            assert!(default.text.contains("ZERO,\n    ONE(1);"));
            assert!(default.text.contains("this(0);"));
            assert!(!default.text.contains("super(arg1, arg2)"));
            assert!(!default.text.contains("String arg1"));

            let default_json =
                serde_json::to_value(&default).expect("the default report serializes");
            let all_json = serde_json::to_value(&all).expect("the all-evidence report serializes");
            let fields = all_json["fields"]
                .as_array()
                .expect("physical fields remain");
            assert_eq!(
                fields
                    .iter()
                    .map(|field| field["item"]["name"]["escaped"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                ["ZERO", "ONE", "value", "$VALUES"]
            );
            assert_eq!(
                fields
                    .iter()
                    .map(|field| field["item"]["index"].as_u64().unwrap())
                    .collect::<Vec<_>>(),
                [0, 1, 2, 3]
            );
            assert!(
                all.fields
                    .iter()
                    .all(|field| field.item.identity.owner == all.class)
            );
            assert_eq!(
                default_json["fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|field| field["item"].clone())
                    .collect::<Vec<_>>(),
                fields
                    .iter()
                    .map(|field| field["item"].clone())
                    .collect::<Vec<_>>()
            );

            let methods = all_json["methods"]
                .as_array()
                .expect("physical methods remain");
            assert_eq!(
                methods
                    .iter()
                    .map(|method| method["item"]["name"]["escaped"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                [
                    "values", "valueOf", "<init>", "<init>", "value", "$values", "<clinit>"
                ]
            );
            assert_eq!(
                all.methods
                    .iter()
                    .filter(|method| method.item.name.raw().0 == b"<init>")
                    .map(|method| method.item.descriptor.raw().0.as_slice())
                    .collect::<Vec<_>>(),
                [DELEGATING_CTOR_DESCRIPTOR, CTOR_DESCRIPTOR]
            );
            assert_eq!(
                methods
                    .iter()
                    .map(|method| method["item"]["index"].as_u64().unwrap())
                    .collect::<Vec<_>>(),
                [0, 1, 2, 3, 4, 5, 6]
            );
            let default_methods = default_json["methods"].as_array().unwrap();
            assert_eq!(default_methods.len(), methods.len());
            for (default_method, method) in default_methods.iter().zip(methods) {
                assert_eq!(default_method["item"], method["item"]);
                assert_eq!(default_method["outcome"]["kind"], method["outcome"]["kind"]);
                assert_eq!(
                    default_method["outcome"]["report"]["text"],
                    method["outcome"]["report"]["text"]
                );
                assert_eq!(method["outcome"]["kind"], "recovered");
                assert_eq!(method["outcome"]["report"]["outcome"], "produced");
                assert_eq!(method["outcome"]["report"]["quality"], "structured");
            }

            let terminal = all
                .methods
                .iter()
                .find(|method| {
                    method.item.name.raw().0 == b"<init>"
                        && method.item.descriptor.raw().0 == CTOR_DESCRIPTOR
                })
                .expect("the physical terminal constructor remains");
            let crate::ClassSourceOutcome::Recovered {
                report: terminal_report,
                ..
            } = &terminal.outcome
            else {
                panic!("the physical terminal constructor keeps its recovered outcome");
            };
            assert!(terminal_report.text.contains("ConstructorEffects.record("));
            assert!(terminal_report.text.contains("this.value = "));
            assert!(
                terminal_report
                    .source_map
                    .text_of_bci(&terminal_report.text, 7)
                    .iter()
                    .any(|text| text.contains("ConstructorEffects.record("))
            );
            assert!(
                terminal_report
                    .source_map
                    .text_of_bci(&terminal_report.text, 12)
                    .iter()
                    .any(|text| text.contains("this.value = "))
            );

            let terminal_json = methods
                .iter()
                .find(|method| {
                    method["item"]["name"]["escaped"] == "<init>"
                        && method["item"]["descriptor"]["escaped"].as_str()
                            == Some("(Ljava/lang/String;II)V")
                })
                .expect("the terminal constructor stays in JSON");
            let source_segments = terminal_json["outcome"]["report"]["source_map"]["segments"]
                .as_array()
                .expect("complete evidence materializes constructor Code origins");
            for bci in [7, 12] {
                assert!(source_segments.iter().any(|segment| {
                    let primary = &segment["origin"]["primary"];
                    primary["bci"].as_u64() == Some(bci)
                        && primary["method"]["name"] == serde_json::json!(b"<init>")
                        && primary["method"]["descriptor"] == serde_json::json!(CTOR_DESCRIPTOR)
                }));
            }
            let delegating_json = methods
                .iter()
                .find(|method| {
                    method["item"]["name"]["escaped"] == "<init>"
                        && method["item"]["descriptor"]["escaped"].as_str()
                            == Some("(Ljava/lang/String;I)V")
                })
                .expect("the delegating constructor stays in JSON");
            assert!(
                delegating_json["outcome"]["report"]["source_map"]["segments"]
                    .as_array()
                    .expect("complete evidence materializes the delegation edge")
                    .iter()
                    .any(|segment| {
                        let primary = &segment["origin"]["primary"];
                        primary["bci"].as_u64() == Some(7)
                            && primary["method"]["descriptor"]
                                == serde_json::json!(DELEGATING_CTOR_DESCRIPTOR)
                    })
            );
            assert!(
                !methods
                    .iter()
                    .any(|method| method["item"]["name"]["escaped"] == "record")
            );

            let value = all
                .methods
                .iter()
                .find(|method| {
                    method.item.name.raw().0 == b"value" && method.item.descriptor.raw().0 == b"()I"
                })
                .expect("the original instance method remains");
            let method_only = crate::Engine::new()
                .recover_method_with_evidence(
                    std::slice::from_ref(&snapshot),
                    crate::SourceTarget::JAVA_8,
                    &crate::ir::MethodAnalysisRequest {
                        environment: request
                            .environment
                            .build(std::slice::from_ref(&snapshot))
                            .expect("the method environment builds from the same request"),
                        method: value.item.identity.clone(),
                        stages: crate::AnalysisStage::ALL.to_vec(),
                    },
                    &crate::RecoveryEvidenceRequest::all(),
                    &mut test_budget(),
                )
                .expect("the method-only request succeeds");
            let crate::ClassSourceOutcome::Recovered {
                report: value_report,
                ..
            } = &value.outcome
            else {
                panic!("the class-source member keeps its original recovered outcome");
            };
            assert_eq!(method_only.recovery().text, value_report.text);
            assert_eq!(method_only.recovery().source_map, value_report.source_map);
        }
    }

    #[test]
    fn projected_delegating_enum_compiles_and_preserves_verified_constructor_behavior() {
        const ORIGINAL: &str = include_str!(
            "../openspec/evidence/java-syntax-2026-09-25/enum-constructor-delegation/source/DelegatingEnum.java"
        );
        const EFFECTS: &str = include_str!(
            "../openspec/evidence/java-syntax-2026-09-25/enum-constructor-delegation/source/ConstructorEffects.java"
        );
        const RUNNER: &str = include_str!(
            "../openspec/evidence/java-syntax-2026-09-25/enum-constructor-delegation/source/EnumRunner.java"
        );
        const EXPECTED: &str = "values=ZERO:0,ONE:1\neffects=2:0,1\ndeclared-constructors=2,3\n";

        for (label, debug) in [("g", true), ("g-none", false)] {
            let original_output = compile_and_run_sources(
                &format!("delegation-original-{label}"),
                &[
                    ("DelegatingEnum.java", ORIGINAL),
                    ("ConstructorEffects.java", EFFECTS),
                    ("EnumRunner.java", RUNNER),
                ],
                debug,
                "EnumRunner",
            );
            assert_eq!(original_output, EXPECTED);

            let bytes = compile_java_class("DelegatingEnum", DELEGATING_ENUM_SOURCE, debug);
            let report = enum_report(&bytes, "DelegatingEnum", &mut test_budget());
            assert!(
                matches!(
                    report.enum_constant_proof,
                    ClassSourceEnumConstantProof::Proved(_)
                ),
                "{label} proof must close before text projection: {:?}",
                report.enum_constant_proof
            );
            let projected_output = compile_and_run_sources(
                &format!("delegation-projected-{label}"),
                &[
                    ("DelegatingEnum.java", &report.text),
                    ("ConstructorEffects.java", EFFECTS),
                    ("EnumRunner.java", RUNNER),
                ],
                debug,
                "EnumRunner",
            );
            assert_eq!(projected_output, EXPECTED, "{label} projected behavior");
        }
    }

    #[test]
    fn terminal_constructor_body_consumes_the_exact_same_run_ast_and_code_sequence() {
        let bytes = compile_java_class("DelegatingEnum", DELEGATING_ENUM_SOURCE, false);
        let fixture = delegation_fixture(&bytes);
        let mut budget = test_budget();
        let body = prove_terminal_constructor_body(fixture.terminal_input(), &mut budget)
            .expect("the terminal-body proof fits the request budget")
            .expect("the exact terminal AST and Code candidates prove");
        assert_eq!(
            body.method_index,
            constructor_index_for(&fixture.method_headers, CTOR_DESCRIPTOR)
        );
        assert_eq!(body.enum_super_call_bci, 3);
        assert_eq!(body.helper_call_bci, 7);
        assert_eq!(body.field_write_bci, 12);
        assert_eq!(body.return_bci, 15);
        assert_eq!(body.helper_owner, b"ConstructorEffects");
        assert_eq!(body.candidate.steps.len(), 4);
        assert!(budget.usage().ir_items > 0);
    }

    #[test]
    fn terminal_constructor_body_rejects_target_parameter_order_signature_effect_and_handler_changes()
     {
        let bytes = compile_java_class("DelegatingEnum", DELEGATING_ENUM_SOURCE, true);
        let baseline = delegation_fixture(&bytes);
        let mut cases: Vec<(&str, DelegationProofFixture)> = Vec::new();

        let mut wrong_helper = baseline.clone();
        let wrong_helper_reference = EnumCodeReference::Method {
            owner: b"ConstructorEffects".to_vec(),
            name: b"other".to_vec(),
            descriptor: b"(I)V".to_vec(),
            interface: false,
        };
        terminal_code_mut(&mut wrong_helper).instructions[5].reference =
            Some(wrong_helper_reference.clone());
        terminal_code_mut(&mut wrong_helper)
            .member_uses
            .iter_mut()
            .find(|use_site| use_site.bci == 7)
            .expect("the helper has one member-use record")
            .reference = wrong_helper_reference;
        cases.push(("wrong helper Methodref", wrong_helper));

        let mut invalid_helper_name = baseline.clone();
        let invalid_helper_reference = EnumCodeReference::Method {
            owner: b"ConstructorEffects".to_vec(),
            name: b"foo/bar".to_vec(),
            descriptor: b"(I)V".to_vec(),
            interface: false,
        };
        terminal_code_mut(&mut invalid_helper_name).instructions[5].reference =
            Some(invalid_helper_reference.clone());
        terminal_code_mut(&mut invalid_helper_name)
            .member_uses
            .iter_mut()
            .find(|use_site| use_site.bci == 7)
            .expect("the helper has one member-use record")
            .reference = invalid_helper_reference;
        let jarde_java::report::ClassEnumConstructorStepKind::Expression(expression) =
            &mut invalid_helper_name.constructor_candidates[0].steps[1].kind
        else {
            panic!("fixture helper expression is retained");
        };
        let jarde_java::ast::ExprKind::Call { name, .. } = &mut expression.kind else {
            panic!("fixture helper AST is a call");
        };
        *name = "foo/bar".to_owned();
        cases.push((
            "helper Methodref and AST use an invalid single-segment Java identifier",
            invalid_helper_name,
        ));

        let mut wrong_field = baseline.clone();
        let wrong_field_reference = EnumCodeReference::Field {
            owner: b"DelegatingEnum".to_vec(),
            name: b"other".to_vec(),
            descriptor: b"I".to_vec(),
        };
        terminal_code_mut(&mut wrong_field).instructions[8].reference =
            Some(wrong_field_reference.clone());
        terminal_code_mut(&mut wrong_field)
            .member_uses
            .iter_mut()
            .find(|use_site| use_site.bci == 12)
            .expect("the field has one member-use record")
            .reference = wrong_field_reference;
        cases.push(("wrong field Fieldref", wrong_field));

        let mut invalid_field_name = baseline.clone();
        let field_index = invalid_field_name
            .source_fields
            .iter()
            .position(|field| field.item.name.raw().0 == b"value")
            .expect("the terminal int field is present");
        let invalid_field_name_bytes = b"bad-name".to_vec();
        let invalid_field_name_text: jarde_reader::model::JvmString =
            serde_json::from_value(serde_json::json!({
                "raw": invalid_field_name_bytes,
                "utf16": invalid_field_name_bytes
                    .iter()
                    .map(|byte| u16::from(*byte))
                    .collect::<Vec<_>>(),
                "escaped": "bad-name",
            }))
            .expect("the ASCII test spelling is a valid raw JVM string");
        invalid_field_name.field_headers[field_index].name = invalid_field_name_text.clone();
        invalid_field_name.source_fields[field_index].item.name = invalid_field_name_text.clone();
        let jarde_reader::model::MemberKey::Field { name, .. } = &mut invalid_field_name
            .source_fields[field_index]
            .item
            .identity
            .member
        else {
            panic!("the class-source identity is a field identity");
        };
        name.0 = invalid_field_name_bytes.clone();
        let invalid_field_reference = EnumCodeReference::Field {
            owner: b"DelegatingEnum".to_vec(),
            name: invalid_field_name_bytes.clone(),
            descriptor: b"I".to_vec(),
        };
        terminal_code_mut(&mut invalid_field_name).instructions[8].reference =
            Some(invalid_field_reference.clone());
        terminal_code_mut(&mut invalid_field_name)
            .member_uses
            .iter_mut()
            .find(|use_site| use_site.bci == 12)
            .expect("the field has one member-use record")
            .reference = invalid_field_reference;
        let jarde_java::report::ClassEnumConstructorStepKind::FieldWrite {
            field: Some(field),
            spelled_name,
            ..
        } = &mut invalid_field_name.constructor_candidates[0].steps[2].kind
        else {
            panic!("fixture field write carries its field@1 claim");
        };
        field.name = crate::JvmBytes::from_unicode("bad-name");
        *spelled_name = "bad-name".to_owned();
        cases.push((
            "Fieldref, field header, and AST use an invalid Java field identifier",
            invalid_field_name,
        ));

        let mut aliased_field = baseline.clone();
        let field_index = aliased_field
            .source_fields
            .iter()
            .position(|field| field.item.name.raw().0 == b"value")
            .expect("the terminal int field is present");
        aliased_field.source_fields[field_index]
            .markers
            .push("field name alias/refusal".to_owned());
        cases.push((
            "field source carries an alias/refusal marker",
            aliased_field,
        ));

        let mut wrong_parameter = baseline.clone();
        terminal_code_mut(&mut wrong_parameter).instructions[4].opcode = 0x1c;
        terminal_code_mut(&mut wrong_parameter).instructions[4].local = Some(2);
        cases.push(("helper reads a different parameter slot", wrong_parameter));

        let mut wrong_order = baseline.clone();
        wrong_order.constructor_candidates[0].steps.swap(1, 2);
        cases.push(("AST statements do not preserve Code order", wrong_order));

        let mut ast_wrong_helper = baseline.clone();
        let jarde_java::report::ClassEnumConstructorStepKind::Expression(expression) =
            &mut ast_wrong_helper.constructor_candidates[0].steps[1].kind
        else {
            panic!("fixture helper expression is retained");
        };
        let jarde_java::ast::ExprKind::Call { name, .. } = &mut expression.kind else {
            panic!("fixture helper AST is a call");
        };
        *name = "other".to_owned();
        cases.push(("AST helper name differs from Methodref", ast_wrong_helper));

        let mut ast_wrong_field = baseline.clone();
        let jarde_java::report::ClassEnumConstructorStepKind::FieldWrite {
            field: Some(field), ..
        } = &mut ast_wrong_field.constructor_candidates[0].steps[2].kind
        else {
            panic!("fixture field write carries its field@1 claim");
        };
        field.name = crate::JvmBytes::from_unicode("other");
        cases.push(("AST field claim differs from Fieldref", ast_wrong_field));

        let mut wrong_terminal_signature = baseline.clone();
        let terminal_index =
            constructor_index_for(&wrong_terminal_signature.method_headers, CTOR_DESCRIPTOR);
        wrong_terminal_signature.source_methods[terminal_index].enum_constructor_source_tail =
            crate::class_source::EnumConstructorSourceTail::Unrecognized;
        cases.push((
            "terminal source Signature has the wrong shape",
            wrong_terminal_signature,
        ));

        let mut wrong_delegating_signature = baseline.clone();
        let delegating_index = constructor_index_for(
            &wrong_delegating_signature.method_headers,
            DELEGATING_CTOR_DESCRIPTOR,
        );
        wrong_delegating_signature.source_methods[delegating_index].enum_constructor_source_tail =
            crate::class_source::EnumConstructorSourceTail::Unrecognized;
        cases.push((
            "delegating source Signature has the wrong shape",
            wrong_delegating_signature,
        ));

        let mut missing_signature = baseline.clone();
        let terminal_index =
            constructor_index_for(&missing_signature.method_headers, CTOR_DESCRIPTOR);
        missing_signature.method_headers[terminal_index]
            .attributes
            .retain(|attribute| attribute.name.raw().0 != b"Signature");
        cases.push(("terminal Signature attribute is missing", missing_signature));

        let mut missing_delegating_signature = baseline.clone();
        let delegating_index = constructor_index_for(
            &missing_delegating_signature.method_headers,
            DELEGATING_CTOR_DESCRIPTOR,
        );
        missing_delegating_signature.method_headers[delegating_index]
            .attributes
            .retain(|attribute| attribute.name.raw().0 != b"Signature");
        cases.push((
            "no-source-argument Signature attribute is missing",
            missing_delegating_signature,
        ));

        let mut extra_code = baseline.clone();
        let code = terminal_code_mut(&mut extra_code);
        code.instructions.insert(
            6,
            EnumCodeInstruction {
                bci: 10,
                width: 1,
                opcode: 0x1d,
                immediate: None,
                local: None,
                increment: None,
                interface_count: None,
                reference: None,
                branch_target_bci: None,
            },
        );
        code.instructions.insert(
            7,
            EnumCodeInstruction {
                bci: 11,
                width: 3,
                opcode: 0xb8,
                immediate: None,
                local: None,
                increment: None,
                interface_count: None,
                reference: Some(EnumCodeReference::Method {
                    owner: b"ConstructorEffects".to_vec(),
                    name: b"audit".to_vec(),
                    descriptor: b"(I)V".to_vec(),
                    interface: false,
                }),
                branch_target_bci: None,
            },
        );
        for instruction in &mut code.instructions[8..] {
            instruction.bci += 4;
        }
        code.member_uses.insert(
            2,
            EnumCodeUse {
                bci: 11,
                reference: EnumCodeReference::Method {
                    owner: b"ConstructorEffects".to_vec(),
                    name: b"audit".to_vec(),
                    descriptor: b"(I)V".to_vec(),
                    interface: false,
                },
            },
        );
        cases.push((
            "well-formed Code includes an extra observable helper call",
            extra_code,
        ));

        let mut extra_ast = baseline.clone();
        let mut extra_step = extra_ast.constructor_candidates[0].steps[3].clone();
        extra_step.order = 4;
        extra_step.bci = 16;
        extra_ast.constructor_candidates[0].steps.push(extra_step);
        cases.push(("terminal AST has an extra statement", extra_ast));

        let mut handler = baseline.clone();
        terminal_code_mut(&mut handler).exception_handler_count = 1;
        cases.push(("terminal Code declares an exception handler", handler));

        let mut ast_handler = baseline.clone();
        ast_handler.constructor_candidates[0].has_exception_handlers = true;
        cases.push((
            "terminal AST sidecar declares an exception handler",
            ast_handler,
        ));

        let mut missing_ast = baseline.clone();
        missing_ast.constructor_candidates.clear();
        cases.push(("terminal AST candidate is missing", missing_ast));

        let mut duplicate_ast = baseline.clone();
        duplicate_ast
            .constructor_candidates
            .push(duplicate_ast.constructor_candidates[0].clone());
        cases.push(("terminal AST candidate is duplicated", duplicate_ast));

        for (label, fixture) in cases {
            let result =
                prove_terminal_constructor_body(fixture.terminal_input(), &mut test_budget())
                    .expect("a refusal is within the request budget");
            assert!(
                result.is_err(),
                "{label} must refuse the terminal body proof"
            );
        }
    }

    #[test]
    fn terminal_constructor_body_propagates_budget_and_cancellation_stops() {
        let bytes = compile_java_class("DelegatingEnum", DELEGATING_ENUM_SOURCE, true);
        let fixture = delegation_fixture(&bytes);
        let mut limits = test_budget().limits().clone();
        limits.ir_items = 0;
        let mut exhausted = Budget::new(limits);
        assert!(prove_terminal_constructor_body(fixture.terminal_input(), &mut exhausted).is_err());

        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(test_budget().limits().clone(), cancellation);
        assert!(prove_terminal_constructor_body(fixture.terminal_input(), &mut cancelled).is_err());
    }

    #[test]
    fn constructor_delegation_edge_rejects_wrong_edges_arguments_effects_handlers_and_duplicates() {
        let bytes = compile_java_class("DelegatingEnum", DELEGATING_ENUM_SOURCE, true);
        let baseline = delegation_fixture(&bytes);
        let mut cases: Vec<(&str, DelegationProofFixture)> = Vec::new();

        let mut wrong_zero_target = baseline.clone();
        let call = initializer_candidate_mut(&mut wrong_zero_target)
            .instructions
            .get_mut(4)
            .unwrap();
        call.reference = Some(EnumCodeReference::Method {
            owner: b"DelegatingEnum".to_vec(),
            name: b"<init>".to_vec(),
            descriptor: CTOR_DESCRIPTOR.to_vec(),
            interface: false,
        });
        cases.push(("ZERO chooses the wrong overload", wrong_zero_target));

        let mut wrong_one_target = baseline.clone();
        let call = initializer_candidate_mut(&mut wrong_one_target)
            .instructions
            .get_mut(11)
            .unwrap();
        call.reference = Some(EnumCodeReference::Method {
            owner: b"DelegatingEnum".to_vec(),
            name: b"<init>".to_vec(),
            descriptor: DELEGATING_CTOR_DESCRIPTOR.to_vec(),
            interface: false,
        });
        cases.push(("ONE chooses the wrong overload", wrong_one_target));

        let mut changed_name = baseline.clone();
        let bridge = constructor_candidate_mut(&mut changed_name, DELEGATING_CTOR_DESCRIPTOR);
        bridge.instructions[1].opcode = 0x01;
        bridge.instructions[1].local = None;
        cases.push(("the bridge replaces name", changed_name));

        let mut changed_ordinal = baseline.clone();
        let bridge = constructor_candidate_mut(&mut changed_ordinal, DELEGATING_CTOR_DESCRIPTOR);
        bridge.instructions[2].opcode = 0x04;
        cases.push(("the bridge replaces ordinal", changed_ordinal));

        let mut changed_zero = baseline.clone();
        let bridge = constructor_candidate_mut(&mut changed_zero, DELEGATING_CTOR_DESCRIPTOR);
        bridge.instructions[3].opcode = 0x04;
        cases.push(("the bridge changes literal zero", changed_zero));

        let mut changed_one_argument = baseline.clone();
        initializer_candidate_mut(&mut changed_one_argument).instructions[10].opcode = 0x05;
        cases.push((
            "ONE changes its direct integer argument",
            changed_one_argument,
        ));

        let mut extra_effect = baseline.clone();
        let bridge = constructor_candidate_mut(&mut extra_effect, DELEGATING_CTOR_DESCRIPTOR);
        let return_instruction = bridge.instructions.pop().unwrap();
        bridge.instructions.push(EnumCodeInstruction {
            bci: return_instruction.bci,
            width: 3,
            opcode: 0xb8,
            immediate: None,
            local: None,
            increment: None,
            interface_count: None,
            reference: Some(EnumCodeReference::Method {
                owner: b"ConstructorEffects".to_vec(),
                name: b"record".to_vec(),
                descriptor: b"(I)V".to_vec(),
                interface: false,
            }),
            branch_target_bci: None,
        });
        let mut shifted_return = return_instruction;
        shifted_return.bci += 3;
        bridge.instructions.push(shifted_return);
        cases.push(("the bridge performs an extra call", extra_effect));

        let mut branch = baseline.clone();
        constructor_candidate_mut(&mut branch, DELEGATING_CTOR_DESCRIPTOR).instructions[5].opcode =
            0xa7;
        cases.push(("the bridge contains a branch", branch));

        let mut bridge_handler = baseline.clone();
        constructor_candidate_mut(&mut bridge_handler, DELEGATING_CTOR_DESCRIPTOR)
            .exception_handler_count = 1;
        cases.push(("the bridge has an exception handler", bridge_handler));

        let mut initializer_handler = baseline.clone();
        initializer_candidate_mut(&mut initializer_handler).exception_handler_count = 1;
        cases.push((
            "the initializer has an exception handler",
            initializer_handler,
        ));

        let mut initializer_branch = baseline.clone();
        initializer_candidate_mut(&mut initializer_branch)
            .instructions
            .last_mut()
            .unwrap()
            .opcode = 0xa7;
        cases.push(("the initializer contains a branch", initializer_branch));

        let mut duplicate_member = baseline.clone();
        duplicate_member.method_headers.push(
            duplicate_member.method_headers[constructor_index_for(
                &duplicate_member.method_headers,
                DELEGATING_CTOR_DESCRIPTOR,
            )]
            .clone(),
        );
        cases.push(("the method table repeats a constructor", duplicate_member));

        let mut duplicate_candidate = baseline.clone();
        let candidate_index = duplicate_candidate
            .code_candidates
            .iter()
            .position(|candidate| {
                candidate.member.as_ref().is_some_and(|member| {
                    member.name.0 == b"<init>" && member.descriptor.0 == DELEGATING_CTOR_DESCRIPTOR
                })
            })
            .unwrap();
        duplicate_candidate
            .code_candidates
            .push(duplicate_candidate.code_candidates[candidate_index].clone());
        cases.push((
            "the candidate table repeats a constructor",
            duplicate_candidate,
        ));

        for (case, fixture) in cases {
            let mut budget = test_budget();
            let result = prove_constructor_delegation_edge(fixture.edge_input(), &mut budget)
                .expect("a semantic refusal is not a request stop");
            assert!(result.is_err(), "{case} must refuse the entire edge proof");
        }
    }

    #[test]
    fn constructor_delegation_edge_preserves_budget_and_cancellation_stops() {
        let bytes = compile_java_class("DelegatingEnum", DELEGATING_ENUM_SOURCE, true);
        let fixture = delegation_fixture(&bytes);
        let mut limits = test_budget().limits().clone();
        limits.ir_items = 0;
        let mut exhausted = Budget::new(limits);
        assert!(prove_constructor_delegation_edge(fixture.edge_input(), &mut exhausted,).is_err());

        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(test_budget().limits().clone(), cancellation);
        assert!(prove_constructor_delegation_edge(fixture.edge_input(), &mut cancelled,).is_err());
    }

    #[test]
    fn helper_constructor_and_prefix_1_2_boundaries_are_refused() {
        let cases: [(&str, &[u8]); 4] = [
            (
                "values-order",
                include_bytes!(
                    "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/helper-constructor-prefix-boundaries/generated/patched-values-order/Measure.class"
                ),
            ),
            (
                "valueof-null-argument",
                include_bytes!(
                    "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/helper-constructor-prefix-boundaries/generated/patched-valueof-null-argument/Measure.class"
                ),
            ),
            (
                "constructor-shape",
                include_bytes!(
                    "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/helper-constructor-prefix-boundaries/generated/input/constructor-shape/Measure.class"
                ),
            ),
            (
                "prefix-effect",
                include_bytes!(
                    "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/helper-constructor-prefix-boundaries/generated/input/prefix-effect/Measure.class"
                ),
            ),
        ];
        for (case, bytes) in cases {
            let report = enum_report(bytes, "Measure", &mut test_budget());
            assert!(
                matches!(
                    report.enum_constant_proof,
                    ClassSourceEnumConstantProof::Refused { .. }
                ),
                "the {case} variant must refuse the complete group, got {:?}",
                report.enum_constant_proof
            );
            assert!(report.text.contains("public static final Measure LOW;"));
            assert!(report.text.contains("Measure.$VALUES = $values();"));
            assert!(!report.text.contains("LOW(2),"));
        }
    }

    #[test]
    fn extra_user_read_of_values_backing_array_refuses_and_stays_visible() {
        let bytes = compile_and_patch_raw_values_read();
        let report = enum_report(&bytes, "E", &mut test_budget());
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Refused { .. }
        ));
        assert!(report.text.contains("public static final E A;"));
        assert!(report.text.contains("public static final E B;"));
        assert!(report.text.contains("return E.$VALUES;"), "{}", report.text);
    }

    #[test]
    fn projection_output_budget_stop_keeps_the_original_class_source() {
        let complete = enum_report(STAGE, "Stage", &mut test_budget());
        let cap = complete
            .usage
            .output_bytes
            .checked_sub(1)
            .expect("the generated projection charged output bytes");
        let mut limits = test_budget().limits().clone();
        limits.output_bytes = cap;
        let report = enum_report(STAGE, "Stage", &mut Budget::new(limits));

        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Stopped { .. }
        ));
        assert!(!matches!(
            report.execution,
            ExecutionReport::Complete { .. }
        ));
        assert!(report.text.contains("public static final Stage START;"));
        assert!(report.text.contains("Stage.$VALUES = $values();"));
        assert!(!report.text.contains("START(4),"));
    }

    #[test]
    fn delegating_projection_output_budget_stop_keeps_both_physical_constructors() {
        let bytes = compile_java_class("DelegatingEnum", DELEGATING_ENUM_SOURCE, true);
        let complete = enum_report(&bytes, "DelegatingEnum", &mut test_budget());
        assert!(matches!(
            complete.enum_constant_proof,
            ClassSourceEnumConstantProof::Proved(_)
        ));
        let cap = complete
            .usage
            .output_bytes
            .checked_sub(1)
            .expect("the complete enum projection charged output bytes");
        let mut limits = test_budget().limits().clone();
        limits.output_bytes = cap;
        let report = enum_report(&bytes, "DelegatingEnum", &mut Budget::new(limits));

        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Stopped { .. }
        ));
        assert!(!matches!(
            report.execution,
            ExecutionReport::Complete { .. }
        ));
        assert!(
            report
                .text
                .contains("public static final DelegatingEnum ZERO;")
        );
        assert!(report.text.contains("$VALUES"));
        assert!(report.text.contains("$values"));
        assert!(
            report
                .text
                .contains("private DelegatingEnum(java.lang.String arg1, int arg2)")
        );
        assert!(
            report
                .text
                .contains("private DelegatingEnum(java.lang.String arg1, int arg2, int value)")
        );
        assert!(!report.text.contains("ZERO,\n    ONE(1);"));
        assert!(!report.text.contains("private DelegatingEnum() {"));
    }

    #[test]
    fn delegating_class_source_stops_do_not_publish_a_partial_projection() {
        fn assert_no_partial_projection(
            outcome: crate::OperationOutcome<crate::ClassSourceReport>,
        ) {
            match outcome {
                crate::OperationOutcome::Performed(report) => {
                    assert!(!matches!(
                        &report.enum_constant_proof,
                        ClassSourceEnumConstantProof::Proved(_)
                    ));
                    assert!(!report.text.contains("ZERO,\n    ONE(1);"));
                    assert!(!report.text.contains("private DelegatingEnum() {"));
                }
                crate::OperationOutcome::Incomplete(candidates) => assert!(!matches!(
                    candidates.execution,
                    ExecutionReport::Complete { .. }
                )),
                crate::OperationOutcome::Ambiguous(candidates) => {
                    panic!("the fixture has one physical enum definition: {candidates:?}")
                }
            }
        }

        let bytes = compile_java_class("DelegatingEnum", DELEGATING_ENUM_SOURCE, true);
        let snapshot = crate::Engine::new()
            .open(crate::ArtifactInput::bytes(bytes), &mut test_budget())
            .expect("the enum class opens");
        let request = enum_request(&snapshot, "DelegatingEnum");

        let mut ir_limits = test_budget().limits().clone();
        ir_limits.ir_items = 0;
        let result = crate::Engine::new().class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::new(ir_limits),
        );
        match result {
            Ok(outcome) => assert_no_partial_projection(outcome),
            Err(crate::Error::BudgetExceeded { dimension, .. }) => {
                assert_eq!(dimension, crate::BudgetDimension::IrItems)
            }
            Err(error) => panic!("unexpected error from the IR budget stop: {error}"),
        }

        let mut elapsed_limits = test_budget().limits().clone();
        elapsed_limits.elapsed_millis = 0;
        let result = crate::Engine::new().class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::new(elapsed_limits),
        );
        match result {
            Ok(outcome) => assert_no_partial_projection(outcome),
            Err(crate::Error::BudgetExceeded { dimension, .. }) => {
                assert_eq!(dimension, crate::BudgetDimension::ElapsedMillis)
            }
            Err(error) => panic!("unexpected error from the elapsed budget stop: {error}"),
        }

        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(test_budget().limits().clone(), cancellation);
        match crate::Engine::new().class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut cancelled,
        ) {
            Ok(crate::OperationOutcome::Incomplete(candidates)) => assert!(matches!(
                candidates.execution,
                ExecutionReport::Cancelled { .. }
            )),
            Err(crate::Error::Cancelled { .. }) => {}
            Ok(outcome) => {
                assert_no_partial_projection(outcome);
                panic!("cancelled dual-constructor class selection was not stopped")
            }
            Err(error) => panic!("unexpected error from cancellation: {error}"),
        }
    }

    #[test]
    fn measure_suffix_output_budget_stop_keeps_the_entire_unprojected_initializer() {
        let complete = enum_report(MEASURE, "Measure", &mut test_budget());
        assert!(
            complete
                .text
                .contains("static {\n        totalUnits = sumUnits();")
        );
        let cap = complete
            .usage
            .output_bytes
            .checked_sub(1)
            .expect("the projected suffix charged output bytes");
        let mut limits = test_budget().limits().clone();
        limits.output_bytes = cap;
        let report = enum_report(MEASURE, "Measure", &mut Budget::new(limits));
        assert!(matches!(
            report.enum_constant_proof,
            ClassSourceEnumConstantProof::Stopped { .. }
        ));
        assert!(!matches!(
            report.execution,
            ExecutionReport::Complete { .. }
        ));
        assert!(report.text.contains("public static final Measure LOW;"));
        assert!(report.text.contains("Measure.$VALUES = $values();"));
        assert!(report.text.contains("Measure.totalUnits = sumUnits();"));
        assert!(!report.text.contains("LOW(2),"));
        let json = serde_json::to_value(&report).expect("the stopped report serializes");
        assert_eq!(json["fields"].as_array().unwrap().len(), 5);
        assert_eq!(json["methods"].as_array().unwrap().len(), 6);
        let initializer = json["methods"]
            .as_array()
            .unwrap()
            .iter()
            .find(|method| method["item"]["name"]["escaped"] == "<clinit>")
            .expect("the physical initializer remains");
        assert_eq!(initializer["outcome"]["kind"], "recovered");
        assert!(
            initializer["outcome"]["report"]["text"]
                .as_str()
                .unwrap()
                .contains("Measure.totalUnits = sumUnits();")
        );
        let initializer = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"<clinit>")
            .expect("the physical initializer stays indexed");
        assert_eq!(
            initializer.item.identity.name.0,
            initializer.item.name.raw().0
        );
        assert_eq!(
            initializer.item.identity.descriptor.0,
            initializer.item.descriptor.raw().0
        );
    }

    #[test]
    fn exhausted_proof_budget_is_reported_as_stopped() {
        let snapshot = crate::Engine::new()
            .open(
                crate::ArtifactInput::bytes(STAGE.to_vec()),
                &mut test_budget(),
            )
            .expect("the enum class opens");
        let request = enum_request(&snapshot, "Stage");
        let mut limits = test_budget().limits().clone();
        limits.ir_items = 0;
        let mut budget = Budget::new(limits);
        let report = performed(
            crate::Engine::new()
                .class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
                .expect("the report carries a bounded stop"),
        );
        assert!(
            matches!(
                report.enum_constant_proof,
                ClassSourceEnumConstantProof::Stopped { .. }
            ),
            "budget stop became {:?}",
            report.enum_constant_proof
        );
        assert!(!matches!(
            report.execution,
            ExecutionReport::Complete { .. }
        ));
    }

    #[test]
    fn cancellation_is_not_converted_into_a_normal_enum_refusal() {
        let snapshot = crate::Engine::new()
            .open(
                crate::ArtifactInput::bytes(STAGE.to_vec()),
                &mut test_budget(),
            )
            .expect("the enum class opens");
        let request = enum_request(&snapshot, "Stage");
        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(test_budget().limits().clone(), cancellation);
        match crate::Engine::new()
            .class_source(std::slice::from_ref(&snapshot), &request, &mut cancelled)
            .expect("cancellation is carried by class selection's stop plane")
        {
            crate::OperationOutcome::Incomplete(candidates) => assert!(matches!(
                candidates.execution,
                ExecutionReport::Cancelled { .. }
            )),
            other => panic!("cancelled selection must not produce an enum proof: {other:?}"),
        }
    }

    #[test]
    fn cancellation_of_the_measure_request_never_publishes_a_partial_enum_projection() {
        let snapshot = crate::Engine::new()
            .open(
                crate::ArtifactInput::bytes(MEASURE.to_vec()),
                &mut test_budget(),
            )
            .expect("the Measure class opens");
        let request = enum_request(&snapshot, "Measure");
        let cancellation = crate::CancellationToken::new();
        cancellation.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(test_budget().limits().clone(), cancellation);
        match crate::Engine::new()
            .class_source(std::slice::from_ref(&snapshot), &request, &mut cancelled)
            .expect("cancellation is stated in the class-source stop plane")
        {
            crate::OperationOutcome::Incomplete(candidates) => assert!(matches!(
                candidates.execution,
                ExecutionReport::Cancelled { .. }
            )),
            other => panic!("cancelled Measure selection must not publish a projection: {other:?}"),
        }
    }

    #[derive(Clone)]
    struct DelegationProofFixture {
        owner: Vec<u8>,
        constants: Vec<(usize, String, i32)>,
        backing_name: Vec<u8>,
        field_headers: Vec<MemberHeader>,
        source_fields: Vec<ClassSourceField>,
        method_headers: Vec<MemberHeader>,
        source_methods: Vec<ClassSourceMethod>,
        code_candidates: Vec<EnumMethodCodeCandidate>,
        constructor_candidates: Vec<jarde_java::report::ClassEnumConstructorCandidates>,
    }

    impl DelegationProofFixture {
        fn edge_input(&self) -> DelegationEdgeInput<'_> {
            DelegationEdgeInput {
                owner: &self.owner,
                constants: &self.constants,
                backing_name: &self.backing_name,
                field_headers: &self.field_headers,
                method_headers: &self.method_headers,
                source_methods: &self.source_methods,
                code_candidates: &self.code_candidates,
            }
        }

        fn terminal_input(&self) -> TerminalConstructorBodyInput<'_> {
            TerminalConstructorBodyInput {
                owner: &self.owner,
                method_index: constructor_index_for(&self.method_headers, CTOR_DESCRIPTOR),
                field_headers: &self.field_headers,
                source_fields: &self.source_fields,
                method_headers: &self.method_headers,
                source_methods: &self.source_methods,
                code_candidates: &self.code_candidates,
                constructor_candidates: &self.constructor_candidates,
            }
        }
    }

    fn delegation_fixture(bytes: &[u8]) -> DelegationProofFixture {
        let snapshot = crate::Engine::new()
            .open(
                crate::ArtifactInput::bytes(bytes.to_vec()),
                &mut test_budget(),
            )
            .expect("the enum class opens");
        let request = enum_request(&snapshot, "DelegatingEnum");
        let report = performed(
            crate::Engine::new()
                .class_source(
                    std::slice::from_ref(&snapshot),
                    &request,
                    &mut test_budget(),
                )
                .expect("the class-source member run succeeds"),
        );
        let source_methods = report.methods.clone();
        let source_fields = report.fields.clone();
        let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(group)) =
            &report.enum_constant_proof
        else {
            panic!(
                "the class-source run proves a terminal constructor body: {:?}",
                report.enum_constant_proof
            );
        };
        let body = group
            .constructor_body
            .as_deref()
            .expect("the proved group carries the terminal body");
        let constructor_candidates = vec![body.candidate.clone()];

        let mut budget = test_budget();
        let read = snapshot
            .prepared_root_class(&mut budget)
            .expect("the same class read prepares");
        let prepared = jarde_reader::prepared::PreparedClass::prepare(&read, &mut budget)
            .expect("the same class member tables prepare");
        let facts = prepared.class_facts();
        let mut code_candidates = Vec::new();
        for (index, header) in facts.methods.iter().enumerate().filter(|(_, header)| {
            header.name.raw().0 == b"<init>" || header.name.raw().0 == b"<clinit>"
        }) {
            budget
                .poll()
                .expect("the test candidate scan is not cancelled");
            let ordinal = jarde_reader::prepared::MethodOrdinal(
                u32::try_from(index).expect("method table index fits its ordinal"),
            );
            let code = prepared
                .method_code(ordinal, &mut budget)
                .expect("the selected constructor or initializer Code decodes");
            let instructions = code
                .instructions
                .iter()
                .enumerate()
                .map(|(instruction_index, instruction)| {
                    enum_instruction(
                        instruction,
                        code.operands().get(instruction_index),
                        &facts.constant_pool,
                        None,
                    )
                })
                .collect::<Vec<_>>();
            let member_uses = instructions
                .iter()
                .filter_map(|instruction| {
                    instruction
                        .reference
                        .as_ref()
                        .filter(|reference| {
                            matches!(
                                reference,
                                EnumCodeReference::Field { .. } | EnumCodeReference::Method { .. }
                            )
                        })
                        .map(|reference| EnumCodeUse {
                            bci: instruction.bci,
                            reference: reference.clone(),
                        })
                })
                .collect();
            code_candidates.push(EnumMethodCodeCandidate {
                table_index: u64::try_from(index).expect("method table index fits u64"),
                member: Some(source_methods[index].item.identity.clone()),
                complete: code.stopped_at.is_none()
                    && matches!(code.execution, ExecutionReport::Complete { .. })
                    && code.exception_handler_count as usize == code.exception_handlers.len()
                    && code.instructions.len() == code.operands().len(),
                exception_handler_count: code.exception_handler_count,
                instructions,
                member_uses,
            });
            let _ = header;
        }
        DelegationProofFixture {
            owner: facts.this_class.raw().0.clone(),
            constants: facts
                .fields
                .iter()
                .enumerate()
                .filter(|(_, field)| field.access_flags & ACC_ENUM != 0)
                .enumerate()
                .map(|(ordinal, (index, field))| {
                    (
                        index,
                        String::from_utf16(field.name.utf16())
                            .expect("fixture enum field names are Java text"),
                        i32::try_from(ordinal).expect("two enum ordinals fit i32"),
                    )
                })
                .collect(),
            backing_name: facts
                .fields
                .iter()
                .find(|field| field.name.raw().0 == b"$VALUES")
                .expect("fixture backing field is present")
                .name
                .raw()
                .0
                .clone(),
            field_headers: facts.fields.clone(),
            source_fields,
            method_headers: facts.methods.clone(),
            source_methods,
            code_candidates,
            constructor_candidates,
        }
    }

    fn constructor_index_for(headers: &[MemberHeader], descriptor: &[u8]) -> usize {
        headers
            .iter()
            .position(|header| {
                header.name.raw().0 == b"<init>" && header.descriptor.raw().0 == descriptor
            })
            .expect("the selected constructor is present")
    }

    fn constructor_candidate_mut<'a>(
        fixture: &'a mut DelegationProofFixture,
        descriptor: &[u8],
    ) -> &'a mut EnumMethodCodeCandidate {
        fixture
            .code_candidates
            .iter_mut()
            .find(|candidate| {
                candidate.member.as_ref().is_some_and(|member| {
                    member.name.0 == b"<init>" && member.descriptor.0 == descriptor
                })
            })
            .expect("the selected constructor candidate is present")
    }

    fn terminal_code_mut(fixture: &mut DelegationProofFixture) -> &mut EnumMethodCodeCandidate {
        constructor_candidate_mut(fixture, CTOR_DESCRIPTOR)
    }

    fn initializer_candidate_mut(
        fixture: &mut DelegationProofFixture,
    ) -> &mut EnumMethodCodeCandidate {
        fixture
            .code_candidates
            .iter_mut()
            .find(|candidate| {
                candidate
                    .member
                    .as_ref()
                    .is_some_and(|member| member.name.0 == b"<clinit>")
            })
            .expect("the class initializer candidate is present")
    }

    fn test_budget() -> Budget {
        let mut limits = crate::task_budget(&[])
            .expect("the default task budget is valid")
            .limits()
            .clone();
        limits.input_bytes = u64::MAX;
        limits.archive_entries = u64::MAX;
        limits.entry_bytes = u64::MAX;
        limits.read_bytes = u64::MAX;
        limits.class_bytes = u64::MAX;
        limits.attribute_bytes = u64::MAX;
        limits.code_bytes = u64::MAX;
        limits.result_items = u64::MAX;
        limits.output_bytes = u64::MAX;
        limits.class_headers = u64::MAX;
        limits.method_bodies = u64::MAX;
        limits.ir_items = u64::MAX;
        limits.ir_edges = u64::MAX;
        limits.analysis_steps = u64::MAX;
        limits.normalization_clones = u64::MAX;
        limits.elapsed_millis = u64::MAX;
        Budget::new(limits)
    }

    fn enum_request(snapshot: &crate::ArtifactSnapshot, name: &str) -> crate::ClassSourceRequest {
        crate::ClassSourceRequest {
            source_target: crate::SourceTarget::JAVA_8,
            class: crate::ClassRef::Name {
                class: crate::ClassNameQuery::internal(name),
            },
            environment: crate::EnvironmentRequest {
                snapshot: snapshot.id().clone(),
                scope: crate::PhysicalScope::SnapshotAll,
                policy: crate::EnvironmentPolicy::SingleClass,
                profile: crate::RuntimeProfile {
                    java_release: 8,
                    multi_release: crate::MultiReleasePolicy::Disabled,
                    layout: crate::LayoutMode::Generic,
                },
                loader: crate::LoaderId("app".to_owned()),
            },
        }
    }

    fn enum_report(bytes: &[u8], name: &str, budget: &mut Budget) -> crate::ClassSourceReport {
        let snapshot = crate::Engine::new()
            .open(
                crate::ArtifactInput::bytes(bytes.to_vec()),
                &mut test_budget(),
            )
            .expect("the enum class opens");
        let request = enum_request(&snapshot, name);
        performed(
            crate::Engine::new()
                .class_source(std::slice::from_ref(&snapshot), &request, budget)
                .expect("the class-source request succeeds"),
        )
    }

    fn enum_report_with_dependencies(
        bytes: &[u8],
        name: &str,
        dependencies: &[Vec<u8>],
        budget: &mut Budget,
    ) -> crate::ClassSourceReport {
        let engine = crate::Engine::new();
        let mut snapshots = vec![
            engine
                .open(
                    crate::ArtifactInput::bytes(bytes.to_vec()),
                    &mut test_budget(),
                )
                .expect("the enum class opens"),
        ];
        for dependency in dependencies {
            snapshots.push(
                engine
                    .open(
                        crate::ArtifactInput::bytes(dependency.clone()),
                        &mut test_budget(),
                    )
                    .expect("the dependency class opens"),
            );
        }
        let mut request = enum_request(&snapshots[0], name);
        request.environment.policy = crate::EnvironmentPolicy::ExplicitClasspath {
            roots: snapshots
                .iter()
                .map(|snapshot| crate::LoadRoot::StandaloneClass {
                    snapshot: snapshot.id().clone(),
                })
                .collect(),
        };
        performed(
            engine
                .class_source(&snapshots, &request, budget)
                .expect("the class-source request succeeds"),
        )
    }

    fn replace_constant_pool_utf8(bytes: &mut [u8], from: &[u8], to: &[u8]) {
        assert_eq!(
            from.len(),
            to.len(),
            "the fixture mutation preserves the constant length"
        );
        let constant_count = usize::from(u16::from_be_bytes([bytes[8], bytes[9]]));
        let mut index = 1;
        let mut cursor = 10;
        let mut replacements = 0;
        while index < constant_count {
            let tag = bytes[cursor];
            cursor += 1;
            match tag {
                1 => {
                    let length =
                        usize::from(u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]));
                    cursor += 2;
                    if &bytes[cursor..cursor + length] == from {
                        bytes[cursor..cursor + length].copy_from_slice(to);
                        replacements += 1;
                    }
                    cursor += length;
                }
                3 | 4 => cursor += 4,
                5 | 6 => {
                    cursor += 8;
                    index += 1;
                }
                7 | 8 | 16 | 19 | 20 => cursor += 2,
                9 | 10 | 11 | 12 | 17 | 18 => cursor += 4,
                15 => cursor += 3,
                other => panic!("unknown constant-pool tag in fixture: {other}"),
            }
            index += 1;
        }
        assert_eq!(
            replacements, 1,
            "the Signature spelling is unique in this fixture"
        );
    }

    fn remove_method_attribute(
        bytes: &mut Vec<u8>,
        method_name: &[u8],
        descriptor: &[u8],
        attribute_name: &[u8],
    ) {
        let read_u16 = |bytes: &[u8], offset: usize| {
            usize::from(u16::from_be_bytes([bytes[offset], bytes[offset + 1]]))
        };
        let constant_count = read_u16(bytes, 8);
        let mut utf8 = vec![None; constant_count];
        let mut index = 1;
        let mut cursor = 10;
        while index < constant_count {
            let tag = bytes[cursor];
            cursor += 1;
            match tag {
                1 => {
                    let length = read_u16(bytes, cursor);
                    cursor += 2;
                    utf8[index] = Some(bytes[cursor..cursor + length].to_vec());
                    cursor += length;
                }
                3 | 4 => cursor += 4,
                5 | 6 => {
                    cursor += 8;
                    index += 1;
                }
                7 | 8 | 16 | 19 | 20 => cursor += 2,
                9 | 10 | 11 | 12 | 17 | 18 => cursor += 4,
                15 => cursor += 3,
                other => panic!("unknown fixture constant-pool tag: {other}"),
            }
            index += 1;
        }
        let member_end = |bytes: &[u8], mut offset: usize| {
            let count_at = offset + 6;
            let count = read_u16(bytes, count_at);
            offset += 8;
            for _ in 0..count {
                let length = usize::try_from(u32::from_be_bytes(
                    bytes[offset + 2..offset + 6].try_into().unwrap(),
                ))
                .unwrap();
                offset += 6 + length;
            }
            (offset, count_at)
        };
        cursor += 6;
        let interfaces = read_u16(bytes, cursor);
        cursor += 2 + interfaces * 2;
        let fields = read_u16(bytes, cursor);
        cursor += 2;
        for _ in 0..fields {
            cursor = member_end(bytes, cursor).0;
        }
        let methods = read_u16(bytes, cursor);
        cursor += 2;
        for _ in 0..methods {
            let start = cursor;
            let name_index = read_u16(bytes, start + 2);
            let descriptor_index = read_u16(bytes, start + 4);
            let attribute_count_offset = start + 6;
            let attribute_count = read_u16(bytes, attribute_count_offset);
            cursor += 8;
            for _ in 0..attribute_count {
                let attribute_start = cursor;
                let attribute_index = read_u16(bytes, cursor);
                let length = usize::try_from(u32::from_be_bytes(
                    bytes[cursor + 2..cursor + 6].try_into().unwrap(),
                ))
                .unwrap();
                cursor += 6 + length;
                if utf8[name_index].as_deref() == Some(method_name)
                    && utf8[descriptor_index].as_deref() == Some(descriptor)
                    && utf8[attribute_index].as_deref() == Some(attribute_name)
                {
                    let count = read_u16(bytes, attribute_count_offset);
                    bytes.drain(attribute_start..cursor);
                    bytes[attribute_count_offset..attribute_count_offset + 2]
                        .copy_from_slice(&u16::try_from(count - 1).unwrap().to_be_bytes());
                    return;
                }
            }
        }
        panic!("the target method attribute is absent");
    }

    fn verify_classfile_without_initialization(class_name: &str, classfile: &[u8]) {
        let (directory, _cleanup) = java_test_directory("class-verification");
        fs::write(directory.join(format!("{class_name}.class")), classfile)
            .expect("the mutated classfile is written");
        fs::write(
            directory.join("VerifyClass.java"),
            "public final class VerifyClass { public static void main(String[] args) throws Exception { Class.forName(args[0], false, VerifyClass.class.getClassLoader()); System.out.println(\"VERIFIED \" + args[0]); } }",
        )
        .expect("the verification helper source is written");
        let compile = Command::new("javac")
            .args(["--release", "8", "-g:none", "-d"])
            .arg(&directory)
            .arg(directory.join("VerifyClass.java"))
            .output()
            .expect("the Java 8 compiler is available");
        assert!(
            compile.status.success(),
            "verification helper compiles: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let output = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(&directory)
            .args(["VerifyClass", class_name])
            .output()
            .expect("the Java 8 runtime is available");
        assert!(
            output.status.success(),
            "class verifies without initialization: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            format!("VERIFIED {class_name}\n")
        );
    }

    fn compile_java_class(class_name: &str, source_text: &str, debug: bool) -> Vec<u8> {
        let (directory, _cleanup) = java_test_directory(&format!("input-{class_name}"));
        let source = directory.join(format!("{class_name}.java"));
        fs::write(&source, source_text).expect("the source fixture is written");
        let mut command = Command::new("javac");
        command.arg("--release").arg("8");
        command.arg(if debug { "-g" } else { "-g:none" });
        let output = command
            .arg("-d")
            .arg(&directory)
            .arg(&source)
            .output()
            .expect("the Java 8 fixture compiler is available");
        assert!(
            output.status.success(),
            "javac failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::read(directory.join(format!("{class_name}.class")))
            .expect("the Java fixture class was emitted")
    }

    fn compile_java_sources(label: &str, sources: &[(&str, &str)]) -> Vec<Vec<u8>> {
        let (directory, _cleanup) = java_test_directory(label);
        let mut command = Command::new("javac");
        command
            .arg("--release")
            .arg("8")
            .arg("-g:none")
            .arg("-d")
            .arg(&directory);
        for (name, source_text) in sources {
            let source = directory.join(format!("{name}.java"));
            if let Some(parent) = source.parent() {
                fs::create_dir_all(parent).expect("the Java fixture package is created");
            }
            fs::write(&source, source_text).expect("the fixture source is written");
            command.arg(source);
        }
        let output = command
            .output()
            .expect("the Java 8 fixture compiler is available");
        assert!(
            output.status.success(),
            "javac failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        sources
            .iter()
            .map(|(name, _)| {
                fs::read(directory.join(format!("{name}.class")))
                    .expect("the fixture class was emitted")
            })
            .collect()
    }

    fn compile_frozen_enum(class_name: &str, source_text: &str, debug: bool) -> Vec<u8> {
        let (directory, _cleanup) = java_test_directory(&format!("frozen-enum-{class_name}"));
        fs::create_dir_all(&directory).expect("the private fixture directory is created");
        let source = directory.join(format!("{class_name}.java"));
        fs::write(&source, source_text).expect("the frozen Java source is written");
        let mut command = Command::new("javac");
        command.arg("--release").arg("8");
        command.arg(if debug { "-g" } else { "-g:none" });
        let output = command
            .arg("-d")
            .arg(&directory)
            .arg(&source)
            .output()
            .expect("the Java 8 fixture compiler is available");
        assert!(
            output.status.success(),
            "javac failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::read(directory.join("demo").join(format!("{class_name}.class")))
            .expect("the packaged Java fixture class was emitted")
    }

    fn compile_and_run_sources(
        label: &str,
        sources: &[(&str, &str)],
        debug: bool,
        main_class: &str,
    ) -> String {
        let (directory, _cleanup) = java_test_directory(label);
        let mut paths = Vec::with_capacity(sources.len());
        for (relative_path, contents) in sources {
            let path = directory.join(relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("the Java package directory is created");
            }
            fs::write(&path, contents).expect("the Java source is written");
            paths.push(path);
        }
        let mut command = Command::new("javac");
        command.arg("--release").arg("8");
        command.arg(if debug { "-g" } else { "-g:none" });
        let output = command
            .arg("-d")
            .arg(&directory)
            .args(&paths)
            .output()
            .expect("the Java 8 compiler is available");
        assert!(
            output.status.success(),
            "javac failed for {label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = Command::new("java")
            .arg("-Xverify:all")
            .arg("-cp")
            .arg(&directory)
            .arg(main_class)
            .output()
            .expect("the Java runtime is available");
        assert!(
            output.status.success(),
            "java failed for {label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("the Java runner output is UTF-8")
    }

    fn java_test_directory(label: &str) -> (std::path::PathBuf, TemporaryDirectory) {
        static NEXT_JAVA_TEST: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jarde-enum-{label}-{}-{}",
            std::process::id(),
            NEXT_JAVA_TEST.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).expect("the private Java test directory is created");
        (directory.clone(), TemporaryDirectory(directory))
    }

    struct TemporaryDirectory(std::path::PathBuf);

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn compile_and_patch_raw_values_read() -> Vec<u8> {
        static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jarde-enum-values-access-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).expect("the private test directory is created");
        let _cleanup = TemporaryDirectory(directory.clone());
        let source = directory.join("E.java");
        fs::write(
            &source,
            include_str!("../openspec/evidence/java-syntax-2026-09-25/enum-values-access/E.java"),
        )
        .expect("the E source fixture is written");
        let output = Command::new("javac")
            .arg("--release")
            .arg("8")
            .arg("-g:none")
            .arg("-d")
            .arg(&directory)
            .arg(&source)
            .output()
            .expect("the Java 8 fixture compiler is available");
        assert!(
            output.status.success(),
            "javac failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let class_path = directory.join("E.class");
        let mut bytes = fs::read(class_path).expect("the fixture class was emitted");
        let facts = jarde_reader::classfile::class_facts(&bytes, &mut test_budget())
            .expect("the fixture classfile facts decode");
        let owner = b"E";
        let values_method = facts
            .constant_pool
            .iter()
            .find_map(|entry| match &entry.kind {
                CpEntryKind::MethodRef {
                    owner: actual_owner,
                    name,
                    descriptor,
                    ..
                } if actual_owner.0 == owner
                    && name.0 == b"values"
                    && descriptor.0 == b"()[LE;" =>
                {
                    Some(entry.index)
                }
                _ => None,
            })
            .expect("the generated values() method reference exists");
        let backing_field = facts
            .constant_pool
            .iter()
            .find_map(|entry| match &entry.kind {
                CpEntryKind::FieldRef {
                    owner: actual_owner,
                    name,
                    descriptor,
                    ..
                } if actual_owner.0 == owner && name.0 == b"$VALUES" && descriptor.0 == b"[LE;" => {
                    Some(entry.index)
                }
                _ => None,
            })
            .expect("the generated backing field reference exists");
        let before = [0xb8, (values_method >> 8) as u8, values_method as u8, 0xb0];
        let after = [0xb2, (backing_field >> 8) as u8, backing_field as u8, 0xb0];
        let sites: Vec<_> = bytes
            .windows(before.len())
            .enumerate()
            .filter_map(|(offset, window)| (window == before).then_some(offset))
            .collect();
        assert_eq!(
            sites.len(),
            1,
            "raw() has one exact javac values() sequence"
        );
        bytes[sites[0]..sites[0] + after.len()].copy_from_slice(&after);
        bytes
    }

    fn performed<T>(outcome: crate::OperationOutcome<T>) -> T {
        match outcome {
            crate::OperationOutcome::Performed(report) => report,
            crate::OperationOutcome::Ambiguous(candidates) => {
                panic!("unexpected ambiguous class selection: {candidates:?}")
            }
            crate::OperationOutcome::Incomplete(candidates) => {
                panic!("unexpected incomplete class selection: {candidates:?}")
            }
        }
    }

    const N0_PATROL_CLASS: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-10-01/inner-enum-args-patrol/fixture/N0.class"
    );
    const N0_PATROL_TEXT: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-10-01/inner-enum-args-patrol/fixture/N0.jarde.java"
    );
    const N3_PATROL_SOURCE: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-10-01/inner-enum-args-patrol/fixture/N3.java"
    );
    const N1_PATROL_SOURCE: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-10-01/inner-enum-args-patrol/fixture/N1.java"
    );
    const N3_PATROL_RUN: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-10-01/inner-enum-args-patrol/fixture/n3.out"
    );
    const N1_PATROL_RUN: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-10-01/inner-enum-args-patrol/fixture/n1.out"
    );

    fn compile_java_sources_to_directory(
        label: &str,
        sources: &[(&str, &str)],
        debug: bool,
    ) -> (std::path::PathBuf, TemporaryDirectory) {
        let (directory, cleanup) = java_test_directory(label);
        let mut command = Command::new("javac");
        command.arg("--release").arg("8");
        command.arg(if debug { "-g" } else { "-g:none" });
        command.arg("-d").arg(&directory);
        for (name, source_text) in sources {
            let source = directory.join(format!("{name}.java"));
            fs::write(&source, source_text).expect("the fixture source is written");
            command.arg(&source);
        }
        let output = command.output().expect("the Java 8 compiler is available");
        assert!(
            output.status.success(),
            "javac failed for {label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        (directory, cleanup)
    }

    fn compile_and_run_folded_sources(
        label: &str,
        folded_sources: &[(&str, &str)],
        runner_class: &str,
        runner_source: &str,
    ) -> String {
        let (directory, _cleanup) = java_test_directory(label);
        let mut compile = Command::new("javac");
        compile.arg("--release").arg("8").arg("-g:none");
        // The folded sources live in the classpath directory itself; an empty `-sourcepath`
        // keeps javac from compiling them a second time through the implicit source lookup.
        compile.arg("-sourcepath").arg("");
        compile.arg("-cp").arg(&directory).arg("-d").arg(&directory);
        let mut paths = Vec::new();
        for (name, text) in folded_sources {
            let path = directory.join(format!("{name}.java"));
            fs::write(&path, text).expect("the folded source is written");
            paths.push(path);
        }
        let runner_path = directory.join(format!("{runner_class}.java"));
        fs::write(&runner_path, runner_source).expect("the runner source is written");
        paths.push(runner_path);
        for path in &paths {
            compile.arg(path);
        }
        let output = compile.output().expect("the Java 8 compiler is available");
        assert!(
            output.status.success(),
            "the folded sources failed to compile for {label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let run = Command::new("java")
            .arg("-Xverify:all")
            .arg("-cp")
            .arg(&directory)
            .arg(runner_class)
            .output()
            .expect("the Java runtime is available");
        assert!(
            run.status.success(),
            "the folded sources failed to run for {label}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        String::from_utf8(run.stdout).expect("the folded run output is UTF-8")
    }

    #[test]
    fn arbitrary_ctor_descriptor_grammar_parses_only_the_admitted_tails() {
        assert_eq!(
            parse_arbitrary_ctor_descriptor(b"(Ljava/lang/String;II)V"),
            Some(vec![EnumUserParameter::IntLiteral(
                EnumIntUserParameter::Int
            )])
        );
        assert_eq!(
            parse_arbitrary_ctor_descriptor(b"(Ljava/lang/String;IBLjava/lang/String;)V"),
            Some(vec![
                EnumUserParameter::IntLiteral(EnumIntUserParameter::Byte),
                EnumUserParameter::StringLiteral,
            ])
        );
        assert_eq!(
            parse_arbitrary_ctor_descriptor(b"(Ljava/lang/String;ILN3$Simple;)V"),
            Some(vec![EnumUserParameter::Object {
                descriptor: b"LN3$Simple;".to_vec(),
            }])
        );
        assert_eq!(
            parse_arbitrary_ctor_descriptor(b"(Ljava/lang/String;I[Ljava/lang/String;)V"),
            None,
            "array parameters stay outside the grammar"
        );
        assert_eq!(
            parse_arbitrary_ctor_descriptor(b"(Ljava/lang/String;IJ)V"),
            None,
            "long literal parameters stay outside the grammar"
        );
        assert_eq!(
            parse_arbitrary_ctor_descriptor(b"(Ljava/lang/String;ID)V"),
            None,
            "double literal parameters stay outside the grammar"
        );
        assert_eq!(
            parse_arbitrary_ctor_descriptor(b"(Ljava/lang/String;I)V"),
            None,
            "the empty tail is the fixed no-argument descriptor"
        );
        assert_eq!(
            parse_arbitrary_ctor_descriptor(
                b"(Ljava/lang/String;IBLjava/lang/String;Ljava/lang/Object;I)V"
            ),
            None,
            "more than three user arguments stay outside the grammar"
        );
        assert_eq!(
            parse_arbitrary_ctor_descriptor(b"(Ljava/lang/String;ILjava/lang/String;)V"),
            Some(vec![EnumUserParameter::StringLiteral]),
            "the fixed String shape also parses, but the whitelist keeps it on the original path"
        );
    }

    #[test]
    fn int_user_arguments_narrow_by_their_parameter_descriptor() {
        assert_eq!(
            narrowed_int_user_text(EnumIntUserParameter::Byte, -56),
            Some("(byte) -56".to_owned())
        );
        assert_eq!(
            narrowed_int_user_text(EnumIntUserParameter::Short, 300),
            Some("(short) 300".to_owned())
        );
        assert_eq!(
            narrowed_int_user_text(EnumIntUserParameter::Int, -1),
            Some("-1".to_owned())
        );
        assert_eq!(
            narrowed_int_user_text(EnumIntUserParameter::Boolean, 1),
            Some("true".to_owned())
        );
        assert_eq!(
            narrowed_int_user_text(EnumIntUserParameter::Boolean, 0),
            Some("false".to_owned())
        );
        assert_eq!(
            narrowed_int_user_text(EnumIntUserParameter::Boolean, 2),
            None,
            "no boolean literal denotes 2"
        );
        assert_eq!(
            narrowed_int_user_text(EnumIntUserParameter::Char, 120),
            Some("'x'".to_owned())
        );
        assert_eq!(
            narrowed_int_user_text(EnumIntUserParameter::Char, 39),
            Some("'\\''".to_owned())
        );
        assert_eq!(
            narrowed_int_user_text(EnumIntUserParameter::Char, -1),
            Some("(char) -1".to_owned()),
            "no char literal denotes -1; the cast is javac's own constant fold"
        );
        assert_eq!(
            narrowed_int_user_text(EnumIntUserParameter::Char, 10),
            Some("(char) 10".to_owned()),
            "control characters keep the cast spelling"
        );
    }

    #[test]
    fn patrol_n0_fixture_keeps_its_exact_folded_text() {
        let report = enum_report(N0_PATROL_CLASS, "N0", &mut test_budget());
        assert_eq!(report.text, N0_PATROL_TEXT);
    }

    #[test]
    fn patrol_n3_probes_fold_and_recompile_run_like_the_fixture_baseline() {
        for debug in [false, true] {
            let (directory, _cleanup) = compile_java_sources_to_directory(
                "patrol-n3-fixture",
                &[("N3", N3_PATROL_SOURCE)],
                debug,
            );
            let original = Command::new("java")
                .arg("-Xverify:all")
                .arg("-cp")
                .arg(&directory)
                .arg("N3")
                .output()
                .expect("the Java runtime is available");
            assert_eq!(
                String::from_utf8(original.stdout).as_deref(),
                Ok(N3_PATROL_RUN),
                "the original fixture classes run like the recorded baseline"
            );
            let mut simple_folded_text = None;
            let mut refs_folded_text = None;
            for (name, folded_line, other_line) in [
                ("N3$Simple", "A((byte) 1, \"x\"),", "B((byte) 2, \"y\");"),
                ("N3$Refs", "A(N3$Simple.A),", "B(N3$Simple.B);"),
            ] {
                let bytes = fs::read(directory.join(format!("{name}.class")))
                    .expect("the fixture nested enum class was emitted");
                let report = enum_report(&bytes, name, &mut test_budget());
                let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(group)) =
                    &report.enum_constant_proof
                else {
                    panic!("{name} refused: {:?}", report.enum_constant_proof);
                };
                assert_eq!(group.constants.len(), 2);
                assert!(report.text.contains(folded_line), "{name}: {}", report.text);
                assert!(report.text.contains(other_line), "{name}: {}", report.text);
                assert!(!report.text.contains("valueOf("));
                assert!(!report.text.contains("$VALUES"));
                if name == "N3$Simple" {
                    assert!(
                        report
                            .text
                            .contains("private N3$Simple(byte arg0, java.lang.String arg1) {")
                    );
                    assert!(report.text.contains("this.num = arg0;"));
                    assert!(report.text.contains("this.s = arg1;"));
                    simple_folded_text = Some(report.text);
                } else {
                    assert!(report.text.contains("private N3$Refs(N3$Simple arg0) {"));
                    assert!(report.text.contains("this.s = arg0;"));
                    refs_folded_text = Some(report.text);
                }
            }
            let simple_folded_text = simple_folded_text.expect("the Simple text was captured");
            let refs_folded_text = refs_folded_text.expect("the Refs text was captured");
            let recovered = compile_and_run_folded_sources(
                "patrol-n3-folded",
                &[
                    ("N3$Simple", simple_folded_text.as_str()),
                    ("N3$Refs", refs_folded_text.as_str()),
                ],
                "N3FoldedRunner",
                "public final class N3FoldedRunner {\n    public static void main(String[] a) {\n        System.out.println(N3$Simple.A.n());\n        System.out.println(N3$Refs.A.g().n());\n    }\n}\n",
            );
            assert_eq!(recovered, N3_PATROL_RUN);
        }
    }

    #[test]
    fn patrol_n1_full_shape_folds_and_recompiles_run_like_the_fixture_baseline() {
        let (directory, _cleanup) = compile_java_sources_to_directory(
            "patrol-n1-fixture",
            &[("N1", N1_PATROL_SOURCE)],
            false,
        );
        let original = Command::new("java")
            .arg("-Xverify:all")
            .arg("-cp")
            .arg(&directory)
            .arg("N1")
            .output()
            .expect("the Java runtime is available");
        assert_eq!(
            String::from_utf8(original.stdout).as_deref(),
            Ok(N1_PATROL_RUN),
            "the original fixture classes run like the recorded baseline"
        );
        let mut folded_texts = Vec::new();
        for (name, folded_lines) in [
            (
                "N1$Numbers$NumString",
                vec!["ONE(\"one\"),", "TWO(\"two\");"],
            ),
            (
                "N1$Numbers",
                vec![
                    "ONE((byte) 1, N1$Numbers$NumString.ONE),",
                    "TWO((byte) 2, N1$Numbers$NumString.TWO);",
                ],
            ),
        ] {
            let bytes = fs::read(directory.join(format!("{name}.class")))
                .expect("the fixture nested enum class was emitted");
            let report = enum_report(&bytes, name, &mut test_budget());
            let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(group)) =
                &report.enum_constant_proof
            else {
                panic!("{name} refused: {:?}", report.enum_constant_proof);
            };
            assert_eq!(group.constants.len(), 2);
            for line in folded_lines {
                assert!(report.text.contains(line), "{name}: {}", report.text);
            }
            assert!(!report.text.contains("valueOf("));
            assert!(!report.text.contains("$VALUES"));
            if name == "N1$Numbers" {
                assert!(
                    report
                        .text
                        .contains("private N1$Numbers(byte arg0, N1$Numbers$NumString arg1) {")
                );
                assert!(report.text.contains("this.num = arg0;"));
                assert!(report.text.contains("this.str = arg1;"));
            }
            folded_texts.push((name, report.text));
        }
        let recovered = compile_and_run_folded_sources(
            "patrol-n1-folded",
            &[
                ("N1$Numbers$NumString", folded_texts[0].1.as_str()),
                ("N1$Numbers", folded_texts[1].1.as_str()),
            ],
            "N1FoldedRunner",
            "public final class N1FoldedRunner {\n    public static void main(String[] args) {\n        System.out.println(N1$Numbers.ONE.getNum());\n        System.out.println(N1$Numbers.ONE.getNumStr());\n        System.out.println(N1$Numbers.ONE.getName());\n        System.out.println(N1$Numbers.TWO.getNum());\n    }\n}\n",
        );
        assert_eq!(recovered, N1_PATROL_RUN);
    }

    #[test]
    fn arbitrary_argument_shapes_fold_with_narrowed_spellings_and_verified_runtime() {
        const RUNNER_BODY: &str = r#"        for (BytesText v : BytesText.values()) System.out.println(v.name() + ":" + v.n() + ":" + v.text());
        for (CrossRefs v : CrossRefs.values()) System.out.println(v.name() + ":" + v.other());
        for (CharArgs v : CharArgs.values()) System.out.println(v.name() + ":" + v.c());
        for (BoolArgs v : BoolArgs.values()) System.out.println(v.name() + ":" + v.flag());
        for (ShortArgs v : ShortArgs.values()) System.out.println(v.name() + ":" + v.s());
        for (IntArgs v : IntArgs.values()) System.out.println(v.name() + ":" + v.i());
        System.out.println(BytesText.valueOf("A").text());
        System.out.println(BytesText.A.n() + CrossRefs.R1.other().toString().length());"#;
        const MATRIX: &str = r#"enum BytesText {
    A((byte) 1, "x"), B((byte) 2, "y");
    private final byte num;
    private final String s;
    BytesText(byte n, String s) { this.num = n; this.s = s; }
    byte n() { return num; }
    String text() { return s; }
}
enum CrossRefs {
    R1(BytesText.A), R2(BytesText.B), R3(null);
    private final Object other;
    CrossRefs(Object other) { this.other = other; }
    Object other() { return other; }
}
enum CharArgs {
    ONE('x'), TWO('\''), THREE((char) -1);
    private final char c;
    CharArgs(char c) { this.c = c; }
    char c() { return c; }
}
enum BoolArgs {
    NO(false), YES(true);
    private final boolean flag;
    BoolArgs(boolean flag) { this.flag = flag; }
    boolean flag() { return flag; }
}
enum ShortArgs {
    BIG((short) 300), SMALL((short) -300);
    private final short s;
    ShortArgs(short s) { this.s = s; }
    short s() { return s; }
}
enum IntArgs {
    MID(5), NEG(-1);
    private final int i;
    IntArgs(int i) { this.i = i; }
    int i() { return i; }
}
public class ArgsMatrix {
    public static void main(String[] args) {
"#;
        let matrix_source = format!("{MATRIX}{RUNNER_BODY}\n    }}\n}}\n");
        let (directory, _cleanup) = compile_java_sources_to_directory(
            "args-matrix",
            &[("ArgsMatrix", &matrix_source)],
            false,
        );
        let original = Command::new("java")
            .arg("-Xverify:all")
            .arg("-cp")
            .arg(&directory)
            .arg("ArgsMatrix")
            .output()
            .expect("the Java runtime is available");
        assert!(original.status.success());
        let original_output = String::from_utf8(original.stdout).expect("matrix output is UTF-8");
        let expectations: Vec<(&str, Vec<&str>)> = vec![
            (
                "BytesText",
                vec![
                    "A((byte) 1, \"x\"),",
                    "B((byte) 2, \"y\");",
                    "private BytesText(byte arg0, java.lang.String arg1) {",
                    "this.num = arg0;",
                    "this.s = arg1;",
                ],
            ),
            (
                "CrossRefs",
                vec![
                    "R1(BytesText.A),",
                    "R2(BytesText.B),",
                    "R3(null);",
                    "private CrossRefs(java.lang.Object arg0) {",
                    "this.other = arg0;",
                ],
            ),
            (
                "CharArgs",
                vec![
                    "ONE('x'),",
                    "TWO('\\''),",
                    "THREE((char) 65535);",
                    "private CharArgs(char arg0) {",
                    "this.c = arg0;",
                ],
            ),
            (
                "BoolArgs",
                vec![
                    "NO(false),",
                    "YES(true);",
                    "private BoolArgs(boolean arg0) {",
                    "this.flag = arg0;",
                ],
            ),
            (
                "ShortArgs",
                vec![
                    "BIG((short) 300),",
                    "SMALL((short) -300);",
                    "private ShortArgs(short arg0) {",
                    "this.s = arg0;",
                ],
            ),
            (
                "IntArgs",
                vec![
                    "MID(5),",
                    "NEG(-1);",
                    "private IntArgs(int arg0) {",
                    "this.i = arg0;",
                ],
            ),
        ];
        let mut folded_sources = Vec::new();
        for (name, lines) in &expectations {
            let bytes = fs::read(directory.join(format!("{name}.class")))
                .expect("the matrix enum class was emitted");
            let report = enum_report(&bytes, name, &mut test_budget());
            let ClassSourceEnumConstantProof::Proved(ProvedEnumConstantGroup::Ordinary(_)) =
                &report.enum_constant_proof
            else {
                panic!("{name} refused: {:?}", report.enum_constant_proof);
            };
            for line in lines {
                assert!(report.text.contains(line), "{name}: {}", report.text);
            }
            assert!(
                !report.text.contains("valueOf(") && !report.text.contains("$VALUES"),
                "{name} keeps values/valueOf/clinit folded into the enum presentation: {}",
                report.text
            );
            folded_sources.push((*name, report.text));
        }
        let recovered = compile_and_run_folded_sources(
            "args-matrix-folded",
            &folded_sources
                .iter()
                .map(|(name, text)| (*name, text.as_str()))
                .collect::<Vec<_>>(),
            "ArgsMatrixFoldedRunner",
            &format!(
                "public final class ArgsMatrixFoldedRunner {{\n    public static void main(String[] args) {{\n{RUNNER_BODY}\n    }}\n}}\n"
            ),
        );
        assert_eq!(recovered, original_output);
    }

    #[test]
    fn unsupported_arbitrary_argument_shapes_refuse_and_keep_the_per_field_presentation() {
        const REFUSALS: &str = r#"enum KindMismatchObject {
    A("plain");
    private final Object v;
    KindMismatchObject(Object v) { this.v = v; }
    Object v() { return v; }
}
enum StringParamGetstatic {
    A(StrHold.source);
    private final String v;
    StringParamGetstatic(String v) { this.v = v; }
    String v() { return v; }
}
enum ExtraStatement {
    A((byte) 1, "x");
    private final byte num;
    private final String s;
    ExtraStatement(byte n, String s) { System.out.println("side"); this.num = n; this.s = s; }
    byte n() { return num; }
}
enum TooManyArgs {
    A((byte) 1, "x", java.math.BigInteger.ZERO, 5);
    private final byte b;
    private final String s;
    private final java.math.BigInteger big;
    private final int i;
    TooManyArgs(byte b, String s, java.math.BigInteger big, int i) {
        this.b = b;
        this.s = s;
        this.big = big;
        this.i = i;
    }
    byte b() { return b; }
    String s() { return s; }
    java.math.BigInteger big() { return big; }
    int i() { return i; }
}
enum LongArg {
    A(1L);
    private final long v;
    LongArg(long v) { this.v = v; }
    long v() { return v; }
}
class StrHold {
    static String source = "held";
}
public class ArgsRefusals {
    public static void main(String[] args) {
        System.out.println(KindMismatchObject.A.v());
        System.out.println(StringParamGetstatic.A.v());
        System.out.println(ExtraStatement.A.n());
        System.out.println(TooManyArgs.A.b() + ":" + TooManyArgs.A.s() + ":" + TooManyArgs.A.big() + ":" + TooManyArgs.A.i());
        System.out.println(LongArg.A.v());
    }
}"#;
        let (directory, _cleanup) = compile_java_sources_to_directory(
            "args-refusals",
            &[("ArgsRefusals", REFUSALS)],
            false,
        );
        let original = Command::new("java")
            .arg("-Xverify:all")
            .arg("-cp")
            .arg(&directory)
            .arg("ArgsRefusals")
            .output()
            .expect("the Java runtime is available");
        assert!(
            original.status.success(),
            "every negative fixture is verifier-valid: {}",
            String::from_utf8_lossy(&original.stderr)
        );
        let expectations: Vec<(&str, &str)> = vec![
            ("KindMismatchObject", "A(\"plain\")"),
            ("StringParamGetstatic", "A(StrHold.source)"),
            ("ExtraStatement", "A((byte) 1, \"x\")"),
            ("TooManyArgs", "A((byte) 1"),
            ("LongArg", "A(1L)"),
        ];
        for (name, folded_line) in expectations {
            let bytes = fs::read(directory.join(format!("{name}.class")))
                .expect("the refusal enum class was emitted");
            let report = enum_report(&bytes, name, &mut test_budget());
            assert!(
                matches!(
                    report.enum_constant_proof,
                    ClassSourceEnumConstantProof::Refused { .. }
                ),
                "{name} should refuse: {:?}",
                report.enum_constant_proof
            );
            assert!(
                report
                    .text
                    .contains(&format!("public static final {name} A;")),
                "{name} keeps the per-field presentation: {}",
                report.text
            );
            assert!(
                !report.text.contains(folded_line),
                "{name} must not fold: {}",
                report.text
            );
        }
    }

    #[test]
    fn arbitrary_argument_fold_stays_a_budget_stop_that_keeps_the_per_field_text() {
        const SOURCE: &str = r#"enum BudgetBytes {
    A((byte) 1, "x"), B((byte) 2, "y");
    private final byte num;
    private final String s;
    BudgetBytes(byte n, String s) { this.num = n; this.s = s; }
}"#;
        let bytes = compile_java_class("BudgetBytes", SOURCE, false);
        let mut limits = test_budget().limits().clone();
        limits.ir_items = 150;
        let limited = enum_report(&bytes, "BudgetBytes", &mut Budget::new(limits));
        assert!(matches!(
            limited.enum_constant_proof,
            ClassSourceEnumConstantProof::Stopped { .. }
        ));
        assert!(limited.text.contains("public static final BudgetBytes A;"));
        assert!(!limited.text.contains("A((byte) 1"));
    }

    #[test]
    fn arbitrary_body_shape_preserves_typed_budget_stops_and_local_refusals() {
        let engine = crate::Engine::new();
        let snapshot = engine
            .open(
                crate::ArtifactInput::bytes(
                    include_bytes!("../tests/fixtures/s1-enum-constructor-stop/input.jar").to_vec(),
                ),
                &mut test_budget(),
            )
            .unwrap();
        let mut request = enum_request(&snapshot, "demo/IntBody");
        request.environment.policy = crate::EnvironmentPolicy::PlainJar;
        let report = performed(
            engine
                .class_source(
                    std::slice::from_ref(&snapshot),
                    &request,
                    &mut test_budget(),
                )
                .unwrap(),
        );
        assert!(report.projection_inputs.enum_projection.is_some());
        let shape = &report.enum_constant_body_relations[0].group_shape;
        assert!(shape.constructor_chain.is_ok());
        assert!(shape.constructor_user_tail.is_some());
        let facts = jarde_reader::classfile::class_facts(
            include_bytes!("../tests/fixtures/s1-enum-constructor-stop/IntBody.class"),
            &mut test_budget(),
        )
        .unwrap();
        let environment = request
            .environment
            .build(std::slice::from_ref(&snapshot))
            .unwrap();
        let mut codes = Vec::new();
        for method in &report.methods {
            if matches!(method.outcome, ClassSourceOutcome::NoBody) {
                continue;
            }
            let analysis = jarde_jvm::analyze_method_ir(
                std::slice::from_ref(&snapshot),
                &crate::ir::MethodAnalysisRequest {
                    environment: environment.clone(),
                    method: method.item.identity.clone(),
                    stages: crate::AnalysisStage::ALL.to_vec(),
                },
                &mut test_budget(),
            )
            .unwrap();
            codes.push(
                capture_method_code(
                    method.item.index,
                    &method.item.identity,
                    analysis.ir(),
                    &mut test_budget(),
                )
                .unwrap()
                .unwrap(),
            );
        }
        let clinit = codes
            .iter()
            .find(|code| code.member.as_ref().unwrap().name.0 == b"<clinit>")
            .unwrap();
        let parameters = parse_arbitrary_ctor_descriptor(CTOR_DESCRIPTOR).unwrap();
        let prove = |codes: &[EnumMethodCodeCandidate], budget: &mut Budget| {
            jarde_java::source_proof::prove_enum_body_candidate_shape(
                &facts.this_class.raw().0,
                facts.access_flags,
                &facts.fields,
                &facts.methods,
                codes,
                clinit.member.as_ref().unwrap(),
                clinit,
                shape.constants.clone(),
                shape.implicit_members.clone(),
                CTOR_DESCRIPTOR,
                &parameters,
                shape.constructors.clone(),
                true,
                shape.abstract_methods.clone(),
                shape.interface_methods.clone(),
                |_| true,
                budget,
            )
        };
        let valid = prove(&codes, &mut test_budget()).unwrap().unwrap();
        assert_eq!(valid, *shape);
        let mut limits = test_budget().limits().clone();
        limits.ir_items = 0;
        let mut local = Budget::new(limits.clone());
        let local_result = prove(&codes, &mut local);
        // A local failed permit has no ledger latch. The proof must return the error itself.
        assert!(local.poll().is_ok());
        let entry = Budget::new(limits);
        let ledger = crate::OperationLedger::new(&entry);
        let mut shared = test_budget();
        shared.with_ledger(ledger.clone(), crate::UsageOwner::Methods);
        let shared_result = prove(&codes, &mut shared);
        // Even the old shape's swallowed error cannot admit a later child read: its first charge polls this latch.
        assert!(
            shared
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .is_err()
        );
        println!(
            "BQ_SHAPE {}",
            serde_json::json!({
                "local_result": format!("{local_result:?}"), "shared_result": format!("{shared_result:?}"),
                "local_usage": local.usage(), "ledger_usage": ledger.usage(), "shared_latched": shared.poll().is_err(),
            })
        );
        for result in [local_result, shared_result] {
            assert!(
                matches!(
                    result,
                    Err(crate::Error::BudgetExceeded {
                        dimension: crate::BudgetDimension::IrItems,
                        ..
                    })
                ),
                "typed stop became a local refusal: {result:?}"
            );
        }
        let mut damaged = codes.clone();
        damaged
            .iter_mut()
            .find(|code| code.member.as_ref().unwrap().descriptor.0 == CTOR_DESCRIPTOR)
            .unwrap()
            .complete = false;
        let mut independent = test_budget();
        let refused = prove(&damaged, &mut independent).unwrap().unwrap();
        assert!(refused.constructor_chain.is_err());
        assert!(independent.poll().is_ok());
        assert!(
            prove(&codes, &mut independent)
                .unwrap()
                .unwrap()
                .constructor_chain
                .is_ok()
        );
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn arbitrary_body_public_ir_budget_keeps_parent_prefix_and_records_actual_work() {
        let engine = crate::Engine::new();
        let snapshot = engine
            .open(
                crate::ArtifactInput::bytes(
                    include_bytes!("../tests/fixtures/s1-enum-constructor-stop/input.jar").to_vec(),
                ),
                &mut test_budget(),
            )
            .unwrap();
        let mut request = enum_request(&snapshot, "demo/IntBody");
        request.environment.policy = crate::EnvironmentPolicy::PlainJar;
        let wide = performed(
            engine
                .class_source(
                    std::slice::from_ref(&snapshot),
                    &request,
                    &mut test_budget(),
                )
                .unwrap(),
        );
        assert!(wide.projection_inputs.enum_projection.is_some());
        // Every root method is complete by this charged prefix; subsequent work is class-level proof and child demand.
        let start = match &wide.methods.last().unwrap().outcome {
            ClassSourceOutcome::Recovered { report, .. } => match &report.execution {
                ExecutionReport::Complete { usage } => usage.ir_items,
                _ => panic!("the initializer completes before class proof"),
            },
            _ => panic!("the fixture initializer recovers"),
        };
        for cap in start..start + 96 {
            let mut limits = test_budget().limits().clone();
            limits.ir_items = cap;
            let mut budget = Budget::new(limits);
            let before = jarde_reader::observation::snapshot();
            let ir_before = jarde_jvm::observation::snapshot();
            let report = performed(
                engine
                    .class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
                    .unwrap(),
            );
            let after = jarde_reader::observation::snapshot();
            let ir_after = jarde_jvm::observation::snapshot();
            assert!(matches!(
                report.execution,
                ExecutionReport::Partial {
                    reason: crate::TerminationReason::BudgetExceeded {
                        dimension: crate::BudgetDimension::IrItems
                    },
                    ..
                }
            ));
            assert!(report.projection_inputs.enum_projection.is_none());
            for (actual, expected) in report.methods.iter().zip(&wide.methods) {
                assert_eq!(actual.item.identity, expected.item.identity);
                assert_eq!(actual.text, expected.text);
            }
            println!(
                "BQ_PUBLIC {}",
                serde_json::json!({"cap":cap,"shared":false,"usage":budget.usage(),
                    "actual_read_attempts":after.read_attempts-before.read_attempts,
                    "actual_body_decodes":after.body_decode_attempts-before.body_decode_attempts,
                    "jvm_before":ir_before,"jvm_after":ir_after,
                    "execution":report.execution,
                })
            );
        }
    }
}
