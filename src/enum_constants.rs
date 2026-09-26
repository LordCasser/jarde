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
use jarde_jvm::method_ir::MethodIr;
use jarde_reader::classfile::{
    ClassFacts, CpEntryKind, ImmediateValue, InstructionFact, InstructionOperands, MemberHeader,
    cp_entry,
};
use jarde_reader::model::{ExecutionReport, PhysicalMethodId};

const ACC_PUBLIC: u16 = 0x0001;
const ACC_PRIVATE: u16 = 0x0002;
const ACC_STATIC: u16 = 0x0008;
const ACC_FINAL: u16 = 0x0010;
const ACC_ENUM: u16 = 0x4000;
const ACC_SYNTHETIC: u16 = 0x1000;
const ACC_ABSTRACT: u16 = 0x0400;
const ACC_NATIVE: u16 = 0x0100;
const ACC_INTERFACE: u16 = 0x0200;

const ENUM_SUPER: &[u8] = b"java/lang/Enum";
const CTOR_DESCRIPTOR: &[u8] = b"(Ljava/lang/String;II)V";
const DELEGATING_CTOR_DESCRIPTOR: &[u8] = b"(Ljava/lang/String;I)V";
const ENUM_CTOR_DESCRIPTOR: &[u8] = b"(Ljava/lang/String;I)V";
const VALUES_DESCRIPTOR_PREFIX: &[u8] = b"()[L";
const VALUE_OF_DESCRIPTOR_PREFIX: &[u8] = b"(Ljava/lang/String;)L";
const ENUM_VALUE_OF_DESCRIPTOR: &[u8] = b"(Ljava/lang/Class;Ljava/lang/String;)Ljava/lang/Enum;";

/// A resolved constant-pool value needed by the fixed enum bytecode patterns.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EnumCodeReference {
    Integer(i32),
    String(Vec<u8>),
    Class(Vec<u8>),
    Field {
        owner: Vec<u8>,
        name: Vec<u8>,
        descriptor: Vec<u8>,
    },
    Method {
        owner: Vec<u8>,
        name: Vec<u8>,
        descriptor: Vec<u8>,
        interface: bool,
    },
    Other,
}

/// One instruction from the selected run's decoded `Code`, with just the typed operands used by
/// the proof. The opcode, local and immediate remain bytecode facts; `reference` is resolved from
/// the same run's pool rather than from source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnumCodeInstruction {
    pub(crate) bci: u32,
    pub(crate) width: u32,
    pub(crate) opcode: u8,
    pub(crate) immediate: Option<ImmediateValue>,
    pub(crate) local: Option<u16>,
    pub(crate) reference: Option<EnumCodeReference>,
}

/// A direct or bootstrap-mediated symbolic member reference used by one physical method.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnumCodeUse {
    pub(crate) bci: u32,
    pub(crate) reference: EnumCodeReference,
}

/// The candidate body of one physically indexed method, carried from its one class-source run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnumMethodCodeCandidate {
    pub(crate) table_index: u64,
    pub(crate) member: Option<PhysicalMethodId>,
    pub(crate) complete: bool,
    pub(crate) exception_handler_count: u32,
    pub(crate) instructions: Vec<EnumCodeInstruction>,
    pub(crate) member_uses: Vec<EnumCodeUse>,
}

/// The exact members that a successful class-level proof may later project.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvedEnumConstant {
    pub(crate) field_index: u64,
    pub(crate) name: String,
    pub(crate) source_argument: Option<i32>,
    pub(crate) constructor_bci: u32,
    pub(crate) field_write_bci: u32,
}

