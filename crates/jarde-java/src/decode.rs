//! ②′ The decode facts turned into the vocabulary the presentation is written in (P3 1.3b).
//!
//! # Where this input comes from
//!
//! One analysis run decodes the method's body once: the `raw_facts` pass reads the instructions and
//! their typed operands out of the class bytes, and the class's constant pool with them. Those
//! facts are moved into [`jarde_jvm::method_ir::MethodIr`] with the tables of that run, and this
//! module is the *only* place that reads them into an [`Operation`]:
//!
//! ```text
//! class bytes ─(raw_facts: one decode)─▶ MethodIr { code, constant_pool, canonical, frames, ssa }
//!                                                     │
//!                                           decode::Operations::of (here)
//!                                                     ▼
//!                                       Operations ──▶ region / build / emit
//! ```
//!
//! Nothing here re-decodes, nothing re-derives an operand from a rendered name, and no caller can
//! state an operation: the table is a pure function of the payload, so a branch's **polarity**, a
//! load's **slot**, a constant's **value**, a branch's **target** and a `switch`'s **keys** have
//! exactly one source — the run that read them. (Before this slice the *caller* built the table
//! beside the run, which meant an `ifeq` mislabelled as `ifne` produced silently inverted Java.)
//!
//! # What it maps and what it refuses
//!
//! Every opcode the provable subset models becomes a variant of [`Operation`], classified on the
//! operand's **effective** opcode (a `wide iload` *is* an `iload`; `0xc4` is a prefix, not an
//! instruction). Everything else — a conversion this subset has no operator for, `jsr`, a `ret` — is
//! [`Operation::Other`], which is a *stated* input: the statement it belongs to becomes a fallback
//! with a diagnostic, never a guess.
//!
//! The array accesses are modelled here as *facts of their own opcodes* (P3 2b): the eight element
//! reads and the eight element writes, the length, and the three creations. What each one states is
//! only what its own instruction states — the element type its opcode names (and, for the four
//! int-sized reads, deliberately none: JVMS 2.11.1 gives those five primitives one value shape) and
//! how many lengths it reads — while what the *array* holds is a fact of a frame and is read where
//! the subscript is written.
//!
//! The P3 2.2 slice models four more of them, and each one is a fact a *pattern rule* reads rather
//! than a presentation this layer writes: the allocation and the duplication a concatenation chain
//! starts with (`new`/`dup`), the field access a synthetic accessor's body reads or writes
//! (`getfield`/`putfield`/`getstatic`/`putstatic`, with the member the pool names), and the cast a
//! bridge applies to the value it forwards (`checkcast`, with the class the pool names). A caller
//! cannot state any of the four — they are read from the same decode as everything else — and
//! nothing below [`crate::build`] decides whether any of them is presented.
//!
//! P3 2.4 adds three more of the same kind: the two monitor instructions and `athrow`. The guarded
//! regions that slice presents are made of exactly these — a `synchronized` statement is one
//! `monitorenter` and the `monitorexit`s that pair with it, and a handler that rethrows its stored
//! exception is an `athrow` of that value — so the rules of [`crate::guard`] have to be able to
//! name them. What they are *not* is a presentation: no statement this layer writes is a monitor
//! instruction or a bare `throw`, and an unclaimed one is quoted like anything else.
//!
//! `invokedynamic` is one of the modelled ones (P3 2.1), and modelling it means stating the *site*:
//! its pool index, the bootstrap entry it names and the name and descriptor it presents. It does
//! **not** mean stating a lambda — a site's shape is decided above this module, against the class's
//! bootstrap table, by the `lambda@1` rule, and a site with any other bootstrap keeps only its
//! identity ([`Operation::InvokeDynamic`]).
//!
//! A constant-pool reference is resolved against the class's own pool, in the form the decode
//! already put it in ([`CpEntryKind::MethodRef`] carries its owner, name and descriptor resolved).
//! A reference that does not resolve, or an instruction whose operand the decode states no fact
//! for, is `Other` as well: this layer never invents the symbol an instruction names.

use std::collections::BTreeMap;

use jarde_reader::classfile::{
    CpEntryFacts, CpEntryKind, DescriptorKind, ImmediateValue, InstructionFact,
    InstructionOperands, MethodCodeFacts, SwitchOperands, cp_entry, descriptor_facts,
};

use crate::ast::Type;
use crate::facts::{
    ArithmeticOp, BitwiseOp, CallTarget, CompareOp, ConstantValue, DynamicSite, FieldAccess,
    InvokeKind, NumericComparisonOp, Operation, ShiftOp,
};

/// The operations of one decoded body, keyed by bytecode index.
///
/// The table holds one entry per instruction the decode published, so a BCI the payload's body does
/// not hold has no entry — which is what makes "this run decoded no instruction here" different
/// from "this run decoded an instruction this subset does not model" ([`Operation::Other`]).
pub(crate) struct Operations {
    operations: BTreeMap<u32, Operation>,
}

impl Operations {
    /// Reads one decoded body into the presentation's vocabulary.
    ///
    /// `pool` is the class's own constant pool as the same decode read it; it is what resolves an
    /// `invoke*`'s target and an `ldc`'s value.
    pub(crate) fn of(code: &MethodCodeFacts, pool: &[CpEntryFacts]) -> Self {
        let operands = code.operands();
        let mut operations = BTreeMap::new();
        for (index, instruction) in code.instructions.iter().enumerate() {
            let operation = operation_of(instruction, operands.get(index), pool);
            operations.insert(instruction.bci, operation);
        }
        Self { operations }
    }

    /// What the instruction at one bytecode index does, when this run decoded one there.
    pub(crate) fn get(&self, bci: u32) -> Option<&Operation> {
        self.operations.get(&bci)
    }

    /// Every decoded operation, in BCI order.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&u32, &Operation)> {
        self.operations.iter()
    }
}

