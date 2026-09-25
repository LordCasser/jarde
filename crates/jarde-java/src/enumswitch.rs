//! The dispatch table a compiler's `switch` over an enum reads (P3 2.3, rule `enumswitch@1`).
//!
//! # The shape, exactly as far as the evidence goes
//!
//! For `switch (e) { case A: … }` a compiler does not branch on the enum constant; it builds a
//! synthetic `int[]` table indexed by the constant's ordinal and branches on the element:
//!
//! ```text
//! getstatic S.$SwitchMap$T [I ─ e.ordinal() ─ iaload ─ tableswitch { … }
//! ```
//!
//! What this rule verifies is the **read**: a `getstatic` of a static field whose descriptor is
//! `[I`, indexed by the result of an instance call whose descriptor is `()I` — both facts the decode
//! and the pool of this run state, and neither a guess about what the call is *called*. Its output
//! is that read, written where the value is consumed, which for this shape is the selector of the
//! `switch`.
//!
//! # What it deliberately does not claim, and why
//!
//! It does not write `switch (e)` with `case T.CONST:` labels, because the two facts that spelling
//! needs are not in this run:
//!
//! * **which enum constants the table's indices stand for** — the enum class's own fields, which are
//!   a class-level fact of *another* class;
//! * **that the entries are the constants' dense index** — the table's contents, written by the
//!   *synthetic* class's own static initializer, again a different class's body.
//!
//! Neither is the field's name: `$SwitchMap$…` is a compiler convention, and a convention is not
//! evidence — the same discipline that keeps `lambda$…` and `access$…` names from deciding their
//! shapes ([`crate::lambda`], [`crate::accessor`]). So what is presented is the table read the
//! bytecode really performs, which selects exactly what the source's `switch` selected, and the run
//! states in its record and its diagnostics that the constant mapping is not claimed rather than
//! inventing labels for it. A read that is *not* this shape is refused and quoted.

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{Definition, SsaTable, ValueId};
use serde::Serialize;

use crate::build::stack_operands;
use crate::decode::Operations;
use crate::evidence::Publication;
use crate::facts::{FieldAccess, InvokeKind, Operation};
use crate::pass::{ENUMSWITCH, Precondition, RuleVersion};
use crate::refusal::{Gap, Refusal};

/// The pass answerable for every verdict of this module.
pub(crate) const RULE: RuleVersion = ENUMSWITCH.rule();

/// The table read one body performs, the ones it does not, and the gaps it states in every
/// selection.
///
/// The plan holds the **decisions**: every array read the rule claimed (with the table and the call
/// it names, which the builder writes) and every candidate it refused, with the link that failed.
/// [`Self::materialize`] writes the owning [`EnumSwitchRecord`]s from them after the artifact is
/// committed, one record per charge and only when the request selected `RuleDetails`.
pub(crate) struct Plan {
    claimed: BTreeMap<u32, (TableRead, IndexCall)>,
    /// Why a candidate read was not presented, in BCI order: the internal decision every selection
    /// reports as a gap beside the records only a selected run materializes.
    refusals: Vec<(u32, Refusal)>,
}

/// The same-run facts a class-source adapter may use to seek proof of enum labels.
///
/// This is deliberately a candidate, not a mapping: the table's definition and `<clinit>` belong
/// to another class and are not established by `enumswitch@1`. The selector identity is an opaque
/// SSA value from this exact method run; it lets the class-source projection join the resolved
/// `ordinal()` receiver without reparsing emitted text.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSourceEnumSwitchCandidate {
    pub member: Option<jarde_reader::model::PhysicalMethodId>,
    pub table: TableRead,
    pub index: IndexCall,
    pub read_bci: u32,
    pub switch_bci: u32,
    pub selector_value: ValueId,
    pub selector_receiver: ValueId,
    /// The receiver's exact verifier reference type, when the frames name one.
    pub selector_receiver_type: Option<Vec<u8>>,
    pub keys: Vec<i64>,
    pub(crate) projection: Option<std::sync::Arc<EnumSwitchProjectionSource>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnumSwitchProjectionSource {
    pub(crate) program: crate::build::Program,
    pub(crate) facts: crate::facts::RecoveryFacts,
    pub(crate) declaration: Option<crate::declaration::Declaration>,
    pub(crate) member: Option<jarde_reader::model::PhysicalMethodId>,
}

/// One enum constant to integer relationship proven from a helper's actual `<clinit>` code.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumSwitchMapEntry {
    pub key: i64,
    pub constant: Vec<u8>,
    pub constant_field_bci: u32,
    pub ordinal_bci: u32,
    pub table_read_bci: u32,
    pub store_bci: u32,
    pub handler_ordinal: u32,
    pub handler_bci: u32,
}

/// Complete fixed-map proof for one helper initializer.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumSwitchMapProof {
    pub initializer: Option<jarde_reader::model::PhysicalMethodId>,
    pub allocation_bci: u32,
    pub table_write_bci: u32,
    pub entries: Vec<EnumSwitchMapEntry>,
}

/// The array factory named by the actual final call in the enum initializer. The caller resolves
/// this symbolic reference against the selected physical enum definition before reading its body.
#[doc(hidden)]
pub fn enum_values_factory_name(
    initializer: &jarde_jvm::method_ir::MethodIr,
    enum_owner: &str,
    values_field: &str,
    budget: &mut jarde_reader::budget::Budget,
) -> Result<Result<String, String>, crate::stop::StopReason> {
    use jarde_reader::budget::CountedBudgetDimension;

    let Some(code) = initializer.code() else {
        return Ok(Err(
            "the enum <clinit> has no complete Code facts".to_owned()
        ));
    };
    if code.stopped_at.is_some() || code.instructions.len() < 3 {
        return Ok(Err("the enum <clinit> is incomplete".to_owned()));
    }
    crate::stop::charge(
        budget,
        CountedBudgetDimension::IrItems,
        u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
        None,
    )?;
    let ops = Operations::of(code, initializer.constant_pool());
    let tail: Vec<_> = code.instructions[code.instructions.len() - 3..]
        .iter()
        .map(|instruction| {
            crate::stop::poll(budget, Some(instruction.bci))?;
            Ok(ops.get(instruction.bci))
        })
        .collect::<Result<_, crate::stop::StopReason>>()?;
    let array_descriptor = format!("[L{enum_owner};");
    let [
        Some(Operation::Invoke(call)),
        Some(Operation::Field {
            access: FieldAccess::Write,
            is_static: true,
            owner,
            name,
            descriptor,
        }),
        Some(Operation::Return),
    ] = tail.as_slice()
    else {
        return Ok(Err(
            "the enum <clinit> has no canonical values-array publication".to_owned(),
        ));
    };
    if call.kind() != InvokeKind::Static
        || call.owner() != enum_owner
        || call.descriptor() != format!("(){array_descriptor}")
        || call.is_interface_reference()
        || owner != enum_owner
        || name != values_field
        || descriptor != &array_descriptor
    {
        return Ok(Err(
            "the enum <clinit> does not call its own values-array factory".to_owned(),
        ));
    }
    Ok(Ok(call.name().to_owned()))
}