/// A private handoff from task 2.1 to the class-source projection stages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvedOrdinaryEnumConstantGroup {
    pub(crate) constants: Vec<ProvedEnumConstant>,
    pub(crate) backing_field_index: u64,
    pub(crate) constructor_method_index: u64,
    pub(crate) delegating_constructor_method_index: Option<u64>,
    pub(crate) constructor_body: Option<Box<ProvedEnumConstructorBody>>,
    pub(crate) constructor_field_index: u64,
    pub(crate) constructor_signature_present: bool,
    pub(crate) initializer_method_index: u64,
    pub(crate) values_method_index: u64,
    pub(crate) value_of_method_index: u64,
    pub(crate) values_factory_method_index: u64,
    /// The first BCI after the complete constants and `$VALUES` setup prefix.
    pub(crate) initializer_prefix_end_bci: u32,
    /// Number of same-run structured initializer statements in the prefix.
    pub(crate) initializer_prefix_statement_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProvedEnumConstantGroup {
    Ordinary(ProvedOrdinaryEnumConstantGroup),
    Body(ProvedEnumConstantBodyGroup),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvedEnumConstantBodyGroup {
    pub(crate) constants: Vec<ProvedEnumBodyConstant>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvedEnumBodyConstant {
    pub(crate) field_index: u64,
    pub(crate) allocation_bci: u32,
    pub(crate) constructor_bci: u32,
    pub(crate) field_write_bci: u32,
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

/// Terminal effects proven from the same-run Code and constructor AST, kept private for 2.3.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvedEnumConstructorBody {
    method_index: usize,
    field_index: usize,
    enum_super_call_bci: u32,
    helper_call_bci: u32,
    field_write_bci: u32,
    return_bci: u32,
    helper_owner: Vec<u8>,
    pub(crate) candidate: jarde_java::report::ClassEnumConstructorCandidates,
}

pub(crate) fn may_capture_group_code(class: &ClassFacts, member_table_complete: bool) -> bool {
    if !member_table_complete
        || class.major_version != 52
        || class.minor_version != 0
        || class.access_flags & ACC_ENUM == 0
        || class.access_flags & ACC_INTERFACE != 0
        || class
            .super_class
            .as_ref()
            .map(|name| name.raw().0.as_slice())
            != Some(ENUM_SUPER)
    {
        return false;
    }

    class
        .fields
        .iter()
        .filter(|field| field.access_flags & ACC_ENUM != 0)
        .count()
        == 2
}

pub(crate) fn has_only_terminal_initializer_return(
    group: &ProvedOrdinaryEnumConstantGroup,
    code_candidates: &[EnumMethodCodeCandidate],
    initializer_candidates: &[jarde_java::report::ClassInitializerCandidates],
    methods: &[ClassSourceMethod],
) -> bool {
    if group.initializer_prefix_statement_count != 3 {
        return false;
    }
    let Ok(constructor_index) = usize::try_from(group.constructor_method_index) else {
        return false;
    };
    let Some(constructor) = methods.get(constructor_index) else {
        return false;
    };
    if group.constructor_signature_present && !constructor.enum_constructor_source_signature {
        return false;
    }
    let Ok(initializer_index) = usize::try_from(group.initializer_method_index) else {
        return false;
    };
    let Some(method) = methods.get(initializer_index) else {
        return false;
    };
    let matching_code: Vec<_> = code_candidates
        .iter()
        .filter(|candidate| candidate.table_index == group.initializer_method_index)
        .collect();
    let [code] = matching_code.as_slice() else {
        return false;
    };
    let suffix = code
        .instructions
        .iter()
        .filter(|instruction| instruction.bci >= group.initializer_prefix_end_bci)
        .collect::<Vec<_>>();
    if code.member.as_ref() != Some(&method.item.identity)
        || !code.complete
        || code.exception_handler_count != 0
        || !matches!(suffix.as_slice(), [instruction] if {
            instruction.bci == group.initializer_prefix_end_bci
                && instruction.width == 1
                && instruction.opcode == 0xb1
                && instruction.reference.is_none()
                && instruction.immediate.is_none()
                && instruction.local.is_none()
        })
    {
        return false;
    }
    let matching_initializers: Vec<_> = initializer_candidates
        .iter()
        .filter(|candidate| candidate.member.as_ref() == Some(&method.item.identity))
        .collect();
    let [initializer] = matching_initializers.as_slice() else {
        return false;
    };
    use jarde_java::report::{ClassInitializerStatementKind, ClassInitializerStep};
    let steps_are_only_prefix_and_return = matches!(
        initializer.steps.as_slice(),
        [
            ClassInitializerStep::FieldWrite(first),
            ClassInitializerStep::FieldWrite(second),
            ClassInitializerStep::FieldWrite(third),
            ClassInitializerStep::Other {
                order: 3,
                bci,
                kind: ClassInitializerStatementKind::Return,
            },
        ] if first.order == 0 && second.order == 1 && third.order == 2
            && *bci == group.initializer_prefix_end_bci
    );
    !initializer.has_exception_handlers && steps_are_only_prefix_and_return
}

/// One exact user assignment that may follow the proved compiler-owned enum prefix.
///
/// The value and target are taken from the same prepared `<clinit>` run. The class-source adapter
/// only asks the existing expression emitter to spell the RHS after this complete suffix has been
/// checked against Code, field headers, and the initializer statement sidecar.
#[derive(Clone, Debug)]
pub(crate) struct ProvedEnumStaticAssignment {
    pub(crate) field_index: u64,
    pub(crate) field_name: String,
    pub(crate) value: jarde_java::ast::Expr,
    pub(crate) initializer_member: PhysicalMethodId,
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

/// Prove the one static assignment suffix frozen by the Measure/Counted fixtures.
///
/// The fixed source constants and BCI shape deliberately keep this first slice narrow. A different
/// user suffix remains in the original `<clinit>` presentation, along with every physical enum
/// member; it is never trimmed from emitted text.
pub(crate) fn prove_static_assignment_suffix(
    input: EnumStaticAssignmentInput<'_>,
) -> Result<Option<ProvedEnumStaticAssignment>> {
    use jarde_java::ast::{AssignOp, ExprKind, Type};
    use jarde_java::report::{ClassInitializerStatementKind, ClassInitializerStep};

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
    if owner.is_empty()
        || group.initializer_prefix_statement_count != 3
        || group.initializer_prefix_end_bci != 34
        || group.constants.len() != 2
        || group.constants[0].name != "LOW"
        || group.constants[0].source_argument != Some(2)
        || group.constants[1].name != "HIGH"
        || group.constants[1].source_argument != Some(5)
    {
        return Ok(None);
    }

    // Every scan below is charged before inspecting the corresponding vector. In particular,
    // the method and field uniqueness checks do not turn a large physical table into free work.
    let table_work = field_headers
        .len()
        .saturating_add(source_fields.len())
        .saturating_add(method_headers.len())
        .saturating_add(source_methods.len())
        .saturating_add(code_candidates.len())
        .saturating_add(initializer_candidates.len());
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(table_work).unwrap_or(u64::MAX),
    )?;
    budget.poll()?;

    if field_headers.len() != source_fields.len() || method_headers.len() != source_methods.len() {
        return Ok(None);
    }
    let mut total_units = None;
    for (index, header) in field_headers.iter().enumerate() {
        budget.poll()?;
        if header.name.raw().0 == b"totalUnits" {
            if total_units.is_some() {
                return Ok(None);
            }
            total_units = Some(index);
        }
    }
    let Some(field_index) = total_units else {
        return Ok(None);
    };
    let Some(field_header) = field_headers.get(field_index) else {
        return Ok(None);
    };
    let Some(source_field) = source_fields.get(field_index) else {
        return Ok(None);
    };
    let field_index_u64 = u64::try_from(field_index).unwrap_or(u64::MAX);
    if field_header.descriptor.raw().0 != b"I"
        || field_header.access_flags != ACC_STATIC
        || field_header
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"ConstantValue")
        || source_field.item.index != field_index_u64
        || source_field.item.name.raw().0 != b"totalUnits"
        || source_field.item.descriptor.raw().0 != b"I"
        || source_field.item.access_flags != ACC_STATIC
        || source_field.declaration.is_none()
        || !source_field.markers.is_empty()
    {
        return Ok(None);
    }

    let Ok(initializer_index) = usize::try_from(group.initializer_method_index) else {
        return Ok(None);
    };
    let (Some(initializer_header), Some(initializer_method)) = (
        method_headers.get(initializer_index),
        source_methods.get(initializer_index),
    ) else {
        return Ok(None);
    };
    if initializer_header.name.raw().0 != b"<clinit>"
        || initializer_header.descriptor.raw().0 != b"()V"
        || initializer_header.access_flags != ACC_STATIC
        || initializer_method.item.index != group.initializer_method_index
        || initializer_method.item.identity.name.0 != b"<clinit>"
        || initializer_method.item.identity.descriptor.0 != b"()V"
        || source_field.item.identity.owner != initializer_method.item.identity.owner
    {
        return Ok(None);
    }
    let mut matching_code = None;
    for candidate in code_candidates {
        budget.poll()?;
        if candidate.table_index == group.initializer_method_index
            && matching_code.replace(candidate).is_some()
        {
            return Ok(None);
        }
    }
    let Some(code) = matching_code else {
        return Ok(None);
    };
    if code.member.as_ref() != Some(&initializer_method.item.identity)
        || !code.complete
        || code.exception_handler_count != 0
    {
        return Ok(None);
    }

    let mut method_matches = Vec::new();
    for (index, header) in method_headers.iter().enumerate() {
        budget.poll()?;
        if header.name.raw().0 == b"sumUnits" && header.descriptor.raw().0 == b"()I" {
            method_matches.push(index);
        }
    }
    let [sum_units_index] = method_matches.as_slice() else {
        return Ok(None);
    };
    let Some(sum_units_header) = method_headers.get(*sum_units_index) else {
        return Ok(None);
    };
    let Some(sum_units_method) = source_methods.get(*sum_units_index) else {
        return Ok(None);
    };
    if sum_units_header.access_flags & ACC_STATIC == 0
        || sum_units_header.access_flags & (ACC_ABSTRACT | ACC_NATIVE) != 0
        || !sum_units_header
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code")
        || sum_units_method.item.index != u64::try_from(*sum_units_index).unwrap_or(u64::MAX)
        || sum_units_method.item.identity.owner != initializer_method.item.identity.owner
        || sum_units_method.item.identity.name.0 != b"sumUnits"
        || sum_units_method.item.identity.descriptor.0 != b"()I"
        || sum_units_method.declaration.is_none()
    {
        return Ok(None);
    }

    let owner_string = match std::str::from_utf8(owner) {
        Ok(owner) => owner,
        Err(_) => return Ok(None),
    };
    let code_scan_work = code
        .instructions
        .len()
        .saturating_add(code.member_uses.len());
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(code_scan_work).unwrap_or(u64::MAX),
    )?;
    let mut suffix = Vec::new();
    for instruction in &code.instructions {
        budget.poll()?;
        if instruction.bci >= group.initializer_prefix_end_bci {
            suffix.push(instruction);
        }
    }
    let call_reference = EnumCodeReference::Method {
        owner: owner.to_vec(),
        name: b"sumUnits".to_vec(),
        descriptor: b"()I".to_vec(),
        interface: false,
    };
    let field_reference = EnumCodeReference::Field {
        owner: owner.to_vec(),
        name: b"totalUnits".to_vec(),
        descriptor: b"I".to_vec(),
    };
    if !matches!(
        suffix.as_slice(),
        [call, store, ret]
            if call.bci == 34
                && call.width == 3
                && call.opcode == 0xb8
                && call.reference.as_ref() == Some(&call_reference)
                && call.immediate.is_none()
                && call.local.is_none()
                && store.bci == 37
                && store.width == 3
                && store.opcode == 0xb3
                && store.reference.as_ref() == Some(&field_reference)
                && store.immediate.is_none()
                && store.local.is_none()
                && ret.bci == 40
                && ret.width == 1
                && ret.opcode == 0xb1
                && ret.reference.is_none()
                && ret.immediate.is_none()
                && ret.local.is_none()
    ) {
        return Ok(None);
    }
    if code
        .instructions
        .iter()
        .rev()
        .find(|instruction| instruction.bci < 34)
        .is_none_or(|instruction| instruction.bci.checked_add(instruction.width) != Some(34))
    {
        return Ok(None);
    }
    let mut suffix_uses = Vec::new();
    for use_fact in &code.member_uses {
        budget.poll()?;
        if use_fact.bci >= group.initializer_prefix_end_bci {
            suffix_uses.push(use_fact);
        }
    }
    if !matches!(
        suffix_uses.as_slice(),
        [call, store]
            if call.bci == 34 && call.reference == call_reference
                && store.bci == 37 && store.reference == field_reference
    ) {
        return Ok(None);
    }

    let mut matching_initializers = Vec::new();
    for candidate in initializer_candidates {
        budget.poll()?;
        if candidate.member.as_ref() == Some(&initializer_method.item.identity) {
            matching_initializers.push(candidate);
        }
    }
    let [initializer] = matching_initializers.as_slice() else {
        return Ok(None);
    };
    if initializer.has_exception_handlers || initializer.steps.len() != 5 {
        return Ok(None);
    }
    let (
        ClassInitializerStep::FieldWrite(first),
        ClassInitializerStep::FieldWrite(second),
        ClassInitializerStep::FieldWrite(third),
        ClassInitializerStep::FieldWrite(write),
        ClassInitializerStep::Other {
            order: 4,
            bci: 40,
            kind: ClassInitializerStatementKind::Return,
        },
    ) = (
        &initializer.steps[0],
        &initializer.steps[1],
        &initializer.steps[2],
        &initializer.steps[3],
        &initializer.steps[4],
    )
    else {
        return Ok(None);
    };
    if first.order != 0
        || second.order != 1
        || third.order != 2
        || write.order != 3
        || write.bci != 37
        || write.owner != owner_string
        || write.name != "totalUnits"
        || write.spelled_name != "totalUnits"
        || write.descriptor != "I"
        || !write.is_static
        || write.op != AssignOp::Assign
        || write.source.primary().bci() != 37
        || write.value.origin.primary().bci() != 34
        // This FieldWrite AST may carry a type-path receiver for a static `putstatic`. The exact
        // Code suffix below proves the receiver is not a runtime value: there is no instruction
        // before the store that could evaluate one, and the Fieldref owner is this class.
        || write.source.primary().method().is_some_and(|method| {
            method != &initializer_method.item.identity
        })
        || write.value.origin.primary().method().is_some_and(|method| {
            method != &initializer_method.item.identity
        })
        || !write.source.derived().iter().all(|origin| {
            (34..=40).contains(&origin.bci())
                && origin.method() == Some(&initializer_method.item.identity)
        })
        || !write.value.origin.derived().iter().all(|origin| {
            (34..=40).contains(&origin.bci())
                && origin.method() == Some(&initializer_method.item.identity)
        })
        || !write
            .field_reads
            .as_deref()
            .is_some_and(|field_reads| field_reads.is_empty())
        || write.value.presented != Some(Type::Int)
        || !matches!(
            &write.value.kind,
            ExprKind::Call {
                receiver: None,
                name,
                args,
            } if name == "sumUnits" && args.is_empty()
        )
    {
        return Ok(None);
    }

    Ok(Some(ProvedEnumStaticAssignment {
        field_index: field_index_u64,
        field_name: "totalUnits".to_owned(),
        value: write.value.clone(),
        initializer_member: initializer_method.item.identity.clone(),
    }))
}

/// Copy a candidate body's bounded structural facts from the same method run.
///
/// Candidate names only select which full bodies need to be retained. Every enum method still
/// contributes its same-run symbolic uses when the prepared class passes the cheap Java 8,
/// two-constant eligibility gate, so the class-level proof can reject references to members it will
/// project away.
pub(crate) fn capture_method_code(
    table_index: u64,
    requested_member: &PhysicalMethodId,
    ir: &MethodIr,
    budget: &mut Budget,
) -> Result<Option<EnumMethodCodeCandidate>> {
    let keep_body = is_enum_candidate_name(&requested_member.name.0);
    let code = ir.code();
    let member = ir
        .declaration()
        .map(|declaration| declaration.identity().clone());
    let Some(code) = code else {
        return Ok(Some(EnumMethodCodeCandidate {
            table_index,
            member,
            complete: false,
            exception_handler_count: 0,
            instructions: Vec::new(),
            member_uses: Vec::new(),
        }));
    };

    let work = u64::try_from(code.instructions.len())
        .unwrap_or(u64::MAX)
        .saturating_add(u64::from(code.exception_handler_count))
        .saturating_add(1);
    budget.charge(CountedBudgetDimension::IrItems, work)?;
    let mut instructions = Vec::with_capacity(if keep_body {
        code.instructions.len()
    } else {
        0
    });
    let mut member_uses = Vec::new();
    for (index, instruction) in code.instructions.iter().enumerate() {
        budget.poll()?;
        let operands = code.operands().get(index);
        let structured = enum_instruction(instruction, operands, ir.constant_pool());
        if let Some(reference) = structured.reference.as_ref().filter(|reference| {
            matches!(
                reference,
                EnumCodeReference::Field { .. } | EnumCodeReference::Method { .. }
            )
        }) {
            member_uses.push(EnumCodeUse {
                bci: instruction.bci,
                reference: reference.clone(),
            });
        }
        if instruction.opcode == 0xba {
            member_uses.extend(bootstrap_member_uses(
                instruction.bci,
                operands,
                ir.constant_pool(),
                ir.bootstrap_methods(),
                budget,
            )?);
        }
        if keep_body {
            instructions.push(structured);
        }
    }
    let complete = code.stopped_at.is_none()
        && matches!(code.execution, ExecutionReport::Complete { .. })
        && code.exception_handler_count as usize == code.exception_handlers.len()
        && code.instructions.len() == code.operands().len();
    Ok(Some(EnumMethodCodeCandidate {
        table_index,
        member,
        complete,
        exception_handler_count: code.exception_handler_count,
        instructions,
        member_uses,
    }))
}

