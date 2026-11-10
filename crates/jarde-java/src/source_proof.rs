//! Java source applicability over same-run JVM facts.
//!
//! The facade supplies selected headers and real resolver results; this module never opens an
//! artifact or constructs a resolver request. Budget and failure values stay on the supplying request.
//! Enum candidate records are fact adapters; public construction does not grant artifact binding.
#![allow(clippy::too_many_arguments)]
use crate as jarde_java;
use crate::StopReason;
use crate::ast::{Expr, ExprKind, Type as JavaType};
use crate::facts::{ACC_ABSTRACT, ACC_PRIVATE, ACC_PUBLIC, ACC_STATIC, ACC_SYNTHETIC};
use crate::names::source_class_name;
use jarde_jvm::ir::MethodAnalysisRequest;
use jarde_jvm::method_ir::MethodIr;
use jarde_jvm::resolver::{ReferenceUse, ResolutionState};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{
    AttributeShell, ClassFacts, ClassMemberFacts, CpEntryKind, DescriptorKind, ImmediateValue,
    InstructionFact, InstructionOperands, MemberHeader, cp_entry, descriptor_facts,
};
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{ExecutionReport, JvmBytes, PhysicalDefinitionId, PhysicalMethodId};
use jarde_reader::view::{DelegationPolicy, LoaderId, ModuleMode, RuntimeUncertainty};
const ACC_INTERFACE: u16 = 0x0200;

/// Raw platform owner shared by enum source proofs over existing classfile facts.
pub const ENUM_SUPER: &[u8] = b"java/lang/Enum";
const ENUM_VALUE_OF_DESCRIPTOR: &[u8] = b"(Ljava/lang/Class;Ljava/lang/String;)Ljava/lang/Enum;";
const ACC_ENUM: u16 = 0x4000;

/// A resolved constant-pool value needed by the fixed enum bytecode patterns.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnumCodeReference {
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
pub struct EnumCodeInstruction {
    pub bci: u32,
    pub width: u32,
    pub opcode: u8,
    pub immediate: Option<ImmediateValue>,
    pub local: Option<u16>,
    pub increment: Option<i32>,
    pub interface_count: Option<u8>,
    pub reference: Option<EnumCodeReference>,
    /// Target derived from this instruction's Code operand; accepted proofs match it to a BCI.
    pub branch_target_bci: Option<u32>,
}

/// A direct or bootstrap-mediated symbolic member reference used by one physical method.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumCodeUse {
    pub bci: u32,
    pub reference: EnumCodeReference,
}

/// The candidate body of one physically indexed method, carried from its one class-source run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumMethodCodeCandidate {
    pub table_index: u64,
    pub member: Option<PhysicalMethodId>,
    pub complete: bool,
    pub exception_handler_count: u32,
    pub instructions: Vec<EnumCodeInstruction>,
    pub member_uses: Vec<EnumCodeUse>,
}

pub fn may_capture_group_code(class: &ClassFacts, member_table_complete: bool) -> bool {
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

    true
}

/// Copy a candidate body's bounded structural facts from the same method run.
///
/// Candidate names only select which full bodies need to be retained. Every enum method still
/// contributes its same-run symbolic uses when the prepared class passes the cheap Java 8,
/// two-constant eligibility gate, so the class-level proof can reject references to members it will
/// project away.
pub fn capture_method_code(
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
        let branch_target_bci = operands
            .and_then(|operands| operands.branch_offset)
            .and_then(|offset| i64::from(instruction.bci).checked_add(i64::from(offset)))
            .and_then(|target| u32::try_from(target).ok());
        let structured =
            enum_instruction(instruction, operands, ir.constant_pool(), branch_target_bci);
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

/// Fact adapter only; does not establish method membership or trusted artifact provenance.
#[doc(hidden)]
pub fn enum_instruction(
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
    pool: &[jarde_reader::classfile::CpEntryFacts],
    branch_target_bci: Option<u32>,
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
        increment: operands.and_then(|operands| operands.increment),
        interface_count: operands.and_then(|operands| operands.interface_count),
        reference,
        branch_target_bci,
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

pub fn object_descriptor(owner: &[u8]) -> Vec<u8> {
    let mut descriptor = Vec::with_capacity(owner.len() + 2);
    descriptor.push(b'L');
    descriptor.extend_from_slice(owner);
    descriptor.push(b';');
    descriptor
}

pub fn array_descriptor(owner: &[u8]) -> Vec<u8> {
    let mut descriptor = Vec::with_capacity(owner.len() + 3);
    descriptor.push(b'[');
    descriptor.extend_from_slice(&object_descriptor(owner));
    descriptor
}

pub fn prove_values(code: &EnumMethodCodeCandidate, owner: &[u8], backing_name: &[u8]) -> bool {
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

pub fn prove_value_of(code: &EnumMethodCodeCandidate, owner: &[u8]) -> bool {
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

pub fn local_load(instruction: &EnumCodeInstruction, kind: u8, slot: u16) -> bool {
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

pub fn field_reference(
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

pub fn method_reference(
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

pub fn class_reference(instruction: &EnumCodeInstruction, opcode: u8, name: &[u8]) -> bool {
    instruction.opcode == opcode
        && matches!(&instruction.reference, Some(EnumCodeReference::Class(actual)) if actual == name)
}

pub fn class_constant(instruction: &EnumCodeInstruction, name: &[u8]) -> bool {
    matches!(instruction.opcode, 0x12 | 0x13)
        && matches!(&instruction.reference, Some(EnumCodeReference::Class(actual)) if actual == name)
}

/// The most user arguments one arbitrary enum constructor tail may carry. The bound keeps the
/// per-argument proof budget bounded; wider tails stay per-field until evidence asks for them.
const MAX_ENUM_USER_ARGUMENTS: usize = 3;
/// The shared `(Ljava/lang/String;I` prefix every enum constructor descriptor starts with.
const ENUM_CTOR_DESCRIPTOR_PREFIX: &[u8] = b"(Ljava/lang/String;I";
/// A source argument tail proved from the enum initializer's same-run Code facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvedEnumSourceArgument {
    Int(ProvedEnumIntArgument),
    String(ProvedEnumStringArgument),
    StringVarargs(Vec<ProvedEnumStringLiteral>),
    /// The full user argument tail of one arbitrary constructor call, in parameter order.
    User(Vec<ProvedEnumUserArgument>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvedEnumStringArgument {
    Literal(ProvedEnumStringLiteral),
    Ternary {
        condition: ProvedEnumIntCondition,
        when_true: ProvedEnumStringLiteral,
        when_false: ProvedEnumStringLiteral,
        branch_bci: u32,
        join_bci: u32,
    },
}

impl ProvedEnumStringArgument {
    fn source_text(&self) -> String {
        let literal = |value: &ProvedEnumStringLiteral| {
            format!("\"{}\"", jarde_java::escape_string(&value.value))
        };
        match self {
            Self::Literal(value) => literal(value),
            Self::Ternary {
                condition,
                when_true,
                when_false,
                ..
            } => format!(
                "{}() ? {} : {}",
                condition.source_name,
                literal(when_true),
                literal(when_false)
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedEnumStringLiteral {
    pub value: String,
    pub bci: u32,
}

/// One classified user parameter of an arbitrary enum constructor tail: the shapes this slice
/// proves past the four fixed descriptors. `J`/`F`/`D` literals and array parameters stay outside
/// it and keep the per-field presentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnumUserParameter {
    /// A primitive int-family parameter: the constant step expects one int literal, and the
    /// presentation narrows it by this parameter type.
    IntLiteral(EnumIntUserParameter),
    /// A `Ljava/lang/String;` parameter: the constant step expects one String literal.
    StringLiteral,
    /// Any other `L…;` object parameter: the constant step expects one static field reference or
    /// `aconst_null`.
    Object { descriptor: Vec<u8> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnumIntUserParameter {
    Byte,
    Char,
    Short,
    Int,
    Boolean,
}

impl EnumUserParameter {
    /// The physical parameter descriptor this parameter contributes to the constructor.
    pub fn descriptor(&self) -> &[u8] {
        match self {
            Self::IntLiteral(EnumIntUserParameter::Byte) => b"B",
            Self::IntLiteral(EnumIntUserParameter::Char) => b"C",
            Self::IntLiteral(EnumIntUserParameter::Short) => b"S",
            Self::IntLiteral(EnumIntUserParameter::Int) => b"I",
            Self::IntLiteral(EnumIntUserParameter::Boolean) => b"Z",
            Self::StringLiteral => b"Ljava/lang/String;",
            Self::Object { descriptor } => descriptor,
        }
    }

    /// Parse one parameter descriptor from the front of a constructor descriptor tail.
    pub fn parse(descriptor: &[u8]) -> Option<(Self, &[u8])> {
        match descriptor.first()? {
            b'B' => Some((
                Self::IntLiteral(EnumIntUserParameter::Byte),
                &descriptor[1..],
            )),
            b'C' => Some((
                Self::IntLiteral(EnumIntUserParameter::Char),
                &descriptor[1..],
            )),
            b'S' => Some((
                Self::IntLiteral(EnumIntUserParameter::Short),
                &descriptor[1..],
            )),
            b'I' => Some((
                Self::IntLiteral(EnumIntUserParameter::Int),
                &descriptor[1..],
            )),
            b'Z' => Some((
                Self::IntLiteral(EnumIntUserParameter::Boolean),
                &descriptor[1..],
            )),
            b'L' => {
                if let Some(rest) = descriptor.strip_prefix(b"Ljava/lang/String;") {
                    return Some((Self::StringLiteral, rest));
                }
                let end = descriptor.iter().position(|byte| *byte == b';')?;
                let name = &descriptor[1..end];
                if name.is_empty() {
                    return None;
                }
                jarde_java::names::source_class_name(&JvmBytes(name.to_vec()))?;
                Some((
                    Self::Object {
                        descriptor: descriptor[..=end].to_vec(),
                    },
                    &descriptor[end + 1..],
                ))
            }
            // `J`/`F`/`D` literal parameters and array parameters are not this slice's grammar.
            _ => None,
        }
    }

    /// The Java type text the folded constructor declaration spells for this parameter, using
    /// the shared raw class-name boundary for object descriptors.
    pub fn source_type_text(&self) -> Option<String> {
        match self {
            Self::IntLiteral(EnumIntUserParameter::Byte) => Some("byte".to_owned()),
            Self::IntLiteral(EnumIntUserParameter::Char) => Some("char".to_owned()),
            Self::IntLiteral(EnumIntUserParameter::Short) => Some("short".to_owned()),
            Self::IntLiteral(EnumIntUserParameter::Int) => Some("int".to_owned()),
            Self::IntLiteral(EnumIntUserParameter::Boolean) => Some("boolean".to_owned()),
            Self::StringLiteral => Some("java.lang.String".to_owned()),
            Self::Object { descriptor } => {
                let name = descriptor.strip_prefix(b"L")?.strip_suffix(b";")?;
                jarde_java::names::source_class_name(&JvmBytes(name.to_vec()))
            }
        }
    }
}

/// Parse an arbitrary enum constructor descriptor: the shared `(Ljava/lang/String;I` prefix plus
/// one to three user parameters of the admitted shapes, then `)V`. The no-user-argument shape is
/// its own fixed descriptor and is never produced here.
pub fn parse_arbitrary_ctor_descriptor(descriptor: &[u8]) -> Option<Vec<EnumUserParameter>> {
    let tail = descriptor.strip_prefix(ENUM_CTOR_DESCRIPTOR_PREFIX)?;
    let tail = tail.strip_suffix(b")V")?;
    if tail.is_empty() {
        return None;
    }
    let mut parameters = Vec::new();
    let mut rest = tail;
    while !rest.is_empty() {
        if parameters.len() == MAX_ENUM_USER_ARGUMENTS {
            return None;
        }
        let (parameter, remainder) = EnumUserParameter::parse(rest)?;
        parameters.push(parameter);
        rest = remainder;
    }
    Some(parameters)
}

/// What the constant step expects at one user argument position, derived from the parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnumUserArgumentExpectation {
    IntLiteral(EnumIntUserParameter),
    StringLiteral,
    StaticFieldOrNull,
}

impl EnumUserArgumentExpectation {
    pub fn of_parameter(parameter: &EnumUserParameter) -> Self {
        match parameter {
            EnumUserParameter::IntLiteral(kind) => Self::IntLiteral(*kind),
            EnumUserParameter::StringLiteral => Self::StringLiteral,
            EnumUserParameter::Object { .. } => Self::StaticFieldOrNull,
        }
    }
}

/// One proved user argument of an arbitrary enum constructor tail, carried with the presentation
/// its parameter descriptor determines. Static field nodes keep the raw symbolic reference's
/// Java spelling from the repository's descriptor-name path and are never resolved across classes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvedEnumUserArgument {
    /// One int-family literal, narrowed by the parameter descriptor (`(byte) 1`, `'x'`, `true`).
    IntLiteral { text: String, bci: u32 },
    /// One String literal (the parameter is `Ljava/lang/String;`).
    StringLiteral(ProvedEnumStringLiteral),
    /// One static field reference spelled `Owner.name`.
    StaticField {
        source_owner: String,
        source_name: String,
        bci: u32,
    },
    /// `aconst_null` at an object parameter.
    Null { bci: u32 },
}

impl ProvedEnumUserArgument {
    fn source_text(&self) -> String {
        match self {
            Self::IntLiteral { text, .. } => text.clone(),
            Self::StringLiteral(value) => {
                format!("\"{}\"", jarde_java::escape_string(&value.value))
            }
            Self::StaticField {
                source_owner,
                source_name,
                ..
            } => format!("{source_owner}.{source_name}"),
            Self::Null { .. } => "null".to_owned(),
        }
    }
}

/// The joined `", "` spelling of one constant's proved user arguments: the one channel both the
/// ordinary constant list and the constant-body presentation read.
pub fn user_arguments_source_text(arguments: &[ProvedEnumUserArgument]) -> String {
    arguments
        .iter()
        .map(|argument| argument.source_text())
        .collect::<Vec<_>>()
        .join(", ")
}

impl ProvedEnumSourceArgument {
    pub fn source_text(&self) -> String {
        match self {
            Self::Int(argument) => argument.source_text(),
            Self::String(argument) => argument.source_text(),
            Self::StringVarargs(values) => values
                .iter()
                .map(|value| format!("\"{}\"", jarde_java::escape_string(&value.value)))
                .collect::<Vec<_>>()
                .join(", "),
            Self::User(arguments) => user_arguments_source_text(arguments),
        }
    }
}

/// A closed source expression proven from the enum initializer's same-run Code facts.
///
/// Every leaf and operator retains its initializer BCI. Static field nodes keep both the raw
/// symbolic reference and the Java spelling derived from the repository's descriptor-name path;
/// they are never replaced with a value looked up from another class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvedEnumIntArgument {
    Literal {
        value: i32,
        bci: u32,
    },
    StaticField {
        owner: Vec<u8>,
        name: Vec<u8>,
        descriptor: Vec<u8>,
        source_owner: String,
        source_name: String,
        bci: u32,
    },
    Add {
        left: Box<ProvedEnumIntArgument>,
        right: Box<ProvedEnumIntArgument>,
        bci: u32,
    },
    Ternary {
        condition: ProvedEnumIntCondition,
        when_true: Box<ProvedEnumIntArgument>,
        when_false: Box<ProvedEnumIntArgument>,
        branch_bci: u32,
        join_bci: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedEnumIntCondition {
    pub owner: Vec<u8>,
    pub name: Vec<u8>,
    pub descriptor: Vec<u8>,
    pub source_name: String,
    pub bci: u32,
}

impl ProvedEnumIntArgument {
    pub fn source_text(&self) -> String {
        match self {
            Self::Literal { value, .. } => value.to_string(),
            Self::StaticField {
                source_owner,
                source_name,
                ..
            } => format!("{source_owner}.{source_name}"),
            Self::Add { left, right, .. } => {
                format!("{} + {}", left.source_text(), right.source_text())
            }
            Self::Ternary {
                condition,
                when_true,
                when_false,
                ..
            } => format!(
                "{}() ? {} : {}",
                condition.source_name,
                when_true.source_text(),
                when_false.source_text()
            ),
        }
    }
}

pub fn unique_method(
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

pub fn decode_ascii_enum_string(raw: &[u8]) -> Option<String> {
    if raw.is_empty() {
        return Some(String::new());
    }
    if raw.iter().any(|byte| !(1..=0x7f).contains(byte)) {
        return None;
    }
    String::from_utf8(raw.to_vec()).ok()
}

pub fn int_constant_at(instructions: &[EnumCodeInstruction], index: usize, value: i32) -> bool {
    instructions
        .get(index)
        .and_then(int_constant_value)
        .is_some_and(|actual| actual == value)
}

pub fn int_constant(instruction: &EnumCodeInstruction, value: i32) -> bool {
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

/// Prove one closed source int expression from consecutive instructions at the initializer stack
/// argument position. The only nonliteral grammar is one resolved `getstatic:I`, optionally
/// followed by one int literal and `iadd`; operand order is retained exactly as executed.
pub fn prove_int_source_argument(
    instructions: &[EnumCodeInstruction],
    start: usize,
    owner: &[u8],
    methods: &[MemberHeader],
) -> Option<(usize, ProvedEnumIntArgument)> {
    if let Some(ternary) = prove_int_ternary_argument(instructions, start, owner, methods) {
        return Some(ternary);
    }
    let first = instructions.get(start)?;
    if let Some(literal) = proved_int_literal(first) {
        return Some((1, literal));
    }
    let field = proved_int_static_field(first)?;
    let second = instructions.get(start.checked_add(1)?)?;
    if let Some(literal) = proved_int_literal(second) {
        let add = instructions.get(start.checked_add(2)?)?;
        if add.opcode == 0x60
            && add.width == 1
            && add.immediate.is_none()
            && add.local.is_none()
            && add.reference.is_none()
            && instruction_sequence_is_contiguous(instructions.get(start..=start + 2)?)
        {
            return Some((
                3,
                ProvedEnumIntArgument::Add {
                    left: Box::new(field),
                    right: Box::new(literal),
                    bci: add.bci,
                },
            ));
        }
    }
    Some((1, field))
}

/// Prove javac's closed `invokestatic; ifeq/ifne; literal; goto; literal; join` lowering.
/// The condition is one same-class, no-argument boolean invocation, so spelling the call at the
/// original constructor argument position preserves its single evaluation and effects.
fn prove_int_ternary_argument(
    instructions: &[EnumCodeInstruction],
    start: usize,
    owner: &[u8],
    methods: &[MemberHeader],
) -> Option<(usize, ProvedEnumIntArgument)> {
    let [condition, branch, fallthrough_arm, jump, taken_arm, join] =
        instructions.get(start..start.checked_add(6)?)?
    else {
        return None;
    };
    let condition_proof = proved_enum_condition(condition, owner, methods)?;
    if !matches!(branch.opcode, 0x99 | 0x9a)
        || branch.width != 3
        || branch.immediate.is_some()
        || branch.local.is_some()
        || branch.reference.is_some()
        || jump.opcode != 0xa7
        || jump.width != 3
        || jump.immediate.is_some()
        || jump.local.is_some()
        || jump.reference.is_some()
        || !instruction_sequence_is_contiguous(&instructions[start..start + 5])
        || join.bci != jump.branch_target_bci?
    {
        return None;
    }
    let fallthrough_value = proved_int_literal(fallthrough_arm)?;
    let taken_value = proved_int_literal(taken_arm)?;
    let (when_true, when_false, expected_target) = if branch.opcode == 0x99 {
        (fallthrough_value, taken_value, taken_arm.bci)
    } else {
        (taken_value, fallthrough_value, taken_arm.bci)
    };
    if branch.branch_target_bci? != expected_target {
        return None;
    }
    Some((
        5,
        ProvedEnumIntArgument::Ternary {
            condition: condition_proof,
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
            branch_bci: branch.bci,
            join_bci: join.bci,
        },
    ))
}

fn proved_enum_condition(
    instruction: &EnumCodeInstruction,
    owner: &[u8],
    methods: &[MemberHeader],
) -> Option<ProvedEnumIntCondition> {
    let EnumCodeReference::Method {
        owner: condition_owner,
        name,
        descriptor,
        interface: false,
    } = instruction.reference.as_ref()?
    else {
        return None;
    };
    if instruction.opcode != 0xb8
        || instruction.width != 3
        || instruction.immediate.is_some()
        || instruction.local.is_some()
        || condition_owner != owner
        || descriptor != b"()Z"
    {
        return None;
    }
    let source_name = jarde_java::names::source_identifier(name)?;
    let method = methods.get(unique_method(methods, name, descriptor).ok()?)?;
    if method.access_flags & ACC_STATIC == 0 {
        return None;
    }
    Some(ProvedEnumIntCondition {
        owner: condition_owner.clone(),
        name: name.clone(),
        descriptor: descriptor.clone(),
        source_name,
        bci: instruction.bci,
    })
}

fn proved_string_literal(instruction: &EnumCodeInstruction) -> Option<ProvedEnumStringLiteral> {
    if !matches!(instruction.opcode, 0x12 | 0x13)
        || instruction.width != if instruction.opcode == 0x12 { 2 } else { 3 }
        || instruction.immediate.is_some()
        || instruction.local.is_some()
    {
        return None;
    }
    let Some(EnumCodeReference::String(raw)) = &instruction.reference else {
        return None;
    };
    Some(ProvedEnumStringLiteral {
        value: decode_ascii_enum_string(raw)?,
        bci: instruction.bci,
    })
}

pub fn prove_string_source_argument(
    instructions: &[EnumCodeInstruction],
    start: usize,
    owner: &[u8],
    methods: &[MemberHeader],
) -> Option<(usize, ProvedEnumStringArgument)> {
    if let Some(literal) = instructions.get(start).and_then(proved_string_literal) {
        return Some((1, ProvedEnumStringArgument::Literal(literal)));
    }
    let [condition, branch, fallthrough_arm, jump, taken_arm, join] =
        instructions.get(start..start.checked_add(6)?)?
    else {
        return None;
    };
    let condition = proved_enum_condition(condition, owner, methods)?;
    if !matches!(branch.opcode, 0x99 | 0x9a)
        || branch.width != 3
        || branch.immediate.is_some()
        || branch.local.is_some()
        || branch.reference.is_some()
        || jump.opcode != 0xa7
        || jump.width != 3
        || jump.immediate.is_some()
        || jump.local.is_some()
        || jump.reference.is_some()
        || !instruction_sequence_is_contiguous(&instructions[start..start + 5])
        || join.bci != jump.branch_target_bci?
        || branch.branch_target_bci? != taken_arm.bci
    {
        return None;
    }
    let fallthrough = proved_string_literal(fallthrough_arm)?;
    let taken = proved_string_literal(taken_arm)?;
    let (when_true, when_false) = if branch.opcode == 0x99 {
        (fallthrough, taken)
    } else {
        (taken, fallthrough)
    };
    Some((
        5,
        ProvedEnumStringArgument::Ternary {
            condition,
            when_true,
            when_false,
            branch_bci: branch.bci,
            join_bci: join.bci,
        },
    ))
}

pub fn proved_int_literal(instruction: &EnumCodeInstruction) -> Option<ProvedEnumIntArgument> {
    let value = int_constant_value(instruction)?;
    let operands_match = match instruction.opcode {
        0x02..=0x08 => {
            instruction.width == 1
                && matches!(instruction.immediate, Some(ImmediateValue::Int(actual)) if actual == value)
                && instruction.local.is_none()
                && instruction.reference.is_none()
        }
        0x10 => {
            instruction.width == 2
                && matches!(instruction.immediate, Some(ImmediateValue::Int(_)))
                && instruction.local.is_none()
                && instruction.reference.is_none()
        }
        0x11 => {
            instruction.width == 3
                && matches!(instruction.immediate, Some(ImmediateValue::Int(_)))
                && instruction.local.is_none()
                && instruction.reference.is_none()
        }
        0x12 => {
            instruction.width == 2
                && instruction.immediate.is_none()
                && instruction.local.is_none()
                && matches!(instruction.reference, Some(EnumCodeReference::Integer(_)))
        }
        0x13 => {
            instruction.width == 3
                && instruction.immediate.is_none()
                && instruction.local.is_none()
                && matches!(instruction.reference, Some(EnumCodeReference::Integer(_)))
        }
        _ => false,
    };
    operands_match.then_some(ProvedEnumIntArgument::Literal {
        value,
        bci: instruction.bci,
    })
}

fn proved_int_static_field(instruction: &EnumCodeInstruction) -> Option<ProvedEnumIntArgument> {
    if instruction.opcode != 0xb2
        || instruction.width != 3
        || instruction.immediate.is_some()
        || instruction.local.is_some()
    {
        return None;
    }
    let Some(EnumCodeReference::Field {
        owner,
        name,
        descriptor,
    }) = &instruction.reference
    else {
        return None;
    };
    if owner.is_empty() || descriptor != b"I" {
        return None;
    }
    let source_name = jarde_java::names::source_identifier(name)?;
    let source_owner = spelled_static_field_owner(owner)?;
    Some(ProvedEnumIntArgument::StaticField {
        owner: owner.clone(),
        name: name.clone(),
        descriptor: descriptor.clone(),
        source_owner,
        source_name,
        bci: instruction.bci,
    })
}

/// The `Owner` half of one static field argument's `Owner.name` spelling, through the
/// shared raw class-name boundary, keeping the final segment used by field presentations.
fn spelled_static_field_owner(owner: &[u8]) -> Option<String> {
    let full_source_owner = jarde_java::names::source_class_name(&JvmBytes(owner.to_vec()))?;
    Some(full_source_owner.rsplit('.').next()?.to_owned())
}

/// Prove one user argument of an arbitrary enum constructor call at the initializer stack
/// position, expecting exactly the shape its parameter descriptor classifies. Every spelling is
/// decided here; a shape outside the expectation refuses the argument and with it the fold.
pub fn prove_user_source_argument(
    instructions: &[EnumCodeInstruction],
    start: usize,
    expectation: EnumUserArgumentExpectation,
) -> Option<(usize, ProvedEnumUserArgument)> {
    let first = instructions.get(start)?;
    match expectation {
        EnumUserArgumentExpectation::IntLiteral(parameter) => {
            let (value, bci) = proved_int_family_literal(first)?;
            let text = narrowed_int_user_text(parameter, value)?;
            Some((1, ProvedEnumUserArgument::IntLiteral { text, bci }))
        }
        EnumUserArgumentExpectation::StringLiteral => {
            let literal = proved_string_literal(first)?;
            Some((1, ProvedEnumUserArgument::StringLiteral(literal)))
        }
        EnumUserArgumentExpectation::StaticFieldOrNull => {
            if let Some(argument) = proved_object_static_field(first) {
                return Some((1, argument));
            }
            if first.opcode == 0x01
                && first.width == 1
                && first.immediate.is_none()
                && first.local.is_none()
                && first.reference.is_none()
            {
                return Some((1, ProvedEnumUserArgument::Null { bci: first.bci }));
            }
            None
        }
    }
}

/// Prove one int-family literal argument instruction: the iconst/bipush/sipush immediate shapes
/// this slice shares with the plain-int path, plus the `ldc`/`ldc_w` Integer shape the proposal's
/// grammar admits (the shared `proved_int_literal` keeps the plain-int path's own stricter shape,
/// so the fixed descriptor behavior stays byte-identical).
fn proved_int_family_literal(instruction: &EnumCodeInstruction) -> Option<(i32, u32)> {
    let value = int_constant_value(instruction)?;
    let operands_match = match instruction.opcode {
        0x02..=0x08 => {
            instruction.width == 1
                && matches!(instruction.immediate, Some(ImmediateValue::Int(actual)) if actual == value)
                && instruction.local.is_none()
                && instruction.reference.is_none()
        }
        0x10 => {
            instruction.width == 2
                && matches!(instruction.immediate, Some(ImmediateValue::Int(actual)) if actual == value)
                && instruction.local.is_none()
                && instruction.reference.is_none()
        }
        0x11 => {
            instruction.width == 3
                && matches!(instruction.immediate, Some(ImmediateValue::Int(actual)) if actual == value)
                && instruction.local.is_none()
                && instruction.reference.is_none()
        }
        0x12 => {
            instruction.width == 2
                && instruction.local.is_none()
                && matches!(instruction.reference, Some(EnumCodeReference::Integer(actual)) if actual == value)
        }
        0x13 => {
            instruction.width == 3
                && instruction.local.is_none()
                && matches!(instruction.reference, Some(EnumCodeReference::Integer(actual)) if actual == value)
        }
        _ => false,
    };
    operands_match.then_some((value, instruction.bci))
}

/// Prove one `getstatic` object argument. The field reference is spelled `Owner.name` and never
/// resolved across classes: the source spelling is exactly what the initializer executes, so the
/// fold stays faithful whatever the target field holds.
fn proved_object_static_field(instruction: &EnumCodeInstruction) -> Option<ProvedEnumUserArgument> {
    if instruction.opcode != 0xb2
        || instruction.width != 3
        || instruction.immediate.is_some()
        || instruction.local.is_some()
    {
        return None;
    }
    let Some(EnumCodeReference::Field {
        owner,
        name,
        descriptor,
    }) = &instruction.reference
    else {
        return None;
    };
    // A verifier-valid object argument carries a reference-typed value, so the field descriptor
    // must be an object or an array of them.
    if owner.is_empty() || !matches!(descriptor.first(), Some(b'L' | b'[')) {
        return None;
    }
    let source_name = jarde_java::names::source_identifier(name)?;
    let source_owner = spelled_static_field_owner(owner)?;
    Some(ProvedEnumUserArgument::StaticField {
        source_owner,
        source_name,
        bci: instruction.bci,
    })
}

/// The presentation of one int-family literal, narrowed by its parameter descriptor: `B` and `S`
/// carry an explicit cast, `I` spells the bare value, `Z` spells the bare boolean keywords, and
/// `C` spells a char literal for the values a char literal denotes (the compile-and-fold-equivalent
/// `(char) v` cast covers the rest).
#[doc(hidden)]
pub fn narrowed_int_user_text(parameter: EnumIntUserParameter, value: i32) -> Option<String> {
    match parameter {
        EnumIntUserParameter::Byte => Some(format!("(byte) {value}")),
        EnumIntUserParameter::Short => Some(format!("(short) {value}")),
        EnumIntUserParameter::Int => Some(value.to_string()),
        EnumIntUserParameter::Boolean => match value {
            0 => Some("false".to_owned()),
            1 => Some("true".to_owned()),
            _ => None,
        },
        EnumIntUserParameter::Char => match u32::try_from(value) {
            Ok(value) if value <= u32::from(u16::MAX) => Some(char_user_text(value)),
            // Outside the char range no char literal denotes this value; the cast is the exact
            // source form javac constant-folds to it.
            _ => Some(format!("(char) {value}")),
        },
    }
}

fn char_user_text(value: u32) -> String {
    match char::from_u32(value) {
        Some('\'') => "'\\''".to_owned(),
        Some('\\') => "'\\\\'".to_owned(),
        Some(character) if ('\u{20}'..='\u{7e}').contains(&character) => {
            format!("'{character}'")
        }
        // Control and non-ASCII values have no single-line char literal this slice spells.
        _ => format!("(char) {value}"),
    }
}

pub fn instruction_sequence_is_contiguous(instructions: &[EnumCodeInstruction]) -> bool {
    instructions.windows(2).all(|pair| {
        pair[0]
            .bci
            .checked_add(pair[0].width)
            .is_some_and(|next| next == pair[1].bci)
    })
}

// Constructor Code/field policy consumes only supplied same-run facts. The declaration
// predicate is queried after unique field matching at the original refusal point; it must not
// perform recovery or reads. Physical pairing and whole-group commit remain with the supplier.
pub const ENUM_CTOR_DESCRIPTOR: &[u8] = b"(Ljava/lang/String;I)V";

/// The source presentation of one arbitrary enum constructor tail: every user parameter with the
/// Java type text its descriptor spells and the own instance field the proved body stores it
/// into, in argument order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedEnumUserConstructorTail {
    pub parameters: Vec<ProvedEnumUserConstructorParameter>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedEnumUserConstructorParameter {
    /// The Java type text the folded constructor declaration spells for this parameter.
    pub type_text: String,
    /// The own instance field the constructor body stores this parameter into.
    pub field_name: String,
}

pub fn prove_string_constructor(
    code: &EnumMethodCodeCandidate,
    owner: &[u8],
    fields: &[MemberHeader],
    budget: &mut Budget,
) -> Result<std::result::Result<(usize, u32), String>> {
    let instructions = &code.instructions;
    let refused = |reason: &str| Err(reason.to_owned());
    if !code.complete
        || code.exception_handler_count != 0
        || instructions.len() != 8
        || !instructions_are_contiguous(instructions, budget)?
    {
        return Ok(refused(
            "the scalar String constructor Code is incomplete, noncanonical, or has handlers",
        ));
    }
    let local = |instruction: &EnumCodeInstruction, opcode: u8, slot: u16| {
        instruction.opcode == opcode
            && instruction.width == 1
            && instruction.immediate.is_none()
            && instruction.local == Some(slot)
            && instruction.reference.is_none()
    };
    if !local(&instructions[0], 0x2a, 0)
        || !local(&instructions[1], 0x2b, 1)
        || !local(&instructions[2], 0x1c, 2)
        || !method_reference(
            &instructions[3],
            0xb7,
            ENUM_SUPER,
            b"<init>",
            ENUM_CTOR_DESCRIPTOR,
            false,
        )
        || instructions[3].width != 3
        || instructions[3].immediate.is_some()
        || instructions[3].local.is_some()
        || !local(&instructions[4], 0x2a, 0)
        || !local(&instructions[5], 0x2d, 3)
        || instructions[6].opcode != 0xb5
        || instructions[6].width != 3
        || instructions[6].immediate.is_some()
        || instructions[6].local.is_some()
        || instructions[7].opcode != 0xb1
        || instructions[7].width != 1
        || instructions[7].immediate.is_some()
        || instructions[7].local.is_some()
        || instructions[7].reference.is_some()
    {
        return Ok(refused(
            "the scalar String constructor has effects beyond Enum initialization and one field store",
        ));
    }
    let Some(EnumCodeReference::Field {
        owner: field_owner,
        name,
        descriptor,
    }) = &instructions[6].reference
    else {
        return Ok(refused("the scalar String field store lacks its Fieldref"));
    };
    if field_owner != owner || descriptor != b"Ljava/lang/String;" {
        return Ok(refused(
            "the scalar String constructor stores a different field type or owner",
        ));
    }
    let matching: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| {
            field.descriptor.raw().0 == b"Ljava/lang/String;"
                && field.access_flags & (ACC_STATIC | ACC_ENUM) == 0
        })
        .map(|(index, _)| index)
        .collect();
    let [field_index] = matching.as_slice() else {
        return Ok(refused(
            "the enum does not have exactly one String instance field",
        ));
    };
    if fields[*field_index].name.raw().0 != *name {
        return Ok(refused(
            "the scalar String constructor stores a field other than the unique target",
        ));
    }
    Ok(Ok((*field_index, instructions[3].bci)))
}

pub fn prove_string_varargs_constructor(
    code: &EnumMethodCodeCandidate,
    owner: &[u8],
    fields: &[MemberHeader],
    budget: &mut Budget,
) -> Result<std::result::Result<(usize, u32), String>> {
    let instructions = &code.instructions;
    let refused = |reason: &str| Err(reason.to_owned());
    if !code.complete || code.exception_handler_count != 0 {
        return Ok(refused(
            "the String... constructor Code is incomplete or has handlers",
        ));
    }
    if instructions.len() != 8 || !instructions_are_contiguous(instructions, budget)? {
        return Ok(refused(
            "the String... constructor is not one complete eight-instruction body",
        ));
    }
    let canonical_local = |instruction: &EnumCodeInstruction, opcode: u8, slot: u16| {
        instruction.opcode == opcode
            && instruction.width == 1
            && instruction.immediate.is_none()
            && instruction.local == Some(slot)
            && instruction.reference.is_none()
    };
    if !canonical_local(&instructions[0], 0x2a, 0)
        || !canonical_local(&instructions[1], 0x2b, 1)
        || !canonical_local(&instructions[2], 0x1c, 2)
        || !method_reference(
            &instructions[3],
            0xb7,
            ENUM_SUPER,
            b"<init>",
            ENUM_CTOR_DESCRIPTOR,
            false,
        )
        || instructions[3].width != 3
        || !canonical_local(&instructions[4], 0x2a, 0)
        || !canonical_local(&instructions[5], 0x2d, 3)
        || instructions[6].opcode != 0xb5
        || instructions[6].width != 3
        || instructions[6].immediate.is_some()
        || instructions[6].local.is_some()
        || instructions[7].opcode != 0xb1
        || instructions[7].width != 1
        || instructions[7].immediate.is_some()
        || instructions[7].local.is_some()
        || instructions[7].reference.is_some()
    {
        return Ok(refused(
            "the String... constructor has effects beyond Enum initialization and one String[] field store",
        ));
    }
    let Some(EnumCodeReference::Field {
        owner: field_owner,
        name,
        descriptor,
    }) = &instructions[6].reference
    else {
        return Ok(refused(
            "the String... field store has no complete Fieldref",
        ));
    };
    if field_owner != owner || descriptor != b"[Ljava/lang/String;" {
        return Ok(refused(
            "the String... constructor stores a different field type or owner",
        ));
    }
    let matching_fields: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| {
            field.descriptor.raw().0 == b"[Ljava/lang/String;"
                && field.access_flags & (ACC_STATIC | ACC_ENUM) == 0
        })
        .map(|(index, _)| index)
        .collect();
    let [field_index] = matching_fields.as_slice() else {
        return Ok(refused(
            "the enum does not have exactly one String[] instance field",
        ));
    };
    if fields[*field_index].name.raw().0 != *name {
        return Ok(refused(
            "the String... constructor stores a String[] field other than the unique target",
        ));
    }
    Ok(Ok((*field_index, instructions[3].bci)))
}

/// Prove the complete user body of one arbitrary enum constructor from its raw same-run Code:
/// the exact `Enum` super call, then exactly one `this`-slot store per user parameter into its
/// own distinct instance field of the parameter's descriptor, then a bare return. Every other
/// effect refuses the constructor and with it the fold.
pub fn prove_arbitrary_user_constructor(
    code: &EnumMethodCodeCandidate,
    owner: &[u8],
    parameters: &[EnumUserParameter],
    fields: &[MemberHeader],
    field_is_declared: impl Fn(usize) -> bool,
    budget: &mut Budget,
) -> Result<std::result::Result<(ProvedEnumUserConstructorTail, u32), String>> {
    let refused = |reason: &str| Ok(Err(reason.to_owned()));
    let instructions = &code.instructions;
    let expected_len = 4 + parameters.len() * 3 + 1;
    if !code.complete
        || code.exception_handler_count != 0
        || instructions.len() != expected_len
        || !instructions_are_contiguous(instructions, budget)?
    {
        return refused(
            "the arbitrary constructor Code is incomplete, noncanonical, or has handlers",
        );
    }
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(instructions.len())
            .unwrap_or(u64::MAX)
            .saturating_add(u64::try_from(code.member_uses.len()).unwrap_or(u64::MAX)),
    )?;
    let pure_local = |instruction: &EnumCodeInstruction, kind: u8, slot: u16| {
        local_load(instruction, kind, slot)
            && instruction.immediate.is_none()
            && instruction.reference.is_none()
    };
    if !pure_local(&instructions[0], b'a', 0)
        || !pure_local(&instructions[1], b'a', 1)
        || !pure_local(&instructions[2], b'i', 2)
        || !method_reference(
            &instructions[3],
            0xb7,
            ENUM_SUPER,
            b"<init>",
            ENUM_CTOR_DESCRIPTOR,
            false,
        )
        || instructions[3].width != 3
        || instructions[3].immediate.is_some()
        || instructions[3].local.is_some()
    {
        return refused("the arbitrary constructor has effects before the exact Enum super call");
    }
    let mut stored_fields = std::collections::BTreeSet::new();
    let mut tail_parameters = Vec::with_capacity(parameters.len());
    for (index, parameter) in parameters.iter().enumerate() {
        budget.poll()?;
        let base = 4 + index * 3;
        let Ok(slot) = u16::try_from(3 + index) else {
            return refused("the arbitrary constructor parameter slot overflows");
        };
        let load_kind = match parameter {
            EnumUserParameter::IntLiteral(_) => b'i',
            EnumUserParameter::StringLiteral | EnumUserParameter::Object { .. } => b'a',
        };
        if !pure_local(&instructions[base], b'a', 0)
            || !pure_local(&instructions[base + 1], load_kind, slot)
            || instructions[base + 2].opcode != 0xb5
            || instructions[base + 2].width != 3
            || instructions[base + 2].immediate.is_some()
            || instructions[base + 2].local.is_some()
        {
            return refused(
                "the arbitrary constructor body is not one this-slot store per user parameter",
            );
        }
        let Some(EnumCodeReference::Field {
            owner: field_owner,
            name,
            descriptor,
        }) = &instructions[base + 2].reference
        else {
            return refused("the arbitrary constructor's parameter store has no resolved Fieldref");
        };
        if field_owner != owner || descriptor.as_slice() != parameter.descriptor() {
            return refused(
                "the arbitrary constructor stores a parameter into a differently typed field",
            );
        }
        let Some(field_name) = jarde_java::names::source_identifier(name) else {
            return refused("the arbitrary constructor's field target is not valid Java text");
        };
        let matching: Vec<_> = fields
            .iter()
            .enumerate()
            .filter(|(_, field)| {
                field.name.raw().0 == name.as_slice()
                    && field.descriptor.raw().0 == parameter.descriptor()
                    && field.access_flags & (ACC_STATIC | ACC_ENUM) == 0
            })
            .map(|(index, _)| index)
            .collect();
        let [field_index] = matching.as_slice() else {
            return refused(
                "the arbitrary constructor's parameter store target is absent or ambiguous",
            );
        };
        if !stored_fields.insert(*field_index) {
            return refused("two arbitrary constructor parameters store into one field");
        }
        if !field_is_declared(*field_index) {
            return refused(
                "the arbitrary constructor's parameter field is not faithfully declared",
            );
        }
        let Some(type_text) = parameter.source_type_text() else {
            return refused(
                "the arbitrary constructor's parameter type has no readable Java spelling",
            );
        };
        tail_parameters.push(ProvedEnumUserConstructorParameter {
            type_text,
            field_name,
        });
    }
    let last = &instructions[expected_len - 1];
    if last.opcode != 0xb1
        || last.width != 1
        || last.immediate.is_some()
        || last.local.is_some()
        || last.reference.is_some()
    {
        return refused("the arbitrary constructor does not end in a bare return");
    }
    Ok(Ok((
        ProvedEnumUserConstructorTail {
            parameters: tail_parameters,
        },
        instructions[3].bci,
    )))
}

pub fn prove_constructor(
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

pub fn instructions_are_contiguous(
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

// Complete enum constructor and initializer policy over supplied same-run facts.
// Selectors retain physical binding at the supplier and are queried only at their original stage.
const ACC_FINAL: u16 = 0x0010;
const ACC_VARARGS: u16 = 0x0080;
pub const CTOR_DESCRIPTOR: &[u8] = b"(Ljava/lang/String;II)V";
pub const DELEGATING_CTOR_DESCRIPTOR: &[u8] = b"(Ljava/lang/String;I)V";
pub const STRING_VARARGS_CTOR_DESCRIPTOR: &[u8] = b"(Ljava/lang/String;I[Ljava/lang/String;)V";
pub const STRING_CTOR_DESCRIPTOR: &[u8] = b"(Ljava/lang/String;ILjava/lang/String;)V";
const MAX_ENUM_STRING_VARARGS_ELEMENTS: usize = 64;
const VALUES_DESCRIPTOR_PREFIX: &[u8] = b"()[L";
const VALUE_OF_DESCRIPTOR_PREFIX: &[u8] = b"(Ljava/lang/String;)L";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnumConstructorSourceTail {
    Unrecognized,
    NoArg,
    Int,
    SingleString,
    StringVarargs,
}

impl EnumConstructorSourceTail {
    pub fn is_recognized(self) -> bool {
        self != Self::Unrecognized
    }
}

pub fn enum_constructor_source_tail(
    signature_count: usize,
    name: &[u8],
    physical_descriptor: &[u8],
    parsed: &jarde_reader::signature::MethodSignature,
) -> EnumConstructorSourceTail {
    use jarde_reader::signature::SignatureType;
    if signature_count == 1
        && name == b"<init>"
        && physical_descriptor == b"(Ljava/lang/String;II)V"
        && parsed.type_parameters.is_empty()
        && parsed.parameters.as_slice() == [SignatureType::Base(b'I')]
        && parsed.result.is_none()
        && parsed.throws.is_empty()
    {
        EnumConstructorSourceTail::Int
    } else if signature_count == 1
        && name == b"<init>"
        && physical_descriptor == b"(Ljava/lang/String;I)V"
        && parsed.type_parameters.is_empty()
        && parsed.parameters.is_empty()
        && parsed.result.is_none()
        && parsed.throws.is_empty()
    {
        EnumConstructorSourceTail::NoArg
    } else if signature_count == 1
        && name == b"<init>"
        && physical_descriptor == b"(Ljava/lang/String;ILjava/lang/String;)V"
        && parsed.type_parameters.is_empty()
        && matches!(parsed.parameters.as_slice(), [SignatureType::Class(class)]
                if class.segments.len() == 1
                    && class.segments[0].binary_name == b"java/lang/String"
                    && class.segments[0].arguments.is_empty())
        && parsed.result.is_none()
        && parsed.throws.is_empty()
    {
        EnumConstructorSourceTail::SingleString
    } else if signature_count == 1
        && name == b"<init>"
        && physical_descriptor == b"(Ljava/lang/String;I[Ljava/lang/String;)V"
        && parsed.type_parameters.is_empty()
        && matches!(parsed.parameters.as_slice(), [SignatureType::Array(element)]
                if matches!(element.as_ref(), SignatureType::Class(class)
                    if class.segments.len() == 1
                        && class.segments[0].binary_name == b"java/lang/String"
                        && class.segments[0].arguments.is_empty()))
        && parsed.result.is_none()
        && parsed.throws.is_empty()
    {
        EnumConstructorSourceTail::StringVarargs
    } else {
        EnumConstructorSourceTail::Unrecognized
    }
}

pub fn values_descriptor(owner: &[u8]) -> Vec<u8> {
    let mut descriptor = Vec::from(VALUES_DESCRIPTOR_PREFIX);
    descriptor.extend_from_slice(owner);
    descriptor.push(b';');
    descriptor
}

pub fn value_of_descriptor(owner: &[u8]) -> Vec<u8> {
    let mut descriptor = Vec::from(VALUE_OF_DESCRIPTOR_PREFIX);
    descriptor.extend_from_slice(owner);
    descriptor.push(b';');
    descriptor
}

pub fn exact_method_flags(method: &MemberHeader, flags: u16) -> bool {
    method.access_flags == flags
}

pub fn allowed_method_attributes(method: &MemberHeader, allowed: &[&[u8]]) -> bool {
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

pub fn single_code_attribute(method: &MemberHeader) -> bool {
    method
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Code")
        .count()
        == 1
}

/// Terminal effects proven from the same-run Code and constructor AST, kept private for 2.3.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedEnumConstructorBody {
    pub method_index: usize,
    pub field_index: usize,
    pub enum_super_call_bci: u32,
    pub helper_call_bci: u32,
    pub field_write_bci: u32,
    pub return_bci: u32,
    pub helper_owner: Vec<u8>,
    pub candidate: jarde_java::report::ClassEnumConstructorCandidates,
}

/// The constructor edge proved from the exact class-source member candidates. This is deliberately
/// not a class-level `Proved` result: the terminal constructor's user effects and Signature are the
/// next proof step, and the current two-constructor path remains refused until that work is done.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedEnumConstructorDelegationEdge {
    pub delegating_method_index: usize,
    pub terminal_method_index: usize,
    pub constant_constructor_bcis: [u32; 2],
    pub constant_constructor_descriptors: [Vec<u8>; 2],
    pub constant_source_arguments: [Option<i32>; 2],
    pub initializer_prefix: InitializerPrefixProof,
}

pub fn prove_implicit_enum_constructor(
    code: &EnumMethodCodeCandidate,
    header: &MemberHeader,
    member: &PhysicalMethodId,
    candidates: &[jarde_java::report::ClassEnumConstructorCandidates],
    budget: &mut Budget,
) -> Result<bool> {
    use jarde_java::report::ClassEnumConstructorStepKind as Step;
    if header.name.raw().0 != b"<init>"
        || header.descriptor.raw().0 != DELEGATING_CTOR_DESCRIPTOR
        || !exact_method_flags(header, ACC_PRIVATE)
        || !single_code_attribute(header)
        || !allowed_method_attributes(header, &[b"Code", b"MethodParameters", b"Signature"])
        || !code.complete
        || code.exception_handler_count != 0
        || code.instructions.len() != 5
        || !instructions_are_contiguous(&code.instructions, budget)?
        || !local_load(&code.instructions[0], b'a', 0)
        || !local_load(&code.instructions[1], b'a', 1)
        || !local_load(&code.instructions[2], b'i', 2)
        || !method_reference(
            &code.instructions[3],
            0xb7,
            ENUM_SUPER,
            b"<init>",
            ENUM_CTOR_DESCRIPTOR,
            false,
        )
        || code.instructions[4].opcode != 0xb1
        || code.member_uses.len() != 1
        || !matches!(
            &code.member_uses[0].reference,
            EnumCodeReference::Method { owner, name, descriptor, interface: false }
                if owner == ENUM_SUPER && name == b"<init>" && descriptor == DELEGATING_CTOR_DESCRIPTOR
        )
    {
        return Ok(false);
    }
    let matching: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.member.as_ref() == Some(member))
        .collect();
    let [candidate] = matching.as_slice() else {
        return Ok(false);
    };
    if !candidate.complete
        || candidate.has_exception_handlers
        || !matches!(candidate.steps.len(), 1 | 2)
    {
        return Ok(false);
    }
    let step = &candidate.steps[0];
    if step.order != 0 || step.bci != code.instructions[3].bci {
        return Ok(false);
    }
    if candidate.steps.len() == 2
        && !matches!(&candidate.steps[1].kind, Step::Return { value: None })
    {
        return Ok(false);
    }
    if candidate.steps.len() == 2
        && (candidate.steps[1].order != 1 || candidate.steps[1].bci != code.instructions[4].bci)
    {
        return Ok(false);
    }
    let Step::ConstructorCall {
        target: jarde_java::ast::ConstructorTarget::Super,
        args,
    } = &step.kind
    else {
        return Ok(false);
    };
    let valid = args.len() == 2
        && matches!(args[0].kind, jarde_java::ast::ExprKind::Local(_))
        && matches!(args[1].kind, jarde_java::ast::ExprKind::Local(_))
        && args[0].origin.primary().bci() == code.instructions[1].bci
        && args[1].origin.primary().bci() == code.instructions[2].bci;
    Ok(valid)
}

pub fn enum_constructor_signature_matches(
    header: &MemberHeader,
    source_tail: EnumConstructorSourceTail,
    erasure_refused: bool,
    marker_count: usize,
    physical_descriptor: &[u8],
) -> bool {
    let source_shape = match physical_descriptor {
        DELEGATING_CTOR_DESCRIPTOR => source_tail == EnumConstructorSourceTail::NoArg,
        CTOR_DESCRIPTOR => source_tail == EnumConstructorSourceTail::Int,
        STRING_VARARGS_CTOR_DESCRIPTOR => source_tail == EnumConstructorSourceTail::StringVarargs,
        STRING_CTOR_DESCRIPTOR => source_tail == EnumConstructorSourceTail::SingleString,
        _ => false,
    };
    source_shape
        && erasure_refused
        && marker_count == 1
        && header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"Signature")
            .count()
            == 1
        && allowed_method_attributes(header, &[b"Code", b"MethodParameters", b"Signature"])
        && (physical_descriptor != STRING_VARARGS_CTOR_DESCRIPTOR
            || header.access_flags == (ACC_PRIVATE | ACC_VARARGS))
}

/// Source-only qualification after the supplier has matched the terminal physical member.
pub fn terminal_enum_constructor_source_applicable(
    header: &MemberHeader,
    source_tail: EnumConstructorSourceTail,
    erasure_refused: bool,
    marker_count: usize,
    recovered: Option<(&jarde_java::report::RecoveryReport, &ExecutionReport)>,
) -> bool {
    exact_method_flags(header, ACC_PRIVATE)
        && single_code_attribute(header)
        && allowed_method_attributes(header, &[b"Code", b"MethodParameters", b"Signature"])
        && header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"Signature")
            .count()
            == 1
        && enum_constructor_signature_matches(
            header,
            source_tail,
            erasure_refused,
            marker_count,
            CTOR_DESCRIPTOR,
        )
        && recovered.is_some_and(|(report, execution)| {
            report.quality == jarde_jvm::ir::Quality::Structured
                && report.content == jarde_java::report::RecoveryContent::ContainsStatements
                && matches!(execution, ExecutionReport::Complete { .. })
        })
}

pub fn prove_constructor_delegation_edge<'c>(
    owner: &[u8],
    constants: &[(usize, String, i32)],
    backing_name: &[u8],
    field_headers: &[MemberHeader],
    method_headers: &[MemberHeader],
    code_candidate_count: usize,
    selected_code: impl Fn(usize) -> std::result::Result<&'c EnumMethodCodeCandidate, String>,
    budget: &mut Budget,
) -> Result<std::result::Result<ProvedEnumConstructorDelegationEdge, String>> {
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
            u64::try_from(code_candidate_count)
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

    let delegating = match selected_code(delegating_index) {
        Ok(candidate) => candidate,
        Err(reason) => return Ok(Err(reason)),
    };
    let terminal = match selected_code(terminal_index) {
        Ok(candidate) => candidate,
        Err(reason) => return Ok(Err(reason)),
    };
    let initializer = match selected_code(clinit_index) {
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
            || jarde_java::names::source_identifier(&field.name.raw().0).as_deref()
                != Some(name.as_str())
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
        .map(|(field_index, _, ordinal)| {
            (field_headers[*field_index].name.raw().0.clone(), *ordinal)
        })
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

pub fn prove_terminal_constructor_body(
    owner: &[u8],
    method_index: usize,
    field_headers: &[MemberHeader],
    code: &EnumMethodCodeCandidate,
    constructor_candidates: &[jarde_java::report::ClassEnumConstructorCandidates],
    member: &PhysicalMethodId,
    source_field_facts: impl Fn(usize, &[u8]) -> Option<(bool, bool, bool, bool, bool)>,
    budget: &mut Budget,
) -> Result<std::result::Result<ProvedEnumConstructorBody, String>> {
    use jarde_java::ast::{AssignOp, ConstructorTarget, ExprKind};
    use jarde_java::report::ClassEnumConstructorStepKind as StepKind;
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    if !code.complete
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
    let Some(helper_name_text) = jarde_java::names::source_identifier(helper_name) else {
        return refuse("the terminal helper name is not valid Java text");
    };
    if helper_owner.is_empty() || helper_owner == owner || helper_descriptor != b"(I)V" {
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
    let Some(field_name_text) = jarde_java::names::source_identifier(field_name) else {
        return refuse("the terminal field name is not valid Java text");
    };
    if field_owner != owner || field_descriptor != b"I" {
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
    let Some((physical_match, declared, markers_empty, annotations_clear, type_annotations_clear)) =
        source_field_facts(*field_index, field_name)
    else {
        return refuse("the terminal field has no class-source record");
    };
    if !physical_match
        || !declared
        || !markers_empty
        || !annotations_clear
        || !type_annotations_clear
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
        .filter(|candidate| candidate.member.as_ref() == Some(member))
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
    let Some(helper_display) =
        jarde_java::names::source_class_name(&JvmBytes(helper_owner.clone()))
    else {
        return refuse("the terminal helper owner cannot be spelled as a Java path");
    };
    let ExprKind::Call {
        receiver: Some(receiver),
        name,
        args,
    } = &helper_expression.kind
    else {
        return refuse("the terminal AST helper call is not a qualified static call");
    };
    if !matches!(&receiver.kind, ExprKind::Path(path) if path == &helper_display)
        || name != &helper_name_text
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
        || ast_field.owner.0.as_slice() != owner
        || ast_field.name.0.as_slice() != *field_name
        || ast_field.descriptor.0.as_slice() != b"I"
        || ast_field.is_static
        || !jarde_java::is_java_identifier(spelled_name)
        || &field_name_text != spelled_name
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

fn is_branch_opcode(opcode: u8) -> bool {
    matches!(opcode, 0x99..=0xa9 | 0xaa..=0xab | 0xc6..=0xc9)
}

pub fn prove_values_factory(
    code: &EnumMethodCodeCandidate,
    owner: &[u8],
    constants: &[(usize, String, i32)],
    fields: &[MemberHeader],
) -> bool {
    let instructions = &code.instructions;
    let expected_len = 3usize.saturating_add(constants.len().saturating_mul(4));
    if instructions.len() != expected_len {
        return false;
    }
    if !int_constant(
        &instructions[0],
        i32::try_from(constants.len()).unwrap_or(i32::MAX),
    ) || !class_reference(&instructions[1], 0xbd, owner)
    {
        return false;
    }
    for (ordinal, (field_index, _, _)) in constants.iter().enumerate() {
        let Ok(ordinal_i32) = i32::try_from(ordinal) else {
            return false;
        };
        let Some(start) = ordinal
            .checked_mul(4)
            .and_then(|offset| offset.checked_add(2))
        else {
            return false;
        };
        if instructions[start].opcode != 0x59
            || !int_constant(&instructions[start + 1], ordinal_i32)
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
    instructions[expected_len - 1].opcode == 0xb0
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnumSourceArgument {
    AnyIntExpression,
    Exact(i32),
    SingleString,
    StringVarargs,
    /// One expectation per user parameter of an arbitrary constructor call, in parameter order.
    UserArguments(Vec<EnumUserArgumentExpectation>),
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializerConstructorCall {
    pub descriptor: Vec<u8>,
    pub source_argument: EnumSourceArgument,
}

pub struct InitializerPrefixInput<'a> {
    pub code: &'a EnumMethodCodeCandidate,
    pub owner: &'a [u8],
    pub constants: &'a [(Vec<u8>, i32)],
    pub constructor_calls: &'a [InitializerConstructorCall],
    pub enum_descriptor: &'a [u8],
    pub fields: &'a [MemberHeader],
    pub methods: &'a [MemberHeader],
    pub backing_name: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializerPrefixProof {
    pub constant_bcis: Vec<u32>,
    pub backing_store_bci: u32,
    pub constructor_bcis: Vec<u32>,
    pub source_arguments: Vec<Option<ProvedEnumSourceArgument>>,
    pub prefix_end_bci: u32,
    pub factory_call_bci: u32,
}

pub fn prove_initializer_prefix(
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
    if constructor_calls.len() != constants.len() {
        return Err("the constructor proof does not match the ordered constant vector".to_owned());
    }
    for call in constructor_calls {
        unique_method(methods, b"<init>", &call.descriptor)?;
    }
    let instructions = &code.instructions;
    let mut cursor = 0_usize;
    let mut constant_bcis = Vec::with_capacity(constants.len());
    let mut constructor_bcis = Vec::with_capacity(constants.len());
    let mut source_arguments = Vec::with_capacity(constants.len());
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
            EnumSourceArgument::AnyIntExpression => {
                let Some((argument_length, argument)) =
                    prove_int_source_argument(instructions, cursor + 4, owner, methods)
                else {
                    return Err(format!(
                        "constant {} has no supported int source argument expression: {:?}",
                        ordinal,
                        instructions.get(cursor + 4)
                    ));
                };
                (
                    4 + argument_length,
                    Some(ProvedEnumSourceArgument::Int(argument)),
                )
            }
            EnumSourceArgument::Exact(expected) => {
                let Some(argument) = instructions.get(cursor + 4).and_then(proved_int_literal)
                else {
                    return Err(format!(
                        "constant {} does not pass an int literal source argument",
                        ordinal
                    ));
                };
                if !matches!(&argument, ProvedEnumIntArgument::Literal { value, .. } if value == expected)
                {
                    return Err(format!(
                        "constant {} does not pass its exact literal source argument",
                        ordinal
                    ));
                }
                (5, Some(ProvedEnumSourceArgument::Int(argument)))
            }
            EnumSourceArgument::SingleString => {
                let Some((length, argument)) =
                    prove_string_source_argument(instructions, cursor + 4, owner, methods)
                else {
                    return Err(format!(
                        "constant {ordinal} has no closed scalar String argument"
                    ));
                };
                (4 + length, Some(ProvedEnumSourceArgument::String(argument)))
            }
            EnumSourceArgument::StringVarargs => {
                let Some(array_length) = instructions.get(cursor + 4).and_then(proved_int_literal)
                else {
                    return Err(format!("constant {ordinal} has no literal String[] length"));
                };
                let ProvedEnumIntArgument::Literal { value: count, .. } = array_length else {
                    return Err(format!(
                        "constant {ordinal} String[] length is not a literal"
                    ));
                };
                let Ok(count) = usize::try_from(count) else {
                    return Err(format!("constant {ordinal} String[] length is negative"));
                };
                if count > MAX_ENUM_STRING_VARARGS_ELEMENTS {
                    return Err(format!(
                        "constant {ordinal} exceeds the bounded String... element count"
                    ));
                }
                let Some(array) = instructions.get(cursor + 5) else {
                    return Err(format!("constant {ordinal} has no String[] allocation"));
                };
                if !class_reference(array, 0xbd, b"java/lang/String")
                    || array.width != 3
                    || array.immediate.is_some()
                    || array.local.is_some()
                {
                    return Err(format!(
                        "constant {ordinal} does not allocate a fresh String[]"
                    ));
                }
                let mut values = Vec::with_capacity(count);
                let mut offset = 6_usize;
                for element_index in 0..count {
                    let start = cursor + offset;
                    let Some([duplicate, index, literal, store]) = instructions
                        .get(start..start + 4)
                        .and_then(|items| <&[EnumCodeInstruction; 4]>::try_from(items).ok())
                    else {
                        return Err(format!(
                            "constant {ordinal} has an incomplete String[] element write"
                        ));
                    };
                    if duplicate.opcode != 0x59
                        || duplicate.width != 1
                        || duplicate.immediate.is_some()
                        || duplicate.local.is_some()
                        || duplicate.reference.is_some()
                        || !matches!(proved_int_literal(index), Some(ProvedEnumIntArgument::Literal { value, .. }) if value == element_index as i32)
                        || !matches!(literal.opcode, 0x12 | 0x13)
                        || literal.width != if literal.opcode == 0x12 { 2 } else { 3 }
                        || literal.immediate.is_some()
                        || literal.local.is_some()
                        || !matches!(&literal.reference, Some(EnumCodeReference::String(_)))
                        || store.opcode != 0x53
                        || store.width != 1
                        || store.immediate.is_some()
                        || store.local.is_some()
                        || store.reference.is_some()
                    {
                        return Err(format!(
                            "constant {ordinal} String[] element {element_index} is not an ordered literal write"
                        ));
                    }
                    let Some(EnumCodeReference::String(raw)) = &literal.reference else {
                        unreachable!();
                    };
                    let value = decode_ascii_enum_string(raw.as_slice())
                        .ok_or_else(|| format!("constant {ordinal} String[] element {element_index} is outside the ASCII literal slice"))?;
                    values.push(ProvedEnumStringLiteral {
                        value,
                        bci: literal.bci,
                    });
                    offset += 4;
                }
                (
                    offset,
                    Some(ProvedEnumSourceArgument::StringVarargs(values)),
                )
            }
            EnumSourceArgument::UserArguments(expectations) => {
                let mut offset = 4_usize;
                let mut arguments = Vec::with_capacity(expectations.len());
                for (argument_index, expectation) in expectations.iter().enumerate() {
                    let start = cursor + offset;
                    let Some((length, argument)) =
                        prove_user_source_argument(instructions, start, *expectation)
                    else {
                        return Err(format!(
                            "constant {ordinal} argument {argument_index} does not match its constructor parameter"
                        ));
                    };
                    arguments.push(argument);
                    offset += length;
                }
                (offset, Some(ProvedEnumSourceArgument::User(arguments)))
            }
            EnumSourceArgument::None => (4, None),
        };
        let Some(constructor_call) = instructions.get(cursor + call_offset) else {
            return Err(format!("constant {ordinal} has no constructor call"));
        };
        let Some(field_store) = instructions.get(cursor + call_offset + 1) else {
            return Err(format!("constant {ordinal} has no field store"));
        };
        if !instruction_sequence_is_contiguous(
            instructions
                .get(cursor..=cursor + call_offset + 1)
                .unwrap_or(&[]),
        ) {
            return Err(format!(
                "constant {ordinal} initializer instructions are not contiguous through the constructor call and field write"
            ));
        }
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
    if !instruction_sequence_is_contiguous(instructions.get(..=cursor).unwrap_or(&[])) {
        return Err(
            "the enum constant and `$VALUES` prefix is not one contiguous instruction sequence"
                .to_owned(),
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

fn string_constant(instruction: Option<&EnumCodeInstruction>, value: &[u8]) -> bool {
    instruction.is_some_and(|instruction| {
        matches!(instruction.opcode, 0x12 | 0x13)
            && matches!(&instruction.reference, Some(EnumCodeReference::String(actual)) if actual == value)
    })
}

/// The exact members that a successful class-level proof may later project.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedEnumConstant {
    pub field_index: u64,
    pub name: String,
    pub ordinal: i32,
    pub source_argument: Option<ProvedEnumSourceArgument>,
    pub constructor_bci: u32,
    pub field_write_bci: u32,
}

/// A private handoff from task 2.1 to the class-source projection stages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedOrdinaryEnumConstantGroup {
    pub constants: Vec<ProvedEnumConstant>,
    pub backing_field_index: u64,
    pub constructor_method_index: u64,
    pub delegating_constructor_method_index: Option<u64>,
    pub constructor_body: Option<Box<ProvedEnumConstructorBody>>,
    pub constructor_field_index: Option<u64>,
    /// The proved presentation tail of one arbitrary single constructor, present exactly when the
    /// physical descriptor came through `parse_arbitrary_ctor_descriptor` instead of one of the
    /// four fixed shapes.
    pub constructor_user_tail: Option<ProvedEnumUserConstructorTail>,
    pub constructor_signature_present: bool,
    pub initializer_method_index: u64,
    pub values_method_index: u64,
    pub value_of_method_index: u64,
    pub values_factory_method_index: u64,
    /// The first BCI after the complete constants and `$VALUES` setup prefix.
    pub initializer_prefix_end_bci: u32,
    /// Number of same-run structured initializer statements in the prefix.
    pub initializer_prefix_statement_count: usize,
}

/// Full ordinary-enum source policy. Suppliers only join existing same-run physical facts.
/// None preserves NotApplicable; the inner error is a local refusal and the outer error a stop.
pub fn prove_ordinary_enum_group<'c, 'm>(
    class_version: (u16, u16),
    class_flags: u16,
    owner: &[u8],
    super_class: Option<&[u8]>,
    field_headers: &[MemberHeader],
    method_headers: &[MemberHeader],
    initializer_candidates: &[jarde_java::report::ClassInitializerCandidates],
    constructor_candidates: &[jarde_java::report::ClassEnumConstructorCandidates],
    validate_tables: impl Fn(&mut Budget) -> Result<std::result::Result<(), String>>,
    selected_code: impl Fn(usize) -> std::result::Result<&'c EnumMethodCodeCandidate, String>,
    field_declared: impl Fn(usize) -> bool,
    source_signature: impl Fn(usize) -> (EnumConstructorSourceTail, bool, usize),
    member_identity: impl Fn(usize) -> &'m PhysicalMethodId,
    constructor_edge: impl Fn(
        &[(usize, String, i32)],
        &[u8],
        &mut Budget,
    ) -> Result<
        std::result::Result<ProvedEnumConstructorDelegationEdge, String>,
    >,
    terminal_body: impl Fn(
        usize,
        &mut Budget,
    ) -> Result<std::result::Result<ProvedEnumConstructorBody, String>>,
    budget: &mut Budget,
) -> Result<std::result::Result<Option<ProvedOrdinaryEnumConstantGroup>, String>> {
    if class_flags & ACC_ENUM == 0 {
        return Ok(Ok(None));
    }
    let refuse = |reason: &str| Err(reason.to_owned());
    if class_version != (52, 0)
        || class_flags & (ACC_INTERFACE | ACC_ABSTRACT) != 0
        || super_class != Some(ENUM_SUPER)
    {
        return Ok(refuse(
            "the class is not a concrete Java 8 enum extending java/lang/Enum",
        ));
    }
    if owner.is_empty() {
        return Ok(refuse(
            "the class-source declaration and physical member owner disagree",
        ));
    }
    if let Err(reason) = validate_tables(budget)? {
        return Ok(Err(reason));
    }

    let enum_fields: Vec<_> = field_headers
        .iter()
        .enumerate()
        .filter(|(_, field)| field.access_flags & ACC_ENUM != 0)
        .collect();
    let enum_descriptor = object_descriptor(owner);
    let mut constants = Vec::with_capacity(enum_fields.len());
    let mut constant_names = std::collections::BTreeSet::new();
    for (ordinal_index, (index, header)) in enum_fields.iter().copied().enumerate() {
        let Ok(ordinal) = i32::try_from(ordinal_index) else {
            return Ok(refuse(
                "the enum constant ordinal exceeds the supported integer range",
            ));
        };
        let expected_flags = ACC_PUBLIC | ACC_STATIC | ACC_FINAL | ACC_ENUM;
        if header.access_flags != expected_flags
            || header.descriptor.raw().0 != enum_descriptor
            || !header.attributes.is_empty()
        {
            return Ok(refuse(
                "an enum constant field has non-standard flags, descriptor, or attributes",
            ));
        }
        let Some(name) = jarde_java::names::source_identifier(&header.name.raw().0) else {
            return Ok(refuse("an enum constant field name is not valid Java text"));
        };
        if !constant_names.insert(name.clone()) {
            return Ok(refuse(
                "enum constant field names are not unique Java identifiers",
            ));
        }
        if !field_declared(index) {
            return Ok(refuse(
                "an enum constant field has no faithful physical declaration",
            ));
        }
        constants.push((index, name, ordinal));
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
    let (
        constructor_index,
        delegating_constructor_index,
        delegation_edge,
        constructor_body,
        user_constructor_parameters,
    ) = match constructor_count {
        1 => {
            let descriptor = method_headers
                .iter()
                .find(|method| method.name.raw().0 == b"<init>")
                .map(|method| method.descriptor.raw().0.as_slice())
                .unwrap_or(&[]);
            let user_parameters = if descriptor == CTOR_DESCRIPTOR
                || descriptor == DELEGATING_CTOR_DESCRIPTOR
                || descriptor == STRING_CTOR_DESCRIPTOR
                || descriptor == STRING_VARARGS_CTOR_DESCRIPTOR
            {
                None
            } else {
                match parse_arbitrary_ctor_descriptor(descriptor) {
                    Some(parameters) => Some(parameters),
                    None => {
                        return Ok(refuse("the enum has an unsupported constructor descriptor"));
                    }
                }
            };
            let constructor_index = match unique_method(method_headers, b"<init>", descriptor) {
                Ok(index) => index,
                Err(reason) => return Ok(refuse(&reason)),
            };
            (constructor_index, None, None, None, user_parameters)
        }
        2 => {
            let edge = match constructor_edge(
                &constants,
                backing_field.name.raw().0.as_slice(),
                budget,
            )? {
                Ok(edge) => edge,
                Err(reason) => {
                    return Ok(refuse(&format!(
                        "enum constructor delegation edge refused: {reason}"
                    )));
                }
            };
            let body = match terminal_body(edge.terminal_method_index, budget)? {
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
                None,
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

    let clinit = match selected_code(clinit_index) {
        Ok(code) => code,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let constructor = match selected_code(constructor_index) {
        Ok(code) => code,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let values = match selected_code(values_index) {
        Ok(code) => code,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let value_of = match selected_code(value_of_index) {
        Ok(code) => code,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let factory = match selected_code(factory_index) {
        Ok(code) => code,
        Err(reason) => return Ok(refuse(&reason)),
    };

    let constructor_flags_valid =
        if method_headers[constructor_index].descriptor.raw().0 == STRING_VARARGS_CTOR_DESCRIPTOR {
            exact_method_flags(
                &method_headers[constructor_index],
                ACC_PRIVATE | ACC_VARARGS,
            )
        } else {
            exact_method_flags(&method_headers[constructor_index], ACC_PRIVATE)
        };
    if !exact_method_flags(&method_headers[clinit_index], ACC_STATIC)
        || !constructor_flags_valid
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
    if matches!(
        method_headers[constructor_index]
            .descriptor
            .raw()
            .0
            .as_slice(),
        STRING_CTOR_DESCRIPTOR | STRING_VARARGS_CTOR_DESCRIPTOR
    ) && !{
        let (tail, erasure_refused, marker_count) = source_signature(constructor_index);
        enum_constructor_signature_matches(
            &method_headers[constructor_index],
            tail,
            erasure_refused,
            marker_count,
            &method_headers[constructor_index].descriptor.raw().0,
        )
    } {
        return Ok(refuse(
            "the String constructor lacks the exact source Signature and access evidence",
        ));
    }
    let projected_constructor_descriptors: Vec<Vec<u8>> = if delegating_constructor_index.is_some()
    {
        vec![
            DELEGATING_CTOR_DESCRIPTOR.to_vec(),
            CTOR_DESCRIPTOR.to_vec(),
        ]
    } else if method_headers[constructor_index].descriptor.raw().0 == DELEGATING_CTOR_DESCRIPTOR {
        vec![DELEGATING_CTOR_DESCRIPTOR.to_vec()]
    } else if method_headers[constructor_index].descriptor.raw().0 == STRING_VARARGS_CTOR_DESCRIPTOR
    {
        vec![STRING_VARARGS_CTOR_DESCRIPTOR.to_vec()]
    } else if method_headers[constructor_index].descriptor.raw().0 == STRING_CTOR_DESCRIPTOR {
        vec![STRING_CTOR_DESCRIPTOR.to_vec()]
    } else if user_constructor_parameters.is_some() {
        // The use census must keep treating this physical descriptor as a projected constructor.
        vec![
            method_headers[constructor_index]
                .descriptor
                .raw()
                .0
                .to_vec(),
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
        let candidate = match selected_code(index) {
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

    let mut constructor_user_tail = None;
    let expected_ctor_field = if let Some(body) = &constructor_body {
        Some(body.field_index)
    } else if let Some(parameters) = &user_constructor_parameters {
        match prove_arbitrary_user_constructor(
            constructor,
            owner,
            parameters,
            field_headers,
            |index| field_declared(index),
            budget,
        )? {
            Ok((tail, _super_call_bci)) => {
                constructor_user_tail = Some(tail);
                None
            }
            Err(reason) => return Ok(refuse(&reason)),
        }
    } else if method_headers[constructor_index].descriptor.raw().0 == DELEGATING_CTOR_DESCRIPTOR {
        if !prove_implicit_enum_constructor(
            constructor,
            &method_headers[constructor_index],
            member_identity(constructor_index),
            constructor_candidates,
            budget,
        )? {
            return Ok(refuse(
                "the implicit enum constructor has extra effects or an incomplete same-run proof",
            ));
        }
        None
    } else if method_headers[constructor_index].descriptor.raw().0 == STRING_VARARGS_CTOR_DESCRIPTOR
    {
        match prove_string_varargs_constructor(constructor, owner, field_headers, budget)? {
            Ok((field_index, _)) => Some(field_index),
            Err(reason) => return Ok(refuse(&reason)),
        }
    } else if method_headers[constructor_index].descriptor.raw().0 == STRING_CTOR_DESCRIPTOR {
        match prove_string_constructor(constructor, owner, field_headers, budget)? {
            Ok((field_index, _)) => Some(field_index),
            Err(reason) => return Ok(refuse(&reason)),
        }
    } else {
        match prove_constructor(constructor, owner, field_headers) {
            Ok((field_index, _)) => Some(field_index),
            Err(reason) => return Ok(refuse(&reason)),
        }
    };
    if expected_ctor_field.is_some_and(|index| !field_declared(index)) {
        return Ok(refuse(
            "the constructor's source field is not faithfully declared",
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
            "$values() does not build the proved constants in field-table order",
        ));
    }

    let constant_specs: Vec<_> = constants
        .iter()
        .map(|(field_index, _, ordinal)| {
            (field_headers[*field_index].name.raw().0.clone(), *ordinal)
        })
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
    } else if method_headers[constructor_index].descriptor.raw().0 == DELEGATING_CTOR_DESCRIPTOR {
        vec![
            InitializerConstructorCall {
                descriptor: DELEGATING_CTOR_DESCRIPTOR.to_vec(),
                source_argument: EnumSourceArgument::None,
            };
            constants.len()
        ]
    } else if method_headers[constructor_index].descriptor.raw().0 == STRING_VARARGS_CTOR_DESCRIPTOR
    {
        vec![
            InitializerConstructorCall {
                descriptor: STRING_VARARGS_CTOR_DESCRIPTOR.to_vec(),
                source_argument: EnumSourceArgument::StringVarargs,
            };
            constants.len()
        ]
    } else if method_headers[constructor_index].descriptor.raw().0 == STRING_CTOR_DESCRIPTOR {
        vec![
            InitializerConstructorCall {
                descriptor: STRING_CTOR_DESCRIPTOR.to_vec(),
                source_argument: EnumSourceArgument::SingleString,
            };
            constants.len()
        ]
    } else if let Some(parameters) = &user_constructor_parameters {
        vec![
            InitializerConstructorCall {
                descriptor: method_headers[constructor_index]
                    .descriptor
                    .raw()
                    .0
                    .to_vec(),
                source_argument: EnumSourceArgument::UserArguments(
                    parameters
                        .iter()
                        .map(EnumUserArgumentExpectation::of_parameter)
                        .collect(),
                ),
            };
            constants.len()
        ]
    } else {
        vec![
            InitializerConstructorCall {
                descriptor: CTOR_DESCRIPTOR.to_vec(),
                source_argument: EnumSourceArgument::AnyIntExpression,
            };
            constants.len()
        ]
    };
    let prefix = if let Some(edge) = &delegation_edge {
        edge.initializer_prefix.clone()
    } else {
        if matches!(
            method_headers[constructor_index]
                .descriptor
                .raw()
                .0
                .as_slice(),
            STRING_CTOR_DESCRIPTOR | STRING_VARARGS_CTOR_DESCRIPTOR
        ) || user_constructor_parameters.is_some()
        {
            // This slice inspects the complete raw initializer stream and copies string literals.
            // Charge it here without changing the pre-existing int/no-arg proof budget behavior.
            let mut literal_bytes = 0_u64;
            for instruction in &clinit.instructions {
                budget.charge(CountedBudgetDimension::IrItems, 3)?;
                budget.poll()?;
                if let Some(EnumCodeReference::String(value)) = &instruction.reference {
                    literal_bytes = literal_bytes
                        .saturating_add(u64::try_from(value.len()).unwrap_or(u64::MAX));
                }
            }
            budget.charge(CountedBudgetDimension::AnalysisSteps, literal_bytes)?;
        }
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
    let selected_clinit_identity = member_identity(clinit_index);
    let matching_initializer_candidates: Vec<_> = initializer_candidates
        .iter()
        .filter(|candidate| candidate.member.as_ref() == Some(selected_clinit_identity))
        .collect();
    let [initializer_candidate] = matching_initializer_candidates.as_slice() else {
        return Ok(refuse(
            "the same-run `<clinit>` structured statement sidecar is absent or ambiguous",
        ));
    };
    if initializer_candidate.has_exception_handlers {
        return Ok(refuse(
            "the `<clinit>` initializer sidecar includes an exception table",
        ));
    }
    let expected_write_bcis: Vec<u32> = prefix
        .constant_bcis
        .iter()
        .copied()
        .chain(std::iter::once(prefix.backing_store_bci))
        .collect();
    // `prove_initializer_prefix` above consumed every raw Code instruction, operand/reference,
    // BCI, constructor call and field write in order. Do not make the AST sidecar a second owner
    // of this prefix: its enum-new fallback intentionally cannot represent getstatic interleaved
    // between javac's `new`/`dup` and constructor call. Keep sidecar identity and handler checks
    // as provenance and reject any unproved Code suffix separately.

    let source_arguments = prefix.source_arguments;
    let proved_constants = constants
        .into_iter()
        .zip(source_arguments)
        .zip(prefix.constructor_bcis)
        .zip(prefix.constant_bcis)
        .map(
            |(
                (((field_index, name, ordinal), source_argument), constructor_bci),
                field_write_bci,
            )| {
                ProvedEnumConstant {
                    field_index: u64::try_from(field_index).unwrap_or(u64::MAX),
                    name,
                    ordinal,
                    source_argument,
                    constructor_bci,
                    field_write_bci,
                }
            },
        )
        .collect();
    Ok(Ok(Some(ProvedOrdinaryEnumConstantGroup {
        constants: proved_constants,
        backing_field_index: u64::try_from(*backing_field_index).unwrap_or(u64::MAX),
        constructor_method_index: u64::try_from(constructor_index).unwrap_or(u64::MAX),
        delegating_constructor_method_index: delegating_constructor_index
            .map(|index| u64::try_from(index).unwrap_or(u64::MAX)),
        constructor_body,
        constructor_field_index: expected_ctor_field
            .map(|index| u64::try_from(index).unwrap_or(u64::MAX)),
        constructor_user_tail,
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
    })))
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

const ACC_NATIVE: u16 = 0x0100;

pub fn has_only_terminal_initializer_return<'m>(
    group: &ProvedOrdinaryEnumConstantGroup,
    code_candidates: &[EnumMethodCodeCandidate],
    initializer_candidates: &[jarde_java::report::ClassInitializerCandidates],
    constructor_source_tail: impl Fn(usize) -> Option<EnumConstructorSourceTail>,
    initializer_member: impl Fn(usize) -> Option<&'m PhysicalMethodId>,
) -> bool {
    if group.initializer_prefix_statement_count != group.constants.len().saturating_add(1) {
        return false;
    }
    let Ok(constructor_index) = usize::try_from(group.constructor_method_index) else {
        return false;
    };
    let Some(constructor_tail) = constructor_source_tail(constructor_index) else {
        return false;
    };
    if group.constructor_user_tail.is_none()
        && group.constructor_signature_present
        && constructor_tail == EnumConstructorSourceTail::Unrecognized
    {
        return false;
    }
    let Ok(initializer_index) = usize::try_from(group.initializer_method_index) else {
        return false;
    };
    let Some(member) = initializer_member(initializer_index) else {
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
    if code.member.as_ref() != Some(member)
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
        .filter(|candidate| candidate.member.as_ref() == Some(member))
        .collect();
    let [initializer] = matching_initializers.as_slice() else {
        return false;
    };
    // The raw complete Code/BCI certificate owns the enum compiler-prefix shape. The structured
    // sidecar can fail to classify javac's interleaved new/dup/getstatic sequence, so it only
    // contributes the same-run identity and handler status here; the raw suffix above proves the
    // exact terminal return boundary.
    !initializer.has_exception_handlers
}

/// One exact user assignment that may follow the proved compiler-owned enum prefix.
///
/// The value and target are taken from the same prepared `<clinit>` run. The class-source adapter
/// only asks the existing expression emitter to spell the RHS after this complete suffix has been
/// checked against Code, field headers, and the initializer statement sidecar.
#[derive(Clone, Debug)]
pub struct ProvedEnumStaticAssignment {
    pub field_index: u64,
    pub field_name: String,
    pub value: jarde_java::ast::Expr,
    pub initializer_member: PhysicalMethodId,
}

pub fn prove_static_assignment_suffix<'f, 'm>(
    group: &ProvedOrdinaryEnumConstantGroup,
    owner: &[u8],
    field_headers: &[MemberHeader],
    method_headers: &[MemberHeader],
    source_field_count: usize,
    source_method_count: usize,
    code_candidates: &[EnumMethodCodeCandidate],
    initializer_candidates: &[jarde_java::report::ClassInitializerCandidates],
    source_field_facts: impl Fn(
        usize,
        &MemberHeader,
    ) -> Option<(&'f PhysicalDefinitionId, bool, bool, bool)>,
    source_method_facts: impl Fn(
        usize,
    ) -> Option<(
        u64,
        &'m PhysicalMethodId,
        bool,
        Option<(&'m jarde_java::report::RecoveryReport, &'m ExecutionReport)>,
    )>,
    budget: &mut Budget,
) -> Result<Option<ProvedEnumStaticAssignment>> {
    use jarde_java::ast::{AssignOp, ExprKind, Type};
    use jarde_java::report::{ClassInitializerStatementKind, ClassInitializerStep};
    if owner.is_empty()
        || group.initializer_prefix_statement_count != group.constants.len().saturating_add(1)
    {
        return Ok(None);
    }

    // Charge the supplied tables before selecting the actual call and field from Code.
    let table_work = field_headers
        .len()
        .saturating_add(source_field_count)
        .saturating_add(method_headers.len())
        .saturating_add(source_method_count)
        .saturating_add(code_candidates.len())
        .saturating_add(initializer_candidates.len());
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(table_work).unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    if field_headers.len() != source_field_count || method_headers.len() != source_method_count {
        return Ok(None);
    }

    let Ok(initializer_index) = usize::try_from(group.initializer_method_index) else {
        return Ok(None);
    };
    let (Some(initializer_header), Some((initializer_method_index, initializer_member, _, _))) = (
        method_headers.get(initializer_index),
        source_method_facts(initializer_index),
    ) else {
        return Ok(None);
    };
    if initializer_header.name.raw().0 != b"<clinit>"
        || initializer_header.descriptor.raw().0 != b"()V"
        || initializer_header.access_flags != ACC_STATIC
        || initializer_method_index != group.initializer_method_index
        || initializer_member.name.0 != b"<clinit>"
        || initializer_member.descriptor.0 != b"()V"
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
    if code.member.as_ref() != Some(initializer_member)
        || !code.complete
        || code.exception_handler_count != 0
    {
        return Ok(None);
    }
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
    let [call, store, ret] = suffix.as_slice() else {
        return Ok(None);
    };
    let (
        Some(
            call_reference @ EnumCodeReference::Method {
                owner: call_owner,
                name: call_name,
                descriptor: call_descriptor,
                interface: false,
            },
        ),
        Some(
            field_reference @ EnumCodeReference::Field {
                owner: field_owner,
                name: field_name,
                descriptor: field_descriptor,
            },
        ),
    ) = (&call.reference, &store.reference)
    else {
        return Ok(None);
    };
    // The supported operation is one own static ()I call stored to an own int field.
    // Its names and positions come from this run, not from a particular enum fixture.
    if call_owner != owner
        || field_owner != owner
        || call_descriptor != b"()I"
        || field_descriptor != b"I"
        || call.bci != group.initializer_prefix_end_bci
        || call.width != 3
        || call.opcode != 0xb8
        || call.immediate.is_some()
        || call.local.is_some()
        || call.bci.checked_add(call.width) != Some(store.bci)
        || store.width != 3
        || store.opcode != 0xb3
        || store.immediate.is_some()
        || store.local.is_some()
        || store.bci.checked_add(store.width) != Some(ret.bci)
        || ret.width != 1
        || ret.opcode != 0xb1
        || ret.reference.is_some()
        || ret.immediate.is_some()
        || ret.local.is_some()
        || code
            .instructions
            .iter()
            .rev()
            .find(|instruction| instruction.bci < call.bci)
            .is_none_or(|instruction| {
                instruction.bci.checked_add(instruction.width) != Some(call.bci)
            })
    {
        return Ok(None);
    }
    let mut suffix_uses = Vec::new();
    for use_fact in &code.member_uses {
        budget.poll()?;
        if use_fact.bci >= call.bci {
            suffix_uses.push(use_fact);
        }
    }
    if !matches!(suffix_uses.as_slice(), [call_use, store_use]
        if call_use.bci == call.bci && &call_use.reference == call_reference
            && store_use.bci == store.bci && &store_use.reference == field_reference)
    {
        return Ok(None);
    }

    let mut matching_field = None;
    for (index, header) in field_headers.iter().enumerate() {
        budget.poll()?;
        if header.name.raw().0 == *field_name
            && header.descriptor.raw().0 == *field_descriptor
            && matching_field.replace((index, header)).is_some()
        {
            return Ok(None);
        }
    }
    let Some((field_index, field_header)) = matching_field else {
        return Ok(None);
    };
    let Some((source_field_owner, matches, declared, markers_empty)) =
        source_field_facts(field_index, field_header)
    else {
        return Ok(None);
    };
    let Some(field_name_text) = jarde_java::names::source_identifier(field_name) else {
        return Ok(None);
    };
    if field_header.access_flags != ACC_STATIC
        || field_header
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"ConstantValue")
        || source_field_owner != &initializer_member.owner
        || !matches
        || !declared
        || !markers_empty
    {
        return Ok(None);
    }

    let mut matching_method = None;
    for (index, header) in method_headers.iter().enumerate() {
        budget.poll()?;
        if header.name.raw().0 == *call_name
            && header.descriptor.raw().0 == *call_descriptor
            && matching_method.replace((index, header)).is_some()
        {
            return Ok(None);
        }
    }
    let Some((method_index, method_header)) = matching_method else {
        return Ok(None);
    };
    let Some((source_method_index, method_member, method_declared, _)) =
        source_method_facts(method_index)
    else {
        return Ok(None);
    };
    let Some(method_name_text) = jarde_java::names::source_identifier(call_name) else {
        return Ok(None);
    };
    if method_header.access_flags & ACC_STATIC == 0
        || method_header.access_flags & (ACC_ABSTRACT | ACC_NATIVE) != 0
        || !method_header
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code")
        || source_method_index != u64::try_from(method_index).unwrap_or(u64::MAX)
        || method_member.owner != initializer_member.owner
        || method_member.name.0 != *call_name
        || method_member.descriptor.0 != *call_descriptor
        || !method_declared
    {
        return Ok(None);
    }

    let mut matching_initializer = None;
    for candidate in initializer_candidates {
        budget.poll()?;
        if candidate.member.as_ref() == Some(initializer_member)
            && matching_initializer.replace(candidate).is_some()
        {
            return Ok(None);
        }
    }
    let Some(initializer) = matching_initializer else {
        return Ok(None);
    };
    let prefix_count = group.initializer_prefix_statement_count;
    if initializer.has_exception_handlers
        || initializer.steps.len() != prefix_count.saturating_add(2)
    {
        return Ok(None);
    }
    for (order, step) in initializer.steps[..prefix_count].iter().enumerate() {
        budget.poll()?;
        if !matches!(step, ClassInitializerStep::FieldWrite(write) if write.order == order) {
            return Ok(None);
        }
    }
    let [
        ClassInitializerStep::FieldWrite(write),
        ClassInitializerStep::Other {
            order: return_order,
            bci: return_bci,
            kind: ClassInitializerStatementKind::Return,
        },
    ] = &initializer.steps[prefix_count..]
    else {
        return Ok(None);
    };
    if write.order != prefix_count
        || *return_order != prefix_count.saturating_add(1)
        || *return_bci != ret.bci
        || write.bci != store.bci
        || write.owner.0 != owner
        || write.name.0 != *field_name
        || write.spelled_name != field_name_text
        || write.descriptor.0 != *field_descriptor
        || !write.is_static
        || write.op != AssignOp::Assign
        || write.source.primary().bci() != store.bci
        || write.value.origin.primary().bci() != call.bci
        // A static putstatic receiver in the sidecar is only a type path: the complete
        // three-instruction suffix above leaves no runtime receiver evaluation to discard.
        || write.source.primary().method().is_some_and(|method| method != initializer_member)
        || write.value.origin.primary().method().is_some_and(|method| method != initializer_member)
        || !write.source.derived().iter().all(|origin| {
            (call.bci..=ret.bci).contains(&origin.bci())
                && origin.method() == Some(initializer_member)
        })
        || !write.value.origin.derived().iter().all(|origin| {
            (call.bci..=ret.bci).contains(&origin.bci())
                && origin.method() == Some(initializer_member)
        })
        || !write.field_reads.as_deref().is_some_and(|reads| reads.is_empty())
        || write.value.presented != Some(Type::Int)
        || !matches!(&write.value.kind, ExprKind::Call { receiver: None, name, args }
            if name == &method_name_text && args.is_empty())
    {
        return Ok(None);
    }

    Ok(Some(ProvedEnumStaticAssignment {
        field_index: u64::try_from(field_index).unwrap_or(u64::MAX),
        field_name: field_name_text,
        value: write.value.clone(),
        initializer_member: initializer_member.clone(),
    }))
}

/// The single javac array loop admitted after a complete ordinary enum prefix.
/// The statements are selected from the same `<clinit>` recovery, never parsed from fallback text.
pub struct ProvedEnumMapSuffix {
    pub field_index: u64,
    pub initializer_member: PhysicalMethodId,
    pub statements: Vec<jarde_java::ast::Stmt>,
}

pub fn prove_map_initializer_suffix<'f, 'm>(
    group: &ProvedOrdinaryEnumConstantGroup,
    owner: &[u8],
    field_headers: &[MemberHeader],
    method_headers: &[MemberHeader],
    source_field_count: usize,
    source_method_count: usize,
    code_candidates: &[EnumMethodCodeCandidate],
    initializer_candidates: &[jarde_java::report::ClassInitializerCandidates],
    source_field_facts: impl Fn(
        usize,
    ) -> Option<(
        u64,
        &'f PhysicalDefinitionId,
        Option<&'f str>,
        &'f [String],
        bool,
        bool,
    )>,
    source_method_facts: impl Fn(
        usize,
    ) -> Option<(
        u64,
        &'m PhysicalMethodId,
        bool,
        Option<(&'m jarde_java::report::RecoveryReport, &'m ExecutionReport)>,
    )>,
    budget: &mut Budget,
) -> Result<Option<ProvedEnumMapSuffix>> {
    use jarde_java::ast::{AssignOp, BinaryOp, Expr, ExprKind, StmtKind, Type};
    use jarde_java::report::{ClassInitializerStatementKind as Kind, ClassInitializerStep as Step};
    if group.constants.len() != 2
        || group.initializer_prefix_statement_count != 3
        || field_headers.len() != 4
        || field_headers.len() != source_field_count
        || method_headers.len() != source_method_count
    {
        return Ok(None);
    }
    let work = field_headers
        .len()
        .saturating_add(method_headers.len())
        .saturating_add(code_candidates.len())
        .saturating_add(initializer_candidates.len());
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(work).unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    let Some(initializer_index) = usize::try_from(group.initializer_method_index).ok() else {
        return Ok(None);
    };
    let (Some(header), Some((method_index, member, _, recovered))) = (
        method_headers.get(initializer_index),
        source_method_facts(initializer_index),
    ) else {
        return Ok(None);
    };
    if header.name.raw().0 != b"<clinit>"
        || header.descriptor.raw().0 != b"()V"
        || header.access_flags != ACC_STATIC
        || method_index != group.initializer_method_index
    {
        return Ok(None);
    }
    let Some((report, analysis_execution)) = recovered else {
        return Ok(None);
    };
    if !report.produced()
        || !matches!(report.execution, ExecutionReport::Complete { .. })
        || !matches!(analysis_execution, ExecutionReport::Complete { .. })
        || report.quality != jarde_jvm::ir::Quality::Structured
        || !report.fallbacks.is_empty()
    {
        return Ok(None);
    }
    let codes: Vec<_> = code_candidates
        .iter()
        .filter(|candidate| candidate.table_index == group.initializer_method_index)
        .collect();
    let [code] = codes.as_slice() else {
        return Ok(None);
    };
    if code.member.as_ref() != Some(member) || !code.complete || code.exception_handler_count != 0 {
        return Ok(None);
    }
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(
            code.instructions
                .len()
                .saturating_add(code.member_uses.len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    let suffix: Vec<_> = code
        .instructions
        .iter()
        .filter(|instruction| instruction.bci >= group.initializer_prefix_end_bci)
        .collect();
    let Ok(suffix): std::result::Result<&[&EnumCodeInstruction; 27], _> =
        suffix.as_slice().try_into()
    else {
        return Ok(None);
    };
    if suffix[0].bci != group.initializer_prefix_end_bci
        || !suffix
            .windows(2)
            .all(|pair| pair[0].bci.checked_add(pair[0].width) == Some(pair[1].bci))
    {
        return Ok(None);
    }
    let Some(EnumCodeReference::Field {
        owner: field_owner,
        name: field_name,
        descriptor,
    }) = &suffix[3].reference
    else {
        return Ok(None);
    };
    if field_owner != owner
        || descriptor != b"Ljava/util/Map;"
        || suffix[3].opcode != 0xb3
        || suffix[18].reference != suffix[3].reference
    {
        return Ok(None);
    }
    let matching_fields: Vec<_> = field_headers
        .iter()
        .enumerate()
        .filter(|(_, field)| {
            field.name.raw().0 == *field_name && field.descriptor.raw().0 == *descriptor
        })
        .collect();
    let [(field_index, field)] = matching_fields.as_slice() else {
        return Ok(None);
    };
    let Some((
        source_field_index,
        source_field_owner,
        source_field_declaration,
        source_field_markers,
        annotations_clear,
        type_annotations_clear,
    )) = source_field_facts(*field_index)
    else {
        return Ok(None);
    };
    let Some(field_name_text) = jarde_java::names::source_identifier(field_name) else {
        return Ok(None);
    };
    if field.access_flags != (ACC_PUBLIC | ACC_STATIC | ACC_FINAL)
        || field
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"ConstantValue")
        || source_field_index != u64::try_from(*field_index).unwrap_or(u64::MAX)
        || source_field_owner != &member.owner
        || source_field_declaration
            .and_then(|declaration| declaration.strip_prefix("public static final java.util.Map "))
            != Some(field_name_text.as_str())
        || source_field_markers
            .iter()
            .any(|marker| !marker.contains("field_generic_source_unproved"))
        || !annotations_clear
        || !type_annotations_clear
    {
        return Ok(None);
    }
    let expected_references = [
        (
            2,
            EnumCodeReference::Method {
                owner: b"java/util/HashMap".to_vec(),
                name: b"<init>".to_vec(),
                descriptor: b"()V".to_vec(),
                interface: false,
            },
        ),
        (3, suffix[3].reference.clone().unwrap()),
        (
            4,
            EnumCodeReference::Method {
                owner: owner.to_vec(),
                name: b"values".to_vec(),
                descriptor: values_descriptor(owner),
                interface: false,
            },
        ),
        (18, suffix[3].reference.clone().unwrap()),
        (
            20,
            EnumCodeReference::Method {
                owner: owner.to_vec(),
                name: b"name".to_vec(),
                descriptor: b"()Ljava/lang/String;".to_vec(),
                interface: false,
            },
        ),
        (
            22,
            EnumCodeReference::Method {
                owner: b"java/util/Map".to_vec(),
                name: b"put".to_vec(),
                descriptor: b"(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;".to_vec(),
                interface: true,
            },
        ),
    ];
    let hash_map_reference = EnumCodeReference::Class(b"java/util/HashMap".to_vec());
    let opcodes = [
        0xbb, 0x59, 0xb7, 0xb3, 0xb8, 0x4b, 0x2a, 0xbe, 0x3c, 0x03, 0x3d, 0x1c, 0x1b, 0xa2, 0x2a,
        0x1c, 0x32, 0x4e, 0xb2, 0x2d, 0xb6, 0x2d, 0xb9, 0x57, 0x84, 0xa7, 0xb1,
    ];
    for (index, instruction) in suffix.iter().enumerate() {
        budget.poll()?;
        if instruction.opcode != opcodes[index]
            || instruction.reference.as_ref()
                != (if index == 0 {
                    Some(&hash_map_reference)
                } else {
                    expected_references
                        .iter()
                        .find(|(position, _)| *position == index)
                        .map(|(_, reference)| reference)
                })
            || instruction.immediate != (index == 9).then_some(ImmediateValue::Int(0))
            || instruction.local
                != (match index {
                    5 | 6 | 14 => Some(0),
                    8 | 12 => Some(1),
                    10 | 11 | 15 | 24 => Some(2),
                    17 | 19 | 21 => Some(3),
                    _ => None,
                })
            || instruction.increment != (index == 24).then_some(1)
            || instruction.interface_count != (index == 22).then_some(3)
            || instruction.branch_target_bci
                != (match index {
                    13 => Some(suffix[26].bci),
                    25 => Some(suffix[11].bci),
                    _ => None,
                })
        {
            return Ok(None);
        }
    }
    if suffix[0].width != 3
        || suffix[2].width != 3
        || suffix[3].width != 3
        || suffix[4].width != 3
        || suffix[13].width != 3
        || suffix[18].width != 3
        || suffix[20].width != 3
        || suffix[22].width != 5
        || suffix[24].width != 3
        || suffix[25].width != 3
        || suffix.iter().enumerate().any(|(index, instruction)| {
            !matches!(index, 0 | 2 | 3 | 4 | 13 | 18 | 20 | 22 | 24 | 25) && instruction.width != 1
        })
        || suffix[9].opcode != 0x03
        || suffix[16].opcode != 0x32
    {
        return Ok(None);
    }
    let suffix_uses: Vec<_> = code
        .member_uses
        .iter()
        .filter(|use_site| use_site.bci >= group.initializer_prefix_end_bci)
        .collect();
    if suffix_uses.len() != expected_references.len()
        || suffix_uses
            .iter()
            .zip(expected_references.iter())
            .any(|(actual, (index, reference))| {
                actual.bci != suffix[*index].bci || &actual.reference != reference
            })
    {
        return Ok(None);
    }
    let candidates: Vec<_> = initializer_candidates
        .iter()
        .filter(|candidate| candidate.member.as_ref() == Some(member))
        .collect();
    let [candidate] = candidates.as_slice() else {
        return Ok(None);
    };
    let steps = &candidate.steps;
    let statements = &candidate.statements;
    if candidate.has_exception_handlers || statements.len() != steps.len() {
        return Ok(None);
    }
    for (position, (step, statement)) in steps.iter().zip(statements).enumerate() {
        budget.poll()?;
        let (order, bci) = match step {
            Step::FieldWrite(write) => (write.order, write.bci),
            Step::Other { order, bci, .. } => (*order, *bci),
        };
        if order != position || bci != statement.origin.primary().bci() {
            return Ok(None);
        }
    }
    // The instruction match above proves the suffix is the single javac array loop: one fixed
    // start, contiguous instructions, exact opcodes and locals, and the one-element increment.
    // One same-run recovery presents those instructions in one of two shapes, and both are
    // admitted here: the explicit index loop, or the enhanced for whose own origins fold the same
    // traversal. Both declare the array local first, then carry the same four field writes and
    // the same loop tail; only the positions of the declarations differ.
    enum SuffixPresentation {
        IndexLoop,
        EnhancedFor,
    }
    let (presentation, writes, loop_statement) = match steps.len() {
        // Three leading declarations: the array local plus the explicit counter and limit.
        11 => (SuffixPresentation::IndexLoop, 3..7, 9_usize),
        // The enhanced for folds the counter and limit into the loop node's own origins.
        8 => (SuffixPresentation::EnhancedFor, 1..5, 6_usize),
        _ => return Ok(None),
    };
    if !steps[..writes.start].iter().all(|step| {
        matches!(
            step,
            Step::Other {
                kind: Kind::Declaration,
                ..
            }
        )
    }) || !steps[writes.clone()]
        .iter()
        .all(|step| matches!(step, Step::FieldWrite(_)))
    {
        return Ok(None);
    }
    for (offset, expected_bci) in [
        group.constants[0].field_write_bci,
        group.constants[1].field_write_bci,
        group.initializer_prefix_end_bci - 3,
        suffix[3].bci,
    ]
    .into_iter()
    .enumerate()
    {
        let Step::FieldWrite(write) = &steps[writes.start + offset] else {
            return Ok(None);
        };
        if write.bci != expected_bci {
            return Ok(None);
        }
    }
    let Step::FieldWrite(map_write) = &steps[writes.end - 1] else {
        return Ok(None);
    };
    if map_write.bci != suffix[3].bci
        || map_write.owner.0.as_slice() != owner
        || map_write.name.0.as_slice() != field_name
        || !map_write.is_static
        || map_write.op != AssignOp::Assign
        || !matches!(&map_write.value.kind, ExprKind::New { ty, qualifier: None, member_name: None, diamond: false, args } if ty == "java.util.HashMap" && args.is_empty())
    {
        return Ok(None);
    }
    if !matches!(&steps[loop_statement], Step::Other { kind: Kind::Loop, bci, .. } if *bci == suffix[13].bci)
        || !matches!(
            &steps[steps.len() - 1],
            Step::Other { bci, kind: Kind::Return, .. } if *bci == suffix[26].bci
        )
    {
        return Ok(None);
    }
    let Some(source_owner) = jarde_java::names::source_class_name(&JvmBytes(owner.to_vec())) else {
        return Ok(None);
    };
    let is_local = |expression: &Expr, expected: &str| matches!(&expression.kind, ExprKind::Local(name) if name == expected);
    // The loop body carries exactly one effect in both presentations: the proved map write keyed
    // by each element's own name, anchored at the proved interface call.
    let is_map_put = |statement: &jarde_java::ast::Stmt| matches!(&statement.kind, StmtKind::Expr(expression) if matches!(&expression.kind, ExprKind::Call { receiver: Some(receiver), name, args } if name == "put" && matches!(&receiver.kind, ExprKind::Field { receiver: owner_expr, name } if matches!(&owner_expr.kind, ExprKind::Path(path) if path == &source_owner) && name == &field_name_text) && matches!(args.as_slice(), [first, second] if matches!(&first.kind, ExprKind::Cast { ty: Type::Reference(ty), value, .. } if ty == "java.lang.Object" && matches!(&value.kind, ExprKind::Call { receiver: Some(receiver), name, args } if is_local(receiver, "local3") && name == "name" && args.is_empty())) && matches!(&second.kind, ExprKind::Cast { ty: Type::Reference(ty), value, .. } if ty == "java.lang.Object" && is_local(value, "local3")))) && expression.origin.primary().bci() == suffix[22].bci);
    if !matches!(&statements[0].kind, StmtKind::Declare { ty: Type::Reference(ty), name, value: None, .. } if ty == &format!("{source_owner}[]") && name == "local0")
    {
        return Ok(None);
    }
    let mut selected = statements[..writes.start].to_vec();
    selected.extend_from_slice(&statements[writes.end - 1..=loop_statement]);
    let StmtKind::FieldAssign { receiver, name, .. } = &mut selected[writes.start].kind else {
        unreachable!()
    };
    if name != &field_name_text {
        return Ok(None);
    }
    *receiver = None;
    match presentation {
        SuffixPresentation::IndexLoop => {
            if !steps[0..3]
                .iter()
                .zip([suffix[5].bci, suffix[8].bci, suffix[10].bci])
                .any(|(step, bci)| {
                    !matches!(step, Step::Other { bci: actual, .. } if *actual == bci)
                })
                || !matches!(&steps[7], Step::Other { kind: Kind::LocalAssignment, bci, .. } if *bci == suffix[5].bci)
                || !matches!(&steps[8], Step::Other { kind: Kind::LocalAssignment, bci, .. } if *bci == suffix[8].bci)
                || !matches!(&statements[1].kind, StmtKind::Declare { ty: Type::Int, name, value: None, .. } if name == "local1")
                || !matches!(&statements[2].kind, StmtKind::Declare { ty: Type::Int, name, value: None, .. } if name == "local2")
                || !matches!(&statements[7].kind, StmtKind::Assign { name, value } if name == "local0" && matches!(&value.kind, ExprKind::Call { receiver: None, name, args } if name == "values" && args.is_empty()) && value.origin.primary().bci() == suffix[4].bci)
                || !matches!(&statements[8].kind, StmtKind::Assign { name, value } if name == "local1" && matches!(&value.kind, ExprKind::ArrayLength { array } if is_local(array, "local0")) && value.origin.primary().bci() == suffix[7].bci)
            {
                return Ok(None);
            }
            let StmtKind::For {
                label: None,
                init,
                cond,
                update,
                body,
            } = &statements[loop_statement].kind
            else {
                return Ok(None);
            };
            if !matches!(&init.kind, StmtKind::Assign { name, value } if name == "local2" && matches!(&value.kind, ExprKind::Integer(0)))
                || !matches!(&cond.kind, ExprKind::Binary { op: BinaryOp::Less, left, right } if is_local(left, "local2") && is_local(right, "local1"))
                || !matches!(&update.kind, StmtKind::Assign { name, value } if name == "local2" && matches!(&value.kind, ExprKind::Binary { op: BinaryOp::Add, left, right } if is_local(left, "local2") && matches!(&right.kind, ExprKind::Integer(1))))
                || body.len() != 2
                || body[0].origin.primary().bci() != suffix[17].bci
                || !is_map_put(&body[1])
            {
                return Ok(None);
            }
            let StmtKind::Declare {
                ty: Type::Reference(local_type),
                name,
                value: Some(value),
                ..
            } = &body[0].kind
            else {
                return Ok(None);
            };
            if local_type != "Object"
                || name != "local3"
                || !matches!(&value.kind, ExprKind::Index { array, index } if is_local(array, "local0") && is_local(index, "local2") && value.origin.primary().bci() == suffix[16].bci)
            {
                return Ok(None);
            }
            let StmtKind::For { body, .. } = &mut selected
                .last_mut()
                .expect("the selected statements end with the loop")
                .kind
            else {
                unreachable!()
            };
            let StmtKind::Declare { ty, .. } = &mut body[0].kind else {
                unreachable!()
            };
            // The recovered element declaration states the erasure type; this proof's own fact —
            // values() returns the owner array — states the element type it projects.
            *ty = Type::Reference(source_owner.clone());
        }
        SuffixPresentation::EnhancedFor => {
            if !matches!(&steps[0], Step::Other { bci, .. } if *bci == suffix[5].bci)
                || !matches!(&steps[5], Step::Other { kind: Kind::LocalAssignment, bci, .. } if *bci == suffix[5].bci)
                || !matches!(&statements[5].kind, StmtKind::Assign { name, value } if name == "local0" && matches!(&value.kind, ExprKind::Call { receiver: None, name, args } if name == "values" && args.is_empty()) && value.origin.primary().bci() == suffix[4].bci)
            {
                return Ok(None);
            }
            let StmtKind::ForEach {
                label: None,
                ty: Type::Reference(local_type),
                name,
                iterable,
                body,
            } = &statements[loop_statement].kind
            else {
                return Ok(None);
            };
            // The node's own origins fold the proved traversal: they must carry the element read
            // the variable is bound to and the one-element increment the instruction match
            // proved, so the presented enhanced for is that index loop and nothing else.
            let loop_origins = statements[loop_statement].origin.bcis();
            if (local_type != "Object" && local_type != &source_owner)
                || name != "local3"
                || !is_local(iterable, "local0")
                || iterable.origin.primary().bci() != suffix[6].bci
                || !loop_origins.contains(&suffix[16].bci)
                || !loop_origins.contains(&suffix[24].bci)
                || body.len() != 1
                || !is_map_put(&body[0])
            {
                return Ok(None);
            }
            let StmtKind::ForEach { ty, .. } = &mut selected
                .last_mut()
                .expect("the selected statements end with the loop")
                .kind
            else {
                unreachable!()
            };
            // The recovered variable states the erasure type whenever the element-type proof did
            // not reach the presentation; the same proved element type replaces either spelling.
            *ty = Type::Reference(source_owner.clone());
        }
    }
    Ok(Some(ProvedEnumMapSuffix {
        field_index: u64::try_from(*field_index).unwrap_or(u64::MAX),
        initializer_member: member.clone(),
        statements: selected,
    }))
}

/// Source relation over already selected enum-child metadata. The supplier retains physical
/// identity, reads, and commit; preserve the original scan charges and cancellation points.
pub fn prove_enum_constant_child_relation<'m>(
    child_owner: &[u8],
    enum_owner: &[u8],
    constructor_descriptor: &[u8],
    child_pool: &[jarde_reader::classfile::CpEntryFacts],
    child_inner_classes: &[jarde_reader::classfile::InnerClassFacts],
    enum_pool: &[jarde_reader::classfile::CpEntryFacts],
    enum_inner_classes: &[jarde_reader::classfile::InnerClassFacts],
    enclosing_method: Option<&jarde_reader::classfile::EnclosingMethodFacts>,
    child_methods: &'m [MemberHeader],
    budget: &mut Budget,
) -> Result<Option<(Vec<Vec<u8>>, &'m MemberHeader)>> {
    let inner_matches: Vec<_> = child_inner_classes
        .iter()
        .filter(|inner| {
            jarde_reader::classfile::cp_class_name(child_pool, inner.class_index)
                .is_ok_and(|name| name.0 == child_owner)
        })
        .collect();
    let [inner] = inner_matches.as_slice() else {
        return Ok(None);
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(child_inner_classes.len()).unwrap_or(u64::MAX),
    )?;
    let mut inner_rows = std::collections::BTreeMap::<Vec<u8>, (usize, bool)>::new();
    for row in child_inner_classes {
        budget.poll()?;
        if let Ok(name) = jarde_reader::classfile::cp_class_name(child_pool, row.class_index) {
            let entry = inner_rows.entry(name.0).or_insert((0, false));
            entry.0 += 1;
            entry.1 = row.outer_class_index == 0 && row.inner_name.is_none();
        }
    }
    let anonymous_inner_owners = inner_rows
        .into_iter()
        .filter_map(|(owner, (count, anonymous))| (count == 1 && anonymous).then_some(owner))
        .collect();
    let enum_inner_rows: Vec<_> = enum_inner_classes
        .iter()
        .filter(|inner| {
            jarde_reader::classfile::cp_class_name(enum_pool, inner.class_index)
                .is_ok_and(|name| name.0 == child_owner)
        })
        .collect();
    let [enum_inner] = enum_inner_rows.as_slice() else {
        return Ok(None);
    };
    let Some(enclosing) = enclosing_method else {
        return Ok(None);
    };
    if inner.outer_class_index != 0
        || inner.inner_name.is_some()
        || enum_inner.outer_class_index != 0
        || enum_inner.inner_name.is_some()
        || jarde_reader::classfile::cp_class_name(child_pool, enclosing.class_index)
            .map_or(true, |name| name.0 != enum_owner)
        || enclosing.method_index != 0
    {
        return Ok(None);
    }
    let child_constructors: Vec<_> = child_methods
        .iter()
        .filter(|method| method.name.raw().0 == b"<init>")
        .collect();
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(child_methods.len()).unwrap_or(u64::MAX),
    )?;
    let matching_constructors: Vec<_> = child_constructors
        .iter()
        .filter(|method| method.descriptor.raw().0 == constructor_descriptor)
        .collect();
    let [child_constructor] = matching_constructors.as_slice() else {
        return Ok(None);
    };
    if child_constructors.len() != 1 {
        return Ok(None);
    }
    Ok(Some((anonymous_inner_owners, **child_constructor)))
}

/// Source eligibility after the supplier has verified the selected enum child's identity.
pub fn enum_constant_child_header_is_supported(
    declaration_present: bool,
    field_count: usize,
) -> std::result::Result<(), &'static str> {
    if !declaration_present {
        return Err("the selected child source changed physical identity");
    }
    if field_count != 0 {
        return Err("the selected child has an instance or static field");
    }
    Ok(())
}

/// Use the same faithful JVM method-name spelling as declarations, without creating an alias.
pub fn enum_constant_child_method_is_supported(
    raw_name: &[u8],
    flags: u16,
) -> std::result::Result<(), &'static str> {
    let raw_name = JvmBytes(raw_name.to_vec());
    if crate::names::source_method_name(&raw_name).is_none() {
        // Only rejected names need another decode to distinguish encoding from source grammar.
        return Err(if raw_name.to_unicode().is_none() {
            "the selected child method name is not Java text"
        } else {
            "the selected child has a non-source override member"
        });
    }
    if flags & (0x0002 | 0x0008 | 0x0040 | 0x0100 | 0x0400 | 0x1000) != 0 {
        return Err("the selected child has a non-source override member");
    }
    Ok(())
}

/// Source visibility over the one base declaration already paired by raw physical facts.
pub fn enum_constant_child_visibility_is_supported(base_flags: u16, child_flags: u16) -> bool {
    base_flags & (0x0002 | 0x0008 | 0x0010) == 0
        && (base_flags & 0x0001 == 0 || child_flags & 0x0001 != 0)
        && (base_flags & 0x0004 == 0 || child_flags & (0x0001 | 0x0004) != 0)
}

/// A same-run child body is presentable independently of optional evidence publication.
pub fn enum_constant_child_recovery_is_supported(
    report: &crate::report::RecoveryReport,
    analysis_execution: &ExecutionReport,
    declaration_present: bool,
    markers_empty: bool,
) -> bool {
    matches!(analysis_execution, ExecutionReport::Complete { .. })
        && matches!(&report.execution, ExecutionReport::Complete { .. })
        && report.produced()
        && report.representation == jarde_jvm::ir::Representation::Java
        && report.quality == jarde_jvm::ir::Quality::Structured
        && report.syntax_status != jarde_jvm::ir::SyntaxStatus::NotJava
        && report.content == crate::report::RecoveryContent::ContainsStatements
        && report.fallbacks.is_empty()
        && report.projection_regions_structured()
        && report.projection_has_own_origin()
        && report.projection_declaration_presented()
        && declaration_present
        && markers_empty
}

/// Static-target source eligibility before the supplier reads the selected child definition.
pub fn static_member_target_root_is_supported(flags: u16, attributes: &[AttributeShell]) -> bool {
    flags & 0x0008 != 0
        && !attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Signature")
}

/// The existing no-capture child source slice; reader completeness stays with the supplier.
pub fn static_member_target_child_is_supported(
    fields: &[MemberHeader],
    attributes: &[AttributeShell],
) -> bool {
    fields.is_empty()
        && !attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Signature")
}

/// Select the one constructor in a caller-confirmed complete table, without inventing identity.
pub fn static_member_constructor_header(methods: &[MemberHeader]) -> Option<&MemberHeader> {
    let mut constructors = methods
        .iter()
        .filter(|method| method.name.raw().0 == b"<init>");
    let constructor = constructors.next()?;
    if constructors.next().is_some() || constructor.descriptor.raw().0 != b"()V" {
        return None;
    }
    Some(constructor)
}

/// Admit an already read and declared integer constant using faithful raw JVM name spelling.
pub fn integer_constant_source_candidate(
    constant: Option<&crate::metadata_literal::MemberDefault>,
    declaration_present: bool,
    flags: u16,
    descriptor: &[u8],
    attributes: &[AttributeShell],
    raw_name: &[u8],
) -> Option<crate::report::IntegerConstantName> {
    let Some(crate::metadata_literal::MemberDefault::Integer(value)) = constant else {
        return None;
    };
    if !declaration_present
        || flags & (0x0008 | 0x0010) != (0x0008 | 0x0010)
        || descriptor != b"I"
        || attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"ConstantValue")
            .count()
            != 1
    {
        return None;
    }
    let name = crate::names::source_identifier(raw_name)?;
    Some(crate::report::IntegerConstantName {
        value: *value,
        name,
    })
}

/// One decision per candidate, in original order. Count all physical field names first;
/// duplicate values are considered only after candidates with non-unique names are excluded.
pub fn unique_integer_constant_candidate_mask<'a>(
    fields: impl Iterator<Item = &'a [u8]>,
    candidates: impl Iterator<Item = (&'a [u8], i32)> + Clone,
) -> Vec<bool> {
    let mut names = std::collections::HashMap::<Vec<u8>, usize>::new();
    for name in fields {
        *names.entry(name.to_vec()).or_default() += 1;
    }
    let mut values = std::collections::HashMap::<i32, usize>::new();
    for (name, value) in candidates.clone() {
        if names.get(name) == Some(&1) {
            *values.entry(value).or_default() += 1;
        }
    }
    candidates
        .map(|(name, value)| names.get(name) == Some(&1) && values.get(&value) == Some(&1))
        .collect()
}

#[cfg(test)]
mod metadata_source_policy_tests {
    use super::*;
    use jarde_reader::model::{ByteSpan, JvmString};

    #[test]
    fn metadata_admission_keeps_raw_selectors_and_candidate_filter_order() {
        assert!(enum_constant_child_header_is_supported(true, 0).is_ok());
        assert_eq!(
            enum_constant_child_header_is_supported(false, 1),
            Err("the selected child source changed physical identity")
        );
        assert_eq!(
            enum_constant_child_header_is_supported(true, 1),
            Err("the selected child has an instance or static field")
        );
        assert!(enum_constant_child_method_is_supported(b"apply", 1).is_ok());
        assert!(enum_constant_child_method_is_supported("计算".as_bytes(), 4).is_ok());
        for flags in [2, 8, 0x40, 0x100, 0x400, 0x1000] {
            assert_eq!(
                enum_constant_child_method_is_supported(b"apply", flags),
                Err("the selected child has a non-source override member")
            );
        }
        for name in ["_", "apply𐐀"] {
            let raw = JvmBytes::from_unicode(name);
            assert!(enum_constant_child_method_is_supported(&raw.0, 1).is_ok());
            assert_eq!(
                enum_constant_child_method_is_supported(&raw.0, 2),
                Err("the selected child has a non-source override member")
            );
        }
        for name in ["for", "", "<clinit>", "a\u{200b}b"] {
            assert_eq!(
                enum_constant_child_method_is_supported(&JvmBytes::from_unicode(name).0, 1),
                Err("the selected child has a non-source override member")
            );
        }
        for raw in [
            b"\xff".as_slice(),
            b"apply\xed\xa0\x81",
            b"apply\xf0\x90\x90\x80",
            b"\xed\xa0",
        ] {
            assert_eq!(
                enum_constant_child_method_is_supported(raw, 2),
                Err("the selected child method name is not Java text"),
                "name decoding still precedes the access-flag refusal"
            );
        }
        for flags in [2, 8, 16] {
            assert!(!enum_constant_child_visibility_is_supported(flags, 1));
        }
        assert!(!enum_constant_child_visibility_is_supported(1, 4));
        assert!(!enum_constant_child_visibility_is_supported(4, 0));
        for (base, child) in [(0, 0), (1, 1), (4, 1), (4, 4)] {
            assert!(enum_constant_child_visibility_is_supported(base, child));
        }
        let attribute = |name: &str| AttributeShell {
            name: JvmString::from_unicode(name),
            span: ByteSpan::new(0, 0),
            content_span: ByteSpan::new(0, 0),
        };
        let member = |name: &str, descriptor: &str| MemberHeader {
            name: JvmString::from_unicode(name),
            descriptor: JvmString::from_unicode(descriptor),
            access_flags: 1,
            attributes: Vec::new(),
        };
        let signature = [attribute("Signature")];
        let constant_attribute = [attribute("ConstantValue")];
        assert!(static_member_target_root_is_supported(0x0009, &[]));
        assert!(!static_member_target_root_is_supported(1, &[]));
        assert!(!static_member_target_root_is_supported(8, &signature));
        assert!(static_member_target_child_is_supported(
            &[],
            &constant_attribute
        ));
        assert!(!static_member_target_child_is_supported(&[], &signature));
        assert!(!static_member_target_child_is_supported(
            &[member("field", "I")],
            &[]
        ));

        let methods = [member("f", "()I"), member("<init>", "()V")];
        assert!(std::ptr::eq(
            static_member_constructor_header(&methods).unwrap(),
            &methods[1]
        ));
        for methods in [
            Vec::new(),
            vec![member("f", "()I")],
            vec![member("<init>", "(I)V")],
            vec![member("<init>", "()V"), member("<init>", "()V")],
            vec![member("<init>", "()V"), member("<init>", "(I)V")],
        ] {
            assert!(static_member_constructor_header(&methods).is_none());
        }

        let integer = crate::metadata_literal::MemberDefault::Integer(7);
        let candidate =
            |declaration, flags, descriptor: &[u8], attrs: &[AttributeShell], name: &[u8]| {
                integer_constant_source_candidate(
                    Some(&integer),
                    declaration,
                    flags,
                    descriptor,
                    attrs,
                    name,
                )
            };
        let admitted = candidate(true, 0x0018, b"I", &constant_attribute, b"VALUE").unwrap();
        assert_eq!((admitted.name.as_str(), admitted.value), ("VALUE", 7));
        assert!(candidate(false, 0x0018, b"I", &constant_attribute, b"VALUE").is_none());
        assert!(candidate(true, 0x0008, b"I", &constant_attribute, b"VALUE").is_none());
        assert!(candidate(true, 0x0018, b"J", &constant_attribute, b"VALUE").is_none());
        assert!(candidate(true, 0x0018, b"I", &[], b"VALUE").is_none());
        assert!(
            candidate(
                true,
                0x0018,
                b"I",
                &[attribute("ConstantValue"), attribute("ConstantValue")],
                b"VALUE"
            )
            .is_none()
        );
        assert!(
            integer_constant_source_candidate(
                None,
                true,
                0x0018,
                b"I",
                &constant_attribute,
                b"VALUE"
            )
            .is_none()
        );
        assert!(
            integer_constant_source_candidate(
                Some(&crate::metadata_literal::MemberDefault::Long(7)),
                true,
                0x0018,
                b"I",
                &constant_attribute,
                b"VALUE"
            )
            .is_none()
        );
        for name in ["_", "class", "a\u{200b}b"] {
            assert!(
                candidate(
                    true,
                    0x0018,
                    b"I",
                    &constant_attribute,
                    &JvmBytes::from_unicode(name).0
                )
                .is_none()
            );
        }
        for name in ["汉", "KEY𐐀"] {
            assert_eq!(
                candidate(
                    true,
                    0x0018,
                    b"I",
                    &constant_attribute,
                    &JvmBytes::from_unicode(name).0
                )
                .unwrap()
                .name,
                name
            );
        }
        for raw in [
            b"KEY\xf0\x90\x90\x80".as_slice(),
            b"\xed\xa0\x81",
            b"\xe6\xb1",
            b"\xff",
            b"a\xc0\x80b",
        ] {
            assert!(candidate(true, 0x0018, b"I", &constant_attribute, raw).is_none());
        }

        let fields = [b"A".as_slice(), b"A", b"B", b"C"];
        let candidates = [
            (b"A".as_slice(), 7),
            (b"B".as_slice(), 7),
            (b"C".as_slice(), 9),
        ];
        assert_eq!(
            unique_integer_constant_candidate_mask(fields.into_iter(), candidates.into_iter()),
            [false, true, true],
            "an excluded duplicate name must not make an otherwise unique value ambiguous"
        );
        assert_eq!(
            unique_integer_constant_candidate_mask(
                [b"A".as_slice(), b"B"].into_iter(),
                [(b"A".as_slice(), 7), (b"B".as_slice(), 7)].into_iter()
            ),
            [false, false]
        );
        assert_eq!(
            unique_integer_constant_candidate_mask(
                [].into_iter(),
                [(b"A".as_slice(), 7)].into_iter()
            ),
            [false]
        );
        let supplementary = JvmBytes::from_unicode("KEY𐐀");
        let raw = supplementary.0.as_slice();
        assert_ne!(raw, "KEY𐐀".as_bytes());
        assert_eq!(
            unique_integer_constant_candidate_mask([raw].into_iter(), [(raw, 7)].into_iter()),
            [true]
        );
        assert_eq!(
            unique_integer_constant_candidate_mask(
                [raw].into_iter(),
                [("KEY𐐀".as_bytes(), 7)].into_iter()
            ),
            [false],
            "source UTF-8 spelling must never replace the physical MUTF-8 key"
        );
        assert_eq!(
            unique_integer_constant_candidate_mask(
                [raw, raw, b"B"].into_iter(),
                [(raw, 7), (b"B".as_slice(), 7)].into_iter()
            ),
            [false, true]
        );
    }
}

/// Check source applicability without replacing the physical target's raw identity.
/// Special-call producers and Java handoff consumers share this spelling boundary.
pub fn source_method_target_is_spellable(
    owner: &JvmBytes,
    name: &JvmBytes,
    descriptor: &JvmBytes,
) -> bool {
    crate::names::source_class_name(owner).is_some()
        && crate::names::source_method_name(name).is_some()
        && crate::lambda::parse_method_bytes(&descriptor.0).is_some()
}

/// The existing member-call census source-name domain, before physical capture/header checks.
/// Keep root decoding, member spelling and the conservative root predicate in that order;
/// faithful class-name spelling alone admits names this source-family proof has not accepted.
pub fn member_family_root_source_name(
    root_name: &[u8],
    member_name: &JvmBytes,
) -> std::result::Result<String, &'static str> {
    let root_binary = JvmBytes(root_name.to_vec())
        .to_unicode()
        .ok_or("root binary name cannot be used for a Java source path")?;
    source_class_name(member_name)
        .ok_or("member binary name cannot be used for a Java source path")?;
    if !root_binary.split('/').all(crate::names::is_java_identifier) {
        return Err("root binary name has no proved Java source spelling");
    }
    Ok(root_binary.replace('/', "."))
}

/// The existing empty-constructor source slice for an already selected static member target.
/// The supplier still binds the method identity and checks analysis completion before this call.
pub fn static_member_constructor_shape_is_supported(ir: &jarde_jvm::method_ir::MethodIr) -> bool {
    let Some(code) = ir.code() else {
        return false;
    };
    if code.exception_handler_count != 0
        || !code.exception_handlers.is_empty()
        || code.instructions.len() != 3
        || code.instructions[0].opcode != 0x2a
        || code.instructions[1].opcode != 0xb7
        || code.instructions[2].opcode != 0xb1
    {
        return false;
    }
    let Some(index) = code.instructions[1].constant_pool_index else {
        return false;
    };
    let Ok(entry) = jarde_reader::classfile::cp_entry(ir.constant_pool(), index) else {
        return false;
    };
    matches!(entry.kind, CpEntryKind::MethodRef { ref owner, ref name, ref descriptor, .. }
        if owner.0 == b"java/lang/Object" && name.0 == b"<init>" && descriptor.0 == b"()V")
}

/// Source recovery must independently confirm the empty static-member constructor slice.
pub fn static_member_constructor_recovery_is_supported(
    recovery: &crate::report::ClassSourceRecovery,
) -> bool {
    recovery.report.produced()
        && recovery.report.quality == jarde_jvm::ir::Quality::Structured
        && recovery.report.fallbacks.is_empty()
        && matches!(recovery.report.execution, ExecutionReport::Complete { .. })
        && recovery
            .candidates
            .generic_constructor
            .as_ref()
            .is_some_and(|candidate| candidate.field_writes.is_empty())
}

/// Static targets retain their own name domain: both names must decode, but only the root
/// has the conservative segment predicate here. The physical relation proves the child path.
pub fn static_member_root_source_name(root_name: &[u8], child_name: &[u8]) -> Option<String> {
    let root_text = JvmBytes(root_name.to_vec()).to_unicode();
    let child_text = JvmBytes(child_name.to_vec()).to_unicode();
    let (Some(root_internal), Some(_child_internal)) =
        (root_text.as_deref(), child_text.as_deref())
    else {
        return None;
    };
    if !root_internal
        .split('/')
        .all(crate::names::is_java_identifier)
    {
        return None;
    }
    Some(root_internal.replace('/', "."))
}

/// Selected inputs for the closed Outer.super source check. The request owner performs reads
/// and merges their original execution reports; Java decides applicability over those facts.
/// Supplying facts here is not a trusted artifact binding.
pub trait OuterSuperSourceReads {
    fn header(
        &mut self,
        owner: &[u8],
        budget: &mut Budget,
    ) -> Result<Option<(PhysicalDefinitionId, ClassMemberFacts)>>;
    fn analyze(
        &mut self,
        method: &PhysicalMethodId,
        budget: &mut Budget,
    ) -> Result<jarde_jvm::method_ir::MethodIrAnalysis>;
}

/// Source overload, generic and checked-exception applicability after the physical bridge and
/// use census have closed. Calls and published declaration facts are borrowed, not retained.
/// This preserves the existing single-reference-parameter subset and its lazy SSA demand.
pub fn prove_outer_super_source_binding<'a>(
    outer: &ClassMemberFacts,
    parent_definition: &PhysicalDefinitionId,
    parent: &ClassMemberFacts,
    target: &PhysicalMethodId,
    child_definition: &PhysicalDefinitionId,
    child_name: Option<&[u8]>,
    calls: impl Iterator<Item = (&'a PhysicalMethodId, &'a [u32])> + Clone,
    declarations: impl Iterator<Item = (&'a PhysicalMethodId, bool, bool)> + Clone,
    reads: &mut impl OuterSuperSourceReads,
    budget: &mut Budget,
) -> Result<std::result::Result<(), String>> {
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    if outer
        .attributes
        .iter()
        .any(|attribute| attribute.name.raw().0 == b"Signature")
    {
        return refuse("bridge target or Outer generic declaration cannot bind as Java source");
    }
    if !source_method_target_is_spellable(parent.this_class.raw(), &target.name, &target.descriptor)
    {
        return refuse("bridge target has no exact Java source spelling");
    }
    let target_parameters = descriptor_facts(&target.descriptor.0, DescriptorKind::Method)
        .ok()
        .map(|descriptor| descriptor.parameters().to_vec());
    let mut proved_argument_type = false;
    let mut pending = vec![(parent.this_class.raw().0.clone(), false)];
    pending.extend(
        outer
            .interfaces
            .iter()
            .map(|interface| (interface.raw().0.clone(), true)),
    );
    let mut visited = std::collections::BTreeSet::new();
    let mut exact_target = 0_u32;
    while let Some((name, interface)) = pending.pop() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if name == b"java/lang/Object" {
            const OBJECT_METHODS: &[&[u8]] = &[
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
            if OBJECT_METHODS.contains(&target.name.0.as_slice()) {
                return refuse(
                    "Object method name prevents proving a source binding without the selected JDK declaration",
                );
            }
            continue;
        }
        if !visited.insert(name.clone()) {
            continue;
        }
        let (definition, facts) = if name == parent.this_class.raw().0 && !interface {
            (parent_definition.clone(), parent.clone())
        } else {
            let Some((definition, read)) = reads.header(&name, budget)? else {
                return refuse(
                    "selected superclass or interface declaration is unavailable for Java source binding",
                );
            };
            (definition, read)
        };
        if facts.stopped_at.is_some()
            || facts.method_count != facts.methods.len() as u64
            || facts.field_count != facts.fields.len() as u64
            || facts.this_class.raw().0 != name
            || (facts.access_flags & ACC_INTERFACE != 0) != interface
            || facts
                .attributes
                .iter()
                .any(|attribute| attribute.name.raw().0 == b"Signature")
        {
            return refuse("source hierarchy declaration is incomplete or generic");
        }
        for method in &facts.methods {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if method.name.raw().0 != target.name.0 {
                continue;
            }
            let is_target =
                definition == target.owner && method.descriptor.raw().0 == target.descriptor.0;
            if !is_target {
                if method.access_flags & (0x0040 | 0x0080 | 0x1000) != 0
                    || method.attributes.iter().any(|attribute| {
                        matches!(
                            attribute.name.raw().0.as_slice(),
                            b"Signature" | b"Exceptions"
                        )
                    })
                {
                    return refuse("competing method has an unproved source signature");
                }
                let Some(candidate) =
                    descriptor_facts(method.descriptor.raw().0.as_slice(), DescriptorKind::Method)
                        .ok()
                else {
                    return refuse("competing method descriptor is unreadable");
                };
                let Some(target_parameters) = target_parameters.as_ref() else {
                    return refuse("bridge target parameters are unreadable");
                };
                if candidate.parameters().len() != target_parameters.len() {
                    continue;
                }
                let ([target_arg], [candidate_arg]) =
                    (target_parameters.as_slice(), candidate.parameters())
                else {
                    return refuse(
                        "same-arity overload is outside the single-reference-argument proof",
                    );
                };
                let (Some(target_name), Some(candidate_name)) =
                    (target_arg.object_name(), candidate_arg.object_name())
                else {
                    return refuse("same-arity overload has an unproved parameter conversion");
                };
                if target_arg.is_array() || candidate_arg.is_array() {
                    return refuse("array overload conversion is unproved");
                }
                if !proved_argument_type {
                    if let Err(reason) = prove_outer_super_argument_type(
                        child_definition,
                        child_name,
                        calls.clone(),
                        declarations.clone(),
                        target_arg.bytes(&target.descriptor.0).unwrap_or_default(),
                        reads,
                        budget,
                    )? {
                        return Ok(Err(reason));
                    }
                    proved_argument_type = true;
                }
                if !proved_strict_class_subtype(&candidate_name.0, &target_name.0, reads, budget)? {
                    return refuse("source hierarchy contains a competing same-name method");
                }
                continue;
            }
            exact_target += 1;
            if exact_target != 1
                || method.access_flags & (0x0040 | 0x0080 | 0x1000) != 0
                || method.attributes.iter().any(|attribute| {
                    matches!(
                        attribute.name.raw().0.as_slice(),
                        b"Signature" | b"Exceptions"
                    )
                })
            {
                return refuse("target method generic or checked-exception surface is unproved");
            }
        }
        pending.extend(
            facts
                .interfaces
                .iter()
                .map(|interface| (interface.raw().0.clone(), true)),
        );
        if !interface {
            let Some(super_class) = facts.super_class.as_ref() else {
                return refuse("source superclass chain ends before java/lang/Object");
            };
            pending.push((super_class.raw().0.clone(), false));
        }
    }
    if exact_target != 1 {
        return refuse("direct parent target has no unique source declaration");
    }
    Ok(Ok(()))
}

/// The source parameter must be published and consumed by its direct entry-local load.
fn prove_outer_super_argument_type<'a>(
    child_definition: &PhysicalDefinitionId,
    child_name: Option<&[u8]>,
    calls: impl Iterator<Item = (&'a PhysicalMethodId, &'a [u32])>,
    declarations: impl Iterator<Item = (&'a PhysicalMethodId, bool, bool)> + Clone,
    expected: &[u8],
    reads: &mut impl OuterSuperSourceReads,
    budget: &mut Budget,
) -> Result<std::result::Result<(), String>> {
    use jarde_jvm::method_ir::{Definition, Slot};

    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    let Some(child_name) = child_name else {
        return refuse("member source declaration is unavailable");
    };
    let Some((selected, read)) = reads.header(child_name, budget)? else {
        return refuse("selected member declaration is unavailable for argument proof");
    };
    let facts = read;
    if selected != *child_definition
        || facts.stopped_at.is_some()
        || facts.method_count != facts.methods.len() as u64
        || facts.this_class.raw().0.as_slice() != child_name
        || facts
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Signature")
    {
        return refuse("member source declaration is incomplete or generic");
    }
    for (caller, argument_bcis) in calls {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let [argument_bci] = argument_bcis else {
            return refuse("overloaded bridge needs one direct member parameter argument");
        };
        let Some(method) = facts.methods.iter().find(|method| {
            method.name.raw().0 == caller.name.0 && method.descriptor.raw().0 == caller.descriptor.0
        }) else {
            return refuse("member caller has no selected physical declaration");
        };
        if method
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Signature")
            || !declarations
                .clone()
                .any(|(method, declared, generic_refused)| {
                    method == caller && declared && !generic_refused
                })
        {
            return refuse("member caller source parameter type is unproved");
        }
        let Ok(signature) = descriptor_facts(&caller.descriptor.0, DescriptorKind::Method) else {
            return refuse("member caller descriptor is unreadable");
        };
        let [parameter] = signature.parameters() else {
            return refuse("member caller is outside the single-parameter proof");
        };
        if parameter.bytes(&caller.descriptor.0) != Some(expected) {
            return refuse("member source parameter differs from bridge target parameter");
        }
        let slot = if method.access_flags & 0x0008 == 0 {
            1
        } else {
            0
        };
        let analyzed = reads.analyze(caller, budget)?;
        if analyzed.report().method != *caller
            || !matches!(
                analyzed.report().execution,
                ExecutionReport::Complete { .. }
            )
        {
            return refuse("member caller SSA is incomplete for source argument proof");
        }
        let Some(ssa) = analyzed.ir().ssa() else {
            return refuse("member caller SSA is unavailable");
        };
        let loads: Vec<_> = ssa
            .blocks()
            .iter()
            .flat_map(|block| block.instructions())
            .filter(|instruction| instruction.bci() == *argument_bci)
            .collect();
        let [load] = loads.as_slice() else {
            return refuse("bridge argument has no unique direct load");
        };
        if !matches!(load.opcode(), 0x19 | 0x2a..=0x2d)
            || !matches!(load.reads(), [(Slot::Local(actual), value)]
                if *actual == slot && matches!(ssa.value(*value).def(), Definition::Entry { slot: Slot::Local(entry), .. } if *entry == slot))
            || load.writes().len() != 1
            || !matches!(ssa.value(load.writes()[0].1).def(), Definition::Instruction { bci, .. } if *bci == *argument_bci)
        {
            return refuse("bridge argument is not a direct member parameter load");
        }
    }
    Ok(Ok(()))
}

/// A statically parent-typed value cannot select a proved strict-subclass parameter.
fn proved_strict_class_subtype(
    narrow: &[u8],
    parent: &[u8],
    reads: &mut impl OuterSuperSourceReads,
    budget: &mut Budget,
) -> Result<bool> {
    if narrow == parent {
        return Ok(false);
    }
    let mut next = narrow.to_vec();
    let mut seen = std::collections::BTreeSet::new();
    loop {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !seen.insert(next.clone()) {
            return Ok(false);
        }
        let Some((_, read)) = reads.header(&next, budget)? else {
            return Ok(false);
        };
        let facts = read;
        if facts.stopped_at.is_some()
            || facts.this_class.raw().0 != next
            || facts.method_count != facts.methods.len() as u64
            || facts.field_count != facts.fields.len() as u64
            || facts.access_flags & ACC_INTERFACE != 0
            || facts
                .attributes
                .iter()
                .any(|attribute| attribute.name.raw().0 == b"Signature")
        {
            return Ok(false);
        }
        if next == parent {
            return Ok(true);
        }
        let Some(super_class) = facts.super_class.as_ref() else {
            return Ok(false);
        };
        next = super_class.raw().0.clone();
    }
}

/// 同次依赖需求的事实状态。Stop 由外层 Result 保留原错误/报告。
/// Unknown 是尚未完成或无法确定，不能授权平台替代，也不能写成负缓存。
/// 同步 Header 查找没有重入回调；层级路径的进行中节点由各 walk 的 ancestor/visited
/// 持有，循环不生产 Missing，不在这里增加没有生产者的缓存状态。
#[derive(Clone)]
pub enum DependencyResult<T> {
    Found(T),
    Missing,
    Ambiguous,
    Unknown,
}

impl<T> DependencyResult<T> {
    /// 只消费事实的候选在最后一层显式拒绝所有未证明状态。
    pub fn found(self) -> Option<T> {
        match self {
            Self::Found(value) => Some(value),
            Self::Missing | Self::Ambiguous | Self::Unknown => None,
        }
    }
}

/// Request-bound selected headers, with resolver uncertainty and stops preserved by the owner.
/// Implementations must not interpret an incomplete lookup as clean platform absence.
pub trait RecoveryHeaders {
    type Error: From<Error>;
    fn caller_loader(&self) -> &LoaderId;
    fn selected(
        &mut self,
        owner: &[u8],
        budget: &mut Budget,
    ) -> std::result::Result<DependencyResult<ReferenceClassHeader>, Self::Error> {
        let loader = self.caller_loader().clone();
        self.selected_from(&loader, owner, budget)
    }
    fn selected_from(
        &mut self,
        initiating_loader: &LoaderId,
        owner: &[u8],
        budget: &mut Budget,
    ) -> std::result::Result<DependencyResult<ReferenceClassHeader>, Self::Error>;
    /// The same selected declaration with its generic metadata established. The default keeps
    /// existing suppliers usable as raw-header suppliers; absent signature facts prove nothing.
    fn selected_generic_from(
        &mut self,
        initiating_loader: &LoaderId,
        owner: &[u8],
        budget: &mut Budget,
    ) -> std::result::Result<DependencyResult<ReferenceClassHeader>, Self::Error> {
        self.selected_from(initiating_loader, owner, budget)
    }
    /// Resolve the physical member that the original use names. Generic substitution must not
    /// choose an inherited declaration by a second, source-only member search.
    fn resolve_member(
        &mut self,
        _symbol: &jarde_reader::model::SymbolRef,
        _use_kind: jarde_jvm::resolver::ReferenceUse,
        budget: &mut Budget,
    ) -> std::result::Result<DependencyResult<jarde_jvm::resolver::ResolvedMemberRef>, Self::Error>
    {
        budget.poll()?;
        Ok(DependencyResult::Unknown)
    }
    fn platform_unprovided(
        &mut self,
        owner: &[u8],
        budget: &mut Budget,
    ) -> std::result::Result<bool, Self::Error>;
}

/// Follow declared class/interface arguments through selected definitions. This computes the
/// target application; bounds, access and overload/source admission remain the consumer's work.
/// Conflicting applications of one physical definition are not resolved by traversal order.
pub fn declared_generic_supertype<H: RecoveryHeaders>(
    source: &jarde_reader::signature::ClassType,
    initiating_loader: &LoaderId,
    target: &[u8],
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<
    DependencyResult<(ReferenceClassHeader, jarde_reader::signature::ClassType)>,
    H::Error,
> {
    use jarde_reader::signature::{ClassType, ClassTypeSegment, SignatureType, TypeArgument};
    use std::collections::{HashMap, VecDeque};
    let mut pending = VecDeque::new();
    budget.charge(CountedBudgetDimension::IrItems, 1)?;
    pending.push_back((initiating_loader.clone(), source.clone(), Vec::new()));
    let mut seen = HashMap::new();
    let mut matched = None;
    let mut incomplete = false;
    while let Some((loader, application, mut ancestors)) = pending.pop_front() {
        budget.poll()?;
        budget.observe_dependency_depth(ancestors.len() as u64)?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let Some(segment) = application.segments.last() else {
            return Ok(DependencyResult::Unknown);
        };
        // Enclosing generic instances need their own enclosing-variable environment.
        if application.segments[..application.segments.len() - 1]
            .iter()
            .any(|segment| !segment.arguments.is_empty())
        {
            return Ok(DependencyResult::Unknown);
        }
        let header = match headers.selected_generic_from(&loader, &segment.binary_name, budget)? {
            DependencyResult::Found(header) => header,
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Missing | DependencyResult::Unknown => {
                incomplete = true;
                continue;
            }
        };
        let key = (header.loader.clone(), header.definition.clone());
        if ancestors.contains(&key) {
            return Ok(DependencyResult::Unknown);
        }
        if let Some(previous) = seen.get(&key) {
            if previous != &application {
                return Ok(DependencyResult::Ambiguous);
            }
            continue;
        }
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        seen.insert(key.clone(), application.clone());
        let Some(signatures) = &header.generic_signatures else {
            incomplete = true;
            continue;
        };
        let parameters = signatures
            .class
            .as_ref()
            .map_or(&[][..], |(class, _proof)| class.type_parameters.as_slice());
        if parameters.len() != segment.arguments.len() {
            return Ok(DependencyResult::Unknown);
        }
        let mut arguments = Vec::new();
        for argument in &segment.arguments {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            let TypeArgument::Exact(argument) = argument else {
                return Ok(DependencyResult::Unknown);
            };
            arguments.push(argument.clone());
        }
        if segment.binary_name == target {
            if matched
                .as_ref()
                .is_some_and(|(_, previous)| previous != &application)
            {
                return Ok(DependencyResult::Ambiguous);
            }
            matched = Some((header, application));
            continue;
        }
        let parent_count = signatures.class.as_ref().map_or_else(
            || usize::from(header.super_name.is_some()) + header.interfaces.len(),
            |(class, _proof)| 1 + class.interfaces.len(),
        );
        budget.charge(CountedBudgetDimension::IrItems, parent_count as u64)?;
        let parents = if let Some((class, _proof)) = &signatures.class {
            std::iter::once(&class.superclass)
                .chain(class.interfaces.iter())
                .cloned()
                .collect::<Vec<_>>()
        } else {
            header
                .super_name
                .iter()
                .chain(header.interfaces.iter())
                .map(|name| ClassType {
                    segments: vec![ClassTypeSegment {
                        binary_name: name.clone(),
                        arguments: Vec::new(),
                    }],
                })
                .collect()
        };
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        ancestors.push(key);
        for parent in parents {
            let substituted = crate::generic_source_proof::substitute_signature_type_parameters(
                &SignatureType::Class(parent),
                parameters,
                &arguments,
                budget,
                0,
            )?;
            let Some(SignatureType::Class(parent)) = substituted else {
                return Ok(DependencyResult::Unknown);
            };
            budget.charge(CountedBudgetDimension::IrItems, 1 + ancestors.len() as u64)?;
            budget.charge(CountedBudgetDimension::IrEdges, 1)?;
            pending.push_back((header.loader.clone(), parent, ancestors.clone()));
        }
    }
    Ok(match (matched, incomplete) {
        (_, true) => DependencyResult::Unknown,
        (Some(found), false) => DependencyResult::Found(found),
        (None, false) => DependencyResult::Missing,
    })
}

fn declared_member_application<H: RecoveryHeaders>(
    source: &jarde_reader::signature::ClassType,
    initiating_loader: &LoaderId,
    member: &jarde_jvm::resolver::ResolvedMemberRef,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<
    DependencyResult<(
        ReferenceClassHeader,
        Vec<jarde_reader::signature::SignatureType>,
    )>,
    H::Error,
> {
    use jarde_reader::{model::SymbolRef, signature::TypeArgument};
    let owner = match &member.member {
        SymbolRef::Field { owner, .. } | SymbolRef::Method { owner, .. } => &owner.0,
        _ => return Ok(DependencyResult::Unknown),
    };
    let found = declared_generic_supertype(source, initiating_loader, owner, headers, budget)?;
    let (header, application) = match found {
        DependencyResult::Found(found) => found,
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
    };
    if header.definition != member.definition || header.loader != member.loader {
        return Ok(DependencyResult::Unknown);
    }
    let Some(segment) = application.segments.last() else {
        return Ok(DependencyResult::Unknown);
    };
    let mut arguments = Vec::new();
    for argument in &segment.arguments {
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        let TypeArgument::Exact(argument) = argument else {
            return Ok(DependencyResult::Unknown);
        };
        arguments.push(argument.clone());
    }
    Ok(DependencyResult::Found((header, arguments)))
}

/// Instantiate the selected field declaration in the receiver's declared generic environment.
/// It remains a type fact; the caller must check its actual stores and uses before publication.
pub fn declared_generic_field_type<H: RecoveryHeaders>(
    source: &jarde_reader::signature::ClassType,
    initiating_loader: &LoaderId,
    member: &jarde_jvm::resolver::ResolvedMemberRef,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<jarde_reader::signature::SignatureType>, H::Error> {
    let jarde_reader::model::SymbolRef::Field {
        name, descriptor, ..
    } = &member.member
    else {
        return Ok(DependencyResult::Unknown);
    };
    let (header, arguments) =
        match declared_member_application(source, initiating_loader, member, headers, budget)? {
            DependencyResult::Found(found) => found,
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        };
    let mut matching = header
        .fields
        .iter()
        .enumerate()
        .filter(|(_, field)| field.name.raw() == name && field.descriptor.raw() == descriptor);
    let Some((index, _)) = matching.next() else {
        return Ok(DependencyResult::Unknown);
    };
    if matching.next().is_some() {
        return Ok(DependencyResult::Unknown);
    }
    let Some(signatures) = &header.generic_signatures else {
        return Ok(DependencyResult::Unknown);
    };
    let Some(field) = signatures.fields.get(index) else {
        return Ok(DependencyResult::Unknown);
    };
    let source = if let Some(field) = field {
        field.ty.clone()
    } else {
        let mut wrapped = b"()".to_vec();
        wrapped.extend_from_slice(&descriptor.0);
        let parsed = jarde_reader::signature::parse_method_signature(&wrapped, budget)?;
        let Some(result) = parsed.result else {
            return Ok(DependencyResult::Unknown);
        };
        result
    };
    let parameters = signatures
        .class
        .as_ref()
        .map_or(&[][..], |(class, _proof)| class.type_parameters.as_slice());
    Ok(
        crate::generic_source_proof::substitute_signature_type_parameters(
            &source, parameters, &arguments, budget, 0,
        )?
        .map_or(DependencyResult::Unknown, DependencyResult::Found),
    )
}

/// Instantiate a resolved method's class binders. Method-level inference and capture are outside
/// this path; their existing proofs remain separate rather than confusing two lexical binders.
pub fn declared_generic_method_signature<H: RecoveryHeaders>(
    source: &jarde_reader::signature::ClassType,
    initiating_loader: &LoaderId,
    member: &jarde_jvm::resolver::ResolvedMemberRef,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<jarde_reader::signature::MethodSignature>, H::Error> {
    let jarde_reader::model::SymbolRef::Method {
        name, descriptor, ..
    } = &member.member
    else {
        return Ok(DependencyResult::Unknown);
    };
    let (header, arguments) =
        match declared_member_application(source, initiating_loader, member, headers, budget)? {
            DependencyResult::Found(found) => found,
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        };
    let mut matching =
        header.methods.iter().enumerate().filter(|(_, method)| {
            method.name.raw() == name && method.descriptor.raw() == descriptor
        });
    let Some((index, _)) = matching.next() else {
        return Ok(DependencyResult::Unknown);
    };
    if matching.next().is_some() {
        return Ok(DependencyResult::Unknown);
    }
    let Some(signatures) = &header.generic_signatures else {
        return Ok(DependencyResult::Unknown);
    };
    let Some(method) = signatures.methods.get(index) else {
        return Ok(DependencyResult::Unknown);
    };
    let method = match method {
        Some((method, _proof)) => method.clone(),
        None => jarde_reader::signature::parse_method_signature(&descriptor.0, budget)?,
    };
    if !method.type_parameters.is_empty() {
        return Ok(DependencyResult::Unknown);
    }
    let parameters = signatures
        .class
        .as_ref()
        .map_or(&[][..], |(class, _proof)| class.type_parameters.as_slice());
    let mut instantiated = jarde_reader::signature::MethodSignature {
        type_parameters: Vec::new(),
        parameters: Vec::new(),
        result: None,
        throws: Vec::new(),
    };
    for source in &method.parameters {
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        let Some(ty) = crate::generic_source_proof::substitute_signature_type_parameters(
            source, parameters, &arguments, budget, 0,
        )?
        else {
            return Ok(DependencyResult::Unknown);
        };
        instantiated.parameters.push(ty);
    }
    if let Some(source) = &method.result {
        let Some(ty) = crate::generic_source_proof::substitute_signature_type_parameters(
            source, parameters, &arguments, budget, 0,
        )?
        else {
            return Ok(DependencyResult::Unknown);
        };
        instantiated.result = Some(ty);
    }
    for source in &method.throws {
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        let Some(ty) = crate::generic_source_proof::substitute_signature_type_parameters(
            source, parameters, &arguments, budget, 0,
        )?
        else {
            return Ok(DependencyResult::Unknown);
        };
        instantiated.throws.push(ty);
    }
    Ok(DependencyResult::Found(instantiated))
}

#[derive(Clone)]
pub struct ReferenceClassHeader {
    pub loader: LoaderId,
    pub definition: PhysicalDefinitionId,
    pub flags: u16,
    pub super_name: Option<Vec<u8>>,
    pub interfaces: Vec<Vec<u8>>,
    pub methods: Vec<MemberHeader>,
    pub fields: Vec<MemberHeader>,
    /// Signature facts are read only for a declared-generic demand. `None` means they were not
    /// established; a present bundle with no class Signature states actual attribute absence.
    pub generic_signatures: Option<std::sync::Arc<GenericHeaderSignatures>>,
}

/// Parsed, erasure-checked signatures beside the selected header's physical member order.
/// These are declaration facts; they authorize no source projection or invocation by themselves.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericHeaderSignatures {
    pub class: Option<(
        jarde_reader::signature::ClassSignature,
        jarde_reader::signature::ClassSignatureErasureProof,
    )>,
    pub fields: Vec<Option<jarde_reader::signature::FieldSignature>>,
    pub methods: Vec<
        Option<(
            jarde_reader::signature::MethodSignature,
            jarde_reader::signature::MethodSignatureErasureProof,
        )>,
    >,
}

impl ReferenceClassHeader {
    /// A complete header can prove absence without reopening its bytes. Only present Signature
    /// shells require a metadata read; None continues to mean signature facts are unstated.
    fn signature_absence(
        attributes: &[AttributeShell],
        fields: &[MemberHeader],
        methods: &[MemberHeader],
    ) -> Option<std::sync::Arc<GenericHeaderSignatures>> {
        if attributes
            .iter()
            .chain(fields.iter().flat_map(|field| &field.attributes))
            .chain(methods.iter().flat_map(|method| &method.attributes))
            .any(|attribute| attribute.name.raw().0 == b"Signature")
        {
            return None;
        }
        Some(std::sync::Arc::new(GenericHeaderSignatures {
            class: None,
            fields: vec![None; fields.len()],
            methods: vec![None; methods.len()],
        }))
    }

    pub fn from_facts(
        facts: &ClassMemberFacts,
        loader: LoaderId,
        definition: PhysicalDefinitionId,
    ) -> Option<Self> {
        if facts.stopped_at.is_some()
            || facts.method_count != u64::try_from(facts.methods.len()).ok()?
            || facts.field_count != u64::try_from(facts.fields.len()).ok()?
        {
            return None;
        }
        Some(Self {
            loader,
            definition,
            flags: facts.access_flags,
            super_name: facts.super_class.as_ref().map(|name| name.raw().0.clone()),
            interfaces: facts
                .interfaces
                .iter()
                .map(|name| name.raw().0.clone())
                .collect(),
            methods: facts.methods.clone(),
            fields: facts.fields.clone(),
            generic_signatures: Self::signature_absence(
                &facts.attributes,
                &facts.fields,
                &facts.methods,
            ),
        })
    }

    pub fn from_ir(ir: &jarde_jvm::method_ir::MethodIr, loader: LoaderId) -> Option<Self> {
        Some(Self {
            loader,
            definition: ir.declaration()?.identity().owner.clone(),
            flags: ir.declaration()?.class_access_flags(),
            super_name: ir.direct_super_class().map(|name| name.0.clone()),
            interfaces: ir
                .direct_interfaces()
                .iter()
                .map(|name| name.raw().0.clone())
                .collect(),
            methods: ir.class_methods()?.to_vec(),
            fields: ir.class_fields()?.to_vec(),
            generic_signatures: ir.class_attributes().and_then(|attributes| {
                Self::signature_absence(attributes, ir.class_fields()?, ir.class_methods()?)
            }),
        })
    }

    /// Read only Signature and its erasure evidence from this header's own selected bytes.
    /// The caller retains definition/loader selection and shares the resulting bundle on clones.
    pub fn with_generic_signatures(
        mut self,
        bytes: &[u8],
        attributes: &[jarde_reader::classfile::AttributeShell],
        pool: &[jarde_reader::classfile::CpEntryFacts],
        budget: &mut Budget,
    ) -> Result<Self> {
        use jarde_reader::signature::{
            parse_class_signature, parse_field_signature, parse_method_signature,
            prove_class_signature_erasure, prove_field_signature_erasure_with_class_scope,
            prove_method_signature_erasure_with_class_scope,
        };
        let signature_attributes = |attributes: &[jarde_reader::classfile::AttributeShell],
                                    method: bool,
                                    budget: &mut Budget| {
            let mut selected = Vec::new();
            for attribute in attributes {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                if attribute.name.raw().0 == b"Signature"
                    || (method && attribute.name.raw().0 == b"Exceptions")
                {
                    budget.charge(CountedBudgetDimension::IrItems, 1)?;
                    selected.push(attribute.clone());
                }
            }
            jarde_reader::classfile::attribute_facts(bytes, &selected, pool, budget)
        };
        let raw_class = signature_attributes(attributes, false, budget)?.signature;
        let class = raw_class
            .as_ref()
            .map(|raw| parse_class_signature(&raw.0, budget))
            .transpose()?;
        let class = match class {
            Some(signature) => {
                let proof = prove_class_signature_erasure(
                    &signature,
                    self.super_name.as_deref().unwrap_or_default(),
                    &self.interfaces,
                    budget,
                )?;
                Some((signature, proof))
            }
            None => None,
        };
        let scope = class
            .as_ref()
            .map_or(&[][..], |(_, proof)| proof.type_parameters.as_slice());
        let mut fields = Vec::new();
        for field in &self.fields {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            let raw = signature_attributes(&field.attributes, false, budget)?.signature;
            let parsed = raw
                .as_ref()
                .map(|raw| parse_field_signature(&raw.0, budget))
                .transpose()?;
            if let Some(signature) = &parsed {
                prove_field_signature_erasure_with_class_scope(
                    signature,
                    &field.descriptor.raw().0,
                    scope,
                    budget,
                )?;
            }
            fields.push(parsed);
        }
        let mut methods = Vec::new();
        for method in &self.methods {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            let facts = signature_attributes(&method.attributes, true, budget)?;
            let parsed = facts
                .signature
                .as_ref()
                .map(|raw| parse_method_signature(&raw.0, budget))
                .transpose()?;
            let proved = if let Some(signature) = parsed {
                budget.charge(
                    CountedBudgetDimension::IrItems,
                    facts.exceptions.len() as u64,
                )?;
                let exceptions: Vec<_> =
                    facts.exceptions.iter().map(|name| name.0.clone()).collect();
                let proof = prove_method_signature_erasure_with_class_scope(
                    &signature,
                    &method.descriptor.raw().0,
                    &exceptions,
                    scope,
                    budget,
                )?;
                Some((signature, proof))
            } else {
                None
            };
            methods.push(proved);
        }
        self.generic_signatures = Some(std::sync::Arc::new(GenericHeaderSignatures {
            class,
            fields,
            methods,
        }));
        Ok(self)
    }
}
pub struct SnapshotReferenceShape {
    /// The full Java spelling: a class name or that class name followed by its array rank.
    pub source_type: String,
    /// The class whose selected header carries the hierarchy edge.
    pub leaf_class: Vec<u8>,
    /// Zero for a scalar class, otherwise the number of reference-array dimensions.
    pub rank: u32,
}
pub struct SelectedInterfaceNode {
    pub loader: LoaderId,
    pub definition: PhysicalDefinitionId,
    pub facts: ClassMemberFacts,
}
pub const SNAPSHOT_HIERARCHY_WALK_DEPTH: u64 = 8;

/// A single relation walk supplies ordinary calls, local declarations and array stores. Real
/// definitions take precedence at every node; only a clean Missing may use a registered platform
/// parent contract. The returned uncertainty is retained instead of being called incompatibility.
fn reference_relation<H: RecoveryHeaders>(
    request: &MethodAnalysisRequest,
    source: &[u8],
    target: &[u8],
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<crate::report::ReferenceRelation, H::Error> {
    use crate::report::ReferenceRelation as Relation;
    let runtime = &request.environment.runtime;
    let platform_allowed = matches!(runtime.profile.java_release, 8 | 23)
        && runtime.load_domain.delegation == DelegationPolicy::ParentFirst
        && runtime.load_domain.module_mode == ModuleMode::ClassPath
        && runtime.load_domain.external_override == RuntimeUncertainty::None
        && runtime.load_domain.runtime_transformation == RuntimeUncertainty::None;
    // A Missing lookup is usable as a platform contract only under the complete initiating
    // loader search chain. The selected source may have a different defining loader from the
    // caller; its ancestors must not inherit the caller's cleaner uncertainty declaration.
    let platform_search_is_closed = |initiating: &LoaderId, budget: &mut Budget| -> Result<bool> {
        let mut loader = initiating;
        let mut visited = std::collections::BTreeSet::new();
        loop {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if !visited.insert(loader.0.as_str()) {
                return Ok(false);
            }
            let domain = if loader == &runtime.load_domain.loader {
                Some(&runtime.load_domain)
            } else {
                let mut selected = None;
                for domain in &request.environment.domains {
                    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                    if &domain.loader == loader {
                        selected = Some(domain);
                        break;
                    }
                }
                selected
            };
            let Some(domain) = domain else {
                return Ok(false);
            };
            if domain.delegation != DelegationPolicy::ParentFirst
                || domain.module_mode != ModuleMode::ClassPath
                || domain.external_override != RuntimeUncertainty::None
                || domain.runtime_transformation != RuntimeUncertainty::None
            {
                return Ok(false);
            }
            match &domain.parent_loader {
                Some(parent) => loader = parent,
                None => return Ok(true),
            }
        }
    };
    let caller = headers.caller_loader().clone();
    // The exit entry makes the active path distinct from a previously completed shared parent.
    // A cycle is Unknown, rather than a negative fact cached as an absent relation.
    let mut pending = vec![(caller.clone(), source.to_vec(), 0_u64, false, false)];
    let mut states = std::collections::BTreeMap::new();
    let (mut missing, mut ambiguous, mut unknown) = (false, false, false);
    while let Some((loader, name, depth, platform, exiting)) = pending.pop() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        budget.observe_dependency_depth(depth)?;
        let key = (loader.0.clone(), name.clone(), platform);
        if exiting {
            states.insert(key, false);
            continue;
        }
        if let Some(active) = states.get(&key) {
            unknown |= *active;
            continue;
        }
        states.insert(key, true);
        pending.push((loader.clone(), name.clone(), depth, platform, true));
        match headers.selected_from(&loader, &name, budget)? {
            DependencyResult::Found(header) => {
                // An unprovided platform definition has no physical defining-loader identity to
                // equate with an explicitly supplied replacement later on its parent path.
                if platform {
                    unknown = true;
                    continue;
                }
                if name == target {
                    if loader == caller {
                        return Ok(Relation::SelectedHeaders);
                    }
                    match headers.selected(target, budget)? {
                        DependencyResult::Found(expected)
                            if expected.loader == header.loader
                                && expected.definition == header.definition =>
                        {
                            return Ok(Relation::SelectedHeaders);
                        }
                        DependencyResult::Found(_) => {}
                        DependencyResult::Missing => missing = true,
                        DependencyResult::Ambiguous => ambiguous = true,
                        DependencyResult::Unknown => unknown = true,
                    }
                    continue;
                }
                if depth >= SNAPSHOT_HIERARCHY_WALK_DEPTH {
                    unknown = true;
                    continue;
                }
                let mut direct_target = None;
                for parent in header.super_name.into_iter().chain(header.interfaces) {
                    budget.poll()?;
                    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                    if parent == target {
                        // This selected physical header states the edge in the caller's own
                        // resolution namespace. Both uses of the target therefore denote the
                        // same class without needing its member header. A different defining
                        // loader still follows the edge and compares the selected definitions.
                        if header.loader == caller {
                            return Ok(Relation::SelectedHeaders);
                        }
                        direct_target = Some(parent);
                    } else {
                        pending.push((header.loader.clone(), parent, depth + 1, false, false));
                    }
                }
                if let Some(parent) = direct_target {
                    pending.push((header.loader.clone(), parent, depth + 1, false, false));
                }
            }
            DependencyResult::Missing => {
                // The closed platform rows answer a search that leaves the runtime's own domain:
                // the presented type's own absence, a chain the header walk delegated to another
                // declared domain, or a parent the rows themselves stated. A chain that never
                // leaves the runtime domain's own namespace keeps that namespace as its only
                // authority — a name it cannot resolve stays Missing (the single-sided platform
                // refusal stands).
                let consult_platform = platform_allowed
                    && (platform || depth == 0 || &loader != &runtime.load_domain.loader);
                let parents = if consult_platform {
                    if !platform_search_is_closed(&loader, budget)? {
                        unknown = true;
                        continue;
                    }
                    crate::assignability::platform_parents(runtime.profile.java_release, &name)
                } else {
                    None
                };
                let Some(parents) = parents else {
                    missing = true;
                    continue;
                };
                if name == target {
                    if loader == caller {
                        return Ok(Relation::PlatformContract {
                            java_release: runtime.profile.java_release,
                        });
                    }
                    match headers.selected(target, budget)? {
                        DependencyResult::Missing => {
                            return Ok(Relation::PlatformContract {
                                java_release: runtime.profile.java_release,
                            });
                        }
                        DependencyResult::Found(_) | DependencyResult::Unknown => unknown = true,
                        DependencyResult::Ambiguous => ambiguous = true,
                    }
                    continue;
                }
                if depth >= SNAPSHOT_HIERARCHY_WALK_DEPTH {
                    unknown = true;
                    continue;
                }
                let mut direct_target = None;
                for parent in parents {
                    budget.poll()?;
                    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                    if parent.as_bytes() == target {
                        direct_target = Some(parent);
                    } else {
                        pending.push((
                            loader.clone(),
                            parent.as_bytes().to_vec(),
                            depth + 1,
                            true,
                            false,
                        ));
                    }
                }
                if let Some(parent) = direct_target {
                    pending.push((
                        loader.clone(),
                        parent.as_bytes().to_vec(),
                        depth + 1,
                        true,
                        false,
                    ));
                }
            }
            DependencyResult::Ambiguous => ambiguous = true,
            DependencyResult::Unknown => unknown = true,
        }
    }
    Ok(if ambiguous {
        Relation::Ambiguous
    } else if unknown {
        Relation::Unknown
    } else if missing {
        Relation::Missing
    } else {
        Relation::Incompatible
    })
}

/// A single reference parameter is the only overload form proved in this slice. Its exact
/// descriptor leaves no varargs or boxing phase to interpret at the call site.
pub fn single_reference_parameter(descriptor: &[u8]) -> Option<&[u8]> {
    let tail = descriptor.strip_prefix(b"(L")?;
    let end = tail.iter().position(|byte| *byte == b';')?;
    (tail.get(end + 1) == Some(&b')') && tail.len() > end + 2).then_some(&tail[..end])
}

pub fn proved_reference_widening<H: RecoveryHeaders>(
    request: &MethodAnalysisRequest,
    source: &[u8],
    target: &[u8],
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<bool, H::Error> {
    if source == target {
        return Ok(false);
    }
    Ok(
        reference_relation(request, source, target, headers, budget)?
            .is_assignable(request.environment.runtime.profile.java_release),
    )
}

/// Follow each superclass/interface name from the loader that defined its declaring header.
/// The initial target remains relative to the caller. A cross-loader edge reaches that target
/// only when both searches select the same (defining loader, physical definition); names or
/// identical bytes alone do not establish a runtime type relation.
pub fn snapshot_header_chain_widens_with(
    source: &[u8],
    target: &[u8],
    initiating_loader: LoaderId,
    max_depth: u64,
    budget: &mut Budget,
    header: impl FnMut(&LoaderId, &[u8], &mut Budget) -> Result<Option<ReferenceClassHeader>>,
) -> Result<bool> {
    snapshot_header_chain_widens_try(source, target, initiating_loader, max_depth, budget, header)
}

pub fn snapshot_header_chain_widens_try<E: From<Error>>(
    source: &[u8],
    target: &[u8],
    initiating_loader: LoaderId,
    max_depth: u64,
    budget: &mut Budget,
    mut header: impl FnMut(
        &LoaderId,
        &[u8],
        &mut Budget,
    ) -> std::result::Result<Option<ReferenceClassHeader>, E>,
) -> std::result::Result<bool, E> {
    if source == target {
        return Ok(false);
    }
    let mut pending = vec![(initiating_loader.clone(), source.to_vec(), 0_u64)];
    let mut visited = std::collections::BTreeSet::new();
    while let Some((loader, name, depth)) = pending.pop() {
        budget.poll()?;
        budget.observe_dependency_depth(depth)?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !visited.insert((loader.0.clone(), name.clone())) {
            continue;
        }
        if name == target {
            if loader == initiating_loader {
                return Ok(true);
            }
            if let (Some(reached), Some(expected)) = (
                header(&loader, target, budget)?,
                header(&initiating_loader, target, budget)?,
            ) && reached.loader == expected.loader
                && reached.definition == expected.definition
            {
                return Ok(true);
            }
            continue;
        }
        let Some(facts) = header(&loader, &name, budget)? else {
            continue;
        };
        if depth >= max_depth {
            continue;
        }
        if let Some(parent) = facts.super_name {
            if parent != b"java/lang/Object" || target == b"java/lang/Object" {
                pending.push((facts.loader.clone(), parent, depth.saturating_add(1)));
            }
        }
        pending.extend(
            facts
                .interfaces
                .into_iter()
                .map(|name| (facts.loader.clone(), name, depth.saturating_add(1))),
        );
    }
    Ok(false)
}

/// The selected physical endpoints of an erased upcast. The shared raw-header walk still owns
/// hierarchy traversal; name equality alone never equates classes from two defining loaders.
pub fn selected_header_widens<H: RecoveryHeaders>(
    source: &jarde_jvm::resolver::ResolvedMemberRef,
    target: &jarde_jvm::resolver::ResolvedMemberRef,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<bool, H::Error> {
    use jarde_reader::model::SymbolRef;
    budget.poll()?;
    let (SymbolRef::Class { owner: source_name }, SymbolRef::Class { owner: target_name }) =
        (&source.member, &target.member)
    else {
        return Ok(false);
    };
    if source == target {
        return Ok(true);
    }
    if source_name == target_name {
        return Ok(false);
    }
    for (name, expected) in [(&source_name.0, source), (&target_name.0, target)] {
        let DependencyResult::Found(header) =
            headers.selected_from(&source.loader, name, budget)?
        else {
            return Ok(false);
        };
        if header.loader != expected.loader || header.definition != expected.definition {
            return Ok(false);
        }
    }
    let max_depth = budget.limits().dependency_depth;
    snapshot_header_chain_widens_try(
        &source_name.0,
        &target_name.0,
        source.loader.clone(),
        max_depth,
        budget,
        |loader, name, budget| Ok(headers.selected_from(loader, name, budget)?.found()),
    )
}

/// Compare a name stated by a selected declaration with the caller's spelling of that name.
fn same_caller_type<H: RecoveryHeaders>(
    loader: &LoaderId,
    name: &[u8],
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<bool, H::Error> {
    if loader == headers.caller_loader() {
        return Ok(true);
    }
    let Some(reached) = headers.selected_from(loader, name, budget)?.found() else {
        return Ok(false);
    };
    let Some(expected) = headers.selected(name, budget)?.found() else {
        return Ok(false);
    };
    Ok(reached.loader == expected.loader && reached.definition == expected.definition)
}

/// The class-source path alone may relax the field rule's exact receiver-owner match. Each
/// certificate names one instruction and one declaration in the selected direct parent of the
/// actual receiver. The recovery layer rechecks that same SSA receiver before emitting a cast.
pub fn prove_direct_parent_field_writes<H: RecoveryHeaders>(
    ir: &jarde_jvm::method_ir::MethodIr,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<Vec<jarde_java::report::SuperclassFieldWriteInput>, H::Error> {
    use jarde_reader::classfile::cp_entry;
    let (Some(code), Some(declaration), Some(ssa)) = (ir.code(), ir.declaration(), ir.ssa())
    else {
        return Ok(Vec::new());
    };
    let site_index = SsaSiteIndex::build(ssa, budget)?;

    let mut proved = Vec::new();
    for instruction in &code.instructions {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if instruction.opcode != 0xb5 {
            continue;
        }
        let Some(index) = instruction.constant_pool_index else {
            continue;
        };
        let Ok(entry) = cp_entry(ir.constant_pool(), index) else {
            continue;
        };
        let CpEntryKind::FieldRef {
            owner,
            name,
            descriptor,
            ..
        } = &entry.kind
        else {
            continue;
        };
        let Some(source) = named_stack_receiver(&site_index, instruction.bci, budget)? else {
            continue;
        };
        if source == owner.0 || !selected_direct_parent(source, &owner.0, headers, budget)? {
            continue;
        }
        let DependencyResult::Found(header) = headers.selected(&owner.0, budget)? else {
            continue;
        };
        if header.flags & ACC_INTERFACE != 0 {
            continue;
        }
        if package_name(&declaration.class_name().0) != package_name(&owner.0)
            && header.flags & 0x0001 == 0
        {
            continue;
        }
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(header.fields.len()).unwrap_or(u64::MAX),
        )?;
        let matching: Vec<_> = header
            .fields
            .iter()
            .filter(|field| field.name.raw().0 == name.0)
            .collect();
        if matching.len() != 1 || matching[0].descriptor.raw().0 != descriptor.0 {
            continue;
        }
        let flags = matching[0].access_flags;
        let existing_public_instance = flags & (0x0001 | 0x0008) == 0x0001;
        let same_package_non_public_instance = {
            const ACC_PROTECTED: u16 = 0x0004;

            package_name(&declaration.class_name().0) == package_name(&owner.0)
                && (flags == 0 || flags == ACC_PROTECTED)
        };
        if !existing_public_instance && !same_package_non_public_instance {
            continue;
        }

        proved.push(jarde_java::report::SuperclassFieldWriteInput {
            bci: instruction.bci,
            source: JvmBytes(source.to_owned()),
            owner: owner.clone(),
            name: name.clone(),
            descriptor: descriptor.clone(),
        });
    }
    Ok(proved)
}

/// Only the exact static synthetic setter call selected on the direct parent receives the
/// otherwise-unavailable B-to-A source cast. Its body is recovered independently as physical Code.
pub fn prove_direct_parent_accessor_calls<H: RecoveryHeaders>(
    ir: &jarde_jvm::method_ir::MethodIr,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<Vec<jarde_java::report::ReferenceOverloadCallInput>, H::Error> {
    use jarde_reader::classfile::cp_entry;
    let (Some(code), Some(declaration), Some(ssa)) = (ir.code(), ir.declaration(), ir.ssa())
    else {
        return Ok(Vec::new());
    };
    let site_index = SsaSiteIndex::build(ssa, budget)?;

    let mut proved = Vec::new();
    for (position, instruction) in code.instructions.iter().enumerate() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if instruction.opcode != 0xb8
            || code
                .instructions
                .get(position + 1)
                .is_none_or(|next| next.opcode != 0x57)
        {
            continue;
        }
        let Some(index) = instruction.constant_pool_index else {
            continue;
        };
        let Ok(entry) = cp_entry(ir.constant_pool(), index) else {
            continue;
        };
        let CpEntryKind::MethodRef {
            owner,
            name,
            descriptor,
            ..
        } = &entry.kind
        else {
            continue;
        };
        if !name.0.starts_with(b"access$") {
            continue;
        }
        let Some(source) = named_stack_receiver(&site_index, instruction.bci, budget)? else {
            continue;
        };
        if source == owner.0 || !selected_direct_parent(source, &owner.0, headers, budget)? {
            continue;
        }
        let mut expected = b"(L".to_vec();
        expected.extend_from_slice(&owner.0);
        expected.extend_from_slice(b";Z)Z");
        if descriptor.0 != expected {
            continue;
        }
        let DependencyResult::Found(header) = headers.selected(&owner.0, budget)? else {
            continue;
        };
        if package_name(&declaration.class_name().0) != package_name(&owner.0)
            && header.flags & 0x0001 == 0
        {
            continue;
        }
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(header.methods.len()).unwrap_or(u64::MAX),
        )?;
        let matching: Vec<_> = header
            .methods
            .iter()
            .filter(|method| method.name.raw().0 == name.0)
            .collect();
        if matching.len() != 1
            || matching[0].descriptor.raw().0 != descriptor.0
            || matching[0].access_flags & (0x0008 | 0x1000) != (0x0008 | 0x1000)
            || matching[0].access_flags & 0x0002 != 0
            || (package_name(&declaration.class_name().0) != package_name(&owner.0)
                && matching[0].access_flags & 0x0001 == 0)
        {
            continue;
        }
        let (Some(source), Some(target)) = (
            source_class_name(&JvmBytes(source.to_vec())),
            source_class_name(&owner),
        ) else {
            continue;
        };
        proved.push(jarde_java::report::ReferenceOverloadCallInput {
            bci: instruction.bci,
            source,
            target,
        });
    }
    Ok(proved)
}

/// One SSA instruction per bytecode index of one method, built once per proof pass and answered
/// by binary search: the pass pays one charged scan at build time, and every per-BCI lookup pays
/// only its own comparison count — no lookup re-scans the table.
pub(crate) struct SsaSiteIndex<'a> {
    ssa: &'a jarde_jvm::method_ir::SsaTable,
    entries: Vec<(u32, &'a jarde_jvm::method_ir::SsaInstruction)>,
}

impl<'a> SsaSiteIndex<'a> {
    /// One charged scan of the SSA table. A bytecode index names one instruction, so the sorted
    /// vector answers every per-BCI lookup this pass makes.
    fn build(ssa: &'a jarde_jvm::method_ir::SsaTable, budget: &mut Budget) -> Result<Self> {
        let mut entries = Vec::new();
        for block in ssa.blocks() {
            for site in block.instructions() {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                entries.push((site.bci(), site));
            }
        }
        entries.sort_unstable_by_key(|(bci, _)| *bci);
        Ok(Self { ssa, entries })
    }

    /// The instruction at one bytecode index, charged with the comparisons its binary search
    /// performs.
    fn site(
        &self,
        bci: u32,
        budget: &mut Budget,
    ) -> Result<Option<&'a jarde_jvm::method_ir::SsaInstruction>> {
        budget.poll()?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            (self.entries.len() as u64 + 1)
                .next_power_of_two()
                .trailing_zeros() as u64,
        )?;
        Ok(self
            .entries
            .binary_search_by_key(&bci, |(key, _)| *key)
            .ok()
            .map(|at| self.entries[at].1))
    }
}

/// The lowest consumed stack slot is the physical receiver or first static argument.
/// Names are frame facts, and the lookup is charged for its own comparisons and reads.
pub(crate) fn named_stack_receiver<'a>(
    index: &SsaSiteIndex<'a>,
    bci: u32,
    budget: &mut Budget,
) -> Result<Option<&'a [u8]>> {
    let Some(site) = index.site(bci, budget)? else {
        return Ok(None);
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(site.reads().len()).unwrap_or(u64::MAX),
    )?;
    let first = site
        .reads()
        .iter()
        .filter_map(|(slot, value)| match slot {
            jarde_jvm::method_ir::Slot::Stack(depth) => Some((*depth, *value)),
            jarde_jvm::method_ir::Slot::Local(_) => None,
        })
        .min_by_key(|(depth, _)| *depth);
    Ok(first.and_then(|(_, value)| match index.ssa.value(value).ty() {
        jarde_jvm::method_ir::Value::Ref(jarde_jvm::method_ir::RefType::Named { name, .. }) => {
            Some(
                name.strip_prefix(b"L")
                    .and_then(|name| name.strip_suffix(b";"))
                    .unwrap_or(name),
            )
        }
        _ => None,
    }))
}

/// The cast is confined to one selected, direct B -> A edge. A missing or ambiguous
/// dependency supplies no source-level relation.
pub fn selected_direct_parent<H: RecoveryHeaders>(
    source: &[u8],
    owner: &[u8],
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<bool, H::Error> {
    let Some(header) = headers.selected(source, budget)?.found() else {
        return Ok(false);
    };
    if header.flags & ACC_INTERFACE != 0 || header.super_name.as_deref() != Some(owner) {
        return Ok(false);
    }
    same_caller_type(&header.loader, owner, headers, budget)
}

/// A cast to the exact declared one-argument type chooses that declaration in Java 8's strict
/// invocation phase. Require one visible declaration on a complete selected class chain; a
/// same-parameter bridge, an unseen parent or an interface graph remains unproved.
pub fn exact_reference_overload_count(
    methods: &[MemberHeader],
    name: &[u8],
    descriptor: &[u8],
    target: &[u8],
    is_static: bool,
    budget: &mut Budget,
) -> Result<Option<u32>> {
    let mut exact = 0_u32;
    for method in methods {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if method.name.raw().0 != name {
            continue;
        }
        let flags = method.access_flags;
        if flags & (0x0040 | 0x0080 | 0x1000) != 0 {
            return Ok(None);
        }
        if single_reference_parameter(&method.descriptor.raw().0) == Some(target) {
            if method.descriptor.raw().0 != descriptor
                || flags & 0x0001 == 0
                || (flags & 0x0008 != 0) != is_static
            {
                return Ok(None);
            }
            exact += 1;
        }
    }
    Ok(Some(exact))
}

pub fn unique_reference_overload_target<H: RecoveryHeaders>(
    owner: &[u8],
    name: &[u8],
    descriptor: &[u8],
    target: &[u8],
    is_static: bool,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<bool, H::Error> {
    let mut current = owner.to_vec();
    let mut loader = headers.caller_loader().clone();
    let mut visited = std::collections::BTreeSet::new();
    let mut exact = 0_u32;
    let mut depth = 0_u64;
    loop {
        budget.poll()?;
        budget.observe_dependency_depth(depth)?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !visited.insert((loader.0.clone(), current.clone())) || current == b"java/lang/Object" {
            return Ok(false);
        }
        let DependencyResult::Found(header) = headers.selected_from(&loader, &current, budget)?
        else {
            return Ok(false);
        };
        if header.flags & ACC_INTERFACE != 0 || !header.interfaces.is_empty() {
            return Ok(false);
        }
        let Some(count) = exact_reference_overload_count(
            &header.methods,
            name,
            descriptor,
            target,
            is_static,
            budget,
        )?
        else {
            return Ok(false);
        };
        if count != 0 && !same_caller_type(&header.loader, target, headers, budget)? {
            return Ok(false);
        }
        exact += count;
        match header.super_name {
            Some(parent) if parent == b"java/lang/Object" => return Ok(exact == 1),
            Some(parent) => {
                loader = header.loader;
                current = parent;
                depth = depth.saturating_add(1);
            }
            None => return Ok(false),
        }
    }
}

/// A private static call in the declaring class can be fixed by its exact
/// descriptor even when another overload takes the more specific source type.
/// The explicit A cast makes only overloads applicable to A relevant.
pub fn unique_private_reference_target(
    ir: &jarde_jvm::method_ir::MethodIr,
    owner: &[u8],
    name: &[u8],
    descriptor: &[u8],
    target: &[u8],
    is_static: bool,
    budget: &mut Budget,
) -> Result<bool> {
    if !is_static
        || ir
            .declaration()
            .is_none_or(|decl| decl.class_name().0 != owner)
        || ir
            .direct_super_class()
            .is_none_or(|parent| parent.0 != b"java/lang/Object")
        || !ir.direct_interfaces().is_empty()
    {
        return Ok(false);
    }
    let Some(methods) = ir.class_methods() else {
        return Ok(false);
    };
    exact_private_reference_target(methods, name, descriptor, target, budget)
}

pub fn exact_private_reference_target(
    methods: &[MemberHeader],
    name: &[u8],
    descriptor: &[u8],
    target: &[u8],
    budget: &mut Budget,
) -> Result<bool> {
    let mut exact = 0_u32;
    for method in methods {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if method.name.raw().0 != name
            || single_reference_parameter(&method.descriptor.raw().0) != Some(target)
        {
            continue;
        }
        let flags = method.access_flags;
        if method.descriptor.raw().0 != descriptor
            || flags & (0x0002 | 0x0008) != (0x0002 | 0x0008)
            || flags & (0x0040 | 0x0080 | 0x1000) != 0
        {
            return Ok(false);
        }
        exact += 1;
    }
    Ok(exact == 1)
}

pub fn prove_reference_overload_calls<H: RecoveryHeaders>(
    request: &MethodAnalysisRequest,
    ir: &jarde_jvm::method_ir::MethodIr,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<Vec<jarde_java::report::ReferenceOverloadCallInput>, H::Error> {
    use jarde_reader::classfile::{cp_class_name, cp_entry};
    let Some(code) = ir.code() else {
        return Ok(Vec::new());
    };
    let mut sources = std::collections::BTreeSet::new();
    for instruction in &code.instructions {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if instruction.opcode == 0xbb
            && let Some(index) = instruction.constant_pool_index
            && let Ok(name) = cp_class_name(ir.constant_pool(), index)
        {
            sources.insert(name.0);
        }
    }
    if sources.is_empty() {
        return Ok(Vec::new());
    }
    let mut proved = Vec::new();
    for instruction in &code.instructions {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let is_static = instruction.opcode == 0xb8;
        if !is_static && instruction.opcode != 0xb6 {
            continue;
        }
        let Some(index) = instruction.constant_pool_index else {
            continue;
        };
        let Ok(entry) = cp_entry(ir.constant_pool(), index) else {
            continue;
        };
        let CpEntryKind::MethodRef {
            owner,
            name,
            descriptor,
            ..
        } = &entry.kind
        else {
            continue;
        };
        let Some(target) = single_reference_parameter(&descriptor.0) else {
            continue;
        };
        if !unique_reference_overload_target(
            &owner.0,
            &name.0,
            &descriptor.0,
            target,
            is_static,
            headers,
            budget,
        )? && !unique_private_reference_target(
            ir,
            &owner.0,
            &name.0,
            &descriptor.0,
            target,
            is_static,
            budget,
        )? {
            continue;
        }
        for source in &sources {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if proved_reference_widening(request, source, target, headers, budget)? {
                let (Some(source), Some(target)) = (
                    source_class_name(&JvmBytes(source.to_vec())),
                    source_class_name(&JvmBytes(target.to_vec())),
                ) else {
                    continue;
                };
                proved.push(jarde_java::report::ReferenceOverloadCallInput {
                    bci: instruction.bci,
                    source,
                    target,
                });
            }
        }
    }
    Ok(proved)
}

pub fn reference_parameter_sites_checked<'a>(
    descriptor: &'a [u8],
    receiver_slots: usize,
    budget: &mut Budget,
) -> Result<Option<Vec<(usize, &'a [u8])>>> {
    let Ok(facts) = jarde_reader::classfile::descriptor_facts(
        descriptor,
        jarde_reader::classfile::DescriptorKind::Method,
    ) else {
        return Ok(None);
    };
    let mut sites = Vec::new();
    for (index, component) in facts.parameters().iter().enumerate() {
        budget.poll()?;
        if component.is_array()
            && let Some(bytes) = component.bytes(descriptor)
        {
            sites.push((receiver_slots + index, bytes));
        } else if let Some(bytes) = component.bytes(descriptor)
            && let Some(internal) = bytes
                .strip_prefix(b"L")
                .and_then(|bytes| bytes.strip_suffix(b";"))
        {
            sites.push((receiver_slots + index, internal));
        }
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(facts.parameters().len()).unwrap_or(u64::MAX),
    )?;
    Ok(Some(sites))
}

/// The class a named SSA reference states, in the internal form both descriptor and class-file
/// spellings normalize to. An array is the array closed set's own shape, not a class name, and a
/// reference the facts do not name states no proof at all.
pub fn named_reference_class(ty: &jarde_jvm::method_ir::Value) -> Option<&[u8]> {
    let jarde_jvm::method_ir::Value::Ref(jarde_jvm::method_ir::RefType::Named { name, .. }) = ty
    else {
        return None;
    };
    match name.first() {
        Some(b'[') => None,
        _ => Some(
            name.strip_prefix(b"L")
                .and_then(|name| name.strip_suffix(b";"))
                .unwrap_or(name),
        ),
    }
}

/// Read a named class or reference-array descriptor without guessing a type for primitive arrays,
/// unknown references, malformed descriptors, or a class name with no faithful source spelling.
pub fn snapshot_reference_shape(name: &[u8]) -> Option<SnapshotReferenceShape> {
    if name.first() == Some(&b'[') {
        let facts = jarde_reader::classfile::descriptor_facts(
            name,
            jarde_reader::classfile::DescriptorKind::Field,
        )
        .ok()?;
        let component = facts.single()?;
        let rank = component.dimensions();
        if !component.is_array() || rank == 0 || rank > u32::from(u8::MAX) {
            return None;
        }
        let encoded = component.bytes(name)?;
        let rank_usize = usize::try_from(rank).ok()?;
        let leaf_descriptor = encoded.get(rank_usize..)?;
        let leaf_class = leaf_descriptor.strip_prefix(b"L")?.strip_suffix(b";")?;
        if leaf_class.is_empty() {
            return None;
        }
        let java_class = source_class_name(&JvmBytes(leaf_class.to_vec()))?;
        Some(SnapshotReferenceShape {
            source_type: format!("{java_class}{}", "[]".repeat(rank_usize)),
            leaf_class: leaf_class.to_vec(),
            rank,
        })
    } else {
        let leaf_class = name
            .strip_prefix(b"L")
            .and_then(|name| name.strip_suffix(b";"))
            .unwrap_or(name);
        if leaf_class.is_empty() {
            return None;
        }
        let java_class = source_class_name(&JvmBytes(leaf_class.to_vec()))?;
        Some(SnapshotReferenceShape {
            source_type: java_class,
            leaf_class: leaf_class.to_vec(),
            rank: 0,
        })
    }
}

/// A component obtained from an array descriptor must itself be a reference descriptor. This
/// separate reader check keeps primitive descriptor letters (for example `I`) from being mistaken
/// for class names, while source-side internal names such as a legal class named `I` remain valid.
pub fn snapshot_reference_descriptor_shape(descriptor: &[u8]) -> Option<SnapshotReferenceShape> {
    let facts = jarde_reader::classfile::descriptor_facts(
        descriptor,
        jarde_reader::classfile::DescriptorKind::Field,
    )
    .ok()?;
    let component = facts.single()?;
    let rank = usize::try_from(component.dimensions()).ok()?;
    let encoded = component.bytes(descriptor)?;
    let leaf_descriptor = encoded.get(rank..)?;
    if !leaf_descriptor.starts_with(b"L") || !leaf_descriptor.ends_with(b";") {
        return None;
    }
    snapshot_reference_shape(descriptor)
}

pub fn snapshot_aastore_reference_pair(
    array_ty: &jarde_jvm::method_ir::Value,
    index_ty: &jarde_jvm::method_ir::Value,
    value_ty: &jarde_jvm::method_ir::Value,
    expected_loader: &jarde_reader::view::LoaderId,
) -> Option<(SnapshotReferenceShape, SnapshotReferenceShape)> {
    use jarde_jvm::method_ir::{RefType, Value};

    let (
        Value::Ref(RefType::Named {
            name: array_name,
            loader: array_loader,
        }),
        Value::Int,
        Value::Ref(RefType::Named {
            name: value_name,
            loader: value_loader,
        }),
    ) = (array_ty, index_ty, value_ty)
    else {
        return None;
    };
    if array_loader != value_loader
        || array_loader.as_ref() != expected_loader
        || value_loader.as_ref() != expected_loader
        || array_name.first() != Some(&b'[')
    {
        return None;
    }
    let array_facts = jarde_reader::classfile::descriptor_facts(
        array_name,
        jarde_reader::classfile::DescriptorKind::Field,
    )
    .ok()?;
    let array_component = array_facts.single()?;
    if !array_component.is_array() || array_component.dimensions() == 0 {
        return None;
    }
    let component_descriptor = array_name.get(1..)?;
    let source = snapshot_reference_shape(value_name)?;
    let target = snapshot_reference_descriptor_shape(component_descriptor)?;
    (source.rank == target.rank).then_some((source, target))
}

/// One `aastore`'s type pair, read from its exact SSA stack operands. The top three absolute stack
/// depths are array reference, index and value; a prefix may remain below them. The component
/// descriptor is independently validated by the reader, and only equal-rank class/reference-array
/// pairs can reach the snapshot header walk.
pub(crate) fn snapshot_aastore_reference_pairs(
    index: &SsaSiteIndex<'_>,
    bci: u32,
    expected_loader: &jarde_reader::view::LoaderId,
    budget: &mut Budget,
) -> Result<Vec<(SnapshotReferenceShape, SnapshotReferenceShape)>> {
    use jarde_jvm::method_ir::Slot;

    let mut pairs = Vec::new();
    if let Some(site) = index.site(bci, budget)? {
        if site.opcode() != 0x53 || site.reads().len() != 3 {
            return Ok(pairs);
        }
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(site.reads().len()).unwrap_or(u64::MAX),
        )?;
        let mut stack_reads: Vec<(u32, jarde_jvm::method_ir::ValueId)> = site
            .reads()
            .iter()
            .filter_map(|(slot, value)| match slot {
                Slot::Stack(depth) => Some((*depth, *value)),
                Slot::Local(_) => None,
            })
            .collect();
        stack_reads.sort_by_key(|(depth, _)| *depth);
        if let [array_operand, index_operand, value_operand] = stack_reads.as_slice()
            && array_operand.0.checked_add(1) == Some(index_operand.0)
            && index_operand.0.checked_add(1) == Some(value_operand.0)
            && let Some(pair) = snapshot_aastore_reference_pair(
                index.ssa.value(array_operand.1).ty(),
                index.ssa.value(index_operand.1).ty(),
                index.ssa.value(value_operand.1).ty(),
                expected_loader,
            )
        {
            pairs.push(pair);
        }
    }
    Ok(pairs)
}

/// Exact-BCI reference relations for invocation/array-store conversions and local legalization.
/// Descriptor, receiver, return, throw, field and array uses constrain local loads; actual SSA
/// producers and copies constrain stores. Every named relation uses the same selected-Header walk
/// and registered release contract. Negative outcomes remain typed facts, never implicit casts.
pub fn prove_reference_relations<H: RecoveryHeaders>(
    request: &MethodAnalysisRequest,
    ir: &jarde_jvm::method_ir::MethodIr,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<Vec<jarde_java::report::ReferenceRelationInput>, H::Error> {
    use jarde_reader::classfile::{CpEntryKind, cp_entry};
    let (Some(code), Some(ssa)) = (ir.code(), ir.ssa()) else {
        return Ok(Vec::new());
    };
    let site_index = SsaSiteIndex::build(ssa, budget)?;
    let mut proved = Vec::new();
    for instruction in &code.instructions {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if instruction.opcode == 0x53 {
            for (source, target) in snapshot_aastore_reference_pairs(
                &site_index,
                instruction.bci,
                &request.environment.runtime.load_domain.loader,
                budget,
            )? {
                if source.rank != target.rank || source.leaf_class == target.leaf_class {
                    continue;
                }
                let relation = reference_relation(
                    request,
                    &source.leaf_class,
                    &target.leaf_class,
                    headers,
                    budget,
                )?;
                {
                    proved.push(jarde_java::report::ReferenceRelationInput {
                        relation,
                        bci: instruction.bci,
                        source: source.source_type,
                        target: target.source_type,
                    });
                }
            }
            continue;
        }
        // An `invokedynamic` site names no method reference at all; every other call carries its
        // receiver as the bottom operand it consumes, and a static call's first parameter stands
        // first instead.
        let receiver_slots = match instruction.opcode {
            0xb6 | 0xb7 | 0xb9 => 1,
            0xb8 => 0,
            _ => continue,
        };
        let Some(index) = instruction.constant_pool_index else {
            continue;
        };
        let Ok(entry) = cp_entry(ir.constant_pool(), index) else {
            continue;
        };
        let descriptor: &[u8] = match &entry.kind {
            CpEntryKind::MethodRef { descriptor, .. }
            | CpEntryKind::InterfaceMethodRef { descriptor, .. } => descriptor.0.as_slice(),
            _ => continue,
        };
        let Some(sites) = reference_parameter_sites_checked(descriptor, receiver_slots, budget)?
        else {
            continue;
        };
        if sites.is_empty() {
            continue;
        }
        if std::env::var_os("JARDE_DEBUG_RELATION").is_some() {
            eprintln!(
                "[dbg] bci={} opcode={:#x} descriptor={} sites={:?}",
                instruction.bci,
                instruction.opcode,
                String::from_utf8_lossy(descriptor),
                sites.iter().map(|(o, t)| (*o, String::from_utf8_lossy(t).to_string())).collect::<Vec<_>>()
            );
        }
        // The operands this call consumes, bottom first: the one site this BCI holds records one
        // read per consumed value at its stack depth — whatever residual prefix the stack carries
        // below the call's own operands — so the ascending depths are the operand order itself.
        let mut operands: Vec<&jarde_jvm::method_ir::Value> = Vec::new();
        if let Some(site) = site_index.site(instruction.bci, budget)? {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(site.reads().len()).unwrap_or(u64::MAX),
            )?;
            let mut reads: Vec<(u32, &jarde_jvm::method_ir::Value)> = site
                .reads()
                .iter()
                .filter_map(|(slot, value)| match slot {
                    jarde_jvm::method_ir::Slot::Stack(depth) => {
                        Some((*depth, ssa.value(*value).ty()))
                    }
                    jarde_jvm::method_ir::Slot::Local(_) => None,
                })
                .collect();
            reads.sort_by_key(|(depth, _)| *depth);
            operands = reads.into_iter().map(|(_, ty)| ty).collect();
        }
        if std::env::var_os("JARDE_DEBUG_RELATION").is_some() {
            eprintln!(
                "[dbg] bci={} operands={} {:?}",
                instruction.bci,
                operands.len(),
                operands
                    .iter()
                    .map(|ty| format!("{ty:?}"))
                    .collect::<Vec<_>>()
            );
        }
        for (ordinal, target) in sites {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            let Some(target) = snapshot_reference_shape(target) else {
                continue;
            };
            let Some(presented) = operands.get(ordinal).and_then(|ty| {
                let jarde_jvm::method_ir::Value::Ref(jarde_jvm::method_ir::RefType::Named {
                    name,
                    ..
                }) = ty
                else {
                    return None;
                };
                snapshot_reference_shape(name)
            }) else {
                continue;
            };
            // Unequal ranks and exact components are intrinsic array-shape questions. A named
            // leaf conversion, scalar or equal-rank array, consumes the same environment proof.
            if presented.rank != target.rank || presented.leaf_class == target.leaf_class {
                continue;
            }
            if target.source_type == "java.lang.Object" {
                continue;
            }
            let relation = reference_relation(
                request,
                &presented.leaf_class,
                &target.leaf_class,
                headers,
                budget,
            )?;
            proved.push(crate::report::ReferenceRelationInput {
                bci: instruction.bci,
                source: presented.source_type,
                target: target.source_type,
                relation,
            });
        }
    }
    append_local_reference_relations(request, ir, headers, budget, &mut proved)?;
    Ok(proved)
}

fn append_local_reference_relations<H: RecoveryHeaders>(
    request: &MethodAnalysisRequest,
    ir: &MethodIr,
    headers: &mut H,
    budget: &mut Budget,
    proved: &mut Vec<crate::report::ReferenceRelationInput>,
) -> std::result::Result<(), H::Error> {
    use crate::assignability::{array_reference_widens, reference_constraints};
    use crate::report::{ReferenceRelation, ReferenceRelationInput};
    use std::collections::{BTreeMap, BTreeSet};
    let (Some(code), Some(ssa), Some(declaration)) = (ir.code(), ir.ssa(), ir.declaration()) else {
        return Ok(());
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        code.instructions.len() as u64,
    )?;
    let operations = crate::decode::Operations::of(code, ir.constant_pool());
    let descriptor = &request.method.descriptor.0;
    let result_type = descriptor_facts(descriptor, DescriptorKind::Method)
        .ok()
        .and_then(|facts| facts.result().and_then(crate::type_of_component));
    let constraints = reference_constraints(
        ssa,
        &operations,
        result_type.as_ref(),
        !declaration.is_static(),
        budget,
    )?;
    // This is a forward catalogue from raw facts. Hierarchy reads never reconstruct a binary
    // identity by reversing a rendered source name or by guessing a nested-class separator.
    let mut raw = BTreeMap::<String, Vec<u8>>::new();
    let mut register = |name: &[u8], descriptor: bool, budget: &mut Budget| -> Result<()> {
        budget.poll()?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            1 + (name.len() as u64).div_ceil(64),
        )?;
        let shape = if descriptor {
            snapshot_reference_descriptor_shape(name)
        } else if name.first() == Some(&b'[') {
            snapshot_reference_shape(name)
        } else {
            source_class_name(&JvmBytes(name.to_vec())).map(|source_type| SnapshotReferenceShape {
                source_type,
                leaf_class: name.to_vec(),
                rank: 0,
            })
        };
        if let Some(shape) = shape {
            let leaf = shape.source_type.trim_end_matches("[]").to_owned();
            if !raw.contains_key(&leaf) {
                budget.charge(CountedBudgetDimension::IrItems, 1)?;
                raw.insert(leaf, shape.leaf_class);
            }
        }
        Ok(())
    };
    for name in [
        b"java/lang/Object".as_slice(),
        b"java/lang/String",
        b"java/lang/Class",
        b"java/lang/Throwable",
    ] {
        register(name, false, budget)?;
    }
    for entry in ir.constant_pool() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let descriptor = match &entry.kind {
            CpEntryKind::Class { name, .. } => {
                register(&name.0, false, budget)?;
                continue;
            }
            CpEntryKind::FieldRef { descriptor, .. }
            | CpEntryKind::MethodRef { descriptor, .. }
            | CpEntryKind::InterfaceMethodRef { descriptor, .. }
            | CpEntryKind::InvokeDynamic { descriptor, .. } => &descriptor.0,
            _ => continue,
        };
        let kind = if descriptor.first() == Some(&b'(') {
            DescriptorKind::Method
        } else {
            DescriptorKind::Field
        };
        if let Ok(facts) = descriptor_facts(descriptor, kind) {
            for component in facts.components() {
                if let Some(bytes) = component.bytes(descriptor) {
                    register(bytes, true, budget)?;
                }
            }
        }
    }
    if let Ok(facts) = descriptor_facts(descriptor, DescriptorKind::Method) {
        for component in facts.components() {
            if let Some(bytes) = component.bytes(descriptor) {
                register(bytes, true, budget)?;
            }
        }
    }
    let mut pairs = BTreeSet::new();
    let mut insert = |bci, source: &str, target: &str, budget: &mut Budget| -> Result<()> {
        if source == target
            || target == "java.lang.Object"
            || array_reference_widens(source, target)
        {
            return Ok(());
        }
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !pairs.contains(&(bci, source.to_owned(), target.to_owned())) {
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            pairs.insert((bci, source.to_owned(), target.to_owned()));
        }
        Ok(())
    };
    for (slot, bci, _) in &constraints.stores {
        if let Some(domain) = constraints.domains.get(slot) {
            for source in domain {
                for target in domain {
                    insert(*bci, source, target, budget)?;
                }
            }
        }
    }
    for ((slot, bci), targets) in &constraints.requirements {
        for source in constraints.domains.get(slot).into_iter().flatten() {
            for target in targets {
                insert(*bci, source, target, budget)?;
            }
        }
    }
    for (bci, value, target) in &constraints.consumers {
        if let Some(types) = constraints.bounds.types_of(*value) {
            for source in types {
                insert(*bci, source, target, budget)?;
            }
        }
        for (slot, _, projection) in constraints.bounds.local_sources(*value, budget)? {
            for source in constraints.domains.get(&slot).into_iter().flatten() {
                let mut component = source.as_str();
                for _ in 0..projection {
                    component = component.strip_suffix("[]").unwrap_or("");
                }
                if !component.is_empty() {
                    insert(*bci, component, target, budget)?;
                }
            }
        }
    }
    // A checkcast states the pair itself: the operand's proved types against the cast's own
    // target. The cast's runtime check is the original compiler's castability proof (JLS 5.5),
    // and the recovered source keeps the cast, so the pair's relation — read from the operand
    // class's own super chain and interfaces — is what lets the cast render without the
    // `(Object)` stair that only uncompilable pairs need.
    let ssa_by_bci: BTreeMap<u32, &_> = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .map(|instruction| (instruction.bci(), instruction))
        .collect();
    for instruction in &code.instructions {
        if instruction.opcode != 0xc0 {
            continue;
        }
        let Some(index) = instruction.constant_pool_index else {
            continue;
        };
        let Ok(entry) = jarde_reader::classfile::cp_entry(ir.constant_pool(), index) else {
            continue;
        };
        let Some(jarde_reader::classfile::CpEntryKind::Class { name, .. }) = Some(&entry.kind)
        else {
            continue;
        };
        let Some(target_shape) = snapshot_reference_shape(&name.0) else {
            continue;
        };
        let Some(ssa_instruction) = ssa_by_bci.get(&instruction.bci) else {
            continue;
        };
        for (slot, value) in ssa_instruction.reads() {
            if !matches!(slot, jarde_jvm::method_ir::Slot::Stack(_)) {
                continue;
            }
            let Some(types) = constraints.bounds.types_of(*value) else {
                continue;
            };
            for source in types {
                if source == target_shape.source_type.as_str() {
                    continue;
                }
                insert(instruction.bci, source, &target_shape.source_type, budget)?;
            }
        }
    }
    let mut existing = BTreeSet::new();
    for proof in proved.iter() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        existing.insert((proof.bci, proof.source.clone(), proof.target.clone()));
    }
    let mut cache = BTreeMap::new();
    for (bci, source, target) in pairs {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if existing.contains(&(bci, source.clone(), target.clone())) {
            continue;
        }
        let key = (source.clone(), target.clone());
        let relation = if let Some(relation) = cache.get(&key) {
            *relation
        } else {
            let source_leaf = source.trim_end_matches("[]");
            let target_leaf = target.trim_end_matches("[]");
            let relation = if source.matches("[]").count() != target.matches("[]").count() {
                ReferenceRelation::Incompatible
            } else if let (Some(source_raw), Some(target_raw)) =
                (raw.get(source_leaf), raw.get(target_leaf))
            {
                reference_relation(request, source_raw, target_raw, headers, budget)?
            } else {
                ReferenceRelation::Unknown
            };
            if relation != ReferenceRelation::Unknown {
                budget.charge(CountedBudgetDimension::IrItems, 1)?;
                cache.insert(key, relation);
            }
            relation
        };
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        proved.push(ReferenceRelationInput {
            bci,
            source,
            target,
            relation,
        });
    }
    Ok(())
}

pub fn interface_reaches(
    start: &usize,
    target: &usize,
    graph: &std::collections::BTreeMap<usize, Vec<usize>>,
    budget: &mut Budget,
) -> Result<Option<bool>> {
    let mut pending = vec![*start];
    let mut visited = std::collections::BTreeSet::new();
    while let Some(owner) = pending.pop() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if &owner == target {
            return Ok(Some(true));
        }
        if !visited.insert(owner.clone()) {
            continue;
        }
        let Some(parents) = graph.get(&owner) else {
            return Ok(None);
        };
        pending.extend(parents.iter().cloned());
    }
    Ok(Some(false))
}

pub fn interface_source_type_accessible(current: &[u8], owner: &[u8], flags: u16) -> bool {
    // A nested interface qualifier is spelled by the binary name the bytes carry — the same name
    // the class-source header writes in its own `implements` clause (`implements F1$A`), because
    // that presentation keeps `$` as an identifier character rather than inventing a nesting
    // structure the class file does not state. So a `$` in the owner is not itself a refusal; the
    // accessibility criterion still is: the qualifier must be public or in the calling class's own
    // package, which are the shapes a Java 8 source can name.
    flags & 0x0001 != 0 || package_name(current) == package_name(owner)
}

pub fn package_name(owner: &[u8]) -> &[u8] {
    owner
        .iter()
        .rposition(|byte| *byte == b'/')
        .map_or(&[], |slash| &owner[..slash])
}

pub fn unique_source_default(
    root: usize,
    name: &[u8],
    descriptor: &[u8],
    nodes: &[SelectedInterfaceNode],
    graph: &std::collections::BTreeMap<usize, Vec<usize>>,
    budget: &mut Budget,
) -> Result<bool> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_BRIDGE: u16 = 0x0040;
    const ACC_NATIVE: u16 = 0x0100;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_SYNTHETIC: u16 = 0x1000;

    let mut pending = vec![root];
    let mut closure = std::collections::BTreeSet::new();
    while let Some(owner) = pending.pop() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !closure.insert(owner.clone()) {
            continue;
        }
        let Some(parents) = graph.get(&owner) else {
            return Ok(false);
        };
        pending.extend(parents.iter().cloned());
    }
    let mut declarations: Vec<(usize, &MemberHeader)> = Vec::new();
    for owner in &closure {
        budget.poll()?;
        let Some(node) = nodes.get(*owner) else {
            return Ok(false);
        };
        let mut exact = 0_u32;
        for method in &node.facts.methods {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if method.name.raw().0.as_slice() != name {
                continue;
            }
            if method.descriptor.raw().0.as_slice() != descriptor {
                // This first slice cannot establish which same-named overload Java would bind.
                return Ok(false);
            }
            exact = exact.saturating_add(1);
            if exact > 1
                || method.access_flags & (ACC_BRIDGE | ACC_SYNTHETIC) != 0
                || method
                    .attributes
                    .iter()
                    .any(|attribute| attribute.name.raw().0.as_slice() == b"Signature")
            {
                return Ok(false);
            }
            declarations.push((*owner, method));
        }
    }
    if declarations.is_empty() {
        return Ok(false);
    }
    let mut maximal = Vec::new();
    'candidate: for (owner, method) in &declarations {
        for (other_owner, _) in &declarations {
            if owner == other_owner {
                continue;
            }
            if interface_reaches(other_owner, owner, graph, budget)? != Some(false) {
                continue 'candidate;
            }
        }
        maximal.push(method);
    }
    let [method] = maximal.as_slice() else {
        return Ok(false);
    };
    let flags = method.access_flags;
    let code_count = method
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0.as_slice() == b"Code")
        .count();
    Ok(flags & ACC_PUBLIC != 0
        && flags & (ACC_PRIVATE | ACC_STATIC | ACC_ABSTRACT | ACC_NATIVE) == 0
        && code_count == 1)
}

/// Closed source-level contract slice for the standard `List<E>` receiver methods used by this
/// proof. The exact erasures and generic declarations were confirmed from JDK 8 and JDK 23
/// `javap -s -v`: inherited `Collection<E>.iterator():Iterator<E>` and `add(E):boolean`, plus
/// non-generic `remove(Object):boolean`, `size():int`, and `clear():void`. `remove(int):E` is
/// deliberately absent; the physical descriptor makes the overload distinction explicit.
pub fn generic_list_member_contract(name: &[u8], descriptor: &[u8]) -> Option<u8> {
    match (name, descriptor) {
        (b"iterator", b"()Ljava/util/Iterator;") => Some(0),
        (b"add", b"(Ljava/lang/Object;)Z") => Some(1),
        (b"remove", b"(Ljava/lang/Object;)Z") => Some(2),
        (b"size", b"()I") => Some(3),
        (b"clear", b"()V") => Some(4),
        (b"get", b"(I)Ljava/lang/Object;") => Some(5),
        _ => None,
    }
}

pub fn generic_call_reference_type_from_descriptor(descriptor: &[u8]) -> Option<crate::ast::Type> {
    let facts = descriptor_facts(descriptor, DescriptorKind::Field).ok()?;
    match crate::type_of_component(facts.single()?)? {
        reference @ crate::ast::Type::Reference(_) => Some(reference),
        _ => None,
    }
}

pub fn internal_package(name: &[u8]) -> &[u8] {
    name.iter()
        .rposition(|byte| *byte == b'/')
        .map_or(&[], |slash| &name[..slash])
}

/// Whether a class's internal name is directly spellable source text at the root class's
/// package: no `$`, every `/` segment a legal Java identifier, and the same package. The
/// anonymous superclass projection's source-type refusals (`anonymous_super_source_type_unproved`)
/// share this one predicate for the direct superclass and — since ring 2
/// (`recover-anonymous-supertype-return`) — for the root method's declared return type.
pub fn spellable_source_type(name: &[u8], root: &[u8]) -> bool {
    internal_package(name) == internal_package(root) && spellable_source_name(name)
}

/// Whether a class's internal name is spellable source text **anywhere**: no `$`, every `/`
/// segment a legal Java identifier.
///
/// [`spellable_source_type`] additionally requires the same package, which is the *companion-body*
/// projections' own rule — their text replaces a declaration's own name, and a type of another
/// package would have to be imported. An **allocation point** only needs the name to be writable
/// where it stands (the presentation spells the fully qualified name), and whether the named class
/// is reachable from there is stated by its own access flags, which the double-brace admission
/// reads beside this predicate.
pub fn spellable_source_name(name: &[u8]) -> bool {
    !name.contains(&b'$') && source_class_name(&JvmBytes(name.to_vec())).is_some()
}

pub fn java_code_name_spans(source: &str, name: &str) -> Option<Vec<(usize, usize)>> {
    fn identifier_byte(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') || byte >= 0x80
    }
    let bytes = source.as_bytes();
    let needle = name.as_bytes();
    let mut spans = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"//") {
            index = bytes[index..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(bytes.len(), |offset| index + offset);
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            let offset = bytes[index + 2..]
                .windows(2)
                .position(|pair| pair == b"*/")?;
            index += 2 + offset + 2;
            continue;
        }
        if matches!(bytes[index], b'"' | b'\'') {
            let quote = bytes[index];
            index += 1;
            let mut closed = false;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                } else if bytes[index] == quote {
                    index += 1;
                    closed = true;
                    break;
                } else {
                    index += 1;
                }
            }
            if !closed {
                return None;
            }
            continue;
        }
        if bytes[index..].starts_with(needle) {
            let before = index
                .checked_sub(1)
                .is_some_and(|pos| identifier_byte(bytes[pos]));
            let end = index + needle.len();
            let after = bytes.get(end).is_some_and(|byte| identifier_byte(*byte));
            if !before && !after {
                spans.push((index, end));
                index = end;
                continue;
            }
        }
        let next = source[index..].chars().next()?;
        index += next.len_utf8();
    }
    Some(spans)
}

/// The index of the delimiter that closes the one at `open`, skipping string and character
/// literals, or `None` when the text ends first.
pub fn matching_delimiter(text: &str, open: usize, left: u8, right: u8) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0isize;
    let mut index = open;
    let mut literal: Option<u8> = None;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(quote) = literal {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote {
                literal = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            literal = Some(byte);
        } else if byte == left {
            depth += 1;
        } else if byte == right {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

/// Whether an argument list text holds a top-level comma (nested parentheses, generics and
/// literals do not count).
pub fn top_level_commas(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut depth = 0isize;
    let mut angles = 0isize;
    let mut index = 0;
    let mut literal: Option<u8> = None;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(quote) = literal {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote {
                literal = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            literal = Some(byte);
        } else {
            match byte {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b'<' => angles += 1,
                b'>' => angles -= 1,
                b',' if depth == 0 && angles == 0 => return true,
                _ => {}
            }
        }
        index += 1;
    }
    false
}

/// Whether the loaded field or invoked result has this exact object type. A method parameter
/// mentioning the class does not establish the type of the produced value.
fn static_fold_pool_value_names_type(
    entry: &jarde_reader::classfile::CpEntryFacts,
    target_binary: &[u8],
) -> bool {
    use jarde_reader::classfile::{CpEntryKind, DescriptorKind, descriptor_facts};
    let (descriptor, kind) = match &entry.kind {
        CpEntryKind::FieldRef { descriptor, .. } | CpEntryKind::Dynamic { descriptor, .. } => {
            (&descriptor.0, DescriptorKind::Field)
        }
        CpEntryKind::MethodRef { descriptor, .. }
        | CpEntryKind::InterfaceMethodRef { descriptor, .. }
        | CpEntryKind::InvokeDynamic { descriptor, .. } => (&descriptor.0, DescriptorKind::Method),
        _ => return false,
    };
    let Ok(facts) = descriptor_facts(descriptor, kind) else {
        return false;
    };
    let value = match kind {
        DescriptorKind::Method => facts.result(),
        _ => facts.single(),
    };
    value
        .and_then(|value| value.object_name())
        .is_some_and(|name| name.0 == target_binary)
}

/// Whether a producer's pool entry names this class directly, as its member owner, or as the
/// field/result type. The direct-coverage matcher below checks the first two positions only.
pub fn static_fold_pool_entry_names_type(
    entry: &jarde_reader::classfile::CpEntryFacts,
    target_binary: &[u8],
) -> bool {
    use jarde_reader::classfile::CpEntryKind;
    let direct = match &entry.kind {
        CpEntryKind::Class { name, .. } => name.0 == target_binary,
        CpEntryKind::FieldRef { owner, .. }
        | CpEntryKind::MethodRef { owner, .. }
        | CpEntryKind::InterfaceMethodRef { owner, .. } => owner.0 == target_binary,
        _ => false,
    };
    direct || static_fold_pool_value_names_type(entry, target_binary)
}

/// The sound anchor for a token no covering instruction names itself: a local-variable
/// declaration's type. A producer-local introduced by the AST uses the instruction's single
/// stack result and its parsed field/return descriptor; a physical local uses the store —
/// exactly one local written, exactly the one stack value read — and the stored value must have
/// one instruction as its single definition, whose own constant-pool entry names the type this
/// token spells. That producer is the token's physical origin: the recovered declaration spells
/// the stored value's type, and the value's type is the producer's pool-typed result (for a
/// lambda site, the `invokedynamic` descriptor's return type). A phi-defined value, an entry
/// value or a producer whose pool entry names nothing is no anchor at all.
pub fn static_fold_stored_type_anchor(
    ssa: Option<&jarde_jvm::method_ir::SsaTable>,
    code: &jarde_reader::classfile::MethodCodeFacts,
    pool: &[jarde_reader::classfile::CpEntryFacts],
    source_bcis: &std::collections::BTreeSet<u32>,
    target_binary: &[u8],
) -> Option<u32> {
    let Some(ssa) = ssa else {
        return None;
    };
    for bci in source_bcis {
        let Some(store) = ssa
            .blocks()
            .iter()
            .flat_map(|block| block.instructions())
            .find(|instruction| instruction.bci() == *bci)
        else {
            continue;
        };
        let produced_stack_value = store
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, jarde_jvm::method_ir::Slot::Stack(_)))
            .count()
            == 1;
        if produced_stack_value
            && let Some(instruction) = code
                .instructions
                .iter()
                .find(|instruction| instruction.bci == *bci)
            && let Some(index) = instruction.constant_pool_index
            && let Ok(entry) = jarde_reader::classfile::cp_entry(pool, index)
            && static_fold_pool_value_names_type(entry, target_binary)
        {
            return Some(*bci);
        }
        let writes_local = store
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, jarde_jvm::method_ir::Slot::Local(_)))
            .collect::<Vec<_>>();
        let [(_, _written)] = writes_local.as_slice() else {
            continue;
        };
        // The store moves one stack value into the local: in this SSA the written local slot
        // gets a fresh value defined by the store itself, so the producer is the *read* value's
        // definition — the one instruction that produced what the store holds.
        let reads_stack = store
            .reads()
            .iter()
            .filter(|(slot, _)| matches!(slot, jarde_jvm::method_ir::Slot::Stack(_)))
            .collect::<Vec<_>>();
        let [(_, stored)] = reads_stack.as_slice() else {
            continue;
        };
        let jarde_jvm::method_ir::Definition::Instruction { bci: producer, .. } =
            ssa.value(*stored).def()
        else {
            continue;
        };
        let Some(instruction) = code
            .instructions
            .iter()
            .find(|instruction| instruction.bci == *producer)
        else {
            continue;
        };
        let Some(index) = instruction.constant_pool_index else {
            continue;
        };
        let Ok(entry) = jarde_reader::classfile::cp_entry(pool, index) else {
            continue;
        };
        if static_fold_pool_entry_names_type(entry, target_binary) {
            return Some(*producer);
        }
    }
    None
}

pub fn source_type_path_segment(
    definition: &PhysicalDefinitionId,
    binary_name: &JvmBytes,
    source_name: String,
    type_parameter_count: usize,
    enclosing_binary_name: Option<JvmBytes>,
    is_static: bool,
) -> jarde_java::report::MemberInnerSourceSegmentInput {
    jarde_java::report::MemberInnerSourceSegmentInput {
        definition: definition.clone(),
        binary_name: binary_name.clone(),
        source_name,
        type_parameter_count,
        enclosing_binary_name,
        is_static,
    }
}

/// Prove one current-class bridge's source target from borrowed same-run facts.
/// The iterator states (identity, flags, declaration-present, optional recovery/analysis).
/// Only the selected-header callback may read dependencies; the facade retains inherited
/// contract resolution, execution merging, stop rollback, and projection commit.
pub fn prove_bridge_source_target<'a>(
    class_bytes: &[u8],
    facts: &ClassMemberFacts,
    definition: &PhysicalDefinitionId,
    candidate: &crate::bridge::ClassSourceBridgeCandidate,
    methods: impl Clone
    + Iterator<
        Item = (
            &'a PhysicalMethodId,
            u16,
            bool,
            Option<(&'a crate::report::RecoveryReport, &'a ExecutionReport)>,
        ),
    >,
    initiating_loader: &LoaderId,
    budget: &mut Budget,
    header: impl FnMut(&LoaderId, &[u8], &mut Budget) -> Result<Option<ReferenceClassHeader>>,
) -> Result<std::result::Result<(&'a PhysicalMethodId, bool, u32), &'static str>> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_BRIDGE: u16 = 0x0040;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_NATIVE: u16 = 0x0100;
    const ACC_SYNTHETIC: u16 = 0x1000;
    const RECONSTRUCTIBLE_BRIDGE_FLAGS: u16 = ACC_PUBLIC | ACC_BRIDGE | ACC_SYNTHETIC;

    let Some(member) = candidate.member.as_ref() else {
        return Ok(Err("the bridge sidecar has no physical method identity"));
    };
    let class_name = facts.this_class.raw().0.as_slice();
    let complete_body = |body: Option<(&crate::report::RecoveryReport, &ExecutionReport)>| {
        body.is_some_and(|(report, analysis)| {
            report.produced()
                && report.quality == jarde_jvm::ir::Quality::Structured
                && report.fallbacks.is_empty()
                && matches!(&report.execution, ExecutionReport::Complete { .. })
                && matches!(analysis, ExecutionReport::Complete { .. })
        })
    };
    if &member.owner != definition {
        return Ok(Err(
            "the sidecar identity belongs to another physical class",
        ));
    }
    if facts.stopped_at.is_some() {
        return Ok(Err("the physical class method table is incomplete"));
    }
    let Some(flags) = candidate.access_flags else {
        return Ok(Err("the bridge member flags were not stated"));
    };
    // The bridge is reconstructible when its modifiers are `public bridge synthetic` — but the
    // *public* bit itself follows the overridden contract: javac writes a package-private
    // bridge for an override of a package-private parent method, and recompiling that source
    // regenerates exactly that visibility. What must hold is the subset (no modifier beyond
    // the three) and the bridge identity itself (`bridge` and `synthetic`, the declared fact
    // this admission never infers from a body); that the bridge never out-views the kept
    // source method is the source gate's own check below.
    if flags & !RECONSTRUCTIBLE_BRIDGE_FLAGS != 0
        || flags & (ACC_BRIDGE | ACC_SYNTHETIC) != (ACC_BRIDGE | ACC_SYNTHETIC)
    {
        return Ok(Err(
            "the physical bridge has modifiers beyond public bridge synthetic that source reconstruction does not prove",
        ));
    }
    if candidate.has_exception_handlers {
        return Ok(Err(
            "the bridge Code declares an exception handler or was unavailable",
        ));
    }
    if !candidate.presented || !candidate.pure_forward {
        return Ok(Err("bridge@1 did not prove a pure single forward"));
    }
    let Some(target) = candidate.target.as_ref() else {
        return Ok(Err("bridge@1 did not retain a structured call target"));
    };
    let Some(call_bci) = candidate.call_bci else {
        return Ok(Err("bridge@1 did not retain the forward call BCI"));
    };
    let bridge_headers: Vec<_> = facts
        .methods
        .iter()
        .filter(|header| {
            header.name.raw().0 == member.name.0 && header.descriptor.raw().0 == member.descriptor.0
        })
        .collect();
    let [bridge_header] = bridge_headers.as_slice() else {
        return Ok(Err("the physical bridge header is missing or ambiguous"));
    };
    if bridge_header.access_flags != flags {
        return Ok(Err(
            "bridge sidecar flags disagree with the unique physical method header",
        ));
    }
    if bridge_header
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0.as_slice() == b"Code")
        .count()
        != 1
    {
        return Ok(Err(
            "the physical bridge header does not declare exactly one Code attribute",
        ));
    }
    // Beside its one Code body (checked above), the only method metadata a bridge may carry is
    // the `MethodParameters` record javac writes for a bridge's mandated parameters: one entry
    // per declared parameter, no name, `synthetic` — the record a recompiled source restates
    // identically, so copying the member loses nothing. Any other attribute, a second
    // `MethodParameters`, or an entry with a name, another flag or the wrong count is a fact
    // the source copy cannot prove.
    let reconstructible_metadata = bridge_header
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0.as_slice() == b"MethodParameters")
        .count()
        <= 1
        && bridge_header.attributes.iter().all(|attribute| {
            let name = attribute.name.raw().0.as_slice();
            name == b"Code"
                || name == b"MethodParameters"
                    && bridge_parameter_count(&member.descriptor.0).is_some_and(|count| {
                        bridge_method_parameters_canonical(class_bytes, attribute, count)
                    })
        });
    if !reconstructible_metadata {
        return Ok(Err(
            "the bridge declares method metadata whose source copying is unproved",
        ));
    }
    let bridge_methods: Vec<_> = methods
        .clone()
        .filter(|method| method.0 == member)
        .collect();
    let [bridge_method] = bridge_methods.as_slice() else {
        return Ok(Err(
            "the physical bridge has no unique class-source method record",
        ));
    };
    let complete_bridge = complete_body(bridge_method.3);
    if !complete_bridge {
        return Ok(Err(
            "the bridge body recovery did not complete as a structured artifact",
        ));
    }
    if target.kind() != jarde_java::facts::InvokeKind::Virtual {
        return Ok(Err(
            "the verified invocation is not the instance virtual call required for source override",
        ));
    }
    if target.raw_owner().0.as_slice() != class_name
        || target.raw_name().0.as_slice() != member.name.0.as_slice()
        || target.is_interface_reference()
    {
        return Ok(Err(
            "the verified invocation does not name this class's source method",
        ));
    }
    let Some((bridge_parameters, bridge_return)) = method_descriptor_parts(&member.descriptor.0)
    else {
        return Ok(Err(
            "the bridge descriptor is not a complete method descriptor",
        ));
    };
    let Some((target_parameters, target_return)) =
        method_descriptor_parts(target.raw_descriptor().0.as_slice())
    else {
        return Ok(Err(
            "the invocation descriptor is not a complete method descriptor",
        ));
    };
    if target.raw_name().0.as_slice() != member.name.0.as_slice() {
        return Ok(Err(
            "the bridge and invoked method do not share a name and parameter descriptor",
        ));
    }
    // The bridge parameters are either already the source method's own (the covariant-return
    // form) or its erasure, each widened parameter narrowed by one cast whose target is
    // exactly the source parameter — the canonical parameter-cast form `bridge@1` proved and
    // stated positionally, and the form a recompiled source regenerates for a specialized
    // override.
    let parameter_cast_form = bridge_parameters != target_parameters;
    if parameter_cast_form
        && bridge_parameter_casts_mismatch(
            &candidate.parameter_casts,
            &member.descriptor.0,
            target.raw_descriptor().0.as_slice(),
        )
    {
        return Ok(Err(
            "the bridge and invoked method do not share a name and parameter descriptor",
        ));
    }
    // The return the forward produces must be the return the erased contract declares: the
    // same type (the canonical parameter-cast form, whose return the recompiled source
    // restates), the erasure of a reference-valued override of an `Object` contract (the
    // previously proved covariant form), or a subtype the snapshot's own class files prove by
    // their header chain — a covariant override of an interface-typed contract. A chain the
    // walk cannot reach stays refused exactly as before.
    let proved_return = if bridge_return == target_return {
        parameter_cast_form
    } else if bridge_return == b"Ljava/lang/Object;" {
        target_return.starts_with(b"L") || target_return.starts_with(b"[")
    } else {
        match (
            internal_reference_name(target_return),
            internal_reference_name(bridge_return),
        ) {
            (Some(source_class), Some(bridge_class)) => snapshot_header_chain_widens_with(
                source_class,
                bridge_class,
                initiating_loader.clone(),
                SNAPSHOT_HIERARCHY_WALK_DEPTH,
                budget,
                header,
            )?,
            _ => false,
        }
    };
    if !proved_return {
        return Ok(Err(
            "the source return type is not a proved covariant subtype of the erased Object return",
        ));
    }
    let matching: Vec<_> = methods
        .clone()
        .filter(|method| {
            method.0.name.0 == member.name.0
                && method.0.descriptor.0.as_slice() == target.raw_descriptor().0.as_slice()
        })
        .collect();
    let [source] = matching.as_slice() else {
        return Ok(Err(
            "the invoked source method is missing or not unique in the current class",
        ));
    };
    let source_headers: Vec<_> = facts
        .methods
        .iter()
        .filter(|header| {
            header.name.raw().0 == source.0.name.0
                && header.descriptor.raw().0 == source.0.descriptor.0
        })
        .collect();
    let [source_header] = source_headers.as_slice() else {
        return Ok(Err("the source method header is missing or ambiguous"));
    };
    let source_flags = source.1;
    if source_flags != source_header.access_flags {
        return Ok(Err(
            "the source record flags disagree with the unique physical method header",
        ));
    }
    if source_flags & ACC_BRIDGE != 0
        // The hidden member must never out-view the source override that replaces it: a
        // public bridge whose source is not public would take a wider call surface with it.
        // (flags here are the bridge's own; a package-private bridge beside a package-private
        // source is the shape javac itself writes for a package-private contract.)
        || (flags & ACC_PUBLIC != 0 && source_flags & ACC_PUBLIC == 0)
        || source_flags & (ACC_PRIVATE | ACC_STATIC | ACC_ABSTRACT | ACC_NATIVE) != 0
        || source_flags & ACC_SYNTHETIC != 0
        || !source.2
    {
        return Ok(Err(
            "the unique target is not a spellable, concrete public source method",
        ));
    }
    if source_header
        .attributes
        .iter()
        .any(|attribute| attribute.name.raw().0.as_slice() != b"Code")
        || source_header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0.as_slice() == b"Code")
            .count()
            != 1
    {
        return Ok(Err(
            "the source method metadata could be copied to an elided bridge and is not proved reconstructible",
        ));
    }
    let complete_source = complete_body(source.3);
    if !complete_source {
        return Ok(Err("the unique source method did not recover completely"));
    }
    Ok(Ok((source.0, parameter_cast_form, call_bci)))
}

/// The exact erased Java SE 8 member, only after the caller proved clean platform absence.
/// A provided definition is resolved from its own raw declarations instead of this table.
pub fn bridge_platform_contract_member(
    owner: &JvmBytes,
    member: &PhysicalMethodId,
    use_kind: jarde_jvm::resolver::ReferenceUse,
) -> bool {
    use_kind == jarde_jvm::resolver::ReferenceUse::InvokeInterface
        && owner.0.as_slice() == b"java/lang/Comparable"
        && member.name.0.as_slice() == b"compareTo"
        && member.descriptor.0.as_slice() == b"(Ljava/lang/Object;)I"
}

/// The selected inherited contract must be regenerable from the same projected class header.
/// Preserve the original Signature demand points: interface parameter bridges inspect their
/// entry; superclass parameter bridges inspect only when their projected header is unavailable.
pub fn prove_bridge_source_contract(
    class_bytes: &[u8],
    facts: &ClassMemberFacts,
    inherited: Option<(&JvmBytes, jarde_jvm::resolver::ReferenceUse)>,
    parameter_cast_form: bool,
    class_header_projected: bool,
    class_header_interface_arguments: &[Vec<u8>],
    budget: &mut Budget,
) -> Result<std::result::Result<(), &'static str>> {
    use jarde_jvm::resolver::ReferenceUse;

    let Some((owner, use_kind)) = inherited else {
        return Ok(Ok(()));
    };
    if use_kind == ReferenceUse::InvokeInterface
        && parameter_cast_form
        && bridge_interface_contract_generic(class_bytes, facts, owner.0.as_slice(), budget)?
        && !class_header_interface_arguments
            .iter()
            .any(|name| name.as_slice() == owner.0.as_slice())
    {
        return Ok(Err(
            "the erased contract comes from a generic interface the class header spells without its type arguments, so the source could not regenerate the bridge",
        ));
    }
    if use_kind == ReferenceUse::InvokeVirtual
        && parameter_cast_form
        && !class_header_projected
        && bridge_superclass_contract_generic(class_bytes, facts, owner.0.as_slice(), budget)?
    {
        return Ok(Err(
            "the erased contract comes from a generic superclass the class header spells without its type arguments, so the source could not regenerate the bridge",
        ));
    }
    Ok(Ok(()))
}

/// Proves which same-run bridge methods may be omitted from the source projection.
///
/// The class-file policy and its refusal vocabulary live here. The caller supplies only the
/// selected header and real resolver operations owned by the facade; those callbacks merge the
/// resolver's execution into the request before returning. `state` lets those separate callbacks
/// share the facade's existing read cache and execution accumulator without this layer owning them.
#[allow(clippy::too_many_arguments)]
pub fn prove_class_source_bridges<'a, S>(
    class_bytes: &[u8],
    facts: &ClassMemberFacts,
    definition: &PhysicalDefinitionId,
    candidates: &[crate::bridge::ClassSourceBridgeCandidate],
    methods: impl Clone
    + Iterator<
        Item = (
            &'a PhysicalMethodId,
            u16,
            bool,
            Option<(&'a crate::report::RecoveryReport, &'a ExecutionReport)>,
        ),
    >,
    initiating_loader: &LoaderId,
    single_class: bool,
    java8_class_path: bool,
    class_header_projected: bool,
    class_header_interface_arguments: &[Vec<u8>],
    state: &mut S,
    budget: &mut Budget,
    mut header: impl FnMut(
        &mut S,
        &LoaderId,
        &[u8],
        &mut Budget,
    ) -> Result<Option<ReferenceClassHeader>>,
    mut resolve: impl FnMut(
        &mut S,
        &PhysicalMethodId,
        &[u8],
        ReferenceUse,
        &mut Budget,
    ) -> Result<jarde_jvm::resolver::ResolutionReport>,
    mut stopped: impl FnMut(&mut S, &Error, &mut Budget),
) -> Vec<crate::bridge::ClassSourceBridgeProof> {
    let class_name = facts.this_class.raw().0.as_slice();
    let mut proofs = Vec::new();
    for candidate in candidates {
        let Some(member) = candidate.member.as_ref() else {
            continue;
        };
        let refuse = |reason: &str| crate::bridge::ClassSourceBridgeProof {
            member: member.clone(),
            target: None,
            call_bci: candidate.call_bci,
            admitted: false,
            projected: false,
            refusal: Some(reason.to_owned()),
        };
        let source_target = prove_bridge_source_target(
            class_bytes,
            facts,
            definition,
            candidate,
            methods.clone(),
            initiating_loader,
            budget,
            |loader, owner, budget| header(state, loader, owner, budget),
        );
        let (source, parameter_cast_form, call_bci) = match source_target {
            Ok(Ok(target)) => target,
            Ok(Err(reason)) => {
                proofs.push(refuse(reason));
                continue;
            }
            Err(error) => {
                stopped(state, &error, budget);
                proofs.clear();
                proofs.push(refuse(
                    "the snapshot hierarchy walk stopped before it proved the erased return type",
                ));
                return proofs;
            }
        };
        let mut inherited = false;
        let mut inherited_owner: Option<(JvmBytes, ReferenceUse)> = None;
        let mut saw_unresolved = false;
        let mut reloaded_current = false;
        let mut stopped_resolution = false;
        let mut direct_supers: Vec<(JvmBytes, ReferenceUse)> = facts
            .interfaces
            .iter()
            .map(|name| (name.raw().clone(), ReferenceUse::InvokeInterface))
            .collect();
        if let Some(super_class) = &facts.super_class {
            direct_supers.push((super_class.raw().clone(), ReferenceUse::InvokeVirtual));
        }
        for (owner, use_kind) in direct_supers {
            if owner.0.as_slice() == class_name {
                saw_unresolved = true;
                continue;
            }
            // Object is the implicit root for every class. It is not evidence that a direct
            // bridge contract exists, and this explicit environment need not carry a JRE image.
            if use_kind == ReferenceUse::InvokeVirtual && owner.0.as_slice() == b"java/lang/Object"
            {
                continue;
            }
            if single_class {
                saw_unresolved = true;
                continue;
            }
            // Java SE 8's Comparable contract is used only under the caller's matching runtime
            // shape and after the resolver proves clean absence. A provided definition always
            // goes through ordinary member resolution below.
            if use_kind == ReferenceUse::InvokeInterface
                && owner.0.as_slice() == b"java/lang/Comparable"
                && member.descriptor.0.as_slice() == b"(Ljava/lang/Object;)I"
                && java8_class_path
            {
                let provided = match resolve(
                    state,
                    member,
                    b"java/lang/Comparable",
                    ReferenceUse::ClassReference,
                    budget,
                ) {
                    Ok(resolution) => resolution,
                    Err(error) => {
                        stopped(state, &error, budget);
                        stopped_resolution = true;
                        break;
                    }
                };
                if !matches!(&provided.execution, ExecutionReport::Complete { .. }) {
                    stopped_resolution = true;
                    break;
                }
                if provided.state == Some(ResolutionState::Missing)
                    && provided.environment_problems.is_empty()
                    && provided.unresolved_dependencies.is_empty()
                    && provided.candidates.is_empty()
                    && bridge_platform_contract_member(&owner, member, use_kind)
                {
                    inherited = true;
                    inherited_owner = Some((owner.clone(), use_kind));
                    break;
                }
            }
            let resolution = match resolve(state, member, &owner.0, use_kind, budget) {
                Ok(resolution) => resolution,
                Err(error) => {
                    stopped(state, &error, budget);
                    stopped_resolution = true;
                    break;
                }
            };
            if !matches!(&resolution.execution, ExecutionReport::Complete { .. }) {
                stopped_resolution = true;
                break;
            }
            // The access rules may read the caller's own declaration. Only an actual hierarchy
            // read back into this prepared class invalidates the proof.
            reloaded_current |= resolution.reads.iter().any(|read| {
                &read.definition == definition
                    && read.reason != jarde_jvm::resolver::ReadReason::MemberOwner
            });
            if resolution.state == Some(ResolutionState::Resolved)
                && resolution.unresolved_dependencies.is_empty()
                && !resolution
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == "resolution_access_not_checked")
                && !reloaded_current
                && resolution.resolved.as_ref().is_some_and(|resolved| {
                    matches!(
                        &resolved.member,
                        jarde_reader::model::SymbolRef::Method { name, descriptor, .. }
                            if name.0.as_slice() == member.name.0.as_slice()
                                && descriptor.0.as_slice() == member.descriptor.0.as_slice()
                    ) && resolution
                        .reads
                        .iter()
                        .any(|read| read.definition == resolved.definition)
                })
            {
                inherited = true;
                inherited_owner = Some((owner.clone(), use_kind));
                break;
            }
            if resolution.state == Some(ResolutionState::UnresolvedDependency)
                || resolution.state == Some(ResolutionState::Missing)
                || resolution.state == Some(ResolutionState::Ambiguous)
                || !resolution.unresolved_dependencies.is_empty()
            {
                saw_unresolved = true;
            }
        }
        if !inherited {
            let reason = if stopped_resolution {
                "resolution of a direct parent or interface stopped before it proved the erased method"
            } else if reloaded_current {
                "the existing resolver traversed back into this prepared class; admission is refused"
            } else if saw_unresolved {
                "a direct parent or interface needed for the erased method is unresolved"
            } else {
                "no resolved direct parent or interface requires the erased method descriptor"
            };
            if stopped_resolution {
                proofs.clear();
                proofs.push(refuse(reason));
                return proofs;
            }
            proofs.push(refuse(reason));
            continue;
        }
        match prove_bridge_source_contract(
            class_bytes,
            facts,
            inherited_owner
                .as_ref()
                .map(|(owner, use_kind)| (owner, *use_kind)),
            parameter_cast_form,
            class_header_projected,
            class_header_interface_arguments,
            budget,
        ) {
            Ok(Ok(())) => {}
            Ok(Err(reason)) => {
                proofs.push(refuse(reason));
                continue;
            }
            Err(error) => {
                stopped(state, &error, budget);
                proofs.clear();
                proofs.push(refuse(
                    "the class Signature stopped before it proved the erased contract's header",
                ));
                return proofs;
            }
        }
        proofs.push(crate::bridge::ClassSourceBridgeProof {
            member: member.clone(),
            target: Some(source.clone()),
            call_bci: Some(call_bci),
            admitted: true,
            projected: false,
            refusal: None,
        });
    }
    proofs
}

/// Whether one interface contract's owner is generic, read from the class's own `Signature`
/// attribute: the interface entry that owns the contract, spelled with arguments there, is
/// generic — and a header that spells it raw would keep the source method from implementing the
/// contract at all. A class without a `Signature` states no generic parent, and an interface the
/// signature spells without arguments is not generic.
fn bridge_interface_contract_generic(
    class_bytes: &[u8],
    facts: &ClassMemberFacts,
    owner: &[u8],
    budget: &mut Budget,
) -> Result<bool> {
    let Some(shell) = facts
        .attributes
        .iter()
        .find(|attribute| attribute.name.raw().0.as_slice() == b"Signature")
    else {
        return Ok(false);
    };
    let Some(index) = facts
        .interfaces
        .iter()
        .position(|name| name.raw().0.as_slice() == owner)
    else {
        return Ok(false);
    };
    let pool = jarde_reader::classfile::class_constant_pool(class_bytes, budget)?;
    let raw = jarde_reader::classfile::attribute_facts(
        class_bytes,
        std::slice::from_ref(shell),
        &pool,
        budget,
    )?
    .signature
    .ok_or_else(|| {
        Error::invalid_input(
            "jvm_signature_missing",
            "class Signature attribute did not resolve",
        )
    })?;
    let parsed = jarde_reader::signature::parse_class_signature(raw.0.as_slice(), budget)?;
    Ok(parsed.interfaces.get(index).is_some_and(|interface| {
        interface
            .segments
            .first()
            .is_some_and(|segment| !segment.arguments.is_empty())
    }))
}

/// Whether one superclass contract's owner is generic, read from the class's own `Signature`
/// attribute: the superclass entry that owns the contract, spelled with arguments there, is
/// generic. Whether the projected header spells those arguments is not this helper's question —
/// the admission carries that fact as `class_header_projected`, read off the same class's header
/// projection, whose success is the admit path. A class without a `Signature` states no generic
/// parent, and a superclass the signature spells without arguments is not generic. The segment
/// test is the header projection's own `parameterized_superclass` computation, so the criterion
/// and the projection cannot drift.
fn bridge_superclass_contract_generic(
    class_bytes: &[u8],
    facts: &ClassMemberFacts,
    owner: &[u8],
    budget: &mut Budget,
) -> Result<bool> {
    let Some(shell) = facts
        .attributes
        .iter()
        .find(|attribute| attribute.name.raw().0.as_slice() == b"Signature")
    else {
        return Ok(false);
    };
    let Some(super_class) = &facts.super_class else {
        return Ok(false);
    };
    if super_class.raw().0.as_slice() != owner {
        return Ok(false);
    }
    let pool = jarde_reader::classfile::class_constant_pool(class_bytes, budget)?;
    let raw = jarde_reader::classfile::attribute_facts(
        class_bytes,
        std::slice::from_ref(shell),
        &pool,
        budget,
    )?
    .signature
    .ok_or_else(|| {
        Error::invalid_input(
            "jvm_signature_missing",
            "class Signature attribute did not resolve",
        )
    })?;
    let parsed = jarde_reader::signature::parse_class_signature(raw.0.as_slice(), budget)?;
    Ok(parsed
        .superclass
        .segments
        .iter()
        .any(|segment| !segment.arguments.is_empty()))
}

pub fn method_descriptor_parts(descriptor: &[u8]) -> Option<(&[u8], &[u8])> {
    descriptor_facts(descriptor, DescriptorKind::Method).ok()?;
    let close = descriptor.iter().position(|byte| *byte == b')')?;
    Some((&descriptor[..=close], &descriptor[close + 1..]))
}

/// How many parameters a method descriptor declares, when it is one a bridge may declare.
pub fn bridge_parameter_count(descriptor: &[u8]) -> Option<usize> {
    descriptor_facts(descriptor, DescriptorKind::Method)
        .ok()
        .map(|facts| facts.parameters().len())
}

/// Whether one `MethodParameters` attribute is the exact record javac writes for a bridge's
/// mandated parameters: one entry per declared parameter, every entry nameless and `synthetic`.
/// The payload is fixed-width (JVMS 4.7.24: a `u8` count, then a `u16` name index and a `u16`
/// flag word per entry), so the bytes decide without the constant pool.
pub fn bridge_method_parameters_canonical(
    class_bytes: &[u8],
    attribute: &AttributeShell,
    parameter_count: usize,
) -> bool {
    let Ok(start) = usize::try_from(attribute.content_span.start) else {
        return false;
    };
    let Ok(length) = usize::try_from(attribute.content_span.length) else {
        return false;
    };
    let Some(content) = class_bytes.get(start..start.saturating_add(length)) else {
        return false;
    };
    let Some(&count) = content.first() else {
        return false;
    };
    usize::from(count) == parameter_count
        && content.len() == 1 + 4 * parameter_count
        && (0..parameter_count).all(|entry| {
            let entry = &content[1 + entry * 4..1 + entry * 4 + 4];
            entry[0] == 0 && entry[1] == 0 && entry[2] == 0x10 && entry[3] == 0x00
        })
}

/// Whether the stated parameter casts fail the canonical form's own bitwise criterion: every
/// parameter the bridge's erased descriptor widens must be narrowed by one stated cast whose
/// target is exactly the source parameter, in parameter order, and a stated cast may name nothing
/// else. (The bridge layer proves this against the invoked descriptor's own bytes; this re-check
/// reads the same two full descriptors the admission already parsed.)
pub fn bridge_parameter_casts_mismatch(
    casts: &[jarde_java::bridge::BridgeParameterCast],
    bridge_descriptor: &[u8],
    target_descriptor: &[u8],
) -> bool {
    let (Ok(bridge), Ok(target)) = (
        descriptor_facts(bridge_descriptor, DescriptorKind::Method),
        descriptor_facts(target_descriptor, DescriptorKind::Method),
    ) else {
        return true;
    };
    let (bridge, target) = (bridge.parameters(), target.parameters());
    if bridge.len() != target.len() {
        return true;
    }
    let mut stated = 0usize;
    let mut previous: Option<u16> = None;
    for (ordinal, (bridge_parameter, target_parameter)) in bridge.iter().zip(target).enumerate() {
        let Ok(ordinal) = u16::try_from(ordinal) else {
            return true;
        };
        let Some(cast) = casts.get(stated) else {
            // No cast left: the parameter must already be the source's own.
            if bridge_parameter.bytes(bridge_descriptor)
                != target_parameter.bytes(target_descriptor)
            {
                return true;
            }
            continue;
        };
        if cast.parameter != ordinal {
            // This parameter carries no cast; it must already be the source's own, and the
            // remaining cast belongs to a later parameter.
            if bridge_parameter.bytes(bridge_descriptor)
                != target_parameter.bytes(target_descriptor)
            {
                return true;
            }
            continue;
        }
        if previous.is_some_and(|previous| previous >= cast.parameter) {
            return true;
        }
        previous = Some(cast.parameter);
        stated += 1;
        if !crate::bridge::canonical_object_cast(&cast.ty, target_parameter) {
            return true;
        }
    }
    stated != casts.len()
}

/// The internal class name one reference-typed return descriptor states, or nothing for any other
/// return (an array or a primitive is no class the snapshot's chain can walk).
pub fn internal_reference_name(return_descriptor: &[u8]) -> Option<&[u8]> {
    return_descriptor
        .strip_prefix(b"L")
        .and_then(|name| name.strip_suffix(b";"))
}

pub fn enum_projection_stop_error(
    stop: jarde_java::StopReason,
    operation: &str,
    missing_table_code: &'static str,
) -> Error {
    match stop {
        jarde_java::StopReason::Budget {
            dimension,
            limit,
            consumed,
            requested,
            ..
        } => Error::BudgetExceeded {
            dimension,
            limit,
            consumed,
            requested,
        },
        jarde_java::StopReason::Cancelled { .. } => Error::Cancelled {
            reason: format!("{operation} was cancelled"),
        },
        jarde_java::StopReason::Interrupted { code, .. } => {
            Error::unsupported(code, format!("{operation} was interrupted"))
        }
        jarde_java::StopReason::IrTableMissing { table } => Error::unsupported(
            missing_table_code,
            format!("required IR table `{table}` is missing"),
        ),
        jarde_java::StopReason::EvidenceRefused { code, message, .. } => {
            Error::unsupported(code, message)
        }
    }
}

// Extracted existing type: MemberInnerTarget
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberInnerTarget {
    pub owner: JvmBytes,
    pub outer: JvmBytes,
    pub simple_name: String,
    pub constructor_descriptor: JvmBytes,
    pub capture_field: JvmBytes,
    pub generic_diamond: bool,
    pub source_type_parameter_count: usize,
}

// Extracted existing type: GenericOuterRelationProof
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericOuterRelationProof {
    pub enclosing: JvmBytes,
    pub simple_name: String,
    pub type_parameter_count: usize,
    /// The selected owner itself is the top-level, non-generic class that declares the target.
    /// This preserves the previously proved `Outer.Inner` source path without treating a `$` in a
    /// binary name as a nested source path.
    pub top_level_non_generic_owner: bool,
}

// Extracted existing helper: member_inner_source_type_path
pub fn member_inner_source_type_path(
    target_definition: &PhysicalDefinitionId,
    target: &MemberInnerTarget,
    outer_definition: &PhysicalDefinitionId,
    generic_outer: &GenericOuterRelationProof,
    enclosing_definition: &PhysicalDefinitionId,
    enclosing_facts: &ClassMemberFacts,
) -> Option<Vec<jarde_java::report::MemberInnerSourceSegmentInput>> {
    if generic_outer.top_level_non_generic_owner {
        if generic_outer.type_parameter_count != 0
            || !generic_outer.simple_name.is_empty()
            || target.outer != generic_outer.enclosing
            || target.owner.0
                != [
                    target.outer.0.as_slice(),
                    b"$",
                    JvmBytes::from_unicode(&target.simple_name).0.as_slice(),
                ]
                .concat()
            || target.source_type_parameter_count != 0
            || enclosing_facts.this_class.raw() != &generic_outer.enclosing
        {
            return None;
        }
        let enclosing_binary = enclosing_facts.this_class.raw();
        let enclosing_text = enclosing_binary.to_unicode()?;
        let package_and_simple: Vec<_> = enclosing_text.split('/').collect();
        if package_and_simple
            .iter()
            .any(|part| !jarde_java::names::is_java_identifier(part) || part.contains('$'))
        {
            return None;
        }
        let root_source_name = enclosing_text.replace('/', ".");
        let target_source_name = format!("{root_source_name}.{}", target.simple_name);
        return Some(vec![
            source_type_path_segment(
                enclosing_definition,
                enclosing_binary,
                root_source_name,
                0,
                None,
                true,
            ),
            source_type_path_segment(
                target_definition,
                &target.owner,
                target_source_name,
                0,
                Some(target.outer.clone()),
                false,
            ),
        ]);
    }
    if generic_outer.type_parameter_count != 1
        || enclosing_facts.this_class.raw() != &generic_outer.enclosing
        || target.outer.0
            != [
                generic_outer.enclosing.0.as_slice(),
                b"$",
                JvmBytes::from_unicode(&generic_outer.simple_name)
                    .0
                    .as_slice(),
            ]
            .concat()
        || target.owner.0
            != [
                target.outer.0.as_slice(),
                b"$",
                JvmBytes::from_unicode(&target.simple_name).0.as_slice(),
            ]
            .concat()
    {
        return None;
    }
    let enclosing_binary = enclosing_facts.this_class.raw();
    let enclosing_text = enclosing_binary.to_unicode()?;
    let package_and_simple: Vec<_> = enclosing_text.split('/').collect();
    if package_and_simple
        .iter()
        .any(|part| !jarde_java::names::is_java_identifier(part))
    {
        return None;
    }
    let root_source_name = enclosing_text.replace('/', ".");
    let outer_source_name = format!("{root_source_name}.{}", generic_outer.simple_name);
    let target_source_name = format!("{outer_source_name}.{}", target.simple_name);
    Some(vec![
        source_type_path_segment(
            enclosing_definition,
            enclosing_binary,
            root_source_name,
            0,
            None,
            true,
        ),
        source_type_path_segment(
            outer_definition,
            &target.outer,
            outer_source_name,
            generic_outer.type_parameter_count,
            Some(generic_outer.enclosing.clone()),
            true,
        ),
        source_type_path_segment(
            target_definition,
            &target.owner,
            target_source_name,
            target.source_type_parameter_count,
            Some(target.outer.clone()),
            false,
        ),
    ])
}

pub fn fold_bridge_call_rewrite_span(
    method: &PhysicalMethodId,
    recovery: &crate::RecoveryReport,
    call_bci: u32,
    owner_dotted: &str,
    bridge_name: &[u8],
    field_name: &str,
    budget: &mut Budget,
) -> Result<Option<(usize, usize, String)>> {
    let Ok(bridge_name) = std::str::from_utf8(bridge_name) else {
        return Ok(None);
    };
    let prefix = format!("{owner_dotted}.{bridge_name}(");
    let mut candidates = Vec::new();
    for segment in recovery.projection_origins() {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if segment.origin().primary().method() != Some(method)
            || segment.origin().primary().bci() != call_bci
            || segment.mentions(call_bci) != Some(jarde_java::Provenance::Direct)
        {
            continue;
        }
        let source = segment.text(&recovery.text);
        if !source.starts_with(&prefix) {
            continue;
        }
        let Some(close) = matching_delimiter(source, prefix.len() - 1, b'(', b')') else {
            continue;
        };
        let argument = &source[prefix.len()..close];
        // The read bridge takes exactly the instance it dereferences: a second top-level
        // argument is a shape this rewrite does not state.
        if top_level_commas(argument) {
            continue;
        }
        candidates.push((
            segment.len(),
            segment.start(),
            segment.start() + close + 1,
            argument.to_owned(),
        ));
    }
    candidates.sort_unstable();
    let Some((_, start, end, argument)) = candidates.first().cloned() else {
        return Ok(None);
    };
    if candidates
        .iter()
        .take_while(|candidate| candidate.0 == candidates[0].0)
        .any(|candidate| (candidate.1, candidate.2) != (start, end))
    {
        return Ok(None);
    }
    Ok(Some((start, end, format!("{argument}.{}", field_name))))
}

pub fn fold_capture_write_statement_span(
    method: &PhysicalMethodId,
    recovery: &crate::RecoveryReport,
    write_bci: u32,
    budget: &mut Budget,
) -> Result<Option<(usize, usize)>> {
    let mut candidates = Vec::new();
    for segment in recovery.projection_origins() {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if segment.origin().primary().method() != Some(method)
            || segment.origin().primary().bci() != write_bci
            || segment.mentions(write_bci) != Some(jarde_java::Provenance::Direct)
        {
            continue;
        }
        let source = segment.text(&recovery.text);
        if source.matches('\n').count() != 1
            || !source.ends_with('\n')
            || !source.trim_end().ends_with(';')
        {
            continue;
        }
        candidates.push((segment.len(), segment.start(), segment.end()));
    }
    candidates.sort_unstable();
    let Some((_, start, end)) = candidates.first().copied() else {
        return Ok(None);
    };
    if candidates
        .iter()
        .take_while(|candidate| candidate.0 == candidates[0].0)
        .any(|candidate| (candidate.1, candidate.2) != (start, end))
    {
        return Ok(None);
    }
    Ok(Some((start, end)))
}

pub fn family_recovery_token_span(
    method: &PhysicalMethodId,
    recovery: &crate::RecoveryReport,
    bci: u32,
    primary_bci: u32,
    needle: &str,
    budget: &mut Budget,
) -> Result<Option<(usize, usize)>> {
    let mut candidates = Vec::new();
    for segment in recovery
        .projection_origins()
        .iter()
        .filter(|segment| segment.mentions(bci).is_some())
    {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if segment.origin().primary().method() != Some(method)
            || segment.origin().primary().bci() != primary_bci
            || segment.mentions(bci)
                != Some(if bci == primary_bci {
                    jarde_java::Provenance::Direct
                } else {
                    jarde_java::Provenance::Derived
                })
        {
            continue;
        }
        let source = segment.text(&recovery.text);
        let mut matches = source.match_indices(needle);
        let Some((at, _)) = matches.next() else {
            continue;
        };
        if matches.next().is_some() {
            continue;
        }
        candidates.push((
            segment.len(),
            segment.start() + at,
            segment.start() + at + needle.len(),
        ));
    }
    candidates.sort_unstable();
    let Some((shortest, start, end)) = candidates.first().copied() else {
        return Ok(None);
    };
    if candidates
        .iter()
        .take_while(|candidate| candidate.0 == shortest)
        .any(|candidate| (candidate.1, candidate.2) != (start, end))
    {
        return Ok(None);
    }
    Ok(Some((start, end)))
}

// Same-run body-family constructor and initializer-prefix Code policy. No resolver or I/O.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingEnumConstructorEdge {
    pub caller: PhysicalMethodId,
    pub call_bci: u32,
    pub target_owner: Vec<u8>,
    pub target_descriptor: Vec<u8>,
    /// The instance String field the proved constructor body stores its one source argument
    /// into: the `putfield` target's field-table index with the raw name and descriptor the
    /// bytecode names. Carried only by the String-argument shape's private constructor edge;
    /// every other edge proves no field identity.
    pub assigned_field: Option<PendingEnumAssignedField>,
}

/// The proved `putfield` target of the String-argument constructor: one identity, resolved to
/// this class's own field-table index while the physical records are still at hand, so the
/// emission stage locates the same field without matching by name a second time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingEnumAssignedField {
    pub index: u64,
    pub name: Vec<u8>,
    pub descriptor: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingEnumConstantBodyInitializerPrefix {
    pub constant_field_write_bcis: Vec<u32>,
    pub values_factory_element_bcis: Vec<u32>,
    pub values_factory_call_bci: u32,
    pub values_field_write_bci: u32,
    pub prefix_end_bci: u32,
    /// The proved user argument tail of each constant step, empty exactly when that constant's
    /// constructor takes no user arguments. Spellings follow the arbitrary-argument channel.
    pub source_arguments: Vec<Vec<ProvedEnumUserArgument>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingEnumConstantBodyConstant {
    pub field_index: u64,
    pub field_name: Vec<u8>,
    pub expected_ordinal: u32,
    pub allocation_bci: u32,
    pub constructor_bci: u32,
    pub allocation_owner: Vec<u8>,
    pub constructor_owner: Vec<u8>,
    pub constructor_descriptor: Vec<u8>,
    /// The classified user parameter tail parsed from the constant's shared constructor
    /// descriptor, empty exactly for the fixed `(Ljava/lang/String;I)V` shape.
    pub source_parameters: Vec<EnumUserParameter>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingEnumConstantBodyMember {
    pub table_index: u64,
    pub name: Vec<u8>,
    pub descriptor: Vec<u8>,
    pub access_flags: u16,
    pub has_code: bool,
    /// The raw object type in a javac enum access-constructor descriptor, when this is that record.
    pub access_marker_owner: Option<Vec<u8>>,
}

/// Check the entire straight-line constructor body, not just its `invokespecial`. The
/// positional loads establish that neither name nor ordinal nor any user parameter was replaced;
/// the optional `aconst_null` is the only value allowed in the marker slot. Each proved user
/// parameter is forwarded from its own local, exactly as the constant step passed it. No other
/// effect can hide here.
pub fn prove_enum_constructor_instructions(
    instructions: &[EnumCodeInstruction],
    expected_owner: &[u8],
    expected_descriptor: &[u8],
    parameters: &[EnumUserParameter],
    marker: bool,
) -> std::result::Result<u32, String> {
    let refusal =
        || "the constructor does not forward unchanged name/ordinal on a pure edge".to_owned();
    // One expected step per instruction: (opcode, width, local). The name/ordinal locals are the
    // VM-injected pair; every user parameter loads its own slot; the marker slot (when the
    // target is an access constructor) takes only `aconst_null`.
    let mut steps: Vec<(u8, u32, Option<u16>)> =
        vec![(0x2a, 1, Some(0)), (0x2b, 1, Some(1)), (0x1c, 1, Some(2))];
    for (index, parameter) in parameters.iter().enumerate() {
        let Some(slot) = u16::try_from(3 + index).ok() else {
            return Err(refusal());
        };
        let kind = match parameter {
            EnumUserParameter::IntLiteral(_) => b'i',
            EnumUserParameter::StringLiteral | EnumUserParameter::Object { .. } => b'a',
        };
        let (wide_opcode, wide_width, narrow_opcode) = if kind == b'i' {
            (0x15, 2, 0x1a)
        } else {
            (0x19, 2, 0x2a)
        };
        if slot <= 3 {
            steps.push((narrow_opcode + slot as u8, 1, Some(slot)));
        } else {
            steps.push((wide_opcode, wide_width, Some(slot)));
        }
    }
    if marker {
        steps.push((0x01, 1, None));
    }
    steps.push((0xb7, 3, None));
    steps.push((0xb1, 1, None));
    if instructions.len() != steps.len() {
        return Err(refusal());
    }
    let mut next_bci = 0;
    for (instruction, expected) in instructions.iter().zip(&steps) {
        let (opcode, width, local) = *expected;
        if instruction.bci != next_bci
            || instruction.opcode != opcode
            || instruction.width != width
            || instruction.immediate.is_some()
            || (instruction.opcode != 0xb7 && instruction.reference.is_some())
            || instruction.local != local
        {
            return Err(refusal());
        }
        next_bci = next_bci
            .checked_add(instruction.width)
            .ok_or_else(refusal)?;
    }
    let call = &instructions[instructions.len() - 2];
    if !matches!(&call.reference,
        Some(EnumCodeReference::Method { owner, name, descriptor, interface: false })
            if owner == expected_owner && name == b"<init>" && descriptor == expected_descriptor)
    {
        return Err(refusal());
    }
    Ok(call.bci)
}

pub fn prove_enum_constructor_candidate(
    candidates: &[EnumMethodCodeCandidate],
    member: &PendingEnumConstantBodyMember,
    identity: &PhysicalMethodId,
    target_owner: &[u8],
    target_descriptor: &[u8],
    parameters: &[EnumUserParameter],
) -> std::result::Result<PendingEnumConstructorEdge, String> {
    let matching: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.table_index == member.table_index)
        .collect();
    let [code] = matching.as_slice() else {
        return Err("the selected main constructor has no unique same-run Code".to_owned());
    };
    if code.member.as_ref() != Some(identity)
        || !member.has_code
        || !code.complete
        || code.exception_handler_count != 0
    {
        return Err("the selected main constructor Code is incomplete".to_owned());
    }
    let call_bci = prove_enum_constructor_instructions(
        &code.instructions,
        target_owner,
        target_descriptor,
        parameters,
        false,
    )?;
    Ok(PendingEnumConstructorEdge {
        caller: identity.clone(),
        call_bci,
        target_owner: target_owner.to_vec(),
        target_descriptor: target_descriptor.to_vec(),
        assigned_field: None,
    })
}

/// Prove the one DT-12 source constructor: enum name/ordinal go unchanged to Enum, then the
/// sole source String argument is stored into the one String instance field the `putfield`
/// names. The written field's own identity — field-table index with raw name and descriptor —
/// is resolved here from this class's physical field records and carried on the edge; the
/// emission stage owns the source-level presentation checks against that same index.
pub fn prove_enum_string_constructor_candidate(
    candidates: &[EnumMethodCodeCandidate],
    member: &PendingEnumConstantBodyMember,
    identity: &PhysicalMethodId,
    enum_owner: &[u8],
    fields: &[jarde_reader::classfile::MemberHeader],
) -> std::result::Result<PendingEnumConstructorEdge, String> {
    use EnumCodeReference;
    let matches: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.table_index == member.table_index)
        .collect();
    let [code] = matches.as_slice() else {
        return Err("the selected String constructor has no unique same-run Code".to_owned());
    };
    let instructions = &code.instructions;
    if code.member.as_ref() != Some(identity)
        || !member.has_code
        || !code.complete
        || code.exception_handler_count != 0
        || instructions.len() != 8
    {
        return Err("the selected String constructor Code is incomplete".to_owned());
    }
    let expected = [0x2a, 0x2b, 0x1c, 0xb7, 0x2a, 0x2d, 0xb5, 0xb1];
    let mut next_bci = 0u32;
    for (index, instruction) in instructions.iter().enumerate() {
        if instruction.bci != next_bci
            || instruction.opcode != expected[index]
            || instruction.width
                != if instruction.opcode == 0xb7 || instruction.opcode == 0xb5 {
                    3
                } else {
                    1
                }
            || instruction.immediate.is_some()
            || instruction.local
                != match index {
                    0 => Some(0),
                    1 => Some(1),
                    2 => Some(2),
                    4 => Some(0),
                    5 => Some(3),
                    _ => None,
                }
        {
            return Err(
                "the String constructor is not a pure single-argument field store".to_owned(),
            );
        }
        next_bci = next_bci
            .checked_add(instruction.width)
            .ok_or_else(|| "constructor BCI overflow".to_owned())?;
    }
    if !matches!(&instructions[3].reference,
        Some(EnumCodeReference::Method { owner, name, descriptor, interface: false })
            if owner == b"java/lang/Enum" && name == b"<init>" && descriptor == b"(Ljava/lang/String;I)V")
    {
        return Err(
            "the String constructor does not preserve Enum and the assigned field semantics"
                .to_owned(),
        );
    }
    let Some(EnumCodeReference::Field {
        owner,
        name,
        descriptor,
    }) = &instructions[6].reference
    else {
        return Err(
            "the String constructor does not preserve Enum and the assigned field semantics"
                .to_owned(),
        );
    };
    if owner.as_slice() != enum_owner || descriptor.as_slice() != b"Ljava/lang/String;" {
        return Err(
            "the String constructor does not preserve Enum and the assigned field semantics"
                .to_owned(),
        );
    }
    // One physical field records this write target. The match keys on the written name and
    // descriptor and skips the static and enum-flagged records, exactly like the arbitrary
    // tail's own store proof; the source-level presentation checks stay at the emission stage.
    let matching: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| {
            field.name.raw().0 == name.as_slice()
                && field.descriptor.raw().0 == descriptor.as_slice()
                && field.access_flags & (0x0008 | 0x4000) == 0
        })
        .map(|(index, _)| index)
        .collect();
    let [field_index] = matching.as_slice() else {
        return Err(
            "the String constructor's assigned field target is absent or ambiguous".to_owned(),
        );
    };
    Ok(PendingEnumConstructorEdge {
        caller: identity.clone(),
        call_bci: instructions[3].bci,
        target_owner: b"java/lang/Enum".to_vec(),
        target_descriptor: b"(Ljava/lang/String;I)V".to_vec(),
        assigned_field: Some(PendingEnumAssignedField {
            index: u64::try_from(*field_index).unwrap_or(u64::MAX),
            name: name.clone(),
            descriptor: descriptor.clone(),
        }),
    })
}

fn prove_enum_body_values_factory(
    code: &EnumMethodCodeCandidate,
    owner: &[u8],
    constants: &[PendingEnumConstantBodyConstant],
) -> std::result::Result<Vec<u32>, String> {
    let refused = || "$values() does not return the constructed constants in order".to_owned();
    let instructions = &code.instructions;
    let expected_length = 2 + constants.len() * 4 + 1;
    if !code.complete || code.exception_handler_count != 0 || instructions.len() != expected_length
    {
        return Err(refused());
    }
    let mut next_bci = 0;
    for instruction in instructions {
        if instruction.bci != next_bci || instruction.width == 0 {
            return Err(refused());
        }
        next_bci = instruction
            .bci
            .checked_add(instruction.width)
            .ok_or_else(refused)?;
    }
    if !int_constant_at(instructions, 0, constants.len() as i32)
        || !matches!(&instructions[1].reference,
            Some(EnumCodeReference::Class(actual))
                if instructions[1].opcode == 0xbd && actual == owner)
        || instructions[expected_length - 1].opcode != 0xb0
    {
        return Err(refused());
    }
    let mut field_read_bcis = Vec::with_capacity(constants.len());
    for (ordinal, constant) in constants.iter().enumerate() {
        let start = 2 + ordinal * 4;
        if instructions[start].opcode != 0x59
            || !int_constant_at(instructions, start + 1, ordinal as i32)
            || !matches!(&instructions[start + 2].reference,
                Some(EnumCodeReference::Field { owner: field_owner, name, descriptor })
                    if instructions[start + 2].opcode == 0xb2
                        && field_owner == owner
                        && name == &constant.field_name
                        && descriptor == &[b"L".as_slice(), owner, b";"].concat())
            || instructions[start + 3].opcode != 0x53
        {
            return Err(refused());
        }
        field_read_bcis.push(instructions[start + 2].bci);
    }
    Ok(field_read_bcis)
}

/// The fixed Java 8 constant prefix leaves exactly one allocation reference after each
/// constructor call. The adjacent `putstatic` consumes it; no other opcode can copy or store it.
pub fn prove_enum_body_initializer_prefix(
    code: &EnumMethodCodeCandidate,
    factory: Option<&EnumMethodCodeCandidate>,
    owner: &[u8],
    constants: &[PendingEnumConstantBodyConstant],
    values_field: &PendingEnumConstantBodyMember,
) -> std::result::Result<PendingEnumConstantBodyInitializerPrefix, String> {
    let refused = || "the same-run <clinit> constant prefix is not exact".to_owned();
    if !code.complete || code.exception_handler_count != 0 || constants.is_empty() {
        return Err(refused());
    }
    let instructions = &code.instructions;
    let mut next_bci = 0;
    for instruction in instructions {
        if instruction.bci != next_bci || instruction.width == 0 {
            return Err("the <clinit> Code has a gap or overlapping instruction".to_owned());
        }
        next_bci = instruction
            .bci
            .checked_add(instruction.width)
            .ok_or_else(refused)?;
    }
    let plain = |instruction: Option<&EnumCodeInstruction>, opcode| {
        instruction.is_some_and(|instruction| {
            instruction.opcode == opcode
                && instruction.reference.is_none()
                && instruction.immediate.is_none()
                && instruction.local.is_none()
        })
    };
    let field = |instruction: Option<&EnumCodeInstruction>, name: &[u8], descriptor: &[u8]| {
        instruction.is_some_and(|instruction| {
            instruction.opcode == 0xb3
                && matches!(&instruction.reference,
                    Some(EnumCodeReference::Field { owner: actual_owner, name: actual_name, descriptor: actual_descriptor })
                        if actual_owner == owner && actual_name == name && actual_descriptor == descriptor)
        })
    };
    let ordinal = |instruction: Option<&EnumCodeInstruction>, expected: u32| {
        let Some(instruction) = instruction else {
            return false;
        };
        let actual = match instruction.opcode {
            0x02..=0x08 => Some(i32::from(instruction.opcode) - 3),
            0x10 | 0x11 => match instruction.immediate {
                Some(jarde_reader::classfile::ImmediateValue::Int(value)) => Some(value),
                _ => None,
            },
            0x12 | 0x13 => match instruction.reference {
                Some(EnumCodeReference::Integer(value)) => Some(value),
                _ => None,
            },
            _ => None,
        };
        actual == i32::try_from(expected).ok()
    };
    let mut cursor = 0;
    let mut field_write_bcis = Vec::with_capacity(constants.len());
    let mut source_arguments = Vec::with_capacity(constants.len());
    let enum_descriptor = [b"L".as_slice(), owner, b";"].concat();
    for constant in constants {
        let width = 6 + constant.source_parameters.len();
        let part = instructions
            .get(cursor..cursor + width)
            .ok_or_else(refused)?;
        if part[0].bci != constant.allocation_bci
            || !matches!(&part[0].reference, Some(EnumCodeReference::Class(actual))
                if part[0].opcode == 0xbb && actual == &constant.allocation_owner)
            || !plain(Some(&part[1]), 0x59)
            || !matches!(&part[2].reference, Some(EnumCodeReference::String(actual))
                if matches!(part[2].opcode, 0x12 | 0x13) && actual == &constant.field_name)
            || !ordinal(Some(&part[3]), constant.expected_ordinal)
        {
            return Err(refused());
        }
        // Each user argument sits at its own step position, proved exactly as the constructor's
        // parameter descriptor classifies it (the shared arbitrary-argument channel).
        let mut arguments = Vec::with_capacity(constant.source_parameters.len());
        let mut argument_offset = 4;
        for parameter in &constant.source_parameters {
            let Some((length, argument)) = prove_user_source_argument(
                instructions,
                cursor + argument_offset,
                EnumUserArgumentExpectation::of_parameter(parameter),
            ) else {
                return Err(refused());
            };
            arguments.push(argument);
            argument_offset += length;
        }
        let constructor = &part[width - 2];
        let field_write = &part[width - 1];
        if argument_offset != width - 2
            || constructor.bci != constant.constructor_bci
            || !matches!(&constructor.reference,
                Some(EnumCodeReference::Method { owner: actual_owner, name, descriptor, interface: false })
                    if constructor.opcode == 0xb7
                        && actual_owner == &constant.constructor_owner
                        && name == b"<init>"
                        && descriptor == &constant.constructor_descriptor)
            || !field(Some(field_write), &constant.field_name, &enum_descriptor)
        {
            return Err(refused());
        }
        field_write_bcis.push(field_write.bci);
        source_arguments.push(arguments);
        cursor += width;
    }
    let factory_call = instructions.get(cursor).ok_or_else(refused)?;
    let values_descriptor = [b"()[L".as_slice(), owner, b";"].concat();
    if !matches!(&factory_call.reference,
        Some(EnumCodeReference::Method { owner: actual_owner, name, descriptor, interface: false })
            if factory_call.opcode == 0xb8 && actual_owner == owner && name == b"$values"
                && descriptor == &values_descriptor)
        || !field(
            instructions.get(cursor + 1),
            &values_field.name,
            &values_field.descriptor,
        )
    {
        return Err(refused());
    }
    let store = &instructions[cursor + 1];
    let prefix_end_bci = store.bci.checked_add(store.width).ok_or_else(refused)?;
    let factory = factory.ok_or_else(|| "the same-run $values() Code is absent".to_owned())?;
    let values_factory_element_bcis = prove_enum_body_values_factory(factory, owner, constants)?;
    let suffix = instructions.get(cursor + 2..).ok_or_else(refused)?;
    if !matches!(suffix.last(), Some(last) if plain(Some(last), 0xb1))
        || suffix.iter().any(|instruction| {
            // A transfer back into the prefix (or an early exit) would invalidate its
            // single-execution, empty-stack boundary. Re-reading a constant or `$VALUES`
            // could create another alias or store it elsewhere. Other suffix effects remain
            // available for the later whole-initializer proof.
            matches!(
                instruction.opcode,
                0x99..=0xa9 | 0xaa | 0xab | 0xbf | 0xc6 | 0xc7 | 0xc8 | 0xc9
            ) || (instruction.opcode == 0xb1 && instruction.bci != suffix.last().unwrap().bci)
                || matches!(&instruction.reference,
                    Some(EnumCodeReference::Field { owner: field_owner, name, .. })
                        if field_owner == owner
                            && (name == &values_field.name
                                || constants.iter().any(|constant| name == &constant.field_name)))
        })
    {
        return Err("the <clinit> suffix does not close after the $VALUES prefix".to_owned());
    }
    Ok(PendingEnumConstantBodyInitializerPrefix {
        constant_field_write_bcis: field_write_bcis,
        values_factory_element_bcis,
        values_factory_call_bci: factory_call.bci,
        values_field_write_bci: store.bci,
        prefix_end_bci,
        source_arguments,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingEnumConstantBodyGroupShape {
    pub enum_access_flags: u16,
    pub constants: Vec<PendingEnumConstantBodyConstant>,
    pub implicit_members: Vec<PendingEnumConstantBodyMember>,
    pub abstract_methods: Vec<PendingEnumConstantBodyMember>,
    /// The one Java 8 direct-interface abstract method used by the DT-12 slice when the enum
    /// inherits its required body signature instead of declaring it itself.
    pub interface_methods: Vec<PendingEnumConstantBodyMember>,
    pub constructors: Vec<PendingEnumConstantBodyMember>,
    /// The private constructor reaches java/lang/Enum unchanged; the optional synthetic
    /// access constructor reaches that private constructor unchanged and ignores its marker.
    pub constructor_chain: std::result::Result<Vec<PendingEnumConstructorEdge>, String>,
    /// The proved user tail of the shared private constructor, present exactly when the constant
    /// arguments came through the arbitrary grammar; `None` for the fixed zero/String shapes.
    pub constructor_user_tail: Option<ProvedEnumUserConstructorTail>,
    /// Ordinary constants use their own physical constructor without an access bridge.
    pub direct_constant_bcis: std::result::Result<Vec<u32>, String>,
    /// Exact same-run `<clinit>` prefix evidence; a refusal leaves the physical relations intact.
    pub initializer_prefix: std::result::Result<PendingEnumConstantBodyInitializerPrefix, String>,
}

/// Full body-family Java policy over already-selected borrowed facts.
/// Suppliers do not read or recover; the assembler retains its ClassSource-owned result types.
pub fn prove_enum_constant_body_group<'a, M: 'a, F: 'a, R: 'a, C>(
    declaration: (&[u8], &PhysicalDefinitionId, &str),
    source_fields: &'a [F],
    source_methods: &'a [M],
    codes: &[EnumMethodCodeCandidate],
    initializers: &[jarde_java::report::ClassInitializerCandidates],
    relations: &'a [R],
    method_facts: impl Fn(
        &'a M,
    ) -> (
        (
            u64,
            &'a PhysicalMethodId,
            &'a jarde_reader::model::JvmString,
            &'a jarde_reader::model::JvmString,
            bool,
            Option<jarde_jvm::ir::NoBodyKind>,
            bool,
            bool,
        ),
        Option<(&'a jarde_java::RecoveryReport, &'a ExecutionReport)>,
        (
            &'a [String],
            EnumConstructorSourceTail,
            bool,
            &'a jarde_java::report::MemberAnnotationUses,
            &'a jarde_java::report::ParameterAnnotationUses,
            &'a jarde_java::report::TypeAnnotationUses,
        ),
    ),
    field_facts: impl Fn(
        &'a F,
    ) -> (
        &'a PhysicalDefinitionId,
        &'a jarde_reader::model::JvmString,
        &'a jarde_reader::model::JvmString,
        u16,
        bool,
        &'a [String],
    ),
    relation_facts: impl Fn(
        &'a R,
    ) -> (
        &'a PendingEnumConstantBodyGroupShape,
        u64,
        u32,
        u32,
        &'a [u8],
        &'a [u8],
        &'a PhysicalDefinitionId,
        bool,
        bool,
        &'a std::result::Result<PendingEnumConstructorEdge, String>,
        Option<&'a std::result::Result<std::sync::Arc<[M]>, String>>,
    ),
    make_constant: impl Fn(
        u64,
        u32,
        u32,
        u32,
        &[ProvedEnumUserArgument],
        Option<&PhysicalDefinitionId>,
        Option<&std::sync::Arc<[M]>>,
    ) -> C,
    budget: &mut Budget,
) -> Result<
    std::result::Result<
        (
            Vec<C>,
            Option<String>,
            Option<ProvedEnumUserConstructorTail>,
        ),
        String,
    >,
> {
    let refuse = |reason: &str| Err(reason.to_owned());
    let Some(shape) = relations.first().map(|relation| relation_facts(relation).0) else {
        return Ok(refuse("the body group has no selected child"));
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(source_methods.len() + relations.len()).unwrap_or(u64::MAX),
    )?;
    let (owner, definition, declaration_name) = declaration;
    let (Ok(prefix), Ok(chain), Ok(direct_bcis)) = (
        &shape.initializer_prefix,
        &shape.constructor_chain,
        &shape.direct_constant_bcis,
    ) else {
        return Ok(refuse(
            "the pending initializer or constructor chain refused",
        ));
    };
    if shape.constants.is_empty()
        || shape.implicit_members.len() != 6
        || chain.is_empty()
        || prefix.constant_field_write_bcis.len() != shape.constants.len()
        || prefix.source_arguments.len() != shape.constants.len()
        || relations
            .iter()
            .any(|relation| relation_facts(relation).0 != shape)
    {
        return Ok(refuse("the pending constant-body group is incomplete"));
    }
    // The two fixed two-constant slices keep their proven scope; the mixed group covers the
    // remaining shapes where every constant carries its proved arbitrary user tail.
    let string_arguments = shape.constants.len() == 2
        && shape.constants.iter().all(|constant| {
            constant.source_parameters.len() == 1
                && constant.constructor_descriptor == b"(Ljava/lang/String;ILjava/lang/String;)V"
        });
    let zero_arguments = shape.constants.len() == 2
        && shape.constants.iter().all(|constant| {
            constant.source_parameters.is_empty()
                && constant.constructor_descriptor == b"(Ljava/lang/String;I)V"
        });
    let arbitrary = !string_arguments
        && !zero_arguments
        && shape
            .constants
            .iter()
            .all(|constant| !constant.source_parameters.is_empty());
    if !string_arguments && !zero_arguments && !arbitrary {
        return Ok(refuse(
            "the constants do not share one supported constructor shape",
        ));
    }
    if string_arguments
        && (shape.enum_access_flags & 0x0400 == 0
            || shape.interface_methods.len() != 1
            || shape
                .constants
                .iter()
                .any(|constant| constant.allocation_owner == owner))
    {
        return Ok(refuse(
            "the String-argument slice requires two abstract-enum anonymous bodies",
        ));
    }
    // The single-String slice's ASCII gate lives in the constant-step proof: its proved
    // argument is exactly one ASCII-decoded String literal or the group refuses there.

    let code_at = |index: u64| -> Option<&EnumMethodCodeCandidate> {
        let matching: Vec<_> = codes
            .iter()
            .filter(|code| code.table_index == index)
            .collect();
        let [code] = matching.as_slice() else {
            return None;
        };
        let source = source_methods.get(usize::try_from(index).ok()?)?;
        let ((_, identity, ..), _, _) = method_facts(source);
        (code.complete && code.member.as_ref() == Some(identity)).then_some(*code)
    };
    for method in source_methods {
        budget.poll()?;
        let ((index, _, _, _, has_code, ..), _, _) = method_facts(method);
        if has_code && code_at(index).is_none() {
            return Ok(refuse(
                "a main enum Code member has no complete same-run candidate",
            ));
        }
    }
    let [
        backing,
        initializer,
        values,
        value_of,
        factory,
        private_constructor,
    ] = shape.implicit_members.as_slice()
    else {
        return Ok(refuse("the implicit member set is incomplete"));
    };
    if backing.name != b"$VALUES"
        || initializer.name != b"<clinit>"
        || values.name != b"values"
        || value_of.name != b"valueOf"
        || factory.name != b"$values"
        || private_constructor.name != b"<init>"
    {
        return Ok(refuse("the pending implicit members changed"));
    }
    let Some(values_code) = code_at(values.table_index) else {
        return Ok(refuse("values Code is absent"));
    };
    let Some(value_of_code) = code_at(value_of.table_index) else {
        return Ok(refuse("valueOf Code is absent"));
    };
    if values_code.exception_handler_count != 0
        || value_of_code.exception_handler_count != 0
        || !prove_values(values_code, owner, b"$VALUES")
        || !prove_value_of(value_of_code, owner)
    {
        return Ok(refuse(
            "the generated enum API does not preserve its standard behavior",
        ));
    }
    let Some(clinit) = code_at(initializer.table_index) else {
        return Ok(refuse("initializer Code is absent"));
    };
    let suffix: Vec<_> = clinit
        .instructions
        .iter()
        .filter(|instruction| instruction.bci >= prefix.prefix_end_bci)
        .collect();
    if clinit.exception_handler_count != 0
        || !matches!(suffix.as_slice(), [end]
            if end.bci == prefix.prefix_end_bci && end.width == 1 && end.opcode == 0xb1
                && end.reference.is_none() && end.immediate.is_none() && end.local.is_none())
    {
        return Ok(refuse(
            "the initializer has effects after the proved constant prefix",
        ));
    }
    let Some(clinit_source) = usize::try_from(initializer.table_index)
        .ok()
        .and_then(|index| source_methods.get(index))
    else {
        return Ok(refuse("the initializer has no physical method record"));
    };
    let ((_, clinit_identity, ..), recovery, _) = method_facts(clinit_source);
    if !matches!(recovery,
        Some((report, execution))
            if report.produced() && report.quality == jarde_jvm::ir::Quality::Structured
                && report.fallbacks.is_empty()
                && matches!(report.execution, ExecutionReport::Complete { .. })
                && matches!(execution, ExecutionReport::Complete { .. }))
    {
        return Ok(refuse(
            "the initializer has no complete structured recovery",
        ));
    }
    let matching: Vec<_> = initializers
        .iter()
        .filter(|candidate| candidate.member.as_ref() == Some(clinit_identity))
        .collect();
    let [sidecar] = matching.as_slice() else {
        return Ok(refuse("the initializer has no unique structured sidecar"));
    };
    if sidecar.has_exception_handlers
        || sidecar.steps.len() != shape.constants.len() + 2
        || !matches!(&sidecar.steps[shape.constants.len() + 1],
            jarde_java::report::ClassInitializerStep::Other {
                order,
                bci,
                kind: jarde_java::report::ClassInitializerStatementKind::Return,
            } if *order == shape.constants.len() + 1 && *bci == prefix.prefix_end_bci)
    {
        return Ok(refuse(
            "the initializer sidecar does not close after the constant writes",
        ));
    }
    for (order, bci) in prefix
        .constant_field_write_bcis
        .iter()
        .copied()
        .chain(std::iter::once(prefix.values_field_write_bci))
        .enumerate()
    {
        let jarde_java::report::ClassInitializerStep::FieldWrite(write) = &sidecar.steps[order]
        else {
            return Ok(refuse("the initializer sidecar omits a field write"));
        };
        let name = if order < shape.constants.len() {
            &shape.constants[order].field_name
        } else {
            &backing.name
        };
        let descriptor = if order < shape.constants.len() {
            [b"L".as_slice(), owner, b";"].concat()
        } else {
            [b"[L".as_slice(), owner, b";"].concat()
        };
        if write.order != order
            || write.bci != bci
            || !write.is_static
            || write.owner.0.as_slice() != owner
            || write.name.0.as_slice() != name
            || write.descriptor.0.as_slice() != descriptor
            || write.op != jarde_java::ast::AssignOp::Assign
            || write
                .field_reads
                .as_ref()
                .is_none_or(|reads| !reads.is_empty())
        {
            return Ok(refuse(
                "an initializer write has unproved structured effects",
            ));
        }
    }
    for abstract_member in &shape.abstract_methods {
        let Some(source) = usize::try_from(abstract_member.table_index)
            .ok()
            .and_then(|index| source_methods.get(index))
        else {
            return Ok(refuse("an abstract enum declaration is absent"));
        };
        let ((_, _, name, descriptor, _, no_body_kind, no_body, declared), _, _) =
            method_facts(source);
        if name.raw().0 != abstract_member.name
            || descriptor.raw().0 != abstract_member.descriptor
            || abstract_member.access_flags & (0x0002 | 0x0008 | 0x0010 | 0x0040 | 0x0100 | 0x1000)
                != 0
            || abstract_member.has_code
            || !matches!(no_body_kind, Some(jarde_jvm::ir::NoBodyKind::Abstract))
            || !no_body
            || !declared
        {
            return Ok(refuse("an abstract enum declaration is not complete"));
        }
    }
    for code in codes {
        if shape
            .implicit_members
            .iter()
            .any(|member| member.table_index == code.table_index)
        {
            continue;
        }
        if code
            .member_uses
            .iter()
            .any(|use_site| match &use_site.reference {
                EnumCodeReference::Field {
                    owner: use_owner,
                    name,
                    ..
                } => use_owner == owner && name == b"$VALUES",
                EnumCodeReference::Method {
                    owner: use_owner,
                    name,
                    ..
                } => use_owner == owner && name == b"$values",
                _ => false,
            })
        {
            return Ok(refuse("a user method uses an implicit enum member"));
        }
    }

    let mut proved = Vec::with_capacity(shape.constants.len());
    let mut selected_children = std::collections::HashSet::new();
    for (ordinal, constant) in shape.constants.iter().enumerate() {
        budget.poll()?;
        let Some(field) = usize::try_from(constant.field_index)
            .ok()
            .and_then(|index| source_fields.get(index))
        else {
            return Ok(refuse("an enum constant field is absent"));
        };
        let (field_owner, field_name, _, _, field_declared, field_markers) = field_facts(field);
        let Some(name) = String::from_utf16(field_name.utf16()).ok() else {
            return Ok(refuse("an enum constant name is not Java text"));
        };
        if field_owner != definition
            || field_name.raw().0 != constant.field_name
            || !field_declared
            || !field_markers.is_empty()
            || !jarde_java::is_java_identifier(&name)
            || constant.expected_ordinal != ordinal as u32
            || constant.constructor_descriptor != shape.constants[0].constructor_descriptor
            || constant.constructor_owner != constant.allocation_owner
        {
            return Ok(refuse("an ordered constant is not presentable"));
        }
        let matching: Vec<_> = relations
            .iter()
            .filter(|relation| relation_facts(relation).1 == constant.field_index)
            .collect();
        let (subclass, methods) = if constant.allocation_owner == owner {
            if !matching.is_empty() || !direct_bcis.contains(&constant.constructor_bci) {
                return Ok(refuse("a direct constant has an unexpected child relation"));
            }
            (None, None)
        } else {
            let [relation] = matching.as_slice() else {
                return Ok(refuse("an anonymous constant has no unique selected child"));
            };
            let (
                _,
                _,
                allocation_bci,
                constructor_bci,
                descriptor,
                subclass_owner,
                subclass,
                exclusive,
                has_refusal,
                constructor_bridge,
                body_proof,
            ) = relation_facts(relation);
            if allocation_bci != constant.allocation_bci
                || constructor_bci != constant.constructor_bci
                || descriptor != constant.constructor_descriptor
                || subclass_owner != constant.allocation_owner
                || !exclusive
                || has_refusal
                || constructor_bridge.is_err()
            {
                return Ok(refuse(
                    "the selected child use or constructor proof is incomplete",
                ));
            }
            let Some(Ok(methods)) = body_proof else {
                return Ok(refuse("a selected child lacks complete override bodies"));
            };
            if shape.abstract_methods.iter().any(|abstract_member| {
                methods
                    .iter()
                    .filter(|method| {
                        let ((_, _, name, descriptor, _, _, _, declared), recovery, (markers, ..)) =
                            method_facts(method);
                        name.raw().0 == abstract_member.name
                            && descriptor.raw().0 == abstract_member.descriptor
                            && recovery.is_some_and(|(report, execution)| {
                                enum_constant_child_recovery_is_supported(
                                    report,
                                    execution,
                                    declared,
                                    markers.is_empty(),
                                )
                            })
                    })
                    .count()
                    != 1
            }) {
                return Ok(refuse(
                    "a constant body does not implement every abstract enum method",
                ));
            }
            if !selected_children.insert(subclass) {
                return Ok(refuse("one selected child is assigned to two constants"));
            }
            (Some(subclass), Some(methods))
        };
        if subclass.is_none() && !shape.abstract_methods.is_empty() {
            return Ok(refuse("an abstract enum has a constant without a body"));
        }
        proved.push(make_constant(
            constant.field_index,
            constant.allocation_bci,
            constant.constructor_bci,
            prefix.constant_field_write_bcis[ordinal],
            &prefix.source_arguments[ordinal],
            subclass,
            methods,
        ));
    }
    if relations.len() != selected_children.len() {
        return Ok(refuse(
            "the child relations do not cover exactly the anonymous constants",
        ));
    }
    let constructor_text = if string_arguments {
        let constructor = shape
            .constructors
            .iter()
            .find(|member| member.descriptor == b"(Ljava/lang/String;ILjava/lang/String;)V");
        let Some(constructor) = constructor else {
            return Ok(refuse("the source String constructor is absent"));
        };
        let Some(source) = usize::try_from(constructor.table_index)
            .ok()
            .and_then(|index| source_methods.get(index))
        else {
            return Ok(refuse("the source String constructor record is absent"));
        };
        let (
            (source_index, _, source_name, _, _, _, _, declared),
            _,
            (
                markers,
                source_tail,
                signature_refused,
                annotations,
                parameter_annotations,
                type_annotations,
            ),
        ) = method_facts(source);
        let has_only_enum_signature_marker = signature_refused && markers.len() == 1;
        if constructor.access_flags != 0x0002
            || source_index != constructor.table_index
            || source_name.raw().0 != b"<init>"
            || source_tail != EnumConstructorSourceTail::SingleString
            || !declared
            || !signature_refused
            || !annotations.refusals.is_empty()
            || !annotations.uses.is_empty()
            || !parameter_annotations.refusals.is_empty()
            || parameter_annotations
                .uses_by_position
                .iter()
                .any(|uses| !uses.is_empty())
            || !type_annotations.refusals.is_empty()
            || type_annotations
                .parameter_uses
                .iter()
                .any(|uses| !uses.is_empty())
            || (!markers.is_empty() && !has_only_enum_signature_marker)
        {
            return Ok(refuse(
                "the source String constructor signature is not safely presentable",
            ));
        }
        // The proof carried the written field's identity from the constructor bytecode. The
        // source field at that index owns the presentation checks — the same attribute filter
        // as before, now keyed on the proved position instead of one hardcoded name — and the
        // emitted text spells that field's own declaration name.
        let Some(assigned) = chain.first().and_then(|edge| edge.assigned_field.as_ref()) else {
            return Ok(refuse(
                "the assigned String field is not uniquely presentable",
            ));
        };
        let Some(field) = usize::try_from(assigned.index)
            .ok()
            .and_then(|index| source_fields.get(index))
        else {
            return Ok(refuse(
                "the assigned String field is not uniquely presentable",
            ));
        };
        let (field_owner, field_name, field_descriptor, field_flags, field_declared, field_markers) =
            field_facts(field);
        if field_name.raw().0 != assigned.name.as_slice()
            || field_descriptor.raw().0 != assigned.descriptor.as_slice()
            || field_descriptor.raw().0 != b"Ljava/lang/String;"
            || field_owner != definition
            || field_flags & (0x0008 | 0x0040 | 0x1000) != 0
            || field_flags & 0x0002 == 0
            || !field_declared
            || !field_markers.is_empty()
        {
            return Ok(refuse(
                "the assigned String field is not uniquely presentable",
            ));
        }
        if source_fields
            .iter()
            .filter(|field| {
                let (_, name, descriptor, ..) = field_facts(field);
                name.raw().0 == assigned.name.as_slice()
                    && descriptor.raw().0 == assigned.descriptor.as_slice()
            })
            .count()
            != 1
        {
            return Ok(refuse("the source String field is ambiguous"));
        }
        let Some(assigned_name) = String::from_utf16(field_name.utf16()).ok() else {
            return Ok(refuse("the assigned String field name is not Java text"));
        };
        if !jarde_java::is_java_identifier(&assigned_name) {
            return Ok(refuse("the assigned String field name is not Java text"));
        }
        Some(format!(
            "    private {}(java.lang.String arg0) {{\n        this.{assigned_name} = arg0;\n    }}\n",
            declaration_name
        ))
    } else {
        None
    };
    // The arbitrary tail's own proof owns the constructor shape; this gate only certifies that
    // the constructor record can be presented without inventing annotations or markers, exactly
    // like the ordinary user-tail projection.
    let constructor_user_tail = if arbitrary {
        let Some(tail) = shape.constructor_user_tail.clone() else {
            return Ok(refuse(
                "the arbitrary constructor tail is absent from the pending shape",
            ));
        };
        let constructor = shape
            .constructors
            .iter()
            .find(|member| member.descriptor == shape.constants[0].constructor_descriptor);
        let Some(constructor) = constructor else {
            return Ok(refuse("the source constructor is absent"));
        };
        let Some(source) = usize::try_from(constructor.table_index)
            .ok()
            .and_then(|index| source_methods.get(index))
        else {
            return Ok(refuse("the source constructor record is absent"));
        };
        let (
            (source_index, _, source_name, _, _, _, _, declared),
            _,
            (markers, _, _, annotations, parameter_annotations, type_annotations),
        ) = method_facts(source);
        let has_arbitrary_signature_marker = markers.len() <= 1
            && markers
                .first()
                .is_none_or(|marker| marker.contains("jvm_signature_erasure_mismatch"));
        if source_index != constructor.table_index
            || source_name.raw().0 != b"<init>"
            || !declared
            || !has_arbitrary_signature_marker
            || !annotations.refusals.is_empty()
            || !annotations.uses.is_empty()
            || !parameter_annotations.refusals.is_empty()
            || parameter_annotations
                .uses_by_position
                .iter()
                .any(|uses| !uses.is_empty())
            || !type_annotations.refusals.is_empty()
            || type_annotations
                .parameter_uses
                .iter()
                .any(|uses| !uses.is_empty())
        {
            return Ok(refuse(
                "the source constructor signature is not safely presentable",
            ));
        }
        Some(tail)
    } else {
        None
    };
    Ok(Ok((proved, constructor_text, constructor_user_tail)))
}

fn enum_body_physical_member(index: usize, header: &MemberHeader) -> PendingEnumConstantBodyMember {
    PendingEnumConstantBodyMember {
        table_index: u64::try_from(index).unwrap_or(u64::MAX),
        name: header.name.raw().0.clone(),
        descriptor: header.descriptor.raw().0.clone(),
        access_flags: header.access_flags,
        has_code: header
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code"),
        access_marker_owner: None,
    }
}

pub fn enum_access_constructor_marker_owner(descriptor: &[u8]) -> Option<Vec<u8>> {
    let tail = descriptor.strip_prefix(b"(Ljava/lang/String;I")?;
    // The descriptor is the proved user parameter tail plus one final marker parameter. Skip the
    // user parameters; the last one must be a plain `L…;` object type — the synthetic marker.
    let mut rest = tail;
    let mut last_marker: Option<Vec<u8>> = None;
    loop {
        if rest == b")V" {
            return last_marker;
        }
        let (parameter, remainder) = EnumUserParameter::parse(rest)?;
        last_marker = match &parameter {
            EnumUserParameter::Object { descriptor } => {
                let owner = descriptor.strip_prefix(b"L")?.strip_suffix(b";")?;
                if owner.is_empty() || owner.contains(&b';') || owner.contains(&b'[') {
                    return None;
                }
                Some(owner.to_vec())
            }
            _ => None,
        };
        rest = remainder;
    }
}

fn enum_constant_constructor_matches(
    constructor_owner: &[u8],
    constructor_name: &[u8],
    constructor_descriptor: &[u8],
    allocation_owner: &[u8],
) -> bool {
    constructor_owner == allocation_owner
        && constructor_name == b"<init>"
        && (constructor_descriptor == b"(Ljava/lang/String;I)V"
            || parse_arbitrary_ctor_descriptor(constructor_descriptor).is_some())
}

/// Qualify the complete main-class body-family candidate before any dependency read.
/// The tuple transfers existing local facts; it creates no retained candidate/report mirror.
pub fn prepare_enum_body_candidate<'a>(
    enum_owner: &[u8],
    enum_access_flags: u16,
    enum_interfaces: &'a [Vec<u8>],
    fields: &[MemberHeader],
    methods: &[MemberHeader],
    code_candidates: &'a [EnumMethodCodeCandidate],
    allocation_scans: &'a [(
        PhysicalMethodId,
        Option<jarde_java::report::AnonymousAllocationScan>,
    )],
    budget: &mut Budget,
) -> Result<
    Option<(
        &'a PhysicalMethodId,
        &'a EnumMethodCodeCandidate,
        Vec<(
            usize,
            &'a jarde_java::report::AnonymousAllocationCandidate,
            u32,
            Vec<u8>,
        )>,
        Vec<PendingEnumConstantBodyConstant>,
        Vec<PendingEnumConstantBodyMember>,
        Vec<u8>,
        Vec<EnumUserParameter>,
        Vec<PendingEnumConstantBodyMember>,
        bool,
        Vec<PendingEnumConstantBodyMember>,
        Option<&'a [u8]>,
    )>,
> {
    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_FINAL: u16 = 0x0010;
    const ACC_SYNTHETIC: u16 = 0x1000;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_ENUM: u16 = 0x4000;
    const BASE_CTOR: &[u8] = b"(Ljava/lang/String;I)V";

    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(methods.len()).unwrap_or(u64::MAX),
    )?;
    let clinit_headers: Vec<_> = methods
        .iter()
        .enumerate()
        .filter(|(_, method)| {
            method.name.raw().0 == b"<clinit>" && method.descriptor.raw().0 == b"()V"
        })
        .collect();
    let [(clinit_index, _clinit_header)] = clinit_headers.as_slice() else {
        return Ok(None);
    };
    let matching_code: Vec<_> = code_candidates
        .iter()
        .filter(|candidate| candidate.table_index == *clinit_index as u64)
        .collect();
    let [code] = matching_code.as_slice() else {
        return Ok(None);
    };
    let Some((clinit_identity, Some(scan))) = allocation_scans
        .iter()
        .find(|(identity, _)| identity.name.0 == b"<clinit>" && identity.descriptor.0 == b"()V")
    else {
        return Ok(None);
    };
    if code.member.as_ref() != Some(clinit_identity) || !code.complete || !scan.complete {
        return Ok(None);
    }

    // The scan lists every same-run `new` instruction (its `complete` flag states that); whether
    // its own construction-site plan verified each site is not an obligation of this proof — the
    // constant-step proof below re-derives and re-proves each construction from the same Code.
    let allocations: Vec<_> = scan
        .allocations
        .iter()
        .filter(|allocation| allocation.member == *clinit_identity)
        .collect();
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(allocations.len()).unwrap_or(u64::MAX),
    )?;
    if allocations.is_empty()
        || !allocations
            .iter()
            .any(|allocation| allocation.class.0.as_slice() != enum_owner)
    {
        return Ok(None);
    }
    let constant_descriptor = object_descriptor(enum_owner);
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(fields.len()).unwrap_or(u64::MAX),
    )?;
    let constants: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| {
            field.access_flags & 0x4000 != 0
                && field.descriptor.raw().0 == constant_descriptor.as_slice()
        })
        .collect();
    if constants.len() < 2
        || fields
            .iter()
            .filter(|field| field.access_flags & ACC_ENUM != 0)
            .count()
            != constants.len()
        || constants.iter().any(|(_, field)| {
            field.access_flags != (0x0001 | ACC_STATIC | ACC_FINAL | ACC_ENUM)
                || !field.attributes.is_empty()
        })
    {
        return Ok(None);
    }

    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
    )?;
    if allocations.len() != constants.len() {
        return Ok(None);
    }
    let mut constructions = Vec::with_capacity(constants.len());
    for allocation in allocations {
        budget.poll()?;
        // The scan shortlists every same-run `new` instruction; whether its construction-site plan
        // verified the arguments is not an obligation here. The constant-step proof below owns
        // the argument shapes, so the constructor call is re-derived from the same-run Code:
        // every admitted argument kind is one instruction, so the first `invokespecial` after the
        // allocation head is this construction's call, and the step proof binds it exactly.
        if !code.instructions.iter().any(|instruction| {
            instruction.bci == allocation.head_bci
                && instruction.opcode == 0xbb
                && instruction.reference
                    == Some(EnumCodeReference::Class(
                        allocation.class.0.as_slice().to_vec(),
                    ))
        }) {
            return Ok(None);
        }
        let head_position = code
            .instructions
            .iter()
            .position(|instruction| instruction.bci == allocation.head_bci)
            .expect("the allocation head instruction exists");
        let Some((constructor_position, constructor_bci)) = code
            .instructions
            .iter()
            .enumerate()
            .skip(head_position + 1)
            .find_map(|(position, instruction)| {
                (instruction.opcode == 0xb7).then_some((position, instruction.bci))
            })
        else {
            return Ok(None);
        };
        let Some(EnumCodeReference::Method {
            owner: constructor_owner,
            name,
            descriptor,
            interface: false,
        }) = code.instructions[constructor_position].reference.as_ref()
        else {
            return Ok(None);
        };
        if !enum_constant_constructor_matches(
            constructor_owner,
            name,
            descriptor,
            allocation.class.0.as_slice(),
        ) {
            return Ok(None);
        }
        // `invokespecial` must be followed by the one physical enum-field write. This makes
        // constructor BCI and field BCI a one-to-one pair before any child definition is read.
        let Some(field_write) = code.instructions.get(constructor_position + 1) else {
            return Ok(None);
        };
        let Some((field_index, _)) = constants.iter().find(|(_, field)| {
            matches!(&field_write.reference,
                Some(EnumCodeReference::Field { owner, name, descriptor })
                    if owner.as_slice() == enum_owner
                        && name.as_slice() == field.name.raw().0.as_slice()
                        && descriptor.as_slice() == field.descriptor.raw().0.as_slice())
        }) else {
            return Ok(None);
        };
        if field_write.opcode != 0xb3 {
            return Ok(None);
        }
        constructions.push((
            *field_index,
            allocation,
            constructor_bci,
            descriptor.clone(),
        ));
    }
    constructions.sort_by_key(|(_, allocation, _, _)| allocation.head_bci);
    if constructions
        .iter()
        .map(|(field_index, _, _, _)| *field_index)
        .ne(constants.iter().map(|(field_index, _)| *field_index))
    {
        return Ok(None);
    }

    // Preserve the entire ordered initializer shape and the physical implicit-member records
    // before resolving child definitions. These facts are intentionally only pending evidence:
    // the ordinary enum proof above still refuses zero-source-argument anonymous-owner groups.
    let mut constant_shape = Vec::with_capacity(constants.len());
    for (
        ordinal,
        ((field_index, field), (constructed_field_index, allocation, constructor_bci, descriptor)),
    ) in constants.iter().zip(&constructions).enumerate()
    {
        if *field_index != *constructed_field_index {
            return Ok(None);
        }
        // The shared constructor descriptor's user tail: the fixed zero-argument shape, or the
        // arbitrary grammar the ordinary channel already established. Anything else refuses.
        let source_parameters = if descriptor.as_slice() == BASE_CTOR {
            Vec::new()
        } else {
            match parse_arbitrary_ctor_descriptor(descriptor) {
                Some(parameters) => parameters,
                None => return Ok(None),
            }
        };
        let Some(constructor_owner) = code
            .instructions
            .iter()
            .find(|instruction| instruction.bci == *constructor_bci)
            .and_then(|instruction| instruction.reference.as_ref())
            .and_then(|reference| match reference {
                EnumCodeReference::Method { owner, .. } => Some(owner.clone()),
                _ => None,
            })
        else {
            return Ok(None);
        };
        if constructor_owner != allocation.class.0.as_slice() {
            return Ok(None);
        }
        constant_shape.push(PendingEnumConstantBodyConstant {
            field_index: u64::try_from(*field_index).unwrap_or(u64::MAX),
            field_name: field.name.raw().0.clone(),
            expected_ordinal: u32::try_from(ordinal).unwrap_or(u32::MAX),
            allocation_bci: allocation.head_bci,
            constructor_bci: *constructor_bci,
            allocation_owner: allocation.class.0.as_slice().to_vec(),
            constructor_owner,
            constructor_descriptor: descriptor.clone(),
            // This tail describes only the constructor descriptor after the VM-injected
            // name/ordinal pair; it does not prove the values passed at this call site.
            source_parameters,
        });
    }

    let unique_method_record = |name: &[u8], descriptor: &[u8]| {
        let matches: Vec<_> = methods
            .iter()
            .enumerate()
            .filter(|(_, method)| {
                method.name.raw().0 == name && method.descriptor.raw().0 == descriptor
            })
            .collect();
        match matches.as_slice() {
            [(index, method)] => Some(enum_body_physical_member(*index, method)),
            _ => None,
        }
    };
    let values_method_descriptor = values_descriptor(enum_owner);
    let value_of_method_descriptor = value_of_descriptor(enum_owner);
    let values_field: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| field.name.raw().0 == b"$VALUES")
        .collect();
    let [(values_field_index, values_field)] = values_field.as_slice() else {
        return Ok(None);
    };
    if values_field.descriptor.raw().0 != array_descriptor(enum_owner)
        || values_field.access_flags != (ACC_PRIVATE | ACC_STATIC | ACC_FINAL | ACC_SYNTHETIC)
        || !values_field.attributes.is_empty()
    {
        return Ok(None);
    }
    let mut implicit_members = vec![PendingEnumConstantBodyMember {
        table_index: u64::try_from(*values_field_index).unwrap_or(u64::MAX),
        name: values_field.name.raw().0.clone(),
        descriptor: values_field.descriptor.raw().0.clone(),
        access_flags: values_field.access_flags,
        has_code: false,
        access_marker_owner: None,
    }];
    // Every constant of one enum invokes the same source constructor, so the shared descriptor
    // is the group's source constructor: the fixed zero-argument shape or one arbitrary tail.
    let source_ctor: Vec<u8> = constant_shape[0].constructor_descriptor.clone();
    if constant_shape
        .iter()
        .any(|constant| constant.constructor_descriptor != source_ctor)
    {
        return Ok(None);
    }
    let shared_source_parameters = constant_shape[0].source_parameters.clone();
    for (name, descriptor, flags) in [
        (b"<clinit>".as_slice(), b"()V".as_slice(), ACC_STATIC),
        (
            b"values".as_slice(),
            values_method_descriptor.as_slice(),
            0x0001 | ACC_STATIC,
        ),
        (
            b"valueOf".as_slice(),
            value_of_method_descriptor.as_slice(),
            0x0001 | ACC_STATIC,
        ),
        (
            b"$values".as_slice(),
            values_method_descriptor.as_slice(),
            ACC_PRIVATE | ACC_STATIC | ACC_SYNTHETIC,
        ),
        (b"<init>".as_slice(), source_ctor.as_slice(), ACC_PRIVATE),
    ] {
        let Some(member) = unique_method_record(name, descriptor) else {
            return Ok(None);
        };
        if member.access_flags != flags || !member.has_code {
            return Ok(None);
        }
        implicit_members.push(member);
    }
    let mut constructors: Vec<_> = methods
        .iter()
        .enumerate()
        .filter(|(_, method)| method.name.raw().0 == b"<init>")
        .map(|(index, method)| enum_body_physical_member(index, method))
        .collect();
    let bridge_indexes: Vec<_> = constructors
        .iter()
        .filter_map(|method| {
            if method.access_flags == ACC_SYNTHETIC {
                enum_access_constructor_marker_owner(&method.descriptor)
                    .map(|owner| (method.table_index, owner))
            } else {
                None
            }
        })
        .collect();
    let needs_bridge = constant_shape
        .iter()
        .any(|constant| constant.allocation_owner != enum_owner);
    if constructors
        .iter()
        .filter(|method| method.descriptor == source_ctor)
        .count()
        != 1
        || bridge_indexes.len() != usize::from(needs_bridge)
        || bridge_indexes.iter().any(|(table_index, owner)| {
            !constructors
                .iter()
                .any(|constructor| constructor.table_index == *table_index && constructor.has_code)
                // The marker parameter type is javac's overload-disambiguation artifact, not a
                // semantic input: a top-level enum reuses its first constant's anonymous
                // subclass (`Op$1`), while a nested enum takes the enclosing scope's synthetic
                // anonymous class (`Holder2$1`). Binding the bridge to a constant by that name
                // would break the nested shape, so the bridge is bound structurally by its
                // proved delegation edges instead: the constructor chain proves its body
                // forwards unchanged name/ordinal to the private constructor, and every body
                // child's constructor must invoke exactly this descriptor with a null marker
                // (the per-relation bridge proof below). The one name rule that remains is
                // that the marker is not the enum owner itself.
                || owner.as_slice() == enum_owner
        })
        || constructors.iter().any(|constructor| {
            constructor.descriptor != source_ctor
                && !bridge_indexes
                    .iter()
                    .any(|(table_index, _)| *table_index == constructor.table_index)
        })
    {
        return Ok(None);
    }
    for (table_index, owner) in bridge_indexes {
        if let Some(constructor) = constructors
            .iter_mut()
            .find(|constructor| constructor.table_index == table_index)
        {
            constructor.access_marker_owner = Some(owner);
        }
    }
    let abstract_methods: Vec<_> = methods
        .iter()
        .enumerate()
        .filter(|(_, method)| method.access_flags & ACC_ABSTRACT != 0)
        .map(|(index, method)| enum_body_physical_member(index, method))
        .collect();
    let interface_owner = if enum_access_flags & ACC_ABSTRACT != 0 && abstract_methods.is_empty() {
        if enum_interfaces.len() != 1 {
            return Ok(None);
        }
        Some(enum_interfaces[0].as_slice())
    } else {
        None
    };
    Ok(Some((
        clinit_identity,
        *code,
        constructions,
        constant_shape,
        implicit_members,
        source_ctor,
        shared_source_parameters,
        constructors,
        needs_bridge,
        abstract_methods,
        interface_owner,
    )))
}

/// Check only the already-read direct interface facts. The facade performs the selected read.
pub fn enum_body_interface_member(
    access_flags: u16,
    methods: &[MemberHeader],
) -> Option<PendingEnumConstantBodyMember> {
    if access_flags & 0x0200 == 0 || methods.len() != 1 {
        return None;
    }
    let method = &methods[0];
    if method.access_flags != (0x0001 | ACC_ABSTRACT)
        || method.name.raw().0.is_empty()
        || method.descriptor.raw().0.is_empty()
        || method
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code")
    {
        return None;
    }
    Some(PendingEnumConstantBodyMember {
        table_index: 0,
        name: method.name.raw().0.clone(),
        descriptor: method.descriptor.raw().0.clone(),
        access_flags: method.access_flags,
        has_code: false,
        access_marker_owner: None,
    })
}

/// Complete the original Code/constructor/prefix shape after the optional interface read.
/// The vectors are the unchanged staged facts returned by `prepare_enum_body_candidate`.
/// Source refusals remain inside the shape; budget and cancellation errors leave the proof.
pub fn prove_enum_body_candidate_shape(
    enum_owner: &[u8],
    enum_access_flags: u16,
    fields: &[MemberHeader],
    methods: &[MemberHeader],
    code_candidates: &[EnumMethodCodeCandidate],
    clinit_identity: &PhysicalMethodId,
    code: &EnumMethodCodeCandidate,
    constant_shape: Vec<PendingEnumConstantBodyConstant>,
    implicit_members: Vec<PendingEnumConstantBodyMember>,
    source_ctor: &[u8],
    shared_source_parameters: &[EnumUserParameter],
    constructors: Vec<PendingEnumConstantBodyMember>,
    needs_bridge: bool,
    abstract_methods: Vec<PendingEnumConstantBodyMember>,
    interface_methods: Vec<PendingEnumConstantBodyMember>,
    source_field_declared: impl Fn(usize) -> bool,
    budget: &mut Budget,
) -> Result<Option<PendingEnumConstantBodyGroupShape>> {
    const STRING_CTOR: &[u8] = b"(Ljava/lang/String;ILjava/lang/String;)V";
    if (!abstract_methods.is_empty() && enum_access_flags & ACC_ABSTRACT == 0)
        || abstract_methods.iter().any(|method| method.has_code)
        || methods.iter().enumerate().any(|(index, method)| {
            !enum_body_physical_member(index, method).has_code
                && method.access_flags & ACC_ABSTRACT == 0
        })
    {
        return Ok(None);
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(code_candidates.len()).unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    let factory_codes: Vec<_> = code_candidates
        .iter()
        .filter(|candidate| {
            candidate.table_index == implicit_members[4].table_index
                && candidate.member.as_ref().is_some_and(|member| {
                    member.owner == clinit_identity.owner
                        && member.name.0 == b"$values"
                        && member.descriptor.0 == implicit_members[4].descriptor
                })
        })
        .collect();
    let factory_code = match factory_codes.as_slice() {
        [factory] => Some(*factory),
        _ => None,
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            code.instructions.len() + factory_code.map_or(0, |factory| factory.instructions.len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    let initializer_prefix = prove_enum_body_initializer_prefix(
        code,
        factory_code,
        enum_owner,
        &constant_shape,
        &implicit_members[0],
    );
    let constructor_proof_work = code_candidates
        .len()
        .saturating_mul(constructors.len())
        .saturating_add(
            code_candidates
                .iter()
                .filter(|candidate| {
                    constructors
                        .iter()
                        .any(|constructor| constructor.table_index == candidate.table_index)
                })
                .map(|candidate| candidate.instructions.len())
                .sum::<usize>(),
        )
        .saturating_add(constant_shape.len());
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(constructor_proof_work).unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    let constructor_identity = |member: &PendingEnumConstantBodyMember| PhysicalMethodId {
        owner: clinit_identity.owner.clone(),
        name: JvmBytes(member.name.clone()),
        descriptor: JvmBytes(member.descriptor.clone()),
    };
    let (constructor_chain, constructor_user_tail) = (|| -> Result<_> {
        let Some(private) = constructors
            .iter()
            .find(|member| member.descriptor == source_ctor)
        else {
            return Ok((
                Err("the unique private enum constructor is absent".to_owned()),
                None,
            ));
        };
        let (private_edge, user_tail) = if shared_source_parameters.is_empty() {
            (
                match prove_enum_constructor_candidate(
                    code_candidates,
                    private,
                    &constructor_identity(private),
                    b"java/lang/Enum",
                    ENUM_CTOR_DESCRIPTOR,
                    &[],
                ) {
                    Ok(edge) => edge,
                    Err(reason) => return Ok((Err(reason), None)),
                },
                None,
            )
        } else if source_ctor == STRING_CTOR {
            (
                match prove_enum_string_constructor_candidate(
                    code_candidates,
                    private,
                    &constructor_identity(private),
                    enum_owner,
                    fields,
                ) {
                    Ok(edge) => edge,
                    Err(reason) => return Ok((Err(reason), None)),
                },
                None,
            )
        } else {
            // The shared arbitrary tail: the private constructor keeps the unchanged Enum super
            // call and stores each user parameter into its own instance field.
            let matching: Vec<_> = code_candidates
                .iter()
                .filter(|candidate| candidate.table_index == private.table_index)
                .collect();
            let [code] = matching.as_slice() else {
                return Ok((
                    Err("the selected private constructor has no unique same-run Code".to_owned()),
                    None,
                ));
            };
            let (tail, super_call_bci) = match prove_arbitrary_user_constructor(
                code,
                enum_owner,
                &shared_source_parameters,
                fields,
                |index| source_field_declared(index),
                budget,
            )? {
                Ok(proved) => proved,
                Err(reason) => {
                    return Ok((
                        Err(format!(
                            "the arbitrary enum constructor body refused: {reason}"
                        )),
                        None,
                    ));
                }
            };
            (
                PendingEnumConstructorEdge {
                    caller: constructor_identity(private),
                    call_bci: super_call_bci,
                    target_owner: b"java/lang/Enum".to_vec(),
                    target_descriptor: ENUM_CTOR_DESCRIPTOR.to_vec(),
                    assigned_field: None,
                },
                Some(tail),
            )
        };
        let mut edges = vec![private_edge];
        if needs_bridge {
            let Some(access) = constructors
                .iter()
                .find(|member| member.access_marker_owner.is_some())
            else {
                return Ok((
                    Err("the unique synthetic access constructor is absent".to_owned()),
                    None,
                ));
            };
            edges.push(
                match prove_enum_constructor_candidate(
                    code_candidates,
                    access,
                    &constructor_identity(access),
                    enum_owner,
                    &source_ctor,
                    &shared_source_parameters,
                ) {
                    Ok(edge) => edge,
                    Err(reason) => return Ok((Err(reason), None)),
                },
            );
        }
        Ok((Ok(edges), user_tail))
    })()?;
    let direct_constant_bcis = (|| {
        initializer_prefix.as_ref().map_err(Clone::clone)?;
        constructor_chain.as_ref().map_err(Clone::clone)?;
        Ok(constant_shape
            .iter()
            .filter(|constant| constant.constructor_owner == enum_owner)
            .map(|constant| constant.constructor_bci)
            .collect())
    })();
    let group_shape = PendingEnumConstantBodyGroupShape {
        enum_access_flags,
        constants: constant_shape,
        implicit_members,
        abstract_methods,
        interface_methods,
        constructors,
        constructor_chain,
        constructor_user_tail,
        direct_constant_bcis,
        initializer_prefix,
    };

    Ok(Some(group_shape))
}

/// One block of text, every non-blank line prefixed with `depth` levels of indentation.
pub fn indent(text: &str, depth: usize) -> String {
    let pad = "    ".repeat(depth);
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        if line.trim_end_matches('\n').is_empty() {
            out.push_str(line);
            continue;
        }
        out.push_str(&pad);
        out.push_str(line);
    }
    out
}

/// The source text of one arbitrary user-tail enum constructor: the folded declaration spells
/// the user parameters only, in argument order, each assigned to its own stored field. One
/// spelling shared by the ordinary constant list and the constant-body projection.
pub fn spell_user_tail_constructor_text(
    declaration_name: &str,
    access_flags: u16,
    tail: &ProvedEnumUserConstructorTail,
) -> String {
    let mut constructor_declaration = String::new();
    if let Some(modifier) = crate::declaration::visibility(access_flags) {
        constructor_declaration.push_str(modifier);
        constructor_declaration.push(' ');
    }
    constructor_declaration.push_str(declaration_name);
    constructor_declaration.push('(');
    for (index, parameter) in tail.parameters.iter().enumerate() {
        if index > 0 {
            constructor_declaration.push_str(", ");
        }
        constructor_declaration.push_str(&parameter.type_text);
        constructor_declaration.push_str(&format!(" arg{index}"));
    }
    constructor_declaration.push(')');
    let mut assignments = String::new();
    for (index, parameter) in tail.parameters.iter().enumerate() {
        assignments.push_str(&format!(
            "        this.{} = arg{index};\n",
            parameter.field_name
        ));
    }
    format!("    {constructor_declaration} {{\n{assignments}    }}\n")
}

/// Qualify and spell the selected enum body's same-run member texts. Accessors borrow existing
/// physical and source facts; this stage neither reads definitions nor retains member reports.
pub fn prepare_enum_constant_body_source_projection<F, M, C>(
    declaration_definition: &PhysicalDefinitionId,
    declaration_name: &str,
    fields: &[F],
    methods: &[M],
    constants: &[C],
    constructor_text: Option<&String>,
    constructor_user_tail: Option<&ProvedEnumUserConstructorTail>,
    shape: &PendingEnumConstantBodyGroupShape,
    field_facts: impl Fn(&F) -> (u64, &PhysicalDefinitionId, &[u8], bool, bool),
    method_facts: impl Fn(&M) -> (u64, &PhysicalMethodId, u16, &[u8], &[u8]),
    method_source: impl Fn(&M) -> (bool, bool, bool, &str, EnumConstructorSourceTail),
    constant_facts: impl Fn(
        &C,
    ) -> (
        u64,
        u32,
        &[ProvedEnumUserArgument],
        Option<&PhysicalDefinitionId>,
        Option<&[M]>,
    ),
    budget: &mut Budget,
) -> Result<Option<(Vec<u64>, u64, Vec<u64>, String, Vec<(u64, String)>)>> {
    if constants.is_empty()
        || shape.constants.len() != constants.len()
        || shape.implicit_members.len() != 6
    {
        return Ok(None);
    }
    let mut constants_text = String::new();
    let mut constant_field_indices = Vec::with_capacity(constants.len());
    for (position, constant) in constants.iter().enumerate() {
        budget.poll()?;
        let (field_index, constructor_bci, source_arguments, subclass, body_methods) =
            constant_facts(constant);
        let Some(field) = usize::try_from(field_index)
            .ok()
            .and_then(|index| fields.get(index))
        else {
            return Ok(None);
        };
        let (physical_index, owner, raw_name, declared, markers_empty) = field_facts(field);
        if shape.constants[position].field_index != field_index
            || shape.constants[position].constructor_bci != constructor_bci
            || physical_index != field_index
            || owner != declaration_definition
            || !declared
            || !markers_empty
        {
            return Ok(None);
        }
        let (name, aliased) = crate::names::written_name(raw_name);
        if aliased {
            return Ok(None);
        }
        constants_text.push_str("    ");
        constants_text.push_str(&name);
        if !source_arguments.is_empty() {
            // The argument spelling is the one arbitrary-argument channel's own joined text.
            let arguments = user_arguments_source_text(source_arguments);
            budget.charge(
                CountedBudgetDimension::OutputBytes,
                u64::try_from(arguments.len().saturating_mul(6).saturating_add(4))
                    .unwrap_or(u64::MAX),
            )?;
            constants_text.push('(');
            constants_text.push_str(&arguments);
            constants_text.push(')');
        }
        match (subclass, body_methods) {
            (None, None) => {}
            (Some(subclass), Some(body_methods)) if !body_methods.is_empty() => {
                constants_text.push_str(" {\n");
                constants_text.push_str("        // jarde: selected enum child definition: ");
                constants_text.push_str(&format!("{subclass:?}"));
                constants_text.push('\n');
                for method in body_methods.iter() {
                    budget.poll()?;
                    let (_, identity, _, _, _) = method_facts(method);
                    let (declared, markers_empty, recovered, text, _) = method_source(method);
                    if identity.owner != *subclass
                        || !declared
                        || !markers_empty
                        || !recovered
                        || !text.ends_with("}\n")
                    {
                        return Ok(None);
                    }
                    constants_text.push_str(&indent(text, 1));
                }
                constants_text.push_str("    }");
            }
            _ => return Ok(None),
        }
        constants_text.push_str(if position + 1 == constants.len() {
            ";\n"
        } else {
            ",\n"
        });
        constant_field_indices.push(field_index);
    }
    let backing_field_index = shape.implicit_members[0].table_index;
    let backing_field = usize::try_from(backing_field_index)
        .ok()
        .and_then(|index| fields.get(index));
    if shape.implicit_members[0].name != b"$VALUES"
        || backing_field.is_none_or(|field| {
            let (index, owner, _, _, _) = field_facts(field);
            index != backing_field_index || owner != declaration_definition
        })
    {
        return Ok(None);
    }
    let mut implicit_method_indices = Vec::new();
    for member in shape
        .implicit_members
        .iter()
        .skip(1)
        .chain(shape.constructors.iter())
    {
        let Some(method) = usize::try_from(member.table_index)
            .ok()
            .and_then(|index| methods.get(index))
        else {
            return Ok(None);
        };
        let (index, identity, _, raw_name, raw_descriptor) = method_facts(method);
        if index != member.table_index
            || identity.owner != *declaration_definition
            || raw_name != member.name
            || raw_descriptor != member.descriptor
        {
            return Ok(None);
        }
        if !implicit_method_indices.contains(&member.table_index) {
            implicit_method_indices.push(member.table_index);
        }
    }
    let mut constructor_texts = Vec::new();
    if let Some(text) = constructor_text {
        let constructors: Vec<_> = shape
            .constructors
            .iter()
            .filter(|member| member.descriptor == b"(Ljava/lang/String;ILjava/lang/String;)V")
            .collect();
        let [constructor] = constructors.as_slice() else {
            return Ok(None);
        };
        let Some(method) = usize::try_from(constructor.table_index)
            .ok()
            .and_then(|index| methods.get(index))
        else {
            return Ok(None);
        };
        let (index, _, _, _, _) = method_facts(method);
        let (_, _, _, _, source_tail) = method_source(method);
        if index != constructor.table_index
            || source_tail != EnumConstructorSourceTail::SingleString
            || !text.ends_with("}\n")
        {
            return Ok(None);
        }
        budget.charge(
            CountedBudgetDimension::OutputBytes,
            u64::try_from(text.len()).unwrap_or(u64::MAX),
        )?;
        constructor_texts.push((constructor.table_index, text.clone()));
    } else if let Some(tail) = constructor_user_tail {
        // The arbitrary tail spells through the one shared user-tail constructor text.
        let constructors: Vec<_> = shape
            .constructors
            .iter()
            .filter(|member| member.descriptor == shape.constants[0].constructor_descriptor)
            .collect();
        let [constructor] = constructors.as_slice() else {
            return Ok(None);
        };
        let Some(method) = usize::try_from(constructor.table_index)
            .ok()
            .and_then(|index| methods.get(index))
        else {
            return Ok(None);
        };
        let (index, _, access_flags, raw_name, _) = method_facts(method);
        if index != constructor.table_index || raw_name != b"<init>" {
            return Ok(None);
        }
        let text = spell_user_tail_constructor_text(declaration_name, access_flags, tail);
        budget.charge(
            CountedBudgetDimension::OutputBytes,
            u64::try_from(text.len()).unwrap_or(u64::MAX),
        )?;
        constructor_texts.push((constructor.table_index, text));
    } else if constants
        .iter()
        .any(|constant| !constant_facts(constant).2.is_empty())
    {
        return Ok(None);
    }
    budget.charge(
        CountedBudgetDimension::OutputBytes,
        u64::try_from(constants_text.len()).unwrap_or(u64::MAX),
    )?;
    Ok(Some((
        constant_field_indices,
        backing_field_index,
        implicit_method_indices,
        constants_text,
        constructor_texts,
    )))
}

pub fn prefix_method_annotations(
    text: String,
    annotations: &crate::report::MemberAnnotationUses,
) -> String {
    if annotations.uses.is_empty() {
        return text;
    }
    let mut prefixed = String::new();
    for annotation in &annotations.uses {
        prefixed.push_str("    ");
        prefixed.push_str(annotation);
        prefixed.push('\n');
    }
    prefixed.push_str(&text);
    prefixed
}

/// Qualify and spell one already-proved ordinary enum over same-run borrowed member facts.
/// Physical reports and final class publication stay with the supplying caller.
pub fn prepare_enum_constant_source_projection<F, M>(
    declaration_name: &str,
    fields: &[F],
    methods: &[M],
    group: &ProvedOrdinaryEnumConstantGroup,
    terminal_constructor_body: Option<String>,
    initializer: Option<(u64, String)>,
    field_facts: impl Fn(
        &F,
    ) -> (
        u64,
        &PhysicalDefinitionId,
        &[u8],
        &[u8],
        u16,
        bool,
        &[String],
    ),
    method_facts: impl Fn(&M) -> (u64, &PhysicalDefinitionId, &[u8], &[u8], u16),
    method_source: impl Fn(
        &M,
    ) -> (
        bool,
        EnumConstructorSourceTail,
        bool,
        &[String],
        &crate::report::MemberAnnotationUses,
        &crate::report::ParameterAnnotationUses,
        &crate::report::TypeAnnotationUses,
    ),
    budget: &mut Budget,
) -> Result<
    Option<(
        Vec<u64>,
        u64,
        Vec<u64>,
        String,
        Vec<(u64, String)>,
        Option<String>,
    )>,
> {
    let Ok(constructor_index) = usize::try_from(group.constructor_method_index) else {
        return Ok(None);
    };
    let source_field = group
        .constructor_field_index
        .and_then(|index| usize::try_from(index).ok())
        .and_then(|index| fields.get(index));
    let Some(constructor) = methods.get(constructor_index) else {
        return Ok(None);
    };
    let (
        constructor_physical_index,
        constructor_owner,
        _,
        constructor_descriptor,
        constructor_access_flags,
    ) = method_facts(constructor);
    let (
        constructor_declared,
        constructor_source_tail,
        constructor_signature_refused,
        constructor_markers,
        constructor_annotations,
        constructor_parameter_annotations,
        constructor_type_annotations,
    ) = method_source(constructor);
    let user_tail = group.constructor_user_tail.as_ref();
    let no_source_constructor = group.constructor_field_index.is_none() && user_tail.is_none();
    let has_only_enum_signature_marker =
        constructor_signature_refused && constructor_markers.len() == 1;
    // The arbitrary tail's own proof owns the constructor shape; the one marker it can carry is
    // the same JVM-only name/ordinal Signature erasure refusal the recognized tails absorb.
    let has_arbitrary_signature_marker = user_tail.is_some()
        && constructor_markers.len() <= 1
        && constructor_markers
            .first()
            .is_none_or(|marker| marker.contains("jvm_signature_erasure_mismatch"));
    if constructor_physical_index != group.constructor_method_index
        || (!no_source_constructor && user_tail.is_none() && source_field.is_none())
        || source_field.is_some_and(|field| {
            let (index, _, _, _, _, declared, _) = field_facts(field);
            Some(index) != group.constructor_field_index || !declared
        })
        || (!no_source_constructor && !constructor_declared)
        || (!no_source_constructor
            && user_tail.is_none()
            && group.constructor_signature_present
            && !matches!(
                constructor_source_tail,
                EnumConstructorSourceTail::Int
                    | EnumConstructorSourceTail::SingleString
                    | EnumConstructorSourceTail::StringVarargs
            ))
        || (!constructor_markers.is_empty()
            && !has_only_enum_signature_marker
            && !has_arbitrary_signature_marker)
        || !constructor_annotations.refusals.is_empty()
        || !constructor_parameter_annotations.refusals.is_empty()
        || !constructor_type_annotations.refusals.is_empty()
        || (user_tail.is_some()
            && (constructor_parameter_annotations
                .uses_by_position
                .iter()
                .any(|uses| !uses.is_empty())
                || constructor_type_annotations
                    .parameter_uses
                    .iter()
                    .any(|uses| !uses.is_empty())))
        || constructor_parameter_annotations
            .uses_by_position
            .iter()
            .take(2)
            .any(|uses| !uses.is_empty())
        || constructor_type_annotations
            .parameter_uses
            .iter()
            .take(2)
            .any(|uses| !uses.is_empty())
    {
        return Ok(None);
    }

    let implicit_signature_shape = if group.constructor_signature_present {
        constructor_source_tail == EnumConstructorSourceTail::NoArg
            && constructor_signature_refused
            && constructor_markers.len() == 1
    } else {
        constructor_source_tail != EnumConstructorSourceTail::NoArg
            && !constructor_signature_refused
            && constructor_markers.is_empty()
    };
    if no_source_constructor
        && (group.delegating_constructor_method_index.is_some()
            || constructor_descriptor != b"(Ljava/lang/String;I)V"
            || constructor_source_tail == EnumConstructorSourceTail::Int
            || constructor_source_tail == EnumConstructorSourceTail::SingleString
            || constructor_source_tail == EnumConstructorSourceTail::StringVarargs
            || !implicit_signature_shape)
    {
        return Ok(None);
    }

    let field_name = if let Some(source_field) = source_field {
        let (name, aliased) = crate::names::written_name(field_facts(source_field).2);
        if aliased {
            return Ok(None);
        }
        Some(name)
    } else {
        None
    };
    let mut constructor_texts = Vec::new();
    if let Some(delegating_index) = group.delegating_constructor_method_index {
        if field_name.is_none() {
            return Ok(None);
        }
        let Ok(delegating_index_usize) = usize::try_from(delegating_index) else {
            return Ok(None);
        };
        let Some(delegating) = methods.get(delegating_index_usize) else {
            return Ok(None);
        };
        let (
            delegating_physical_index,
            _,
            delegating_name,
            delegating_descriptor,
            delegating_access_flags,
        ) = method_facts(delegating);
        let (_, _, _, _, delegating_annotations, _, _) = method_source(delegating);
        let valid_hidden_constructor = |method: &M| {
            let (
                declared,
                source_tail,
                signature_refused,
                markers,
                annotations,
                parameter_annotations,
                type_annotations,
            ) = method_source(method);
            declared
                && source_tail == EnumConstructorSourceTail::NoArg
                && signature_refused
                && markers.len() == 1
                && annotations.refusals.is_empty()
                && parameter_annotations.refusals.is_empty()
                && type_annotations.refusals.is_empty()
                && parameter_annotations
                    .uses_by_position
                    .iter()
                    .all(Vec::is_empty)
                && type_annotations.parameter_uses.iter().all(Vec::is_empty)
        };
        if delegating_physical_index != delegating_index
            || delegating_name != b"<init>"
            || delegating_descriptor != b"(Ljava/lang/String;I)V"
            || !valid_hidden_constructor(delegating)
            || group.constructor_body.is_none()
            || terminal_constructor_body.is_none()
            || group.constructor_signature_present
                != (constructor_source_tail == EnumConstructorSourceTail::Int)
        {
            return Ok(None);
        }

        let mut delegating_declaration = String::new();
        if let Some(modifier) = crate::declaration::visibility(delegating_access_flags) {
            delegating_declaration.push_str(modifier);
            delegating_declaration.push(' ');
        }
        delegating_declaration.push_str(declaration_name);
        delegating_declaration.push_str("()");
        let delegating_body =
            format!("    {delegating_declaration} {{\n        this(0);\n    }}\n");
        constructor_texts.push((
            delegating_index,
            prefix_method_annotations(delegating_body, &delegating_annotations),
        ));

        let source_parameter_type_uses = constructor_type_annotations
            .parameter_uses
            .get(2)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let Some(parameter_type) = (if source_parameter_type_uses.is_empty() {
            Some("int".to_owned())
        } else {
            crate::declaration::decorate_qualified_type("int", source_parameter_type_uses)
        }) else {
            return Ok(None);
        };
        let source_parameter_annotations = constructor_parameter_annotations
            .uses_by_position
            .get(2)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let parameter_annotations = source_parameter_annotations
            .iter()
            .map(|annotation| format!("{annotation} "))
            .collect::<String>();
        let mut terminal_declaration = String::new();
        if let Some(modifier) = crate::declaration::visibility(constructor_access_flags) {
            terminal_declaration.push_str(modifier);
            terminal_declaration.push(' ');
        }
        terminal_declaration.push_str(declaration_name);
        terminal_declaration.push('(');
        terminal_declaration.push_str(&parameter_annotations);
        terminal_declaration.push_str(&parameter_type);
        terminal_declaration.push_str(" arg0)");
        let Some(statements) = terminal_constructor_body else {
            return Ok(None);
        };
        let terminal_body = format!("    {terminal_declaration} {{\n{statements}    }}\n");
        constructor_texts.push((
            group.constructor_method_index,
            prefix_method_annotations(terminal_body, &constructor_annotations),
        ));
    } else {
        if terminal_constructor_body.is_some() {
            return Ok(None);
        }
        if let Some(tail) = user_tail {
            // The arbitrary tail's proof carries each parameter's type text and its own stored
            // field; the folded declaration spells the user parameters only, in argument order.
            constructor_texts.push((
                group.constructor_method_index,
                prefix_method_annotations(
                    spell_user_tail_constructor_text(
                        declaration_name,
                        constructor_access_flags,
                        tail,
                    ),
                    &constructor_annotations,
                ),
            ));
        } else if !no_source_constructor {
            let Some(field_name) = field_name.as_deref() else {
                return Ok(None);
            };
            let string_varargs =
                constructor_source_tail == EnumConstructorSourceTail::StringVarargs;
            let single_string = constructor_source_tail == EnumConstructorSourceTail::SingleString;
            if string_varargs
                && (constructor_descriptor != b"(Ljava/lang/String;I[Ljava/lang/String;)V"
                    || constructor_access_flags != 0x0082
                    || constructor_type_annotations
                        .parameter_uses
                        .get(2)
                        .is_some_and(|uses| !uses.is_empty()))
            {
                return Ok(None);
            }
            let source_parameter_type_uses = constructor_type_annotations
                .parameter_uses
                .get(2)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let Some(parameter_type) = (if string_varargs {
                Some("java.lang.String...".to_owned())
            } else if single_string && source_parameter_type_uses.is_empty() {
                Some("java.lang.String".to_owned())
            } else if source_parameter_type_uses.is_empty() {
                Some("int".to_owned())
            } else {
                crate::declaration::decorate_qualified_type(
                    if single_string {
                        "java.lang.String"
                    } else {
                        "int"
                    },
                    source_parameter_type_uses,
                )
            }) else {
                return Ok(None);
            };
            let source_parameter_annotations = constructor_parameter_annotations
                .uses_by_position
                .get(2)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let parameter_annotations = source_parameter_annotations
                .iter()
                .map(|annotation| format!("{annotation} "))
                .collect::<String>();
            let mut constructor_declaration = String::new();
            if let Some(modifier) = crate::declaration::visibility(constructor_access_flags) {
                constructor_declaration.push_str(modifier);
                constructor_declaration.push(' ');
            }
            constructor_declaration.push_str(declaration_name);
            constructor_declaration.push('(');
            constructor_declaration.push_str(&parameter_annotations);
            constructor_declaration.push_str(&parameter_type);
            constructor_declaration.push_str(" arg0)");
            let constructor_body = format!(
                "    {constructor_declaration} {{\n        this.{field_name} = arg0;\n    }}\n"
            );
            // The one known marker comes from the JVM-only name/ordinal parameters making the physical
            // descriptor longer than the already parsed source Signature. The enum proof checks that
            // source tail independently; no other constructor marker is absorbed by this projection.
            constructor_texts.push((
                group.constructor_method_index,
                prefix_method_annotations(constructor_body, &constructor_annotations),
            ));
        }
    }

    if initializer.is_some() {
        budget.charge(
            CountedBudgetDimension::IrItems,
            u64::try_from(fields.len()).unwrap_or(u64::MAX),
        )?;
    }
    let initializer_text = match initializer {
        Some((field_index, text)) => {
            let mut target = None;
            for field in fields {
                budget.poll()?;
                if field_facts(field).0 == field_index && target.replace(field).is_some() {
                    return Ok(None);
                }
            }
            let Some(target) = target else {
                return Ok(None);
            };
            let (
                _,
                target_owner,
                _,
                target_descriptor,
                target_access_flags,
                target_declared,
                target_markers,
            ) = field_facts(target);
            if !((target_descriptor == b"I" && target_access_flags == 0x0008)
                || (target_descriptor == b"Ljava/util/Map;" && target_access_flags == 0x0019))
                || target_owner != constructor_owner
                || !target_declared
                || (target_descriptor == b"I" && !target_markers.is_empty())
                || (target_descriptor == b"Ljava/util/Map;"
                    && target_markers
                        .iter()
                        .any(|marker| !marker.contains("field_generic_source_unproved")))
            {
                return Ok(None);
            }
            Some(text)
        }
        None => None,
    };

    let mut constants_text = String::new();
    for (index, constant) in group.constants.iter().enumerate() {
        constants_text.push_str("    ");
        constants_text.push_str(&constant.name);
        if let Some(argument) = &constant.source_argument {
            if let ProvedEnumSourceArgument::StringVarargs(values) = argument {
                let source_capacity = values.iter().fold(0_u64, |total, value| {
                    total
                        .saturating_add(
                            u64::try_from(value.value.len())
                                .unwrap_or(u64::MAX)
                                .saturating_mul(6),
                        )
                        .saturating_add(4)
                });
                budget.charge(CountedBudgetDimension::OutputBytes, source_capacity)?;
            }
            constants_text.push('(');
            constants_text.push_str(&argument.source_text());
            constants_text.push(')');
        }
        if index + 1 == group.constants.len() {
            constants_text.push_str(";\n");
        } else {
            constants_text.push_str(",\n");
        }
    }
    let mut implicit_method_indices: Vec<u64> = [
        group.initializer_method_index,
        group.values_method_index,
        group.value_of_method_index,
        group.values_factory_method_index,
    ]
    .into();
    if no_source_constructor {
        implicit_method_indices.push(group.constructor_method_index);
    }
    if group.constants.is_empty() {
        let hidden: std::collections::BTreeSet<u64> =
            implicit_method_indices.iter().copied().collect();
        let hidden_fields: std::collections::BTreeSet<u64> = group
            .constants
            .iter()
            .map(|constant| constant.field_index)
            .chain(std::iter::once(group.backing_field_index))
            .collect();
        let has_user_member = fields.iter().any(|field| {
            let (index, _, _, _, _, declared, _) = field_facts(field);
            !hidden_fields.contains(&index) && declared
        }) || methods.iter().any(|method| {
            let (index, _, name, _, _) = method_facts(method);
            !hidden.contains(&index)
                && !constructor_texts
                    .iter()
                    .any(|(constructor_index, _)| *constructor_index == index)
                && name != b"<clinit>"
        }) || initializer_text.is_some();
        if has_user_member {
            constants_text.push_str("    ;\n");
        }
    }
    let output_bytes = u64::try_from(constants_text.len())
        .unwrap_or(u64::MAX)
        .saturating_add(constructor_texts.iter().fold(0u64, |total, (_, text)| {
            total.saturating_add(u64::try_from(text.len()).unwrap_or(u64::MAX))
        }))
        .saturating_add(
            initializer_text
                .as_ref()
                .map_or(0, |text| u64::try_from(text.len()).unwrap_or(u64::MAX)),
        );
    budget.charge(CountedBudgetDimension::OutputBytes, output_bytes)?;

    Ok(Some((
        group
            .constants
            .iter()
            .map(|constant| constant.field_index)
            .collect(),
        group.backing_field_index,
        implicit_method_indices,
        constants_text,
        constructor_texts,
        initializer_text,
    )))
}

fn nested_enum_reference_spelling(own_prefix: &str, source_name: &str) -> String {
    source_name
        .strip_prefix(own_prefix)
        .unwrap_or(source_name)
        .to_owned()
}

/// Scan one recovered member and its declaration against already-selected nested enum targets.
/// Required projection origins are borrowed independently of optional serialized evidence.
pub fn prepare_nested_enum_reference_edits<'a, T>(
    own_prefix: &str,
    targets: &'a [T],
    target_names: impl Fn(&T) -> (&Vec<u8>, &String),
    method_identity: &PhysicalMethodId,
    method_index: u64,
    method_descriptor: &[u8],
    declaration: Option<&String>,
    recovery_text: &str,
    projection_origins: &[crate::source_map::Segment],
    budget: &mut Budget,
) -> Result<
    std::result::Result<
        Option<(
            Vec<(usize, usize, &'a T, std::collections::BTreeSet<u32>)>,
            Option<String>,
        )>,
        String,
    >,
> {
    let mut edits = Vec::new();
    for target in targets {
        // The lexical scan examines every byte even when it finds no candidate. Charge that
        // work before scanning so a large recovered method cannot bypass the request budget.
        budget.poll()?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            recovery_text.len() as u64,
        )?;
        let (binary_name, source_name) = target_names(target);
        let Some(pool_name) = jarde_java::names::source_class_name(&JvmBytes(binary_name.clone()))
        else {
            return Ok(Err("nested enum has no exact source class name".to_owned()));
        };
        let dotted_name = source_name;
        // The body states the class as the pool spells it or as an enum-switch plan already
        // dotted it; both spell the class this fold declares, and both become the fold's own
        // nested spelling below.
        let Some(dotted_spans) = java_code_name_spans(recovery_text, &dotted_name) else {
            return Ok(Err("nested enum source artifact is malformed".to_owned()));
        };
        let Some(pool_spans) = java_code_name_spans(recovery_text, &pool_name) else {
            return Ok(Err("nested enum source artifact is malformed".to_owned()));
        };
        let mut spans = dotted_spans;
        spans.extend(pool_spans);
        spans.sort();
        if spans.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Ok(Err(
                "nested enum source artifact states overlapping name tokens".to_owned(),
            ));
        }
        for (start, end) in spans {
            budget.poll()?;
            let mut smallest = usize::MAX;
            let mut source_bcis = std::collections::BTreeSet::new();
            for segment in projection_origins {
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                if segment.start() > start
                    || segment.end() < end
                    || segment.origin().primary().method() != Some(method_identity)
                {
                    continue;
                }
                if segment.len() < smallest {
                    smallest = segment.len();
                    source_bcis.clear();
                }
                if segment.len() == smallest {
                    source_bcis.extend(segment.origin().bcis());
                }
            }
            if source_bcis.is_empty() {
                return Ok(Err(format!(
                    "nested enum token in method {} has no exact source-map origin",
                    method_index
                )));
            }
            edits.push((start, end, target, source_bcis));
        }
    }
    // The declaration line names the folded class wherever the member's own descriptor does
    // (`enumSwitch(V1$Op arg0, …)`), and the fold renders that class as a nested declaration
    // of this text — so the declaration is re-spelled with the same fold spelling the body
    // tokens take. A member whose descriptor names no target keeps its declaration verbatim.
    let mut declaration_edits = Vec::new();
    if let Some(declaration) = declaration {
        for target in targets {
            budget.poll()?;
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                declaration.len() as u64,
            )?;
            let (binary_name, source_name) = target_names(target);
            let mut descriptor_entry = vec![b'L'];
            descriptor_entry.extend_from_slice(binary_name);
            descriptor_entry.push(b';');
            if !method_descriptor
                .windows(descriptor_entry.len())
                .any(|window| window == descriptor_entry)
            {
                continue;
            }
            let Some(pool_name) =
                jarde_java::names::source_class_name(&JvmBytes(binary_name.clone()))
            else {
                return Ok(Err("nested enum has no exact source class name".to_owned()));
            };
            let Some(spans) = java_code_name_spans(declaration, &pool_name) else {
                return Ok(Err("nested enum source declaration is malformed".to_owned()));
            };
            let spelling = nested_enum_reference_spelling(own_prefix, source_name);
            declaration_edits.extend(
                spans
                    .into_iter()
                    .map(|(start, end)| (start, end, spelling.clone())),
            );
        }
    }
    declaration_edits.sort_by_key(|(start, ..)| *start);
    if declaration_edits
        .windows(2)
        .any(|pair| pair[0].1 > pair[1].0)
    {
        return Ok(Err(
            "nested enum source declaration states overlapping name tokens".to_owned(),
        ));
    }
    let rewritten_declaration = (!declaration_edits.is_empty()).then(|| {
        let mut declaration = declaration.expect("checked above").clone();
        for (start, end, spelling) in declaration_edits.iter().rev() {
            declaration.replace_range(start..end, spelling);
        }
        declaration
    });
    if edits.is_empty() && declaration_edits.is_empty() {
        return Ok(Ok(None));
    }
    Ok(Ok(Some((edits, rewritten_declaration))))
}

/// Prove prepared reference edits against the caller's already-read IR and physical field keys.
pub fn prove_nested_enum_reference_replacements<T, F>(
    own_prefix: &str,
    method_index: u64,
    ir: &MethodIr,
    edits: Vec<(usize, usize, &T, std::collections::BTreeSet<u32>)>,
    target_facts: impl Fn(
        &T,
    ) -> (
        &Vec<u8>,
        &String,
        &[F],
        Option<&ProvedOrdinaryEnumConstantGroup>,
        &PhysicalDefinitionId,
        &PhysicalDefinitionId,
    ),
    field_facts: impl Fn(&F) -> (&jarde_reader::model::PhysicalMemberId, u64),
) -> std::result::Result<
    Vec<(
        usize,
        usize,
        String,
        u32,
        jarde_reader::model::PhysicalMemberId,
        u64,
        PhysicalDefinitionId,
        PhysicalDefinitionId,
    )>,
    String,
> {
    let mut replacements = Vec::new();
    let Some(code) = ir.code() else {
        return Err("nested enum source reference has no complete bytecode".to_owned());
    };
    for (start, end, target, source_bcis) in edits {
        let (child_internal, source_name, fields, group, owner_definition, child_definition) =
            target_facts(target);
        let Some(group) = group else {
            return Err("nested enum reference target has no ordinary enum proof".to_owned());
        };
        let mut matching = Vec::new();
        for bci in source_bcis {
            let Some(instruction) = code.instructions.iter().find(|insn| insn.bci == bci) else {
                continue;
            };
            if instruction.opcode != 0xb2 {
                continue;
            }
            let Some(index) = instruction.constant_pool_index else {
                continue;
            };
            let Ok(entry) = jarde_reader::classfile::cp_entry(ir.constant_pool(), index) else {
                continue;
            };
            let jarde_reader::classfile::CpEntryKind::FieldRef {
                owner: field_owner,
                name,
                descriptor,
                ..
            } = &entry.kind
            else {
                continue;
            };
            if field_owner.0 != *child_internal {
                continue;
            }
            let Some(field) = fields.iter().find(|field| {
                let jarde_reader::model::MemberKey::Field {
                    name: physical_name,
                    descriptor: physical_descriptor,
                } = &field_facts(field).0.member
                else {
                    return false;
                };
                physical_name.0 == name.0
                    && physical_descriptor.0 == descriptor.0
                    && group
                        .constants
                        .iter()
                        .any(|constant| constant.field_index == field_facts(field).1)
            }) else {
                continue;
            };
            matching.push((bci, field));
        }
        if matching.len() != 1 {
            return Err(format!(
                "nested enum token in method {} is not tied to one proved enum constant reference",
                method_index
            ));
        }
        let (bci, field) = matching[0];
        replacements.push((
            start,
            end,
            nested_enum_reference_spelling(own_prefix, source_name),
            bci,
            field_facts(field).0.clone(),
            field_facts(field).1,
            owner_definition.clone(),
            child_definition.clone(),
        ));
    }
    Ok(replacements)
}

/// Apply complete reference replacements and adjust their placed ranges before final publication.
/// The callbacks place existing method text and assemble the caller's original projection record.
pub fn render_nested_enum_reference_method<P>(
    recovery: &crate::report::RecoveryReport,
    mut replacements: Vec<(
        usize,
        usize,
        String,
        u32,
        jarde_reader::model::PhysicalMemberId,
        u64,
        PhysicalDefinitionId,
        PhysicalDefinitionId,
    )>,
    rewritten_declaration: Option<&str>,
    place_span: impl Fn(
        &crate::report::RecoveryReport,
        Option<&str>,
        usize,
        usize,
    ) -> Option<(usize, usize)>,
    assemble_projection: impl Fn(
        usize,
        usize,
        u32,
        &jarde_reader::model::PhysicalMemberId,
        u64,
        &PhysicalDefinitionId,
        &PhysicalDefinitionId,
    ) -> P,
    render_method: impl Fn(&crate::report::RecoveryReport, Option<&str>) -> Option<String>,
) -> Result<std::result::Result<(String, Vec<P>), String>> {
    replacements.sort_by_key(|replacement| replacement.0);
    if replacements.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Ok(Err("nested enum source references overlap".to_owned()));
    }
    let mut rewritten = recovery.clone();
    for (start, end, replacement, ..) in replacements.iter().rev() {
        rewritten.text.replace_range(start..end, replacement);
    }
    let mut adjusted = Vec::new();
    let mut prefix_delta = 0isize;
    for (start, end, replacement, bci, field, index, owner_definition, child_definition) in
        &replacements
    {
        let final_start = start.checked_add_signed(prefix_delta).ok_or_else(|| {
            Error::invalid_input("nested_enum_source_offset_invalid", "source span overflow")
        })?;
        let final_end = final_start + replacement.len();
        let Some((local_start, local_end)) =
            place_span(&rewritten, rewritten_declaration, final_start, final_end)
        else {
            return Ok(Err(
                "nested enum source-map span does not survive method placement".to_owned(),
            ));
        };
        adjusted.push(assemble_projection(
            local_start,
            local_end,
            *bci,
            field,
            *index,
            owner_definition,
            child_definition,
        ));
        prefix_delta += replacement.len() as isize - (end - start) as isize;
    }
    let Some(text) = render_method(&rewritten, rewritten_declaration) else {
        return Ok(Err(
            "nested enum source method cannot be reassembled".to_owned()
        ));
    };
    Ok(Ok((text, adjusted)))
}

/// Re-spell one physical field declaration without copying its class-source record.
pub fn project_static_fold_field<T, P>(
    declaration: Option<&String>,
    field_descriptor: &[u8],
    field_identity: &jarde_reader::model::PhysicalMemberId,
    field_index: u64,
    markers: &[String],
    targets: &[T],
    target_facts: impl Fn(
        &T,
    ) -> (
        &Vec<u8>,
        &String,
        &String,
        &PhysicalDefinitionId,
        &PhysicalDefinitionId,
    ),
    assemble_projection: impl Fn(
        usize,
        usize,
        &jarde_reader::model::PhysicalMemberId,
        u64,
        &PhysicalDefinitionId,
        &PhysicalDefinitionId,
    ) -> P,
    render_field: impl Fn(&str) -> String,
    budget: &mut Budget,
) -> Result<std::result::Result<Option<(String, Vec<P>)>, String>> {
    budget.poll()?;
    let Some(declaration) = declaration else {
        return Ok(Ok(None));
    };
    let mut edits = Vec::new();
    for target in targets {
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            declaration.len() as u64,
        )?;
        let (binary, pool_dotted, simple, root_definition, child_definition) = target_facts(target);
        let descriptor_entry = [b"L".as_slice(), binary, b";"].concat();
        if !field_descriptor
            .windows(descriptor_entry.len())
            .any(|window| window == descriptor_entry)
        {
            continue;
        }
        let Some(spans) = java_code_name_spans(declaration, pool_dotted) else {
            return Ok(Err("static fold field declaration is malformed".to_owned()));
        };
        for (start, end) in spans {
            budget.poll()?;
            edits.push((
                start,
                end,
                simple.clone(),
                root_definition.clone(),
                child_definition.clone(),
                field_identity.clone(),
                field_index,
            ));
        }
    }
    if edits.is_empty() {
        return Ok(Ok(None));
    }
    edits.sort_by_key(|(start, ..)| *start);
    if edits.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Ok(Err(
            "static fold field declaration states overlapping name tokens".to_owned(),
        ));
    }
    let mut declaration = declaration.clone();
    // The staged text is `declaration_member`'s own form — one indented marker line per
    // marker, then the four-space declaration — so every edit offset shifts by that prefix.
    let staged_prefix = markers
        .iter()
        .map(|marker| marker.len() + "    \n".len())
        .sum::<usize>()
        + "    ".len();
    let mut derived = Vec::new();
    let mut prefix_delta = 0isize;
    for (start, end, spelling, root_definition, child_definition, identity, index) in &edits {
        // The declaration is rewritten in place below, so this edit's own span is the one the
        // scan stated, moved by every earlier replacement's delta — the same move the derived
        // record states its position in the staged text with. Replacing at the raw offsets
        // would land `prefix_delta` bytes late from the second edit on and eat the text that
        // followed the name it meant to re-spell.
        let local_start = start
            .checked_add_signed(prefix_delta)
            .ok_or_else(|| Error::invalid_input("static_fold_offset", "span overflow"))?;
        let local_end = end
            .checked_add_signed(prefix_delta)
            .ok_or_else(|| Error::invalid_input("static_fold_offset", "span overflow"))?;
        let final_start = local_start
            .checked_add(staged_prefix)
            .ok_or_else(|| Error::invalid_input("static_fold_offset", "span overflow"))?;
        let final_end = final_start + spelling.len();
        declaration.replace_range(local_start..local_end, spelling);
        derived.push(assemble_projection(
            final_start,
            final_end,
            identity,
            *index,
            root_definition,
            child_definition,
        ));
        prefix_delta += spelling.len() as isize - (end - start) as isize;
    }
    Ok(Ok(Some((render_field(&declaration), derived))))
}

/// Select the text a static fold may re-spell; incompatible projected compositions refuse.
pub fn static_fold_body_source<'a>(
    method_identity: &PhysicalMethodId,
    method_index: u64,
    recovery: &'a crate::report::RecoveryReport,
    has_projected_text: bool,
    projected_body: Option<&'a String>,
    overrides: &[(PhysicalMethodId, crate::report::RecoveryReport)],
    bridge_calls: &[(PhysicalMethodId, u32, usize)],
    ctor_removals: &[(PhysicalMethodId, u32)],
) -> std::result::Result<&'a str, String> {
    if has_projected_text {
        if overrides.iter().any(|(id, _)| *id == *method_identity) {
            return Err(format!(
                "static fold cannot compose an instance re-run with the projected body of method {}",
                method_index
            ));
        }
        if bridge_calls
            .iter()
            .any(|(id, _, _)| *id == *method_identity)
            || ctor_removals.iter().any(|(id, _)| *id == *method_identity)
        {
            return Err(format!(
                "static fold cannot re-spell an access bridge or capture write inside the projected body of method {}",
                method_index
            ));
        }
    }
    let body_text = projected_body.map_or(recovery.text.as_str(), |body| body.as_str());
    Ok(body_text)
}

/// Scan all body reference tokens using the selected physical or retained projected origins.
pub fn scan_static_fold_body_references<'a, T, S>(
    targets: &'a [T],
    target_names: impl Fn(&T) -> (&String, &String),
    method_identity: &PhysicalMethodId,
    method_index: u64,
    recovery: &crate::report::RecoveryReport,
    body_text: &str,
    projected_segments: Option<&[S]>,
    segment_facts: impl Fn(&S) -> (u64, u64, &[u32]),
    budget: &mut Budget,
) -> Result<std::result::Result<Vec<(usize, usize, &'a T, std::collections::BTreeSet<u32>)>, String>>
{
    let mut edits = Vec::new();
    for target in targets {
        // The lexical scan examines every byte even when it finds no candidate. Charge that
        // work before scanning so a large recovered method cannot bypass the request budget.
        budget.poll()?;
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            body_text.len() as u64,
        )?;
        let (pool_name, dotted_name) = target_names(target);
        let mut spans: Vec<(usize, usize, bool)> =
            match java_code_name_spans(body_text, pool_name) {
                Some(spans) => spans
                    .into_iter()
                    .map(|(start, end)| (start, end, false))
                    .collect(),
                None => return Ok(Err("static fold source artifact is malformed".to_owned())),
            };
        if dotted_name != pool_name {
            let dotted_spans = match java_code_name_spans(body_text, dotted_name) {
                Some(spans) => spans,
                None => return Ok(Err("static fold source artifact is malformed".to_owned())),
            };
            spans.extend(
                dotted_spans
                    .into_iter()
                    .map(|(start, end)| (start, end, true)),
            );
            spans.sort();
        };
        if spans.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Ok(Err(
                "static fold source artifact states overlapping name tokens".to_owned(),
            ));
        }
        for (start, end, from_dotted) in spans {
            budget.poll()?;
            // A token the recovery spelled as the proved dotted **source path** — the member
            // feeds hand the run that spelling, and the family keeps it where the run states
            // a fresh local's declared type (the separated run writes the same qualified
            // line). The rewrite exists to replace spellings the artifact cannot keep (the
            // binary `$` name) with the simple name the fold's own nested declaration uses;
            // it re-spells *references*. A dotted-source token in a declaration position — a
            // statement-leading `Type name =` / `Type name;` / a for-each header — is a
            // declaration, not a reference, so the scan leaves it qualified.
            let line_start = body_text[..start].rfind('\n').map_or(0, |at| at + 1);
            let rest = body_text[end..].trim_start();
            let identifier_len = rest
                .find(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch == '$'))
                .unwrap_or(rest.len());
            let after_identifier = rest[identifier_len..].trim_start();
            if from_dotted
                && body_text[line_start..start].trim().is_empty()
                && identifier_len > 0
                && after_identifier.starts_with(['=', ';', ',', ':', ')'])
            {
                continue;
            }
            // Every segment of this method that covers the token contributes its bcis: the
            // token of a synthesized cast or receiver qualifier is covered by the enclosing
            // expression's segments, and the instruction that names the class — the cast's
            // own checkcast, the invoked member's owner — is among them. A projected body
            // reads the same coverage from its retained table, whose segments were already
            // filtered to this member when the first pass retained them.
            let mut source_bcis = std::collections::BTreeSet::new();
            match projected_segments {
                Some(segments) => {
                    for segment in segments {
                        let (segment_start, segment_end, segment_bcis) = segment_facts(segment);
                        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                        if usize::try_from(segment_start).unwrap_or(usize::MAX) > start
                            || usize::try_from(segment_end).unwrap_or(0) < end
                        {
                            continue;
                        }
                        source_bcis.extend(segment_bcis.iter().copied());
                    }
                }
                None => {
                    for segment in recovery.projection_origins() {
                        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                        if segment.start() > start
                            || segment.end() < end
                            || segment.origin().primary().method() != Some(&*method_identity)
                        {
                            continue;
                        }
                        source_bcis.extend(segment.origin().bcis());
                    }
                }
            }
            if source_bcis.is_empty() {
                return Ok(Err(format!(
                    "static fold token in method {} has no exact source-map origin",
                    method_index
                )));
            }
            edits.push((start, end, target, source_bcis));
        }
    }
    Ok(Ok(edits))
}

/// Decide whether this declaration needs the caller's original Exceptions read, after its charge.
pub fn static_fold_declaration_descriptor_names(
    declaration: &str,
    method_descriptor: &[u8],
    binary: &[u8],
    budget: &mut Budget,
) -> Result<bool> {
    budget.poll()?;
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        declaration.len() as u64,
    )?;
    let descriptor_entry = [b"L".as_slice(), binary, b";"].concat();
    Ok(method_descriptor
        .windows(descriptor_entry.len())
        .any(|window| window == descriptor_entry))
}

/// Collect the declaration edits only after its own descriptor or read Exceptions names the target.
pub fn append_static_fold_declaration_edits<R: Clone>(
    declaration: &str,
    pool_dotted: &str,
    simple: &String,
    relation: &R,
    names_target: bool,
    declaration_edits: &mut Vec<(usize, usize, String, R)>,
) -> std::result::Result<(), String> {
    if !names_target {
        return Ok(());
    }
    let Some(spans) = java_code_name_spans(declaration, pool_dotted) else {
        return Err("static fold source declaration is malformed".to_owned());
    };
    declaration_edits.extend(
        spans
            .into_iter()
            .map(|(start, end)| (start, end, simple.clone(), relation.clone())),
    );
    Ok(())
}

/// Finish declaration qualification and the complete no-op/projected-member decision before IR.
pub fn prepare_static_fold_method_edits<R>(
    declaration: Option<&String>,
    mut declaration_edits: Vec<(usize, usize, String, R)>,
    method_identity: &PhysicalMethodId,
    method_index: u64,
    body_edits_empty: bool,
    has_projected_text: bool,
    has_projected_emission: bool,
    overrides: &[(PhysicalMethodId, crate::report::RecoveryReport)],
    bridge_calls: &[(PhysicalMethodId, u32, usize)],
    ctor_removals: &[(PhysicalMethodId, u32)],
) -> std::result::Result<Option<(Option<String>, Vec<(u32, usize)>, Option<u32>)>, String> {
    declaration_edits.sort_by_key(|(start, ..)| *start);
    if declaration_edits
        .windows(2)
        .any(|pair| pair[0].1 > pair[1].0)
    {
        return Err("static fold source declaration states overlapping name tokens".to_owned());
    }
    let rewritten_declaration = (!declaration_edits.is_empty()).then(|| {
        let mut declaration = declaration.expect("checked above").clone();
        for (start, end, spelling, _) in declaration_edits.iter().rev() {
            declaration.replace_range(start..end, spelling);
        }
        declaration
    });
    let this_bridge_calls: Vec<(u32, usize)> = bridge_calls
        .iter()
        .filter(|(id, _, _)| *id == *method_identity)
        .map(|(_, bci, index)| (*bci, *index))
        .collect();
    let this_ctor_removal = ctor_removals
        .iter()
        .find(|(id, _)| *id == *method_identity)
        .map(|(_, bci)| *bci);
    let has_override = overrides.iter().any(|(id, _)| *id == *method_identity);
    // A projected member text without its re-spellable emission (a renamed companion, an
    // array-constructor rewrite) may keep its projection but not be re-spelled through the
    // physical body it replaced: staging the physical one would publish a body the first
    // pass already rewrote away. Such a member refuses the fold outright.
    if has_projected_text
        && !has_projected_emission
        && (!body_edits_empty || !declaration_edits.is_empty())
    {
        return Err(format!(
            "static fold cannot re-spell the projected member text of method {}",
            method_index
        ));
    }
    if !has_override
        && body_edits_empty
        && declaration_edits.is_empty()
        && this_bridge_calls.is_empty()
        && this_ctor_removal.is_none()
    {
        return Ok(None);
    }
    Ok(Some((
        rewritten_declaration,
        this_bridge_calls,
        this_ctor_removal,
    )))
}

/// Prove all reference tokens against already-read Code, handlers and SSA; no resolver is invoked.
pub fn prove_static_fold_body_references<T>(
    ir: &MethodIr,
    method_index: u64,
    edits: Vec<(usize, usize, &T, std::collections::BTreeSet<u32>)>,
    target_facts: impl Fn(
        &T,
    ) -> (
        &Vec<u8>,
        &String,
        &PhysicalDefinitionId,
        &PhysicalDefinitionId,
    ),
) -> std::result::Result<
    Vec<(
        usize,
        usize,
        String,
        u32,
        PhysicalDefinitionId,
        PhysicalDefinitionId,
    )>,
    String,
> {
    let mut replacements = Vec::new();
    let Some(code) = ir.code() else {
        return Err("static fold source reference has no complete bytecode".to_owned());
    };
    for (start, end, target, source_bcis) in edits {
        // One proved class reference: an instruction whose constant-pool entry names the
        // folded class in a position that types the token — the class itself, or the
        // owner of a field, method or interface method reference the token's own
        // expression reads — or the exception handler a catch clause's own type selected.
        // An interface call's owner is the same evidence a virtual call's owner is: the
        // invoked member belongs to that class, so its owner names it.
        let (binary, simple, root_definition, child_definition) = target_facts(target);
        let mut matching = Vec::new();
        for bci in source_bcis.iter().copied() {
            let Some(instruction) = code.instructions.iter().find(|insn| insn.bci == bci) else {
                continue;
            };
            let Some(index) = instruction.constant_pool_index else {
                continue;
            };
            let Ok(entry) = jarde_reader::classfile::cp_entry(ir.constant_pool(), index) else {
                continue;
            };
            let names_target = match &entry.kind {
                jarde_reader::classfile::CpEntryKind::Class { name, .. } => {
                    // The class's own name, or the array descriptor whose element is this
                    // class: the array class literal the text spells (`Nested[].class`)
                    // carries the element's name inside the descriptor, and the token it
                    // re-spells is that element's name.
                    name.0 == *binary || name.0 == [b"[L".as_slice(), binary, b";"].concat()
                }
                jarde_reader::classfile::CpEntryKind::FieldRef { owner, .. }
                | jarde_reader::classfile::CpEntryKind::MethodRef { owner, .. }
                | jarde_reader::classfile::CpEntryKind::InterfaceMethodRef { owner, .. } => {
                    owner.0 == *binary
                }
                _ => false,
            };
            if names_target {
                matching.push(bci);
            }
        }
        let anchor_bci = if let Some(bci) = matching.first().copied() {
            bci
        } else {
            let handler = code.exception_handlers.iter().find(|handler| {
                        source_bcis.contains(&handler.handler_bci)
                            && handler
                                .catch_type_index
                                .is_some_and(|index| {
                                    jarde_reader::classfile::cp_entry(
                                        ir.constant_pool(),
                                        index,
                                    )
                                    .is_ok_and(|entry| {
                                        matches!(&entry.kind, jarde_reader::classfile::CpEntryKind::Class { name, .. } if name.0 == *binary)
                                    })
                                })
                    });
            if let Some(handler) = handler {
                handler.handler_bci
            } else {
                // The declared type of a freshly introduced local: a covering store
                // whose stored value one instruction produced, where the producer's own
                // constant-pool entry names the type the declaration spells. The
                // store itself (`astore` carries no pool index) never names anything;
                // the value's single definition does — for a lambda site the
                // `invokedynamic` whose descriptor types the local, and the recovered
                // declaration spells exactly that value's type, so the producer's pool
                // entry is this token's own physical origin.
                let stored = static_fold_stored_type_anchor(
                    ir.ssa(),
                    code,
                    ir.constant_pool(),
                    &source_bcis,
                    binary,
                );
                let Some(anchor) = stored else {
                    return Err(format!(
                        "static fold token in method {} is not tied to one proved class reference",
                        method_index
                    ));
                };
                anchor
            }
        };
        replacements.push((
            start,
            end,
            simple.clone(),
            anchor_bci,
            root_definition.clone(),
            child_definition.clone(),
        ));
    }
    Ok(replacements)
}

/// Compose the complete token/bridge/capture edit set and adjust ranges before root publication.
pub fn render_static_fold_method<B, P>(
    method_identity: &PhysicalMethodId,
    method_index: u64,
    recovery: &crate::report::RecoveryReport,
    body_text: &str,
    has_projected_emission: bool,
    rewritten_declaration: Option<&str>,
    mut replacements: Vec<(
        usize,
        usize,
        String,
        u32,
        PhysicalDefinitionId,
        PhysicalDefinitionId,
    )>,
    this_bridge_calls: &[(u32, usize)],
    this_ctor_removal: Option<u32>,
    bridges: &[B],
    bridge_facts: impl Fn(&B) -> (&PhysicalMethodId, &str),
    bridge_owner_dotted: &str,
    place_span: impl Fn(
        &str,
        Option<&crate::report::RecoveryReport>,
        Option<&str>,
        usize,
        usize,
    ) -> Option<(usize, usize)>,
    token_projection: impl Fn(usize, usize, u32, &PhysicalDefinitionId, &PhysicalDefinitionId) -> P,
    bridge_projection: impl Fn(usize, usize, u32, &B) -> P,
    write_projection: impl Fn(usize, u32) -> P,
    render_method: impl Fn(&str, Option<&crate::report::RecoveryReport>, Option<&str>) -> Option<String>,
    budget: &mut Budget,
) -> Result<std::result::Result<(String, Vec<P>), String>> {
    replacements.sort_by_key(
        |replacement: &(
            usize,
            usize,
            String,
            u32,
            PhysicalDefinitionId,
            PhysicalDefinitionId,
        )| replacement.0,
    );
    if replacements.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Ok(Err("static fold source references overlap".to_owned()));
    }
    // The hidden access bridges' call sites: one proved `Owner.access$NNN(arg)` span whose
    // invokestatic bytecode anchors it, re-spelled to the qualified field read the bridge
    // forwarded.
    let mut bridge_edits = Vec::new();
    for (bci, index) in this_bridge_calls {
        let bridge = &bridges[*index];
        let Some((start, end, replacement)) = fold_bridge_call_rewrite_span(
            method_identity,
            recovery,
            *bci,
            bridge_owner_dotted,
            &bridge_facts(bridge).0.name.0,
            bridge_facts(bridge).1,
            budget,
        )?
        else {
            return Ok(Err(format!(
                "access bridge call in method {} has no exact source span",
                method_index
            )));
        };
        bridge_edits.push((start, end, replacement, *bci, *index));
    }
    bridge_edits.sort_by_key(|edit: &(usize, usize, String, u32, usize)| edit.0);
    if bridge_edits.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Ok(Err("access bridge call rewrites overlap".to_owned()));
    }
    // The capture write's own statement: exactly one line whose primary origin is the
    // putfield, removed from the body it never belonged to in source.
    let write_edit = match this_ctor_removal {
        Some(write_bci) => {
            match fold_capture_write_statement_span(method_identity, recovery, write_bci, budget)? {
                Some((start, end)) => Some((start, end, write_bci)),
                None => {
                    return Ok(Err(format!(
                        "capture write in constructor {} has no exact statement span",
                        method_index
                    )));
                }
            }
        }
        None => None,
    };
    // Every edit this fold states in one body: the spans were all computed on the same
    // recovery text, so one ordered walk carries every offset — applied innermost-last,
    // recorded with the deltas of every edit before it.
    enum PendingEdit {
        Token {
            start: usize,
            end: usize,
            bci: u32,
            root: PhysicalDefinitionId,
            child: PhysicalDefinitionId,
        },
        Bridge {
            start: usize,
            end: usize,
            bci: u32,
            index: usize,
        },
        Write {
            start: usize,
            end: usize,
        },
    }
    let mut pending: Vec<(usize, PendingEdit)> = replacements
        .iter()
        .map(|(start, end, _spelling, bci, root, child)| {
            (
                *start,
                PendingEdit::Token {
                    start: *start,
                    end: *end,
                    bci: *bci,
                    root: root.clone(),
                    child: child.clone(),
                },
            )
        })
        .chain(bridge_edits.iter().map(|(start, end, _, bci, index)| {
            (
                *start,
                PendingEdit::Bridge {
                    start: *start,
                    end: *end,
                    bci: *bci,
                    index: *index,
                },
            )
        }))
        .chain(write_edit.iter().map(|(start, end, _)| {
            (
                *start,
                PendingEdit::Write {
                    start: *start,
                    end: *end,
                },
            )
        }))
        .collect();
    pending.sort_by_key(|(start, _)| *start);
    let end_of = |edit: &PendingEdit| match edit {
        PendingEdit::Token { end, .. } | PendingEdit::Bridge { end, .. } => *end,
        PendingEdit::Write { end, .. } => *end,
    };
    let start_of = |edit: &PendingEdit| match edit {
        PendingEdit::Token { start, .. } | PendingEdit::Bridge { start, .. } => *start,
        PendingEdit::Write { start, .. } => *start,
    };
    if pending
        .windows(2)
        .any(|pair| end_of(&pair[0].1) > start_of(&pair[1].1))
    {
        return Ok(Err("member fold rewrites overlap".to_owned()));
    }
    let mut rewritten_text = body_text.to_owned();
    for (_, edit) in pending.iter().rev() {
        match edit {
            PendingEdit::Token { start, end, .. } => {
                let Some((_, _, spelling, ..)) = replacements.iter().find(|(at, ..)| at == start)
                else {
                    unreachable!("token edit carries its spelling")
                };
                rewritten_text.replace_range(start..end, spelling);
            }
            PendingEdit::Bridge { start, end, .. } => {
                let Some((_, _, replacement, ..)) =
                    bridge_edits.iter().find(|(at, ..)| at == start)
                else {
                    unreachable!("bridge edit carries its replacement")
                };
                rewritten_text.replace_range(start..end, replacement);
            }
            PendingEdit::Write { start, end } => {
                rewritten_text.replace_range(start..end, "");
            }
        }
    }
    // The capture constructor's proved shape is the capture write, the direct zero-argument
    // super call and the return. The write edit removed the write's own line; the family
    // presentation of this constructor is the bare template the capture certificate fabricates
    // for it — the super call and the return, with no recovered marker lines. What the removal
    // leaves behind is exactly those lines, so the constructor keeps the physical body's own
    // layout without them. (A body this proof shaped cannot carry token edits, so no span below
    // is computed against the stripped text.)
    let rewritten_text = if write_edit.is_some() {
        rewritten_text
            .split('\n')
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<&str>>()
            .join("\n")
    } else {
        rewritten_text
    };
    // The physical base's span mapping and placement read a whole recovery report; the
    // projected base works from the rewritten emission text and the projection's own extra
    // markers (bridge and capture-write edits were refused for it above, so its pending
    // edits are tokens alone).
    let rewritten_recovery = (!has_projected_emission).then(|| {
        let mut rewritten = recovery.clone();
        rewritten.text = rewritten_text.clone();
        rewritten
    });
    let mut adjusted = Vec::new();
    let mut prefix_delta = 0isize;
    for (at, edit) in &pending {
        let _ = at;
        match edit {
            PendingEdit::Token {
                start,
                end,
                bci,
                root,
                child,
            } => {
                let Some((_, _, spelling, ..)) = replacements
                    .iter()
                    .find(|(candidate, ..)| candidate == start)
                else {
                    unreachable!("token edit carries its spelling")
                };
                let final_start = start
                    .checked_add_signed(prefix_delta)
                    .ok_or_else(|| Error::invalid_input("static_fold_offset", "span overflow"))?;
                let final_end = final_start + spelling.len();
                let span = place_span(
                    &rewritten_text,
                    rewritten_recovery.as_ref(),
                    rewritten_declaration,
                    final_start,
                    final_end,
                );
                let Some((local_start, local_end)) = span else {
                    return Ok(Err(
                        "static fold source-map span does not survive method placement".to_owned(),
                    ));
                };
                adjusted.push(token_projection(local_start, local_end, *bci, root, child));
                prefix_delta += spelling.len() as isize - (end - start) as isize;
            }
            PendingEdit::Bridge {
                start,
                end,
                bci,
                index,
            } => {
                let Some((_, _, replacement, ..)) = bridge_edits
                    .iter()
                    .find(|(candidate, ..)| candidate == start)
                else {
                    unreachable!("bridge edit carries its replacement")
                };
                let final_start = start
                    .checked_add_signed(prefix_delta)
                    .ok_or_else(|| Error::invalid_input("static_fold_offset", "span overflow"))?;
                let final_end = final_start + replacement.len();
                // A bridge edit implies the physical base: a projected body with one was
                // refused above.
                let Some((local_start, local_end)) = place_span(
                    &rewritten_text,
                    rewritten_recovery.as_ref(),
                    rewritten_declaration,
                    final_start,
                    final_end,
                ) else {
                    return Ok(Err(
                        "access bridge span does not survive method placement".to_owned()
                    ));
                };
                let bridge = &bridges[*index];
                adjusted.push(bridge_projection(local_start, local_end, *bci, bridge));
                prefix_delta += replacement.len() as isize - (end - start) as isize;
            }
            PendingEdit::Write { start, end } => {
                prefix_delta -= (*end - *start) as isize;
            }
        }
    }
    // Declaration rewrites carry no derived record, exactly as the nested-enum projection:
    // the recovery map anchors body tokens only, and the declaration rewrite stays gated on
    // the member's own descriptor or `Exceptions` attribute.
    let composed_text = render_method(
        &rewritten_text,
        rewritten_recovery.as_ref(),
        rewritten_declaration,
    );
    let Some(text) = composed_text else {
        return Ok(Err(
            "static fold source method cannot be reassembled".to_owned()
        ));
    };
    if let Some((_, _, write_bci)) = write_edit {
        // The hidden capture write keeps its record on the constructor's declaration line —
        // the position its parameter's own elision is stated beside.
        let line_end = text.find('\n').unwrap_or(text.len());
        adjusted.push(write_projection(line_end, write_bci));
    }
    Ok(Ok((text, adjusted)))
}

/// Check the complete member-family preparation before any retained-text replay or re-recovery.
/// Accessors borrow the original reports; projection callbacks remain lazy at the original gates.
pub fn qualify_member_fold_sources<M, D, R>(
    root_execution: &ExecutionReport,
    members: &[M],
    child: impl Fn(&M) -> (&ExecutionReport, Option<&D>),
    signature: impl Fn(&D) -> (Option<&JvmBytes>, Option<&str>, Option<&str>),
    has_enum_projection: impl Fn(&M) -> bool,
    initializer_order: impl Fn(&M) -> Option<Vec<usize>>,
    methods: impl Fn(&M) -> &[R],
    method_identity: impl Fn(&R) -> &PhysicalMethodId,
) -> std::result::Result<(), String> {
    if !matches!(*root_execution, ExecutionReport::Complete { .. })
        || members.iter().any(|member| {
            !matches!(*child(member).0, ExecutionReport::Complete { .. })
                || child(member).1.is_none()
        })
    {
        return Err("member fold physical preparation is incomplete".to_owned());
    }
    // A member class whose `Signature` this fold does not carry stays outside it: the nested
    // header this fold writes is the row's own flags over the physical parents, and the one
    // Signature position it spells is a child run's proved type-parameter header. A child
    // whose header projection was refused — or produced a projection with no type parameters,
    // such as a parameterized parent — keeps the separated presentation the narrow channels
    // already state.
    if members.iter().any(|member| {
        child(member).1.is_some_and(|declaration| {
            signature(declaration).0.is_some()
                && !(signature(declaration).1.is_some() && signature(declaration).2.is_none())
        })
    }) {
        return Err(
            "member fold member has a class Signature projection this fold does not carry"
                .to_owned(),
        );
    }
    // The nested child block this fold writes carries the child's own member-text projection
    // channels (omitted members, markers, projected member texts) but not its enum-constant or
    // initializer projections: a child whose first pass staged either would come out of the
    // nested writer physically ordered and with its `<clinit>` back, and that is a degradation
    // this fold refuses rather than publishes. An initializer proof that staged the identity
    // order over a child with no `<clinit>` changed nothing and is not a projection here.
    if members.iter().any(|member| {
        has_enum_projection(member)
            || initializer_order(member).is_some_and(|order| {
                order
                    .iter()
                    .enumerate()
                    .any(|(position, field)| position != *field)
                    || methods(member).iter().any(|method| {
                        method_identity(method).name.0 == b"<clinit>"
                            && method_identity(method).descriptor.0 == b"()V"
                    })
            })
    }) {
        return Err(
            "member fold member has an enum or initializer projection this fold does not carry"
                .to_owned(),
        );
    }
    Ok(())
}

/// Prepare every source spelling and row/binary match in physical member order. The caller's
/// retained-text replay is invoked only after that child's exact class name was established.
pub fn prepare_member_fold_targets<'a, M, T>(
    root_binary: &JvmBytes,
    members: &'a [M],
    child_binary: impl Fn(&M) -> &JvmBytes,
    replay_child: impl Fn(&M) -> bool,
    simple_name: impl Fn(&M) -> &String,
    make_target: impl Fn(&'a M, Vec<u8>, String, String, String) -> T,
) -> std::result::Result<Vec<T>, String> {
    let Some(root_source_name) = source_class_name(&root_binary.clone()) else {
        return Err("member fold root has no exact source class name".to_owned());
    };
    let mut targets = Vec::with_capacity(members.len());
    for member in members {
        let binary = child_binary(member).0.clone();
        let Some(source) = source_class_name(&JvmBytes(binary.clone())) else {
            return Err("member fold member has no exact source class name".to_owned());
        };
        if !replay_child(member) {
            return Err("one member child has another physical source projection".to_owned());
        }
        let spelling = simple_name(member).clone();
        let mut expected_binary = root_binary.0.clone();
        expected_binary.push(b'$');
        expected_binary.extend_from_slice(&JvmBytes::from_unicode(&spelling).0);
        if binary != expected_binary {
            return Err(
                "member fold member row and binary name disagree on the source name".to_owned(),
            );
        }
        targets.push(make_target(
            member,
            binary,
            source,
            format!("{root_source_name}.{spelling}"),
            spelling,
        ));
    }
    Ok(targets)
}

/// A re-recovered physical method replaces its old text at the family projection's own
/// bar for a capture-involved member.
pub fn qualify_member_fold_recovery(
    recovery: &crate::RecoveryReport,
    method: &PhysicalMethodId,
) -> std::result::Result<(), String> {
    // The family projection's own member check (the facade loop that writes the folded
    // source) requires a complete, produced run from every member, and requires the
    // full body-quality planes — statements, structured regions, no fallbacks — only of
    // a member its capture proof does not involve. A member with accessor call sites,
    // capture reads or Outer.super sites keeps its own presentation: its text *is* the
    // capture shape's answer, quoted or not, exactly as the separated per-member run
    // wrote it. This fold gate replaces that member's physical text with the re-run's,
    // and every method the fold re-recovers comes from the capture plan's own claim set
    // (constructor writes, capture reads, bridge call sites) — involved by
    // construction. The mirror is therefore the family check's universal half: the
    // re-run must be complete and produced, and nothing more, or one claimed member
    // whose single-class re-run cannot prove an enclosing receiver (a body whose text
    // equals the baseline quote) would abandon the whole fold and demote the family to
    // the root's refusal — the one outcome the fold exists to prevent.
    if !matches!(recovery.execution, ExecutionReport::Complete { .. }) || !recovery.produced() {
        return Err(format!(
            "member fold re-recovery of {method:?} retains an incomplete re-run"
        ));
    }
    Ok(())
}

/// Spell the already selected root for the family's bridge rewrites after re-recovery completes.
pub fn member_fold_bridge_owner(root_binary: &JvmBytes) -> std::result::Result<String, String> {
    let Some(bridge_owner) = root_binary.clone().to_unicode() else {
        return Err("member fold root has no scalar MUTF-8 source name".to_owned());
    };
    Ok(bridge_owner.replace('/', "."))
}

/// Spell a nested member header from the caller's original cloned physical declaration facts.
fn nested_static_member_header(
    simple_name: &str,
    access_flags: u16,
    this_class: &jarde_reader::model::JvmString,
    physical_flags: u16,
    super_class: Option<&jarde_reader::model::JvmString>,
    interfaces: &[jarde_reader::model::JvmString],
    generic_type_parameters: Option<&str>,
    fold_targets: &[(&str, &str)],
) -> Option<String> {
    const ACC_PROTECTED: u16 = 0x0004;
    const ACC_STRICT: u16 = 0x0800;
    const SOURCE_CLASS_FLAGS: u16 = ACC_PUBLIC
        | ACC_PRIVATE
        | ACC_PROTECTED
        | ACC_ABSTRACT
        | ACC_FINAL
        | ACC_STRICT
        | ACC_STATIC;
    let rewrite = |name: &str| -> String {
        fold_targets
            .iter()
            .find_map(|(pool, spelling)| (name == *pool).then(|| (*spelling).to_owned()))
            .unwrap_or_else(|| name.to_owned())
    };
    let flags = (physical_flags & !SOURCE_CLASS_FLAGS) | (access_flags & SOURCE_CLASS_FLAGS);
    let superclass = match super_class {
        Some(super_class) => Some(rewrite(&source_class_name(&JvmBytes(
            super_class.raw().0.to_vec(),
        ))?)),
        None => None,
    };
    let rewritten_interfaces = interfaces
        .iter()
        .map(|interface| {
            source_class_name(&JvmBytes(interface.raw().0.to_vec())).map(|name| rewrite(&name))
        })
        .collect::<Option<Vec<_>>>()?;
    // A child whose own run projected its type-parameter header carries it verbatim: that
    // projection is the one Signature position this nested rebuild already spells every other
    // part of (row flags, physical parents), so the two spellings cannot diverge.
    let mut class_header_declaration = crate::declaration::class_declaration_with_types(
        simple_name,
        this_class,
        flags,
        super_class,
        interfaces,
        generic_type_parameters,
        superclass.as_deref(),
        Some(&rewritten_interfaces),
    )?;
    if access_flags & ACC_STATIC != 0 {
        let visibility_len = ["public ", "protected ", "private "]
            .iter()
            .find(|prefix| class_header_declaration.starts_with(**prefix))
            .map_or(0, |prefix| prefix.len());
        class_header_declaration.insert_str(visibility_len, "static ");
    }
    Some(class_header_declaration)
}

/// Render the complete nested member unit using the existing staged objects and source spans.
/// Accessors must be pure borrowed-field projections; factories only construct the caller's
/// original report/anchor objects. No reader, resolver, recovery, or publication runs here.
pub fn nested_static_member_source_text<F, M, T: Clone, R, D: Clone>(
    simple_name: &str,
    access_flags: u16,
    header_facts: (
        &jarde_reader::model::JvmString,
        u16,
        Option<&jarde_reader::model::JvmString>,
        &[jarde_reader::model::JvmString],
    ),
    generic_type_parameters: Option<&str>,
    old_name: &str,
    annotation_uses: &[String],
    annotation_refusals: &[String],
    fields: &[F],
    field_texts: &[T],
    methods: &[M],
    method_texts: &[T],
    fold_targets: &[(&str, &str)],
    capture_field_index: Option<u64>,
    retained: (&[u64], &[String], &[R], &[u64]),
    field_facts: impl Fn(&F) -> (u64, &[String], Option<&String>, &[String]),
    method_facts: impl Fn(&M) -> (u64, &PhysicalMethodId, &String),
    staged_facts: impl Fn(&T) -> (u64, &String, &[D]),
    staged_mut: impl Fn(&mut T) -> (&mut String, &mut Vec<D>),
    retained_facts: impl Fn(&R) -> (u64, &String),
    make_staged: impl Fn(u64, String) -> T,
    range: impl Fn(&D) -> (usize, usize, bool),
    set_range: impl Fn(&mut D, usize, usize),
    class_projection: impl Fn(usize, usize) -> D,
    field_projection: impl Fn(usize, usize, &F, bool) -> D,
    constructor_projection: impl Fn(usize, usize, &PhysicalMethodId, bool) -> D,
) -> Option<(String, Vec<D>)> {
    let class_header_declaration = nested_static_member_header(
        simple_name,
        access_flags,
        header_facts.0,
        header_facts.1,
        header_facts.2,
        header_facts.3,
        generic_type_parameters,
        fold_targets,
    )?;
    let (omitted_methods, retained_markers, retained_member_texts, hidden_fields) = retained;
    let mut text = String::new();
    let mut derived = Vec::new();
    for annotation in annotation_uses {
        text.push_str(&indent(&format!("{annotation}\n"), 1));
    }
    for refusal in annotation_refusals {
        text.push_str(&indent(
            &format!("// jarde: class annotation refused: {refusal}\n"),
            1,
        ));
    }
    let header_line = indent(&format!("{class_header_declaration} {{\n"), 1);
    let own_start = text.len() + "    ".len();
    text.push_str(&header_line);
    derived.push(class_projection(own_start, text.len() - 1));
    let hidden_span = (own_start, text.len() - 1);
    // The first pass's own marker lines — omitted helper and projection notes — sit directly
    // under the header, before any member, exactly where the flat unit placed them.
    for marker in retained_markers {
        text.push_str(&indent(&format!("{marker}\n"), 2));
    }
    let mut first = true;
    for field in fields {
        let (field_index, annotations, field_declaration, field_markers) = field_facts(field);
        if capture_field_index == Some(field_index) {
            // The synthetic capture field's declaration is hidden with its constructor
            // parameter: the fold's own text never names it, and the derived record keeps
            // the physical field it removed, anchored on the declaration it hides beside.
            derived.push(field_projection(hidden_span.0, hidden_span.1, field, true));
            continue;
        }
        if hidden_fields.contains(&field_index) {
            // The child's own assert projection proved its switch field rewritten; the fold's
            // nested rendering keeps that hiding, anchored beside the header as its flat text
            // was.
            derived.push(field_projection(hidden_span.0, hidden_span.1, field, false));
            continue;
        }
        if !first {
            text.push('\n');
        }
        first = false;
        for annotation in annotations {
            text.push_str(&indent(&format!("{annotation}\n"), 2));
        }
        if let Some(projected) = field_texts
            .iter()
            .find(|projected| staged_facts(projected).0 == field_index)
        {
            let offset = text.len();
            text.push_str(&indent(staged_facts(projected).1, 1));
            for entry in staged_facts(projected).2 {
                let source_span = staged_facts(projected)
                    .1
                    .get(range(entry).0..range(entry).1)?;
                let start = indented_offset(staged_facts(projected).1, range(entry).0, 1, false)?;
                let end = indented_offset(staged_facts(projected).1, range(entry).1, 1, true)?;
                let mut translated = entry.clone();
                set_range(&mut translated, offset + start, offset + end);
                if text.get(range(&translated).0..range(&translated).1) != Some(source_span) {
                    return None;
                }
                derived.push(translated);
            }
        } else {
            let member = match field_declaration {
                Some(declaration) => declaration_member(declaration, field_markers),
                None => comment_member(field_markers),
            };
            text.push_str(&indent(&member, 1));
        }
    }
    for method in methods {
        let (method_index, method_identity, physical_text) = method_facts(method);
        // A member the first pass's projection omitted carries no declaration here either: its
        // uses were rewritten in the retained member texts, and the physical record stays in the
        // child's own report.
        if omitted_methods.contains(&method_index) {
            continue;
        }
        if !first {
            text.push('\n');
        }
        first = false;
        // A member whose body the first pass projected keeps that projected text here — the
        // fold's own staged texts (method_texts) still win, because those are the ones carrying
        // the fold's re-spelled references.
        let mut staged = method_texts
            .iter()
            .find(|projected| staged_facts(projected).0 == method_index)
            .cloned()
            .or_else(|| {
                retained_member_texts
                    .iter()
                    .find(|projected| retained_facts(projected).0 == method_index)
                    .map(|projected| make_staged(method_index, retained_facts(projected).1.clone()))
            })
            .unwrap_or_else(|| make_staged(method_index, physical_text.clone()));
        let (staged_text, staged_derived) = staged_mut(&mut staged);
        if method_identity.name.0 == b"<init>" {
            let old_token = format!("{old_name}(");
            let Some(local_start) = staged_text.find(&old_token) else {
                return None;
            };
            let local_end = local_start + old_name.len();
            staged_text.replace_range(local_start..local_end, simple_name);
            staged_derived.push(constructor_projection(
                local_start,
                local_start + simple_name.len(),
                method_identity,
                false,
            ));
            if capture_field_index.is_some() {
                // The synthetic first parameter leaves the constructor's source list: the
                // descriptor's first parameter is the proved Outer capture, and the capture
                // write that consumed it is already gone from the staged body. Derived ranges
                // the facade staged after this list move by exactly the bytes dropped, plus
                // the rename's own delta.
                let list_start = local_start + simple_name.len() + "(".len();
                let Some(drop) = top_level_parameter_end(&staged_text, list_start) else {
                    return None;
                };
                let name_delta = simple_name.len() as isize - old_name.len() as isize;
                staged_text.replace_range(list_start..drop, "");
                shift_derived_ranges(
                    staged_derived,
                    local_start + old_name.len(),
                    name_delta - (drop - list_start) as isize,
                    &range,
                    &set_range,
                );
                // The hidden capture write's record keeps the declaration line as its span: the
                // line itself just changed length, so the range is re-clamped to it.
                let line_end = staged_text.find('\n').unwrap_or(staged_text.len());
                for entry in staged_derived.iter_mut() {
                    if range(entry).2 {
                        let start = range(entry).0.min(line_end);
                        set_range(entry, start, line_end);
                    }
                }
                staged_derived.push(constructor_projection(
                    local_start,
                    local_start + simple_name.len() + "()".len(),
                    method_identity,
                    true,
                ));
            }
        }
        let offset = text.len();
        text.push_str(&indent(&staged_text, 1));
        for entry in &*staged_derived {
            let Some(source_span) = staged_text.get(range(entry).0..range(entry).1) else {
                return None;
            };
            let Some(start) = indented_offset(&staged_text, range(entry).0, 1, false) else {
                return None;
            };
            let Some(end) = indented_offset(&staged_text, range(entry).1, 1, true) else {
                return None;
            };
            let mut translated = entry.clone();
            set_range(&mut translated, offset + start, offset + end);
            if text.get(range(&translated).0..range(&translated).1) != Some(source_span) {
                return None;
            }
            derived.push(translated);
        }
    }
    text.push_str("    }\n");
    Some((text, derived))
}

/// The end of a constructor's first source parameter: just past the first top-level separator,
/// or at the closing parenthesis when the list holds only that parameter. The list begins just
/// after its opening parenthesis; generic angle brackets, annotations, nested parentheses and
/// literals never separate parameters.
fn top_level_parameter_end(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 1isize;
    let mut angles = 0isize;
    let mut index = open;
    let mut literal: Option<u8> = None;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(quote) = literal {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote {
                literal = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            literal = Some(byte);
        } else {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                b'<' => angles += 1,
                b'>' => angles -= 1,
                b',' if depth == 1 && angles == 0 => {
                    let mut end = index + 1;
                    if bytes.get(end) == Some(&b' ') {
                        end += 1;
                    }
                    return Some(end);
                }
                _ => {}
            }
        }
        index += 1;
    }
    None
}

/// Move every derived range that begins at or after `from` by `delta` byte positions.
fn shift_derived_ranges<D>(
    derived: &mut [D],
    from: usize,
    delta: isize,
    range: &impl Fn(&D) -> (usize, usize, bool),
    set_range: &impl Fn(&mut D, usize, usize),
) {
    for entry in derived.iter_mut() {
        if range(entry).0 >= from {
            let (start, end, _) = range(entry);
            set_range(
                entry,
                (start as isize + delta).max(0) as usize,
                (end as isize + delta).max(0) as usize,
            );
        }
    }
}

pub fn declaration_member(declaration: &str, markers: &[String]) -> String {
    let mut out = String::new();
    for marker in markers {
        out.push_str(&indent(&format!("{marker}\n"), 1));
    }
    out.push_str(&format!("    {declaration};\n"));
    out
}

pub fn comment_member(markers: &[String]) -> String {
    let mut out = String::new();
    for marker in markers {
        out.push_str(&indent(&format!("{marker}\n"), 1));
    }
    out
}

pub fn indented_offset(
    text: &str,
    offset: usize,
    depth: usize,
    end_boundary: bool,
) -> Option<usize> {
    if offset > text.len() || !text.is_char_boundary(offset) {
        return None;
    }
    let mut source = 0;
    let mut written = 0;
    for line in text.split_inclusive('\n') {
        if end_boundary && offset == source {
            return Some(written);
        }
        let padding = if line.trim_end_matches('\n').is_empty() {
            0
        } else {
            4 * depth
        };
        if offset < source + line.len() {
            return Some(written + padding + offset - source);
        }
        written += padding + line.len();
        source += line.len();
    }
    (offset == source).then_some(written)
}

/// Render an already-proved enum unit; method spans keep byte equality while descendant
/// spans retain the original nonempty/UTF-8-boundary contract. Accessors are pure borrows.
pub fn nested_enum_source_text<F, M, T, N, D: Clone>(
    simple_name: &str,
    access_flags: u16,
    header_facts: (
        &jarde_reader::model::JvmString,
        u16,
        Option<&jarde_reader::model::JvmString>,
        &[jarde_reader::model::JvmString],
    ),
    fields: &[F],
    methods: &[M],
    projection: (&str, u64, &[u64], &[(u64, String)], &[u64], Option<&str>),
    method_texts: &[T],
    descendant: Option<&N>,
    field_facts: impl Fn(&F) -> (u64, &[String], Option<&String>, &[String]),
    method_facts: impl Fn(&M) -> (u64, &PhysicalMethodId, &String),
    staged_facts: impl Fn(&T) -> (u64, &String, &[D]),
    descendant_facts: impl Fn(&N) -> (&String, &[D]),
    range: impl Fn(&D) -> (usize, usize),
    set_range: impl Fn(&mut D, usize, usize),
    declaration_projection: impl FnOnce(usize, usize) -> D,
) -> Option<(String, Vec<D>)> {
    const ACC_PROTECTED: u16 = 0x0004;
    const ACC_STRICT: u16 = 0x0800;
    const SOURCE_CLASS_FLAGS: u16 =
        ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED | ACC_ABSTRACT | ACC_FINAL | ACC_STRICT;
    let flags = (header_facts.1 & !SOURCE_CLASS_FLAGS) | (access_flags & SOURCE_CLASS_FLAGS);
    let header = crate::declaration::nested_member_class_header(
        simple_name,
        header_facts.0,
        flags,
        header_facts.2,
        header_facts.3,
    )?;
    let (
        constants_text,
        backing_field_index,
        constant_field_indices,
        constructor_texts,
        implicit_method_indices,
        initializer_text,
    ) = projection;
    let mut text = format!("    {header} {{\n");
    let own_start = "    ".len();
    let mut derived = Vec::new();
    let mut first = true;
    text.push_str(&indent(constants_text, 1));
    if !constants_text.is_empty() {
        first = false;
    }
    for field in fields {
        let (field_index, annotations, declaration, markers) = field_facts(field);
        if field_index == backing_field_index || constant_field_indices.contains(&field_index) {
            continue;
        }
        if !first {
            text.push('\n');
        }
        first = false;
        for annotation in annotations {
            text.push_str(&indent(&format!("{annotation}\n"), 2));
        }
        let member = match declaration {
            Some(declaration) => declaration_member(declaration, markers),
            None => comment_member(markers),
        };
        text.push_str(&indent(&member, 1));
    }
    if let Some(initializer) = &initializer_text {
        if !first {
            text.push('\n');
        }
        first = false;
        text.push_str(&indent(initializer, 1));
    }
    for method in methods {
        let (method_index, identity, physical_text) = method_facts(method);
        if let Some((_, replacement)) = constructor_texts
            .iter()
            .find(|(index, _)| *index == method_index)
        {
            if !first {
                text.push('\n');
            }
            first = false;
            text.push_str(&indent(replacement, 1));
            continue;
        }
        if implicit_method_indices.contains(&method_index)
            || (initializer_text.is_some()
                && identity.name.0 == b"<clinit>"
                && identity.descriptor.0 == b"()V")
        {
            continue;
        }
        if !first {
            text.push('\n');
        }
        first = false;
        let Some(replacement) = method_texts
            .iter()
            .find(|replacement| staged_facts(replacement).0 == method_index)
        else {
            text.push_str(&indent(physical_text, 1));
            continue;
        };
        let offset = text.len();
        text.push_str(&indent(staged_facts(replacement).1, 1));
        for entry in staged_facts(replacement).2 {
            let source_span = staged_facts(replacement)
                .1
                .get(range(entry).0..range(entry).1)?;
            let start = indented_offset(staged_facts(replacement).1, range(entry).0, 1, false)?;
            let end = indented_offset(staged_facts(replacement).1, range(entry).1, 1, true)?;
            let mut translated = entry.clone();
            set_range(&mut translated, offset + start, offset + end);
            if text.get(range(&translated).0..range(&translated).1) != Some(source_span) {
                return None;
            }
            derived.push(translated);
        }
    }
    if let Some(descendant) = descendant {
        if !first {
            text.push('\n');
        }
        let (descendant_source, descendant_derived) = descendant_facts(descendant);
        let offset = text.len();
        let descendant_text = indent(descendant_source, 1);
        text.push_str(&descendant_text);
        for entry in descendant_derived {
            let start = indented_offset(descendant_source, range(entry).0, 1, false)?;
            let end = indented_offset(descendant_source, range(entry).1, 1, true)?;
            let mut translated = entry.clone();
            set_range(&mut translated, offset + start, offset + end);
            if range(&translated).0 >= range(&translated).1
                || !text.is_char_boundary(range(&translated).0)
                || !text.is_char_boundary(range(&translated).1)
            {
                return None;
            }
            derived.push(translated);
        }
    }
    text.push_str("    }\n");
    derived.push(declaration_projection(own_start, text.len() - 1));
    Some((text, derived))
}

/// Qualify the original one-element annotation source subset over raw member fields.
pub fn qualify_nested_annotation_element<M>(
    field_count: usize,
    methods: &[M],
    facts: impl Fn(&M) -> (&JvmBytes, &JvmBytes, Option<&str>, bool, &[String]),
) -> std::result::Result<&M, String> {
    if field_count != 0 || methods.len() != 1 {
        return Err(
            "nested annotation physical records are outside the one-element subset".to_owned(),
        );
    }
    let method = &methods[0];
    let (name, descriptor, declaration, no_body, markers) = facts(method);
    if name.0 != b"value"
        || descriptor.0 != b"()F"
        || declaration.is_none_or(|declaration| !declaration.contains(" default "))
        || !no_body
        || markers.len() != 1
        || !markers[0].contains("no body")
    {
        return Err(
            "nested annotation element does not reproduce one complete default declaration"
                .to_owned(),
        );
    }
    Ok(method)
}

/// Reject owner references outside the annotation declaration slice after child text replay.
pub fn qualify_nested_annotation_owner_references<'a>(
    child_binary: &JvmBytes,
    texts: impl Iterator<Item = &'a str>,
) -> std::result::Result<(), String> {
    let Some(binary_source_name) = source_class_name(child_binary) else {
        return Err("nested annotation has no exact source class name".to_owned());
    };
    let binary_leaf = binary_source_name.rsplit('.').next().unwrap();
    if texts.into_iter().any(|text| text.contains(binary_leaf)) {
        return Err(
            "owner source references the annotation binary name outside this declaration slice"
                .to_owned(),
        );
    }
    Ok(())
}

/// Spell only the annotation block: root retains physical facts, polling, anchors and commit.
pub fn nested_annotation_source_text(
    simple_name: &str,
    access_flags: u16,
    header_facts: (
        &jarde_reader::model::JvmString,
        u16,
        Option<&jarde_reader::model::JvmString>,
        &[jarde_reader::model::JvmString],
    ),
    method_text: &str,
) -> std::result::Result<(String, usize), String> {
    const VISIBILITY_AND_STATIC: u16 = 0x0001 | 0x0002 | 0x0004 | 0x0008;
    let flags =
        (header_facts.1 & !VISIBILITY_AND_STATIC) | (access_flags & (0x0001 | 0x0002 | 0x0004));
    let Some(header) = crate::declaration::nested_member_class_header(
        simple_name,
        header_facts.0,
        flags,
        header_facts.2,
        header_facts.3,
    ) else {
        return Err("nested annotation header has no faithful source name".to_owned());
    };
    let mut nested_text = String::new();
    let header_text = format!("    {header} {{\n");
    nested_text.push_str(&header_text);
    for line in method_text.split_inclusive('\n') {
        nested_text.push_str("    ");
        nested_text.push_str(line);
    }
    nested_text.push_str("    }\n");
    let nested_end = nested_text.len().saturating_sub(1);
    Ok((nested_text, nested_end))
}

/// Render the complete narrow/generic/capture member family from already-proved facts.
/// Accessors are pure borrows; factories construct only the caller's original physical anchors.
pub fn render_member_class<F, M, T: Clone, D: Clone>(
    simple_name: &str,
    access_flags: u16,
    header_facts: (
        &jarde_reader::model::JvmString,
        u16,
        Option<&jarde_reader::model::JvmString>,
        &[jarde_reader::model::JvmString],
    ),
    old_name: &str,
    annotation_uses: &[String],
    capture_field_index: Option<u64>,
    generic_static: Option<(&String, u64, &String, &PhysicalMethodId, u32)>,
    fields: &[F],
    methods: &[M],
    hidden_fields: &[u64],
    omitted_methods: &[u64],
    method_texts: &[T],
    field_facts: impl Fn(&F) -> (u64, &[String], Option<&String>, &[String]),
    method_facts: impl Fn(&M) -> (u64, &PhysicalMethodId, &String),
    staged_facts: impl Fn(&T) -> (u64, &String, &[D]),
    staged_mut: impl Fn(&mut T) -> (&mut String, &mut Vec<D>),
    range: impl Fn(&D) -> (usize, usize),
    set_range: impl Fn(&mut D, usize, usize),
    capture_projection: impl Fn(usize, usize, u64) -> D,
    class_projection: impl Fn(usize, usize) -> D,
    generic_projection: impl Fn(usize, usize, &PhysicalMethodId, u32, bool) -> D,
    field_projection: impl Fn(usize, usize, &F, bool) -> D,
    constructor_projection: impl Fn(usize, usize, &PhysicalMethodId) -> D,
) -> Option<(String, Vec<D>)> {
    const ACC_PROTECTED: u16 = 0x0004;
    const ACC_STRICT: u16 = 0x0800;
    const SOURCE_CLASS_FLAGS: u16 = ACC_PUBLIC
        | ACC_PRIVATE
        | ACC_PROTECTED
        | ACC_ABSTRACT
        | ACC_FINAL
        | ACC_STRICT
        | ACC_STATIC;
    let flags = (header_facts.1 & !SOURCE_CLASS_FLAGS) | (access_flags & SOURCE_CLASS_FLAGS);
    let mut out = String::new();
    let mut derived = Vec::new();
    for annotation in annotation_uses {
        out.push_str(&indent(&format!("{annotation}\n"), 1));
    }
    let mut class_header_declaration = generic_static.map_or_else(
        || {
            crate::declaration::class_declaration_with_types(
                simple_name,
                header_facts.0,
                flags,
                header_facts.2,
                header_facts.3,
                None,
                None,
                None,
            )
        },
        |generic| Some(generic.0.clone()),
    )?;
    if access_flags & ACC_STATIC != 0 {
        let visibility_len = ["public ", "protected ", "private "]
            .iter()
            .find(|prefix| class_header_declaration.starts_with(**prefix))
            .map_or(0, |prefix| prefix.len());
        class_header_declaration.insert_str(visibility_len, "static ");
    }
    let class_header = indent(&format!("{class_header_declaration} {{\n"), 1);
    let header_start = out.len();
    out.push_str(&class_header);
    if let Some(capture_field_index) = capture_field_index {
        derived.push(capture_projection(
            header_start,
            out.len() - 1,
            capture_field_index,
        ));
    } else {
        derived.push(class_projection(header_start, out.len() - 1));
    }
    if let Some(generic) = generic_static {
        derived.push(generic_projection(
            header_start,
            out.len() - 1,
            generic.3,
            generic.4,
            false,
        ));
        derived.push(generic_projection(
            header_start,
            out.len() - 1,
            generic.3,
            generic.4,
            true,
        ));
    }
    let mut first = true;
    for field in fields {
        let (field_index, annotations, declaration, markers) = field_facts(field);
        if capture_field_index == Some(field_index) {
            continue;
        }
        if hidden_fields.contains(&field_index) {
            // The child's own assert projection hid this field; the nested rendering hides it
            // too, anchored beside the header its flat text anchored it on.
            derived.push(field_projection(header_start, out.len() - 1, field, false));
            continue;
        }
        if !first {
            out.push('\n');
        }
        first = false;
        for annotation in annotations {
            out.push_str(&indent(&format!("{annotation}\n"), 2));
        }
        let text = match declaration {
            Some(_) if generic_static.is_some_and(|generic| generic.1 == field_index) => {
                declaration_member(generic_static.unwrap().2, &[])
            }
            Some(declaration) => declaration_member(declaration, markers),
            None => comment_member(markers),
        };
        let field_start = out.len();
        out.push_str(&indent(&text, 1));
        if generic_static.is_some_and(|generic| generic.1 == field_index) {
            derived.push(field_projection(field_start, out.len() - 1, field, true));
        }
    }
    for method in methods {
        let (method_index, identity, physical_text) = method_facts(method);
        if generic_static.is_some_and(|generic| generic.3 == identity) {
            continue;
        }
        // A member the child's own assert projection omitted (its switch line was all its
        // `<clinit>` held) carries no declaration here either, exactly as the child's flat text
        // omits it. Other first passes keep their members here as before: this channel reaches
        // the nested writer only beside the child's hidden fields.
        if !hidden_fields.is_empty() && omitted_methods.contains(&method_index) {
            continue;
        }
        if !first {
            out.push('\n');
        }
        first = false;
        if let Some(projected) = method_texts
            .iter()
            .find(|projected| staged_facts(projected).0 == method_index)
        {
            let mut projected = projected.clone();
            let (projected_text, projected_derived) = staged_mut(&mut projected);
            if identity.name.0 == b"<init>" && capture_field_index.is_none() {
                let old_token = format!("{old_name}(");
                let Some(local_start) = projected_text.find(&old_token) else {
                    return None;
                };
                let local_end = local_start + old_name.len();
                projected_text.replace_range(local_start..local_end, simple_name);
                projected_derived.push(constructor_projection(
                    local_start,
                    local_start + simple_name.len(),
                    identity,
                ));
            }
            append_family_method(
                &mut out,
                projected_text,
                projected_derived,
                true,
                &mut derived,
                &range,
                &set_range,
            )?;
        } else {
            out.push_str(&indent(physical_text, 1));
        }
    }
    out.push_str("    }\n");
    Some((out, derived))
}

/// Append one original staged method. Flat placement only adds its offset; nested placement
/// preserves the original indentation and source-span equality checks, including clone order.
pub fn append_family_method<D: Clone>(
    out: &mut String,
    method_text: &str,
    method_derived: &[D],
    nested: bool,
    derived: &mut Vec<D>,
    range: impl Fn(&D) -> (usize, usize),
    set_range: impl Fn(&mut D, usize, usize),
) -> Option<()> {
    let offset = out.len();
    if nested {
        out.push_str(&indent(method_text, 1));
    } else {
        out.push_str(method_text);
    }
    for entry in method_derived {
        let mut entry = entry.clone();
        if nested {
            let source_span = method_text.get(range(&entry).0..range(&entry).1)?;
            let start = indented_offset(method_text, range(&entry).0, 1, false)?;
            let end = indented_offset(method_text, range(&entry).1, 1, true)?;
            set_range(&mut entry, offset + start, offset + end);
            if out.get(range(&entry).0..range(&entry).1) != Some(source_span) {
                return None;
            }
        } else {
            let (start, end) = range(&entry);
            set_range(&mut entry, start + offset, end + offset);
        }
        derived.push(entry);
    }
    Some(())
}

// Ordinary class/interface static-initializer policy over selected same-run facts.

/// Whether the complete ordinary interface or class static initializer group passed proof.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClassSourceInitializerProof {
    /// The class is outside this proof's scope or has no ordinary-class runtime static group.
    NotApplicable,
    /// Every runtime field write in the group passed, in original `<clinit>` order.
    Proved {
        /// Physical field index and its same-run write position and BCI, in initializer order.
        fields: Vec<ClassSourceInitializerField>,
    },
    /// The whole runtime initializer group was refused with the first unmet proof condition.
    Refused { reason: String },
}

/// The write whose value would be eligible for a later source projection.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceInitializerField {
    /// The unchanged physical `field_info` table index.
    pub field_index: u64,
    /// The statement's order in the same-run `<clinit>` AST.
    pub write_order: usize,
    /// The `putstatic` BCI claimed by `field@1`.
    pub write_bci: u32,
}

/// One physical field and the facts needed to join static writes to a same-run `<clinit>`.
struct StaticInitializerField {
    index: u64,
    name: String,
    ty: JavaType,
    is_static: bool,
    has_constant_value: bool,
}

/// The all-or-nothing structural proof for an ordinary interface or class static field group.
///
/// This consumes the existing member read, source field records and same-run AST sidecars. It
/// performs no class read or recovery, and never consults either report text. The result is only a
/// verdict and origin mapping; source emission remains a later step.
#[allow(clippy::too_many_arguments)]
pub fn prove_static_initializer_group<'a, F: 'a, M: 'a>(
    declaration: (u16, &jarde_reader::model::JvmString),
    field_headers: &[MemberHeader],
    field_count: u64,
    fields_complete: bool,
    source_fields: &'a [F],
    constant_value_spellable: &[bool],
    method_headers: &[MemberHeader],
    method_count: u64,
    methods_complete: bool,
    source_methods: &'a [M],
    initializer_candidates: &[jarde_java::report::ClassInitializerCandidates],
    field_facts: impl Fn(
        &'a F,
    ) -> (
        u64,
        &'a jarde_reader::model::JvmString,
        &'a jarde_reader::model::JvmString,
        u16,
        bool,
    ),
    method_facts: impl Fn(
        &'a M,
    ) -> (
        &'a PhysicalMethodId,
        Option<(&'a crate::RecoveryReport, &'a ExecutionReport)>,
    ),
    budget: &mut Budget,
) -> Result<ClassSourceInitializerProof> {
    let (class_flags, class_name) = declaration;
    let is_interface = class_flags & ACC_INTERFACE != 0;
    let class_name = String::from_utf16(class_name.utf16()).ok();
    let Some(class_name) = class_name else {
        return Ok(initializer_refused(
            "the class's internal name is not a Unicode Java name",
        ));
    };
    if !fields_complete
        || !methods_complete
        || u64::try_from(field_headers.len()).unwrap_or(u64::MAX) != field_count
        || source_fields.len() != field_headers.len()
        || constant_value_spellable.len() != field_headers.len()
        || u64::try_from(method_headers.len()).unwrap_or(u64::MAX) != method_count
        || source_methods.len() != method_headers.len()
    {
        return Ok(initializer_refused(
            "the class member table or its published declarations are incomplete",
        ));
    }

    let mut fields = Vec::with_capacity(field_headers.len());
    let mut field_by_identity = std::collections::BTreeMap::<(Vec<u8>, Vec<u8>), usize>::new();
    let mut field_names = std::collections::BTreeSet::<String>::new();
    for (index, (header, source)) in field_headers.iter().zip(source_fields).enumerate() {
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        let (source_index, source_name, source_descriptor, source_flags, source_declared) =
            field_facts(source);
        let Some(name) = String::from_utf16(header.name.utf16()).ok() else {
            return Ok(initializer_refused(format!(
                "field at physical index {index} has no Java name"
            )));
        };
        if !jarde_java::is_java_identifier(&name) || !source_declared {
            return Ok(initializer_refused(format!(
                "field `{name}` at physical index {index} has no faithful Java declaration"
            )));
        }
        if !field_names.insert(name.clone()) {
            return Ok(initializer_refused(format!(
                "field name `{name}` is ambiguous in Java source"
            )));
        }
        let interface_flags = 0x0001 | 0x0008 | 0x0010;
        let forbidden_interface_flags = 0x0002 | 0x0004 | 0x0040 | 0x0080 | 0x4000;
        let class_field_flags = 0x0001 | 0x0002 | 0x0004 | 0x0008 | 0x0010 | 0x0040 | 0x0080;
        let visibility_flags = header.access_flags & (0x0001 | 0x0002 | 0x0004);
        if (is_interface
            && (header.access_flags & interface_flags != interface_flags
                || header.access_flags & forbidden_interface_flags != 0))
            || (!is_interface
                && (header.access_flags & !class_field_flags != 0
                    || visibility_flags.count_ones() > 1
                    || header.access_flags & (0x0010 | 0x0040) == (0x0010 | 0x0040)))
        {
            return Ok(initializer_refused(format!(
                "field `{name}` does not have unambiguous Java source flags"
            )));
        }
        if source_index != u64::try_from(index).unwrap_or(u64::MAX)
            || source_name != &header.name
            || source_descriptor != &header.descriptor
            || source_flags != header.access_flags
        {
            return Ok(initializer_refused(format!(
                "field `{name}` at physical index {index} does not match the published field identity"
            )));
        }
        let descriptor = header.descriptor.raw().0.clone();
        let Some(ty) = interface_field_type(&descriptor) else {
            return Ok(initializer_refused(format!(
                "field `{name}` has a descriptor with no Java source type"
            )));
        };
        let constant_value_count = header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"ConstantValue")
            .count();
        if constant_value_count > 1 {
            return Ok(initializer_refused(format!(
                "field `{name}` has more than one ConstantValue attribute"
            )));
        }
        let has_constant_value = constant_value_count == 1;
        if has_constant_value && !constant_value_spellable[index] {
            return Ok(initializer_refused(format!(
                "field `{name}` declares a ConstantValue that has no complete Java spelling"
            )));
        }
        let key = (header.name.raw().0.clone(), descriptor.clone());
        let field_index = fields.len();
        if field_by_identity.insert(key, field_index).is_some() {
            return Ok(initializer_refused(format!(
                "field `{name}` and its descriptor are duplicated"
            )));
        }
        fields.push(StaticInitializerField {
            index: source_index,
            name,
            ty,
            is_static: header.access_flags & 0x0008 != 0,
            has_constant_value,
        });
    }

    let clinit_positions: Vec<usize> = method_headers
        .iter()
        .enumerate()
        .filter_map(|(index, method)| (method.name.raw().0 == b"<clinit>").then_some(index))
        .collect();
    if clinit_positions
        .iter()
        .any(|index| method_headers[*index].descriptor.raw().0 != b"()V")
        || clinit_positions.len() > 1
    {
        return Ok(initializer_refused(
            "the class does not have one unique `<clinit>()V` member",
        ));
    }

    let runtime_field_count = fields
        .iter()
        .filter(|field| field.is_static && !field.has_constant_value)
        .count();
    if !is_interface && clinit_positions.is_empty() {
        return Ok(ClassSourceInitializerProof::NotApplicable);
    }
    if !is_interface && runtime_field_count == 0 {
        return Ok(ClassSourceInitializerProof::NotApplicable);
    }
    if !is_interface && fields.iter().any(|field| field.has_constant_value) {
        return Ok(initializer_refused(
            "an ordinary class field has a ConstantValue initialization phase",
        ));
    }
    let Some(&clinit_index) = clinit_positions.first() else {
        if runtime_field_count == 0 {
            return Ok(ClassSourceInitializerProof::Proved { fields: Vec::new() });
        }
        return Ok(initializer_refused(
            "the class has runtime-initialized static fields but no `<clinit>()V` member",
        ));
    };
    let clinit_header = &method_headers[clinit_index];
    if clinit_header.access_flags & 0x0008 == 0
        || (!is_interface && clinit_header.access_flags != 0x0008)
        || clinit_header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"Code")
            .count()
            != 1
    {
        return Ok(initializer_refused(
            "the unique `<clinit>()V` is not one complete static Code member",
        ));
    }
    let Some(clinit_method) = source_methods.get(clinit_index) else {
        return Ok(initializer_refused(
            "the `<clinit>()V` declaration was not published",
        ));
    };
    let (clinit_identity, recovered) = method_facts(clinit_method);
    let expected_clinit_name = jarde_reader::model::JvmBytes(b"<clinit>".to_vec());
    let expected_clinit_descriptor = jarde_reader::model::JvmBytes(b"()V".to_vec());
    if clinit_identity.name != expected_clinit_name
        || clinit_identity.descriptor != expected_clinit_descriptor
    {
        return Ok(initializer_refused(
            "the published initializer identity does not match `<clinit>()V`",
        ));
    }
    let Some((report, analysis_execution)) = recovered else {
        return Ok(initializer_refused(
            "the `<clinit>()V` body has no completed recovery",
        ));
    };
    if !report.produced()
        || !matches!(analysis_execution, ExecutionReport::Complete { .. })
        || !matches!(report.execution, ExecutionReport::Complete { .. })
        || report.quality != jarde_jvm::ir::Quality::Structured
        || !report.fallbacks.is_empty()
    {
        return Ok(initializer_refused(
            "the `<clinit>()V` analysis or recovery did not complete without fallback",
        ));
    }
    let matching_candidates: Vec<_> = initializer_candidates
        .iter()
        .filter(|candidates| candidates.member.as_ref() == Some(clinit_identity))
        .collect();
    let [candidates] = matching_candidates.as_slice() else {
        return Ok(initializer_refused(
            "the completed `<clinit>()V` run has no unique same-run candidate sequence",
        ));
    };
    if candidates.has_exception_handlers {
        return Ok(initializer_refused(
            "the `<clinit>()V` Code has an exception-table edge or an incomplete handler read",
        ));
    }

    let mut writes_by_field = vec![None::<(usize, u32)>; fields.len()];
    let mut ordered_writes = Vec::<(usize, &jarde_java::report::ClassInitializerFieldWrite)>::new();
    let mut last_write_bci = None;
    let mut saw_return = false;
    for (position, step) in candidates.steps.iter().enumerate() {
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        match step {
            jarde_java::report::ClassInitializerStep::FieldWrite(write) => {
                if write.order != position
                    || write.source.primary().bci() != write.bci
                    || last_write_bci.is_some_and(|previous| previous >= write.bci)
                {
                    return Ok(initializer_refused(
                        "the `<clinit>()V` candidate order does not match its write origins",
                    ));
                }
                last_write_bci = Some(write.bci);
                if write.owner != JvmBytes::from_unicode(&class_name) || !write.is_static {
                    return Ok(initializer_refused(format!(
                        "write at BCI {} does not target a static field of this class",
                        write.bci
                    )));
                }
                let key = (write.name.0.clone(), write.descriptor.0.clone());
                let Some(&field_index) = field_by_identity.get(&key) else {
                    return Ok(initializer_refused(format!(
                        "write at BCI {} does not name one field in the complete field table",
                        write.bci
                    )));
                };
                let field = &fields[field_index];
                if !field.is_static || field.has_constant_value {
                    return Ok(initializer_refused(format!(
                        "write at BCI {} does not target a runtime static field `{}`",
                        write.bci, field.name
                    )));
                }
                if write.spelled_name != field.name || write.op != jarde_java::ast::AssignOp::Assign
                {
                    return Ok(initializer_refused(format!(
                        "write at BCI {} is not a simple assignment to field `{}`",
                        write.bci, field.name
                    )));
                }
                if writes_by_field[field_index]
                    .replace((write.order, write.bci))
                    .is_some()
                {
                    return Ok(initializer_refused(format!(
                        "field `{}` is written more than once by `<clinit>()V`",
                        field.name
                    )));
                }
                ordered_writes.push((field_index, write));
            }
            jarde_java::report::ClassInitializerStep::Other {
                order,
                kind: jarde_java::report::ClassInitializerStatementKind::Return,
                ..
            } if *order == position && position + 1 == candidates.steps.len() && !saw_return => {
                saw_return = true;
            }
            jarde_java::report::ClassInitializerStep::Other { kind, .. } => {
                return Ok(initializer_refused(format!(
                    "`<clinit>()V` contains an unclaimed top-level effect ({kind:?})"
                )));
            }
        }
    }
    if !saw_return {
        return Ok(initializer_refused(
            "the `<clinit>()V` candidate sequence has no unique trailing normal return",
        ));
    }
    for (index, field) in fields.iter().enumerate() {
        if field.has_constant_value {
            if writes_by_field[index].is_some() {
                return Ok(initializer_refused(format!(
                    "ConstantValue field `{}` also has a `<clinit>()V` write",
                    field.name
                )));
            }
        } else if field.is_static && writes_by_field[index].is_none() {
            return Ok(initializer_refused(format!(
                "runtime field `{}` has no unique `<clinit>()V` write",
                field.name
            )));
        }
    }

    let mut proof_fields = Vec::with_capacity(runtime_field_count);
    for (field_index, write) in ordered_writes {
        let field = &fields[field_index];
        if write.field_reads.is_none() {
            return Ok(initializer_refused(format!(
                "field read identity inside RHS of `{}` is unproved",
                field.name
            )));
        }
        let type_matches = match write.value.presented.as_ref() {
            Some(actual_type) => actual_type == &field.ty,
            None => {
                matches!(write.value.kind, ExprKind::Null)
                    && matches!(field.ty, JavaType::Reference(_))
            }
        };
        if !type_matches {
            let actual = write
                .value
                .presented
                .as_ref()
                .map(JavaType::spell)
                .unwrap_or("unknown");
            return Ok(initializer_refused(format!(
                "RHS of field `{}` is presented as `{}` but its descriptor requires `{}`",
                field.name,
                actual,
                field.ty.spell()
            )));
        }
        let Some(reads) = write.field_reads.as_ref() else {
            unreachable!("the sidecar read identity was checked above")
        };
        let mut read_claims = std::collections::BTreeMap::<
            u32,
            Vec<&jarde_java::report::ClassInitializerFieldRead>,
        >::new();
        for read in reads {
            read_claims.entry(read.bci).or_default().push(read);
        }
        let mut consumed_reads = std::collections::BTreeSet::new();
        let mut own_runtime_reads = Vec::<(u32, usize)>::new();
        let mut expression_state = InitializerExpressionState::default();
        if let Some(reason) = validate_initializer_expression(
            &write.value,
            write.bci,
            &class_name,
            &fields,
            &field_by_identity,
            &read_claims,
            &mut consumed_reads,
            &mut own_runtime_reads,
            &mut expression_state,
            budget,
        )? {
            return Ok(initializer_refused(reason));
        }
        if consumed_reads.len() != reads.len() {
            return Ok(initializer_refused(format!(
                "the AST and field@1 read claims inside RHS of `{}` are not one-to-one",
                field.name
            )));
        }
        if expression_state.has_unknown_static_field {
            return Ok(initializer_refused(format!(
                "RHS of field `{}` reads an external static field whose ConstantValue and source-level initialization behavior are unknown",
                field.name
            )));
        }
        if !expression_state.has_nonconstant_shape
            && field_headers[field_index].access_flags & 0x0010 != 0
        {
            return Ok(initializer_refused(format!(
                "RHS of field `{}` is a Java constant expression and would change initialization phase",
                field.name
            )));
        }
        for (read_bci, target_index) in own_runtime_reads {
            if !is_interface
                && writes_by_field[target_index].is_some_and(|(order, _)| order >= write.order)
            {
                return Ok(initializer_refused(format!(
                    "read at BCI {read_bci} names a forward static field"
                )));
            }
            let Some((target_order, target_write_bci)) = writes_by_field[target_index] else {
                return Ok(initializer_refused(format!(
                    "read at BCI {read_bci} names a runtime field with no proved write"
                )));
            };
            let source_read_precedes_target_write = write.order <= target_order;
            let bytecode_read_precedes_target_write = read_bci < target_write_bci;
            if source_read_precedes_target_write != bytecode_read_precedes_target_write {
                return Ok(initializer_refused(format!(
                    "read at BCI {read_bci} would observe a different initialization phase after source-order projection"
                )));
            }
        }
        proof_fields.push(ClassSourceInitializerField {
            field_index: field.index,
            write_order: write.order,
            write_bci: write.bci,
        });
    }
    Ok(ClassSourceInitializerProof::Proved {
        fields: proof_fields,
    })
}

#[doc(hidden)]
pub enum InitializerProjectionFailure {
    Refused(String),
    Stopped(StopReason),
    Error(Error),
}

/// Stage an admitted runtime initializer group only after every RHS fragment has been emitted.
/// The supplier commits the returned fragments atomically; source order never changes physical field order.
pub fn stage_static_initializer_group<'a, F: 'a, M: 'a>(
    proof: &ClassSourceInitializerProof,
    source_names: jarde_java::names::SourceClassNames<'_>,
    fields: &'a [F],
    methods: &'a [M],
    candidate_runs: &[jarde_java::report::ClassInitializerCandidates],
    field_facts: impl Fn(&'a F) -> (u64, bool),
    method_identity: impl Fn(&'a M) -> &'a PhysicalMethodId,
    budget: &mut Budget,
) -> std::result::Result<Option<(Vec<usize>, Vec<(usize, String)>)>, InitializerProjectionFailure> {
    let ClassSourceInitializerProof::Proved {
        fields: proved_fields,
    } = proof
    else {
        return Ok(None);
    };

    let clinit_methods: Vec<_> = methods
        .iter()
        .filter(|method| {
            method_identity(method).name.0 == b"<clinit>"
                && method_identity(method).descriptor.0 == b"()V"
        })
        .collect();
    let clinit_method = match clinit_methods.as_slice() {
        [method] => Some(*method),
        [] if proved_fields.is_empty() => None,
        [] => {
            return Err(InitializerProjectionFailure::Refused(
                "the proved initializer writes have no retained `<clinit>()V` member".to_owned(),
            ));
        }
        _ => {
            return Err(InitializerProjectionFailure::Refused(
                "the proved initializer group has more than one `<clinit>()V` member".to_owned(),
            ));
        }
    };
    if proved_fields.is_empty() {
        return Ok(Some(((0..fields.len()).collect(), Vec::new())));
    }
    let Some(clinit_method) = clinit_method else {
        return Err(InitializerProjectionFailure::Refused(
            "the proved initializer writes have no retained `<clinit>()V` member".to_owned(),
        ));
    };

    let matching: Vec<_> = candidate_runs
        .iter()
        .filter(|candidates| candidates.member.as_ref() == Some(method_identity(clinit_method)))
        .collect();
    let candidates = match matching.as_slice() {
        [candidates] => *candidates,
        _ => {
            return Err(InitializerProjectionFailure::Refused(
                "the proved initializer group has no unique retained `<clinit>()V` candidate"
                    .to_owned(),
            ));
        }
    };

    let mut field_order = Vec::with_capacity(fields.len());
    let mut projected_fields = std::collections::BTreeSet::new();
    let mut staged_initializers = Vec::with_capacity(proved_fields.len());
    let mut previous_write_order = None;
    for proved in proved_fields {
        if previous_write_order.is_some_and(|previous| previous >= proved.write_order) {
            return Err(InitializerProjectionFailure::Refused(
                "the proved field list is not in strict `<clinit>()V` write order".to_owned(),
            ));
        }
        previous_write_order = Some(proved.write_order);
        let matching_fields: Vec<_> = fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                (field_facts(field).0 == proved.field_index).then_some(index)
            })
            .collect();
        let [physical_index] = matching_fields.as_slice() else {
            return Err(InitializerProjectionFailure::Refused(format!(
                "proved physical field index {} is absent or ambiguous during projection",
                proved.field_index
            )));
        };
        let physical_index = *physical_index;
        if !projected_fields.insert(physical_index) {
            return Err(InitializerProjectionFailure::Refused(format!(
                "physical field index {} occurs more than once in the projection",
                proved.field_index
            )));
        }
        field_order.push(physical_index);

        let Some(jarde_java::report::ClassInitializerStep::FieldWrite(write)) =
            candidates.steps.get(proved.write_order)
        else {
            return Err(InitializerProjectionFailure::Refused(format!(
                "proved write position {} is absent from the retained `<clinit>()V` candidate",
                proved.write_order
            )));
        };
        if write.order != proved.write_order || write.bci != proved.write_bci {
            return Err(InitializerProjectionFailure::Refused(format!(
                "proved field index {} no longer matches write position {} at BCI {}",
                proved.field_index, proved.write_order, proved.write_bci
            )));
        }
        if !field_facts(&fields[physical_index]).1 {
            return Err(InitializerProjectionFailure::Refused(format!(
                "proved field index {} has no retained declaration",
                proved.field_index
            )));
        }
        let fragment = jarde_java::report::emit_class_initializer_value(
            &write.value,
            method_identity(clinit_method),
            source_names,
            budget,
        )
        .map_err(InitializerProjectionFailure::Stopped)?;
        staged_initializers.push((physical_index, fragment));
    }

    for (physical_index, _) in fields.iter().enumerate() {
        if projected_fields.insert(physical_index) {
            field_order.push(physical_index);
        }
    }
    if field_order.len() != fields.len() {
        return Err(InitializerProjectionFailure::Refused(
            "the projected source field order is not a complete physical-field permutation"
                .to_owned(),
        ));
    }

    Ok(Some((field_order, staged_initializers)))
}

/// Returns a reason in the report's existing refusal vocabulary without making a partial proof.
fn initializer_refused(reason: impl Into<String>) -> ClassSourceInitializerProof {
    ClassSourceInitializerProof::Refused {
        reason: reason.into(),
    }
}

fn interface_field_type(descriptor: &[u8]) -> Option<JavaType> {
    let facts = descriptor_facts(descriptor, DescriptorKind::Field).ok()?;
    crate::type_of_component(facts.single()?)
}

fn is_java_constant_variable_type(ty: &JavaType) -> bool {
    matches!(
        ty,
        JavaType::Boolean
            | JavaType::Byte
            | JavaType::Char
            | JavaType::Short
            | JavaType::Int
            | JavaType::Long
            | JavaType::Float
            | JavaType::Double
    ) || matches!(ty, JavaType::Reference(name) if name == "java.lang.String")
}

#[derive(Default)]
struct InitializerExpressionState {
    has_nonconstant_shape: bool,
    has_unknown_static_field: bool,
    effect_bcis: std::collections::BTreeSet<u32>,
}

/// Walks one RHS AST once, matching every field node to its exact sidecar read and finding whether
/// the emitted source could be a Java constant expression. The explicit stack keeps a deep but
/// already-bounded expression from adding recursion to class-source assembly.
#[allow(clippy::too_many_arguments)]
fn validate_initializer_expression(
    root: &Expr,
    write_bci: u32,
    class_name: &str,
    fields: &[StaticInitializerField],
    field_by_identity: &std::collections::BTreeMap<(Vec<u8>, Vec<u8>), usize>,
    read_claims: &std::collections::BTreeMap<
        u32,
        Vec<&jarde_java::report::ClassInitializerFieldRead>,
    >,
    consumed_reads: &mut std::collections::BTreeSet<u32>,
    own_runtime_reads: &mut Vec<(u32, usize)>,
    state: &mut InitializerExpressionState,
    budget: &mut Budget,
) -> Result<Option<String>> {
    let mut pending = vec![root];
    while let Some(expression) = pending.pop() {
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        match &expression.kind {
            ExprKind::Field { receiver, name } => {
                let read_bci = expression.origin.primary().bci();
                if read_bci >= write_bci {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} is not before its owning putstatic at BCI {write_bci}"
                    )));
                }
                let Some([claim]) = read_claims.get(&read_bci).map(Vec::as_slice) else {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} has no unique field@1 claim"
                    )));
                };
                if !consumed_reads.insert(read_bci)
                    || claim.name != JvmBytes::from_unicode(name)
                    || !claim.is_static && claim.owner == JvmBytes::from_unicode(&class_name)
                {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} does not match its unique claim"
                    )));
                }
                let Some(claim_type) = interface_field_type(claim.descriptor.0.as_slice()) else {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} has an unspellable claimed descriptor"
                    )));
                };
                if expression.presented.as_ref() != Some(&claim_type) {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} does not have its claimed descriptor type"
                    )));
                }
                let Some(expected_owner_path) =
                    claim.owner.to_unicode().map(|name| name.replace('/', "."))
                else {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} has no scalar source owner"
                    )));
                };
                if claim.is_static {
                    if !matches!(&receiver.kind, ExprKind::Path(path) if *path == expected_owner_path)
                    {
                        return Ok(Some(format!(
                            "static field read at BCI {read_bci} is not qualified by its proven owner `{expected_owner_path}`"
                        )));
                    }
                } else {
                    let expected_receiver = JavaType::Reference(expected_owner_path);
                    if receiver.presented.as_ref() != Some(&expected_receiver) {
                        return Ok(Some(format!(
                            "instance field read at BCI {read_bci} has no receiver of its proven owner type"
                        )));
                    }
                    pending.push(receiver);
                }
                if claim.owner == JvmBytes::from_unicode(&class_name) {
                    let key = (claim.name.0.clone(), claim.descriptor.0.clone());
                    let Some(&target_index) = field_by_identity.get(&key) else {
                        return Ok(Some(format!(
                            "same-class field read at BCI {read_bci} does not resolve to one declared field"
                        )));
                    };
                    let field = &fields[target_index];
                    if !field.is_static {
                        return Ok(Some(format!(
                            "same-class instance field read at BCI {read_bci} cannot be moved to a static declaration"
                        )));
                    }
                    if field.has_constant_value {
                        // A ConstantValue with a primitive or String type is a Java constant
                        // variable. Other reference-typed ConstantValue attributes (legal as class
                        // file data but not Java constant variables) still require a field read.
                        if !is_java_constant_variable_type(&field.ty) {
                            state.has_nonconstant_shape = true;
                        }
                    } else {
                        own_runtime_reads.push((read_bci, target_index));
                        state.has_nonconstant_shape = true;
                    }
                } else if !claim.is_static {
                    state.has_nonconstant_shape = true;
                } else {
                    state.has_unknown_static_field = true;
                }
            }
            ExprKind::Call { receiver, args, .. } => {
                if let Some(reason) = record_initializer_effect(expression, write_bci, state) {
                    return Ok(Some(reason));
                }
                state.has_nonconstant_shape = true;
                for argument in args.iter().rev() {
                    pending.push(argument);
                }
                if let Some(receiver) = receiver {
                    pending.push(receiver);
                }
            }
            ExprKind::New {
                qualifier, args, ..
            } => {
                if let Some(reason) = record_initializer_effect(expression, write_bci, state) {
                    return Ok(Some(reason));
                }
                state.has_nonconstant_shape = true;
                for argument in args.iter().rev() {
                    pending.push(argument);
                }
                if let Some(qualifier) = qualifier {
                    pending.push(qualifier);
                }
            }
            ExprKind::NewArray {
                lengths,
                initializers,
                ..
            } => {
                if let Some(reason) = record_initializer_effect(expression, write_bci, state) {
                    return Ok(Some(reason));
                }
                state.has_nonconstant_shape = true;
                if let Some(initializers) = initializers {
                    pending.extend(initializers.iter().rev());
                }
                pending.extend(lengths.iter().rev());
            }
            ExprKind::Index { array, index } => {
                if let Some(reason) = record_initializer_effect(expression, write_bci, state) {
                    return Ok(Some(reason));
                }
                state.has_nonconstant_shape = true;
                pending.push(index);
                pending.push(array);
            }
            ExprKind::PostfixUpdate { .. } | ExprKind::PrefixUpdate { .. } => {
                return Ok(Some(
                    "a postfix update has no static-initializer evaluation proof".to_owned(),
                ));
            }
            ExprKind::ArrayLength { array } => {
                if let Some(reason) = record_initializer_effect(expression, write_bci, state) {
                    return Ok(Some(reason));
                }
                state.has_nonconstant_shape = true;
                pending.push(array);
            }
            ExprKind::Cast { ty, value, .. } => {
                if !matches!(
                    ty,
                    JavaType::Boolean
                        | JavaType::Byte
                        | JavaType::Char
                        | JavaType::Short
                        | JavaType::Int
                        | JavaType::Long
                        | JavaType::Float
                        | JavaType::Double
                ) && !matches!(ty, JavaType::Reference(name) if name == "java.lang.String")
                {
                    state.has_nonconstant_shape = true;
                }
                pending.push(value);
            }
            ExprKind::Not { value } | ExprKind::Neg { value } => pending.push(value),
            ExprKind::InstanceOf { value, .. } => {
                state.has_nonconstant_shape = true;
                pending.push(value);
            }
            ExprKind::Binary { left, right, .. } => {
                pending.push(right);
                pending.push(left);
            }
            ExprKind::Concat { parts } => {
                pending.extend(parts.iter().rev().map(|part| &part.value));
            }
            ExprKind::Lambda { .. } | ExprKind::MethodReference { .. } => {
                return Ok(Some(
                    "a lambda or method reference has no complete initializer type and evaluation proof".to_owned(),
                ));
            }
            ExprKind::Conditional { .. } => {
                return Ok(Some(
                    "conditional expression constant-expression phase proof is unavailable"
                        .to_owned(),
                ));
            }
            ExprKind::Local(name) => {
                return Ok(Some(format!("RHS refers to unscoped local `{name}`")));
            }
            ExprKind::ArrayAssign { .. } => {
                return Ok(Some(
                    "an array update has no static-initializer evaluation proof".to_owned(),
                ));
            }
            ExprKind::LocalAssign { .. } => {
                return Ok(Some(
                    "RHS assigns a local whose initializer scope is unproved".to_owned(),
                ));
            }
            ExprKind::QualifiedThis { .. } => {
                return Ok(Some("RHS refers to a lexical outer instance".to_owned()));
            }
            ExprKind::Null | ExprKind::ClassLiteral { .. } => {
                state.has_nonconstant_shape = true;
            }
            ExprKind::Integer(_)
            | ExprKind::IntegerConstantName { .. }
            | ExprKind::Boolean(_)
            | ExprKind::Long(_)
            | ExprKind::Float(_)
            | ExprKind::Double(_)
            | ExprKind::Str(_)
            | ExprKind::Path(_)
            | ExprKind::Super { .. } => {}
        }
    }
    Ok(None)
}

fn record_initializer_effect(
    expression: &Expr,
    write_bci: u32,
    state: &mut InitializerExpressionState,
) -> Option<String> {
    let bci = expression.origin.primary().bci();
    if bci >= write_bci {
        return Some(format!(
            "RHS effect at BCI {bci} is not before its owning putstatic at BCI {write_bci}"
        ));
    }
    if !state.effect_bcis.insert(bci) {
        return Some(format!(
            "RHS effect at BCI {bci} would be evaluated more than once"
        ));
    }
    None
}

// Complete enum int-field source qualification over on-demand selected facts.

/// Certify every field leaf in the narrow enum int-expression grammar before source emission.
/// The enum Code proof retains the original symbolic owner/name/descriptor; this boundary only
/// establishes that emitting that access as Java source is legal and does not turn a constant
/// variable read into compile-time inlining.
#[doc(hidden)]
pub fn certify_enum_int_static_fields<R>(
    enum_owner: &[u8],
    constants: &[ProvedEnumConstant],
    mut enum_attributes: impl FnMut(&mut Budget) -> Result<jarde_reader::classfile::AttributeFacts>,
    mut selected_owner: impl FnMut(&[u8], &mut Budget) -> Result<Option<R>>,
    read_facts: impl Fn(&R) -> (&[u8], &ClassMemberFacts),
    mut owner_attributes: impl FnMut(
        &R,
        &mut Budget,
    ) -> Result<(
        Vec<jarde_reader::classfile::CpEntryFacts>,
        jarde_reader::classfile::AttributeFacts,
    )>,
    budget: &mut Budget,
) -> Result<Option<String>> {
    fn collect(
        argument: &ProvedEnumIntArgument,
        refs: &mut std::collections::BTreeMap<(Vec<u8>, Vec<u8>, Vec<u8>), String>,
        budget: &mut Budget,
    ) -> Result<()> {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        use ProvedEnumIntArgument as Argument;
        match argument {
            Argument::Literal { .. } => {}
            Argument::StaticField {
                owner,
                name,
                descriptor,
                source_owner,
                ..
            } => {
                refs.entry((owner.clone(), name.clone(), descriptor.clone()))
                    .or_insert_with(|| source_owner.clone());
            }
            Argument::Add { left, right, .. } => {
                collect(left, refs, budget)?;
                collect(right, refs, budget)?;
            }
            Argument::Ternary {
                when_true,
                when_false,
                ..
            } => {
                collect(when_true, refs, budget)?;
                collect(when_false, refs, budget)?;
            }
        }
        Ok(())
    }

    let mut refs = std::collections::BTreeMap::new();
    for constant in constants {
        if let Some(ProvedEnumSourceArgument::Int(argument)) = &constant.source_argument {
            collect(argument, &mut refs, budget)?;
        }
    }
    for ((owner, name, descriptor), source_owner) in refs {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if let Some(reason) = enum_int_static_field_owner_refusal(enum_owner, &owner, &descriptor) {
            return Ok(Some(reason));
        }
        let faithful_short_name =
            jarde_java::names::source_class_name(&jarde_reader::model::JvmBytes(owner.clone()))
                .is_some_and(|spelled| spelled.rsplit('.').next() == Some(source_owner.as_str()));
        if !faithful_short_name {
            return Ok(Some(format!(
                "enum constructor argument owner {} is not a same-package source short name",
                source_owner
            )));
        }
        if enum_owner_has_type_name_shadow(
            enum_owner,
            &owner,
            &enum_attributes(budget)?.inner_classes,
        ) {
            return Ok(Some(format!(
                "enum member type shadows the same-package source name {}",
                source_owner
            )));
        }
        let Some(selected) = selected_owner(&owner, budget)? else {
            return Ok(Some(format!(
                "enum constructor argument field owner {} did not resolve uniquely in the selected environment",
                source_owner
            )));
        };
        let (bytes, facts) = read_facts(&selected);
        let class_version = bytes.get(4..8).map(|version| {
            (
                u16::from_be_bytes([version[0], version[1]]),
                u16::from_be_bytes([version[2], version[3]]),
            )
        });
        if facts.stopped_at.is_some()
            || facts.field_count != u64::try_from(facts.fields.len()).unwrap_or(u64::MAX)
            || facts.this_class.raw().0.as_slice() != owner
            || class_version != Some((0, 52))
            || facts.access_flags & (0x0002 | 0x0004 | 0x0200 | 0x1000 | 0x2000 | 0x8000) != 0
        {
            return Ok(Some(format!(
                "enum constructor argument field owner {} is not a complete Java 8 top-level accessible class",
                source_owner
            )));
        }
        let (owner_pool, owner_attributes) = owner_attributes(&selected, budget)?;
        if owner_attributes.enclosing_method.is_some()
            || owner_attributes.inner_classes.iter().any(|row| {
                jarde_reader::classfile::cp_class_name(&owner_pool, row.class_index)
                    .is_ok_and(|name| name.0 == owner)
            })
        {
            return Ok(Some(format!(
                "enum constructor argument field owner {} is not a top-level source type",
                source_owner
            )));
        }
        let mut selected_field = None;
        for field in &facts.fields {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if field.name.raw().0.as_slice() == name
                && field.descriptor.raw().0.as_slice() == descriptor
            {
                if selected_field.is_some() {
                    return Ok(Some(format!(
                        "enum constructor argument field {}.{} is ambiguous",
                        source_owner,
                        String::from_utf8_lossy(&name)
                    )));
                }
                selected_field = Some(field);
            }
        }
        let Some(field) = selected_field else {
            return Ok(Some(format!(
                "enum constructor argument field {}.{} is absent",
                source_owner,
                String::from_utf8_lossy(&name)
            )));
        };
        const ACC_PRIVATE_FIELD: u16 = 0x0002;
        const ACC_STATIC_FIELD: u16 = 0x0008;
        const ACC_FINAL_FIELD: u16 = 0x0010;
        const ACC_SYNTHETIC_FIELD: u16 = 0x1000;
        if field.access_flags & ACC_STATIC_FIELD == 0
            || field.access_flags & (ACC_PRIVATE_FIELD | ACC_FINAL_FIELD | ACC_SYNTHETIC_FIELD) != 0
        {
            return Ok(Some(format!(
                "enum constructor argument field {}.{} is not a same-package accessible static non-final int",
                source_owner,
                String::from_utf8_lossy(&name)
            )));
        }
    }
    Ok(None)
}

#[doc(hidden)]
pub fn enum_int_static_field_owner_refusal(
    enum_owner: &[u8],
    owner: &[u8],
    descriptor: &[u8],
) -> Option<String> {
    fn package(name: &[u8]) -> &[u8] {
        name.iter()
            .rposition(|byte| *byte == b'/')
            .map_or(&name[..0], |slash| &name[..slash])
    }
    if owner == enum_owner {
        return Some(
            "an enum constructor argument reads a static field from its own enum before initialization".to_owned(),
        );
    }
    if owner.contains(&b'$') {
        return Some(
            "an enum constructor argument field owner needs an unproved member-class source name"
                .to_owned(),
        );
    }
    if package(enum_owner) != package(owner) || descriptor != b"I" {
        return Some(
            "an enum constructor argument field is outside the proved same-package int subset"
                .to_owned(),
        );
    }
    None
}

#[doc(hidden)]
pub fn enum_owner_has_type_name_shadow(
    enum_owner: &[u8],
    field_owner: &[u8],
    inner_classes: &[jarde_reader::classfile::InnerClassFacts],
) -> bool {
    fn package(owner: &[u8]) -> &[u8] {
        owner
            .iter()
            .rposition(|byte| *byte == b'/')
            .map(|slash| &owner[..slash])
            .unwrap_or(&owner[..0])
    }
    fn simple_name(owner: &[u8]) -> &[u8] {
        owner.rsplit(|byte| *byte == b'/').next().unwrap_or(owner)
    }
    package(enum_owner) == package(field_owner)
        && inner_classes.iter().any(|row| {
            row.inner_name
                .as_ref()
                .is_some_and(|name| name.0 == simple_name(field_owner))
        })
}

/// Stage the whole common primitive-array constructor group over same-run borrowed records.
/// Root owns reads and publication. Member writer facts are borrowed metadata; Java applies the
/// shared current-body and placement policy at the original qualification/staging points.
#[doc(hidden)]
pub fn stage_instance_array_initializer_group<'a, F: 'a, M: 'a, A: 'a>(
    class_flags: u16,
    class_name: &[u8],
    headers: &[MemberHeader],
    fields: &'a [F],
    methods: &'a [M],
    asts: &'a [A],
    field_headers: &[MemberHeader],
    field_order: Option<&[usize]>,
    member_texts: &[(u64, String)],
    already_staged_members: &[u64],
    field_facts: impl Fn(&'a F) -> (&'a [u8], &'a [u8], u16, Option<&'a str>),
    method_facts: impl Fn(
        &'a M,
    ) -> (
        u64,
        &'a PhysicalMethodId,
        Option<(&'a crate::RecoveryReport, &'a ExecutionReport)>,
    ),
    ast_facts: impl Fn(
        &'a A,
    ) -> (
        &'a PhysicalMethodId,
        &'a crate::report::ClassSourceMethodAst,
    ),
    method_writer_facts: impl Fn(
        &'a M,
    ) -> (
        Option<&'a str>,
        &'a str,
        &'a [String],
        &'a crate::report::MemberAnnotationUses,
        &'a jarde_reader::model::JvmString,
        &'a jarde_reader::model::JvmString,
    ),
    budget: &mut Budget,
) -> std::result::Result<
    Option<(Vec<(usize, String)>, Vec<(u64, String)>)>,
    InitializerProjectionFailure,
> {
    if class_flags & (ACC_INTERFACE | jarde_java::facts::ACC_ANNOTATION | 0x4000 | 0x8000) != 0
        || headers.len() != methods.len()
        || field_headers.len() != fields.len()
    {
        return Ok(None);
    }

    // `headers` and `methods` are the complete physical member census. Non-constructors remain in
    // that census but need no constructor AST. Count first, then allocate the selected constructor
    // records only after their exact IR cost has been charged.
    let mut constructor_count = 0usize;
    for header in headers {
        budget.poll().map_err(InitializerProjectionFailure::Error)?;
        budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)
            .map_err(InitializerProjectionFailure::Error)?;
        if header.name.raw().0 == b"<init>" {
            if !(header
                .attributes
                .iter()
                .any(|shell| shell.name.raw().0 == b"Code")
                && crate::declaration::spellable_method_descriptor(&header.descriptor.raw().0))
            {
                return Ok(None);
            }
            constructor_count = constructor_count.saturating_add(1);
        }
    }
    if constructor_count == 0 {
        return Ok(None);
    }

    // A non-static ConstantValue is unusual but still physically stated. Moving any constructor
    // assignment across it could change the field writer's chosen initialization representation.
    for header in field_headers {
        budget.poll().map_err(InitializerProjectionFailure::Error)?;
        budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)
            .map_err(InitializerProjectionFailure::Error)?;
        if header.access_flags & 0x0008 == 0
            && header
                .attributes
                .iter()
                .any(|shell| shell.name.raw().0 == b"ConstantValue")
        {
            return Ok(None);
        }
    }

    budget
        .charge(
            CountedBudgetDimension::IrItems,
            u64::try_from(constructor_count).unwrap_or(u64::MAX),
        )
        .map_err(InitializerProjectionFailure::Error)?;
    let mut selected = Vec::with_capacity(constructor_count);
    for header in headers {
        budget.poll().map_err(InitializerProjectionFailure::Error)?;
        budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)
            .map_err(InitializerProjectionFailure::Error)?;
        if header.name.raw().0 != b"<init>" {
            continue;
        }
        let mut ast_match = None;
        for candidate in asts {
            budget.poll().map_err(InitializerProjectionFailure::Error)?;
            budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .map_err(InitializerProjectionFailure::Error)?;
            if ast_facts(candidate).0.name.0 == header.name.raw().0
                && ast_facts(candidate).0.descriptor.0 == header.descriptor.raw().0
            {
                if ast_match.is_some() {
                    return Ok(None);
                }
                ast_match = Some(candidate);
            }
        }
        let Some(candidate) = ast_match else {
            return Ok(None);
        };
        let (identity, ast) = ast_facts(candidate);
        let mut method_match = None;
        for method in methods {
            budget.poll().map_err(InitializerProjectionFailure::Error)?;
            budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .map_err(InitializerProjectionFailure::Error)?;
            if *method_facts(method).1 == *identity {
                if method_match.is_some() {
                    return Ok(None);
                }
                method_match = Some(method);
            }
        }
        let Some(method) = method_match else {
            return Ok(None);
        };
        let Some((report, analysis_execution)) = method_facts(method).2 else {
            return Ok(None);
        };
        if !matches!(&report.execution, ExecutionReport::Complete { .. })
            || !matches!(analysis_execution, ExecutionReport::Complete { .. })
            || report.quality != jarde_jvm::ir::Quality::Structured
            || report.representation != jarde_jvm::ir::Representation::Java
            || report.content != jarde_java::RecoveryContent::ContainsStatements
            || !report.fallbacks.is_empty()
        {
            return Ok(None);
        }
        let Some(projection) =
            jarde_java::report::class_source_instance_array_constructor_projection(ast, budget)
                .map_err(InitializerProjectionFailure::Stopped)?
        else {
            return Ok(None);
        };
        if projection.member != *identity {
            return Ok(None);
        }
        selected.push((method_facts(method).0, method, ast, projection));
    }

    let Some((_, _, first_ast, first)) = selected.first() else {
        return Ok(None);
    };
    for (_, _, ast, projection) in selected.iter().skip(1) {
        budget.poll().map_err(InitializerProjectionFailure::Error)?;
        budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)
            .map_err(InitializerProjectionFailure::Error)?;
        if projection.writes.len() != first.writes.len() {
            return Ok(None);
        }
        for (left, right) in first.writes.iter().zip(&projection.writes) {
            budget.poll().map_err(InitializerProjectionFailure::Error)?;
            budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .map_err(InitializerProjectionFailure::Error)?;
            if left.owner != right.owner
                || left.name != right.name
                || left.descriptor != right.descriptor
                || !jarde_java::report::class_source_instance_array_rhs_equal(
                    first_ast, left.bci, ast, right.bci, budget,
                )
                .map_err(InitializerProjectionFailure::Stopped)?
            {
                return Ok(None);
            }
        }
    }

    let stage_items = u64::try_from(first.writes.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(2);
    budget
        .charge(CountedBudgetDimension::IrItems, stage_items)
        .map_err(InitializerProjectionFailure::Error)?;
    let mut staged_fields = Vec::with_capacity(first.writes.len());
    let mut field_indices = Vec::with_capacity(first.writes.len());
    for write in &first.writes {
        budget.poll().map_err(InitializerProjectionFailure::Error)?;
        budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)
            .map_err(InitializerProjectionFailure::Error)?;
        if write.owner.0.as_slice() != class_name
            || !primitive_array_descriptor(&write.element, &write.descriptor.0)
        {
            return Ok(None);
        }
        let mut found = None;
        for (index, field) in fields.iter().enumerate() {
            budget.poll().map_err(InitializerProjectionFailure::Error)?;
            budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .map_err(InitializerProjectionFailure::Error)?;
            if field_facts(field).0 == write.name.0.as_slice()
                && field_facts(field).1 == write.descriptor.0.as_slice()
            {
                if found.is_some() {
                    return Ok(None);
                }
                found = Some((index, field));
            }
        }
        let Some((index, field)) = found else {
            return Ok(None);
        };
        if field_facts(field).2 & 0x0008 != 0
            || field_facts(field).3.is_none_or(|text| text.contains('='))
        {
            return Ok(None);
        }
        let mut matching_header = None;
        for header in field_headers {
            budget.poll().map_err(InitializerProjectionFailure::Error)?;
            budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .map_err(InitializerProjectionFailure::Error)?;
            if header.name.raw().0 == write.name.0.as_slice()
                && header.descriptor.raw().0 == write.descriptor.0.as_slice()
            {
                if matching_header.is_some() {
                    return Ok(None);
                }
                matching_header = Some(header);
            }
        }
        let Some(header) = matching_header else {
            return Ok(None);
        };
        if header
            .attributes
            .iter()
            .any(|shell| shell.name.raw().0 == b"ConstantValue")
            || header
                .attributes
                .iter()
                .any(|shell| shell.name.raw().0 == b"Signature")
        {
            return Ok(None);
        }
        let Some(fragment) = jarde_java::report::emit_class_source_instance_array_initializer(
            first_ast, write.bci, budget,
        )
        .map_err(InitializerProjectionFailure::Stopped)?
        else {
            return Ok(None);
        };
        let declaration = field_facts(field).3.expect("checked above");
        budget
            .charge(
                CountedBudgetDimension::OutputBytes,
                u64::try_from(declaration.len().saturating_add(fragment.len())).unwrap_or(u64::MAX),
            )
            .map_err(InitializerProjectionFailure::Error)?;
        let projected = format!("{declaration}{fragment}");
        staged_fields.push((index, projected));
        field_indices.push(index);

        let Some(calls) =
            jarde_java::report::class_source_instance_array_calls(first_ast, write.bci, budget)
                .map_err(InitializerProjectionFailure::Stopped)?
        else {
            return Ok(None);
        };
        for call in calls {
            let target = &call.target;
            budget.poll().map_err(InitializerProjectionFailure::Error)?;
            budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .map_err(InitializerProjectionFailure::Error)?;
            if target.kind() != jarde_java::facts::InvokeKind::Static
                || target.raw_owner().0 != class_name
                || target.is_interface_reference()
            {
                return Ok(None);
            }
            let mut matching_header = None;
            for header in headers {
                budget.poll().map_err(InitializerProjectionFailure::Error)?;
                budget
                    .charge(CountedBudgetDimension::AnalysisSteps, 1)
                    .map_err(InitializerProjectionFailure::Error)?;
                if header.name.raw().0 == target.raw_name().0
                    && header.descriptor.raw().0 == target.raw_descriptor().0
                {
                    if matching_header.is_some() {
                        return Ok(None);
                    }
                    matching_header = Some(header);
                }
            }
            let Some(header) = matching_header else {
                return Ok(None);
            };
            if header.access_flags & 0x0008 == 0
                || header
                    .attributes
                    .iter()
                    .any(|shell| shell.name.raw().0 == b"Exceptions")
            {
                return Ok(None);
            }
            let Ok(descriptor) =
                descriptor_facts(&target.raw_descriptor().0, DescriptorKind::Method)
            else {
                return Ok(None);
            };
            let Some(result_type) = descriptor.result().and_then(primitive_descriptor_type) else {
                return Ok(None);
            };
            if descriptor.parameters().len() != call.arguments.len()
                || Some(result_type) != call.presented
            {
                return Ok(None);
            }
            for (argument, parameter) in call.arguments.iter().zip(descriptor.parameters()) {
                budget.poll().map_err(InitializerProjectionFailure::Error)?;
                budget
                    .charge(CountedBudgetDimension::AnalysisSteps, 1)
                    .map_err(InitializerProjectionFailure::Error)?;
                if parameter.dimensions() != 0 || primitive_descriptor_type(parameter) != *argument
                {
                    return Ok(None);
                }
            }
        }
    }

    let mut prior = None;
    for index in &field_indices {
        budget.poll().map_err(InitializerProjectionFailure::Error)?;
        budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)
            .map_err(InitializerProjectionFailure::Error)?;
        let position = if let Some(order) = field_order {
            let mut found = None;
            for (position, candidate) in order.iter().enumerate() {
                budget.poll().map_err(InitializerProjectionFailure::Error)?;
                budget
                    .charge(CountedBudgetDimension::AnalysisSteps, 1)
                    .map_err(InitializerProjectionFailure::Error)?;
                if candidate == index {
                    found = Some(position);
                    break;
                }
            }
            let Some(position) = found else {
                return Ok(None);
            };
            position
        } else {
            *index
        };
        if prior.is_some_and(|previous| previous >= position) {
            return Ok(None);
        }
        prior = Some(position);
    }

    let stage_methods = u64::try_from(selected.len()).unwrap_or(u64::MAX);
    budget
        .charge(CountedBudgetDimension::IrItems, stage_methods)
        .map_err(InitializerProjectionFailure::Error)?;
    let mut staged_methods = Vec::with_capacity(selected.len());
    for (member_index, method, ast, projection) in &selected {
        budget.poll().map_err(InitializerProjectionFailure::Error)?;
        budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)
            .map_err(InitializerProjectionFailure::Error)?;
        for (index, _) in member_texts.iter() {
            budget.poll().map_err(InitializerProjectionFailure::Error)?;
            budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .map_err(InitializerProjectionFailure::Error)?;
            if *index == *member_index {
                return Ok(None);
            }
        }
        for index in already_staged_members {
            budget.poll().map_err(InitializerProjectionFailure::Error)?;
            budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .map_err(InitializerProjectionFailure::Error)?;
            if *index == *member_index {
                return Ok(None);
            }
        }
        let Some((report, _)) = method_facts(method).2 else {
            return Ok(None);
        };
        let original_statements = jarde_java::report::emit_class_source_method_ast(ast, 4, budget)
            .map_err(InitializerProjectionFailure::Stopped)?;
        budget
            .charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(report.text.len().saturating_add(original_statements.len()))
                    .unwrap_or(u64::MAX),
            )
            .map_err(InitializerProjectionFailure::Error)?;
        let (declaration, current_text, markers, annotations, name, descriptor) =
            method_writer_facts(method);
        if !matches_current_text(
            declaration,
            current_text,
            &report.text,
            Some(report),
            markers,
            name,
            descriptor,
        ) || !lines_match(&report.text, &original_statements)
        {
            return Ok(None);
        }
        budget
            .charge(
                CountedBudgetDimension::IrItems,
                u64::try_from(projection.writes.len()).unwrap_or(u64::MAX),
            )
            .map_err(InitializerProjectionFailure::Error)?;
        let mut removed = Vec::with_capacity(projection.writes.len());
        for write in &projection.writes {
            budget.poll().map_err(InitializerProjectionFailure::Error)?;
            budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .map_err(InitializerProjectionFailure::Error)?;
            removed.push(write.bci);
        }
        let body = jarde_java::report::emit_class_source_instance_array_constructor_body(
            ast, &removed, budget,
        )
        .map_err(InitializerProjectionFailure::Stopped)?;
        let Some(text) =
            assert_projection_text(declaration, Some(report), markers, annotations, &body)
        else {
            return Ok(None);
        };
        budget
            .charge(
                CountedBudgetDimension::OutputBytes,
                u64::try_from(text.len()).unwrap_or(u64::MAX),
            )
            .map_err(InitializerProjectionFailure::Error)?;
        staged_methods.push((*member_index, text));
    }

    // The proof, all fields, all text-only constructor placements and every output charge are
    // complete. Poll before the no-fail commit; reserved IR cost already covers each staged
    // record. No body emission is registered: this projection has no re-spellable anchor table,
    // and its member_texts entry reserves the constructor against a later competing rewrite.
    budget.poll().map_err(InitializerProjectionFailure::Error)?;
    Ok(Some((staged_fields, staged_methods)))
}
fn primitive_descriptor_type(
    component: &jarde_reader::classfile::DescriptorComponent,
) -> Option<JavaType> {
    if component.dimensions() != 0 {
        return None;
    }
    let jarde_reader::classfile::Base::Primitive(base) = component.base() else {
        return None;
    };
    Some(match base {
        jarde_reader::classfile::BaseType::Boolean => JavaType::Boolean,
        jarde_reader::classfile::BaseType::Byte => JavaType::Byte,
        jarde_reader::classfile::BaseType::Char => JavaType::Char,
        jarde_reader::classfile::BaseType::Short => JavaType::Short,
        jarde_reader::classfile::BaseType::Int => JavaType::Int,
        jarde_reader::classfile::BaseType::Long => JavaType::Long,
        jarde_reader::classfile::BaseType::Float => JavaType::Float,
        jarde_reader::classfile::BaseType::Double => JavaType::Double,
    })
}

fn primitive_array_descriptor(element: &JavaType, descriptor: &[u8]) -> bool {
    let suffix = match element {
        JavaType::Boolean => "Z",
        JavaType::Byte => "B",
        JavaType::Char => "C",
        JavaType::Short => "S",
        JavaType::Int => "I",
        JavaType::Long => "J",
        JavaType::Float => "F",
        JavaType::Double => "D",
        JavaType::Reference(_) => return false,
    };
    descriptor == format!("[{suffix}").as_bytes()
}

#[doc(hidden)]
pub fn lines_match(artifact_text: &str, emitted_statements: &str) -> bool {
    let artifact_statements = artifact_text
        .split_once("\n{\n")
        .and_then(|(_, rest)| rest.rsplit_once("\n}"))
        .map(|(statements, _)| statements);
    let Some(artifact_statements) = artifact_statements else {
        return false;
    };
    let artifact_lines: Vec<&str> = artifact_statements
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let emitted_lines: Vec<&str> = emitted_statements
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    artifact_lines == emitted_lines
}

/// Borrowed source envelope/statements only; construction grants no artifact binding.
#[doc(hidden)]
pub struct Artifact<'a> {
    /// The envelope: the `//` lines the artifact opens with (`@method`, `@declaration`, the
    /// statement that the presentation is not claimed to compile).
    pub envelope: &'a str,
    /// The statements, with neither the artifact's opening brace nor its closing one.
    pub statements: &'a str,
}

/// Splits one produced artifact into its envelope and its statements, by the shape the emitter of
/// [`jarde_java::recover`] writes: comment lines, a line holding `{` alone, the statements at one
/// level of indentation, and a line holding `}` alone as the last line.
///
/// The split is a reading of that shape and it is deliberately strict: the statements of a body are
/// indented, so `{` alone can only be the block's own opening brace, and `}` alone as the last line
/// can only be its closing one. Anything else is not placed at all — the caller quotes it line by
/// line instead of guessing where a statement starts — because placing a statement outside the
/// member it belongs to would be worse than not placing it.
#[doc(hidden)]
pub fn artifact(text: &str) -> Option<Artifact<'_>> {
    let mut open: Option<usize> = None;
    let mut close: Option<usize> = None;
    let mut at = 0usize;
    for line in text.split_inclusive('\n') {
        let bare = line.trim_end_matches('\n');
        if bare == "{" && open.is_none() {
            open = Some(at);
        }
        if bare == "}" {
            close = Some(at);
        }
        at += line.len();
    }
    let open = open?;
    let close = close?;
    if close < open + 2 {
        return None;
    }
    let trailing = &text[close..];
    if trailing != "}\n" && trailing != "}" {
        return None;
    }
    Some(Artifact {
        envelope: &text[..open],
        statements: &text[open + 2..close],
    })
}

/// One artifact this presentation could not place, quoted line by line as comments: nothing a run
/// produced is dropped, and none of it is read as a statement.
fn quote(text: &str) -> String {
    let mut quoted = String::new();
    for line in text.lines() {
        quoted.push_str("// ");
        quoted.push_str(line);
        quoted.push('\n');
    }
    quoted
}

/// What one member's block holds.
#[doc(hidden)]
pub enum Placed<'a> {
    /// The artifact's own envelope and statements, as the recovery layer wrote them.
    Block(Artifact<'a>),
    /// A produced artifact whose shape this presentation does not recognize: quoted line by line.
    Quoted(&'a str),
    /// No artifact at all: the marker is the whole body.
    Without,
}

/// One member written as a declaration and a block, at one level of indentation.
#[doc(hidden)]
pub fn block_member(declaration: &str, placed: Placed<'_>, markers: &[String]) -> String {
    let mut out = format!("    {declaration} {{\n");
    for marker in markers {
        out.push_str(&indent(&format!("{marker}\n"), 2));
    }
    match placed {
        Placed::Without => {}
        Placed::Block(artifact) => {
            // The envelope is written one level in from the class body (it stood outside the
            // artifact's braces, and this presentation writes it inside them), and the statements
            // keep the one level of indentation their own block gave them: both land at the same
            // depth, one level below the declaration above them.
            out.push_str(&indent(artifact.envelope, 2));
            out.push_str(&indent(artifact.statements, 1));
        }
        Placed::Quoted(text) => out.push_str(&indent(&quote(text), 2)),
    }
    out.push_str("    }\n");
    out
}

/// Exact byte length of one already-known block rendering, used to reserve output budget before
/// allocating the projection text.
fn indented_text_bytes(text: &str, depth: usize) -> u64 {
    let pad = u64::try_from(depth.saturating_mul(4)).unwrap_or(u64::MAX);
    text.split_inclusive('\n').fold(0u64, |total, line| {
        let line_bytes = u64::try_from(line.len()).unwrap_or(u64::MAX);
        let indentation = if line.trim_end_matches('\n').is_empty() {
            0
        } else {
            pad
        };
        total.saturating_add(line_bytes).saturating_add(indentation)
    })
}

#[doc(hidden)]
pub fn block_member_text_bytes(
    declaration: &str,
    envelope: &str,
    statements: &str,
    markers: &[String],
    annotations: &crate::report::MemberAnnotationUses,
) -> u64 {
    let marker_bytes = markers.iter().fold(0u64, |total, marker| {
        total
            .saturating_add(indented_text_bytes(marker, 2))
            .saturating_add(1)
    });
    let annotation_bytes = annotations.uses.iter().fold(0u64, |total, annotation| {
        total
            .saturating_add(4)
            .saturating_add(u64::try_from(annotation.len()).unwrap_or(u64::MAX))
            .saturating_add(1)
    });
    4u64.saturating_add(u64::try_from(declaration.len()).unwrap_or(u64::MAX))
        .saturating_add(3) // ` {\n`
        .saturating_add(marker_bytes)
        .saturating_add(indented_text_bytes(envelope, 2))
        .saturating_add(indented_text_bytes(statements, 1))
        .saturating_add(6) // `    }\n`
        .saturating_add(annotation_bytes)
}

#[doc(hidden)]
pub fn declaration_member_text_bytes(
    declaration: &str,
    markers: &[String],
    annotations: &crate::report::MemberAnnotationUses,
) -> u64 {
    let marker_bytes = markers.iter().fold(0u64, |total, marker| {
        total
            .saturating_add(indented_text_bytes(marker, 1))
            .saturating_add(1)
    });
    let annotation_bytes = annotations.uses.iter().fold(0u64, |total, annotation| {
        total
            .saturating_add(4)
            .saturating_add(u64::try_from(annotation.len()).unwrap_or(u64::MAX))
            .saturating_add(1)
    });
    marker_bytes
        .saturating_add(4)
        .saturating_add(u64::try_from(declaration.len()).unwrap_or(u64::MAX))
        .saturating_add(2) // `;\n`
        .saturating_add(annotation_bytes)
}

#[doc(hidden)]
pub fn comment_name(value: &jarde_reader::model::JvmString) -> String {
    crate::comment_text(&value.escaped())
}

#[doc(hidden)]
pub fn member_label(
    name: &jarde_reader::model::JvmString,
    descriptor: &jarde_reader::model::JvmString,
) -> String {
    format!("{}{}", comment_name(name), comment_name(descriptor))
}

#[doc(hidden)]
pub fn assert_projection_text(
    declaration: Option<&str>,
    recovery: Option<&crate::RecoveryReport>,
    markers: &[String],
    annotations: &crate::report::MemberAnnotationUses,
    body: &str,
) -> Option<String> {
    let declaration = declaration?;
    let report = recovery?;
    let original = artifact(&report.text)?;
    let staged = Artifact {
        envelope: original.envelope,
        statements: body,
    };
    Some(prefix_method_annotations(
        block_member(declaration, Placed::Block(staged), markers),
        annotations,
    ))
}

#[doc(hidden)]
pub fn has_only_explanation_marker(
    recovery: Option<&crate::RecoveryReport>,
    markers: &[String],
    name: &jarde_reader::model::JvmString,
    descriptor: &jarde_reader::model::JvmString,
) -> bool {
    let Some(report) = recovery else {
        return false;
    };
    report.content == crate::RecoveryContent::ExplanationOnly
        && markers
            == [format!(
                "// jarde: not recovered: the recovery run for `{}` produced no statement (explanation only); the artifact's own comment lines are below",
                member_label(name, descriptor)
            )]
}

#[doc(hidden)]
pub fn matches_current_text(
    declaration: Option<&str>,
    current_text: &str,
    recovery_text: &str,
    physical_recovery: Option<&crate::RecoveryReport>,
    markers: &[String],
    name: &jarde_reader::model::JvmString,
    descriptor: &jarde_reader::model::JvmString,
) -> bool {
    let Some(declaration) = declaration else {
        return false;
    };
    let Some(artifact) = artifact(recovery_text) else {
        return false;
    };
    let Some(region) = current_body_region(current_text, declaration) else {
        return false;
    };
    // This exact marker belongs to the physical run's own fallback, not to a prior body
    // rewrite. Include it in the comparison; every byte of the body is still checked.
    let own_marker = has_only_explanation_marker(physical_recovery, markers, name, descriptor)
        .then_some(markers)
        .unwrap_or(&[]);
    let expected: Vec<&str> = own_marker
        .iter()
        .map(String::as_str)
        .chain(artifact.envelope.lines())
        .chain(artifact.statements.lines())
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    expected == region
}

fn current_body_region<'a>(current_text: &'a str, declaration: &str) -> Option<Vec<&'a str>> {
    let opening = format!("    {declaration} {{");
    let mut opening_at = None;
    let mut closing_at = None;
    for (index, line) in current_text.lines().enumerate() {
        if line == opening && opening_at.is_none() {
            opening_at = Some(index);
        }
        if line == "    }" {
            closing_at = Some(index);
        }
    }
    let opening = opening_at?;
    let closing = closing_at.filter(|closing| *closing > opening)?;
    Some(
        current_text
            .lines()
            .skip(opening + 1)
            .take(closing - opening - 1)
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect(),
    )
}

#[doc(hidden)]
pub fn placed_artifact_span(
    body: &Artifact<'_>,
    start: usize,
    end: usize,
    block_prefix: usize,
) -> Option<(usize, usize)> {
    let envelope_end = body.envelope.len();
    let (part, local_start, local_end, depth, prefix) = if end <= envelope_end {
        (body.envelope, start, end, 2, block_prefix)
    } else {
        let statement_start = envelope_end + 2;
        if start < statement_start || end > statement_start + body.statements.len() {
            return None;
        }
        (
            body.statements,
            start - statement_start,
            end - statement_start,
            1,
            block_prefix + indent(body.envelope, 2).len(),
        )
    };
    let mapped_start = prefix + indented_offset(part, local_start, depth, false)?;
    let mapped_end = prefix + indented_offset(part, local_end, depth, true)?;
    Some((mapped_start, mapped_end))
}

/// A class's own typed row is only a candidate until the selected child's row agrees.
#[derive(Clone, Debug, Eq, PartialEq)]
#[doc(hidden)]
pub struct FamilyRootCandidate {
    pub child_name: Vec<u8>,
    pub simple_name: String,
    pub access_flags: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[doc(hidden)]
pub enum FamilyRootScan {
    Absent,
    Refused(String),
    Candidate(FamilyRootCandidate),
    DeclarationPair([FamilyRootCandidate; 2]),
    /// Every direct static member row, in InnerClasses order: the family a static fold
    /// projection renders as nested `static class`/`static interface` declarations. Two or more
    /// rows reach this arm; a single static row still selects through [`Self::Candidate`], where
    /// the proved narrow channels keep their priority and a refused projection falls back here.
    StaticMembers(Vec<FamilyRootCandidate>),
    /// The mixed family: the direct static member rows that fold beside one non-static
    /// candidate. The static subset no longer waits for the non-static candidate's absence —
    /// each static row that meets the per-row criteria takes the same fold road a pure static
    /// family's rows do, while the candidate keeps the separated narrow channel exactly as
    /// before; neither member blocks the other.
    StaticMembersWithInstance {
        statics: Vec<FamilyRootCandidate>,
        candidate: FamilyRootCandidate,
    },
}

/// Find the one Java 8 nested enum slice this class-source writer supports. The typed row supplies
/// the identity candidate; `$` is checked only as a consistency constraint after that evidence.
pub fn scan_nested_enum_root(
    root: &[u8],
    major_version: Option<u16>,
    resolved_inner_classes: &[jarde_reader::classfile::ResolvedInnerClass],
    budget: &mut Budget,
) -> Result<FamilyRootScan> {
    if major_version != Some(52) {
        return Ok(FamilyRootScan::Absent);
    }
    const ACC_ENUM: u16 = 0x4000;
    const ACC_STATIC: u16 = 0x0008;
    const VISIBILITY: u16 = 0x0001 | 0x0002 | 0x0004;
    let mut candidate = None;
    for row in resolved_inner_classes {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if row.outer_class.as_deref() != Some(root) || row.access_flags & ACC_ENUM == 0 {
            continue;
        }
        if candidate.is_some() {
            return Ok(FamilyRootScan::Refused(
                "owner has multiple direct nested enum rows outside the two-level subset"
                    .to_owned(),
            ));
        }
        let Some(simple_name) = row.inner_name.as_ref().and_then(JvmBytes::to_unicode) else {
            return Ok(FamilyRootScan::Refused(
                "nested enum row has no scalar MUTF-8 source name".to_owned(),
            ));
        };
        if !jarde_java::names::is_java_identifier(&simple_name)
            || row.access_flags & ACC_STATIC == 0
            || (row.access_flags & VISIBILITY).count_ones() > 1
            || row.class != [root, b"$", &JvmBytes::from_unicode(&simple_name).0].concat()
        {
            return Ok(FamilyRootScan::Refused(
                "nested enum row is not a source-spellable static member".to_owned(),
            ));
        }
        candidate = Some(FamilyRootCandidate {
            child_name: row.class.clone(),
            simple_name: simple_name.to_owned(),
            access_flags: row.access_flags,
        });
    }
    if let Some(selected) = &candidate {
        let mut matching_rows = 0usize;
        for row in resolved_inner_classes {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if row.outer_class.as_deref() == Some(root)
                && (row.class == selected.child_name
                    || row
                        .inner_name
                        .as_ref()
                        .is_some_and(|name| name == &JvmBytes::from_unicode(&selected.simple_name)))
            {
                matching_rows += 1;
                if matching_rows > 1 {
                    return Ok(FamilyRootScan::Refused(
                        "nested enum owner row is duplicated or conflicts with another member row"
                            .to_owned(),
                    ));
                }
            }
        }
    }
    Ok(candidate.map_or(FamilyRootScan::Absent, FamilyRootScan::Candidate))
}

/// Find the one direct Java 8 member annotation supported by class-source projection. The typed
/// InnerClasses row supplies identity; the binary `$` spelling is checked only for consistency.
pub fn scan_nested_annotation_root(
    root: &[u8],
    major_version: Option<u16>,
    resolved_inner_classes: &[jarde_reader::classfile::ResolvedInnerClass],
    budget: &mut Budget,
) -> Result<FamilyRootScan> {
    if major_version != Some(52) {
        return Ok(FamilyRootScan::Absent);
    }
    const ACC_INTERFACE: u16 = 0x0200;
    const ACC_ANNOTATION: u16 = 0x2000;
    const ACC_STATIC: u16 = 0x0008;
    const KIND: u16 = ACC_INTERFACE | ACC_ANNOTATION;
    const VISIBILITY: u16 = 0x0001 | 0x0002 | 0x0004;
    let mut candidate = None;
    let mut direct_rows = 0usize;
    for row in resolved_inner_classes {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if row.outer_class.as_deref() != Some(root) {
            continue;
        }
        direct_rows += 1;
        if row.access_flags & KIND != KIND {
            continue;
        }
        if candidate.is_some() {
            return Ok(FamilyRootScan::Refused(
                "owner has multiple direct nested annotation rows outside the one-child subset"
                    .to_owned(),
            ));
        }
        let Some(simple_name) = row.inner_name.as_ref().and_then(JvmBytes::to_unicode) else {
            return Ok(FamilyRootScan::Refused(
                "nested annotation row has no scalar MUTF-8 source name".to_owned(),
            ));
        };
        if !jarde_java::names::is_java_identifier(&simple_name)
            || row.access_flags & ACC_STATIC == 0
            || (row.access_flags & VISIBILITY).count_ones() > 1
            || row.class != [root, b"$", &JvmBytes::from_unicode(&simple_name).0].concat()
        {
            return Ok(FamilyRootScan::Refused(
                "nested annotation row is not a source-spellable static member".to_owned(),
            ));
        }
        candidate = Some(FamilyRootCandidate {
            child_name: row.class.clone(),
            simple_name: simple_name.to_owned(),
            access_flags: row.access_flags,
        });
    }
    let Some(selected) = candidate else {
        return Ok(FamilyRootScan::Absent);
    };
    // This slice has one direct child total. A sibling could introduce a competing source family
    // or require another declaration position, so refuse before selecting any projection.
    if direct_rows != 1 {
        return Ok(FamilyRootScan::Refused(
            "owner has additional direct member rows outside the one-child annotation subset"
                .to_owned(),
        ));
    }
    let matching_rows = resolved_inner_classes
        .iter()
        .filter(|row| {
            row.outer_class.as_deref() == Some(root)
                && (row.class == selected.child_name
                    || row
                        .inner_name
                        .as_ref()
                        .is_some_and(|name| name == &JvmBytes::from_unicode(&selected.simple_name)))
        })
        .count();
    if matching_rows != 1 {
        return Ok(FamilyRootScan::Refused(
            "nested annotation owner row is duplicated or conflicts with another member row"
                .to_owned(),
        ));
    }
    Ok(FamilyRootScan::Candidate(selected))
}

/// Qualify and stage the complete assert group from same-run borrowed records.
/// No artifact reads or root writer callbacks occur here. The supplier publishes only a full plan.
#[doc(hidden)]
pub fn stage_class_source_assert_statements<'a, F: 'a, M: 'a>(
    class_flags: u16,
    this_class: &[u8],
    fields: &'a [F],
    methods: &'a [M],
    method_asts: &[(PhysicalMethodId, crate::report::ClassSourceMethodAst)],
    field_uses: &[crate::report::ClassSourceEnumSwitchFieldUse],
    major_version: Option<u16>,
    resolved_inner_classes: &[jarde_reader::classfile::ResolvedInnerClass],
    initializer_field_order: Option<&[usize]>,
    enum_projection_taken: bool,
    anonymous_candidate: bool,
    integer_constant_candidate_count: usize,
    staged_member_text_count: usize,
    structure_complete: bool,
    execution: &ExecutionReport,
    field_facts: impl Fn(&'a F) -> (&'a [u8], &'a [u8], u16, u64),
    method_facts: impl Fn(&'a M) -> (u64, &'a PhysicalMethodId, Option<&'a crate::RecoveryReport>),
    writer_facts: impl Fn(
        &'a M,
    ) -> (
        Option<&'a str>,
        &'a str,
        &'a [String],
        &'a crate::report::MemberAnnotationUses,
        &'a jarde_reader::model::JvmString,
        &'a jarde_reader::model::JvmString,
    ),
    budget: &mut Budget,
) -> std::result::Result<
    Option<(u64, Vec<(usize, String, String)>, Option<usize>)>,
    InitializerProjectionFailure,
> {
    const ACC_SYNTHETIC: u16 = 0x1000;
    const ACC_STATIC_FINAL: u16 = 0x0008 | 0x0010;
    // The channels that would re-assemble this class's text from unretained inputs (the narrow
    // member family, a declaration pair, a nested enum or annotation, an anonymous interface
    // child, integer-constant names) each refuse beside a staged projection, which would drop a
    // child the text carries today. The static member fold — the A1 shape — rebuilds from the
    // retained channels instead and carries the projection through.
    // A member text another projection already composed (a lambda companion rewrite) is not this
    // AST's own emission, and a family writer that refuses beside staged texts would drop a
    // child the text carries — so a staged first pass keeps the physical presentation.
    if class_flags & (0x0200 | 0x2000 | 0x4000 | 0x8000) != 0
        || !structure_complete
        || !matches!(execution, ExecutionReport::Complete { .. })
        || initializer_field_order.is_some()
        || enum_projection_taken
        || anonymous_candidate
        || integer_constant_candidate_count != 0
        || staged_member_text_count != 0
    {
        return Ok(None);
    }
    match scan_nested_enum_root(this_class, major_version, resolved_inner_classes, budget)
        .map_err(InitializerProjectionFailure::Error)?
    {
        FamilyRootScan::Absent | FamilyRootScan::Refused(_) => {}
        _ => return Ok(None),
    }
    match scan_nested_annotation_root(this_class, major_version, resolved_inner_classes, budget)
        .map_err(InitializerProjectionFailure::Error)?
    {
        FamilyRootScan::Absent | FamilyRootScan::Refused(_) => {}
        _ => return Ok(None),
    }
    // The field: exactly one synthetic static final boolean `$assertionsDisabled`.
    let switch_fields: Vec<_> = fields
        .iter()
        .filter(|field| {
            field_facts(field).0 == jarde_java::report::ASSERT_SWITCH_FIELD_NAME.as_bytes()
                && field_facts(field).1 == b"Z"
                && field_facts(field).2 & (ACC_STATIC_FINAL | ACC_SYNTHETIC)
                    == (ACC_STATIC_FINAL | ACC_SYNTHETIC)
        })
        .collect();
    let [switch_field] = switch_fields.as_slice() else {
        return Ok(None);
    };
    let field_name = jarde_java::report::ASSERT_SWITCH_FIELD_NAME;
    // The one `<clinit>` with a retained AST and its proved initialization line. The index, not
    // the record: the commit at the end mutates the records this pass has only read.
    let clinit_index = methods.iter().position(|method| {
        method_facts(method).1.name.0 == b"<clinit>"
            && method_facts(method).1.descriptor.0 == b"()V"
    });
    let Some(clinit_index) = clinit_index else {
        return Ok(None);
    };
    let clinit = &methods[clinit_index];
    if method_facts(clinit)
        .2
        .is_none_or(|report| !report.produced())
    {
        return Ok(None);
    }
    let clinit_identity = method_facts(clinit).1.clone();
    let Some((_, clinit_ast)) = method_asts
        .iter()
        .find(|(member, _)| *member == clinit_identity)
    else {
        return Ok(None);
    };
    let Some(line) =
        jarde_java::report::class_source_assert_switch_line(clinit_ast, field_name, budget)
            .map_err(InitializerProjectionFailure::Stopped)?
    else {
        return Ok(None);
    };
    // The switch reads the outermost enclosing class's own status: javac emits the outermost
    // class's literal in every nested class's `<clinit>` (verified for two nesting levels), and
    // any other receiver is a shape this proof refuses — a patched line would move the switch
    // when recompiled. The walk uses only this class's own `InnerClasses` rows, which state the
    // whole enclosing chain.
    let Some(outermost) = assert_outermost_class(this_class, resolved_inner_classes, budget)
        .map_err(InitializerProjectionFailure::Error)?
    else {
        return Ok(None);
    };
    let Some(outermost_source) = JvmBytes(outermost.to_vec()).to_unicode() else {
        return Ok(None);
    };
    if line.class_literal != outermost_source.replace('/', ".") {
        return Ok(None);
    }
    // The census: every member's own recovery named the field instructions it holds. A read the
    // fold did not consume — or a fold's read the records do not state — keeps the class
    // physical, and so does the `<clinit>` holding anything but the one line's write.
    let Some(internal) = JvmBytes(this_class.to_vec()).to_unicode() else {
        return Ok(None);
    };
    let mut folds: Vec<(
        &jarde_java::report::ClassSourceMethodAst,
        jarde_java::report::ClassSourceAssertMemberFold,
        usize,
    )> = Vec::new();
    for (method_index, method) in methods.iter().enumerate() {
        let Some(_) = method_facts(method).2 else {
            continue;
        };
        // The census reads the run's own Fieldref operations — the physical record of every field
        // instruction this body holds, independent of what the presentation layer claimed.
        let mut reads = Vec::new();
        let mut writes = Vec::new();
        for record in field_uses
            .iter()
            .filter(|record| record.member.as_ref() == Some(&*method_facts(method).1))
        {
            if record.owner != JvmBytes::from_unicode(&internal)
                || record.name != JvmBytes::from_unicode(field_name)
                || !record.is_static
            {
                continue;
            }
            if record.write {
                writes.push(record.bci);
            } else {
                reads.push(record.bci);
            }
        }
        if reads.is_empty() && writes.is_empty() {
            continue;
        }
        if method_index == clinit_index {
            if reads.is_empty() && writes.as_slice() == [line.write_bci] {
                continue;
            }
            return Ok(None);
        }
        let Some((_, ast)) = method_asts
            .iter()
            .find(|(member, _)| *member == *method_facts(method).1)
        else {
            return Ok(None);
        };
        let Some(fold) =
            jarde_java::report::class_source_assert_member_fold(ast, field_name, budget)
                .map_err(InitializerProjectionFailure::Stopped)?
        else {
            return Ok(None);
        };
        let mut consumed = fold.reads.clone();
        let mut stated = reads.clone();
        consumed.sort_unstable();
        stated.sort_unstable();
        if !writes.is_empty() || consumed != stated {
            return Ok(None);
        }
        folds.push((ast, fold, method_index));
    }
    if folds.is_empty() {
        return Ok(None);
    }
    // Stage every rewritten member and the stripped `<clinit>`, paid for before any record
    // changes; a refused stage keeps the class exactly as the recovery wrote it. The commit
    // replaces each member record's placed text — the channel the first-pass assembly and the
    // family writers read — and stages the same body through the projected-member channel, the
    // one a static member fold re-spells from instead of the physical recovery text.
    let mut staged: Vec<(usize, String, String)> = Vec::new();
    for (ast, fold, method_index) in &folds {
        let method = &methods[*method_index];
        let emitted = jarde_java::report::class_source_assert_member_current(ast, budget)
            .map_err(InitializerProjectionFailure::Stopped)?;
        let own_artifact = match method_facts(method).2 {
            Some(report) => report.text.clone(),
            None => return Ok(None),
        };
        let (declaration, current_text, markers, annotations, name, descriptor) =
            writer_facts(method);
        if !matches_current_text(
            declaration,
            current_text,
            &own_artifact,
            method_facts(method).2,
            markers,
            name,
            descriptor,
        ) || !lines_match(&own_artifact, &emitted)
        {
            // Another projection replaced this member's body, or this run's own statements
            // re-emit differently than the artifact it placed: either way the fold has no text
            // of this AST's own to stand in for.
            return Ok(None);
        }
        let folded =
            jarde_java::report::emit_class_source_assert_statements(ast, &fold.statements, budget)
                .map_err(InitializerProjectionFailure::Stopped)?;
        let Some(text) = assert_projection_text(
            declaration,
            method_facts(method).2,
            markers,
            annotations,
            &folded,
        ) else {
            return Ok(None);
        };
        budget
            .charge(
                CountedBudgetDimension::OutputBytes,
                u64::try_from(text.len()).unwrap_or(u64::MAX),
            )
            .map_err(InitializerProjectionFailure::Error)?;
        staged.push((*method_index, text, folded));
    }
    let emitted = jarde_java::report::class_source_assert_member_current(clinit_ast, budget)
        .map_err(InitializerProjectionFailure::Stopped)?;
    let clinit_artifact = match method_facts(&methods[clinit_index]).2 {
        Some(report) => report.text.clone(),
        None => return Ok(None),
    };
    let (declaration, current_text, markers, annotations, name, descriptor) =
        writer_facts(&methods[clinit_index]);
    if !matches_current_text(
        declaration,
        current_text,
        &clinit_artifact,
        method_facts(&methods[clinit_index]).2,
        markers,
        name,
        descriptor,
    ) || !lines_match(&clinit_artifact, &emitted)
    {
        return Ok(None);
    }
    let mut omit_clinit = false;
    if let Some(statements) =
        jarde_java::report::class_source_assert_clinit_without_line(clinit_ast, field_name, budget)
            .map_err(InitializerProjectionFailure::Stopped)?
    {
        let stripped_text = jarde_java::report::emit_class_source_assert_statements(
            clinit_ast,
            &statements,
            budget,
        )
        .map_err(InitializerProjectionFailure::Stopped)?;
        if stripped_text.trim().is_empty() {
            // Everything the `<clinit>` held was the switch line (its terminator is the return
            // the initializer context never writes): the member itself goes, exactly as the
            // enum-constant projection omits the physical initializer it replaced.
            omit_clinit = true;
        } else {
            let Some(text) = assert_projection_text(
                declaration,
                method_facts(&methods[clinit_index]).2,
                markers,
                annotations,
                &stripped_text,
            ) else {
                return Ok(None);
            };
            budget
                .charge(
                    CountedBudgetDimension::OutputBytes,
                    u64::try_from(text.len()).unwrap_or(u64::MAX),
                )
                .map_err(InitializerProjectionFailure::Error)?;
            staged.push((clinit_index, text, stripped_text));
        }
    }
    budget.poll().map_err(InitializerProjectionFailure::Error)?;
    Ok(Some((
        field_facts(switch_field).3,
        staged,
        omit_clinit.then_some(clinit_index),
    )))
}

// At most one enclosing edge may leave a class. A walk that cannot terminate within the number
// of metadata rows revisits a class; it cannot prove an outermost owner. Charge every examined
// row, including unsuccessful searches, and poll even when the table is empty.
fn assert_outermost_class<'a>(
    this_class: &'a [u8],
    rows: &'a [jarde_reader::classfile::ResolvedInnerClass],
    budget: &mut Budget,
) -> Result<Option<&'a [u8]>> {
    let mut current = this_class;
    for _ in 0..=rows.len() {
        budget.poll()?;
        let mut next = None;
        for row in rows {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if row.class == current
                && let Some(outer) = row.outer_class.as_deref()
            {
                if next.is_some() {
                    return Ok(None);
                }
                next = Some(outer);
            }
        }
        match next {
            Some(outer) => current = outer,
            None => return Ok(Some(current)),
        }
    }
    Ok(None)
}

#[cfg(test)]
mod assert_outermost_tests {
    use super::*;
    use jarde_reader::classfile::ResolvedInnerClass;
    fn row(class: &[u8], outer: &[u8]) -> ResolvedInnerClass {
        ResolvedInnerClass {
            class: class.to_vec(),
            outer_class: Some(outer.to_vec()),
            inner_name: None,
            access_flags: 0,
        }
    }
    fn budget() -> Budget {
        Budget::new(jarde_reader::budget::Limits {
            analysis_steps: 10000,
            elapsed_millis: 60000,
            ..Default::default()
        })
    }

    #[test]
    fn assert_outermost_proves_only_a_terminating_unambiguous_chain() {
        let cases = [
            (vec![], Some(b"A1$Sub".as_slice())),
            (vec![row(b"A1$Sub", b"A1")], Some(b"A1".as_slice())),
            (
                vec![row(b"A1$Sub", b"A1$Mid"), row(b"A1$Mid", b"A1")],
                Some(b"A1".as_slice()),
            ),
            (vec![row(b"A1$Sub", b"A1$Sub")], None),
            (
                vec![row(b"A1$Sub", b"A1$Mid"), row(b"A1$Mid", b"A1$Sub")],
                None,
            ),
            (vec![row(b"A1$Sub", b"A1"), row(b"A1$Sub", b"A1")], None),
            (vec![row(b"A1$Sub", b"A1"), row(b"A1$Sub", b"Other")], None),
        ];
        for (rows, expected) in cases {
            let mut budget = budget();
            assert_eq!(
                assert_outermost_class(b"A1$Sub", &rows, &mut budget).unwrap(),
                expected
            );
            assert!(budget.usage().analysis_steps <= (rows.len() * (rows.len() + 1)) as u64);
        }
    }

    #[test]
    fn assert_outermost_polls_and_bills_before_examining_metadata() {
        let rows = vec![row(b"A1$Sub", b"A1")];
        let mut limited = budget();
        let mut limits = limited.limits().clone();
        limits.analysis_steps = 0;
        limited = Budget::new(limits);
        assert!(matches!(
            assert_outermost_class(b"A1$Sub", &rows, &mut limited),
            Err(Error::BudgetExceeded {
                dimension: jarde_reader::budget::BudgetDimension::AnalysisSteps,
                ..
            })
        ));
        assert_eq!(limited.usage().analysis_steps, 0);
        let mut cancelled = budget();
        cancelled.cancellation_token().cancel();
        assert!(matches!(
            assert_outermost_class(b"A1$Sub", &[], &mut cancelled),
            Err(Error::Cancelled { .. })
        ));
    }
}

/// Compose an admitted integer-name body in its original physical method placement.
#[doc(hidden)]
pub fn integer_constant_projection_text(
    declaration: Option<&str>,
    recovery: Option<&crate::RecoveryReport>,
    markers: &[String],
    annotations: &crate::report::MemberAnnotationUses,
    body: &str,
) -> Option<String> {
    let declaration = declaration?;
    let report = recovery?;
    let original = artifact(&report.text)?;
    if !markers.is_empty() {
        return None;
    }
    let staged = Artifact {
        envelope: original.envelope,
        statements: body,
    };
    Some(prefix_method_annotations(
        block_member(declaration, Placed::Block(staged), markers),
        annotations,
    ))
}

/// Translate one body-only integer-name span through that exact same placement.
#[doc(hidden)]
pub fn integer_constant_projection_span(
    declaration: Option<&str>,
    recovery: Option<&crate::RecoveryReport>,
    markers: &[String],
    annotations: &crate::report::MemberAnnotationUses,
    body: &str,
    start: usize,
    end: usize,
) -> Option<(usize, usize)> {
    let declaration = declaration?;
    let report = recovery?;
    let original = artifact(&report.text)?;
    if !markers.is_empty() || start >= end {
        return None;
    }
    let source = body.get(start..end)?;
    if source.contains('\n') {
        return None;
    }
    let artifact_start = original.envelope.len().checked_add(2)?.checked_add(start)?;
    let artifact_end = original.envelope.len().checked_add(2)?.checked_add(end)?;
    let annotation_len = annotations
        .uses
        .iter()
        .map(|annotation| 4 + annotation.len() + 1)
        .sum::<usize>();
    let block_prefix = annotation_len + format!("    {declaration} {{\n").len();
    let staged = Artifact {
        envelope: original.envelope,
        statements: body,
    };
    let mapped = placed_artifact_span(&staged, artifact_start, artifact_end, block_prefix)?;
    let text =
        integer_constant_projection_text(Some(declaration), recovery, markers, annotations, body)?;
    (text.get(mapped.0..mapped.1)? == source).then_some(mapped)
}

/// Stage same-run integer-name method replacements without reading artifacts or publishing text.
#[doc(hidden)]
pub fn stage_integer_constant_method_group<'a, C: 'a, M: 'a, A: 'a, S: 'a, D, P>(
    execution: &ExecutionReport,
    constant_fields: &'a [C],
    methods: &'a [M],
    asts: &'a [A],
    prior_member_texts: &'a [S],
    constant_facts: impl Fn(&'a C) -> (i32, &'a str),
    method_facts: impl Fn(
        &'a M,
    ) -> (
        u64,
        &'a PhysicalMethodId,
        &'a [u8],
        Option<&'a crate::RecoveryReport>,
    ),
    writer_facts: impl Fn(
        &'a M,
    ) -> (
        Option<&'a str>,
        &'a str,
        &'a [String],
        &'a crate::report::MemberAnnotationUses,
    ),
    ast_facts: impl Fn(
        &'a A,
    ) -> (
        &'a PhysicalMethodId,
        &'a crate::report::ClassSourceMethodAst,
    ),
    staged_member_index: impl Fn(&'a S) -> u64,
    make_derived: impl Fn(&'a M, &'a C, usize, usize, u32) -> D,
    make_projected: impl Fn(&'a M, String, Vec<D>) -> P,
    budget: &mut Budget,
) -> Result<Option<Vec<P>>> {
    let project_text = |method: &'a M, body: &str| {
        let (declaration, _, markers, annotations) = writer_facts(method);
        integer_constant_projection_text(
            declaration,
            method_facts(method).3,
            markers,
            annotations,
            body,
        )
    };
    let project_span = |method: &'a M, body: &str, start: usize, end: usize| {
        let (declaration, _, markers, annotations) = writer_facts(method);
        integer_constant_projection_span(
            declaration,
            method_facts(method).3,
            markers,
            annotations,
            body,
            start,
            end,
        )
    };
    if constant_fields.is_empty()
        || !matches!(execution, ExecutionReport::Complete { .. })
        || methods.iter().any(|method| {
            method_facts(method)
                .3
                .is_some_and(|report| !matches!(report.execution, ExecutionReport::Complete { .. }))
        })
    {
        return Ok(None);
    }
    let candidates: Vec<_> = constant_fields
        .iter()
        .map(|candidate| jarde_java::report::IntegerConstantName {
            value: constant_facts(candidate).0,
            name: constant_facts(candidate).1.to_owned(),
        })
        .collect();
    let mut projected = Vec::new();
    for method in methods {
        let Some(recovery) = method_facts(method).3 else {
            continue;
        };
        if recovery.quality != jarde_jvm::ir::Quality::Structured
            || recovery.representation != jarde_jvm::ir::Representation::Java
            || recovery.content != crate::RecoveryContent::ContainsStatements
            || !recovery.fallbacks.is_empty()
        {
            continue;
        }
        let Some(ast) = asts
            .iter()
            .find(|entry| ast_facts(entry).0 == method_facts(method).1)
            .map(|entry| ast_facts(entry).1)
        else {
            continue;
        };
        let returns_int = method_facts(method).2.ends_with(b")I");
        let returns_int_array = method_facts(method).2.ends_with(b")[I");
        let Some((body, uses, original_body)) =
            jarde_java::report::project_class_source_integer_constants(
                ast,
                &candidates,
                returns_int,
                returns_int_array,
                budget,
            )
            .map_err(|stop| {
                enum_projection_stop_error(
                    stop,
                    "integer constant name projection",
                    "integer_constant_ir_missing",
                )
            })?
        else {
            continue;
        };
        let has_array_use = uses.iter().any(|use_site| use_site.body_range.is_some());
        if has_array_use {
            let mut prior_same_member = false;
            for staged in prior_member_texts {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                if staged_member_index(staged) == method_facts(method).0 {
                    prior_same_member = true;
                    break;
                }
            }
            if prior_same_member {
                continue;
            }
            budget.poll()?;
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(writer_facts(method).1.len()).unwrap_or(u64::MAX),
            )?;
            let original_body_matches = original_body
                .as_deref()
                .and_then(|original| project_text(method, original))
                .as_deref()
                == Some(writer_facts(method).1);
            if !original_body_matches {
                continue;
            }
        }
        let Some(text) = project_text(method, &body) else {
            continue;
        };
        let mut derived = Vec::new();
        let mut unique = true;
        for use_site in uses {
            let span = if let Some(range) = use_site.body_range.as_ref() {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::IrItems, 1)?;
                budget.charge(
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(range.end.saturating_sub(range.start)).unwrap_or(u64::MAX),
                )?;
                project_span(method, &body, range.start, range.end)
            } else {
                let (prefix, suffix) = if use_site.case_label {
                    ("case ", ":")
                } else {
                    ("return ", ";")
                };
                let needle = format!("{prefix}{}{suffix}", use_site.name);
                let mut matches = text.match_indices(&needle);
                let Some((at, _)) = matches.next() else {
                    unique = false;
                    break;
                };
                if matches.next().is_some() {
                    unique = false;
                    break;
                }
                let start = at + prefix.len();
                Some((start, start + use_site.name.len()))
            };
            let Some((start, end)) = span else {
                unique = false;
                break;
            };
            if text.get(start..end) != Some(use_site.name.as_str()) {
                unique = false;
                break;
            }
            let field = if use_site.body_range.is_some() {
                budget.charge(CountedBudgetDimension::IrItems, 1)?;
                let mut matching = None;
                for candidate in constant_fields {
                    budget.poll()?;
                    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                    if constant_facts(candidate).1 == use_site.name.as_str() {
                        matching = Some(candidate);
                        break;
                    }
                }
                matching
            } else {
                constant_fields
                    .iter()
                    .find(|candidate| constant_facts(candidate).1 == use_site.name.as_str())
            };
            let Some(field) = field else {
                unique = false;
                break;
            };
            if use_site.body_range.is_some() {
                budget.charge(CountedBudgetDimension::IrItems, 3)?;
            }
            derived.push(make_derived(method, field, start, end, use_site.bci));
        }
        if unique {
            if has_array_use {
                budget.charge(CountedBudgetDimension::IrItems, 1)?;
            }
            projected.push(make_projected(method, text, derived));
        }
    }
    Ok((!projected.is_empty()).then_some(projected))
}

/// Qualify a whole ordinary root replay, then verify every assembled derived range.
/// The closure supplies only the existing physical class assembly. Source eligibility lives here.
#[doc(hidden)]
pub fn assemble_method_projection_root<P, D>(
    original: String,
    current: &str,
    projected: &[P],
    derived_count: impl Fn(&P) -> usize,
    assemble: impl FnOnce() -> Option<(String, Vec<D>)>,
    derived_facts: impl Fn(&D) -> (usize, usize, bool),
) -> Option<(String, Vec<D>)> {
    if original != current {
        return None;
    }
    // The old comparison temporary died before assembly; do not retain both full root strings.
    drop(original);
    let (text, derived) = assemble()?;
    let expected = projected.iter().map(derived_count).sum::<usize>();
    if derived.len() != expected
        || derived.iter().any(|entry| {
            let (start, end, has_anchors) = derived_facts(entry);
            start >= end || text.get(start..end).is_none() || !has_anchors
        })
    {
        return None;
    }
    Some((text, derived))
}

/// Reserve and stage the whole integer-name publication before its supplier mutates any record.
#[doc(hidden)]
pub fn stage_integer_constant_publication<'a, M: 'a, P: 'a>(
    text: &str,
    methods: &'a [M],
    projected: &'a [P],
    method_index: impl Fn(&'a M) -> u64,
    projected_facts: impl Fn(&'a P) -> (u64, &'a str),
    budget: &mut Budget,
) -> Result<Option<Vec<Option<String>>>> {
    budget.charge(
        CountedBudgetDimension::OutputBytes,
        u64::try_from(text.len()).unwrap_or(u64::MAX),
    )?;
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(methods.len()).unwrap_or(u64::MAX),
    )?;
    let mut staged_method_texts = vec![None; methods.len()];
    for staged in projected {
        let mut matched_index = None;
        for (index, method) in methods.iter().enumerate() {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if method_index(method) == projected_facts(staged).0 {
                matched_index = Some(index);
                break;
            }
        }
        let Some(index) = matched_index else {
            return Ok(None);
        };
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        staged_method_texts[index] = Some(projected_facts(staged).1.to_owned());
    }
    budget.poll()?;
    Ok(Some(staged_method_texts))
}

#[doc(hidden)]
#[derive(Clone)]
pub struct ProvedEnumSwitchMap {
    pub labels: std::collections::BTreeMap<i64, String>,
    pub stores: Vec<crate::enumswitch::EnumSwitchMapEntry>,
    pub helper: PhysicalDefinitionId,
    pub enum_definition: PhysicalDefinitionId,
}

/// The map entries this class-source request proved from a selected helper's physical `<clinit>`.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceEnumSwitchEntry {
    pub key: i64,
    pub constant: Vec<u8>,
    pub constant_field_bci: u32,
    pub ordinal_bci: u32,
    pub table_read_bci: u32,
    pub store_bci: u32,
    pub handler_ordinal: u32,
    pub handler_bci: u32,
}

/// One same-run enum-switch candidate's class-level proof and projection outcome.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSourceEnumSwitchProof {
    pub member: jarde_reader::model::PhysicalMethodId,
    pub switch_bci: u32,
    pub read_bci: u32,
    pub table_owner: JvmBytes,
    pub table_name: JvmBytes,
    pub helper: Option<jarde_reader::model::PhysicalDefinitionId>,
    pub enum_definition: Option<jarde_reader::model::PhysicalDefinitionId>,
    pub entries: Vec<ClassSourceEnumSwitchEntry>,
    pub projected: bool,
    pub refusal: Option<String>,
}

#[doc(hidden)]
pub fn materialize_proved_enum_switch_map(
    proof: crate::enumswitch::EnumSwitchMapProof,
    helper: &PhysicalDefinitionId,
    enum_definition: &PhysicalDefinitionId,
    keys: &[i64],
) -> std::result::Result<ProvedEnumSwitchMap, String> {
    let mut labels = std::collections::BTreeMap::new();
    let mut stores = Vec::new();
    for entry in proof.entries {
        let Some(label) = JvmBytes(entry.constant.clone()).to_unicode() else {
            return Err("an enum constant name has no Unicode source label".to_owned());
        };
        if !crate::names::is_java_identifier(&label) || labels.insert(entry.key, label).is_some() {
            return Err("the proven enum map has an invalid or repeated key/label".to_owned());
        }
        stores.push(entry);
    }
    if keys.iter().any(|key| !labels.contains_key(key)) {
        return Err(
            "the proven initializer map does not cover every switch integer key".to_owned(),
        );
    }
    Ok(ProvedEnumSwitchMap {
        labels,
        stores,
        helper: helper.clone(),
        enum_definition: enum_definition.clone(),
    })
}

/// Compose one proved enum-switch body and all its site markers without mutating physical records.
#[doc(hidden)]
pub fn place_enum_switch_method(
    declaration: Option<&str>,
    existing_markers: &[String],
    annotations: &crate::report::MemberAnnotationUses,
    recovery_text: &str,
    markers_to_add: Vec<String>,
) -> Option<(String, Vec<String>)> {
    let declaration = declaration?;
    let artifact = artifact(recovery_text)?;
    let mut markers = existing_markers.to_vec();
    markers.extend(markers_to_add);
    let text = block_member(declaration, Placed::Block(artifact), &markers);
    let text = prefix_method_annotations(text, annotations);
    Some((text, markers))
}

/// Stage a complete class's enum-switch method groups from same-run candidates and physical map facts.
/// The supplier retains reads/IR and stop reporting; no report is bound or published here.
#[doc(hidden)]
pub fn stage_class_source_enum_switch_groups<
    'a,
    M: Clone + 'a,
    F: From<Error> + std::fmt::Display,
>(
    projection_complete: bool,
    methods: &'a [M],
    candidate_runs: &[crate::enumswitch::ClassSourceEnumSwitchCandidate],
    field_uses: &[crate::report::ClassSourceEnumSwitchFieldUse],
    proofs: &mut Vec<ClassSourceEnumSwitchProof>,
    execution: &mut ExecutionReport,
    mut prove_maps: impl FnMut(
        &[crate::enumswitch::ClassSourceEnumSwitchCandidate],
        &mut ExecutionReport,
        &mut Budget,
    ) -> std::result::Result<
        Vec<std::result::Result<ProvedEnumSwitchMap, String>>,
        F,
    >,
    mut on_stop: impl FnMut(F, &mut ExecutionReport, &Budget),
    method_facts: impl Fn(
        &'a M,
    ) -> (
        &'a PhysicalMethodId,
        Option<(&'a crate::RecoveryReport, &'a ExecutionReport)>,
    ),
    writer_facts: impl for<'m> Fn(
        &'m M,
    ) -> (
        Option<&'m str>,
        &'m [String],
        &'m crate::report::MemberAnnotationUses,
    ),
    set_method: impl Fn(&mut M, String, Vec<String>),
    budget: &mut Budget,
) -> Option<(Vec<M>, Vec<usize>)> {
    let mut staged_enum_projections: Vec<(usize, String, Vec<String>, Vec<usize>)> = Vec::new();
    let mut enum_projection_stopped = false;
    if projection_complete && !candidate_runs.is_empty() {
        let mut by_method: Vec<(
            PhysicalMethodId,
            Vec<crate::enumswitch::ClassSourceEnumSwitchCandidate>,
        )> = Vec::new();
        for candidate in candidate_runs.iter().cloned() {
            if let Some(member) = &candidate.member {
                if let Some((_, candidates)) =
                    by_method.iter_mut().find(|(known, _)| known == member)
                {
                    candidates.push(candidate);
                } else {
                    by_method.push((member.clone(), vec![candidate]));
                }
            }
        }
        for (member, candidates) in by_method {
            let Some(method_index) = methods
                .iter()
                .position(|method| method_facts(method).0 == &member)
            else {
                continue;
            };
            let method_complete = method_facts(&methods[method_index]).1.is_some_and(
                |(report, analysis_execution)| {
                    report.produced()
                        && report.quality == jarde_jvm::ir::Quality::Structured
                        && report.fallbacks.is_empty()
                        && matches!(&report.execution, ExecutionReport::Complete { .. })
                        && matches!(analysis_execution, ExecutionReport::Complete { .. })
                },
            );
            if !method_complete {
                for candidate in &candidates {
                    proofs.push(ClassSourceEnumSwitchProof {
                        member: member.clone(),
                        switch_bci: candidate.switch_bci,
                        read_bci: candidate.read_bci,
                        table_owner: candidate.table.owner.clone(),
                        table_name: candidate.table.name.clone(),
                        helper: None,
                        enum_definition: None,
                        entries: Vec::new(),
                        projected: false,
                        refusal: Some(
                            "the candidate method did not recover completely as structured Java"
                                .to_owned(),
                        ),
                    });
                }
                continue;
            }
            let extra_table_uses = candidates
                .iter()
                .map(|candidate| {
                    field_uses.iter().find(|use_site| {
                        use_site.owner == candidate.table.owner
                            && use_site.name == candidate.table.name
                            && use_site.descriptor == candidate.table.descriptor
                            && !candidates.iter().any(|owned| {
                                owned.member.as_ref() == Some(&member)
                                    && owned.table.owner == use_site.owner
                                    && owned.table.name == use_site.name
                                    && owned.table.descriptor == use_site.descriptor
                                    && owned.table.bci == use_site.bci
                                    && use_site.member.as_ref() == Some(&member)
                                    && use_site.is_static
                                    && !use_site.write
                            })
                    })
                })
                .collect::<Vec<_>>();
            let mut rejected = false;
            for (candidate, extra_use) in candidates.iter().zip(&extra_table_uses) {
                if let Some(use_site) = extra_use {
                    proofs.push(ClassSourceEnumSwitchProof {
                    member: member.clone(),
                    switch_bci: candidate.switch_bci,
                    read_bci: candidate.read_bci,
                    table_owner: candidate.table.owner.clone(),
                    table_name: candidate.table.name.clone(),
                    helper: None,
                    enum_definition: None,
                    entries: Vec::new(),
                    projected: false,
                    refusal: Some(format!(
                        "visible method writes or reads the selected table outside the proved candidate at BCI {}",
                        use_site.bci,
                    )),
                });
                    rejected = true;
                }
            }
            let mut method_staged: Vec<(
                &crate::enumswitch::ClassSourceEnumSwitchCandidate,
                std::collections::BTreeMap<i64, String>,
                String,
                usize,
            )> = Vec::new();
            let map_results = match prove_maps(&candidates, execution, budget) {
                Ok(results) => results,
                Err(error) => {
                    for candidate in &candidates {
                        if proofs.iter().any(|proof| {
                            proof.member == member && proof.switch_bci == candidate.switch_bci
                        }) {
                            continue;
                        }
                        proofs.push(ClassSourceEnumSwitchProof {
                            member: member.clone(),
                            switch_bci: candidate.switch_bci,
                            read_bci: candidate.read_bci,
                            table_owner: candidate.table.owner.clone(),
                            table_name: candidate.table.name.clone(),
                            helper: None,
                            enum_definition: None,
                            entries: Vec::new(),
                            projected: false,
                            refusal: Some(format!("cross-class proof stopped: {error}")),
                        });
                    }
                    on_stop(error, execution, budget);
                    enum_projection_stopped = true;
                    break;
                }
            };
            for (candidate, map_result) in candidates.iter().zip(map_results) {
                if proofs.iter().any(|proof| {
                    proof.member == member
                        && proof.switch_bci == candidate.switch_bci
                        && proof.refusal.is_some()
                }) {
                    continue;
                }
                let map = match map_result {
                    Ok(map) => map,
                    Err(reason) => {
                        proofs.push(ClassSourceEnumSwitchProof {
                            member: member.clone(),
                            switch_bci: candidate.switch_bci,
                            read_bci: candidate.read_bci,
                            table_owner: candidate.table.owner.clone(),
                            table_name: candidate.table.name.clone(),
                            helper: None,
                            enum_definition: None,
                            entries: Vec::new(),
                            projected: false,
                            refusal: Some(reason),
                        });
                        rejected = true;
                        continue;
                    }
                };
                let map_text = map
                    .stores
                    .iter()
                    .map(|entry| {
                        format!(
                            "{}={}@{}[field {}, ordinal {}, read {}, handler {}:{}]",
                            entry.key,
                            map.labels[&entry.key],
                            entry.store_bci,
                            entry.constant_field_bci,
                            entry.ordinal_bci,
                            entry.table_read_bci,
                            entry.handler_ordinal,
                            entry.handler_bci,
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let marker = format!(
                    "// jarde: enum switch projected at BCI {} from {:?}.{} via {:?}; initializer stores [{}]",
                    candidate.switch_bci,
                    map.helper,
                    crate::facts::display_jvm_bytes(&candidate.table.name),
                    map.enum_definition,
                    map_text,
                );
                let proof_index = proofs.len();
                proofs.push(ClassSourceEnumSwitchProof {
                    member: member.clone(),
                    switch_bci: candidate.switch_bci,
                    read_bci: candidate.read_bci,
                    table_owner: candidate.table.owner.clone(),
                    table_name: candidate.table.name.clone(),
                    helper: Some(map.helper.clone()),
                    enum_definition: Some(map.enum_definition.clone()),
                    entries: map
                        .stores
                        .iter()
                        .map(|entry| ClassSourceEnumSwitchEntry {
                            key: entry.key,
                            constant: entry.constant.clone(),
                            constant_field_bci: entry.constant_field_bci,
                            ordinal_bci: entry.ordinal_bci,
                            table_read_bci: entry.table_read_bci,
                            store_bci: entry.store_bci,
                            handler_ordinal: entry.handler_ordinal,
                            handler_bci: entry.handler_bci,
                        })
                        .collect(),
                    projected: false,
                    refusal: None,
                });
                method_staged.push((candidate, map.labels, marker, proof_index));
            }
            if rejected {
                for (_, _, _, proof_index) in method_staged {
                    proofs[proof_index].refusal = Some(
                    "group projection withheld because another site in this method was not proved"
                        .to_owned(),
                );
                }
                if enum_projection_stopped {
                    break;
                }
                continue;
            }
            let mut marker_stopped = false;
            for (candidate, _, marker, proof_index) in &method_staged {
                if let Err(error) = budget.charge(
                    CountedBudgetDimension::OutputBytes,
                    u64::try_from(marker.len()).unwrap_or(u64::MAX),
                ) {
                    proofs[*proof_index].refusal = Some(format!(
                        "projection marker output for BCI {} was stopped: {error}",
                        candidate.switch_bci,
                    ));
                    for (_, _, _, sibling_index) in &method_staged {
                        if *sibling_index != *proof_index {
                            proofs[*sibling_index].refusal = Some(
                            "group projection withheld because another site's marker exceeded the output budget"
                                .to_owned(),
                        );
                        }
                    }
                    on_stop(error.into(), execution, budget);
                    enum_projection_stopped = true;
                    marker_stopped = true;
                    break;
                }
            }
            if marker_stopped {
                break;
            }
            let emitter_candidates = method_staged
                .iter()
                .map(|(candidate, labels, _, _)| (*candidate, labels))
                .collect::<Vec<_>>();
            let recovery_text = match crate::report::emit_class_source_enum_switch_group(
                &emitter_candidates,
                budget,
            ) {
                Ok(Some(text)) => text,
                Ok(None) => {
                    for (candidate, _, _, proof_index) in &method_staged {
                        proofs[*proof_index].refusal = Some(format!(
                            "the same-run AST did not contain the matching enum switch shape at BCI {}",
                            candidate.switch_bci,
                        ));
                    }
                    continue;
                }
                Err(stop) => {
                    let error = enum_projection_stop_error(
                        stop,
                        "grouped enum switch projection",
                        "enum_switch_ir_missing",
                    );
                    for (candidate, _, _, proof_index) in &method_staged {
                        proofs[*proof_index].refusal = Some(format!(
                            "grouped projection emission stopped at BCI {}: {error}",
                            candidate.switch_bci,
                        ));
                    }
                    on_stop(error.into(), execution, budget);
                    enum_projection_stopped = true;
                    break;
                }
            };
            staged_enum_projections.push((
                method_index,
                recovery_text,
                method_staged
                    .iter()
                    .map(|(_, _, marker, _)| marker.clone())
                    .collect(),
                method_staged
                    .iter()
                    .map(|(_, _, _, proof_index)| *proof_index)
                    .collect(),
            ));
        }
    }
    if !enum_projection_stopped {
        let mut projected_methods = methods.to_vec();
        let mut projected_proof_indices = Vec::new();
        for (method_index, recovery_text, markers, proof_indices) in staged_enum_projections {
            let (declaration, existing_markers, annotations) =
                writer_facts(&projected_methods[method_index]);
            let Some((text, placed_markers)) = place_enum_switch_method(
                declaration,
                existing_markers,
                annotations,
                &recovery_text,
                markers,
            ) else {
                enum_projection_stopped = true;
                for proof_index in proof_indices {
                    proofs[proof_index].refusal = Some(
                        "the emitted method could not be atomically placed in its physical member"
                            .to_owned(),
                    );
                }
                break;
            };
            set_method(&mut projected_methods[method_index], text, placed_markers);
            projected_proof_indices.extend(proof_indices);
        }
        if !enum_projection_stopped {
            return Some((projected_methods, projected_proof_indices));
        }
    }
    if enum_projection_stopped {
        for proof in proofs.iter_mut() {
            if proof.refusal.is_none() && !proof.projected {
                proof.refusal =
                    Some("the class-source enum projection batch stopped before commit".to_owned());
            }
        }
    }
    None
}

#[doc(hidden)]
#[derive(Clone)]
pub struct PreparedEnumSwitchMap {
    pub helper: PhysicalDefinitionId,
    pub helper_table_names: Vec<Vec<u8>>,
    pub enum_definition: PhysicalDefinitionId,
    pub enum_owner: JvmBytes,
    pub constants: Vec<Vec<u8>>,
    pub table_name: JvmBytes,
    pub keys: Vec<i64>,
}

/// Prepare the complete enum-switch source input from selected same-run physical facts.
/// Reads and analyses stay with the request supplier, including its original execution merging.
#[doc(hidden)]
pub fn prepare_class_source_enum_switch<R, F: From<Error>>(
    dependency_scope: bool,
    target: &PhysicalDefinitionId,
    candidate: &crate::enumswitch::ClassSourceEnumSwitchCandidate,
    execution: &mut ExecutionReport,
    mut read: impl FnMut(
        Option<&PhysicalMethodId>,
        &[u8],
        &mut ExecutionReport,
        &mut Budget,
    ) -> std::result::Result<Option<(PhysicalDefinitionId, R)>, F>,
    facts: impl for<'r> Fn(&'r R) -> &'r ClassMemberFacts,
    mut analyze: impl FnMut(
        &PhysicalMethodId,
        &mut ExecutionReport,
        &mut Budget,
    ) -> std::result::Result<jarde_jvm::method_ir::MethodIrAnalysis, F>,
    budget: &mut Budget,
) -> std::result::Result<std::result::Result<PreparedEnumSwitchMap, String>, F> {
    if !dependency_scope {
        return Ok(Err(
            "the SingleClass environment does not provide the helper and enum dependencies"
                .to_owned(),
        ));
    }
    if candidate
        .member
        .as_ref()
        .is_none_or(|member| &member.owner != target)
    {
        return Ok(Err(
            "the candidate does not belong to the class-source physical definition".to_owned(),
        ));
    }
    let Some(receiver) = candidate.selector_receiver_type.as_deref() else {
        return Ok(Err(
            "the selector receiver has no exact verifier class type".to_owned(),
        ));
    };
    let Some(enum_owner) = receiver
        .strip_prefix(b"L")
        .and_then(|name| name.strip_suffix(b";"))
    else {
        return Ok(Err(
            "the selector receiver verifier type is not one exact object class".to_owned(),
        ));
    };
    let enum_owner = JvmBytes(enum_owner.to_vec());
    if enum_owner.0.is_empty()
        || candidate.index.name.0 != b"ordinal"
        || candidate.index.descriptor.0 != b"()I"
        || (candidate.index.owner != enum_owner && candidate.index.owner.0 != b"java/lang/Enum")
    {
        return Ok(Err(
            "the table index call is not the receiver enum's inherited ordinal()I".to_owned(),
        ));
    }

    let Some((helper_definition, helper_read)) = read(
        candidate.member.as_ref(),
        &candidate.table.owner.0,
        execution,
        budget,
    )?
    else {
        return Ok(Err(
            "the selected environment did not uniquely resolve the helper class".to_owned(),
        ));
    };
    let helper_facts = facts(&helper_read);
    let Some((enum_definition, enum_read)) =
        read(candidate.member.as_ref(), &enum_owner.0, execution, budget)?
    else {
        return Ok(Err(
            "the selected environment did not uniquely resolve the enum class".to_owned(),
        ));
    };
    let enum_facts = facts(&enum_read);
    if helper_facts.stopped_at.is_some()
        || helper_facts.this_class.raw().0.as_slice() != candidate.table.owner.0.as_slice()
        || helper_facts.access_flags & 0x1000 == 0
    {
        return Ok(Err(
            "the selected helper definition is incomplete, mismatched, or not synthetic".to_owned(),
        ));
    }
    if enum_facts.stopped_at.is_some()
        || enum_facts.this_class.raw() != &enum_owner
        || enum_facts.access_flags & 0x4000 == 0
        || enum_facts
            .super_class
            .as_ref()
            .map(|name| name.raw().0.as_slice())
            != Some(b"java/lang/Enum".as_slice())
    {
        return Ok(Err("the selected verifier receiver definition is not a complete ACC_ENUM class extending java/lang/Enum".to_owned()));
    }
    let helper_tables: Vec<_> = helper_facts
        .fields
        .iter()
        .filter(|field| {
            field.name.raw().0.as_slice() == candidate.table.name.0.as_slice()
                && field.descriptor.raw().0.as_slice() == b"[I"
        })
        .collect();
    if helper_tables.len() != 1
        || helper_tables[0].access_flags & (0x0008 | 0x0010 | 0x1000) != (0x0008 | 0x0010 | 0x1000)
    {
        return Ok(Err(
            "the helper does not declare one unique static final synthetic int[] table field"
                .to_owned(),
        ));
    }
    let values_descriptor = [b"()[L".as_slice(), enum_owner.0.as_slice(), b";"].concat();
    let values_methods: Vec<_> = enum_facts
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0.as_slice() == b"values"
                && method.descriptor.raw().0.as_slice() == values_descriptor.as_slice()
                && method.access_flags & (0x0001 | 0x0008) == (0x0001 | 0x0008)
        })
        .collect();
    let overrides_ordinal = enum_facts.methods.iter().any(|method| {
        method.name.raw().0.as_slice() == b"ordinal"
            && method.descriptor.raw().0.as_slice() == b"()I"
    });
    if values_methods.len() != 1 || overrides_ordinal {
        return Ok(Err(
            "the enum lacks one real static values() method or declares its own ordinal()I"
                .to_owned(),
        ));
    }
    let initializer_headers: Vec<_> = helper_facts
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0.as_slice() == b"<clinit>"
                && method.descriptor.raw().0.as_slice() == b"()V"
        })
        .collect();
    if initializer_headers.len() != 1 || helper_facts.methods.len() != 1 {
        return Ok(Err("the helper has missing/duplicate <clinit> or another method that could expose a table alias".to_owned()));
    }
    let constants: Vec<Vec<u8>> = enum_facts
        .fields
        .iter()
        .filter(|field| {
            field.access_flags & 0x4000 != 0
                && field.access_flags & 0x0008 != 0
                && field.access_flags & 0x0010 != 0
                && field.descriptor.raw().0.as_slice()
                    == [b"L".as_slice(), enum_owner.0.as_slice(), b";"]
                        .concat()
                        .as_slice()
        })
        .map(|field| field.name.raw().0.clone())
        .collect();
    if constants.is_empty()
        || constants
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != constants.len()
        || constants.iter().any(|constant| {
            JvmBytes(constant.clone())
                .to_unicode()
                .is_none_or(|name| !crate::names::is_java_identifier(&name))
        })
    {
        return Ok(Err(
            "the enum has no uniquely spellable ACC_ENUM constant fields".to_owned(),
        ));
    }

    let enum_clinit_headers: Vec<_> = enum_facts
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0.as_slice() == b"<clinit>"
                && method.descriptor.raw().0.as_slice() == b"()V"
        })
        .collect();
    let enum_constructor_headers: Vec<_> = enum_facts
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0.as_slice() == b"<init>"
                && method.descriptor.raw().0.as_slice() == b"(Ljava/lang/String;I)V"
        })
        .collect();
    let values_field_descriptor = [b"[L".as_slice(), enum_owner.0.as_slice(), b";"].concat();
    let values_fields: Vec<_> = enum_facts
        .fields
        .iter()
        .filter(|field| {
            field.access_flags & (0x0008 | 0x0010 | 0x1000) == (0x0008 | 0x0010 | 0x1000)
                && field.descriptor.raw().0.as_slice() == values_field_descriptor.as_slice()
        })
        .collect();
    if enum_clinit_headers.len() != 1
        || enum_constructor_headers.len() != 1
        || values_fields.len() != 1
    {
        return Ok(Err(
            "the enum has no unique <clinit> and (String,int) constructor for constant-identity proof".to_owned(),
        ));
    }
    let enum_clinit = PhysicalMethodId {
        owner: enum_definition.clone(),
        name: JvmBytes(b"<clinit>".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    };
    let enum_constructor = PhysicalMethodId {
        owner: enum_definition.clone(),
        name: JvmBytes(b"<init>".to_vec()),
        descriptor: JvmBytes(b"(Ljava/lang/String;I)V".to_vec()),
    };
    let values_field_name = values_fields[0].name.raw();
    let enum_clinit_analysis = analyze(&enum_clinit, execution, budget)?;

    if enum_clinit_analysis.report().method != enum_clinit
        || !matches!(
            enum_clinit_analysis.report().execution,
            ExecutionReport::Complete { .. }
        )
    {
        return Ok(Err(
            "the enum <clinit> IR did not complete for the selected physical definition".to_owned(),
        ));
    }
    let factory_name = match crate::enumswitch::enum_values_factory_name(
        enum_clinit_analysis.ir(),
        &enum_owner,
        values_field_name,
        budget,
    ) {
        Ok(Ok(name)) => name,
        Ok(Err(reason)) => return Ok(Err(reason)),
        Err(stop) => {
            return Err(enum_projection_stop_error(
                stop,
                "enum switch projection",
                "enum_switch_ir_missing",
            )
            .into());
        }
    };
    let factory_headers: Vec<_> = enum_facts
        .methods
        .iter()
        .filter(|method| {
            method.name.raw() == &factory_name
                && method.descriptor.raw().0.as_slice() == values_descriptor.as_slice()
                && method.access_flags & (0x0002 | 0x0008 | 0x1000) == (0x0002 | 0x0008 | 0x1000)
        })
        .collect();
    if factory_headers.len() != 1 || factory_name.0 == b"values" {
        return Ok(Err(
            "the enum <clinit> target is not one private static synthetic values-array factory"
                .to_owned(),
        ));
    }
    let enum_values_factory = PhysicalMethodId {
        owner: enum_definition.clone(),
        name: factory_name,
        descriptor: JvmBytes(values_descriptor.clone()),
    };
    let enum_constructor_analysis = analyze(&enum_constructor, execution, budget)?;

    if enum_constructor_analysis.report().method != enum_constructor
        || !matches!(
            enum_constructor_analysis.report().execution,
            ExecutionReport::Complete { .. }
        )
    {
        return Ok(Err(
            "the enum constructor IR did not complete for the selected physical definition"
                .to_owned(),
        ));
    }
    let enum_factory_analysis = analyze(&enum_values_factory, execution, budget)?;

    if enum_factory_analysis.report().method != enum_values_factory
        || !matches!(
            enum_factory_analysis.report().execution,
            ExecutionReport::Complete { .. }
        )
    {
        return Ok(Err(
            "the enum values-array factory IR did not complete for the selected physical definition".to_owned(),
        ));
    }
    let enum_values_method = PhysicalMethodId {
        owner: enum_definition.clone(),
        name: JvmBytes(b"values".to_vec()),
        descriptor: JvmBytes(values_descriptor.clone()),
    };
    let enum_values_analysis = analyze(&enum_values_method, execution, budget)?;

    if enum_values_analysis.report().method != enum_values_method
        || !matches!(
            enum_values_analysis.report().execution,
            ExecutionReport::Complete { .. }
        )
    {
        return Ok(Err(
            "the enum public values() IR did not complete for the selected physical definition"
                .to_owned(),
        ));
    }
    match crate::enumswitch::prove_enum_constant_ordinals(
        enum_clinit_analysis.ir(),
        enum_constructor_analysis.ir(),
        (enum_factory_analysis.ir(), enum_values_analysis.ir()),
        &enum_owner,
        &constants,
        values_field_name,
        budget,
    ) {
        Ok(Ok(())) => {}
        Ok(Err(reason)) => return Ok(Err(reason)),
        Err(stop) => {
            return Err((match stop {
                crate::StopReason::Budget {
                    dimension,
                    limit,
                    consumed,
                    requested,
                    ..
                } => Error::BudgetExceeded {
                    dimension,
                    limit,
                    consumed,
                    requested,
                },
                crate::StopReason::Cancelled { .. } => Error::Cancelled {
                    reason: "enum constant identity proof was cancelled".to_owned(),
                },
                crate::StopReason::Interrupted { code, .. } => {
                    Error::unsupported(code, "enum constant identity proof was interrupted")
                }
                crate::StopReason::IrTableMissing { table } => Error::unsupported(
                    "enum_switch_ir_missing",
                    format!("required IR table `{table}` is missing"),
                ),
                crate::StopReason::EvidenceRefused { code, message, .. } => {
                    Error::unsupported(code, message)
                }
            })
            .into());
        }
    }

    Ok(Ok(PreparedEnumSwitchMap {
        helper: helper_definition,
        helper_table_names: helper_facts
            .fields
            .iter()
            .filter(|field| {
                field.descriptor.raw().0.as_slice() == b"[I"
                    && field.access_flags & (0x0008 | 0x0010 | 0x1000) == (0x0008 | 0x0010 | 0x1000)
            })
            .map(|field| field.name.raw().0.clone())
            .collect(),
        enum_definition,
        enum_owner,
        constants,
        table_name: candidate.table.name.clone(),
        keys: candidate.keys.clone(),
    }))
}

/// Prove a prepared helper's complete selected table set and expand its per-site results.
/// The analysis callback only supplies the selected physical helper IR; no report is published.
#[doc(hidden)]
pub fn prove_prepared_enum_switch_maps<F: From<Error>>(
    candidates: &[crate::enumswitch::ClassSourceEnumSwitchCandidate],
    prepared: Vec<(usize, PreparedEnumSwitchMap)>,
    mut results: Vec<Option<std::result::Result<ProvedEnumSwitchMap, String>>>,
    mut analyze: impl FnMut(
        &PhysicalMethodId,
        &mut Budget,
    ) -> std::result::Result<jarde_jvm::method_ir::MethodIrAnalysis, F>,
    budget: &mut Budget,
) -> std::result::Result<Vec<Option<std::result::Result<ProvedEnumSwitchMap, String>>>, F> {
    let mut groups: Vec<(PhysicalDefinitionId, Vec<(usize, PreparedEnumSwitchMap)>)> = Vec::new();
    for (index, proof_input) in prepared {
        if let Some((_, members)) = groups
            .iter_mut()
            .find(|(helper, _)| *helper == proof_input.helper)
        {
            members.push((index, proof_input));
        } else {
            groups.push((proof_input.helper.clone(), vec![(index, proof_input)]));
        }
    }

    for (helper, members) in groups {
        if members.len() > 1 {
            let mut selected: Vec<_> = members
                .iter()
                .map(|(_, input)| input.table_name.0.clone())
                .collect();
            selected.sort();
            selected.dedup();
            let mut declared = members[0].1.helper_table_names.clone();
            declared.sort();
            declared.dedup();
            if selected != declared {
                let reason =
                    "the selected candidates do not form the helper's closed int[] table set"
                        .to_owned();
                for (index, _) in members {
                    results[index] = Some(Err(reason.clone()));
                }
                continue;
            }
        }
        let initializer = PhysicalMethodId {
            owner: helper.clone(),
            name: JvmBytes(b"<clinit>".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        };
        let analysis = analyze(&initializer, budget)?;
        if analysis.report().method != initializer
            || !matches!(
                analysis.report().execution,
                ExecutionReport::Complete { .. }
            )
        {
            let reason = "the helper <clinit> IR did not complete for the selected physical method"
                .to_owned();
            for (index, _) in members {
                results[index] = Some(Err(reason.clone()));
            }
            continue;
        }

        let mut table_groups: Vec<(PreparedEnumSwitchMap, Vec<usize>)> = Vec::new();
        let mut conflicting_table = false;
        for (index, input) in &members {
            if let Some((known, indices)) = table_groups
                .iter_mut()
                .find(|(known, _)| known.table_name == input.table_name)
            {
                if known.enum_owner != input.enum_owner
                    || known.enum_definition != input.enum_definition
                    || known.constants != input.constants
                {
                    conflicting_table = true;
                    break;
                }
                known.keys.extend(input.keys.iter().copied());
                known.keys.sort_unstable();
                known.keys.dedup();
                indices.push(*index);
            } else {
                table_groups.push((input.clone(), vec![*index]));
            }
        }
        if conflicting_table {
            let reason =
                "one selected helper table is associated with conflicting enum definitions"
                    .to_owned();
            for (index, _) in members {
                results[index] = Some(Err(reason.clone()));
            }
            continue;
        }

        if table_groups.len() == 1 {
            let (input, indices) = &table_groups[0];
            let proof = crate::enumswitch::prove_enum_switch_map_initializer(
                analysis.ir(),
                &candidates[indices[0]].table.owner.0,
                &input.table_name.0,
                &input.enum_owner,
                &input.constants,
                &input.keys,
                budget,
            );
            let result = match proof {
                Ok(Ok(proof)) if proof.initializer.as_ref() == Some(&initializer) => {
                    materialize_proved_enum_switch_map(
                        proof,
                        &input.helper,
                        &input.enum_definition,
                        &input.keys,
                    )
                }
                Ok(Ok(_)) => Err(
                    "initializer proof identity differs from the selected physical <clinit>"
                        .to_owned(),
                ),
                Ok(Err(reason)) => Err(reason),
                Err(stop) => {
                    return Err(enum_projection_stop_error(
                        stop,
                        "enum switch proof",
                        "enum_switch_ir_missing",
                    )
                    .into());
                }
            };
            for index in indices {
                results[*index] = Some(result.clone());
            }
            continue;
        }

        let requests = table_groups
            .iter()
            .map(|(input, _)| crate::enumswitch::EnumSwitchMapRequest {
                table_name: input.table_name.clone(),
                enum_owner: input.enum_owner.clone(),
                enum_constants: input.constants.clone(),
                switch_keys: input.keys.clone(),
            })
            .collect::<Vec<_>>();
        let proofs = crate::enumswitch::prove_enum_switch_map_initializer_group(
            analysis.ir(),
            &candidates[members[0].0].table.owner.0,
            &requests,
            budget,
        );
        match proofs {
            Ok(Ok(proofs)) if proofs.len() == table_groups.len() => {
                for ((input, indices), proof) in table_groups.into_iter().zip(proofs) {
                    let result = if proof.initializer.as_ref() == Some(&initializer) {
                        materialize_proved_enum_switch_map(
                            proof,
                            &input.helper,
                            &input.enum_definition,
                            &input.keys,
                        )
                    } else {
                        Err("initializer proof identity differs from the selected physical <clinit>".to_owned())
                    };
                    for index in indices {
                        results[index] = Some(result.clone());
                    }
                }
            }
            Ok(Ok(_)) => {
                let reason =
                    "the shared helper proof did not return one map for every selected table"
                        .to_owned();
                for (index, _) in &members {
                    results[*index] = Some(Err(reason.clone()));
                }
            }
            Ok(Err(reason)) => {
                let failed_table = reason
                    .strip_prefix("selected table `")
                    .and_then(|tail| tail.split_once('`'))
                    .map(|(name, _)| name);
                for (input, indices) in &table_groups {
                    let site_reason = if failed_table
                        == Some(crate::facts::display_jvm_bytes(&input.table_name).as_str())
                    {
                        reason.clone()
                    } else if let Some(failed_table) = failed_table {
                        format!(
                            "grouped map proof withheld because selected table `{failed_table}` was rejected: {reason}"
                        )
                    } else {
                        reason.clone()
                    };
                    for index in indices {
                        results[*index] = Some(Err(site_reason.clone()));
                    }
                }
            }
            Err(stop) => {
                return Err(enum_projection_stop_error(
                    stop,
                    "shared enum switch proof",
                    "enum_switch_ir_missing",
                )
                .into());
            }
        }
    }

    Ok(results)
}

/// Same-run member placement policy; caller retains physical identities and publication.
pub fn projected_body_method_text(
    declaration: Option<&str>,
    recovery: Option<&crate::RecoveryReport>,
    markers: &[String],
    name: &jarde_reader::model::JvmString,
    descriptor: &jarde_reader::model::JvmString,
    annotations: &crate::report::MemberAnnotationUses,
    body: &str,
    declaration_override: Option<&str>,
    extra_markers: &[String],
) -> Option<String> {
    let declaration = declaration_override.unwrap_or(declaration?);
    let artifact = artifact(body)?;
    // The complete derived body replaces only the physical run's explanation marker.
    // Its physical record remains unchanged; other projection markers are preserved.
    let mut all = if has_only_explanation_marker(recovery, markers, name, descriptor) {
        Vec::new()
    } else {
        markers.to_vec()
    };
    all.extend(extra_markers.iter().cloned());
    Some(prefix_method_annotations(
        block_member(declaration, Placed::Block(artifact), &all),
        annotations,
    ))
}

/// Same-run member placement policy; caller retains physical identities and publication.
pub fn projected_body_span(
    declaration: Option<&str>,
    recovery: Option<&crate::RecoveryReport>,
    markers: &[String],
    name: &jarde_reader::model::JvmString,
    descriptor: &jarde_reader::model::JvmString,
    annotations: &crate::report::MemberAnnotationUses,
    body: &str,
    declaration_override: Option<&str>,
    extra_markers: &[String],
    start: usize,
    end: usize,
) -> Option<(usize, usize)> {
    let declaration = declaration_override.unwrap_or(declaration?);
    let placed = artifact(body)?;
    let source = body.get(start..end)?;
    if source.is_empty() || source.contains('\n') {
        return None;
    }
    let annotation_len = annotations
        .uses
        .iter()
        .map(|annotation| 4 + annotation.len() + 1)
        .sum::<usize>();
    let own_markers = if has_only_explanation_marker(recovery, markers, name, descriptor) {
        &[][..]
    } else {
        markers
    };
    let marker_len = own_markers
        .iter()
        .chain(extra_markers)
        .map(|marker| indent(&format!("{marker}\n"), 2).len())
        .sum::<usize>();
    let block_prefix = annotation_len + format!("    {declaration} {{\n").len() + marker_len;
    let mapped = placed_artifact_span(&placed, start, end, block_prefix)?;
    let text = projected_body_method_text(
        Some(declaration),
        recovery,
        markers,
        name,
        descriptor,
        annotations,
        body,
        declaration_override,
        extra_markers,
    )?;
    (text.get(mapped.0..mapped.1)? == source).then_some(mapped)
}

/// Same-run member placement policy; caller retains physical identities and publication.
pub fn anonymous_projection_text(
    declaration: Option<&str>,
    recovered: bool,
    markers: &[String],
    annotations: (
        &crate::report::MemberAnnotationUses,
        &crate::report::ParameterAnnotationUses,
        &crate::report::TypeAnnotationUses,
    ),
    access_flags: u16,
    body: &str,
) -> Option<String> {
    anonymous_projection_text_at(
        declaration,
        recovered,
        markers,
        annotations,
        access_flags,
        body,
        3,
    )
}

/// Same-run member placement policy; caller retains physical identities and publication.
pub fn anonymous_projection_text_at(
    declaration: Option<&str>,
    recovered: bool,
    markers: &[String],
    annotations: (
        &crate::report::MemberAnnotationUses,
        &crate::report::ParameterAnnotationUses,
        &crate::report::TypeAnnotationUses,
    ),
    access_flags: u16,
    body: &str,
    declaration_indent: usize,
) -> Option<String> {
    let declaration = declaration?;
    // Java 8 anonymous bodies cannot declare static methods; bridge and synthetic methods
    // are physical implementation artifacts rather than source declarations.
    if access_flags & (ACC_STATIC | 0x0040 | 0x1000) != 0
        || !markers.is_empty()
        || !annotations.1.refusals.is_empty()
        || annotations
            .1
            .uses_by_position
            .iter()
            .any(|uses| !uses.is_empty())
        || !annotations.2.refusals.is_empty()
        || !annotations.2.field_uses.is_empty()
        || !annotations.2.return_uses.is_empty()
        || annotations
            .2
            .parameter_uses
            .iter()
            .any(|uses| !uses.is_empty())
        || !recovered
    {
        return None;
    }
    let mut text = String::new();
    let indentation = "    ".repeat(declaration_indent);
    for annotation in &annotations.0.uses {
        text.push_str(&indentation);
        text.push_str(annotation);
        text.push('\n');
    }
    text.push_str(&format!("{indentation}{declaration} {{\n"));
    text.push_str(body);
    text.push_str(&indentation);
    text.push_str("}\n");
    Some(text)
}

/// Same-run member placement policy; caller retains physical identities and publication.
pub fn anonymous_return_projection_text(
    declaration: Option<&str>,
    recovered: bool,
    markers: &[String],
    annotations: (
        &crate::report::MemberAnnotationUses,
        &crate::report::ParameterAnnotationUses,
        &crate::report::TypeAnnotationUses,
    ),
    body: &str,
) -> Option<String> {
    anonymous_return_projection_parts(declaration, recovered, markers, annotations, body, 1)
        .map(|(text, _)| text)
}

/// Same-run member placement policy; caller retains physical identities and publication.
pub fn anonymous_return_projection<P>(
    declaration: Option<&str>,
    recovered: bool,
    markers: &[String],
    annotations: (
        &crate::report::MemberAnnotationUses,
        &crate::report::ParameterAnnotationUses,
        &crate::report::TypeAnnotationUses,
    ),
    body: &crate::report::ClassSourceAnonymousReturn,
    make: impl FnOnce(String, usize, usize) -> P,
) -> Option<P> {
    let (text, body_start) = anonymous_return_projection_parts(
        declaration,
        recovered,
        markers,
        annotations,
        &body.text,
        1,
    )?;
    let start = body_start.checked_add(body.expression_range.start)?;
    let end = body_start.checked_add(body.expression_range.end)?;
    if start >= end || text.get(start..end)? != body.text.get(body.expression_range.clone())? {
        return None;
    }
    Some(make(text, start, end))
}

/// Same-run member placement policy; caller retains physical identities and publication.
pub fn projected_statements_method_text<P>(
    declaration: Option<&str>,
    recovered: bool,
    current_text: &str,
    body: &crate::report::ClassSourceAnonymousReturn,
    make: impl FnOnce(String, usize, usize) -> P,
) -> Option<P> {
    let declaration = declaration?;
    if !recovered {
        return None;
    }
    let declaration_line = format!("    {declaration} {{\n");
    if current_text.match_indices(&declaration_line).count() != 1 {
        return None;
    }
    let at = current_text.find(&declaration_line)?;
    let prefix = current_text.get(..at + declaration_line.len())?;
    let mut text = String::with_capacity(prefix.len() + body.text.len() + "    }\n".len());
    text.push_str(prefix);
    let body_start = text.len();
    text.push_str(&body.text);
    text.push_str("    }\n");
    let start = body_start.checked_add(body.expression_range.start)?;
    let end = body_start.checked_add(body.expression_range.end)?;
    if start >= end || text.get(start..end)? != body.text.get(body.expression_range.clone())? {
        return None;
    }
    Some(make(text, start, end))
}

/// Same-run member placement policy; caller retains physical identities and publication.
fn anonymous_return_projection_parts(
    declaration: Option<&str>,
    recovered: bool,
    markers: &[String],
    annotations: (
        &crate::report::MemberAnnotationUses,
        &crate::report::ParameterAnnotationUses,
        &crate::report::TypeAnnotationUses,
    ),
    body: &str,
    declaration_indent: usize,
) -> Option<(String, usize)> {
    let declaration = declaration?;
    if !markers.is_empty()
        || !annotations.1.refusals.is_empty()
        || annotations
            .1
            .uses_by_position
            .iter()
            .any(|uses| !uses.is_empty())
        || !annotations.2.refusals.is_empty()
        || !annotations.2.field_uses.is_empty()
        || !annotations.2.return_uses.is_empty()
        || annotations
            .2
            .parameter_uses
            .iter()
            .any(|uses| !uses.is_empty())
        || !recovered
    {
        return None;
    }
    let mut text = String::new();
    let indentation = "    ".repeat(declaration_indent);
    for annotation in &annotations.0.uses {
        text.push_str(&indentation);
        text.push_str(annotation);
        text.push('\n');
    }
    text.push_str(&format!("{indentation}{declaration} {{\n"));
    let body_start = text.len();
    text.push_str(body);
    text.push_str(&indentation);
    text.push_str("}\n");
    Some((text, body_start))
}

/// Physical evidence for capture only. A writer must separately prove every changed expression.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemberCaptureProof {
    pub field_index: u64,
    pub field_name: JvmBytes,
    pub constructor: PhysicalMethodId,
    pub write_bci: u32,
    pub reads: Vec<MemberCaptureRead>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemberCaptureRead {
    pub method: PhysicalMethodId,
    pub bci: u32,
    /// SSA consumers of the value loaded from the capture field, in this physical method.
    pub consumer_bcis: Vec<u32>,
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn prepare_anonymous_interface_site<'a>(
    sites: &'a [(
        PhysicalMethodId,
        u32,
        u32,
        JvmBytes,
        String,
        crate::report::AnonymousSiteShape,
    )],
    root_name: Option<&jarde_reader::model::JvmString>,
    root_major_version: Option<u16>,
    root_inner_classes: &[jarde_reader::classfile::ResolvedInnerClass],
    root_asts: &'a [(
        PhysicalMethodId,
        crate::report::ClassSourceMethodAst,
        Option<crate::report::GenericConstructorCandidate>,
        Option<crate::report::AnonymousAllocationScan>,
    )],
) -> Result<(
    &'a (
        PhysicalMethodId,
        u32,
        u32,
        JvmBytes,
        String,
        crate::report::AnonymousSiteShape,
    ),
    Vec<u8>,
    &'a crate::report::ClassSourceMethodAst,
    u16,
)> {
    let [(root_method, _, _, allocation_class, _, _)] = sites else {
        return Err(Error::unsupported(
            "anonymous_interface_site_not_unique",
            "anonymous interface projection needs exactly one direct-return allocation",
        ));
    };
    let root_name = root_name
        .ok_or_else(|| Error::unsupported("anonymous_root_unspelled", "root class is unspelled"))?
        .raw()
        .0
        .clone();
    let child_name = allocation_class.0.as_slice();
    let root_rows: Vec<_> = root_inner_classes
        .iter()
        .filter(|row| row.class == child_name)
        .collect();
    if root_rows.len() != 1
        || root_rows[0].outer_class.is_some()
        || root_rows[0].inner_name.is_some()
        || root_major_version.is_none_or(|version| version > 52)
    {
        return Err(Error::unsupported(
            "anonymous_typed_nesting_unproved",
            "the root class does not declare one matching anonymous InnerClasses row",
        ));
    }
    let Some((_, root_ast, _, _)) = root_asts
        .iter()
        .find(|(member, _, _, _)| member == root_method)
    else {
        return Err(Error::unsupported(
            "anonymous_root_ast_missing",
            "the selected allocation has no retained root method AST",
        ));
    };
    Ok((&sites[0], root_name, root_ast, root_rows[0].access_flags))
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn anonymous_interface_is_superclass(child_facts: &ClassMemberFacts) -> bool {
    child_facts
        .super_class
        .as_ref()
        .map(|name| name.raw().0.as_slice())
        != Some(b"java/lang/Object")
        && child_facts.interfaces.is_empty()
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn require_anonymous_interface_direct_return(
    site_shape: &crate::report::AnonymousSiteShape,
) -> Result<()> {
    if *site_shape != jarde_java::report::AnonymousSiteShape::DirectReturn {
        return Err(Error::unsupported(
            "anonymous_interface_site_shape_unsupported",
            "the interface projection keeps the direct-return site shape; a local-declaration initializer site belongs to the superclass projection only",
        ));
    }
    Ok(())
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn qualify_anonymous_interface_nesting(
    child_name: &[u8],
    root_name: &[u8],
    root_method: &PhysicalMethodId,
    root_row_flags: u16,
    child_major_version: Option<u16>,
    child_inner_classes: &[jarde_reader::classfile::ResolvedInnerClass],
    child_enclosing: Option<(&[u8], Option<&(JvmBytes, JvmBytes)>)>,
) -> Result<()> {
    let child_self_rows: Vec<_> = child_inner_classes
        .iter()
        .filter(|row| row.class == child_name)
        .collect();
    if child_self_rows.len() != 1
        || child_self_rows[0].outer_class.is_some()
        || child_self_rows[0].inner_name.is_some()
        || child_self_rows[0].access_flags != root_row_flags
        || child_major_version.is_none_or(|version| version > 52)
        || child_enclosing.is_none_or(|enclosing| {
            enclosing.0 != root_name
                || enclosing.1 != Some(&(root_method.name.clone(), root_method.descriptor.clone()))
        })
    {
        return Err(Error::unsupported(
            "anonymous_child_enclosing_method_unproved",
            "the child lacks the exact typed EnclosingMethod and anonymous self row",
        ));
    }
    Ok(())
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn anonymous_interface_capture_candidate(child_facts: &ClassMemberFacts) -> bool {
    child_facts.field_count == 1 && child_facts.fields.len() == 1
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn qualify_anonymous_interface_child(
    child_facts: &ClassMemberFacts,
    child_bytes: &[u8],
    root_name: &[u8],
    root_method: &PhysicalMethodId,
    root_bci: &u32,
    constructor_bci: &u32,
    allocation_class: &JvmBytes,
    root_ast: &crate::report::ClassSourceMethodAst,
    root_asts: &[(
        PhysicalMethodId,
        crate::report::ClassSourceMethodAst,
        Option<crate::report::GenericConstructorCandidate>,
        Option<crate::report::AnonymousAllocationScan>,
    )],
    capture_proof: Option<&MemberCaptureProof>,
    root_method_flags: impl FnOnce() -> Option<u16>,
) -> Result<(Option<Vec<u8>>, Vec<u8>, String, Option<(u16, String)>)> {
    let capture_descriptor = capture_proof
        .as_ref()
        .map(|_| child_facts.fields[0].descriptor.raw().0.clone());
    // The val$ capture kind is proved for the superclass projection; the interface body's
    // rewrite channels only know the double parameter and the enclosing instance, so any
    // other proved capture kind keeps the physical class rather than a mis-typed rewrite.
    if capture_descriptor.as_deref().is_some_and(|descriptor| {
        descriptor != b"D" && descriptor != [b"L".as_slice(), root_name, b";"].concat()
    }) {
        return Err(Error::unsupported(
            "anonymous_child_capture_kind_unsupported",
            "only the double and enclosing-instance capture kinds present here",
        ));
    }
    if child_facts.stopped_at.is_some()
        || child_bytes
            .get(6..8)
            .is_none_or(|version| u16::from_be_bytes([version[0], version[1]]) > 52)
        || (child_facts.field_count != 0 && capture_proof.is_none())
        || child_facts.fields.len() as u64 != child_facts.field_count
        || child_facts.methods.len() as u64 != child_facts.method_count
        || child_facts.methods.is_empty()
        || child_facts
            .methods
            .iter()
            .any(|method| method.name.raw().0 == b"<clinit>")
        || child_facts
            .super_class
            .as_ref()
            .map(|name| name.raw().0.as_slice())
            != Some(b"java/lang/Object")
        || child_facts.interfaces.len() != 1
        || child_facts.access_flags & 0x0200 != 0
    {
        return Err(Error::unsupported(
            "anonymous_child_shape_unproved",
            "the selected child is not an Object subclass of exactly one source-spellable interface",
        ));
    }
    let interface_name = child_facts.interfaces[0].raw().0.clone();
    let captured_root_parameter = if capture_descriptor.as_deref() == Some(b"D".as_slice()) {
        let expected_descriptor = [b"(D)L".as_slice(), interface_name.as_slice(), b";"].concat();
        let method_flags = root_method_flags();
        let returned = jarde_java::report::class_source_anonymous_return_site(root_ast);
        let allocation_argument_bcis = returned
            .as_ref()
            .map(|(_, _, arguments, _)| arguments.as_slice());
        let matching_scan = root_asts
            .iter()
            .find(|(method, _, _, _)| method == root_method)
            .and_then(|(_, _, _, scan)| scan.as_ref());
        let matching_sites: Vec<_> = matching_scan
            .into_iter()
            .flat_map(|scan| scan.allocations.iter())
            .filter(|site| {
                site.member == *root_method
                    && site.head_bci == *root_bci
                    && site.class == *allocation_class
                    && site.constructor_bci == Some(*constructor_bci)
            })
            .collect();
        let [site] = matching_sites.as_slice() else {
            return Err(Error::unsupported(
                "anonymous_capture_argument_unproved",
                "the double capture allocation has no unique same-run constructor argument proof",
            ));
        };
        let parameter_name = jarde_java::report::class_source_single_parameter_name(root_ast, 0);
        if root_method.descriptor.0 != expected_descriptor
            || method_flags.is_none_or(|flags| flags & 0x0008 == 0)
            || matching_scan.is_none_or(|scan| !scan.complete)
            || !site.verified
            || site.argument_bcis.len() != 1
            || site.argument_parameter_slots.as_slice() != [Some(0)]
            || allocation_argument_bcis != Some(site.argument_bcis.as_slice())
            || parameter_name.is_none()
        {
            return Err(Error::unsupported(
                "anonymous_capture_argument_unproved",
                "the unique constructor argument is not the unmodified root double parameter",
            ));
        }
        Some((
            0,
            parameter_name.expect("the source parameter name was checked"),
        ))
    } else {
        None
    };
    let expected_return = [b"()L".as_slice(), interface_name.as_slice(), b";"].concat();
    let return_matches = if captured_root_parameter.is_some() {
        root_method.descriptor.0 == [b"(D)L".as_slice(), interface_name.as_slice(), b";"].concat()
    } else {
        root_method.descriptor.0 == expected_return
    };
    let interface_source_name =
        jarde_java::names::source_class_name(&JvmBytes(interface_name.clone()));
    if !return_matches || interface_name.contains(&b'$') || interface_source_name.is_none() {
        return Err(Error::unsupported(
            "anonymous_interface_source_type_unproved",
            "the allocated child does not implement the exact source-level return interface",
        ));
    }
    Ok((
        capture_descriptor,
        interface_name,
        interface_source_name.expect("the interface source name was validated"),
        captured_root_parameter,
    ))
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn prove_anonymous_interface_contract<R>(
    child_facts: &ClassMemberFacts,
    root_name: &[u8],
    interface_name: &[u8],
    budget: &mut Budget,
    mut read_interface: impl FnMut(&[u8], &mut Budget) -> Result<Option<R>>,
    facts_of: impl for<'r> Fn(&'r R) -> &'r ClassMemberFacts,
    bytes_of: impl for<'r> Fn(&'r R) -> &'r [u8],
) -> Result<()> {
    let interface_contract_proved = if interface_name == b"java/lang/Runnable" {
        child_facts.methods.iter().any(|method| {
            method.name.raw().0 == b"run"
                && method.descriptor.raw().0 == b"()V"
                && method.access_flags & 0x0001 != 0
                && method.access_flags & (0x0008 | 0x0040 | 0x1000) == 0
        })
    } else {
        let Some(interface_read) = read_interface(interface_name, budget)? else {
            return Err(Error::unsupported(
                "anonymous_interface_unresolved",
                "the exact return interface does not resolve in the selected environment",
            ));
        };
        let source_accessible = facts_of(&interface_read).access_flags & 0x0200 != 0
            && facts_of(&interface_read).access_flags & crate::facts::ACC_ANNOTATION == 0
            && facts_of(&interface_read).stopped_at.is_none()
            && bytes_of(&interface_read)
                .get(6..8)
                .is_some_and(|version| u16::from_be_bytes([version[0], version[1]]) <= 52)
            && (facts_of(&interface_read).access_flags & 0x0001 != 0
                || internal_package(interface_name) == internal_package(root_name));
        source_accessible
            && facts_of(&interface_read).interfaces.is_empty()
            && facts_of(&interface_read).methods.len() == 1
            && facts_of(&interface_read).method_count == 1
            && facts_of(&interface_read).methods[0].access_flags == 0x0401
            && child_facts.methods.iter().any(|method| {
                method.name.raw().0 == facts_of(&interface_read).methods[0].name.raw().0
                    && method.descriptor.raw().0
                        == facts_of(&interface_read).methods[0].descriptor.raw().0
                    && method.access_flags & 0x0001 != 0
                    && method.access_flags & (0x0008 | 0x0040 | 0x1000) == 0
            })
    };
    if !interface_contract_proved {
        return Err(Error::unsupported(
            "anonymous_interface_contract_unproved",
            "the child does not provide the selected interface's single Java 8 abstract method",
        ));
    }
    Ok(())
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn qualify_anonymous_interface_constructor(
    child_facts: &ClassMemberFacts,
    capture_proof: Option<&MemberCaptureProof>,
) -> Result<()> {
    let constructors: Vec<_> = child_facts
        .methods
        .iter()
        .filter(|method| method.name.raw().0 == b"<init>")
        .collect();
    if constructors.len() != 1
        || (capture_proof.is_none() && constructors[0].descriptor.raw().0 != b"()V")
        || capture_proof.as_ref().is_some_and(|proof| {
            proof.constructor.descriptor.0 != constructors[0].descriptor.raw().0
        })
    {
        return Err(Error::unsupported(
            "anonymous_child_constructor_unproved",
            "the child does not have one zero-argument physical constructor",
        ));
    }
    Ok(())
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn qualify_anonymous_interface_methods<'a>(
    execution: &ExecutionReport,
    method_count: usize,
    child_facts: &ClassMemberFacts,
    child_asts: &[(
        PhysicalMethodId,
        crate::report::ClassSourceMethodAst,
        Option<crate::report::GenericConstructorCandidate>,
        Option<crate::report::AnonymousAllocationScan>,
    )],
    mut methods: impl Iterator<
        Item = (
            Option<&'a str>,
            Option<(&'a crate::RecoveryReport, &'a ExecutionReport)>,
        ),
    >,
) -> Result<()> {
    if !matches!(execution, ExecutionReport::Complete { .. })
        || method_count != child_facts.methods.len()
        || child_asts.len() != child_facts.methods.len()
        || child_asts
            .iter()
            .any(|(_, _, _, scan)| scan.as_ref().is_none_or(|scan| !scan.complete))
        || methods.any(|(declaration, recovered)| {
            !complete_anonymous_method(recovered) || declaration.is_none()
        })
    {
        return Err(Error::unsupported(
            "anonymous_child_methods_incomplete",
            "every child method must have a complete structured body and source declaration",
        ));
    }
    Ok(())
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn require_anonymous_interface_allocation_closed(
    child_asts: &[(
        PhysicalMethodId,
        crate::report::ClassSourceMethodAst,
        Option<crate::report::GenericConstructorCandidate>,
        Option<crate::report::AnonymousAllocationScan>,
    )],
) -> Result<()> {
    if child_asts.iter().any(|(_, _, _, scan)| {
        scan.as_ref()
            .is_none_or(|scan| !scan.allocations.is_empty())
    }) {
        return Err(Error::unsupported(
            "anonymous_child_additional_allocation",
            "the selected anonymous child has an allocation outside the one direct-return chain",
        ));
    }
    Ok(())
}

pub struct ProvedNestedAnonymousSite<'a> {
    pub method: &'a PhysicalMethodId,
    pub ast: &'a jarde_java::report::ClassSourceMethodAst,
    pub allocation_bci: u32,
    pub constructor_bci: u32,
    pub allocation_class: JvmBytes,
    pub allocation_type: String,
}

pub fn unique_anonymous_allocation<'a>(
    scans: &'a [(
        PhysicalMethodId,
        Option<jarde_java::report::AnonymousAllocationScan>,
    )],
    expected_methods: &[PhysicalMethodId],
    target: &str,
) -> Option<&'a jarde_java::report::AnonymousAllocationCandidate> {
    if scans.is_empty()
        || scans.len() != expected_methods.len()
        || scans
            .iter()
            .map(|(method, _)| method)
            .collect::<std::collections::HashSet<_>>()
            .len()
            != scans.len()
        || scans
            .iter()
            .map(|(method, _)| method)
            .collect::<std::collections::HashSet<_>>()
            != expected_methods
                .iter()
                .collect::<std::collections::HashSet<_>>()
        || expected_methods
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            != expected_methods.len()
        || scans
            .iter()
            .any(|(_, scan)| scan.as_ref().is_none_or(|scan| !scan.complete))
    {
        return None;
    }
    let mut matches = scans.iter().flat_map(|(_, scan)| {
        scan.as_ref()
            .into_iter()
            .flat_map(|scan| scan.allocations.iter())
            .filter(|allocation| allocation.class == JvmBytes::from_unicode(target))
    });
    let selected = matches.next()?;
    if matches.next().is_some() || !selected.verified {
        return None;
    }
    Some(selected)
}

pub fn proved_nested_anonymous_site<'a>(
    child_method_count: usize,
    child_asts: &'a [(
        PhysicalMethodId,
        jarde_java::report::ClassSourceMethodAst,
        Option<jarde_java::report::GenericConstructorCandidate>,
        Option<jarde_java::report::AnonymousAllocationScan>,
    )],
    facts: &ClassMemberFacts,
    definition: &PhysicalDefinitionId,
    budget: &mut Budget,
) -> Result<Option<ProvedNestedAnonymousSite<'a>>> {
    if facts.stopped_at.is_some()
        || child_method_count != facts.method_count as usize
        || child_asts.len() != facts.method_count as usize
    {
        return Ok(None);
    }
    let implementations: Vec<_> = facts
        .methods
        .iter()
        .enumerate()
        .filter(|(_, method)| method.name.raw().0 != b"<init>")
        .collect();
    let [(_, method)] = implementations.as_slice() else {
        return Ok(None);
    };
    if method.access_flags & 0x0001 == 0
        || method.access_flags & (0x0008 | 0x0040 | 0x1000) != 0
        || facts
            .methods
            .iter()
            .filter(|method| method.name.raw().0 == b"<init>")
            .count()
            != 1
    {
        return Ok(None);
    }
    let method_id = PhysicalMethodId {
        owner: definition.clone(),
        name: method.name.raw().clone(),
        descriptor: method.descriptor.raw().clone(),
    };
    let Some((retained_id, ast, _, _)) = child_asts
        .iter()
        .find(|(member, _, _, _)| member == &method_id)
    else {
        return Ok(None);
    };
    let Some((
        origin_bcis,
        target,
        argument_bcis,
        jarde_java::report::AnonymousSiteShape::DirectReturn,
    )) = jarde_java::report::class_source_anonymous_return_site(ast)
    else {
        return Ok(None);
    };
    let expected_methods: Vec<_> = facts
        .methods
        .iter()
        .filter(|method| {
            method
                .attributes
                .iter()
                .any(|shell| shell.name.raw().0 == b"Code")
        })
        .map(|method| PhysicalMethodId {
            owner: definition.clone(),
            name: method.name.raw().clone(),
            descriptor: method.descriptor.raw().clone(),
        })
        .collect();
    let selected_type = target.replace('.', "/");
    let scans: Vec<_> = child_asts
        .iter()
        .map(|(member, _, _, scan)| (member.clone(), scan.clone()))
        .collect();
    let Some(allocation) = unique_anonymous_allocation(&scans, &expected_methods, &selected_type)
    else {
        return Ok(None);
    };
    budget.poll()?;
    if retained_id != &method_id
        || allocation.member != method_id
        || !origin_bcis.contains(&allocation.head_bci)
        || !allocation
            .constructor_bci
            .is_some_and(|bci| origin_bcis.contains(&bci))
        || allocation.argument_bcis != argument_bcis
    {
        return Ok(None);
    }
    Ok(Some(ProvedNestedAnonymousSite {
        method: retained_id,
        ast,
        allocation_bci: allocation.head_bci,
        constructor_bci: allocation
            .constructor_bci
            .expect("verified anonymous allocation has a constructor BCI"),
        allocation_class: allocation.class.clone(),
        allocation_type: target,
    }))
}

pub fn complete_anonymous_method(
    recovered: Option<(&crate::RecoveryReport, &ExecutionReport)>,
) -> bool {
    matches!(
        recovered,
        Some((report, analysis_execution))
            if report.produced()
                && report.quality == jarde_jvm::ir::Quality::Structured
                && report.representation == jarde_jvm::ir::Representation::Java
                && report.syntax_status != jarde_jvm::ir::SyntaxStatus::NotJava
                && report.fallbacks.is_empty()
                && matches!(report.execution, ExecutionReport::Complete { .. })
                && matches!(analysis_execution, ExecutionReport::Complete { .. })
    )
}

/// Source applicability over selected same-run facts; physical IO and publication stay with the caller.
pub fn emit_anonymous_interface<M>(
    methods: &[M],
    identity_of: impl Fn(&M) -> &PhysicalMethodId,
    mut place_method: impl FnMut(&M, &str) -> Option<String>,
    child_asts: &[(
        PhysicalMethodId,
        crate::report::ClassSourceMethodAst,
        Option<crate::report::GenericConstructorCandidate>,
        Option<crate::report::AnonymousAllocationScan>,
    )],
    child_facts: &ClassMemberFacts,
    child_definition: &PhysicalDefinitionId,
    root_name: &[u8],
    root_ast: &crate::report::ClassSourceMethodAst,
    root_bci: &u32,
    allocation_type: &str,
    interface_source_name: &str,
    capture_proof: Option<&MemberCaptureProof>,
    capture_descriptor: Option<&[u8]>,
    captured_root_parameter: Option<&(u16, String)>,
    budget: &mut Budget,
) -> Result<crate::report::ClassSourceAnonymousReturn> {
    let constructor_identity = PhysicalMethodId {
        owner: child_definition.clone(),
        name: JvmBytes(b"<init>".to_vec()),
        descriptor: capture_proof.as_ref().map_or_else(
            || JvmBytes(b"()V".to_vec()),
            |proof| proof.constructor.descriptor.clone(),
        ),
    };
    let constructor_ast = child_asts
        .iter()
        .find(|(member, _, _, _)| member == &constructor_identity)
        .and_then(|(_, _, proof, _)| proof.as_ref());
    if capture_proof.is_none() && constructor_ast.is_none() {
        return Err(Error::unsupported(
            "anonymous_child_constructor_ast_missing",
            "the empty child constructor lacks its same-run generic-constructor proof",
        ));
    }
    if constructor_ast.is_some_and(|proof| !proof.parameters.is_empty()) && capture_proof.is_none()
    {
        return Err(Error::unsupported(
            "anonymous_child_constructor_not_empty",
            "the child constructor proof has source parameters",
        ));
    }
    let outer_source_name = if capture_proof
        .as_ref()
        .is_some_and(|proof| !proof.reads.is_empty())
        && capture_descriptor != Some(b"D".as_slice())
    {
        Some(
            jarde_java::names::source_class_name(&JvmBytes(root_name.to_vec())).ok_or_else(
                || Error::unsupported("anonymous_root_unspelled", "root class has no source name"),
            )?,
        )
    } else {
        None
    };
    let mut child_method_texts = Vec::new();
    for method in methods {
        if identity_of(method) == &constructor_identity {
            continue;
        }
        let captured_outer: Vec<_> = capture_proof
            .as_ref()
            .into_iter()
            .flat_map(|proof| proof.reads.iter())
            .filter(|read| &read.method == identity_of(method))
            .filter(|_| capture_descriptor != Some(b"D".as_slice()))
            .map(|read| jarde_java::report::CapturedOuterReadInput {
                method: identity_of(method).clone(),
                read_bci: read.bci,
                field_owner: child_facts.this_class.raw().clone(),
                field_name: capture_proof
                    .as_ref()
                    .expect("a captured read has a capture proof")
                    .field_name
                    .clone(),
                field_descriptor: JvmBytes([b"L".as_slice(), &root_name, b";"].concat()),
                outer_internal_name: JvmBytes(root_name.to_vec()),
                outer_source_name: outer_source_name
                    .clone()
                    .expect("a captured outer read needs its validated source name"),
                constructor: capture_proof
                    .as_ref()
                    .expect("a captured read has a capture proof")
                    .constructor
                    .clone(),
                constructor_write_bci: capture_proof
                    .as_ref()
                    .expect("a captured read has a capture proof")
                    .write_bci,
            })
            .collect();
        let captured_parameter: Vec<_> = capture_proof
            .as_ref()
            .into_iter()
            .flat_map(|proof| proof.reads.iter())
            .filter(|read| &read.method == identity_of(method))
            .filter(|_| capture_descriptor == Some(b"D".as_slice()))
            .map(|read| {
                let (parameter_slot, parameter_name) = captured_root_parameter
                    .as_ref()
                    .expect("the root parameter capture was proved");
                jarde_java::report::CapturedParameterReadInput {
                    method: identity_of(method).clone(),
                    read_bci: read.bci,
                    field_owner: child_facts.this_class.raw().clone(),
                    field_name: capture_proof
                        .as_ref()
                        .expect("a captured read has a capture proof")
                        .field_name
                        .clone(),
                    field_descriptor: JvmBytes::from_unicode("D"),
                    parameter_slot: *parameter_slot,
                    parameter_name: parameter_name.clone(),
                    parameter_presented: Some(JavaType::Double),
                    constructor: capture_proof
                        .as_ref()
                        .expect("a captured read has a capture proof")
                        .constructor
                        .clone(),
                    constructor_write_bci: capture_proof
                        .as_ref()
                        .expect("a captured read has a capture proof")
                        .write_bci,
                }
            })
            .collect();
        let ast = child_asts
            .iter()
            .find(|(member, _, _, _)| member == identity_of(method))
            .map(|(_, ast, _, _)| ast);
        let Some(ast) = ast else {
            return Err(Error::unsupported(
                "anonymous_child_method_ast_missing",
                "a presented child method has no same-run AST",
            ));
        };
        let projected_ast = if captured_outer.is_empty() && captured_parameter.is_empty() {
            ast.clone()
        } else if !captured_parameter.is_empty() {
            jarde_java::report::project_class_source_captured_parameter_reads(
                    ast,
                    &captured_parameter,
                    budget,
                )
                .map_err(|stop| {
                    enum_projection_stop_error(
                        stop,
                        "anonymous interface capture projection",
                        "anonymous_interface_ir_missing",
                    )
                })?
                .ok_or_else(|| Error::unsupported(
                    "anonymous_child_capture_ast_mismatch",
                    "each proved parameter capture read must map to exactly one matching field expression in the same-run AST",
                ))?
        } else {
            jarde_java::report::project_class_source_captured_outer_reads(
                    ast,
                    &captured_outer,
                    budget,
                )
                .map_err(|stop| {
                    enum_projection_stop_error(
                        stop,
                        "anonymous interface capture projection",
                        "anonymous_interface_ir_missing",
                    )
                })?
                .ok_or_else(|| Error::unsupported(
                    "anonymous_child_capture_ast_mismatch",
                    "each proved capture read must map to exactly one matching field expression in the same-run AST",
                ))?
        };
        let body = jarde_java::report::emit_class_source_method_ast(&projected_ast, 4, budget)
            .map_err(|stop| {
                enum_projection_stop_error(
                    stop,
                    "anonymous interface projection",
                    "anonymous_interface_ir_missing",
                )
            })?;
        let Some(text) = place_method(method, &body) else {
            return Err(Error::unsupported(
                "anonymous_child_method_unspellable",
                "a child method could not be projected from its physical declaration",
            ));
        };
        child_method_texts.push(text);
    }
    let methods = child_method_texts.concat();
    let hidden_outer_argument_bci = if capture_proof.is_some() {
        Some(
                if capture_descriptor == Some(b"D".as_slice()) {
                    jarde_java::report::class_source_anonymous_single_argument_bci(root_ast)
                } else {
                    jarde_java::report::class_source_anonymous_outer_argument_bci(root_ast)
                }
                    .ok_or_else(|| Error::unsupported(
                        "anonymous_outer_argument_unproved",
                        "the captured anonymous allocation does not pass exactly its proved value as the sole source argument",
                    ))?,
            )
    } else {
        None
    };
    let Some(body) = jarde_java::report::emit_class_source_anonymous_return(
        root_ast,
        *root_bci,
        allocation_type,
        interface_source_name,
        &methods,
        hidden_outer_argument_bci,
        budget,
    )
    .map_err(|stop| {
        enum_projection_stop_error(
            stop,
            "anonymous interface projection",
            "anonymous_interface_ir_missing",
        )
    })?
    else {
        return Err(Error::unsupported(
            "anonymous_return_emission_unmatched",
            "the root emitter could not match the exact allocation node",
        ));
    };
    Ok(body)
}
