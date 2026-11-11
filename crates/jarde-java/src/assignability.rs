//! Bounded reference producer/copy constraints shared by selected-Header proofs and local planning.
//! Descriptor uses constrain the declarations Java will read; verifier names on copies and phis
//! are not independent producers. Named conversions consume exact-BCI, environment-bound facts.
//! The registered platform parent rows are raw class-header facts, separate from selected Headers.

use crate::decode::Operations;
use crate::facts::{ConstantValue, FieldAccess, Operation};
use jarde_jvm::method_ir::{
    Definition, PhiInput, RefType, Slot, SsaTable, SsaValue, Value, ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::error::Result;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Reference constraints over the current immutable method. Domains contain producer types and
/// descriptor requirements, propagated only along actual local copies. They are possible source
/// declarations, not assertions that any conversion is legal.
pub(crate) struct ReferenceConstraints {
    pub(crate) bounds: ReferenceBounds,
    pub(crate) domains: BTreeMap<u16, BTreeSet<String>>,
    pub(crate) requirements: BTreeMap<(u16, u32), BTreeSet<String>>,
    pub(crate) consumers: Vec<(u32, ValueId, String)>,
    pub(crate) stores: Vec<(u16, u32, ValueId)>,
    pub(crate) array_uses: BTreeSet<(u16, u32, u8)>,
}

pub(crate) fn reference_constraints(
    ssa: &SsaTable,
    operations: &Operations,
    return_type: Option<&crate::ast::Type>,
    has_receiver: bool,
    budget: &mut Budget,
) -> Result<ReferenceConstraints> {
    use crate::facts::InvokeKind;
    let bounds = reference_bounds(ssa, operations, has_receiver, budget)?;
    let mut domains = BTreeMap::<u16, BTreeSet<String>>::new();
    let mut requirements = BTreeMap::<(u16, u32), BTreeSet<String>>::new();
    let mut consumers = Vec::new();
    let mut stores = Vec::new();
    let mut copies = BTreeSet::new();
    let mut array_uses = BTreeSet::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            let bci = instruction.bci();
            let mut stack: Vec<_> = instruction
                .reads()
                .iter()
                .filter_map(|(slot, value)| {
                    if let Slot::Stack(depth) = slot {
                        Some((*depth, *value))
                    } else {
                        None
                    }
                })
                .collect();
            stack.sort_by_key(|(depth, _)| *depth);
            if matches!(
                operations.get(bci),
                Some(
                    Operation::ArrayLength
                        | Operation::ArrayLoad
                        | Operation::ArrayElementLoad { .. }
                        | Operation::ArrayStore { .. }
                )
            ) {
                if let Some((_, array)) = stack.first() {
                    for origin in bounds.local_sources(*array, budget)? {
                        budget.charge(CountedBudgetDimension::IrItems, 1)?;
                        array_uses.insert(origin);
                    }
                }
            }
            let mut require = |ordinal: usize, ty: crate::ast::Type| {
                if let (Some((_, value)), crate::ast::Type::Reference(target)) =
                    (stack.get(ordinal), ty)
                {
                    consumers.push((bci, *value, target));
                }
            };
            match operations.get(bci) {
                Some(Operation::Invoke(target)) => {
                    let receiver = usize::from(target.kind() != InvokeKind::Static);
                    if matches!(target.kind(), InvokeKind::Virtual | InvokeKind::Interface) {
                        if let Some(owner) = crate::build::spell_class_operand(target.raw_owner()) {
                            require(0, crate::ast::Type::Reference(owner));
                        }
                    }
                    if let Ok(facts) = jarde_reader::classfile::descriptor_facts(
                        &target.raw_descriptor().0,
                        jarde_reader::classfile::DescriptorKind::Method,
                    ) {
                        for (ordinal, component) in facts.parameters().iter().enumerate() {
                            if let Some(ty) = crate::type_of_component(component) {
                                require(ordinal + receiver, ty);
                            }
                        }
                    }
                }
                Some(Operation::InvokeDynamic(site)) => {
                    if let Ok(facts) = jarde_reader::classfile::descriptor_facts(
                        &site.raw_descriptor().0,
                        jarde_reader::classfile::DescriptorKind::Method,
                    ) {
                        for (ordinal, component) in facts.parameters().iter().enumerate() {
                            if let Some(ty) = crate::type_of_component(component) {
                                require(ordinal, ty);
                            }
                        }
                    }
                }
                Some(Operation::Return) => {
                    if let Some(ty) = return_type {
                        require(0, ty.clone());
                    }
                }
                Some(Operation::Throw) => {
                    require(
                        0,
                        crate::ast::Type::Reference("java.lang.Throwable".to_owned()),
                    );
                }
                Some(
                    Operation::ArrayElementLoad { element: None }
                    | Operation::ArrayStore { element: None },
                ) => {
                    require(
                        0,
                        crate::ast::Type::Reference("java.lang.Object[]".to_owned()),
                    );
                }
                Some(Operation::Field {
                    access,
                    is_static,
                    owner,
                    descriptor,
                    ..
                }) => {
                    if !is_static {
                        if let Some(owner) = crate::names::source_class_name(owner) {
                            require(0, crate::ast::Type::Reference(owner));
                        }
                    }
                    if *access == FieldAccess::Write {
                        if let Some(ty) =
                            crate::source_proof::generic_call_reference_type_from_descriptor(
                                &descriptor.0,
                            )
                        {
                            require(usize::from(!is_static), ty);
                        }
                    }
                }
                Some(Operation::Store { slot }) if stack.len() == 1 => {
                    let value = stack[0].1;
                    if let Some(types) = bounds.types_of(value) {
                        let domain = domains.entry(*slot).or_default();
                        budget.charge(CountedBudgetDimension::IrItems, 1)?;
                        domain.insert("java.lang.Object".to_owned());
                        for ty in types {
                            budget.charge(CountedBudgetDimension::IrItems, 1)?;
                            domain.insert(ty.to_owned());
                        }
                        stores.push((*slot, bci, value));
                        for (source, load, projection) in bounds.local_sources(value, budget)? {
                            copies.insert((source, load, projection, *slot, bci));
                        }
                    }
                }
                Some(Operation::Load { slot }) => {
                    for (_, value) in instruction.writes() {
                        if let Some(types) = bounds.types_of(*value) {
                            for ty in types {
                                budget.charge(CountedBudgetDimension::IrItems, 1)?;
                                domains.entry(*slot).or_default().insert(ty.to_owned());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    for (_, value, target) in &consumers {
        for (slot, load, projection) in bounds.local_sources(*value, budget)? {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::IrItems, 2)?;
            let target = format!("{target}{}", "[]".repeat(usize::from(projection)));
            domains.entry(slot).or_default().insert(target.clone());
            requirements.entry((slot, load)).or_default().insert(target);
        }
    }
    // A copy can constrain both ends. The queue contains physical slots only once at a time;
    // each successful insertion is charged before allocating the new domain cell.
    let mut adjacent = BTreeMap::<u16, BTreeSet<usize>>::new();
    let copies: Vec<_> = copies.into_iter().collect();
    for (index, (source, _, _, target, _)) in copies.iter().enumerate() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::IrEdges, 2)?;
        adjacent.entry(*source).or_default().insert(index);
        adjacent.entry(*target).or_default().insert(index);
    }
    budget.charge(CountedBudgetDimension::IrItems, domains.len() as u64 * 2)?;
    let mut queue: VecDeque<_> = domains.keys().copied().collect();
    let mut queued: BTreeSet<_> = domains.keys().copied().collect();
    while let Some(slot) = queue.pop_front() {
        queued.remove(&slot);
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let count = domains.get(&slot).map_or(0, BTreeSet::len);
        budget.charge(CountedBudgetDimension::AnalysisSteps, count as u64)?;
        let source = domains.get(&slot).cloned().unwrap_or_default();
        for index in adjacent.get(&slot).into_iter().flatten() {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            let (from, _, projection, to, _) = &copies[*index];
            for name in &source {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                let (destination, candidate) = if *from == slot {
                    let mut candidate = name.as_str();
                    let mut valid = true;
                    for _ in 0..*projection {
                        if let Some(component) = candidate.strip_suffix("[]") {
                            candidate = component;
                        } else {
                            valid = false;
                            break;
                        }
                    }
                    if !valid {
                        continue;
                    }
                    (*to, candidate.to_owned())
                } else {
                    if name.matches("[]").count() + usize::from(*projection) > 255 {
                        continue;
                    }
                    (
                        *from,
                        format!("{name}{}", "[]".repeat(usize::from(*projection))),
                    )
                };
                let domain = domains.entry(destination).or_default();
                if !domain.contains(&candidate) {
                    budget.charge(CountedBudgetDimension::IrItems, 1)?;
                    domain.insert(candidate);
                    if queued.insert(destination) {
                        queue.push_back(destination);
                    }
                }
            }
        }
    }
    Ok(ReferenceConstraints {
        bounds,
        domains,
        requirements,
        consumers,
        stores,
        array_uses,
    })
}

/// Complete reference producer bounds for this SSA table. Types are interned once; propagation
/// copies small indices, rather than duplicating class-name strings at each value and phi.
pub(crate) struct ReferenceBounds {
    names: Vec<String>,
    values: BTreeMap<ValueId, BTreeSet<usize>>,
    reads: BTreeMap<ValueId, (u16, u32)>,
    flows: BTreeMap<ValueId, Vec<(ValueId, u8)>>,
}

/// The original named producer type and its format, before source spelling. A CONSTANT_Class
/// operand is an internal name/array descriptor; a declared parameter/field/result is a descriptor.
/// Copies and phis are handled by the existing flow graph, never by reinterpreting their names.
pub(crate) fn named_reference_type<'a>(
    value: &'a SsaValue,
    operations: &Operations,
    has_receiver: bool,
) -> Option<(&'a [u8], &'a jarde_reader::view::LoaderId, bool)> {
    let Value::Ref(RefType::Named { name, loader }) = value.ty() else {
        return None;
    };
    let class_operand = match value.def() {
        Definition::Entry {
            slot: Slot::Local(0),
            ..
        } => has_receiver,
        Definition::Caught { .. } => true,
        Definition::Instruction { bci, .. } => match operations.get(*bci) {
            Some(Operation::Allocate { .. } | Operation::CheckCast { .. }) => true,
            Some(Operation::Invoke(target)) => target.raw_name().0 == b"<init>",
            _ => false,
        },
        _ => false,
    };
    Some((name, loader, class_operand))
}

impl ReferenceBounds {
    /// The same actual SSA copy/phi/array dependencies used by erased bounds. Source Signature
    /// propagation consumes this graph rather than reconstructing another stack or local walk.
    pub(crate) fn parents_of(&self, value: ValueId) -> &[(ValueId, u8)] {
        self.flows.get(&value).map_or(&[], Vec::as_slice)
    }

    /// None is an unproved producer; an empty set is the proved null-only bound.
    pub(crate) fn types_of(&self, value: ValueId) -> Option<impl Iterator<Item = &str>> {
        Some(
            self.values
                .get(&value)?
                .iter()
                .map(|index| self.names[*index].as_str()),
        )
    }

    /// The physical loads from which a consumer obtains its reference. A projection counts
    /// reference-array element reads between that load and the consumer. Casts and fresh values
    /// stop the walk: they have their own type and do not impose an upcast on the earlier local.
    pub(crate) fn local_sources(
        &self,
        value: ValueId,
        budget: &mut Budget,
    ) -> Result<BTreeSet<(u16, u32, u8)>> {
        let mut pending = vec![(value, 0_u8)];
        let mut seen = BTreeSet::new();
        let mut result = BTreeSet::new();
        while let Some((value, projection)) = pending.pop() {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if !seen.insert((value, projection)) {
                continue;
            }
            if let Some((slot, at)) = self.reads.get(&value) {
                result.insert((*slot, *at, projection));
                continue;
            }
            for (parent, peel) in self.flows.get(&value).into_iter().flatten() {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                // An array has at most 255 dimensions. A cycle requiring more cannot describe
                // a source reference-array type; it provides no additional load origin.
                if let Some(projection) = projection.checked_add(*peel) {
                    pending.push((*parent, projection));
                }
            }
        }
        Ok(result)
    }
}

/// Solve copies and reference phis with a bounded work queue. Every incoming dependency is
/// retained: one unknown leaf blocks the result even when another predecessor has a useful
/// type. A cyclic component needs a real producer or null seed, so a rootless self-phi cannot
/// manufacture a declared type. This uses the immutable SSA; it reads no dependency Body.
pub(crate) fn reference_bounds(
    ssa: &SsaTable,
    operations: &Operations,
    has_receiver: bool,
    budget: &mut Budget,
) -> Result<ReferenceBounds> {
    let mut instructions = BTreeMap::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            instructions.insert((block.block().clone(), instruction.bci()), instruction);
        }
    }
    let mut phis = BTreeMap::new();
    for phi in ssa.phis() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        phis.insert(phi.value(), phi);
    }
    let mut names = Vec::new();
    let mut name_indices = BTreeMap::new();
    let mut values = BTreeMap::<ValueId, BTreeSet<usize>>::new();
    let mut dependencies = BTreeMap::<ValueId, Vec<(ValueId, u8)>>::new();
    let mut reads = BTreeMap::new();
    let mut rooted = BTreeSet::new();
    let mut blocked = BTreeSet::new();
    for (id, value) in ssa.values_with_ids() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !matches!(value.ty(), Value::Ref(_) | Value::Null) {
            continue;
        }
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        values.insert(id, BTreeSet::new());
        let mut direct = None;
        let mut parents = Vec::new();
        let mut seed = false;
        if let Some(replaced) = value.replaced_by() {
            parents.push((replaced, 0));
        } else if let Some((name, _, class_operand)) =
            named_reference_type(value, operations, has_receiver)
        {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                (name.len() as u64).div_ceil(64),
            )?;
            let raw = jarde_reader::model::JvmBytes(name.to_vec());
            direct = if class_operand {
                crate::build::spell_class_operand(&raw)
            } else {
                raw.to_unicode()
                    .and_then(|name| crate::build::spell_reference(&name))
            };
            seed = direct.is_some();
        } else if matches!(value.ty(), Value::Null) {
            seed = true;
        } else {
            match value.def() {
                Definition::Phi { .. } => {
                    if let Some(phi) = phis.get(&id) {
                        for input in phi.inputs() {
                            budget.poll()?;
                            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                            parents.push((
                                match input {
                                    PhiInput::Value(value) => *value,
                                    PhiInput::Itself => id,
                                },
                                0,
                            ));
                        }
                    }
                }
                Definition::Instruction { block, bci } => {
                    if let Some(instruction) = instructions.get(&(block.clone(), *bci)) {
                        match operations.get(*bci) {
                            Some(Operation::Push(ConstantValue::String(_))) => {
                                direct = Some("java.lang.String".to_owned())
                            }
                            Some(Operation::Push(ConstantValue::Class { .. })) => {
                                direct = Some("java.lang.Class".to_owned())
                            }
                            Some(Operation::Load { slot }) => {
                                parents.extend(instruction.reads().iter().filter_map(
                                    |(read, value)| {
                                        (*read == Slot::Local(*slot)).then_some((*value, 0))
                                    },
                                ));
                            }
                            Some(Operation::Store { .. } | Operation::Duplicate) => {
                                if let [(Slot::Stack(_), value)] = instruction.reads() {
                                    parents.push((*value, 0));
                                }
                            }
                            Some(Operation::ArrayLoad | Operation::ArrayElementLoad { .. }) => {
                                // The lowest consumed stack depth is the array; the index is
                                // above it. A reference element peels one source array dimension.
                                if let Some((_, array)) = instruction
                                    .reads()
                                    .iter()
                                    .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
                                    .min_by_key(|(slot, _)| match slot {
                                        Slot::Stack(depth) => *depth,
                                        _ => unreachable!(),
                                    })
                                {
                                    parents.push((*array, 1));
                                }
                            }
                            Some(Operation::Field {
                                access: FieldAccess::Read,
                                descriptor,
                                ..
                            }) => {
                                direct = crate::source_proof::generic_call_reference_type_from_descriptor(&descriptor.0)
                                    .map(|ty| ty.spell().to_owned());
                            }
                            Some(Operation::Invoke(target)) => {
                                if let Ok(facts) = jarde_reader::classfile::descriptor_facts(
                                    &target.raw_descriptor().0,
                                    jarde_reader::classfile::DescriptorKind::Method,
                                ) {
                                    direct = facts
                                        .result()
                                        .and_then(crate::type_of_component)
                                        .filter(|ty| matches!(ty, crate::ast::Type::Reference(_)))
                                        .map(|ty| ty.spell().to_owned());
                                }
                            }
                            Some(Operation::CheckCast { ty }) => {
                                direct = crate::build::spell_class_operand(ty);
                            }
                            Some(Operation::NewArray {
                                element,
                                total_dimensions,
                                ..
                            }) => {
                                direct = Some(format!(
                                    "{}{}",
                                    element.spell(),
                                    "[]".repeat(usize::from(*total_dimensions))
                                ));
                            }
                            _ => {}
                        }
                        seed = direct.is_some();
                    }
                }
                Definition::Entry { .. } | Definition::Caught { .. } => {}
            }
        }
        if let Definition::Instruction { block, bci } = value.def() {
            if let Some(instruction) = instructions.get(&(block.clone(), *bci)) {
                match operations.get(*bci) {
                    Some(Operation::Load { slot }) => {
                        reads.insert(id, (*slot, *bci));
                        if parents.is_empty() {
                            parents.extend(instruction.reads().iter().filter_map(
                                |(read, value)| {
                                    (*read == Slot::Local(*slot)).then_some((*value, 0))
                                },
                            ));
                        }
                    }
                    Some(Operation::Store { .. } | Operation::Duplicate) if parents.is_empty() => {
                        if let [(Slot::Stack(_), source)] = instruction.reads() {
                            parents.push((*source, 0));
                        }
                    }
                    Some(Operation::ArrayLoad | Operation::ArrayElementLoad { .. })
                        if parents.is_empty() =>
                    {
                        if let Some((_, source)) = instruction
                            .reads()
                            .iter()
                            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
                            .min_by_key(|(slot, _)| match slot {
                                Slot::Stack(depth) => *depth,
                                _ => unreachable!(),
                            })
                        {
                            parents.push((*source, 1));
                        }
                    }
                    _ => {}
                }
            }
        } else if let Definition::Phi { .. } = value.def() {
            if parents.is_empty() {
                if let Some(phi) = phis.get(&id) {
                    for input in phi.inputs() {
                        budget.poll()?;
                        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                        parents.push((
                            match input {
                                PhiInput::Value(source) => *source,
                                PhiInput::Itself => id,
                            },
                            0,
                        ));
                    }
                }
            }
        }
        if let Definition::Instruction { block, bci } = value.def()
            && let Some(instruction) = instructions.get(&(block.clone(), *bci))
            && let Some((_, source)) = instruction
                .copies()
                .iter()
                .find(|(output, _)| *output == id)
        {
            parents.clear();
            parents.push((*source, 0));
        }
        // A verifier name on a copy or phi is a bound, not an additional producer. In particular
        // it cannot root an otherwise rootless cycle or turn a null/String copy into Object.
        let copy = matches!(value.def(), Definition::Instruction { bci, .. }
            if matches!(operations.get(*bci), Some(Operation::Load { .. } | Operation::Store { .. }
                | Operation::Duplicate | Operation::ArrayLoad | Operation::ArrayElementLoad { .. })));
        if copy || !parents.is_empty() || matches!(value.def(), Definition::Phi { .. }) {
            direct = None;
            seed = false;
        }
        if let Some(name) = direct {
            budget.charge(
                CountedBudgetDimension::AnalysisSteps,
                (name.len() as u64).div_ceil(64),
            )?;
            let index = match name_indices.get(&name) {
                Some(index) => *index,
                None => {
                    budget.charge(CountedBudgetDimension::IrItems, 1)?;
                    let index = names.len();
                    names.push(name.clone());
                    name_indices.insert(name, index);
                    index
                }
            };
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            values
                .get_mut(&id)
                .expect("reference node exists")
                .insert(index);
        }
        if seed {
            rooted.insert(id);
        }
        if !parents.is_empty() {
            dependencies.insert(id, parents);
        } else if !seed {
            blocked.insert(id);
        }
    }
    let mut readers = BTreeMap::<ValueId, BTreeSet<(ValueId, u8)>>::new();
    for (value, parents) in &dependencies {
        for (parent, projection) in parents {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if !values.contains_key(parent) {
                blocked.insert(*value);
            }
            budget.charge(CountedBudgetDimension::IrEdges, 1)?;
            readers
                .entry(*parent)
                .or_default()
                .insert((*value, *projection));
        }
    }
    let mut queue: VecDeque<_> = values.keys().copied().collect();
    let mut queued: BTreeSet<_> = values.keys().copied().collect();
    while let Some(value) = queue.pop_front() {
        queued.remove(&value);
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let source = &values[&value];
        budget.charge(CountedBudgetDimension::AnalysisSteps, source.len() as u64)?;
        let source = source.clone();
        for (reader, projection) in readers.get(&value).into_iter().flatten() {
            budget.poll()?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            let mut changed = false;
            if rooted.contains(&value) {
                changed |= rooted.insert(*reader);
            }
            if blocked.contains(&value) {
                changed |= blocked.insert(*reader);
            }
            for index in &source {
                budget.poll()?;
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                let index = if *projection == 0 {
                    *index
                } else {
                    let Some(component) = names[*index].strip_suffix("[]") else {
                        changed |= blocked.insert(*reader);
                        continue;
                    };
                    if matches!(
                        component,
                        "boolean" | "byte" | "short" | "char" | "int" | "long" | "float" | "double"
                    ) {
                        changed |= blocked.insert(*reader);
                        continue;
                    }
                    budget.charge(
                        CountedBudgetDimension::AnalysisSteps,
                        (component.len() as u64).div_ceil(64),
                    )?;
                    let component = component.to_owned();
                    match name_indices.get(&component) {
                        Some(index) => *index,
                        None => {
                            budget.charge(CountedBudgetDimension::IrItems, 1)?;
                            let index = names.len();
                            names.push(component.clone());
                            name_indices.insert(component, index);
                            index
                        }
                    }
                };
                let target = values
                    .get_mut(reader)
                    .expect("dependency reader is a reference node");
                if !target.contains(&index) {
                    budget.charge(CountedBudgetDimension::IrItems, 1)?;
                    target.insert(index);
                    changed = true;
                }
            }
            if changed && queued.insert(*reader) {
                queue.push_back(*reader);
            }
        }
    }
    values.retain(|value, _| rooted.contains(value) && !blocked.contains(value));
    Ok(ReferenceBounds {
        names,
        values,
        reads,
        flows: dependencies,
    })
}