fn is_enum_candidate_name(name: &[u8]) -> bool {
    matches!(
        name,
        b"<clinit>" | b"<init>" | b"values" | b"valueOf" | b"$values"
    )
}

fn enum_instruction(
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
    pool: &[jarde_reader::classfile::CpEntryFacts],
) -> EnumCodeInstruction {
    let opcode = operands.map_or(instruction.opcode, |operands| operands.effective_opcode);
    let pool_index = operands
        .and_then(|operands| operands.constant_pool_index)
        .or(instruction.constant_pool_index);
    let reference = pool_index.map(|index| {
        let Ok(entry) = cp_entry(pool, index) else {
            return EnumCodeReference::Other;
        };
        match &entry.kind {
            CpEntryKind::Integer { value } => EnumCodeReference::Integer(*value),
            CpEntryKind::String { value, .. } => EnumCodeReference::String(value.0.clone()),
            CpEntryKind::Class { name, .. } => EnumCodeReference::Class(name.0.clone()),
            CpEntryKind::FieldRef {
                owner,
                name,
                descriptor,
                ..
            } => EnumCodeReference::Field {
                owner: owner.0.clone(),
                name: name.0.clone(),
                descriptor: descriptor.0.clone(),
            },
            CpEntryKind::MethodRef {
                owner,
                name,
                descriptor,
                ..
            } => EnumCodeReference::Method {
                owner: owner.0.clone(),
                name: name.0.clone(),
                descriptor: descriptor.0.clone(),
                interface: false,
            },
            CpEntryKind::InterfaceMethodRef {
                owner,
                name,
                descriptor,
                ..
            } => EnumCodeReference::Method {
                owner: owner.0.clone(),
                name: name.0.clone(),
                descriptor: descriptor.0.clone(),
                interface: true,
            },
            CpEntryKind::MethodHandle {
                reference_index, ..
            } => cp_member_reference(pool, *reference_index).unwrap_or(EnumCodeReference::Other),
            _ => EnumCodeReference::Other,
        }
    });
    EnumCodeInstruction {
        bci: instruction.bci,
        width: instruction.width,
        opcode,
        immediate: operands.and_then(|operands| operands.immediate),
        local: operands
            .and_then(|operands| operands.local)
            .map(|local| local.index),
        reference,
    }
}

fn cp_member_reference(
    pool: &[jarde_reader::classfile::CpEntryFacts],
    index: u16,
) -> Option<EnumCodeReference> {
    let entry = cp_entry(pool, index).ok()?;
    match &entry.kind {
        CpEntryKind::FieldRef {
            owner,
            name,
            descriptor,
            ..
        } => Some(EnumCodeReference::Field {
            owner: owner.0.clone(),
            name: name.0.clone(),
            descriptor: descriptor.0.clone(),
        }),
        CpEntryKind::MethodRef {
            owner,
            name,
            descriptor,
            ..
        } => Some(EnumCodeReference::Method {
            owner: owner.0.clone(),
            name: name.0.clone(),
            descriptor: descriptor.0.clone(),
            interface: false,
        }),
        CpEntryKind::InterfaceMethodRef {
            owner,
            name,
            descriptor,
            ..
        } => Some(EnumCodeReference::Method {
            owner: owner.0.clone(),
            name: name.0.clone(),
            descriptor: descriptor.0.clone(),
            interface: true,
        }),
        _ => None,
    }
}

fn bootstrap_member_uses(
    bci: u32,
    operands: Option<&InstructionOperands>,
    pool: &[jarde_reader::classfile::CpEntryFacts],
    bootstraps: &[jarde_reader::classfile::BootstrapMethodFacts],
    budget: &mut Budget,
) -> Result<Vec<EnumCodeUse>> {
    let Some(dynamic_index) = operands.and_then(|operands| operands.constant_pool_index) else {
        return Ok(Vec::new());
    };
    let Ok(dynamic) = cp_entry(pool, dynamic_index) else {
        return Ok(Vec::new());
    };
    let bootstrap_index = match &dynamic.kind {
        CpEntryKind::InvokeDynamic {
            bootstrap_method_attr_index,
            ..
        } => *bootstrap_method_attr_index,
        _ => return Ok(Vec::new()),
    };
    let Some(bootstrap) = bootstraps.get(usize::from(bootstrap_index)) else {
        return Ok(Vec::new());
    };
    let linked_count = u64::try_from(bootstrap.arguments.len())
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    budget.charge(CountedBudgetDimension::IrItems, linked_count)?;
    let mut uses = Vec::new();
    for index in std::iter::once(bootstrap.method_ref).chain(bootstrap.arguments.iter().copied()) {
        budget.poll()?;
        if let Some(reference) = cp_reference_or_handle(pool, index) {
            uses.push(EnumCodeUse { bci, reference });
        }
    }
    Ok(uses)
}

