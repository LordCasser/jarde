//! ③→④ The AST builder: turns the region tree plus the IR's value flow plus the decoded facts into
//! [`Stmt`]s, without writing a single byte of text.
//!
//! # The rule that keeps this layer honest
//!
//! A statement is built only where every piece of its text is *evidence*:
//!
//! * the shape comes from the region ([`crate::region`]) — a straight run, an `if`, or a fallback;
//! * the operands come from the IR and the decoded operations: a value whose definition is an
//!   instruction is rendered from that instruction's [`Operation`], a value described by a frame
//!   entry or a phi of a local slot is rendered as that local's name (both arms assigned it, so the
//!   join reads it by name), and anything else — a caught exception reference, a stack value no
//!   instruction produced, an operation this subset does not model, a receiver that cannot be
//!   rendered — makes the statement it belongs to a [`StmtKind::Fallback`] that quotes its bytecode.
//!
//! Every `None`, `Err` and `fallback` below is therefore a *stated* one: the artifact says "this BCI
//! is bytecode this layer cannot present" instead of printing a guess, and the report counts it (a
//! run with any fallback is `Mixed`/`Fallback`, never `Java`/`Structured`).
//!
//! # Which instruction becomes a statement
//!
//! * [`Operation::Store`], [`Operation::Invoke`] and [`Operation::Return`] are statements: they are
//!   what the bytecode does to the program's state.
//! * [`Operation::Push`], [`Operation::Load`] and [`Operation::Arithmetic`] are not: they produce a
//!   value whose text lands where the value is consumed, and a load whose value nobody consumes has
//!   no Java effect either.
//! * [`Operation::Comparison`] is not: the branch that tests it prints it — as an `if`'s condition
//!   (the *negation* of the jump sense, because a branch transfers only when its sense holds), or as
//!   part of the quoted bytecode when the region could not be structured.
//! * [`Operation::Other`], and an instruction this run decoded no operation for, are *stated* as
//!   unusable: neither is dropped silently, because a silently dropped instruction is exactly the
//!   failure a presentation must not have.

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalCfg, Definition, RefType, Slot, SsaInstruction, SsaTable, Value,
    ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};

use crate::ast::{BinaryOp, Expr, ExprKind, Stmt, StmtKind, SwitchArm, Type};
use crate::decode::Operations;
use crate::facts::{ArithmeticOp, CallTarget, CompareOp, ConstantValue, InvokeKind, Operation};
use crate::names::NameTable;
use crate::region::{Continuation, LoopForm, Region};
use crate::source_map::{Origin, OriginSet};
use crate::stop::{StopReason, charge, poll};

/// How deep a value expression may nest before the builder refuses it.
///
/// Bytecode nests as deeply as the source expression did, and a source expression is bounded by the
/// source; the bound exists so that a pathological body cannot make this layer recurse without
/// limit. Twenty-four levels is far above what the corpus holds and far below a stack this process
/// cannot afford.
const MAX_VALUE_DEPTH: usize = 24;

/// The statements of one method, in method order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Program {
    /// The body, in method order.
    pub(crate) stmts: Vec<Stmt>,
    /// How many statements the build produced.
    pub(crate) statements: usize,
    /// Whether any statement is a fallback: the run's representation and quality read this.
    pub(crate) ragged: bool,
}

/// Builds the statements of one method from its regions.
pub(crate) fn build(
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    operations: &Operations,
    parameters: u16,
    names: &NameTable,
    regions: &[Region],
    budget: &mut Budget,
) -> Result<Program, StopReason> {
    let mut instructions: BTreeMap<u32, &SsaInstruction> = BTreeMap::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            instructions.insert(instruction.bci(), instruction);
        }
    }
    let mut builder = Builder {
        canonical,
        ssa,
        operations,
        parameters,
        names,
        instructions,
        budget,
        declared: BTreeSet::new(),
        stmts: Vec::new(),
        statements: 0,
        ragged: false,
    };
    for region in regions {
        builder.region(region)?;
    }
    Ok(Program {
        statements: builder.statements,
        ragged: builder.ragged,
        stmts: builder.stmts,
    })
}