/// Shared source-side consumption of the relation established for this exact physical use.
/// The proved relation at the exact site decides before any static row: a matched negative
/// refuses decisively, a matched positive carries the answer, and only a site no relation
/// states falls back to the structural head and the release-8 canonical rows. The previous
/// order — canonical rows first — answered a table-covered pair before its own relation was
/// read, so a snapshot-shadowed platform name, an uncertain loader chain or a non-8 release
/// kept a cast the environment's own header walk refuses (F2's five production-reachable
/// forms; the N3 adjudication orders the facts by strength).
pub(crate) fn reference_assignable(
    java_release: u16,
    source: &str,
    target: &str,
    bci: u32,
    relations: &[crate::report::ReferenceRelationInput],
) -> bool {
    let mut stated = false;
    for proof in relations
        .iter()
        .filter(|proof| proof.bci == bci && proof.source == source && proof.target == target)
    {
        if !proof.relation.is_assignable(java_release) {
            return false;
        }
        stated = true;
    }
    if stated {
        return true;
    }
    structurally_assignable(source, target)
        || (java_release == 8 && canonical_platform_rows(source, target))
}

/// The initializer element's own contract: the closed structural facts, then the proved
/// relations at this exact store BCI — and **no** canonical platform floor. A platform table
/// alone is not an initializer proof (pinned by `build.rs`'s
/// `initializer_reference_widening_reuses_scalar_and_equal_rank_array_facts`): the
/// invocation-argument floor admits hierarchy rows the snapshot header never proved for this
/// store, while an initializer element may widen only where Java's own shape says so or where
/// the header walk proved this exact pair at this exact store.
pub(crate) fn initializer_reference_assignable(
    java_release: u16,
    source: &str,
    target: &str,
    bci: u32,
    relations: &[crate::report::ReferenceRelationInput],
) -> bool {
    if structurally_assignable(source, target) {
        return true;
    }
    proved_reference_relations(java_release, source, target, bci, relations)
}