/// Proves that every declared enum constant is backed by a distinct object with a distinct
/// ordinal, and that the enum constructor forwards its name and ordinal to `java/lang/Enum`.
///
/// This is intentionally a small javac-shaped proof. It is not enough for a class to carry
/// `ACC_ENUM` or for its fields to be named constants: a legal class file can alias those fields
/// or rewrite constructor ordinals. Unknown shapes are refused so a source `case C` cannot silently
/// change the integer dispatch selected by a synthetic table.
#[doc(hidden)]
pub fn prove_enum_constant_ordinals(
    initializer: &jarde_jvm::method_ir::MethodIr,
    constructor: &jarde_jvm::method_ir::MethodIr,
    values_bodies: (
        &jarde_jvm::method_ir::MethodIr,
        &jarde_jvm::method_ir::MethodIr,
    ),
    enum_owner: &str,
    enum_constants: &[Vec<u8>],
    values_field: &str,
    budget: &mut jarde_reader::budget::Budget,
) -> Result<Result<(), String>, crate::stop::StopReason> {
    use crate::facts::{ConstantValue, FieldAccess};
    use jarde_reader::budget::CountedBudgetDimension;

    let (values_factory, values_method) = values_bodies;
    let Some(factory_declaration) = values_factory.declaration() else {
        return Ok(Err(
            "the enum values-array factory has no physical declaration".to_owned(),
        ));
    };
    let Some(values_declaration) = values_method.declaration() else {
        return Ok(Err(
            "the enum public values() has no physical declaration".to_owned()
        ));
    };
    let expected_values_descriptor = format!("()[L{enum_owner};");
    if factory_declaration.class_name().0 != enum_owner.as_bytes()
        || factory_declaration.descriptor().0 != expected_values_descriptor.as_bytes()
        || values_declaration.class_name().0 != enum_owner.as_bytes()
        || values_declaration.name().0 != b"values"
        || values_declaration.descriptor().0 != expected_values_descriptor.as_bytes()
    {
        return Ok(Err(
            "the enum values-array bodies have mismatched physical declarations".to_owned(),
        ));
    }

    let shape = |reason: &str| Ok(Err(reason.to_owned()));
    let Some(code) = initializer.code() else {
        return shape("the enum <clinit> has no complete Code facts");
    };
    if code.stopped_at.is_some()
        || code.instructions.is_empty()
        || !code.exception_handlers.is_empty()
    {
        return shape("the enum <clinit> is incomplete or has exception handlers");
    }
    let Some(cfg) = initializer.canonical() else {
        return shape("the enum <clinit> has no canonical CFG");
    };
    if !cfg.completeness().is_complete()
        || !cfg.unreachable().is_empty()
        || cfg.blocks().len() != 1
        || cfg.blocks().iter().any(|block| block.id().is_clone())
    {
        return shape("the enum <clinit> is not one complete straight-line control-flow path");
    }
    let Some(ssa) = initializer.ssa() else {
        return shape("the enum <clinit> has no SSA table");
    };
    let _ = ssa;
    crate::stop::charge(
        budget,
        CountedBudgetDimension::IrItems,
        u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
        None,
    )?;
    let operations = Operations::of(code, initializer.constant_pool());
    for instruction in &code.instructions {
        crate::stop::poll(budget, Some(instruction.bci))?;
        if operations.get(instruction.bci).is_none() {
            return shape("the enum <clinit> contains an undecoded instruction");
        }
    }

    let descriptor = format!("L{enum_owner};");
    let array_descriptor = format!("[L{enum_owner};");
    let actual: Vec<_> = code
        .instructions
        .iter()
        .filter_map(|instruction| operations.get(instruction.bci))
        .collect();
    let expected_len = enum_constants.len().saturating_mul(6).saturating_add(3);
    if actual.len() != expected_len {
        return shape(
            "the enum <clinit> contains instructions outside the complete constant and values-array whitelist",
        );
    }
    let mut ordinals = BTreeSet::new();
    for (index, constant) in enum_constants.iter().enumerate() {
        let Ok(name) = std::str::from_utf8(constant) else {
            return shape("an enum constant field name is not UTF-8");
        };
        let start = index * 6;
        let [
            Operation::Allocate { ty },
            Operation::Duplicate,
            Operation::Push(ConstantValue::String(constructor_name)),
            Operation::Push(ConstantValue::Int(ordinal)),
            Operation::Invoke(call),
            Operation::Field {
                access: FieldAccess::Write,
                is_static: true,
                owner,
                name: field_name,
                descriptor: field_descriptor,
            },
        ] = &actual[start..start + 6]
        else {
            return shape(
                "an enum constant is not initialized by its exact independent six-instruction javac sequence",
            );
        };
        if ty != enum_owner
            || constructor_name != name
            || call.kind() != crate::facts::InvokeKind::Special
            || call.owner() != enum_owner
            || call.name() != "<init>"
            || call.descriptor() != "(Ljava/lang/String;I)V"
            || *owner != enum_owner
            || field_name.as_bytes() != constant
            || field_descriptor != &descriptor
            || *ordinal != i64::try_from(index).unwrap_or(i64::MAX)
            || !ordinals.insert(*ordinal)
        {
            return shape(
                "an enum constant lacks a distinct object, matching name, unique runtime ordinal, constructor forward, or same-field assignment",
            );
        }
    }
    if !matches!(
        &actual[enum_constants.len() * 6..],
        [
            Operation::Invoke(call),
            Operation::Field {
                access: FieldAccess::Write,
                is_static: true,
                owner,
                name,
                descriptor: field_descriptor,
            },
            Operation::Return,
        ] if call.kind() == crate::facts::InvokeKind::Static
            && call.owner() == enum_owner
            && call.descriptor() == format!("()[L{enum_owner};")
            && call.name().as_bytes() == factory_declaration.name().0.as_slice()
            && *owner == enum_owner
            && name == values_field
            && field_descriptor == &array_descriptor
    ) {
        return shape(
            "the enum <clinit> does not end with one canonical values-array publication and return",
        );
    }

    let Some(ctor_code) = constructor.code() else {
        return shape("the enum constructor has no complete Code facts");
    };
    if ctor_code.stopped_at.is_some()
        || !ctor_code.exception_handlers.is_empty()
        || ctor_code.instructions.len() != 5
    {
        return shape(
            "the enum constructor is not the exact five-instruction ordinal-preserving form",
        );
    }
    let Some(ctor_cfg) = constructor.canonical() else {
        return shape("the enum constructor has no canonical CFG");
    };
    if !ctor_cfg.completeness().is_complete()
        || !ctor_cfg.unreachable().is_empty()
        || ctor_cfg.blocks().len() != 1
    {
        return shape("the enum constructor is not a complete straight-line body");
    }
    crate::stop::charge(
        budget,
        CountedBudgetDimension::IrItems,
        u64::try_from(ctor_code.instructions.len()).unwrap_or(u64::MAX),
        None,
    )?;
    let ctor_ops = Operations::of(ctor_code, constructor.constant_pool());
    let actual: Vec<_> = ctor_code
        .instructions
        .iter()
        .map(|instruction| {
            crate::stop::poll(budget, Some(instruction.bci))?;
            Ok(ctor_ops.get(instruction.bci).cloned())
        })
        .collect::<Result<_, crate::stop::StopReason>>()?;
    if !matches!(
        actual.as_slice(),
        [
            Some(Operation::Load { slot: 0 }),
            Some(Operation::Load { slot: 1 }),
            Some(Operation::Load { slot: 2 }),
            Some(Operation::Invoke(call)),
            Some(Operation::Return),
        ] if call.kind() == crate::facts::InvokeKind::Special
            && call.owner() == "java/lang/Enum"
            && call.name() == "<init>"
            && call.descriptor() == "(Ljava/lang/String;I)V"
    ) {
        return shape(
            "the enum constructor does not forward its exact name and ordinal to java/lang/Enum",
        );
    }

    let checked_operations =
        |ir: &jarde_jvm::method_ir::MethodIr,
         label: &str,
         budget: &mut jarde_reader::budget::Budget|
         -> Result<Result<Vec<Operation>, String>, crate::stop::StopReason> {
            let Some(code) = ir.code() else {
                return Ok(Err(format!("the enum {label} has no complete Code facts")));
            };
            let Some(cfg) = ir.canonical() else {
                return Ok(Err(format!("the enum {label} has no canonical CFG")));
            };
            if code.stopped_at.is_some()
                || code.instructions.is_empty()
                || !code.exception_handlers.is_empty()
                || !cfg.completeness().is_complete()
                || !cfg.unreachable().is_empty()
                || cfg.blocks().len() != 1
                || cfg.blocks().iter().any(|block| block.id().is_clone())
                || ir.ssa().is_none()
            {
                return Ok(Err(format!(
                    "the enum {label} is not one complete straight-line body"
                )));
            }
            crate::stop::charge(
                budget,
                CountedBudgetDimension::IrItems,
                u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
                None,
            )?;
            let ops = Operations::of(code, ir.constant_pool());
            let mut actual = Vec::with_capacity(code.instructions.len());
            for instruction in &code.instructions {
                crate::stop::poll(budget, Some(instruction.bci))?;
                let Some(op) = ops.get(instruction.bci) else {
                    return Ok(Err(format!(
                        "the enum {label} has an undecoded instruction"
                    )));
                };
                actual.push(op.clone());
            }
            Ok(Ok(actual))
        };
    let factory_ops = match checked_operations(values_factory, "values-array factory", budget)? {
        Ok(ops) => ops,
        Err(reason) => return Ok(Err(reason)),
    };
    if factory_ops.len() != enum_constants.len().saturating_mul(4).saturating_add(3)
        || !matches!(factory_ops.first(), Some(Operation::Push(ConstantValue::Int(length)))
            if *length == i64::try_from(enum_constants.len()).unwrap_or(i64::MAX))
        || !matches!(factory_ops.get(1), Some(Operation::NewArray {
            element: crate::ast::Type::Reference(element), dimensions: 1, total_dimensions: 1,
        }) if element == &enum_owner.replace('/', "."))
        || !matches!(factory_ops.last(), Some(Operation::Return))
    {
        return shape(
            "the enum values-array factory does not allocate and return one exact-length enum array",
        );
    }
    for (index, constant) in enum_constants.iter().enumerate() {
        let start = 2 + index * 4;
        if !matches!(&factory_ops[start..start + 4], [
            Operation::Duplicate,
            Operation::Push(ConstantValue::Int(ordinal)),
            Operation::Field {
                access: FieldAccess::Read, is_static: true, owner, name,
                descriptor: field_descriptor,
            },
            Operation::ArrayStore { element: None },
        ] if *ordinal == i64::try_from(index).unwrap_or(i64::MAX)
            && owner == enum_owner
            && name.as_bytes() == constant
            && field_descriptor == &descriptor)
        {
            return shape(
                "the enum values-array factory does not store each matching constant at its ordinal",
            );
        }
    }
    let values_ops = match checked_operations(values_method, "public values()", budget)? {
        Ok(ops) => ops,
        Err(reason) => return Ok(Err(reason)),
    };
    if !matches!(values_ops.as_slice(), [
        Operation::Field {
            access: FieldAccess::Read, is_static: true, owner, name,
            descriptor: field_descriptor,
        },
        Operation::Invoke(call),
        Operation::CheckCast { ty },
        Operation::Return,
    ] if owner == enum_owner
        && name == values_field
        && field_descriptor == &array_descriptor
        && call.kind() == InvokeKind::Virtual
        && call.owner() == array_descriptor
        && call.name() == "clone"
        && call.descriptor() == "()Ljava/lang/Object;"
        && !call.is_interface_reference()
        && ty == &array_descriptor)
    {
        return shape(
            "the enum public values() does not only clone and return the published array",
        );
    }
    Ok(Ok(()))
}

