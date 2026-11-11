//! Admission for a parameter load nested in a return expression. This proof never renders or
//! publishes a binding: the ordinary binding transaction still owns the declaration and name.
use super::*;

#[derive(Clone, Copy)]
struct Pending {
    value: ValueId,
    at: u32,
    depth: usize,
    before: u32,
}

// An observer may cancel on the final charged operation, so poll after the charge too.
fn bill(budget: &mut Budget, dimension: CountedBudgetDimension, at: u32) -> Result<(), StopReason> {
    charge(budget, dimension, 1, Some(at))?;
    poll(budget, Some(at))
}

impl Builder<'_> {
    /// Prove the entire terminal tree, not merely the path from the proposed saved load.
    /// Every other leaf must already be spellable where the return evaluates it. Parameter
    /// Stores are separate evaluation roots, each checked at its own BCI and depth zero.
    pub(super) fn nested_parameter_snapshot(
        &mut self,
        saved: ValueId,
        producer: u32,
        reader: u32,
    ) -> Result<bool, StopReason> {
        bill(self.budget, CountedBudgetDimension::AnalysisSteps, reader)?;
        let Some(block) = self.block_of.get(&producer) else {
            return Ok(false);
        };
        if block.bci() != 0 || block.is_clone() || self.block_of.get(&reader) != Some(block) {
            return Ok(false);
        }
        // Only an original block reaches this clone: its jsr path is empty and allocates nothing.
        let block = block.clone();
        for edge in self.canonical.edges() {
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, reader)?;
            if edge.to() == &block {
                return Ok(false);
            }
        }
        let Some(names) = self.ssa.block(&block) else {
            return Ok(false);
        };
        let Some(terminal) = names.instructions().last() else {
            return Ok(false);
        };
        if terminal.bci() != reader
            || producer >= reader
            || !matches!(self.operations.get(reader), Some(Operation::Return))
            || !matches!(
                (terminal.opcode(), self.return_type.as_ref()),
                (0xac, Some(Type::Int)) | (0xad, Some(Type::Long))
            )
        {
            return Ok(false);
        }
        // Stable indirect loads are common in large arithmetic trees. Exclude them before
        // allocating or walking any expression: this is only a cheap necessary condition,
        // not a substitute for the full identity/write/renderability proof below.
        bill(self.budget, CountedBudgetDimension::AnalysisSteps, producer)?;
        let Some(load) = self.instructions.get(&producer).copied() else {
            return Ok(false);
        };
        let Some(Operation::Load { slot }) = self.operations.get(producer) else {
            return Ok(false);
        };
        let slot = *slot;
        let mut denotes = None;
        for (read_slot, value) in load.reads() {
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, producer)?;
            if *read_slot != Slot::Local(slot) || denotes.replace(*value).is_some() {
                return Ok(false);
            }
        }
        let Some(denotes) = denotes else {
            return Ok(false);
        };
        let mut loaded = None;
        for (written_slot, value) in load.writes() {
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, producer)?;
            if !matches!(written_slot, Slot::Stack(_)) || loaded.replace(*value).is_some() {
                return Ok(false);
            }
        }
        if loaded != Some(saved) {
            return Ok(false);
        }
        let mut exit = None;
        for (exit_slot, value) in names.exit() {
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, reader)?;
            if *exit_slot == Slot::Local(slot) {
                exit = Some(*value);
            }
        }
        if exit.is_none() || exit == Some(denotes) {
            return Ok(false);
        }

        let mut returned = None;
        for (slot, value) in terminal.reads() {
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, reader)?;
            if !matches!(slot, Slot::Stack(_)) || returned.replace(*value).is_some() {
                return Ok(false);
            }
        }
        if !terminal.writes().is_empty() {
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, reader)?;
            return Ok(false);
        }
        let Some(returned) = returned else {
            return Ok(false);
        };
        bill(self.budget, CountedBudgetDimension::AnalysisSteps, reader)?;
        if !matches!(
            (self.ssa.value(returned).ty(), self.return_type.as_ref()),
            (Value::Int, Some(Type::Int)) | (Value::Long, Some(Type::Long))
        ) {
            return Ok(false);
        }
        bill(self.budget, CountedBudgetDimension::IrItems, reader)?;
        let mut pending = vec![Pending {
            value: returned,
            at: reader,
            depth: 0,
            before: reader,
        }];
        // A shallower visit cannot certify a deeper occurrence. The key includes evaluation
        // BCI, and the value records the greatest checked depth; deeper visits must be walked.
        let mut seen: BTreeMap<(ValueId, u32), usize> = BTreeMap::new();
        let mut found_saved = false;
        while let Some(work) = pending.pop() {
            if work.depth > MAX_VALUE_DEPTH {
                return Ok(false);
            }
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, work.at)?;
            let facts = self.ssa.value(work.value);
            let Definition::Instruction {
                block: defined_in,
                bci,
            } = facts.def()
            else {
                return Ok(false);
            };
            let bci = *bci;
            if defined_in != &block
                || bci >= work.before
                || !matches!(facts.ty(), Value::Int | Value::Long)
                || self.bindings.contains_key(&work.value)
                || self.binding_refused.contains(&work.value)
                || self.conditional_values.contains_key(&work.value)
                || self.binding_plans.contains_key(&bci)
                || self.postfix.snapshot_at(work.value).is_some()
                || self.postfix.owns(bci)
                || self.compounds.owns_read(bci)
                || self.compounds.keeps_inline(bci)
                || self.chains.owns(bci)
                || self.field_copies.owns(bci)
                || self.sites.owns(bci)
                || self.array_initializers.owns(bci)
            {
                return Ok(false);
            }
            if seen
                .get(&(work.value, work.at))
                .is_some_and(|depth| *depth >= work.depth)
            {
                continue;
            }
            bill(self.budget, CountedBudgetDimension::IrItems, work.at)?;
            seen.insert((work.value, work.at), work.depth);
            let Some(instruction) = self.instructions.get(&bci).copied() else {
                return Ok(false);
            };
            if !instruction.copies().is_empty() {
                return Ok(false);
            }
            let mut output = None;
            for (slot, value) in instruction.writes() {
                bill(self.budget, CountedBudgetDimension::AnalysisSteps, bci)?;
                if !matches!(slot, Slot::Stack(_)) || output.replace(*value).is_some() {
                    return Ok(false);
                }
            }
            if output != Some(work.value) {
                return Ok(false);
            }
            if let Some(Operation::Load { slot }) = self.operations.get(bci) {
                let slot = *slot;
                if !matches!(
                    (facts.ty(), instruction.opcode()),
                    (Value::Int, 0x15 | 0x1a..=0x1d) | (Value::Long, 0x16 | 0x1e..=0x21)
                ) {
                    return Ok(false);
                }
                let mut denotes = None;
                for (read_slot, value) in instruction.reads() {
                    bill(self.budget, CountedBudgetDimension::AnalysisSteps, bci)?;
                    if *read_slot != Slot::Local(slot) || denotes.replace(*value).is_some() {
                        return Ok(false);
                    }
                }
                let Some(denotes) = denotes else {
                    return Ok(false);
                };
                let is_saved = work.value == saved;
                if is_saved && bci != producer {
                    return Ok(false);
                }
                if !self.snapshot_parameter_is_live(
                    slot,
                    denotes,
                    bci,
                    if is_saved { producer } else { work.at },
                    if is_saved { reader } else { work.at },
                    is_saved,
                    &mut pending,
                )? {
                    return Ok(false);
                }
                found_saved |= is_saved;
                continue;
            }
            let arity = match (self.operations.get(bci), facts.ty(), instruction.opcode()) {
                (
                    Some(Operation::Push(ConstantValue::Int(_))),
                    Value::Int,
                    0x02..=0x08 | 0x10..=0x13,
                )
                | (
                    Some(Operation::Push(ConstantValue::Long(_))),
                    Value::Long,
                    0x09 | 0x0a | 0x14,
                ) => 0,
                (
                    Some(Operation::Arithmetic {
                        op: ArithmeticOp::Add,
                    }),
                    Value::Int,
                    0x60,
                )
                | (
                    Some(Operation::Arithmetic {
                        op: ArithmeticOp::Add,
                    }),
                    Value::Long,
                    0x61,
                )
                | (
                    Some(Operation::Arithmetic {
                        op: ArithmeticOp::Subtract,
                    }),
                    Value::Int,
                    0x64,
                )
                | (
                    Some(Operation::Arithmetic {
                        op: ArithmeticOp::Subtract,
                    }),
                    Value::Long,
                    0x65,
                )
                | (
                    Some(Operation::Arithmetic {
                        op: ArithmeticOp::Multiply,
                    }),
                    Value::Int,
                    0x68,
                )
                | (
                    Some(Operation::Arithmetic {
                        op: ArithmeticOp::Multiply,
                    }),
                    Value::Long,
                    0x69,
                ) => 2,
                (Some(Operation::Negate), Value::Int, 0x74)
                | (Some(Operation::Negate), Value::Long, 0x75) => 1,
                _ => return Ok(false),
            };
            let mut operands = 0;
            for (slot, operand) in instruction.reads() {
                bill(self.budget, CountedBudgetDimension::AnalysisSteps, bci)?;
                if !matches!(slot, Slot::Stack(_)) || self.ssa.value(*operand).ty() != facts.ty() {
                    return Ok(false);
                }
                operands += 1;
                if operands > arity {
                    return Ok(false);
                }
                bill(self.budget, CountedBudgetDimension::IrItems, bci)?;
                pending.push(Pending {
                    value: *operand,
                    at: work.at,
                    depth: work.depth + 1,
                    before: bci,
                });
            }
            if operands != arity {
                return Ok(false);
            }
        }
        Ok(found_saved)
    }

    /// Read the original header parameter, all writes before the evaluation and its final SSA
    /// state. A refused Store must never make a later load appear declared merely by writing SSA.
    fn snapshot_parameter_is_live(
        &mut self,
        slot: u16,
        denotes: ValueId,
        load: u32,
        evaluation: u32,
        end: u32,
        original: bool,
        pending: &mut Vec<Pending>,
    ) -> Result<bool, StopReason> {
        bill(self.budget, CountedBudgetDimension::AnalysisSteps, load)?;
        let variable = LocalVariable::whole(slot);
        let Some(ty @ (Type::Int | Type::Long)) = self.parameter_types.get(&slot) else {
            return Ok(false);
        };
        if slot >= self.parameters
            || (slot == 0 && self.has_receiver)
            || matches!(
                self.reuse.evidence().get(usize::from(slot)),
                Some(crate::names::SlotEvidence::Split(_))
            )
            || self.reuse.variable_at(slot, load) != Some(variable)
            || self.reuse.variable_at(slot, evaluation) != Some(variable)
            || self.decided_type(variable).as_ref() != Some(ty)
        {
            return Ok(false);
        }
        let Some(name) = self.names.text(variable) else {
            return Ok(false);
        };
        if self.undeclared.contains(name) {
            return Ok(false);
        }
        let Some(block) = self.block_of.get(&load) else {
            return Ok(false);
        };
        let Some(names) = self.ssa.block(block) else {
            return Ok(false);
        };
        let mut entry = None;
        for (candidate, value) in names.entry() {
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, load)?;
            if *candidate == Slot::Local(slot) {
                entry = Some(*value);
            }
        }
        let Some(entry) = entry else { return Ok(false) };
        bill(self.budget, CountedBudgetDimension::AnalysisSteps, load)?;
        if !matches!(self.ssa.value(entry).def(), Definition::Entry { block: at, slot: entry_slot }
            if at == block && *entry_slot == Slot::Local(slot))
            || !matches!(
                (self.ssa.value(entry).ty(), ty),
                (Value::Int, Type::Int) | (Value::Long, Type::Long)
            )
            || self.ssa.value(denotes).ty() != self.ssa.value(entry).ty()
            || (original && denotes != entry)
        {
            return Ok(false);
        }
        let mut current = entry;
        let mut at_load = None;
        let mut at_evaluation = None;
        for instruction in names.instructions() {
            let at = instruction.bci();
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, at)?;
            if at == load {
                at_load = Some(current);
            }
            if at == evaluation {
                at_evaluation = Some(current);
            }
            if at >= end {
                break;
            }
            let mut write = None;
            for (target, value) in instruction.writes() {
                bill(self.budget, CountedBudgetDimension::AnalysisSteps, at)?;
                // A new source-local declaration is outside this parameter-only closure.
                if matches!(target, Slot::Local(local) if *local >= self.parameters) {
                    return Ok(false);
                }
                if *target == Slot::Local(slot) {
                    write = Some(*value);
                }
            }
            let Some(written) = write else { continue };
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, at)?;
            if self.ssa.value(written).ty() != self.ssa.value(entry).ty()
                || !matches!(self.ssa.value(written).def(), Definition::Instruction { block: at_block, bci }
                    if at_block == block && *bci == at)
                || self.reuse.variable_at(slot, at) != Some(variable)
                || self.postfix.owns(at)
                || self.local_assignments.contains_key(&at)
                || self.compounds.owns_read(at)
                || self.compounds.keeps_inline(at)
                || !instruction.copies().is_empty()
            {
                return Ok(false);
            }
            let mut input = None;
            for (read_slot, value) in instruction.reads() {
                bill(self.budget, CountedBudgetDimension::AnalysisSteps, at)?;
                if input.replace((*read_slot, *value)).is_some() {
                    return Ok(false);
                }
            }
            match (self.operations.get(at), instruction.opcode(), ty, input) {
                (
                    Some(Operation::Increment { slot: target, .. }),
                    0x84,
                    Type::Int,
                    Some((Slot::Local(read), value)),
                ) if *target == slot && read == slot && value == current => {}
                (
                    Some(Operation::Store { slot: target }),
                    opcode,
                    _,
                    Some((Slot::Stack(_), rhs)),
                ) if *target == slot
                    && matches!(
                        (ty, opcode),
                        (Type::Int, 0x36 | 0x3b..=0x3e) | (Type::Long, 0x37 | 0x3f..=0x42)
                    ) =>
                {
                    bill(self.budget, CountedBudgetDimension::AnalysisSteps, at)?;
                    if self.ssa.value(rhs).ty() != self.ssa.value(entry).ty() {
                        return Ok(false);
                    }
                    bill(self.budget, CountedBudgetDimension::IrItems, at)?;
                    pending.push(Pending {
                        value: rhs,
                        at,
                        depth: 0,
                        before: at,
                    });
                }
                _ => return Ok(false),
            }
            current = written;
        }
        if names
            .instructions()
            .last()
            .is_some_and(|instruction| instruction.bci() == end)
        {
            let mut exit = None;
            for (candidate, value) in names.exit() {
                bill(self.budget, CountedBudgetDimension::AnalysisSteps, end)?;
                if *candidate == Slot::Local(slot) {
                    exit = Some(*value);
                }
            }
            if exit != Some(current) {
                return Ok(false);
            }
        }
        Ok(at_load == Some(denotes)
            && at_evaluation == Some(denotes)
            && (!original || current != denotes))
    }
}