/// The structural head both floors share: identity, the common supertype of every reference,
/// and Java's closed array-shape rules.
fn structurally_assignable(source: &str, target: &str) -> bool {
    source == target || target == "java.lang.Object" || array_reference_widens(source, target)
}

/// The proved-relations walk both floors share: relations matching this exact site decide — a
/// non-assignable relation at the site refuses decisively, a proved one carries the answer.
fn proved_reference_relations(
    java_release: u16,
    source: &str,
    target: &str,
    bci: u32,
    relations: &[crate::report::ReferenceRelationInput],
) -> bool {
    let mut proved = false;
    for proof in relations
        .iter()
        .filter(|proof| proof.bci == bci && proof.source == source && proof.target == target)
    {
        if !proof.relation.is_assignable(java_release) {
            return false;
        }
        proved = true;
    }
    proved
}

/// The transcribed release-8 platform rows the engine states as its own canonical hierarchy:
/// the `java.util` collection tree, the `java.lang` Throwable tree, and the platform interface
/// and array families. Every row is receipted under `openspec/evidence/java-syntax-2026-10-05/`
/// (the `widening-row-sources` javap transcription of Corretto 1.8.0_432 `rt.jar`) and pinned by
/// the `build.rs` unit tests; no release other than 8 has a receipt, so every caller of this
/// fold passes `8` — a caller with a live release context gates on it.
fn canonical_platform_rows(source: &str, target: &str) -> bool {
    release_reference_argument_widens(8, source, target)
        || java_lang_throwable_widens(source, target)
        || platform_interface_argument_widens(8, source, target)
        || platform_array_argument_widens(8, source, target)
}

pub(crate) fn reference_intrinsically_assignable(source: &str, target: &str) -> bool {
    if structurally_assignable(source, target) {
        return true;
    }
    // The canonical platform rows are the rest of the intrinsic floor. A caller without a
    // release context — the local declaration planner's write-compatibility closure — admits
    // exactly the transcribed hierarchy this engine states, releaseless by construction: the
    // planner's candidate domains are frame/descriptor facts the class file itself carries,
    // not an environment answer a release could contradict.
    canonical_platform_rows(source, target)
}

/// Whether one presented array type can be widened to a proven array type or array supertype using
/// only Java's closed array component rules. Class and interface inheritance is deliberately not
/// consulted here.
///
/// Both names are Java spellings read from existing type facts; this helper only peels their array
/// suffixes. A descriptor can contain at most 255 array dimensions, which bounds the component
/// walk.
pub(crate) fn array_reference_widens(presented: &str, required: &str) -> bool {
    const MAX_ARRAY_DIMENSIONS: usize = 255;

    fn array_parts(name: &str) -> Option<(&str, usize)> {
        let mut base = name;
        let mut dimensions = 0usize;
        while let Some(component) = base.strip_suffix("[]") {
            dimensions += 1;
            if dimensions > MAX_ARRAY_DIMENSIONS {
                return None;
            }
            base = component;
        }
        if dimensions == 0 || base.is_empty() || base == "void" {
            return None;
        }
        Some((base, dimensions))
    }

    fn primitive(name: &str) -> bool {
        matches!(
            name,
            "boolean" | "byte" | "char" | "short" | "int" | "long" | "float" | "double"
        )
    }

    let Some((presented_base, mut presented_dimensions)) = array_parts(presented) else {
        return false;
    };
    let (required_base, mut required_dimensions) = match array_parts(required) {
        Some((base, dimensions)) => (base, dimensions),
        // Every array type implements these two marker interfaces. A class component with one of
        // these names is not enough evidence: the marker shortcut applies only to an array value.
        None if matches!(required, "java.lang.Cloneable" | "java.io.Serializable") => {
            return true;
        }
        None => return false,
    };

    loop {
        // The current outer arrays have been reduced to their component types. An array component
        // is a reference even when its own element type is primitive (`int[]` is an Object).
        let presented_component_is_array = presented_dimensions > 1;
        let required_component_is_array = required_dimensions > 1;
        let presented_component_is_reference =
            presented_component_is_array || !primitive(presented_base);

        if !required_component_is_array
            && required_base == "java.lang.Object"
            && presented_component_is_reference
        {
            return true;
        }

        if !required_component_is_array
            && matches!(
                required_base,
                "java.lang.Cloneable" | "java.io.Serializable"
            )
            && presented_component_is_array
        {
            return true;
        }

        if presented_component_is_array && required_component_is_array {
            presented_dimensions -= 1;
            required_dimensions -= 1;
            continue;
        }

        if !presented_component_is_array && !required_component_is_array {
            // Primitive components are invariant; reference components are admitted only when
            // identical here. The Object and marker cases above are the sole widenings proved by
            // array shape.
            return presented_base == required_base;
        }

        // An array component cannot become an arbitrary class/interface component, and an
        // arbitrary class/interface component cannot become an array component.
        return false;
    }
}

/// The array position of the platform tables above: whether one presented **array** type can be
/// widened to a required array type whose deepest reference components are a pair one of those
/// closed tables proves. `String[]` at a `CharSequence[]` position is the shape
/// `String.join(CharSequence, CharSequence...)` writes for a `String[]` argument, and
/// `String[][]` at a `CharSequence[][]` position is the same relation one rank deeper.
///
/// The array shapes must agree rank for rank — a primitive component never stands in for a
/// reference one, and an array component never meets a class component — exactly as
/// [`array_reference_widens`] states them; this predicate then states no fact of its own, it only
/// asks these tables the scalar positions ask. The two channels that answered arrays before this
/// change are deliberately **not** consulted here: [`release_reference_argument_widens`] and
/// [`java_lang_throwable_widens`] keep the array answer they always stated, so an
/// `IllegalStateException[]` argument at a `Throwable[]` position stays refused exactly as before.
// The invocation-argument floor reads these transcribed release-8 rows: a site no proved
// relation states is answered from the canonical rows alone, and the `build.rs` unit tests keep
// pinning every row (evidence receipts under `openspec/evidence/java-syntax-2026-10-05/`).
pub(crate) fn platform_array_argument_widens(
    java_release: u16,
    presented: &str,
    required: &str,
) -> bool {
    const MAX_ARRAY_DIMENSIONS: usize = 255;

    let mut presented = presented;
    let mut required = required;
    for _ in 0..MAX_ARRAY_DIMENSIONS {
        let (Some(presented_component), Some(required_component)) =
            (presented.strip_suffix("[]"), required.strip_suffix("[]"))
        else {
            return false;
        };
        if presented_component.is_empty() || required_component.is_empty() {
            return false;
        }
        let presented_is_array = presented_component.ends_with("[]");
        let required_is_array = required_component.ends_with("[]");
        if presented_is_array != required_is_array {
            return false;
        }
        if !presented_is_array {
            return platform_interface_argument_widens(
                java_release,
                presented_component,
                required_component,
            );
        }
        presented = presented_component;
        required = required_component;
    }
    false
}