/// What one decoded instruction does, as far as the presentation needs to know.
fn operation_of(
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
    pool: &[CpEntryFacts],
) -> Operation {
    // The effective opcode: `wide iload` is an `iload` and `wide iinc` an `iinc`, so every
    // classification below reads the opcode the instruction *is*, never the prefix byte.
    let opcode = operands.map_or(instruction.opcode, |operands| operands.effective_opcode);
    match opcode {
        // Constants the opcode itself encodes, and the two immediate forms.
        0x01 => Operation::Push(ConstantValue::Null),
        0x02..=0x08 => Operation::Push(ConstantValue::Int(i64::from(opcode) - 0x03)),
        0x09 => Operation::Push(ConstantValue::Long(0)),
        0x0a => Operation::Push(ConstantValue::Long(1)),
        0x10 | 0x11 => match operands.and_then(|operands| operands.immediate) {
            Some(ImmediateValue::Int(value)) => {
                Operation::Push(ConstantValue::Int(i64::from(value)))
            }
            // `fconst`/`dconst` decode as the same `ImmediateValue`s but have no literal this
            // subset writes (a `float`/`double` literal needs a spelling decision of its own).
            _ => Operation::Other,
        },
        0x12..=0x14 => constant(instruction, operands, pool),
        // Locals: the explicit forms and the `_0`..`_3` forms, whose index the decode states.
        0x15..=0x2d => match slot(operands) {
            Some(slot) => Operation::Load { slot },
            None => Operation::Other,
        },
        0x36..=0x4e => match slot(operands) {
            Some(slot) => Operation::Store { slot },
            None => Operation::Other,
        },
        0x60..=0x73 => match arithmetic(opcode) {
            Some(op) => Operation::Arithmetic { op },
            None => Operation::Other,
        },
        0x78..=0x7d => match shift(opcode) {
            Some(op) => Operation::Shift { op },
            None => Operation::Other,
        },
        0x85..=0x93 => match primitive_conversion(opcode) {
            Some((source, target)) => Operation::PrimitiveConversion { source, target },
            None => Operation::Other,
        },
        0x7e..=0x83 => match bitwise(opcode) {
            Some(op) => Operation::Bitwise { op },
            None => Operation::Other,
        },
        0x94..=0x98 => numeric_comparison(opcode),
        0x74..=0x77 => Operation::Negate,
        0x59 => Operation::Duplicate,
        0x84 => match (
            slot(operands),
            operands.and_then(|operands| operands.increment),
        ) {
            (Some(slot), Some(amount)) => Operation::Increment { slot, amount },
            _ => Operation::Other,
        },
        0x99..=0xa6 | 0xc6 | 0xc7 => comparison(opcode, instruction, operands),
        0xaa | 0xab => switch(instruction, operands),
        0xa7 | 0xc8 => Operation::Transfer,
        0xbb => allocate(instruction, operands, pool),
        0xb2..=0xb5 => field(opcode, instruction, operands, pool),
        0xb6..=0xb9 => invoke(opcode, instruction, pool),
        0xba => invokedynamic(instruction, pool),
        // The array element accesses (P3 2b): the eight reads and the eight writes, each with the
        // element type its own opcode states. The family a read belongs to is the element type the
        // **opcode** names, which is a fact of the instruction and not of the array it reads — what
        // the array's own type adds is said in [`Operation`] and read by `crate::build`.
        0x2e..=0x35 => array_read(opcode),
        0x4f..=0x56 => array_write(opcode),
        // The array's length, and the three instructions that allocate one (P3 2b). What each
        // allocates is the element type its own operand states and the number of lengths it reads.
        0xbe => Operation::ArrayLength,
        0xbc | 0xbd | 0xc5 => array_creation(opcode, instruction, operands, pool),
        0xc0 => check_cast(instruction, operands, pool),
        0xc1 => instance_of(instruction, operands, pool),
        0xac..=0xb1 => Operation::Return,
        0xbf => Operation::Throw,
        // The two monitor instructions of P3 2.4: the enter and the exit a `synchronized` statement
        // is made of. They are modelled as *shape* facts (`monitor@1` reads them to prove the
        // pairing), not as a presentation — nothing below this module writes either of them.
        0xc2 => Operation::Monitor { enter: true },
        0xc3 => Operation::Monitor { enter: false },
        _ => Operation::Other,
    }
}

/// The five JVM numeric comparison opcodes, retaining their operand type and unordered bias.
fn numeric_comparison(opcode: u8) -> Operation {
    let op = match opcode {
        0x94 => NumericComparisonOp::Long,
        0x95 => NumericComparisonOp::FloatLess,
        0x96 => NumericComparisonOp::FloatGreater,
        0x97 => NumericComparisonOp::DoubleLess,
        0x98 => NumericComparisonOp::DoubleGreater,
        _ => return Operation::Other,
    };
    Operation::NumericComparison { op }
}

/// The source stack category and explicit Java result type of one primitive conversion opcode.
///
/// The first twelve opcodes convert among the four JVM numeric categories. The final three still
/// read and write the verifier's int category, but their Java result types are byte, char and short;
/// keeping those targets here is what lets later expression consumers preserve their static type.
fn primitive_conversion(opcode: u8) -> Option<(Type, Type)> {
    let (source, target) = match opcode {
        0x85 => (Type::Int, Type::Long),
        0x86 => (Type::Int, Type::Float),
        0x87 => (Type::Int, Type::Double),
        0x88 => (Type::Long, Type::Int),
        0x89 => (Type::Long, Type::Float),
        0x8a => (Type::Long, Type::Double),
        0x8b => (Type::Float, Type::Int),
        0x8c => (Type::Float, Type::Long),
        0x8d => (Type::Float, Type::Double),
        0x8e => (Type::Double, Type::Int),
        0x8f => (Type::Double, Type::Long),
        0x90 => (Type::Double, Type::Float),
        0x91 => (Type::Int, Type::Byte),
        0x92 => (Type::Int, Type::Char),
        0x93 => (Type::Int, Type::Short),
        _ => return None,
    };
    Some((source, target))
}

/// The slot one instruction names, when the decode states it.
fn slot(operands: Option<&InstructionOperands>) -> Option<u16> {
    operands
        .and_then(|operands| operands.local)
        .map(|local| local.index)
}

/// The constant one `ldc` family instruction pushes, resolved in the class's own pool.
fn constant(
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
    pool: &[CpEntryFacts],
) -> Operation {
    let opcode = operands.map_or(instruction.opcode, |operands| operands.effective_opcode);
    let index = operands
        .and_then(|operands| operands.constant_pool_index)
        .or(instruction.constant_pool_index);
    let Some(index) = index else {
        return Operation::Other;
    };
    match cp_entry(pool, index).map(|entry| &entry.kind) {
        Ok(CpEntryKind::Integer { value }) => {
            Operation::Push(ConstantValue::Int(i64::from(*value)))
        }
        Ok(CpEntryKind::Long { value }) => Operation::Push(ConstantValue::Long(*value)),
        Ok(CpEntryKind::String { value, .. }) => {
            Operation::Push(ConstantValue::String(lossy(value)))
        }
        Ok(CpEntryKind::Class { name, .. }) if matches!(opcode, 0x12 | 0x13) => {
            match class_literal_type(name) {
                Some(ty) => Operation::Push(ConstantValue::Class {
                    ty,
                    pool_index: index,
                }),
                None => Operation::Other,
            }
        }
        // A `float`/`double`/`MethodType`/`MethodHandle` constant has no literal this subset writes,
        // and an index that resolves to nothing is not a constant this run can name. `ldc2_w` is
        // reserved for category-two values, so a Class item there is not admitted either.
        _ => Operation::Other,
    }
}

/// One `CONSTANT_Class` name that has a proved Java source spelling.
///
/// Ordinary Class entries carry internal names; array Class entries carry field descriptors. The
/// reader keeps the pool name as Modified UTF-8 bytes. Decode that encoding before validating the
/// source spelling: a supplementary Java identifier is a surrogate pair in the pool, not standard
/// UTF-8. Malformed sequences become U+FFFD in `lossy` and are rejected here rather than published
/// as a plausible Java path. The current type-name policy is the existing bounded ASCII identifier
/// grammar excluding `$`; valid Unicode names and binary names containing `$` remain known recovery
/// boundaries until Java 8's identifier tables and the required nested/classpath name evidence are
/// represented here.
fn class_literal_type(name: &jarde_reader::model::JvmBytes) -> Option<String> {
    let raw = lossy(name);
    if raw.contains('\u{fffd}') {
        return None;
    }
    if let Some(array_descriptor) = raw.strip_prefix('[') {
        let element_descriptor = array_descriptor.trim_start_matches('[');
        if let Some(internal_name) = element_descriptor
            .strip_prefix('L')
            .and_then(|element| element.strip_suffix(';'))
        {
            if !source_internal_name(internal_name) {
                return None;
            }
        } else if !matches!(
            element_descriptor,
            "Z" | "B" | "C" | "S" | "I" | "J" | "F" | "D"
        ) {
            return None;
        }
        let (ty, end) = crate::lambda::parse_type(raw.as_bytes(), 0)?;
        return match ty {
            Type::Reference(spelling) if end == raw.len() => Some(spelling),
            _ => None,
        };
    }
    if !source_internal_name(&raw) {
        return None;
    }
    crate::build::spell_reference(&raw)
}