/// Proves the complete javac-style table initializer from this method's decoded code, CFG and SSA.
///
/// The accepted bytecode has one `values().length` allocation assigned once to the selected table,
/// followed by one independently `NoSuchFieldError`-guarded array write for each used switch key.
/// Every instruction and both continuations of every handler must be accounted for. The field and
/// call owners are supplied from the selected physical class facts; this function does not consult
/// helper names or enum declaration order to create a mapping.
#[doc(hidden)]
pub fn prove_enum_switch_map_initializer(
    ir: &jarde_jvm::method_ir::MethodIr,
    helper_owner: &str,
    table_name: &str,
    enum_owner: &str,
    enum_constants: &[Vec<u8>],
    switch_keys: &[i64],
    budget: &mut jarde_reader::budget::Budget,
) -> Result<Result<EnumSwitchMapProof, String>, crate::stop::StopReason> {
    let Some(code) = ir.code() else {
        return Ok(Err(
            "the helper `<clinit>` has no complete Code facts".to_owned()
        ));
    };
    if code.stopped_at.is_some()
        || code.instructions.is_empty()
        || code.instructions.len() > usize::from(u16::MAX)
    {
        return Ok(Err(
            "the helper `<clinit>` instruction stream is incomplete".to_owned(),
        ));
    }
    let Some(cfg) = ir.canonical() else {
        return Ok(Err("the helper `<clinit>` has no canonical CFG".to_owned()));
    };
    if !cfg.completeness().is_complete()
        || !cfg.unreachable().is_empty()
        || cfg.blocks().iter().any(|block| block.id().is_clone())
    {
        return Ok(Err(
            "the helper `<clinit>` has unreachable, cloned, or incomplete control flow".to_owned(),
        ));
    }
    let Some(ssa) = ir.ssa() else {
        return Ok(Err("the helper `<clinit>` has no SSA table".to_owned()));
    };
    let operations = Operations::of(code, ir.constant_pool());
    let by_bci: BTreeMap<u32, &jarde_reader::classfile::InstructionFact> = code
        .instructions
        .iter()
        .map(|instruction| (instruction.bci, instruction))
        .collect();
    let operation_at = |bci| operations.get(bci);
    let shape = |message: &str| Ok(Err(message.to_owned()));
    let table_field = |access: FieldAccess, owner: &str, name: &str, descriptor: &str| {
        access == FieldAccess::Read
            && owner == helper_owner
            && name == table_name
            && descriptor == "[I"
    };

    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
        None,
    )?;
    for instruction in &code.instructions {
        crate::stop::poll(budget, Some(instruction.bci))?;
        if operation_at(instruction.bci).is_none() {
            return shape("the helper `<clinit>` contains an undecoded instruction");
        }
    }

    let mut table_write = None;
    let mut table_reads = Vec::new();
    let mut enum_field_reads = Vec::new();
    let mut values_calls = Vec::new();
    let mut ordinal_calls = Vec::new();
    let mut arrays = Vec::new();
    let mut lengths = Vec::new();
    let mut stores = Vec::new();
    let mut transfers = Vec::new();
    let mut local_stores = Vec::new();
    let mut returns = Vec::new();
    let mut used_pushes = BTreeSet::new();
    let mut mapped_pushes = BTreeSet::new();

    for instruction in &code.instructions {
        let bci = instruction.bci;
        let Some(operation) = operation_at(bci) else {
            return shape("the helper `<clinit>` contains an undecoded instruction");
        };
        match operation {
            Operation::Field {
                access: FieldAccess::Write,
                is_static: true,
                owner,
                name,
                descriptor,
            } if owner == helper_owner && name == table_name && descriptor == "[I" => {
                if table_write.replace(bci).is_some() {
                    return shape("the helper writes the selected table more than once");
                }
            }
            Operation::Field {
                access: FieldAccess::Read,
                is_static: true,
                owner,
                name,
                descriptor,
            } if table_field(FieldAccess::Read, owner, name, descriptor) => {
                table_reads.push(bci);
            }
            Operation::Field {
                access: FieldAccess::Read,
                is_static: true,
                owner,
                name,
                descriptor,
            } if owner == enum_owner
                && descriptor.as_bytes() == enum_descriptor(enum_owner).as_slice()
                && enum_constants
                    .iter()
                    .any(|constant| constant.as_slice() == name.as_bytes()) =>
            {
                enum_field_reads.push((bci, name.as_bytes().to_vec()));
            }
            Operation::Field { .. } => {
                return shape("the helper `<clinit>` reads or writes an unrelated field");
            }
            Operation::Invoke(target)
                if target.kind() == crate::facts::InvokeKind::Static
                    && target.owner() == enum_owner
                    && target.name() == "values"
                    && target.descriptor() == format!("()[L{enum_owner};") =>
            {
                values_calls.push(bci);
            }
            Operation::Invoke(target)
                if target.kind() == crate::facts::InvokeKind::Virtual
                    && target.owner() == enum_owner
                    && target.name() == "ordinal"
                    && target.descriptor() == "()I" =>
            {
                ordinal_calls.push(bci);
            }
            Operation::Invoke(_) => {
                return shape("the helper `<clinit>` invokes an unrelated method");
            }
            Operation::NewArray {
                element: crate::ast::Type::Int,
                dimensions: 1,
                total_dimensions: 1,
            } => arrays.push(bci),
            Operation::NewArray { .. } => {
                return shape("the helper `<clinit>` allocates an array other than one int[]");
            }
            Operation::ArrayLength => lengths.push(bci),
            Operation::ArrayStore {
                element: Some(crate::ast::Type::Int),
            } => stores.push(bci),
            Operation::ArrayStore { .. } => {
                return shape("the helper `<clinit>` performs a non-int array write");
            }
            Operation::Transfer => transfers.push(bci),
            Operation::Store { .. } => local_stores.push(bci),
            Operation::Return => returns.push(bci),
            Operation::Push(crate::facts::ConstantValue::Int(_)) => {
                used_pushes.insert(bci);
            }
            Operation::Push(_) => {
                return shape("the helper `<clinit>` pushes a non-integer constant");
            }
            Operation::Other
            | Operation::Load { .. }
            | Operation::Arithmetic { .. }
            | Operation::Shift { .. }
            | Operation::Bitwise { .. }
            | Operation::Negate
            | Operation::PrimitiveConversion { .. }
            | Operation::Increment { .. }
            | Operation::Comparison { .. }
            | Operation::NumericComparison { .. }
            | Operation::Switch { .. }
            | Operation::Allocate { .. }
            | Operation::Duplicate
            | Operation::CheckCast { .. }
            | Operation::InstanceOf { .. }
            | Operation::InvokeDynamic(_)
            | Operation::ArrayLoad
            | Operation::ArrayElementLoad { .. }
            | Operation::Monitor { .. }
            | Operation::Throw => {
                return shape(
                    "the helper `<clinit>` contains an unproved operation or side effect",
                );
            }
        }
    }

    if table_reads.len() != stores.len()
        || enum_field_reads.len() != stores.len()
        || ordinal_calls.len() != stores.len()
        || transfers.len() != stores.len()
        || local_stores.len() != code.exception_handlers.len()
        || returns.len() != 1
        || code.exception_handlers.len() != stores.len()
        || values_calls.len() != 1
        || arrays.len() != 1
        || lengths.len() != 1
        || stores.is_empty()
        || table_write.is_none()
    {
        return shape("the helper `<clinit>` does not have one complete guarded write per mapping");
    }
    let table_write_bci = table_write.expect("checked exactly one selected table write");
    let Some((_, allocated_array)) = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == table_write_bci)
        .and_then(|instruction| stack_operands(instruction).last().copied())
    else {
        return shape("the selected table assignment has no SSA value");
    };
    let Some((allocation_bci, Operation::NewArray { .. })) =
        producer(ssa, allocated_array, &operations)
    else {
        return shape("the selected table is not assigned the proved int[] allocation");
    };
    let Some(allocation_instruction) = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == allocation_bci)
    else {
        return shape("the selected array allocation has no SSA instruction");
    };
    let Some((_, array_length)) = stack_operands(allocation_instruction).last().copied() else {
        return shape("the int[] allocation has no SSA length");
    };
    let Some((length_bci, Operation::ArrayLength)) = producer(ssa, array_length, &operations)
    else {
        return shape("the int[] length is not the enum values-array length");
    };
    let Some(length_instruction) = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == length_bci)
    else {
        return shape("the enum values-array length has no SSA instruction");
    };
    let Some((_, values_array)) = stack_operands(length_instruction).last().copied() else {
        return shape("the enum values-array length reads no array");
    };
    let Some((values_bci, Operation::Invoke(_))) = producer(ssa, values_array, &operations) else {
        return shape("the allocated length is not produced by enum values()");
    };
    if values_bci != values_calls[0]
        || length_bci != lengths[0]
        || allocation_bci != arrays[0]
        || !instruction_follows(
            by_bci.get(&values_bci).copied(),
            by_bci.get(&length_bci).copied(),
        )
        || !instruction_follows(
            by_bci.get(&length_bci).copied(),
            by_bci.get(&allocation_bci).copied(),
        )
        || !instruction_follows(
            by_bci.get(&allocation_bci).copied(),
            by_bci.get(&table_write_bci).copied(),
        )
    {
        return shape("the table is not uniquely initialized from enum values().length");
    }

    let mut entries = Vec::new();
    let mut seen_keys = BTreeSet::new();
    let mut seen_constants = BTreeSet::new();
    let mut accounted_bcis = BTreeSet::from([
        values_bci,
        length_bci,
        allocation_bci,
        table_write_bci,
        returns[0],
    ]);
    for store_bci in stores.iter().copied() {
        crate::stop::poll(budget, Some(store_bci))?;
        let Some(store_instruction) = ssa
            .blocks()
            .iter()
            .flat_map(|block| block.instructions())
            .find(|instruction| instruction.bci() == store_bci)
        else {
            return shape("an array write has no SSA instruction");
        };
        let operands = stack_operands(store_instruction);
        if operands.len() != 3 {
            return shape("an int[] mapping write does not read exactly array, index and key");
        }
        let Some((
            table_read_bci,
            Operation::Field {
                access,
                is_static: true,
                owner,
                name,
                descriptor,
            },
        )) = producer(ssa, operands[0].1, &operations)
        else {
            return shape("an int[] mapping write does not use a static table field read");
        };
        if !table_field(*access, owner, name, descriptor) || !table_reads.contains(&table_read_bci)
        {
            return shape("an int[] mapping write uses a different table field");
        }
        let Some((ordinal_bci, Operation::Invoke(target))) =
            producer(ssa, operands[1].1, &operations)
        else {
            return shape("a table index is not produced by enum ordinal()");
        };
        if target.kind() != crate::facts::InvokeKind::Virtual
            || target.owner() != enum_owner
            || target.name() != "ordinal"
            || target.descriptor() != "()I"
        {
            return shape("a table index is not the selected enum's instance ordinal() call");
        }
        let Some(ordinal_instruction) = ssa
            .blocks()
            .iter()
            .flat_map(|block| block.instructions())
            .find(|instruction| instruction.bci() == ordinal_bci)
        else {
            return shape("the ordinal call has no SSA instruction");
        };
        let Some((_, constant_receiver)) = stack_operands(ordinal_instruction).first().copied()
        else {
            return shape("the ordinal call has no enum receiver");
        };
        let Some((
            constant_bci,
            Operation::Field {
                access: FieldAccess::Read,
                is_static: true,
                owner,
                name: constant,
                descriptor,
            },
        )) = producer(ssa, constant_receiver, &operations)
        else {
            return shape("the ordinal receiver is not an enum constant field read");
        };
        if owner != enum_owner
            || descriptor.as_bytes() != enum_descriptor(enum_owner).as_slice()
            || !enum_constants
                .iter()
                .any(|candidate| candidate.as_slice() == constant.as_bytes())
            || !enum_field_reads.contains(&(constant_bci, constant.as_bytes().to_vec()))
        {
            return shape(
                "the ordinal receiver is not a declared ACC_ENUM field of the selected enum",
            );
        }
        let Some((key_bci, Operation::Push(crate::facts::ConstantValue::Int(key)))) =
            producer(ssa, operands[2].1, &operations)
        else {
            return shape("a table mapping value is not a proved integer constant");
        };
        if !used_pushes.contains(&key_bci)
            || !seen_keys.insert(i64::from(*key))
            || !seen_constants.insert(constant.as_bytes().to_vec())
        {
            return shape("the helper repeats a table key, enum constant, or key producer");
        }

        let Some(table_read_instruction) = by_bci.get(&table_read_bci).copied() else {
            return shape("a table read has no physical instruction fact");
        };
        let Some(constant_instruction) = by_bci.get(&constant_bci).copied() else {
            return shape("an enum constant read has no physical instruction fact");
        };
        let Some(ordinal_instruction_fact) = by_bci.get(&ordinal_bci).copied() else {
            return shape("an ordinal call has no physical instruction fact");
        };
        let Some(key_instruction) = by_bci.get(&key_bci).copied() else {
            return shape("a mapping key has no physical instruction fact");
        };
        let Some(store_instruction_fact) = by_bci.get(&store_bci).copied() else {
            return shape("an array write has no physical instruction fact");
        };
        if !instruction_follows(Some(table_read_instruction), Some(constant_instruction))
            || !instruction_follows(Some(constant_instruction), Some(ordinal_instruction_fact))
            || !instruction_follows(Some(ordinal_instruction_fact), Some(key_instruction))
            || !instruction_follows(Some(key_instruction), Some(store_instruction_fact))
        {
            return shape(
                "a mapping write's table, enum, ordinal, key and store are not one ordered path",
            );
        }
        let group_start = table_read_bci;
        let group_end = store_bci.saturating_add(store_instruction_fact.width);
        let Some(handler) = code
            .exception_handlers
            .iter()
            .find(|handler| handler.start_bci == group_start && handler.end_bci == group_end)
        else {
            return shape("a mapping write is not enclosed by its exact exception-table range");
        };
        if !catch_type_is(
            ir.constant_pool(),
            handler.catch_type_index,
            b"java/lang/NoSuchFieldError",
        ) {
            return shape("a mapping write's handler does not catch exactly NoSuchFieldError");
        }
        let Some(handler_instruction) = by_bci.get(&handler.handler_bci).copied() else {
            return shape("a NoSuchFieldError handler has no instruction");
        };
        if !matches!(
            operation_at(handler.handler_bci),
            Some(Operation::Store { .. })
        ) {
            return shape(
                "a mapping write's handler has unproved effects or no enclosing transfer",
            );
        }
        let transfer_bci = store_instruction_fact
            .bci
            .saturating_add(store_instruction_fact.width);
        let Some(transfer_fact) = by_bci.get(&transfer_bci).copied() else {
            return shape("a mapping write is not followed by its normal-path transfer");
        };
        if operation_at(transfer_bci) != Some(&Operation::Transfer)
            || !instruction_follows(Some(store_instruction_fact), Some(transfer_fact))
        {
            return shape("a mapping write is not followed by its normal-path transfer");
        }
        let handler_next = handler_instruction
            .bci
            .saturating_add(handler_instruction.width);
        let next_mapping_read = table_reads
            .iter()
            .copied()
            .filter(|read| *read > table_read_bci)
            .min()
            .unwrap_or(returns[0]);
        if handler_next != next_mapping_read {
            return shape("a NoSuchFieldError handler does not rejoin at the next mapping step");
        }
        let Some(source_block) = cfg.blocks().iter().find(|block| {
            block
                .blocks()
                .first()
                .is_some_and(|start| *start <= transfer_bci && transfer_bci < block.end_bci())
        }) else {
            return shape("a mapping transfer has no CFG block");
        };
        let Some(target_block) = cfg.blocks().iter().find(|block| {
            block.blocks().first().is_some_and(|start| {
                *start <= next_mapping_read && next_mapping_read < block.end_bci()
            })
        }) else {
            return shape("a mapping continuation has no CFG block");
        };
        let normal_successors: Vec<_> = cfg
            .edges()
            .iter()
            .filter(|edge| {
                edge.from() == source_block.id()
                    && edge.kind() == jarde_jvm::method_ir::CanonicalEdgeKind::Normal
            })
            .collect();
        if normal_successors.len() != 1 || normal_successors[0].to() != target_block.id() {
            return shape(
                "the mapping goto does not have one unique normal successor at the next mapping step",
            );
        }

        accounted_bcis.extend([
            table_read_bci,
            constant_bci,
            ordinal_bci,
            key_bci,
            store_bci,
            transfer_bci,
            handler.handler_bci,
        ]);
        mapped_pushes.insert(key_bci);
        entries.push(EnumSwitchMapEntry {
            key: i64::from(*key),
            constant: constant.as_bytes().to_vec(),
            constant_field_bci: constant_bci,
            ordinal_bci,
            table_read_bci,
            store_bci,
            handler_ordinal: handler.ordinal,
            handler_bci: handler.handler_bci,
        });
    }
    entries.sort_by_key(|entry| entry.store_bci);
    let mut expected_keys: Vec<i64> = switch_keys.to_vec();
    expected_keys.sort_unstable();
    expected_keys.dedup();
    let mut mapped_keys: Vec<i64> = entries.iter().map(|entry| entry.key).collect();
    mapped_keys.sort_unstable();
    if expected_keys
        .iter()
        .any(|expected| mapped_keys.binary_search(expected).is_err())
    {
        return shape("the helper does not map every used non-default switch key");
    }
    if used_pushes != mapped_pushes {
        return shape("the helper has an unconsumed integer constant");
    }
    if accounted_bcis.len() != code.instructions.len()
        || code
            .instructions
            .iter()
            .any(|instruction| !accounted_bcis.contains(&instruction.bci))
    {
        return shape("the helper `<clinit>` has an instruction outside the proved map paths");
    }
    let expected_handler_ordinals: BTreeSet<u32> =
        entries.iter().map(|entry| entry.handler_ordinal).collect();
    if expected_handler_ordinals.len() != code.exception_handlers.len()
        || code
            .exception_handlers
            .iter()
            .any(|handler| !expected_handler_ordinals.contains(&handler.ordinal))
    {
        return shape("the helper has an extra, shared, or unmatched exception handler");
    }

    crate::stop::charge(
        budget,
        jarde_reader::budget::CountedBudgetDimension::IrItems,
        u64::try_from(entries.len())
            .unwrap_or(u64::MAX)
            .saturating_mul(8),
        Some(table_write_bci),
    )?;
    Ok(Ok(EnumSwitchMapProof {
        initializer: ir
            .declaration()
            .map(|declaration| declaration.identity().clone()),
        allocation_bci,
        table_write_bci,
        entries,
    }))
}