/// The Java 8 `java.util` collection hierarchy one invocation argument may cross, stated as the
/// fixed direct edges of the release-8 tree, plus the two `java.io` reader edges the three-layer
/// stream-construction chain states. A class row is a javadoc `extends` (one superclass) or
/// `implements` (one interface) relation of that class; an interface row is a javadoc `extends`
/// relation of that interface. The walk follows the rows to their roots, so the answer is exactly
/// the ancestors this closed set reaches — `List -> Collection -> Iterable`, `TreeSet ->
/// NavigableSet -> SortedSet -> Set -> Collection`, and the like — and nothing outside the
/// enumerated `java.util` collection types (`java.util.concurrent`, the `Collections`/`Arrays`
/// factories, `EnumSet`/`IdentityHashMap`/`PriorityQueue` and the non-collection `java.util` types
/// such as `Dictionary`/`Date`, plus every user class), and nothing outside the two `java.io`
/// edges' own ancestors, widens until a resolution-environment hierarchy proof exists. The same
/// channel already states the `java.lang` Throwable tree ([`java_lang_throwable_widens`]); this is
/// its `java.util` counterpart.
///
/// The two `java.io` rows are the third layer of the wrapped-stream construction: a
/// `BufferedReader` over an `InputStreamReader` over a `FileInputStream` presents each inner
/// stream at the next constructor's own parameter type. Both rows are `javap` transcriptions of
/// the release-8 library (`InputStreamReader`'s constructor is `(java.io.InputStream,
/// java.lang.String)` and `BufferedReader`'s is `(java.io.Reader)`, and each class's own header
/// names the direct superclass the row states); the transcription and its self-check live in
/// `openspec/evidence/java-syntax-2026-10-05/widening-row-sources/`. The set stays closed: a
/// presented type widens exactly to the ancestors these declared edges reach, and no other
/// `java.io` type states an edge here, so `java.io.FileReader` (a subclass of `InputStreamReader`)
/// and every other reader stay refused until their own row is transcribed.
pub(crate) fn release_reference_argument_widens(
    java_release: u16,
    presented: &str,
    required: &str,
) -> bool {
    // One row per documented direct `extends`/`implements` of the release-8 `java.util` collection
    // tree. A row's two names are the javadoc's own declaration of that type.
    const DIRECT_EDGES: &[(&str, &str)] = &[
        // The `Abstract*` skeletons the concrete classes are built on.
        ("java.util.AbstractCollection", "java.util.Collection"),
        ("java.util.AbstractList", "java.util.AbstractCollection"),
        ("java.util.AbstractList", "java.util.List"),
        ("java.util.AbstractSequentialList", "java.util.AbstractList"),
        ("java.util.AbstractSet", "java.util.AbstractCollection"),
        ("java.util.AbstractSet", "java.util.Set"),
        ("java.util.AbstractMap", "java.util.Map"),
        // The `List` implementations.
        ("java.util.ArrayList", "java.util.AbstractList"),
        ("java.util.ArrayList", "java.util.List"),
        ("java.util.LinkedList", "java.util.AbstractSequentialList"),
        ("java.util.LinkedList", "java.util.List"),
        ("java.util.LinkedList", "java.util.Deque"),
        ("java.util.Vector", "java.util.AbstractList"),
        ("java.util.Vector", "java.util.List"),
        ("java.util.Stack", "java.util.Vector"),
        // The `Set` implementations.
        ("java.util.HashSet", "java.util.AbstractSet"),
        ("java.util.HashSet", "java.util.Set"),
        ("java.util.LinkedHashSet", "java.util.HashSet"),
        ("java.util.LinkedHashSet", "java.util.Set"),
        ("java.util.TreeSet", "java.util.AbstractSet"),
        ("java.util.TreeSet", "java.util.NavigableSet"),
        // The `Map` implementations. `Hashtable`'s other javadoc parent is `java.util.Dictionary`,
        // which is not a collection type: no row states it, so a `Dictionary` slot stays refused.
        ("java.util.HashMap", "java.util.AbstractMap"),
        ("java.util.HashMap", "java.util.Map"),
        ("java.util.LinkedHashMap", "java.util.HashMap"),
        ("java.util.LinkedHashMap", "java.util.Map"),
        ("java.util.Hashtable", "java.util.Map"),
        ("java.util.Properties", "java.util.Hashtable"),
        ("java.util.TreeMap", "java.util.AbstractMap"),
        ("java.util.TreeMap", "java.util.NavigableMap"),
        // The `Deque` implementation and its skeleton.
        ("java.util.ArrayDeque", "java.util.AbstractCollection"),
        ("java.util.ArrayDeque", "java.util.Deque"),
        // The interface chain itself, down to the `Iterable` root.
        ("java.util.Collection", "java.lang.Iterable"),
        ("java.util.List", "java.util.Collection"),
        ("java.util.Queue", "java.util.Collection"),
        ("java.util.Deque", "java.util.Queue"),
        ("java.util.Set", "java.util.Collection"),
        ("java.util.SortedSet", "java.util.Set"),
        ("java.util.NavigableSet", "java.util.SortedSet"),
        ("java.util.SortedMap", "java.util.Map"),
        ("java.util.NavigableMap", "java.util.SortedMap"),
        // The two `java.io` edges of the three-layer stream chain: `FileInputStream` at
        // `InputStreamReader(InputStream, String)` and `InputStreamReader` at
        // `BufferedReader(Reader)`. Each is the class's own `extends` clause (javap,
        // release 8), transcribed with its header in the widening-row-sources protocol file.
        ("java.io.FileInputStream", "java.io.InputStream"),
        ("java.io.InputStreamReader", "java.io.Reader"),
    ];
    // The rows are one acyclic superclass/interface graph, so the walk from the presented type
    // reaches every ancestor the table proves; the step cap only bounds a corrupted table the same
    // way the array walk bounds a corrupted rank, and each parent is compared as it is appended, so
    // the answer is the widening relation alone — a presented type equal to the required one is the
    // dispatch's same-name answer, never this table's.
    if java_release != 8 {
        return false;
    }
    let mut frontier: Vec<&str> = vec![presented];
    for _ in 0..DIRECT_EDGES.len() {
        let mut next: Vec<&str> = Vec::new();
        for current in &frontier {
            for (child, parent) in DIRECT_EDGES {
                if child == current {
                    if *parent == required {
                        return true;
                    }
                    next.push(parent);
                }
            }
        }
        if next.is_empty() {
            return false;
        }
        frontier = next;
    }
    false
}

/// The JDK 8 `java.lang` Throwable family one invocation argument may cross. The JVM verifier
/// already admitted the argument's value to the invoked descriptor's parameter, so an exception
/// value a `Throwable` slot receives (a wrap-and-rethrow's cause, a forwarding method's caught
/// exception) cannot fail the widening on either side — the relation is the platform's own fixed
/// class hierarchy, the same closed knowledge [`release_reference_argument_widens`] states for
/// its one pair. Every row is one direct superclass edge of the release-8 `java.lang` tree, kept
/// spelled out for review; the walk admits exactly the ancestors these rows reach, and nothing
/// outside `java.lang` (the `java.io`, `java.util` and user subclasses of the same family state
/// no row here) widens until a resolution-environment hierarchy proof exists.
// The invocation-argument floor reads these transcribed release-8 rows: a site no proved
// relation states is answered from the canonical rows alone, and the `build.rs` unit tests keep
// pinning every row (evidence receipts under `openspec/evidence/java-syntax-2026-10-05/`).
pub(crate) fn java_lang_throwable_widens(presented: &str, required: &str) -> bool {
    // (class, direct superclass), one row per documented `extends` of the JDK 8 java.lang
    // Throwable tree. A row's two names are the javadoc's own declaration of that class.
    const DIRECT_EDGES: &[(&str, &str)] = &[
        // The two roots of the checked and unchecked halves.
        ("java.lang.Exception", "java.lang.Throwable"),
        ("java.lang.Error", "java.lang.Throwable"),
        // The unchecked half under RuntimeException.
        ("java.lang.RuntimeException", "java.lang.Exception"),
        (
            "java.lang.IllegalStateException",
            "java.lang.RuntimeException",
        ),
        (
            "java.lang.IllegalArgumentException",
            "java.lang.RuntimeException",
        ),
        (
            "java.lang.NullPointerException",
            "java.lang.RuntimeException",
        ),
        ("java.lang.ClassCastException", "java.lang.RuntimeException"),
        (
            "java.lang.ArrayStoreException",
            "java.lang.RuntimeException",
        ),
        (
            "java.lang.ArithmeticException",
            "java.lang.RuntimeException",
        ),
        (
            "java.lang.IndexOutOfBoundsException",
            "java.lang.RuntimeException",
        ),
        (
            "java.lang.NegativeArraySizeException",
            "java.lang.RuntimeException",
        ),
        (
            "java.lang.IllegalMonitorStateException",
            "java.lang.RuntimeException",
        ),
        ("java.lang.SecurityException", "java.lang.RuntimeException"),
        (
            "java.lang.UnsupportedOperationException",
            "java.lang.RuntimeException",
        ),
        (
            "java.lang.TypeNotPresentException",
            "java.lang.RuntimeException",
        ),
        (
            "java.lang.EnumConstantNotPresentException",
            "java.lang.RuntimeException",
        ),
        // The checked half under Exception.
        (
            "java.lang.CloneNotSupportedException",
            "java.lang.Exception",
        ),
        ("java.lang.InterruptedException", "java.lang.Exception"),
        (
            "java.lang.ReflectiveOperationException",
            "java.lang.Exception",
        ),
        (
            "java.lang.ClassNotFoundException",
            "java.lang.ReflectiveOperationException",
        ),
        // The two documented IllegalArgumentException subclasses.
        (
            "java.lang.NumberFormatException",
            "java.lang.IllegalArgumentException",
        ),
        (
            "java.lang.IllegalThreadStateException",
            "java.lang.IllegalArgumentException",
        ),
        // The two documented IndexOutOfBoundsException subclasses.
        (
            "java.lang.ArrayIndexOutOfBoundsException",
            "java.lang.IndexOutOfBoundsException",
        ),
        (
            "java.lang.StringIndexOutOfBoundsException",
            "java.lang.IndexOutOfBoundsException",
        ),
        // The Error tree: the roots of its three branches, then each branch's own subclasses.
        ("java.lang.ThreadDeath", "java.lang.Error"),
        ("java.lang.AssertionError", "java.lang.Error"),
        ("java.lang.LinkageError", "java.lang.Error"),
        ("java.lang.VirtualMachineError", "java.lang.Error"),
        ("java.lang.ClassCircularityError", "java.lang.LinkageError"),
        ("java.lang.ClassFormatError", "java.lang.LinkageError"),
        (
            "java.lang.ExceptionInInitializerError",
            "java.lang.LinkageError",
        ),
        ("java.lang.NoClassDefFoundError", "java.lang.LinkageError"),
        ("java.lang.UnsatisfiedLinkError", "java.lang.LinkageError"),
        ("java.lang.VerifyError", "java.lang.LinkageError"),
        ("java.lang.BootstrapMethodError", "java.lang.LinkageError"),
        (
            "java.lang.IncompatibleClassChangeError",
            "java.lang.LinkageError",
        ),
        (
            "java.lang.AbstractMethodError",
            "java.lang.IncompatibleClassChangeError",
        ),
        (
            "java.lang.IllegalAccessError",
            "java.lang.IncompatibleClassChangeError",
        ),
        (
            "java.lang.InstantiationError",
            "java.lang.IncompatibleClassChangeError",
        ),
        (
            "java.lang.NoSuchFieldError",
            "java.lang.IncompatibleClassChangeError",
        ),
        (
            "java.lang.NoSuchMethodError",
            "java.lang.IncompatibleClassChangeError",
        ),
        ("java.lang.InternalError", "java.lang.VirtualMachineError"),
        (
            "java.lang.OutOfMemoryError",
            "java.lang.VirtualMachineError",
        ),
        (
            "java.lang.StackOverflowError",
            "java.lang.VirtualMachineError",
        ),
        ("java.lang.UnknownError", "java.lang.VirtualMachineError"),
    ];
    // The rows are one acyclic superclass chain, so the walk from the presented class reaches
    // every ancestor the table proves; the step cap only bounds a corrupted table the same way
    // the array walk bounds a corrupted rank. The parent is compared after each step, so the
    // answer is the widening relation alone: a presented type equal to the required one is the
    // dispatch's same-name answer, never this table's.
    let mut current = presented;
    for _ in 0..DIRECT_EDGES.len() {
        let Some((_, parent)) = DIRECT_EDGES.iter().find(|(child, _)| *child == current) else {
            return false;
        };
        current = parent;
        if current == required {
            return true;
        }
    }
    false
}