/// Whether every component of one internal class name is a source identifier this layer writes.
/// `$` is deliberately refused: from this pool entry alone it may be a nested binary name that
/// needs `Outer.Inner` source spelling, or a top-level `$` identifier whose classpath binding this
/// single-class read cannot prove.
fn source_internal_name(name: &str) -> bool {
    !name.is_empty() && !name.contains('$') && name.split('/').all(crate::names::is_java_identifier)
}

/// The arithmetic one opcode performs, when this subset has that operator.
///
/// The opcodes run in groups of four — `int`, `long`, `float`, `double` for each operator — so the
/// operator is the group, and the operand's own type (which the values carry) decides nothing here.
fn arithmetic(opcode: u8) -> Option<ArithmeticOp> {
    match (opcode - 0x60) / 4 {
        0 => Some(ArithmeticOp::Add),
        1 => Some(ArithmeticOp::Subtract),
        2 => Some(ArithmeticOp::Multiply),
        3 => Some(ArithmeticOp::Divide),
        4 => Some(ArithmeticOp::Remainder),
        _ => None,
    }
}

/// The shared operator of the paired `int` and `long` bitwise opcodes.
fn bitwise(opcode: u8) -> Option<BitwiseOp> {
    match opcode {
        0x7e | 0x7f => Some(BitwiseOp::And),
        0x80 | 0x81 => Some(BitwiseOp::Or),
        0x82 | 0x83 => Some(BitwiseOp::Xor),
        _ => None,
    }
}

fn shift(opcode: u8) -> Option<ShiftOp> {
    match opcode {
        0x78 | 0x79 => Some(ShiftOp::Left),
        0x7a | 0x7b => Some(ShiftOp::Right),
        0x7c | 0x7d => Some(ShiftOp::UnsignedRight),
        _ => None,
    }
}

/// The array read one `*aload` opcode is, with the element type the opcode itself states.
///
/// The eight reads fall into three groups, and the group is the whole of what the opcode says about
/// the element:
///
/// * `iaload`, and the `baload`/`caload`/`saload` that share its shape, read one **int-sized**
///   value. Which of `boolean`, `byte`, `char`, `short` and `int` the array holds is not in the
///   instruction: JVMS 2.11.1 gives all five one value shape and one slot, so `[Z` and `[B` are
///   read with the very same `baload`. The array's own type states it, and the frames carry it — so
///   `crate::build` reads the refinement there, and a read whose array the frames leave unknown is
///   the `int` the opcode's shape is;
/// * `laload`, `faload` and `daload` name their element type outright;
/// * `aaload` reads a reference, and *which* reference is a fact of the array's own type alone
///   (`[Ljava/lang/String;` reads a `java.lang.String`), which is why it states none.
fn array_read(opcode: u8) -> Operation {
    match opcode {
        // The one read the enum-switch rule reads (`enumswitch@1`, P3 2.3): an `int[]`'s element.
        // The variant is that rule's own name for the read, and the three reads below are *not* it —
        // a dispatch table is an `int[]`, so a `baload` is no candidate of that rule at all.
        0x2e => Operation::ArrayLoad,
        0x2f => Operation::ArrayElementLoad {
            element: Some(Type::Long),
        },
        0x30 => Operation::ArrayElementLoad {
            element: Some(Type::Float),
        },
        0x31 => Operation::ArrayElementLoad {
            element: Some(Type::Double),
        },
        0x32 => Operation::ArrayElementLoad { element: None },
        // The three that share the int-sized shape with `iaload`: the opcode states the shape's own
        // `int`, and the array's own type is what may state another of the four.
        0x33..=0x35 => Operation::ArrayElementLoad {
            element: Some(Type::Int),
        },
        _ => Operation::Other,
    }
}

/// The array write one `*astore` opcode is: the mirror of [`array_read`], read the same way.
fn array_write(opcode: u8) -> Operation {
    match opcode {
        0x4f | 0x54..=0x56 => Operation::ArrayStore {
            element: Some(Type::Int),
        },
        0x50 => Operation::ArrayStore {
            element: Some(Type::Long),
        },
        0x51 => Operation::ArrayStore {
            element: Some(Type::Float),
        },
        0x52 => Operation::ArrayStore {
            element: Some(Type::Double),
        },
        0x53 => Operation::ArrayStore { element: None },
        _ => Operation::Other,
    }
}

/// The array one allocation instruction creates (`newarray`, `anewarray`, `multianewarray`).
///
/// Three instructions state the same two facts from three different sources (JVMS 6.5):
///
/// * `newarray` names a **primitive** element with an `atype` code of its own (4..=11) and reads
///   the one length of the one dimension it allocates;
/// * `anewarray` names a **class** in the class's own pool, and allocates one dimension whose
///   element is that class — or whose component is itself an array descriptor. The latter is the
///   `new int[n][]` shape: the pool's array descriptor supplies the component rank and this
///   instruction supplies its outer dimension;
/// * `multianewarray` names an **array class** in the pool and states in its own operand how many of
///   that class's dimensions this instruction allocates. The element type is the component the
///   descriptor names and the number of lengths is the operand; a smaller positive count is a
///   partial allocation whose remaining dimensions are written as empty brackets.
fn array_creation(
    opcode: u8,
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
    pool: &[CpEntryFacts],
) -> Operation {
    match opcode {
        0xbc => match operands.and_then(|operands| operands.atype) {
            Some(atype) => match primitive_array_of(atype) {
                Some(element) => Operation::NewArray {
                    element,
                    dimensions: 1,
                    total_dimensions: 1,
                },
                None => Operation::Other,
            },
            None => Operation::Other,
        },
        0xbd => {
            let Some(index) = pool_index(instruction, operands) else {
                return Operation::Other;
            };
            let Ok(entry) = cp_entry(pool, index) else {
                return Operation::Other;
            };
            let CpEntryKind::Class { name, .. } = &entry.kind else {
                return Operation::Other;
            };
            let name = lossy(name);
            // A `CONSTANT_Class` entry names an array with a descriptor and a class with an
            // internal name (JVMS 4.4.1). An array descriptor is the component of the one new
            // dimension this instruction allocates; a class name is the ordinary rank-one case.
            if name.starts_with('[') {
                let Ok(facts) = descriptor_facts(name.as_bytes(), DescriptorKind::Field) else {
                    return Operation::Other;
                };
                let Some(component) = facts.single() else {
                    return Operation::Other;
                };
                let Some(total_dimensions) = component.dimensions().checked_add(1) else {
                    return Operation::Other;
                };
                match crate::lambda::type_of_base(component.base()) {
                    Some(element)
                        if component.is_array() && total_dimensions <= u32::from(u8::MAX) =>
                    {
                        Operation::NewArray {
                            element,
                            dimensions: 1,
                            total_dimensions: u8::try_from(total_dimensions).unwrap_or(u8::MAX),
                        }
                    }
                    _ => Operation::Other,
                }
            } else {
                match crate::build::spell_reference(&name) {
                    Some(element) => Operation::NewArray {
                        element: Type::Reference(element),
                        dimensions: 1,
                        total_dimensions: 1,
                    },
                    None => Operation::Other,
                }
            }
        }
        0xc5 => {
            let Some(index) = pool_index(instruction, operands) else {
                return Operation::Other;
            };
            let Some(dimensions) = operands.and_then(|operands| operands.dimensions) else {
                return Operation::Other;
            };
            let Ok(entry) = cp_entry(pool, index) else {
                return Operation::Other;
            };
            let CpEntryKind::Class { name, .. } = &entry.kind else {
                return Operation::Other;
            };
            let descriptor = lossy(name);
            let Ok(facts) = descriptor_facts(descriptor.as_bytes(), DescriptorKind::Field) else {
                return Operation::Other;
            };
            let Some(component) = facts.single() else {
                return Operation::Other;
            };
            // The operand's count is how many dimensions this instruction allocates, while the
            // descriptor's rank is the complete result. A smaller positive count is the legal
            // prefix form (`new int[n][m][]`).
            let total_dimensions = component.dimensions();
            if component.is_array()
                && dimensions != 0
                && u32::from(dimensions) <= total_dimensions
                && total_dimensions <= u32::from(u8::MAX)
            {
                match crate::lambda::type_of_base(component.base()) {
                    Some(element) => Operation::NewArray {
                        element,
                        dimensions,
                        total_dimensions: u8::try_from(total_dimensions).unwrap_or(u8::MAX),
                    },
                    None => Operation::Other,
                }
            } else {
                Operation::Other
            }
        }
        _ => Operation::Other,
    }
}