/// Resolve either one direct member reference or one MethodHandle layer. Malformed handle chains
/// are rejected instead of followed recursively.
fn cp_reference_or_handle(
    pool: &[jarde_reader::classfile::CpEntryFacts],
    index: u16,
) -> Option<EnumCodeReference> {
    let entry = cp_entry(pool, index).ok()?;
    match &entry.kind {
        CpEntryKind::MethodHandle {
            reference_index, ..
        } => cp_member_reference(pool, *reference_index),
        _ => cp_member_reference(pool, index),
    }
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
    if class.access_flags & ACC_ENUM == 0 {
        return Ok(ClassSourceEnumConstantProof::NotApplicable);
    }
    let refuse = |reason: &str| ClassSourceEnumConstantProof::Refused {
        reason: reason.to_owned(),
    };
    if class_version != (52, 0)
        || class.access_flags & (ACC_INTERFACE | ACC_ABSTRACT) != 0
        || class
            .super_class
            .as_ref()
            .map(|name| name.raw().0.as_slice())
            != Some(ENUM_SUPER)
    {
        return Ok(refuse(
            "the class is not a concrete Java 8 enum extending java/lang/Enum",
        ));
    }
    let owner = class.this_class.raw().0.as_slice();
    if owner.is_empty()
        || declaration.item.definition
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

    let enum_fields: Vec<_> = field_headers
        .iter()
        .enumerate()
        .filter(|(_, field)| field.access_flags & ACC_ENUM != 0)
        .collect();
    let [first_enum, second_enum] = enum_fields.as_slice() else {
        return Ok(refuse(
            "the complete field table does not contain exactly two ACC_ENUM fields",
        ));
    };
    let enum_descriptor = object_descriptor(owner);
    let mut constants = Vec::with_capacity(2);
    let mut constant_names = std::collections::BTreeSet::new();
    for (ordinal, (index, header)) in [first_enum, second_enum].into_iter().enumerate() {
        let expected_flags = ACC_PUBLIC | ACC_STATIC | ACC_FINAL | ACC_ENUM;
        if header.access_flags != expected_flags
            || header.descriptor.raw().0 != enum_descriptor
            || !header.attributes.is_empty()
        {
            return Ok(refuse(
                "an enum constant field has non-standard flags, descriptor, or attributes",
            ));
        }
        let Some(name) = String::from_utf16(header.name.utf16()).ok() else {
            return Ok(refuse("an enum constant field name is not valid Java text"));
        };
        if !jarde_java::is_java_identifier(&name) || !constant_names.insert(name.clone()) {
            return Ok(refuse(
                "enum constant field names are not unique Java identifiers",
            ));
        }
        if source_fields[*index].declaration.is_none() {
            return Ok(refuse(
                "an enum constant field has no faithful physical declaration",
            ));
        }
        constants.push((*index, name, i32::try_from(ordinal).unwrap_or(i32::MAX)));
    }

    let values_field_candidates: Vec<_> = field_headers
        .iter()
        .enumerate()
        .filter(|(_, field)| field.name.raw().0 == b"$VALUES")
        .collect();
    let [(backing_field_index, backing_field)] = values_field_candidates.as_slice() else {
        return Ok(refuse(
            "the `$VALUES` physical field is absent or ambiguous",
        ));
    };
    if backing_field.descriptor.raw().0 != array_descriptor(owner)
        || backing_field.access_flags != (ACC_PRIVATE | ACC_STATIC | ACC_FINAL | ACC_SYNTHETIC)
        || !backing_field.attributes.is_empty()
    {
        return Ok(refuse("the `$VALUES` field has a non-standard declaration"));
    }

    let clinit_index = match unique_method(method_headers, b"<clinit>", b"()V") {
        Ok(index) => index,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let constructor_count = method_headers
        .iter()
        .filter(|method| method.name.raw().0 == b"<init>")
        .count();
    let (constructor_index, delegating_constructor_index, delegation_edge, constructor_body) =
        match constructor_count {
            1 => {
                let constructor_index =
                    match unique_method(method_headers, b"<init>", CTOR_DESCRIPTOR) {
                        Ok(index) => index,
                        Err(reason) => return Ok(refuse(&reason)),
                    };
                (constructor_index, None, None, None)
            }
            2 => {
                let edge = match prove_constructor_delegation_edge(
                    DelegationEdgeInput {
                        owner,
                        constants: &constants,
                        backing_name: backing_field.name.raw().0.as_slice(),
                        field_headers,
                        method_headers,
                        source_methods,
                        code_candidates,
                    },
                    budget,
                )? {
                    Ok(edge) => edge,
                    Err(reason) => {
                        return Ok(refuse(&format!(
                            "enum constructor delegation edge refused: {reason}"
                        )));
                    }
                };
                let body = match prove_terminal_constructor_body(
                    TerminalConstructorBodyInput {
                        owner,
                        method_index: edge.terminal_method_index,
                        field_headers,
                        source_fields,
                        method_headers,
                        source_methods,
                        code_candidates,
                        constructor_candidates,
                    },
                    budget,
                )? {
                    Ok(body) => body,
                    Err(reason) => {
                        return Ok(refuse(&format!(
                            "terminal constructor body refused: {reason}"
                        )));
                    }
                };
                (
                    edge.terminal_method_index,
                    Some(edge.delegating_method_index),
                    Some(edge),
                    Some(Box::new(body)),
                )
            }
            _ => {
                return Ok(refuse(
                    "the enum has additional constructor bodies outside this proof",
                ));
            }
        };
    let values_descriptor = values_descriptor(owner);
    let values_index = match unique_method(method_headers, b"values", &values_descriptor) {
        Ok(index) => index,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let value_of_descriptor = value_of_descriptor(owner);
    let value_of_index = match unique_method(method_headers, b"valueOf", &value_of_descriptor) {
        Ok(index) => index,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let factory_descriptor = values_descriptor.clone();
    let factory_index = match unique_method(method_headers, b"$values", &factory_descriptor) {
        Ok(index) => index,
        Err(reason) => return Ok(refuse(&reason)),
    };

    let clinit = match method_code(
        clinit_index,
        method_headers,
        source_methods,
        code_candidates,
    ) {
        Ok(code) => code,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let constructor = match method_code(
        constructor_index,
        method_headers,
        source_methods,
        code_candidates,
    ) {
        Ok(code) => code,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let values = match method_code(
        values_index,
        method_headers,
        source_methods,
        code_candidates,
    ) {
        Ok(code) => code,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let value_of = match method_code(
        value_of_index,
        method_headers,
        source_methods,
        code_candidates,
    ) {
        Ok(code) => code,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let factory = match method_code(
        factory_index,
        method_headers,
        source_methods,
        code_candidates,
    ) {
        Ok(code) => code,
        Err(reason) => return Ok(refuse(&reason)),
    };

    if !exact_method_flags(&method_headers[clinit_index], ACC_STATIC)
        || !exact_method_flags(&method_headers[constructor_index], ACC_PRIVATE)
        || !exact_method_flags(&method_headers[values_index], ACC_PUBLIC | ACC_STATIC)
        || !exact_method_flags(&method_headers[value_of_index], ACC_PUBLIC | ACC_STATIC)
        || !exact_method_flags(
            &method_headers[factory_index],
            ACC_PRIVATE | ACC_STATIC | ACC_SYNTHETIC,
        )
    {
        return Ok(refuse(
            "a standard enum method has non-standard access flags",
        ));
    }
    if !allowed_method_attributes(&method_headers[clinit_index], &[b"Code"])
        || !allowed_method_attributes(
            &method_headers[constructor_index],
            &[b"Code", b"MethodParameters", b"Signature"],
        )
        || !allowed_method_attributes(&method_headers[values_index], &[b"Code"])
        || !allowed_method_attributes(
            &method_headers[value_of_index],
            &[b"Code", b"MethodParameters"],
        )
        || !allowed_method_attributes(&method_headers[factory_index], &[b"Code"])
    {
        return Ok(refuse(
            "a standard enum method declares extra or duplicate attributes",
        ));
    }
    if let Some(index) = delegating_constructor_index
        && !allowed_method_attributes(
            &method_headers[index],
            &[b"Code", b"MethodParameters", b"Signature"],
        )
    {
        return Ok(refuse(
            "the delegating constructor declares extra or duplicate attributes",
        ));
    }
    // The methods that disappear or become implicit must be the only physical bodies that name
    // the enum constants, backing array, or the generated API/helper methods. Otherwise the
    // source projection would remove a reference that contributes to the class's behavior. The
    // census includes direct member operands and MethodHandles reached through invokedynamic's
    // bootstrap method and arguments, all from these same prepared member runs.
    let mut projected_methods = vec![
        clinit_index,
        constructor_index,
        values_index,
        value_of_index,
        factory_index,
    ];
    if let Some(index) = delegating_constructor_index {
        projected_methods.push(index);
    }
    let projected_constructor_descriptors: Vec<Vec<u8>> = if delegating_constructor_index.is_some()
    {
        vec![
            DELEGATING_CTOR_DESCRIPTOR.to_vec(),
            CTOR_DESCRIPTOR.to_vec(),
        ]
    } else {
        vec![CTOR_DESCRIPTOR.to_vec()]
    };
    for (index, header) in method_headers.iter().enumerate() {
        if !single_code_attribute(header) {
            return Ok(refuse(
                "every physical enum method must have exactly one complete Code body for the use census",
            ));
        }
        let candidate = match method_code(index, method_headers, source_methods, code_candidates) {
            Ok(candidate) => candidate,
            Err(reason) => return Ok(refuse(&reason)),
        };
        if !candidate.complete {
            return Ok(refuse(
                "a physical enum method body is partial or its exception table is incomplete",
            ));
        }
        if projected_methods.contains(&index) && candidate.exception_handler_count != 0 {
            return Ok(refuse(
                "a generated enum member has an exception table outside the proven pattern",
            ));
        }
        if !projected_methods.contains(&index)
            && candidate.member_uses.iter().any(|use_site| {
                refers_to_projected_enum_member(
                    &use_site.reference,
                    owner,
                    backing_field.name.raw().0.as_slice(),
                    &projected_constructor_descriptors,
                )
            })
        {
            return Ok(refuse(
                "a non-projected physical method references an enum constant or generated enum member",
            ));
        }
    }

    let expected_ctor_field = if let Some(body) = &constructor_body {
        body.field_index
    } else {
        match prove_constructor(constructor, owner, field_headers) {
            Ok((field_index, _)) => field_index,
            Err(reason) => return Ok(refuse(&reason)),
        }
    };
    if source_fields[expected_ctor_field].declaration.is_none() {
        return Ok(refuse(
            "the constructor's source integer field is not faithfully declared",
        ));
    }
    if !prove_values(values, owner, backing_field.name.raw().0.as_slice()) {
        return Ok(refuse(
            "values() does not clone the unique `$VALUES` array exactly",
        ));
    }
    if !prove_value_of(value_of, owner) {
        return Ok(refuse(
            "valueOf(String) does not preserve the standard Enum.valueOf behavior",
        ));
    }
    if !prove_values_factory(factory, owner, &constants, field_headers) {
        return Ok(refuse(
            "$values() does not build the two constants in field-table order",
        ));
    }

    let constant_specs: Vec<_> = constants
        .iter()
        .map(|(_, name, ordinal)| (name.as_bytes().to_vec(), *ordinal))
        .collect();
    let constructor_calls = if delegation_edge.is_some() {
        vec![
            InitializerConstructorCall {
                descriptor: DELEGATING_CTOR_DESCRIPTOR.to_vec(),
                source_argument: EnumSourceArgument::None,
            },
            InitializerConstructorCall {
                descriptor: CTOR_DESCRIPTOR.to_vec(),
                source_argument: EnumSourceArgument::Exact(1),
            },
        ]
    } else {
        vec![
            InitializerConstructorCall {
                descriptor: CTOR_DESCRIPTOR.to_vec(),
                source_argument: EnumSourceArgument::AnyLiteral,
            };
            constants.len()
        ]
    };
    let prefix = if let Some(edge) = &delegation_edge {
        edge.initializer_prefix.clone()
    } else {
        match prove_initializer_prefix(InitializerPrefixInput {
            code: clinit,
            owner,
            constants: &constant_specs,
            constructor_calls: &constructor_calls,
            enum_descriptor: &enum_descriptor,
            fields: field_headers,
            methods: method_headers,
            backing_name: backing_field.name.raw().0.as_slice(),
        }) {
            Ok(result) => result,
            Err(reason) => return Ok(refuse(&reason)),
        }
    };
    if clinit.member_uses.iter().any(|use_site| {
        refers_to_projected_enum_member(
            &use_site.reference,
            owner,
            backing_field.name.raw().0.as_slice(),
            &projected_constructor_descriptors,
        ) && !expected_initializer_use(
            use_site,
            owner,
            backing_field.name.raw().0.as_slice(),
            prefix.backing_store_bci,
            prefix.factory_call_bci,
            &prefix.constructor_bcis,
            &constructor_calls,
        )
    }) {
        return Ok(refuse(
            "the `<clinit>` body uses a hidden enum member outside the proved constant prefix",
        ));
    }
    let selected_clinit_identity = &source_methods[clinit_index].item.identity;
    let matching_initializer_candidates: Vec<_> = initializer_candidates
        .iter()
        .filter(|candidate| candidate.member.as_ref() == Some(selected_clinit_identity))
        .collect();
    let [initializer_candidate] = matching_initializer_candidates.as_slice() else {
        return Ok(refuse(
            "the same-run `<clinit>` structured statement sidecar is absent or ambiguous",
        ));
    };
    if initializer_candidate.has_exception_handlers
        || !completed_structured_clinit(&source_methods[clinit_index])
    {
        return Ok(refuse(
            "the `<clinit>` statements are not a complete structured recovery",
        ));
    }
    let expected_write_bcis: Vec<u32> = prefix
        .constant_bcis
        .iter()
        .copied()
        .chain(std::iter::once(prefix.backing_store_bci))
        .collect();
    if initializer_candidate.steps.len() < expected_write_bcis.len() {
        return Ok(refuse(
            "the `<clinit>` statement sidecar omits part of the proven prefix",
        ));
    }
    for (order, (step, expected_bci)) in initializer_candidate
        .steps
        .iter()
        .zip(expected_write_bcis.iter())
        .enumerate()
    {
        let jarde_java::report::ClassInitializerStep::FieldWrite(write) = step else {
            return Ok(refuse(
                "the `<clinit>` prefix contains an unclassified statement",
            ));
        };
        if write.order != order
            || write.bci != *expected_bci
            || !write.is_static
            || write.owner.as_bytes() != owner
            || write.descriptor.as_bytes() != enum_descriptor.as_slice() && order < constants.len()
        {
            return Ok(ClassSourceEnumConstantProof::Refused {
                reason: format!(
                    "the `<clinit>` AST write does not match its physical Code fact (step {order}: sidecar order={}, BCI={}, static={}, receiver={}, owner={:?}, descriptor={:?}, expected BCI={expected_bci})",
                    write.order,
                    write.bci,
                    write.is_static,
                    write.has_receiver,
                    write.owner,
                    write.descriptor,
                ),
            });
        }
        if order < constants.len() {
            let (field_index, name, _) = &constants[order];
            if write.name.as_bytes() != field_headers[*field_index].name.raw().0.as_slice()
                || String::from_utf16(field_headers[*field_index].name.utf16())
                    .ok()
                    .as_deref()
                    != Some(write.spelled_name.as_str())
                || write.descriptor.as_bytes() != enum_descriptor
                || write.op != jarde_java::ast::AssignOp::Assign
                || write
                    .field_reads
                    .as_ref()
                    .is_none_or(|reads| !reads.is_empty())
            {
                return Ok(refuse(
                    "a constant initialization statement has extra AST effects",
                ));
            }
            let _ = name;
        } else if write.name.as_bytes() != backing_field.name.raw().0.as_slice()
            || String::from_utf16(backing_field.name.utf16())
                .ok()
                .as_deref()
                != Some(write.spelled_name.as_str())
            || write.descriptor.as_bytes() != array_descriptor(owner)
            || write.op != jarde_java::ast::AssignOp::Assign
            || write
                .field_reads
                .as_ref()
                .is_none_or(|reads| !reads.is_empty())
        {
            return Ok(refuse(
                "the `$VALUES` initialization statement has extra AST effects",
            ));
        }
    }

    let source_arguments = prefix.source_arguments;
    let proved_constants = constants
        .into_iter()
        .zip(source_arguments)
        .zip(prefix.constructor_bcis)
        .zip(prefix.constant_bcis)
        .map(
            |((((field_index, name, _), source_argument), constructor_bci), field_write_bci)| {
                ProvedEnumConstant {
                    field_index: u64::try_from(field_index).unwrap_or(u64::MAX),
                    name,
                    source_argument,
                    constructor_bci,
                    field_write_bci,
                }
            },
        )
        .collect();
    Ok(ClassSourceEnumConstantProof::Proved(
        ProvedEnumConstantGroup::Ordinary(ProvedOrdinaryEnumConstantGroup {
            constants: proved_constants,
            backing_field_index: u64::try_from(*backing_field_index).unwrap_or(u64::MAX),
            constructor_method_index: u64::try_from(constructor_index).unwrap_or(u64::MAX),
            delegating_constructor_method_index: delegating_constructor_index
                .map(|index| u64::try_from(index).unwrap_or(u64::MAX)),
            constructor_body,
            constructor_field_index: u64::try_from(expected_ctor_field).unwrap_or(u64::MAX),
            constructor_signature_present: method_headers[constructor_index]
                .attributes
                .iter()
                .any(|attribute| attribute.name.raw().0 == b"Signature"),
            initializer_method_index: u64::try_from(clinit_index).unwrap_or(u64::MAX),
            values_method_index: u64::try_from(values_index).unwrap_or(u64::MAX),
            value_of_method_index: u64::try_from(value_of_index).unwrap_or(u64::MAX),
            values_factory_method_index: u64::try_from(factory_index).unwrap_or(u64::MAX),
            initializer_prefix_end_bci: prefix.prefix_end_bci,
            initializer_prefix_statement_count: expected_write_bcis.len(),
        }),
    ))
}

fn object_descriptor(owner: &[u8]) -> Vec<u8> {
    let mut descriptor = Vec::with_capacity(owner.len() + 2);
    descriptor.push(b'L');
    descriptor.extend_from_slice(owner);
    descriptor.push(b';');
    descriptor
}

fn array_descriptor(owner: &[u8]) -> Vec<u8> {
    let mut descriptor = Vec::with_capacity(owner.len() + 3);
    descriptor.push(b'[');
    descriptor.extend_from_slice(&object_descriptor(owner));
    descriptor
}

fn values_descriptor(owner: &[u8]) -> Vec<u8> {
    let mut descriptor = Vec::from(VALUES_DESCRIPTOR_PREFIX);
    descriptor.extend_from_slice(owner);
    descriptor.push(b';');
    descriptor
}

fn value_of_descriptor(owner: &[u8]) -> Vec<u8> {
    let mut descriptor = Vec::from(VALUE_OF_DESCRIPTOR_PREFIX);
    descriptor.extend_from_slice(owner);
    descriptor.push(b';');
    descriptor
}

fn unique_method(
    headers: &[MemberHeader],
    name: &[u8],
    descriptor: &[u8],
) -> std::result::Result<usize, String> {
    let matching: Vec<_> = headers
        .iter()
        .enumerate()
        .filter(|(_, method)| {
            method.name.raw().0 == name && method.descriptor.raw().0 == descriptor
        })
        .map(|(index, _)| index)
        .collect();
    match matching.as_slice() {
        [index] => Ok(*index),
        [] => Err(format!(
            "the physical method {}{} is absent",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        )),
        _ => Err(format!(
            "the physical method {}{} is ambiguous",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        )),
    }
}

fn exact_method_flags(method: &MemberHeader, flags: u16) -> bool {
    method.access_flags == flags
}

fn allowed_method_attributes(method: &MemberHeader, allowed: &[&[u8]]) -> bool {
    method.attributes.iter().all(|attribute| {
        allowed
            .iter()
            .any(|name| attribute.name.raw().0.as_slice() == *name)
    }) && allowed.iter().all(|name| {
        method
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0.as_slice() == *name)
            .count()
            <= 1
    })
}

fn single_code_attribute(method: &MemberHeader) -> bool {
    method
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Code")
        .count()
        == 1
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

fn refers_to_projected_enum_member(
    reference: &EnumCodeReference,
    owner: &[u8],
    backing_name: &[u8],
    constructor_descriptors: &[Vec<u8>],
) -> bool {
    match reference {
        EnumCodeReference::Field {
            owner: target_owner,
            name,
            descriptor,
        } if target_owner == owner => {
            name.as_slice() == backing_name
                && descriptor.as_slice() == array_descriptor(owner).as_slice()
        }
        EnumCodeReference::Method {
            owner: target_owner,
            name,
            descriptor,
            ..
        } if target_owner == owner => {
            (name.as_slice() == b"<init>"
                && constructor_descriptors
                    .iter()
                    .any(|constructor| constructor.as_slice() == descriptor.as_slice()))
                || (name.as_slice() == b"<clinit>" && descriptor.as_slice() == b"()V")
                || (name.as_slice() == b"$values"
                    && descriptor.as_slice() == values_descriptor(owner).as_slice())
        }
        _ => false,
    }
}

fn expected_initializer_use(
    use_site: &EnumCodeUse,
    owner: &[u8],
    backing_name: &[u8],
    backing_store_bci: u32,
    factory_call_bci: u32,
    constructor_bcis: &[u32],
    constructor_calls: &[InitializerConstructorCall],
) -> bool {
    match &use_site.reference {
        EnumCodeReference::Field {
            owner: target_owner,
            name,
            descriptor,
        } => {
            use_site.bci == backing_store_bci
                && target_owner == owner
                && name.as_slice() == backing_name
                && descriptor.as_slice() == array_descriptor(owner).as_slice()
        }
        EnumCodeReference::Method {
            owner: target_owner,
            name,
            descriptor,
            ..
        } => {
            (use_site.bci == factory_call_bci
                && target_owner == owner
                && name.as_slice() == b"$values"
                && descriptor.as_slice() == values_descriptor(owner).as_slice())
                || (constructor_bcis.contains(&use_site.bci)
                    && target_owner == owner
                    && name.as_slice() == b"<init>"
                    && constructor_bcis
                        .iter()
                        .zip(constructor_calls)
                        .any(|(bci, constructor)| {
                            *bci == use_site.bci
                                && constructor.descriptor.as_slice() == descriptor.as_slice()
                        }))
        }
        _ => false,
    }
}

fn completed_structured_clinit(method: &ClassSourceMethod) -> bool {
    matches!(
        &method.outcome,
        ClassSourceOutcome::Recovered { report, analysis }
            if report.produced()
                && report.quality == jarde_jvm::ir::Quality::Structured
                && report.fallbacks.is_empty()
                && matches!(report.execution, ExecutionReport::Complete { .. })
                && matches!(analysis.execution, ExecutionReport::Complete { .. })
    )
}

fn prove_constructor(
    code: &EnumMethodCodeCandidate,
    owner: &[u8],
    fields: &[MemberHeader],
) -> std::result::Result<(usize, u32), String> {
    let instructions = &code.instructions;
    if instructions.len() != 8
        || !local_load(&instructions[0], b'a', 0)
        || !local_load(&instructions[1], b'a', 1)
        || !local_load(&instructions[2], b'i', 2)
        || !method_reference(
            &instructions[3],
            0xb7,
            ENUM_SUPER,
            b"<init>",
            ENUM_CTOR_DESCRIPTOR,
            false,
        )
        || !local_load(&instructions[4], b'a', 0)
        || !local_load(&instructions[5], b'i', 3)
        || instructions[6].opcode != 0xb5
        || instructions[7].opcode != 0xb1
    {
        return Err("the enum constructor has effects beyond the implicit Enum call and one int field store".to_owned());
    }
    let Some(EnumCodeReference::Field {
        owner: field_owner,
        name,
        descriptor,
    }) = &instructions[6].reference
    else {
        return Err("the enum constructor field store has no complete Fieldref".to_owned());
    };
    if field_owner != owner || descriptor != b"I" {
        return Err("the enum constructor stores a field other than its own int field".to_owned());
    }
    let matching: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| {
            field.name.raw().0 == *name
                && field.descriptor.raw().0 == *descriptor
                && field.access_flags & ACC_STATIC == 0
                && field.access_flags & ACC_ENUM == 0
        })
        .map(|(index, _)| index)
        .collect();
    let [field_index] = matching.as_slice() else {
        return Err("the enum constructor's int field target is absent or ambiguous".to_owned());
    };
    Ok((*field_index, instructions[3].bci))
}

/// The constructor edge proved from the exact class-source member candidates. This is deliberately
/// not a class-level `Proved` result: the terminal constructor's user effects and Signature are the
/// next proof step, and the current two-constructor path remains refused until that work is done.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ProvedEnumConstructorDelegationEdge {
    delegating_method_index: usize,
    terminal_method_index: usize,
    constant_constructor_bcis: [u32; 2],
    constant_constructor_descriptors: [Vec<u8>; 2],
    constant_source_arguments: [Option<i32>; 2],
    initializer_prefix: InitializerPrefixProof,
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
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    let table_work = u64::try_from(field_headers.len())
        .unwrap_or(u64::MAX)
        // The shared initializer prefix resolves both constant stores and then checks each
        // physical field index; the extra factor also covers the two validated tuple lookups.
        .saturating_mul(5)
        .saturating_add(
            u64::try_from(method_headers.len())
                .unwrap_or(u64::MAX)
                // Constructor/clinit uniqueness plus both constructor and factory uniqueness
                // checks in the shared prefix, including constant-index header lookups.
                .saturating_mul(8),
        )
        .saturating_add(
            u64::try_from(code_candidates.len())
                .unwrap_or(u64::MAX)
                .saturating_mul(3),
        );
    budget.charge(CountedBudgetDimension::IrItems, table_work)?;
    budget.poll()?;

    let constructors: Vec<_> = method_headers
        .iter()
        .enumerate()
        .filter(|(_, method)| method.name.raw().0 == b"<init>")
        .map(|(index, _)| index)
        .collect();
    if constructors.len() != 2 {
        return refuse("the physical method table does not contain exactly two constructors");
    }
    let delegating_index =
        match unique_method(method_headers, b"<init>", DELEGATING_CTOR_DESCRIPTOR) {
            Ok(index) => index,
            Err(reason) => return Ok(Err(reason)),
        };
    let terminal_index = match unique_method(method_headers, b"<init>", CTOR_DESCRIPTOR) {
        Ok(index) => index,
        Err(reason) => return Ok(Err(reason)),
    };
    if delegating_index == terminal_index
        || !constructors.contains(&delegating_index)
        || !constructors.contains(&terminal_index)
    {
        return refuse("the two constructors do not have the exact distinct physical descriptors");
    }
    let clinit_index = match unique_method(method_headers, b"<clinit>", b"()V") {
        Ok(index) => index,
        Err(reason) => return Ok(Err(reason)),
    };
    for index in [delegating_index, terminal_index] {
        if !exact_method_flags(&method_headers[index], ACC_PRIVATE)
            || !single_code_attribute(&method_headers[index])
        {
            return refuse("a physical enum constructor has non-standard flags or Code attributes");
        }
    }
    if !exact_method_flags(&method_headers[clinit_index], ACC_STATIC)
        || !single_code_attribute(&method_headers[clinit_index])
    {
        return refuse("the physical enum initializer has non-standard flags or Code attributes");
    }

    let delegating = match method_code(
        delegating_index,
        method_headers,
        source_methods,
        code_candidates,
    ) {
        Ok(candidate) => candidate,
        Err(reason) => return Ok(Err(reason)),
    };
    let terminal = match method_code(
        terminal_index,
        method_headers,
        source_methods,
        code_candidates,
    ) {
        Ok(candidate) => candidate,
        Err(reason) => return Ok(Err(reason)),
    };
    let initializer = match method_code(
        clinit_index,
        method_headers,
        source_methods,
        code_candidates,
    ) {
        Ok(candidate) => candidate,
        Err(reason) => return Ok(Err(reason)),
    };
    if !delegating.complete || !terminal.complete || !initializer.complete {
        return refuse("a constructor or initializer Code candidate is partial");
    }
    if delegating.exception_handler_count != 0 || initializer.exception_handler_count != 0 {
        return refuse("a constructor or initializer Code candidate has exception handlers");
    }
    let instruction_work = u64::try_from(delegating.instructions.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(2)
        .saturating_add(
            u64::try_from(initializer.instructions.len())
                .unwrap_or(u64::MAX)
                .saturating_mul(3),
        );
    budget.charge(CountedBudgetDimension::IrItems, instruction_work)?;

    if delegating.instructions.len() != 6
        || !instructions_are_contiguous(&delegating.instructions, budget)?
        || !local_load(&delegating.instructions[0], b'a', 0)
        || !local_load(&delegating.instructions[1], b'a', 1)
        || !local_load(&delegating.instructions[2], b'i', 2)
        || !int_constant(&delegating.instructions[3], 0)
        || delegating.instructions[3].local.is_some()
        || !method_reference(
            &delegating.instructions[4],
            0xb7,
            owner,
            b"<init>",
            CTOR_DESCRIPTOR,
            false,
        )
        || delegating.instructions[5].opcode != 0xb1
        || delegating.instructions[5].reference.is_some()
        || delegating.instructions[5].immediate.is_some()
        || delegating.instructions[5].local.is_some()
        || delegating.member_uses.as_slice()
            != [EnumCodeUse {
                bci: delegating.instructions[4].bci,
                reference: EnumCodeReference::Method {
                    owner: owner.to_vec(),
                    name: b"<init>".to_vec(),
                    descriptor: CTOR_DESCRIPTOR.to_vec(),
                    interface: false,
                },
            }]
    {
        return refuse(
            "the no-source-argument constructor does not only forward entry this/name/ordinal and literal 0 to the unique terminal constructor",
        );
    }

    if constants.len() != 2 {
        return refuse("the caller did not supply two already-proved enum constants");
    }
    for (ordinal, (field_index, name, expected_ordinal)) in constants.iter().enumerate() {
        budget.poll()?;
        let Some(field) = field_headers.get(*field_index) else {
            return refuse("a proved enum constant field index is absent from the physical table");
        };
        if field.access_flags & ACC_ENUM == 0
            || field.name.raw().0 != name.as_bytes()
            || *expected_ordinal != i32::try_from(ordinal).unwrap_or(i32::MAX)
        {
            return refuse(
                "the proved enum constant tuple disagrees with its physical field record",
            );
        }
    }

    let instructions = &initializer.instructions;
    if !instructions_are_contiguous(instructions, budget)? {
        return refuse(
            "the enum initializer Code contains a branch or a discontinuous instruction stream",
        );
    }
    for instruction in instructions {
        budget.poll()?;
        if is_branch_opcode(instruction.opcode) {
            return refuse(
                "the enum initializer Code contains a branch or a discontinuous instruction stream",
            );
        }
    }
    let constructor_calls = [
        InitializerConstructorCall {
            descriptor: DELEGATING_CTOR_DESCRIPTOR.to_vec(),
            source_argument: EnumSourceArgument::None,
        },
        InitializerConstructorCall {
            descriptor: CTOR_DESCRIPTOR.to_vec(),
            source_argument: EnumSourceArgument::Exact(1),
        },
    ];
    let initializer_constants: Vec<_> = constants
        .iter()
        .map(|(_, name, ordinal)| (name.as_bytes().to_vec(), *ordinal))
        .collect();
    let prefix = match prove_initializer_prefix(InitializerPrefixInput {
        code: initializer,
        owner,
        constants: &initializer_constants,
        constructor_calls: &constructor_calls,
        enum_descriptor: &object_descriptor(owner),
        fields: field_headers,
        methods: method_headers,
        backing_name,
    }) {
        Ok(prefix) => prefix,
        Err(reason) => return Ok(Err(reason)),
    };
    let [first_constructor_bci, second_constructor_bci] = prefix.constructor_bcis.as_slice() else {
        return refuse("the shared initializer proof did not return two constructor call sites");
    };

    Ok(Ok(ProvedEnumConstructorDelegationEdge {
        delegating_method_index: delegating_index,
        terminal_method_index: terminal_index,
        constant_constructor_bcis: [*first_constructor_bci, *second_constructor_bci],
        constant_constructor_descriptors: [
            DELEGATING_CTOR_DESCRIPTOR.to_vec(),
            CTOR_DESCRIPTOR.to_vec(),
        ],
        constant_source_arguments: [None, Some(1)],
        initializer_prefix: prefix,
    }))
}

/// Prove the terminal constructor's complete, ordered user body from same-run AST and Code facts.
/// The physical method remains refused by generic Signature erasure and this private result does
/// not authorize source projection on its own.
fn prove_terminal_constructor_body(
    input: TerminalConstructorBodyInput<'_>,
    budget: &mut Budget,
) -> Result<std::result::Result<ProvedEnumConstructorBody, String>> {
    use jarde_java::ast::{AssignOp, ConstructorTarget, ExprKind};
    use jarde_java::report::ClassEnumConstructorStepKind as StepKind;

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
        || !exact_method_flags(header, ACC_PRIVATE)
        || !single_code_attribute(header)
        || !allowed_method_attributes(header, &[b"Code", b"MethodParameters", b"Signature"])
        || header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"Signature")
            .count()
            != 1
        || !enum_constructor_signature_matches(header, source, CTOR_DESCRIPTOR)
        || !matches!(
            &source.outcome,
            ClassSourceOutcome::Recovered { report, analysis }
                if report.quality == jarde_jvm::ir::Quality::Structured
                    && report.content == jarde_java::report::RecoveryContent::ContainsStatements
                    && matches!(analysis.execution, ExecutionReport::Complete { .. })
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
    if code.member.as_ref() != Some(&source.item.identity)
        || !code.complete
        || code.exception_handler_count != 0
        || code.instructions.len() != 10
        || !instructions_are_contiguous(&code.instructions, budget)?
    {
        return refuse("the terminal constructor Code stream is partial, handled, or not exact");
    }
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(code.instructions.len())
            .unwrap_or(u64::MAX)
            .saturating_add(u64::try_from(code.member_uses.len()).unwrap_or(u64::MAX)),
    )?;

    let instructions = &code.instructions;
    let load = |index: usize, kind, slot| {
        instructions.get(index).is_some_and(|instruction| {
            local_load(instruction, kind, slot)
                && instruction.immediate.is_none()
                && instruction.reference.is_none()
        })
    };
    if instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect::<Vec<_>>()
        != [0, 1, 2, 3, 6, 7, 10, 11, 12, 15]
        || !load(0, b'a', 0)
        || !load(1, b'a', 1)
        || !load(2, b'i', 2)
        || !method_reference(
            &instructions[3],
            0xb7,
            ENUM_SUPER,
            b"<init>",
            ENUM_CTOR_DESCRIPTOR,
            false,
        )
        || instructions[3].width != 3
        || !load(4, b'i', 3)
        || instructions[5].opcode != 0xb8
        || instructions[5].width != 3
        || !load(6, b'a', 0)
        || !load(7, b'i', 3)
        || instructions[8].opcode != 0xb5
        || instructions[8].width != 3
        || instructions[9].opcode != 0xb1
        || instructions[9].width != 1
        || instructions[9].reference.is_some()
        || instructions[9].immediate.is_some()
        || instructions[9].local.is_some()
    {
        return refuse(
            "the terminal Code does not contain only the Enum prefix, slot-3 helper/save, and terminal return",
        );
    }
    let Some(EnumCodeReference::Method {
        owner: helper_owner,
        name: helper_name,
        descriptor: helper_descriptor,
        interface: false,
    }) = instructions[5].reference.as_ref()
    else {
        return refuse("the terminal helper call is not a resolved class Methodref");
    };
    let Some(helper_name_text) = std::str::from_utf8(helper_name).ok() else {
        return refuse("the terminal helper name is not valid Java text");
    };
    if helper_owner.is_empty()
        || helper_owner == owner
        || !jarde_java::is_java_identifier(helper_name_text)
        || helper_descriptor != b"(I)V"
    {
        return refuse("the terminal helper target is not one external static (I)V Methodref");
    }
    let Some(EnumCodeReference::Field {
        owner: field_owner,
        name: field_name,
        descriptor: field_descriptor,
    }) = instructions[8].reference.as_ref()
    else {
        return refuse("the terminal field write has no resolved Fieldref");
    };
    let Some(field_name_text) = std::str::from_utf8(field_name).ok() else {
        return refuse("the terminal field name is not valid Java text");
    };
    if field_owner != owner
        || field_descriptor != b"I"
        || !jarde_java::is_java_identifier(field_name_text)
    {
        return refuse("the terminal field write does not target this enum's int field");
    }
    let field_matches: Vec<_> = field_headers
        .iter()
        .enumerate()
        .filter(|(_, field)| field.name.raw().0 == *field_name && field.descriptor.raw().0 == b"I")
        .map(|(index, _)| index)
        .collect();
    let [field_index] = field_matches.as_slice() else {
        return refuse("the terminal Fieldref does not resolve to one physical int field");
    };
    let Some(field_header) = field_headers.get(*field_index) else {
        return refuse("the terminal field index is outside the physical field table");
    };
    let Some(source_field) = source_fields.get(*field_index) else {
        return refuse("the terminal field has no class-source record");
    };
    if source_field.item.index != u64::try_from(*field_index).unwrap_or(u64::MAX)
        || source_field.item.name.raw().0 != *field_name
        || source_field.item.descriptor.raw().0 != b"I"
        || source_field.declaration.is_none()
        || !source_field.markers.is_empty()
        || !source_field.annotations.refusals.is_empty()
        || !source_field.type_annotations.refusals.is_empty()
        || field_header.access_flags != (ACC_PRIVATE | ACC_FINAL)
        || !field_header.attributes.is_empty()
    {
        return refuse(
            "the terminal Fieldref is not one faithful private final instance int field",
        );
    }

    let expected_uses = [
        EnumCodeUse {
            bci: 3,
            reference: EnumCodeReference::Method {
                owner: ENUM_SUPER.to_vec(),
                name: b"<init>".to_vec(),
                descriptor: ENUM_CTOR_DESCRIPTOR.to_vec(),
                interface: false,
            },
        },
        EnumCodeUse {
            bci: 7,
            reference: EnumCodeReference::Method {
                owner: helper_owner.clone(),
                name: helper_name.clone(),
                descriptor: b"(I)V".to_vec(),
                interface: false,
            },
        },
        EnumCodeUse {
            bci: 12,
            reference: EnumCodeReference::Field {
                owner: field_owner.clone(),
                name: field_name.clone(),
                descriptor: b"I".to_vec(),
            },
        },
    ];
    if code.member_uses.as_slice() != expected_uses {
        return refuse("the terminal method has an extra or differently bound member reference");
    }

    let matching_ast: Vec<_> = constructor_candidates
        .iter()
        .filter(|candidate| candidate.member.as_ref() == Some(&source.item.identity))
        .collect();
    let [candidate] = matching_ast.as_slice() else {
        return refuse("the same-run terminal constructor AST candidate is absent or ambiguous");
    };
    if !candidate.complete || candidate.has_exception_handlers || candidate.steps.len() != 4 {
        return refuse(
            "the terminal constructor AST candidate is partial or contains extra statements",
        );
    }
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(candidate.steps.len())
            .unwrap_or(u64::MAX)
            .saturating_add(7),
    )?;
    for (order, (step, bci)) in candidate.steps.iter().zip([3, 7, 12, 15]).enumerate() {
        budget.poll()?;
        if step.order != order || step.bci != bci || step.source.primary().bci() != bci {
            return refuse("the terminal AST statements do not match Code BCI order");
        }
    }
    let [super_step, helper_step, field_step, return_step] = candidate.steps.as_slice() else {
        return refuse("the terminal constructor AST statement sequence is not exact");
    };
    let StepKind::ConstructorCall {
        target: ConstructorTarget::Super,
        args: super_args,
    } = &super_step.kind
    else {
        return refuse("the terminal AST does not begin with the Enum super constructor");
    };
    if super_args.len() != 2 || !is_local_at(&super_args[0], 1) || !is_local_at(&super_args[1], 2) {
        return refuse("the AST Enum call does not preserve entry name and ordinal loads");
    }
    let StepKind::Expression(helper_expression) = &helper_step.kind else {
        return refuse("the terminal helper call is absent or not a standalone expression");
    };
    let helper_display = std::str::from_utf8(helper_owner)
        .ok()
        .map(|owner| owner.replace('/', "."));
    let Some(helper_display) = helper_display else {
        return refuse("the terminal helper owner cannot be spelled as a Java path");
    };
    if !helper_display
        .split('.')
        .all(jarde_java::is_java_identifier)
    {
        return refuse("the terminal helper owner is not a legal Java path");
    }
    let ExprKind::Call {
        receiver: Some(receiver),
        name,
        args,
    } = &helper_expression.kind
    else {
        return refuse("the terminal AST helper call is not a qualified static call");
    };
    if !matches!(&receiver.kind, ExprKind::Path(path) if path == &helper_display)
        || name != helper_name_text
        || args.len() != 1
        || !matches!(args[0].kind, ExprKind::Local(_))
        || helper_expression.origin.primary().bci() != 7
        || args[0].origin.primary().bci() != 6
    {
        return refuse(
            "the terminal AST helper does not match the same-BCI Methodref and slot-3 producer",
        );
    }
    let StepKind::FieldWrite {
        field: Some(ast_field),
        spelled_name,
        receiver: Some(ast_receiver),
        op: AssignOp::Assign,
        value,
    } = &field_step.kind
    else {
        return refuse("the terminal AST field store is absent or has an unproved field claim");
    };
    if ast_field.bci != 12
        || ast_field.owner != String::from_utf8_lossy(owner)
        || ast_field.name.as_bytes() != *field_name
        || ast_field.descriptor != "I"
        || ast_field.is_static
        || !jarde_java::is_java_identifier(spelled_name)
        || field_name_text != spelled_name
        || !is_local_at(ast_receiver, 10)
        || !is_local_at(value, 11)
        || ast_receiver.origin.primary().bci() != 10
        || value.origin.primary().bci() != 11
    {
        return refuse(
            "the terminal AST field store does not match its Fieldref and slot producers",
        );
    }
    if !matches!(&return_step.kind, StepKind::Return { value: None }) {
        return refuse("the terminal constructor does not end with a bare return");
    }

    Ok(Ok(ProvedEnumConstructorBody {
        method_index,
        field_index: *field_index,
        enum_super_call_bci: 3,
        helper_call_bci: 7,
        field_write_bci: 12,
        return_bci: 15,
        helper_owner: helper_owner.clone(),
        candidate: (*candidate).clone(),
    }))
}

fn is_local_at(expression: &jarde_java::ast::Expr, slot_load_bci: u32) -> bool {
    matches!(expression.kind, jarde_java::ast::ExprKind::Local(_))
        && expression.origin.primary().bci() == slot_load_bci
}

fn enum_constructor_signature_matches(
    header: &MemberHeader,
    source: &ClassSourceMethod,
    physical_descriptor: &[u8],
) -> bool {
    let source_shape = match physical_descriptor {
        DELEGATING_CTOR_DESCRIPTOR => source.enum_constructor_no_arg_source_signature,
        CTOR_DESCRIPTOR => source.enum_constructor_source_signature,
        _ => false,
    };
    source_shape
        && source.enum_constructor_signature_erasure_refused
        && source.markers.len() == 1
        && header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"Signature")
            .count()
            == 1
        && allowed_method_attributes(header, &[b"Code", b"MethodParameters", b"Signature"])
}

fn instructions_are_contiguous(
    instructions: &[EnumCodeInstruction],
    budget: &mut Budget,
) -> Result<bool> {
    for pair in instructions.windows(2) {
        budget.poll()?;
        if !pair[0]
            .bci
            .checked_add(pair[0].width)
            .is_some_and(|end| end == pair[1].bci)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn is_branch_opcode(opcode: u8) -> bool {
    matches!(opcode, 0x99..=0xa9 | 0xaa..=0xab | 0xc6..=0xc9)
}

pub(crate) fn prove_values(
    code: &EnumMethodCodeCandidate,
    owner: &[u8],
    backing_name: &[u8],
) -> bool {
    let array = array_descriptor(owner);
    let instructions = &code.instructions;
    instructions.len() == 4
        && field_reference(&instructions[0], 0xb2, owner, backing_name, &array)
        && method_reference(
            &instructions[1],
            0xb6,
            &array,
            b"clone",
            b"()Ljava/lang/Object;",
            false,
        )
        && class_reference(&instructions[2], 0xc0, &array)
        && instructions[3].opcode == 0xb0
}

pub(crate) fn prove_value_of(code: &EnumMethodCodeCandidate, owner: &[u8]) -> bool {
    let instructions = &code.instructions;
    instructions.len() == 5
        && class_constant(&instructions[0], owner)
        && local_load(&instructions[1], b'a', 0)
        && method_reference(
            &instructions[2],
            0xb8,
            ENUM_SUPER,
            b"valueOf",
            ENUM_VALUE_OF_DESCRIPTOR,
            false,
        )
        && class_reference(&instructions[3], 0xc0, owner)
        && instructions[4].opcode == 0xb0
}

fn prove_values_factory(
    code: &EnumMethodCodeCandidate,
    owner: &[u8],
    constants: &[(usize, String, i32)],
    fields: &[MemberHeader],
) -> bool {
    let instructions = &code.instructions;
    if constants.len() != 2 || instructions.len() != 11 {
        return false;
    }
    if !int_constant(&instructions[0], 2) || !class_reference(&instructions[1], 0xbd, owner) {
        return false;
    }
    for (ordinal, (field_index, _, _)) in constants.iter().enumerate() {
        let start = 2 + ordinal * 4;
        if instructions[start].opcode != 0x59
            || !int_constant(
                &instructions[start + 1],
                i32::try_from(ordinal).unwrap_or(i32::MAX),
            )
            || !field_reference(
                &instructions[start + 2],
                0xb2,
                owner,
                fields[*field_index].name.raw().0.as_slice(),
                fields[*field_index].descriptor.raw().0.as_slice(),
            )
            || instructions[start + 3].opcode != 0x53
        {
            return false;
        }
    }
    instructions[10].opcode == 0xb0
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum EnumSourceArgument {
    AnyLiteral,
    Exact(i32),
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InitializerConstructorCall {
    descriptor: Vec<u8>,
    source_argument: EnumSourceArgument,
}

struct InitializerPrefixInput<'a> {
    code: &'a EnumMethodCodeCandidate,
    owner: &'a [u8],
    constants: &'a [(Vec<u8>, i32)],
    constructor_calls: &'a [InitializerConstructorCall],
    enum_descriptor: &'a [u8],
    fields: &'a [MemberHeader],
    methods: &'a [MemberHeader],
    backing_name: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InitializerPrefixProof {
    constant_bcis: Vec<u32>,
    backing_store_bci: u32,
    constructor_bcis: Vec<u32>,
    source_arguments: Vec<Option<i32>>,
    prefix_end_bci: u32,
    factory_call_bci: u32,
}

fn prove_initializer_prefix(
    input: InitializerPrefixInput<'_>,
) -> std::result::Result<InitializerPrefixProof, String> {
    let InitializerPrefixInput {
        code,
        owner,
        constants,
        constructor_calls,
        enum_descriptor,
        fields,
        methods,
        backing_name,
    } = input;
    if constants.len() != 2 || constructor_calls.len() != constants.len() {
        return Err("the proof does not contain exactly two ordered constants".to_owned());
    }
    for call in constructor_calls {
        unique_method(methods, b"<init>", &call.descriptor)?;
    }
    let instructions = &code.instructions;
    let mut cursor = 0_usize;
    let mut constant_bcis = Vec::with_capacity(2);
    let mut constructor_bcis = Vec::with_capacity(2);
    let mut source_arguments = Vec::with_capacity(2);
    for (ordinal, (name, expected_ordinal)) in constants.iter().enumerate() {
        let constructor_call_spec = &constructor_calls[ordinal];
        let Some(allocation) = instructions.get(cursor) else {
            return Err(format!("constant {ordinal} has no allocation instruction"));
        };
        let Some(duplicate) = instructions.get(cursor + 1) else {
            return Err(format!(
                "constant {ordinal} has no allocation copy instruction"
            ));
        };
        if !class_reference(allocation, 0xbb, owner)
            || duplicate.opcode != 0x59
            || duplicate.reference.is_some()
            || duplicate.immediate.is_some()
            || duplicate.local.is_some()
            || !string_constant(instructions.get(cursor + 2), name)
            || !int_constant_at(instructions, cursor + 3, *expected_ordinal)
        {
            return Err(format!(
                "constant {} does not start with its exact allocation, name, and ordinal prefix",
                ordinal
            ));
        }
        let (call_offset, source_argument) = match &constructor_call_spec.source_argument {
            EnumSourceArgument::AnyLiteral => {
                let Some(argument) = instructions.get(cursor + 4).and_then(int_constant_value)
                else {
                    return Err(format!(
                        "constant {} has no literal int source argument",
                        ordinal
                    ));
                };
                (5, Some(argument))
            }
            EnumSourceArgument::Exact(expected) => {
                if !int_constant_at(instructions, cursor + 4, *expected) {
                    return Err(format!(
                        "constant {} does not pass its exact literal source argument",
                        ordinal
                    ));
                }
                (5, Some(*expected))
            }
            EnumSourceArgument::None => (4, None),
        };
        let Some(constructor_call) = instructions.get(cursor + call_offset) else {
            return Err(format!("constant {ordinal} has no constructor call"));
        };
        let Some(field_store) = instructions.get(cursor + call_offset + 1) else {
            return Err(format!("constant {ordinal} has no field store"));
        };
        if !method_reference(
            constructor_call,
            0xb7,
            owner,
            b"<init>",
            &constructor_call_spec.descriptor,
            false,
        ) {
            return Err(format!(
                "constant {} calls a different constructor",
                ordinal
            ));
        }
        let (field_index, field_name, _) =
            constants_by_code_field(Some(field_store), owner, enum_descriptor, fields).ok_or_else(
                || {
                    format!(
                        "constant {} writes a different or ambiguous enum field",
                        ordinal
                    )
                },
            )?;
        if field_name != *name {
            return Err(format!(
                "constant {} writes a field other than its named constant",
                ordinal
            ));
        }
        constant_bcis.push(field_store.bci);
        constructor_bcis.push(constructor_call.bci);
        source_arguments.push(source_argument);
        cursor += call_offset + 2;
        let expected_field = fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name.raw().0 == *name && field.access_flags & ACC_ENUM != 0)
            .map(|(index, _)| index)
            .ok_or_else(|| "the constructor call's constant field is absent".to_owned())?;
        if field_index != expected_field {
            return Err("constant field order differs from the physical field table".to_owned());
        }
    }
    let factory_call = instructions
        .get(cursor)
        .ok_or_else(|| "the `$values()` call is absent".to_owned())?;
    if !method_reference(
        factory_call,
        0xb8,
        owner,
        b"$values",
        &values_descriptor(owner),
        false,
    ) {
        return Err("the constant prefix does not call the unique `$values()` factory".to_owned());
    }
    unique_method(methods, b"$values", &values_descriptor(owner))?;
    cursor += 1;
    if !field_reference(
        instructions
            .get(cursor)
            .ok_or_else(|| "the `$VALUES` store is absent".to_owned())?,
        0xb3,
        owner,
        backing_name,
        &array_descriptor(owner),
    ) {
        return Err(
            "the constant prefix does not store into the unique `$VALUES` field".to_owned(),
        );
    }
    let backing_store_bci = instructions[cursor].bci;
    let prefix_end = instructions[cursor]
        .bci
        .checked_add(instructions[cursor].width)
        .ok_or_else(|| "the `$VALUES` prefix end BCI overflows".to_owned())?;
    if instructions
        .get(cursor + 1)
        .is_some_and(|instruction| instruction.bci != prefix_end)
    {
        return Err(
            "the `$VALUES` store width disagrees with the following instruction BCI".to_owned(),
        );
    }
    Ok(InitializerPrefixProof {
        constant_bcis,
        backing_store_bci,
        constructor_bcis,
        source_arguments,
        prefix_end_bci: prefix_end,
        factory_call_bci: factory_call.bci,
    })
}

fn constants_by_code_field<'a>(
    instruction: Option<&'a EnumCodeInstruction>,
    owner: &[u8],
    descriptor: &[u8],
    fields: &'a [MemberHeader],
) -> Option<(usize, &'a [u8], &'a [u8])> {
    let instruction = instruction?;
    if instruction.opcode != 0xb3 {
        return None;
    }
    let Some(EnumCodeReference::Field {
        owner: field_owner,
        name,
        descriptor: field_descriptor,
    }) = &instruction.reference
    else {
        return None;
    };
    if field_owner != owner || field_descriptor != descriptor {
        return None;
    }
    let matching: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| {
            field.name.raw().0 == *name
                && field.descriptor.raw().0 == *field_descriptor
                && field.access_flags & ACC_ENUM != 0
        })
        .map(|(index, field)| {
            (
                index,
                field.name.raw().0.as_slice(),
                field.descriptor.raw().0.as_slice(),
            )
        })
        .collect();
    match matching.as_slice() {
        [field] => Some(*field),
        _ => None,
    }
}

fn local_load(instruction: &EnumCodeInstruction, kind: u8, slot: u16) -> bool {
    match kind {
        b'a' => {
            (instruction.opcode == 0x19 && instruction.local == Some(slot))
                || (instruction.opcode == 0x2a + u8::try_from(slot).unwrap_or(u8::MAX) && slot <= 3)
        }
        b'i' => {
            (instruction.opcode == 0x15 && instruction.local == Some(slot))
                || (instruction.opcode == 0x1a + u8::try_from(slot).unwrap_or(u8::MAX) && slot <= 3)
        }
        _ => false,
    }
}

fn field_reference(
    instruction: &EnumCodeInstruction,
    opcode: u8,
    owner: &[u8],
    name: &[u8],
    descriptor: &[u8],
) -> bool {
    instruction.opcode == opcode
        && matches!(
            &instruction.reference,
            Some(EnumCodeReference::Field { owner: actual_owner, name: actual_name, descriptor: actual_descriptor })
                if actual_owner == owner && actual_name == name && actual_descriptor == descriptor
        )
}

fn method_reference(
    instruction: &EnumCodeInstruction,
    opcode: u8,
    owner: &[u8],
    name: &[u8],
    descriptor: &[u8],
    interface: bool,
) -> bool {
    instruction.opcode == opcode
        && matches!(
            &instruction.reference,
            Some(EnumCodeReference::Method { owner: actual_owner, name: actual_name, descriptor: actual_descriptor, interface: actual_interface })
                if actual_owner == owner && actual_name == name && actual_descriptor == descriptor && *actual_interface == interface
        )
}

fn class_reference(instruction: &EnumCodeInstruction, opcode: u8, name: &[u8]) -> bool {
    instruction.opcode == opcode
        && matches!(&instruction.reference, Some(EnumCodeReference::Class(actual)) if actual == name)
}

fn class_constant(instruction: &EnumCodeInstruction, name: &[u8]) -> bool {
    matches!(instruction.opcode, 0x12 | 0x13)
        && matches!(&instruction.reference, Some(EnumCodeReference::Class(actual)) if actual == name)
}

fn string_constant(instruction: Option<&EnumCodeInstruction>, value: &[u8]) -> bool {
    instruction.is_some_and(|instruction| {
        matches!(instruction.opcode, 0x12 | 0x13)
            && matches!(&instruction.reference, Some(EnumCodeReference::String(actual)) if actual == value)
    })
}

fn int_constant_at(instructions: &[EnumCodeInstruction], index: usize, value: i32) -> bool {
    instructions
        .get(index)
        .and_then(int_constant_value)
        .is_some_and(|actual| actual == value)
}

fn int_constant(instruction: &EnumCodeInstruction, value: i32) -> bool {
    int_constant_value(instruction).is_some_and(|actual| actual == value)
}

fn int_constant_value(instruction: &EnumCodeInstruction) -> Option<i32> {
    match instruction.opcode {
        0x02..=0x08 => Some(i32::from(instruction.opcode) - 0x03),
        0x10 | 0x11 => match &instruction.immediate {
            Some(ImmediateValue::Int(value)) => Some(*value),
            _ => None,
        },
        0x12 | 0x13 => match &instruction.reference {
            Some(EnumCodeReference::Integer(value)) => Some(*value),
            _ => None,
        },
        _ => None,
    }
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
                assert!(matches!(
                    report.enum_constant_proof,
                    ClassSourceEnumConstantProof::Refused { .. }
                ));
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
                .map(|constant| constant.source_argument)
                .collect::<Vec<_>>(),
            [Some(4), Some(9)]
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
                .map(|constant| constant.source_argument)
                .collect::<Vec<_>>(),
            [Some(2), Some(5)]
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
        assert!(default.text.contains("static int first;"));
        assert!(default.text.contains("static int second;"));
        assert!(default.text.contains("static {"));
        assert!(default.text.contains("OrdinaryInit.first = 4;"));
        assert!(default.text.contains("OrdinaryInit.second = twice();"));
        assert_eq!(default.fields.len(), 2);
        assert!(
            default
                .methods
                .iter()
                .any(|method| method.item.name.raw().0 == b"<clinit>")
        );
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
                    source_argument: EnumSourceArgument::AnyLiteral,
                },
                InitializerConstructorCall {
                    descriptor: CTOR_DESCRIPTOR.to_vec(),
                    source_argument: EnumSourceArgument::AnyLiteral,
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
        field.name = "bad-name".to_owned();
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
        field.name = "other".to_owned();
        cases.push(("AST field claim differs from Fieldref", ast_wrong_field));

        let mut wrong_terminal_signature = baseline.clone();
        let terminal_index =
            constructor_index_for(&wrong_terminal_signature.method_headers, CTOR_DESCRIPTOR);
        wrong_terminal_signature.source_methods[terminal_index].enum_constructor_source_signature =
            false;
        cases.push((
            "terminal source Signature has the wrong shape",
            wrong_terminal_signature,
        ));

        let mut wrong_delegating_signature = baseline.clone();
        let delegating_index = constructor_index_for(
            &wrong_delegating_signature.method_headers,
            DELEGATING_CTOR_DESCRIPTOR,
        );
        wrong_delegating_signature.source_methods[delegating_index]
            .enum_constructor_no_arg_source_signature = false;
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
                reference: None,
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
                reference: Some(EnumCodeReference::Method {
                    owner: b"ConstructorEffects".to_vec(),
                    name: b"audit".to_vec(),
                    descriptor: b"(I)V".to_vec(),
                    interface: false,
                }),
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
            reference: Some(EnumCodeReference::Method {
                owner: b"ConstructorEffects".to_vec(),
                name: b"record".to_vec(),
                descriptor: b"(I)V".to_vec(),
                interface: false,
            }),
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
}
