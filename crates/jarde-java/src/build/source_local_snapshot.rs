//! A source-local load saved for a pure integer assignment after an intervening update.
//! This is separate from parameter admission: the local's declaration must be proved too.
//! The ordinary binding planner and transaction still own every saved name and statement.
use super::*;

fn bill(budget: &mut Budget, dimension: CountedBudgetDimension, at: u32) -> Result<(), StopReason> {
    charge(budget, dimension, 1, Some(at))?;
    poll(budget, Some(at))
}

impl Builder<'_> {
    /// Certify the whole entry-block prefix through a Store, including its first declaration.
    /// One unsplit int local is initialized by a pure expression, then changed only by unowned
    /// increments before the assignment. All stack values have exactly one forward consumer.
    /// Thus every stale leaf of the Store RHS is independently retained by the ordinary binding
    /// planner; proving only the requested leaf would leave a saved declaration before a refused
    /// sibling. No expression is rendered and no builder state is published during this proof.
    pub(super) fn source_local_snapshot(
        &mut self,
        saved: ValueId,
        producer: u32,
        reader: u32,
    ) -> Result<bool, StopReason> {
        bill(self.budget, CountedBudgetDimension::AnalysisSteps, producer)?;
        let Some(Operation::Load { slot }) = self.operations.get(producer) else {
            return Ok(false);
        };
        let slot = *slot;
        if slot < self.parameters
            || producer >= reader
            || !matches!(self.operations.get(reader), Some(Operation::Store { slot: target }) if *target == slot)
        {
            return Ok(false);
        }
        let Some(block) = self.block_of.get(&producer) else {
            return Ok(false);
        };
        if block.bci() != 0
            || block.is_clone()
            || self.block_of.get(&reader) != Some(block)
            || self.code.exception_handler_count != 0
            || !self.code.exception_handlers.is_empty()
        {
            return Ok(false);
        }
        // The terminal Store always changes the block's exit identity, even when the RHS
        // loads were stable, so comparing with that exit would not reject large stable trees.
        // In this proof's prefix, only iinc can end a loaded local identity before the Store.
        // Its read is that identity's last SSA use: run_block records instruction reads in
        // order, and the full prefix below excludes earlier Stores, handlers and merges.
        // This is only a necessary filter. It neither publishes a binding nor substitutes for
        // the complete proof, and stable leaves pay only these endpoint checks, with no scan.
        let Some(load) = self.instructions.get(&producer).copied() else {
            return Ok(false);
        };
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
        bill(self.budget, CountedBudgetDimension::AnalysisSteps, producer)?;
        let local = self.ssa.value(denotes);
        if !matches!(local.def(), Definition::Instruction { block: defined_in, bci }
            if defined_in == block && *bci < producer)
        {
            return Ok(false);
        }
        let Some(last_use) = local.uses().last() else {
            return Ok(false);
        };
        let Some(update) = last_use.bci() else {
            return Ok(false);
        };
        if last_use.block() != block
            || !(producer < update && update < reader)
            || !matches!(self.operations.get(update), Some(Operation::Increment { slot: target, .. }) if *target == slot)
        {
            return Ok(false);
        }
        let block = block.clone();
        for edge in self.canonical.edges() {
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, reader)?;
            if edge.to() == &block {
                return Ok(false);
            }
        }
        let variable = LocalVariable::whole(slot);
        if self.decided_type(variable) != Some(Type::Int)
            || matches!(
                self.reuse.evidence().get(usize::from(slot)),
                Some(crate::names::SlotEvidence::Split(_))
            )
            || !matches!(
                self.declarations.placements.get(&variable),
                Some(DeclarationPlacement::Local { .. })
            )
        {
            return Ok(false);
        }
        let Some(name) = self.names.text(variable) else {
            return Ok(false);
        };
        if self.undeclared.contains(name) {
            return Ok(false);
        }
        let Some(names) = self.ssa.block(&block) else {
            return Ok(false);
        };
        for (entry, _) in names.entry() {
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, producer)?;
            if matches!(entry, Slot::Stack(_)) {
                return Ok(false);
            }
        }
        // Values leave this map at their sole use. A depth is the maximum leaf depth of the
        // complete pure expression, so a deep sibling cannot hide behind a shallower saved leaf.
        let mut expressions: BTreeMap<ValueId, usize> = BTreeMap::new();
        let mut loads = Vec::new();
        let mut current = None;
        let mut reached = false;
        for instruction in names.instructions() {
            let at = instruction.bci();
            if at > reader {
                break;
            }
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, at)?;
            if !instruction.copies().is_empty()
                || self.postfix.owns(at)
                || self.compounds.owns_read(at)
                || self.compounds.keeps_inline(at)
                || self.chains.owns(at)
                || self.field_copies.owns(at)
                || self.sites.owns(at)
                || self.array_initializers.owns(at)
                || self.local_assignments.contains_key(&at)
                || self.compound_refused_producers.contains(&at)
                || self.binding_plans.contains_key(&at)
            {
                return Ok(false);
            }
            let (arity, local_read, local_write) =
                match (self.operations.get(at), instruction.opcode()) {
                    (Some(Operation::Push(ConstantValue::Int(_))), 0x02..=0x08 | 0x10..=0x13) => {
                        (0, false, false)
                    }
                    (Some(Operation::Load { slot: target }), 0x15 | 0x1a..=0x1d)
                        if *target == slot =>
                    {
                        (0, true, false)
                    }
                    (Some(Operation::Increment { slot: target, .. }), 0x84) if *target == slot => {
                        (0, true, true)
                    }
                    (Some(Operation::Store { slot: target }), 0x36 | 0x3b..=0x3e)
                        if *target == slot =>
                    {
                        (1, false, true)
                    }
                    (
                        Some(Operation::Arithmetic {
                            op: ArithmeticOp::Add,
                        }),
                        0x60,
                    )
                    | (
                        Some(Operation::Arithmetic {
                            op: ArithmeticOp::Subtract,
                        }),
                        0x64,
                    )
                    | (
                        Some(Operation::Arithmetic {
                            op: ArithmeticOp::Multiply,
                        }),
                        0x68,
                    ) => (2, false, false),
                    (Some(Operation::Negate), 0x74) => (1, false, false),
                    _ => return Ok(false),
                };
            if (local_read || local_write) && self.reuse.variable_at(slot, at) != Some(variable) {
                return Ok(false);
            }
            let mut read_count = 0;
            let mut depth = 0;
            for (source, input) in instruction.reads() {
                bill(self.budget, CountedBudgetDimension::AnalysisSteps, at)?;
                read_count += 1;
                if self.ssa.value(*input).ty() != &Value::Int {
                    return Ok(false);
                }
                if local_read {
                    if *source != Slot::Local(slot) || current != Some(*input) || read_count != 1 {
                        return Ok(false);
                    }
                } else {
                    if !matches!(source, Slot::Stack(_)) || read_count > arity {
                        return Ok(false);
                    }
                    let Some(input_depth) = expressions.remove(input) else {
                        return Ok(false);
                    };
                    depth = depth.max(input_depth + usize::from(!local_write));
                }
            }
            if read_count != if local_read { 1 } else { arity } || depth > MAX_VALUE_DEPTH {
                return Ok(false);
            }
            let mut output = None;
            for (target, written) in instruction.writes() {
                bill(self.budget, CountedBudgetDimension::AnalysisSteps, at)?;
                if output.replace(*written).is_some()
                    || self.ssa.value(*written).ty() != &Value::Int
                    || !matches!(self.ssa.value(*written).def(), Definition::Instruction { block: defined_in, bci } if defined_in == &block && *bci == at)
                    || if local_write {
                        *target != Slot::Local(slot)
                    } else {
                        !matches!(target, Slot::Stack(_))
                    }
                    || self.bindings.contains_key(written)
                    || self.binding_refused.contains(written)
                    || self.conditional_values.contains_key(written)
                    || self.postfix.snapshot_at(*written).is_some()
                    || self.short_circuit_operands.contains(written)
                    || self.copy_inputs.contains(written)
                {
                    return Ok(false);
                }
            }
            let Some(output) = output else {
                return Ok(false);
            };
            if local_write {
                if at == reader {
                    // This is an assignment to the previously declared local. Leave current at
                    // its pre-Store value: it is exactly what names in this RHS would observe.
                    if local_read || current.is_none() {
                        return Ok(false);
                    }
                    reached = true;
                } else {
                    // Only the first Store may declare the local. Later writes before the
                    // consumer are iinc, whose spelling reads the same current int identity.
                    if !local_read && current.is_some() {
                        return Ok(false);
                    }
                    current = Some(output);
                }
            } else {
                bill(self.budget, CountedBudgetDimension::AnalysisSteps, at)?;
                let [use_] = self.ssa.value(output).uses() else {
                    return Ok(false);
                };
                if use_.block() != &block
                    || !use_
                        .bci()
                        .is_some_and(|use_at| at < use_at && use_at <= reader)
                {
                    return Ok(false);
                }
                bill(self.budget, CountedBudgetDimension::IrItems, at)?;
                expressions.insert(output, depth);
                if local_read {
                    bill(self.budget, CountedBudgetDimension::IrItems, at)?;
                    loads.push((output, at, current));
                }
            }
        }
        if !reached || !expressions.is_empty() {
            return Ok(false);
        }
        let mut found_saved = false;
        for (value, at, denotes) in loads {
            bill(self.budget, CountedBudgetDimension::AnalysisSteps, at)?;
            if denotes == current {
                continue;
            }
            // Every stale sibling must have the same one terminal Store. Single-use pure
            // arithmetic cannot become another saved producer that absorbs this load, and an
            // unowned iinc separates it from the Store, so ordinary retention keeps its save.
            if self.terminal_consumer(value, 0, None)? != Some(reader) {
                return Ok(false);
            }
            poll(self.budget, Some(reader))?;
            found_saved |= value == saved && at == producer;
        }
        Ok(found_saved)
    }
}