/// The release-8 implementer sets and superclasses of the platform types the two tables above do
/// not state: the `java.lang` interfaces `CharSequence` and `Comparable`, the `java.lang.Number`
/// superclass of the six boxed numeric classes, `java.io.Serializable`, and the enum family's own
/// `java.util.EnumSet` collection type.
///
/// The JVM verifier already admitted the argument's value to the invoked descriptor's parameter,
/// so a value one of these rows names cannot fail the widening: every row is the platform's own
/// fixed relation between two names the JDK ships, stated exactly as the release-8 javadoc
/// declares it — a class row is the implementer's own declaration of that interface, or the
/// superclass whose declaration carries it — and a pair no row states does not widen. The answer is
/// therefore the same closed knowledge [`release_reference_argument_widens`] states for its
/// `java.util` tree and [`java_lang_throwable_widens`] for its `java.lang` Throwable family, and a
/// caller renders a hit exactly as it renders theirs.
///
/// An enum constant's value needs no row here: the enum class is a class of the analyzed snapshot,
/// and its own class-file header names `java.lang.Enum`, which the snapshot hierarchy proof already
/// states. Only the platform type a snapshot does not hold is tabled.
///
/// Two further families are stated the same way, both transcribed from the same release-8 `rt.jar`
/// (`openspec/evidence/java-syntax-2026-10-05/widening-row-sources/`, the 2026-10-06 rows): the
/// `java.time` Temporal family (its seven named types, each under `java.time.temporal.Temporal` and
/// `java.time.temporal.TemporalAccessor` — the second read through the first's own header, the
/// chain the `TEMPORAL_FAMILY` row note spells out) and `java.util.concurrent.CompletableFuture`'s
/// two interfaces. Their rows, their gate and the render are this function's, so a `LocalDateTime`
/// at a `TemporalAccessor` slot and a `CompletableFuture` at a `CompletionStage` slot are answered
/// exactly as a `String` at a `CharSequence` slot is.
///
/// The boxed-number family the 2026-10-07 slice states is the `java.lang.Number` edge of the same
/// six names the `COMPARABLE` and `SERIALIZABLE` tables already carry, transcribed from the same
/// release-8 `rt.jar` (the six headers' own `extends java.lang.Number` clause, the lines
/// `openspec/evidence/java-syntax-2026-10-05/widening-row-sources/javap-headers.txt` already
/// holds). The release-8 direct-child check
/// (`openspec/changes/recover-boxed-number-widening/results/probe/number-universe.out`) found
/// exactly those six direct `java.lang` subclasses, with no intermediate `java.lang` class. This
/// table additionally carries the separately verified `BigDecimal -> Number` header edge from
/// `release8-header-root-v1.json`. `BigInteger` and the `java.util.concurrent.atomic` subclasses
/// state no row; a pair no row states keeps its refusal.
// The invocation-argument floor reads these transcribed release-8 rows: a site no proved
// relation states is answered from the canonical rows alone, and the `build.rs` unit tests keep
// pinning every row (evidence receipts under `openspec/evidence/java-syntax-2026-10-05/`).
pub(crate) fn platform_interface_argument_widens(
    java_release: u16,
    presented: &str,
    required: &str,
) -> bool {
    // `java.lang.CharSequence`'s implementers: `String`, `StringBuffer` and `StringBuilder`
    // declare the interface in their own headers, and so does `java.nio.CharBuffer`. The closed
    // four rows are the release-8 javadoc's set this change pins; the same javadoc's implementer
    // list also names `javax.swing.text.Segment`, which states no row here, so that pair keeps its
    // refusal rather than widening from memory.
    const CHAR_SEQUENCE: &[(&str, &str)] = &[
        ("java.lang.String", "java.lang.CharSequence"),
        ("java.lang.StringBuffer", "java.lang.CharSequence"),
        ("java.lang.StringBuilder", "java.lang.CharSequence"),
        ("java.nio.CharBuffer", "java.lang.CharSequence"),
    ];
    // `java.lang.Comparable`'s `java.lang` implementers: `String` and the eight boxed types, each
    // of which declares `Comparable<itself>` in its own header. The `java.math` and other JDK
    // implementers state no row (this table is the `java.lang` set).
    const COMPARABLE: &[(&str, &str)] = &[
        ("java.lang.String", "java.lang.Comparable"),
        ("java.lang.Byte", "java.lang.Comparable"),
        ("java.lang.Short", "java.lang.Comparable"),
        ("java.lang.Integer", "java.lang.Comparable"),
        ("java.lang.Long", "java.lang.Comparable"),
        ("java.lang.Float", "java.lang.Comparable"),
        ("java.lang.Double", "java.lang.Comparable"),
        ("java.lang.Character", "java.lang.Comparable"),
        ("java.lang.Boolean", "java.lang.Comparable"),
    ];
    // `java.io.Serializable`'s `java.lang` implementers of the same nine names: `String`,
    // `Character` and `Boolean` declare the interface in their own headers, and the six `Number`
    // subclasses reach it through the `java.lang.Number` header that declares it — the javadoc's
    // own implemented-interface list, read the way this file's java.util table reads
    // `Properties -> java.util.Map` off `Hashtable`.
    const SERIALIZABLE: &[(&str, &str)] = &[
        ("java.lang.String", "java.io.Serializable"),
        ("java.lang.Byte", "java.io.Serializable"),
        ("java.lang.Short", "java.io.Serializable"),
        ("java.lang.Integer", "java.io.Serializable"),
        ("java.lang.Long", "java.io.Serializable"),
        ("java.lang.Float", "java.io.Serializable"),
        ("java.lang.Double", "java.io.Serializable"),
        ("java.lang.Character", "java.io.Serializable"),
        ("java.lang.Boolean", "java.io.Serializable"),
    ];
    // `java.lang.Number`'s direct children pinned here: the six boxed numbers and BigDecimal. The
    // six boxed headers and the Corretto 8 `java.math.BigDecimal` header each state
    // `extends java.lang.Number`; the latter is pinned by
    // `openspec/changes/recover-bigdecimal-number-widening/results/release8-header-root-v1.json`
    // (rt.jar SHA-256 `b27515a608ee447566b688e2bbb2257b1f0d8eceb96c307b87eb28d90a6630f4`).
    // BigInteger and the `java.util.concurrent.atomic` family remain outside this closed set.
    const NUMBER_FAMILY: &[(&str, &str)] = &[
        ("java.lang.Byte", "java.lang.Number"),
        ("java.lang.Short", "java.lang.Number"),
        ("java.lang.Integer", "java.lang.Number"),
        ("java.lang.Long", "java.lang.Number"),
        ("java.lang.Float", "java.lang.Number"),
        ("java.lang.Double", "java.lang.Number"),
        ("java.math.BigDecimal", "java.lang.Number"),
    ];
    // The enum family's collection type: `java.util.EnumSet` extends `java.util.AbstractSet`, and
    // the javadoc's implemented-interface list reaches `Set`, `Collection` and `Iterable` — the
    // rows the `EnumSet.retainAll(Collection)` argument position of an `EnumSet` parameter needs.
    // `EnumSet`'s `Cloneable`/`Serializable` relations state no row: the collection positions are
    // the ones the enum change pins.
    const ENUM_FAMILY: &[(&str, &str)] = &[
        ("java.util.EnumSet", "java.util.AbstractSet"),
        ("java.util.EnumSet", "java.util.Set"),
        ("java.util.EnumSet", "java.util.Collection"),
        ("java.util.EnumSet", "java.lang.Iterable"),
    ];
    // The `java.time` Temporal family: every one of the seven names declares
    // `java.time.temporal.Temporal` in its own header, and the `TemporalAccessor` row of each name
    // is that interface's own declaration (`java.time.temporal.Temporal extends
    // java.time.temporal.TemporalAccessor`) — the javadoc-style read of an implemented-interface
    // list through the header that carries it, the way this file's java.util table reads
    // `Properties -> java.util.Map` off `Hashtable`. The chain is the same for all seven:
    //
    //   java.time.<name> --implements--> java.time.temporal.Temporal
    //                    --extends-->    java.time.temporal.TemporalAccessor
    //
    // The closed fourteen rows are the ones the change pins after the header check: the MVP's
    // `LocalDateTime`/`LocalDate`/`LocalTime` and the four the proposal admits (`Instant`,
    // `ZonedDateTime`, `OffsetDateTime`, `OffsetTime`). The interface family's other platform
    // implementers (`java.time.chrono.*`, the adjuster types) state no row here.
    const TEMPORAL_FAMILY: &[(&str, &str)] = &[
        ("java.time.LocalDateTime", "java.time.temporal.Temporal"),
        (
            "java.time.LocalDateTime",
            "java.time.temporal.TemporalAccessor",
        ),
        ("java.time.LocalDate", "java.time.temporal.Temporal"),
        ("java.time.LocalDate", "java.time.temporal.TemporalAccessor"),
        ("java.time.LocalTime", "java.time.temporal.Temporal"),
        ("java.time.LocalTime", "java.time.temporal.TemporalAccessor"),
        ("java.time.Instant", "java.time.temporal.Temporal"),
        ("java.time.Instant", "java.time.temporal.TemporalAccessor"),
        ("java.time.ZonedDateTime", "java.time.temporal.Temporal"),
        (
            "java.time.ZonedDateTime",
            "java.time.temporal.TemporalAccessor",
        ),
        ("java.time.OffsetDateTime", "java.time.temporal.Temporal"),
        (
            "java.time.OffsetDateTime",
            "java.time.temporal.TemporalAccessor",
        ),
        ("java.time.OffsetTime", "java.time.temporal.Temporal"),
        (
            "java.time.OffsetTime",
            "java.time.temporal.TemporalAccessor",
        ),
    ];
    // The batch-2 rows the CompletableFuture patrol registered: one header line declares both
    // interfaces (`CompletableFuture<T> implements Future<T>, CompletionStage<T>`), so both are
    // stated and the rest of the `java.util.concurrent` tree is not.
    const COMPLETABLE_FUTURE: &[(&str, &str)] = &[
        (
            "java.util.concurrent.CompletableFuture",
            "java.util.concurrent.CompletionStage",
        ),
        (
            "java.util.concurrent.CompletableFuture",
            "java.util.concurrent.Future",
        ),
    ];
    // Every row is a release-8 javadoc relation, so the gate the java.util table states for its own
    // rows applies here too: another release's platform facts are not this table's.
    if java_release != 8 {
        return false;
    }
    [
        CHAR_SEQUENCE,
        COMPARABLE,
        SERIALIZABLE,
        NUMBER_FAMILY,
        ENUM_FAMILY,
        TEMPORAL_FAMILY,
        COMPLETABLE_FUTURE,
    ]
    .iter()
    .any(|rows| rows.contains(&(presented, required)))
}