fn enum_descriptor(owner: &str) -> Vec<u8> {
    format!("L{owner};").into_bytes()
}

fn instruction_follows(
    first: Option<&jarde_reader::classfile::InstructionFact>,
    second: Option<&jarde_reader::classfile::InstructionFact>,
) -> bool {
    first
        .zip(second)
        .is_some_and(|(first, second)| first.bci.checked_add(first.width) == Some(second.bci))
}

fn catch_type_is(
    constant_pool: &[jarde_reader::classfile::CpEntryFacts],
    catch_type_index: Option<u16>,
    expected: &[u8],
) -> bool {
    let Some(index) = catch_type_index else {
        return false;
    };
    constant_pool.iter().any(|entry| {
        entry.index == index
            && matches!(
                &entry.kind,
                jarde_reader::classfile::CpEntryKind::Class { name, .. }
                    if name.0.as_slice() == expected
            )
    })
}

/// One verdict of this rule's plan: the read it claimed, or the candidate it refused.
enum Decision<'a> {
    Claimed(u32, &'a (TableRead, IndexCall)),
    Refused(u32, &'a Refusal),
}

impl Decision<'_> {
    fn at(&self) -> u32 {
        match self {
            Self::Claimed(at, _) => *at,
            Self::Refused(at, _) => *at,
        }
    }

    /// The owning record, built here and only here.
    fn record(self) -> EnumSwitchRecord {
        crate::demand_counts::record_built(crate::evidence::RecoveryEvidenceKind::RuleDetails);
        match self {
            Self::Claimed(read, (table, index)) => EnumSwitchRecord {
                read,
                table: Some(table.clone()),
                index: Some(index.clone()),
                presented: true,
                refusal: None,
            },
            Self::Refused(read, refusal) => EnumSwitchRecord {
                read,
                table: None,
                index: None,
                presented: false,
                refusal: Some(EnumSwitchRefusal::of(refusal, read)),
            },
        }
    }
}