/// The element type one `newarray` `atype` code names (JVMS 6.5), when it names one at all.
///
/// The codes are the JVM's own array element codes: `4` boolean, `5` char, `6` float, `7` double,
/// `8` byte, `9` short, `10` int and `11` long. Anything else is not an element type of this
/// instruction, which the reader leaves to this reading rather than deciding for it.
fn primitive_array_of(atype: u8) -> Option<Type> {
    Some(match atype {
        4 => Type::Boolean,
        5 => Type::Char,
        6 => Type::Float,
        7 => Type::Double,
        8 => Type::Byte,
        9 => Type::Short,
        10 => Type::Int,
        11 => Type::Long,
        _ => return None,
    })
}

/// The condition one branch tests and the BCI it transfers to.
///
/// Both halves are decode facts. The *sense* is what an `ifeq` means; the *target* is the address
/// the branch's own operand names, and it is what distinguishes the arm the branch transfers to
/// from the one control falls through to — an `ifeq` whose target is *behind* it is the condition
/// of a loop, and no structural rule could tell that from the successor order alone.
fn comparison(
    opcode: u8,
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
) -> Operation {
    let op = match opcode {
        0x99 => CompareOp::JumpIfZero,
        0x9a => CompareOp::JumpIfNotZero,
        0x9b => CompareOp::JumpIfNegative,
        0x9c => CompareOp::JumpIfNotNegative,
        0x9d => CompareOp::JumpIfPositive,
        0x9e => CompareOp::JumpIfNotPositive,
        0x9f | 0xa5 => CompareOp::JumpIfSame,
        0xa0 | 0xa6 => CompareOp::JumpIfDifferent,
        0xa1 => CompareOp::JumpIfLess,
        0xa2 => CompareOp::JumpIfGreaterOrEqual,
        0xa3 => CompareOp::JumpIfGreater,
        0xa4 => CompareOp::JumpIfLessOrEqual,
        0xc6 => CompareOp::JumpIfNull,
        0xc7 => CompareOp::JumpIfNotNull,
        _ => return Operation::Other,
    };
    match operands
        .and_then(|operands| operands.branch_offset)
        .and_then(|offset| absolute(instruction.bci, offset))
    {
        Some(target) => Operation::Comparison { op, target },
        // A branch whose target the decode does not state cannot be split into arms at all, and
        // guessing the target from the successor order is exactly the guess this layer refuses.
        None => Operation::Other,
    }
}

/// The keys and targets one `tableswitch`/`lookupswitch` enumerates.
fn switch(instruction: &InstructionFact, operands: Option<&InstructionOperands>) -> Operation {
    let Some(switch) = operands.and_then(|operands| operands.switch.as_ref()) else {
        return Operation::Other;
    };
    let (cases, default) = match switch {
        SwitchOperands::Table {
            default_offset,
            low,
            high,
            offsets,
        } => {
            // A hostile `high` cannot invent entries: the keys are exactly the range the decode
            // enumerated, and the offsets must agree with it entry for entry.
            let expected = i64::from(*high) - i64::from(*low) + 1;
            if expected < 0 || usize::try_from(expected).ok() != Some(offsets.len()) {
                return Operation::Other;
            }
            let cases = offsets
                .iter()
                .enumerate()
                .map(|(index, offset)| {
                    absolute(instruction.bci, *offset)
                        .map(|target| (i64::from(*low) + index as i64, target))
                })
                .collect::<Option<Vec<(i64, u32)>>>();
            let default = absolute(instruction.bci, *default_offset);
            (cases, default)
        }
        SwitchOperands::Lookup {
            default_offset,
            pairs,
        } => {
            let cases = pairs
                .iter()
                .map(|(key, offset)| {
                    absolute(instruction.bci, *offset).map(|target| (i64::from(*key), target))
                })
                .collect::<Option<Vec<(i64, u32)>>>();
            let default = absolute(instruction.bci, *default_offset);
            (cases, default)
        }
    };
    match (cases, default) {
        (Some(cases), Some(default)) => Operation::Switch { cases, default },
        _ => Operation::Other,
    }
}

/// The invocation one `invoke*` instruction performs, with the symbol the class's pool states.
fn invoke(opcode: u8, instruction: &InstructionFact, pool: &[CpEntryFacts]) -> Operation {
    let kind = match opcode {
        0xb6 => InvokeKind::Virtual,
        0xb7 => InvokeKind::Special,
        0xb8 => InvokeKind::Static,
        0xb9 => InvokeKind::Interface,
        _ => return Operation::Other,
    };
    let Some(index) = instruction.constant_pool_index else {
        return Operation::Other;
    };
    match cp_entry(pool, index).map(|entry| &entry.kind) {
        Ok(CpEntryKind::MethodRef {
            owner,
            name,
            descriptor,
            ..
        }) => Operation::Invoke(CallTarget::new(
            kind,
            lossy(owner),
            lossy(name),
            lossy(descriptor),
            false,
        )),
        Ok(CpEntryKind::InterfaceMethodRef {
            owner,
            name,
            descriptor,
            ..
        }) => Operation::Invoke(CallTarget::new(
            kind,
            lossy(owner),
            lossy(name),
            lossy(descriptor),
            true,
        )),
        // `invokedynamic` names a bootstrap method, not a member this layer can write a call for.
        _ => Operation::Other,
    }
}

/// The dynamic call site one `invokedynamic` performs.
///
/// The site's identity is in the class's own pool: its `InvokeDynamic` entry states the name and
/// descriptor the site presents, and the index of the `BootstrapMethods` entry that resolves it.
/// The *shape* is deliberately not read here — whether this site is a lambda depends on the method
/// handle and the static arguments that entry names, which is [`crate::lambda`]'s reading of the
/// payload's bootstrap table and the same pool, under the `lambda@1` rule. Stating the site here and
/// the shape there is what keeps "an `invokedynamic` was decoded" separate from "this one is a
/// `LambdaMetafactory` call", which is exactly the difference A04 turns on.
///
/// A site whose pool entry does not resolve is `Other`, like every other reference this layer cannot
/// name: the instruction is stated as unmodelled rather than presented from half its facts.
fn invokedynamic(instruction: &InstructionFact, pool: &[CpEntryFacts]) -> Operation {
    let Some(index) = instruction.constant_pool_index else {
        return Operation::Other;
    };
    match cp_entry(pool, index).map(|entry| &entry.kind) {
        Ok(CpEntryKind::InvokeDynamic {
            bootstrap_method_attr_index,
            name,
            descriptor,
            ..
        }) => Operation::InvokeDynamic(DynamicSite::new(
            index,
            *bootstrap_method_attr_index,
            lossy(name),
            lossy(descriptor),
        )),
        // `invokedynamic` names a dynamic call site and nothing else: a different pool entry behind
        // the opcode is a class this layer cannot state an operation for.
        _ => Operation::Other,
    }
}