// Direct parent facts from Corretto 8 rt.jar, SHA256
// b27515a608ee447566b688e2bbb2257b1f0d8eceb96c307b87eb28d90a6630f4.
// Source/header pins: results/s2a-variable-legalization-v1/platform-parents-v1.
// Each node is consulted only after the environment establishes clean absence.
const JAVA8_DIRECT_PARENTS: &[(&str, &[&str])] = &[
    (
        "java/io/Closeable",
        &["java/lang/Object", "java/lang/AutoCloseable"],
    ),
    ("java/io/FileInputStream", &["java/io/InputStream"]),
    (
        "java/io/InputStream",
        &["java/lang/Object", "java/io/Closeable"],
    ),
    ("java/io/InputStreamReader", &["java/io/Reader"]),
    (
        "java/io/Reader",
        &[
            "java/lang/Object",
            "java/lang/Readable",
            "java/io/Closeable",
        ],
    ),
    ("java/io/Serializable", &["java/lang/Object"]),
    (
        "java/lang/AbstractMethodError",
        &["java/lang/IncompatibleClassChangeError"],
    ),
    (
        "java/lang/AbstractStringBuilder",
        &[
            "java/lang/Object",
            "java/lang/Appendable",
            "java/lang/CharSequence",
        ],
    ),
    ("java/lang/Appendable", &["java/lang/Object"]),
    (
        "java/lang/ArithmeticException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/ArrayIndexOutOfBoundsException",
        &["java/lang/IndexOutOfBoundsException"],
    ),
    (
        "java/lang/ArrayStoreException",
        &["java/lang/RuntimeException"],
    ),
    ("java/lang/AssertionError", &["java/lang/Error"]),
    ("java/lang/AutoCloseable", &["java/lang/Object"]),
    (
        "java/lang/Boolean",
        &[
            "java/lang/Object",
            "java/io/Serializable",
            "java/lang/Comparable",
        ],
    ),
    (
        "java/lang/BootstrapMethodError",
        &["java/lang/LinkageError"],
    ),
    (
        "java/lang/Byte",
        &["java/lang/Number", "java/lang/Comparable"],
    ),
    ("java/lang/CharSequence", &["java/lang/Object"]),
    (
        "java/lang/Character",
        &[
            "java/lang/Object",
            "java/io/Serializable",
            "java/lang/Comparable",
        ],
    ),
    (
        "java/lang/ClassCastException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/ClassCircularityError",
        &["java/lang/LinkageError"],
    ),
    ("java/lang/ClassFormatError", &["java/lang/LinkageError"]),
    (
        "java/lang/ClassNotFoundException",
        &["java/lang/ReflectiveOperationException"],
    ),
    (
        "java/lang/CloneNotSupportedException",
        &["java/lang/Exception"],
    ),
    ("java/lang/Cloneable", &["java/lang/Object"]),
    ("java/lang/Comparable", &["java/lang/Object"]),
    (
        "java/lang/Double",
        &["java/lang/Number", "java/lang/Comparable"],
    ),
    (
        "java/lang/Enum",
        &[
            "java/lang/Object",
            "java/lang/Comparable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/lang/EnumConstantNotPresentException",
        &["java/lang/RuntimeException"],
    ),
    ("java/lang/Error", &["java/lang/Throwable"]),
    ("java/lang/Exception", &["java/lang/Throwable"]),
    (
        "java/lang/ExceptionInInitializerError",
        &["java/lang/LinkageError"],
    ),
    (
        "java/lang/Float",
        &["java/lang/Number", "java/lang/Comparable"],
    ),
    (
        "java/lang/IllegalAccessError",
        &["java/lang/IncompatibleClassChangeError"],
    ),
    (
        "java/lang/IllegalArgumentException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/IllegalMonitorStateException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/IllegalStateException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/IllegalThreadStateException",
        &["java/lang/IllegalArgumentException"],
    ),
    (
        "java/lang/IncompatibleClassChangeError",
        &["java/lang/LinkageError"],
    ),
    (
        "java/lang/IndexOutOfBoundsException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/InstantiationError",
        &["java/lang/IncompatibleClassChangeError"],
    ),
    (
        "java/lang/Integer",
        &["java/lang/Number", "java/lang/Comparable"],
    ),
    (
        "java/lang/InternalError",
        &["java/lang/VirtualMachineError"],
    ),
    ("java/lang/InterruptedException", &["java/lang/Exception"]),
    ("java/lang/Iterable", &["java/lang/Object"]),
    ("java/lang/LinkageError", &["java/lang/Error"]),
    (
        "java/lang/Long",
        &["java/lang/Number", "java/lang/Comparable"],
    ),
    (
        "java/lang/NegativeArraySizeException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/NoClassDefFoundError",
        &["java/lang/LinkageError"],
    ),
    (
        "java/lang/NoSuchFieldError",
        &["java/lang/IncompatibleClassChangeError"],
    ),
    (
        "java/lang/NoSuchMethodError",
        &["java/lang/IncompatibleClassChangeError"],
    ),
    (
        "java/lang/NullPointerException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/Number",
        &["java/lang/Object", "java/io/Serializable"],
    ),
    (
        "java/lang/NumberFormatException",
        &["java/lang/IllegalArgumentException"],
    ),
    ("java/lang/Object", &[]),
    (
        "java/lang/OutOfMemoryError",
        &["java/lang/VirtualMachineError"],
    ),
    ("java/lang/Readable", &["java/lang/Object"]),
    (
        "java/lang/ReflectiveOperationException",
        &["java/lang/Exception"],
    ),
    ("java/lang/Runnable", &["java/lang/Object"]),
    ("java/lang/RuntimeException", &["java/lang/Exception"]),
    (
        "java/lang/SecurityException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/Short",
        &["java/lang/Number", "java/lang/Comparable"],
    ),
    (
        "java/lang/StackOverflowError",
        &["java/lang/VirtualMachineError"],
    ),
    (
        "java/lang/String",
        &[
            "java/lang/Object",
            "java/io/Serializable",
            "java/lang/Comparable",
            "java/lang/CharSequence",
        ],
    ),
    (
        "java/lang/StringBuffer",
        &[
            "java/lang/AbstractStringBuilder",
            "java/io/Serializable",
            "java/lang/CharSequence",
        ],
    ),
    (
        "java/lang/StringBuilder",
        &[
            "java/lang/AbstractStringBuilder",
            "java/io/Serializable",
            "java/lang/CharSequence",
        ],
    ),
    (
        "java/lang/StringIndexOutOfBoundsException",
        &["java/lang/IndexOutOfBoundsException"],
    ),
    ("java/lang/ThreadDeath", &["java/lang/Error"]),
    (
        "java/lang/Throwable",
        &["java/lang/Object", "java/io/Serializable"],
    ),
    (
        "java/lang/TypeNotPresentException",
        &["java/lang/RuntimeException"],
    ),
    ("java/lang/UnknownError", &["java/lang/VirtualMachineError"]),
    (
        "java/lang/UnsatisfiedLinkError",
        &["java/lang/LinkageError"],
    ),
    (
        "java/lang/UnsupportedOperationException",
        &["java/lang/RuntimeException"],
    ),
    ("java/lang/VerifyError", &["java/lang/LinkageError"]),
    ("java/lang/VirtualMachineError", &["java/lang/Error"]),
    (
        "java/math/BigDecimal",
        &["java/lang/Number", "java/lang/Comparable"],
    ),
    ("java/nio/Buffer", &["java/lang/Object"]),
    (
        "java/nio/CharBuffer",
        &[
            "java/nio/Buffer",
            "java/lang/Comparable",
            "java/lang/Appendable",
            "java/lang/CharSequence",
            "java/lang/Readable",
        ],
    ),
    (
        "java/time/Instant",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/LocalDate",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/time/chrono/ChronoLocalDate",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/LocalDateTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/time/chrono/ChronoLocalDateTime",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/LocalTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/OffsetDateTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/OffsetTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/ZonedDateTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/chrono/ChronoZonedDateTime",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/chrono/ChronoLocalDate",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
        ],
    ),
    (
        "java/time/chrono/ChronoLocalDateTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
        ],
    ),
    (
        "java/time/chrono/ChronoZonedDateTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/lang/Comparable",
        ],
    ),
    (
        "java/time/temporal/Temporal",
        &["java/lang/Object", "java/time/temporal/TemporalAccessor"],
    ),
    ("java/time/temporal/TemporalAccessor", &["java/lang/Object"]),
    ("java/time/temporal/TemporalAdjuster", &["java/lang/Object"]),
    (
        "java/util/AbstractCollection",
        &["java/lang/Object", "java/util/Collection"],
    ),
    (
        "java/util/AbstractList",
        &["java/util/AbstractCollection", "java/util/List"],
    ),
    (
        "java/util/AbstractMap",
        &["java/lang/Object", "java/util/Map"],
    ),
    (
        "java/util/AbstractSequentialList",
        &["java/util/AbstractList"],
    ),
    (
        "java/util/AbstractSet",
        &["java/util/AbstractCollection", "java/util/Set"],
    ),
    (
        "java/util/ArrayDeque",
        &[
            "java/util/AbstractCollection",
            "java/util/Deque",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/ArrayList",
        &[
            "java/util/AbstractList",
            "java/util/List",
            "java/util/RandomAccess",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/Collection",
        &["java/lang/Object", "java/lang/Iterable"],
    ),
    ("java/util/Comparator", &["java/lang/Object"]),
    ("java/util/Deque", &["java/lang/Object", "java/util/Queue"]),
    ("java/util/Dictionary", &["java/lang/Object"]),
    (
        "java/util/EnumSet",
        &[
            "java/util/AbstractSet",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/HashMap",
        &[
            "java/util/AbstractMap",
            "java/util/Map",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/HashSet",
        &[
            "java/util/AbstractSet",
            "java/util/Set",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/Hashtable",
        &[
            "java/util/Dictionary",
            "java/util/Map",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/LinkedHashMap",
        &["java/util/HashMap", "java/util/Map"],
    ),
    (
        "java/util/LinkedHashSet",
        &[
            "java/util/HashSet",
            "java/util/Set",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/LinkedList",
        &[
            "java/util/AbstractSequentialList",
            "java/util/List",
            "java/util/Deque",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/List",
        &["java/lang/Object", "java/util/Collection"],
    ),
    ("java/util/Map", &["java/lang/Object"]),
    (
        "java/util/NavigableMap",
        &["java/lang/Object", "java/util/SortedMap"],
    ),
    (
        "java/util/NavigableSet",
        &["java/lang/Object", "java/util/SortedSet"],
    ),
    ("java/util/Properties", &["java/util/Hashtable"]),
    (
        "java/util/Queue",
        &["java/lang/Object", "java/util/Collection"],
    ),
    ("java/util/RandomAccess", &["java/lang/Object"]),
    (
        "java/util/Set",
        &["java/lang/Object", "java/util/Collection"],
    ),
    (
        "java/util/SortedMap",
        &["java/lang/Object", "java/util/Map"],
    ),
    (
        "java/util/SortedSet",
        &["java/lang/Object", "java/util/Set"],
    ),
    ("java/util/Stack", &["java/util/Vector"]),
    (
        "java/util/TreeMap",
        &[
            "java/util/AbstractMap",
            "java/util/NavigableMap",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/TreeSet",
        &[
            "java/util/AbstractSet",
            "java/util/NavigableSet",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/Vector",
        &[
            "java/util/AbstractList",
            "java/util/List",
            "java/util/RandomAccess",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/concurrent/CompletableFuture",
        &[
            "java/lang/Object",
            "java/util/concurrent/Future",
            "java/util/concurrent/CompletionStage",
        ],
    ),
    (
        "java/util/concurrent/CompletionStage",
        &["java/lang/Object"],
    ),
    ("java/util/concurrent/Future", &["java/lang/Object"]),
];

const JAVA23_DIRECT_PARENTS: &[(&str, &[&str])] = &[
    (
        "java/io/Closeable",
        &["java/lang/Object", "java/lang/AutoCloseable"],
    ),
    ("java/io/FileInputStream", &["java/io/InputStream"]),
    (
        "java/io/InputStream",
        &["java/lang/Object", "java/io/Closeable"],
    ),
    ("java/io/InputStreamReader", &["java/io/Reader"]),
    (
        "java/io/Reader",
        &[
            "java/lang/Object",
            "java/lang/Readable",
            "java/io/Closeable",
        ],
    ),
    ("java/io/Serializable", &["java/lang/Object"]),
    (
        "java/lang/AbstractMethodError",
        &["java/lang/IncompatibleClassChangeError"],
    ),
    (
        "java/lang/AbstractStringBuilder",
        &[
            "java/lang/Object",
            "java/lang/Appendable",
            "java/lang/CharSequence",
        ],
    ),
    ("java/lang/Appendable", &["java/lang/Object"]),
    (
        "java/lang/ArithmeticException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/ArrayIndexOutOfBoundsException",
        &["java/lang/IndexOutOfBoundsException"],
    ),
    (
        "java/lang/ArrayStoreException",
        &["java/lang/RuntimeException"],
    ),
    ("java/lang/AssertionError", &["java/lang/Error"]),
    ("java/lang/AutoCloseable", &["java/lang/Object"]),
    (
        "java/lang/Boolean",
        &[
            "java/lang/Object",
            "java/io/Serializable",
            "java/lang/Comparable",
            "java/lang/constant/Constable",
        ],
    ),
    (
        "java/lang/BootstrapMethodError",
        &["java/lang/LinkageError"],
    ),
    (
        "java/lang/Byte",
        &[
            "java/lang/Number",
            "java/lang/Comparable",
            "java/lang/constant/Constable",
        ],
    ),
    ("java/lang/CharSequence", &["java/lang/Object"]),
    (
        "java/lang/Character",
        &[
            "java/lang/Object",
            "java/io/Serializable",
            "java/lang/Comparable",
            "java/lang/constant/Constable",
        ],
    ),
    (
        "java/lang/ClassCastException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/ClassCircularityError",
        &["java/lang/LinkageError"],
    ),
    ("java/lang/ClassFormatError", &["java/lang/LinkageError"]),
    (
        "java/lang/ClassNotFoundException",
        &["java/lang/ReflectiveOperationException"],
    ),
    (
        "java/lang/CloneNotSupportedException",
        &["java/lang/Exception"],
    ),
    ("java/lang/Cloneable", &["java/lang/Object"]),
    ("java/lang/Comparable", &["java/lang/Object"]),
    (
        "java/lang/Double",
        &[
            "java/lang/Number",
            "java/lang/Comparable",
            "java/lang/constant/Constable",
            "java/lang/constant/ConstantDesc",
        ],
    ),
    (
        "java/lang/Enum",
        &[
            "java/lang/Object",
            "java/lang/constant/Constable",
            "java/lang/Comparable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/lang/EnumConstantNotPresentException",
        &["java/lang/RuntimeException"],
    ),
    ("java/lang/Error", &["java/lang/Throwable"]),
    ("java/lang/Exception", &["java/lang/Throwable"]),
    (
        "java/lang/ExceptionInInitializerError",
        &["java/lang/LinkageError"],
    ),
    (
        "java/lang/Float",
        &[
            "java/lang/Number",
            "java/lang/Comparable",
            "java/lang/constant/Constable",
            "java/lang/constant/ConstantDesc",
        ],
    ),
    (
        "java/lang/IllegalAccessError",
        &["java/lang/IncompatibleClassChangeError"],
    ),
    (
        "java/lang/IllegalArgumentException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/IllegalMonitorStateException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/IllegalStateException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/IllegalThreadStateException",
        &["java/lang/IllegalArgumentException"],
    ),
    (
        "java/lang/IncompatibleClassChangeError",
        &["java/lang/LinkageError"],
    ),
    (
        "java/lang/IndexOutOfBoundsException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/InstantiationError",
        &["java/lang/IncompatibleClassChangeError"],
    ),
    (
        "java/lang/Integer",
        &[
            "java/lang/Number",
            "java/lang/Comparable",
            "java/lang/constant/Constable",
            "java/lang/constant/ConstantDesc",
        ],
    ),
    (
        "java/lang/InternalError",
        &["java/lang/VirtualMachineError"],
    ),
    ("java/lang/InterruptedException", &["java/lang/Exception"]),
    ("java/lang/Iterable", &["java/lang/Object"]),
    ("java/lang/LinkageError", &["java/lang/Error"]),
    (
        "java/lang/Long",
        &[
            "java/lang/Number",
            "java/lang/Comparable",
            "java/lang/constant/Constable",
            "java/lang/constant/ConstantDesc",
        ],
    ),
    (
        "java/lang/NegativeArraySizeException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/NoClassDefFoundError",
        &["java/lang/LinkageError"],
    ),
    (
        "java/lang/NoSuchFieldError",
        &["java/lang/IncompatibleClassChangeError"],
    ),
    (
        "java/lang/NoSuchMethodError",
        &["java/lang/IncompatibleClassChangeError"],
    ),
    (
        "java/lang/NullPointerException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/Number",
        &["java/lang/Object", "java/io/Serializable"],
    ),
    (
        "java/lang/NumberFormatException",
        &["java/lang/IllegalArgumentException"],
    ),
    ("java/lang/Object", &[]),
    (
        "java/lang/OutOfMemoryError",
        &["java/lang/VirtualMachineError"],
    ),
    ("java/lang/Readable", &["java/lang/Object"]),
    (
        "java/lang/ReflectiveOperationException",
        &["java/lang/Exception"],
    ),
    ("java/lang/Runnable", &["java/lang/Object"]),
    ("java/lang/RuntimeException", &["java/lang/Exception"]),
    (
        "java/lang/SecurityException",
        &["java/lang/RuntimeException"],
    ),
    (
        "java/lang/Short",
        &[
            "java/lang/Number",
            "java/lang/Comparable",
            "java/lang/constant/Constable",
        ],
    ),
    (
        "java/lang/StackOverflowError",
        &["java/lang/VirtualMachineError"],
    ),
    (
        "java/lang/String",
        &[
            "java/lang/Object",
            "java/io/Serializable",
            "java/lang/Comparable",
            "java/lang/CharSequence",
            "java/lang/constant/Constable",
            "java/lang/constant/ConstantDesc",
        ],
    ),
    (
        "java/lang/StringBuffer",
        &[
            "java/lang/AbstractStringBuilder",
            "java/lang/Appendable",
            "java/io/Serializable",
            "java/lang/Comparable",
            "java/lang/CharSequence",
        ],
    ),
    (
        "java/lang/StringBuilder",
        &[
            "java/lang/AbstractStringBuilder",
            "java/lang/Appendable",
            "java/io/Serializable",
            "java/lang/Comparable",
            "java/lang/CharSequence",
        ],
    ),
    (
        "java/lang/StringIndexOutOfBoundsException",
        &["java/lang/IndexOutOfBoundsException"],
    ),
    ("java/lang/ThreadDeath", &["java/lang/Error"]),
    (
        "java/lang/Throwable",
        &["java/lang/Object", "java/io/Serializable"],
    ),
    (
        "java/lang/TypeNotPresentException",
        &["java/lang/RuntimeException"],
    ),
    ("java/lang/UnknownError", &["java/lang/VirtualMachineError"]),
    (
        "java/lang/UnsatisfiedLinkError",
        &["java/lang/LinkageError"],
    ),
    (
        "java/lang/UnsupportedOperationException",
        &["java/lang/RuntimeException"],
    ),
    ("java/lang/VerifyError", &["java/lang/LinkageError"]),
    ("java/lang/VirtualMachineError", &["java/lang/Error"]),
    ("java/lang/constant/Constable", &["java/lang/Object"]),
    ("java/lang/constant/ConstantDesc", &["java/lang/Object"]),
    (
        "java/math/BigDecimal",
        &["java/lang/Number", "java/lang/Comparable"],
    ),
    ("java/nio/Buffer", &["java/lang/Object"]),
    (
        "java/nio/CharBuffer",
        &[
            "java/nio/Buffer",
            "java/lang/Comparable",
            "java/lang/Appendable",
            "java/lang/CharSequence",
            "java/lang/Readable",
        ],
    ),
    (
        "java/time/Instant",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/LocalDate",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/time/chrono/ChronoLocalDate",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/LocalDateTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/time/chrono/ChronoLocalDateTime",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/LocalTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/OffsetDateTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/OffsetTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/ZonedDateTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/chrono/ChronoZonedDateTime",
            "java/io/Serializable",
        ],
    ),
    (
        "java/time/chrono/ChronoLocalDate",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
        ],
    ),
    (
        "java/time/chrono/ChronoLocalDateTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/time/temporal/TemporalAdjuster",
            "java/lang/Comparable",
        ],
    ),
    (
        "java/time/chrono/ChronoZonedDateTime",
        &[
            "java/lang/Object",
            "java/time/temporal/Temporal",
            "java/lang/Comparable",
        ],
    ),
    (
        "java/time/temporal/Temporal",
        &["java/lang/Object", "java/time/temporal/TemporalAccessor"],
    ),
    ("java/time/temporal/TemporalAccessor", &["java/lang/Object"]),
    ("java/time/temporal/TemporalAdjuster", &["java/lang/Object"]),
    (
        "java/util/AbstractCollection",
        &["java/lang/Object", "java/util/Collection"],
    ),
    (
        "java/util/AbstractList",
        &["java/util/AbstractCollection", "java/util/List"],
    ),
    (
        "java/util/AbstractMap",
        &["java/lang/Object", "java/util/Map"],
    ),
    (
        "java/util/AbstractSequentialList",
        &["java/util/AbstractList"],
    ),
    (
        "java/util/AbstractSet",
        &["java/util/AbstractCollection", "java/util/Set"],
    ),
    (
        "java/util/ArrayDeque",
        &[
            "java/util/AbstractCollection",
            "java/util/Deque",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/ArrayList",
        &[
            "java/util/AbstractList",
            "java/util/List",
            "java/util/RandomAccess",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/Collection",
        &["java/lang/Object", "java/lang/Iterable"],
    ),
    ("java/util/Comparator", &["java/lang/Object"]),
    (
        "java/util/Deque",
        &[
            "java/lang/Object",
            "java/util/Queue",
            "java/util/SequencedCollection",
        ],
    ),
    ("java/util/Dictionary", &["java/lang/Object"]),
    (
        "java/util/EnumSet",
        &[
            "java/util/AbstractSet",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/HashMap",
        &[
            "java/util/AbstractMap",
            "java/util/Map",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/HashSet",
        &[
            "java/util/AbstractSet",
            "java/util/Set",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/Hashtable",
        &[
            "java/util/Dictionary",
            "java/util/Map",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/LinkedHashMap",
        &["java/util/HashMap", "java/util/SequencedMap"],
    ),
    (
        "java/util/LinkedHashSet",
        &[
            "java/util/HashSet",
            "java/util/SequencedSet",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/LinkedList",
        &[
            "java/util/AbstractSequentialList",
            "java/util/List",
            "java/util/Deque",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/List",
        &["java/lang/Object", "java/util/SequencedCollection"],
    ),
    ("java/util/Map", &["java/lang/Object"]),
    (
        "java/util/NavigableMap",
        &["java/lang/Object", "java/util/SortedMap"],
    ),
    (
        "java/util/NavigableSet",
        &["java/lang/Object", "java/util/SortedSet"],
    ),
    ("java/util/Properties", &["java/util/Hashtable"]),
    (
        "java/util/Queue",
        &["java/lang/Object", "java/util/Collection"],
    ),
    ("java/util/RandomAccess", &["java/lang/Object"]),
    (
        "java/util/SequencedCollection",
        &["java/lang/Object", "java/util/Collection"],
    ),
    (
        "java/util/SequencedMap",
        &["java/lang/Object", "java/util/Map"],
    ),
    (
        "java/util/SequencedSet",
        &[
            "java/lang/Object",
            "java/util/SequencedCollection",
            "java/util/Set",
        ],
    ),
    (
        "java/util/Set",
        &["java/lang/Object", "java/util/Collection"],
    ),
    (
        "java/util/SortedMap",
        &["java/lang/Object", "java/util/SequencedMap"],
    ),
    (
        "java/util/SortedSet",
        &[
            "java/lang/Object",
            "java/util/Set",
            "java/util/SequencedSet",
        ],
    ),
    ("java/util/Stack", &["java/util/Vector"]),
    (
        "java/util/TreeMap",
        &[
            "java/util/AbstractMap",
            "java/util/NavigableMap",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/TreeSet",
        &[
            "java/util/AbstractSet",
            "java/util/NavigableSet",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/Vector",
        &[
            "java/util/AbstractList",
            "java/util/List",
            "java/util/RandomAccess",
            "java/lang/Cloneable",
            "java/io/Serializable",
        ],
    ),
    (
        "java/util/concurrent/CompletableFuture",
        &[
            "java/lang/Object",
            "java/util/concurrent/Future",
            "java/util/concurrent/CompletionStage",
        ],
    ),
    (
        "java/util/concurrent/CompletionStage",
        &["java/lang/Object"],
    ),
    ("java/util/concurrent/Future", &["java/lang/Object"]),
];

pub(crate) fn platform_parents(java_release: u16, name: &[u8]) -> Option<&'static [&'static str]> {
    let parents = match java_release {
        8 => JAVA8_DIRECT_PARENTS,
        23 => JAVA23_DIRECT_PARENTS,
        _ => return None,
    };
    let index = parents
        .binary_search_by(|(class, _)| class.as_bytes().cmp(name))
        .ok()?;
    Some(parents[index].1)
}

#[cfg(test)]
mod relation_ordering_tests {
    use super::reference_assignable;
    use crate::report::{ReferenceRelation, ReferenceRelationInput};

    fn site(
        bci: u32,
        relation: ReferenceRelation,
        source: &str,
        target: &str,
    ) -> ReferenceRelationInput {
        ReferenceRelationInput {
            relation,
            bci,
            source: source.to_owned(),
            target: target.to_owned(),
        }
    }

    /// N3 ordering adjudication: `List -> Collection` is a canonical-row pair, so the previous
    /// order — canonical rows consulted first — returned `true` before this exact site's own
    /// walk outcome was read, and the first assertion fails under it (F2's production-reachable
    /// conflict forms: a snapshot-shadowed platform name, an uncertain loader chain, a walk the
    /// environment declines all left the static cast standing against the environment's own
    /// answer). The reversed order pins the facts by strength: a matched negative refuses
    /// decisively, a matched positive carries the answer, and only a site no relation states
    /// falls to the static rows.
    #[test]
    fn matched_site_relation_decides_before_the_canonical_rows() {
        let missing = [site(
            17,
            ReferenceRelation::Missing,
            "java.util.List",
            "java.util.Collection",
        )];
        assert!(
            !reference_assignable(8, "java.util.List", "java.util.Collection", 17, &missing),
            "the walk's Missing at this exact site refuses the table-covered pair"
        );
        let proved = [site(
            17,
            ReferenceRelation::PlatformContract { java_release: 8 },
            "java.util.List",
            "java.util.Collection",
        )];
        assert!(reference_assignable(
            8,
            "java.util.List",
            "java.util.Collection",
            17,
            &proved
        ));
        // A site no relation states keeps the canonical floor: the walk outcome names BCI 17,
        // this query is BCI 21.
        assert!(reference_assignable(
            8,
            "java.util.List",
            "java.util.Collection",
            21,
            &missing
        ));
    }

    /// The canonical rows are release-8 receipts (`widening-row-sources`, the javap
    /// transcription of Corretto 1.8.0_432 `rt.jar`); no other release has one, so a caller
    /// with a live release context gates the floor on 8 and a release-23 site answers from the
    /// structural facts and the proved walk alone.
    #[test]
    fn the_canonical_floor_answers_release_8_only() {
        assert!(reference_assignable(
            8,
            "java.util.List",
            "java.util.Collection",
            3,
            &[]
        ));
        assert!(!reference_assignable(
            23,
            "java.util.List",
            "java.util.Collection",
            3,
            &[]
        ));
        // The structural head is release-independent.
        assert!(reference_assignable(
            23,
            "java.util.List",
            "java.util.List",
            3,
            &[]
        ));
    }
}