impl Plan {
    /// An empty plan: a body that reads no `int[]` element at all.
    pub(crate) fn empty() -> Self {
        Self {
            claimed: BTreeMap::new(),
            refusals: Vec::new(),
        }
    }

    /// Whether the instruction at one BCI is an array read this rule claimed.
    ///
    /// This is the question [`crate::build`] asks before it treats the read as a reader of another
    /// instruction's value: a claimed read writes the values it reads into its own text, and one no
    /// rule claimed is quoted and writes nothing.
    pub(crate) fn owns(&self, bci: u32) -> bool {
        self.claimed.contains_key(&bci)
    }

    /// The table and the call one claimed read names, or `None` when this rule refused it.
    pub(crate) fn claim(&self, bci: u32) -> Option<(&TableRead, &IndexCall)> {
        self.claimed.get(&bci).map(|(table, index)| (table, index))
    }

    /// Every candidate read the rule refused, in BCI order.
    pub(crate) fn refusals(&self) -> impl Iterator<Item = Gap> + '_ {
        self.refusals.iter().map(|(at, refusal)| {
            let refusal = EnumSwitchRefusal::of(refusal, *at);
            Gap::at(refusal.code, *at, refusal.message)
        })
    }

    /// The owning records this plan publishes under `publication`, in BCI order, within the phase's
    /// remaining allowance.
    pub(crate) fn materialize(
        &self,
        publication: Publication,
        phase: &mut crate::evidence::EvidencePhase,
        budget: &mut jarde_reader::budget::Budget,
    ) -> (Vec<EnumSwitchRecord>, crate::evidence::Materialized) {
        let mut decisions: Vec<Decision<'_>> = self
            .claimed
            .iter()
            .map(|(at, claim)| Decision::Claimed(*at, claim))
            .chain(
                self.refusals
                    .iter()
                    .map(|(at, refusal)| Decision::Refused(*at, refusal)),
            )
            .collect();
        decisions.sort_by_key(Decision::at);
        phase.materialize(
            budget,
            decisions
                .into_iter()
                .filter(|decision| publication.publishes(&[decision.at()])),
            Decision::record,
        )
    }

    /// Whether this rule decided anything about this body.
    pub(crate) fn answered(&self) -> bool {
        !self.claimed.is_empty() || !self.refusals.is_empty()
    }

    /// How many candidate reads the rule read, and how many of them it presented.
    pub(crate) fn counts(&self) -> (u64, u64) {
        let presented = u64::try_from(self.claimed.len()).unwrap_or(u64::MAX);
        let refused = u64::try_from(self.refusals.len()).unwrap_or(u64::MAX);
        (presented + refused, presented)
    }

    /// Captures enum-switch read candidates consumed by a real integer switch in this same SSA
    /// run. It does not resolve the call or interpret the table name; the class-source layer must
    /// prove both across the selected environment.
    pub(crate) fn class_source_candidates(
        &self,
        ssa: &SsaTable,
        operations: &Operations,
        member: Option<jarde_reader::model::PhysicalMethodId>,
        budget: &mut jarde_reader::budget::Budget,
    ) -> Result<Vec<ClassSourceEnumSwitchCandidate>, crate::stop::StopReason> {
        let mut candidates = Vec::new();
        for switch in ssa
            .blocks()
            .iter()
            .flat_map(|block| block.instructions())
            .filter(|instruction| {
                operations
                    .get(instruction.bci())
                    .is_some_and(|op| op.switch().is_some())
            })
        {
            let switch_bci = switch.bci();
            crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::IrItems,
                1,
                Some(switch_bci),
            )?;
            crate::stop::poll(budget, Some(switch_bci))?;
            let Some((_, selector_value)) = stack_operands(switch).last().copied() else {
                continue;
            };
            let Some((read_bci, Operation::ArrayLoad)) = producer(ssa, selector_value, operations)
            else {
                continue;
            };
            let Some((table, index)) = self.claimed.get(&read_bci) else {
                continue;
            };
            let Some(index_instruction) = ssa
                .blocks()
                .iter()
                .flat_map(|block| block.instructions())
                .find(|instruction| instruction.bci() == index.bci)
            else {
                continue;
            };
            let Some((_, selector_receiver)) = stack_operands(index_instruction).first().copied()
            else {
                continue;
            };
            let selector_receiver_type = match ssa.value(selector_receiver).ty() {
                jarde_jvm::method_ir::Value::Ref(jarde_jvm::method_ir::RefType::Named {
                    name,
                    ..
                }) => Some(name.clone()),
                jarde_jvm::method_ir::Value::Top
                | jarde_jvm::method_ir::Value::Second
                | jarde_jvm::method_ir::Value::Int
                | jarde_jvm::method_ir::Value::Float
                | jarde_jvm::method_ir::Value::Long
                | jarde_jvm::method_ir::Value::Double
                | jarde_jvm::method_ir::Value::Null
                | jarde_jvm::method_ir::Value::Ref(jarde_jvm::method_ir::RefType::Unknown)
                | jarde_jvm::method_ir::Value::UninitializedThis
                | jarde_jvm::method_ir::Value::Uninitialized { .. }
                | jarde_jvm::method_ir::Value::ReturnAddress => None,
            };
            let Some((keys, _)) = operations.get(switch_bci).and_then(Operation::switch) else {
                continue;
            };
            crate::stop::charge(
                budget,
                jarde_reader::budget::CountedBudgetDimension::IrItems,
                6_u64.saturating_add(u64::try_from(keys.len()).unwrap_or(u64::MAX)),
                Some(read_bci),
            )?;
            crate::stop::poll(budget, Some(read_bci))?;
            candidates.push(ClassSourceEnumSwitchCandidate {
                member: member.clone(),
                table: table.clone(),
                index: index.clone(),
                read_bci,
                switch_bci,
                selector_value,
                selector_receiver,
                selector_receiver_type,
                keys: keys.iter().map(|(key, _)| *key).collect(),
                projection: None,
            });
        }
        Ok(candidates)
    }
}