/// The instance one `new` allocates, named by the class its own pool entry states.
///
/// An allocation is the *start* of a shape, not a presentation: the value it pushes is not even an
/// instance until a constructor has run on it, so nothing here writes `new …` — the concatenation
/// rule is what decides that a particular allocation is the one a verified chain builds.
fn allocate(
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
    pool: &[CpEntryFacts],
) -> Operation {
    let Some(index) = pool_index(instruction, operands) else {
        return Operation::Other;
    };
    match cp_entry(pool, index).map(|entry| &entry.kind) {
        Ok(CpEntryKind::Class { name, .. }) => Operation::Allocate { ty: lossy(name) },
        // A `new` whose class this run cannot name is an instruction whose target is unknown, and
        // this layer never invents the symbol an instruction refers to.
        _ => Operation::Other,
    }
}

/// One field access, with the member the class's own pool states.
fn field(
    opcode: u8,
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
    pool: &[CpEntryFacts],
) -> Operation {
    let access = match opcode {
        0xb2 | 0xb4 => FieldAccess::Read,
        0xb3 | 0xb5 => FieldAccess::Write,
        _ => return Operation::Other,
    };
    let is_static = matches!(opcode, 0xb2 | 0xb3);
    let Some(index) = pool_index(instruction, operands) else {
        return Operation::Other;
    };
    match cp_entry(pool, index).map(|entry| &entry.kind) {
        Ok(CpEntryKind::FieldRef {
            owner,
            name,
            descriptor,
            ..
        }) => Operation::Field {
            access,
            is_static,
            owner: lossy(owner),
            name: lossy(name),
            descriptor: lossy(descriptor),
        },
        _ => Operation::Other,
    }
}

/// The class one `checkcast` requires its value to be an instance of.
fn check_cast(
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
    pool: &[CpEntryFacts],
) -> Operation {
    let Some(index) = pool_index(instruction, operands) else {
        return Operation::Other;
    };
    match cp_entry(pool, index).map(|entry| &entry.kind) {
        Ok(CpEntryKind::Class { name, .. }) => Operation::CheckCast { ty: lossy(name) },
        _ => Operation::Other,
    }
}

fn instance_of(
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
    pool: &[CpEntryFacts],
) -> Operation {
    let Some(index) = pool_index(instruction, operands) else {
        return Operation::Other;
    };
    match cp_entry(pool, index).map(|entry| &entry.kind) {
        Ok(CpEntryKind::Class { name, .. }) => Operation::InstanceOf { ty: lossy(name) },
        _ => Operation::Other,
    }
}

/// The pool index one instruction names: the operand's own index when the decode states one, and
/// the instruction's when it does not.
fn pool_index(
    instruction: &InstructionFact,
    operands: Option<&InstructionOperands>,
) -> Option<u16> {
    operands
        .and_then(|operands| operands.constant_pool_index)
        .or(instruction.constant_pool_index)
}

/// One BCI plus a relative offset, when the sum is a bytecode index at all.
fn absolute(bci: u32, offset: i32) -> Option<u32> {
    u32::try_from(i64::from(bci) + i64::from(offset)).ok()
}