struct Builder<'a> {
    canonical: &'a CanonicalCfg,
    ssa: &'a SsaTable,
    operations: &'a Operations,
    /// How many local slots the method's parameters occupy, `this` included when the caller
    /// counted it: the slots below this one are declared by the signature, not by the body.
    parameters: u16,
    names: &'a NameTable,
    instructions: BTreeMap<u32, &'a SsaInstruction>,
    budget: &'a mut Budget,
    declared: BTreeSet<u16>,
    stmts: Vec<Stmt>,
    statements: usize,
    ragged: bool,
}

impl Builder<'_> {
    /// Appends the statements of one region.
    fn region(&mut self, region: &Region) -> Result<(), StopReason> {
        match region {
            Region::Straight { blocks } => {
                for block in blocks {
                    self.block(block)?;
                }
                Ok(())
            }
            Region::If {
                prefix,
                branch,
                branch_bci,
                then_arm,
                else_arm,
                ..
            } => {
                for block in prefix {
                    self.block(block)?;
                }
                // Everything the test block does *before* its branch ran before the branch in the
                // bytecode too, so it is written here, in order: a store or a call the test block
                // makes is a statement of its own, and it must not be dropped just because the
                // branch that follows it is what becomes the `if`.
                self.test_effects(branch, *branch_bci)?;
                let cond = match self.test_expr(*branch_bci, false) {
                    Ok(cond) => cond,
                    Err(reason) => {
                        return self.fallback(vec![*branch_bci], &reason, *branch_bci);
                    }
                };
                // The condition's text is tested *by* the branch: the `if` statement is the branch,
                // and the condition node presents it. One BCI reaching two nodes with the two
                // provenances is exactly what the segment table records as it writes.
                let cond = cond.derived_from(*branch_bci);
                let mut then_body = Vec::new();
                self.arm(then_arm, &mut then_body)?;
                let mut else_body = Vec::new();
                self.arm(else_arm, &mut else_body)?;
                self.push(Stmt::new(
                    StmtKind::If {
                        cond,
                        then_body,
                        else_body,
                    },
                    OriginSet::new(Origin::direct(*branch_bci)),
                ))
            }
            Region::Switch {
                prefix,
                branch,
                branch_bci,
                groups,
                ..
            } => {
                for block in prefix {
                    self.block(block)?;
                }
                // As for an `if`: what the switch block does before its selector is read runs
                // before the switch in the bytecode too.
                self.test_effects(branch, *branch_bci)?;
                let Some(instruction) = self.instructions.get(branch_bci).copied() else {
                    return self.fallback(
                        vec![*branch_bci],
                        &format!("no names record for the switch at BCI {branch_bci}"),
                        *branch_bci,
                    );
                };
                let Some((_, value)) = stack_operands(instruction).last().copied() else {
                    return self.fallback(
                        vec![*branch_bci],
                        &format!("the switch at BCI {branch_bci} reads no value to select on"),
                        *branch_bci,
                    );
                };
                // The selector is a value the switch reads; the text that renders it is anchored
                // where the value was produced, and the switch that reads it is added as a
                // presented anchor, exactly like a branch's condition.
                let value = match self.render_value(value, *branch_bci, 0) {
                    Ok(value) => value.derived_from(*branch_bci),
                    Err(reason) => return self.fallback(vec![*branch_bci], &reason, *branch_bci),
                };
                let mut arms = Vec::with_capacity(groups.len());
                for group in groups {
                    let mut body = Vec::new();
                    self.arm(&group.arm, &mut body)?;
                    arms.push(SwitchArm {
                        keys: group.keys.clone(),
                        default: group.default,
                        body,
                    });
                }
                self.push(Stmt::new(
                    StmtKind::Switch { value, arms },
                    OriginSet::new(Origin::direct(*branch_bci)),
                ))
            }
            Region::Loop {
                test_bci,
                form,
                continuation,
                body,
                ..
            } => {
                // The test is written *inside* the loop statement, so the values it reads and the
                // calls it makes run once per evaluation — the loop's own count, not the count of a
                // hoisted copy. Which of the two senses continues the loop is a decode fact
                // (`Continuation`), and the condition is that sense, not its negation.
                let taken = *continuation == Continuation::Taken;
                let cond = match self.test_expr(*test_bci, taken) {
                    Ok(cond) => cond,
                    Err(reason) => {
                        return self.fallback(vec![*test_bci], &reason, *test_bci);
                    }
                };
                let cond = cond.derived_from(*test_bci);
                let mut loop_body = Vec::new();
                self.arm(body, &mut loop_body)?;
                let kind = match form {
                    LoopForm::While => StmtKind::While {
                        cond,
                        body: loop_body,
                    },
                    LoopForm::DoWhile => StmtKind::DoWhile {
                        cond,
                        body: loop_body,
                    },
                };
                self.push(Stmt::new(kind, OriginSet::new(Origin::direct(*test_bci))))
            }
            Region::Fallback { blocks, reason } => {
                let bcis: Vec<u32> = blocks
                    .iter()
                    .flat_map(|block| self.covered_bcis(block))
                    .collect();
                let at = bcis.first().copied().unwrap_or(0);
                self.fallback(bcis, &reason.message(), at)
            }
        }
    }

    /// Writes what a test block does before the test itself, in the order it does it.
    ///
    /// A block whose terminal instruction is a branch can still hold statements before it — an
    /// assignment made while the condition is evaluated, a call whose result the test reads. For an
    /// `if` or a `switch` those run exactly once, before the test, so writing them there keeps their
    /// count and their order. (A loop's test block is the one place this cannot be done: its
    /// statements would run once per iteration, and a `while (…)` has nowhere to write them that
    /// does — which is why [`crate::region`] refuses such a loop instead of hoisting them.)
    fn test_effects(&mut self, block: &CanonicalBlockId, test_bci: u32) -> Result<(), StopReason> {
        let Some(names) = self.ssa.block(block) else {
            return Ok(());
        };
        let instructions: Vec<SsaInstruction> = names
            .instructions()
            .iter()
            .filter(|instruction| instruction.bci() != test_bci)
            .cloned()
            .collect();
        for instruction in &instructions {
            self.instruction(instruction)?;
        }
        Ok(())
    }

    /// The condition one branch's test states, as the structure that holds it writes it.
    ///
    /// `taken` says which sense of the branch the structure continues on: an `if` always writes the
    /// fall-through condition (the branch *leaves* the `if` when its sense holds), a loop writes the
    /// sense that iterates. Both are the same decode fact read two ways.
    fn test_expr(&self, branch_bci: u32, taken: bool) -> Result<Expr, String> {
        let Some((op, _)) = self
            .operations
            .get(branch_bci)
            .and_then(Operation::comparison)
        else {
            return Err(format!(
                "the branch at BCI {branch_bci} has no decoded sense, so its condition cannot be written"
            ));
        };
        let Some(instruction) = self.instructions.get(&branch_bci).copied() else {
            return Err(format!(
                "no names record for the branch at BCI {branch_bci}"
            ));
        };
        condition(op, &stack_operands(instruction), self, branch_bci, taken)
    }

    /// Appends one arm's statements to a vector of its own, so the `if` can hold them.
    fn arm(&mut self, region: &Region, into: &mut Vec<Stmt>) -> Result<(), StopReason> {
        let outer = std::mem::take(&mut self.stmts);
        self.region(region)?;
        let arm = std::mem::replace(&mut self.stmts, outer);
        into.extend(arm);
        Ok(())
    }

    /// Appends the statements of one block, in BCI order.
    fn block(&mut self, block: &CanonicalBlockId) -> Result<(), StopReason> {
        let Some(names) = self.ssa.block(block) else {
            let bcis = self.covered_bcis(block);
            let at = bcis.first().copied().unwrap_or(block.bci());
            return self.fallback(
                bcis,
                &format!("no names record for the block at BCI {}", block.bci()),
                at,
            );
        };
        let instructions: Vec<SsaInstruction> = names.instructions().to_vec();
        for instruction in &instructions {
            self.instruction(instruction)?;
        }
        Ok(())
    }

    /// Appends the statement one instruction became, if it became one.
    fn instruction(&mut self, instruction: &SsaInstruction) -> Result<(), StopReason> {
        poll(self.budget, Some(instruction.bci()))?;
        let at = instruction.bci();
        let write = instruction
            .writes()
            .iter()
            .find_map(|(slot, value)| match slot {
                Slot::Local(slot) => Some((*slot, *value)),
                Slot::Stack(_) => None,
            });
        match self.operations.get(at) {
            Some(Operation::Store { .. }) => {
                let Some((slot, written)) = write else {
                    return self.fallback(
                        vec![at],
                        &format!("the store at BCI {at} writes no local slot this run names"),
                        at,
                    );
                };
                let target = match self.names.text(slot) {
                    Some(name) => name.to_string(),
                    None => {
                        return self.fallback(
                            vec![at],
                            &format!(
                                "the store at BCI {at} writes local {slot}, which has no name"
                            ),
                            at,
                        );
                    }
                };
                let Some((_, value)) = stack_operands(instruction).last().copied() else {
                    return self.fallback(
                        vec![at],
                        &format!("the store at BCI {at} reads no value to store"),
                        at,
                    );
                };
                let value = match self.render_value(value, at, 0) {
                    Ok(value) => value,
                    Err(reason) => return self.fallback(vec![at], &reason, at),
                };
                match self.declare(slot, written, at)? {
                    Some(ty) => self.push(Stmt::new(
                        StmtKind::Declare {
                            ty,
                            name: target,
                            value: Some(value),
                        },
                        OriginSet::new(Origin::direct(at)),
                    )),
                    None => self.push(Stmt::new(
                        StmtKind::Assign {
                            name: target,
                            value,
                        },
                        OriginSet::new(Origin::direct(at)),
                    )),
                }
            }
            Some(Operation::Invoke(target)) => {
                let call = match self.call_expr(at, instruction, target) {
                    Ok(call) => call,
                    Err(reason) => return self.fallback(vec![at], &reason, at),
                };
                let Some((slot, written)) = write else {
                    // No local slot takes the result: the call is a statement of its own.
                    return self.push(Stmt::new(
                        StmtKind::Expr(call),
                        OriginSet::new(Origin::direct(at)),
                    ));
                };
                let Some(target_name) = self.names.text(slot).map(str::to_owned) else {
                    return self.fallback(
                        vec![at],
                        &format!("the call at BCI {at} writes local {slot}, which has no name"),
                        at,
                    );
                };
                match self.declare(slot, written, at)? {
                    Some(ty) => self.push(Stmt::new(
                        StmtKind::Declare {
                            ty,
                            name: target_name,
                            value: Some(call),
                        },
                        OriginSet::new(Origin::direct(at)),
                    )),
                    None => self.push(Stmt::new(
                        StmtKind::Assign {
                            name: target_name,
                            value: call,
                        },
                        OriginSet::new(Origin::direct(at)),
                    )),
                }
            }
            Some(Operation::Return) => {
                let value = match stack_operands(instruction).last().copied() {
                    Some((_, value)) => match self.render_value(value, at, 0) {
                        Ok(value) => Some(value),
                        Err(reason) => return self.fallback(vec![at], &reason, at),
                    },
                    None => None,
                };
                self.push(Stmt::new(
                    StmtKind::Return { value },
                    OriginSet::new(Origin::direct(at)),
                ))
            }
            Some(Operation::Increment { slot, amount }) => {
                let Some(target_name) = self.names.text(*slot).map(str::to_owned) else {
                    return self.fallback(
                        vec![at],
                        &format!(
                            "the increment at BCI {at} writes local {slot}, which has no name"
                        ),
                        at,
                    );
                };
                let (op, magnitude) = if *amount < 0 {
                    (BinaryOp::Subtract, -i64::from(*amount))
                } else {
                    (BinaryOp::Add, i64::from(*amount))
                };
                let value = Expr::direct(
                    ExprKind::Binary {
                        op,
                        left: Box::new(Expr::direct(ExprKind::Local(target_name.clone()), at)),
                        right: Box::new(Expr::direct(ExprKind::Integer(magnitude), at)),
                    },
                    at,
                );
                self.push(Stmt::new(
                    StmtKind::Assign {
                        name: target_name,
                        value,
                    },
                    OriginSet::new(Origin::direct(at)),
                ))
            }
            // Values whose text lands where they are consumed, the branches and switches the region
            // layer turned into `if`/`while`/`switch`, and the transfer the region's edge already
            // stands for: no statement of their own.
            Some(
                Operation::Push(_)
                | Operation::Load { .. }
                | Operation::Arithmetic { .. }
                | Operation::Comparison { .. }
                | Operation::Switch { .. }
                | Operation::Transfer,
            ) => Ok(()),
            Some(Operation::Other) | None => self.fallback(
                vec![at],
                &format!("the instruction at BCI {at} is not part of the provable subset"),
                at,
            ),
        }
    }

    /// Whether a local slot has to be declared at this write, and with which type.
    ///
    /// The type comes from the value being written, not from the frame's entry state for the slot: at
    /// the block where a local is first written the frame still says `Top` for it — nothing has
    /// written it yet — so the frame cannot state a type here, while the value that is about to fill
    /// the slot can, and is the same evidence the source's declaration was read from.
    ///
    /// `Ok(None)` means "no declaration is due": the slot is a parameter (its declaration is the
    /// method's signature) or it was declared at an earlier write in this run.
    fn declare(
        &mut self,
        slot: u16,
        written: ValueId,
        at: u32,
    ) -> Result<Option<Type>, StopReason> {
        if self.declared.contains(&slot) || slot < self.parameters {
            return Ok(None);
        }
        if self.names.text(slot).is_none() {
            // No name to declare: the assignment that follows states the same thing and is already
            // reported as a fallback of its own.
            return Ok(None);
        }
        let Some(ty) = value_type(self.ssa.value(written).ty()) else {
            self.fallback(
                vec![at],
                &format!("local {slot} has no frame entry stating its type"),
                at,
            )?;
            return Ok(None);
        };
        self.declared.insert(slot);
        Ok(Some(ty))
    }

    /// Renders one SSA value as an expression.
    fn render_value(&self, value: ValueId, at: u32, depth: usize) -> Result<Expr, String> {
        if depth > MAX_VALUE_DEPTH {
            return Err(format!(
                "the value at BCI {at} nests deeper than this layer renders"
            ));
        }
        match self.ssa.value(value).def() {
            Definition::Entry { slot, .. } | Definition::Phi { slot, .. } => match slot {
                Slot::Local(slot) => match self.names.text(*slot) {
                    Some(name) => Ok(Expr::direct(ExprKind::Local(name.to_string()), at)),
                    None => Err(format!("local {slot} has no name to write")),
                },
                Slot::Stack(stack) => Err(format!(
                    "the value at BCI {at} is the entry state of stack depth {stack}, which no instruction produced"
                )),
            },
            Definition::Instruction { bci, .. } => {
                let bci = *bci;
                let Some(operation) = self.operations.get(bci) else {
                    return Err(format!(
                        "the value at BCI {at} comes from BCI {bci}, whose operation this run did not decode"
                    ));
                };
                match operation {
                    Operation::Push(constant) => Ok(Expr::direct(literal(constant), bci)),
                    Operation::Load { slot } => match self.names.text(*slot) {
                        Some(name) => Ok(Expr::direct(ExprKind::Local(name.to_string()), bci)),
                        None => Err(format!("local {slot} has no name to write")),
                    },
                    Operation::Arithmetic { op } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!(
                                "no names record for the instruction at BCI {bci}"
                            ));
                        };
                        let operands = stack_operands(instruction);
                        if operands.len() != 2 {
                            return Err(format!(
                                "the arithmetic at BCI {bci} reads {} values, not two",
                                operands.len()
                            ));
                        }
                        let left = self.render_value(operands[0].1, bci, depth + 1)?;
                        let right = self.render_value(operands[1].1, bci, depth + 1)?;
                        Ok(Expr::direct(
                            ExprKind::Binary {
                                op: arithmetic_op(*op),
                                left: Box::new(left),
                                right: Box::new(right),
                            },
                            bci,
                        ))
                    }
                    Operation::Invoke(target) => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the call at BCI {bci}"));
                        };
                        self.call_expr(bci, instruction, target)
                    }
                    other => Err(format!(
                        "the value at BCI {at} comes from an {other:?} at BCI {bci}, which produces no expression this subset writes"
                    )),
                }
            }
            Definition::Caught { bci, .. } => Err(format!(
                "the value at BCI {at} is the exception reference of the throw site at BCI {bci}"
            )),
        }
    }

    /// Renders one invocation, with its receiver and its arguments.
    fn call_expr(
        &self,
        bci: u32,
        instruction: &SsaInstruction,
        target: &CallTarget,
    ) -> Result<Expr, String> {
        let operands = stack_operands(instruction);
        let (receiver, args) = match target.kind() {
            InvokeKind::Static => (None, operands.as_slice()),
            _ => {
                let (receiver, args) = operands
                    .split_first()
                    .ok_or_else(|| format!("the call at BCI {bci} reads no receiver"))?;
                // The receiver is a value the call reads; when the subset cannot render it (an
                // uninitialized `new`, an operation it does not model) the call falls back rather
                // than naming the owner type in its place.
                (Some(Box::new(self.render_value(receiver.1, bci, 0)?)), args)
            }
        };
        let mut arguments = Vec::with_capacity(args.len());
        for (_, value) in args {
            arguments.push(self.render_value(*value, bci, 0)?);
        }
        Ok(Expr::direct(
            ExprKind::Call {
                receiver,
                name: target.name().to_string(),
                args: arguments,
            },
            bci,
        ))
    }

    /// Appends the quoted bytecode of one region or instruction.
    fn fallback(&mut self, bcis: Vec<u32>, reason: &str, at: u32) -> Result<(), StopReason> {
        self.ragged = true;
        self.push(Stmt::new(
            StmtKind::Fallback {
                reason: reason.to_string(),
                bcis,
            },
            OriginSet::new(Origin::direct(at)),
        ))
    }

    /// Bills and appends one statement.
    fn push(&mut self, stmt: Stmt) -> Result<(), StopReason> {
        let at = stmt.origin.primary().bci();
        poll(self.budget, Some(at))?;
        charge(self.budget, CountedBudgetDimension::IrItems, 1, Some(at))?;
        self.statements += 1;
        self.stmts.push(stmt);
        Ok(())
    }

    /// The instruction starts one canonical block covers.
    fn covered_bcis(&self, block: &CanonicalBlockId) -> Vec<u32> {
        self.canonical
            .blocks()
            .iter()
            .find(|candidate| candidate.id() == block)
            .map(|block| block.blocks().to_vec())
            .unwrap_or_default()
    }
}