/// Reads every candidate dispatch-table read of one body.
///
/// The decision — the claim or the refusal — is taken for every selection, and this function builds
/// no owning record: the verdicts stay in the [`Plan`] and [`Plan::materialize`] writes the records
/// from them after the artifact is committed.
pub(crate) fn plan(ssa: &SsaTable, operations: &Operations) -> Plan {
    let mut plan = Plan::empty();
    for instruction in ssa.blocks().iter().flat_map(|block| block.instructions()) {
        let at = instruction.bci();
        if operations.get(at) != Some(&Operation::ArrayLoad) {
            continue;
        }
        match verify(instruction, ssa, operations) {
            Ok((table, index)) => {
                plan.claimed.insert(at, (table, index));
            }
            Err(refusal) => plan.refusals.push((at, refusal)),
        }
    }
    plan.refusals.sort_by_key(|(at, _)| *at);
    plan
}

/// Verifies one array read as a dispatch-table read, or states the link that failed.
fn verify(
    instruction: &jarde_jvm::method_ir::SsaInstruction,
    ssa: &SsaTable,
    operations: &Operations,
) -> Result<(TableRead, IndexCall), Refusal> {
    let at = instruction.bci();
    let shape = |detail: String| Refusal::shape("jre_enumswitch_shape", detail);
    let operands = stack_operands(instruction);
    if operands.len() != 2 {
        return Err(shape(format!(
            "the array read at BCI {at} reads {} value(s), and a dispatch-table read reads the table and the index",
            operands.len()
        )));
    }
    let (_, array) = operands[0];
    let (_, index) = operands[1];
    // The table: a **static** field of descriptor `[I`, named by the class's own pool.
    let table = match producer(ssa, array, operations) {
        Some((
            bci,
            Operation::Field {
                access: FieldAccess::Read,
                is_static: true,
                owner,
                name,
                descriptor,
            },
        )) => {
            if descriptor != "[I" {
                return Err(shape(format!(
                    "the array read at BCI {at} reads `{owner}.{name}` of descriptor `{descriptor}`, and a dispatch table is an `int[]`"
                )));
            }
            TableRead {
                bci,
                owner: owner.clone(),
                name: name.clone(),
                descriptor: descriptor.clone(),
            }
        }
        Some((bci, _)) => {
            return Err(shape(format!(
                "the array the read at BCI {at} indexes is produced at BCI {bci}, which is not a static field of this run's pool"
            )));
        }
        None => {
            return Err(shape(format!(
                "the array the read at BCI {at} indexes was not produced by an instruction of this body, so which table it reads is not stated"
            )));
        }
    };
    // The index: the result of an instance call that returns an `int` — the ordinal a compiler reads
    // the case index out of. Whether that call is *called* `ordinal` is not part of the shape.
    let index_call = match producer(ssa, index, operations) {
        Some((bci, Operation::Invoke(target))) => {
            if target.descriptor() != "()I"
                || !matches!(target.kind(), InvokeKind::Virtual | InvokeKind::Interface)
            {
                return Err(shape(format!(
                    "the read at BCI {at} is indexed by the call at BCI {bci} (`{}`), and a dispatch table is indexed by the result of an instance call returning an `int`",
                    target.descriptor()
                )));
            }
            IndexCall {
                bci,
                owner: target.owner().to_string(),
                name: target.name().to_string(),
                descriptor: target.descriptor().to_string(),
            }
        }
        Some((bci, _)) => {
            return Err(shape(format!(
                "the read at BCI {at} is indexed by the value produced at BCI {bci}, which is not a call"
            )));
        }
        None => {
            return Err(shape(format!(
                "the read at BCI {at} is indexed by a value no instruction of this body produced"
            )));
        }
    };
    Ok((table, index_call))
}