/// The text of one pool fact, as its Modified UTF-8 bytes read back (JVMS 4.4.7).
///
/// Modified UTF-8 is UTF-8 with two differences, and both of them are why a standard-UTF-8 read of
/// these bytes is wrong rather than merely lossy: U+0000 is written as the overlong `C0 80`, which a
/// standard reader refuses, and a supplementary character is written as the two halves of its
/// surrogate pair, each in the three-byte form of one UTF-16 code unit, which a standard reader
/// refuses as well. `String::from_utf8_lossy` therefore reads the string constant `"a\u0000b"` —
/// the bytes `61 C0 80 62` — as `a` and **two** U+FFFD: one NUL the class file states, lost.
///
/// One sequence at a time, by the form its lead byte states:
///
/// * `01..=7F` is that character; `C0 80` is U+0000; `C2..=DF 80..=BF` is U+0080..=U+07FF; and
///   `E0..=EF 80..=BF 80..=BF` is the one code point U+0800..=U+FFFF those bytes carry;
/// * a high surrogate immediately followed by a low surrogate, each in its own three-byte form, is
///   the one scalar that pair stands for;
/// * every other sequence is written as exactly one U+FFFD, and the read goes on after it: a raw
///   `00` (not a character in this encoding), a continuation byte where a lead belongs, an overlong
///   two- or three-byte form, a lone half of a surrogate pair, a truncated sequence, and a four-byte
///   or longer UTF-8 form — which this encoding has none of, so it is never read as the scalar a
///   standard reader would make of it.
///
/// The read never fails and never panics. A payload a class file should not hold is evidence this
/// layer can state but cannot spell, and one replacement character per sequence is the whole of the
/// difference between those bytes and the text.
fn lossy(bytes: &jarde_reader::model::JvmBytes) -> String {
    /// What one sequence with no Modified UTF-8 reading is written as.
    const REPLACEMENT: char = '\u{fffd}';

    // The two forms this encoding has carry U+0080..=U+FFFF less the surrogate halves, so a code
    // point that reaches this closure is always a scalar: `unwrap_or` is what states that this read
    // cannot panic, not what handles a case.
    let scalar = |value: u32| char::from_u32(value).unwrap_or(REPLACEMENT);

    let raw = &bytes.0;
    let mut text = String::with_capacity(raw.len());
    let mut offset = 0usize;
    while offset < raw.len() {
        let first = raw[offset];
        // The width the lead byte states. The two bytes that state nothing at all — a raw `00`,
        // which is not a character here, and a continuation byte where a lead belongs — are one
        // replacement each, and the read goes on with the byte after them.
        let width = match first {
            0x01..=0x7f => {
                text.push(char::from(first));
                offset += 1;
                continue;
            }
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            // A four-byte or longer UTF-8 form — a shape this encoding has none of. Its width is
            // the count of leading one bits in the lead byte, and the sequence is replaced whole
            // below rather than decoded as the scalar a standard reader would make of it.
            0xf0..=0xf7 => 4,
            0xf8..=0xfb => 5,
            0xfc..=0xfd => 6,
            _ => {
                text.push(REPLACEMENT);
                offset += 1;
                continue;
            }
        };
        // A sequence the payload truncates is one replacement for the prefix that is there — the
        // lead byte and the continuation bytes that follow it — and the read resumes after them.
        let mut taken = 1usize;
        while taken < width
            && raw
                .get(offset + taken)
                .is_some_and(|byte| *byte & 0xc0 == 0x80)
        {
            taken += 1;
        }
        if taken < width {
            text.push(REPLACEMENT);
            offset += taken;
            continue;
        }
        if width > 3 {
            text.push(REPLACEMENT);
            offset += width;
            continue;
        }
        // The code point the sequence's bytes hold, out of the bits below each lead and
        // continuation prefix.
        let value = if width == 2 {
            (u32::from(first & 0x1f) << 6) | u32::from(raw[offset + 1] & 0x3f)
        } else {
            (u32::from(first & 0x0f) << 12)
                | (u32::from(raw[offset + 1] & 0x3f) << 6)
                | u32::from(raw[offset + 2] & 0x3f)
        };
        if width == 2 {
            match value {
                // `C0 80`: this encoding's U+0000, the one code point under U+0080 a two-byte form
                // is allowed to carry. Every other one is the overlong spelling of a character a
                // one-byte form already states.
                0 => text.push('\u{0}'),
                0x01..=0x7f => text.push(REPLACEMENT),
                _ => text.push(scalar(value)),
            }
            offset += 2;
            continue;
        }
        match value {
            // An overlong three-byte form, and a low half of a surrogate pair with no high half
            // before it: neither is a code point this text can hold.
            0x0000..=0x07ff | 0xdc00..=0xdfff => text.push(REPLACEMENT),
            // A high half: one scalar exactly when the low half of the pair immediately follows in
            // its own three-byte form, and otherwise the one replacement this sequence is written
            // as. The bytes right after this sequence state it — `ED` is the only lead byte whose
            // three-byte form can land in U+DC00..=U+DFFF, and only while both bytes below it are
            // continuation bytes.
            0xd800..=0xdbff => {
                let low = match raw.get(offset + 3..offset + 6) {
                    Some([0xed, second, third])
                        if second & 0xc0 == 0x80 && third & 0xc0 == 0x80 =>
                    {
                        let low =
                            0xd000 | (u32::from(second & 0x3f) << 6) | u32::from(third & 0x3f);
                        (0xdc00..=0xdfff).contains(&low).then_some(low)
                    }
                    _ => None,
                };
                match low {
                    Some(low) => {
                        // The scalar the pair encodes, which is always in the astral planes.
                        text.push(scalar(0x1_0000 + ((value - 0xd800) << 10) + (low - 0xdc00)));
                        offset += 6;
                        continue;
                    }
                    None => text.push(REPLACEMENT),
                }
            }
            _ => text.push(scalar(value)),
        }
        offset += 3;
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    use jarde_reader::budget::{Budget, Limits};
    use jarde_reader::classfile::{class_facts, cp_class_name, method_code_facts, test_class};
    use jarde_reader::model::JvmBytes;

    /// The operations of one assembled body, read exactly the way a recovery run reads them: a real
    /// class file, the reader's own decode, and the pool of the same read.
    fn decoded(code: &[u8]) -> Operations {
        decoded_with_pool(code, |_| {})
    }

    /// The same body decode with one pool fact changed to exercise a constant-kind refusal.
    fn decoded_with_pool(code: &[u8], change: impl FnOnce(&mut [CpEntryFacts])) -> Operations {
        let bytes = test_class::single_method(52, 8, 3, code);
        let mut budget = Budget::new(Limits {
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        let header = class_facts(&bytes, &mut budget).expect("the assembled class is a class file");
        let mut pool = header.constant_pool.clone();
        let member = header
            .methods
            .iter()
            .find(|member| member.name.raw().0 == b"method")
            .expect("the assembled class declares `method`");
        let facts =
            method_code_facts(&bytes, member, &mut budget).expect("the assembled body decodes");
        change(&mut pool);
        Operations::of(&facts, &pool)
    }

    fn at(operations: &Operations, bci: u32) -> Operation {
        operations
            .get(bci)
            .unwrap_or_else(|| panic!("the body decodes an instruction at BCI {bci}"))
            .clone()
    }

    #[test]
    fn ldc_and_ldc_w_admit_class_pool_entries_with_their_pool_index() {
        let ldc = decoded(&[0x12, 11, 0x57, 0xb1]);
        let ldc_w = decoded(&[0x13, 0, 11, 0x57, 0xb1]);
        let expected = Operation::Push(ConstantValue::Class {
            ty: "java.lang.Runnable".to_owned(),
            pool_index: 11,
        });
        assert_eq!(at(&ldc, 0), expected);
        assert_eq!(at(&ldc_w, 0), expected);

        let ldc2_w = decoded(&[0x14, 0, 11, 0x57, 0xb1]);
        assert_eq!(at(&ldc2_w, 0), Operation::Other);
    }

    #[test]
    fn ldc_does_not_admit_method_type_or_method_handle_pool_entries() {
        let method_type = decoded_with_pool(&[0x12, 11, 0x57, 0xb1], |pool| {
            pool[10].kind = CpEntryKind::MethodType {
                descriptor_index: 6,
                descriptor: jarde_reader::model::JvmBytes(b"()V".to_vec()),
            };
        });
        assert_eq!(at(&method_type, 0), Operation::Other);

        let method_handle = decoded_with_pool(&[0x12, 11, 0x57, 0xb1], |pool| {
            pool[10].kind = CpEntryKind::MethodHandle {
                reference_kind: 1,
                reference_index: 15,
            };
        });
        assert_eq!(at(&method_handle, 0), Operation::Other);
    }

    #[test]
    fn modified_utf8_class_names_are_decoded_before_the_ascii_spelling_gate() {
        let deseret = jarde_reader::model::JvmBytes(vec![0xed, 0xa0, 0x81, 0xed, 0xb0, 0x80]);
        assert_eq!(lossy(&deseret), "𐐀");
        assert_eq!(
            class_literal_type(&deseret),
            None,
            "the class name is valid Modified UTF-8 but outside the current ASCII source-name policy"
        );

        let malformed_pair = jarde_reader::model::JvmBytes(vec![0xed, 0xa0, 0x81]);
        assert_eq!(lossy(&malformed_pair), "\u{fffd}");
        assert_eq!(class_literal_type(&malformed_pair), None);

        let malformed_overlong = jarde_reader::model::JvmBytes(vec![0xc1, 0x81]);
        assert_eq!(class_literal_type(&malformed_overlong), None);

        let invalid_java_identifier = jarde_reader::model::JvmBytes(b"invalid-name".to_vec());
        assert_eq!(class_literal_type(&invalid_java_identifier), None);

        let dollar_binary_name = jarde_reader::model::JvmBytes(b"OuterDollarProbe$Inner".to_vec());
        assert_eq!(
            class_literal_type(&dollar_binary_name),
            None,
            "the Class pool alone cannot distinguish a nested binary name from a top-level `$` name"
        );
    }

    #[test]
    fn the_polarity_of_a_branch_and_the_target_it_jumps_to_come_from_the_decode() {
        // iconst_0; istore_1; iload_1; ifeq +8; iconst_1; istore_2; goto +5; iconst_2; istore_2; return
        let operations = decoded(&[
            0x03, 0x3c, 0x1b, 0x99, 0x00, 0x08, 0x04, 0x3d, 0xa7, 0x00, 0x05, 0x05, 0x3d, 0xb1,
        ]);
        assert_eq!(at(&operations, 0), Operation::Push(ConstantValue::Int(0)));
        assert_eq!(at(&operations, 1), Operation::Store { slot: 1 });
        assert_eq!(at(&operations, 2), Operation::Load { slot: 1 });
        assert_eq!(
            at(&operations, 3),
            Operation::Comparison {
                op: CompareOp::JumpIfZero,
                target: 11,
            },
            "`ifeq 11` is the sense and the address the operand names"
        );
        assert_eq!(at(&operations, 8), Operation::Transfer);
        assert_eq!(at(&operations, 13), Operation::Return);
        assert_eq!(
            operations.iter().count(),
            10,
            "one operation per instruction of the body, in BCI order"
        );
        assert_eq!(
            operations.iter().map(|(bci, _)| *bci).collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 6, 7, 8, 11, 12, 13],
            "the BCIs are the ones the decode published"
        );
    }

    #[test]
    fn the_six_integral_bitwise_opcodes_keep_their_shared_operators() {
        let operations = decoded(&[
            0x03, 0x04, 0x7e, // iconst_0; iconst_1; iand
            0x09, 0x0a, 0x7f, // lconst_0; lconst_1; land
            0x03, 0x04, 0x80, // iconst_0; iconst_1; ior
            0x09, 0x0a, 0x81, // lconst_0; lconst_1; lor
            0x03, 0x04, 0x82, // iconst_0; iconst_1; ixor
            0x09, 0x0a, 0x83, // lconst_0; lconst_1; lxor
            0xb1,
        ]);
        for (bci, op) in [
            (2, BitwiseOp::And),
            (5, BitwiseOp::And),
            (8, BitwiseOp::Or),
            (11, BitwiseOp::Or),
            (14, BitwiseOp::Xor),
            (17, BitwiseOp::Xor),
        ] {
            assert_eq!(at(&operations, bci), Operation::Bitwise { op });
        }
    }

    #[test]
    fn the_six_shift_opcodes_keep_their_directions() {
        let operations = decoded(&[
            0x03, 0x04, 0x78, // ishl
            0x09, 0x04, 0x79, // lshl
            0x03, 0x04, 0x7a, // ishr
            0x09, 0x04, 0x7b, // lshr
            0x03, 0x04, 0x7c, // iushr
            0x09, 0x04, 0x7d, // lushr
            0xb1,
        ]);
        for (bci, op) in [
            (2, ShiftOp::Left),
            (5, ShiftOp::Left),
            (8, ShiftOp::Right),
            (11, ShiftOp::Right),
            (14, ShiftOp::UnsignedRight),
            (17, ShiftOp::UnsignedRight),
        ] {
            assert_eq!(at(&operations, bci), Operation::Shift { op });
        }
    }

    #[test]
    fn an_invocation_names_its_target_out_of_the_classs_own_pool() {
        // aconst_null; astore_1; aload_1; lconst_0; invokeinterface Runnable.run:(J)V; return
        let operations = decoded(&[0x01, 0x4c, 0x2b, 0x09, 0xb9, 0x00, 0x0f, 0x02, 0x00, 0xb1]);
        match at(&operations, 4) {
            Operation::Invoke(target) => {
                assert_eq!(target.kind(), InvokeKind::Interface);
                assert_eq!(
                    target.owner(),
                    "java/lang/Runnable",
                    "the owner comes from the pool entry the instruction names"
                );
                assert_eq!(target.name(), "run");
                assert_eq!(target.descriptor(), "(J)V");
                assert!(target.is_interface_reference());
            }
            other => panic!("expected the invocation, got {other:?}"),
        }
        assert_eq!(at(&operations, 3), Operation::Push(ConstantValue::Long(0)));
    }

    #[test]
    fn the_four_numeric_negation_opcodes_decode_to_one_fact() {
        // iconst_1; ineg; lconst_0; lneg; fconst_0; fneg; dconst_0; dneg; return
        let operations = decoded(&[0x04, 0x74, 0x09, 0x75, 0x0b, 0x76, 0x0e, 0x77, 0xb1]);
        for bci in [1, 3, 5, 7] {
            assert_eq!(
                at(&operations, bci),
                Operation::Negate,
                "numeric negation at BCI {bci} shares one value operation"
            );
        }
    }

    #[test]
    fn all_fifteen_numeric_conversion_opcodes_keep_their_source_and_result_types() {
        let operations = decoded(&[
            0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x92,
            0x93, 0xb1,
        ]);
        for (bci, source, target) in [
            (0, Type::Int, Type::Long),
            (1, Type::Int, Type::Float),
            (2, Type::Int, Type::Double),
            (3, Type::Long, Type::Int),
            (4, Type::Long, Type::Float),
            (5, Type::Long, Type::Double),
            (6, Type::Float, Type::Int),
            (7, Type::Float, Type::Long),
            (8, Type::Float, Type::Double),
            (9, Type::Double, Type::Int),
            (10, Type::Double, Type::Long),
            (11, Type::Double, Type::Float),
            (12, Type::Int, Type::Byte),
            (13, Type::Int, Type::Char),
            (14, Type::Int, Type::Short),
        ] {
            assert_eq!(
                at(&operations, bci),
                Operation::PrimitiveConversion { source, target },
                "conversion at BCI {bci} retains its opcode-defined source and target"
            );
        }
        assert_eq!(at(&operations, 15), Operation::Return);
    }

    #[test]
    fn the_array_accesses_are_the_facts_their_own_opcodes_state() {
        // aload_1; iaload; laload; faload; daload; aaload; baload; caload; saload; iastore;
        // lastore; fastore; dastore; aastore; bastore; castore; sastore; arraylength; return
        //
        // One instruction per fact, in opcode order, so the table below reads as the classification
        // itself: the four int-sized reads carry **no** element (JVMS 2.11.1 gives the five primitives
        // one value shape and one slot, so the opcode cannot state which the array holds), the three
        // wide ones state theirs, and `aaload`/`aastore` state none because a reference's type is a
        // fact of the array.
        let operations = decoded(&[
            0x2b, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x4f, 0x50, 0x51, 0x52, 0x53,
            0x54, 0x55, 0x56, 0xbe, 0xb1,
        ]);
        assert_eq!(
            at(&operations, 1),
            Operation::ArrayLoad,
            "`iaload` is the read the enum-switch rule names (P3 2.3)"
        );
        for bci in [6, 7, 8] {
            assert_eq!(
                at(&operations, bci),
                Operation::ArrayElementLoad {
                    element: Some(Type::Int)
                },
                "an int-sized read at BCI {bci} states the one shape its family has"
            );
        }
        assert_eq!(
            at(&operations, 2),
            Operation::ArrayElementLoad {
                element: Some(Type::Long)
            }
        );
        assert_eq!(
            at(&operations, 3),
            Operation::ArrayElementLoad {
                element: Some(Type::Float)
            }
        );
        assert_eq!(
            at(&operations, 4),
            Operation::ArrayElementLoad {
                element: Some(Type::Double)
            }
        );
        assert_eq!(
            at(&operations, 5),
            Operation::ArrayElementLoad { element: None },
            "`aaload`'s element is a fact of the array's own type, never of the instruction"
        );
        for bci in [9, 14, 15, 16] {
            assert_eq!(
                at(&operations, bci),
                Operation::ArrayStore {
                    element: Some(Type::Int)
                },
                "an int-sized write at BCI {bci} states the one shape its element family has"
            );
        }
        assert_eq!(
            at(&operations, 10),
            Operation::ArrayStore {
                element: Some(Type::Long)
            }
        );
        assert_eq!(
            at(&operations, 11),
            Operation::ArrayStore {
                element: Some(Type::Float)
            }
        );
        assert_eq!(
            at(&operations, 12),
            Operation::ArrayStore {
                element: Some(Type::Double)
            }
        );
        assert_eq!(
            at(&operations, 13),
            Operation::ArrayStore { element: None },
            "`aastore`'s element is a fact of the array's own type"
        );
        assert_eq!(at(&operations, 17), Operation::ArrayLength);
    }

    #[test]
    fn a_newarray_creation_states_the_atype_code_as_an_element_type() {
        // iconst_3; newarray int; newarray boolean; return — the code names the element, and the
        // instruction reads exactly one length for the one dimension it allocates.
        let operations = decoded(&[0x06, 0xbc, 0x0a, 0xbc, 0x04, 0xb1]);
        assert_eq!(
            at(&operations, 1),
            Operation::NewArray {
                element: Type::Int,
                dimensions: 1,
                total_dimensions: 1,
            }
        );
        assert_eq!(
            at(&operations, 3),
            Operation::NewArray {
                element: Type::Boolean,
                dimensions: 1,
                total_dimensions: 1,
            }
        );

        // A code outside 4..=11 is part of the instruction's own encoding (JVMS 6.5), so the
        // instruction does not decode at all: the reader stops there, the reliable prefix keeps the
        // instructions before it, and this layer has *no operation* for that BCI — which is the
        // stated gap that names the BCI rather than an element type nothing stated (P3 2b).
        let unknown = decoded(&[0x06, 0xbc, 0x03, 0xb1]);
        assert_eq!(unknown.get(1), None, "no operation for an undecodable code");
        assert_eq!(unknown.get(5), None, "the decode stops at it");
        assert_eq!(at(&unknown, 0), Operation::Push(ConstantValue::Int(3)));
    }

    #[test]
    fn the_creation_instructions_take_their_arrays_from_their_own_operands() {
        // `anewarray java/lang/Runnable` (pool entry 11): one dimension of the class the pool names.
        let operations = decoded(&[0x06, 0xbd, 0x00, 0x0b, 0xb1]);
        assert_eq!(
            at(&operations, 1),
            Operation::NewArray {
                element: Type::Reference("java.lang.Runnable".to_string()),
                dimensions: 1,
                total_dimensions: 1,
            }
        );

        // `anewarray [[I` (pool entry 9): the class is itself an array, so this allocates one outer
        // dimension over a two-dimensional component.
        let array_class = decoded(&[0x06, 0xbd, 0x00, 0x09, 0xb1]);
        assert_eq!(
            at(&array_class, 1),
            Operation::NewArray {
                element: Type::Int,
                dimensions: 1,
                total_dimensions: 3,
            }
        );

        // `multianewarray [[I, 2`: the descriptor's rank is the complete result, and the element is
        // the component the descriptor names.
        let full = decoded(&[0x06, 0x07, 0xc5, 0x00, 0x09, 0x02, 0xb1]);
        assert_eq!(
            at(&full, 2),
            Operation::NewArray {
                element: Type::Int,
                dimensions: 2,
                total_dimensions: 2,
            }
        );

        // The same class with one dimension allocated: `new int[n][]` keeps the unallocated suffix.
        let prefix = decoded(&[0x06, 0xc5, 0x00, 0x09, 0x01, 0xb1]);
        assert_eq!(
            at(&prefix, 1),
            Operation::NewArray {
                element: Type::Int,
                dimensions: 1,
                total_dimensions: 2,
            }
        );

        // A zero count or a count beyond the descriptor rank is not a proved creation shape.
        let zero = decoded(&[0x06, 0xc5, 0x00, 0x09, 0x00, 0xb1]);
        assert_eq!(at(&zero, 1), Operation::Other);
        let too_many = decoded(&[0x06, 0x07, 0x08, 0xc5, 0x00, 0x09, 0x03, 0xb1]);
        assert_eq!(at(&too_many, 3), Operation::Other);
    }

    #[test]
    fn a_switch_states_its_keys_and_the_bci_each_key_transfers_to() {
        // iconst_0; istore_1; iload_1; tableswitch { 0 → 27, 1 → 27, default → 31 }; …
        //
        // Two keys that share one target — the shape a `case 0: case 1:` falls through as — and a
        // default of its own: the grouping a Java `switch` needs is exactly what the decode states.
        let operations = decoded(&[
            0x03, // 0: iconst_0
            0x3c, // 1: istore_1
            0x1b, // 2: iload_1
            0xaa, // 3: tableswitch (its operands start at 4, already aligned: no padding)
            0x00, 0x00, 0x00, 0x19, // 4: default → +25 = 28
            0x00, 0x00, 0x00, 0x00, // 8: low = 0
            0x00, 0x00, 0x00, 0x01, // 12: high = 1
            0x00, 0x00, 0x00, 0x15, // 16: key 0 → +21 = 24
            0x00, 0x00, 0x00, 0x15, // 20: key 1 → +21 = 24
            0x03, 0x3d, // 24: iconst_0; istore_2
            0x05, 0x3d, // 26: iconst_2; istore_2
            0xb1, // 28: return
        ]);
        match at(&operations, 3) {
            Operation::Switch { cases, default } => {
                assert_eq!(cases, vec![(0, 24), (1, 24)], "two keys, one shared target");
                assert_eq!(default, 28, "the default is the offset the payload states");
            }
            other => panic!("expected the switch, got {other:?}"),
        }
    }

    #[test]
    fn the_pool_entries_the_body_names_are_the_ones_it_reads() {
        let bytes = test_class::single_method(52, 8, 2, &[0xb1]);
        let mut budget = Budget::new(Limits {
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        let header = class_facts(&bytes, &mut budget).expect("the assembled class is a class file");
        let owner = cp_class_name(&header.constant_pool, 11).expect("entry 11 is a class name");
        assert_eq!(
            &owner.0, b"java/lang/Runnable",
            "the fixture's own pool, not a name written into the test"
        );
    }

    /// One pool fact's bytes, as the decode reads them.
    fn text(bytes: &[u8]) -> String {
        lossy(&JvmBytes(bytes.to_vec()))
    }

    #[test]
    fn a_nul_decodes_to_the_one_nul_its_modified_utf8_bytes_state() {
        // `"a\u0000b"` as a class file holds it: one NUL, in the two-byte form this encoding gives
        // it. The standard-UTF-8 read this replaces refused the overlong form and made two
        // replacement characters of it — the NUL the class file states was lost.
        let bytes = &[0x61, 0xc0, 0x80, 0x62];
        assert_eq!(text(bytes), "a\u{0}b", "one NUL, not two U+FFFD");
        assert_eq!(text(bytes).chars().count(), 3);
        assert_eq!(text(bytes).len(), 3);
        assert_eq!(text(bytes).matches('\u{0}').count(), 1);
        assert_eq!(text(&[0xc0, 0x80]), "\u{0}", "`C0 80` is U+0000 by itself");
    }

    #[test]
    fn ordinary_text_survives_the_modified_utf8_read_unchanged() {
        // `正在` is two ordinary three-byte sequences and ASCII stays ASCII: neither goes through a
        // replacement, and neither is re-encoded into a different spelling.
        assert_eq!(text("正在".as_bytes()), "正在");
        assert_eq!(text(b"a/b/C.class"), "a/b/C.class");
        assert_eq!(text(&[]), "");
    }

    #[test]
    fn a_surrogate_pair_is_one_scalar_and_a_lone_half_is_one_replacement() {
        // U+1F600 as a class file holds it: the two halves of its pair, each in its own three-byte
        // form.
        assert_eq!(text(&[0xed, 0xa0, 0xbd, 0xed, 0xb8, 0x80]), "😀");
        assert_eq!(text(&[0xed, 0xa0, 0xbd]), "\u{fffd}", "a lone high half");
        assert_eq!(text(&[0xed, 0xb8, 0x80]), "\u{fffd}", "a lone low half");
    }

    #[test]
    fn a_raw_zero_byte_is_one_replacement_and_never_a_character() {
        // A class file writes U+0000 as `C0 80`, so a raw `00` is not a character in this encoding:
        // it is one replacement, not the NUL a standard reader would see in it.
        assert_eq!(text(&[0x00]), "\u{fffd}");
        assert_eq!(text(&[0x61, 0x00, 0x62]), "a\u{fffd}b");
    }

    #[test]
    fn every_form_this_encoding_does_not_have_is_one_replacement_per_sequence() {
        // The overlong two-byte form of a character a one-byte form already carries: one replacement
        // for the whole sequence, never the character it spells.
        assert_eq!(text(&[0xc1, 0x81]), "\u{fffd}");
        // The same overlong rule below U+0800 in the three-byte form, which also carries U+0000.
        assert_eq!(text(&[0xe0, 0x80, 0x80]), "\u{fffd}");
        // A four-byte UTF-8 form: this encoding has no such shape, so it is not read as the astral
        // scalar a standard reader would make of it.
        assert_eq!(text(&[0xf0, 0x9f, 0x98, 0x80]), "\u{fffd}");
        // A truncated sequence is one replacement for the prefix that is there, and the read goes on
        // with the byte that ended it.
        assert_eq!(text(&[0xe4, 0xb8]), "\u{fffd}");
        assert_eq!(text(&[0xc0, 0x41]), "\u{fffd}A");
        // A continuation byte where a lead belongs is not a sequence of its own.
        assert_eq!(text(&[0x80, 0x41]), "\u{fffd}A");
    }
}