/// The values one instruction reads off the operand stack, ordered by depth.
///
/// A local read is not an operand: an instruction that loads a local *produces* a value, and reading
/// the local here would print `x = x` for a store that loads the slot it writes. A statement's
/// operands are exactly the stack values the instruction consumes, from the bottom of the consumed
/// region upwards, which is the order a call's receiver and arguments are written in.
fn stack_operands(instruction: &SsaInstruction) -> Vec<(Slot, ValueId)> {
    let mut reads: Vec<(Slot, ValueId)> = instruction
        .reads()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
        .copied()
        .collect();
    reads.sort_by_key(|(slot, _)| match slot {
        Slot::Stack(depth) => *depth,
        Slot::Local(slot) => u32::from(*slot),
    });
    reads
}

/// The expression a constant becomes.
fn literal(constant: &ConstantValue) -> ExprKind {
    match constant {
        ConstantValue::Int(value) => ExprKind::Integer(*value),
        ConstantValue::Long(value) => ExprKind::Long(*value),
        ConstantValue::String(value) => ExprKind::Str(value.clone()),
        ConstantValue::Null => ExprKind::Null,
    }
}

/// The operator an arithmetic operation becomes.
fn arithmetic_op(op: ArithmeticOp) -> BinaryOp {
    match op {
        ArithmeticOp::Add => BinaryOp::Add,
        ArithmeticOp::Subtract => BinaryOp::Subtract,
        ArithmeticOp::Multiply => BinaryOp::Multiply,
        ArithmeticOp::Divide => BinaryOp::Divide,
        ArithmeticOp::Remainder => BinaryOp::Remainder,
    }
}