/// The instruction that produced one value, with its operation.
fn producer<'a>(
    ssa: &SsaTable,
    value: ValueId,
    operations: &'a Operations,
) -> Option<(u32, &'a Operation)> {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return None;
    };
    operations.get(*bci).map(|operation| (*bci, operation))
}

/// One static `int[]` field the run read as a dispatch table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TableRead {
    /// The BCI of the `getstatic`.
    pub bci: u32,
    /// The member's owner, in internal form, as the pool states it.
    pub owner: String,
    /// The member's name — evidence, never the shape: `$SwitchMap$…` is the compiler's convention.
    pub name: String,
    /// The member's descriptor, `[I` for a table this rule presents.
    pub descriptor: String,
}

/// The call one dispatch-table read is indexed by.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IndexCall {
    /// The BCI of the call.
    pub bci: u32,
    /// The call's owner, in internal form.
    pub owner: String,
    /// The call's name — evidence, never the shape.
    pub name: String,
    /// The call's descriptor, `()I` for an index this rule presents.
    pub descriptor: String,
}

/// What one candidate dispatch-table read was presented as, or why it was not (P3 2.3).
///
/// The record states the table and the call with their own BCIs, so "which `int[]` was this, indexed
/// by what" is answered by the run rather than by reading the text. What the record deliberately
/// does **not** state is a mapping to enum constants: that is another class's declaration, and this
/// run never read it (see the module documentation).
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EnumSwitchRecord {
    /// The BCI of the array read.
    pub read: u32,
    /// The static `int[]` field it reads, when the walk reached one.
    pub table: Option<TableRead>,
    /// The call it is indexed by, when the walk reached one.
    pub index: Option<IndexCall>,
    /// Whether the read was presented as the table read it is.
    pub presented: bool,
    /// Why it was not, when it was not.
    pub refusal: Option<EnumSwitchRefusal>,
}

