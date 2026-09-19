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
//! instruction). Everything else — an array operation, a conversion this subset has no operator
//! for, `athrow`, `jsr`, a `ret` — is [`Operation::Other`], which is a *stated* input: the
//! statement it belongs to becomes a fallback with a diagnostic, never a guess.
//!
//! The P3 2.2 slice models four more of them, and each one is a fact a *pattern rule* reads rather
//! than a presentation this layer writes: the allocation and the duplication a concatenation chain
//! starts with (`new`/`dup`), the field access a synthetic accessor's body reads or writes
//! (`getfield`/`putfield`/`getstatic`/`putstatic`, with the member the pool names), and the cast a
//! bridge applies to the value it forwards (`checkcast`, with the class the pool names). A caller
//! cannot state any of the four — they are read from the same decode as everything else — and
//! nothing below [`crate::build`] decides whether any of them is presented.
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
    CpEntryFacts, CpEntryKind, ImmediateValue, InstructionFact, InstructionOperands,
    MethodCodeFacts, SwitchOperands, cp_entry,
};

use crate::facts::{
    ArithmeticOp, CallTarget, CompareOp, ConstantValue, DynamicSite, FieldAccess, InvokeKind,
    Operation,
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
    #[cfg(test)]
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
        0xc0 => check_cast(instruction, operands, pool),
        0xac..=0xb1 => Operation::Return,
        _ => Operation::Other,
    }
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
        // A `float`/`double`/`Class`/`MethodType` constant has no literal this subset writes, and
        // an index that resolves to nothing is not a constant this run can name.
        _ => Operation::Other,
    }
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
        })
        | Ok(CpEntryKind::InterfaceMethodRef {
            owner,
            name,
            descriptor,
            ..
        }) => Operation::Invoke(CallTarget::new(
            kind,
            lossy(owner),
            lossy(name),
            lossy(descriptor),
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

/// The text of one pool fact, as Modified UTF-8 read back.
///
/// Modified UTF-8 is not valid UTF-8 for every payload (a supplementary character is encoded as a
/// surrogate pair), so this is a lossy read: a name that is not valid UTF-8 is evidence this layer
/// can state but cannot spell byte for byte, and the presentation is marked `NotJava` rather than
/// carrying a byte sequence Java cannot hold in a string.
fn lossy(bytes: &jarde_reader::model::JvmBytes) -> String {
    String::from_utf8_lossy(&bytes.0).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    use jarde_reader::budget::{Budget, Limits};
    use jarde_reader::classfile::{class_facts, cp_class_name, method_code_facts, test_class};

    /// The operations of one assembled body, read exactly the way a recovery run reads them: a real
    /// class file, the reader's own decode, and the pool of the same read.
    fn decoded(code: &[u8]) -> Operations {
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
        let member = header
            .methods
            .iter()
            .find(|member| member.name.raw().0 == b"method")
            .expect("the assembled class declares `method`");
        let facts =
            method_code_facts(&bytes, member, &mut budget).expect("the assembled body decodes");
        Operations::of(&facts, &header.constant_pool)
    }

    fn at(operations: &Operations, bci: u32) -> Operation {
        operations
            .get(bci)
            .unwrap_or_else(|| panic!("the body decodes an instruction at BCI {bci}"))
            .clone()
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
            }
            other => panic!("expected the invocation, got {other:?}"),
        }
        assert_eq!(at(&operations, 3), Operation::Push(ConstantValue::Long(0)));
    }

    #[test]
    fn an_opcode_this_subset_does_not_model_is_stated_as_other() {
        // aconst_null; astore_1; aload_1; arraylength; istore_2; return
        let operations = decoded(&[0x01, 0x4c, 0x2b, 0xbe, 0x3d, 0xb1]);
        assert_eq!(
            at(&operations, 3),
            Operation::Other,
            "`arraylength` is not modelled: stated, never guessed"
        );
        assert_eq!(at(&operations, 4), Operation::Store { slot: 2 });
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
}