/// The condition one sense of a branch states, as the structure that holds it writes it.
///
/// `taken = false` writes the **fall-through** condition — the sense negated, which is what an `if`
/// statement tests: a branch transfers only when its sense holds, so control stays in the `if` when
/// it does not. `taken = true` writes the sense itself, which is what a loop tests when the branch's
/// target is the block that iterates. Writing either one here — once, next to the sense's meaning —
/// is what makes an emitted condition a consequence of the decoded fact rather than of the order two
/// successors happened to be published in.
///
/// The condition node is anchored where the value it tests was produced, and the branch that tests it
/// is added as a *presented* anchor by the caller: the text is the value's, the test is the branch's,
/// and keeping those two apart is what makes a shared BCI visible in the segment table with both
/// provenances instead of one node claiming both.
fn condition(
    op: CompareOp,
    operands: &[(Slot, ValueId)],
    builder: &Builder<'_>,
    branch_bci: u32,
    taken: bool,
) -> Result<Expr, String> {
    let expected = if op.reads_two() { 2 } else { 1 };
    if operands.len() != expected {
        return Err(format!(
            "the branch at BCI {branch_bci} reads {} values, but its decoded sense needs {expected}",
            operands.len()
        ));
    }
    let left = builder.render_value(operands[0].1, branch_bci, 0)?;
    let anchor = left.origin.primary().bci();
    // The operator the *sense* states, and the operator its negation states.
    let (positive, negative) = match op {
        CompareOp::JumpIfZero => (Test::Zero(BinaryOp::Equal), Test::Zero(BinaryOp::NotEqual)),
        CompareOp::JumpIfNotZero => (Test::Zero(BinaryOp::NotEqual), Test::Zero(BinaryOp::Equal)),
        CompareOp::JumpIfNegative => (
            Test::Zero(BinaryOp::Less),
            Test::Zero(BinaryOp::GreaterOrEqual),
        ),
        CompareOp::JumpIfNotNegative => (
            Test::Zero(BinaryOp::GreaterOrEqual),
            Test::Zero(BinaryOp::Less),
        ),
        CompareOp::JumpIfPositive => (
            Test::Zero(BinaryOp::Greater),
            Test::Zero(BinaryOp::LessOrEqual),
        ),
        CompareOp::JumpIfNotPositive => (
            Test::Zero(BinaryOp::LessOrEqual),
            Test::Zero(BinaryOp::Greater),
        ),
        CompareOp::JumpIfNull => (Test::Null(BinaryOp::Equal), Test::Null(BinaryOp::NotEqual)),
        CompareOp::JumpIfNotNull => (Test::Null(BinaryOp::NotEqual), Test::Null(BinaryOp::Equal)),
        CompareOp::JumpIfSame => (Test::Pair(BinaryOp::Equal), Test::Pair(BinaryOp::NotEqual)),
        CompareOp::JumpIfDifferent => (Test::Pair(BinaryOp::NotEqual), Test::Pair(BinaryOp::Equal)),
        CompareOp::JumpIfLess => (
            Test::Pair(BinaryOp::Less),
            Test::Pair(BinaryOp::GreaterOrEqual),
        ),
        CompareOp::JumpIfLessOrEqual => (
            Test::Pair(BinaryOp::LessOrEqual),
            Test::Pair(BinaryOp::Greater),
        ),
        CompareOp::JumpIfGreater => (
            Test::Pair(BinaryOp::Greater),
            Test::Pair(BinaryOp::LessOrEqual),
        ),
        CompareOp::JumpIfGreaterOrEqual => (
            Test::Pair(BinaryOp::GreaterOrEqual),
            Test::Pair(BinaryOp::Less),
        ),
    };
    let test = if taken { positive } else { negative };
    match test {
        // The second operand is rendered only where it is written, so a zero or null test does not
        // ask the value flow for a value the instruction never read.
        Test::Zero(op) => Ok(binary(op, left, zero(branch_bci), anchor)),
        Test::Null(op) => Ok(binary(
            op,
            left,
            Expr::direct(ExprKind::Null, branch_bci),
            anchor,
        )),
        Test::Pair(op) => {
            let right = builder.render_value(operands[1].1, branch_bci, 0)?;
            Ok(binary(op, left, right, anchor))
        }
    }
}