impl EnumSwitchRecord {
    /// Whether this read was presented.
    pub fn presented(&self) -> bool {
        self.presented
    }

    /// The rule this record is answerable to.
    pub fn rule(&self) -> RuleVersion {
        RULE
    }
}

/// Why one array read was not presented as a dispatch-table read.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EnumSwitchRefusal {
    /// The diagnostic code, one `jre_enumswitch_*` per link of the verification that can fail.
    pub code: &'static str,
    /// The rule that refused the read.
    pub rule: RuleVersion,
    /// The declared requirement that fell short, when the refusal is one of the rule's own
    /// preconditions.
    pub requirement: Option<String>,
    /// One sentence stating which link failed, with the read's BCI in it.
    pub message: String,
}

impl EnumSwitchRefusal {
    fn of(refusal: &Refusal, read: u32) -> Self {
        Self {
            code: refusal.code(),
            rule: RULE,
            requirement: refusal.requirement().map(Precondition::describe),
            message: format!(
                "the array read at BCI {read} was not presented: {}",
                refusal.message()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pass::IrTable;

    #[test]
    fn the_rule_states_the_tables_it_reads_and_claims_no_mapping() {
        assert!(ENUMSWITCH.requires(Precondition::IrTable(IrTable::Ssa)));
        assert!(ENUMSWITCH.requires(Precondition::IrTable(IrTable::Code)));
        assert_eq!(
            ENUMSWITCH.required_release(),
            None,
            "a `switch` and an array read are Java in every release; the shape is the input's"
        );
        assert_eq!(RULE.citation(), "enumswitch@1");
    }
}