/// The comparison one sense of a branch states, before its operands are rendered.
enum Test {
    /// One value against the constant zero, with the operator the sense states.
    Zero(BinaryOp),
    /// One reference against `null`.
    Null(BinaryOp),
    /// The two values the branch reads.
    Pair(BinaryOp),
}

/// A binary expression anchored at one bytecode index.
fn binary(op: BinaryOp, left: Expr, right: Expr, bci: u32) -> Expr {
    Expr::direct(
        ExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
        bci,
    )
}

/// The `0` a zero test compares against, anchored where the test is.
fn zero(bci: u32) -> Expr {
    Expr::direct(ExprKind::Integer(0), bci)
}

/// The Java source spelling of an internal name.
fn source_name(internal: &str) -> String {
    internal.replace('/', ".")
}

/// The declared type of a local, when the frames state one.
fn value_type(value: &Value) -> Option<Type> {
    match value {
        Value::Int => Some(Type::Int),
        Value::Long => Some(Type::Long),
        Value::Float => Some(Type::Float),
        Value::Double => Some(Type::Double),
        Value::Ref(RefType::Named { name, .. }) => {
            Some(Type::Reference(source_name(&String::from_utf8_lossy(name))))
        }
        Value::Ref(_) | Value::Null => Some(Type::Reference("Object".to_string())),
        _ => None,
    }
}
